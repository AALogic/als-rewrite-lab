use rescue_application::{
    BatchCopyDiagnosticReport, BatchCopyJobResult, BatchCopyResult, BatchCopySummary,
    CourierBatchWave, CourierCollectionOrchestrator, DesktopCopyDiagnosticReport,
    DesktopCopyResult, ProjectSelection,
};
use std::path::{Path, PathBuf};

fn selection(root: &Path, name: &str) -> ProjectSelection {
    let path = root.join(format!("{name}.als"));
    std::fs::write(&path, name.as_bytes()).expect("ALS fixture");
    ProjectSelection {
        selection_id: format!("selection-{name}"),
        selection_source: "manual".to_string(),
        live_set_id: None,
        native_als_path: path,
        catalog_revision: None,
        observation_fingerprint: None,
        freshness_status: "manual_unverified".to_string(),
    }
}

fn orchestrator(root: &Path) -> CourierCollectionOrchestrator {
    CourierCollectionOrchestrator::new("collection-1".to_string(), root.join("output"))
        .expect("orchestrator")
}

fn add(
    orchestrator: &mut CourierCollectionOrchestrator,
    selection: ProjectSelection,
    name: &str,
) -> String {
    orchestrator
        .add_selection(selection, format!("{name}.als"))
        .expect("queue item")
}

fn batch_result(wave: &CourierBatchWave, root: &Path, statuses: &[&str]) -> BatchCopyResult {
    let jobs = wave
        .selections
        .iter()
        .zip(statuses)
        .enumerate()
        .map(|(index, (selection, status))| {
            let target = root.join(format!("Target-{index} Rescue Project"));
            let copy = if matches!(*status, "completed" | "completed_incomplete") {
                std::fs::create_dir_all(&target).expect("target fixture");
                Some(copy_result(
                    target.clone(),
                    &format!("{}:result-{index}", wave.wave_id),
                    *status == "completed_incomplete",
                ))
            } else {
                None
            };
            BatchCopyJobResult {
                job_id: format!("{}:job-{index}", wave.wave_id),
                selection_id: selection.selection_id.clone(),
                source_als_path: selection.native_als_path.clone(),
                target_project_root: target,
                job_status: (*status).to_string(),
                result: copy,
                errors: Vec::new(),
            }
        })
        .collect::<Vec<_>>();
    let summary = BatchCopySummary {
        total_job_count: jobs.len(),
        ready_job_count: jobs.len(),
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
    };
    BatchCopyResult {
        service_version: "0.1.0".to_string(),
        request_id: wave.wave_id.clone(),
        run_status: "completed".to_string(),
        destination_parent: wave.destination_parent.clone(),
        jobs,
        diagnostic_report: BatchCopyDiagnosticReport {
            diagnostic_schema_version: "0.1".to_string(),
            request_id: wave.wave_id.clone(),
            service_version: "0.1.0".to_string(),
            host_os: "test".to_string(),
            host_arch: "test".to_string(),
            run_status: "completed".to_string(),
            elapsed_ms: 1,
            summary: summary.clone(),
            error_codes: Vec::new(),
        },
        summary,
        errors: Vec::new(),
    }
}

fn copy_result(target: PathBuf, request_id: &str, incomplete: bool) -> DesktopCopyResult {
    let run_status = if incomplete {
        "incomplete_copy_ready_for_manual_check"
    } else {
        "complete_copy_ready_for_manual_check"
    };
    DesktopCopyResult {
        service_version: "0.6.0".to_string(),
        request_id: request_id.to_string(),
        run_status: run_status.to_string(),
        final_target_root: Some(target),
        system_dependency_count: 0,
        copied_asset_count: 1,
        rewritten_reference_count: 1,
        omitted_asset_count: usize::from(incomplete),
        diagnostic_report: DesktopCopyDiagnosticReport {
            diagnostic_schema_version: "0.2".to_string(),
            request_id: request_id.to_string(),
            operation_kind: "execute_copy".to_string(),
            service_version: "0.6.0".to_string(),
            pipeline_version: "test".to_string(),
            build_commit: "test".to_string(),
            host_os: "test".to_string(),
            host_arch: "test".to_string(),
            rewrite_policy: "strict".to_string(),
            ableton_document_version: None,
            ableton_creator_version: None,
            ableton_minor_version: None,
            compatibility_status: "confirmed_profile".to_string(),
            run_status: run_status.to_string(),
            completed_stage: "complete".to_string(),
            elapsed_ms: 1,
            required_asset_count: 1,
            system_dependency_count: 0,
            copied_asset_count: 1,
            rewritten_reference_count: 1,
            omitted_asset_count: usize::from(incomplete),
            errors: Vec::new(),
        },
        errors: Vec::new(),
    }
}

#[test]
fn play_freezes_all_pending_items_into_one_wave() {
    let temp = tempfile::tempdir().expect("fixture");
    let mut state = orchestrator(temp.path());
    add(&mut state, selection(temp.path(), "A"), "A");
    add(&mut state, selection(temp.path(), "B"), "B");
    let wave = state.start_processing().expect("wave");
    assert_eq!(wave.selections.len(), 2);
    assert_eq!(state.snapshot().queue.pending_item_count, 0);
}

#[test]
fn intake_during_active_wave_enters_next_wave() {
    let temp = tempfile::tempdir().expect("fixture");
    let mut state = orchestrator(temp.path());
    add(&mut state, selection(temp.path(), "A"), "A");
    let first = state.start_processing().expect("first wave");
    add(&mut state, selection(temp.path(), "B"), "B");
    state
        .merge_wave_result(
            &first.wave_id,
            &batch_result(&first, temp.path(), &["completed"]),
        )
        .expect("merge");
    let second = state.next_wave().expect("next").expect("second wave");
    assert_eq!(second.selections[0].selection_id, "selection-B");
}

#[test]
fn drain_boundary_never_loses_or_duplicates_item() {
    let temp = tempfile::tempdir().expect("fixture");
    let mut state = orchestrator(temp.path());
    add(&mut state, selection(temp.path(), "A"), "A");
    let first = state.start_processing().expect("first");
    state
        .merge_wave_result(
            &first.wave_id,
            &batch_result(&first, temp.path(), &["completed"]),
        )
        .expect("merge");
    add(&mut state, selection(temp.path(), "B"), "B");
    let second = state.start_processing().expect("second");
    assert_eq!(second.selections.len(), 1);
    assert_eq!(second.selections[0].selection_id, "selection-B");
}

#[test]
fn active_wave_is_never_mutated() {
    let temp = tempfile::tempdir().expect("fixture");
    let mut state = orchestrator(temp.path());
    add(&mut state, selection(temp.path(), "A"), "A");
    let frozen = state.start_processing().expect("wave");
    let before = frozen.clone();
    add(&mut state, selection(temp.path(), "B"), "B");
    assert_eq!(frozen, before);
    assert_eq!(frozen.selections.len(), 1);
}

#[test]
fn successful_waves_accumulate_one_collection() {
    let temp = tempfile::tempdir().expect("fixture");
    let mut state = orchestrator(temp.path());
    add(&mut state, selection(temp.path(), "A"), "A");
    let first = state.start_processing().expect("first");
    add(&mut state, selection(temp.path(), "B"), "B");
    state
        .merge_wave_result(
            &first.wave_id,
            &batch_result(&first, temp.path(), &["completed"]),
        )
        .expect("first merge");
    let second = state.next_wave().expect("next").expect("second");
    let snapshot = state
        .merge_wave_result(
            &second.wave_id,
            &batch_result(&second, temp.path(), &["completed_incomplete"]),
        )
        .expect("second merge");
    assert_eq!(snapshot.items.len(), 2);
    assert_eq!(snapshot.revision, 2);
    assert_eq!(snapshot.work_phase, "ready");
}

#[test]
fn failed_and_blocked_jobs_do_not_enter_collection() {
    let temp = tempfile::tempdir().expect("fixture");
    let mut state = orchestrator(temp.path());
    add(&mut state, selection(temp.path(), "A"), "A");
    add(&mut state, selection(temp.path(), "B"), "B");
    let wave = state.start_processing().expect("wave");
    let snapshot = state
        .merge_wave_result(
            &wave.wave_id,
            &batch_result(&wave, temp.path(), &["failed", "skipped_blocked"]),
        )
        .expect("merge");
    assert!(snapshot.items.is_empty());
    assert_eq!(snapshot.failed_item_count, 2);
}

#[test]
fn ready_collection_returns_to_collecting_after_new_intake() {
    let temp = tempfile::tempdir().expect("fixture");
    let mut state = orchestrator(temp.path());
    add(&mut state, selection(temp.path(), "A"), "A");
    let wave = state.start_processing().expect("wave");
    state
        .merge_wave_result(
            &wave.wave_id,
            &batch_result(&wave, temp.path(), &["completed"]),
        )
        .expect("merge");
    add(&mut state, selection(temp.path(), "B"), "B");
    assert_eq!(state.snapshot().work_phase, "collecting");
    assert_eq!(state.snapshot().items.len(), 1);
}

#[test]
fn delivery_snapshot_is_revision_bound_and_immutable() {
    let temp = tempfile::tempdir().expect("fixture");
    let mut state = orchestrator(temp.path());
    add(&mut state, selection(temp.path(), "A"), "A");
    let wave = state.start_processing().expect("wave");
    state
        .merge_wave_result(
            &wave.wave_id,
            &batch_result(&wave, temp.path(), &["completed"]),
        )
        .expect("merge");
    let delivery = state.delivery_snapshot().expect("delivery");
    add(&mut state, selection(temp.path(), "B"), "B");
    assert_eq!(delivery.revision, 1);
    assert_eq!(delivery.items.len(), 1);
}

#[test]
fn wave_result_identity_mismatch_fails_closed() {
    let temp = tempfile::tempdir().expect("fixture");
    let mut state = orchestrator(temp.path());
    add(&mut state, selection(temp.path(), "A"), "A");
    let wave = state.start_processing().expect("wave");
    let result = batch_result(&wave, temp.path(), &["completed"]);
    let failure = state
        .merge_wave_result("wrong-wave", &result)
        .expect_err("mismatch");
    assert_eq!(failure.error_code, "COURIER_WAVE_RESULT_MISMATCH");
    assert!(state.delivery_snapshot().is_err());
}

#[test]
fn orchestrator_errors_are_path_free() {
    let secret = PathBuf::from("/Users/private/Secret Destination");
    let mut state = CourierCollectionOrchestrator::new("collection".to_string(), secret.clone())
        .expect("state");
    let failure = state.start_processing().expect_err("empty");
    let json = serde_json::to_string(&failure).expect("serialize");
    assert!(!json.contains(&secret.to_string_lossy().to_string()));
}

fn ready_collection(root: &Path) -> CourierCollectionOrchestrator {
    let mut state = orchestrator(root);
    add(&mut state, selection(root, "handoff"), "handoff");
    let wave = state.start_processing().expect("wave");
    state
        .merge_wave_result(&wave.wave_id, &batch_result(&wave, root, &["completed"]))
        .expect("ready collection");
    state
}

#[test]
fn only_one_handoff_can_be_active() {
    let temp = tempfile::tempdir().expect("fixture");
    let mut state = ready_collection(temp.path());
    state
        .begin_handoff("collection-1", 1, "native_drag")
        .expect("first handoff");
    let failure = state
        .begin_handoff("collection-1", 1, "local_google_drive")
        .expect_err("second handoff");
    assert_eq!(failure.error_code, "COURIER_HANDOFF_BUSY");
}

#[test]
fn handoff_identity_and_revision_must_match() {
    let temp = tempfile::tempdir().expect("fixture");
    let mut state = ready_collection(temp.path());
    for (collection_id, revision) in [("stale", 1), ("collection-1", 9)] {
        let failure = state
            .begin_handoff(collection_id, revision, "native_drag")
            .expect_err("stale handoff");
        assert_eq!(failure.error_code, "COURIER_HANDOFF_STALE");
    }
}

#[test]
fn completed_handoff_is_terminal_for_direct_intake() {
    let temp = tempfile::tempdir().expect("fixture");
    let mut state = ready_collection(temp.path());
    state
        .begin_handoff("collection-1", 1, "native_drag")
        .expect("begin");
    state
        .complete_handoff("collection-1", 1, "native_drag")
        .expect("complete");
    let failure = state
        .add_selection(selection(temp.path(), "next"), "next.als".to_string())
        .expect_err("terminal intake");
    assert_eq!(failure.error_code, "COURIER_HANDOFF_COMPLETED");
    assert_eq!(state.snapshot().items.len(), 1);
}

#[test]
fn cancelled_and_retryable_handoffs_keep_payload_available() {
    let temp = tempfile::tempdir().expect("fixture");
    let mut state = ready_collection(temp.path());
    state
        .begin_handoff("collection-1", 1, "native_drag")
        .expect("begin drag");
    state
        .cancel_handoff("collection-1", 1, "native_drag")
        .expect("cancel drag");
    assert_eq!(state.snapshot().handoff.status, "not_started");
    assert!(state.delivery_snapshot().is_ok());

    state
        .begin_handoff("collection-1", 1, "local_google_drive")
        .expect("begin delivery");
    state
        .mark_handoff_retryable("collection-1", 1, "local_google_drive")
        .expect("retryable delivery");
    assert_eq!(state.snapshot().handoff.status, "retryable_issue");
    assert!(state.delivery_snapshot().is_ok());
}

#[test]
fn explicit_reopen_preserves_collection_revision() {
    let temp = tempfile::tempdir().expect("fixture");
    let mut state = ready_collection(temp.path());
    let before = state.delivery_snapshot().expect("before");
    state
        .begin_handoff("collection-1", 1, "local_google_drive")
        .expect("begin");
    state
        .complete_handoff("collection-1", 1, "local_google_drive")
        .expect("complete");
    assert!(state.delivery_snapshot().is_err());
    state.reopen_handoff().expect("reopen");
    let after = state.delivery_snapshot().expect("after");
    assert_eq!(before, after);
}
