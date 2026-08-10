use crate::courier_preferences::{load_for_app, save_for_app, CourierPreferences};
use rescue_application::{
    BatchExecuteCopyRequest, BatchPrepareCopyRequest, CourierBatchWave,
    CourierCollectionOrchestrator, CourierCollectionSnapshot, DesktopApplicationError,
    ProjectSelection,
};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Mutex;
use tauri::{AppHandle, Emitter, Manager, State};

pub(crate) const COURIER_SNAPSHOT_EVENT: &str = "courier-snapshot-changed";
#[cfg(any(target_os = "macos", test))]
pub(crate) const NATIVE_HANDOFF_CHANNEL: &str = "native_drag";
pub(crate) const LOCAL_HANDOFF_CHANNEL: &str = "local_google_drive";
static COLLECTION_SEQUENCE: AtomicU64 = AtomicU64::new(1);
static SELECTION_SEQUENCE: AtomicU64 = AtomicU64::new(1);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CourierDisplayItem {
    pub work_item_id: String,
    pub source_display_name: String,
    pub status: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CourierDisplaySnapshot {
    pub schema_version: String,
    pub collection_id: Option<String>,
    pub revision: u64,
    pub work_phase: String,
    pub handoff_status: String,
    pub handoff_channel: Option<String>,
    pub destination_display_path: String,
    pub items: Vec<CourierDisplayItem>,
    pub pending_item_count: usize,
    pub completed_item_count: usize,
    pub incomplete_item_count: usize,
    pub failed_item_count: usize,
    pub omitted_asset_count: usize,
    pub delivery_available: bool,
    pub local_delivery_configured: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CourierIntakeResult {
    pub schema_version: String,
    pub accepted_count: usize,
    pub refused_count: usize,
    pub error_codes: Vec<String>,
    pub snapshot: CourierDisplaySnapshot,
}

#[derive(Default)]
struct CourierRuntimeStore {
    orchestrator: Option<CourierCollectionOrchestrator>,
    delivered_item_ids: HashSet<String>,
    pending_next_order: Vec<(ProjectSelection, String)>,
}

#[derive(Default)]
pub(crate) struct CourierRuntimeState {
    store: Mutex<CourierRuntimeStore>,
}

#[tauri::command]
pub(crate) fn get_courier_snapshot(
    app: AppHandle,
    state: State<'_, CourierRuntimeState>,
) -> Result<CourierDisplaySnapshot, DesktopApplicationError> {
    display_snapshot(&app, &state)
}

#[tauri::command(rename_all = "snake_case")]
pub(crate) fn add_courier_sources(
    app: AppHandle,
    state: State<'_, CourierRuntimeState>,
    source_als_paths: Vec<PathBuf>,
) -> Result<CourierIntakeResult, DesktopApplicationError> {
    intake_paths(&app, &state, &source_als_paths)
}

#[tauri::command(rename_all = "snake_case")]
pub(crate) fn remove_courier_item(
    app: AppHandle,
    state: State<'_, CourierRuntimeState>,
    work_item_id: String,
) -> Result<CourierDisplaySnapshot, DesktopApplicationError> {
    let mut store = lock_store(&state, "remove")?;
    let orchestrator = store.orchestrator.as_mut().ok_or_else(|| {
        adapter_error(
            "COURIER_COLLECTION_EMPTY",
            "remove",
            "No courier collection is active.",
        )
    })?;
    orchestrator
        .remove_item(&work_item_id)
        .map_err(collection_error)?;
    drop(store);
    emit_current_snapshot(&app, &state)
}

#[tauri::command(rename_all = "snake_case")]
pub(crate) fn set_courier_destination(
    app: AppHandle,
    state: State<'_, CourierRuntimeState>,
    destination_parent: PathBuf,
) -> Result<CourierDisplaySnapshot, DesktopApplicationError> {
    validate_existing_directory(&destination_parent, "destination")?;
    let mut preferences = load_for_app(&app).map_err(preferences_error)?;
    preferences.output_parent = destination_parent.clone();
    save_for_app(&app, &preferences).map_err(preferences_error)?;
    let mut store = lock_store(&state, "destination")?;
    if let Some(orchestrator) = store.orchestrator.as_mut() {
        orchestrator
            .set_destination_parent(destination_parent)
            .map_err(collection_error)?;
    }
    drop(store);
    emit_current_snapshot(&app, &state)
}

#[tauri::command]
pub(crate) async fn start_courier_processing(
    app: AppHandle,
    state: State<'_, CourierRuntimeState>,
) -> Result<CourierDisplaySnapshot, DesktopApplicationError> {
    ensure_output_parent(&app)?;
    let first_wave = {
        let mut store = lock_store(&state, "start")?;
        let orchestrator = store.orchestrator.as_mut().ok_or_else(|| {
            adapter_error("COURIER_COLLECTION_EMPTY", "start", "No ALS is queued.")
        })?;
        orchestrator.start_processing().map_err(collection_error)?
    };
    emit_current_snapshot(&app, &state)?;
    let worker_app = app.clone();
    tauri::async_runtime::spawn_blocking(move || process_waves(&worker_app, first_wave))
        .await
        .map_err(|_| {
            adapter_error(
                "COURIER_BACKGROUND_FAILED",
                "process",
                "Courier processing stopped unexpectedly.",
            )
        })??;
    display_snapshot(&app, &state)
}

#[tauri::command]
pub(crate) fn reset_courier_collection(
    app: AppHandle,
    state: State<'_, CourierRuntimeState>,
) -> Result<CourierDisplaySnapshot, DesktopApplicationError> {
    let mut store = lock_store(&state, "reset")?;
    if store
        .orchestrator
        .as_ref()
        .is_some_and(|value| value.snapshot().work_phase == "processing")
    {
        return Err(adapter_error(
            "COURIER_COLLECTION_BUSY",
            "reset",
            "The courier is still processing.",
        ));
    }
    *store = CourierRuntimeStore::default();
    drop(store);
    crate::universal_payload_drag_commands::reset_universal_payload_drag(&app);
    emit_current_snapshot(&app, &state)
}

pub(crate) fn intake_native_paths(
    app: &AppHandle,
    paths: &[PathBuf],
) -> Result<CourierIntakeResult, DesktopApplicationError> {
    let state = app.state::<CourierRuntimeState>();
    intake_paths(app, &state, paths)
}

pub(crate) fn ready_delivery_snapshot(
    state: &CourierRuntimeState,
) -> Result<rescue_application::DeliveryCollectionSnapshot, DesktopApplicationError> {
    let store = state.store.lock().map_err(|_| state_error("delivery"))?;
    let orchestrator = store.orchestrator.as_ref().ok_or_else(|| {
        adapter_error(
            "COURIER_COLLECTION_EMPTY",
            "delivery",
            "No courier collection is active.",
        )
    })?;
    orchestrator.delivery_snapshot().map_err(collection_error)
}

pub(crate) fn delivered_item_ids(
    state: &CourierRuntimeState,
) -> Result<Vec<String>, DesktopApplicationError> {
    let store = state.store.lock().map_err(|_| state_error("delivery"))?;
    Ok(store.delivered_item_ids.iter().cloned().collect())
}

pub(crate) fn record_delivered_items(
    state: &CourierRuntimeState,
    item_ids: impl IntoIterator<Item = String>,
) -> Result<(), DesktopApplicationError> {
    let mut store = state.store.lock().map_err(|_| state_error("delivery"))?;
    store.delivered_item_ids.extend(item_ids);
    Ok(())
}

pub(crate) fn begin_courier_handoff(
    app: &AppHandle,
    collection_id: &str,
    revision: u64,
    channel: &str,
) -> Result<(), DesktopApplicationError> {
    let state = app.state::<CourierRuntimeState>();
    let mut store = lock_store(&state, "begin_handoff")?;
    let orchestrator = active_orchestrator(&mut store, "begin_handoff")?;
    orchestrator
        .begin_handoff(collection_id, revision, channel)
        .map_err(collection_error)?;
    drop(store);
    emit_runtime_snapshot(app)
}

pub(crate) fn complete_courier_handoff(
    app: &AppHandle,
    collection_id: &str,
    revision: u64,
    channel: &str,
) -> Result<(), DesktopApplicationError> {
    let state = app.state::<CourierRuntimeState>();
    let mut store = lock_store(&state, "complete_handoff")?;
    active_orchestrator(&mut store, "complete_handoff")?
        .complete_handoff(collection_id, revision, channel)
        .map_err(collection_error)?;
    rotate_pending_next_order(&mut store)?;
    drop(store);
    crate::universal_payload_drag_commands::reset_universal_payload_drag(app);
    emit_runtime_snapshot(app)
}

#[cfg(target_os = "macos")]
pub(crate) fn cancel_courier_handoff(
    app: &AppHandle,
    collection_id: &str,
    revision: u64,
    channel: &str,
) -> Result<(), DesktopApplicationError> {
    finish_retryable_handoff(app, collection_id, revision, channel, false)
}

pub(crate) fn retry_courier_handoff(
    app: &AppHandle,
    collection_id: &str,
    revision: u64,
    channel: &str,
) -> Result<(), DesktopApplicationError> {
    finish_retryable_handoff(app, collection_id, revision, channel, true)
}

#[tauri::command]
pub(crate) fn reopen_courier_handoff(
    app: AppHandle,
    state: State<'_, CourierRuntimeState>,
) -> Result<CourierDisplaySnapshot, DesktopApplicationError> {
    let snapshot = {
        let mut store = lock_store(&state, "reopen_handoff")?;
        let snapshot = {
            let orchestrator = active_orchestrator(&mut store, "reopen_handoff")?;
            orchestrator.reopen_handoff().map_err(collection_error)?;
            orchestrator.delivery_snapshot().map_err(collection_error)?
        };
        store.delivered_item_ids.clear();
        snapshot
    };
    crate::universal_payload_drag_commands::reset_universal_payload_drag(&app);
    if !app
        .state::<crate::transfer_payload::TransferPayloadState>()
        .register_collection(&snapshot)
    {
        return Err(adapter_error(
            "PAYLOAD_CANDIDATE_INVALID",
            "reopen_handoff",
            "The completed collection could not be made available again.",
        ));
    }
    emit_current_snapshot(&app, &state)
}

fn intake_paths(
    app: &AppHandle,
    state: &CourierRuntimeState,
    paths: &[PathBuf],
) -> Result<CourierIntakeResult, DesktopApplicationError> {
    let preferences = load_for_app(app).map_err(preferences_error)?;
    let mut candidates = Vec::new();
    let mut error_codes = Vec::new();
    for path in paths {
        match selection_from_path(path) {
            Ok(candidate) => candidates.push(candidate),
            Err(failure) => error_codes.push(failure.error_code),
        }
    }
    let mut store = state.store.lock().map_err(|_| state_error("intake"))?;
    if !candidates.is_empty() {
        start_fresh_order_if_completed(&mut store, &preferences.output_parent)?;
    }
    if store.orchestrator.is_none() && !candidates.is_empty() {
        store.orchestrator = Some(new_orchestrator(&preferences.output_parent)?);
    }
    let mut accepted_count = 0;
    let handoff_in_progress = store
        .orchestrator
        .as_ref()
        .is_some_and(|orchestrator| orchestrator.snapshot().handoff.status == "in_progress");
    for (selection, display) in candidates {
        if handoff_in_progress {
            store.pending_next_order.push((selection, display));
            accepted_count += 1;
            continue;
        }
        let result = active_orchestrator(&mut store, "intake")?
            .add_selection(selection, display)
            .map_err(collection_error);
        if let Err(failure) = result {
            error_codes.push(failure.error_code);
        } else {
            accepted_count += 1;
        }
    }
    let snapshot = display_from_store(&store, &preferences);
    drop(store);
    if accepted_count > 0 && !handoff_in_progress {
        crate::universal_payload_drag_commands::reset_universal_payload_drag(app);
    }
    let _ = app.emit_to(
        crate::quick_copy_entry::QUICK_WINDOW_LABEL,
        COURIER_SNAPSHOT_EVENT,
        &snapshot,
    );
    Ok(CourierIntakeResult {
        schema_version: "0.2".to_string(),
        accepted_count,
        refused_count: paths.len().saturating_sub(accepted_count),
        error_codes,
        snapshot,
    })
}

fn finish_retryable_handoff(
    app: &AppHandle,
    collection_id: &str,
    revision: u64,
    channel: &str,
    retryable: bool,
) -> Result<(), DesktopApplicationError> {
    let state = app.state::<CourierRuntimeState>();
    let mut store = lock_store(&state, "finish_handoff")?;
    let orchestrator = active_orchestrator(&mut store, "finish_handoff")?;
    if retryable {
        orchestrator
            .mark_handoff_retryable(collection_id, revision, channel)
            .map_err(collection_error)?;
    } else {
        orchestrator
            .cancel_handoff(collection_id, revision, channel)
            .map_err(collection_error)?;
    }
    merge_pending_into_active(&mut store)?;
    let delivery = store
        .orchestrator
        .as_ref()
        .filter(|value| value.snapshot().work_phase == "ready")
        .and_then(|value| value.delivery_snapshot().ok());
    drop(store);
    crate::universal_payload_drag_commands::reset_universal_payload_drag(app);
    if let Some(snapshot) = delivery {
        let _ = app
            .state::<crate::transfer_payload::TransferPayloadState>()
            .register_collection(&snapshot);
    }
    emit_runtime_snapshot(app)
}

fn new_orchestrator(
    destination_parent: &Path,
) -> Result<CourierCollectionOrchestrator, DesktopApplicationError> {
    CourierCollectionOrchestrator::new(next_collection_id(), destination_parent.to_path_buf())
        .map_err(collection_error)
}

fn start_fresh_order_if_completed(
    store: &mut CourierRuntimeStore,
    destination_parent: &Path,
) -> Result<(), DesktopApplicationError> {
    if store
        .orchestrator
        .as_ref()
        .is_some_and(CourierCollectionOrchestrator::handoff_completed)
    {
        store.orchestrator = Some(new_orchestrator(destination_parent)?);
        store.delivered_item_ids.clear();
        store.pending_next_order.clear();
    }
    Ok(())
}

fn active_orchestrator<'a>(
    store: &'a mut CourierRuntimeStore,
    stage: &str,
) -> Result<&'a mut CourierCollectionOrchestrator, DesktopApplicationError> {
    store.orchestrator.as_mut().ok_or_else(|| {
        adapter_error(
            "COURIER_COLLECTION_EMPTY",
            stage,
            "No courier collection is active.",
        )
    })
}

fn rotate_pending_next_order(
    store: &mut CourierRuntimeStore,
) -> Result<(), DesktopApplicationError> {
    if store.pending_next_order.is_empty() {
        return Ok(());
    }
    let destination = store
        .orchestrator
        .as_ref()
        .ok_or_else(|| state_error("rotate_order"))?
        .snapshot()
        .destination_parent;
    let mut next = new_orchestrator(&destination)?;
    for (selection, display) in &store.pending_next_order {
        next.add_selection(selection.clone(), display.clone())
            .map_err(collection_error)?;
    }
    store.orchestrator = Some(next);
    store.delivered_item_ids.clear();
    store.pending_next_order.clear();
    Ok(())
}

fn merge_pending_into_active(
    store: &mut CourierRuntimeStore,
) -> Result<(), DesktopApplicationError> {
    let pending = store.pending_next_order.clone();
    for (selection, display) in pending {
        active_orchestrator(store, "merge_pending")?
            .add_selection(selection, display)
            .map_err(collection_error)?;
    }
    store.pending_next_order.clear();
    Ok(())
}

fn process_waves(
    app: &AppHandle,
    mut wave: CourierBatchWave,
) -> Result<(), DesktopApplicationError> {
    loop {
        let preview = rescue_application::prepare_batch_copy(&BatchPrepareCopyRequest {
            request_id: wave.wave_id.clone(),
            selections: wave.selections.clone(),
            destination_parent: wave.destination_parent.clone(),
            experimental_compatibility_consent: false,
        });
        let result = rescue_application::execute_batch_copy(&BatchExecuteCopyRequest {
            request_id: wave.wave_id.clone(),
            preview,
            write_consent: true,
        });
        let next = merge_and_next(app, &wave, &result)?;
        emit_runtime_snapshot(app)?;
        match next {
            Some(next_wave) => wave = next_wave,
            None => {
                publish_delivery_candidate(app)?;
                return Ok(());
            }
        }
    }
}

fn merge_and_next(
    app: &AppHandle,
    wave: &CourierBatchWave,
    result: &rescue_application::BatchCopyResult,
) -> Result<Option<CourierBatchWave>, DesktopApplicationError> {
    let state = app.state::<CourierRuntimeState>();
    let mut store = state.store.lock().map_err(|_| state_error("merge"))?;
    let orchestrator = store
        .orchestrator
        .as_mut()
        .ok_or_else(|| state_error("merge"))?;
    let snapshot = orchestrator
        .merge_wave_result(&wave.wave_id, result)
        .map_err(collection_error)?;
    if snapshot.work_phase == "processing" {
        orchestrator.next_wave().map_err(collection_error)
    } else {
        Ok(None)
    }
}

fn publish_delivery_candidate(app: &AppHandle) -> Result<(), DesktopApplicationError> {
    let runtime = app.state::<CourierRuntimeState>();
    let snapshot = ready_delivery_snapshot(&runtime)?;
    if !app
        .state::<crate::transfer_payload::TransferPayloadState>()
        .register_collection(&snapshot)
    {
        return Err(adapter_error(
            "PAYLOAD_CANDIDATE_INVALID",
            "publish",
            "The collection payload is unavailable.",
        ));
    }
    Ok(())
}

fn selection_from_path(path: &Path) -> Result<(ProjectSelection, String), DesktopApplicationError> {
    let display = path
        .file_name()
        .and_then(|name| name.to_str())
        .filter(|name| !name.trim().is_empty())
        .ok_or_else(|| {
            adapter_error(
                "COURIER_SELECTION_INVALID",
                "intake",
                "The ALS name is invalid.",
            )
        })?;
    let sequence = SELECTION_SEQUENCE.fetch_add(1, Ordering::Relaxed);
    Ok((
        ProjectSelection {
            selection_id: format!("courier-selection-{}-{sequence}", std::process::id()),
            selection_source: "courier_intake".to_string(),
            live_set_id: None,
            native_als_path: path.to_path_buf(),
            catalog_revision: None,
            observation_fingerprint: None,
            freshness_status: "manual_unverified".to_string(),
        },
        display.to_string(),
    ))
}

fn display_snapshot(
    app: &AppHandle,
    state: &CourierRuntimeState,
) -> Result<CourierDisplaySnapshot, DesktopApplicationError> {
    let preferences = load_for_app(app).map_err(preferences_error)?;
    let store = state.store.lock().map_err(|_| state_error("snapshot"))?;
    Ok(display_from_store(&store, &preferences))
}

fn display_from_store(
    store: &CourierRuntimeStore,
    preferences: &CourierPreferences,
) -> CourierDisplaySnapshot {
    let Some(snapshot) = store
        .orchestrator
        .as_ref()
        .map(CourierCollectionOrchestrator::snapshot)
    else {
        return empty_display(preferences);
    };
    display_from_collection(&snapshot, preferences)
}

fn display_from_collection(
    snapshot: &CourierCollectionSnapshot,
    preferences: &CourierPreferences,
) -> CourierDisplaySnapshot {
    CourierDisplaySnapshot {
        schema_version: "0.2".to_string(),
        collection_id: Some(snapshot.collection_id.clone()),
        revision: snapshot.revision,
        work_phase: snapshot.work_phase.clone(),
        handoff_status: snapshot.handoff.status.clone(),
        handoff_channel: snapshot.handoff.channel.clone(),
        destination_display_path: snapshot.destination_parent.to_string_lossy().into_owned(),
        items: snapshot
            .queue
            .items
            .iter()
            .filter(|item| item.status != "removed")
            .map(|item| CourierDisplayItem {
                work_item_id: item.work_item_id.clone(),
                source_display_name: item.source_display_name.clone(),
                status: item.status.clone(),
            })
            .collect(),
        pending_item_count: snapshot.queue.pending_item_count,
        completed_item_count: snapshot.completed_item_count,
        incomplete_item_count: snapshot.incomplete_item_count,
        failed_item_count: snapshot.failed_item_count,
        omitted_asset_count: snapshot.omitted_asset_count,
        delivery_available: snapshot.work_phase == "ready"
            && !snapshot.items.is_empty()
            && matches!(
                snapshot.handoff.status.as_str(),
                "not_started" | "retryable_issue"
            ),
        local_delivery_configured: preferences.local_delivery_root.is_some(),
    }
}

fn empty_display(preferences: &CourierPreferences) -> CourierDisplaySnapshot {
    CourierDisplaySnapshot {
        schema_version: "0.2".to_string(),
        collection_id: None,
        revision: 0,
        work_phase: "empty".to_string(),
        handoff_status: "not_started".to_string(),
        handoff_channel: None,
        destination_display_path: preferences.output_parent.to_string_lossy().into_owned(),
        items: Vec::new(),
        pending_item_count: 0,
        completed_item_count: 0,
        incomplete_item_count: 0,
        failed_item_count: 0,
        omitted_asset_count: 0,
        delivery_available: false,
        local_delivery_configured: preferences.local_delivery_root.is_some(),
    }
}

fn emit_current_snapshot(
    app: &AppHandle,
    state: &CourierRuntimeState,
) -> Result<CourierDisplaySnapshot, DesktopApplicationError> {
    let snapshot = display_snapshot(app, state)?;
    let _ = app.emit_to(
        crate::quick_copy_entry::QUICK_WINDOW_LABEL,
        COURIER_SNAPSHOT_EVENT,
        &snapshot,
    );
    Ok(snapshot)
}

fn emit_runtime_snapshot(app: &AppHandle) -> Result<(), DesktopApplicationError> {
    let state = app.state::<CourierRuntimeState>();
    emit_current_snapshot(app, &state).map(|_| ())
}

fn ensure_output_parent(app: &AppHandle) -> Result<(), DesktopApplicationError> {
    let preferences = load_for_app(app).map_err(preferences_error)?;
    if preferences.output_parent.exists() {
        validate_existing_directory(&preferences.output_parent, "destination")
    } else {
        std::fs::create_dir_all(&preferences.output_parent).map_err(|_| {
            adapter_error(
                "COURIER_DESTINATION_UNAVAILABLE",
                "destination",
                "The destination folder could not be created.",
            )
        })
    }
}

fn validate_existing_directory(path: &Path, stage: &str) -> Result<(), DesktopApplicationError> {
    let metadata = std::fs::symlink_metadata(path).map_err(|_| {
        adapter_error(
            "COURIER_DESTINATION_UNAVAILABLE",
            stage,
            "The destination folder is unavailable.",
        )
    })?;
    if !path.is_absolute() || metadata.file_type().is_symlink() || !metadata.is_dir() {
        return Err(adapter_error(
            "COURIER_DESTINATION_UNAVAILABLE",
            stage,
            "The destination folder is invalid.",
        ));
    }
    Ok(())
}

fn next_collection_id() -> String {
    let sequence = COLLECTION_SEQUENCE.fetch_add(1, Ordering::Relaxed);
    format!("courier-collection-{}-{sequence}", std::process::id())
}

fn lock_store<'a>(
    state: &'a CourierRuntimeState,
    stage: &str,
) -> Result<std::sync::MutexGuard<'a, CourierRuntimeStore>, DesktopApplicationError> {
    state.store.lock().map_err(|_| state_error(stage))
}

fn preferences_error(
    error: crate::courier_preferences::CourierPreferencesError,
) -> DesktopApplicationError {
    adapter_error(&error.error_code, &error.stage, &error.message)
}

fn collection_error(error: rescue_application::CourierCollectionError) -> DesktopApplicationError {
    adapter_error(&error.error_code, &error.stage, &error.message)
}

fn state_error(stage: &str) -> DesktopApplicationError {
    adapter_error(
        "COURIER_STATE_UNAVAILABLE",
        stage,
        "Courier state is unavailable.",
    )
}

fn adapter_error(code: &str, stage: &str, message: &str) -> DesktopApplicationError {
    DesktopApplicationError {
        error_code: code.to_string(),
        stage: stage.to_string(),
        message: message.to_string(),
    }
}

#[cfg(test)]
#[path = "courier_commands_tests.rs"]
mod tests;
