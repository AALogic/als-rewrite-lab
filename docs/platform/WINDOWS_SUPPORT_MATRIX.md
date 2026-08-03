# Windows Support Matrix

Status: active Alpha boundary
Date: 2026-08-03

## Platform Matrix

| Area | Windows Alpha status | Evidence required |
| --- | --- | --- |
| Windows 11 x64 | Target | CI plus manual host test |
| Windows 10 x64 | Private legacy test only | Manual host report |
| Windows ARM64 | Not supported | Separate build and runtime plan |
| Windows 32-bit | Not supported | Separate product decision |
| Local NTFS | Target | Native adapter and pipeline tests |
| ReFS/FAT/exFAT | Not supported | Filesystem-specific evidence |
| UNC/network share | Rejected | Separate remote-filesystem policy |
| Junction/reparse path | Rejected | Separate safe traversal policy |
| Cloud-sync folder | Not supported | Provider-specific failure tests |
| Path at or above `MAX_PATH` | Rejected | Long-path manifest and API work |

## Product Matrix

| Capability | Windows Alpha status |
| --- | --- |
| Install for current user without admin | Included |
| Offline WebView2 installation | Included |
| Read/analyze one ALS | Included |
| Preview one current-path copy | Included |
| Create one Live 11.3 copy within supported rewrite profiles | CI target; manual proof pending |
| Redacted copy-operation diagnostic | Included |
| Live 9/10 read-only analysis | Laboratory use only |
| Live 9/10 rewrite | Blocked |
| Full-disk sample search | Deferred |
| Batch projects | Deferred |
| Code signing | Deferred; unsigned Alpha warning expected |

## Claim Rule

Passing Windows CI proves compilation and synthetic behavior on the hosted
runner. It does not prove installation, SmartScreen behavior, physical NTFS
behavior, or Ableton acceptance on the user's computer. Those claims require
the manual checklist in `docs/setup/WINDOWS_TEST_LAB.md`.
