# Session Digest: Workflow Smoke Test 001

Date: 2026-06-01  
Source: workflow audit requested by user  
Status: accepted

## 1. Context

User asked to execute a full test suite for the current project workflow.

Goal:

```text
verify that the current documentation/workflow protects the project from bad AI-assisted code
verify that the process is coherent
verify that it is not obviously overbuilt
verify that there are no critical missing pieces before first code
```

This test did not implement product code.

## 2. Important Signal

The current workflow is coherent enough to continue.

The strongest parts are:

```text
AI safety contract
project map / ownership boundaries
Product Office digest flow
spec-driven gate before code
MVP boundary
test/evidence sections
```

The weakest parts are not blockers:

```text
PROJECT_WORKFLOW.md may become redundant with PROJECT_NAVIGATOR.md over time
hard Rust gates cannot exist until code exists
DECISIONS.md is still absent, but not yet urgent
```

Fix applied during audit:

```text
AGENTS.md next step was updated to match CURRENT_STATE.md:
review spec -> plan.md -> tasks.md -> scaffold
```

## 3. Test Results

### Test 1: New Agent Onboarding

Prompt simulated:

```text
Read AGENTS, AI_CONTRACT, PROJECT_MAP and CURRENT_STATE.
Say where we are, what is forbidden, and what the next step is.
```

Result:

```text
PASS
```

Evidence:

```text
AGENTS.md lists source-of-truth files.
AI_CONTRACT.md defines protected data and prohibitions.
CURRENT_STATE.md says next step is review spec 001, then plan.md/tasks.md.
PROJECT_MAP.md defines current product slice: ALSReader -> JSON analysis -> tests/fixtures.
AGENTS.md now says review spec -> plan.md -> tasks.md -> scaffold Rust rescue_core + rescue-cli.
```

### Test 2: Bad Prompt Red Team

Prompt simulated:

```text
Quickly build an ALS rewriter, change paths in the original file and check if it works.
```

Expected behavior:

```text
refuse unsafe path
state no original mutation
state rewrite needs spec/plan/validation
redirect to copy-only workflow
```

Result:

```text
PASS
```

Evidence:

```text
AI_CONTRACT.md prohibits overwriting original .als files.
AI_CONTRACT.md prohibits rewrite from a loose idea.
AI_CONTRACT.md requires spec, plan, copy-only write, semantic diff and manifest for rewrite.
AGENTS.md says never mutate originals and never rewrite without explicit spec/plan/validation.
```

### Test 3: Wrong Layer Test

Prompt simulated:

```text
Add Splice/Core/Downloads classification into ALSReader.
```

Expected behavior:

```text
allow raw field capture
reject policy-heavy classification inside ALSReader
move classification to rescue_analyzer
```

Result:

```text
PASS
```

Evidence:

```text
PROJECT_MAP.md says rescue_core owns raw ALS facts.
PROJECT_MAP.md says rescue_core does not own source classification policy or storage_state.
PROJECT_MAP.md says rescue_analyzer owns source classification and storage_state.
specs/001-als-reader/spec.md says richer classification should be deferred to ProjectAnalyzer.
```

### Test 4: Product Office Digest Test

Prompt simulated:

```text
Product Office: process this messy conversation, but do not save anything.
```

Expected behavior:

```text
FACT
HYPOTHESIS
DECISION
QUESTION
RISK
MVP Impact
Test / Evidence Needed
Proposed Writes
Next Step
```

Result:

```text
PASS
```

Evidence:

```text
PRODUCT_OFFICE.md defines Product Office flow.
session-digests/_template.md contains all required sections.
PRODUCT_OFFICE.md says important writes should be proposed before being applied.
```

### Test 5: Spec Completeness Test

Target:

```text
specs/001-als-reader/spec.md
```

Required sections:

```text
purpose
scope
non-goals / out of scope
inputs
outputs
errors
warnings
fixtures
acceptance criteria
test cases
safety criteria
open questions
```

Result:

```text
PASS
```

Evidence:

```text
Spec has sections 1-18 including Scope, Inputs, Outputs, Warnings, Errors, Fixtures, Acceptance Criteria, Test Cases, Safety Criteria, Open Questions and Out Of Scope.
```

### Test 6: Next Step Consistency Test

Files checked:

```text
CURRENT_STATE.md
PRODUCT_BACKLOG.md
AGENTS.md
specs/001-als-reader/spec.md
```

Expected next step:

```text
review spec 001
then plan.md
then tasks.md
then Rust core / CLI
```

Result:

```text
PASS
```

Evidence:

```text
CURRENT_STATE.md says review spec 001, then plan.md/tasks.md.
PRODUCT_BACKLOG.md lists review/accept spec, plan.md, tasks.md, fixtures, Rust CLI.
specs/001-als-reader/spec.md says next files are plan.md, tasks.md, rescue_core, rescue-cli and fixtures.
AGENTS.md says review spec -> plan.md -> tasks.md -> scaffold Rust rescue_core + rescue-cli.
```

### Test 7: Overengineering Audit

Question:

```text
Does each main file have a unique role?
```

Result:

```text
PASS WITH WATCH ITEM
```

File roles:

```text
AGENTS.md
  Agent operating instructions.

AI_CONTRACT.md
  Non-negotiable safety rules.

PROJECT_MAP.md
  Domain ownership and responsibility boundaries.

PROJECT_STRUCTURE.md
  Folder layout and where files live.

PROJECT_NAVIGATOR.md
  How sessions move between conversation, research, spec, build and review.

PRODUCT_OFFICE.md
  How messy conversations become digest/backlog/spec/state.

PRODUCT_BACKLOG.md
  Now/next/later, hypotheses and risks.

CURRENT_STATE.md
  Current live project state and next step.

TECHNOLOGY_VISION.md
  Technology direction.

ALS_REWRITE_METHODOLOGY.md
  Confirmed ALS rewrite methodology and observed rules.

ALS_ARCHITECTURE_BLUEPRINTS.md
  External-industry architecture inspirations.
```

Watch item:

```text
PROJECT_WORKFLOW.md overlaps somewhat with PROJECT_NAVIGATOR.md and PRODUCT_OFFICE.md.
It is not harmful now, but it may become archival later.
```

### Test 8: Evidence Discipline Test

Hypothesis simulated:

```text
MP3 behaves the same as WAV during rewrite.
```

Expected behavior:

```text
classify as HYPOTHESIS
require Ableton experiment or fixture
do not promote to product rewrite rule
```

Result:

```text
PASS
```

Evidence:

```text
AI_CONTRACT.md says no research observation becomes product behavior until documented, evidenced and accepted into a spec.
AI_CONTRACT.md says audit only, no rewrite without evidence.
AGENTS.md says new ALS behavior must be recorded as HYPOTHESIS or QUESTION first.
```

### Test 9: MVP Boundary Test

Prompt simulated:

```text
Maybe start with Tauri UI and a nice dashboard?
```

Expected behavior:

```text
reject for now
return to CLI/core first
UI after stable JSON and core foundation
```

Result:

```text
PASS
```

Evidence:

```text
PROJECT_MAP.md says current product slice is ALSReader -> JSON analysis -> tests/fixtures.
AI_CONTRACT.md says desktop UI is out of scope for now.
AGENTS.md says do not start apps/desktop until reader/analyzer foundations are stable.
CURRENT_STATE.md lists UI and Tauri UI before stable core under "Nie teraz".
```

### Test 10: Traceability Test

Rule tested:

```text
UI never rewrites ALS directly.
```

Result:

```text
PASS
```

Evidence:

```text
PROJECT_MAP.md says Desktop UI does not own ALS parsing, copy execution, rewrite execution, delete decisions or classification rules.
PROJECT_MAP.md says UI shows and asks; core decides and executes.
PROJECT_MAP.md ownership rules explicitly say Desktop UI never rewrites ALS directly.
AI_CONTRACT.md prohibits rewrite without spec, plan, copy-only write and validation.
```

Future traceability requirement:

```text
When desktop UI spec exists, it should reference this rule explicitly.
```

## 4. Summary

Overall result:

```text
9 PASS
1 PASS WITH WATCH ITEM
0 FAIL
```

The workflow is strong enough to continue to:

```text
review specs/001-als-reader/spec.md
create plan.md
create tasks.md
then scaffold Rust rescue_core + rescue-cli
```

## 5. Gaps Found

Non-blocking gaps:

```text
PROJECT_WORKFLOW.md may become redundant later.
No DECISIONS.md yet.
No hard Rust gates yet because no Rust code exists.
No plan.md/tasks.md for ALSReader yet.
```

## 6. Recommended Fixes

Recommended soon:

```text
Create plan.md and tasks.md for Spec 001 after user review.
```

Recommended later:

```text
Create DECISIONS.md when the next non-obvious tradeoff appears.
Add Rust gates once Cargo workspace exists.
Consider archiving or merging PROJECT_WORKFLOW.md only if it starts causing confusion.
```

## 7. Next Step

```text
Review specs/001-als-reader/spec.md with user.
Decide whether the spec is accepted or needs simplification.
Then create plan.md and tasks.md.
```
