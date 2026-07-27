# Project Map

Status: active responsibility map

Date: 2026-07-27

## Purpose

This file answers one question:

```text
Which part of the system owns each decision and side effect?
```

Product direction lives in `PRODUCT_SPINE.md`. Current progress lives in
`CURRENT_STATE.md`. Exact behavior lives in module specifications.

## Platform Stance

```text
macOS is the current hands-on laboratory platform
core contracts use native PathBuf values rather than string concatenation
foreign Windows path text is preserved and classified without pretending it is native
platform-specific filesystem behavior stays behind narrow modules
quality CI targets macOS and Windows
Windows support is not claimed until native CI and laboratory evidence pass
```

## Current Flow

```text
discovery/read
-> extract reference occurrences
-> observe paths
-> assess required assets
-> report
-> inventory selected scopes
-> resolve candidates
-> plan
-> stage copies
-> rewrite copied ALS
-> validate
-> write evidence
-> promote fresh package
```

## Ownership

### Product Spine

Owns product promise, use cases, MVP boundary, and gates from vision to module.
It does not own code details.

### Specifications And Machine Contracts

Each `specs/<module>/` directory owns responsibility, inputs, outputs,
refusals, safety policy, fixtures, acceptance tests, implementation plan, and
machine-checkable obligations for one module.

### `rescue_core`

Owns:

```text
ALS gzip/XML loading
raw document and active-reference facts
one-to-one dependency occurrence extraction
raw path text classification
read-only path observations
```

Does not own asset identity, search, package decisions, copying, or rewrite.

### `rescue_analyzer`

Owns:

```text
conservative Project root discovery
grouping reference occurrences into RequiredAsset records
dependency availability assessment
read-only preflight reports
```

Does not search globally, select files, or write.

### `rescue_catalog`

Owns bounded filesystem inventory, stable full-file SHA-256, content records,
and distinct file occurrences. It does not decide which occurrence satisfies a
project requirement.

### `rescue_resolution`

Owns candidate evidence, scoring, ambiguity policy, and resolution decisions.
It is pure and does not touch files. A partial inventory cannot produce an
automatic match. Scores rank candidates; without expected content identity or
an explicit user decision, a candidate remains unselected.

### `rescue_packaging`

Owns immutable copy and rewrite plans, target collision checks, ruleset gates,
exact operation preconditions, and rejection of operation-free laboratory
plans. It performs no filesystem writes.

### `rescue_execution`

Owns creation of fresh staging and byte-for-byte execution of approved copy
operations. It verifies source and target hashes and never promotes staging.

### `rescue_rewriter`

Owns only snapshot-bound, allowlisted edits to the staged ALS. It verifies the
source hash, locator, old values, ruleset, gzip, and XML. It does not plan,
resolve, copy audio, validate the full package, or touch the original ALS.

### `rescue_validation`

Owns independent complete-tree verification and semantic ALS diff. It is
read-only and decides whether staging matches the plan and rewrite allowlist.

### `rescue_manifest`

Owns:

```text
private ledger with full local audit evidence outside the package
portable manifest with relative, privacy-safe evidence inside the package
atomic no-clobber manifest writes
```

### `rescue_promotion`

Owns the final same-filesystem move from validated staging to an absent target.
It re-verifies all inputs and files. It does not merge, overwrite, or clean.

### `rescue_pipeline`

Owns ordering and fail-closed handoff of the modules above for one laboratory
run. It requires a confirmed source Project root and keeps every output outside
that root after resolving filesystem aliases and platform case behavior. It
retains completed discovery evidence when that boundary blocks a run and
contains no duplicate ALS parsing, matching, planning, or rewrite policy.

### CLI

Owns argument parsing, explicit laboratory consent, JSON rendering, and exit
codes. It delegates all domain behavior.

### Future Desktop UI

May display reports, collect user choices, start plans, show progress, and
request validation. It must never parse or rewrite ALS directly.

### Future Persistent Index

Will own incremental scan state, project registry, content cache, verification
timestamps, reachability, and later redirect history. It does not exist yet.

## One-Way Dependency Rule

```text
UI / CLI
-> orchestration
-> domain contracts
-> narrow filesystem/XML adapters
```

Low-level modules do not import CLI or UI. Pure decision modules do not access
the filesystem. Modules communicate only through public typed contracts.

## Forbidden Placements

```text
rewrite logic in UI, CLI, ALSReader, or matcher
matching policy inside filesystem scanning
filesystem access inside AssetResolution or PackagePlanner
copy decisions inside StagingExecutor
promotion before PackageValidator and ManifestWriter
delete or cleanup logic anywhere in the current MVP
platform path guessing through manual string concatenation
domain safety rules that exist only in chat or prose
```

## Open Boundaries

```text
explicit user-confirmed candidate selection contract
persistent index and cache schema
progress/cancellation and resumable batch orchestration
native Windows path and promotion adapters
plugin, preset, Pack, Core Library, and Max for Live dependencies
commercial release packaging and signing
```

## Update Rule

Update this file only when responsibility moves, a new domain is introduced,
the end-to-end flow changes, or a previously open boundary becomes a durable
decision.
