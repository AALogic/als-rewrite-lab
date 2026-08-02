# Fixture Contract 009: PackagePlanner

Status: ready
Date: 2026-07-27

Synthetic immutable contracts cover planner logic. No test writes a package.

Cases:

```text
valid one-reference Live 11.3 experimental package
two occurrences sharing one selected content
unresolved/manual resolution
source equals target
same target filename with different content
unsupported Live version
unsupported RelativePathType
safe project-local Type 3 Path-only relocation
mixed external Type 1 and project-local Type 3 references
unsafe Type 3 traversal path
existing unsupported reference produces no orphan audio copy
unknown or requires-test rewrite support evidence
outdated resolution policy
zero active references in laboratory rewrite mode
missing or mismatched locator/ref
copy-only mode
unchanged input/output determinism
fingerprint order independence for equivalent operations
fingerprint change after write-relevant plan mutation
```

Fingerprint cases use cloned ready plans and never read audio bytes. Reordering
equivalent operations must preserve `PlanFingerprint v0.1`; changing a copy
destination, expected size or rewrite value must change it.
