use rescue_execution::StagingExecutionResult;
use rescue_packaging::PackagePlan;
use rescue_rewriter::ALSRewriteResult;
use rescue_validation::PackageValidationResult;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

pub const MANIFEST_WRITER_VERSION: &str = "0.4.0";
pub const PRIVATE_LEDGER_SCHEMA_VERSION: &str = "0.4";
pub const PACKAGE_MANIFEST_SCHEMA_VERSION: &str = "0.4";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ManifestWriteRequest {
    pub manifest_id: String,
    pub private_ledger_path: PathBuf,
    pub package_manifest_relative_path: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PrivateLedger {
    pub ledger_schema_version: String,
    pub ledger_id: String,
    pub plan: PackagePlan,
    pub staging: StagingExecutionResult,
    pub rewrite: ALSRewriteResult,
    pub validation: PackageValidationResult,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PackageManifest {
    pub manifest_schema_version: String,
    pub manifest_id: String,
    pub plan_id: String,
    pub source_set_filename: String,
    pub source_als_sha256: String,
    pub rewrite_ruleset_version: String,
    pub package_status: String,
    pub directories: Vec<ManifestDirectory>,
    pub files: Vec<ManifestFile>,
    pub rewrites: Vec<ManifestRewrite>,
    pub system_dependencies: Vec<ManifestSystemDependency>,
    pub omissions: Vec<ManifestOmission>,
    pub validation: ManifestValidationSummary,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ManifestDirectory {
    pub relative_path: PathBuf,
    pub purpose: String,
    pub status: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ManifestFile {
    pub role: String,
    pub relative_path: PathBuf,
    pub sha256: Option<String>,
    pub size: u64,
    pub verification_method: String,
    pub content_identity_status: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ManifestRewrite {
    pub operation_id: String,
    pub xml_locator: String,
    pub changed_fields: Vec<String>,
    pub ruleset_version: String,
    pub status: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ManifestSystemDependency {
    pub required_asset_id: String,
    pub source_category: String,
    pub filename: Option<String>,
    pub occurrence_count: usize,
    pub package_action: String,
    pub portability_status: String,
    pub required_environment: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ManifestOmission {
    pub required_asset_id: String,
    pub reason: String,
    pub reference_status: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ManifestValidationSummary {
    pub validator_version: String,
    pub validation_id: String,
    pub validation_status: String,
    pub verified_directory_count: usize,
    pub verified_file_count: usize,
    pub verified_rewrite_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ManifestWriteResult {
    pub metadata: ManifestWriteMetadata,
    pub private_ledger_path: PathBuf,
    pub package_manifest_relative_path: PathBuf,
    pub package_manifest_sha256: Option<String>,
    pub package_manifest_size: Option<u64>,
    pub private_ledger_sha256: Option<String>,
    pub private_ledger_size: Option<u64>,
    pub write_records: Vec<ManifestWriteRecord>,
    pub write_status: String,
    pub errors: Vec<ManifestWriteError>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ManifestWriteMetadata {
    pub writer_version: String,
    pub manifest_id: String,
    pub plan_id: String,
    pub execution_id: String,
    pub rewrite_id: String,
    pub validation_id: String,
    pub error_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ManifestWriteRecord {
    pub artifact_kind: String,
    pub path: PathBuf,
    pub sha256: Option<String>,
    pub size: Option<u64>,
    pub write_status: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ManifestWriteError {
    pub error_code: String,
    pub message: String,
    pub artifact_kind: Option<String>,
    pub path: Option<PathBuf>,
}

pub fn write_package_evidence(
    request: &ManifestWriteRequest,
    plan: &PackagePlan,
    staging: &StagingExecutionResult,
    rewrite: &ALSRewriteResult,
    validation: &PackageValidationResult,
) -> ManifestWriteResult {
    crate::manifest_writer_impl::write_package_evidence_impl(
        request, plan, staging, rewrite, validation,
    )
}
