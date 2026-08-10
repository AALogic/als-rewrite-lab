use crate::courier_commands::CourierRuntimeState;
use crate::courier_preferences::{load_for_app, save_for_app, CourierPreferences};
use rescue_application::{
    CollectionDeliveryRequest, CollectionDeliveryResult, DesktopApplicationError,
};
use std::path::{Path, PathBuf};
use tauri::{AppHandle, State};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum LocalDeliveryHandoffDisposition {
    Complete,
    Retry,
}

#[tauri::command]
pub(crate) fn get_courier_preferences(
    app: AppHandle,
) -> Result<CourierPreferences, DesktopApplicationError> {
    load_for_app(&app).map_err(preferences_error)
}

#[tauri::command(rename_all = "snake_case")]
pub(crate) fn set_courier_delivery_root(
    app: AppHandle,
    local_delivery_root: PathBuf,
) -> Result<CourierPreferences, DesktopApplicationError> {
    validate_real_directory(&local_delivery_root)?;
    let mut preferences = load_for_app(&app).map_err(preferences_error)?;
    preferences.local_delivery_root = Some(local_delivery_root);
    save_for_app(&app, &preferences).map_err(preferences_error)?;
    Ok(preferences)
}

#[tauri::command]
pub(crate) async fn deliver_courier_collection(
    app: AppHandle,
    state: State<'_, CourierRuntimeState>,
) -> Result<CollectionDeliveryResult, DesktopApplicationError> {
    let preferences = load_for_app(&app).map_err(preferences_error)?;
    let destination_root = preferences.local_delivery_root.ok_or_else(|| {
        adapter_error(
            "DELIVERY_SETTINGS_INVALID",
            "delivery",
            "A local delivery folder has not been configured.",
        )
    })?;
    validate_real_directory(&destination_root)?;
    let snapshot = crate::courier_commands::ready_delivery_snapshot(&state)?;
    let collection_id = snapshot.collection_id.clone();
    let collection_revision = snapshot.revision;
    let previously_delivered_item_ids = crate::courier_commands::delivered_item_ids(&state)?;
    crate::courier_commands::begin_courier_handoff(
        &app,
        &collection_id,
        collection_revision,
        crate::courier_commands::LOCAL_HANDOFF_CHANNEL,
    )?;
    crate::universal_payload_drag_commands::reset_universal_payload_drag(&app);
    let request = CollectionDeliveryRequest {
        request_id: format!(
            "courier-delivery-{}-{}",
            std::process::id(),
            snapshot.revision
        ),
        snapshot,
        destination_root,
        previously_delivered_item_ids,
        write_consent: true,
    };
    let result = match tauri::async_runtime::spawn_blocking(move || {
        let plan = rescue_application::plan_collection_delivery(&request);
        rescue_application::execute_collection_delivery(&request, &plan)
    })
    .await
    {
        Ok(result) => result,
        Err(_) => {
            let _ = crate::courier_commands::retry_courier_handoff(
                &app,
                &collection_id,
                collection_revision,
                crate::courier_commands::LOCAL_HANDOFF_CHANNEL,
            );
            return Err(adapter_error(
                "DELIVERY_BACKGROUND_FAILED",
                "delivery",
                "Local collection delivery stopped unexpectedly.",
            ));
        }
    };
    let completed = result
        .items
        .iter()
        .filter(|item| item.status == "completed")
        .map(|item| item.item_id.clone());
    if let Err(failure) = crate::courier_commands::record_delivered_items(&state, completed) {
        let _ = crate::courier_commands::retry_courier_handoff(
            &app,
            &collection_id,
            collection_revision,
            crate::courier_commands::LOCAL_HANDOFF_CHANNEL,
        );
        return Err(failure);
    }
    match local_delivery_handoff_disposition(&result.run_status) {
        LocalDeliveryHandoffDisposition::Complete => {
            crate::courier_commands::complete_courier_handoff(
                &app,
                &collection_id,
                collection_revision,
                crate::courier_commands::LOCAL_HANDOFF_CHANNEL,
            )?;
        }
        LocalDeliveryHandoffDisposition::Retry => {
            crate::courier_commands::retry_courier_handoff(
                &app,
                &collection_id,
                collection_revision,
                crate::courier_commands::LOCAL_HANDOFF_CHANNEL,
            )?;
        }
    }
    Ok(result)
}

fn local_delivery_handoff_disposition(run_status: &str) -> LocalDeliveryHandoffDisposition {
    if run_status == "completed" {
        LocalDeliveryHandoffDisposition::Complete
    } else {
        LocalDeliveryHandoffDisposition::Retry
    }
}

fn validate_real_directory(path: &Path) -> Result<(), DesktopApplicationError> {
    let metadata = std::fs::symlink_metadata(path).map_err(|_| {
        adapter_error(
            "DELIVERY_DESTINATION_INVALID",
            "delivery",
            "The local delivery folder is unavailable.",
        )
    })?;
    if !path.is_absolute() || metadata.file_type().is_symlink() || !metadata.is_dir() {
        return Err(adapter_error(
            "DELIVERY_DESTINATION_INVALID",
            "delivery",
            "The local delivery folder is invalid.",
        ));
    }
    Ok(())
}

fn preferences_error(
    error: crate::courier_preferences::CourierPreferencesError,
) -> DesktopApplicationError {
    adapter_error(&error.error_code, &error.stage, &error.message)
}

fn adapter_error(code: &str, stage: &str, message: &str) -> DesktopApplicationError {
    DesktopApplicationError {
        error_code: code.to_string(),
        stage: stage.to_string(),
        message: message.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn local_delivery_outcomes_drive_handoff_lifecycle() {
        assert_eq!(
            local_delivery_handoff_disposition("completed"),
            LocalDeliveryHandoffDisposition::Complete
        );
        for status in ["completed_with_issues", "failed"] {
            assert_eq!(
                local_delivery_handoff_disposition(status),
                LocalDeliveryHandoffDisposition::Retry
            );
        }
    }
}
