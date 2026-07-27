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

The write modules remain available for isolated laboratory verification, but
the composed command currently blocks before staging until expected content
identity or an explicit user selection is present and every rewrite reference
has explicit supported-context evidence.

## Verify The Repository

Install the Rust toolchain selected by `rust-toolchain.toml`, then run:

```sh
cargo fmt --check
cargo test --workspace --locked
cargo clippy --workspace --all-targets --locked -- -D warnings
python3 tools/private_path_guard.py
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
primary controls are keeping all real test material outside this repository and
running `python3 tools/private_path_guard.py` before every publication.

The guard reads tracked paths, modes, blobs, and symlink targets from Git's
staged index while reading untracked entries from the worktree. It checks private
home paths, private-corpus identifiers, UTF-8 and UTF-16 path dumps, and rejects
unscannable tracked binary content. It also rejects tracked Ableton and common
audio extensions case-insensitively before decoding payloads, including OGG,
AAC, and SD2. Eligible staged blobs are size-checked before their contents are
streamed, and oversized blobs fail closed without loading their payloads. No
tracked synthetic media or binary fixture is allowlisted; tests create synthetic
media only in temporary directories. The guard does not sanitize existing Git
objects. Before this repository is handed to another laptop, the canonical
repository maintainer must scrub sensitive paths and real data from all reachable
history, verify the result from a fresh clone, and retire any pre-scrub clone or
reference that can still reach the old objects.
