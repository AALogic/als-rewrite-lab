# History Reconstruction - 2026-08-10

## Purpose

This record explains how a large accepted working tree was converted into an
auditable Git milestone without pretending that every earlier conversational
step had been committed when it happened.

## Preserved History

The existing 53 commits leading to `2add94a` were preserved unchanged. The
working tree was moved from the historically named Windows branch to
`codex/als-rescue-desktop-milestone`. No old commit was rebased, squashed or
rewritten.

The uncommitted product work was divided by evidence and responsibility:

1. `971f6b6` records project discovery, the project catalog, batch rescue,
   courier collection and delivery, desktop surfaces, module specifications,
   tests and the final pre-migration macOS window policy.
2. `23cff6c` records only the Finder-compatible floating-window migration, the
   cold-start Open With buffer, updated module 025 contracts and the isolated
   runtime evidence supporting that change.
3. The following documentation commit records the recovery procedure, toolchain
   pins and this reconstruction statement.
4. `d73f776` corrects two Windows-only compile boundaries exposed by the first
   canonical Draft PR: generated Win32 constant namespaces and macOS-only Tauri
   run events.
5. `2559342` aligns the native-path normalization test with Windows
   case-insensitive queue keys and removes a platform-specific unused import.
6. `85b3ba7` replaces Unix-only batch-copy fixture paths with one shared native
   path factory, preserving the same test scenarios on macOS and Windows.

The immutable `v0.1.0-courier-macos-alpha` tag remains the first validated macOS
recovery point. It is not moved. The preferred
`v0.1.1-courier-macos-alpha` tag identifies the complete state including both
Windows CI corrections.

## Why The Split Is Honest

The macOS floating-window change had a separate backup of the exact earlier
source and dedicated experiment evidence. That made a real before/after commit
boundary recoverable. The rest of the accepted working tree did not contain
enough reliable timestamps or intermediate snapshots to reconstruct smaller
commits without inventing history, so it is represented as one explicit
milestone commit.

## Off-Repository Recovery Material

The backup directory is:

```text
$HOME/Documents/New project/history-reconstruction-backup-2026-08-10/
```

Checksums:

```text
470d8cec7cf170b81a2e971316cc7ba56ce90a593f9e808b5ec06a20ca280bbc  als-rewrite-lab-windows-working-tree-source.tar.gz
0e6cce5d40b0f2e4e06228d7bfd7b0899f26686b6e6746a8b993c4d70aa6402b  als-rewrite-lab-all-refs.bundle
03fef7397d9fb82dc0cdcfb134eed9c1de83d80b3bb67895e38fdeec70946af1  ALS-Rescue-installed-runtime-validated.zip
```

A separate pre-migration source and application backup remains under:

```text
$HOME/Documents/New project/migration-backups/courier-production-before-floating-2026-08-10/
```

These directories are local recovery material and are intentionally not tracked
because they contain bulky runtime artifacts and source snapshots already
represented by Git commits.

## Validation Performed

The reconstructed source was checked with locked Rust compilation, all workspace
tests, warning-free Clippy, Rust formatting, frontend tests, TypeScript checking,
the production frontend build, dependency audit, private-data guard and all
module workflow contracts. The tagged state is additionally rebuilt from a
fresh clone as part of milestone publication.

The macOS application had already passed the recorded owner/runtime smoke tests.
The Git validation does not replace the remaining repeated native acceptance
matrix documented in the module 025 evidence.

## Known Boundary

The default private-data guard passes for the current tree. The full-history
audit was also run and found 220 violations across 37 reachable legacy objects:
144 private identifiers, 50 private corpus roots, 18 macOS home paths, four
Windows home paths, three private provenance records and one private media
filename.

The complete private audit report remains outside Git at:

```text
$HOME/Documents/New project/history-reconstruction-backup-2026-08-10/release-history-audit-2026-08-10.txt
```

Its SHA-256 is:

```text
5547b48a4054da7c6abeb8f3e5e0aa973d781562bf549a1273b5e865fe60f224
```

The private repository and private alpha release may be used for recovery, but
a public or commercial release remains blocked until
`private_path_guard.py --release-history` passes after the documented
history-recovery procedure is completed.

## Future Rule

Accepted product changes should be committed in small behavior-oriented units.
Each release candidate must have:

- a clean tagged source state;
- passing locked checks;
- a release artifact and checksum;
- a short runtime acceptance record;
- a verified fresh-clone build.

Do not create fictional micro-commits later. If work accumulates again, record
one honest checkpoint and split only the boundaries supported by source backups,
tests or experiment evidence.
