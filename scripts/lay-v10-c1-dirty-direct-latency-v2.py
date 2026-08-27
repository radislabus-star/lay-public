#!/usr/bin/env python3
"""One-shot direct C1 latency observation under the current background load."""

from __future__ import annotations

import argparse
import contextlib
import datetime as dt
import fcntl
import hashlib
import importlib.util
import json
import os
import pathlib
import re
import shlex
import signal
import socket
import stat
import subprocess
import sys
import tempfile
import time
from types import ModuleType
from typing import Any, Iterable, Sequence


TASK_ID = "slice8b-v10-c1-dirty-direct-latency-v2-20260825"
REMOTE = "e@192.168.3.94"
REMOTE_HOSTNAME = "e-MEGA-MINI-M1-13th"
REMOTE_MACHINE_ID_SHA256 = "5ac0bb537745673ef80942c2df32d586f5753c85de1c8b2453aa4e7317b77441"
REMOTE_PROVENANCE = pathlib.Path("/home/e/.local/share/lay/provenance")
REMOTE_FINAL = REMOTE_PROVENANCE / TASK_ID
REMOTE_WORK = REMOTE_PROVENANCE / f".{TASK_ID}.work"
REMOTE_STATE = pathlib.Path("/home/e/.local/state/lay") / TASK_ID

CLEAN_TASK_ID = "slice8b-v10-c1-direct-latency-v1-20260825"
CLEAN_FINAL = REMOTE_PROVENANCE / CLEAN_TASK_ID
CLEAN_WORK = REMOTE_PROVENANCE / f".{CLEAN_TASK_ID}.work"
CLEAN_STATE = pathlib.Path("/home/e/.local/state/lay") / CLEAN_TASK_ID
CLEAN_CONTROLLER_REMOTE = CLEAN_WORK / "inputs/lay-v10-c1-direct-latency.py"
CLEAN_ELF = CLEAN_WORK / "build-v1/c1-test-elf"
CLEAN_BUILD_RECEIPT = CLEAN_WORK / "build-v1/EXECUTABLE_PROVENANCE.json"
CLEAN_PARITY_RECEIPT = CLEAN_WORK / "parity-v1/subject/SUBJECT_RECEIPT.json"

PROJECT_ROOT = pathlib.Path(__file__).resolve().parents[1]
CLEAN_CONTROLLER_LOCAL = PROJECT_ROOT / "scripts/lay-v10-c1-direct-latency.py"
ACTIVE_V11 = PROJECT_ROOT / "src/nanda_wave/l2_field/v13_typed_peak.rs"
PREFLIGHT_MANIFEST = PROJECT_ROOT / (
    "docs/structural_gates/preflights/"
    "LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_C1_DIRTY_LOAD_REPLICATION_V2_2026-08-25.json"
)
PREFLIGHT_RECEIPT = PROJECT_ROOT / (
    "docs/structural_gates/receipts/"
    "LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_C1_DIRTY_LOAD_REPLICATION_V2_PREFLIGHT_2026-08-25.json"
)
CONTRACT = PROJECT_ROOT / (
    "docs/structural_gates/"
    "LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_C1_DIRTY_LOAD_REPLICATION_V2_CONTRACT_2026-08-25.md"
)
ROUTE_RECEIPT = PROJECT_ROOT / (
    "docs/structural_gates/receipts/"
    "LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_C1_DIRTY_LOAD_ROUTE_V2_2026-08-25.json"
)
LOCAL_FINAL = PROJECT_ROOT / (
    "docs/structural_gates/receipts/"
    "LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_C1_DIRTY_DIRECT_LATENCY_V2_2026-08-25"
)

EXPECTED = {
    "clean_controller_local": "e6e3db8529b4bd5d64d11b903b3b2ef67e808dd202d5de0b7287fc644a151718",
    "clean_controller_remote": "4c09ac00e3c67f3cdb9b52a9930618e6d438dfa1c887e5fd48b346e73033e5bf",
    "elf": "ead184029a2923cfd24c5d02e91e91f9d69fb01c7daa0fb21ba69f267234c93c",
    "elf_size": 20_531_304,
    "elf_build_id": "665458be5064a689951c6f074276ab0bb4d2beb4",
    "active_v11": "d9edfe7346b8636096701d4ac38044d5e386e6741a086a4d8da57ac15bffdf3b",
    "contract": "162a6295f9d3a8ee9fd00bdff128b6a5aa39fe94ed60179f780a2820ec2ab0a7",
    "route_receipt": "bacd4f8b010d84a665ae005333b0cb44d7e5fdad454403beb342cc9bc167903a",
    "preflight_manifest_file": "5694e3d757be731a00331470a073035c7cab3e75c1a17e960d95822692ca028b",
    "preflight_manifest_identity": "29859b272ee7ee69818fd2a8aa08d81774037a49b8117c4a8f551d4b72ffbf02",
    "preflight_receipt": "72de009dc052a8a2831c87c8369753dfb8caf03f47d58942132aa5aa24599a0c",
}

ORDER = ["S1", "T1", "T2", "S2", "S3", "T3", "T4", "S4", "S5", "T5"]
SAMPLES_PER_RUN = {"S": 38_200, "T": 95_500}
LATENCY_THRESHOLDS = {
    "single_search_p99_us": 3_000,
    "single_total_p99_us": 5_000,
    "twenty_total_p99_us": 5_000,
    "fairness_total_p99_us": 5_000,
}
EXPECTED_CLEAN_MARKERS = sorted(
    [f"{route}.available" for route in ORDER]
    + ["build.consumed-before-exec", "parity.consumed-before-exec"]
)
REMOTE_EXECUTION = bool(globals().get("REMOTE_EXECUTION", False))
CONTROLLER_SOURCE_BYTES = globals().get("CONTROLLER_SOURCE_BYTES")


class DirtyDirectError(RuntimeError):
    pass


def require(condition: bool, message: str) -> None:
    if not condition:
        raise DirtyDirectError(message)


def now() -> str:
    return dt.datetime.now(dt.timezone.utc).isoformat()


def pretty_json(value: Any) -> bytes:
    return (json.dumps(value, indent=2, sort_keys=True) + "\n").encode()


def sha256_bytes(value: bytes) -> str:
    return hashlib.sha256(value).hexdigest()


def sha256_file(path: pathlib.Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as source:
        for block in iter(lambda: source.read(1024 * 1024), b""):
            digest.update(block)
    return digest.hexdigest()


def mode_string(path: pathlib.Path) -> str:
    return f"{stat.S_IMODE(path.stat().st_mode):04o}"


def file_identity(path: pathlib.Path) -> dict[str, Any]:
    return {
        "path": str(path),
        "size_bytes": path.stat().st_size,
        "sha256": sha256_file(path),
        "mode": mode_string(path),
    }


def require_file(
    path: pathlib.Path,
    *,
    digest: str | None = None,
    size: int | None = None,
    mode: str | None = None,
) -> dict[str, Any]:
    require(path.is_file(), f"missing file: {path}")
    identity = file_identity(path)
    if digest is not None:
        require(identity["sha256"] == digest, f"SHA-256 mismatch: {path}")
    if size is not None:
        require(identity["size_bytes"] == size, f"size mismatch: {path}")
    if mode is not None:
        require(identity["mode"] == mode, f"mode mismatch: {path}")
    return identity


def write_new_bytes(path: pathlib.Path, value: bytes, mode: int = 0o600) -> None:
    path.parent.mkdir(parents=True, exist_ok=True, mode=0o700)
    descriptor = os.open(path, os.O_WRONLY | os.O_CREAT | os.O_EXCL, mode)
    try:
        with os.fdopen(descriptor, "wb", closefd=False) as output:
            output.write(value)
            output.flush()
            os.fsync(output.fileno())
    finally:
        os.close(descriptor)


def write_new_json(path: pathlib.Path, value: Any, mode: int = 0o600) -> None:
    write_new_bytes(path, pretty_json(value), mode)


def write_sha256sums(root: pathlib.Path) -> None:
    rows = []
    for path in sorted(item for item in root.rglob("*") if item.is_file()):
        if path.name != "SHA256SUMS":
            rows.append(f"{sha256_file(path)}  {path.relative_to(root)}\n")
    write_new_bytes(root / "SHA256SUMS", "".join(rows).encode())


def verify_sha256sums(root: pathlib.Path) -> int:
    manifest = root / "SHA256SUMS"
    require(manifest.is_file(), f"missing manifest: {manifest}")
    count = 0
    for line in manifest.read_text().splitlines():
        digest, relative = line.split("  ", 1)
        require(sha256_file(root / relative) == digest, f"manifest mismatch: {relative}")
        count += 1
    return count


def seal_tree(root: pathlib.Path) -> None:
    for path in sorted(root.rglob("*"), reverse=True):
        path.chmod(0o555 if path.is_dir() else 0o444)
    root.chmod(0o555)


def atomic_publish(stage: pathlib.Path, final: pathlib.Path) -> None:
    require(not final.exists(), f"final path already exists: {final}")
    os.rename(stage, final)


def load_clean_controller(path: pathlib.Path, digest: str) -> ModuleType:
    require_file(path, digest=digest)
    spec = importlib.util.spec_from_file_location("lay_v10_clean_c1_controller", path)
    require(spec is not None and spec.loader is not None, "cannot load clean C1 controller")
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def clean_marker_snapshot() -> dict[str, Any]:
    require(CLEAN_STATE.is_dir(), "clean C1 state is absent")
    marker_root = CLEAN_STATE / "markers"
    require(marker_root.is_dir(), "clean C1 marker directory is absent")
    names = sorted(path.name for path in marker_root.iterdir())
    require(names == EXPECTED_CLEAN_MARKERS, f"clean C1 marker set drift: {names}")
    require(not CLEAN_FINAL.exists(), "clean C1 final path unexpectedly exists")
    return {
        "state_path": str(CLEAN_STATE),
        "state_mode": mode_string(CLEAN_STATE),
        "marker_directory_mode": mode_string(marker_root),
        "markers": {name: file_identity(marker_root / name) for name in names},
        "clean_final_exists": False,
    }


def verify_parity() -> dict[str, Any]:
    receipt = json.loads(CLEAN_PARITY_RECEIPT.read_text())
    expected = {
        "verdict": "PASS",
        "records": 382,
        "schedule_records": 382,
        "terminal_mismatches": 0,
        "peak_mismatches": 0,
        "completeness_mismatches": 0,
        "work_mismatches": 0,
        "target_form_retained": 382,
        "target_lemma_retained": 382,
        "false_certificates": 0,
        "maximum_product_states": 35_590,
        "maximum_scratch_bytes": 6_656,
    }
    for key, value in expected.items():
        require(receipt.get(key) == value, f"parity mismatch {key}: {receipt.get(key)!r}")
    return {"identity": file_identity(CLEAN_PARITY_RECEIPT), "receipt": receipt}


def verify_remote_subject(clean: ModuleType) -> dict[str, Any]:
    require(socket.gethostname() == REMOTE_HOSTNAME, "remote hostname mismatch")
    require(
        sha256_file(pathlib.Path("/etc/machine-id")) == REMOTE_MACHINE_ID_SHA256,
        "remote machine identity mismatch",
    )
    inputs = clean.remote_input_identities()
    build = json.loads(CLEAN_BUILD_RECEIPT.read_text())
    executable = build.get("executable", {})
    require(executable.get("sha256") == EXPECTED["elf"], "build receipt ELF SHA mismatch")
    require(executable.get("size_bytes") == EXPECTED["elf_size"], "build receipt ELF size mismatch")
    require(executable.get("build_id") == EXPECTED["elf_build_id"], "build receipt Build ID mismatch")
    elf = require_file(
        CLEAN_ELF,
        digest=EXPECTED["elf"],
        size=EXPECTED["elf_size"],
        mode="0555",
    )
    require(clean.elf_build_id(CLEAN_ELF) == EXPECTED["elf_build_id"], "ELF Build ID mismatch")
    parity = verify_parity()
    return {
        "inputs": inputs,
        "build_receipt": file_identity(CLEAN_BUILD_RECEIPT),
        "elf": elf,
        "elf_build_id": EXPECTED["elf_build_id"],
        "parity": parity,
        "parity_rerun": False,
        "build_executed": False,
    }


def read_text(path: pathlib.Path) -> str | None:
    try:
        return path.read_text(encoding="utf-8", errors="replace").strip()
    except (FileNotFoundError, OSError, PermissionError):
        return None


def pressure(path: pathlib.Path) -> dict[str, dict[str, float]]:
    result: dict[str, dict[str, float]] = {}
    for line in (read_text(path) or "").splitlines():
        fields = line.split()
        result[fields[0]] = {
            key: float(value)
            for key, value in (field.split("=", 1) for field in fields[1:])
        }
    return result


def temperatures() -> list[dict[str, Any]]:
    result = []
    for path in sorted(pathlib.Path("/sys/class/hwmon").glob("hwmon*/temp*_input")):
        raw = read_text(path)
        if raw and raw.lstrip("-").isdigit():
            result.append(
                {
                    "path": str(path),
                    "label": read_text(path.with_name(path.name.replace("_input", "_label"))),
                    "millidegrees_c": int(raw),
                }
            )
    return result


def procs_running() -> int | None:
    for line in (read_text(pathlib.Path("/proc/stat")) or "").splitlines():
        if line.startswith("procs_running "):
            return int(line.split()[1])
    return None


def load_snapshot(clean: ModuleType) -> dict[str, Any]:
    thermal = temperatures()
    return {
        "observed_at": now(),
        "cpu_pressure": pressure(pathlib.Path("/proc/pressure/cpu")),
        "memory_pressure": pressure(pathlib.Path("/proc/pressure/memory")),
        "io_pressure": pressure(pathlib.Path("/proc/pressure/io")),
        "procs_running": procs_running(),
        "temperatures": thermal,
        "maximum_temperature_c": max(
            (item["millidegrees_c"] for item in thermal), default=0
        )
        / 1000.0,
        "stable_host_projection": clean.stable_host_projection(),
        "throttle_counters": clean.throttle_counters(),
    }


def process_deltas(
    before: dict[int, dict[str, Any]],
    after: dict[int, dict[str, Any]],
    elapsed_seconds: float,
) -> list[dict[str, Any]]:
    rows = []
    for pid in before.keys() & after.keys():
        delta = after[pid]["cpu_seconds"] - before[pid]["cpu_seconds"]
        cpu_percent = delta / max(elapsed_seconds, 0.001) * 100.0
        if cpu_percent >= 0.1 and after[pid].get("cmdline"):
            rows.append(
                {
                    "pid": pid,
                    "comm": after[pid].get("comm"),
                    "cmdline": after[pid].get("cmdline"),
                    "cpu_seconds_delta": delta,
                    "cpu_percent": cpu_percent,
                    "cpus_allowed_list": after[pid].get("cpus_allowed_list"),
                }
            )
    return sorted(rows, key=lambda item: item["cpu_percent"], reverse=True)[:100]


def consume_dirty_marker(route: str) -> pathlib.Path:
    available = REMOTE_STATE / "markers" / f"{route}.available"
    consumed = REMOTE_STATE / "markers" / f"{route}.consumed-before-exec"
    require(available.is_file() and not consumed.exists(), f"dirty marker unavailable: {route}")
    os.rename(available, consumed)
    return consumed


@contextlib.contextmanager
def dirty_route_lock() -> Iterable[None]:
    lock = REMOTE_STATE / "route.lock"
    descriptor = os.open(lock, os.O_RDONLY)
    try:
        try:
            fcntl.flock(descriptor, fcntl.LOCK_EX | fcntl.LOCK_NB)
        except BlockingIOError as error:
            raise DirtyDirectError("another dirty direct owner holds the route lock") from error
        yield
    finally:
        with contextlib.suppress(OSError):
            fcntl.flock(descriptor, fcntl.LOCK_UN)
        os.close(descriptor)


def initialize_remote_state(subject: dict[str, Any], clean_before: dict[str, Any]) -> None:
    require(not REMOTE_FINAL.exists(), "dirty direct final already exists")
    require(not REMOTE_WORK.exists(), "dirty direct work path already exists")
    require(not REMOTE_STATE.exists(), "dirty direct state already exists")
    REMOTE_STATE.mkdir(mode=0o700)
    marker_root = REMOTE_STATE / "markers"
    marker_root.mkdir(mode=0o700)
    write_new_bytes(REMOTE_STATE / "route.lock", b"dirty-direct\n", 0o400)
    for route in ORDER:
        write_new_json(
            marker_root / f"{route}.available",
            {"route": route, "available_at": now(), "retry_permitted": False},
            0o400,
        )
    REMOTE_WORK.mkdir(mode=0o700)
    write_new_json(
        REMOTE_WORK / "PREPARATION.json",
        {
            "schema": "lay.v10.c1-dirty-direct-preparation.v1",
            "task_id": TASK_ID,
            "prepared_at": now(),
            "controller_sha256": sha256_bytes(CONTROLLER_SOURCE_BYTES),
            "subject": subject,
            "clean_state_before": clean_before,
            "order": ORDER,
            "environment_intentionally_loaded": True,
            "quiet_admission_executed": False,
            "foreign_process_control": False,
            "perf_invoked": False,
            "pmu_event_opened": False,
            "cargo_invoked": False,
            "parity_rerun": False,
            "runtime_authority_changed": False,
        },
    )


def run_route(clean: ModuleType, route: str, clean_before: dict[str, Any]) -> dict[str, Any]:
    destination = REMOTE_WORK / f"run-{route}"
    require(not destination.exists(), f"dirty route already exists: {route}")
    stage = REMOTE_WORK / f".run-{route}.stage-{os.getpid()}-{time.time_ns()}"
    stage.mkdir(mode=0o700)
    marker = consume_dirty_marker(route)
    print(f"DIRTY_DIRECT_START {route} {now()}", file=sys.stderr, flush=True)
    started = time.monotonic()
    before_load = load_snapshot(clean)
    before_processes = clean.all_processes()
    consumed = True
    try:
        exit_code, wall_ns = clean.execute_subject(
            CLEAN_ELF,
            clean.TESTS[route[0]],
            route,
            stage,
        )
        elapsed = time.monotonic() - started
        after_processes = clean.all_processes()
        after_load = load_snapshot(clean)
        receipt, samples = clean.validate_subject_route(route, stage)
        require(exit_code == 0, f"dirty direct {route} exited {exit_code}")
        require(clean_marker_snapshot() == clean_before, f"clean marker drift during {route}")
        require(
            before_load["stable_host_projection"] == after_load["stable_host_projection"],
            f"stable host policy changed during {route}",
        )
        wrapper = {
            "schema": "lay.v10.c1-dirty-direct-route.v1",
            "task_id": TASK_ID,
            "route": route,
            "verdict": "DIRTY_ROUTE_OBSERVED",
            "measured_at": now(),
            "marker": str(marker),
            "marker_consumed_before_exec": consumed,
            "subject_receipt": receipt,
            "process_wall_ns_diagnostic": wall_ns,
            "percentiles": clean.route_percentiles(samples),
            "load_before": before_load,
            "load_after": after_load,
            "background_process_cpu_deltas": process_deltas(
                before_processes, after_processes, elapsed
            ),
            "throttle_counter_drift": (
                before_load["throttle_counters"] != after_load["throttle_counters"]
            ),
            "quiet_admission_executed": False,
            "foreign_load_was_blocker": False,
            "foreign_process_control": False,
            "replacement_process_admitted": False,
            "perf_invoked": False,
            "pmu_event_opened": False,
            "runtime_authority_changed": False,
        }
        write_new_json(stage / "DIRTY_ROUTE_RECEIPT.json", wrapper)
        write_sha256sums(stage)
        seal_tree(stage)
        atomic_publish(stage, destination)
        print(
            f"DIRTY_DIRECT_DONE {route} total_p99_us={wrapper['percentiles']['total_p99_us']}",
            file=sys.stderr,
            flush=True,
        )
        return wrapper
    except BaseException as error:
        if stage.exists():
            with contextlib.suppress(Exception):
                write_new_json(
                    stage / "FAILURE.json",
                    {
                        "verdict": "DIRTY_MATRIX_TERMINAL_NO_RETRY",
                        "route": route,
                        "error": str(error),
                        "marker_consumed_before_exec": consumed,
                    },
                )
                write_sha256sums(stage)
                seal_tree(stage)
                atomic_publish(stage, REMOTE_WORK / f"run-{route}.failed")
        raise


def aggregate_matrix(clean: ModuleType) -> dict[str, Any]:
    routes: dict[str, Any] = {}
    pooled_s: list[dict[str, int]] = []
    pooled_t: list[dict[str, int]] = []
    query_samples: dict[int, list[dict[str, int]]] = {index: [] for index in range(382)}
    fairness: list[tuple[str, int, int]] = []
    for route in ORDER:
        root = REMOTE_WORK / f"run-{route}"
        verify_sha256sums(root)
        samples = clean.parse_samples(root / "subject/samples.bin")
        kind = route[0]
        require(len(samples) == SAMPLES_PER_RUN[kind], f"sample count mismatch: {route}")
        routes[route] = clean.route_percentiles(samples)
        (pooled_s if kind == "S" else pooled_t).extend(samples)
        for sample in samples:
            query_samples[sample["query_ordinal"]].append(sample)
        if kind == "T":
            workers: dict[str, Any] = {}
            for worker in range(20):
                worker_samples = [
                    sample for sample in samples if sample["worker_id"] == worker
                ]
                workers[str(worker)] = clean.route_percentiles(worker_samples)
                fairness.append((route, worker, workers[str(worker)]["total_p99_us"]))
            routes[route]["workers"] = workers
    require(len(pooled_s) == 191_000, "pooled S denominator mismatch")
    require(len(pooled_t) == 477_500, "pooled T denominator mismatch")
    pooled = {
        "S": clean.route_percentiles(pooled_s),
        "T": clean.route_percentiles(pooled_t),
    }
    per_query = {
        str(query): clean.route_percentiles(samples)
        for query, samples in query_samples.items()
    }
    worst_query = max(per_query.items(), key=lambda item: item[1]["total_p99_us"])
    worst_worker = max(fairness, key=lambda item: item[2])
    errors = sum(sample["flags"] & 1 != 0 for sample in [*pooled_s, *pooled_t])
    unresolved = sum(sample["flags"] & 2 != 0 for sample in [*pooled_s, *pooled_t])
    comparisons = {
        "s_pooled_search": pooled["S"]["search_p99_us"] <= 3_000,
        "s_pooled_total": pooled["S"]["total_p99_us"] <= 5_000,
        "s_runs_search": all(
            routes[f"S{index}"]["search_p99_us"] <= 3_000 for index in range(1, 6)
        ),
        "s_runs_total": all(
            routes[f"S{index}"]["total_p99_us"] <= 5_000 for index in range(1, 6)
        ),
        "t_pooled_total": pooled["T"]["total_p99_us"] <= 5_000,
        "t_runs_total": all(
            routes[f"T{index}"]["total_p99_us"] <= 5_000 for index in range(1, 6)
        ),
        "fairness": worst_worker[2] <= 5_000,
        "errors": errors == 0,
        "unresolved": unresolved == 0,
    }
    return {
        "thresholds_would_pass": all(comparisons.values()),
        "threshold_comparisons": comparisons,
        "thresholds_us": LATENCY_THRESHOLDS,
        "pooled": pooled,
        "routes": routes,
        "fairness": {
            "maximum_run": worst_worker[0],
            "maximum_worker": worst_worker[1],
            "maximum_total_p99_us": worst_worker[2],
            "minimum_total_p99_us": min(item[2] for item in fairness),
            "spread_us": worst_worker[2] - min(item[2] for item in fairness),
        },
        "query_diagnostics": {
            "per_query": per_query,
            "worst_query_ordinal": int(worst_query[0]),
            "worst_query_total_p99_us": worst_query[1]["total_p99_us"],
        },
        "excess_us": {
            "single_search": max(0, pooled["S"]["search_p99_us"] - 3_000),
            "single_total": max(0, pooled["S"]["total_p99_us"] - 5_000),
            "twenty_total": max(0, pooled["T"]["total_p99_us"] - 5_000),
            "fairness": max(0, worst_worker[2] - 5_000),
        },
        "sample_counts": {"S": len(pooled_s), "T": len(pooled_t)},
        "errors": errors,
        "unresolved": unresolved,
    }


def remote_run() -> None:
    require(REMOTE_EXECUTION, "remote run is not a local action")
    require(isinstance(CONTROLLER_SOURCE_BYTES, bytes), "controller source bytes are absent")
    clean = load_clean_controller(CLEAN_CONTROLLER_REMOTE, EXPECTED["clean_controller_remote"])
    subject = verify_remote_subject(clean)
    clean_before = clean_marker_snapshot()
    initialize_remote_state(subject, clean_before)
    failure: BaseException | None = None
    completed: list[str] = []
    result: dict[str, Any] = {
        "schema": "lay.v10.c1-dirty-direct-latency.v1",
        "task_id": TASK_ID,
        "verdict": "DIRTY_LOAD_OBSERVATION_INVALID",
        "started_at": now(),
        "claim": "LOADED_PRODUCT_DIAGNOSTIC_ONLY",
    }
    with dirty_route_lock():
        try:
            for route in ORDER:
                run_route(clean, route, clean_before)
                completed.append(route)
            result.update(aggregate_matrix(clean))
            result["verdict"] = "DIRTY_LOAD_OBSERVATION"
        except BaseException as error:
            failure = error
            result["error"] = str(error)
        clean_after = clean_marker_snapshot()
        result.update(
            {
                "completed_at": now(),
                "completed_routes": completed,
                "fixed_order": ORDER,
                "clean_state_before": clean_before,
                "clean_state_after": clean_after,
                "clean_state_unchanged": clean_before == clean_after,
                "clean_c1_verdict": "NOT_MEASURED_BLOCKED_ENVIRONMENT",
                "clean_markers_consumed": False,
                "clean_final_published": False,
                "environment_intentionally_loaded": True,
                "quiet_admission_executed": False,
                "foreign_process_control": False,
                "host_tuning": False,
                "build_executed": False,
                "parity_rerun": False,
                "perf_invoked": False,
                "pmu_event_opened": False,
                "formal_b_pass": False,
                "v12_admitted": False,
                "runtime_authority_changed": False,
                "installed_lay_changed": False,
                "retry_permitted": False,
            }
        )
        write_new_json(REMOTE_WORK / "DIRTY_LOAD_RESULT.json", result)
        write_sha256sums(REMOTE_WORK)
        seal_tree(REMOTE_WORK)
        atomic_publish(REMOTE_WORK, REMOTE_FINAL)
        write_sha256sums(REMOTE_STATE)
        seal_tree(REMOTE_STATE)
    print(
        json.dumps(
            {
                "verdict": result["verdict"],
                "thresholds_would_pass": result.get("thresholds_would_pass"),
                "completed_routes": completed,
                "remote_final": str(REMOTE_FINAL),
                "error": result.get("error"),
            },
            sort_keys=True,
        )
    )
    if failure is not None:
        raise SystemExit(1)


def remote_audit() -> None:
    require(REMOTE_EXECUTION, "remote audit is not a local action")
    require(not REMOTE_FINAL.exists(), "dirty direct final already exists")
    require(not REMOTE_WORK.exists(), "dirty direct work path already exists")
    require(not REMOTE_STATE.exists(), "dirty direct state already exists")
    clean = load_clean_controller(CLEAN_CONTROLLER_REMOTE, EXPECTED["clean_controller_remote"])
    subject = verify_remote_subject(clean)
    print(
        json.dumps(
            {
                "verdict": "READY_FOR_DIRTY_S1",
                "subject": subject,
                "clean_state": clean_marker_snapshot(),
                "remote_writes": False,
                "subject_executions": 0,
                "quiet_admission_executed": False,
            },
            sort_keys=True,
        )
    )


def remote_verify() -> None:
    require(REMOTE_EXECUTION, "remote verify is not a local action")
    count = verify_sha256sums(REMOTE_FINAL)
    state_count = verify_sha256sums(REMOTE_STATE)
    clean = clean_marker_snapshot()
    writable = [
        str(path)
        for root in (REMOTE_FINAL, REMOTE_STATE)
        for path in [root, *root.rglob("*")]
        if stat.S_IMODE(path.stat().st_mode) & 0o222
    ]
    require(not writable, f"writable published objects: {writable}")
    print(
        json.dumps(
            {
                "verdict": "PASS",
                "remote_manifest_entries": count,
                "state_manifest_entries": state_count,
                "writable_objects": 0,
                "clean_state": clean,
            },
            sort_keys=True,
        )
    )


REMOTE_BOOTSTRAP = (
    "import hashlib,sys\n"
    "source=sys.stdin.buffer.read()\n"
    "assert hashlib.sha256(source).hexdigest()==sys.argv[1], 'controller SHA mismatch'\n"
    "action=sys.argv[2]\n"
    "sys.argv=['lay-v10-c1-dirty-direct-latency.py',action]\n"
    "ns={'__name__':'__main__','__file__':'<lay-v10-c1-dirty-direct-latency.py>',"
    "'REMOTE_EXECUTION':True,'CONTROLLER_SOURCE_BYTES':source}\n"
    "exec(compile(source,ns['__file__'],'exec'),ns)\n"
)


def remote_call(action: str, timeout: int) -> subprocess.CompletedProcess[bytes]:
    source = pathlib.Path(__file__).read_bytes()
    command = shlex.join(
        ["/usr/bin/python3", "-c", REMOTE_BOOTSTRAP, sha256_bytes(source), action]
    )
    return subprocess.run(
        [
            "/usr/bin/ssh",
            "-i",
            str(pathlib.Path.home() / ".ssh/mega-mini-admin"),
            "-o",
            "BatchMode=yes",
            "-o",
            "ConnectTimeout=8",
            REMOTE,
            command,
        ],
        input=source,
        stdout=subprocess.PIPE,
        stderr=None,
        timeout=timeout,
        check=False,
    )


def verify_local_preflight() -> dict[str, Any]:
    require_file(PREFLIGHT_MANIFEST, digest=EXPECTED["preflight_manifest_file"])
    require_file(PREFLIGHT_RECEIPT, digest=EXPECTED["preflight_receipt"])
    require_file(CONTRACT, digest=EXPECTED["contract"])
    require_file(ROUTE_RECEIPT, digest=EXPECTED["route_receipt"])
    require_file(CLEAN_CONTROLLER_LOCAL, digest=EXPECTED["clean_controller_local"])
    require_file(ACTIVE_V11, digest=EXPECTED["active_v11"])
    receipt = json.loads(PREFLIGHT_RECEIPT.read_text())
    require(receipt.get("verdict") == "READY_TO_IMPLEMENT", "preflight did not pass")
    require(receipt.get("safe_to_implement") is True, "preflight is not safe to implement")
    require(
        receipt.get("manifest_sha256") == EXPECTED["preflight_manifest_identity"],
        "preflight manifest identity mismatch",
    )
    route = json.loads(ROUTE_RECEIPT.read_text())
    require(route.get("verdict") == "PASS", "route review did not pass")
    require(route.get("agent_decision", {}).get("safe_to_edit") is True, "route is not safe")
    return receipt


def run_local(command: Sequence[str]) -> subprocess.CompletedProcess[bytes]:
    return subprocess.run(list(command), stdout=subprocess.PIPE, stderr=subprocess.PIPE, check=False)


def local_runtime_snapshot() -> dict[str, Any]:
    processes = run_local(
        ["/usr/bin/pgrep", "-a", "-f", "^(ibus-daemon|lay-daemon|lay-ibus-engine)( |$)"]
    )
    require(processes.returncode in (0, 1), "cannot inspect local Lay processes")
    version = run_local([str(pathlib.Path.home() / ".local/bin/lay"), "--version"])
    return {
        "active_v11_sha256": sha256_file(ACTIVE_V11),
        "lay_version_stdout": version.stdout.decode(errors="replace").strip(),
        "lay_version_stderr": version.stderr.decode(errors="replace").strip(),
        "lay_processes": processes.stdout.decode(errors="replace").splitlines(),
    }


def fetch_remote_result() -> bytes:
    command = shlex.join(
        ["/usr/bin/cat", str(REMOTE_FINAL / "DIRTY_LOAD_RESULT.json")]
    )
    result = subprocess.run(
        [
            "/usr/bin/ssh",
            "-i",
            str(pathlib.Path.home() / ".ssh/mega-mini-admin"),
            "-o",
            "BatchMode=yes",
            REMOTE,
            command,
        ],
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        check=False,
    )
    require(result.returncode == 0, f"cannot fetch remote result: {result.stderr[-2000:]!r}")
    return result.stdout


def publish_local_index(
    remote_bytes: bytes,
    verification: dict[str, Any],
    runtime_before: dict[str, Any],
    runtime_after: dict[str, Any],
) -> None:
    require(not LOCAL_FINAL.exists(), "local dirty direct result already exists")
    require(runtime_before == runtime_after, "installed Lay runtime changed during observation")
    stage = LOCAL_FINAL.with_name(f".{LOCAL_FINAL.name}.stage-{os.getpid()}-{time.time_ns()}")
    stage.mkdir(parents=True, mode=0o700)
    write_new_bytes(stage / "REMOTE_RESULT.json", remote_bytes)
    write_new_json(
        stage / "LOCAL_INDEX.json",
        {
            "schema": "lay.v10.c1-dirty-direct-local-index.v1",
            "task_id": TASK_ID,
            "recorded_at": now(),
            "controller_sha256": sha256_file(pathlib.Path(__file__)),
            "remote_result_sha256": sha256_bytes(remote_bytes),
            "remote_verification": verification,
            "preflight_manifest_sha256": EXPECTED["preflight_manifest_file"],
            "preflight_receipt_sha256": EXPECTED["preflight_receipt"],
            "runtime_before": runtime_before,
            "runtime_after": runtime_after,
            "clean_c1_verdict": "NOT_MEASURED_BLOCKED_ENVIRONMENT",
            "runtime_authority_changed": False,
            "installed_lay_changed": False,
        },
    )
    write_sha256sums(stage)
    seal_tree(stage)
    atomic_publish(stage, LOCAL_FINAL)


def self_check() -> None:
    verify_local_preflight()
    clean = load_clean_controller(CLEAN_CONTROLLER_LOCAL, EXPECTED["clean_controller_local"])
    require(ORDER == clean.ORDER, "route order drift")
    require(SAMPLES_PER_RUN == clean.SAMPLES_PER_RUN, "sample denominator drift")
    require(LATENCY_THRESHOLDS == clean.LATENCY_THRESHOLDS, "threshold drift")
    require(clean.nearest_rank(range(1, 501), 99) == 495, "nearest-rank drift")
    source = pathlib.Path(__file__).read_text()
    forbidden = [
        "/usr/bin/" + "perf",
        "perf_event_" + "open",
        "cargo" + " build",
        "cargo" + " test",
        "cargo" + " check",
        "system" + "ctl",
        "pk" + "ill",
        "kill" + "all",
        "cpu" + "power",
    ]
    require(not any(token in source for token in forbidden), "forbidden command token")
    with tempfile.TemporaryDirectory(prefix="lay-dirty-direct-self-check-") as directory:
        root = pathlib.Path(directory)
        marker_root = root / "markers"
        marker_root.mkdir()
        write_new_bytes(marker_root / "S1.available", b"available\n")
        os.rename(marker_root / "S1.available", marker_root / "S1.consumed-before-exec")
        require(not (marker_root / "S1.available").exists(), "marker was not consumed")
        stage = root / "stage"
        stage.mkdir()
        write_new_json(stage / "value.json", {"pass": True})
        write_sha256sums(stage)
        seal_tree(stage)
        verify_sha256sums(stage)
    print(
        json.dumps(
            {
                "verdict": "PASS",
                "checks": 24,
                "remote_actions": 0,
                "subject_executions": 0,
                "cargo_invocations": 0,
                "parity_reruns": 0,
                "perf_invocations": 0,
                "clean_marker_consumptions": 0,
            },
            sort_keys=True,
        )
    )


def local_audit() -> None:
    verify_local_preflight()
    result = remote_call("remote-audit", timeout=120)
    require(result.returncode == 0, f"remote audit failed: {result.stdout[-4000:]!r}")
    value = json.loads(result.stdout)
    require(value.get("verdict") == "READY_FOR_DIRTY_S1", "remote audit did not pass")
    print(json.dumps(value, indent=2, sort_keys=True))


def local_run() -> None:
    verify_local_preflight()
    require(not LOCAL_FINAL.exists(), "local dirty direct result already exists")
    runtime_before = local_runtime_snapshot()
    result = remote_call("remote-run", timeout=10_800)
    require(result.returncode == 0, f"remote dirty direct run failed: {result.stdout[-4000:]!r}")
    verification_result = remote_call("remote-verify", timeout=120)
    require(
        verification_result.returncode == 0,
        f"remote dirty direct verification failed: {verification_result.stdout[-4000:]!r}",
    )
    verification = json.loads(verification_result.stdout)
    require(verification.get("verdict") == "PASS", "remote verification did not pass")
    remote_bytes = fetch_remote_result()
    remote_value = json.loads(remote_bytes)
    require(remote_value.get("verdict") == "DIRTY_LOAD_OBSERVATION", "dirty observation invalid")
    require(remote_value.get("clean_state_unchanged") is True, "clean state changed")
    runtime_after = local_runtime_snapshot()
    publish_local_index(remote_bytes, verification, runtime_before, runtime_after)
    print(
        json.dumps(
            {
                "verdict": remote_value["verdict"],
                "thresholds_would_pass": remote_value["thresholds_would_pass"],
                "pooled": remote_value["pooled"],
                "fairness": remote_value["fairness"],
                "excess_us": remote_value["excess_us"],
                "local_index": str(LOCAL_FINAL),
            },
            indent=2,
            sort_keys=True,
        )
    )


def remote_dispatch(action: str) -> None:
    if action == "remote-run":
        remote_run()
    elif action == "remote-audit":
        remote_audit()
    elif action == "remote-verify":
        remote_verify()
    else:
        raise DirtyDirectError(f"unknown remote action: {action}")


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument(
        "action",
        choices=["self-check", "audit", "run", "remote-audit", "remote-run", "remote-verify"],
    )
    arguments = parser.parse_args()
    if arguments.action.startswith("remote-"):
        remote_dispatch(arguments.action)
    elif arguments.action == "self-check":
        self_check()
    elif arguments.action == "audit":
        local_audit()
    else:
        local_run()


if __name__ == "__main__":
    main()
