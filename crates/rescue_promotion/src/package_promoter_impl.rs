use crate::{PackagePromotionRequest, PackagePromotionResult};
use rescue_execution::StagingExecutionResult;
use rescue_manifest::ManifestWriteResult;
use rescue_packaging::PackagePlan;
use rescue_rewriter::ALSRewriteResult;
use rescue_validation::PackageValidationResult;
use std::fs;

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
    if let Err(rename_error) = fs::rename(
        &context.staging.staging_root,
        &context.plan.target_project_root,
    ) {
        return failed(
            context,
            preflight_records,
            crate::package_promoter_result::error(
                "PROMOTION_RENAME_FAILED",
                format!("Cannot promote staging directory: {rename_error}"),
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
    let rollback = fs::rename(
        &context.plan.target_project_root,
        &context.staging.staging_root,
    );
    let mut errors = vec![original_error];
    let status = if let Err(rollback_error) = rollback {
        errors.push(crate::package_promoter_result::error(
            "PROMOTION_ROLLBACK_FAILED",
            format!("Cannot restore staging after failed promotion: {rollback_error}"),
            Some(&context.staging.staging_root),
        ));
        "promotion_failed_rollback_failed"
    } else {
        "promotion_failed_rolled_back"
    };
    crate::package_promoter_result::result(context, records, status, errors)
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

#[cfg(not(unix))]
fn sync_target_parent(_target: &std::path::Path) -> Result<(), crate::PackagePromotionError> {
    Ok(())
}
