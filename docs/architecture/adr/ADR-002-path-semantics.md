# ADR-002: Raw ALS Paths Are Preserved And Parsed Separately

Status: accepted  
Date: 2026-06-30

## Context

ALS files may contain paths created on macOS, Windows, network shares, external
volumes, project-relative locations or bare filenames. The host operating system
must not be allowed to reinterpret foreign path text too early.

Earlier hardening showed that using host `Path` semantics for ALS path text can
produce incorrect filename/extension behavior for Windows-style paths on macOS.

## Decision

The core keeps raw ALS path fields as text and adds a separate path parsing
layer:

```text
RawAlsPath
ParsedAlsPath
AlsPathKind
parse_als_path
```

Path parsing may classify text shape, derive display filename/extension and
preserve a normalized display string. It must not check disk existence, search
for files, resolve symlinks or decide source category.

## Consequences

`ALSReader` remains a read-only ALS fact reader.

`DependencyExtractor` remains a normalizer.

`PathObservation` is the first module allowed to combine dependency path text
with local filesystem metadata. It returns observations, not a selected or
verified asset identity.

Windows and macOS support can grow through filesystem adapters without changing
raw ALS facts already captured by upstream modules.
