# Tasks 002: DependencyExtractor

Status: implemented and verified  
Date: 2026-06-09  
Scope: build DependencyExtractor v0.1 after spec review

## 1. Spec And Contract

- [x] Create `specs/002-dependency-extractor/spec.md`.
- [x] Create `specs/002-dependency-extractor/fixture-contract.md`.
- [x] Create `specs/002-dependency-extractor/tasks.md`.
- [x] Review spec with Product Office before implementation.
- [x] Confirm `DependencyRef v0.1` fields.
- [x] Confirm `DependencyExtractionResult v0.1` fields.
- [x] Confirm forbidden fields for v0.1.
- [x] Clarify forbidden field ownership before implementation.
- [x] Clarify invalid input output shape before implementation.
- [x] Clarify warning `dependency_id` / `als_ref_id` optionality.
- [x] Add `module.contract.json` for guarded build pilot.
- [x] Add `plan.md` for guarded build readiness.

## 2. Test Obligations

- [x] ENFORCED_BY_TEST: valid_model_extracts_audio_dependencies
- [x] ENFORCED_BY_TEST: zero_active_refs_is_valid
- [x] ENFORCED_BY_TEST: one_active_ref_becomes_one_dependency_ref
- [x] ENFORCED_BY_TEST: duplicate_refs_are_not_deduplicated
- [x] ENFORCED_BY_TEST: incomplete_ref_is_preserved_with_warning
- [x] ENFORCED_BY_TEST: historical_refs_are_ignored_with_summary
- [x] ENFORCED_BY_TEST: non_audio_signals_are_ignored_with_summary
- [x] ENFORCED_BY_TEST: unsupported_als_read_model_version_returns_error
- [x] ENFORCED_BY_TEST: input_with_fatal_reader_errors_returns_no_trusted_model_error
- [x] ENFORCED_BY_TEST: output_is_deterministic
- [x] ENFORCED_BY_TEST: forbidden_downstream_fields_are_absent
- [x] ENFORCED_BY_TEST: raw_scalar_values_remain_strings
- [x] ENFORCED_BY_TEST: fake_path_observation_consumes_dependency_result
- [x] ENFORCED_BY_TYPE: DependencyExtractionResult
- [x] ENFORCED_BY_TYPE: DependencyExtractionMetadata
- [x] ENFORCED_BY_TYPE: DependencyRef
- [x] ENFORCED_BY_TYPE: IgnoredInputSummary
- [x] ENFORCED_BY_TYPE: DependencyExtractionWarning
- [x] ENFORCED_BY_TYPE: DependencyExtractionError
- [x] DOCUMENTED_ONLY: module belongs to Mass Collect / dependency reporting flow
- [x] DOCUMENTED_ONLY: DependencyExtractor is a normalizer, not verifier/classifier/matcher/planner/rewriter

## 3. Engineering Quality Obligations

- [x] ENFORCED_BY_GUARD: ENGINEERING_RULES.md version 0.1 referenced
- [x] ENFORCED_BY_GUARD: forbidden panic/unwrap/expect in product source
- [x] ENFORCED_BY_GUARD: forbidden destructive filesystem operations in product source
- [x] REVIEW_ONLY: names are understandable in the Ableton dependency domain
- [x] REVIEW_ONLY: implementation avoids premature abstraction
- [x] REVIEW_ONLY: implementation follows ENGINEERING_RULES.md code review checklist
- [x] ENFORCED_BY_GUARD: required obligations checked before acceptance
- [x] ENFORCED_BY_GUARD: required tests are not ignored and have substance
- [x] ENFORCED_BY_GUARD: public contract exact field and type check
- [x] ENFORCED_BY_GUARD: scope ownership verbs blocked
- [x] ENFORCED_BY_GUARD: core source high-risk patterns blocked
- [x] ENFORCED_BY_GUARD: dependencies limited to allowed list
- [x] ENFORCED_BY_GUARD: hidden uncertainty phrases block build

## 4. Data Models

- [x] Define `DependencyExtractionResult`.
- [x] Define `DependencyExtractionMetadata`.
- [x] Define `DependencyRef`.
- [x] Define `IgnoredInputSummary`.
- [x] Define `DependencyExtractionWarning`.
- [x] Define `DependencyExtractionError`.
- [x] Add contract version constants.
- [x] Ensure output models are JSON-serializable.

## 5. Extractor Logic

- [x] Add pure function `extract_dependencies(model)`.
- [x] Validate ALSReadModel version.
- [x] Reject untrusted model with fatal errors.
- [x] Iterate `active_audio_references` in order.
- [x] Create one `DependencyRef` per active audio ref.
- [x] Generate deterministic `dependency_id`.
- [x] Compute `path_basis`.
- [x] Compute `extraction_status`.
- [x] Preserve raw fields.
- [x] Preserve OriginalFileSize and OriginalCrc as evidence only.
- [x] Propagate relevant upstream warnings.
- [x] Add extraction warnings for incomplete refs.
- [x] Build `ignored_input_summary`.

## 6. Boundary Protection

- [x] Confirm extractor does not read sample files.
- [x] Confirm extractor does not check path existence.
- [x] Confirm extractor does not create `resolved_path`.
- [x] Confirm extractor does not create `existence_status`.
- [x] Confirm extractor does not create `source_category`.
- [x] Confirm extractor does not create `risk_flags`.
- [x] Confirm extractor does not deduplicate dependencies.
- [x] Confirm extractor does not copy/delete/rewrite files.

## 7. Tests

- [x] Add `valid_model_extracts_audio_dependencies`.
- [x] Add `zero_active_refs_is_valid`.
- [x] Add `one_active_ref_becomes_one_dependency_ref`.
- [x] Add `duplicate_refs_are_not_deduplicated`.
- [x] Add `incomplete_ref_is_preserved_with_warning`.
- [x] Add `historical_refs_are_ignored_with_summary`.
- [x] Add `non_audio_signals_are_ignored_with_summary`.
- [x] Add `unsupported_als_read_model_version_returns_error`.
- [x] Add `input_with_fatal_reader_errors_returns_no_trusted_model_error`.
- [x] Add `output_is_deterministic`.
- [x] Add `forbidden_downstream_fields_are_absent`.
- [x] Add `raw_scalar_values_remain_strings`.
- [x] Add `fake_path_observation_consumes_dependency_result`.

## 8. Verification

- [x] Run `python3 tools/workflow_guard.py module-ready 002-dependency-extractor`.
- [x] Run `cargo fmt --check`.
- [x] Run `cargo test --workspace`.
- [x] Run `cargo check --workspace`.
- [x] Run `cargo clippy --workspace --all-targets -- -D warnings` if clippy is installed.
- [x] Run `python3 tools/workflow_guard.py verify-module 002-dependency-extractor`.
- [x] Update `CURRENT_STATE.md`.
- [x] Add session digest or closeout note after implementation.

## 9. Non-Blocking Future Work

- [ ] Add CLI command only if useful after core behavior is stable.
- [ ] Add richer dependency kinds after VST/preset specs exist.
- [ ] Add candidate path observations only in PathObservation.
- [ ] Add source category only in DependencyAssessment.
- [ ] Add grouping only in DependencyAssessment; never mutate reference occurrences.
