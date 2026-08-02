use crate::package_planner_operations::SelectedAsset;
use crate::UnresolvedPackageRequirement;
use rescue_analyzer::DependencyAssessmentResult;
use rescue_catalog::{AssetInventoryResult, FileOccurrence};
use rescue_resolution::ResolutionDecision;
use std::collections::BTreeMap;

pub(crate) fn selected_assets<'a>(
    assessment: &'a DependencyAssessmentResult,
    inventory: &'a AssetInventoryResult,
    decisions: &[ResolutionDecision],
    planning_mode: &str,
) -> (Vec<SelectedAsset<'a>>, Vec<UnresolvedPackageRequirement>) {
    let decisions_by_asset: BTreeMap<_, _> = decisions
        .iter()
        .map(|decision| (decision.required_asset_id.as_str(), decision))
        .collect();
    let occurrences: BTreeMap<_, _> = inventory
        .file_occurrences
        .iter()
        .map(|occurrence| (occurrence.file_occurrence_id.as_str(), occurrence))
        .collect();
    let mut selected = Vec::new();
    let mut unresolved = Vec::new();
    for asset in &assessment.required_assets {
        if asset.is_confirmed_system_dependency() {
            continue;
        }
        let Some(decision) = decisions_by_asset.get(asset.required_asset_id.as_str()) else {
            continue;
        };
        if let Some(occurrence) = accepted_occurrence(decision, &occurrences) {
            selected.push(selected_asset(asset, occurrence));
        } else {
            unresolved.push(UnresolvedPackageRequirement {
                required_asset_id: asset.required_asset_id.clone(),
                decision_status: decision.decision_status.clone(),
                reason: if decision.decision_status == "unresolved" {
                    "recorded_path_missing"
                } else {
                    "resolution_not_auto_accepted"
                }
                .to_string(),
                blocks_execution: planning_mode != "current_paths_copy"
                    || decision.decision_status != "unresolved",
            });
        }
    }
    (selected, unresolved)
}

fn selected_asset<'a>(
    asset: &'a rescue_analyzer::RequiredAsset,
    occurrence: &FileOccurrence,
) -> SelectedAsset<'a> {
    SelectedAsset {
        asset,
        source_path: occurrence.native_path.clone(),
        filename: occurrence.filename.clone(),
        expected_size: occurrence.file_size,
        source_binding_id: occurrence.file_occurrence_id.clone(),
        dedup_key: occurrence.content_id.clone(),
        expected_sha256: Some(
            occurrence
                .content_id
                .trim_start_matches("sha256:")
                .to_string(),
        ),
        content_id: Some(occurrence.content_id.clone()),
        verification_policy: crate::VERIFY_SHA256_AND_SIZE.to_string(),
    }
}

fn accepted_occurrence<'a>(
    decision: &ResolutionDecision,
    occurrences: &BTreeMap<&str, &'a FileOccurrence>,
) -> Option<&'a FileOccurrence> {
    if decision.decision_status != "auto_accepted"
        || !matches!(
            decision.decision_basis.as_str(),
            "current_recorded_path_binding" | "explicit_user_selection"
        )
    {
        return None;
    }
    let occurrence_id = decision.selected_file_occurrence_id.as_deref()?;
    let occurrence = occurrences.get(occurrence_id).copied()?;
    (decision.selected_content_id.as_deref() == Some(occurrence.content_id.as_str()))
        .then_some(occurrence)
}
