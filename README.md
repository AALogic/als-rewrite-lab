# ALS Rewrite Lab

Private Rust/Tauri project for evidence-based Ableton audio dependency recovery
and portable project-copy workflows.

The current code implements the guarded one-project pipeline and reuses it in a
project catalog, sequential batch workflow and compact macOS Courier:

```text
read ALS -> extract audio references -> observe paths -> assess requirements
-> inventory bounded folders -> resolve candidates -> plan -> stage copies
-> rewrite a copied ALS -> validate -> write manifests -> promote a fresh package
```

The original ALS and original media remain read-only. The local Tauri alpha can
analyze projects and create plan-bound current-path copies. It is still a
private test release: it is not notarized for public distribution and does not
claim broad rewrite compatibility across Live or Windows versions.

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
python3 tools/workflow_guard.py verify-all
npm --prefix apps/rescue-desktop ci
npm --prefix apps/rescue-desktop test
npm --prefix apps/rescue-desktop run check
npm --prefix apps/rescue-desktop run build
```

The CLI can then be built with:

```sh
cargo build --release --locked -p rescue-cli
```

The [quality workflow](.github/workflows/quality.yml) runs the locked Rust
workspace on macOS and Windows, checks the private-data policy and every
discovered module contract, and tests/builds the desktop frontend. A configured
workflow is not evidence of native Windows product support until its run passes.

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

For an immutable milestone, exact verification commands and clean-machine build
instructions, use [Build And Restore](docs/setup/BUILD_AND_RESTORE.md). The
history reconstruction is recorded in
[History Reconstruction 2026-08-10](docs/history/HISTORY_RECONSTRUCTION_2026-08-10.md).

## Windows Test Laboratory

Use [docs/setup/WINDOWS_TEST_LAB.md](docs/setup/WINDOWS_TEST_LAB.md) to prepare a
Windows laptop with Ableton Live 9 or 10. Start with read-only analysis. The
current rewrite ruleset must reject unsupported Live versions rather than guess.

When handing the copied project folder or cloned repository to Codex on the
Windows laptop, start with
[docs/setup/WINDOWS_CODEX_HANDOFF.md](docs/setup/WINDOWS_CODEX_HANDOFF.md). It
contains the exact repository fallback, test scope, safety boundary, and report
expected from that Windows session.

## Private Data Rule

Never commit real ALS files, audio, Ableton analysis files, local path dumps, or
private ledgers. `.gitignore` blocks common Ableton and audio extensions, but the
primary controls are keeping all real test material outside this repository and
running `python3 tools/private_path_guard.py` before every private preparatory PR
or Windows read-only laboratory handoff.

The default guard checks the staged index and untracked worktree. It checks
private home paths, private-corpus identifiers, UTF-8 and UTF-16 path dumps, and
rejects unscannable tracked binary content except for an exact allowlist of
required application icons and product-owned visual assets. It also rejects
tracked Ableton and common audio extensions case-insensitively before decoding payloads, including
OGG, AAC, and SD2. Eligible objects are size-checked before their contents are
streamed, and oversized objects fail closed without loading their payloads. No
tracked synthetic media or binary fixture is allowlisted; tests create synthetic
media only in temporary directories.

Before a public or commercial release, run the explicit full-history audit:

```sh
python3 tools/private_path_guard.py --release-history
```

That mode scans paths and object contents reachable from every local Git ref;
shallow clones fail because they cannot prove complete history. The audit
detects unsafe history but does not rewrite it. Legacy `reachable_*` findings do
not block a private preparatory PR or read-only Windows laboratory handoff, but
they must be resolved before any public or commercial release as described in
[the release-history recovery procedure](docs/setup/WINDOWS_TEST_LAB.md#blocked-release-history-recovery).
