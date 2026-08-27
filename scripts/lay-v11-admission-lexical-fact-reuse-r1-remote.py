#!/usr/bin/env python3
"""No-build executable-projection repair for the V11 paired comparison."""

from __future__ import annotations

import argparse
import contextlib
import fcntl
import importlib.util
import json
import os
import pathlib
import shutil
import subprocess
import sys
import tempfile
import time
from collections.abc import Mapping
from typing import Any


LOCAL_BASE = pathlib.Path(__file__).with_name("lay-v11-admission-lexical-fact-reuse-remote.py")
DEPLOYED_BASE = pathlib.Path(__file__).with_name("v11-base.py")
BASE_PATH = DEPLOYED_BASE if DEPLOYED_BASE.is_file() else LOCAL_BASE
SPEC = importlib.util.spec_from_file_location("lay_v11_base", BASE_PATH)
if SPEC is None or SPEC.loader is None:
    raise RuntimeError(f"cannot load V11 base controller: {BASE_PATH}")
base = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(base)


TASK_ID = "slice8b-v11-admission-lexical-fact-reuse-paired-r1-v1-20260827"
TRANSACTION_ID = "a6f54d6a2cc8295c139e65a09aa4fdd070a3b913e8b367d31afc7010615cae4d"
PARENT = pathlib.Path("/home/e/.local/share/lay/provenance") / TASK_ID
STATE = pathlib.Path("/home/e/.local/state/lay") / TASK_ID
TERMINAL = PARENT / "terminal-v1"
FAILED_PARENT = pathlib.Path("/home/e/.local/share/lay/provenance/slice8b-v11-admission-lexical-fact-reuse-paired-v3-20260827")
FAILED_STATE = pathlib.Path("/home/e/.local/state/lay/slice8b-v11-admission-lexical-fact-reuse-paired-v3-20260827")
FAILED_ELF = FAILED_PARENT / "build-v1/m3-v11-test-elf"
EXECUTABLE = PARENT / "bootstrap/m3-v11-test-elf"

ACTIONS = ("self-check", "status", "bootstrap", "run-once", "terminal")
ROUTES = ("B0R", "B1R")
MARKERS = {"B0R": "b0r", "B1R": "b1r"}
MODES = {"B0R": "UNCACHED", "B1R": "REUSE"}
PREFLIGHT_MANIFEST_SHA256 = "a02baf5e09ae8e9802add6cf300581448c14021807f1ba889408eaab8315cf56"
PREFLIGHT_RECEIPT_SHA256 = "b8fdc7c20dd772be0494228a98a54cdb8966c8e35ee8e4998ad0d994ff022071"
FAILURE_RECEIPT_SHA256 = "b1f2fc1305169e25e5801594f87ec76e660e4e54df29d7d10c7a86e8b853a506"
ELF_SIZE = 321_129_832
ELF_SHA256 = "dbd5feb315e9537b0797cb98d6f38dd66fdc6ee3e562bf9db479ed2c9f34b51a"
ELF_BUILD_ID = "533bd03d86a8381f80e3c3c9c8d1b7c0bd4cc7e7"
SOURCE_SHA256 = "e8a6a182753084659e00ccd5e20238d585d859437824609a987ea03ce6edca72"


class RepairError(RuntimeError):
    pass


def need(condition: bool, message: str) -> None:
    if not condition:
        raise RepairError(message)


def load_admission(root: pathlib.Path) -> dict[str, Any]:
    value = base.load_json(root / "ADMISSION.json")
    need(value.get("schema") == "lay.v11-paired-r1-admission.v1", "admission schema drift")
    need(value.get("verdict") == "V11_R1_EXECUTION_ADMITTED", "R1 not admitted")
    need(value.get("task_id") == TASK_ID and value.get("transaction_id") == TRANSACTION_ID, "admission namespace drift")
    need(value.get("preflight_manifest_sha256") == PREFLIGHT_MANIFEST_SHA256, "preflight manifest drift")
    need(value.get("preflight_receipt_sha256") == PREFLIGHT_RECEIPT_SHA256, "preflight receipt drift")
    need(value.get("failure_receipt_sha256") == FAILURE_RECEIPT_SHA256, "failure receipt drift")
    need(value.get("elf_sha256") == ELF_SHA256 and value.get("elf_size_bytes") == ELF_SIZE, "admitted ELF drift")
    need(value.get("remote_controller_sha256") == base.sha256_file(root / "remote-controller.py"), "R1 controller SHA drift")
    need(value.get("base_controller_sha256") == base.sha256_file(root / "v11-base.py"), "base controller SHA drift")
    return value


def marker_payload(route: str, admission: Mapping[str, Any]) -> bytes:
    need(route in ROUTES, f"unknown route: {route}")
    return base.canonical({
        "schema": "lay.v11-paired-r1-marker.v1",
        "task_id": TASK_ID,
        "transaction_id": TRANSACTION_ID,
        "route": route,
        "elf_sha256": ELF_SHA256,
        "remote_controller_sha256": admission["remote_controller_sha256"],
        "one_shot": True,
        "retry_permitted": False,
    })


def marker_inventory() -> dict[str, list[str]]:
    root = STATE / "markers"
    if not root.is_dir():
        return {"available": [], "consumed": []}
    return {
        "available": sorted(path.name for path in root.glob("*.available")),
        "consumed": sorted(path.name for path in root.glob("*.consumed-before-exec")),
    }


def consume_marker(route: str, admission: Mapping[str, Any]) -> dict[str, Any]:
    root = STATE / "markers"
    stem = MARKERS[route]
    available = root / f"{stem}.available"
    consumed = root / f"{stem}.consumed-before-exec"
    expected = marker_payload(route, admission)
    before = base.require_file(available, digest=base.sha256_bytes(expected), mode="0400")
    need(available.read_bytes() == expected and not consumed.exists(), f"marker drift: {route}")
    os.rename(available, consumed)
    base.fsync_dir(root)
    after = base.require_file(consumed, size=before["size_bytes"], digest=before["sha256"], mode="0400")
    return {"before": before, "after": after, "consumed_before_execution": True}


def append_state(sequence: int, state: str, **extra: Any) -> None:
    base.write_json(
        STATE / f"STATE-{sequence:02d}-{state}.json",
        {
            "schema": "lay.v11-paired-r1-state.v1",
            "task_id": TASK_ID,
            "transaction_id": TRANSACTION_ID,
            "sequence": sequence,
            "state": state,
            "markers": marker_inventory(),
            **extra,
        },
        0o444,
    )
    base.fsync_dir(STATE)


def latest_state() -> dict[str, Any] | None:
    if not STATE.is_dir():
        return None
    rows = sorted(STATE.glob("STATE-*.json"))
    return base.load_json(rows[-1]) if rows else None


@contextlib.contextmanager
def route_lock() -> Any:
    descriptor = os.open(STATE / "route.lock", os.O_RDWR)
    try:
        fcntl.flock(descriptor, fcntl.LOCK_EX | fcntl.LOCK_NB)
        yield
    finally:
        os.close(descriptor)


def failed_projection() -> dict[str, Any]:
    build = base.load_json(FAILED_PARENT / "build-v1/BUILD_RECEIPT.json")
    b0 = base.load_json(FAILED_PARENT / "b0-v1/SUBJECT_WRAPPER.json")
    state = base.load_json(FAILED_STATE / "STATE-02-BLOCKED_SEMANTIC.json")
    base.require_file(FAILED_ELF, size=ELF_SIZE, digest=ELF_SHA256, mode="0444")
    need(build.get("verdict") == "V11_BUILD_CREATED" and build.get("elf", {}).get("build_id") == ELF_BUILD_ID, "failed build identity drift")
    need(b0.get("subject_receipt", {}).get("verdict") == "BLOCKED_PROVENANCE", "failed B0 interpretation drift")
    need(state.get("state") == "BLOCKED_SEMANTIC", "historical controller state drift")
    return {
        "build_receipt_sha256": base.sha256_file(FAILED_PARENT / "build-v1/BUILD_RECEIPT.json"),
        "b0_wrapper_sha256": base.sha256_file(FAILED_PARENT / "b0-v1/SUBJECT_WRAPPER.json"),
        "historical_state_sha256": base.sha256_file(FAILED_STATE / "STATE-02-BLOCKED_SEMANTIC.json"),
        "elf": {**base.file_row(FAILED_ELF), "build_id": ELF_BUILD_ID},
    }


def bootstrap(bundle: pathlib.Path) -> dict[str, Any]:
    need(os.geteuid() == 0, "bootstrap requires root")
    need(not PARENT.exists() and not STATE.exists(), "R1 namespace already exists")
    need(not base.active_conflicts(), "conflicting performance process active")
    admission = load_admission(bundle)
    host = base.verify_host()
    inputs = base.verify_inputs()
    failed = failed_projection()
    stage = pathlib.Path(f"{PARENT}.stage-{os.getpid()}-{time.time_ns()}")
    state_stage = pathlib.Path(f"{STATE}.stage-{os.getpid()}-{time.time_ns()}")
    stage.mkdir(parents=True, mode=0o755)
    state_stage.mkdir(parents=True, mode=0o700)
    try:
        copied = stage / "bootstrap"
        shutil.copytree(bundle, copied)
        executable = copied / "m3-v11-test-elf"
        shutil.copyfile(FAILED_ELF, executable)
        executable.chmod(0o555)
        projected = base.require_file(executable, size=ELF_SIZE, digest=ELF_SHA256, mode="0555")
        uid = base.uid_probe(stage)
        base.write_new(state_stage / "route.lock", b"v11-r1-route-lock\n", 0o600)
        markers = state_stage / "markers"
        markers.mkdir(mode=0o700)
        marker_rows = {}
        for route in ROUTES:
            path = markers / f"{MARKERS[route]}.available"
            base.write_new(path, marker_payload(route, admission), 0o400)
            marker_rows[route] = base.file_row(path)
        receipt = {
            "schema": "lay.v11-paired-r1-bootstrap.v1",
            "task_id": TASK_ID,
            "transaction_id": TRANSACTION_ID,
            "verdict": "V11_R1_MARKERS_AVAILABLE",
            "admission": admission,
            "host": host,
            "inputs": inputs,
            "failed_predecessor": failed,
            "executable_projection": {**projected, "build_id": ELF_BUILD_ID},
            "uid_capability": uid,
            "marker_rows": marker_rows,
            "markers_created": 2,
            "markers_consumed": 0,
            "cargo_invocations": 0,
            "rustc_compilations": 0,
            "subject_executions": 0,
            "runtime_authority_changed": False,
        }
        base.write_json(stage / "BOOTSTRAP_RECEIPT.json", receipt)
        base.write_manifest(stage)
        base.seal_tree(stage)
        executable.chmod(0o555)
        os.rename(stage, PARENT)
        os.rename(state_stage, STATE)
        base.fsync_dir(PARENT.parent)
        base.fsync_dir(STATE.parent)
        append_state(0, "R1_MARKERS_AVAILABLE")
        return receipt
    except BaseException:
        if stage.exists():
            shutil.rmtree(stage)
        if state_stage.exists():
            shutil.rmtree(state_stage)
        raise


def subject_environment(route: str, subject: pathlib.Path) -> dict[str, str]:
    paths = base.input_paths()
    environment = base.controlled_environment()
    environment.update({
        "LAY_M3_ACTUAL_OWNER_PACKAGE": str(paths["v13"]),
        "LAY_M3_ACTUAL_OWNER_V7": str(paths["v7"]),
        "LAY_M3_V8_EVIDENCE_DIR": str(subject / "evidence"),
        "LAY_M3_V8_RECEIPT": str(subject / "SUBJECT_RECEIPT.json"),
        "LAY_L2_PACKAGE": str(paths["v13"]),
        "LAY_L2_PRODUCTIVE_V1_PACKAGE": str(paths["productive"]),
        "LAY_L11_RECEIPT": str(paths["l11_receipt"]),
        "LAY_PROPOSAL_ADMISSION_FACT_REUSE": MODES[route],
    })
    return environment


def route_root(route: str) -> pathlib.Path:
    return PARENT / f"{route.lower()}-v1"


def run_once(route: str) -> dict[str, Any]:
    need(os.geteuid() == 0, "subject route requires root")
    need(route in ROUTES, "route must be B0R or B1R")
    state = latest_state()
    expected = "R1_MARKERS_AVAILABLE" if route == "B0R" else "B0R_CREATED"
    need(state is not None and state.get("state") == expected, f"{route} predecessor drift")
    if route == "B1R":
        prior = base.load_json(route_root("B0R") / "SUBJECT_WRAPPER.json")
        need(prior.get("verdict") == "V11_B0R_CREATED" and base.semantic_ok(prior.get("subject_receipt", {})), "B0R did not admit B1R")
    root = route_root(route)
    need(not root.exists(), f"{route} evidence already exists")
    need(not base.active_conflicts(), "conflicting performance process active")
    admission = load_admission(PARENT / "bootstrap")
    host = base.verify_host()
    base.verify_inputs()
    elf = base.require_file(EXECUTABLE, size=ELF_SIZE, digest=ELF_SHA256, mode="0555")
    stage = PARENT / f"{route.lower()}-v1.stage-{os.getpid()}-{time.time_ns()}"
    subject = stage / "subject"
    final_subject = root / "subject"
    evidence = subject / "evidence"
    stage.mkdir(mode=0o755)
    subject.mkdir(mode=0o700)
    evidence.mkdir(mode=0o700)
    shutil.chown(subject, user="e", group="e")
    shutil.chown(evidence, user="e", group="e")
    environment = subject_environment(route, final_subject)
    command = [
        "/usr/bin/sudo", "-n", "-u", "e", "/usr/bin/env",
        *[f"{key}={value}" for key, value in sorted(environment.items())],
        "/usr/bin/taskset", "-c", "0", str(EXECUTABLE),
        "--ignored", "--exact", base.SCIENTIFIC_TEST, "--nocapture", "--test-threads=1",
    ]
    base.write_json(stage / "PRE_SUBJECT.json", {
        "schema": "lay.v11-paired-r1-pre-subject.v1",
        "task_id": TASK_ID,
        "transaction_id": TRANSACTION_ID,
        "route": route,
        "mode": MODES[route],
        "elf": {**elf, "build_id": ELF_BUILD_ID},
        "command": command,
        "environment": environment,
        "subject_started": False,
        "retry_permitted": False,
    })
    base.fsync_dir(stage)
    marker = consume_marker(route, admission)
    base.write_json(stage / "MARKER_CONSUMED.json", marker)
    os.rename(stage, root)
    base.fsync_dir(PARENT)
    thermal_before = base.throttle_counters()
    process = None
    stdout = b""
    stderr = b""
    controller_error = None
    try:
        process = subprocess.Popen(command, stdout=subprocess.PIPE, stderr=subprocess.PIPE, start_new_session=True)
        stdout, stderr = process.communicate(timeout=10_800)
    except BaseException as error:
        controller_error = f"{type(error).__name__}: {error}"
        base.terminate(process)
        if process is not None:
            with contextlib.suppress(Exception):
                stdout, stderr = process.communicate(timeout=1)
    thermal_after = base.throttle_counters()
    base.write_new(root / "stdout.log", stdout)
    base.write_new(root / "stderr.log", stderr)
    receipt_path = final_subject / "SUBJECT_RECEIPT.json"
    receipt = None
    if receipt_path.is_file():
        try:
            receipt = base.load_json(receipt_path)
        except BaseException as error:
            controller_error = controller_error or f"{type(error).__name__}: {error}"
    complete = isinstance(receipt, dict) and receipt.get("schema") == "lay.m3-end-to-end-test-owner.v1" and receipt.get("observation_complete", True) is not False
    semantic = complete and base.semantic_ok(receipt)
    exit_code = process.returncode if process is not None else None
    exit_consistent = complete and ((receipt.get("verdict") == "M3_END_TO_END_TEST_OWNER_PASS" and exit_code == 0) or (receipt.get("verdict") != "M3_END_TO_END_TEST_OWNER_PASS" and exit_code not in (None, 0)))
    if controller_error is not None or not complete or not exit_consistent:
        verdict = "BLOCKED_PROVENANCE"
        state_name = "BLOCKED_PROVENANCE"
    elif not semantic:
        verdict = "BLOCKED_SEMANTIC"
        state_name = "BLOCKED_SEMANTIC"
    else:
        verdict = f"V11_{route}_CREATED"
        state_name = f"{route}_CREATED"
    wrapper = {
        "schema": "lay.v11-paired-r1-subject-wrapper.v1",
        "task_id": TASK_ID,
        "transaction_id": TRANSACTION_ID,
        "route": route,
        "mode": MODES[route],
        "verdict": verdict,
        "controller_error": controller_error,
        "marker": marker,
        "host": host,
        "elf": {**elf, "build_id": ELF_BUILD_ID},
        "command": command,
        "environment": environment,
        "exit_code": exit_code,
        "subject_receipt": receipt,
        "outputs_complete": complete,
        "semantic_exact": semantic,
        "thermal_throttle_drift": base.throttle_drift(thermal_before, thermal_after),
        "subject_executions": int(process is not None),
        "cargo_invocations": 0,
        "rustc_compilations": 0,
        "perf_record_invocations": 0,
        "perf_stat_invocations": 0,
        "runtime_authority_changed": False,
        "retry_permitted": False,
    }
    base.write_json(root / "SUBJECT_WRAPPER.json", wrapper)
    base.write_manifest(root)
    base.seal_tree(root)
    append_state(1 if route == "B0R" else 2, state_name, wrapper_sha256=base.sha256_file(root / "SUBJECT_WRAPPER.json"))
    return wrapper


def terminal() -> dict[str, Any]:
    need(os.geteuid() == 0, "terminal requires root")
    state = latest_state()
    need(state is not None and state.get("state") == "B1R_CREATED", "terminal predecessor drift")
    need(not TERMINAL.exists(), "terminal already exists")
    b0 = base.load_json(route_root("B0R") / "SUBJECT_WRAPPER.json")
    b1 = base.load_json(route_root("B1R") / "SUBJECT_WRAPPER.json")
    r0 = b0.get("subject_receipt", {})
    r1 = b1.get("subject_receipt", {})
    failures = []
    if b0.get("verdict") != "V11_B0R_CREATED" or b1.get("verdict") != "V11_B1R_CREATED":
        failures.append("paired wrapper verdict drift")
    if b0.get("mode") != "UNCACHED" or b1.get("mode") != "REUSE":
        failures.append("paired mode drift")
    if b0.get("elf") != b1.get("elf"):
        failures.append("B0R/B1R ELF mismatch")
    if not base.semantic_ok(r0) or not base.semantic_ok(r1):
        failures.append("semantic proof incomplete")
    if not failures and base.stable_subject(r0) != base.stable_subject(r1):
        failures.append("paired scientific envelope mismatch")
    f0 = r0.get("fixed_proof", {})
    f1 = r1.get("fixed_proof", {})
    p0 = f0.get("pooled", {})
    p1 = f1.get("pooled", {})
    metrics = {
        "maximum_round_search_p99_us": [f0.get("maximum_round_search_p99_us"), f1.get("maximum_round_search_p99_us")],
        "maximum_round_total_material_p99_us": [f0.get("maximum_round_total_material_p99_us"), f1.get("maximum_round_total_material_p99_us")],
        "pooled_search_p99_us": [p0.get("search", {}).get("p99_us"), p1.get("search", {}).get("p99_us")],
        "pooled_final_materialize_p99_us": [p0.get("final_materialize", {}).get("p99_us"), p1.get("final_materialize", {}).get("p99_us")],
        "pooled_total_material_p99_us": [p0.get("total_material", {}).get("p99_us"), p1.get("total_material", {}).get("p99_us")],
    }
    deltas = {
        key: base.delta_pct(pair[0], pair[1]) if all(isinstance(value, (int, float)) for value in pair) else None
        for key, pair in metrics.items()
    }
    key_deltas = [deltas["maximum_round_total_material_p99_us"], deltas["pooled_total_material_p99_us"]]
    if all(value is not None and value < 0 for value in key_deltas):
        assessment = "OBSERVED_IMPROVEMENT"
    elif all(value is not None and value > 0 for value in key_deltas):
        assessment = "NO_OBSERVED_IMPROVEMENT"
    else:
        assessment = "MIXED_PERFORMANCE"
    verdict = "V11_R1_PAIRED_COMPARISON_COMPLETE" if not failures else "BLOCKED_SEMANTIC"
    receipt = {
        "schema": "lay.v11-paired-r1-terminal.v1",
        "task_id": TASK_ID,
        "transaction_id": TRANSACTION_ID,
        "verdict": verdict,
        "failures": failures,
        "failed_namespace_reused_build_only": True,
        "cargo_invocations": 0,
        "elf": b0.get("elf"),
        "modes": {"B0R": "UNCACHED", "B1R": "REUSE"},
        "semantic_exact": not failures,
        "metrics_b0r_b1r": metrics,
        "b1r_delta_percent": deltas,
        "mechanism_assessment": assessment,
        "legacy_absolute_gates": {
            "B0R": {
                "search_p99_le_3000": isinstance(metrics["maximum_round_search_p99_us"][0], int) and metrics["maximum_round_search_p99_us"][0] <= 3000,
                "total_material_p99_le_5000": isinstance(metrics["maximum_round_total_material_p99_us"][0], int) and metrics["maximum_round_total_material_p99_us"][0] <= 5000,
            },
            "B1R": {
                "search_p99_le_3000": isinstance(metrics["maximum_round_search_p99_us"][1], int) and metrics["maximum_round_search_p99_us"][1] <= 3000,
                "total_material_p99_le_5000": isinstance(metrics["maximum_round_total_material_p99_us"][1], int) and metrics["maximum_round_total_material_p99_us"][1] <= 5000,
            },
            "terminal_if_small_miss": False,
        },
        "markers": marker_inventory(),
        "builds": 0,
        "subject_executions": 2,
        "perf_record_invocations": 0,
        "perf_stat_invocations": 0,
        "pmu_events": 0,
        "runtime_authority_changed": False,
        "production_authority_admitted": False,
        "next_action": "select or reject bounded lexical-fact reuse from measured paired effect",
    }
    stage = PARENT / f"terminal-v1.stage-{os.getpid()}-{time.time_ns()}"
    stage.mkdir(mode=0o700)
    base.write_json(stage / "TERMINAL.json", receipt)
    base.write_manifest(stage)
    base.seal_tree(stage)
    os.rename(stage, TERMINAL)
    base.fsync_dir(PARENT)
    append_state(3, verdict, terminal_sha256=base.sha256_file(TERMINAL / "TERMINAL.json"))
    return receipt


def status() -> dict[str, Any]:
    return {
        "schema": "lay.v11-paired-r1-status.v1",
        "task_id": TASK_ID,
        "transaction_id": TRANSACTION_ID,
        "verdict": "V11_R1_STATUS",
        "parent_exists": PARENT.exists(),
        "state_exists": STATE.exists(),
        "latest_state": latest_state(),
        "markers": marker_inventory(),
        "b0r_exists": route_root("B0R").exists(),
        "b1r_exists": route_root("B1R").exists(),
        "terminal_exists": TERMINAL.exists(),
        "executable": base.file_row(EXECUTABLE) if EXECUTABLE.is_file() else None,
        "active_conflicts": base.active_conflicts(),
    }


def self_check() -> dict[str, Any]:
    need(ACTIONS == ("self-check", "status", "bootstrap", "run-once", "terminal"), "action registry drift")
    need(ROUTES == ("B0R", "B1R") and MODES == {"B0R": "UNCACHED", "B1R": "REUSE"}, "route registry drift")
    need("build-once" not in ACTIONS, "build route became reachable")
    need(PREFLIGHT_RECEIPT_SHA256 and ELF_SHA256, "pinned identity absent")
    return {
        "schema": "lay.v11-paired-r1-remote-self-check.v1",
        "verdict": "V11_R1_REMOTE_CONTROLLER_STATIC_PASS",
        "task_id": TASK_ID,
        "transaction_id": TRANSACTION_ID,
        "actions": list(ACTIONS),
        "routes": list(ROUTES),
        "modes": MODES,
        "cargo_reachable": False,
        "rustc_reachable": False,
        "perf_reachable": False,
        "new_elf_bytes_reachable": False,
        "small_performance_miss_terminal": False,
    }


def parser() -> argparse.ArgumentParser:
    value = argparse.ArgumentParser()
    value.add_argument("action", choices=ACTIONS)
    value.add_argument("--bundle", type=pathlib.Path)
    value.add_argument("--route", choices=ROUTES)
    return value


def main() -> int:
    args = parser().parse_args()
    try:
        if args.action == "self-check":
            value = self_check()
        elif args.action == "status":
            value = status()
        elif args.action == "bootstrap":
            need(args.bundle is not None, "--bundle is required")
            value = bootstrap(args.bundle)
        elif args.action == "run-once":
            need(args.route is not None, "--route is required")
            with route_lock():
                value = run_once(args.route)
        else:
            with route_lock():
                value = terminal()
        print(json.dumps(value, ensure_ascii=False, sort_keys=True))
        return 0
    except BaseException as error:
        print(json.dumps({
            "schema": "lay.v11-paired-r1-remote-error.v1",
            "verdict": "ERROR",
            "task_id": TASK_ID,
            "transaction_id": TRANSACTION_ID,
            "error": f"{type(error).__name__}: {error}",
        }, ensure_ascii=False, sort_keys=True))
        return 1


if __name__ == "__main__":
    sys.exit(main())
