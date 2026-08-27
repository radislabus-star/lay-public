#!/usr/bin/env python3
"""Local controller for D5 multiworker TID attribution."""

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
TASK_ID = "slice8b-v10-e1-traversal-d5-multiworker-tid-estimator-v1-20260826"
TRANSACTION_ID = "3ee46e2c915677e1b2d3cd6bcc9709e0232252dbc120745b097d736537779036"
REMOTE = "e@192.168.3.94"
REMOTE_HOSTNAME = "e-MEGA-MINI-M1-13th"
SSH_IDENTITY = pathlib.Path("/home/ubu/.ssh/mega-mini-admin")

CONTROLLER = PROJECT_ROOT / "scripts/lay-v10-e1-traversal-d5-multiworker-tid.py"
REMOTE_CONTROLLER = PROJECT_ROOT / "scripts/lay-v10-e1-traversal-d5-multiworker-tid-remote.py"
BOOTSTRAP_AUDITOR = PROJECT_ROOT / "scripts/lay-v10-e1-traversal-d5-bootstrap-audit.py"
MARKER_AUDITOR = PROJECT_ROOT / "scripts/lay-v10-e1-traversal-d5-marker-audit.py"
TERMINAL_AUDITOR = PROJECT_ROOT / "scripts/lay-v10-e1-traversal-d5-terminal-audit.py"
PAPER = PROJECT_ROOT / (
    "docs/structural_gates/"
    "LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_D5_MULTIWORKER_TID_ESTIMATOR_V1_2026-08-26.md"
)
PREFLIGHT = PROJECT_ROOT / (
    "docs/structural_gates/preflights/"
    "LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_D5_MULTIWORKER_TID_IMPLEMENTATION_V3_2026-08-26.json"
)
PREFLIGHT_RECEIPT = PROJECT_ROOT / (
    "docs/structural_gates/receipts/"
    "LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_D5_MULTIWORKER_TID_IMPLEMENTATION_V3_PREFLIGHT_2026-08-26.json"
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
D4_TERMINAL = PROJECT_ROOT / (
    "docs/structural_gates/receipts/"
    "LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_D4_TERMINAL_AUDIT_V1_2026-08-26/"
    "D4_TERMINAL_AUDIT_RECEIPT.json"
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
    "LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_D5_BOOTSTRAP_AUDIT_V1_2026-08-26/"
    "D5_BOOTSTRAP_AUDIT_RECEIPT.json"
)
MARKER_AUDIT_RECEIPT = PROJECT_ROOT / (
    "docs/structural_gates/receipts/"
    "LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_D5_MARKER_AUDIT_V1_2026-08-26/"
    "D5_MARKER_AUDIT_RECEIPT.json"
)

PAPER_SHA256 = "cfe512a162d257ecf5b3c37c2dbb1c39ecacf831b70954f46912035d00169c87"
PREFLIGHT_SHA256 = "4c51d24eaa8cdd1052f5220d453fc3ad5d8e5d0b43fda7c972d3b0c8063ef77a"
PREFLIGHT_RECEIPT_SHA256 = "78cde27fbf1062122fb74e3c0f5028c29784e9d16417cad7ae0638864ef6455b"
D2_TERMINAL_SHA256 = "75dc2703e279524a76c49a455b6081634d1c67274106da45f0d7d610af95e608"
D3_TERMINAL_SHA256 = "7f4c0fd9466cced361c21858ea2017910adc7b92db7c1edfe02f3be09d5c4299"
D4_TERMINAL_SHA256 = "f748a7c448f7f142a342efa2b916a8995969044567a3aaa9a7de3aff6ba8645b"
ELF_SHA256 = "bb7c68528ec3fec7074919c3c64cce233d48c26efefe9d9b32735c5f5865a178"
MAP_SHA256 = "2b93aedf11101a8bc5fc7ed4ffaa056b50476d96cdd2eabf9d185f59ee60f846"

REMOTE_PARENT = pathlib.PurePosixPath("/home/e/.local/share/lay/provenance") / TASK_ID
LOCAL_BOOTSTRAP = PROJECT_ROOT / (
    "docs/structural_gates/receipts/"
    "LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_D5_BOOTSTRAP_V1_2026-08-26"
)
LOCAL_MARKER_CREATION = PROJECT_ROOT / (
    "docs/structural_gates/receipts/"
    "LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_D5_MARKER_CREATION_V1_2026-08-26"
)
LOCAL_RESULTS = {
    "U4-FIXED": PROJECT_ROOT / (
        "docs/structural_gates/receipts/"
        "LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_D5_U4_FIXED_V1_2026-08-26"
    ),
    "T4-FIXED": PROJECT_ROOT / (
        "docs/structural_gates/receipts/"
        "LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_D5_T4_FIXED_V1_2026-08-26"
    ),
    "U4-REVERSED": PROJECT_ROOT / (
        "docs/structural_gates/receipts/"
        "LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_D5_U4_REVERSED_V1_2026-08-26"
    ),
    "T4-REVERSED": PROJECT_ROOT / (
        "docs/structural_gates/receipts/"
        "LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_D5_T4_REVERSED_V1_2026-08-26"
    ),
}
REMOTE_RESULTS = {
    "BOOTSTRAP": REMOTE_PARENT / "bootstrap-v1",
    "MARKERS": REMOTE_PARENT / "marker-creation-v1",
    "U4-FIXED": REMOTE_PARENT / "u4-fixed-v1",
    "T4-FIXED": REMOTE_PARENT / "t4-fixed-v1",
    "U4-REVERSED": REMOTE_PARENT / "u4-reversed-v1",
    "T4-REVERSED": REMOTE_PARENT / "t4-reversed-v1",
}
ROUTE_ORDER = ("U4-FIXED", "T4-FIXED", "U4-REVERSED", "T4-REVERSED")
ACTION_TO_ROUTE = {
    "run-u4-fixed": "U4-FIXED",
    "run-t4-fixed": "T4-FIXED",
    "run-u4-reversed": "U4-REVERSED",
    "run-t4-reversed": "T4-REVERSED",
}
EXTERNAL_ACTIONS = ("self-check", "bootstrap", "create-markers", *ACTION_TO_ROUTE)
ALLOWED_TERMINALS = {
    "BLOCKED_PROVENANCE",
    "BLOCKED_THERMAL",
    "BLOCKED_SEMANTIC",
    "BLOCKED_CAPABILITY",
    "BLOCKED_BUCKET_MAP",
    "BLOCKED_PERTURBATION",
    "BLOCKED_SAMPLE_COVERAGE",
}


class LocalD5Error(RuntimeError):
    pass


def require(condition: bool, message: str) -> None:
    if not condition:
        raise LocalD5Error(message)


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
        raise LocalD5Error(
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
    "sys.argv=['lay-v10-e1-traversal-d5-multiworker-tid-remote.py',base64.b64encode(payload).decode()]\n"
    "ns={'__name__':'__main__','__file__':'<lay-v10-e1-traversal-d5-multiworker-tid-remote.py>'}\n"
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
        "d4_terminal_b64": base64.b64encode(D4_TERMINAL.read_bytes()).decode(),
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
    preflight = require_file(PREFLIGHT, digest=PREFLIGHT_SHA256, size=29_159, mode="0444")
    preflight_value = json.loads(PREFLIGHT.read_text())
    require(
        preflight_value.get("task_id")
        == "LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_D5_MULTIWORKER_TID_IMPLEMENTATION_V3_2026-08-26",
        "preflight task drift",
    )
    receipt = require_file(
        PREFLIGHT_RECEIPT,
        digest=PREFLIGHT_RECEIPT_SHA256,
        size=12_074,
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
    d4_terminal = require_file(D4_TERMINAL, digest=D4_TERMINAL_SHA256, size=3_685, mode="0444")
    d4_terminal_value = json.loads(D4_TERMINAL.read_text())
    require(d4_terminal_value.get("verdict") == "D4_SINGLE_ESTIMATOR_PASS", "D4 terminal verdict drift")
    return {
        "paper": require_file(PAPER, digest=PAPER_SHA256, size=9_319, mode="0444"),
        "preflight": preflight,
        "preflight_receipt": receipt,
        "d2_terminal": terminal,
        "d3_terminal": d3_terminal,
        "d4_terminal": d4_terminal,
        "elf": require_file(LOCAL_ELF, digest=ELF_SHA256, size=317_706_232, mode="0555"),
        "map": require_file(LOCAL_MAP, digest=MAP_SHA256, size=390_324, mode="0444"),
        "ssh_identity": require_file(SSH_IDENTITY, mode="0600"),
        "controller": file_identity(CONTROLLER),
        "remote_controller": file_identity(REMOTE_CONTROLLER),
        "bootstrap_auditor": file_identity(BOOTSTRAP_AUDITOR),
        "marker_auditor": file_identity(MARKER_AUDITOR),
        "terminal_auditor": file_identity(TERMINAL_AUDITOR),
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
    terminal_auditor_source = TERMINAL_AUDITOR.read_text()
    compile(local_source, str(CONTROLLER), "exec")
    compile(remote_source, str(REMOTE_CONTROLLER), "exec")
    compile(auditor_source, str(BOOTSTRAP_AUDITOR), "exec")
    compile(marker_auditor_source, str(MARKER_AUDITOR), "exec")
    compile(terminal_auditor_source, str(TERMINAL_AUDITOR), "exec")
    namespace: dict[str, Any] = {
        "__name__": "d5_remote_static_registry",
        "__file__": str(REMOTE_CONTROLLER),
    }
    exec(compile(remote_source, str(REMOTE_CONTROLLER), "exec"), namespace)
    auditor_namespace: dict[str, Any] = {
        "__name__": "d5_bootstrap_auditor_static_registry",
        "__file__": str(BOOTSTRAP_AUDITOR),
    }
    exec(compile(auditor_source, str(BOOTSTRAP_AUDITOR), "exec"), auditor_namespace)
    marker_auditor_namespace: dict[str, Any] = {
        "__name__": "d5_marker_auditor_static_registry",
        "__file__": str(MARKER_AUDITOR),
    }
    exec(compile(marker_auditor_source, str(MARKER_AUDITOR), "exec"), marker_auditor_namespace)
    terminal_auditor_namespace: dict[str, Any] = {
        "__name__": "d5_terminal_auditor_static_registry",
        "__file__": str(TERMINAL_AUDITOR),
    }
    exec(
        compile(terminal_auditor_source, str(TERMINAL_AUDITOR), "exec"),
        terminal_auditor_namespace,
    )
    require(tuple(namespace["ROUTE_ORDER"]) == ROUTE_ORDER, "remote route registry drift")
    require(set(namespace["ROUTES"]) == set(ROUTE_ORDER), "remote route membership drift")
    require(dict(namespace["ACTION_TO_ROUTE"]) == ACTION_TO_ROUTE, "remote action registry drift")
    require(tuple(EXTERNAL_ACTIONS) == ("self-check", "bootstrap", "create-markers", *ACTION_TO_ROUTE), "local action registry drift")
    require(namespace["U4_EDGES"] == 502_915_120, "U4 denominator drift")
    require(namespace["T4_EDGES"] == 528_060_876, "T4 denominator drift")
    expected_cpus = {
        "U4-FIXED": tuple(range(20)),
        "T4-FIXED": tuple(range(20)),
        "U4-REVERSED": tuple(reversed(range(20))),
        "T4-REVERSED": tuple(reversed(range(20))),
    }
    for route in ROUTE_ORDER:
        spec = namespace["ROUTES"][route]
        require(tuple(spec["cpus"]) == expected_cpus[route], f"route CPU mapping drift: {route}")
        marker = namespace["canonical_json_bytes"](namespace["marker_body"](route))
        require(len(marker) == spec["marker_size"], f"marker size drift: {route}")
        require(sha256_bytes(marker) == spec["marker_sha256"], f"marker SHA drift: {route}")
        environment = namespace["controlled_environment"](pathlib.Path("/tmp/d5-subject"), route)
        require(
            environment["LAY_V10_D1_CPUS"] == ",".join(str(cpu) for cpu in expected_cpus[route]),
            f"route environment mapping drift: {route}",
        )
    require(
        namespace["subject_command"]("U4-FIXED")
        == [
            "/usr/bin/taskset",
            "--cpu-list",
            "0-19",
            "/usr/lib/x86_64-linux-gnu/ld-linux-x86-64.so.2",
            str(namespace["ELF"]),
            "--exact",
            "nanda_wave::l2_field::v13_typed_peak::tests::v10_d1_component_twenty",
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
        tuple(namespace["U4_DISPATCH"])
        == (
            ("provenance", "BLOCKED_PROVENANCE"),
            ("thermal", "BLOCKED_THERMAL"),
            ("semantic", "BLOCKED_SEMANTIC"),
        ),
        "U4 dispatch drift",
    )
    require(
        tuple(namespace["T4_DISPATCH"])
        == (
            ("provenance", "BLOCKED_PROVENANCE"),
            ("thermal", "BLOCKED_THERMAL"),
            ("capability", "BLOCKED_CAPABILITY"),
            ("bucket_map", "BLOCKED_BUCKET_MAP"),
            ("perturbation", "BLOCKED_PERTURBATION"),
            ("sample_coverage", "BLOCKED_SAMPLE_COVERAGE"),
        ),
        "T4 dispatch drift",
    )
    tree = ast.parse(remote_source, filename=str(REMOTE_CONTROLLER))
    u4_node = function_node(tree, "run_u4")
    t4_node = function_node(tree, "run_t4")
    route_once_node = function_node(tree, "route_once")
    require(
        not any(isinstance(node, ast.Name) and node.id == "PERF_RECORD_PREFIX" for node in ast.walk(u4_node)),
        "perf record reachable from U4",
    )
    require(
        any(isinstance(node, ast.Name) and node.id == "PERF_RECORD_PREFIX" for node in ast.walk(t4_node)),
        "perf record not owned by T4",
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
    string_literals = []
    for node in ast.walk(tree):
        if isinstance(node, ast.Call) and isinstance(node.func, ast.Attribute):
            calls.append(node.func.attr)
        if isinstance(node, ast.Constant) and isinstance(node.value, str):
            string_literals.append(node.value)
    require("kill" not in calls and "killpg" not in calls, "signal lifecycle reachable")
    require("--pid" not in string_literals, "pid attach reachable")
    require("perf stat" not in "\n".join(string_literals), "perf stat reachable")
    readers = namespace["reader_commands"](pathlib.Path("/tmp/perf.data"))
    require(set(readers) == {"evlist", "samples", "raw-records", "buildids"}, "reader graph drift")
    for command in readers.values():
        require(command[:2] != ["/usr/bin/perf", "stat"], "perf stat reachable")
    fake = {
        "route": "T4-FIXED",
        "complete": True,
        "violations": {name: [] for name, _ in namespace["T4_DISPATCH"]},
    }
    for cause, terminal in namespace["T4_DISPATCH"]:
        sample = json.loads(json.dumps(fake))
        sample["violations"][cause] = [cause]
        require(namespace["dispatch_observation"](sample, "T4-FIXED")["verdict"] == terminal, f"T4 dispatch mismatch: {cause}")
    malformed_t = json.loads(json.dumps(fake))
    malformed_t["violations"]["unexpected"] = []
    require(
        namespace["dispatch_observation"](malformed_t, "T4-FIXED")["verdict"]
        == "BLOCKED_PROVENANCE",
        "T4 dispatch accepted an unknown predicate",
    )
    incomplete_t = json.loads(json.dumps(fake))
    incomplete_t["complete"] = False
    require(
        namespace["dispatch_observation"](incomplete_t, "T4-FIXED")["verdict"]
        == "BLOCKED_PROVENANCE",
        "T4 dispatch accepted an unexplained incomplete observation",
    )
    fake_u = {
        "route": "U4-FIXED",
        "complete": True,
        "violations": {name: [] for name, _ in namespace["U4_DISPATCH"]},
    }
    for cause, terminal in namespace["U4_DISPATCH"]:
        sample = json.loads(json.dumps(fake_u))
        sample["violations"][cause] = [cause]
        require(namespace["dispatch_observation"](sample, "U4-FIXED")["verdict"] == terminal, f"U4 dispatch mismatch: {cause}")
    malformed_u = json.loads(json.dumps(fake_u))
    malformed_u["route"] = "U4-REVERSED"
    require(
        namespace["dispatch_observation"](malformed_u, "U4-FIXED")["verdict"]
        == "BLOCKED_PROVENANCE",
        "U4 dispatch accepted a mismatched route",
    )
    for route in ("U4-FIXED", "T4-FIXED"):
        failure_observation = namespace["controller_failure_observation"](
            route, RuntimeError("injected route fault")
        )
        failure_dispatch = namespace["dispatch_observation"](failure_observation, route)
        require(
            failure_observation["complete"] is False
            and failure_observation["execution_ledger_complete"] is False
            and failure_dispatch["selected_cause"] == "provenance"
            and failure_dispatch["verdict"] == "BLOCKED_PROVENANCE",
            f"post-consumption controller failure dispatch drift: {route}",
        )
    require(
        tuple(auditor_namespace["EXTERNAL_ACTIONS"]) == ("self-check", "audit"),
        "bootstrap auditor action graph drift",
    )
    probe_source = auditor_namespace["projection_source"]()
    probe_tree = ast.parse(probe_source, filename="<d5-bootstrap-audit-remote-projection>")
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
    require(
        tuple(terminal_auditor_namespace["EXTERNAL_ACTIONS"])
        == ("self-check", "audit"),
        "terminal auditor action graph drift",
    )
    terminal_probe_tree = ast.parse(
        terminal_auditor_namespace["projection_source"](),
        filename="<d5-terminal-audit-remote-projection>",
    )
    for node in ast.walk(terminal_probe_tree):
        if not isinstance(node, ast.Call):
            continue
        if isinstance(node.func, ast.Attribute):
            if node.func.attr == "replace":
                require(
                    isinstance(node.func.value, ast.Name)
                    and node.func.value.id == "route",
                    "terminal auditor filesystem replace reachable",
                )
            else:
                require(
                    node.func.attr not in mutating_attributes,
                    f"terminal auditor remote mutation reachable: {node.func.attr}",
                )
            if node.func.attr == "open":
                open_mode = "r" if not node.args else ast.literal_eval(node.args[0])
                require(
                    open_mode in ("r", "rb"),
                    f"terminal auditor remote writable open reachable: {open_mode}",
                )
        elif isinstance(node.func, ast.Name):
            require(
                node.func.id not in {"exec", "eval", "open"},
                f"terminal auditor remote dynamic or writable call reachable: {node.func.id}",
            )
    return {
        "route_registry": list(ROUTE_ORDER),
        "subject_command": namespace["subject_command"]("U4-FIXED"),
        "route_stage_mode": "0755",
        "perf_record_reachable_only_from": ["T4-FIXED", "T4-REVERSED"],
        "perf_stat_routes": [],
        "pid_attach": False,
        "sigint_lifecycle": False,
        "fixed_or_reversed_routes": list(ROUTE_ORDER),
        "u4_denominator_edges": namespace["U4_EDGES"],
        "t4_denominator_edges": namespace["T4_EDGES"],
        "u4_dispatch": list(namespace["U4_DISPATCH"]),
        "t4_dispatch": list(namespace["T4_DISPATCH"]),
        "auditor_actions": list(auditor_namespace["EXTERNAL_ACTIONS"]),
        "marker_auditor_actions": list(marker_auditor_namespace["EXTERNAL_ACTIONS"]),
        "terminal_auditor_actions": list(terminal_auditor_namespace["EXTERNAL_ACTIONS"]),
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
    require(remote.get("verdict") == "D5_REMOTE_ABSENT_PROBE_PASS", "remote absent probe drift")
    require(remote.get("host", {}).get("hostname") == REMOTE_HOSTNAME, "remote host drift")
    require(remote.get("remote_writes") == 0, "remote absent probe wrote state")
    return {
        "schema": "lay.v10.e1-traversal-d5-controller-self-check.v1",
        "task_id": TASK_ID,
        "transaction_id": TRANSACTION_ID,
        "verdict": "D5_CONTROLLER_VERIFIED_UNRUN",
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
    require(remote.get("verdict") == "D5_UID_PROOF_CREATED_UNAUDITED", "remote bootstrap verdict drift")
    require(remote.get("markers_created") == 0 and remote.get("markers_consumed") == 0, "bootstrap marker ledger drift")
    require(remote.get("uid_probe", {}).get("value", {}).get("verdict") == "D5_UID_E_CAPABILITY_PROOF_PASS", "UID probe verdict drift")
    require(remote.get("perf_record") == 0 and remote.get("subject_executions") == 0, "bootstrap execution ledger drift")
    runtime_after = local_runtime_snapshot()
    require(runtime_before == runtime_after, "installed runtime changed during bootstrap")
    local_receipt = {
        "schema": "lay.v10.e1-traversal-d5-local-bootstrap.v1",
        "task_id": TASK_ID,
        "transaction_id": TRANSACTION_ID,
        "verdict": "D5_UID_PROOF_CREATED_UNAUDITED",
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
        "next_action_admitted": "independent D5 bootstrap audit only",
    }
    published = publish_local_result(
        LOCAL_BOOTSTRAP,
        REMOTE_RESULTS["BOOTSTRAP"],
        "D5_BOOTSTRAP_RECEIPT.json",
        local_receipt,
        {
            "SELF_CHECK.json": canonical_json_bytes(check),
            "REMOTE_RESPONSE.json": canonical_json_bytes(remote),
            "local-controller.py": CONTROLLER.read_bytes(),
            "remote-controller.py": REMOTE_CONTROLLER.read_bytes(),
            "bootstrap-auditor.py": BOOTSTRAP_AUDITOR.read_bytes(),
            "marker-auditor.py": MARKER_AUDITOR.read_bytes(),
            "terminal-auditor-skeleton.py": TERMINAL_AUDITOR.read_bytes(),
            "preflight-v3.json": PREFLIGHT.read_bytes(),
            "preflight-v3-receipt.json": PREFLIGHT_RECEIPT.read_bytes(),
        },
    )
    return {**local_receipt, **published}


def verify_bootstrap_audit() -> dict[str, Any]:
    value = require_file(BOOTSTRAP_AUDIT_RECEIPT, mode="0444")
    body = json.loads(BOOTSTRAP_AUDIT_RECEIPT.read_text())
    require(body.get("verdict") == "D5_UID_ACCESS_AUDIT_PASS_MARKER_CREATION", "bootstrap audit verdict drift")
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
    require(probe.get("verdict") == "D5_REMOTE_BOOTSTRAP_PROBE_PASS", "bootstrap probe drift")
    runtime_before = local_runtime_snapshot()
    remote = parse_last_json(
        remote_call("create-markers", include_audit=True, timeout=1800),
        "create-markers",
    )
    require(remote.get("verdict") == "D5_MARKERS_CREATED_UNAUDITED", "marker creation verdict drift")
    require(remote.get("markers_created") == 4 and remote.get("markers_consumed") == 0, "marker creation ledger drift")
    runtime_after = local_runtime_snapshot()
    require(runtime_before == runtime_after, "installed runtime changed during marker creation")
    local_receipt = {
        "schema": "lay.v10.e1-traversal-d5-local-marker-creation.v1",
        "task_id": TASK_ID,
        "transaction_id": TRANSACTION_ID,
        "verdict": "D5_MARKERS_CREATED_UNAUDITED",
        "remote_receipt_sha256": remote["published_receipt_sha256"],
        "bootstrap_audit_sha256": audit["sha256"],
        "inputs": inputs,
        "command_graph": graph,
        "markers_created": 4,
        "markers_consumed": 0,
        "subject_executions": 0,
        "perf_record": 0,
        "perf_stat": 0,
        "pmu_events_opened": 0,
        "runtime_before": runtime_before,
        "runtime_after": runtime_after,
        "runtime_authority_changed": False,
        "retry_permitted": False,
        "next_action_admitted": "independent D5 marker audit only",
    }
    published = publish_local_result(
        LOCAL_MARKER_CREATION,
        REMOTE_RESULTS["MARKERS"],
        "D5_MARKER_CREATION_RECEIPT.json",
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
    require(body.get("verdict") == "D5_MARKER_AUDIT_PASS_U4_FIXED_ADMITTED", "marker audit verdict drift")
    require(body.get("task_id") == TASK_ID and body.get("transaction_id") == TRANSACTION_ID, "marker audit namespace drift")
    require(body.get("markers_created") == 4 and body.get("markers_consumed") == 0, "marker audit ledger drift")
    require(body.get("remote_writes") == 0, "marker audit remote write drift")
    require(body.get("local_controller_sha256") == sha256_file(CONTROLLER), "marker-audited local controller drift")
    require(body.get("remote_controller_sha256") == sha256_file(REMOTE_CONTROLLER), "marker-audited remote controller drift")
    require(
        body.get("bootstrap_audit_sha256") == sha256_file(BOOTSTRAP_AUDIT_RECEIPT),
        "marker audit bootstrap link drift",
    )
    return {**value, "value": body}


def route_self_check(route: str) -> dict[str, Any]:
    require(route in ROUTE_ORDER, f"unknown route: {route}")
    inputs = verify_local_inputs()
    graph = verify_command_graph()
    audit = verify_bootstrap_audit()
    marker_audit = verify_marker_audit()
    require(not LOCAL_RESULTS[route].exists(), f"local route result already exists: {route}")
    route_index = ROUTE_ORDER.index(route)
    expected_pass = {
        "U4-FIXED": "U4_FIXED_PASS",
        "T4-FIXED": "T4_FIXED_PASS",
        "U4-REVERSED": "U4_REVERSED_PASS",
    }
    for previous in ROUTE_ORDER[:route_index]:
        previous_receipt = LOCAL_RESULTS[previous] / "REMOTE_EVIDENCE/D5_ROUTE_RECEIPT.json"
        require(previous_receipt.is_file(), f"local previous receipt missing: {previous}")
        verify_sha256sums(LOCAL_RESULTS[previous])
        verify_sha256sums(LOCAL_RESULTS[previous] / "REMOTE_EVIDENCE")
        require(
            json.loads(previous_receipt.read_text()).get("verdict")
            == expected_pass[previous],
            f"local previous route did not PASS: {previous}",
        )
    for pending in ROUTE_ORDER[route_index:]:
        require(not LOCAL_RESULTS[pending].exists(), f"pending local route result exists: {pending}")
    remote = parse_last_json(
        remote_call("probe-ready", route, include_audit=True, timeout=1800),
        "probe-ready",
    )
    require(remote.get("verdict") == "D5_REMOTE_READY_PROBE_PASS", "remote ready probe drift")
    require(remote.get("route") == route and remote.get("remote_writes") == 0, "remote route probe drift")
    return {
        "schema": "lay.v10.e1-traversal-d5-route-self-check.v1",
        "task_id": TASK_ID,
        "transaction_id": TRANSACTION_ID,
        "route": route,
        "verdict": "D5_ROUTE_CONTROLLER_VERIFIED_UNRUN",
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
    action = next(action for action, candidate in ACTION_TO_ROUTE.items() if candidate == route)
    remote = parse_last_json(
        remote_call(action, route, include_audit=True, timeout=7200),
        action,
    )
    require(remote.get("route") == route, "remote route receipt drift")
    require(remote.get("retry_permitted") is False, "remote retry authority drift")
    allowed = ALLOWED_TERMINALS | {
        {
            "U4-FIXED": "U4_FIXED_PASS",
            "T4-FIXED": "T4_FIXED_PASS",
            "U4-REVERSED": "U4_REVERSED_PASS",
            "T4-REVERSED": "D5_MULTIWORKER_ATTRIBUTION_PASS",
        }[route]
    }
    require(remote.get("verdict") in allowed, "remote route verdict invalid")
    post = parse_last_json(
        remote_call("probe-after", route, include_audit=True, timeout=1800),
        "probe-after",
    )
    require(post.get("verdict") == "D5_REMOTE_POST_PROBE_PASS", "remote post probe drift")
    require(post.get("route_verdict") == remote.get("verdict"), "remote post verdict drift")
    runtime_after = local_runtime_snapshot()
    require(runtime_before == runtime_after, "installed runtime changed during route")
    local_receipt = {
        "schema": "lay.v10.e1-traversal-d5-local-route.v1",
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
        "subject_executions": remote.get("subject_executions"),
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
        "D5_ROUTE_RECEIPT.json",
        local_receipt,
        {
            "SELF_CHECK.json": canonical_json_bytes(check),
            "REMOTE_RESPONSE.json": canonical_json_bytes(remote),
            "REMOTE_POST_PROBE.json": canonical_json_bytes(post),
            "local-controller.py": CONTROLLER.read_bytes(),
            "remote-controller.py": REMOTE_CONTROLLER.read_bytes(),
            "bootstrap-audit.json": BOOTSTRAP_AUDIT_RECEIPT.read_bytes(),
            "marker-audit.json": MARKER_AUDIT_RECEIPT.read_bytes(),
            "preflight-v3.json": PREFLIGHT.read_bytes(),
            "preflight-v3-receipt.json": PREFLIGHT_RECEIPT.read_bytes(),
        },
    )
    copied = json.loads(
        (LOCAL_RESULTS[route] / "REMOTE_EVIDENCE/D5_ROUTE_RECEIPT.json").read_text()
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
        else:
            value = run_route_once(ACTION_TO_ROUTE[arguments.action])
        print(json.dumps(value, ensure_ascii=False, sort_keys=True))
        return 0
    except Exception as error:
        print(f"D5 LOCAL ERROR: {type(error).__name__}: {error}", file=sys.stderr)
        return 1


if __name__ == "__main__":
    raise SystemExit(main())
