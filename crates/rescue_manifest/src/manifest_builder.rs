use crate::{
    ManifestDirectory, ManifestFile, ManifestOmission, ManifestRewrite, ManifestSystemDependency,
    ManifestValidationSummary, PackageManifest, PrivateLedger, PACKAGE_MANIFEST_SCHEMA_VERSION,
    PRIVATE_LEDGER_SCHEMA_VERSION,
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
        let size = record
            .observed_size
            .ok_or_else(|| "Validated file record has no observed size".to_string())?;
        files.push(ManifestFile {
            role: operation.operation_kind.clone(),
            relative_path: record.target_relative_path.clone(),
            sha256: record.observed_sha256.clone(),
            size,
            verification_method: record.verification_method.clone(),
            content_identity_status: if record.observed_sha256.is_some() {
                "computed"
            } else {
                "not_computed"
            }
            .to_string(),
        });
    }
    let directories = validation
        .directory_records
        .iter()
        .map(|record| ManifestDirectory {
            relative_path: record.target_relative_path.clone(),
            purpose: record.purpose.clone(),
            status: record.directory_status.clone(),
        })
        .collect();
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
    let omissions = plan
        .unresolved_requirements
        .iter()
        .map(|requirement| ManifestOmission {
            required_asset_id: requirement.required_asset_id.clone(),
            reason: requirement.reason.clone(),
            reference_status: if requirement.blocks_execution {
                "blocked"
            } else {
                "left_unchanged"
            }
            .to_string(),
        })
        .collect();
    let system_dependencies = plan
        .system_dependencies
        .iter()
        .map(|dependency| ManifestSystemDependency {
            required_asset_id: dependency.required_asset_id.clone(),
            source_category: dependency.source_category.clone(),
            filename: dependency.filename.clone(),
            occurrence_count: dependency.occurrence_count,
            package_action: dependency.package_action.clone(),
            portability_status: dependency.portability_status.clone(),
            required_environment: "compatible_ableton_core_library".to_string(),
        })
        .collect();
    Ok(PackageManifest {
        manifest_schema_version: PACKAGE_MANIFEST_SCHEMA_VERSION.to_string(),
        manifest_id: manifest_id.to_string(),
        plan_id: plan.metadata.plan_id.clone(),
        source_set_filename,
        source_als_sha256: plan.source_als.source_file_hash.clone(),
        rewrite_ruleset_version: plan.metadata.rewrite_ruleset_version.clone(),
        package_status: manifest_package_status(plan).to_string(),
        directories,
        files,
        rewrites,
        system_dependencies,
        omissions,
        validation: ManifestValidationSummary {
            validator_version: validation.metadata.validator_version.clone(),
            validation_id: validation.metadata.validation_id.clone(),
            validation_status: validation.validation_status.clone(),
            verified_directory_count: validation.metadata.verified_directory_count,
            verified_file_count: validation.metadata.verified_file_count,
            verified_rewrite_count: validation.metadata.verified_rewrite_count,
        },
    })
}

fn manifest_package_status(plan: &PackagePlan) -> &'static str {
    match plan.plan_status.as_str() {
        "ready_current_paths_complete" => "complete_copy_ready_for_manual_check",
        "ready_current_paths_incomplete" => "incomplete_copy_ready_for_manual_check",
        _ => "ready_for_manual_ableton_check",
    }
}
