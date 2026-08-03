use flate2::write::GzEncoder;
use flate2::Compression;
use rescue_application::{
    analyze_project, execute_copy, finalize_compatibility_test_report, prepare_copy,
    CompatibilityTestReportRequest, DesktopAnalyzeRequest, DesktopExecuteCopyRequest,
    DesktopPrepareCopyRequest,
};
use std::fs;
use std::io::Write;
use std::path::PathBuf;

struct Fixture {
    _temp: tempfile::TempDir,
    source_als: PathBuf,
    target_root: PathBuf,
}

fn fixture() -> Fixture {
    let temp = tempfile::tempdir().expect("tempdir");
    let source_root = temp.path().join("PRIVATE_PROJECT_SENTINEL Project");
    let destination = temp.path().join("destination");
    fs::create_dir(&source_root).expect("source root");
    fs::create_dir(source_root.join("Ableton Project Info")).expect("project marker");
    fs::create_dir(&destination).expect("destination");
    let audio = source_root.join("PRIVATE_SAMPLE_SENTINEL.wav");
    let source_als = source_root.join("PRIVATE_SET_SENTINEL.als");
    fs::write(&audio, b"private-audio").expect("audio");
    let xml = format!(
        r#"<Ableton MajorVersion="5" MinorVersion="11.0_11300" Creator="Ableton Live 11.3.43" SchemaChangeCount="7">
<LiveSet><AudioClip><SampleRef><FileRef>
<Path Value="{}"/><RelativePath Value="../PRIVATE_SAMPLE_SENTINEL.wav"/>
<RelativePathType Value="1"/><Type Value="1"/><OriginalFileSize Value="13"/><OriginalCrc Value="1"/>
</FileRef><DefaultDuration Value="1"/><DefaultSampleRate Value="44100"/></SampleRef></AudioClip></LiveSet>
</Ableton>"#,
        audio.to_string_lossy()
    );
    let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
    encoder.write_all(xml.as_bytes()).expect("gzip write");
    fs::write(&source_als, encoder.finish().expect("gzip finish")).expect("ALS");
    Fixture {
        _temp: temp,
        source_als,
        target_root: destination.join("PRIVATE_TARGET_SENTINEL Project"),
    }
}

fn reports() -> (
    rescue_application::DesktopDiagnosticReport,
    rescue_application::DesktopCopyDiagnosticReport,
) {
    let fixture = fixture();
    let analysis = analyze_project(&DesktopAnalyzeRequest {
        request_id: "compat-analysis".to_string(),
        source_als_path: fixture.source_als.clone(),
    });
    let preview = prepare_copy(&DesktopPrepareCopyRequest {
        request_id: "compat-preview".to_string(),
        source_als_path: fixture.source_als,
        target_project_root: fixture.target_root,
        experimental_compatibility_consent: false,
    });
    let result = execute_copy(&DesktopExecuteCopyRequest {
        request_id: "compat-execution".to_string(),
        preview,
        write_consent: true,
    });
    assert!(
        result.errors.is_empty(),
        "copy errors: {:#?}",
        result.errors
    );
    (analysis.diagnostic_report, result.diagnostic_report)
}

#[test]
fn compatibility_report_omits_private_paths_and_names() {
    let (analysis_report, copy_report) = reports();
    let report = finalize_compatibility_test_report(&CompatibilityTestReportRequest {
        analysis_report,
        copy_report: Some(copy_report),
        manual_verification_outcome: "not_checked".to_string(),
        tested_ableton_version: Some("Live 11.3.43".to_string()),
    })
    .expect("compatibility report");
    let json = serde_json::to_string_pretty(&report).expect("report JSON");

    assert!(!json.contains("PRIVATE_PROJECT_SENTINEL"));
    assert!(!json.contains("PRIVATE_SAMPLE_SENTINEL"));
    assert!(!json.contains("PRIVATE_SET_SENTINEL"));
    assert!(!json.contains("PRIVATE_TARGET_SENTINEL"));
}

#[test]
fn manual_outcome_controls_provisional_conclusion() {
    let (analysis_report, copy_report) = reports();
    for (outcome, expected) in [
        ("not_checked", "insufficient_evidence"),
        ("opened_without_missing_files", "candidate_success"),
        ("opened_with_missing_files", "candidate_failure"),
        ("failed_to_open", "candidate_failure"),
    ] {
        let report = finalize_compatibility_test_report(&CompatibilityTestReportRequest {
            analysis_report: analysis_report.clone(),
            copy_report: Some(copy_report.clone()),
            manual_verification_outcome: outcome.to_string(),
            tested_ableton_version: Some("Live 11.3.43".to_string()),
        })
        .expect("compatibility report");
        assert_eq!(report.provisional_conclusion, expected);
    }
}

#[test]
fn compatibility_lab_wire_contract_is_stable() {
    let (analysis_report, copy_report) = reports();
    let report = finalize_compatibility_test_report(&CompatibilityTestReportRequest {
        analysis_report,
        copy_report: Some(copy_report),
        manual_verification_outcome: "not_checked".to_string(),
        tested_ableton_version: None,
    })
    .expect("compatibility report");
    let value = serde_json::to_value(report).expect("wire JSON");
    let mut keys: Vec<_> = value
        .as_object()
        .expect("report object")
        .keys()
        .map(String::as_str)
        .collect();
    keys.sort_unstable();

    assert_eq!(
        keys,
        vec![
            "analysis_report",
            "application_profile",
            "build_commit",
            "copy_report",
            "host_arch",
            "host_os",
            "manual_verification_outcome",
            "provisional_conclusion",
            "report_schema_version",
            "tested_ableton_version",
        ]
    );
}
