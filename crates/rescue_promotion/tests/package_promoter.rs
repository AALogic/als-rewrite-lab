use rescue_execution::{CopyExecutionRecord, StagingExecutionMetadata, StagingExecutionResult};
use rescue_manifest::{write_package_evidence, ManifestWriteRequest, ManifestWriteResult};
use rescue_packaging::{
    CopyOperation, PackagePlan, PackagePlanMetadata, PlannedSourceAls, RewriteOperation,
};
use rescue_promotion::{promote_validated_package, PackagePromotionRequest};
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
    source_als: PathBuf,
    source_audio: PathBuf,
    staging_root: PathBuf,
    final_root: PathBuf,
    private_ledger: PathBuf,
    plan: PackagePlan,
    staging: StagingExecutionResult,
    rewrite: ALSRewriteResult,
    validation: PackageValidationResult,
    manifests: ManifestWriteResult,
}

fn digest(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn fixture() -> Fixture {
    let temp = tempfile::tempdir().expect("tempdir");
    let source_root = temp.path().join("source");
    let staging_root = temp.path().join("staging");
    let final_parent = temp.path().join("final");
    let final_root = final_parent.join("Project");
    let evidence = temp.path().join("evidence");
    fs::create_dir(&source_root).expect("source root");
    fs::create_dir(&staging_root).expect("staging");
    fs::create_dir(&final_parent).expect("final parent");
    fs::create_dir(&evidence).expect("evidence");
    fs::create_dir_all(staging_root.join("Samples/Imported")).expect("samples");
    let source_als = source_root.join("Set.als");
    let source_audio = source_root.join("shared.wav");
    fs::write(&source_als, b"source ALS").expect("source ALS");
    fs::write(&source_audio, b"audio").expect("source audio");
    fs::write(staging_root.join("Set.als"), b"rewritten ALS").expect("staged ALS");
    fs::write(staging_root.join("Samples/Imported/shared.wav"), b"audio").expect("staged audio");
    let source_hash = digest(b"source ALS");
    let rewritten_hash = digest(b"rewritten ALS");
    let audio_hash = digest(b"audio");
    let copies = copy_operations(&source_als, &source_audio, &source_hash, &audio_hash);
    let rewrites = rewrite_operations(&final_root, &source_hash);
    let plan = package_plan(
        &source_als,
        &final_root,
        &source_hash,
        copies.clone(),
        rewrites.clone(),
    );
    let staging = staging_result(&staging_root, &source_hash, &copies);
    let rewrite = rewrite_result(&source_hash, &rewritten_hash);
    let validation = validation_result(
        &staging_root,
        &final_root,
        &source_hash,
        &rewritten_hash,
        &audio_hash,
    );
    let private_ledger = evidence.join("private-ledger.json");
    let manifests = write_package_evidence(
        &ManifestWriteRequest {
            manifest_id: "manifest0".to_string(),
            private_ledger_path: private_ledger.clone(),
            package_manifest_relative_path: PathBuf::from("Rescue Manifest/package-manifest.json"),
        },
        &plan,
        &staging,
        &rewrite,
        &validation,
    );
    assert_eq!(manifests.write_status, "manifests_written");
    Fixture {
        _temp: temp,
        source_als,
        source_audio,
        staging_root,
        final_root,
        private_ledger,
        plan,
        staging,
        rewrite,
        validation,
        manifests,
    }
}

fn copy_operations(
    source_als: &Path,
    source_audio: &Path,
    source_hash: &str,
    audio_hash: &str,
) -> Vec<CopyOperation> {
    vec![
        CopyOperation {
            operation_id: "copy_als".to_string(),
            operation_kind: "copy_als".to_string(),
            source_path: source_als.to_path_buf(),
            target_relative_path: PathBuf::from("Set.als"),
            expected_source_sha256: Some(source_hash.to_string()),
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
            source_path: source_audio.to_path_buf(),
            target_relative_path: PathBuf::from("Samples/Imported/shared.wav"),
            expected_source_sha256: Some(audio_hash.to_string()),
            expected_source_size: 5,
            content_id: Some(format!("sha256:{audio_hash}")),
            source_binding_id: "occ:audio".to_string(),
            verification_policy: rescue_packaging::VERIFY_SHA256_AND_SIZE.to_string(),
            collision_policy: "fail_if_exists".to_string(),
            preconditions: Vec::new(),
        },
    ]
}

fn rewrite_operations(final_root: &Path, source_hash: &str) -> Vec<RewriteOperation> {
    vec![RewriteOperation {
        operation_id: "rewrite0".to_string(),
        required_asset_id: "asset0".to_string(),
        dependency_id: "dep0".to_string(),
        als_ref_id: 0,
        xml_locator: "SampleRef[0]/FileRef".to_string(),
        source_als_hash: source_hash.to_string(),
        old_path: Some("/source/shared.wav".to_string()),
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
    }]
}

fn package_plan(
    source_als: &Path,
    final_root: &Path,
    source_hash: &str,
    copies: Vec<CopyOperation>,
    rewrites: Vec<RewriteOperation>,
) -> PackagePlan {
    PackagePlan {
        metadata: PackagePlanMetadata {
            planner_version: "0.1.0".to_string(),
            plan_schema_version: "0.1".to_string(),
            plan_id: "plan0".to_string(),
            planning_mode: "laboratory_rescue_rewrite".to_string(),
            source_als_hash: source_hash.to_string(),
            resolution_policy_version: "0.2.0".to_string(),
            rewrite_ruleset_version: "live11_3_current_paths_v0.2-lab".to_string(),
            required_asset_count: 1,
            directory_operation_count: 0,
            copy_operation_count: copies.len(),
            rewrite_operation_count: rewrites.len(),
            system_dependency_count: 0,
            unresolved_count: 0,
            warning_count: 0,
            error_count: 0,
        },
        source_als: PlannedSourceAls {
            source_als_path: source_als.to_path_buf(),
            source_file_hash: source_hash.to_string(),
            source_file_size: 10,
            target_relative_path: PathBuf::from("Set.als"),
            ableton_document_version: Some("5".to_string()),
            ableton_creator_version: Some("Ableton Live 11.3.43".to_string()),
            ableton_minor_version: Some("11.0_11300".to_string()),
        },
        target_project_root: final_root.to_path_buf(),
        directory_operations: Vec::new(),
        copy_operations: copies,
        rewrite_operations: rewrites,
        system_dependencies: Vec::new(),
        unresolved_requirements: Vec::new(),
        plan_status: "ready_for_laboratory_execution".to_string(),
        warnings: Vec::new(),
        errors: Vec::new(),
    }
}

fn staging_result(
    staging_root: &Path,
    source_hash: &str,
    copies: &[CopyOperation],
) -> StagingExecutionResult {
    StagingExecutionResult {
        metadata: StagingExecutionMetadata {
            executor_version: "0.1.0".to_string(),
            execution_schema_version: "0.1".to_string(),
            execution_id: "execution0".to_string(),
            plan_id: "plan0".to_string(),
            source_als_hash: source_hash.to_string(),
            planned_directory_count: 0,
            completed_directory_count: 0,
            planned_copy_count: copies.len(),
            completed_copy_count: copies.len(),
            warning_count: 0,
            error_count: 0,
        },
        staging_root: staging_root.to_path_buf(),
        staged_als_relative_path: PathBuf::from("Set.als"),
        directory_records: Vec::new(),
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
    }
}

fn rewrite_result(source_hash: &str, rewritten_hash: &str) -> ALSRewriteResult {
    ALSRewriteResult {
        metadata: ALSRewriteMetadata {
            rewriter_version: "0.1.0".to_string(),
            rewrite_schema_version: "0.1".to_string(),
            rewrite_id: "rewrite0".to_string(),
            execution_id: "execution0".to_string(),
            plan_id: "plan0".to_string(),
            source_als_hash: source_hash.to_string(),
            rewrite_ruleset_version: "live11_3_current_paths_v0.2-lab".to_string(),
            planned_operation_count: 1,
            completed_operation_count: 1,
            warning_count: 0,
            error_count: 0,
        },
        staged_als_relative_path: PathBuf::from("Set.als"),
        original_staged_als_hash: Some(source_hash.to_string()),
        rewritten_staged_als_hash: Some(rewritten_hash.to_string()),
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
    }
}

fn validation_result(
    staging_root: &Path,
    final_root: &Path,
    source_hash: &str,
    rewritten_hash: &str,
    audio_hash: &str,
) -> PackageValidationResult {
    PackageValidationResult {
        metadata: PackageValidationMetadata {
            validator_version: "0.1.0".to_string(),
            validation_schema_version: "0.1".to_string(),
            validation_id: "validation0".to_string(),
            plan_id: "plan0".to_string(),
            execution_id: "execution0".to_string(),
            rewrite_id: "rewrite0".to_string(),
            source_als_hash: source_hash.to_string(),
            planned_directory_count: 0,
            verified_directory_count: 0,
            planned_file_count: 2,
            verified_file_count: 2,
            planned_rewrite_count: 1,
            verified_rewrite_count: 1,
            warning_count: 0,
            error_count: 0,
        },
        staging_root: staging_root.to_path_buf(),
        final_target_root: final_root.to_path_buf(),
        directory_records: Vec::new(),
        file_records: vec![
            FileValidationRecord {
                operation_id: "copy_als".to_string(),
                target_relative_path: PathBuf::from("Set.als"),
                expected_sha256: Some(rewritten_hash.to_string()),
                observed_sha256: Some(rewritten_hash.to_string()),
                expected_size: None,
                observed_size: Some(13),
                verification_method: rescue_packaging::VERIFY_SHA256_AND_SIZE.to_string(),
                file_status: "verified".to_string(),
            },
            FileValidationRecord {
                operation_id: "copy_audio".to_string(),
                target_relative_path: PathBuf::from("Samples/Imported/shared.wav"),
                expected_sha256: Some(audio_hash.to_string()),
                observed_sha256: Some(audio_hash.to_string()),
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
    }
}

fn promote(fixture: &Fixture) -> rescue_promotion::PackagePromotionResult {
    promote_validated_package(
        &PackagePromotionRequest {
            promotion_id: "promotion0".to_string(),
        },
        &fixture.plan,
        &fixture.staging,
        &fixture.rewrite,
        &fixture.validation,
        &fixture.manifests,
    )
}

#[test]
fn validated_staging_is_promoted_without_touching_sources() {
    let fixture = fixture();
    let source_als_before = fs::read(&fixture.source_als).expect("source ALS");
    let source_audio_before = fs::read(&fixture.source_audio).expect("source audio");
    let result = promote(&fixture);

    assert_eq!(result.promotion_status, "promoted_ready_for_manual_check");
    assert!(!fixture.staging_root.exists());
    assert!(fixture.final_root.join("Set.als").exists());
    assert!(fixture
        .final_root
        .join("Rescue Manifest/package-manifest.json")
        .exists());
    assert_eq!(
        fs::read(&fixture.source_als).expect("source ALS"),
        source_als_before
    );
    assert_eq!(
        fs::read(&fixture.source_audio).expect("source audio"),
        source_audio_before
    );
}

#[test]
fn existing_final_target_is_never_overwritten() {
    let fixture = fixture();
    fs::create_dir(&fixture.final_root).expect("target");
    let marker = fixture.final_root.join("keep.txt");
    fs::write(&marker, b"keep").expect("marker");
    let result = promote(&fixture);

    assert_eq!(result.promotion_status, "promotion_rejected");
    assert!(fixture.staging_root.exists());
    assert_eq!(fs::read(marker).expect("marker"), b"keep");
}

#[test]
fn tampered_staged_file_blocks_promotion() {
    let fixture = fixture();
    fs::write(
        fixture.staging_root.join("Samples/Imported/shared.wav"),
        b"tampered",
    )
    .expect("tamper");
    let result = promote(&fixture);

    assert_eq!(result.promotion_status, "promotion_rejected");
    assert!(result
        .errors
        .iter()
        .any(|error| error.error_code == "PROMOTION_FILE_MISMATCH"));
    assert!(fixture.staging_root.exists());
    assert!(!fixture.final_root.exists());
}

#[test]
fn unexpected_file_blocks_promotion() {
    let fixture = fixture();
    fs::write(fixture.staging_root.join("unexpected.txt"), b"x").expect("extra");
    let result = promote(&fixture);

    assert_eq!(result.promotion_status, "promotion_rejected");
    assert!(result
        .errors
        .iter()
        .any(|error| error.error_code == "PROMOTION_UNEXPECTED_FILE"));
}

#[test]
fn missing_private_ledger_blocks_promotion() {
    let fixture = fixture();
    fs::remove_file(&fixture.private_ledger).expect("remove test ledger");
    let result = promote(&fixture);

    assert_eq!(result.promotion_status, "promotion_rejected");
    assert!(result
        .errors
        .iter()
        .any(|error| error.error_code == "PROMOTION_FILE_UNAVAILABLE"));
}

#[test]
fn failed_validation_state_blocks_promotion() {
    let mut fixture = fixture();
    fixture.validation.validation_status = "validation_failed".to_string();
    let result = promote(&fixture);

    assert_eq!(result.promotion_status, "promotion_rejected");
    assert!(result
        .errors
        .iter()
        .any(|error| error.error_code == "PROMOTION_PIPELINE_NOT_COMPLETE"));
}

#[test]
fn repeated_promotion_verifies_existing_package() {
    let fixture = fixture();
    let first = promote(&fixture);
    let second = promote(&fixture);

    assert_eq!(first.promotion_status, "promoted_ready_for_manual_check");
    assert_eq!(second.promotion_status, "already_promoted_verified");
}

#[cfg(unix)]
#[test]
fn symlink_inside_staging_blocks_promotion() {
    use std::os::unix::fs::symlink;

    let fixture = fixture();
    symlink(
        &fixture.source_audio,
        fixture.staging_root.join("linked.wav"),
    )
    .expect("symlink");
    let result = promote(&fixture);

    assert_eq!(result.promotion_status, "promotion_rejected");
    assert!(result
        .errors
        .iter()
        .any(|error| error.error_code == "PROMOTION_SYMLINK_FORBIDDEN"));
}

#[test]
fn changed_original_source_blocks_promotion() {
    let fixture = fixture();
    fs::write(&fixture.source_als, b"changed source").expect("change source");
    let result = promote(&fixture);

    assert_eq!(result.promotion_status, "promotion_rejected");
    assert!(result
        .errors
        .iter()
        .any(|error| error.error_code == "ORIGINAL_FILE_CHANGED"));
}
