use crate::{
    DesktopApplicationError, DesktopCopyPreview, DesktopCopyResult, DesktopExecuteCopyRequest,
    DesktopPrepareCopyRequest,
};
use rescue_pipeline::{prepare_current_path_copy, run_current_path_copy, CurrentPathCopyRequest};
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};
use std::time::Instant;

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
    let started = Instant::now();
    if let Some(error) = validate_prepare(request) {
        return failed_preview(request, error, elapsed_ms(started));
    }
    let rewrite_policy = if request.experimental_compatibility_consent {
        rescue_pipeline::COMPATIBILITY_LAB_REWRITE_POLICY
    } else {
        rescue_pipeline::STRICT_REWRITE_POLICY
    };
    let pipeline_request = pipeline_request(
        &request.request_id,
        rewrite_policy,
        &request.source_als_path,
        None,
        None,
        &request.target_project_root,
    );
    crate::desktop_copy_result::preview_from_pipeline(
        request,
        prepare_current_path_copy(&pipeline_request),
        elapsed_ms(started),
    )
}

pub(crate) fn execute(request: &DesktopExecuteCopyRequest) -> DesktopCopyResult {
    let started = Instant::now();
    if let Some(error) = validate_execute(request) {
        return failed_result(request, vec![error], elapsed_ms(started));
    }
    let preview = &request.preview;
    let pipeline_request = pipeline_request(
        &preview.request_id,
        &preview.rewrite_policy,
        &preview.source_als_path,
        Some(preview.source_als_sha256.clone()),
        preview.plan_fingerprint.clone(),
        &preview.target_project_root,
    );
    crate::desktop_copy_result::result_from_pipeline(
        request,
        run_current_path_copy(&pipeline_request),
        elapsed_ms(started),
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
    if request.experimental_compatibility_consent && !cfg!(feature = "compatibility-lab") {
        return Some(error(
            "DESKTOP_COMPATIBILITY_LAB_UNAVAILABLE",
            "request_validation",
            "This application build cannot execute experimental compatibility copies",
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
    if request.preview.rewrite_policy == rescue_pipeline::COMPATIBILITY_LAB_REWRITE_POLICY
        && !cfg!(feature = "compatibility-lab")
    {
        return Some(error(
            "DESKTOP_COMPATIBILITY_LAB_UNAVAILABLE",
            "preview_validation",
            "This application build cannot execute an experimental compatibility preview",
        ));
    }
    if !matches!(
        request.preview.rewrite_policy.as_str(),
        rescue_pipeline::STRICT_REWRITE_POLICY | rescue_pipeline::COMPATIBILITY_LAB_REWRITE_POLICY
    ) {
        return Some(error(
            "DESKTOP_REWRITE_POLICY_UNSUPPORTED",
            "preview_validation",
            "Desktop preview contains an unsupported rewrite policy",
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
    rewrite_policy: &str,
    source_als_path: &Path,
    expected_hash: Option<String>,
    expected_plan_fingerprint: Option<rescue_pipeline::PlanFingerprint>,
    target_project_root: &Path,
) -> CurrentPathCopyRequest {
    let parent = target_project_root.parent().unwrap_or(target_project_root);
    let token = artifact_token(run_id, target_project_root);
    CurrentPathCopyRequest {
        run_id: run_id.to_string(),
        rewrite_policy: rewrite_policy.to_string(),
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
    elapsed_ms: u64,
) -> DesktopCopyPreview {
    crate::desktop_copy_result::failed_preview(request, error, elapsed_ms)
}

fn failed_result(
    request: &DesktopExecuteCopyRequest,
    errors: Vec<DesktopApplicationError>,
    elapsed_ms: u64,
) -> DesktopCopyResult {
    crate::desktop_copy_result::failed_result(request, errors, elapsed_ms)
}

fn elapsed_ms(started: Instant) -> u64 {
    u64::try_from(started.elapsed().as_millis()).unwrap_or(u64::MAX)
}

fn error(code: &str, stage: &str, message: &str) -> DesktopApplicationError {
    DesktopApplicationError {
        error_code: code.to_string(),
        stage: stage.to_string(),
        message: message.to_string(),
    }
}
