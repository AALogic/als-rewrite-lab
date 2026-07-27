#!/usr/bin/env python3
"""Reject private absolute home paths in repository text files."""

from __future__ import annotations

import os
import re
import subprocess
from dataclasses import dataclass
from pathlib import Path


PLACEHOLDER_USERS = frozenset(
    {
        "...",
        "$USER",
        "%USERNAME%",
        "example",
        "fixture",
        "me",
        "name",
        "private",
        "root",
        "runner",
        "test",
        "user",
        "username",
    }
)

HOME_PATTERNS = (
    (
        "macos_home",
        re.compile(r"/" + r"Users/" + r"(?P<user>[^/\\\s\"'`<>]+)"),
    ),
    (
        "linux_home",
        re.compile(r"/" + r"home/" + r"(?P<user>[^/\\\s\"'`<>]+)"),
    ),
    (
        "windows_home",
        re.compile(
            r"[A-Za-z]:[\\/]+"
            + r"Users[\\/]+"
            + r"(?P<user>[^/\\\s\"'`<>]+)",
            re.IGNORECASE,
        ),
    ),
)


@dataclass(frozen=True)
class PrivatePathViolation:
    path: Path
    line: int
    category: str


def text_violations(text: str, path: Path = Path("<memory>")) -> list[PrivatePathViolation]:
    violations = []
    for line_number, line in enumerate(text.splitlines(), start=1):
        for category, pattern in HOME_PATTERNS:
            for match in pattern.finditer(line):
                if match.group("user") not in PLACEHOLDER_USERS:
                    violations.append(PrivatePathViolation(path, line_number, category))
    return violations


def repository_files(repository: Path) -> list[Path]:
    completed = subprocess.run(
        [
            "git",
            "ls-files",
            "--cached",
            "--others",
            "--exclude-standard",
            "-z",
        ],
        cwd=repository,
        check=True,
        stdout=subprocess.PIPE,
    )
    return [
        repository / os.fsdecode(raw_path)
        for raw_path in completed.stdout.split(b"\0")
        if raw_path
    ]


def scan_repository(repository: Path) -> list[PrivatePathViolation]:
    violations = []
    for path in repository_files(repository):
        if path.is_symlink() or not path.is_file():
            continue
        content = path.read_bytes()
        if b"\0" in content:
            continue
        relative_path = path.relative_to(repository)
        violations.extend(text_violations(content.decode("utf-8", errors="ignore"), relative_path))
    return violations


def main() -> int:
    repository = Path(__file__).resolve().parents[1]
    violations = scan_repository(repository)
    if violations:
        for violation in violations:
            print(f"{violation.path}:{violation.line}: {violation.category}")
        print(f"private-path guard: FAIL ({len(violations)} violation(s))")
        return 1
    print("private-path guard: PASS")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
