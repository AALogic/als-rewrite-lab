use rescue_execution::{CopyExecutionRecord, StagingExecutionMetadata, StagingExecutionResult};
use rescue_manifest::{write_package_evidence, ManifestWriteRequest};
use rescue_packaging::{
    CopyOperation, PackagePlan, PackagePlanMetadata, PlannedSourceAls, RewriteOperation,
};
use rescue_rewriter::{ALSRewriteMetadata, ALSRewriteResult, RewriteExecutionRecord};
use rescue_validation::{
    FileValidationRecord, PackageValidationMetadata, PackageValidationResult, SemanticDiffRecord,
};
use sha2::{Digest, Sha256};
use std::fs;
use std::path::{Path, PathBuf};
use tempfile::TempDir;

struct Fixture {
    _temp: TempDir,
    staging_root: PathBuf,
    final_root: PathBuf,
    source_als: PathBuf,
    private_ledger: PathBuf,
    plan: PackagePlan,
    staging: StagingExecutionResult,
    rewrite: ALSRewriteResult,
    validation: PackageValidationResult,
}

fn digest(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn fixture() -> Fixture {
    let temp = tempfile::tempdir().expect("tempdir");
    let source_root = temp.path().join("private-source");
    let staging_root = temp.path().join("staging");
    let final_root = temp.path().join("final").join("Project");
    let evidence_root = temp.path().join("private-evidence");
    fs::create_dir_all(&source_root).expect("source root");
    fs::create_dir_all(staging_root.join("Samples/Imported")).expect("staging");
    fs::create_dir(&evidence_root).expect("evidence");
    let source_als = source_root.join("Set.als");
    let source_audio = source_root.join("shared.wav");
    let staged_als = staging_root.join("Set.als");
    let staged_audio = staging_root.join("Samples/Imported/shared.wav");
    fs::write(&source_als, b"source ALS").expect("source ALS");
    fs::write(&source_audio, b"audio").expect("source audio");
    fs::write(&staged_als, b"rewritten ALS").expect("staged ALS");
    fs::write(&staged_audio, b"audio").expect("staged audio");
    let source_hash = digest(b"source ALS");
    let rewritten_hash = digest(b"rewritten ALS");
    let audio_hash = digest(b"audio");
    let copies = vec![
        CopyOperation {
            operation_id: "copy_als".to_string(),
            operation_kind: "copy_als".to_string(),
            source_path: source_als.clone(),
            target_relative_path: PathBuf::from("Set.als"),
            expected_source_sha256: source_hash.clone(),
            expected_source_size: 10,
            content_id: format!("als:{source_hash}"),
            collision_policy: "fail_if_exists".to_string(),
            preconditions: Vec::new(),
        },
        CopyOperation {
            operation_id: "copy_audio".to_string(),
            operation_kind: "copy_audio".to_string(),
            source_path: source_audio,
            target_relative_path: PathBuf::from("Samples/Imported/shared.wav"),
            expected_source_sha256: audio_hash.clone(),
            expected_source_size: 5,
            content_id: format!("sha256:{audio_hash}"),
            collision_policy: "fail_if_exists".to_string(),
            preconditions: Vec::new(),
        },
    ];
    let rewrites = vec![RewriteOperation {
        operation_id: "rewrite0".to_string(),
        required_asset_id: "asset0".to_string(),
        dependency_id: "dep0".to_string(),
        als_ref_id: 0,
        xml_locator: "SampleRef[0]/FileRef".to_string(),
        source_als_hash: source_hash.clone(),
        old_path: Some("/private/source/shared.wav".to_string()),
        old_relative_path: Some("../shared.wav".to_string()),
        old_relative_path_type: Some("1".to_string()),
        new_path: final_root
            .join("Samples/Imported/shared.wav")
            .to_string_lossy()
            .to_string(),
        new_relative_path: "Samples/Imported/shared.wav".to_string(),
        new_relative_path_type: "3".to_string(),
        fields_to_change: vec![
            "Path".to_string(),
            "RelativePath".to_string(),
            "RelativePathType".to_string(),
        ],
        rule_id: "live11_3_external_to_imported_v0.1-experimental".to_string(),
        support_status: "experimental_lab_only".to_string(),
    }];
    let plan = PackagePlan {
        metadata: PackagePlanMetadata {
            planner_version: "0.1.0".to_string(),
            plan_schema_version: "0.1".to_string(),
            plan_id: "plan0".to_string(),
            planning_mode: "laboratory_rescue_rewrite".to_string(),
            source_als_hash: source_hash.clone(),
            resolution_policy_version: "0.2.0".to_string(),
            rewrite_ruleset_version: "live11_3_external_to_imported_v0.1-experimental".to_string(),
            required_asset_count: 1,
            copy_operation_count: 2,
            rewrite_operation_count: 1,
            unresolved_count: 0,
            warning_count: 0,
            error_count: 0,
        },
        source_als: PlannedSourceAls {
            source_als_path: source_als.clone(),
            source_file_hash: source_hash.clone(),
            source_file_size: 10,
            target_relative_path: PathBuf::from("Set.als"),
            ableton_document_version: Some("5".to_string()),
            ableton_creator_version: Some("Ableton Live 11.3.43".to_string()),
            ableton_minor_version: Some("11.0_11300".to_string()),
        },
        target_project_root: final_root.clone(),
        copy_operations: copies.clone(),
        rewrite_operations: rewrites.clone(),
        unresolved_requirements: Vec::new(),
        plan_status: "ready_for_laboratory_execution".to_string(),
        warnings: Vec::new(),
        errors: Vec::new(),
    };
    let staging = StagingExecutionResult {
        metadata: StagingExecutionMetadata {
            executor_version: "0.1.0".to_string(),
            execution_schema_version: "0.1".to_string(),
            execution_id: "execution0".to_string(),
            plan_id: "plan0".to_string(),
            source_als_hash: source_hash.clone(),
            planned_copy_count: 2,
            completed_copy_count: 2,
            warning_count: 0,
            error_count: 0,
        },
        staging_root: staging_root.clone(),
        staged_als_relative_path: PathBuf::from("Set.als"),
        copy_records: copies
            .iter()
            .map(|operation| CopyExecutionRecord {
                operation_id: operation.operation_id.clone(),
                operation_kind: operation.operation_kind.clone(),
                source_path: operation.source_path.clone(),
                target_relative_path: operation.target_relative_path.clone(),
                expected_sha256: operation.expected_source_sha256.clone(),
                observed_sha256: Some(operation.expected_source_sha256.clone()),
                expected_size: operation.expected_source_size,
                observed_size: Some(operation.expected_source_size),
                operation_status: "copied_and_verified".to_string(),
            })
            .collect(),
        execution_status: "staging_complete".to_string(),
        warnings: Vec::new(),
        errors: Vec::new(),
    };
    let rewrite = ALSRewriteResult {
        metadata: ALSRewriteMetadata {
            rewriter_version: "0.1.0".to_string(),
            rewrite_schema_version: "0.1".to_string(),
            rewrite_id: "rewrite-run0".to_string(),
            execution_id: "execution0".to_string(),
            plan_id: "plan0".to_string(),
            source_als_hash: source_hash.clone(),
            rewrite_ruleset_version: "live11_3_external_to_imported_v0.1-experimental".to_string(),
            planned_operation_count: 1,
            completed_operation_count: 1,
            warning_count: 0,
            error_count: 0,
        },
        staged_als_relative_path: PathBuf::from("Set.als"),
        original_staged_als_hash: Some(source_hash.clone()),
        rewritten_staged_als_hash: Some(rewritten_hash.clone()),
        operation_records: vec![RewriteExecutionRecord {
            operation_id: "rewrite0".to_string(),
            als_ref_id: 0,
            xml_locator: "SampleRef[0]/FileRef".to_string(),
            changed_fields: vec![
                "Path".to_string(),
                "RelativePath".to_string(),
                "RelativePathType".to_string(),
            ],
            operation_status: "rewritten_and_verified".to_string(),
        }],
        rewrite_status: "rewrite_complete".to_string(),
        warnings: Vec::new(),
        errors: Vec::new(),
    };
    let validation = PackageValidationResult {
        metadata: PackageValidationMetadata {
            validator_version: "0.1.0".to_string(),
            validation_schema_version: "0.1".to_string(),
            validation_id: "validation0".to_string(),
            plan_id: "plan0".to_string(),
            execution_id: "execution0".to_string(),
            rewrite_id: "rewrite-run0".to_string(),
            source_als_hash: source_hash,
            planned_file_count: 2,
            verified_file_count: 2,
            planned_rewrite_count: 1,
            verified_rewrite_count: 1,
            warning_count: 0,
            error_count: 0,
        },
        staging_root: staging_root.clone(),
        final_target_root: final_root.clone(),
        file_records: vec![
            FileValidationRecord {
                operation_id: "copy_als".to_string(),
                target_relative_path: PathBuf::from("Set.als"),
                expected_sha256: rewritten_hash.clone(),
                observed_sha256: Some(rewritten_hash),
                expected_size: None,
                observed_size: Some(13),
                file_status: "verified".to_string(),
            },
            FileValidationRecord {
                operation_id: "copy_audio".to_string(),
                target_relative_path: PathBuf::from("Samples/Imported/shared.wav"),
                expected_sha256: audio_hash.clone(),
                observed_sha256: Some(audio_hash),
                expected_size: Some(5),
                observed_size: Some(5),
                file_status: "verified".to_string(),
            },
        ],
        semantic_diff_records: vec![SemanticDiffRecord {
            operation_id: "rewrite0".to_string(),
            als_ref_id: 0,
            xml_locator: "SampleRef[0]/FileRef".to_string(),
            verified_fields: vec![
                "Path".to_string(),
                "RelativePath".to_string(),
                "RelativePathType".to_string(),
            ],
            diff_status: "verified_allowed_change".to_string(),
        }],
        validation_status: "validation_passed".to_string(),
        warnings: Vec::new(),
        errors: Vec::new(),
    };
    Fixture {
        _temp: temp,
        staging_root,
        final_root,
        source_als,
        private_ledger: evidence_root.join("private-ledger.json"),
        plan,
        staging,
        rewrite,
        validation,
    }
}

fn request(fixture: &Fixture) -> ManifestWriteRequest {
    ManifestWriteRequest {
        manifest_id: "manifest0".to_string(),
        private_ledger_path: fixture.private_ledger.clone(),
        package_manifest_relative_path: PathBuf::from("Rescue Manifest/package-manifest.json"),
    }
}

fn write(fixture: &Fixture) -> rescue_manifest::ManifestWriteResult {
    write_package_evidence(
        &request(fixture),
        &fixture.plan,
        &fixture.staging,
        &fixture.rewrite,
        &fixture.validation,
    )
}

#[test]
fn passed_validation_writes_portable_manifest_and_private_ledger() {
    let fixture = fixture();
    let result = write(&fixture);

    assert_eq!(result.write_status, "manifests_written");
    assert_eq!(result.write_records.len(), 2);
    assert!(fixture.private_ledger.exists());
    assert!(fixture
        .staging_root
        .join("Rescue Manifest/package-manifest.json")
        .exists());
}

#[test]
fn portable_manifest_contains_no_absolute_local_paths() {
    let fixture = fixture();
    let result = write(&fixture);
    assert_eq!(result.write_status, "manifests_written");
    let text = fs::read_to_string(
        fixture
            .staging_root
            .join("Rescue Manifest/package-manifest.json"),
    )
    .expect("portable manifest");

    assert!(!text.contains(fixture._temp.path().to_string_lossy().as_ref()));
    assert!(!text.contains(fixture.source_als.to_string_lossy().as_ref()));
    assert!(!text.contains(fixture.final_root.to_string_lossy().as_ref()));
    assert!(!text.contains("/private/source/shared.wav"));
    assert!(text.contains("Samples/Imported/shared.wav"));
}

#[test]
fn private_ledger_retains_full_audit_paths() {
    let fixture = fixture();
    let result = write(&fixture);
    assert_eq!(result.write_status, "manifests_written");
    let text = fs::read_to_string(&fixture.private_ledger).expect("private ledger");

    assert!(text.contains(fixture.source_als.to_string_lossy().as_ref()));
    assert!(text.contains(fixture.staging_root.to_string_lossy().as_ref()));
    assert!(text.contains("/private/source/shared.wav"));
}

#[test]
fn repeated_identical_write_is_idempotent() {
    let fixture = fixture();
    let first = write(&fixture);
    let second = write(&fixture);

    assert_eq!(first.write_status, "manifests_written");
    assert_eq!(second.write_status, "manifests_written");
    assert!(second
        .write_records
        .iter()
        .all(|record| record.write_status == "already_present_verified"));
}

#[test]
fn conflicting_existing_manifest_is_not_overwritten() {
    let fixture = fixture();
    let target = fixture
        .staging_root
        .join("Rescue Manifest/package-manifest.json");
    fs::create_dir_all(target.parent().expect("parent")).expect("manifest parent");
    fs::write(&target, b"do not overwrite").expect("conflict");
    let result = write(&fixture);

    assert_eq!(result.write_status, "manifest_write_failed");
    assert_eq!(result.errors[0].error_code, "MANIFEST_TARGET_CONFLICT");
    assert_eq!(
        fs::read(target).expect("conflict remains"),
        b"do not overwrite"
    );
    assert!(!fixture.private_ledger.exists());
}

#[test]
fn failed_validation_blocks_all_manifest_writes() {
    let mut fixture = fixture();
    fixture.validation.validation_status = "validation_failed".to_string();
    let result = write(&fixture);

    assert_eq!(result.write_status, "manifest_write_failed");
    assert!(result
        .errors
        .iter()
        .any(|error| error.error_code == "MANIFEST_VALIDATION_NOT_PASSED"));
    assert!(!fixture.private_ledger.exists());
    assert!(!fixture.staging_root.join("Rescue Manifest").exists());
}

#[test]
fn unsafe_package_manifest_path_is_rejected() {
    let fixture = fixture();
    let mut request = request(&fixture);
    request.package_manifest_relative_path = PathBuf::from("../leak.json");
    let result = write_package_evidence(
        &request,
        &fixture.plan,
        &fixture.staging,
        &fixture.rewrite,
        &fixture.validation,
    );

    assert_eq!(result.write_status, "manifest_write_failed");
    assert!(result
        .errors
        .iter()
        .any(|error| error.error_code == "PACKAGE_MANIFEST_PATH_UNSAFE"));
}

#[test]
fn package_manifest_cannot_replace_planned_file() {
    let fixture = fixture();
    let mut request = request(&fixture);
    request.package_manifest_relative_path = PathBuf::from("Set.als");
    let result = write_package_evidence(
        &request,
        &fixture.plan,
        &fixture.staging,
        &fixture.rewrite,
        &fixture.validation,
    );

    assert_eq!(result.write_status, "manifest_write_failed");
    assert!(result
        .errors
        .iter()
        .any(|error| error.error_code == "PACKAGE_MANIFEST_TARGET_COLLISION"));
}

#[test]
fn existing_final_target_blocks_manifest_write() {
    let fixture = fixture();
    fs::create_dir_all(&fixture.final_root).expect("final target");
    let result = write(&fixture);

    assert_eq!(result.write_status, "manifest_write_failed");
    assert!(result
        .errors
        .iter()
        .any(|error| error.error_code == "MANIFEST_FINAL_TARGET_EXISTS"));
}

#[test]
fn successful_atomic_writes_leave_no_temporary_files() {
    let fixture = fixture();
    let result = write(&fixture);
    assert_eq!(result.write_status, "manifests_written");

    assert!(!has_temp_file(&fixture.staging_root));
    assert!(!has_temp_file(
        fixture.private_ledger.parent().expect("ledger parent")
    ));
}

fn has_temp_file(root: &Path) -> bool {
    let mut stack = vec![root.to_path_buf()];
    while let Some(directory) = stack.pop() {
        for entry in fs::read_dir(directory).expect("tree") {
            let path = entry.expect("entry").path();
            if path.is_dir() {
                stack.push(path);
            } else if path
                .file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| name.ends_with(".rescue-write.tmp"))
            {
                return true;
            }
        }
    }
    false
}
