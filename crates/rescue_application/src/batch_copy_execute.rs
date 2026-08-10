use crate::{
    BatchCopyJobResult, BatchCopyObserver, BatchCopyOperations, BatchCopyResult,
    BatchExecuteCopyRequest, BatchProgressEvent, DesktopExecuteCopyRequest,
    BATCH_COPY_SERVICE_VERSION,
};
use std::time::Instant;

pub(crate) fn execute_batch_copy_impl(
    request: &BatchExecuteCopyRequest,
    operations: &mut dyn BatchCopyOperations,
    observer: &mut dyn BatchCopyObserver,
) -> BatchCopyResult {
    let started = Instant::now();
    if let Some((code, stage, message)) = crate::batch_copy_validation::validate_execution(request)
    {
        return crate::batch_copy_result::failed_execution(request, code, stage, message, started);
    }
    emit(observer, request, "started", None, 0, 0, 0);
    let mut jobs = Vec::with_capacity(request.preview.jobs.len());
    let mut completed = 0usize;
    let mut failed = 0usize;
    let mut cancelled = false;
    for (index, preview_job) in request.preview.jobs.iter().enumerate() {
        let result = if preview_job.job_status != "ready" {
            skipped_job(preview_job)
        } else if cancelled || observer.is_cancelled() {
            cancelled = true;
            cancelled_job(preview_job)
        } else {
            emit(
                observer,
                request,
                "job_started",
                Some(&preview_job.job_id),
                index,
                completed,
                failed,
            );
            execute_ready_job(request, preview_job, operations)
        };
        if matches!(
            result.job_status.as_str(),
            "completed" | "completed_incomplete"
        ) {
            completed += 1;
        } else if result.job_status == "failed" {
            failed += 1;
        }
        emit(
            observer,
            request,
            &progress_stage(&result),
            Some(&result.job_id),
            index,
            completed,
            failed,
        );
        jobs.push(result);
    }
    complete_result(request, jobs, started, observer)
}

fn execute_ready_job(
    request: &BatchExecuteCopyRequest,
    job: &crate::BatchPreviewJob,
    operations: &mut dyn BatchCopyOperations,
) -> BatchCopyJobResult {
    let Some(preview) = job.preview.clone() else {
        return invalid_ready_job(job);
    };
    let result = operations.execute(&DesktopExecuteCopyRequest {
        request_id: format!("{}:{}:execute", request.request_id, job.job_id),
        preview,
        write_consent: true,
    });
    let status = match result.run_status.as_str() {
        "complete_copy_ready_for_manual_check" => "completed",
        "incomplete_copy_ready_for_manual_check" => "completed_incomplete",
        _ => "failed",
    };
    BatchCopyJobResult {
        job_id: job.job_id.clone(),
        selection_id: job.selection_id.clone(),
        source_als_path: job.source_als_path.clone(),
        target_project_root: job.target_project_root.clone(),
        job_status: status.to_string(),
        errors: result.errors.clone(),
        result: Some(result),
    }
}

fn invalid_ready_job(job: &crate::BatchPreviewJob) -> BatchCopyJobResult {
    let error = crate::batch_copy_result::error(
        "BATCH_PREVIEW_INVALID",
        "preview_validation",
        "Ready batch job contains no executable one-project preview.",
    );
    BatchCopyJobResult {
        job_id: job.job_id.clone(),
        selection_id: job.selection_id.clone(),
        source_als_path: job.source_als_path.clone(),
        target_project_root: job.target_project_root.clone(),
        job_status: "failed".to_string(),
        result: None,
        errors: vec![error],
    }
}

fn skipped_job(job: &crate::BatchPreviewJob) -> BatchCopyJobResult {
    BatchCopyJobResult {
        job_id: job.job_id.clone(),
        selection_id: job.selection_id.clone(),
        source_als_path: job.source_als_path.clone(),
        target_project_root: job.target_project_root.clone(),
        job_status: "skipped_blocked".to_string(),
        result: None,
        errors: job.errors.clone(),
    }
}

fn cancelled_job(job: &crate::BatchPreviewJob) -> BatchCopyJobResult {
    BatchCopyJobResult {
        job_id: job.job_id.clone(),
        selection_id: job.selection_id.clone(),
        source_als_path: job.source_als_path.clone(),
        target_project_root: job.target_project_root.clone(),
        job_status: "cancelled".to_string(),
        result: None,
        errors: Vec::new(),
    }
}

fn complete_result(
    request: &BatchExecuteCopyRequest,
    jobs: Vec<BatchCopyJobResult>,
    started: Instant,
    observer: &mut dyn BatchCopyObserver,
) -> BatchCopyResult {
    let summary = crate::batch_copy_result::result_summary(&request.preview, &jobs);
    let status = result_status(&summary);
    let error_codes = jobs.iter().flat_map(|job| {
        job.errors
            .iter()
            .map(|error| error.error_code.clone())
            .collect::<Vec<_>>()
    });
    let diagnostic_report = crate::batch_copy_result::diagnostic(
        &request.request_id,
        status,
        &summary,
        error_codes,
        started,
    );
    emit(
        observer,
        request,
        status,
        None,
        jobs.len(),
        summary.completed_job_count + summary.incomplete_job_count,
        summary.failed_job_count,
    );
    BatchCopyResult {
        service_version: BATCH_COPY_SERVICE_VERSION.to_string(),
        request_id: request.request_id.clone(),
        run_status: status.to_string(),
        destination_parent: request.preview.destination_parent.clone(),
        jobs,
        summary,
        diagnostic_report,
        errors: Vec::new(),
    }
}

fn result_status(summary: &crate::BatchCopySummary) -> &'static str {
    if summary.cancelled_job_count > 0 {
        "cancelled"
    } else if summary.ready_job_count == 0 {
        "blocked"
    } else if summary.failed_job_count > 0 || summary.blocked_job_count > 0 {
        "completed_with_issues"
    } else {
        "completed"
    }
}

fn progress_stage(result: &BatchCopyJobResult) -> String {
    format!("job_{}", result.job_status)
}

#[allow(clippy::too_many_arguments)]
fn emit(
    observer: &mut dyn BatchCopyObserver,
    request: &BatchExecuteCopyRequest,
    stage: &str,
    job_id: Option<&str>,
    job_index: usize,
    completed: usize,
    failed: usize,
) {
    observer.on_progress(&BatchProgressEvent {
        request_id: request.request_id.clone(),
        stage: stage.to_string(),
        job_id: job_id.map(str::to_string),
        job_index,
        total_job_count: request.preview.jobs.len(),
        completed_job_count: completed,
        failed_job_count: failed,
    });
}
