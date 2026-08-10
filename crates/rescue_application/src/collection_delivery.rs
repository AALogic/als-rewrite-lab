use crate::collection_delivery_io::{
    cleanup_owned_staging, copy_and_promote, read_tree, validate_real_directory,
};
use crate::DeliveryCollectionSnapshot;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::path::{Path, PathBuf};

pub const COLLECTION_DELIVERY_SCHEMA_VERSION: &str = "0.1";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CollectionDeliveryRequest {
    pub request_id: String,
    pub snapshot: DeliveryCollectionSnapshot,
    pub destination_root: PathBuf,
    pub previously_delivered_item_ids: Vec<String>,
    pub write_consent: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CollectionDeliveryPlan {
    pub schema_version: String,
    pub request_id: String,
    pub collection_id: String,
    pub collection_revision: u64,
    pub destination_root: PathBuf,
    pub operations: Vec<CollectionDeliveryOperation>,
    pub skipped_item_ids: Vec<String>,
    pub errors: Vec<CollectionDeliveryError>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CollectionDeliveryOperation {
    pub item_id: String,
    pub source_root: PathBuf,
    pub staging_root: PathBuf,
    pub target_root: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CollectionDeliveryResult {
    pub schema_version: String,
    pub request_id: String,
    pub run_status: String,
    pub collection_id: String,
    pub collection_revision: u64,
    pub items: Vec<CollectionDeliveryItemResult>,
    pub skipped_item_ids: Vec<String>,
    pub error_codes: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CollectionDeliveryItemResult {
    pub item_id: String,
    pub status: String,
    pub final_target_root: Option<PathBuf>,
    pub file_count: usize,
    pub total_bytes: u64,
    pub error_code: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CollectionDeliveryError {
    pub error_code: String,
    pub stage: String,
    pub message: String,
}

pub fn plan_collection_delivery(request: &CollectionDeliveryRequest) -> CollectionDeliveryPlan {
    let mut plan = empty_plan(request);
    if let Err(failure) = validate_request(request) {
        plan.errors.push(failure);
        return plan;
    }
    let delivered = request
        .previously_delivered_item_ids
        .iter()
        .cloned()
        .collect::<HashSet<_>>();
    let mut reserved = existing_names(&request.destination_root);
    for (index, item) in request.snapshot.items.iter().enumerate() {
        if delivered.contains(&item.work_item_id) {
            plan.skipped_item_ids.push(item.work_item_id.clone());
            continue;
        }
        match plan_operation(request, item, index, &mut reserved) {
            Ok(operation) => plan.operations.push(operation),
            Err(failure) => plan.errors.push(failure),
        }
    }
    plan
}

pub fn execute_collection_delivery(
    request: &CollectionDeliveryRequest,
    plan: &CollectionDeliveryPlan,
) -> CollectionDeliveryResult {
    if !request.write_consent {
        return failed_result(request, plan, "DELIVERY_CONSENT_REQUIRED");
    }
    if !plan_matches_request(request, plan) {
        return failed_result(request, plan, "DELIVERY_PLAN_INVALID");
    }
    let mut items = Vec::with_capacity(plan.operations.len());
    for operation in &plan.operations {
        items.push(execute_operation(operation));
    }
    let mut error_codes = plan
        .errors
        .iter()
        .map(|failure| failure.error_code.clone())
        .collect::<Vec<_>>();
    error_codes.extend(items.iter().filter_map(|item| item.error_code.clone()));
    let run_status = if error_codes.is_empty() {
        "completed"
    } else if items.iter().any(|item| item.status == "completed") {
        "completed_with_issues"
    } else {
        "failed"
    };
    CollectionDeliveryResult {
        schema_version: COLLECTION_DELIVERY_SCHEMA_VERSION.to_string(),
        request_id: request.request_id.clone(),
        run_status: run_status.to_string(),
        collection_id: request.snapshot.collection_id.clone(),
        collection_revision: request.snapshot.revision,
        items,
        skipped_item_ids: plan.skipped_item_ids.clone(),
        error_codes,
    }
}

fn plan_matches_request(
    request: &CollectionDeliveryRequest,
    plan: &CollectionDeliveryPlan,
) -> bool {
    plan.schema_version == COLLECTION_DELIVERY_SCHEMA_VERSION
        && plan.request_id == request.request_id
        && plan.collection_id == request.snapshot.collection_id
        && plan.collection_revision == request.snapshot.revision
        && plan.destination_root == request.destination_root
        && plan.operations.iter().all(|operation| {
            request.snapshot.items.iter().any(|item| {
                item.work_item_id == operation.item_id
                    && item.target_project_root == operation.source_root
            })
        })
}

fn validate_request(request: &CollectionDeliveryRequest) -> Result<(), CollectionDeliveryError> {
    if request.request_id.trim().is_empty()
        || request.snapshot.collection_id.trim().is_empty()
        || request.snapshot.revision == 0
        || request.snapshot.items.is_empty()
    {
        return Err(delivery_error(
            "DELIVERY_REQUEST_INVALID",
            "plan",
            "The collection delivery request is incomplete.",
        ));
    }
    validate_real_directory(
        &request.destination_root,
        "DELIVERY_DESTINATION_INVALID",
        "plan",
    )
}

fn plan_operation(
    request: &CollectionDeliveryRequest,
    item: &crate::CourierCollectionItem,
    index: usize,
    reserved: &mut HashSet<String>,
) -> Result<CollectionDeliveryOperation, CollectionDeliveryError> {
    validate_real_directory(&item.target_project_root, "DELIVERY_SOURCE_INVALID", "plan")?;
    read_tree(&item.target_project_root)?;
    let base_name = item
        .target_project_root
        .file_name()
        .and_then(|name| name.to_str())
        .filter(|name| !name.trim().is_empty())
        .ok_or_else(|| {
            delivery_error(
                "DELIVERY_SOURCE_INVALID",
                "plan",
                "A source Project name is invalid.",
            )
        })?;
    let target_name = available_name(base_name, reserved);
    reserved.insert(path_key(&target_name));
    let target_root = request.destination_root.join(target_name);
    let staging_root = request.destination_root.join(format!(
        ".als-rescue-delivery-{}-{index:06}.staging",
        safe_component(&request.request_id)
    ));
    if staging_root.exists() {
        return Err(delivery_error(
            "DELIVERY_STAGING_EXISTS",
            "plan",
            "The owned delivery staging directory already exists.",
        ));
    }
    Ok(CollectionDeliveryOperation {
        item_id: item.work_item_id.clone(),
        source_root: item.target_project_root.clone(),
        staging_root,
        target_root,
    })
}

fn execute_operation(operation: &CollectionDeliveryOperation) -> CollectionDeliveryItemResult {
    match copy_and_promote(operation) {
        Ok((file_count, total_bytes)) => CollectionDeliveryItemResult {
            item_id: operation.item_id.clone(),
            status: "completed".to_string(),
            final_target_root: Some(operation.target_root.clone()),
            file_count,
            total_bytes,
            error_code: None,
        },
        Err(failure) => {
            cleanup_owned_staging(&operation.staging_root);
            CollectionDeliveryItemResult {
                item_id: operation.item_id.clone(),
                status: "failed".to_string(),
                final_target_root: None,
                file_count: 0,
                total_bytes: 0,
                error_code: Some(failure.error_code),
            }
        }
    }
}

fn existing_names(root: &Path) -> HashSet<String> {
    std::fs::read_dir(root)
        .ok()
        .into_iter()
        .flatten()
        .filter_map(Result::ok)
        .map(|entry| path_key(&entry.file_name().to_string_lossy()))
        .collect()
}

fn available_name(base: &str, reserved: &HashSet<String>) -> String {
    if !reserved.contains(&path_key(base)) {
        return base.to_string();
    }
    (2_u64..)
        .map(|index| format!("{base} ({index})"))
        .find(|candidate| !reserved.contains(&path_key(candidate)))
        .unwrap_or_else(|| format!("{base} (copy)"))
}

fn safe_component(value: &str) -> String {
    value
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() {
                character
            } else {
                '-'
            }
        })
        .take(48)
        .collect()
}

fn path_key(value: &str) -> String {
    if cfg!(windows) {
        value.to_lowercase()
    } else {
        value.to_string()
    }
}

fn empty_plan(request: &CollectionDeliveryRequest) -> CollectionDeliveryPlan {
    CollectionDeliveryPlan {
        schema_version: COLLECTION_DELIVERY_SCHEMA_VERSION.to_string(),
        request_id: request.request_id.clone(),
        collection_id: request.snapshot.collection_id.clone(),
        collection_revision: request.snapshot.revision,
        destination_root: request.destination_root.clone(),
        operations: Vec::new(),
        skipped_item_ids: Vec::new(),
        errors: Vec::new(),
    }
}

fn failed_result(
    request: &CollectionDeliveryRequest,
    plan: &CollectionDeliveryPlan,
    code: &str,
) -> CollectionDeliveryResult {
    CollectionDeliveryResult {
        schema_version: COLLECTION_DELIVERY_SCHEMA_VERSION.to_string(),
        request_id: request.request_id.clone(),
        run_status: "failed".to_string(),
        collection_id: request.snapshot.collection_id.clone(),
        collection_revision: request.snapshot.revision,
        items: Vec::new(),
        skipped_item_ids: plan.skipped_item_ids.clone(),
        error_codes: vec![code.to_string()],
    }
}

pub(crate) fn delivery_error(code: &str, stage: &str, message: &str) -> CollectionDeliveryError {
    CollectionDeliveryError {
        error_code: code.to_string(),
        stage: stage.to_string(),
        message: message.to_string(),
    }
}
