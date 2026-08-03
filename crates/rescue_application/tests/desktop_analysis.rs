use flate2::write::GzEncoder;
use flate2::Compression;
use rescue_application::{analyze_project, DesktopAnalyzeRequest};
use sha2::{Digest, Sha256};
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use tempfile::TempDir;

struct Fixture {
    _temp: TempDir,
    root: PathBuf,
    als: PathBuf,
    private_filename: String,
}

fn fixture() -> Fixture {
    let temp = tempfile::tempdir().expect("fixture root");
    let root = temp.path().join("Private Session Project");
    let imported = root.join("Samples/Imported");
    fs::create_dir_all(root.join("Ableton Project Info")).expect("project marker");
    fs::create_dir_all(&imported).expect("sample directory");
    let private_filename = "Secret Kick.wav".to_string();
    let audio = imported.join(&private_filename);
    fs::write(&audio, b"RIFFfixture").expect("audio fixture");
    let als = root.join("Private Session.als");
    write_als(&als, &audio, &private_filename);
    Fixture {
        _temp: temp,
        root,
        als,
        private_filename,
    }
}

fn write_als(path: &Path, audio: &Path, filename: &str) {
    let xml = format!(
        r#"<Ableton MajorVersion="5" MinorVersion="11.0_11300" SchemaChangeCount="7" Creator="Ableton Live 11.3.43">
  <LiveSet><AudioClip><SampleRef><FileRef>
    <Path Value="{}"/><RelativePath Value="Samples/Imported/{}"/>
    <RelativePathType Value="1"/><Type Value="2"/>
    <OriginalFileSize Value="11"/><OriginalCrc Value="456"/>
  </FileRef><DefaultDuration Value="44.1"/><DefaultSampleRate Value="44100"/>
  </SampleRef></AudioClip></LiveSet>
</Ableton>"#,
        audio.to_string_lossy(),
        filename
    );
    let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
    encoder.write_all(xml.as_bytes()).expect("gzip input");
    let bytes = encoder.finish().expect("gzip finish");
    fs::write(path, bytes).expect("ALS fixture");
}

fn request(fixture: &Fixture) -> DesktopAnalyzeRequest {
    DesktopAnalyzeRequest {
        request_id: "desktop-test-001".to_string(),
        source_als_path: fixture.als.clone(),
    }
}

#[test]
fn valid_project_returns_desktop_preflight() {
    let fixture = fixture();
    let result = analyze_project(&request(&fixture));

    assert_eq!(result.run_status, "analysis_complete");
    assert!(result.errors.is_empty());
    let report = result.preflight_report.expect("preflight report");
    assert_eq!(report.summary.required_asset_count, 1);
    assert_eq!(report.summary.candidate_observed_count, 1);
    assert_eq!(
        report.requirements[0].filename.as_deref(),
        Some("Secret Kick.wav")
    );
}

#[test]
fn diagnostic_report_omits_private_names_and_paths() {
    let fixture = fixture();
    let result = analyze_project(&request(&fixture));
    let json = serde_json::to_string_pretty(&result.diagnostic_report).expect("diagnostic JSON");

    assert!(!json.contains(&fixture.private_filename));
    assert!(!json.contains(&fixture.root.to_string_lossy().to_string()));
    assert!(!json.contains("source_als_path"));
    assert!(!json.contains("candidate_paths"));
    assert_eq!(result.diagnostic_report.requirements.len(), 1);
}

#[test]
fn missing_als_returns_structured_failure() {
    let temp = tempfile::tempdir().expect("fixture root");
    let result = analyze_project(&DesktopAnalyzeRequest {
        request_id: "missing-001".to_string(),
        source_als_path: temp.path().join("Missing.als"),
    });

    assert_eq!(result.run_status, "analysis_failed");
    assert!(result.preflight_report.is_none());
    assert_eq!(
        result.errors[0].error_code,
        "PROJECT_DISCOVERY_SOURCE_NOT_FOUND"
    );
    assert_eq!(
        result.diagnostic_report.error_codes,
        vec!["PROJECT_DISCOVERY_SOURCE_NOT_FOUND"]
    );
}

#[test]
fn desktop_analysis_is_read_only() {
    let fixture = fixture();
    let before = tree_hash(&fixture.root);
    let result = analyze_project(&request(&fixture));
    let after = tree_hash(&fixture.root);

    assert_eq!(result.run_status, "analysis_complete");
    assert_eq!(before, after);
}

#[test]
fn desktop_analysis_is_deterministic() {
    let fixture = fixture();
    let request = request(&fixture);
    let mut first = analyze_project(&request);
    let mut second = analyze_project(&request);
    first.diagnostic_report.elapsed_ms = 0;
    second.diagnostic_report.elapsed_ms = 0;
    assert_eq!(first, second);
}

fn tree_hash(root: &Path) -> String {
    let mut paths = Vec::new();
    collect_paths(root, &mut paths);
    paths.sort();
    let mut digest = Sha256::new();
    for path in paths {
        let relative = path.strip_prefix(root).expect("path below root");
        digest.update(relative.to_string_lossy().as_bytes());
        if path.is_file() {
            digest.update(fs::read(path).expect("fixture file readable"));
        }
    }
    format!("{:x}", digest.finalize())
}

fn collect_paths(root: &Path, paths: &mut Vec<PathBuf>) {
    let mut entries: Vec<_> = fs::read_dir(root)
        .expect("fixture directory readable")
        .map(|entry| entry.expect("fixture entry").path())
        .collect();
    entries.sort();
    for path in entries {
        paths.push(path.clone());
        if path.is_dir() {
            collect_paths(&path, paths);
        }
    }
}
