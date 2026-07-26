# Local Fixture Policy

The current `.als` fixtures are available locally so the existing regression
suite can run, but they contain private paths and project metadata. `.gitignore`
therefore keeps Ableton and audio binaries untracked.

Before the first shared commit or remote push:

1. replace core parser cases with synthetic gzip/XML `.als` fixtures;
2. keep any approved real-world corpus outside the repository;
3. store only hashes, expected counts and sanitized evidence for private files;
4. review fixture licensing and privacy explicitly;
5. make CI generate or receive an approved fixture pack without embedding user
   projects in the source repository.

Do not remove the local files until equivalent synthetic regression coverage
exists.
