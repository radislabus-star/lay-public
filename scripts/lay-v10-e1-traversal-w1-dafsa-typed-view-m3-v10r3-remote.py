#!/usr/bin/env python3
"""TRACE-only remote producer reusing the sealed V10R2 test ELF."""

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


TASK_ID = "slice8b-v10-e1-traversal-w1-dafsa-typed-view-m3-admission-substage-v10r3-20260827"
TRANSACTION_ID = "8fa454ad944ce8ba9e6256bc55bdf8ff8231cf26bac7eaab591ad8564515436c"
OLD_TASK_ID = "slice8b-v10-e1-traversal-w1-dafsa-typed-view-m3-admission-substage-v10r2-20260827"
OLD_TRANSACTION_ID = "fe5b4741c5d5711b48f356569f3be32a87142edd78979edf0aeb72a9616de7e6"
HOSTNAME = "e-MEGA-MINI-M1-13th"
MACHINE_ID_SHA256 = "5ac0bb537745673ef80942c2df32d586f5753c85de1c8b2453aa4e7317b77441"
KERNEL = "6.8.0-124-generic"

PARENT = pathlib.Path("/home/e/.local/share/lay/provenance") / TASK_ID
STATE = pathlib.Path("/home/e/.local/state/lay") / TASK_ID
BOOTSTRAP = PARENT / "bootstrap-reuse-v1"
TRACE = PARENT / "trace-v1"

OLD_PARENT = pathlib.Path("/home/e/.local/share/lay/provenance") / OLD_TASK_ID
OLD_STATE = pathlib.Path("/home/e/.local/state/lay") / OLD_TASK_ID
OLD_BOOTSTRAP = OLD_PARENT / "bootstrap-v1"
OLD_BUILD = OLD_PARENT / "build-v1"
OLD_ELF = OLD_BUILD / "v10-test-elf"

ACTIONS = ("self-check", "status", "bootstrap-reuse", "create-marker", "trace-once")
ROUTES = ("TRACE-REUSE",)
MARKER_NAMES = {"TRACE-REUSE": "trace"}
SCIENTIFIC_TEST = (
    "nanda_wave::l2_field::v13_typed_peak::tests::m3_v8::"
    "m3_end_to_end_physical_proof"
)

EXPECTED_ELF_SIZE = 320_986_144
EXPECTED_ELF_SHA256 = "0378514225ccec3cadbcfedd21ec77db66518a5eb6789f9acd83525ccf009696"
EXPECTED_ELF_BUILD_ID = "9e2e7c1fef9272f87c14876d7194609df6ac948d"
EXPECTED_INPUTS = {
    "LAY-L2-RU-FULL-v13.bin": (140_556_462, "cce259fe0ce5dce67702383363b66f0fe9b9ff5a87d8f01c4fcf342d91218d7b"),
    "slice8b-v7-fixed-13x100.json": (1_606_189, "33fded73e13f565bdc08a83bd440be67c80f6d85a196453050b09bc9f7ef28e4"),
    "LAY-L2-PRODUCTIVE-PARADIGM-v90.p2m": (17_309_944, "40fb6a9f0d92c3c7502e47f9c70230d9b86020f622b08a5c799342f13e09ce44"),
    "LAY-L2-PRODUCTIVE-PARADIGM-v90.p2r": (2_123_112, "de7972c80448dc792759d70de99cda6ec48c3d6af337763856601db563ab167e"),
    "LAY-L1.1-RU-COMPOSITE-EN300K-PHASE8I-v9.v9.bin": (77_962_328, "bf5a1619a89038466ef786305cf35eda5f4af5b9f12b9140f7d3cac407e2f2a7"),
    "l11-proof.json": (539_536, "4983930495e793c1d28c7558fe006ddf8097ee575bebb1afd3f1dba4ddb1d01d"),
}


class V10R3RemoteError(RuntimeError):
    pass


def need(condition: bool, message: str) -> None:
    if not condition:
        raise V10R3RemoteError(message)


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
    need(path.is_file(), f"required file absent: {path}")
    row = file_row(path)
    if size is not None:
        need(row["size_bytes"] == size, f"file size drift: {path}")
    if digest is not None:
        need(row["sha256"] == digest, f"file SHA drift: {path}")
    if mode is not None:
        need(row["mode"] == mode, f"file mode drift: {path}")
    return row


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
    check: bool = True,
    timeout: float = 120,
    environment: Mapping[str, str] | None = None,
) -> subprocess.CompletedProcess[bytes]:
    result = subprocess.run(
        list(argv),
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        timeout=timeout,
        env=dict(environment) if environment is not None else None,
        check=False,
    )
    if check and result.returncode != 0:
        raise V10R3RemoteError(
            f"command failed rc={result.returncode}: {list(argv)!r}; "
            f"stdout={result.stdout[-4096:]!r}; stderr={result.stderr[-4096:]!r}"
        )
    return result


def controlled_environment() -> dict[str, str]:
    return {
        "HOME": "/home/e",
        "LANG": "C",
        "LC_ALL": "C",
        "PATH": "/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin",
        "RUST_BACKTRACE": "0",
        "TZ": "UTC",
    }


def write_manifest(root: pathlib.Path) -> None:
    rows = []
    for path in sorted(root.rglob("*")):
        if path.is_file() and path.name != "SHA256SUMS":
            rows.append(f"{sha256_file(path)}  {path.relative_to(root)}\n")
    write_new(root / "SHA256SUMS", "".join(rows).encode(), 0o444)


def seal_tree(root: pathlib.Path) -> None:
    for path in sorted(root.rglob("*"), reverse=True):
        path.chmod(0o444 if path.is_file() else 0o555)
    root.chmod(0o555)


def elf_build_id(path: pathlib.Path) -> str:
    output = run(["/usr/bin/readelf", "-n", str(path)]).stdout.decode(errors="replace")
    match = re.search(r"Build ID:\s*([0-9a-f]+)", output)
    need(match is not None, "ELF Build ID absent")
    return match.group(1)


def verify_host() -> dict[str, Any]:
    need(os.uname().nodename == HOSTNAME, "hostname drift")
    need(os.uname().release == KERNEL, "kernel drift")
    machine = sha256_file(pathlib.Path("/etc/machine-id"))
    need(machine == MACHINE_ID_SHA256, "machine identity drift")
    online = pathlib.Path("/sys/devices/system/cpu/online").read_text().strip()
    core = pathlib.Path("/sys/bus/event_source/devices/cpu_core/cpus").read_text().strip()
    atom = pathlib.Path("/sys/bus/event_source/devices/cpu_atom/cpus").read_text().strip()
    need((online, core, atom) == ("0-19", "0-11", "12-19"), "CPU topology drift")
    return {
        "hostname": HOSTNAME,
        "kernel": KERNEL,
        "machine_id_sha256": machine,
        "online": online,
        "core": core,
        "atom": atom,
    }


def input_paths() -> dict[str, pathlib.Path]:
    inputs = OLD_BOOTSTRAP / "inputs"
    return {
        "v13": inputs / "LAY-L2-RU-FULL-v13.bin",
        "v7": inputs / "slice8b-v7-fixed-13x100.json",
        "productive": inputs / "LAY-L2-PRODUCTIVE-PARADIGM-v90.p2m",
        "recovery": inputs / "LAY-L2-PRODUCTIVE-PARADIGM-v90.p2r",
        "l11": inputs / "LAY-L1.1-RU-COMPOSITE-EN300K-PHASE8I-v9.v9.bin",
        "l11_proof": inputs / "l11-proof.json",
        "l11_receipt": inputs / "l11-installed.json",
    }


def verify_inputs() -> dict[str, Any]:
    rows = {}
    for name, (size, digest) in EXPECTED_INPUTS.items():
        rows[name] = require_file(OLD_BOOTSTRAP / "inputs" / name, size=size, digest=digest, mode="0444")
    receipt = require_file(input_paths()["l11_receipt"], mode="0444")
    rows["l11-installed.json"] = receipt
    return rows


def marker_inventory(root: pathlib.Path = STATE) -> dict[str, list[str]]:
    markers = root / "markers"
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


def active_conflicts() -> list[dict[str, Any]]:
    conflicts = []
    own = {os.getpid(), os.getppid()}
    tokens = ("perf " + "record", "perf " + "stat", SCIENTIFIC_TEST, str(OLD_ELF))
    for path in pathlib.Path("/proc").iterdir():
        if not path.name.isdigit() or int(path.name) in own:
            continue
        try:
            command = (path / "cmdline").read_bytes().replace(b"\0", b" ").decode(errors="replace").strip()
        except (FileNotFoundError, PermissionError, ProcessLookupError):
            continue
        if command and any(token in command for token in tokens):
            conflicts.append({"pid": int(path.name), "command": command})
    return sorted(conflicts, key=lambda row: row["pid"])


def old_projection() -> dict[str, Any]:
    need(OLD_PARENT.is_dir() and OLD_STATE.is_dir(), "V10R2 namespace absent")
    need(OLD_BUILD.is_dir() and not (OLD_PARENT / "trace-v1").exists(), "V10R2 build/TRACE boundary drift")
    elf = require_file(OLD_ELF, size=EXPECTED_ELF_SIZE, digest=EXPECTED_ELF_SHA256, mode="0555")
    build_id = elf_build_id(OLD_ELF)
    need(build_id == EXPECTED_ELF_BUILD_ID, "V10R2 ELF Build ID drift")
    markers = marker_inventory(OLD_STATE)
    need(markers == {"available": ["trace.available"], "consumed": ["build.consumed-before-exec"]}, "V10R2 marker projection drift")
    marker_rows = {
        name: file_row(OLD_STATE / "markers" / name)
        for name in markers["available"] + markers["consumed"]
    }
    state = latest_state(OLD_STATE)
    need(state is not None and state.get("state") == "BUILD_CREATED_UNAUDITED", "V10R2 state drift")
    return {
        "task_id": OLD_TASK_ID,
        "transaction_id": OLD_TRANSACTION_ID,
        "elf": elf,
        "elf_build_id": build_id,
        "markers": markers,
        "marker_rows": marker_rows,
        "latest_state": state,
        "inputs": verify_inputs(),
        "trace_exists": False,
    }


def verify_execution_admission(path: pathlib.Path, controller_sha: str) -> dict[str, Any]:
    value = load_json(path)
    need(value.get("verdict") == "V10R3_EXECUTION_ADMITTED", "execution admission verdict drift")
    need(value.get("safe_to_execute") is True, "execution admission is not safe")
    need(value.get("task_id") == TASK_ID and value.get("transaction_id") == TRANSACTION_ID, "execution admission namespace drift")
    need(value.get("remote_controller_sha256") == controller_sha, "execution admission controller SHA drift")
    return value


def verify_upload(path: pathlib.Path, payload: Mapping[str, Any]) -> dict[str, Any]:
    expected = payload.get("files")
    need(isinstance(expected, dict), "bootstrap inventory absent")
    rows = {}
    for name, contract in expected.items():
        need(isinstance(name, str) and isinstance(contract, dict), "bootstrap inventory malformed")
        rows[name] = require_file(path / name, size=int(contract["size_bytes"]), digest=str(contract["sha256"]))
    need(set(rows) == {"EXECUTION_ADMISSION.json", "remote-controller.py"}, "bootstrap upload set drift")
    return rows


def uid_capability_probe(stage_parent: pathlib.Path) -> dict[str, Any]:
    probe = stage_parent / "uid-capability"
    probe.mkdir(mode=0o700)
    shutil.chown(probe, user="e", group="e")
    code = (
        "import os,pathlib; p=pathlib.Path(os.environ['P']); "
        "assert pathlib.Path(os.environ['ELF']).open('rb').read(4)==b'\\x7fELF'; "
        "assert pathlib.Path(os.environ['INPUT']).open('rb').read(1); "
        "a=p/'a'; b=p/'b'; f=open(a,'xb'); f.write(b'v10r3-uid-proof\\n'); "
        "f.flush(); os.fsync(f.fileno()); f.close(); os.rename(a,b); "
        "assert b.read_bytes()==b'v10r3-uid-proof\\n'; b.unlink()"
    )
    result = run([
        "/usr/bin/sudo", "-n", "-u", "e", "/usr/bin/env",
        f"P={probe}", f"ELF={OLD_ELF}", f"INPUT={input_paths()['v13']}",
        "/usr/bin/python3", "-c", code,
    ], check=False)
    need(result.returncode == 0 and not any(probe.iterdir()), "UID e capability proof failed")
    probe.rmdir()
    return {
        "uid": 1000,
        "operations": ["traverse", "read-old-elf", "read-old-input", "create", "write", "fsync", "rename", "read", "unlink"],
        "verdict": "PASS",
    }


def append_state(sequence: int, state: str, **extra: Any) -> None:
    write_json(STATE / f"STATE-{sequence:02d}-{state}.json", {
        "schema": "lay.v10r3-remote-state.v1",
        "task_id": TASK_ID,
        "transaction_id": TRANSACTION_ID,
        "sequence": sequence,
        "state": state,
        "markers": marker_inventory(),
        **extra,
    }, 0o444)
    fsync_dir(STATE)


def bootstrap_reuse(path: pathlib.Path) -> dict[str, Any]:
    need(os.geteuid() == 0, "bootstrap requires root")
    need(not PARENT.exists() and not STATE.exists(), "V10R3 namespace already exists")
    payload = load_json(path / "PAYLOAD.json")
    need(payload.get("schema") == "lay.v10r3-bootstrap-reuse-payload.v1", "bootstrap payload schema drift")
    need(payload.get("task_id") == TASK_ID and payload.get("transaction_id") == TRANSACTION_ID, "bootstrap payload namespace drift")
    controller_sha = sha256_file(path / "remote-controller.py")
    admission = verify_execution_admission(path / "EXECUTION_ADMISSION.json", controller_sha)
    uploaded = verify_upload(path, payload)
    host = verify_host()
    old = old_projection()
    need(old == admission.get("old_v10r2_projection"), "V10R2 live projection changed after admission")
    need(not active_conflicts(), "conflicting experiment active before bootstrap")

    stage = pathlib.Path(f"{PARENT}.stage-{os.getpid()}-{time.time_ns()}")
    state_stage = pathlib.Path(f"{STATE}.stage-{os.getpid()}-{time.time_ns()}")
    stage.mkdir(parents=True, mode=0o755)
    state_stage.mkdir(parents=True, mode=0o700)
    try:
        copied = stage / "bootstrap-reuse-v1"
        copied.mkdir(mode=0o755)
        shutil.copy2(path / "remote-controller.py", copied / "remote-controller.py")
        shutil.copy2(path / "EXECUTION_ADMISSION.json", copied / "EXECUTION_ADMISSION.json")
        (copied / "remote-controller.py").chmod(0o555)
        (copied / "EXECUTION_ADMISSION.json").chmod(0o444)
        uid = uid_capability_probe(stage)
        write_new(state_stage / "route.lock", b"v10r3-trace-reuse-route-lock\n", 0o600)
        receipt = {
            "schema": "lay.v10r3-bootstrap-reuse.v1",
            "task_id": TASK_ID,
            "transaction_id": TRANSACTION_ID,
            "verdict": "V10R3_BOOTSTRAP_REUSE_CREATED_UNAUDITED",
            "controller_sha256": controller_sha,
            "execution_admission_sha256": sha256_file(path / "EXECUTION_ADMISSION.json"),
            "host": host,
            "old_v10r2_projection": old,
            "uploaded": uploaded,
            "uid_capability": uid,
            "routes": list(ROUTES),
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
        write_json(stage / "BOOTSTRAP_REUSE_RECEIPT.json", receipt)
        write_manifest(stage)
        seal_tree(stage)
        os.rename(stage, PARENT)
        os.rename(state_stage, STATE)
        fsync_dir(PARENT.parent)
        fsync_dir(STATE.parent)
        append_state(0, "BOOTSTRAP_REUSE_CREATED_UNAUDITED")
        return receipt
    except BaseException:
        if stage.exists():
            shutil.rmtree(stage)
        if state_stage.exists():
            shutil.rmtree(state_stage)
        raise


def verify_bootstrap_audit(path: pathlib.Path, controller_sha: str) -> dict[str, Any]:
    value = load_json(path)
    need(value.get("verdict") == "V10R3_BOOTSTRAP_REUSE_AUDIT_PASS_QUIET_ADMITTED", "bootstrap audit did not admit quiet gate")
    need(value.get("task_id") == TASK_ID and value.get("transaction_id") == TRANSACTION_ID, "bootstrap audit namespace drift")
    need(value.get("remote_controller_sha256") == controller_sha, "bootstrap audit controller SHA drift")
    return value


def verify_quiet_admission(path: pathlib.Path, controller_sha: str) -> dict[str, Any]:
    value = load_json(path)
    need(value.get("verdict") == "V10R3_QUIET_READY_TRACE_ADMITTED", "quiet audit did not admit TRACE")
    need(value.get("task_id") == TASK_ID and value.get("transaction_id") == TRANSACTION_ID, "quiet audit namespace drift")
    need(value.get("remote_controller_sha256") == controller_sha, "quiet audit controller SHA drift")
    need(value.get("consecutive_passes") == 3, "quiet audit consecutive-pass drift")
    need(value.get("window_seconds") == 5 and value.get("attempts", 0) <= 120, "quiet audit bounded-window drift")
    need(value.get("cpu0_idle_threshold") == 0.95 and value.get("all_cpu_idle_threshold") == 0.90, "quiet threshold drift")
    need(value.get("old_elf_sha256") == EXPECTED_ELF_SHA256, "quiet audit ELF binding drift")
    need(value.get("conflict_windows") == 0 and value.get("thermal_drift_windows") == 0, "quiet observation failure")
    bootstrap_copy = path.parent / "BOOTSTRAP_AUDIT.json"
    need(bootstrap_copy.is_file(), "quiet upload lacks bootstrap audit")
    bootstrap = verify_bootstrap_audit(bootstrap_copy, controller_sha)
    need(value.get("bootstrap_audit_sha256") == sha256_file(bootstrap_copy), "quiet/bootstrap binding drift")
    for key in ("local_controller_sha256", "remote_controller_sha256", "auditor_sha256"):
        need(value.get(key) == bootstrap.get(key), f"quiet identity drift: {key}")
    value["receipt_sha256"] = sha256_file(path)
    return value


def marker_payload(authority: Mapping[str, Any]) -> bytes:
    return canonical({
        "schema": "lay.v10r3-one-shot-marker.v1",
        "task_id": TASK_ID,
        "transaction_id": TRANSACTION_ID,
        "route": "TRACE-REUSE",
        "local_controller_sha256": authority["local_controller_sha256"],
        "remote_controller_sha256": authority["remote_controller_sha256"],
        "auditor_sha256": authority["auditor_sha256"],
        "quiet_receipt_sha256": authority["receipt_sha256"],
        "old_elf_sha256": EXPECTED_ELF_SHA256,
        "one_shot": True,
        "retry_permitted": False,
    })


def create_marker(quiet_path: pathlib.Path, controller_sha: str) -> dict[str, Any]:
    need(os.geteuid() == 0, "marker creation requires root")
    quiet = verify_quiet_admission(quiet_path, controller_sha)
    need(old_projection() == quiet.get("old_v10r2_projection"), "V10R2 projection drift before marker")
    need(not active_conflicts(), "conflicting experiment active before marker")
    markers = STATE / "markers"
    need(not markers.exists(), "V10R3 marker tree already exists")
    stage = STATE / f"markers.stage-{os.getpid()}-{time.time_ns()}"
    stage.mkdir(mode=0o700)
    try:
        marker = stage / "trace.available"
        write_new(marker, marker_payload(quiet), 0o400)
        row = file_row(marker)
        os.rename(stage, markers)
        fsync_dir(STATE)
    except BaseException:
        if stage.exists():
            shutil.rmtree(stage)
        raise
    append_state(1, "QUIET_PASS_TRACE_MARKER_AVAILABLE", quiet_receipt_sha256=quiet["receipt_sha256"])
    return {
        "schema": "lay.v10r3-marker.v1",
        "task_id": TASK_ID,
        "transaction_id": TRANSACTION_ID,
        "verdict": "V10R3_TRACE_MARKER_AVAILABLE",
        "marker": row,
        "markers": marker_inventory(),
        "markers_created": 1,
        "markers_consumed": 0,
        "cargo_invocations": 0,
        "rustc_compilations": 0,
        "subject_executions": 0,
        "runtime_authority_changed": False,
    }


def consume_marker(authority: Mapping[str, Any]) -> dict[str, Any]:
    markers = STATE / "markers"
    available = markers / "trace.available"
    consumed = markers / "trace.consumed-before-exec"
    expected = marker_payload(authority)
    before = require_file(available, digest=sha256_bytes(expected), mode="0400")
    need(available.read_bytes() == expected and not consumed.exists(), "TRACE-REUSE marker state drift")
    os.rename(available, consumed)
    fsync_dir(markers)
    after = require_file(consumed, size=before["size_bytes"], digest=before["sha256"], mode="0400")
    return {"before": before, "after": after, "consumed_before_execution": True}


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
        "LAY_L2_FIELD_TRACE": "1",
        "LAY_L2_PRODUCTIVE_V1_PACKAGE": str(paths["productive"]),
        "LAY_L11_RECEIPT": str(paths["l11_receipt"]),
        "LAY_PROPOSAL_ADMISSION_TRACE": "1",
    })
    return environment


def trace_once(quiet_path: pathlib.Path, controller_sha: str) -> dict[str, Any]:
    need(os.geteuid() == 0, "TRACE-REUSE requires root controller")
    quiet = verify_quiet_admission(quiet_path, controller_sha)
    state = latest_state()
    need(state is not None and state.get("state") == "QUIET_PASS_TRACE_MARKER_AVAILABLE", "TRACE predecessor drift")
    need(not TRACE.exists() and not (PARENT / "trace-failure-v1").exists(), "V10R3 TRACE evidence already exists")
    need(old_projection() == quiet.get("old_v10r2_projection"), "V10R2 projection drift before TRACE")
    need(not active_conflicts(), "conflicting experiment active before TRACE")

    stage = PARENT / f"trace-v1.stage-{os.getpid()}-{time.time_ns()}"
    subject = stage / "subject"
    final_subject = TRACE / "subject"
    evidence = subject / "evidence"
    stage.mkdir(mode=0o755)
    subject.mkdir(mode=0o700)
    evidence.mkdir(mode=0o700)
    shutil.chown(subject, user="e", group="e")
    shutil.chown(evidence, user="e", group="e")
    environment = scientific_environment(final_subject)
    command = [
        "/usr/bin/sudo", "-n", "-u", "e", "/usr/bin/env",
        *[f"{key}={value}" for key, value in sorted(environment.items())],
        "/usr/bin/taskset", "-c", "0", str(OLD_ELF),
        "--ignored", "--exact", SCIENTIFIC_TEST, "--nocapture", "--test-threads=1",
    ]
    write_json(stage / "PRE_TRACE.json", {
        "schema": "lay.v10r3-pre-trace.v1",
        "task_id": TASK_ID,
        "transaction_id": TRANSACTION_ID,
        "remote_controller_sha256": controller_sha,
        "quiet_admission_sha256": sha256_file(quiet_path),
        "elf": file_row(OLD_ELF),
        "elf_build_id": elf_build_id(OLD_ELF),
        "command": command,
        "environment": environment,
        "subject_started": False,
        "retry_permitted": False,
    })
    fsync_dir(stage)
    marker = consume_marker(quiet)
    write_json(stage / "TRACE_MARKER_CONSUMED.json", marker)
    os.rename(stage, TRACE)
    fsync_dir(PARENT)

    root = TRACE
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
        "schema": "lay.v10r3-trace-wrapper.v1",
        "task_id": TASK_ID,
        "transaction_id": TRANSACTION_ID,
        "verdict": "V10R3_TRACE_REUSE_CREATED_UNAUDITED" if producer_complete else "BLOCKED_PROVENANCE",
        "controller_error": controller_error,
        "marker": marker,
        "command": command,
        "environment": environment,
        "exit_code": exit_code,
        "subject_receipt": receipt,
        "outputs_complete": complete,
        "trace_lines": {
            "v9_aggregate": stderr.count(b"productive_v90_materialization_trace "),
            "v10_substage": stderr.count(b"proposal_admission_substage_trace "),
        },
        "reused_elf": file_row(OLD_ELF),
        "reused_elf_build_id": elf_build_id(OLD_ELF),
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
    write_json(root / "TRACE_WRAPPER.json", wrapper)
    write_manifest(root)
    seal_tree(root)
    append_state(2, "TRACE_REUSE_CREATED_UNAUDITED" if producer_complete else "BLOCKED_PROVENANCE", trace_wrapper_sha256=sha256_file(root / "TRACE_WRAPPER.json"))
    return wrapper


def status() -> dict[str, Any]:
    old = None
    old_error = None
    try:
        old = old_projection()
    except BaseException as error:
        old_error = f"{type(error).__name__}: {error}"
    return {
        "schema": "lay.v10r3-status.v1",
        "task_id": TASK_ID,
        "transaction_id": TRANSACTION_ID,
        "verdict": "V10R3_STATUS",
        "parent_exists": PARENT.exists(),
        "state_exists": STATE.exists(),
        "bootstrap_exists": BOOTSTRAP.exists(),
        "trace_exists": TRACE.exists(),
        "latest_state": latest_state(),
        "markers": marker_inventory(),
        "active_conflicts": active_conflicts(),
        "old_v10r2_projection": old,
        "old_v10r2_error": old_error,
    }


def self_check() -> dict[str, Any]:
    trace_tail = ("--ignored", "--exact", SCIENTIFIC_TEST, "--nocapture", "--test-threads=1")
    source = pathlib.Path(__file__).read_text()
    forbidden_commands = (
        "/usr/bin/" + "cargo",
        "cargo-guard" + ".sh",
        "/usr/bin/" + "rustc",
        "/usr/bin/" + "perf",
        "perf " + "record",
        "perf " + "stat",
    )
    need(ACTIONS == ("self-check", "status", "bootstrap-reuse", "create-marker", "trace-once"), "action registry drift")
    need(ROUTES == ("TRACE-REUSE",) and set(MARKER_NAMES) == set(ROUTES), "route registry drift")
    need(not any(token in source for token in forbidden_commands[:3]), "build command became reachable")
    need(not any(token in source for token in forbidden_commands[3:]), "perf route became reachable")
    need("--pid" not in trace_tail and ("SIG" + "INT") not in source, "attach or interrupt-signal lifecycle became reachable")
    need(SCIENTIFIC_TEST.endswith("m3_end_to_end_physical_proof"), "scientific test drift")
    need(("ld" + "-linux") not in source, "TRACE must execute the sealed test ELF directly")
    return {
        "schema": "lay.v10r3-remote-self-check.v1",
        "verdict": "V10R3_REMOTE_CONTROLLER_STATIC_PASS",
        "task_id": TASK_ID,
        "transaction_id": TRANSACTION_ID,
        "controller_sha256": sha256_file(pathlib.Path(__file__)),
        "actions": list(ACTIONS),
        "routes": list(ROUTES),
        "markers": ["trace.available"],
        "trace_argv_tail": list(trace_tail),
        "reused_elf_sha256": EXPECTED_ELF_SHA256,
        "reused_elf_build_id": EXPECTED_ELF_BUILD_ID,
        "direct_elf_execution": True,
        "build_reachable": False,
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
        elif args.action == "bootstrap-reuse":
            need(args.bootstrap is not None, "--bootstrap is required")
            value = bootstrap_reuse(args.bootstrap)
        elif args.action == "create-marker":
            need(args.quiet is not None, "--quiet is required")
            value = with_route_lock(lambda: create_marker(args.quiet, controller_sha))
        else:
            need(args.quiet is not None, "--quiet is required")
            value = with_route_lock(lambda: trace_once(args.quiet, controller_sha))
        print(json.dumps(value, sort_keys=True))
        return 0
    except BaseException as error:
        print(json.dumps({
            "schema": "lay.v10r3-remote-error.v1",
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
