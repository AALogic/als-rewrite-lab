use crate::package_planner_operations::{build_operations, OperationBuild, SelectedAsset};
use crate::package_planner_result::{build_plan, fatal_plan};
use crate::{
    PackagePlan, PackagePlanError, PackagePlanningRequest, PlannedSourceAls,
    UnresolvedPackageRequirement,
};
use rescue_analyzer::DependencyAssessmentResult;
use rescue_catalog::{AssetInventoryResult, FileOccurrence};
use rescue_core::ALSReadModel;
use rescue_resolution::{AssetResolutionResult, ResolutionDecision, RESOLUTION_POLICY_VERSION};
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Component, Path, PathBuf};

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
        return fatal_plan(request, model, resolution, source_als, validation_errors);
    }
    let (selected, mut unresolved) = selected_assets(assessment, inventory, &resolution.decisions);
    let OperationBuild {
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
        request, model, resolution, source_als, copies, rewrites, unresolved, errors,
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
        "copy_only" | "laboratory_rescue_rewrite"
    ) {
        errors.push(error(
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
        errors.push(error(
            "PACKAGE_UNTRUSTED_INPUT",
            "At least one producer contains fatal errors",
            None,
        ));
    }
    if model.set_metadata.source_file_hash != assessment.assessment_metadata.source_file_hash {
        errors.push(error(
            "PACKAGE_SNAPSHOT_MISMATCH",
            "ALS model and assessment describe different snapshots",
            None,
        ));
    }
    if inventory.metadata.scan_run_id != resolution.metadata.scan_run_id {
        errors.push(error(
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
        errors.push(error(
            "PACKAGE_RESOLUTION_POLICY_UNSUPPORTED",
            "Resolution decisions do not use the required safety policy",
            None,
        ));
    }
    if request.planning_mode == "laboratory_rescue_rewrite" && !supported_lab_document(model) {
        errors.push(error(
            "PACKAGE_REWRITE_DOCUMENT_UNSUPPORTED",
            "ALS document is outside the E-03 laboratory support profile",
            None,
        ));
    }
    validate_decision_ids(assessment, resolution, &mut errors);
    errors
}

fn validate_paths(
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
        errors.push(error(
            "PACKAGE_TARGET_ROOT_UNSAFE",
            "Target Project root must be absolute without parent traversal",
            Some(&request.target_project_root),
        ));
    }
    if source_als.target_relative_path.as_os_str().is_empty()
        || source_als.target_relative_path.components().count() != 1
    {
        errors.push(error(
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
        errors.push(error(
            "PACKAGE_TARGET_EQUALS_SOURCE",
            "Target ALS path equals the original ALS path",
            Some(&source_als.source_als_path),
        ));
    }
}

fn supported_lab_document(model: &ALSReadModel) -> bool {
    model.set_metadata.ableton_document_version.as_deref() == Some("5")
        && model.set_metadata.ableton_minor_version.as_deref() == Some("11.0_11300")
        && model
            .set_metadata
            .ableton_creator_version
            .as_deref()
            .is_some_and(|value| value.starts_with("Ableton Live 11.3."))
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
        errors.push(error(
            "PACKAGE_DECISION_CONTRACT_MISMATCH",
            "Resolution decisions are not one-to-one with required assets",
            None,
        ));
    }
}

fn selected_assets<'a>(
    assessment: &'a DependencyAssessmentResult,
    inventory: &'a AssetInventoryResult,
    decisions: &[ResolutionDecision],
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
        let Some(decision) = decisions_by_asset.get(asset.required_asset_id.as_str()) else {
            continue;
        };
        let occurrence = accepted_occurrence(decision, &occurrences);
        if let Some(occurrence) = occurrence {
            selected.push(SelectedAsset { asset, occurrence });
        } else {
            unresolved.push(UnresolvedPackageRequirement {
                required_asset_id: asset.required_asset_id.clone(),
                decision_status: decision.decision_status.clone(),
                reason: "resolution_not_auto_accepted".to_string(),
            });
        }
    }
    (selected, unresolved)
}

fn accepted_occurrence<'a>(
    decision: &ResolutionDecision,
    occurrences: &BTreeMap<&str, &'a FileOccurrence>,
) -> Option<&'a FileOccurrence> {
    if decision.decision_status != "auto_accepted" {
        return None;
    }
    let occurrence_id = decision.selected_file_occurrence_id.as_deref()?;
    let occurrence = occurrences.get(occurrence_id).copied()?;
    (decision.selected_content_id.as_deref() == Some(occurrence.content_id.as_str()))
        .then_some(occurrence)
}

fn planned_source_als(model: &ALSReadModel) -> PlannedSourceAls {
    PlannedSourceAls {
        source_als_path: PathBuf::from(&model.set_metadata.source_als_path),
        source_file_hash: model.set_metadata.source_file_hash.clone(),
        source_file_size: model.set_metadata.source_file_size,
        target_relative_path: model
            .set_metadata
            .source_als_filename
            .as_deref()
            .map(PathBuf::from)
            .unwrap_or_default(),
        ableton_document_version: model.set_metadata.ableton_document_version.clone(),
        ableton_creator_version: model.set_metadata.ableton_creator_version.clone(),
        ableton_minor_version: model.set_metadata.ableton_minor_version.clone(),
    }
}

fn error(code: &str, message: &str, path: Option<&Path>) -> PackagePlanError {
    PackagePlanError {
        error_code: code.to_string(),
        message: message.to_string(),
        path: path.map(Path::to_path_buf),
    }
}
