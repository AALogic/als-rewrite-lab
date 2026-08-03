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
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            get_application_profile,
            finalize_compatibility_report,
            analyze_project,
            suggest_target_project_root,
            prepare_copy,
            execute_copy
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
