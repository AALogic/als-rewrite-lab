use crate::macos_folder_drag::FolderDragSurfaceRect;
use crate::transfer_payload::{TransferPayloadError, TransferPayloadState};
use crate::universal_payload_drag::{
    finished, prepared, validate_request, UniversalPayloadDragError, UniversalPayloadDragFinished,
    UniversalPayloadDragPrepared, UniversalPayloadDragRequest,
};
use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager, State};

pub(crate) const DRAG_FINISHED_EVENT: &str = "universal-payload-drag-finished";
pub(crate) const DRAG_STARTED_EVENT: &str = "universal-payload-drag-started";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum NativeHandoffDisposition {
    Complete,
    Cancel,
    Retry,
}

#[derive(Clone, Serialize)]
struct UniversalPayloadDragStarted<'a> {
    schema_version: &'static str,
    attempt_id: &'a str,
    state: &'static str,
}

#[tauri::command(rename_all = "snake_case")]
pub(crate) async fn prepare_universal_payload_drag(
    app: AppHandle,
    state: State<'_, TransferPayloadState>,
    request: UniversalPayloadDragRequest,
    surface_rect: FolderDragSurfaceRect,
) -> Result<UniversalPayloadDragPrepared, UniversalPayloadDragError> {
    validate_request(&request)?;
    let attempt = state.prepare_attempt(&request.collection_id, request.collection_revision)?;
    let response = prepared(attempt.attempt_id);
    if let Err(failure) = crate::macos_folder_drag::arm_folder_drag_surface(
        &app,
        response.attempt_id.clone(),
        surface_rect,
    )
    .await
    {
        let _ = state.cancel_armed(&response.attempt_id);
        return Err(failure);
    }
    Ok(response)
}

#[tauri::command(rename_all = "snake_case")]
pub(crate) fn cancel_universal_payload_drag(
    app: AppHandle,
    state: State<'_, TransferPayloadState>,
    attempt_id: String,
) -> Result<UniversalPayloadDragFinished, UniversalPayloadDragError> {
    state.cancel_armed(&attempt_id)?;
    crate::macos_folder_drag::remove_folder_drag_surface(&app, &attempt_id);
    let result = finished(&attempt_id, "cancelled", None);
    emit_finished(&app, &result);
    Ok(result)
}

pub(crate) fn reset_universal_payload_drag(app: &AppHandle) {
    let state = app.state::<TransferPayloadState>();
    let _ = state.reset();
    crate::macos_folder_drag::remove_all_folder_drag_surfaces(app);
}

#[tauri::command]
pub(crate) fn reset_universal_payload_attempt(
    app: AppHandle,
    state: State<'_, TransferPayloadState>,
) -> Result<(), TransferPayloadError> {
    if let Some(attempt_id) = state.clear_attempt()? {
        crate::macos_folder_drag::remove_folder_drag_surface(&app, &attempt_id);
    }
    Ok(())
}

pub(crate) fn emit_native_drag_started(app: &AppHandle, attempt_id: &str) {
    let payload = UniversalPayloadDragStarted {
        schema_version: "0.1",
        attempt_id,
        state: "dragging",
    };
    let _ = app.emit_to(
        crate::quick_copy_entry::QUICK_WINDOW_LABEL,
        DRAG_STARTED_EVENT,
        payload,
    );
}

pub(crate) fn finish_native_drag(
    app: &AppHandle,
    attempt_id: &str,
    outcome: &str,
    error_code: Option<&str>,
) {
    let state = app.state::<TransferPayloadState>();
    let attempt = match state.finish_attempt(attempt_id) {
        Ok(value) => value,
        Err(failure) => {
            emit_finished(
                app,
                &finished(attempt_id, "failed", Some(&failure.error_code)),
            );
            return;
        }
    };
    let lifecycle_result = match native_handoff_disposition(outcome) {
        NativeHandoffDisposition::Complete => crate::courier_commands::complete_courier_handoff(
            app,
            &attempt.collection_id,
            attempt.collection_revision,
            crate::courier_commands::NATIVE_HANDOFF_CHANNEL,
        ),
        NativeHandoffDisposition::Cancel => crate::courier_commands::cancel_courier_handoff(
            app,
            &attempt.collection_id,
            attempt.collection_revision,
            crate::courier_commands::NATIVE_HANDOFF_CHANNEL,
        ),
        NativeHandoffDisposition::Retry => crate::courier_commands::retry_courier_handoff(
            app,
            &attempt.collection_id,
            attempt.collection_revision,
            crate::courier_commands::NATIVE_HANDOFF_CHANNEL,
        ),
    };
    if let Err(failure) = lifecycle_result {
        emit_finished(
            app,
            &finished(attempt_id, "failed", Some(&failure.error_code)),
        );
        return;
    }
    emit_finished(app, &finished(attempt_id, outcome, error_code));
}

fn native_handoff_disposition(outcome: &str) -> NativeHandoffDisposition {
    match outcome {
        "dropped" => NativeHandoffDisposition::Complete,
        "cancelled" => NativeHandoffDisposition::Cancel,
        _ => NativeHandoffDisposition::Retry,
    }
}

fn emit_finished(app: &AppHandle, result: &UniversalPayloadDragFinished) {
    let _ = app.emit_to(
        crate::quick_copy_entry::QUICK_WINDOW_LABEL,
        DRAG_FINISHED_EVENT,
        result,
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn native_drag_outcomes_drive_handoff_lifecycle() {
        assert_eq!(
            native_handoff_disposition("dropped"),
            NativeHandoffDisposition::Complete
        );
        assert_eq!(
            native_handoff_disposition("cancelled"),
            NativeHandoffDisposition::Cancel
        );
        assert_eq!(
            native_handoff_disposition("failed"),
            NativeHandoffDisposition::Retry
        );
    }

    #[test]
    fn reset_releases_all_native_surfaces_without_attempt_identity() {
        let source = include_str!("universal_payload_drag_commands.rs");
        let reset = source
            .split("pub(crate) fn reset_universal_payload_drag")
            .nth(1)
            .and_then(|value| value.split("#[tauri::command]").next())
            .unwrap_or_default();
        assert!(reset.contains("let _ = state.reset();"));
        assert!(reset.contains("remove_all_folder_drag_surfaces(app);"));
    }
}
