# Windows Codex Handoff

Status: active operational handoff for the first native Windows laboratory run

This document is for the Codex session opened on the Windows laptop. It is a
bounded execution brief, not a new source of product truth. `CURRENT_STATE.md`
and `PRODUCT_SPINE.md` remain authoritative.

## 1. Your Assignment

Validate the existing Rescue repository on native Windows and collect read-only
evidence from copied Ableton Live 9 and Live 10 projects.

Do not implement Live 9/10 rewrite support. Do not reorganize the architecture.
Do not run `no-mistakes`, Firstmate, subagents, or a full-repository audit. Use
one Codex agent, targeted commands, and the smallest useful report.

The desired outcome is:

```text
repository verified
-> native Windows build and tests executed
-> copied Live 9/10 ALS files analyzed read-only
-> differences and blockers summarized without private data
-> no original project or audio file changed
```

## 2. Mandatory Reading

Read only these files before running commands:

```text
AGENTS.md
CURRENT_STATE.md
PRODUCT_SPINE.md
docs/setup/WINDOWS_TEST_LAB.md
this file
```

Do not bulk-read `docs/archive`, `session-digests`, old audits, or every module
specification. Open one module specification only if a specific failing command
requires it.

## 3. Find Or Recover The Repository

If the user supplied a copied folder, locate its repository root:

```powershell
git rev-parse --show-toplevel
git status --short --branch
git remote -v
```

The expected remote is:

```text
https://github.com/AALogic/als-rewrite-lab.git
```

The reviewed Windows-lab branch and validated code baseline are:

```text
branch: codex/overnight-safe-vertical-slice
validated code baseline: c78102b71eebbfde3b2318284c23bd15c3b30834
PR: https://github.com/AALogic/als-rewrite-lab/pull/1
```

If the copied folder is absent, incomplete, or is not a Git repository, clone
the private canonical repository. Authenticate the `AALogic` GitHub account
first if Git asks for access:

```powershell
cd C:\
mkdir RescueDevelopment -ErrorAction SilentlyContinue
cd RescueDevelopment
git clone https://github.com/AALogic/als-rewrite-lab.git
cd als-rewrite-lab
git fetch --prune origin
git switch --detach origin/codex/overnight-safe-vertical-slice
```

Verify the exact source before testing:

```powershell
git rev-parse HEAD
git status --short --branch
$Baseline = "c78102b71eebbfde3b2318284c23bd15c3b30834"
git merge-base --is-ancestor $Baseline HEAD
if ($LASTEXITCODE -ne 0) { throw "Checkout does not contain the validated baseline" }
python tools/private_path_guard.py
```

Record the actual `HEAD` used for the Windows run. Stop and report a mismatch if
the checkout does not contain the validated baseline. Do not reset, force-push,
merge, or create an unrelated replacement repository.

## 4. Private Test Data Layout

Real ALS and audio data must remain outside Git:

```text
C:\RescueLab\input       copied ALS/projects; never originals
C:\RescueLab\working     disposable build and experiment output
C:\RescueLab\evidence    private raw command output and path-bearing reports
```

Before testing, ask the user to copy selected projects into
`C:\RescueLab\input`. Never move, rename, rewrite, or delete the originals. Do
not commit real `.als`, audio, `.asd`, private paths, or raw reports.

## 5. Native Windows Baseline

Use the Windows-native Codex agent and PowerShell, not WSL. Record privately:

```powershell
git --version
rustup show
rustc --version
cargo --version
python --version
git rev-parse HEAD
```

Also record the Windows edition/build and exact Ableton Live 9/10 versions. Keep
machine names, usernames, serials, and private paths out of committed reports.

Run the existing gates once:

```powershell
cargo fmt --check
cargo test --workspace --locked
cargo clippy --workspace --all-targets --locked -- -D warnings
python tools/private_path_guard.py
python -m unittest discover -s tools/tests -p "test_*.py"
cargo build --release --locked -p rescue-cli
.\target\release\rescue.exe --help
```

If one command fails, diagnose that command only. Do not launch a broad refactor
or rewrite tests to hide a platform defect.

## 6. Ableton Evidence Set

Select five to ten copied projects in total. Prefer coverage over volume:

```text
at least two Live 9 ALS files
at least two Live 10 ALS files
one project with project-local audio
one project referencing another drive or Downloads-like location
one project with a missing sample
one same-filename/different-file case if available
one pre/post Collect All and Save pair if available
WAV, AIFF, and MP3 references where available
```

Assign neutral IDs such as `W9-001`, `W10-001`, and `CAS-PAIR-001`. Do not use
real artist, project, sample, or folder names in a committed summary.

## 7. Commands To Run On Each Copy

Run read-only commands first and save raw output only under the private evidence
folder:

```powershell
$Rescue = ".\target\release\rescue.exe"
$Set = "C:\RescueLab\input\<copied-project>\<copied-set>.als"

& $Rescue analyze $Set
& $Rescue extract $Set
& $Rescue preflight $Set
```

For each fixture, check and record:

```text
command exit status
Ableton document/version fields observed
number of active audio references
path forms and drive-letter behavior
relative versus absolute references
OriginalCrc and size fields when present
warnings, unsupported structures, and parser errors
whether recorded paths currently exist
```

Do not run `lab-package` against Live 9 or Live 10 during this first session.
The current product has no confirmed rewrite profile for those versions. A
refusal is expected; a successful write attempt would be a safety defect.

## 8. Required Result

Create one sanitized summary:

```text
docs/experiments/windows-live9-10-readonly-<date>.md
```

It must contain:

```text
tested commit SHA
Windows version and architecture
Ableton version/build groups
fixture IDs and coverage categories
commands executed
pass/fail table
structural differences observed between Live 9, Live 10, and supported fixtures
failures or unknowns requiring a focused experiment
confirmation that originals were not modified
recommended next smallest task
```

Before committing, run:

```powershell
python tools/private_path_guard.py
git diff --check
git status --short
```

Keep raw output in `C:\RescueLab\evidence`; commit only the sanitized summary.
If the user wants to share it through Git, use a machine-specific branch:

```powershell
git switch -c codex/windows-live9-10-evidence
git add docs/experiments/windows-live9-10-readonly-<date>.md
git commit -m "docs: record Windows Live 9 and 10 read-only evidence"
```

Do not push until the user reviews the sanitized diff.

## 9. Stop Conditions

Stop immediately and report rather than improvising when:

```text
the repository does not contain the validated baseline
a command attempts to write beside an original project
a real ALS/audio/private report appears in Git status
Live 9/10 reaches a write-capable stage
the parser encounters an unsupported structure that would require guessing
the copied data is insufficient to distinguish a hypothesis
```

The first Windows session ends after the sanitized read-only report. It does
not implement fixes, rewrite rules, indexing, UI, packaging, or cleanup unless
the user starts a separate, explicitly scoped task.
