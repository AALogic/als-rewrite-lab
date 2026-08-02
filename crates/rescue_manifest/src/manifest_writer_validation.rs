use crate::{ManifestWriteError, ManifestWriteRequest};
use rescue_execution::StagingExecutionResult;
use rescue_packaging::PackagePlan;
use rescue_rewriter::ALSRewriteResult;
use rescue_validation::PackageValidationResult;
use std::collections::BTreeSet;
use std::path::{Component, Path};

pub(crate) fn validate_inputs(
    request: &ManifestWriteRequest,
    plan: &PackagePlan,
    staging: &StagingExecutionResult,
    rewrite: &ALSRewriteResult,
    validation: &PackageValidationResult,
) -> Vec<ManifestWriteError> {
    let mut errors = Vec::new();
    if request.manifest_id.trim().is_empty() {
        errors.push(error(
            "MANIFEST_ID_EMPTY",
            "Manifest ID must not be empty",
            None,
        ));
    }
    if validation.validation_status != "validation_passed"
        || !validation.errors.is_empty()
        || validation.metadata.verified_directory_count != plan.directory_operations.len()
        || validation.metadata.verified_file_count != plan.copy_operations.len()
        || validation.metadata.verified_rewrite_count != plan.rewrite_operations.len()
    {
        errors.push(error(
            "MANIFEST_VALIDATION_NOT_PASSED",
            "Manifests require a fully passed validation result",
            None,
        ));
    }
    if validation.metadata.plan_id != plan.metadata.plan_id
        || validation.metadata.execution_id != staging.metadata.execution_id
        || validation.metadata.rewrite_id != rewrite.metadata.rewrite_id
        || validation.staging_root != staging.staging_root
    {
        errors.push(error(
            "MANIFEST_RUN_IDENTITY_MISMATCH",
            "Plan and run results do not identify one validated package",
            None,
        ));
    }
    validate_paths(request, plan, staging, &mut errors);
    errors
}

fn validate_paths(
    request: &ManifestWriteRequest,
    plan: &PackagePlan,
    staging: &StagingExecutionResult,
    errors: &mut Vec<ManifestWriteError>,
) {
    if !safe_absolute(&request.private_ledger_path)
        || request
            .private_ledger_path
            .starts_with(&staging.staging_root)
    {
        errors.push(error(
            "PRIVATE_LEDGER_PATH_UNSAFE",
            "Private ledger must be an absolute path outside the portable package",
            Some(&request.private_ledger_path),
        ));
    }
    if !safe_relative(&request.package_manifest_relative_path) {
        errors.push(error(
            "PACKAGE_MANIFEST_PATH_UNSAFE",
            "Package manifest path must be safe and relative",
            Some(&request.package_manifest_relative_path),
        ));
    }
    let planned_targets: BTreeSet<_> = plan
        .copy_operations
        .iter()
        .map(|operation| operation.target_relative_path.as_path())
        .collect();
    if planned_targets.contains(request.package_manifest_relative_path.as_path()) {
        errors.push(error(
            "PACKAGE_MANIFEST_TARGET_COLLISION",
            "Package manifest must not replace a planned package file",
            Some(&request.package_manifest_relative_path),
        ));
    }
    if plan.target_project_root.exists() {
        errors.push(error(
            "MANIFEST_FINAL_TARGET_EXISTS",
            "Final target appeared after validation",
            Some(&plan.target_project_root),
        ));
    }
}

fn error(code: &str, message: &str, path: Option<&Path>) -> ManifestWriteError {
    crate::manifest_writer_result::error(code, message, path)
}

fn safe_absolute(path: &Path) -> bool {
    path.is_absolute()
        && !path
            .components()
            .any(|component| matches!(component, Component::ParentDir))
}

fn safe_relative(path: &Path) -> bool {
    !path.as_os_str().is_empty()
        && !path.is_absolute()
        && path
            .components()
            .all(|component| matches!(component, Component::Normal(_)))
}
