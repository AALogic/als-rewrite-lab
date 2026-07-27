# ALS Rewrite Lab

Private Rust laboratory for evidence-based Ableton audio dependency recovery.

The current code implements a guarded, one-project vertical slice:

```text
read ALS -> extract audio references -> observe paths -> assess requirements
-> inventory bounded folders -> resolve candidates -> plan -> stage copies
-> rewrite a copied ALS -> validate -> write manifests -> promote a fresh package
```

The original ALS and original media remain read-only. The laboratory command is
not a finished desktop product and does not yet claim rewrite support for Live 9,
Live 10, Live 12, or native Windows projects.

## Verify The Repository

Install the Rust toolchain selected by `rust-toolchain.toml`, then run:

```sh
cargo fmt --check
cargo test --workspace --locked
cargo clippy --workspace --all-targets --locked -- -D warnings
python3 -m unittest discover -s tools/tests -p "test_*.py"
```

The CLI can then be built with:

```sh
cargo build --release --locked -p rescue-cli
```

## Repository Map

```text
crates/                 domain libraries
cli/rescue-cli/         laboratory command-line interface
specs/                  module contracts and acceptance obligations
tools/workflow_guard.py machine-checkable workflow rules
docs/                   architecture and experiment summaries
tests/fixtures/         synthetic, reproducible fixtures only
```

Read `AGENTS.md`, `CURRENT_STATE.md`, and `PRODUCT_SPINE.md` before changing
product behavior.

## Windows Test Laboratory

Use [docs/setup/WINDOWS_TEST_LAB.md](docs/setup/WINDOWS_TEST_LAB.md) to prepare a
Windows laptop with Ableton Live 9 or 10. Start with read-only analysis. The
current rewrite ruleset must reject unsupported Live versions rather than guess.

## Private Data Rule

Never commit real ALS files, audio, Ableton analysis files, local path dumps, or
private ledgers. `.gitignore` blocks common Ableton and audio extensions, but the
primary control is keeping all real test material outside this repository.

