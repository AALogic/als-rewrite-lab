# Fixture Contract 007: AssetInventory

Status: ready
Date: 2026-07-27

Tests create temporary directories and small synthetic byte files. They never
scan user folders.

Required cases:

```text
recognized and ignored extensions
uppercase extension
two paths with identical bytes
same filename with different bytes
file and directory symlinks
relative, missing and non-directory roots
overlapping roots
entry limit
read-only snapshot
deterministic output on unchanged fixture
```
