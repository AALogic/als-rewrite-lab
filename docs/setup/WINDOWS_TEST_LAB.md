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

## Canonical Repository Release Gate

The canonical repository must remain private. Before another laptop receives a
clone, its maintainer must complete all of the following outside this test lab:

1. Run `python3 tools/private_path_guard.py` against the publication tree.
2. Rewrite every reachable Git ref that contains real ALS/audio data or private
   absolute paths; deleting a current-tree file is not enough.
3. Remove or quarantine pre-scrub remote refs, tags, and clones that still make
   the old objects reachable.
4. Create a fresh clone from the canonical remote, inspect all advertised refs,
   rerun the private-path guard, and record the verified commit SHA.

This review worktree does not rewrite canonical history. Windows handoff remains
blocked until the maintainer records that the history scrub and fresh-clone
verification passed.

### Blocked History Recovery

If `python3 tools/private_path_guard.py` reports a `reachable_*` violation,
stop the active gate without pushing, tagging, or cloning onto the Windows
laptop. The `<reachable-object-...>` marker contains the full Git object ID but
does not print the private value.

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

Use each reported object ID to locate every commit that introduced or removed
it:

```powershell
git log --all --find-object=<full-object-id> --oneline
```

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
python tools/private_path_guard.py |
  Tee-Object C:\RescueLab\evidence\fresh-clone-private-path-guard.txt
if ($LASTEXITCODE -ne 0) { throw "Canonical history remains unsafe" }
git rev-parse HEAD |
  Tee-Object C:\RescueLab\evidence\fresh-clone-verified-commit.txt
git for-each-ref --format="%(objectname) %(refname)" refs/privacy-verification |
  Set-Content C:\RescueLab\evidence\fresh-clone-advertised-refs.txt
```

The three evidence files, for the same reviewed commit, are the release-gate
record. Only a full `PASS` may unblock the Windows clone and later publication
steps.

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
