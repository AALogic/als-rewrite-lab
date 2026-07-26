use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RawAlsPath(pub String);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ParsedAlsPath {
    pub raw: String,
    pub kind: AlsPathKind,
    pub filename: Option<String>,
    pub extension: Option<String>,
    pub normalized_display: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AlsPathKind {
    MacAbsolute,
    MacVolumeAbsolute,
    WindowsDriveAbsolute,
    WindowsUnc,
    RelativeForwardSlash,
    RelativeBackslash,
    BareFilename,
    Empty,
    Unknown,
}

pub fn parse_als_path(raw: impl Into<String>) -> ParsedAlsPath {
    let raw = raw.into();
    let trimmed = raw.trim();
    let kind = classify_als_path(trimmed);
    let filename = filename_from_raw_path_text(trimmed);
    let extension = filename
        .as_ref()
        .and_then(|filename| extension_from_filename(filename));

    ParsedAlsPath {
        normalized_display: normalized_display(trimmed),
        raw,
        kind,
        filename,
        extension,
    }
}

fn classify_als_path(value: &str) -> AlsPathKind {
    if value.is_empty() {
        return AlsPathKind::Empty;
    }

    if value.starts_with(r"\\") {
        return AlsPathKind::WindowsUnc;
    }

    if is_windows_drive_absolute(value) {
        return AlsPathKind::WindowsDriveAbsolute;
    }

    if has_windows_drive_prefix(value) {
        return AlsPathKind::Unknown;
    }

    if value.starts_with("/Volumes/") {
        return AlsPathKind::MacVolumeAbsolute;
    }

    if value.starts_with('/') {
        return AlsPathKind::MacAbsolute;
    }

    if value.contains('\\') {
        return AlsPathKind::RelativeBackslash;
    }

    if value.contains('/') {
        return AlsPathKind::RelativeForwardSlash;
    }

    AlsPathKind::BareFilename
}

fn is_windows_drive_absolute(value: &str) -> bool {
    let bytes = value.as_bytes();
    bytes.len() >= 3
        && bytes[0].is_ascii_alphabetic()
        && bytes[1] == b':'
        && (bytes[2] == b'\\' || bytes[2] == b'/')
}

fn has_windows_drive_prefix(value: &str) -> bool {
    let bytes = value.as_bytes();
    bytes.len() >= 2 && bytes[0].is_ascii_alphabetic() && bytes[1] == b':'
}

fn filename_from_raw_path_text(value: &str) -> Option<String> {
    value
        .rsplit(['/', '\\'])
        .find(|segment| !segment.is_empty())
        .map(ToString::to_string)
}

fn extension_from_filename(filename: &str) -> Option<String> {
    let (stem, extension) = filename.rsplit_once('.')?;
    if stem.is_empty() || extension.is_empty() {
        return None;
    }
    Some(extension.to_string())
}

fn normalized_display(value: &str) -> String {
    value.replace('\\', "/")
}
