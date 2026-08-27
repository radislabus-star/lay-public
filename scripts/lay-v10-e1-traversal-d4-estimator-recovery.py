#!/usr/bin/env python3
"""Local controller for D4 single-worker estimator recovery."""

from __future__ import annotations

import argparse
import ast
import base64
import hashlib
import json
import os
import pathlib
import shlex
import shutil
import stat
import subprocess
import sys
import time
from typing import Any, Mapping, Sequence


PROJECT_ROOT = pathlib.Path(__file__).resolve().parents[1]
TASK_ID = "slice8b-v10-e1-traversal-d4-estimator-recovery-v1-20260826"
TRANSACTION_ID = "2d3002b7cf615459a4250d7e44eb2094863dc422f908080b7afa59551ba4ee26"
REMOTE = "e@192.168.3.94"
REMOTE_HOSTNAME = "e-MEGA-MINI-M1-13th"
SSH_IDENTITY = pathlib.Path("/home/ubu/.ssh/mega-mini-admin")

CONTROLLER = PROJECT_ROOT / "scripts/lay-v10-e1-traversal-d4-estimator-recovery.py"
REMOTE_CONTROLLER = PROJECT_ROOT / "scripts/lay-v10-e1-traversal-d4-estimator-recovery-remote.py"
BOOTSTRAP_AUDITOR = PROJECT_ROOT / "scripts/lay-v10-e1-traversal-d4-bootstrap-audit.py"
MARKER_AUDITOR = PROJECT_ROOT / "scripts/lay-v10-e1-traversal-d4-marker-audit.py"
PAPER = PROJECT_ROOT / (
    "docs/structural_gates/"
    "LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_D4_TASK_CLOCK_ESTIMATOR_RECOVERY_V1_2026-08-26.md"
)
PREFLIGHT = PROJECT_ROOT / (
    "docs/structural_gates/preflights/"
    "LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_D4_ESTIMATOR_RECOVERY_IMPLEMENTATION_V2_2026-08-26.json"
)
PREFLIGHT_RECEIPT = PROJECT_ROOT / (
    "docs/structural_gates/receipts/"
    "LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_D4_ESTIMATOR_RECOVERY_IMPLEMENTATION_V2_PREFLIGHT_2026-08-26.json"
)
D2_TERMINAL = PROJECT_ROOT / (
    "docs/structural_gates/receipts/"
    "LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_D2_PRIMARY_ONLY_"
    "T_SINGLE_TERMINAL_AUDIT_V1_2026-08-26/T_SINGLE_TERMINAL_AUDIT_RECEIPT.json"
)
D3_TERMINAL = PROJECT_ROOT / (
    "docs/structural_gates/receipts/"
    "LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_D3_TERMINAL_AUDIT_V1_2026-08-26/"
    "D3_TERMINAL_AUDIT_RECEIPT.json"
)
LOCAL_ELF = PROJECT_ROOT / (
    "docs/structural_gates/receipts/"
    "LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_D2_PRIMARY_ONLY_BUILD_V1_2026-08-25/"
    "REMOTE_EVIDENCE/d2-test-elf"
)
LOCAL_MAP = PROJECT_ROOT / (
    "docs/structural_gates/receipts/"
    "LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_D2_PRIMARY_ONLY_BUCKET_MAP_V1_2026-08-26/"
    "REMOTE_EVIDENCE/D2_BUCKET_MAP.json"
)
BOOTSTRAP_AUDIT_RECEIPT = PROJECT_ROOT / (
    "docs/structural_gates/receipts/"
    "LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_D4_BOOTSTRAP_AUDIT_V1_2026-08-26/"
    "D4_BOOTSTRAP_AUDIT_RECEIPT.json"
)
MARKER_AUDIT_RECEIPT = PROJECT_ROOT / (
    "docs/structural_gates/receipts/"
    "LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_D4_MARKER_AUDIT_V1_2026-08-26/"
    "D4_MARKER_AUDIT_RECEIPT.json"
)

PAPER_SHA256 = "50b5cde0514d356375464f8155905201c395112c88a8d5022c70913b5da4d7b3"
PREFLIGHT_SHA256 = "109d284d0031df29bd1747571690fc410fba72381fe73d4adf647e550ce86a78"
PREFLIGHT_RECEIPT_SHA256 = "21dc062bf9996bd793d9392e46fc747dc89b8c5878275d257e578e0dc501c282"
D2_TERMINAL_SHA256 = "75dc2703e279524a76c49a455b6081634d1c67274106da45f0d7d610af95e608"
D3_TERMINAL_SHA256 = "7f4c0fd9466cced361c21858ea2017910adc7b92db7c1edfe02f3be09d5c4299"
ELF_SHA256 = "bb7c68528ec3fec7074919c3c64cce233d48c26efefe9d9b32735c5f5865a178"
MAP_SHA256 = "2b93aedf11101a8bc5fc7ed4ffaa056b50476d96cdd2eabf9d185f59ee60f846"

REMOTE_PARENT = pathlib.PurePosixPath("/home/e/.local/share/lay/provenance") / TASK_ID
LOCAL_BOOTSTRAP = PROJECT_ROOT / (
    "docs/structural_gates/receipts/"
    "LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_D4_BOOTSTRAP_V1_2026-08-26"
)
LOCAL_MARKER_CREATION = PROJECT_ROOT / (
    "docs/structural_gates/receipts/"
    "LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_D4_MARKER_CREATION_V1_2026-08-26"
)
LOCAL_RESULTS = {
    "U3-SINGLE": PROJECT_ROOT / (
        "docs/structural_gates/receipts/"
        "LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_D4_U3_SINGLE_V1_2026-08-26"
    ),
    "T3-SINGLE": PROJECT_ROOT / (
        "docs/structural_gates/receipts/"
        "LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_D4_T3_SINGLE_V1_2026-08-26"
    ),
}
REMOTE_RESULTS = {
    "BOOTSTRAP": REMOTE_PARENT / "bootstrap-v1",
    "MARKERS": REMOTE_PARENT / "marker-creation-v1",
    "U3-SINGLE": REMOTE_PARENT / "u3-single-v1",
    "T3-SINGLE": REMOTE_PARENT / "t3-single-v1",
}
ROUTE_ORDER = ("U3-SINGLE", "T3-SINGLE")
EXTERNAL_ACTIONS = ("self-check", "bootstrap", "create-markers", "run-u3", "run-t3")
ALLOWED_TERMINALS = {
    "BLOCKED_PROVENANCE",
    "BLOCKED_THERMAL",
    "BLOCKED_SEMANTIC",
    "BLOCKED_CAPABILITY",
    "BLOCKED_BUCKET_MAP",
    "BLOCKED_PERTURBATION",
    "BLOCKED_SAMPLE_COVERAGE",
}


class LocalD4Error(RuntimeError):
    pass


def require(condition: bool, message: str) -> None:
    if not condition:
        raise LocalD4Error(message)


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
    require(path.is_file() and not path.is_symlink(), f"missing or invalid file: {path}")
    return {
        "path": str(path),
        "mode": mode_string(path),
        "size_bytes": path.stat().st_size,
        "sha256": sha256_file(path),
    }


def require_file(
    path: pathlib.Path,
    *,
    digest: str | None = None,
    size: int | None = None,
    mode: str | None = None,
) -> dict[str, Any]:
    value = file_identity(path)
    if digest is not None:
        require(value["sha256"] == digest, f"SHA drift: {path}")
    if size is not None:
        require(value["size_bytes"] == size, f"size drift: {path}")
    if mode is not None:
        require(value["mode"] == mode, f"mode drift: {path}")
    return value


def canonical_json_bytes(value: Any) -> bytes:
    return (json.dumps(value, ensure_ascii=False, indent=2, sort_keys=True) + "\n").encode()


def write_new_bytes(path: pathlib.Path, value: bytes, mode: int = 0o600) -> None:
    descriptor = os.open(path, os.O_WRONLY | os.O_CREAT | os.O_EXCL, mode)
    try:
        with os.fdopen(descriptor, "wb", closefd=False) as output:
            output.write(value)
            output.flush()
            os.fsync(output.fileno())
    finally:
        os.close(descriptor)
    os.chmod(path, mode)


def write_new_json(path: pathlib.Path, value: Any, mode: int = 0o600) -> None:
    write_new_bytes(path, canonical_json_bytes(value), mode)


def fsync_directory(path: pathlib.Path) -> None:
    descriptor = os.open(path, os.O_RDONLY | os.O_DIRECTORY)
    try:
        os.fsync(descriptor)
    finally:
        os.close(descriptor)


def seal_tree(root: pathlib.Path) -> None:
    for path in sorted(root.rglob("*"), reverse=True):
        path.chmod(0o555 if path.is_dir() else 0o444)
    root.chmod(0o555)


def write_sha256sums(root: pathlib.Path) -> None:
    lines = []
    for path in sorted(item for item in root.rglob("*") if item.is_file() and item.name != "SHA256SUMS"):
        lines.append(f"{sha256_file(path)}  {path.relative_to(root)}\n")
    write_new_bytes(root / "SHA256SUMS", "".join(lines).encode())


def verify_sha256sums(root: pathlib.Path) -> int:
    manifest = root / "SHA256SUMS"
    require(manifest.is_file(), f"SHA256SUMS missing: {root}")
    count = 0
    for line in manifest.read_text().splitlines():
        digest, relative = line.split("  ", 1)
        path = root / relative
        require(path.is_file() and sha256_file(path) == digest, f"manifest mismatch: {path}")
        count += 1
    actual = sum(1 for path in root.rglob("*") if path.is_file() and path.name != "SHA256SUMS")
    require(count == actual, f"manifest cardinality drift: {count} != {actual}")
    return count


def run(
    command: Sequence[str],
    *,
    input_bytes: bytes | None = None,
    timeout: int = 600,
    check: bool = True,
) -> subprocess.CompletedProcess[bytes]:
    result = subprocess.run(
        list(command),
        input=input_bytes,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        timeout=timeout,
        check=False,
    )
    if check and result.returncode != 0:
        raise LocalD4Error(
            f"command failed {result.returncode}: {shlex.join(command)}\n"
            f"{result.stderr.decode(errors='replace')[-8000:]}"
        )
    return result


def ssh_argv(command: Sequence[str]) -> list[str]:
    return [
        "/usr/bin/ssh",
        "-i",
        str(SSH_IDENTITY),
        "-o",
        "BatchMode=yes",
        "-o",
        "ConnectTimeout=8",
        REMOTE,
        shlex.join(command),
    ]


REMOTE_BOOTSTRAP = (
    "import base64,hashlib,json,sys\n"
    "envelope=json.loads(sys.stdin.buffer.read())\n"
    "source=base64.b64decode(envelope['remote_controller'],validate=True)\n"
    "payload=base64.b64decode(envelope['payload'],validate=True)\n"
    "assert hashlib.sha256(source).hexdigest()==sys.argv[1]\n"
    "assert hashlib.sha256(payload).hexdigest()==sys.argv[2]\n"
    "sys.argv=['lay-v10-e1-traversal-d4-estimator-recovery-remote.py',base64.b64encode(payload).decode()]\n"
    "ns={'__name__':'__main__','__file__':'<lay-v10-e1-traversal-d4-estimator-recovery-remote.py>'}\n"
    "exec(compile(source,ns['__file__'],'exec'),ns)\n"
)


def payload(action: str, route: str | None = None, *, include_audit: bool) -> bytes:
    local_source = CONTROLLER.read_bytes()
    remote_source = REMOTE_CONTROLLER.read_bytes()
    value: dict[str, Any] = {
        "action": action,
        "route": route,
        "paper_b64": base64.b64encode(PAPER.read_bytes()).decode(),
        "preflight_b64": base64.b64encode(PREFLIGHT.read_bytes()).decode(),
        "preflight_receipt_b64": base64.b64encode(PREFLIGHT_RECEIPT.read_bytes()).decode(),
        "d2_terminal_b64": base64.b64encode(D2_TERMINAL.read_bytes()).decode(),
        "d3_terminal_b64": base64.b64encode(D3_TERMINAL.read_bytes()).decode(),
        "local_controller_sha256": sha256_bytes(local_source),
        "local_controller_b64": base64.b64encode(local_source).decode(),
        "remote_controller_sha256": sha256_bytes(remote_source),
        "remote_controller_b64": base64.b64encode(remote_source).decode(),
    }
    if include_audit:
        audit = BOOTSTRAP_AUDIT_RECEIPT.read_bytes()
        value["bootstrap_audit_sha256"] = sha256_bytes(audit)
        value["bootstrap_audit_b64"] = base64.b64encode(audit).decode()
        if MARKER_AUDIT_RECEIPT.is_file():
            marker_audit = MARKER_AUDIT_RECEIPT.read_bytes()
            value["marker_audit_sha256"] = sha256_bytes(marker_audit)
            value["marker_audit_b64"] = base64.b64encode(marker_audit).decode()
    return canonical_json_bytes(value)


def remote_call(
    action: str,
    route: str | None = None,
    *,
    include_audit: bool,
    timeout: int,
) -> subprocess.CompletedProcess[bytes]:
    remote_source = REMOTE_CONTROLLER.read_bytes()
    request = payload(action, route, include_audit=include_audit)
    envelope = canonical_json_bytes(
        {
            "remote_controller": base64.b64encode(remote_source).decode(),
            "payload": base64.b64encode(request).decode(),
        }
    )
    command = [
        "/usr/bin/sudo",
        "-n",
        "/usr/bin/python3",
        "-c",
        REMOTE_BOOTSTRAP,
        sha256_bytes(remote_source),
        sha256_bytes(request),
    ]
    return run(ssh_argv(command), input_bytes=envelope, timeout=timeout, check=False)


def parse_last_json(result: subprocess.CompletedProcess[bytes], action: str) -> dict[str, Any]:
    require(
        result.returncode == 0,
        f"remote {action} failed ({result.returncode}):\n{result.stderr.decode(errors='replace')[-8000:]}",
    )
    lines = result.stdout.decode(errors="replace").strip().splitlines()
    require(bool(lines), f"remote {action} produced no output")
    return json.loads(lines[-1])


def local_runtime_snapshot() -> dict[str, Any]:
    launcher = pathlib.Path.home() / ".local/bin/lay"
    resolved = launcher.resolve(strict=True)
    return {
        "launcher": str(launcher),
        "resolved": str(resolved),
        "resolved_sha256": sha256_file(resolved),
    }


def verify_local_inputs() -> dict[str, Any]:
    preflight = require_file(PREFLIGHT, digest=PREFLIGHT_SHA256, size=20_020, mode="0444")
    preflight_value = json.loads(PREFLIGHT.read_text())
    require(
        preflight_value.get("task_id")
        == "LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_D4_ESTIMATOR_RECOVERY_IMPLEMENTATION_V2_2026-08-26",
        "preflight task drift",
    )
    receipt = require_file(
        PREFLIGHT_RECEIPT,
        digest=PREFLIGHT_RECEIPT_SHA256,
        size=9_396,
        mode="0444",
    )
    receipt_value = json.loads(PREFLIGHT_RECEIPT.read_text())
    require(
        receipt_value.get("verdict") == "READY_TO_IMPLEMENT"
        and receipt_value.get("safe_to_implement") is True,
        "preflight receipt drift",
    )
    terminal = require_file(D2_TERMINAL, digest=D2_TERMINAL_SHA256, size=14_907, mode="0444")
    terminal_value = json.loads(D2_TERMINAL.read_text())
    require(terminal_value.get("verdict") == "BLOCKED_PROVENANCE", "D2 terminal verdict drift")
    d3_terminal = require_file(D3_TERMINAL, digest=D3_TERMINAL_SHA256, size=3_706, mode="0444")
    d3_terminal_value = json.loads(D3_TERMINAL.read_text())
    require(d3_terminal_value.get("verdict") == "BLOCKED_PROVENANCE", "D3 terminal verdict drift")
    return {
        "paper": require_file(PAPER, digest=PAPER_SHA256, size=9_567, mode="0444"),
        "preflight": preflight,
        "preflight_receipt": receipt,
        "d2_terminal": terminal,
        "d3_terminal": d3_terminal,
        "elf": require_file(LOCAL_ELF, digest=ELF_SHA256, size=317_706_232, mode="0555"),
        "map": require_file(LOCAL_MAP, digest=MAP_SHA256, size=390_324, mode="0444"),
        "ssh_identity": require_file(SSH_IDENTITY, mode="0600"),
        "controller": file_identity(CONTROLLER),
        "remote_controller": file_identity(REMOTE_CONTROLLER),
        "bootstrap_auditor": file_identity(BOOTSTRAP_AUDITOR),
        "marker_auditor": file_identity(MARKER_AUDITOR),
    }


def function_node(tree: ast.Module, name: str) -> ast.FunctionDef:
    values = [node for node in tree.body if isinstance(node, ast.FunctionDef) and node.name == name]
    require(len(values) == 1, f"function cardinality drift: {name}")
    return values[0]


def verify_command_graph() -> dict[str, Any]:
    local_source = CONTROLLER.read_text()
    remote_source = REMOTE_CONTROLLER.read_text()
    auditor_source = BOOTSTRAP_AUDITOR.read_text()
    marker_auditor_source = MARKER_AUDITOR.read_text()
    compile(local_source, str(CONTROLLER), "exec")
    compile(remote_source, str(REMOTE_CONTROLLER), "exec")
    compile(auditor_source, str(BOOTSTRAP_AUDITOR), "exec")
    compile(marker_auditor_source, str(MARKER_AUDITOR), "exec")
    namespace: dict[str, Any] = {
        "__name__": "d4_remote_static_registry",
        "__file__": str(REMOTE_CONTROLLER),
    }
    exec(compile(remote_source, str(REMOTE_CONTROLLER), "exec"), namespace)
    auditor_namespace: dict[str, Any] = {
        "__name__": "d4_bootstrap_auditor_static_registry",
        "__file__": str(BOOTSTRAP_AUDITOR),
    }
    exec(compile(auditor_source, str(BOOTSTRAP_AUDITOR), "exec"), auditor_namespace)
    marker_auditor_namespace: dict[str, Any] = {
        "__name__": "d4_marker_auditor_static_registry",
        "__file__": str(MARKER_AUDITOR),
    }
    exec(compile(marker_auditor_source, str(MARKER_AUDITOR), "exec"), marker_auditor_namespace)
    require(tuple(namespace["ROUTE_ORDER"]) == ROUTE_ORDER, "remote route registry drift")
    require(set(namespace["ROUTES"]) == set(ROUTE_ORDER), "remote route membership drift")
    require(namespace["U3_EDGES"] == 502_915_120, "U3 denominator drift")
    require(namespace["T3_EDGES"] == 528_060_876, "T3 denominator drift")
    require(
        namespace["subject_command"]()
        == [
            "/usr/bin/taskset",
            "--cpu-list",
            "6",
            "/usr/lib/x86_64-linux-gnu/ld-linux-x86-64.so.2",
            str(namespace["ELF"]),
            "--exact",
            "nanda_wave::l2_field::v13_typed_peak::tests::v10_d1_component_single",
            "--ignored",
            "--nocapture",
            "--test-threads=1",
        ],
        "subject argv drift",
    )
    require(
        tuple(namespace["PERF_RECORD_PREFIX"])
        == (
            "/usr/bin/sudo",
            "-n",
            "/usr/bin/perf",
            "record",
            "--buildid-all",
            "--sample-cpu",
            "--timestamp",
            "--event",
            "task-clock:u",
            "--count",
            "200000",
        ),
        "perf record argv drift",
    )
    require(
        tuple(namespace["U3_DISPATCH"])
        == (
            ("provenance", "BLOCKED_PROVENANCE"),
            ("thermal", "BLOCKED_THERMAL"),
            ("semantic", "BLOCKED_SEMANTIC"),
        ),
        "U3 dispatch drift",
    )
    require(
        tuple(namespace["T3_DISPATCH"])
        == (
            ("provenance", "BLOCKED_PROVENANCE"),
            ("thermal", "BLOCKED_THERMAL"),
            ("capability", "BLOCKED_CAPABILITY"),
            ("bucket_map", "BLOCKED_BUCKET_MAP"),
            ("perturbation", "BLOCKED_PERTURBATION"),
            ("sample_coverage", "BLOCKED_SAMPLE_COVERAGE"),
        ),
        "T3 dispatch drift",
    )
    tree = ast.parse(remote_source, filename=str(REMOTE_CONTROLLER))
    u3_node = function_node(tree, "run_u3")
    t3_node = function_node(tree, "run_t3")
    route_once_node = function_node(tree, "route_once")
    require(
        not any(isinstance(node, ast.Name) and node.id == "PERF_RECORD_PREFIX" for node in ast.walk(u3_node)),
        "perf record reachable from U3",
    )
    require(
        any(isinstance(node, ast.Name) and node.id == "PERF_RECORD_PREFIX" for node in ast.walk(t3_node)),
        "perf record not owned by T3",
    )
    stage_modes = []
    for node in ast.walk(route_once_node):
        if not isinstance(node, ast.Call) or not isinstance(node.func, ast.Attribute):
            continue
        if node.func.attr != "mkdir" or not isinstance(node.func.value, ast.Name) or node.func.value.id != "stage":
            continue
        modes = [keyword.value for keyword in node.keywords if keyword.arg == "mode"]
        require(len(modes) == 1, "route stage mkdir mode missing")
        stage_modes.append(ast.literal_eval(modes[0]))
    require(stage_modes == [0o755], f"route stage traversal mode drift: {stage_modes}")
    calls = []
    for node in ast.walk(tree):
        if isinstance(node, ast.Call) and isinstance(node.func, ast.Attribute):
            calls.append(node.func.attr)
    require("kill" not in calls and "killpg" not in calls, "signal lifecycle reachable")
    readers = namespace["reader_commands"](pathlib.Path("/tmp/perf.data"))
    require(set(readers) == {"evlist", "samples", "raw-records", "buildids"}, "reader graph drift")
    for command in readers.values():
        require(command[:2] != ["/usr/bin/perf", "stat"], "perf stat reachable")
    fake = {"violations": {name: [] for name, _ in namespace["T3_DISPATCH"]}}
    for cause, terminal in namespace["T3_DISPATCH"]:
        sample = json.loads(json.dumps(fake))
        sample["violations"][cause] = [cause]
        require(namespace["dispatch_observation"](sample, "T3-SINGLE")["verdict"] == terminal, f"dispatch mismatch: {cause}")
    require(
        tuple(auditor_namespace["EXTERNAL_ACTIONS"]) == ("self-check", "audit"),
        "bootstrap auditor action graph drift",
    )
    probe_source = auditor_namespace["projection_source"]()
    probe_tree = ast.parse(probe_source, filename="<d4-bootstrap-audit-remote-projection>")
    mutating_attributes = {
        "chmod",
        "mkdir",
        "rename",
        "replace",
        "rmdir",
        "touch",
        "truncate",
        "unlink",
        "write_bytes",
        "write_text",
    }
    for node in ast.walk(probe_tree):
        if not isinstance(node, ast.Call):
            continue
        if isinstance(node.func, ast.Attribute):
            require(node.func.attr not in mutating_attributes, f"auditor remote mutation reachable: {node.func.attr}")
            if node.func.attr == "open":
                mode = "r" if not node.args else ast.literal_eval(node.args[0])
                require(mode in ("r", "rb"), f"auditor remote writable open reachable: {mode}")
        elif isinstance(node.func, ast.Name):
            require(
                node.func.id not in {"exec", "eval", "open"},
                f"auditor remote dynamic or writable call reachable: {node.func.id}",
            )
    remote_tree = ast.parse(remote_source, filename=str(REMOTE_CONTROLLER))
    bootstrap_node = function_node(remote_tree, "bootstrap")
    marker_node = function_node(remote_tree, "create_markers")
    require(
        not any(isinstance(node, ast.Name) and node.id == "marker_body" for node in ast.walk(bootstrap_node)),
        "scientific marker creation reachable from bootstrap",
    )
    require(
        any(isinstance(node, ast.Name) and node.id == "marker_body" for node in ast.walk(marker_node)),
        "marker serializer absent from post-audit marker action",
    )
    require(tuple(marker_auditor_namespace["EXTERNAL_ACTIONS"]) == ("self-check", "audit"), "marker auditor action graph drift")
    return {
        "route_registry": list(ROUTE_ORDER),
        "subject_command": namespace["subject_command"](),
        "route_stage_mode": "0755",
        "perf_record_reachable_only_from": ["T3-SINGLE"],
        "perf_stat_routes": [],
        "pid_attach": False,
        "sigint_lifecycle": False,
        "fixed_or_reversed_routes": [],
        "u3_denominator_edges": namespace["U3_EDGES"],
        "t3_denominator_edges": namespace["T3_EDGES"],
        "u3_dispatch": list(namespace["U3_DISPATCH"]),
        "t3_dispatch": list(namespace["T3_DISPATCH"]),
        "auditor_actions": list(auditor_namespace["EXTERNAL_ACTIONS"]),
        "marker_auditor_actions": list(marker_auditor_namespace["EXTERNAL_ACTIONS"]),
        "auditor_remote_mutation_calls": 0,
        "bootstrap_scientific_marker_calls": 0,
    }


def initial_self_check() -> dict[str, Any]:
    inputs = verify_local_inputs()
    graph = verify_command_graph()
    remote = parse_last_json(
        remote_call("probe-absent", include_audit=False, timeout=1200),
        "probe-absent",
    )
    require(remote.get("verdict") == "D4_REMOTE_ABSENT_PROBE_PASS", "remote absent probe drift")
    require(remote.get("host", {}).get("hostname") == REMOTE_HOSTNAME, "remote host drift")
    require(remote.get("remote_writes") == 0, "remote absent probe wrote state")
    return {
        "schema": "lay.v10.e1-traversal-d4-controller-self-check.v1",
        "task_id": TASK_ID,
        "transaction_id": TRANSACTION_ID,
        "verdict": "D4_CONTROLLER_VERIFIED_UNRUN",
        "inputs": inputs,
        "command_graph": graph,
        "remote_probe_sha256": sha256_bytes(canonical_json_bytes(remote)),
        "remote_writes": 0,
        "marker_mutations": 0,
        "subject_executions": 0,
        "perf_record": 0,
        "perf_stat": 0,
        "pmu_events_opened": 0,
    }


def copy_remote_tree(remote_path: pathlib.PurePosixPath, destination: pathlib.Path) -> None:
    result = run(
        [
            "/usr/bin/scp",
            "-i",
            str(SSH_IDENTITY),
            "-o",
            "BatchMode=yes",
            "-o",
            "ConnectTimeout=8",
            "-q",
            "-p",
            "-r",
            f"{REMOTE}:{remote_path}",
            str(destination),
        ],
        timeout=3600,
        check=False,
    )
    require(result.returncode == 0, result.stderr.decode(errors="replace")[-8000:])


def publish_local_result(
    destination: pathlib.Path,
    remote_path: pathlib.PurePosixPath,
    receipt_name: str,
    local_receipt: Mapping[str, Any],
    extra_files: Mapping[str, bytes],
) -> dict[str, Any]:
    require(not destination.exists(), f"local result exists: {destination}")
    stage = pathlib.Path(f"{destination}.stage-{os.getpid()}-{time.time_ns()}")
    stage.mkdir(parents=True, mode=0o700)
    try:
        remote_evidence = stage / "REMOTE_EVIDENCE"
        copy_remote_tree(remote_path, remote_evidence)
        manifest_entries = verify_sha256sums(remote_evidence)
        remote_receipt = remote_evidence / receipt_name
        require(remote_receipt.is_file(), f"remote receipt missing: {receipt_name}")
        for name, value in extra_files.items():
            write_new_bytes(stage / name, value)
        write_new_json(stage / "LOCAL_RECEIPT.json", dict(local_receipt))
        write_sha256sums(stage)
        seal_tree(stage)
        os.rename(stage, destination)
        fsync_directory(destination.parent)
        return {
            "path": str(destination),
            "remote_manifest_entries": manifest_entries,
            "remote_receipt_sha256": sha256_file(destination / f"REMOTE_EVIDENCE/{receipt_name}"),
        }
    except BaseException:
        if stage.exists():
            shutil.rmtree(stage)
        raise


def bootstrap_once() -> dict[str, Any]:
    require(not LOCAL_BOOTSTRAP.exists(), "local bootstrap result already exists")
    check = initial_self_check()
    runtime_before = local_runtime_snapshot()
    remote = parse_last_json(
        remote_call("bootstrap", include_audit=False, timeout=1800),
        "bootstrap",
    )
    require(remote.get("verdict") == "D4_UID_PROOF_CREATED_UNAUDITED", "remote bootstrap verdict drift")
    require(remote.get("markers_created") == 0 and remote.get("markers_consumed") == 0, "bootstrap marker ledger drift")
    require(remote.get("uid_probe", {}).get("value", {}).get("verdict") == "D4_UID_E_CAPABILITY_PROOF_PASS", "UID probe verdict drift")
    require(remote.get("perf_record") == 0 and remote.get("subject_executions") == 0, "bootstrap execution ledger drift")
    runtime_after = local_runtime_snapshot()
    require(runtime_before == runtime_after, "installed runtime changed during bootstrap")
    local_receipt = {
        "schema": "lay.v10.e1-traversal-d4-local-bootstrap.v1",
        "task_id": TASK_ID,
        "transaction_id": TRANSACTION_ID,
        "verdict": "D4_UID_PROOF_CREATED_UNAUDITED",
        "remote_receipt_sha256": remote["published_receipt_sha256"],
        "local_controller_sha256": sha256_file(CONTROLLER),
        "remote_controller_sha256": sha256_file(REMOTE_CONTROLLER),
        "bootstrap_auditor_sha256": sha256_file(BOOTSTRAP_AUDITOR),
        "marker_auditor_sha256": sha256_file(MARKER_AUDITOR),
        "markers_created": 0,
        "markers_consumed": 0,
        "cargo_invocations": 0,
        "perf_record": 0,
        "perf_stat": 0,
        "pmu_events_opened": 0,
        "subject_executions": 0,
        "runtime_before": runtime_before,
        "runtime_after": runtime_after,
        "runtime_authority_changed": False,
        "next_action_admitted": "independent D4 bootstrap audit only",
    }
    published = publish_local_result(
        LOCAL_BOOTSTRAP,
        REMOTE_RESULTS["BOOTSTRAP"],
        "D4_BOOTSTRAP_RECEIPT.json",
        local_receipt,
        {
            "SELF_CHECK.json": canonical_json_bytes(check),
            "REMOTE_RESPONSE.json": canonical_json_bytes(remote),
            "local-controller.py": CONTROLLER.read_bytes(),
            "remote-controller.py": REMOTE_CONTROLLER.read_bytes(),
            "bootstrap-auditor.py": BOOTSTRAP_AUDITOR.read_bytes(),
            "marker-auditor.py": MARKER_AUDITOR.read_bytes(),
            "preflight-v2.json": PREFLIGHT.read_bytes(),
            "preflight-v2-receipt.json": PREFLIGHT_RECEIPT.read_bytes(),
        },
    )
    return {**local_receipt, **published}


def verify_bootstrap_audit() -> dict[str, Any]:
    value = require_file(BOOTSTRAP_AUDIT_RECEIPT, mode="0444")
    body = json.loads(BOOTSTRAP_AUDIT_RECEIPT.read_text())
    require(body.get("verdict") == "D4_UID_ACCESS_AUDIT_PASS_MARKER_CREATION", "bootstrap audit verdict drift")
    require(body.get("task_id") == TASK_ID and body.get("transaction_id") == TRANSACTION_ID, "bootstrap audit namespace drift")
    require(body.get("local_controller_sha256") == sha256_file(CONTROLLER), "audited local controller drift")
    require(body.get("remote_controller_sha256") == sha256_file(REMOTE_CONTROLLER), "audited remote controller drift")
    require(
        body.get("markers_created") == 0
        and body.get("markers_consumed") == 0
        and body.get("remote_writes") == 0
        and body.get("scp_as_e_pass") is True,
        "bootstrap audit ledger drift",
    )
    return {**value, "value": body}


def create_markers_once() -> dict[str, Any]:
    require(not LOCAL_MARKER_CREATION.exists(), "local marker creation result already exists")
    inputs = verify_local_inputs()
    graph = verify_command_graph()
    audit = verify_bootstrap_audit()
    probe = parse_last_json(
        remote_call("probe-bootstrap", include_audit=False, timeout=1800),
        "probe-bootstrap",
    )
    require(probe.get("verdict") == "D4_REMOTE_BOOTSTRAP_PROBE_PASS", "bootstrap probe drift")
    runtime_before = local_runtime_snapshot()
    remote = parse_last_json(
        remote_call("create-markers", include_audit=True, timeout=1800),
        "create-markers",
    )
    require(remote.get("verdict") == "D4_MARKERS_CREATED_UNAUDITED", "marker creation verdict drift")
    require(remote.get("markers_created") == 2 and remote.get("markers_consumed") == 0, "marker creation ledger drift")
    runtime_after = local_runtime_snapshot()
    require(runtime_before == runtime_after, "installed runtime changed during marker creation")
    local_receipt = {
        "schema": "lay.v10.e1-traversal-d4-local-marker-creation.v1",
        "task_id": TASK_ID,
        "transaction_id": TRANSACTION_ID,
        "verdict": "D4_MARKERS_CREATED_UNAUDITED",
        "remote_receipt_sha256": remote["published_receipt_sha256"],
        "bootstrap_audit_sha256": audit["sha256"],
        "inputs": inputs,
        "command_graph": graph,
        "markers_created": 2,
        "markers_consumed": 0,
        "subject_executions": 0,
        "perf_record": 0,
        "perf_stat": 0,
        "pmu_events_opened": 0,
        "runtime_before": runtime_before,
        "runtime_after": runtime_after,
        "runtime_authority_changed": False,
        "retry_permitted": False,
        "next_action_admitted": "independent D4 marker audit only",
    }
    published = publish_local_result(
        LOCAL_MARKER_CREATION,
        REMOTE_RESULTS["MARKERS"],
        "D4_MARKER_CREATION_RECEIPT.json",
        local_receipt,
        {
            "REMOTE_BOOTSTRAP_PROBE.json": canonical_json_bytes(probe),
            "REMOTE_RESPONSE.json": canonical_json_bytes(remote),
            "bootstrap-audit.json": BOOTSTRAP_AUDIT_RECEIPT.read_bytes(),
            "local-controller.py": CONTROLLER.read_bytes(),
            "remote-controller.py": REMOTE_CONTROLLER.read_bytes(),
        },
    )
    return {**local_receipt, **published}


def verify_marker_audit() -> dict[str, Any]:
    value = require_file(MARKER_AUDIT_RECEIPT, mode="0444")
    body = json.loads(MARKER_AUDIT_RECEIPT.read_text())
    require(body.get("verdict") == "D4_MARKER_AUDIT_PASS_U3_ADMITTED", "marker audit verdict drift")
    require(body.get("task_id") == TASK_ID and body.get("transaction_id") == TRANSACTION_ID, "marker audit namespace drift")
    require(body.get("markers_created") == 2 and body.get("markers_consumed") == 0, "marker audit ledger drift")
    require(body.get("remote_writes") == 0, "marker audit remote write drift")
    return {**value, "value": body}


def route_self_check(route: str) -> dict[str, Any]:
    require(route in ROUTE_ORDER, f"unknown route: {route}")
    inputs = verify_local_inputs()
    graph = verify_command_graph()
    audit = verify_bootstrap_audit()
    marker_audit = verify_marker_audit()
    require(not LOCAL_RESULTS[route].exists(), f"local route result already exists: {route}")
    if route == "T3-SINGLE":
        u3_receipt = LOCAL_RESULTS["U3-SINGLE"] / "REMOTE_EVIDENCE/D4_ROUTE_RECEIPT.json"
        require(u3_receipt.is_file(), "local U3 receipt missing")
        require(json.loads(u3_receipt.read_text()).get("verdict") == "U3_SINGLE_PASS", "local U3 did not PASS")
    remote = parse_last_json(
        remote_call("probe-ready", route, include_audit=True, timeout=1800),
        "probe-ready",
    )
    require(remote.get("verdict") == "D4_REMOTE_READY_PROBE_PASS", "remote ready probe drift")
    require(remote.get("route") == route and remote.get("remote_writes") == 0, "remote route probe drift")
    return {
        "schema": "lay.v10.e1-traversal-d4-route-self-check.v1",
        "task_id": TASK_ID,
        "transaction_id": TRANSACTION_ID,
        "route": route,
        "verdict": "D4_ROUTE_CONTROLLER_VERIFIED_UNRUN",
        "inputs": inputs,
        "command_graph": graph,
        "bootstrap_audit": audit,
        "marker_audit": marker_audit,
        "remote_probe_sha256": sha256_bytes(canonical_json_bytes(remote)),
        "remote_writes": 0,
        "marker_mutations": 0,
        "subject_executions": 0,
        "perf_record": 0,
        "perf_stat": 0,
    }


def run_route_once(route: str) -> dict[str, Any]:
    check = route_self_check(route)
    runtime_before = local_runtime_snapshot()
    action = "run-u3" if route == "U3-SINGLE" else "run-t3"
    remote = parse_last_json(
        remote_call(action, route, include_audit=True, timeout=7200),
        action,
    )
    require(remote.get("route") == route, "remote route receipt drift")
    require(remote.get("retry_permitted") is False, "remote retry authority drift")
    allowed = ALLOWED_TERMINALS | ({"U3_SINGLE_PASS"} if route == "U3-SINGLE" else {"D4_SINGLE_ESTIMATOR_PASS"})
    require(remote.get("verdict") in allowed, "remote route verdict invalid")
    post = parse_last_json(
        remote_call("probe-after", route, include_audit=True, timeout=1800),
        "probe-after",
    )
    require(post.get("verdict") == "D4_REMOTE_POST_PROBE_PASS", "remote post probe drift")
    require(post.get("route_verdict") == remote.get("verdict"), "remote post verdict drift")
    runtime_after = local_runtime_snapshot()
    require(runtime_before == runtime_after, "installed runtime changed during route")
    local_receipt = {
        "schema": "lay.v10.e1-traversal-d4-local-route.v1",
        "task_id": TASK_ID,
        "transaction_id": TRANSACTION_ID,
        "route": route,
        "verdict": remote["verdict"],
        "dispatch": remote.get("dispatch"),
        "remote_receipt_sha256": remote["published_receipt_sha256"],
        "marker_consumed": True,
        "cargo_invocations": 0,
        "perf_record": remote.get("perf_record"),
        "perf_readers": remote.get("perf_readers"),
        "perf_stat": 0,
        "pmu_events_opened": remote.get("pmu_events_opened"),
        "subject_executions": 1,
        "d2_marker_mutations": 0,
        "runtime_before": runtime_before,
        "runtime_after": runtime_after,
        "runtime_authority_changed": False,
        "retry_permitted": False,
        "next_action_admitted": remote.get("next_action_admitted"),
    }
    published = publish_local_result(
        LOCAL_RESULTS[route],
        REMOTE_RESULTS[route],
        "D4_ROUTE_RECEIPT.json",
        local_receipt,
        {
            "SELF_CHECK.json": canonical_json_bytes(check),
            "REMOTE_RESPONSE.json": canonical_json_bytes(remote),
            "REMOTE_POST_PROBE.json": canonical_json_bytes(post),
            "local-controller.py": CONTROLLER.read_bytes(),
            "remote-controller.py": REMOTE_CONTROLLER.read_bytes(),
            "bootstrap-audit.json": BOOTSTRAP_AUDIT_RECEIPT.read_bytes(),
            "marker-audit.json": MARKER_AUDIT_RECEIPT.read_bytes(),
            "preflight-v2.json": PREFLIGHT.read_bytes(),
            "preflight-v2-receipt.json": PREFLIGHT_RECEIPT.read_bytes(),
        },
    )
    copied = json.loads(
        (LOCAL_RESULTS[route] / "REMOTE_EVIDENCE/D4_ROUTE_RECEIPT.json").read_text()
    )
    require(copied.get("verdict") == remote.get("verdict"), "copied route verdict drift")
    require(published["remote_receipt_sha256"] == remote["published_receipt_sha256"], "copied route receipt SHA drift")
    return {**local_receipt, **published}


def parser() -> argparse.ArgumentParser:
    value = argparse.ArgumentParser()
    value.add_argument("action", choices=EXTERNAL_ACTIONS)
    return value


def main() -> int:
    arguments = parser().parse_args()
    try:
        if arguments.action == "self-check":
            value = initial_self_check()
        elif arguments.action == "bootstrap":
            value = bootstrap_once()
        elif arguments.action == "create-markers":
            value = create_markers_once()
        elif arguments.action == "run-u3":
            value = run_route_once("U3-SINGLE")
        else:
            value = run_route_once("T3-SINGLE")
        print(json.dumps(value, ensure_ascii=False, sort_keys=True))
        return 0
    except Exception as error:
        print(f"D4 LOCAL ERROR: {type(error).__name__}: {error}", file=sys.stderr)
        return 1


if __name__ == "__main__":
    raise SystemExit(main())
