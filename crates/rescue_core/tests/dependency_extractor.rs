mod support;

use rescue_core::{
    analyze_als, extract_dependencies, ALSReadError, ALSReadModel, ALSReadWarning,
    ActiveAudioReference, DependencyExtractionResult, DependencyRef, HistoricalReference,
    NonAudioDependencySignal, SetMetadata,
};

use support::synthetic_als;

fn metadata(version: &str) -> SetMetadata {
    SetMetadata {
        source_als_path: "/fixture/test.als".to_string(),
        source_als_filename: Some("test.als".to_string()),
        source_project_root: None,
        source_file_size: 100,
        source_file_hash: "sha256-test".to_string(),
        analysis_started_at: "static-start".to_string(),
        analysis_completed_at: "static-end".to_string(),
        reader_version: "0.2".to_string(),
        als_read_model_version: version.to_string(),
        ableton_document_version: None,
        ableton_creator_version: None,
        ableton_minor_version: None,
        ableton_schema_change_count: None,
        decompressed_xml_size: 10,
        xml_root_name: "Ableton".to_string(),
        sample_ref_count: 0,
        active_audio_ref_count: 0,
        historical_ref_count: 0,
        non_audio_signal_count: 0,
        warning_count: 0,
        error_count: 0,
    }
}

fn active_ref(
    ref_id: usize,
    raw_path: Option<&str>,
    raw_relative_path: Option<&str>,
) -> ActiveAudioReference {
    ActiveAudioReference {
        ref_id,
        source_kind: "sample_ref_file_ref".to_string(),
        raw_path: raw_path.map(str::to_string),
        raw_relative_path: raw_relative_path.map(str::to_string),
        relative_path_type: Some("3".to_string()),
        file_type: Some("Wav".to_string()),
        filename: Some("Kick.wav".to_string()),
        extension: Some("wav".to_string()),
        original_file_size: Some("00123".to_string()),
        original_crc: Some("58123".to_string()),
        default_duration: Some("44.100".to_string()),
        default_sample_rate: Some("44100".to_string()),
        usage_context: "unknown".to_string(),
        xml_context: "SampleRef/FileRef".to_string(),
        xml_locator: format!("SampleRef[{ref_id}]/FileRef"),
        is_rewrite_candidate: false,
        rewrite_support_status: "requires_test".to_string(),
        warnings: Vec::new(),
    }
}

fn historical_ref(ref_id: usize) -> HistoricalReference {
    HistoricalReference {
        ref_id,
        source_kind: "original_file_ref".to_string(),
        raw_path: Some("/old/Kick.wav".to_string()),
        raw_relative_path: None,
        relative_path_type: Some("1".to_string()),
        filename: Some("Kick.wav".to_string()),
        extension: Some("wav".to_string()),
        original_file_size: Some("123".to_string()),
        original_crc: Some("58123".to_string()),
        xml_context: "SourceContext/OriginalFileRef".to_string(),
        xml_locator: format!("OriginalFileRef[{ref_id}]"),
        linked_active_ref_id: None,
        usage_note: "historical_provenance".to_string(),
        warnings: Vec::new(),
    }
}

fn non_audio_signal(signal_id: usize) -> NonAudioDependencySignal {
    NonAudioDependencySignal {
        signal_id,
        signal_kind: "plugin_signal".to_string(),
        name: Some("Example".to_string()),
        raw_path: None,
        raw_relative_path: None,
        relative_path_type: None,
        plugin_format: Some("vst3".to_string()),
        plugin_identifier: Some("example".to_string()),
        plugin_version: None,
        xml_context: "PluginDevice".to_string(),
        xml_locator: format!("PluginDevice[{signal_id}]"),
        support_status: "report_only".to_string(),
        warnings: Vec::new(),
    }
}

fn reader_warning(id: usize) -> ALSReadWarning {
    ALSReadWarning {
        warning_id: id,
        warning_code: "ALS_TEST_WARNING".to_string(),
        severity: "warning".to_string(),
        message: "test warning".to_string(),
        xml_context: Some("SampleRef/FileRef".to_string()),
        related_ref_id: Some(id),
        evidence_status: "observed".to_string(),
    }
}

fn model_with_refs(active_audio_references: Vec<ActiveAudioReference>) -> ALSReadModel {
    let mut model = ALSReadModel {
        set_metadata: metadata("0.2"),
        active_audio_references,
        historical_refs: Vec::new(),
        non_audio_dependency_signals: Vec::new(),
        warnings: Vec::new(),
        errors: Vec::new(),
    };
    model.set_metadata.active_audio_ref_count = model.active_audio_references.len();
    model
}

fn assert_no_forbidden_downstream_fields(result: &DependencyExtractionResult) {
    let json = serde_json::to_string(result).expect("result should serialize");
    for forbidden in [
        "resolved_path",
        "existence_status",
        "source_category",
        "risk_flags",
        "match_candidates",
        "copy_decision",
        "rewrite_decision",
    ] {
        assert!(
            !json.contains(forbidden),
            "result should not contain forbidden field {forbidden}"
        );
    }
}

#[test]
fn valid_model_extracts_audio_dependencies() {
    let fixture = synthetic_als("dependency_extractor_eleven_refs", &["1"; 11], 0);
    let model = analyze_als(fixture.path()).expect("fixture should parse");
    let result = extract_dependencies(&model);

    assert_eq!(result.errors, Vec::new());
    assert_eq!(result.dependencies.len(), 11);
    assert_eq!(result.extraction_metadata.dependency_count, 11);
    assert_eq!(result.dependencies[0].dependency_id, "dep_audio_000000");
    assert_eq!(result.dependencies[10].dependency_id, "dep_audio_000010");
    assert_eq!(result.dependencies[0].dependency_kind, "audio_sample");
    assert_eq!(
        result.dependencies[0].raw_path,
        model.active_audio_references[0].raw_path
    );
}

#[test]
fn zero_active_refs_is_valid() {
    let fixture = synthetic_als("dependency_extractor_zero_active", &[], 6);
    let model = analyze_als(fixture.path()).expect("fixture should parse");
    let result = extract_dependencies(&model);

    assert!(result.dependencies.is_empty());
    assert!(result.errors.is_empty());
    assert_eq!(result.extraction_metadata.dependency_count, 0);
    assert_eq!(
        result.ignored_input_summary.historical_refs_ignored_count,
        model.historical_refs.len()
    );
}

#[test]
fn one_active_ref_becomes_one_dependency_ref() {
    let model = model_with_refs(vec![active_ref(
        7,
        Some("/Samples/Kick.wav"),
        Some("Samples/Kick.wav"),
    )]);
    let result = extract_dependencies(&model);
    let dependency = &result.dependencies[0];

    assert_eq!(result.dependencies.len(), 1);
    assert_eq!(dependency.dependency_id, "dep_audio_000000");
    assert_eq!(dependency.als_ref_id, 7);
    assert_eq!(dependency.path_basis, "raw_path_and_raw_relative_path");
    assert_eq!(dependency.extraction_status, "extracted");
    assert!(dependency
        .evidence_notes
        .contains(&"original_crc_weak_identity_signal".to_string()));
}

#[test]
fn duplicate_refs_are_not_deduplicated() {
    let model = model_with_refs(vec![
        active_ref(0, Some("/Samples/Kick.wav"), Some("Samples/Kick.wav")),
        active_ref(1, Some("/Samples/Kick.wav"), Some("Samples/Kick.wav")),
    ]);
    let result = extract_dependencies(&model);

    assert_eq!(result.dependencies.len(), 2);
    assert_eq!(
        result.dependencies[0].raw_path,
        result.dependencies[1].raw_path
    );
    assert_ne!(
        result.dependencies[0].dependency_id,
        result.dependencies[1].dependency_id
    );
}

#[test]
fn incomplete_ref_is_preserved_with_warning() {
    let mut active = active_ref(3, None, None);
    active.filename = None;
    active.relative_path_type = None;
    let model = model_with_refs(vec![active]);
    let result = extract_dependencies(&model);
    let dependency = &result.dependencies[0];

    assert_eq!(result.dependencies.len(), 1);
    assert_eq!(dependency.extraction_status, "incomplete");
    assert_eq!(dependency.path_basis, "path_unavailable");
    assert!(dependency
        .warnings
        .iter()
        .any(|warning| warning.warning_code == "DEPENDENCY_PATH_UNAVAILABLE"));
    assert!(result
        .warnings
        .iter()
        .any(|warning| warning.warning_code == "DEPENDENCY_FILENAME_MISSING"));
}

#[test]
fn historical_refs_are_ignored_with_summary() {
    let mut model = model_with_refs(Vec::new());
    model.historical_refs = vec![historical_ref(0), historical_ref(1)];
    let result = extract_dependencies(&model);

    assert!(result.dependencies.is_empty());
    assert_eq!(
        result.ignored_input_summary.historical_refs_ignored_count,
        2
    );
    assert!(result
        .ignored_input_summary
        .ignored_scope_notes
        .contains(&"historical_refs_ignored_by_v0_1_scope".to_string()));
}

#[test]
fn non_audio_signals_are_ignored_with_summary() {
    let mut model = model_with_refs(Vec::new());
    model.non_audio_dependency_signals = vec![non_audio_signal(0), non_audio_signal(1)];
    let result = extract_dependencies(&model);

    assert!(result.dependencies.is_empty());
    assert_eq!(
        result
            .ignored_input_summary
            .non_audio_dependency_signals_ignored_count,
        2
    );
    assert!(result
        .ignored_input_summary
        .ignored_scope_notes
        .contains(&"non_audio_signals_ignored_by_v0_1_scope".to_string()));
}

#[test]
fn unsupported_als_read_model_version_returns_error() {
    let mut model = model_with_refs(vec![active_ref(
        0,
        Some("/Samples/Kick.wav"),
        Some("Samples/Kick.wav"),
    )]);
    model.set_metadata.als_read_model_version = "999".to_string();
    let result = extract_dependencies(&model);

    assert!(result.dependencies.is_empty());
    assert_eq!(result.errors.len(), 1);
    assert_eq!(
        result.errors[0].error_code,
        "DEPENDENCY_UNSUPPORTED_READ_MODEL_VERSION"
    );
    assert_eq!(result.extraction_metadata.error_count, 1);
}

#[test]
fn input_with_fatal_reader_errors_returns_no_trusted_model_error() {
    let mut model = model_with_refs(vec![active_ref(
        0,
        Some("/Samples/Kick.wav"),
        Some("Samples/Kick.wav"),
    )]);
    model.errors.push(ALSReadError {
        error_code: "ALS_XML_INVALID".to_string(),
        message: "invalid XML".to_string(),
        path: Some("/fixture/test.als".to_string()),
    });
    let result = extract_dependencies(&model);

    assert!(result.dependencies.is_empty());
    assert_eq!(result.errors.len(), 1);
    assert_eq!(
        result.errors[0].error_code,
        "DEPENDENCY_NO_TRUSTED_ALS_MODEL"
    );
}

#[test]
fn output_is_deterministic() {
    let mut model = model_with_refs(vec![
        active_ref(0, Some("/Samples/Kick.wav"), Some("Samples/Kick.wav")),
        active_ref(1, Some("/Samples/Snare.wav"), None),
    ]);
    model.active_audio_references[1]
        .warnings
        .push(reader_warning(1));

    let first = extract_dependencies(&model);
    let second = extract_dependencies(&model);

    assert_eq!(first, second);
    assert_eq!(first.dependencies[0].dependency_id, "dep_audio_000000");
    assert_eq!(first.dependencies[1].dependency_id, "dep_audio_000001");
}

#[test]
fn forbidden_downstream_fields_are_absent() {
    let model = model_with_refs(vec![active_ref(
        0,
        Some("/Samples/Kick.wav"),
        Some("Samples/Kick.wav"),
    )]);
    let result = extract_dependencies(&model);

    assert_no_forbidden_downstream_fields(&result);
}

#[test]
fn raw_scalar_values_remain_strings() {
    let model = model_with_refs(vec![active_ref(
        0,
        Some("/Samples/Kick.wav"),
        Some("Samples/Kick.wav"),
    )]);
    let result = extract_dependencies(&model);
    let dependency = &result.dependencies[0];
    let json = serde_json::to_value(dependency).expect("dependency should serialize");

    assert_eq!(dependency.original_file_size.as_deref(), Some("00123"));
    assert_eq!(dependency.original_crc.as_deref(), Some("58123"));
    assert_eq!(dependency.default_duration.as_deref(), Some("44.100"));
    assert!(json["original_file_size"].is_string());
    assert!(json["original_crc"].is_string());
    assert!(json["default_duration"].is_string());
}

#[test]
fn fake_path_observation_consumes_dependency_result() {
    fn fake_consumer(
        result: &DependencyExtractionResult,
    ) -> Vec<(String, Option<String>, Option<String>)> {
        result
            .dependencies
            .iter()
            .map(|dependency: &DependencyRef| {
                (
                    dependency.dependency_id.clone(),
                    dependency.raw_path.clone(),
                    dependency.raw_relative_path.clone(),
                )
            })
            .collect()
    }

    let model = model_with_refs(vec![active_ref(
        0,
        Some("/Samples/Kick.wav"),
        Some("Samples/Kick.wav"),
    )]);
    let result = extract_dependencies(&model);
    let consumed = fake_consumer(&result);

    assert_eq!(consumed.len(), 1);
    assert_eq!(consumed[0].0, "dep_audio_000000");
    assert_eq!(consumed[0].1.as_deref(), Some("/Samples/Kick.wav"));
}
