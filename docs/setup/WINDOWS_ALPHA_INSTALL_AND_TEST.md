# Windows x64 Alpha: Install And Test

Status: private test procedure

## What This Build Is

This is the same ALS Rescue application and repository used on macOS. GitHub
Actions compiles the shared code on native Windows and packages an unsigned
Windows x64 NSIS installer.

Supported Alpha boundary:

- Windows 11 x64 is the target host;
- an updated Windows 10 x64 machine is a private legacy test host;
- local drive-letter paths on NTFS only;
- one selected Ableton project per operation;
- no UNC shares, cloud folders, junctions, symlinks, or other reparse points;
- no automatic rewrite support for Live 9 or Live 10.

The installer contains the WebView2 offline installer and installs for the
current user without requesting administrator rights. It is not digitally
signed, so Windows may show an unknown-publisher or SmartScreen warning.

## Download From GitHub Actions

1. Open the private repository on GitHub.
2. Open **Actions**, then the `quality` workflow for the reviewed commit.
3. Confirm that every job passed, including `Windows x64 Alpha installer`.
4. Download the artifact named
   `als-rescue-windows-x64-alpha-<full-commit-sha>`.
5. Extract the downloaded ZIP locally.
6. Keep both the `.exe` and matching `.sha256` file.

The artifact is retained for 14 days. Its filename contains the exact commit
used to build it.

## Verify SHA-256

In PowerShell, from the extracted artifact folder:

```powershell
$Installer = Get-ChildItem -Filter "*.exe" | Select-Object -First 1
$Expected = (Get-Content "$($Installer.FullName).sha256").Split()[0]
$Observed = (Get-FileHash -Algorithm SHA256 $Installer.FullName).Hash.ToLowerInvariant()
if ($Observed -ne $Expected) { throw "Installer SHA-256 mismatch" }
"SHA-256 verified: $Observed"
```

Stop if the hash differs.

## Install

1. Run the `.exe` as the normal Windows user.
2. Do not choose **Run as administrator**.
3. Record any SmartScreen or antivirus warning as test evidence.
4. Start **ALS Rescue** from the Start menu.

## First Test

Use a copied Ableton project. Never use the only copy of a real project.

1. Select one copied `.als` file.
2. Run analysis and record whether it completes.
3. Select an empty destination folder on the same local NTFS drive.
4. Create the rescue copy.
5. Confirm that the source `.als` and source audio remain byte-for-byte present.
6. Confirm that the destination contains the copied `.als`, required copied
   samples, and `Rescue Manifest/package-manifest.json`.
7. Open the copied project manually in a supported Ableton Live 11.3 setup.
8. Record whether Ableton reports missing files.

Live 9 and Live 10 projects may be analyzed read-only, but an expected rewrite
refusal is the correct Alpha behavior.

## Failure Report

When the application stops an operation:

1. Keep the source and failed destination unchanged.
2. In the error panel, note the displayed code and stage.
3. Select **Kopiuj raport błędu**.
4. Paste the JSON report into a private issue or Codex task.

The copy-operation report is designed to include versions, build commit,
platform, stage, status, counters, and all structured errors. It must not
contain project names, sample names, or user paths. Review it before sharing and
remove anything private if a Windows-specific message unexpectedly includes it.

## Test Checklist

Record the result against the exact installer commit:

```text
Installer commit:
Windows edition and build:
CPU architecture:
Filesystem and drive letter:
Install completed: PASS / FAIL
App launched: PASS / FAIL
Analysis completed: PASS / FAIL
Complete-copy project: PASS / FAIL / BLOCKED
Missing-sample project: PASS / FAIL / BLOCKED
Source files unchanged: PASS / FAIL
Destination manifest present: PASS / FAIL
Manual Ableton check: PASS / FAIL / NOT RUN
Error report copied and reviewed for privacy: PASS / FAIL / NOT APPLICABLE
Notes:
```

## Uninstall

Use **Settings > Apps > Installed apps > ALS Rescue > Uninstall**. Removing the
application must not remove source projects or rescue copies created by the
user.
