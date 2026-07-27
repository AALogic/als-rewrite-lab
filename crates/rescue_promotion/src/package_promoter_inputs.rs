use crate::PackagePromotionError;
use rescue_execution::StagingExecutionResult;
use rescue_manifest::ManifestWriteResult;
use rescue_packaging::PackagePlan;
use rescue_rewriter::ALSRewriteResult;
use rescue_validation::PackageValidationResult;
use std::fs;
use std::path::{Component, Path};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum PromotionMode {
    Promote,
    VerifyExisting,
}

pub(crate) fn validate_inputs(
    promotion_id: &str,
    plan: &PackagePlan,
    staging: &StagingExecutionResult,
    rewrite: &ALSRewriteResult,
    validation: &PackageValidationResult,
    manifests: &ManifestWriteResult,
) -> Result<PromotionMode, Vec<PackagePromotionError>> {
    let mut errors = Vec::new();
    if promotion_id.trim().is_empty() {
        errors.push(error(
            "PROMOTION_ID_EMPTY",
            "Promotion ID must not be empty",
            None,
        ));
    }
    validate_statuses(plan, staging, rewrite, validation, manifests, &mut errors);
    validate_identity(plan, staging, rewrite, validation, manifests, &mut errors);
    validate_paths(plan, staging, manifests, &mut errors);
    if !errors.is_empty() {
        return Err(errors);
    }
    match (
        staging.staging_root.exists(),
        plan.target_project_root.exists(),
    ) {
        (true, false) => {
            if let Err(error) = same_filesystem(&staging.staging_root, &plan.target_project_root) {
                Err(vec![error])
            } else {
                Ok(PromotionMode::Promote)
            }
        }
        (false, true) => Ok(PromotionMode::VerifyExisting),
        (true, true) => Err(vec![error(
            "PROMOTION_TARGET_CONFLICT",
            "Staging and final target both exist; nothing will be overwritten",
            Some(&plan.target_project_root),
        )]),
        (false, false) => Err(vec![error(
            "PROMOTION_SOURCE_MISSING",
            "Neither staging nor a previously promoted target exists",
            Some(&staging.staging_root),
        )]),
    }
}

fn validate_statuses(
    plan: &PackagePlan,
    staging: &StagingExecutionResult,
    rewrite: &ALSRewriteResult,
    validation: &PackageValidationResult,
    manifests: &ManifestWriteResult,
    errors: &mut Vec<PackagePromotionError>,
) {
    if plan.plan_status != "ready_for_laboratory_execution"
        || staging.execution_status != "staging_complete"
        || rewrite.rewrite_status != "rewrite_complete"
        || validation.validation_status != "validation_passed"
        || manifests.write_status != "manifests_written"
        || !plan.errors.is_empty()
        || !staging.errors.is_empty()
        || !rewrite.errors.is_empty()
        || !validation.errors.is_empty()
        || !manifests.errors.is_empty()
    {
        errors.push(error(
            "PROMOTION_PIPELINE_NOT_COMPLETE",
            "Every prior package stage must have completed successfully",
            None,
        ));
    }
    if manifests.package_manifest_sha256.is_none()
        || manifests.package_manifest_size.is_none()
        || manifests.private_ledger_sha256.is_none()
        || manifests.private_ledger_size.is_none()
    {
        errors.push(error(
            "PROMOTION_MANIFEST_EVIDENCE_INCOMPLETE",
            "Manifest hashes and sizes are required before promotion",
            None,
        ));
    }
}

fn validate_identity(
    plan: &PackagePlan,
    staging: &StagingExecutionResult,
    rewrite: &ALSRewriteResult,
    validation: &PackageValidationResult,
    manifests: &ManifestWriteResult,
    errors: &mut Vec<PackagePromotionError>,
) {
    if staging.metadata.plan_id != plan.metadata.plan_id
        || rewrite.metadata.plan_id != plan.metadata.plan_id
        || validation.metadata.plan_id != plan.metadata.plan_id
        || manifests.metadata.plan_id != plan.metadata.plan_id
        || rewrite.metadata.execution_id != staging.metadata.execution_id
        || validation.metadata.execution_id != staging.metadata.execution_id
        || manifests.metadata.execution_id != staging.metadata.execution_id
        || validation.metadata.rewrite_id != rewrite.metadata.rewrite_id
        || manifests.metadata.rewrite_id != rewrite.metadata.rewrite_id
        || manifests.metadata.validation_id != validation.metadata.validation_id
    {
        errors.push(error(
            "PROMOTION_RUN_IDENTITY_MISMATCH",
            "Promotion inputs do not describe one pipeline run",
            None,
        ));
    }
}

fn validate_paths(
    plan: &PackagePlan,
    staging: &StagingExecutionResult,
    manifests: &ManifestWriteResult,
    errors: &mut Vec<PackagePromotionError>,
) {
    if !safe_absolute(&staging.staging_root)
        || !safe_absolute(&plan.target_project_root)
        || staging.staging_root == plan.target_project_root
    {
        errors.push(error(
            "PROMOTION_ROOT_UNSAFE",
            "Staging and final roots must be distinct safe absolute paths",
            None,
        ));
    }
    if !safe_relative(&manifests.package_manifest_relative_path) {
        errors.push(error(
            "PROMOTION_MANIFEST_PATH_UNSAFE",
            "Portable manifest path must be safe and relative",
            Some(&manifests.package_manifest_relative_path),
        ));
    }
    if manifests
        .private_ledger_path
        .starts_with(&staging.staging_root)
        || manifests
            .private_ledger_path
            .starts_with(&plan.target_project_root)
    {
        errors.push(error(
            "PROMOTION_PRIVATE_LEDGER_INSIDE_PACKAGE",
            "Private ledger must remain outside staging and final package",
            Some(&manifests.private_ledger_path),
        ));
    }
    let Some(parent) = plan.target_project_root.parent() else {
        errors.push(error(
            "PROMOTION_TARGET_PARENT_INVALID",
            "Final target has no parent directory",
            Some(&plan.target_project_root),
        ));
        return;
    };
    match fs::symlink_metadata(parent) {
        Ok(metadata) if metadata.file_type().is_dir() && !metadata.file_type().is_symlink() => {}
        _ => errors.push(error(
            "PROMOTION_TARGET_PARENT_INVALID",
            "Final target parent must already be a non-symlink directory",
            Some(parent),
        )),
    }
}

#[cfg(unix)]
fn same_filesystem(staging: &Path, target: &Path) -> Result<(), PackagePromotionError> {
    use std::os::unix::fs::MetadataExt;

    let target_parent = target.parent().ok_or_else(|| {
        error(
            "PROMOTION_TARGET_PARENT_INVALID",
            "Final target has no parent directory",
            Some(target),
        )
    })?;
    let staging_device = fs::metadata(staging)
        .map_err(|_| {
            error(
                "PROMOTION_SOURCE_INVALID",
                "Cannot inspect staging",
                Some(staging),
            )
        })?
        .dev();
    let target_device = fs::metadata(target_parent)
        .map_err(|_| {
            error(
                "PROMOTION_TARGET_PARENT_INVALID",
                "Cannot inspect final target parent",
                Some(target_parent),
            )
        })?
        .dev();
    if staging_device != target_device {
        return Err(error(
            "PROMOTION_CROSS_DEVICE",
            "Atomic directory promotion requires staging and target on one filesystem",
            Some(target),
        ));
    }
    Ok(())
}

#[cfg(not(unix))]
fn same_filesystem(_staging: &Path, _target: &Path) -> Result<(), PackagePromotionError> {
    Ok(())
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

fn error(code: &str, message: &str, path: Option<&Path>) -> PackagePromotionError {
    crate::package_promoter_result::error(code, message, path)
}
