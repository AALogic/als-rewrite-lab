# Session Digest: Software Process Meta Review

Date: 2026-06-10

## 1. Source

Input reviewed:

- pasted conversation and GPT meta-analysis from
  `/Users/tru.siak/.codex/attachments/0cfc33d5-88c1-4015-a688-009f94cf9b18/pasted-text.txt`
- current project files in `/Users/tru.siak/Documents/New project/als-rewrite-lab`

Local checks performed:

```text
python3 tools/workflow_guard.py verify-module 001-als-reader
python3 tools/workflow_guard.py verify-module 002-dependency-extractor
cargo test --workspace
wc -l key docs and source files
find .DS_Store / target / zip artifacts
inspect module.contract.json source coverage
inspect absence of Makefile / Justfile
inspect string-heavy public models
```

Observed local verification:

```text
verify-module 001-als-reader: PASS
verify-module 002-dependency-extractor: PASS
cargo test --workspace: PASS, 24 tests passed
```

## 2. Honest Verdict

The GPT review is broadly correct.

The project is not just a pile of Markdown. It already has a real first version
of executable governance:

```text
specs
module.contract.json
Rust tests
negative tests
workflow_guard.py
dependency allowlists
public contract checks
forbidden fields / forbidden source patterns
closeout notes
```

That is a strong direction.

The main correction is this:

```text
The system is good enough to continue building product modules, but not mature
enough to trust every PASS as full project health.
```

The workflow is ahead of the product code. That is acceptable at this stage,
but it creates a new risk: the meta-system can become heavier than the product.

## 3. What Is Strong

### 3.1 The project has moved beyond vibe coding

The actual workflow is now close to:

```text
conversation
-> spec
-> plan
-> tasks
-> fixture contract
-> module.contract.json
-> workflow_guard module-ready
-> tests
-> code
-> workflow_guard verify-module
-> closeout
```

This is the right pattern for AI-assisted coding in a high-risk local data
product.

### 3.2 Safety philosophy is right

The product handles Ableton projects and user audio files. The correct default
is conservative:

```text
read originals
copy before modifying
never rewrite original ALS
never delete original audio
do not guess ambiguous matches
wrong match is worse than failed match
```

This is one of the strongest parts of the project.

### 3.3 Module boundaries are good

Current boundaries are healthy:

```text
ALSReader:
  reads ALS
  emits ALSReadModel
  does not verify filesystem
  does not match
  does not rewrite

DependencyExtractor:
  consumes ALSReadModel
  emits DependencyExtractionResult
  does not verify filesystem
  does not classify
  does not match
  does not plan copy/rewrite
```

This directly counters a common AI failure mode: doing too much "to be helpful".

### 3.4 Tests check forbidden behavior

Good examples already exist:

```text
als_reader_v0_2_does_not_check_filesystem_paths
read_only_safety
forbidden_downstream_fields_are_absent
duplicate_refs_are_not_deduplicated
output_is_deterministic
```

This is important because AI-assisted development needs tests for "must not",
not only tests for "must".

## 4. Confirmed Weak Points

### 4.1 Guard coverage can create false confidence

This is the most important finding.

Current `module.contract.json` files use `expected_source_files` for public
contract and quality limits.

Observed:

```text
001 expected_source_files:
  crates/rescue_core/src/als_reader.rs
  crates/rescue_core/src/models.rs

001 implementation file not included:
  crates/rescue_core/src/als_reader_impl.rs

002 expected_source_files:
  crates/rescue_core/src/dependency_extractor.rs

002 implementation file not included:
  crates/rescue_core/src/dependency_extractor_impl.rs
```

The guard still scans all core source for high-risk patterns through
`guarded_source_globs`, so destructive operations are partly covered.

But quality limits such as max file length and max function length apply only
to `expected_source_files`, not to implementation files.

Result:

```text
verify-module can pass while large implementation files are not checked by
quality limits.
```

This is not a catastrophic bug. It is a real example of why executable
governance must itself be tested and improved.

### 4.2 No project-level verification command

Current state:

```text
verify-module exists
verify-project does not exist
Makefile / Justfile does not exist
```

This means project health is still a sequence of commands remembered by Codex
or the user.

Better target:

```text
make verify
```

or:

```text
just verify
```

or:

```text
python3 tools/workflow_guard.py verify-project
```

One command should define "the project is healthy".

### 4.3 Too much context can become drag

The repo has many high-level documents:

```text
PRODUCT_SPEC.md
PRODUCT_SPINE.md
PRODUCT_OFFICE.md
PROJECT_NAVIGATOR.md
PROJECT_WORKFLOW.md
PROJECT_MAP.md
PROJECT_STRUCTURE.md
CURRENT_STATE.md
ENGINEERING_RULES.md
AI_CONTRACT.md
```

This is understandable because the project was born through AI conversations.
These files are not useless. They solved real problems.

The risk is that the agent or user may not know which file is authoritative for
which question.

The next improvement should be context routing, not more context.

### 4.4 Public models are string-heavy

The Rust models still use many strings for domain concepts:

```text
severity: String
evidence_status: String
rewrite_support_status: String
source_kind: String
dependency_kind: String
extraction_status: String
path_basis: String
```

This is acceptable for early JSON contracts, but risky long term:

```text
typos become possible
status sets are not compiler-checked
downstream modules may depend on undocumented values
```

Do not rewrite everything now. Start introducing enums from new modules onward,
especially in `003-path-verifier`.

### 4.5 Repository hygiene needs attention

Observed in project folder:

```text
.DS_Store
experiments/.DS_Store
specs/.DS_Store
tests/.DS_Store
tools/.DS_Store
target/
target/.DS_Store
Archiwum.zip
```

There is no visible `.gitignore` in or near the project.

This matters for AI work because generated artifacts and system files pollute
context and can confuse analysis.

## 5. What GPT Got Right

The strongest parts of GPT's review:

```text
Markdown is not enough.
Executable governance is the right next maturity step.
workflow_guard.py is valuable but only as good as its configuration.
verify-project is now more important than another large document.
Context must be routed, not endlessly accumulated.
The project should not rush into ALS rewrite.
PathVerifier -> classifier/preflight -> planner/stager/validator should come first.
```

I agree with these points.

## 6. What Needs Nuance

### 6.1 The project is not over-engineered yet, but it is close to the line

The amount of process was justified because:

```text
the product can damage valuable user files
the user is discovering software process while building
AI coding is prone to scope creep
ALS behavior is partly reverse-engineered
```

But from this point onward, new process layers need a strict justification.

Rule:

```text
Add a new document/guard/process only if it:
1. protects user data,
2. blocks a real or highly likely AI failure,
3. speeds up future implementation,
4. improves product verification.
```

### 6.2 Code quality score is "early", not "bad"

The product code is young, but the first two modules are coherent.

The right interpretation:

```text
ALSReader and DependencyExtractor are good MVP core modules.
They are not final architecture.
```

The next modules should raise the bar by adding stronger typed status models
and filesystem adapters.

## 7. Recommended Action Plan

### Priority 0: Do not lose momentum on 003

Do not pause product building for a large meta-refactor.

Before implementing `003-path-verifier`, do only the small process fixes that
directly protect the next module.

### Priority 1: Fix source coverage in module contracts

Add implementation source tracking.

Recommended contract shape:

```json
{
  "public_source_files": [],
  "model_source_files": [],
  "implementation_source_files": [],
  "expected_source_files": []
}
```

Or simpler first step:

```text
include *_impl.rs in expected_source_files
```

But better is to teach `workflow_guard.py` that public contract checks apply to
public/model files while quality limits apply to implementation files too.

### Priority 2: Add one project verification command

Add either:

```text
Makefile
```

or:

```text
Justfile
```

or enhance:

```text
tools/workflow_guard.py verify-project
```

Minimum project verification:

```text
verify-module 001-als-reader
verify-module 002-dependency-extractor
cargo fmt --check
cargo check --workspace
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

### Priority 3: Add project hygiene rules

Add `.gitignore` or project hygiene guard for:

```text
target/
.DS_Store
__MACOSX/
*.zip
```

Do not delete files casually in the same step unless the user approves cleanup.

### Priority 4: Use enums for new module statuses

For `003-path-verifier`, prefer enums for:

```text
PathKind
PathVerificationStatus
CandidateCheckStatus
SizeMatchStatus
PlatformPathKind
PathVerificationWarningCode
PathVerificationErrorCode
```

Serialize them as strings for JSON if useful, but keep the Rust code typed.

### Priority 5: Add context routing rather than more docs

Add a short "context routing" table to `AGENTS.md` or `PROJECT_NAVIGATOR.md`.

Example:

```text
Coding a module:
  AGENTS.md
  CURRENT_STATE.md
  ENGINEERING_RULES.md
  AI_CONTRACT.md
  specs/<module>/*

Strategic product review:
  PRODUCT_SPINE.md
  PRODUCT_SPEC.md
  PRODUCT_BACKLOG.md
  session-digests/

ALS research:
  experiments/
  ALS_REWRITE_METHODOLOGY.md
  ALS_ARCHITECTURE_BLUEPRINTS.md
```

## 8. What Not To Do Now

Do not add a large state machine runtime yet.

Do not split into multi-agent workflow yet.

Do not rewrite all models into enums in one large refactor.

Do not create many new process documents.

Do not jump to ALSRewriter.

Do not treat `verify-module: PASS` as full project health until `verify-project`
exists.

## 9. Recommendation For Immediate Next Step

Before continuing `003-path-verifier`, do a small "guard hardening" step:

```text
1. Teach workflow_guard.py about implementation_source_files.
2. Update 001 and 002 module.contract.json to include implementation files.
3. Add a simple project-level verify command or script.
4. Add .gitignore / hygiene rule for generated artifacts.
```

Then return to the paused `003` interview at:

```text
Block 7: PathVerifier tests
```

This is a small enough process improvement to pay for itself immediately,
because `003` is the first filesystem-touching module.

## 10. Final Assessment

The project direction is strong.

The main danger is no longer "AI writes random code". The current system already
reduces that risk.

The main danger now is subtler:

```text
The project may believe it is fully guarded because module checks pass, while
some real implementation or project-level risks are outside the guard.
```

The cure is not more Markdown.

The cure is:

```text
small executable checks
typed contracts
one verify command
better source coverage
context routing
continuing product modules in small slices
```

