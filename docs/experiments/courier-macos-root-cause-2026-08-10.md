# Courier macOS Root-Cause Report

Status: root causes confirmed in the isolated Courier diagnostics Lab

Date: 2026-08-10

Environment:

- macOS 15.5, build 24F74;
- Apple Silicon (`arm64`);
- Tauri 2.11.5;
- tauri-runtime-wry 2.11.4;
- Wry 0.55.1;
- Tao 0.35.3.

This report covers two failures that had become mixed together during manual
testing:

1. a cold `Open With` crash;
2. loss of Finder file-drop delivery after switching through Safari or Spaces.

They are independent failures. Both were reproduced, instrumented and reduced
to one concrete cause in the Lab copy. No ALS source file was modified.

## Executive Verdict

### Cold Open With

`RunEvent::Opened` can arrive before Tauri executes `.setup`. The event handler
immediately called code that used `app.state::<CourierDropDiagnostics>()`, but
that state was managed only inside `.setup`. Tauri therefore panicked with:

```text
state() called before manage() for
als_rescue_desktop_lib::courier_drop_diagnostics::CourierDropDiagnostics
```

The panic crossed a callback that cannot unwind, producing `SIGABRT` in
`tao::application_open_urls`.

The repair is a small startup buffer managed before `.setup`. Cold Open URLs
are preserved in order, `.setup` marks the runtime ready, and the existing
ready-state route consumes the buffered requests.

### Finder Drop After Safari

The configured `NSScreenSaverWindowLevel` (`1000`) is the cause of missing
Finder drag callbacks on this macOS environment.

The sequence that made the defect look intermittent was:

```text
quick window requests screenSaver level 1000
-> Finder activation temporarily leaves/resets the Tauri window at level 5
-> Finder drop works
-> Safari or Space transition occurs
-> our observer reasserts screenSaver level 1000
-> Finder drag no longer reaches any destination callback
```

In the failed state there is no `tauri_drag_entered`. React, collection state,
ALS validation and the order reset are downstream and cannot cause that
absence.

A standalone AppKit receiver proved the level dependency:

| Native window variant | Before Safari | After Safari |
| --- | --- | --- |
| floating, ordinary Space flags | accepted | accepted |
| floating + join-all-applications | accepted | accepted |
| screenSaver, ordinary Space flags | no callback | no callback |
| screenSaver + join-all-applications | no callback | no callback |

This isolates `screenSaver` from `joinAllApplications`, Wry and React.

The working quick-window policy is:

```text
application activation policy: Accessory while Courier mode is active
window level: NSFloatingWindowLevel
CanJoinAllSpaces: true
FullScreenAuxiliary: true
CanJoinAllApplications: true
Stationary: true
IgnoresCycle: true
HidesOnDeactivate: false
orderFrontRegardless: yes
```

Opening the ordinary main application restores the `Regular` activation
policy through the pre-existing `restore_regular_application_policy` path.

## Evidence Protocol

### Open With Baseline

The Lab was stopped and its stale `launchctl` KeepAlive job was unloaded. The
test then launched the packaged Lab directly with one copied ALS:

```text
open -n -a "ALS Rescue Courier Lab.app" "test_nr_1.als"
```

Baseline result:

- crash report count increased from 10 to 11;
- the process died in under one second;
- the new report contained `SIGABRT` and
  `tao::application_open_urls + 1168`;
- the temporary panic probe captured the exact missing managed state.

The earlier KeepAlive job was a test contaminant because it restarted the Lab,
but it was not the crash cause. The clean launch reproduced after that job was
removed.

### Open With A/B

After adding the ready-state buffer:

- all 49 Rust tests passed;
- the release `.app` built successfully;
- 5 of 5 fresh cold Open With launches remained alive;
- each cold launch loaded one ALS;
- crash report count remained 11;
- panic-record count remained unchanged;
- warm Open With appended a second ALS to the live collection.

### Finder Drag Driver

The initial UI automation moved too quickly to prove a native file drag. A
small `CGEvent` driver was therefore added. It:

1. moves to a Finder icon;
2. holds the left button for 350 ms;
3. traverses the path in approximately 16 ms steps;
4. pauses over the target;
5. releases the file.

The same driver produced the complete native AppKit sequence and complete
Tauri sequence, so subsequent negative results are meaningful:

```text
draggingEntered
prepareForDragOperation
performDragOperation
concludeDragOperation
```

### Tauri ScreenSaver Baseline

With the old policy, an early Finder test succeeded while diagnostics reported
window level `5`. After Safari and an active-Space transition, the observer
reasserted `1000`. The same driver and same coordinates then produced no
`tauri_drag_entered`.

The Finder source window and Courier target window coordinates were queried
from Accessibility before the attempt, preventing an off-target result.

### Tauri Floating Verification

With `NSFloatingWindowLevel`:

- the same post-Safari scenario succeeded;
- five consecutive Safari -> Finder -> drop cycles produced five deliveries;
- the level remained `3` when the Space policy was reasserted.

With `Accessory + NSFloatingWindowLevel`:

- the Courier appeared in `CGWindowListCopyWindowInfo(.optionOnScreenOnly)`
  above the large Safari content window;
- 3 of 3 Safari -> Finder cycles reported `onscreen_over_safari=yes`;
- 3 of 3 cycles produced `tauri_drop_received`;
- Safari was set to `AXFullScreen=true`; the Courier remained onscreen;
- the following Finder transition and drop succeeded once again.

## Why Earlier Repairs Did Not Close The Problem

Removing the application's competing registration on `WryWebViewParent` was a
valid ownership correction. Wry should remain the sole inbound Finder-drop
owner. It did not address the window-server routing failure created by the
screen-saver level.

Likewise, resetting collection or outbound parcel state cannot rearm a drag
destination that never receives `draggingEntered`. The apparent relation to
`Nowe zlecenie` was temporal: users commonly changed application or Space
during a completed order, and that transition reasserted level `1000`.

## Implemented Lab Changes

1. `StartupOpenUrlBuffer` protects Open With events that arrive before setup.
2. `finish_startup` drains pending Open URLs after diagnostics state exists.
3. the quick window uses `NSFloatingWindowLevel` instead of
   `NSScreenSaverWindowLevel`;
4. Courier mode uses the Accessory activation policy;
5. opening the normal main window still restores Regular activation policy;
6. a standalone AppKit receiver and deterministic drag driver preserve the
   diagnosis as a repeatable laboratory experiment;
7. the startup panic probe remains Lab-only evidence instrumentation.

## Recommended Production Change

Promote the two small behavioral repairs from the Lab only after review:

1. merge the startup Open URL buffer;
2. replace the screen-saver level with floating level;
3. use Accessory activation only while the quick Courier surface is active;
4. restore Regular activation before showing the full application;
5. retain Wry as the only inbound file-drop destination;
6. retain `CanJoinAllSpaces`, `FullScreenAuxiliary` and
   `CanJoinAllApplications` because the tested combination remained onscreen;
7. do not add registration/rearm calls on reset;
8. preserve a packaged macOS regression protocol because Finder/AppKit drag
   cannot be fully proven by headless unit tests.

Do not replace the Tauri/Wry drop path with a custom native destination. The
native probe demonstrated that the existing destination works once the window
level is compatible with Finder drag routing.

## Residual Risk

The result is confirmed for macOS 15.5 on one Apple Silicon machine. The
policy should still be checked on the oldest supported macOS release and on an
Intel machine before commercial distribution. That is platform coverage, not
an unresolved root cause for this machine.

`CanJoinAllApplications` is a platform-specific behavior and should stay
isolated behind the macOS window-policy module. It must not leak into ALS,
copying or application-service contracts.

## Evidence Files

- `artifacts/courier-open-with-startup-panic-2026-08-10.jsonl`
- `artifacts/courier-drop-regular-screen-saver-current-2026-08-10.jsonl`
- `artifacts/native-drop-floating-matrix-2026-08-10.jsonl`
- `artifacts/native-drop-screensaver-nojoin-2026-08-10.jsonl`
- `artifacts/native-drop-screensaver-joinall-2026-08-10.jsonl`
- `artifacts/native-drop-floating-joinall-2026-08-10.jsonl`
- `artifacts/courier-drop-floating-accessory-confirmed-2026-08-10.jsonl`

The disposable test tools live in `tools/macos-drop-probe`.

## Source Notes

- Tao issue showing Open URLs may precede the normal ready/setup point:
  https://github.com/tauri-apps/tao/issues/1235
- Apple's drag-destination overview:
  https://developer.apple.com/library/archive/documentation/Cocoa/Conceptual/DragandDrop/Concepts/dragdestination.html
- Wry's installed implementation:
  `wry-0.55.1/src/wkwebview/class/wry_web_view.rs` and
  `wry-0.55.1/src/wkwebview/drag_drop.rs`
- Tao 0.35.3 also registers the native window for legacy filenames in
  `tao-0.35.3/src/platform_impl/macos/window.rs`; the controlled level matrix,
  rather than source inspection alone, establishes the cause observed here.
