use rescue_analyzer::{
    DependencyAssessmentError, DependencyAssessmentMetadata, DependencyAssessmentResult,
    RequiredAsset, RequiredAssetCandidateObservation,
};
use rescue_catalog::{AssetInventoryMetadata, AssetInventoryResult, ContentRecord, FileOccurrence};
use rescue_resolution::{resolve_assets, AssetResolutionResult};
use std::path::PathBuf;

fn required_asset(
    filename: Option<&str>,
    size: Option<&str>,
    crc: Option<&str>,
    observed_paths: &[&str],
) -> RequiredAsset {
    RequiredAsset {
        required_asset_id: "asset0".to_string(),
        grouping_basis: "exact_recorded_reference_claim".to_string(),
        dependency_ids: vec!["dep0".to_string()],
        als_ref_ids: vec![0],
        occurrence_count: 1,
        filename: filename.map(str::to_string),
        extension: filename.and_then(|value| {
            value
                .rsplit_once('.')
                .map(|(_, extension)| extension.to_string())
        }),
        original_file_size: size.map(str::to_string),
        original_crc: crc.map(str::to_string),
        candidate_observations: observed_paths
            .iter()
            .enumerate()
            .map(|(index, path)| RequiredAssetCandidateObservation {
                dependency_id: "dep0".to_string(),
                als_ref_id: 0,
                candidate_id: format!("path{index}"),
                candidate_basis: "recorded_raw_absolute_path".to_string(),
                candidate_path: (*path).to_string(),
                platform_status: "checkable_on_current_platform".to_string(),
                safety_status: "safe_for_metadata_read".to_string(),
                availability_status: "existing_regular_file".to_string(),
                entry_kind: "regular_file".to_string(),
                size_evidence_status: "matches_expected_size".to_string(),
                observed_file_size: size.and_then(|value| value.parse().ok()),
                expected_file_size: size.and_then(|value| value.parse().ok()),
            })
            .collect(),
        availability_status: if observed_paths.is_empty() {
            "no_regular_file_candidate_observed"
        } else {
            "regular_file_candidate_observed"
        }
        .to_string(),
        resolution_status: "unresolved".to_string(),
        risk_flags: Vec::new(),
        evidence_status: "observed_unresolved".to_string(),
    }
}

fn assessment(asset: RequiredAsset) -> DependencyAssessmentResult {
    DependencyAssessmentResult {
        assessment_metadata: DependencyAssessmentMetadata {
            assessment_version: "0.1.0".to_string(),
            input_dependency_ref_version: "0.1".to_string(),
            input_path_observation_model_version: "0.2".to_string(),
            source_als_path: "/project/Set.als".to_string(),
            source_file_hash: "hash".to_string(),
            occurrence_count: 1,
            required_asset_count: 1,
            regular_file_candidate_asset_count: 0,
            missing_candidate_asset_count: 0,
            unknown_asset_count: 1,
            warning_count: 0,
            error_count: 0,
        },
        required_assets: vec![asset],
        warnings: Vec::new(),
        errors: Vec::new(),
    }
}

fn inventory(status: &str, files: &[(&str, &str, u64, &str)]) -> AssetInventoryResult {
    let file_occurrences: Vec<_> = files
        .iter()
        .enumerate()
        .map(|(index, (path, filename, size, digest))| FileOccurrence {
            file_occurrence_id: format!("occ{index}"),
            content_id: format!("sha256:{digest}"),
            source_root: PathBuf::from("/"),
            native_path: PathBuf::from(path),
            relative_path: PathBuf::from(path.trim_start_matches('/')),
            filename: (*filename).to_string(),
            extension: filename
                .rsplit_once('.')
                .map(|(_, extension)| extension)
                .unwrap_or("")
                .to_string(),
            file_size: *size,
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
            scan_status: status.to_string(),
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

#[test]
fn exact_path_name_and_size_requires_confirmation_without_expected_hash() {
    let assessment = assessment(required_asset(
        Some("kick.wav"),
        Some("100"),
        None,
        &["/audio/kick.wav"],
    ));
    let inventory = inventory("complete", &[("/audio/kick.wav", "kick.wav", 100, "aaa")]);
    let result = resolve_assets(&assessment, &inventory);

    assert_eq!(result.proposals[0].candidates[0].score, 100);
    assert_eq!(
        result.decisions[0].decision_status,
        "needs_user_confirmation"
    );
    assert_eq!(result.decisions[0].selected_candidate_id, None);
    assert!(result.decisions[0].requires_user_confirmation);
    assert_eq!(result.decisions[0].policy_version, "0.2.0");
    assert!(result
        .warnings
        .iter()
        .any(|warning| warning.warning_code == "RESOLUTION_STRONG_IDENTITY_REQUIRED"));
}

#[test]
fn path_observer_status_v0_2_is_consumed_without_translation() {
    let assessment = assessment(required_asset(
        Some("kick.wav"),
        Some("100"),
        None,
        &["/audio/kick.wav"],
    ));
    let inventory = inventory("complete", &[("/audio/kick.wav", "kick.wav", 100, "aaa")]);
    let result = resolve_assets(&assessment, &inventory);

    assert!(result.proposals[0].candidates[0]
        .evidence
        .iter()
        .any(|evidence| evidence.evidence_code == "exact_observed_native_path"));
    assert_eq!(
        result.decisions[0].decision_status,
        "needs_user_confirmation"
    );
}

#[test]
fn name_and_size_only_requires_confirmation() {
    let assessment = assessment(required_asset(Some("kick.wav"), Some("100"), None, &[]));
    let inventory = inventory(
        "complete",
        &[("/elsewhere/kick.wav", "kick.wav", 100, "aaa")],
    );
    let result = resolve_assets(&assessment, &inventory);

    assert!(result.proposals[0].candidates[0].score < 95);
    assert_eq!(
        result.decisions[0].decision_status,
        "needs_user_confirmation"
    );
    assert_eq!(result.decisions[0].selected_candidate_id, None);
}

#[test]
fn same_name_different_content_remains_distinct() {
    let assessment = assessment(required_asset(Some("kick.wav"), Some("100"), None, &[]));
    let inventory = inventory(
        "complete",
        &[
            ("/A/kick.wav", "kick.wav", 100, "aaa"),
            ("/B/kick.wav", "kick.wav", 100, "bbb"),
        ],
    );
    let result = resolve_assets(&assessment, &inventory);

    assert_eq!(result.proposals[0].candidates.len(), 2);
    assert_ne!(
        result.proposals[0].candidates[0].content_id,
        result.proposals[0].candidates[1].content_id
    );
    assert_eq!(
        result.decisions[0].decision_status,
        "needs_user_confirmation"
    );
}

#[test]
fn high_score_tie_blocks_automatic_resolution() {
    let assessment = assessment(required_asset(
        Some("kick.wav"),
        Some("100"),
        None,
        &["/A/kick.wav", "/B/kick.wav"],
    ));
    let inventory = inventory(
        "complete",
        &[
            ("/A/kick.wav", "kick.wav", 100, "aaa"),
            ("/B/kick.wav", "kick.wav", 100, "bbb"),
        ],
    );
    let result = resolve_assets(&assessment, &inventory);

    assert_eq!(
        result.decisions[0].decision_basis,
        "ambiguous_high_confidence_candidates"
    );
    assert_eq!(result.decisions[0].selected_candidate_id, None);
}

#[test]
fn no_candidate_remains_unresolved() {
    let assessment = assessment(required_asset(Some("wanted.wav"), Some("100"), None, &[]));
    let inventory = inventory("complete", &[("/audio/other.wav", "other.wav", 100, "aaa")]);
    let result = resolve_assets(&assessment, &inventory);

    assert!(result.proposals[0].candidates.is_empty());
    assert_eq!(result.decisions[0].decision_status, "unresolved");
}

#[test]
fn original_crc_alone_does_not_create_candidate() {
    let assessment = assessment(required_asset(None, None, Some("42"), &[]));
    let inventory = inventory("complete", &[("/audio/other.wav", "other.wav", 100, "aaa")]);
    let result = resolve_assets(&assessment, &inventory);

    assert!(result.proposals[0].candidates.is_empty());
    assert!(result
        .warnings
        .iter()
        .any(|warning| warning.warning_code == "RESOLUTION_ORIGINAL_CRC_NOT_SCORED"));
}

#[test]
fn partial_inventory_blocks_auto_acceptance() {
    let assessment = assessment(required_asset(
        Some("kick.wav"),
        Some("100"),
        None,
        &["/audio/kick.wav"],
    ));
    let inventory = inventory("partial", &[("/audio/kick.wav", "kick.wav", 100, "aaa")]);
    let result = resolve_assets(&assessment, &inventory);

    assert_eq!(result.proposals[0].candidates[0].score, 100);
    assert_eq!(
        result.decisions[0].decision_status,
        "needs_user_confirmation"
    );
}

#[test]
fn untrusted_upstream_fails_closed() {
    let mut assessment = assessment(required_asset(Some("kick.wav"), Some("100"), None, &[]));
    assessment.errors.push(DependencyAssessmentError {
        error_code: "fixture".to_string(),
        message: "fixture".to_string(),
        source_file_hash: "hash".to_string(),
    });
    let result = resolve_assets(&assessment, &inventory("complete", &[]));

    assert!(result.proposals.is_empty());
    assert_eq!(
        result.errors[0].error_code,
        "RESOLUTION_UNTRUSTED_ASSESSMENT"
    );
}

#[test]
fn resolution_output_is_deterministic() {
    let assessment = assessment(required_asset(Some("kick.wav"), Some("100"), None, &[]));
    let inventory = inventory(
        "complete",
        &[
            ("/B/kick.wav", "kick.wav", 100, "bbb"),
            ("/A/kick.wav", "kick.wav", 100, "aaa"),
        ],
    );
    assert_eq!(
        resolve_assets(&assessment, &inventory),
        resolve_assets(&assessment, &inventory)
    );
}

#[test]
fn fake_package_planner_receives_no_unconfirmed_selection() {
    fn accepted_sources(result: &AssetResolutionResult) -> Vec<(&str, &str)> {
        result
            .decisions
            .iter()
            .filter(|decision| decision.decision_status == "auto_accepted")
            .map(|decision| {
                (
                    decision.required_asset_id.as_str(),
                    decision
                        .selected_file_occurrence_id
                        .as_deref()
                        .unwrap_or("missing"),
                )
            })
            .collect()
    }

    let assessment = assessment(required_asset(
        Some("kick.wav"),
        Some("100"),
        None,
        &["/audio/kick.wav"],
    ));
    let inventory = inventory("complete", &[("/audio/kick.wav", "kick.wav", 100, "aaa")]);
    let result = resolve_assets(&assessment, &inventory);

    assert!(accepted_sources(&result).is_empty());
    assert_eq!(result.metadata.auto_accepted_count, 0);
    assert_eq!(result.metadata.manual_review_count, 1);
}
