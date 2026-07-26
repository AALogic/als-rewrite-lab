use flate2::write::GzEncoder;
use flate2::Compression;
use rescue_core::{analyze_als, ALSError, ALSReadModel, MAX_COMPRESSED_ALS_BYTES};
use std::collections::BTreeMap;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

fn fixture_path(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/fixtures")
        .join(name)
}

fn corpus_copy_path(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../experiments/2026-06-02_als_structure_corpus_20/copies")
        .join(name)
}

fn write_temp_gzip(name: &str, body: &str) -> PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let path = std::env::temp_dir().join(format!("{name}_{unique}.als"));
    let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
    encoder.write_all(body.as_bytes()).unwrap();
    let bytes = encoder.finish().unwrap();
    fs::write(&path, bytes).unwrap();
    path
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
    let model = analyze_als(fixture_path("als/cziki_before_cas.als")).unwrap();

    assert_eq!(model.set_metadata.als_read_model_version, "0.2");
    assert_eq!(model.set_metadata.sample_ref_count, 153);
    assert_eq!(model.set_metadata.active_audio_ref_count, 153);
    assert_eq!(model.active_audio_references.len(), 153);
    assert_eq!(model.set_metadata.historical_ref_count, 71);
    assert_eq!(model.historical_refs.len(), 71);

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
    let template = analyze_als(fixture_path("als/template_zero_active.als")).unwrap();

    assert_eq!(template.set_metadata.sample_ref_count, 0);
    assert_eq!(template.set_metadata.active_audio_ref_count, 0);
    assert_eq!(template.active_audio_references.len(), 0);
    assert_eq!(template.set_metadata.historical_ref_count, 6);
    assert_eq!(template.historical_refs.len(), 6);
    assert!(template
        .historical_refs
        .iter()
        .all(|historical_ref| historical_ref.usage_note == "historical_provenance"));

    let cziki = analyze_als(fixture_path("als/cziki_before_cas.als")).unwrap();

    assert_eq!(cziki.set_metadata.sample_ref_count, 153);
    assert_eq!(cziki.set_metadata.active_audio_ref_count, 153);
    assert_eq!(cziki.active_audio_references.len(), 153);
    assert_eq!(cziki.set_metadata.historical_ref_count, 71);
    assert_eq!(cziki.historical_refs.len(), 71);
}

#[test]
fn relative_path_type_is_captured_raw() {
    let before = analyze_als(fixture_path("als/cziki_before_cas.als")).unwrap();
    let after = analyze_als(fixture_path("als/cziki_after_cas.als")).unwrap();

    assert_eq!(
        relative_path_type_counts(&before),
        BTreeMap::from([("1".to_string(), 33), ("5".to_string(), 120)])
    );
    assert_eq!(
        relative_path_type_counts(&after),
        BTreeMap::from([("3".to_string(), 33), ("5".to_string(), 120)])
    );
}

#[test]
fn als_reader_v0_2_does_not_check_filesystem_paths() {
    let model = analyze_als(fixture_path("als/kombinacja_piejo.als")).unwrap();
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
    let model = analyze_als(fixture_path("als/kombinacja_piejo.als")).unwrap();
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
fn als_reader_does_not_infer_project_root_from_als_parent() {
    let path = write_temp_gzip(
        "project_root_not_inferred",
        &minimal_ableton_with_file_ref("/Samples/Kick.wav", "Samples/Kick.wav"),
    );
    let model = analyze_als(&path).unwrap();

    let _ = fs::remove_file(path);
    assert_eq!(model.set_metadata.reader_version, "0.2.2");
    assert_eq!(model.set_metadata.source_project_root, None);
}

#[test]
fn xml_context_does_not_duplicate_the_observed_element() {
    let path = write_temp_gzip(
        "xml_context_without_duplicate",
        minimal_ableton_with_historical_and_non_audio_refs(),
    );
    let model = analyze_als(&path).unwrap();

    let _ = fs::remove_file(path);
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
    let path = write_temp_gzip(
        "windows_backslash_path",
        &minimal_ableton_with_file_ref(r"C:\Users\Bartosz\Samples\kick.wav", ""),
    );
    let model = analyze_als(&path).unwrap();

    let _ = fs::remove_file(path);
    let first = &model.active_audio_references[0];
    assert_eq!(first.filename.as_deref(), Some("kick.wav"));
    assert_eq!(first.extension.as_deref(), Some("wav"));
    assert_eq!(
        first.raw_path.as_deref(),
        Some(r"C:\Users\Bartosz\Samples\kick.wav")
    );
}

#[test]
fn filename_from_handles_windows_unc_path_on_macos() {
    let path = write_temp_gzip(
        "windows_unc_path",
        &minimal_ableton_with_file_ref(r"\\NAS\Samples\snare.aif", ""),
    );
    let model = analyze_als(&path).unwrap();

    let _ = fs::remove_file(path);
    let first = &model.active_audio_references[0];
    assert_eq!(first.filename.as_deref(), Some("snare.aif"));
    assert_eq!(first.extension.as_deref(), Some("aif"));
    assert_eq!(first.raw_path.as_deref(), Some(r"\\NAS\Samples\snare.aif"));
}

#[test]
fn filename_from_uses_relative_path_when_raw_path_is_empty() {
    let path = write_temp_gzip(
        "empty_raw_path_with_relative_path",
        &minimal_ableton_with_file_ref("", "Samples/Recorded/kick.aiff"),
    );
    let model = analyze_als(&path).unwrap();

    let _ = fs::remove_file(path);
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
    let error = analyze_als(fixture_path("invalid/not_gzip.als")).unwrap_err();

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
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let path = std::env::temp_dir().join(format!("too_large_compressed_{unique}.als"));
    let file = fs::File::create(&path).unwrap();
    file.set_len(MAX_COMPRESSED_ALS_BYTES + 1).unwrap();

    let error = analyze_als(&path).unwrap_err();

    let _ = fs::remove_file(path);
    assert!(matches!(error, ALSError::CompressedTooLarge { .. }));
    assert_eq!(error.to_info().error_code, "ALS_COMPRESSED_TOO_LARGE");
}

#[test]
fn gzip_with_invalid_xml_returns_structured_error() {
    let path = write_temp_gzip("invalid_xml", "<Ableton><LiveSet>");
    let error = analyze_als(&path).unwrap_err();

    let _ = fs::remove_file(path);
    assert!(matches!(error, ALSError::InvalidXml { .. }));
    assert_eq!(error.to_info().error_code, "ALS_XML_INVALID");
}

#[test]
fn gzip_without_ableton_root_returns_structured_error() {
    let path = write_temp_gzip("missing_ableton_root", "<NotAbleton />");
    let error = analyze_als(&path).unwrap_err();

    let _ = fs::remove_file(path);
    assert!(matches!(error, ALSError::MissingAbletonRoot { .. }));
    assert_eq!(error.to_info().error_code, "ALS_UNSUPPORTED_ROOT");
}

#[test]
fn active_relative_path_type_zero_is_preserved() {
    let model = analyze_als(corpus_copy_path("14__POLISHBOYS_WWA_11.10_GOSCINKA.als")).unwrap();
    let counts = relative_path_type_counts(&model);

    assert_eq!(model.set_metadata.sample_ref_count, 1800);
    assert_eq!(counts.get("0"), Some(&1100));
    assert_eq!(counts.get("3"), Some(&700));
    assert!(model.warnings.iter().all(|warning| {
        warning.warning_code != "unknown_relative_path_type"
            && warning.warning_code != "missing_relative_path_type"
    }));
}

#[test]
fn read_only_safety() {
    let path = fixture_path("als/kombinacja_piejo.als");
    let before = fs::read(&path).unwrap();

    let analysis = analyze_als(&path).unwrap();

    let after = fs::read(&path).unwrap();
    assert_eq!(before, after);
    assert_eq!(analysis.set_metadata.sample_ref_count, 11);
    assert_eq!(analysis.set_metadata.active_audio_ref_count, 11);
}
