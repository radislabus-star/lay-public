#!/usr/bin/env python3
"""Local orchestrator for the one-shot primary-only D2 bucket map."""

from __future__ import annotations

import argparse
import base64
import functools
import hashlib
import json
import os
import pathlib
import re
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
REMOTE_MACHINE_ID_SHA256 = "5ac0bb537745673ef80942c2df32d586f5753c85de1c8b2453aa4e7317b77441"
SSH_IDENTITY = pathlib.Path("/home/ubu/.ssh/mega-mini-admin")

CONTROLLER = pathlib.Path(__file__).resolve()
PROJECT_ROOT = CONTROLLER.parents[1]
REMOTE_CONTROLLER = PROJECT_ROOT / "scripts/lay-v10-e1-traversal-d2-bucket-map-remote.py"
BUILD_AUDIT = PROJECT_ROOT / (
    "docs/structural_gates/receipts/"
    "LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_D2_PRIMARY_ONLY_"
    "BUILD_AUDIT_V1_2026-08-26/D2_BUILD_AUDIT_RECEIPT.json"
)
LOCAL_D2_ELF = PROJECT_ROOT / (
    "docs/structural_gates/receipts/"
    "LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_D2_PRIMARY_ONLY_"
    "BUILD_V1_2026-08-25/REMOTE_EVIDENCE/d2-test-elf"
)
READER_CORRECTION_V3 = PROJECT_ROOT / (
    "docs/structural_gates/"
    "LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_D2_"
    "BUCKET_MAP_READER_SCOPE_CORRECTION_V3_2026-08-26.md"
)
LOCAL_RESULT = PROJECT_ROOT / (
    "docs/structural_gates/receipts/"
    "LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_D2_PRIMARY_ONLY_"
    "BUCKET_MAP_V1_2026-08-26"
)
REMOTE_PARENT = pathlib.Path("/home/e/.local/share/lay/provenance") / TASK_ID
REMOTE_RESULT = REMOTE_PARENT / "bucket-map-v1"
REMOTE_FAILURE = REMOTE_PARENT / "bucket-map-failure-v1"
REMOTE_STATE = pathlib.Path("/home/e/.local/state/lay") / TASK_ID

EXPECTED = {
    "build_audit": "4e19e2e806e2b3f04f8c0286a63f64631e7cf6ff143d9a90da4036654bc8339c",
    "elf": "bb7c68528ec3fec7074919c3c64cce233d48c26efefe9d9b32735c5f5865a178",
    "elf_size": 317_706_232,
    "build_id": "eb951f1a7526a9f1cb365040c10989aa5d3fc50f",
    "text": "f57eba60bc4b1cadbeb2dfc524af59a7ab011a2e64afb0e1a0fe610755129d94",
    "instruction_count": 1_064,
    "address_list": "fca1804a3d0af34ae462938f970fd181706846da6e42dbb725c5ef1775581c58",
    "addr2line_output": "8b9b4767557a3ea019bbaebb280d1a56ab2180f34ad1e05aed9c2affb4c8a9e6",
    "addr2line_output_size": 697_799,
    "reader_correction_v3": "af1761fe71a976939cfaeac5248a1f9cd5f474e85654ca1b5563baf94dcb9214",
    "bucket_marker": "4471cb4edbaa8c7fc7a2f7b2ffeb4894e9b66cc904d81c0e8b1122c506745bb7",
    "build_marker": "d21b10eac837b740fa6cb9e84c75ff76bc5ec9dc388f0896575d6f697e0c2964",
}
SEALED_SYMBOL_INTERVALS = (
    ("hot", 0x778320, 0x7793AE),
    ("edge", 0x926520, 0x926643),
    ("state", 0x9266B0, 0x926808),
)
LOCAL_OBJDUMP_ARGVS = tuple(
    (
        "/usr/bin/objdump",
        "--disassemble",
        "--demangle",
        "--wide",
        f"--start-address={start:#x}",
        f"--stop-address={end:#x}",
        str(LOCAL_D2_ELF),
    )
    for _key, start, end in SEALED_SYMBOL_INTERVALS
)
LOCAL_ADDR2LINE_PREFIX = (
    "/usr/bin/addr2line",
    f"--exe={LOCAL_D2_ELF}",
    "--functions",
    "--inlines",
    "--demangle",
    "--addresses",
)
EXPECTED_MARKERS = {
    "bucket-map.available": (EXPECTED["bucket_marker"], 483),
    "build.consumed-before-exec": (EXPECTED["build_marker"], 478),
    "parity.available": ("ef5eef0d5ae91bea7bff2c1077cceb8c77d2f2d5a0e7263a70f044c648115c55", 479),
    "t-fixed.available": ("7915c483243c7116f2d023895948667cda3708f23afa7bcc7abed614772f49b0", 480),
    "t-reversed.available": ("26eecf8ae977c0428a5371c30d85277e5974e2ec5768629ac95212cd8cd20c9e", 483),
    "t-single.available": ("8f9e716a687622cd04f693350371228072c1a303d65834c6b647fd900322fe7b", 481),
    "u-fixed.available": ("58435bf78041efe8a24191551c48062a9a0617d9ac9d468b918138b268ed27a8", 480),
    "u-reversed.available": ("c13f9e22ead22c1f3afe231848a49673c93690c88ba5bbaa1426e1f46507fe0e", 483),
    "u-single.available": ("bb7b16f685e7c1a8818bc7185f0d6991f83183d8f035e90688fc66d83ba2a46b", 481),
    "v-fixed-instr.available": ("760e09bb85418e31732fcf24f93e8e81d683ecaded94e4afd29bc5d44c1c2f82", 486),
    "v-reversed-instr.available": ("a87b98f363b0c51f1a36896d1892cf0c508997ab91f05980480770c6601583dc", 489),
}
EXTERNAL_ACTIONS = ("self-check", "bucket-map-once")


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
    return (json.dumps(value, sort_keys=True, separators=(",", ":")) + "\n").encode()


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
    write_new_bytes(path, json.dumps(value, sort_keys=True, indent=2).encode() + b"\n", mode)


def fsync_directory(path: pathlib.Path) -> None:
    descriptor = os.open(path, os.O_RDONLY | os.O_DIRECTORY)
    try:
        os.fsync(descriptor)
    finally:
        os.close(descriptor)


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
    rows = [value for value in inventory(root) if value["path"] != "SHA256SUMS"]
    write_new_bytes(
        root / "SHA256SUMS",
        "".join(f"{value['sha256']}  {value['path']}\n" for value in rows).encode(),
    )


def verify_sha256sums(root: pathlib.Path) -> int:
    manifest = root / "SHA256SUMS"
    require(manifest.is_file(), f"missing SHA256SUMS: {root}")
    seen: set[str] = set()
    for line in manifest.read_text().splitlines():
        digest, separator, relative = line.partition("  ")
        require(separator == "  " and len(digest) == 64, f"bad manifest row: {line}")
        path = pathlib.PurePosixPath(relative)
        require(not path.is_absolute() and ".." not in path.parts, f"unsafe manifest path: {relative}")
        require(relative not in seen and relative != "SHA256SUMS", f"duplicate manifest row: {relative}")
        seen.add(relative)
        require(sha256_file(root / path) == digest, f"manifest mismatch: {relative}")
    actual = {value["path"] for value in inventory(root) if value["path"] != "SHA256SUMS"}
    require(seen == actual, "manifest membership mismatch")
    return len(seen)


def seal_tree(root: pathlib.Path) -> None:
    for path in sorted(root.rglob("*"), reverse=True):
        require(not path.is_symlink(), f"symlink before seal: {path}")
        path.chmod(0o555 if path.is_dir() or path.stat().st_mode & 0o111 else 0o444)
    root.chmod(0o555)


def remove_owned_tree(root: pathlib.Path) -> None:
    if not root.exists():
        return
    root.chmod(0o700)
    for path in sorted(root.rglob("*")):
        require(not path.is_symlink(), f"symlink in owned stage: {path}")
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
            + result.stderr.decode(errors="replace")[-6000:]
        )
    return result


def first_version_line(argv: Sequence[str]) -> str:
    result = run(argv, timeout=30)
    lines = result.stdout.decode(errors="strict").splitlines()
    require(bool(lines), f"version command produced no output: {shlex.join(argv)}")
    return lines[0]


def parse_local_instruction_addresses(outputs: Sequence[bytes]) -> list[int]:
    require(len(outputs) == len(SEALED_SYMBOL_INTERVALS), "local objdump output cardinality drift")
    pattern = re.compile(r"^\s*([0-9a-fA-F]+):\s+((?:[0-9a-fA-F]{2}\s)+)\s*(.*)$")
    addresses: list[int] = []
    for (key, start, end), output in zip(SEALED_SYMBOL_INTERVALS, outputs, strict=True):
        cursor = start
        found = 0
        for line in output.decode(errors="replace").splitlines():
            match = pattern.match(line)
            if match is None:
                continue
            address = int(match.group(1), 16)
            if not start <= address < end:
                continue
            machine = bytes.fromhex(match.group(2))
            require(address == cursor, f"local instruction gap or overlap in {key} at {cursor:#x}")
            require(bool(machine), f"empty local instruction in {key} at {address:#x}")
            addresses.append(address)
            cursor += len(machine)
            found += 1
        require(found > 0, f"no local instructions for {key}")
        require(cursor == end, f"local instruction end drift in {key}: {cursor:#x}")
    require(addresses == sorted(addresses), "local instruction addresses are not strictly ordered")
    require(len(addresses) == len(set(addresses)), "duplicate local instruction address")
    return addresses


def verify_local_addr2line_output(output: bytes, addresses: Sequence[int]) -> None:
    lines = output.decode(errors="strict").splitlines()
    observed: list[int] = []
    index = 0
    while index < len(lines):
        require(re.fullmatch(r"0x[0-9a-f]+", lines[index]) is not None, f"bad local addr2line address row: {lines[index]!r}")
        observed.append(int(lines[index], 16))
        index += 1
        frame_rows = 0
        while index < len(lines) and re.fullmatch(r"0x[0-9a-f]+", lines[index]) is None:
            require(index + 1 < len(lines), "truncated local addr2line frame")
            index += 2
            frame_rows += 1
        require(frame_rows > 0, f"local addr2line returned no frame for {observed[-1]:#x}")
    require(observed == list(addresses), "local addr2line address order or membership drift")


@functools.cache
def prepare_local_addr2line() -> dict[str, Any]:
    elf = require_file(LOCAL_D2_ELF, digest=EXPECTED["elf"], size=EXPECTED["elf_size"], mode="0555")
    objdump_version = first_version_line(["/usr/bin/objdump", "--version"])
    addr2line_version = first_version_line(["/usr/bin/addr2line", "--version"])
    require(objdump_version == "GNU objdump (GNU Binutils for Ubuntu) 2.46", "local objdump version drift")
    require(addr2line_version == "GNU addr2line (GNU Binutils for Ubuntu) 2.46", "local addr2line version drift")

    outputs = [run(argv, timeout=120).stdout for argv in LOCAL_OBJDUMP_ARGVS]
    addresses = parse_local_instruction_addresses(outputs)
    address_bytes = "".join(f"0x{address:x}\n" for address in addresses).encode()
    require(len(addresses) == EXPECTED["instruction_count"], "local instruction count drift")
    require(sha256_bytes(address_bytes) == EXPECTED["address_list"], "local address-list SHA drift")

    addr2line_argv = (*LOCAL_ADDR2LINE_PREFIX, *address_bytes.decode().splitlines())
    result = run(addr2line_argv, timeout=120)
    require(result.stderr == b"", "local addr2line produced stderr")
    require(len(result.stdout) == EXPECTED["addr2line_output_size"], "local addr2line output size drift")
    require(sha256_bytes(result.stdout) == EXPECTED["addr2line_output"], "local addr2line output SHA drift")
    verify_local_addr2line_output(result.stdout, addresses)
    return {
        "schema": "lay.v10.e1-traversal-d2-local-addr2line-evidence.v1",
        "elf": elf,
        "instruction_count": len(addresses),
        "address_list_sha256": sha256_bytes(address_bytes),
        "address_list_b64": base64.b64encode(address_bytes).decode(),
        "objdump_tool_version": objdump_version,
        "objdump_argvs": [list(argv) for argv in LOCAL_OBJDUMP_ARGVS],
        "addr2line_tool_version": addr2line_version,
        "addr2line_argv": list(addr2line_argv),
        "addr2line_stdout_size_bytes": len(result.stdout),
        "addr2line_stdout_sha256": sha256_bytes(result.stdout),
        "addr2line_stdout_b64": base64.b64encode(result.stdout).decode(),
        "addr2line_stderr_sha256": sha256_bytes(result.stderr),
        "addr2line_stderr_b64": base64.b64encode(result.stderr).decode(),
    }


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


def controller_source() -> bytes:
    return CONTROLLER.read_bytes()


def remote_controller_source() -> bytes:
    return REMOTE_CONTROLLER.read_bytes()


def build_audit_bytes() -> bytes:
    return BUILD_AUDIT.read_bytes()


def command_graph() -> dict[str, Any]:
    return {
        "external_actions": list(EXTERNAL_ACTIONS),
        "remote_actions": ["probe", "bucket-map-once"],
        "remote_map_readers": ["readelf", "objdump", "nm"],
        "local_map_readers": ["objdump", "addr2line"],
        "marker_mutation_routes": ["bucket-map-once"],
        "cargo_routes": [],
        "perf_routes": [],
        "subject_routes": [],
        "parity_routes": [],
        "runtime_routes": [],
    }


def verify_command_graph() -> dict[str, Any]:
    graph = command_graph()
    require(tuple(graph["external_actions"]) == EXTERNAL_ACTIONS, "local action graph drift")
    require(graph["remote_actions"] == ["probe", "bucket-map-once"], "remote action graph drift")
    require(graph["remote_map_readers"] == ["readelf", "objdump", "nm"], "remote reader graph drift")
    require(graph["local_map_readers"] == ["objdump", "addr2line"], "local reader graph drift")
    require(graph["marker_mutation_routes"] == ["bucket-map-once"], "marker mutation graph drift")
    for key in ("cargo_routes", "perf_routes", "subject_routes", "parity_routes", "runtime_routes"):
        require(graph[key] == [], f"forbidden route present: {key}")
    source = remote_controller_source().decode()
    compile(source, str(REMOTE_CONTROLLER), "exec")
    require("choices=" not in source, "remote action registry must remain explicit in main")
    for forbidden in ("cargo ", "rustc ", "perf record", "perf stat", "--pid", "SIGINT", "ld-linux"):
        require(forbidden not in source, f"remote source contains forbidden execution token: {forbidden}")
    require(source.count("os.rename(available, consumed)") == 1, "marker rename cardinality drift")
    require(source.count("run_to_files(") == 4, "remote map reader call-site graph drift")
    require("run_to_files(addr2line" not in source, "remote addr2line execution route present")
    require(
        "zip(EXPECTED_SYMBOLS, OBJDUMP_ARGVS, strict=True)" in source,
        "three-symbol objdump loop drift",
    )
    return graph


def verify_local_admission() -> dict[str, Any]:
    require(not LOCAL_RESULT.exists(), "local bucket-map result already exists")
    require_file(SSH_IDENTITY, mode="0600")
    audit = require_file(BUILD_AUDIT, digest=EXPECTED["build_audit"], mode="0444")
    value = json.loads(BUILD_AUDIT.read_text())
    require(value.get("verdict") == "D2_BUILD_AUDITED", "build audit verdict drift")
    require(value.get("d2", {}).get("elf", {}).get("sha256") == EXPECTED["elf"], "audit ELF drift")
    require(value.get("d2", {}).get("build_id") == EXPECTED["build_id"], "audit Build ID drift")
    require(value.get("bucket_map_marker_available") is True, "build audit did not admit bucket map")
    return {
        "build_audit": audit,
        "local_d2_elf": require_file(
            LOCAL_D2_ELF,
            digest=EXPECTED["elf"],
            size=EXPECTED["elf_size"],
            mode="0555",
        ),
        "reader_correction_v3": require_file(
            READER_CORRECTION_V3,
            digest=EXPECTED["reader_correction_v3"],
            mode="0444",
        ),
        "controller": file_identity(CONTROLLER),
        "remote_controller": file_identity(REMOTE_CONTROLLER),
    }


REMOTE_BOOTSTRAP = (
    "import base64,hashlib,json,sys\n"
    "envelope=json.loads(sys.stdin.buffer.read())\n"
    "source=base64.b64decode(envelope['remote_controller'],validate=True)\n"
    "payload=base64.b64decode(envelope['payload'],validate=True)\n"
    "assert hashlib.sha256(source).hexdigest()==sys.argv[1], 'remote controller SHA mismatch'\n"
    "assert hashlib.sha256(payload).hexdigest()==sys.argv[2], 'payload SHA mismatch'\n"
    "sys.argv=['lay-v10-e1-traversal-d2-bucket-map-remote.py',base64.b64encode(payload).decode()]\n"
    "ns={'__name__':'__main__','__file__':'<lay-v10-e1-traversal-d2-bucket-map-remote.py>'}\n"
    "exec(compile(source,ns['__file__'],'exec'),ns)\n"
)


def payload(action: str) -> bytes:
    local_source = controller_source()
    remote_source = remote_controller_source()
    audit = build_audit_bytes()
    return canonical_json_bytes(
        {
            "action": action,
            "build_audit_sha256": EXPECTED["build_audit"],
            "build_audit_receipt_b64": base64.b64encode(audit).decode(),
            "reader_correction_v3_sha256": EXPECTED["reader_correction_v3"],
            "reader_correction_v3_b64": base64.b64encode(READER_CORRECTION_V3.read_bytes()).decode(),
            "local_controller_sha256": sha256_bytes(local_source),
            "local_controller_b64": base64.b64encode(local_source).decode(),
            "remote_controller_sha256": sha256_bytes(remote_source),
            "remote_controller_b64": base64.b64encode(remote_source).decode(),
            "local_addr2line": prepare_local_addr2line(),
        }
    )


def remote_call(action: str, *, timeout: int) -> subprocess.CompletedProcess[bytes]:
    remote_source = remote_controller_source()
    request = payload(action)
    envelope = canonical_json_bytes(
        {
            "remote_controller": base64.b64encode(remote_source).decode(),
            "payload": base64.b64encode(request).decode(),
        }
    )
    command = [
        "/usr/bin/python3",
        "-c",
        REMOTE_BOOTSTRAP,
        sha256_bytes(remote_source),
        sha256_bytes(request),
    ]
    return run(ssh_argv(command), input_bytes=envelope, check=False, timeout=timeout)


def parse_last_json(result: subprocess.CompletedProcess[bytes], action: str) -> dict[str, Any]:
    require(result.returncode == 0, f"remote {action} failed ({result.returncode}):\n{result.stderr.decode(errors='replace')[-8000:]}")
    lines = result.stdout.decode(errors="replace").strip().splitlines()
    require(bool(lines), f"remote {action} produced no output")
    return json.loads(lines[-1])


def verify_marker_rows(rows: Sequence[Mapping[str, Any]], *, post_map: bool) -> None:
    observed = {str(value["name"]): value for value in rows}
    expected = dict(EXPECTED_MARKERS)
    if post_map:
        expected["bucket-map.consumed-before-exec"] = expected.pop("bucket-map.available")
    require(set(observed) == set(expected), f"marker membership drift: {sorted(observed)}")
    for name, (digest, size) in expected.items():
        value = observed[name]
        require(value.get("sha256") == digest and value.get("size_bytes") == size, f"marker identity drift: {name}")
        require(value.get("mode") == "0400", f"marker mode drift: {name}")


def verify_probe(value: Mapping[str, Any]) -> None:
    require(value.get("verdict") == "D2_BUCKET_MAP_REMOTE_PROBE_PASS", "remote probe verdict drift")
    require(value.get("hostname") == REMOTE_HOSTNAME, "remote host drift")
    require(value.get("machine_id_sha256") == REMOTE_MACHINE_ID_SHA256, "remote machine drift")
    require(value.get("parent_entries") == ["build-v1", "d2a-v1"], "remote parent drift")
    require(value.get("state_entries") == ["BUILD_STATE.json", "STATE.json", "markers", "route.lock"], "remote state drift")
    require(value.get("map_artifacts_present") is False and value.get("remote_writes") == 0, "probe observed map artifacts or writes")
    elf = value.get("elf") or {}
    require(elf.get("sha256") == EXPECTED["elf"] and elf.get("size_bytes") == EXPECTED["elf_size"], "remote ELF drift")
    require(value.get("text_sha256") == EXPECTED["text"], "remote .text drift")
    addr2line = value.get("local_addr2line") or {}
    require(addr2line.get("elf_sha256") == EXPECTED["elf"], "remote probe local addr2line ELF drift")
    require(addr2line.get("instruction_count") == EXPECTED["instruction_count"], "remote probe instruction count drift")
    require(addr2line.get("address_list_sha256") == EXPECTED["address_list"], "remote probe address-list SHA drift")
    require(addr2line.get("addr2line_stdout_size_bytes") == EXPECTED["addr2line_output_size"], "remote probe addr2line size drift")
    require(addr2line.get("addr2line_stdout_sha256") == EXPECTED["addr2line_output"], "remote probe addr2line SHA drift")
    verify_marker_rows(value.get("markers", []), post_map=False)


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
    compile(CONTROLLER.read_text(), str(CONTROLLER), "exec")
    admission = verify_local_admission()
    graph = verify_command_graph()
    result = remote_call("probe", timeout=120)
    probe = parse_last_json(result, "probe")
    verify_probe(probe)
    addr2line = prepare_local_addr2line()
    return {
        "schema": "lay.v10.e1-traversal-d2-primary-only-bucket-map-self-check.v1",
        "task_id": TASK_ID,
        "transaction_id": TRANSACTION_ID,
        "verdict": "D2_BUCKET_MAP_CONTROLLER_VERIFIED_UNRUN",
        "admission": admission,
        "command_graph": graph,
        "local_addr2line": {
            "elf_sha256": addr2line["elf"]["sha256"],
            "instruction_count": addr2line["instruction_count"],
            "address_list_sha256": addr2line["address_list_sha256"],
            "tool_version": addr2line["addr2line_tool_version"],
            "stdout_size_bytes": addr2line["addr2line_stdout_size_bytes"],
            "stdout_sha256": addr2line["addr2line_stdout_sha256"],
        },
        "remote_probe_sha256": sha256_bytes(canonical_json_bytes(probe)),
        "marker_mutations": 0,
        "map_created": False,
        "cargo_invocations": 0,
        "perf_record": 0,
        "perf_stat": 0,
        "pmu_events_opened": 0,
        "d2_subject": 0,
        "parity_executed": False,
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
            f"{REMOTE}:{REMOTE_RESULT}",
            str(destination),
        ],
        check=False,
        timeout=900,
    )
    require(result.returncode == 0, result.stderr.decode(errors="replace")[-6000:])


def bucket_map_once() -> dict[str, Any]:
    check = self_check()
    runtime_before = local_runtime_snapshot()
    result = remote_call("bucket-map-once", timeout=1800)
    remote_receipt = parse_last_json(result, "bucket-map-once")
    require(remote_receipt.get("verdict") == "D2_BUCKET_MAP_SEALED", "remote map verdict drift")
    require(remote_receipt.get("elf", {}).get("sha256") == EXPECTED["elf"], "remote map ELF drift")
    require(remote_receipt.get("build_id") == EXPECTED["build_id"], "remote map Build ID drift")
    require(remote_receipt.get("gap_bytes") == 0 and remote_receipt.get("overlap_count") == 0, "remote map coverage failed")
    require(remote_receipt.get("machine_byte_hash_mismatches") == 0, "remote map byte hash failed")
    require(remote_receipt.get("parity_executed") is False, "parity executed during map route")

    post_probe_result = remote_call("probe", timeout=120)
    require(post_probe_result.returncode != 0, "pre-map probe unexpectedly remained admissible after marker consumption")
    runtime_after = local_runtime_snapshot()
    require(runtime_before == runtime_after, "installed runtime projection changed during bucket-map route")

    stage = pathlib.Path(f"{LOCAL_RESULT}.stage-{os.getpid()}-{time.time_ns()}")
    stage.mkdir(parents=True, mode=0o700)
    try:
        remote_evidence = stage / "REMOTE_EVIDENCE"
        copy_remote_evidence(remote_evidence)
        entries = verify_sha256sums(remote_evidence)
        copied_receipt_path = remote_evidence / "D2_BUCKET_MAP_RECEIPT.json"
        copied_receipt = json.loads(copied_receipt_path.read_text())
        for key in ("verdict", "build_id", "range_count", "text_coverage_bytes"):
            require(copied_receipt.get(key) == remote_receipt.get(key), f"copied receipt mismatch: {key}")
        bucket_map = json.loads((remote_evidence / "D2_BUCKET_MAP.json").read_text())
        require(bucket_map.get("range_count") == len(bucket_map.get("ranges", [])), "map range count drift")
        require(bucket_map.get("coverage", {}).get("covered_bytes") == 15_980_919, "map byte coverage drift")
        require(bucket_map.get("coverage", {}).get("gap_bytes") == 0, "map gap drift")
        require(bucket_map.get("coverage", {}).get("overlap_count") == 0, "map overlap drift")
        require(bucket_map.get("reserved_absent_sub_buckets", {}).get("REDUNDANT_STATE_DECODE", {}).get("status") == "ABSENT", "redundant-state status drift")
        write_new_json(stage / "SELF_CHECK.json", check)
        write_new_json(stage / "RUNTIME_BEFORE.json", runtime_before)
        write_new_json(stage / "RUNTIME_AFTER.json", runtime_after)
        write_new_bytes(stage / "local-controller.py", controller_source())
        write_new_bytes(stage / "remote-controller.py", remote_controller_source())
        write_new_bytes(stage / "D2_BUILD_AUDIT_RECEIPT.json", build_audit_bytes())
        write_new_bytes(stage / "READER_SCOPE_CORRECTION_V3.md", READER_CORRECTION_V3.read_bytes())
        local_addr2line = prepare_local_addr2line()
        write_new_json(
            stage / "LOCAL_ADDR2LINE_PRODUCER.json",
            {
                "schema": local_addr2line["schema"],
                "elf": local_addr2line["elf"],
                "instruction_count": local_addr2line["instruction_count"],
                "address_list_sha256": local_addr2line["address_list_sha256"],
                "objdump_tool_version": local_addr2line["objdump_tool_version"],
                "objdump_argvs": local_addr2line["objdump_argvs"],
                "addr2line_tool_version": local_addr2line["addr2line_tool_version"],
                "addr2line_argv": local_addr2line["addr2line_argv"],
                "addr2line_stdout_size_bytes": local_addr2line["addr2line_stdout_size_bytes"],
                "addr2line_stdout_sha256": local_addr2line["addr2line_stdout_sha256"],
                "addr2line_stderr_sha256": local_addr2line["addr2line_stderr_sha256"],
            },
        )
        local_receipt = {
            "schema": "lay.v10.e1-traversal-d2-primary-only-local-bucket-map.v1",
            "task_id": TASK_ID,
            "transaction_id": TRANSACTION_ID,
            "verdict": "D2_BUCKET_MAP_SEALED",
            "local_controller": file_identity(stage / "local-controller.py"),
            "remote_controller": file_identity(stage / "remote-controller.py"),
            "build_audit_receipt_sha256": EXPECTED["build_audit"],
            "reader_correction_v3_sha256": EXPECTED["reader_correction_v3"],
            "addr2line_output_sha256": EXPECTED["addr2line_output"],
            "address_list_sha256": EXPECTED["address_list"],
            "remote_manifest_entries": entries,
            "remote_receipt_sha256": sha256_file(copied_receipt_path),
            "map_sha256": sha256_file(remote_evidence / "D2_BUCKET_MAP.json"),
            "range_count": remote_receipt["range_count"],
            "text_coverage_bytes": remote_receipt["text_coverage_bytes"],
            "gap_bytes": 0,
            "overlap_count": 0,
            "machine_byte_hash_mismatches": 0,
            "bucket_map_marker_consumed": True,
            "parity_marker_consumed": False,
            "other_available_markers": 9,
            "other_consumed_markers": 1,
            "cargo_invocations": 0,
            "perf_record": 0,
            "perf_stat": 0,
            "pmu_events_opened": 0,
            "d2_subject_executed": False,
            "parity_executed": False,
            "runtime_before": runtime_before,
            "runtime_after": runtime_after,
            "runtime_authority_changed": False,
            "next_action_admitted": "independent read-only bucket-map audit only",
        }
        write_new_json(stage / "LOCAL_BUCKET_MAP_RECEIPT.json", local_receipt)
        write_sha256sums(stage)
        seal_tree(stage)
        require(not LOCAL_RESULT.exists(), "local result appeared during publication")
        os.rename(stage, LOCAL_RESULT)
        fsync_directory(LOCAL_RESULT.parent)
    except BaseException:
        remove_owned_tree(stage)
        raise
    return {
        "verdict": "D2_BUCKET_MAP_SEALED",
        "local_result": str(LOCAL_RESULT),
        "map_sha256": sha256_file(LOCAL_RESULT / "REMOTE_EVIDENCE/D2_BUCKET_MAP.json"),
        "range_count": remote_receipt["range_count"],
        "parity_executed": False,
        "runtime_authority_changed": False,
        "next_action_admitted": "independent read-only bucket-map audit only",
    }


def parser() -> argparse.ArgumentParser:
    value = argparse.ArgumentParser()
    value.add_argument("action", choices=EXTERNAL_ACTIONS)
    return value


def main() -> int:
    arguments = parser().parse_args()
    try:
        value = self_check() if arguments.action == "self-check" else bucket_map_once()
        print(json.dumps(value, ensure_ascii=False, sort_keys=True))
        return 0
    except Exception as error:
        print(f"D2 BUCKET MAP ERROR: {type(error).__name__}: {error}", file=sys.stderr)
        return 1


if __name__ == "__main__":
    raise SystemExit(main())
