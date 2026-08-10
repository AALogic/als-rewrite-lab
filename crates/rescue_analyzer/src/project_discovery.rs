use serde::{Deserialize, Serialize};
use std::path::PathBuf;

pub const PROJECT_DISCOVERY_VERSION: &str = "0.1.0";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProjectDiscoveryRequest {
    pub source_als_path: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProjectDiscoveryResult {
    pub metadata: ProjectDiscoveryMetadata,
    pub candidates: Vec<ProjectRootCandidate>,
    pub confirmed_project_root: Option<PathBuf>,
    pub discovery_status: String,
    pub set_location: String,
    pub warnings: Vec<ProjectDiscoveryWarning>,
    pub errors: Vec<ProjectDiscoveryError>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProjectDiscoveryMetadata {
    pub discovery_version: String,
    pub source_als_path: PathBuf,
    pub ancestors_checked: usize,
    pub candidate_count: usize,
    pub warning_count: usize,
    pub error_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProjectRootCandidate {
    pub candidate_path: PathBuf,
    pub marker_path: PathBuf,
    pub marker_status: String,
    pub depth_from_set: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProjectDiscoveryWarning {
    pub warning_id: usize,
    pub warning_code: String,
    pub message: String,
    pub path: Option<PathBuf>,
    pub evidence_status: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProjectDiscoveryError {
    pub error_code: String,
    pub message: String,
    pub path: Option<PathBuf>,
}

pub fn discover_project(request: &ProjectDiscoveryRequest) -> ProjectDiscoveryResult {
    crate::project_discovery_impl::discover_project_impl(request)
}
