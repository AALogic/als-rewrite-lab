use rescue_execution::{
    CopyExecutionRecord, DirectoryExecutionRecord, StagingExecutionMetadata, StagingExecutionResult,
};
use rescue_manifest::{write_package_evidence, ManifestWriteRequest, PrivateLedger};
use rescue_packaging::{
    CopyOperation, CreateDirectoryOperation, PackagePlan, PackagePlanMetadata, PlannedSourceAls,
    RewriteOperation, SystemDependencyRequirement,
};
use rescue_rewriter::{ALSRewriteMetadata, ALSRewriteResult, RewriteExecutionRecord};
use rescue_validation::{
    DirectoryValidationRecord, FileValidationRecord, PackageValidationMetadata,
    PackageValidationResult, SemanticDiffRecord,
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

fn directory(id: &str, path: &str, purpose: &str) -> CreateDirectoryOperation {
    CreateDirectoryOperation {
        operation_id: id.to_string(),
        target_relative_path: PathBuf::from(path),
        purpose: purpose.to_string(),
        collision_policy: "fail_if_exists".to_string(),
    }
}

fn fixture() -> Fixture {
    let temp = tempfile::tempdir().expect("tempdir");
    let source_root = temp.path().join("private-source");
    let staging_root = temp.path().join("staging");
    let final_root = temp.path().join("final").join("Project");
    let evidence_root = temp.path().join("private-evidence");
    fs::create_dir_all(&source_root).expect("source root");
    fs::create_dir_all(staging_root.join("Samples/Imported")).expect("staging");
    fs::create_dir(staging_root.join("Ableton Project Info")).expect("project marker");
    fs::create_dir(&evidence_root).expect("evidence");
    let source_als = source_root.join("Set.als");
    let source_audio = source_root.join("shared.wav");
    let system_audio = source_root.join("Core Library Kick.wav");
    let staged_als = staging_root.join("Set.als");
    let staged_audio = staging_root.join("Samples/Imported/shared.wav");
    fs::write(&source_als, b"source ALS").expect("source ALS");
    fs::write(&source_audio, b"audio").expect("source audio");
    fs::write(&system_audio, b"system audio").expect("system audio");
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
            expected_source_sha256: Some(source_hash.clone()),
            expected_source_size: 10,
            content_id: Some(format!("als:{source_hash}")),
            source_binding_id: format!("als:{source_hash}"),
            verification_policy: rescue_packaging::VERIFY_SHA256_AND_SIZE.to_string(),
            collision_policy: "fail_if_exists".to_string(),
            preconditions: Vec::new(),
        },
        CopyOperation {
            operation_id: "copy_audio".to_string(),
            operation_kind: "copy_audio".to_string(),
            source_path: source_audio,
            target_relative_path: PathBuf::from("Samples/Imported/shared.wav"),
            expected_source_sha256: Some(audio_hash.clone()),
            expected_source_size: 5,
            content_id: Some(format!("sha256:{audio_hash}")),
            source_binding_id: "occ:audio".to_string(),
            verification_policy: rescue_packaging::VERIFY_SHA256_AND_SIZE.to_string(),
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
        rule_id: "live11_3_current_paths_v0.2-lab".to_string(),
        support_status: "experimental_lab_only".to_string(),
    }];
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
    let plan = PackagePlan {
        metadata: PackagePlanMetadata {
            planner_version: "0.1.0".to_string(),
            plan_schema_version: "0.1".to_string(),
            plan_id: "plan0".to_string(),
            planning_mode: "laboratory_rescue_rewrite".to_string(),
            source_als_hash: source_hash.clone(),
            resolution_policy_version: "0.2.0".to_string(),
            rewrite_ruleset_version: "live11_3_current_paths_v0.2-lab".to_string(),
            required_asset_count: 2,
            directory_operation_count: directories.len(),
            copy_operation_count: 2,
            rewrite_operation_count: 1,
            system_dependency_count: 1,
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
        directory_operations: directories.clone(),
        copy_operations: copies.clone(),
        rewrite_operations: rewrites.clone(),
        system_dependencies: vec![SystemDependencyRequirement {
            required_asset_id: "system_asset0".to_string(),
            source_category: "ableton_core_library".to_string(),
            filename: Some("Core Library Kick.wav".to_string()),
            occurrence_count: 4,
            als_ref_ids: vec![10, 11, 12, 13],
            observed_source_paths: vec![system_audio],
            package_action: "leave_system_managed".to_string(),
            portability_status: "portable_risk".to_string(),
            reason: "ableton_core_library_dependency".to_string(),
        }],
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
            planned_directory_count: directories.len(),
            completed_directory_count: directories.len(),
            planned_copy_count: 2,
            completed_copy_count: 2,
            warning_count: 0,
            error_count: 0,
        },
        staging_root: staging_root.clone(),
        staged_als_relative_path: PathBuf::from("Set.als"),
        directory_records: directories
            .iter()
            .map(|operation| DirectoryExecutionRecord {
                operation_id: operation.operation_id.clone(),
                target_relative_path: operation.target_relative_path.clone(),
                purpose: operation.purpose.clone(),
                operation_status: "created".to_string(),
            })
            .collect(),
        copy_records: copies
            .iter()
            .map(|operation| CopyExecutionRecord {
                operation_id: operation.operation_id.clone(),
                operation_kind: operation.operation_kind.clone(),
                source_path: operation.source_path.clone(),
                target_relative_path: operation.target_relative_path.clone(),
                expected_sha256: operation.expected_source_sha256.clone(),
                observed_sha256: operation.expected_source_sha256.clone(),
                expected_size: operation.expected_source_size,
                observed_size: Some(operation.expected_source_size),
                verification_method: operation.verification_policy.clone(),
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
            rewrite_ruleset_version: "live11_3_current_paths_v0.2-lab".to_string(),
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
            planned_directory_count: directories.len(),
            verified_directory_count: directories.len(),
            planned_file_count: 2,
            verified_file_count: 2,
            planned_rewrite_count: 1,
            verified_rewrite_count: 1,
            warning_count: 0,
            error_count: 0,
        },
        staging_root: staging_root.clone(),
        final_target_root: final_root.clone(),
        directory_records: directories
            .iter()
            .map(|operation| DirectoryValidationRecord {
                operation_id: operation.operation_id.clone(),
                target_relative_path: operation.target_relative_path.clone(),
                purpose: operation.purpose.clone(),
                directory_status: "verified".to_string(),
            })
            .collect(),
        file_records: vec![
            FileValidationRecord {
                operation_id: "copy_als".to_string(),
                target_relative_path: PathBuf::from("Set.als"),
                expected_sha256: Some(rewritten_hash.clone()),
                observed_sha256: Some(rewritten_hash),
                expected_size: None,
                observed_size: Some(13),
                verification_method: rescue_packaging::VERIFY_SHA256_AND_SIZE.to_string(),
                file_status: "verified".to_string(),
            },
            FileValidationRecord {
                operation_id: "copy_audio".to_string(),
                target_relative_path: PathBuf::from("Samples/Imported/shared.wav"),
                expected_sha256: Some(audio_hash.clone()),
                observed_sha256: Some(audio_hash),
                expected_size: Some(5),
                observed_size: Some(5),
                verification_method: rescue_packaging::VERIFY_SHA256_AND_SIZE.to_string(),
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
    let manifest: serde_json::Value = serde_json::from_slice(
        &fs::read(
            fixture
                .staging_root
                .join("Rescue Manifest/package-manifest.json"),
        )
        .expect("portable manifest"),
    )
    .expect("manifest JSON");
    assert_eq!(
        manifest["system_dependencies"].as_array().map(Vec::len),
        Some(1)
    );
    assert_eq!(
        manifest["system_dependencies"][0]["package_action"],
        "leave_system_managed"
    );
    assert_eq!(
        manifest["system_dependencies"][0]["portability_status"],
        "portable_risk"
    );
}

#[test]
fn metadata_only_audio_manifest_marks_content_identity_not_computed() {
    let mut fixture = fixture();
    let operation = fixture
        .plan
        .copy_operations
        .iter_mut()
        .find(|operation| operation.operation_kind == "copy_audio")
        .expect("audio operation");
    operation.expected_source_sha256 = None;
    operation.content_id = None;
    operation.verification_policy = rescue_packaging::VERIFY_STABLE_SOURCE_AND_SIZE.to_string();
    let staging_record = fixture
        .staging
        .copy_records
        .iter_mut()
        .find(|record| record.operation_kind == "copy_audio")
        .expect("audio staging record");
    staging_record.expected_sha256 = None;
    staging_record.observed_sha256 = None;
    staging_record.verification_method =
        rescue_packaging::VERIFY_STABLE_SOURCE_AND_SIZE.to_string();
    let validation_record = fixture
        .validation
        .file_records
        .iter_mut()
        .find(|record| record.operation_id == "copy_audio")
        .expect("audio validation record");
    validation_record.expected_sha256 = None;
    validation_record.observed_sha256 = None;
    validation_record.verification_method =
        rescue_packaging::VERIFY_STABLE_SOURCE_AND_SIZE.to_string();

    let result = write(&fixture);
    let manifest: serde_json::Value = serde_json::from_slice(
        &fs::read(
            fixture
                .staging_root
                .join("Rescue Manifest/package-manifest.json"),
        )
        .expect("portable manifest"),
    )
    .expect("manifest JSON");
    let audio = manifest["files"]
        .as_array()
        .expect("files")
        .iter()
        .find(|file| file["role"] == "copy_audio")
        .expect("audio manifest record");

    assert_eq!(result.write_status, "manifests_written");
    assert!(audio["sha256"].is_null());
    assert_eq!(audio["content_identity_status"], "not_computed");
    assert_eq!(
        audio["verification_method"],
        rescue_packaging::VERIFY_STABLE_SOURCE_AND_SIZE
    );
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
    assert!(text.contains("Ableton Project Info"));
    assert!(text.contains("ableton_project_marker"));
    assert!(text.contains("Samples/Imported/shared.wav"));
}

#[test]
fn portable_manifest_records_system_dependency_without_local_path() {
    let fixture = fixture();
    let result = write(&fixture);
    let text = fs::read_to_string(
        fixture
            .staging_root
            .join("Rescue Manifest/package-manifest.json"),
    )
    .expect("portable manifest");
    let manifest: serde_json::Value = serde_json::from_str(&text).expect("manifest JSON");

    assert_eq!(result.write_status, "manifests_written");
    assert_eq!(
        manifest["system_dependencies"].as_array().map(Vec::len),
        Some(1)
    );
    assert_eq!(
        manifest["system_dependencies"][0]["required_environment"],
        "compatible_ableton_core_library"
    );
    assert!(!text.contains(fixture._temp.path().to_string_lossy().as_ref()));
}

#[test]
fn private_ledger_retains_full_audit_paths() {
    let fixture = fixture();
    let result = write(&fixture);
    assert_eq!(result.write_status, "manifests_written");
    let text = fs::read_to_string(&fixture.private_ledger).expect("private ledger");
    let ledger: PrivateLedger = serde_json::from_str(&text).expect("private ledger JSON");

    assert_eq!(ledger.plan.source_als.source_als_path, fixture.source_als);
    assert_eq!(ledger.staging.staging_root, fixture.staging_root);
    assert_eq!(
        ledger.plan.rewrite_operations[0].old_path.as_deref(),
        Some("/private/source/shared.wav")
    );
    assert_eq!(ledger.plan.system_dependencies.len(), 1);
    assert_eq!(
        ledger.plan.system_dependencies[0]
            .observed_source_paths
            .len(),
        1
    );
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

#[cfg(windows)]
#[test]
fn windows_manifest_write_is_noclobber_and_idempotent_on_ntfs() {
    let fixture = fixture();

    let first = write(&fixture);
    let second = write(&fixture);

    assert_eq!(first.write_status, "manifests_written");
    assert_eq!(second.write_status, "manifests_written");
    assert!(second
        .write_records
        .iter()
        .all(|record| record.write_status == "already_present_verified"));
    assert!(!has_temp_file(&fixture.staging_root));
    assert!(!has_temp_file(
        fixture.private_ledger.parent().expect("ledger parent")
    ));
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
