# ADR-012: Quick Copy Entry And UI Boundary

Status: accepted
Date: 2026-08-05

## Context

The existing Desktop already creates complete or incomplete self-contained
copies through module 018. The user also needs a low-friction path starting
from one ALS found in Finder, without navigating the full application window.

A direct Finder Sync extension would add contextual menus only inside monitored
folders. Apple describes Finder Sync as a synchronization-oriented extension,
not a general Finder modification mechanism. A persistent selection watcher
would also add permissions and behavior unrelated to the copy capability.

Tauri 2.11.5 already exposes macOS opened-file events, file-association bundle
configuration, cursor position, multiple windows and a supported
single-instance plugin.

## Decision

Version 0.1 uses `Open With -> ALS Rescue` for one `.als` file. ALS Rescue is
registered as `Viewer` with `LSHandlerRank.Alternate`, so Ableton remains the
primary opener.

The existing application binary owns two presentation surfaces:

```text
main window
  Project catalog, analysis, one-project and batch workflows

quick-copy window
  one ALS, destination selection, preview, Play and compact result
```

The quick surface calls the unchanged module-018 target suggestion, preview and
execute operations. It introduces no domain or filesystem policy.

The v0.1 quick surface is small, undecorated and cursor-relative. Visually it
contains only the opaque character, speech bubble and controls: no rectangular
background or system shadow is shown. The character body acts as the drag
handle for the complete surface. On macOS this uses Tauri's
`macos-private-api` transparency flag, so it supports direct signing and
notarization but not Mac App Store submission.

Finder Sync, a Share extension, persistent Finder selection watching, macOS
Services and Windows shell integration are outside this increment. A later
platform adapter may create the same `QuickCopyLaunchContext v0.1`.

## Consequences

- There is one copy behavior and two UI entry surfaces.
- Normal launch behavior remains unchanged.
- Open With can be tested without monitoring the filesystem or Finder.
- The assistant remains simple: deterministic states, short copy and bounded
  sprite loops.
- Internal structured failures remain available to diagnostics, while quick UI
  presents one generic unable message.
- A future Mac App Store build would require an opaque platform variant without
  the private API flag; the direct-download build keeps the floating assistant.
- A later macOS Service or Windows shell menu does not require changes to the
  core pipeline.

## Evidence

- Tauri `RunEvent::Opened` is available on macOS.
- Tauri bundle schema maps file association role and rank to
  `CFBundleTypeRole` and `LSHandlerRank`.
- Tauri exposes application cursor position on macOS and Windows.
- Tauri single-instance plugin supports macOS and Windows and must be
  registered before other plugins.
- Apple Finder Sync contextual menus apply to registered monitored folders and
  Finder Sync is not intended as a general Finder UI modification tool.

References:

- <https://docs.rs/tauri/2.11.5/tauri/enum.RunEvent.html>
- <https://v2.tauri.app/plugin/single-instance/>
- <https://developer.apple.com/library/archive/documentation/General/Conceptual/ExtensibilityPG/Finder.html>
