# Windows Test Laboratory

Status: setup procedure for native Windows compatibility and Ableton evidence

## Purpose

Use one Windows laptop to run the same committed Rescue code against copied
Ableton projects. The laptop is a test node, not an independent product fork.

This laboratory should answer two different questions:

1. Does the Rust workspace compile and behave correctly on native Windows?
2. What facts and rewrite rules differ in ALS documents produced by Live 9 or
   Live 10?

The second question requires Ableton experiments. Passing Rust tests alone is
not evidence that rewrite is supported for an older Live version.

## Safety Boundary

```text
real projects                 read-only source
C:\RescueLab\input            private copied test inputs
C:\RescueLab\working          disposable generated work
C:\RescueLab\evidence         private raw reports
Git repository               code and sanitized evidence only
```

Rules:

- Never run write-capable experiments against original projects.
- Never add real ALS, audio, `.asd`, absolute path dumps, or private ledgers to
  Git.
- Keep Codex in the Windows sandbox with approval enabled.
- Grant access to the repository and `C:\RescueLab` only.
- Do not use WSL for native path and filesystem evidence.
- Preserve the tested commit hash in every report.

## Supported Host Baseline

Prefer Windows 11. A recent, fully updated Windows 10 can be used as a best
effort test host. Record the exact Windows build with `winver` before testing.

Use the Windows-native Codex agent and PowerShell. WSL is useful for Linux
development, but it does not prove native drive-letter, NTFS, junction, path,
or promotion behavior.

## Prerequisites

Install:

1. Git for Windows.
2. Rustup with the stable MSVC host.
3. Visual Studio Build Tools with the `Desktop development with C++` workload.
4. Python 3 for the workflow-guard tests.
5. Codex or the ChatGPT Windows app when agent-assisted execution is desired.

Confirm in a new PowerShell window:

```powershell
git --version
rustup --version
python --version
```

## Private Preparatory Handoff Gate

The canonical repository must remain private. Before a private preparatory PR or
read-only Windows laboratory handoff, run:

```powershell
python tools/private_path_guard.py
```

This mandatory preparatory check scans the staged index and untracked worktree.
Keep the remote private and record the reviewed commit SHA. Legacy private data
that is reachable only from older Git history does not block this private,
read-only handoff.

## Public Or Commercial Release History Audit

Before any public or commercial release, run the explicit full-history mode in
a fresh, non-shallow clone containing every advertised ref:

```powershell
python tools/private_path_guard.py --release-history
```

This review worktree does not rewrite canonical history. Windows handoff remains
private and preparatory while legacy reachable-history findings remain. Public
or commercial release remains blocked until the maintainer records that the
history scrub and fresh-clone verification passed.

### Blocked Release-History Recovery

If `python3 tools/private_path_guard.py --release-history` reports a
`reachable_*` violation, stop the public or commercial release. This does not
block a private preparatory PR or read-only Windows laboratory clone. The
`<reachable-object-...>` marker contains the full Git object ID but does not
print the private value.

History rewriting is a separate, coordinated maintenance operation between
gate runs. In a dedicated maintenance clone, fetch every advertised namespace
and preserve a private ref inventory before changing anything:

```powershell
$Maintenance = "C:\RescueLab\working\als-rewrite-lab-history-scrub"
git clone --mirror https://github.com/AALogic/als-rewrite-lab.git $Maintenance
cd $Maintenance
git fetch --force --prune origin "+refs/*:refs/*"
git for-each-ref --format="%(objectname) %(refname)" |
  Set-Content C:\RescueLab\evidence\pre-scrub-refs.txt
```

Classify each reported object ID before locating its history. Regular-file path
and tracked-media violations report a blob ID. Private commit or annotated-tag
text reports that commit or tag ID; snapshot-only metadata such as a gitlink
path reports the containing commit:

```powershell
$ObjectId = "<full-object-id>"
$ObjectType = git cat-file -t $ObjectId
if ($ObjectType -eq "blob") {
  git log --all --find-object=$ObjectId --oneline
} elseif ($ObjectType -eq "commit") {
  git show --name-status --format=fuller $ObjectId
  git branch --all --contains $ObjectId
  git tag --contains $ObjectId
} elseif ($ObjectType -eq "tag") {
  git show --no-patch $ObjectId
} else {
  throw "Unexpected reported Git object type: $ObjectType"
}
```

For a blob, review every commit returned by `--find-object`, including both its
introduction and removal. For a commit or tag, inspect the reported object and
every ref that contains it. Keep the command output private because local
history inspection can reveal the value that the guard intentionally redacts.

Rewrite every affected canonical branch and tag with a reviewed history-rewrite
procedure that removes prohibited ALS/audio blobs and replaces private text.
Then delete obsolete remote refs, force-update only the reviewed sanitized
refs, and quarantine every pre-scrub clone. If the host still advertises a PR,
tag, or other ref that reaches an old object, publication remains blocked until
that ref is removed or the host confirms that the sensitive-data purge is
complete. Do not improvise the rewrite inside a no-mistakes worktree.

After the rewrite, verify from a new non-shallow clone. The explicit fetch maps
every advertised remote ref into a local verification namespace so the guard's
all-ref traversal includes it:

```powershell
$Verification = "C:\RescueLab\working\als-rewrite-lab-fresh-verification"
git clone https://github.com/AALogic/als-rewrite-lab.git $Verification
cd $Verification
git fetch --force --prune origin "+refs/*:refs/privacy-verification/*"
if ((git rev-parse --is-shallow-repository) -ne "false") {
  throw "Fresh-clone verification is shallow"
}
git switch --detach refs/privacy-verification/heads/codex/overnight-safe-vertical-slice
python tools/private_path_guard.py --release-history |
  Tee-Object C:\RescueLab\evidence\fresh-clone-private-path-guard.txt
if ($LASTEXITCODE -ne 0) { throw "Canonical history remains unsafe" }
git rev-parse HEAD |
  Tee-Object C:\RescueLab\evidence\fresh-clone-verified-commit.txt
git for-each-ref --format="%(objectname) %(refname)" refs/privacy-verification |
  Set-Content C:\RescueLab\evidence\fresh-clone-advertised-refs.txt
```

The three evidence files, for the same reviewed commit, are the public/commercial
release-gate record. Only a full `PASS` may unblock those release steps.

## Clone The Canonical Repository

```powershell
$ReviewRef = "origin/codex/overnight-safe-vertical-slice"
cd C:\
mkdir RescueDevelopment
cd RescueDevelopment
git clone https://github.com/AALogic/als-rewrite-lab.git
cd als-rewrite-lab
git fetch --tags --prune
git switch --detach $ReviewRef
git status --short --branch
git rev-parse HEAD
```

The checkout should be detached at the review branch commit. Stop if the review
ref is absent or if its SHA differs from the commit selected for the lab. Do not
develop directly from this detached checkout.

`lab-v0.1.0-rc.1` is a governed release-candidate tag, not a prerequisite for
testing the review branch. The canonical maintainer may create it only after the
no-mistakes gate, macOS and Windows CI, workflow contracts, private-path guard,
and history-scrub verification all pass for the same reviewed commit. After the
tag exists, a fresh lab checkout may use:

```powershell
git fetch --tags --prune
git switch --detach lab-v0.1.0-rc.1
git rev-parse HEAD
```

## Verify And Build

```powershell
rustup show
cargo fmt --check
cargo test --workspace --locked
cargo clippy --workspace --all-targets --locked -- -D warnings
python tools/private_path_guard.py
python -m unittest discover -s tools/tests -p "test_*.py"
cargo build --release --locked -p rescue-cli
.\target\release\rescue.exe --help
```

Do not continue to Ableton data if repository verification fails.

## First Evidence Run

Start with five to ten copied projects that cover different cases:

- project-local audio;
- audio on another drive;
- a missing sample;
- two different samples with the same filename;
- WAV, AIFF, and MP3;
- a project before and after Ableton `Collect All and Save`;
- a larger real project with many references.

For Live 9 and Live 10, run read-only commands first:

```powershell
.\target\release\rescue.exe analyze C:\RescueLab\input\example.als
.\target\release\rescue.exe extract C:\RescueLab\input\example.als
.\target\release\rescue.exe preflight C:\RescueLab\input\example.als
```

The current laboratory packaging command supports only its confirmed Live 11.3
rewrite profile. A Live 9 or Live 10 request must stop before writing. Treat a
refusal as the correct result until version-specific evidence and tests exist.

## Sharing Results

Create one branch per machine and purpose, for example:

```powershell
git switch -c codex/windows-live10-evidence
```

Share only a sanitized Markdown summary and small machine-readable report. Each
report must include:

```text
tested commit SHA
Windows edition and build
CPU architecture
Ableton version and build
fixture identifiers, not private project names
commands run
pass, fail, blocked, or unknown result
observed ALS structure differences
remaining manual checks
```

Do not turn an observation into a rewrite rule on the Windows branch. Submit the
evidence for review; update product rules, specifications, fixtures, tests, and
code together in a separate implementation change.
