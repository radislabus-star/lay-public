#!/usr/bin/env python3
"""Local controller for the W1 fused-minimum M2 experiment."""

from __future__ import annotations

import argparse
import ast
import hashlib
import json
import os
import pathlib
import runpy
import shlex
import shutil
import stat
import subprocess
import sys
import tempfile
import time
from collections.abc import Mapping, Sequence
from typing import Any


ROOT = pathlib.Path(__file__).resolve().parents[1]
TASK_ID = "slice8b-v10-e1-traversal-w1-fused-minimum-m2-v1-20260826"
TRANSACTION_ID = "c760eea52b6416b3529f9d684c315147b5a1140522114642c417d7db4065102c"
REMOTE = "e@192.168.3.94"
SSH_IDENTITY = pathlib.Path("/home/ubu/.ssh/mega-mini-admin")
REMOTE_CACHE = pathlib.PurePosixPath("/home/e/.cache") / f"lay-m2-{TRANSACTION_ID}"
REMOTE_PARENT = pathlib.PurePosixPath("/home/e/.local/share/lay/provenance") / TASK_ID
CONTROLLER = pathlib.Path(__file__).resolve()
REMOTE_CONTROLLER = ROOT / "scripts/lay-v10-e1-traversal-w1-fused-minimum-m2-remote.py"
BOOTSTRAP_AUDITOR = ROOT / "scripts/lay-v10-e1-traversal-w1-fused-minimum-m2-bootstrap-audit.py"
BUILD_AUDITOR = ROOT / "scripts/lay-v10-e1-traversal-w1-fused-minimum-m2-build-audit.py"
TERMINAL_AUDITOR = ROOT / "scripts/lay-v10-e1-traversal-w1-fused-minimum-m2-terminal-audit.py"
FRAGMENT = ROOT / "scripts/lay_v10_e1_traversal_w1_fused_minimum_m2_test_module.rs.inc"
V10_SOURCE = pathlib.Path(
    "/home/ubu/.local/share/lay/provenance/"
    "slice8b-v10-f6178f/artifacts/v13_typed_peak.v10.rs"
)
PREFLIGHT = ROOT / (
    "docs/structural_gates/preflights/"
    "LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_W1_FUSED_MINIMUM_M2_"
    "IMPLEMENTATION_V3_2026-08-26.json"
)
PREFLIGHT_RECEIPT = ROOT / (
    "docs/structural_gates/receipts/"
    "LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_W1_FUSED_MINIMUM_M2_"
    "IMPLEMENTATION_V3_PREFLIGHT_2026-08-26.json"
)
IMPLEMENTATION_RECEIPT = ROOT / (
    "docs/structural_gates/receipts/"
    "LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_W1_FUSED_MINIMUM_M2_"
    "IMPLEMENTATION_SELF_CHECK_V2_2026-08-26.json"
)
EXECUTION_ADMISSION = ROOT / (
    "docs/structural_gates/preflights/"
    "LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_W1_FUSED_MINIMUM_M2_"
    "EXECUTION_ADMISSION_V1_2026-08-26.json"
)
BOOTSTRAP_AUDIT_RECEIPT = ROOT / (
    "docs/structural_gates/receipts/"
    "LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_W1_FUSED_MINIMUM_M2_"
    "BOOTSTRAP_AUDIT_V1_2026-08-26/M2_BOOTSTRAP_AUDIT_RECEIPT.json"
)
BUILD_AUDIT_RECEIPT = ROOT / (
    "docs/structural_gates/receipts/"
    "LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_W1_FUSED_MINIMUM_M2_"
    "BUILD_AUDIT_V1_2026-08-26/M2_BUILD_AUDIT_RECEIPT.json"
)
TERMINAL_AUDIT_RECEIPT = ROOT / (
    "docs/structural_gates/receipts/"
    "LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_W1_FUSED_MINIMUM_M2_"
    "TERMINAL_AUDIT_V1_2026-08-26/M2_TERMINAL_AUDIT_RECEIPT.json"
)
PREFLIGHT_SHA256 = "06043cceb8264f9bfe95abe54131e332a684815b0f0a6ab3cf1356158c035520"
PREFLIGHT_RECEIPT_SHA256 = "6ff65d0136cb713e0af9e6930c2a27708dd4c3b762d82b5fc22fa6ee000b33ae"
FRAGMENT_SHA256 = "b0a775420edf9e9d6e7f0b59f9ad840e4822cd0fdd0adc2429ea22a3e9e3a175"
ASSEMBLED_SHA256 = "8654217a1509ef4ca9ef3c3dda5080a7c784fb767359c52531a772c0feae68dc"
V10_SHA256 = "f6178f3882c1a216a9c2aaf7ff59dd6bdd78798708516b753ea7208177d9718c"
D1_FRAGMENT_SHA256 = "bbd8b8d318810eec721812f21efbeb5f231dacba774cb5ade854e2201c6c7665"
PRODUCTION_PREFIX_SHA256 = "ce9ea2d290602774e2f45444e10e6fdabce6af4a61ae2bdca54c6d2e31a53f26"
ROUTE_ORDER = (
    "B0-ITERATOR",
    "G0-M1-GUARDED",
    "I0-INTERLEAVED",
    "I1-INTERLEAVED",
    "G1-M1-GUARDED",
    "B1-ITERATOR",
)
MARKER_ROUTES = ("BUILD", "PARITY", *ROUTE_ORDER)
MUTABLE_BASELINE_IDS = {
    "m2-local-controller-unverified",
    "m2-remote-controller-unverified",
    "m2-bootstrap-auditor-unverified",
    "m2-build-auditor-unverified",
    "m2-terminal-auditor-unverified",
    "m2-rust-fragment-unverified",
}
COMMAND_GRAPH = {
    "BOOTSTRAP": ("bootstrap_once", ()),
    "BUILD": ("build_once", ("cargo-guard",)),
    "READ-ONLY BUILD AUDIT": ("audit", ("readelf", "nm")),
    "PARITY": ("parity_once", ("diagnostic-test-elf",)),
    **{route: ("run_physical", ("perf stat", "diagnostic-test-elf")) for route in ROUTE_ORDER},
    "TERMINAL AUDIT": ("audit", ()),
}


class M2ControllerError(RuntimeError):
    pass


def need(condition: bool, message: str) -> None:
    if not condition:
        raise M2ControllerError(message)


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


def file_row(path: pathlib.Path) -> dict[str, Any]:
    need(path.is_file() and not path.is_symlink(), f"file absent: {path}")
    return {"path": str(path), "mode": mode_string(path), "size_bytes": path.stat().st_size, "sha256": sha256_file(path)}


def require_file(path: pathlib.Path, *, digest: str | None = None, size: int | None = None, mode: str | None = None) -> dict[str, Any]:
    value = file_row(path)
    if digest is not None:
        need(value["sha256"] == digest, f"SHA drift: {path}")
    if size is not None:
        need(value["size_bytes"] == size, f"size drift: {path}")
    if mode is not None:
        need(value["mode"] == mode, f"mode drift: {path}")
    return value


def canonical(value: Any) -> bytes:
    return (json.dumps(value, ensure_ascii=False, sort_keys=True, separators=(",", ":")) + "\n").encode()


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


def run(command: Sequence[str], *, input_bytes: bytes | None = None, timeout: float = 3_600, check: bool = True) -> subprocess.CompletedProcess[bytes]:
    result = subprocess.run(list(command), input=input_bytes, stdout=subprocess.PIPE, stderr=subprocess.PIPE, timeout=timeout, check=False)
    if check and result.returncode != 0:
        raise M2ControllerError(f"command failed ({result.returncode}): {shlex.join(command)}\n{result.stderr.decode(errors='replace')[-5000:]}")
    return result


def assemble_source() -> bytes:
    v10 = V10_SOURCE.read_bytes()
    fragment = FRAGMENT.read_bytes()
    need(len(v10) == 91_518 and sha256_bytes(v10) == V10_SHA256 and v10.endswith(b"}\n"), "V10 source drift")
    need(len(fragment) == 155_810 and sha256_bytes(fragment) == FRAGMENT_SHA256, "M2 fragment drift")
    need(sha256_bytes(fragment[:113_204]) == D1_FRAGMENT_SHA256, "D1 fragment prefix drift")
    assembled = v10[:-2] + fragment + b"}\n"
    need(len(assembled) == 247_328 and sha256_bytes(assembled) == ASSEMBLED_SHA256, "assembled source drift")
    need(sha256_bytes(assembled[:39_047]) == PRODUCTION_PREFIX_SHA256, "production prefix drift")
    return assembled


def verify_preflight() -> dict[str, Any]:
    preflight_row = require_file(PREFLIGHT, digest=PREFLIGHT_SHA256, size=12_492, mode="0444")
    receipt_row = require_file(PREFLIGHT_RECEIPT, digest=PREFLIGHT_RECEIPT_SHA256, size=8_213, mode="0444")
    preflight = json.loads(PREFLIGHT.read_text())
    receipt = json.loads(PREFLIGHT_RECEIPT.read_text())
    need(receipt.get("verdict") == "READY_TO_IMPLEMENT" and receipt.get("safe_to_implement") is True and not receipt.get("blockers"), "M2 implementation preflight is not ready")
    need(
        receipt.get("task_id")
        == "LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_W1_FUSED_MINIMUM_M2_IMPLEMENTATION_V3_2026-08-26",
        "M2 implementation preflight task drift",
    )
    sealed_baseline = {row.get("id"): row for row in receipt.get("baseline_receipts", [])}
    baseline = {}
    for check in preflight.get("baseline_checks", []):
        if check.get("kind") != "file":
            continue
        expected = check.get("expect", {})
        path = (PREFLIGHT.parent / check["path"]).resolve()
        if check["id"] in MUTABLE_BASELINE_IDS:
            row = sealed_baseline.get(check["id"], {})
            need(
                row.get("exists") is True
                and row.get("sha256") == expected.get("sha256")
                and row.get("size_bytes") == expected.get("size_bytes")
                and row.get("mode") == expected.get("mode"),
                f"mutable preflight baseline receipt drift: {check['id']}",
            )
            baseline[check["id"]] = {"pre_edit_receipt": row, "current": file_row(path)}
        else:
            baseline[check["id"]] = require_file(path, digest=expected.get("sha256"), size=expected.get("size_bytes"), mode=expected.get("mode"))
    need(len(baseline) == 15, f"M2 pinned file closure drift: {len(baseline)}")
    return {"preflight": preflight_row, "preflight_receipt": receipt_row, "baseline": baseline}


def python_sources() -> tuple[pathlib.Path, ...]:
    return (CONTROLLER, REMOTE_CONTROLLER, BOOTSTRAP_AUDITOR, BUILD_AUDITOR, TERMINAL_AUDITOR)


def verify_python_graph() -> dict[str, Any]:
    trees = {}
    for path in python_sources():
        text = path.read_text()
        compile(text, str(path), "exec")
        trees[path.name] = ast.parse(text)
    remote_text = REMOTE_CONTROLLER.read_text()
    terminal_text = TERMINAL_AUDITOR.read_text()
    forbidden = ("perf record", "--pid", "SIGINT", "precise_ip", "systemctl", "install-release", "I-ATOM", "pkill", "killall")
    for token in forbidden:
        need(token not in remote_text, f"forbidden remote route token: {token}")
    need(remote_text.count('"/usr/bin/perf", "stat"') == 1, "perf stat producer cardinality drift")
    need("def build_once(" in remote_text and "cargo-guard.sh" in remote_text, "build producer absent")
    need("def parity_once(" in remote_text and "def run_physical(" in remote_text, "subject producer graph incomplete")
    need("class FailureSet" in terminal_text and "def decide(" in terminal_text, "terminal dispatch owner absent")
    need("EXPECTED_BUILD_ENVIRONMENT" in BUILD_AUDITOR.read_text(), "build environment auditor absent")
    need("controller_error" in remote_text and "PARITY_WRAPPER.json" in remote_text, "parity failure wrapper closure absent")
    remote_tree = trees[REMOTE_CONTROLLER.name]
    assignments = {
        node.targets[0].id: ast.literal_eval(node.value)
        for node in remote_tree.body
        if isinstance(node, ast.Assign)
        and len(node.targets) == 1
        and isinstance(node.targets[0], ast.Name)
        and node.targets[0].id
        in {"ROUTE_ORDER", "EVENTS", "BUILD_ENVIRONMENT_FIXED", "BUILD_ENVIRONMENT_KEYS"}
    }
    need(tuple(assignments.get("ROUTE_ORDER", ())) == ROUTE_ORDER, "remote route registry drift")
    need(
        'MARKER_ROUTES = ("BUILD", "PARITY", *ROUTE_ORDER)' in remote_text,
        "remote marker registry drift",
    )
    need(
        tuple(assignments.get("EVENTS", ()))
        == ("instructions", "cycles", "branches", "branch-misses", "task-clock"),
        "remote perf event registry drift",
    )
    need(
        assignments.get("BUILD_ENVIRONMENT_FIXED")
        == {
            "CARGO_BUILD_JOBS": "20",
            "CARGO_INCREMENTAL": "0",
            "CARGO_NET_OFFLINE": "true",
            "CARGO_PROFILE_RELEASE_DEBUG": "2",
            "CARGO_PROFILE_RELEASE_STRIP": "none",
            "RUSTFLAGS": "",
        },
        "remote build symbolization environment drift",
    )
    need(
        set(assignments.get("BUILD_ENVIRONMENT_KEYS", ()))
        == {*assignments["BUILD_ENVIRONMENT_FIXED"], "CARGO_TARGET_DIR"},
        "remote build environment key closure drift",
    )
    functions = {node.name for node in ast.walk(remote_tree) if isinstance(node, ast.FunctionDef)}
    required = {"bootstrap_once", "create_markers", "build_once", "parity_once", "run_physical", "consume_marker"}
    need(required <= functions, "remote executable function graph incomplete")
    remote_functions = {
        node.name: ast.get_source_segment(remote_text, node) or ""
        for node in remote_tree.body
        if isinstance(node, ast.FunctionDef)
    }
    parity_source = remote_functions["parity_once"]
    need(
        parity_source.index('stage / "PRE_PARITY.json"')
        < parity_source.index('consume_marker("PARITY"')
        < parity_source.index("subprocess.Popen"),
        "parity pre-marker lifecycle drift",
    )
    need(
        "except BaseException" in parity_source
        and '"BLOCKED_PROVENANCE" if controller_error else "BLOCKED_PARITY"' in parity_source,
        "parity failure dispatch drift",
    )
    physical_source = remote_functions["run_physical"]
    need(
        physical_source.index('stage / "PRE_ROUTE.json"')
        < physical_source.index("consume_marker(route")
        < physical_source.index("subprocess.Popen"),
        "physical pre-marker lifecycle drift",
    )
    local_tree = trees[CONTROLLER.name]
    execute_node = next(
        node for node in local_tree.body if isinstance(node, ast.FunctionDef) and node.name == "execute_once"
    )
    first_statement = execute_node.body[0]
    need(
        isinstance(first_statement, ast.Assign)
        and isinstance(first_statement.value, ast.Call)
        and isinstance(first_statement.value.func, ast.Name)
        and first_statement.value.func.id == "verify_execution_admission",
        "execution admission is not the first execution action",
    )
    need(set(COMMAND_GRAPH) == {"BOOTSTRAP", "BUILD", "READ-ONLY BUILD AUDIT", "PARITY", *ROUTE_ORDER, "TERMINAL AUDIT"}, "closed command graph drift")
    return {"compiled": [path.name for path in python_sources()], "remote_functions": sorted(required), "command_graph": {key: {"function": value[0], "external": list(value[1])} for key, value in COMMAND_GRAPH.items()}}


def verify_fragment() -> dict[str, Any]:
    text = FRAGMENT.read_text()
    required = (
        "m2_u1_advance_b",
        "m2_u1_advance_g",
        "m2_u1_advance_i",
        "m2_enumerate_lane_b",
        "m2_enumerate_lane_g",
        "m2_enumerate_lane_i",
        "v10_m2_fused_minimum_parity",
        "v10_m2_fused_minimum_physical",
        "B0-ITERATOR",
        "G0-M1-GUARDED",
        "I0-INTERLEAVED",
        "I1-INTERLEAVED",
        "G1-M1-GUARDED",
        "B1-ITERATOR",
        "subject-ready",
        "controller-disabled",
    )
    for token in required:
        need(token in text, f"M2 fragment token absent: {token}")
    need(text.count("fn m2_u1_advance_b") == 1 and text.count("fn m2_u1_advance_g") == 1 and text.count("fn m2_u1_advance_i") == 1, "candidate definition cardinality drift")
    assembled = assemble_source()
    parsed = run(["rustfmt", "--edition", "2024", "--emit", "stdout"], input_bytes=assembled, timeout=120)
    need(parsed.returncode == 0, "assembled M2 source failed rustfmt parse")
    return {"fragment": file_row(FRAGMENT), "assembled_source_sha256": sha256_bytes(assembled), "assembled_source_size_bytes": len(assembled), "production_prefix_sha256": sha256_bytes(assembled[:39_047]), "rustfmt_parse": "PASS"}


def verify_terminal_state_model() -> dict[str, Any]:
    namespace = runpy.run_path(str(TERMINAL_AUDITOR))
    validate = namespace["validate_live"]
    controller_sha = "a" * 64

    def projection(states: Sequence[str], consumed: set[str]) -> dict[str, Any]:
        markers = []
        for route in MARKER_ROUTES:
            suffix = "consumed-before-exec" if route in consumed else "available"
            markers.append(
                {
                    "name": f"{route.lower().replace('-', '_')}.{suffix}",
                    "row": {"mode": "0400"},
                    "value": {
                        "task_id": TASK_ID,
                        "transaction_id": TRANSACTION_ID,
                        "route": route,
                        "controller_sha256": controller_sha,
                        "retry_permitted": False,
                    },
                }
            )
        return {
            "hostname": "e-MEGA-MINI-M1-13th",
            "uid": 0,
            "active_subjects": [],
            "state_rows": [
                {"value": {"sequence": index, "state": state}}
                for index, state in enumerate(states)
            ],
            "markers": markers,
        }

    cases = {
        "producer-build-block": (
            ["BOOTSTRAP_CREATED_UNAUDITED", "ALL_MARKERS_AVAILABLE", "BLOCKED_BUILD"],
            {"BUILD"},
            None,
            "BLOCKED_BUILD",
            1,
        ),
        "audit-build-block": (
            ["BOOTSTRAP_CREATED_UNAUDITED", "ALL_MARKERS_AVAILABLE", "BUILD_CREATED_UNAUDITED"],
            {"BUILD"},
            "BLOCKED_BUILD",
            "BLOCKED_BUILD",
            1,
        ),
        "parity-provenance-block": (
            [
                "BOOTSTRAP_CREATED_UNAUDITED",
                "ALL_MARKERS_AVAILABLE",
                "BUILD_CREATED_UNAUDITED",
                "BLOCKED_PROVENANCE",
            ],
            {"BUILD", "PARITY"},
            "M2_BUILD_AUDITED_PARITY_ADMITTED",
            "BLOCKED_PROVENANCE",
            2,
        ),
        "full-route-pass": (
            [
                "BOOTSTRAP_CREATED_UNAUDITED",
                "ALL_MARKERS_AVAILABLE",
                "BUILD_CREATED_UNAUDITED",
                "PARITY_PASS",
                *(f"{route}_PASS" for route in ROUTE_ORDER),
            ],
            set(MARKER_ROUTES),
            "M2_BUILD_AUDITED_PARITY_ADMITTED",
            None,
            8,
        ),
    }
    checked = {}
    for name, (states, consumed, build_verdict, blocked, count) in cases.items():
        result = validate(projection(states, consumed), controller_sha, build_verdict)
        need(result["blocked_verdict"] == blocked, f"terminal model verdict drift: {name}")
        need(result["markers_consumed"] == count, f"terminal model marker drift: {name}")
        checked[name] = {
            "blocked_verdict": result["blocked_verdict"],
            "markers_consumed": result["markers_consumed"],
        }
    return {"scope": "static model only; not scientific evidence", "cases": checked}


def local_runtime_snapshot() -> dict[str, Any]:
    binaries = {}
    for path in sorted(pathlib.Path("/home/ubu/.local/bin").glob("lay*")):
        try:
            target = path.resolve(strict=True)
        except OSError:
            continue
        if target.is_file():
            binaries[path.name] = {"target": str(target), "sha256": sha256_file(target)}
    return {"installed_lay_hashes": binaries}


def self_check() -> dict[str, Any]:
    admission = verify_preflight()
    graph = verify_python_graph()
    source = verify_fragment()
    terminal_state_model = verify_terminal_state_model()
    need(not EXECUTION_ADMISSION.exists(), "execution admission exists during offline implementation pass")
    need(not IMPLEMENTATION_RECEIPT.exists(), "implementation receipt already exists")
    source_rows = {path.name: file_row(path) for path in (*python_sources(), FRAGMENT)}
    return {
        "schema": "lay.v10.e1-traversal-w1-fused-minimum-m2-controller-self-check.v2",
        "task_id": TASK_ID,
        "transaction_id": TRANSACTION_ID,
        "verdict": "M2_CONTROLLER_STATIC_SELF_CHECK_PASS",
        "admission": admission,
        "source": source,
        "sources": source_rows,
        "python_graph": graph,
        "terminal_state_model": terminal_state_model,
        "candidate_registry": ["ITERATOR_BASELINE", "M1_GUARDED_CHAIN", "INTERLEAVED_RUNNING_MIN"],
        "route_order": list(ROUTE_ORDER),
        "marker_routes": list(MARKER_ROUTES),
        "execution_admission_present": False,
        "network_access": 0,
        "remote_reads": 0,
        "remote_writes": 0,
        "markers_created": 0,
        "markers_consumed": 0,
        "cargo_invocations": 0,
        "rustc_compilations": 0,
        "perf_stat_invocations": 0,
        "perf_record_invocations": 0,
        "subject_executions": 0,
        "runtime_authority_changed": False,
    }


def seal_self_check() -> dict[str, Any]:
    check = self_check()
    runtime_before = local_runtime_snapshot()
    for path in python_sources():
        path.chmod(0o555)
    FRAGMENT.chmod(0o444)
    sealed_sources = {path.name: file_row(path) for path in (*python_sources(), FRAGMENT)}
    for name, row in sealed_sources.items():
        before = check["sources"][name]
        need(
            row["sha256"] == before["sha256"] and row["size_bytes"] == before["size_bytes"],
            f"source bytes changed while sealing: {name}",
        )
        need(
            row["mode"] == ("0444" if name.endswith(".rs.inc") else "0555"),
            f"sealed source mode drift: {name}",
        )
    runtime_after = local_runtime_snapshot()
    need(runtime_after == runtime_before, "local runtime changed during M2 implementation seal")
    receipt = {
        "schema": "lay.v10.e1-traversal-w1-fused-minimum-m2-implementation-self-check.v2",
        "task_id": TASK_ID,
        "transaction_id": TRANSACTION_ID,
        "verdict": "M2_CONTROLLER_VERIFIED_UNRUN",
        "controller_sha256": sha256_file(CONTROLLER),
        "remote_controller_sha256": sha256_file(REMOTE_CONTROLLER),
        "bootstrap_auditor_sha256": sha256_file(BOOTSTRAP_AUDITOR),
        "build_auditor_sha256": sha256_file(BUILD_AUDITOR),
        "terminal_auditor_sha256": sha256_file(TERMINAL_AUDITOR),
        "fragment_sha256": sha256_file(FRAGMENT),
        "assembled_source_sha256": ASSEMBLED_SHA256,
        "production_prefix_sha256": PRODUCTION_PREFIX_SHA256,
        "self_check": check,
        "sealed_sources": sealed_sources,
        "runtime_before": runtime_before,
        "runtime_after": runtime_after,
        "execution_admission_present": False,
        "network_access": 0,
        "remote_reads": 0,
        "remote_writes": 0,
        "markers_created": 0,
        "markers_consumed": 0,
        "cargo_invocations": 0,
        "rustc_compilations": 0,
        "perf_stat_invocations": 0,
        "perf_record_invocations": 0,
        "subject_executions": 0,
        "runtime_authority_changed": False,
        "next_action_admitted": "independent M2 execution preflight only; remote experiment remains forbidden",
    }
    write_new(IMPLEMENTATION_RECEIPT, canonical(receipt), 0o444)
    fsync_dir(IMPLEMENTATION_RECEIPT.parent)
    return {**receipt, "receipt_path": str(IMPLEMENTATION_RECEIPT), "receipt_sha256": sha256_file(IMPLEMENTATION_RECEIPT), "receipt_mode": mode_string(IMPLEMENTATION_RECEIPT)}


def verify_execution_admission() -> dict[str, Any]:
    need(IMPLEMENTATION_RECEIPT.is_file(), "sealed implementation receipt absent")
    implementation = json.loads(IMPLEMENTATION_RECEIPT.read_text())
    need(implementation.get("verdict") == "M2_CONTROLLER_VERIFIED_UNRUN", "implementation receipt verdict drift")
    need(implementation.get("controller_sha256") == sha256_file(CONTROLLER), "controller changed after seal")
    need(EXECUTION_ADMISSION.is_file(), "M2 execution admission absent; remote actions are forbidden")
    admission = json.loads(EXECUTION_ADMISSION.read_text())
    need(admission.get("verdict") == "M2_EXECUTION_ADMITTED" and admission.get("safe_to_execute") is True, "M2 execution not admitted")
    need(admission.get("task_id") == TASK_ID and admission.get("transaction_id") == TRANSACTION_ID, "execution admission namespace drift")
    for key, digest in {
        "controller_sha256": sha256_file(CONTROLLER),
        "remote_controller_sha256": sha256_file(REMOTE_CONTROLLER),
        "bootstrap_auditor_sha256": sha256_file(BOOTSTRAP_AUDITOR),
        "build_auditor_sha256": sha256_file(BUILD_AUDITOR),
        "terminal_auditor_sha256": sha256_file(TERMINAL_AUDITOR),
        "fragment_sha256": sha256_file(FRAGMENT),
        "implementation_receipt_sha256": sha256_file(IMPLEMENTATION_RECEIPT),
    }.items():
        need(admission.get(key) == digest, f"execution admission identity drift: {key}")
    return admission


def ssh(command: Sequence[str], *, timeout: float = 3_600) -> bytes:
    return run(["/usr/bin/ssh", "-i", str(SSH_IDENTITY), "-o", "BatchMode=yes", "-o", "ConnectTimeout=10", REMOTE, shlex.join(list(command))], timeout=timeout).stdout


def remote_controller_path() -> pathlib.PurePosixPath:
    return REMOTE_PARENT / "bootstrap-v1/remote-controller.py"


def remote_call(action: str, *arguments: str, timeout: float = 3_600) -> dict[str, Any]:
    raw = ssh(["/usr/bin/sudo", "-n", "/usr/bin/python3", str(remote_controller_path()), action, *arguments], timeout=timeout)
    lines = raw.decode().strip().splitlines()
    need(lines, f"empty remote response: {action}")
    return json.loads(lines[-1])


def upload_file(local: pathlib.Path, remote: pathlib.PurePosixPath) -> None:
    run(["/usr/bin/scp", "-i", str(SSH_IDENTITY), "-o", "BatchMode=yes", "-o", "ConnectTimeout=10", "-q", "-p", str(local), f"{REMOTE}:{remote}"], timeout=3_600)


def run_auditor(path: pathlib.Path) -> dict[str, Any]:
    result = run([str(path), "audit"], timeout=10_800)
    lines = result.stdout.decode().strip().splitlines()
    need(lines, f"auditor returned no receipt: {path.name}")
    return json.loads(lines[-1])


def prepare_remote_bootstrap(admission: Mapping[str, Any]) -> pathlib.Path:
    local = pathlib.Path(tempfile.mkdtemp(prefix="lay-m2-bootstrap-"))
    files = {
        "v10.rs": V10_SOURCE,
        "fragment.inc": FRAGMENT,
        "local-controller.py": CONTROLLER,
        "remote-controller.py": REMOTE_CONTROLLER,
        "bootstrap-auditor.py": BOOTSTRAP_AUDITOR,
        "build-auditor.py": BUILD_AUDITOR,
        "terminal-auditor.py": TERMINAL_AUDITOR,
    }
    for name, source in files.items():
        shutil.copyfile(source, local / name)
    payload = {
        "schema": "lay.v10.e1-traversal-w1-fused-minimum-m2-bootstrap-payload.v1",
        "task_id": TASK_ID,
        "transaction_id": TRANSACTION_ID,
        "execution_admission_verdict": admission["verdict"],
        "execution_admission_sha256": sha256_file(EXECUTION_ADMISSION),
        "files": {name: sha256_file(path) for name, path in ((name, local / name) for name in files)},
    }
    write_new(local / "PAYLOAD.json", canonical(payload))
    return local


def execute_once() -> dict[str, Any]:
    admission = verify_execution_admission()
    need(not BOOTSTRAP_AUDIT_RECEIPT.exists() and not BUILD_AUDIT_RECEIPT.exists() and not TERMINAL_AUDIT_RECEIPT.exists(), "M2 local execution evidence already exists")
    bootstrap = prepare_remote_bootstrap(admission)
    controller_sha = sha256_file(CONTROLLER)
    try:
        ssh(["/usr/bin/mkdir", "-m", "0700", str(REMOTE_CACHE)])
        run(["/usr/bin/scp", "-i", str(SSH_IDENTITY), "-o", "BatchMode=yes", "-o", "ConnectTimeout=10", "-q", "-p", "-r", f"{bootstrap}/.", f"{REMOTE}:{REMOTE_CACHE}"], timeout=3_600)
        raw = ssh(["/usr/bin/sudo", "-n", "/usr/bin/python3", str(REMOTE_CACHE / "remote-controller.py"), "bootstrap", "--bootstrap", str(REMOTE_CACHE)], timeout=3_600)
        need(json.loads(raw.decode().strip().splitlines()[-1]).get("verdict") == "M2_BOOTSTRAP_CREATED_UNAUDITED", "remote bootstrap failed")
        bootstrap_audit = run_auditor(BOOTSTRAP_AUDITOR)
        need(bootstrap_audit.get("verdict") == "M2_BOOTSTRAP_AUDIT_PASS_MARKERS_ADMITTED", "bootstrap audit blocked")
        remote_bootstrap_audit = REMOTE_CACHE / "M2_BOOTSTRAP_AUDIT_RECEIPT.json"
        upload_file(BOOTSTRAP_AUDIT_RECEIPT, remote_bootstrap_audit)
        remote_call("create-markers", "--admission", str(remote_bootstrap_audit))
        build = remote_call("build-once", "--controller-sha256", controller_sha, timeout=10_800)
        if build.get("verdict") == "BLOCKED_BUILD":
            return run_auditor(TERMINAL_AUDITOR)
        need(build.get("verdict") == "M2_BUILD_CREATED_UNAUDITED", "build producer verdict drift")
        build_audit = run_auditor(BUILD_AUDITOR)
        if build_audit.get("verdict") in {"BLOCKED_BUILD", "BLOCKED_PROVENANCE"}:
            return run_auditor(TERMINAL_AUDITOR)
        need(build_audit.get("verdict") == "M2_BUILD_AUDITED_PARITY_ADMITTED", "build audit verdict drift")
        remote_build_audit = REMOTE_CACHE / "M2_BUILD_AUDIT_RECEIPT.json"
        upload_file(BUILD_AUDIT_RECEIPT, remote_build_audit)
        parity = remote_call("parity-once", "--controller-sha256", controller_sha, "--admission", str(remote_build_audit), timeout=10_800)
        if parity.get("verdict") == "PASS":
            for route in ROUTE_ORDER:
                response = remote_call("run-route", "--controller-sha256", controller_sha, "--route", route, timeout=10_800)
                if response.get("verdict") != "PASS_UNAUDITED":
                    break
        terminal = run_auditor(TERMINAL_AUDITOR)
        return terminal
    finally:
        shutil.rmtree(bootstrap)


def status() -> dict[str, Any]:
    return {
        "task_id": TASK_ID,
        "transaction_id": TRANSACTION_ID,
        "implementation_receipt": file_row(IMPLEMENTATION_RECEIPT) if IMPLEMENTATION_RECEIPT.is_file() else None,
        "execution_admission_present": EXECUTION_ADMISSION.is_file(),
        "bootstrap_audit_present": BOOTSTRAP_AUDIT_RECEIPT.is_file(),
        "build_audit_present": BUILD_AUDIT_RECEIPT.is_file(),
        "terminal_audit_present": TERMINAL_AUDIT_RECEIPT.is_file(),
        "remote_reads": 0,
        "remote_writes": 0,
    }


def parser() -> argparse.ArgumentParser:
    value = argparse.ArgumentParser()
    value.add_argument("action", choices=("self-check", "seal-self-check", "status", "run"))
    return value


def main() -> int:
    arguments = parser().parse_args()
    try:
        if arguments.action == "self-check":
            value = self_check()
        elif arguments.action == "seal-self-check":
            value = seal_self_check()
        elif arguments.action == "status":
            value = status()
        else:
            value = execute_once()
        print(json.dumps(value, ensure_ascii=False, sort_keys=True))
        return 0
    except Exception as error:
        print(f"M2 CONTROLLER ERROR: {type(error).__name__}: {error}", file=sys.stderr)
        return 1


if __name__ == "__main__":
    raise SystemExit(main())
