use crate::{
    CopyExecutionRecord, DirectoryExecutionRecord, StagingExecutionError, StagingExecutionMetadata,
    StagingExecutionRequest, StagingExecutionResult, STAGING_EXECUTION_SCHEMA_VERSION,
    STAGING_EXECUTOR_VERSION,
};
use rescue_packaging::{CopyOperation, PackagePlan};
use std::fs;
use std::path::PathBuf;

pub(crate) fn execute_staging_impl(
    request: &StagingExecutionRequest,
    plan: &PackagePlan,
) -> StagingExecutionResult {
    let mut errors = crate::staging_validation::validate_request(request, plan);
    if !errors.is_empty() {
        return result(request, plan, Vec::new(), Vec::new(), "rejected", errors);
    }
    if let Err(error) = fs::create_dir(&request.staging_root) {
        errors.push(execution_error(
            "STAGING_ROOT_CREATE_FAILED",
            format!("Cannot create a new staging root: {error}"),
            None,
            Some(request.staging_root.clone()),
        ));
        return result(request, plan, Vec::new(), Vec::new(), "copy_failed", errors);
    }

    let directory_records = match crate::staging_directories::create_directories(
        &request.staging_root,
        &plan.directory_operations,
    ) {
        Ok(records) => records,
        Err(failure) => {
            errors.push(failure.error);
            return result(
                request,
                plan,
                failure.records,
                Vec::new(),
                "directory_failed",
                errors,
            );
        }
    };

    let mut records = Vec::new();
    for (index, operation) in plan.copy_operations.iter().enumerate() {
        match crate::staging_copy::copy_verified(&request.staging_root, operation, index) {
            Ok(outcome) => records.push(record(
                operation,
                outcome.digest,
                Some(outcome.size),
                "copied_and_verified",
            )),
            Err(failure) => {
                records.push(record(operation, None, None, "failed"));
                errors.push(execution_error(
                    failure.code,
                    failure.message,
                    Some(operation.operation_id.clone()),
                    failure.path,
                ));
                return result(
                    request,
                    plan,
                    directory_records,
                    records,
                    "copy_failed",
                    errors,
                );
            }
        }
    }
    result(
        request,
        plan,
        directory_records,
        records,
        "staging_complete",
        errors,
    )
}

fn record(
    operation: &CopyOperation,
    digest: Option<String>,
    size: Option<u64>,
    status: &str,
) -> CopyExecutionRecord {
    CopyExecutionRecord {
        operation_id: operation.operation_id.clone(),
        operation_kind: operation.operation_kind.clone(),
        source_path: operation.source_path.clone(),
        target_relative_path: operation.target_relative_path.clone(),
        expected_sha256: operation.expected_source_sha256.clone(),
        observed_sha256: digest,
        expected_size: operation.expected_source_size,
        observed_size: size,
        verification_method: operation.verification_policy.clone(),
        operation_status: status.to_string(),
    }
}

fn result(
    request: &StagingExecutionRequest,
    plan: &PackagePlan,
    directory_records: Vec<DirectoryExecutionRecord>,
    records: Vec<CopyExecutionRecord>,
    status: &str,
    errors: Vec<StagingExecutionError>,
) -> StagingExecutionResult {
    let completed = records
        .iter()
        .filter(|record| record.operation_status == "copied_and_verified")
        .count();
    let completed_directories = directory_records
        .iter()
        .filter(|record| record.operation_status == "created")
        .count();
    StagingExecutionResult {
        metadata: StagingExecutionMetadata {
            executor_version: STAGING_EXECUTOR_VERSION.to_string(),
            execution_schema_version: STAGING_EXECUTION_SCHEMA_VERSION.to_string(),
            execution_id: request.execution_id.clone(),
            plan_id: plan.metadata.plan_id.clone(),
            source_als_hash: plan.metadata.source_als_hash.clone(),
            planned_directory_count: plan.directory_operations.len(),
            completed_directory_count: completed_directories,
            planned_copy_count: plan.copy_operations.len(),
            completed_copy_count: completed,
            warning_count: 0,
            error_count: errors.len(),
        },
        staging_root: request.staging_root.clone(),
        staged_als_relative_path: plan.source_als.target_relative_path.clone(),
        directory_records,
        copy_records: records,
        execution_status: status.to_string(),
        warnings: Vec::new(),
        errors,
    }
}

pub(crate) fn execution_error(
    code: &str,
    message: String,
    operation_id: Option<String>,
    path: Option<PathBuf>,
) -> StagingExecutionError {
    StagingExecutionError {
        error_code: code.to_string(),
        message,
        operation_id,
        path,
    }
}
