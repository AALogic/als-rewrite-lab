use rescue_analyzer::{
    DependencyAssessmentError, DependencyAssessmentMetadata, DependencyAssessmentResult,
    RequiredAsset, RequiredAssetCandidateObservation,
};
use rescue_resolution::{
    bind_current_paths, CURRENT_PATH_BINDING_POLICY_VERSION, CURRENT_PATH_BINDING_VERSION,
};

fn assessment(candidates: Vec<RequiredAssetCandidateObservation>) -> DependencyAssessmentResult {
    DependencyAssessmentResult {
        assessment_metadata: DependencyAssessmentMetadata {
            assessment_version: "0.2.0".to_string(),
            input_dependency_ref_version: "0.1.0".to_string(),
            input_path_observation_model_version: "0.2.0".to_string(),
            source_als_path: "/project/Set.als".to_string(),
            source_file_hash: "als-hash".to_string(),
            occurrence_count: 1,
            required_asset_count: 1,
            regular_file_candidate_asset_count: 1,
            missing_candidate_asset_count: 0,
            unknown_asset_count: 0,
            warning_count: 0,
            error_count: 0,
        },
        required_assets: vec![RequiredAsset {
            required_asset_id: "asset0".to_string(),
            grouping_basis: "complete_claim".to_string(),
            dependency_ids: vec!["dep0".to_string()],
            als_ref_ids: vec![0],
            occurrence_count: 1,
            filename: Some("sample.wav".to_string()),
            extension: Some("wav".to_string()),
            original_file_size: Some("5".to_string()),
            original_crc: Some("101".to_string()),
            source_category: "unclassified".to_string(),
            management_class: "unclassified".to_string(),
            source_classification_status: "unknown".to_string(),
            source_classification_basis: "insufficient_source_category_evidence".to_string(),
            candidate_observations: candidates,
            availability_status: "candidate_observed".to_string(),
            resolution_status: "unresolved".to_string(),
            risk_flags: Vec::new(),
            evidence_status: "observed".to_string(),
        }],
        warnings: Vec::new(),
        errors: Vec::new(),
    }
}

fn candidate(
    id: &str,
    path: &str,
    status: &str,
    size_status: &str,
) -> RequiredAssetCandidateObservation {
    RequiredAssetCandidateObservation {
        dependency_id: "dep0".to_string(),
        als_ref_id: 0,
        candidate_id: id.to_string(),
        candidate_basis: "raw_path".to_string(),
        candidate_path: path.to_string(),
        platform_status: "checkable_on_current_platform".to_string(),
        safety_status: "safe_for_metadata_read".to_string(),
        availability_status: status.to_string(),
        entry_kind: if status == "existing_regular_file" {
            "regular_file"
        } else {
            "unknown"
        }
        .to_string(),
        size_evidence_status: size_status.to_string(),
        observed_file_size: (status == "existing_regular_file").then_some(5),
        expected_file_size: Some(5),
    }
}

#[test]
fn exact_current_path_becomes_metadata_binding_without_content_identity() {
    let input = assessment(vec![candidate(
        "candidate0",
        "/project/sample.wav",
        "existing_regular_file",
        "matches_expected_size",
    )]);
    let result = bind_current_paths(&input);

    assert_eq!(
        result.metadata.binding_version,
        CURRENT_PATH_BINDING_VERSION
    );
    assert_eq!(
        result.metadata.policy_version,
        CURRENT_PATH_BINDING_POLICY_VERSION
    );
    assert_eq!(result.bindings.len(), 1);
    assert_eq!(result.bindings[0].observed_size, 5);
    assert_eq!(
        result.bindings[0].decision_basis,
        "current_recorded_path_metadata_binding"
    );
    assert!(result.omissions.is_empty());
    assert!(result.errors.is_empty());
}

#[test]
fn missing_current_path_is_a_non_blocking_omission() {
    let input = assessment(vec![candidate(
        "candidate0",
        "/project/sample.wav",
        "missing",
        "not_applicable",
    )]);
    let result = bind_current_paths(&input);

    assert!(result.bindings.is_empty());
    assert_eq!(result.omissions[0].reason, "recorded_path_missing");
    assert!(!result.omissions[0].blocks_execution);
}

#[test]
fn conflicting_size_blocks_metadata_binding() {
    let input = assessment(vec![candidate(
        "candidate0",
        "/project/sample.wav",
        "existing_regular_file",
        "differs_from_expected_size",
    )]);
    let result = bind_current_paths(&input);

    assert!(result.bindings.is_empty());
    assert_eq!(result.omissions[0].reason, "recorded_path_size_conflict");
    assert!(result.omissions[0].blocks_execution);
}

#[test]
fn multiple_current_paths_block_without_content_identity() {
    let input = assessment(vec![
        candidate(
            "candidate0",
            "/project/a/sample.wav",
            "existing_regular_file",
            "matches_expected_size",
        ),
        candidate(
            "candidate1",
            "/project/b/sample.wav",
            "existing_regular_file",
            "matches_expected_size",
        ),
    ]);
    let result = bind_current_paths(&input);

    assert!(result.bindings.is_empty());
    assert_eq!(
        result.omissions[0].reason,
        "multiple_current_paths_without_content_identity"
    );
    assert!(result.omissions[0].blocks_execution);
}

#[cfg(windows)]
#[test]
fn equivalent_windows_path_spellings_form_one_current_binding() {
    let input = assessment(vec![
        candidate(
            "candidate0",
            r"C:\Project\Samples\Recorded\sample.wav",
            "existing_regular_file",
            "matches_expected_size",
        ),
        candidate(
            "candidate1",
            r"C:/Project/Samples/Recorded/sample.wav",
            "existing_regular_file",
            "matches_expected_size",
        ),
        candidate(
            "candidate2",
            r"\\?\C:\Project\Samples\Recorded\sample.wav",
            "existing_regular_file",
            "matches_expected_size",
        ),
    ]);
    let result = bind_current_paths(&input);

    assert_eq!(result.bindings.len(), 1);
    assert!(result.omissions.is_empty());
    assert!(result.errors.is_empty());
}

#[test]
fn untrusted_assessment_fails_closed() {
    let mut input = assessment(Vec::new());
    input.errors.push(DependencyAssessmentError {
        error_code: "TEST".to_string(),
        message: "untrusted".to_string(),
        source_file_hash: "als-hash".to_string(),
    });
    let result = bind_current_paths(&input);

    assert!(result.bindings.is_empty());
    assert!(result.omissions.is_empty());
    assert_eq!(
        result.errors[0].error_code,
        "CURRENT_PATH_BINDING_UNTRUSTED_INPUT"
    );
}
