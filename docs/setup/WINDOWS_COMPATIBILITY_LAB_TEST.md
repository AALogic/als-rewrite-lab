# Windows Compatibility Lab: Install And Test

Status: private evidence-gathering procedure

## Purpose

This is a separate unsigned build named **ALS Rescue Compatibility Lab**. It
can coexist with the strict **ALS Rescue** Alpha because it has a different
application identifier. It does not declare Live 9, 10, 11.2 or 12 supported.
It tests whether an unconfirmed Set uses a rewrite structure that the current
engine already understands.

The Lab keeps the ordinary safety chain:

```text
analyze -> explicit experimental consent -> preview -> stage a fresh copy
-> rewrite known reference shapes -> validate -> manifest -> promote
```

Unknown reference shapes still block before any destination is created.
Original ALS and media remain read-only.

## Verified Private Installer

Build commit:

```text
71b9fabe159a47e22ac81e3021e128c4296ca8ac
```

Expected SHA-256:

```text
d1ffb5c0c3af19f3646a1a407a90b12df4c14ffbb315d4e30a6d0b7dbbc5b179
```

The local downloaded artifact is outside Git under:

```text
windows-installers/compatibility-lab-71b9fab/
```

The installer is unsigned. Windows can show an unknown-publisher or
SmartScreen warning. This build is for the owner's private machines only.

## Test One Project

Never test the only copy of a project.

1. Prepare a copied Ableton project with its current media still available.
2. Start **ALS Rescue Compatibility Lab**.
3. Select one copied `.als` and run analysis.
4. Confirm that the screen identifies the Ableton version as unconfirmed and
   shows the count of known and unknown reference structures.
5. Stop if unknown structures are reported or the application blocks the plan.
6. Enable **Zezwalam na eksperymentalna kopie do recznego sprawdzenia**.
7. Choose a new empty destination on a local NTFS drive.
8. Review the copy plan and create the copy.
9. Open only the generated ALS in the intended Ableton version.
10. In the Lab choose the observed result:
    - opens without missing files;
    - opens with missing files;
    - fails to open;
    - not checked.
11. Enter the Ableton version used for the manual test.
12. Select **Kopiuj raport dla Codexa** and review the JSON before sharing it.

Repeat the procedure separately for a Live 10 and a Live 11.2 project. Do not
batch projects in this experiment: each report must describe one source Set,
one generated copy and one manual Ableton result.

## What To Return

Return the complete JSON copied by **Kopiuj raport dla Codexa**. If the
operation blocks before manual verification, also return the JSON from
**Kopiuj raport bledu**.

The report should contain:

- build commit, Windows version and architecture;
- Ableton document metadata without local paths;
- counts and structural profiles of active audio references;
- rewrite policy and completed pipeline stage;
- copy/rewrite/omission counts and structured error codes;
- the manually selected Ableton outcome;
- a provisional candidate success/failure conclusion.

The report must not contain project names, sample names, local paths or raw XML.
Review it before sharing and remove unexpected private data if found.

## Interpretation

One successful project is evidence for that exact structural case, not support
for an entire Ableton release. A version can move toward strict support only
after multiple representative fixtures, regression tests and explicit review
of the rewrite rules. A failure remains valuable evidence and must not cause
the Lab to weaken a safety blocker automatically.
