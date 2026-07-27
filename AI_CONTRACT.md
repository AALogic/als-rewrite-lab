# AI Contract

Status: active safety contract
Date: 2026-07-27
Scope: non-negotiable rules for AI-assisted work on this project

## 1. Purpose

This contract protects the project from two failures:

```text
coding before understanding
modifying user data unsafely
building locally correct code that misses product intent
```

It applies to Codex and any future AI agent working in this repository.

## 2. Highest-Level Rule

```text
User data safety is more important than feature progress.
```

For this project, a failed operation is acceptable.

A wrong operation that silently damages a project is not acceptable.

## 3. Protected User Data

Treat these as protected:

```text
original .als files
original Ableton project folders
audio samples
recordings
Downloads audio files
Splice/cache files
User Library files
Core Library references
project backups
manifest history
experiment snapshots
```

Protected data must not be modified unless the user explicitly asks for a specific operation and the operation is covered by a spec/plan.

## 4. Non-Negotiable Prohibitions

Never:

```text
overwrite an original .als
delete original audio files
delete user project folders
rewrite ALS from a loose idea
rewrite ALS without semantic validation
perform unrestricted global search/replace inside ALS XML
silently choose between ambiguous sample matches
trust Ableton OriginalCrc as sole file identity
change SourceContext/OriginalFileRef in v0.1
modify plugins, presets or device state
touch Ableton Core Library refs
touch User Library refs without a specific supported rule
claim safety without evidence
```

## 5. Allowed Safe Work

Allowed without special approval:

```text
read files
inspect ALS structure
create documentation
create specs
create test plans
create analysis reports
copy files into experiment folders
write code that reads ALS but does not modify inputs
generate JSON analysis output
```

Still required:

```text
do not expose private paths in user-facing output unless relevant
do not move/delete original files
do not confuse experiment scripts with product code
```

## 6. Rewrite Rules

Any ALS rewrite must satisfy all:

```text
operation is described in a spec
operation has a plan
operation writes a copy, not original
fields to change are explicitly listed
semantic diff is checked
all expected new non-core paths exist
unexpected XML changes block the operation
manifest/report is written
user can open result in Ableton for verification
```

Implemented rewrite code remains laboratory-only until its active module spec,
evidence gates, and manual Ableton verification say otherwise. Exact supported
profiles and current readiness live in `CURRENT_STATE.md` and the active
rewriter/pipeline specifications; implementation alone is not a support claim.

## 7. Matching Rules

File matching must prefer safety over convenience.

Allowed confidence sources:

```text
exact existing path
content hash
file size plus hash
controlled fixture evidence
explicit user selection
```

Weak hints:

```text
filename only
Ableton OriginalCrc only
duration only
sample rate only
folder name only
```

Rule:

```text
Ambiguous match blocks auto-rewrite.
```

Worst failure:

```text
wrong sample match
```

Not worst failure:

```text
failed match with clear warning
```

## 8. Research vs Product

Research may explore.

Product must be conservative.

Research outputs:

```text
experiments/
session-digests/
PRODUCT_BACKLOG.md
hypotheses
questions
rule candidates
```

Product outputs:

```text
specs/
crates/
cli/
tests/
validated reports
```

Do not promote a research observation into product behavior until it is:

```text
documented
tested or otherwise evidenced
accepted into a spec
```

## 9. MVP Boundary

`PRODUCT_SPINE.md` is the sole owner of the active MVP boundary, and
`CURRENT_STATE.md` owns implemented status and current blockers. This contract
does not duplicate those changing lists.

Regardless of MVP position, no implemented module weakens the prohibitions,
rewrite gates, matching rules, or evidence requirements in this file.

## 10. Stop Conditions

Stop and ask or switch to research mode if:

```text
ALS structure is not recognized
SampleRef context is unknown
target path is missing
matching is ambiguous
operation would touch originals
test contradicts spec
new behavior would require changing safety rules
user asks a strategy question rather than implementation task
module can pass local tests but Product Spine traceability is unclear
downstream consumer needs are unknown and AI would need to guess
```

## 11. Evidence Rule

Every confirmed ALS rule should be backed by at least one:

```text
before/after Ableton experiment
fixture test
semantic diff report
manual Ableton-open verification
explicit documented decision
```

If there is no evidence:

```text
audit only, no rewrite
```

## 12. Review Rule

After any implementation:

```text
summarize changed files
state tests run
state risks remaining
update CURRENT_STATE if direction changed
do not pretend untested behavior is verified
```

## 13. Refactor Rule

Refactor means:

```text
change code structure without changing behavior
```

During a refactor:

```text
do not change tests
do not change fixtures
do not change expected output
do not change product behavior
```

Allowed during refactor:

```text
rename internal functions if public contract is preserved
extract helpers
simplify code
improve module boundaries
remove duplication
```

If a test must change, stop the refactor and treat it as a separate contract/spec change.

Rule:

```text
test changes and refactor changes must not be mixed unless user explicitly approves a separate test-contract update
```
