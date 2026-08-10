# Current State

Status: Courier Collection V1 is implemented and verified on macOS

Date: 2026-08-06

Branch: `codex/windows-x64-alpha`

Purpose: short source of truth for what exists, what was proved, and what remains

## 1. Product In Plain Language

The product is a local Ableton project safety tool. It should find the audio a
Live Set depends on, explain whether those files still exist, locate reliable
candidates, and create a self-contained project copy without modifying the
original ALS or original media.

The central safety rule is:

```text
read facts -> observe files -> assess evidence -> plan -> stage a copy
-> rewrite only approved references -> validate -> record -> promote
```

No uncertain match may silently become a file operation.

## 2. What Exists On The Isolated Branch

The implemented one-project audio path is:

```text
001 ALSReader
002 DependencyExtractor
003 PathObservation
004 ProjectDiscovery
005 DependencyAssessment
006 PreflightReport
007 AssetInventory
008 AssetResolution
009 PackagePlanner
010 StagingExecutor
011 ALSRewriter
012 PackageValidator / SemanticDiff
013 PrivateLedger / PackageManifest
014 PackagePromoter
015 LaboratoryPipeline
016 DesktopApplicationService
017 CurrentPathCopyPipeline
018 DesktopCopyApplicationService
019 CompatibilityLab
020 ALSProjectScanner
021 ProjectCatalogBuilder
022 ProjectCatalogStore
023 ProjectCatalogApplicationService
024 BatchCopyApplicationService
```

The CLI now supports:

```text
rescue analyze <set.als>
rescue extract <set.als>
rescue preflight <set.als>
rescue lab-package <set.als> ... --laboratory-write
```

`lab-package` is intentionally explicit and experimental. It requires bounded
scan roots, fresh staging/final/ledger locations, and a separate write-consent
flag.

The desktop application exists under `apps/rescue-desktop`. It is a local Tauri
2 application with a React/TypeScript interface. Its current flow is:

```text
load or refresh the private Project catalog
-> select one or many concrete Live Sets, or add ALS files manually
-> one Set: use the existing analysis and copy flow
-> many Sets: choose one destination and review every preview before any write
-> explicitly confirm -> create and validate fresh Project copies sequentially
-> show per-project results plus one aggregate, path-redacted report
```

The UI calls application services; it does not parse ALS, inspect paths, decide
completeness, copy files or rewrite references itself. The current-path copy
uses only exact files that still exist at locations observed from the ALS. It
does not search for moved samples. Missing references remain unchanged and are
recorded as omissions in the package manifest.

The Project catalog and batch foundation now includes modules 020 through 024.
`ALSProjectScanner` walks only explicit approved roots, observes regular `.als`
files and exact `Ableton Project Info` directories, reports partial or cancelled
coverage, never follows filesystem links and never opens ALS content.
`ProjectCatalogBuilder` then performs a pure transformation into physical
ProjectFolder and LiveSet records. It retains multiple main Sets, separates
exact Backup children, preserves ungrouped or ambiguous Sets and never invents
a primary Set or version family. `ProjectCatalogStore` persists those records in
a private versioned JSON state, preserves prior evidence after partial scans,
marks complete-scan absence as stale rather than deleting history, and writes
atomically on macOS and Windows x64.

`ProjectCatalogApplicationService` exposes only path-redacted display records,
resolves explicit catalog or manual selections into one shared
`ProjectSelection` contract and leaves grouping policy outside React. The
Desktop can select one or many concrete Live Sets. One selection enters the
existing one-project flow; many selections enter `BatchCopyApplicationService`.
Batch prepares every one-project preview before any write, blocks target
collisions, executes ready jobs sequentially, isolates failures, supports
cancellation between projects and emits an aggregate path-redacted report.
The underlying one-project planning, rewrite, validation, manifest and promotion
policy remains unchanged.

As of ADR-007, this desktop flow no longer runs AssetInventory or computes
SHA-256 for audio. `CurrentPathBindingResult v0.1` records current path and
observed size without claiming historical content identity. Audio is read once
for the required copy, with stable source metadata and counted byte-size checks.
Validation and promotion check exact tree, regular-file type and size without
reading audio bytes again. ALS, portable manifest and private ledger retain
their small-artifact hash checks. Persistent audio hashes remain deferred to
the future SQLite index.

A separate compile-time `compatibility-lab` profile now exists for private
Windows evidence gathering. It does not change the strict Alpha policy. The
Lab can admit an unconfirmed Ableton document version only when every selected
active audio reference matches one of the already implemented structural
profiles, the user gives explicit experimental consent, and the ordinary
plan/stage/rewrite/validate/manifest/promote safety chain passes. Unknown usage
contexts, XML locators, path types and project-relative shapes still block
before staging. A manual Ableton outcome is required for any provisional
success/failure conclusion; the application never promotes a version to
supported automatically.

## 3. What The Vertical Slice Can Do

For one selected Live 11.3 ALS with supported external audio references, the
strict laboratory path can:

1. read and snapshot the ALS;
2. preserve one record per active audio reference;
3. observe recorded paths without treating existence as identity;
4. group references into required assets;
5. produce a read-only preflight report;
6. scan explicitly selected local folders and hash audio files;
7. rank candidates while requiring expected content identity or an explicit
   user decision before selection;
8. build an immutable copy/rewrite plan;
9. copy source ALS and selected audio into fresh staging with hash checks;
10. create and verify planned Ableton project directories, including
    Ableton Project Info;
11. rewrite only approved `Path`, `RelativePath`, and
    `RelativePathType` values in the staged ALS;
12. prove all other XML bytes and historical references stayed unchanged;
13. write a full private ledger outside the package and a redacted portable
    manifest inside it;
14. rename validated staging to an absent final target;
15. leave the result marked `ready_for_manual_ableton_check` only when every
    upstream safety contract is explicit.

The desktop current-path variant can also produce:

```text
complete_copy_ready_for_manual_check
incomplete_copy_ready_for_manual_check
```

An incomplete copy remains a valid output: available supported references are
copied and relinked, missing references are left unchanged, and even a project
with all audio missing can be copied as an ALS-only package with omissions.
Safety failures, stale previews, changed package plans and occupied targets
still block execution. Its PackagePlan v0.5 records `stable_source_and_size`, `None` for audio hash and
`None` for audio content identity. The manifest records
`content_identity_status = not_computed` instead of fabricating evidence.

The read-only preview now carries `PlanFingerprint v0.1`. Execution rebuilds
the plan, compares its canonical semantic fingerprint before staging, and
returns `preview_plan_changed` when audio appeared, disappeared, changed size,
or otherwise changed the approved copy/rewrite plan. Runtime IDs and ordering
do not affect the fingerprint. A same-path replacement with the same byte size
remains the explicit metadata-only MVP limitation.

AssetResolution policy v0.3 distinguishes the file currently bound at the
recorded active path from a moved recovery candidate. The former may be copied
as current project state without claiming historical identity. The latter
requires a `UserSelectionSet v0.1` bound to source ALS SHA-256, candidate native
path and candidate SHA-256. Changed or stale selections block before staging.

ALSReader v0.2.4 marks confirmed Live 11.3 `AudioClip` and `MultiSamplePart`
references as supported only for two bounded profiles. External Type 1 uses the
experimental Imported three-field rewrite. Safe project-local Type 3 under
`Samples/...` uses the confirmed-laboratory Path-only relocation. Other
versions, contexts, unsafe relative paths and path types remain `requires_test`
and do not authorize rewrite.

## 4. Historical Real Laboratory Evidence

On 2026-08-02, a copied six-reference Live 11.3 project completed the policy
v0.3 pipeline with five current recorded-path bindings and one explicit moved
candidate selection. Private paths and filenames remain outside Git.

```text
required audio assets: 6
copy operations: 7
rewrite operations: 6
validation: passed
promotion: promoted_ready_for_manual_check
source ALS and source audio hashes: unchanged
copied audio hashes: equal to selected sources
result ALS: valid gzip/XML, six Samples/Imported Type 3 references
manual Ableton open: failed, all six media files reported missing
```

The failed runtime check exposed a missing package invariant: the first
generated target omitted the empty Ableton Project Info directory. Static
file/hash/XML validation could not detect that semantic project-structure
defect. PackagePlan v0.3 now plans required directories explicitly,
StagingExecutionResult v0.2 records their creation, PackageValidationResult
v0.2 rejects a missing or symlinked marker, and PackageManifest v0.2 records
the verified directory structure. A newly generated package still requires a
manual Live check before the root cause is considered confirmed.

The corrected pipeline then generated a fresh v2 target without reusing the
failed target or staging directory:

```text
planned directories: 3
created directories: 3
verified directories: 3
Ableton Project Info: real directory, not a symlink
audio hashes: verified against PackageManifest v0.2
source ALS SHA-256: unchanged
validation: passed
promotion: promoted_ready_for_manual_check
manual Ableton open: passed, no missing media
```

The manual Live 11.3 check confirmed that the corrected v2 package opens
without missing files. Together with the failed package that differed by the
missing project marker, this confirms that the generated project must include
the planned and validated Ableton Project Info directory.

Before the fail-closed policy correction above, a copied one-reference ALS was
run through the complete pipeline while its real audio source remained
read-only. This proves the isolated mechanics, not that the former automatic
selection and rewrite authorization policy remains accepted.

```text
filesystem entries visited: 68,568
audio occurrences indexed: 31,502
unique content records: 8,108
selected match score: 100/100
copy operations: 2
rewrite operations: 1
pipeline errors: 0
final status: ready_for_manual_ableton_check
```

Independent checks confirmed:

```text
source ALS SHA-256 unchanged
source audio SHA-256 unchanged
copied audio SHA-256 equals source
result ALS passes gzip and XML parsing
result has one active relative Samples/Imported reference of Type 3
portable manifest contains no user-home absolute path
```

Ableton Live 11 accepted the generated ALS path through macOS `open`, and its
own log recorded that document argument. Accessibility control was unavailable,
so the UI, missing-media state, and audible project behavior remain manually
unverified. This is not counted as a successful Ableton runtime check.

Private ALS, media, paths, reports, and ledgers remain outside Git under the
dedicated laboratory directory.

## 5. Quality State

Pre-review branch verification baseline:

```text
cargo fmt --check: PASS
cargo test --workspace: PASS, 204 tests
cargo clippy --workspace --all-targets -- -D warnings: PASS
Python workflow-guard tests: PASS, 7 tests
workflow_guard verify-module 001 through 015: PASS
cargo audit: PASS, no known vulnerability reported
```

The current desktop slice now has this local verification:

```text
cargo fmt --check: PASS
cargo test --workspace --locked: PASS
cargo clippy --workspace --all-targets --locked -- -D warnings: PASS
Python tool and workflow-guard tests: PASS
workflow_guard verify-all: PASS
shared Rust/TypeScript desktop IPC fixture tests: PASS
TypeScript check, Vitest and Vite production build: PASS
Tauri release application build: PASS
private-path worktree guard: PASS
desktop and 390 px browser layouts: no horizontal overflow
```

The updated unsigned macOS application is available at:

```text
target/release/bundle/macos/ALS Rescue.app
```

The `.app` bundle and an unsigned local DMG exist for owner-only testing. They
are not distribution-ready release artifacts: signing, notarization and release
provenance remain incomplete.

The same repository and shared domain pipeline now contain a bounded Windows
x64 Alpha profile. Its target is Windows 11 x64 on a local NTFS volume; Windows
10 x64 remains a private legacy test host. The profile uses `ReplaceFileW` for
validated staged ALS replacement and no-clobber `MoveFileExW` promotion. It
rejects unsupported volume, reparse-point, UNC, cloud-folder and long-path
conditions instead of guessing. Windows path aliases are normalized only for
current-location binding, while ALS project-relative paths retain portable `/`
separators.

The desktop error view exposes a public error code and failing stage plus a
`Kopiuj raport błędu` action. The copied diagnostic includes application,
pipeline, commit, OS/architecture, status, stage, counters and all structured
errors, while tests require local project paths and media names to be redacted.
The GitHub workflow builds an unsigned, current-user, offline NSIS x64 installer
and a matching SHA-256 file. Final Alpha completion still requires a green
native Windows workflow at the latest branch commit and verification of the
downloaded private artifact.

The separate Compatibility Lab workflow passed natively on Windows at commit
`71b9fabe159a47e22ac81e3021e128c4296ca8ac` (GitHub Actions run
`30789593459`). It passed the all-features Rust workspace, frontend contract
tests and TypeScript check, then built and uploaded the unsigned offline NSIS
installer. The downloaded installer is stored outside Git at:

```text
../windows-installers/compatibility-lab-71b9fab/
  als-rescue-compatibility-lab-windows-x64-71b9fabe159a47e22ac81e3021e128c4296ca8ac/
```

The downloaded executable is 205 MB and its independently recomputed SHA-256
matches the CI sidecar:

```text
d1ffb5c0c3af19f3646a1a407a90b12df4c14ffbb315d4e30a6d0b7dbbc5b179
```

A local macOS bundle with the same compile-time profile was also rendered and
smoke-tested. It used the distinct name and identifier `ALS Rescue
Compatibility Lab` / `com.alsrescue.compatibility-lab`, showed the correct
profile label and opened/cancelled the native ALS picker. This is UI evidence,
not evidence that an older Live document rewrites correctly.

ADR-007 and the current contracts now remove all audio SHA-256 passes from the
normal desktop current-path flow. Static inspection confirms that
`current_path_copy` no longer calls `snapshot_asset_files`, `resolve_assets`, a
hasher or a regular-file hashing helper. The end-to-end test confirms that
audio hashes remain absent in PackagePlan v0.5, StagingExecutionResult v0.3,
PackageValidationResult v0.3, PackageManifest v0.3 and PackagePromotionResult
v0.2. The source ALS and the two small evidence artifacts remain hash-backed.

Known accepted MVP limit: replacing audio at the recorded path with different
bytes of exactly the same size before a run is not detected as historical
identity drift. The application packages the file currently bound at that
path, makes no historical identity claim, preserves originals and still
requires manual Ableton verification.

A first real current-path preview exposed a resolver defect in a Live 11.3 Set
with 3,744 active references and 30 required assets. Four otherwise valid
recorded-path files carried `OriginalFileSize = 0`. Resolution incorrectly
treated zero as an expected zero-byte file and blocked the plan. Policy v0.3
now treats zero as unavailable size evidence, matching the existing
PathObservation rule while preserving the raw ALS value. The same source-bound
preview now reports a complete plan with 29 unique audio copies, 3,744 rewrites,
zero omissions and no writes during preview.

The completed package was then compared read-only with its source and with its
post-Ableton-open state. All 29 copied audio hashes matched; the 651,645-element
XML trees differed only in the 3,744 authorized values of each of `Path`,
`RelativePath` and `RelativePathType`; historical and non-audio records were
identical. The comparison also exposed follow-ups: claim-group counts are
currently presented as asset counts, `.asd` sidecars are regenerated by
Ableton but lack a final product policy, and filesystem metadata handling is
implicit. Full evidence is in
`docs/experiments/real-project-source-vs-rescue-copy-2026-08-02.md`.

The Type 3 gap is implemented by PackagePlanner v0.5 and ruleset
`live11_3_current_paths_v0.2-lab`. Type 3 files preserve their existing
`Samples/...` placement and only the active absolute `Path` changes. Rewriter
and SemanticDiff consume the operation-specific changed-field allowlist.
Existing unsupported references now block before staging instead of creating
copied-but-unreferenced audio.

A read-only preview of a second large private Live 11.3 project produced 1,190
audio copies and 1,468 approved rewrites, including 1,467 Type 3 Path-only
operations and five multisample references, with zero omissions and zero
planning errors. No package was written during this verification. Full redacted
evidence is in
`docs/experiments/type3-current-path-relocation-2026-08-02.md`.

Desktop analysis adds five application-service tests for a valid gzip/XML project,
redacted diagnostics, structured failure, read-only behavior and determinism.
The React production build, TypeScript check, Tauri Rust compilation and module
016 workflow guard pass. Browser rendering was checked at desktop and 390 px
width without overflow or console errors.

The unsigned local macOS Alpha bundle was built successfully at:

```text
target/release/bundle/macos/ALS Rescue.app
```

The packaged native read-only application was then exercised through the macOS file
picker against the copied six-reference laboratory project. The screen reported
six required audio assets, five observed files and one file requiring search.
The copied diagnostic payload was valid JSON with six requirements and no
project filename, source path, candidate path, confirmed project root or user
home path. This proves the first real desktop read-only flow. The newly
connected desktop copy flow is covered by synthetic application and pipeline
integration tests; it still needs a fresh manual run against copied real
projects in Ableton.

The Tauri adapter now runs analysis, preview and execution services on blocking
workers while React keeps a visible elapsed-time activity state and prevents a
second operation. Rust serialization and versioned JSON fixtures are the source
of truth for the desktop IPC boundary; Rust roundtrip tests and frontend key
tests run in CI.

Dependency review after adding Tauri reported no npm vulnerabilities and no
RustSec vulnerability that is built into the current macOS target. RustSec
does report maintenance warnings in Tauri's transitive graph and one `glib`
unsoundness advisory in the Linux-only GTK path. `cargo tree -i glib` is empty
for the macOS target; Linux is outside the supported Alpha target and must not
be claimed without a separate dependency decision and build review.

The integrated test found and fixed one genuine contract drift: PathObservation
emitted `safe_for_metadata_read`, while AssetResolution expected an invented
`accepted` value. A cross-module regression test now enforces the actual v0.2
handoff.

A real scan exposed slow portable SHA-256 hashing. The global assembly feature
was removed because its locked backend does not compile on Windows; digest
behavior remains portable, and acceleration may return only behind a supported
target-specific configuration.

The repository now has a GitHub Actions quality workflow that runs the locked
Rust workspace on macOS and Windows and verifies the private-data policy and all
module contracts on Ubuntu. PR #1 passed the macOS, Windows, and workflow-contract
jobs at commit `c78102b71eebbfde3b2318284c23bd15c3b30834`. Windows CI exposed and
fixed one test portability defect: the private-ledger assertion compared a
native Windows path with JSON source text instead of parsing the JSON value.
Native compilation is now proved in CI, but real NTFS/Ableton behavior still
requires Windows laboratory evidence.

## 6. Hard Boundaries

Still enforced:

```text
original ALS and media are read-only
no recursive deletion or cleanup
no overwrite or merge into an existing target
no write before a ready immutable plan
no automatic choice from partial inventory
no automatic choice of a moved candidate without explicit user selection
current recorded-path binding is not presented as historical content identity
current-path desktop audio is not hashed before the persistent index exists
no automatic choice for tied or low-confidence candidates
no write when Project root discovery is unknown or outputs are inside that root
no write when an output parent resolves through an alias or case variant into that root
no laboratory write when the plan contains zero approved rewrite operations
no rewrite unless the ALS handoff explicitly marks the reference supported
no rewrite outside the supported ruleset and exact source snapshot
no write when the current semantic plan differs from the accepted preview
no promotion before independent validation and manifests
```

## 7. What Is Not Complete

The branch is not a shippable desktop product. Missing or intentionally blocked:

```text
manual installation and GUI run on the owner's Windows 11/10 machines
manual Ableton verification of a package created on physical NTFS
Windows code signing and SmartScreen reputation
full-disk discovery UX, cancellation, progress and persistent incremental index
fine-grained stage progress, cancellation and recovery after interruption
desktop user-selection UI for future moved-sample recovery
explicit count vocabulary for occurrences, claim groups and unique files
fresh manual Ableton verification of the Type 3 Path-only desktop package
explicit cross-platform copied-file metadata policy
batch processing and resume
plugin, preset, Max for Live, Factory Pack and cross-platform Core Library portability
Live 9/10/12 rewrite rules
hardened platform-specific no-replace directory promotion primitive
code signing, packaging, SBOM and release security review
```

The macOS desktop MVP now recognizes confirmed Ableton Core Library references
as `system_dependency`. The default package policy records them as
`leave_system_managed`, creates no audio copy or ALS rewrite, keeps them out of
the missing-file count and records `portable_risk` in the manifest. Validation
rejects any attempt to copy or rewrite a declared system dependency. The real
`OAKS_v4.als` preview classified 78 unique Core Library requirements (117
occurrences), planned 30 user-managed files and retained four genuine missing
requirements without a safety blocker.

This rule is intentionally narrow: it requires confirmed macOS Core Library
path evidence plus `RelativePathType = 5`. Factory Packs, Windows Core Library
paths and strict-portable collection remain research work described in
`docs/experiments/core-library-portability-grand-piano-2026-08-02.md`.

Automatic whole-computer audio discovery is also a consciously deferred product
stage. The intended experience does not require the user to select search
folders: after explicit permission, the application discovers readable local
and attached volumes, builds a private persistent index, refreshes it
incrementally and reports inaccessible areas as partial coverage. The first
production design should reuse `007 AssetInventory`, scan metadata before
content, hash only changed files or relevant match candidates, and provide
background progress, cancellation and resume. Continuous macOS FSEvents
monitoring is a later optimization, not a requirement for the first usable
index. This work starts only after the one-project Desktop recovery flow is
usable end to end.

The rewrite ruleset remains laboratory-only:

```text
Live 11.3.x
direct active SampleRef/FileRef
confirmed AudioClip or MultiSamplePart context
Type 1 -> Samples/Imported, change Path / RelativePath / RelativePathType
safe Type 3 under Samples/... -> preserve placement, change Path only
unsupported context, type or unsafe relative path blocks before staging
```

## 8. Active Next Work

The user accepted Project discovery, simple selection and sequential batch
execution. The complete implemented branch is:

```text
020 ALSProjectScanner
-> 021 ProjectCatalogBuilder
-> 022 ProjectCatalogStore
-> 023 ProjectCatalogApplicationService
-> 024 BatchCopyApplicationService
```

Modules 020 through 024 passed the normal spec, contract, guard, test,
implementation and review gates. The Desktop now exposes the path-redacted
Project list, explicit multi-selection, manual multi-ALS fallback and
sequential batch execution. This branch still does not parse every discovered
ALS, scan audio, compute audio hashes, search for missing samples or change the
current one-project copy policy.

A fresh macOS debug bundle was built on 2026-08-03 and opened successfully. It
loaded the private catalog and rendered 407 concrete Live Sets grouped by their
physical Project folders. Automated service tests prove multi-selection,
all-previews-before-write, collision handling, sequential execution, failure
isolation, cancellation boundaries and path-redacted diagnostics. On
2026-08-05 the owner reported a real macOS batch run with 11 jobs: 10 complete,
one incomplete, no blocked or failed jobs. The generated project copies opened
successfully in Ableton. This is field evidence for the macOS journey, while
the one incomplete result and Windows behavior remain separate review targets.

Module 025 QuickCopyAssistant is now implemented. A macOS `Open With` launch for
one explicit ALS opens a separate 300 by 250 pixel surface near the cursor,
while normal launch still opens the existing main application. The adapter
registers ALS Rescue as `Viewer` / `Alternate`, keeps Ableton as the default
handler and routes warm launches into one process through the single-instance
plugin.

The compact React surface uses a pure tested state machine and the unchanged
module-018 `suggest_target_project_root`, `prepare_copy` and `execute_copy`
boundaries. It contains no ALS parsing, copy or rewrite policy. An original
opaque pixel-art worker provides deterministic state animations and respects
reduced motion.

Packaged macOS evidence on 2026-08-05 includes normal launch, cold and warm ALS
launch, one-process reuse, complete and incomplete real copy runs, unchanged
source hashes/inventories and a generated complete project opened successfully
in Ableton Live 11. The local `.app` is ad-hoc signed for development and is not
yet a notarized commercial release.

Module 026 ExternalFolderHandoff is implemented for macOS. A packaged-app
regression run on 2026-08-05 confirmed that `W` arms the worker, shows the parcel
state and opens `https://wetransfer.com/` in the default browser. It accepts only
the latest successful module-018 result, opens a
closed `wetransfer_web` provider mapping in the system default browser and arms
one native AppKit directory drag with copy-only semantics. React cannot supply
an arbitrary provider URL or replace the backend candidate path. Missing,
stale, mismatched and symlink targets fail closed; dropped, cancelled and
failed drags all consume the session.

The first owner run exposed a real integration defect: beginning AppKit drag
after a React pointer event could lose the native mouse event, leave the worker
stationary and keep the session busy. The corrected adapter installs a native
input surface over the worker before opening the browser, accepts the browser-
focus first click and starts from its real AppKit `mouseDown`. New copy attempts,
new QuickCopy ALS launches and quick-window teardown now clear any stale session
and native surface, so a failed attempt cannot block the next `W` action.
The first corrected package then exposed an IPC naming defect before provider
opening: Tauri expected camel-case multi-word arguments while React sent
snake-case fields. Both handoff commands now declare `rename_all = "snake_case"`
and a source-level regression test locks that boundary.

QuickCopyAssistant now shows a pixel `W` control only after complete or
incomplete copy success. Its worker carries a parcel while armed, says
`Zabierz to ode mnie!`, disables ordinary window dragging during handoff and
restores the exact prior copy outcome after every terminal drag result. The
feature does not archive, upload, automate a browser, inspect a remote page or
claim remote transfer completion.

Automated evidence before the responsibility split included 22 Tauri unit tests, 21
frontend tests, TypeScript checking, frontend production build, full Rust
workspace tests and warning-free workspace Clippy. A corrected local arm64 DMG was
built at `target/release/bundle/dmg/ALS Rescue_0.1.0_aarch64.dmg`. The owner has
explicitly retained two manual tests: confirm WeTransfer's native drag-hover
treatment and confirm a downloaded transfer preserves the Project tree. Those
checks are not yet claimed as passed.

Controlled macOS and Windows runs with real Project folders remain valuable
parallel evidence. Resumable queues, background filesystem watching,
version-family inference, Finder selection watching and parallel copy remain
deferred until measurements justify them.

The active accepted work is now a behavior-preserving cleanup of the compact
assistant before any new character capability is added:

```text
025 QuickCopy job
-> 027 AssistantHost presentation
-> 028 TransferPayload private candidate and attempt lifecycle
-> 026 current WeTransfer/native-drag coordinator
```

ADR-014 owns this separation. The refactor must keep module 018, visible copy
behavior, the one current courier and the one current `wetransfer_web` action
unchanged. It advances the handoff request to v0.2 by removing the redundant
UI-supplied native target path. Provider catalogs, inbound ALS drop,
parcel-only drag presentation, delivered/reload UX, character catalogs,
multiple visible characters and licensing remain explicitly deferred until the
foundation passes regression, package and manual smoke verification.

The code-level split is now implemented. QuickCopy uses independent copy-job
and frontend payload reducers; AssistantHost receives one presentation model;
and the backend owns the private target path through TransferPayloadState.
`ExternalFolderHandoffRequest v0.2` carries only request, provider and copy-result
identities. Module guards 025-028, 24 Tauri tests, 26 frontend tests, TypeScript
checking, the frontend production build, full locked Rust workspace tests and
warning-free workspace Clippy pass. A fresh arm64 DMG was built and mounted;
its QuickCopy launch accepted `first new_3.als`, created a complete Project in a
fresh temporary root, emitted a manifest whose source ALS hash matched the
post-run source hash, opened WeTransfer and armed the courier without accepting
a React-nominated payload path. The run stopped before folder drop, so no
upload or remote-completion claim is made. This behavior-preserving cleanup is
accepted; provider drop verification and all deferred character capabilities
remain separate later work.

Manual macOS/Windows/Ableton compatibility runs remain valuable parallel
evidence but do not block the read-only Project catalog foundation. Rewrite
support for Live 9/10/12 remains unchanged and laboratory-only where already
specified.

Read next:

```text
PRODUCT_SPINE.md
docs/architecture/traceability.md
docs/architecture/adr/ADR-011-project-catalog-identity-and-boundaries.md
specs/023-project-catalog-application/spec.md
specs/024-batch-copy-application/spec.md
specs/025-quick-copy-assistant/spec.md
specs/026-external-folder-handoff/spec.md
docs/architecture/adr/ADR-013-external-folder-handoff-boundary.md
docs/architecture/adr/ADR-014-assistant-host-and-transfer-payload.md
specs/027-assistant-host/spec.md
specs/028-transfer-payload/spec.md
```

## 9. Active Accepted Increment: Courier Collection V1

The owner accepted the next compact-assistant increment on 2026-08-05.
ADR-015 is the durable decision. Implementation must preserve module 024 and
compose it through immutable waves:

```text
029 CourierWorkQueue
-> 030 CourierCollectionOrchestrator
-> unchanged 024 BatchCopyApplicationService
-> collection snapshot
-> 028 TransferPayload v0.2
-> 031 UniversalPayloadDrag
   or 032 CollectionDelivery
-> 027 AssistantHost v0.2 presentation
```

The user can add ALS files before or during processing. Files accepted during
an active wave are processed automatically in a later immutable wave. Files
added after the collection becomes ready return it to collecting and require a
new Play action. Successful outputs from every wave remain in one logical
collection. No physical batch wrapper directory is required in v0.1.

The first configured output parent is `$HOME/Downloads/sexy_testy`. The first
configured local delivery target is the owner's Google Drive Projects folder.
Both values are private local settings selected or seeded on the owner's Mac;
neither value may be hard-coded into source or exported diagnostics.

Modules 029 through 032 and the v0.2 updates to modules 025, 027 and 028 are now
implemented. Module 024 copy, rewrite, validation and naming policy remains the
single-project execution engine and was not duplicated in React or the Courier
coordinator.

The packaged macOS debug application was exercised on 2026-08-06 with real
copied project inputs. Evidence includes one complete single-project copy, one
generic unable outcome for an unsupported input, one two-project collection
processed by a single Play action, and one successful local delivery of both
result Project directories to the configured Google Drive filesystem folder.
The collection UI retained ordered queue display, bounded expansion/removal,
stable character scale, separate parcel drag geometry and context-menu close /
reload actions.

Automated acceptance passed the frontend TypeScript check, 25 Vitest tests,
frontend production build, full Rust workspace tests, Tauri library tests,
`cargo fmt --check`, and all 32 workflow-guard module contracts. The current
local ad-hoc-signed debug application is available at:

```text
target/debug/bundle/macos/ALS Rescue.app
```

`codesign --verify --deep --strict` passes for this local artifact. It is still
not Developer ID signed or notarized and must not be presented as a commercial
distribution build.

Two external-provider claims remain deliberately open. The owner still needs
to confirm that native parcel dragging shows WeTransfer's own drop affordance
and that a downloaded transfer preserves every Project directory. Until then,
the product claims only a copy-only native folder drag attempt, not a remote
upload result.

During a final run, the destination directory `$HOME/Downloads/sexy_testy`
stopped responding to ordinary directory enumeration. A process sample showed
the application waiting in the operating system's `hard_link` call while
writing a private ledger; Terminal directory listing blocked independently
after the application was stopped. macOS live APFS verification detected the
volume as needing repair and completed a deferred repair. This is recorded as a
host-filesystem incident rather than a Courier state-machine failure; the Mac
should be restarted before that exact directory is used again.

## 10. Terminal Courier Delivery

The courier delivery lifecycle was corrected on 2026-08-06 after owner testing
showed that the old `No. Zabrane.` presentation did not define whether a task
was still active, delivered or ready for another project. The durable behavior
is now owned by the backend collection orchestrator rather than inferred by the
React presentation.

Preparation and delivery are separate state axes. A ready immutable collection
can begin exactly one `native_drag` or `local_google_drive` handoff. A successful
handoff is terminal for the active order: the held parcel and delivery controls
disappear. An accepted native drop displays exactly `Dostarczone pod drzwi`.
Successful local Google Drive delivery displays
`Gotowe. Zlecenie wyslane do folderu Google Drive.` Partial or failed local
delivery remains retryable and retains the package.

After terminal delivery, a new ALS automatically starts a fresh collection with
one visible item; completed projects from the previous order are not carried
into that list. Intake arriving while a handoff is still in progress is held for
the next order and is promoted only after the current delivery succeeds. A
cancelled or failed handoff returns that deferred intake to the current order.
`Wyslij ponownie` explicitly reopens the same immutable completed package without
running ALS processing again. Opening WeTransfer alone never completes an order.

The parcel keeps a 48 by 48 logical-pixel hit surface while the character keeps
the same scale in every state. Folder and Play use complete 44 by 44 hit targets.
The van arrival duration is 1300 ms. The quick-window policy remains isolated in
its platform adapter and is reapplied whenever the window is shown; macOS uses
all-spaces and full-screen auxiliary behavior in addition to floating-window
level.

Automated acceptance passed the complete locked Rust workspace check, tests and
warning-free Clippy, 33 Tauri tests, 32 frontend tests, TypeScript checking,
frontend production build, formatting, semantic diff checks and module guards
025, 030, 031 and 032. The ad-hoc-signed debug bundle was rebuilt, strict
signature verification passed and the installed application opened a copied ALS
in the compact courier entry state. Automated macOS accessibility control cannot
reliably perform the final drag from the frameless transparent window, so one
packaged owner test remains: drop the parcel into Finder, confirm
`Dostarczone pod drzwi`, then add another ALS and confirm a fresh one-item order.

The current local ad-hoc-signed application is installed at:

```text
/Applications/ALS Rescue.app
```

Successful Finder/WeTransfer drop and visible layering over the owner's exact
full-screen Safari arrangement remain owner-observable compatibility checks.
The application reports the native operating-system drop result; it still never
claims that WeTransfer completed a remote upload.

## 11. Full-Screen Presence And Integrated Held Parcel

The quick courier presentation was tightened on 2026-08-06 without changing the
copy pipeline. In the ready state the courier and held parcel are now painted as
one stable 96 by 128 sprite. The separate 48 by 48 payload surface is invisible
hit geometry only. Native drag start switches the character to the empty-hands
sprite while AppKit owns the parcel image under the pointer. A cancelled or
failed drag restores the integrated held-parcel sprite; a successful handoff
keeps it absent.

The first packaged macOS attempt used status window level and passed its unit
contract, but a runtime full-screen test disproved it: the courier disappeared
after Safari entered its own full-screen Space. The corrected adapter switches
quick mode to accessory application policy, removes incompatible inherited
window behaviors, joins all eligible Spaces/applications, uses full-screen
auxiliary behavior and AppKit screen-saver window level, and restores regular
application policy when the main window is opened. Active-Space reassertion
never calls `set_focus`.

The rebuilt and ad-hoc-signed application passed the runtime check with Safari
frontmost, ALS Rescue inactive and the courier still on screen at Core Graphics
layer 1000. The evidence and the failed first attempt are recorded in
`docs/experiments/macos-courier-fullscreen-overlay-2026-08-06.md`.

The owner then found that an ALS dragged from Finder passed through the visible
overlay. The platform adapter had applied AppKit's `NonactivatingPanel` style to
the ordinary `NSWindow` created by Tauri, although Apple defines that style only
for `NSPanel` and its subclasses. The correction removes that unsupported style
while retaining accessory application policy, first-mouse acceptance, explicit
no-focus ordering, all-Spaces/full-screen behavior and screen-saver level. The
standard Wry/WebKit drag destination remains the only inbound file-drop adapter.

The correction passes 39 Tauri tests, warning-free Clippy, formatting, module
guard 025, the production frontend build and strict code-signature verification.
The packaged owner check must still confirm both facts together: the courier
remains visible above Safari and one copied ALS dropped from Finder becomes the
second queued item. The current local test application remains:

```text
/Applications/ALS Rescue.app
```

## 12. Idle Courier Session Close

Owner testing found that `Zamknij kuriera` destroyed only the quick window while
the process-level collection remained alive. Opening another ALS therefore
appended it to abandoned queued work. The close lifecycle now cancels an armed
native payload attempt, resets every idle collection in the backend, and only
then closes the window. Copy waves and local delivery already in progress are
preserved so closing the presentation cannot silently cancel filesystem work.

The correction is covered by frontend policy and orchestration tests, 41
frontend tests, TypeScript checking, the production frontend build and module
guard 025. A fresh ad-hoc-signed debug application was installed at
`/Applications/ALS Rescue.app`; closing an idle test order and reopening the
courier completed through the new lifecycle without retaining the prior window.

## 13. Requested Versus Packaged Project Count

A real four-project courier run demonstrated that the native parcel correctly
contained only the two projects whose copy jobs completed. One loose ALS was
blocked by `PIPELINE_PROJECT_ROOT_UNCONFIRMED`; another job was blocked by
`PIPELINE_OUTPUT_ALREADY_EXISTS`. The expanded list exposed those statuses, but
its compact `4 projekty` label incorrectly suggested that all four outputs were
inside the parcel. Ready collections now show packaged results against requested
work, for example `2 z 4 projektów`, while retaining every attempted item and
its status in the expanded list.

The correction passes 42 frontend tests, TypeScript checking, the production
frontend build and module guard 025. The refreshed ad-hoc-signed application is
installed at `/Applications/ALS Rescue.app`.

## 14. Repeatable Finder Intake After Native Handoff

Owner testing found a second-cycle macOS defect: the first Finder intake,
processing and native handoff succeeded, but after `Nowe zlecenie` the courier
could no longer accept another ALS drop. Logical collection reset was correct.
Removing consumed outbound parcel surfaces did not solve the defect. A later
attempt re-registered dragged types on both Wry hierarchy levels, but the
owner's exact runtime check disproved that hypothesis as well.

The corrected investigation combined Apple documentation, the exact installed
Tauri/Wry source and a live LLDB hierarchy inspection. AppKit delivers a drag
only to a registered object that also implements the destination callbacks.
Wry implements those callbacks on its nested `WryWebView`; its
`WryWebViewParent` is only a wrapper view. The custom adapter registered both
objects, which created a second, silent destination on the parent. Static tests
had verified the register/unregister sequence rather than proving that a second
Finder drop reached the Tauri event handler.

Tauri/Wry is now the sole owner of inbound Finder drop registration. The custom
AppKit inbound adapter and its lifecycle calls were removed. `Nowe zlecenie`
resets the collection and outbound payload only, while Tauri's window-wide drop
event continues to pass paths through the shared regular-file `.als` validator.
Outbound parcel dragging remains isolated in its existing AppKit source.

Architecture tests now reject custom inbound registration and require the
Tauri/Wry event to use the shared Finder route. The owner's exact handoff ->
`Nowe zlecenie` -> Finder drop remains the final packaged acceptance check and
must not be marked complete from static tests alone.

The correction passes 46 Tauri tests, 42 frontend tests, `cargo check`, Rust
formatting and all 32 module guards. A fresh ad-hoc-signed debug bundle is
installed at `/Applications/ALS Rescue.app` and passes strict signature
verification. Post-change LLDB inspection confirms empty dragged-type lists on
the Wry parent views while the nested Wry WebViews retain
`NSFilenamesPboardType` and their regular WebKit destination types.

## 15. Finder-Compatible Floating Overlay Migration

The isolated diagnostics Lab disproved the remaining screen-saver-level
hypothesis. AppKit screen-saver level kept the Courier visible but prevented
Tauri/Wry from receiving Finder drag-destination callbacks after the relevant
Safari and Space transitions. A native comparison matrix showed that an
ordinary Tauri window at AppKit floating level retained both overlay presence
and Finder drops. The Lab also reproduced a separate cold `Open With` timing
failure: macOS can deliver `RunEvent::Opened` before Tauri setup has installed
all managed state.

Production module 025 now uses a Finder-compatible floating overlay while
preserving accessory application policy, all-Spaces/full-screen auxiliary
behavior, no-focus ordering and restoration of Regular policy for the main
window. Tauri/Wry remains the sole inbound Finder-drop owner. A new narrow
startup buffer retains pre-setup Open With URLs and drains them in order before
the normal main-window schedule. Lab panic hooks, diagnostic loggers and native
drop probes were deliberately not migrated.

The migrated source passes the locked Rust workspace check, tests and
warning-free Clippy, Rust formatting, 48 desktop Rust tests, 42 frontend tests,
TypeScript checking, the frontend production build and module 025 readiness and
verification guards. A release macOS bundle was built, ad-hoc signed, strictly
verified and installed at `/Applications/ALS Rescue.app`; its executable hash
matches the tested release candidate.

Packaged runtime smoke tests confirm one cold Open With fixture in the visible
queue, warm append, deterministic Finder intake, continued Courier visibility
with full-screen Safari frontmost, successful Finder intake after that Safari
transition, successful intake after `Nowe zlecenie`, and unaffected ordinary
main-window launch. The stricter five-repeat matrices and complete outbound
parcel handoff remain open acceptance work. Evidence and rollback details are
recorded in
`docs/experiments/macos-courier-production-floating-migration-2026-08-10.md`.

## 16. Reproducible Source Milestone

The previously uncommitted accepted product state is now represented by an
honest Git checkpoint followed by a separate Finder-compatible macOS migration
commit. Existing history through `2add94a` remains unchanged. The immutable
recovery point is `v0.1.0-courier-macos-alpha` in the private canonical GitHub
repository.

Exact clone, verification and build commands live in
`docs/setup/BUILD_AND_RESTORE.md`. The commit split, limitations and checksums of
the off-repository source/ref/runtime backup are recorded in
`docs/history/HISTORY_RECONSTRUCTION_2026-08-10.md`.

The current tree passes the private-data guard. The explicit full-history audit
found 220 legacy violations in 37 reachable objects, so public or commercial
publication remains blocked until a deliberate history-sanitization migration
is completed and verified from a fresh full clone.

## 17. Windows CI Portability Correction

The first canonical Draft PR exposed two compile-time portability defects that
macOS could not detect. Windows drive-type constants were imported from the
file-system namespace although `windows-sys 0.61.2` defines them under
`Win32::System::WindowsProgramming`. The desktop run loop also matched Tauri
`Opened` and `Reopen` variants unconditionally even though those variants are
platform-gated by Tauri.

The Windows dependency now enables the exact additional Win32 feature, the
constants come from their generated namespace, and the macOS-only run-event
arms are explicitly target-gated. Domain, copy, rewrite and Courier contracts
are unchanged. GitHub Windows compilation is the acceptance test for this
platform boundary.
