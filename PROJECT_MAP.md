# Project Map

Status: active responsibility map

Date: 2026-08-05

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
approved scan roots
-> observe ALS files and Project markers
-> build physical ProjectFolder / LiveSet catalog
-> persist catalog freshness
-> create explicit ProjectSelection records
-> select one or many concrete Live Sets

discovery/read
-> extract reference occurrences
-> observe paths
-> assess required assets
-> report
-> create metadata-only bindings for files still present at recorded paths
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
confirmed source classification for supported dependency categories
read-only preflight reports
```

Does not search globally, select files, or write.

### `rescue_catalog`

Owns two separate catalog responsibilities behind separate contracts:

```text
lightweight Project catalog discovery
  bounded observation of ALS files and exact Project markers
  physical ProjectFolder / LiveSet / BackupSet catalog construction
  scan coverage, denied locations and deterministic ordering

audio asset inventory
  bounded filesystem inventory
  exact-file snapshotting and stable full-file SHA-256
  content records and distinct file occurrences
```

Project catalog discovery does not parse ALS, scan audio, infer version
families or select a Set. Audio inventory does not decide which occurrence
satisfies a project requirement.

### `rescue_resolution`

Owns candidate evidence, scoring, ambiguity policy, and resolution decisions.
It is pure and does not touch files. A partial inventory cannot produce an
automatic match. Scores rank candidates; without expected content identity or
an explicit user decision, a candidate remains unselected.

It also owns `CurrentPathBindingResult`, the smaller desktop-MVP contract that
binds exactly one safe recorded path and observed size without computing or
claiming content identity.

### `rescue_packaging`

Owns immutable copy and rewrite plans, their canonical semantic fingerprint,
target collision checks, ruleset gates, exact operation preconditions, and
rejection of operation-free laboratory plans. It performs no filesystem writes.

### `rescue_execution`

Owns creation of fresh staging and byte-for-byte execution of approved copy
operations. It enforces each operation's declared verification policy. Hash
backed files use SHA-256 and size; current-path audio uses stable source
metadata plus counted size and is read only once for the actual copy. It never
promotes staging.

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

Owns ordering and fail-closed handoff of the modules above. The strict
laboratory pipeline supports candidate recovery. The current-path pipeline
uses metadata-only bindings for files observed at recorded ALS paths and
permits an auditable incomplete copy without searching or hashing audio. Both
require a confirmed source Project root
and keep every output outside it after resolving aliases and platform case
behavior.
The current-path pipeline compares the rebuilt semantic plan with the accepted
preview fingerprint before the first write.

### CLI

Owns argument parsing, explicit laboratory consent, JSON rendering, and exit
codes. It delegates all domain behavior.

### Desktop Application Service

Owns user-workflow orchestration above the domain pipeline: read-only analysis,
source-bound copy preview, explicit write consent, desktop-facing results and
safe derivation of private staging paths. It contains no ALS parsing, matching,
copy, completeness or rewrite policy.

For the Project catalog increment it also owns:

```text
small desktop-facing Project list projections
manual Add ALS and catalog selection converging on ProjectSelection
sequential batch orchestration above the unchanged one-project services
per-project status isolation and aggregate reporting
```

### Desktop UI

May display reports, collect user choices, start plans, show progress, and
request validation. It must never parse or rewrite ALS directly.

The Project list UI receives display contracts and opaque identifiers. It does
not group paths, infer version families, choose a Set from a multi-Set folder,
or implement batch/package policy.

QuickCopy is a second workflow in the same application. Its copy-job state owns
only destination choice, preview, execution and the resulting complete or
incomplete outcome. It reuses DesktopCopyApplicationService and never
implements copy, rewrite, validation, completeness or target-naming policy.

AssistantHost renders the compact courier, speech, controls and separate
window-move / payload-interaction geometry from an explicit presentation
model. It owns no Tauri effects, provider policy, private native path or copy
state.

After a successful quick copy, TransferPayload privately binds the exact final
Project directory to the copy-result identity and issues one validated,
one-shot native attempt at a time. ExternalFolderHandoff coordinates the
current closed `wetransfer_web` route, browser opening and platform drag-source
port. Neither module uploads, automates the browser, moves files or interprets
remote completion.

CourierWorkQueue owns ordered, deduplicated ALS intake and pending-item
lifecycle. CourierCollectionOrchestrator freezes pending items into immutable
module-024 waves and merges successful output directories into a versioned
logical collection. It never mutates an active batch request and owns no copy,
rewrite or validation policy.

UniversalPayloadDrag owns platform-native copy-only dragging of one immutable
collection snapshot. CollectionDelivery owns plan-first, validated local copies
of that snapshot to a configured destination. Provider opening remains a
separate closed adapter. AssistantHost only projects these states visually.

### Tauri Adapter

Owns IPC serialization and scheduling synchronous application services on
blocking workers so filesystem and parsing work cannot freeze the webview. It
does not own domain policy. Versioned JSON fixtures verify its Rust/TypeScript
wire boundary.

For quick copy it also owns opened-file event translation, one-ALS launch
validation, single-instance routing, quick-window lifecycle and
cursor-relative placement. Platform entry adapters produce one shared launch
contract; they do not call the domain pipeline directly.

For Courier Collection it owns local file-drop translation, process-lifetime
queue state, background scheduling of immutable waves, local settings I/O and
platform effect adapters. Product ordering, collection revision and snapshot
rules remain typed application contracts rather than React state.

For external folder handoff it owns the system-default browser opener and
platform `FolderDragSourcePort`. The macOS adapter uses AppKit; React supplies
only an opaque copy-result identity and presentation geometry, never a native
path payload or arbitrary provider URL.

CollectionDelivery uses a separate narrow filesystem adapter. It may copy only
from backend-owned validated collection items into a configured local root and
must never modify, merge or delete a source Project copy.

### Persistent Local Stores

Project catalog persistence owns Project scan runs, physical catalog snapshots,
coverage and freshness. A partial scan may not mark unseen records missing or
delete prior history.

A later audio asset index will own content cache, verification timestamps,
reachability and redirect history. Persistent audio hashes and reusable
`ContentIdentity` are introduced there, not retrofitted into the metadata-only
current-path flow.

Storage schemas are private adapter details. Domain and UI modules consume
versioned contracts, not SQLite table shapes.

## One-Way Dependency Rule

```text
UI / CLI
-> Desktop Application Service / orchestration
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
Project catalog persistence schema and freshness migration
continuous filesystem watching and incremental refresh
resumable batch execution after the first sequential batch
native Windows path and promotion adapters
plugin, preset, Factory Pack, cross-platform Core Library, and Max for Live dependencies
commercial release packaging and signing
```

## Update Rule

Update this file only when responsibility moves, a new domain is introduced,
the end-to-end flow changes, or a previously open boundary becomes a durable
decision.
