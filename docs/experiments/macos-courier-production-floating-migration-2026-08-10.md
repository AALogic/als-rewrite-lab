# macOS Courier Production Floating Migration

Date: 2026-08-10
Status: implemented and runtime-smoke-tested
Scope: production Courier source and locally installed macOS application

## Purpose

Migrate only the confirmed conclusions from the isolated Courier diagnostics
Lab into the production application:

- replace the screen-saver window level with AppKit floating level;
- preserve accessory application policy, all-Spaces and full-screen auxiliary
  behavior;
- keep Tauri/Wry as the sole inbound Finder-drop owner;
- buffer macOS Open With URLs received before Tauri setup is ready;
- exclude Lab-only panic hooks, drop loggers and native probes from the product.

## Product Changes

`quick_window_policy.rs` now applies `NSFloatingWindowLevel`. The remaining
production overlay policy stays unchanged and the normal main window still
restores Regular application policy.

`quick_startup_buffer.rs` owns the narrow startup lifecycle boundary. Opened
URLs received before setup are retained in order, then drained before the
normal main-window schedule begins. Ready-state requests continue through the
same routing path without buffering.

No custom AppKit Finder destination was added. Tauri/Wry remains the only
inbound file-drop destination.

## Automated Evidence

The following checks passed against the migrated production source:

- `cargo fmt --all --check`;
- `cargo check --workspace --locked`;
- `cargo test --workspace --locked`;
- `cargo clippy --workspace --all-targets --locked -- -D warnings`;
- frontend `npm test`: 42 tests passed;
- frontend `npm run check`;
- frontend `npm run build`;
- module 025 `module-ready` and `verify-module` workflow guards;
- release Tauri bundle build;
- strict deep code-signature verification of the ad-hoc-signed local bundle.

The desktop Rust package contains 48 passing tests after the startup-buffer
addition.

## Packaged Runtime Evidence

The release candidate was copied to `/Applications/ALS Rescue.app` only after
the previous installed application and impacted source files were backed up.
The installed executable SHA-256 matched the release candidate executable:

```text
e2659fe71e0a41e7184d318af97787f5e5e871f3cc1f3c77b37e1c7e6276db4b
```

Observed runtime checks:

1. A cold `Open With` launch using a copied ALS remained alive and the visible
   Courier queue contained exactly that fixture (`cold-3.als`).
2. A warm Open With request appended another ALS to the existing collection.
3. A deterministic Finder drag appended one copied ALS exactly once.
4. Safari was made frontmost in a full-screen window. Core Graphics still
   reported the Courier on screen at floating layer 3.
5. After returning to Finder, another deterministic ALS drag increased the
   visible queue from two to three projects exactly once.
6. `Nowe zlecenie` cleared the test collection. A subsequent Finder drag
   created a fresh one-item order containing `cold-1.als`.
7. A launch without an ALS opened the normal project-catalog window, confirming
   that Regular main-window behavior was retained.

The strict five-repeat cold-start, five-repeat post-Safari drop and complete
outbound parcel-handoff matrices remain open. They are intentionally not marked
complete from these smoke checks.

## Installation And Rollback

Installed application:

```text
/Applications/ALS Rescue.app
```

Pre-migration backup:

```text
$HOME/Documents/New project/migration-backups/courier-production-before-floating-2026-08-10
```

The backup contains the prior installed application, its ZIP archive and the
impacted pre-migration source/specification files. The ZIP SHA-256 is:

```text
6d57a04c673926348f57c1acea2271312d930ac1cd73b5db2765eea04db401fd
```

This build is ad-hoc signed for local testing. It is not Developer ID signed or
notarized and is not a commercial distribution artifact.
