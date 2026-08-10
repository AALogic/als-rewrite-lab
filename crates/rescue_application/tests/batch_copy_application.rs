use rescue_application::{
    execute_batch_copy_with_operations, prepare_batch_copy_with_operations, BatchCopyObserver,
    BatchCopyOperations, BatchExecuteCopyRequest, BatchPrepareCopyRequest, BatchProgressEvent,
    DesktopApplicationError, DesktopCopyPreview, DesktopCopyResult, DesktopExecuteCopyRequest,
    DesktopPrepareCopyRequest, ProjectSelection,
};
use std::collections::HashSet;
use std::path::{Path, PathBuf};

#[derive(Default)]
struct FakeOperations {
    calls: Vec<String>,
    prepare_requests: Vec<DesktopPrepareCopyRequest>,
    blocked_names: HashSet<String>,
    failed_names: HashSet<String>,
}

impl BatchCopyOperations for FakeOperations {
    fn prepare(&mut self, request: &DesktopPrepareCopyRequest) -> DesktopCopyPreview {
        let name = filename(&request.source_als_path);
        self.calls.push(format!("prepare:{name}"));
        self.prepare_requests.push(request.clone());
        let mut preview = preview_fixture();
        preview.request_id = request.request_id.clone();
        preview.source_als_path = request.source_als_path.clone();
        preview.target_project_root = request.target_project_root.clone();
        if self.blocked_names.contains(&name) {
            let error = test_error("TEST_PREVIEW_BLOCKED", "preview", "Fixture blocker");
            preview.preview_status = "copy_preview_failed".to_string();
            preview.source_als_sha256.clear();
            preview.plan_fingerprint = None;
            preview.errors = vec![error];
        }
        preview
    }

    fn execute(&mut self, request: &DesktopExecuteCopyRequest) -> DesktopCopyResult {
        let name = filename(&request.preview.source_als_path);
        self.calls.push(format!("execute:{name}"));
        let mut result = if self.failed_names.contains(&name) {
            failed_result_fixture()
        } else {
            result_fixture()
        };
        result.request_id = request.request_id.clone();
        result.final_target_root = (result.run_status == "complete_copy_ready_for_manual_check")
            .then(|| request.preview.target_project_root.clone());
        result
    }
}

#[derive(Default)]
struct CancelAfterFirst {
    cancelled: bool,
    events: Vec<BatchProgressEvent>,
}

impl BatchCopyObserver for CancelAfterFirst {
    fn on_progress(&mut self, event: &BatchProgressEvent) {
        self.events.push(event.clone());
        if event.stage == "job_completed" {
            self.cancelled = true;
        }
    }

    fn is_cancelled(&self) -> bool {
        self.cancelled
    }
}

fn selection(id: &str, path: &str) -> ProjectSelection {
    ProjectSelection {
        selection_id: id.to_string(),
        selection_source: "catalog".to_string(),
        live_set_id: Some(format!("set-{id}")),
        native_als_path: PathBuf::from(path),
        catalog_revision: Some(3),
        observation_fingerprint: Some(format!("fingerprint-{id}")),
        freshness_status: "observed_in_latest_scan".to_string(),
    }
}

fn prepare_request(selections: Vec<ProjectSelection>) -> BatchPrepareCopyRequest {
    BatchPrepareCopyRequest {
        request_id: "batch-preview".to_string(),
        selections,
        destination_parent: PathBuf::from("/target"),
        experimental_compatibility_consent: false,
    }
}

fn prepared_batch(operations: &mut FakeOperations) -> rescue_application::BatchCopyPreview {
    prepare_batch_copy_with_operations(
        &prepare_request(vec![
            selection("a", "/source/A.als"),
            selection("b", "/source/B.als"),
        ]),
        operations,
    )
}

fn execute_request(preview: rescue_application::BatchCopyPreview) -> BatchExecuteCopyRequest {
    BatchExecuteCopyRequest {
        request_id: "batch-execute".to_string(),
        preview,
        write_consent: true,
    }
}

fn preview_fixture() -> DesktopCopyPreview {
    serde_json::from_str(include_str!(
        "../../../contracts/desktop-ipc/v1/copy-preview.json"
    ))
    .expect("copy preview fixture")
}

fn result_fixture() -> DesktopCopyResult {
    serde_json::from_str(include_str!(
        "../../../contracts/desktop-ipc/v1/copy-result.json"
    ))
    .expect("copy result fixture")
}

fn failed_result_fixture() -> DesktopCopyResult {
    serde_json::from_str(include_str!(
        "../../../contracts/desktop-ipc/v1/copy-error-result.json"
    ))
    .expect("copy error result fixture")
}

fn filename(path: &Path) -> String {
    path.file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("unknown")
        .to_string()
}

fn test_error(code: &str, stage: &str, message: &str) -> DesktopApplicationError {
    DesktopApplicationError {
        error_code: code.to_string(),
        stage: stage.to_string(),
        message: message.to_string(),
    }
}

#[test]
fn all_previews_finish_before_first_write() {
    let mut operations = FakeOperations::default();
    let preview = prepared_batch(&mut operations);
    assert_eq!(operations.calls, ["prepare:A.als", "prepare:B.als"]);
    assert_eq!(preview.preview_status, "ready");

    let mut observer = ();
    let _ = execute_batch_copy_with_operations(
        &execute_request(preview),
        &mut operations,
        &mut observer,
    );
    assert_eq!(
        operations.calls,
        [
            "prepare:A.als",
            "prepare:B.als",
            "execute:A.als",
            "execute:B.als"
        ]
    );
}

#[test]
fn unchanged_one_project_contract_is_reused() {
    let mut operations = FakeOperations::default();
    let preview = prepared_batch(&mut operations);
    assert_eq!(
        operations.prepare_requests[0].source_als_path,
        PathBuf::from("/source/A.als")
    );
    assert_eq!(
        operations.prepare_requests[0].target_project_root,
        PathBuf::from("/target/A Rescue Project")
    );
    assert!(!operations.prepare_requests[0].experimental_compatibility_consent);
    assert_eq!(
        preview.jobs[0].preview.as_ref(),
        Some(&preview_fixture_with_paths(
            "/source/A.als",
            "/target/A Rescue Project",
            &preview.jobs[0]
        ))
    );
}

fn preview_fixture_with_paths(
    source: &str,
    target: &str,
    job: &rescue_application::BatchPreviewJob,
) -> DesktopCopyPreview {
    let mut fixture = preview_fixture();
    fixture.request_id = job
        .preview
        .as_ref()
        .map(|preview| preview.request_id.clone())
        .unwrap_or_default();
    fixture.source_als_path = PathBuf::from(source);
    fixture.target_project_root = PathBuf::from(target);
    fixture
}

#[test]
fn target_collision_blocks_all_colliding_jobs() {
    let mut operations = FakeOperations::default();
    let preview = prepare_batch_copy_with_operations(
        &prepare_request(vec![
            selection("a", "/one/Same.als"),
            selection("b", "/two/Same.als"),
        ]),
        &mut operations,
    );
    assert!(operations.calls.is_empty());
    assert_eq!(preview.preview_status, "blocked");
    assert!(preview
        .jobs
        .iter()
        .all(|job| job.job_status == "target_collision"));
}

#[test]
fn blocked_preview_does_not_block_ready_job() {
    let mut operations = FakeOperations {
        blocked_names: HashSet::from(["A.als".to_string()]),
        ..FakeOperations::default()
    };
    let preview = prepared_batch(&mut operations);
    assert_eq!(preview.preview_status, "partially_ready");
    let result =
        execute_batch_copy_with_operations(&execute_request(preview), &mut operations, &mut ());
    assert_eq!(
        operations.calls.last().map(String::as_str),
        Some("execute:B.als")
    );
    assert_eq!(result.jobs[0].job_status, "skipped_blocked");
    assert_eq!(result.jobs[1].job_status, "completed");
}

#[test]
fn execution_is_strictly_sequential() {
    let mut operations = FakeOperations::default();
    let preview = prepared_batch(&mut operations);
    let _ = execute_batch_copy_with_operations(&execute_request(preview), &mut operations, &mut ());
    assert_eq!(&operations.calls[2..], ["execute:A.als", "execute:B.als"]);
}

#[test]
fn failed_job_does_not_stop_later_job() {
    let mut operations = FakeOperations {
        failed_names: HashSet::from(["A.als".to_string()]),
        ..FakeOperations::default()
    };
    let preview = prepared_batch(&mut operations);
    let result =
        execute_batch_copy_with_operations(&execute_request(preview), &mut operations, &mut ());
    assert_eq!(result.jobs[0].job_status, "failed");
    assert_eq!(result.jobs[1].job_status, "completed");
    assert_eq!(result.run_status, "completed_with_issues");
}

#[test]
fn cancellation_stops_before_next_job() {
    let mut operations = FakeOperations::default();
    let preview = prepared_batch(&mut operations);
    let mut observer = CancelAfterFirst::default();
    let result = execute_batch_copy_with_operations(
        &execute_request(preview),
        &mut operations,
        &mut observer,
    );
    assert_eq!(&operations.calls[2..], ["execute:A.als"]);
    assert_eq!(result.jobs[0].job_status, "completed");
    assert_eq!(result.jobs[1].job_status, "cancelled");
    assert_eq!(result.run_status, "cancelled");
}

#[test]
fn no_batch_write_without_consent() {
    let mut operations = FakeOperations::default();
    let preview = prepared_batch(&mut operations);
    let mut request = execute_request(preview);
    request.write_consent = false;
    let result = execute_batch_copy_with_operations(&request, &mut operations, &mut ());
    assert_eq!(result.run_status, "blocked");
    assert_eq!(operations.calls, ["prepare:A.als", "prepare:B.als"]);
}

#[test]
fn tampered_batch_summary_is_rejected_without_write() {
    let mut operations = FakeOperations::default();
    let mut preview = prepared_batch(&mut operations);
    preview.summary.ready_job_count += 1;
    let result =
        execute_batch_copy_with_operations(&execute_request(preview), &mut operations, &mut ());
    assert_eq!(result.run_status, "blocked");
    assert_eq!(result.errors[0].error_code, "BATCH_PREVIEW_INVALID");
    assert_eq!(operations.calls, ["prepare:A.als", "prepare:B.als"]);
}

#[test]
fn ready_target_outside_destination_is_rejected_without_write() {
    let mut operations = FakeOperations::default();
    let mut preview = prepared_batch(&mut operations);
    let outside = PathBuf::from("/outside/A Rescue Project");
    preview.jobs[0].target_project_root = outside.clone();
    preview.jobs[0]
        .preview
        .as_mut()
        .expect("ready preview")
        .target_project_root = outside;
    let result =
        execute_batch_copy_with_operations(&execute_request(preview), &mut operations, &mut ());
    assert_eq!(result.run_status, "blocked");
    assert_eq!(result.errors[0].error_code, "BATCH_PREVIEW_INVALID");
    assert_eq!(operations.calls, ["prepare:A.als", "prepare:B.als"]);
}

#[test]
fn batch_result_preserves_selection_order() {
    let mut operations = FakeOperations::default();
    let preview = prepared_batch(&mut operations);
    let result =
        execute_batch_copy_with_operations(&execute_request(preview), &mut operations, &mut ());
    assert_eq!(
        result
            .jobs
            .iter()
            .map(|job| job.selection_id.as_str())
            .collect::<Vec<_>>(),
        ["a", "b"]
    );
}

#[test]
fn batch_diagnostic_is_path_free() {
    let mut operations = FakeOperations::default();
    let preview = prepared_batch(&mut operations);
    let serialized = serde_json::to_string(&preview.diagnostic_report).expect("diagnostic JSON");
    assert!(!serialized.contains("/source"));
    assert!(!serialized.contains("A.als"));
    assert!(!serialized.contains("/target"));
}

#[test]
fn batch_wire_contract_is_stable() {
    let mut operations = FakeOperations::default();
    let preview = prepared_batch(&mut operations);
    let value = serde_json::to_value(preview).expect("batch wire JSON");
    let keys = value.as_object().expect("batch preview object");
    assert_eq!(keys.len(), 8);
    for key in [
        "service_version",
        "request_id",
        "preview_status",
        "destination_parent",
        "jobs",
        "summary",
        "diagnostic_report",
        "errors",
    ] {
        assert!(keys.contains_key(key));
    }
    let shared_preview: rescue_application::BatchCopyPreview = serde_json::from_str(include_str!(
        "../../../contracts/desktop-ipc/v1/batch-preview.json"
    ))
    .expect("shared batch preview fixture");
    let shared_result: rescue_application::BatchCopyResult = serde_json::from_str(include_str!(
        "../../../contracts/desktop-ipc/v1/batch-result.json"
    ))
    .expect("shared batch result fixture");
    assert_eq!(shared_preview.jobs[0].job_status, "ready");
    assert_eq!(shared_result.jobs[0].job_status, "completed");
}
