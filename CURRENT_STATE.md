# Current State

Status: Windows x64 Alpha implementation is ready for final native CI and installer packaging

Date: 2026-08-03

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
choose one ALS -> analyze project -> show dependency summary and audio table
-> optionally copy a redacted diagnostic report
-> choose a destination -> review a read-only complete/incomplete copy preview
-> explicitly confirm -> create and validate a fresh Project copy
```

The UI calls application services; it does not parse ALS, inspect paths, decide
completeness, copy files or rewrite references itself. The current-path copy
uses only exact files that still exist at locations observed from the ALS. It
does not search for moved samples. Missing references remain unchanged and are
recorded as omissions in the package manifest.

As of ADR-007, this desktop flow no longer runs AssetInventory or computes
SHA-256 for audio. `CurrentPathBindingResult v0.1` records current path and
observed size without claiming historical content identity. Audio is read once
for the required copy, with stable source metadata and counted byte-size checks.
Validation and promotion check exact tree, regular-file type and size without
reading audio bytes again. ALS, portable manifest and private ledger retain
their small-artifact hash checks. Persistent audio hashes remain deferred to
the future SQLite index.

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

## 8. Next Decision

Before merging this branch or widening rewrite support:

1. use the updated desktop bundle to generate a fresh Type 3 project copy and
   open it manually in Ableton Live 11.3 with no missing media;
2. review generated manifests and diagnostic reports from those manual runs;
3. install the private Windows x64 Alpha artifact using
   `docs/setup/WINDOWS_ALPHA_INSTALL_AND_TEST.md`, run its diagnostic checklist,
   and keep Live 9/10 rewrite disabled;
4. add persistent incremental indexing only after the one-project desktop copy
   flow is accepted end to end.

Read next:

```text
PRODUCT_SPINE.md
docs/experiments/overnight-safe-vertical-slice-2026-07-27.md
docs/architecture/traceability.md
the active module specification
```
