use crate::catalog_roots::platform_project_scan_scope;
use rescue_application::{
    DesktopApplicationError, ProjectCatalogListRequest, ProjectCatalogListResult,
    ProjectCatalogRefreshRequest, ProjectCatalogRefreshResult, ProjectSelectionRequest,
    ProjectSelectionResult,
};
use serde::Deserialize;
use std::path::PathBuf;
use tauri::Manager;

const PROJECT_SCAN_MAX_ENTRIES: usize = 8_000_000;

#[derive(Debug, Deserialize)]
pub(crate) struct DesktopProjectCatalogRefreshRequest {
    request_id: String,
    full_scan_consent: bool,
    include_backups: bool,
}

#[derive(Debug, Deserialize)]
pub(crate) struct DesktopProjectCatalogListRequest {
    include_backups: bool,
}

#[derive(Debug, Deserialize)]
pub(crate) struct DesktopProjectSelectionRequest {
    request_id: String,
    catalog_revision: Option<u64>,
    catalog_live_set_ids: Vec<String>,
    manual_als_paths: Vec<PathBuf>,
}

#[tauri::command]
pub(crate) async fn refresh_project_catalog(
    app: tauri::AppHandle,
    request: DesktopProjectCatalogRefreshRequest,
) -> Result<ProjectCatalogRefreshResult, DesktopApplicationError> {
    if !request.full_scan_consent {
        return Err(adapter_error(
            "PROJECT_SCAN_CONSENT_REQUIRED",
            "consent",
            "A broad local Project scan requires explicit user consent.",
        ));
    }
    let store_path = project_catalog_store_path(&app)?;
    let home = app.path().home_dir().ok();
    let scope = platform_project_scan_scope(home);
    if scope.roots.is_empty() {
        return Err(adapter_error(
            "PROJECT_SCAN_SCOPE_UNAVAILABLE",
            "scope",
            "No supported local Project scan root is available.",
        ));
    }
    let application_request = ProjectCatalogRefreshRequest {
        request_id: request.request_id,
        roots: scope.roots,
        excluded_roots: scope.excluded_roots,
        store_path,
        max_entries: PROJECT_SCAN_MAX_ENTRIES,
        max_depth: None,
        include_backups: request.include_backups,
        include_stale: false,
    };
    tauri::async_runtime::spawn_blocking(move || {
        rescue_application::refresh_project_catalog(&application_request)
    })
    .await
    .map_err(|failure| {
        adapter_error(
            "PROJECT_CATALOG_BACKGROUND_TASK_FAILED",
            "refresh",
            &format!("Project catalog refresh failed: {failure}"),
        )
    })
}

#[tauri::command]
pub(crate) async fn list_project_catalog(
    app: tauri::AppHandle,
    request: DesktopProjectCatalogListRequest,
) -> Result<ProjectCatalogListResult, DesktopApplicationError> {
    let application_request = ProjectCatalogListRequest {
        store_path: project_catalog_store_path(&app)?,
        include_backups: request.include_backups,
        include_stale: false,
    };
    tauri::async_runtime::spawn_blocking(move || {
        rescue_application::list_project_catalog(&application_request)
    })
    .await
    .map_err(|failure| {
        adapter_error(
            "PROJECT_CATALOG_BACKGROUND_TASK_FAILED",
            "list",
            &format!("Project catalog load failed: {failure}"),
        )
    })
}

#[tauri::command]
pub(crate) async fn resolve_project_selection(
    app: tauri::AppHandle,
    request: DesktopProjectSelectionRequest,
) -> Result<ProjectSelectionResult, DesktopApplicationError> {
    let application_request = ProjectSelectionRequest {
        request_id: request.request_id,
        store_path: project_catalog_store_path(&app)?,
        catalog_revision: request.catalog_revision,
        catalog_live_set_ids: request.catalog_live_set_ids,
        manual_als_paths: request.manual_als_paths,
    };
    tauri::async_runtime::spawn_blocking(move || {
        rescue_application::resolve_project_selection(&application_request)
    })
    .await
    .map_err(|failure| {
        adapter_error(
            "PROJECT_CATALOG_BACKGROUND_TASK_FAILED",
            "selection",
            &format!("Project selection failed: {failure}"),
        )
    })
}

fn project_catalog_store_path(app: &tauri::AppHandle) -> Result<PathBuf, DesktopApplicationError> {
    app.path()
        .app_data_dir()
        .map(|root| root.join("project-catalog").join("catalog-v0.1.json"))
        .map_err(|failure| {
            adapter_error(
                "PROJECT_CATALOG_STORE_PATH_UNAVAILABLE",
                "store",
                &format!("Private Project catalog location is unavailable: {failure}"),
            )
        })
}

fn adapter_error(code: &str, stage: &str, message: &str) -> DesktopApplicationError {
    DesktopApplicationError {
        error_code: code.to_string(),
        stage: stage.to_string(),
        message: message.to_string(),
    }
}
