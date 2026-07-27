use rescue_analyzer::{DependencyAssessmentMetadata, DependencyAssessmentResult, RequiredAsset};
use rescue_catalog::{AssetInventoryMetadata, AssetInventoryResult, ContentRecord, FileOccurrence};
use rescue_core::{ALSReadModel, ActiveAudioReference, SetMetadata};
use rescue_packaging::{plan_package, PackagePlanningRequest};
use rescue_resolution::{AssetResolutionMetadata, AssetResolutionResult, ResolutionDecision};
use std::path::PathBuf;

fn active_ref(index: usize) -> ActiveAudioReference {
    ActiveAudioReference {
        ref_id: index,
        source_kind: "sample_ref".to_string(),
        raw_path: Some(format!("/source/sample{index}.wav")),
        raw_relative_path: Some(format!("../sample{index}.wav")),
        relative_path_type: Some("1".to_string()),
        file_type: Some("1".to_string()),
        filename: Some("sample.wav".to_string()),
        extension: Some("wav".to_string()),
        original_file_size: Some("100".to_string()),
        original_crc: Some("42".to_string()),
        default_duration: None,
        default_sample_rate: None,
        usage_context: "unknown".to_string(),
        xml_context: "Ableton/LiveSet/SampleRef/FileRef".to_string(),
        xml_locator: format!("SampleRef[{index}]/FileRef"),
        is_rewrite_candidate: false,
        rewrite_support_status: "requires_test".to_string(),
        warnings: Vec::new(),
    }
}

fn als_model(reference_count: usize) -> ALSReadModel {
    ALSReadModel {
        set_metadata: SetMetadata {
            source_als_path: "/source/Set.als".to_string(),
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
            assessment_version: "0.1.0".to_string(),
            input_dependency_ref_version: "0.1".to_string(),
            input_path_observation_model_version: "0.2".to_string(),
            source_als_path: "/source/Set.als".to_string(),
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
            source_root: PathBuf::from("/source"),
            native_path: PathBuf::from(path),
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
                policy_version: "0.1.0".to_string(),
                decision_basis: "fixture".to_string(),
                requires_user_confirmation: !accepted && statuses[index] != "unresolved",
            }
        })
        .collect();
    AssetResolutionResult {
        metadata: AssetResolutionMetadata {
            resolution_version: "0.1.0".to_string(),
            policy_version: "0.1.0".to_string(),
            input_assessment_version: "0.1.0".to_string(),
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
        target_project_root: PathBuf::from("/target/Project"),
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
fn target_equal_to_source_is_rejected() {
    let (model, assessment, inventory, resolution) = valid_inputs(1);
    let mut request = request("copy_only");
    request.target_project_root = PathBuf::from("/source");
    let plan = plan_package(&request, &model, &assessment, &inventory, &resolution);

    assert_eq!(plan.plan_status, "blocked");
    assert_eq!(plan.errors[0].error_code, "PACKAGE_TARGET_EQUALS_SOURCE");
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
