use crate::{
    DesktopApplicationError, DesktopCopyPreview, DesktopCopyResult, DesktopExecuteCopyRequest,
    DesktopPrepareCopyRequest,
};
use rescue_pipeline::{prepare_current_path_copy, run_current_path_copy, CurrentPathCopyRequest};
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};

pub(crate) fn default_target(
    source_als_path: &Path,
    destination_parent: &Path,
) -> Result<PathBuf, DesktopApplicationError> {
    if !source_als_path.is_absolute() || !destination_parent.is_absolute() {
        return Err(error(
            "DESKTOP_COPY_PATH_NOT_ABSOLUTE",
            "target_suggestion",
            "Source ALS and destination parent must be absolute paths",
        ));
    }
    let stem = source_als_path
        .file_stem()
        .and_then(|value| value.to_str())
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| {
            error(
                "DESKTOP_COPY_SOURCE_NAME_INVALID",
                "target_suggestion",
                "Selected ALS has no portable filename",
            )
        })?;
    Ok(destination_parent.join(format!("{stem} Rescue Project")))
}

pub(crate) fn prepare(request: &DesktopPrepareCopyRequest) -> DesktopCopyPreview {
    if let Some(error) = validate_prepare(request) {
        return failed_preview(request, error);
    }
    let pipeline_request = pipeline_request(
        &request.request_id,
        &request.source_als_path,
        None,
        None,
        &request.target_project_root,
    );
    crate::desktop_copy_result::preview_from_pipeline(
        request,
        prepare_current_path_copy(&pipeline_request),
    )
}

pub(crate) fn execute(request: &DesktopExecuteCopyRequest) -> DesktopCopyResult {
    if let Some(error) = validate_execute(request) {
        return failed_result(request, vec![error]);
    }
    let preview = &request.preview;
    let pipeline_request = pipeline_request(
        &preview.request_id,
        &preview.source_als_path,
        Some(preview.source_als_sha256.clone()),
        preview.plan_fingerprint.clone(),
        &preview.target_project_root,
    );
    crate::desktop_copy_result::result_from_pipeline(
        request,
        run_current_path_copy(&pipeline_request),
    )
}

fn validate_prepare(request: &DesktopPrepareCopyRequest) -> Option<DesktopApplicationError> {
    if request.request_id.trim().is_empty() {
        return Some(error(
            "DESKTOP_COPY_REQUEST_ID_EMPTY",
            "request_validation",
            "Copy request ID must not be empty",
        ));
    }
    if !request.source_als_path.is_absolute() || !request.target_project_root.is_absolute() {
        return Some(error(
            "DESKTOP_COPY_PATH_NOT_ABSOLUTE",
            "request_validation",
            "Source ALS and target Project root must be absolute paths",
        ));
    }
    None
}

fn validate_execute(request: &DesktopExecuteCopyRequest) -> Option<DesktopApplicationError> {
    if request.request_id.trim().is_empty() {
        return Some(error(
            "DESKTOP_COPY_REQUEST_ID_EMPTY",
            "request_validation",
            "Execution request ID must not be empty",
        ));
    }
    if !request.write_consent {
        return Some(error(
            "DESKTOP_COPY_CONSENT_REQUIRED",
            "write_consent",
            "Explicit write consent is required",
        ));
    }
    if !matches!(
        request.preview.preview_status.as_str(),
        "complete_copy_preview_ready" | "incomplete_copy_preview_ready"
    ) || !request.preview.errors.is_empty()
        || request.preview.source_als_sha256.is_empty()
        || request.preview.plan_fingerprint.is_none()
    {
        return Some(error(
            "DESKTOP_COPY_PREVIEW_NOT_EXECUTABLE",
            "preview_validation",
            "Only a successful source-bound preview can be executed",
        ));
    }
    None
}

fn pipeline_request(
    run_id: &str,
    source_als_path: &Path,
    expected_hash: Option<String>,
    expected_plan_fingerprint: Option<rescue_pipeline::PlanFingerprint>,
    target_project_root: &Path,
) -> CurrentPathCopyRequest {
    let parent = target_project_root.parent().unwrap_or(target_project_root);
    let token = artifact_token(run_id, target_project_root);
    CurrentPathCopyRequest {
        run_id: run_id.to_string(),
        source_als_path: source_als_path.to_path_buf(),
        expected_source_als_sha256: expected_hash,
        expected_plan_fingerprint,
        staging_root: parent.join(format!(".als-rescue-{token}.staging")),
        target_project_root: target_project_root.to_path_buf(),
        private_ledger_path: parent.join(format!(".als-rescue-{token}.ledger.json")),
    }
}

fn artifact_token(run_id: &str, target: &Path) -> String {
    let mut hash = Sha256::new();
    hash.update(run_id.as_bytes());
    hash.update([0]);
    hash.update(target.as_os_str().to_string_lossy().as_bytes());
    hash.finalize()[..8]
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

fn failed_preview(
    request: &DesktopPrepareCopyRequest,
    error: DesktopApplicationError,
) -> DesktopCopyPreview {
    crate::desktop_copy_result::failed_preview(request, error)
}

fn failed_result(
    request: &DesktopExecuteCopyRequest,
    errors: Vec<DesktopApplicationError>,
) -> DesktopCopyResult {
    crate::desktop_copy_result::failed_result(request, errors)
}

fn error(code: &str, stage: &str, message: &str) -> DesktopApplicationError {
    DesktopApplicationError {
        error_code: code.to_string(),
        stage: stage.to_string(),
        message: message.to_string(),
    }
}
