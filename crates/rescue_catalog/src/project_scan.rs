use serde::{Deserialize, Serialize};
use std::path::PathBuf;

pub const PROJECT_SCANNER_VERSION: &str = "0.1.0";
pub const PROJECT_SCAN_TRAVERSAL_POLICY: &str = "project_scan_v0.1";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProjectScanRequest {
    pub scan_run_id: String,
    pub roots: Vec<PathBuf>,
    pub excluded_roots: Vec<PathBuf>,
    pub max_entries: usize,
    pub max_depth: Option<usize>,
    pub traversal_policy_version: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProjectScanResult {
    pub metadata: ProjectScanMetadata,
    pub als_files: Vec<ALSFileObservation>,
    pub project_markers: Vec<ProjectMarkerObservation>,
    pub warnings: Vec<ProjectScanWarning>,
    pub errors: Vec<ProjectScanError>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProjectScanMetadata {
    pub scanner_version: String,
    pub traversal_policy_version: String,
    pub scan_run_id: String,
    pub scan_status: String,
    pub requested_root_count: usize,
    pub accepted_root_count: usize,
    pub directories_visited: usize,
    pub entries_visited: usize,
    pub als_file_count: usize,
    pub project_marker_count: usize,
    pub skipped_symlink_count: usize,
    pub skipped_excluded_count: usize,
    pub warning_count: usize,
    pub error_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ALSFileObservation {
    pub observation_id: String,
    pub source_root: PathBuf,
    pub native_path: PathBuf,
    pub relative_path: PathBuf,
    pub filename: String,
    pub extension: String,
    pub file_size: u64,
    pub modified_time_unix_ms: Option<u64>,
    pub entry_kind: String,
    pub observation_status: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProjectMarkerObservation {
    pub marker_observation_id: String,
    pub source_root: PathBuf,
    pub project_root_candidate: PathBuf,
    pub marker_path: PathBuf,
    pub relative_marker_path: PathBuf,
    pub marker_name: String,
    pub entry_kind: String,
    pub observation_status: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProjectScanProgress {
    pub scan_run_id: String,
    pub stage: String,
    pub requested_root_count: usize,
    pub roots_completed: usize,
    pub directories_visited: usize,
    pub entries_visited: usize,
    pub als_file_count: usize,
    pub project_marker_count: usize,
    pub warning_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProjectScanWarning {
    pub warning_id: usize,
    pub warning_code: String,
    pub message: String,
    pub path: Option<PathBuf>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProjectScanError {
    pub error_code: String,
    pub message: String,
    pub path: Option<PathBuf>,
}

pub fn scan_projects(request: &ProjectScanRequest) -> ProjectScanResult {
    let mut observer = ();
    scan_projects_controlled(request, &mut observer)
}

pub fn scan_projects_controlled(
    request: &ProjectScanRequest,
    observer: &mut dyn crate::ProjectScanObserver,
) -> ProjectScanResult {
    crate::project_scan_impl::scan_projects_impl(request, observer)
}
