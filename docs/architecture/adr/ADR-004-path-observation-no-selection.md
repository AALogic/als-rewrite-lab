# ADR-004: Path Checks Produce Observations, Not Selection

Status: accepted  
Date: 2026-07-26

## Context

The original 003 PathVerifier contract preferred a project-relative candidate,
then a raw path, and selected the first existing file. Existence and matching
size are useful evidence but do not prove that the file is the expected sample.
The contract also inferred Project root from the parent directory of the ALS.

## Decision

003 is redefined as PathObservation.

It:

```text
receives explicit application context
derives only allowed candidates
records every safe candidate
uses read-only metadata
distinguishes missing, inaccessible, foreign-platform, directory and symlink
never follows a symlink in its first version
never selects or resolves asset identity
```

ALSReader leaves its legacy `source_project_root` field null. Project root is
supplied by ProjectDiscovery, explicit user context or a controlled fixture.

## Consequences

- `selected_path_candidate` and `verified_exact_path` are forbidden outputs.
- Both existing candidates remain visible to downstream consumers.
- Size equality remains supporting evidence only.
- AssetResolution or an explicit user decision may later select an asset.
- E-01 must confirm path semantics before 003 implementation starts.
