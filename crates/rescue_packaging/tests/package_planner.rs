use rescue_analyzer::{DependencyAssessmentMetadata, DependencyAssessmentResult, RequiredAsset};
use rescue_catalog::{AssetInventoryMetadata, AssetInventoryResult, ContentRecord, FileOccurrence};
use rescue_core::{ALSReadModel, ActiveAudioReference, SetMetadata};
use rescue_packaging::{
    fingerprint_package_plan, plan_current_path_package, plan_package, PackagePlanningRequest,
};
use rescue_resolution::{
    AssetResolutionMetadata, AssetResolutionResult, CurrentPathBinding, CurrentPathBindingMetadata,
    CurrentPathBindingResult, ResolutionDecision, CURRENT_PATH_BINDING_POLICY_VERSION,
    CURRENT_PATH_BINDING_VERSION,
};
use std::path::{Path, PathBuf};

#[cfg(windows)]
fn native_absolute(parts: &[&str]) -> PathBuf {
    let mut path = PathBuf::from(r"C:\");
    path.extend(parts);
    path
}

#[cfg(not(windows))]
fn native_absolute(parts: &[&str]) -> PathBuf {
    let mut path = PathBuf::from("/");
    path.extend(parts);
    path
}

fn native_fixture_path(path: &str) -> PathBuf {
    native_absolute(
        &path
            .split('/')
            .filter(|component| !component.is_empty())
            .collect::<Vec<_>>(),
    )
}

fn active_ref(index: usize) -> ActiveAudioReference {
    let filename = format!("sample{index}.wav");
    ActiveAudioReference {
        ref_id: index,
        source_kind: "sample_ref".to_string(),
        raw_path: Some(
            native_absolute(&["source", &filename])
                .to_string_lossy()
                .to_string(),
        ),
        raw_relative_path: Some(format!("../sample{index}.wav")),
        relative_path_type: Some("1".to_string()),
        file_type: Some("1".to_string()),
        filename: Some("sample.wav".to_string()),
        extension: Some("wav".to_string()),
        original_file_size: Some("100".to_string()),
        original_crc: Some("42".to_string()),
        default_duration: None,
        default_sample_rate: None,
        usage_context: "audio_clip".to_string(),
        xml_context: "Ableton/LiveSet/SampleRef/FileRef".to_string(),
        xml_locator: format!("SampleRef[{index}]/FileRef"),
        is_rewrite_candidate: true,
        rewrite_support_status: "supported".to_string(),
        warnings: Vec::new(),
    }
}

fn project_local_ref(index: usize, relative_path: &str) -> ActiveAudioReference {
    let mut reference = active_ref(index);
    reference.raw_path = Some(
        native_absolute(&["source", "Project", relative_path])
            .to_string_lossy()
            .to_string(),
    );
    reference.raw_relative_path = Some(relative_path.to_string());
    reference.relative_path_type = Some("3".to_string());
    reference
}

fn als_model(reference_count: usize) -> ALSReadModel {
    ALSReadModel {
        set_metadata: SetMetadata {
            source_als_path: native_absolute(&["source", "Set.als"])
                .to_string_lossy()
                .to_string(),
            source_als_filename: Some("Set.als".to_string()),
            source_project_root: None,
            source_file_size: 500,
            source_file_hash: "als-hash".to_string(),
            analysis_started_at: "fixture".to_string(),
            analysis_completed_at: "fixture".to_string(),
            reader_version: "0.2.2".to_string(),
            als_read_model_version: "0.2".to_string(),
            ableton_document_version: Some("5".to_string()),
            ableton_creator_version: Some("Ableton Live 11.3.43".to_string()),
            ableton_minor_version: Some("11.0_11300".to_string()),
            ableton_schema_change_count: Some("7".to_string()),
            decompressed_xml_size: 1000,
            xml_root_name: "Ableton".to_string(),
            sample_ref_count: reference_count,
            active_audio_ref_count: reference_count,
            historical_ref_count: 0,
            non_audio_signal_count: 0,
            warning_count: 0,
            error_count: 0,
        },
        active_audio_references: (0..reference_count).map(active_ref).collect(),
        historical_refs: Vec::new(),
        non_audio_dependency_signals: Vec::new(),
        warnings: Vec::new(),
        errors: Vec::new(),
    }
}

fn asset(id: &str, dependency_start: usize, occurrence_count: usize) -> RequiredAsset {
    RequiredAsset {
        required_asset_id: id.to_string(),
        grouping_basis: "exact_recorded_reference_claim".to_string(),
        dependency_ids: (dependency_start..dependency_start + occurrence_count)
            .map(|index| format!("dep{index}"))
            .collect(),
        als_ref_ids: (dependency_start..dependency_start + occurrence_count).collect(),
        occurrence_count,
        filename: Some("sample.wav".to_string()),
        extension: Some("wav".to_string()),
        original_file_size: Some("100".to_string()),
        original_crc: Some("42".to_string()),
        source_category: "unclassified".to_string(),
        management_class: "unclassified".to_string(),
        source_classification_status: "unknown".to_string(),
        source_classification_basis: "insufficient_source_category_evidence".to_string(),
        candidate_observations: Vec::new(),
        availability_status: "regular_file_candidate_observed".to_string(),
        resolution_status: "unresolved".to_string(),
        risk_flags: Vec::new(),
        evidence_status: "observed_unresolved".to_string(),
    }
}

fn assessment(assets: Vec<RequiredAsset>) -> DependencyAssessmentResult {
    let occurrence_count = assets.iter().map(|asset| asset.occurrence_count).sum();
    DependencyAssessmentResult {
        assessment_metadata: DependencyAssessmentMetadata {
            assessment_version: "0.2.0".to_string(),
            input_dependency_ref_version: "0.1".to_string(),
            input_path_observation_model_version: "0.2".to_string(),
            source_als_path: native_absolute(&["source", "Set.als"])
                .to_string_lossy()
                .to_string(),
            source_file_hash: "als-hash".to_string(),
            occurrence_count,
            required_asset_count: assets.len(),
            regular_file_candidate_asset_count: assets.len(),
            missing_candidate_asset_count: 0,
            unknown_asset_count: 0,
            warning_count: 0,
            error_count: 0,
        },
        required_assets: assets,
        warnings: Vec::new(),
        errors: Vec::new(),
    }
}

fn inventory(files: &[(&str, &str, &str)]) -> AssetInventoryResult {
    let file_occurrences: Vec<_> = files
        .iter()
        .enumerate()
        .map(|(index, (path, filename, digest))| FileOccurrence {
            file_occurrence_id: format!("occ{index}"),
            content_id: format!("sha256:{digest}"),
            source_root: native_absolute(&["source"]),
            native_path: native_fixture_path(path),
            relative_path: PathBuf::from(filename),
            filename: (*filename).to_string(),
            extension: "wav".to_string(),
            file_size: 100,
            entry_kind: "regular_file".to_string(),
            observation_status: "stable_full_hash".to_string(),
        })
        .collect();
    let content_records = file_occurrences
        .iter()
        .map(|occurrence| ContentRecord {
            content_id: occurrence.content_id.clone(),
            hash_algorithm: "sha256_full_bytes".to_string(),
            digest: occurrence
                .content_id
                .trim_start_matches("sha256:")
                .to_string(),
            file_size: occurrence.file_size,
            occurrence_ids: vec![occurrence.file_occurrence_id.clone()],
        })
        .collect();
    AssetInventoryResult {
        metadata: AssetInventoryMetadata {
            inventory_version: "0.1.0".to_string(),
            scan_run_id: "scan0".to_string(),
            scan_status: "complete".to_string(),
            requested_root_count: 1,
            scanned_root_count: 1,
            entries_visited: files.len(),
            audio_file_count: files.len(),
            content_record_count: files.len(),
            skipped_symlink_count: 0,
            warning_count: 0,
            error_count: 0,
        },
        file_occurrences,
        content_records,
        warnings: Vec::new(),
        errors: Vec::new(),
    }
}

fn resolution(
    assets: &[RequiredAsset],
    occurrences: &[FileOccurrence],
    statuses: &[&str],
) -> AssetResolutionResult {
    let decisions: Vec<_> = assets
        .iter()
        .enumerate()
        .map(|(index, asset)| {
            let accepted = statuses[index] == "auto_accepted";
            let occurrence = occurrences.get(index);
            ResolutionDecision {
                required_asset_id: asset.required_asset_id.clone(),
                decision_status: statuses[index].to_string(),
                selected_candidate_id: accepted.then(|| format!("candidate{index}")),
                selected_file_occurrence_id: accepted
                    .then(|| occurrence.map(|item| item.file_occurrence_id.clone()))
                    .flatten(),
                selected_content_id: accepted
                    .then(|| occurrence.map(|item| item.content_id.clone()))
                    .flatten(),
                score: accepted.then_some(100),
                policy_version: "0.3.0".to_string(),
                decision_basis: if accepted {
                    "current_recorded_path_binding".to_string()
                } else {
                    "fixture_unresolved".to_string()
                },
                requires_user_confirmation: !accepted && statuses[index] != "unresolved",
            }
        })
        .collect();
    AssetResolutionResult {
        metadata: AssetResolutionMetadata {
            resolution_version: "0.1.0".to_string(),
            policy_version: "0.3.0".to_string(),
            input_assessment_version: "0.2.0".to_string(),
            input_inventory_version: "0.1.0".to_string(),
            scan_run_id: "scan0".to_string(),
            required_asset_count: assets.len(),
            proposal_count: assets.len(),
            auto_accepted_count: statuses
                .iter()
                .filter(|status| **status == "auto_accepted")
                .count(),
            manual_review_count: statuses
                .iter()
                .filter(|status| **status == "needs_user_confirmation")
                .count(),
            unresolved_count: statuses
                .iter()
                .filter(|status| **status == "unresolved")
                .count(),
            warning_count: 0,
            error_count: 0,
        },
        proposals: Vec::new(),
        decisions,
        warnings: Vec::new(),
        errors: Vec::new(),
    }
}

fn request(mode: &str) -> PackagePlanningRequest {
    PackagePlanningRequest {
        plan_id: "plan0".to_string(),
        target_project_root: native_absolute(&["target", "Project"]),
        planning_mode: mode.to_string(),
    }
}

fn valid_inputs(
    occurrence_count: usize,
) -> (
    ALSReadModel,
    DependencyAssessmentResult,
    AssetInventoryResult,
    AssetResolutionResult,
) {
    let model = als_model(occurrence_count);
    let assets = vec![asset("asset0", 0, occurrence_count)];
    let assessment = assessment(assets.clone());
    let inventory = inventory(&[("/source/sample.wav", "sample.wav", "aaa")]);
    let resolution = resolution(&assets, &inventory.file_occurrences, &["auto_accepted"]);
    (model, assessment, inventory, resolution)
}

#[test]
fn valid_lab_input_builds_copy_and_rewrite_plan() {
    let (model, assessment, inventory, resolution) = valid_inputs(1);
    let plan = plan_package(
        &request("laboratory_rescue_rewrite"),
        &model,
        &assessment,
        &inventory,
        &resolution,
    );

    assert_eq!(plan.plan_status, "ready_for_laboratory_execution");
    assert_eq!(plan.directory_operations.len(), 3);
    assert!(plan.directory_operations.iter().any(|operation| {
        operation.target_relative_path == Path::new("Ableton Project Info")
            && operation.purpose == "ableton_project_marker"
    }));
    assert_eq!(plan.copy_operations.len(), 2);
    assert_eq!(plan.rewrite_operations.len(), 1);
    assert_eq!(
        plan.rewrite_operations[0].fields_to_change,
        vec!["Path", "RelativePath", "RelativePathType"]
    );
}

#[test]
fn duplicate_occurrences_share_audio_copy() {
    let (model, assessment, inventory, resolution) = valid_inputs(2);
    let plan = plan_package(
        &request("laboratory_rescue_rewrite"),
        &model,
        &assessment,
        &inventory,
        &resolution,
    );

    assert_eq!(plan.copy_operations.len(), 2);
    assert_eq!(plan.rewrite_operations.len(), 2);
}

#[test]
fn unresolved_decision_blocks_plan() {
    let model = als_model(1);
    let assets = vec![asset("asset0", 0, 1)];
    let assessment = assessment(assets.clone());
    let inventory = inventory(&[("/source/sample.wav", "sample.wav", "aaa")]);
    let resolution = resolution(&assets, &inventory.file_occurrences, &["unresolved"]);
    let plan = plan_package(
        &request("laboratory_rescue_rewrite"),
        &model,
        &assessment,
        &inventory,
        &resolution,
    );

    assert_eq!(plan.plan_status, "blocked");
    assert_eq!(plan.unresolved_requirements.len(), 1);
}

#[test]
fn missing_current_path_produces_non_blocking_incomplete_plan() {
    let model = als_model(1);
    let assets = vec![asset("asset0", 0, 1)];
    let assessment = assessment(assets.clone());
    let inventory = inventory(&[]);
    let resolution = resolution(&assets, &[], &["unresolved"]);
    let plan = plan_package(
        &request("current_paths_copy"),
        &model,
        &assessment,
        &inventory,
        &resolution,
    );

    assert_eq!(plan.plan_status, "ready_current_paths_incomplete");
    assert_eq!(plan.copy_operations.len(), 1);
    assert!(plan.rewrite_operations.is_empty());
    assert_eq!(plan.unresolved_requirements.len(), 1);
    assert_eq!(
        plan.unresolved_requirements[0].reason,
        "recorded_path_missing"
    );
    assert!(!plan.unresolved_requirements[0].blocks_execution);
}

#[test]
fn available_current_path_produces_copy_and_rewrite() {
    let (model, assessment, inventory, resolution) = valid_inputs(1);
    let plan = plan_package(
        &request("current_paths_copy"),
        &model,
        &assessment,
        &inventory,
        &resolution,
    );

    assert_eq!(plan.plan_status, "ready_current_paths_complete");
    assert_eq!(plan.copy_operations.len(), 2);
    assert_eq!(plan.rewrite_operations.len(), 1);
    assert!(plan.unresolved_requirements.is_empty());
}

#[test]
fn metadata_only_current_path_plan_has_no_audio_hash_or_content_id() {
    let model = als_model(1);
    let assets = vec![asset("asset0", 0, 1)];
    let assessment = assessment(assets);
    let source_path = native_absolute(&["source", "sample0.wav"]);
    let bindings = CurrentPathBindingResult {
        metadata: CurrentPathBindingMetadata {
            binding_version: CURRENT_PATH_BINDING_VERSION.to_string(),
            policy_version: CURRENT_PATH_BINDING_POLICY_VERSION.to_string(),
            input_assessment_version: "0.1.0".to_string(),
            source_file_hash: "als-hash".to_string(),
            required_asset_count: 1,
            binding_count: 1,
            omission_count: 0,
            error_count: 0,
        },
        bindings: vec![CurrentPathBinding {
            required_asset_id: "asset0".to_string(),
            candidate_id: "candidate0".to_string(),
            source_path,
            filename: "sample0.wav".to_string(),
            observed_size: 100,
            decision_basis: "current_recorded_path_metadata_binding".to_string(),
        }],
        omissions: Vec::new(),
        errors: Vec::new(),
    };

    let plan = plan_current_path_package(
        &request("current_paths_copy"),
        &model,
        &assessment,
        &bindings,
    );
    let als = plan
        .copy_operations
        .iter()
        .find(|operation| operation.operation_kind == "copy_als")
        .expect("ALS operation");
    let audio = plan
        .copy_operations
        .iter()
        .find(|operation| operation.operation_kind == "copy_audio")
        .expect("audio operation");

    assert_eq!(plan.plan_status, "ready_current_paths_complete");
    assert!(als.expected_source_sha256.is_some());
    assert_eq!(audio.expected_source_sha256, None);
    assert_eq!(audio.content_id, None);
    assert_eq!(
        audio.verification_policy,
        rescue_packaging::VERIFY_STABLE_SOURCE_AND_SIZE
    );
}

#[test]
fn confirmed_core_library_dependency_is_left_system_managed_without_blocking() {
    let mut model = als_model(1);
    model.active_audio_references[0].relative_path_type = Some("5".to_string());
    model.active_audio_references[0].is_rewrite_candidate = false;
    model.active_audio_references[0].rewrite_support_status = "requires_test".to_string();
    let mut system_asset = asset("asset0", 0, 1);
    system_asset.source_category = "ableton_core_library".to_string();
    system_asset.management_class = "system_dependency".to_string();
    system_asset.source_classification_status = "confirmed".to_string();
    system_asset.source_classification_basis =
        "macos_core_library_path_relative_type_5_and_regular_file".to_string();
    let assessment = assessment(vec![system_asset]);
    let source_path = native_absolute(&["Applications", "Ableton Core Library", "sample.wav"]);
    let bindings = CurrentPathBindingResult {
        metadata: CurrentPathBindingMetadata {
            binding_version: CURRENT_PATH_BINDING_VERSION.to_string(),
            policy_version: CURRENT_PATH_BINDING_POLICY_VERSION.to_string(),
            input_assessment_version: "0.2.0".to_string(),
            source_file_hash: "als-hash".to_string(),
            required_asset_count: 1,
            binding_count: 1,
            omission_count: 0,
            error_count: 0,
        },
        bindings: vec![CurrentPathBinding {
            required_asset_id: "asset0".to_string(),
            candidate_id: "candidate0".to_string(),
            source_path,
            filename: "sample.wav".to_string(),
            observed_size: 100,
            decision_basis: "current_recorded_path_metadata_binding".to_string(),
        }],
        omissions: Vec::new(),
        errors: Vec::new(),
    };

    let plan = plan_current_path_package(
        &request("current_paths_copy"),
        &model,
        &assessment,
        &bindings,
    );

    assert_eq!(plan.plan_status, "ready_current_paths_complete");
    assert_eq!(plan.system_dependencies.len(), 1);
    assert_eq!(plan.metadata.system_dependency_count, 1);
    assert_eq!(
        plan.system_dependencies[0].package_action,
        "leave_system_managed"
    );
    assert!(plan.unresolved_requirements.is_empty());
    assert!(plan.rewrite_operations.is_empty());
    assert!(plan
        .copy_operations
        .iter()
        .all(|operation| operation.operation_kind == "copy_als"));
}

#[test]
fn project_local_type3_preserves_target_and_changes_path_only() {
    let mut model = als_model(1);
    model.active_audio_references[0] =
        project_local_ref(0, "Samples/Processed/Consolidate/sample.wav");
    let assets = vec![asset("asset0", 0, 1)];
    let assessment = assessment(assets.clone());
    let inventory = inventory(&[(
        "/source/Project/Samples/Processed/Consolidate/sample.wav",
        "sample.wav",
        "aaa",
    )]);
    let resolution = resolution(&assets, &inventory.file_occurrences, &["auto_accepted"]);

    let plan = plan_package(
        &request("current_paths_copy"),
        &model,
        &assessment,
        &inventory,
        &resolution,
    );

    assert_eq!(plan.plan_status, "ready_current_paths_complete");
    assert_eq!(plan.rewrite_operations[0].fields_to_change, vec!["Path"]);
    assert_eq!(
        plan.copy_operations[1].target_relative_path,
        Path::new("Samples/Processed/Consolidate/sample.wav")
    );
    assert!(plan.directory_operations.iter().any(|operation| {
        operation.target_relative_path == Path::new("Samples/Processed/Consolidate")
    }));
}

#[test]
fn mixed_type1_and_type3_plan_has_no_orphan_audio_copy() {
    let mut model = als_model(2);
    model.active_audio_references[1] = project_local_ref(1, "Samples/Recorded/project-local.wav");
    let assets = vec![asset("asset0", 0, 2)];
    let assessment = assessment(assets.clone());
    let inventory = inventory(&[(
        "/source/Project/Samples/Recorded/project-local.wav",
        "sample.wav",
        "aaa",
    )]);
    let resolution = resolution(&assets, &inventory.file_occurrences, &["auto_accepted"]);

    let plan = plan_package(
        &request("current_paths_copy"),
        &model,
        &assessment,
        &inventory,
        &resolution,
    );

    assert_eq!(plan.plan_status, "ready_current_paths_complete");
    assert_eq!(plan.rewrite_operations.len(), 2);
    let audio_targets: Vec<_> = plan
        .copy_operations
        .iter()
        .filter(|operation| operation.operation_kind == "copy_audio")
        .map(|operation| operation.target_relative_path.clone())
        .collect();
    assert_eq!(audio_targets.len(), 2);
    assert!(audio_targets.contains(&PathBuf::from("Samples/Imported/sample.wav")));
    assert!(audio_targets.contains(&PathBuf::from("Samples/Recorded/project-local.wav")));
    assert!(plan
        .rewrite_operations
        .iter()
        .any(|rewrite| rewrite.new_relative_path == "Samples/Imported/sample.wav"));
    assert!(audio_targets.iter().all(|target| plan
        .rewrite_operations
        .iter()
        .any(|rewrite| Path::new(&rewrite.new_relative_path) == target.as_path())));
}

#[test]
fn external_relocation_uses_portable_als_path_separator() {
    let (model, assessment, inventory, resolution) = valid_inputs(1);
    let plan = plan_package(
        &request("current_paths_copy"),
        &model,
        &assessment,
        &inventory,
        &resolution,
    );

    assert_eq!(plan.rewrite_operations.len(), 1);
    assert_eq!(
        plan.rewrite_operations[0].new_relative_path,
        "Samples/Imported/sample.wav"
    );
    assert!(!plan.rewrite_operations[0].new_relative_path.contains('\\'));
}

#[test]
fn unsupported_existing_reference_blocks_without_audio_copy() {
    let (mut model, assessment, inventory, resolution) = valid_inputs(1);
    model.active_audio_references[0].relative_path_type = Some("5".to_string());

    let plan = plan_package(
        &request("current_paths_copy"),
        &model,
        &assessment,
        &inventory,
        &resolution,
    );

    assert_eq!(plan.plan_status, "blocked");
    assert!(plan
        .copy_operations
        .iter()
        .all(|operation| operation.operation_kind == "copy_als"));
    assert!(plan.unresolved_requirements[0].blocks_execution);
}

#[test]
fn unsafe_type3_relative_path_blocks_without_audio_copy() {
    let mut model = als_model(1);
    model.active_audio_references[0] = project_local_ref(0, "Samples/../outside/sample.wav");
    let assets = vec![asset("asset0", 0, 1)];
    let assessment = assessment(assets.clone());
    let inventory = inventory(&[("/source/outside/sample.wav", "sample.wav", "aaa")]);
    let resolution = resolution(&assets, &inventory.file_occurrences, &["auto_accepted"]);

    let plan = plan_package(
        &request("current_paths_copy"),
        &model,
        &assessment,
        &inventory,
        &resolution,
    );

    assert_eq!(plan.plan_status, "blocked");
    assert!(plan
        .copy_operations
        .iter()
        .all(|operation| operation.operation_kind == "copy_als"));
}

#[test]
fn safety_failure_still_blocks_current_paths_copy() {
    let (model, assessment, inventory, resolution) = valid_inputs(1);
    let mut unsafe_request = request("current_paths_copy");
    unsafe_request.target_project_root = native_absolute(&["source"]);
    let plan = plan_package(
        &unsafe_request,
        &model,
        &assessment,
        &inventory,
        &resolution,
    );

    assert_eq!(plan.plan_status, "blocked");
    assert!(plan
        .errors
        .iter()
        .any(|error| error.error_code == "PACKAGE_TARGET_EQUALS_SOURCE"));
}

#[test]
fn zero_reference_laboratory_plan_is_blocked() {
    let model = als_model(0);
    let assessment = assessment(Vec::new());
    let inventory = inventory(&[]);
    let resolution = resolution(&[], &[], &[]);
    let plan = plan_package(
        &request("laboratory_rescue_rewrite"),
        &model,
        &assessment,
        &inventory,
        &resolution,
    );

    assert_eq!(plan.plan_status, "blocked");
    assert!(plan
        .copy_operations
        .iter()
        .all(|operation| operation.operation_kind == "copy_als"));
    assert!(plan.rewrite_operations.is_empty());
    assert!(plan
        .errors
        .iter()
        .any(|error| error.error_code == "PACKAGE_REWRITE_OPERATIONS_EMPTY"));
}

#[test]
fn target_equal_to_source_is_rejected() {
    let (model, assessment, inventory, resolution) = valid_inputs(1);
    let mut request = request("copy_only");
    request.target_project_root = native_absolute(&["source"]);
    let plan = plan_package(&request, &model, &assessment, &inventory, &resolution);

    assert_eq!(plan.plan_status, "blocked");
    assert_eq!(plan.errors[0].error_code, "PACKAGE_TARGET_EQUALS_SOURCE");
}

#[test]
fn plan_fingerprint_is_order_independent() {
    let (model, assessment, inventory, resolution) = valid_inputs(2);
    let plan = plan_package(
        &request("current_paths_copy"),
        &model,
        &assessment,
        &inventory,
        &resolution,
    );
    let mut reordered = plan.clone();
    reordered.metadata.plan_id = "another-run:plan".to_string();
    reordered.directory_operations.reverse();
    reordered.copy_operations.reverse();
    reordered.rewrite_operations.reverse();
    for (index, operation) in reordered.copy_operations.iter_mut().enumerate() {
        operation.operation_id = format!("runtime-copy-{index}");
        operation.preconditions.reverse();
    }
    for (index, operation) in reordered.rewrite_operations.iter_mut().enumerate() {
        operation.operation_id = format!("runtime-rewrite-{index}");
        operation.fields_to_change.reverse();
    }

    assert_eq!(
        fingerprint_package_plan(&plan).expect("fingerprint"),
        fingerprint_package_plan(&reordered).expect("reordered fingerprint")
    );
}

#[test]
fn plan_fingerprint_changes_with_semantic_plan() {
    let (model, assessment, inventory, resolution) = valid_inputs(1);
    let plan = plan_package(
        &request("current_paths_copy"),
        &model,
        &assessment,
        &inventory,
        &resolution,
    );
    let mut changed = plan.clone();
    changed.copy_operations[1].expected_source_size += 1;

    assert_ne!(
        fingerprint_package_plan(&plan).expect("fingerprint"),
        fingerprint_package_plan(&changed).expect("changed fingerprint")
    );
}

#[test]
fn different_content_same_target_name_blocks() {
    let model = als_model(2);
    let assets = vec![asset("asset0", 0, 1), asset("asset1", 1, 1)];
    let assessment = assessment(assets.clone());
    let inventory = inventory(&[
        ("/source/A/sample.wav", "sample.wav", "aaa"),
        ("/source/B/sample.wav", "sample.wav", "bbb"),
    ]);
    let resolution = resolution(
        &assets,
        &inventory.file_occurrences,
        &["auto_accepted", "auto_accepted"],
    );
    let plan = plan_package(
        &request("laboratory_rescue_rewrite"),
        &model,
        &assessment,
        &inventory,
        &resolution,
    );

    assert_eq!(plan.plan_status, "blocked");
    assert!(plan
        .errors
        .iter()
        .any(|error| error.error_code == "PACKAGE_TARGET_COLLISION"));
}

#[test]
fn unsupported_live_version_blocks_rewrite() {
    let (mut model, assessment, inventory, resolution) = valid_inputs(1);
    model.set_metadata.ableton_minor_version = Some("12.0".to_string());
    let plan = plan_package(
        &request("laboratory_rescue_rewrite"),
        &model,
        &assessment,
        &inventory,
        &resolution,
    );

    assert_eq!(plan.plan_status, "blocked");
    assert_eq!(
        plan.errors[0].error_code,
        "PACKAGE_REWRITE_DOCUMENT_UNSUPPORTED"
    );
}

#[test]
fn unsupported_relative_path_type_blocks_rewrite() {
    let (mut model, assessment, inventory, resolution) = valid_inputs(1);
    model.active_audio_references[0].relative_path_type = Some("5".to_string());
    let plan = plan_package(
        &request("laboratory_rescue_rewrite"),
        &model,
        &assessment,
        &inventory,
        &resolution,
    );

    assert_eq!(plan.plan_status, "blocked");
    assert_eq!(
        plan.unresolved_requirements[0].reason,
        "rewrite_reference_not_supported"
    );
}

#[test]
fn unknown_rewrite_support_blocks_rewrite() {
    let (mut model, assessment, inventory, resolution) = valid_inputs(1);
    model.active_audio_references[0].usage_context = "unknown".to_string();
    model.active_audio_references[0].is_rewrite_candidate = false;
    model.active_audio_references[0].rewrite_support_status = "requires_test".to_string();
    let plan = plan_package(
        &request("laboratory_rescue_rewrite"),
        &model,
        &assessment,
        &inventory,
        &resolution,
    );

    assert_eq!(plan.plan_status, "blocked");
    assert_eq!(
        plan.unresolved_requirements[0].reason,
        "rewrite_reference_not_supported"
    );
    assert!(plan.rewrite_operations.is_empty());
}

#[test]
fn outdated_resolution_policy_blocks_plan() {
    let (model, assessment, inventory, mut resolution) = valid_inputs(1);
    resolution.metadata.policy_version = "0.1.0".to_string();
    resolution.decisions[0].policy_version = "0.1.0".to_string();
    let plan = plan_package(
        &request("copy_only"),
        &model,
        &assessment,
        &inventory,
        &resolution,
    );

    assert_eq!(plan.plan_status, "blocked");
    assert!(plan
        .errors
        .iter()
        .any(|error| error.error_code == "PACKAGE_RESOLUTION_POLICY_UNSUPPORTED"));
}

#[test]
fn locator_mismatch_blocks_rewrite() {
    let (mut model, assessment, inventory, resolution) = valid_inputs(1);
    model.active_audio_references[0].xml_locator = "SampleRef[99]/FileRef".to_string();
    let plan = plan_package(
        &request("laboratory_rescue_rewrite"),
        &model,
        &assessment,
        &inventory,
        &resolution,
    );

    assert_eq!(plan.plan_status, "blocked");
    assert!(plan.rewrite_operations.is_empty());
}

#[test]
fn copy_only_mode_has_no_rewrite_operations() {
    let (model, assessment, inventory, resolution) = valid_inputs(1);
    let plan = plan_package(
        &request("copy_only"),
        &model,
        &assessment,
        &inventory,
        &resolution,
    );

    assert_eq!(plan.plan_status, "ready_copy_only");
    assert!(plan.rewrite_operations.is_empty());
    assert_eq!(plan.copy_operations.len(), 2);
}

#[test]
fn package_plan_is_deterministic() {
    let (model, assessment, inventory, resolution) = valid_inputs(1);
    let request = request("laboratory_rescue_rewrite");
    assert_eq!(
        plan_package(&request, &model, &assessment, &inventory, &resolution),
        plan_package(&request, &model, &assessment, &inventory, &resolution)
    );
}

#[test]
fn package_planner_is_pure() {
    let (model, assessment, inventory, resolution) = valid_inputs(1);
    let before = (
        model.clone(),
        assessment.clone(),
        inventory.clone(),
        resolution.clone(),
    );
    let _ = plan_package(
        &request("laboratory_rescue_rewrite"),
        &model,
        &assessment,
        &inventory,
        &resolution,
    );

    assert_eq!(before, (model, assessment, inventory, resolution));
}
