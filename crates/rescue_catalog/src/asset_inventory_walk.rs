use crate::AssetInventoryWarning;
use std::collections::{BTreeSet, VecDeque};
use std::fs;
use std::path::{Path, PathBuf};

const AUDIO_EXTENSIONS: &[&str] = &["aac", "aif", "aiff", "flac", "m4a", "mp3", "ogg", "wav"];

pub(crate) struct DiscoveredAudioFile {
    pub source_root: PathBuf,
    pub native_path: PathBuf,
    pub relative_path: PathBuf,
    pub filename: String,
    pub extension: String,
}

pub(crate) struct WalkResult {
    pub files: Vec<DiscoveredAudioFile>,
    pub entries_visited: usize,
    pub skipped_symlink_count: usize,
    pub warnings: Vec<AssetInventoryWarning>,
    pub partial: bool,
}

pub(crate) fn discover_audio_files(roots: &[PathBuf], max_entries: usize) -> WalkResult {
    let mut queue: VecDeque<_> = roots
        .iter()
        .cloned()
        .map(|root| (root.clone(), root))
        .collect();
    let mut seen_paths = BTreeSet::new();
    let mut result = empty_result();

    while let Some((source_root, directory)) = queue.pop_front() {
        if !seen_paths.insert(directory.clone()) {
            continue;
        }
        let entries = match sorted_entries(&directory) {
            Ok(entries) => entries,
            Err(()) => {
                result.partial = true;
                push_warning(
                    &mut result.warnings,
                    "INVENTORY_DIRECTORY_READ_FAILED",
                    "Directory could not be read",
                    directory,
                );
                continue;
            }
        };
        for path in entries {
            if result.entries_visited >= max_entries {
                result.partial = true;
                push_warning(
                    &mut result.warnings,
                    "INVENTORY_ENTRY_LIMIT_REACHED",
                    "Entry limit reached before scan completion",
                    path,
                );
                return result;
            }
            result.entries_visited += 1;
            inspect_entry(&source_root, path, &mut queue, &mut seen_paths, &mut result);
        }
    }
    result
        .files
        .sort_by(|left, right| left.native_path.cmp(&right.native_path));
    result
}

fn inspect_entry(
    source_root: &Path,
    path: PathBuf,
    queue: &mut VecDeque<(PathBuf, PathBuf)>,
    seen_paths: &mut BTreeSet<PathBuf>,
    result: &mut WalkResult,
) {
    let metadata = match fs::symlink_metadata(&path) {
        Ok(metadata) => metadata,
        Err(_) => {
            result.partial = true;
            push_warning(
                &mut result.warnings,
                "INVENTORY_ENTRY_METADATA_FAILED",
                "Directory entry metadata could not be read",
                path,
            );
            return;
        }
    };
    if metadata.file_type().is_symlink() {
        result.skipped_symlink_count += 1;
        push_warning(
            &mut result.warnings,
            "INVENTORY_SYMLINK_SKIPPED",
            "Symlink was not followed",
            path,
        );
    } else if metadata.is_dir() {
        if !seen_paths.contains(&path) {
            queue.push_back((source_root.to_path_buf(), path));
        }
    } else if metadata.is_file() {
        add_audio_file(source_root, path, result);
    }
}

fn add_audio_file(source_root: &Path, path: PathBuf, result: &mut WalkResult) {
    let Some(extension) = recognized_extension(&path) else {
        return;
    };
    let Some(filename) = path
        .file_name()
        .and_then(|value| value.to_str())
        .map(str::to_string)
    else {
        result.partial = true;
        push_warning(
            &mut result.warnings,
            "INVENTORY_NON_UNICODE_FILENAME",
            "Non-Unicode filename is not supported by the JSON contract",
            path,
        );
        return;
    };
    let Ok(relative_path) = path.strip_prefix(source_root).map(Path::to_path_buf) else {
        result.partial = true;
        push_warning(
            &mut result.warnings,
            "INVENTORY_SCOPE_INVARIANT_FAILED",
            "Discovered file was not lexically inside its selected root",
            path,
        );
        return;
    };
    result.files.push(DiscoveredAudioFile {
        source_root: source_root.to_path_buf(),
        native_path: path,
        relative_path,
        filename,
        extension,
    });
}

pub(crate) fn recognized_extension(path: &Path) -> Option<String> {
    let extension = path.extension()?.to_str()?.to_ascii_lowercase();
    AUDIO_EXTENSIONS
        .contains(&extension.as_str())
        .then_some(extension)
}

fn sorted_entries(directory: &Path) -> Result<Vec<PathBuf>, ()> {
    let entries = fs::read_dir(directory).map_err(|_| ())?;
    let mut paths = Vec::new();
    for entry in entries {
        paths.push(entry.map_err(|_| ())?.path());
    }
    paths.sort();
    Ok(paths)
}

fn empty_result() -> WalkResult {
    WalkResult {
        files: Vec::new(),
        entries_visited: 0,
        skipped_symlink_count: 0,
        warnings: Vec::new(),
        partial: false,
    }
}

pub(crate) fn push_warning(
    warnings: &mut Vec<AssetInventoryWarning>,
    code: &str,
    message: &str,
    path: PathBuf,
) {
    warnings.push(AssetInventoryWarning {
        warning_id: warnings.len(),
        warning_code: code.to_string(),
        message: message.to_string(),
        path: Some(path),
    });
}
