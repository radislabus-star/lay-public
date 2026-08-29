#!/usr/bin/env python3
"""Fail-closed analyzer for the fixed managed-IME immediate-Space replay."""

from __future__ import annotations

import argparse
import hashlib
import json
import math
import os
import tempfile
from collections import Counter, defaultdict
from pathlib import Path
from typing import Any

from runtime_smoke.receipt import (
    SCHEMA_V2,
    SCHEMA_V3,
    validate_runtime_smoke_receipt,
)


MAX_TRACE_BYTES = 8 * 1024 * 1024
LEASE_OUTCOMES = {"ready", "not_ready", "stale", "unauthorized", "applied"}
ROUTE_FIELDS = {
    "projection",
    "outcome",
    "worker_generation",
    "tail_epoch",
    "engine_path",
    "field_producer_count",
    "field_cache_disposition",
    "field_generation",
    "l11_us",
    "productive_v90_us",
    "display_l3_us",
    "semantic_l3_us",
    "correction_l3_us",
    "space_lookup_wait_us",
    "decision_total_us",
    "correction_total_us",
    "candidates",
}
LEASE_FIELDS = {
    "outcome",
    "worker_generation",
    "tail_epoch",
    "engine_path",
    "space_lookup_wait_us",
}


class AnalysisError(RuntimeError):
    pass


def sha256(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as source:
        for chunk in iter(lambda: source.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def text_sha256(value: str) -> str:
    return hashlib.sha256(value.encode("utf-8")).hexdigest()


def load_json(path: Path) -> dict[str, Any]:
    try:
        value = json.loads(path.read_text(encoding="utf-8"))
    except (OSError, UnicodeDecodeError, json.JSONDecodeError) as error:
        raise AnalysisError(f"cannot read JSON {path}: {error}") from error
    if not isinstance(value, dict):
        raise AnalysisError(f"JSON root must be an object: {path}")
    return value


def load_trace(path: Path) -> list[dict[str, Any]]:
    try:
        size = path.stat().st_size
    except OSError as error:
        raise AnalysisError(f"cannot stat trace {path}: {error}") from error
    if size == 0:
        raise AnalysisError("trace is empty")
    if size > MAX_TRACE_BYTES:
        raise AnalysisError(f"trace exceeds {MAX_TRACE_BYTES} bytes: {size}")

    records: list[dict[str, Any]] = []
    try:
        lines = path.read_text(encoding="utf-8").splitlines()
    except (OSError, UnicodeDecodeError) as error:
        raise AnalysisError(f"cannot read trace {path}: {error}") from error
    for line_number, line in enumerate(lines, 1):
        if not line.strip():
            continue
        try:
            record = json.loads(line)
        except json.JSONDecodeError as error:
            raise AnalysisError(f"invalid JSONL row {line_number}: {error}") from error
        if not isinstance(record, dict) or not isinstance(record.get("kind"), str):
            raise AnalysisError(f"trace row {line_number} is not a typed object")
        record["_line_index"] = line_number
        records.append(record)
    if not records:
        raise AnalysisError("trace has no typed records")
    return records


def require_fields(record: dict[str, Any], required: set[str], label: str) -> None:
    missing = sorted(required.difference(record))
    if missing:
        raise AnalysisError(
            f"{label} row {record['_line_index']} misses fields: {', '.join(missing)}"
        )


def require_positive_int(manifest: dict[str, Any], key: str, *, allow_zero: bool = False) -> int:
    value = manifest.get(key)
    minimum = 0 if allow_zero else 1
    if not isinstance(value, int) or isinstance(value, bool) or value < minimum:
        raise AnalysisError(f"manifest field {key} must be an integer >= {minimum}")
    return value


def frame_key(record: dict[str, Any]) -> tuple[str, int]:
    path = record.get("engine_path")
    epoch = record.get("tail_epoch")
    if not isinstance(path, str) or not isinstance(epoch, int) or isinstance(epoch, bool):
        raise AnalysisError(
            f"row {record['_line_index']} has invalid engine_path/tail_epoch identity"
        )
    return path, epoch


def percentile(values: list[int], quantile: float) -> int:
    if not values:
        raise AnalysisError("cannot compute a percentile over an empty denominator")
    ordered = sorted(values)
    index = max(0, math.ceil(quantile * len(ordered)) - 1)
    return ordered[index]


def latency_summary(values: list[int]) -> dict[str, int]:
    return {
        "count": len(values),
        "p50_us": percentile(values, 0.50),
        "p90_us": percentile(values, 0.90),
        "p99_us": percentile(values, 0.99),
        "max_us": max(values),
    }


def integer_field(record: dict[str, Any], key: str, label: str) -> int:
    value = record.get(key)
    if not isinstance(value, int) or isinstance(value, bool) or value < 0:
        raise AnalysisError(f"{label} row {record['_line_index']} has invalid {key}")
    return value


def delayed_fallback_edits(
    records: list[dict[str, Any]], leases: list[dict[str, Any]]
) -> tuple[int, list[int]]:
    fallback_rows = [row for row in leases if row.get("outcome") == "not_ready"]
    late_edit_rows: list[int] = []
    for fallback in fallback_rows:
        start = integer_field(fallback, "_line_index", "lease")
        end = len(records) + 1
        saw_space_completion = False
        for record in records:
            line = integer_field(record, "_line_index", "trace")
            if line <= start:
                continue
            if record.get("kind") == "ibus_key":
                stage = record.get("stage")
                if stage in {"space_managed_commit", "space_managed_autocorrect"}:
                    saw_space_completion = True
                    continue
                if saw_space_completion and isinstance(stage, str) and stage.startswith("printable"):
                    end = line
                    break
        for record in records:
            line = integer_field(record, "_line_index", "trace")
            if start < line < end and record.get("kind") == "ibus_committed_tail_replace":
                late_edit_rows.append(line)
    return len(fallback_rows), late_edit_rows


def validate_harness_output(
    harness_path: Path, manifest: dict[str, Any]
) -> tuple[dict[str, Any], list[str]]:
    harness = load_json(harness_path)
    schema = harness.get("schema")
    if schema not in {"lay.runtime-smoke-receipt.v1", SCHEMA_V2, SCHEMA_V3}:
        raise AnalysisError("unsupported runtime smoke receipt schema")
    v2_issues: list[str] = []
    if schema in {SCHEMA_V2, SCHEMA_V3}:
        try:
            validate_runtime_smoke_receipt(harness)
        except ValueError as error:
            raise AnalysisError(f"invalid v2 runtime smoke receipt: {error}") from error
        if harness.get("fatal_error") is not None:
            v2_issues.append("runtime smoke ended with a fatal error")
        if harness.get("desktop_restoration_verified") is not True:
            v2_issues.append("runtime smoke desktop restoration was not verified")
        if harness.get("all_passed") is not True:
            v2_issues.append("runtime smoke did not pass all selected cases")
    rows = harness.get("cases")
    if not isinstance(rows, list):
        raise AnalysisError("runtime smoke receipt cases must be a list")
    expected_hashes = manifest.get("expected_text_sha256")
    if not isinstance(expected_hashes, dict):
        raise AnalysisError("manifest expected_text_sha256 must be an object")

    required_names = [manifest.get("warmup_case"), manifest.get("eligible_case")]
    if any(not isinstance(name, str) or not name for name in required_names):
        raise AnalysisError("manifest warmup_case and eligible_case must be strings")
    if set(expected_hashes) != set(required_names):
        raise AnalysisError("manifest expected_text_sha256 keys must match the replay cases")

    issues: list[str] = list(v2_issues)
    cases: list[dict[str, Any]] = []
    for name in required_names:
        matches = [row for row in rows if isinstance(row, dict) and row.get("name") == name]
        if len(matches) != 1:
            issues.append(f"harness case {name!r} count {len(matches)} != 1")
            continue
        row = matches[0]
        got = row.get("got")
        expected = row.get("expected")
        ok = row.get("ok")
        expected_hash = expected_hashes.get(name)
        if not isinstance(got, str) or not isinstance(expected, str) or not isinstance(ok, bool):
            raise AnalysisError(f"runtime smoke case {name!r} has invalid field types")
        if not isinstance(expected_hash, str) or len(expected_hash) != 64:
            raise AnalysisError(f"manifest expected hash for {name!r} is invalid")
        got_hash = text_sha256(got)
        fixture_hash = text_sha256(expected)
        matched = ok and got == expected and fixture_hash == expected_hash and got_hash == expected_hash
        if not matched:
            issues.append(f"harness output mismatch for {name!r}")
        cases.append(
            {
                "name": name,
                "ok": ok,
                "matched_manifest": matched,
                "got_sha256": got_hash,
                "expected_sha256": fixture_hash,
                "manifest_sha256": expected_hash,
                "got_chars": len(got),
                "expected_chars": len(expected),
            }
        )
    if harness.get("all_passed") is not True:
        issues.append("runtime smoke receipt did not pass all selected cases")
    return (
        {
            "path": str(harness_path.resolve()),
            "sha256": sha256(harness_path),
            "all_passed": harness.get("all_passed") is True,
            "cases": cases,
        },
        issues,
    )


def analyze(trace_path: Path, manifest_path: Path, harness_path: Path) -> dict[str, Any]:
    manifest = load_json(manifest_path)
    if manifest.get("schema") != "lay.ime-immediate-space-replay.v2":
        raise AnalysisError("unsupported immediate-Space replay manifest schema")
    warmup_count = require_positive_int(manifest, "warmup_space_events", allow_zero=True)
    eligible_count = require_positive_int(manifest, "eligible_space_events")
    eligible_applied_count = require_positive_int(manifest, "eligible_applied_space_events")
    if eligible_applied_count > eligible_count:
        raise AnalysisError("eligible_applied_space_events exceeds eligible_space_events")
    producer_limit = require_positive_int(
        manifest, "maximum_field_producers_per_frame", allow_zero=True
    )
    required_projections = manifest.get("required_projections")
    if (
        not isinstance(required_projections, list)
        or not required_projections
        or any(not isinstance(value, str) for value in required_projections)
    ):
        raise AnalysisError("manifest required_projections must be a non-empty string list")

    harness_receipt, harness_issues = validate_harness_output(harness_path, manifest)
    records = load_trace(trace_path)
    routes = [row for row in records if row.get("kind") == "ibus_token_field_route"]
    leases = [row for row in records if row.get("kind") == "ibus_space_correction_lease"]
    for row in routes:
        require_fields(row, ROUTE_FIELDS, "field route")
    for row in leases:
        require_fields(row, LEASE_FIELDS, "Space lease")
        if row.get("outcome") not in LEASE_OUTCOMES:
            raise AnalysisError(
                f"Space lease row {row['_line_index']} has unknown outcome {row.get('outcome')!r}"
            )

    expected_total = warmup_count + eligible_count
    issues: list[str] = list(harness_issues)
    if len(leases) != expected_total:
        issues.append(f"lease denominator {len(leases)} != expected {expected_total}")
    eligible_leases = leases[warmup_count : warmup_count + eligible_count]
    if len(eligible_leases) != eligible_count:
        issues.append(
            f"eligible lease denominator {len(eligible_leases)} != expected {eligible_count}"
        )

    eligible_keys = [frame_key(row) for row in eligible_leases]
    if len(set(eligible_keys)) != len(eligible_keys):
        issues.append("eligible lease frames are not unique by engine_path and tail_epoch")

    routes_by_frame: dict[tuple[str, int], list[dict[str, Any]]] = defaultdict(list)
    for route in routes:
        routes_by_frame[frame_key(route)].append(route)

    frame_receipts: list[dict[str, Any]] = []
    projection_failures = 0
    producer_failures = 0
    generation_failures = 0
    for key in eligible_keys:
        frame_routes = routes_by_frame.get(key, [])
        counts = Counter(str(row.get("projection")) for row in frame_routes)
        bad_projection = any(counts.get(role, 0) != 1 for role in required_projections)
        unexpected = sorted(set(counts).difference(required_projections))
        if unexpected:
            bad_projection = True
        producers = sum(
            integer_field(row, "field_producer_count", "field route") for row in frame_routes
        )
        generations = {
            integer_field(row, "field_generation", "field route") for row in frame_routes
        }
        generation_mismatch = len(generations) > 1
        projection_failures += int(bad_projection)
        producer_failures += int(producers > producer_limit)
        generation_failures += int(generation_mismatch)
        frame_receipts.append(
            {
                "engine_path": key[0],
                "tail_epoch": key[1],
                "projection_counts": dict(sorted(counts.items())),
                "field_producer_count": producers,
                "field_generations": sorted(generations),
            }
        )

    if projection_failures:
        issues.append(f"projection cardinality failed for {projection_failures} eligible frames")
    if producer_failures:
        issues.append(f"field producer budget failed for {producer_failures} eligible frames")
    if generation_failures:
        issues.append(f"field generation parity failed for {generation_failures} eligible frames")

    warmup_outcomes = Counter(str(row.get("outcome")) for row in leases[:warmup_count])
    eligible_outcomes = Counter(str(row.get("outcome")) for row in eligible_leases)
    eligible_not_ready = eligible_outcomes.get("not_ready", 0)
    eligible_applied = eligible_outcomes.get("applied", 0)
    if eligible_not_ready:
        issues.append(f"eligible not_ready outcomes: {eligible_not_ready}")
    if eligible_applied != eligible_applied_count:
        issues.append(
            f"eligible applied outcomes: {eligible_applied} != expected {eligible_applied_count}"
        )

    managed_space_rows = [
        row
        for row in records
        if row.get("kind") == "ibus_space_key_timing"
        and isinstance(row.get("route"), str)
        and row["route"].startswith("managed_")
    ]
    if len(managed_space_rows) != expected_total:
        issues.append(
            f"managed Space timing denominator {len(managed_space_rows)} != expected {expected_total}"
        )
    eligible_space_rows = managed_space_rows[warmup_count : warmup_count + eligible_count]
    space_values = [integer_field(row, "total_us", "Space timing") for row in eligible_space_rows]
    printable_rows = [row for row in records if row.get("kind") == "ibus_printable_key_timing"]
    printable_values = [
        integer_field(row, "total_us", "printable timing") for row in printable_rows
    ]
    if len(space_values) != eligible_count:
        issues.append(f"eligible Space timing count {len(space_values)} != expected {eligible_count}")
    if not printable_values:
        issues.append("printable timing denominator is empty")

    space_latency = latency_summary(space_values) if space_values else None
    printable_latency = latency_summary(printable_values) if printable_values else None
    if space_latency is not None:
        if space_latency["p99_us"] > require_positive_int(manifest, "space_p99_us_max"):
            issues.append(f"Space p99 exceeded: {space_latency['p99_us']} us")
        if space_latency["max_us"] > require_positive_int(manifest, "space_max_us_max"):
            issues.append(f"Space max exceeded: {space_latency['max_us']} us")
    if printable_latency is not None:
        if printable_latency["p99_us"] > require_positive_int(
            manifest, "printable_p99_us_max"
        ):
            issues.append(f"printable p99 exceeded: {printable_latency['p99_us']} us")
        if printable_latency["max_us"] > require_positive_int(
            manifest, "printable_max_us_max"
        ):
            issues.append(f"printable max exceeded: {printable_latency['max_us']} us")

    fallback_count, late_edit_rows = delayed_fallback_edits(records, leases)
    if late_edit_rows:
        issues.append(f"delayed edits after literal fallback at rows {late_edit_rows}")

    gates = {
        "harness_output_parity": not harness_issues,
        "eligible_not_ready_zero": eligible_not_ready == 0,
        "eligible_applied_exact": eligible_applied == eligible_applied_count,
        "projection_cardinality": projection_failures == 0 and len(eligible_keys) == eligible_count,
        "field_producer_budget": producer_failures == 0,
        "field_generation_parity": generation_failures == 0,
        "space_latency": space_latency is not None
        and space_latency["p99_us"] <= manifest["space_p99_us_max"]
        and space_latency["max_us"] <= manifest["space_max_us_max"],
        "printable_latency": printable_latency is not None
        and printable_latency["p99_us"] <= manifest["printable_p99_us_max"]
        and printable_latency["max_us"] <= manifest["printable_max_us_max"],
        "no_delayed_edit_after_literal_fallback": not late_edit_rows,
    }
    if not all(gates.values()) and not issues:
        issues.append("one or more replay gates failed")

    return {
        "schema": "lay.ime-immediate-space-replay-receipt.v2",
        "verdict": "PASS" if not issues and all(gates.values()) else "FAIL",
        "runtime_authority_changed": False,
        "trace": {
            "path": str(trace_path.resolve()),
            "sha256": sha256(trace_path),
            "bytes": trace_path.stat().st_size,
            "typed_rows": len(records),
        },
        "manifest": {
            "path": str(manifest_path.resolve()),
            "sha256": sha256(manifest_path),
            "warmup_case": manifest.get("warmup_case"),
            "eligible_case": manifest.get("eligible_case"),
        },
        "harness": harness_receipt,
        "denominators": {
            "warmup_space_events": warmup_count,
            "eligible_space_events": eligible_count,
            "observed_space_leases": len(leases),
            "observed_managed_space_timings": len(managed_space_rows),
            "observed_printable_timings_session": len(printable_rows),
        },
        "lease_outcomes": {
            "warmup": dict(sorted(warmup_outcomes.items())),
            "eligible": dict(sorted(eligible_outcomes.items())),
        },
        "projection_receipt": {
            "failed_frames": projection_failures,
            "producer_budget_failed_frames": producer_failures,
            "generation_mismatch_frames": generation_failures,
            "frames": frame_receipts,
        },
        "latency": {
            "space_eligible": space_latency,
            "printable_session": printable_latency,
        },
        "literal_fallback": {
            "not_ready_frames_observed": fallback_count,
            "late_edit_rows": late_edit_rows,
            "physical_fault_path_exercised": fallback_count > 0,
        },
        "gates": gates,
        "issues": issues,
        "scope": {
            "tested": [
                "fixed managed GUI replay trace",
                "exact GTK output parity against manifest-pinned fixtures",
                "eligible Space lease delivery",
                "per-frame projection cardinality",
                "single-flight producer count",
                "Space and printable latency",
            ],
            "not_tested": [
                "aggregate restoration quality",
                "WeChat Telegram and browser input",
                "installed runtime authority",
            ],
        },
    }


def write_json_atomic(path: Path, value: dict[str, Any]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    handle, temp_name = tempfile.mkstemp(prefix=f".{path.name}.", dir=path.parent)
    try:
        with os.fdopen(handle, "w", encoding="utf-8") as output:
            json.dump(value, output, ensure_ascii=False, indent=2, sort_keys=True)
            output.write("\n")
            output.flush()
            os.fsync(output.fileno())
        os.replace(temp_name, path)
    except BaseException:
        Path(temp_name).unlink(missing_ok=True)
        raise


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("trace", type=Path)
    parser.add_argument(
        "--manifest",
        type=Path,
        default=Path("data/test_input/ime_immediate_space_replay_manifest.json"),
    )
    parser.add_argument("--harness", type=Path, required=True)
    parser.add_argument("--out", type=Path)
    args = parser.parse_args()
    try:
        receipt = analyze(args.trace, args.manifest, args.harness)
    except AnalysisError as error:
        receipt = {
            "schema": "lay.ime-immediate-space-replay-receipt.v2",
            "verdict": "ERROR",
            "runtime_authority_changed": False,
            "issues": [str(error)],
        }
    if args.out is not None:
        write_json_atomic(args.out, receipt)
    print(json.dumps(receipt, ensure_ascii=False, indent=2, sort_keys=True))
    return 0 if receipt["verdict"] == "PASS" else 1


if __name__ == "__main__":
    raise SystemExit(main())
