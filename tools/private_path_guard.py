#!/usr/bin/env python3
"""Reject private data and configured private identifiers in repository entries."""

from __future__ import annotations

import argparse
import hashlib
import os
import re
import subprocess
from collections.abc import Iterator
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
ALLOWED_TRACKED_BINARY_FILES = frozenset(
    {
        "apps/rescue-desktop/public/quick-worker/appearing.png",
        "apps/rescue-desktop/public/quick-worker/complete.png",
        "apps/rescue-desktop/public/quick-worker/destination-required.png",
        "apps/rescue-desktop/public/quick-worker/incomplete.png",
        "apps/rescue-desktop/public/quick-worker/parcel.png",
        "apps/rescue-desktop/public/quick-worker/preparing.png",
        "apps/rescue-desktop/public/quick-worker/ready.png",
        "apps/rescue-desktop/public/quick-worker/share-armed.png",
        "apps/rescue-desktop/public/quick-worker/unable.png",
        "apps/rescue-desktop/public/quick-worker/van-arrival.png",
        "apps/rescue-desktop/public/quick-worker/working.png",
        "apps/rescue-desktop/src-tauri/assets/quick-parcel-drag.png",
        "apps/rescue-desktop/src-tauri/assets/quick-worker-share-drag.png",
        "apps/rescue-desktop/src-tauri/icons/128x128.png",
        "apps/rescue-desktop/src-tauri/icons/128x128@2x.png",
        "apps/rescue-desktop/src-tauri/icons/32x32.png",
        "apps/rescue-desktop/src-tauri/icons/Square107x107Logo.png",
        "apps/rescue-desktop/src-tauri/icons/Square142x142Logo.png",
        "apps/rescue-desktop/src-tauri/icons/Square150x150Logo.png",
        "apps/rescue-desktop/src-tauri/icons/Square284x284Logo.png",
        "apps/rescue-desktop/src-tauri/icons/Square30x30Logo.png",
        "apps/rescue-desktop/src-tauri/icons/Square310x310Logo.png",
        "apps/rescue-desktop/src-tauri/icons/Square44x44Logo.png",
        "apps/rescue-desktop/src-tauri/icons/Square71x71Logo.png",
        "apps/rescue-desktop/src-tauri/icons/Square89x89Logo.png",
        "apps/rescue-desktop/src-tauri/icons/StoreLogo.png",
        "apps/rescue-desktop/src-tauri/icons/icon.icns",
        "apps/rescue-desktop/src-tauri/icons/icon.ico",
        "apps/rescue-desktop/src-tauri/icons/icon.png",
        "docs/design/quick-copy-worker/quick-worker-master-alpha.png",
        "docs/design/quick-copy-worker/quick-worker-share-source.png",
    }
)
GIT_BLOB_MODES = frozenset({"100644", "100755", "120000"})
GITLINK_MODE = "160000"
GIT_SCANNABLE_OBJECT_TYPES = frozenset({"blob", "commit", "tag"})
MAX_TRACKED_BLOB_BYTES = 8 * 1024 * 1024
ALLOWED_TEXT_CONTROL_CHARACTERS = frozenset({"\t", "\n", "\r", "\f"})

HOME_PATTERNS = (
    (
        "macos_home",
        re.compile(
            r"/" + r"Users/" + r"(?P<user>[^/\\\s\"'`<>]+)",
            re.IGNORECASE,
        ),
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
    r"experiments[\\/](?:[^/\\\r\n\"'<>]+[\\/])*"
    r"(?:copies|project_copy)[\\/][^\r\n\"'<>]+",
    re.IGNORECASE,
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


@dataclass(frozen=True)
class GitObjectMetadata:
    object_id: str
    object_type: str
    size: int


@dataclass(frozen=True)
class GitHistoryEntry:
    snapshot_id: str
    path: Path
    mode: str
    object_type: str
    object_id: str


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


def git_object_metadata(
    repository: Path, object_ids: list[str]
) -> list[GitObjectMetadata]:
    object_ids = list(dict.fromkeys(object_ids))
    if not object_ids:
        return []
    completed = subprocess.run(
        ["git", "cat-file", "--batch-check"],
        cwd=repository,
        check=True,
        input="".join(f"{object_id}\n" for object_id in object_ids).encode("ascii"),
        stdout=subprocess.PIPE,
    )
    output_lines = completed.stdout.splitlines()
    if len(output_lines) != len(object_ids):
        raise RuntimeError("git cat-file returned incomplete blob metadata")
    metadata = []
    for object_id, output_line in zip(object_ids, output_lines):
        header = output_line.split(b" ")
        if len(header) != 3 or header[0].decode("ascii") != object_id:
            raise RuntimeError("git cat-file returned invalid object metadata")
        metadata.append(
            GitObjectMetadata(
                object_id=object_id,
                object_type=header[1].decode("ascii"),
                size=int(header[2]),
            )
        )
    return metadata


def git_index_blob_sizes(repository: Path, object_ids: list[str]) -> dict[str, int]:
    metadata = git_object_metadata(repository, object_ids)
    if any(item.object_type != "blob" for item in metadata):
        raise RuntimeError("git index entry does not reference a blob")
    return {item.object_id: item.size for item in metadata}


def git_object_contents(
    repository: Path,
    object_ids: list[str],
    allowed_types: frozenset[str],
) -> Iterator[tuple[str, str, bytes]]:
    object_ids = list(dict.fromkeys(object_ids))
    if not object_ids:
        return
    process = subprocess.Popen(
        ["git", "cat-file", "--batch"],
        cwd=repository,
        stdin=subprocess.PIPE,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
    )
    if process.stdin is None or process.stdout is None or process.stderr is None:
        process.kill()
        process.wait()
        raise RuntimeError("git cat-file streams are unavailable")
    try:
        for object_id in object_ids:
            process.stdin.write(f"{object_id}\n".encode("ascii"))
            process.stdin.flush()
            header = process.stdout.readline().rstrip(b"\n").split(b" ")
            object_type = header[1].decode("ascii") if len(header) == 3 else ""
            if (
                len(header) != 3
                or header[0].decode("ascii") != object_id
                or object_type not in allowed_types
            ):
                raise RuntimeError("git object has an unexpected type")
            size = int(header[2])
            if size > MAX_TRACKED_BLOB_BYTES:
                raise RuntimeError("git cat-file returned an oversized object")
            content = process.stdout.read(size)
            if len(content) != size or process.stdout.read(1) != b"\n":
                raise RuntimeError("git cat-file returned an incomplete object")
            yield object_id, object_type, content
        process.stdin.close()
        stderr = process.stderr.read()
        return_code = process.wait()
        if return_code:
            raise RuntimeError(
                f"git cat-file failed with exit code {return_code}: "
                f"{stderr.decode('utf-8', errors='replace').strip()}"
            )
    finally:
        if not process.stdin.closed:
            try:
                process.stdin.close()
            except BrokenPipeError:
                pass
        if process.poll() is None:
            process.kill()
            process.wait()
        process.stdout.close()
        process.stderr.close()


def git_index_blob_contents(
    repository: Path, object_ids: list[str]
) -> Iterator[tuple[str, bytes]]:
    for object_id, _, content in git_object_contents(
        repository, object_ids, frozenset({"blob"})
    ):
        yield object_id, content


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


def git_repository_is_shallow(repository: Path) -> bool:
    completed = subprocess.run(
        ["git", "rev-parse", "--is-shallow-repository"],
        cwd=repository,
        check=True,
        stdout=subprocess.PIPE,
        text=True,
    )
    return completed.stdout.strip() == "true"


def git_reachable_objects(repository: Path) -> list[GitObjectMetadata]:
    completed = subprocess.run(
        ["git", "rev-list", "--objects", "--all", "--no-object-names"],
        cwd=repository,
        check=True,
        stdout=subprocess.PIPE,
        text=True,
    )
    return git_object_metadata(repository, completed.stdout.splitlines())


def git_history_entries(
    repository: Path, objects: list[GitObjectMetadata]
) -> list[GitHistoryEntry]:
    roots = [item.object_id for item in objects if item.object_type == "commit"]
    object_types = {item.object_id: item.object_type for item in objects}
    completed = subprocess.run(
        ["git", "for-each-ref", "--format=%(objectname) %(*objectname)"],
        cwd=repository,
        check=True,
        stdout=subprocess.PIPE,
        text=True,
    )
    for line in completed.stdout.splitlines():
        for object_id in line.split():
            if object_types.get(object_id) == "tree":
                roots.append(object_id)

    entries: dict[tuple[str, str, str, str], GitHistoryEntry] = {}
    for root in dict.fromkeys(roots):
        completed = subprocess.run(
            ["git", "ls-tree", "-r", "-z", "--full-tree", root],
            cwd=repository,
            check=True,
            stdout=subprocess.PIPE,
        )
        for raw_entry in completed.stdout.split(b"\0"):
            if not raw_entry:
                continue
            raw_metadata, raw_path = raw_entry.split(b"\t", 1)
            raw_mode, raw_type, raw_object_id = raw_metadata.split(b" ", 2)
            entry = GitHistoryEntry(
                snapshot_id=root,
                path=Path(os.fsdecode(raw_path)),
                mode=raw_mode.decode("ascii"),
                object_type=raw_type.decode("ascii"),
                object_id=raw_object_id.decode("ascii"),
            )
            key = (
                entry.path.as_posix(),
                entry.mode,
                entry.object_type,
                entry.object_id,
            )
            entries.setdefault(key, entry)
    return list(entries.values())


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


def is_plausible_text(text: str) -> bool:
    for character in text:
        codepoint = ord(character)
        if character in ALLOWED_TEXT_CONTROL_CHARACTERS:
            continue
        if codepoint < 0x20 or 0x7F <= codepoint <= 0x9F:
            return False
    return True


def decode_text_content(content: bytes) -> str | None:
    if b"\0" not in content:
        try:
            text = content.decode("utf-8")
        except UnicodeDecodeError:
            return None
        return text if is_plausible_text(text) else None
    encoding = utf16_encoding(content)
    if encoding is None:
        return None
    try:
        text = content.decode(encoding)
    except UnicodeDecodeError:
        return None
    return text if "\0" not in text and is_plausible_text(text) else None


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
    entries_by_object_id: dict[str, list[GitIndexEntry]] = {}
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
        entries_by_object_id.setdefault(entry.object_id, []).append(entry)

    object_ids = list(entries_by_object_id)
    blob_sizes = git_index_blob_sizes(repository, object_ids) if object_ids else {}
    bounded_object_ids = []
    for object_id in object_ids:
        if blob_sizes[object_id] <= MAX_TRACKED_BLOB_BYTES:
            bounded_object_ids.append(object_id)
            continue
        for entry in entries_by_object_id[object_id]:
            violations.append(
                PrivatePathViolation(entry.path, 1, "oversized_tracked_blob")
            )

    if bounded_object_ids:
        for object_id, content in git_index_blob_contents(
            repository, bounded_object_ids
        ):
            for entry in entries_by_object_id[object_id]:
                violations.extend(
                    content_violations(
                        content,
                        entry.path,
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


def reachable_object_path(object_id: str) -> Path:
    return Path(f"<reachable-object-{object_id}>")


def reachable_violation(
    violation: PrivatePathViolation, object_id: str
) -> PrivatePathViolation:
    return PrivatePathViolation(
        reachable_object_path(object_id),
        violation.line,
        f"reachable_{violation.category}",
    )


def scan_reachable_history(
    repository: Path,
    private_identifier_hashes: frozenset[str] = PRIVATE_IDENTIFIER_HASHES,
) -> list[PrivatePathViolation]:
    violations = []
    if git_repository_is_shallow(repository):
        violations.append(
            PrivatePathViolation(
                Path("<reachable-history>"),
                1,
                "reachable_history_is_shallow",
            )
        )

    objects = git_reachable_objects(repository)
    history_entries = git_history_entries(repository, objects)
    allowed_binary_object_ids = {
        entry.object_id
        for entry in history_entries
        if entry.path.as_posix() in ALLOWED_TRACKED_BINARY_FILES
    }
    disallowed_binary_object_ids = {
        entry.object_id
        for entry in history_entries
        if entry.path.as_posix() not in ALLOWED_TRACKED_BINARY_FILES
    }
    allowed_binary_object_ids.difference_update(disallowed_binary_object_ids)
    for entry in history_entries:
        marker_id = (
            entry.object_id if entry.object_type == "blob" else entry.snapshot_id
        )
        for violation in path_violations(entry.path, private_identifier_hashes):
            violations.append(reachable_violation(violation, marker_id))
        if is_prohibited_tracked_media(entry.path):
            violations.append(
                PrivatePathViolation(
                    reachable_object_path(marker_id),
                    1,
                    "reachable_tracked_private_media",
                )
            )
            continue
        if entry.mode == GITLINK_MODE and entry.object_type == "commit":
            continue
        if entry.mode not in GIT_BLOB_MODES or entry.object_type != "blob":
            violations.append(
                PrivatePathViolation(
                    reachable_object_path(marker_id),
                    1,
                    "reachable_unsupported_tracked_mode",
                )
            )

    bounded_object_ids = []
    for item in objects:
        if item.object_type == "tree":
            continue
        if item.object_id in allowed_binary_object_ids:
            continue
        if item.object_type not in GIT_SCANNABLE_OBJECT_TYPES:
            violations.append(
                PrivatePathViolation(
                    reachable_object_path(item.object_id),
                    1,
                    "reachable_unsupported_object_type",
                )
            )
            continue
        if item.size > MAX_TRACKED_BLOB_BYTES:
            violations.append(
                PrivatePathViolation(
                    reachable_object_path(item.object_id),
                    1,
                    "reachable_oversized_object",
                )
            )
            continue
        bounded_object_ids.append(item.object_id)

    if bounded_object_ids:
        for object_id, _, content in git_object_contents(
            repository,
            bounded_object_ids,
            GIT_SCANNABLE_OBJECT_TYPES,
        ):
            object_path = reachable_object_path(object_id)
            for violation in content_violations(
                content,
                object_path,
                private_identifier_hashes,
                fail_closed=True,
            ):
                violations.append(reachable_violation(violation, object_id))
    return violations


def scan_publication(
    repository: Path,
    private_identifier_hashes: frozenset[str] = PRIVATE_IDENTIFIER_HASHES,
) -> list[PrivatePathViolation]:
    violations = scan_repository(repository, private_identifier_hashes)
    violations.extend(
        scan_reachable_history(repository, private_identifier_hashes)
    )
    return violations


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(
        description="Check repository entries for private data."
    )
    parser.add_argument(
        "--release-history",
        action="store_true",
        help=(
            "also scan all objects reachable from local Git refs; required before "
            "a public or commercial release"
        ),
    )
    args = parser.parse_args(argv)
    repository = Path(__file__).resolve().parents[1]
    violations = (
        scan_publication(repository)
        if args.release_history
        else scan_repository(repository)
    )
    if violations:
        for violation in violations:
            print(f"{violation.path}:{violation.line}: {violation.category}")
        print(f"private-path guard: FAIL ({len(violations)} violation(s))")
        if args.release_history and any(
            violation.category.startswith("reachable_") for violation in violations
        ):
            print(
                "public/commercial release blocked: reachable Git history is not "
                "sanitized"
            )
            print(
                "recovery: rewrite every affected canonical ref outside the active "
                "gate, then remove pre-scrub refs and clones"
            )
            print(
                "recovery: make a fresh full clone, fetch and prune all refs and "
                "tags, rerun this guard, and record `git rev-parse HEAD`"
            )
            print(
                "recovery procedure: "
                "docs/setup/WINDOWS_TEST_LAB.md#blocked-release-history-recovery"
            )
        return 1
    completed = subprocess.run(
        ["git", "rev-parse", "HEAD"],
        cwd=repository,
        check=True,
        stdout=subprocess.PIPE,
        text=True,
    )
    commit = completed.stdout.strip()
    if args.release_history:
        print(
            "private-path guard: PASS "
            f"(staged, untracked, and reachable history at {commit})"
        )
    else:
        print(
            "private-path guard: PASS "
            f"(staged index and untracked worktree at {commit}; "
            "reachable history not audited)"
        )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
