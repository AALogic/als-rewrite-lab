# Plan 004: ProjectDiscovery

Status: ready
Date: 2026-07-27

1. Add `rescue_analyzer` as a small workspace crate.
2. Write synthetic filesystem tests before implementation.
3. Add public discovery models and one read-only entrypoint.
4. Keep marker inspection separate from result classification.
5. Do not add recursive search, UI, database, or ALS parsing.
6. Run `cargo fmt --check`.
7. Run `cargo check --workspace --locked`.
8. Run `cargo test --workspace --locked`.
9. Run `cargo clippy --workspace --all-targets --locked -- -D warnings`.
10. Run `python3 tools/workflow_guard.py verify-module 004-project-discovery`.
