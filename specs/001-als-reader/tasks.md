# Tasks 001: ALSReader

Status: v0.1 completed; v0.2 contract implemented as first pass  
Date: 2026-06-02  
Scope: small implementation tasks for ALSReader

Update note 2026-06-09:

```text
This file keeps the v0.1 task history.
The controlling module contract is now spec.md v0.2.
Any v0.1 task that mentions ALSAnalysis, ActiveFileRef or path existence checks
inside ALSReader is legacy and must not override ALSReadModel v0.2.
```

## 1. Fixture Contract

- [x] Decide whether local ALS fixtures are copied into `tests/fixtures/als/`.
- [x] If copied, copy only small `.als` files needed for tests.
- [x] Record expected counts for each fixture.
- [x] Add one invalid non-gzip fixture.

## 2. Rust Scaffold

- [x] Create root `Cargo.toml` workspace.
- [x] Create `crates/rescue_core/Cargo.toml`.
- [x] Create `crates/rescue_core/src/lib.rs`.
- [x] Create `cli/rescue-cli/Cargo.toml`.
- [x] Create `cli/rescue-cli/src/main.rs`.
- [x] Add shared dependencies for gzip, XML, JSON, errors, and CLI parsing.

## 3. Models

- [x] v0.1 legacy: Define `ALSAnalysis`.
- [x] Define `ActiveFileRef`.
- [x] Define `ALSWarning`.
- [x] Define `ALSError`.
- [x] Ensure output models are JSON-serializable.

## 4. Reader Logic

- [x] Add `analyze_als(path)` public API.
- [x] Check file existence.
- [x] Check file readability.
- [x] Decompress gzip.
- [x] Parse XML.
- [x] Validate Ableton root.
- [x] Extract Ableton metadata.
- [x] Count `SampleRef` nodes.
- [x] Extract direct child `SampleRef/FileRef`.
- [x] Count `SampleRef/SourceContext/.../OriginalFileRef`.
- [x] Count total `OriginalFileRef` nodes.
- [x] Capture raw `RelativePathType`.
- [x] Capture path fields and audio metadata fields.
- [x] v0.1 legacy: Check whether active paths exist on disk.
- [x] Return warnings for incomplete refs.

## 5. CLI

- [x] Implement `rescue analyze <path>` with JSON as its single output format.
- [x] v0.1 legacy: Print `ALSAnalysis` JSON to stdout.
- [x] Print fatal errors as structured JSON.
- [x] Return non-zero exit code on fatal errors.

## 6. Tests

- [x] Add `valid_als_returns_analysis_json`.
- [x] Add `active_refs_are_not_historical_refs`.
- [x] Add `relative_path_type_is_captured_raw`.
- [x] Add `invalid_gzip_returns_structured_error`.
- [x] Add `read_only_safety`.
- [x] Run manual CLI smoke test.
- [ ] Add automated CLI smoke test if practical.

## 7. Review

- [x] Run `cargo fmt --check`.
- [x] Run `cargo test` after Rust toolchain is available.
- [x] Run `rescue analyze` on at least one fixture after Rust toolchain is available.
- [x] Confirm no source fixture bytes changed through Rust test run.
- [x] Summarize changed files.
- [x] Update `CURRENT_STATE.md`.

## 8. v0.2 Contract Update

- [x] Replace module spec with `ALSReadModel v0.2` contract.
- [x] Update this task list so v0.1 responsibilities are marked as legacy.
- [x] Update fixture contract with v0.2 reader expectations and PathVerifier boundary.
- [x] Define `ALSReadModel`.
- [x] Define `SetMetadata`.
- [x] Define `ActiveAudioReference`.
- [x] Define `HistoricalReference`.
- [x] Define `NonAudioDependencySignal`.
- [x] Define `ALSReadWarning`.
- [x] Define `ALSReadError`.
- [x] Replace or wrap `ALSAnalysis` so downstream code does not consume CLI JSON as the internal contract.
- [x] Remove `exists_on_disk` from ALSReader output.
- [ ] Move path existence checks to future `PathVerifier` tests.
- [x] Preserve raw path fields and raw `RelativePathType`.
- [x] Preserve active vs historical references separately.
- [x] Add `usage_context`, `xml_context` and `xml_locator` when possible.
- [x] Add non-audio dependency signal extraction as report-only evidence.
- [x] Add contract tests for downstream-required fields.
- [x] Add a fixture or expectation for `RelativePathType 0`.
- [x] Keep CLI output as a user-facing diagnostic view derived from the reader model.
- [x] Run `cargo fmt --check`.
- [x] Run `cargo test`.

## 9. v0.2 Follow-Up Tasks

- [ ] Add automated CLI smoke test if practical.
- [ ] Decide exact stable `xml_locator` strategy before ALS rewrite work.
- [ ] Improve `usage_context` beyond `unknown` using confirmed ALS structure evidence.
- [ ] Create a dedicated PathVerifier spec before implementing path existence checks.

## 10. v0.2 Hardening After Review

- [x] Align ALSReader fatal error codes with spec-style taxonomy.
- [x] Add missing-file structured error test.
- [x] Add gzip-invalid-XML structured error test.
- [x] Add gzip-without-Ableton-root structured error test.
- [x] Add active `RelativePathType 0` regression test from copied ALS corpus.
- [x] Run `cargo fmt --check`.
- [x] Run `cargo test`.

## 11. Retrospective Guard Obligations

- [x] ENFORCED_BY_TEST: valid_als_returns_read_model_json
- [x] ENFORCED_BY_TEST: active_refs_are_not_historical_refs
- [x] ENFORCED_BY_TEST: relative_path_type_is_captured_raw
- [x] ENFORCED_BY_TEST: als_reader_v0_2_does_not_check_filesystem_paths
- [x] ENFORCED_BY_TEST: active_audio_refs_preserve_rewrite_relevant_fields
- [x] ENFORCED_BY_TEST: als_reader_does_not_infer_project_root_from_als_parent
- [x] ENFORCED_BY_TEST: xml_context_does_not_duplicate_the_observed_element
- [x] ENFORCED_BY_TEST: invalid_gzip_returns_structured_error
- [x] ENFORCED_BY_TEST: missing_file_returns_structured_error
- [x] ENFORCED_BY_TEST: gzip_with_invalid_xml_returns_structured_error
- [x] ENFORCED_BY_TEST: gzip_without_ableton_root_returns_structured_error
- [x] ENFORCED_BY_TEST: active_relative_path_type_zero_is_preserved
- [x] ENFORCED_BY_TEST: read_only_safety
- [x] ENFORCED_BY_TYPE: ALSReadModel
- [x] ENFORCED_BY_TYPE: SetMetadata
- [x] ENFORCED_BY_TYPE: ActiveAudioReference
- [x] ENFORCED_BY_TYPE: HistoricalReference
- [x] ENFORCED_BY_TYPE: NonAudioDependencySignal
- [x] ENFORCED_BY_TYPE: ALSReadWarning
- [x] ENFORCED_BY_TYPE: ALSReadError
- [x] ENFORCED_BY_GUARD: ENGINEERING_RULES.md version 0.1 referenced
- [x] ENFORCED_BY_GUARD: forbidden panic/unwrap/expect in product source
- [x] ENFORCED_BY_GUARD: forbidden destructive filesystem operations in product source
- [x] REVIEW_ONLY: names are understandable in the Ableton reader domain
- [x] REVIEW_ONLY: implementation avoids premature abstraction
- [x] REVIEW_ONLY: implementation follows ENGINEERING_RULES.md code review checklist
- [x] ENFORCED_BY_GUARD: required obligations checked before acceptance
- [x] ENFORCED_BY_GUARD: required tests are not ignored and have substance
- [x] ENFORCED_BY_GUARD: public contract exact field and type check
- [x] ENFORCED_BY_GUARD: scope ownership verbs blocked
- [x] ENFORCED_BY_GUARD: core source high-risk patterns blocked
- [x] ENFORCED_BY_GUARD: dependencies limited to allowed list
- [x] ENFORCED_BY_GUARD: hidden uncertainty phrases block build
- [x] DOCUMENTED_ONLY: module belongs to ALS inspection and dependency fact extraction flow
- [x] DOCUMENTED_ONLY: ALSReader reads ALS and emits ALSReadModel; it is not a verifier/classifier/matcher/planner/rewriter

## 12. Retrospective Guard Verification

- [x] Add `specs/001-als-reader/module.contract.json`.
- [x] Split public ALSReader API from private implementation for guarded contract clarity.
- [x] Run `python3 tools/workflow_guard.py module-ready 001-als-reader`.
- [x] Run `python3 tools/workflow_guard.py verify-module 001-als-reader`.
- [x] Run `cargo fmt --check`.
- [x] Run `cargo check --workspace`.
- [x] Run `cargo test --workspace`.
- [x] Run `cargo clippy --workspace --all-targets -- -D warnings`.
- [x] Update `CURRENT_STATE.md`.
- [x] Add session digest or closeout note.

## 13. v0.2.1 Hardening After External Review

- [x] Add text-based raw ALS path helper that handles `/` and `\` separators.
- [x] Stop using host `std::path::Path` for derived filename/extension from raw ALS paths.
- [x] Use `raw_relative_path` for derived filename/extension when `raw_path` is empty.
- [x] Add regression test for Windows drive-letter path on macOS.
- [x] Add regression test for Windows UNC path on macOS.
- [x] Add regression test for empty `raw_path` with present `raw_relative_path`.
- [x] Add compressed ALS input size limit.
- [x] Add decompressed XML size limit.
- [x] Add structured error codes for ALS size limit failures.
- [x] Bump `ALS_READER_VERSION` to `0.2.1`.
- [x] Add typed ALS path parser as future PathVerifier input support.
- [x] Add tests for macOS, Windows, UNC, relative and bare ALS path shapes.
- [x] Run `cargo fmt`.
- [x] Run `cargo test --workspace`.

## 14. v0.2.2 Corrective Audit Changes

- [x] Stop inferring `source_project_root` from the parent ALS directory.
- [x] Keep `source_project_root` null as a v0.2 compatibility field.
- [x] Fix duplicated terminal elements in historical/non-audio `xml_context`.
- [x] Remove the redundant CLI `--json` flag.
- [x] Add regression tests for Project root semantics and XML context.
- [x] Bump `ALS_READER_VERSION` to `0.2.2`.

Deferred from the v0.2.1 hardening pass and still deferred:

- [ ] Migrate decision/status strings to enums gradually, starting with 003 where possible.
- [ ] Add `activity_status` or rename semantics before ALS rewrite readiness if SampleRef context tests require it.
- [ ] Add stable dependency identity before ManifestWriter or cross-run comparison.
- [ ] Add parsed numeric evidence fields in PathVerifier/SampleMatcher, while keeping ALSReader raw fields as strings.
