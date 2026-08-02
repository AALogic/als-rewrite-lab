use rescue_execution::StagingExecutionResult;
use rescue_manifest::ManifestWriteResult;
use rescue_packaging::PackagePlan;
use rescue_rewriter::ALSRewriteResult;
use rescue_validation::PackageValidationResult;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

pub const PACKAGE_PROMOTER_VERSION: &str = "0.2.0";
pub const PACKAGE_PROMOTION_SCHEMA_VERSION: &str = "0.2";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PackagePromotionRequest {
    pub promotion_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PackagePromotionResult {
    pub metadata: PackagePromotionMetadata,
    pub former_staging_root: PathBuf,
    pub final_target_root: PathBuf,
    pub promoted_files: Vec<PromotedFileRecord>,
    pub promotion_status: String,
    pub manual_check_status: String,
    pub errors: Vec<PackagePromotionError>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PackagePromotionMetadata {
    pub promoter_version: String,
    pub promotion_schema_version: String,
    pub promotion_id: String,
    pub plan_id: String,
    pub execution_id: String,
    pub rewrite_id: String,
    pub validation_id: String,
    pub manifest_id: String,
    pub verified_file_count: usize,
    pub error_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PromotedFileRecord {
    pub relative_path: PathBuf,
    pub expected_sha256: Option<String>,
    pub observed_sha256: Option<String>,
    pub expected_size: u64,
    pub observed_size: Option<u64>,
    pub verification_method: String,
    pub file_status: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PackagePromotionError {
    pub error_code: String,
    pub message: String,
    pub path: Option<PathBuf>,
}

pub fn promote_validated_package(
    request: &PackagePromotionRequest,
    plan: &PackagePlan,
    staging: &StagingExecutionResult,
    rewrite: &ALSRewriteResult,
    validation: &PackageValidationResult,
    manifests: &ManifestWriteResult,
) -> PackagePromotionResult {
    crate::package_promoter_impl::promote_validated_package_impl(
        request, plan, staging, rewrite, validation, manifests,
    )
}
