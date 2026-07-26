use rescue_core::{parse_als_path, AlsPathKind};

#[test]
fn path_parser_windows_drive_on_macos_style_string() {
    let parsed = parse_als_path(r"C:\Samples\Kick.wav");

    assert_eq!(parsed.kind, AlsPathKind::WindowsDriveAbsolute);
    assert_eq!(parsed.filename.as_deref(), Some("Kick.wav"));
    assert_eq!(parsed.extension.as_deref(), Some("wav"));
    assert_eq!(parsed.normalized_display, "C:/Samples/Kick.wav");
}

#[test]
fn path_parser_windows_unc_path() {
    let parsed = parse_als_path(r"\\StudioNas\Samples\Snare.aif");

    assert_eq!(parsed.kind, AlsPathKind::WindowsUnc);
    assert_eq!(parsed.filename.as_deref(), Some("Snare.aif"));
    assert_eq!(parsed.extension.as_deref(), Some("aif"));
}

#[test]
fn path_parser_mac_absolute_path() {
    let parsed = parse_als_path("/tmp/Samples/Hat.wav");

    assert_eq!(parsed.kind, AlsPathKind::MacAbsolute);
    assert_eq!(parsed.filename.as_deref(), Some("Hat.wav"));
    assert_eq!(parsed.extension.as_deref(), Some("wav"));
}

#[test]
fn path_parser_mac_volume_path() {
    let parsed = parse_als_path("/Volumes/SampleDisk/Pack/Loop.aiff");

    assert_eq!(parsed.kind, AlsPathKind::MacVolumeAbsolute);
    assert_eq!(parsed.filename.as_deref(), Some("Loop.aiff"));
    assert_eq!(parsed.extension.as_deref(), Some("aiff"));
}

#[test]
fn path_parser_relative_forward_slash_path() {
    let parsed = parse_als_path("Samples/Imported/Vocal.flac");

    assert_eq!(parsed.kind, AlsPathKind::RelativeForwardSlash);
    assert_eq!(parsed.filename.as_deref(), Some("Vocal.flac"));
    assert_eq!(parsed.extension.as_deref(), Some("flac"));
}

#[test]
fn path_parser_relative_backslash_path() {
    let parsed = parse_als_path(r"Samples\Imported\Vocal.flac");

    assert_eq!(parsed.kind, AlsPathKind::RelativeBackslash);
    assert_eq!(parsed.filename.as_deref(), Some("Vocal.flac"));
    assert_eq!(parsed.extension.as_deref(), Some("flac"));
}

#[test]
fn path_parser_bare_filename() {
    let parsed = parse_als_path("808 Kick.wav");

    assert_eq!(parsed.kind, AlsPathKind::BareFilename);
    assert_eq!(parsed.filename.as_deref(), Some("808 Kick.wav"));
    assert_eq!(parsed.extension.as_deref(), Some("wav"));
}

#[test]
fn path_parser_empty_path() {
    let parsed = parse_als_path("   ");

    assert_eq!(parsed.kind, AlsPathKind::Empty);
    assert_eq!(parsed.filename, None);
    assert_eq!(parsed.extension, None);
}

#[test]
fn path_parser_preserves_raw_path() {
    let raw = r"C:\Messy Downloads\Some Sample.wav";
    let parsed = parse_als_path(raw);

    assert_eq!(parsed.raw, raw);
    assert_eq!(
        parsed.normalized_display,
        "C:/Messy Downloads/Some Sample.wav"
    );
}

#[test]
fn path_parser_windows_drive_relative_is_unknown() {
    let parsed = parse_als_path("C:Samples/Kick.wav");

    assert_eq!(parsed.kind, AlsPathKind::Unknown);
    assert_eq!(parsed.filename.as_deref(), Some("Kick.wav"));
}
