#!/usr/bin/env python3
"""Local verifier/orchestrator for primary-only D2 task-clock routes."""

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


TASK_ID = "slice8b-v10-e1-traversal-d2-primary-only-v2-20260825"
TRANSACTION_ID = "35dbe5c5eaec7dbb5b5b112313a3285dbe2ceaea0a910719ad379255ca5569e7"
REMOTE = "e@192.168.3.94"
REMOTE_HOSTNAME = "e-MEGA-MINI-M1-13th"
SSH_IDENTITY = pathlib.Path("/home/ubu/.ssh/mega-mini-admin")

CONTROLLER = pathlib.Path(__file__).resolve()
PROJECT_ROOT = CONTROLLER.parents[1]
REMOTE_CONTROLLER = PROJECT_ROOT / "scripts/lay-v10-e1-traversal-d2-t-sampling-remote.py"
PREFLIGHT = PROJECT_ROOT / (
    "docs/structural_gates/preflights/"
    "LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_D2_PRIMARY_ONLY_"
    "IMPLEMENTATION_V4_2026-08-25.json"
)
PREFLIGHT_RECEIPT = PROJECT_ROOT / (
    "docs/structural_gates/receipts/"
    "LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_D2_PRIMARY_ONLY_"
    "IMPLEMENTATION_V4_PREFLIGHT_2026-08-25.json"
)
U_SINGLE_SALVAGE = PROJECT_ROOT / (
    "docs/structural_gates/receipts/"
    "LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_D2_PRIMARY_ONLY_"
    "U_SINGLE_SALVAGE_V1_2026-08-26/U_SINGLE_SALVAGE_RECEIPT.json"
)
LOCAL_ELF = PROJECT_ROOT / (
    "docs/structural_gates/receipts/"
    "LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_D2_PRIMARY_ONLY_"
    "BUILD_V1_2026-08-25/REMOTE_EVIDENCE/d2-test-elf"
)
LOCAL_MAP = PROJECT_ROOT / (
    "docs/structural_gates/receipts/"
    "LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_D2_PRIMARY_ONLY_"
    "BUCKET_MAP_V1_2026-08-26/REMOTE_EVIDENCE/D2_BUCKET_MAP.json"
)
REMOTE_PARENT = pathlib.Path("/home/e/.local/share/lay/provenance") / TASK_ID

PREFLIGHT_SHA256 = "e9d6328b9f610ede73ae2e8d3c819b9728ac0fc5e4c263e9fe25a61978f80f5a"
PREFLIGHT_RECEIPT_SHA256 = "740c008d59fb4689826537e46a35da554bde863358d2c18382f315395ee835e0"
U_SINGLE_SALVAGE_SHA256 = "9617502776537ca4181bd9bf195e1fd5b8fbd2679f1dfd00737f128cb88bfe0b"
ELF_SHA256 = "bb7c68528ec3fec7074919c3c64cce233d48c26efefe9d9b32735c5f5865a178"
MAP_SHA256 = "2b93aedf11101a8bc5fc7ed4ffaa056b50476d96cdd2eabf9d185f59ee60f846"

UV_RECEIPTS = {
    "U-FIXED": (
        "U_FIXED_V1_2026-08-26",
        "0ef07db4b8a07efb2ed09c3c47d6b0a9ef88e4529dd436d5d375cecf62339b59",
        "U_FIXED_PASS",
    ),
    "U-REVERSED": (
        "U_REVERSED_V1_2026-08-26",
        "080917d52f3e36abffb6eab47b9e56c4a1e771b4da21caaf6fa3e2cfe686a0fb",
        "ALL_U_PASS",
    ),
    "V-FIXED-INSTR": (
        "V_FIXED_INSTR_V1_2026-08-26",
        "56c862759c95de6682571aed3d68098dab084319817fba5af3853042e0396bae",
        "V_FIXED_PASS",
    ),
    "V-REVERSED-INSTR": (
        "V_REVERSED_INSTR_V1_2026-08-26",
        "5d75a502ad9e509dc6810b5494067f602d5ace1237e73f4d7913d5b9b1fc9de2",
        "ALL_UV_VALIDITY_PASS",
    ),
}

ROUTE_ORDER = ("T-SINGLE", "T-FIXED", "T-REVERSED")
ROUTES: dict[str, dict[str, Any]] = {
    "T-SINGLE": {
        "pass_state": "T_SINGLE_PASS",
        "remote_result": "t-single-v1",
        "local_result": "T_SINGLE_V1",
        "test": "nanda_wave::l2_field::v13_typed_peak::tests::v10_d1_component_single",
        "cpus": [0],
    },
    "T-FIXED": {
        "pass_state": "T_FIXED_PASS",
        "remote_result": "t-fixed-v1",
        "local_result": "T_FIXED_V1",
        "test": "nanda_wave::l2_field::v13_typed_peak::tests::v10_d1_component_twenty",
        "cpus": list(range(20)),
    },
    "T-REVERSED": {
        "pass_state": "ALL_T_PASS",
        "remote_result": "t-reversed-v1",
        "local_result": "T_REVERSED_V1",
        "test": "nanda_wave::l2_field::v13_typed_peak::tests::v10_d1_component_twenty",
        "cpus": list(reversed(range(20))),
    },
}
EXTERNAL_ACTIONS = ("self-check", "run-once")
T_TERMINALS = {
    "BLOCKED_PROVENANCE",
    "BLOCKED_THERMAL",
    "BLOCKED_CAPABILITY",
    "BLOCKED_BUCKET_MAP",
    "BLOCKED_PERTURBATION",
    "BLOCKED_SAMPLE_COVERAGE",
}


class ControllerError(RuntimeError):
    pass


def require(condition: bool, message: str) -> None:
    if not condition:
        raise ControllerError(message)


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
        require(value["sha256"] == digest, f"SHA mismatch: {path}")
    if size is not None:
        require(value["size_bytes"] == size, f"size mismatch: {path}")
    if mode is not None:
        require(value["mode"] == mode, f"mode mismatch: {path}")
    return value


def canonical_json_bytes(value: Any) -> bytes:
    return json.dumps(value, ensure_ascii=False, sort_keys=True, separators=(",", ":")).encode()


def write_new_bytes(path: pathlib.Path, value: bytes, mode: int = 0o444) -> None:
    descriptor = os.open(path, os.O_WRONLY | os.O_CREAT | os.O_EXCL, 0o600)
    try:
        view = memoryview(value)
        while view:
            written = os.write(descriptor, view)
            require(written > 0, "short write made no progress")
            view = view[written:]
        os.fsync(descriptor)
        os.fchmod(descriptor, mode)
    finally:
        os.close(descriptor)


def write_new_json(path: pathlib.Path, value: Any, mode: int = 0o444) -> None:
    write_new_bytes(
        path,
        json.dumps(value, ensure_ascii=False, sort_keys=True, indent=2).encode() + b"\n",
        mode,
    )


def inventory(root: pathlib.Path) -> list[dict[str, Any]]:
    values = []
    for path in sorted(root.rglob("*")):
        require(not path.is_symlink(), f"symlink in evidence: {path}")
        if path.is_file():
            values.append(
                {
                    "path": path.relative_to(root).as_posix(),
                    "mode": mode_string(path),
                    "size_bytes": path.stat().st_size,
                    "sha256": sha256_file(path),
                }
            )
    return values


def write_sha256sums(root: pathlib.Path) -> None:
    values = [value for value in inventory(root) if value["path"] != "SHA256SUMS"]
    write_new_bytes(
        root / "SHA256SUMS",
        "".join(f"{value['sha256']}  {value['path']}\n" for value in values).encode(),
    )


def verify_sha256sums(root: pathlib.Path) -> int:
    manifest = root / "SHA256SUMS"
    require(manifest.is_file(), f"missing SHA256SUMS: {root}")
    seen: set[str] = set()
    for line in manifest.read_text().splitlines():
        digest, separator, relative = line.partition("  ")
        require(separator == "  " and len(digest) == 64, f"bad manifest row: {line}")
        pure = pathlib.PurePosixPath(relative)
        require(not pure.is_absolute() and ".." not in pure.parts and relative not in seen, f"unsafe manifest path: {relative}")
        seen.add(relative)
        require(sha256_file(root / pure) == digest, f"manifest mismatch: {relative}")
    actual = {value["path"] for value in inventory(root) if value["path"] != "SHA256SUMS"}
    require(seen == actual, f"manifest membership mismatch: {root}")
    return len(seen)


def seal_tree(root: pathlib.Path) -> None:
    for path in sorted(root.rglob("*"), reverse=True):
        require(not path.is_symlink(), f"symlink before seal: {path}")
        path.chmod(0o555 if path.is_dir() or path.stat().st_mode & 0o111 else 0o444)
    root.chmod(0o555)


def fsync_directory(path: pathlib.Path) -> None:
    descriptor = os.open(path, os.O_RDONLY | os.O_DIRECTORY)
    try:
        os.fsync(descriptor)
    finally:
        os.close(descriptor)


def remove_owned_tree(root: pathlib.Path) -> None:
    if not root.exists():
        return
    for path in [root, *root.rglob("*")]:
        path.chmod(0o700 if path.is_dir() else 0o600)
    shutil.rmtree(root)


def run(
    command: Sequence[str],
    *,
    input_bytes: bytes | None = None,
    check: bool = True,
    timeout: int | None = None,
) -> subprocess.CompletedProcess[bytes]:
    result = subprocess.run(
        list(command),
        input=input_bytes,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        check=False,
        timeout=timeout,
    )
    if check and result.returncode != 0:
        raise ControllerError(
            f"command failed ({result.returncode}): {shlex.join(command)}\n"
            f"{result.stderr.decode(errors='replace')[-6000:]}"
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
    "sys.argv=['lay-v10-e1-traversal-d2-t-sampling-remote.py',base64.b64encode(payload).decode()]\n"
    "ns={'__name__':'__main__','__file__':'<lay-v10-e1-traversal-d2-t-sampling-remote.py>'}\n"
    "exec(compile(source,ns['__file__'],'exec'),ns)\n"
)


def payload(action: str, route: str) -> bytes:
    local_source = CONTROLLER.read_bytes()
    remote_source = REMOTE_CONTROLLER.read_bytes()
    return canonical_json_bytes(
        {
            "action": action,
            "route": route,
            "preflight_b64": base64.b64encode(PREFLIGHT.read_bytes()).decode(),
            "preflight_receipt_b64": base64.b64encode(PREFLIGHT_RECEIPT.read_bytes()).decode(),
            "u_single_salvage_b64": base64.b64encode(U_SINGLE_SALVAGE.read_bytes()).decode(),
            "local_controller_sha256": sha256_bytes(local_source),
            "local_controller_b64": base64.b64encode(local_source).decode(),
            "remote_controller_sha256": sha256_bytes(remote_source),
            "remote_controller_b64": base64.b64encode(remote_source).decode(),
        }
    )


def remote_call(action: str, route: str, *, timeout: int) -> subprocess.CompletedProcess[bytes]:
    remote_source = REMOTE_CONTROLLER.read_bytes()
    request = payload(action, route)
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
    return run(ssh_argv(command), input_bytes=envelope, check=False, timeout=timeout)


def parse_last_json(result: subprocess.CompletedProcess[bytes], action: str) -> dict[str, Any]:
    require(
        result.returncode == 0,
        f"remote {action} failed ({result.returncode}):\n"
        f"{result.stderr.decode(errors='replace')[-8000:]}",
    )
    lines = result.stdout.decode(errors="replace").strip().splitlines()
    require(bool(lines), f"remote {action} produced no output")
    return json.loads(lines[-1])


def local_result(route: str) -> pathlib.Path:
    return PROJECT_ROOT / (
        "docs/structural_gates/receipts/"
        "LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_D2_PRIMARY_ONLY_"
        f"{ROUTES[route]['local_result']}_2026-08-26"
    )


def uv_receipt_path(suffix: str) -> pathlib.Path:
    return PROJECT_ROOT / (
        "docs/structural_gates/receipts/"
        "LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_D2_PRIMARY_ONLY_"
        f"{suffix}/REMOTE_EVIDENCE/D2_UV_ROUTE_RECEIPT.json"
    )


def local_runtime_snapshot() -> dict[str, Any]:
    launcher = pathlib.Path.home() / ".local/bin/lay"
    resolved = launcher.resolve(strict=True)
    return {
        "launcher": str(launcher),
        "resolved": str(resolved),
        "resolved_sha256": sha256_file(resolved),
    }


def verify_local_admission(route: str) -> dict[str, Any]:
    require(route in ROUTES, f"unknown route: {route}")
    require(not local_result(route).exists(), f"local T result already exists: {route}")
    require_file(SSH_IDENTITY, mode="0600")
    preflight = require_file(PREFLIGHT, digest=PREFLIGHT_SHA256, size=98_316, mode="0444")
    preflight_value = json.loads(PREFLIGHT.read_text())
    require(preflight_value.get("scoped_positive_verdict") == "READY_TO_IMPLEMENT_PRIMARY_ONLY_D2", "preflight verdict drift")
    preflight_receipt = require_file(
        PREFLIGHT_RECEIPT, digest=PREFLIGHT_RECEIPT_SHA256, size=14_171, mode="0444"
    )
    preflight_receipt_value = json.loads(PREFLIGHT_RECEIPT.read_text())
    require(
        preflight_receipt_value.get("verdict") == "READY_TO_IMPLEMENT"
        and preflight_receipt_value.get("safe_to_implement") is True,
        "preflight receipt drift",
    )
    salvage = require_file(
        U_SINGLE_SALVAGE, digest=U_SINGLE_SALVAGE_SHA256, size=2_531, mode="0444"
    )
    salvage_value = json.loads(U_SINGLE_SALVAGE.read_text())
    require(
        salvage_value.get("verdict") == "U_SINGLE_RECOVERED_FROM_SEALED_EVIDENCE"
        and salvage_value.get("effective_route_state") == "U_SINGLE_PASS",
        "U-SINGLE salvage drift",
    )
    uv = {}
    for name, (suffix, digest, verdict) in UV_RECEIPTS.items():
        path = uv_receipt_path(suffix)
        value = require_file(path, digest=digest, mode="0444")
        body = json.loads(path.read_text())
        require(body.get("route") == name and body.get("verdict") == verdict, f"UV receipt verdict drift: {name}")
        uv[name] = value
    elf = require_file(LOCAL_ELF, digest=ELF_SHA256, size=317_706_232, mode="0555")
    map_value = require_file(LOCAL_MAP, digest=MAP_SHA256, mode="0444")
    return {
        "preflight": preflight,
        "preflight_receipt": preflight_receipt,
        "u_single_salvage": salvage,
        "uv_receipts": uv,
        "elf": elf,
        "map": map_value,
        "controller": file_identity(CONTROLLER),
        "remote_controller": file_identity(REMOTE_CONTROLLER),
    }


def verify_command_graph() -> dict[str, Any]:
    source = REMOTE_CONTROLLER.read_text()
    compile(CONTROLLER.read_text(), str(CONTROLLER), "exec")
    compile(source, str(REMOTE_CONTROLLER), "exec")
    namespace: dict[str, Any] = {"__name__": "d2_t_static_registry", "__file__": str(REMOTE_CONTROLLER)}
    exec(compile(source, str(REMOTE_CONTROLLER), "exec"), namespace)
    remote_order = tuple(namespace["ROUTE_ORDER"])
    remote_routes = namespace["ROUTES"]
    require(remote_order == ROUTE_ORDER and tuple(ROUTES) == ROUTE_ORDER, "T route registry drift")
    for name in ROUTE_ORDER:
        require(remote_routes[name]["test"] == ROUTES[name]["test"], f"T test drift: {name}")
        require(remote_routes[name]["cpus"] == ROUTES[name]["cpus"], f"T CPU drift: {name}")
        require(remote_routes[name]["pass_state"] == ROUTES[name]["pass_state"], f"T state drift: {name}")
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
            "100000",
        ),
        "perf record prefix drift",
    )
    readers = namespace["reader_commands"](pathlib.Path("/tmp/perf.data"))
    require(
        readers
        == {
            "evlist": ["/usr/bin/perf", "evlist", "-v", "-i", "/tmp/perf.data"],
            "samples": [
                "/usr/bin/perf",
                "script",
                "-i",
                "/tmp/perf.data",
                "-F",
                "comm,pid,tid,cpu,time,event,ip,dso,period",
            ],
            "raw-records": ["/usr/bin/perf", "script", "-D", "-i", "/tmp/perf.data"],
            "buildids": ["/usr/bin/perf", "buildid-list", "-i", "/tmp/perf.data"],
        },
        "T reader graph drift",
    )
    require(
        tuple(namespace["T_DISPATCH"])
        == (
            ("provenance", "BLOCKED_PROVENANCE"),
            ("thermal", "BLOCKED_THERMAL"),
            ("capability", "BLOCKED_CAPABILITY"),
            ("bucket_map", "BLOCKED_BUCKET_MAP"),
            ("perturbation", "BLOCKED_PERTURBATION"),
            ("sample_coverage", "BLOCKED_SAMPLE_COVERAGE"),
        ),
        "T dispatch drift",
    )
    tree = ast.parse(source, filename=str(REMOTE_CONTROLLER))
    constants = {
        node.value for node in ast.walk(tree) if isinstance(node, ast.Constant) and isinstance(node.value, str)
    }
    for forbidden in ("--pid", "SIGINT", "I-CORE", "I-ATOM", "--no-inherit", "cargo", "rustc", "stat"):
        require(forbidden not in constants, f"forbidden T executable token: {forbidden}")
    call_names = []
    for node in ast.walk(tree):
        if isinstance(node, ast.Call) and isinstance(node.func, ast.Attribute) and isinstance(node.func.value, ast.Name):
            call_names.append(f"{node.func.value.id}.{node.func.attr}")
    require(call_names.count("subprocess.Popen") == 0, "T lifecycle must not use Popen")
    require(call_names.count("os.rename") == 3, "T marker/result/failure rename cardinality drift")
    return {
        "external_actions": list(EXTERNAL_ACTIONS),
        "remote_actions": ["probe-before", "run-once", "probe-after"],
        "route_registry": list(ROUTE_ORDER),
        "perf_record_reachable_only_from": list(ROUTE_ORDER),
        "perf_readers_per_route": list(namespace["READER_KINDS"]),
        "perf_stat_routes": [],
        "cargo_routes": [],
        "pid_attach": False,
        "sigint_lifecycle": False,
        "dispatch": [cause for cause, _terminal in namespace["T_DISPATCH"]],
    }


def verify_probe(value: Mapping[str, Any], route: str, *, post: bool) -> None:
    expected = "D2_T_REMOTE_POST_PROBE_PASS" if post else "D2_T_REMOTE_PROBE_PASS"
    require(value.get("verdict") == expected, "remote T probe verdict drift")
    require(value.get("route") == route, "remote T route drift")
    require(value.get("hostname") == REMOTE_HOSTNAME, "remote host drift")
    require(value.get("elf", {}).get("sha256") == ELF_SHA256, "remote ELF drift")
    require(value.get("map", {}).get("sha256") == MAP_SHA256, "remote map drift")
    require(value.get("map_check", {}).get("machine_byte_mismatches") == [], "remote map bytes drift")
    require(len(value.get("markers", [])) == 11, "remote marker count drift")


def self_check(route: str) -> dict[str, Any]:
    admission = verify_local_admission(route)
    graph = verify_command_graph()
    probe = parse_last_json(remote_call("probe-before", route, timeout=600), "probe-before")
    verify_probe(probe, route, post=False)
    return {
        "schema": "lay.v10.e1-traversal-d2-primary-only-t-self-check.v1",
        "task_id": TASK_ID,
        "transaction_id": TRANSACTION_ID,
        "route": route,
        "verdict": "D2_T_CONTROLLER_VERIFIED_ROUTE_UNRUN",
        "admission": admission,
        "command_graph": graph,
        "remote_probe_sha256": sha256_bytes(canonical_json_bytes(probe)),
        "marker_mutations": 0,
        "subject_executions": 0,
        "cargo_invocations": 0,
        "perf_record": 0,
        "perf_readers": 0,
        "perf_stat": 0,
        "pmu_events_opened": 0,
    }


def copy_remote_evidence(route: str, destination: pathlib.Path) -> None:
    remote_result = REMOTE_PARENT / ROUTES[route]["remote_result"]
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
            f"{REMOTE}:{remote_result}",
            str(destination),
        ],
        check=False,
        timeout=2400,
    )
    require(result.returncode == 0, result.stderr.decode(errors="replace")[-6000:])


def run_once(route: str) -> dict[str, Any]:
    check = self_check(route)
    runtime_before = local_runtime_snapshot()
    remote_receipt = parse_last_json(remote_call("run-once", route, timeout=5400), "run-once")
    require(remote_receipt.get("route") == route, "remote T route receipt drift")
    require(remote_receipt.get("retry_permitted") is False, "remote retry authority drift")
    require(remote_receipt.get("perf_record") == 1, "T perf record count drift")
    require(remote_receipt.get("perf_readers") == 4, "T reader count drift")
    require(remote_receipt.get("perf_stat") == 0, "perf stat invoked during T")
    allowed = T_TERMINALS | {ROUTES[route]["pass_state"]}
    require(remote_receipt.get("verdict") in allowed, "remote T verdict invalid")
    post = parse_last_json(remote_call("probe-after", route, timeout=1200), "probe-after")
    verify_probe(post, route, post=True)
    require(post.get("route_verdict") == remote_receipt.get("verdict"), "post-probe T verdict drift")
    runtime_after = local_runtime_snapshot()
    require(runtime_before == runtime_after, "installed runtime projection changed during T")

    destination = local_result(route)
    stage = pathlib.Path(f"{destination}.stage-{os.getpid()}-{time.time_ns()}")
    stage.mkdir(parents=True, mode=0o700)
    try:
        remote_evidence = stage / "REMOTE_EVIDENCE"
        copy_remote_evidence(route, remote_evidence)
        entries = verify_sha256sums(remote_evidence)
        receipt_file = remote_evidence / "D2_T_ROUTE_RECEIPT.json"
        copied = json.loads(receipt_file.read_text())
        require(copied.get("route") == route, "copied T route receipt drift")
        require(copied.get("verdict") == remote_receipt.get("verdict"), "copied T verdict drift")
        require(copied.get("runtime_authority_changed") is False, "remote runtime authority drift")
        write_new_json(stage / "SELF_CHECK.json", check)
        write_new_json(stage / "RUNTIME_BEFORE.json", runtime_before)
        write_new_json(stage / "RUNTIME_AFTER.json", runtime_after)
        write_new_bytes(stage / "local-controller.py", CONTROLLER.read_bytes())
        write_new_bytes(stage / "remote-controller.py", REMOTE_CONTROLLER.read_bytes())
        write_new_bytes(stage / "preflight-v4.json", PREFLIGHT.read_bytes())
        write_new_bytes(stage / "preflight-v4-receipt.json", PREFLIGHT_RECEIPT.read_bytes())
        write_new_bytes(stage / "u-single-salvage-v1.json", U_SINGLE_SALVAGE.read_bytes())
        local_receipt = {
            "schema": "lay.v10.e1-traversal-d2-primary-only-local-t-route.v1",
            "task_id": TASK_ID,
            "transaction_id": TRANSACTION_ID,
            "route": route,
            "verdict": copied["verdict"],
            "dispatch": copied.get("dispatch"),
            "remote_manifest_entries": entries,
            "remote_receipt_sha256": sha256_file(receipt_file),
            "marker_consumed": True,
            "cargo_invocations": 0,
            "perf_record": 1,
            "perf_readers": 4,
            "perf_stat": 0,
            "pmu_events_opened": 1,
            "d2_subject_executions": 1,
            "runtime_before": runtime_before,
            "runtime_after": runtime_after,
            "runtime_authority_changed": False,
            "retry_permitted": False,
            "next_action_admitted": copied.get("next_action_admitted"),
        }
        write_new_json(stage / "LOCAL_T_ROUTE_RECEIPT.json", local_receipt)
        write_sha256sums(stage)
        seal_tree(stage)
        os.rename(stage, destination)
        fsync_directory(destination.parent)
    except BaseException:
        remove_owned_tree(stage)
        raise
    return {
        "route": route,
        "verdict": remote_receipt["verdict"],
        "local_result": str(destination),
        "remote_receipt_sha256": sha256_file(destination / "REMOTE_EVIDENCE/D2_T_ROUTE_RECEIPT.json"),
        "runtime_authority_changed": False,
        "retry_permitted": False,
        "next_action_admitted": remote_receipt.get("next_action_admitted"),
    }


def parser() -> argparse.ArgumentParser:
    value = argparse.ArgumentParser()
    value.add_argument("action", choices=EXTERNAL_ACTIONS)
    value.add_argument("route", choices=ROUTE_ORDER)
    return value


def main() -> int:
    arguments = parser().parse_args()
    try:
        value = self_check(arguments.route) if arguments.action == "self-check" else run_once(arguments.route)
        print(json.dumps(value, ensure_ascii=False, sort_keys=True))
        return 0
    except Exception as error:
        print(f"D2 T ERROR: {type(error).__name__}: {error}", file=sys.stderr)
        return 1


if __name__ == "__main__":
    raise SystemExit(main())
