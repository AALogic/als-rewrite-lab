use crate::{
    AssetResolutionError, AssetResolutionMetadata, AssetResolutionResult, AssetResolutionWarning,
    ResolutionDecision, ResolutionProposal, ASSET_RESOLUTION_VERSION, RESOLUTION_POLICY_VERSION,
};
use rescue_analyzer::DependencyAssessmentResult;
use rescue_catalog::AssetInventoryResult;

pub(crate) fn complete_result(
    assessment: &DependencyAssessmentResult,
    inventory: &AssetInventoryResult,
    proposals: Vec<ResolutionProposal>,
    decisions: Vec<ResolutionDecision>,
    warnings: Vec<AssetResolutionWarning>,
) -> AssetResolutionResult {
    let auto = count_decisions(&decisions, "auto_accepted");
    let manual = count_decisions(&decisions, "needs_user_confirmation");
    let unresolved = count_decisions(&decisions, "unresolved");
    AssetResolutionResult {
        metadata: metadata(
            assessment,
            inventory,
            proposals.len(),
            auto,
            manual,
            unresolved,
            warnings.len(),
            0,
        ),
        proposals,
        decisions,
        warnings,
        errors: Vec::new(),
    }
}

pub(crate) fn fatal_result(
    assessment: &DependencyAssessmentResult,
    inventory: &AssetInventoryResult,
    code: &str,
    message: &str,
) -> AssetResolutionResult {
    AssetResolutionResult {
        metadata: metadata(assessment, inventory, 0, 0, 0, 0, 0, 1),
        proposals: Vec::new(),
        decisions: Vec::new(),
        warnings: Vec::new(),
        errors: vec![AssetResolutionError {
            error_code: code.to_string(),
            message: message.to_string(),
        }],
    }
}

fn count_decisions(decisions: &[ResolutionDecision], status: &str) -> usize {
    decisions
        .iter()
        .filter(|decision| decision.decision_status == status)
        .count()
}

#[allow(clippy::too_many_arguments)]
fn metadata(
    assessment: &DependencyAssessmentResult,
    inventory: &AssetInventoryResult,
    proposal_count: usize,
    auto: usize,
    manual: usize,
    unresolved: usize,
    warning_count: usize,
    error_count: usize,
) -> AssetResolutionMetadata {
    AssetResolutionMetadata {
        resolution_version: ASSET_RESOLUTION_VERSION.to_string(),
        policy_version: RESOLUTION_POLICY_VERSION.to_string(),
        input_assessment_version: assessment.assessment_metadata.assessment_version.clone(),
        input_inventory_version: inventory.metadata.inventory_version.clone(),
        scan_run_id: inventory.metadata.scan_run_id.clone(),
        required_asset_count: assessment.required_assets.len(),
        proposal_count,
        auto_accepted_count: auto,
        manual_review_count: manual,
        unresolved_count: unresolved,
        warning_count,
        error_count,
    }
}
