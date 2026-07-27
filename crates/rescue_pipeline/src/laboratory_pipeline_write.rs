use crate::{LaboratoryPackageError, LaboratoryPackageRequest};
use rescue_execution::{execute_staging, StagingExecutionRequest, StagingExecutionResult};
use rescue_manifest::{write_package_evidence, ManifestWriteRequest, ManifestWriteResult};
use rescue_packaging::PackagePlan;
use rescue_promotion::{
    promote_validated_package, PackagePromotionRequest, PackagePromotionResult,
};
use rescue_rewriter::{rewrite_staged_als, ALSRewriteRequest, ALSRewriteResult};
use rescue_validation::{
    validate_staged_package, PackageValidationRequest, PackageValidationResult,
};
use std::path::PathBuf;

pub(crate) struct WriteStage {
    pub staging: StagingExecutionResult,
    pub rewrite: ALSRewriteResult,
    pub validation: PackageValidationResult,
    pub manifests: ManifestWriteResult,
    pub promotion: PackagePromotionResult,
}

#[derive(Default)]
pub(crate) struct WriteProgress {
    pub staging: Option<StagingExecutionResult>,
    pub rewrite: Option<ALSRewriteResult>,
    pub validation: Option<PackageValidationResult>,
    pub manifests: Option<ManifestWriteResult>,
    pub promotion: Option<PackagePromotionResult>,
}

pub(crate) struct WriteFailure {
    pub error: LaboratoryPackageError,
    pub progress: WriteProgress,
}

pub(crate) fn execute_package(
    request: &LaboratoryPackageRequest,
    plan: &PackagePlan,
) -> Result<WriteStage, Box<WriteFailure>> {
    let mut progress = WriteProgress::default();
    let staging = execute_staging(
        &StagingExecutionRequest {
            execution_id: format!("{}:execution", request.run_id),
            staging_root: request.staging_root.clone(),
        },
        plan,
    );
    let staging_ok = staging.execution_status == "staging_complete";
    progress.staging = Some(staging);
    if !staging_ok {
        return Err(failure(
            "PIPELINE_STAGING_FAILED",
            "staging",
            "Staging did not complete",
            progress,
        ));
    }
    let staging_ref = progress.staging.as_ref().ok_or_else(|| {
        failure(
            "PIPELINE_INTERNAL_STATE_ERROR",
            "staging",
            "Staging result was not retained",
            WriteProgress::default(),
        )
    })?;
    let rewrite = rewrite_staged_als(
        &ALSRewriteRequest {
            rewrite_id: format!("{}:rewrite", request.run_id),
            staging_root: request.staging_root.clone(),
        },
        plan,
        staging_ref,
    );
    let rewrite_ok = rewrite.rewrite_status == "rewrite_complete";
    progress.rewrite = Some(rewrite);
    if !rewrite_ok {
        return Err(failure(
            "PIPELINE_REWRITE_FAILED",
            "als_rewrite",
            "ALS rewrite did not complete",
            progress,
        ));
    }
    continue_after_rewrite(request, plan, progress)
}

fn continue_after_rewrite(
    request: &LaboratoryPackageRequest,
    plan: &PackagePlan,
    mut progress: WriteProgress,
) -> Result<WriteStage, Box<WriteFailure>> {
    let (Some(staging), Some(rewrite)) = (progress.staging.as_ref(), progress.rewrite.as_ref())
    else {
        return Err(failure(
            "PIPELINE_INTERNAL_STATE_ERROR",
            "validation",
            "Required execution results are absent",
            progress,
        ));
    };
    let validation = validate_staged_package(
        &PackageValidationRequest {
            validation_id: format!("{}:validation", request.run_id),
            staging_root: request.staging_root.clone(),
        },
        plan,
        staging,
        rewrite,
    );
    let validation_ok = validation.validation_status == "validation_passed";
    progress.validation = Some(validation);
    if !validation_ok {
        return Err(failure(
            "PIPELINE_VALIDATION_FAILED",
            "package_validation",
            "Package validation did not pass",
            progress,
        ));
    }
    continue_after_validation(request, plan, progress)
}

fn continue_after_validation(
    request: &LaboratoryPackageRequest,
    plan: &PackagePlan,
    mut progress: WriteProgress,
) -> Result<WriteStage, Box<WriteFailure>> {
    let (Some(staging), Some(rewrite), Some(validation)) = (
        progress.staging.as_ref(),
        progress.rewrite.as_ref(),
        progress.validation.as_ref(),
    ) else {
        return Err(failure(
            "PIPELINE_INTERNAL_STATE_ERROR",
            "manifest",
            "Required validated results are absent",
            progress,
        ));
    };
    let manifests = write_package_evidence(
        &ManifestWriteRequest {
            manifest_id: format!("{}:manifest", request.run_id),
            private_ledger_path: request.private_ledger_path.clone(),
            package_manifest_relative_path: PathBuf::from("Rescue Manifest/package-manifest.json"),
        },
        plan,
        staging,
        rewrite,
        validation,
    );
    let manifest_ok = manifests.write_status == "manifests_written";
    progress.manifests = Some(manifests);
    if !manifest_ok {
        return Err(failure(
            "PIPELINE_MANIFEST_FAILED",
            "manifest",
            "Package evidence was not written",
            progress,
        ));
    }
    promote(request, plan, progress)
}

fn promote(
    request: &LaboratoryPackageRequest,
    plan: &PackagePlan,
    mut progress: WriteProgress,
) -> Result<WriteStage, Box<WriteFailure>> {
    let (Some(staging), Some(rewrite), Some(validation), Some(manifests)) = (
        progress.staging.as_ref(),
        progress.rewrite.as_ref(),
        progress.validation.as_ref(),
        progress.manifests.as_ref(),
    ) else {
        return Err(failure(
            "PIPELINE_INTERNAL_STATE_ERROR",
            "promotion",
            "Required manifest results are absent",
            progress,
        ));
    };
    let promotion = promote_validated_package(
        &PackagePromotionRequest {
            promotion_id: format!("{}:promotion", request.run_id),
        },
        plan,
        staging,
        rewrite,
        validation,
        manifests,
    );
    let promotion_ok = promotion.promotion_status == "promoted_ready_for_manual_check";
    progress.promotion = Some(promotion);
    if !promotion_ok {
        return Err(failure(
            "PIPELINE_PROMOTION_FAILED",
            "promotion",
            "Package promotion did not complete",
            progress,
        ));
    }
    Ok(WriteStage {
        staging: progress.staging.take().ok_or_else(internal_failure)?,
        rewrite: progress.rewrite.take().ok_or_else(internal_failure)?,
        validation: progress.validation.take().ok_or_else(internal_failure)?,
        manifests: progress.manifests.take().ok_or_else(internal_failure)?,
        promotion: progress.promotion.take().ok_or_else(internal_failure)?,
    })
}

fn internal_failure() -> Box<WriteFailure> {
    failure(
        "PIPELINE_INTERNAL_STATE_ERROR",
        "promotion",
        "Completed pipeline output was not retained",
        WriteProgress::default(),
    )
}

fn failure(code: &str, stage: &str, message: &str, progress: WriteProgress) -> Box<WriteFailure> {
    Box::new(WriteFailure {
        error: LaboratoryPackageError {
            error_code: code.to_string(),
            stage: stage.to_string(),
            message: message.to_string(),
        },
        progress,
    })
}
