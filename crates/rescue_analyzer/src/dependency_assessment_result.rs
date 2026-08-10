use crate::{
    DependencyAssessmentError, DependencyAssessmentMetadata, DependencyAssessmentResult,
    DependencyAssessmentWarning, RequiredAsset, DEPENDENCY_ASSESSMENT_VERSION,
};
use rescue_core::{DependencyExtractionResult, PathObservationResult};

pub(crate) fn complete_result(
    extraction: &DependencyExtractionResult,
    observations: &PathObservationResult,
    assets: Vec<RequiredAsset>,
    warnings: Vec<DependencyAssessmentWarning>,
) -> DependencyAssessmentResult {
    let regular = count_assets(&assets, "regular_file_candidate_observed");
    let missing = count_assets(&assets, "no_regular_file_candidate_observed");
    let unknown = count_assets(&assets, "unknown");
    DependencyAssessmentResult {
        assessment_metadata: metadata(
            extraction,
            observations,
            assets.len(),
            regular,
            missing,
            unknown,
            warnings.len(),
            0,
        ),
        required_assets: assets,
        warnings,
        errors: Vec::new(),
    }
}

pub(crate) fn fatal_result(
    extraction: &DependencyExtractionResult,
    observations: &PathObservationResult,
    code: &str,
    message: &str,
) -> DependencyAssessmentResult {
    DependencyAssessmentResult {
        assessment_metadata: metadata(extraction, observations, 0, 0, 0, 0, 0, 1),
        required_assets: Vec::new(),
        warnings: Vec::new(),
        errors: vec![DependencyAssessmentError {
            error_code: code.to_string(),
            message: message.to_string(),
            source_file_hash: extraction.extraction_metadata.source_file_hash.clone(),
        }],
    }
}

fn count_assets(assets: &[RequiredAsset], status: &str) -> usize {
    assets
        .iter()
        .filter(|asset| asset.availability_status == status)
        .count()
}

#[allow(clippy::too_many_arguments)]
fn metadata(
    extraction: &DependencyExtractionResult,
    observations: &PathObservationResult,
    asset_count: usize,
    regular: usize,
    missing: usize,
    unknown: usize,
    warning_count: usize,
    error_count: usize,
) -> DependencyAssessmentMetadata {
    DependencyAssessmentMetadata {
        assessment_version: DEPENDENCY_ASSESSMENT_VERSION.to_string(),
        input_dependency_ref_version: extraction
            .extraction_metadata
            .dependency_ref_version
            .clone(),
        input_path_observation_model_version: observations
            .observation_metadata
            .path_observation_model_version
            .clone(),
        source_als_path: extraction.extraction_metadata.source_als_path.clone(),
        source_file_hash: extraction.extraction_metadata.source_file_hash.clone(),
        occurrence_count: extraction.dependencies.len(),
        required_asset_count: asset_count,
        regular_file_candidate_asset_count: regular,
        missing_candidate_asset_count: missing,
        unknown_asset_count: unknown,
        warning_count,
        error_count,
    }
}
