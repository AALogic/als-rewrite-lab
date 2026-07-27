# Overnight Safe Vertical Slice Report

Date: 2026-07-27
Branch: `codex/overnight-safe-vertical-slice`
State: implementation and static validation complete; manual Ableton UI gate pending

## Objective

Build the largest evidence-backed, one-project audio recovery flow that can be
tested without modifying original user data. Work was isolated from the main
working tree. No branch was pushed or merged.

## Implemented Product Flow

```text
ProjectDiscovery
-> ALSReader
-> DependencyExtractor
-> PathObservation
-> DependencyAssessment
-> PreflightReport
-> AssetInventory
-> AssetResolution
-> PackagePlanner
-> StagingExecutor
-> ALSRewriter
-> PackageValidator / SemanticDiff
-> PrivateLedger / PackageManifest
-> PackagePromoter
-> LaboratoryPipeline
```

The CLI exposes this only as `lab-package` with explicit scan, staging,
target, ledger, and `--laboratory-write` arguments.

## Safety Design

The flow rejects the request or stops before writing when:

- source, scan, staging, target, or ledger paths are unsafe;
- staging or final target already exists;
- scan inventory is partial;
- a candidate is missing, tied, ambiguous, or below the automatic threshold;
- source snapshots or public handoff identities disagree;
- the ALS version, locator, old values, or rewrite rule is unsupported;
- copied bytes do not match the planned SHA-256 and size;
- any non-allowlisted XML byte or historical reference changes;
- a private ledger or portable manifest cannot be written safely;
- staging contains a symlink, missing file, unexpected file, or changed file.

Write modules cannot be reached before `PackagePlan` reports
`ready_for_laboratory_execution`.

## Synthetic End-To-End Evidence

Eight orchestration tests cover:

1. complete package promotion with byte-identical sources;
2. final absolute and relative ALS paths;
3. missing sample blocking before staging;
4. ambiguous same-name candidates blocking before staging;
5. existing-target rejection before any write;
6. rejection of an unbounded filesystem root;
7. unsupported Live version blocking before staging;
8. portable manifest redaction of laboratory absolute paths.

The integrated slice exposed a real contract drift hidden by isolated unit
fixtures:

```text
PathObservation output: safety_status = safe_for_metadata_read
old AssetResolution expectation: safety_status = accepted
```

The consumer now uses the actual v0.2 status, and a cross-module regression test
enforces that handoff.

## Real Private Run

A copied Live 11.3 one-reference ALS and an existing external audio source were
used. The original source paths are omitted from committed evidence.

```text
entries visited: 68,568
audio file occurrences: 31,502
unique content records: 8,108
inventory status: complete
selected candidate: unique, score 100
copy operations: 2
rewrite operations: 1
validation: passed
manifest write: passed
promotion: passed
pipeline errors: 0
elapsed release run: 135.94 seconds
```

Independent post-run checks:

```text
source ALS hash unchanged: PASS
source audio hash unchanged: PASS
target audio hash equals source: PASS
target ALS gzip validation: PASS
target ALS XML/reader validation: PASS
active rewritten reference count: 1
relative destination: Samples/Imported/<filename>
new RelativePathType: 3
portable manifest user-home path scan: PASS
```

Full reports, paths, ledgers, ALS files, and audio remain in the private
laboratory directory and are excluded from Git.

## Ableton Runtime Gate

macOS dispatched the generated ALS to Ableton Live 11, and Ableton's log
recorded that exact generated document argument. Computer Accessibility control
timed out, so the UI could not be inspected.

Therefore:

```text
launch dispatch: OBSERVED
clean set load: NOT CONFIRMED
missing media state: NOT CONFIRMED
playback: NOT CONFIRMED
user acceptance: NOT CONFIRMED
```

The rewrite profile remains experimental.

## Performance Finding

The first release run without the optional accelerated SHA-256 backend spent
91.24 seconds and was still hashing the first 134 MB audio source when stopped.
After enabling the RustCrypto `sha2` assembly feature, a partial 10,000-entry
run indexed 4,128 audio occurrences in 25.63 seconds, and the complete
68,568-entry run finished the entire pipeline in 135.94 seconds.

This was not a controlled microbenchmark, but it was sufficient to expose an
unacceptable backend choice. All hashing crates now inherit one workspace
`sha2` configuration. The successful package and independent system hash
confirmed digest compatibility. Windows build and performance CI remain
required.

## Verification Summary

```text
cargo fmt --check: PASS
cargo test --workspace: PASS (178 tests)
cargo clippy --workspace --all-targets -- -D warnings: PASS
workflow guard Python tests: PASS (7 tests)
verify-module 001..015: PASS
cargo audit: PASS (71 locked dependencies scanned)
cargo deny: not installed, not run
```

## Residual Risks

- The tested rewrite family is narrow: Live 11.3, direct active
  `SampleRef/FileRef`, external Type 1 to imported Type 3.
- A complete selected-root scan is required for automatic matching; persistent
  incremental indexing is not implemented.
- Explicit user confirmation for lower-confidence candidates is not wired into
  the pipeline.
- Package promotion still needs a hardened platform-specific no-replace
  directory primitive before commercial release.
- Native Windows compilation, path behavior, and filesystem operations have
  not yet run in CI.
- Plugins, presets, Packs, Core Library, Max for Live, batch operation, and UI
  are outside this slice.

## Current Review Routing

This report owns the dated experiment evidence only. `CURRENT_STATE.md` owns
the remaining blockers, merge decision, CI status, and next step.
