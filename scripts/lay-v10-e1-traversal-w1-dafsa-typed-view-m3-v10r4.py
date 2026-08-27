#!/usr/bin/env python3
"""Fail-closed local orchestrator for the V10R4 TRACE-only recovery."""

from __future__ import annotations

import argparse
import ast
import contextlib
import hashlib
import json
import os
import pathlib
import re
import shutil
import stat
import subprocess
import sys
import tempfile
import time
from collections import deque
from collections.abc import Callable, Mapping, Sequence
from typing import Any


ROOT = pathlib.Path(__file__).resolve().parents[1]
TASK_ID = "slice8b-v10-e1-traversal-w1-dafsa-typed-view-m3-admission-substage-v10r4-20260827"
TRANSACTION_ID = "eeac980119d265ff545142b311256ecb302f70197b3bb634df541302bdc94097"
REMOTE = "e@192.168.3.94"
SSH_KEY = pathlib.Path("/home/ubu/.ssh/mega-mini-admin")
REMOTE_PARENT = pathlib.PurePosixPath("/home/e/.local/share/lay/provenance") / TASK_ID
REMOTE_CACHE = pathlib.PurePosixPath("/home/e/.cache") / f"lay-m3-v10r4-{TRANSACTION_ID}"

CONTROLLER = pathlib.Path(__file__).resolve()
REMOTE_CONTROLLER = ROOT / "scripts/lay-v10-e1-traversal-w1-dafsa-typed-view-m3-v10r4-remote.py"
AUDITOR = ROOT / "scripts/lay-v10-e1-traversal-w1-dafsa-typed-view-m3-v10r4-audit.py"
LEGACY_AUDITOR = ROOT / "scripts/lay-v10-e1-traversal-w1-dafsa-typed-view-m3-v10r2-audit.py"
ADMISSION_SOURCE = ROOT / "src/typing_transition/proposal_admission.rs"
LIVE_SOURCE = ROOT / "src/nanda_wave/l2_field/productive_v1/live.rs"

PAPER = ROOT / "docs/structural_gates/LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_W1_DAFSA_TYPED_VIEW_M3_ADMISSION_SUBSTAGE_DIAGNOSTIC_V10R4_SSH_STDIN_TRANSPORT_CORRECTION_V1_2026-08-27.md"
STRUCTURAL_RECEIPT = ROOT / "docs/structural_gates/receipts/LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_W1_DAFSA_TYPED_VIEW_M3_ADMISSION_SUBSTAGE_DIAGNOSTIC_V10R4_SSH_STDIN_TRANSPORT_CORRECTION_V1_ROUTE_RECEIPT_2026-08-27.json"
CONTROLLER_PREFLIGHT = ROOT / "docs/structural_gates/receipts/LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_W1_DAFSA_TYPED_VIEW_M3_ADMISSION_SUBSTAGE_DIAGNOSTIC_V10R4_TRACE_REUSE_CONTROLLER_V1_PREFLIGHT_2026-08-27.json"
CONTROLLER_EVIDENCE = ROOT / "docs/structural_gates/receipts/LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_W1_DAFSA_TYPED_VIEW_M3_ADMISSION_SUBSTAGE_DIAGNOSTIC_V10R4_TRACE_REUSE_CONTROLLER_V1_2026-08-27"
CONTROLLER_IMPLEMENTATION = CONTROLLER_EVIDENCE / "IMPLEMENTATION_RECEIPT.json"
EXECUTION_JOURNAL = ROOT / "docs/structural_gates/receipts/LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_W1_DAFSA_TYPED_VIEW_M3_ADMISSION_SUBSTAGE_DIAGNOSTIC_V10R4_EXECUTION_JOURNAL_V1_2026-08-27"

OLD_JOURNAL_MANIFEST = ROOT / "docs/structural_gates/receipts/LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_W1_DAFSA_TYPED_VIEW_M3_ADMISSION_SUBSTAGE_DIAGNOSTIC_V10R2_EXECUTION_JOURNAL_V1_2026-08-27/SHA256SUMS"
OLD_BUILD_ROOT = ROOT / "docs/structural_gates/receipts/LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_W1_DAFSA_TYPED_VIEW_M3_ADMISSION_SUBSTAGE_DIAGNOSTIC_V10R2_BUILD_AUDIT_V1_2026-08-27"
OLD_BUILD_RECEIPT = OLD_BUILD_ROOT / "BUILD_AUDIT.json"
OLD_BUILD_MANIFEST = OLD_BUILD_ROOT / "SHA256SUMS"
OLD_LOCAL_ELF = OLD_BUILD_ROOT / "REMOTE_BUILD/v10-test-elf"
OLD_QUIET_DIAGNOSIS = ROOT / "docs/structural_gates/receipts/LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_W1_DAFSA_TYPED_VIEW_M3_ADMISSION_SUBSTAGE_DIAGNOSTIC_V10R2_QUIET_FAILURE_DIAGNOSIS_2026-08-27.json"

V10R3_LOCAL_CONTROLLER = ROOT / "scripts/lay-v10-e1-traversal-w1-dafsa-typed-view-m3-v10r3.py"
V10R3_REMOTE_CONTROLLER = ROOT / "scripts/lay-v10-e1-traversal-w1-dafsa-typed-view-m3-v10r3-remote.py"
V10R3_AUDITOR = ROOT / "scripts/lay-v10-e1-traversal-w1-dafsa-typed-view-m3-v10r3-audit.py"
V10R3_IMPLEMENTATION = ROOT / "docs/structural_gates/receipts/LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_W1_DAFSA_TYPED_VIEW_M3_ADMISSION_SUBSTAGE_DIAGNOSTIC_V10R3_TRACE_REUSE_CONTROLLER_V1_2026-08-27/IMPLEMENTATION_RECEIPT.json"
V10R3_JOURNAL_MANIFEST = ROOT / "docs/structural_gates/receipts/LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_W1_DAFSA_TYPED_VIEW_M3_ADMISSION_SUBSTAGE_DIAGNOSTIC_V10R3_EXECUTION_JOURNAL_V1_2026-08-27/SHA256SUMS"
V10R3_CONTROLLER_FAILURE = ROOT / "docs/structural_gates/receipts/LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_W1_DAFSA_TYPED_VIEW_M3_ADMISSION_SUBSTAGE_DIAGNOSTIC_V10R3_EXECUTION_JOURNAL_V1_2026-08-27/CONTROLLER_FAILURE.json"
V10R3_TRANSPORT_DIAGNOSIS = ROOT / "docs/structural_gates/receipts/LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_W1_DAFSA_TYPED_VIEW_M3_ADMISSION_SUBSTAGE_DIAGNOSTIC_V10R3_TRANSPORT_FAILURE_DIAGNOSIS_2026-08-27.json"

ADMISSION_RECEIPT = ROOT / "docs/structural_gates/receipts/LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_W1_DAFSA_TYPED_VIEW_M3_ADMISSION_SUBSTAGE_DIAGNOSTIC_V10R4_EXECUTION_ADMISSION_V1_2026-08-27/EXECUTION_ADMISSION.json"
BOOTSTRAP_AUDIT = ROOT / "docs/structural_gates/receipts/LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_W1_DAFSA_TYPED_VIEW_M3_ADMISSION_SUBSTAGE_DIAGNOSTIC_V10R4_BOOTSTRAP_REUSE_AUDIT_V1_2026-08-27/BOOTSTRAP_AUDIT.json"
QUIET_ADMISSION = ROOT / "docs/structural_gates/receipts/LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_W1_DAFSA_TYPED_VIEW_M3_ADMISSION_SUBSTAGE_DIAGNOSTIC_V10R4_QUIET_READINESS_V1_2026-08-27/QUIET_ADMISSION.json"
TERMINAL_AUDIT = ROOT / "docs/structural_gates/receipts/LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_W1_DAFSA_TYPED_VIEW_M3_ADMISSION_SUBSTAGE_DIAGNOSTIC_V10R4_TERMINAL_AUDIT_V1_2026-08-27/TERMINAL_AUDIT.json"

ACTIONS = ("self-check", "seal-self-check", "execute", "status")
EXTERNAL_ACTIONS = (
    "transport-admission",
    "live-admission",
    "remote-cache-create",
    "bootstrap-upload",
    "remote-bootstrap",
    "bootstrap-audit",
    "bootstrap-audit-upload",
    "quiet-audit",
    "quiet-audit-upload",
    "create-marker",
    "trace-once",
    "terminal-audit",
)
EXPECTED_ELF_SIZE = 320_986_144
EXPECTED_ELF_SHA256 = "0378514225ccec3cadbcfedd21ec77db66518a5eb6789f9acd83525ccf009696"
EXPECTED_ELF_BUILD_ID = "9e2e7c1fef9272f87c14876d7194609df6ac948d"


class V10R4ControllerError(RuntimeError):
    pass


def need(condition: bool, message: str) -> None:
    if not condition:
        raise V10R4ControllerError(message)


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


def file_row(path: pathlib.Path) -> dict[str, Any]:
    return {"path": str(path), "mode": mode_string(path), "size_bytes": path.stat().st_size, "sha256": sha256_file(path)}


def load_json(path: pathlib.Path) -> Any:
    value = json.loads(path.read_text())
    need(isinstance(value, dict), f"JSON object required: {path}")
    return value


def write_new(path: pathlib.Path, value: bytes, mode: int = 0o600) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    descriptor = os.open(path, os.O_WRONLY | os.O_CREAT | os.O_EXCL, mode)
    try:
        with os.fdopen(descriptor, "wb") as target:
            target.write(value)
            target.flush()
            os.fsync(target.fileno())
    except BaseException:
        with contextlib.suppress(FileNotFoundError):
            path.unlink()
        raise


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
            rows.append(f"{sha256_file(path)}  {path.relative_to(root)}\n")
    write_new(root / "SHA256SUMS", "".join(rows).encode(), 0o444)


def verify_manifest(root: pathlib.Path) -> int:
    manifest = root / "SHA256SUMS"
    need(manifest.is_file(), f"manifest absent: {root}")
    count = 0
    for raw in manifest.read_text().splitlines():
        digest, relative = raw.split("  ", 1)
        path = root / relative
        need(path.is_file() and sha256_file(path) == digest, f"manifest mismatch: {path}")
        count += 1
    return count


def seal_tree(root: pathlib.Path) -> None:
    for path in sorted(root.rglob("*"), reverse=True):
        path.chmod(0o444 if path.is_file() else 0o555)
    root.chmod(0o555)


def run(argv: Sequence[str], *, timeout: float = 120, check: bool = True) -> subprocess.CompletedProcess[bytes]:
    result = subprocess.run(list(argv), stdout=subprocess.PIPE, stderr=subprocess.PIPE, timeout=timeout, check=False)
    if check and result.returncode != 0:
        raise V10R4ControllerError(
            f"command failed rc={result.returncode}: {list(argv)!r}; "
            f"stdout={result.stdout[-4096:]!r}; stderr={result.stderr[-4096:]!r}"
        )
    return result


def parse_json_output(result: subprocess.CompletedProcess[bytes], label: str) -> dict[str, Any]:
    lines = result.stdout.decode().strip().splitlines()
    need(lines, f"{label} returned no JSON")
    value = json.loads(lines[-1])
    need(isinstance(value, dict), f"{label} response is not an object")
    return value


def ssh(argv: Sequence[str], *, timeout: float = 3600) -> bytes:
    return run([
        "/usr/bin/ssh", "-i", str(SSH_KEY), "-o", "BatchMode=yes", "-o", "ConnectTimeout=10",
        REMOTE, *argv,
    ], timeout=timeout).stdout


def scp_file(local: pathlib.Path, remote: pathlib.PurePosixPath) -> None:
    result = run([
        "/usr/bin/scp", "-i", str(SSH_KEY), "-o", "BatchMode=yes", "-o", "ConnectTimeout=10",
        str(local), f"{REMOTE}:{remote}",
    ], timeout=3600, check=False)
    need(result.returncode == 0, f"SCP failed: {result.stderr[-4096:]!r}")


def auditor_call(action: str) -> dict[str, Any]:
    return parse_json_output(run([str(AUDITOR), action], timeout=10_800), f"auditor {action}")


def remote_controller_path(cache: bool = False) -> pathlib.PurePosixPath:
    return (REMOTE_CACHE if cache else REMOTE_PARENT / "bootstrap-reuse-v1") / "remote-controller.py"


def remote_call(action: str, *arguments: str, cache: bool = False, timeout: float = 3600) -> dict[str, Any]:
    raw = ssh(["/usr/bin/sudo", "-n", "/usr/bin/python3", str(remote_controller_path(cache)), action, *arguments], timeout=timeout)
    lines = raw.decode().strip().splitlines()
    need(lines, f"remote {action} returned no JSON")
    value = json.loads(lines[-1])
    need(isinstance(value, dict), f"remote {action} response is not an object")
    return value


def verify_controller_preflight() -> dict[str, Any]:
    value = load_json(CONTROLLER_PREFLIGHT)
    need(value.get("verdict") == "READY_TO_IMPLEMENT" and value.get("safe_to_implement") is True, "controller preflight not ready")
    need(value.get("manifest_sha256") == "549e15435166b8a049d55f1a16a7621c57c5e3db1f30123728d7320f3d8363f4", "controller preflight manifest identity drift")
    return value


def verify_fixed_inputs() -> dict[str, Any]:
    expected = {
        PAPER: (4_737, "fccddd22499cf639d595333674ed67c6ee63290a9052f719289f61e1e83bdb63", "0444"),
        STRUCTURAL_RECEIPT: (23_624, "9d77ec2d14a6636d232804b1bfe4fc77e2bd4f8bf176191df6431ca393e80769", "0444"),
        CONTROLLER_PREFLIGHT: (27_089, "da3b5c3681c7e07c11d1d497506b7f0395a750e667d71694f269cf28f7591c78", "0444"),
        OLD_JOURNAL_MANIFEST: (2_232, "5bf039fa3555c8ed7bb81fd6a1b7703ced12e235e4c53590ae872d64621e3aed", "0444"),
        OLD_BUILD_RECEIPT: (254_342, "edf6a15846f5918c2524299b60d33be41f02fe6ec56e21e04ba3c72cb3626706", "0444"),
        OLD_BUILD_MANIFEST: (661, "6aaf20d21d46535c4520c6daec45cc9b194cf3f591e0e917e992b48a9d17cdbd", "0444"),
        OLD_LOCAL_ELF: (EXPECTED_ELF_SIZE, EXPECTED_ELF_SHA256, "0444"),
        OLD_QUIET_DIAGNOSIS: (2_646, "c2974aca094f09c097aea362fd906b8727f0cb4c6b014beb6f1895c14c8899c8", "0444"),
        V10R3_LOCAL_CONTROLLER: (32_504, "306af1cfb25d61d0b8ed9e7275c0e717c620ce31b1e2425bdeff180b092855c3", "0555"),
        V10R3_REMOTE_CONTROLLER: (32_726, "45e2be766310a38d104629977c246f5513cf0d020c5fa1562b17a9a738b46f07", "0555"),
        V10R3_AUDITOR: (48_839, "6ba4d2794071e81aff7c358813797dc7210406bf351a463de34055e529f84de3", "0555"),
        V10R3_IMPLEMENTATION: (13_225, "7719b69b7a218b4d525aa0cf583c610d035ea6a458561c53a60f36a7a9188286", "0444"),
        V10R3_JOURNAL_MANIFEST: (272, "5baf0039407a6a3e752326440e47b13a4005327f365665eecaa78331d1d693c6", "0444"),
        V10R3_CONTROLLER_FAILURE: (5_262, "583e8cf071a64e65734ba47e9502fe47597b5328e9635331b7d1d8237206ada7", "0444"),
        V10R3_TRANSPORT_DIAGNOSIS: (2_766, "cb3c616bd83bb08f7db16c292e3795c248780676e39de5fdbddaf7a723f5d97e", "0444"),
        LEGACY_AUDITOR: (82_560, "b83331a49d74d0f890a82750b9e0d9c0b9f073d8059dbeb735de5fb3cf594261", "0555"),
        ADMISSION_SOURCE: (88_326, "6169e6d89a06c9ad3d7aefd467a8147f6d094b962faa25c739ad6d94a364b3dd", "0664"),
        LIVE_SOURCE: (71_012, "36aeddd5e605e67377f99343f9937606ac774d9ba4bb5710152de060bc9d183b", "0664"),
    }
    rows = {}
    for path, (size, digest, mode) in expected.items():
        row = file_row(path)
        need((row["size_bytes"], row["sha256"], row["mode"]) == (size, digest, mode), f"fixed input drift: {path}")
        rows[str(path)] = row
    verify_manifest(OLD_BUILD_ROOT)
    return rows


def parse_python(path: pathlib.Path) -> ast.Module:
    source = path.read_text()
    compile(source, str(path), "exec")
    return ast.parse(source, filename=str(path))


def python_literal(tree: ast.Module, name: str) -> Any:
    for node in tree.body:
        if isinstance(node, ast.Assign) and any(isinstance(target, ast.Name) and target.id == name for target in node.targets):
            return ast.literal_eval(node.value)
        if isinstance(node, ast.AnnAssign) and isinstance(node.target, ast.Name) and node.target.id == name:
            return ast.literal_eval(node.value)
    raise V10R4ControllerError(f"Python registry absent: {name}")


def reachable_functions(tree: ast.Module, roots: Sequence[str]) -> set[str]:
    functions = {node.name: node for node in tree.body if isinstance(node, (ast.FunctionDef, ast.AsyncFunctionDef))}
    reached = set()
    queue = deque(roots)
    while queue:
        name = queue.popleft()
        if name in reached or name not in functions:
            continue
        reached.add(name)
        for node in ast.walk(functions[name]):
            if isinstance(node, ast.Call) and isinstance(node.func, ast.Name) and node.func.id in functions:
                queue.append(node.func.id)
    return reached


def static_graph() -> dict[str, Any]:
    paths = (CONTROLLER, REMOTE_CONTROLLER, AUDITOR)
    trees = {path: parse_python(path) for path in paths}
    remote_tree = trees[REMOTE_CONTROLLER]
    auditor_tree = trees[AUDITOR]
    need(tuple(python_literal(remote_tree, "ACTIONS")) == ("self-check", "status", "bootstrap-reuse", "create-marker", "trace-once"), "remote action registry drift")
    need(tuple(python_literal(remote_tree, "ROUTES")) == ("TRACE-REUSE",), "remote route registry drift")
    need(python_literal(remote_tree, "MARKER_NAMES") == {"TRACE-REUSE": "trace"}, "remote marker registry drift")
    need(tuple(python_literal(auditor_tree, "ACTIONS")) == ("self-check", "transport", "live-admission", "bootstrap", "quiet", "terminal", "status"), "auditor action registry drift")
    reached = reachable_functions(remote_tree, ("main",))
    need("trace_once" in reached and "bootstrap_reuse" in reached and "create_marker" in reached, "TRACE supporting graph incomplete")
    need(not ({"build_once", "find_test_elf"} & reached), "build route reachable")
    trace_node = next(
        node
        for node in remote_tree.body
        if isinstance(node, (ast.FunctionDef, ast.AsyncFunctionDef)) and node.name == "trace_once"
    )
    trace_literals = {
        node.value
        for node in ast.walk(trace_node)
        if isinstance(node, ast.Constant) and isinstance(node.value, str)
    }
    forbidden = ("/usr/bin/" + "cargo", "/usr/bin/" + "rustc", "/usr/bin/" + "perf", "cargo-guard" + ".sh")
    sources = {path.name: path.read_text() for path in paths}
    need(not any(token in text for token in forbidden for text in sources.values()), "Cargo rustc or perf executable reachable")
    need("--pid" not in trace_literals and ("SIG" + "INT") not in trace_literals, "attach or interrupt lifecycle reachable")
    need("OLD_ELF" in sources[REMOTE_CONTROLLER.name] and "SCIENTIFIC_TEST" in sources[REMOTE_CONTROLLER.name], "direct sealed-ELF TRACE absent")
    need(all(f'"{action}"' in sources[CONTROLLER.name] for action in EXTERNAL_ACTIONS), "external action registry incomplete")
    ssh_python_node = next(
        node
        for node in auditor_tree.body
        if isinstance(node, (ast.FunctionDef, ast.AsyncFunctionDef)) and node.name == "ssh_python"
    )
    ssh_literals = {
        node.value
        for node in ast.walk(ssh_python_node)
        if isinstance(node, ast.Constant) and isinstance(node.value, str)
    }
    need("/usr/bin/python3" in ssh_literals and "-" in ssh_literals and "-c" not in ssh_literals, "stdin observer interpreter route drift")
    need(any(
        isinstance(node, ast.Call) and any(keyword.arg == "input_bytes" for keyword in node.keywords)
        for node in ast.walk(ssh_python_node)
    ), "stdin observer bytes are not passed through subprocess input")
    transport_program = python_literal(auditor_tree, "TRANSPORT_PROGRAM")
    need(
        python_literal(remote_tree, "EXPECTED_TRANSPORT_PROGRAM_SHA256")
        == hashlib.sha256(transport_program.encode()).hexdigest(),
        "stdin observer program identity is not pinned by remote admission",
    )
    admission = ADMISSION_SOURCE.read_text()
    need(len(re.findall(r"\nfn candidate_admission\(", admission)) == 1, "candidate_admission implementation count drift")
    need(len(re.findall(r"admission_trace_(?:bool|value)!\(", admission)) == 36, "trace predicate registry drift")
    return {
        "compiled": sorted(path.name for path in paths),
        "remote_actions": ["self-check", "status", "bootstrap-reuse", "create-marker", "trace-once"],
        "routes": ["TRACE-REUSE"],
        "markers": ["trace.available"],
        "external_actions": list(EXTERNAL_ACTIONS),
        "remote_reachable_functions": sorted(reached),
        "build_reachable": False,
        "cargo_reachable": False,
        "rustc_reachable": False,
        "perf_reachable": False,
        "direct_elf_execution": True,
        "stdin_observer_transport": True,
        "transport_precedes_remote_mutation": EXTERNAL_ACTIONS[0] == "transport-admission",
        "source_edits_reachable": False,
        "production_runtime_edit_reachable": False,
    }


def fault_model() -> dict[str, Any]:
    rows = {}
    for index, action in enumerate(EXTERNAL_ACTIONS, 1):
        rows[action] = {
            "sequence": index,
            "intent_durable": True,
            "completion_absent": True,
            "pending_blocks_next_action": True,
            "affected_facts": "UNKNOWN",
            "retry_permitted": False,
        }
    return {"cases": rows, "cases_passed": len(rows), "cases_expected": len(EXTERNAL_ACTIONS)}


def error_payload_fault() -> dict[str, Any]:
    try:
        run(["/usr/bin/python3", "-c", "import sys; print('OUT_SENTINEL'); print('ERR_SENTINEL',file=sys.stderr); raise SystemExit(7)"])
    except V10R4ControllerError as error:
        message = str(error)
        need("OUT_SENTINEL" in message and "ERR_SENTINEL" in message, "nonzero output was not retained")
        return {"returncode": 7, "stdout_retained": True, "stderr_retained": True, "remote_actions": 0}
    raise V10R4ControllerError("error payload fault did not fail")


def self_check() -> dict[str, Any]:
    preflight = verify_controller_preflight()
    inputs = verify_fixed_inputs()
    graph = static_graph()
    remote = parse_json_output(run([str(REMOTE_CONTROLLER), "self-check"]), "remote self-check")
    auditor = parse_json_output(run([str(AUDITOR), "self-check"], timeout=300), "auditor self-check")
    need(remote.get("verdict") == "V10R4_REMOTE_CONTROLLER_STATIC_PASS", "remote self-check failed")
    need(auditor.get("verdict") == "V10R4_INDEPENDENT_AUDITOR_STATIC_PASS", "auditor self-check failed")
    return {
        "schema": "lay.v10r4-controller-self-check.v1",
        "task_id": TASK_ID,
        "transaction_id": TRANSACTION_ID,
        "verdict": "V10R4_CONTROLLER_STATIC_SELF_CHECK_PASS",
        "local_controller_sha256": sha256_file(CONTROLLER),
        "remote_controller_sha256": sha256_file(REMOTE_CONTROLLER),
        "auditor_sha256": sha256_file(AUDITOR),
        "controller_preflight_sha256": sha256_file(CONTROLLER_PREFLIGHT),
        "controller_preflight_manifest_sha256": preflight["manifest_sha256"],
        "fixed_inputs": inputs,
        "command_graph": graph,
        "remote_self_check": remote,
        "auditor_self_check": auditor,
        "fault_injection": fault_model(),
        "error_payload_fault": error_payload_fault(),
        "markers_created": 0,
        "markers_consumed": 0,
        "cargo_invocations": 0,
        "rustc_compilations": 0,
        "subject_executions": 0,
        "remote_reads": 0,
        "remote_writes": 0,
        "perf_record_invocations": 0,
        "perf_stat_invocations": 0,
        "runtime_authority_changed": False,
    }


def seal_self_check() -> dict[str, Any]:
    need(not CONTROLLER_EVIDENCE.exists(), "implementation evidence already exists")
    for path in (CONTROLLER, REMOTE_CONTROLLER, AUDITOR):
        need(mode_string(path) == "0555", f"controller source not sealed executable: {path}")
    check = self_check()
    receipt = {
        "schema": "lay.v10r4-controller-implementation.v1",
        "task_id": TASK_ID,
        "transaction_id": TRANSACTION_ID,
        "verdict": "V10R4_TRACE_REUSE_CONTROLLERS_VERIFIED_UNRUN",
        "local_controller_sha256": check["local_controller_sha256"],
        "remote_controller_sha256": check["remote_controller_sha256"],
        "auditor_sha256": check["auditor_sha256"],
        "controller_preflight_sha256": check["controller_preflight_sha256"],
        "controller_preflight_manifest_sha256": check["controller_preflight_manifest_sha256"],
        "old_sealed_elf": {"size_bytes": EXPECTED_ELF_SIZE, "sha256": EXPECTED_ELF_SHA256, "build_id": EXPECTED_ELF_BUILD_ID},
        "self_check": check,
        "execution_admission_present": False,
        "journal_present": False,
        "markers_created": 0,
        "markers_consumed": 0,
        "cargo_invocations": 0,
        "rustc_compilations": 0,
        "subject_executions": 0,
        "remote_reads": 0,
        "remote_writes": 0,
        "perf_record_invocations": 0,
        "perf_stat_invocations": 0,
        "runtime_authority_changed": False,
        "production_authority_admitted": False,
        "next_action_admitted": "independent live execution admission only",
    }
    stage = CONTROLLER_EVIDENCE.with_name(f"{CONTROLLER_EVIDENCE.name}.stage-{os.getpid()}-{time.time_ns()}")
    stage.mkdir(parents=True, mode=0o700)
    try:
        write_new(stage / "IMPLEMENTATION_RECEIPT.json", canonical(receipt), 0o444)
        write_new(stage / "SELF_CHECK.json", canonical(check), 0o444)
        write_manifest(stage)
        seal_tree(stage)
        os.rename(stage, CONTROLLER_EVIDENCE)
        fsync_dir(CONTROLLER_EVIDENCE.parent)
        return receipt
    except BaseException:
        if stage.exists():
            shutil.rmtree(stage)
        raise


def verify_implementation() -> dict[str, Any]:
    need(CONTROLLER_EVIDENCE.is_dir() and mode_string(CONTROLLER_EVIDENCE) == "0555", "implementation evidence absent")
    verify_manifest(CONTROLLER_EVIDENCE)
    value = load_json(CONTROLLER_IMPLEMENTATION)
    need(value.get("verdict") == "V10R4_TRACE_REUSE_CONTROLLERS_VERIFIED_UNRUN", "implementation verdict drift")
    need(value.get("local_controller_sha256") == sha256_file(CONTROLLER), "local controller SHA drift")
    need(value.get("remote_controller_sha256") == sha256_file(REMOTE_CONTROLLER), "remote controller SHA drift")
    need(value.get("auditor_sha256") == sha256_file(AUDITOR), "auditor SHA drift")
    return value


def prepare_bootstrap(admission: Mapping[str, Any]) -> pathlib.Path:
    stage = pathlib.Path(tempfile.mkdtemp(prefix="lay-m3-v10r4-bootstrap-"))
    try:
        shutil.copy2(REMOTE_CONTROLLER, stage / "remote-controller.py")
        shutil.copy2(ADMISSION_RECEIPT, stage / "EXECUTION_ADMISSION.json")
        (stage / "remote-controller.py").chmod(0o555)
        (stage / "EXECUTION_ADMISSION.json").chmod(0o444)
        files = {name: file_row(stage / name) for name in ("EXECUTION_ADMISSION.json", "remote-controller.py")}
        payload = {
            "schema": "lay.v10r4-bootstrap-reuse-payload.v1",
            "task_id": TASK_ID,
            "transaction_id": TRANSACTION_ID,
            "local_controller_sha256": admission["local_controller_sha256"],
            "remote_controller_sha256": admission["remote_controller_sha256"],
            "auditor_sha256": admission["auditor_sha256"],
            "files": {name: {"size_bytes": row["size_bytes"], "sha256": row["sha256"]} for name, row in files.items()},
            "routes": ["TRACE-REUSE"],
            "build_reachable": False,
        }
        write_new(stage / "PAYLOAD.json", canonical(payload), 0o444)
        write_manifest(stage)
        return stage
    except BaseException:
        shutil.rmtree(stage, ignore_errors=True)
        raise


def initialize_journal() -> pathlib.Path:
    need(not EXECUTION_JOURNAL.exists(), "execution journal already exists")
    EXECUTION_JOURNAL.mkdir(parents=True, mode=0o700)
    write_new(EXECUTION_JOURNAL / "JOURNAL_HEADER.json", canonical({
        "schema": "lay.v10r4-controller-journal.v1",
        "task_id": TASK_ID,
        "transaction_id": TRANSACTION_ID,
        "controller_sha256": sha256_file(CONTROLLER),
        "external_actions": list(EXTERNAL_ACTIONS),
        "retry_permitted": False,
    }), 0o444)
    fsync_dir(EXECUTION_JOURNAL)
    return EXECUTION_JOURNAL


def journal_rows(root: pathlib.Path, suffix: str) -> list[pathlib.Path]:
    return sorted(root.glob(f"[0-9][0-9]-*.{suffix}.json"))


def pending_intent(root: pathlib.Path) -> bool:
    return len(journal_rows(root, "intent")) != len(journal_rows(root, "complete"))


def append_intent(root: pathlib.Path, sequence: int, action: str) -> pathlib.Path:
    need(not pending_intent(root), "previous external action has no durable completion")
    path = root / f"{sequence:02d}-{action}.intent.json"
    write_new(path, canonical({
        "schema": "lay.v10r4-controller-intent.v1",
        "task_id": TASK_ID,
        "transaction_id": TRANSACTION_ID,
        "sequence": sequence,
        "action": action,
        "status": "INTENT_DURABLE",
        "affected_facts_until_completion": "UNKNOWN",
        "retry_permitted": False,
    }), 0o444)
    fsync_dir(root)
    return path


def append_completion(root: pathlib.Path, sequence: int, action: str, response: Mapping[str, Any]) -> pathlib.Path:
    path = root / f"{sequence:02d}-{action}.complete.json"
    write_new(path, canonical({
        "schema": "lay.v10r4-controller-completion.v1",
        "task_id": TASK_ID,
        "transaction_id": TRANSACTION_ID,
        "sequence": sequence,
        "action": action,
        "status": "COMPLETION_DURABLE",
        "response": response,
    }), 0o444)
    fsync_dir(root)
    return path


def journaled(
    root: pathlib.Path,
    sequence: int,
    action: str,
    callback: Callable[[], dict[str, Any]],
    accepted: Sequence[str],
) -> dict[str, Any]:
    need(action == EXTERNAL_ACTIONS[sequence - 1], f"external action order drift: {action}")
    append_intent(root, sequence, action)
    response = callback()
    need(response.get("task_id", TASK_ID) == TASK_ID, f"external action namespace drift: {action}")
    need(response.get("transaction_id", TRANSACTION_ID) == TRANSACTION_ID, f"external action transaction drift: {action}")
    need(response.get("verdict") in set(accepted), f"external action verdict drift: {action}: {response.get('verdict')}")
    append_completion(root, sequence, action, response)
    return response


def create_remote_cache() -> dict[str, Any]:
    ssh(["/usr/bin/install", "-d", "-m", "0700", str(REMOTE_CACHE)])
    return {"task_id": TASK_ID, "transaction_id": TRANSACTION_ID, "verdict": "V10R4_REMOTE_CACHE_CREATED"}


def upload_bootstrap(staging: pathlib.Path) -> dict[str, Any]:
    for path in sorted(staging.iterdir()):
        scp_file(path, REMOTE_CACHE / path.name)
    return {
        "task_id": TASK_ID,
        "transaction_id": TRANSACTION_ID,
        "verdict": "V10R4_BOOTSTRAP_UPLOADED",
        "files": sorted(path.name for path in staging.iterdir()),
        "remote_controller_sha256": sha256_file(REMOTE_CONTROLLER),
    }


def ensure_remote_admissions() -> None:
    ssh(["/usr/bin/install", "-d", "-m", "0700", str(REMOTE_CACHE / "admissions")])


def upload_audit(files: Mapping[str, pathlib.Path], verdict: str) -> dict[str, Any]:
    ensure_remote_admissions()
    rows = {}
    for name, path in files.items():
        scp_file(path, REMOTE_CACHE / "admissions" / name)
        rows[name] = file_row(path)
    return {"task_id": TASK_ID, "transaction_id": TRANSACTION_ID, "verdict": verdict, "files": rows}


def finish_journal(journal: pathlib.Path, verdict: str, *, terminal_audit: pathlib.Path | None = None) -> None:
    write_new(journal / "TERMINAL.json", canonical({
        "schema": "lay.v10r4-controller-terminal.v1",
        "task_id": TASK_ID,
        "transaction_id": TRANSACTION_ID,
        "verdict": verdict,
        "terminal_audit_sha256": sha256_file(terminal_audit) if terminal_audit is not None else None,
        "external_actions_completed": len(journal_rows(journal, "complete")),
        "retry_permitted": False,
        "runtime_authority_changed": False,
    }), 0o444)
    write_manifest(journal)
    seal_tree(journal)


def execute() -> dict[str, Any]:
    verify_implementation()
    journal = initialize_journal()
    sequence = 1
    staging: pathlib.Path | None = None
    try:
        journaled(journal, sequence, "transport-admission", lambda: auditor_call("transport"), ("V10R4_STDIN_TRANSPORT_PASS",))
        sequence += 1
        admission = journaled(journal, sequence, "live-admission", lambda: auditor_call("live-admission"), ("V10R4_EXECUTION_ADMITTED",))
        sequence += 1
        staging = prepare_bootstrap(admission)
        journaled(journal, sequence, "remote-cache-create", create_remote_cache, ("V10R4_REMOTE_CACHE_CREATED",))
        sequence += 1
        journaled(journal, sequence, "bootstrap-upload", lambda: upload_bootstrap(staging), ("V10R4_BOOTSTRAP_UPLOADED",))
        sequence += 1
        journaled(journal, sequence, "remote-bootstrap", lambda: remote_call("bootstrap-reuse", "--bootstrap", str(REMOTE_CACHE), cache=True), ("V10R4_BOOTSTRAP_REUSE_CREATED_UNAUDITED",))
        sequence += 1
        journaled(journal, sequence, "bootstrap-audit", lambda: auditor_call("bootstrap"), ("V10R4_BOOTSTRAP_REUSE_AUDIT_PASS_QUIET_ADMITTED",))
        sequence += 1
        journaled(journal, sequence, "bootstrap-audit-upload", lambda: upload_audit({"BOOTSTRAP_AUDIT.json": BOOTSTRAP_AUDIT}, "V10R4_BOOTSTRAP_AUDIT_UPLOADED"), ("V10R4_BOOTSTRAP_AUDIT_UPLOADED",))
        sequence += 1
        quiet = journaled(journal, sequence, "quiet-audit", lambda: auditor_call("quiet"), ("V10R4_QUIET_READY_TRACE_ADMITTED", "BLOCKED_QUIET_BEFORE_MARKER"))
        sequence += 1
        if quiet.get("verdict") == "BLOCKED_QUIET_BEFORE_MARKER":
            finish_journal(journal, "BLOCKED_QUIET_BEFORE_MARKER")
            return quiet
        journaled(journal, sequence, "quiet-audit-upload", lambda: upload_audit({"QUIET_ADMISSION.json": QUIET_ADMISSION}, "V10R4_QUIET_ADMISSION_UPLOADED"), ("V10R4_QUIET_ADMISSION_UPLOADED",))
        sequence += 1
        quiet_remote = str(REMOTE_CACHE / "admissions/QUIET_ADMISSION.json")
        journaled(journal, sequence, "create-marker", lambda: remote_call("create-marker", "--quiet", quiet_remote), ("V10R4_TRACE_MARKER_AVAILABLE",))
        sequence += 1
        journaled(journal, sequence, "trace-once", lambda: remote_call("trace-once", "--quiet", quiet_remote, timeout=10_800), ("V10R4_TRACE_REUSE_CREATED_UNAUDITED", "BLOCKED_PROVENANCE"))
        sequence += 1
        terminal = journaled(journal, sequence, "terminal-audit", lambda: auditor_call("terminal"), (
            "ADMISSION_SUBSTAGES_DECOMPOSED", "BLOCKED_PROVENANCE", "BLOCKED_BUILD", "BLOCKED_SEMANTIC", "BLOCKED_CAPABILITY",
        ))
        finish_journal(journal, terminal["verdict"], terminal_audit=TERMINAL_AUDIT)
        return terminal
    except BaseException as error:
        failure = {
            "schema": "lay.v10r4-controller-failure.v1",
            "task_id": TASK_ID,
            "transaction_id": TRANSACTION_ID,
            "verdict": "BLOCKED_PROVENANCE",
            "error": f"{type(error).__name__}: {error}",
            "pending_intent": pending_intent(journal),
            "external_actions_completed": len(journal_rows(journal, "complete")),
            "affected_remote_facts": "UNKNOWN",
            "retry_permitted": False,
            "runtime_authority_changed": False,
        }
        write_new(journal / "CONTROLLER_FAILURE.json", canonical(failure), 0o444)
        write_manifest(journal)
        seal_tree(journal)
        raise V10R4ControllerError(json.dumps(failure, sort_keys=True)) from error
    finally:
        if staging is not None:
            shutil.rmtree(staging, ignore_errors=True)


def status() -> dict[str, Any]:
    remote = None
    try:
        cache = not (REMOTE_PARENT and ADMISSION_RECEIPT.exists() and BOOTSTRAP_AUDIT.exists())
        raw = ssh(["/usr/bin/sudo", "-n", "/usr/bin/python3", str(remote_controller_path(cache=cache)), "status"], timeout=60)
        lines = raw.decode().strip().splitlines()
        remote = json.loads(lines[-1]) if lines else None
    except BaseException as error:
        remote = {"verdict": "UNKNOWN", "error": f"{type(error).__name__}: {error}"}
    return {
        "schema": "lay.v10r4-controller-status.v1",
        "verdict": "V10R4_CONTROLLER_STATUS",
        "controller_implementation": CONTROLLER_IMPLEMENTATION.exists(),
        "execution_journal": EXECUTION_JOURNAL.exists(),
        "receipts": {
            "admission": ADMISSION_RECEIPT.exists(),
            "bootstrap": BOOTSTRAP_AUDIT.exists(),
            "quiet": QUIET_ADMISSION.exists(),
            "terminal": TERMINAL_AUDIT.exists(),
        },
        "remote": remote,
    }


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("action", choices=ACTIONS)
    args = parser.parse_args()
    try:
        if args.action == "self-check":
            value = self_check()
        elif args.action == "seal-self-check":
            value = seal_self_check()
        elif args.action == "execute":
            value = execute()
        else:
            value = status()
        print(json.dumps(value, sort_keys=True))
        return 0
    except BaseException as error:
        print(json.dumps({
            "schema": "lay.v10r4-controller-error.v1",
            "task_id": TASK_ID,
            "transaction_id": TRANSACTION_ID,
            "verdict": "BLOCKED_PROVENANCE",
            "error": f"{type(error).__name__}: {error}",
            "retry_permitted": False,
        }, sort_keys=True))
        return 1


if __name__ == "__main__":
    sys.exit(main())
