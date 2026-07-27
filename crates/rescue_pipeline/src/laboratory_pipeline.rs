use rescue_analyzer::{DependencyAssessmentResult, PreflightReport, ProjectDiscoveryResult};
use rescue_catalog::AssetInventoryResult;
use rescue_core::{ALSReadModel, DependencyExtractionResult, PathObservationResult};
use rescue_execution::StagingExecutionResult;
use rescue_manifest::ManifestWriteResult;
use rescue_packaging::PackagePlan;
use rescue_promotion::PackagePromotionResult;
use rescue_resolution::AssetResolutionResult;
use rescue_rewriter::ALSRewriteResult;
use rescue_validation::PackageValidationResult;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

pub const LABORATORY_PIPELINE_VERSION: &str = "0.1.0";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LaboratoryPackageRequest {
    pub run_id: String,
    pub source_als_path: PathBuf,
    pub scan_roots: Vec<PathBuf>,
    pub max_scan_entries: usize,
    pub staging_root: PathBuf,
    pub target_project_root: PathBuf,
    pub private_ledger_path: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LaboratoryPackageResult {
    pub pipeline_version: String,
    pub run_id: String,
    pub run_status: String,
    pub completed_stage: String,
    pub discovery: Option<ProjectDiscoveryResult>,
    pub als_read_model: Option<ALSReadModel>,
    pub extraction: Option<DependencyExtractionResult>,
    pub path_observations: Option<PathObservationResult>,
    pub assessment: Option<DependencyAssessmentResult>,
    pub preflight: Option<PreflightReport>,
    pub inventory: Option<AssetInventoryResult>,
    pub resolution: Option<AssetResolutionResult>,
    pub package_plan: Option<PackagePlan>,
    pub staging: Option<StagingExecutionResult>,
    pub rewrite: Option<ALSRewriteResult>,
    pub validation: Option<PackageValidationResult>,
    pub manifests: Option<ManifestWriteResult>,
    pub promotion: Option<PackagePromotionResult>,
    pub errors: Vec<LaboratoryPackageError>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LaboratoryPackageError {
    pub error_code: String,
    pub stage: String,
    pub message: String,
}

pub fn run_laboratory_package(request: &LaboratoryPackageRequest) -> LaboratoryPackageResult {
    crate::laboratory_pipeline_impl::run_laboratory_package_impl(request)
}
