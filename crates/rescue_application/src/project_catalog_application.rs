use serde::{Deserialize, Serialize};
use std::path::PathBuf;

pub const PROJECT_CATALOG_APPLICATION_SERVICE_VERSION: &str = "0.1.0";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProjectCatalogRefreshRequest {
    pub request_id: String,
    pub roots: Vec<PathBuf>,
    pub excluded_roots: Vec<PathBuf>,
    pub store_path: PathBuf,
    pub max_entries: usize,
    pub max_depth: Option<usize>,
    pub include_backups: bool,
    pub include_stale: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProjectCatalogRefreshResult {
    pub service_version: String,
    pub request_id: String,
    pub refresh_status: String,
    pub scan_status: String,
    pub store_status: String,
    pub catalog: ProjectCatalogListResult,
    pub warnings: Vec<ProjectCatalogApplicationWarning>,
    pub errors: Vec<ProjectCatalogApplicationError>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProjectCatalogListRequest {
    pub store_path: PathBuf,
    pub include_backups: bool,
    pub include_stale: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProjectCatalogListResult {
    pub metadata: ProjectCatalogListMetadata,
    pub groups: Vec<ProjectListGroup>,
    pub items: Vec<ProjectListItem>,
    pub warnings: Vec<ProjectCatalogApplicationWarning>,
    pub errors: Vec<ProjectCatalogApplicationError>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProjectCatalogListMetadata {
    pub service_version: String,
    pub catalog_revision: u64,
    pub coverage_status: String,
    pub total_group_count: usize,
    pub total_item_count: usize,
    pub visible_item_count: usize,
    pub hidden_backup_count: usize,
    pub hidden_stale_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProjectListGroup {
    pub group_id: String,
    pub display_name: String,
    pub group_kind: String,
    pub item_ids: Vec<String>,
    pub main_set_count: usize,
    pub backup_set_count: usize,
    pub freshness_status: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProjectListItem {
    pub item_id: String,
    pub live_set_id: String,
    pub group_id: String,
    pub display_name: String,
    pub project_display_name: Option<String>,
    pub location_kind: String,
    pub freshness_status: String,
    pub selection_status: String,
    pub modified_time_unix_ms: Option<u64>,
    pub file_size: u64,
    pub duplicate_display_name_count: usize,
    pub warning_codes: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProjectSelectionRequest {
    pub request_id: String,
    pub store_path: PathBuf,
    pub catalog_revision: Option<u64>,
    pub catalog_live_set_ids: Vec<String>,
    pub manual_als_paths: Vec<PathBuf>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProjectSelectionResult {
    pub service_version: String,
    pub request_id: String,
    pub selection_status: String,
    pub selections: Vec<ProjectSelection>,
    pub warnings: Vec<ProjectCatalogApplicationWarning>,
    pub errors: Vec<ProjectCatalogApplicationError>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProjectSelection {
    pub selection_id: String,
    pub selection_source: String,
    pub live_set_id: Option<String>,
    pub native_als_path: PathBuf,
    pub catalog_revision: Option<u64>,
    pub observation_fingerprint: Option<String>,
    pub freshness_status: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProjectCatalogApplicationWarning {
    pub warning_code: String,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProjectCatalogApplicationError {
    pub error_code: String,
    pub stage: String,
    pub message: String,
}

pub fn refresh_project_catalog(
    request: &ProjectCatalogRefreshRequest,
) -> ProjectCatalogRefreshResult {
    crate::project_catalog_refresh::refresh_project_catalog_impl(request)
}

pub fn list_project_catalog(request: &ProjectCatalogListRequest) -> ProjectCatalogListResult {
    crate::project_catalog_projection::list_project_catalog_impl(request)
}

pub fn resolve_project_selection(request: &ProjectSelectionRequest) -> ProjectSelectionResult {
    crate::project_selection::resolve_project_selection_impl(request)
}
