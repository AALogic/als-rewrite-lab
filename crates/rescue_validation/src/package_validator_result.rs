use crate::{
    DirectoryValidationRecord, FileValidationRecord, PackageValidationError,
    PackageValidationMetadata, PackageValidationRequest, PackageValidationResult,
    SemanticDiffRecord, PACKAGE_VALIDATION_SCHEMA_VERSION, PACKAGE_VALIDATOR_VERSION,
};
use rescue_execution::StagingExecutionResult;
use rescue_packaging::PackagePlan;
use rescue_rewriter::ALSRewriteResult;
use std::path::{Path, PathBuf};

pub(crate) struct ValidationRecords {
    pub directories: Vec<DirectoryValidationRecord>,
    pub files: Vec<FileValidationRecord>,
    pub semantic: Vec<SemanticDiffRecord>,
}

pub(crate) fn result(
    request: &PackageValidationRequest,
    plan: &PackagePlan,
    staging: &StagingExecutionResult,
    rewrite: &ALSRewriteResult,
    records: ValidationRecords,
    errors: Vec<PackageValidationError>,
) -> PackageValidationResult {
    let files_verified = records
        .files
        .iter()
        .filter(|record| record.file_status == "verified")
        .count();
    let directories_verified = records
        .directories
        .iter()
        .filter(|record| record.directory_status == "verified")
        .count();
    let rewrites_verified = records
        .semantic
        .iter()
        .filter(|record| record.diff_status == "verified_allowed_change")
        .count();
    let status = if errors.is_empty()
        && directories_verified == plan.directory_operations.len()
        && files_verified == plan.copy_operations.len()
        && rewrites_verified == plan.rewrite_operations.len()
    {
        "validation_passed"
    } else {
        "validation_failed"
    };
    PackageValidationResult {
        metadata: PackageValidationMetadata {
            validator_version: PACKAGE_VALIDATOR_VERSION.to_string(),
            validation_schema_version: PACKAGE_VALIDATION_SCHEMA_VERSION.to_string(),
            validation_id: request.validation_id.clone(),
            plan_id: plan.metadata.plan_id.clone(),
            execution_id: staging.metadata.execution_id.clone(),
            rewrite_id: rewrite.metadata.rewrite_id.clone(),
            source_als_hash: plan.metadata.source_als_hash.clone(),
            planned_directory_count: plan.directory_operations.len(),
            verified_directory_count: directories_verified,
            planned_file_count: plan.copy_operations.len(),
            verified_file_count: files_verified,
            planned_rewrite_count: plan.rewrite_operations.len(),
            verified_rewrite_count: rewrites_verified,
            warning_count: 0,
            error_count: errors.len(),
        },
        staging_root: request.staging_root.clone(),
        final_target_root: plan.target_project_root.clone(),
        directory_records: records.directories,
        file_records: records.files,
        semantic_diff_records: records.semantic,
        validation_status: status.to_string(),
        warnings: Vec::new(),
        errors,
    }
}

pub(crate) fn error(
    code: &str,
    message: impl Into<String>,
    operation_id: Option<&str>,
    path: Option<&Path>,
) -> PackageValidationError {
    PackageValidationError {
        error_code: code.to_string(),
        message: message.into(),
        operation_id: operation_id.map(ToString::to_string),
        path: path.map(PathBuf::from),
    }
}
