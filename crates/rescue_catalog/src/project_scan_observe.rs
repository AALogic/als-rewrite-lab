use crate::project_scan_walk::{
    DirectoryWork, ProjectWalkResult, RawAlsObservation, RawMarkerObservation,
};
use crate::{ProjectScanRequest, ProjectScanWarning};
use std::collections::VecDeque;
use std::fs::{self, Metadata};
use std::path::{Path, PathBuf};
use std::time::UNIX_EPOCH;

pub(crate) fn inspect_entry(
    request: &ProjectScanRequest,
    exclusions: &[PathBuf],
    work: &DirectoryWork,
    path: PathBuf,
    queue: &mut VecDeque<DirectoryWork>,
    result: &mut ProjectWalkResult,
) {
    if is_excluded(&path, exclusions) {
        result.skipped_excluded_count += 1;
        return;
    }
    let metadata = match fs::symlink_metadata(&path) {
        Ok(metadata) => metadata,
        Err(_) => {
            result.partial = true;
            push_warning(
                result,
                "PROJECT_SCAN_ENTRY_METADATA_FAILED",
                "Directory entry metadata could not be read",
                path,
            );
            return;
        }
    };
    if is_traversal_link(&metadata) {
        result.skipped_symlink_count += 1;
        let code = traversal_link_warning_code(&metadata);
        push_warning(result, code, "Filesystem link was not followed", path);
    } else if metadata.is_dir() {
        inspect_directory(request, work, path, queue, result);
    } else if metadata.is_file() {
        inspect_regular_file(work, path, metadata, result);
    }
}

fn inspect_directory(
    request: &ProjectScanRequest,
    work: &DirectoryWork,
    path: PathBuf,
    queue: &mut VecDeque<DirectoryWork>,
    result: &mut ProjectWalkResult,
) {
    if path.file_name().and_then(|name| name.to_str()) == Some("Ableton Project Info") {
        add_marker(&work.source_root, path, result);
        return;
    }
    if request.max_depth.is_some_and(|limit| work.depth >= limit) {
        result.partial = true;
        push_warning(
            result,
            "PROJECT_SCAN_DEPTH_LIMIT_REACHED",
            "Depth limit prevented directory traversal",
            path,
        );
        return;
    }
    queue.push_back(DirectoryWork {
        source_root: work.source_root.clone(),
        directory: path,
        depth: work.depth + 1,
    });
}

fn inspect_regular_file(
    work: &DirectoryWork,
    path: PathBuf,
    metadata: Metadata,
    result: &mut ProjectWalkResult,
) {
    if !is_als_path(&path) {
        return;
    }
    let Some(filename) = unicode_filename(&path) else {
        result.partial = true;
        push_warning(
            result,
            "PROJECT_SCAN_NON_UNICODE_PATH",
            "Non-Unicode ALS path is not supported by the public contract",
            path,
        );
        return;
    };
    let Ok(relative_path) = path.strip_prefix(&work.source_root).map(Path::to_path_buf) else {
        result.partial = true;
        push_warning(
            result,
            "PROJECT_SCAN_SCOPE_INVARIANT_FAILED",
            "Observed ALS was not lexically inside its source root",
            path,
        );
        return;
    };
    result.als_files.push(RawAlsObservation {
        source_root: work.source_root.clone(),
        native_path: path,
        relative_path,
        filename,
        file_size: metadata.len(),
        modified_time_unix_ms: modified_time_ms(&metadata),
    });
}

fn add_marker(source_root: &Path, marker_path: PathBuf, result: &mut ProjectWalkResult) {
    if marker_path.to_str().is_none() || source_root.to_str().is_none() {
        result.partial = true;
        push_warning(
            result,
            "PROJECT_SCAN_NON_UNICODE_PATH",
            "Non-Unicode Project marker path is not supported by the public contract",
            marker_path,
        );
        return;
    }
    let Some(project_root_candidate) = marker_path.parent().map(Path::to_path_buf) else {
        return;
    };
    let Ok(relative_marker_path) = marker_path.strip_prefix(source_root).map(Path::to_path_buf)
    else {
        result.partial = true;
        push_warning(
            result,
            "PROJECT_SCAN_SCOPE_INVARIANT_FAILED",
            "Project marker was not lexically inside its source root",
            marker_path,
        );
        return;
    };
    result.project_markers.push(RawMarkerObservation {
        source_root: source_root.to_path_buf(),
        project_root_candidate,
        marker_path,
        relative_marker_path,
    });
}

fn unicode_filename(path: &Path) -> Option<String> {
    path.to_str()?;
    path.file_name()?.to_str().map(str::to_string)
}

fn is_als_path(path: &Path) -> bool {
    path.extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| extension.eq_ignore_ascii_case("als"))
}

fn modified_time_ms(metadata: &Metadata) -> Option<u64> {
    let millis = metadata
        .modified()
        .ok()?
        .duration_since(UNIX_EPOCH)
        .ok()?
        .as_millis();
    u64::try_from(millis).ok()
}

fn is_excluded(path: &Path, exclusions: &[PathBuf]) -> bool {
    exclusions.iter().any(|excluded| path.starts_with(excluded))
}

fn is_traversal_link(metadata: &Metadata) -> bool {
    if metadata.file_type().is_symlink() {
        return true;
    }
    #[cfg(windows)]
    {
        use std::os::windows::fs::MetadataExt;
        const FILE_ATTRIBUTE_REPARSE_POINT: u32 = 0x400;
        return metadata.file_attributes() & FILE_ATTRIBUTE_REPARSE_POINT != 0;
    }
    #[cfg(not(windows))]
    false
}

fn traversal_link_warning_code(metadata: &Metadata) -> &'static str {
    if metadata.file_type().is_symlink() {
        return "PROJECT_SCAN_SYMLINK_SKIPPED";
    }
    "PROJECT_SCAN_REPARSE_POINT_SKIPPED"
}

pub(crate) fn push_warning(
    result: &mut ProjectWalkResult,
    code: &str,
    message: &str,
    path: PathBuf,
) {
    result.warnings.push(ProjectScanWarning {
        warning_id: result.warnings.len(),
        warning_code: code.to_string(),
        message: message.to_string(),
        path: Some(path),
    });
}
