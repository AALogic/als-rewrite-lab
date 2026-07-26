# Session Digest: DependencyExtractor v0.1 Closeout

Date: 2026-06-09

## Summary

Implemented `002-dependency-extractor` as the first downstream core module after ALSReader.

The module converts `ALSReadModel v0.2` into `DependencyExtractionResult v0.1`.
It normalizes active audio references into dependency records, preserves raw ALS evidence,
and records ignored historical/non-audio inputs without taking responsibility for
filesystem verification, source classification, matching, planning, copying, rewrite, or validation.

## Code Added

- `crates/rescue_core/src/dependency_extractor.rs`
- `crates/rescue_core/src/dependency_extractor_impl.rs`
- `crates/rescue_core/tests/dependency_extractor.rs`

Updated:

- `crates/rescue_core/src/lib.rs`
- `specs/002-dependency-extractor/tasks.md`
- `CURRENT_STATE.md`

## Contract

Public API:

- `extract_dependencies(model: &ALSReadModel) -> DependencyExtractionResult`

Public data models:

- `DependencyExtractionResult`
- `DependencyExtractionMetadata`
- `DependencyRef`
- `IgnoredInputSummary`
- `DependencyExtractionWarning`
- `DependencyExtractionError`

Boundary:

- accepts only `ALSReadModel v0.2`
- produces `DependencyExtractionResult v0.1`
- does not read sample files
- does not check whether paths exist
- does not add downstream fields such as `resolved_path`, `existence_status`, `source_category`, `risk_flags`
- does not deduplicate active references
- keeps `OriginalCrc` as weak evidence only

## Verification

- `cargo fmt --check`: PASS
- `cargo check --workspace`: PASS
- `cargo test --workspace`: PASS, 24 tests passed
- `python3 tools/workflow_guard.py verify-module 002-dependency-extractor`: PASS
- `cargo clippy --workspace --all-targets -- -D warnings`: PASS

## What This Enables Next

The next natural module is `003-path-verifier`.

It should consume `DependencyExtractionResult` and check whether referenced audio files exist
at the paths described by ALS-derived dependency refs.

Important boundary for 003:

- it may inspect the filesystem
- it must not match missing samples globally
- it must not classify source type beyond path verification status unless explicitly specified
- it must not rewrite ALS or copy files
