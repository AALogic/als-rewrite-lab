use crate::{
    BatchCopyOperations, BatchCopyPreview, BatchPrepareCopyRequest, BatchPreviewJob,
    DesktopPrepareCopyRequest, BATCH_COPY_SERVICE_VERSION,
};
use std::collections::{HashMap, HashSet};
use std::path::Path;
use std::time::Instant;

pub(crate) fn prepare_batch_copy_impl(
    request: &BatchPrepareCopyRequest,
    operations: &mut dyn BatchCopyOperations,
) -> BatchCopyPreview {
    let started = Instant::now();
    if let Some((code, message)) = validate_request(request) {
        return failed_preview(request, code, message, started);
    }
    let mut jobs = target_jobs(request);
    mark_target_collisions(&mut jobs);
    for job in jobs.iter_mut().filter(|job| job.job_status == "pending") {
        let preview = operations.prepare(&DesktopPrepareCopyRequest {
            request_id: format!("{}:{}:preview", request.request_id, job.job_id),
            source_als_path: job.source_als_path.clone(),
            target_project_root: job.target_project_root.clone(),
            experimental_compatibility_consent: request.experimental_compatibility_consent,
        });
        job.errors = preview.errors.clone();
        job.job_status = if preview_is_ready(&preview) {
            "ready"
        } else {
            "blocked"
        }
        .to_string();
        job.preview = Some(preview);
    }
    complete_preview(request, jobs, started)
}

fn validate_request(request: &BatchPrepareCopyRequest) -> Option<(&'static str, &'static str)> {
    if request.request_id.trim().is_empty() {
        return Some(("BATCH_REQUEST_EMPTY", "Batch request ID must not be empty."));
    }
    if request.selections.is_empty() {
        return Some((
            "BATCH_SELECTION_EMPTY",
            "At least one Project selection is required.",
        ));
    }
    if !request.destination_parent.is_absolute() {
        return Some((
            "BATCH_DESTINATION_NOT_ABSOLUTE",
            "Batch destination parent must be an absolute path.",
        ));
    }
    let mut ids = HashSet::new();
    request
        .selections
        .iter()
        .any(|selection| !ids.insert(&selection.selection_id))
        .then_some((
            "BATCH_DUPLICATE_SELECTION",
            "A Project selection occurs more than once.",
        ))
}

fn target_jobs(request: &BatchPrepareCopyRequest) -> Vec<BatchPreviewJob> {
    request
        .selections
        .iter()
        .enumerate()
        .map(|(index, selection)| {
            let suggested = crate::default_target_project_root(
                &selection.native_als_path,
                &request.destination_parent,
            );
            let (target, status, errors) = match suggested {
                Ok(target) => (target, "pending".to_string(), Vec::new()),
                Err(error) => (
                    request.destination_parent.clone(),
                    "blocked".to_string(),
                    vec![error],
                ),
            };
            BatchPreviewJob {
                job_id: format!("batch_job_{index:06}"),
                selection_id: selection.selection_id.clone(),
                source_als_path: selection.native_als_path.clone(),
                target_project_root: target,
                job_status: status,
                preview: None,
                errors,
            }
        })
        .collect()
}

fn mark_target_collisions(jobs: &mut [BatchPreviewJob]) {
    let mut targets = HashMap::<String, Vec<usize>>::new();
    for (index, job) in jobs
        .iter()
        .enumerate()
        .filter(|(_, job)| job.job_status == "pending")
    {
        targets
            .entry(path_key(&job.target_project_root))
            .or_default()
            .push(index);
    }
    for indices in targets.values().filter(|indices| indices.len() > 1) {
        for index in indices {
            let job = &mut jobs[*index];
            job.job_status = "target_collision".to_string();
            job.errors.push(crate::batch_copy_result::error(
                "BATCH_TARGET_COLLISION",
                "target_planning",
                "Two selected Projects resolve to the same target location.",
            ));
        }
    }
}

fn path_key(path: &Path) -> String {
    let value = path.as_os_str().to_string_lossy();
    if cfg!(windows) {
        value.to_lowercase()
    } else {
        value.into_owned()
    }
}

fn preview_is_ready(preview: &crate::DesktopCopyPreview) -> bool {
    matches!(
        preview.preview_status.as_str(),
        "complete_copy_preview_ready" | "incomplete_copy_preview_ready"
    ) && preview.errors.is_empty()
        && preview.plan_fingerprint.is_some()
        && !preview.source_als_sha256.is_empty()
}

fn complete_preview(
    request: &BatchPrepareCopyRequest,
    jobs: Vec<BatchPreviewJob>,
    started: Instant,
) -> BatchCopyPreview {
    let summary = crate::batch_copy_result::preview_summary(&jobs);
    let status = if summary.ready_job_count == summary.total_job_count {
        "ready"
    } else if summary.ready_job_count > 0 {
        "partially_ready"
    } else {
        "blocked"
    };
    let error_codes = jobs.iter().flat_map(|job| {
        job.errors
            .iter()
            .map(|error| error.error_code.clone())
            .collect::<Vec<_>>()
    });
    BatchCopyPreview {
        service_version: BATCH_COPY_SERVICE_VERSION.to_string(),
        request_id: request.request_id.clone(),
        preview_status: status.to_string(),
        destination_parent: request.destination_parent.clone(),
        diagnostic_report: crate::batch_copy_result::diagnostic(
            &request.request_id,
            status,
            &summary,
            error_codes,
            started,
        ),
        jobs,
        summary,
        errors: Vec::new(),
    }
}

fn failed_preview(
    request: &BatchPrepareCopyRequest,
    code: &str,
    message: &str,
    started: Instant,
) -> BatchCopyPreview {
    let error = crate::batch_copy_result::error(code, "request_validation", message);
    let summary = crate::BatchCopySummary::default();
    BatchCopyPreview {
        service_version: BATCH_COPY_SERVICE_VERSION.to_string(),
        request_id: request.request_id.clone(),
        preview_status: "blocked".to_string(),
        destination_parent: request.destination_parent.clone(),
        jobs: Vec::new(),
        diagnostic_report: crate::batch_copy_result::diagnostic(
            &request.request_id,
            "blocked",
            &summary,
            std::iter::once(code.to_string()),
            started,
        ),
        summary,
        errors: vec![error],
    }
}
