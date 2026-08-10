# macOS Courier Full-Screen Overlay Runtime Test

Date: 2026-08-06
Host: macOS 15.5, Apple Silicon
Artifact: packaged and ad-hoc signed `ALS Rescue.app`

## Purpose

Verify the product requirement that the quick courier remains visible above an
unrelated full-screen application without taking keyboard focus when its native
presence policy is reasserted.

## Initial Result

The first implementation used `NSStatusWindowLevel` with
`CanJoinAllSpaces` and `FullScreenAuxiliary`. Unit tests passed, but the runtime
window disappeared from the on-screen Core Graphics window list after Safari
entered its full-screen Space. This disproved the status-level design.

## Corrected Policy

- quick mode uses `NSApplicationActivationPolicyAccessory`;
- the quick window uses `CanJoinAllSpaces`, `CanJoinAllApplications`,
  `FullScreenAuxiliary`, `Stationary` and `IgnoresCycle`;
- incompatible inherited collection flags are removed;
- the quick window uses `NSScreenSaverWindowLevel`;
- `hidesOnDeactivate` is false;
- the quick window uses a nonactivating style and accepts the first inactive
  mouse interaction;
- active-Space reassertion uses `orderFrontRegardless` and never calls
  `set_focus`;
- the main application restores regular activation policy.

## Verified Result

After activating full-screen Safari:

```text
FRONT=Safari
RESCUE_POLICY=1
ACTIVE=false
ALS Rescue layer=1000 bounds={Height=300, Width=360, X=725, Y=747}
Safari layer=0 bounds={Height=1080, Width=1728, X=0, Y=37}
```

The courier remained on screen at layer 1000 while Safari was the frontmost
application and ALS Rescue was inactive. The packaged application passed strict
code-signature verification before this test.

## Finder Drop Regression

The first corrected overlay remained visible above full-screen Safari, but its
standard activating-window behavior and an explicit `set_focus` call broke the
Finder-to-courier ALS drop workflow. The queue and ALS validation adapters were
unchanged and their tests still passed, which isolated the regression to the
native window boundary.

The follow-up correction adds the nonactivating style expected from a compact
macOS overlay, enables first-mouse interaction and removes the forced focus
after the window is shown. A packaged runtime Finder drop remains a mandatory
manual check because the available UI automation cannot release a Finder drag
over a different application's always-on-top window.

## Transparent Input Footprint

AppKit hit-tests a transparent window as a native rectangle; CSS transparency
does not make the desktop underneath that rectangle interactive. A full
pixel-shaped window or a global cursor monitor would add fragile native event
machinery and could regress Finder drag-and-drop.

The bounded solution is a two-size quick window. With hover speech hidden, a
compact 232 x 184 logical-pixel footprint contains the character, controls,
arrival sprite and context menu. Visible speech expands the same window to
360 x 300. The native adapter preserves the bottom-right character anchor and
clamps expansion to the monitor work area. Pointer leave hides hover-only
speech and restores the compact rectangle. Some transparent pixels remain, but
the large persistent blocker above and to the left of the courier is removed.
