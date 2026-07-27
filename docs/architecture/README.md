# Architecture Notes

Status: active architectural index  
Date: 2026-07-27

This directory preserves durable module boundaries, safety decisions, and the
trace from product capabilities to executable contracts.

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
Partial inventory cannot authorize automatic matching.
Wrong sample match is worse than an unresolved sample.
No ready plan means no write-capable stage.
Rewrite applies only to a copied ALS and a snapshot-bound allowlist.
No validation and manifests means no final promotion.
Existing final targets are never intentionally overwritten or merged.
```

## Rewrite Evidence Boundary

The only implemented write profile is experimental and laboratory-only:

```text
Live 11.3.x external direct SampleRef/FileRef
RelativePathType 1 -> 3
destination Samples/Imported
allowlisted fields: Path, RelativePath, RelativePathType
```

Every unrelated XML byte and every historical reference must remain unchanged.
Unsupported versions, locators, reference types, collisions, or stale source
hashes block the operation.

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

traceability.md
  Product capability to module, contract, and test evidence.
```
