#[cfg(any(unix, windows))]
use flate2::read::GzDecoder;
use flate2::write::GzEncoder;
use flate2::Compression;
use rescue_pipeline::{prepare_current_path_copy, run_current_path_copy, CurrentPathCopyRequest};
use std::fs;
#[cfg(any(unix, windows))]
use std::io::Read;
use std::io::Write;
use std::path::{Path, PathBuf};
use tempfile::TempDir;

struct Fixture {
    _temp: TempDir,
    source_als: PathBuf,
    available_audio: PathBuf,
    #[cfg(any(unix, windows))]
    missing_audio: PathBuf,
    staging_root: PathBuf,
    target_root: PathBuf,
    ledger_path: PathBuf,
}

fn fixture(include_missing_reference: bool) -> Fixture {
    let temp = tempfile::tempdir().expect("tempdir");
    let source_root = temp.path().join("source Project");
    let output_root = temp.path().join("output");
    let evidence_root = temp.path().join("evidence");
    fs::create_dir(&source_root).expect("source root");
    fs::create_dir(source_root.join("Ableton Project Info")).expect("project marker");
    fs::create_dir(&output_root).expect("output root");
    fs::create_dir(&evidence_root).expect("evidence root");

    let available_audio = source_root.join("available.wav");
    let missing_audio = source_root.join("gone").join("missing.wav");
    let source_als = source_root.join("Set.als");
    fs::write(&available_audio, b"available-audio").expect("available audio");
    fs::write(
        &source_als,
        gzip(&project_xml(
            &available_audio,
            include_missing_reference.then_some(missing_audio.as_path()),
        )),
    )
    .expect("source ALS");

    Fixture {
        _temp: temp,
        source_als,
        available_audio,
        #[cfg(any(unix, windows))]
        missing_audio,
        staging_root: output_root.join("Project.staging"),
        target_root: output_root.join("Project"),
        ledger_path: evidence_root.join("private-ledger.json"),
    }
}

#[cfg(any(unix, windows))]
fn project_local_fixture() -> Fixture {
    let temp = tempfile::tempdir().expect("tempdir");
    let source_root = temp.path().join("source Project");
    let output_root = temp.path().join("output");
    let evidence_root = temp.path().join("evidence");
    let recorded_root = source_root.join("Samples/Recorded");
    fs::create_dir_all(&recorded_root).expect("recorded source root");
    fs::create_dir(source_root.join("Ableton Project Info")).expect("project marker");
    fs::create_dir(&output_root).expect("output root");
    fs::create_dir(&evidence_root).expect("evidence root");

    let available_audio = recorded_root.join("shared.wav");
    let relative_path = "Samples/Recorded/shared.wav";
    let source_als = source_root.join("Set.als");
    fs::write(&available_audio, b"project-local-audio").expect("project-local audio");
    let reference = |parent: &str| {
        format!(
            r#"<{parent}>
      <SampleRef>
        <FileRef>
          <Path Value="{}"/>
          <RelativePath Value="{relative_path}"/>
          <RelativePathType Value="3"/>
          <Type Value="1"/>
          <OriginalFileSize Value="19"/>
          <OriginalCrc Value="303"/>
        </FileRef>
        <DefaultDuration Value="10"/>
        <DefaultSampleRate Value="44100"/>
      </SampleRef>
    </{parent}>"#,
            available_audio.to_string_lossy()
        )
    };
    let xml = format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<Ableton MajorVersion="5" MinorVersion="11.0_11300" Creator="Ableton Live 11.3.43" SchemaChangeCount="7">
  <LiveSet>{}{}</LiveSet>
</Ableton>"#,
        reference("AudioClip"),
        reference("MultiSamplePart")
    );
    fs::write(&source_als, gzip(&xml)).expect("project-local ALS");

    Fixture {
        _temp: temp,
        source_als,
        available_audio,
        #[cfg(any(unix, windows))]
        missing_audio: source_root.join("unused-missing.wav"),
        staging_root: output_root.join("Project.staging"),
        target_root: output_root.join("Project"),
        ledger_path: evidence_root.join("private-ledger.json"),
    }
}

fn request(fixture: &Fixture) -> CurrentPathCopyRequest {
    CurrentPathCopyRequest {
        run_id: "current-path-run".to_string(),
        rewrite_policy: rescue_pipeline::STRICT_REWRITE_POLICY.to_string(),
        source_als_path: fixture.source_als.clone(),
        expected_source_als_sha256: None,
        expected_plan_fingerprint: None,
        staging_root: fixture.staging_root.clone(),
        target_project_root: fixture.target_root.clone(),
        private_ledger_path: fixture.ledger_path.clone(),
    }
}

fn gzip(xml: &str) -> Vec<u8> {
    let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
    encoder.write_all(xml.as_bytes()).expect("gzip write");
    encoder.finish().expect("gzip finish")
}

#[cfg(any(unix, windows))]
fn unzip(path: &Path) -> String {
    let mut decoder = GzDecoder::new(fs::File::open(path).expect("open ALS"));
    let mut xml = String::new();
    decoder.read_to_string(&mut xml).expect("decode ALS");
    xml
}

#[cfg(any(unix, windows))]
fn path_value_count(xml: &str, expected: &Path) -> usize {
    let expected = fs::canonicalize(expected).expect("canonical expected path");
    let document = roxmltree::Document::parse(xml).expect("parsed rewritten XML");
    document
        .descendants()
        .filter(|node| node.has_tag_name("Path"))
        .filter_map(|node| node.attribute("Value"))
        .filter_map(|value| fs::canonicalize(Path::new(value)).ok())
        .filter(|value| value == &expected)
        .count()
}

fn project_xml(available: &Path, missing: Option<&Path>) -> String {
    let mut references = sample_reference(available, "available.wav", 15, 101);
    if let Some(missing) = missing {
        references.push_str(&sample_reference(missing, "missing.wav", 13, 202));
    }
    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<Ableton MajorVersion="5" MinorVersion="11.0_11300" Creator="Ableton Live 11.3.43" SchemaChangeCount="7">
  <LiveSet>{references}</LiveSet>
</Ableton>"#
    )
}

fn sample_reference(path: &Path, filename: &str, size: u64, crc: u64) -> String {
    format!(
        r#"
    <AudioClip>
      <SampleRef>
        <FileRef>
          <Path Value="{}"/>
          <RelativePath Value="../{filename}"/>
          <RelativePathType Value="1"/>
          <Type Value="1"/>
          <OriginalFileSize Value="{size}"/>
          <OriginalCrc Value="{crc}"/>
        </FileRef>
        <DefaultDuration Value="10"/>
        <DefaultSampleRate Value="44100"/>
      </SampleRef>
    </AudioClip>"#,
        path.to_string_lossy()
    )
}

#[cfg(any(unix, windows))]
#[test]
fn complete_current_path_copy_is_promoted() {
    let fixture = fixture(false);
    let result = run_current_path_copy(&request(&fixture));

    assert_eq!(
        result.run_status, "complete_copy_ready_for_manual_check",
        "pipeline errors: {:#?}",
        result.errors
    );
    assert_eq!(result.required_asset_count, 1);
    assert_eq!(result.copied_asset_count, 1);
    assert_eq!(result.rewritten_reference_count, 1);
    assert_eq!(result.omitted_asset_count, 0);
    assert!(fixture.target_root.join("Set.als").is_file());
    assert!(fixture
        .target_root
        .join("Samples/Imported/available.wav")
        .is_file());
}

#[cfg(any(unix, windows))]
#[test]
fn current_path_audio_uses_metadata_only_verification_end_to_end() {
    let fixture = fixture(false);
    let result = run_current_path_copy(&request(&fixture));
    let plan = result.package_plan.as_ref().expect("package plan");
    let als_operation = plan
        .copy_operations
        .iter()
        .find(|operation| operation.operation_kind == "copy_als")
        .expect("ALS operation");
    let audio_operation = plan
        .copy_operations
        .iter()
        .find(|operation| operation.operation_kind == "copy_audio")
        .expect("audio operation");

    assert!(als_operation.expected_source_sha256.is_some());
    assert_eq!(
        als_operation.verification_policy,
        rescue_packaging::VERIFY_SHA256_AND_SIZE
    );
    assert_eq!(audio_operation.expected_source_sha256, None);
    assert_eq!(audio_operation.content_id, None);
    assert_eq!(
        audio_operation.verification_policy,
        rescue_packaging::VERIFY_STABLE_SOURCE_AND_SIZE
    );

    let ledger: serde_json::Value =
        serde_json::from_slice(&fs::read(&fixture.ledger_path).expect("private ledger"))
            .expect("ledger JSON");
    let staging_audio = ledger["staging"]["copy_records"]
        .as_array()
        .expect("staging records")
        .iter()
        .find(|record| record["operation_kind"] == "copy_audio")
        .expect("staging audio");
    assert!(staging_audio["expected_sha256"].is_null());
    assert!(staging_audio["observed_sha256"].is_null());
    assert_eq!(
        staging_audio["verification_method"],
        rescue_packaging::VERIFY_STABLE_SOURCE_AND_SIZE
    );

    let validation_audio = ledger["validation"]["file_records"]
        .as_array()
        .expect("validation records")
        .iter()
        .find(|record| record["operation_id"] == audio_operation.operation_id)
        .expect("validation audio");
    assert!(validation_audio["expected_sha256"].is_null());
    assert!(validation_audio["observed_sha256"].is_null());
    assert_eq!(
        validation_audio["verification_method"],
        rescue_packaging::VERIFY_STABLE_SOURCE_AND_SIZE
    );

    let manifest: serde_json::Value = serde_json::from_slice(
        &fs::read(
            fixture
                .target_root
                .join("Rescue Manifest/package-manifest.json"),
        )
        .expect("package manifest"),
    )
    .expect("manifest JSON");
    let manifest_audio = manifest["files"]
        .as_array()
        .expect("manifest files")
        .iter()
        .find(|record| record["role"] == "copy_audio")
        .expect("manifest audio");
    assert!(manifest_audio["sha256"].is_null());
    assert_eq!(manifest_audio["content_identity_status"], "not_computed");
    assert_eq!(
        manifest_audio["verification_method"],
        rescue_packaging::VERIFY_STABLE_SOURCE_AND_SIZE
    );

    let promoted_audio = result
        .promotion
        .as_ref()
        .expect("promotion")
        .promoted_files
        .iter()
        .find(|record| record.relative_path == audio_operation.target_relative_path)
        .expect("promoted audio");
    assert_eq!(promoted_audio.expected_sha256, None);
    assert_eq!(promoted_audio.observed_sha256, None);
    assert_eq!(
        promoted_audio.verification_method,
        rescue_packaging::VERIFY_STABLE_SOURCE_AND_SIZE
    );
}

#[cfg(any(unix, windows))]
#[test]
fn project_local_type3_copy_preserves_structure_and_rewrites_path_only() {
    let fixture = project_local_fixture();
    let original_path = fixture.available_audio.to_string_lossy().to_string();
    let result = run_current_path_copy(&request(&fixture));
    let target_audio = fixture.target_root.join("Samples/Recorded/shared.wav");

    assert_eq!(
        result.run_status, "complete_copy_ready_for_manual_check",
        "pipeline errors: {:#?}",
        result.errors
    );
    let rewritten_xml = unzip(&fixture.target_root.join("Set.als"));
    assert_eq!(result.required_asset_count, 1);
    assert_eq!(result.copied_asset_count, 1);
    assert_eq!(result.rewritten_reference_count, 2);
    assert_eq!(result.omitted_asset_count, 0);
    assert!(target_audio.is_file());
    assert!(!fixture
        .target_root
        .join("Samples/Imported/shared.wav")
        .exists());
    assert_eq!(rewritten_xml.matches(&original_path).count(), 0);
    assert_eq!(path_value_count(&rewritten_xml, &target_audio), 2);
    assert_eq!(
        rewritten_xml
            .matches("RelativePath Value=\"Samples/Recorded/shared.wav\"")
            .count(),
        2
    );
    assert_eq!(
        rewritten_xml
            .matches("RelativePathType Value=\"3\"")
            .count(),
        2
    );
}

#[cfg(any(unix, windows))]
#[test]
fn missing_asset_creates_incomplete_copy() {
    let fixture = fixture(true);
    let result = run_current_path_copy(&request(&fixture));

    assert_eq!(result.run_status, "incomplete_copy_ready_for_manual_check");
    assert_eq!(result.required_asset_count, 2);
    assert_eq!(result.copied_asset_count, 1);
    assert_eq!(result.rewritten_reference_count, 1);
    assert_eq!(result.omitted_asset_count, 1);
    assert!(fixture.target_root.join("Set.als").is_file());

    let manifest: serde_json::Value = serde_json::from_slice(
        &fs::read(
            fixture
                .target_root
                .join("Rescue Manifest/package-manifest.json"),
        )
        .expect("package manifest"),
    )
    .expect("manifest JSON");
    assert_eq!(
        manifest["package_status"],
        "incomplete_copy_ready_for_manual_check"
    );
    assert_eq!(manifest["omissions"].as_array().map(Vec::len), Some(1));
}

#[test]
fn all_missing_assets_create_incomplete_als_copy() {
    let fixture = fixture(true);
    fs::remove_file(&fixture.available_audio).expect("remove copied fixture audio");
    let result = run_current_path_copy(&request(&fixture));

    assert_eq!(result.run_status, "incomplete_copy_ready_for_manual_check");
    assert_eq!(result.required_asset_count, 2);
    assert_eq!(result.copied_asset_count, 0);
    assert_eq!(result.rewritten_reference_count, 0);
    assert_eq!(result.omitted_asset_count, 2);
    assert!(fixture.target_root.join("Set.als").is_file());
    assert!(fixture.target_root.join("Samples/Imported").is_dir());
}

#[cfg(any(unix, windows))]
#[test]
fn missing_reference_remains_unchanged() {
    let fixture = fixture(true);
    let original_missing_path = fixture.missing_audio.to_string_lossy().to_string();
    let result = run_current_path_copy(&request(&fixture));
    let rewritten_xml = unzip(&fixture.target_root.join("Set.als"));

    assert_eq!(result.run_status, "incomplete_copy_ready_for_manual_check");
    assert!(rewritten_xml.contains(&format!("Path Value=\"{original_missing_path}\"")));
    assert!(rewritten_xml.contains("RelativePath Value=\"../missing.wav\""));
}

#[cfg(any(unix, windows))]
#[test]
fn available_assets_are_relinked_when_another_asset_is_missing() {
    let fixture = fixture(true);
    let original_available_path = fixture.available_audio.to_string_lossy().to_string();
    let result = run_current_path_copy(&request(&fixture));
    let rewritten_xml = unzip(&fixture.target_root.join("Set.als"));

    assert_eq!(result.run_status, "incomplete_copy_ready_for_manual_check");
    assert!(!rewritten_xml.contains(&format!("Path Value=\"{original_available_path}\"")));
    assert_eq!(
        path_value_count(
            &rewritten_xml,
            &fixture.target_root.join("Samples/Imported/available.wav")
        ),
        1
    );
    assert!(rewritten_xml.contains("RelativePath Value=\"Samples/Imported/available.wav\""));
    assert!(rewritten_xml.contains("RelativePathType Value=\"3\""));
}

#[test]
fn unsafe_output_still_blocks_before_write() {
    let fixture = fixture(false);
    let mut unsafe_request = request(&fixture);
    unsafe_request.target_project_root = fixture.source_als.parent().expect("parent").to_path_buf();
    let result = run_current_path_copy(&unsafe_request);

    assert_eq!(result.run_status, "rejected_before_read");
    assert!(result
        .errors
        .iter()
        .any(|error| error.error_code == "PIPELINE_OUTPUT_ALREADY_EXISTS"));
    assert!(!fixture.staging_root.exists());
    assert!(!fixture.ledger_path.exists());
}

#[test]
fn blocked_plan_exposes_specific_planner_error() {
    let fixture = fixture(false);
    let unsupported_xml = project_xml(&fixture.available_audio, None)
        .replace("11.0_11300", "12.0_12000")
        .replace("Ableton Live 11.3.43", "Ableton Live 12.0.1");
    fs::write(&fixture.source_als, gzip(&unsupported_xml)).expect("unsupported ALS");
    let result = prepare_current_path_copy(&request(&fixture));

    assert_eq!(result.run_status, "snapshot_or_plan_blocked");
    assert!(result
        .errors
        .iter()
        .any(|error| error.error_code == "PACKAGE_REWRITE_DOCUMENT_UNSUPPORTED"));
}

#[test]
fn strict_profile_still_blocks_unconfirmed_document() {
    let fixture = fixture(false);
    let unconfirmed_xml = project_xml(&fixture.available_audio, None)
        .replace("11.0_11300", "10.0_10000")
        .replace("Ableton Live 11.3.43", "Ableton Live 10.1.43");
    fs::write(&fixture.source_als, gzip(&unconfirmed_xml)).expect("unconfirmed ALS");

    let result = prepare_current_path_copy(&request(&fixture));

    assert_eq!(result.run_status, "snapshot_or_plan_blocked");
    assert!(result
        .errors
        .iter()
        .any(|error| error.error_code == "PACKAGE_REWRITE_DOCUMENT_UNSUPPORTED"));
    assert!(!fixture.staging_root.exists());
    assert!(!fixture.target_root.exists());
}

#[test]
fn compatibility_lab_blocks_unknown_reference_shape() {
    let fixture = fixture(false);
    let unknown_xml = project_xml(&fixture.available_audio, None)
        .replace("11.0_11300", "10.0_10000")
        .replace("Ableton Live 11.3.43", "Ableton Live 10.1.43")
        .replace("<AudioClip>", "<UnknownClip>")
        .replace("</AudioClip>", "</UnknownClip>");
    fs::write(&fixture.source_als, gzip(&unknown_xml)).expect("unknown-shape ALS");
    let mut lab_request = request(&fixture);
    lab_request.rewrite_policy = rescue_pipeline::COMPATIBILITY_LAB_REWRITE_POLICY.to_string();

    let result = prepare_current_path_copy(&lab_request);

    assert_eq!(result.run_status, "snapshot_or_plan_blocked");
    assert!(result.errors.iter().any(|error| {
        error.error_code == "CURRENT_PATH_UNRESOLVED_BLOCKER"
            && error.message.contains("rewrite_reference_not_supported")
    }));
    assert!(!fixture.staging_root.exists());
    assert!(!fixture.target_root.exists());
}

#[cfg(any(unix, windows))]
#[test]
fn compatibility_lab_keeps_sources_read_only() {
    let fixture = fixture(false);
    let unconfirmed_xml = project_xml(&fixture.available_audio, None)
        .replace("11.0_11300", "10.0_10000")
        .replace("Ableton Live 11.3.43", "Ableton Live 10.1.43");
    fs::write(&fixture.source_als, gzip(&unconfirmed_xml)).expect("unconfirmed ALS");
    let als_before = fs::read(&fixture.source_als).expect("ALS before");
    let audio_before = fs::read(&fixture.available_audio).expect("audio before");
    let mut lab_request = request(&fixture);
    lab_request.rewrite_policy = rescue_pipeline::COMPATIBILITY_LAB_REWRITE_POLICY.to_string();

    let result = run_current_path_copy(&lab_request);

    assert_eq!(
        result.run_status, "complete_copy_ready_for_manual_check",
        "copy errors: {:#?}",
        result.errors
    );
    assert_eq!(result.rewritten_reference_count, 1);
    assert_eq!(
        fs::read(&fixture.source_als).expect("ALS after"),
        als_before
    );
    assert_eq!(
        fs::read(&fixture.available_audio).expect("audio after"),
        audio_before
    );
}

#[test]
fn current_path_copy_keeps_sources_read_only() {
    let fixture = fixture(true);
    let als_before = fs::read(&fixture.source_als).expect("ALS before");
    let audio_before = fs::read(&fixture.available_audio).expect("audio before");
    let preview = prepare_current_path_copy(&request(&fixture));

    assert_eq!(preview.run_status, "incomplete_copy_preview_ready");
    assert!(!fixture.staging_root.exists());
    assert!(!fixture.target_root.exists());
    assert!(!fixture.ledger_path.exists());
    assert_eq!(
        fs::read(&fixture.source_als).expect("ALS after"),
        als_before
    );
    assert_eq!(
        fs::read(&fixture.available_audio).expect("audio after"),
        audio_before
    );
}

#[test]
fn changed_plan_fingerprint_blocks_before_write() {
    let fixture = fixture(false);
    let mut execute_request = request(&fixture);
    let preview = prepare_current_path_copy(&execute_request);
    execute_request.expected_source_als_sha256 = preview
        .package_plan
        .as_ref()
        .map(|plan| plan.source_als.source_file_hash.clone());
    execute_request.expected_plan_fingerprint = preview.plan_fingerprint;

    fs::write(&fixture.available_audio, vec![b'x'; 16]).expect("resize source audio");
    let result = run_current_path_copy(&execute_request);

    assert_eq!(result.run_status, "preview_plan_changed");
    assert!(result
        .errors
        .iter()
        .any(|error| error.error_code == "CURRENT_PATH_PREVIEW_PLAN_CHANGED"));
    assert!(!fixture.target_root.exists());
    assert!(!fixture.staging_root.exists());
    assert!(!fixture.ledger_path.exists());
}

#[cfg(any(unix, windows))]
#[test]
fn matching_plan_fingerprint_allows_execution() {
    let fixture = fixture(false);
    let mut execute_request = request(&fixture);
    let preview = prepare_current_path_copy(&execute_request);
    execute_request.expected_source_als_sha256 = preview
        .package_plan
        .as_ref()
        .map(|plan| plan.source_als.source_file_hash.clone());
    execute_request.expected_plan_fingerprint = preview.plan_fingerprint;

    let result = run_current_path_copy(&execute_request);

    assert_eq!(result.run_status, "complete_copy_ready_for_manual_check");
    assert!(fixture.target_root.join("Set.als").is_file());
}

#[cfg(not(any(unix, windows)))]
#[test]
fn unsupported_atomic_replace_platform_fails_closed_at_pipeline() {
    let fixture = fixture(false);
    let als_before = fs::read(&fixture.source_als).expect("ALS before");
    let audio_before = fs::read(&fixture.available_audio).expect("audio before");

    let result = run_current_path_copy(&request(&fixture));

    assert_eq!(result.run_status, "write_pipeline_failed");
    assert!(result
        .errors
        .iter()
        .any(|error| error.error_code == "PIPELINE_REWRITE_FAILED"));
    assert!(!fixture.target_root.exists());
    assert_eq!(
        fs::read(&fixture.source_als).expect("ALS after"),
        als_before
    );
    assert_eq!(
        fs::read(&fixture.available_audio).expect("audio after"),
        audio_before
    );
}

#[cfg(windows)]
#[test]
fn windows_complete_current_path_copy_is_promoted_on_ntfs() {
    let fixture = fixture(false);
    let source_als_before = fs::read(&fixture.source_als).expect("source ALS before");
    let source_audio_before = fs::read(&fixture.available_audio).expect("source audio before");

    let result = run_current_path_copy(&request(&fixture));

    assert_eq!(result.run_status, "complete_copy_ready_for_manual_check");
    assert!(fixture.target_root.join("Set.als").is_file());
    assert!(fixture
        .target_root
        .join("Samples/Imported/available.wav")
        .is_file());
    assert_eq!(
        fs::read(&fixture.source_als).expect("source ALS after"),
        source_als_before
    );
    assert_eq!(
        fs::read(&fixture.available_audio).expect("source audio after"),
        source_audio_before
    );
    assert!(!fixture.staging_root.exists());
}
