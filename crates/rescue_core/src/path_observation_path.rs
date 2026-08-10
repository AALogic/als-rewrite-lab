use crate::{parse_als_path, AlsPathKind};
use std::path::{Component, Path, PathBuf};

pub(crate) fn safe_relative_path(raw: &str, platform: &str) -> Result<PathBuf, &'static str> {
    let parsed = parse_als_path(raw);
    let admitted = match platform {
        "macos" | "posix" => matches!(
            parsed.kind,
            AlsPathKind::RelativeForwardSlash | AlsPathKind::BareFilename
        ),
        "windows" => matches!(
            parsed.kind,
            AlsPathKind::RelativeForwardSlash
                | AlsPathKind::RelativeBackslash
                | AlsPathKind::BareFilename
        ),
        _ => false,
    };
    if !admitted {
        return Err("invalid_relative_shape");
    }
    let path = PathBuf::from(raw);
    if has_parent_component(&path) {
        return Err("parent_escape");
    }
    if path
        .components()
        .any(|component| matches!(component, Component::RootDir | Component::Prefix(_)))
    {
        return Err("invalid_relative_shape");
    }
    Ok(path)
}

pub(crate) fn has_parent_component(path: &Path) -> bool {
    path.components()
        .any(|component| component == Component::ParentDir)
}

pub(crate) fn is_native_absolute(kind: &AlsPathKind, platform: &str) -> bool {
    match platform {
        "macos" | "posix" => matches!(
            kind,
            AlsPathKind::MacAbsolute | AlsPathKind::MacVolumeAbsolute
        ),
        "windows" => matches!(
            kind,
            AlsPathKind::WindowsDriveAbsolute | AlsPathKind::WindowsUnc
        ),
        _ => false,
    }
}

pub(crate) fn is_foreign_absolute(kind: &AlsPathKind, platform: &str) -> bool {
    match platform {
        "macos" | "posix" => matches!(
            kind,
            AlsPathKind::WindowsDriveAbsolute | AlsPathKind::WindowsUnc
        ),
        "windows" => matches!(
            kind,
            AlsPathKind::MacAbsolute | AlsPathKind::MacVolumeAbsolute
        ),
        _ => false,
    }
}
