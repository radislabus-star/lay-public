#!/usr/bin/env python3
"""Remote one-shot producer for the W1 fused-minimum M2 experiment."""

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


TASK_ID = "slice8b-v10-e1-traversal-w1-fused-minimum-m2-v1-20260826"
TRANSACTION_ID = "c760eea52b6416b3529f9d684c315147b5a1140522114642c417d7db4065102c"
REMOTE_HOSTNAME = "e-MEGA-MINI-M1-13th"
REMOTE_MACHINE_ID_SHA256 = "5ac0bb537745673ef80942c2df32d586f5753c85de1c8b2453aa4e7317b77441"
PARENT = pathlib.Path("/home/e/.local/share/lay/provenance") / TASK_ID
STATE = pathlib.Path("/home/e/.local/state/lay") / TASK_ID
BUILD = PARENT / "build-v1"
LOADER = pathlib.Path("/usr/lib/x86_64-linux-gnu/ld-linux-x86-64.so.2")
B0A = pathlib.Path(
    "/home/e/.local/share/lay/provenance/"
    "slice8b-v10-hardware-b0-b2-v3-20260824/b0a-input-closure-v2"
)
B0B = pathlib.Path(
    "/home/e/.local/share/lay/provenance/"
    "slice8b-v10-hardware-b0-b2-v3-20260824/b0b-schedule-closure-v1"
)
ROUTE_ORDER = (
    "B0-ITERATOR",
    "G0-M1-GUARDED",
    "I0-INTERLEAVED",
    "I1-INTERLEAVED",
    "G1-M1-GUARDED",
    "B1-ITERATOR",
)
MARKER_ROUTES = ("BUILD", "PARITY", *ROUTE_ORDER)
PARITY_TEST = (
    "nanda_wave::l2_field::v13_typed_peak::tests::"
    "v10_m2_fused_minimum_parity"
)
PHYSICAL_TEST = (
    "nanda_wave::l2_field::v13_typed_peak::tests::"
    "v10_m2_fused_minimum_physical"
)
EVENTS = ("instructions", "cycles", "branches", "branch-misses", "task-clock")
EXPECTED = {
    "v10": "f6178f3882c1a216a9c2aaf7ff59dd6bdd78798708516b753ea7208177d9718c",
    "fragment": "b0a775420edf9e9d6e7f0b59f9ad840e4822cd0fdd0adc2429ea22a3e9e3a175",
    "assembled": "8654217a1509ef4ca9ef3c3dda5080a7c784fb767359c52531a772c0feae68dc",
    "production_prefix": "ce9ea2d290602774e2f45444e10e6fdabce6af4a61ae2bdca54c6d2e31a53f26",
    "package": "cce259fe0ce5dce67702383363b66f0fe9b9ff5a87d8f01c4fcf342d91218d7b",
    "sidecar": "a1aa95be8c43fae8fedf02d3261dd1265dcbfbc8d618330fcd6449ae829df8cd",
    "v7": "33fded73e13f565bdc08a83bd440be67c80f6d85a196453050b09bc9f7ef28e4",
    "schedule": "2f8346d97246ac3704434279739fa5ed27705465d628890a367c21875d879e78",
    "cargo_toml": "90a3ae6b16677ff49b70aab3457b794cd1e114a547f9e113a8b49595f976207b",
    "cargo_lock": "e6399be48c393e1557b451fcb04d886f9fbdaa9812109410ac3d9ecdc98b93f1",
    "cargo_guard": "a5b01f9a12b32ee2bcfc957a5e9f65efb52fe6e7644850ae3f00e412648243fe",
    "loader": "8d06f393f4a93bcf9b81145a259524d66a95522a646bf8d7e05b6ffdf2e63dcc",
}
EXPECTED_CARGO = "cargo 1.97.1 (c980f4866 2026-06-30)"
EXPECTED_RUSTC = (
    "release: 1.97.1",
    "commit-hash: 8bab26f4f68e0e26f0bb7960be334d5b520ea452",
    "host: x86_64-unknown-linux-gnu",
    "LLVM version: 22.1.6",
)
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


class M2RemoteError(RuntimeError):
    pass


def need(condition: bool, message: str) -> None:
    if not condition:
        raise M2RemoteError(message)


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


def canonical_json_bytes(value: Any) -> bytes:
    return (json.dumps(value, ensure_ascii=False, sort_keys=True, separators=(",", ":")) + "\n").encode()


def load_json(path: pathlib.Path) -> Any:
    with path.open("rb") as source:
        return json.load(source)


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


def require_file(
    path: pathlib.Path,
    *,
    digest: str | None = None,
    size: int | None = None,
    mode: str | None = None,
) -> dict[str, Any]:
    need(path.is_file() and not path.is_symlink(), f"required file absent: {path}")
    value = {
        "path": str(path),
        "sha256": sha256_file(path),
        "size_bytes": path.stat().st_size,
        "mode": mode_string(path),
    }
    if digest is not None:
        need(value["sha256"] == digest, f"SHA drift: {path}")
    if size is not None:
        need(value["size_bytes"] == size, f"size drift: {path}")
    if mode is not None:
        need(value["mode"] == mode, f"mode drift: {path}")
    return value


def run(
    command: Sequence[str],
    *,
    cwd: pathlib.Path | None = None,
    env: Mapping[str, str] | None = None,
    timeout: float = 3_600,
    check: bool = True,
    stdout: Any = subprocess.PIPE,
    stderr: Any = subprocess.PIPE,
) -> subprocess.CompletedProcess[bytes]:
    result = subprocess.run(
        list(command),
        cwd=cwd,
        env=dict(env) if env is not None else None,
        stdout=stdout,
        stderr=stderr,
        timeout=timeout,
        check=False,
    )
    if check and result.returncode != 0:
        detail = b"" if not isinstance(result.stderr, bytes) else result.stderr[-5000:]
        raise M2RemoteError(f"command failed ({result.returncode}): {command!r}\n{detail.decode(errors='replace')}")
    return result


def controlled_environment() -> dict[str, str]:
    return {
        "HOME": "/home/e",
        "LANG": "C.UTF-8",
        "LC_ALL": "C.UTF-8",
        "PATH": "/home/e/.cargo/bin:/home/e/.local/bin:/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin",
        "RUST_BACKTRACE": "0",
    }


def input_paths() -> dict[str, pathlib.Path]:
    artifacts = B0A / "inputs/slice8b-v10-f6178f/artifacts"
    return {
        "package": B0A / "inputs/LAY-L2-RU-FULL-v13.bin",
        "sidecar": artifacts / "LAY-L2-RU-FULL-v13.dafsa",
        "v7": artifacts / "slice8b-v7-fixed-13x100.json",
        "schedule": B0B / "query-schedule.json",
        "cargo_toml": artifacts / "Cargo.toml",
        "cargo_lock": artifacts / "Cargo.lock",
        "cargo_guard": B0A / "inputs/controller/cargo-guard.sh",
        "source_closure": B0A / "inputs/surviving-source-closure",
    }


def verify_host() -> dict[str, Any]:
    need(os.uname().nodename == REMOTE_HOSTNAME, "remote hostname drift")
    machine = require_file(pathlib.Path("/etc/machine-id"))
    need(machine["sha256"] == REMOTE_MACHINE_ID_SHA256, "remote machine-id drift")
    online = pathlib.Path("/sys/devices/system/cpu/online").read_text().strip()
    core = pathlib.Path("/sys/bus/event_source/devices/cpu_core/cpus").read_text().strip()
    atom = pathlib.Path("/sys/bus/event_source/devices/cpu_atom/cpus").read_text().strip()
    need(online == "0-19" and core == "0-11" and atom == "12-19", "remote topology drift")
    return {"hostname": REMOTE_HOSTNAME, "machine_id_sha256": machine["sha256"], "online": online, "core": core, "atom": atom}


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


def verify_inputs() -> dict[str, Any]:
    paths = input_paths()
    rows = {
        "package": require_file(paths["package"], digest=EXPECTED["package"], size=140_556_462, mode="0444"),
        "sidecar": require_file(paths["sidecar"], digest=EXPECTED["sidecar"], size=3_689_884, mode="0444"),
        "v7": require_file(paths["v7"], digest=EXPECTED["v7"], size=1_606_189, mode="0444"),
        "schedule": require_file(paths["schedule"], digest=EXPECTED["schedule"], size=174_941, mode="0444"),
        "cargo_toml": require_file(paths["cargo_toml"], digest=EXPECTED["cargo_toml"], size=2_399, mode="0444"),
        "cargo_lock": require_file(paths["cargo_lock"], digest=EXPECTED["cargo_lock"], size=70_770, mode="0444"),
        "cargo_guard": require_file(paths["cargo_guard"], digest=EXPECTED["cargo_guard"]),
        "loader": require_file(LOADER, digest=EXPECTED["loader"]),
    }
    need(paths["source_closure"].is_dir() and not (paths["source_closure"] / "target").exists(), "source closure drift")
    return rows


def assemble_source(v10: bytes, fragment: bytes) -> bytes:
    need(sha256_bytes(v10) == EXPECTED["v10"] and len(v10) == 91_518, "V10 source drift")
    need(sha256_bytes(fragment) == EXPECTED["fragment"] and len(fragment) == 155_810, "M2 fragment drift")
    need(v10.endswith(b"}\n"), "V10 terminal brace drift")
    assembled = v10[:-2] + fragment + b"}\n"
    need(len(assembled) == 247_328 and sha256_bytes(assembled) == EXPECTED["assembled"], "assembled M2 source drift")
    need(sha256_bytes(assembled[:39_047]) == EXPECTED["production_prefix"], "production prefix drift")
    return assembled


def verify_bootstrap(path: pathlib.Path) -> dict[str, Any]:
    payload = load_json(path / "PAYLOAD.json")
    need(payload.get("task_id") == TASK_ID and payload.get("transaction_id") == TRANSACTION_ID, "bootstrap namespace drift")
    need(payload.get("execution_admission_verdict") == "M2_EXECUTION_ADMITTED", "execution admission missing")
    files = {}
    for name, digest in payload.get("files", {}).items():
        files[name] = require_file(path / name, digest=digest)
    need({"v10.rs", "fragment.inc", "local-controller.py", "remote-controller.py", "bootstrap-auditor.py", "build-auditor.py", "terminal-auditor.py"} <= set(files), "bootstrap inventory incomplete")
    assemble_source((path / "v10.rs").read_bytes(), (path / "fragment.inc").read_bytes())
    return {"payload": payload, "files": files}


def marker_payload(route: str, controller_sha256: str) -> bytes:
    need(route in MARKER_ROUTES, f"unknown marker route: {route}")
    return canonical_json_bytes({
        "schema": "lay.v10.e1-traversal-w1-fused-minimum-m2-marker.v1",
        "task_id": TASK_ID,
        "transaction_id": TRANSACTION_ID,
        "route": route,
        "controller_sha256": controller_sha256,
        "one_shot": True,
        "retry_permitted": False,
    })


def marker_name(route: str) -> str:
    return route.lower().replace("-", "_")


def consume_marker(route: str, controller_sha256: str) -> dict[str, Any]:
    markers = STATE / "markers"
    available = markers / f"{marker_name(route)}.available"
    consumed = markers / f"{marker_name(route)}.consumed-before-exec"
    expected = marker_payload(route, controller_sha256)
    before = require_file(available, digest=sha256_bytes(expected), mode="0400")
    need(available.read_bytes() == expected and not consumed.exists(), f"marker state drift: {route}")
    os.rename(available, consumed)
    fsync_directory(markers)
    after = require_file(consumed, digest=before["sha256"], size=before["size_bytes"], mode="0400")
    return {"before": before, "after": after, "consumed_before_execution": True}


def marker_inventory() -> dict[str, list[str]]:
    markers = STATE / "markers"
    return {
        "available": sorted(path.name for path in markers.glob("*.available")),
        "consumed": sorted(path.name for path in markers.glob("*.consumed-before-exec")),
    }


def write_manifest(root: pathlib.Path) -> None:
    rows = []
    for path in sorted(candidate for candidate in root.rglob("*") if candidate.is_file() and candidate.name != "SHA256SUMS"):
        rows.append(f"{sha256_file(path)}  {path.relative_to(root).as_posix()}\n")
    write_new_bytes(root / "SHA256SUMS", "".join(rows).encode())


def seal_tree(root: pathlib.Path) -> None:
    for path in sorted(root.rglob("*"), reverse=True):
        path.chmod(0o555 if path.is_dir() else 0o444)
    root.chmod(0o555)


def append_state(sequence: int, state: str, **extra: Any) -> None:
    write_new_json(
        STATE / f"STATE-{sequence:02d}-{state}.json",
        {
            "schema": "lay.v10.e1-traversal-w1-fused-minimum-m2-state.v1",
            "task_id": TASK_ID,
            "transaction_id": TRANSACTION_ID,
            "sequence": sequence,
            "state": state,
            "markers": marker_inventory(),
            **extra,
        },
        0o444,
    )
    fsync_directory(STATE)


def uid_capability_probe(parent: pathlib.Path) -> dict[str, Any]:
    subject = parent / "uid-probe"
    subject.mkdir(mode=0o700)
    shutil.chown(subject, user="e", group="e")
    code = (
        "import os,pathlib; p=pathlib.Path(os.environ['P']); "
        "a=p/'a'; b=p/'b'; f=open(a,'xb'); f.write(b'm2-uid-proof\\n'); f.flush(); os.fsync(f.fileno()); f.close(); "
        "os.rename(a,b); assert b.read_bytes()==b'm2-uid-proof\\n'; b.unlink()"
    )
    result = run(["/usr/bin/sudo", "-n", "-u", "e", "/usr/bin/env", f"P={subject}", "/usr/bin/python3", "-c", code], check=False)
    need(result.returncode == 0 and not any(subject.iterdir()), "UID e path/write capability proof failed")
    subject.chmod(0o555)
    return {"uid": 1000, "operations": ["traverse", "create", "write", "fsync", "rename", "read", "unlink"], "verdict": "PASS"}


def bootstrap_once(bootstrap: pathlib.Path) -> dict[str, Any]:
    need(os.geteuid() == 0, "bootstrap requires root controller")
    need(not PARENT.exists() and not STATE.exists(), "M2 namespace already exists")
    closure = verify_bootstrap(bootstrap)
    host = verify_host()
    inputs = verify_inputs()
    stage = pathlib.Path(f"{PARENT}.stage-{os.getpid()}-{time.time_ns()}")
    state_stage = pathlib.Path(f"{STATE}.stage-{os.getpid()}-{time.time_ns()}")
    stage.mkdir(parents=True, mode=0o755)
    state_stage.mkdir(parents=True, mode=0o755)
    try:
        copied = stage / "bootstrap-v1"
        shutil.copytree(bootstrap, copied)
        for path in copied.rglob("*"):
            path.chmod(0o555 if path.is_dir() else 0o444)
        copied.chmod(0o555)
        uid = uid_capability_probe(stage)
        write_new_bytes(state_stage / "route.lock", b"m2-route-lock\n", 0o600)
        receipt = {
            "schema": "lay.v10.e1-traversal-w1-fused-minimum-m2-bootstrap.v1",
            "task_id": TASK_ID,
            "transaction_id": TRANSACTION_ID,
            "verdict": "M2_BOOTSTRAP_CREATED_UNAUDITED",
            "host": host,
            "inputs": inputs,
            "bootstrap": closure,
            "uid_capability": uid,
            "markers_expected": len(MARKER_ROUTES),
            "markers_created": 0,
            "markers_consumed": 0,
            "cargo_invocations": 0,
            "perf_stat_invocations": 0,
            "subject_executions": 0,
            "runtime_authority_changed": False,
        }
        write_new_json(stage / "M2_BOOTSTRAP_RECEIPT.json", receipt)
        write_manifest(stage)
        seal_tree(stage)
        os.rename(stage, PARENT)
        os.rename(state_stage, STATE)
        fsync_directory(PARENT.parent)
        fsync_directory(STATE.parent)
        append_state(0, "BOOTSTRAP_CREATED_UNAUDITED")
        return receipt
    except BaseException:
        if stage.exists():
            shutil.rmtree(stage)
        if state_stage.exists():
            shutil.rmtree(state_stage)
        raise


def create_markers(admission: pathlib.Path) -> dict[str, Any]:
    value = load_json(admission)
    need(value.get("verdict") == "M2_BOOTSTRAP_AUDIT_PASS_MARKERS_ADMITTED", "bootstrap audit did not admit markers")
    need(value.get("task_id") == TASK_ID and value.get("transaction_id") == TRANSACTION_ID, "bootstrap audit namespace drift")
    controller_sha = str(value.get("local_controller_sha256", ""))
    need(re.fullmatch(r"[0-9a-f]{64}", controller_sha) is not None, "controller SHA absent from bootstrap audit")
    markers = STATE / "markers"
    need(not markers.exists(), "M2 marker tree already exists")
    stage = STATE / f"markers.stage-{os.getpid()}-{time.time_ns()}"
    stage.mkdir(mode=0o700)
    try:
        for route in MARKER_ROUTES:
            write_new_bytes(stage / f"{marker_name(route)}.available", marker_payload(route, controller_sha), 0o400)
        os.rename(stage, markers)
        fsync_directory(STATE)
    except BaseException:
        if stage.exists():
            shutil.rmtree(stage)
        raise
    append_state(1, "ALL_MARKERS_AVAILABLE", bootstrap_audit_sha256=sha256_file(admission))
    return {"verdict": "M2_ALL_MARKERS_AVAILABLE", "markers": marker_inventory(), "markers_created": len(MARKER_ROUTES), "markers_consumed": 0}


def make_writable(root: pathlib.Path) -> None:
    for path in [root, *root.rglob("*")]:
        path.chmod(0o700 if path.is_dir() else 0o600)


def find_test_elf(target: pathlib.Path) -> pathlib.Path:
    candidates = []
    for path in (target / "release/deps").glob("lay-*"):
        if path.is_file() and os.access(path, os.X_OK) and path.read_bytes()[:4] == b"\x7fELF":
            candidates.append(path)
    need(len(candidates) == 1, f"expected one test ELF, found {len(candidates)}")
    return candidates[0]


def elf_build_id(path: pathlib.Path) -> str:
    output = run(["/usr/bin/readelf", "-n", str(path)]).stdout
    match = re.search(rb"Build ID:\s*([0-9a-f]+)", output)
    need(match is not None, "ELF Build ID absent")
    return match.group(1).decode()


def build_once(controller_sha: str) -> dict[str, Any]:
    need(not BUILD.exists() and not (PARENT / "build-failure-v1").exists(), "M2 build artifact already exists")
    paths = input_paths()
    bootstrap = PARENT / "bootstrap-v1"
    assembled = assemble_source((bootstrap / "v10.rs").read_bytes(), (bootstrap / "fragment.inc").read_bytes())
    stage = PARENT / f"build-v1.stage-{os.getpid()}-{time.time_ns()}"
    workspace = PARENT / f"workspace-{os.getpid()}-{time.time_ns()}"
    stage.mkdir(mode=0o700)
    workspace.mkdir(mode=0o700)
    marker: dict[str, Any] | None = None
    cargo_started = False
    try:
        shutil.copytree(paths["source_closure"], workspace, dirs_exist_ok=True)
        make_writable(workspace)
        shutil.copyfile(paths["cargo_toml"], workspace / "Cargo.toml")
        shutil.copyfile(paths["cargo_lock"], workspace / "Cargo.lock")
        (workspace / "scripts").mkdir(exist_ok=True)
        shutil.copyfile(paths["cargo_guard"], workspace / "scripts/cargo-guard.sh")
        (workspace / "scripts/cargo-guard.sh").chmod(0o775)
        source = workspace / "src/nanda_wave/l2_field/v13_typed_peak.rs"
        source.parent.mkdir(parents=True, exist_ok=True)
        source.write_bytes(assembled)
        source.chmod(0o444)
        cargo = run(["cargo", "-V"], env=controlled_environment()).stdout.decode().strip()
        rustc = run(["rustc", "-Vv"], env=controlled_environment()).stdout.decode().strip()
        need(cargo == EXPECTED_CARGO and all(token in rustc for token in EXPECTED_RUSTC), "toolchain drift")
        environment = controlled_environment()
        environment.update(BUILD_ENVIRONMENT_FIXED)
        environment["CARGO_TARGET_DIR"] = str(workspace / "target")
        command = [str(workspace / "scripts/cargo-guard.sh"), "test", "--offline", "--locked", "--release", "--lib", "--no-run", PHYSICAL_TEST]
        write_new_json(stage / "PREBUILD.json", {
            "task_id": TASK_ID,
            "transaction_id": TRANSACTION_ID,
            "controller_sha256": controller_sha,
            "assembled_source_sha256": EXPECTED["assembled"],
            "command": command,
            "environment": {key: environment[key] for key in BUILD_ENVIRONMENT_KEYS},
            "cargo_started": False,
            "retry_permitted": False,
        })
        marker = consume_marker("BUILD", controller_sha)
        write_new_json(stage / "BUILD_MARKER_CONSUMED.json", marker)
        cargo_started = True
        with (stage / "cargo.log").open("wb") as log:
            result = run(command, cwd=workspace, env=environment, timeout=10_800, check=False, stdout=log, stderr=subprocess.STDOUT)
            log.flush()
            os.fsync(log.fileno())
        need(result.returncode == 0, f"M2 Cargo exited {result.returncode}")
        elf = find_test_elf(workspace / "target")
        candidate = stage / "diagnostic-test-elf"
        shutil.copyfile(elf, candidate)
        candidate.chmod(0o555)
        provenance = {
            "schema": "lay.v10.e1-traversal-w1-fused-minimum-m2-build.v1",
            "task_id": TASK_ID,
            "transaction_id": TRANSACTION_ID,
            "verdict": "M2_BUILD_CREATED_UNAUDITED",
            "source": {"v10_sha256": EXPECTED["v10"], "fragment_sha256": EXPECTED["fragment"], "assembled_source_sha256": EXPECTED["assembled"], "production_prefix_sha256": EXPECTED["production_prefix"]},
            "build": {"command": command, "environment": {key: environment[key] for key in BUILD_ENVIRONMENT_KEYS}, "cargo_invocations": 1, "exit_code": 0, "marker": marker, "retry_permitted": False},
            "executable": {"sha256": sha256_file(candidate), "size_bytes": candidate.stat().st_size, "mode": mode_string(candidate), "build_id": elf_build_id(candidate), "executed": False},
            "perf_stat_invocations": 0,
            "runtime_authority_changed": False,
        }
        write_new_json(stage / "BUILD_PROVENANCE.json", provenance)
        write_new_bytes(stage / "diagnostic-source.rs", assembled)
        write_manifest(stage)
        seal_tree(stage)
        os.rename(stage, BUILD)
        fsync_directory(PARENT)
        append_state(2, "BUILD_CREATED_UNAUDITED", build_provenance_sha256=sha256_file(BUILD / "BUILD_PROVENANCE.json"))
        return provenance
    except BaseException as error:
        if marker is not None and stage.exists():
            failure = {
                "schema": "lay.v10.e1-traversal-w1-fused-minimum-m2-build-failure.v1",
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
            write_new_json(stage / "BUILD_FAILURE.json", failure)
            write_manifest(stage)
            seal_tree(stage)
            failure_root = PARENT / "build-failure-v1"
            os.rename(stage, failure_root)
            fsync_directory(PARENT)
            append_state(
                2,
                "BLOCKED_BUILD",
                build_failure_sha256=sha256_file(failure_root / "BUILD_FAILURE.json"),
            )
            return failure
        raise
    finally:
        if workspace.exists():
            shutil.rmtree(workspace)


def subject_environment(output: pathlib.Path, route: str) -> dict[str, str]:
    paths = input_paths()
    value = controlled_environment()
    value.update({
        "LAY_V10_D1_PACKAGE": str(paths["package"]),
        "LAY_V10_D1_SIDECAR": str(paths["sidecar"]),
        "LAY_V10_D1_V7": str(paths["v7"]),
        "LAY_V10_D1_SCHEDULE": str(paths["schedule"]),
        "LAY_V10_D1_OUTPUT": str(output),
        "LAY_V10_D1_RUN_ID": route,
        "LAY_V10_D1_CPUS": "0",
        "LAY_V10_M2_ROUTE": route,
    })
    return value


def child_as_e(environment: Mapping[str, str], test: str) -> list[str]:
    assignments = [f"{key}={value}" for key, value in sorted(environment.items())]
    return ["/usr/bin/sudo", "-n", "-u", "e", "/usr/bin/env", *assignments, str(LOADER), str(BUILD / "diagnostic-test-elf"), "--exact", test, "--ignored", "--nocapture", "--test-threads=1"]


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


def wait_file(process: subprocess.Popen[bytes], path: pathlib.Path, timeout: float) -> None:
    deadline = time.monotonic() + timeout
    while time.monotonic() < deadline:
        if path.is_file():
            return
        need(process.poll() is None, f"subject exited before {path.name}")
        time.sleep(0.002)
    raise M2RemoteError(f"timeout waiting for {path}")


def open_fifo(path: pathlib.Path, flags: int, deadline: float) -> int:
    while time.monotonic() < deadline:
        try:
            return os.open(path, flags | os.O_NONBLOCK)
        except OSError as error:
            if error.errno not in (6, 11):
                raise
            time.sleep(0.005)
    raise M2RemoteError(f"FIFO open timeout: {path}")


def read_fifo_line(descriptor: int, deadline: float) -> str:
    value = bytearray()
    while time.monotonic() < deadline:
        try:
            chunk = os.read(descriptor, 4096)
        except BlockingIOError:
            chunk = b""
        if chunk:
            value.extend(chunk)
            if b"\n" in value:
                return bytes(value).decode(errors="replace").strip()
        time.sleep(0.002)
    raise M2RemoteError("perf acknowledgement timeout")


def require_build_audit(path: pathlib.Path) -> dict[str, Any]:
    value = load_json(path)
    need(value.get("verdict") == "M2_BUILD_AUDITED_PARITY_ADMITTED", "build audit did not admit parity")
    need(value.get("task_id") == TASK_ID and value.get("transaction_id") == TRANSACTION_ID, "build audit namespace drift")
    need(value.get("elf_sha256") == sha256_file(BUILD / "diagnostic-test-elf"), "build audit ELF link drift")
    return value


def parity_once(controller_sha: str, build_audit: pathlib.Path) -> dict[str, Any]:
    require_build_audit(build_audit)
    root = PARENT / "parity-v1"
    need(not root.exists(), "M2 parity already exists")
    stage = PARENT / f"parity-v1.stage-{os.getpid()}-{time.time_ns()}"
    subject = stage / "subject"
    stage.mkdir(mode=0o755)
    subject.mkdir(mode=0o700)
    shutil.chown(subject, user="e", group="e")
    environment = subject_environment(subject, "PARITY")
    command = child_as_e(environment, PARITY_TEST)
    write_new_json(
        stage / "PRE_PARITY.json",
        {
            "task_id": TASK_ID,
            "transaction_id": TRANSACTION_ID,
            "command": command,
            "environment": environment,
            "subject_started": False,
            "retry_permitted": False,
        },
    )
    fsync_directory(stage)
    marker = consume_marker("PARITY", controller_sha)
    process: subprocess.Popen[bytes] | None = None
    stdout = b""
    stderr = b""
    controller_error = None
    try:
        process = subprocess.Popen(
            command,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            start_new_session=True,
        )
        stdout, stderr = process.communicate(timeout=10_800)
    except BaseException as error:
        controller_error = f"{type(error).__name__}: {error}"
        terminate(process)
        if process is not None and process.stdout is not None and process.stderr is not None:
            with contextlib.suppress(Exception):
                stdout, stderr = process.communicate(timeout=1)
    write_new_bytes(stage / "stdout.log", stdout)
    write_new_bytes(stage / "stderr.log", stderr)
    receipt_path = subject / "SUBJECT_RECEIPT.json"
    receipt = None
    if receipt_path.is_file():
        try:
            receipt = load_json(receipt_path)
        except BaseException as error:
            controller_error = controller_error or f"{type(error).__name__}: {error}"
    passed = (
        controller_error is None
        and process is not None
        and process.returncode == 0
        and isinstance(receipt, dict)
        and receipt.get("verdict") == "PASS"
    )
    verdict = "PASS" if passed else ("BLOCKED_PROVENANCE" if controller_error else "BLOCKED_PARITY")
    wrapper = {
        "schema": "lay.v10.e1-traversal-w1-fused-minimum-m2-parity-wrapper.v2",
        "task_id": TASK_ID,
        "transaction_id": TRANSACTION_ID,
        "verdict": verdict,
        "controller_error": controller_error,
        "marker": marker,
        "command": command,
        "environment": environment,
        "exit_code": process.returncode if process is not None else None,
        "subject_receipt": receipt,
        "outputs_complete": isinstance(receipt, dict),
        "subject_executions": int(process is not None),
        "perf_stat_invocations": 0,
        "runtime_authority_changed": False,
        "retry_permitted": False,
    }
    write_new_json(stage / "PARITY_WRAPPER.json", wrapper)
    write_manifest(stage)
    seal_tree(stage)
    os.rename(stage, root)
    fsync_directory(PARENT)
    append_state(3, "PARITY_PASS" if verdict == "PASS" else verdict, parity_wrapper_sha256=sha256_file(root / "PARITY_WRAPPER.json"))
    return wrapper


def run_physical(route: str, controller_sha: str) -> dict[str, Any]:
    need(route in ROUTE_ORDER, f"unknown M2 route: {route}")
    index = ROUTE_ORDER.index(route)
    previous = "PARITY_PASS" if index == 0 else f"{ROUTE_ORDER[index - 1]}_PASS"
    states = sorted(STATE.glob("STATE-*.json"))
    need(states and load_json(states[-1]).get("state") == previous, f"M2 route predecessor drift: {route}")
    root = PARENT / f"route-{index + 1:02d}-{marker_name(route)}"
    need(not root.exists(), f"M2 route already exists: {route}")
    stage = PARENT / f"{root.name}.stage-{os.getpid()}-{time.time_ns()}"
    subject = stage / "subject"
    control = stage / "control"
    stage.mkdir(mode=0o755)
    subject.mkdir(mode=0o700)
    control.mkdir(mode=0o700)
    shutil.chown(subject, user="e", group="e")
    shutil.chown(control, user="e", group="e")
    control_fifo = stage / "perf-control.fifo"
    ack_fifo = stage / "perf-ack.fifo"
    os.mkfifo(control_fifo, 0o600)
    os.mkfifo(ack_fifo, 0o600)
    environment = subject_environment(subject, route)
    environment["LAY_V10_M2_CONTROL_DIR"] = str(control)
    child = child_as_e(environment, PHYSICAL_TEST)
    command = ["/usr/bin/sudo", "-n", "/usr/bin/perf", "stat", "--json-output", "--no-big-num", "--delay=-1", f"--control=fifo:{control_fifo},{ack_fifo}", "--event", ",".join(EVENTS), "--", *child]
    thermal_before = throttle_counters()
    write_new_json(
        stage / "PRE_ROUTE.json",
        {
            "route": route,
            "command": command,
            "events": list(EVENTS),
            "thermal_before": thermal_before,
            "subject_started": False,
            "retry_permitted": False,
        },
    )
    fsync_directory(stage)
    marker = consume_marker(route, controller_sha)
    write_new_json(stage / "OBSERVATION_PLAN.json", {"route": route, "command": command, "events": list(EVENTS), "marker": marker, "retry_permitted": False})
    process: subprocess.Popen[bytes] | None = None
    control_fd: int | None = None
    ack_fd: int | None = None
    stdout = b""
    stderr = b""
    enable_ack = None
    disable_ack = None
    controller_error = None
    try:
        process = subprocess.Popen(command, stdout=subprocess.PIPE, stderr=subprocess.PIPE, start_new_session=True)
        wait_file(process, control / "subject-ready", 3_600)
        deadline = time.monotonic() + 15
        control_fd = open_fifo(control_fifo, os.O_WRONLY, deadline)
        ack_fd = open_fifo(ack_fifo, os.O_RDONLY, deadline)
        os.write(control_fd, b"enable\n")
        enable_ack = read_fifo_line(ack_fd, time.monotonic() + 15)
        write_new_bytes(control / "controller-enabled", b"enabled\n")
        wait_file(process, control / "subject-done", 3_600)
        os.write(control_fd, b"disable\n")
        disable_ack = read_fifo_line(ack_fd, time.monotonic() + 15)
        write_new_bytes(control / "controller-disabled", b"disabled\n")
        stdout, stderr = process.communicate(timeout=300)
    except BaseException as error:
        controller_error = f"{type(error).__name__}: {error}"
        terminate(process)
        if process is not None and process.stdout is not None and process.stderr is not None:
            with contextlib.suppress(Exception):
                stdout, stderr = process.communicate(timeout=1)
    finally:
        if control_fd is not None:
            os.close(control_fd)
        if ack_fd is not None:
            os.close(ack_fd)
        with contextlib.suppress(FileNotFoundError):
            control_fifo.unlink()
        with contextlib.suppress(FileNotFoundError):
            ack_fifo.unlink()
    write_new_bytes(stage / "stdout.log", stdout)
    write_new_bytes(stage / "perf.raw", stderr)
    thermal_after: dict[str, int] = {}
    thermal_change: dict[str, list[int]] = {}
    receipt = None
    complete = False
    try:
        thermal_after = throttle_counters()
        thermal_change = throttle_drift(thermal_before, thermal_after)
        receipt_path = subject / "SUBJECT_RECEIPT.json"
        required = (receipt_path, subject / "component-samples.bin", subject / "structure.json")
        complete = all(path.is_file() for path in required)
        if receipt_path.is_file():
            receipt = load_json(receipt_path)
    except BaseException as error:
        controller_error = controller_error or f"{type(error).__name__}: {error}"
    pass_unchecked = controller_error is None and process is not None and process.returncode == 0 and complete and isinstance(receipt, dict) and receipt.get("verdict") == "PASS" and str(enable_ack).startswith("ack") and str(disable_ack).startswith("ack") and not thermal_change
    verdict = "PASS_UNAUDITED" if pass_unchecked else ("BLOCKED_THERMAL" if thermal_change else "BLOCKED_PROVENANCE")
    wrapper = {"schema": "lay.v10.e1-traversal-w1-fused-minimum-m2-route-wrapper.v1", "task_id": TASK_ID, "transaction_id": TRANSACTION_ID, "route": route, "verdict": verdict, "controller_error": controller_error, "marker": marker, "command": command, "environment": environment, "exit_code": process.returncode if process is not None else None, "enable_ack": enable_ack, "disable_ack": disable_ack, "subject_receipt": receipt, "outputs_complete": complete, "thermal_before": thermal_before, "thermal_after": thermal_after, "thermal_throttle_drift": thermal_change, "subject_executions": 1, "perf_stat_invocations": 1, "perf_record_invocations": 0, "runtime_authority_changed": False, "retry_permitted": False}
    write_new_json(stage / "ROUTE_WRAPPER.json", wrapper)
    write_manifest(stage)
    seal_tree(stage)
    os.rename(stage, root)
    fsync_directory(PARENT)
    terminal_state = f"{route}_PASS" if pass_unchecked else verdict
    append_state(4 + index, terminal_state, route_wrapper_sha256=sha256_file(root / "ROUTE_WRAPPER.json"))
    return wrapper


def status() -> dict[str, Any]:
    return {
        "task_id": TASK_ID,
        "transaction_id": TRANSACTION_ID,
        "parent_exists": PARENT.exists(),
        "state_exists": STATE.exists(),
        "build_exists": BUILD.exists(),
        "markers": marker_inventory() if (STATE / "markers").is_dir() else None,
        "states": [load_json(path) for path in sorted(STATE.glob("STATE-*.json"))] if STATE.is_dir() else [],
        "remote_writes": 0,
    }


def self_check() -> dict[str, Any]:
    need(set(ROUTE_ORDER) == {"B0-ITERATOR", "G0-M1-GUARDED", "I0-INTERLEAVED", "I1-INTERLEAVED", "G1-M1-GUARDED", "B1-ITERATOR"}, "route registry drift")
    need(len(MARKER_ROUTES) == 8 and MARKER_ROUTES[:2] == ("BUILD", "PARITY"), "marker registry drift")
    need(EVENTS == ("instructions", "cycles", "branches", "branch-misses", "task-clock"), "event registry drift")
    need(
        BUILD_ENVIRONMENT_FIXED
        == {
            "CARGO_BUILD_JOBS": "20",
            "CARGO_INCREMENTAL": "0",
            "CARGO_NET_OFFLINE": "true",
            "CARGO_PROFILE_RELEASE_DEBUG": "2",
            "CARGO_PROFILE_RELEASE_STRIP": "none",
            "RUSTFLAGS": "",
        },
        "build environment drift",
    )
    need(set(BUILD_ENVIRONMENT_KEYS) == {*BUILD_ENVIRONMENT_FIXED, "CARGO_TARGET_DIR"}, "build environment key closure drift")
    return {"verdict": "M2_REMOTE_CONTROLLER_VERIFIED_UNRUN", "task_id": TASK_ID, "transaction_id": TRANSACTION_ID, "routes": list(ROUTE_ORDER), "markers": list(MARKER_ROUTES), "cargo_invocations": 0, "perf_stat_invocations": 0, "subject_executions": 0, "runtime_authority_changed": False}


def parser() -> argparse.ArgumentParser:
    value = argparse.ArgumentParser()
    value.add_argument("action", choices=("self-check", "status", "bootstrap", "create-markers", "build-once", "parity-once", "run-route"))
    value.add_argument("--bootstrap", type=pathlib.Path)
    value.add_argument("--admission", type=pathlib.Path)
    value.add_argument("--controller-sha256")
    value.add_argument("--route", choices=ROUTE_ORDER)
    return value


def main() -> int:
    arguments = parser().parse_args()
    try:
        if arguments.action == "self-check":
            result = self_check()
        elif arguments.action == "status":
            result = status()
        elif arguments.action == "bootstrap":
            need(arguments.bootstrap is not None, "bootstrap path missing")
            result = bootstrap_once(arguments.bootstrap.resolve())
        else:
            need(PARENT.is_dir() and STATE.is_dir(), "M2 bootstrap absent")
            with (STATE / "route.lock").open("rb") as lock:
                fcntl.flock(lock.fileno(), fcntl.LOCK_EX)
                if arguments.action == "create-markers":
                    need(arguments.admission is not None, "bootstrap audit admission missing")
                    result = create_markers(arguments.admission.resolve())
                else:
                    need(isinstance(arguments.controller_sha256, str), "controller SHA missing")
                    if arguments.action == "build-once":
                        result = build_once(arguments.controller_sha256)
                    elif arguments.action == "parity-once":
                        need(arguments.admission is not None, "build audit admission missing")
                        result = parity_once(arguments.controller_sha256, arguments.admission.resolve())
                    else:
                        need(arguments.route is not None, "physical route missing")
                        result = run_physical(arguments.route, arguments.controller_sha256)
        print(json.dumps(result, ensure_ascii=False, sort_keys=True))
        return 0
    except Exception as error:
        print(f"M2 REMOTE ERROR: {type(error).__name__}: {error}", file=sys.stderr)
        return 1


if __name__ == "__main__":
    raise SystemExit(main())
