#!/usr/bin/env python3
"""Fault-closed local controller for the M2R1 fused-minimum repair experiment."""

from __future__ import annotations

import argparse
import ast
import difflib
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
TASK_ID = "slice8b-v10-e1-traversal-w1-fused-minimum-m2r1-v1-20260827"
TRANSACTION_ID = "2dae728a39aecd422995828674d12e311ab6362ebab4013c4f2520b3f6933c5f"
REMOTE = "e@192.168.3.94"
SSH_IDENTITY = pathlib.Path("/home/ubu/.ssh/mega-mini-admin")
REMOTE_CACHE = pathlib.PurePosixPath("/home/e/.cache") / f"lay-m2r1-{TRANSACTION_ID}"
REMOTE_PARENT = pathlib.PurePosixPath("/home/e/.local/share/lay/provenance") / TASK_ID
CONTROLLER = pathlib.Path(__file__).resolve()
M2_PREDECESSOR_CONTROLLER = ROOT / "scripts/lay-v10-e1-traversal-w1-fused-minimum-m2-v3.py"
REMOTE_CONTROLLER = ROOT / "scripts/lay-v10-e1-traversal-w1-fused-minimum-m2r1-remote.py"
BOOTSTRAP_AUDITOR = ROOT / "scripts/lay-v10-e1-traversal-w1-fused-minimum-m2r1-bootstrap-audit.py"
BUILD_AUDITOR = ROOT / "scripts/lay-v10-e1-traversal-w1-fused-minimum-m2r1-build-audit.py"
TERMINAL_AUDITOR = ROOT / "scripts/lay-v10-e1-traversal-w1-fused-minimum-m2r1-terminal-audit.py"
FRAGMENT = ROOT / "scripts/lay_v10_e1_traversal_w1_fused_minimum_m2r1_test_module.rs.inc"
M2_FRAGMENT = ROOT / "scripts/lay_v10_e1_traversal_w1_fused_minimum_m2_test_module.rs.inc"
LOCAL_COMPILE_PROOF = ROOT / (
    "docs/structural_gates/receipts/"
    "LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_W1_FUSED_MINIMUM_M2R1_"
    "LOCAL_COMPILE_PROOF_V1_2026-08-27.json"
)
V10_SOURCE = pathlib.Path(
    "/home/ubu/.local/share/lay/provenance/"
    "slice8b-v10-f6178f/artifacts/v13_typed_peak.v10.rs"
)
PREFLIGHT = ROOT / (
    "docs/structural_gates/preflights/"
    "LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_W1_FUSED_MINIMUM_M2R1_"
    "IMPLEMENTATION_V2_2026-08-27.json"
)
PREFLIGHT_RECEIPT = ROOT / (
    "docs/structural_gates/receipts/"
    "LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_W1_FUSED_MINIMUM_M2R1_"
    "IMPLEMENTATION_V2_PREFLIGHT_2026-08-27.json"
)
IMPLEMENTATION_RECEIPT = ROOT / (
    "docs/structural_gates/receipts/"
    "LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_W1_FUSED_MINIMUM_M2R1_"
    "IMPLEMENTATION_SELF_CHECK_V1_2026-08-27.json"
)
EXECUTION_ADMISSION = ROOT / (
    "docs/structural_gates/preflights/"
    "LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_W1_FUSED_MINIMUM_M2R1_"
    "EXECUTION_ADMISSION_V1_2026-08-27.json"
)
BOOTSTRAP_AUDIT_RECEIPT = ROOT / (
    "docs/structural_gates/receipts/"
    "LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_W1_FUSED_MINIMUM_M2R1_"
    "BOOTSTRAP_AUDIT_V1_2026-08-27/M2R1_BOOTSTRAP_AUDIT_RECEIPT.json"
)
BUILD_AUDIT_RECEIPT = ROOT / (
    "docs/structural_gates/receipts/"
    "LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_W1_FUSED_MINIMUM_M2R1_"
    "BUILD_AUDIT_V1_2026-08-27/M2R1_BUILD_AUDIT_RECEIPT.json"
)
TERMINAL_AUDIT_RECEIPT = ROOT / (
    "docs/structural_gates/receipts/"
    "LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_W1_FUSED_MINIMUM_M2R1_"
    "TERMINAL_AUDIT_V1_2026-08-27/M2R1_TERMINAL_AUDIT_RECEIPT.json"
)
EXECUTION_JOURNAL = ROOT / (
    "docs/structural_gates/receipts/"
    "LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_W1_FUSED_MINIMUM_M2R1_"
    "EXECUTION_JOURNAL_V1_2026-08-27"
)
CONTROLLER_FAILURE = ROOT / (
    "docs/structural_gates/receipts/"
    "LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_W1_FUSED_MINIMUM_M2R1_"
    "CONTROLLER_FAILURE_V1_2026-08-27"
)
PREFLIGHT_SHA256 = "42b8ba9c75c49d4473dea244eed32f18100dda2fc714c7a2353086d28d23f563"
PREFLIGHT_RECEIPT_SHA256 = "4f304bcac3f5e7f1304946122fc29a3cf9819b12c63f3288af9abfcf52c29dee"
FRAGMENT_SHA256 = "a6ea388d5d76f8223511fd4822cff2df9fd0c3394fc200f4fc52db956522ce5b"
ASSEMBLED_SHA256 = "b2b44ed1b42e79dcdb9b7dcfb2753eec547fa7d1e26b5fb63772a706f786ccd5"
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
MUTABLE_BASELINE_IDS: set[str] = set()
COMMAND_GRAPH = {
    "BOOTSTRAP": ("bootstrap_once", ()),
    "BUILD": ("build_once", ("cargo-guard",)),
    "READ-ONLY BUILD AUDIT": ("audit", ("readelf", "nm")),
    "PARITY": ("parity_once", ("diagnostic-test-elf",)),
    **{route: ("run_physical", ("perf stat", "diagnostic-test-elf")) for route in ROUTE_ORDER},
    "TERMINAL AUDIT": ("audit", ()),
}
ACTION_SPECS = (
    ("remote-cache-create", "execution-admission", "remote cache mutation", None),
    ("bootstrap-upload", "remote-cache-create", "remote bootstrap payload write", None),
    ("remote-bootstrap", "bootstrap-upload", "remote namespace bootstrap mutation", None),
    ("bootstrap-audit", "remote-bootstrap", "read-only bootstrap audit", None),
    ("bootstrap-audit-upload", "bootstrap-audit", "remote admission receipt write", None),
    ("create-markers", "bootstrap-audit-upload", "atomic eight-marker creation", None),
    ("build", "create-markers", "BUILD one-shot producer", "BUILD"),
    ("build-audit", "build", "read-only build audit", None),
    ("build-audit-upload", "build-audit", "remote build-audit receipt write", None),
    ("parity", "build-audit-upload", "PARITY one-shot producer", "PARITY"),
    *( 
        (
            f"physical-{route.lower().replace('-', '_')}",
            "parity" if index == 0 else f"physical-{ROUTE_ORDER[index - 1].lower().replace('-', '_')}",
            "physical perf-stat one-shot producer",
            route,
        )
        for index, route in enumerate(ROUTE_ORDER)
    ),
    ("terminal-audit", "last completed producer or auditor", "read-only terminal audit", None),
)
ACTION_REGISTRY = {
    action_id: {
        "registry_sequence": index,
        "expected_predecessor": predecessor,
        "scope": scope,
        "route": route,
    }
    for index, (action_id, predecessor, scope, route) in enumerate(ACTION_SPECS)
}
UNKNOWN = "UNKNOWN"


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


def write_json(path: pathlib.Path, value: Any, mode: int = 0o444) -> None:
    write_new(path, canonical(value), mode)


def atomic_publish_file(
    path: pathlib.Path,
    value: bytes,
    mode: int = 0o444,
    before_rename: Any | None = None,
) -> None:
    need(not path.exists(), f"immutable publication already exists: {path}")
    path.parent.mkdir(parents=True, exist_ok=True)
    stage = path.parent / f".{path.name}.stage-{os.getpid()}-{time.time_ns()}"
    try:
        write_new(stage, value, mode)
        fsync_dir(path.parent)
        if before_rename is not None:
            before_rename()
        os.rename(stage, path)
        fsync_dir(path.parent)
    except BaseException:
        if stage.exists():
            stage.unlink()
            fsync_dir(path.parent)
        raise


def write_manifest(root: pathlib.Path) -> None:
    rows = [
        f"{sha256_file(path)}  {path.relative_to(root).as_posix()}\n"
        for path in sorted(root.rglob("*"))
        if path.is_file() and path.name != "SHA256SUMS"
    ]
    write_new(root / "SHA256SUMS", "".join(rows).encode(), 0o444)


def seal_tree(root: pathlib.Path) -> None:
    for path in sorted(root.rglob("*"), reverse=True):
        path.chmod(0o555 if path.is_dir() else 0o444)
    root.chmod(0o555)


def journal_metadata(admission: Mapping[str, Any]) -> dict[str, Any]:
    return {
        "schema": "lay.v10.e1-traversal-w1-fused-minimum-m2r1-execution-journal.v1",
        "task_id": TASK_ID,
        "transaction_id": TRANSACTION_ID,
        "controller_sha256": sha256_file(CONTROLLER),
        "remote_controller_sha256": sha256_file(REMOTE_CONTROLLER),
        "bootstrap_auditor_sha256": sha256_file(BOOTSTRAP_AUDITOR),
        "build_auditor_sha256": sha256_file(BUILD_AUDITOR),
        "terminal_auditor_sha256": sha256_file(TERMINAL_AUDITOR),
        "implementation_receipt_sha256": sha256_file(IMPLEMENTATION_RECEIPT),
        "execution_admission_sha256": sha256_file(EXECUTION_ADMISSION),
        "execution_admission_verdict": admission.get("verdict"),
        "action_registry": ACTION_REGISTRY,
        "retry_permitted": False,
        "remote_execution_started": False,
    }


def initialize_execution_journal(root: pathlib.Path, metadata: Mapping[str, Any]) -> pathlib.Path:
    need(not root.exists(), f"execution journal already exists: {root}")
    root.parent.mkdir(parents=True, exist_ok=True)
    stage = pathlib.Path(f"{root}.stage-{os.getpid()}-{time.time_ns()}")
    stage.mkdir(mode=0o700)
    try:
        events = stage / "events"
        events.mkdir(mode=0o700)
        write_json(stage / "JOURNAL.json", dict(metadata))
        fsync_dir(events)
        fsync_dir(stage)
        os.rename(stage, root)
        fsync_dir(root.parent)
    except BaseException:
        if stage.exists():
            shutil.rmtree(stage)
        raise
    return root


def journal_rows(root: pathlib.Path, prefix: str) -> list[pathlib.Path]:
    return sorted((root / "events").glob(f"{prefix}-*.json"))


def pending_intent(root: pathlib.Path) -> dict[str, Any] | None:
    intents = journal_rows(root, "INTENT")
    completions = journal_rows(root, "COMPLETE")
    completed_sequences = {int(path.name.split("-", 2)[1]) for path in completions}
    pending = [path for path in intents if int(path.name.split("-", 2)[1]) not in completed_sequences]
    need(len(pending) <= 1, "multiple uncompleted journal intents")
    return json.loads(pending[0].read_text()) if pending else None


def append_intent(root: pathlib.Path, action_id: str) -> dict[str, Any]:
    need(action_id in ACTION_REGISTRY, f"unregistered external action: {action_id}")
    need(pending_intent(root) is None, "uncompleted intent forbids retry or resume")
    sequence = len(journal_rows(root, "INTENT"))
    spec = ACTION_REGISTRY[action_id]
    intent = {
        "schema": "lay.v10.e1-traversal-w1-fused-minimum-m2r1-action-intent.v1",
        "task_id": TASK_ID,
        "transaction_id": TRANSACTION_ID,
        "sequence": sequence,
        "registry_sequence": spec["registry_sequence"],
        "action_id": action_id,
        "expected_predecessor": spec["expected_predecessor"],
        "expected_scope": spec["scope"],
        "one_shot_route": spec["route"],
        "failure_default": "BLOCKED_PROVENANCE",
        "retry_permitted": False,
    }
    path = root / "events" / f"INTENT-{sequence:02d}-{action_id}.json"
    write_json(path, intent)
    fsync_dir(path.parent)
    return intent


def append_completion(root: pathlib.Path, intent: Mapping[str, Any], response: Mapping[str, Any]) -> dict[str, Any]:
    sequence = int(intent["sequence"])
    completion = {
        "schema": "lay.v10.e1-traversal-w1-fused-minimum-m2r1-action-completion.v1",
        "task_id": TASK_ID,
        "transaction_id": TRANSACTION_ID,
        "sequence": sequence,
        "action_id": intent["action_id"],
        "response": dict(response),
        "response_sha256": sha256_bytes(canonical(response)),
        "retry_permitted": False,
    }
    path = root / "events" / f"COMPLETE-{sequence:02d}-{intent['action_id']}.json"
    write_json(path, completion)
    fsync_dir(path.parent)
    return completion


def run_journaled(
    root: pathlib.Path,
    action_id: str,
    callable_action: Any,
) -> dict[str, Any]:
    intent = append_intent(root, action_id)
    response = callable_action()
    need(isinstance(response, Mapping), f"external action returned no structured response: {action_id}")
    value = dict(response)
    append_completion(root, intent, value)
    return value


def initial_execution_knowledge() -> dict[str, Any]:
    return {
        "remote_state": UNKNOWN,
        "marker_state": UNKNOWN,
        "cargo_invocations": 0,
        "perf_stat_invocations": 0,
        "subject_executions": 0,
    }


def consume_known_marker(value: dict[str, Any], route: str) -> None:
    markers = value.get("marker_state")
    if not isinstance(markers, dict):
        value["marker_state"] = UNKNOWN
        return
    available = set(markers.get("available", []))
    consumed = set(markers.get("consumed", []))
    name = route.lower().replace("-", "_")
    available.discard(f"{name}.available")
    consumed.add(f"{name}.consumed-before-exec")
    value["marker_state"] = {"available": sorted(available), "consumed": sorted(consumed)}


def update_execution_knowledge(value: dict[str, Any], action_id: str, response: Mapping[str, Any]) -> None:
    verdict = response.get("verdict")
    if action_id in {"remote-bootstrap", "create-markers", "build", "parity"} or action_id.startswith("physical-"):
        value["remote_state"] = {"last_completed_action": action_id, "verdict": verdict}
    if action_id == "remote-bootstrap":
        value.update(
            marker_state={"available": [], "consumed": []},
            cargo_invocations=0,
            perf_stat_invocations=0,
            subject_executions=0,
        )
    elif action_id == "create-markers":
        markers = response.get("markers")
        value["marker_state"] = dict(markers) if isinstance(markers, Mapping) else UNKNOWN
    elif action_id == "build":
        consume_known_marker(value, "BUILD")
        cargo = response.get("cargo_invocations")
        if cargo is None and isinstance(response.get("build"), Mapping):
            cargo = response["build"].get("cargo_invocations")
        value["cargo_invocations"] = cargo if isinstance(cargo, int) else UNKNOWN
    elif action_id == "parity":
        consume_known_marker(value, "PARITY")
        count = response.get("subject_executions")
        value["subject_executions"] = count if isinstance(count, int) else UNKNOWN
    elif action_id.startswith("physical-"):
        route = str(response.get("route", ACTION_REGISTRY[action_id]["route"]))
        consume_known_marker(value, route)
        perf = response.get("perf_stat_invocations")
        subjects = response.get("subject_executions")
        if isinstance(value.get("perf_stat_invocations"), int) and isinstance(perf, int):
            value["perf_stat_invocations"] += perf
        else:
            value["perf_stat_invocations"] = UNKNOWN
        if isinstance(value.get("subject_executions"), int) and isinstance(subjects, int):
            value["subject_executions"] += subjects
        else:
            value["subject_executions"] = UNKNOWN


def failure_knowledge(action_id: str, known: Mapping[str, Any]) -> dict[str, Any]:
    value = dict(known)
    affected = {"remote_state"}
    if action_id == "create-markers":
        affected.add("marker_state")
    elif action_id == "build":
        affected.update(("marker_state", "cargo_invocations"))
    elif action_id == "parity":
        affected.update(("marker_state", "subject_executions"))
    elif action_id.startswith("physical-"):
        affected.update(("marker_state", "perf_stat_invocations", "subject_executions"))
    for key in affected:
        value[key] = UNKNOWN
    return value


def controller_failure_payload(
    intent: Mapping[str, Any],
    error: BaseException,
    known: Mapping[str, Any],
) -> dict[str, Any]:
    projection = failure_knowledge(str(intent["action_id"]), known)
    return {
        "schema": "lay.v10.e1-traversal-w1-fused-minimum-m2r1-controller-failure.v1",
        "task_id": TASK_ID,
        "transaction_id": TRANSACTION_ID,
        "verdict": "BLOCKED_PROVENANCE",
        "failed_action": dict(intent),
        "error": f"{type(error).__name__}: {error}",
        **projection,
        "retry_permitted": False,
        "runtime_authority_changed": False,
        "next_action_admitted": "independent read-only recovery audit only",
    }


def publish_controller_failure(
    journal: pathlib.Path,
    intent: Mapping[str, Any],
    error: BaseException,
    known: Mapping[str, Any],
) -> dict[str, Any]:
    need(not CONTROLLER_FAILURE.exists(), "controller failure receipt already exists")
    receipt = controller_failure_payload(intent, error, known)
    stage = pathlib.Path(f"{CONTROLLER_FAILURE}.stage-{os.getpid()}-{time.time_ns()}")
    stage.mkdir(parents=True, mode=0o700)
    try:
        write_json(stage / "M2R1_CONTROLLER_FAILURE_RECEIPT.json", receipt)
        write_json(
            stage / "JOURNAL_SNAPSHOT.json",
            {
                "journal": file_row(journal / "JOURNAL.json"),
                "events": [file_row(path) for path in sorted((journal / "events").glob("*.json"))],
            },
        )
        write_new(stage / "controller.py", CONTROLLER.read_bytes(), 0o444)
        write_manifest(stage)
        seal_tree(stage)
        os.rename(stage, CONTROLLER_FAILURE)
        fsync_dir(CONTROLLER_FAILURE.parent)
    except BaseException:
        if stage.exists():
            shutil.rmtree(stage)
        raise
    return {
        **receipt,
        "receipt_sha256": sha256_file(CONTROLLER_FAILURE / "M2R1_CONTROLLER_FAILURE_RECEIPT.json"),
    }


def finalize_execution_journal(root: pathlib.Path, terminal: Mapping[str, Any]) -> None:
    write_json(
        root / "FINAL.json",
        {
            "schema": "lay.v10.e1-traversal-w1-fused-minimum-m2r1-execution-journal-final.v1",
            "task_id": TASK_ID,
            "transaction_id": TRANSACTION_ID,
            "terminal_verdict": terminal.get("verdict"),
            "terminal_receipt_sha256": terminal.get("receipt_sha256"),
            "retry_permitted": False,
        },
    )
    write_manifest(root)
    seal_tree(root)
    fsync_dir(root.parent)


def run(command: Sequence[str], *, input_bytes: bytes | None = None, timeout: float = 3_600, check: bool = True) -> subprocess.CompletedProcess[bytes]:
    result = subprocess.run(list(command), input=input_bytes, stdout=subprocess.PIPE, stderr=subprocess.PIPE, timeout=timeout, check=False)
    if check and result.returncode != 0:
        raise M2ControllerError(f"command failed ({result.returncode}): {shlex.join(command)}\n{result.stderr.decode(errors='replace')[-5000:]}")
    return result


def assemble_source() -> bytes:
    v10 = V10_SOURCE.read_bytes()
    fragment = FRAGMENT.read_bytes()
    need(len(v10) == 91_518 and sha256_bytes(v10) == V10_SHA256 and v10.endswith(b"}\n"), "V10 source drift")
    need(len(fragment) == 156_122 and sha256_bytes(fragment) == FRAGMENT_SHA256, "M2R1 fragment drift")
    need(sha256_bytes(fragment[:113_204]) == D1_FRAGMENT_SHA256, "D1 fragment prefix drift")
    assembled = v10[:-2] + fragment + b"}\n"
    need(len(assembled) == 247_640 and sha256_bytes(assembled) == ASSEMBLED_SHA256, "assembled source drift")
    need(sha256_bytes(assembled[:39_047]) == PRODUCTION_PREFIX_SHA256, "production prefix drift")
    return assembled


def verify_preflight() -> dict[str, Any]:
    preflight_row = require_file(PREFLIGHT, digest=PREFLIGHT_SHA256, size=11_670, mode="0444")
    receipt_row = require_file(PREFLIGHT_RECEIPT, digest=PREFLIGHT_RECEIPT_SHA256, size=6_930, mode="0444")
    preflight = json.loads(PREFLIGHT.read_text())
    receipt = json.loads(PREFLIGHT_RECEIPT.read_text())
    need(receipt.get("verdict") == "READY_TO_IMPLEMENT" and receipt.get("safe_to_implement") is True and not receipt.get("blockers"), "M2 implementation preflight is not ready")
    need(
        receipt.get("task_id")
        == "LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_W1_FUSED_MINIMUM_M2R1_IMPLEMENTATION_V2_2026-08-27",
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
    need(len(baseline) == 13, f"M2R1 pinned file closure drift: {len(baseline)}")
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
    for auditor in (BOOTSTRAP_AUDITOR, BUILD_AUDITOR, TERMINAL_AUDITOR):
        auditor_text = auditor.read_text()
        need("IMPLEMENTATION_SELF_CHECK_V1_2026-08-27.json" in auditor_text, f"M2R1 receipt pin absent: {auditor.name}")
        need("lay-v10-e1-traversal-w1-fused-minimum-m2r1-v1.py" in auditor_text, f"M2R1 controller pin absent: {auditor.name}")
    require_file(
        M2_PREDECESSOR_CONTROLLER,
        digest="4f2f6484d7ada483688b59a2403354548d4aaffdc594074c4447326c7b8c1f7f",
        size=65_528,
        mode="0555",
    )
    require_file(
        REMOTE_CONTROLLER,
        digest="26bfbbf2a9626b36bad549d6827d7442d1d1b5db7ced0f65da41c099a793de09",
        size=40_141,
        mode="0555",
    )
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
    local_functions = {
        node.name: ast.get_source_segment(CONTROLLER.read_text(), node) or ""
        for node in local_tree.body
        if isinstance(node, ast.FunctionDef)
    }
    journaled_source = local_functions["run_journaled"]
    need(
        journaled_source.index("append_intent")
        < journaled_source.index("callable_action()")
        < journaled_source.index("append_completion"),
        "journal intent/callable/completion order drift",
    )
    execute_source = local_functions["execute_once"]
    need(
        execute_source.index("create_execution_journal") < execute_source.index("run_checked_action"),
        "execution journal is not durable before the first external action",
    )
    execute_parents = {
        child: parent
        for parent in ast.walk(execute_node)
        for child in ast.iter_child_nodes(parent)
    }
    externally_effectful = {"create_remote_cache", "upload_bootstrap", "remote_bootstrap", "run_auditor", "upload_with_receipt", "remote_call"}
    wrapped_calls = []
    for node in ast.walk(execute_node):
        if not isinstance(node, ast.Call) or not isinstance(node.func, ast.Name):
            continue
        if node.func.id not in externally_effectful:
            continue
        parent = execute_parents.get(node)
        wrapped = False
        while parent is not None and parent is not execute_node:
            if (
                isinstance(parent, ast.Call)
                and isinstance(parent.func, ast.Name)
                and parent.func.id == "run_checked_action"
            ):
                wrapped = True
                break
            parent = execute_parents.get(parent)
        need(wrapped, f"external action bypasses journal wrapper: {node.func.id}")
        wrapped_calls.append(node.func.id)
    need(
        sorted(wrapped_calls)
        == sorted(
            [
                "upload_bootstrap",
                "run_auditor",
                "upload_with_receipt",
                "remote_call",
                "remote_call",
                "run_auditor",
                "run_auditor",
                "upload_with_receipt",
                "remote_call",
                "remote_call",
                "run_auditor",
                "run_auditor",
            ]
        ),
        "external call-site inventory drift",
    )
    checked_calls = [
        node
        for node in ast.walk(execute_node)
        if isinstance(node, ast.Call)
        and isinstance(node.func, ast.Name)
        and node.func.id == "run_checked_action"
    ]
    constant_actions = {
        node.args[2].value: node
        for node in checked_calls
        if len(node.args) >= 4 and isinstance(node.args[2], ast.Constant) and isinstance(node.args[2].value, str)
    }
    for action_id, callable_name in {
        "remote-cache-create": "create_remote_cache",
        "remote-bootstrap": "remote_bootstrap",
    }.items():
        node = constant_actions.get(action_id)
        need(
            node is not None and isinstance(node.args[3], ast.Name) and node.args[3].id == callable_name,
            f"bare external callable bypass or identity drift: {action_id}",
        )
    need(
        set(ACTION_REGISTRY) - {f"physical-{route.lower().replace('-', '_')}" for route in ROUTE_ORDER}
        <= set(constant_actions),
        "constant external action call-site closure drift",
    )
    checked_source = local_functions["run_checked_action"]
    need(
        checked_source.index("checked_response")
        < checked_source.index("update_execution_knowledge")
        < checked_source.index("run_journaled"),
        "checked external response is not journal-bound",
    )
    failure_source = local_functions["publish_controller_failure"]
    need(
        failure_source.index("write_json")
        < failure_source.index("write_manifest")
        < failure_source.index("seal_tree")
        < failure_source.index("os.rename"),
        "controller-failure publication is not atomic and seal-first",
    )
    atomic_source = local_functions["atomic_publish_file"]
    seal_source = local_functions["seal_self_check"]
    need(
        atomic_source.index("write_new")
        < atomic_source.index("before_rename()")
        < atomic_source.index("os.rename")
        < atomic_source.rindex("fsync_dir"),
        "immutable file publication order drift",
    )
    need("atomic_publish_file(IMPLEMENTATION_RECEIPT" in seal_source, "implementation receipt bypasses atomic publisher")
    expected_actions = (
        "remote-cache-create",
        "bootstrap-upload",
        "remote-bootstrap",
        "bootstrap-audit",
        "bootstrap-audit-upload",
        "create-markers",
        "build",
        "build-audit",
        "build-audit-upload",
        "parity",
        *(f"physical-{route.lower().replace('-', '_')}" for route in ROUTE_ORDER),
        "terminal-audit",
    )
    need(tuple(ACTION_REGISTRY) == expected_actions, "external action registry drift")
    need(set(COMMAND_GRAPH) == {"BOOTSTRAP", "BUILD", "READ-ONLY BUILD AUDIT", "PARITY", *ROUTE_ORDER, "TERMINAL AUDIT"}, "closed command graph drift")
    return {
        "compiled": [path.name for path in python_sources()],
        "remote_functions": sorted(required),
        "command_graph": {key: {"function": value[0], "external": list(value[1])} for key, value in COMMAND_GRAPH.items()},
        "external_action_registry": list(ACTION_REGISTRY),
        "journal_order": ["intent", "callable", "completion"],
        "failure_publication": "stage-manifest-seal-atomic-rename",
        "journal_wrapped_external_call_sites": wrapped_calls,
    }


def verify_fragment() -> dict[str, Any]:
    text = FRAGMENT.read_text()
    predecessor_text = M2_FRAGMENT.read_text()
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
    delta = "".join(
        difflib.unified_diff(
            predecessor_text.splitlines(keepends=True),
            text.splitlines(keepends=True),
            fromfile="m2",
            tofile="m2r1",
            n=3,
        )
    ).encode()
    need(
        len(delta) == 900
        and sha256_bytes(delta) == "e8245121c155da5e22bb32dc63564be892e497c9e6d49868a66b2f2d525cd078",
        "M2R1 fragment delta exceeds the frozen result-signature repair",
    )
    compile_proof = json.loads(LOCAL_COMPILE_PROOF.read_text())
    need(
        compile_proof.get("verdict") == "M2R1_LOCAL_COMPILE_PROOF_PASS"
        and compile_proof.get("assembled_source", {}).get("sha256") == ASSEMBLED_SHA256
        and compile_proof.get("cargo", {}).get("exit_code") == 0
        and compile_proof.get("markers_created") == 0
        and compile_proof.get("markers_consumed") == 0
        and compile_proof.get("disposable_elf", {}).get("retained") is False,
        "M2R1 local compile proof drift",
    )
    assembled = assemble_source()
    parsed = run(["rustfmt", "--edition", "2024", "--emit", "stdout"], input_bytes=assembled, timeout=120)
    need(parsed.returncode == 0, "assembled M2 source failed rustfmt parse")
    return {
        "fragment": file_row(FRAGMENT),
        "predecessor_fragment": file_row(M2_FRAGMENT),
        "fragment_delta_sha256": sha256_bytes(delta),
        "fragment_delta_size_bytes": len(delta),
        "local_compile_proof": file_row(LOCAL_COMPILE_PROOF),
        "assembled_source_sha256": sha256_bytes(assembled),
        "assembled_source_size_bytes": len(assembled),
        "production_prefix_sha256": sha256_bytes(assembled[:39_047]),
        "rustfmt_parse": "PASS",
    }


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


def fault_response(action_id: str) -> dict[str, Any]:
    if action_id == "remote-bootstrap":
        return {
            "verdict": "M2_BOOTSTRAP_CREATED_UNAUDITED",
            "markers_created": 0,
            "markers_consumed": 0,
            "cargo_invocations": 0,
            "perf_stat_invocations": 0,
            "subject_executions": 0,
        }
    if action_id == "create-markers":
        return {
            "verdict": "M2_ALL_MARKERS_AVAILABLE",
            "markers": {
                "available": [f"{route.lower().replace('-', '_')}.available" for route in MARKER_ROUTES],
                "consumed": [],
            },
            "markers_created": 8,
            "markers_consumed": 0,
        }
    if action_id == "build":
        return {"verdict": "M2_BUILD_CREATED_UNAUDITED", "build": {"cargo_invocations": 1}}
    if action_id == "parity":
        return {"verdict": "PASS", "subject_executions": 1, "perf_stat_invocations": 0}
    if action_id.startswith("physical-"):
        return {
            "verdict": "PASS_UNAUDITED",
            "route": ACTION_REGISTRY[action_id]["route"],
            "perf_stat_invocations": 1,
            "subject_executions": 1,
        }
    verdicts = {
        "remote-cache-create": "REMOTE_CACHE_CREATED",
        "bootstrap-upload": "BOOTSTRAP_PAYLOAD_UPLOADED",
        "bootstrap-audit": "M2_BOOTSTRAP_AUDIT_PASS_MARKERS_ADMITTED",
        "bootstrap-audit-upload": "BOOTSTRAP_AUDIT_UPLOADED",
        "build-audit": "M2_BUILD_AUDITED_PARITY_ADMITTED",
        "build-audit-upload": "BUILD_AUDIT_UPLOADED",
        "terminal-audit": "W1_FUSED_MINIMUM_MECHANISM_REJECTED",
    }
    return {"verdict": verdicts[action_id]}


def verify_failure_publication_model() -> dict[str, Any]:
    checked: dict[str, Any] = {}
    action_ids = tuple(ACTION_REGISTRY)
    need(len(action_ids) == 17 and len(set(action_ids)) == 17, "external action registry cardinality drift")
    for failed_index, failed_action in enumerate(action_ids):
        with tempfile.TemporaryDirectory(prefix="lay-m2-v3-fault-") as raw:
            journal = initialize_execution_journal(
                pathlib.Path(raw) / "journal",
                {
                    "schema": "lay.v10.e1-traversal-w1-fused-minimum-m2r1-fault-model.v1",
                    "task_id": TASK_ID,
                    "transaction_id": TRANSACTION_ID,
                    "retry_permitted": False,
                },
            )
            known = initial_execution_knowledge()
            invoked: list[str] = []
            terminal: list[dict[str, Any]] = []
            for action_id in action_ids:
                def invoke(current: str = action_id) -> dict[str, Any]:
                    active = pending_intent(journal)
                    need(active is not None and active["action_id"] == current, "intent was not durable before callable")
                    need(
                        not any(
                            int(path.name.split("-", 2)[1]) == active["sequence"]
                            for path in journal_rows(journal, "COMPLETE")
                        ),
                        "completion preceded callable",
                    )
                    invoked.append(current)
                    if current == failed_action:
                        raise RuntimeError(f"injected failure at {current}")
                    return fault_response(current)

                try:
                    response = run_journaled(journal, action_id, invoke)
                    update_execution_knowledge(known, action_id, response)
                except RuntimeError as error:
                    active = pending_intent(journal)
                    need(active is not None and active["action_id"] == failed_action, "failed intent identity drift")
                    terminal.append(controller_failure_payload(active, error, known))
                    break
            need(invoked == list(action_ids[: failed_index + 1]), f"later action ran after fault: {failed_action}")
            need(len(terminal) == 1, f"terminal observation cardinality drift: {failed_action}")
            receipt = terminal[0]
            need(
                receipt["verdict"] == "BLOCKED_PROVENANCE"
                and receipt["retry_permitted"] is False
                and receipt["failed_action"]["action_id"] == failed_action,
                f"failure schema drift: {failed_action}",
            )
            need(pending_intent(journal) is not None, f"failed intent was not retained: {failed_action}")
            try:
                append_intent(journal, failed_action)
            except M2ControllerError:
                retry_blocked = True
            else:
                retry_blocked = False
            need(retry_blocked, f"retry remained possible after intent: {failed_action}")
            checked[failed_action] = {
                "invocations_before_stop": len(invoked),
                "terminal_observations": len(terminal),
                "retry_permitted": receipt["retry_permitted"],
                "remote_state": receipt["remote_state"],
                "marker_state": receipt["marker_state"],
                "cargo_invocations": receipt["cargo_invocations"],
                "perf_stat_invocations": receipt["perf_stat_invocations"],
                "subject_executions": receipt["subject_executions"],
            }
    need(checked["build"]["cargo_invocations"] == UNKNOWN, "lost build response was projected as zero")
    need(checked["build"]["marker_state"] == UNKNOWN, "lost build response retained a false marker projection")
    for action_id in (f"physical-{route.lower().replace('-', '_')}" for route in ROUTE_ORDER):
        need(
            checked[action_id]["perf_stat_invocations"] == UNKNOWN
            and checked[action_id]["subject_executions"] == UNKNOWN
            and checked[action_id]["marker_state"] == UNKNOWN,
            f"lost physical response retained false counters: {action_id}",
        )
    return {
        "scope": "local disposable fault model only; no network or scientific execution",
        "actions_checked": len(checked),
        "cases": checked,
        "hard_crash": {
            "intent_without_completion": True,
            "default_verdict": "BLOCKED_PROVENANCE",
            "retry_permitted": False,
        },
    }


def verify_atomic_self_check_publication() -> dict[str, Any]:
    payload = canonical(
        {
            "schema": "lay.v10.e1-traversal-w1-fused-minimum-m2r1-publication-fault-model.v1",
            "verdict": "M2R1_CONTROLLER_VERIFIED_UNRUN",
        }
    )
    with tempfile.TemporaryDirectory(prefix="lay-m2-v3-publication-") as raw:
        destination = pathlib.Path(raw) / "receipt.json"

        def interrupt() -> None:
            raise RuntimeError("injected pre-rename publication failure")

        try:
            atomic_publish_file(destination, payload, before_rename=interrupt)
        except RuntimeError:
            pass
        else:
            raise M2ControllerError("publication fault injection did not interrupt")
        need(not destination.exists(), "partial implementation receipt reached final path")
        need(not list(destination.parent.glob(".*.stage-*")), "publication stage survived failure")
        atomic_publish_file(destination, payload)
        need(destination.read_bytes() == payload and mode_string(destination) == "0444", "atomic publication success drift")
    return {
        "injected_before_rename": True,
        "partial_final_path": False,
        "stage_retained": False,
        "success_bytes_exact": True,
    }


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
    failure_publication = verify_failure_publication_model()
    self_check_publication = verify_atomic_self_check_publication()
    need(not EXECUTION_ADMISSION.exists(), "execution admission exists during offline implementation pass")
    need(not IMPLEMENTATION_RECEIPT.exists(), "implementation receipt already exists")
    need(not EXECUTION_JOURNAL.exists(), "execution journal exists during offline implementation pass")
    need(not CONTROLLER_FAILURE.exists(), "controller failure evidence exists during offline implementation pass")
    need(
        not BOOTSTRAP_AUDIT_RECEIPT.exists()
        and not BUILD_AUDIT_RECEIPT.exists()
        and not TERMINAL_AUDIT_RECEIPT.exists(),
        "M2R1 execution evidence exists during offline implementation pass",
    )
    source_rows = {path.name: file_row(path) for path in (*python_sources(), FRAGMENT)}
    return {
        "schema": "lay.v10.e1-traversal-w1-fused-minimum-m2r1-controller-self-check.v1",
        "task_id": TASK_ID,
        "transaction_id": TRANSACTION_ID,
        "verdict": "M2R1_CONTROLLER_STATIC_SELF_CHECK_PASS",
        "admission": admission,
        "source": source,
        "sources": source_rows,
        "python_graph": graph,
        "terminal_state_model": terminal_state_model,
        "failure_publication": failure_publication,
        "self_check_publication": self_check_publication,
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
        "schema": "lay.v10.e1-traversal-w1-fused-minimum-m2r1-implementation-self-check.v1",
        "task_id": TASK_ID,
        "transaction_id": TRANSACTION_ID,
        "verdict": "M2R1_CONTROLLER_VERIFIED_UNRUN",
        "controller_sha256": sha256_file(CONTROLLER),
        "remote_controller_sha256": sha256_file(REMOTE_CONTROLLER),
        "bootstrap_auditor_sha256": sha256_file(BOOTSTRAP_AUDITOR),
        "build_auditor_sha256": sha256_file(BUILD_AUDITOR),
        "terminal_auditor_sha256": sha256_file(TERMINAL_AUDITOR),
        "fragment_sha256": sha256_file(FRAGMENT),
        "m2_predecessor_controller_sha256": sha256_file(M2_PREDECESSOR_CONTROLLER),
        "m2_predecessor_implementation_receipt_sha256": "55b938ef7851bcf560c1e165e0ebe3c1c5906df6f8c9c5a76559488f0ab35f0a",
        "m2_terminal_receipt_sha256": "21c2098f7c457c4939d6aef2b36e13ed895f7e8f77e040d8b99950f5db2cf85c",
        "local_compile_proof_sha256": sha256_file(LOCAL_COMPILE_PROOF),
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
        "failure_publication_actions_checked": check["failure_publication"]["actions_checked"],
        "next_action_admitted": "independent M2R1 live execution admission only; remote experiment remains forbidden",
    }
    atomic_publish_file(IMPLEMENTATION_RECEIPT, canonical(receipt), 0o444)
    return {**receipt, "receipt_path": str(IMPLEMENTATION_RECEIPT), "receipt_sha256": sha256_file(IMPLEMENTATION_RECEIPT), "receipt_mode": mode_string(IMPLEMENTATION_RECEIPT)}


def verify_execution_admission() -> dict[str, Any]:
    need(IMPLEMENTATION_RECEIPT.is_file(), "sealed implementation receipt absent")
    implementation = json.loads(IMPLEMENTATION_RECEIPT.read_text())
    need(implementation.get("verdict") == "M2R1_CONTROLLER_VERIFIED_UNRUN", "implementation receipt verdict drift")
    need(implementation.get("controller_sha256") == sha256_file(CONTROLLER), "controller changed after seal")
    need(EXECUTION_ADMISSION.is_file(), "M2 execution admission absent; remote actions are forbidden")
    admission = json.loads(EXECUTION_ADMISSION.read_text())
    need(admission.get("verdict") == "M2R1_EXECUTION_ADMITTED" and admission.get("safe_to_execute") is True, "M2 execution not admitted")
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
        "schema": "lay.v10.e1-traversal-w1-fused-minimum-m2r1-bootstrap-payload.v1",
        "task_id": TASK_ID,
        "transaction_id": TRANSACTION_ID,
        "execution_admission_verdict": admission["verdict"],
        "execution_admission_sha256": sha256_file(EXECUTION_ADMISSION),
        "files": {name: sha256_file(path) for name, path in ((name, local / name) for name in files)},
    }
    write_new(local / "PAYLOAD.json", canonical(payload))
    return local


def create_execution_journal(admission: Mapping[str, Any]) -> pathlib.Path:
    return initialize_execution_journal(EXECUTION_JOURNAL, journal_metadata(admission))


def checked_response(
    action_id: str,
    response: Mapping[str, Any],
    allowed_verdicts: Sequence[str],
) -> dict[str, Any]:
    value = dict(response)
    need(value.get("verdict") in set(allowed_verdicts), f"external action verdict drift: {action_id}")
    return value


def run_checked_action(
    journal: pathlib.Path,
    knowledge: dict[str, Any],
    action_id: str,
    callable_action: Any,
    allowed_verdicts: Sequence[str],
) -> dict[str, Any]:
    def invoke() -> dict[str, Any]:
        response = callable_action()
        need(isinstance(response, Mapping), f"external action returned no mapping: {action_id}")
        value = checked_response(action_id, response, allowed_verdicts)
        update_execution_knowledge(knowledge, action_id, value)
        return value

    return run_journaled(journal, action_id, invoke)


def create_remote_cache() -> dict[str, Any]:
    output = ssh(["/usr/bin/mkdir", "-m", "0700", str(REMOTE_CACHE)])
    return {"verdict": "REMOTE_CACHE_CREATED", "stdout_sha256": sha256_bytes(output)}


def upload_bootstrap(bootstrap: pathlib.Path) -> dict[str, Any]:
    run(
        [
            "/usr/bin/scp",
            "-i",
            str(SSH_IDENTITY),
            "-o",
            "BatchMode=yes",
            "-o",
            "ConnectTimeout=10",
            "-q",
            "-p",
            "-r",
            f"{bootstrap}/.",
            f"{REMOTE}:{REMOTE_CACHE}",
        ],
        timeout=3_600,
    )
    return {"verdict": "BOOTSTRAP_PAYLOAD_UPLOADED", "payload_sha256": sha256_file(bootstrap / "PAYLOAD.json")}


def remote_bootstrap() -> dict[str, Any]:
    raw = ssh(
        [
            "/usr/bin/sudo",
            "-n",
            "/usr/bin/python3",
            str(REMOTE_CACHE / "remote-controller.py"),
            "bootstrap",
            "--bootstrap",
            str(REMOTE_CACHE),
        ],
        timeout=3_600,
    )
    lines = raw.decode().strip().splitlines()
    need(lines, "empty remote bootstrap response")
    value = json.loads(lines[-1])
    need(isinstance(value, dict), "remote bootstrap response is not an object")
    return value


def upload_with_receipt(local: pathlib.Path, remote: pathlib.PurePosixPath, verdict: str) -> dict[str, Any]:
    upload_file(local, remote)
    return {"verdict": verdict, "local_receipt_sha256": sha256_file(local), "remote_path": str(remote)}


def execute_once() -> dict[str, Any]:
    admission = verify_execution_admission()
    need(
        not BOOTSTRAP_AUDIT_RECEIPT.exists()
        and not BUILD_AUDIT_RECEIPT.exists()
        and not TERMINAL_AUDIT_RECEIPT.exists()
        and not EXECUTION_JOURNAL.exists()
        and not CONTROLLER_FAILURE.exists(),
        "M2 local execution evidence already exists",
    )
    bootstrap = prepare_remote_bootstrap(admission)
    controller_sha = sha256_file(CONTROLLER)
    knowledge = initial_execution_knowledge()
    journal: pathlib.Path | None = None
    try:
        journal = create_execution_journal(admission)
        run_checked_action(journal, knowledge, "remote-cache-create", create_remote_cache, ("REMOTE_CACHE_CREATED",))
        run_checked_action(
            journal,
            knowledge,
            "bootstrap-upload",
            lambda: upload_bootstrap(bootstrap),
            ("BOOTSTRAP_PAYLOAD_UPLOADED",),
        )
        run_checked_action(
            journal,
            knowledge,
            "remote-bootstrap",
            remote_bootstrap,
            ("M2_BOOTSTRAP_CREATED_UNAUDITED",),
        )
        bootstrap_audit = run_checked_action(
            journal,
            knowledge,
            "bootstrap-audit",
            lambda: run_auditor(BOOTSTRAP_AUDITOR),
            ("M2_BOOTSTRAP_AUDIT_PASS_MARKERS_ADMITTED",),
        )
        remote_bootstrap_audit = REMOTE_CACHE / "M2R1_BOOTSTRAP_AUDIT_RECEIPT.json"
        run_checked_action(
            journal,
            knowledge,
            "bootstrap-audit-upload",
            lambda: upload_with_receipt(
                BOOTSTRAP_AUDIT_RECEIPT,
                remote_bootstrap_audit,
                "BOOTSTRAP_AUDIT_UPLOADED",
            ),
            ("BOOTSTRAP_AUDIT_UPLOADED",),
        )
        run_checked_action(
            journal,
            knowledge,
            "create-markers",
            lambda: remote_call("create-markers", "--admission", str(remote_bootstrap_audit)),
            ("M2_ALL_MARKERS_AVAILABLE",),
        )
        build = run_checked_action(
            journal,
            knowledge,
            "build",
            lambda: remote_call("build-once", "--controller-sha256", controller_sha, timeout=10_800),
            ("M2_BUILD_CREATED_UNAUDITED", "BLOCKED_BUILD"),
        )
        if build.get("verdict") == "BLOCKED_BUILD":
            return run_checked_action(
                journal,
                knowledge,
                "terminal-audit",
                lambda: run_auditor(TERMINAL_AUDITOR),
                ("BLOCKED_BUILD",),
            )
        build_audit = run_checked_action(
            journal,
            knowledge,
            "build-audit",
            lambda: run_auditor(BUILD_AUDITOR),
            ("M2_BUILD_AUDITED_PARITY_ADMITTED", "BLOCKED_BUILD", "BLOCKED_PROVENANCE"),
        )
        if build_audit.get("verdict") in {"BLOCKED_BUILD", "BLOCKED_PROVENANCE"}:
            return run_checked_action(
                journal,
                knowledge,
                "terminal-audit",
                lambda: run_auditor(TERMINAL_AUDITOR),
                (str(build_audit["verdict"]),),
            )
        remote_build_audit = REMOTE_CACHE / "M2R1_BUILD_AUDIT_RECEIPT.json"
        run_checked_action(
            journal,
            knowledge,
            "build-audit-upload",
            lambda: upload_with_receipt(BUILD_AUDIT_RECEIPT, remote_build_audit, "BUILD_AUDIT_UPLOADED"),
            ("BUILD_AUDIT_UPLOADED",),
        )
        parity = run_checked_action(
            journal,
            knowledge,
            "parity",
            lambda: remote_call(
                "parity-once",
                "--controller-sha256",
                controller_sha,
                "--admission",
                str(remote_build_audit),
                timeout=10_800,
            ),
            ("PASS", "BLOCKED_PARITY", "BLOCKED_PROVENANCE"),
        )
        if parity.get("verdict") == "PASS":
            for route in ROUTE_ORDER:
                action_id = f"physical-{route.lower().replace('-', '_')}"
                response = run_checked_action(
                    journal,
                    knowledge,
                    action_id,
                    lambda current=route: remote_call(
                        "run-route",
                        "--controller-sha256",
                        controller_sha,
                        "--route",
                        current,
                        timeout=10_800,
                    ),
                    ("PASS_UNAUDITED", "BLOCKED_PROVENANCE", "BLOCKED_THERMAL"),
                )
                if response.get("verdict") != "PASS_UNAUDITED":
                    break
        terminal = run_checked_action(
            journal,
            knowledge,
            "terminal-audit",
            lambda: run_auditor(TERMINAL_AUDITOR),
            (
                "W1_FUSED_MINIMUM_MECHANISM_PASS",
                "W1_FUSED_MINIMUM_MECHANISM_REJECTED",
                "BLOCKED_PROVENANCE",
                "BLOCKED_BUILD",
                "BLOCKED_PARITY",
                "BLOCKED_THERMAL",
                "BLOCKED_CAPABILITY",
                "BLOCKED_MEASUREMENT",
                "BLOCKED_PERTURBATION",
            ),
        )
        return terminal
    except BaseException as error:
        if journal is None and EXECUTION_JOURNAL.is_dir():
            journal = EXECUTION_JOURNAL
        if journal is None:
            raise
        intent = pending_intent(journal)
        if intent is None:
            intents = journal_rows(journal, "INTENT")
            if intents:
                intent = json.loads(intents[-1].read_text())
            else:
                intent = {
                    "schema": "lay.v10.e1-traversal-w1-fused-minimum-m2r1-action-intent.v1",
                    "task_id": TASK_ID,
                    "transaction_id": TRANSACTION_ID,
                    "sequence": -1,
                    "registry_sequence": -1,
                    "action_id": "journal-initialize",
                    "expected_predecessor": "execution-admission",
                    "expected_scope": "local durable journal creation",
                    "one_shot_route": None,
                    "failure_default": "BLOCKED_PROVENANCE",
                    "retry_permitted": False,
                }
        return publish_controller_failure(journal, intent, error, knowledge)
    finally:
        shutil.rmtree(bootstrap, ignore_errors=True)


def status() -> dict[str, Any]:
    return {
        "task_id": TASK_ID,
        "transaction_id": TRANSACTION_ID,
        "implementation_receipt": file_row(IMPLEMENTATION_RECEIPT) if IMPLEMENTATION_RECEIPT.is_file() else None,
        "execution_admission_present": EXECUTION_ADMISSION.is_file(),
        "bootstrap_audit_present": BOOTSTRAP_AUDIT_RECEIPT.is_file(),
        "build_audit_present": BUILD_AUDIT_RECEIPT.is_file(),
        "terminal_audit_present": TERMINAL_AUDIT_RECEIPT.is_file(),
        "execution_journal_present": EXECUTION_JOURNAL.is_dir(),
        "controller_failure_present": CONTROLLER_FAILURE.is_dir(),
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
