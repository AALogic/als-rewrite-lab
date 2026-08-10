use crate::BatchExecuteCopyRequest;
use std::collections::HashSet;

pub(crate) fn validate_execution(
    request: &BatchExecuteCopyRequest,
) -> Option<(&'static str, &'static str, &'static str)> {
    if request.request_id.trim().is_empty() {
        return Some((
            "BATCH_REQUEST_EMPTY",
            "request_validation",
            "Batch request ID must not be empty.",
        ));
    }
    if !request.write_consent {
        return Some((
            "BATCH_WRITE_CONSENT_REQUIRED",
            "write_consent",
            "Explicit batch write consent is required.",
        ));
    }
    if request.preview.jobs.is_empty()
        || request.preview.service_version != crate::BATCH_COPY_SERVICE_VERSION
        || request.preview.request_id.trim().is_empty()
        || !matches!(
            request.preview.preview_status.as_str(),
            "ready" | "partially_ready" | "blocked"
        )
        || !request.preview.errors.is_empty()
        || !request.preview.destination_parent.is_absolute()
        || !preview_summary_is_consistent(request)
        || !preview_jobs_are_valid(request)
    {
        return Some((
            "BATCH_PREVIEW_INVALID",
            "preview_validation",
            "Batch preview is incomplete, inconsistent or not executable.",
        ));
    }
    None
}

fn preview_summary_is_consistent(request: &BatchExecuteCopyRequest) -> bool {
    let summary = crate::batch_copy_result::preview_summary(&request.preview.jobs);
    let expected_status = if summary.ready_job_count == summary.total_job_count {
        "ready"
    } else if summary.ready_job_count > 0 {
        "partially_ready"
    } else {
        "blocked"
    };
    request.preview.summary == summary && request.preview.preview_status == expected_status
}

fn preview_jobs_are_valid(request: &BatchExecuteCopyRequest) -> bool {
    let mut job_ids = HashSet::new();
    let mut selection_ids = HashSet::new();
    let mut targets = HashSet::new();
    let mut ready_count = 0usize;
    for job in &request.preview.jobs {
        if !job_ids.insert(&job.job_id) || !selection_ids.insert(&job.selection_id) {
            return false;
        }
        if !matches!(
            job.job_status.as_str(),
            "ready" | "blocked" | "target_collision"
        ) {
            return false;
        }
        if job.job_status != "ready" {
            continue;
        }
        ready_count += 1;
        if job.target_project_root.parent().map(path_key)
            != Some(path_key(&request.preview.destination_parent))
            || !targets.insert(path_key(&job.target_project_root))
            || !ready_preview_matches(job)
        {
            return false;
        }
    }
    ready_count == request.preview.summary.ready_job_count
}

fn ready_preview_matches(job: &crate::BatchPreviewJob) -> bool {
    let Some(preview) = job.preview.as_ref() else {
        return false;
    };
    preview.source_als_path == job.source_als_path
        && preview.target_project_root == job.target_project_root
        && preview.errors.is_empty()
        && preview.plan_fingerprint.is_some()
        && !preview.source_als_sha256.is_empty()
        && matches!(
            preview.preview_status.as_str(),
            "complete_copy_preview_ready" | "incomplete_copy_preview_ready"
        )
}

fn path_key(path: &std::path::Path) -> String {
    let value = path.as_os_str().to_string_lossy();
    if cfg!(windows) {
        value.to_lowercase()
    } else {
        value.into_owned()
    }
}
