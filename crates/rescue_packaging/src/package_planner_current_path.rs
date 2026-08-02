use crate::package_planner_operations::{build_operations, OperationBuild, SelectedAsset};
use crate::package_planner_result::{build_plan, fatal_plan, package_error, planned_source_als};
use crate::{
    PackagePlan, PackagePlanError, PackagePlanningRequest, UnresolvedPackageRequirement,
    VERIFY_STABLE_SOURCE_AND_SIZE,
};
use rescue_analyzer::DependencyAssessmentResult;
use rescue_core::ALSReadModel;
use rescue_resolution::{CurrentPathBindingResult, CURRENT_PATH_BINDING_POLICY_VERSION};
use std::collections::{BTreeMap, BTreeSet};

pub(crate) fn plan_current_path_package_impl(
    request: &PackagePlanningRequest,
    model: &ALSReadModel,
    assessment: &DependencyAssessmentResult,
    bindings: &CurrentPathBindingResult,
) -> PackagePlan {
    let source_als = planned_source_als(model);
    let errors = validate_inputs(request, model, assessment, bindings, &source_als);
    if !errors.is_empty() {
        return fatal_plan(
            request,
            model,
            &bindings.metadata.policy_version,
            assessment.required_assets.len(),
            source_als,
            errors,
        );
    }

    let binding_by_asset: BTreeMap<_, _> = bindings
        .bindings
        .iter()
        .map(|binding| (binding.required_asset_id.as_str(), binding))
        .collect();
    let system_dependencies =
        crate::package_planner_system_dependencies::planned_system_dependencies(assessment);
    let selected: Vec<_> = assessment
        .required_assets
        .iter()
        .filter(|asset| !asset.is_confirmed_system_dependency())
        .filter_map(|asset| {
            let binding = binding_by_asset.get(asset.required_asset_id.as_str())?;
            Some(SelectedAsset {
                asset,
                source_path: binding.source_path.clone(),
                filename: binding.filename.clone(),
                expected_size: binding.observed_size,
                source_binding_id: binding.candidate_id.clone(),
                dedup_key: format!("path:{}", binding.source_path.to_string_lossy()),
                expected_sha256: None,
                content_id: None,
                verification_policy: VERIFY_STABLE_SOURCE_AND_SIZE.to_string(),
            })
        })
        .collect();
    let mut unresolved: Vec<_> = bindings
        .omissions
        .iter()
        .map(|omission| UnresolvedPackageRequirement {
            required_asset_id: omission.required_asset_id.clone(),
            decision_status: if omission.blocks_execution {
                "blocked"
            } else {
                "unresolved"
            }
            .to_string(),
            reason: omission.reason.clone(),
            blocks_execution: omission.blocks_execution,
        })
        .collect();
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
        &bindings.metadata.policy_version,
        assessment.required_assets.len(),
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
    bindings: &CurrentPathBindingResult,
    source_als: &crate::PlannedSourceAls,
) -> Vec<PackagePlanError> {
    let mut errors = Vec::new();
    if request.planning_mode != "current_paths_copy" {
        errors.push(package_error(
            "PACKAGE_MODE_UNSUPPORTED",
            "Metadata-only current-path planning requires current_paths_copy mode",
            None,
        ));
    }
    crate::package_planner_impl::validate_paths(request, source_als, &mut errors);
    if !model.errors.is_empty() || !assessment.errors.is_empty() || !bindings.errors.is_empty() {
        errors.push(package_error(
            "PACKAGE_UNTRUSTED_INPUT",
            "At least one current-path producer contains fatal errors",
            None,
        ));
    }
    if model.set_metadata.source_file_hash != assessment.assessment_metadata.source_file_hash
        || model.set_metadata.source_file_hash != bindings.metadata.source_file_hash
    {
        errors.push(package_error(
            "PACKAGE_SNAPSHOT_MISMATCH",
            "ALS model, assessment and current-path bindings describe different snapshots",
            None,
        ));
    }
    if bindings.metadata.policy_version != CURRENT_PATH_BINDING_POLICY_VERSION {
        errors.push(package_error(
            "PACKAGE_CURRENT_PATH_POLICY_UNSUPPORTED",
            "Current-path bindings do not use the required policy",
            None,
        ));
    }
    if !crate::package_planner_impl::supported_lab_document(model) {
        errors.push(package_error(
            "PACKAGE_REWRITE_DOCUMENT_UNSUPPORTED",
            "ALS document is outside the E-03 laboratory support profile",
            None,
        ));
    }
    validate_binding_ids(assessment, bindings, &mut errors);
    if bindings
        .bindings
        .iter()
        .any(|binding| !binding.source_path.is_absolute())
    {
        errors.push(package_error(
            "PACKAGE_CURRENT_PATH_SOURCE_UNSAFE",
            "Every current-path source must be absolute",
            None,
        ));
    }
    errors
}

fn validate_binding_ids(
    assessment: &DependencyAssessmentResult,
    bindings: &CurrentPathBindingResult,
    errors: &mut Vec<PackagePlanError>,
) {
    let required: BTreeSet<_> = assessment
        .required_assets
        .iter()
        .map(|asset| asset.required_asset_id.as_str())
        .collect();
    let mut covered = BTreeSet::new();
    let mut duplicate = false;
    for id in bindings
        .bindings
        .iter()
        .map(|item| item.required_asset_id.as_str())
        .chain(
            bindings
                .omissions
                .iter()
                .map(|item| item.required_asset_id.as_str()),
        )
    {
        duplicate |= !covered.insert(id);
    }
    if duplicate || required.len() != assessment.required_assets.len() || required != covered {
        errors.push(package_error(
            "PACKAGE_CURRENT_PATH_CONTRACT_MISMATCH",
            "Bindings and omissions must cover each required asset exactly once",
            None,
        ));
    }
}
