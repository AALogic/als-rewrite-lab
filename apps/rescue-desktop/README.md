# ALS Rescue Desktop

Local Tauri 2 + React/TypeScript adapter for the ALS Rescue Rust workspace.

## Development

```text
npm install
npm run check
npm run tauri dev
```

Use the Tauri command above for interactive development. Running the Rust
binary with `cargo run -p als-rescue-desktop` alone skips Tauri's frontend dev
command and may display assets embedded by an older build.

The first screen is the Project catalog. It can refresh a private read-only
catalog after explicit scan consent or accept one or more ALS files through
`Add ALS`. One selected Set enters the existing one-project analysis/copy flow.
Multiple selections enter a batch preview, ask for one destination parent and
then create independent Project folders sequentially after explicit write
consent.

Catalog refresh observes ALS paths and Project markers only. It does not parse
every discovered ALS, scan audio, compute audio hashes or search for missing
samples. Deep analysis starts only for explicit selections.

## Quick Copy

The same application also exposes a compact macOS `Open With` surface for one
explicit `.als`. ALS Rescue is registered as an alternate Viewer and does not
replace Ableton as the default opener. Normal launch opens the main Project
catalog; opening an ALS with ALS Rescue opens a separate 300 by 250 pixel window
near the cursor.

Quick Copy asks for one destination parent, prepares the unchanged one-project
preview and treats Play as explicit write consent. A successful complete copy
shows `Gotowe.`; a successful copy with omissions reports the returned omitted
count without asking a second question. Blocked or failed operations collapse
to one generic user message while retaining structured internal diagnostics.

The character and compact state machine are presentation only. They call the
same `suggest_target_project_root`, `prepare_copy` and `execute_copy` commands
as the main application and contain no ALS, copy, rewrite or completeness
policy.

After a successful complete or incomplete Quick Copy, the `W` control opens
the fixed WeTransfer URL in the system default browser. The backend binds one
opaque handoff session to that exact successful result. While armed, pressing
and dragging the worker starts an AppKit copy-only drag containing the final
Project directory; the worker window stays always-on-top and its ordinary
window-drag region is temporarily disabled.

ALS Rescue does not create a ZIP, upload data, automate the browser, inspect
the WeTransfer page or claim that a remote transfer succeeded. Drop,
cancellation and native failure all consume the one-shot session. A second
drag requires the user to choose `W` again. Live WeTransfer hover and downloaded
folder-tree behavior remain owner-run manual checks.

## Local macOS Alpha Bundle

```text
npm run tauri build -- --bundles dmg
```

The local development installer is written under
`target/release/bundle/dmg/ALS Rescue_0.1.0_aarch64.dmg`. It is not notarized
for public distribution.

## Architecture Boundary

The frontend selects input and renders results. It must not parse ALS files,
inspect dependency paths or make grouping, matching, copy or rewrite decisions.
Tauri owns native dialogs, platform scan roots, private store location and IPC.
It delegates catalog policy to `ProjectCatalogApplicationService`, one-project
analysis/copy to the existing application services and batch orchestration to
`BatchCopyApplicationService`.

Batch behavior is deliberately conservative:

```text
resolve explicit selections
-> prepare every one-project preview without writing
-> block target collisions
-> show aggregate and per-project readiness
-> require write consent
-> execute ready projects sequentially
-> preserve independent failed/blocked results
```

Cancellation is checked only between one-project transactions. Existing target
folders are never merged or silently renamed, and one failed project does not
erase already completed results or prevent later independent jobs.

## Verification Boundary

The catalog UI, shared IPC fixtures and batch orchestration are covered by Rust
and TypeScript tests. A fresh macOS debug bundle has also loaded the real private
catalog successfully. Before a release claim, perform one manual batch run on
macOS and one on Windows with disposable destinations, then open every generated
ALS in the matching Ableton version and retain the aggregate diagnostic report.
