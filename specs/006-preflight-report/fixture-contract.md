# Fixture Contract 006: PreflightReport

Status: ready
Date: 2026-07-27

Synthetic trusted inputs cover all aggregate statuses. A generated gzip/XML ALS
is used by the CLI smoke test so no private fixture is required.

Cases:

```text
all requirements have observed regular-file candidates
at least one requirement has only missing candidates
at least one requirement is unknown
upstream project discovery or assessment is untrusted
source paths disagree
local candidate paths remain visible only in this private report
```
