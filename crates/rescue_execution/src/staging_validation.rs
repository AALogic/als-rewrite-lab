use crate::{StagingExecutionError, StagingExecutionRequest};
use rescue_packaging::{CopyOperation, PackagePlan};
use std::collections::BTreeSet;
use std::path::{Component, Path};

pub(crate) fn validate_request(
    request: &StagingExecutionRequest,
    plan: &PackagePlan,
) -> Vec<StagingExecutionError> {
    let mut errors = Vec::new();
    if !matches!(
        plan.plan_status.as_str(),
        "ready_copy_only"
            | "ready_for_laboratory_execution"
            | "ready_current_paths_complete"
            | "ready_current_paths_incomplete"
    ) || !plan.errors.is_empty()
        || plan
            .unresolved_requirements
            .iter()
            .any(|item| item.blocks_execution)
    {
        errors.push(error(
            "STAGING_PLAN_NOT_READY",
            "Only a complete ready plan may be executed",
            None,
        ));
    }
    if !safe_absolute_root(&request.staging_root) {
        errors.push(root_error(
            "STAGING_ROOT_UNSAFE",
            "Staging root must be an absolute path without traversal",
            &request.staging_root,
        ));
    } else if request.staging_root.exists() {
        errors.push(root_error(
            "STAGING_ROOT_EXISTS",
            "Staging root must not exist before execution",
            &request.staging_root,
        ));
    }
    if request.staging_root == plan.target_project_root {
        errors.push(root_error(
            "STAGING_EQUALS_FINAL_TARGET",
            "Staging and final target roots must be different",
            &request.staging_root,
        ));
    }
    errors.extend(crate::staging_directories::validate_directories(
        plan.metadata.directory_operation_count,
        &plan.directory_operations,
    ));
    validate_copy_operations(&plan.copy_operations, &mut errors);
    errors
}

fn validate_copy_operations(operations: &[CopyOperation], errors: &mut Vec<StagingExecutionError>) {
    let mut ids = BTreeSet::new();
    let mut targets = BTreeSet::new();
    if operations.is_empty() {
        errors.push(error(
            "STAGING_PLAN_EMPTY",
            "A staging plan must contain copy operations",
            None,
        ));
    }
    for operation in operations {
        if !ids.insert(operation.operation_id.as_str()) {
            errors.push(error(
                "STAGING_DUPLICATE_OPERATION_ID",
                "Copy operation IDs must be unique",
                Some(operation),
            ));
        }
        if !safe_relative_target(&operation.target_relative_path) {
            errors.push(error(
                "STAGING_TARGET_PATH_UNSAFE",
                "Copy target must be a non-empty safe relative path",
                Some(operation),
            ));
        } else if !targets.insert(operation.target_relative_path.clone()) {
            errors.push(error(
                "STAGING_DUPLICATE_TARGET",
                "Each copy operation must have a unique target",
                Some(operation),
            ));
        }
        validate_copy_policy(operation, errors);
    }
}

fn validate_copy_policy(operation: &CopyOperation, errors: &mut Vec<StagingExecutionError>) {
    if operation.collision_policy != "fail_if_exists" {
        errors.push(error(
            "STAGING_COLLISION_POLICY_UNSUPPORTED",
            "Executor supports only fail_if_exists",
            Some(operation),
        ));
    }
    let policy_valid = match operation.verification_policy.as_str() {
        rescue_packaging::VERIFY_SHA256_AND_SIZE => operation.expected_source_sha256.is_some(),
        rescue_packaging::VERIFY_STABLE_SOURCE_AND_SIZE => {
            operation.expected_source_sha256.is_none() && operation.content_id.is_none()
        }
        _ => false,
    };
    if !policy_valid {
        errors.push(error(
            "STAGING_VERIFICATION_POLICY_INVALID",
            "Copy operation verification policy and evidence are inconsistent",
            Some(operation),
        ));
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

fn error(code: &str, message: &str, operation: Option<&CopyOperation>) -> StagingExecutionError {
    crate::staging_executor_impl::execution_error(
        code,
        message.to_string(),
        operation.map(|item| item.operation_id.clone()),
        operation.map(|item| item.target_relative_path.clone()),
    )
}

fn root_error(code: &str, message: &str, path: &Path) -> StagingExecutionError {
    crate::staging_executor_impl::execution_error(
        code,
        message.to_string(),
        None,
        Some(path.to_path_buf()),
    )
}
