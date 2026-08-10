# Fixture Contract 020: ALSProjectScanner

Status: ready
Date: 2026-08-03

Automated tests create temporary synthetic directory trees. They never scan the
user home folder and never use private real project names.

## Required Synthetic Tree

```text
approved-root-a/
  Standard Project/
    Ableton Project Info/
    Main.als
    second-main.ALS
    notes.txt
    Backup/
      Main [2026-08-01 010101].als
  Standalone/
    orphan.als
  Broken/
    not-gzip.als
  Excluded/
    ignored.als

approved-root-b/
  Standard Project/
    Main.als
```

Files may contain tiny marker bytes. Their contents are not valid ALS unless a
test explicitly needs to prove the scanner does not parse content.

## Required Cases

```text
standard marker and ALS observation
Backup ALS retained as a raw observation
standalone ALS retained
invalid gzip ALS retained without opening
uppercase ALS extension
non-ALS ignored
multiple main Sets retained
same filename and folder name under different roots retained separately
overlapping roots
absolute excluded subtree
file and directory symlinks
positive entry limit
optional depth limit
unreadable directory where the host can enforce permissions
cancellation after bounded progress
progress-event redaction
unchanged source tree
deterministic repeated result
fake ProjectCatalogBuilder consumer
```

## Platform Cases

```text
macOS/Linux:
  directory and file symlinks
  optional non-Unicode path test on Unix

Windows native CI:
  backslash/native PathBuf behavior
  directory symlink or junction/reparse traversal rejection
  case-insensitive .ALS extension

Physical Windows laboratory:
  local NTFS root
  removable root disappearance or access failure if safely reproducible
```

Platform-specific tests may be conditionally compiled only when the condition
is the platform behavior being tested. Core contract tests must run on macOS
and Windows.

## Read-Only Proof

Before and after the scan, the test records:

```text
relative entry list
regular-file byte contents
regular-file sizes
```

The scanner passes only when these values are unchanged and no new entry was
created.
