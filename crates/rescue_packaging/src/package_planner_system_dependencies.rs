use crate::SystemDependencyRequirement;
use rescue_analyzer::DependencyAssessmentResult;
use std::collections::BTreeSet;
use std::path::PathBuf;

pub(crate) fn planned_system_dependencies(
    assessment: &DependencyAssessmentResult,
) -> Vec<SystemDependencyRequirement> {
    assessment
        .required_assets
        .iter()
        .filter(|asset| asset.is_confirmed_system_dependency())
        .map(|asset| SystemDependencyRequirement {
            required_asset_id: asset.required_asset_id.clone(),
            source_category: asset.source_category.clone(),
            filename: asset.filename.clone(),
            occurrence_count: asset.occurrence_count,
            als_ref_ids: asset.als_ref_ids.clone(),
            observed_source_paths: asset
                .candidate_observations
                .iter()
                .filter(|candidate| candidate.availability_status == "existing_regular_file")
                .map(|candidate| PathBuf::from(&candidate.candidate_path))
                .collect::<BTreeSet<_>>()
                .into_iter()
                .collect(),
            package_action: "leave_system_managed".to_string(),
            portability_status: "portable_risk".to_string(),
            reason: "ableton_core_library_dependency".to_string(),
        })
        .collect()
}
