use crate::{DependencyAssessmentWarning, RequiredAssetCandidateObservation};
use rescue_core::{DependencyPathObservation, DependencyRef};

pub(crate) fn candidate_observations(
    observations: &[&DependencyPathObservation],
) -> Vec<RequiredAssetCandidateObservation> {
    observations
        .iter()
        .flat_map(|observation| {
            observation
                .candidates
                .iter()
                .map(|candidate| RequiredAssetCandidateObservation {
                    dependency_id: observation.dependency_id.clone(),
                    als_ref_id: observation.als_ref_id,
                    candidate_id: candidate.candidate_id.clone(),
                    candidate_basis: candidate.candidate_basis.clone(),
                    candidate_path: candidate.candidate_path.clone(),
                    platform_status: candidate.platform_status.clone(),
                    safety_status: candidate.safety_status.clone(),
                    availability_status: candidate.availability_status.clone(),
                    entry_kind: candidate.entry_kind.clone(),
                    size_evidence_status: candidate.size_evidence_status.clone(),
                    observed_file_size: candidate.observed_file_size,
                    expected_file_size: candidate.expected_file_size,
                })
        })
        .collect()
}

pub(crate) fn availability_status(candidates: &[RequiredAssetCandidateObservation]) -> String {
    if candidates
        .iter()
        .any(|item| item.availability_status == "existing_regular_file")
    {
        "regular_file_candidate_observed"
    } else if !candidates.is_empty()
        && candidates
            .iter()
            .all(|item| item.availability_status == "missing")
    {
        "no_regular_file_candidate_observed"
    } else {
        "unknown"
    }
    .to_string()
}

pub(crate) fn risk_flags(
    basis: &str,
    candidates: &[RequiredAssetCandidateObservation],
    availability: &str,
) -> Vec<String> {
    let mut flags = Vec::new();
    if basis == "single_incomplete_occurrence" {
        flags.push("incomplete_reference_claim".to_string());
    }
    if regular_candidate_count(candidates) > 1 {
        flags.push("multiple_regular_file_candidates_observed".to_string());
    }
    if candidates
        .iter()
        .any(|item| item.size_evidence_status == "differs_from_expected_size")
    {
        flags.push("candidate_size_differs".to_string());
    }
    if availability == "no_regular_file_candidate_observed" {
        flags.push("required_audio_not_locally_observed".to_string());
    }
    if availability == "unknown" {
        flags.push("availability_unknown".to_string());
    }
    flags
}

pub(crate) fn add_warnings(
    asset_id: &str,
    basis: &str,
    dependencies: &[&DependencyRef],
    candidates: &[RequiredAssetCandidateObservation],
    availability: &str,
    warnings: &mut Vec<DependencyAssessmentWarning>,
) {
    if basis == "single_incomplete_occurrence" {
        push_warning(
            warnings,
            "ASSESSMENT_INCOMPLETE_REFERENCE",
            "Incomplete reference was kept as a separate requirement",
            asset_id,
            dependencies[0],
        );
    }
    if regular_candidate_count(candidates) > 1 {
        push_warning(
            warnings,
            "ASSESSMENT_MULTIPLE_REGULAR_FILE_CANDIDATES",
            "Multiple local file candidates were observed; none was selected",
            asset_id,
            dependencies[0],
        );
    }
    if availability == "unknown" {
        push_warning(
            warnings,
            "ASSESSMENT_AVAILABILITY_UNKNOWN",
            "Requirement availability could not be established",
            asset_id,
            dependencies[0],
        );
    }
}

fn regular_candidate_count(candidates: &[RequiredAssetCandidateObservation]) -> usize {
    candidates
        .iter()
        .filter(|item| item.availability_status == "existing_regular_file")
        .count()
}

fn push_warning(
    warnings: &mut Vec<DependencyAssessmentWarning>,
    code: &str,
    message: &str,
    asset_id: &str,
    dependency: &DependencyRef,
) {
    warnings.push(DependencyAssessmentWarning {
        warning_id: warnings.len(),
        warning_code: code.to_string(),
        message: message.to_string(),
        required_asset_id: Some(asset_id.to_string()),
        dependency_id: Some(dependency.dependency_id.clone()),
        evidence_status: "observed".to_string(),
    });
}
