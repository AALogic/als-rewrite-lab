use crate::{CopyOperation, PackagePlanError, RewriteOperation, UnresolvedPackageRequirement};
use rescue_analyzer::RequiredAsset;
use rescue_catalog::FileOccurrence;
use rescue_core::{ALSReadModel, ActiveAudioReference};
use std::collections::BTreeMap;
use std::path::{Component, Path, PathBuf};

pub(crate) const LAB_RULE_ID: &str = "live11_3_external_to_imported_v0.1-experimental";

pub(crate) struct OperationBuild {
    pub copies: Vec<CopyOperation>,
    pub rewrites: Vec<RewriteOperation>,
    pub unresolved: Vec<UnresolvedPackageRequirement>,
    pub errors: Vec<PackagePlanError>,
}

pub(crate) struct SelectedAsset<'a> {
    pub asset: &'a RequiredAsset,
    pub occurrence: &'a FileOccurrence,
}

pub(crate) fn build_operations(
    mode: &str,
    target_root: &Path,
    als_model: &ALSReadModel,
    selected: &[SelectedAsset<'_>],
) -> OperationBuild {
    let mut build = OperationBuild {
        copies: vec![als_copy_operation(als_model)],
        rewrites: Vec::new(),
        unresolved: Vec::new(),
        errors: Vec::new(),
    };
    let mut audio_targets: BTreeMap<PathBuf, (&str, &FileOccurrence)> = BTreeMap::new();
    for selected_asset in selected {
        let Some(target_relative) = imported_target(&selected_asset.occurrence.filename) else {
            push_unresolved(
                &mut build,
                selected_asset.asset,
                "unsafe_or_empty_target_filename",
            );
            continue;
        };
        match audio_targets.get(&target_relative) {
            Some((content_id, _))
                if *content_id != selected_asset.occurrence.content_id.as_str() =>
            {
                build.errors.push(PackagePlanError {
                    error_code: "PACKAGE_TARGET_COLLISION".to_string(),
                    message: "Different content would use the same target path".to_string(),
                    path: Some(target_relative),
                });
                continue;
            }
            Some(_) => {}
            None => {
                audio_targets.insert(
                    target_relative.clone(),
                    (
                        selected_asset.occurrence.content_id.as_str(),
                        selected_asset.occurrence,
                    ),
                );
            }
        }
        if mode == "laboratory_rescue_rewrite" {
            add_rewrite_operations(
                &mut build,
                target_root,
                als_model,
                selected_asset.asset,
                &target_relative,
            );
        }
    }
    for (index, (target, (_, occurrence))) in audio_targets.into_iter().enumerate() {
        build
            .copies
            .push(audio_copy_operation(index, target, occurrence));
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
        expected_source_sha256: model.set_metadata.source_file_hash.clone(),
        expected_source_size: model.set_metadata.source_file_size,
        content_id: format!("als:{}", model.set_metadata.source_file_hash),
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
    occurrence: &FileOccurrence,
) -> CopyOperation {
    CopyOperation {
        operation_id: format!("copy_audio_{index:06}"),
        operation_kind: "copy_audio".to_string(),
        source_path: occurrence.native_path.clone(),
        target_relative_path,
        expected_source_sha256: occurrence
            .content_id
            .trim_start_matches("sha256:")
            .to_string(),
        expected_source_size: occurrence.file_size,
        content_id: occurrence.content_id.clone(),
        collision_policy: "fail_if_exists".to_string(),
        preconditions: vec![
            "source_is_regular_file".to_string(),
            "source_hash_matches_plan".to_string(),
            "target_does_not_exist".to_string(),
        ],
    }
}

fn imported_target(filename: &str) -> Option<PathBuf> {
    let path = Path::new(filename);
    if path.is_absolute()
        || path.components().count() != 1
        || path
            .components()
            .any(|component| !matches!(component, Component::Normal(_)))
    {
        return None;
    }
    Some(PathBuf::from("Samples").join("Imported").join(path))
}

fn add_rewrite_operations(
    build: &mut OperationBuild,
    target_root: &Path,
    model: &ALSReadModel,
    asset: &RequiredAsset,
    target_relative: &Path,
) {
    if asset.dependency_ids.len() != asset.als_ref_ids.len() {
        push_unresolved(build, asset, "occurrence_contract_mismatch");
        return;
    }
    let Some(new_relative) = target_relative.to_str() else {
        push_unresolved(build, asset, "target_relative_path_is_not_unicode");
        return;
    };
    let target_absolute = target_root.join(target_relative);
    let Some(new_path) = target_absolute.to_str() else {
        push_unresolved(build, asset, "target_absolute_path_is_not_unicode");
        return;
    };
    for (dependency_id, als_ref_id) in asset.dependency_ids.iter().zip(&asset.als_ref_ids) {
        let matching_refs: Vec<_> = model
            .active_audio_references
            .iter()
            .filter(|reference| reference.ref_id == *als_ref_id)
            .collect();
        let [reference] = matching_refs.as_slice() else {
            push_unresolved(build, asset, "active_reference_not_unique");
            continue;
        };
        if !supported_external_reference(reference) {
            push_unresolved(build, asset, "rewrite_reference_not_supported");
            continue;
        }
        let operation_id = format!("rewrite_active_{:06}", build.rewrites.len());
        build.rewrites.push(rewrite_operation(
            operation_id,
            model,
            asset,
            dependency_id,
            reference,
            new_path,
            new_relative,
        ));
    }
}

fn supported_external_reference(reference: &ActiveAudioReference) -> bool {
    reference.relative_path_type.as_deref() == Some("1")
        && reference.xml_locator == format!("SampleRef[{}]/FileRef", reference.ref_id)
        && reference.xml_locator.starts_with("SampleRef[")
        && reference.xml_locator.ends_with("]/FileRef")
}

fn rewrite_operation(
    operation_id: String,
    model: &ALSReadModel,
    asset: &RequiredAsset,
    dependency_id: &str,
    reference: &ActiveAudioReference,
    new_path: &str,
    new_relative: &str,
) -> RewriteOperation {
    RewriteOperation {
        operation_id,
        required_asset_id: asset.required_asset_id.clone(),
        dependency_id: dependency_id.to_string(),
        als_ref_id: reference.ref_id,
        xml_locator: reference.xml_locator.clone(),
        source_als_hash: model.set_metadata.source_file_hash.clone(),
        old_path: reference.raw_path.clone(),
        old_relative_path: reference.raw_relative_path.clone(),
        old_relative_path_type: reference.relative_path_type.clone(),
        new_path: new_path.to_string(),
        new_relative_path: new_relative.to_string(),
        new_relative_path_type: "3".to_string(),
        fields_to_change: vec![
            "Path".to_string(),
            "RelativePath".to_string(),
            "RelativePathType".to_string(),
        ],
        rule_id: LAB_RULE_ID.to_string(),
        support_status: "experimental_lab_only".to_string(),
    }
}

fn push_unresolved(build: &mut OperationBuild, asset: &RequiredAsset, reason: &str) {
    build.unresolved.push(UnresolvedPackageRequirement {
        required_asset_id: asset.required_asset_id.clone(),
        decision_status: "blocked".to_string(),
        reason: reason.to_string(),
    });
}
