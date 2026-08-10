use crate::package_planner_relocation::{imported_target, relocation_for, ReferenceRelocation};
use crate::{
    CopyOperation, CreateDirectoryOperation, PackagePlanError, RewriteOperation,
    UnresolvedPackageRequirement,
};
use rescue_analyzer::RequiredAsset;
use rescue_core::{ALSReadModel, ActiveAudioReference};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

pub(crate) const LAB_RULE_ID: &str = "live11_3_current_paths_v0.2-lab";
pub(crate) const COMPATIBILITY_LAB_RULE_ID: &str = "known_shape_compatibility_v0.1-lab";

pub(crate) struct OperationBuild {
    pub directories: Vec<CreateDirectoryOperation>,
    pub copies: Vec<CopyOperation>,
    pub rewrites: Vec<RewriteOperation>,
    pub unresolved: Vec<UnresolvedPackageRequirement>,
    pub errors: Vec<PackagePlanError>,
}

#[derive(Clone)]
pub(crate) struct SelectedAsset<'a> {
    pub asset: &'a RequiredAsset,
    pub source_path: PathBuf,
    pub filename: String,
    pub expected_size: u64,
    pub source_binding_id: String,
    pub dedup_key: String,
    pub expected_sha256: Option<String>,
    pub content_id: Option<String>,
    pub verification_policy: String,
}

pub(crate) fn build_operations(
    mode: &str,
    target_root: &Path,
    als_model: &ALSReadModel,
    selected: &[SelectedAsset<'_>],
) -> OperationBuild {
    let mut build = OperationBuild {
        directories: crate::package_planner_directories::project_directories(),
        copies: vec![als_copy_operation(als_model)],
        rewrites: Vec::new(),
        unresolved: Vec::new(),
        errors: Vec::new(),
    };
    let mut audio_targets: BTreeMap<PathBuf, SelectedAsset<'_>> = BTreeMap::new();
    for selected_asset in selected {
        if mode == "copy_only" {
            match imported_target(&selected_asset.filename) {
                Some(target) => {
                    register_audio_target(&mut build, &mut audio_targets, target, selected_asset);
                }
                None => push_unresolved(
                    &mut build,
                    selected_asset.asset,
                    "unsafe_or_empty_target_filename",
                    true,
                ),
            }
        } else {
            add_rewrite_operations(
                &mut build,
                &mut audio_targets,
                mode,
                target_root,
                als_model,
                selected_asset.asset,
                selected_asset,
            );
        }
    }
    crate::package_planner_directories::add_audio_parent_directories(
        &mut build.directories,
        audio_targets.keys(),
    );
    for (index, (target, selected_asset)) in audio_targets.into_iter().enumerate() {
        build
            .copies
            .push(audio_copy_operation(index, target, &selected_asset));
    }
    build
}

fn als_copy_operation(model: &ALSReadModel) -> CopyOperation {
    let target = model
        .set_metadata
        .source_als_filename
        .as_deref()
        .map(PathBuf::from)
        .unwrap_or_default();
    CopyOperation {
        operation_id: "copy_als_000000".to_string(),
        operation_kind: "copy_als".to_string(),
        source_path: PathBuf::from(&model.set_metadata.source_als_path),
        target_relative_path: target,
        expected_source_sha256: Some(model.set_metadata.source_file_hash.clone()),
        expected_source_size: model.set_metadata.source_file_size,
        content_id: Some(format!("als:{}", model.set_metadata.source_file_hash)),
        source_binding_id: format!("als:{}", model.set_metadata.source_file_hash),
        verification_policy: crate::VERIFY_SHA256_AND_SIZE.to_string(),
        collision_policy: "fail_if_exists".to_string(),
        preconditions: vec![
            "source_is_regular_file".to_string(),
            "source_hash_matches_plan".to_string(),
            "target_does_not_exist".to_string(),
        ],
    }
}

fn audio_copy_operation(
    index: usize,
    target_relative_path: PathBuf,
    selected: &SelectedAsset<'_>,
) -> CopyOperation {
    CopyOperation {
        operation_id: format!("copy_audio_{index:06}"),
        operation_kind: "copy_audio".to_string(),
        source_path: selected.source_path.clone(),
        target_relative_path,
        expected_source_sha256: selected.expected_sha256.clone(),
        expected_source_size: selected.expected_size,
        content_id: selected.content_id.clone(),
        source_binding_id: selected.source_binding_id.clone(),
        verification_policy: selected.verification_policy.clone(),
        collision_policy: "fail_if_exists".to_string(),
        preconditions: vec![
            "source_is_regular_file".to_string(),
            if selected.expected_sha256.is_some() {
                "source_hash_matches_plan"
            } else {
                "source_metadata_stable_during_copy"
            }
            .to_string(),
            "target_does_not_exist".to_string(),
        ],
    }
}

fn register_audio_target<'a>(
    build: &mut OperationBuild,
    targets: &mut BTreeMap<PathBuf, SelectedAsset<'a>>,
    target: PathBuf,
    selected: &SelectedAsset<'a>,
) -> bool {
    match targets.get(&target) {
        Some(existing) if existing.dedup_key != selected.dedup_key => {
            build.errors.push(PackagePlanError {
                error_code: "PACKAGE_TARGET_COLLISION".to_string(),
                message: "Different content would use the same target path".to_string(),
                path: Some(target),
            });
            false
        }
        Some(_) => true,
        None => {
            targets.insert(target, selected.clone());
            true
        }
    }
}

fn add_rewrite_operations<'a>(
    build: &mut OperationBuild,
    audio_targets: &mut BTreeMap<PathBuf, SelectedAsset<'a>>,
    mode: &str,
    target_root: &Path,
    model: &ALSReadModel,
    asset: &RequiredAsset,
    selected: &SelectedAsset<'a>,
) {
    if asset.dependency_ids.len() != asset.als_ref_ids.len() {
        push_unresolved(build, asset, "occurrence_contract_mismatch", true);
        return;
    }
    for (dependency_id, als_ref_id) in asset.dependency_ids.iter().zip(&asset.als_ref_ids) {
        let matching_refs: Vec<_> = model
            .active_audio_references
            .iter()
            .filter(|reference| reference.ref_id == *als_ref_id)
            .collect();
        let [reference] = matching_refs.as_slice() else {
            push_unresolved(build, asset, "active_reference_not_unique", true);
            continue;
        };
        let relocation = match relocation_for(
            reference,
            &selected.filename,
            crate::is_compatibility_lab_mode(mode),
        ) {
            Ok(relocation) => relocation,
            Err(reason) => {
                push_unresolved(build, asset, reason, true);
                continue;
            }
        };
        if !register_audio_target(
            build,
            audio_targets,
            relocation.target_relative.clone(),
            selected,
        ) {
            continue;
        }
        let target_absolute = target_root.join(&relocation.target_relative);
        let Some(new_path) = target_absolute.to_str() else {
            push_unresolved(build, asset, "target_absolute_path_is_not_unicode", true);
            continue;
        };
        let operation_id = format!("rewrite_active_{:06}", build.rewrites.len());
        build.rewrites.push(rewrite_operation(
            operation_id,
            RewriteOperationContext {
                mode,
                model,
                asset,
                dependency_id,
                reference,
                new_path,
            },
            relocation,
        ));
    }
}

struct RewriteOperationContext<'a> {
    mode: &'a str,
    model: &'a ALSReadModel,
    asset: &'a RequiredAsset,
    dependency_id: &'a str,
    reference: &'a ActiveAudioReference,
    new_path: &'a str,
}

fn rewrite_operation(
    operation_id: String,
    context: RewriteOperationContext<'_>,
    relocation: ReferenceRelocation,
) -> RewriteOperation {
    RewriteOperation {
        operation_id,
        required_asset_id: context.asset.required_asset_id.clone(),
        dependency_id: context.dependency_id.to_string(),
        als_ref_id: context.reference.ref_id,
        xml_locator: context.reference.xml_locator.clone(),
        source_als_hash: context.model.set_metadata.source_file_hash.clone(),
        old_path: context.reference.raw_path.clone(),
        old_relative_path: context.reference.raw_relative_path.clone(),
        old_relative_path_type: context.reference.relative_path_type.clone(),
        new_path: context.new_path.to_string(),
        new_relative_path: relocation.new_relative_path,
        new_relative_path_type: relocation.new_relative_path_type,
        fields_to_change: relocation.fields_to_change,
        rule_id: if crate::is_compatibility_lab_mode(context.mode) {
            COMPATIBILITY_LAB_RULE_ID
        } else {
            LAB_RULE_ID
        }
        .to_string(),
        support_status: relocation.support_status.to_string(),
    }
}

fn push_unresolved(
    build: &mut OperationBuild,
    asset: &RequiredAsset,
    reason: &str,
    blocks_execution: bool,
) {
    build.unresolved.push(UnresolvedPackageRequirement {
        required_asset_id: asset.required_asset_id.clone(),
        decision_status: "blocked".to_string(),
        reason: reason.to_string(),
        blocks_execution,
    });
}
