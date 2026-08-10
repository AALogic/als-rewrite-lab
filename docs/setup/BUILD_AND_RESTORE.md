# Build And Restore

This document is the shortest supported route from an empty machine to the
source milestone represented by `v0.1.0-courier-macos-alpha`.

## Canonical Source

- private repository: `https://github.com/AALogic/als-rewrite-lab.git`
- release tag: `v0.1.0-courier-macos-alpha`
- Rust version: pinned by `rust-toolchain.toml`
- Node version used for the milestone: pinned by `.nvmrc`
- Rust and JavaScript dependencies: pinned by `Cargo.lock` and
  `apps/rescue-desktop/package-lock.json`

The release tag is the immutable recovery point. Branch names describe work in
progress and must not be treated as release identifiers.

## Restore The Tagged Source

```sh
git clone https://github.com/AALogic/als-rewrite-lab.git
cd als-rewrite-lab
git fetch --tags
git switch --detach v0.1.0-courier-macos-alpha
```

Authentication is required because the repository is private.

## Verify The Source

From the repository root:

```sh
cargo fmt --all --check
cargo check --workspace --locked
cargo test --workspace --locked
cargo clippy --workspace --all-targets --locked -- -D warnings
python3 tools/private_path_guard.py
python3 -m unittest discover -s tools/tests -p "test_*.py"
python3 tools/workflow_guard.py verify-all
npm --prefix apps/rescue-desktop ci
npm --prefix apps/rescue-desktop audit --audit-level=high
npm --prefix apps/rescue-desktop test -- --run
npm --prefix apps/rescue-desktop run check
npm --prefix apps/rescue-desktop run build
```

The private-data guard checks the tagged tree. A separate full-history audit is
required before a public or commercial release:

```sh
python3 tools/private_path_guard.py --release-history
```

## Build The macOS Application

On Apple Silicon macOS with Xcode Command Line Tools installed:

```sh
npm --prefix apps/rescue-desktop ci
cd apps/rescue-desktop
npm run tauri -- build
```

The application bundle is created under
`target/release/bundle/macos/ALS Rescue.app` relative to the repository root.
The private alpha artifact is ad-hoc signed and is not notarized for public
distribution.

## Build On Windows

Use an x64 Windows host with the Microsoft C++ Build Tools and WebView2 runtime:

```powershell
rustup show
npm --prefix apps/rescue-desktop ci
cargo test --workspace --locked
cd apps/rescue-desktop
npm run tauri -- build
```

The code and installer pipeline have Windows coverage, but the tagged release
does not claim broad runtime compatibility across Live and Windows versions.
Follow the test procedure in `docs/setup/WINDOWS_ALPHA_INSTALL_AND_TEST.md`.

## Restore The Emergency Backup

The local reconstruction backup is outside Git:

```text
$HOME/Documents/New project/history-reconstruction-backup-2026-08-10/
```

It contains a source archive, an all-refs Git bundle and the validated installed
macOS runtime. Verify the checksums recorded in
`docs/history/HISTORY_RECONSTRUCTION_2026-08-10.md` before using it.

To inspect all preserved Git refs without contacting GitHub:

```sh
git bundle verify als-rewrite-lab-all-refs.bundle
git clone als-rewrite-lab-all-refs.bundle als-rewrite-lab-restored
```

Real ALS files, audio, local manifests and private diagnostics are deliberately
not stored in Git or in the GitHub release.
