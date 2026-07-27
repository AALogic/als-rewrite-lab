use crate::asset_inventory_hash::hash_stable_file;
use crate::asset_inventory_walk::{discover_audio_files, push_warning};
use crate::{
    AssetInventoryError, AssetInventoryMetadata, AssetInventoryRequest, AssetInventoryResult,
    ContentRecord, FileOccurrence, ASSET_INVENTORY_VERSION,
};
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

pub(crate) fn scan_assets_impl(request: &AssetInventoryRequest) -> AssetInventoryResult {
    let roots = normalized_roots(&request.roots);
    let errors = validate_request(request, &roots);
    if !errors.is_empty() {
        return failed_result(request, errors);
    }

    let walk = discover_audio_files(&roots, request.max_entries);
    let mut warnings = walk.warnings;
    let mut hashed_files = Vec::new();
    let mut partial = walk.partial;
    for discovered in walk.files {
        match hash_stable_file(&discovered.native_path) {
            Ok(hash) => hashed_files.push((discovered, hash)),
            Err(failure) => {
                partial = true;
                push_warning(
                    &mut warnings,
                    failure.code,
                    failure.message,
                    discovered.native_path,
                );
            }
        }
    }
    let (file_occurrences, content_records) = build_records(hashed_files);
    let scan_status = if partial { "partial" } else { "complete" };
    AssetInventoryResult {
        metadata: metadata(
            request,
            scan_status,
            roots.len(),
            walk.entries_visited,
            file_occurrences.len(),
            content_records.len(),
            walk.skipped_symlink_count,
            warnings.len(),
            0,
        ),
        file_occurrences,
        content_records,
        warnings,
        errors: Vec::new(),
    }
}

fn validate_request(
    request: &AssetInventoryRequest,
    roots: &[PathBuf],
) -> Vec<AssetInventoryError> {
    let mut errors = Vec::new();
    if roots.is_empty() {
        errors.push(error(
            "INVENTORY_EMPTY_SCOPE",
            "At least one selected scan root is required",
            None,
        ));
    }
    if request.max_entries == 0 {
        errors.push(error(
            "INVENTORY_INVALID_ENTRY_LIMIT",
            "Entry limit must be positive",
            None,
        ));
    }
    for root in roots {
        if !root.is_absolute() {
            errors.push(error(
                "INVENTORY_ROOT_NOT_ABSOLUTE",
                "Selected scan root must be absolute",
                Some(root),
            ));
            continue;
        }
        validate_root_metadata(root, &mut errors);
    }
    errors
}

fn validate_root_metadata(root: &Path, errors: &mut Vec<AssetInventoryError>) {
    match fs::symlink_metadata(root) {
        Ok(metadata) if metadata.file_type().is_symlink() => errors.push(error(
            "INVENTORY_ROOT_IS_SYMLINK",
            "Selected scan root cannot be a symlink",
            Some(root),
        )),
        Ok(metadata) if !metadata.is_dir() => errors.push(error(
            "INVENTORY_ROOT_NOT_DIRECTORY",
            "Selected scan root is not a directory",
            Some(root),
        )),
        Ok(_) => {}
        Err(io_error) if io_error.kind() == std::io::ErrorKind::NotFound => errors.push(error(
            "INVENTORY_ROOT_NOT_FOUND",
            "Selected scan root was not found",
            Some(root),
        )),
        Err(_) => errors.push(error(
            "INVENTORY_ROOT_METADATA_FAILED",
            "Selected scan root metadata could not be read",
            Some(root),
        )),
    }
}

fn normalized_roots(roots: &[PathBuf]) -> Vec<PathBuf> {
    let mut roots = roots.to_vec();
    roots.sort();
    roots.dedup();
    roots
}

fn build_records(
    mut hashed_files: Vec<(
        crate::asset_inventory_walk::DiscoveredAudioFile,
        crate::asset_inventory_hash::HashObservation,
    )>,
) -> (Vec<FileOccurrence>, Vec<ContentRecord>) {
    hashed_files.sort_by(|left, right| left.0.native_path.cmp(&right.0.native_path));
    let mut content_occurrences: BTreeMap<(String, u64), Vec<String>> = BTreeMap::new();
    let file_occurrences = hashed_files
        .into_iter()
        .enumerate()
        .map(|(index, (file, hash))| {
            let occurrence_id = format!("file_occurrence_{index:06}");
            let content_id = format!("sha256:{}", hash.digest);
            content_occurrences
                .entry((hash.digest, hash.file_size))
                .or_default()
                .push(occurrence_id.clone());
            FileOccurrence {
                file_occurrence_id: occurrence_id,
                content_id,
                source_root: file.source_root,
                native_path: file.native_path,
                relative_path: file.relative_path,
                filename: file.filename,
                extension: file.extension,
                file_size: hash.file_size,
                entry_kind: "regular_file".to_string(),
                observation_status: "stable_full_hash".to_string(),
            }
        })
        .collect();
    let content_records = content_occurrences
        .into_iter()
        .map(|((digest, file_size), occurrence_ids)| ContentRecord {
            content_id: format!("sha256:{digest}"),
            hash_algorithm: "sha256_full_bytes".to_string(),
            digest,
            file_size,
            occurrence_ids,
        })
        .collect();
    (file_occurrences, content_records)
}

fn failed_result(
    request: &AssetInventoryRequest,
    errors: Vec<AssetInventoryError>,
) -> AssetInventoryResult {
    AssetInventoryResult {
        metadata: metadata(request, "failed", 0, 0, 0, 0, 0, 0, errors.len()),
        file_occurrences: Vec::new(),
        content_records: Vec::new(),
        warnings: Vec::new(),
        errors,
    }
}

#[allow(clippy::too_many_arguments)]
fn metadata(
    request: &AssetInventoryRequest,
    status: &str,
    scanned_roots: usize,
    entries_visited: usize,
    audio_files: usize,
    content_records: usize,
    skipped_symlinks: usize,
    warning_count: usize,
    error_count: usize,
) -> AssetInventoryMetadata {
    AssetInventoryMetadata {
        inventory_version: ASSET_INVENTORY_VERSION.to_string(),
        scan_run_id: request.scan_run_id.clone(),
        scan_status: status.to_string(),
        requested_root_count: request.roots.len(),
        scanned_root_count: scanned_roots,
        entries_visited,
        audio_file_count: audio_files,
        content_record_count: content_records,
        skipped_symlink_count: skipped_symlinks,
        warning_count,
        error_count,
    }
}

fn error(code: &str, message: &str, path: Option<&Path>) -> AssetInventoryError {
    AssetInventoryError {
        error_code: code.to_string(),
        message: message.to_string(),
        path: path.map(Path::to_path_buf),
    }
}
