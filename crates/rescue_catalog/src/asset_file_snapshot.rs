use crate::asset_inventory_hash::hash_stable_file;
use crate::asset_inventory_impl::build_records;
use crate::asset_inventory_walk::{recognized_extension, DiscoveredAudioFile};
use crate::{
    AssetInventoryError, AssetInventoryMetadata, AssetInventoryResult, ASSET_INVENTORY_VERSION,
};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Component, Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AssetFileSnapshotRequest {
    pub scan_run_id: String,
    pub paths: Vec<PathBuf>,
}

pub fn snapshot_asset_files(request: &AssetFileSnapshotRequest) -> AssetInventoryResult {
    let paths = normalized_paths(&request.paths);
    let mut errors = validate_paths(request, &paths);
    if !errors.is_empty() {
        return failed_result(request, errors);
    }
    let mut hashed = Vec::new();
    for path in &paths {
        let discovered = discovered_file(path);
        match hash_stable_file(path) {
            Ok(hash) => hashed.push((discovered, hash)),
            Err(failure) => errors.push(error(failure.code, failure.message, path)),
        }
    }
    if !errors.is_empty() {
        return failed_result(request, errors);
    }
    let (file_occurrences, content_records) = build_records(hashed);
    AssetInventoryResult {
        metadata: metadata(
            request,
            "complete",
            paths.len(),
            file_occurrences.len(),
            content_records.len(),
            0,
        ),
        file_occurrences,
        content_records,
        warnings: Vec::new(),
        errors: Vec::new(),
    }
}

fn validate_paths(
    request: &AssetFileSnapshotRequest,
    paths: &[PathBuf],
) -> Vec<AssetInventoryError> {
    let mut errors = Vec::new();
    if request.scan_run_id.trim().is_empty() {
        errors.push(error(
            "INVENTORY_SNAPSHOT_RUN_ID_EMPTY",
            "Snapshot run ID must not be empty",
            Path::new(""),
        ));
    }
    for path in paths {
        if !safe_absolute(path) {
            errors.push(error(
                "INVENTORY_SNAPSHOT_PATH_UNSAFE",
                "Exact snapshot path must be absolute without traversal",
                path,
            ));
            continue;
        }
        match fs::symlink_metadata(path) {
            Ok(metadata) if metadata.file_type().is_symlink() => errors.push(error(
                "INVENTORY_SNAPSHOT_PATH_IS_SYMLINK",
                "Exact snapshot path cannot be a symlink",
                path,
            )),
            Ok(metadata) if !metadata.is_file() => errors.push(error(
                "INVENTORY_SNAPSHOT_PATH_NOT_FILE",
                "Exact snapshot path must be a regular file",
                path,
            )),
            Ok(_) if recognized_extension(path).is_none() => errors.push(error(
                "INVENTORY_SNAPSHOT_NOT_AUDIO",
                "Exact snapshot path is not a recognized audio file",
                path,
            )),
            Ok(_) => {}
            Err(_) => errors.push(error(
                "INVENTORY_SNAPSHOT_PATH_UNAVAILABLE",
                "Exact snapshot path is unavailable",
                path,
            )),
        }
    }
    errors
}

fn discovered_file(path: &Path) -> DiscoveredAudioFile {
    let source_root = path.parent().map(Path::to_path_buf).unwrap_or_default();
    let relative_path = path.file_name().map(PathBuf::from).unwrap_or_default();
    DiscoveredAudioFile {
        source_root,
        native_path: path.to_path_buf(),
        relative_path,
        filename: path
            .file_name()
            .map(|value| value.to_string_lossy().into_owned())
            .unwrap_or_default(),
        extension: recognized_extension(path).unwrap_or_default(),
    }
}

fn normalized_paths(paths: &[PathBuf]) -> Vec<PathBuf> {
    let mut paths = paths.to_vec();
    paths.sort();
    paths.dedup();
    paths
}

fn safe_absolute(path: &Path) -> bool {
    path.is_absolute()
        && !path
            .components()
            .any(|component| matches!(component, Component::ParentDir))
}

fn failed_result(
    request: &AssetFileSnapshotRequest,
    errors: Vec<AssetInventoryError>,
) -> AssetInventoryResult {
    AssetInventoryResult {
        metadata: metadata(request, "failed", 0, 0, 0, errors.len()),
        file_occurrences: Vec::new(),
        content_records: Vec::new(),
        warnings: Vec::new(),
        errors,
    }
}

fn metadata(
    request: &AssetFileSnapshotRequest,
    status: &str,
    processed_paths: usize,
    audio_files: usize,
    content_records: usize,
    error_count: usize,
) -> AssetInventoryMetadata {
    AssetInventoryMetadata {
        inventory_version: ASSET_INVENTORY_VERSION.to_string(),
        scan_run_id: request.scan_run_id.clone(),
        scan_status: status.to_string(),
        requested_root_count: request.paths.len(),
        scanned_root_count: processed_paths,
        entries_visited: processed_paths,
        audio_file_count: audio_files,
        content_record_count: content_records,
        skipped_symlink_count: 0,
        warning_count: 0,
        error_count,
    }
}

fn error(code: &str, message: &str, path: &Path) -> AssetInventoryError {
    AssetInventoryError {
        error_code: code.to_string(),
        message: message.to_string(),
        path: (!path.as_os_str().is_empty()).then(|| path.to_path_buf()),
    }
}
