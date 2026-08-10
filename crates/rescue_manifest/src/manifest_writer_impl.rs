use crate::{ManifestWriteRequest, ManifestWriteResult};
use rescue_execution::StagingExecutionResult;
use rescue_packaging::PackagePlan;
use rescue_rewriter::ALSRewriteResult;
use rescue_validation::PackageValidationResult;

pub(crate) fn write_package_evidence_impl(
    request: &ManifestWriteRequest,
    plan: &PackagePlan,
    staging: &StagingExecutionResult,
    rewrite: &ALSRewriteResult,
    validation: &PackageValidationResult,
) -> ManifestWriteResult {
    let context = crate::manifest_writer_result::WriteContext {
        request,
        plan,
        staging,
        rewrite,
        validation,
    };
    let errors = crate::manifest_writer_validation::validate_inputs(
        request, plan, staging, rewrite, validation,
    );
    if !errors.is_empty() {
        return crate::manifest_writer_result::result(&context, Vec::new(), None, None, errors);
    }
    let package_manifest =
        match crate::manifest_builder::package_manifest(&request.manifest_id, plan, validation) {
            Ok(manifest) => manifest,
            Err(message) => {
                return crate::manifest_writer_result::failed(
                    &context,
                    "PACKAGE_MANIFEST_BUILD_FAILED",
                    message,
                    None,
                )
            }
        };
    if let Err(message) = crate::manifest_privacy::validate_portable_manifest(&package_manifest) {
        return crate::manifest_writer_result::failed(
            &context,
            "PACKAGE_MANIFEST_PRIVACY_FAILED",
            message,
            None,
        );
    }
    let private_ledger = crate::manifest_builder::private_ledger(
        &request.manifest_id,
        plan,
        staging,
        rewrite,
        validation,
    );
    let package_bytes = match crate::manifest_io::serialize_json(&package_manifest) {
        Ok(bytes) => bytes,
        Err(failure) => {
            return crate::manifest_writer_result::io_failed(&context, failure, "package_manifest")
        }
    };
    let ledger_bytes = match crate::manifest_io::serialize_json(&private_ledger) {
        Ok(bytes) => bytes,
        Err(failure) => {
            return crate::manifest_writer_result::io_failed(&context, failure, "private_ledger")
        }
    };
    write_artifacts(&context, &package_bytes, &ledger_bytes)
}

fn write_artifacts(
    context: &crate::manifest_writer_result::WriteContext<'_>,
    package_bytes: &[u8],
    ledger_bytes: &[u8],
) -> ManifestWriteResult {
    let package_path = context
        .staging
        .staging_root
        .join(&context.request.package_manifest_relative_path);
    let package_write =
        match crate::manifest_io::write_json_noclobber(&package_path, package_bytes, true) {
            Ok(write) => write,
            Err(failure) => {
                return crate::manifest_writer_result::io_failed(
                    context,
                    failure,
                    "package_manifest",
                )
            }
        };
    let mut records = vec![crate::manifest_writer_result::record(
        "package_manifest",
        context.request.package_manifest_relative_path.clone(),
        &package_write,
    )];
    let ledger_write = match crate::manifest_io::write_json_noclobber(
        &context.request.private_ledger_path,
        ledger_bytes,
        false,
    ) {
        Ok(write) => write,
        Err(failure) => {
            let error = crate::manifest_writer_result::write_error(failure, "private_ledger");
            return crate::manifest_writer_result::result(
                context,
                records,
                Some((package_write.sha256, package_write.size)),
                None,
                vec![error],
            );
        }
    };
    records.push(crate::manifest_writer_result::record(
        "private_ledger",
        context.request.private_ledger_path.clone(),
        &ledger_write,
    ));
    crate::manifest_writer_result::result(
        context,
        records,
        Some((package_write.sha256, package_write.size)),
        Some((ledger_write.sha256, ledger_write.size)),
        Vec::new(),
    )
}
