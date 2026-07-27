use crate::dependency_assessment_grouping::group_occurrences;
use crate::dependency_assessment_result::{complete_result, fatal_result};
use crate::dependency_assessment_status::{
    add_warnings, availability_status, candidate_observations, risk_flags,
};
use crate::{DependencyAssessmentResult, DependencyAssessmentWarning, RequiredAsset};
use rescue_core::{DependencyExtractionResult, DependencyPathObservation, PathObservationResult};
use std::collections::{BTreeMap, BTreeSet};

pub(crate) fn assess_dependencies_impl(
    extraction: &DependencyExtractionResult,
    observations: &PathObservationResult,
) -> DependencyAssessmentResult {
    if let Some((code, message)) = validate_handoff(extraction, observations) {
        return fatal_result(extraction, observations, code, message);
    }

    let observations_by_id: BTreeMap<_, _> = observations
        .dependency_observations
        .iter()
        .map(|item| (item.dependency_id.as_str(), item))
        .collect();
    let mut warnings = Vec::new();
    let required_assets = group_occurrences(&extraction.dependencies)
        .into_iter()
        .enumerate()
        .map(|(group_index, group)| {
            let dependencies: Vec<_> = group
                .dependency_indexes
                .iter()
                .map(|index| &extraction.dependencies[*index])
                .collect();
            let grouped_observations: Vec<_> = dependencies
                .iter()
                .map(|dependency| observations_by_id[dependency.dependency_id.as_str()])
                .collect();
            build_required_asset(
                group_index,
                group.basis,
                &dependencies,
                &grouped_observations,
                &mut warnings,
            )
        })
        .collect();
    complete_result(extraction, observations, required_assets, warnings)
}

fn validate_handoff(
    extraction: &DependencyExtractionResult,
    observations: &PathObservationResult,
) -> Option<(&'static str, &'static str)> {
    if extraction.extraction_metadata.dependency_ref_version != "0.1" {
        return Some((
            "ASSESSMENT_UNSUPPORTED_EXTRACTION_MODEL",
            "DependencyRef v0.1 is required",
        ));
    }
    if observations
        .observation_metadata
        .path_observation_model_version
        != "0.2"
    {
        return Some((
            "ASSESSMENT_UNSUPPORTED_OBSERVATION_MODEL",
            "PathObservationResult v0.2 is required",
        ));
    }
    if !extraction.errors.is_empty() {
        return Some((
            "ASSESSMENT_UNTRUSTED_EXTRACTION",
            "Dependency extraction contains fatal errors",
        ));
    }
    if !observations.errors.is_empty() {
        return Some((
            "ASSESSMENT_UNTRUSTED_OBSERVATIONS",
            "Path observations contain fatal errors",
        ));
    }
    if extraction.extraction_metadata.source_file_hash
        != observations.observation_metadata.source_file_hash
    {
        return Some((
            "ASSESSMENT_SNAPSHOT_MISMATCH",
            "Inputs describe different ALS snapshots",
        ));
    }
    let dependency_ids: BTreeSet<_> = extraction
        .dependencies
        .iter()
        .map(|item| item.dependency_id.as_str())
        .collect();
    let observation_ids: BTreeSet<_> = observations
        .dependency_observations
        .iter()
        .map(|item| item.dependency_id.as_str())
        .collect();
    if dependency_ids.len() != extraction.dependencies.len()
        || observation_ids.len() != observations.dependency_observations.len()
        || dependency_ids != observation_ids
    {
        return Some((
            "ASSESSMENT_OCCURRENCE_CONTRACT_MISMATCH",
            "Occurrence IDs are not a one-to-one handoff",
        ));
    }
    None
}

fn build_required_asset(
    group_index: usize,
    basis: &str,
    dependencies: &[&rescue_core::DependencyRef],
    observations: &[&DependencyPathObservation],
    warnings: &mut Vec<DependencyAssessmentWarning>,
) -> RequiredAsset {
    let required_asset_id = format!("required_asset_{group_index:06}");
    let first = dependencies[0];
    let candidates = candidate_observations(observations);
    let availability_status = availability_status(&candidates);
    let risk_flags = risk_flags(basis, &candidates, &availability_status);
    add_warnings(
        &required_asset_id,
        basis,
        dependencies,
        &candidates,
        &availability_status,
        warnings,
    );
    RequiredAsset {
        required_asset_id,
        grouping_basis: basis.to_string(),
        dependency_ids: dependencies
            .iter()
            .map(|item| item.dependency_id.clone())
            .collect(),
        als_ref_ids: dependencies.iter().map(|item| item.als_ref_id).collect(),
        occurrence_count: dependencies.len(),
        filename: first.filename.clone(),
        extension: first.extension.clone(),
        original_file_size: first.original_file_size.clone(),
        original_crc: first.original_crc.clone(),
        candidate_observations: candidates,
        availability_status,
        resolution_status: "unresolved".to_string(),
        risk_flags,
        evidence_status: "observed_unresolved".to_string(),
    }
}
