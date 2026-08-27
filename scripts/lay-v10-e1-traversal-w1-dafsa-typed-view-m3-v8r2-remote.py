#!/usr/bin/env python3
"""Fail-closed remote producer for the M3 V8R2 direct-exec proof."""

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


TASK_ID = "slice8b-v10-e1-traversal-w1-dafsa-typed-view-m3-e2e-v8r2-direct-exec-20260827"
TRANSACTION_ID = "59694b7b9f0327d78896b5bc4797671f54478674186558e338e4a1b0d9ef7813"
V8R1_TASK_ID = "slice8b-v10-e1-traversal-w1-dafsa-typed-view-m3-e2e-v8r1-20260827"
HOSTNAME = "e-MEGA-MINI-M1-13th"
MACHINE_ID_SHA256 = "5ac0bb537745673ef80942c2df32d586f5753c85de1c8b2453aa4e7317b77441"
KERNEL = "6.8.0-124-generic"

PARENT = pathlib.Path("/home/e/.local/share/lay/provenance") / TASK_ID
STATE = pathlib.Path("/home/e/.local/state/lay") / TASK_ID
BOOTSTRAP = PARENT / "bootstrap-v1"
EXECUTABLE = BOOTSTRAP / "m3-v8r2-test-elf"
E2E = PARENT / "e2e-v1"
V8R1_PARENT = pathlib.Path("/home/e/.local/share/lay/provenance") / V8R1_TASK_ID
V8R1_STATE = pathlib.Path("/home/e/.local/state/lay") / V8R1_TASK_ID
V8R1_ELF = V8R1_PARENT / "build-v1/m3-v8r1-test-elf"
V8R1_INPUTS = V8R1_PARENT / "bootstrap-v1/inputs"

ACTIONS = ("self-check", "status", "bootstrap", "create-marker", "e2e-once")
ROUTES = ("E2E",)
MARKER_NAME = "e2e"
SCIENTIFIC_TEST = (
    "nanda_wave::l2_field::v13_typed_peak::tests::m3_v8::"
    "m3_end_to_end_physical_proof"
)
EXPECTED_ELF_SIZE = 320_613_368
EXPECTED_ELF_SHA256 = "0af3cc6679396650245e924976d8a3bb432dfa6a1086b1c46bcfa7497307afea"
EXPECTED_BUILD_ID = "c6ddac7181428a303cbc51be61dd3bb115677562"
EXPECTED_BUILD_AUDIT_SHA256 = "d7d5e7110171e5c6546016ff0c9374c323804014ef8cfa7a690ad7d1d11c8340"
EXPECTED_TERMINAL_AUDIT_SHA256 = "04d0e17158a63a49088e8c8ff9dc25df67e50ac6a97b770ea3fcf1a73d67ec91"
EXPECTED_DIAGNOSIS_SHA256 = "9b05af87d83c937dcc1e4eab0e398ab3d93ef49ac3e0bfb8089a58ba3d64bae0"
EXPECTED_INPUTS = {
    "LAY-L2-RU-FULL-v13.bin": (140_556_462, "cce259fe0ce5dce67702383363b66f0fe9b9ff5a87d8f01c4fcf342d91218d7b"),
    "slice8b-v7-fixed-13x100.json": (1_606_189, "33fded73e13f565bdc08a83bd440be67c80f6d85a196453050b09bc9f7ef28e4"),
    "LAY-L2-PRODUCTIVE-PARADIGM-v90.p2m": (17_309_944, "40fb6a9f0d92c3c7502e47f9c70230d9b86020f622b08a5c799342f13e09ce44"),
    "LAY-L2-PRODUCTIVE-PARADIGM-v90.p2r": (2_123_112, "de7972c80448dc792759d70de99cda6ec48c3d6af337763856601db563ab167e"),
    "LAY-L1.1-RU-COMPOSITE-EN300K-PHASE8I-v9.v9.bin": (77_962_328, "bf5a1619a89038466ef786305cf35eda5f4af5b9f12b9140f7d3cac407e2f2a7"),
    "l11-proof.json": (539_536, "4983930495e793c1d28c7558fe006ddf8097ee575bebb1afd3f1dba4ddb1d01d"),
}


class V8R2RemoteError(RuntimeError):
    pass


def need(condition: bool, message: str) -> None:
    if not condition:
        raise V8R2RemoteError(message)


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
    env: Mapping[str, str] | None = None,
    timeout: float = 3_600,
    check: bool = True,
) -> subprocess.CompletedProcess[bytes]:
    result = subprocess.run(
        list(argv),
        env=dict(env) if env is not None else None,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        timeout=timeout,
        check=False,
    )
    if check and result.returncode != 0:
        raise V8R2RemoteError(
            f"command failed ({result.returncode}): {list(argv)!r}\n"
            f"{result.stderr[-4000:].decode(errors='replace')}"
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


def seal_tree(root: pathlib.Path, executable: pathlib.Path | None = None) -> None:
    for path in sorted(root.rglob("*"), reverse=True):
        if path.is_dir():
            path.chmod(0o555)
        else:
            path.chmod(0o555 if executable is not None and path == executable else 0o444)
    root.chmod(0o555)


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
        rows[relative] = require_file(
            root / relative,
            size=int(expected.get("size_bytes", -1)),
            digest=str(expected.get("sha256", "")),
        )
    return rows


def elf_build_id(path: pathlib.Path) -> str:
    output = run(["/usr/bin/readelf", "-n", str(path)]).stdout
    match = re.search(rb"Build ID:\s*([0-9a-f]+)", output)
    need(match is not None, "ELF Build ID absent")
    return match.group(1).decode()


def marker_inventory() -> dict[str, list[str]]:
    markers = STATE / "markers"
    if not markers.is_dir():
        return {"available": [], "consumed": []}
    return {
        "available": sorted(path.name for path in markers.glob("*.available")),
        "consumed": sorted(path.name for path in markers.glob("*.consumed-before-exec")),
    }


def latest_state(root: pathlib.Path = STATE) -> dict[str, Any] | None:
    if not root.is_dir():
        return None
    rows = sorted(root.glob("STATE-*.json"))
    return load_json(rows[-1]) if rows else None


def append_state(sequence: int, state: str, **extra: Any) -> None:
    write_json(
        STATE / f"STATE-{sequence:02d}-{state}.json",
        {
            "schema": "lay.m3-v8r2-remote-state.v1",
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


def verify_host() -> dict[str, Any]:
    need(os.uname().nodename == HOSTNAME, "remote hostname drift")
    need(os.uname().release == KERNEL, "remote kernel drift")
    machine = require_file(pathlib.Path("/etc/machine-id"))
    need(machine["sha256"] == MACHINE_ID_SHA256, "remote machine-id drift")
    online = pathlib.Path("/sys/devices/system/cpu/online").read_text().strip()
    core = pathlib.Path("/sys/bus/event_source/devices/cpu_core/cpus").read_text().strip()
    atom = pathlib.Path("/sys/bus/event_source/devices/cpu_atom/cpus").read_text().strip()
    need((online, core, atom) == ("0-19", "0-11", "12-19"), "remote CPU topology drift")
    return {
        "hostname": HOSTNAME,
        "kernel": KERNEL,
        "machine_id_sha256": machine["sha256"],
        "online": online,
        "core": core,
        "atom": atom,
    }


def input_paths() -> dict[str, pathlib.Path]:
    return {
        "v13": V8R1_INPUTS / "LAY-L2-RU-FULL-v13.bin",
        "v7": V8R1_INPUTS / "slice8b-v7-fixed-13x100.json",
        "productive": V8R1_INPUTS / "LAY-L2-PRODUCTIVE-PARADIGM-v90.p2m",
        "recovery": V8R1_INPUTS / "LAY-L2-PRODUCTIVE-PARADIGM-v90.p2r",
        "l11": V8R1_INPUTS / "LAY-L1.1-RU-COMPOSITE-EN300K-PHASE8I-v9.v9.bin",
        "l11_proof": V8R1_INPUTS / "l11-proof.json",
        "l11_receipt": V8R1_INPUTS / "l11-installed.json",
    }


def verify_v8r1_predecessor(payload_root: pathlib.Path) -> dict[str, Any]:
    build_audit_path = payload_root / "V8R1_BUILD_AUDIT.json"
    terminal_audit_path = payload_root / "V8R1_TERMINAL_AUDIT.json"
    diagnosis_path = payload_root / "V8R1_DIAGNOSIS.json"
    require_file(build_audit_path, digest=EXPECTED_BUILD_AUDIT_SHA256, mode="0444")
    require_file(terminal_audit_path, digest=EXPECTED_TERMINAL_AUDIT_SHA256, mode="0444")
    require_file(diagnosis_path, digest=EXPECTED_DIAGNOSIS_SHA256, mode="0444")
    build = load_json(build_audit_path)
    terminal = load_json(terminal_audit_path)
    diagnosis = load_json(diagnosis_path)
    need(build.get("verdict") == "M3_V8R1_BUILD_AUDIT_PASS_E2E_ADMITTED", "V8R1 build audit verdict drift")
    need(terminal.get("verdict") == "BLOCKED_PROVENANCE", "V8R1 terminal verdict drift")
    need(diagnosis.get("verdict") == "V8R1_LOADER_CURRENT_EXE_DEFECT_CONFIRMED", "V8R1 diagnosis verdict drift")
    original = require_file(
        V8R1_ELF,
        size=EXPECTED_ELF_SIZE,
        digest=EXPECTED_ELF_SHA256,
        mode="0444",
    )
    need(elf_build_id(V8R1_ELF) == EXPECTED_BUILD_ID, "V8R1 Build ID drift")
    v8r1_markers = V8R1_STATE / "markers"
    consumed = sorted(path.name for path in v8r1_markers.glob("*.consumed-before-exec"))
    available = sorted(path.name for path in v8r1_markers.glob("*.available"))
    need(consumed == ["build.consumed-before-exec", "e2e.consumed-before-exec"] and not available, "V8R1 marker history drift")
    need(latest_state(V8R1_STATE).get("state") == "BLOCKED_PROVENANCE", "V8R1 terminal state drift")
    inputs = {}
    for name, (size, digest) in EXPECTED_INPUTS.items():
        inputs[name] = require_file(V8R1_INPUTS / name, size=size, digest=digest, mode="0444")
    l11_receipt = load_json(V8R1_INPUTS / "l11-installed.json")
    need(l11_receipt.get("artifact_sha256") == EXPECTED_INPUTS["LAY-L1.1-RU-COMPOSITE-EN300K-PHASE8I-v9.v9.bin"][1], "L1.1 receipt drift")
    return {
        "build_audit_sha256": sha256_file(build_audit_path),
        "terminal_audit_sha256": sha256_file(terminal_audit_path),
        "diagnosis_sha256": sha256_file(diagnosis_path),
        "elf": original,
        "build_id": EXPECTED_BUILD_ID,
        "inputs": inputs,
        "markers": {"available": available, "consumed": consumed},
    }


def verify_execution_admission(path: pathlib.Path, controller_sha: str) -> dict[str, Any]:
    value = load_json(path)
    need(value.get("verdict") == "M3_V8R2_EXECUTION_ADMITTED", "execution admission verdict drift")
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
        "f=open(a,'xb'); f.write(b'm3-v8r2-uid-proof\\n'); f.flush(); os.fsync(f.fileno()); f.close(); "
        "os.rename(a,b); assert b.read_bytes()==b'm3-v8r2-uid-proof\\n'; b.unlink()"
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


def bootstrap_once(path: pathlib.Path) -> dict[str, Any]:
    need(os.geteuid() == 0, "bootstrap requires root")
    need(not PARENT.exists() and not STATE.exists(), "V8R2 remote namespace already exists")
    payload = load_json(path / "PAYLOAD.json")
    need(payload.get("schema") == "lay.m3-v8r2-bootstrap-payload.v1", "bootstrap payload schema drift")
    need(payload.get("task_id") == TASK_ID and payload.get("transaction_id") == TRANSACTION_ID, "bootstrap payload namespace drift")
    controller_sha = sha256_file(path / "remote-controller.py")
    admission = verify_execution_admission(path / "EXECUTION_ADMISSION.json", controller_sha)
    inventory = verify_inventory(path, payload)
    host = verify_host()
    predecessor = verify_v8r1_predecessor(path)

    stage = pathlib.Path(f"{PARENT}.stage-{os.getpid()}-{time.time_ns()}")
    state_stage = pathlib.Path(f"{STATE}.stage-{os.getpid()}-{time.time_ns()}")
    stage.mkdir(parents=True, mode=0o755)
    state_stage.mkdir(parents=True, mode=0o700)
    try:
        copied = stage / "bootstrap-v1"
        shutil.copytree(path, copied)
        executable = copied / "m3-v8r2-test-elf"
        shutil.copyfile(V8R1_ELF, executable)
        executable.chmod(0o555)
        copied_elf = require_file(
            executable,
            size=EXPECTED_ELF_SIZE,
            digest=EXPECTED_ELF_SHA256,
            mode="0555",
        )
        copied_elf["path"] = str(EXECUTABLE)
        need(elf_build_id(executable) == EXPECTED_BUILD_ID, "V8R2 copied Build ID drift")
        need(require_file(V8R1_ELF, digest=EXPECTED_ELF_SHA256, mode="0444")["sha256"] == copied_elf["sha256"], "source ELF mutated")
        uid = uid_capability_probe(stage)
        write_new(state_stage / "route.lock", b"m3-v8r2-route-lock\n", 0o600)
        receipt = {
            "schema": "lay.m3-v8r2-bootstrap.v1",
            "task_id": TASK_ID,
            "transaction_id": TRANSACTION_ID,
            "verdict": "M3_V8R2_BOOTSTRAP_CREATED_UNAUDITED",
            "controller_sha256": controller_sha,
            "execution_admission_sha256": sha256_file(path / "EXECUTION_ADMISSION.json"),
            "execution_admission_verdict": admission["verdict"],
            "host": host,
            "v8r1_predecessor": predecessor,
            "executable_copy": copied_elf,
            "executable_build_id": EXPECTED_BUILD_ID,
            "input_bindings": {key: str(value) for key, value in input_paths().items()},
            "uploaded_files": len(inventory),
            "uid_capability": uid,
            "markers_expected": 1,
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
        seal_tree(stage, executable)
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


def verify_bootstrap_audit(path: pathlib.Path, controller_sha: str) -> dict[str, Any]:
    value = load_json(path)
    need(value.get("verdict") == "M3_V8R2_BOOTSTRAP_AUDIT_PASS_MARKER_ADMITTED", "bootstrap audit did not admit marker")
    need(value.get("task_id") == TASK_ID and value.get("transaction_id") == TRANSACTION_ID, "bootstrap audit namespace drift")
    need(value.get("remote_controller_sha256") == controller_sha, "bootstrap audit controller SHA drift")
    for key in ("local_controller_sha256", "remote_controller_sha256", "auditor_sha256"):
        need(re.fullmatch(r"[0-9a-f]{64}", str(value.get(key, ""))) is not None, f"bootstrap audit lacks {key}")
    return value


def marker_payload(admission: Mapping[str, Any]) -> bytes:
    return canonical({
        "schema": "lay.m3-v8r2-one-shot-marker.v1",
        "task_id": TASK_ID,
        "transaction_id": TRANSACTION_ID,
        "route": "E2E",
        "local_controller_sha256": admission["local_controller_sha256"],
        "remote_controller_sha256": admission["remote_controller_sha256"],
        "auditor_sha256": admission["auditor_sha256"],
        "one_shot": True,
        "retry_permitted": False,
    })


def create_marker(audit_path: pathlib.Path, controller_sha: str) -> dict[str, Any]:
    need(os.geteuid() == 0, "marker creation requires root")
    audit = verify_bootstrap_audit(audit_path, controller_sha)
    markers = STATE / "markers"
    need(not markers.exists(), "V8R2 marker tree already exists")
    stage = STATE / f"markers.stage-{os.getpid()}-{time.time_ns()}"
    stage.mkdir(mode=0o700)
    try:
        marker = stage / "e2e.available"
        write_new(marker, marker_payload(audit), 0o400)
        row = file_row(marker)
        os.rename(stage, markers)
        fsync_dir(STATE)
    except BaseException:
        if stage.exists():
            shutil.rmtree(stage)
        raise
    append_state(1, "E2E_MARKER_AVAILABLE", bootstrap_audit_sha256=sha256_file(audit_path))
    return {
        "schema": "lay.m3-v8r2-marker.v1",
        "task_id": TASK_ID,
        "transaction_id": TRANSACTION_ID,
        "verdict": "M3_V8R2_E2E_MARKER_AVAILABLE",
        "markers": marker_inventory(),
        "marker_row": row,
        "markers_created": 1,
        "markers_consumed": 0,
        "cargo_invocations": 0,
        "rustc_compilations": 0,
        "subject_executions": 0,
        "runtime_authority_changed": False,
    }


def verify_quiet_admission(path: pathlib.Path, controller_sha: str) -> dict[str, Any]:
    value = load_json(path)
    need(value.get("verdict") == "M3_V8R2_QUIET_HOST_E2E_ADMITTED", "quiet admission verdict drift")
    need(value.get("task_id") == TASK_ID and value.get("transaction_id") == TRANSACTION_ID, "quiet admission namespace drift")
    need(value.get("remote_controller_sha256") == controller_sha, "quiet admission controller SHA drift")
    need(value.get("elf_sha256") == EXPECTED_ELF_SHA256, "quiet admission ELF drift")
    return value


def consume_marker(admission: Mapping[str, Any]) -> dict[str, Any]:
    markers = STATE / "markers"
    available = markers / "e2e.available"
    consumed = markers / "e2e.consumed-before-exec"
    expected = marker_payload(admission)
    before = require_file(available, digest=sha256_bytes(expected), mode="0400")
    need(available.read_bytes() == expected and not consumed.exists(), "E2E marker state drift")
    os.rename(available, consumed)
    fsync_dir(markers)
    after = require_file(consumed, size=before["size_bytes"], digest=before["sha256"], mode="0400")
    return {"before": before, "after": after, "consumed_before_execution": True}


def throttle_counters() -> dict[str, int]:
    rows = {}
    for path in pathlib.Path("/sys/devices/system/cpu").glob("cpu*/thermal_throttle/*"):
        try:
            value = path.read_text().strip()
        except OSError:
            continue
        if value.isdigit():
            rows[str(path)] = int(value)
    return rows


def throttle_drift(before: Mapping[str, int], after: Mapping[str, int]) -> dict[str, list[int]]:
    return {
        key: [int(before.get(key, -1)), int(after.get(key, -1))]
        for key in sorted(set(before) | set(after))
        if before.get(key) != after.get(key)
    }


def terminate(process: subprocess.Popen[bytes] | None) -> None:
    if process is None or process.poll() is not None:
        return
    with contextlib.suppress(ProcessLookupError):
        os.killpg(process.pid, signal.SIGKILL)
    with contextlib.suppress(Exception):
        process.wait(timeout=10)


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


def scientific_command(subject: pathlib.Path) -> list[str]:
    environment = scientific_environment(subject)
    return [
        "/usr/bin/sudo",
        "-n",
        "-u",
        "e",
        "/usr/bin/env",
        *[f"{key}={value}" for key, value in sorted(environment.items())],
        "/usr/bin/taskset",
        "-c",
        "0",
        str(EXECUTABLE),
        "--ignored",
        "--exact",
        SCIENTIFIC_TEST,
        "--nocapture",
        "--test-threads=1",
    ]


def observe_direct_parent(process: subprocess.Popen[bytes], timeout: float = 5.0) -> dict[str, Any]:
    deadline = time.monotonic() + timeout
    target = str(EXECUTABLE.resolve())
    while time.monotonic() < deadline:
        rows = []
        for path in pathlib.Path("/proc").iterdir():
            if not path.name.isdigit():
                continue
            try:
                executable = str((path / "exe").resolve())
                command = (path / "cmdline").read_bytes().split(b"\0")
                argv = [item.decode(errors="replace") for item in command if item]
            except (FileNotFoundError, PermissionError, ProcessLookupError):
                continue
            if executable == target and SCIENTIFIC_TEST in argv:
                rows.append({"pid": int(path.name), "executable": executable, "argv": argv})
        if rows:
            rows.sort(key=lambda row: row["pid"])
            return {"observed": True, "target": target, "processes": rows}
        if process.poll() is not None:
            break
        time.sleep(0.01)
    return {"observed": False, "target": target, "processes": []}


def e2e_once(quiet_path: pathlib.Path, controller_sha: str) -> dict[str, Any]:
    need(os.geteuid() == 0, "E2E requires root controller")
    quiet = verify_quiet_admission(quiet_path, controller_sha)
    state = latest_state()
    need(state is not None and state.get("state") == "E2E_MARKER_AVAILABLE", "E2E state predecessor drift")
    need(not E2E.exists() and not (PARENT / "e2e-failure-v1").exists(), "V8R2 E2E evidence already exists")
    require_file(EXECUTABLE, size=EXPECTED_ELF_SIZE, digest=EXPECTED_ELF_SHA256, mode="0555")
    need(elf_build_id(EXECUTABLE) == EXPECTED_BUILD_ID, "V8R2 executable Build ID drift")

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
    command = scientific_command(final_subject)
    executable_index = command.index(str(EXECUTABLE))
    need(command[executable_index - 3 : executable_index] == ["/usr/bin/taskset", "-c", "0"], "direct command prefix drift")
    need(not any("ld-linux" in token for token in command), "loader became reachable")
    write_json(stage / "PRE_E2E.json", {
        "schema": "lay.m3-v8r2-pre-e2e.v1",
        "task_id": TASK_ID,
        "transaction_id": TRANSACTION_ID,
        "remote_controller_sha256": controller_sha,
        "quiet_admission_sha256": sha256_file(quiet_path),
        "elf_sha256": EXPECTED_ELF_SHA256,
        "elf_build_id": EXPECTED_BUILD_ID,
        "command": command,
        "environment": environment,
        "subject_started": False,
        "retry_permitted": False,
    })
    fsync_dir(stage)
    marker = consume_marker(quiet)
    write_json(stage / "E2E_MARKER_CONSUMED.json", marker)
    os.rename(stage, E2E)
    fsync_dir(PARENT)

    thermal_before = throttle_counters()
    process: subprocess.Popen[bytes] | None = None
    stdout = b""
    stderr = b""
    controller_error = None
    direct_identity: dict[str, Any] = {"observed": False, "target": str(EXECUTABLE), "processes": []}
    try:
        process = subprocess.Popen(command, stdout=subprocess.PIPE, stderr=subprocess.PIPE, start_new_session=True)
        direct_identity = observe_direct_parent(process)
        need(direct_identity["observed"] is True, "direct executable identity was not observed")
        stdout, stderr = process.communicate(timeout=10_800)
    except BaseException as error:
        controller_error = f"{type(error).__name__}: {error}"
        terminate(process)
        if process is not None:
            with contextlib.suppress(Exception):
                stdout, stderr = process.communicate(timeout=1)
    thermal_after = throttle_counters()
    thermal_change = throttle_drift(thermal_before, thermal_after)
    write_new(E2E / "stdout.log", stdout)
    write_new(E2E / "stderr.log", stderr)
    receipt_path = E2E / "subject/SUBJECT_RECEIPT.json"
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
    producer_complete = controller_error is None and direct_identity["observed"] is True and complete and exit_consistent
    wrapper = {
        "schema": "lay.m3-v8r2-e2e-wrapper.v1",
        "task_id": TASK_ID,
        "transaction_id": TRANSACTION_ID,
        "verdict": "M3_V8R2_E2E_CREATED_UNAUDITED" if producer_complete else "BLOCKED_PROVENANCE",
        "controller_error": controller_error,
        "marker": marker,
        "command": command,
        "environment": environment,
        "direct_exec_identity": direct_identity,
        "exit_code": exit_code,
        "subject_receipt": receipt,
        "outputs_complete": complete,
        "thermal_before": thermal_before,
        "thermal_after": thermal_after,
        "thermal_throttle_drift": thermal_change,
        "subject_executions": int(process is not None),
        "cargo_invocations": 0,
        "rustc_compilations": 0,
        "perf_record_invocations": 0,
        "perf_stat_invocations": 0,
        "runtime_authority_changed": False,
        "retry_permitted": False,
    }
    write_json(E2E / "E2E_WRAPPER.json", wrapper)
    write_manifest(E2E)
    seal_tree(E2E)
    append_state(
        2,
        "E2E_CREATED_UNAUDITED" if producer_complete else "BLOCKED_PROVENANCE",
        e2e_wrapper_sha256=sha256_file(E2E / "E2E_WRAPPER.json"),
    )
    return wrapper


def owned_processes() -> list[dict[str, Any]]:
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
    return sorted(processes, key=lambda row: row["pid"])


def status() -> dict[str, Any]:
    return {
        "schema": "lay.m3-v8r2-status.v1",
        "task_id": TASK_ID,
        "transaction_id": TRANSACTION_ID,
        "verdict": "M3_V8R2_STATUS",
        "parent_exists": PARENT.exists(),
        "state_exists": STATE.exists(),
        "bootstrap_exists": BOOTSTRAP.exists(),
        "executable_exists": EXECUTABLE.exists(),
        "e2e_exists": E2E.exists(),
        "latest_state": latest_state(),
        "markers": marker_inventory(),
        "active_owned_processes": owned_processes(),
    }


def self_check() -> dict[str, Any]:
    command = scientific_command(E2E / "subject")
    executable_index = command.index(str(EXECUTABLE))
    need(ACTIONS == ("self-check", "status", "bootstrap", "create-marker", "e2e-once"), "action registry drift")
    need(ROUTES == ("E2E",) and MARKER_NAME == "e2e", "route registry drift")
    need(command[executable_index - 3 : executable_index] == ["/usr/bin/taskset", "-c", "0"], "direct command drift")
    need(command[executable_index + 1 :] == ["--ignored", "--exact", SCIENTIFIC_TEST, "--nocapture", "--test-threads=1"], "scientific argv tail drift")
    need(not any("ld-linux" in token for token in command), "loader route became reachable")
    need(SCIENTIFIC_TEST.endswith("m3_end_to_end_physical_proof"), "scientific test drift")
    return {
        "schema": "lay.m3-v8r2-remote-self-check.v1",
        "verdict": "M3_V8R2_REMOTE_CONTROLLER_STATIC_PASS",
        "task_id": TASK_ID,
        "transaction_id": TRANSACTION_ID,
        "controller_sha256": sha256_file(pathlib.Path(__file__)),
        "actions": list(ACTIONS),
        "routes": list(ROUTES),
        "markers": ["e2e.available"],
        "e2e_argv": command,
        "cargo_invocations": 0,
        "rustc_compilations": 0,
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
        elif args.action == "create-marker":
            need(args.audit is not None, "--audit is required")
            value = with_route_lock(lambda: create_marker(args.audit, controller_sha))
        else:
            need(args.quiet is not None, "--quiet is required")
            value = with_route_lock(lambda: e2e_once(args.quiet, controller_sha))
        print(json.dumps(value, sort_keys=True))
        return 0
    except BaseException as error:
        print(json.dumps({
            "schema": "lay.m3-v8r2-remote-error.v1",
            "task_id": TASK_ID,
            "transaction_id": TRANSACTION_ID,
            "verdict": "BLOCKED_PROVENANCE",
            "error": f"{type(error).__name__}: {error}",
            "retry_permitted": False,
        }, sort_keys=True))
        return 1


if __name__ == "__main__":
    sys.exit(main())
