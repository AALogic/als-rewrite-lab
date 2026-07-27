use crate::package_planner_operations::LAB_RULE_ID;
use crate::{
    CopyOperation, PackagePlan, PackagePlanError, PackagePlanMetadata, PackagePlanningRequest,
    PlannedSourceAls, RewriteOperation, UnresolvedPackageRequirement, PACKAGE_PLANNER_VERSION,
    PACKAGE_PLAN_SCHEMA_VERSION,
};
use rescue_core::ALSReadModel;
use rescue_resolution::AssetResolutionResult;

#[allow(clippy::too_many_arguments)]
pub(crate) fn build_plan(
    request: &PackagePlanningRequest,
    model: &ALSReadModel,
    resolution: &AssetResolutionResult,
    source_als: PlannedSourceAls,
    copies: Vec<CopyOperation>,
    rewrites: Vec<RewriteOperation>,
    unresolved: Vec<UnresolvedPackageRequirement>,
    errors: Vec<PackagePlanError>,
) -> PackagePlan {
    let status = if !errors.is_empty() || !unresolved.is_empty() {
        "blocked"
    } else if request.planning_mode == "copy_only" {
        "ready_copy_only"
    } else {
        "ready_for_laboratory_execution"
    };
    PackagePlan {
        metadata: metadata(
            request,
            model,
            resolution,
            copies.len(),
            rewrites.len(),
            unresolved.len(),
            0,
            errors.len(),
        ),
        source_als,
        target_project_root: request.target_project_root.clone(),
        copy_operations: copies,
        rewrite_operations: rewrites,
        unresolved_requirements: unresolved,
        plan_status: status.to_string(),
        warnings: Vec::new(),
        errors,
    }
}

pub(crate) fn fatal_plan(
    request: &PackagePlanningRequest,
    model: &ALSReadModel,
    resolution: &AssetResolutionResult,
    source_als: PlannedSourceAls,
    errors: Vec<PackagePlanError>,
) -> PackagePlan {
    PackagePlan {
        metadata: metadata(request, model, resolution, 0, 0, 0, 0, errors.len()),
        source_als,
        target_project_root: request.target_project_root.clone(),
        copy_operations: Vec::new(),
        rewrite_operations: Vec::new(),
        unresolved_requirements: Vec::new(),
        plan_status: "blocked".to_string(),
        warnings: Vec::new(),
        errors,
    }
}

#[allow(clippy::too_many_arguments)]
fn metadata(
    request: &PackagePlanningRequest,
    model: &ALSReadModel,
    resolution: &AssetResolutionResult,
    copy_count: usize,
    rewrite_count: usize,
    unresolved_count: usize,
    warning_count: usize,
    error_count: usize,
) -> PackagePlanMetadata {
    PackagePlanMetadata {
        planner_version: PACKAGE_PLANNER_VERSION.to_string(),
        plan_schema_version: PACKAGE_PLAN_SCHEMA_VERSION.to_string(),
        plan_id: request.plan_id.clone(),
        planning_mode: request.planning_mode.clone(),
        source_als_hash: model.set_metadata.source_file_hash.clone(),
        resolution_policy_version: resolution.metadata.policy_version.clone(),
        rewrite_ruleset_version: if request.planning_mode == "laboratory_rescue_rewrite" {
            LAB_RULE_ID.to_string()
        } else {
            "none".to_string()
        },
        required_asset_count: resolution.metadata.required_asset_count,
        copy_operation_count: copy_count,
        rewrite_operation_count: rewrite_count,
        unresolved_count,
        warning_count,
        error_count,
    }
}
