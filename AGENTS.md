# AGENTS

Status: active project instructions
Date: 2026-07-27
Scope: how Codex works inside this repository

## 1. Project Identity

This repository builds a local Ableton project dependency safety tool.

Direction:

```text
Rust core
CLI first
desktop UI later
SQLite only when a persistent inventory is proven necessary
macOS first without avoidable assumptions that block Windows
Python only for experiments and one-off research
```

The product is not a general ALS editor. Its core domain is evidence-based
dependency resolution and safe project recovery.

## 2. Mandatory Reading Route

Always start with:

```text
1. CURRENT_STATE.md
2. PRODUCT_SPINE.md
3. the one active spec for the requested module
```

Read only when relevant:

```text
AI_CONTRACT.md
  destructive, write, rewrite, privacy or uncertainty-sensitive work

ENGINEERING_RULES.md
  implementation, review and refactoring

PROJECT_MAP.md
  ownership or dependency-direction decisions

ALS_REWRITE_METHODOLOGY.md
  ALS write/rewrite research

docs/architecture/
  cross-module architecture and ADRs

docs/audits/
  audit evidence and recommendations

session-digests/ and docs/archive/
  history only; never active instructions
```

If active sources conflict, `CURRENT_STATE.md` wins for project position and
`PRODUCT_SPINE.md` wins for product purpose. Record the conflict instead of
silently selecting a historical instruction.

## 3. Current Implementation Boundary

`CURRENT_STATE.md` is the sole owner of implemented-module status, current
blockers, and the allowed next step. The presence of laboratory write modules
does not authorize production use or scope expansion.

Before changing a module, confirm that the work is within the boundary in
`CURRENT_STATE.md` and is covered by that module's active specification. Do not
revive superseded contracts from historical documents.

## 4. Product Model Rules

Keep these concepts separate:

```text
ReferenceOccurrence
  one reference occurrence extracted from ALS

RequiredAsset
  one logical asset requirement that may be referenced many times

FileOccurrence
  one file at one observed native location and time

ContentIdentity
  content identity supported by strong evidence such as a full hash
```

Hard rules:

```text
path is not identity
file existence is not asset identity
parent(ALS) is not automatically Project root
OriginalCrc is weak evidence only
scores rank candidates but do not silently confirm a match
facts, observations, evidence and decisions use separate contracts
```

## 5. Spec-Driven Workflow

Default sequence:

```text
conversation or evidence
-> update CURRENT_STATE / PRODUCT_SPINE only if product truth changed
-> spec
-> plan
-> tasks
-> fixture or experiment contract
-> module.contract.json
-> workflow_guard module-ready
-> failing tests when practical
-> implementation
-> tests
-> workflow_guard verify-module
-> review and CURRENT_STATE update
```

Do not jump from a loose idea directly to product code.

Before implementation, confirm:

```text
Parent Product Capability
Supported Use Case
Operation Flow Position
Inputs and outputs
Downstream Consumers
Non-responsibilities
Error and uncertainty behavior
Safety invariants
Test or experiment evidence
Gate Status Summary
```

An important `UNKNOWN`, `AMBIGUOUS` or `CONFLICTING` gate blocks build. Narrow
the scope or return to the user; do not guess.

## 6. Machine Guard Rule

Every implementation module uses `module.contract.json` and references the
active `ENGINEERING_RULES.md` version.

Before coding:

```text
python3 tools/workflow_guard.py module-ready <module-id>
```

Before acceptance:

```text
cargo fmt --check
cargo check --workspace --locked
cargo test --workspace --locked
cargo clippy --workspace --all-targets --locked -- -D warnings
python3 tools/workflow_guard.py verify-module <module-id>
```

Guards are executable checks, not proof of semantic correctness. The final
review must state what each guard did and did not verify.

## 7. Safety Rules

Follow `AI_CONTRACT.md` and `ENGINEERING_RULES.md`.

```text
Never mutate an original Ableton project.
Never delete user audio.
Never rewrite an original ALS.
Never perform global XML search/replace.
Never turn a hypothesis into an automatic rule.
Wrong sample match is worse than failed match.
```

Write-path invariants:

```text
no write without an explicit plan
all writes happen in staging or an explicitly selected target
no final promotion without validation
no silent overwrite
every decision and operation is auditable
unknown ALS structures block rewrite
```

## 8. Engineering Rules

Keep edits small and follow existing Rust patterns.

```text
core domain code does not depend on CLI or UI
raw ALS facts remain separate from filesystem observations
filesystem access stays behind a narrow boundary
platform-specific path semantics stay in platform adapters
public contracts are versioned
refactors preserve tests and behavior
behavior changes update tests and contract versions deliberately
```

Do not add an abstraction, crate, database or dependency without a current,
measured problem that it solves.

## 9. Documentation Rule

Only these files actively steer ordinary work:

```text
CURRENT_STATE.md
PRODUCT_SPINE.md
the active module spec
AGENTS.md
```

Use ADRs for durable cross-module decisions. Use session digests for history.
Do not copy the same `next step` into multiple documents.

Tiny housekeeping and documentation corrections do not need a new spec.
Product behavior and public-contract changes do.

## 10. Next-Step Ownership

Use exactly the next step stated in `CURRENT_STATE.md`. Historical next-step
instructions and snapshots copied into other documents are not active.
