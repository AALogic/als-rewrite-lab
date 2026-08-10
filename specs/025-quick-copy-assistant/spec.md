# Module Spec 025: QuickCopyAssistant

Status: v0.8 accepted for Finder-compatible macOS overlay and startup lifecycle
Date: 2026-08-10
Parent capability: PC-025 / quick entry to UC2 self-contained copy
Upstream contracts: module 018 `DesktopCopyPreview` and `DesktopCopyResult`
Presentation host: module 027 `AssistantHost`
Payload collaborator: module 028 `TransferPayload` through module 026
Durable decision: `docs/architecture/adr/ADR-012-quick-copy-entry-and-ui-boundary.md`

## 1. Responsibility

QuickCopyAssistant exposes the existing one-project Desktop copy flow through a
small macOS entry surface launched for one explicit `.als` file.

```text
Finder Open With ALS Rescue
-> validate one local ALS launch request
-> show the compact assistant near the cursor
-> choose destination parent
-> call existing target suggestion and prepare_copy
-> show Play only for an executable preview
-> Play supplies the existing explicit write consent
-> call existing execute_copy
-> present complete, incomplete or unable outcome
```

It owns launch routing, compact-window lifecycle, QuickCopy job state, control
intent, short status facts and courier-specific animation selection. Module 027
owns reusable compact-shell rendering. It does not own ALS
reading, dependency interpretation, target naming, copy, rewrite, validation,
manifest, promotion or incomplete-copy policy.

## 2. Product Boundary

Version 0.1 supports:

```text
macOS first
Open With as an alternate handler for .als
one ALS per quick session
one destination-parent chooser
one prepare step and one explicit Play action
complete and incomplete successful copies
one generic unable state
```

Version 0.1 does not support:

```text
Finder selection watching
a persistent assistant at the screen edge
Finder Sync
Share extension
multiple selected ALS files
technical stage labels
percentage progress
mid-project cancellation
new copy or rewrite policy
```

The product says `create a self-contained copy`, never `move`, because source
projects and source audio remain untouched.

## 3. Launch Contract

`QuickCopyLaunchContext v0.1`:

```text
request_id
source_als_path
source_display_name
launch_source
```

`launch_source` is `macos_open_with` in v0.1. The contract keeps the entry
adapter replaceable so a later macOS Service or Windows shell entry can feed the
same compact UI without changing copy behavior.

The adapter accepts exactly one local regular file whose extension is `.als`
case-insensitively. It refuses directories, non-file URLs, non-ALS files and
multi-file launches. Refusal creates the `unable` presentation state and never
calls `prepare_copy` or `execute_copy`.

The launch path is local IPC data. It must not enter path-redacted diagnostics,
telemetry or exported support reports.

## 4. Window And Positioning

The quick surface is a second window in the same Tauri application, not a
second product or helper executable.

Version 0.2 window policy:

```text
separate label: quick-copy
approximately 300 x 250 logical pixels
character visual box no larger than 96 x 128 logical pixels
not resizable
no title bar or standard window decorations
opaque character and opaque speech/control surfaces
no visible rectangular background or window shadow
transparent pixels outside character, speech and control surfaces
compact native input footprint while the speech bubble is hidden
expanded native input footprint only while speech or queue details are visible
dragging the character body moves the complete quick surface
always visible above ordinary application windows while the session is active
does not steal keyboard focus when its native level is reasserted
joins ordinary Spaces and eligible full-screen application Spaces on macOS
uses the tested AppKit floating level only while the quick surface is active
uses accessory application policy in quick mode and restores regular policy for the main window
one quick window reused for later launch requests
```

The Tauri adapter reads the current cursor position, offsets the window so it
does not cover the pointer, and clamps the complete window to the available
work area of the active monitor. If positioning fails, it uses a stable corner
fallback. Multi-monitor and display-scale behavior require a manual macOS test.

A transparent desktop window remains a rectangular native mouse target even
when most of its pixels are invisible. Quick mode therefore uses two explicit
presentation footprints. The compact footprint tightly contains the character,
controls, arrival animation and context menu. The expanded footprint is used
only while the speech bubble or queue details are visible. Resizing preserves
the bottom-right character anchor and clamps expansion to the active monitor,
so the character does not jump and the hidden area does not continuously block
the application below it. Leaving the quick surface dismisses hover-only speech
and restores the compact footprint. This is bounded-area reduction, not a claim
that individual transparent pixels are click-through.

The macOS adapter owns the Finder-compatible floating overlay policy. It combines accessory
application policy, all-Spaces/all-applications/full-screen-auxiliary collection
behavior, the AppKit floating window level and non-activating front ordering.
The screen-saver level is prohibited for the Courier because controlled native
and Tauri tests showed that Finder drag callbacks stop at that level. Floating
level with the same collection behavior remained visible above Safari and
accepted Finder drops, including after a full-screen Space transition.
The elevated level is an AppKit window-ordering mechanism, not permission to
interact with login, lock-screen or other security surfaces. It registers at
most one active-Space observer and reasserts the policy after a Space transition
without calling `set_focus`. Opening the main application restores regular
application policy. Other platforms keep a no-op platform adapter behind the
same boundary. The login window, screen lock and Mission Control remain explicit
exclusions from the always-visible product claim.

The existing main window must not flash during a cold quick launch. On macOS,
an opened-file event may arrive before Tauri setup completes. The entry adapter
therefore buffers startup Open URLs in arrival order, marks the runtime ready
during setup and drains the buffered requests before scheduling the normal main
window. Warm requests continue through the same ready-state route immediately. A normal
application launch still opens the main window. If the main application is
already running, the quick request is routed to the same process and does not
replace, resize or hide the main window.

## 5. QuickCopy Job State Machine

The QuickCopy job uses these states:

```text
destination_required
preparing
ready
working
complete
incomplete
unable
```

State transitions:

```text
valid launch -> destination_required
folder selected -> preparing
executable preview -> ready
blocked preview or adapter refusal -> unable
Play -> working
complete result -> complete
incomplete result -> incomplete
execution refusal/failure -> unable
change-folder action from ready -> destination_required
```

`ready` retains the exact `DesktopCopyPreview` returned by module 018. Play
passes that preview unchanged to `execute_copy` with `write_consent = true`.
The quick UI does not rebuild, weaken or reinterpret the preview.

Provider opening and native drag attempt states are not QuickCopy job phases.
They are composed from module 028 through module 026 and projected by the
QuickCopy presenter into the module-027 AssistantHost model.

An executable incomplete preview does not cause a second confirmation. The
single Play action is the explicit write consent. The final `incomplete` state
reports the omission count returned by module 018.

## 6. Controls

```text
destination_required
  Folder: choose destination parent
  Close: close without writing

preparing
  no write control
  Close: close because preview is read-only

ready
  Play: execute the retained preview
  Folder: choose another destination and prepare again
  Close: close without writing

working
  controls disabled until the one-project transaction finishes

complete / incomplete
  Open Folder: reveal the returned final target
  Close: finish the quick session

unable
  Close: finish the quick session
```

Controls use familiar icons with accessible labels and tooltips. The source
filename and selected destination are truncated visually without changing the
underlying path values.

## 7. Character And Animation Contract

The character is an original pixel-art worker inspired only by the supplied
mood reference. The reference image is not copied or shipped.

The character pixels are fully opaque and use normal colors. Transparent alpha
is permitted outside the silhouette inside the sprite asset. No rectangular
application surface is visible around the character, speech bubble or controls.
The character body is the drag handle for moving the complete assistant.

Animation uses small horizontal PNG sprite strips, one strip per state. CSS
`steps()` drives the loop; JavaScript does not advance individual frames.

```text
appearing
  3-4 frames, once
  small rise into position and one hat adjustment

destination_required
  2 frames, slow loop
  still posture, occasional blink and slight impatient foot movement

preparing
  3 frames, loop
  narrowed eyes and a small downward checking motion

ready
  2 frames, slow loop
  arms held close, one impatient eyebrow or foot movement

working
  4-6 frames, loop
  forward lean, short repeated hand movement and one sweat pixel
  no running, boxes, scenery or narrative action

complete
  3 frames once, then a 2-frame idle
  shoulders drop and forehead wipe

incomplete
  2-3 frames, short shrug, then hold

unable
  2-3 frames, one head shake, then arms folded
```

Recommended source frame size is fixed for every strip. Every state keeps the
same `96 x 128` logical-pixel interaction canvas. When source artwork has
different internal transparent margins, a state-specific CSS transform may
normalize the visible silhouette inside that canvas, anchored at the feet and
clipped to the fixed frame. The character must remain visually subordinate to
the speech bubble and controls. Rendering uses `image-rendering: pixelated`. A
reduced-motion preference shows a representative still frame instead of a loop.

Speech is deterministic and status-bound in v0.1:

```text
destination_required: "Ehh... gdzie to przenieść?"
preparing: "Sprawdzam."
ready: "To tutaj?"
working: "Już robię."
complete: "Gotowe."
incomplete: "Gotowe. Brakuje {omitted_asset_count} plików."
unable: "Nie mogę dla ciebie tego zrobić."
```

There are no random conversations, dialogue trees or technical stage names.
The speech bubble is a normal opaque UI element, not part of the sprite image.

## 8. Existing-Code Integration

The current application already owns the required copy behavior:

```text
suggest_target_project_root
prepare_copy -> DesktopCopyPreview
execute_copy -> DesktopCopyResult
```

QuickCopyAssistant calls those same Tauri/application-service boundaries. It
must not call `rescue_pipeline`, `rescue_rewriter` or filesystem copy APIs.

The React entry point chooses presentation by window label:

```text
main -> existing App
quick-copy -> QuickCopyAssistant
```

The Tauri launch adapter owns macOS `RunEvent::Opened`, file URL validation,
single-instance routing, quick-window creation/reuse and cursor-relative
placement. The adapter passes only `QuickCopyLaunchContext v0.1` to React.

`QuickCopyAssistant` remains the effect-owning courier composition. It supplies
an `AssistantPresentationModel v0.1` to module 027 and does not duplicate the
host shell, bubble, character or action-tray markup.

Tauri bundle configuration registers `.als` with role `Viewer` and rank
`Alternate`. ALS Rescue must not claim ownership of the Ableton file format or
replace Ableton as the default opener.

## 8A. State-To-Module Reuse Map

Animations bind to stable UI states, not to individual internal module events.
This keeps presentation unchanged when the pipeline is refactored.

| UI state | Public boundary | Existing modules reused internally | Animation |
| --- | --- | --- | --- |
| `destination_required` | Tauri launch adapter and folder dialog | no domain module | waiting |
| `preparing` | module 018 `prepare_copy` | module 017 orchestrates 004, 001, 002, 003, 005, 006, 008 and 009 plus plan fingerprint | checking |
| `ready` | retained `DesktopCopyPreview` | no module is running | impatient ready idle |
| `working` | module 018 `execute_copy` | module 017 recomputes the bound plan, then runs 010, 011, 012, 013 and 014 | working loop |
| `complete` | successful `DesktopCopyResult` | no module is running | relieved complete |
| `incomplete` | successful incomplete `DesktopCopyResult` | no module is running | short shrug |
| `unable` | adapter, preview or execution refusal | no additional module is started by the UI | head shake and folded arms |

Detailed preparing reuse:

```text
004 ProjectDiscovery
-> 001 ALSReader
-> 002 DependencyExtractor
-> 003 PathObservation
-> 005 DependencyAssessment
-> 006 PreflightReport
-> 008 CurrentPathBinding
-> 009 PackagePlanner and PlanFingerprint
```

Detailed working reuse after the plan is rebuilt and compared with the accepted
fingerprint:

```text
010 StagingExecutor
-> 011 ALSRewriter
-> 012 PackageValidator
-> 013 ManifestWriter
-> 014 PackagePromoter
```

QuickCopyAssistant receives only the final preview/result transitions. It does
not subscribe to or display the internal stage names above.

If another opened-file event arrives while the quick surface is `preparing` or
`working`, the active operation keeps ownership of the window and the new event
is ignored. v0.1 does not replace, cancel, overlap or queue a second project
during an active transaction.

## 9. Safety And Failure Presentation

Domain and filesystem errors remain structured internally even though the quick
surface collapses them into one user-facing `unable` state.

```text
no write before an executable DesktopCopyPreview
no write before Play
no second copy implementation
no silent target overwrite
no mutation of source ALS or source audio
no fake completion after a blocked or failed result
incomplete is shown only for a successful incomplete DesktopCopyResult
```

Closing during `working` is disabled in v0.1 because the existing one-project
transaction is not cancellable mid-run. The application must never terminate a
write-capable operation merely because the compact surface lost focus.

## 10. Acceptance

- Ableton remains the default opener after ALS Rescue is installed.
- Open With on one ALS opens only the compact surface near the cursor.
- A normal application launch still opens the existing main window.
- A warm quick launch reuses the running process and quick window.
- The main and quick surfaces produce the same module-018 preview and result for
  the same source, destination and application profile.
- Folder, Play, Open Folder and Close availability follows the state contract.
- Play is unavailable for `unable` and before a preview is ready.
- An incomplete executable preview runs after one Play and reports its final
  omission count without another question.
- The original ALS and audio remain unchanged.
- Character animation is pixel-sharp, opaque, bounded and respects reduced
  motion; no rectangular surface is visible around the assistant.
- Dragging the character body moves the complete assistant across the desktop.
- The packaged Courier accepts repeated Finder drops after Safari activation,
  a full-screen Space transition and a completed handoff followed by `Nowe zlecenie`.
- The quick window uses AppKit floating level and never screen-saver level.
- Cold Open With preserves every startup URL until setup is ready and does not
  flash the normal main window.
- No technical stage label or invented percentage is displayed.

## 11. Gate Status

Product boundary: CONFIRMED

Architecture boundary: CONFIRMED

Animation direction: CONFIRMED

macOS API availability: CONFIRMED by Tauri 2.11.5 `RunEvent::Opened`, bundle
file-association schema, cursor-position API and official single-instance
plugin documentation.

Packaging behavior on the owner's installed macOS build: requires the first
implementation fixture run but does not require a new product decision.

## 12. Courier Collection v0.2 Amendment

`QuickCopyLaunchContext v0.2` still represents one explicit local ALS intake
event. Open With, a warm secondary-instance launch and an explicit Finder drop
may produce multiple ordered contexts. QuickCopy forwards every accepted
context to module 029 and renders module-030 state; it no longer owns a one-file
copy session.

```text
one or more accepted ALS contexts
-> module 029 ordered queue
-> module 030 immutable waves
-> unchanged module 024
```

The first intake opens the compact surface. Later intake appends to the active
collection even while processing. QuickCopy never mutates an active wave,
chooses batch targets, merges collection results or supplies native payload
paths. Play means explicit consent to process the currently queued work and any
later work accepted before the processing run drains.

The destination is a saved local preference and may be changed before Play.
All Project outputs remain independent direct children of that parent. The old
one-project reducer remains historical v0.1 behavior and may be removed only
after the collection flow has equivalent tests.

```text
intake gate: CLEAR
module-024 reuse gate: CLEAR
dynamic-wave ownership: module 030
Finder drop adapter: Tauri boundary
source mutation: prohibited
```

## 13. Terminal Delivery And Fresh Order Correction

A fully successful handoff is terminal for the active order. Accepted ALS
intake after terminal delivery creates a fresh collection and becomes item one;
no prior project remains in the active list. Intake accepted while a handoff is
in progress is held for that next order and cannot mutate the frozen delivery
snapshot. Cancelled or failed handoff returns the package to the active order.

During the ready step the destination-folder action is hidden. The available
actions are the held native parcel, WeTransfer and configured local delivery.
The parcel disappears from the courier's hands when native dragging starts;
the platform drag image alone follows the pointer. A successful drop closes the
order and displays `Dostarczone pod drzwi`. Successful local delivery displays
`Gotowe. Zlecenie wyslane do folderu Google Drive.` Both outcomes hide the
parcel and delivery controls.

The context action `Wyslij ponownie` explicitly reopens the same immutable
ready collection without reprocessing ALS files. Opening WeTransfer alone is
never a handoff and never completes the order.

`Zamknij kuriera` ends an idle quick session, including queued work that has not
been started, and clears its in-memory collection before the window closes. A
later Open With or Finder drop therefore starts with item one. Closing while a
copy wave or local handoff is actively running may hide the window but must not
discard that operation; reopening the courier resumes the same active state.

## 14. Finder-Compatible Overlay And Startup Lifecycle v0.8

The macOS Courier uses `NSFloatingWindowLevel`, not
`NSScreenSaverWindowLevel`. The rest of the tested presence contract remains
unchanged: Accessory activation, `CanJoinAllSpaces`, `FullScreenAuxiliary`,
`CanJoinAllApplications`, `Stationary`, `IgnoresCycle`,
`hidesOnDeactivate(false)` and `orderFrontRegardless()`.

Tauri/Wry remains the sole inbound Finder-drop destination. No collection
reset, outbound handoff or presentation transition may register, unregister or
rearm an AppKit drop destination.

Cold macOS Open URLs are buffered behind a startup-ready boundary and drained
before normal-main-window scheduling. This lifecycle hardening does not alter
`QuickCopyLaunchContext`, ALS validation, queue ordering or copy behavior.

Acceptance requires a packaged macOS run, because unit tests cannot prove
WindowServer and Finder drag routing. The runtime matrix covers cold Open With,
repeated Finder drops, Safari activation, Safari full screen, `Nowe zlecenie`,
outbound parcel dragging and restoration of Regular activation for the main
window.

Before processing completes, the compact queue summary reports the number of
requested projects. Once the collection is ready, it reports packaged results
against requested work, for example `2 z 4 projektów`. Blocked and failed jobs
remain visible in the expanded list but never inflate the parcel count.

The complete Play button surface is interactive. The quick window reapplies its
full-screen overlay policy whenever it is shown and after an active-Space
transition. On macOS it joins eligible full-screen spaces as an accessory
overlay, accepts the first inactive mouse interaction and is ordered to the
front without an explicit focus request. The Tauri-created object remains a
standard `NSWindow`; the adapter must not apply the `NonactivatingPanel` style,
which AppKit defines only for `NSPanel` and its subclasses. This preserves the
standard WebKit/AppKit input path so an ALS can still be dropped onto the
always-on-top courier.

## 14. Repeatable Finder Intake Ownership

Finder intake and parcel handoff are separate native responsibilities. Tauri/Wry
is the sole owner of inbound Finder file-drop registration and delivery. The
outbound AppKit parcel surface may exist only while a ready collection is armed
and must not register, unregister or otherwise reset the inbound destination.

Application code must not add an AppKit inbound overlay and must not call
`registerForDraggedTypes` or `unregisterDraggedTypes` on Wry's parent or WebView.
Registering a view is not equivalent to implementing Wry's drag-destination
callback and can create a competing destination that consumes a Finder drag
without forwarding paths to Tauri.

The Tauri/Wry window event passes every dropped path through the same regular
file `.als` validation used by Open With, then forwards accepted paths to the
existing courier intake command. It owns no queue, copy, rewrite or handoff
policy. `Nowe zlecenie` resets collection and outbound-payload state only; it
must leave the framework-owned inbound registration untouched.
