# Session Digest: ALSReader Guard Retrofit

Date: 2026-06-09

## Summary

Added the same guarded workflow style to `001-als-reader` that already protects
`002-dependency-extractor`.

This was a retrospective hardening step. ALSReader already had behavioral Rust
tests, but it did not yet have a `module.contract.json`, so `workflow_guard`
could not verify its public contract, required tests, checked task obligations,
source boundaries and engineering rules.

## Added

- `specs/001-als-reader/module.contract.json`
- `crates/rescue_core/src/als_reader_impl.rs`

## Updated

- `crates/rescue_core/src/als_reader.rs`
- `crates/rescue_core/src/lib.rs`
- `specs/001-als-reader/spec.md`
- `specs/001-als-reader/plan.md`
- `specs/001-als-reader/fixture-contract.md`
- `specs/001-als-reader/tasks.md`
- `CURRENT_STATE.md`

## Refactor

`als_reader.rs` is now a small public API wrapper.

The private parsing implementation was moved into `als_reader_impl.rs`.

Why:

- keep public contract files small and guardable
- mirror the `002-dependency-extractor` structure
- avoid treating a large parser file as the public contract surface
- preserve behavior while improving maintainability

## Guarded Contract

The new contract checks:

- required docs exist
- required spec literals are present
- required tests exist and are not ignored
- public structs and field types match `ALSReadModel v0.2`
- forbidden output fields are absent, including `exists_on_disk`, `source_category`, `storage_state`, `resolved_path`, `match_candidates`
- source does not contain destructive filesystem operations
- dependency list stays inside the allowed set
- engineering rules are referenced
- task obligations are checked

## Verification

- `python3 tools/workflow_guard.py module-ready 001-als-reader`: PASS
- `python3 tools/workflow_guard.py verify-module 001-als-reader`: PASS
- `cargo fmt --check`: PASS
- `cargo check --workspace`: PASS
- `cargo test --workspace`: PASS, 24 tests passed
- `cargo clippy --workspace --all-targets -- -D warnings`: PASS
- `python3 tools/workflow_guard.py verify-module 002-dependency-extractor`: PASS

## Result

`001-als-reader` and `002-dependency-extractor` are now both covered by the same
module guard pattern.

The next module can be created using this pattern from the start instead of
retroactively.

