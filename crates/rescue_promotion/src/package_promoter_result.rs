use crate::{
    PackagePromotionError, PackagePromotionMetadata, PackagePromotionRequest,
    PackagePromotionResult, PromotedFileRecord, PACKAGE_PROMOTER_VERSION,
    PACKAGE_PROMOTION_SCHEMA_VERSION,
};
use rescue_execution::StagingExecutionResult;
use rescue_manifest::ManifestWriteResult;
use rescue_packaging::PackagePlan;
use rescue_rewriter::ALSRewriteResult;
use rescue_validation::PackageValidationResult;
use std::path::{Path, PathBuf};

pub(crate) struct PromotionContext<'a> {
    pub request: &'a PackagePromotionRequest,
    pub plan: &'a PackagePlan,
    pub staging: &'a StagingExecutionResult,
    pub rewrite: &'a ALSRewriteResult,
    pub validation: &'a PackageValidationResult,
    pub manifests: &'a ManifestWriteResult,
}

pub(crate) fn result(
    context: &PromotionContext<'_>,
    records: Vec<PromotedFileRecord>,
    status: &str,
    errors: Vec<PackagePromotionError>,
) -> PackagePromotionResult {
    let verified = records
        .iter()
        .filter(|record| record.file_status == "verified")
        .count();
    PackagePromotionResult {
        metadata: PackagePromotionMetadata {
            promoter_version: PACKAGE_PROMOTER_VERSION.to_string(),
            promotion_schema_version: PACKAGE_PROMOTION_SCHEMA_VERSION.to_string(),
            promotion_id: context.request.promotion_id.clone(),
            plan_id: context.plan.metadata.plan_id.clone(),
            execution_id: context.staging.metadata.execution_id.clone(),
            rewrite_id: context.rewrite.metadata.rewrite_id.clone(),
            validation_id: context.validation.metadata.validation_id.clone(),
            manifest_id: context.manifests.metadata.manifest_id.clone(),
            verified_file_count: verified,
            error_count: errors.len(),
        },
        former_staging_root: context.staging.staging_root.clone(),
        final_target_root: context.plan.target_project_root.clone(),
        promoted_files: records,
        promotion_status: status.to_string(),
        manual_check_status: if errors.is_empty() {
            "ready_for_manual_ableton_check"
        } else {
            "not_ready"
        }
        .to_string(),
        errors,
    }
}

pub(crate) fn error(
    code: &str,
    message: impl Into<String>,
    path: Option<&Path>,
) -> PackagePromotionError {
    PackagePromotionError {
        error_code: code.to_string(),
        message: message.into(),
        path: path.map(PathBuf::from),
    }
}
