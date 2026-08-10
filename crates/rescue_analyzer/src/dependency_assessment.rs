use rescue_core::{DependencyExtractionResult, PathObservationResult};
use serde::{Deserialize, Serialize};

pub const DEPENDENCY_ASSESSMENT_VERSION: &str = "0.2.0";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DependencyAssessmentResult {
    pub assessment_metadata: DependencyAssessmentMetadata,
    pub required_assets: Vec<RequiredAsset>,
    pub warnings: Vec<DependencyAssessmentWarning>,
    pub errors: Vec<DependencyAssessmentError>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DependencyAssessmentMetadata {
    pub assessment_version: String,
    pub input_dependency_ref_version: String,
    pub input_path_observation_model_version: String,
    pub source_als_path: String,
    pub source_file_hash: String,
    pub occurrence_count: usize,
    pub required_asset_count: usize,
    pub regular_file_candidate_asset_count: usize,
    pub missing_candidate_asset_count: usize,
    pub unknown_asset_count: usize,
    pub warning_count: usize,
    pub error_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RequiredAsset {
    pub required_asset_id: String,
    pub grouping_basis: String,
    pub dependency_ids: Vec<String>,
    pub als_ref_ids: Vec<usize>,
    pub occurrence_count: usize,
    pub filename: Option<String>,
    pub extension: Option<String>,
    pub original_file_size: Option<String>,
    pub original_crc: Option<String>,
    pub source_category: String,
    pub management_class: String,
    pub source_classification_status: String,
    pub source_classification_basis: String,
    pub candidate_observations: Vec<RequiredAssetCandidateObservation>,
    pub availability_status: String,
    pub resolution_status: String,
    pub risk_flags: Vec<String>,
    pub evidence_status: String,
}

impl RequiredAsset {
    pub fn is_confirmed_system_dependency(&self) -> bool {
        self.management_class == "system_dependency"
            && self.source_classification_status == "confirmed"
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RequiredAssetCandidateObservation {
    pub dependency_id: String,
    pub als_ref_id: usize,
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
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DependencyAssessmentWarning {
    pub warning_id: usize,
    pub warning_code: String,
    pub message: String,
    pub required_asset_id: Option<String>,
    pub dependency_id: Option<String>,
    pub evidence_status: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DependencyAssessmentError {
    pub error_code: String,
    pub message: String,
    pub source_file_hash: String,
}

pub fn assess_dependencies(
    extraction: &DependencyExtractionResult,
    observations: &PathObservationResult,
) -> DependencyAssessmentResult {
    crate::dependency_assessment_impl::assess_dependencies_impl(extraction, observations)
}
