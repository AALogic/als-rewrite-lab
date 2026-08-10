use rescue_execution::{execute_staging, StagingExecutionRequest};
use rescue_packaging::{
    CopyOperation, CreateDirectoryOperation, PackagePlan, PackagePlanMetadata, PlannedSourceAls,
};
use sha2::{Digest, Sha256};
use std::fs;
use std::path::{Path, PathBuf};
use tempfile::TempDir;

struct Fixture {
    _temp: TempDir,
    source_als: PathBuf,
    source_audio: PathBuf,
    staging_root: PathBuf,
    final_root: PathBuf,
}

fn fixture() -> Fixture {
    let temp = tempfile::tempdir().expect("tempdir");
    let source_root = temp.path().join("source");
    fs::create_dir(&source_root).expect("source root");
    let source_als = source_root.join("Set.als");
    let source_audio = source_root.join("sample.wav");
    fs::write(&source_als, b"als fixture").expect("write ALS");
    fs::write(&source_audio, b"audio fixture").expect("write audio");
    let staging_root = temp.path().join("staging");
    let final_root = temp.path().join("final");
    Fixture {
        _temp: temp,
        source_als,
        source_audio,
        staging_root,
        final_root,
    }
}

fn digest(path: &Path) -> String {
    format!(
        "{:x}",
        Sha256::digest(fs::read(path).expect("read fixture"))
    )
}

fn operation(id: &str, kind: &str, source: &Path, target: &str) -> CopyOperation {
    CopyOperation {
        operation_id: id.to_string(),
        operation_kind: kind.to_string(),
        source_path: source.to_path_buf(),
        target_relative_path: PathBuf::from(target),
        expected_source_sha256: Some(digest(source)),
        expected_source_size: fs::metadata(source).expect("metadata").len(),
        content_id: Some(format!("sha256:{}", digest(source))),
        source_binding_id: format!("test:{id}"),
        verification_policy: rescue_packaging::VERIFY_SHA256_AND_SIZE.to_string(),
        collision_policy: "fail_if_exists".to_string(),
        preconditions: vec!["source_hash_matches_plan".to_string()],
    }
}

fn plan(fixture: &Fixture) -> PackagePlan {
    let als_hash = digest(&fixture.source_als);
    let operations = vec![
        operation(
            "copy_als_000000",
            "copy_als",
            &fixture.source_als,
            "Set.als",
        ),
        operation(
            "copy_audio_000000",
            "copy_audio",
            &fixture.source_audio,
            "Samples/Imported/sample.wav",
        ),
    ];
    let directories = vec![
        directory(
            "create_project_info",
            "Ableton Project Info",
            "ableton_project_marker",
        ),
        directory("create_samples", "Samples", "ableton_samples_root"),
        directory(
            "create_imported_samples",
            "Samples/Imported",
            "imported_audio_root",
        ),
    ];
    PackagePlan {
        metadata: PackagePlanMetadata {
            planner_version: "0.1.0".to_string(),
            plan_schema_version: "0.1".to_string(),
            plan_id: "plan0".to_string(),
            planning_mode: "laboratory_rescue_rewrite".to_string(),
            source_als_hash: als_hash.clone(),
            resolution_policy_version: "0.2.0".to_string(),
            rewrite_ruleset_version: "live11_3_current_paths_v0.2-lab".to_string(),
            required_asset_count: 1,
            directory_operation_count: directories.len(),
            copy_operation_count: operations.len(),
            rewrite_operation_count: 0,
            system_dependency_count: 0,
            unresolved_count: 0,
            warning_count: 0,
            error_count: 0,
        },
        source_als: PlannedSourceAls {
            source_als_path: fixture.source_als.clone(),
            source_file_hash: als_hash,
            source_file_size: fs::metadata(&fixture.source_als).expect("metadata").len(),
            target_relative_path: PathBuf::from("Set.als"),
            ableton_document_version: Some("5".to_string()),
            ableton_creator_version: Some("Ableton Live 11.3.43".to_string()),
            ableton_minor_version: Some("11.0_11300".to_string()),
        },
        target_project_root: fixture.final_root.clone(),
        directory_operations: directories,
        copy_operations: operations,
        rewrite_operations: Vec::new(),
        system_dependencies: Vec::new(),
        unresolved_requirements: Vec::new(),
        plan_status: "ready_for_laboratory_execution".to_string(),
        warnings: Vec::new(),
        errors: Vec::new(),
    }
}

fn directory(id: &str, path: &str, purpose: &str) -> CreateDirectoryOperation {
    CreateDirectoryOperation {
        operation_id: id.to_string(),
        target_relative_path: PathBuf::from(path),
        purpose: purpose.to_string(),
        collision_policy: "fail_if_exists".to_string(),
    }
}

fn request(fixture: &Fixture) -> StagingExecutionRequest {
    StagingExecutionRequest {
        execution_id: "execution0".to_string(),
        staging_root: fixture.staging_root.clone(),
    }
}

#[test]
fn ready_plan_is_copied_and_hash_verified_in_new_staging_root() {
    let fixture = fixture();
    let result = execute_staging(&request(&fixture), &plan(&fixture));

    assert_eq!(result.execution_status, "staging_complete");
    assert_eq!(result.metadata.completed_directory_count, 3);
    assert_eq!(result.metadata.completed_copy_count, 2);
    assert!(fixture.staging_root.join("Ableton Project Info").is_dir());
    assert_eq!(
        fs::read(fixture.staging_root.join("Set.als")).expect("staged ALS"),
        b"als fixture"
    );
    assert_eq!(
        fs::read(fixture.staging_root.join("Samples/Imported/sample.wav")).expect("staged audio"),
        b"audio fixture"
    );
}

#[test]
fn metadata_only_audio_copy_records_no_content_hash() {
    let fixture = fixture();
    let mut plan = plan(&fixture);
    let audio = plan
        .copy_operations
        .iter_mut()
        .find(|operation| operation.operation_kind == "copy_audio")
        .expect("audio operation");
    audio.expected_source_sha256 = None;
    audio.content_id = None;
    audio.verification_policy = rescue_packaging::VERIFY_STABLE_SOURCE_AND_SIZE.to_string();

    let result = execute_staging(&request(&fixture), &plan);
    let record = result
        .copy_records
        .iter()
        .find(|record| record.operation_kind == "copy_audio")
        .expect("audio record");

    assert_eq!(result.execution_status, "staging_complete");
    assert_eq!(record.expected_sha256, None);
    assert_eq!(record.observed_sha256, None);
    assert_eq!(
        record.verification_method,
        rescue_packaging::VERIFY_STABLE_SOURCE_AND_SIZE
    );
}

#[test]
fn source_hash_mismatch_fails_before_target_promotion() {
    let fixture = fixture();
    let mut plan = plan(&fixture);
    plan.copy_operations[0].expected_source_sha256 = Some("wrong".to_string());
    let result = execute_staging(&request(&fixture), &plan);

    assert_eq!(result.execution_status, "copy_failed");
    assert_eq!(result.errors[0].error_code, "COPY_HASH_MISMATCH");
    assert!(!fixture.staging_root.join("Set.als").exists());
}

#[test]
fn existing_staging_root_is_rejected_without_writes() {
    let fixture = fixture();
    fs::create_dir(&fixture.staging_root).expect("existing staging");
    let marker = fixture.staging_root.join("keep.txt");
    fs::write(&marker, b"keep").expect("marker");
    let result = execute_staging(&request(&fixture), &plan(&fixture));

    assert_eq!(result.execution_status, "rejected");
    assert_eq!(result.errors[0].error_code, "STAGING_ROOT_EXISTS");
    assert_eq!(fs::read(marker).expect("marker remains"), b"keep");
}

#[test]
fn unsafe_relative_target_is_rejected() {
    let fixture = fixture();
    let mut plan = plan(&fixture);
    plan.copy_operations[0].target_relative_path = PathBuf::from("../escape.als");
    let result = execute_staging(&request(&fixture), &plan);

    assert_eq!(result.execution_status, "rejected");
    assert!(result
        .errors
        .iter()
        .any(|error| error.error_code == "STAGING_TARGET_PATH_UNSAFE"));
    assert!(!fixture.staging_root.exists());
}

#[test]
fn blocked_plan_is_rejected() {
    let fixture = fixture();
    let mut plan = plan(&fixture);
    plan.plan_status = "blocked".to_string();
    let result = execute_staging(&request(&fixture), &plan);

    assert_eq!(result.execution_status, "rejected");
    assert!(result
        .errors
        .iter()
        .any(|error| error.error_code == "STAGING_PLAN_NOT_READY"));
}

#[test]
fn missing_ableton_project_marker_in_plan_is_rejected_before_writes() {
    let fixture = fixture();
    let mut plan = plan(&fixture);
    plan.directory_operations
        .retain(|operation| operation.purpose != "ableton_project_marker");
    plan.metadata.directory_operation_count = plan.directory_operations.len();
    let result = execute_staging(&request(&fixture), &plan);

    assert_eq!(result.execution_status, "rejected");
    assert!(result
        .errors
        .iter()
        .any(|error| { error.error_code == "STAGING_ABLETON_PROJECT_MARKER_NOT_PLANNED" }));
    assert!(!fixture.staging_root.exists());
}

#[test]
fn repeated_execution_reports_existing_staging_instead_of_overwriting() {
    let fixture = fixture();
    let first = execute_staging(&request(&fixture), &plan(&fixture));
    let second = execute_staging(&request(&fixture), &plan(&fixture));

    assert_eq!(first.execution_status, "staging_complete");
    assert_eq!(second.execution_status, "rejected");
    assert!(second
        .errors
        .iter()
        .any(|error| error.error_code == "STAGING_ROOT_EXISTS"));
}

#[test]
fn final_target_and_sources_remain_untouched() {
    let fixture = fixture();
    let als_before = fs::read(&fixture.source_als).expect("source ALS");
    let audio_before = fs::read(&fixture.source_audio).expect("source audio");
    let result = execute_staging(&request(&fixture), &plan(&fixture));

    assert_eq!(result.execution_status, "staging_complete");
    assert!(!fixture.final_root.exists());
    assert_eq!(
        fs::read(&fixture.source_als).expect("source ALS"),
        als_before
    );
    assert_eq!(
        fs::read(&fixture.source_audio).expect("source audio"),
        audio_before
    );
}

#[cfg(unix)]
#[test]
fn symlink_source_is_rejected() {
    use std::os::unix::fs::symlink;

    let fixture = fixture();
    let link = fixture.source_audio.with_file_name("linked.wav");
    symlink(&fixture.source_audio, &link).expect("symlink");
    let mut plan = plan(&fixture);
    plan.copy_operations[0] = operation("copy_link", "copy_audio", &link, "linked.wav");
    let result = execute_staging(&request(&fixture), &plan);

    assert_eq!(result.execution_status, "copy_failed");
    assert_eq!(result.errors[0].error_code, "COPY_SOURCE_NOT_REGULAR_FILE");
}
