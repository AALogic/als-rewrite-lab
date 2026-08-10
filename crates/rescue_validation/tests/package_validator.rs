use flate2::read::GzDecoder;
use flate2::write::GzEncoder;
use flate2::Compression;
use rescue_execution::{execute_staging, StagingExecutionRequest, StagingExecutionResult};
use rescue_packaging::{
    CopyOperation, CreateDirectoryOperation, PackagePlan, PackagePlanMetadata, PlannedSourceAls,
    RewriteOperation, SystemDependencyRequirement,
};
use rescue_rewriter::ALSRewriteResult;
use rescue_validation::{validate_staged_package, PackageValidationRequest};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::fs;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use tempfile::TempDir;

const OLD_PATH: &str = "/external/shared.wav";
const OLD_RELATIVE: &str = "../shared.wav";

struct Run {
    _temp: TempDir,
    source_als: PathBuf,
    source_audio: PathBuf,
    staging_root: PathBuf,
    final_root: PathBuf,
    plan: PackagePlan,
    staging: StagingExecutionResult,
    rewrite: ALSRewriteResult,
}

fn source_xml() -> String {
    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<Ableton MajorVersion="5" MinorVersion="11.0_11300" Creator="Ableton Live 11.3.43">
  <LiveSet Name="fixture">
    <SampleRef><FileRef><Path Value="{OLD_PATH}"/><RelativePath Value="{OLD_RELATIVE}"/><RelativePathType Value="1"/><OriginalFileSize Value="5"/></FileRef></SampleRef>
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

fn gunzip(path: &Path) -> String {
    let bytes = fs::read(path).expect("ALS bytes");
    let mut decoder = GzDecoder::new(bytes.as_slice());
    let mut xml = String::new();
    decoder.read_to_string(&mut xml).expect("gunzip");
    xml
}

fn digest_bytes(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn digest_file(path: &Path) -> String {
    digest_bytes(&fs::read(path).expect("fixture bytes"))
}

fn copy_operation(id: &str, kind: &str, source: &Path, target: &str) -> CopyOperation {
    CopyOperation {
        operation_id: id.to_string(),
        operation_kind: kind.to_string(),
        source_path: source.to_path_buf(),
        target_relative_path: PathBuf::from(target),
        expected_source_sha256: Some(digest_file(source)),
        expected_source_size: fs::metadata(source).expect("metadata").len(),
        content_id: Some(format!("sha256:{}", digest_file(source))),
        source_binding_id: format!("test:{id}"),
        verification_policy: rescue_packaging::VERIFY_SHA256_AND_SIZE.to_string(),
        collision_policy: "fail_if_exists".to_string(),
        preconditions: Vec::new(),
    }
}

#[cfg(unix)]
fn prepare_rewrite(
    staging_root: &Path,
    plan: &PackagePlan,
    staging: &StagingExecutionResult,
) -> ALSRewriteResult {
    let rewrite = rescue_rewriter::rewrite_staged_als(
        &rescue_rewriter::ALSRewriteRequest {
            rewrite_id: "rewrite0".to_string(),
            staging_root: staging_root.to_path_buf(),
        },
        plan,
        staging,
    );
    assert_eq!(rewrite.rewrite_status, "rewrite_complete");
    rewrite
}

#[cfg(not(unix))]
fn prepare_rewrite(
    staging_root: &Path,
    plan: &PackagePlan,
    staging: &StagingExecutionResult,
) -> ALSRewriteResult {
    let operation = &plan.rewrite_operations[0];
    let staged_als = staging_root.join(&staging.staged_als_relative_path);
    let original_hash = digest_file(&staged_als);
    let rewritten_xml = gunzip(&staged_als)
        .replacen(
            &format!(r#"<Path Value="{OLD_PATH}"/>"#),
            &format!(r#"<Path Value="{}"/>"#, operation.new_path),
            1,
        )
        .replacen(
            &format!(r#"<RelativePath Value="{OLD_RELATIVE}"/>"#),
            &format!(r#"<RelativePath Value="{}"/>"#, operation.new_relative_path),
            1,
        )
        .replacen(
            r#"<RelativePathType Value="1"/>"#,
            r#"<RelativePathType Value="3"/>"#,
            1,
        );
    let rewritten_bytes = gzip(&rewritten_xml);
    fs::write(&staged_als, &rewritten_bytes).expect("synthetic rewritten ALS");
    ALSRewriteResult {
        metadata: rescue_rewriter::ALSRewriteMetadata {
            rewriter_version: rescue_rewriter::ALS_REWRITER_VERSION.to_string(),
            rewrite_schema_version: rescue_rewriter::ALS_REWRITE_SCHEMA_VERSION.to_string(),
            rewrite_id: "rewrite0".to_string(),
            execution_id: staging.metadata.execution_id.clone(),
            plan_id: plan.metadata.plan_id.clone(),
            source_als_hash: plan.metadata.source_als_hash.clone(),
            rewrite_ruleset_version: plan.metadata.rewrite_ruleset_version.clone(),
            planned_operation_count: 1,
            completed_operation_count: 1,
            warning_count: 0,
            error_count: 0,
        },
        staged_als_relative_path: staging.staged_als_relative_path.clone(),
        original_staged_als_hash: Some(original_hash),
        rewritten_staged_als_hash: Some(digest_bytes(&rewritten_bytes)),
        operation_records: vec![rescue_rewriter::RewriteExecutionRecord {
            operation_id: operation.operation_id.clone(),
            als_ref_id: operation.als_ref_id,
            xml_locator: operation.xml_locator.clone(),
            changed_fields: operation.fields_to_change.clone(),
            operation_status: "rewritten_and_verified".to_string(),
        }],
        rewrite_status: "rewrite_complete".to_string(),
        warnings: Vec::new(),
        errors: Vec::new(),
    }
}

fn prepare_run() -> Run {
    let temp = tempfile::tempdir().expect("tempdir");
    let source_root = temp.path().join("source");
    let staging_root = temp.path().join("staging");
    let final_root = temp.path().join("final").join("Project");
    fs::create_dir(&source_root).expect("source root");
    let source_als = source_root.join("Set.als");
    let source_audio = source_root.join("shared.wav");
    fs::write(&source_als, gzip(&source_xml())).expect("source ALS");
    fs::write(&source_audio, b"audio").expect("source audio");
    let als_hash = digest_file(&source_als);
    let target_relative = "Samples/Imported/shared.wav";
    let copies = vec![
        copy_operation("copy_als_000000", "copy_als", &source_als, "Set.als"),
        copy_operation(
            "copy_audio_000000",
            "copy_audio",
            &source_audio,
            target_relative,
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
    let rewrites = vec![RewriteOperation {
        operation_id: "rewrite_active_000000".to_string(),
        required_asset_id: "asset0".to_string(),
        dependency_id: "dep0".to_string(),
        als_ref_id: 0,
        xml_locator: "SampleRef[0]/FileRef".to_string(),
        source_als_hash: als_hash.clone(),
        old_path: Some(OLD_PATH.to_string()),
        old_relative_path: Some(OLD_RELATIVE.to_string()),
        old_relative_path_type: Some("1".to_string()),
        new_path: final_root
            .join(target_relative)
            .to_string_lossy()
            .to_string(),
        new_relative_path: target_relative.to_string(),
        new_relative_path_type: "3".to_string(),
        fields_to_change: vec![
            "Path".to_string(),
            "RelativePath".to_string(),
            "RelativePathType".to_string(),
        ],
        rule_id: "live11_3_current_paths_v0.2-lab".to_string(),
        support_status: "experimental_lab_only".to_string(),
    }];
    let plan = PackagePlan {
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
            copy_operation_count: copies.len(),
            rewrite_operation_count: rewrites.len(),
            system_dependency_count: 0,
            unresolved_count: 0,
            warning_count: 0,
            error_count: 0,
        },
        source_als: PlannedSourceAls {
            source_als_path: source_als.clone(),
            source_file_hash: als_hash,
            source_file_size: fs::metadata(&source_als).expect("metadata").len(),
            target_relative_path: PathBuf::from("Set.als"),
            ableton_document_version: Some("5".to_string()),
            ableton_creator_version: Some("Ableton Live 11.3.43".to_string()),
            ableton_minor_version: Some("11.0_11300".to_string()),
        },
        target_project_root: final_root.clone(),
        directory_operations: directories,
        copy_operations: copies,
        rewrite_operations: rewrites,
        system_dependencies: Vec::new(),
        unresolved_requirements: Vec::new(),
        plan_status: "ready_for_laboratory_execution".to_string(),
        warnings: Vec::new(),
        errors: Vec::new(),
    };
    let staging = execute_staging(
        &StagingExecutionRequest {
            execution_id: "execution0".to_string(),
            staging_root: staging_root.clone(),
        },
        &plan,
    );
    assert_eq!(staging.execution_status, "staging_complete");
    let rewrite = prepare_rewrite(&staging_root, &plan, &staging);
    Run {
        _temp: temp,
        source_als,
        source_audio,
        staging_root,
        final_root,
        plan,
        staging,
        rewrite,
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

fn request(run: &Run) -> PackageValidationRequest {
    PackageValidationRequest {
        validation_id: "validation0".to_string(),
        staging_root: run.staging_root.clone(),
    }
}

fn validate(run: &Run) -> rescue_validation::PackageValidationResult {
    validate_staged_package(&request(run), &run.plan, &run.staging, &run.rewrite)
}

fn rewrite_staged_xml(run: &mut Run, transform: impl FnOnce(String) -> String) {
    let staged_als = run.staging_root.join("Set.als");
    let xml = transform(gunzip(&staged_als));
    let bytes = gzip(&xml);
    fs::write(&staged_als, &bytes).expect("replace staged fixture");
    run.rewrite.rewritten_staged_als_hash = Some(digest_bytes(&bytes));
}

#[test]
fn complete_staging_and_exact_semantic_diff_pass_validation() {
    let run = prepare_run();
    let result = validate(&run);

    assert_eq!(result.validation_status, "validation_passed");
    assert_eq!(result.metadata.verified_directory_count, 3);
    assert_eq!(result.metadata.verified_file_count, 2);
    assert_eq!(result.metadata.verified_rewrite_count, 1);
    assert!(result.errors.is_empty());
}

#[test]
fn metadata_only_audio_validates_without_content_hash() {
    let mut run = prepare_run();
    let operation = run
        .plan
        .copy_operations
        .iter_mut()
        .find(|operation| operation.operation_kind == "copy_audio")
        .expect("audio operation");
    operation.expected_source_sha256 = None;
    operation.content_id = None;
    operation.verification_policy = rescue_packaging::VERIFY_STABLE_SOURCE_AND_SIZE.to_string();
    let staging_record = run
        .staging
        .copy_records
        .iter_mut()
        .find(|record| record.operation_kind == "copy_audio")
        .expect("audio staging record");
    staging_record.expected_sha256 = None;
    staging_record.observed_sha256 = None;
    staging_record.verification_method =
        rescue_packaging::VERIFY_STABLE_SOURCE_AND_SIZE.to_string();

    let result = validate(&run);
    let record = result
        .file_records
        .iter()
        .find(|record| record.operation_id == "copy_audio_000000")
        .expect("audio validation record");

    assert_eq!(result.validation_status, "validation_passed");
    assert_eq!(record.expected_sha256, None);
    assert_eq!(record.observed_sha256, None);
    assert_eq!(
        record.verification_method,
        rescue_packaging::VERIFY_STABLE_SOURCE_AND_SIZE
    );
}

#[test]
fn path_only_semantic_diff_preserves_relative_fields() {
    let mut run = prepare_run();
    let source_xml = gunzip(&run.source_als)
        .replacen(OLD_RELATIVE, "Samples/Imported/shared.wav", 1)
        .replacen(
            "RelativePathType Value=\"1\"",
            "RelativePathType Value=\"3\"",
            1,
        );
    let source_bytes = gzip(&source_xml);
    fs::write(&run.source_als, &source_bytes).expect("project-local source ALS");
    let source_hash = digest_bytes(&source_bytes);

    run.plan.metadata.source_als_hash = source_hash.clone();
    run.plan.source_als.source_file_hash = source_hash.clone();
    run.plan.source_als.source_file_size = source_bytes.len() as u64;
    run.plan.copy_operations[0].expected_source_sha256 = Some(source_hash.clone());
    run.plan.copy_operations[0].expected_source_size = source_bytes.len() as u64;
    run.plan.rewrite_operations[0].source_als_hash = source_hash.clone();
    run.plan.rewrite_operations[0].old_relative_path =
        Some("Samples/Imported/shared.wav".to_string());
    run.plan.rewrite_operations[0].old_relative_path_type = Some("3".to_string());
    run.plan.rewrite_operations[0].fields_to_change = vec!["Path".to_string()];
    run.plan.rewrite_operations[0].support_status = "confirmed_lab".to_string();

    run.staging.metadata.source_als_hash = source_hash.clone();
    run.staging.copy_records[0].expected_sha256 = Some(source_hash.clone());
    run.staging.copy_records[0].observed_sha256 = Some(source_hash.clone());
    run.staging.copy_records[0].expected_size = source_bytes.len() as u64;
    run.staging.copy_records[0].observed_size = Some(source_bytes.len() as u64);

    run.rewrite.metadata.source_als_hash = source_hash.clone();
    run.rewrite.original_staged_als_hash = Some(source_hash);
    run.rewrite.operation_records[0].changed_fields = vec!["Path".to_string()];

    let result = validate(&run);

    assert_eq!(result.validation_status, "validation_passed");
    assert_eq!(
        result.semantic_diff_records[0].verified_fields,
        vec!["Path"]
    );
}

#[test]
fn tampered_audio_fails_hash_validation() {
    let run = prepare_run();
    fs::write(
        run.staging_root.join("Samples/Imported/shared.wav"),
        b"changed",
    )
    .expect("tamper audio");
    let result = validate(&run);

    assert_eq!(result.validation_status, "validation_failed");
    assert!(result
        .errors
        .iter()
        .any(|error| error.error_code == "VALIDATION_FILE_HASH_MISMATCH"));
}

#[test]
fn unexpected_staged_file_is_rejected() {
    let run = prepare_run();
    fs::write(run.staging_root.join("unexpected.txt"), b"unexpected").expect("extra file");
    let result = validate(&run);

    assert_eq!(result.validation_status, "validation_failed");
    assert!(result
        .errors
        .iter()
        .any(|error| error.error_code == "VALIDATION_UNEXPECTED_STAGED_FILE"));
}

#[test]
fn unrelated_xml_change_fails_semantic_diff() {
    let mut run = prepare_run();
    rewrite_staged_xml(&mut run, |xml| {
        xml.replace(r#"<LiveSet Name="fixture">"#, r#"<LiveSet Name="changed">"#)
    });
    let result = validate(&run);

    assert_eq!(result.validation_status, "validation_failed");
    assert!(result
        .errors
        .iter()
        .any(|error| error.error_code == "SEMANTIC_DIFF_UNEXPECTED_CHANGE"));
}

#[test]
fn historical_reference_change_fails_semantic_diff() {
    let mut run = prepare_run();
    rewrite_staged_xml(&mut run, |xml| {
        xml.replacen(
            &format!(r#"<Path Value="{OLD_PATH}"/>"#),
            r#"<Path Value="/historical-changed.wav"/>"#,
            1,
        )
    });
    let result = validate(&run);

    assert_eq!(result.validation_status, "validation_failed");
    assert!(result
        .errors
        .iter()
        .any(|error| error.error_code == "SEMANTIC_DIFF_UNEXPECTED_CHANGE"));
}

#[test]
fn approved_field_value_mismatch_fails_validation() {
    let mut run = prepare_run();
    run.plan.rewrite_operations[0].new_path = run
        .final_root
        .join("Samples/Imported/different.wav")
        .to_string_lossy()
        .to_string();
    let result = validate(&run);

    assert_eq!(result.validation_status, "validation_failed");
    assert!(result
        .errors
        .iter()
        .any(|error| error.error_code == "SEMANTIC_DIFF_VALUE_MISMATCH"));
}

#[test]
fn changed_original_source_is_detected() {
    let run = prepare_run();
    fs::write(
        &run.source_als,
        gzip(&source_xml().replace("fixture", "changed")),
    )
    .expect("change original fixture");
    let result = validate(&run);

    assert_eq!(result.validation_status, "validation_failed");
    assert!(result
        .errors
        .iter()
        .any(|error| error.error_code == "ORIGINAL_FILE_CHANGED"));
}

#[test]
fn existing_final_target_blocks_validation() {
    let run = prepare_run();
    fs::create_dir_all(&run.final_root).expect("final target");
    let result = validate(&run);

    assert_eq!(result.validation_status, "validation_failed");
    assert!(result
        .errors
        .iter()
        .any(|error| error.error_code == "VALIDATION_FINAL_TARGET_EXISTS"));
}

#[test]
fn missing_ableton_project_marker_blocks_validation() {
    let run = prepare_run();
    fs::remove_dir(run.staging_root.join("Ableton Project Info")).expect("remove marker");
    let result = validate(&run);

    assert_eq!(result.validation_status, "validation_failed");
    assert!(result
        .errors
        .iter()
        .any(|error| error.error_code == "VALIDATION_ABLETON_PROJECT_MARKER_MISSING"));
}

#[cfg(unix)]
#[test]
fn symlink_ableton_project_marker_blocks_validation() {
    use std::os::unix::fs::symlink;

    let run = prepare_run();
    let marker = run.staging_root.join("Ableton Project Info");
    fs::remove_dir(&marker).expect("remove marker");
    symlink(run._temp.path(), &marker).expect("marker symlink");
    let result = validate(&run);

    assert_eq!(result.validation_status, "validation_failed");
    assert!(result.errors.iter().any(|error| {
        error.error_code == "VALIDATION_PLANNED_DIRECTORY_INVALID"
            || error.error_code == "VALIDATION_STAGING_SYMLINK"
    }));
}

#[test]
fn mismatched_run_contracts_are_rejected() {
    let mut run = prepare_run();
    run.rewrite.metadata.plan_id = "another-plan".to_string();
    let result = validate(&run);

    assert_eq!(result.validation_status, "validation_failed");
    assert!(result
        .errors
        .iter()
        .any(|error| error.error_code == "VALIDATION_CONTRACT_IDENTITY_MISMATCH"));
}

#[test]
fn system_dependency_cannot_be_copied_or_rewritten() {
    let mut run = prepare_run();
    run.plan.metadata.system_dependency_count = 1;
    run.plan
        .system_dependencies
        .push(SystemDependencyRequirement {
            required_asset_id: "asset0".to_string(),
            source_category: "ableton_core_library".to_string(),
            filename: Some("shared.wav".to_string()),
            occurrence_count: 1,
            als_ref_ids: vec![0],
            observed_source_paths: vec![run.source_audio.clone()],
            package_action: "leave_system_managed".to_string(),
            portability_status: "portable_risk".to_string(),
            reason: "ableton_core_library_dependency".to_string(),
        });

    let result = validate(&run);

    assert_eq!(result.validation_status, "validation_failed");
    assert!(result
        .errors
        .iter()
        .any(|error| { error.error_code == "VALIDATION_SYSTEM_DEPENDENCY_COPY_FORBIDDEN" }));
    assert!(result
        .errors
        .iter()
        .any(|error| { error.error_code == "VALIDATION_SYSTEM_DEPENDENCY_REWRITE_FORBIDDEN" }));
}

#[test]
fn validation_is_read_only() {
    let run = prepare_run();
    let before = tree_hashes(&run.staging_root);
    let first = validate(&run);
    let second = validate(&run);
    let after = tree_hashes(&run.staging_root);

    assert_eq!(first, second);
    assert_eq!(before, after);
    assert_eq!(digest_file(&run.source_audio), digest_bytes(b"audio"));
}

fn tree_hashes(root: &Path) -> BTreeMap<PathBuf, String> {
    let mut result = BTreeMap::new();
    let mut stack = vec![root.to_path_buf()];
    while let Some(directory) = stack.pop() {
        for entry in fs::read_dir(&directory).expect("read tree") {
            let path = entry.expect("entry").path();
            if path.is_dir() {
                stack.push(path);
            } else {
                result.insert(
                    path.strip_prefix(root).expect("relative").to_path_buf(),
                    digest_file(&path),
                );
            }
        }
    }
    result
}
