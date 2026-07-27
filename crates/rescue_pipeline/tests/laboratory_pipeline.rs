use flate2::write::GzEncoder;
use flate2::Compression;
use rescue_pipeline::{run_laboratory_package, LaboratoryPackageRequest};
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use tempfile::TempDir;

struct Fixture {
    _temp: TempDir,
    source_root: PathBuf,
    source_als: PathBuf,
    source_audio: PathBuf,
    staging_root: PathBuf,
    target_root: PathBuf,
    ledger_path: PathBuf,
}

fn gzip(xml: &str) -> Vec<u8> {
    let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
    encoder.write_all(xml.as_bytes()).expect("gzip write");
    encoder.finish().expect("gzip finish")
}

fn xml_for(path: &Path, minor: &str, creator: &str) -> String {
    let path = path.to_string_lossy();
    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<Ableton MajorVersion="5" MinorVersion="{minor}" Creator="{creator}" SchemaChangeCount="7">
  <LiveSet>
    <SampleRef>
      <FileRef>
        <Path Value="{path}"/>
        <RelativePath Value="../shared.wav"/>
        <RelativePathType Value="1"/>
        <Type Value="1"/>
        <OriginalFileSize Value="5"/>
        <OriginalCrc Value="123"/>
      </FileRef>
      <DefaultDuration Value="10"/>
      <DefaultSampleRate Value="44100"/>
    </SampleRef>
  </LiveSet>
</Ableton>"#
    )
}

fn zero_reference_xml(minor: &str, creator: &str) -> String {
    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<Ableton MajorVersion="5" MinorVersion="{minor}" Creator="{creator}" SchemaChangeCount="7">
  <LiveSet/>
</Ableton>"#
    )
}

fn fixture() -> Fixture {
    let temp = tempfile::tempdir().expect("tempdir");
    let source_root = temp.path().join("source");
    let output_root = temp.path().join("output");
    let evidence_root = temp.path().join("evidence");
    fs::create_dir(&source_root).expect("source");
    fs::create_dir(source_root.join("Ableton Project Info")).expect("project marker");
    fs::create_dir(&output_root).expect("output");
    fs::create_dir(&evidence_root).expect("evidence");
    let source_als = source_root.join("Set.als");
    let source_audio = source_root.join("shared.wav");
    fs::write(&source_audio, b"audio").expect("audio");
    fs::write(
        &source_als,
        gzip(&xml_for(
            &source_audio,
            "11.0_11300",
            "Ableton Live 11.3.43",
        )),
    )
    .expect("ALS");
    Fixture {
        _temp: temp,
        source_root,
        source_als,
        source_audio,
        staging_root: output_root.join("Project.staging"),
        target_root: output_root.join("Project"),
        ledger_path: evidence_root.join("private-ledger.json"),
    }
}

fn request(fixture: &Fixture) -> LaboratoryPackageRequest {
    LaboratoryPackageRequest {
        run_id: "lab-run0".to_string(),
        source_als_path: fixture.source_als.clone(),
        scan_roots: vec![fixture.source_root.clone()],
        max_scan_entries: 10_000,
        staging_root: fixture.staging_root.clone(),
        target_project_root: fixture.target_root.clone(),
        private_ledger_path: fixture.ledger_path.clone(),
    }
}

#[test]
fn complete_pipeline_blocks_unconfirmed_candidate_before_staging() {
    let fixture = fixture();
    let source_als_before = fs::read(&fixture.source_als).expect("ALS before");
    let source_audio_before = fs::read(&fixture.source_audio).expect("audio before");
    let result = run_laboratory_package(&request(&fixture));
    assert_eq!(result.run_status, "resolution_or_plan_blocked");
    assert_eq!(result.errors[0].error_code, "PIPELINE_PACKAGE_PLAN_BLOCKED");
    assert_eq!(
        result.resolution.as_ref().expect("resolution").decisions[0].decision_status,
        "needs_user_confirmation"
    );
    assert!(!fixture.staging_root.exists());
    assert!(!fixture.target_root.exists());
    assert!(!fixture.ledger_path.exists());
    assert_eq!(
        fs::read(&fixture.source_als).expect("ALS after"),
        source_als_before
    );
    assert_eq!(
        fs::read(&fixture.source_audio).expect("audio after"),
        source_audio_before
    );
}

#[test]
fn confirmed_project_root_is_preserved_in_blocked_result() {
    let fixture = fixture();
    let result = run_laboratory_package(&request(&fixture));

    assert_eq!(result.run_status, "resolution_or_plan_blocked");
    assert_eq!(
        result
            .discovery
            .as_ref()
            .expect("discovery")
            .confirmed_project_root
            .as_deref(),
        Some(fixture.source_root.as_path())
    );
    assert_eq!(result.completed_stage, "package_planning");
}

#[test]
fn zero_reference_project_blocks_before_staging() {
    let fixture = fixture();
    fs::write(
        &fixture.source_als,
        gzip(&zero_reference_xml("11.0_11300", "Ableton Live 11.3.43")),
    )
    .expect("ALS");
    let result = run_laboratory_package(&request(&fixture));

    assert_eq!(result.run_status, "resolution_or_plan_blocked");
    assert!(result
        .package_plan
        .as_ref()
        .expect("plan")
        .errors
        .iter()
        .any(|error| error.error_code == "PACKAGE_REWRITE_OPERATIONS_EMPTY"));
    assert!(result.staging.is_none());
    assert!(!fixture.staging_root.exists());
    assert!(!fixture.target_root.exists());
    assert!(!fixture.ledger_path.exists());
}

#[test]
fn missing_sample_blocks_before_staging() {
    let fixture = fixture();
    fs::write(
        &fixture.source_als,
        gzip(&xml_for(
            &fixture.source_root.join("missing.wav"),
            "11.0_11300",
            "Ableton Live 11.3.43",
        )),
    )
    .expect("ALS");
    fs::remove_file(&fixture.source_audio).expect("remove test audio");
    let result = run_laboratory_package(&request(&fixture));

    assert_eq!(result.run_status, "resolution_or_plan_blocked");
    assert_eq!(result.errors[0].error_code, "PIPELINE_PACKAGE_PLAN_BLOCKED");
    assert!(!fixture.staging_root.exists());
    assert!(!fixture.target_root.exists());
    assert!(!fixture.ledger_path.exists());
}

#[test]
fn ambiguous_same_name_candidates_block_before_staging() {
    let fixture = fixture();
    let missing = fixture.source_root.join("missing").join("shared.wav");
    fs::write(
        &fixture.source_als,
        gzip(&xml_for(&missing, "11.0_11300", "Ableton Live 11.3.43")),
    )
    .expect("ALS");
    let root_a = fixture._temp.path().join("library-a");
    let root_b = fixture._temp.path().join("library-b");
    fs::create_dir(&root_a).expect("root a");
    fs::create_dir(&root_b).expect("root b");
    fs::write(root_a.join("shared.wav"), b"aaaaa").expect("a");
    fs::write(root_b.join("shared.wav"), b"bbbbb").expect("b");
    let mut request = request(&fixture);
    request.scan_roots = vec![root_a, root_b];
    let result = run_laboratory_package(&request);

    assert_eq!(result.run_status, "resolution_or_plan_blocked");
    assert!(result
        .resolution
        .as_ref()
        .expect("resolution")
        .decisions
        .iter()
        .all(|decision| decision.decision_status != "auto_accepted"));
    assert!(!fixture.staging_root.exists());
}

#[test]
fn existing_target_is_rejected_before_any_write() {
    let fixture = fixture();
    fs::create_dir(&fixture.target_root).expect("existing target");
    let marker = fixture.target_root.join("keep.txt");
    fs::write(&marker, b"keep").expect("marker");
    let result = run_laboratory_package(&request(&fixture));

    assert_eq!(result.run_status, "rejected_before_read");
    assert!(result
        .errors
        .iter()
        .any(|error| error.error_code == "PIPELINE_OUTPUT_ALREADY_EXISTS"));
    assert_eq!(fs::read(marker).expect("marker"), b"keep");
    assert!(!fixture.staging_root.exists());
}

#[cfg(unix)]
#[test]
fn dangling_output_symlink_is_rejected_before_any_write() {
    use std::os::unix::fs::symlink;

    let fixture = fixture();
    symlink(
        fixture._temp.path().join("missing-target"),
        &fixture.target_root,
    )
    .expect("dangling target symlink");
    let result = run_laboratory_package(&request(&fixture));

    assert_eq!(result.run_status, "rejected_before_read");
    assert!(result
        .errors
        .iter()
        .any(|error| error.error_code == "PIPELINE_OUTPUT_ALREADY_EXISTS"));
    assert!(fs::symlink_metadata(&fixture.target_root).is_ok());
    assert!(!fixture.staging_root.exists());
    assert!(!fixture.ledger_path.exists());
}

#[test]
fn outputs_inside_confirmed_project_root_are_rejected() {
    for output in ["staging", "target", "ledger"] {
        let fixture = fixture();
        let mut request = request(&fixture);
        match output {
            "staging" => request.staging_root = fixture.source_root.join("unsafe.staging"),
            "target" => request.target_project_root = fixture.source_root.join("unsafe-target"),
            "ledger" => {
                request.private_ledger_path = fixture.source_root.join("unsafe-ledger.json")
            }
            _ => unreachable!(),
        }
        let result = run_laboratory_package(&request);

        assert_eq!(result.run_status, "read_stage_failed");
        assert!(result
            .errors
            .iter()
            .any(|error| error.error_code == "PIPELINE_OUTPUT_INSIDE_SOURCE_PROJECT"));
        assert_eq!(
            result
                .discovery
                .as_ref()
                .expect("discovery")
                .confirmed_project_root
                .as_deref(),
            Some(fixture.source_root.as_path())
        );
        assert!(result.inventory.is_none());
    }
}

#[cfg(unix)]
#[test]
fn ancestor_symlink_into_project_is_rejected() {
    use std::os::unix::fs::symlink;

    let fixture = fixture();
    let alias_root = fixture._temp.path().join("alias");
    symlink(fixture._temp.path(), &alias_root).expect("ancestor alias");
    let mut request = request(&fixture);
    request.staging_root = alias_root.join("source").join("unsafe.staging");
    let result = run_laboratory_package(&request);

    assert_eq!(result.run_status, "read_stage_failed");
    assert!(result
        .errors
        .iter()
        .any(|error| error.error_code == "PIPELINE_OUTPUT_INSIDE_SOURCE_PROJECT"));
    assert_eq!(
        result
            .discovery
            .as_ref()
            .expect("discovery")
            .confirmed_project_root
            .as_deref(),
        Some(fixture.source_root.as_path())
    );
    assert!(!request.staging_root.exists());
    assert!(!fixture.target_root.exists());
    assert!(!fixture.ledger_path.exists());
}

#[cfg(any(target_os = "macos", target_os = "windows"))]
#[test]
fn case_variant_project_path_is_rejected() {
    let fixture = fixture();
    let case_variant_root = fixture._temp.path().join("SOURCE");
    if fs::symlink_metadata(&case_variant_root).is_err() {
        return;
    }
    let mut request = request(&fixture);
    request.target_project_root = case_variant_root.join("unsafe-target");
    let result = run_laboratory_package(&request);

    assert_eq!(result.run_status, "read_stage_failed");
    assert!(result
        .errors
        .iter()
        .any(|error| error.error_code == "PIPELINE_OUTPUT_INSIDE_SOURCE_PROJECT"));
    assert!(result.discovery.is_some());
    assert!(!fixture.staging_root.exists());
    assert!(!request.target_project_root.exists());
    assert!(!fixture.ledger_path.exists());
}

#[test]
fn unbounded_filesystem_root_is_rejected() {
    let fixture = fixture();
    let mut request = request(&fixture);
    request.scan_roots = vec![PathBuf::from("/")];
    let result = run_laboratory_package(&request);

    assert_eq!(result.run_status, "rejected_before_read");
    assert!(result
        .errors
        .iter()
        .any(|error| error.error_code == "PIPELINE_SCAN_ROOT_UNSAFE"));
}

#[test]
fn unsupported_live_version_blocks_before_staging() {
    let fixture = fixture();
    fs::write(
        &fixture.source_als,
        gzip(&xml_for(
            &fixture.source_audio,
            "12.0_12000",
            "Ableton Live 12.0.1",
        )),
    )
    .expect("ALS");
    let result = run_laboratory_package(&request(&fixture));

    assert_eq!(result.run_status, "resolution_or_plan_blocked");
    assert!(result
        .package_plan
        .as_ref()
        .expect("plan")
        .errors
        .iter()
        .any(|error| error.error_code == "PACKAGE_REWRITE_DOCUMENT_UNSUPPORTED"));
    assert!(!fixture.staging_root.exists());
}

#[test]
fn unknown_project_root_blocks_before_inventory_and_writes() {
    let fixture = fixture();
    fs::remove_dir(fixture.source_root.join("Ableton Project Info"))
        .expect("remove synthetic project marker");
    let result = run_laboratory_package(&request(&fixture));

    assert_eq!(result.run_status, "read_stage_failed");
    assert_eq!(
        result.errors[0].error_code,
        "PIPELINE_PROJECT_ROOT_UNCONFIRMED"
    );
    assert_eq!(
        result
            .discovery
            .as_ref()
            .expect("discovery")
            .discovery_status,
        "unknown"
    );
    assert!(result.inventory.is_none());
    assert!(!fixture.staging_root.exists());
    assert!(!fixture.target_root.exists());
    assert!(!fixture.ledger_path.exists());
}
