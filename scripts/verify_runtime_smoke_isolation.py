#!/usr/bin/env python3
"""Verify receipt-bound single, forward, and reversed runtime-smoke parity."""

from __future__ import annotations

import argparse
import hashlib
import json
import os
import tempfile
from pathlib import Path

from runtime_smoke.receipt import (
    CURRENT_SCHEMA,
    case_results_sha256,
    validate_runtime_smoke_receipt,
)


VERDICT_SCHEMA = "lay.runtime-smoke-isolation-verdict.v1"


def load_receipt(path: Path) -> tuple[dict[str, object], str]:
    raw = path.read_bytes()
    value = json.loads(raw)
    if not isinstance(value, dict):
        raise ValueError(f"receipt is not an object: {path}")
    validate_runtime_smoke_receipt(value)
    if value["schema"] != CURRENT_SCHEMA:
        raise ValueError(f"receipt is not current schema: {path}")
    if value["all_passed"] is not True:
        raise ValueError(f"receipt did not pass: {path}")
    return value, hashlib.sha256(raw).hexdigest()


def verify_isolation(
    single_paths: list[Path], forward_path: Path, reversed_path: Path
) -> dict[str, object]:
    if len(single_paths) != 2:
        raise ValueError("exactly two --single receipts are required")
    loaded = {
        "single_a": load_receipt(single_paths[0]),
        "single_b": load_receipt(single_paths[1]),
        "forward": load_receipt(forward_path),
        "reversed": load_receipt(reversed_path),
    }
    receipts = {role: value[0] for role, value in loaded.items()}
    run_ids = {str(receipt["run_id"]) for receipt in receipts.values()}
    if len(run_ids) != 1:
        raise ValueError("isolation receipts use different run IDs")

    single_cases: list[dict[str, object]] = []
    for role in ("single_a", "single_b"):
        cases = receipts[role]["cases"]
        if not isinstance(cases, list) or len(cases) != 1:
            raise ValueError(f"{role} must contain exactly one case")
        single_cases.append(cases[0])
    single_names = [str(row["name"]) for row in single_cases]
    if len(set(single_names)) != 2:
        raise ValueError("single receipts do not contain two unique cases")

    forward_order = receipts["forward"]["execution_order"]
    reversed_order = receipts["reversed"]["execution_order"]
    if forward_order != single_names:
        raise ValueError("forward execution order does not match singles")
    if reversed_order != list(reversed(single_names)):
        raise ValueError("reversed receipt does not bind the inverse order")

    projection = case_results_sha256(single_cases)
    for role in ("forward", "reversed"):
        receipt = receipts[role]
        if set(receipt["selected_cases"]) != set(single_names):
            raise ValueError(f"{role} selected-case set differs from singles")
        if receipt["case_results_sha256"] != projection:
            raise ValueError(f"{role} semantic projection differs from singles")

    paths = {
        "single_a": single_paths[0],
        "single_b": single_paths[1],
        "forward": forward_path,
        "reversed": reversed_path,
    }
    evidence = {}
    raw_trace_records = {}
    for role, receipt in receipts.items():
        evidence[role] = {
            "path": str(paths[role].resolve()),
            "sha256": loaded[role][1],
            "case_results_sha256": receipt["case_results_sha256"],
            "execution_order": receipt["execution_order"],
        }
        raw_trace_records[role] = {
            row["name"]: row["trace"]["records"] for row in receipt["cases"]
        }
    return {
        "schema": VERDICT_SCHEMA,
        "verdict": "RUNTIME_SMOKE_ORDER_ISOLATION_PASS",
        "run_id": run_ids.pop(),
        "cases": single_names,
        "semantic_projection_sha256": projection,
        "volatile_trace_kinds_excluded": ["ibus_cursor"],
        "raw_trace_records": raw_trace_records,
        "receipts": evidence,
    }


def write_json_atomic(path: Path, value: dict[str, object]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    handle, temporary = tempfile.mkstemp(prefix=f".{path.name}.", dir=path.parent)
    try:
        with os.fdopen(handle, "w", encoding="utf-8") as output:
            json.dump(value, output, ensure_ascii=False, indent=2, sort_keys=True)
            output.write("\n")
            output.flush()
            os.fsync(output.fileno())
        os.replace(temporary, path)
    except BaseException:
        Path(temporary).unlink(missing_ok=True)
        raise


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--single", action="append", type=Path, required=True)
    parser.add_argument("--forward", type=Path, required=True)
    parser.add_argument("--reversed", type=Path, required=True)
    parser.add_argument("--json-out", type=Path, required=True)
    args = parser.parse_args()
    verdict = verify_isolation(args.single, args.forward, args.reversed)
    write_json_atomic(args.json_out, verdict)
    print(f"runtime smoke isolation: {verdict['verdict']}")
    print(f"evidence: {args.json_out.resolve()}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
