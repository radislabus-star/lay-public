#!/usr/bin/env python3
"""Check canon links and explicit rule changes; never certify runtime behavior."""

from __future__ import annotations

import argparse
import json
import re
import subprocess
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
RULES = {f"C{number:02}" for number in range(1, 11)}
ENTRYPOINTS = (
    "AGENTS.md", "CLAUDE.md", "GEMINI.md", ".github/copilot-instructions.md",
)
PROTECTED = {
    "ARCHITECTURE.md", *ENTRYPOINTS,
    "scripts/check_architecture_canon.py", "tests/test_architecture_canon.py",
    "scripts/check-architecture.sh", "scripts/architecture_graph_gate.py",
    "scripts/architecture_scope_gate.py", "scripts/rust_source_scope.py",
    "scripts/check-lay-changed.sh", "scripts/check-lay-full.sh",
    "tests/text_mutation_monopoly_contract.rs",
    "tests/typing_transition_authority_contract.rs",
    "tests/text_edit_gate_contract.rs",
    "docs/architecture/accepted-space-autoflip-2026-09-28.md",
    "src/bin/lay_ibus_engine/layout_sync.rs",
    "src/bin/lay_ibus_engine/layout_sync/tests.rs",
    "src/bin/lay_ibus_engine/state.rs",
    "src/bin/lay_ibus_engine/committed_tail.rs",
    "src/bin/lay_ibus_engine/composition_commit.rs",
    "src/bin/lay_ibus_engine/context_admission.rs",
    "src/bin/lay_ibus_engine/context_admission/adapter.rs",
    "src/bin/lay_ibus_engine/ibus_interface.rs",
    "src/bin/lay_ibus_engine/managed.rs",
    "src/bin/lay_ibus_engine/tail_memory.rs",
    "src/bin/lay_ibus_engine/window_interaction/observation.rs",
    "extension/lay@radislabus-star.github.io/dbus_service.js",
    "extension/lay@radislabus-star.github.io/lay-impl.js",
    "tests/typing_assist_short_alternating.rs",
    "tests/fixtures/typing_assist_short_alternating_pairs.tsv",
    "scripts/test-lanes/manifest.json",
    "scripts/test-lanes/known_failures.json",
}
DECISIONS = "docs/architecture/decisions/"
LINK = re.compile(r"\[[^\]\n]+\]\(([^\s)]+)\)")


def linked_files(root: Path, name: str, errors: list[str]) -> set[str]:
    path = root / name
    if not path.is_file():
        errors.append(f"missing_file:{name}")
        return set()
    targets = set()
    for target in LINK.findall(path.read_text(encoding="utf-8")):
        if re.match(r"[a-zA-Z][a-zA-Z0-9+.-]*:", target) or target.startswith("#"):
            continue
        destination = (path.parent / target.split("#", 1)[0]).resolve()
        if not destination.is_relative_to(root):
            errors.append(f"link_outside_checkout:{name}:{target}")
            continue
        targets.add(destination.relative_to(root).as_posix())
        if not destination.is_file():
            errors.append(f"broken_link:{name}:{target}")
    return targets


def check_links(root: Path) -> list[str]:
    root = root.resolve()
    errors: list[str] = []
    linked_files(root, "ARCHITECTURE.md", errors)
    canon = root / "ARCHITECTURE.md"
    if canon.is_file():
        identifiers = re.findall(r"^## (C\d\d)\b", canon.read_text(encoding="utf-8"), re.M)
        if set(identifiers) != RULES or len(identifiers) != len(RULES):
            errors.append("canon_rule_ids:expected_C01_through_C10_once")
    for name in ENTRYPOINTS:
        targets = linked_files(root, name, errors)
        required = {"ARCHITECTURE.md"} | ({"AGENTS.md"} if name != "AGENTS.md" else set())
        for missing in sorted(required - targets):
            errors.append(f"entrypoint_missing_link:{name}:{missing}")
    return errors


def protected(name: str) -> bool:
    return name in PROTECTED or name.startswith(".github/workflows/")


def check_decisions(root: Path, changed: set[str], added: set[str]) -> list[str]:
    required = {name for name in changed if protected(name)}
    if not required:
        return []
    errors: list[str] = []
    covered: set[str] = set()
    for name in sorted(added):
        if not name.startswith(DECISIONS) or not name.endswith(".json"):
            continue
        try:
            record = json.loads((root / name).read_text(encoding="utf-8"))
            if not isinstance(record, dict) or record.get("schema") != "lay.architecture-decision.v1":
                raise ValueError("wrong schema")
            for field in ("decision", "reason", "verification", "not_tested"):
                if not isinstance(record.get(field), str) or not record[field].strip():
                    raise ValueError(f"missing {field}")
            for field in ("invariants", "protected_paths"):
                if (not isinstance(record.get(field), list) or not record[field]
                        or any(not isinstance(value, str) for value in record[field])):
                    raise ValueError(f"invalid {field}")
            if not set(record["invariants"]) <= RULES:
                raise ValueError("unknown invariant")
            if type(record.get("runtime_authority_changed")) is not bool:
                raise ValueError("runtime_authority_changed must be boolean")
            covered.update(record["protected_paths"])
        except (OSError, ValueError) as error:
            errors.append(f"invalid_new_decision:{name}:{error}")
    for name in sorted(required - covered):
        errors.append(f"protected_change_without_new_decision:{name}")
    return errors


def changes_since(root: Path, base: str) -> tuple[set[str], set[str]]:
    def git(*args: str) -> set[str]:
        result = subprocess.run(
            ["git", "-C", str(root), *args], check=True, capture_output=True, text=True,
        )
        return set(filter(None, result.stdout.split("\0")))

    # Empty-tree comparison for the first push of a branch/repository.
    if base and set(base) == {"0"}:
        base = "4b825dc642cb6eb9a060e54bf8d69288fbee4904"
    changed = git("diff", "--no-renames", "--name-only", "-z", base, "--")
    added = git("diff", "--no-renames", "--name-only", "--diff-filter=A", "-z", base, "--")
    untracked = git("ls-files", "--others", "--exclude-standard", "-z")
    return changed | untracked, added | untracked


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--root", type=Path, default=ROOT)
    parser.add_argument("--base", help="Git base for rule-change review; omitted means links only")
    args = parser.parse_args()
    if args.base is not None and not args.base.strip():
        parser.error("--base must be nonempty; omit it only for an explicit links-only check")
    root = args.root.resolve()
    errors = check_links(root)
    if args.base:
        try:
            errors.extend(check_decisions(root, *changes_since(root, args.base)))
        except subprocess.CalledProcessError as error:
            errors.append(f"base_diff_failed:{error.stderr.strip()}")
    for error in errors:
        print(f"canon_error={error}")
    print(f"canon_links_and_records={'FAIL' if errors else 'PASS'} "
          f"change_record={'CHECKED' if args.base else 'NOT_CHECKED'} "
          "runtime_behavior=NOT_TESTED")
    return bool(errors)


if __name__ == "__main__":
    raise SystemExit(main())
