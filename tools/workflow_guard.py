#!/usr/bin/env python3
"""Minimal workflow guard for guarded module builds.

This tool turns selected Markdown rules into deterministic checks. It is not a
general agent platform; it is a small local guard for this repository.
"""

from __future__ import annotations

import argparse
import json
import re
import sys
from dataclasses import dataclass, field
from pathlib import Path
from typing import Any, Dict, Iterable, List, Set


ROOT = Path(__file__).resolve().parents[1]
@dataclass
class GuardResult:
    command: str
    module_id: str
    passed: bool = True
    failures: List[str] = field(default_factory=list)
    warnings: List[str] = field(default_factory=list)

    def fail(self, message: str) -> None:
        self.passed = False
        self.failures.append(message)

    def warn(self, message: str) -> None:
        self.warnings.append(message)


@dataclass
class RustTestCase:
    name: str
    path: Path
    body: str
    ignored: bool


def read_text(path: Path) -> str:
    if not path.exists():
        return ""
    return path.read_text(encoding="utf-8")


def load_json(path: Path) -> Dict[str, Any]:
    if not path.exists():
        return {}
    return json.loads(read_text(path))


def module_dir(module_id: str) -> Path:
    return ROOT / "specs" / module_id


def discover_module_ids() -> List[str]:
    specs_dir = ROOT / "specs"
    if not specs_dir.exists():
        return []
    return sorted(
        path.parent.name
        for path in specs_dir.glob("*/module.contract.json")
        if path.is_file()
    )


def markdown_heading_exists(text: str, heading: str) -> bool:
    return bool(re.search(rf"^##+\s+.*\b{re.escape(heading)}\b.*$", text, re.M))


def engineering_rules_version(text: str) -> str:
    match = re.search(r"Engineering Rules Version:\s*([0-9A-Za-z.\-_]+)", text)
    if not match:
        return ""
    return match.group(1)


def extract_balanced_block(text: str, opening_brace_index: int) -> str:
    brace_balance = 0
    body_start = opening_brace_index + 1
    for idx in range(opening_brace_index, len(text)):
        char = text[idx]
        if char == "{":
            brace_balance += 1
        elif char == "}":
            brace_balance -= 1
            if brace_balance == 0:
                return text[body_start:idx]
    return text[body_start:]


def rust_test_cases() -> Dict[str, RustTestCase]:
    cases: Dict[str, RustTestCase] = {}
    test_paths: Set[Path] = set()
    for workspace_area in ("crates", "cli"):
        area_root = ROOT / workspace_area
        if area_root.exists():
            test_paths.update(area_root.glob("*/tests/**/*.rs"))
            test_paths.update(area_root.glob("*/src/**/*.rs"))
    for path in sorted(test_paths):
        text = read_text(path)
        for match in re.finditer(
            r"(?P<attrs>(?:\s*#\[[^\]]+\]\s*)*)\s*fn\s+(?P<name>[a-zA-Z0-9_]+)\s*\([^)]*\)\s*\{",
            text,
        ):
            attrs = match.group("attrs")
            if not re.search(r"#\[(?:[A-Za-z0-9_]+::)?test(?:\s|\(|\])", attrs):
                continue
            name = match.group("name")
            body = extract_balanced_block(text, match.end() - 1)
            cases[name] = RustTestCase(
                name=name,
                path=path,
                body=body,
                ignored="#[ignore" in attrs,
            )
    return cases


def rust_test_names() -> List[str]:
    return sorted(rust_test_cases().keys())


def rust_source_paths(contract: Dict[str, Any], key: str) -> List[Path]:
    paths: List[Path] = []
    seen: Set[Path] = set()
    for pattern in contract.get(key, []):
        pattern_text = str(pattern)
        matches = sorted(ROOT.glob(pattern_text))
        if not matches:
            candidate = ROOT / pattern_text
            if candidate.exists():
                matches = [candidate]
        for path in matches:
            if path.is_file() and path not in seen:
                paths.append(path)
                seen.add(path)
    return paths


def module_source_text(contract: Dict[str, Any]) -> str:
    chunks: List[str] = []
    for expected_source in contract.get("expected_source_files", []):
        path = ROOT / str(expected_source)
        if path.exists():
            chunks.append(read_text(path))
    return "\n".join(chunks)


def guarded_source_text(contract: Dict[str, Any]) -> str:
    chunks: List[str] = []
    for path in rust_source_paths(contract, "guarded_source_globs"):
        chunks.append(read_text(path))
    return "\n".join(chunks)


def extract_struct_fields(source: str, struct_name: str) -> Dict[str, str]:
    match = re.search(rf"struct\s+{re.escape(struct_name)}\s*\{{(?P<body>.*?)\n\}}", source, re.S)
    if not match:
        return {}
    fields: Dict[str, str] = {}
    for line in match.group("body").splitlines():
        field_match = re.search(r"\bpub\s+([a-zA-Z0-9_]+)\s*:\s*([^,]+)", line)
        if field_match:
            fields[field_match.group(1)] = " ".join(field_match.group(2).split())
    return fields


def public_struct_names(source: str) -> Set[str]:
    return set(re.findall(r"\bpub\s+struct\s+([a-zA-Z0-9_]+)\b", source))


def function_names(source: str) -> List[str]:
    return re.findall(r"\b(?:pub\s+)?fn\s+([a-zA-Z0-9_]+)\s*\(", source)


def function_line_lengths(source: str) -> Dict[str, int]:
    lengths: Dict[str, int] = {}
    lines = source.splitlines()
    for idx, line in enumerate(lines):
        match = re.search(r"\bfn\s+([a-zA-Z0-9_]+)\s*\(", line)
        if not match:
            continue
        name = match.group(1)
        brace_balance = line.count("{") - line.count("}")
        end_idx = idx
        while end_idx + 1 < len(lines) and brace_balance > 0:
            end_idx += 1
            brace_balance += lines[end_idx].count("{") - lines[end_idx].count("}")
        lengths[name] = end_idx - idx + 1
    return lengths


def require_literals(
    result: GuardResult,
    label: str,
    text: str,
    literals: Iterable[str],
) -> None:
    for literal in literals:
        if literal not in text:
            result.fail(f"{label} missing required literal: {literal!r}")


def normalize_code_body(text: str) -> str:
    without_line_comments = re.sub(r"//.*", "", text)
    without_block_comments = re.sub(r"/\*.*?\*/", "", without_line_comments, flags=re.S)
    return re.sub(r"\s+", "", without_block_comments)


def is_tautological_test(body: str) -> bool:
    normalized = normalize_code_body(body)
    tautologies = {
        "",
        "assert!(true);",
        "assert_eq!(1,1);",
        "assert_eq!(2+2,4);",
    }
    return normalized in tautologies


def is_checked_task(tasks_text: str, literal: str) -> bool:
    escaped = re.escape(literal)
    return bool(re.search(rf"^\s*-\s*\[[xX]\]\s+{escaped}\s*$", tasks_text, re.M))


def cargo_dependency_names(cargo_paths: Iterable[Path]) -> Set[str]:
    names: Set[str] = set()
    dependency_sections = {
        "dependencies",
        "dev-dependencies",
        "build-dependencies",
        "workspace.dependencies",
    }
    for cargo_path in cargo_paths:
        current_section = ""
        for line in read_text(cargo_path).splitlines():
            stripped = line.strip()
            if not stripped or stripped.startswith("#"):
                continue
            section_match = re.match(r"^\[([^\]]+)\]$", stripped)
            if section_match:
                current_section = section_match.group(1)
                continue
            if current_section not in dependency_sections:
                continue
            if "=" not in stripped:
                continue
            name = stripped.split("=", 1)[0].strip()
            if name.endswith(".workspace"):
                name = name.removesuffix(".workspace")
            if name and all(ch not in name for ch in " {}[]"):
                names.add(name)
    return names


def module_cargo_manifest_paths(contract: Dict[str, Any]) -> Set[Path]:
    manifests: Set[Path] = set()
    for configured_path in contract.get("dependency_manifest_paths", []):
        candidate = ROOT / str(configured_path)
        if candidate.exists():
            manifests.add(candidate)

    source_paths: Set[Path] = set()
    for key in ("expected_source_files", "quality_source_globs", "guarded_source_globs"):
        source_paths.update(rust_source_paths(contract, key))
    for source_path in source_paths:
        current = source_path.parent
        while True:
            candidate = current / "Cargo.toml"
            if candidate.exists():
                manifests.add(candidate)
                break
            if current == ROOT or current.parent == current:
                break
            current = current.parent
    return manifests


def check_required_doc_literacy(
    result: GuardResult,
    spec_dir: Path,
    contract: Dict[str, Any],
) -> None:
    min_chars = contract.get("required_doc_min_chars")
    if isinstance(min_chars, int):
        for doc_name in contract.get("required_docs", []):
            path = spec_dir / str(doc_name)
            if path.exists() and len(read_text(path).strip()) < min_chars:
                result.fail(f"required doc too small or empty: {path}")

    required_doc_literals = contract.get("required_doc_literals", {})
    if isinstance(required_doc_literals, dict):
        for doc_name, literals in required_doc_literals.items():
            text = read_text(spec_dir / str(doc_name))
            require_literals(result, str(doc_name), text, literals)


def check_hidden_uncertainty(
    result: GuardResult,
    combined_text: str,
    contract: Dict[str, Any],
) -> None:
    if not contract.get("hidden_uncertainty_gate", False):
        return
    allowed_markers = {
        str(marker).lower() for marker in contract.get("allowed_uncertainty_markers", [])
    }
    phrases = contract.get(
        "hidden_uncertainty_phrases",
        [
            "TBD",
            "Question left open",
            "do ustalenia",
            "nie wiadomo",
            "not sure",
            "unclear",
        ],
    )
    for phrase in phrases:
        phrase_text = str(phrase)
        if phrase_text.lower() in allowed_markers:
            continue
        if re.search(re.escape(phrase_text), combined_text, re.I):
            result.fail(f"hidden uncertainty phrase present without non-blocking marker: {phrase_text}")


def check_engineering_rules(
    result: GuardResult,
    contract: Dict[str, Any],
    tasks_text: str,
) -> None:
    rules_config = contract.get("engineering_rules")
    if not isinstance(rules_config, dict):
        result.fail("module.contract.json missing engineering_rules")
        return

    document = str(rules_config.get("document", ""))
    expected_version = str(rules_config.get("version", ""))
    if not document:
        result.fail("engineering_rules.document is missing")
        return

    rules_path = ROOT / document
    rules_text = read_text(rules_path)
    if not rules_text:
        result.fail(f"missing or empty engineering rules document: {rules_path}")
        return

    actual_version = engineering_rules_version(rules_text)
    if not actual_version:
        result.fail(f"{document} missing Engineering Rules Version")
    elif expected_version and actual_version != expected_version:
        result.fail(
            f"{document} version mismatch: expected {expected_version}, found {actual_version}"
        )

    for section in rules_config.get("required_sections", []):
        if not markdown_heading_exists(rules_text, str(section)):
            result.fail(f"{document} missing required section: {section}")

    for obligation in contract.get("required_quality_obligations", []):
        if str(obligation) not in tasks_text:
            result.fail(f"tasks.md missing required quality obligation: {obligation!r}")


def check_quality_limits(result: GuardResult, contract: Dict[str, Any]) -> None:
    quality_limits = contract.get("quality_limits", {})
    if not isinstance(quality_limits, dict):
        return

    quality_source_paths = rust_source_paths(contract, "quality_source_globs")
    if not quality_source_paths:
        quality_source_paths = [
            ROOT / str(path)
            for path in contract.get("expected_source_files", [])
            if (ROOT / str(path)).exists()
        ]

    max_source_file_lines = quality_limits.get("max_source_file_lines")
    if isinstance(max_source_file_lines, int):
        for path in quality_source_paths:
            line_count = len(read_text(path).splitlines())
            if line_count > max_source_file_lines:
                result.fail(
                    f"source file too long: {path} has {line_count} lines > {max_source_file_lines}"
                )

    max_function_lines = quality_limits.get("max_function_lines")
    if isinstance(max_function_lines, int):
        for path in quality_source_paths:
            for function_name, line_count in function_line_lengths(read_text(path)).items():
                if line_count > max_function_lines:
                    result.fail(
                        f"function too long: {path}:{function_name} has {line_count} lines > {max_function_lines}"
                    )


def check_module_ready(module_id: str) -> GuardResult:
    result = GuardResult("module-ready", module_id)
    spec_dir = module_dir(module_id)
    contract_path = spec_dir / "module.contract.json"
    contract = load_json(contract_path)

    if not spec_dir.exists():
        result.fail(f"module spec directory does not exist: {spec_dir}")
        return result

    if not contract:
        result.fail(f"missing or invalid module.contract.json: {contract_path}")
        return result

    for doc_name in contract.get("required_docs", []):
        path = spec_dir / str(doc_name)
        if not path.exists():
            result.fail(f"missing required doc: {path}")

    spec_text = read_text(spec_dir / "spec.md")
    plan_text = read_text(spec_dir / "plan.md")
    tasks_text = read_text(spec_dir / "tasks.md")
    fixture_text = read_text(spec_dir / "fixture-contract.md")
    combined_text = "\n".join([spec_text, plan_text, tasks_text, fixture_text])

    require_literals(
        result,
        "spec.md",
        spec_text,
        contract.get("required_spec_literals", []),
    )

    for marker in contract.get("blocking_markers", []):
        if str(marker) in combined_text:
            result.fail(f"blocking marker present: {marker!r}")

    for obligation in contract.get("required_obligations", []):
        if str(obligation) not in tasks_text:
            result.fail(f"tasks.md missing required obligation: {obligation!r}")

    check_required_doc_literacy(result, spec_dir, contract)
    check_hidden_uncertainty(result, combined_text, contract)
    check_engineering_rules(result, contract, tasks_text)

    if not result.failures:
        result.warn("module-ready checks passed; implementation may start.")

    return result


def check_verify_module(module_id: str) -> GuardResult:
    result = check_module_ready(module_id)
    result.command = "verify-module"
    if result.warnings == ["module-ready checks passed; implementation may start."]:
        result.warnings = ["module-ready subset passed; verifying implementation artifacts."]
    spec_dir = module_dir(module_id)
    contract = load_json(spec_dir / "module.contract.json")
    source = module_source_text(contract)
    guarded_source = guarded_source_text(contract)
    tests = rust_test_cases()
    tasks_text = read_text(spec_dir / "tasks.md")

    for expected_source in contract.get("expected_source_files", []):
        path = ROOT / str(expected_source)
        if not path.exists():
            result.fail(f"expected source file missing: {path}")

    if contract.get("required_checked_obligations", False):
        checked_obligations = []
        seen_obligations: Set[str] = set()
        for obligation in contract.get("required_obligations", []) + contract.get(
            "required_quality_obligations", []
        ):
            obligation_text = str(obligation)
            if obligation_text not in seen_obligations:
                checked_obligations.append(obligation_text)
                seen_obligations.add(obligation_text)

        for obligation in checked_obligations:
            if not is_checked_task(tasks_text, str(obligation)):
                result.fail(f"tasks.md required obligation not checked: {obligation!r}")

    for test_name in contract.get("required_tests", []):
        test = tests.get(str(test_name))
        if test is None:
            result.fail(f"required Rust test missing: {test_name}")
            continue
        if contract.get("required_tests_must_not_be_ignored", False) and test.ignored:
            result.fail(f"required Rust test is ignored: {test_name}")
        if contract.get("required_tests_must_have_substance", False) and is_tautological_test(test.body):
            result.fail(f"required Rust test has no meaningful substance: {test_name}")

    for field_name in contract.get("forbidden_output_fields", []):
        if re.search(rf"\b{re.escape(str(field_name))}\b", source):
            result.fail(f"forbidden output/responsibility field appears in source: {field_name}")

    for import_name in contract.get("forbidden_imports", []):
        if str(import_name) in source:
            result.fail(f"forbidden import/dependency appears in source: {import_name}")

    for pattern_config in contract.get("forbidden_source_patterns", []):
        if isinstance(pattern_config, dict):
            literal = str(pattern_config.get("literal", ""))
            label = str(pattern_config.get("label", literal))
        else:
            literal = str(pattern_config)
            label = literal
        if literal and literal in source:
            result.fail(f"forbidden source pattern appears in source: {label}")

    for pattern_config in contract.get("forbidden_guarded_source_patterns", []):
        if isinstance(pattern_config, dict):
            literal = str(pattern_config.get("literal", ""))
            label = str(pattern_config.get("label", literal))
        else:
            literal = str(pattern_config)
            label = literal
        if literal and literal in guarded_source:
            result.fail(f"forbidden guarded source pattern appears in source: {label}")

    for verb in contract.get("forbidden_scope_verbs", []):
        pattern = re.compile(rf"(^|_){re.escape(str(verb))}($|_)", re.I)
        for function_name in function_names(source):
            if pattern.search(function_name):
                result.fail(f"forbidden scope verb appears in function name: {function_name}")

    for term in contract.get("forbidden_scope_terms", []):
        pattern = re.compile(re.escape(str(term)), re.I)
        for public_name in sorted(public_struct_names(source).union(function_names(source))):
            if pattern.search(public_name):
                result.fail(f"forbidden scope term appears in public/module symbol: {public_name}")

    allowed_dependency_names = contract.get("allowed_dependency_names")
    if isinstance(allowed_dependency_names, list):
        allowed = {str(name) for name in allowed_dependency_names}
        manifests = module_cargo_manifest_paths(contract)
        for dependency_name in sorted(cargo_dependency_names(manifests) - allowed):
            result.fail(f"dependency not allowed by module contract: {dependency_name}")

    public_contract = contract.get("public_contract", {})
    if isinstance(public_contract, dict):
        expected_public_structs = set(str(type_name) for type_name in public_contract.keys())
        if contract.get("public_contract_mode") == "exact":
            actual_public_structs = public_struct_names(source)
            for extra_struct in sorted(actual_public_structs - expected_public_structs):
                result.fail(f"extra public struct not declared in contract: {extra_struct}")

        field_types_contract = contract.get("public_contract_field_types", {})
        if not isinstance(field_types_contract, dict):
            field_types_contract = {}

        for type_name, expected_fields in public_contract.items():
            actual_fields = extract_struct_fields(source, str(type_name))
            if not actual_fields:
                result.fail(f"public struct missing: {type_name}")
                continue
            expected_type_fields = field_types_contract.get(str(type_name), {})
            if isinstance(expected_fields, dict):
                expected_field_names = set(str(field_name) for field_name in expected_fields.keys())
                expected_type_fields = expected_fields
            else:
                expected_field_names = set(str(field_name) for field_name in expected_fields)

            for field_name in expected_fields:
                if field_name not in actual_fields:
                    result.fail(f"{type_name} missing public field: {field_name}")
            if contract.get("public_contract_mode") == "exact":
                for extra_field in sorted(set(actual_fields.keys()) - expected_field_names):
                    result.fail(f"{type_name} has extra public field not declared in contract: {extra_field}")

            if isinstance(expected_type_fields, dict):
                for field_name, expected_type in expected_type_fields.items():
                    actual_type = actual_fields.get(str(field_name))
                    if actual_type is None:
                        continue
                    if actual_type != str(expected_type):
                        result.fail(
                            f"{type_name}.{field_name} type mismatch: expected {expected_type}, found {actual_type}"
                        )

    check_quality_limits(result, contract)

    return result


def check_verify_all() -> GuardResult:
    result = GuardResult("verify-all", "all-modules")
    module_ids = discover_module_ids()
    if not module_ids:
        result.fail("no module.contract.json files found under specs/")
        return result

    for module_id in module_ids:
        module_result = check_verify_module(module_id)
        for failure in module_result.failures:
            result.fail(f"{module_id}: {failure}")
        for warning in module_result.warnings:
            result.warn(f"{module_id}: {warning}")

    if result.passed:
        result.warnings = [f"verified {len(module_ids)} module contracts"]
    return result


def emit_result(result: GuardResult, as_json: bool) -> int:
    if as_json:
        print(json.dumps(result.__dict__, indent=2))
    else:
        status = "PASS" if result.passed else "FAIL"
        print(f"{result.command} {result.module_id}: {status}")
        for warning in result.warnings:
            print(f"warning: {warning}")
        for failure in result.failures:
            print(f"failure: {failure}")
    return 0 if result.passed else 1


def main() -> int:
    parser = argparse.ArgumentParser(description="Guarded workflow checks")
    parser.add_argument("command", choices=["module-ready", "verify-module", "verify-all"])
    parser.add_argument("module_id", nargs="?")
    parser.add_argument("--json", action="store_true")
    args = parser.parse_args()

    if args.command == "verify-all":
        if args.module_id is not None:
            parser.error("verify-all does not accept a module_id")
        result = check_verify_all()
    elif args.module_id is None:
        parser.error(f"{args.command} requires a module_id")
    elif args.command == "module-ready":
        result = check_module_ready(args.module_id)
    else:
        result = check_verify_module(args.module_id)

    return emit_result(result, args.json)


if __name__ == "__main__":
    raise SystemExit(main())
