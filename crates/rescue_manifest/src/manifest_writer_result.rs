use crate::{
    ManifestWriteError, ManifestWriteMetadata, ManifestWriteRecord, ManifestWriteRequest,
    ManifestWriteResult, MANIFEST_WRITER_VERSION,
};
use rescue_execution::StagingExecutionResult;
use rescue_packaging::PackagePlan;
use rescue_rewriter::ALSRewriteResult;
use rescue_validation::PackageValidationResult;
use std::path::{Path, PathBuf};

pub(crate) struct WriteContext<'a> {
    pub request: &'a ManifestWriteRequest,
    pub plan: &'a PackagePlan,
    pub staging: &'a StagingExecutionResult,
    pub rewrite: &'a ALSRewriteResult,
    pub validation: &'a PackageValidationResult,
}

pub(crate) fn result(
    context: &WriteContext<'_>,
    records: Vec<ManifestWriteRecord>,
    package: Option<(String, u64)>,
    ledger: Option<(String, u64)>,
    errors: Vec<ManifestWriteError>,
) -> ManifestWriteResult {
    ManifestWriteResult {
        metadata: ManifestWriteMetadata {
            writer_version: MANIFEST_WRITER_VERSION.to_string(),
            manifest_id: context.request.manifest_id.clone(),
            plan_id: context.plan.metadata.plan_id.clone(),
            execution_id: context.staging.metadata.execution_id.clone(),
            rewrite_id: context.rewrite.metadata.rewrite_id.clone(),
            validation_id: context.validation.metadata.validation_id.clone(),
            error_count: errors.len(),
        },
        private_ledger_path: context.request.private_ledger_path.clone(),
        package_manifest_relative_path: context.request.package_manifest_relative_path.clone(),
        package_manifest_sha256: package.as_ref().map(|value| value.0.clone()),
        package_manifest_size: package.as_ref().map(|value| value.1),
        private_ledger_sha256: ledger.as_ref().map(|value| value.0.clone()),
        private_ledger_size: ledger.as_ref().map(|value| value.1),
        write_records: records,
        write_status: if errors.is_empty() {
            "manifests_written"
        } else {
            "manifest_write_failed"
        }
        .to_string(),
        errors,
    }
}

pub(crate) fn failed(
    context: &WriteContext<'_>,
    code: &str,
    message: String,
    path: Option<&Path>,
) -> ManifestWriteResult {
    result(
        context,
        Vec::new(),
        None,
        None,
        vec![error(code, message, path)],
    )
}

pub(crate) fn io_failed(
    context: &WriteContext<'_>,
    failure: crate::manifest_io::ArtifactWriteFailure,
    artifact: &str,
) -> ManifestWriteResult {
    result(
        context,
        Vec::new(),
        None,
        None,
        vec![write_error(failure, artifact)],
    )
}

pub(crate) fn record(
    kind: &str,
    path: PathBuf,
    write: &crate::manifest_io::ArtifactWrite,
) -> ManifestWriteRecord {
    ManifestWriteRecord {
        artifact_kind: kind.to_string(),
        path,
        sha256: Some(write.sha256.clone()),
        size: Some(write.size),
        write_status: write.status.clone(),
    }
}

pub(crate) fn write_error(
    failure: crate::manifest_io::ArtifactWriteFailure,
    artifact: &str,
) -> ManifestWriteError {
    ManifestWriteError {
        error_code: failure.code.to_string(),
        message: failure.message,
        artifact_kind: Some(artifact.to_string()),
        path: Some(failure.path),
    }
}

pub(crate) fn error(
    code: &str,
    message: impl Into<String>,
    path: Option<&Path>,
) -> ManifestWriteError {
    ManifestWriteError {
        error_code: code.to_string(),
        message: message.into(),
        artifact_kind: None,
        path: path.map(Path::to_path_buf),
    }
}
