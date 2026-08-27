#!/usr/bin/env python3
"""Prepare and run one clean B5/B6 comparison over the sealed V10 proxy."""

from __future__ import annotations

import argparse
import datetime as dt
import importlib.util
import json
import os
import pathlib
import subprocess
import sys
import time
from typing import Any


TASK_ID = "slice8b-v10-clean-speed-v2-20260825"
REMOTE = "e@192.168.3.94"
PROJECT_ROOT = pathlib.Path(__file__).resolve().parents[1]
BASE_PATH = PROJECT_ROOT / "scripts/lay-v10-hardware-dirty.py"
BASE_SHA256 = "f0aa4b55a803ef204d1d898cec70eb282757009a71ef5d9310417b2ac17a45e8"
PREFLIGHT_RECEIPT = PROJECT_ROOT / (
    "docs/structural_gates/receipts/"
    "LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_CLEAN_SPEED_PREFLIGHT_V2_2026-08-25.json"
)
PREFLIGHT_MANIFEST_SHA256 = "9f90f17794e99712e1ad137a282d0a2da78f08cf8a67bdd39cdefe3119768bd0"
LOCAL_RESULT = PROJECT_ROOT / (
    "docs/structural_gates/receipts/"
    "LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_CLEAN_SPEED_V2_2026-08-25"
)
PREPARATION_RECEIPT = PROJECT_ROOT / (
    "docs/structural_gates/receipts/"
    "LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_CLEAN_SPEED_PREPARED_V2_2026-08-25.json"
)
REMOTE_FINAL_TEXT = f"/home/e/.local/share/lay/provenance/{TASK_ID}"
REMOTE_STATE_TEXT = f"/home/e/.local/state/lay/{TASK_ID}"
DIRTY_RECEIPT = PROJECT_ROOT / (
    "docs/structural_gates/receipts/"
    "LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_DIRTY_SPEED_V1_2026-08-25/REMOTE_RECEIPT.json"
)
DIRTY_RECEIPT_SHA256 = "9255310624b55d370db55800f88eb6a04172d555235aecc6587d0ac15826af48"
THRESHOLDS = {
    "cpu_psi_some_avg10": 2.0,
    "memory_psi_full_avg10": 0.10,
    "io_psi_full_avg10": 0.10,
    "procs_running": 2,
    "temperature_c": 90.0,
    "busy_process_cpu_percent": 1.0,
}
class CleanError(RuntimeError):
    pass


def require(condition: bool, message: str) -> None:
    if not condition:
        raise CleanError(message)


def now() -> str:
    return dt.datetime.now(dt.timezone.utc).isoformat()


def load_local_base() -> Any:
    specification = importlib.util.spec_from_file_location("lay_v10_dirty_base", BASE_PATH)
    require(specification is not None and specification.loader is not None, "cannot load V1 controller")
    module = importlib.util.module_from_spec(specification)
    specification.loader.exec_module(module)
    require(module.sha256_file(BASE_PATH) == BASE_SHA256, "V1 controller identity mismatch")
    return module


def load_remote_base() -> dict[str, Any]:
    source = globals().get("BASE_SOURCE")
    require(isinstance(source, str), "remote V1 controller source is absent")
    namespace: dict[str, Any] = {
        "__name__": "lay_v10_dirty_base",
        "__file__": "<lay-v10-hardware-dirty.py>",
    }
    exec(compile(source, "lay-v10-hardware-dirty.py", "exec"), namespace)
    require(namespace["sha256_bytes"](source.encode()) == BASE_SHA256, "remote V1 controller identity mismatch")
    namespace["TASK_ID"] = TASK_ID
    namespace["REMOTE_FINAL"] = pathlib.Path(REMOTE_FINAL_TEXT)
    namespace["REMOTE_STATE"] = pathlib.Path(REMOTE_STATE_TEXT)
    return namespace


def all_processes(base: dict[str, Any]) -> dict[int, dict[str, Any]]:
    records = {}
    for path in pathlib.Path("/proc").iterdir():
        if not path.name.isdigit():
            continue
        record = base["process_record"](path)
        if record is not None:
            records[record["pid"]] = record
    return records


def quiet_sample(base: dict[str, Any]) -> dict[str, Any]:
    before = all_processes(base)
    started = time.monotonic()
    time.sleep(0.25)
    elapsed = time.monotonic() - started
    after = all_processes(base)
    busy = []
    for pid in sorted(before.keys() & after.keys()):
        if pid == os.getpid() or not after[pid]["cmdline"]:
            continue
        cpu_percent = (after[pid]["cpu_seconds"] - before[pid]["cpu_seconds"]) / elapsed * 100.0
        if cpu_percent > THRESHOLDS["busy_process_cpu_percent"]:
            busy.append(
                {
                    "pid": pid,
                    "comm": after[pid]["comm"],
                    "cmdline": after[pid]["cmdline"],
                    "cpu_percent": cpu_percent,
                }
            )
    sample = base["light_sample"]()
    sample["busy_processes"] = busy
    return sample


def sample_failures(sample: dict[str, Any]) -> list[str]:
    failures = []
    cpu = sample["cpu_pressure"].get("some", {}).get("avg10", float("inf"))
    memory = sample["memory_pressure"].get("full", {}).get("avg10", float("inf"))
    io = sample["io_pressure"].get("full", {}).get("avg10", float("inf"))
    if cpu > THRESHOLDS["cpu_psi_some_avg10"]:
        failures.append(f"cpu_psi:{cpu:.2f}")
    if memory > THRESHOLDS["memory_psi_full_avg10"]:
        failures.append(f"memory_psi:{memory:.2f}")
    if io > THRESHOLDS["io_psi_full_avg10"]:
        failures.append(f"io_psi:{io:.2f}")
    if sample["procs_running"] is None or sample["procs_running"] > THRESHOLDS["procs_running"]:
        failures.append(f"procs_running:{sample['procs_running']}")
    temperature = sample["maximum_temperature_c"]
    if temperature <= 0.0 or temperature >= THRESHOLDS["temperature_c"]:
        failures.append(f"temperature:{temperature:.1f}")
    failures.extend(
        f"busy:{value['pid']}:{value['comm']}:{value['cpu_percent']:.1f}"
        for value in sample["busy_processes"]
    )
    return failures


def readiness(base: dict[str, Any]) -> dict[str, Any]:
    inputs = base["verify_remote_inputs"]()
    require(base["read_text"](pathlib.Path("/sys/devices/system/cpu/online")) == "0-19", "CPU map drift")
    stable_before = base["stable_host_projection"]()
    throttles_before = base["throttle_counters"]()
    samples = [quiet_sample(base) for _ in range(3)]
    stable_after = base["stable_host_projection"]()
    throttles_after = base["throttle_counters"]()
    failures = sorted({failure for sample in samples for failure in sample_failures(sample)})
    if stable_before != stable_after:
        failures.append("stable_host_projection_drift")
    if throttles_before != throttles_after:
        failures.append("thermal_throttle_counter_drift")
    final = pathlib.Path(REMOTE_FINAL_TEXT)
    state = pathlib.Path(REMOTE_STATE_TEXT)
    return {
        "schema": "lay.v10.clean-speed-readiness.v2",
        "task_id": TASK_ID,
        "observed_at": now(),
        "ready": not failures,
        "verdict": "READY_FOR_ONE_CLEAN_RUN" if not failures else "BLOCKED_ENVIRONMENT",
        "failures": sorted(set(failures)),
        "samples": samples,
        "thresholds": THRESHOLDS,
        "stable_host_unchanged": stable_before == stable_after,
        "throttle_counters_unchanged": throttles_before == throttles_after,
        "sealed_inputs": inputs,
        "measurement_attempt_consumed": state.exists(),
        "final_output_exists": final.exists(),
        "remote_writes_performed": False,
        "formal_b_pass": False,
        "per_query_p99_observed": False,
        "v12_admitted": False,
    }


def normalize_route(base: dict[str, Any], stage: pathlib.Path, route: str, value: dict[str, Any]) -> dict[str, Any]:
    old_path = stage / route.lower() / "DIRTY_ROUTE_RECEIPT.json"
    require(old_path.is_file(), f"{route} V1 route receipt is absent")
    old_path.unlink()
    clean = dict(value)
    clean.update(
        {
            "schema": "lay.v10.clean-speed-route.v2",
            "verdict": "CLEAN_DIAGNOSTIC_OBSERVED",
            "environment_admitted_before_run": True,
            "formal_b_pass": False,
            "per_query_p99_observed": False,
        }
    )
    base["write_new_json"](stage / route.lower() / "CLEAN_ROUTE_RECEIPT.json", clean)
    return clean


def clean_run(base: dict[str, Any]) -> None:
    final = pathlib.Path(REMOTE_FINAL_TEXT)
    state = pathlib.Path(REMOTE_STATE_TEXT)
    require(not final.exists(), f"clean result already exists: {final}")
    require(not state.exists(), f"clean attempt already consumed: {state}")
    admitted = readiness(base)
    if not admitted["ready"]:
        print(json.dumps(admitted, sort_keys=True))
        raise SystemExit(3)
    stage = base["REMOTE_PROVENANCE"] / f".{TASK_ID}.stage-{os.getpid()}-{time.time_ns()}"
    stage.mkdir(mode=0o700)
    state.mkdir(parents=True, mode=0o700)
    combined_marker = state / "measurement-consumed"
    base["write_new_json"](
        combined_marker,
        {
            "task_id": TASK_ID,
            "consumed_at": now(),
            "admission_verdict": admitted["verdict"],
            "routes": ["B5", "B6"],
            "retry_permitted": False,
        },
    )
    base["consume_marker"] = lambda _route: combined_marker
    result: dict[str, Any] = {
        "schema": "lay.v10.clean-speed.v2",
        "task_id": TASK_ID,
        "started_at": now(),
        "verdict": "ERROR",
        "claim": "CLEAN_AGGREGATE_DIAGNOSTIC_ONLY",
        "admission": admitted,
        "formal_b_pass": False,
        "per_query_p99_observed": False,
        "b1_v7_unchanged": True,
        "b2_executed": False,
        "b3_executed": False,
        "proxy_semantic_parity_executed": False,
        "quality": "UNKNOWN",
        "v12_admitted": False,
        "perf_invoked": False,
        "pmu_event_opened": False,
        "cargo_invoked": False,
        "host_policy_modified": False,
        "foreign_process_stopped": False,
        "runtime_authority_changed": False,
        "installed_lay_changed": False,
    }
    failure: Exception | None = None
    try:
        stable_before = base["stable_host_projection"]()
        throttles_before = base["throttle_counters"]()
        b5 = normalize_route(base, stage, "B5", base["run_route"]("B5", stage))
        b6 = normalize_route(base, stage, "B6", base["run_route"]("B6", stage))
        stable_after = base["stable_host_projection"]()
        throttles_after = base["throttle_counters"]()
        require(stable_before == stable_after, "stable host projection changed during clean run")
        require(throttles_before == throttles_after, "thermal throttle counters changed during clean run")
        dirty = json.loads((base["REMOTE_PROVENANCE"] / "slice8b-v10-dirty-speed-v1-20260825/DIRTY_SPEED_RECEIPT.json").read_text())
        require(base["sha256_file"](base["REMOTE_PROVENANCE"] / "slice8b-v10-dirty-speed-v1-20260825/DIRTY_SPEED_RECEIPT.json") == DIRTY_RECEIPT_SHA256, "dirty comparison receipt drift")
        result["routes"] = {"B5": b5, "B6": b6}
        result["dirty_comparison"] = {
            route: {
                "dirty_wall_window_ms": dirty["routes"][route]["wall_window_ms"],
                "clean_wall_window_ms": result["routes"][route]["wall_window_ms"],
                "clean_over_dirty_ratio": result["routes"][route]["wall_window_ms"]
                / dirty["routes"][route]["wall_window_ms"],
            }
            for route in ("B5", "B6")
        }
        result["stable_host_unchanged"] = True
        result["throttle_counters_unchanged"] = True
        result["verdict"] = "CLEAN_SPEED_OBSERVED_NO_FORMAL_PROMOTION"
    except Exception as error:
        failure = error
        result["error"] = str(error)
    result["completed_at"] = now()
    base["write_new_json"](stage / "CLEAN_SPEED_RECEIPT.json", result)
    base["publish"](stage, final)
    print(json.dumps({"verdict": result["verdict"], "receipt": str(final / "CLEAN_SPEED_RECEIPT.json"), "error": result.get("error")}, sort_keys=True))
    if failure is not None:
        raise SystemExit(1)


def remote_main(action: str) -> None:
    base = load_remote_base()
    if action == "ready":
        print(json.dumps(readiness(base), sort_keys=True))
    elif action == "run":
        clean_run(base)
    else:
        raise CleanError(f"unknown remote action: {action}")


def remote_call(action: str) -> subprocess.CompletedProcess[bytes]:
    clean_source = pathlib.Path(__file__).read_text()
    base_source = BASE_PATH.read_text()
    payload = (
        "import sys\n"
        f"base_source = {base_source!r}\n"
        f"clean_source = {clean_source!r}\n"
        "namespace = {'__name__': 'lay_clean_remote', "
        "'__file__': '<lay-clean-remote>', 'BASE_SOURCE': base_source}\n"
        "exec(compile(clean_source, 'lay-v10-hardware-clean.py', 'exec'), namespace)\n"
        "namespace['remote_main'](sys.argv[1])\n"
    ).encode()
    return subprocess.run(
        ["/usr/bin/ssh", "-o", "BatchMode=yes", REMOTE, "/usr/bin/python3", "-", action],
        input=payload,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        check=False,
    )


def local_ready() -> dict[str, Any]:
    result = remote_call("ready")
    require(result.returncode == 0, f"readiness transport failed: {result.stderr[-2000:]!r}")
    value = json.loads(result.stdout)
    print(json.dumps(value, indent=2, sort_keys=True))
    return value


def local_prepare() -> None:
    base = load_local_base()
    require(not PREPARATION_RECEIPT.exists(), f"preparation receipt already exists: {PREPARATION_RECEIPT}")
    preflight = json.loads(PREFLIGHT_RECEIPT.read_text())
    require(preflight.get("verdict") == "READY_TO_IMPLEMENT", "clean V2 preflight did not pass")
    result = remote_call("ready")
    require(result.returncode == 0, f"readiness transport failed: {result.stderr[-2000:]!r}")
    readiness_value = json.loads(result.stdout)
    require(readiness_value.get("measurement_attempt_consumed") is False, "measurement attempt is already consumed")
    require(readiness_value.get("final_output_exists") is False, "clean output already exists")
    base.write_new_json(
        PREPARATION_RECEIPT,
        {
            "schema": "lay.v10.clean-speed-preparation.v2",
            "task_id": TASK_ID,
            "prepared_at": now(),
            "state": "CLEAN_TEST_PREPARED_UNRUN",
            "controller": {
                "path": str(pathlib.Path(__file__)),
                "sha256": base.sha256_file(pathlib.Path(__file__)),
            },
            "base_controller_sha256": BASE_SHA256,
            "preflight_manifest_sha256": PREFLIGHT_MANIFEST_SHA256,
            "preflight_verdict": preflight["verdict"],
            "readiness_at_preparation": readiness_value,
            "commands": {
                "refresh_readiness": "scripts/lay-v10-hardware-clean.py ready",
                "run_once_when_ready": "scripts/lay-v10-hardware-clean.py run",
            },
            "measurement_attempt_consumed": False,
            "remote_output_exists": False,
            "cargo_invoked": False,
            "perf_invoked": False,
            "subject_executed": False,
            "formal_b_pass": False,
            "per_query_p99_observed": False,
            "v12_admitted": False,
            "runtime_authority_changed": False,
            "installed_lay_changed": False,
        },
    )
    print(
        json.dumps(
            {
                "state": "CLEAN_TEST_PREPARED_UNRUN",
                "ready_now": readiness_value["ready"],
                "failures": readiness_value["failures"],
                "measurement_attempt_consumed": False,
                "receipt": str(PREPARATION_RECEIPT),
            },
            indent=2,
            sort_keys=True,
        )
    )


def local_run() -> None:
    base = load_local_base()
    preflight = json.loads(PREFLIGHT_RECEIPT.read_text())
    require(preflight.get("verdict") == "READY_TO_IMPLEMENT", "clean V2 preflight did not pass")
    require(preflight.get("manifest_sha256") == PREFLIGHT_MANIFEST_SHA256, "clean V2 preflight identity mismatch")
    require(not LOCAL_RESULT.exists(), f"local clean result already exists: {LOCAL_RESULT}")
    local_before = base.local_runtime_snapshot()
    result = remote_call("run")
    if result.returncode == 3:
        blocked = json.loads(result.stdout)
        print(json.dumps(blocked, indent=2, sort_keys=True))
        raise SystemExit(3)
    remote_receipt = subprocess.run(
        ["/usr/bin/ssh", "-o", "BatchMode=yes", REMOTE, "/usr/bin/cat", f"{REMOTE_FINAL_TEXT}/CLEAN_SPEED_RECEIPT.json"],
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        check=False,
    )
    require(remote_receipt.returncode == 0, f"clean receipt unavailable: {remote_receipt.stderr[-2000:]!r}")
    receipt = json.loads(remote_receipt.stdout)
    local_after = base.local_runtime_snapshot()
    require(local_before == local_after, "local Lay runtime changed during clean run")
    require(result.returncode == 0 and receipt.get("verdict") == "CLEAN_SPEED_OBSERVED_NO_FORMAL_PROMOTION", "clean run failed")
    stage = LOCAL_RESULT.with_name(f".{LOCAL_RESULT.name}.stage-{os.getpid()}-{time.time_ns()}")
    stage.mkdir(parents=True, mode=0o700)
    base.write_new_bytes(stage / "REMOTE_RECEIPT.json", remote_receipt.stdout)
    base.write_new_json(
        stage / "LOCAL_RUN_RECEIPT.json",
        {
            "schema": "lay.v10.clean-speed-local-index.v2",
            "task_id": TASK_ID,
            "recorded_at": now(),
            "controller_sha256": base.sha256_file(pathlib.Path(__file__)),
            "preflight_manifest_sha256": PREFLIGHT_MANIFEST_SHA256,
            "remote_receipt_sha256": base.sha256_bytes(remote_receipt.stdout),
            "local_runtime_before": local_before,
            "local_runtime_after": local_after,
            "formal_b_pass": False,
            "runtime_authority_changed": False,
            "installed_lay_changed": False,
        },
    )
    base.publish(stage, LOCAL_RESULT)
    print(json.dumps({"verdict": receipt["verdict"], "dirty_comparison": receipt["dirty_comparison"]}, indent=2, sort_keys=True))


def self_check() -> None:
    base = load_local_base()
    require(base.sha256_file(DIRTY_RECEIPT) == DIRTY_RECEIPT_SHA256, "dirty receipt identity mismatch")
    require(base.CPU_LISTS == {"B5": [0], "B6": list(range(20))}, "CPU route drift")
    require(base.REQUESTS == 382, "request denominator drift")
    quiet = {
        "cpu_pressure": {"some": {"avg10": 0.5}},
        "memory_pressure": {"full": {"avg10": 0.0}},
        "io_pressure": {"full": {"avg10": 0.0}},
        "procs_running": 1,
        "maximum_temperature_c": 50.0,
        "busy_processes": [],
    }
    require(sample_failures(quiet) == [], "quiet admission model rejected a valid sample")
    busy = dict(quiet)
    busy["cpu_pressure"] = {"some": {"avg10": 3.0}}
    require(any(value.startswith("cpu_psi") for value in sample_failures(busy)), "busy admission model accepted CPU pressure")
    source = pathlib.Path(__file__).read_text()
    forbidden = ["/usr/bin/" + "perf", "cargo" + " build", "system" + "ctl", "cpu" + "power"]
    require(not any(token in source for token in forbidden), "forbidden command token in clean controller")
    print(json.dumps({"verdict": "PASS", "checks": 7, "remote_execution": False}, sort_keys=True))


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("action", choices=("self-check", "prepare", "ready", "run"))
    args = parser.parse_args()
    if args.action == "self-check":
        self_check()
    elif args.action == "prepare":
        local_prepare()
    elif args.action == "ready":
        local_ready()
    else:
        local_run()


if __name__ == "__main__":
    main()
