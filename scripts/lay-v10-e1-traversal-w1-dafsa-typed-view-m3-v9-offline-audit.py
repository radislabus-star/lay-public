#!/usr/bin/env python3
"""Independent offline correction audit for the sealed M3 V9 trace."""

from __future__ import annotations

import argparse
import ast
import hashlib
import json
import os
import pathlib
import re
import shutil
import stat
import tempfile
import time
from collections.abc import Mapping, Sequence
from typing import Any


ROOT = pathlib.Path(__file__).resolve().parents[1]
TASK_ID = "slice8b-v10-e1-traversal-w1-dafsa-typed-view-m3-final-materialization-v9-offline-terminal-audit-v2-20260827"
TRANSACTION_ID = "8fe4616a32137a9f8b42e6a11f677fd3657d43c7c1502093272ca2a0d0d3ee5a"
V9_TASK_ID = "slice8b-v10-e1-traversal-w1-dafsa-typed-view-m3-final-materialization-v9-20260827"
V9_TRANSACTION_ID = "ed21c54906eebc5a9a99afc873b3a38b8a6ca5e6003b539d019539403aa2ffb1"
V8R3_TASK_ID = "slice8b-v10-e1-traversal-w1-dafsa-typed-view-m3-e2e-v8r3-terminal-projection-20260827"
SCIENTIFIC_TEST = (
    "nanda_wave::l2_field::v13_typed_peak::tests::m3_v8::"
    "m3_end_to_end_physical_proof"
)
V8R1_INPUTS = pathlib.PurePosixPath(
    "/home/e/.local/share/lay/provenance/"
    "slice8b-v10-e1-traversal-w1-dafsa-typed-view-m3-e2e-v8r1-20260827/"
    "bootstrap-v1/inputs"
)
V8R3_ELF = pathlib.PurePosixPath(
    "/home/e/.local/share/lay/provenance/"
    f"{V8R3_TASK_ID}/bootstrap-v1/m3-v8r3-test-elf"
)
V9_REMOTE_PARENT = pathlib.PurePosixPath("/home/e/.local/share/lay/provenance") / V9_TASK_ID
V9_REMOTE_STATE = pathlib.PurePosixPath("/home/e/.local/state/lay") / V9_TASK_ID

CORRECTION = ROOT / "docs/structural_gates/LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_W1_DAFSA_TYPED_VIEW_M3_FINAL_MATERIALIZATION_V9_TERMINAL_ENVIRONMENT_CORRECTION_V2_2026-08-27.md"
ROUTE = ROOT / "docs/structural_gates/LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_W1_DAFSA_TYPED_VIEW_M3_FINAL_MATERIALIZATION_V9_TERMINAL_ENVIRONMENT_CORRECTION_V2_ROUTE_2026-08-27.md"
ROUTE_RECEIPT = ROOT / "docs/structural_gates/receipts/LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_W1_DAFSA_TYPED_VIEW_M3_FINAL_MATERIALIZATION_V9_TERMINAL_ENVIRONMENT_CORRECTION_V2_ROUTE_RECEIPT_2026-08-27.json"
PREFLIGHT = ROOT / "docs/structural_gates/preflights/LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_W1_DAFSA_TYPED_VIEW_M3_FINAL_MATERIALIZATION_V9_OFFLINE_TERMINAL_AUDIT_V2_IMPLEMENTATION_2026-08-27.json"
PREFLIGHT_RECEIPT = ROOT / "docs/structural_gates/receipts/LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_W1_DAFSA_TYPED_VIEW_M3_FINAL_MATERIALIZATION_V9_OFFLINE_TERMINAL_AUDIT_V2_IMPLEMENTATION_PREFLIGHT_2026-08-27.json"
V1_ROOT = ROOT / "docs/structural_gates/receipts/LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_W1_DAFSA_TYPED_VIEW_M3_FINAL_MATERIALIZATION_DIAGNOSTIC_V9_TERMINAL_AUDIT_V1_2026-08-27"
JOURNAL_ROOT = ROOT / "docs/structural_gates/receipts/LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_W1_DAFSA_TYPED_VIEW_M3_FINAL_MATERIALIZATION_DIAGNOSTIC_V9_EXECUTION_JOURNAL_V1_2026-08-27"
TRACE_ROOT = V1_ROOT / "REMOTE_TRACE"
V1_TERMINAL = V1_ROOT / "TERMINAL_AUDIT.json"
WRAPPER = TRACE_ROOT / "TRACE_WRAPPER.json"
STDERR = TRACE_ROOT / "stderr.log"
TRACE_ROWS = TRACE_ROOT / "TRACE_ROWS.json"
TRACE_SUMMARY = TRACE_ROOT / "TRACE_SUMMARY.json"
SUBJECT = TRACE_ROOT / "subject/SUBJECT_RECEIPT.json"
DESTINATION = ROOT / "docs/structural_gates/receipts/LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_W1_DAFSA_TYPED_VIEW_M3_FINAL_MATERIALIZATION_V9_OFFLINE_TERMINAL_AUDIT_V2_2026-08-27"
AUDITOR = pathlib.Path(__file__).resolve()

ACTIONS = ("self-check", "audit")
EXPECTED_TRACE_ROWS = 1_910
WARMUP_ROWS = 382
MEASURED_ROWS = 1_528
TAIL_ROWS = 16
TRACE_PREFIX = "productive_v90_materialization_trace "
TRACE_PATTERN = re.compile(
    r"^productive_v90_materialization_trace surfaces=(\d+) emitted=(\d+) setup_us=(\d+) "
    r"projection_us=(\d+) classify_us=(\d+) gate_us=(\d+) evidence_us=(\d+)$"
)
STAGES = ("setup_us", "projection_us", "classify_us", "gate_us", "evidence_us")
SEMANTIC_FIELDS = (
    "candidate_mismatches",
    "certificate_mismatches",
    "structured_certificate_mismatches",
    "schedule_mismatches",
    "completeness_mismatches",
    "lattice_marker_mismatches",
    "emitted_surface_mismatches",
    "gate_mismatches",
    "certificate_collisions",
    "semantic_total",
)
EXPECTED = {
    CORRECTION: (5_508, "0af397c9969077cb6348cbba84f27837f19554d2cb06fda752fb7682cb249204"),
    ROUTE: (3_950, "f1cbd35c38eaecd6fc4940c116c0f8ced94484de0907a1a7ba0976cdbd08bbfc"),
    ROUTE_RECEIPT: (22_377, "a8d6e20966421dd4caa256d5491fd4d5f986484ba4f06b741cfd0cd6af01a299"),
    PREFLIGHT: (11_022, "c93392bb6312f503742c36e5036e480d7514360e17447863f3e15990c16b907e"),
    PREFLIGHT_RECEIPT: (7_213, "496b779d5df4b9fd398472304632749c752e64e85d6e38b26539e1ad8f3a7860"),
    V1_TERMINAL: (37_744, "f68b213d6404ae1e82593be8e4663de528e32accc5f7e7b5fead1cf63292616e"),
    V1_ROOT / "SHA256SUMS": (983, "c03ec661768262e3a3d366c67d606b3bc6ef5cf1575f4a02dad7107c00ccf62e"),
    JOURNAL_ROOT / "SHA256SUMS": (2_321, "92c6f8cf8e293773dc7f72cbc2f8dfe1ff83d8493564b8079ef6d9307329e879"),
    WRAPPER: (34_721, "574fc73e334132aa0ebd3eeb4b4d044ef73258bda7f5347e9f6c8cd50ffc4c89"),
    STDERR: (627_258, "2564fa7403a182ce15bb2213f9f8e183153d19076b928d3f3011187edffdfaf9"),
    TRACE_ROWS: (436_739, "4d97d55d8b3f32aeca843cf1d44f4018dcfabd1686f6be9e3fd64a60ababcd2b"),
    TRACE_SUMMARY: (3_398, "dc037e8899f85b5a7c9f01aeabf57f8b79c5cd18e8f306b5ddb552cd2a5ca027"),
    SUBJECT: (10_377, "01ea35ea7ead276039dc67fa097a9be6cfe20d2d85a3dc24e33c1ee294e56eb1"),
}
EXPECTED_SOURCE = {
    "v13_sha256": "cce259fe0ce5dce67702383363b66f0fe9b9ff5a87d8f01c4fcf342d91218d7b",
    "v7_sha256": "33fded73e13f565bdc08a83bd440be67c80f6d85a196453050b09bc9f7ef28e4",
    "productive_v90_sha256": "40fb6a9f0d92c3c7502e47f9c70230d9b86020f622b08a5c799342f13e09ce44",
    "l11_sha256": "bf5a1619a89038466ef786305cf35eda5f4af5b9f12b9140f7d3cac407e2f2a7",
    "test_elf_sha256": "0af3cc6679396650245e924976d8a3bb432dfa6a1086b1c46bcfa7497307afea",
}


class OfflineAuditError(RuntimeError):
    pass


def need(condition: bool, message: str) -> None:
    if not condition:
        raise OfflineAuditError(message)


def canonical(value: Any) -> bytes:
    return (json.dumps(value, ensure_ascii=False, sort_keys=True, separators=(",", ":")) + "\n").encode()


def sha256_file(path: pathlib.Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as source:
        for block in iter(lambda: source.read(1024 * 1024), b""):
            digest.update(block)
    return digest.hexdigest()


def mode_string(path: pathlib.Path) -> str:
    return f"{stat.S_IMODE(path.stat().st_mode):04o}"


def load_json(path: pathlib.Path) -> Any:
    with path.open("rb") as source:
        return json.load(source)


def file_row(path: pathlib.Path) -> dict[str, Any]:
    need(path.is_file() and not path.is_symlink(), f"required regular file absent: {path}")
    return {
        "path": str(path),
        "mode": mode_string(path),
        "size_bytes": path.stat().st_size,
        "sha256": sha256_file(path),
    }


def write_new(path: pathlib.Path, value: bytes, mode: int = 0o600) -> None:
    descriptor = os.open(path, os.O_WRONLY | os.O_CREAT | os.O_EXCL, mode)
    try:
        with os.fdopen(descriptor, "wb", closefd=False) as output:
            output.write(value)
            output.flush()
            os.fsync(output.fileno())
    finally:
        os.close(descriptor)
    os.chmod(path, mode)


def fsync_dir(path: pathlib.Path) -> None:
    descriptor = os.open(path, os.O_RDONLY | os.O_DIRECTORY)
    try:
        os.fsync(descriptor)
    finally:
        os.close(descriptor)


def write_manifest(root: pathlib.Path) -> None:
    rows = []
    for path in sorted(root.rglob("*")):
        if path.is_file() and path.name != "SHA256SUMS":
            rows.append(f"{sha256_file(path)}  {path.relative_to(root).as_posix()}\n")
    write_new(root / "SHA256SUMS", "".join(rows).encode())


def seal_tree(root: pathlib.Path) -> None:
    for path in sorted(root.rglob("*"), reverse=True):
        path.chmod(0o555 if path.is_dir() else 0o444)
    root.chmod(0o555)


def verify_manifest(root: pathlib.Path) -> int:
    manifest = root / "SHA256SUMS"
    need(manifest.is_file(), f"manifest absent: {root}")
    expected = set()
    rows = manifest.read_text().splitlines()
    for row in rows:
        digest, relative = row.split("  ", 1)
        path = root / relative
        need(path.is_file() and sha256_file(path) == digest, f"manifest mismatch: {root}: {relative}")
        expected.add(relative)
    actual = {
        path.relative_to(root).as_posix()
        for path in root.rglob("*")
        if path.is_file() and path.name != "SHA256SUMS"
    }
    need(actual == expected, f"manifest inventory drift: {root}")
    return len(rows)


def fixed_inputs() -> dict[str, Any]:
    rows = {}
    for path, (size, digest) in EXPECTED.items():
        row = file_row(path)
        need(row["mode"] == "0444", f"sealed mode drift: {path}")
        need(row["size_bytes"] == size and row["sha256"] == digest, f"sealed identity drift: {path}")
        rows[str(path.relative_to(ROOT))] = row
    route = load_json(ROUTE_RECEIPT)
    preflight = load_json(PREFLIGHT_RECEIPT)
    need(route.get("verdict") == "PASS", "correction structural route did not pass")
    need(
        preflight.get("verdict") == "READY_TO_IMPLEMENT"
        and preflight.get("safe_to_implement") is True
        and preflight.get("manifest_sha256") == "efd12a22018e502bbe2d3503c3d4eaa164fe10265c03534f2bb424b57789202f",
        "correction implementation preflight drift",
    )
    return rows


def expected_environment() -> dict[str, str]:
    subject = V9_REMOTE_PARENT / "trace-v1/subject"
    return {
        "HOME": "/home/e",
        "LANG": "C.UTF-8",
        "LAY_L11_RECEIPT": str(V8R1_INPUTS / "l11-installed.json"),
        "LAY_L2_FIELD_TRACE": "1",
        "LAY_L2_PACKAGE": str(V8R1_INPUTS / "LAY-L2-RU-FULL-v13.bin"),
        "LAY_L2_PRODUCTIVE_V1_PACKAGE": str(V8R1_INPUTS / "LAY-L2-PRODUCTIVE-PARADIGM-v90.p2m"),
        "LAY_M3_ACTUAL_OWNER_PACKAGE": str(V8R1_INPUTS / "LAY-L2-RU-FULL-v13.bin"),
        "LAY_M3_ACTUAL_OWNER_V7": str(V8R1_INPUTS / "slice8b-v7-fixed-13x100.json"),
        "LAY_M3_V8_EVIDENCE_DIR": str(subject / "evidence"),
        "LAY_M3_V8_RECEIPT": str(subject / "SUBJECT_RECEIPT.json"),
        "LC_ALL": "C.UTF-8",
        "PATH": "/home/e/.cargo/bin:/home/e/.local/bin:/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin",
        "RUST_BACKTRACE": "0",
    }


def expected_command() -> list[str]:
    environment = expected_environment()
    return [
        "/usr/bin/sudo", "-n", "-u", "e", "/usr/bin/env",
        *[f"{key}={value}" for key, value in sorted(environment.items())],
        "/usr/bin/taskset", "-c", "0", str(V8R3_ELF),
        "--ignored", "--exact", SCIENTIFIC_TEST, "--nocapture", "--test-threads=1",
    ]


def parse_environment(command: Sequence[str]) -> dict[str, str]:
    need(command.count("/usr/bin/env") == 1, "environment sentinel cardinality drift")
    need(command.count("/usr/bin/taskset") == 1, "taskset sentinel cardinality drift")
    start = command.index("/usr/bin/env") + 1
    end = command.index("/usr/bin/taskset")
    need(start < end, "environment sentinel order drift")
    tokens = list(command[start:end])
    need(tokens, "environment segment is empty")
    result = {}
    for token in tokens:
        match = re.fullmatch(r"([A-Z_][A-Z0-9_]*)=(.*)", token)
        need(match is not None, f"non-assignment in environment segment: {token!r}")
        key, value = match.groups()
        need(key not in result, f"duplicate environment key: {key}")
        result[key] = value
    return result


def nearest_rank(values: Sequence[int], percentile: int) -> int:
    need(values, "empty distribution")
    ordered = sorted(int(value) for value in values)
    rank = max(1, (percentile * len(ordered) + 99) // 100)
    return ordered[rank - 1]


def distribution(rows: Sequence[Mapping[str, Any]], field: str) -> dict[str, int]:
    values = [int(row[field]) for row in rows]
    return {
        "count": len(values),
        "p50_us": nearest_rank(values, 50),
        "p99_us": nearest_rank(values, 99),
        "max_us": max(values),
        "sum_us": sum(values),
    }


def summarize_trace(rows: Sequence[Mapping[str, Any]]) -> dict[str, Any]:
    need(len(rows) == EXPECTED_TRACE_ROWS, "trace cardinality drift")
    warmup = list(rows[:WARMUP_ROWS])
    measured = list(rows[WARMUP_ROWS:])
    need(len(warmup) == WARMUP_ROWS and len(measured) == MEASURED_ROWS, "trace denominator drift")
    fields = (*STAGES, "traced_total_us")
    schedules = ("FORWARD", "REVERSED", "FORWARD", "REVERSED")
    rounds = []
    for index, schedule in enumerate(schedules):
        subset = measured[index * WARMUP_ROWS : (index + 1) * WARMUP_ROWS]
        rounds.append({
            "round": index + 1,
            "schedule": schedule,
            "stages": {field: distribution(subset, field) for field in fields},
        })
    tail = sorted(measured, key=lambda row: (-int(row["traced_total_us"]), int(row["ordinal"])))[:TAIL_ROWS]
    tail_total = sum(int(row["traced_total_us"]) for row in tail)
    need(tail_total > 0, "trace tail total is zero")
    stage_rows = {}
    candidates = []
    for stage in STAGES:
        stage_sum = sum(int(row[stage]) for row in tail)
        largest = 0
        for row in tail:
            maximum = max(int(row[name]) for name in STAGES)
            winners = [name for name in STAGES if int(row[name]) == maximum]
            largest += int(winners == [stage])
        share = stage_sum / tail_total
        stage_rows[stage] = {"sum_us": stage_sum, "share": share, "largest_stage_rows": largest}
        if share >= 0.80 and largest >= 15:
            candidates.append(stage)
    return {
        "schema": "lay.m3-v9-materialization-trace.v1",
        "trace_rows": len(rows),
        "warmup_rows": len(warmup),
        "measured_rows": len(measured),
        "pooled": {field: distribution(measured, field) for field in fields},
        "rounds": rounds,
        "tail": {
            "rows": len(tail),
            "ordinals": [int(row["ordinal"]) for row in tail],
            "traced_total_us": tail_total,
            "stages": stage_rows,
            "dominant_stage": candidates[0] if len(candidates) == 1 else None,
            "dominance_candidates": candidates,
            "thresholds": {"aggregate_share": 0.80, "largest_stage_rows": 15},
        },
        "claim_boundary": {
            "outer_per_request_join": False,
            "v8r3_latency_reinterpreted": False,
            "production_authority_admitted": False,
        },
    }


def parse_trace(raw: bytes) -> tuple[list[dict[str, Any]], list[str], dict[str, Any] | None]:
    rows = []
    errors = []
    schedules = ("FORWARD", "REVERSED", "FORWARD", "REVERSED")
    for line_number, line in enumerate(raw.decode("utf-8", errors="replace").splitlines(), start=1):
        if not line.startswith(TRACE_PREFIX):
            continue
        match = TRACE_PATTERN.fullmatch(line)
        if match is None:
            errors.append(f"line {line_number}: malformed trace row")
            continue
        values = [int(value) for value in match.groups()]
        ordinal = len(rows)
        if ordinal < WARMUP_ROWS:
            phase, round_index, schedule, case_ordinal = "WARMUP", 0, "FORWARD", ordinal
        else:
            measured = ordinal - WARMUP_ROWS
            round_index = measured // WARMUP_ROWS + 1
            case_ordinal = measured % WARMUP_ROWS
            phase = "MEASURED"
            schedule = schedules[min(round_index - 1, len(schedules) - 1)]
        row = {
            "ordinal": ordinal,
            "stderr_line": line_number,
            "phase": phase,
            "round": round_index,
            "schedule": schedule,
            "case_ordinal": case_ordinal,
            "surfaces": values[0],
            "emitted": values[1],
            "setup_us": values[2],
            "projection_us": values[3],
            "classify_us": values[4],
            "gate_us": values[5],
            "evidence_us": values[6],
        }
        row["traced_total_us"] = sum(int(row[name]) for name in STAGES)
        rows.append(row)
    if len(rows) != EXPECTED_TRACE_ROWS:
        errors.append(f"trace row count {len(rows)} != {EXPECTED_TRACE_ROWS}")
    summary = None
    if not errors:
        try:
            summary = summarize_trace(rows)
        except BaseException as error:
            errors.append(f"{type(error).__name__}: {error}")
    return rows, errors, summary


def integer(value: Any, default: int = -1) -> int:
    return value if isinstance(value, int) and not isinstance(value, bool) else default


def evaluate() -> tuple[str, dict[str, list[str]], dict[str, Any]]:
    old = load_json(V1_TERMINAL)
    wrapper = load_json(WRAPPER)
    subject = load_json(SUBJECT)
    retained_rows = load_json(TRACE_ROWS)
    retained_summary = load_json(TRACE_SUMMARY)
    rows, parse_errors, summary = parse_trace(STDERR.read_bytes())
    failures = {key: [] for key in ("provenance", "semantic", "capability")}

    if (
        old.get("schema") != "lay.m3-v9-terminal-audit.v1"
        or old.get("task_id") != V9_TASK_ID
        or old.get("transaction_id") != V9_TRANSACTION_ID
        or old.get("verdict") != "BLOCKED_PROVENANCE"
        or old.get("failures") != {"capability": [], "provenance": ["scientific environment drift"], "semantic": []}
    ):
        failures["provenance"].append("historical V1 terminal projection drift")
    command = wrapper.get("command") if isinstance(wrapper.get("command"), list) else []
    try:
        parsed_environment = parse_environment(command)
    except OfflineAuditError as error:
        parsed_environment = {}
        failures["provenance"].append(str(error))
    if command != expected_command():
        failures["provenance"].append("scientific command drift")
    if parsed_environment != expected_environment() or wrapper.get("environment") != expected_environment():
        failures["provenance"].append("corrected scientific environment drift")
    legacy_environment = {
        token.split("=", 1)[0]: token.split("=", 1)[1]
        for token in command
        if "=" in token and not token.startswith("/")
    }
    legacy_extra = {key: value for key, value in legacy_environment.items() if key not in expected_environment()}
    if legacy_extra != {"--test-threads": "1"}:
        failures["provenance"].append("documented V1 parser defect is not exact")
    direct = wrapper.get("direct_exec_identity", {})
    processes = direct.get("processes") if isinstance(direct.get("processes"), list) else []
    if direct.get("observed") is not True or direct.get("target") != str(V8R3_ELF) or not processes:
        failures["capability"].append("direct subject observation unavailable")
    for process in processes:
        if process.get("executable") != str(V8R3_ELF) or process.get("argv", [None])[0] != str(V8R3_ELF) or SCIENTIFIC_TEST not in process.get("argv", []):
            failures["provenance"].append("direct subject identity drift")
    if wrapper.get("schema") != "lay.m3-v9-trace-wrapper.v1" or wrapper.get("task_id") != V9_TASK_ID or wrapper.get("transaction_id") != V9_TRANSACTION_ID:
        failures["provenance"].append("wrapper namespace drift")
    if wrapper.get("verdict") != "M3_V9_TRACE_CREATED_UNAUDITED" or wrapper.get("controller_error") is not None or wrapper.get("outputs_complete") is not True or wrapper.get("exit_pair_consistent") is not True:
        failures["capability"].append("producer observation incomplete")
    if wrapper.get("subject_receipt") != subject:
        failures["provenance"].append("wrapper and retained subject receipt differ")
    if parse_errors:
        failures["provenance"].extend(parse_errors)
    if summary is None:
        failures["capability"].append("trace summary unavailable")
    if (
        retained_rows.get("schema") != "lay.m3-v9-materialization-trace-rows.v1"
        or retained_rows.get("task_id") != V9_TASK_ID
        or retained_rows.get("transaction_id") != V9_TRANSACTION_ID
        or retained_rows.get("parse_errors") != parse_errors
        or retained_rows.get("rows") != rows
        or retained_summary != summary
        or wrapper.get("trace_summary") != summary
        or wrapper.get("trace_rows") != len(rows)
        or wrapper.get("trace_parse_errors") != parse_errors
    ):
        failures["provenance"].append("retained trace derivatives differ from independent recomputation")

    source = subject.get("source", {})
    for key, expected in EXPECTED_SOURCE.items():
        if source.get(key) != expected:
            failures["provenance"].append(f"subject source identity drift: {key}")
    fixed = subject.get("fixed_proof", {})
    semantic = fixed.get("semantic", {})
    for field in SEMANTIC_FIELDS:
        if integer(semantic.get(field)) != 0:
            failures["semantic"].append(f"{field} is nonzero or unknown")
    if integer(semantic.get("capacity_failures")) != 0 or integer(semantic.get("unresolved")) != 0 or integer(fixed.get("empty_lane_mismatches")) != 0:
        failures["semantic"].append("capacity unresolved or empty-lane mismatch")
    if integer(fixed.get("maximum_query_scratch_bytes"), 2**63) > 512 * 1024:
        failures["semantic"].append("query scratch gate failed")
    if (
        integer(fixed.get("cases")) != 382
        or integer(fixed.get("measured_rounds")) != 4
        or integer(fixed.get("measured_samples")) != MEASURED_ROWS
        or fixed.get("schedule") != ["FORWARD", "REVERSED", "FORWARD", "REVERSED"]
    ):
        failures["provenance"].append("subject denominator or schedule drift")
    if integer(fixed.get("cpu")) != 0 or integer(fixed.get("cpu_mismatches")) != 0 or integer(fixed.get("warmup_cpu_mismatches")) != 0:
        failures["semantic"].append("CPU closure failed")
    gates = subject.get("gates", {})
    for field in ("semantic", "capacity", "reload_identity", "rss", "environment"):
        if gates.get(field) is not True:
            failures["semantic"].append(f"non-latency gate failed: {field}")
    accepted_pair = (
        subject.get("verdict") == "M3_END_TO_END_TEST_OWNER_PASS"
        and wrapper.get("exit_code") == 0
        and gates.get("latency") is True
    ) or (
        subject.get("verdict") == "BLOCKED_LATENCY"
        and wrapper.get("exit_code") == 101
        and gates.get("latency") is False
    )
    if not accepted_pair and not failures["semantic"]:
        failures["capability"].append("subject verdict/exit pair unsupported")
    reload = subject.get("reload", {})
    for key, expected in {
        "reader_identity_mismatches": 0,
        "mixed_generation_observations": 0,
        "stale_a_commits": 0,
        "stale_a_cancellations": 1,
        "current_b_commits": 1,
        "failed_build_publications": 0,
        "rollback_identity_mismatches": 0,
        "typed_materializations": 2,
        "per_request_typed_materializations": 0,
    }.items():
        if integer(reload.get(key)) != expected:
            failures["semantic"].append(f"reload field drift: {key}")
    if reload.get("held_a_survived_publication") is not True:
        failures["semantic"].append("held generation A did not survive")
    pss = subject.get("pss", {})
    if integer(pss.get("aggregate_delta_pss_kib"), 2**63) > 40 * 1024 or integer(pss.get("typed_owned_bytes_per_process")) != 3_689_628 or integer(pss.get("sidecar_bytes"), 2**63) > 32 * 1024 * 1024 or integer(pss.get("helper_failures")) != 0:
        failures["semantic"].append("RSS or sidecar gate failed")
    claim = subject.get("claim_boundary", {})
    if claim.get("test_only_generation_owner") is not True or claim.get("production_authority_admitted") is not False or claim.get("runtime_reload_edit_admitted") is not False:
        failures["provenance"].append("subject claim boundary drift")
    if subject.get("runtime_authority_changed") is not False or subject.get("production_activation_admitted") is not False or subject.get("perf_or_pmu_used") is not False or subject.get("network_used_by_subject") is not False or subject.get("installed_package_changed") is not False:
        failures["provenance"].append("subject side-effect boundary drift")
    if wrapper.get("thermal_throttle_drift") not in ({}, None):
        failures["provenance"].append("thermal throttle drift")
    if wrapper.get("subject_executions") != 1 or any(wrapper.get(key) != 0 for key in ("cargo_invocations", "rustc_compilations", "perf_record_invocations", "perf_stat_invocations")) or wrapper.get("runtime_authority_changed") is not False:
        failures["provenance"].append("wrapper execution ledger drift")
    marker = wrapper.get("marker", {})
    before = marker.get("before", {}) if isinstance(marker, dict) else {}
    after = marker.get("after", {}) if isinstance(marker, dict) else {}
    if (
        marker.get("consumed_before_execution") is not True
        or before.get("path") != str(V9_REMOTE_STATE / "markers/trace.available")
        or after.get("path") != str(V9_REMOTE_STATE / "markers/trace.consumed-before-exec")
        or before.get("mode") != "0400"
        or after.get("mode") != "0400"
        or before.get("sha256") != after.get("sha256")
        or before.get("size_bytes") != after.get("size_bytes")
    ):
        failures["provenance"].append("one-shot marker evidence drift")

    priority = (("BLOCKED_PROVENANCE", "provenance"), ("BLOCKED_SEMANTIC", "semantic"), ("BLOCKED_CAPABILITY", "capability"))
    verdict = "FINAL_MATERIALIZATION_DECOMPOSED"
    for candidate, key in priority:
        if failures[key]:
            verdict = candidate
            break
    observation = {
        "corrected_environment": parsed_environment,
        "legacy_extra_environment": legacy_extra,
        "trace": summary,
        "subject_verdict": subject.get("verdict"),
        "subject_exit_code": wrapper.get("exit_code"),
        "semantic_gates": gates,
    }
    return verdict, failures, observation


def self_check() -> dict[str, Any]:
    fixed_inputs()
    need(ACTIONS == ("self-check", "audit"), "action registry drift")
    need(not DESTINATION.exists(), "offline terminal receipt already exists")
    parsed = ast.parse(AUDITOR.read_text(), filename=str(AUDITOR))
    imports = {
        alias.name.split(".", 1)[0]
        for node in ast.walk(parsed)
        if isinstance(node, (ast.Import, ast.ImportFrom))
        for alias in (node.names if isinstance(node, ast.Import) else [ast.alias(name=node.module or "")])
    }
    need(not ({"subprocess", "socket", "paramiko", "requests"} & imports), "external execution or network import reachable")
    command = expected_command()
    environment = parse_environment(command)
    need(environment == expected_environment(), "bounded environment parser self-check failed")
    legacy = {token.split("=", 1)[0]: token.split("=", 1)[1] for token in command if "=" in token and not token.startswith("/")}
    need({key: value for key, value in legacy.items() if key not in environment} == {"--test-threads": "1"}, "legacy defect fixture drift")
    rows, errors, summary = parse_trace(STDERR.read_bytes())
    need(not errors and summary == load_json(TRACE_SUMMARY), "independent trace parser self-check failed")
    need(load_json(TRACE_ROWS).get("rows") == rows, "independent trace rows self-check failed")
    need(verify_manifest(V1_ROOT) == 10 and verify_manifest(JOURNAL_ROOT) == 24 and verify_manifest(TRACE_ROOT) == 9, "sealed manifest closure drift")
    verdict, failures, observation = evaluate()
    need(verdict == "FINAL_MATERIALIZATION_DECOMPOSED" and not any(failures.values()), "corrected dispatch self-check failed")
    with tempfile.TemporaryDirectory(prefix="lay-m3-v9-offline-fault-") as raw:
        root = pathlib.Path(raw)
        partial = root / "partial.py"
        partial.write_bytes(AUDITOR.read_bytes()[:31])
        try:
            ast.parse(partial.read_text())
        except SyntaxError:
            pass
        else:
            raise OfflineAuditError("partial auditor source was accepted")
        collision = root / "receipt.json"
        write_new(collision, b"first\n")
        try:
            write_new(collision, b"second\n")
        except FileExistsError:
            pass
        else:
            raise OfflineAuditError("exclusive publication fault failed")
    return {
        "schema": "lay.m3-v9-offline-auditor-self-check.v1",
        "task_id": TASK_ID,
        "transaction_id": TRANSACTION_ID,
        "verdict": "M3_V9_OFFLINE_AUDITOR_STATIC_PASS",
        "auditor_sha256": sha256_file(AUDITOR),
        "actions": list(ACTIONS),
        "tests": {
            "auditor-write-fault": "PASS",
            "static-fault": "PASS",
            "receipt-publication-fault": "PASS",
            "paper-route-parity": "PASS",
            "environment-parser-parity": "PASS",
            "trace-recomputation-parity": "PASS",
            "v1-preservation-parity": "PASS",
            "closed-command-graph": "PASS",
            "claim-boundary-parity": "PASS",
        },
        "corrected_dispatch": verdict,
        "legacy_extra_environment": observation["legacy_extra_environment"],
        "trace_rows": len(rows),
        "measured_rows": summary["measured_rows"],
        "dominant_stage": summary["tail"]["dominant_stage"],
        "external_processes": 0,
        "network_actions": 0,
        "subject_executions": 0,
        "markers_created": 0,
        "markers_consumed": 0,
        "runtime_authority_changed": False,
    }


def audit() -> dict[str, Any]:
    need(mode_string(AUDITOR) == "0555", "offline auditor is not sealed executable source")
    check = self_check()
    terminal_before = file_row(V1_TERMINAL)
    terminal_manifest_before = file_row(V1_ROOT / "SHA256SUMS")
    journal_manifest_before = file_row(JOURNAL_ROOT / "SHA256SUMS")
    verdict, failures, observation = evaluate()
    receipt = {
        "schema": "lay.m3-v9-offline-terminal-audit.v2",
        "task_id": TASK_ID,
        "transaction_id": TRANSACTION_ID,
        "verdict": verdict,
        "positive_verdict": "FINAL_MATERIALIZATION_DECOMPOSED",
        "failure_priority": ["provenance", "semantic", "capability", "complete_decomposition"],
        "failures": failures,
        "auditor_sha256": sha256_file(AUDITOR),
        "self_check": check,
        "correction_sha256": sha256_file(CORRECTION),
        "structural_route_sha256": sha256_file(ROUTE),
        "structural_receipt_sha256": sha256_file(ROUTE_RECEIPT),
        "implementation_preflight_sha256": sha256_file(PREFLIGHT_RECEIPT),
        "historical_v1": {
            "verdict": "BLOCKED_PROVENANCE",
            "terminal_receipt": terminal_before,
            "terminal_manifest": terminal_manifest_before,
            "journal_manifest": journal_manifest_before,
            "terminal_manifest_entries": verify_manifest(V1_ROOT),
            "journal_manifest_entries": verify_manifest(JOURNAL_ROOT),
            "trace_manifest_entries": verify_manifest(TRACE_ROOT),
            "modified": False,
        },
        "environment_correction": {
            "parser_scope": "tokens strictly between unique /usr/bin/env and /usr/bin/taskset sentinels",
            "legacy_extra": observation["legacy_extra_environment"],
            "corrected_environment": observation["corrected_environment"],
            "wrapper_environment_exact": True,
        },
        "scientific_receipt_verdict": observation["subject_verdict"],
        "scientific_exit_code": observation["subject_exit_code"],
        "semantic_gates": observation["semantic_gates"],
        "trace": observation["trace"],
        "dominant_stage": observation["trace"]["tail"]["dominant_stage"] if observation["trace"] else None,
        "trace_rows": observation["trace"]["trace_rows"] if observation["trace"] else 0,
        "measured_rows": observation["trace"]["measured_rows"] if observation["trace"] else 0,
        "subject_executions": 0,
        "network_actions": 0,
        "remote_reads": 0,
        "remote_writes": 0,
        "markers_created": 0,
        "markers_consumed": 0,
        "cargo_invocations": 0,
        "rustc_compilations": 0,
        "perf_record_invocations": 0,
        "perf_stat_invocations": 0,
        "v8r3_latency_reinterpreted": False,
        "outer_per_request_join_claimed": False,
        "runtime_authority_changed": False,
        "production_authority_admitted": False,
        "next_if_pass": "separate paper decision about the measured dominant stage or its absence only",
    }
    need(not DESTINATION.exists(), "offline terminal receipt already exists")
    stage = DESTINATION.with_name(f"{DESTINATION.name}.stage-{os.getpid()}-{time.time_ns()}")
    stage.mkdir(parents=True, mode=0o700)
    try:
        write_new(stage / "SELF_CHECK.json", canonical(check))
        write_new(stage / "TERMINAL_AUDIT.json", canonical(receipt))
        write_manifest(stage)
        seal_tree(stage)
        os.rename(stage, DESTINATION)
        fsync_dir(DESTINATION.parent)
    except BaseException:
        if stage.exists():
            shutil.rmtree(stage)
        raise
    need(file_row(V1_TERMINAL) == terminal_before, "V1 terminal changed during offline audit")
    need(file_row(V1_ROOT / "SHA256SUMS") == terminal_manifest_before, "V1 terminal manifest changed during offline audit")
    need(file_row(JOURNAL_ROOT / "SHA256SUMS") == journal_manifest_before, "V1 journal manifest changed during offline audit")
    return load_json(DESTINATION / "TERMINAL_AUDIT.json")


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("action", choices=ACTIONS)
    args = parser.parse_args()
    try:
        value = self_check() if args.action == "self-check" else audit()
        print(json.dumps(value, sort_keys=True))
        return 0
    except BaseException as error:
        print(json.dumps({
            "schema": "lay.m3-v9-offline-auditor-error.v1",
            "task_id": TASK_ID,
            "transaction_id": TRANSACTION_ID,
            "verdict": "BLOCKED_PROVENANCE",
            "error": f"{type(error).__name__}: {error}",
            "subject_executions": 0,
            "network_actions": 0,
            "runtime_authority_changed": False,
        }, sort_keys=True))
        return 1


if __name__ == "__main__":
    raise SystemExit(main())
