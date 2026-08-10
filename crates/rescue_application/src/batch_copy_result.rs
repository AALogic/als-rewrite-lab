use crate::{
    BatchCopyDiagnosticReport, BatchCopyJobResult, BatchCopyPreview, BatchCopyResult,
    BatchCopySummary, DesktopApplicationError, BATCH_COPY_DIAGNOSTIC_SCHEMA_VERSION,
    BATCH_COPY_SERVICE_VERSION,
};
use std::collections::BTreeSet;
use std::time::Instant;

pub(crate) fn preview_summary(jobs: &[crate::BatchPreviewJob]) -> BatchCopySummary {
    BatchCopySummary {
        total_job_count: jobs.len(),
        ready_job_count: jobs.iter().filter(|job| job.job_status == "ready").count(),
        blocked_job_count: jobs.iter().filter(|job| job.job_status != "ready").count(),
        ..BatchCopySummary::default()
    }
}

pub(crate) fn result_summary(
    preview: &BatchCopyPreview,
    jobs: &[BatchCopyJobResult],
) -> BatchCopySummary {
    BatchCopySummary {
        total_job_count: jobs.len(),
        ready_job_count: preview.summary.ready_job_count,
        blocked_job_count: jobs
            .iter()
            .filter(|job| job.job_status == "skipped_blocked")
            .count(),
        completed_job_count: jobs
            .iter()
            .filter(|job| job.job_status == "completed")
            .count(),
        incomplete_job_count: jobs
            .iter()
            .filter(|job| job.job_status == "completed_incomplete")
            .count(),
        failed_job_count: jobs.iter().filter(|job| job.job_status == "failed").count(),
        cancelled_job_count: jobs
            .iter()
            .filter(|job| job.job_status == "cancelled")
            .count(),
    }
}

pub(crate) fn diagnostic(
    request_id: &str,
    run_status: &str,
    summary: &BatchCopySummary,
    errors: impl Iterator<Item = String>,
    started: Instant,
) -> BatchCopyDiagnosticReport {
    BatchCopyDiagnosticReport {
        diagnostic_schema_version: BATCH_COPY_DIAGNOSTIC_SCHEMA_VERSION.to_string(),
        request_id: request_id.to_string(),
        service_version: BATCH_COPY_SERVICE_VERSION.to_string(),
        host_os: std::env::consts::OS.to_string(),
        host_arch: std::env::consts::ARCH.to_string(),
        run_status: run_status.to_string(),
        elapsed_ms: u64::try_from(started.elapsed().as_millis()).unwrap_or(u64::MAX),
        summary: summary.clone(),
        error_codes: errors.collect::<BTreeSet<_>>().into_iter().collect(),
    }
}

pub(crate) fn failed_execution(
    request: &crate::BatchExecuteCopyRequest,
    code: &str,
    stage: &str,
    message: &str,
    started: Instant,
) -> BatchCopyResult {
    let error = error(code, stage, message);
    let summary = BatchCopySummary {
        total_job_count: request.preview.jobs.len(),
        blocked_job_count: request.preview.jobs.len(),
        ..BatchCopySummary::default()
    };
    let diagnostic_report = diagnostic(
        &request.request_id,
        "blocked",
        &summary,
        std::iter::once(code.to_string()),
        started,
    );
    BatchCopyResult {
        service_version: BATCH_COPY_SERVICE_VERSION.to_string(),
        request_id: request.request_id.clone(),
        run_status: "blocked".to_string(),
        destination_parent: request.preview.destination_parent.clone(),
        jobs: Vec::new(),
        summary,
        diagnostic_report,
        errors: vec![error],
    }
}

pub(crate) fn error(code: &str, stage: &str, message: &str) -> DesktopApplicationError {
    DesktopApplicationError {
        error_code: code.to_string(),
        stage: stage.to_string(),
        message: message.to_string(),
    }
}
