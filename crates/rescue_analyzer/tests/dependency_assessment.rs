use rescue_analyzer::{assess_dependencies, DependencyAssessmentResult};
use rescue_core::{
    CandidatePathObservation, DependencyExtractionMetadata, DependencyExtractionResult,
    DependencyPathObservation, DependencyRef, IgnoredInputSummary, PathObservationMetadata,
    PathObservationResult,
};

fn dependency(
    id: &str,
    path: Option<&str>,
    filename: Option<&str>,
    crc: Option<&str>,
) -> DependencyRef {
    DependencyRef {
        dependency_id: id.to_string(),
        dependency_kind: "audio_sample".to_string(),
        als_ref_id: id.trim_start_matches("dep").parse().unwrap_or(0),
        source_kind: "active_sample_ref".to_string(),
        raw_path: path.map(str::to_string),
        raw_relative_path: None,
        relative_path_type: Some("0".to_string()),
        file_type: Some("wav".to_string()),
        filename: filename.map(str::to_string),
        extension: Some("wav".to_string()),
        original_file_size: Some("100".to_string()),
        original_crc: crc.map(str::to_string),
        default_duration: None,
        default_sample_rate: None,
        usage_context: "unknown".to_string(),
        xml_context: "Ableton/LiveSet/SampleRef".to_string(),
        rewrite_support_status: "not_evaluated".to_string(),
        extraction_status: "complete".to_string(),
        path_basis: "raw_path".to_string(),
        evidence_status: "observed".to_string(),
        evidence_notes: Vec::new(),
        warnings: Vec::new(),
    }
}

fn extraction(dependencies: Vec<DependencyRef>) -> DependencyExtractionResult {
    DependencyExtractionResult {
        extraction_metadata: DependencyExtractionMetadata {
            extractor_version: "0.1".to_string(),
            dependency_ref_version: "0.1".to_string(),
            input_als_read_model_version: "0.2".to_string(),
            source_als_path: "/private/fixture.als".to_string(),
            source_project_root: None,
            source_file_hash: "snapshot-hash".to_string(),
            dependency_count: dependencies.len(),
            warning_count: 0,
            error_count: 0,
        },
        dependencies,
        ignored_input_summary: IgnoredInputSummary {
            historical_refs_ignored_count: 0,
            non_audio_dependency_signals_ignored_count: 0,
            input_warnings_seen_count: 0,
            input_errors_seen_count: 0,
            ignored_scope_notes: Vec::new(),
        },
        warnings: Vec::new(),
        errors: Vec::new(),
    }
}

fn candidate(dependency: &DependencyRef, status: &str, suffix: &str) -> CandidatePathObservation {
    CandidatePathObservation {
        candidate_id: format!("{}:{suffix}", dependency.dependency_id),
        candidate_basis: "recorded_raw_absolute_path".to_string(),
        candidate_path: dependency.raw_path.clone().unwrap_or_default(),
        platform_status: "checkable_on_current_platform".to_string(),
        safety_status: "accepted".to_string(),
        availability_status: status.to_string(),
        entry_kind: if status == "existing_regular_file" {
            "regular_file"
        } else {
            "unknown"
        }
        .to_string(),
        size_evidence_status: "matches_expected_size".to_string(),
        observed_file_size: (status == "existing_regular_file").then_some(100),
        expected_file_size: Some(100),
        evidence_notes: Vec::new(),
        warnings: Vec::new(),
    }
}

fn observations(
    extraction: &DependencyExtractionResult,
    statuses: &[Option<&str>],
) -> PathObservationResult {
    let dependency_observations: Vec<_> = extraction
        .dependencies
        .iter()
        .zip(statuses)
        .map(|(dependency, status)| {
            let candidates = status
                .map(|value| vec![candidate(dependency, value, "candidate")])
                .unwrap_or_default();
            DependencyPathObservation {
                dependency_id: dependency.dependency_id.clone(),
                als_ref_id: dependency.als_ref_id,
                raw_path: dependency.raw_path.clone(),
                raw_relative_path: dependency.raw_relative_path.clone(),
                parsed_raw_path: None,
                parsed_raw_relative_path: None,
                candidates,
                availability_summary: "fixture".to_string(),
                identity_status: "not_evaluated".to_string(),
                warnings: Vec::new(),
            }
        })
        .collect();
    PathObservationResult {
        observation_metadata: PathObservationMetadata {
            observer_version: "0.2.0".to_string(),
            path_observation_model_version: "0.2".to_string(),
            input_dependency_ref_version: "0.1".to_string(),
            source_als_path: extraction.extraction_metadata.source_als_path.clone(),
            source_file_hash: extraction.extraction_metadata.source_file_hash.clone(),
            project_root_basis: None,
            dependency_count: extraction.dependencies.len(),
            candidate_count: dependency_observations
                .iter()
                .map(|item| item.candidates.len())
                .sum(),
            regular_file_count: 0,
            missing_count: 0,
            unknown_count: 0,
            warning_count: 0,
            error_count: 0,
        },
        dependency_observations,
        warnings: Vec::new(),
        errors: Vec::new(),
    }
}

fn assess(
    dependencies: Vec<DependencyRef>,
    statuses: &[Option<&str>],
) -> DependencyAssessmentResult {
    let extraction = extraction(dependencies);
    let observations = observations(&extraction, statuses);
    assess_dependencies(&extraction, &observations)
}

#[test]
fn identical_complete_claims_group() {
    let first = dependency(
        "dep0",
        Some("Samples/Kick.wav"),
        Some("Kick.wav"),
        Some("9"),
    );
    let mut second = first.clone();
    second.dependency_id = "dep1".to_string();
    second.als_ref_id = 1;
    let result = assess(vec![first, second], &[Some("missing"), Some("missing")]);

    assert_eq!(result.required_assets.len(), 1);
    assert_eq!(result.required_assets[0].occurrence_count, 2);
    assert_eq!(
        result.required_assets[0].grouping_basis,
        "exact_recorded_reference_claim"
    );
}

#[test]
fn same_filename_different_path_does_not_group() {
    let result = assess(
        vec![
            dependency("dep0", Some("A/Kick.wav"), Some("Kick.wav"), Some("9")),
            dependency("dep1", Some("B/Kick.wav"), Some("Kick.wav"), Some("9")),
        ],
        &[Some("missing"), Some("missing")],
    );
    assert_eq!(result.required_assets.len(), 2);
}

#[test]
fn same_crc_different_path_does_not_group() {
    let result = assess(
        vec![
            dependency("dep0", Some("A/One.wav"), Some("One.wav"), Some("42")),
            dependency("dep1", Some("B/Two.wav"), Some("Two.wav"), Some("42")),
        ],
        &[Some("missing"), Some("missing")],
    );
    assert_eq!(result.required_assets.len(), 2);
}

#[test]
fn incomplete_occurrences_remain_separate() {
    let first = dependency("dep0", None, None, Some("42"));
    let mut second = first.clone();
    second.dependency_id = "dep1".to_string();
    second.als_ref_id = 1;
    let result = assess(vec![first, second], &[None, None]);

    assert_eq!(result.required_assets.len(), 2);
    assert!(result
        .required_assets
        .iter()
        .all(|item| item.grouping_basis == "single_incomplete_occurrence"));
}

#[test]
fn observed_file_candidate_does_not_resolve_asset() {
    let result = assess(
        vec![dependency(
            "dep0",
            Some("/audio/Kick.wav"),
            Some("Kick.wav"),
            Some("1"),
        )],
        &[Some("existing_regular_file")],
    );
    let asset = &result.required_assets[0];
    assert_eq!(asset.availability_status, "regular_file_candidate_observed");
    assert_eq!(asset.resolution_status, "unresolved");
}

#[test]
fn missing_candidates_are_assessed_conservatively() {
    let result = assess(
        vec![dependency(
            "dep0",
            Some("/gone/Kick.wav"),
            Some("Kick.wav"),
            Some("1"),
        )],
        &[Some("missing")],
    );
    assert_eq!(
        result.required_assets[0].availability_status,
        "no_regular_file_candidate_observed"
    );
    assert!(result.required_assets[0]
        .risk_flags
        .contains(&"required_audio_not_locally_observed".to_string()));
}

#[test]
fn snapshot_mismatch_fails_closed() {
    let extraction = extraction(vec![dependency("dep0", Some("A.wav"), Some("A.wav"), None)]);
    let mut observations = observations(&extraction, &[Some("missing")]);
    observations.observation_metadata.source_file_hash = "different".to_string();
    let result = assess_dependencies(&extraction, &observations);

    assert!(result.required_assets.is_empty());
    assert_eq!(result.errors[0].error_code, "ASSESSMENT_SNAPSHOT_MISMATCH");
}

#[test]
fn occurrence_handoff_mismatch_fails_closed() {
    let extraction = extraction(vec![dependency("dep0", Some("A.wav"), Some("A.wav"), None)]);
    let mut observations = observations(&extraction, &[Some("missing")]);
    observations.dependency_observations.clear();
    let result = assess_dependencies(&extraction, &observations);

    assert!(result.required_assets.is_empty());
    assert_eq!(
        result.errors[0].error_code,
        "ASSESSMENT_OCCURRENCE_CONTRACT_MISMATCH"
    );
}

#[test]
fn assessment_output_is_deterministic() {
    let extraction = extraction(vec![dependency("dep0", Some("A.wav"), Some("A.wav"), None)]);
    let observations = observations(&extraction, &[Some("missing")]);
    assert_eq!(
        assess_dependencies(&extraction, &observations),
        assess_dependencies(&extraction, &observations)
    );
}

#[test]
fn fake_preflight_consumer_uses_assessment_contract() {
    fn report_counts(result: &DependencyAssessmentResult) -> (usize, usize) {
        (
            result.assessment_metadata.required_asset_count,
            result.assessment_metadata.missing_candidate_asset_count,
        )
    }
    let result = assess(
        vec![dependency("dep0", Some("A.wav"), Some("A.wav"), None)],
        &[Some("missing")],
    );
    assert_eq!(report_counts(&result), (1, 1));
}

#[test]
fn assessment_does_not_touch_candidate_paths() {
    let path = "/definitely/not/a/real/path/fixture.wav";
    let result = assess(
        vec![dependency("dep0", Some(path), Some("fixture.wav"), None)],
        &[Some("existing_regular_file")],
    );
    assert_eq!(
        result.required_assets[0].candidate_observations[0].candidate_path,
        path
    );
}
