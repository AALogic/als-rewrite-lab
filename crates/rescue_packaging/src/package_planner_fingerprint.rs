use crate::PackagePlan;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

pub const PACKAGE_PLAN_FINGERPRINT_SCHEMA_VERSION: &str = "0.1";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlanFingerprint {
    pub schema_version: String,
    pub sha256: String,
}

pub fn fingerprint_package_plan(plan: &PackagePlan) -> Result<PlanFingerprint, serde_json::Error> {
    let normalized = normalized_plan(plan);
    let bytes = serde_json::to_vec(&normalized)?;
    let sha256 = Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect();
    Ok(PlanFingerprint {
        schema_version: PACKAGE_PLAN_FINGERPRINT_SCHEMA_VERSION.to_string(),
        sha256,
    })
}

fn normalized_plan(plan: &PackagePlan) -> PackagePlan {
    let mut normalized = plan.clone();
    normalize_runtime_fields(&mut normalized);
    sort_plan_components(&mut normalized);
    normalized
}

fn normalize_runtime_fields(plan: &mut PackagePlan) {
    // Runtime identifiers explain an execution, but do not change what it will do.
    plan.metadata.plan_id.clear();
    for operation in &mut plan.directory_operations {
        operation.operation_id.clear();
    }
    for operation in &mut plan.copy_operations {
        operation.operation_id.clear();
        operation.preconditions.sort();
    }
    for operation in &mut plan.rewrite_operations {
        operation.operation_id.clear();
        operation.fields_to_change.sort();
    }
    for dependency in &mut plan.system_dependencies {
        dependency.als_ref_ids.sort_unstable();
        dependency.observed_source_paths.sort();
    }
    for warning in &mut plan.warnings {
        warning.warning_id = 0;
    }
}

fn sort_plan_components(plan: &mut PackagePlan) {
    plan.directory_operations.sort_by(|left, right| {
        (
            &left.target_relative_path,
            &left.purpose,
            &left.collision_policy,
        )
            .cmp(&(
                &right.target_relative_path,
                &right.purpose,
                &right.collision_policy,
            ))
    });
    plan.copy_operations.sort_by(|left, right| {
        (
            &left.target_relative_path,
            &left.source_path,
            &left.operation_kind,
            &left.source_binding_id,
        )
            .cmp(&(
                &right.target_relative_path,
                &right.source_path,
                &right.operation_kind,
                &right.source_binding_id,
            ))
    });
    plan.rewrite_operations.sort_by(|left, right| {
        (
            &left.required_asset_id,
            left.als_ref_id,
            &left.xml_locator,
            &left.dependency_id,
        )
            .cmp(&(
                &right.required_asset_id,
                right.als_ref_id,
                &right.xml_locator,
                &right.dependency_id,
            ))
    });
    plan.system_dependencies
        .sort_by(|left, right| left.required_asset_id.cmp(&right.required_asset_id));
    plan.unresolved_requirements.sort_by(|left, right| {
        (&left.required_asset_id, &left.decision_status)
            .cmp(&(&right.required_asset_id, &right.decision_status))
    });
    plan.warnings.sort_by(|left, right| {
        (&left.warning_code, &left.required_asset_id, &left.message).cmp(&(
            &right.warning_code,
            &right.required_asset_id,
            &right.message,
        ))
    });
    plan.errors.sort_by(|left, right| {
        (&left.error_code, &left.path, &left.message).cmp(&(
            &right.error_code,
            &right.path,
            &right.message,
        ))
    });
}
