#!/usr/bin/env python3
"""Normalize and verify Lay's compiler diagnostic inventory."""

from __future__ import annotations

import argparse
import json
import pathlib
import subprocess
import sys
import tempfile
from typing import Any


SCHEMA = "lay.dead-code-baseline.v4"
LEGACY_SCHEMA = "lay.dead-code-baseline.v3"
SCOPES = {
    "default": {
        "cargo_args": ["check", "--locked", "--all-targets"],
        "features": "default",
    },
    "research-tools": {
        "cargo_args": [
            "check",
            "--locked",
            "--all-targets",
            "--features",
            "research-tools",
        ],
        "features": "research-tools",
    },
}


def canonical(value: Any) -> str:
    return json.dumps(value, ensure_ascii=True, sort_keys=True, separators=(",", ":"))


def toolchain_identity() -> dict[str, str]:
    rustc = subprocess.run(
        ["rustc", "-Vv"], check=True, capture_output=True, text=True
    ).stdout
    fields = {}
    for line in rustc.splitlines():
        if ": " in line:
            key, value = line.split(": ", 1)
            fields[key] = value
    return {
        "cargo": subprocess.run(
            ["cargo", "-V"], check=True, capture_output=True, text=True
        ).stdout.strip(),
        "clippy": subprocess.run(
            ["clippy-driver", "-V"], check=True, capture_output=True, text=True
        ).stdout.strip(),
        "rustc_commit": fields.get("commit-hash", ""),
        "rustc_release": fields.get("release", ""),
    }


def normalized_span_text(span: dict[str, Any]) -> list[str]:
    return [
        row.get("text", "").strip()
        for row in span.get("text", [])
        if row.get("text", "").strip()
    ]


def diagnostic_subjects(diagnostic: dict[str, Any]) -> list[dict[str, Any]]:
    subjects: dict[str, dict[str, Any]] = {}
    for span in diagnostic.get("spans", []):
        if not span.get("is_primary") and not span.get("label"):
            continue
        subject = {
            "file": span.get("file_name", "<unknown>"),
            "is_primary": bool(span.get("is_primary")),
            "label": span.get("label") or "",
            "text": normalized_span_text(span),
        }
        subjects[canonical(subject)] = subject
    if not subjects:
        fallback = {
            "file": "<unknown>",
            "is_primary": True,
            "label": "",
            "text": [],
        }
        subjects[canonical(fallback)] = fallback
    return [subjects[key] for key in sorted(subjects)]


def diagnostic_locations(diagnostic: dict[str, Any]) -> list[dict[str, Any]]:
    spans = []
    for span in diagnostic.get("spans", []):
        if not span.get("is_primary"):
            continue
        byte_start = span.get("byte_start")
        byte_end = span.get("byte_end")
        if not isinstance(byte_start, int) or not isinstance(byte_end, int):
            raise ValueError("dead_code diagnostic lacks a primary byte range")
        spans.append(
            {
                "file": span.get("file_name", "<unknown>"),
                "byte_start": byte_start,
                "byte_end": byte_end,
            }
        )
    if not spans:
        raise ValueError("dead_code diagnostic lacks a primary source location")
    return sorted(spans, key=canonical)


def diagnostic_location(diagnostic: dict[str, Any]) -> str:
    return canonical(diagnostic_locations(diagnostic))


def diagnostic_code(diagnostic: dict[str, Any]) -> str:
    return (diagnostic.get("code") or {}).get("code") or "<none>"


def normalized_dead_entry(message: dict[str, Any]) -> dict[str, Any]:
    diagnostic = message["message"]
    target = message.get("target") or {}
    return {
        "code": "dead_code",
        "message": diagnostic.get("message", ""),
        "locations": diagnostic_locations(diagnostic),
        "subjects": diagnostic_subjects(diagnostic),
        "target": {
            "crate_types": sorted(target.get("crate_types") or []),
            "kind": sorted(target.get("kind") or []),
            "name": target.get("name", "<unknown>"),
        },
    }


def read_diagnostics(path: pathlib.Path) -> tuple[list[dict[str, Any]], list[dict[str, Any]]]:
    dead_entries: dict[str, tuple[dict[str, Any], set[str]]] = {}
    violations: list[dict[str, Any]] = []
    build_finished = False
    for line_number, line in enumerate(path.read_text().splitlines(), 1):
        if build_finished:
            raise ValueError(f"{path}:{line_number}: content after Cargo build-finished")
        try:
            message = json.loads(line)
        except json.JSONDecodeError as error:
            raise ValueError(f"{path}:{line_number}: invalid Cargo JSON: {error}") from error
        if message.get("reason") == "build-finished":
            if message.get("success") is not True:
                raise ValueError(f"{path}: Cargo build-finished reported failure")
            build_finished = True
            continue
        if message.get("reason") != "compiler-message":
            continue
        diagnostic = message.get("message") or {}
        level = diagnostic.get("level")
        if level not in {"warning", "error"}:
            continue
        code = diagnostic_code(diagnostic)
        if level == "warning" and code == "dead_code":
            entry = normalized_dead_entry(message)
            key = canonical(entry)
            stored_entry, locations = dead_entries.setdefault(key, (entry, set()))
            locations.add(diagnostic_location(diagnostic))
            dead_entries[key] = (stored_entry, locations)
            continue
        spans = [span for span in diagnostic.get("spans", []) if span.get("is_primary")]
        violations.append(
            {
                "code": code,
                "file": spans[0].get("file_name", "<unknown>") if spans else "<unknown>",
                "level": level,
                "line": spans[0].get("line_start", 0) if spans else 0,
                "message": diagnostic.get("message", ""),
                "target": (message.get("target") or {}).get("name", "<unknown>"),
            }
        )
    if not build_finished:
        raise ValueError(f"{path}: missing successful Cargo build-finished record")
    inventory = []
    for key in sorted(dead_entries):
        entry, locations = dead_entries[key]
        for occurrence in range(1, len(locations) + 1):
            inventory.append({**entry, "occurrence": occurrence})
    return sorted(inventory, key=canonical), violations


def print_violations(violations: list[dict[str, Any]]) -> None:
    for row in violations[:80]:
        print(
            f"{row['level']}[{row['code']}] {row['file']}:{row['line']} "
            f"({row['target']}): {row['message']}",
            file=sys.stderr,
        )
    if len(violations) > 80:
        print(f"... {len(violations) - 80} more diagnostics", file=sys.stderr)


def load_baseline(
    path: pathlib.Path, *, allow_legacy: bool = False
) -> dict[str, Any]:
    payload = json.loads(path.read_text())
    accepted_schemas = {SCHEMA, LEGACY_SCHEMA} if allow_legacy else {SCHEMA}
    if payload.get("schema") not in accepted_schemas:
        raise ValueError(f"{path}: unexpected schema {payload.get('schema')!r}")
    entries = payload.get("entries")
    if not isinstance(entries, list):
        raise ValueError(f"{path}: entries must be a list")
    keys = [canonical(entry) for entry in entries]
    if keys != sorted(set(keys)):
        raise ValueError(f"{path}: entries must be unique and canonically sorted")
    return payload


def compare_inventory(
    current: list[dict[str, Any]], baseline: dict[str, Any]
) -> tuple[list[dict[str, Any]], list[dict[str, Any]]]:
    current_by_key = {canonical(entry): entry for entry in current}
    expected_by_key = {canonical(entry): entry for entry in baseline["entries"]}
    added = [current_by_key[key] for key in sorted(current_by_key.keys() - expected_by_key)]
    removed = [expected_by_key[key] for key in sorted(expected_by_key.keys() - current_by_key)]
    return added, removed


def logical_inventory(entries: list[dict[str, Any]]) -> list[dict[str, Any]]:
    """Project exact-span rows onto stable logical identities for baseline writes."""
    grouped: dict[str, tuple[dict[str, Any], int]] = {}
    for entry in entries:
        logical = {
            key: value
            for key, value in entry.items()
            if key not in {"locations", "occurrence"}
        }
        key = canonical(logical)
        stored, count = grouped.get(key, (logical, 0))
        grouped[key] = (stored, count + 1)
    projected = []
    for key in sorted(grouped):
        entry, count = grouped[key]
        for occurrence in range(1, count + 1):
            projected.append({**entry, "occurrence": occurrence})
    return sorted(projected, key=canonical)


def validate_baseline_contract(baseline: dict[str, Any], feature_scope: str) -> None:
    if baseline.get("scope") != SCOPES[feature_scope]:
        raise ValueError("dead-code baseline scope drift")
    if baseline.get("toolchain") != toolchain_identity():
        raise ValueError("dead-code baseline toolchain drift")


def require_monotonic_reduction(
    current: list[dict[str, Any]], baseline: dict[str, Any]
) -> list[dict[str, Any]]:
    current_logical = logical_inventory(current)
    baseline_logical = logical_inventory(baseline["entries"])
    added, removed = compare_inventory(
        current_logical, {"entries": baseline_logical}
    )
    if added:
        preview = "\n".join(f"dead_code ADDED: {canonical(row)}" for row in added[:40])
        raise ValueError(
            f"dead-code baseline may only decrease; added={len(added)}\n{preview}"
        )
    return removed


def write_baseline(
    path: pathlib.Path, entries: list[dict[str, Any]], feature_scope: str
) -> None:
    payload = {
        "schema": SCHEMA,
        "scope": SCOPES[feature_scope],
        "toolchain": toolchain_identity(),
        "entries": entries,
    }
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n")


def verify_inventory(
    input_path: pathlib.Path, baseline_path: pathlib.Path, feature_scope: str
) -> int:
    current, violations = read_diagnostics(input_path)
    if violations:
        print_violations(violations)
        return 1
    baseline = load_baseline(baseline_path)
    validate_baseline_contract(baseline, feature_scope)
    added, removed = compare_inventory(current, baseline)
    if added or removed:
        for label, rows in (("ADDED", added), ("REMOVED", removed)):
            for row in rows[:40]:
                print(f"dead_code {label}: {canonical(row)}", file=sys.stderr)
        print(
            f"dead-code baseline mismatch: added={len(added)} removed={len(removed)}",
            file=sys.stderr,
        )
        return 1
    print(f"dead_code_inventory={len(current)} baseline=PASS")
    return 0


def verify_clean(input_path: pathlib.Path) -> int:
    dead_entries, violations = read_diagnostics(input_path)
    if dead_entries:
        print("clippy emitted dead_code despite the explicit allow", file=sys.stderr)
        return 1
    if violations:
        print_violations(violations)
        return 1
    print("non_dead_diagnostics=0")
    return 0


def self_test() -> None:
    target = {"name": "lay", "kind": ["lib"], "crate_types": ["lib"]}
    build_finished = json.dumps({"reason": "build-finished", "success": True})

    def message(
        code: str,
        text: str,
        file_name: str = "src/lib.rs",
        subject: str = "fn old() {}",
        byte_start: int = 10,
    ) -> str:
        return json.dumps(
            {
                "reason": "compiler-message",
                "target": target,
                "message": {
                    "code": {"code": code},
                    "level": "warning",
                    "message": text,
                    "spans": [
                        {
                            "file_name": file_name,
                            "is_primary": True,
                            "label": None,
                            "line_start": 1,
                            "byte_start": byte_start,
                            "byte_end": byte_start + len(subject),
                            "text": [{"text": subject}],
                        }
                    ],
                },
            }
        )

    def stream(*messages: str) -> str:
        return "\n".join((*messages, build_finished, ""))

    with tempfile.TemporaryDirectory() as directory:
        path = pathlib.Path(directory) / "messages.jsonl"
        path.write_text(stream(message("dead_code", "function `old` is never used")))
        current, violations = read_diagnostics(path)
        assert not violations and len(current) == 1
        baseline = {"entries": current}
        assert compare_inventory(current, baseline) == ([], [])

        path.write_text(
            stream(
                message("dead_code", "function `old` is never used"),
                message(
                    "dead_code",
                    "function `new` is never used",
                    "src/new.rs",
                    "fn new() {}",
                ),
            )
        )
        added_current, _ = read_diagnostics(path)
        added, removed = compare_inventory(added_current, baseline)
        assert len(added) == 1 and not removed

        added_rows, removed_rows = compare_inventory([], baseline)
        assert not added_rows and len(removed_rows) == 1

        path.write_text(
            stream(
                message(
                    "dead_code",
                    "function `old` is never used",
                    "src/renamed.rs",
                )
            )
        )
        moved_current, _ = read_diagnostics(path)
        added_rows, removed_rows = compare_inventory(moved_current, baseline)
        assert len(added_rows) == 1 and len(removed_rows) == 1

        duplicate = message("dead_code", "function `old` is never used")
        path.write_text(stream(duplicate, duplicate))
        duplicate_current, _ = read_diagnostics(path)
        assert duplicate_current == current

        generic = "multiple fields are never read"
        path.write_text(
            stream(
                message("dead_code", generic, subject="field_a: usize,"),
                message(
                    "dead_code",
                    generic,
                    subject="field_b: usize,",
                    byte_start=30,
                ),
            )
        )
        distinct_current, _ = read_diagnostics(path)
        assert len(distinct_current) == 2

        path.write_text(
            stream(
                message("dead_code", generic, subject="field: usize,", byte_start=10),
                message("dead_code", generic, subject="field: usize,", byte_start=30),
            )
        )
        repeated_shape_current, _ = read_diagnostics(path)
        assert len(repeated_shape_current) == 2
        assert [row["occurrence"] for row in repeated_shape_current] == [1, 1]
        assert repeated_shape_current[0]["locations"] != repeated_shape_current[1]["locations"]

        path.write_text(stream(message("unused_imports", "unused import: `Old`")))
        _, violations = read_diagnostics(path)
        assert len(violations) == 1 and violations[0]["code"] == "unused_imports"

        for incomplete in ("", message("dead_code", "function `old` is never used")):
            path.write_text(incomplete)
            try:
                read_diagnostics(path)
            except ValueError as error:
                assert "build-finished" in str(error)
            else:
                raise AssertionError("incomplete Cargo stream was accepted")

        path.write_text(
            build_finished
            + "\n"
            + message("dead_code", "function `late` is never used")
            + "\n"
        )
        try:
            read_diagnostics(path)
        except ValueError as error:
            assert "content after" in str(error)
        else:
            raise AssertionError("post-build-finished content was accepted")

        monotonic_baseline = {
            "entries": repeated_shape_current,
        }
        reduced = repeated_shape_current[:1]
        assert len(require_monotonic_reduction(reduced, monotonic_baseline)) == 1
        try:
            require_monotonic_reduction(
                repeated_shape_current
                + [{**repeated_shape_current[-1], "occurrence": 3}],
                monotonic_baseline,
            )
        except ValueError as error:
            assert "may only decrease" in str(error)
        else:
            raise AssertionError("baseline writer accepted a dead-code addition")

        other_path = pathlib.Path(directory) / "other.jsonl"
        path.write_text(
            stream(
                message(
                    "dead_code",
                    "function `same` is never used",
                    subject="fn same() {}",
                    byte_start=10,
                )
            )
        )
        other_path.write_text(
            stream(
                message(
                    "dead_code",
                    "function `same` is never used",
                    subject="fn same() {}",
                    byte_start=30,
                )
            )
        )
        old_location, _ = read_diagnostics(path)
        new_location, _ = read_diagnostics(other_path)
        added_rows, removed_rows = compare_inventory(
            new_location, {"entries": old_location}
        )
        assert len(added_rows) == 1 and len(removed_rows) == 1
        assert require_monotonic_reduction(
            new_location, {"entries": old_location}
        ) == []

    print("lint_inventory_self_test=PASS")


def main() -> int:
    parser = argparse.ArgumentParser()
    subparsers = parser.add_subparsers(dest="command", required=True)
    inventory = subparsers.add_parser("inventory")
    inventory.add_argument("--input", type=pathlib.Path, required=True)
    inventory.add_argument("--baseline", type=pathlib.Path, required=True)
    inventory.add_argument(
        "--feature-scope", choices=sorted(SCOPES), required=True
    )
    inventory.add_argument("--write-to", type=pathlib.Path)
    clean = subparsers.add_parser("clean")
    clean.add_argument("--input", type=pathlib.Path, required=True)
    subparsers.add_parser("self-test")
    args = parser.parse_args()

    try:
        if args.command == "self-test":
            self_test()
            return 0
        if args.command == "clean":
            return verify_clean(args.input)
        entries, violations = read_diagnostics(args.input)
        if violations:
            print_violations(violations)
            return 1
        if args.write_to is not None:
            baseline = load_baseline(args.baseline, allow_legacy=True)
            validate_baseline_contract(baseline, args.feature_scope)
            removed = require_monotonic_reduction(entries, baseline)
            write_baseline(args.write_to, entries, args.feature_scope)
            print(
                f"dead_code_inventory={len(entries)} baseline=WRITTEN "
                f"removed={len(removed)}"
            )
            return 0
        return verify_inventory(args.input, args.baseline, args.feature_scope)
    except (OSError, ValueError) as error:
        print(f"lint inventory error: {error}", file=sys.stderr)
        return 1


if __name__ == "__main__":
    raise SystemExit(main())
