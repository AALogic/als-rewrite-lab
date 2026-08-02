mod support;

use rescue_core::{analyze_als, ALSError, ALSReadModel, MAX_COMPRESSED_ALS_BYTES};
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use support::{gzip_als, raw_als, synthetic_als};

fn fixture_path(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/fixtures")
        .join(name)
}

fn minimal_ableton_with_file_ref(path: &str, relative_path: &str) -> String {
    format!(
        r#"<Ableton MajorVersion="5" MinorVersion="12" SchemaChangeCount="1" Creator="Ableton Live">
  <LiveSet>
    <SampleRef>
      <FileRef>
        <Path Value="{path}" />
        <RelativePath Value="{relative_path}" />
        <RelativePathType Value="1" />
        <Type Value="2" />
        <OriginalFileSize Value="123" />
        <OriginalCrc Value="456" />
      </FileRef>
      <DefaultDuration Value="44.1" />
      <DefaultSampleRate Value="44100" />
    </SampleRef>
  </LiveSet>
</Ableton>"#
    )
}

fn minimal_ableton_with_historical_and_non_audio_refs() -> &'static str {
    r#"<Ableton MajorVersion="5" MinorVersion="12" SchemaChangeCount="1" Creator="Ableton Live">
  <LiveSet>
    <SourceContext>
      <OriginalFileRef>
        <FileRef>
          <Path Value="/old/Kick.wav" />
        </FileRef>
      </OriginalFileRef>
    </SourceContext>
    <PluginDevice>
      <FileRef>
        <Path Value="/preset/Default.aupreset" />
      </FileRef>
    </PluginDevice>
  </LiveSet>
</Ableton>"#
}

fn live_11_3_audio_clip_with_file_ref() -> &'static str {
    r#"<Ableton MajorVersion="5" MinorVersion="11.0_11300" SchemaChangeCount="7" Creator="Ableton Live 11.3.43">
  <LiveSet>
    <AudioClip>
      <SampleRef>
        <FileRef>
          <Path Value="/external/kick.wav" />
          <RelativePath Value="../kick.wav" />
          <RelativePathType Value="1" />
          <Type Value="2" />
          <OriginalFileSize Value="123" />
          <OriginalCrc Value="456" />
        </FileRef>
        <DefaultDuration Value="44.1" />
        <DefaultSampleRate Value="44100" />
      </SampleRef>
    </AudioClip>
  </LiveSet>
</Ableton>"#
}

fn live_11_3_type3_reference(parent: &str, relative_path: &str) -> String {
    format!(
        r#"<Ableton MajorVersion="5" MinorVersion="11.0_11300" SchemaChangeCount="7" Creator="Ableton Live 11.3.43">
  <LiveSet>
    <{parent}>
      <SampleRef>
        <FileRef>
          <Path Value="/old/Project/{relative_path}" />
          <RelativePath Value="{relative_path}" />
          <RelativePathType Value="3" />
          <Type Value="2" />
          <OriginalFileSize Value="123" />
          <OriginalCrc Value="456" />
        </FileRef>
        <DefaultDuration Value="44.1" />
        <DefaultSampleRate Value="44100" />
      </SampleRef>
    </{parent}>
  </LiveSet>
</Ableton>"#
    )
}

fn relative_path_type_counts(model: &ALSReadModel) -> BTreeMap<String, usize> {
    let mut counts = BTreeMap::new();

    for active_ref in &model.active_audio_references {
        let key = active_ref
            .relative_path_type
            .clone()
            .unwrap_or_else(|| "<missing>".to_string());
        *counts.entry(key).or_insert(0) += 1;
    }

    counts
}

#[test]
fn valid_als_returns_read_model_json() {
    let fixture = synthetic_als("valid_read_model", &["1", "1", "5", "5"], 2);
    let model = analyze_als(fixture.path()).unwrap();

    assert_eq!(model.set_metadata.als_read_model_version, "0.2");
    assert_eq!(model.set_metadata.sample_ref_count, 4);
    assert_eq!(model.set_metadata.active_audio_ref_count, 4);
    assert_eq!(model.active_audio_references.len(), 4);
    assert_eq!(model.set_metadata.historical_ref_count, 2);
    assert_eq!(model.historical_refs.len(), 2);

    let json = serde_json::to_value(&model).unwrap();
    assert!(json.get("set_metadata").is_some());
    assert!(json.get("active_audio_references").is_some());
    assert!(json.get("historical_refs").is_some());
    assert!(json.get("non_audio_dependency_signals").is_some());
    assert!(json.get("warnings").is_some());
    assert!(json.get("errors").is_some());
}

#[test]
fn active_refs_are_not_historical_refs() {
    let template_fixture = synthetic_als("zero_active", &[], 6);
    let template = analyze_als(template_fixture.path()).unwrap();

    assert_eq!(template.set_metadata.sample_ref_count, 0);
    assert_eq!(template.set_metadata.active_audio_ref_count, 0);
    assert_eq!(template.active_audio_references.len(), 0);
    assert_eq!(template.set_metadata.historical_ref_count, 6);
    assert_eq!(template.historical_refs.len(), 6);
    assert!(template
        .historical_refs
        .iter()
        .all(|historical_ref| historical_ref.usage_note == "historical_provenance"));

    let active_fixture = synthetic_als("active_and_historical", &["1", "1", "5", "5"], 2);
    let active = analyze_als(active_fixture.path()).unwrap();

    assert_eq!(active.set_metadata.sample_ref_count, 4);
    assert_eq!(active.set_metadata.active_audio_ref_count, 4);
    assert_eq!(active.active_audio_references.len(), 4);
    assert_eq!(active.set_metadata.historical_ref_count, 2);
    assert_eq!(active.historical_refs.len(), 2);
}

#[test]
fn relative_path_type_is_captured_raw() {
    let before_fixture = synthetic_als("before_cas_shape", &["1", "1", "5", "5", "5"], 0);
    let after_fixture = synthetic_als("after_cas_shape", &["3", "3", "5", "5", "5"], 0);
    let before = analyze_als(before_fixture.path()).unwrap();
    let after = analyze_als(after_fixture.path()).unwrap();

    assert_eq!(
        relative_path_type_counts(&before),
        BTreeMap::from([("1".to_string(), 2), ("5".to_string(), 3)])
    );
    assert_eq!(
        relative_path_type_counts(&after),
        BTreeMap::from([("3".to_string(), 2), ("5".to_string(), 3)])
    );
}

#[test]
fn als_reader_v0_2_does_not_check_filesystem_paths() {
    let fixture = synthetic_als("no_filesystem_check", &["1"], 0);
    let model = analyze_als(fixture.path()).unwrap();
    let json = serde_json::to_value(&model).unwrap();

    assert!(json
        .pointer("/active_audio_references/0/raw_path")
        .is_some());
    assert!(json.to_string().contains("raw_path"));
    assert!(!json.to_string().contains("exists_on_disk"));
    assert!(!json.to_string().contains("source_category"));
    assert!(!json.to_string().contains("storage_state"));
    assert!(model.warnings.iter().all(|warning| {
        warning.warning_code != "active_path_missing_on_disk"
            && warning.warning_code != "file_path_does_not_exist"
    }));
}

#[test]
fn active_audio_refs_preserve_rewrite_relevant_fields() {
    let fixture = synthetic_als("rewrite_fields", &["1"], 0);
    let model = analyze_als(fixture.path()).unwrap();
    let first = model
        .active_audio_references
        .first()
        .expect("fixture should contain active audio refs");

    assert_eq!(first.source_kind, "sample_ref_file_ref");
    assert_eq!(first.usage_context, "unknown");
    assert_eq!(first.xml_context, "SampleRef/FileRef");
    assert!(first.xml_locator.starts_with("SampleRef["));
    assert!(first.raw_path.is_some() || first.raw_relative_path.is_some());
    assert_eq!(first.rewrite_support_status, "requires_test");
    assert!(!first.is_rewrite_candidate);
}

#[test]
fn live_11_3_audio_clip_reference_is_supported_for_laboratory_rewrite() {
    let fixture = gzip_als("live_11_3_audio_clip", live_11_3_audio_clip_with_file_ref());
    let model = analyze_als(fixture.path()).expect("fixture should parse");
    let reference = &model.active_audio_references[0];

    assert_eq!(reference.usage_context, "audio_clip");
    assert!(reference.is_rewrite_candidate);
    assert_eq!(reference.rewrite_support_status, "supported");
}

#[test]
fn live_11_3_project_local_type3_reference_is_supported_for_path_only_rewrite() {
    let xml = live_11_3_type3_reference("AudioClip", "Samples/Recorded/kick.wav");
    let fixture = gzip_als("live_11_3_project_local_type3", &xml);
    let model = analyze_als(fixture.path()).expect("fixture should parse");
    let reference = &model.active_audio_references[0];

    assert_eq!(reference.usage_context, "audio_clip");
    assert!(reference.is_rewrite_candidate);
    assert_eq!(reference.rewrite_support_status, "supported");
}

#[test]
fn live_11_3_multisample_type3_reference_is_supported_for_path_only_rewrite() {
    let xml = live_11_3_type3_reference("MultiSamplePart", "Samples/Recorded/kick.wav");
    let fixture = gzip_als("live_11_3_multisample_type3", &xml);
    let model = analyze_als(fixture.path()).expect("fixture should parse");
    let reference = &model.active_audio_references[0];

    assert_eq!(reference.usage_context, "simpler_multisample");
    assert!(reference.is_rewrite_candidate);
    assert_eq!(reference.rewrite_support_status, "supported");
}

#[test]
fn unsafe_project_local_relative_path_requires_test() {
    let xml = live_11_3_type3_reference("AudioClip", "Samples/../outside.wav");
    let fixture = gzip_als("live_11_3_unsafe_project_local", &xml);
    let model = analyze_als(fixture.path()).expect("fixture should parse");
    let reference = &model.active_audio_references[0];

    assert!(!reference.is_rewrite_candidate);
    assert_eq!(reference.rewrite_support_status, "requires_test");
}

#[test]
fn unknown_or_unsupported_context_is_not_a_rewrite_candidate() {
    let xml = minimal_ableton_with_file_ref("/external/kick.wav", "../kick.wav")
        .replace("MinorVersion=\"12\"", "MinorVersion=\"11.0_11300\"")
        .replace(
            "Creator=\"Ableton Live\"",
            "Creator=\"Ableton Live 11.3.43\"",
        );
    let fixture = gzip_als("live_11_3_unknown_context", &xml);
    let model = analyze_als(fixture.path()).expect("fixture should parse");
    let reference = &model.active_audio_references[0];

    assert_eq!(reference.usage_context, "unknown");
    assert!(!reference.is_rewrite_candidate);
    assert_eq!(reference.rewrite_support_status, "requires_test");
}

#[test]
fn als_reader_does_not_infer_project_root_from_als_parent() {
    let fixture = gzip_als(
        "project_root_not_inferred",
        &minimal_ableton_with_file_ref("/Samples/Kick.wav", "Samples/Kick.wav"),
    );
    let model = analyze_als(fixture.path()).unwrap();

    assert_eq!(model.set_metadata.reader_version, "0.2.4");
    assert_eq!(model.set_metadata.source_project_root, None);
}

#[test]
fn xml_context_does_not_duplicate_the_observed_element() {
    let fixture = gzip_als(
        "xml_context_without_duplicate",
        minimal_ableton_with_historical_and_non_audio_refs(),
    );
    let model = analyze_als(fixture.path()).unwrap();

    assert_eq!(
        model.historical_refs[0].xml_context,
        "Ableton/LiveSet/SourceContext/OriginalFileRef"
    );
    assert_eq!(
        model.non_audio_dependency_signals[0].xml_context,
        "Ableton/LiveSet/PluginDevice/FileRef"
    );
    assert!(!model.historical_refs[0]
        .xml_context
        .contains("OriginalFileRef/OriginalFileRef"));
    assert!(!model.non_audio_dependency_signals[0]
        .xml_context
        .contains("FileRef/FileRef"));
}

#[test]
fn filename_from_handles_windows_backslashes_on_macos() {
    let fixture = gzip_als(
        "windows_backslash_path",
        &minimal_ableton_with_file_ref(r"C:\Users\fixture\Samples\kick.wav", ""),
    );
    let model = analyze_als(fixture.path()).unwrap();

    let first = &model.active_audio_references[0];
    assert_eq!(first.filename.as_deref(), Some("kick.wav"));
    assert_eq!(first.extension.as_deref(), Some("wav"));
    assert_eq!(
        first.raw_path.as_deref(),
        Some(r"C:\Users\fixture\Samples\kick.wav")
    );
}

#[test]
fn filename_from_handles_windows_unc_path_on_macos() {
    let fixture = gzip_als(
        "windows_unc_path",
        &minimal_ableton_with_file_ref(r"\\NAS\Samples\snare.aif", ""),
    );
    let model = analyze_als(fixture.path()).unwrap();

    let first = &model.active_audio_references[0];
    assert_eq!(first.filename.as_deref(), Some("snare.aif"));
    assert_eq!(first.extension.as_deref(), Some("aif"));
    assert_eq!(first.raw_path.as_deref(), Some(r"\\NAS\Samples\snare.aif"));
}

#[test]
fn filename_from_uses_relative_path_when_raw_path_is_empty() {
    let fixture = gzip_als(
        "empty_raw_path_with_relative_path",
        &minimal_ableton_with_file_ref("", "Samples/Recorded/kick.aiff"),
    );
    let model = analyze_als(fixture.path()).unwrap();

    let first = &model.active_audio_references[0];
    assert_eq!(first.filename.as_deref(), Some("kick.aiff"));
    assert_eq!(first.extension.as_deref(), Some("aiff"));
    assert_eq!(first.raw_path.as_deref(), Some(""));
    assert_eq!(
        first.raw_relative_path.as_deref(),
        Some("Samples/Recorded/kick.aiff")
    );
}

#[test]
fn invalid_gzip_returns_structured_error() {
    let fixture = raw_als("not_gzip", b"this is not a gzip stream");
    let error = analyze_als(fixture.path()).unwrap_err();

    assert!(matches!(error, ALSError::NotGzip { .. }));
    assert_eq!(error.to_info().error_code, "ALS_NOT_GZIP");
}

#[test]
fn missing_file_returns_structured_error() {
    let error = analyze_als(fixture_path("als/does_not_exist.als")).unwrap_err();

    assert!(matches!(error, ALSError::FileNotFound { .. }));
    assert_eq!(error.to_info().error_code, "ALS_NOT_FOUND");
}

#[test]
fn compressed_als_size_limit_returns_structured_error() {
    let fixture = raw_als("too_large_compressed", &[]);
    let file = fs::OpenOptions::new()
        .write(true)
        .open(fixture.path())
        .unwrap();
    file.set_len(MAX_COMPRESSED_ALS_BYTES + 1).unwrap();

    let error = analyze_als(fixture.path()).unwrap_err();

    assert!(matches!(error, ALSError::CompressedTooLarge { .. }));
    assert_eq!(error.to_info().error_code, "ALS_COMPRESSED_TOO_LARGE");
}

#[test]
fn gzip_with_invalid_xml_returns_structured_error() {
    let fixture = gzip_als("invalid_xml", "<Ableton><LiveSet>");
    let error = analyze_als(fixture.path()).unwrap_err();

    assert!(matches!(error, ALSError::InvalidXml { .. }));
    assert_eq!(error.to_info().error_code, "ALS_XML_INVALID");
}

#[test]
fn gzip_without_ableton_root_returns_structured_error() {
    let fixture = gzip_als("missing_ableton_root", "<NotAbleton />");
    let error = analyze_als(fixture.path()).unwrap_err();

    assert!(matches!(error, ALSError::MissingAbletonRoot { .. }));
    assert_eq!(error.to_info().error_code, "ALS_UNSUPPORTED_ROOT");
}

#[test]
fn active_relative_path_type_zero_is_preserved() {
    let fixture = synthetic_als("relative_path_type_zero", &["0", "0", "0", "3", "3"], 0);
    let model = analyze_als(fixture.path()).unwrap();
    let counts = relative_path_type_counts(&model);

    assert_eq!(model.set_metadata.sample_ref_count, 5);
    assert_eq!(counts.get("0"), Some(&3));
    assert_eq!(counts.get("3"), Some(&2));
    assert!(model.warnings.iter().all(|warning| {
        warning.warning_code != "unknown_relative_path_type"
            && warning.warning_code != "missing_relative_path_type"
    }));
}

#[test]
fn read_only_safety() {
    let fixture = synthetic_als("read_only_safety", &["1"; 11], 0);
    let before = fs::read(fixture.path()).unwrap();

    let analysis = analyze_als(fixture.path()).unwrap();

    let after = fs::read(fixture.path()).unwrap();
    assert_eq!(before, after);
    assert_eq!(analysis.set_metadata.sample_ref_count, 11);
    assert_eq!(analysis.set_metadata.active_audio_ref_count, 11);
}
