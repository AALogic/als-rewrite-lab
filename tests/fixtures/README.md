# Fixture Policy

The committed regression suite generates small synthetic gzip/XML `.als` files
at runtime. Core parser and extractor tests therefore run in clean clones,
worktrees and CI without private Ableton projects.

Real-world `.als` files may still be used as a private, optional evidence corpus.
They contain private paths and project metadata, so `.gitignore` keeps Ableton
and audio binaries untracked.

Rules for private evidence:

1. keep the corpus outside the repository or in ignored local paths;
2. store only hashes, expected counts and sanitized evidence in Git;
3. review fixture licensing and privacy before sharing it;
4. never make core tests depend on the private corpus;
5. use private files for additional regression and Ableton-behavior research.

Evidence recorded on 2026-07-26:

```text
A clean worktree previously failed 8 tests because ignored private `.als` files
were absent. The tests now generate equivalent, minimal contract fixtures and
must pass without any private files.
```
