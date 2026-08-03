use crate::{PackagePromotionRequest, PackagePromotionResult};
use rescue_execution::StagingExecutionResult;
use rescue_manifest::ManifestWriteResult;
use rescue_packaging::PackagePlan;
use rescue_rewriter::ALSRewriteResult;
use rescue_validation::PackageValidationResult;
#[cfg(unix)]
use std::fs;

pub(crate) struct DirectoryMoveFailure {
    pub code: &'static str,
    pub message: String,
}

pub(crate) fn promote_validated_package_impl(
    request: &PackagePromotionRequest,
    plan: &PackagePlan,
    staging: &StagingExecutionResult,
    rewrite: &ALSRewriteResult,
    validation: &PackageValidationResult,
    manifests: &ManifestWriteResult,
) -> PackagePromotionResult {
    let context = crate::package_promoter_result::PromotionContext {
        request,
        plan,
        staging,
        rewrite,
        validation,
        manifests,
    };
    let mode = match crate::package_promoter_inputs::validate_inputs(
        &request.promotion_id,
        plan,
        staging,
        rewrite,
        validation,
        manifests,
    ) {
        Ok(mode) => mode,
        Err(errors) => {
            return crate::package_promoter_result::result(
                &context,
                Vec::new(),
                "promotion_rejected",
                errors,
            )
        }
    };
    if let Err(error) = crate::package_promoter_files::verify_original_source(plan) {
        return failed(&context, Vec::new(), error);
    }
    if let Err(error) = crate::package_promoter_files::verify_private_ledger(manifests) {
        return failed(&context, Vec::new(), error);
    }
    let root = match mode {
        crate::package_promoter_inputs::PromotionMode::Promote => &staging.staging_root,
        crate::package_promoter_inputs::PromotionMode::VerifyExisting => &plan.target_project_root,
    };
    let verified =
        crate::package_promoter_files::verify_package_root(root, plan, validation, manifests);
    if !verified.errors.is_empty() {
        return crate::package_promoter_result::result(
            &context,
            verified.records,
            "promotion_rejected",
            verified.errors,
        );
    }
    if mode == crate::package_promoter_inputs::PromotionMode::VerifyExisting {
        return crate::package_promoter_result::result(
            &context,
            verified.records,
            "already_promoted_verified",
            Vec::new(),
        );
    }
    promote_directory(&context, verified.records)
}

fn promote_directory(
    context: &crate::package_promoter_result::PromotionContext<'_>,
    preflight_records: Vec<crate::PromotedFileRecord>,
) -> PackagePromotionResult {
    if context.plan.target_project_root.exists() {
        return failed(
            context,
            preflight_records,
            crate::package_promoter_result::error(
                "PROMOTION_TARGET_CONFLICT",
                "Final target appeared before promotion",
                Some(&context.plan.target_project_root),
            ),
        );
    }
    if let Err(rename_error) = move_directory(
        &context.staging.staging_root,
        &context.plan.target_project_root,
    ) {
        return failed(
            context,
            preflight_records,
            crate::package_promoter_result::error(
                rename_error.code,
                rename_error.message,
                Some(&context.plan.target_project_root),
            ),
        );
    }
    let after = crate::package_promoter_files::verify_package_root(
        &context.plan.target_project_root,
        context.plan,
        context.validation,
        context.manifests,
    );
    if after.errors.is_empty() {
        if let Err(sync_error) = sync_target_parent(&context.plan.target_project_root) {
            return rollback_after_failure(context, after.records, sync_error);
        }
        return crate::package_promoter_result::result(
            context,
            after.records,
            "promoted_ready_for_manual_check",
            Vec::new(),
        );
    }
    let message = "Promoted package failed post-rename verification".to_string();
    rollback_after_failure(
        context,
        after.records,
        crate::package_promoter_result::error(
            "PROMOTION_POSTCHECK_FAILED",
            message,
            Some(&context.plan.target_project_root),
        ),
    )
}

fn rollback_after_failure(
    context: &crate::package_promoter_result::PromotionContext<'_>,
    records: Vec<crate::PromotedFileRecord>,
    original_error: crate::PackagePromotionError,
) -> PackagePromotionResult {
    let rollback = move_directory(
        &context.plan.target_project_root,
        &context.staging.staging_root,
    );
    let mut errors = vec![original_error];
    let status = if let Err(rollback_error) = rollback {
        errors.push(crate::package_promoter_result::error(
            "PROMOTION_ROLLBACK_FAILED",
            format!(
                "Cannot restore staging after failed promotion: {}",
                rollback_error.message
            ),
            Some(&context.staging.staging_root),
        ));
        "promotion_failed_rollback_failed"
    } else {
        "promotion_failed_rolled_back"
    };
    crate::package_promoter_result::result(context, records, status, errors)
}

#[cfg(unix)]
fn move_directory(
    source: &std::path::Path,
    target: &std::path::Path,
) -> Result<(), DirectoryMoveFailure> {
    fs::rename(source, target).map_err(|error| DirectoryMoveFailure {
        code: "PROMOTION_RENAME_FAILED",
        message: format!("Cannot promote staging directory: {error}"),
    })
}

#[cfg(windows)]
fn move_directory(
    source: &std::path::Path,
    target: &std::path::Path,
) -> Result<(), DirectoryMoveFailure> {
    crate::package_promoter_windows::move_directory_no_replace(source, target)
}

#[cfg(not(any(unix, windows)))]
fn move_directory(
    _source: &std::path::Path,
    _target: &std::path::Path,
) -> Result<(), DirectoryMoveFailure> {
    Err(DirectoryMoveFailure {
        code: "PROMOTION_PLATFORM_UNSUPPORTED",
        message: "Package promotion is not implemented on this platform".to_string(),
    })
}

fn failed(
    context: &crate::package_promoter_result::PromotionContext<'_>,
    records: Vec<crate::PromotedFileRecord>,
    error: crate::PackagePromotionError,
) -> PackagePromotionResult {
    crate::package_promoter_result::result(context, records, "promotion_rejected", vec![error])
}

#[cfg(unix)]
fn sync_target_parent(target: &std::path::Path) -> Result<(), crate::PackagePromotionError> {
    let parent = target.parent().ok_or_else(|| {
        crate::package_promoter_result::error(
            "PROMOTION_TARGET_PARENT_INVALID",
            "Final target has no parent directory",
            Some(target),
        )
    })?;
    std::fs::File::open(parent)
        .and_then(|directory| directory.sync_all())
        .map_err(|sync_error| {
            crate::package_promoter_result::error(
                "PROMOTION_PARENT_SYNC_FAILED",
                format!("Cannot sync final target parent: {sync_error}"),
                Some(parent),
            )
        })
}

#[cfg(not(any(unix, windows)))]
fn sync_target_parent(_target: &std::path::Path) -> Result<(), crate::PackagePromotionError> {
    Ok(())
}

#[cfg(windows)]
fn sync_target_parent(_target: &std::path::Path) -> Result<(), crate::PackagePromotionError> {
    Ok(())
}
