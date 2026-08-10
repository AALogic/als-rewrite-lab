use crate::package_planner_operations::{COMPATIBILITY_LAB_RULE_ID, LAB_RULE_ID};
use crate::{
    CopyOperation, CreateDirectoryOperation, PackagePlan, PackagePlanError, PackagePlanMetadata,
    PackagePlanningRequest, PlannedSourceAls, RewriteOperation, SystemDependencyRequirement,
    UnresolvedPackageRequirement, PACKAGE_PLANNER_VERSION, PACKAGE_PLAN_SCHEMA_VERSION,
};
use rescue_core::ALSReadModel;
use std::path::{Path, PathBuf};

#[allow(clippy::too_many_arguments)]
pub(crate) fn build_plan(
    request: &PackagePlanningRequest,
    model: &ALSReadModel,
    policy_version: &str,
    required_asset_count: usize,
    source_als: PlannedSourceAls,
    directories: Vec<CreateDirectoryOperation>,
    copies: Vec<CopyOperation>,
    rewrites: Vec<RewriteOperation>,
    system_dependencies: Vec<SystemDependencyRequirement>,
    unresolved: Vec<UnresolvedPackageRequirement>,
    mut errors: Vec<PackagePlanError>,
) -> PackagePlan {
    if request.planning_mode == "laboratory_rescue_rewrite" && rewrites.is_empty() {
        errors.push(PackagePlanError {
            error_code: "PACKAGE_REWRITE_OPERATIONS_EMPTY".to_string(),
            message: "Laboratory execution requires at least one approved rewrite operation"
                .to_string(),
            path: None,
        });
    }
    let has_blocking_omission = unresolved.iter().any(|item| item.blocks_execution);
    let status = if !errors.is_empty() || has_blocking_omission {
        "blocked"
    } else if request.planning_mode == "copy_only" {
        "ready_copy_only"
    } else if crate::is_current_paths_mode(&request.planning_mode) && unresolved.is_empty() {
        "ready_current_paths_complete"
    } else if crate::is_current_paths_mode(&request.planning_mode) {
        "ready_current_paths_incomplete"
    } else {
        "ready_for_laboratory_execution"
    };
    PackagePlan {
        metadata: metadata(
            request,
            model,
            policy_version,
            required_asset_count,
            directories.len(),
            copies.len(),
            rewrites.len(),
            system_dependencies.len(),
            unresolved.len(),
            0,
            errors.len(),
        ),
        source_als,
        target_project_root: request.target_project_root.clone(),
        directory_operations: directories,
        copy_operations: copies,
        rewrite_operations: rewrites,
        system_dependencies,
        unresolved_requirements: unresolved,
        plan_status: status.to_string(),
        warnings: Vec::new(),
        errors,
    }
}

pub(crate) fn fatal_plan(
    request: &PackagePlanningRequest,
    model: &ALSReadModel,
    policy_version: &str,
    required_asset_count: usize,
    source_als: PlannedSourceAls,
    errors: Vec<PackagePlanError>,
) -> PackagePlan {
    PackagePlan {
        metadata: metadata(
            request,
            model,
            policy_version,
            required_asset_count,
            0,
            0,
            0,
            0,
            0,
            0,
            errors.len(),
        ),
        source_als,
        target_project_root: request.target_project_root.clone(),
        directory_operations: Vec::new(),
        copy_operations: Vec::new(),
        rewrite_operations: Vec::new(),
        system_dependencies: Vec::new(),
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
    policy_version: &str,
    required_asset_count: usize,
    directory_count: usize,
    copy_count: usize,
    rewrite_count: usize,
    system_dependency_count: usize,
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
        resolution_policy_version: policy_version.to_string(),
        rewrite_ruleset_version: if crate::is_compatibility_lab_mode(&request.planning_mode) {
            COMPATIBILITY_LAB_RULE_ID.to_string()
        } else if matches!(
            request.planning_mode.as_str(),
            "laboratory_rescue_rewrite" | "current_paths_copy"
        ) {
            LAB_RULE_ID.to_string()
        } else {
            "none".to_string()
        },
        required_asset_count,
        directory_operation_count: directory_count,
        copy_operation_count: copy_count,
        rewrite_operation_count: rewrite_count,
        system_dependency_count,
        unresolved_count,
        warning_count,
        error_count,
    }
}

pub(crate) fn planned_source_als(model: &ALSReadModel) -> PlannedSourceAls {
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

pub(crate) fn package_error(code: &str, message: &str, path: Option<&Path>) -> PackagePlanError {
    PackagePlanError {
        error_code: code.to_string(),
        message: message.to_string(),
        path: path.map(Path::to_path_buf),
    }
}
