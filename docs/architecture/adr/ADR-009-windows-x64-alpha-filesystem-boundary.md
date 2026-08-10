# ADR-009: Windows x64 Alpha Filesystem Boundary

Status: accepted
Date: 2026-08-03

## Context

The shared Rust and Tauri code compiles on Windows, but the write path still
contains Unix-only assumptions. Windows needs native replace-existing and
no-clobber directory-move primitives before the desktop copy flow can be used.

Windows platform support and Ableton document-version support are separate.
This decision does not authorize rewrite for Live 9, Live 10, or Live 12.

## Decision

The private Windows Alpha supports this bounded environment:

```text
target host: serviced Windows 11 x64
legacy private test host: Windows 10 x64
filesystem: local NTFS
operation: one selected project
installer: unsigned per-user NSIS with offline WebView2
```

The Alpha rejects network/UNC roots, cross-volume promotion, path components
implemented by reparse points, and Win32 paths at or above the legacy
`MAX_PATH` boundary. Cloud-sync folders and mounted folders are unsupported
until they have explicit evidence and tests.

The existing module boundaries remain unchanged:

- `rescue_rewriter` uses `ReplaceFileW` to replace only the staged ALS. It
  writes and validates a temporary sibling first, retains a same-volume backup
  during replacement, and reports a stable application error code plus the raw
  Windows error number on failure.
- `rescue_promotion` verifies local NTFS, one volume, and no reparse points,
  then uses `MoveFileExW` without `MOVEFILE_REPLACE_EXISTING`. An existing
  target remains a conflict.
- `rescue_manifest` retains its create-new temporary file plus no-clobber hard
  link policy. Native Windows tests must prove this behavior on NTFS.
- `rescue_application` produces a redacted operation diagnostic. It contains
  build/platform/stage/error evidence but no project paths or media names.

No platform-specific rule moves into the UI, pipeline policy, ALS parser, or
asset matching modules.

## Distribution

GitHub Actions builds the unsigned x64 NSIS installer on a native Windows
runner. The installer uses current-user mode and embeds the WebView2 offline
installer. It is a private test artifact, not a public release.

## Consequences

- macOS and Windows share product/domain code and one repository.
- Windows filesystem risk stays behind narrow adapters owned by write modules.
- The Alpha fails closed outside the tested boundary instead of implying broad
  Windows filesystem support.
- Physical Windows startup, SmartScreen, NTFS, and Ableton behavior still need
  the documented manual test; CI evidence cannot prove those facts.

## Sources

- Microsoft `ReplaceFileW`:
  https://learn.microsoft.com/en-us/windows/win32/api/winbase/nf-winbase-replacefilew
- Microsoft `MoveFileExW`:
  https://learn.microsoft.com/en-us/windows/win32/api/winbase/nf-winbase-movefileexw
- Microsoft reparse points:
  https://learn.microsoft.com/en-us/windows/win32/fileio/reparse-points
- Tauri Windows installer:
  https://v2.tauri.app/distribute/windows-installer/
