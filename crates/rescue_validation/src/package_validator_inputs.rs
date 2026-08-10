use crate::{PackageValidationError, PackageValidationRequest};
use rescue_execution::StagingExecutionResult;
use rescue_packaging::PackagePlan;
use rescue_rewriter::ALSRewriteResult;
use std::collections::BTreeSet;
use std::fs;
use std::path::{Component, Path};

pub(crate) fn validate_inputs(
    request: &PackageValidationRequest,
    plan: &PackagePlan,
    staging: &StagingExecutionResult,
    rewrite: &ALSRewriteResult,
) -> Vec<PackageValidationError> {
    let mut errors = Vec::new();
    if !matches!(
        plan.plan_status.as_str(),
        "ready_for_laboratory_execution"
            | "ready_current_paths_complete"
            | "ready_current_paths_incomplete"
    ) || !plan.errors.is_empty()
        || plan
            .unresolved_requirements
            .iter()
            .any(|item| item.blocks_execution)
    {
        errors.push(error(
            "VALIDATION_PLAN_NOT_READY",
            "Validation requires a complete laboratory plan",
            None,
        ));
    }
    if staging.execution_status != "staging_complete" || !staging.errors.is_empty() {
        errors.push(error(
            "VALIDATION_STAGING_NOT_COMPLETE",
            "Validation requires completed staging",
            Some(&staging.staging_root),
        ));
    }
    let expected_rewrite_status = if plan.rewrite_operations.is_empty() {
        "not_required"
    } else {
        "rewrite_complete"
    };
    if rewrite.rewrite_status != expected_rewrite_status || !rewrite.errors.is_empty() {
        errors.push(error(
            "VALIDATION_REWRITE_NOT_COMPLETE",
            "Validation requires the expected successful rewrite result",
            None,
        ));
    }
    validate_identity(request, plan, staging, rewrite, &mut errors);
    validate_roots(request, plan, &mut errors);
    validate_operation_sets(plan, staging, rewrite, &mut errors);
    validate_system_dependencies(plan, &mut errors);
    errors
}

fn validate_system_dependencies(plan: &PackagePlan, errors: &mut Vec<PackageValidationError>) {
    if plan.metadata.system_dependency_count != plan.system_dependencies.len() {
        errors.push(error(
            "VALIDATION_SYSTEM_DEPENDENCY_COUNT_MISMATCH",
            "System dependency metadata must match the planned dispositions",
            None,
        ));
    }
    let mut ids = BTreeSet::new();
    for dependency in &plan.system_dependencies {
        let contract_valid = ids.insert(dependency.required_asset_id.as_str())
            && dependency.source_category == "ableton_core_library"
            && dependency.package_action == "leave_system_managed"
            && dependency.portability_status == "portable_risk";
        if !contract_valid {
            errors.push(error(
                "VALIDATION_SYSTEM_DEPENDENCY_CONTRACT_INVALID",
                "System dependency records must be unique confirmed Core Library dispositions",
                None,
            ));
            break;
        }
        if plan.rewrite_operations.iter().any(|operation| {
            operation.required_asset_id == dependency.required_asset_id
                || dependency.als_ref_ids.contains(&operation.als_ref_id)
        }) {
            errors.push(error(
                "VALIDATION_SYSTEM_DEPENDENCY_REWRITE_FORBIDDEN",
                "System-managed references must remain unchanged",
                None,
            ));
        }
        if plan.copy_operations.iter().any(|operation| {
            dependency
                .observed_source_paths
                .contains(&operation.source_path)
        }) {
            errors.push(error(
                "VALIDATION_SYSTEM_DEPENDENCY_COPY_FORBIDDEN",
                "System-managed files must not be copied by the default package policy",
                None,
            ));
        }
    }
}

fn validate_identity(
    request: &PackageValidationRequest,
    plan: &PackagePlan,
    staging: &StagingExecutionResult,
    rewrite: &ALSRewriteResult,
    errors: &mut Vec<PackageValidationError>,
) {
    if request.staging_root != staging.staging_root {
        errors.push(error(
            "VALIDATION_STAGING_ROOT_MISMATCH",
            "Request and staging result identify different roots",
            Some(&request.staging_root),
        ));
    }
    if staging.metadata.plan_id != plan.metadata.plan_id
        || rewrite.metadata.plan_id != plan.metadata.plan_id
        || rewrite.metadata.execution_id != staging.metadata.execution_id
        || staging.metadata.source_als_hash != plan.metadata.source_als_hash
        || rewrite.metadata.source_als_hash != plan.metadata.source_als_hash
    {
        errors.push(error(
            "VALIDATION_CONTRACT_IDENTITY_MISMATCH",
            "Plan, staging, and rewrite results do not describe one run",
            None,
        ));
    }
    if rewrite.staged_als_relative_path != staging.staged_als_relative_path {
        errors.push(error(
            "VALIDATION_ALS_PATH_MISMATCH",
            "Staging and rewrite results identify different ALS paths",
            Some(&rewrite.staged_als_relative_path),
        ));
    }
}

fn validate_roots(
    request: &PackageValidationRequest,
    plan: &PackagePlan,
    errors: &mut Vec<PackageValidationError>,
) {
    if !safe_absolute(&request.staging_root) {
        errors.push(error(
            "VALIDATION_STAGING_ROOT_UNSAFE",
            "Staging root must be an absolute path without traversal",
            Some(&request.staging_root),
        ));
    } else {
        match fs::symlink_metadata(&request.staging_root) {
            Ok(metadata) if metadata.file_type().is_dir() && !metadata.file_type().is_symlink() => {
            }
            _ => errors.push(error(
                "VALIDATION_STAGING_ROOT_INVALID",
                "Staging root must be an existing non-symlink directory",
                Some(&request.staging_root),
            )),
        }
    }
    if plan.target_project_root.exists() {
        errors.push(error(
            "VALIDATION_FINAL_TARGET_EXISTS",
            "Final target must remain absent before promotion",
            Some(&plan.target_project_root),
        ));
    }
}

fn validate_operation_sets(
    plan: &PackagePlan,
    staging: &StagingExecutionResult,
    rewrite: &ALSRewriteResult,
    errors: &mut Vec<PackageValidationError>,
) {
    let planned_directory_ids: BTreeSet<_> = plan
        .directory_operations
        .iter()
        .map(|operation| operation.operation_id.as_str())
        .collect();
    let staged_directory_ids: BTreeSet<_> = staging
        .directory_records
        .iter()
        .filter(|record| record.operation_status == "created")
        .map(|record| record.operation_id.as_str())
        .collect();
    if planned_directory_ids.len() != plan.directory_operations.len()
        || staged_directory_ids != planned_directory_ids
        || staging.metadata.completed_directory_count != plan.directory_operations.len()
    {
        errors.push(error(
            "VALIDATION_DIRECTORY_RECORD_MISMATCH",
            "Staging records are not one-to-one with planned directories",
            None,
        ));
    }
    let planned_copy_ids: BTreeSet<_> = plan
        .copy_operations
        .iter()
        .map(|operation| operation.operation_id.as_str())
        .collect();
    let staged_copy_ids: BTreeSet<_> = staging
        .copy_records
        .iter()
        .filter(|record| record.operation_status == "copied_and_verified")
        .map(|record| record.operation_id.as_str())
        .collect();
    if planned_copy_ids.len() != plan.copy_operations.len()
        || staged_copy_ids != planned_copy_ids
        || staging.metadata.completed_copy_count != plan.copy_operations.len()
    {
        errors.push(error(
            "VALIDATION_COPY_RECORD_MISMATCH",
            "Staging records are not one-to-one with planned copies",
            None,
        ));
    }
    let planned_rewrite_ids: BTreeSet<_> = plan
        .rewrite_operations
        .iter()
        .map(|operation| operation.operation_id.as_str())
        .collect();
    let completed_rewrite_ids: BTreeSet<_> = rewrite
        .operation_records
        .iter()
        .filter(|record| record.operation_status == "rewritten_and_verified")
        .map(|record| record.operation_id.as_str())
        .collect();
    if planned_rewrite_ids.len() != plan.rewrite_operations.len()
        || completed_rewrite_ids != planned_rewrite_ids
        || rewrite.metadata.completed_operation_count != plan.rewrite_operations.len()
    {
        errors.push(error(
            "VALIDATION_REWRITE_RECORD_MISMATCH",
            "Rewrite records are not one-to-one with planned operations",
            None,
        ));
    }
}

fn safe_absolute(path: &Path) -> bool {
    path.is_absolute()
        && !path
            .components()
            .any(|component| matches!(component, Component::ParentDir))
}

fn error(code: &str, message: &str, path: Option<&Path>) -> PackageValidationError {
    crate::package_validator_result::error(code, message, None, path)
}
