#!/usr/bin/env python3
"""Offline accounting for the V10 E1 single-to-twenty traversal inflation."""

from __future__ import annotations

import argparse
import ast
import hashlib
import itertools
import json
import math
import os
import pathlib
import shutil
import stat
import struct
import sys
import time
from typing import Any


AUDITOR = pathlib.Path(__file__).resolve()
ROOT = AUDITOR.parents[1]
TASK_ID = "slice8b-v10-e1-traversal-d6-concurrency-accounting-v1-20260826"
RECORD = struct.Struct("<HHBB" + "Q" * 14)
PHASES = ("oracle", "lanes", "eqmask", "traversal", "merge", "certificate")
ROUTES = ("C-SINGLE", "C-FIXED", "C-REVERSED")
EDGES_PER_ROUND = 25_145_756
MEASURED_ROUNDS = 20
MEASURED_EDGES = EDGES_PER_ROUND * MEASURED_ROUNDS

PAPER = ROOT / (
    "docs/structural_gates/"
    "LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_D6_"
    "CONCURRENCY_ACCOUNTING_V1_2026-08-26.md"
)
STRUCTURAL_REVIEW = ROOT / (
    "docs/structural_gates/receipts/"
    "LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_D6_"
    "CONCURRENCY_ACCOUNTING_STRUCTURAL_REVIEW_V1_2026-08-26.json"
)
PREFLIGHT = ROOT / (
    "docs/structural_gates/preflights/"
    "LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_D6_"
    "CONCURRENCY_ACCOUNTING_IMPLEMENTATION_V1_2026-08-26.json"
)
PREFLIGHT_RECEIPT = ROOT / (
    "docs/structural_gates/receipts/"
    "LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_D6_"
    "CONCURRENCY_ACCOUNTING_IMPLEMENTATION_V1_PREFLIGHT_2026-08-26.json"
)
D1_ROOT = ROOT / (
    "docs/structural_gates/receipts/"
    "LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_REMAINING_COST_D1_2026-08-25"
)
D1_MANIFEST = D1_ROOT / "SHA256SUMS"
PMU = ROOT / (
    "docs/structural_gates/receipts/"
    "LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_LOADED_PMU_DIAGNOSIS_"
    "COMBINED_V3_V4_2026-08-25.json"
)
D4_TERMINAL = ROOT / (
    "docs/structural_gates/receipts/"
    "LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_D4_"
    "TERMINAL_AUDIT_V1_2026-08-26/D4_TERMINAL_AUDIT_RECEIPT.json"
)
D5_FORENSIC = ROOT / (
    "docs/structural_gates/receipts/"
    "LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_D5_T4_"
    "OFFLINE_FORENSIC_AUDIT_V1_2026-08-26/D5_T4_FORENSIC_AUDIT_RECEIPT.json"
)
RESULT = ROOT / (
    "docs/structural_gates/receipts/"
    "LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_D6_"
    "CONCURRENCY_ACCOUNTING_V1_2026-08-26"
)

PINNED: dict[pathlib.Path, tuple[str, int, str]] = {
    PAPER: ("0444", 7605, "c23f1ffd52b08683a43984ed91f28cb3daf28f98d86e72967b67a27ba2a8567d"),
    STRUCTURAL_REVIEW: (
        "0444",
        1290,
        "8e41540abeb8cd008f40793352f7d9e3037949b48196ec07d73f6dc6caee1c15",
    ),
    PREFLIGHT: (
        "0444",
        12010,
        "7d81562a555a14751470d2846e8bfced420ef521ca7b61acf765acdc7276118e",
    ),
    PREFLIGHT_RECEIPT: (
        "0444",
        8015,
        "1e5af9e3ee08db674e67ed2fac7599a897b4dd0b6f874af01641f8e216821377",
    ),
    D1_MANIFEST: (
        "0444",
        7021,
        "c6e6f0674f773fd397a1fcc0b383fb1e7d1693a768b06e1c6fd21bb7291dcd83",
    ),
    PMU: ("0664", 4621, "ea9a19cace1eab5418f783dfb6c18a4de2adb7281356afffd12bb2b28cdacbd1"),
    D4_TERMINAL: (
        "0444",
        3685,
        "f748a7c448f7f142a342efa2b916a8995969044567a3aaa9a7de3aff6ba8645b",
    ),
    D5_FORENSIC: (
        "0444",
        2115,
        "d44ade85316f6f6f6eeb0917d3cdea168fc083e1a52b6c3b5e88fdf2df80ae20",
    ),
}


class AccountingError(RuntimeError):
    pass


def require(condition: bool, message: str) -> None:
    if not condition:
        raise AccountingError(message)


def sha256_file(path: pathlib.Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as handle:
        for chunk in iter(lambda: handle.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def mode(path: pathlib.Path) -> str:
    return f"{stat.S_IMODE(path.stat().st_mode):04o}"


def row(path: pathlib.Path) -> dict[str, Any]:
    return {
        "path": path.relative_to(ROOT).as_posix(),
        "mode": mode(path),
        "size_bytes": path.stat().st_size,
        "sha256": sha256_file(path),
    }


def write_new(path: pathlib.Path, data: bytes, file_mode: int = 0o444) -> None:
    descriptor = os.open(path, os.O_WRONLY | os.O_CREAT | os.O_EXCL, 0o600)
    try:
        view = memoryview(data)
        while view:
            written = os.write(descriptor, view)
            require(written > 0, "short write made no progress")
            view = view[written:]
        os.fsync(descriptor)
        os.fchmod(descriptor, file_mode)
    finally:
        os.close(descriptor)


def write_json(path: pathlib.Path, value: Any) -> None:
    write_new(path, json.dumps(value, indent=2, sort_keys=True).encode() + b"\n")


def fsync_dir(path: pathlib.Path) -> None:
    descriptor = os.open(path, os.O_RDONLY | os.O_DIRECTORY)
    try:
        os.fsync(descriptor)
    finally:
        os.close(descriptor)


def inventory(root: pathlib.Path) -> list[dict[str, Any]]:
    return [
        {
            "path": path.relative_to(root).as_posix(),
            "mode": mode(path),
            "size_bytes": path.stat().st_size,
            "sha256": sha256_file(path),
        }
        for path in sorted(root.rglob("*"))
        if path.is_file()
    ]


def write_sums(root: pathlib.Path) -> None:
    entries = [item for item in inventory(root) if item["path"] != "SHA256SUMS"]
    write_new(
        root / "SHA256SUMS",
        "".join(f"{item['sha256']}  {item['path']}\n" for item in entries).encode(),
    )


def seal(root: pathlib.Path) -> None:
    for path in sorted(root.rglob("*"), reverse=True):
        path.chmod(0o555 if path.is_dir() or path.stat().st_mode & 0o111 else 0o444)
    root.chmod(0o555)


def verify_pinned() -> dict[str, dict[str, Any]]:
    values: dict[str, dict[str, Any]] = {}
    for path, (expected_mode, expected_size, expected_sha) in PINNED.items():
        require(path.is_file(), f"missing pinned input: {path}")
        value = row(path)
        require(value["mode"] == expected_mode, f"mode drift: {path}")
        require(value["size_bytes"] == expected_size, f"size drift: {path}")
        require(value["sha256"] == expected_sha, f"SHA drift: {path}")
        values[value["path"]] = value
    preflight = json.loads(PREFLIGHT_RECEIPT.read_text())
    require(
        preflight.get("verdict") == "READY_TO_IMPLEMENT"
        and preflight.get("safe_to_implement") is True,
        "D6 preflight is not READY_TO_IMPLEMENT",
    )
    structural = json.loads(STRUCTURAL_REVIEW.read_text())
    require(
        structural.get("verdict") == "STRUCTURALLY_ACCEPTED_WITH_SPLIT"
        and structural.get("all_routes_pass") is True
        and structural.get("authority_ready") is False,
        "D6 structural review drift",
    )
    return values


def verify_d1_manifest() -> dict[str, Any]:
    expected: dict[str, str] = {}
    for line in D1_MANIFEST.read_text().splitlines():
        digest, separator, relative = line.partition("  ")
        require(separator == "  " and len(digest) == 64, f"bad D1 manifest row: {line}")
        pure = pathlib.PurePosixPath(relative)
        require(not pure.is_absolute() and ".." not in pure.parts, f"unsafe D1 path: {relative}")
        require(relative not in expected, f"duplicate D1 manifest row: {relative}")
        expected[relative] = digest
    actual_paths = {
        path.relative_to(D1_ROOT).as_posix()
        for path in D1_ROOT.rglob("*")
        if path.is_file() and path != D1_MANIFEST
    }
    require(actual_paths == set(expected), "D1 manifest membership drift")
    for relative, digest in expected.items():
        require(sha256_file(D1_ROOT / relative) == digest, f"D1 manifest digest drift: {relative}")
    return {"entries": len(expected), "manifest": row(D1_MANIFEST)}


def structure_for(route: str) -> tuple[dict[int, int], dict[str, Any]]:
    path = D1_ROOT / route / "subject/structure.json"
    value = json.loads(path.read_text())
    queries = value.get("queries")
    require(value.get("schema") == "lay.v10.e1-remaining-cost-d1-structure.v1", "structure schema drift")
    require(isinstance(queries, list) and len(queries) == 382, f"structure query count drift: {route}")
    edges = {int(item["query_ordinal"]): int(item["examined_edges"]) for item in queries}
    require(sorted(edges) == list(range(382)), f"structure ordinals drift: {route}")
    require(sum(edges.values()) == EDGES_PER_ROUND, f"structure edge denominator drift: {route}")
    return edges, row(path)


def expected_worker(route: str, query: int) -> int:
    return 255 if route == "C-SINGLE" else query // 20


def parse_component(route: str) -> dict[str, Any]:
    sample_path = D1_ROOT / route / "subject/component-samples.bin"
    receipt_path = D1_ROOT / route / "subject/SUBJECT_RECEIPT.json"
    edges, structure_row = structure_for(route)
    raw = sample_path.read_bytes()
    require(len(raw) == 7_640 * RECORD.size, f"component size drift: {route}")
    phase_cpu = {phase: 0 for phase in PHASES}
    phase_wall = {phase: 0 for phase in PHASES}
    outer_cpu = 0
    outer_wall = 0
    seen: set[tuple[int, int]] = set()
    worker_totals: dict[int, dict[str, int]] = {}
    for values in RECORD.iter_unpack(raw):
        query, round_id, worker, flags = values[:4]
        require(query < 382 and round_id < 20, f"component index drift: {route}")
        require(flags == 0, f"component error flags: {route}")
        require(worker == expected_worker(route, query), f"worker/query drift: {route}")
        key = (query, round_id)
        require(key not in seen, f"duplicate component record: {route} {key}")
        seen.add(key)
        outer_wall += values[4]
        outer_cpu += values[5]
        for index, phase in enumerate(PHASES):
            phase_wall[phase] += values[6 + index * 2]
            phase_cpu[phase] += values[7 + index * 2]
        worker_value = worker_totals.setdefault(worker, {"edges": 0, "traversal_cpu_ns": 0})
        worker_value["edges"] += edges[query]
        worker_value["traversal_cpu_ns"] += values[13]
    require(len(seen) == 7_640, f"component record closure drift: {route}")
    receipt = json.loads(receipt_path.read_text())
    require(receipt.get("rounds") == 20 and receipt.get("queries") == 382, f"receipt envelope drift: {route}")
    require(receipt.get("samples", {}).get("errors") == 0, f"receipt errors: {route}")
    require(receipt.get("samples", {}).get("unresolved") == 0, f"receipt unresolved: {route}")
    if route == "C-SINGLE":
        require(receipt.get("mapping") == "SINGLE" and receipt.get("cpus") == [0], "single mapping drift")
        require(receipt.get("workers") == 1 and receipt.get("thread_migrations") == 0, "single worker drift")
    else:
        cpus = list(range(20)) if route == "C-FIXED" else list(reversed(range(20)))
        require(receipt.get("mapping") == route.removeprefix("C-"), f"mapping label drift: {route}")
        require(receipt.get("cpus") == cpus, f"CPU mapping drift: {route}")
        require(receipt.get("worker_affinities") == [[cpu] for cpu in cpus], f"affinity drift: {route}")
        require(receipt.get("worker_migration_deltas") == [0] * 20, f"migration drift: {route}")
    for value in worker_totals.values():
        value["ns_per_edge"] = value["traversal_cpu_ns"] / value["edges"]
    return {
        "route": route,
        "records": len(seen),
        "measured_edges": MEASURED_EDGES,
        "outer_thread_cpu_ns": outer_cpu,
        "outer_thread_cpu_per_edge_ns": outer_cpu / MEASURED_EDGES,
        "outer_wall_ns": outer_wall,
        "phase_thread_cpu_ns": phase_cpu,
        "phase_thread_cpu_per_edge_ns": {
            phase: value / MEASURED_EDGES for phase, value in phase_cpu.items()
        },
        "phase_wall_ns": phase_wall,
        "workers": worker_totals,
        "samples": row(sample_path),
        "structure": structure_row,
        "subject_receipt": row(receipt_path),
    }


def paired_core_class(routes: dict[str, dict[str, Any]]) -> dict[str, Any]:
    totals = {
        "P": {"traversal_cpu_ns": 0, "edges": 0},
        "E": {"traversal_cpu_ns": 0, "edges": 0},
    }
    crossing_workers = []
    for worker in range(20):
        fixed_cpu = worker
        reversed_cpu = 19 - worker
        if (fixed_cpu < 12) == (reversed_cpu < 12):
            continue
        crossing_workers.append(worker)
        for route, cpu in (("C-FIXED", fixed_cpu), ("C-REVERSED", reversed_cpu)):
            label = "P" if cpu < 12 else "E"
            value = routes[route]["workers"][worker]
            totals[label]["traversal_cpu_ns"] += value["traversal_cpu_ns"]
            totals[label]["edges"] += value["edges"]
    require(crossing_workers == list(range(8)) + list(range(12, 20)), "cross-class worker set drift")
    for value in totals.values():
        value["ns_per_edge"] = value["traversal_cpu_ns"] / value["edges"]
    delta = totals["E"]["ns_per_edge"] - totals["P"]["ns_per_edge"]
    return {
        "crossing_workers": crossing_workers,
        "P": totals["P"],
        "E": totals["E"],
        "E_minus_P_ns_per_edge": delta,
        "E_over_P": totals["E"]["ns_per_edge"] / totals["P"]["ns_per_edge"],
    }


def pmu_accounting(single_ns: float, fixed_ns: float) -> dict[str, Any]:
    value = json.loads(PMU.read_text())
    require(
        value.get("verdict") == "LOADED_PMU_DIAGNOSIS_OBSERVED_WITH_L1_MISS_GAP",
        "PMU verdict drift",
    )
    require(value.get("subject", {}).get("b5_b6_structural_work_identical") is True, "PMU work drift")
    g0 = value.get("g0", {})
    b5 = g0.get("b5", {})
    b6 = g0.get("b6", {})
    required = ("instructions_per_request", "cycles_per_request", "ipc", "task_clock_ms_per_request")
    require(all(isinstance(b5.get(key), (int, float)) for key in required), "B5 G0 incomplete")
    require(all(isinstance(b6.get(key), (int, float)) for key in required), "B6 G0 incomplete")
    frequency_b5 = b5["cycles_per_request"] / (b5["task_clock_ms_per_request"] * 1_000_000)
    frequency_b6 = b6["cycles_per_request"] / (b6["task_clock_ms_per_request"] * 1_000_000)
    factors = {
        "instructions": b6["instructions_per_request"] / b5["instructions_per_request"],
        "ipc_loss": b5["ipc"] / b6["ipc"],
        "frequency_loss": frequency_b5 / frequency_b6,
    }
    model = single_ns * math.prod(factors.values())
    contributions = {key: 0.0 for key in factors}
    for permutation in itertools.permutations(factors):
        current = single_ns
        for key in permutation:
            following = current * factors[key]
            contributions[key] += following - current
            current = following
    for key in contributions:
        contributions[key] /= math.factorial(len(factors))
    observed_delta = fixed_ns - single_ns
    scale = observed_delta / (model - single_ns)
    return {
        "B5": b5,
        "B6": b6,
        "effective_frequency_ghz": {"B5": frequency_b5, "B6": frequency_b6},
        "factors": factors,
        "combined_factor": math.prod(factors.values()),
        "predicted_fixed_ns_per_edge": model,
        "observed_fixed_ns_per_edge": fixed_ns,
        "residual_ns_per_edge": model - fixed_ns,
        "residual_percent_of_observed": (model - fixed_ns) / fixed_ns * 100,
        "shapley_unscaled_ns_per_edge": contributions,
        "shapley_scaled_to_observed_ns_per_edge": {
            key: contribution * scale for key, contribution in contributions.items()
        },
        "source": row(PMU),
    }


def close_expected(detail: dict[str, Any]) -> None:
    routes = detail["routes"]
    expected = {
        "C-SINGLE": 25.96501044152341,
        "C-FIXED": 44.735012045372585,
        "C-REVERSED": 44.6967635154815,
    }
    for route, value in expected.items():
        observed = routes[route]["phase_thread_cpu_per_edge_ns"]["traversal"]
        require(math.isclose(observed, value, rel_tol=0.0, abs_tol=1e-12), f"traversal result drift: {route}")
    paired = detail["paired_core_class"]
    require(math.isclose(paired["P"]["ns_per_edge"], 42.97686435720271, abs_tol=1e-12), "P result drift")
    require(math.isclose(paired["E"]["ns_per_edge"], 46.75424073181106, abs_tol=1e-12), "E result drift")
    pmu = detail["pmu_accounting"]
    require(
        math.isclose(pmu["predicted_fixed_ns_per_edge"], 44.80148646392056, abs_tol=1e-12),
        "PMU model drift",
    )
    require(pmu["residual_percent_of_observed"] < 0.15, "cross-route residual exceeds contract")


def self_check() -> dict[str, Any]:
    tree = ast.parse(AUDITOR.read_text())
    imported = {
        alias.name.split(".")[0]
        for node in ast.walk(tree)
        if isinstance(node, ast.Import)
        for alias in node.names
    }
    imported.update(
        node.module.split(".")[0]
        for node in ast.walk(tree)
        if isinstance(node, ast.ImportFrom) and node.module
    )
    forbidden_modules = {"sub" + "process", "socket", "urllib", "http", "requests", "paramiko"}
    require(not imported.intersection(forbidden_modules), "external-capable module imported")
    return {
        "schema": "lay.v10.e1-traversal-d6-concurrency-accounting-self-check.v1",
        "task_id": TASK_ID,
        "verdict": "D6_ACCOUNTING_AUDITOR_VERIFIED_UNRUN",
        "auditor": row(AUDITOR),
        "external_commands": 0,
        "network_or_remote": 0,
        "perf_or_pmu": 0,
        "subject_executions": 0,
        "marker_mutations": 0,
        "runtime_authority_changed": False,
    }


def copy_input(source: pathlib.Path, destination: pathlib.Path) -> None:
    data = source.read_bytes()
    write_new(destination, data, 0o555 if source == AUDITOR else 0o444)


def audit() -> dict[str, Any]:
    require(not RESULT.exists(), f"D6 result already exists: {RESULT}")
    pinned = verify_pinned()
    manifest = verify_d1_manifest()
    d4 = json.loads(D4_TERMINAL.read_text())
    require(
        d4.get("verdict") == "D4_SINGLE_ESTIMATOR_PASS"
        and d4.get("optimization_authority") is False,
        "D4 authority drift",
    )
    d5 = json.loads(D5_FORENSIC.read_text())
    require(
        d5.get("verdict") == "D5_T4_FORENSIC_DIAGNOSTIC_COMPLETE"
        and d5.get("diagnostic_attribution_valid_for_d5") is False
        and d5.get("optimization_authority") is False,
        "D5 forensic boundary drift",
    )
    routes = {route: parse_component(route) for route in ROUTES}
    single = routes["C-SINGLE"]["phase_thread_cpu_per_edge_ns"]["traversal"]
    fixed = routes["C-FIXED"]["phase_thread_cpu_per_edge_ns"]["traversal"]
    reversed_value = routes["C-REVERSED"]["phase_thread_cpu_per_edge_ns"]["traversal"]
    detail = {
        "schema": "lay.v10.e1-traversal-d6-concurrency-accounting-detail.v1",
        "task_id": TASK_ID,
        "denominator": {
            "examined_edges_per_round": EDGES_PER_ROUND,
            "measured_rounds": MEASURED_ROUNDS,
            "measured_edges": MEASURED_EDGES,
        },
        "routes": routes,
        "fixed_minus_single_ns_per_edge": fixed - single,
        "reversed_minus_single_ns_per_edge": reversed_value - single,
        "paired_core_class": paired_core_class(routes),
        "pmu_accounting": pmu_accounting(single, fixed),
        "throughput": {
            "single_edges_per_second": 1_000_000_000 / single,
            "twenty_total_edges_per_second": 20_000_000_000 / fixed,
            "twenty_vs_single_scaling": 20 * single / fixed,
            "parallel_efficiency": single / fixed,
        },
        "claim_boundary": {
            "aggregate_accounting_complete": True,
            "microarchitectural_root_cause_isolated": False,
            "production_twenty_worker_concurrency_established": False,
            "optimization_authority": False,
        },
    }
    close_expected(detail)
    check = self_check()
    stage = RESULT.with_name(f".{RESULT.name}.stage-{os.getpid()}-{time.time_ns()}")
    require(not stage.exists(), "D6 stage collision")
    stage.mkdir(mode=0o700, parents=False)
    try:
        receipt = {
            "schema": "lay.v10.e1-traversal-d6-concurrency-accounting-audit.v1",
            "task_id": TASK_ID,
            "verdict": "D6_CONCURRENCY_ACCOUNTING_COMPLETE",
            "single_ns_per_edge": single,
            "fixed_ns_per_edge": fixed,
            "reversed_ns_per_edge": reversed_value,
            "fixed_minus_single_ns_per_edge": fixed - single,
            "reversed_minus_single_ns_per_edge": reversed_value - single,
            "paired_E_minus_P_ns_per_edge": detail["paired_core_class"]["E_minus_P_ns_per_edge"],
            "instruction_factor": detail["pmu_accounting"]["factors"]["instructions"],
            "ipc_loss_factor": detail["pmu_accounting"]["factors"]["ipc_loss"],
            "frequency_loss_factor": detail["pmu_accounting"]["factors"]["frequency_loss"],
            "predicted_fixed_ns_per_edge": detail["pmu_accounting"]["predicted_fixed_ns_per_edge"],
            "model_residual_ns_per_edge": detail["pmu_accounting"]["residual_ns_per_edge"],
            "model_residual_percent": detail["pmu_accounting"]["residual_percent_of_observed"],
            "shapley_is_accounting_convention": True,
            "aggregate_accounting_complete": True,
            "microarchitectural_root_cause_isolated": False,
            "production_twenty_worker_concurrency_established": False,
            "optimization_authority": False,
            "next_action_admitted": "separate 1/6/12/14/20 worker-count and CPU-placement sweep paper only",
            "external_commands": 0,
            "network_or_remote": 0,
            "perf_or_pmu": 0,
            "subject_executions": 0,
            "marker_mutations": 0,
            "runtime_authority_changed": False,
        }
        write_json(stage / "D6_CONCURRENCY_ACCOUNTING_RECEIPT.json", receipt)
        write_json(stage / "ACCOUNTING_DETAIL.json", detail)
        write_json(stage / "INPUT_IDENTITIES.json", {"pinned": pinned, "d1_manifest": manifest})
        write_json(stage / "SELF_CHECK.json", check)
        copy_input(AUDITOR, stage / "auditor.py")
        copy_input(PAPER, stage / "paper.md")
        copy_input(STRUCTURAL_REVIEW, stage / "structural-review.json")
        copy_input(PREFLIGHT, stage / "preflight-v1.json")
        copy_input(PREFLIGHT_RECEIPT, stage / "preflight-v1-receipt.json")
        write_sums(stage)
        fsync_dir(stage)
        seal(stage)
        os.rename(stage, RESULT)
        fsync_dir(RESULT.parent)
    except BaseException:
        if stage.exists():
            shutil.rmtree(stage)
        raise
    receipt_path = RESULT / "D6_CONCURRENCY_ACCOUNTING_RECEIPT.json"
    return {
        **receipt,
        "result": str(RESULT),
        "receipt_sha256": sha256_file(receipt_path),
        "auditor_sha256": sha256_file(AUDITOR),
    }


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("action", choices=("self-check", "audit"))
    arguments = parser.parse_args()
    try:
        value = self_check() if arguments.action == "self-check" else audit()
    except (AccountingError, OSError, ValueError, KeyError, json.JSONDecodeError) as error:
        print(f"D6 ACCOUNTING ERROR: {type(error).__name__}: {error}", file=sys.stderr)
        return 1
    print(json.dumps(value, sort_keys=True))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
