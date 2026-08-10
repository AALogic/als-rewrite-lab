use crate::LaboratoryPackageError;
use rescue_packaging::PackagePlan;
pub use rescue_packaging::PlanFingerprint;
use rescue_promotion::PackagePromotionResult;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

pub const CURRENT_PATH_COPY_PIPELINE_VERSION: &str = "0.6.0";
pub const STRICT_REWRITE_POLICY: &str = "strict_confirmed_profile";
pub const COMPATIBILITY_LAB_REWRITE_POLICY: &str = "compatibility_lab_known_shapes";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CurrentPathCopyRequest {
    pub run_id: String,
    pub rewrite_policy: String,
    pub source_als_path: PathBuf,
    pub expected_source_als_sha256: Option<String>,
    pub expected_plan_fingerprint: Option<PlanFingerprint>,
    pub staging_root: PathBuf,
    pub target_project_root: PathBuf,
    pub private_ledger_path: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CurrentPathCopyResult {
    pub pipeline_version: String,
    pub run_id: String,
    pub rewrite_policy: String,
    pub run_status: String,
    pub completed_stage: String,
    pub required_asset_count: usize,
    pub system_dependency_count: usize,
    pub copied_asset_count: usize,
    pub rewritten_reference_count: usize,
    pub omitted_asset_count: usize,
    pub plan_fingerprint: Option<PlanFingerprint>,
    pub package_plan: Option<PackagePlan>,
    pub promotion: Option<PackagePromotionResult>,
    pub errors: Vec<LaboratoryPackageError>,
}

pub fn prepare_current_path_copy(request: &CurrentPathCopyRequest) -> CurrentPathCopyResult {
    crate::current_path_copy_impl::prepare(request)
}

pub fn run_current_path_copy(request: &CurrentPathCopyRequest) -> CurrentPathCopyResult {
    crate::current_path_copy_impl::run(request)
}
