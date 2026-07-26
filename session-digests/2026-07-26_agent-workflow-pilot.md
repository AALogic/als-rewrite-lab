# Session Digest: Agent Workflow Pilot

Date: 2026-07-26
Source: tool installation / synthetic experiment / clean-clone verification
Status: accepted

## 1. Context

The project needed evidence that AI workflow tools improve implementation and
review rather than only adding more Markdown or coordination overhead.

## 2. Important Signal

The guard and delegation mechanisms work, but the product repository is not yet
reproducible in a clean worktree because its regression suite depends on local,
ignored private ALS fixtures.

## 3. FACT

```text
GitHub CLI is authenticated as AALogic.
The product has local baseline commit 6d4d093 and no configured remote.
No-Mistakes v1.41.2 caught two deliberate contract/test omissions.
It fixed them, reran review/test/document/lint and opened private pilot PR #1.
Firstmate dispatched one Codex scout with xhigh effort through Herdr.
Treehouse created an isolated worktree and returned it after the report.
The scout did not modify, commit, push or merge project files.
A clean product clone failed 8 tests due to missing ignored ALS fixtures.
```

## 4. HYPOTHESIS

```text
No-Mistakes should reduce missed contract and safety errors on risky changes.
Firstmate should help when tasks are genuinely independent or parallel.
Using Firstmate for small sequential work is likely slower and more expensive.
```

## 5. DECISION

```text
Do not publish or connect the product repository to automated coding yet.
Do not commit private real-world ALS projects merely to make CI green.
Create synthetic gzip/XML ALS fixtures first.
Use direct Codex for small work, No-Mistakes for risky validation and
Firstmate selectively for independent research/review or parallel tasks.
```

## 6. QUESTION

```text
Which smallest synthetic ALS fixture set preserves the confirmed behavior of
the current real fixtures without including private names, paths or metadata?
```

## 7. RISK

```text
Agents in clean worktrees may report false regressions or change correct code
when required local fixtures are missing.
Multi-agent coordination can consume disproportionate time and tokens.
```

## 8. MVP Impact

This does not change MVP product scope. It adds a repository reproducibility
gate before further delegated implementation.

## 9. Test / Evidence Needed

```text
Generate synthetic ALS fixtures.
Run cargo test --workspace --locked in a fresh clone.
Run workflow guards in the same fresh clone.
Confirm no private absolute paths or project metadata are tracked.
```

## 10. Proposed Writes

Applied to:

```text
CURRENT_STATE.md
PRODUCT_BACKLOG.md
tests/fixtures/README.md
this session digest
```

## 11. Next Step

Design and generate the minimum synthetic fixture set, then prove that the full
repository test suite passes in a clean clone.
