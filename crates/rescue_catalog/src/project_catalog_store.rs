use crate::{LiveSetRecord, ProjectCatalogSnapshot, ProjectCatalogWarning, ProjectFolderRecord};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

pub const PROJECT_CATALOG_STORAGE_SCHEMA_VERSION: &str = "0.1.0";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProjectCatalogStoreRequest {
    pub store_path: PathBuf,
    pub coverage_scope_id: String,
    pub snapshot: ProjectCatalogSnapshot,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProjectCatalogStoreResult {
    pub operation_status: String,
    pub catalog: Option<StoredProjectCatalog>,
    pub warnings: Vec<ProjectCatalogStoreWarning>,
    pub errors: Vec<ProjectCatalogStoreError>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProjectCatalogLoadResult {
    pub load_status: String,
    pub catalog: Option<StoredProjectCatalog>,
    pub errors: Vec<ProjectCatalogStoreError>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StoredProjectCatalog {
    pub metadata: ProjectCatalogStoreMetadata,
    pub project_folders: Vec<StoredProjectFolderRecord>,
    pub live_sets: Vec<StoredLiveSetRecord>,
    pub catalog_warnings: Vec<ProjectCatalogWarning>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProjectCatalogStoreMetadata {
    pub storage_schema_version: String,
    pub revision: u64,
    pub coverage_scope_id: String,
    pub last_snapshot_id: String,
    pub last_scan_run_id: String,
    pub last_scan_status: String,
    pub coverage_status: String,
    pub project_folder_count: usize,
    pub live_set_count: usize,
    pub observed_project_folder_count: usize,
    pub observed_live_set_count: usize,
    pub retained_project_folder_count: usize,
    pub retained_live_set_count: usize,
    pub stale_project_folder_count: usize,
    pub stale_live_set_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StoredProjectFolderRecord {
    pub record: ProjectFolderRecord,
    pub freshness_status: String,
    pub last_observed_scan_run_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StoredLiveSetRecord {
    pub record: LiveSetRecord,
    pub freshness_status: String,
    pub last_observed_scan_run_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProjectCatalogStoreWarning {
    pub warning_code: String,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProjectCatalogStoreError {
    pub error_code: String,
    pub message: String,
}

pub fn store_project_catalog(request: &ProjectCatalogStoreRequest) -> ProjectCatalogStoreResult {
    crate::project_catalog_store_impl::store_project_catalog_impl(request)
}

pub fn load_project_catalog(store_path: &Path) -> ProjectCatalogLoadResult {
    crate::project_catalog_store_impl::load_project_catalog_impl(store_path)
}
