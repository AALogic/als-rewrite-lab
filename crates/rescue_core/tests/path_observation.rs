use rescue_core::{
    observe_dependency_paths, CandidatePathObservation, DependencyExtractionMetadata,
    DependencyExtractionResult, DependencyRef, IgnoredInputSummary, PathObservationContext,
    PathObservationResult,
};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

static NEXT_DIR_ID: AtomicU64 = AtomicU64::new(0);

struct TempTree {
    root: PathBuf,
}

impl TempTree {
    fn new(name: &str) -> Self {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock should be after Unix epoch")
            .as_nanos();
        let id = NEXT_DIR_ID.fetch_add(1, Ordering::Relaxed);
        let root = std::env::temp_dir().join(format!(
            "rescue_path_observation_{name}_{}_{stamp}_{id}",
            std::process::id()
        ));
        fs::create_dir_all(&root).expect("temporary fixture root should exist");
        Self { root }
    }

    fn file(&self, relative: &str, bytes: &[u8]) -> PathBuf {
        let path = self.root.join(relative);
        fs::create_dir_all(path.parent().expect("fixture path should have parent"))
            .expect("fixture parent should exist");
        fs::write(&path, bytes).expect("fixture file should be writable");
        path
    }

    fn directory(&self, relative: &str) -> PathBuf {
        let path = self.root.join(relative);
        fs::create_dir_all(&path).expect("fixture directory should exist");
        path
    }
}

impl Drop for TempTree {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

fn host_platform() -> String {
    if cfg!(target_os = "windows") {
        "windows".to_string()
    } else if cfg!(target_os = "macos") {
        "macos".to_string()
    } else {
        "posix".to_string()
    }
}

fn context(root: Option<&Path>) -> PathObservationContext {
    PathObservationContext {
        host_platform: host_platform(),
        confirmed_project_root: root.map(Path::to_path_buf),
        project_root_basis: root.map(|_| "controlled_fixture".to_string()),
    }
}

fn dependency(
    id: usize,
    path_type: &str,
    raw_path: Option<String>,
    raw_relative_path: Option<&str>,
    size: Option<&str>,
) -> DependencyRef {
    DependencyRef {
        dependency_id: format!("dep_audio_{id:06}"),
        dependency_kind: "audio_sample".to_string(),
        als_ref_id: id,
        source_kind: "sample_ref_file_ref".to_string(),
        raw_path,
        raw_relative_path: raw_relative_path.map(str::to_string),
        relative_path_type: Some(path_type.to_string()),
        file_type: Some("2".to_string()),
        filename: Some("Kick.wav".to_string()),
        extension: Some("wav".to_string()),
        original_file_size: size.map(str::to_string),
        original_crc: Some("12345".to_string()),
        default_duration: None,
        default_sample_rate: None,
        usage_context: "unknown".to_string(),
        xml_context: "SampleRef/FileRef".to_string(),
        rewrite_support_status: "requires_test".to_string(),
        extraction_status: "extracted".to_string(),
        path_basis: "raw_path_and_raw_relative_path".to_string(),
        evidence_status: "observed".to_string(),
        evidence_notes: Vec::new(),
        warnings: Vec::new(),
    }
}

fn extraction(dependencies: Vec<DependencyRef>) -> DependencyExtractionResult {
    DependencyExtractionResult {
        extraction_metadata: DependencyExtractionMetadata {
            extractor_version: "0.1".to_string(),
            dependency_ref_version: "0.1".to_string(),
            input_als_read_model_version: "0.2".to_string(),
            source_als_path: "/fixture/project.als".to_string(),
            source_project_root: None,
            source_file_hash: "sha256-fixture".to_string(),
            dependency_count: dependencies.len(),
            warning_count: 0,
            error_count: 0,
        },
        dependencies,
        ignored_input_summary: IgnoredInputSummary {
            historical_refs_ignored_count: 0,
            non_audio_dependency_signals_ignored_count: 0,
            input_warnings_seen_count: 0,
            input_errors_seen_count: 0,
            ignored_scope_notes: Vec::new(),
        },
        warnings: Vec::new(),
        errors: Vec::new(),
    }
}

fn observe(dependencies: Vec<DependencyRef>, root: Option<&Path>) -> PathObservationResult {
    observe_dependency_paths(&extraction(dependencies), &context(root))
}

fn candidate<'a>(result: &'a PathObservationResult, basis: &str) -> &'a CandidatePathObservation {
    result.dependency_observations[0]
        .candidates
        .iter()
        .find(|item| item.candidate_basis == basis)
        .expect("expected candidate basis should be present")
}

#[test]
fn confirmed_project_root_produces_relative_candidate() {
    let tree = TempTree::new("confirmed_root");
    tree.file("Samples/Imported/Kick.wav", b"kick");
    let result = observe(
        vec![dependency(
            0,
            "3",
            None,
            Some("Samples/Imported/Kick.wav"),
            Some("4"),
        )],
        Some(&tree.root),
    );

    let item = candidate(&result, "confirmed_project_root_plus_raw_relative_path");
    assert_eq!(item.availability_status, "existing_regular_file");
    assert_eq!(
        result.dependency_observations[0].identity_status,
        "not_evaluated"
    );
}

#[test]
fn relative_path_type_zero_uses_raw_path() {
    let tree = TempTree::new("type_zero");
    tree.file("Samples/Processed/Kick.aif", b"audio");
    let result = observe(
        vec![dependency(
            0,
            "0",
            Some("Samples/Processed/Kick.aif".to_string()),
            Some(""),
            Some("5"),
        )],
        Some(&tree.root),
    );

    let item = candidate(&result, "confirmed_project_root_plus_raw_path");
    assert_eq!(item.availability_status, "existing_regular_file");
    assert_eq!(item.observed_file_size, Some(5));
}

#[test]
fn type_one_and_five_relative_paths_are_not_project_joined() {
    let tree = TempTree::new("non_project_relative_types");
    tree.file("Samples/Imported/Kick.wav", b"wrong");
    let dependencies = vec![
        dependency(0, "1", None, Some("Samples/Imported/Kick.wav"), None),
        dependency(1, "5", None, Some("Samples/Imported/Kick.wav"), None),
    ];
    let result = observe(dependencies, Some(&tree.root));

    assert!(result
        .dependency_observations
        .iter()
        .all(|item| item.candidates.is_empty()));
}

#[test]
fn absent_project_root_produces_no_relative_candidate() {
    let result = observe(
        vec![dependency(
            0,
            "3",
            None,
            Some("Samples/Imported/Kick.wav"),
            None,
        )],
        None,
    );

    assert!(result.dependency_observations[0].candidates.is_empty());
    assert!(result
        .warnings
        .iter()
        .any(|warning| { warning.warning_code == "PATH_PROJECT_ROOT_UNAVAILABLE" }));
}

#[test]
fn both_safe_candidates_are_preserved_without_selection() {
    let tree = TempTree::new("two_candidates");
    let direct = tree.file("external/Kick.wav", b"direct");
    tree.file("Samples/Imported/Kick.wav", b"project");
    let result = observe(
        vec![dependency(
            0,
            "3",
            Some(direct.to_string_lossy().to_string()),
            Some("Samples/Imported/Kick.wav"),
            None,
        )],
        Some(&tree.root),
    );

    assert_eq!(result.dependency_observations[0].candidates.len(), 2);
    let json = serde_json::to_string(&result).expect("result should serialize");
    assert!(!json.contains("selected_path_candidate"));
    assert!(!json.contains("verified_exact_path"));
}

#[test]
fn existing_regular_file_is_observed_without_identity_claim() {
    let tree = TempTree::new("regular_file");
    let direct = tree.file("external/Kick.wav", b"audio");
    let result = observe(
        vec![dependency(
            0,
            "1",
            Some(direct.to_string_lossy().to_string()),
            None,
            Some("5"),
        )],
        Some(&tree.root),
    );

    assert_eq!(
        result.dependency_observations[0].availability_summary,
        "regular_file_observed"
    );
    assert_eq!(
        result.dependency_observations[0].identity_status,
        "not_evaluated"
    );
}

#[test]
fn missing_candidate_is_observed() {
    let tree = TempTree::new("missing");
    let path = tree.root.join("missing.wav");
    let result = observe(
        vec![dependency(
            0,
            "1",
            Some(path.to_string_lossy().to_string()),
            None,
            None,
        )],
        Some(&tree.root),
    );

    assert_eq!(
        result.dependency_observations[0].candidates[0].availability_status,
        "missing"
    );
}

#[test]
fn directory_candidate_is_observed() {
    let tree = TempTree::new("directory");
    let path = tree.directory("not-a-file.wav");
    let result = observe(
        vec![dependency(
            0,
            "1",
            Some(path.to_string_lossy().to_string()),
            None,
            None,
        )],
        Some(&tree.root),
    );

    assert_eq!(
        result.dependency_observations[0].candidates[0].entry_kind,
        "directory"
    );
}

#[cfg(unix)]
#[test]
fn symlink_is_reported_and_not_followed() {
    use std::os::unix::fs::symlink;

    let tree = TempTree::new("symlink");
    let target = tree.file("target.wav", b"target");
    let link = tree.root.join("link.wav");
    symlink(&target, &link).expect("fixture symlink should be created");
    let result = observe(
        vec![dependency(
            0,
            "1",
            Some(link.to_string_lossy().to_string()),
            None,
            None,
        )],
        Some(&tree.root),
    );

    let item = &result.dependency_observations[0].candidates[0];
    assert_eq!(item.availability_status, "existing_symlink");
    assert_eq!(item.entry_kind, "symlink");
    assert_eq!(item.observed_file_size, None);
}

#[test]
fn parent_escape_candidate_is_rejected() {
    let tree = TempTree::new("parent_escape");
    let result = observe(
        vec![dependency(0, "3", None, Some("../../outside.wav"), None)],
        Some(&tree.root),
    );

    let item = &result.dependency_observations[0].candidates[0];
    assert_eq!(item.safety_status, "rejected_parent_escape");
    assert_eq!(item.availability_status, "not_checked");
}

#[test]
fn size_mismatch_does_not_select_or_resolve_asset() {
    let tree = TempTree::new("size_mismatch");
    let direct = tree.file("Kick.wav", b"12345");
    let result = observe(
        vec![dependency(
            0,
            "1",
            Some(direct.to_string_lossy().to_string()),
            None,
            Some("999"),
        )],
        Some(&tree.root),
    );

    let item = &result.dependency_observations[0].candidates[0];
    assert_eq!(item.size_evidence_status, "differs_from_expected_size");
    assert_eq!(
        result.dependency_observations[0].identity_status,
        "not_evaluated"
    );
}

#[test]
fn foreign_platform_path_is_preserved_but_not_checked() {
    let raw = if cfg!(target_os = "windows") {
        "/Volumes/Studio/Samples/Kick.wav"
    } else {
        r"C:\\Studio\\Samples\\Kick.wav"
    };
    let result = observe(
        vec![dependency(0, "1", Some(raw.to_string()), None, None)],
        None,
    );

    let item = &result.dependency_observations[0].candidates[0];
    assert_eq!(item.candidate_path, raw);
    assert_eq!(item.platform_status, "foreign_platform_path");
    assert_eq!(item.availability_status, "not_checked");
}

#[test]
fn duplicate_reference_occurrences_are_not_deduplicated() {
    let tree = TempTree::new("duplicates");
    let direct = tree.file("Kick.wav", b"audio");
    let raw = direct.to_string_lossy().to_string();
    let result = observe(
        vec![
            dependency(0, "1", Some(raw.clone()), None, None),
            dependency(1, "1", Some(raw), None, None),
        ],
        None,
    );

    assert_eq!(result.dependency_observations.len(), 2);
    assert_ne!(
        result.dependency_observations[0].dependency_id,
        result.dependency_observations[1].dependency_id
    );
}

#[test]
fn raw_paths_are_preserved_exactly() {
    let raw = "Samples/Impo\u{301}rted/KICK one.wav";
    let result = observe(
        vec![dependency(0, "0", Some(raw.to_string()), None, None)],
        None,
    );

    assert_eq!(
        result.dependency_observations[0].raw_path.as_deref(),
        Some(raw)
    );
    assert_eq!(
        result.dependency_observations[0]
            .parsed_raw_path
            .as_ref()
            .map(|path| path.raw.as_str()),
        Some(raw)
    );
}

#[test]
fn path_observation_is_metadata_only() {
    let tree = TempTree::new("metadata_only");
    let direct = tree.file("Kick.wav", b"unchanged-audio-bytes");
    let before = fs::read(&direct).expect("fixture should be readable");
    let result = observe(
        vec![dependency(
            0,
            "1",
            Some(direct.to_string_lossy().to_string()),
            None,
            None,
        )],
        Some(&tree.root),
    );
    let after = fs::read(&direct).expect("fixture should still be readable");

    assert!(result.errors.is_empty());
    assert_eq!(before, after);
}

#[test]
fn fake_dependency_assessment_consumes_all_observations() {
    fn fake_consumer(result: &PathObservationResult) -> Vec<(&str, usize)> {
        result
            .dependency_observations
            .iter()
            .map(|item| (item.identity_status.as_str(), item.candidates.len()))
            .collect()
    }

    let tree = TempTree::new("downstream");
    let direct = tree.file("Kick.wav", b"audio");
    let result = observe(
        vec![dependency(
            0,
            "1",
            Some(direct.to_string_lossy().to_string()),
            None,
            None,
        )],
        None,
    );
    let consumed = fake_consumer(&result);

    assert_eq!(consumed, vec![("not_evaluated", 1)]);
}

#[test]
fn unsupported_input_contract_returns_fatal_error() {
    let mut input = extraction(Vec::new());
    input.extraction_metadata.dependency_ref_version = "999".to_string();
    let result = observe_dependency_paths(&input, &context(None));

    assert!(result.dependency_observations.is_empty());
    assert_eq!(
        result.errors[0].error_code,
        "PATH_OBSERVATION_UNSUPPORTED_INPUT_MODEL"
    );
}

#[test]
fn untrusted_extraction_returns_fatal_error() {
    let mut input = extraction(Vec::new());
    input.errors.push(rescue_core::DependencyExtractionError {
        error_code: "DEPENDENCY_NO_TRUSTED_ALS_MODEL".to_string(),
        message: "fixture error".to_string(),
        input_model_version: "0.2".to_string(),
    });
    let result = observe_dependency_paths(&input, &context(None));

    assert!(result.dependency_observations.is_empty());
    assert_eq!(
        result.errors[0].error_code,
        "PATH_OBSERVATION_UNTRUSTED_INPUT"
    );
}

#[test]
fn inconsistent_context_returns_fatal_error() {
    let invalid = PathObservationContext {
        host_platform: host_platform(),
        confirmed_project_root: Some(PathBuf::from("/fixture")),
        project_root_basis: None,
    };
    let result = observe_dependency_paths(&extraction(Vec::new()), &invalid);

    assert!(result.dependency_observations.is_empty());
    assert_eq!(
        result.errors[0].error_code,
        "PATH_OBSERVATION_INVALID_CONTEXT"
    );
}
