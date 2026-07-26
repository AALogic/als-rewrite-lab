# Tasks 003: PathObservation

Status: blocked before build  
Date: 2026-07-26

## 1. Evidence And Readiness

- [ ] DOCUMENTED_ONLY: E-01 confirms admitted RelativePathType candidate rules
- [ ] DOCUMENTED_ONLY: synthetic cross-platform fixture matrix is accepted
- [ ] ENFORCED_BY_GUARD: blocking markers removed only after evidence review
- [ ] ENFORCED_BY_GUARD: ENGINEERING_RULES.md version 0.1 referenced
- [ ] ENFORCED_BY_GUARD: required obligations checked before acceptance
- [ ] ENFORCED_BY_GUARD: required tests are not ignored and have substance
- [ ] ENFORCED_BY_GUARD: public contract exact field and type check
- [ ] ENFORCED_BY_GUARD: scope ownership verbs blocked
- [ ] ENFORCED_BY_GUARD: core source high-risk patterns blocked
- [ ] ENFORCED_BY_GUARD: dependencies limited to allowed list
- [ ] ENFORCED_BY_GUARD: hidden uncertainty phrases block build
- [ ] REVIEW_ONLY: names describe observations, not verification or identity
- [ ] REVIEW_ONLY: implementation avoids premature abstraction
- [ ] REVIEW_ONLY: implementation follows ENGINEERING_RULES.md

## 2. Required Behavior Obligations

- [ ] ENFORCED_BY_TEST: confirmed_project_root_produces_relative_candidate
- [ ] ENFORCED_BY_TEST: absent_project_root_produces_no_relative_candidate
- [ ] ENFORCED_BY_TEST: both_safe_candidates_are_preserved_without_selection
- [ ] ENFORCED_BY_TEST: existing_regular_file_is_observed_without_identity_claim
- [ ] ENFORCED_BY_TEST: missing_candidate_is_observed
- [ ] ENFORCED_BY_TEST: directory_candidate_is_observed
- [ ] ENFORCED_BY_TEST: symlink_is_reported_and_not_followed
- [ ] ENFORCED_BY_TEST: parent_escape_candidate_is_rejected
- [ ] ENFORCED_BY_TEST: size_mismatch_does_not_select_or_resolve_asset
- [ ] ENFORCED_BY_TEST: foreign_platform_path_is_preserved_but_not_checked
- [ ] ENFORCED_BY_TEST: duplicate_reference_occurrences_are_not_deduplicated
- [ ] ENFORCED_BY_TEST: raw_paths_are_preserved_exactly
- [ ] ENFORCED_BY_TEST: path_observation_is_metadata_only
- [ ] ENFORCED_BY_TEST: fake_dependency_assessment_consumes_all_observations
- [ ] ENFORCED_BY_TYPE: PathObservationContext
- [ ] ENFORCED_BY_TYPE: PathObservationResult
- [ ] ENFORCED_BY_TYPE: PathObservationMetadata
- [ ] ENFORCED_BY_TYPE: DependencyPathObservation
- [ ] ENFORCED_BY_TYPE: CandidatePathObservation
- [ ] ENFORCED_BY_TYPE: PathObservationWarning
- [ ] ENFORCED_BY_TYPE: PathObservationError
- [ ] DOCUMENTED_ONLY: DependencyRef v0.1 is a ReferenceOccurrence compatibility model
- [ ] DOCUMENTED_ONLY: PathObservation does not select or verify asset identity

## 3. Implementation Tasks

- [ ] Create `crates/rescue_core/src/path_observation.rs`.
- [ ] Create `crates/rescue_core/src/path_observation_impl.rs`.
- [ ] Export `observe_dependency_paths` and public result models.
- [ ] Add `crates/rescue_core/tests/path_observation.rs`.
- [ ] Add synthetic path fixture helpers without private user files.
- [ ] Run `cargo fmt --check`.
- [ ] Run `cargo check --workspace --locked`.
- [ ] Run `cargo test --workspace --locked`.
- [ ] Run `cargo clippy --workspace --all-targets -- -D warnings`.
- [ ] Run workflow_guard verify-module 003-path-verifier.

## 4. Explicitly Not A Task

```text
selecting the first existing candidate
creating verified_exact_path
grouping occurrences into RequiredAsset
scanning directories
matching missing samples
copying or rewriting files
```
