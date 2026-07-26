# Architecture Notes

Status: active architectural index  
Date: 2026-07-26

This directory keeps a small set of durable architecture artifacts for the
Ableton project rescue tool. It is intentionally lighter than the product spec:
the goal is to preserve decisions, module boundaries and traceability that code
and module contracts can later enforce.

## Current Core Pipeline

```text
ProjectDiscovery / explicit context
-> ALSReader
-> DependencyExtractor
-> PathObservation
-> DependencyAssessment
-> read-only project report
```

Later write path:

```text
AssetInventory
-> AssetResolution
-> PackagePlanner
-> StagingExecutor
-> optional supported ALSRewriter
-> Validator
-> PrivateLedger / PackageManifest
```

The first MVP ends at the read-only report.

Implemented modules:

```text
ALSReader
-> DependencyExtractor
PathParser
```

Blocked next contract:

```text
003 PathObservation waits for E-01 path/project-root evidence
```

## Permanent Invariants

```text
Original Ableton projects are read-only.
ALS rewrite can happen only on a copied ALS.
No write operation happens without a plan.
No success state exists without validation and manifest/audit evidence.
Raw ALS values are preserved; interpretation lives in downstream modules.
Wrong sample match is worse than failed match.
Path availability is not asset identity.
PathObservation never selects a candidate.
```

## Files In This Directory

```text
diagrams/001-core-pipeline.mmd
  Mermaid diagram of the current intended pipeline.

adr/ADR-001-no-original-rewrite.md
  Decision record for never mutating original user projects.

adr/ADR-002-path-semantics.md
  Decision record for preserving raw ALS paths and parsing them separately.

adr/ADR-003-asset-identity-model.md
  Decision record separating references, requirements, locations and content.

adr/ADR-004-path-observation-no-selection.md
  Decision record for observation-only filesystem checks.

traceability.md
  Product capability -> module -> contract/test trace.
```
