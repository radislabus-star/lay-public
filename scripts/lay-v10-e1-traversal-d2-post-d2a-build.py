#!/usr/bin/env python3
"""One-shot post-D2A builder. Stops at D2_BUILD_CREATED_UNAUDITED."""

from __future__ import annotations

import argparse
import base64
import contextlib
import fcntl
import hashlib
import json
import os
import pathlib
import shlex
import shutil
import signal
import stat
import subprocess
import sys
import time
from typing import Any, Iterable, Mapping, Sequence


TASK_ID = "slice8b-v10-e1-traversal-d2-primary-only-v2-20260825"
TRANSACTION_ID = "35dbe5c5eaec7dbb5b5b112313a3285dbe2ceaea0a910719ad379255ca5569e7"
REMOTE = "e@192.168.3.94"
REMOTE_HOSTNAME = "e-MEGA-MINI-M1-13th"
REMOTE_MACHINE_ID_SHA256 = "5ac0bb537745673ef80942c2df32d586f5753c85de1c8b2453aa4e7317b77441"
SSH_IDENTITY = pathlib.Path("/home/ubu/.ssh/mega-mini-admin")

REMOTE_PARENT = pathlib.Path("/home/e/.local/share/lay/provenance") / TASK_ID
REMOTE_D2A = REMOTE_PARENT / "d2a-v1"
REMOTE_D2A_FAILURE = REMOTE_PARENT / "d2a-failure-v1"
REMOTE_BUILD = REMOTE_PARENT / "build-v1"
REMOTE_BUILD_FAILURE = REMOTE_PARENT / "build-failure-v1"
REMOTE_STATE = pathlib.Path("/home/e/.local/state/lay") / TASK_ID
REMOTE_MARKERS = REMOTE_STATE / "markers"
REMOTE_BUILD_STATE = REMOTE_STATE / "BUILD_STATE.json"
REMOTE_B0A = pathlib.Path(
    "/home/e/.local/share/lay/provenance/"
    "slice8b-v10-hardware-b0-b2-v3-20260824/b0a-input-closure-v2"
)
REMOTE_SOURCE_CLOSURE = REMOTE_B0A / "inputs/surviving-source-closure"
REMOTE_ARTIFACTS = REMOTE_B0A / "inputs/slice8b-v10-f6178f/artifacts"
REMOTE_CARGO_TOML = REMOTE_ARTIFACTS / "Cargo.toml"
REMOTE_CARGO_LOCK = REMOTE_ARTIFACTS / "Cargo.lock"
REMOTE_V10 = REMOTE_ARTIFACTS / "v13_typed_peak.v10.rs"
REMOTE_CARGO_GUARD = REMOTE_B0A / "inputs/controller/cargo-guard.sh"
REMOTE_FRAGMENT = REMOTE_D2A / "inputs/fragment.inc"
REMOTE_D2A_RECEIPT = REMOTE_D2A / "D2A_RECEIPT.json"

CONTROLLER = pathlib.Path(__file__).resolve()
PROJECT_ROOT = CONTROLLER.parents[1]
D2A_RESULT = PROJECT_ROOT / (
    "docs/structural_gates/receipts/"
    "LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_D2_PRIMARY_ONLY_V2_2026-08-25"
)
AUDIT_RESULT = PROJECT_ROOT / (
    "docs/structural_gates/receipts/"
    "LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_D2_PRIMARY_ONLY_"
    "D2A_INDEPENDENT_AUDIT_V2_2026-08-25"
)
LOCAL_D2A_RECEIPT = D2A_RESULT / "D2A_RECEIPT.json"
LOCAL_AUDIT_RECEIPT = AUDIT_RESULT / "D2A_INDEPENDENT_AUDIT_RECEIPT.json"
PRODUCER_CONTROLLER = PROJECT_ROOT / "scripts/lay-v10-e1-traversal-d2-primary-only.py"
AUDITOR = PROJECT_ROOT / "scripts/lay-v10-e1-traversal-d2a-independent-audit-v2.py"
LOCAL_RESULT = PROJECT_ROOT / (
    "docs/structural_gates/receipts/"
    "LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_D2_PRIMARY_ONLY_BUILD_V1_2026-08-25"
)

REMOTE_EXECUTION = bool(globals().get("REMOTE_EXECUTION", False))
CONTROLLER_SOURCE_BYTES = bytes(globals().get("CONTROLLER_SOURCE_BYTES", b""))
AUDIT_RECEIPT_BYTES = bytes(globals().get("AUDIT_RECEIPT_BYTES", b""))

EXPECTED = {
    "d2a_receipt": "998ca180a976384acb215b9e72a8d956fd830fdf6a1c0641b59eea10cbb00e0f",
    "d2a_receipt_size": 21_910,
    "audit_receipt": "ca681196361bb434d16fc334e4481609441e891c76d4e5f3728f93be945d4168",
    "audit_receipt_size": 13_868,
    "producer_controller": "9329a32b9e4e9edf5d83ddf624e8c9ce6a456494057f4ea3ef5aff6f382d6ec0",
    "producer_controller_size": 64_492,
    "auditor": "28199196ad542d77063962b128d8d82db92a9864ae951e151d0da2f70d361c68",
    "auditor_size": 33_335,
    "d2a_state": "fb7de0be1dbb7a99c2ddcb2bd1dbc7f469d4fc975b6a564546fd6994c196075a",
    "d2a_state_size": 3_707,
    "route_lock": "ddfafcaec3c8068ea0b853cb8a34cf0b40408fbbdc137a6dae3932b5396c3c5d",
    "route_lock_size": 249,
    "b0a_manifest": "920374efc2e75a021d53235aafea2a74dc7258546219bcae9c6a2bf53e194916",
    "b0a_receipt": "48176ec6faae86f43ddda8404542367be5b6c9d6813762dedb829b4946593eb3",
    "v10_source": "f6178f3882c1a216a9c2aaf7ff59dd6bdd78798708516b753ea7208177d9718c",
    "v10_source_size": 91_518,
    "d1_fragment": "bbd8b8d318810eec721812f21efbeb5f231dacba774cb5ade854e2201c6c7665",
    "d1_fragment_size": 113_204,
    "assembled_source": "6cd9edece91ac2e0c0e6dda7658e104dcf8953f1c16b1acff6108ea44ada0181",
    "assembled_source_size": 204_722,
    "production_prefix": "ce9ea2d290602774e2f45444e10e6fdabce6af4a61ae2bdca54c6d2e31a53f26",
    "production_prefix_size": 39_047,
    "cargo_toml": "90a3ae6b16677ff49b70aab3457b794cd1e114a547f9e113a8b49595f976207b",
    "cargo_toml_size": 2_399,
    "cargo_lock": "e6399be48c393e1557b451fcb04d886f9fbdaa9812109410ac3d9ecdc98b93f1",
    "cargo_lock_size": 70_770,
    "cargo_guard": "a5b01f9a12b32ee2bcfc957a5e9f65efb52fe6e7644850ae3f00e412648243fe",
    "cargo_guard_size": 2_534,
}

MARKERS = {
    "build.available": ("BUILD", "d21b10eac837b740fa6cb9e84c75ff76bc5ec9dc388f0896575d6f697e0c2964", 478),
    "bucket-map.available": ("BUCKET-MAP", "4471cb4edbaa8c7fc7a2f7b2ffeb4894e9b66cc904d81c0e8b1122c506745bb7", 483),
    "parity.available": ("PARITY", "ef5eef0d5ae91bea7bff2c1077cceb8c77d2f2d5a0e7263a70f044c648115c55", 479),
    "u-single.available": ("U-SINGLE", "bb7b16f685e7c1a8818bc7185f0d6991f83183d8f035e90688fc66d83ba2a46b", 481),
    "u-fixed.available": ("U-FIXED", "58435bf78041efe8a24191551c48062a9a0617d9ac9d468b918138b268ed27a8", 480),
    "u-reversed.available": ("U-REVERSED", "c13f9e22ead22c1f3afe231848a49673c93690c88ba5bbaa1426e1f46507fe0e", 483),
    "v-fixed-instr.available": ("V-FIXED-INSTR", "760e09bb85418e31732fcf24f93e8e81d683ecaded94e4afd29bc5d44c1c2f82", 486),
    "v-reversed-instr.available": ("V-REVERSED-INSTR", "a87b98f363b0c51f1a36896d1892cf0c508997ab91f05980480770c6601583dc", 489),
    "t-single.available": ("T-SINGLE", "8f9e716a687622cd04f693350371228072c1a303d65834c6b647fd900322fe7b", 481),
    "t-fixed.available": ("T-FIXED", "7915c483243c7116f2d023895948667cda3708f23afa7bcc7abed614772f49b0", 480),
    "t-reversed.available": ("T-REVERSED", "26eecf8ae977c0428a5371c30d85277e5974e2ec5768629ac95212cd8cd20c9e", 483),
}

EXTERNAL_ACTIONS = ("self-check", "build-once")
BUILD_COMMAND_TEMPLATE = (
    "<workspace>/scripts/cargo-guard.sh",
    "test",
    "--offline",
    "--locked",
    "--release",
    "--lib",
    "--no-run",
    "nanda_wave::l2_field::v13_typed_peak::tests",
)
FROZEN_BUILD_ENVIRONMENT = {
    "CARGO_BUILD_JOBS": "20",
    "CARGO_INCREMENTAL": "0",
    "CARGO_NET_OFFLINE": "true",
    "CARGO_PROFILE_RELEASE_DEBUG": "2",
    "CARGO_PROFILE_RELEASE_STRIP": "none",
    "RUSTFLAGS": "",
}
MIN_FREE_BYTES = 15 * 1024**3


class ControllerError(RuntimeError):
    pass


def require(condition: bool, message: str) -> None:
    if not condition:
        raise ControllerError(message)


def now() -> str:
    return time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime())


def controller_source() -> bytes:
    if REMOTE_EXECUTION:
        require(bool(CONTROLLER_SOURCE_BYTES), "remote controller source is absent")
        return CONTROLLER_SOURCE_BYTES
    return CONTROLLER.read_bytes()


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
    require(path.is_file(), f"missing file: {path}")
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
        require(value["sha256"] == digest, f"SHA-256 mismatch: {path}")
    if size is not None:
        require(value["size_bytes"] == size, f"size mismatch: {path}")
    if mode is not None:
        require(value["mode"] == mode, f"mode mismatch: {path}")
    return value


def load_json(path: pathlib.Path) -> Any:
    with path.open("rb") as source:
        return json.load(source)


def canonical_json_bytes(value: Any) -> bytes:
    return (json.dumps(value, ensure_ascii=False, sort_keys=True, separators=(",", ":")) + "\n").encode()


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
    write_new_bytes(path, json.dumps(value, ensure_ascii=False, sort_keys=True, indent=2).encode() + b"\n", mode)


def fsync_directory(path: pathlib.Path) -> None:
    descriptor = os.open(path, os.O_RDONLY | os.O_DIRECTORY)
    try:
        os.fsync(descriptor)
    finally:
        os.close(descriptor)


def fsync_tree(root: pathlib.Path) -> None:
    for path in sorted(root.rglob("*")):
        require(not path.is_symlink(), f"symlink is forbidden: {path}")
        if path.is_file():
            descriptor = os.open(path, os.O_RDONLY)
            try:
                os.fsync(descriptor)
            finally:
                os.close(descriptor)
    for path in sorted((candidate for candidate in root.rglob("*") if candidate.is_dir()), reverse=True):
        fsync_directory(path)
    fsync_directory(root)


def inventory(root: pathlib.Path) -> list[dict[str, Any]]:
    rows = []
    for path in sorted(root.rglob("*")):
        require(not path.is_symlink(), f"symlink is forbidden: {path}")
        if path.is_file():
            rows.append(
                {
                    "path": path.relative_to(root).as_posix(),
                    "mode": mode_string(path),
                    "size_bytes": path.stat().st_size,
                    "sha256": sha256_file(path),
                }
            )
    return rows


def write_sha256sums(root: pathlib.Path) -> None:
    rows = [row for row in inventory(root) if row["path"] != "SHA256SUMS"]
    write_new_bytes(
        root / "SHA256SUMS",
        "".join(f"{row['sha256']}  {row['path']}\n" for row in rows).encode(),
        0o444,
    )


def verify_sha256sums(root: pathlib.Path) -> int:
    manifest = root / "SHA256SUMS"
    require(manifest.is_file(), f"missing SHA256SUMS: {root}")
    seen: set[str] = set()
    for line in manifest.read_text(encoding="utf-8").splitlines():
        digest, separator, relative = line.partition("  ")
        require(separator == "  " and len(digest) == 64, f"invalid manifest row: {line}")
        path = pathlib.PurePosixPath(relative)
        require(not path.is_absolute() and ".." not in path.parts, f"unsafe manifest path: {relative}")
        require(relative not in seen and relative != "SHA256SUMS", f"duplicate manifest path: {relative}")
        seen.add(relative)
        require(sha256_file(root / path) == digest, f"manifest mismatch: {root / path}")
    actual = {row["path"] for row in inventory(root) if row["path"] != "SHA256SUMS"}
    require(seen == actual, f"manifest membership mismatch: {root}")
    return len(seen)


def make_tree_writable(root: pathlib.Path) -> None:
    if not root.exists():
        return
    root.chmod(0o700)
    for path in sorted(root.rglob("*")):
        require(not path.is_symlink(), f"symlink is forbidden: {path}")
        path.chmod(0o700 if path.is_dir() else (0o700 if path.stat().st_mode & 0o111 else 0o600))


def remove_owned_tree(root: pathlib.Path) -> None:
    if root.exists():
        make_tree_writable(root)
        shutil.rmtree(root)


def seal_tree(root: pathlib.Path) -> None:
    for path in sorted(root.rglob("*"), reverse=True):
        require(not path.is_symlink(), f"symlink is forbidden: {path}")
        path.chmod(0o555 if path.is_dir() or path.stat().st_mode & 0o111 else 0o444)
    root.chmod(0o555)


def atomic_publish(stage: pathlib.Path, final: pathlib.Path) -> None:
    require(stage.parent == final.parent, "publication must stay on one filesystem")
    require(stage.is_dir() and not final.exists(), "publication precondition failed")
    for path in [stage, *stage.rglob("*")]:
        require(not path.is_symlink(), f"symlink before publication: {path}")
        require(stat.S_IMODE(path.stat().st_mode) & 0o222 == 0, f"writable publication object: {path}")
    fsync_directory(stage)
    os.rename(stage, final)
    fsync_directory(final.parent)


def run(
    command: Sequence[str],
    *,
    cwd: pathlib.Path | None = None,
    env: Mapping[str, str] | None = None,
    check: bool = True,
    timeout: int | None = None,
) -> subprocess.CompletedProcess[bytes]:
    result = subprocess.run(
        list(command),
        cwd=cwd,
        env=dict(env) if env is not None else None,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        check=False,
        timeout=timeout,
    )
    if check and result.returncode != 0:
        raise ControllerError(
            f"command failed ({result.returncode}): {shlex.join(command)}\n"
            + result.stderr.decode(errors="replace")[-4000:]
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


def assemble_source(v10: bytes, fragment: bytes) -> bytes:
    require(len(v10) == EXPECTED["v10_source_size"], "V10 source size mismatch")
    require(sha256_bytes(v10) == EXPECTED["v10_source"], "V10 source SHA mismatch")
    require(v10.endswith(b"}\n"), "V10 terminal brace mismatch")
    require(len(fragment) == EXPECTED["d1_fragment_size"], "D1 fragment size mismatch")
    require(sha256_bytes(fragment) == EXPECTED["d1_fragment"], "D1 fragment SHA mismatch")
    require(fragment.startswith(b"\n    const D1_PARITY_TEST"), "D1 fragment prefix mismatch")
    source = v10[:-2] + fragment + b"}\n"
    require(len(source) == EXPECTED["assembled_source_size"], "assembled source size mismatch")
    require(sha256_bytes(source) == EXPECTED["assembled_source"], "assembled source SHA mismatch")
    prefix = source[: EXPECTED["production_prefix_size"]]
    require(sha256_bytes(prefix) == EXPECTED["production_prefix"], "production prefix SHA mismatch")
    require(prefix == v10[: EXPECTED["production_prefix_size"]], "production prefix bytes mismatch")
    return source


def verify_local_admission() -> dict[str, Any]:
    require(not LOCAL_RESULT.exists(), "local build result already exists")
    require_file(SSH_IDENTITY, mode="0600")
    d2a = require_file(
        LOCAL_D2A_RECEIPT,
        digest=EXPECTED["d2a_receipt"],
        size=EXPECTED["d2a_receipt_size"],
        mode="0444",
    )
    audit = require_file(
        LOCAL_AUDIT_RECEIPT,
        digest=EXPECTED["audit_receipt"],
        size=EXPECTED["audit_receipt_size"],
        mode="0444",
    )
    producer = require_file(
        PRODUCER_CONTROLLER,
        digest=EXPECTED["producer_controller"],
        size=EXPECTED["producer_controller_size"],
        mode="0755",
    )
    auditor = require_file(
        AUDITOR,
        digest=EXPECTED["auditor"],
        size=EXPECTED["auditor_size"],
        mode="0755",
    )
    d2a_value = load_json(LOCAL_D2A_RECEIPT)
    audit_value = load_json(LOCAL_AUDIT_RECEIPT)
    require(d2a_value.get("task_id") == TASK_ID, "D2-A task identity mismatch")
    require(d2a_value.get("transaction_id") == TRANSACTION_ID, "D2-A transaction mismatch")
    require(d2a_value.get("verdict") == "D2A_CLOSED_ALL_MARKERS_AVAILABLE", "D2-A verdict mismatch")
    require(d2a_value.get("markers_created") == 11 and d2a_value.get("markers_consumed") == 0, "D2-A marker ledger mismatch")
    require(d2a_value.get("cargo_invocations") == 0 and d2a_value.get("rustc_compilations") == 0, "D2-A compilation ledger mismatch")
    require(d2a_value.get("perf_record") == 0 and d2a_value.get("perf_stat") == 0, "D2-A perf ledger mismatch")
    require(d2a_value.get("d2_subject") == 0 and d2a_value.get("d2_elf_created") is False, "D2-A subject/ELF ledger mismatch")
    require(audit_value.get("task_id") == TASK_ID, "audit task identity mismatch")
    require(audit_value.get("verdict") == "D2A_AUDIT_PASS_BUILD_ADMISSION", "build is not admitted")
    require(audit_value.get("build_admitted") is True and audit_value.get("build_executed") is False, "audit build boundary mismatch")
    require(audit_value.get("build_marker_state") == "available", "audit marker state mismatch")
    require(audit_value.get("markers_consumed") == 0, "audit observed consumed markers")
    require(audit_value.get("producer_controller", {}).get("sha256") == EXPECTED["producer_controller"], "audit producer identity mismatch")
    require(audit_value.get("authoritative_d2a_receipt", {}).get("sha256") == EXPECTED["d2a_receipt"], "audit D2-A identity mismatch")
    require(verify_sha256sums(D2A_RESULT) > 0, "D2-A local manifest is empty")
    require(verify_sha256sums(AUDIT_RESULT) == 5, "audit V2 manifest membership mismatch")
    return {
        "d2a_receipt": d2a,
        "audit_receipt": audit,
        "producer_controller": producer,
        "auditor": auditor,
        "transaction_id": TRANSACTION_ID,
    }


def command_graph() -> dict[str, Any]:
    return {
        "external_actions": list(EXTERNAL_ACTIONS),
        "build": list(BUILD_COMMAND_TEMPLATE),
        "remote_transport": ["ssh", "python3", "stdin-sealed-controller-and-audit"],
        "cargo_routes": ["build-once"],
        "perf_routes": [],
        "subject_routes": [],
        "scientific_elf_audit_routes": [],
    }


def verify_command_graph() -> dict[str, Any]:
    graph = command_graph()
    require(tuple(graph["external_actions"]) == EXTERNAL_ACTIONS, "external action graph drift")
    require(tuple(graph["build"]) == BUILD_COMMAND_TEMPLATE, "build argv template drift")
    require("--message-format" not in graph["build"], "unfrozen Cargo argument present")
    require(graph["cargo_routes"] == ["build-once"], "Cargo route graph drift")
    require(graph["perf_routes"] == [] and graph["subject_routes"] == [], "forbidden execution route present")
    require(graph["scientific_elf_audit_routes"] == [], "scientific ELF audit route present")
    return graph


def remote_probe_source() -> str:
    constants = "\n".join(
        (
            f"TASK_ID={TASK_ID!r}",
            f"PARENT=pathlib.Path({str(REMOTE_PARENT)!r})",
            f"D2A=pathlib.Path({str(REMOTE_D2A)!r})",
            f"FAILURE=pathlib.Path({str(REMOTE_D2A_FAILURE)!r})",
            f"BUILD=pathlib.Path({str(REMOTE_BUILD)!r})",
            f"BUILD_FAILURE=pathlib.Path({str(REMOTE_BUILD_FAILURE)!r})",
            f"STATE=pathlib.Path({str(REMOTE_STATE)!r})",
        )
    )
    body = r'''
import hashlib,json,os,stat
def sha(path):
    digest=hashlib.sha256()
    with path.open('rb') as source:
        for block in iter(lambda:source.read(1024*1024),b''):
            digest.update(block)
    return digest.hexdigest()
def row(path):
    return {'path':str(path),'mode':f'{stat.S_IMODE(path.stat().st_mode):04o}',
            'size_bytes':path.stat().st_size,'sha256':sha(path)}
markers=STATE/'markers'
marker_rows=[]
if markers.is_dir():
    for path in sorted(markers.iterdir()):
        marker_rows.append({**row(path),'name':path.name,'value':json.loads(path.read_text())})
elf_files=[]
if PARENT.is_dir():
    for path in PARENT.rglob('*'):
        if path.is_file():
            with path.open('rb') as source:
                if source.read(4)==b'\x7fELF':
                    elf_files.append(str(path))
owned=[]
for proc in pathlib.Path('/proc').glob('[0-9]*'):
    try:
        comm=(proc/'comm').read_text().strip()
        cmd=(proc/'cmdline').read_bytes().replace(b'\0',b' ').decode(errors='replace')
    except (FileNotFoundError,PermissionError,ProcessLookupError):
        continue
    if comm in {'cargo','rustc','perf'} and (TASK_ID in cmd or 'lay-d2-primary-only-build' in cmd):
        owned.append({'pid':int(proc.name),'comm':comm,'cmdline':cmd})
result={
    'hostname':os.uname().nodename,
    'machine_id_sha256':sha(pathlib.Path('/etc/machine-id')),
    'parent_entries':sorted(path.name for path in PARENT.iterdir()) if PARENT.is_dir() else [],
    'd2a':row(D2A/'D2A_RECEIPT.json') if (D2A/'D2A_RECEIPT.json').is_file() else None,
    'd2a_failure_exists':FAILURE.exists(),
    'build_exists':BUILD.exists(),
    'build_failure_exists':BUILD_FAILURE.exists(),
    'state_entries':sorted(path.name for path in STATE.iterdir()) if STATE.is_dir() else [],
    'state':row(STATE/'STATE.json') if (STATE/'STATE.json').is_file() else None,
    'state_value':json.loads((STATE/'STATE.json').read_text()) if (STATE/'STATE.json').is_file() else None,
    'route_lock':row(STATE/'route.lock') if (STATE/'route.lock').is_file() else None,
    'route_lock_value':json.loads((STATE/'route.lock').read_text()) if (STATE/'route.lock').is_file() else None,
    'markers':marker_rows,
    'elf_files':sorted(elf_files),
    'owned_build_processes':owned,
    'build_state':row(STATE/'BUILD_STATE.json') if (STATE/'BUILD_STATE.json').is_file() else None,
    'build_state_value':json.loads((STATE/'BUILD_STATE.json').read_text()) if (STATE/'BUILD_STATE.json').is_file() else None,
}
print(json.dumps(result,sort_keys=True,separators=(',',':')))
'''
    return "import pathlib\n" + constants + "\n" + body


def remote_projection() -> dict[str, Any]:
    source = remote_probe_source()
    compile(source, "<post-d2a-read-only-projection>", "exec")
    result = run(ssh_argv(["/usr/bin/python3", "-c", source]))
    return json.loads(result.stdout)


def expected_marker_identity(name: str) -> tuple[str, int]:
    _, digest, size = MARKERS[name]
    return digest, size


def verify_prebuild_projection(
    value: Mapping[str, Any],
    *,
    allowed_parent_entries: Sequence[str] = ("d2a-v1",),
) -> None:
    require(value.get("hostname") == REMOTE_HOSTNAME, "remote hostname drift")
    require(value.get("machine_id_sha256") == REMOTE_MACHINE_ID_SHA256, "remote machine identity drift")
    require(
        value.get("parent_entries") == sorted(allowed_parent_entries),
        f"remote D2 parent drift: {value.get('parent_entries')}",
    )
    require(value.get("d2a_failure_exists") is False, "D2-A failure tree exists")
    require(value.get("build_exists") is False and value.get("build_failure_exists") is False, "D2 build evidence already exists")
    require(value.get("state_entries") == ["STATE.json", "markers", "route.lock"], "remote state entries drift")
    require(value.get("elf_files") == [], "D2 ELF already exists")
    require(value.get("owned_build_processes") == [], "D2-owned Cargo/rustc/perf process exists")
    require(value.get("build_state") is None, "BUILD_STATE already exists")
    d2a = value.get("d2a") or {}
    require(d2a.get("sha256") == EXPECTED["d2a_receipt"] and d2a.get("size_bytes") == EXPECTED["d2a_receipt_size"], "remote D2-A receipt drift")
    require(d2a.get("mode") == "0444", "remote D2-A receipt mode drift")
    state = value.get("state") or {}
    require(state.get("sha256") == EXPECTED["d2a_state"] and state.get("size_bytes") == EXPECTED["d2a_state_size"], "STATE.json drift")
    require(state.get("mode") == "0400", "STATE.json mode drift")
    state_value = value.get("state_value") or {}
    require(state_value.get("state") == "D2A_CLOSED_ALL_MARKERS_AVAILABLE", "D2-A state verdict drift")
    require(state_value.get("transaction_id") == TRANSACTION_ID, "D2-A state transaction drift")
    lock = value.get("route_lock") or {}
    require(lock.get("sha256") == EXPECTED["route_lock"] and lock.get("size_bytes") == EXPECTED["route_lock_size"], "route.lock drift")
    require(lock.get("mode") == "0400", "route.lock mode drift")
    require((value.get("route_lock_value") or {}).get("transaction_id") == TRANSACTION_ID, "route.lock transaction drift")
    observed = {row["name"]: row for row in value.get("markers", [])}
    require(set(observed) == set(MARKERS), f"marker set drift: {sorted(observed)}")
    for name, (route, digest, size) in MARKERS.items():
        row = observed[name]
        require(row.get("mode") == "0400", f"marker mode drift: {name}")
        require(row.get("sha256") == digest and row.get("size_bytes") == size, f"marker identity drift: {name}")
        marker_value = row.get("value") or {}
        require(marker_value.get("route_id") == route, f"marker route drift: {name}")
        require(marker_value.get("transaction_id") == TRANSACTION_ID, f"marker transaction drift: {name}")


def verify_postbuild_projection(value: Mapping[str, Any], candidate_path: str) -> None:
    require(value.get("hostname") == REMOTE_HOSTNAME, "remote hostname drift after build")
    require(value.get("machine_id_sha256") == REMOTE_MACHINE_ID_SHA256, "remote machine drift after build")
    require(value.get("parent_entries") == ["build-v1", "d2a-v1"], f"postbuild parent drift: {value.get('parent_entries')}")
    require(value.get("d2a_failure_exists") is False and value.get("build_failure_exists") is False, "failure evidence exists after successful build")
    require(value.get("build_exists") is True, "build-v1 is absent")
    require(value.get("state_entries") == ["BUILD_STATE.json", "STATE.json", "markers", "route.lock"], "postbuild state entries drift")
    require(value.get("owned_build_processes") == [], "D2-owned process remains after build")
    require(value.get("elf_files") == [candidate_path], f"unexpected ELF set: {value.get('elf_files')}")
    observed = {row["name"]: row for row in value.get("markers", [])}
    expected_names = (set(MARKERS) - {"build.available"}) | {"build.consumed-before-exec"}
    require(set(observed) == expected_names, f"postbuild marker set drift: {sorted(observed)}")
    build_row = observed["build.consumed-before-exec"]
    digest, size = expected_marker_identity("build.available")
    require(build_row.get("sha256") == digest and build_row.get("size_bytes") == size, "consumed build marker identity drift")
    require(build_row.get("mode") == "0400", "consumed build marker mode drift")
    for name, (_, marker_digest, marker_size) in MARKERS.items():
        if name == "build.available":
            continue
        row = observed[name]
        require(row.get("sha256") == marker_digest and row.get("size_bytes") == marker_size, f"non-build marker drift: {name}")
        require(row.get("mode") == "0400", f"non-build marker mode drift: {name}")
    build_state = value.get("build_state_value") or {}
    require(build_state.get("state") == "D2_BUILD_CREATED_UNAUDITED", "BUILD_STATE verdict mismatch")
    require(build_state.get("transaction_id") == TRANSACTION_ID, "BUILD_STATE transaction mismatch")
    require(build_state.get("build_marker_consumed") is True, "BUILD_STATE marker ledger mismatch")
    require(build_state.get("other_markers_consumed") == 0, "BUILD_STATE other marker ledger mismatch")


def controlled_environment(target: pathlib.Path) -> dict[str, str]:
    value = {
        "HOME": "/home/e",
        "LANG": "C.UTF-8",
        "LC_ALL": "C.UTF-8",
        "LOGNAME": "e",
        "PATH": "/home/e/.cargo/bin:/usr/local/bin:/usr/bin:/bin",
        "TZ": "Europe/Tallinn",
        "USER": "e",
        **FROZEN_BUILD_ENVIRONMENT,
        "CARGO_TARGET_DIR": str(target),
    }
    return value


def build_command(workspace: pathlib.Path) -> list[str]:
    return [str(workspace / "scripts/cargo-guard.sh"), *BUILD_COMMAND_TEMPLATE[1:]]


def verify_remote_inputs() -> dict[str, Any]:
    require_file(
        REMOTE_B0A / "SHA256SUMS",
        digest=EXPECTED["b0a_manifest"],
        mode="0444",
    )
    require_file(
        REMOTE_B0A / "INPUT_CLOSURE.json",
        digest=EXPECTED["b0a_receipt"],
        mode="0444",
    )
    require(verify_sha256sums(REMOTE_B0A) > 0, "B0a manifest is empty")
    require(REMOTE_SOURCE_CLOSURE.is_dir(), "surviving source closure is absent")
    source_inventory = inventory(REMOTE_SOURCE_CLOSURE)
    require(bool(source_inventory), "surviving source closure is empty")
    rows = {
        "v10": require_file(REMOTE_V10, digest=EXPECTED["v10_source"], size=EXPECTED["v10_source_size"], mode="0444"),
        "fragment": require_file(REMOTE_FRAGMENT, digest=EXPECTED["d1_fragment"], size=EXPECTED["d1_fragment_size"], mode="0444"),
        "cargo_toml": require_file(REMOTE_CARGO_TOML, digest=EXPECTED["cargo_toml"], size=EXPECTED["cargo_toml_size"], mode="0444"),
        "cargo_lock": require_file(REMOTE_CARGO_LOCK, digest=EXPECTED["cargo_lock"], size=EXPECTED["cargo_lock_size"], mode="0444"),
        "cargo_guard": require_file(REMOTE_CARGO_GUARD, digest=EXPECTED["cargo_guard"], size=EXPECTED["cargo_guard_size"], mode="0555"),
        "d2a_receipt": require_file(REMOTE_D2A_RECEIPT, digest=EXPECTED["d2a_receipt"], size=EXPECTED["d2a_receipt_size"], mode="0444"),
    }
    rows["source_closure_inventory_sha256"] = sha256_bytes(canonical_json_bytes(source_inventory))
    rows["source_closure_inventory_entries"] = len(source_inventory)
    return rows


def verify_workspace(workspace: pathlib.Path, assembled: bytes) -> dict[str, Any]:
    source_path = workspace / "src/nanda_wave/l2_field/v13_typed_peak.rs"
    rows = {
        "assembled_source": require_file(source_path, digest=EXPECTED["assembled_source"], size=EXPECTED["assembled_source_size"], mode="0444"),
        "cargo_toml": require_file(workspace / "Cargo.toml", digest=EXPECTED["cargo_toml"], size=EXPECTED["cargo_toml_size"]),
        "cargo_lock": require_file(workspace / "Cargo.lock", digest=EXPECTED["cargo_lock"], size=EXPECTED["cargo_lock_size"]),
        "cargo_guard": require_file(workspace / "scripts/cargo-guard.sh", digest=EXPECTED["cargo_guard"], size=EXPECTED["cargo_guard_size"], mode="0755"),
    }
    require(source_path.read_bytes() == assembled, "workspace source bytes drift")
    rows["inventory"] = inventory(workspace)
    rows["inventory_sha256"] = sha256_bytes(canonical_json_bytes(rows["inventory"]))
    return rows


@contextlib.contextmanager
def route_lock() -> Iterable[None]:
    lock = REMOTE_STATE / "route.lock"
    require_file(lock, digest=EXPECTED["route_lock"], size=EXPECTED["route_lock_size"], mode="0400")
    descriptor = os.open(lock, os.O_RDONLY)
    try:
        try:
            fcntl.flock(descriptor, fcntl.LOCK_EX | fcntl.LOCK_NB)
        except BlockingIOError as error:
            raise ControllerError("another D2 route owner holds route.lock") from error
        yield
    finally:
        with contextlib.suppress(OSError):
            fcntl.flock(descriptor, fcntl.LOCK_UN)
        os.close(descriptor)


def consume_build_marker() -> pathlib.Path:
    available = REMOTE_MARKERS / "build.available"
    consumed = REMOTE_MARKERS / "build.consumed-before-exec"
    digest, size = expected_marker_identity("build.available")
    require_file(available, digest=digest, size=size, mode="0400")
    require(not consumed.exists(), "build marker was already consumed")
    os.rename(available, consumed)
    fsync_directory(REMOTE_MARKERS)
    require_file(consumed, digest=digest, size=size, mode="0400")
    return consumed


def find_test_executable(target: pathlib.Path) -> pathlib.Path:
    candidates = []
    for path in (target / "release/deps").glob("lay-*"):
        if path.is_file() and os.access(path, os.X_OK) and not path.name.endswith((".d", ".rlib", ".rmeta")):
            with path.open("rb") as source:
                if source.read(4) == b"\x7fELF":
                    candidates.append(path)
    require(len(candidates) == 1, f"expected exactly one fresh test ELF, found {candidates}")
    return candidates[0]


def build_state_value(verdict: str, controller_digest: str, candidate: Mapping[str, Any] | None) -> dict[str, Any]:
    return {
        "schema": "lay.v10.e1-traversal-d2-primary-only-build-state.v1",
        "task_id": TASK_ID,
        "transaction_id": TRANSACTION_ID,
        "state": verdict,
        "post_d2a_controller_sha256": controller_digest,
        "build_marker_consumed": True,
        "other_markers_consumed": 0,
        "cargo_invocations": 1,
        "candidate_elf": candidate,
        "retry_permitted": False,
    }


def remote_build_once() -> dict[str, Any]:
    require(REMOTE_EXECUTION, "remote build is not a local action")
    source_bytes = controller_source()
    controller_digest = sha256_bytes(source_bytes)
    require(sha256_bytes(AUDIT_RECEIPT_BYTES) == EXPECTED["audit_receipt"], "sealed audit payload mismatch")
    audit_value = json.loads(AUDIT_RECEIPT_BYTES)
    require(audit_value.get("verdict") == "D2A_AUDIT_PASS_BUILD_ADMISSION", "sealed audit did not admit build")
    require(audit_value.get("producer_controller", {}).get("sha256") == EXPECTED["producer_controller"], "sealed audit producer drift")
    require(audit_value.get("authoritative_d2a_receipt", {}).get("sha256") == EXPECTED["d2a_receipt"], "sealed audit D2-A drift")
    require(os.uname().nodename == REMOTE_HOSTNAME, "remote hostname drift")
    require(sha256_file(pathlib.Path("/etc/machine-id")) == REMOTE_MACHINE_ID_SHA256, "remote machine identity drift")

    stage = REMOTE_PARENT / f".build-v1.stage-{os.getpid()}-{time.time_ns()}"
    workspace = REMOTE_PARENT / f".lay-d2-primary-only-build-{os.getpid()}-{time.time_ns()}"
    marker_consumed = False
    cargo_started = False
    cargo_exit: int | None = None
    candidate_row: dict[str, Any] | None = None
    state_stage = REMOTE_STATE / f".BUILD_STATE.stage-{os.getpid()}-{time.time_ns()}"
    with route_lock():
        try:
            projection_before = remote_state_projection_direct()
            verify_prebuild_projection(projection_before)
            require(shutil.disk_usage(REMOTE_PARENT).free >= MIN_FREE_BYTES, "insufficient build disk headroom")
            inputs = verify_remote_inputs()
            assembled = assemble_source(REMOTE_V10.read_bytes(), REMOTE_FRAGMENT.read_bytes())
            stage.mkdir(mode=0o700)
            workspace.mkdir(mode=0o700)
            shutil.copytree(REMOTE_SOURCE_CLOSURE, workspace, dirs_exist_ok=True)
            make_tree_writable(workspace)
            shutil.copyfile(REMOTE_CARGO_TOML, workspace / "Cargo.toml")
            shutil.copyfile(REMOTE_CARGO_LOCK, workspace / "Cargo.lock")
            (workspace / "scripts").mkdir(exist_ok=True)
            shutil.copyfile(REMOTE_CARGO_GUARD, workspace / "scripts/cargo-guard.sh")
            (workspace / "scripts/cargo-guard.sh").chmod(0o755)
            source_path = workspace / "src/nanda_wave/l2_field/v13_typed_peak.rs"
            source_path.parent.mkdir(parents=True, exist_ok=True)
            source_path.write_bytes(assembled)
            source_path.chmod(0o444)
            workspace_value = verify_workspace(workspace, assembled)

            inputs_dir = stage / "inputs"
            inputs_dir.mkdir(mode=0o700)
            write_new_bytes(inputs_dir / "post-d2a-controller.py", source_bytes, 0o444)
            write_new_bytes(inputs_dir / "D2A_RECEIPT.json", REMOTE_D2A_RECEIPT.read_bytes(), 0o444)
            write_new_bytes(inputs_dir / "D2A_INDEPENDENT_AUDIT_RECEIPT.json", AUDIT_RECEIPT_BYTES, 0o444)
            write_new_bytes(stage / "assembled_d2_source.rs", assembled, 0o444)
            shutil.copytree(workspace, stage / "source")
            write_new_json(stage / "WORKSPACE_PREBUILD_INVENTORY.json", workspace_value["inventory"], 0o444)
            command = build_command(workspace)
            environment = controlled_environment(workspace / "target")
            prebuild = {
                "schema": "lay.v10.e1-traversal-d2-primary-only-prebuild.v1",
                "task_id": TASK_ID,
                "transaction_id": TRANSACTION_ID,
                "prepared_at": now(),
                "post_d2a_controller_sha256": controller_digest,
                "d2a_receipt_sha256": EXPECTED["d2a_receipt"],
                "independent_audit_sha256": EXPECTED["audit_receipt"],
                "producer_controller_sha256": EXPECTED["producer_controller"],
                "build_available_pre_consumption": next(
                    row for row in projection_before["markers"] if row["name"] == "build.available"
                ),
                "assembled_source": {"sha256": sha256_bytes(assembled), "size_bytes": len(assembled)},
                "production_prefix": {"sha256": sha256_bytes(assembled[: EXPECTED["production_prefix_size"]]), "size_bytes": EXPECTED["production_prefix_size"]},
                "cargo_toml": workspace_value["cargo_toml"],
                "cargo_lock": workspace_value["cargo_lock"],
                "cargo_guard": workspace_value["cargo_guard"],
                "workspace_path": str(workspace),
                "target_path": str(workspace / "target"),
                "workspace_inventory_sha256": workspace_value["inventory_sha256"],
                "workspace_inventory_entries": len(workspace_value["inventory"]),
                "remote_inputs": inputs,
                "cargo_argv": command,
                "build_environment": environment,
                "cargo_started": False,
                "retry_permitted": False,
            }
            write_new_json(stage / "PREBUILD.json", prebuild, 0o444)
            fsync_tree(stage)

            verify_prebuild_projection(
                remote_state_projection_direct(),
                allowed_parent_entries=("d2a-v1", stage.name, workspace.name),
            )
            require(verify_workspace(workspace, assembled)["inventory_sha256"] == workspace_value["inventory_sha256"], "workspace drift immediately before marker consumption")
            require(sha256_file(stage / "PREBUILD.json") == sha256_bytes((stage / "PREBUILD.json").read_bytes()), "PREBUILD readback mismatch")
            consumed = consume_build_marker()
            marker_consumed = True
            write_new_json(
                stage / "MARKER_CONSUMPTION.json",
                {
                    "schema": "lay.v10.e1-traversal-d2-primary-only-marker-consumption.v1",
                    "task_id": TASK_ID,
                    "transaction_id": TRANSACTION_ID,
                    "marker": file_identity(consumed),
                    "build_marker_consumed_before_cargo": True,
                    "cargo_started": False,
                    "retry_permitted": False,
                },
                0o444,
            )
            fsync_tree(stage)

            stdout_path = stage / "cargo.stdout.log"
            stderr_path = stage / "cargo.stderr.log"
            with stdout_path.open("xb") as stdout, stderr_path.open("xb") as stderr:
                cargo_started = True
                process = subprocess.Popen(
                    command,
                    cwd=workspace,
                    env=environment,
                    stdout=stdout,
                    stderr=stderr,
                    start_new_session=True,
                )
                try:
                    cargo_exit = process.wait(timeout=3600)
                except BaseException:
                    with contextlib.suppress(ProcessLookupError):
                        os.killpg(process.pid, signal.SIGTERM)
                    with contextlib.suppress(subprocess.TimeoutExpired):
                        process.wait(timeout=15)
                    raise
                stdout.flush()
                os.fsync(stdout.fileno())
                stderr.flush()
                os.fsync(stderr.fileno())
            write_new_json(
                stage / "CARGO_RESULT.json",
                {
                    "schema": "lay.v10.e1-traversal-d2-primary-only-cargo-result.v1",
                    "command": command,
                    "environment": environment,
                    "exit_code": cargo_exit,
                    "cargo_invocations": 1,
                    "build_marker_consumed_before_cargo": True,
                },
                0o444,
            )
            require(cargo_exit == 0, f"guarded D2 build exited {cargo_exit}")
            executable = find_test_executable(workspace / "target")
            candidate = stage / "d2-test-elf"
            shutil.copyfile(executable, candidate)
            candidate.chmod(0o555)
            with candidate.open("rb") as source:
                require(source.read(4) == b"\x7fELF", "candidate ELF magic mismatch")
            candidate_row = {
                "path": str(REMOTE_BUILD / "d2-test-elf"),
                "mode": "0555",
                "size_bytes": candidate.stat().st_size,
                "sha256": sha256_file(candidate),
                "scientific_audit": "NOT_PERFORMED",
            }
            remove_owned_tree(workspace)
            build_state = build_state_value("D2_BUILD_CREATED_UNAUDITED", controller_digest, candidate_row)
            receipt = {
                "schema": "lay.v10.e1-traversal-d2-primary-only-build.v1",
                "task_id": TASK_ID,
                "transaction_id": TRANSACTION_ID,
                "verdict": "D2_BUILD_CREATED_UNAUDITED",
                "post_d2a_controller_sha256": controller_digest,
                "d2a_receipt_sha256": EXPECTED["d2a_receipt"],
                "independent_audit_sha256": EXPECTED["audit_receipt"],
                "producer_controller_sha256": EXPECTED["producer_controller"],
                "build_marker_consumed": True,
                "build_marker_consumed_before_cargo": True,
                "other_markers_consumed": 0,
                "cargo_invocations": 1,
                "cargo_exit": cargo_exit,
                "rustc_compilations": "build-owned-only",
                "candidate_elf": candidate_row,
                "elf_executed": False,
                "elf_scientific_audit": False,
                "bucket_map_created": False,
                "parity_executed": False,
                "u_routes_executed": 0,
                "v_routes_executed": 0,
                "t_routes_executed": 0,
                "perf_record": 0,
                "perf_stat": 0,
                "pmu_events_opened": 0,
                "d2_subject": 0,
                "runtime_authority_changed": False,
                "workspace_retained": False,
                "workspace_source_snapshot_retained": True,
                "retry_permitted": False,
                "next_action_admitted": "independent read-only D2 build audit only",
            }
            write_new_json(stage / "BUILD_STATE.json", build_state, 0o444)
            write_new_json(stage / "D2_BUILD_RECEIPT.json", receipt, 0o444)
            write_sha256sums(stage)
            seal_tree(stage)
            atomic_publish(stage, REMOTE_BUILD)
            write_new_json(state_stage, build_state, 0o400)
            os.rename(state_stage, REMOTE_BUILD_STATE)
            fsync_directory(REMOTE_STATE)
            return receipt
        except BaseException as error:
            if marker_consumed:
                with contextlib.suppress(Exception):
                    if workspace.exists():
                        workspace_row = {
                            "path": str(workspace),
                            "inventory_sha256": sha256_bytes(canonical_json_bytes(inventory(workspace))),
                        }
                    else:
                        workspace_row = {"path": str(workspace), "exists": False}
                    if not stage.exists():
                        stage.mkdir(mode=0o700)
                    if not (stage / "inputs").exists():
                        (stage / "inputs").mkdir(mode=0o700)
                    if not (stage / "inputs/post-d2a-controller.py").exists():
                        write_new_bytes(stage / "inputs/post-d2a-controller.py", source_bytes, 0o444)
                    failure = {
                        "schema": "lay.v10.e1-traversal-d2-primary-only-build-failure.v1",
                        "task_id": TASK_ID,
                        "transaction_id": TRANSACTION_ID,
                        "verdict": "BLOCKED_BUILD",
                        "error": str(error),
                        "post_d2a_controller_sha256": controller_digest,
                        "build_marker_consumed": True,
                        "cargo_started": cargo_started,
                        "cargo_invocations": 1 if cargo_started else 0,
                        "cargo_exit": cargo_exit,
                        "candidate_elf": candidate_row,
                        "workspace_identity": workspace_row,
                        "retry_permitted": False,
                        "perf_record": 0,
                        "perf_stat": 0,
                        "pmu_events_opened": 0,
                        "d2_subject": 0,
                        "runtime_authority_changed": False,
                    }
                    write_new_json(stage / "D2_BUILD_FAILURE.json", failure, 0o444)
                    write_sha256sums(stage)
                    seal_tree(stage)
                    atomic_publish(stage, REMOTE_BUILD_FAILURE)
                    if not REMOTE_BUILD_STATE.exists() and not state_stage.exists():
                        write_new_json(
                            state_stage,
                            build_state_value("BLOCKED_BUILD", controller_digest, candidate_row),
                            0o400,
                        )
                        os.rename(state_stage, REMOTE_BUILD_STATE)
                        fsync_directory(REMOTE_STATE)
            else:
                remove_owned_tree(stage)
                remove_owned_tree(workspace)
            raise


def remote_state_projection_direct() -> dict[str, Any]:
    markers = []
    if REMOTE_MARKERS.is_dir():
        for path in sorted(REMOTE_MARKERS.iterdir()):
            row = file_identity(path)
            row.update({"name": path.name, "value": load_json(path)})
            markers.append(row)
    elf_files = []
    if REMOTE_PARENT.is_dir():
        for path in REMOTE_PARENT.rglob("*"):
            if path.is_file():
                with path.open("rb") as source:
                    if source.read(4) == b"\x7fELF":
                        elf_files.append(str(path))
    owned = []
    for proc in pathlib.Path("/proc").glob("[0-9]*"):
        try:
            comm = (proc / "comm").read_text().strip()
            command = (proc / "cmdline").read_bytes().replace(b"\0", b" ").decode(errors="replace")
        except (FileNotFoundError, PermissionError, ProcessLookupError):
            continue
        if comm in {"cargo", "rustc", "perf"} and (TASK_ID in command or "lay-d2-primary-only-build" in command):
            owned.append({"pid": int(proc.name), "comm": comm, "cmdline": command})
    return {
        "hostname": os.uname().nodename,
        "machine_id_sha256": sha256_file(pathlib.Path("/etc/machine-id")),
        "parent_entries": sorted(path.name for path in REMOTE_PARENT.iterdir()) if REMOTE_PARENT.is_dir() else [],
        "d2a": file_identity(REMOTE_D2A_RECEIPT) if REMOTE_D2A_RECEIPT.is_file() else None,
        "d2a_failure_exists": REMOTE_D2A_FAILURE.exists(),
        "build_exists": REMOTE_BUILD.exists(),
        "build_failure_exists": REMOTE_BUILD_FAILURE.exists(),
        "state_entries": sorted(path.name for path in REMOTE_STATE.iterdir()) if REMOTE_STATE.is_dir() else [],
        "state": file_identity(REMOTE_STATE / "STATE.json") if (REMOTE_STATE / "STATE.json").is_file() else None,
        "state_value": load_json(REMOTE_STATE / "STATE.json") if (REMOTE_STATE / "STATE.json").is_file() else None,
        "route_lock": file_identity(REMOTE_STATE / "route.lock") if (REMOTE_STATE / "route.lock").is_file() else None,
        "route_lock_value": load_json(REMOTE_STATE / "route.lock") if (REMOTE_STATE / "route.lock").is_file() else None,
        "markers": markers,
        "elf_files": sorted(elf_files),
        "owned_build_processes": owned,
        "build_state": file_identity(REMOTE_BUILD_STATE) if REMOTE_BUILD_STATE.is_file() else None,
        "build_state_value": load_json(REMOTE_BUILD_STATE) if REMOTE_BUILD_STATE.is_file() else None,
    }


REMOTE_BOOTSTRAP = (
    "import base64,hashlib,json,sys\n"
    "envelope=json.loads(sys.stdin.buffer.read())\n"
    "source=base64.b64decode(envelope['controller'],validate=True)\n"
    "audit=base64.b64decode(envelope['audit_receipt'],validate=True)\n"
    "assert hashlib.sha256(source).hexdigest()==sys.argv[1], 'controller SHA mismatch'\n"
    "assert hashlib.sha256(audit).hexdigest()==sys.argv[2], 'audit SHA mismatch'\n"
    "sys.argv=['lay-v10-e1-traversal-d2-post-d2a-build.py','build-once']\n"
    "ns={'__name__':'__main__','__file__':'<lay-v10-e1-traversal-d2-post-d2a-build.py>',"
    "'REMOTE_EXECUTION':True,'CONTROLLER_SOURCE_BYTES':source,'AUDIT_RECEIPT_BYTES':audit}\n"
    "exec(compile(source,ns['__file__'],'exec'),ns)\n"
)


def remote_build_call(audit_receipt: bytes) -> subprocess.CompletedProcess[bytes]:
    source = controller_source()
    envelope = canonical_json_bytes(
        {
            "controller": base64.b64encode(source).decode(),
            "audit_receipt": base64.b64encode(audit_receipt).decode(),
        }
    )
    command = [
        "/usr/bin/python3",
        "-c",
        REMOTE_BOOTSTRAP,
        sha256_bytes(source),
        sha256_bytes(audit_receipt),
    ]
    return subprocess.run(
        ssh_argv(command),
        input=envelope,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        timeout=3700,
        check=False,
    )


def local_runtime_snapshot() -> dict[str, Any]:
    def pids(name: str) -> list[int]:
        result = run(["pgrep", "-x", name], check=False)
        return sorted(int(value) for value in result.stdout.split())

    launcher = pathlib.Path.home() / ".local/bin/lay"
    resolved = launcher.resolve(strict=True)
    return {
        "launcher": str(launcher),
        "resolved": str(resolved),
        "resolved_sha256": sha256_file(resolved),
        "ibus_daemon_pids": pids("ibus-daemon"),
        "lay_daemon_pids": pids("lay-daemon"),
        "lay_ibus_engine_pids": pids("lay-ibus-engine"),
    }


def self_check() -> dict[str, Any]:
    require(not REMOTE_EXECUTION, "self-check is local-only")
    compile(CONTROLLER.read_text(encoding="utf-8"), str(CONTROLLER), "exec")
    admission = verify_local_admission()
    graph = verify_command_graph()
    source = controller_source().decode("utf-8")
    forbidden_elf_readers = ("/usr/bin/" + "readelf", "obj" + "dump")
    require(not any(token in source for token in forbidden_elf_readers), "scientific ELF reader is reachable")
    require("--pid" not in graph["build"], "attach route present")
    projection = remote_projection()
    verify_prebuild_projection(projection)
    return {
        "schema": "lay.v10.e1-traversal-d2-primary-only-post-d2a-build-self-check.v1",
        "task_id": TASK_ID,
        "transaction_id": TRANSACTION_ID,
        "verdict": "POST_D2A_BUILD_CONTROLLER_VERIFIED_UNRUN",
        "controller": file_identity(CONTROLLER),
        "admission": admission,
        "command_graph": graph,
        "build_environment": FROZEN_BUILD_ENVIRONMENT,
        "live_remote_projection_sha256": sha256_bytes(canonical_json_bytes(projection)),
        "build_available": True,
        "markers_consumed": 0,
        "cargo_invocations": 0,
        "rustc_compilations": 0,
        "perf_record": 0,
        "perf_stat": 0,
        "pmu_events_opened": 0,
        "d2_subject": 0,
        "d2_elf_created": False,
        "remote_writes": 0,
    }


def copy_remote_evidence(destination: pathlib.Path) -> None:
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
            f"{REMOTE}:{REMOTE_BUILD}",
            str(destination),
        ],
        check=False,
    )
    require(result.returncode == 0, result.stderr.decode(errors="replace")[-4000:])


def local_build_once() -> dict[str, Any]:
    require(not REMOTE_EXECUTION, "local build action entered remote mode")
    check = self_check()
    before = remote_projection()
    verify_prebuild_projection(before)
    runtime_before = local_runtime_snapshot()
    audit_receipt = LOCAL_AUDIT_RECEIPT.read_bytes()
    result = remote_build_call(audit_receipt)
    require(result.returncode == 0, f"remote build failed ({result.returncode}):\n{result.stderr.decode(errors='replace')[-12000:]}")
    output_lines = result.stdout.decode(errors="replace").strip().splitlines()
    require(output_lines, "remote build returned no receipt")
    remote_receipt = json.loads(output_lines[-1])
    require(remote_receipt.get("verdict") == "D2_BUILD_CREATED_UNAUDITED", "remote build verdict mismatch")
    candidate_path = remote_receipt.get("candidate_elf", {}).get("path")
    require(candidate_path == str(REMOTE_BUILD / "d2-test-elf"), "remote candidate path mismatch")
    after = remote_projection()
    verify_postbuild_projection(after, candidate_path)
    runtime_after = local_runtime_snapshot()
    require(runtime_before == runtime_after, "installed runtime projection changed during remote build")

    stage = pathlib.Path(f"{LOCAL_RESULT}.stage-{os.getpid()}-{time.time_ns()}")
    stage.mkdir(parents=True, mode=0o700)
    try:
        remote_evidence = stage / "REMOTE_EVIDENCE"
        copy_remote_evidence(remote_evidence)
        entries = verify_sha256sums(remote_evidence)
        copied_receipt = load_json(remote_evidence / "D2_BUILD_RECEIPT.json")
        require(copied_receipt == remote_receipt, "copied build receipt differs from remote output")
        candidate = remote_evidence / "d2-test-elf"
        require_file(
            candidate,
            digest=remote_receipt["candidate_elf"]["sha256"],
            size=remote_receipt["candidate_elf"]["size_bytes"],
            mode="0555",
        )
        with candidate.open("rb") as source:
            require(source.read(4) == b"\x7fELF", "copied candidate ELF magic mismatch")
        write_new_bytes(stage / "post-d2a-controller.py", controller_source(), 0o444)
        write_new_bytes(stage / "D2A_RECEIPT.json", LOCAL_D2A_RECEIPT.read_bytes(), 0o444)
        write_new_bytes(stage / "D2A_INDEPENDENT_AUDIT_RECEIPT.json", audit_receipt, 0o444)
        shutil.copy2(remote_evidence / "D2_BUILD_RECEIPT.json", stage / "D2_BUILD_RECEIPT.json")
        write_new_json(stage / "LIVE_REMOTE_BEFORE.json", before, 0o444)
        write_new_json(stage / "LIVE_REMOTE_AFTER.json", after, 0o444)
        local_receipt = {
            "schema": "lay.v10.e1-traversal-d2-primary-only-local-build.v1",
            "task_id": TASK_ID,
            "transaction_id": TRANSACTION_ID,
            "verdict": "D2_BUILD_CREATED_UNAUDITED",
            "post_d2a_controller": file_identity(stage / "post-d2a-controller.py"),
            "d2a_receipt_sha256": EXPECTED["d2a_receipt"],
            "independent_audit_sha256": EXPECTED["audit_receipt"],
            "remote_manifest_entries": entries,
            "remote_build_receipt_sha256": sha256_file(remote_evidence / "D2_BUILD_RECEIPT.json"),
            "candidate_elf": remote_receipt["candidate_elf"],
            "build_marker_consumed": True,
            "other_markers_consumed": 0,
            "cargo_invocations": 1,
            "rustc_compilations": "build-owned-only",
            "elf_executed": False,
            "elf_scientific_audit": False,
            "bucket_map_created": False,
            "parity_executed": False,
            "u_routes_executed": 0,
            "v_routes_executed": 0,
            "t_routes_executed": 0,
            "perf_record": 0,
            "perf_stat": 0,
            "pmu_events_opened": 0,
            "d2_subject": 0,
            "runtime_before": runtime_before,
            "runtime_after": runtime_after,
            "runtime_authority_changed": False,
            "next_action_admitted": "independent read-only D2 build audit only",
            "stop": True,
        }
        write_new_json(stage / "LOCAL_BUILD_RECEIPT.json", local_receipt, 0o444)
        write_sha256sums(stage)
        seal_tree(stage)
        atomic_publish(stage, LOCAL_RESULT)
    except BaseException:
        remove_owned_tree(stage)
        raise
    return {
        "verdict": "D2_BUILD_CREATED_UNAUDITED",
        "local_result": str(LOCAL_RESULT),
        "candidate_elf": remote_receipt["candidate_elf"],
        "build_marker_consumed": True,
        "cargo_invocations": 1,
        "elf_executed": False,
        "elf_scientific_audit": False,
        "perf_record": 0,
        "perf_stat": 0,
        "pmu_events_opened": 0,
        "d2_subject": 0,
        "runtime_authority_changed": False,
        "next_action_admitted": "independent read-only D2 build audit only",
    }


def parser() -> argparse.ArgumentParser:
    value = argparse.ArgumentParser()
    value.add_argument("action", choices=EXTERNAL_ACTIONS)
    return value


def main() -> int:
    arguments = parser().parse_args()
    try:
        if REMOTE_EXECUTION:
            require(arguments.action == "build-once", "remote action is not build-once")
            value = remote_build_once()
        elif arguments.action == "self-check":
            value = self_check()
        else:
            value = local_build_once()
        print(json.dumps(value, ensure_ascii=False, sort_keys=True))
        return 0
    except Exception as error:
        print(f"POST-D2A BUILD ERROR: {error}", file=sys.stderr)
        return 1


if __name__ == "__main__":
    raise SystemExit(main())
