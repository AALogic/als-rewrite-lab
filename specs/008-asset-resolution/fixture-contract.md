# Fixture Contract 008: AssetResolution

Status: ready
Date: 2026-07-27

All tests use synthetic immutable assessment and inventory contracts.

Required cases:

```text
one exact path + filename + size candidate
filename + size without exact path
same filename with different content IDs
two exact high-score paths
no plausible candidate
same OriginalCrc without path/name evidence
partial inventory
untrusted upstream result
stable candidate ordering
```
