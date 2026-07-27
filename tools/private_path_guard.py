#!/usr/bin/env python3
"""Reject private data and configured private identifiers in repository entries."""

from __future__ import annotations

import hashlib
import os
import re
import subprocess
from dataclasses import dataclass
from pathlib import Path
from urllib.parse import unquote


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
PLACEHOLDER_USERS_CASEFOLD = frozenset(value.casefold() for value in PLACEHOLDER_USERS)

PRIVATE_IDENTIFIER_HASHES = frozenset(
    {
        "09b72beb6019afff9327c47730c1e0b115985fd429e8645ffe1b01254eb86293",
        "0dffb8bdf58fe2019b2729f7350fc136648cd4517fa1048750fd06b1b8f29575",
        "19382bd84f94bb7084c2c38f679c6bd1705415b9c0724b7b881ad381fd8d509a",
        "2466ae5a037b87591e0f9343c69068c1f8fb15712955e30c7f1af0243a36870b",
        "377d9fb524b645be005b30eccf74560c7ef37cf88b587e92d4d3c913c7d6a6a6",
        "6b3f9e77ecd7bcaff6845bbb5b291dc40a100e8f438ace224c7af985dfb036f1",
        "6c4c5463e5d52442f86071a2c9ee59645b0bcb09db87d17cd646a3ce3d263098",
        "6d95d5ad48e4f26f890ba8a254d48570f9d355ffc2324c185e78f8f85b4ab450",
        "6f873f50c90f2e687a81a2d4a81783059d6abdbf2641a7290fc8df0681a0848b",
        "70f824e07560eaa1c28c89627aca0aa6d9b6bb41e59588080beea233d4121b60",
        "73bdbda22718eefb48a5d3119427d8a2900efc78399a9052633a22cee8d6911c",
        "790c1aa44db71f81aa5f6502b8574cdc85606db5047bcfc5bdc9457e1a1af329",
        "7b35cf85d99c7538fffd45ee5917cd6384a3eccb7747694ba6e55408b82afdda",
        "7ddf34a092c45442bfc7a0e6d0418acb8b4c1f9a3c8929b81909cb1826efb196",
        "892197fe562c5367a7612ab477e3dc2a0dddb93052169df8543181aa6a5c578f",
        "8a641ff78603a8c68cfed0cbe5b25200c2e1462597b9bd35bcc4c811e0b0b7a5",
        "b468a9ef34e298d74994be31fdf0039d24d46067e5e074bf3fbdcaea14424b07",
        "b800aebe6d38fd66c0a805bd223d383cced828144edcf2137f627c7ba3b1008a",
        "b81cb7ee5ea1ba6e1aeb445b4b032a0b45bb88f963d54c65d19219615d7f82e1",
        "bf491c6a3f99f41b08fb0d98b7ce50cbde1f02f94a0d0ecf59225124564a1267",
        "ce57ed8d6c454695e33e671d6a84950e300f0e12b23c28646e8cb7308a1c1412",
        "e4054b9c382216d2ad6384ad4e3ec9092a41a6c3c900e21cc7fca78ee64c248a",
    }
)
MAX_PRIVATE_IDENTIFIER_TOKENS = 12

PROHIBITED_TRACKED_MEDIA_EXTENSIONS = frozenset(
    {
        ".aac",
        ".aif",
        ".aiff",
        ".als",
        ".asd",
        ".flac",
        ".m4a",
        ".mp3",
        ".ogg",
        ".sd2",
        ".wav",
        ".wave",
    }
)
ALLOWED_TRACKED_MEDIA_FIXTURES: frozenset[str] = frozenset()
ALLOWED_TRACKED_BINARY_FILES: frozenset[str] = frozenset()
GIT_BLOB_MODES = frozenset({"100644", "100755", "120000"})
GITLINK_MODE = "160000"

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
IDENTIFIER_TOKEN_PATTERN = re.compile(r"[a-z0-9]+")
MEDIA_EXTENSION_PATTERN = "|".join(
    re.escape(extension.removeprefix("."))
    for extension in sorted(PROHIBITED_TRACKED_MEDIA_EXTENSIONS)
)
LABELED_MEDIA_FILENAME_PATTERN = re.compile(
    rf"^\s*(?:sample|filename):\s*"
    rf"(?P<filename>[^<>\r\n]+\.(?:{MEDIA_EXTENSION_PATTERN}))\s*$",
    re.IGNORECASE,
)
OPAQUE_MEDIA_FILENAME_PATTERN = re.compile(
    rf"private_(?:audio|corpus)_fixture_\d+\.(?:{MEDIA_EXTENSION_PATTERN})",
    re.IGNORECASE,
)
PRIVATE_CORPUS_ROOT_PATTERN = re.compile(
    r"experiments[\\/]\d{4}-\d{2}-\d{2}_als_structure_corpus"
    r"(?:_[^/\\\s\"'<>]+)?",
    re.IGNORECASE,
)
PRIVATE_PROVENANCE_PATTERN = re.compile(
    r"experiments[\\/][^/\\\s\"'<>]+[\\/]"
    r"(?:copies|project_copy)[\\/][^\s\"'<>]+"
)


@dataclass(frozen=True)
class PrivatePathViolation:
    path: Path
    line: int
    category: str


@dataclass(frozen=True)
class GitIndexEntry:
    path: Path
    mode: str
    object_id: str
    stage: int


def identifier_hash(value: str) -> str:
    tokens = IDENTIFIER_TOKEN_PATTERN.findall(unquote(value).casefold())
    return hashlib.sha256(" ".join(tokens).encode("utf-8")).hexdigest()


def contains_private_identifier(text: str, hashes: frozenset[str]) -> bool:
    tokens = IDENTIFIER_TOKEN_PATTERN.findall(unquote(text).casefold())
    for width in range(1, min(len(tokens), MAX_PRIVATE_IDENTIFIER_TOKENS) + 1):
        for start in range(len(tokens) - width + 1):
            value = " ".join(tokens[start : start + width])
            if hashlib.sha256(value.encode("utf-8")).hexdigest() in hashes:
                return True
    return False


def text_violations(
    text: str,
    path: Path = Path("<memory>"),
    private_identifier_hashes: frozenset[str] = PRIVATE_IDENTIFIER_HASHES,
) -> list[PrivatePathViolation]:
    violations = []
    for line_number, line in enumerate(text.splitlines(), start=1):
        if contains_private_identifier(line, private_identifier_hashes):
            violations.append(PrivatePathViolation(path, line_number, "private_identifier"))
        if PRIVATE_CORPUS_ROOT_PATTERN.search(line):
            violations.append(PrivatePathViolation(path, line_number, "private_corpus_root"))
        elif PRIVATE_PROVENANCE_PATTERN.search(line):
            violations.append(PrivatePathViolation(path, line_number, "private_provenance"))
        media_match = LABELED_MEDIA_FILENAME_PATTERN.search(line)
        if media_match and not OPAQUE_MEDIA_FILENAME_PATTERN.fullmatch(
            media_match.group("filename")
        ):
            violations.append(
                PrivatePathViolation(path, line_number, "private_media_filename")
            )
        for category, pattern in HOME_PATTERNS:
            for match in pattern.finditer(line):
                if match.group("user").casefold() not in PLACEHOLDER_USERS_CASEFOLD:
                    violations.append(PrivatePathViolation(path, line_number, category))
    return violations


def git_index_entries(repository: Path) -> list[GitIndexEntry]:
    completed = subprocess.run(
        ["git", "ls-files", "--stage", "-z"],
        cwd=repository,
        check=True,
        stdout=subprocess.PIPE,
    )
    entries = []
    for raw_entry in completed.stdout.split(b"\0"):
        if not raw_entry:
            continue
        raw_metadata, raw_path = raw_entry.split(b"\t", 1)
        raw_mode, raw_object_id, raw_stage = raw_metadata.split(b" ", 2)
        entries.append(
            GitIndexEntry(
                path=Path(os.fsdecode(raw_path)),
                mode=raw_mode.decode("ascii"),
                object_id=raw_object_id.decode("ascii"),
                stage=int(raw_stage),
            )
        )
    return entries


def git_index_blobs(
    repository: Path, entries: list[GitIndexEntry]
) -> dict[str, bytes]:
    object_ids = list(
        dict.fromkeys(
            entry.object_id for entry in entries if entry.mode in GIT_BLOB_MODES
        )
    )
    if not object_ids:
        return {}
    completed = subprocess.run(
        ["git", "cat-file", "--batch"],
        cwd=repository,
        check=True,
        input="".join(f"{object_id}\n" for object_id in object_ids).encode("ascii"),
        stdout=subprocess.PIPE,
    )
    output = completed.stdout
    cursor = 0
    blobs = {}
    for object_id in object_ids:
        header_end = output.find(b"\n", cursor)
        if header_end < 0:
            raise RuntimeError("git cat-file returned an incomplete header")
        header = output[cursor:header_end].split(b" ")
        if len(header) != 3 or header[1] != b"blob":
            raise RuntimeError("git index entry does not reference a blob")
        size = int(header[2])
        content_start = header_end + 1
        content_end = content_start + size
        if output[content_end : content_end + 1] != b"\n":
            raise RuntimeError("git cat-file returned an incomplete blob")
        blobs[object_id] = output[content_start:content_end]
        cursor = content_end + 1
    return blobs


def untracked_repository_files(repository: Path) -> list[Path]:
    completed = subprocess.run(
        ["git", "ls-files", "--others", "--exclude-standard", "-z"],
        cwd=repository,
        check=True,
        stdout=subprocess.PIPE,
    )
    return [
        Path(os.fsdecode(raw_path))
        for raw_path in completed.stdout.split(b"\0")
        if raw_path
    ]


def utf16_encoding(content: bytes) -> str | None:
    if content.startswith((b"\xff\xfe", b"\xfe\xff")):
        return "utf-16"
    if len(content) % 2:
        return None
    pair_count = len(content) // 2
    threshold = max(2, pair_count // 4)
    even_nuls = content[::2].count(0)
    odd_nuls = content[1::2].count(0)
    if odd_nuls >= threshold and odd_nuls > even_nuls * 2:
        return "utf-16-le"
    if even_nuls >= threshold and even_nuls > odd_nuls * 2:
        return "utf-16-be"
    return None


def decode_text_content(content: bytes) -> str | None:
    if b"\0" not in content:
        return content.decode("utf-8", errors="ignore")
    encoding = utf16_encoding(content)
    if encoding is None:
        return None
    try:
        text = content.decode(encoding)
    except UnicodeDecodeError:
        return None
    return None if "\0" in text else text


def content_violations(
    content: bytes,
    path: Path,
    private_identifier_hashes: frozenset[str],
    fail_closed: bool,
) -> list[PrivatePathViolation]:
    text = decode_text_content(content)
    if text is not None:
        return text_violations(text, path, private_identifier_hashes)
    if fail_closed and path.as_posix() not in ALLOWED_TRACKED_BINARY_FILES:
        return [PrivatePathViolation(path, 1, "unscannable_tracked_binary")]
    return []


def path_violations(
    relative_path: Path,
    private_identifier_hashes: frozenset[str],
) -> list[PrivatePathViolation]:
    return text_violations(
        relative_path.as_posix(),
        relative_path,
        private_identifier_hashes,
    )


def is_prohibited_tracked_media(relative_path: Path) -> bool:
    return (
        relative_path.suffix.casefold() in PROHIBITED_TRACKED_MEDIA_EXTENSIONS
        and relative_path.as_posix() not in ALLOWED_TRACKED_MEDIA_FIXTURES
    )


def scan_repository(
    repository: Path,
    private_identifier_hashes: frozenset[str] = PRIVATE_IDENTIFIER_HASHES,
) -> list[PrivatePathViolation]:
    violations = []
    index_entries = git_index_entries(repository)
    index_blobs = git_index_blobs(repository, index_entries)
    for entry in index_entries:
        relative_path = entry.path
        violations.extend(path_violations(relative_path, private_identifier_hashes))
        if is_prohibited_tracked_media(relative_path):
            violations.append(
                PrivatePathViolation(relative_path, 1, "tracked_private_media")
            )
            continue
        if entry.mode == GITLINK_MODE:
            continue
        if entry.mode not in GIT_BLOB_MODES:
            violations.append(
                PrivatePathViolation(relative_path, 1, "unsupported_tracked_mode")
            )
            continue
        violations.extend(
            content_violations(
                index_blobs[entry.object_id],
                relative_path,
                private_identifier_hashes,
                fail_closed=True,
            )
        )

    for relative_path in untracked_repository_files(repository):
        violations.extend(path_violations(relative_path, private_identifier_hashes))
        path = repository / relative_path
        if path.is_symlink():
            violations.extend(
                text_violations(
                    os.fsdecode(os.readlink(path)),
                    relative_path,
                    private_identifier_hashes,
                )
            )
            continue
        if not path.is_file():
            continue
        violations.extend(
            content_violations(
                path.read_bytes(),
                relative_path,
                private_identifier_hashes,
                fail_closed=False,
            )
        )
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
