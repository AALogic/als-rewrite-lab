use crate::{
    DesktopApplicationError, DesktopCopyPreview, DesktopCopyResult, DesktopExecuteCopyRequest,
    DesktopPrepareCopyRequest, DESKTOP_COPY_SERVICE_VERSION,
};
use rescue_pipeline::{CurrentPathCopyResult, LaboratoryPackageError};

pub(crate) fn preview_from_pipeline(
    request: &DesktopPrepareCopyRequest,
    result: CurrentPathCopyResult,
) -> DesktopCopyPreview {
    let errors = map_errors(&result.errors);
    let diagnostic_report = crate::desktop_copy_diagnostic::from_pipeline(
        &request.request_id,
        "copy_preview",
        &result,
        &errors,
        &[&request.source_als_path, &request.target_project_root],
    );
    let source_hash = result
        .package_plan
        .as_ref()
        .map(|plan| plan.source_als.source_file_hash.clone())
        .unwrap_or_default();
    let expected = match result.run_status.as_str() {
        "complete_copy_preview_ready" => "complete_copy_ready_for_manual_check",
        "incomplete_copy_preview_ready" => "incomplete_copy_ready_for_manual_check",
        _ => "blocked",
    };
    DesktopCopyPreview {
        service_version: DESKTOP_COPY_SERVICE_VERSION.to_string(),
        request_id: request.request_id.clone(),
        preview_status: result.run_status,
        source_als_path: request.source_als_path.clone(),
        source_als_sha256: source_hash,
        plan_fingerprint: result.plan_fingerprint,
        target_project_root: request.target_project_root.clone(),
        required_asset_count: result.required_asset_count,
        system_dependency_count: result.system_dependency_count,
        copy_asset_count: result.copied_asset_count,
        rewrite_reference_count: result.rewritten_reference_count,
        omitted_asset_count: result.omitted_asset_count,
        expected_result_status: expected.to_string(),
        diagnostic_report,
        errors,
    }
}

pub(crate) fn result_from_pipeline(
    request: &DesktopExecuteCopyRequest,
    result: CurrentPathCopyResult,
) -> DesktopCopyResult {
    let errors = map_errors(&result.errors);
    let diagnostic_report = crate::desktop_copy_diagnostic::from_pipeline(
        &request.request_id,
        "copy_execution",
        &result,
        &errors,
        &[
            &request.preview.source_als_path,
            &request.preview.target_project_root,
        ],
    );
    let successful = matches!(
        result.run_status.as_str(),
        "complete_copy_ready_for_manual_check" | "incomplete_copy_ready_for_manual_check"
    );
    DesktopCopyResult {
        service_version: DESKTOP_COPY_SERVICE_VERSION.to_string(),
        request_id: request.request_id.clone(),
        run_status: result.run_status,
        final_target_root: successful.then(|| request.preview.target_project_root.clone()),
        system_dependency_count: result.system_dependency_count,
        copied_asset_count: result.copied_asset_count,
        rewritten_reference_count: result.rewritten_reference_count,
        omitted_asset_count: result.omitted_asset_count,
        diagnostic_report,
        errors,
    }
}

pub(crate) fn failed_preview(
    request: &DesktopPrepareCopyRequest,
    error: DesktopApplicationError,
) -> DesktopCopyPreview {
    let errors = vec![error];
    let diagnostic_report = crate::desktop_copy_diagnostic::before_pipeline(
        &request.request_id,
        "copy_preview",
        "copy_preview_failed",
        "request_validation",
        &errors,
        &[&request.source_als_path, &request.target_project_root],
    );
    DesktopCopyPreview {
        service_version: DESKTOP_COPY_SERVICE_VERSION.to_string(),
        request_id: request.request_id.clone(),
        preview_status: "copy_preview_failed".to_string(),
        source_als_path: request.source_als_path.clone(),
        source_als_sha256: String::new(),
        plan_fingerprint: None,
        target_project_root: request.target_project_root.clone(),
        required_asset_count: 0,
        system_dependency_count: 0,
        copy_asset_count: 0,
        rewrite_reference_count: 0,
        omitted_asset_count: 0,
        expected_result_status: "blocked".to_string(),
        diagnostic_report,
        errors,
    }
}

pub(crate) fn failed_result(
    request: &DesktopExecuteCopyRequest,
    errors: Vec<DesktopApplicationError>,
) -> DesktopCopyResult {
    let completed_stage = errors
        .first()
        .map(|error| error.stage.as_str())
        .unwrap_or("request_validation");
    let diagnostic_report = crate::desktop_copy_diagnostic::before_pipeline(
        &request.request_id,
        "copy_execution",
        "copy_execution_failed",
        completed_stage,
        &errors,
        &[
            &request.preview.source_als_path,
            &request.preview.target_project_root,
        ],
    );
    DesktopCopyResult {
        service_version: DESKTOP_COPY_SERVICE_VERSION.to_string(),
        request_id: request.request_id.clone(),
        run_status: "copy_execution_failed".to_string(),
        final_target_root: None,
        system_dependency_count: 0,
        copied_asset_count: 0,
        rewritten_reference_count: 0,
        omitted_asset_count: 0,
        diagnostic_report,
        errors,
    }
}

fn map_errors(errors: &[LaboratoryPackageError]) -> Vec<DesktopApplicationError> {
    errors
        .iter()
        .map(|item| DesktopApplicationError {
            error_code: item.error_code.clone(),
            stage: item.stage.clone(),
            message: item.message.clone(),
        })
        .collect()
}
