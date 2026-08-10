use crate::{DependencyExtractionResult, ParsedAlsPath};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

pub const PATH_OBSERVER_VERSION: &str = "0.2.0";
pub const PATH_OBSERVATION_MODEL_VERSION: &str = "0.2";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PathObservationContext {
    pub host_platform: String,
    pub confirmed_project_root: Option<PathBuf>,
    pub project_root_basis: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PathObservationResult {
    pub observation_metadata: PathObservationMetadata,
    pub dependency_observations: Vec<DependencyPathObservation>,
    pub warnings: Vec<PathObservationWarning>,
    pub errors: Vec<PathObservationError>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PathObservationMetadata {
    pub observer_version: String,
    pub path_observation_model_version: String,
    pub input_dependency_ref_version: String,
    pub source_als_path: String,
    pub source_file_hash: String,
    pub project_root_basis: Option<String>,
    pub dependency_count: usize,
    pub candidate_count: usize,
    pub regular_file_count: usize,
    pub missing_count: usize,
    pub unknown_count: usize,
    pub warning_count: usize,
    pub error_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DependencyPathObservation {
    pub dependency_id: String,
    pub als_ref_id: usize,
    pub raw_path: Option<String>,
    pub raw_relative_path: Option<String>,
    pub parsed_raw_path: Option<ParsedAlsPath>,
    pub parsed_raw_relative_path: Option<ParsedAlsPath>,
    pub candidates: Vec<CandidatePathObservation>,
    pub availability_summary: String,
    pub identity_status: String,
    pub warnings: Vec<PathObservationWarning>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CandidatePathObservation {
    pub candidate_id: String,
    pub candidate_basis: String,
    pub candidate_path: String,
    pub platform_status: String,
    pub safety_status: String,
    pub availability_status: String,
    pub entry_kind: String,
    pub size_evidence_status: String,
    pub observed_file_size: Option<u64>,
    pub expected_file_size: Option<u64>,
    pub evidence_notes: Vec<String>,
    pub warnings: Vec<PathObservationWarning>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PathObservationWarning {
    pub warning_id: usize,
    pub warning_code: String,
    pub severity: String,
    pub message: String,
    pub dependency_id: Option<String>,
    pub als_ref_id: Option<usize>,
    pub candidate_id: Option<String>,
    pub evidence_status: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PathObservationError {
    pub error_code: String,
    pub message: String,
    pub input_dependency_ref_version: String,
}

pub fn observe_dependency_paths(
    extraction: &DependencyExtractionResult,
    context: &PathObservationContext,
) -> PathObservationResult {
    crate::path_observation_impl::observe_dependency_paths_impl(extraction, context)
}
