use crate::{
    ManifestFile, ManifestRewrite, ManifestValidationSummary, PackageManifest, PrivateLedger,
    PACKAGE_MANIFEST_SCHEMA_VERSION, PRIVATE_LEDGER_SCHEMA_VERSION,
};
use rescue_execution::StagingExecutionResult;
use rescue_packaging::PackagePlan;
use rescue_rewriter::ALSRewriteResult;
use rescue_validation::PackageValidationResult;
use std::collections::BTreeMap;

pub(crate) fn private_ledger(
    ledger_id: &str,
    plan: &PackagePlan,
    staging: &StagingExecutionResult,
    rewrite: &ALSRewriteResult,
    validation: &PackageValidationResult,
) -> PrivateLedger {
    PrivateLedger {
        ledger_schema_version: PRIVATE_LEDGER_SCHEMA_VERSION.to_string(),
        ledger_id: ledger_id.to_string(),
        plan: plan.clone(),
        staging: staging.clone(),
        rewrite: rewrite.clone(),
        validation: validation.clone(),
    }
}

pub(crate) fn package_manifest(
    manifest_id: &str,
    plan: &PackagePlan,
    validation: &PackageValidationResult,
) -> Result<PackageManifest, String> {
    let operations: BTreeMap<_, _> = plan
        .copy_operations
        .iter()
        .map(|operation| (operation.operation_id.as_str(), operation))
        .collect();
    let mut files = Vec::new();
    for record in &validation.file_records {
        let operation = operations
            .get(record.operation_id.as_str())
            .ok_or_else(|| "Validation file record is not present in the plan".to_string())?;
        let hash = record
            .observed_sha256
            .clone()
            .ok_or_else(|| "Validated file record has no observed hash".to_string())?;
        let size = record
            .observed_size
            .ok_or_else(|| "Validated file record has no observed size".to_string())?;
        files.push(ManifestFile {
            role: operation.operation_kind.clone(),
            relative_path: record.target_relative_path.clone(),
            sha256: hash,
            size,
        });
    }
    let rewrites = plan
        .rewrite_operations
        .iter()
        .map(|operation| ManifestRewrite {
            operation_id: operation.operation_id.clone(),
            xml_locator: operation.xml_locator.clone(),
            changed_fields: operation.fields_to_change.clone(),
            ruleset_version: operation.rule_id.clone(),
            status: "verified_allowed_change".to_string(),
        })
        .collect();
    let source_set_filename = plan
        .source_als
        .target_relative_path
        .file_name()
        .and_then(|value| value.to_str())
        .ok_or_else(|| "Source Set filename is not portable Unicode".to_string())?
        .to_string();
    Ok(PackageManifest {
        manifest_schema_version: PACKAGE_MANIFEST_SCHEMA_VERSION.to_string(),
        manifest_id: manifest_id.to_string(),
        plan_id: plan.metadata.plan_id.clone(),
        source_set_filename,
        source_als_sha256: plan.source_als.source_file_hash.clone(),
        rewrite_ruleset_version: plan.metadata.rewrite_ruleset_version.clone(),
        package_status: "ready_for_manual_ableton_check".to_string(),
        files,
        rewrites,
        validation: ManifestValidationSummary {
            validator_version: validation.metadata.validator_version.clone(),
            validation_id: validation.metadata.validation_id.clone(),
            validation_status: validation.validation_status.clone(),
            verified_file_count: validation.metadata.verified_file_count,
            verified_rewrite_count: validation.metadata.verified_rewrite_count,
        },
    })
}
