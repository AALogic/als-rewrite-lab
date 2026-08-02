use rescue_analyzer::{
    build_preflight_report, DependencyAssessmentError, DependencyAssessmentMetadata,
    DependencyAssessmentResult, ProjectDiscoveryMetadata, ProjectDiscoveryResult, RequiredAsset,
    RequiredAssetCandidateObservation,
};
use std::path::PathBuf;

fn discovery(path: &str) -> ProjectDiscoveryResult {
    ProjectDiscoveryResult {
        metadata: ProjectDiscoveryMetadata {
            discovery_version: "0.1.0".to_string(),
            source_als_path: PathBuf::from(path),
            ancestors_checked: 2,
            candidate_count: 1,
            warning_count: 0,
            error_count: 0,
        },
        candidates: Vec::new(),
        confirmed_project_root: Some(PathBuf::from("/project")),
        discovery_status: "confirmed".to_string(),
        set_location: "project_root".to_string(),
        warnings: Vec::new(),
        errors: Vec::new(),
    }
}

fn asset(index: usize, status: &str, path: Option<&str>) -> RequiredAsset {
    let candidate_observations = path
        .map(|candidate_path| {
            vec![RequiredAssetCandidateObservation {
                dependency_id: format!("dep{index}"),
                als_ref_id: index,
                candidate_id: format!("candidate{index}"),
                candidate_basis: "fixture".to_string(),
                candidate_path: candidate_path.to_string(),
                platform_status: "checkable_on_current_platform".to_string(),
                safety_status: "accepted".to_string(),
                availability_status: if status == "regular_file_candidate_observed" {
                    "existing_regular_file"
                } else {
                    "missing"
                }
                .to_string(),
                entry_kind: "regular_file".to_string(),
                size_evidence_status: "matches_expected_size".to_string(),
                observed_file_size: Some(100),
                expected_file_size: Some(100),
            }]
        })
        .unwrap_or_default();
    RequiredAsset {
        required_asset_id: format!("asset{index}"),
        grouping_basis: "exact_recorded_reference_claim".to_string(),
        dependency_ids: vec![format!("dep{index}")],
        als_ref_ids: vec![index],
        occurrence_count: 1,
        filename: Some(format!("sample{index}.wav")),
        extension: Some("wav".to_string()),
        original_file_size: Some("100".to_string()),
        original_crc: None,
        source_category: "unclassified".to_string(),
        management_class: "unclassified".to_string(),
        source_classification_status: "unknown".to_string(),
        source_classification_basis: "insufficient_source_category_evidence".to_string(),
        candidate_observations,
        availability_status: status.to_string(),
        resolution_status: "unresolved".to_string(),
        risk_flags: if status == "unknown" {
            vec!["availability_unknown".to_string()]
        } else {
            Vec::new()
        },
        evidence_status: "observed_unresolved".to_string(),
    }
}

fn assessment(statuses: &[(&str, Option<&str>)]) -> DependencyAssessmentResult {
    let required_assets: Vec<_> = statuses
        .iter()
        .enumerate()
        .map(|(index, (status, path))| asset(index, status, *path))
        .collect();
    let regular = required_assets
        .iter()
        .filter(|item| item.availability_status == "regular_file_candidate_observed")
        .count();
    let missing = required_assets
        .iter()
        .filter(|item| item.availability_status == "no_regular_file_candidate_observed")
        .count();
    let unknown = required_assets.len() - regular - missing;
    DependencyAssessmentResult {
        assessment_metadata: DependencyAssessmentMetadata {
            assessment_version: "0.2.0".to_string(),
            input_dependency_ref_version: "0.1".to_string(),
            input_path_observation_model_version: "0.2".to_string(),
            source_als_path: "/project/Set.als".to_string(),
            source_file_hash: "hash".to_string(),
            occurrence_count: required_assets.len(),
            required_asset_count: required_assets.len(),
            regular_file_candidate_asset_count: regular,
            missing_candidate_asset_count: missing,
            unknown_asset_count: unknown,
            warning_count: 0,
            error_count: 0,
        },
        required_assets,
        warnings: Vec::new(),
        errors: Vec::new(),
    }
}

#[test]
fn all_candidates_observed_remain_unresolved() {
    let report = build_preflight_report(
        &discovery("/project/Set.als"),
        &assessment(&[("regular_file_candidate_observed", Some("/audio/one.wav"))]),
    );
    assert_eq!(
        report.summary.overall_status,
        "candidates_observed_not_resolved"
    );
    assert_eq!(report.summary.unresolved_count, 1);
    assert_eq!(report.requirements[0].resolution_status, "unresolved");
}

#[test]
fn system_dependency_is_exposed_separately_from_missing_assets() {
    let mut input = assessment(&[(
        "regular_file_candidate_observed",
        Some("/Applications/Ableton/Core Library/sample.wav"),
    )]);
    let system = &mut input.required_assets[0];
    system.source_category = "ableton_core_library".to_string();
    system.management_class = "system_dependency".to_string();
    system.source_classification_status = "confirmed".to_string();
    system.source_classification_basis =
        "macos_core_library_path_relative_type_5_and_regular_file".to_string();
    system.risk_flags = vec!["portable_risk".to_string()];

    let report = build_preflight_report(&discovery("/project/Set.als"), &input);

    assert_eq!(report.summary.system_dependency_count, 1);
    assert_eq!(report.summary.needs_search_count, 0);
    assert_eq!(report.requirements[0].management_class, "system_dependency");
    assert_eq!(report.requirements[0].portability_status, "portable_risk");
}

#[test]
fn missing_requirement_requests_asset_search() {
    let report = build_preflight_report(
        &discovery("/project/Set.als"),
        &assessment(&[("no_regular_file_candidate_observed", Some("/gone/one.wav"))]),
    );
    assert_eq!(report.summary.overall_status, "needs_asset_search");
    assert_eq!(report.summary.needs_search_count, 1);
}

#[test]
fn unknown_requirement_requests_review() {
    let report = build_preflight_report(
        &discovery("/project/Set.als"),
        &assessment(&[("unknown", None)]),
    );
    assert_eq!(report.summary.overall_status, "needs_review");
    assert_eq!(report.summary.unknown_count, 1);
}

#[test]
fn untrusted_input_blocks_report() {
    let mut assessment = assessment(&[("unknown", None)]);
    assessment.errors.push(DependencyAssessmentError {
        error_code: "fixture".to_string(),
        message: "fixture".to_string(),
        source_file_hash: "hash".to_string(),
    });
    let report = build_preflight_report(&discovery("/project/Set.als"), &assessment);

    assert_eq!(report.summary.overall_status, "blocked");
    assert!(report.requirements.is_empty());
    assert_eq!(
        report.errors[0].error_code,
        "PREFLIGHT_UNTRUSTED_ASSESSMENT"
    );
}

#[test]
fn source_mismatch_blocks_report() {
    let report = build_preflight_report(
        &discovery("/other/Set.als"),
        &assessment(&[("unknown", None)]),
    );
    assert_eq!(report.errors[0].error_code, "PREFLIGHT_SOURCE_MISMATCH");
}

#[test]
fn local_candidate_paths_are_explainable() {
    let report = build_preflight_report(
        &discovery("/project/Set.als"),
        &assessment(&[(
            "regular_file_candidate_observed",
            Some("/private/audio.wav"),
        )]),
    );
    assert_eq!(
        report.requirements[0].candidate_paths,
        vec!["/private/audio.wav"]
    );
}

#[test]
fn preflight_output_is_deterministic() {
    let discovery = discovery("/project/Set.als");
    let assessment = assessment(&[("unknown", None)]);
    assert_eq!(
        build_preflight_report(&discovery, &assessment),
        build_preflight_report(&discovery, &assessment)
    );
}
