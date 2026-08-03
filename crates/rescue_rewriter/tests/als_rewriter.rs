#[cfg(any(unix, windows))]
use flate2::read::GzDecoder;
use flate2::write::GzEncoder;
use flate2::Compression;
use rescue_execution::{CopyExecutionRecord, StagingExecutionMetadata, StagingExecutionResult};
use rescue_packaging::{
    CopyOperation, PackagePlan, PackagePlanMetadata, PlannedSourceAls, RewriteOperation,
};
use rescue_rewriter::{rewrite_staged_als, ALSRewriteRequest};
use sha2::{Digest, Sha256};
use std::fs;
#[cfg(any(unix, windows))]
use std::io::Read;
use std::io::Write;
#[cfg(any(unix, windows))]
use std::path::Path;
use std::path::PathBuf;
use tempfile::TempDir;

const OLD_PATH: &str = "/external/shared.wav";
const OLD_RELATIVE: &str = "../shared.wav";

struct Fixture {
    _temp: TempDir,
    staging_root: PathBuf,
    final_root: PathBuf,
    staged_als: PathBuf,
    original_bytes: Vec<u8>,
}

fn source_xml() -> String {
    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<Ableton MajorVersion="5" MinorVersion="11.0_11300" Creator="Ableton Live 11.3.43">
  <LiveSet>
    <SampleRef><FileRef><Path Value="{OLD_PATH}"/><RelativePath Value="{OLD_RELATIVE}"/><RelativePathType Value="1"/><OriginalFileSize Value="10"/></FileRef></SampleRef>
    <SampleRef><FileRef><Path Value="/external/other.wav"/><RelativePath Value="../other.wav"/><RelativePathType Value="1"/></FileRef></SampleRef>
    <OriginalFileRef><FileRef><Path Value="{OLD_PATH}"/><RelativePath Value="{OLD_RELATIVE}"/><RelativePathType Value="1"/></FileRef></OriginalFileRef>
  </LiveSet>
</Ableton>"#
    )
}

fn gzip(xml: &str) -> Vec<u8> {
    let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
    encoder.write_all(xml.as_bytes()).expect("gzip write");
    encoder.finish().expect("gzip finish")
}

#[cfg(any(unix, windows))]
fn gunzip(path: &Path) -> String {
    let bytes = fs::read(path).expect("read ALS");
    let mut decoder = GzDecoder::new(bytes.as_slice());
    let mut xml = String::new();
    decoder.read_to_string(&mut xml).expect("gunzip");
    xml
}

fn sha256(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn fixture() -> Fixture {
    let temp = tempfile::tempdir().expect("tempdir");
    let staging_root = temp.path().join("staging");
    let final_root = temp.path().join("final").join("Project");
    fs::create_dir_all(staging_root.join("Samples/Imported")).expect("staging");
    let original_bytes = gzip(&source_xml());
    let staged_als = staging_root.join("Set.als");
    fs::write(&staged_als, &original_bytes).expect("staged ALS");
    fs::write(staging_root.join("Samples/Imported/shared.wav"), b"audio").expect("staged audio");
    Fixture {
        _temp: temp,
        staging_root,
        final_root,
        staged_als,
        original_bytes,
    }
}

fn plan(fixture: &Fixture) -> PackagePlan {
    let source_hash = sha256(&fixture.original_bytes);
    let target_relative = PathBuf::from("Samples/Imported/shared.wav");
    let copy_operations = vec![
        CopyOperation {
            operation_id: "copy_als_000000".to_string(),
            operation_kind: "copy_als".to_string(),
            source_path: fixture._temp.path().join("source/Set.als"),
            target_relative_path: PathBuf::from("Set.als"),
            expected_source_sha256: Some(source_hash.clone()),
            expected_source_size: fixture.original_bytes.len() as u64,
            content_id: Some(format!("als:{source_hash}")),
            source_binding_id: format!("als:{source_hash}"),
            verification_policy: rescue_packaging::VERIFY_SHA256_AND_SIZE.to_string(),
            collision_policy: "fail_if_exists".to_string(),
            preconditions: Vec::new(),
        },
        CopyOperation {
            operation_id: "copy_audio_000000".to_string(),
            operation_kind: "copy_audio".to_string(),
            source_path: fixture._temp.path().join("source/shared.wav"),
            target_relative_path: target_relative.clone(),
            expected_source_sha256: Some("audio-hash".to_string()),
            expected_source_size: 5,
            content_id: Some("sha256:audio-hash".to_string()),
            source_binding_id: "occ:test".to_string(),
            verification_policy: rescue_packaging::VERIFY_SHA256_AND_SIZE.to_string(),
            collision_policy: "fail_if_exists".to_string(),
            preconditions: Vec::new(),
        },
    ];
    let rewrite = RewriteOperation {
        operation_id: "rewrite_active_000000".to_string(),
        required_asset_id: "asset0".to_string(),
        dependency_id: "dep0".to_string(),
        als_ref_id: 0,
        xml_locator: "SampleRef[0]/FileRef".to_string(),
        source_als_hash: source_hash.clone(),
        old_path: Some(OLD_PATH.to_string()),
        old_relative_path: Some(OLD_RELATIVE.to_string()),
        old_relative_path_type: Some("1".to_string()),
        new_path: fixture
            .final_root
            .join(&target_relative)
            .to_string_lossy()
            .to_string(),
        new_relative_path: "Samples/Imported/shared.wav".to_string(),
        new_relative_path_type: "3".to_string(),
        fields_to_change: vec![
            "Path".to_string(),
            "RelativePath".to_string(),
            "RelativePathType".to_string(),
        ],
        rule_id: "live11_3_current_paths_v0.2-lab".to_string(),
        support_status: "experimental_lab_only".to_string(),
    };
    PackagePlan {
        metadata: PackagePlanMetadata {
            planner_version: "0.1.0".to_string(),
            plan_schema_version: "0.1".to_string(),
            plan_id: "plan0".to_string(),
            planning_mode: "laboratory_rescue_rewrite".to_string(),
            source_als_hash: source_hash.clone(),
            resolution_policy_version: "0.2.0".to_string(),
            rewrite_ruleset_version: "live11_3_current_paths_v0.2-lab".to_string(),
            required_asset_count: 1,
            directory_operation_count: 0,
            copy_operation_count: 2,
            rewrite_operation_count: 1,
            system_dependency_count: 0,
            unresolved_count: 0,
            warning_count: 0,
            error_count: 0,
        },
        source_als: PlannedSourceAls {
            source_als_path: fixture._temp.path().join("source/Set.als"),
            source_file_hash: source_hash,
            source_file_size: fixture.original_bytes.len() as u64,
            target_relative_path: PathBuf::from("Set.als"),
            ableton_document_version: Some("5".to_string()),
            ableton_creator_version: Some("Ableton Live 11.3.43".to_string()),
            ableton_minor_version: Some("11.0_11300".to_string()),
        },
        target_project_root: fixture.final_root.clone(),
        directory_operations: Vec::new(),
        copy_operations,
        rewrite_operations: vec![rewrite],
        system_dependencies: Vec::new(),
        unresolved_requirements: Vec::new(),
        plan_status: "ready_for_laboratory_execution".to_string(),
        warnings: Vec::new(),
        errors: Vec::new(),
    }
}

fn staging(fixture: &Fixture, plan: &PackagePlan) -> StagingExecutionResult {
    StagingExecutionResult {
        metadata: StagingExecutionMetadata {
            executor_version: "0.1.0".to_string(),
            execution_schema_version: "0.1".to_string(),
            execution_id: "execution0".to_string(),
            plan_id: plan.metadata.plan_id.clone(),
            source_als_hash: plan.metadata.source_als_hash.clone(),
            planned_directory_count: 0,
            completed_directory_count: 0,
            planned_copy_count: 2,
            completed_copy_count: 2,
            warning_count: 0,
            error_count: 0,
        },
        staging_root: fixture.staging_root.clone(),
        staged_als_relative_path: PathBuf::from("Set.als"),
        directory_records: Vec::new(),
        copy_records: vec![CopyExecutionRecord {
            operation_id: "copy_als_000000".to_string(),
            operation_kind: "copy_als".to_string(),
            source_path: plan.source_als.source_als_path.clone(),
            target_relative_path: PathBuf::from("Set.als"),
            expected_sha256: Some(plan.source_als.source_file_hash.clone()),
            observed_sha256: Some(plan.source_als.source_file_hash.clone()),
            expected_size: fixture.original_bytes.len() as u64,
            observed_size: Some(fixture.original_bytes.len() as u64),
            verification_method: rescue_packaging::VERIFY_SHA256_AND_SIZE.to_string(),
            operation_status: "copied_and_verified".to_string(),
        }],
        execution_status: "staging_complete".to_string(),
        warnings: Vec::new(),
        errors: Vec::new(),
    }
}

fn request(fixture: &Fixture) -> ALSRewriteRequest {
    ALSRewriteRequest {
        rewrite_id: "rewrite0".to_string(),
        staging_root: fixture.staging_root.clone(),
    }
}

#[cfg(any(unix, windows))]
#[test]
fn approved_locator_changes_only_three_active_fields() {
    let fixture = fixture();
    let plan = plan(&fixture);
    let result = rewrite_staged_als(&request(&fixture), &plan, &staging(&fixture, &plan));
    let xml = gunzip(&fixture.staged_als);

    assert_eq!(result.rewrite_status, "rewrite_complete");
    assert_eq!(result.metadata.completed_operation_count, 1);
    assert!(xml.contains(&format!(
        r#"<Path Value="{}"/>"#,
        plan.rewrite_operations[0].new_path
    )));
    assert!(xml.contains(r#"<RelativePath Value="Samples/Imported/shared.wav"/>"#));
    assert!(xml.contains(r#"<RelativePathType Value="3"/>"#));
    assert_eq!(
        xml.matches(&format!(r#"<Path Value="{OLD_PATH}"/>"#))
            .count(),
        1
    );
    assert!(xml.contains(r#"<Path Value="/external/other.wav"/>"#));
}

#[cfg(any(unix, windows))]
#[test]
fn project_local_operation_changes_only_path() {
    let mut fixture = fixture();
    let old_project_path = "/old/Project/Samples/Recorded/shared.wav";
    let project_relative = "Samples/Recorded/shared.wav";
    let xml = source_xml()
        .replacen(OLD_PATH, old_project_path, 1)
        .replacen(OLD_RELATIVE, project_relative, 1)
        .replacen(
            "RelativePathType Value=\"1\"",
            "RelativePathType Value=\"3\"",
            1,
        );
    fixture.original_bytes = gzip(&xml);
    fs::write(&fixture.staged_als, &fixture.original_bytes).expect("project-local staged ALS");
    fs::create_dir_all(fixture.staging_root.join("Samples/Recorded"))
        .expect("project-local staging directory");
    fs::write(fixture.staging_root.join(project_relative), b"audio")
        .expect("project-local staged audio");

    let mut plan = plan(&fixture);
    let operation = &mut plan.rewrite_operations[0];
    operation.old_path = Some(old_project_path.to_string());
    operation.old_relative_path = Some(project_relative.to_string());
    operation.old_relative_path_type = Some("3".to_string());
    operation.new_path = fixture
        .final_root
        .join(project_relative)
        .to_string_lossy()
        .to_string();
    operation.new_relative_path = project_relative.to_string();
    operation.new_relative_path_type = "3".to_string();
    operation.fields_to_change = vec!["Path".to_string()];
    operation.support_status = "confirmed_lab".to_string();
    let expected_new_path = operation.new_path.clone();
    plan.copy_operations[1].target_relative_path = PathBuf::from(project_relative);

    let result = rewrite_staged_als(&request(&fixture), &plan, &staging(&fixture, &plan));
    let rewritten = gunzip(&fixture.staged_als);

    assert_eq!(result.rewrite_status, "rewrite_complete");
    assert!(rewritten.contains(&format!(r#"<Path Value="{expected_new_path}"/>"#)));
    assert!(rewritten.contains(&format!(r#"<RelativePath Value="{project_relative}"/>"#)));
    assert!(rewritten.contains(r#"<RelativePathType Value="3"/>"#));
}

#[test]
fn source_snapshot_mismatch_leaves_staged_als_unchanged() {
    let fixture = fixture();
    let mut plan = plan(&fixture);
    plan.source_als.source_file_hash = "wrong".to_string();
    plan.metadata.source_als_hash = "wrong".to_string();
    plan.rewrite_operations[0].source_als_hash = "wrong".to_string();
    let before = fs::read(&fixture.staged_als).expect("before");
    let result = rewrite_staged_als(&request(&fixture), &plan, &staging(&fixture, &plan));

    assert_eq!(result.rewrite_status, "rewrite_failed");
    assert_eq!(result.errors[0].error_code, "REWRITE_SOURCE_HASH_MISMATCH");
    assert_eq!(fs::read(&fixture.staged_als).expect("after"), before);
}

#[test]
fn old_value_mismatch_fails_closed() {
    let fixture = fixture();
    let mut plan = plan(&fixture);
    plan.rewrite_operations[0].old_path = Some("/wrong.wav".to_string());
    let before = fs::read(&fixture.staged_als).expect("before");
    let result = rewrite_staged_als(&request(&fixture), &plan, &staging(&fixture, &plan));

    assert_eq!(result.rewrite_status, "rewrite_failed");
    assert_eq!(result.errors[0].error_code, "REWRITE_OLD_VALUE_MISMATCH");
    assert_eq!(fs::read(&fixture.staged_als).expect("after"), before);
}

#[test]
fn locator_mismatch_fails_closed() {
    let fixture = fixture();
    let mut plan = plan(&fixture);
    plan.rewrite_operations[0].xml_locator = "SampleRef[1]/FileRef".to_string();
    let result = rewrite_staged_als(&request(&fixture), &plan, &staging(&fixture, &plan));

    assert_eq!(result.rewrite_status, "rewrite_failed");
    assert_eq!(result.errors[0].error_code, "REWRITE_LOCATOR_MISMATCH");
}

#[test]
fn unsupported_ruleset_is_rejected_before_reading_xml() {
    let fixture = fixture();
    let mut plan = plan(&fixture);
    plan.rewrite_operations[0].rule_id = "unknown".to_string();
    let result = rewrite_staged_als(&request(&fixture), &plan, &staging(&fixture, &plan));

    assert_eq!(result.rewrite_status, "rejected");
    assert!(result
        .errors
        .iter()
        .any(|error| error.error_code == "REWRITE_RULE_UNSUPPORTED"));
}

#[test]
fn incomplete_staging_is_rejected() {
    let fixture = fixture();
    let plan = plan(&fixture);
    let mut staging = staging(&fixture, &plan);
    staging.execution_status = "copy_failed".to_string();
    let result = rewrite_staged_als(&request(&fixture), &plan, &staging);

    assert_eq!(result.rewrite_status, "rejected");
    assert!(result
        .errors
        .iter()
        .any(|error| error.error_code == "REWRITE_STAGING_NOT_COMPLETE"));
}

#[test]
fn duplicate_reference_operations_are_rejected() {
    let fixture = fixture();
    let mut plan = plan(&fixture);
    let mut duplicate = plan.rewrite_operations[0].clone();
    duplicate.operation_id = "rewrite_duplicate".to_string();
    plan.rewrite_operations.push(duplicate);
    let result = rewrite_staged_als(&request(&fixture), &plan, &staging(&fixture, &plan));

    assert_eq!(result.rewrite_status, "rewrite_failed");
    assert_eq!(result.errors[0].error_code, "REWRITE_DUPLICATE_REFERENCE");
}

#[cfg(any(unix, windows))]
#[test]
fn repeated_rewrite_does_not_stack_changes() {
    let fixture = fixture();
    let plan = plan(&fixture);
    let staging = staging(&fixture, &plan);
    let first = rewrite_staged_als(&request(&fixture), &plan, &staging);
    let once = fs::read(&fixture.staged_als).expect("once");
    let second = rewrite_staged_als(&request(&fixture), &plan, &staging);

    assert_eq!(first.rewrite_status, "rewrite_complete");
    assert_eq!(second.rewrite_status, "rewrite_failed");
    assert_eq!(second.errors[0].error_code, "REWRITE_SOURCE_HASH_MISMATCH");
    assert_eq!(fs::read(&fixture.staged_als).expect("twice"), once);
}

#[cfg(any(unix, windows))]
#[test]
fn xml_special_characters_are_escaped_and_round_trip() {
    let fixture = fixture();
    let mut plan = plan(&fixture);
    plan.target_project_root = fixture._temp.path().join("A&B").join("Project");
    plan.rewrite_operations[0].new_path = plan
        .target_project_root
        .join("Samples/Imported/shared.wav")
        .to_string_lossy()
        .to_string();
    let result = rewrite_staged_als(&request(&fixture), &plan, &staging(&fixture, &plan));
    let xml = gunzip(&fixture.staged_als);

    assert_eq!(result.rewrite_status, "rewrite_complete");
    assert!(xml.contains("A&amp;B"));
    roxmltree::Document::parse(&xml).expect("valid rewritten XML");
}

#[cfg(not(any(unix, windows)))]
#[test]
fn unsupported_platform_atomic_replace_fails_closed() {
    let fixture = fixture();
    let plan = plan(&fixture);
    let before = fs::read(&fixture.staged_als).expect("before");
    let result = rewrite_staged_als(&request(&fixture), &plan, &staging(&fixture, &plan));

    assert_eq!(result.rewrite_status, "rewrite_failed");
    assert!(result
        .errors
        .iter()
        .any(|error| error.error_code == "REWRITE_ATOMIC_REPLACE_UNSUPPORTED"));
    assert_eq!(fs::read(&fixture.staged_als).expect("after"), before);
}

#[cfg(windows)]
#[test]
fn windows_atomic_replace_preserves_previous_file_on_lock() {
    use std::fs::OpenOptions;
    use std::os::windows::fs::OpenOptionsExt;
    use windows_sys::Win32::Storage::FileSystem::FILE_SHARE_READ;

    let fixture = fixture();
    let plan = plan(&fixture);
    let before = fs::read(&fixture.staged_als).expect("staged ALS before");
    let lock = OpenOptions::new()
        .read(true)
        .share_mode(FILE_SHARE_READ)
        .open(&fixture.staged_als)
        .expect("read-only sharing lock");

    let result = rewrite_staged_als(&request(&fixture), &plan, &staging(&fixture, &plan));
    drop(lock);

    assert_eq!(result.rewrite_status, "rewrite_failed");
    assert!(result
        .errors
        .iter()
        .any(|error| error.error_code == "REWRITE_WINDOWS_REPLACE_FAILED"));
    assert_eq!(
        fs::read(&fixture.staged_als).expect("staged ALS after"),
        before
    );
}

#[cfg(windows)]
#[test]
fn windows_atomic_replace_supports_unicode_and_cleans_temporary_files() {
    let mut fixture = fixture();
    let unicode_staging = fixture._temp.path().join("staging zażółć");
    fs::rename(&fixture.staging_root, &unicode_staging).expect("unicode staging rename");
    fixture.staging_root = unicode_staging;
    fixture.staged_als = fixture.staging_root.join("Set.als");
    fixture.final_root = fixture._temp.path().join("final").join("Projekt zażółć");
    let plan = plan(&fixture);

    let result = rewrite_staged_als(&request(&fixture), &plan, &staging(&fixture, &plan));

    assert_eq!(result.rewrite_status, "rewrite_complete");
    assert!(gunzip(&fixture.staged_als).contains("Projekt zażółć"));
    assert!(!fixture
        .staging_root
        .join(".Set.als.rescue-rewrite.tmp")
        .exists());
    assert!(!fixture
        .staging_root
        .join(".Set.als.rescue-rewrite.backup")
        .exists());
}
