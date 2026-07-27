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
unknown or requires-test rewrite support evidence
outdated resolution policy
missing or mismatched locator/ref
copy-only mode
unchanged input/output determinism
```
