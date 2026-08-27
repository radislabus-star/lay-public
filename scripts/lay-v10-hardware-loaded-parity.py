#!/usr/bin/env python3
"""Run the sealed V10 proxy semantic parity once under the current host load."""

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


TASK_ID = "slice8b-v10-loaded-parity-v1-20260825"
REMOTE = "e@192.168.3.94"
PROJECT_ROOT = pathlib.Path(__file__).resolve().parents[1]
BASE_PATH = PROJECT_ROOT / "scripts/lay-v10-hardware-dirty.py"
BASE_SHA256 = "f0aa4b55a803ef204d1d898cec70eb282757009a71ef5d9310417b2ac17a45e8"
PROXY_SOURCE_PATH = PROJECT_ROOT / "scripts/lay_v10_hardware_test_module.rs.inc"
PROXY_SOURCE_SHA256 = "178ec12a3029fe04ae02b060de2e58f83c6481f05fbba2443ad233e9e9d94757"
PREFLIGHT_RECEIPT = PROJECT_ROOT / (
    "docs/structural_gates/receipts/"
    "LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_LOADED_PARITY_PREFLIGHT_V1_2026-08-25.json"
)
PREFLIGHT_MANIFEST_SHA256 = "1dcf302b1ba72cce7a8293d8581bcc993491f03cad6dd11c3ce6fcca22b2f247"
LOCAL_RESULT = PROJECT_ROOT / (
    "docs/structural_gates/receipts/"
    "LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_LOADED_PARITY_V1_2026-08-25"
)
REMOTE_FINAL_TEXT = f"/home/e/.local/share/lay/provenance/{TASK_ID}"
REMOTE_STATE_TEXT = f"/home/e/.local/state/lay/{TASK_ID}"
PARITY_TEST = "nanda_wave::l2_field::v13_typed_peak::tests::v10_hardware_semantic_parity"
EXPECTED_PARITY = {
    "verdict": "PASS",
    "records": 382,
    "terminal_mismatches": 0,
    "peak_mismatches": 0,
    "completeness_mismatches": 0,
    "work_mismatches": 0,
    "full_row_mismatches": 0,
    "target_form_retained": 382,
    "target_lemma_retained": 382,
    "false_certificates": 0,
    "maximum_product_states": 35_590,
    "maximum_scratch_bytes": 6_656,
}


class ParityError(RuntimeError):
    pass


def require(condition: bool, message: str) -> None:
    if not condition:
        raise ParityError(message)


def now() -> str:
    return dt.datetime.now(dt.timezone.utc).isoformat()


def load_local_base() -> Any:
    specification = importlib.util.spec_from_file_location("lay_v10_dirty_base", BASE_PATH)
    require(specification is not None and specification.loader is not None, "cannot load base controller")
    module = importlib.util.module_from_spec(specification)
    specification.loader.exec_module(module)
    require(module.sha256_file(BASE_PATH) == BASE_SHA256, "base controller identity mismatch")
    return module


def load_remote_base() -> dict[str, Any]:
    source = globals().get("BASE_SOURCE")
    require(isinstance(source, str), "remote base source is absent")
    namespace: dict[str, Any] = {
        "__name__": "lay_v10_dirty_base",
        "__file__": "<lay-v10-hardware-dirty.py>",
    }
    exec(compile(source, "lay-v10-hardware-dirty.py", "exec"), namespace)
    require(namespace["sha256_bytes"](source.encode()) == BASE_SHA256, "remote base identity mismatch")
    namespace["TASK_ID"] = TASK_ID
    namespace["REMOTE_FINAL"] = pathlib.Path(REMOTE_FINAL_TEXT)
    namespace["REMOTE_STATE"] = pathlib.Path(REMOTE_STATE_TEXT)
    return namespace


def validate_parity(value: dict[str, Any]) -> None:
    require(value.get("schema") == "lay.v10.hardware-diagnostic-proxy-parity.v1", "parity schema mismatch")
    require(value.get("schedule_sha256") == "2f8346d97246ac3704434279739fa5ed27705465d628890a367c21875d879e78", "schedule mismatch")
    for key, expected in EXPECTED_PARITY.items():
        require(value.get(key) == expected, f"parity mismatch for {key}: {value.get(key)!r}")
    require(value.get("runtime_authority_changed") is False, "parity changed runtime authority")
    require(value.get("installed_lay_changed") is False, "parity changed installed Lay")


def parity_environment(base: dict[str, Any], receipt: pathlib.Path) -> dict[str, str]:
    return {
        "HOME": "/home/e",
        "LANG": "C.UTF-8",
        "LC_ALL": "C.UTF-8",
        "LOGNAME": "e",
        "PATH": "/home/e/.cargo/bin:/usr/local/bin:/usr/bin:/bin",
        "USER": "e",
        "RUST_TEST_THREADS": "1",
        "LAY_V10_HW_V13_PACKAGE": str(base["PACKAGE"]),
        "LAY_V10_HW_SIDECAR": str(base["SIDECAR"]),
        "LAY_V10_HW_V7": str(base["V7"]),
        "LAY_V10_HW_SCHEDULE": str(base["SCHEDULE"]),
        "LAY_V10_HW_PARITY_RECEIPT": str(receipt),
    }


def remote_run(base: dict[str, Any]) -> None:
    final = pathlib.Path(REMOTE_FINAL_TEXT)
    state = pathlib.Path(REMOTE_STATE_TEXT)
    require(not final.exists(), f"loaded parity result already exists: {final}")
    require(not state.exists(), f"loaded parity attempt already consumed: {state}")
    inputs = base["verify_remote_inputs"]()
    stage = base["REMOTE_PROVENANCE"] / f".{TASK_ID}.stage-{os.getpid()}-{time.time_ns()}"
    stage.mkdir(mode=0o700)
    state.mkdir(parents=True, mode=0o700)
    marker = state / "parity-consumed"
    base["write_new_json"](
        marker,
        {
            "task_id": TASK_ID,
            "consumed_at": now(),
            "test": PARITY_TEST,
            "retry_permitted": False,
        },
    )
    result: dict[str, Any] = {
        "schema": "lay.v10.loaded-semantic-parity-wrapper.v1",
        "task_id": TASK_ID,
        "started_at": now(),
        "verdict": "ERROR",
        "claim": "LOADED_SEMANTIC_PARITY_ONLY",
        "environment_intentionally_loaded": True,
        "inputs": inputs,
        "formal_b_pass": False,
        "latency_measured": False,
        "speed_measurement_repeated": False,
        "v12_admitted": False,
        "perf_invoked": False,
        "pmu_event_opened": False,
        "cargo_invoked": False,
        "host_policy_modified": False,
        "foreign_process_stopped": False,
        "runtime_authority_changed": False,
        "installed_lay_changed": False,
    }
    process: subprocess.Popen[bytes] | None = None
    failure: Exception | None = None
    stdout = b""
    stderr = b""
    try:
        stable_before = base["stable_host_projection"]()
        throttles_before = base["throttle_counters"]()
        load_before = base["light_sample"]()
        background_before = base["tracked_processes"]()
        parity_receipt = stage / "PARITY_RECEIPT.json"
        command = [
            str(base["ELF"]),
            "--exact",
            PARITY_TEST,
            "--ignored",
            "--nocapture",
            "--test-threads=1",
        ]
        started_ns = time.perf_counter_ns()
        process = subprocess.Popen(
            command,
            env=parity_environment(base, parity_receipt),
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            start_new_session=True,
        )
        stdout, stderr = process.communicate(timeout=300)
        completed_ns = time.perf_counter_ns()
        require(process.returncode == 0, f"parity subject exited {process.returncode}: {stderr[-2000:]!r}")
        require(parity_receipt.is_file(), "parity receipt is absent")
        parity = json.loads(parity_receipt.read_text())
        validate_parity(parity)
        background_after = base["tracked_processes"]()
        load_after = base["light_sample"]()
        stable_after = base["stable_host_projection"]()
        throttles_after = base["throttle_counters"]()
        require(stable_before == stable_after, "stable host projection changed")
        require(throttles_before == throttles_after, "thermal throttle counters changed")
        wall_seconds = (completed_ns - started_ns) / 1_000_000_000.0
        result.update(
            {
                "verdict": "LOADED_PARITY_PASS_NO_FORMAL_PROMOTION",
                "command": command,
                "whole_process_wall_ms_context_only": (completed_ns - started_ns) / 1_000_000.0,
                "parity": parity,
                "parity_receipt_sha256": base["sha256_file"](parity_receipt),
                "load_before": load_before,
                "load_after": load_after,
                "background_process_cpu_deltas": base["process_deltas"](
                    background_before, background_after, wall_seconds
                ),
                "stable_host_unchanged": True,
                "throttle_counters_unchanged": True,
            }
        )
    except Exception as error:
        failure = error
        result["error"] = str(error)
        if process is not None:
            base["terminate_owned"](process)
            if not stdout and not stderr:
                try:
                    stdout, stderr = process.communicate(timeout=1)
                except Exception:
                    pass
    base["write_new_bytes"](stage / "stdout.log", stdout)
    base["write_new_bytes"](stage / "stderr.log", stderr)
    result["completed_at"] = now()
    base["write_new_json"](stage / "LOADED_PARITY_WRAPPER_RECEIPT.json", result)
    base["publish"](stage, final)
    print(json.dumps({"verdict": result["verdict"], "receipt": str(final / "LOADED_PARITY_WRAPPER_RECEIPT.json"), "error": result.get("error")}, sort_keys=True))
    if failure is not None:
        raise SystemExit(1)


def remote_main(action: str) -> None:
    require(action == "run", f"unknown remote action: {action}")
    remote_run(load_remote_base())


def remote_call() -> subprocess.CompletedProcess[bytes]:
    clean_source = pathlib.Path(__file__).read_text()
    base_source = BASE_PATH.read_text()
    payload = (
        "import sys\n"
        f"base_source = {base_source!r}\n"
        f"runner_source = {clean_source!r}\n"
        "namespace = {'__name__': 'lay_loaded_parity_remote', "
        "'__file__': '<lay-loaded-parity-remote>', 'BASE_SOURCE': base_source}\n"
        "exec(compile(runner_source, 'lay-v10-hardware-loaded-parity.py', 'exec'), namespace)\n"
        "namespace['remote_main'](sys.argv[1])\n"
    ).encode()
    return subprocess.run(
        ["/usr/bin/ssh", "-o", "BatchMode=yes", REMOTE, "/usr/bin/python3", "-", "run"],
        input=payload,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        check=False,
    )


def local_run() -> None:
    base = load_local_base()
    preflight = json.loads(PREFLIGHT_RECEIPT.read_text())
    require(preflight.get("verdict") == "READY_TO_IMPLEMENT", "loaded parity preflight did not pass")
    require(preflight.get("manifest_sha256") == PREFLIGHT_MANIFEST_SHA256, "preflight identity mismatch")
    require(not LOCAL_RESULT.exists(), f"local loaded parity result already exists: {LOCAL_RESULT}")
    local_before = base.local_runtime_snapshot()
    process = remote_call()
    remote_receipt = subprocess.run(
        ["/usr/bin/ssh", "-o", "BatchMode=yes", REMOTE, "/usr/bin/cat", f"{REMOTE_FINAL_TEXT}/LOADED_PARITY_WRAPPER_RECEIPT.json"],
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        check=False,
    )
    require(remote_receipt.returncode == 0, f"remote parity receipt unavailable: {remote_receipt.stderr[-2000:]!r}")
    receipt = json.loads(remote_receipt.stdout)
    local_after = base.local_runtime_snapshot()
    require(local_before == local_after, "local Lay runtime changed during parity")
    require(process.returncode == 0, f"remote parity failed: {process.stdout[-2000:]!r} {process.stderr[-2000:]!r}")
    require(receipt.get("verdict") == "LOADED_PARITY_PASS_NO_FORMAL_PROMOTION", "loaded parity did not pass")
    stage = LOCAL_RESULT.with_name(f".{LOCAL_RESULT.name}.stage-{os.getpid()}-{time.time_ns()}")
    stage.mkdir(parents=True, mode=0o700)
    base.write_new_bytes(stage / "REMOTE_RECEIPT.json", remote_receipt.stdout)
    base.write_new_json(
        stage / "LOCAL_RUN_RECEIPT.json",
        {
            "schema": "lay.v10.loaded-semantic-parity-local-index.v1",
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
    print(
        json.dumps(
            {
                "verdict": receipt["verdict"],
                "whole_process_wall_ms_context_only": receipt["whole_process_wall_ms_context_only"],
                "parity": {key: receipt["parity"][key] for key in EXPECTED_PARITY},
            },
            indent=2,
            sort_keys=True,
        )
    )


def self_check() -> None:
    base = load_local_base()
    require(base.sha256_file(PROXY_SOURCE_PATH) == PROXY_SOURCE_SHA256, "proxy source identity mismatch")
    proxy_source = PROXY_SOURCE_PATH.read_text()
    require(proxy_source.count(PARITY_TEST) == 1, "parity entrypoint drift")
    fake = {
        "schema": "lay.v10.hardware-diagnostic-proxy-parity.v1",
        "schedule_sha256": "2f8346d97246ac3704434279739fa5ed27705465d628890a367c21875d879e78",
        "runtime_authority_changed": False,
        "installed_lay_changed": False,
        **EXPECTED_PARITY,
    }
    validate_parity(fake)
    source = pathlib.Path(__file__).read_text()
    forbidden = [
        "v10_hardware_" + "b5_proxy",
        "v10_hardware_" + "b6_proxy",
        "/usr/bin/" + "perf",
        "cargo" + " build",
        "system" + "ctl",
    ]
    require(not any(token in source for token in forbidden), "forbidden route token in parity runner")
    print(json.dumps({"verdict": "PASS", "checks": 5, "remote_execution": False}, sort_keys=True))


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("action", choices=("self-check", "run"))
    args = parser.parse_args()
    if args.action == "self-check":
        self_check()
    else:
        local_run()


if __name__ == "__main__":
    main()
