#!/usr/bin/env python3
"""Run-only V2 correction for the sealed exact V10 fused-band M1 ELF."""

from __future__ import annotations

import argparse
import contextlib
import importlib.util
import json
import os
import pathlib
import subprocess
import sys
import time
from typing import Any


TASK_ID = "slice8b-v10-exact-fused-band-transition-m1-20260825-run-v2"
REMOTE = "e@192.168.3.94"
REMOTE_HOSTNAME = "e-MEGA-MINI-M1-13th"
REMOTE_MACHINE_ID_SHA256 = "5ac0bb537745673ef80942c2df32d586f5753c85de1c8b2453aa4e7317b77441"
REMOTE_PARENT = pathlib.Path(
    "/home/e/.local/share/lay/provenance/"
    "slice8b-v10-exact-fused-band-transition-m1-20260825"
)
REMOTE_BUILD = REMOTE_PARENT / "build-v1"
REMOTE_RESULT = REMOTE_PARENT / "result-v2"
REMOTE_FAILURE = REMOTE_PARENT / "run-failure-v2"
REMOTE_STATE = pathlib.Path(
    "/home/e/.local/state/lay/"
    "slice8b-v10-exact-fused-band-transition-m1-20260825-run-v2"
)
V1_REMOTE_STATE = pathlib.Path(
    "/home/e/.local/state/lay/"
    "slice8b-v10-exact-fused-band-transition-m1-20260825"
)

CONTROLLER = pathlib.Path(__file__).resolve()
REMOTE_BOOTSTRAP = CONTROLLER.with_name("controller-v1.py").is_file()
PROJECT_ROOT = (
    pathlib.Path("/home/ubu/projects/lay-l1-exact-peak-search")
    if REMOTE_BOOTSTRAP
    else CONTROLLER.parents[1]
)
V1_CONTROLLER = (
    CONTROLLER.with_name("controller-v1.py")
    if REMOTE_BOOTSTRAP
    else PROJECT_ROOT / "scripts/lay-v10-fused-band-transition-m1.py"
)
BASE_CONTROLLER = (
    CONTROLLER.with_name("base.py")
    if REMOTE_BOOTSTRAP
    else PROJECT_ROOT / "scripts/lay-v10-structural-work.py"
)
FRAGMENT = (
    CONTROLLER.with_name("fragment.inc")
    if REMOTE_BOOTSTRAP
    else PROJECT_ROOT / "scripts/lay_v10_fused_band_transition_m1_test_module.rs.inc"
)
CORRECTION_CONTRACT = PROJECT_ROOT / (
    "docs/structural_gates/"
    "LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_EXACT_FUSED_BAND_TRANSITION_M1_"
    "RUN_CORRECTION_V2_2026-08-25.md"
)
CORRECTION_ROUTE = PROJECT_ROOT / (
    "docs/structural_gates/"
    "LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_EXACT_FUSED_BAND_TRANSITION_M1_"
    "RUN_CORRECTION_V2_ROUTE.md"
)
CORRECTION_ROUTE_RECEIPT = PROJECT_ROOT / (
    "docs/structural_gates/receipts/"
    "LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_EXACT_FUSED_BAND_TRANSITION_M1_"
    "RUN_CORRECTION_V2_ROUTE_RECEIPT_2026-08-25.json"
)
PREFLIGHT = PROJECT_ROOT / (
    "docs/structural_gates/preflights/"
    "LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_EXACT_FUSED_BAND_TRANSITION_M1_"
    "RUN_CORRECTION_V2_IMPLEMENTATION_V2_2026-08-25.json"
)
PREFLIGHT_RECEIPT = PROJECT_ROOT / (
    "docs/structural_gates/receipts/"
    "LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_EXACT_FUSED_BAND_TRANSITION_M1_"
    "RUN_CORRECTION_V2_IMPLEMENTATION_V2_PREFLIGHT_2026-08-25.json"
)
LOCAL_RESULT = PROJECT_ROOT / (
    "docs/structural_gates/receipts/"
    "LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_EXACT_FUSED_BAND_TRANSITION_M1_2026-08-25"
)
ACTIVE_V11 = PROJECT_ROOT / "src/nanda_wave/l2_field/v13_typed_peak.rs"

EXPECTED = {
    "v1_controller": "3132a8f29f25ffef44d9e2fce724e88f56f3798515b6c57de350b1d659028237",
    "fragment": "bdef992d1f9bec095b3f683b384f1e7d23323823625cf3547dc44480511f0d76",
    "preflight_file": "625230fbd1a1e6f87ad4daffec0f6fd457b06a48fdb0089a1007da9f02aeee4d",
    "preflight_receipt": "cd2b61ebe226a34674d7da21d46d471d630abd7a51ced0958266c7e99b3f9b31",
    "executable": "a8fb59fb3745d5b60bf455957b0c1da200a6419b2f65ceee02a4558bf03c1e89",
    "build_id": "31949c25f1fdb513d064b4953aea1ebc5d8828d9",
    "executable_bytes": 20_510_360,
    "production_prefix": "ce9ea2d290602774e2f45444e10e6fdabce6af4a61ae2bdca54c6d2e31a53f26",
    "package": "cce259fe0ce5dce67702383363b66f0fe9b9ff5a87d8f01c4fcf342d91218d7b",
    "sidecar": "a1aa95be8c43fae8fedf02d3261dd1265dcbfbc8d618330fcd6449ae829df8cd",
    "v7": "33fded73e13f565bdc08a83bd440be67c80f6d85a196453050b09bc9f7ef28e4",
    "schedule": "2f8346d97246ac3704434279739fa5ed27705465d628890a367c21875d879e78",
    "active_v11": "d9edfe7346b8636096701d4ac38044d5e386e6741a086a4d8da57ac15bffdf3b",
    "v1_failure_manifest": "9e34abc512307db4730aba857701b73fd7727b03ccc0ca052e58ef0318a2391c",
}
ACTION_CHOICES = ("self-check", "run", "status", "remote-run")


class RunV2Error(RuntimeError):
    pass


def require(condition: bool, message: str) -> None:
    if not condition:
        raise RunV2Error(message)


def load_module(path: pathlib.Path, name: str) -> Any:
    require(path.is_file(), f"missing module: {path}")
    spec = importlib.util.spec_from_file_location(name, path)
    require(spec is not None and spec.loader is not None, f"cannot load module: {path}")
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def corrected_subject_environment(
    m1: Any, base: Any, output: pathlib.Path, mode: str
) -> dict[str, str]:
    artifacts = base.REMOTE_B0A / "inputs/slice8b-v10-f6178f/artifacts"
    environment = base.controlled_environment()
    environment.update(
        {
            "LAY_V10_M1_MODE": mode,
            "LAY_V10_M1_OUTPUT": str(output),
            "LAY_V10_M1_V13_PACKAGE": str(
                base.REMOTE_B0A / "inputs/LAY-L2-RU-FULL-v13.bin"
            ),
            "LAY_V10_M1_SIDECAR": str(artifacts / "LAY-L2-RU-FULL-v13.dafsa"),
            "LAY_V10_M1_V7": str(artifacts / "slice8b-v7-fixed-13x100.json"),
            "LAY_V10_M1_SCHEDULE": str(base.REMOTE_B0B / "query-schedule.json"),
        }
    )
    return environment


def load_components() -> tuple[Any, Any]:
    m1 = load_module(V1_CONTROLLER, "lay_v10_m1_v1_measurement")
    m1.TASK_ID = TASK_ID
    m1.REMOTE = REMOTE
    m1.REMOTE_HOSTNAME = REMOTE_HOSTNAME
    m1.REMOTE_MACHINE_ID_SHA256 = REMOTE_MACHINE_ID_SHA256
    m1.REMOTE_PARENT = REMOTE_PARENT
    m1.REMOTE_BUILD = REMOTE_BUILD
    m1.REMOTE_RESULT = REMOTE_RESULT
    m1.REMOTE_STATE = REMOTE_STATE
    m1.CONTROLLER = CONTROLLER
    m1.FRAGMENT = FRAGMENT
    m1.LOCAL_RESULT = LOCAL_RESULT
    m1.ACTIVE_V11 = ACTIVE_V11
    base = m1.load_base()
    m1.subject_environment = lambda base_value, output, mode: corrected_subject_environment(
        m1, base_value, output, mode
    )
    return m1, base


def verify_manifest_baselines(base: Any) -> dict[str, Any]:
    base.require_file(PREFLIGHT, sha256=EXPECTED["preflight_file"], mode="0664")
    base.require_file(
        PREFLIGHT_RECEIPT, sha256=EXPECTED["preflight_receipt"], mode="0664"
    )
    manifest = base.load_json(PREFLIGHT)
    receipt = base.load_json(PREFLIGHT_RECEIPT)
    require(receipt.get("verdict") == "READY_TO_IMPLEMENT", "V2 preflight is not ready")
    require(receipt.get("safe_to_implement") is True, "V2 preflight is unsafe")
    require(not receipt.get("blockers"), "V2 preflight has blockers")
    checked: dict[str, Any] = {}
    for item in manifest.get("baseline_checks", []):
        identifier = item["id"]
        if identifier == "v2-controller-absent":
            checked[identifier] = base.require_file(CONTROLLER, mode="0755")
            continue
        path = (PREFLIGHT.parent / item["path"]).resolve()
        if item["kind"] == "absent":
            require(not path.exists(), f"baseline expected absent: {path}")
            checked[identifier] = {"path": str(path), "exists": False}
            continue
        expected = item.get("expect", {})
        checked[identifier] = base.require_file(
            path,
            sha256=expected.get("sha256"),
            mode=expected.get("mode"),
            size=expected.get("size_bytes"),
        )
    require(len(checked) == len(manifest.get("baseline_checks", [])), "baseline count drift")
    return checked


def verify_local_admission(base: Any) -> dict[str, Any]:
    checked = verify_manifest_baselines(base)
    base.require_file(V1_CONTROLLER, sha256=EXPECTED["v1_controller"], mode="0755")
    base.require_file(FRAGMENT, sha256=EXPECTED["fragment"], mode="0664")
    base.require_file(ACTIVE_V11, sha256=EXPECTED["active_v11"])
    route = base.load_json(CORRECTION_ROUTE_RECEIPT)
    require(route.get("verdict") == "PASS", "V2 correction route is not PASS")
    require(route.get("authority_ready") is False, "V2 correction route gained authority")
    require(not LOCAL_RESULT.exists(), "local M1 V2 result already exists")
    return checked


def remote_probe(base: Any, *, require_v2_absent: bool) -> dict[str, Any]:
    script = f"""
import hashlib,json,os,pathlib,re,stat,subprocess
def digest(path):
    value=hashlib.sha256()
    with path.open('rb') as source:
        for block in iter(lambda: source.read(1024*1024), b''):
            value.update(block)
    return value.hexdigest()
def item(path):
    path=pathlib.Path(path)
    return {{'path':str(path),'exists':path.is_file(),'sha256':digest(path) if path.is_file() else None,'bytes':path.stat().st_size if path.is_file() else None,'mode':f'{{stat.S_IMODE(path.stat().st_mode):04o}}' if path.exists() else None}}
build=pathlib.Path('{REMOTE_BUILD}')
b0a=pathlib.Path('/home/e/.local/share/lay/provenance/slice8b-v10-hardware-b0-b2-v3-20260824/b0a-input-closure-v2')
b0b=pathlib.Path('/home/e/.local/share/lay/provenance/slice8b-v10-hardware-b0-b2-v3-20260824/b0b-schedule-closure-v1')
v1=pathlib.Path('{V1_REMOTE_STATE}')
v2=pathlib.Path('{REMOTE_STATE}')
readelf=subprocess.check_output(['readelf','-n',str(build/'diagnostic-test-elf')],text=True)
match=re.search(r'Build ID:\\s*([0-9a-f]+)',readelf)
print(json.dumps({{
 'host':os.uname().nodename,
 'machine_id_sha256':digest(pathlib.Path('/etc/machine-id')),
 'elf':item(build/'diagnostic-test-elf'),
 'build_id':match.group(1) if match else None,
 'package':item(b0a/'inputs/LAY-L2-RU-FULL-v13.bin'),
 'sidecar':item(b0a/'inputs/slice8b-v10-f6178f/artifacts/LAY-L2-RU-FULL-v13.dafsa'),
 'v7':item(b0a/'inputs/slice8b-v10-f6178f/artifacts/slice8b-v7-fixed-13x100.json'),
 'schedule':item(b0b/'query-schedule.json'),
 'v1_markers':sorted(path.name for path in (v1/'markers').glob('*')) if (v1/'markers').is_dir() else [],
 'v1_failure_manifest':item(pathlib.Path('{REMOTE_PARENT}')/'run-failure-v1'/'SHA256SUMS'),
 'v1_result_exists':(pathlib.Path('{REMOTE_PARENT}')/'result-v1').exists(),
 'v2_state_exists':v2.exists(),
 'v2_result_exists':pathlib.Path('{REMOTE_RESULT}').exists(),
 'v2_failure_exists':pathlib.Path('{REMOTE_FAILURE}').exists(),
}},sort_keys=True))
"""
    result = base.ssh(["python3", "-c", script])
    value = json.loads(result.stdout)
    require(value["host"] == REMOTE_HOSTNAME, "remote hostname mismatch")
    require(value["machine_id_sha256"] == REMOTE_MACHINE_ID_SHA256, "remote machine mismatch")
    require(value["elf"]["sha256"] == EXPECTED["executable"], "sealed ELF SHA drift")
    require(value["elf"]["bytes"] == EXPECTED["executable_bytes"], "sealed ELF size drift")
    require(value["elf"]["mode"] == "0444", "sealed ELF mode drift")
    require(value["build_id"] == EXPECTED["build_id"], "sealed ELF Build ID drift")
    for name in ("package", "sidecar", "v7", "schedule"):
        require(value[name]["sha256"] == EXPECTED[name], f"remote {name} identity drift")
        require(value[name]["mode"] == "0444", f"remote {name} is writable")
    require(
        value["v1_markers"]
        == [
            "build.consumed-before-exec",
            "g0.available",
            "g1.available",
            "parity.consumed-before-exec",
            "u1.available",
        ],
        "V1 marker state drift",
    )
    require(
        value["v1_failure_manifest"]["sha256"] == EXPECTED["v1_failure_manifest"],
        "V1 failure evidence drift",
    )
    require(value["v1_result_exists"] is False, "V1 result unexpectedly exists")
    if require_v2_absent:
        require(value["v2_state_exists"] is False, "V2 state already exists")
        require(value["v2_result_exists"] is False, "V2 result already exists")
        require(value["v2_failure_exists"] is False, "V2 failure already exists")
    return value


def local_self_check(m1: Any, base: Any) -> None:
    checked = verify_local_admission(base)
    source = CONTROLLER.read_text(encoding="utf-8")
    compile(source, str(CONTROLLER), "exec")
    require("build" not in ACTION_CHOICES, "V2 exposes a build action")
    require("result-v2" in source and "run-failure-v2" in source, "V2 namespaces missing")
    require("LAY-L2-RU-FULL-v13.dafsa" in source, "corrected sidecar missing")
    require("slice8b-v7-fixed-13x100.json" in source, "corrected V7 missing")
    old_sidecar = "v13-typed-peak-" + "dafsa.bin"
    old_v7 = "denominator-v7" + ".json"
    require(old_sidecar not in source, "old sidecar alias remains")
    require(old_v7 not in source, "old V7 alias remains")
    remote = remote_probe(base, require_v2_absent=True)
    print(
        json.dumps(
            {
                "verdict": "PASS",
                "task_id": TASK_ID,
                "baseline_checks": len(checked),
                "controller_sha256": base.sha256_file(CONTROLLER),
                "v1_controller_sha256": base.sha256_file(V1_CONTROLLER),
                "sealed_elf_sha256": remote["elf"]["sha256"],
                "sealed_elf_build_id": remote["build_id"],
                "assets": {name: remote[name]["sha256"] for name in ("package", "sidecar", "v7", "schedule")},
                "actions": ACTION_CHOICES,
                "build_action": False,
                "v1_preserved": True,
                "v2_state": "ABSENT_UNRUN",
                "formal_b_pass": False,
                "v12_admitted": False,
            },
            sort_keys=True,
        )
    )


def initialize_v2_state(base: Any) -> None:
    require(REMOTE_PARENT.is_dir(), "M1 remote parent missing")
    require(not REMOTE_STATE.exists(), "V2 remote state already exists")
    markers = REMOTE_STATE / "markers"
    markers.mkdir(parents=True, mode=0o700)
    for name in ("parity", "g0", "g1", "u1"):
        base.write_new_json(
            markers / f"{name}.available",
            {
                "task_id": TASK_ID,
                "route": name,
                "retry_permitted": False,
                "sealed_elf_sha256": EXPECTED["executable"],
            },
            0o400,
        )
    base.write_new_bytes(REMOTE_STATE / "route.lock", b"M1-RUN-V2\n", 0o400)
    base.fsync_directory(markers)
    base.fsync_directory(REMOTE_STATE)


def in_process_inputs(base: Any) -> dict[str, Any]:
    base.remote_machine_identity()
    base.verify_sha256sums(REMOTE_BUILD)
    executable = REMOTE_BUILD / "diagnostic-test-elf"
    provenance = base.load_json(REMOTE_BUILD / "EXECUTABLE_PROVENANCE.json")
    base.require_file(
        executable,
        sha256=EXPECTED["executable"],
        mode="0444",
        size=EXPECTED["executable_bytes"],
    )
    require(base.elf_build_id(executable) == EXPECTED["build_id"], "Build ID drift")
    require(provenance["executable"]["sha256"] == EXPECTED["executable"], "provenance ELF drift")
    require(
        provenance["source"]["production_prefix_sha256"] == EXPECTED["production_prefix"],
        "production prefix drift",
    )
    artifacts = base.REMOTE_B0A / "inputs/slice8b-v10-f6178f/artifacts"
    inputs = {
        "package": base.require_file(
            base.REMOTE_B0A / "inputs/LAY-L2-RU-FULL-v13.bin",
            sha256=EXPECTED["package"],
            mode="0444",
        ),
        "sidecar": base.require_file(
            artifacts / "LAY-L2-RU-FULL-v13.dafsa",
            sha256=EXPECTED["sidecar"],
            mode="0444",
        ),
        "v7": base.require_file(
            artifacts / "slice8b-v7-fixed-13x100.json",
            sha256=EXPECTED["v7"],
            mode="0444",
        ),
        "schedule": base.require_file(
            base.REMOTE_B0B / "query-schedule.json",
            sha256=EXPECTED["schedule"],
            mode="0444",
        ),
    }
    return {"provenance": provenance, "inputs": inputs}


def v1_projection(base: Any) -> dict[str, Any]:
    markers = V1_REMOTE_STATE / "markers"
    value = {
        "markers": sorted(path.name for path in markers.glob("*")),
        "failure_manifest_sha256": base.sha256_file(
            REMOTE_PARENT / "run-failure-v1/SHA256SUMS"
        ),
        "result_v1_exists": (REMOTE_PARENT / "result-v1").exists(),
        "build_manifest_sha256": base.sha256_file(REMOTE_BUILD / "SHA256SUMS"),
    }
    require(
        value["markers"]
        == [
            "build.consumed-before-exec",
            "g0.available",
            "g1.available",
            "parity.consumed-before-exec",
            "u1.available",
        ],
        "V1 markers changed",
    )
    require(
        value["failure_manifest_sha256"] == EXPECTED["v1_failure_manifest"],
        "V1 failure changed",
    )
    require(value["result_v1_exists"] is False, "V1 result appeared")
    return value


def remote_run(m1: Any, base: Any) -> None:
    require(os.uname().nodename == REMOTE_HOSTNAME, "remote-run on wrong host")
    require(not REMOTE_RESULT.exists(), "M1 V2 result already exists")
    require(not REMOTE_FAILURE.exists(), "M1 V2 failure already exists")
    require(not REMOTE_STATE.exists(), "M1 V2 state already exists")
    verified = in_process_inputs(base)
    v1_before = v1_projection(base)
    initialize_v2_state(base)
    with base.remote_lock():
        stage = REMOTE_PARENT / f"result-v2.stage-{os.getpid()}-{time.time_ns()}"
        stage.mkdir(mode=0o700)
        before = base.environment_snapshot()
        try:
            parity = m1.run_parity(base, stage)
            physical = {
                mode: m1.run_physical(base, stage, mode, parity) for mode in m1.MODES
            }
            g0 = physical["G0"]["derived"]["instructions_per_transition"]
            g1 = physical["G1"]["derived"]["instructions_per_transition"]
            u1 = physical["U1"]["derived"]["instructions_per_transition"]
            delta_per_query = (g0 - u1) * m1.TRANSITIONS_PER_QUERY
            projected_saving = delta_per_query / m1.BASELINE_INSTRUCTIONS_PER_QUERY
            passed = u1 < g0 and projected_saving >= m1.PROJECTED_SAVING_GATE
            after = base.environment_snapshot()
            v1_after = v1_projection(base)
            require(v1_before == v1_after, "V1 state or evidence changed during V2")
            decision = {
                "schema": "lay.v10.exact-fused-band-transition-m1-decision.v2",
                "verdict": "M1_PASS" if passed else "M1_REJECT_FUSED",
                "task_id": TASK_ID,
                "subject": verified["provenance"]["executable"],
                "production_prefix_bytes": 39_047,
                "production_prefix_sha256": EXPECTED["production_prefix"],
                "parity": parity,
                "physical": physical,
                "comparison": {
                    "g0_instructions_per_transition": g0,
                    "g1_instructions_per_transition": g1,
                    "u1_instructions_per_transition": u1,
                    "g0_to_g1_percent": 100.0 * (g0 - g1) / g0,
                    "g1_to_u1_percent": 100.0 * (g1 - u1) / g1,
                    "g0_to_u1_percent": 100.0 * (g0 - u1) / g0,
                    "projected_instruction_delta_per_query": delta_per_query,
                    "projected_whole_query_instruction_saving": projected_saving,
                    "projected_saving_gate": m1.PROJECTED_SAVING_GATE,
                },
                "asset_identities": verified["inputs"],
                "environment_before": before,
                "environment_after": after,
                "loaded_host_is_blocker": False,
                "s1_swar_candidate": "OMITTED",
                "claim_boundary": {
                    "latency_prediction": False,
                    "latency_pass": False,
                    "full_executor_admitted": False,
                    "formal_b_pass": False,
                    "v12_admitted": False,
                    "runtime_authority_changed": False,
                },
            }
            base.write_new_json(stage / "M1_DECISION.json", decision)
            base.write_new_json(
                stage / "RUN_PROVENANCE.json",
                {
                    "schema": "lay.v10.exact-fused-band-transition-m1-run.v2",
                    "task_id": TASK_ID,
                    "controller_v2_sha256": base.sha256_file(CONTROLLER),
                    "measurement_controller_v1_sha256": base.sha256_file(V1_CONTROLLER),
                    "base_controller_sha256": base.sha256_file(BASE_CONTROLLER),
                    "fragment_sha256": base.sha256_file(FRAGMENT),
                    "sealed_elf_reused": True,
                    "build_executed": False,
                    "markers_consumed": ["parity", "g0", "g1", "u1"],
                    "state_namespace": str(REMOTE_STATE),
                    "result_namespace": str(REMOTE_RESULT),
                    "adaptive_rerun": False,
                    "third_loaded_c1_run": False,
                    "clean_c1_marker_consumed": False,
                    "foreign_process_control": False,
                    "host_tuning": False,
                    "installed_lay_changed": False,
                    "v1_projection_before": v1_before,
                    "v1_projection_after": v1_after,
                    "runtime_authority_changed": False,
                },
            )
            base.write_sha256sums(stage)
            base.seal_tree(stage)
            os.rename(stage, REMOTE_RESULT)
            base.fsync_directory(REMOTE_PARENT)
            print(json.dumps({"state": decision["verdict"], "result": str(REMOTE_RESULT)}))
        except Exception as error:
            with contextlib.suppress(Exception):
                base.write_new_json(
                    stage / "FAILURE.json",
                    {
                        "schema": "lay.v10.exact-fused-band-transition-m1-failure.v2",
                        "error": str(error),
                        "state_namespace": str(REMOTE_STATE),
                        "retry_permitted": False,
                        "runtime_authority_changed": False,
                    },
                )
                base.write_sha256sums(stage)
                base.seal_tree(stage)
                os.rename(stage, REMOTE_FAILURE)
                base.fsync_directory(REMOTE_PARENT)
            raise


def upload_bootstrap(base: Any) -> str:
    temporary = base.ssh(
        ["mktemp", "-d", "/tmp/lay-v10-m1-run-v2.XXXXXX"]
    ).stdout.decode().strip()
    require(temporary.startswith("/tmp/lay-v10-m1-run-v2."), "unexpected bootstrap path")
    for source, name in (
        (CONTROLLER, "controller.py"),
        (V1_CONTROLLER, "controller-v1.py"),
        (BASE_CONTROLLER, "base.py"),
        (FRAGMENT, "fragment.inc"),
    ):
        base.scp(source, f"{temporary}/{name}")
    return temporary


def local_run(m1: Any, base: Any) -> None:
    verify_local_admission(base)
    remote_probe(base, require_v2_absent=True)
    before = base.local_runtime_snapshot()
    temporary = upload_bootstrap(base)
    try:
        result = base.ssh(
            ["python3", f"{temporary}/controller.py", "remote-run"], check=False
        )
        require(result.returncode == 0, result.stderr.decode(errors="replace")[-10_000:])
        remote_state = result.stdout.decode().strip()
    finally:
        base.ssh(["rm", "-rf", "--", temporary], check=False)
    after = base.local_runtime_snapshot()
    require(before == after, f"installed runtime changed: {before} != {after}")
    stage = pathlib.Path(f"{LOCAL_RESULT}.stage-{os.getpid()}-{time.time_ns()}")
    try:
        base.run(["scp", "-q", "-pr", f"{REMOTE}:{REMOTE_RESULT}", str(stage)])
        entries = base.verify_sha256sums(stage)
        decision = base.load_json(stage / "M1_DECISION.json")
        require(decision.get("verdict") in ("M1_PASS", "M1_REJECT_FUSED"), "invalid M1 decision")
        require(decision.get("claim_boundary", {}).get("v12_admitted") is False, "M1 admitted V12")
        require(
            not any(path.stat().st_mode & 0o222 for path in stage.rglob("*")),
            "remote V2 result contains writable objects",
        )
        base.seal_tree(stage)
        os.rename(stage, LOCAL_RESULT)
        base.fsync_directory(LOCAL_RESULT.parent)
        print(
            json.dumps(
                {
                    "state": decision["verdict"],
                    "remote": remote_state,
                    "local_result": str(LOCAL_RESULT),
                    "manifest_entries": entries,
                    "runtime_stable": True,
                },
                sort_keys=True,
            )
        )
    except Exception:
        base.remove_tree(stage)
        raise


def status(base: Any) -> None:
    script = (
        "import json,pathlib;"
        f"p=pathlib.Path('{REMOTE_PARENT}');s=pathlib.Path('{REMOTE_STATE}');"
        "m=s/'markers';"
        "print(json.dumps({'build':(p/'build-v1').exists(),"
        "'result_v2':(p/'result-v2').exists(),'failure_v2':(p/'run-failure-v2').exists(),"
        "'state_v2':s.exists(),'markers_v2':sorted(x.name for x in m.glob('*')) if m.is_dir() else []}))"
    )
    result = base.ssh(["python3", "-c", script])
    print(result.stdout.decode().strip())


def parser() -> argparse.ArgumentParser:
    value = argparse.ArgumentParser()
    value.add_argument("action", choices=ACTION_CHOICES)
    return value


def main() -> int:
    arguments = parser().parse_args()
    try:
        m1, base = load_components()
        if arguments.action == "self-check":
            local_self_check(m1, base)
        elif arguments.action == "run":
            local_run(m1, base)
        elif arguments.action == "status":
            status(base)
        elif arguments.action == "remote-run":
            remote_run(m1, base)
        return 0
    except Exception as error:
        print(f"M1 RUN V2 ERROR: {error}", file=sys.stderr)
        return 1


if __name__ == "__main__":
    raise SystemExit(main())
