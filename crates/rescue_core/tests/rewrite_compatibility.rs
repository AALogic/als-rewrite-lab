use rescue_core::{assess_rewrite_compatibility, ALSReadModel, ActiveAudioReference, SetMetadata};

fn model(creator: &str, minor: &str, reference: ActiveAudioReference) -> ALSReadModel {
    ALSReadModel {
        set_metadata: SetMetadata {
            source_als_path: "/private/Secret Project.als".to_string(),
            source_als_filename: Some("Secret Project.als".to_string()),
            source_project_root: None,
            source_file_size: 100,
            source_file_hash: "als-hash".to_string(),
            analysis_started_at: "start".to_string(),
            analysis_completed_at: "end".to_string(),
            reader_version: "reader".to_string(),
            als_read_model_version: "model".to_string(),
            ableton_document_version: Some("5".to_string()),
            ableton_creator_version: Some(creator.to_string()),
            ableton_minor_version: Some(minor.to_string()),
            ableton_schema_change_count: Some("7".to_string()),
            decompressed_xml_size: 200,
            xml_root_name: "Ableton".to_string(),
            sample_ref_count: 1,
            active_audio_ref_count: 1,
            historical_ref_count: 0,
            non_audio_signal_count: 0,
            warning_count: 0,
            error_count: 0,
        },
        active_audio_references: vec![reference],
        historical_refs: Vec::new(),
        non_audio_dependency_signals: Vec::new(),
        warnings: Vec::new(),
        errors: Vec::new(),
    }
}

fn type1_reference(strict: bool) -> ActiveAudioReference {
    ActiveAudioReference {
        ref_id: 0,
        source_kind: "sample_ref_file_ref".to_string(),
        raw_path: Some("/private/Secret Sample.wav".to_string()),
        raw_relative_path: None,
        relative_path_type: Some("1".to_string()),
        file_type: Some("2".to_string()),
        filename: Some("Secret Sample.wav".to_string()),
        extension: Some("wav".to_string()),
        original_file_size: Some("4".to_string()),
        original_crc: Some("5".to_string()),
        default_duration: Some("1".to_string()),
        default_sample_rate: Some("44100".to_string()),
        usage_context: "audio_clip".to_string(),
        xml_context: "SampleRef/FileRef".to_string(),
        xml_locator: "SampleRef[0]/FileRef".to_string(),
        is_rewrite_candidate: strict,
        rewrite_support_status: if strict { "supported" } else { "requires_test" }.to_string(),
        warnings: Vec::new(),
    }
}

#[test]
fn strict_profile_still_marks_confirmed_document() {
    let result = assess_rewrite_compatibility(&model(
        "Ableton Live 11.3.43",
        "11.0_11300",
        type1_reference(true),
    ));

    assert_eq!(result.document_profile, "confirmed_live_11_3");
    assert_eq!(result.strict_supported_count, 1);
    assert_eq!(result.lab_compatible_count, 1);
}

#[test]
fn unconfirmed_version_with_known_shape_is_experimental_candidate() {
    let result = assess_rewrite_compatibility(&model(
        "Ableton Live 10.1.43",
        "10.0_10000",
        type1_reference(false),
    ));

    assert_eq!(result.document_profile, "unconfirmed_ableton_version");
    assert_eq!(result.evidence_status, "experimental_candidate");
    assert_eq!(result.strict_supported_count, 0);
    assert_eq!(result.lab_compatible_count, 1);
    assert!(result.references[0].reason_codes.is_empty());
}

#[test]
fn unknown_shape_is_not_lab_compatible() {
    let mut reference = type1_reference(false);
    reference.usage_context = "unknown".to_string();
    reference.relative_path_type = Some("9".to_string());
    let result =
        assess_rewrite_compatibility(&model("Ableton Live 10.1.43", "10.0_10000", reference));

    assert_eq!(result.lab_compatible_count, 0);
    assert_eq!(result.unsupported_shape_count, 1);
    assert!(result.references[0]
        .reason_codes
        .contains(&"unsupported_usage_context".to_string()));
    assert!(result.references[0]
        .reason_codes
        .contains(&"unsupported_relative_path_type".to_string()));
}

#[test]
fn compatibility_assessment_omits_paths_and_names() {
    let report = assess_rewrite_compatibility(&model(
        "Ableton Live 10.1.43",
        "10.0_10000",
        type1_reference(false),
    ));
    let json = serde_json::to_string(&report).expect("compatibility report JSON");

    assert!(!json.contains("Secret Project"));
    assert!(!json.contains("Secret Sample"));
    assert!(!json.contains("/private"));
}

#[test]
fn compatibility_assessment_rejects_path_like_version_metadata() {
    let report = assess_rewrite_compatibility(&model(
        "C:\\Users\\Private\\Ableton Live 10",
        "10.0_10000\n/private/project",
        type1_reference(false),
    ));
    let json = serde_json::to_string(&report).expect("compatibility report JSON");

    assert!(report.ableton_creator_version.is_none());
    assert!(report.ableton_minor_version.is_none());
    assert!(!json.contains("Users"));
    assert!(!json.contains("/private"));
}
