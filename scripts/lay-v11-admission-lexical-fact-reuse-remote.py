#!/usr/bin/env python3
"""Remote one-build, two-subject producer for V11 lexical-fact reuse."""

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


TASK_ID = "slice8b-v11-admission-lexical-fact-reuse-paired-v3-20260827"
TRANSACTION_ID = "edc8266912d2d41ce112997f32dfe1f0748ae2699f777e70af55a54f55e0461e"
HOSTNAME = "e-MEGA-MINI-M1-13th"
MACHINE_ID_SHA256 = "5ac0bb537745673ef80942c2df32d586f5753c85de1c8b2453aa4e7317b77441"
KERNEL = "6.8.0-124-generic"

PARENT = pathlib.Path("/home/e/.local/share/lay/provenance") / TASK_ID
STATE = pathlib.Path("/home/e/.local/state/lay") / TASK_ID
BUILD = PARENT / "build-v1"
TERMINAL = PARENT / "terminal-v1"
CACHE = pathlib.Path("/home/e/.cache") / f"lay-v11-{TRANSACTION_ID}"
OLD_PARENT = pathlib.Path("/home/e/.local/share/lay/provenance/slice8b-v10-e1-traversal-w1-dafsa-typed-view-m3-e2e-v8r1-20260827")
OLD_BOOTSTRAP = OLD_PARENT / "bootstrap-v1"
OLD_SOURCE = OLD_BOOTSTRAP / "source-closure"
LOADER = pathlib.Path("/usr/lib/x86_64-linux-gnu/ld-linux-x86-64.so.2")

ACTIONS = ("self-check", "status", "bootstrap", "build-once", "run-once", "terminal")
ROUTES = ("BUILD", "B0", "B1")
MARKERS = {"BUILD": "build", "B0": "b0", "B1": "b1"}
MODES = {"B0": "UNCACHED", "B1": "REUSE"}
SCIENTIFIC_TEST = (
    "nanda_wave::l2_field::v13_typed_peak::tests::m3_v8::"
    "m3_end_to_end_physical_proof"
)

PREFLIGHT_MANIFEST_SHA256 = "78f1f5146acbf53e025922f805994534ee5ef6edbe5b19660d10d4eb926dd5e2"
PREFLIGHT_RECEIPT_SHA256 = "bcb47347eda5fc9f3667a4fb4cfbc795e108ad40b322432db6684206f5181eeb"
V11_SOURCE_SIZE = 119_643
V11_SOURCE_SHA256 = "e8a6a182753084659e00ccd5e20238d585d859437824609a987ea03ce6edca72"
OLD_SOURCE_MANIFEST_SHA256 = "35bd9a8fafe46250190dba29c6186bc450798f9a2810da99730da5c358a1b342"
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
BUILD_ENVIRONMENT = {
    "CARGO_BUILD_JOBS": "20",
    "CARGO_INCREMENTAL": "0",
    "CARGO_NET_OFFLINE": "true",
    "CARGO_PROFILE_RELEASE_DEBUG": "2",
    "CARGO_PROFILE_RELEASE_STRIP": "none",
    "RUSTFLAGS": "",
}


class ControllerError(RuntimeError):
    pass


def need(condition: bool, message: str) -> None:
    if not condition:
        raise ControllerError(message)


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


def run(
    argv: Sequence[str],
    *,
    cwd: pathlib.Path | None = None,
    env: Mapping[str, str] | None = None,
    timeout: float = 3600,
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
        raise ControllerError(
            f"command failed rc={result.returncode}: {list(argv)!r}; "
            f"stderr={detail.decode(errors='replace')!r}"
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


def verify_host() -> dict[str, Any]:
    need(os.uname().nodename == HOSTNAME, "hostname drift")
    need(os.uname().release == KERNEL, "kernel drift")
    machine = require_file(pathlib.Path("/etc/machine-id"))
    need(machine["sha256"] == MACHINE_ID_SHA256, "machine-id drift")
    online = pathlib.Path("/sys/devices/system/cpu/online").read_text().strip()
    core = pathlib.Path("/sys/bus/event_source/devices/cpu_core/cpus").read_text().strip()
    atom = pathlib.Path("/sys/bus/event_source/devices/cpu_atom/cpus").read_text().strip()
    need((online, core, atom) == ("0-19", "0-11", "12-19"), "CPU topology drift")
    cargo = run(["/home/e/.cargo/bin/cargo", "-V"], env=controlled_environment()).stdout.decode().strip()
    rustc = run(["/home/e/.cargo/bin/rustc", "-Vv"], env=controlled_environment()).stdout.decode()
    need(cargo == EXPECTED_CARGO, "Cargo drift")
    need(all(token in rustc for token in EXPECTED_RUSTC), "rustc drift")
    return {
        "hostname": HOSTNAME,
        "kernel": KERNEL,
        "machine_id_sha256": machine["sha256"],
        "cpu_topology": {"online": online, "core": core, "atom": atom},
        "toolchain": {"cargo": cargo, "rustc": rustc.strip()},
    }


def verify_old_source() -> dict[str, Any]:
    manifest_path = OLD_SOURCE / "SOURCE_MANIFEST.json"
    require_file(manifest_path, digest=OLD_SOURCE_MANIFEST_SHA256, mode="0444")
    manifest = load_json(manifest_path)
    need(manifest.get("schema") == "lay.m3-v8r1-source-closure.v1", "old source schema drift")
    rows = manifest.get("files")
    need(isinstance(rows, dict) and len(rows) == 770, "old source inventory drift")
    actual = {
        path.relative_to(OLD_SOURCE).as_posix()
        for path in OLD_SOURCE.rglob("*")
        if path.is_file() and path.name != "SOURCE_MANIFEST.json"
    }
    need(actual == set(rows), "old source file set drift")
    for relative, expected in sorted(rows.items()):
        require_file(
            OLD_SOURCE / relative,
            size=int(expected["size_bytes"]),
            digest=str(expected["sha256"]),
        )
    return {
        "manifest_sha256": sha256_file(manifest_path),
        "files": len(rows),
        "bytes": sum(int(row["size_bytes"]) for row in rows.values()),
    }


def input_paths() -> dict[str, pathlib.Path]:
    root = OLD_BOOTSTRAP / "inputs"
    return {
        "v13": root / "LAY-L2-RU-FULL-v13.bin",
        "v7": root / "slice8b-v7-fixed-13x100.json",
        "productive": root / "LAY-L2-PRODUCTIVE-PARADIGM-v90.p2m",
        "recovery": root / "LAY-L2-PRODUCTIVE-PARADIGM-v90.p2r",
        "l11": root / "LAY-L1.1-RU-COMPOSITE-EN300K-PHASE8I-v9.v9.bin",
        "l11_proof": root / "l11-proof.json",
        "l11_receipt": root / "l11-installed.json",
    }


def verify_inputs() -> dict[str, Any]:
    rows = {}
    for name, (size, digest) in EXPECTED_INPUTS.items():
        rows[name] = require_file(OLD_BOOTSTRAP / "inputs" / name, size=size, digest=digest)
    require_file(input_paths()["l11_receipt"])
    return rows


def ancestors() -> set[int]:
    result = {os.getpid()}
    current = os.getppid()
    while current > 1 and current not in result:
        result.add(current)
        try:
            fields = pathlib.Path(f"/proc/{current}/stat").read_text().split()
            current = int(fields[3])
        except (FileNotFoundError, PermissionError, ValueError, IndexError):
            break
    return result


def active_conflicts() -> list[dict[str, Any]]:
    own = ancestors()
    tokens = (SCIENTIFIC_TEST, "perf record", "perf stat", "scripts/cargo-guard.sh")
    rows = []
    for path in pathlib.Path("/proc").iterdir():
        if not path.name.isdigit() or int(path.name) in own:
            continue
        try:
            command = (path / "cmdline").read_bytes().replace(b"\0", b" ").decode(errors="replace").strip()
        except (FileNotFoundError, PermissionError, ProcessLookupError):
            continue
        if command and any(token in command for token in tokens):
            rows.append({"pid": int(path.name), "command": command})
    return sorted(rows, key=lambda row: row["pid"])


def load_admission(root: pathlib.Path) -> dict[str, Any]:
    value = load_json(root / "ADMISSION.json")
    need(value.get("schema") == "lay.v11-paired-execution-admission.v1", "admission schema drift")
    need(value.get("verdict") == "V11_PAIRED_EXECUTION_ADMITTED", "execution not admitted")
    need(value.get("task_id") == TASK_ID and value.get("transaction_id") == TRANSACTION_ID, "admission namespace drift")
    need(value.get("preflight_manifest_sha256") == PREFLIGHT_MANIFEST_SHA256, "preflight manifest drift")
    need(value.get("preflight_receipt_sha256") == PREFLIGHT_RECEIPT_SHA256, "preflight receipt drift")
    need(value.get("source_sha256") == V11_SOURCE_SHA256 and value.get("source_size_bytes") == V11_SOURCE_SIZE, "admitted source drift")
    need(value.get("remote_controller_sha256") == sha256_file(root / "remote-controller.py"), "controller SHA drift")
    require_file(root / "proposal_admission.rs", size=V11_SOURCE_SIZE, digest=V11_SOURCE_SHA256)
    return value


def marker_payload(route: str, admission: Mapping[str, Any]) -> bytes:
    need(route in ROUTES, f"unknown route: {route}")
    return canonical({
        "schema": "lay.v11-paired-marker.v1",
        "task_id": TASK_ID,
        "transaction_id": TRANSACTION_ID,
        "route": route,
        "source_sha256": V11_SOURCE_SHA256,
        "remote_controller_sha256": admission["remote_controller_sha256"],
        "one_shot": True,
        "retry_permitted": False,
    })


def marker_inventory() -> dict[str, list[str]]:
    root = STATE / "markers"
    if not root.is_dir():
        return {"available": [], "consumed": []}
    return {
        "available": sorted(path.name for path in root.glob("*.available")),
        "consumed": sorted(path.name for path in root.glob("*.consumed-before-exec")),
    }


def consume_marker(route: str, admission: Mapping[str, Any]) -> dict[str, Any]:
    root = STATE / "markers"
    stem = MARKERS[route]
    available = root / f"{stem}.available"
    consumed = root / f"{stem}.consumed-before-exec"
    expected = marker_payload(route, admission)
    before = require_file(available, digest=sha256_bytes(expected), mode="0400")
    need(available.read_bytes() == expected and not consumed.exists(), f"marker drift: {route}")
    os.rename(available, consumed)
    fsync_dir(root)
    after = require_file(consumed, size=before["size_bytes"], digest=before["sha256"], mode="0400")
    return {"before": before, "after": after, "consumed_before_execution": True}


def append_state(sequence: int, state: str, **extra: Any) -> None:
    write_json(
        STATE / f"STATE-{sequence:02d}-{state}.json",
        {
            "schema": "lay.v11-paired-state.v1",
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


def latest_state() -> dict[str, Any] | None:
    if not STATE.is_dir():
        return None
    rows = sorted(STATE.glob("STATE-*.json"))
    return load_json(rows[-1]) if rows else None


@contextlib.contextmanager
def route_lock() -> Any:
    lock = STATE / "route.lock"
    descriptor = os.open(lock, os.O_RDWR)
    try:
        fcntl.flock(descriptor, fcntl.LOCK_EX | fcntl.LOCK_NB)
        yield
    finally:
        os.close(descriptor)


def uid_probe(parent: pathlib.Path) -> dict[str, Any]:
    probe = parent / "uid-capability"
    probe.mkdir(mode=0o700)
    shutil.chown(probe, user="e", group="e")
    code = (
        "import os,pathlib; p=pathlib.Path(os.environ['P']); a=p/'a'; b=p/'b'; "
        "f=open(a,'xb'); f.write(b'v11-uid-proof\\n'); f.flush(); os.fsync(f.fileno()); f.close(); "
        "os.rename(a,b); assert b.read_bytes()==b'v11-uid-proof\\n'; b.unlink()"
    )
    result = run(
        ["/usr/bin/sudo", "-n", "-u", "e", "/usr/bin/env", f"P={probe}", "/usr/bin/python3", "-c", code],
        check=False,
    )
    need(result.returncode == 0 and not any(probe.iterdir()), "UID e capability proof failed")
    probe.rmdir()
    return {"uid": 1000, "operations": ["create", "write", "fsync", "rename", "read", "unlink"], "verdict": "PASS"}


def bootstrap(bundle: pathlib.Path) -> dict[str, Any]:
    need(os.geteuid() == 0, "bootstrap requires root")
    need(not PARENT.exists() and not STATE.exists(), "V11 namespace already exists")
    need(not active_conflicts(), "conflicting performance process active")
    admission = load_admission(bundle)
    host = verify_host()
    old_source = verify_old_source()
    inputs = verify_inputs()
    stage = pathlib.Path(f"{PARENT}.stage-{os.getpid()}-{time.time_ns()}")
    state_stage = pathlib.Path(f"{STATE}.stage-{os.getpid()}-{time.time_ns()}")
    stage.mkdir(parents=True, mode=0o755)
    state_stage.mkdir(parents=True, mode=0o700)
    try:
        shutil.copytree(bundle, stage / "bootstrap")
        uid = uid_probe(stage)
        write_new(state_stage / "route.lock", b"v11-paired-route-lock\n", 0o600)
        markers = state_stage / "markers"
        markers.mkdir(mode=0o700)
        marker_rows = {}
        for route in ROUTES:
            path = markers / f"{MARKERS[route]}.available"
            write_new(path, marker_payload(route, admission), 0o400)
            marker_rows[route] = file_row(path)
        receipt = {
            "schema": "lay.v11-paired-bootstrap.v1",
            "task_id": TASK_ID,
            "transaction_id": TRANSACTION_ID,
            "verdict": "V11_ALL_MARKERS_AVAILABLE",
            "admission": admission,
            "host": host,
            "old_source": old_source,
            "v11_source": file_row(stage / "bootstrap/proposal_admission.rs"),
            "inputs": inputs,
            "uid_capability": uid,
            "marker_rows": marker_rows,
            "markers_created": 3,
            "markers_consumed": 0,
            "cargo_invocations": 0,
            "subject_executions": 0,
            "runtime_authority_changed": False,
        }
        write_json(stage / "BOOTSTRAP_RECEIPT.json", receipt)
        write_manifest(stage)
        seal_tree(stage)
        os.rename(stage, PARENT)
        os.rename(state_stage, STATE)
        fsync_dir(PARENT.parent)
        fsync_dir(STATE.parent)
        append_state(0, "ALL_MARKERS_AVAILABLE")
        return receipt
    except BaseException:
        if stage.exists():
            shutil.rmtree(stage)
        if state_stage.exists():
            shutil.rmtree(state_stage)
        raise


def find_test_elf(target: pathlib.Path) -> pathlib.Path:
    candidates = []
    for path in (target / "release/deps").glob("lay-*"):
        if path.is_file() and os.access(path, os.X_OK):
            with path.open("rb") as source:
                if source.read(4) == b"\x7fELF":
                    candidates.append(path)
    need(len(candidates) == 1, f"expected one test ELF, found {len(candidates)}")
    return candidates[0]


def elf_build_id(path: pathlib.Path) -> str:
    output = run(["/usr/bin/readelf", "-n", str(path)]).stdout
    match = re.search(rb"Build ID:\s*([0-9a-f]+)", output)
    need(match is not None, "ELF Build ID absent")
    return match.group(1).decode()


def build_once() -> dict[str, Any]:
    need(os.geteuid() == 0, "build requires root")
    state = latest_state()
    need(state is not None and state.get("state") == "ALL_MARKERS_AVAILABLE", "build predecessor drift")
    need(not BUILD.exists() and not (PARENT / "build-failure-v1").exists(), "build evidence already exists")
    need(not active_conflicts(), "conflicting performance process active")
    admission = load_admission(PARENT / "bootstrap")
    verify_host()
    verify_old_source()
    verify_inputs()
    stage = PARENT / f"build-v1.stage-{os.getpid()}-{time.time_ns()}"
    workspace = CACHE / "workspace"
    need(not workspace.exists(), "fresh build workspace already exists")
    stage.mkdir(mode=0o700)
    workspace.parent.mkdir(parents=True, exist_ok=True, mode=0o700)
    marker = None
    cargo_started = False
    try:
        shutil.copytree(OLD_SOURCE, workspace)
        make_writable(workspace)
        source = workspace / "src/typing_transition/proposal_admission.rs"
        shutil.copyfile(PARENT / "bootstrap/proposal_admission.rs", source)
        source.chmod(0o600)
        require_file(source, size=V11_SOURCE_SIZE, digest=V11_SOURCE_SHA256)
        guard = workspace / "scripts/cargo-guard.sh"
        guard.chmod(0o775)
        environment = controlled_environment()
        environment.update(BUILD_ENVIRONMENT)
        environment["CARGO_TARGET_DIR"] = str(workspace / "target")
        command = [
            str(guard), "test", "--offline", "--locked", "--release", "--lib", "--no-run", "m3_v8"
        ]
        write_json(stage / "PREBUILD.json", {
            "schema": "lay.v11-paired-prebuild.v1",
            "task_id": TASK_ID,
            "transaction_id": TRANSACTION_ID,
            "source_sha256": V11_SOURCE_SHA256,
            "old_source_manifest_sha256": OLD_SOURCE_MANIFEST_SHA256,
            "command": command,
            "environment": {key: environment[key] for key in (*BUILD_ENVIRONMENT, "CARGO_TARGET_DIR")},
            "cargo_started": False,
            "retry_permitted": False,
        })
        fsync_dir(stage)
        marker = consume_marker("BUILD", admission)
        write_json(stage / "BUILD_MARKER_CONSUMED.json", marker)
        cargo_started = True
        with (stage / "cargo.log").open("wb") as log:
            result = run(command, cwd=workspace, env=environment, timeout=10_800, check=False, stdout=log, stderr=subprocess.STDOUT)
            log.flush()
            os.fsync(log.fileno())
        need(result.returncode == 0, f"Cargo exited {result.returncode}")
        elf = find_test_elf(workspace / "target")
        candidate = stage / "m3-v11-test-elf"
        shutil.copyfile(elf, candidate)
        candidate.chmod(0o555)
        receipt = {
            "schema": "lay.v11-paired-build.v1",
            "task_id": TASK_ID,
            "transaction_id": TRANSACTION_ID,
            "verdict": "V11_BUILD_CREATED",
            "source": file_row(source),
            "old_source_manifest_sha256": OLD_SOURCE_MANIFEST_SHA256,
            "marker": marker,
            "command": command,
            "environment": {key: environment[key] for key in (*BUILD_ENVIRONMENT, "CARGO_TARGET_DIR")},
            "cargo_exit_code": 0,
            "cargo_invocations": 1,
            "elf": {**file_row(candidate), "build_id": elf_build_id(candidate), "executed": False},
            "subject_executions": 0,
            "runtime_authority_changed": False,
            "retry_permitted": False,
        }
        write_json(stage / "BUILD_RECEIPT.json", receipt)
        write_manifest(stage)
        seal_tree(stage)
        os.rename(stage, BUILD)
        fsync_dir(PARENT)
        append_state(1, "BUILD_CREATED", build_receipt_sha256=sha256_file(BUILD / "BUILD_RECEIPT.json"))
        return receipt
    except BaseException as error:
        if marker is not None and stage.exists():
            failure = {
                "schema": "lay.v11-paired-build-failure.v1",
                "task_id": TASK_ID,
                "transaction_id": TRANSACTION_ID,
                "verdict": "BLOCKED_BUILD",
                "error": f"{type(error).__name__}: {error}",
                "marker": marker,
                "cargo_started": cargo_started,
                "cargo_invocations": int(cargo_started),
                "retry_permitted": False,
                "runtime_authority_changed": False,
            }
            write_json(stage / "BUILD_FAILURE.json", failure)
            write_manifest(stage)
            seal_tree(stage)
            target = PARENT / "build-failure-v1"
            os.rename(stage, target)
            fsync_dir(PARENT)
            append_state(1, "BLOCKED_BUILD", failure_sha256=sha256_file(target / "BUILD_FAILURE.json"))
            return failure
        raise
    finally:
        if workspace.exists():
            shutil.rmtree(workspace)


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


def semantic_ok(receipt: Mapping[str, Any]) -> bool:
    fixed = receipt.get("fixed_proof")
    if not isinstance(fixed, dict):
        return False
    semantic = fixed.get("semantic")
    return (
        isinstance(semantic, dict)
        and all(isinstance(value, int) and value == 0 for value in semantic.values())
        and fixed.get("cases") == 382
        and fixed.get("warmup_rounds") == 1
        and fixed.get("measured_rounds") == 4
        and fixed.get("measured_samples") == 1528
        and fixed.get("schedule") == ["FORWARD", "REVERSED", "FORWARD", "REVERSED"]
        and receipt.get("gates", {}).get("semantic") is True
        and receipt.get("runtime_authority_changed") is False
        and receipt.get("installed_package_changed") is False
        and receipt.get("network_used_by_subject") is False
        and receipt.get("perf_or_pmu_used") is False
    )


def subject_environment(route: str, subject: pathlib.Path) -> dict[str, str]:
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
        "LAY_PROPOSAL_ADMISSION_FACT_REUSE": MODES[route],
    })
    return environment


def route_root(route: str) -> pathlib.Path:
    return PARENT / f"{route.lower()}-v1"


def run_once(route: str) -> dict[str, Any]:
    need(os.geteuid() == 0, "subject route requires root")
    need(route in MODES, "route must be B0 or B1")
    state = latest_state()
    expected_state = "BUILD_CREATED" if route == "B0" else "B0_CREATED"
    need(state is not None and state.get("state") == expected_state, f"{route} predecessor drift")
    if route == "B1":
        prior = load_json(route_root("B0") / "SUBJECT_WRAPPER.json")
        need(prior.get("verdict") == "V11_B0_CREATED" and semantic_ok(prior.get("subject_receipt", {})), "B0 did not admit B1")
    root = route_root(route)
    need(not root.exists() and not (PARENT / f"{route.lower()}-failure-v1").exists(), f"{route} evidence already exists")
    need(not active_conflicts(), "conflicting performance process active")
    admission = load_admission(PARENT / "bootstrap")
    host = verify_host()
    verify_inputs()
    build = load_json(BUILD / "BUILD_RECEIPT.json")
    need(build.get("verdict") == "V11_BUILD_CREATED", "build receipt drift")
    elf = BUILD / "m3-v11-test-elf"
    elf_row = require_file(elf, digest=build["elf"]["sha256"], mode="0444")
    stage = PARENT / f"{route.lower()}-v1.stage-{os.getpid()}-{time.time_ns()}"
    subject = stage / "subject"
    final_subject = root / "subject"
    evidence = subject / "evidence"
    stage.mkdir(mode=0o755)
    subject.mkdir(mode=0o700)
    evidence.mkdir(mode=0o700)
    shutil.chown(subject, user="e", group="e")
    shutil.chown(evidence, user="e", group="e")
    environment = subject_environment(route, final_subject)
    command = [
        "/usr/bin/sudo", "-n", "-u", "e", "/usr/bin/env",
        *[f"{key}={value}" for key, value in sorted(environment.items())],
        "/usr/bin/taskset", "-c", "0", str(LOADER), str(elf),
        "--ignored", "--exact", SCIENTIFIC_TEST, "--nocapture", "--test-threads=1",
    ]
    write_json(stage / "PRE_SUBJECT.json", {
        "schema": "lay.v11-paired-pre-subject.v1",
        "task_id": TASK_ID,
        "transaction_id": TRANSACTION_ID,
        "route": route,
        "mode": MODES[route],
        "elf": elf_row,
        "command": command,
        "environment": environment,
        "subject_started": False,
        "retry_permitted": False,
    })
    fsync_dir(stage)
    marker = consume_marker(route, admission)
    write_json(stage / "MARKER_CONSUMED.json", marker)
    os.rename(stage, root)
    fsync_dir(PARENT)
    thermal_before = throttle_counters()
    process = None
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
    write_new(root / "stdout.log", stdout)
    write_new(root / "stderr.log", stderr)
    receipt_path = final_subject / "SUBJECT_RECEIPT.json"
    receipt = None
    if receipt_path.is_file():
        try:
            receipt = load_json(receipt_path)
        except BaseException as error:
            controller_error = controller_error or f"{type(error).__name__}: {error}"
    complete = isinstance(receipt, dict) and receipt.get("schema") == "lay.m3-end-to-end-test-owner.v1"
    semantic = complete and semantic_ok(receipt)
    exit_code = process.returncode if process is not None else None
    exit_consistent = complete and ((receipt.get("verdict") == "M3_END_TO_END_TEST_OWNER_PASS" and exit_code == 0) or (receipt.get("verdict") != "M3_END_TO_END_TEST_OWNER_PASS" and exit_code not in (None, 0)))
    if controller_error is not None or not complete or not exit_consistent:
        verdict = "BLOCKED_PROVENANCE"
        state_name = "BLOCKED_PROVENANCE"
    elif not semantic:
        verdict = "BLOCKED_SEMANTIC"
        state_name = "BLOCKED_SEMANTIC"
    else:
        verdict = f"V11_{route}_CREATED"
        state_name = f"{route}_CREATED"
    wrapper = {
        "schema": "lay.v11-paired-subject-wrapper.v1",
        "task_id": TASK_ID,
        "transaction_id": TRANSACTION_ID,
        "route": route,
        "mode": MODES[route],
        "verdict": verdict,
        "controller_error": controller_error,
        "marker": marker,
        "host": host,
        "elf": {**elf_row, "build_id": build["elf"]["build_id"]},
        "command": command,
        "environment": environment,
        "exit_code": exit_code,
        "subject_receipt": receipt,
        "outputs_complete": complete,
        "semantic_exact": semantic,
        "thermal_throttle_drift": throttle_drift(thermal_before, thermal_after),
        "subject_executions": int(process is not None),
        "cargo_invocations": 0,
        "perf_record_invocations": 0,
        "perf_stat_invocations": 0,
        "runtime_authority_changed": False,
        "retry_permitted": False,
    }
    write_json(root / "SUBJECT_WRAPPER.json", wrapper)
    write_manifest(root)
    seal_tree(root)
    append_state(2 if route == "B0" else 3, state_name, wrapper_sha256=sha256_file(root / "SUBJECT_WRAPPER.json"))
    return wrapper


def stable_subject(receipt: Mapping[str, Any]) -> dict[str, Any]:
    fixed = receipt["fixed_proof"]
    source = receipt["source"]
    initial = receipt["generation_owner"]["initial_generation"]
    return {
        "fixed": {
            key: fixed[key]
            for key in ("cases", "cpu", "cpu_mismatches", "empty_lane_mismatches", "maximum_query_scratch_bytes", "measured_rounds", "measured_samples", "schedule", "semantic", "warmup_cpu_mismatches", "warmup_rounds")
        },
        "source": {
            key: source[key]
            for key in ("l11_sha256", "productive_v90_sha256", "test_elf_sha256", "v13_bytes", "v13_sha256", "v7_bytes", "v7_sha256")
        },
        "generation": initial,
        "reload": {
            key: receipt["reload"][key]
            for key in ("current_b_commits", "failed_build_publications", "held_a_survived_publication", "mixed_generation_observations", "per_request_typed_materializations", "published_generations", "reader_count", "reader_identity_mismatches", "rollback_identity_mismatches", "stale_a_cancellations", "stale_a_commits", "typed_materializations")
        },
        "flags": {
            key: receipt[key]
            for key in ("installed_package_changed", "network_used_by_subject", "perf_or_pmu_used", "production_activation_admitted", "runtime_authority_changed")
        },
    }


def delta_pct(before: int | float, after: int | float) -> float | None:
    return None if before == 0 else (float(after) - float(before)) / float(before) * 100.0


def terminal() -> dict[str, Any]:
    need(os.geteuid() == 0, "terminal requires root")
    state = latest_state()
    need(state is not None and state.get("state") == "B1_CREATED", "terminal predecessor drift")
    need(not TERMINAL.exists(), "terminal already exists")
    b0 = load_json(route_root("B0") / "SUBJECT_WRAPPER.json")
    b1 = load_json(route_root("B1") / "SUBJECT_WRAPPER.json")
    failures = []
    if b0.get("verdict") != "V11_B0_CREATED" or b1.get("verdict") != "V11_B1_CREATED":
        failures.append("paired wrapper verdict drift")
    if b0.get("mode") != "UNCACHED" or b1.get("mode") != "REUSE":
        failures.append("paired mode drift")
    if b0.get("elf") != b1.get("elf"):
        failures.append("B0/B1 ELF identity mismatch")
    r0 = b0.get("subject_receipt", {})
    r1 = b1.get("subject_receipt", {})
    if not semantic_ok(r0) or not semantic_ok(r1):
        failures.append("semantic proof incomplete")
    if not failures and stable_subject(r0) != stable_subject(r1):
        failures.append("paired scientific envelope mismatch")
    f0 = r0.get("fixed_proof", {})
    f1 = r1.get("fixed_proof", {})
    p0 = f0.get("pooled", {})
    p1 = f1.get("pooled", {})
    metrics = {
        "maximum_round_search_p99_us": [f0.get("maximum_round_search_p99_us"), f1.get("maximum_round_search_p99_us")],
        "maximum_round_total_material_p99_us": [f0.get("maximum_round_total_material_p99_us"), f1.get("maximum_round_total_material_p99_us")],
        "pooled_search_p99_us": [p0.get("search", {}).get("p99_us"), p1.get("search", {}).get("p99_us")],
        "pooled_final_materialize_p99_us": [p0.get("final_materialize", {}).get("p99_us"), p1.get("final_materialize", {}).get("p99_us")],
        "pooled_total_material_p99_us": [p0.get("total_material", {}).get("p99_us"), p1.get("total_material", {}).get("p99_us")],
        "pooled_final_materialize_sum_us": [p0.get("final_materialize", {}).get("sum_us"), p1.get("final_materialize", {}).get("sum_us")],
    }
    deltas = {
        key: delta_pct(pair[0], pair[1]) if all(isinstance(value, (int, float)) for value in pair) else None
        for key, pair in metrics.items()
    }
    key_deltas = [deltas["maximum_round_total_material_p99_us"], deltas["pooled_total_material_p99_us"]]
    if all(value is not None and value < 0 for value in key_deltas):
        assessment = "OBSERVED_IMPROVEMENT"
    elif all(value is not None and value > 0 for value in key_deltas):
        assessment = "NO_OBSERVED_IMPROVEMENT"
    else:
        assessment = "MIXED_PERFORMANCE"
    verdict = "V11_PAIRED_COMPARISON_COMPLETE" if not failures else "BLOCKED_SEMANTIC"
    receipt = {
        "schema": "lay.v11-paired-terminal.v1",
        "task_id": TASK_ID,
        "transaction_id": TRANSACTION_ID,
        "verdict": verdict,
        "failures": failures,
        "source_sha256": V11_SOURCE_SHA256,
        "elf": b0.get("elf"),
        "modes": {"B0": "UNCACHED", "B1": "REUSE"},
        "semantic_exact": not failures,
        "metrics_b0_b1": metrics,
        "b1_delta_percent": deltas,
        "mechanism_assessment": assessment,
        "legacy_absolute_gates": {
            "B0": {
                "search_p99_le_3000": isinstance(metrics["maximum_round_search_p99_us"][0], int) and metrics["maximum_round_search_p99_us"][0] <= 3000,
                "total_material_p99_le_5000": isinstance(metrics["maximum_round_total_material_p99_us"][0], int) and metrics["maximum_round_total_material_p99_us"][0] <= 5000,
            },
            "B1": {
                "search_p99_le_3000": isinstance(metrics["maximum_round_search_p99_us"][1], int) and metrics["maximum_round_search_p99_us"][1] <= 3000,
                "total_material_p99_le_5000": isinstance(metrics["maximum_round_total_material_p99_us"][1], int) and metrics["maximum_round_total_material_p99_us"][1] <= 5000,
            },
            "terminal_if_small_miss": False,
        },
        "markers": marker_inventory(),
        "builds": 1,
        "subject_executions": 2,
        "perf_record_invocations": 0,
        "perf_stat_invocations": 0,
        "pmu_events": 0,
        "runtime_authority_changed": False,
        "production_authority_admitted": False,
        "next_action": "interpret measured gain and either select bounded reuse or close this microoptimization",
    }
    stage = PARENT / f"terminal-v1.stage-{os.getpid()}-{time.time_ns()}"
    stage.mkdir(mode=0o700)
    write_json(stage / "TERMINAL.json", receipt)
    write_manifest(stage)
    seal_tree(stage)
    os.rename(stage, TERMINAL)
    fsync_dir(PARENT)
    append_state(4, verdict, terminal_sha256=sha256_file(TERMINAL / "TERMINAL.json"))
    return receipt


def status() -> dict[str, Any]:
    return {
        "schema": "lay.v11-paired-status.v1",
        "task_id": TASK_ID,
        "transaction_id": TRANSACTION_ID,
        "verdict": "V11_PAIRED_STATUS",
        "parent_exists": PARENT.exists(),
        "state_exists": STATE.exists(),
        "latest_state": latest_state(),
        "markers": marker_inventory(),
        "build_exists": BUILD.exists(),
        "b0_exists": route_root("B0").exists(),
        "b1_exists": route_root("B1").exists(),
        "terminal_exists": TERMINAL.exists(),
        "active_conflicts": active_conflicts(),
    }


def self_check() -> dict[str, Any]:
    need(ACTIONS == ("self-check", "status", "bootstrap", "build-once", "run-once", "terminal"), "action registry drift")
    need(ROUTES == ("BUILD", "B0", "B1") and set(MARKERS) == set(ROUTES), "route registry drift")
    need(MODES == {"B0": "UNCACHED", "B1": "REUSE"}, "mode registry drift")
    need(SCIENTIFIC_TEST.endswith("m3_end_to_end_physical_proof"), "subject route drift")
    need(PREFLIGHT_RECEIPT_SHA256 and V11_SOURCE_SHA256, "pinned identity absent")
    return {
        "schema": "lay.v11-paired-remote-self-check.v1",
        "verdict": "V11_REMOTE_CONTROLLER_STATIC_PASS",
        "task_id": TASK_ID,
        "transaction_id": TRANSACTION_ID,
        "actions": list(ACTIONS),
        "routes": list(ROUTES),
        "modes": MODES,
        "builds": 1,
        "subjects": 2,
        "same_elf_required": True,
        "perf_reachable": False,
        "runtime_install_reachable": False,
        "small_performance_miss_terminal": False,
    }


def parser() -> argparse.ArgumentParser:
    value = argparse.ArgumentParser()
    value.add_argument("action", choices=ACTIONS)
    value.add_argument("--bundle", type=pathlib.Path)
    value.add_argument("--route", choices=("B0", "B1"))
    return value


def main() -> int:
    args = parser().parse_args()
    try:
        if args.action == "self-check":
            value = self_check()
        elif args.action == "status":
            value = status()
        elif args.action == "bootstrap":
            need(args.bundle is not None, "--bundle is required")
            value = bootstrap(args.bundle)
        elif args.action == "build-once":
            with route_lock():
                value = build_once()
        elif args.action == "run-once":
            need(args.route is not None, "--route is required")
            with route_lock():
                value = run_once(args.route)
        else:
            with route_lock():
                value = terminal()
        print(json.dumps(value, ensure_ascii=False, sort_keys=True))
        return 0
    except BaseException as error:
        print(json.dumps({
            "schema": "lay.v11-paired-remote-error.v1",
            "verdict": "ERROR",
            "task_id": TASK_ID,
            "transaction_id": TRANSACTION_ID,
            "error": f"{type(error).__name__}: {error}",
        }, ensure_ascii=False, sort_keys=True))
        return 1


if __name__ == "__main__":
    sys.exit(main())
