use flate2::write::GzEncoder;
use flate2::Compression;
use rescue_application::{
    execute_copy, prepare_copy, DesktopExecuteCopyRequest, DesktopPrepareCopyRequest,
};
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use tempfile::TempDir;

struct Fixture {
    _temp: TempDir,
    source_als: PathBuf,
    source_audio: PathBuf,
    missing_audio: PathBuf,
    target_root: PathBuf,
}

fn fixture(include_missing: bool) -> Fixture {
    let temp = tempfile::tempdir().expect("tempdir");
    let source_root = temp.path().join("source Project");
    let destination = temp.path().join("destination");
    fs::create_dir(&source_root).expect("source root");
    fs::create_dir(source_root.join("Ableton Project Info")).expect("project marker");
    fs::create_dir(&destination).expect("destination");
    let source_audio = source_root.join("available.wav");
    let missing_audio = source_root.join("missing.wav");
    let source_als = source_root.join("Set.als");
    fs::write(&source_audio, b"available-audio").expect("audio");
    write_als(
        &source_als,
        &source_audio,
        include_missing.then_some(missing_audio.as_path()),
        "7",
    );
    Fixture {
        _temp: temp,
        source_als,
        source_audio,
        missing_audio,
        target_root: destination.join("Set Rescue Project"),
    }
}

fn prepare_request(fixture: &Fixture) -> DesktopPrepareCopyRequest {
    DesktopPrepareCopyRequest {
        request_id: "desktop-preview".to_string(),
        source_als_path: fixture.source_als.clone(),
        target_project_root: fixture.target_root.clone(),
    }
}

fn execute_request(
    preview: rescue_application::DesktopCopyPreview,
    consent: bool,
) -> DesktopExecuteCopyRequest {
    DesktopExecuteCopyRequest {
        request_id: "desktop-execution".to_string(),
        preview,
        write_consent: consent,
    }
}

fn write_als(path: &Path, available: &Path, missing: Option<&Path>, schema_change_count: &str) {
    let mut references = sample_reference(available, "available.wav", 15, 101);
    if let Some(missing) = missing {
        references.push_str(&sample_reference(missing, "missing.wav", 13, 202));
    }
    let xml = format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<Ableton MajorVersion="5" MinorVersion="11.0_11300" Creator="Ableton Live 11.3.43" SchemaChangeCount="{schema_change_count}">
  <LiveSet>{references}</LiveSet>
</Ableton>"#
    );
    let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
    encoder.write_all(xml.as_bytes()).expect("gzip write");
    fs::write(path, encoder.finish().expect("gzip finish")).expect("ALS");
}

fn sample_reference(path: &Path, filename: &str, size: u64, crc: u64) -> String {
    format!(
        r#"
    <AudioClip><SampleRef><FileRef>
      <Path Value="{}"/><RelativePath Value="../{filename}"/>
      <RelativePathType Value="1"/><Type Value="1"/>
      <OriginalFileSize Value="{size}"/><OriginalCrc Value="{crc}"/>
    </FileRef><DefaultDuration Value="10"/><DefaultSampleRate Value="44100"/></SampleRef></AudioClip>"#,
        path.to_string_lossy()
    )
}

#[test]
fn copy_preview_is_read_only() {
    let fixture = fixture(true);
    let als_before = fs::read(&fixture.source_als).expect("ALS before");
    let audio_before = fs::read(&fixture.source_audio).expect("audio before");
    let preview = prepare_copy(&prepare_request(&fixture));

    assert_eq!(preview.preview_status, "incomplete_copy_preview_ready");
    assert_eq!(preview.omitted_asset_count, 1);
    assert!(!fixture.target_root.exists());
    assert_eq!(
        fs::read(&fixture.source_als).expect("ALS after"),
        als_before
    );
    assert_eq!(
        fs::read(&fixture.source_audio).expect("audio after"),
        audio_before
    );
}

#[test]
fn missing_write_consent_is_rejected() {
    let fixture = fixture(false);
    let preview = prepare_copy(&prepare_request(&fixture));
    let result = execute_copy(&execute_request(preview, false));

    assert_eq!(result.run_status, "copy_execution_failed");
    assert_eq!(result.errors[0].error_code, "DESKTOP_COPY_CONSENT_REQUIRED");
    assert!(!fixture.target_root.exists());
}

#[test]
fn copy_diagnostic_contains_stage_build_and_all_errors() {
    let fixture = fixture(false);
    let preview = prepare_copy(&prepare_request(&fixture));
    let result = execute_copy(&execute_request(preview, false));

    assert_eq!(result.diagnostic_report.operation_kind, "copy_execution");
    assert_eq!(result.diagnostic_report.pipeline_version, "0.5.0");
    assert!(!result.diagnostic_report.build_commit.is_empty());
    assert_eq!(result.diagnostic_report.completed_stage, "write_consent");
    assert_eq!(result.diagnostic_report.errors.len(), result.errors.len());
    assert_eq!(
        result.diagnostic_report.errors[0].error_code,
        "DESKTOP_COPY_CONSENT_REQUIRED"
    );
}

#[test]
fn copy_diagnostic_omits_private_paths_and_names() {
    let fixture = fixture(false);
    let preview = prepare_copy(&prepare_request(&fixture));
    let json = serde_json::to_string(&preview.diagnostic_report).expect("diagnostic JSON");

    assert!(!json.contains(&fixture.source_als.to_string_lossy().to_string()));
    assert!(!json.contains(&fixture.target_root.to_string_lossy().to_string()));
    assert!(!json.contains("Set.als"));
    assert!(!json.contains("available.wav"));
    assert!(!json.contains("ledger.json"));
}

#[test]
fn stale_preview_is_rejected() {
    let fixture = fixture(false);
    let preview = prepare_copy(&prepare_request(&fixture));
    write_als(&fixture.source_als, &fixture.source_audio, None, "8");
    let result = execute_copy(&execute_request(preview, true));

    assert_eq!(result.run_status, "snapshot_or_plan_blocked");
    assert!(result
        .errors
        .iter()
        .any(|error| error.error_code == "CURRENT_PATH_SOURCE_SNAPSHOT_CHANGED"));
    assert!(!fixture.target_root.exists());
}

#[cfg(any(unix, windows))]
#[test]
fn desktop_executes_complete_copy() {
    let fixture = fixture(false);
    let preview = prepare_copy(&prepare_request(&fixture));
    let result = execute_copy(&execute_request(preview, true));

    assert_eq!(result.run_status, "complete_copy_ready_for_manual_check");
    assert_eq!(
        result.final_target_root.as_ref(),
        Some(&fixture.target_root)
    );
    assert_eq!(result.copied_asset_count, 1);
    assert_eq!(result.omitted_asset_count, 0);
    assert!(fixture.target_root.join("Set.als").is_file());
}

#[cfg(any(unix, windows))]
#[test]
fn desktop_executes_incomplete_copy() {
    let fixture = fixture(true);
    let missing_path = fixture.missing_audio.to_string_lossy().to_string();
    let preview = prepare_copy(&prepare_request(&fixture));
    let result = execute_copy(&execute_request(preview, true));

    assert_eq!(result.run_status, "incomplete_copy_ready_for_manual_check");
    assert_eq!(result.copied_asset_count, 1);
    assert_eq!(result.rewritten_reference_count, 1);
    assert_eq!(result.omitted_asset_count, 1);
    assert!(fixture.target_root.join("Set.als").is_file());
    assert!(missing_path.ends_with("missing.wav"));
}

#[test]
fn changed_audio_after_preview_is_rejected_before_write() {
    let fixture = fixture(false);
    let preview = prepare_copy(&prepare_request(&fixture));
    fs::write(&fixture.source_audio, vec![b'x'; 16]).expect("resize source audio");

    let result = execute_copy(&execute_request(preview, true));

    assert_eq!(result.run_status, "preview_plan_changed");
    assert!(result
        .errors
        .iter()
        .any(|error| error.error_code == "CURRENT_PATH_PREVIEW_PLAN_CHANGED"));
    assert!(!fixture.target_root.exists());
}

#[test]
fn audio_disappearance_after_preview_is_rejected_before_write() {
    let fixture = fixture(false);
    let preview = prepare_copy(&prepare_request(&fixture));
    fs::remove_file(&fixture.source_audio).expect("remove source audio");

    let result = execute_copy(&execute_request(preview, true));

    assert_eq!(result.run_status, "preview_plan_changed");
    assert!(!fixture.target_root.exists());
}

#[test]
fn missing_audio_appearance_after_preview_is_rejected_before_write() {
    let fixture = fixture(true);
    let preview = prepare_copy(&prepare_request(&fixture));
    fs::write(&fixture.missing_audio, vec![b'm'; 13]).expect("restore missing audio");

    let result = execute_copy(&execute_request(preview, true));

    assert_eq!(result.run_status, "preview_plan_changed");
    assert!(!fixture.target_root.exists());
}

#[cfg(any(unix, windows))]
#[test]
fn same_size_current_path_replacement_remains_allowed() {
    let fixture = fixture(false);
    let preview = prepare_copy(&prepare_request(&fixture));
    let replacement = vec![b'z'; 15];
    fs::write(&fixture.source_audio, &replacement).expect("replace source audio");

    let result = execute_copy(&execute_request(preview, true));

    assert_eq!(result.run_status, "complete_copy_ready_for_manual_check");
    assert_eq!(
        fs::read(fixture.target_root.join("Samples/Imported/available.wav"))
            .expect("copied replacement"),
        replacement
    );
}

#[cfg(not(any(unix, windows)))]
#[test]
fn unsupported_atomic_replace_platform_fails_closed() {
    let fixture = fixture(false);
    let als_before = fs::read(&fixture.source_als).expect("ALS before");
    let audio_before = fs::read(&fixture.source_audio).expect("audio before");
    let preview = prepare_copy(&prepare_request(&fixture));

    let result = execute_copy(&execute_request(preview, true));

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
        fs::read(&fixture.source_audio).expect("audio after"),
        audio_before
    );
}
