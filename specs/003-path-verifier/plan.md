# Plan 003: PathObservation

Status: blocked before build  
Date: 2026-07-26

## 1. Goal

Implement a read-only module that records all safe filesystem observations for
path candidates derived from `DependencyRef v0.1` without selecting or
confirming asset identity.

## 2. Evidence Before Code

1. Run E-01 project-root and path-semantics experiment.
2. Add synthetic fixtures for macOS, Windows, Unicode, parent escape, directory,
   symlink, inaccessible and missing paths.
3. Record which `RelativePathType` values may create a project-relative
   candidate in v0.2.
4. Remove `BLOCKING_UNKNOWN` markers only when fixture evidence resolves them.
5. Run workflow_guard module-ready 003-path-verifier.

Do not write product code while the module-ready guard is blocked.

## 3. Build Sequence After Evidence

1. Add public observation models in
   `crates/rescue_core/src/path_observation.rs`.
2. Add private candidate-generation and metadata implementation in
   `crates/rescue_core/src/path_observation_impl.rs`.
3. Use `ParsedAlsPath` and `AlsPathKind` only for path-shape interpretation.
4. Introduce a narrow read-only filesystem metadata port.
5. Write required tests before implementation where practical.
6. Export `observe_dependency_paths` from `rescue_core`.
7. Do not add a CLI command until the core contract is accepted.
8. Run `cargo fmt --check`.
9. Run `cargo check --workspace --locked`.
10. Run `cargo test --workspace --locked`.
11. Run `cargo clippy --workspace --all-targets -- -D warnings`.
12. Run workflow_guard verify-module 003-path-verifier.

## 4. Implementation Constraints

```text
no async runtime
no SQLite
no global scanning
no directory walking
no hashing or audio decode
no candidate selection
no copy or rewrite
no inferred Project root
no new dependency without explicit review
```

Candidate generation and filesystem metadata reads remain separate so platform
adapters can change without changing domain output.

## 5. Stop Conditions

Stop and return to specification/evidence if:

```text
project-relative meaning requires an untested RelativePathType assumption
code needs to follow a symlink
the output starts representing resolved asset identity
the module needs to search beyond explicit candidates
the application cannot provide explicit PathObservationContext
tests require private user projects rather than synthetic fixtures
```

## 6. Expected Downstream Proof

Before closing 003, build a fake `DependencyAssessment` consumer proving that:

```text
all candidates are available to downstream
no field implies a selected path
availability and identity remain separate
unknown and unsupported states remain explainable
```
