#!/usr/bin/env python3
"""Fail-closed controller for the admitted V10 hardware B0a through B2 route."""

from __future__ import annotations

import argparse
import ast
import contextlib
import fcntl
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
import tempfile
import time
from typing import Any, Iterable, Sequence


TASK_ID = "slice8b-v10-hardware-b0-b2-v3-20260824"
REMOTE = "e@192.168.3.94"
REMOTE_HOSTNAME = "e-MEGA-MINI-M1-13th"
REMOTE_MACHINE_ID_SHA256 = "5ac0bb537745673ef80942c2df32d586f5753c85de1c8b2453aa4e7317b77441"
REMOTE_DONOR_ROOT = pathlib.Path("/home/e/projects/lay-slice8b-e4b5447")
REMOTE_INSTALLED_V13 = pathlib.Path(
    "/home/e/.local/share/lay/nanda_wave/l2/LAY-L2-RU-FULL-v13.bin"
)
REMOTE_PROVENANCE_ROOT = pathlib.Path("/home/e/.local/share/lay/provenance")
REMOTE_NEAREST_EXISTING_PARENT = pathlib.Path("/home/e/.local/share/lay")
REMOTE_NEAREST_EXISTING_PARENT_DEVICE = 66_306
REMOTE_PARENT = REMOTE_PROVENANCE_ROOT / TASK_ID
REMOTE_B0A = REMOTE_PARENT / "b0a-input-closure-v2"
REMOTE_BUILD = REMOTE_PARENT / "diagnostic-build-v1"
REMOTE_FREEZER = REMOTE_PARENT / "schedule-freezer-v1"
REMOTE_B0B = REMOTE_PARENT / "b0b-schedule-closure-v1"
REMOTE_B1 = REMOTE_PARENT / "b1-environment-v1"
REMOTE_B2 = REMOTE_PARENT / "b2-benign-pmu-capability-v1"
REMOTE_STATE = pathlib.Path("/home/e/.local/state/lay") / TASK_ID

PROJECT_ROOT = pathlib.Path(__file__).resolve().parents[1]
P0 = pathlib.Path("/home/ubu/.local/share/lay/provenance/slice8b-v10-f6178f")
P0_CORRECTION = pathlib.Path(
    "/home/ubu/.local/share/lay/provenance/slice8b-v10-f6178f-correction-v1"
)
FRAGMENT = PROJECT_ROOT / "scripts/lay_v10_hardware_test_module.rs.inc"
CARGO_GUARD = PROJECT_ROOT / "scripts/cargo-guard.sh"
ACTIVE_PREFLIGHT = PROJECT_ROOT / (
    "docs/structural_gates/preflights/"
    "LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_HARDWARE_B0_B2_IMPLEMENTATION_V7_2026-08-25.json"
)
ACTIVE_PREFLIGHT_RECEIPT = PROJECT_ROOT / (
    "docs/structural_gates/receipts/"
    "LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_HARDWARE_B0_B2_IMPLEMENTATION_PREFLIGHT_V7_2026-08-25.json"
)

EXPECTED = {
    "active_preflight": "71c9a5b1de02d14ac175c0f4a32b5bfdbc9037097147aab10f0cddaba7dde1ab",
    "active_preflight_receipt": "e67d8fc1af6ba63680a5185c4a269baf1dc0b7f752baa4759a1a94d0c2a0044f",
    "v10_source": "f6178f3882c1a216a9c2aaf7ff59dd6bdd78798708516b753ea7208177d9718c",
    "production_prefix": "ce9ea2d290602774e2f45444e10e6fdabce6af4a61ae2bdca54c6d2e31a53f26",
    "historical_elf": "1e83ef76df68cd2f0238d1334eb4049f9063608292e72e5454a09f21a4afacc1",
    "historical_build_id": "32e47da137adff6d49f9209ccd2804b6daa728ae",
    "sidecar": "a1aa95be8c43fae8fedf02d3261dd1265dcbfbc8d618330fcd6449ae829df8cd",
    "v7": "33fded73e13f565bdc08a83bd440be67c80f6d85a196453050b09bc9f7ef28e4",
    "v13_package": "cce259fe0ce5dce67702383363b66f0fe9b9ff5a87d8f01c4fcf342d91218d7b",
    "cargo_toml": "90a3ae6b16677ff49b70aab3457b794cd1e114a547f9e113a8b49595f976207b",
    "cargo_lock": "e6399be48c393e1557b451fcb04d886f9fbdaa9812109410ac3d9ecdc98b93f1",
    "cargo_guard": "a5b01f9a12b32ee2bcfc957a5e9f65efb52fe6e7644850ae3f00e412648243fe",
    "yes": "ee431b97fb62f59ee94fa698dbc98971001bbb1cbd9c5e32ce4ab4c5530924d8",
    "taskset": "63e52c4b99a688ccd7bab6edbc6df2af1acad124eb852adcb4d20043d28eb2d3",
    "perf": "2d0953085bf720a25efbe24f853e97d27b1f12f18a398255ff82cbafde254dad",
}

EXPECTED_SIZES = {
    "v10_source": 91_518,
    "production_prefix": 39_047,
    "historical_elf": 20_457_336,
    "sidecar": 3_689_884,
    "v7": 1_606_189,
    "v13_package": 140_556_462,
    "yes": 31_112,
}

TEST_NAMES = {
    "freezer": "nanda_wave::l2_field::v13_typed_peak::tests::v10_hardware_schedule_freezer",
    "parity": "nanda_wave::l2_field::v13_typed_peak::tests::v10_hardware_semantic_parity",
    "b5": "nanda_wave::l2_field::v13_typed_peak::tests::v10_hardware_b5_proxy",
    "b6": "nanda_wave::l2_field::v13_typed_peak::tests::v10_hardware_b6_proxy",
}

PERF_GROUPS = {
    "G0": [
        "task-clock",
        "cycles",
        "ref-cycles",
        "instructions",
        "context-switches",
        "cpu-migrations",
        "page-faults",
    ],
    "G1": ["branches", "branch-misses", "cache-references", "cache-misses"],
    "G2": ["L1-dcache-loads", "L1-dcache-load-misses", "LLC-loads", "LLC-load-misses"],
    "G3": ["dTLB-loads", "dTLB-load-misses"],
}

SOFTWARE_EVENTS = {
    "task-clock",
    "context-switches",
    "cpu-migrations",
    "page-faults",
}

EXPECTED_CARGO_VERSION = "cargo 1.97.1 (c980f4866 2026-06-30)"
EXPECTED_RUSTC_RELEASE = "release: 1.97.1"
EXPECTED_RUSTC_COMMIT = "commit-hash: 8bab26f4f"
EXPECTED_RUSTC_HOST = "host: x86_64-unknown-linux-gnu"
EXPECTED_LLVM_VERSION = "LLVM version: 22.1.6"
REQUIRED_TARGET_FEATURES = {"fxsr", "sse", "sse2"}
MIN_BUILD_FREE_BYTES = 16 * 1024 * 1024 * 1024


class GateError(RuntimeError):
    pass


def sha256_bytes(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def sha256_file(path: pathlib.Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as source:
        for block in iter(lambda: source.read(1024 * 1024), b""):
            digest.update(block)
    return digest.hexdigest()


def mode_string(path: pathlib.Path) -> str:
    return f"{stat.S_IMODE(path.stat().st_mode):04o}"


def require(condition: bool, message: str) -> None:
    if not condition:
        raise GateError(message)


def nearest_existing_directory(path: pathlib.Path) -> tuple[pathlib.Path, os.stat_result]:
    candidate = path
    while True:
        try:
            metadata = candidate.stat()
        except FileNotFoundError:
            parent = candidate.parent
            require(parent != candidate, f"no existing ancestor for {path}")
            candidate = parent
            continue
        require(stat.S_ISDIR(metadata.st_mode), f"nearest existing ancestor is not a directory: {candidate}")
        return candidate, metadata


def require_directory_device(path: pathlib.Path, expected_device: int) -> os.stat_result:
    metadata = path.stat()
    require(stat.S_ISDIR(metadata.st_mode), f"not a directory: {path}")
    require(metadata.st_dev == expected_device, f"filesystem device mismatch: {path}")
    return metadata


def require_file(
    path: pathlib.Path,
    *,
    sha256: str | None = None,
    size: int | None = None,
    mode: str | None = None,
) -> dict[str, Any]:
    require(path.is_file(), f"missing file: {path}")
    actual = {
        "path": str(path),
        "mode": mode_string(path),
        "size_bytes": path.stat().st_size,
        "sha256": sha256_file(path),
    }
    if sha256 is not None:
        require(actual["sha256"] == sha256, f"SHA-256 mismatch: {path}")
    if size is not None:
        require(actual["size_bytes"] == size, f"size mismatch: {path}")
    if mode is not None:
        require(actual["mode"] == mode, f"mode mismatch: {path}")
    return actual


def load_json(path: pathlib.Path) -> Any:
    with path.open("rb") as source:
        return json.load(source)


def canonical_json_bytes(value: Any) -> bytes:
    return (json.dumps(value, ensure_ascii=False, sort_keys=True, separators=(",", ":")) + "\n").encode()


def write_all(descriptor: int, data: bytes) -> None:
    remaining = memoryview(data)
    while remaining:
        written = os.write(descriptor, remaining)
        require(written > 0, "short write made no progress")
        remaining = remaining[written:]


def write_new_json(path: pathlib.Path, value: Any, mode: int = 0o444) -> None:
    flags = os.O_WRONLY | os.O_CREAT | os.O_EXCL
    descriptor = os.open(path, flags, 0o600)
    try:
        data = json.dumps(value, ensure_ascii=False, sort_keys=True, indent=2).encode() + b"\n"
        write_all(descriptor, data)
        os.fsync(descriptor)
        os.fchmod(descriptor, mode)
    finally:
        os.close(descriptor)


def write_new_bytes(path: pathlib.Path, data: bytes, mode: int = 0o444) -> None:
    descriptor = os.open(path, os.O_WRONLY | os.O_CREAT | os.O_EXCL, 0o600)
    try:
        write_all(descriptor, data)
        os.fsync(descriptor)
        os.fchmod(descriptor, mode)
    finally:
        os.close(descriptor)


def fsync_directory(path: pathlib.Path) -> None:
    descriptor = os.open(path, os.O_RDONLY | os.O_DIRECTORY)
    try:
        os.fsync(descriptor)
    finally:
        os.close(descriptor)


def chmod_readonly_tree(root: pathlib.Path) -> None:
    for path in sorted(root.rglob("*"), reverse=True):
        if path.is_symlink():
            raise GateError(f"symlink is forbidden in evidence tree: {path}")
        if path.is_dir():
            path.chmod(0o555)
        else:
            executable = bool(path.stat().st_mode & 0o111)
            path.chmod(0o555 if executable else 0o444)
    root.chmod(0o555)


def make_tree_writable(root: pathlib.Path) -> None:
    if not root.exists():
        return
    if root.is_dir():
        root.chmod(0o755)
    for path in sorted(root.rglob("*")):
        if path.is_symlink():
            raise GateError(f"symlink is forbidden in managed tree: {path}")
        if path.is_dir():
            path.chmod(0o755)
        else:
            executable = bool(path.stat().st_mode & 0o111)
            path.chmod(0o755 if executable else 0o644)


def remove_tree(root: pathlib.Path) -> None:
    if root.exists():
        make_tree_writable(root)
        shutil.rmtree(root)


def atomic_publish(stage: pathlib.Path, final: pathlib.Path) -> None:
    require(stage.parent == final.parent, "stage and final must share a parent")
    require(stage.is_dir(), f"missing stage: {stage}")
    require(not final.exists(), f"final path already exists: {final}")
    for path in [stage, *stage.rglob("*")]:
        require(not path.is_symlink(), f"symlink is forbidden before publication: {path}")
        require(stat.S_IMODE(path.stat().st_mode) & 0o222 == 0, f"writable object before publication: {path}")
    fsync_directory(stage)
    os.rename(stage, final)
    fsync_directory(final.parent)


def inventory(root: pathlib.Path) -> list[dict[str, Any]]:
    records = []
    for path in sorted(root.rglob("*")):
        if path.is_file():
            records.append(
                {
                    "path": str(path.relative_to(root)),
                    "mode": mode_string(path),
                    "size_bytes": path.stat().st_size,
                    "sha256": sha256_file(path),
                }
            )
    return records


def write_sha256sums(root: pathlib.Path, *, exclude: Iterable[str] = ()) -> None:
    excluded = set(exclude)
    lines = []
    for record in inventory(root):
        if record["path"] not in excluded:
            lines.append(f"{record['sha256']}  {record['path']}\n")
    write_new_bytes(root / "SHA256SUMS", "".join(lines).encode())


def verify_sha256sums(root: pathlib.Path) -> int:
    manifest = root / "SHA256SUMS"
    require(manifest.is_file(), f"missing SHA256SUMS: {root}")
    seen: set[str] = set()
    for line in manifest.read_text(encoding="utf-8").splitlines():
        digest, separator, relative = line.partition("  ")
        require(separator == "  " and re.fullmatch(r"[0-9a-f]{64}", digest) is not None, "invalid SHA256SUMS row")
        path = pathlib.PurePosixPath(relative)
        require(not path.is_absolute() and ".." not in path.parts, f"unsafe SHA256SUMS path: {relative}")
        normalized = str(path)
        require(normalized not in seen and normalized != "SHA256SUMS", f"duplicate or recursive SHA256SUMS path: {relative}")
        seen.add(normalized)
        require_file(root / path, sha256=digest)
    actual = {record["path"] for record in inventory(root) if record["path"] != "SHA256SUMS"}
    require(seen == actual, "SHA256SUMS inventory mismatch")
    return len(seen)


def verify_external_sha256sums(root: pathlib.Path, manifest: pathlib.Path) -> int:
    seen: set[str] = set()
    for line in manifest.read_text(encoding="utf-8").splitlines():
        digest, separator, relative = line.partition("  ")
        require(separator == "  " and re.fullmatch(r"[0-9a-f]{64}", digest) is not None, f"invalid manifest row: {manifest}")
        normalized = relative.removeprefix("./")
        path = pathlib.PurePosixPath(normalized)
        require(not path.is_absolute() and ".." not in path.parts, f"unsafe manifest path: {relative}")
        require(normalized not in seen, f"duplicate manifest path: {relative}")
        seen.add(normalized)
        require_file(root / path, sha256=digest)
    return len(seen)


def seal_stage(stage: pathlib.Path) -> int:
    write_sha256sums(stage)
    count = verify_sha256sums(stage)
    chmod_readonly_tree(stage)
    require(verify_sha256sums(stage) == count, "sealed SHA256SUMS verification drift")
    return count


def controlled_environment() -> dict[str, str]:
    return {
        "HOME": "/home/e",
        "LANG": "C.UTF-8",
        "LC_ALL": "C.UTF-8",
        "LOGNAME": "e",
        "PATH": "/home/e/.cargo/bin:/usr/local/bin:/usr/bin:/bin",
        "USER": "e",
    }


def run(
    command: Sequence[str],
    *,
    cwd: pathlib.Path | None = None,
    env: dict[str, str] | None = None,
    input_bytes: bytes | None = None,
    stdout: Any = subprocess.PIPE,
    stderr: Any = subprocess.PIPE,
    check: bool = True,
) -> subprocess.CompletedProcess[bytes]:
    result = subprocess.run(
        list(command),
        cwd=cwd,
        env=env,
        input=input_bytes,
        stdout=stdout,
        stderr=stderr,
        check=False,
    )
    if check and result.returncode != 0:
        stderr_text = (result.stderr or b"").decode(errors="replace")[-4000:]
        raise GateError(f"command failed ({result.returncode}): {command!r}\n{stderr_text}")
    return result


def ssh_command(command: Sequence[str]) -> str:
    arguments = [str(value) for value in command]
    require(arguments, "remote command must not be empty")
    require(not any("\0" in value for value in arguments), "remote argv contains NUL")
    return shlex.join(arguments)


def ssh_process_argv(command: Sequence[str]) -> list[str]:
    return [
        "ssh",
        "-o",
        "BatchMode=yes",
        "-o",
        "ConnectTimeout=8",
        REMOTE,
        ssh_command(command),
    ]


def ssh(command: Sequence[str], *, check: bool = True) -> subprocess.CompletedProcess[bytes]:
    return run(ssh_process_argv(command), check=check)


def scp(source: pathlib.Path, destination: pathlib.PurePosixPath, *, recursive: bool = False) -> None:
    command = ["scp", "-q", "-p"]
    if recursive:
        command.append("-r")
    command.extend([str(source), f"{REMOTE}:{destination}"])
    run(command)


def remote_controller(action: str) -> None:
    controller = REMOTE_B0A / "inputs/controller/lay-v10-hardware-b0-b2.py"
    result = ssh(["/usr/bin/python3", str(controller), action], check=False)
    if result.stdout:
        sys.stdout.buffer.write(result.stdout)
    if result.stderr:
        sys.stderr.buffer.write(result.stderr)
    if result.returncode != 0:
        raise GateError(f"remote action {action} failed with {result.returncode}")


def parse_source_closure(path: pathlib.Path) -> list[dict[str, Any]]:
    rows = []
    lines = path.read_text(encoding="utf-8").splitlines()
    require(lines and lines[0].startswith("dependency_path\t"), "invalid source closure header")
    for line in lines[1:]:
        source, mode, size, digest, status = line.split("\t")
        source_path = pathlib.PurePosixPath(source)
        if source_path.is_absolute():
            try:
                relative = source_path.relative_to(REMOTE_DONOR_ROOT)
            except ValueError as error:
                raise GateError(f"source closure path escapes donor root: {source}") from error
        else:
            relative = pathlib.PurePosixPath(os.path.normpath(source))
        require(not relative.is_absolute() and ".." not in relative.parts, f"unsafe source path: {source}")
        rows.append(
            {
                "source": source,
                "relative": str(relative),
                "mode": f"{int(mode, 8):04o}",
                "size_bytes": int(size),
                "sha256": digest,
                "status": status,
            }
        )
    require(len(rows) == 509, f"expected 509 source closure rows, found {len(rows)}")
    by_relative: dict[str, dict[str, Any]] = {}
    for row in rows:
        previous = by_relative.get(row["relative"])
        if previous is None:
            by_relative[row["relative"]] = row
            continue
        require(
            all(previous[key] == row[key] for key in ("mode", "size_bytes", "sha256", "status")),
            f"normalized source aliases disagree: {row['relative']}",
        )
    return rows


def verify_local_admission() -> dict[str, Any]:
    preflight = require_file(
        ACTIVE_PREFLIGHT, sha256=EXPECTED["active_preflight"], mode="0664"
    )
    receipt = require_file(
        ACTIVE_PREFLIGHT_RECEIPT,
        sha256=EXPECTED["active_preflight_receipt"],
        mode="0664",
    )
    receipt_json = load_json(ACTIVE_PREFLIGHT_RECEIPT)
    require(receipt_json.get("verdict") == "READY_TO_IMPLEMENT", "active preflight is not ready")
    require(receipt_json.get("safe_to_implement") is True, "active preflight is not safe")
    require(not receipt_json.get("blockers"), "active preflight has blockers")
    audit = PROJECT_ROOT / (
        "docs/structural_gates/receipts/"
        "LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_HARDWARE_PERF_AUDIT_CORRECTION_V3_2026-08-24/"
        "PERF_AUDIT_CORRECTION_V3.json"
    )
    audit_json = load_json(audit)
    require(audit_json["totals"]["pre_b2_perf_executable_invocations"] == 2, "perf ledger drift")
    require(audit_json["totals"]["pmu_events_opened_or_measured"] == 0, "PMU ledger drift")
    return {"manifest": preflight, "receipt": receipt, "perf_audit": require_file(audit)}


def verify_p0_inputs() -> dict[str, Any]:
    checks = {
        "v10_source": require_file(
            P0 / "artifacts/v13_typed_peak.v10.rs",
            sha256=EXPECTED["v10_source"],
            size=EXPECTED_SIZES["v10_source"],
            mode="0444",
        ),
        "historical_elf": require_file(
            P0 / "artifacts/lay-8b3af2179d6f9249",
            sha256=EXPECTED["historical_elf"],
            size=EXPECTED_SIZES["historical_elf"],
            mode="0555",
        ),
        "sidecar": require_file(
            P0 / "artifacts/LAY-L2-RU-FULL-v13.dafsa",
            sha256=EXPECTED["sidecar"],
            size=EXPECTED_SIZES["sidecar"],
            mode="0444",
        ),
        "v7": require_file(
            P0 / "artifacts/slice8b-v7-fixed-13x100.json",
            sha256=EXPECTED["v7"],
            size=EXPECTED_SIZES["v7"],
            mode="0400",
        ),
        "cargo_toml": require_file(
            P0 / "artifacts/Cargo.toml", sha256=EXPECTED["cargo_toml"], mode="0444"
        ),
        "cargo_lock": require_file(
            P0 / "artifacts/Cargo.lock", sha256=EXPECTED["cargo_lock"], mode="0444"
        ),
        "cargo_guard": require_file(CARGO_GUARD, sha256=EXPECTED["cargo_guard"], mode="0775"),
        "fragment": require_file(FRAGMENT, mode="0664"),
        "controller": require_file(pathlib.Path(__file__).resolve(), mode="0755"),
    }
    source = (P0 / "artifacts/v13_typed_peak.v10.rs").read_bytes()
    require(source.endswith(b"}\n"), "V10 source does not end with the test-module brace")
    require(sha256_bytes(source[:39_047]) == EXPECTED["production_prefix"], "production prefix drift")
    require(source[39_047:].startswith(b"#[cfg(test)]"), "test-module boundary drift")
    historical_elf = P0 / "artifacts/lay-8b3af2179d6f9249"
    elf_header = run(["readelf", "-h", str(historical_elf)]).stdout.decode(errors="replace")
    elf_sections = run(["readelf", "-S", str(historical_elf)]).stdout.decode(errors="replace")
    require("Class:                             ELF64" in elf_header, "historical ELF class drift")
    require("Machine:                           Advanced Micro Devices X86-64" in elf_header, "historical ELF machine drift")
    require(".symtab" not in elf_sections, "historical ELF unexpectedly has .symtab")
    require(elf_build_id(historical_elf) == EXPECTED["historical_build_id"], "historical Build ID drift")
    checks["historical_elf_shape"] = {
        "class": "ELF64",
        "machine": "x86-64",
        "stripped": True,
        "build_id": EXPECTED["historical_build_id"],
    }
    closure = P0 / "manifests/source-closure-observed.tsv"
    checks["source_closure"] = require_file(closure, mode="0444")
    parse_source_closure(closure)
    original_manifest = P0 / "manifests/SHA256SUMS"
    correction_manifest = P0_CORRECTION / "SHA256SUMS"
    checks["p0_sha256sums"] = require_file(original_manifest, mode="0444")
    checks["correction_sha256sums"] = require_file(correction_manifest, mode="0444")
    checks["p0_manifest_entries"] = verify_external_sha256sums(P0, original_manifest)
    checks["correction_manifest_entries"] = verify_external_sha256sums(
        P0_CORRECTION, correction_manifest
    )
    return checks


def remote_identity_probe() -> dict[str, Any]:
    code = r'''
import hashlib, json, pathlib, socket, stat, sys
def digest(path):
    h=hashlib.sha256()
    with open(path,'rb') as f:
        for b in iter(lambda:f.read(1024*1024),b''): h.update(b)
    return h.hexdigest()
def nearest_existing_directory(path):
    candidate=path
    while True:
        try: metadata=candidate.stat()
        except FileNotFoundError:
            parent=candidate.parent
            if parent == candidate: raise SystemExit(f'no existing ancestor for {path}')
            candidate=parent
            continue
        if not stat.S_ISDIR(metadata.st_mode):
            raise SystemExit(f'nearest existing ancestor is not a directory: {candidate}')
        return candidate, metadata
machine_path=pathlib.Path('/etc/machine-id')
v13=pathlib.Path('/home/e/.local/share/lay/nanda_wave/l2/LAY-L2-RU-FULL-v13.bin')
requested_parent=pathlib.Path(sys.argv[1])
nearest_parent, nearest_metadata=nearest_existing_directory(requested_parent)
print(json.dumps({
 'hostname': socket.gethostname(),
 'machine_id_sha256': digest(machine_path),
 'v13_exists': v13.is_file(),
 'v13_bytes': v13.stat().st_size if v13.is_file() else None,
 'v13_sha256': digest(v13) if v13.is_file() else None,
 'requested_output_parent': str(requested_parent),
 'requested_output_parent_exists': requested_parent.exists(),
 'nearest_existing_parent': str(nearest_parent),
 'nearest_existing_parent_device': nearest_metadata.st_dev,
 'nearest_existing_parent_inode': nearest_metadata.st_ino,
}))
'''
    result = ssh(["/usr/bin/python3", "-c", code, str(REMOTE_PROVENANCE_ROOT)])
    value = json.loads(result.stdout)
    require(value["hostname"] == REMOTE_HOSTNAME, "remote hostname mismatch")
    require(value["machine_id_sha256"] == REMOTE_MACHINE_ID_SHA256, "remote machine mismatch")
    require(value["v13_bytes"] == EXPECTED_SIZES["v13_package"], "remote V13 size mismatch")
    require(value["v13_sha256"] == EXPECTED["v13_package"], "remote V13 SHA mismatch")
    require(value["requested_output_parent"] == str(REMOTE_PROVENANCE_ROOT), "remote output parent mismatch")
    require(value["requested_output_parent_exists"] is False, "remote provenance root unexpectedly exists")
    require(
        value["nearest_existing_parent"] == str(REMOTE_NEAREST_EXISTING_PARENT),
        "nearest existing remote parent drift",
    )
    require(
        value["nearest_existing_parent_device"] == REMOTE_NEAREST_EXISTING_PARENT_DEVICE,
        "nearest existing remote parent device drift",
    )
    return value


def create_remote_b0a_stage(stage: pathlib.PurePosixPath, remote_probe: dict[str, Any]) -> None:
    code = r'''
import pathlib, stat, sys
stage=pathlib.Path(sys.argv[1]); parent=stage.parent
expected_ancestor=pathlib.Path(sys.argv[2]); expected_device=int(sys.argv[3]); expected_inode=int(sys.argv[4])
created=[]
def nearest_existing_directory(path):
    candidate=path
    while True:
        try: metadata=candidate.stat()
        except FileNotFoundError:
            next_parent=candidate.parent
            if next_parent == candidate: raise SystemExit(f'no existing ancestor for {path}')
            candidate=next_parent
            continue
        if not stat.S_ISDIR(metadata.st_mode):
            raise SystemExit(f'nearest existing ancestor is not a directory: {candidate}')
        return candidate, metadata
def require_device(path):
    metadata=path.stat()
    if not stat.S_ISDIR(metadata.st_mode) or metadata.st_dev != expected_device:
        raise SystemExit(f'filesystem device mismatch: {path}')
    return metadata
try:
    ancestor, metadata=nearest_existing_directory(parent)
    if ancestor != expected_ancestor or metadata.st_dev != expected_device or metadata.st_ino != expected_inode:
        raise SystemExit('nearest existing output parent identity drift')
    cursor=ancestor
    for part in parent.relative_to(ancestor).parts:
        cursor=cursor/part
        try: cursor.mkdir(mode=0o755)
        except FileExistsError:
            require_device(cursor)
        else:
            created.append(cursor)
    require_device(parent)
    if stage.exists(): raise SystemExit('stage collision')
    stage.mkdir(mode=0o700); created.append(stage)
    require_device(stage)
    (stage/'inputs').mkdir(mode=0o700); created.append(stage/'inputs')
    (stage/'inputs'/'controller').mkdir(mode=0o700); created.append(stage/'inputs'/'controller')
except BaseException:
    for path in reversed(created):
        try: path.rmdir()
        except OSError: pass
    raise
'''
    ssh(
        [
            "/usr/bin/python3",
            "-c",
            code,
            str(stage),
            remote_probe["nearest_existing_parent"],
            str(remote_probe["nearest_existing_parent_device"]),
            str(remote_probe["nearest_existing_parent_inode"]),
        ]
    )


def local_b0a() -> None:
    admission = verify_local_admission()
    p0_checks = verify_p0_inputs()
    remote_probe = remote_identity_probe()
    token = f"{os.getpid()}-{time.time_ns()}"
    stage = pathlib.PurePosixPath(f"{REMOTE_B0A}.stage-{token}")
    try:
        create_remote_b0a_stage(stage, remote_probe)
        scp(P0, stage / "inputs", recursive=True)
        scp(P0_CORRECTION, stage / "inputs", recursive=True)
        scp(pathlib.Path(__file__).resolve(), stage / "inputs/controller/lay-v10-hardware-b0-b2.py")
        scp(FRAGMENT, stage / "inputs/controller/lay_v10_hardware_test_module.rs.inc")
        scp(CARGO_GUARD, stage / "inputs/controller/cargo-guard.sh")
        metadata = {
            "schema": "lay.v10.hardware-b0a-dispatch.v1",
            "task_id": TASK_ID,
            "admission": admission,
            "p0_checks": p0_checks,
            "remote_probe": remote_probe,
            "stage": str(stage),
            "final": str(REMOTE_B0A),
            "controller_sha256": sha256_file(pathlib.Path(__file__).resolve()),
            "fragment_sha256": sha256_file(FRAGMENT),
            "cargo_guard_sha256": sha256_file(CARGO_GUARD),
            "pre_b2_perf_executable_invocations": 2,
            "pre_b2_pmu_measurements": 0,
            "runtime_authority_changed": False,
            "installed_lay_changed": False,
        }
        with tempfile.TemporaryDirectory(prefix="lay-v10-b0a-dispatch-") as temporary:
            dispatch_path = pathlib.Path(temporary) / "DISPATCH.json"
            write_new_json(dispatch_path, metadata, 0o444)
            scp(dispatch_path, stage / "DISPATCH.json")
        staged_controller = stage / "inputs/controller/lay-v10-hardware-b0-b2.py"
        result = ssh(
            [
                "/usr/bin/python3",
                str(staged_controller),
                "remote-b0a-finalize",
                "--stage",
                str(stage),
            ],
            check=False,
        )
        if result.stdout:
            sys.stdout.buffer.write(result.stdout)
        if result.stderr:
            sys.stderr.buffer.write(result.stderr)
        if result.returncode != 0:
            raise GateError(f"B0a finalize failed with {result.returncode}")
    except Exception:
        cleanup = r'''
import pathlib, shutil, sys
p=pathlib.Path(sys.argv[1]); parent=pathlib.Path(sys.argv[2])
if p.parent != parent or not p.name.startswith('b0a-input-closure-v2.stage-'):
    raise SystemExit('unsafe B0a cleanup path')
if p.exists():
    for x in [p, *p.rglob('*')]:
        try: x.chmod(0o755 if x.is_dir() else 0o644)
        except OSError: pass
    shutil.rmtree(p)
'''
        with contextlib.suppress(Exception):
            ssh(["/usr/bin/python3", "-c", cleanup, str(stage), str(REMOTE_B0A.parent)])
        raise


def copy_verified(source: pathlib.Path, destination: pathlib.Path, expected: dict[str, Any]) -> None:
    actual = require_file(
        source,
        sha256=expected["sha256"],
        size=expected["size_bytes"],
        mode=expected["mode"],
    )
    destination.parent.mkdir(parents=True, exist_ok=True)
    shutil.copyfile(source, destination)
    destination.chmod(int(actual["mode"], 8))
    require_file(destination, sha256=actual["sha256"], size=actual["size_bytes"])


def remote_b0a_finalize(stage: pathlib.Path) -> None:
    require(os.uname().nodename == REMOTE_HOSTNAME, "wrong B0a host")
    require(stage.is_dir() and stage.parent == REMOTE_B0A.parent, "invalid B0a stage")
    require(not REMOTE_B0A.exists(), "B0a final path collision")
    require(not REMOTE_STATE.exists(), "B0 state already exists")
    dispatch = load_json(stage / "DISPATCH.json")
    require(dispatch.get("task_id") == TASK_ID, "B0a dispatch task mismatch")
    require(dispatch["pre_b2_perf_executable_invocations"] == 2, "perf ledger mismatch")
    require(dispatch["pre_b2_pmu_measurements"] == 0, "PMU ledger mismatch")
    output_parent = dispatch["remote_probe"]
    require(
        output_parent["requested_output_parent"] == str(REMOTE_PROVENANCE_ROOT),
        "B0a output parent dispatch mismatch",
    )
    require(
        output_parent["nearest_existing_parent"] == str(REMOTE_NEAREST_EXISTING_PARENT),
        "B0a nearest parent dispatch mismatch",
    )
    require(
        output_parent["nearest_existing_parent_device"] == REMOTE_NEAREST_EXISTING_PARENT_DEVICE,
        "B0a parent device dispatch mismatch",
    )
    nearest_metadata = require_directory_device(
        REMOTE_NEAREST_EXISTING_PARENT, REMOTE_NEAREST_EXISTING_PARENT_DEVICE
    )
    require(
        nearest_metadata.st_ino == output_parent["nearest_existing_parent_inode"],
        "B0a nearest parent inode drift",
    )
    require_directory_device(REMOTE_B0A.parent, REMOTE_NEAREST_EXISTING_PARENT_DEVICE)
    require_directory_device(stage, REMOTE_NEAREST_EXISTING_PARENT_DEVICE)
    inputs = stage / "inputs"
    archived = inputs / "slice8b-v10-f6178f"
    correction = inputs / "slice8b-v10-f6178f-correction-v1"
    require_file(
        archived / "artifacts/v13_typed_peak.v10.rs",
        sha256=EXPECTED["v10_source"],
        size=EXPECTED_SIZES["v10_source"],
        mode="0444",
    )
    require_file(
        archived / "artifacts/LAY-L2-RU-FULL-v13.dafsa",
        sha256=EXPECTED["sidecar"],
        mode="0444",
    )
    require_file(correction / "CORRECTION.json", sha256="74f66275a6e1f00b4dedea16ea2b62ab1ad2f4fffa6fada012bca19cc7a1ac90")
    controller = inputs / "controller/lay-v10-hardware-b0-b2.py"
    fragment = inputs / "controller/lay_v10_hardware_test_module.rs.inc"
    cargo_guard = inputs / "controller/cargo-guard.sh"
    require_file(
        controller,
        sha256=dispatch["controller_sha256"],
        mode="0755",
    )
    require(sha256_file(pathlib.Path(__file__).resolve()) == dispatch["controller_sha256"], "executing controller mismatch")
    require_file(fragment, sha256=dispatch["fragment_sha256"], mode="0664")
    require_file(cargo_guard, sha256=dispatch["cargo_guard_sha256"], mode="0775")
    require(dispatch["cargo_guard_sha256"] == EXPECTED["cargo_guard"], "cargo guard dispatch mismatch")
    original_manifest_count = verify_external_sha256sums(
        archived, archived / "manifests/SHA256SUMS"
    )
    correction_manifest_count = verify_external_sha256sums(
        correction, correction / "SHA256SUMS"
    )

    closure_rows = parse_source_closure(archived / "manifests/source-closure-observed.tsv")
    frozen_root = inputs / "surviving-source-closure"
    frozen_root.mkdir(mode=0o700)
    copied = []
    copied_destinations: dict[str, dict[str, Any]] = {}
    for row in closure_rows:
        source = REMOTE_DONOR_ROOT / row["relative"]
        destination = frozen_root / row["relative"]
        reused_destination = row["relative"] in copied_destinations
        if reused_destination:
            previous = copied_destinations[row["relative"]]
            require(
                all(previous[key] == row[key] for key in ("mode", "size_bytes", "sha256", "status")),
                f"source alias identity mismatch: {row['relative']}",
            )
            require_file(
                destination,
                sha256=row["sha256"],
                size=row["size_bytes"],
                mode=row["mode"],
            )
        else:
            copy_verified(source, destination, row)
            copied_destinations[row["relative"]] = row
        copied.append(
            {
                **row,
                "destination": str(destination.relative_to(stage)),
                "reused_normalized_destination": reused_destination,
            }
        )

    v13_copy = inputs / "LAY-L2-RU-FULL-v13.bin"
    copy_verified(
        REMOTE_INSTALLED_V13,
        v13_copy,
        {
            "sha256": EXPECTED["v13_package"],
            "size_bytes": EXPECTED_SIZES["v13_package"],
            "mode": "0644",
        },
    )
    v13_copy.chmod(0o444)
    planned_writes = [
        str(REMOTE_BUILD),
        str(REMOTE_FREEZER),
        str(REMOTE_B0B),
        str(REMOTE_B1),
        str(REMOTE_B2),
        str(REMOTE_STATE),
    ]
    receipt = {
        "schema": "lay.v10.hardware-input-closure.v2",
        "task_id": TASK_ID,
        "host": {
            "hostname": os.uname().nodename,
            "machine_id_sha256": sha256_file(pathlib.Path("/etc/machine-id")),
        },
        "output_parent_identity": {
            "requested_output_parent": output_parent["requested_output_parent"],
            "requested_output_parent_existed_before_mutation": output_parent[
                "requested_output_parent_exists"
            ],
            "nearest_existing_parent_before_mutation": output_parent["nearest_existing_parent"],
            "nearest_existing_parent_device": output_parent[
                "nearest_existing_parent_device"
            ],
            "nearest_existing_parent_inode": output_parent["nearest_existing_parent_inode"],
            "created_stage_parent_device": REMOTE_B0A.parent.stat().st_dev,
            "stage_device": stage.stat().st_dev,
        },
        "frozen_inputs": inventory(inputs),
        "source_closure": {
            "dependency_paths": len(copied),
            "unique_destination_paths": len(copied_destinations),
            "status": "SURVIVING_REMOTE_BYTES_NOT_BUILD_TIME_SNAPSHOT",
            "records": copied,
        },
        "archive_manifests": {
            "original_entries_verified": original_manifest_count,
            "correction_entries_verified": correction_manifest_count,
        },
        "historical_elf": {
            "sha256": EXPECTED["historical_elf"],
            "build_id": EXPECTED["historical_build_id"],
            "class": "ELF64",
            "machine": "x86-64",
            "stripped": True,
            "test": "nanda_wave::l2_field::v13_typed_peak::tests::v10_full_v13_abc_proof",
            "argv": ["--ignored", "--exact", "--nocapture"],
            "permitted_exit_states": ["success_all_conjuncts", "assertion_failure_latency_only"],
        },
        "planned_writes": planned_writes,
        "measurement_lock": str(REMOTE_STATE / "route.lock"),
        "diagnostic_build_status": "NOT_BUILT",
        "query_schedule_status": "NOT_CREATED",
        "pre_b2_perf_executable_invocations": 2,
        "pre_b2_pmu_measurements": 0,
        "full_source_closure": "WATCH",
        "runtime_authority_changed": False,
        "installed_lay_changed": False,
    }
    state_created = False
    try:
        write_new_json(stage / "INPUT_CLOSURE.json", receipt)
        seal_stage(stage)

        REMOTE_STATE.mkdir(parents=True, mode=0o700)
        state_created = True
        write_new_bytes(REMOTE_STATE / "route.lock", f"{TASK_ID}\n".encode(), 0o400)
        markers = REMOTE_STATE / "markers"
        markers.mkdir(mode=0o700)
        write_new_bytes(markers / "build.available", b"one guarded build\n", 0o400)
        write_new_bytes(markers / "freezer.available", b"one unmeasured freezer\n", 0o400)
        atomic_publish(stage, REMOTE_B0A)
    except Exception:
        if stage.exists():
            remove_tree(stage)
        if state_created and not REMOTE_B0A.exists():
            remove_tree(REMOTE_STATE)
        raise
    print(json.dumps({"state": "B0A_PASS_BUILD_UNUSED", "receipt": str(REMOTE_B0A / "INPUT_CLOSURE.json")}))


def consume_marker(name: str, state_root: pathlib.Path = REMOTE_STATE) -> pathlib.Path:
    markers = state_root / "markers"
    available = markers / f"{name}.available"
    consumed = markers / f"{name}.consumed-before-exec"
    require(available.is_file(), f"{name} marker is unavailable or already consumed")
    require(not consumed.exists(), f"{name} marker already consumed")
    os.rename(available, consumed)
    fsync_directory(markers)
    return consumed


@contextlib.contextmanager
def exclusive_route_lock() -> Iterable[None]:
    lock_path = REMOTE_STATE / "route.lock"
    require_file(lock_path, mode="0400")
    descriptor = os.open(lock_path, os.O_RDONLY)
    try:
        try:
            fcntl.flock(descriptor, fcntl.LOCK_EX | fcntl.LOCK_NB)
        except BlockingIOError as error:
            raise GateError("another B0-B2 owner currently holds the route lock") from error
        yield
    finally:
        with contextlib.suppress(OSError):
            fcntl.flock(descriptor, fcntl.LOCK_UN)
        os.close(descriptor)


def assemble_diagnostic_source(v10: bytes, fragment: bytes) -> bytes:
    require(sha256_bytes(v10) == EXPECTED["v10_source"], "V10 source identity mismatch")
    require(v10.endswith(b"}\n"), "V10 source terminal brace mismatch")
    require(fragment.startswith(b"\n    const HARDWARE_SCHEDULE_SCHEMA"), "fragment prefix mismatch")
    final = v10[:-2] + fragment + b"}\n"
    require(final[:39_047] == v10[:39_047], "production prefix changed")
    require(sha256_bytes(final[:39_047]) == EXPECTED["production_prefix"], "production prefix SHA mismatch")
    return final


def find_test_executable(target: pathlib.Path) -> pathlib.Path:
    candidates = []
    for path in (target / "release/deps").glob("lay-*"):
        if path.is_file() and os.access(path, os.X_OK) and not path.name.endswith((".d", ".rlib", ".rmeta")):
            with path.open("rb") as source:
                if source.read(4) == b"\x7fELF":
                    candidates.append(path)
    require(len(candidates) == 1, f"expected one release test ELF, found {candidates}")
    return candidates[0]


def elf_build_id(path: pathlib.Path) -> str:
    result = run(["readelf", "-n", str(path)])
    match = re.search(rb"Build ID:\s*([0-9a-f]+)", result.stdout)
    require(match is not None, f"ELF has no Build ID: {path}")
    return match.group(1).decode()


def verify_remote_machine_identity() -> dict[str, str]:
    identity = {
        "hostname": os.uname().nodename,
        "machine_id_sha256": sha256_file(pathlib.Path("/etc/machine-id")),
    }
    require(identity["hostname"] == REMOTE_HOSTNAME, "remote hostname drift")
    require(
        identity["machine_id_sha256"] == REMOTE_MACHINE_ID_SHA256,
        "remote machine identity drift",
    )
    return identity


def verify_build_prerequisites(workspace: pathlib.Path) -> dict[str, Any]:
    guard = workspace / "scripts/cargo-guard.sh"
    require_file(guard, sha256=EXPECTED["cargo_guard"], mode="0775")
    environment = controlled_environment()
    environment.update(
        {
            "CARGO_INCREMENTAL": "0",
            "CARGO_TARGET_DIR": str(workspace / "target"),
            "RUSTFLAGS": "",
        }
    )
    cargo = run(["cargo", "-V"], env=environment).stdout.decode().strip()
    rustc = run(["rustc", "-Vv"], env=environment).stdout.decode().strip()
    cfg = run(["rustc", "--print", "cfg"], env=environment).stdout.decode().splitlines()
    features = {
        match.group(1)
        for line in cfg
        if (match := re.fullmatch(r'target_feature="([^"]+)"', line)) is not None
    }
    require(cargo == EXPECTED_CARGO_VERSION, f"Cargo toolchain drift: {cargo}")
    for expected in (
        EXPECTED_RUSTC_RELEASE,
        EXPECTED_RUSTC_COMMIT,
        EXPECTED_RUSTC_HOST,
        EXPECTED_LLVM_VERSION,
    ):
        require(expected in rustc, f"rustc toolchain drift: missing {expected}")
    require(REQUIRED_TARGET_FEATURES <= features, f"target feature drift: {sorted(features)}")
    guard_status = run([str(guard), "--status"], cwd=workspace, env=environment)
    disk = shutil.disk_usage(REMOTE_PARENT)
    require(
        disk.free >= MIN_BUILD_FREE_BYTES,
        f"insufficient isolated build headroom: {disk.free} < {MIN_BUILD_FREE_BYTES}",
    )
    return {
        "host": verify_remote_machine_identity(),
        "cargo": cargo,
        "rustc_vv": rustc,
        "target_features": sorted(features),
        "required_target_features": sorted(REQUIRED_TARGET_FEATURES),
        "guard_status": guard_status.stdout.decode().strip(),
        "cargo_incremental": "0",
        "rustflags": "",
        "default_features": True,
        "minimum_free_bytes": MIN_BUILD_FREE_BYTES,
        "observed_free_bytes": disk.free,
        "target_budget_bytes": 12_884_901_888,
    }


def remote_build() -> None:
    require(REMOTE_B0A.is_dir(), "B0a closure is absent")
    require(not REMOTE_BUILD.exists(), "diagnostic build final already exists")
    verify_sha256sums(REMOTE_B0A)
    b0a_receipt = load_json(REMOTE_B0A / "INPUT_CLOSURE.json")
    require(b0a_receipt.get("task_id") == TASK_ID, "B0a task identity mismatch")
    require(b0a_receipt.get("diagnostic_build_status") == "NOT_BUILT", "B0a build status mismatch")
    require((REMOTE_STATE / "markers/build.available").is_file(), "build marker is unavailable")
    require((REMOTE_STATE / "markers/freezer.available").is_file(), "freezer marker changed before build")
    stage = pathlib.Path(f"{REMOTE_BUILD}.stage-{os.getpid()}-{time.time_ns()}")
    workspace = REMOTE_PARENT / f"diagnostic-workspace-{os.getpid()}-{time.time_ns()}"
    stage.mkdir(mode=0o700)
    workspace.mkdir(mode=0o700)
    log_path = stage / "cargo.log"
    marker_consumed = False
    try:
        source_root = REMOTE_B0A / "inputs/surviving-source-closure"
        shutil.copytree(source_root, workspace, dirs_exist_ok=True)
        make_tree_writable(workspace)
        shutil.copyfile(REMOTE_B0A / "inputs/slice8b-v10-f6178f/artifacts/Cargo.toml", workspace / "Cargo.toml")
        shutil.copyfile(REMOTE_B0A / "inputs/slice8b-v10-f6178f/artifacts/Cargo.lock", workspace / "Cargo.lock")
        (workspace / "scripts").mkdir(exist_ok=True)
        shutil.copyfile(REMOTE_B0A / "inputs/controller/cargo-guard.sh", workspace / "scripts/cargo-guard.sh")
        (workspace / "scripts/cargo-guard.sh").chmod(0o775)
        v10_path = REMOTE_B0A / "inputs/slice8b-v10-f6178f/artifacts/v13_typed_peak.v10.rs"
        fragment_path = REMOTE_B0A / "inputs/controller/lay_v10_hardware_test_module.rs.inc"
        final_source = assemble_diagnostic_source(v10_path.read_bytes(), fragment_path.read_bytes())
        source_path = workspace / "src/nanda_wave/l2_field/v13_typed_peak.rs"
        source_path.parent.mkdir(parents=True, exist_ok=True)
        source_path.write_bytes(final_source)
        source_path.chmod(0o444)
        source_inventory_before = inventory(workspace)
        prerequisites = verify_build_prerequisites(workspace)
        write_new_bytes(stage / "diagnostic-source.rs", final_source, 0o444)
        write_new_json(
            stage / "PREBUILD_PROVENANCE.json",
            {
                "schema": "lay.v10.hardware-diagnostic-prebuild.v1",
                "task_id": TASK_ID,
                "recovered_v10_sha256": EXPECTED["v10_source"],
                "production_prefix_bytes": 39_047,
                "production_prefix_sha256": EXPECTED["production_prefix"],
                "test_module_fragment_sha256": sha256_file(fragment_path),
                "final_source_sha256": sha256_bytes(final_source),
                "source_inventory_sha256": sha256_bytes(
                    canonical_json_bytes(source_inventory_before)
                ),
                "prerequisites": prerequisites,
                "cargo_started_when_receipt_written": False,
            },
        )
        try:
            consume_marker("build")
        finally:
            marker_consumed = (
                REMOTE_STATE / "markers/build.consumed-before-exec"
            ).is_file()
        environment = controlled_environment()
        environment.update(
            {
                "CARGO_BUILD_JOBS": "20",
                "CARGO_INCREMENTAL": "0",
                "CARGO_NET_OFFLINE": "true",
                "CARGO_TARGET_DIR": str(workspace / "target"),
                "RUSTFLAGS": "",
            }
        )
        command = [
            str(workspace / "scripts/cargo-guard.sh"),
            "test",
            "--offline",
            "--locked",
            "--release",
            "--lib",
            "--no-run",
            "nanda_wave::l2_field::v13_typed_peak::tests",
        ]
        with log_path.open("wb") as log:
            result = run(command, cwd=workspace, env=environment, stdout=log, stderr=subprocess.STDOUT, check=False)
            log.flush()
            os.fsync(log.fileno())
        require(result.returncode == 0, f"guarded diagnostic build failed with {result.returncode}")
        executable = find_test_executable(workspace / "target")
        evidence_source = stage / "source"
        shutil.copytree(workspace, evidence_source, ignore=shutil.ignore_patterns("target"))
        shutil.copyfile(executable, stage / "diagnostic-test-elf")
        (stage / "diagnostic-test-elf").chmod(0o555)
        provenance = {
            "schema": "lay.v10.hardware-diagnostic-executable-provenance.v1",
            "task_id": TASK_ID,
            "source_identity": {
                "recovered_v10_sha256": EXPECTED["v10_source"],
                "production_prefix_bytes": 39_047,
                "production_prefix_sha256": EXPECTED["production_prefix"],
                "test_module_fragment_sha256": sha256_file(fragment_path),
                "final_source_sha256": sha256_bytes(final_source),
                "surviving_source_inventory_sha256": sha256_bytes(canonical_json_bytes(source_inventory_before)),
                "full_source_closure": "WATCH",
            },
            "build": {
                "hostname": os.uname().nodename,
                "command": command,
                "cargo_incremental": "0",
                "cargo_net_offline": "true",
                "rustflags": "",
                "profile": "release",
                "default_features": True,
                "build_marker_consumed_before_cargo": True,
                "retry_permitted": False,
                "prerequisites": prerequisites,
            },
            "executable": {
                "sha256": sha256_file(stage / "diagnostic-test-elf"),
                "size_bytes": (stage / "diagnostic-test-elf").stat().st_size,
                "build_id": elf_build_id(stage / "diagnostic-test-elf"),
                "test_entrypoints": TEST_NAMES,
            },
            "executed": False,
            "perf_invoked": False,
            "pmu_event_opened": False,
            "runtime_authority_changed": False,
            "installed_lay_changed": False,
        }
        write_new_json(stage / "EXECUTABLE_PROVENANCE.json", provenance)
        seal_stage(stage)
        atomic_publish(stage, REMOTE_BUILD)
        print(json.dumps({"state": "DIAGNOSTIC_EXECUTABLE_SEALED_BUILD_CONSUMED", "receipt": str(REMOTE_BUILD / "EXECUTABLE_PROVENANCE.json")}))
    except Exception as error:
        if not marker_consumed:
            remove_tree(stage)
            raise
        failure = {
            "schema": "lay.v10.hardware-diagnostic-build-failure.v1",
            "error": str(error),
            "build_marker_consumed": True,
            "retry_permitted": False,
            "runtime_authority_changed": False,
            "installed_lay_changed": False,
        }
        with contextlib.suppress(Exception):
            write_new_json(stage / "FAILURE.json", failure)
            seal_stage(stage)
            failed = pathlib.Path(f"{REMOTE_BUILD}.failed-{time.time_ns()}")
            atomic_publish(stage, failed)
        raise
    finally:
        with contextlib.suppress(Exception):
            remove_tree(workspace)


def execute_test(
    executable: pathlib.Path,
    test_name: str,
    environment: dict[str, str],
    stdout_path: pathlib.Path,
    stderr_path: pathlib.Path,
) -> int:
    with stdout_path.open("xb") as stdout, stderr_path.open("xb") as stderr:
        process = subprocess.run(
            [str(executable), "--ignored", "--exact", test_name, "--nocapture"],
            env=environment,
            stdout=stdout,
            stderr=stderr,
            check=False,
        )
        stdout.flush()
        stderr.flush()
        os.fsync(stdout.fileno())
        os.fsync(stderr.fileno())
    return process.returncode


def remote_freezer() -> None:
    require(REMOTE_BUILD.is_dir(), "sealed diagnostic build is absent")
    require(not REMOTE_FREEZER.exists(), "freezer final already exists")
    verify_sha256sums(REMOTE_BUILD)
    require(
        (REMOTE_STATE / "markers/build.consumed-before-exec").is_file(),
        "build marker was not consumed",
    )
    require((REMOTE_STATE / "markers/freezer.available").is_file(), "freezer marker is unavailable")
    stage = pathlib.Path(f"{REMOTE_FREEZER}.stage-{os.getpid()}-{time.time_ns()}")
    stage.mkdir(mode=0o700)
    marker_consumed = False
    try:
        executable = REMOTE_BUILD / "diagnostic-test-elf"
        executable_provenance = load_json(REMOTE_BUILD / "EXECUTABLE_PROVENANCE.json")
        require(
            executable_provenance.get("executed") is False,
            "diagnostic executable provenance already reports execution",
        )
        require_file(
            executable,
            sha256=executable_provenance["executable"]["sha256"],
            size=executable_provenance["executable"]["size_bytes"],
            mode="0555",
        )
        require(
            elf_build_id(executable) == executable_provenance["executable"]["build_id"],
            "diagnostic executable Build ID drift",
        )
        schedule = stage / "query-schedule.json"
        receipt = stage / "FREEZER_RECEIPT.json"
        environment = controlled_environment()
        environment.update(
            {
                "LAY_V10_HW_V7": str(REMOTE_B0A / "inputs/slice8b-v10-f6178f/artifacts/slice8b-v7-fixed-13x100.json"),
                "LAY_V10_HW_FREEZER_OUTPUT": str(schedule),
                "LAY_V10_HW_FREEZER_RECEIPT": str(receipt),
            }
        )
        try:
            consume_marker("freezer")
        finally:
            marker_consumed = (
                REMOTE_STATE / "markers/freezer.consumed-before-exec"
            ).is_file()
        exit_code = execute_test(
            executable,
            TEST_NAMES["freezer"],
            environment,
            stage / "stdout.log",
            stage / "stderr.log",
        )
        require(exit_code == 0, f"schedule freezer exited {exit_code}")
        freezer = load_json(receipt)
        require(freezer.get("entries") == 382, "freezer did not emit 382 entries")
        require(freezer.get("perf_invoked") is False, "freezer invoked perf")
        require(freezer.get("pmu_event_opened") is False, "freezer opened PMU")
        require(freezer.get("schedule_sha256") == sha256_file(schedule), "freezer schedule SHA mismatch")
        wrapper = {
            "schema": "lay.v10.hardware-schedule-freezer-wrapper.v1",
            "task_id": TASK_ID,
            "executable_sha256": sha256_file(executable),
            "test": TEST_NAMES["freezer"],
            "argv": ["--ignored", "--exact", TEST_NAMES["freezer"], "--nocapture"],
            "exit_code": exit_code,
            "schedule_sha256": sha256_file(schedule),
            "freezer_marker_consumed_before_exec": True,
            "retry_permitted": False,
            "perf_invoked": False,
            "pmu_event_opened": False,
            "runtime_authority_changed": False,
            "installed_lay_changed": False,
        }
        write_new_json(stage / "WRAPPER_RECEIPT.json", wrapper)
        seal_stage(stage)
        atomic_publish(stage, REMOTE_FREEZER)
        print(json.dumps({"state": "FREEZER_OUTPUT_STAGED_EXECUTABLE_UNCHANGED", "receipt": str(REMOTE_FREEZER / "WRAPPER_RECEIPT.json")}))
    except Exception as error:
        if not marker_consumed:
            remove_tree(stage)
            raise
        with contextlib.suppress(Exception):
            write_new_json(
                stage / "FAILURE.json",
                {
                    "schema": "lay.v10.hardware-schedule-freezer-failure.v1",
                    "error": str(error),
                    "freezer_marker_consumed": marker_consumed,
                    "retry_permitted": False,
                    "runtime_authority_changed": False,
                    "installed_lay_changed": False,
                },
            )
            seal_stage(stage)
            atomic_publish(stage, pathlib.Path(f"{REMOTE_FREEZER}.failed-{time.time_ns()}"))
        raise


def recursive_forbidden_target(value: Any, path: str = "$") -> None:
    if isinstance(value, dict):
        for key, child in value.items():
            require("target" not in key.lower(), f"forbidden schedule key {path}.{key}")
            recursive_forbidden_target(child, f"{path}.{key}")
    elif isinstance(value, list):
        for index, child in enumerate(value):
            recursive_forbidden_target(child, f"{path}[{index}]")


def validate_schedule_python(
    schedule: Any,
    *,
    v7_path: pathlib.Path | None = None,
    phase7d_semantics_sha256: str | None = None,
) -> None:
    recursive_forbidden_target(schedule)
    require(set(schedule) == {"schema", "source", "partition", "entries"}, "unexpected schedule top-level fields")
    require(schedule.get("schema") == "lay.v10.hardware-query-schedule.v1", "schedule schema mismatch")
    source = schedule.get("source")
    require(
        isinstance(source, dict)
        and set(source)
        == {
            "v7_sha256",
            "phase7d_semantics_sha256",
            "composite_identity_encoding",
            "damaged_surface_encoding",
            "retrieval_lanes_encoding",
        },
        "schedule source fields mismatch",
    )
    require(source["v7_sha256"] == EXPECTED["v7"], "schedule V7 mismatch")
    if phase7d_semantics_sha256 is not None:
        require(
            source["phase7d_semantics_sha256"] == phase7d_semantics_sha256,
            "Phase7D semantics digest mismatch",
        )
    require(
        source["composite_identity_encoding"]
        == "sha256(class_utf8 || NUL || proof_identity_json_utf8)",
        "composite identity encoding mismatch",
    )
    require(source["damaged_surface_encoding"] == "sha256(utf8)", "damaged-surface encoding mismatch")
    require(
        source["retrieval_lanes_encoding"]
        == "sha256(compact_json([{symbols,maximum_levenshtein_distance}]))",
        "retrieval-lane encoding mismatch",
    )
    require(
        schedule.get("partition")
        == {"b5_requests": 382, "b6_workers": 20, "b6_chunk_size": 20},
        "schedule partition contract mismatch",
    )
    entries = schedule.get("entries")
    require(isinstance(entries, list) and len(entries) == 382, "schedule entry count mismatch")
    identities = set()
    for ordinal, entry in enumerate(entries):
        require(
            set(entry)
            == {
                "source_ordinal",
                "composite_identity_sha256",
                "damaged_surface_sha256",
                "retrieval_lanes_sha256",
                "b5_ordinal",
                "b6_worker",
                "b6_chunk_start",
                "b6_chunk_end_exclusive",
            },
            f"unexpected schedule entry fields at {ordinal}",
        )
        require(entry.get("source_ordinal") == ordinal, f"source ordinal mismatch at {ordinal}")
        require(entry.get("b5_ordinal") == ordinal, f"B5 ordinal mismatch at {ordinal}")
        worker = ordinal // 20
        require(entry.get("b6_worker") == worker, f"B6 worker mismatch at {ordinal}")
        require(entry.get("b6_chunk_start") == worker * 20, f"B6 start mismatch at {ordinal}")
        require(entry.get("b6_chunk_end_exclusive") == min(worker * 20 + 20, 382), f"B6 end mismatch at {ordinal}")
        identity = entry.get("composite_identity_sha256")
        require(isinstance(identity, str) and re.fullmatch(r"[0-9a-f]{64}", identity) is not None, "invalid composite identity")
        require(identity not in identities, "duplicate composite identity")
        identities.add(identity)
        for key in ("damaged_surface_sha256", "retrieval_lanes_sha256"):
            require(re.fullmatch(r"[0-9a-f]{64}", str(entry.get(key))) is not None, f"invalid {key}")
    require([sum(1 for entry in entries if entry["b6_worker"] == worker) for worker in range(20)] == [20] * 19 + [2], "B6 partition mismatch")
    if v7_path is not None:
        source_v7 = load_json(v7_path)
        records = source_v7.get("live_cohort_compare_shadow", {}).get("no_field_records")
        require(isinstance(records, list) and len(records) == 382, "V7 target audit denominator mismatch")
        forbidden_values = set()
        for record in records:
            target = record.get("target_surface")
            require(isinstance(target, str), "V7 target audit field missing")
            forbidden_values.add(target)
            forbidden_values.add(sha256_bytes(target.encode()))
        for value in schedule_values(schedule):
            require(value not in forbidden_values, "target-derived schedule value detected")


def schedule_values(value: Any) -> Iterable[str]:
    if isinstance(value, dict):
        for child in value.values():
            yield from schedule_values(child)
    elif isinstance(value, list):
        for child in value:
            yield from schedule_values(child)
    elif isinstance(value, str):
        yield value


def remote_b0b() -> None:
    require(REMOTE_FREEZER.is_dir(), "freezer output is absent")
    require(not REMOTE_B0B.exists(), "B0b final already exists")
    verify_sha256sums(REMOTE_FREEZER)
    stage = pathlib.Path(f"{REMOTE_B0B}.stage-{os.getpid()}-{time.time_ns()}")
    stage.mkdir(mode=0o700)
    source_schedule = REMOTE_FREEZER / "query-schedule.json"
    schedule = load_json(source_schedule)
    phase7d_source = (
        REMOTE_B0A
        / "inputs/surviving-source-closure/src/nanda_wave/lexical_grokking/typed_edit_traversal.rs"
    )
    phase7d_semantics_sha256 = sha256_file(phase7d_source)
    v7_path = (
        REMOTE_B0A
        / "inputs/slice8b-v10-f6178f/artifacts/slice8b-v7-fixed-13x100.json"
    )
    validate_schedule_python(
        schedule,
        v7_path=v7_path,
        phase7d_semantics_sha256=phase7d_semantics_sha256,
    )
    freezer = load_json(REMOTE_FREEZER / "FREEZER_RECEIPT.json")
    executable_provenance = load_json(REMOTE_BUILD / "EXECUTABLE_PROVENANCE.json")
    schedule_sha256 = sha256_file(source_schedule)
    require(freezer.get("schedule_sha256") == schedule_sha256, "freezer binding mismatch")
    verify_sha256sums(REMOTE_FREEZER)
    verify_sha256sums(REMOTE_BUILD)
    verify_sha256sums(REMOTE_B0A)
    require_file(
        REMOTE_BUILD / "diagnostic-test-elf",
        sha256=executable_provenance["executable"]["sha256"],
        size=executable_provenance["executable"]["size_bytes"],
        mode="0555",
    )
    shutil.copyfile(source_schedule, stage / "query-schedule.json")
    closure = {
        "schema": "lay.v10.hardware-query-schedule-closure.v1",
        "task_id": TASK_ID,
        "schedule_schema": schedule["schema"],
        "schedule_sha256": schedule_sha256,
        "entries": 382,
        "unique_composite_identities": 382,
        "v7_sha256": EXPECTED["v7"],
        "phase7d_semantics_sha256": phase7d_semantics_sha256,
        "production_prefix_sha256": EXPECTED["production_prefix"],
        "b0a_closure_sha256": sha256_file(REMOTE_B0A / "INPUT_CLOSURE.json"),
        "diagnostic_executable_sha256": executable_provenance["executable"]["sha256"],
        "diagnostic_executable_build_id": executable_provenance["executable"]["build_id"],
        "freezer_receipt_sha256": sha256_file(REMOTE_FREEZER / "FREEZER_RECEIPT.json"),
        "target_fields_present": False,
        "build_executed_by_b0b": False,
        "code_executed_by_b0b": False,
        "runtime_authority_changed": False,
        "installed_lay_changed": False,
    }
    write_new_json(stage / "SCHEDULE_CLOSURE.json", closure)
    seal_stage(stage)
    atomic_publish(stage, REMOTE_B0B)
    print(json.dumps({"state": "B0B_PASS_SCHEDULE_SEALED", "receipt": str(REMOTE_B0B / "SCHEDULE_CLOSURE.json")}))


def read_optional(path: pathlib.Path) -> str | None:
    try:
        return path.read_text(encoding="utf-8", errors="replace").strip()
    except (FileNotFoundError, PermissionError, OSError):
        return None


def parse_cpu_list(raw: str) -> list[int]:
    values: list[int] = []
    for item in raw.strip().split(","):
        if not item:
            continue
        if "-" in item:
            start, end = (int(value) for value in item.split("-", 1))
            values.extend(range(start, end + 1))
        else:
            values.append(int(item))
    return sorted(set(values))


def pressure(path: pathlib.Path) -> dict[str, dict[str, float]]:
    result: dict[str, dict[str, float]] = {}
    text = read_optional(path) or ""
    for line in text.splitlines():
        fields = line.split()
        result[fields[0]] = {
            key: float(value)
            for key, value in (field.split("=", 1) for field in fields[1:])
        }
    return result


def process_snapshot() -> dict[int, dict[str, Any]]:
    clock_ticks = os.sysconf(os.sysconf_names["SC_CLK_TCK"])
    pagesize = os.sysconf("SC_PAGE_SIZE")
    result: dict[int, dict[str, Any]] = {}
    for directory in pathlib.Path("/proc").iterdir():
        if not directory.name.isdigit():
            continue
        try:
            stat_text = (directory / "stat").read_text()
            stat_tail = stat_text[stat_text.rfind(")") + 2 :].split()
            command = (directory / "comm").read_text().strip()
            cmdline = (directory / "cmdline").read_bytes().replace(b"\0", b" ").decode(errors="replace").strip()
            status = (directory / "status").read_text()
            rss_match = re.search(r"^VmRSS:\s+(\d+)", status, re.MULTILINE)
            result[int(directory.name)] = {
                "comm": command,
                "cmdline": cmdline,
                "ppid": int(stat_tail[1]),
                "cpu_seconds": (int(stat_tail[11]) + int(stat_tail[12])) / clock_ticks,
                "rss_bytes": int(rss_match.group(1)) * 1024 if rss_match else int(stat_tail[21]) * pagesize,
            }
        except (FileNotFoundError, ProcessLookupError, PermissionError, IndexError, ValueError):
            continue
    return result


def cpu_topology() -> list[dict[str, Any]]:
    online = parse_cpu_list(read_optional(pathlib.Path("/sys/devices/system/cpu/online")) or "")
    records = []
    for cpu in online:
        root = pathlib.Path(f"/sys/devices/system/cpu/cpu{cpu}")
        topology = root / "topology"
        caches = []
        for cache in sorted((root / "cache").glob("index*")):
            caches.append(
                {
                    "level": read_optional(cache / "level"),
                    "type": read_optional(cache / "type"),
                    "size": read_optional(cache / "size"),
                    "shared_cpu_list": read_optional(cache / "shared_cpu_list"),
                }
            )
        records.append(
            {
                "cpu": cpu,
                "package_id": read_optional(topology / "physical_package_id"),
                "core_id": read_optional(topology / "core_id"),
                "thread_siblings_list": read_optional(topology / "thread_siblings_list"),
                "core_type": read_optional(topology / "core_type"),
                "node": next((path.name for path in root.glob("node[0-9]*")), None),
                "caches": caches,
            }
        )
    return records


def keyed_integer_file(path: pathlib.Path, keys: set[str]) -> dict[str, int]:
    values: dict[str, int] = {}
    for line in (read_optional(path) or "").splitlines():
        fields = line.split()
        if len(fields) == 2 and fields[0] in keys and fields[1].lstrip("-").isdigit():
            values[fields[0]] = int(fields[1])
    return values


def vulnerability_snapshot() -> dict[str, str | None]:
    root = pathlib.Path("/sys/devices/system/cpu/vulnerabilities")
    return {path.name: read_optional(path) for path in sorted(root.glob("*")) if path.is_file()}


def powercap_snapshot() -> list[dict[str, Any]]:
    records = []
    for zone in sorted(pathlib.Path("/sys/class/powercap").glob("intel-rapl*")):
        if not zone.is_dir():
            continue
        constraints = {}
        for path in sorted(zone.glob("constraint_*")):
            constraints[path.name] = read_optional(path)
        records.append(
            {
                "zone": zone.name,
                "name": read_optional(zone / "name"),
                "enabled": read_optional(zone / "enabled"),
                "max_energy_range_uj": read_optional(zone / "max_energy_range_uj"),
                "constraints": constraints,
            }
        )
    return records


def interrupt_snapshot() -> dict[str, Any]:
    path = pathlib.Path("/proc/interrupts")
    try:
        data = path.read_bytes()
    except (FileNotFoundError, PermissionError, OSError):
        return {"sha256": None, "per_cpu_totals": []}
    lines = data.decode(errors="replace").splitlines()
    cpu_count = len(lines[0].split()) if lines else 0
    totals = [0] * cpu_count
    for line in lines[1:]:
        fields = line.split()
        for index, value in enumerate(fields[1 : 1 + cpu_count]):
            if value.isdigit():
                totals[index] += int(value)
    return {"sha256": sha256_bytes(data), "per_cpu_totals": totals}


def frequency_policy() -> list[dict[str, Any]]:
    records = []
    for policy in sorted(pathlib.Path("/sys/devices/system/cpu/cpufreq").glob("policy*")):
        records.append(
            {
                "policy": policy.name,
                "affected_cpus": read_optional(policy / "affected_cpus"),
                "scaling_driver": read_optional(policy / "scaling_driver"),
                "scaling_governor": read_optional(policy / "scaling_governor"),
                "energy_performance_preference": read_optional(policy / "energy_performance_preference"),
                "scaling_min_freq": read_optional(policy / "scaling_min_freq"),
                "scaling_max_freq": read_optional(policy / "scaling_max_freq"),
                "scaling_cur_freq": read_optional(policy / "scaling_cur_freq"),
            }
        )
    return records


def thermal_snapshot() -> dict[str, Any]:
    temperatures = []
    for path in sorted(pathlib.Path("/sys/class/hwmon").glob("hwmon*/temp*_input")):
        raw = read_optional(path)
        if raw and raw.lstrip("-").isdigit():
            temperatures.append({"path": str(path), "millidegrees_c": int(raw)})
    throttles = {}
    for path in sorted(pathlib.Path("/sys/devices/system/cpu").glob("cpu*/thermal_throttle/*")):
        raw = read_optional(path)
        if raw and raw.isdigit():
            throttles[str(path)] = int(raw)
    return {"temperatures": temperatures, "throttle_counters": throttles}


def environment_snapshot() -> dict[str, Any]:
    processes = process_snapshot()
    meminfo = {}
    for line in (read_optional(pathlib.Path("/proc/meminfo")) or "").splitlines():
        key, value = line.split(":", 1)
        if key in {"MemAvailable", "Cached", "SwapTotal", "SwapFree"}:
            meminfo[key] = value.strip()
    return {
        "identity": {
            "hostname": os.uname().nodename,
            "machine_id_sha256": sha256_file(pathlib.Path("/etc/machine-id")),
            "boot_id": read_optional(pathlib.Path("/proc/sys/kernel/random/boot_id")),
            "kernel_release": os.uname().release,
            "kernel_command_line": read_optional(pathlib.Path("/proc/cmdline")),
            "microcode": read_optional(pathlib.Path("/sys/devices/system/cpu/cpu0/microcode/version")),
            "dmi_product": read_optional(pathlib.Path("/sys/class/dmi/id/product_name")),
            "bios_version": read_optional(pathlib.Path("/sys/class/dmi/id/bios_version")),
            "bios_date": read_optional(pathlib.Path("/sys/class/dmi/id/bios_date")),
        },
        "topology": {
            "online": read_optional(pathlib.Path("/sys/devices/system/cpu/online")),
            "present": read_optional(pathlib.Path("/sys/devices/system/cpu/present")),
            "cpus": cpu_topology(),
            "numa_online": read_optional(pathlib.Path("/sys/devices/system/node/online")),
            "smt_active": read_optional(pathlib.Path("/sys/devices/system/cpu/smt/active")),
        },
        "frequency_policy": frequency_policy(),
        "pstate": {
            key: read_optional(pathlib.Path("/sys/devices/system/cpu/intel_pstate") / key)
            for key in ("status", "min_perf_pct", "max_perf_pct", "no_turbo")
        },
        "boost": read_optional(pathlib.Path("/sys/devices/system/cpu/cpufreq/boost")),
        "vulnerabilities": vulnerability_snapshot(),
        "thermal": thermal_snapshot(),
        "powercap": powercap_snapshot(),
        "pressure": {
            "cpu": pressure(pathlib.Path("/proc/pressure/cpu")),
            "memory": pressure(pathlib.Path("/proc/pressure/memory")),
            "io": pressure(pathlib.Path("/proc/pressure/io")),
        },
        "loadavg": read_optional(pathlib.Path("/proc/loadavg")),
        "uptime": read_optional(pathlib.Path("/proc/uptime")),
        "meminfo": meminfo,
        "proc_stat": keyed_integer_file(
            pathlib.Path("/proc/stat"), {"procs_running", "procs_blocked"}
        ),
        "vmstat": keyed_integer_file(
            pathlib.Path("/proc/vmstat"), {"pswpin", "pswpout", "pgmajfault", "oom_kill"}
        ),
        "processes": [
            {"pid": pid, **record}
            for pid, record in sorted(processes.items(), key=lambda item: item[1]["rss_bytes"], reverse=True)[:40]
        ],
        "top_cpu_processes": [
            {"pid": pid, **record}
            for pid, record in sorted(
                processes.items(), key=lambda item: item[1]["cpu_seconds"], reverse=True
            )[:40]
        ],
        "competing_processes": [
            {"pid": pid, **record}
            for pid, record in sorted(processes.items())
            if record["comm"] in {"cargo", "rustc", "perf"}
            or "v10_full_v13_abc_proof" in record["cmdline"]
            or "diagnostic-test-elf" in record["cmdline"]
        ],
        "irqbalance_active": any(record["comm"] == "irqbalance" for record in processes.values()),
        "interrupts": interrupt_snapshot(),
    }


def stable_environment_projection(snapshot: dict[str, Any]) -> dict[str, Any]:
    frequency_policy = [
        {key: value for key, value in policy.items() if key != "scaling_cur_freq"}
        for policy in snapshot["frequency_policy"]
    ]
    return {
        "identity": snapshot["identity"],
        "topology": snapshot["topology"],
        "frequency_policy": frequency_policy,
        "pstate": snapshot["pstate"],
        "boost": snapshot["boost"],
        "vulnerabilities": snapshot["vulnerabilities"],
        "powercap": snapshot["powercap"],
    }


def environment_admission(snapshot: dict[str, Any]) -> list[str]:
    failures = []
    identity = snapshot["identity"]
    if identity["hostname"] != REMOTE_HOSTNAME or identity["machine_id_sha256"] != REMOTE_MACHINE_ID_SHA256:
        failures.append("host_identity")
    cpus = snapshot["topology"]["cpus"]
    if len(cpus) != 20:
        failures.append("online_cpu_count")
    cpu_some = snapshot["pressure"]["cpu"].get("some", {}).get("avg10", float("inf"))
    memory_full = snapshot["pressure"]["memory"].get("full", {}).get("avg10", float("inf"))
    io_full = snapshot["pressure"]["io"].get("full", {}).get("avg10", float("inf"))
    if cpu_some > 2.0:
        failures.append("cpu_psi")
    if memory_full > 0.10:
        failures.append("memory_psi")
    if io_full > 0.10:
        failures.append("io_psi")
    temperatures = [value["millidegrees_c"] for value in snapshot["thermal"]["temperatures"]]
    if not temperatures:
        failures.append("temperature_unobservable")
    elif max(temperatures) >= 90_000:
        failures.append("temperature")
    for process in snapshot["competing_processes"]:
        command = process["comm"]
        cmdline = process["cmdline"]
        if command in {"cargo", "rustc", "perf"} or "v10_full_v13_abc_proof" in cmdline:
            failures.append(f"competing_process:{process['pid']}:{command}")
    return failures


def quiet_sample() -> tuple[dict[str, Any], list[str]]:
    before = process_snapshot()
    before_time = time.monotonic()
    time.sleep(0.25)
    after_time = time.monotonic()
    after = process_snapshot()
    interval = after_time - before_time
    busy = []
    for pid, current in after.items():
        if pid == os.getpid() or pid not in before:
            continue
        if not current["cmdline"]:
            continue
        cpu_percent = (current["cpu_seconds"] - before[pid]["cpu_seconds"]) / interval * 100.0
        if cpu_percent > 1.0:
            busy.append(f"{pid}:{current['comm']}:{cpu_percent:.3f}")
    snapshot = environment_snapshot()
    running = snapshot["proc_stat"].get("procs_running", 999)
    failures = environment_admission(snapshot)
    if running > 2:
        failures.append(f"procs_running:{running}")
    if busy:
        failures.append("busy_processes:" + ",".join(busy))
    return {"snapshot": snapshot, "procs_running": running, "busy_processes": busy}, failures


def remote_b1() -> None:
    require(REMOTE_B0B.is_dir(), "B0b closure is absent")
    require(not REMOTE_B1.exists(), "B1 final already exists")
    verify_sha256sums(REMOTE_B0B)
    stage = pathlib.Path(f"{REMOTE_B1}.stage-{os.getpid()}-{time.time_ns()}")
    stage.mkdir(mode=0o700)
    deadline = time.monotonic() + 120.0
    samples = []
    failures: list[str] = ["not_sampled"]
    while time.monotonic() < deadline:
        samples = []
        all_failures = []
        for _ in range(3):
            sample, sample_failures = quiet_sample()
            samples.append(sample)
            all_failures.extend(sample_failures)
        failures = sorted(set(all_failures))
        if not failures:
            break
    accepted = not failures
    first = samples[0]["snapshot"] if samples else environment_snapshot()
    last = samples[-1]["snapshot"] if samples else first
    if stable_environment_projection(first) != stable_environment_projection(last):
        failures.append("stable_projection_drift")
        accepted = False
    if first["thermal"]["throttle_counters"] != last["thermal"]["throttle_counters"]:
        failures.append("thermal_throttle_counter_increase_or_drift")
        accepted = False
    receipt = {
        "schema": "lay.v10.hardware-environment-snapshot.v1",
        "task_id": TASK_ID,
        "verdict": "PASS" if accepted else "BLOCKED_ENVIRONMENT",
        "samples": samples,
        "stable_projection": stable_environment_projection(first),
        "failures": sorted(set(failures)),
        "wait_limit_seconds": 120,
        "host_policy_modified": False,
        "perf_invoked": False,
        "pmu_event_opened": False,
        "runtime_authority_changed": False,
        "installed_lay_changed": False,
    }
    write_new_json(stage / "ENVIRONMENT_RECEIPT.json", receipt)
    seal_stage(stage)
    atomic_publish(stage, REMOTE_B1)
    if not accepted:
        raise GateError("B1 blocked: " + ", ".join(receipt["failures"]))
    print(json.dumps({"state": "B1_PASS_ENVIRONMENT_FROZEN", "receipt": str(REMOTE_B1 / "ENVIRONMENT_RECEIPT.json")}))


def read_fifo_line(descriptor: int, deadline: float) -> str:
    data = bytearray()
    while time.monotonic() < deadline:
        try:
            block = os.read(descriptor, 4096)
        except BlockingIOError:
            time.sleep(0.005)
            continue
        if block:
            data.extend(block)
            if b"\n" in data:
                return data.split(b"\n", 1)[0].decode(errors="replace")
        else:
            time.sleep(0.005)
    raise GateError("timed out waiting for perf control acknowledgement")


def b2_workload(cpus: list[int], control_dir: pathlib.Path) -> None:
    children = []
    try:
        for cpu in cpus:
            children.append(
                subprocess.Popen(
                    ["/usr/bin/taskset", "-c", str(cpu), "/usr/bin/yes"],
                    stdout=subprocess.DEVNULL,
                    stderr=subprocess.DEVNULL,
                )
            )
        write_new_bytes(control_dir / "workload-ready", b"ready\n", 0o400)
        deadline = time.monotonic() + 20.0
        while time.monotonic() < deadline and not (control_dir / "workload-stop").is_file():
            if any(child.poll() is not None for child in children):
                raise GateError("benign yes workload exited before stop")
            time.sleep(0.01)
        require((control_dir / "workload-stop").is_file(), "benign workload stop timeout")
    finally:
        for child in children:
            if child.poll() is None:
                child.terminate()
        for child in children:
            with contextlib.suppress(subprocess.TimeoutExpired):
                child.wait(timeout=2)
            if child.poll() is None:
                child.kill()
                child.wait()


def normalized_perf_event_name(value: str) -> str:
    rendered = value.lower().replace("cpu_core/", "").replace("cpu_atom/", "")
    rendered = rendered.strip("/").split(":", 1)[0]
    aliases = {
        "cpu-cycles": "cycles",
        "branch-instructions": "branches",
    }
    return aliases.get(rendered, rendered)


def parse_perf_json(
    path: pathlib.Path,
    expected_events: list[str],
    *,
    route: str,
) -> dict[str, Any]:
    text = path.read_text(encoding="utf-8", errors="replace").strip()
    values: list[Any] = []
    if text.startswith("["):
        parsed = json.loads(text)
        values = parsed if isinstance(parsed, list) else [parsed]
    else:
        for line in text.splitlines():
            line = line.strip().rstrip(",")
            if line.startswith("{"):
                values.append(json.loads(line))
    require(values, f"perf produced no machine-readable counters: {path}")
    event_names: list[str] = []
    time_records = []
    for value in values:
        require(isinstance(value, dict), f"non-object perf counter record: {value!r}")
        rendered_value = json.dumps(value).lower()
        require("not supported" not in rendered_value, "unsupported perf event")
        require("not counted" not in rendered_value, "uncounted perf event")
        percent = value.get("pcnt-running", value.get("percent-running", 100.0))
        if isinstance(percent, str):
            percent = float(percent.rstrip("%"))
        require(float(percent) == 100.0, f"multiplexed or scaled perf event: {value}")
        event_name = value.get("event", value.get("event-name"))
        require(isinstance(event_name, str) and event_name, f"perf record lacks event name: {value}")
        event_names.append(event_name)
        counter = value.get("counter-value")
        require(counter is not None, f"perf record lacks counter value: {value}")
        try:
            numeric_counter = float(str(counter).replace(",", ""))
        except ValueError as error:
            raise GateError(f"non-numeric perf counter: {value}") from error
        runtime = value.get("event-runtime")
        require(isinstance(runtime, (int, float)) and runtime > 0, f"invalid event runtime: {value}")
        time_enabled = float(runtime) / (float(percent) / 100.0)
        require(abs(time_enabled - float(runtime)) <= 0.5, "time_running != time_enabled")
        time_records.append(
            {
                "event": event_name,
                "counter_value": numeric_counter,
                "time_running": runtime,
                "time_enabled_derived": time_enabled,
                "percent_running": float(percent),
            }
        )
    normalized = [normalized_perf_event_name(value) for value in event_names]
    for event in expected_events:
        require(event.lower() in normalized, f"missing exact perf event {event}: {event_names}")
    hybrid_coverage: dict[str, list[str]] = {}
    if route == "FULL_20":
        for event in expected_events:
            if event in SOFTWARE_EVENTS:
                continue
            raw = [
                value.lower()
                for value in event_names
                if normalized_perf_event_name(value) == event.lower()
            ]
            coverage = sorted(
                pmu for pmu in ("cpu_core", "cpu_atom") if any(pmu in value for value in raw)
            )
            require(
                coverage == ["cpu_atom", "cpu_core"],
                f"missing hybrid PMU coverage for {event}: {raw}",
            )
            hybrid_coverage[event] = coverage
    return {
        "records": values,
        "event_names": event_names,
        "time_records": time_records,
        "hybrid_coverage": hybrid_coverage,
    }


def parse_perf_verbose(stderr: bytes) -> dict[str, Any]:
    text = stderr.decode(errors="replace")
    require("perf_event_attr:" in text, "perf verbose output lacks perf_event_attr")
    require(re.search(r"\binherit\s+1\b", text) is not None, "perf attrs do not prove inheritance")
    require(re.search(r"\bexclude_user\s+0\b", text) is not None, "perf attrs exclude user execution")
    require(re.search(r"\bexclude_kernel\s+0\b", text) is not None, "perf attrs exclude kernel execution")
    require(re.search(r"\bexclude_hv\s+1\b", text) is not None, "perf attrs do not exclude hypervisor")
    require(re.search(r"\btype\s+\d+", text) is not None, "perf attrs lack selector type")
    require(re.search(r"\bconfig\s+0x?[0-9a-f]+", text, re.IGNORECASE) is not None, "perf attrs lack selector config")
    attributes = []
    for block in text.split("perf_event_attr:")[1:]:
        fields: dict[str, str] = {}
        for line in block.splitlines():
            stripped = line.strip()
            if not stripped:
                if fields:
                    break
                continue
            match = re.match(r"([a-zA-Z0-9_]+)\s+(.+)", stripped)
            if match:
                fields[match.group(1)] = match.group(2)
        if fields:
            attributes.append(fields)
    require(attributes, "perf verbose attrs could not be parsed")
    return {
        "attributes": attributes,
        "inherit_proven": True,
        "include_user_proven": True,
        "include_kernel_proven": True,
        "exclude_hypervisor_proven": True,
        "raw_sha256": sha256_bytes(stderr),
    }


def pmu_inventory() -> dict[str, Any]:
    devices = {}
    root = pathlib.Path("/sys/bus/event_source/devices")
    for device in sorted(root.iterdir() if root.is_dir() else []):
        if not device.is_dir():
            continue
        formats = {
            path.name: read_optional(path)
            for path in sorted((device / "format").glob("*"))
            if path.is_file()
        }
        aliases = {
            path.name: read_optional(path)
            for path in sorted((device / "events").glob("*"))
            if path.is_file()
        }
        caps = {
            path.name: read_optional(path)
            for path in sorted((device / "caps").glob("*"))
            if path.is_file()
        }
        devices[device.name] = {
            "type": read_optional(device / "type"),
            "cpus": read_optional(device / "cpus"),
            "cpumask": read_optional(device / "cpumask"),
            "format": formats,
            "event_aliases": aliases,
            "caps": caps,
        }
    require("cpu_core" in devices and "cpu_atom" in devices, "hybrid cpu_core/cpu_atom PMUs are absent")
    require(devices["cpu_core"]["type"] is not None, "cpu_core PMU type is absent")
    require(devices["cpu_atom"]["type"] is not None, "cpu_atom PMU type is absent")
    return devices


def perf_capability_probe(
    route: str,
    group: str,
    cpus: list[int],
    stage: pathlib.Path,
) -> dict[str, Any]:
    probe = stage / f"{route.lower()}-{group.lower()}"
    probe.mkdir(mode=0o700)
    control_fifo = probe / "control.fifo"
    ack_fifo = probe / "ack.fifo"
    os.mkfifo(control_fifo, 0o600)
    os.mkfifo(ack_fifo, 0o600)
    output = probe / "perf.json"
    workload_dir = probe / "workload"
    workload_dir.mkdir(mode=0o700)
    events = PERF_GROUPS[group]
    command = [
        "/usr/bin/perf",
        "stat",
        "-vv",
        "--json-output",
        "--no-big-num",
        "--per-thread",
        "--delay=-1",
        f"--control=fifo:{control_fifo},{ack_fifo}",
        "--event",
        ",".join(events),
        "--output",
        str(output),
        "--",
        "/usr/bin/taskset",
        "-c",
        ",".join(str(cpu) for cpu in cpus),
        "/usr/bin/python3",
        str(REMOTE_B0A / "inputs/controller/lay-v10-hardware-b0-b2.py"),
        "remote-b2-workload",
        "--cpus",
        ",".join(str(cpu) for cpu in cpus),
        "--control-dir",
        str(workload_dir),
    ]
    process: subprocess.Popen[bytes] | None = None
    control_fd: int | None = None
    ack_fd: int | None = None
    stdout = b""
    stderr = b""
    try:
        process = subprocess.Popen(
            command,
            env=controlled_environment(),
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            start_new_session=True,
        )
        deadline = time.monotonic() + 20.0
        while time.monotonic() < deadline and not (workload_dir / "workload-ready").is_file():
            if process.poll() is not None:
                stdout, stderr = process.communicate()
                raise GateError(f"perf probe exited before ready: {stdout!r} {stderr!r}")
            time.sleep(0.01)
        require((workload_dir / "workload-ready").is_file(), "benign workload did not become ready")
        control_fd = os.open(control_fifo, os.O_WRONLY | os.O_NONBLOCK)
        ack_fd = os.open(ack_fifo, os.O_RDONLY | os.O_NONBLOCK)
        os.write(control_fd, b"enable\n")
        enable_ack = read_fifo_line(ack_fd, time.monotonic() + 5.0)
        time.sleep(0.25)
        os.write(control_fd, b"disable\n")
        disable_ack = read_fifo_line(ack_fd, time.monotonic() + 5.0)
        write_new_bytes(workload_dir / "workload-stop", b"stop\n", 0o400)
        stdout, stderr = process.communicate(timeout=10)
    finally:
        if not (workload_dir / "workload-stop").exists():
            with contextlib.suppress(Exception):
                write_new_bytes(workload_dir / "workload-stop", b"stop\n", 0o400)
        if control_fd is not None:
            os.close(control_fd)
        if ack_fd is not None:
            os.close(ack_fd)
        if process is not None and process.poll() is None:
            with contextlib.suppress(subprocess.TimeoutExpired):
                stdout, stderr = process.communicate(timeout=3)
            if process.poll() is None:
                with contextlib.suppress(ProcessLookupError):
                    os.killpg(process.pid, 15)
                with contextlib.suppress(subprocess.TimeoutExpired):
                    stdout, stderr = process.communicate(timeout=2)
            if process.poll() is None:
                with contextlib.suppress(ProcessLookupError):
                    os.killpg(process.pid, 9)
                stdout, stderr = process.communicate()
        with contextlib.suppress(FileNotFoundError):
            control_fifo.unlink()
        with contextlib.suppress(FileNotFoundError):
            ack_fifo.unlink()
    require(process is not None, "perf capability process was not created")
    write_new_bytes(probe / "stdout.log", stdout)
    write_new_bytes(probe / "stderr.log", stderr)
    require(process.returncode == 0, f"perf capability probe exited {process.returncode}: {stderr[-2000:]!r}")
    counters = parse_perf_json(output, events, route=route)
    selector_resolution = parse_perf_verbose(stderr)
    return {
        "route": route,
        "group": group,
        "cpus": cpus,
        "events": events,
        "command": command,
        "enable_ack": enable_ack,
        "disable_ack": disable_ack,
        "counter_records": counters,
        "selector_resolution": selector_resolution,
        "per_thread_output_requested_and_accepted": True,
        "perf_output_sha256": sha256_file(output),
    }


def remote_b2() -> None:
    require(REMOTE_B1.is_dir(), "accepted B1 snapshot is absent")
    require(not REMOTE_B2.exists(), "B2 final already exists")
    verify_sha256sums(REMOTE_B1)
    b1 = load_json(REMOTE_B1 / "ENVIRONMENT_RECEIPT.json")
    require(b1.get("verdict") == "PASS", "B1 did not pass")
    stage = pathlib.Path(f"{REMOTE_B2}.stage-{os.getpid()}-{time.time_ns()}")
    stage.mkdir(mode=0o700)
    try:
        yes_identity = require_file(
            pathlib.Path("/usr/bin/yes"),
            sha256=EXPECTED["yes"],
            size=EXPECTED_SIZES["yes"],
            mode="0755",
        )
        taskset_identity = require_file(
            pathlib.Path("/usr/bin/taskset"), sha256=EXPECTED["taskset"], mode="0755"
        )
        perf_identity = require_file(
            pathlib.Path("/usr/bin/perf"), sha256=EXPECTED["perf"], mode="0755"
        )
        environment = controlled_environment()
        perf_version = run(["/usr/bin/perf", "version"], env=environment).stdout.decode().strip()
        pmus = pmu_inventory()
        before = environment_snapshot()
        require(
            stable_environment_projection(before) == b1["stable_projection"],
            "B1 environment drift before B2",
        )
        require(not environment_admission(before), "environment not admitted before B2")
        cpus = [record["cpu"] for record in before["topology"]["cpus"]]
        require(len(cpus) == 20, "B2 requires 20 online CPUs")
        p_core = [
            record["cpu"]
            for record in before["topology"]["cpus"]
            if record.get("core_type") in {"2", "core", "Core"}
            and record.get("thread_siblings_list")
        ]
        e_core = [
            record["cpu"]
            for record in before["topology"]["cpus"]
            if record.get("core_type") in {"1", "atom", "Atom"}
        ]
        require(p_core, "no observed P-core CPU with sibling mapping")
        require(e_core, "no observed E-core CPU mapping")
        selected_p_core = min(p_core)
        probes = []
        for group in ("G0", "G1", "G2", "G3"):
            probes.append(perf_capability_probe("P_CORE", group, [selected_p_core], stage))
            probes.append(perf_capability_probe("FULL_20", group, cpus, stage))
        after = environment_snapshot()
        require(stable_environment_projection(after) == b1["stable_projection"], "B1 environment drift after B2")
        require(not environment_admission(after), "environment not admitted after B2")
        require(before["thermal"]["throttle_counters"] == after["thermal"]["throttle_counters"], "throttle counter drift during B2")
        receipt = {
            "schema": "lay.v10.hardware-benign-pmu-capability.v1",
            "task_id": TASK_ID,
            "verdict": "PASS",
            "perf": perf_identity,
            "perf_version": perf_version,
            "yes": yes_identity,
            "taskset": taskset_identity,
            "perf_event_paranoid": read_optional(pathlib.Path("/proc/sys/kernel/perf_event_paranoid")),
            "kptr_restrict": read_optional(pathlib.Path("/proc/sys/kernel/kptr_restrict")),
            "nmi_watchdog": read_optional(pathlib.Path("/proc/sys/kernel/nmi_watchdog")),
            "selected_p_core": selected_p_core,
            "observed_p_core_cpus": p_core,
            "observed_e_core_cpus": e_core,
            "online_cpus": cpus,
            "available_pmus_and_aliases": pmus,
            "groups": PERF_GROUPS,
            "probes": probes,
            "pre_b2_perf_executable_invocations": 2,
            "b2_perf_executable_invocations": 9,
            "b2_perf_stat_invocations": 8,
            "b2_perf_version_invocations": 1,
            "b1_stable_projection_sha256": sha256_bytes(canonical_json_bytes(b1["stable_projection"])),
            "historical_v10_executed": False,
            "diagnostic_proxy_executed": False,
            "v13_loaded": False,
            "quality_or_latency_measured": False,
            "later_gates_admitted": False,
            "runtime_authority_changed": False,
            "installed_lay_changed": False,
        }
        write_new_json(stage / "PMU_CAPABILITY_RECEIPT.json", receipt)
        seal_stage(stage)
        atomic_publish(stage, REMOTE_B2)
        print(json.dumps({"state": "B0_B2_CAPABILITY_READY_LATER_GATES_NOT_ADMITTED", "receipt": str(REMOTE_B2 / "PMU_CAPABILITY_RECEIPT.json")}))
    except Exception as error:
        with contextlib.suppress(Exception):
            write_new_json(
                stage / "BLOCKED_PMU.json",
                {
                    "schema": "lay.v10.hardware-benign-pmu-capability-blocked.v1",
                    "verdict": "BLOCKED_PMU",
                    "error": str(error),
                    "later_gates_admitted": False,
                    "runtime_authority_changed": False,
                    "installed_lay_changed": False,
                },
            )
            seal_stage(stage)
            atomic_publish(stage, REMOTE_B2)
        raise


def remote_status() -> None:
    paths = {
        "b0a": REMOTE_B0A,
        "build": REMOTE_BUILD,
        "freezer": REMOTE_FREEZER,
        "b0b": REMOTE_B0B,
        "b1": REMOTE_B1,
        "b2": REMOTE_B2,
    }
    markers = REMOTE_STATE / "markers"
    print(
        json.dumps(
            {
                "task_id": TASK_ID,
                "host": os.uname().nodename,
                "outputs": {name: path.is_dir() for name, path in paths.items()},
                "markers": sorted(path.name for path in markers.iterdir()) if markers.is_dir() else [],
                "later_gates_admitted": False,
            },
            sort_keys=True,
        )
    )


def self_check_schedule() -> dict[str, Any]:
    entries = []
    for ordinal in range(382):
        worker = ordinal // 20
        entries.append(
            {
                "source_ordinal": ordinal,
                "composite_identity_sha256": sha256_bytes(f"identity:{ordinal}".encode()),
                "damaged_surface_sha256": sha256_bytes(f"damaged:{ordinal}".encode()),
                "retrieval_lanes_sha256": sha256_bytes(f"lanes:{ordinal}".encode()),
                "b5_ordinal": ordinal,
                "b6_worker": worker,
                "b6_chunk_start": worker * 20,
                "b6_chunk_end_exclusive": min(worker * 20 + 20, 382),
            }
        )
    return {
        "schema": "lay.v10.hardware-query-schedule.v1",
        "source": {
            "v7_sha256": EXPECTED["v7"],
            "phase7d_semantics_sha256": "1" * 64,
            "composite_identity_encoding": "sha256(class_utf8 || NUL || proof_identity_json_utf8)",
            "damaged_surface_encoding": "sha256(utf8)",
            "retrieval_lanes_encoding": "sha256(compact_json([{symbols,maximum_levenshtein_distance}]))",
        },
        "partition": {"b5_requests": 382, "b6_workers": 20, "b6_chunk_size": 20},
        "entries": entries,
    }


def expect_gate_error(action: Any, label: str) -> None:
    try:
        action()
    except GateError:
        return
    raise GateError(f"fault check did not fail closed: {label}")


def local_fault_checks() -> dict[str, Any]:
    checks = 0
    with tempfile.TemporaryDirectory(prefix="lay-v10-hardware-self-check-") as temporary:
        root = pathlib.Path(temporary)
        stage = root / "publication.stage"
        final = root / "publication.final"
        stage.mkdir()
        write_new_bytes(stage / "payload", b"payload\n")
        seal_stage(stage)
        before = inventory(stage)
        atomic_publish(stage, final)
        require(not stage.exists() and inventory(final) == before, "atomic publication parity failed")
        verify_sha256sums(final)
        checks += 1

        collision_stage = root / "collision.stage"
        collision_final = root / "collision.final"
        collision_stage.mkdir()
        write_new_bytes(collision_stage / "payload", b"stage\n")
        seal_stage(collision_stage)
        collision_final.mkdir()
        expect_gate_error(lambda: atomic_publish(collision_stage, collision_final), "final collision")
        require(collision_stage.exists(), "collision destroyed unpublished staging")
        checks += 1

        missing_parent = root / "missing-provenance" / "task"
        nearest_parent, nearest_metadata = nearest_existing_directory(missing_parent)
        require(nearest_parent == root, "nearest existing parent resolution drift")
        require(
            nearest_metadata.st_dev == root.stat().st_dev,
            "nearest existing parent device drift",
        )
        missing_parent.mkdir(parents=True)
        require_directory_device(missing_parent, nearest_metadata.st_dev)
        checks += 1

        expect_gate_error(
            lambda: require_directory_device(missing_parent, nearest_metadata.st_dev + 1),
            "created parent device mismatch",
        )
        checks += 1

        machine_id_sentinel = root / "machine-id"
        write_new_bytes(machine_id_sentinel, b"0123456789abcdef0123456789abcdef\n")
        exact_machine_id = sha256_file(machine_id_sentinel)
        stripped_machine_id = sha256_bytes(machine_id_sentinel.read_bytes().strip())
        require(exact_machine_id != stripped_machine_id, "machine-id normalization fault collapsed")
        checks += 1

        state = root / "state"
        markers = state / "markers"
        markers.mkdir(parents=True)
        write_new_bytes(markers / "build.available", b"one\n", 0o400)
        consumed = consume_marker("build", state)
        require(consumed.name == "build.consumed-before-exec", "marker was not consumed")
        expect_gate_error(lambda: consume_marker("build", state), "second marker consumption")
        checks += 1

        perf_json = root / "perf.json"
        perf_records = []
        for event in PERF_GROUPS["G1"]:
            for pmu in ("cpu_core", "cpu_atom"):
                perf_records.append(
                    {
                        "counter-value": "100",
                        "event": f"{pmu}/{event}/",
                        "event-runtime": 1_000_000,
                        "pcnt-running": 100.0,
                    }
                )
        write_new_bytes(perf_json, json.dumps(perf_records).encode())
        parsed = parse_perf_json(perf_json, PERF_GROUPS["G1"], route="FULL_20")
        require(
            all(value == ["cpu_atom", "cpu_core"] for value in parsed["hybrid_coverage"].values()),
            "hybrid perf parser coverage mismatch",
        )
        verbose = (
            b"perf_event_attr:\n  type 4\n  config 0x0\n  inherit 1\n"
            b"  exclude_user 0\n  exclude_kernel 0\n  exclude_hv 1\n\n"
        )
        parse_perf_verbose(verbose)
        missing_atom = root / "perf-missing-atom.json"
        write_new_bytes(
            missing_atom,
            json.dumps(
                [record for record in perf_records if "cpu_atom/" not in record["event"]]
            ).encode(),
        )
        expect_gate_error(
            lambda: parse_perf_json(missing_atom, PERF_GROUPS["G1"], route="FULL_20"),
            "missing cpu_atom coverage",
        )
        checks += 2

    schedule = self_check_schedule()
    validate_schedule_python(schedule, phase7d_semantics_sha256="1" * 64)
    checks += 1
    leaked = json.loads(json.dumps(schedule))
    leaked["entries"][0]["target_surface"] = "forbidden"
    expect_gate_error(lambda: validate_schedule_python(leaked), "target key leak")
    duplicate = json.loads(json.dumps(schedule))
    duplicate["entries"][1]["composite_identity_sha256"] = duplicate["entries"][0][
        "composite_identity_sha256"
    ]
    expect_gate_error(lambda: validate_schedule_python(duplicate), "duplicate identity")
    checks += 2

    remote_arguments = [
        "/usr/bin/python3",
        "-c",
        "import pathlib\nprint(pathlib.Path('a b'))\n",
        "space separated",
        "single'quote",
        'double"quote',
        "$HOME;touch forbidden",
        "",
    ]
    process_arguments = ssh_process_argv(remote_arguments)
    require(process_arguments[-2] == REMOTE, "ssh destination placement drift")
    require(
        shlex.split(process_arguments[-1]) == remote_arguments,
        "quoted remote argv did not round-trip byte-identically",
    )
    raw_join = " ".join(remote_arguments)
    with contextlib.suppress(ValueError):
        require(shlex.split(raw_join) != remote_arguments, "known raw join unexpectedly preserved argv")
    expect_gate_error(lambda: ssh_command([]), "empty remote argv")
    expect_gate_error(lambda: ssh_command(["python3", "bad\0arg"]), "remote argv NUL")
    checks += 2

    controller = pathlib.Path(__file__).read_text(encoding="utf-8")
    normalized_machine_id_call = "read_bytes()" + ".strip()"
    require(
        controller.count(normalized_machine_id_call) == 1,
        "machine-id normalization escaped its negative-control test",
    )
    exact_machine_id_call = 'sha256_file(pathlib.Path(' + '"/etc/machine-id"))'
    require(
        controller.count(exact_machine_id_call) == 3,
        "exact-file machine-id owners are incomplete",
    )
    remote_probe_exact_call = "'machine_id_sha256': " + "digest(machine_path)"
    require(remote_probe_exact_call in controller, "remote probe exact hash is absent")
    tree = ast.parse(controller)
    pre_b2_owners = {
        "remote_b0a_finalize",
        "remote_build",
        "remote_freezer",
        "remote_b0b",
        "remote_b1",
    }
    for node in tree.body:
        if isinstance(node, ast.FunctionDef) and node.name in pre_b2_owners:
            literals = [
                child.value
                for child in ast.walk(node)
                if isinstance(child, ast.Constant) and isinstance(child.value, str)
            ]
            require(not any("/usr/bin/perf" in value for value in literals), f"perf path leaked into {node.name}")
            calls = [
                child.func.id
                for child in ast.walk(node)
                if isinstance(child, ast.Call) and isinstance(child.func, ast.Name)
            ]
            require("perf_capability_probe" not in calls, f"perf helper leaked into {node.name}")
    checks += 1

    fragment = FRAGMENT.read_text(encoding="utf-8")
    schedule_region = fragment.split("fn hardware_schedule_value", 1)[1].split(
        "fn hardware_write_new_json", 1
    )[0]
    require("target" not in schedule_region.lower(), "target leaked into Rust schedule builder")
    freezer_region = fragment.split("fn v10_hardware_schedule_freezer", 1)[1].split(
        "fn v10_hardware_semantic_parity", 1
    )[0]
    for forbidden in (
        "hardware_run_executor",
        "enumerate_lane",
        "search_typed_peaks",
        "Instant::now",
        "perf_event_open",
        "Command::new",
    ):
        require(forbidden not in freezer_region, f"freezer route contains {forbidden}")
    require('"perf_invoked": false' in freezer_region, "freezer receipt lacks negative perf fact")
    checks += 1
    return {"checks": checks}


def local_self_check() -> None:
    admission = verify_local_admission()
    p0 = verify_p0_inputs()
    fragment = FRAGMENT.read_bytes()
    final = assemble_diagnostic_source((P0 / "artifacts/v13_typed_peak.v10.rs").read_bytes(), fragment)
    require(final[:39_047] == (P0 / "artifacts/v13_typed_peak.v10.rs").read_bytes()[:39_047], "prefix parity")
    faults = local_fault_checks()
    print(
        json.dumps(
            {
                "verdict": "PASS",
                "admission_files": len(admission),
                "p0_inputs": len(p0),
                "production_prefix_sha256": sha256_bytes(final[:39_047]),
                "final_source_sha256": sha256_bytes(final),
                "test_entrypoints": TEST_NAMES,
                "focused_checks": faults["checks"],
                "remote_actions_executed": 0,
                "perf_invocations": 0,
            },
            sort_keys=True,
        )
    )


def parse_cpus(raw: str) -> list[int]:
    cpus = [int(value) for value in raw.split(",") if value]
    require(cpus and len(cpus) == len(set(cpus)), "CPU list must be unique and nonempty")
    return cpus


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(description=__doc__)
    subparsers = parser.add_subparsers(dest="action", required=True)
    for action in ("self-check", "b0a", "build", "freezer", "b0b", "b1", "b2", "status"):
        subparsers.add_parser(action)
    finalize = subparsers.add_parser("remote-b0a-finalize", help=argparse.SUPPRESS)
    finalize.add_argument("--stage", required=True, type=pathlib.Path)
    for action in ("remote-build", "remote-freezer", "remote-b0b", "remote-b1", "remote-b2", "remote-status"):
        subparsers.add_parser(action, help=argparse.SUPPRESS)
    workload = subparsers.add_parser("remote-b2-workload", help=argparse.SUPPRESS)
    workload.add_argument("--cpus", required=True)
    workload.add_argument("--control-dir", required=True, type=pathlib.Path)
    return parser


def require_remote_host() -> None:
    require(os.uname().nodename == REMOTE_HOSTNAME, "remote-only action on wrong host")


def main() -> int:
    arguments = build_parser().parse_args()
    try:
        if arguments.action == "self-check":
            local_self_check()
        elif arguments.action == "b0a":
            local_b0a()
        elif arguments.action in {"build", "freezer", "b0b", "b1", "b2", "status"}:
            remote_controller("remote-" + arguments.action)
        elif arguments.action == "remote-b0a-finalize":
            require_remote_host()
            remote_b0a_finalize(arguments.stage)
        elif arguments.action == "remote-build":
            require_remote_host()
            with exclusive_route_lock():
                remote_build()
        elif arguments.action == "remote-freezer":
            require_remote_host()
            with exclusive_route_lock():
                remote_freezer()
        elif arguments.action == "remote-b0b":
            require_remote_host()
            with exclusive_route_lock():
                remote_b0b()
        elif arguments.action == "remote-b1":
            require_remote_host()
            with exclusive_route_lock():
                remote_b1()
        elif arguments.action == "remote-b2":
            require_remote_host()
            with exclusive_route_lock():
                remote_b2()
        elif arguments.action == "remote-status":
            require_remote_host()
            remote_status()
        elif arguments.action == "remote-b2-workload":
            require_remote_host()
            b2_workload(parse_cpus(arguments.cpus), arguments.control_dir)
        else:
            raise GateError(f"unsupported action {arguments.action}")
        return 0
    except GateError as error:
        print(json.dumps({"verdict": "BLOCKED", "action": arguments.action, "error": str(error)}), file=sys.stderr)
        return 2


if __name__ == "__main__":
    raise SystemExit(main())
