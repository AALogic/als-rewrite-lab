use crate::asset_resolution_result::{complete_result, fatal_result};
use crate::asset_resolution_scoring::scored_candidate;
use crate::asset_resolution_selection;
use crate::{
    AssetResolutionResult, AssetResolutionWarning, ResolutionCandidate, ResolutionDecision,
    ResolutionProposal, UserAssetSelection, UserSelectionSet, RESOLUTION_POLICY_VERSION,
};
use rescue_analyzer::{DependencyAssessmentResult, RequiredAsset};
use rescue_catalog::AssetInventoryResult;

const AUTO_ACCEPT_THRESHOLD: u8 = 95;
const EXACT_PATH_EVIDENCE: &str = "exact_observed_native_path";

pub(crate) fn resolve_assets_impl(
    assessment: &DependencyAssessmentResult,
    inventory: &AssetInventoryResult,
    selections: Option<&UserSelectionSet>,
) -> AssetResolutionResult {
    if let Some((code, message)) = validate_inputs(assessment, inventory) {
        return fatal_result(assessment, inventory, code, message);
    }

    let validated_selections = match asset_resolution_selection::validate(assessment, selections) {
        Ok(value) => value,
        Err((code, message)) => return fatal_result(assessment, inventory, code, message),
    };
    let mut warnings = Vec::new();
    if inventory.metadata.scan_status == "partial" {
        push_warning(
            &mut warnings,
            "RESOLUTION_PARTIAL_INVENTORY",
            "Inventory is partial; automatic acceptance is disabled",
            None,
        );
    }
    let mut proposals = Vec::new();
    let mut decisions = Vec::new();
    for asset in &assessment.required_assets {
        if asset.original_crc.is_some() {
            push_warning(
                &mut warnings,
                "RESOLUTION_ORIGINAL_CRC_NOT_SCORED",
                "OriginalCrc has no confirmed candidate-side identity meaning",
                Some(&asset.required_asset_id),
            );
        }
        let candidates = candidates_for(asset, inventory);
        let decision = decide(
            asset,
            &candidates,
            &inventory.metadata.scan_status,
            validated_selections.get(&asset.required_asset_id),
            &mut warnings,
        );
        proposals.push(ResolutionProposal {
            required_asset_id: asset.required_asset_id.clone(),
            recorded_filename: asset.filename.clone(),
            candidates,
        });
        decisions.push(decision);
    }
    complete_result(assessment, inventory, proposals, decisions, warnings)
}

fn validate_inputs(
    assessment: &DependencyAssessmentResult,
    inventory: &AssetInventoryResult,
) -> Option<(&'static str, &'static str)> {
    if assessment.assessment_metadata.assessment_version != "0.2.0" {
        return Some((
            "RESOLUTION_UNSUPPORTED_ASSESSMENT_MODEL",
            "DependencyAssessmentResult v0.2.0 is required",
        ));
    }
    if inventory.metadata.inventory_version != "0.1.0" {
        return Some((
            "RESOLUTION_UNSUPPORTED_INVENTORY_MODEL",
            "AssetInventoryResult v0.1.0 is required",
        ));
    }
    if !assessment.errors.is_empty() {
        return Some((
            "RESOLUTION_UNTRUSTED_ASSESSMENT",
            "Dependency assessment contains fatal errors",
        ));
    }
    if !inventory.errors.is_empty() || inventory.metadata.scan_status == "failed" {
        return Some((
            "RESOLUTION_UNTRUSTED_INVENTORY",
            "Asset inventory contains fatal errors",
        ));
    }
    None
}

fn candidates_for(
    asset: &RequiredAsset,
    inventory: &AssetInventoryResult,
) -> Vec<ResolutionCandidate> {
    let mut candidates: Vec<_> = inventory
        .file_occurrences
        .iter()
        .filter_map(|occurrence| scored_candidate(asset, occurrence))
        .collect();
    candidates.sort_by(|left, right| {
        right
            .score
            .cmp(&left.score)
            .then_with(|| left.native_path.cmp(&right.native_path))
            .then_with(|| left.file_occurrence_id.cmp(&right.file_occurrence_id))
    });
    candidates
}

fn decide(
    asset: &RequiredAsset,
    candidates: &[ResolutionCandidate],
    scan_status: &str,
    selection: Option<&UserAssetSelection>,
    warnings: &mut Vec<AssetResolutionWarning>,
) -> ResolutionDecision {
    if scan_status == "complete" {
        if let Some(selection) = selection {
            if let Some(candidate) =
                asset_resolution_selection::matching_candidate(candidates, selection)
            {
                return accepted_decision(asset, candidate, "explicit_user_selection");
            }
            push_warning(
                warnings,
                "RESOLUTION_USER_SELECTION_STALE_OR_CHANGED",
                "Selected path and SHA-256 do not match a current candidate",
                Some(&asset.required_asset_id),
            );
            return empty_selection_decision(
                asset,
                "needs_user_confirmation",
                "user_selection_stale_or_changed",
                true,
            );
        }
        let bound: Vec<_> = candidates
            .iter()
            .filter(|candidate| {
                candidate.conflicts.is_empty() && has_evidence(candidate, EXACT_PATH_EVIDENCE)
            })
            .collect();
        if bound.len() == 1 {
            return accepted_decision(asset, bound[0], "current_recorded_path_binding");
        }
    }
    let qualified: Vec<_> = candidates
        .iter()
        .filter(|candidate| {
            candidate.score >= AUTO_ACCEPT_THRESHOLD && candidate.conflicts.is_empty()
        })
        .collect();
    if qualified.len() > 1 {
        push_warning(
            warnings,
            "RESOLUTION_AMBIGUOUS_HIGH_CONFIDENCE",
            "Multiple candidates meet the automatic threshold",
            Some(&asset.required_asset_id),
        );
    }
    if candidates.is_empty() {
        unresolved_decision(asset)
    } else {
        manual_decision(asset, scan_status, qualified.len())
    }
}

fn has_evidence(candidate: &ResolutionCandidate, code: &str) -> bool {
    candidate
        .evidence
        .iter()
        .any(|evidence| evidence.evidence_code == code)
}

fn accepted_decision(
    asset: &RequiredAsset,
    candidate: &ResolutionCandidate,
    basis: &str,
) -> ResolutionDecision {
    ResolutionDecision {
        required_asset_id: asset.required_asset_id.clone(),
        decision_status: "auto_accepted".to_string(),
        selected_candidate_id: Some(candidate.candidate_id.clone()),
        selected_file_occurrence_id: Some(candidate.file_occurrence_id.clone()),
        selected_content_id: Some(candidate.content_id.clone()),
        score: Some(candidate.score),
        policy_version: RESOLUTION_POLICY_VERSION.to_string(),
        decision_basis: basis.to_string(),
        requires_user_confirmation: false,
    }
}

fn manual_decision(
    asset: &RequiredAsset,
    scan_status: &str,
    qualified_count: usize,
) -> ResolutionDecision {
    let basis = if scan_status != "complete" {
        "inventory_incomplete"
    } else if qualified_count > 1 {
        "ambiguous_high_confidence_candidates"
    } else if qualified_count == 1 {
        "explicit_user_selection_required"
    } else {
        "candidate_below_automatic_threshold"
    };
    empty_selection_decision(asset, "needs_user_confirmation", basis, true)
}

fn unresolved_decision(asset: &RequiredAsset) -> ResolutionDecision {
    empty_selection_decision(asset, "unresolved", "no_plausible_candidate", false)
}

fn empty_selection_decision(
    asset: &RequiredAsset,
    status: &str,
    basis: &str,
    confirmation: bool,
) -> ResolutionDecision {
    ResolutionDecision {
        required_asset_id: asset.required_asset_id.clone(),
        decision_status: status.to_string(),
        selected_candidate_id: None,
        selected_file_occurrence_id: None,
        selected_content_id: None,
        score: None,
        policy_version: RESOLUTION_POLICY_VERSION.to_string(),
        decision_basis: basis.to_string(),
        requires_user_confirmation: confirmation,
    }
}

fn push_warning(
    warnings: &mut Vec<AssetResolutionWarning>,
    code: &str,
    message: &str,
    asset_id: Option<&str>,
) {
    warnings.push(AssetResolutionWarning {
        warning_id: warnings.len(),
        warning_code: code.to_string(),
        message: message.to_string(),
        required_asset_id: asset_id.map(str::to_string),
    });
}
