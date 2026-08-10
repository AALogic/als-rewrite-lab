# Module Spec 027: AssistantHost

Status: v0.3 accepted for integrated held-payload presentation
Date: 2026-08-05
Parent capability: PC-027 / reusable compact character presentation host
Supported use case: UC2 alternate compact entry presentation
Upstream contracts: module 025 QuickCopy presentation model
Durable decision: `docs/architecture/adr/ADR-014-assistant-host-and-transfer-payload.md`

## 1. Responsibility

AssistantHost renders the reusable compact character surface without owning the
work performed by the active capability.

```text
AssistantPresentationModel v0.1
-> speech and detail text
-> character animation class
-> character and payload accessibility labels
-> contextual action controls supplied as React children
-> separate window-move and payload-interaction surfaces
```

The first implementation hosts only QuickCopyAssistant's courier. It is an
internal reusable UI boundary, not a plugin system or character catalog.

## 2. Inputs And Outputs

`AssistantPresentationModel v0.1`:

```text
status_text
source_display_name
destination_display_path
animation_class
character_accessible_label
window_drag_enabled
payload_drag_enabled
```

AssistantHost returns React presentation only. It emits no domain result and
performs no filesystem, browser, Tauri IPC or module-018 operation.

## 3. Ownership

AssistantHost owns:

```text
compact shell markup
speech bubble markup
character visual box and refs
action-tray placement
close-control placement
data-tauri-drag-region presentation
accessible labels and reduced visual clutter
```

It does not own:

```text
copy job state or transitions
payload or drag-attempt state
provider identity or URL
folder chooser effects
Tauri invoke/listen effects
ALS, package, rewrite, validation or manifest policy
licensing or multiple-character selection
```

## 4. Presentation Rules

The existing 300 by 250 logical-pixel quick window, 96 by 128 character box,
opaque sprite, speech bubble and controls remain visually unchanged during the
refactor. AssistantHost accepts already-derived presentation facts and does not
derive product status itself.

The character body is a window drag region only when `window_drag_enabled` is
true. Payload interaction geometry is exposed separately through a supplied
ref so the native adapter does not require the host to know a folder path or
attempt identity.

## 5. Test Contract

Automated tests must prove:

```text
assistant_host_renders_supplied_status_without_domain_interpretation
assistant_host_window_drag_region_follows_model
assistant_host_keeps_payload_and_window_drag_geometry_separate
assistant_host_exposes_supplied_controls_without_owning_actions
```

Existing QuickCopy reducer tests remain the behavioral baseline. Rendering
tests may use static markup or component-level assertions and must not require
real filesystem or Tauri effects.

## 6. Safety And Privacy

AssistantHost receives display-safe text only. Native absolute paths may exist
in `title` attributes only when explicitly supplied by the current local UI;
they must never enter exported diagnostics, telemetry or remote content.

The host cannot write, upload, copy, move, delete or rewrite user data.

## 7. Gate Status

Product gate: CLEAR

Architecture gate: CLEAR

Behavior gate: CLEAR, behavior-preserving refactor

Safety gate: CLEAR

Future character catalog and licensing: explicitly outside module scope

## 8. Courier Presentation v0.2 Amendment

`AssistantPresentationModel v0.2` projects the separate queue, work and payload
facts required by Courier Collection. AssistantHost renders:

```text
fast van arrival inside the bounded transparent surface
hover-only speech bubble
stable-size courier on a 96 by 128 pixel character canvas
stacked Folder and Play blocks in collecting state
ordered queue count and expandable bounded list
working box-handling loop while module 030 is processing
tired ready state, parcel and delivery controls
parcel-only outbound interaction surface
small context menu on double-click or secondary click
```

The queue list can remove only items that the presentation model marks
removable. AssistantHost reports intents to its owner and performs no queue,
settings, browser, filesystem or payload effect. A visible permanent close
button is removed; the context menu exposes Close and Reload/New task.

The courier bounding box, baseline and apparent scale remain stable across
collecting, ready and parcel states. Perspective scaling is permitted only
inside the bounded working animation. Reduced-motion mode uses stable key
frames and does not hide work status.

The ready parcel is painted inside the courier sprite at hand height; it is not
a loose foreground control. The separate hit surface is transparent. When the
native drag begins, the host switches to the empty-hands sprite because module
031 owns the parcel image under the pointer.
Folder and Play controls expose their complete stable button rectangles as hit
targets. The arrival remains short but visible long enough to read as a van
entering the desktop.

## 9. Integrated Held Payload v0.3 Amendment

The integrated held-payload visual is one 96 by 128 pixel sprite-strip composition:
the ready courier and parcel are rendered together,
not two independently animated visible elements. The existing
`share-armed.png` strip is the authoritative held-payload visual. A transparent
48 by 48 payload hit surface remains separate from the character window-drag
surface and aligns with the parcel painted inside that strip.

`AssistantPresentationModel v0.3` renames the misleading `parcel_visible` fact
to `payload_hit_surface_visible`. The fact controls native pointer geometry
only; it must never cause a second parcel image to be painted by React.

The state-to-visual projection is deterministic:

```text
ready + idle/arming/armed -> share-armed.png + transparent payload hit surface
native dragging -> ready.png empty-hands loop + native parcel under pointer
cancelled/failed drag -> share-armed.png restored
completed handoff -> complete.png with no payload hit surface
local handoff in progress -> preparing.png with no native payload surface
```

Every strip uses 96 by 128 logical-pixel frames. Payload transitions may change
the pose but never the character canvas, CSS scale, stage baseline or worker
area. The native adapter remains the sole owner of the parcel image under the
pointer and module 027 remains incapable of filesystem or handoff effects.
