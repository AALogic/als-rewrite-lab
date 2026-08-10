use crate::package_planner_operations::{build_operations, OperationBuild};
use crate::package_planner_result::{build_plan, fatal_plan, package_error, planned_source_als};
use crate::{PackagePlan, PackagePlanError, PackagePlanningRequest, PlannedSourceAls};
use rescue_analyzer::DependencyAssessmentResult;
use rescue_catalog::AssetInventoryResult;
use rescue_core::ALSReadModel;
use rescue_resolution::{AssetResolutionResult, RESOLUTION_POLICY_VERSION};
use std::collections::BTreeSet;
use std::path::Component;

pub(crate) fn plan_package_impl(
    request: &PackagePlanningRequest,
    model: &ALSReadModel,
    assessment: &DependencyAssessmentResult,
    inventory: &AssetInventoryResult,
    resolution: &AssetResolutionResult,
) -> PackagePlan {
    let source_als = planned_source_als(model);
    let validation_errors = validate_inputs(
        request,
        model,
        assessment,
        inventory,
        resolution,
        &source_als,
    );
    if !validation_errors.is_empty() {
        return fatal_plan(
            request,
            model,
            &resolution.metadata.policy_version,
            resolution.metadata.required_asset_count,
            source_als,
            validation_errors,
        );
    }
    let (selected, mut unresolved) = crate::package_planner_selection::selected_assets(
        assessment,
        inventory,
        &resolution.decisions,
        &request.planning_mode,
    );
    let system_dependencies =
        crate::package_planner_system_dependencies::planned_system_dependencies(assessment);
    let OperationBuild {
        directories,
        copies,
        rewrites,
        unresolved: operation_unresolved,
        errors,
    } = build_operations(
        &request.planning_mode,
        &request.target_project_root,
        model,
        &selected,
    );
    unresolved.extend(operation_unresolved);
    build_plan(
        request,
        model,
        &resolution.metadata.policy_version,
        resolution.metadata.required_asset_count,
        source_als,
        directories,
        copies,
        rewrites,
        system_dependencies,
        unresolved,
        errors,
    )
}

fn validate_inputs(
    request: &PackagePlanningRequest,
    model: &ALSReadModel,
    assessment: &DependencyAssessmentResult,
    inventory: &AssetInventoryResult,
    resolution: &AssetResolutionResult,
    source_als: &PlannedSourceAls,
) -> Vec<PackagePlanError> {
    let mut errors = Vec::new();
    if !matches!(
        request.planning_mode.as_str(),
        "copy_only"
            | "laboratory_rescue_rewrite"
            | "current_paths_copy"
            | "compatibility_lab_current_paths_copy"
    ) {
        errors.push(package_error(
            "PACKAGE_MODE_UNSUPPORTED",
            "Planning mode is unsupported",
            None,
        ));
    }
    validate_paths(request, source_als, &mut errors);
    if !model.errors.is_empty()
        || !assessment.errors.is_empty()
        || !inventory.errors.is_empty()
        || !resolution.errors.is_empty()
    {
        errors.push(package_error(
            "PACKAGE_UNTRUSTED_INPUT",
            "At least one producer contains fatal errors",
            None,
        ));
    }
    if model.set_metadata.source_file_hash != assessment.assessment_metadata.source_file_hash {
        errors.push(package_error(
            "PACKAGE_SNAPSHOT_MISMATCH",
            "ALS model and assessment describe different snapshots",
            None,
        ));
    }
    if inventory.metadata.scan_run_id != resolution.metadata.scan_run_id {
        errors.push(package_error(
            "PACKAGE_INVENTORY_RESOLUTION_MISMATCH",
            "Resolution does not belong to the supplied inventory",
            None,
        ));
    }
    if resolution.metadata.policy_version != RESOLUTION_POLICY_VERSION
        || resolution
            .decisions
            .iter()
            .any(|decision| decision.policy_version != RESOLUTION_POLICY_VERSION)
    {
        errors.push(package_error(
            "PACKAGE_RESOLUTION_POLICY_UNSUPPORTED",
            "Resolution decisions do not use the required safety policy",
            None,
        ));
    }
    let strict_rewrite_mode = matches!(
        request.planning_mode.as_str(),
        "laboratory_rescue_rewrite" | "current_paths_copy"
    );
    if strict_rewrite_mode && !supported_lab_document(model) {
        errors.push(package_error(
            "PACKAGE_REWRITE_DOCUMENT_UNSUPPORTED",
            "ALS document is outside the E-03 laboratory support profile",
            None,
        ));
    }
    validate_decision_ids(assessment, resolution, &mut errors);
    errors
}

pub(crate) fn validate_paths(
    request: &PackagePlanningRequest,
    source_als: &PlannedSourceAls,
    errors: &mut Vec<PackagePlanError>,
) {
    if !request.target_project_root.is_absolute()
        || request
            .target_project_root
            .components()
            .any(|component| matches!(component, Component::ParentDir))
    {
        errors.push(package_error(
            "PACKAGE_TARGET_ROOT_UNSAFE",
            "Target Project root must be absolute without parent traversal",
            Some(&request.target_project_root),
        ));
    }
    if source_als.target_relative_path.as_os_str().is_empty()
        || source_als.target_relative_path.components().count() != 1
    {
        errors.push(package_error(
            "PACKAGE_SOURCE_ALS_FILENAME_UNSAFE",
            "Source ALS filename cannot be used as a target filename",
            Some(&source_als.target_relative_path),
        ));
    }
    if request
        .target_project_root
        .join(&source_als.target_relative_path)
        == source_als.source_als_path
    {
        errors.push(package_error(
            "PACKAGE_TARGET_EQUALS_SOURCE",
            "Target ALS path equals the original ALS path",
            Some(&source_als.source_als_path),
        ));
    }
}

pub(crate) fn supported_lab_document(model: &ALSReadModel) -> bool {
    rescue_core::is_confirmed_live_11_3_document(model)
}

fn validate_decision_ids(
    assessment: &DependencyAssessmentResult,
    resolution: &AssetResolutionResult,
    errors: &mut Vec<PackagePlanError>,
) {
    let asset_ids: BTreeSet<_> = assessment
        .required_assets
        .iter()
        .map(|asset| asset.required_asset_id.as_str())
        .collect();
    let decision_ids: BTreeSet<_> = resolution
        .decisions
        .iter()
        .map(|decision| decision.required_asset_id.as_str())
        .collect();
    if asset_ids.len() != assessment.required_assets.len()
        || decision_ids.len() != resolution.decisions.len()
        || asset_ids != decision_ids
    {
        errors.push(package_error(
            "PACKAGE_DECISION_CONTRACT_MISMATCH",
            "Resolution decisions are not one-to-one with required assets",
            None,
        ));
    }
}
