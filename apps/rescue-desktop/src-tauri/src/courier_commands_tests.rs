use super::*;
use rescue_application::{
    BatchCopyDiagnosticReport, BatchCopyJobResult, BatchCopyResult, BatchCopySummary,
    DesktopCopyDiagnosticReport, DesktopCopyResult,
};

fn selection(root: &Path, name: &str) -> ProjectSelection {
    let path = root.join(format!("{name}.als"));
    std::fs::write(&path, name.as_bytes()).expect("ALS fixture");
    ProjectSelection {
        selection_id: format!("selection-{name}"),
        selection_source: "test".to_string(),
        live_set_id: None,
        native_als_path: path,
        catalog_revision: None,
        observation_fingerprint: None,
        freshness_status: "manual_unverified".to_string(),
    }
}

fn ready_orchestrator(root: &Path, id: &str) -> CourierCollectionOrchestrator {
    let mut orchestrator =
        CourierCollectionOrchestrator::new(id.to_string(), root.join("output")).expect("state");
    orchestrator
        .add_selection(selection(root, "old"), "old.als".to_string())
        .expect("intake");
    let wave = orchestrator.start_processing().expect("wave");
    let target = root.join("Old Rescue Project");
    std::fs::create_dir_all(&target).expect("target");
    let result = completed_batch_result(&wave, target);
    orchestrator
        .merge_wave_result(&wave.wave_id, &result)
        .expect("merge");
    orchestrator
}

fn completed_batch_result(wave: &CourierBatchWave, target: PathBuf) -> BatchCopyResult {
    let copy = DesktopCopyResult {
        service_version: "test".to_string(),
        request_id: format!("{}:result", wave.wave_id),
        run_status: "complete_copy_ready_for_manual_check".to_string(),
        final_target_root: Some(target.clone()),
        system_dependency_count: 0,
        copied_asset_count: 1,
        rewritten_reference_count: 1,
        omitted_asset_count: 0,
        diagnostic_report: DesktopCopyDiagnosticReport {
            diagnostic_schema_version: "0.2".to_string(),
            request_id: format!("{}:result", wave.wave_id),
            operation_kind: "execute_copy".to_string(),
            service_version: "test".to_string(),
            pipeline_version: "test".to_string(),
            build_commit: "test".to_string(),
            host_os: "test".to_string(),
            host_arch: "test".to_string(),
            rewrite_policy: "strict".to_string(),
            ableton_document_version: None,
            ableton_creator_version: None,
            ableton_minor_version: None,
            compatibility_status: "confirmed_profile".to_string(),
            run_status: "complete_copy_ready_for_manual_check".to_string(),
            completed_stage: "complete".to_string(),
            elapsed_ms: 1,
            required_asset_count: 1,
            system_dependency_count: 0,
            copied_asset_count: 1,
            rewritten_reference_count: 1,
            omitted_asset_count: 0,
            errors: Vec::new(),
        },
        errors: Vec::new(),
    };
    let job = BatchCopyJobResult {
        job_id: format!("{}:job", wave.wave_id),
        selection_id: wave.selections[0].selection_id.clone(),
        source_als_path: wave.selections[0].native_als_path.clone(),
        target_project_root: target,
        job_status: "completed".to_string(),
        result: Some(copy),
        errors: Vec::new(),
    };
    let summary = BatchCopySummary {
        total_job_count: 1,
        ready_job_count: 1,
        blocked_job_count: 0,
        completed_job_count: 1,
        incomplete_job_count: 0,
        failed_job_count: 0,
        cancelled_job_count: 0,
    };
    BatchCopyResult {
        service_version: "test".to_string(),
        request_id: wave.wave_id.clone(),
        run_status: "completed".to_string(),
        destination_parent: wave.destination_parent.clone(),
        jobs: vec![job],
        diagnostic_report: BatchCopyDiagnosticReport {
            diagnostic_schema_version: "0.1".to_string(),
            request_id: wave.wave_id.clone(),
            service_version: "test".to_string(),
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

#[test]
fn completed_handoff_new_intake_starts_fresh_collection() {
    let temp = tempfile::tempdir().expect("fixture");
    let mut old = ready_orchestrator(temp.path(), "old-collection");
    old.begin_handoff("old-collection", 1, NATIVE_HANDOFF_CHANNEL)
        .expect("begin");
    old.complete_handoff("old-collection", 1, NATIVE_HANDOFF_CHANNEL)
        .expect("complete");
    let mut store = CourierRuntimeStore {
        orchestrator: Some(old),
        delivered_item_ids: HashSet::from(["old-item".to_string()]),
        pending_next_order: Vec::new(),
    };
    start_fresh_order_if_completed(&mut store, &temp.path().join("output")).expect("fresh order");
    let next = store.orchestrator.as_mut().expect("next state");
    next.add_selection(selection(temp.path(), "new"), "new.als".to_string())
        .expect("new intake");
    let snapshot = next.snapshot();
    assert_ne!(snapshot.collection_id, "old-collection");
    assert_eq!(snapshot.queue.items.len(), 1);
    assert_eq!(snapshot.queue.items[0].source_display_name, "new.als");
    assert!(store.delivered_item_ids.is_empty());
}

#[test]
fn intake_during_handoff_starts_next_order_exactly_once() {
    let temp = tempfile::tempdir().expect("fixture");
    let mut old = ready_orchestrator(temp.path(), "old-collection");
    old.begin_handoff("old-collection", 1, LOCAL_HANDOFF_CHANNEL)
        .expect("begin");
    old.complete_handoff("old-collection", 1, LOCAL_HANDOFF_CHANNEL)
        .expect("complete");
    let mut store = CourierRuntimeStore {
        orchestrator: Some(old),
        delivered_item_ids: HashSet::new(),
        pending_next_order: vec![(selection(temp.path(), "next"), "next.als".to_string())],
    };
    rotate_pending_next_order(&mut store).expect("rotate");
    let snapshot = store.orchestrator.as_ref().expect("next state").snapshot();
    assert_ne!(snapshot.collection_id, "old-collection");
    assert_eq!(snapshot.queue.items.len(), 1);
    assert_eq!(snapshot.queue.pending_item_count, 1);
    assert!(store.pending_next_order.is_empty());
}

#[test]
fn cancelled_handoff_merges_deferred_intake_into_current_order() {
    let temp = tempfile::tempdir().expect("fixture");
    let mut current = ready_orchestrator(temp.path(), "collection");
    current
        .begin_handoff("collection", 1, NATIVE_HANDOFF_CHANNEL)
        .expect("begin");
    current
        .cancel_handoff("collection", 1, NATIVE_HANDOFF_CHANNEL)
        .expect("cancel");
    let mut store = CourierRuntimeStore {
        orchestrator: Some(current),
        delivered_item_ids: HashSet::new(),
        pending_next_order: vec![(selection(temp.path(), "next"), "next.als".to_string())],
    };
    merge_pending_into_active(&mut store).expect("merge pending");
    let snapshot = store.orchestrator.as_ref().expect("state").snapshot();
    assert_eq!(snapshot.collection_id, "collection");
    assert_eq!(snapshot.work_phase, "collecting");
    assert_eq!(snapshot.queue.items.len(), 2);
    assert!(store.pending_next_order.is_empty());
}
