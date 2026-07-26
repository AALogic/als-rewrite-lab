# ADR-001: Original User Projects Are Read-Only

Status: accepted  
Date: 2026-06-30

## Context

The product exists to rescue and package Ableton projects. User projects may be
old, valuable and hard to recreate. A failed write to an original `.als` or a
destructive operation on source samples could destroy the only working copy.

## Decision

The core must treat original Ableton projects and original audio files as
read-only inputs.

Allowed:

```text
read ALS files
read referenced audio metadata
copy files into a staged target
rewrite a copied ALS inside a staged target
write manifest/audit artifacts for generated output
```

Forbidden:

```text
rewrite original ALS files
delete original audio files
delete original Ableton project folders
run global XML search/replace as rewrite strategy
mark a package successful before validation and manifest recording
```

## Consequences

Every future module that writes to disk must receive a plan object and must
write into a target/staging area. If a task needs to modify an original project,
the request conflicts with this ADR and must be redesigned.
