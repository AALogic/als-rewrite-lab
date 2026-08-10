use std::path::PathBuf;

pub(crate) struct ProjectScanScope {
    pub roots: Vec<PathBuf>,
    pub excluded_roots: Vec<PathBuf>,
}

pub(crate) fn platform_project_scan_scope(home: Option<PathBuf>) -> ProjectScanScope {
    let mut roots = platform_roots(home.as_deref());
    let mut excluded_roots = platform_exclusions(home.as_deref(), &roots);
    roots.sort();
    roots.dedup();
    excluded_roots.sort();
    excluded_roots.dedup();
    ProjectScanScope {
        roots,
        excluded_roots,
    }
}

#[cfg(target_os = "macos")]
fn platform_roots(_home: Option<&std::path::Path>) -> Vec<PathBuf> {
    existing_paths([PathBuf::from("/Users"), PathBuf::from("/Volumes")])
}

#[cfg(target_os = "macos")]
fn platform_exclusions(home: Option<&std::path::Path>, _roots: &[PathBuf]) -> Vec<PathBuf> {
    let Some(home) = home else {
        return Vec::new();
    };
    [
        "Library",
        ".Trash",
        ".cache",
        ".cargo",
        ".rustup",
        ".npm",
        ".pnpm-store",
    ]
    .into_iter()
    .map(|suffix| home.join(suffix))
    .collect()
}

#[cfg(windows)]
fn platform_roots(_home: Option<&std::path::Path>) -> Vec<PathBuf> {
    use windows_sys::Win32::Storage::FileSystem::{
        GetDriveTypeW, GetLogicalDrives, DRIVE_FIXED, DRIVE_REMOVABLE,
    };

    let mask = unsafe { GetLogicalDrives() };
    (0..26)
        .filter(|index| mask & (1 << index) != 0)
        .filter_map(|index| {
            let letter = b'A' + index as u8;
            let wide = [letter as u16, b':' as u16, b'\\' as u16, 0];
            let drive_type = unsafe { GetDriveTypeW(wide.as_ptr()) };
            matches!(drive_type, DRIVE_FIXED | DRIVE_REMOVABLE)
                .then(|| PathBuf::from(format!("{}:\\", letter as char)))
        })
        .collect()
}

#[cfg(windows)]
fn platform_exclusions(home: Option<&std::path::Path>, roots: &[PathBuf]) -> Vec<PathBuf> {
    let mut exclusions = Vec::new();
    for root in roots {
        for suffix in [
            "$Recycle.Bin",
            "System Volume Information",
            "Windows",
            "Program Files",
            "Program Files (x86)",
            "ProgramData",
        ] {
            exclusions.push(root.join(suffix));
        }
    }
    if let Some(home) = home {
        exclusions.extend([home.join("AppData"), home.join(".cache")]);
    }
    exclusions
}

#[cfg(not(any(target_os = "macos", windows)))]
fn platform_roots(home: Option<&std::path::Path>) -> Vec<PathBuf> {
    existing_paths(
        home.into_iter()
            .map(std::path::Path::to_path_buf)
            .chain([PathBuf::from("/mnt"), PathBuf::from("/media")]),
    )
}

#[cfg(not(any(target_os = "macos", windows)))]
fn platform_exclusions(home: Option<&std::path::Path>, _roots: &[PathBuf]) -> Vec<PathBuf> {
    let Some(home) = home else {
        return Vec::new();
    };
    [".cache", ".local/share/Trash", ".cargo", ".rustup", ".npm"]
        .into_iter()
        .map(|suffix| home.join(suffix))
        .collect()
}

fn existing_paths(paths: impl IntoIterator<Item = PathBuf>) -> Vec<PathBuf> {
    paths.into_iter().filter(|path| path.is_dir()).collect()
}

#[cfg(test)]
mod tests {
    use super::platform_project_scan_scope;
    use std::path::PathBuf;

    #[test]
    fn platform_scope_is_deduplicated_and_absolute() {
        let scope = platform_project_scan_scope(std::env::var_os("HOME").map(Into::into));
        assert!(!scope.roots.is_empty());
        assert!(scope.roots.iter().all(|path| path.is_absolute()));
        assert!(scope.excluded_roots.iter().all(|path| path.is_absolute()));
        assert!(scope.roots.windows(2).all(|pair| pair[0] < pair[1]));
        assert!(scope
            .excluded_roots
            .windows(2)
            .all(|pair| pair[0] < pair[1]));
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn exclusions_do_not_require_existing_or_readable_directories() {
        let home = PathBuf::from("/definitely-not-an-existing-home");
        let scope = platform_project_scan_scope(Some(home.clone()));
        assert!(scope.excluded_roots.contains(&home.join("Library")));
        assert!(scope.excluded_roots.contains(&home.join(".Trash")));
    }
}
