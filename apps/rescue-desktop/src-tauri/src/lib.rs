mod batch_commands;
mod catalog_commands;
mod catalog_roots;
mod collection_delivery_commands;
mod courier_commands;
mod courier_preferences;
mod external_folder_handoff;
mod external_folder_handoff_commands;
mod macos_folder_drag;
mod quick_als_intake;
mod quick_copy_entry;
mod quick_startup_buffer;
mod quick_window_layout;
mod quick_window_policy;
mod transfer_payload;
mod universal_payload_drag;
mod universal_payload_drag_commands;

#[tauri::command]
async fn analyze_project(
    request: rescue_application::DesktopAnalyzeRequest,
) -> Result<rescue_application::DesktopAnalyzeResult, rescue_application::DesktopApplicationError> {
    tauri::async_runtime::spawn_blocking(move || rescue_application::analyze_project(&request))
        .await
        .map_err(|failure| background_task_error("analysis", &failure.to_string()))
}

#[tauri::command]
fn get_application_profile() -> rescue_application::DesktopApplicationProfile {
    rescue_application::application_profile()
}

#[tauri::command]
fn finalize_compatibility_report(
    request: rescue_application::CompatibilityTestReportRequest,
) -> Result<rescue_application::CompatibilityTestReport, rescue_application::DesktopApplicationError>
{
    rescue_application::finalize_compatibility_test_report(&request)
}

#[tauri::command(rename_all = "snake_case")]
fn suggest_target_project_root(
    source_als_path: std::path::PathBuf,
    destination_parent: std::path::PathBuf,
) -> Result<std::path::PathBuf, rescue_application::DesktopApplicationError> {
    rescue_application::default_target_project_root(&source_als_path, &destination_parent)
}

#[tauri::command]
async fn prepare_copy(
    request: rescue_application::DesktopPrepareCopyRequest,
) -> Result<rescue_application::DesktopCopyPreview, rescue_application::DesktopApplicationError> {
    tauri::async_runtime::spawn_blocking(move || rescue_application::prepare_copy(&request))
        .await
        .map_err(|failure| background_task_error("copy_preview", &failure.to_string()))
}

#[tauri::command]
async fn execute_copy(
    request: rescue_application::DesktopExecuteCopyRequest,
) -> Result<rescue_application::DesktopCopyResult, rescue_application::DesktopApplicationError> {
    tauri::async_runtime::spawn_blocking(move || rescue_application::execute_copy(&request))
        .await
        .map_err(|failure| background_task_error("copy_execution", &failure.to_string()))
}

fn background_task_error(
    stage: &str,
    message: &str,
) -> rescue_application::DesktopApplicationError {
    rescue_application::DesktopApplicationError {
        error_code: "DESKTOP_BACKGROUND_TASK_FAILED".to_string(),
        stage: stage.to_string(),
        message: format!("Background task failed: {message}"),
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let app = tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, args, _cwd| {
            quick_copy_entry::route_secondary_args(app, &args);
        }))
        .manage(batch_commands::BatchCancellationState::default())
        .manage(courier_commands::CourierRuntimeState::default())
        .manage(transfer_payload::TransferPayloadState::default())
        .manage(quick_copy_entry::QuickCopyEntryState::default())
        .manage(quick_startup_buffer::StartupOpenUrlBuffer::default())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            quick_copy_entry::finish_startup(app.handle());
            quick_copy_entry::schedule_normal_main_window(app.handle());
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            batch_commands::cancel_batch_copy,
            batch_commands::execute_batch_copy,
            batch_commands::prepare_batch_copy,
            catalog_commands::list_project_catalog,
            catalog_commands::refresh_project_catalog,
            catalog_commands::resolve_project_selection,
            collection_delivery_commands::deliver_courier_collection,
            collection_delivery_commands::get_courier_preferences,
            collection_delivery_commands::set_courier_delivery_root,
            courier_commands::add_courier_sources,
            courier_commands::get_courier_snapshot,
            courier_commands::remove_courier_item,
            courier_commands::reopen_courier_handoff,
            courier_commands::reset_courier_collection,
            courier_commands::set_courier_destination,
            courier_commands::start_courier_processing,
            get_application_profile,
            external_folder_handoff_commands::open_external_provider,
            quick_copy_entry::get_quick_copy_launch_context,
            quick_window_layout::set_quick_window_footprint,
            finalize_compatibility_report,
            analyze_project,
            suggest_target_project_root,
            prepare_copy,
            execute_copy,
            universal_payload_drag_commands::cancel_universal_payload_drag,
            universal_payload_drag_commands::prepare_universal_payload_drag,
            universal_payload_drag_commands::reset_universal_payload_attempt
        ])
        .build(tauri::generate_context!())
        .expect("error while building tauri application");
    app.run(|app_handle, event| match event {
        #[cfg(target_os = "macos")]
        tauri::RunEvent::Opened { urls } => {
            quick_copy_entry::route_opened_urls(app_handle, &urls, "macos_open_with");
        }
        #[cfg(target_os = "macos")]
        tauri::RunEvent::Reopen {
            has_visible_windows: false,
            ..
        } => quick_copy_entry::show_main_window(app_handle),
        tauri::RunEvent::WindowEvent {
            label,
            event: tauri::WindowEvent::DragDrop(tauri::DragDropEvent::Drop { paths, .. }),
            ..
        } if label == quick_copy_entry::QUICK_WINDOW_LABEL => {
            let _ = quick_als_intake::route_finder_drop(app_handle, &paths);
        }
        tauri::RunEvent::WindowEvent { label, event, .. }
            if label == quick_copy_entry::QUICK_WINDOW_LABEL
                && matches!(
                    event,
                    tauri::WindowEvent::CloseRequested { .. } | tauri::WindowEvent::Destroyed
                ) =>
        {
            universal_payload_drag_commands::reset_universal_payload_drag(app_handle);
        }
        _ => {}
    });
}
