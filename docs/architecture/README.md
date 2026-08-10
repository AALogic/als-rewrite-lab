# Architecture Notes

Status: active architectural index  
Date: 2026-08-05

This directory preserves durable module boundaries, safety decisions, and the
trace from product capabilities to executable contracts.

## Active Project Catalog Branch

```text
approved roots
-> ALSProjectScanner
-> ProjectCatalogBuilder
-> ProjectCatalogStore
-> ProjectCatalogApplicationService
-> explicit ProjectSelection
-> existing one-project application flow
-> BatchCopyApplicationService
```

The catalog branch keeps physical Project folders, concrete Live Sets, Backup
Sets and future logical version families separate. It does not parse every ALS,
index audio, or introduce a second package/rewrite policy.

## Current Laboratory Pipeline

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
```

`LaboratoryPipeline` composes those public contracts for one selected project.
It contains orchestration only; each domain rule remains owned by its module.

## Permanent Invariants

```text
Original Ableton projects and media are read-only.
Raw ALS facts, filesystem observations, identity evidence, and decisions remain separate.
Path availability is not asset identity.
Partial inventory cannot authorize automatic matching, and scores alone do not
replace expected content identity or an explicit user decision.
Wrong sample match is worse than an unresolved sample.
No ready plan means no write-capable stage.
Rewrite applies only to a copied ALS and a snapshot-bound allowlist.
No validation and manifests means no final promotion.
Existing final targets are never intentionally overwritten or merged.
```

## Rewrite Evidence Boundary

The implemented write profiles remain experimental and require manual Ableton
verification:

```text
Live 11.3.x external direct SampleRef/FileRef
RelativePathType 1 -> 3
destination Samples/Imported
allowlisted fields: Path, RelativePath, RelativePathType

Live 11.3.x safe project-local SampleRef/FileRef under Samples/...
RelativePathType 3 remains 3
preserve target-relative placement
allowlisted field: Path
```

Every unrelated XML byte and every historical reference must remain unchanged.
Unsupported versions, locators, reference types, collisions, or stale source
hashes block the operation. Desktop execution also blocks when the canonical
semantic fingerprint of the rebuilt package plan differs from the accepted
preview.

## Files In This Directory

```text
diagrams/001-core-pipeline.mmd
  Current read, decision, and laboratory write flow.

adr/ADR-001-no-original-rewrite.md
  Never mutate original user projects.

adr/ADR-002-path-semantics.md
  Preserve raw ALS path data and interpret it separately.

adr/ADR-003-asset-identity-model.md
  Separate references, requirements, locations, and content.

adr/ADR-004-path-observation-no-selection.md
  Filesystem observation does not select a sample.

adr/ADR-005-snapshot-bound-rewrite-evidence.md
  Rewrite requires exact source hash, locator, old values, and ruleset.

adr/ADR-006-desktop-application-boundary.md
  Desktop UI calls application services and does not own domain policy.

adr/ADR-007-current-path-copy-without-content-identity.md
  Current-path audio uses a metadata binding and stable copy, not a newly
  computed content identity.

adr/ADR-008-core-library-system-dependencies.md
  Confirmed narrow Core Library references are reported as system dependencies
  and left system-managed by the default desktop package policy.

adr/ADR-009-windows-x64-alpha-filesystem-boundary.md
  Windows Alpha uses bounded native write adapters for local NTFS and rejects
  unsupported path and filesystem environments.

adr/ADR-011-project-catalog-identity-and-boundaries.md
  Separates ProjectFolder, LiveSet, BackupSet and future ProjectWork identity,
  and keeps batch orchestration above the unchanged one-project pipeline.

adr/ADR-012-quick-copy-entry-and-ui-boundary.md
  Adds a compact Open With entry as a second presentation surface over the
  unchanged one-project Desktop copy service.

adr/ADR-013-external-folder-handoff-boundary.md
  Adds a trusted provider action and native copy-only folder drag above the
  latest successful Desktop copy result.

adr/ADR-014-assistant-host-and-transfer-payload.md
  Separates copy-job state, reusable character presentation, private payload
  ownership and provider-specific handoff coordination.

traceability.md
  Product capability to module, contract, and test evidence.
```
