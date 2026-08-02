use rescue_analyzer::DependencyAssessmentResult;
use rescue_catalog::AssetInventoryResult;
use rescue_core::ALSReadModel;
use rescue_resolution::AssetResolutionResult;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

pub const PACKAGE_PLANNER_VERSION: &str = "0.5.0";
pub const PACKAGE_PLAN_SCHEMA_VERSION: &str = "0.5";

pub const VERIFY_SHA256_AND_SIZE: &str = "sha256_and_size";
pub const VERIFY_STABLE_SOURCE_AND_SIZE: &str = "stable_source_and_size";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PackagePlanningRequest {
    pub plan_id: String,
    pub target_project_root: PathBuf,
    pub planning_mode: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PackagePlan {
    pub metadata: PackagePlanMetadata,
    pub source_als: PlannedSourceAls,
    pub target_project_root: PathBuf,
    pub directory_operations: Vec<CreateDirectoryOperation>,
    pub copy_operations: Vec<CopyOperation>,
    pub rewrite_operations: Vec<RewriteOperation>,
    pub system_dependencies: Vec<SystemDependencyRequirement>,
    pub unresolved_requirements: Vec<UnresolvedPackageRequirement>,
    pub plan_status: String,
    pub warnings: Vec<PackagePlanWarning>,
    pub errors: Vec<PackagePlanError>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PackagePlanMetadata {
    pub planner_version: String,
    pub plan_schema_version: String,
    pub plan_id: String,
    pub planning_mode: String,
    pub source_als_hash: String,
    pub resolution_policy_version: String,
    pub rewrite_ruleset_version: String,
    pub required_asset_count: usize,
    pub directory_operation_count: usize,
    pub copy_operation_count: usize,
    pub rewrite_operation_count: usize,
    pub system_dependency_count: usize,
    pub unresolved_count: usize,
    pub warning_count: usize,
    pub error_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlannedSourceAls {
    pub source_als_path: PathBuf,
    pub source_file_hash: String,
    pub source_file_size: u64,
    pub target_relative_path: PathBuf,
    pub ableton_document_version: Option<String>,
    pub ableton_creator_version: Option<String>,
    pub ableton_minor_version: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CreateDirectoryOperation {
    pub operation_id: String,
    pub target_relative_path: PathBuf,
    pub purpose: String,
    pub collision_policy: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CopyOperation {
    pub operation_id: String,
    pub operation_kind: String,
    pub source_path: PathBuf,
    pub target_relative_path: PathBuf,
    pub expected_source_sha256: Option<String>,
    pub expected_source_size: u64,
    pub content_id: Option<String>,
    pub source_binding_id: String,
    pub verification_policy: String,
    pub collision_policy: String,
    pub preconditions: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RewriteOperation {
    pub operation_id: String,
    pub required_asset_id: String,
    pub dependency_id: String,
    pub als_ref_id: usize,
    pub xml_locator: String,
    pub source_als_hash: String,
    pub old_path: Option<String>,
    pub old_relative_path: Option<String>,
    pub old_relative_path_type: Option<String>,
    pub new_path: String,
    pub new_relative_path: String,
    pub new_relative_path_type: String,
    pub fields_to_change: Vec<String>,
    pub rule_id: String,
    pub support_status: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SystemDependencyRequirement {
    pub required_asset_id: String,
    pub source_category: String,
    pub filename: Option<String>,
    pub occurrence_count: usize,
    pub als_ref_ids: Vec<usize>,
    pub observed_source_paths: Vec<PathBuf>,
    pub package_action: String,
    pub portability_status: String,
    pub reason: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UnresolvedPackageRequirement {
    pub required_asset_id: String,
    pub decision_status: String,
    pub reason: String,
    pub blocks_execution: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PackagePlanWarning {
    pub warning_id: usize,
    pub warning_code: String,
    pub message: String,
    pub required_asset_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PackagePlanError {
    pub error_code: String,
    pub message: String,
    pub path: Option<PathBuf>,
}

pub fn plan_package(
    request: &PackagePlanningRequest,
    als_model: &ALSReadModel,
    assessment: &DependencyAssessmentResult,
    inventory: &AssetInventoryResult,
    resolution: &AssetResolutionResult,
) -> PackagePlan {
    crate::package_planner_impl::plan_package_impl(
        request, als_model, assessment, inventory, resolution,
    )
}

pub fn plan_current_path_package(
    request: &PackagePlanningRequest,
    als_model: &ALSReadModel,
    assessment: &DependencyAssessmentResult,
    bindings: &rescue_resolution::CurrentPathBindingResult,
) -> PackagePlan {
    crate::package_planner_current_path::plan_current_path_package_impl(
        request, als_model, assessment, bindings,
    )
}
