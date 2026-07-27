use crate::{
    CopyExecutionRecord, StagingExecutionError, StagingExecutionMetadata, StagingExecutionRequest,
    StagingExecutionResult, STAGING_EXECUTION_SCHEMA_VERSION, STAGING_EXECUTOR_VERSION,
};
use rescue_packaging::{CopyOperation, PackagePlan};
use std::collections::BTreeSet;
use std::fs;
use std::path::{Component, Path, PathBuf};

pub(crate) fn execute_staging_impl(
    request: &StagingExecutionRequest,
    plan: &PackagePlan,
) -> StagingExecutionResult {
    let mut errors = validate_request(request, plan);
    if !errors.is_empty() {
        return result(request, plan, Vec::new(), "rejected", errors);
    }
    if let Err(error) = fs::create_dir(&request.staging_root) {
        errors.push(execution_error(
            "STAGING_ROOT_CREATE_FAILED",
            format!("Cannot create a new staging root: {error}"),
            None,
            Some(request.staging_root.clone()),
        ));
        return result(request, plan, Vec::new(), "copy_failed", errors);
    }

    let mut records = Vec::new();
    for (index, operation) in plan.copy_operations.iter().enumerate() {
        match crate::staging_copy::copy_verified(&request.staging_root, operation, index) {
            Ok(outcome) => records.push(record(
                operation,
                Some(outcome.digest),
                Some(outcome.size),
                "copied_and_verified",
            )),
            Err(failure) => {
                records.push(record(operation, None, None, "failed"));
                errors.push(execution_error(
                    failure.code,
                    failure.message,
                    Some(operation.operation_id.clone()),
                    failure.path,
                ));
                return result(request, plan, records, "copy_failed", errors);
            }
        }
    }
    result(request, plan, records, "staging_complete", errors)
}

fn validate_request(
    request: &StagingExecutionRequest,
    plan: &PackagePlan,
) -> Vec<StagingExecutionError> {
    let mut errors = Vec::new();
    if !matches!(
        plan.plan_status.as_str(),
        "ready_copy_only" | "ready_for_laboratory_execution"
    ) || !plan.errors.is_empty()
        || !plan.unresolved_requirements.is_empty()
    {
        errors.push(execution_error(
            "STAGING_PLAN_NOT_READY",
            "Only a complete ready plan may be executed".to_string(),
            None,
            None,
        ));
    }
    if !safe_absolute_root(&request.staging_root) {
        errors.push(execution_error(
            "STAGING_ROOT_UNSAFE",
            "Staging root must be an absolute path without traversal".to_string(),
            None,
            Some(request.staging_root.clone()),
        ));
    } else if request.staging_root.exists() {
        errors.push(execution_error(
            "STAGING_ROOT_EXISTS",
            "Staging root must not exist before execution".to_string(),
            None,
            Some(request.staging_root.clone()),
        ));
    }
    if request.staging_root == plan.target_project_root {
        errors.push(execution_error(
            "STAGING_EQUALS_FINAL_TARGET",
            "Staging and final target roots must be different".to_string(),
            None,
            Some(request.staging_root.clone()),
        ));
    }
    validate_operations(&plan.copy_operations, &mut errors);
    errors
}

fn validate_operations(operations: &[CopyOperation], errors: &mut Vec<StagingExecutionError>) {
    let mut ids = BTreeSet::new();
    let mut targets = BTreeSet::new();
    if operations.is_empty() {
        errors.push(execution_error(
            "STAGING_PLAN_EMPTY",
            "A staging plan must contain copy operations".to_string(),
            None,
            None,
        ));
    }
    for operation in operations {
        if !ids.insert(operation.operation_id.as_str()) {
            errors.push(execution_error(
                "STAGING_DUPLICATE_OPERATION_ID",
                "Copy operation IDs must be unique".to_string(),
                Some(operation.operation_id.clone()),
                None,
            ));
        }
        if !safe_relative_target(&operation.target_relative_path) {
            errors.push(execution_error(
                "STAGING_TARGET_PATH_UNSAFE",
                "Copy target must be a non-empty safe relative path".to_string(),
                Some(operation.operation_id.clone()),
                Some(operation.target_relative_path.clone()),
            ));
        } else if !targets.insert(operation.target_relative_path.clone()) {
            errors.push(execution_error(
                "STAGING_DUPLICATE_TARGET",
                "Each copy operation must have a unique target".to_string(),
                Some(operation.operation_id.clone()),
                Some(operation.target_relative_path.clone()),
            ));
        }
        if operation.collision_policy != "fail_if_exists" {
            errors.push(execution_error(
                "STAGING_COLLISION_POLICY_UNSUPPORTED",
                "Executor supports only fail_if_exists".to_string(),
                Some(operation.operation_id.clone()),
                None,
            ));
        }
    }
}

fn safe_absolute_root(path: &Path) -> bool {
    path.is_absolute()
        && !path
            .components()
            .any(|component| matches!(component, Component::ParentDir))
}

fn safe_relative_target(path: &Path) -> bool {
    !path.as_os_str().is_empty()
        && !path.is_absolute()
        && path
            .components()
            .all(|component| matches!(component, Component::Normal(_)))
}

fn record(
    operation: &CopyOperation,
    digest: Option<String>,
    size: Option<u64>,
    status: &str,
) -> CopyExecutionRecord {
    CopyExecutionRecord {
        operation_id: operation.operation_id.clone(),
        operation_kind: operation.operation_kind.clone(),
        source_path: operation.source_path.clone(),
        target_relative_path: operation.target_relative_path.clone(),
        expected_sha256: operation.expected_source_sha256.clone(),
        observed_sha256: digest,
        expected_size: operation.expected_source_size,
        observed_size: size,
        operation_status: status.to_string(),
    }
}

fn result(
    request: &StagingExecutionRequest,
    plan: &PackagePlan,
    records: Vec<CopyExecutionRecord>,
    status: &str,
    errors: Vec<StagingExecutionError>,
) -> StagingExecutionResult {
    let completed = records
        .iter()
        .filter(|record| record.operation_status == "copied_and_verified")
        .count();
    StagingExecutionResult {
        metadata: StagingExecutionMetadata {
            executor_version: STAGING_EXECUTOR_VERSION.to_string(),
            execution_schema_version: STAGING_EXECUTION_SCHEMA_VERSION.to_string(),
            execution_id: request.execution_id.clone(),
            plan_id: plan.metadata.plan_id.clone(),
            source_als_hash: plan.metadata.source_als_hash.clone(),
            planned_copy_count: plan.copy_operations.len(),
            completed_copy_count: completed,
            warning_count: 0,
            error_count: errors.len(),
        },
        staging_root: request.staging_root.clone(),
        staged_als_relative_path: plan.source_als.target_relative_path.clone(),
        copy_records: records,
        execution_status: status.to_string(),
        warnings: Vec::new(),
        errors,
    }
}

fn execution_error(
    code: &str,
    message: String,
    operation_id: Option<String>,
    path: Option<PathBuf>,
) -> StagingExecutionError {
    StagingExecutionError {
        error_code: code.to_string(),
        message,
        operation_id,
        path,
    }
}
