#!/usr/bin/env python3
"""Check source-only architecture invariants with explicit route ownership."""

from __future__ import annotations

import argparse
import re
import sys
from pathlib import Path


SCRIPT_DIR = Path(__file__).resolve().parent
if str(SCRIPT_DIR) not in sys.path:
    sys.path.insert(0, str(SCRIPT_DIR))

from rust_source_scope import production_code_projection, production_rows


ROOT = Path(__file__).resolve().parents[1]

ABUSIVE_SNIPPET = re.compile(
    r"БЛЯТЬ|ДОЛБА|СУКА|ДИКТУ|ебан|хуе|пизд|YTN YJ|LJK<FT"
)
THREAD_SLEEP_CALL = re.compile(
    r"(?<![A-Za-z0-9_])(?:(?:std\s*::\s*)?thread\s*::\s*)sleep\s*\("
)

SLEEP_OWNER_PREFIXES = (
    "src/bin/lay_daemon/text_output/",
    "src/bin/lay_daemon/layout_controller/",
    "src/bin/lay_test_input/",
    "src/bin/lay_nanda_wave_train/",
)
SLEEP_OWNER_FILES = {
    "src/bin/lay_daemon/text_output.rs",
    "src/bin/lay_daemon/layout_controller.rs",
    "src/bin/lay_test_input.rs",
    "src/exact_layout_authority/proof.rs",
    "src/nanda_wave/context_phase/runtime_refresh.rs",
    "src/nanda_wave/lexical_grokking/training_budget.rs",
}

OPEN_OPTIONS_OWNERS = {
    "src/private_file.rs": "private-user-data",
    "src/text_edit/output_transaction.rs": "durable-output-journal",
    "src/nanda_wave/context_phase/composite.rs": "immutable-context-package",
    "src/nanda_wave/l4_cross_scene/format.rs": "immutable-l4-package",
    "src/nanda_wave/l4_cross_scene/incremental.rs": "l4-package-lock",
    "src/nanda_wave/lexical_grokking/composite.rs": "immutable-lexical-package",
}
PERMISSION_OWNERS = {
    "src/private_file.rs": "private-user-data",
    "src/bin/lay_l1_1_serve.rs": "unix-socket-mode",
}

RULE_ID_PATTERN = re.compile(
    r'"(?:moved_prefix_pair|split_word_pair|visual_b|personal_phrase|personal_token|'
    r'duplicate_layout_prefix|mixed_script_layout|layout_technical|layout_ru_to_en|'
    r'layout_en_to_ru|contextual_layout_en_to_ru|cyrillic_case|hard_sign|'
    r'adjacent_transposition|repeated_letter|single_letter_substitution|verb_ending|'
    r'vowel_confusion|extra_letters|missing_letter|glued_phrase)"'
)
RULE_ID_LITERAL_OWNERS = {
    "src/typing_rule_graph/ids.rs": "typing-rule-id-authority",
    "src/transition_relation.rs": "transition-relation-parser",
    "src/nanda_wave/l2_field/productive_v1/material_frame.rs": "wave-error-class",
    "src/nanda_wave/lexical_grokking/behavior_fingerprint.rs": "wave-error-class",
    "src/nanda_wave/lexical_grokking/corruption.rs": "wave-error-class",
    "src/nanda_wave/lexical_grokking/proof_matrix.rs": "wave-proof-class",
    "src/nanda_wave/lexical_grokking/typed_edit_traversal.rs": "wave-error-class",
    "src/nanda_wave/self_teacher_l3.rs": "wave-training-class",
}
NON_RUNTIME_RULE_TOOL_PREFIXES = (
    "src/bin/lay_nanda_dataset.rs",
    "src/bin/lay_nanda_wave_train",
    "src/bin/lay_nanda_wave_eval/",
)
PROOF_ONLY_SOURCE_FILES = {
    "src/bin/lay_ibus_engine/atomic/proof.rs",
    "src/bin/lay_ibus_engine/space_autocorrect_prefetch/proof.rs",
    "src/exact_layout_authority/proof.rs",
    "src/nanda_wave/context_phase/proof.rs",
    "src/nanda_wave/l2_field/productive_v1/proof.rs",
    "src/nanda_wave/l2_field/proof.rs",
    "src/nanda_wave/l4_cross_scene/proof.rs",
    "src/nanda_wave/lexical_grokking/proof.rs",
    "src/nanda_wave/morphology_phase/proof.rs",
}


def relative(path: Path, root: Path) -> str:
    return path.relative_to(root).as_posix()


def rust_sources(root: Path) -> list[Path]:
    source = root / "src"
    return sorted(source.rglob("*.rs")) if source.is_dir() else []


def line_rows(path: Path) -> list[tuple[int, str]]:
    return list(enumerate(path.read_text(encoding="utf-8").splitlines(), 1))


def is_test_or_proof_source(path: Path, root: Path) -> bool:
    name = relative(path, root)
    return (
        name.startswith("tests/")
        or "/tests/" in name
        or name.endswith("_tests.rs")
        or name.endswith("/tests.rs")
        or name in PROOF_ONLY_SOURCE_FILES
        or name.startswith("src/bin/lay_test_input")
        or name.startswith("src/bin/lay_nanda_wave_eval")
    )


def active_text_files(root: Path) -> list[Path]:
    paths = [path for path in rust_sources(root) if not is_test_or_proof_source(path, root)]
    for directory in (root / "extension",):
        if directory.is_dir():
            paths.extend(
                path
                for path in directory.rglob("*")
                if path.is_file() and path.suffix in {".js", ".json", ".py", ".sh"}
            )
    for name in ("install.sh", "update.sh", "README.md", "HOW_IT_WORKS.md"):
        path = root / name
        if path.is_file():
            paths.append(path)
    return sorted(set(paths))


def find_abusive_snippets(root: Path = ROOT) -> list[str]:
    violations: list[str] = []
    for path in active_text_files(root):
        source = path.read_text(encoding="utf-8")
        if ABUSIVE_SNIPPET.search(source) is None:
            continue
        rows = production_rows(source) if path.suffix == ".rs" else line_rows(path)
        for line_number, line in rows:
            if ABUSIVE_SNIPPET.search(line):
                violations.append(
                    f"{relative(path, root)}:{line_number}:abusive_snippet"
                )
    return violations


def sleep_owner(path: Path, root: Path, line_number: int, lines: list[str]) -> str | None:
    name = relative(path, root)
    if is_test_or_proof_source(path, root):
        return "test-or-proof"
    if name in SLEEP_OWNER_FILES:
        return "explicit-delay-owner"
    if any(name.startswith(prefix) for prefix in SLEEP_OWNER_PREFIXES):
        return "explicit-delay-owner"
    return None


def find_sleep_violations(root: Path = ROOT) -> list[str]:
    violations: list[str] = []
    for path in rust_sources(root):
        source = path.read_text(encoding="utf-8")
        if "sleep" not in source:
            continue
        lines = source.splitlines()
        code = production_code_projection(source)
        for match in THREAD_SLEEP_CALL.finditer(code):
            line_number = code.count("\n", 0, match.start()) + 1
            if sleep_owner(path, root, line_number, lines) is None:
                violations.append(
                    f"{relative(path, root)}:{line_number}:unowned_thread_sleep"
                )
    return violations


def file_operation_owner(
    path: Path,
    root: Path,
    line_number: int,
    token: str,
    lines: list[str],
) -> str | None:
    name = relative(path, root)
    if is_test_or_proof_source(path, root):
        return "test-or-proof"
    owners = OPEN_OPTIONS_OWNERS if token == "OpenOptionsExt" else PERMISSION_OWNERS
    if name in owners:
        return owners[name]
    return None


def find_file_operation_violations(root: Path = ROOT) -> list[str]:
    return find_open_options_violations(root) + find_permission_violations(root)


def find_open_options_violations(root: Path = ROOT) -> list[str]:
    return find_operation_token_violations(root, ("OpenOptionsExt",))


def find_permission_violations(root: Path = ROOT) -> list[str]:
    return find_operation_token_violations(root, ("PermissionsExt", "set_permissions"))


def find_operation_token_violations(root: Path, tokens: tuple[str, ...]) -> list[str]:
    violations: list[str] = []
    for path in rust_sources(root):
        source = path.read_text(encoding="utf-8")
        if not any(token in source for token in tokens):
            continue
        lines = source.splitlines()
        for line_number, line in production_rows(source):
            for token in tokens:
                if token not in line:
                    continue
                if file_operation_owner(path, root, line_number, token, lines) is None:
                    violations.append(
                        f"{relative(path, root)}:{line_number}:unowned_{token}"
                    )
    return violations


def find_runtime_tmp_path_violations(root: Path = ROOT) -> list[str]:
    violations: list[str] = []
    for path in active_text_files(root):
        source = path.read_text(encoding="utf-8")
        if "/tmp/lay" not in source:
            continue
        rows = production_rows(source) if path.suffix == ".rs" else line_rows(path)
        for line_number, line in rows:
            if "/tmp/lay" not in line:
                continue
            violations.append(
                f"{relative(path, root)}:{line_number}:runtime_tmp_lay_path"
            )
    return violations


def active_rust_sources(root: Path = ROOT):
    for path in rust_sources(root):
        if is_test_or_proof_source(path, root):
            continue
        yield path, path.read_text(encoding="utf-8")


def find_production_literal_violations(pattern: str, root: Path = ROOT) -> list[str]:
    violations: list[str] = []
    for path, source in active_rust_sources(root):
        if pattern not in source:
            continue
        violations.extend(
            f"{relative(path, root)}:{line_number}:production_literal:{pattern}"
            for line_number, line in production_rows(source)
            if pattern in line
        )
    return violations


def find_call_owner_violations(
    name: str,
    allowed: tuple[str, ...],
    root: Path = ROOT,
) -> list[str]:
    name = name.removesuffix("(")
    call = re.compile(rf"(?<![A-Za-z0-9_]){re.escape(name)}\s*\(")
    violations: list[str] = []
    for path, source in active_rust_sources(root):
        if name not in source:
            continue
        path_name = relative(path, root)
        if any(path_name == owner or (owner.endswith("/") and path_name.startswith(owner))
               for owner in allowed):
            continue
        code = production_code_projection(source)
        violations.extend(
            f"{path_name}:{code.count(chr(10), 0, match.start()) + 1}:unowned_call:{name}"
            for match in call.finditer(code)
        )
    return violations


def find_runtime_rule_id_violations(root: Path = ROOT) -> list[str]:
    violations: list[str] = []
    for path, source in active_rust_sources(root):
        path_name = relative(path, root)
        if path_name in RULE_ID_LITERAL_OWNERS or any(
            path_name.startswith(prefix) for prefix in NON_RUNTIME_RULE_TOOL_PREFIXES
        ):
            continue
        if RULE_ID_PATTERN.search(source) is None:
            continue
        violations.extend(
            f"{path_name}:{line_number}:unowned_runtime_rule_id"
            for line_number, line in production_rows(source)
            if RULE_ID_PATTERN.search(line) is not None
        )
    return violations


def violations_for(check: str, root: Path) -> list[str]:
    if check == "abusive":
        return find_abusive_snippets(root)
    if check == "sleep":
        return find_sleep_violations(root)
    if check == "file-operations":
        return find_file_operation_violations(root)
    if check == "open-options":
        return find_open_options_violations(root)
    if check == "permissions":
        return find_permission_violations(root)
    if check == "tmp-path":
        return find_runtime_tmp_path_violations(root)
    if check == "runtime-rule-ids":
        return find_runtime_rule_id_violations(root)
    return (
        find_abusive_snippets(root)
        + find_sleep_violations(root)
        + find_file_operation_violations(root)
        + find_runtime_tmp_path_violations(root)
    )


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument(
        "check",
        choices=(
            "abusive",
            "sleep",
            "open-options",
            "permissions",
            "file-operations",
            "tmp-path",
            "runtime-rule-ids",
            "production-literal",
            "call-owner",
            "all",
        ),
        nargs="?",
        default="all",
    )
    parser.add_argument("--root", type=Path, default=ROOT)
    parser.add_argument("--pattern")
    parser.add_argument("--allow", action="append", default=[])
    args = parser.parse_args()
    root = args.root.resolve()
    if args.check in {"production-literal", "call-owner"} and not args.pattern:
        parser.error(f"{args.check} requires --pattern")
    if args.check == "production-literal":
        violations = find_production_literal_violations(args.pattern, root)
    elif args.check == "call-owner":
        violations = find_call_owner_violations(args.pattern, tuple(args.allow), root)
    else:
        violations = violations_for(args.check, root)
    for violation in violations:
        print(violation)
    return 1 if violations else 0


if __name__ == "__main__":
    raise SystemExit(main())
