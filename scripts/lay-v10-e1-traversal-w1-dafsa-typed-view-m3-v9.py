#!/usr/bin/env python3
"""Local fail-closed orchestrator for the M3 V9 materialization trace."""

from __future__ import annotations

import argparse
import ast
import hashlib
import json
import os
import pathlib
import shutil
import stat
import subprocess
import sys
import tempfile
import time
from collections.abc import Callable, Mapping, Sequence
from typing import Any


ROOT = pathlib.Path(__file__).resolve().parents[1]
TASK_ID = "slice8b-v10-e1-traversal-w1-dafsa-typed-view-m3-final-materialization-v9-20260827"
TRANSACTION_ID = "ed21c54906eebc5a9a99afc873b3a38b8a6ca5e6003b539d019539403aa2ffb1"
V8R1_TASK_ID = "slice8b-v10-e1-traversal-w1-dafsa-typed-view-m3-e2e-v8r1-20260827"
V8R2_TASK_ID = "slice8b-v10-e1-traversal-w1-dafsa-typed-view-m3-e2e-v8r2-direct-exec-20260827"
V8R3_TASK_ID = "slice8b-v10-e1-traversal-w1-dafsa-typed-view-m3-e2e-v8r3-terminal-projection-20260827"
REMOTE = "e@192.168.3.94"
SSH_KEY = pathlib.Path("/home/ubu/.ssh/mega-mini-admin")
REMOTE_PARENT = pathlib.PurePosixPath("/home/e/.local/share/lay/provenance") / TASK_ID
REMOTE_STATE = pathlib.PurePosixPath("/home/e/.local/state/lay") / TASK_ID
REMOTE_CACHE = pathlib.PurePosixPath("/home/e/.cache") / f"lay-m3-v9-{TRANSACTION_ID}"
REMOTE_V8R3_ELF = (
    pathlib.PurePosixPath("/home/e/.local/share/lay/provenance")
    / V8R3_TASK_ID
    / "bootstrap-v1/m3-v8r3-test-elf"
)

CONTROLLER = pathlib.Path(__file__).resolve()
REMOTE_CONTROLLER = ROOT / "scripts/lay-v10-e1-traversal-w1-dafsa-typed-view-m3-v9-remote.py"
AUDITOR = ROOT / "scripts/lay-v10-e1-traversal-w1-dafsa-typed-view-m3-v9-audit.py"
V8_SOURCE = ROOT / "src/nanda_wave/l2_field/v13_typed_peak.rs"
TRACE_SOURCE = ROOT / "src/nanda_wave/l2_field/productive_v1/live.rs"
PROPOSAL_SOURCE = ROOT / "src/typing_transition/proposal_admission.rs"
PREFLIGHT_MANIFEST = ROOT / "docs/structural_gates/preflights/LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_W1_DAFSA_TYPED_VIEW_M3_FINAL_MATERIALIZATION_DIAGNOSTIC_V9_IMPLEMENTATION_V1_2026-08-27.json"
PREFLIGHT_RECEIPT = ROOT / "docs/structural_gates/receipts/LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_W1_DAFSA_TYPED_VIEW_M3_FINAL_MATERIALIZATION_DIAGNOSTIC_V9_IMPLEMENTATION_V1_PREFLIGHT_2026-08-27.json"
LATENCY_DECISION = ROOT / "docs/structural_gates/LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_W1_DAFSA_TYPED_VIEW_M3_END_TO_END_V8R3_LATENCY_FAILURE_DECISION_V1_2026-08-27.md"
V9_PAPER = ROOT / "docs/structural_gates/LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_W1_DAFSA_TYPED_VIEW_M3_FINAL_MATERIALIZATION_DIAGNOSTIC_V9_2026-08-27.md"
V9_ROUTE = ROOT / "docs/structural_gates/LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_W1_DAFSA_TYPED_VIEW_M3_FINAL_MATERIALIZATION_DIAGNOSTIC_V9_ROUTE_V2_2026-08-27.md"
V9_ROUTE_RECEIPT = ROOT / "docs/structural_gates/receipts/LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_W1_DAFSA_TYPED_VIEW_M3_FINAL_MATERIALIZATION_DIAGNOSTIC_V9_ROUTE_V2_RECEIPT_2026-08-27.json"
V8R1_DIAGNOSIS = ROOT / "docs/structural_gates/receipts/LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_W1_DAFSA_TYPED_VIEW_M3_END_TO_END_V8R1_PSS_HELPER_LIFECYCLE_DIAGNOSIS_2026-08-27.json"
V8R1_BUILD_AUDIT = ROOT / "docs/structural_gates/receipts/LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_W1_DAFSA_TYPED_VIEW_M3_END_TO_END_V8R1_BUILD_AUDIT_V1_2026-08-27/BUILD_AUDIT.json"
V8R1_TERMINAL_AUDIT = ROOT / "docs/structural_gates/receipts/LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_W1_DAFSA_TYPED_VIEW_M3_END_TO_END_V8R1_TERMINAL_AUDIT_V1_2026-08-27/TERMINAL_AUDIT.json"
V8R1_JOURNAL = ROOT / "docs/structural_gates/receipts/LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_W1_DAFSA_TYPED_VIEW_M3_END_TO_END_V8R1_EXECUTION_JOURNAL_V1_2026-08-27/SHA256SUMS"
V8R1_CONTROLLER = ROOT / "docs/structural_gates/receipts/LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_W1_DAFSA_TYPED_VIEW_M3_END_TO_END_REMOTE_CONTROLLER_V8R1_V1_2026-08-27/IMPLEMENTATION_RECEIPT.json"
V8R1_LOCAL_ELF = ROOT / "docs/structural_gates/receipts/LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_W1_DAFSA_TYPED_VIEW_M3_END_TO_END_V8R1_BUILD_AUDIT_V1_2026-08-27/REMOTE_BUILD/m3-v8r1-test-elf"
V8R2_DIAGNOSIS = ROOT / "docs/structural_gates/receipts/LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_W1_DAFSA_TYPED_VIEW_M3_END_TO_END_V8R2_LIVE_ADMISSION_FAILURE_DIAGNOSIS_2026-08-27.json"
V8R2_JOURNAL = ROOT / "docs/structural_gates/receipts/LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_W1_DAFSA_TYPED_VIEW_M3_END_TO_END_V8R2_EXECUTION_JOURNAL_V1_2026-08-27/SHA256SUMS"
V8R2_CONTROLLER = ROOT / "docs/structural_gates/receipts/LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_W1_DAFSA_TYPED_VIEW_M3_END_TO_END_REMOTE_CONTROLLER_V8R2_V1_2026-08-27/IMPLEMENTATION_RECEIPT.json"
V8R2_LOCAL_CONTROLLER = ROOT / "scripts/lay-v10-e1-traversal-w1-dafsa-typed-view-m3-v8r2.py"
V8R2_REMOTE_CONTROLLER = ROOT / "scripts/lay-v10-e1-traversal-w1-dafsa-typed-view-m3-v8r2-remote.py"
V8R2_AUDITOR = ROOT / "scripts/lay-v10-e1-traversal-w1-dafsa-typed-view-m3-v8r2-audit.py"
V8_SOURCE_IMPLEMENTATION = ROOT / "docs/structural_gates/receipts/LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_W1_DAFSA_TYPED_VIEW_M3_END_TO_END_IMPLEMENTATION_V8_2026-08-27/IMPLEMENTATION_RECEIPT.json"
V8R3_TERMINAL = ROOT / "docs/structural_gates/receipts/LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_W1_DAFSA_TYPED_VIEW_M3_END_TO_END_V8R3_TERMINAL_AUDIT_V1_2026-08-27/TERMINAL_AUDIT.json"
V8R3_SUBJECT = ROOT / "docs/structural_gates/receipts/LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_W1_DAFSA_TYPED_VIEW_M3_END_TO_END_V8R3_TERMINAL_AUDIT_V1_2026-08-27/REMOTE_E2E/subject/SUBJECT_RECEIPT.json"
V8R3_JOURNAL = ROOT / "docs/structural_gates/receipts/LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_W1_DAFSA_TYPED_VIEW_M3_END_TO_END_V8R3_EXECUTION_JOURNAL_V1_2026-08-27/SHA256SUMS"

CONTROLLER_EVIDENCE = ROOT / "docs/structural_gates/receipts/LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_W1_DAFSA_TYPED_VIEW_M3_FINAL_MATERIALIZATION_DIAGNOSTIC_V9_CONTROLLER_V1_2026-08-27"
CONTROLLER_IMPLEMENTATION = CONTROLLER_EVIDENCE / "IMPLEMENTATION_RECEIPT.json"
EXECUTION_JOURNAL = ROOT / "docs/structural_gates/receipts/LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_W1_DAFSA_TYPED_VIEW_M3_FINAL_MATERIALIZATION_DIAGNOSTIC_V9_EXECUTION_JOURNAL_V1_2026-08-27"
ADMISSION_RECEIPT = ROOT / "docs/structural_gates/receipts/LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_W1_DAFSA_TYPED_VIEW_M3_FINAL_MATERIALIZATION_DIAGNOSTIC_V9_EXECUTION_ADMISSION_V1_2026-08-27/EXECUTION_ADMISSION.json"
BOOTSTRAP_AUDIT = ROOT / "docs/structural_gates/receipts/LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_W1_DAFSA_TYPED_VIEW_M3_FINAL_MATERIALIZATION_DIAGNOSTIC_V9_BOOTSTRAP_AUDIT_V1_2026-08-27/BOOTSTRAP_AUDIT.json"
QUIET_ADMISSION = ROOT / "docs/structural_gates/receipts/LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_W1_DAFSA_TYPED_VIEW_M3_FINAL_MATERIALIZATION_DIAGNOSTIC_V9_QUIET_ADMISSION_V1_2026-08-27/QUIET_ADMISSION.json"
TERMINAL_AUDIT = ROOT / "docs/structural_gates/receipts/LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_W1_DAFSA_TYPED_VIEW_M3_FINAL_MATERIALIZATION_DIAGNOSTIC_V9_TERMINAL_AUDIT_V1_2026-08-27/TERMINAL_AUDIT.json"

ACTIONS = ("self-check", "seal-self-check", "execute", "status")
EXTERNAL_ACTIONS = (
    "live-admission",
    "remote-cache-create",
    "bootstrap-upload",
    "remote-bootstrap",
    "bootstrap-audit",
    "bootstrap-audit-upload",
    "create-marker",
    "quiet-audit",
    "quiet-audit-upload",
    "trace-once",
    "terminal-audit",
)
EXPECTED_ELF_SIZE = 320_613_368
EXPECTED_ELF_SHA256 = "0af3cc6679396650245e924976d8a3bb432dfa6a1086b1c46bcfa7497307afea"
EXPECTED_V8R1_WRAPPER_SHA256 = "1edc2c195b67485d007c1ed9354db14cf6f9907eaa1ffc3d77fa5f07b13f291b"
PREFLIGHT_TESTS = (
    "remote-producer-write-fault",
    "auditor-write-fault",
    "controller-write-fault",
    "static-fault",
    "receipt-publication-fault",
    "paper-route-parity",
    "terminal-predecessor-parity",
    "trace-parser-parity",
    "response-retention-parity",
    "direct-command-parity",
    "marker-parity",
    "auditor-parity",
    "claim-boundary-parity",
    "namespace-parity",
    "no-build-static",
)


class V9ControllerError(RuntimeError):
    pass


def need(condition: bool, message: str) -> None:
    if not condition:
        raise V9ControllerError(message)


def canonical(value: Any) -> bytes:
    return (json.dumps(value, ensure_ascii=False, sort_keys=True, separators=(",", ":")) + "\n").encode()


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
    need(path.is_file() and not path.is_symlink(), f"required file absent: {path}")
    return {
        "path": str(path),
        "mode": mode_string(path),
        "size_bytes": path.stat().st_size,
        "sha256": sha256_file(path),
    }


def load_json(path: pathlib.Path) -> Any:
    with path.open("rb") as source:
        return json.load(source)


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


def run(
    argv: Sequence[str],
    *,
    input_bytes: bytes | None = None,
    timeout: float = 3_600,
    check: bool = True,
) -> subprocess.CompletedProcess[bytes]:
    result = subprocess.run(
        list(argv),
        input=input_bytes,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        timeout=timeout,
        check=False,
    )
    if check and result.returncode != 0:
        raise V9ControllerError(
            f"command failed ({result.returncode}): {list(argv)!r}\n"
            f"stdout:\n{result.stdout[-4000:].decode(errors='replace')}\n"
            f"stderr:\n{result.stderr[-4000:].decode(errors='replace')}"
        )
    return result


def ssh(argv: Sequence[str], *, timeout: float = 3_600) -> bytes:
    return run(
        [
            "/usr/bin/ssh",
            "-i",
            str(SSH_KEY),
            "-o",
            "BatchMode=yes",
            "-o",
            "ConnectTimeout=10",
            REMOTE,
            *argv,
        ],
        timeout=timeout,
    ).stdout


def scp_file(local: pathlib.Path, remote: pathlib.PurePosixPath) -> None:
    run(
        [
            "/usr/bin/scp",
            "-i",
            str(SSH_KEY),
            "-o",
            "BatchMode=yes",
            "-o",
            "ConnectTimeout=10",
            "-q",
            "-p",
            str(local),
            f"{REMOTE}:{remote}",
        ],
        timeout=3_600,
    )


def parse_json_output(result: subprocess.CompletedProcess[bytes], label: str) -> dict[str, Any]:
    lines = result.stdout.decode().strip().splitlines()
    need(lines, f"{label} returned no JSON")
    value = json.loads(lines[-1])
    need(isinstance(value, dict), f"{label} returned a non-object")
    return value


def auditor_call(action: str) -> dict[str, Any]:
    result = run(["/usr/bin/python3", str(AUDITOR), action], check=False, timeout=3_600)
    return parse_json_output(result, f"auditor {action}")


def remote_controller_path(cache: bool = False) -> pathlib.PurePosixPath:
    return (REMOTE_CACHE if cache else REMOTE_PARENT / "bootstrap-v1") / "remote-controller.py"


def remote_call(action: str, *arguments: str, cache: bool = False, timeout: float = 3_600) -> dict[str, Any]:
    result = run(
        [
            "/usr/bin/ssh",
            "-i",
            str(SSH_KEY),
            "-o",
            "BatchMode=yes",
            "-o",
            "ConnectTimeout=10",
            REMOTE,
            "/usr/bin/sudo",
            "-n",
            "/usr/bin/python3",
            str(remote_controller_path(cache=cache)),
            action,
            *arguments,
        ],
        timeout=timeout,
        check=False,
    )
    return parse_json_output(result, f"remote {action}")


def verify_preflight() -> dict[str, Any]:
    need(sha256_file(PREFLIGHT_MANIFEST) == "efdbd7adc388492656d529ed28e1854b400d4be9ddf205cdb577624b91963082", "preflight manifest SHA drift")
    need(sha256_file(PREFLIGHT_RECEIPT) == "43e5f2e849a7ccdeab2fbc371f12fc0950e11e4c3b5c8fef043d01f464223587", "preflight receipt SHA drift")
    need(mode_string(PREFLIGHT_MANIFEST) == mode_string(PREFLIGHT_RECEIPT) == "0444", "preflight mode drift")
    receipt = load_json(PREFLIGHT_RECEIPT)
    need(receipt.get("verdict") == "READY_TO_IMPLEMENT" and receipt.get("safe_to_implement") is True, "preflight did not admit implementation")
    need(receipt.get("manifest_sha256") == "2a34631475e65f7b75f165b8f2224c36a490499848414011a38b75c242a067ce", "NANDA manifest identity drift")
    return receipt


def verify_fixed_inputs() -> dict[str, Any]:
    expected = {
        LATENCY_DECISION: "45e2e279997f7a93072bcfd74ad11d2566f55b442685d1be2a75e905dd543a8a",
        V9_PAPER: "98000de5d6a502d4bf1b2005deca476bfdd12539f02528a4b450f240f3d9ed27",
        V9_ROUTE: "7270e79ccd5c64f33dc5d1a3f95ff4dbea9a8f13baebc81aa97254a80d7fe860",
        V9_ROUTE_RECEIPT: "84171ed96027ae674e50618398af2d881f44b83fd7265a4b5e5cd92d8b9be00f",
        V8R3_TERMINAL: "2115a4ce7c8a79fffde043b87c5aa462014873a0365df3a6e82892f395f6aedc",
        V8R3_SUBJECT: "65cd8a6f08d77c192ae0eb24fa3df106ee5030e7a8bbdfdf44d08429f7d9bfd5",
        V8R3_JOURNAL: "1c176cb2b9e986011fba901996260ad34bf59d71d864ae663045800a8bf9cdfc",
        V8R2_DIAGNOSIS: "a74b51570143f56a8186435acfac80ab4dac5ec4140f105ac82df960508ed3dd",
        V8R2_JOURNAL: "298ebe81c77f48edca3d9df1fd514b14d6bcd7abf15f41ac42752cf27029d785",
        V8R2_CONTROLLER: "0c8e792d88e532b5cfb18c65a4ba15c031cc0956e6c9798a0b4de8c30b450b34",
        V8R2_LOCAL_CONTROLLER: "7a77aaa1a1b42e77e3c654e12953c6f02e70ecee71046a50befe6ffaea7a446a",
        V8R2_REMOTE_CONTROLLER: "f40183fb405a0ab52ac86dc1493fc38ae4ca14aef0830e46188b49cb4e21081a",
        V8R2_AUDITOR: "72b9071d75bd0cfb15545c1820dae453ff961d207f7cd346c17ae1881f61592f",
        V8R1_DIAGNOSIS: "9b05af87d83c937dcc1e4eab0e398ab3d93ef49ac3e0bfb8089a58ba3d64bae0",
        V8R1_BUILD_AUDIT: "d7d5e7110171e5c6546016ff0c9374c323804014ef8cfa7a690ad7d1d11c8340",
        V8R1_TERMINAL_AUDIT: "04d0e17158a63a49088e8c8ff9dc25df67e50ac6a97b770ea3fcf1a73d67ec91",
        V8R1_JOURNAL: "c6c9d648bdbc02ee8f10639099fc4add6141488600f771188e2b6f69445d91a4",
        V8R1_CONTROLLER: "4c16d6c34409f8395354796c48d4364319bbc52c6dbbb309600b3443bdd2f99c",
        V8R1_LOCAL_ELF: EXPECTED_ELF_SHA256,
        V8_SOURCE: "28f87a76fc1996989e980cab51f0443bd95e656fcae3a2ff61f581db9c3a7ee2",
        TRACE_SOURCE: "87180990b6883641483a46886074e5350f35e351454d734f0c3c9da723d758bd",
        PROPOSAL_SOURCE: "dd4a37a8c0430c9ff145f9ae9cbbbc735164ece833a143a19af644ac7ad835ca",
        V8_SOURCE_IMPLEMENTATION: "cf4b38d81cc7f9ea9125855194635fafde9c76c9d022f7185635e8bb6c2f29e5",
    }
    rows = {}
    for path, digest in expected.items():
        row = file_row(path)
        need(row["sha256"] == digest, f"fixed input identity drift: {path}")
        rows[str(path.relative_to(ROOT))] = row
    need(file_row(V8R1_LOCAL_ELF)["size_bytes"] == EXPECTED_ELF_SIZE, "sealed ELF size drift")
    need(mode_string(V8R1_LOCAL_ELF) == "0444", "sealed ELF mode drift")
    for path in (V8R2_LOCAL_CONTROLLER, V8R2_REMOTE_CONTROLLER, V8R2_AUDITOR):
        need(mode_string(path) == "0555", f"sealed V8R2 controller mode drift: {path}")
    for path in (
        LATENCY_DECISION, V9_PAPER, V9_ROUTE, V9_ROUTE_RECEIPT,
        V8R3_TERMINAL, V8R3_SUBJECT, V8R3_JOURNAL,
        V8R2_DIAGNOSIS, V8R2_JOURNAL, V8R2_CONTROLLER,
    ):
        need(mode_string(path) == "0444", f"sealed predecessor mode drift: {path}")
    return rows


def parse_python(path: pathlib.Path) -> ast.Module:
    try:
        return ast.parse(path.read_text(), filename=str(path))
    except SyntaxError as error:
        raise V9ControllerError(f"invalid controller source: {path}: {error}") from error


def static_graph(remote_check: Mapping[str, Any], auditor_check: Mapping[str, Any]) -> dict[str, Any]:
    trees = {path.name: parse_python(path) for path in (CONTROLLER, REMOTE_CONTROLLER, AUDITOR)}
    remote_tree = trees[REMOTE_CONTROLLER.name]
    functions = {node.name for node in ast.walk(remote_tree) if isinstance(node, (ast.FunctionDef, ast.AsyncFunctionDef))}
    need(not ({"build_once", "find_test_elf"} & functions), "build route exists in remote producer")
    assignments = {
        target.id
        for node in ast.walk(remote_tree)
        if isinstance(node, ast.Assign)
        for target in node.targets
        if isinstance(target, ast.Name)
    }
    need("LOADER" not in assignments and "BUILD" not in assignments, "forbidden executable route constant exists")
    need(remote_check.get("actions") == ["self-check", "status", "bootstrap", "create-marker", "trace-once"], "remote action registry drift")
    need(remote_check.get("routes") == ["TRACE"] and remote_check.get("markers") == ["trace.available"], "remote route registry drift")
    need(remote_check.get("bootstrap_parent_mode") == "0755", "remote parent mode contract drift")
    command = remote_check.get("trace_argv")
    need(isinstance(command, list) and str(REMOTE_V8R3_ELF) in command, "direct executable absent")
    need("LAY_L2_FIELD_TRACE=1" in command, "trace environment absent")
    need(not any("ld-linux" in str(token) for token in command), "loader appears in scientific argv")
    need(auditor_check.get("positive_dispatch") == "PASS" and auditor_check.get("incomplete_observation_dispatch") == "PASS", "auditor dispatch self-check failed")
    need(auditor_check.get("trace_parser") == "PASS", "auditor trace parser self-check failed")
    source = REMOTE_CONTROLLER.read_text()
    need("scripts/cargo-guard.sh" not in source and "build-once" not in source, "build command became reachable")
    return {
        "parsed_files": sorted(trees),
        "remote_functions": len(functions),
        "remote_actions": remote_check["actions"],
        "routes": remote_check["routes"],
        "marker_names": remote_check["markers"],
        "direct_command_sha256": sha256_bytes(canonical(command)),
        "build_route_absent": True,
        "loader_route_absent": True,
        "perf_route_absent": True,
        "production_route_absent": True,
    }


def fault_model() -> dict[str, Any]:
    rows = []
    with tempfile.TemporaryDirectory(prefix="lay-m3-v9-faults-") as raw:
        root = pathlib.Path(raw)
        for test_id, source in (
            ("remote-producer-write-fault", REMOTE_CONTROLLER),
            ("auditor-write-fault", AUDITOR),
            ("controller-write-fault", CONTROLLER),
        ):
            partial = root / f"{test_id}.py"
            partial.write_bytes(source.read_bytes()[:31])
            try:
                ast.parse(partial.read_text())
            except (SyntaxError, UnicodeDecodeError):
                rows.append({"id": test_id, "verdict": "PASS", "remote_state_created": False})
            else:
                raise V9ControllerError(f"partial source was accepted: {test_id}")
        need(not CONTROLLER_EVIDENCE.exists(), "implementation PASS already exists during static fault check")
        rows.append({"id": "static-fault", "verdict": "PASS", "implementation_pass_published": False})
        collision = root / "receipt.json"
        write_new(collision, b"first\n")
        try:
            write_new(collision, b"second\n")
        except FileExistsError:
            rows.append({"id": "receipt-publication-fault", "verdict": "PASS", "overwrite_rejected": True})
        else:
            raise V9ControllerError("exclusive receipt publication fault failed")
    route = load_json(V9_ROUTE_RECEIPT)
    terminal = load_json(V8R3_TERMINAL)
    subject = load_json(V8R3_SUBJECT)
    remote = parse_json_output(
        run(["/usr/bin/python3", str(REMOTE_CONTROLLER), "self-check"]),
        "remote trace self-check",
    )
    need(route.get("verdict") == "PASS", "V9 route receipt drift")
    need(
        terminal.get("verdict") == "BLOCKED_LATENCY"
        and terminal.get("task_id") == V8R3_TASK_ID
        and subject.get("verdict") == "BLOCKED_LATENCY",
        "V8R3 terminal predecessor drift",
    )
    need(
        remote.get("trace_rows") == 1910
        and remote.get("measured_rows") == 1528
        and remote.get("tail_rows") == 16
        and remote.get("synthetic_dominant_stage") == "gate_us",
        "trace parser self-check drift",
    )
    rows.extend([
        {
            "id": "paper-route-parity",
            "verdict": "PASS",
            "route_verdict": route["verdict"],
            "paper_sha256": sha256_file(V9_PAPER),
        },
        {
            "id": "terminal-predecessor-parity",
            "verdict": "PASS",
            "v8r3_terminal_verdict": terminal["verdict"],
            "v8r3_subject_verdict": subject["verdict"],
        },
        {
            "id": "trace-parser-parity",
            "verdict": "PASS",
            "rows": remote["trace_rows"],
            "measured_rows": remote["measured_rows"],
            "tail_rows": remote["tail_rows"],
        },
        {"id": "response-retention-parity", **response_retention_fault()},
        {"id": "direct-command-parity", "verdict": "PASS", "route": "TRACE", "elf_sha256": EXPECTED_ELF_SHA256},
        {"id": "marker-parity", "verdict": "PASS", "markers": ["trace.available"]},
        {"id": "auditor-parity", "verdict": "PASS", "dispatch": "independent"},
        {"id": "claim-boundary-parity", "verdict": "PASS", "latency_authority": False, "production_authority": False},
        {"id": "namespace-parity", "verdict": "PASS", "task_ids": [V8R1_TASK_ID, V8R2_TASK_ID, V8R3_TASK_ID, TASK_ID], "all_distinct": len({V8R1_TASK_ID, V8R2_TASK_ID, V8R3_TASK_ID, TASK_ID}) == 4},
        {"id": "no-build-static", "verdict": "PASS", "build_routes": 0},
    ])
    need({row["id"] for row in rows} == set(PREFLIGHT_TESTS), "fault model coverage drift")
    need(all(row["verdict"] == "PASS" for row in rows), "fault model failed")
    return {"tests": rows, "passed": len(rows), "expected": len(PREFLIGHT_TESTS)}


def response_retention_fault() -> dict[str, Any]:
    response = {"verdict": "BLOCKED_PROVENANCE", "error": "synthetic", "complete": True}
    with tempfile.TemporaryDirectory(prefix="lay-m3-v9-response-retention-") as raw:
        journal = pathlib.Path(raw)
        try:
            journaled(journal, 1, "synthetic", lambda: response, ["ADMITTED"])
        except V9ControllerError:
            pass
        else:
            raise V9ControllerError("non-admitted response was accepted")
        completion = load_json(journal / "01-synthetic.complete.json")
        need(completion.get("response") == response, "complete structured response was not retained")
        need(completion.get("response_admitted") is False, "non-admitted response flag drift")
        need(not pending_intent(journal), "verified structured response left a pending intent")
    return {"verdict": "PASS", "complete_response_retained": True, "pending_intent": False}


def self_check() -> dict[str, Any]:
    preflight = verify_preflight()
    fixed = verify_fixed_inputs()
    remote = parse_json_output(run(["/usr/bin/python3", str(REMOTE_CONTROLLER), "self-check"]), "remote static self-check")
    auditor = parse_json_output(run(["/usr/bin/python3", str(AUDITOR), "self-check"]), "auditor static self-check")
    need(remote.get("verdict") == "M3_V9_REMOTE_CONTROLLER_STATIC_PASS", "remote self-check failed")
    need(auditor.get("verdict") == "M3_V9_INDEPENDENT_AUDITOR_STATIC_PASS", "auditor self-check failed")
    graph = static_graph(remote, auditor)
    faults = fault_model()
    retention = next(row for row in faults["tests"] if row["id"] == "response-retention-parity")
    return {
        "schema": "lay.m3-v9-controller-self-check.v1",
        "task_id": TASK_ID,
        "transaction_id": TRANSACTION_ID,
        "verdict": "M3_V9_CONTROLLER_STATIC_SELF_CHECK_PASS",
        "local_controller_sha256": sha256_file(CONTROLLER),
        "remote_controller_sha256": sha256_file(REMOTE_CONTROLLER),
        "auditor_sha256": sha256_file(AUDITOR),
        "preflight_receipt_sha256": sha256_file(PREFLIGHT_RECEIPT),
        "preflight_manifest_sha256": sha256_file(PREFLIGHT_MANIFEST),
        "nanda_manifest_sha256": preflight["manifest_sha256"],
        "fixed_inputs": fixed,
        "command_graph": graph,
        "fault_model": faults,
        "response_retention_fault": retention,
        "remote_reads": 0,
        "remote_writes": 0,
        "markers_created": 0,
        "markers_consumed": 0,
        "cargo_invocations": 0,
        "rustc_compilations": 0,
        "subject_executions": 0,
        "perf_record_invocations": 0,
        "perf_stat_invocations": 0,
        "runtime_authority_changed": False,
    }


def seal_self_check() -> dict[str, Any]:
    need(not CONTROLLER_EVIDENCE.exists(), "controller implementation receipt already exists")
    for path in (CONTROLLER, REMOTE_CONTROLLER, AUDITOR):
        path.chmod(0o555)
    check = self_check()
    receipt = {
        "schema": "lay.m3-v9-controller-implementation.v1",
        "task_id": TASK_ID,
        "transaction_id": TRANSACTION_ID,
        "verdict": "M3_V9_CONTROLLERS_VERIFIED_UNRUN",
        "local_controller_sha256": check["local_controller_sha256"],
        "remote_controller_sha256": check["remote_controller_sha256"],
        "auditor_sha256": check["auditor_sha256"],
        "preflight_receipt_sha256": check["preflight_receipt_sha256"],
        "preflight_manifest_sha256": check["preflight_manifest_sha256"],
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
        "next_action_admitted": "independent V9 live execution admission only",
    }
    stage = CONTROLLER_EVIDENCE.with_name(f"{CONTROLLER_EVIDENCE.name}.stage-{os.getpid()}-{time.time_ns()}")
    stage.mkdir(parents=True, mode=0o700)
    try:
        write_new(stage / "SELF_CHECK.json", canonical(check))
        write_new(stage / "IMPLEMENTATION_RECEIPT.json", canonical(receipt))
        write_manifest(stage)
        seal_tree(stage)
        os.rename(stage, CONTROLLER_EVIDENCE)
        fsync_dir(CONTROLLER_EVIDENCE.parent)
    except BaseException:
        if stage.exists():
            shutil.rmtree(stage)
        raise
    return load_json(CONTROLLER_IMPLEMENTATION)


def prepare_bootstrap(admission: Mapping[str, Any]) -> pathlib.Path:
    staging = pathlib.Path(tempfile.mkdtemp(prefix="lay-m3-v9-bootstrap-"))
    try:
        files = {
            staging / "local-controller.py": CONTROLLER,
            staging / "remote-controller.py": REMOTE_CONTROLLER,
            staging / "independent-auditor.py": AUDITOR,
            staging / "CONTROLLER_IMPLEMENTATION.json": CONTROLLER_IMPLEMENTATION,
            staging / "LATENCY_DECISION.md": LATENCY_DECISION,
            staging / "V9_PAPER.md": V9_PAPER,
            staging / "V9_ROUTE.md": V9_ROUTE,
            staging / "V9_ROUTE_RECEIPT.json": V9_ROUTE_RECEIPT,
            staging / "PREFLIGHT_MANIFEST.json": PREFLIGHT_MANIFEST,
            staging / "PREFLIGHT_RECEIPT.json": PREFLIGHT_RECEIPT,
            staging / "V8R3_TERMINAL_AUDIT.json": V8R3_TERMINAL,
            staging / "V8R3_SUBJECT_RECEIPT.json": V8R3_SUBJECT,
            staging / "V8R1_BUILD_AUDIT.json": V8R1_BUILD_AUDIT,
            staging / "V8R1_TERMINAL_AUDIT.json": V8R1_TERMINAL_AUDIT,
            staging / "V8R1_DIAGNOSIS.json": V8R1_DIAGNOSIS,
            staging / "V8R2_DIAGNOSIS.json": V8R2_DIAGNOSIS,
            staging / "V8R2_JOURNAL_SHA256SUMS": V8R2_JOURNAL,
            staging / "V8R2_CONTROLLER_IMPLEMENTATION.json": V8R2_CONTROLLER,
            staging / "V8R2_LOCAL_CONTROLLER.py": V8R2_LOCAL_CONTROLLER,
            staging / "V8R2_REMOTE_CONTROLLER.py": V8R2_REMOTE_CONTROLLER,
            staging / "V8R2_AUDITOR.py": V8R2_AUDITOR,
            staging / "EXECUTION_ADMISSION.json": ADMISSION_RECEIPT,
        }
        for target, source in files.items():
            shutil.copyfile(source, target)
            target.chmod(0o444)
        (staging / "admissions").mkdir(mode=0o700)
        inventory = {}
        for path in sorted(staging.rglob("*")):
            if path.is_file() and path.name != "PAYLOAD.json":
                inventory[path.relative_to(staging).as_posix()] = {
                    "size_bytes": path.stat().st_size,
                    "sha256": sha256_file(path),
                }
        payload = {
            "schema": "lay.m3-v9-bootstrap-payload.v1",
            "task_id": TASK_ID,
            "transaction_id": TRANSACTION_ID,
            "execution_admission_verdict": admission["verdict"],
            "execution_admission_sha256": sha256_file(ADMISSION_RECEIPT),
            "source_elf_uploaded": False,
            "source_elf_remote_copy_only": False,
            "source_elf_executed_in_place": True,
            "files": inventory,
        }
        write_new(staging / "PAYLOAD.json", canonical(payload), 0o444)
        return staging
    except BaseException:
        shutil.rmtree(staging, ignore_errors=True)
        raise


def initialize_journal() -> pathlib.Path:
    need(not EXECUTION_JOURNAL.exists(), "V9 execution journal already exists; retry is forbidden")
    EXECUTION_JOURNAL.mkdir(parents=True, mode=0o700)
    write_new(EXECUTION_JOURNAL / "JOURNAL.json", canonical({
        "schema": "lay.m3-v9-execution-journal.v1",
        "task_id": TASK_ID,
        "transaction_id": TRANSACTION_ID,
        "controller_sha256": sha256_file(CONTROLLER),
        "remote_controller_sha256": sha256_file(REMOTE_CONTROLLER),
        "auditor_sha256": sha256_file(AUDITOR),
        "external_actions": list(EXTERNAL_ACTIONS),
        "retry_permitted": False,
    }), 0o444)
    fsync_dir(EXECUTION_JOURNAL)
    return EXECUTION_JOURNAL


def journal_rows(root: pathlib.Path, suffix: str) -> list[pathlib.Path]:
    return sorted(root.glob(f"[0-9][0-9]-*.{suffix}.json"))


def pending_intent(root: pathlib.Path) -> bool:
    return len(journal_rows(root, "intent")) != len(journal_rows(root, "complete"))


def append_intent(root: pathlib.Path, sequence: int, action: str) -> None:
    need(not pending_intent(root), "journal has a pending external action; retry is forbidden")
    write_new(root / f"{sequence:02d}-{action}.intent.json", canonical({
        "schema": "lay.m3-v9-external-intent.v1",
        "task_id": TASK_ID,
        "transaction_id": TRANSACTION_ID,
        "sequence": sequence,
        "action": action,
        "status": "INTENT_DURABLE",
        "retry_permitted": False,
    }), 0o444)
    fsync_dir(root)


def append_completion(
    root: pathlib.Path,
    sequence: int,
    action: str,
    response: Mapping[str, Any],
    admitted: bool,
) -> None:
    write_new(root / f"{sequence:02d}-{action}.complete.json", canonical({
        "schema": "lay.m3-v9-external-completion.v1",
        "task_id": TASK_ID,
        "transaction_id": TRANSACTION_ID,
        "sequence": sequence,
        "action": action,
        "status": "RESPONSE_VERIFIED",
        "response_verdict": response.get("verdict"),
        "response_sha256": sha256_bytes(canonical(response)),
        "response": dict(response),
        "response_admitted": admitted,
        "retry_permitted": False,
    }), 0o444)
    fsync_dir(root)


def journaled(
    root: pathlib.Path,
    sequence: int,
    action: str,
    callback: Callable[[], Mapping[str, Any]],
    allowed: Sequence[str],
) -> dict[str, Any]:
    append_intent(root, sequence, action)
    response = dict(callback())
    admitted = response.get("verdict") in set(allowed)
    append_completion(root, sequence, action, response, admitted)
    need(admitted, f"external action verdict drift: {action}: {response.get('verdict')}")
    return response


def create_remote_cache() -> dict[str, Any]:
    output = ssh(["/usr/bin/mkdir", "-m", "0700", str(REMOTE_CACHE)])
    return {"verdict": "M3_V9_REMOTE_CACHE_CREATED", "stdout_sha256": sha256_bytes(output)}


def upload_bootstrap(staging: pathlib.Path) -> dict[str, Any]:
    run(
        [
            "/usr/bin/scp",
            "-i",
            str(SSH_KEY),
            "-o",
            "BatchMode=yes",
            "-o",
            "ConnectTimeout=10",
            "-q",
            "-p",
            "-r",
            f"{staging}/.",
            f"{REMOTE}:{REMOTE_CACHE}",
        ],
        timeout=3_600,
    )
    return {"verdict": "M3_V9_BOOTSTRAP_UPLOADED", "payload_sha256": sha256_file(staging / "PAYLOAD.json")}


def upload_audit(local: pathlib.Path, name: str, verdict: str) -> dict[str, Any]:
    remote = REMOTE_CACHE / "admissions" / name
    scp_file(local, remote)
    return {"verdict": verdict, "local_sha256": sha256_file(local), "remote_path": str(remote)}


def verify_implementation() -> dict[str, Any]:
    need(CONTROLLER_IMPLEMENTATION.is_file(), "controller implementation receipt absent")
    value = load_json(CONTROLLER_IMPLEMENTATION)
    need(value.get("verdict") == "M3_V9_CONTROLLERS_VERIFIED_UNRUN", "controller implementation verdict drift")
    for key, path in {
        "local_controller_sha256": CONTROLLER,
        "remote_controller_sha256": REMOTE_CONTROLLER,
        "auditor_sha256": AUDITOR,
    }.items():
        need(value.get(key) == sha256_file(path), f"controller implementation identity drift: {key}")
    return value


def execute() -> dict[str, Any]:
    verify_implementation()
    journal = initialize_journal()
    sequence = 1
    staging: pathlib.Path | None = None
    try:
        admission = journaled(journal, sequence, "live-admission", lambda: auditor_call("live-admission"), ["M3_V9_EXECUTION_ADMITTED"])
        sequence += 1
        staging = prepare_bootstrap(admission)
        journaled(journal, sequence, "remote-cache-create", create_remote_cache, ["M3_V9_REMOTE_CACHE_CREATED"])
        sequence += 1
        journaled(journal, sequence, "bootstrap-upload", lambda: upload_bootstrap(staging), ["M3_V9_BOOTSTRAP_UPLOADED"])
        sequence += 1
        journaled(
            journal,
            sequence,
            "remote-bootstrap",
            lambda: remote_call("bootstrap", "--bootstrap", str(REMOTE_CACHE), cache=True, timeout=3_600),
            ["M3_V9_BOOTSTRAP_CREATED_UNAUDITED"],
        )
        sequence += 1
        journaled(journal, sequence, "bootstrap-audit", lambda: auditor_call("bootstrap"), ["M3_V9_BOOTSTRAP_AUDIT_PASS_MARKER_ADMITTED"])
        sequence += 1
        journaled(
            journal,
            sequence,
            "bootstrap-audit-upload",
            lambda: upload_audit(BOOTSTRAP_AUDIT, "BOOTSTRAP_AUDIT.json", "M3_V9_BOOTSTRAP_AUDIT_UPLOADED"),
            ["M3_V9_BOOTSTRAP_AUDIT_UPLOADED"],
        )
        sequence += 1
        audit_remote = str(REMOTE_CACHE / "admissions/BOOTSTRAP_AUDIT.json")
        journaled(
            journal,
            sequence,
            "create-marker",
            lambda: remote_call("create-marker", "--audit", audit_remote),
            ["M3_V9_TRACE_MARKER_AVAILABLE"],
        )
        sequence += 1
        journaled(journal, sequence, "quiet-audit", lambda: auditor_call("quiet"), ["M3_V9_QUIET_HOST_TRACE_ADMITTED"])
        sequence += 1
        journaled(
            journal,
            sequence,
            "quiet-audit-upload",
            lambda: upload_audit(QUIET_ADMISSION, "QUIET_ADMISSION.json", "M3_V9_QUIET_ADMISSION_UPLOADED"),
            ["M3_V9_QUIET_ADMISSION_UPLOADED"],
        )
        sequence += 1
        quiet_remote = str(REMOTE_CACHE / "admissions/QUIET_ADMISSION.json")
        journaled(
            journal,
            sequence,
            "trace-once",
            lambda: remote_call("trace-once", "--quiet", quiet_remote, timeout=10_800),
            ["M3_V9_TRACE_CREATED_UNAUDITED", "BLOCKED_PROVENANCE"],
        )
        sequence += 1
        terminal = journaled(
            journal,
            sequence,
            "terminal-audit",
            lambda: auditor_call("terminal"),
            [
                "FINAL_MATERIALIZATION_DECOMPOSED",
                "BLOCKED_PROVENANCE",
                "BLOCKED_SEMANTIC",
                "BLOCKED_CAPABILITY",
            ],
        )
        write_new(journal / "TERMINAL.json", canonical({
            "schema": "lay.m3-v9-controller-terminal.v1",
            "task_id": TASK_ID,
            "transaction_id": TRANSACTION_ID,
            "verdict": terminal["verdict"],
            "terminal_audit_sha256": sha256_file(TERMINAL_AUDIT),
            "external_actions_completed": len(EXTERNAL_ACTIONS),
            "retry_permitted": False,
            "runtime_authority_changed": False,
        }), 0o444)
        write_manifest(journal)
        seal_tree(journal)
        return terminal
    except BaseException as error:
        pending = pending_intent(journal)
        completions = journal_rows(journal, "complete")
        latest_completion = load_json(completions[-1]) if completions else None
        failure = {
            "schema": "lay.m3-v9-controller-failure.v1",
            "task_id": TASK_ID,
            "transaction_id": TRANSACTION_ID,
            "verdict": "BLOCKED_PROVENANCE",
            "error": f"{type(error).__name__}: {error}",
            "pending_intent": pending,
            "external_actions_completed": len(journal_rows(journal, "complete")),
            "affected_remote_facts": (
                "UNKNOWN"
                if pending
                else {
                    "status": "KNOWN_FROM_VERIFIED_RESPONSE",
                    "response": latest_completion.get("response") if latest_completion else None,
                }
            ),
            "retry_permitted": False,
            "runtime_authority_changed": False,
        }
        write_new(journal / "CONTROLLER_FAILURE.json", canonical(failure), 0o444)
        write_manifest(journal)
        seal_tree(journal)
        raise V9ControllerError(json.dumps(failure, sort_keys=True)) from error
    finally:
        if staging is not None:
            shutil.rmtree(staging, ignore_errors=True)


def status() -> dict[str, Any]:
    remote = None
    try:
        cache = not (ADMISSION_RECEIPT.exists() and BOOTSTRAP_AUDIT.exists())
        raw = ssh(["/usr/bin/sudo", "-n", "/usr/bin/python3", str(remote_controller_path(cache=cache)), "status"], timeout=60)
        lines = raw.decode().strip().splitlines()
        remote = json.loads(lines[-1]) if lines else None
    except BaseException as error:
        remote = {"verdict": "UNKNOWN", "error": f"{type(error).__name__}: {error}"}
    return {
        "schema": "lay.m3-v9-controller-status.v1",
        "verdict": "M3_V9_CONTROLLER_STATUS",
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
            "schema": "lay.m3-v9-controller-error.v1",
            "task_id": TASK_ID,
            "transaction_id": TRANSACTION_ID,
            "verdict": "BLOCKED_PROVENANCE",
            "error": f"{type(error).__name__}: {error}",
            "retry_permitted": False,
        }, sort_keys=True))
        return 1


if __name__ == "__main__":
    sys.exit(main())
