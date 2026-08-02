# Fixture Contract 008: AssetResolution

Status: ready for policy v0.3
Date: 2026-08-02

All tests use synthetic immutable assessment and inventory contracts.

Required cases:

```text
one exact path + filename + size candidate without an expected hash
filename + size without exact path
same filename with different content IDs
two exact high-score paths
no plausible candidate
same OriginalCrc without path/name evidence
partial inventory
untrusted upstream result
stable candidate ordering
exact recorded-path binding without a historical identity claim
source-bound explicit path-and-SHA-256 selection
selected file changed after the decision
selection bound to another ALS snapshot
```
