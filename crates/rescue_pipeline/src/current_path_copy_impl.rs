use crate::current_path_copy::{
    CurrentPathCopyRequest, CurrentPathCopyResult, CURRENT_PATH_COPY_PIPELINE_VERSION,
};
use crate::current_path_copy_binding::prepare_stage;
use crate::{LaboratoryPackageError, LaboratoryPackageRequest};
use rescue_packaging::{PackagePlan, PlanFingerprint};
use std::path::PathBuf;

pub(crate) fn prepare(request: &CurrentPathCopyRequest) -> CurrentPathCopyResult {
    match prepare_stage(request) {
        Ok(prepared) => result_from_plan(
            request,
            prepared.plan,
            prepared.fingerprint,
            None,
            true,
            Vec::new(),
        ),
        Err(result) => *result,
    }
}

pub(crate) fn run(request: &CurrentPathCopyRequest) -> CurrentPathCopyResult {
    let prepared = match prepare_stage(request) {
        Ok(prepared) => prepared,
        Err(result) => return plan_bound_failure(request, *result),
    };
    if request
        .expected_plan_fingerprint
        .as_ref()
        .is_some_and(|expected| expected != &prepared.fingerprint)
    {
        return failure_result(
            request,
            "preview_plan_changed",
            "plan_binding",
            Some(prepared.plan),
            Some(prepared.fingerprint),
            vec![error(
                "CURRENT_PATH_PREVIEW_PLAN_CHANGED",
                "plan_binding",
                "The current copy plan no longer matches the accepted desktop preview",
            )],
        );
    }
    let lab_request = laboratory_request(request);
    match crate::laboratory_pipeline_write::execute_package(&lab_request, &prepared.plan) {
        Ok(written) => result_from_plan(
            request,
            prepared.plan,
            prepared.fingerprint,
            Some(written.promotion),
            false,
            Vec::new(),
        ),
        Err(failure) => {
            let (promotion, errors) = write_failure_details(*failure);
            result_from_plan(
                request,
                prepared.plan,
                prepared.fingerprint,
                promotion,
                false,
                errors,
            )
        }
    }
}

fn write_failure_details(
    failure: crate::laboratory_pipeline_write::WriteFailure,
) -> (
    Option<rescue_promotion::PackagePromotionResult>,
    Vec<LaboratoryPackageError>,
) {
    let mut errors = vec![failure.error];
    if let Some(staging) = failure.progress.staging.as_ref() {
        errors.extend(
            staging
                .errors
                .iter()
                .map(|item| error(&item.error_code, "staging", &item.message)),
        );
    }
    if let Some(rewrite) = failure.progress.rewrite.as_ref() {
        errors.extend(
            rewrite
                .errors
                .iter()
                .map(|item| error(&item.error_code, "als_rewrite", &item.message)),
        );
    }
    if let Some(validation) = failure.progress.validation.as_ref() {
        errors.extend(
            validation
                .errors
                .iter()
                .map(|item| error(&item.error_code, "package_validation", &item.message)),
        );
    }
    if let Some(manifests) = failure.progress.manifests.as_ref() {
        errors.extend(
            manifests
                .errors
                .iter()
                .map(|item| error(&item.error_code, "manifest", &item.message)),
        );
    }
    if let Some(promotion) = failure.progress.promotion.as_ref() {
        errors.extend(
            promotion
                .errors
                .iter()
                .map(|item| error(&item.error_code, "promotion", &item.message)),
        );
    }
    (failure.progress.promotion, errors)
}

fn plan_bound_failure(
    request: &CurrentPathCopyRequest,
    mut result: CurrentPathCopyResult,
) -> CurrentPathCopyResult {
    let plan_may_have_changed = request.expected_plan_fingerprint.is_some()
        && matches!(
            result.completed_stage.as_str(),
            "current_path_binding" | "package_planning" | "plan_fingerprint"
        );
    if plan_may_have_changed {
        result.run_status = "preview_plan_changed".to_string();
        result.errors.insert(
            0,
            error(
                "CURRENT_PATH_PREVIEW_PLAN_CHANGED",
                "plan_binding",
                "The current copy plan no longer matches the accepted desktop preview",
            ),
        );
    }
    result
}

pub(crate) fn laboratory_request(request: &CurrentPathCopyRequest) -> LaboratoryPackageRequest {
    let bounded_root = request
        .source_als_path
        .parent()
        .map(PathBuf::from)
        .into_iter()
        .collect();
    LaboratoryPackageRequest {
        run_id: request.run_id.clone(),
        source_als_path: request.source_als_path.clone(),
        scan_roots: bounded_root,
        max_scan_entries: 1,
        staging_root: request.staging_root.clone(),
        target_project_root: request.target_project_root.clone(),
        private_ledger_path: request.private_ledger_path.clone(),
        user_selection_set: None,
    }
}

fn result_from_plan(
    request: &CurrentPathCopyRequest,
    plan: PackagePlan,
    fingerprint: PlanFingerprint,
    promotion: Option<rescue_promotion::PackagePromotionResult>,
    preview: bool,
    errors: Vec<LaboratoryPackageError>,
) -> CurrentPathCopyResult {
    let incomplete = plan.plan_status == "ready_current_paths_incomplete";
    let successful = errors.is_empty();
    let status = if !successful {
        "write_pipeline_failed"
    } else if preview && incomplete {
        "incomplete_copy_preview_ready"
    } else if preview {
        "complete_copy_preview_ready"
    } else if incomplete {
        "incomplete_copy_ready_for_manual_check"
    } else {
        "complete_copy_ready_for_manual_check"
    };
    let copied = plan
        .copy_operations
        .iter()
        .filter(|operation| operation.operation_kind == "copy_audio")
        .count();
    CurrentPathCopyResult {
        pipeline_version: CURRENT_PATH_COPY_PIPELINE_VERSION.to_string(),
        run_id: request.run_id.clone(),
        rewrite_policy: request.rewrite_policy.clone(),
        run_status: status.to_string(),
        completed_stage: if preview {
            "package_planning"
        } else if successful {
            "promotion"
        } else {
            "write_pipeline"
        }
        .to_string(),
        required_asset_count: plan.metadata.required_asset_count,
        system_dependency_count: plan.metadata.system_dependency_count,
        copied_asset_count: copied,
        rewritten_reference_count: plan.rewrite_operations.len(),
        omitted_asset_count: plan.unresolved_requirements.len(),
        plan_fingerprint: Some(fingerprint),
        package_plan: Some(plan),
        promotion,
        errors,
    }
}

pub(crate) fn failure_result(
    request: &CurrentPathCopyRequest,
    status: &str,
    stage: &str,
    plan: Option<PackagePlan>,
    plan_fingerprint: Option<PlanFingerprint>,
    errors: Vec<LaboratoryPackageError>,
) -> CurrentPathCopyResult {
    CurrentPathCopyResult {
        pipeline_version: CURRENT_PATH_COPY_PIPELINE_VERSION.to_string(),
        run_id: request.run_id.clone(),
        rewrite_policy: request.rewrite_policy.clone(),
        run_status: status.to_string(),
        completed_stage: stage.to_string(),
        required_asset_count: plan
            .as_ref()
            .map(|item| item.metadata.required_asset_count)
            .unwrap_or_default(),
        system_dependency_count: plan
            .as_ref()
            .map(|item| item.metadata.system_dependency_count)
            .unwrap_or_default(),
        copied_asset_count: 0,
        rewritten_reference_count: 0,
        omitted_asset_count: plan
            .as_ref()
            .map(|item| item.unresolved_requirements.len())
            .unwrap_or_default(),
        plan_fingerprint,
        package_plan: plan,
        promotion: None,
        errors,
    }
}

pub(crate) fn error(code: &str, stage: &str, message: &str) -> LaboratoryPackageError {
    LaboratoryPackageError {
        error_code: code.to_string(),
        stage: stage.to_string(),
        message: message.to_string(),
    }
}
