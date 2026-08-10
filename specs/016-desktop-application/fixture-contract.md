# Fixture Contract: 016 DesktopApplicationService

## Valid Fixture

A synthetic Live 11.3 gzip/XML ALS inside a copied Ableton Project directory
with one active audio reference and one existing audio file.

Expected: analysis completes, preflight is returned, diagnostic counts are
correct, and private filenames/paths are absent from serialized diagnostic
JSON.

## Failure Fixture

An absolute path to a nonexistent ALS.

Expected: `analysis_failed`, no preflight, structured error code, no writes.

## Safety Evidence

Hash the complete fixture tree before and after repeated analysis. Values must
be identical.

## Wire Evidence

Versioned, privacy-safe JSON fixtures cover an analysis result, copy preview,
execute request and copy result. Rust serialization must match the committed
fixtures, and the TypeScript contract test must accept the same payloads.
