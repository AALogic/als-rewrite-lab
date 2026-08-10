# Courier Collection V1: macOS E2E Evidence

Status: implementation accepted; external native-drop owner checks pending

Date: 2026-08-06

## Scope

This record covers the compact Courier flow composed from modules 024 through
032. It does not claim a WeTransfer upload, a cloud-provider API integration,
or production signing/notarization.

## Real Application Evidence

The packaged debug `.app` was launched through macOS Open With against copied
real ALS inputs. The following outcomes were observed:

1. A complete one-project request created a new Ableton Project directory with
   the rewritten ALS, four audio files and its manifest. The generated Set
   opened successfully in Ableton.
2. An unsupported input produced the generic Courier message
   `Nie mogę dla ciebie tego zrobić.` without exposing internal paths.
3. Two ALS inputs were retained in one ordered queue, shown as `2 projekty`,
   expanded into a removable list, and processed by one Play action.
4. Both successful Project directories entered one immutable collection
   snapshot.
5. Local collection delivery copied both Project directories into the
   configured Google Drive filesystem folder. The destination tree existed
   after delivery, and the UI returned to the ready state.
6. The ready scene showed one stable-size character, a separate parcel surface,
   Folder, WeTransfer and local-delivery controls, and no permanent close
   button. Double-click exposed context actions for a new order, reload where
   applicable, and close.

## Automated Evidence

The acceptance run passed:

```text
npm run check
npm test -- --run
npm run build
cargo fmt --check
cargo test --workspace
cargo test -p als-rescue-desktop --lib
cargo check -p als-rescue-desktop --lib
python3 tools/workflow_guard.py verify-all
```

The frontend suite contained 25 passing tests. Workflow Guard verified all 32
module contracts.

The final debug `.app` was re-signed locally with an ad hoc identity and passed
`codesign --verify --deep --strict`. This proves bundle integrity on the test
Mac; it is not Developer ID signing or notarization.

## Remaining Manual Evidence

The following are intentionally not marked complete:

1. Finder receives every selected Project directory from one parcel drag.
2. The parcel-only native drag image follows the pointer and a cancelled drag
   restores the ready state.
3. WeTransfer displays its own native folder-drop hover treatment.
4. A downloaded transfer preserves the complete Project directory trees.

These checks require the owner's live cross-application interaction and were
explicitly excluded from automated execution.

## Host Filesystem Incident

A later run against `$HOME/Downloads/sexy_testy` stopped inside the operating
system while creating the private ledger. A process sample identified
`std::fs::hard_link` beneath `write_json_noclobber`; after the application was
terminated, ordinary Terminal enumeration of that same directory also blocked.
`diskutil verifyVolume /System/Volumes/Data` reported that the APFS volume needed
repair and completed a deferred repair.

The evidence distinguishes this from an application-level queue deadlock: the
application process was idle in a kernel filesystem operation, and independent
directory readers blocked on the same host path. Restart macOS before reusing
that directory, then verify it with Finder and a simple directory listing. Do
not delete or rewrite source ALS or source audio as part of recovery.
