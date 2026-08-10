# Tasks 003: PathObservation

Status: ready for build
Date: 2026-07-27

## 1. Evidence And Readiness

- [x] DOCUMENTED_ONLY: E-01 confirms admitted RelativePathType candidate rules
- [x] DOCUMENTED_ONLY: synthetic cross-platform fixture matrix is accepted
- [x] ENFORCED_BY_GUARD: blocking markers removed only after evidence review
- [x] ENFORCED_BY_GUARD: ENGINEERING_RULES.md version 0.1 referenced
- [x] ENFORCED_BY_GUARD: required obligations checked before acceptance
- [x] ENFORCED_BY_GUARD: required tests are not ignored and have substance
- [x] ENFORCED_BY_GUARD: public contract exact field and type check
- [x] ENFORCED_BY_GUARD: scope ownership verbs blocked
- [x] ENFORCED_BY_GUARD: core source high-risk patterns blocked
- [x] ENFORCED_BY_GUARD: dependencies limited to allowed list
- [x] ENFORCED_BY_GUARD: hidden uncertainty phrases block build
- [x] REVIEW_ONLY: names describe observations, not verification or identity
- [x] REVIEW_ONLY: implementation avoids premature abstraction
- [x] REVIEW_ONLY: implementation follows ENGINEERING_RULES.md

## 2. Required Behavior Obligations

- [x] ENFORCED_BY_TEST: confirmed_project_root_produces_relative_candidate
- [x] ENFORCED_BY_TEST: relative_path_type_zero_uses_raw_path
- [x] ENFORCED_BY_TEST: type_one_and_five_relative_paths_are_not_project_joined
- [x] ENFORCED_BY_TEST: absent_project_root_produces_no_relative_candidate
- [x] ENFORCED_BY_TEST: both_safe_candidates_are_preserved_without_selection
- [x] ENFORCED_BY_TEST: existing_regular_file_is_observed_without_identity_claim
- [x] ENFORCED_BY_TEST: missing_candidate_is_observed
- [x] ENFORCED_BY_TEST: directory_candidate_is_observed
- [x] ENFORCED_BY_TEST: symlink_is_reported_and_not_followed
- [x] ENFORCED_BY_TEST: parent_escape_candidate_is_rejected
- [x] ENFORCED_BY_TEST: size_mismatch_does_not_select_or_resolve_asset
- [x] ENFORCED_BY_TEST: foreign_platform_path_is_preserved_but_not_checked
- [x] ENFORCED_BY_TEST: duplicate_reference_occurrences_are_not_deduplicated
- [x] ENFORCED_BY_TEST: raw_paths_are_preserved_exactly
- [x] ENFORCED_BY_TEST: path_observation_is_metadata_only
- [x] ENFORCED_BY_TEST: fake_dependency_assessment_consumes_all_observations
- [x] ENFORCED_BY_TYPE: PathObservationContext
- [x] ENFORCED_BY_TYPE: PathObservationResult
- [x] ENFORCED_BY_TYPE: PathObservationMetadata
- [x] ENFORCED_BY_TYPE: DependencyPathObservation
- [x] ENFORCED_BY_TYPE: CandidatePathObservation
- [x] ENFORCED_BY_TYPE: PathObservationWarning
- [x] ENFORCED_BY_TYPE: PathObservationError
- [x] DOCUMENTED_ONLY: DependencyRef v0.1 is a ReferenceOccurrence compatibility model
- [x] DOCUMENTED_ONLY: PathObservation does not select or verify asset identity

## 3. Implementation Tasks

- [x] Create `crates/rescue_core/src/path_observation.rs`.
- [x] Create `crates/rescue_core/src/path_observation_impl.rs`.
- [x] Export `observe_dependency_paths` and public result models.
- [x] Add `crates/rescue_core/tests/path_observation.rs`.
- [x] Add synthetic path fixture helpers without private user files.
- [x] Run `cargo fmt --check`.
- [x] Run `cargo check --workspace --locked`.
- [x] Run `cargo test --workspace --locked`.
- [x] Run `cargo clippy --workspace --all-targets -- -D warnings`.
- [x] Run workflow_guard verify-module 003-path-verifier.

## 4. Explicitly Not A Task

```text
selecting the first existing candidate
creating verified_exact_path
grouping occurrences into RequiredAsset
scanning directories
matching missing samples
copying or rewriting files
```
