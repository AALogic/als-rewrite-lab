use crate::project_scan_observe::{inspect_entry, push_warning};
use crate::{ProjectScanObserver, ProjectScanProgress, ProjectScanRequest, ProjectScanWarning};
use std::collections::VecDeque;
use std::fs;
use std::path::{Path, PathBuf};

pub(crate) struct RawAlsObservation {
    pub source_root: PathBuf,
    pub native_path: PathBuf,
    pub relative_path: PathBuf,
    pub filename: String,
    pub file_size: u64,
    pub modified_time_unix_ms: Option<u64>,
}

pub(crate) struct RawMarkerObservation {
    pub source_root: PathBuf,
    pub project_root_candidate: PathBuf,
    pub marker_path: PathBuf,
    pub relative_marker_path: PathBuf,
}

pub(crate) struct ProjectWalkResult {
    pub als_files: Vec<RawAlsObservation>,
    pub project_markers: Vec<RawMarkerObservation>,
    pub warnings: Vec<ProjectScanWarning>,
    pub directories_visited: usize,
    pub entries_visited: usize,
    pub skipped_symlink_count: usize,
    pub skipped_excluded_count: usize,
    pub roots_completed: usize,
    pub partial: bool,
    pub cancelled: bool,
}

pub(crate) struct DirectoryWork {
    pub source_root: PathBuf,
    pub directory: PathBuf,
    pub depth: usize,
}

pub(crate) fn walk_project_roots(
    request: &ProjectScanRequest,
    roots: &[PathBuf],
    exclusions: &[PathBuf],
    observer: &mut dyn ProjectScanObserver,
) -> ProjectWalkResult {
    let mut result = ProjectWalkResult::empty();
    emit_progress(request, "started", &result, observer);
    for root in roots {
        let mut queue = VecDeque::from([DirectoryWork {
            source_root: root.clone(),
            directory: root.clone(),
            depth: 0,
        }]);
        if walk_one_root(request, exclusions, &mut queue, observer, &mut result) {
            break;
        }
        result.roots_completed += 1;
    }
    result
}

fn walk_one_root(
    request: &ProjectScanRequest,
    exclusions: &[PathBuf],
    queue: &mut VecDeque<DirectoryWork>,
    observer: &mut dyn ProjectScanObserver,
    result: &mut ProjectWalkResult,
) -> bool {
    while let Some(work) = queue.pop_front() {
        if observer.is_cancelled() {
            result.cancelled = true;
            return true;
        }
        if is_excluded(&work.directory, exclusions) {
            result.skipped_excluded_count += 1;
            continue;
        }
        let entries = match sorted_entries(&work.directory) {
            Ok(entries) => entries,
            Err(()) => {
                result.partial = true;
                push_warning(
                    result,
                    "PROJECT_SCAN_DIRECTORY_READ_FAILED",
                    "Directory could not be read",
                    work.directory,
                );
                continue;
            }
        };
        result.directories_visited += 1;
        if inspect_entries(request, exclusions, work, entries, queue, observer, result) {
            return true;
        }
        emit_progress(request, "walking", result, observer);
    }
    false
}

#[allow(clippy::too_many_arguments)]
fn inspect_entries(
    request: &ProjectScanRequest,
    exclusions: &[PathBuf],
    work: DirectoryWork,
    entries: Vec<PathBuf>,
    queue: &mut VecDeque<DirectoryWork>,
    observer: &mut dyn ProjectScanObserver,
    result: &mut ProjectWalkResult,
) -> bool {
    for path in entries {
        if observer.is_cancelled() {
            result.cancelled = true;
            return true;
        }
        if result.entries_visited >= request.max_entries {
            result.partial = true;
            push_warning(
                result,
                "PROJECT_SCAN_ENTRY_LIMIT_REACHED",
                "Entry limit reached before scan completion",
                path,
            );
            return true;
        }
        result.entries_visited += 1;
        inspect_entry(request, exclusions, &work, path, queue, result);
    }
    false
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

fn is_excluded(path: &Path, exclusions: &[PathBuf]) -> bool {
    exclusions.iter().any(|excluded| path.starts_with(excluded))
}

pub(crate) fn emit_progress(
    request: &ProjectScanRequest,
    stage: &str,
    result: &ProjectWalkResult,
    observer: &mut dyn ProjectScanObserver,
) {
    observer.on_progress(&ProjectScanProgress {
        scan_run_id: request.scan_run_id.clone(),
        stage: stage.to_string(),
        requested_root_count: request.roots.len(),
        roots_completed: result.roots_completed,
        directories_visited: result.directories_visited,
        entries_visited: result.entries_visited,
        als_file_count: result.als_files.len(),
        project_marker_count: result.project_markers.len(),
        warning_count: result.warnings.len(),
    });
}

impl ProjectWalkResult {
    fn empty() -> Self {
        Self {
            als_files: Vec::new(),
            project_markers: Vec::new(),
            warnings: Vec::new(),
            directories_visited: 0,
            entries_visited: 0,
            skipped_symlink_count: 0,
            skipped_excluded_count: 0,
            roots_completed: 0,
            partial: false,
            cancelled: false,
        }
    }
}
