use crate::ProjectScanResult;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

pub const PROJECT_CATALOG_VERSION: &str = "0.1.0";
pub const PROJECT_CATALOG_IDENTITY_POLICY: &str = "catalog_native_path_sha256_v0.1";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProjectCatalogBuildRequest {
    pub snapshot_id: String,
    pub scan_result: ProjectScanResult,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProjectCatalogSnapshot {
    pub metadata: ProjectCatalogMetadata,
    pub project_folders: Vec<ProjectFolderRecord>,
    pub live_sets: Vec<LiveSetRecord>,
    pub warnings: Vec<ProjectCatalogWarning>,
    pub errors: Vec<ProjectCatalogError>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProjectCatalogMetadata {
    pub catalog_version: String,
    pub identity_policy_version: String,
    pub snapshot_id: String,
    pub source_scan_run_id: String,
    pub source_scan_status: String,
    pub build_status: String,
    pub project_folder_count: usize,
    pub live_set_count: usize,
    pub main_set_count: usize,
    pub backup_set_count: usize,
    pub ungrouped_set_count: usize,
    pub ambiguous_set_count: usize,
    pub source_warning_count: usize,
    pub source_error_count: usize,
    pub warning_count: usize,
    pub error_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProjectFolderRecord {
    pub project_folder_id: String,
    pub source_root: PathBuf,
    pub native_root_path: PathBuf,
    pub marker_path: PathBuf,
    pub display_name: String,
    pub structural_status: String,
    pub main_set_ids: Vec<String>,
    pub backup_set_ids: Vec<String>,
    pub latest_modified_time_unix_ms: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LiveSetRecord {
    pub live_set_id: String,
    pub source_root: PathBuf,
    pub native_als_path: PathBuf,
    pub relative_path: PathBuf,
    pub display_name: String,
    pub file_size: u64,
    pub modified_time_unix_ms: Option<u64>,
    pub location_kind: String,
    pub association_status: String,
    pub project_folder_id: Option<String>,
    pub candidate_project_folder_ids: Vec<String>,
    pub observation_fingerprint: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProjectCatalogWarning {
    pub warning_id: usize,
    pub warning_code: String,
    pub message: String,
    pub related_set_id: Option<String>,
    pub related_project_folder_ids: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProjectCatalogError {
    pub error_code: String,
    pub message: String,
}

pub fn build_project_catalog(request: ProjectCatalogBuildRequest) -> ProjectCatalogSnapshot {
    crate::project_catalog_impl::build_project_catalog_impl(request)
}
