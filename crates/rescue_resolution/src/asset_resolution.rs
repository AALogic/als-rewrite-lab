use rescue_analyzer::DependencyAssessmentResult;
use rescue_catalog::AssetInventoryResult;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

pub const ASSET_RESOLUTION_VERSION: &str = "0.1.0";
pub const RESOLUTION_POLICY_VERSION: &str = "0.3.0";
pub const USER_SELECTION_SCHEMA_VERSION: &str = "0.1";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UserSelectionSet {
    pub selection_schema_version: String,
    pub source_als_sha256: String,
    pub selections: Vec<UserAssetSelection>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UserAssetSelection {
    pub required_asset_id: String,
    pub selected_native_path: PathBuf,
    pub selected_content_sha256: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AssetResolutionResult {
    pub metadata: AssetResolutionMetadata,
    pub proposals: Vec<ResolutionProposal>,
    pub decisions: Vec<ResolutionDecision>,
    pub warnings: Vec<AssetResolutionWarning>,
    pub errors: Vec<AssetResolutionError>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AssetResolutionMetadata {
    pub resolution_version: String,
    pub policy_version: String,
    pub input_assessment_version: String,
    pub input_inventory_version: String,
    pub scan_run_id: String,
    pub required_asset_count: usize,
    pub proposal_count: usize,
    pub auto_accepted_count: usize,
    pub manual_review_count: usize,
    pub unresolved_count: usize,
    pub warning_count: usize,
    pub error_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResolutionProposal {
    pub required_asset_id: String,
    pub recorded_filename: Option<String>,
    pub candidates: Vec<ResolutionCandidate>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResolutionCandidate {
    pub candidate_id: String,
    pub file_occurrence_id: String,
    pub content_id: String,
    pub native_path: PathBuf,
    pub score: u8,
    pub evidence: Vec<ResolutionEvidence>,
    pub conflicts: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResolutionEvidence {
    pub evidence_code: String,
    pub weight: u8,
    pub explanation: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResolutionDecision {
    pub required_asset_id: String,
    pub decision_status: String,
    pub selected_candidate_id: Option<String>,
    pub selected_file_occurrence_id: Option<String>,
    pub selected_content_id: Option<String>,
    pub score: Option<u8>,
    pub policy_version: String,
    pub decision_basis: String,
    pub requires_user_confirmation: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AssetResolutionWarning {
    pub warning_id: usize,
    pub warning_code: String,
    pub message: String,
    pub required_asset_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AssetResolutionError {
    pub error_code: String,
    pub message: String,
}

pub fn resolve_assets(
    assessment: &DependencyAssessmentResult,
    inventory: &AssetInventoryResult,
) -> AssetResolutionResult {
    resolve_assets_with_selections(assessment, inventory, None)
}

pub fn resolve_assets_with_selections(
    assessment: &DependencyAssessmentResult,
    inventory: &AssetInventoryResult,
    selections: Option<&UserSelectionSet>,
) -> AssetResolutionResult {
    crate::asset_resolution_impl::resolve_assets_impl(assessment, inventory, selections)
}
