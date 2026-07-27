# Current State

Status: laboratory vertical slice under fail-closed safety review on an isolated branch

Date: 2026-07-27

Branch: `codex/overnight-safe-vertical-slice`

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

## 3. What The Vertical Slice Can Do

For one selected Live 11.3 ALS with supported external audio references, it can:

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
10. rewrite only approved `Path`, `RelativePath`, and
    `RelativePathType` values in the staged ALS;
11. prove all other XML bytes and historical references stayed unchanged;
12. write a full private ledger outside the package and a redacted portable
    manifest inside it;
13. rename validated staging to an absent final target;
14. leave the result marked `ready_for_manual_ableton_check` only when every
    upstream safety contract is explicit.

The current composed request contract has no expected content hash or explicit
user-selection input, and ALSReader still reports active references as
`usage_context = unknown`, `is_rewrite_candidate = false`, and
`rewrite_support_status = requires_test`. AssetResolution policy v0.2 and
PackagePlanner therefore block the laboratory pipeline before staging. The
write modules remain implemented and isolated, but the composed pipeline no
longer turns path/name/size evidence or unknown rewrite context into writes.

## 4. Historical Real Laboratory Evidence

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
cargo test --workspace: PASS, 178 tests
cargo clippy --workspace --all-targets -- -D warnings: PASS
Python workflow-guard tests: PASS, 7 tests
workflow_guard verify-module 001 through 015: PASS
cargo audit: PASS, no known vulnerability reported
```

The integrated test found and fixed one genuine contract drift: PathObservation
emitted `safe_for_metadata_read`, while AssetResolution expected an invented
`accepted` value. A cross-module regression test now enforces the actual v0.2
handoff.

A real scan exposed slow portable SHA-256 hashing. The global assembly feature
was removed because its locked backend does not compile on Windows; digest
behavior remains portable, and acceleration may return only behind a supported
target-specific configuration. Windows builds still require CI validation.

## 6. Hard Boundaries

Still enforced:

```text
original ALS and media are read-only
no recursive deletion or cleanup
no overwrite or merge into an existing target
no write before a ready immutable plan
no automatic choice from partial inventory
no automatic choice without expected content identity or explicit user selection
no automatic choice for tied or low-confidence candidates
no write when Project root discovery is unknown or outputs are inside that root
no rewrite unless the ALS handoff explicitly marks the reference supported
no rewrite outside the supported ruleset and exact source snapshot
no promotion before independent validation and manifests
```

## 7. What Is Not Complete

The branch is not a shippable desktop product. Missing or intentionally blocked:

```text
manual confirmation that the generated real set opens cleanly in Ableton
native Windows build and filesystem tests
macOS/Windows desktop UI
full-disk discovery UX, cancellation, progress and persistent incremental index
explicit user-confirmed candidate selection
batch processing and resume
plugin, preset, Max for Live, Pack and Core Library portability
Live 9/10/12 rewrite rules
hardened platform-specific no-replace directory promotion primitive
code signing, packaging, SBOM and release security review
```

The rewrite rule remains laboratory-only:

```text
Live 11.3.x
direct active SampleRef/FileRef
old RelativePathType 1
destination Samples/Imported
new RelativePathType 3
only Path / RelativePath / RelativePathType may change
```

## 8. Next Decision

Before merging this branch or widening rewrite support:

1. manually inspect and play the generated real project in Ableton Live 11;
2. review the overnight report and code commits;
3. decide whether the next product slice is explicit user confirmation,
   persistent incremental indexing, or the first desktop workflow;
4. add macOS and Windows CI before claiming cross-platform support.

Read next:

```text
PRODUCT_SPINE.md
docs/experiments/overnight-safe-vertical-slice-2026-07-27.md
docs/architecture/traceability.md
the active module specification
```
