# Engineering Rules

Status: active engineering standard  
Engineering Rules Version: 0.1  
Date: 2026-06-09  
Applies to: all implementation modules from `002-dependency-extractor` onward

## 1. Purpose

This file defines how code should be designed, implemented, tested and
refactored in this project.

`PRODUCT_SPEC.md` defines what the product should do.

`ENGINEERING_RULES.md` defines how the code is allowed to be built.

The goal is clean, maintainable, scalable and understandable code that protects
user data. This file is not only advice. Critical rules should be connected to
module contracts, tests, types or `workflow_guard.py` whenever practical.

## 2. Enforcement Levels

Use these labels in module specs, tasks and contracts:

```text
ENFORCED_BY_GUARD
  A script can check the rule before or after implementation.

ENFORCED_BY_TEST
  A unit, fixture, integration or end-to-end test checks the rule.

ENFORCED_BY_TYPE
  Rust types, enums, visibility or function signatures prevent misuse.

REVIEW_ONLY
  A human or Codex review must judge the rule.

DOCUMENTED_ONLY
  The rule records intent, context or a limitation that is not yet enforced.
```

Rule:

```text
MUST: Critical safety, contract and module-boundary rules should not remain
only DOCUMENTED_ONLY when they can reasonably become a guard, test or type.
```

## 3. How Codex Must Use This File

Before implementing or refactoring a module, Codex must check:

```text
module spec
module.contract.json
tasks.md
ENGINEERING_RULES.md
```

Before accepting a module, Codex must verify:

```text
required quality obligations are present
required obligations are checked when verify-module is run
required tests exist
required tests are not ignored or empty
forbidden responsibilities are absent
public contracts match the module contract
workflow_guard verify-module passes
remaining risks are stated
```

If a requested change conflicts with these rules, Codex must stop and explain
the conflict instead of silently choosing a shortcut.

Guard v0.2:

```text
verify-module should reject:
  unchecked required obligations
  ignored required tests
  empty or tautological required tests
  extra public structs or fields when public_contract_mode is exact
  public field type mismatches
  forbidden scope verbs/terms in module symbols
  forbidden high-risk core source patterns
  unapproved dependency names
  required docs with no useful content
  unresolved uncertainty phrases that are not explicitly non-blocking
```

## 4. Clean Code

Rules:

```text
SHOULD: Use names that reveal domain meaning, not implementation accidents.
SHOULD: Prefer names like DependencyRef, PackagePlan, RewriteOperation and
        ValidationResult over vague names like data, thing, item or result2.
SHOULD: Keep functions short and focused.
MUST: A function should have one primary reason to change.
SHOULD: Code should be understandable without comments that restate syntax.
SHOULD: Comments should explain decisions, risks, domain context or non-obvious
        constraints.
MUST NOT: Hide important behavior behind clever string manipulation when a
          structured parser, type or explicit model is available.
```

Typical enforcement:

```text
max_function_lines -> ENFORCED_BY_GUARD
public model fields -> ENFORCED_BY_TYPE / ENFORCED_BY_GUARD
name clarity -> REVIEW_ONLY
```

## 5. SOLID In This Project

### 5.1 Single Responsibility Principle

```text
MUST: Each module must own one clear responsibility.
MUST NOT: A module must not perform work assigned to downstream modules.
```

Example:

```text
ALSReader reads ALS and returns ALSReadModel.
DependencyExtractor normalizes active audio references.
PathVerifier checks whether paths exist.
SampleMatcher proposes matches.
PackagePlanner decides copy/rewrite plans.
ALSRewriter rewrites supported ALS references in copies.
```

### 5.2 Open / Closed Principle

```text
SHOULD: Add support for new dependency kinds by extending models and strategies,
        not by rewriting unrelated modules.
MUST: New behavior that changes public output must update the module contract.
```

### 5.3 Liskov Substitution Principle

```text
SHOULD: When traits or adapters are introduced, implementations must preserve
        the promised behavior of the trait.
MUST NOT: A test adapter may not silently behave differently from the production
          adapter in ways that hide safety risks.
```

### 5.4 Interface Segregation Principle

```text
SHOULD: Keep public interfaces small.
MUST NOT: Force a module to depend on a broad interface when it only needs a
          narrow contract.
```

Example:

```text
DependencyExtractor should consume ALSReadModel, not a filesystem scanner,
not a UI state object and not an entire application context.
```

### 5.5 Dependency Inversion Principle

```text
SHOULD: Core domain logic should depend on explicit data contracts and narrow
        traits, not concrete UI, filesystem or platform implementations.
MUST: UI must not call low-level rewrite or copy behavior directly.
```

## 6. Modularity

Rules:

```text
MUST: Modules must have clear boundaries.
MUST: Public API should expose only what downstream modules need.
SHOULD: Helper functions should stay private by default.
MUST NOT: Mix ALS reading, dependency extraction, filesystem verification,
          matching, planning, copying, rewriting and validation in one module.
MUST NOT: Add fields to a module output only because a later module may someday
          need them.
```

Adding a field for future use is allowed only when:

```text
the downstream consumer is named
the field meaning is clear
the field owner is clear
the module is the correct source of truth
the field is included in module.contract.json
```

## 7. Encapsulation

Rules:

```text
MUST: Keep implementation details private unless they are part of the public
      module contract.
MUST: Public structs and functions must be intentional.
SHOULD: Prefer private helper functions for parsing, normalization and local
        transformations.
MUST NOT: Other modules may rely on private implementation details.
```

Example:

```text
ALSReader may have private XML traversal helpers.
DependencyExtractor should not know how ALSReader traverses XML.
DependencyExtractor should only consume ALSReadModel.
```

## 8. Communication Between Modules

Rules:

```text
MUST: Modules communicate through explicit input and output contracts.
MUST: Public data models must have stable names and versions when downstream
      modules depend on them.
MUST: Raw ALS scalar values that are not understood must be preserved as raw
      strings when the spec requires preservation.
MUST NOT: Use hidden global state to pass decisions between modules.
MUST NOT: Let downstream modules depend on diagnostic CLI JSON unless the spec
          explicitly declares it as a contract.
```

Good module output answers:

```text
what this module knows
what this module does not know
what was ignored
what is uncertain
what downstream module may safely consume
```

## 9. Testability

Rules:

```text
MUST: Pure domain logic should be testable without filesystem side effects.
MUST: Filesystem, platform, Ableton and UI behavior should be isolated behind
      narrow boundaries when introduced.
SHOULD: Use dependency injection or narrow traits when a module needs external
        effects.
MUST: Unit tests should cover pure logic.
MUST: Fixture tests should cover real ALS/domain data when a module touches ALS.
MUST: Safety tests are required before write, copy, delete or rewrite behavior.
SHOULD: End-to-end tests should cover complete user flows after the pieces are
        stable enough to connect.
```

## 10. Refactoring

Rules:

```text
MUST: Refactoring means changing structure without changing behavior.
MUST: If behavior changes, call it a contract change, not a refactor.
MUST: Do not change tests during a pure refactor unless the tests are being
      renamed or moved without changing their assertions.
MUST: Run the existing tests before accepting a refactor.
SHOULD: Refactor in small steps.
SHOULD: Refactor when a module is getting harder to test, explain or extend.
```

Examples:

```text
Splitting lib.rs into als_reader.rs and models.rs is a refactor if public API
and tests keep the same behavior.

Changing ALSReadModel fields is not a refactor. It is a contract change.
```

## 11. DRY, KISS, YAGNI

Rules:

```text
DRY:
  SHOULD NOT duplicate the same domain logic in multiple modules.

KISS:
  SHOULD prefer simple explicit code over broad abstractions.

YAGNI:
  MUST NOT build features only because they may be useful someday.
```

Important nuance:

```text
Do not create abstractions too early.
Do create clear data contracts early when downstream modules depend on them.
```

## 12. Error Handling

Rules:

```text
MUST: Expected failures must return structured errors.
MUST NOT: Use panic for recoverable user, filesystem, data or validation errors.
MUST NOT: Silently ignore errors that affect user trust, data safety or output
          correctness.
MUST: Error messages should help diagnose what failed and where.
SHOULD: Error codes should distinguish user errors, system errors, data errors
        and programmer errors.
```

Project error examples:

```text
ALS_NOT_FOUND
ALS_NOT_READABLE
ALS_NOT_GZIP
ALS_XML_INVALID
ALS_UNSUPPORTED_ROOT
SAMPLE_NOT_FOUND
AMBIGUOUS_MATCH
COPY_FAILED
REWRITE_PLAN_INVALID
VALIDATION_FAILED
```

## 13. Data Safety

Hard rules:

```text
MUST NOT: Overwrite original user files.
MUST NOT: Delete user audio files.
MUST NOT: Rewrite original .als files.
MUST: Rewrite and package operations work on copies.
MUST: Copy/rewrite operations require a plan before execution.
MUST: Writes require validation before final acceptance.
MUST: Important decisions and operations must be recorded in a manifest or
      equivalent operation log when the module performs write/copy/rewrite work.
MUST: Ambiguous matches must block automatic destructive or rewrite behavior.
```

For read-only modules:

```text
MUST: The module must not create, modify, delete, move or rewrite user files.
```

## 14. Scalability And Performance

Rules:

```text
SHOULD: Avoid repeating expensive scans when a cache or index is appropriate.
SHOULD: Design for incremental scanning once persistent indexing exists.
SHOULD: Normalize ordering before returning public results.
SHOULD: Consider large sample libraries and many projects when designing data
        structures.
MUST NOT: Add premature distributed systems, databases or background services
          before the MVP needs them.
```

Performance is not an excuse to weaken safety. A faster wrong sample match is
worse than a slower blocked match.

## 15. Clear Architecture

Rules:

```text
MUST: Keep UI, domain logic, filesystem access and persistence separated.
MUST: Dependencies should point inward toward domain models and explicit
      contracts, not outward toward UI or platform details.
MUST: UI must not rewrite ALS directly.
MUST: ALSRewriter must not decide what should be copied.
MUST: PackagePlanner must not perform the copy.
MUST: CopyStager must not rewrite ALS.
```

Layer intent:

```text
UI:
  presents decisions and asks user for approval.

Core domain:
  reads, extracts, classifies, matches, plans and validates through explicit
  contracts.

Filesystem adapters:
  perform controlled reads/writes requested by core plans.

Persistence:
  stores manifests, indexes and operation history.
```

## 16. Technical Documentation

Document when:

```text
a module contract is created or changed
public data models are introduced
a safety rule is added
an ALS behavior is confirmed or rejected
a known unknown affects implementation
a decision changes architecture or product direction
```

Module specs should include:

```text
responsibility
inputs
outputs
data contracts
machine contract
does not do
algorithm notes
safety rules
error codes
tests
fixtures or evidence plan
acceptance criteria
downstream consumers
known unknowns
```

## 17. Code Review Checklist

Before accepting code, review:

```text
Does the code do one clear thing?
Are names understandable in this product domain?
Is duplicated logic avoided?
Are public types and functions intentional?
Are module boundaries respected?
Are errors structured and diagnostic?
Are tests present for required behavior?
Are safety rules tested when user files are involved?
Does the change avoid unnecessary abstraction?
Does the change avoid future-only features?
Does the module avoid knowing too much about other modules?
Does workflow_guard verify-module pass when required?
```

## 18. Project Hard Rules

These rules override convenience:

```text
MUST NOT: Original Ableton projects may not be mutated.
MUST NOT: Original .als files may not be rewritten.
MUST NOT: User audio files may not be deleted.
MUST NOT: ALS rewrite may not use global XML search/replace.
MUST NOT: OriginalCrc may not be treated as proof of file identity.
MUST: Every copy/rewrite operation must have a plan.
MUST: Every write-capable operation must produce manifest or operation evidence.
MUST: Every new implementation module must have a spec before implementation.
MUST: Every new implementation module from 002 onward must have
      module.contract.json.
MUST: Every new implementation module from 002 onward must reference this
      Engineering Rules version in module.contract.json.
```

## 19. Module Review Template

Use this template during module design or review:

```text
Module name:
Engineering Rules version:

Responsibility:

Inputs:

Outputs:

Public API:

Private helpers:

Does Not Do:

Dependencies:

Possible errors:

Unit tests:

Integration tests:

Fixture/evidence tests:

Acceptance criteria:

Risks:

Safety rules:

Enforcement summary:
  ENFORCED_BY_GUARD:
  ENFORCED_BY_TEST:
  ENFORCED_BY_TYPE:
  REVIEW_ONLY:
  DOCUMENTED_ONLY:
```
