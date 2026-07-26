# Session Digest: Product Office Trial

Date: 2026-06-01  
Source: Codex conversation  
Status: accepted

## 1. Context

User wants to work mostly in one Codex project conversation instead of splitting thinking between GPT and Codex.

User is a vision-driven founder/operator who generates many ideas and does not want to manually sort every conversation into specs, backlog, architecture, hypotheses and tasks.

Need:

```text
a lightweight project function that processes conversations
keeps momentum
protects MVP scope
asks before writing important updates
```

## 2. Important Signal

The project needs a workflow layer that behaves like a small product team:

```text
Product Lead
MVP Guard
Tech Lead
Research Lead
Safety / QA
Builder
```

This should not replace user decisions.

It should reduce cognitive load by proposing classification and next steps.

## 3. FACT

```text
The project already has navigator/state/structure/technology documents.
The user wants Codex to become the main workspace for both thinking and building.
Long chat history should not be the only memory of the project.
Important outcomes should be stored in project files.
```

## 4. HYPOTHESIS

```text
A Product Office workflow will reduce chaos and help convert conversations into product progress.
Session digests can be a better intermediate artifact than updating specs after every conversation.
The user should not manually decide where every idea belongs.
```

## 5. DECISION

```text
Add a trial Product Office workflow.
Create session-digests/ for processed conversations.
Create intake/ for long raw materials.
Create PRODUCT_BACKLOG.md for now/next/later/hypotheses/risks.
Update PROJECT_NAVIGATOR.md and PROJECT_STRUCTURE.md to reference this workflow.
```

## 6. QUESTION

```text
Will this workflow feel lighter or heavier after 2-3 real uses?
Should Product Office always ask before updating files, or should some files be auto-updated?
How detailed should session digests be?
```

## 7. RISK

```text
Too many process files could recreate the same chaos they are meant to reduce.
Session digests could become busywork if they are too long.
Automatically writing to specs from loose conversations could pollute product requirements.
```

## 8. MVP Impact

This does not change the product MVP itself.

It changes the way we manage discovery and decisions before implementation.

MVP remains:

```text
Spec 001 ALSReader
Rust CLI analyze --json
No UI
No rewrite
No delete
```

## 9. Proposed Writes

Applied:

```text
PRODUCT_OFFICE.md
PRODUCT_BACKLOG.md
intake/README.md
session-digests/README.md
session-digests/_template.md
session-digests/2026-06-01_product-office-trial.md
PROJECT_NAVIGATOR.md
PROJECT_STRUCTURE.md
CURRENT_STATE.md
```

## 10. Next Step

Use the workflow once on a real messy conversation.

Recommended command:

```text
Product Office: przetworz te rozmowe. Zrob digest i zaproponuj, co zapisac gdzie. Nie koduj.
```
