# Plan 002: DependencyExtractor

Status: ready for guarded build pilot  
Date: 2026-06-09  
Scope: implement DependencyExtractor v0.1 after machine guard passes

## 1. Purpose

Build a pure normalization module that consumes `ALSReadModel v0.2` and returns
`DependencyExtractionResult v0.1`.

This module prepares active audio sample dependencies for later path verification,
classification, matching, planning and rewrite steps.

## 2. Build Order

```text
1. Run workflow_guard module-ready 002-dependency-extractor.
2. Confirm ENGINEERING_RULES.md quality obligations for this module.
3. Add data models and contract version constants.
4. Add tests from tasks.md Test Obligations.
5. Implement pure extract_dependencies(model).
6. Add boundary tests proving no filesystem, classification, match, copy or rewrite.
7. Run cargo fmt --check.
8. Run cargo check --workspace.
9. Run cargo test --workspace.
10. Run workflow_guard verify-module 002-dependency-extractor.
11. Update CURRENT_STATE / closeout.
```

`verify-module` uses guard v0.2 for this module. Acceptance is blocked if:

```text
required obligations are not checked
required tests are ignored, empty or tautological
public output differs from the exact contract
scope verbs/terms indicate scanner/matcher/rewriter/UI behavior
core source contains high-risk patterns
dependencies are outside the allowlist
hidden uncertainty is present without non-blocking classification
```

## 3. Implementation Boundary

Implementation should live in:

```text
crates/rescue_core/src/dependency_extractor.rs
```

`crates/rescue_core/src/lib.rs` should only expose the public API.

## 4. Refactor Rule

If implementation requires changing ALSReader models, stop and treat that as a
contract change, not normal implementation.

Do not change existing ALSReader tests as part of this module build.
