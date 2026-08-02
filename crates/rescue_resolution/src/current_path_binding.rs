use rescue_analyzer::{DependencyAssessmentResult, RequiredAssetCandidateObservation};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::PathBuf;

pub const CURRENT_PATH_BINDING_VERSION: &str = "0.1.0";
pub const CURRENT_PATH_BINDING_POLICY_VERSION: &str = "0.1.0";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CurrentPathBindingResult {
    pub metadata: CurrentPathBindingMetadata,
    pub bindings: Vec<CurrentPathBinding>,
    pub omissions: Vec<CurrentPathBindingOmission>,
    pub errors: Vec<CurrentPathBindingError>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CurrentPathBindingMetadata {
    pub binding_version: String,
    pub policy_version: String,
    pub input_assessment_version: String,
    pub source_file_hash: String,
    pub required_asset_count: usize,
    pub binding_count: usize,
    pub omission_count: usize,
    pub error_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CurrentPathBinding {
    pub required_asset_id: String,
    pub candidate_id: String,
    pub source_path: PathBuf,
    pub filename: String,
    pub observed_size: u64,
    pub decision_basis: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CurrentPathBindingOmission {
    pub required_asset_id: String,
    pub reason: String,
    pub blocks_execution: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CurrentPathBindingError {
    pub error_code: String,
    pub message: String,
    pub required_asset_id: Option<String>,
}

pub fn bind_current_paths(assessment: &DependencyAssessmentResult) -> CurrentPathBindingResult {
    let mut bindings = Vec::new();
    let mut omissions = Vec::new();
    let mut errors = Vec::new();

    if !assessment.errors.is_empty() {
        errors.push(CurrentPathBindingError {
            error_code: "CURRENT_PATH_BINDING_UNTRUSTED_INPUT".to_string(),
            message: "Dependency assessment contains fatal errors".to_string(),
            required_asset_id: None,
        });
    } else {
        for asset in &assessment.required_assets {
            bind_asset(asset, &mut bindings, &mut omissions, &mut errors);
        }
    }

    CurrentPathBindingResult {
        metadata: CurrentPathBindingMetadata {
            binding_version: CURRENT_PATH_BINDING_VERSION.to_string(),
            policy_version: CURRENT_PATH_BINDING_POLICY_VERSION.to_string(),
            input_assessment_version: assessment.assessment_metadata.assessment_version.clone(),
            source_file_hash: assessment.assessment_metadata.source_file_hash.clone(),
            required_asset_count: assessment.required_assets.len(),
            binding_count: bindings.len(),
            omission_count: omissions.len(),
            error_count: errors.len(),
        },
        bindings,
        omissions,
        errors,
    }
}

fn bind_asset(
    asset: &rescue_analyzer::RequiredAsset,
    bindings: &mut Vec<CurrentPathBinding>,
    omissions: &mut Vec<CurrentPathBindingOmission>,
    errors: &mut Vec<CurrentPathBindingError>,
) {
    let candidates: BTreeMap<_, _> = asset
        .candidate_observations
        .iter()
        .filter(|candidate| is_existing_regular(candidate))
        .map(|candidate| (candidate.candidate_path.as_str(), candidate))
        .collect();

    if candidates.is_empty() {
        omissions.push(CurrentPathBindingOmission {
            required_asset_id: asset.required_asset_id.clone(),
            reason: "recorded_path_missing".to_string(),
            blocks_execution: false,
        });
        return;
    }
    if candidates.len() != 1 {
        omissions.push(CurrentPathBindingOmission {
            required_asset_id: asset.required_asset_id.clone(),
            reason: "multiple_current_paths_without_content_identity".to_string(),
            blocks_execution: true,
        });
        return;
    }

    let candidate = candidates.values().next().copied();
    let Some(candidate) = candidate else {
        return;
    };
    if candidate.size_evidence_status == "differs_from_expected_size" {
        omissions.push(CurrentPathBindingOmission {
            required_asset_id: asset.required_asset_id.clone(),
            reason: "recorded_path_size_conflict".to_string(),
            blocks_execution: true,
        });
        return;
    }
    let Some(observed_size) = candidate.observed_file_size else {
        omissions.push(CurrentPathBindingOmission {
            required_asset_id: asset.required_asset_id.clone(),
            reason: "recorded_path_size_unavailable".to_string(),
            blocks_execution: true,
        });
        return;
    };
    let source_path = PathBuf::from(&candidate.candidate_path);
    let Some(filename) = source_path
        .file_name()
        .and_then(|value| value.to_str())
        .map(str::to_string)
    else {
        errors.push(CurrentPathBindingError {
            error_code: "CURRENT_PATH_BINDING_FILENAME_UNSAFE".to_string(),
            message: "Current-path source filename is not portable Unicode".to_string(),
            required_asset_id: Some(asset.required_asset_id.clone()),
        });
        return;
    };
    bindings.push(CurrentPathBinding {
        required_asset_id: asset.required_asset_id.clone(),
        candidate_id: candidate.candidate_id.clone(),
        source_path,
        filename,
        observed_size,
        decision_basis: "current_recorded_path_metadata_binding".to_string(),
    });
}

fn is_existing_regular(candidate: &RequiredAssetCandidateObservation) -> bool {
    candidate.platform_status == "checkable_on_current_platform"
        && candidate.safety_status == "safe_for_metadata_read"
        && candidate.availability_status == "existing_regular_file"
        && candidate.entry_kind == "regular_file"
}
