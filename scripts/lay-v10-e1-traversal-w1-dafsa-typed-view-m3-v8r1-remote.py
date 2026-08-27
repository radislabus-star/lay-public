#!/usr/bin/env python3
"""Fail-closed remote producer for the M3 V8 end-to-end proof."""

from __future__ import annotations

import argparse
import contextlib
import fcntl
import hashlib
import json
import os
import pathlib
import re
import shutil
import signal
import stat
import subprocess
import sys
import time
from collections.abc import Mapping, Sequence
from typing import Any


TASK_ID = "slice8b-v10-e1-traversal-w1-dafsa-typed-view-m3-e2e-v8r1-20260827"
TRANSACTION_ID = "7d6455e678c244be3c31dc52c2b64d55f34d0a91338afa1219acf06ff327ffb9"
HOSTNAME = "e-MEGA-MINI-M1-13th"
MACHINE_ID_SHA256 = "5ac0bb537745673ef80942c2df32d586f5753c85de1c8b2453aa4e7317b77441"
KERNEL = "6.8.0-124-generic"

PARENT = pathlib.Path("/home/e/.local/share/lay/provenance") / TASK_ID
STATE = pathlib.Path("/home/e/.local/state/lay") / TASK_ID
BOOTSTRAP = PARENT / "bootstrap-v1"
BUILD = PARENT / "build-v1"
E2E = PARENT / "e2e-v1"
LOADER = pathlib.Path("/usr/lib/x86_64-linux-gnu/ld-linux-x86-64.so.2")

ACTIONS = (
    "self-check",
    "status",
    "bootstrap",
    "create-markers",
    "build-once",
    "e2e-once",
)
ROUTES = ("BUILD", "E2E")
MARKER_NAMES = {
    "BUILD": "build",
    "E2E": "e2e",
}
SCIENTIFIC_TEST = (
    "nanda_wave::l2_field::v13_typed_peak::tests::m3_v8::"
    "m3_end_to_end_physical_proof"
)

EXPECTED_CARGO = "cargo 1.97.1 (c980f4866 2026-06-30)"
EXPECTED_RUSTC = (
    "release: 1.97.1",
    "commit-hash: 8bab26f4f68e0e26f0bb7960be334d5b520ea452",
    "host: x86_64-unknown-linux-gnu",
    "LLVM version: 22.1.6",
)
EXPECTED_INPUTS = {
    "LAY-L2-RU-FULL-v13.bin": (140_556_462, "cce259fe0ce5dce67702383363b66f0fe9b9ff5a87d8f01c4fcf342d91218d7b"),
    "slice8b-v7-fixed-13x100.json": (1_606_189, "33fded73e13f565bdc08a83bd440be67c80f6d85a196453050b09bc9f7ef28e4"),
    "LAY-L2-PRODUCTIVE-PARADIGM-v90.p2m": (17_309_944, "40fb6a9f0d92c3c7502e47f9c70230d9b86020f622b08a5c799342f13e09ce44"),
    "LAY-L2-PRODUCTIVE-PARADIGM-v90.p2r": (2_123_112, "de7972c80448dc792759d70de99cda6ec48c3d6af337763856601db563ab167e"),
    "LAY-L1.1-RU-COMPOSITE-EN300K-PHASE8I-v9.v9.bin": (77_962_328, "bf5a1619a89038466ef786305cf35eda5f4af5b9f12b9140f7d3cac407e2f2a7"),
    "l11-proof.json": (539_536, "4983930495e793c1d28c7558fe006ddf8097ee575bebb1afd3f1dba4ddb1d01d"),
}
EXPECTED_SOURCE = {
    "Cargo.toml": (2_399, "90a3ae6b16677ff49b70aab3457b794cd1e114a547f9e113a8b49595f976207b"),
    "Cargo.lock": (70_770, "e6399be48c393e1557b451fcb04d886f9fbdaa9812109410ac3d9ecdc98b93f1"),
    "scripts/cargo-guard.sh": (2_534, "a5b01f9a12b32ee2bcfc957a5e9f65efb52fe6e7644850ae3f00e412648243fe"),
    "src/nanda_wave/l2_field/v13_typed_peak.rs": (253_080, "28f87a76fc1996989e980cab51f0443bd95e656fcae3a2ff61f581db9c3a7ee2"),
    "src/nanda_wave/l2_field/v13_typed_peak/typed_exact.rs": (26_203, "325bdd386b13de77bd030ef1af0fecb12be8bdd2bdd3c841d10257647a4ceaf4"),
    "src/nanda_wave/l2_field/mod.rs": (52_483, "c76c28debdac8e43d360a5a82811ea86e3e6f03b98e4f81cb6678e07953e0953"),
    "src/nanda_wave/l2_field/bridge.rs": (111_807, "5f3cd350c59b0b84a6f2250077f4d3e6f061c93a548451c9aafe4cf0f5f820ad"),
    "src/nanda_wave/l2_field/cache.rs": (23_664, "9da969bfff12dba0217954647b1ba8e21302365770abaa82178953fcf63fec07"),
    "src/nanda_wave/l2_field/runtime.rs": (142_756, "cb94bd3ffaf61b31f7cce8d2051ae9087cf8aba26487c037ba6fbeaaa57f0966"),
    "src/nanda_wave/l2_field/productive_v1/live.rs": (70_181, "87180990b6883641483a46886074e5350f35e351454d734f0c3c9da723d758bd"),
    "src/nanda_wave/l2_field/productive_v1/material_frame.rs": (74_452, "2efdceca380534930ff9e42a4f504a798363a9e86497ec71ee52104246e4ec55"),
}
BUILD_ENVIRONMENT_FIXED = {
    "CARGO_BUILD_JOBS": "20",
    "CARGO_INCREMENTAL": "0",
    "CARGO_NET_OFFLINE": "true",
    "CARGO_PROFILE_RELEASE_DEBUG": "2",
    "CARGO_PROFILE_RELEASE_STRIP": "none",
    "RUSTFLAGS": "",
}
BUILD_ENVIRONMENT_KEYS = (
    "CARGO_BUILD_JOBS",
    "CARGO_INCREMENTAL",
    "CARGO_NET_OFFLINE",
    "CARGO_PROFILE_RELEASE_DEBUG",
    "CARGO_PROFILE_RELEASE_STRIP",
    "CARGO_TARGET_DIR",
    "RUSTFLAGS",
)


class V8RemoteError(RuntimeError):
    pass


def need(condition: bool, message: str) -> None:
    if not condition:
        raise V8RemoteError(message)


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


def require_file(
    path: pathlib.Path,
    *,
    size: int | None = None,
    digest: str | None = None,
    mode: str | None = None,
) -> dict[str, Any]:
    row = file_row(path)
    if size is not None:
        need(row["size_bytes"] == size, f"size drift: {path}")
    if digest is not None:
        need(row["sha256"] == digest, f"SHA drift: {path}")
    if mode is not None:
        need(row["mode"] == mode, f"mode drift: {path}")
    return row


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


def write_json(path: pathlib.Path, value: Any, mode: int = 0o600) -> None:
    write_new(path, canonical(value), mode)


def fsync_dir(path: pathlib.Path) -> None:
    descriptor = os.open(path, os.O_RDONLY | os.O_DIRECTORY)
    try:
        os.fsync(descriptor)
    finally:
        os.close(descriptor)


def run(
    argv: Sequence[str],
    *,
    cwd: pathlib.Path | None = None,
    env: Mapping[str, str] | None = None,
    timeout: float = 3_600,
    check: bool = True,
    stdout: Any = subprocess.PIPE,
    stderr: Any = subprocess.PIPE,
) -> subprocess.CompletedProcess[bytes]:
    result = subprocess.run(
        list(argv),
        cwd=cwd,
        env=dict(env) if env is not None else None,
        stdout=stdout,
        stderr=stderr,
        timeout=timeout,
        check=False,
    )
    if check and result.returncode != 0:
        detail = result.stderr[-4000:] if isinstance(result.stderr, bytes) else b""
        raise V8RemoteError(
            f"command failed ({result.returncode}): {list(argv)!r}\n{detail.decode(errors='replace')}"
        )
    return result


def controlled_environment() -> dict[str, str]:
    return {
        "HOME": "/home/e",
        "LANG": "C.UTF-8",
        "LC_ALL": "C.UTF-8",
        "PATH": "/home/e/.cargo/bin:/home/e/.local/bin:/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin",
        "RUST_BACKTRACE": "0",
    }


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


def make_writable(root: pathlib.Path) -> None:
    for path in [root, *root.rglob("*")]:
        path.chmod(0o700 if path.is_dir() else 0o600)


def verify_host() -> dict[str, Any]:
    need(os.uname().nodename == HOSTNAME, "remote hostname drift")
    need(os.uname().release == KERNEL, "remote kernel drift")
    machine = require_file(pathlib.Path("/etc/machine-id"))
    need(machine["sha256"] == MACHINE_ID_SHA256, "remote machine-id drift")
    online = pathlib.Path("/sys/devices/system/cpu/online").read_text().strip()
    core = pathlib.Path("/sys/bus/event_source/devices/cpu_core/cpus").read_text().strip()
    atom = pathlib.Path("/sys/bus/event_source/devices/cpu_atom/cpus").read_text().strip()
    need((online, core, atom) == ("0-19", "0-11", "12-19"), "remote CPU topology drift")
    cargo = run(["/home/e/.cargo/bin/cargo", "-V"], env=controlled_environment()).stdout.decode().strip()
    rustc = run(["/home/e/.cargo/bin/rustc", "-Vv"], env=controlled_environment()).stdout.decode()
    need(cargo == EXPECTED_CARGO, "remote Cargo drift")
    need(all(token in rustc for token in EXPECTED_RUSTC), "remote rustc drift")
    return {
        "hostname": HOSTNAME,
        "kernel": KERNEL,
        "machine_id_sha256": machine["sha256"],
        "online": online,
        "core": core,
        "atom": atom,
        "cargo": cargo,
        "rustc": rustc.strip(),
    }


def verify_inventory(root: pathlib.Path, payload: Mapping[str, Any]) -> dict[str, Any]:
    inventory = payload.get("files")
    need(isinstance(inventory, dict) and inventory, "bootstrap payload inventory absent")
    actual = {
        path.relative_to(root).as_posix()
        for path in root.rglob("*")
        if path.is_file() and path.name != "PAYLOAD.json"
    }
    need(actual == set(inventory), "bootstrap payload file set drift")
    rows = {}
    for relative, expected in sorted(inventory.items()):
        need(isinstance(expected, dict), f"invalid inventory row: {relative}")
        path = root / relative
        rows[relative] = require_file(
            path,
            size=int(expected.get("size_bytes", -1)),
            digest=str(expected.get("sha256", "")),
        )
    return rows


def verify_source_closure(root: pathlib.Path) -> dict[str, Any]:
    manifest_path = root / "SOURCE_MANIFEST.json"
    manifest = load_json(manifest_path)
    need(manifest.get("schema") == "lay.m3-v8r1-source-closure.v1", "source manifest schema drift")
    rows = manifest.get("files")
    need(isinstance(rows, dict) and rows, "source manifest file inventory absent")
    actual = {
        path.relative_to(root).as_posix()
        for path in root.rglob("*")
        if path.is_file() and path.name != "SOURCE_MANIFEST.json"
    }
    need(actual == set(rows), "source closure file set drift")
    for relative, expected in sorted(rows.items()):
        require_file(
            root / relative,
            size=int(expected["size_bytes"]),
            digest=str(expected["sha256"]),
        )
    for relative, (size, digest) in EXPECTED_SOURCE.items():
        require_file(root / relative, size=size, digest=digest)
    return {
        "files": len(rows),
        "bytes": sum(int(row["size_bytes"]) for row in rows.values()),
        "manifest_sha256": sha256_file(manifest_path),
    }


def input_paths(base: pathlib.Path = BOOTSTRAP) -> dict[str, pathlib.Path]:
    inputs = base / "inputs"
    return {
        "v13": inputs / "LAY-L2-RU-FULL-v13.bin",
        "v7": inputs / "slice8b-v7-fixed-13x100.json",
        "productive": inputs / "LAY-L2-PRODUCTIVE-PARADIGM-v90.p2m",
        "recovery": inputs / "LAY-L2-PRODUCTIVE-PARADIGM-v90.p2r",
        "l11": inputs / "LAY-L1.1-RU-COMPOSITE-EN300K-PHASE8I-v9.v9.bin",
        "l11_proof": inputs / "l11-proof.json",
        "l11_receipt": inputs / "l11-installed.json",
    }


def verify_inputs(base: pathlib.Path) -> dict[str, Any]:
    rows = {}
    for name, (size, digest) in EXPECTED_INPUTS.items():
        rows[name] = require_file(base / "inputs" / name, size=size, digest=digest)
    return rows


def build_l11_receipt(final_bootstrap: pathlib.Path) -> dict[str, Any]:
    paths = input_paths(final_bootstrap)
    return {
        "schema": "lay.l11.installed-package.v1",
        "package_id": "LAY-L1.1-RU-COMPOSITE-EN300K-PHASE8I-v9",
        "format": "V9 phase8i-exact-typed-basin",
        "installed_artifact": str(paths["l11"]),
        "artifact_bytes": EXPECTED_INPUTS[paths["l11"].name][0],
        "artifact_sha256": EXPECTED_INPUTS[paths["l11"].name][1],
        "proof_receipt": str(paths["l11_proof"]),
        "proof_source": "sealed experiment-local proof copy",
        "proof_sha256": EXPECTED_INPUTS["l11-proof.json"][1],
        "proof_verdict": "PASS_C_QUALITY",
        "runtime_authority": False,
        "runtime_admitted": True,
    }


def verify_execution_admission(path: pathlib.Path, controller_sha: str) -> dict[str, Any]:
    value = load_json(path)
    need(value.get("verdict") == "M3_V8R1_EXECUTION_ADMITTED", "execution admission verdict drift")
    need(value.get("safe_to_execute") is True, "execution admission is not safe")
    need(value.get("task_id") == TASK_ID and value.get("transaction_id") == TRANSACTION_ID, "execution admission namespace drift")
    need(value.get("remote_controller_sha256") == controller_sha, "execution admission controller SHA drift")
    return value


def uid_capability_probe(stage_parent: pathlib.Path) -> dict[str, Any]:
    probe = stage_parent / "uid-capability"
    probe.mkdir(mode=0o700)
    shutil.chown(probe, user="e", group="e")
    code = (
        "import os,pathlib; p=pathlib.Path(os.environ['P']); a=p/'a'; b=p/'b'; "
        "f=open(a,'xb'); f.write(b'm3-v8-uid-proof\\n'); f.flush(); os.fsync(f.fileno()); f.close(); "
        "os.rename(a,b); assert b.read_bytes()==b'm3-v8-uid-proof\\n'; b.unlink()"
    )
    result = run(
        ["/usr/bin/sudo", "-n", "-u", "e", "/usr/bin/env", f"P={probe}", "/usr/bin/python3", "-c", code],
        check=False,
    )
    need(result.returncode == 0 and not any(probe.iterdir()), "UID e capability proof failed")
    probe.rmdir()
    return {
        "uid": 1000,
        "operations": ["traverse", "create", "write", "fsync", "rename", "read", "unlink"],
        "verdict": "PASS",
    }


def marker_inventory() -> dict[str, list[str]]:
    markers = STATE / "markers"
    if not markers.is_dir():
        return {"available": [], "consumed": []}
    return {
        "available": sorted(path.name for path in markers.glob("*.available")),
        "consumed": sorted(path.name for path in markers.glob("*.consumed-before-exec")),
    }


def append_state(sequence: int, state: str, **extra: Any) -> None:
    write_json(
        STATE / f"STATE-{sequence:02d}-{state}.json",
        {
            "schema": "lay.m3-v8r1-remote-state.v1",
            "task_id": TASK_ID,
            "transaction_id": TRANSACTION_ID,
            "sequence": sequence,
            "state": state,
            "markers": marker_inventory(),
            **extra,
        },
        0o444,
    )
    fsync_dir(STATE)


def bootstrap_once(path: pathlib.Path) -> dict[str, Any]:
    need(os.geteuid() == 0, "bootstrap requires root")
    need(not PARENT.exists() and not STATE.exists(), "V8 remote namespace already exists")
    payload = load_json(path / "PAYLOAD.json")
    need(payload.get("schema") == "lay.m3-v8r1-bootstrap-payload.v1", "bootstrap payload schema drift")
    need(payload.get("task_id") == TASK_ID and payload.get("transaction_id") == TRANSACTION_ID, "bootstrap payload namespace drift")
    controller_sha = sha256_file(path / "remote-controller.py")
    admission = verify_execution_admission(path / "EXECUTION_ADMISSION.json", controller_sha)
    inventory = verify_inventory(path, payload)
    source = verify_source_closure(path / "source-closure")
    inputs = verify_inputs(path)
    host = verify_host()

    stage = pathlib.Path(f"{PARENT}.stage-{os.getpid()}-{time.time_ns()}")
    state_stage = pathlib.Path(f"{STATE}.stage-{os.getpid()}-{time.time_ns()}")
    stage.mkdir(parents=True, mode=0o755)
    state_stage.mkdir(parents=True, mode=0o700)
    try:
        copied = stage / "bootstrap-v1"
        shutil.copytree(path, copied)
        receipt_path = copied / "inputs/l11-installed.json"
        need(not receipt_path.exists(), "experiment-local L1.1 receipt unexpectedly exists")
        write_json(receipt_path, build_l11_receipt(BOOTSTRAP), 0o444)
        uid = uid_capability_probe(stage)
        write_new(state_stage / "route.lock", b"m3-v8-route-lock\n", 0o600)
        receipt = {
            "schema": "lay.m3-v8r1-bootstrap.v1",
            "task_id": TASK_ID,
            "transaction_id": TRANSACTION_ID,
            "verdict": "M3_V8R1_BOOTSTRAP_CREATED_UNAUDITED",
            "controller_sha256": controller_sha,
            "execution_admission_sha256": sha256_file(path / "EXECUTION_ADMISSION.json"),
            "execution_admission_verdict": admission["verdict"],
            "host": host,
            "source_closure": source,
            "inputs": inputs,
            "uploaded_files": len(inventory),
            "uid_capability": uid,
            "markers_expected": 2,
            "markers_created": 0,
            "markers_consumed": 0,
            "cargo_invocations": 0,
            "rustc_compilations": 0,
            "subject_executions": 0,
            "perf_record_invocations": 0,
            "perf_stat_invocations": 0,
            "runtime_authority_changed": False,
        }
        write_json(stage / "BOOTSTRAP_RECEIPT.json", receipt)
        write_manifest(stage)
        seal_tree(stage)
        os.rename(stage, PARENT)
        os.rename(state_stage, STATE)
        fsync_dir(PARENT.parent)
        fsync_dir(STATE.parent)
        append_state(0, "BOOTSTRAP_CREATED_UNAUDITED")
        return receipt
    except BaseException:
        if stage.exists():
            shutil.rmtree(stage)
        if state_stage.exists():
            shutil.rmtree(state_stage)
        raise


def marker_payload(route: str, admission: Mapping[str, Any]) -> bytes:
    need(route in ROUTES, f"unknown marker route: {route}")
    return canonical({
        "schema": "lay.m3-v8r1-one-shot-marker.v1",
        "task_id": TASK_ID,
        "transaction_id": TRANSACTION_ID,
        "route": route,
        "local_controller_sha256": admission["local_controller_sha256"],
        "remote_controller_sha256": admission["remote_controller_sha256"],
        "auditor_sha256": admission["auditor_sha256"],
        "one_shot": True,
        "retry_permitted": False,
    })


def verify_bootstrap_audit(path: pathlib.Path, controller_sha: str) -> dict[str, Any]:
    value = load_json(path)
    need(value.get("verdict") == "M3_V8R1_BOOTSTRAP_AUDIT_PASS_MARKERS_ADMITTED", "bootstrap audit did not admit markers")
    need(value.get("task_id") == TASK_ID and value.get("transaction_id") == TRANSACTION_ID, "bootstrap audit namespace drift")
    need(value.get("remote_controller_sha256") == controller_sha, "bootstrap audit controller SHA drift")
    for key in ("local_controller_sha256", "remote_controller_sha256", "auditor_sha256"):
        need(re.fullmatch(r"[0-9a-f]{64}", str(value.get(key, ""))) is not None, f"bootstrap audit lacks {key}")
    return value


def create_markers(audit_path: pathlib.Path, controller_sha: str) -> dict[str, Any]:
    need(os.geteuid() == 0, "marker creation requires root")
    audit = verify_bootstrap_audit(audit_path, controller_sha)
    markers = STATE / "markers"
    need(not markers.exists(), "V8 marker tree already exists")
    stage = STATE / f"markers.stage-{os.getpid()}-{time.time_ns()}"
    stage.mkdir(mode=0o700)
    try:
        rows = {}
        for route in ROUTES:
            name = f"{MARKER_NAMES[route]}.available"
            write_new(stage / name, marker_payload(route, audit), 0o400)
            rows[route] = file_row(stage / name)
        os.rename(stage, markers)
        fsync_dir(STATE)
    except BaseException:
        if stage.exists():
            shutil.rmtree(stage)
        raise
    append_state(1, "ALL_MARKERS_AVAILABLE", bootstrap_audit_sha256=sha256_file(audit_path))
    return {
        "schema": "lay.m3-v8r1-markers.v1",
        "task_id": TASK_ID,
        "transaction_id": TRANSACTION_ID,
        "verdict": "M3_V8R1_ALL_MARKERS_AVAILABLE",
        "markers": marker_inventory(),
        "marker_rows": rows,
        "markers_created": 2,
        "markers_consumed": 0,
        "cargo_invocations": 0,
        "subject_executions": 0,
        "runtime_authority_changed": False,
    }


def consume_marker(route: str, admission: Mapping[str, Any]) -> dict[str, Any]:
    markers = STATE / "markers"
    stem = MARKER_NAMES[route]
    available = markers / f"{stem}.available"
    consumed = markers / f"{stem}.consumed-before-exec"
    expected = marker_payload(route, admission)
    before = require_file(available, digest=sha256_bytes(expected), mode="0400")
    need(available.read_bytes() == expected and not consumed.exists(), f"marker state drift: {route}")
    os.rename(available, consumed)
    fsync_dir(markers)
    after = require_file(consumed, size=before["size_bytes"], digest=before["sha256"], mode="0400")
    return {"before": before, "after": after, "consumed_before_execution": True}


def latest_state() -> dict[str, Any] | None:
    if not STATE.is_dir():
        return None
    rows = sorted(STATE.glob("STATE-*.json"))
    return load_json(rows[-1]) if rows else None


def find_test_elf(target: pathlib.Path) -> pathlib.Path:
    candidates = []
    for path in (target / "release/deps").glob("lay-*"):
        if path.is_file() and os.access(path, os.X_OK):
            with path.open("rb") as source:
                if source.read(4) == b"\x7fELF":
                    candidates.append(path)
    need(len(candidates) == 1, f"expected one release test ELF, found {len(candidates)}")
    return candidates[0]


def elf_build_id(path: pathlib.Path) -> str:
    output = run(["/usr/bin/readelf", "-n", str(path)]).stdout
    match = re.search(rb"Build ID:\s*([0-9a-f]+)", output)
    need(match is not None, "ELF Build ID absent")
    return match.group(1).decode()


def build_once(audit_path: pathlib.Path, controller_sha: str) -> dict[str, Any]:
    need(os.geteuid() == 0, "build requires root")
    admission = verify_bootstrap_audit(audit_path, controller_sha)
    state = latest_state()
    need(state is not None and state.get("state") == "ALL_MARKERS_AVAILABLE", "build state predecessor drift")
    need(not BUILD.exists() and not (PARENT / "build-failure-v1").exists(), "V8 build already exists")

    stage = PARENT / f"build-v1.stage-{os.getpid()}-{time.time_ns()}"
    workspace = pathlib.Path("/home/e/.cache") / f"lay-m3-v8r1-build-{TRANSACTION_ID}"
    need(not workspace.exists(), "V8 build workspace already exists")
    stage.mkdir(mode=0o700)
    workspace.mkdir(mode=0o700)
    marker: dict[str, Any] | None = None
    cargo_started = False
    try:
        shutil.copytree(BOOTSTRAP / "source-closure", workspace, dirs_exist_ok=True)
        make_writable(workspace)
        guard = workspace / "scripts/cargo-guard.sh"
        guard.chmod(0o775)
        verify_source_closure(BOOTSTRAP / "source-closure")
        cargo = run(["/home/e/.cargo/bin/cargo", "-V"], env=controlled_environment()).stdout.decode().strip()
        rustc = run(["/home/e/.cargo/bin/rustc", "-Vv"], env=controlled_environment()).stdout.decode()
        need(cargo == EXPECTED_CARGO and all(token in rustc for token in EXPECTED_RUSTC), "build toolchain drift")
        environment = controlled_environment()
        environment.update(BUILD_ENVIRONMENT_FIXED)
        environment["CARGO_TARGET_DIR"] = str(workspace / "target")
        command = [
            str(guard),
            "test",
            "--offline",
            "--locked",
            "--release",
            "--lib",
            "--no-run",
            "m3_v8",
        ]
        write_json(stage / "PREBUILD.json", {
            "schema": "lay.m3-v8r1-prebuild.v1",
            "task_id": TASK_ID,
            "transaction_id": TRANSACTION_ID,
            "remote_controller_sha256": controller_sha,
            "source_manifest_sha256": sha256_file(BOOTSTRAP / "source-closure/SOURCE_MANIFEST.json"),
            "command": command,
            "environment": {key: environment[key] for key in BUILD_ENVIRONMENT_KEYS},
            "cargo_started": False,
            "retry_permitted": False,
        })
        fsync_dir(stage)
        marker = consume_marker("BUILD", admission)
        write_json(stage / "BUILD_MARKER_CONSUMED.json", marker)
        cargo_started = True
        with (stage / "cargo.log").open("wb") as log:
            result = run(
                command,
                cwd=workspace,
                env=environment,
                timeout=10_800,
                check=False,
                stdout=log,
                stderr=subprocess.STDOUT,
            )
            log.flush()
            os.fsync(log.fileno())
        need(result.returncode == 0, f"V8 Cargo exited {result.returncode}")
        elf = find_test_elf(workspace / "target")
        candidate = stage / "m3-v8r1-test-elf"
        shutil.copyfile(elf, candidate)
        candidate.chmod(0o555)
        shutil.copyfile(BOOTSTRAP / "source-closure/src/nanda_wave/l2_field/v13_typed_peak.rs", stage / "v13_typed_peak.rs")
        provenance = {
            "schema": "lay.m3-v8r1-build.v1",
            "task_id": TASK_ID,
            "transaction_id": TRANSACTION_ID,
            "verdict": "M3_V8R1_BUILD_CREATED_UNAUDITED",
            "source": {
                "source_manifest_sha256": sha256_file(BOOTSTRAP / "source-closure/SOURCE_MANIFEST.json"),
                "v13_typed_peak_sha256": EXPECTED_SOURCE["src/nanda_wave/l2_field/v13_typed_peak.rs"][1],
                "cargo_toml_sha256": EXPECTED_SOURCE["Cargo.toml"][1],
                "cargo_lock_sha256": EXPECTED_SOURCE["Cargo.lock"][1],
                "cargo_guard_sha256": EXPECTED_SOURCE["scripts/cargo-guard.sh"][1],
            },
            "build": {
                "command": command,
                "environment": {key: environment[key] for key in BUILD_ENVIRONMENT_KEYS},
                "cargo": cargo,
                "rustc": rustc.strip(),
                "cargo_invocations": 1,
                "rustc_compilations": 1,
                "exit_code": 0,
                "marker": marker,
                "retry_permitted": False,
            },
            "executable": {
                "path": str(BUILD / "m3-v8r1-test-elf"),
                "sha256": sha256_file(candidate),
                "size_bytes": candidate.stat().st_size,
                "mode": mode_string(candidate),
                "build_id": elf_build_id(candidate),
                "executed": False,
            },
            "subject_executions": 0,
            "perf_record_invocations": 0,
            "perf_stat_invocations": 0,
            "runtime_authority_changed": False,
        }
        write_json(stage / "BUILD_PROVENANCE.json", provenance)
        write_manifest(stage)
        seal_tree(stage)
        os.rename(stage, BUILD)
        fsync_dir(PARENT)
        append_state(2, "BUILD_CREATED_UNAUDITED", build_provenance_sha256=sha256_file(BUILD / "BUILD_PROVENANCE.json"))
        return provenance
    except BaseException as error:
        if marker is not None and stage.exists():
            failure = {
                "schema": "lay.m3-v8r1-build-failure.v1",
                "task_id": TASK_ID,
                "transaction_id": TRANSACTION_ID,
                "verdict": "BLOCKED_BUILD",
                "error": f"{type(error).__name__}: {error}",
                "cargo_started": cargo_started,
                "cargo_invocations": int(cargo_started),
                "marker": marker,
                "retry_permitted": False,
                "runtime_authority_changed": False,
            }
            write_json(stage / "BUILD_FAILURE.json", failure)
            write_manifest(stage)
            seal_tree(stage)
            failure_root = PARENT / "build-failure-v1"
            os.rename(stage, failure_root)
            fsync_dir(PARENT)
            append_state(2, "BLOCKED_BUILD", build_failure_sha256=sha256_file(failure_root / "BUILD_FAILURE.json"))
            return failure
        raise
    finally:
        if workspace.exists():
            shutil.rmtree(workspace)


def verify_build_audit(path: pathlib.Path, controller_sha: str) -> dict[str, Any]:
    value = load_json(path)
    need(value.get("verdict") == "M3_V8R1_BUILD_AUDIT_PASS_E2E_ADMITTED", "build audit did not admit E2E")
    need(value.get("task_id") == TASK_ID and value.get("transaction_id") == TRANSACTION_ID, "build audit namespace drift")
    need(value.get("remote_controller_sha256") == controller_sha, "build audit controller SHA drift")
    need(value.get("elf_sha256") == sha256_file(BUILD / "m3-v8r1-test-elf"), "build audit ELF SHA drift")
    return value


def verify_quiet_admission(path: pathlib.Path, controller_sha: str, build_audit: Mapping[str, Any]) -> dict[str, Any]:
    value = load_json(path)
    need(value.get("verdict") == "M3_V8R1_QUIET_HOST_E2E_ADMITTED", "quiet-host audit did not admit E2E")
    need(value.get("task_id") == TASK_ID and value.get("transaction_id") == TRANSACTION_ID, "quiet-host namespace drift")
    need(value.get("remote_controller_sha256") == controller_sha, "quiet-host controller SHA drift")
    need(value.get("build_audit_sha256") == sha256_file(path.parent / "BUILD_AUDIT.json") or value.get("elf_sha256") == build_audit.get("elf_sha256"), "quiet-host build binding drift")
    return value


def throttle_counters() -> dict[str, int]:
    values = {}
    for path in sorted(pathlib.Path("/sys/devices/system/cpu").glob("cpu*/thermal_throttle/*")):
        raw = path.read_text(errors="replace").strip()
        if raw.isdigit():
            values[str(path)] = int(raw)
    return values


def throttle_drift(before: Mapping[str, int], after: Mapping[str, int]) -> dict[str, list[int]]:
    return {
        key: [before.get(key, 0), after.get(key, 0)]
        for key in sorted(set(before) | set(after))
        if before.get(key, 0) != after.get(key, 0)
    }


def terminate(process: subprocess.Popen[bytes] | None) -> None:
    if process is None or process.poll() is not None:
        return
    with contextlib.suppress(ProcessLookupError):
        os.killpg(process.pid, signal.SIGTERM)
    with contextlib.suppress(subprocess.TimeoutExpired):
        process.wait(timeout=3)
    if process.poll() is None:
        with contextlib.suppress(ProcessLookupError):
            os.killpg(process.pid, signal.SIGKILL)
        process.wait()


def scientific_environment(subject: pathlib.Path) -> dict[str, str]:
    paths = input_paths()
    environment = controlled_environment()
    environment.update({
        "LAY_M3_ACTUAL_OWNER_PACKAGE": str(paths["v13"]),
        "LAY_M3_ACTUAL_OWNER_V7": str(paths["v7"]),
        "LAY_M3_V8_EVIDENCE_DIR": str(subject / "evidence"),
        "LAY_M3_V8_RECEIPT": str(subject / "SUBJECT_RECEIPT.json"),
        "LAY_L2_PACKAGE": str(paths["v13"]),
        "LAY_L2_PRODUCTIVE_V1_PACKAGE": str(paths["productive"]),
        "LAY_L11_RECEIPT": str(paths["l11_receipt"]),
    })
    return environment


def e2e_once(build_audit_path: pathlib.Path, quiet_path: pathlib.Path, controller_sha: str) -> dict[str, Any]:
    need(os.geteuid() == 0, "E2E requires root controller")
    build_audit = verify_build_audit(build_audit_path, controller_sha)
    quiet = verify_quiet_admission(quiet_path, controller_sha, build_audit)
    state = latest_state()
    need(state is not None and state.get("state") == "BUILD_CREATED_UNAUDITED", "E2E state predecessor drift")
    need(not E2E.exists() and not (PARENT / "e2e-failure-v1").exists(), "V8 E2E evidence already exists")

    stage = PARENT / f"e2e-v1.stage-{os.getpid()}-{time.time_ns()}"
    subject = stage / "subject"
    final_subject = E2E / "subject"
    evidence = subject / "evidence"
    stage.mkdir(mode=0o755)
    subject.mkdir(mode=0o700)
    evidence.mkdir(mode=0o700)
    shutil.chown(subject, user="e", group="e")
    shutil.chown(evidence, user="e", group="e")
    environment = scientific_environment(final_subject)
    command = [
        "/usr/bin/sudo",
        "-n",
        "-u",
        "e",
        "/usr/bin/env",
        *[f"{key}={value}" for key, value in sorted(environment.items())],
        "/usr/bin/taskset",
        "-c",
        "0",
        str(LOADER),
        str(BUILD / "m3-v8r1-test-elf"),
        "--ignored",
        "--exact",
        SCIENTIFIC_TEST,
        "--nocapture",
        "--test-threads=1",
    ]
    write_json(stage / "PRE_E2E.json", {
        "schema": "lay.m3-v8r1-pre-e2e.v1",
        "task_id": TASK_ID,
        "transaction_id": TRANSACTION_ID,
        "remote_controller_sha256": controller_sha,
        "build_audit_sha256": sha256_file(build_audit_path),
        "quiet_admission_sha256": sha256_file(quiet_path),
        "elf_sha256": build_audit["elf_sha256"],
        "command": command,
        "environment": environment,
        "subject_started": False,
        "retry_permitted": False,
    })
    fsync_dir(stage)
    marker = consume_marker("E2E", verify_bootstrap_audit(quiet_path, controller_sha) if quiet.get("verdict") == "M3_V8R1_BOOTSTRAP_AUDIT_PASS_MARKERS_ADMITTED" else quiet)
    # Quiet admission carries the same controller/auditor identities needed by the marker schema.
    write_json(stage / "E2E_MARKER_CONSUMED.json", marker)
    os.rename(stage, E2E)
    fsync_dir(PARENT)
    root = E2E
    subject = root / "subject"
    thermal_before = throttle_counters()
    process: subprocess.Popen[bytes] | None = None
    stdout = b""
    stderr = b""
    controller_error = None
    try:
        process = subprocess.Popen(command, stdout=subprocess.PIPE, stderr=subprocess.PIPE, start_new_session=True)
        stdout, stderr = process.communicate(timeout=10_800)
    except BaseException as error:
        controller_error = f"{type(error).__name__}: {error}"
        terminate(process)
        if process is not None:
            with contextlib.suppress(Exception):
                stdout, stderr = process.communicate(timeout=1)
    thermal_after = throttle_counters()
    thermal_change = throttle_drift(thermal_before, thermal_after)
    write_new(root / "stdout.log", stdout)
    write_new(root / "stderr.log", stderr)
    receipt_path = subject / "SUBJECT_RECEIPT.json"
    receipt = None
    if receipt_path.is_file():
        try:
            receipt = load_json(receipt_path)
        except BaseException as error:
            controller_error = controller_error or f"{type(error).__name__}: {error}"
    complete = isinstance(receipt, dict) and receipt.get("schema") == "lay.m3-end-to-end-test-owner.v1"
    positive = complete and receipt.get("verdict") == "M3_END_TO_END_TEST_OWNER_PASS"
    exit_code = process.returncode if process is not None else None
    exit_consistent = (positive and exit_code == 0) or (complete and not positive and exit_code not in (None, 0))
    producer_complete = controller_error is None and complete and exit_consistent
    wrapper = {
        "schema": "lay.m3-v8r1-e2e-wrapper.v1",
        "task_id": TASK_ID,
        "transaction_id": TRANSACTION_ID,
        "verdict": "M3_V8R1_E2E_CREATED_UNAUDITED" if producer_complete else "BLOCKED_PROVENANCE",
        "controller_error": controller_error,
        "marker": marker,
        "command": command,
        "environment": environment,
        "exit_code": exit_code,
        "subject_receipt": receipt,
        "outputs_complete": complete,
        "thermal_before": thermal_before,
        "thermal_after": thermal_after,
        "thermal_throttle_drift": thermal_change,
        "subject_executions": int(process is not None),
        "cargo_invocations": 0,
        "perf_record_invocations": 0,
        "perf_stat_invocations": 0,
        "runtime_authority_changed": False,
        "retry_permitted": False,
    }
    write_json(root / "E2E_WRAPPER.json", wrapper)
    write_manifest(root)
    seal_tree(root)
    append_state(
        3,
        "E2E_CREATED_UNAUDITED" if producer_complete else "BLOCKED_PROVENANCE",
        e2e_wrapper_sha256=sha256_file(root / "E2E_WRAPPER.json"),
    )
    return wrapper


def status() -> dict[str, Any]:
    processes = []
    for path in pathlib.Path("/proc").iterdir():
        if not path.name.isdigit():
            continue
        try:
            command = (path / "cmdline").read_bytes().replace(b"\0", b" ").decode(errors="replace").strip()
        except (FileNotFoundError, PermissionError, ProcessLookupError):
            continue
        if command and any(token in command for token in (TASK_ID, SCIENTIFIC_TEST, "perf record", "perf stat")):
            processes.append({"pid": int(path.name), "command": command})
    return {
        "schema": "lay.m3-v8r1-status.v1",
        "task_id": TASK_ID,
        "transaction_id": TRANSACTION_ID,
        "verdict": "M3_V8R1_STATUS",
        "parent_exists": PARENT.exists(),
        "state_exists": STATE.exists(),
        "build_exists": BUILD.exists(),
        "e2e_exists": E2E.exists(),
        "latest_state": latest_state(),
        "markers": marker_inventory(),
        "active_owned_processes": sorted(processes, key=lambda row: row["pid"]),
    }


def self_check() -> dict[str, Any]:
    build_tail = ("test", "--offline", "--locked", "--release", "--lib", "--no-run", "m3_v8")
    e2e_tail = ("--ignored", "--exact", SCIENTIFIC_TEST, "--nocapture", "--test-threads=1")
    need(ACTIONS == ("self-check", "status", "bootstrap", "create-markers", "build-once", "e2e-once"), "action registry drift")
    need(ROUTES == ("BUILD", "E2E") and set(MARKER_NAMES) == set(ROUTES), "route registry drift")
    need("perf" not in " ".join(build_tail + e2e_tail), "perf route became reachable")
    need("--pid" not in build_tail + e2e_tail, "attach route became reachable")
    need(SCIENTIFIC_TEST.endswith("m3_end_to_end_physical_proof"), "scientific test drift")
    return {
        "schema": "lay.m3-v8r1-remote-self-check.v1",
        "verdict": "M3_V8R1_REMOTE_CONTROLLER_STATIC_PASS",
        "task_id": TASK_ID,
        "transaction_id": TRANSACTION_ID,
        "controller_sha256": sha256_file(pathlib.Path(__file__)),
        "actions": list(ACTIONS),
        "routes": list(ROUTES),
        "markers": ["build.available", "e2e.available"],
        "build_argv_tail": list(build_tail),
        "e2e_argv_tail": list(e2e_tail),
        "perf_record_invocations": 0,
        "perf_stat_invocations": 0,
        "production_authority_admitted": False,
    }


def with_route_lock(callback: Any) -> dict[str, Any]:
    lock_path = STATE / "route.lock"
    need(lock_path.is_file(), "route lock absent")
    with lock_path.open("rb") as lock:
        fcntl.flock(lock.fileno(), fcntl.LOCK_EX | fcntl.LOCK_NB)
        return callback()


def parser() -> argparse.ArgumentParser:
    value = argparse.ArgumentParser()
    value.add_argument("action", choices=ACTIONS)
    value.add_argument("--bootstrap", type=pathlib.Path)
    value.add_argument("--audit", type=pathlib.Path)
    value.add_argument("--build-audit", type=pathlib.Path)
    value.add_argument("--quiet", type=pathlib.Path)
    return value


def main() -> int:
    args = parser().parse_args()
    try:
        controller_sha = sha256_file(pathlib.Path(__file__))
        if args.action == "self-check":
            value = self_check()
        elif args.action == "status":
            value = status()
        elif args.action == "bootstrap":
            need(args.bootstrap is not None, "--bootstrap is required")
            value = bootstrap_once(args.bootstrap)
        elif args.action == "create-markers":
            need(args.audit is not None, "--audit is required")
            value = with_route_lock(lambda: create_markers(args.audit, controller_sha))
        elif args.action == "build-once":
            need(args.audit is not None, "--audit is required")
            value = with_route_lock(lambda: build_once(args.audit, controller_sha))
        else:
            need(args.build_audit is not None and args.quiet is not None, "--build-audit and --quiet are required")
            value = with_route_lock(lambda: e2e_once(args.build_audit, args.quiet, controller_sha))
        print(json.dumps(value, sort_keys=True))
        return 0
    except BaseException as error:
        print(json.dumps({
            "schema": "lay.m3-v8r1-remote-error.v1",
            "task_id": TASK_ID,
            "transaction_id": TRANSACTION_ID,
            "verdict": "BLOCKED_PROVENANCE",
            "error": f"{type(error).__name__}: {error}",
            "retry_permitted": False,
            "runtime_authority_changed": False,
        }, sort_keys=True))
        return 1


if __name__ == "__main__":
    sys.exit(main())
