# Local Fixture Policy

The current `.als` fixtures are available locally so the existing regression
suite can run, but they contain private paths and project metadata. `.gitignore`
therefore keeps Ableton and audio binaries untracked.

Before a remote push, remote CI or delegated work in disposable worktrees:

1. replace core parser cases with synthetic gzip/XML `.als` fixtures;
2. keep any approved real-world corpus outside the repository;
3. store only hashes, expected counts and sanitized evidence for private files;
4. review fixture licensing and privacy explicitly;
5. make CI generate or receive an approved fixture pack without embedding user
   projects in the source repository.

Evidence recorded on 2026-07-26:

```text
A clean local clone compiled successfully but failed 8 ALSReader fixture tests
because the ignored private `.als` files were absent. Passing tests in the
primary checkout therefore do not yet prove that CI or agent worktrees are
reproducible.
```

Do not remove the local files until equivalent synthetic regression coverage
exists.
