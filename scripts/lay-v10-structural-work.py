#!/usr/bin/env python3
"""One-shot controller for the exact V10 structural-work A1 observer."""

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


TASK_ID = "slice8b-v10-structural-work-a2-20260825"
REMOTE = "e@192.168.3.94"
REMOTE_HOSTNAME = "e-MEGA-MINI-M1-13th"
REMOTE_MACHINE_ID_SHA256 = "5ac0bb537745673ef80942c2df32d586f5753c85de1c8b2453aa4e7317b77441"
REMOTE_PARENT = pathlib.Path("/home/e/.local/share/lay/provenance") / TASK_ID
REMOTE_BUILD = REMOTE_PARENT / "build-v1"
REMOTE_RESULT = REMOTE_PARENT / "result-v1"
REMOTE_STATE = pathlib.Path("/home/e/.local/state/lay") / TASK_ID
REMOTE_LOADER = pathlib.Path("/usr/lib/x86_64-linux-gnu/ld-linux-x86-64.so.2")
REMOTE_B0A = pathlib.Path(
    "/home/e/.local/share/lay/provenance/"
    "slice8b-v10-hardware-b0-b2-v3-20260824/b0a-input-closure-v2"
)
REMOTE_B0B = pathlib.Path(
    "/home/e/.local/share/lay/provenance/"
    "slice8b-v10-hardware-b0-b2-v3-20260824/b0b-schedule-closure-v1"
)

PROJECT_ROOT = pathlib.Path(__file__).resolve().parents[1]
P0 = pathlib.Path("/home/ubu/.local/share/lay/provenance/slice8b-v10-f6178f")
FRAGMENT = PROJECT_ROOT / "scripts/lay_v10_structural_work_test_module.rs.inc"
CONTROLLER = pathlib.Path(__file__).resolve()
CONTRACT = PROJECT_ROOT / (
    "docs/structural_gates/"
    "LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_STRUCTURAL_WORK_A2_RUN_CORRECTION_2026-08-25.md"
)
ROUTE = PROJECT_ROOT / (
    "docs/structural_gates/"
    "LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_STRUCTURAL_WORK_A2_RUN_ROUTE.md"
)
ROUTE_RECEIPT = PROJECT_ROOT / (
    "docs/structural_gates/receipts/"
    "LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_STRUCTURAL_WORK_A2_RUN_ROUTE_RECEIPT_2026-08-25.json"
)
PREFLIGHT = PROJECT_ROOT / (
    "docs/structural_gates/preflights/"
    "LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_STRUCTURAL_WORK_A2_RUN_2026-08-25.json"
)
PREFLIGHT_RECEIPT = PROJECT_ROOT / (
    "docs/structural_gates/receipts/"
    "LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_STRUCTURAL_WORK_A2_RUN_PREFLIGHT_2026-08-25.json"
)
PMU_RECEIPT = PROJECT_ROOT / (
    "docs/structural_gates/receipts/"
    "LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_LOADED_PMU_DIAGNOSIS_COMBINED_V3_V4_2026-08-25.json"
)
LOCAL_RESULT = PROJECT_ROOT / (
    "docs/structural_gates/receipts/"
    "LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_STRUCTURAL_WORK_A2_2026-08-25"
)
ACTIVE_V11 = PROJECT_ROOT / "src/nanda_wave/l2_field/v13_typed_peak.rs"

EXPECTED = {
    "contract": "fd2278841f5c3d65fd08410816660503967b22634b6aa6a3b64c41e3e63df038",
    "route": "f930455bfb75a9a893043a1b93d83e50c03654df7189156389f2ff9c3ad0170e",
    "route_receipt": "27641d35a9b08f21cb5ef618a0d22378bb0bcab8f3b8154bd11053805e395407",
    "preflight": "1d9217b6fa456868ebf502853b31ad55b0358c78f76dc3c5a8eff988cea7cced",
    "preflight_receipt": "523e6ca13c527d0d3c1c8c96fe0a4b0d50e547f3e03a11447978395a442d2e4e",
    "pmu_receipt": "ea9a19cace1eab5418f783dfb6c18a4de2adb7281356afffd12bb2b28cdacbd1",
    "v10_source": "f6178f3882c1a216a9c2aaf7ff59dd6bdd78798708516b753ea7208177d9718c",
    "production_prefix": "ce9ea2d290602774e2f45444e10e6fdabce6af4a61ae2bdca54c6d2e31a53f26",
    "active_v11": "d9edfe7346b8636096701d4ac38044d5e386e6741a086a4d8da57ac15bffdf3b",
    "b0a_manifest": "920374efc2e75a021d53235aafea2a74dc7258546219bcae9c6a2bf53e194916",
    "b0a_receipt": "48176ec6faae86f43ddda8404542367be5b6c9d6813762dedb829b4946593eb3",
    "b0b_schedule": "2f8346d97246ac3704434279739fa5ed27705465d628890a367c21875d879e78",
    "v13_package": "cce259fe0ce5dce67702383363b66f0fe9b9ff5a87d8f01c4fcf342d91218d7b",
    "sidecar": "a1aa95be8c43fae8fedf02d3261dd1265dcbfbc8d618330fcd6449ae829df8cd",
    "v7": "33fded73e13f565bdc08a83bd440be67c80f6d85a196453050b09bc9f7ef28e4",
    "cargo_guard": "a5b01f9a12b32ee2bcfc957a5e9f65efb52fe6e7644850ae3f00e412648243fe",
    "loader": "8d06f393f4a93bcf9b81145a259524d66a95522a646bf8d7e05b6ffdf2e63dcc",
}

TEST_NAME = "nanda_wave::l2_field::v13_typed_peak::tests::v10_structural_work_a1"
EXPECTED_CARGO_VERSION = "cargo 1.97.1 (c980f4866 2026-06-30)"
EXPECTED_RUSTC = (
    "release: 1.97.1",
    "commit-hash: 8bab26f4f",
    "host: x86_64-unknown-linux-gnu",
    "LLVM version: 22.1.6",
)


class GateError(RuntimeError):
    pass


def require(condition: bool, message: str) -> None:
    if not condition:
        raise GateError(message)


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


def require_file(
    path: pathlib.Path,
    *,
    sha256: str | None = None,
    mode: str | None = None,
    size: int | None = None,
) -> dict[str, Any]:
    require(path.is_file(), f"missing file: {path}")
    value = {
        "path": str(path),
        "sha256": sha256_file(path),
        "mode": mode_string(path),
        "size_bytes": path.stat().st_size,
    }
    if sha256 is not None:
        require(value["sha256"] == sha256, f"SHA-256 mismatch: {path}")
    if mode is not None:
        require(value["mode"] == mode, f"mode mismatch: {path}")
    if size is not None:
        require(value["size_bytes"] == size, f"size mismatch: {path}")
    return value


def load_json(path: pathlib.Path) -> Any:
    with path.open("rb") as source:
        return json.load(source)


def canonical_json_bytes(value: Any) -> bytes:
    return (
        json.dumps(value, ensure_ascii=False, sort_keys=True, separators=(",", ":")) + "\n"
    ).encode()


def write_new_bytes(path: pathlib.Path, data: bytes, mode: int = 0o444) -> None:
    descriptor = os.open(path, os.O_WRONLY | os.O_CREAT | os.O_EXCL, 0o600)
    try:
        view = memoryview(data)
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


def run(
    command: Sequence[str],
    *,
    cwd: pathlib.Path | None = None,
    env: dict[str, str] | None = None,
    check: bool = True,
    stdout: int | Any = subprocess.PIPE,
    stderr: int | Any = subprocess.PIPE,
) -> subprocess.CompletedProcess[bytes]:
    result = subprocess.run(
        list(command), cwd=cwd, env=env, stdout=stdout, stderr=stderr, check=False
    )
    if check and result.returncode != 0:
        detail = result.stderr.decode(errors="replace")[-4000:] if result.stderr else ""
        raise GateError(f"command failed ({result.returncode}): {shlex.join(command)}\n{detail}")
    return result


def ssh(command: Sequence[str], *, check: bool = True) -> subprocess.CompletedProcess[bytes]:
    remote_command = shlex.join(list(command))
    return run(
        ["ssh", "-o", "BatchMode=yes", "-o", "ConnectTimeout=10", REMOTE, remote_command],
        check=check,
    )


def scp(source: pathlib.Path, destination: str, *, recursive: bool = False) -> None:
    command = ["scp", "-q", "-p"]
    if recursive:
        command.append("-r")
    command.extend([str(source), f"{REMOTE}:{destination}"])
    run(command)


def assemble_source(v10: bytes, fragment: bytes) -> bytes:
    require(sha256_bytes(v10) == EXPECTED["v10_source"], "V10 source identity mismatch")
    require(v10.endswith(b"}\n"), "V10 terminal brace mismatch")
    require(fragment.startswith(b"\n    const A1_STRUCTURAL_TEST"), "A1 fragment prefix mismatch")
    final = v10[:-2] + fragment + b"}\n"
    require(final[:39_047] == v10[:39_047], "V10 production prefix changed")
    require(
        sha256_bytes(final[:39_047]) == EXPECTED["production_prefix"],
        "V10 production prefix SHA mismatch",
    )
    return final


def verify_local_admission() -> dict[str, Any]:
    files = {
        "contract": require_file(CONTRACT, sha256=EXPECTED["contract"]),
        "route": require_file(ROUTE, sha256=EXPECTED["route"]),
        "route_receipt": require_file(ROUTE_RECEIPT, sha256=EXPECTED["route_receipt"]),
        "preflight": require_file(PREFLIGHT, sha256=EXPECTED["preflight"]),
        "preflight_receipt": require_file(
            PREFLIGHT_RECEIPT, sha256=EXPECTED["preflight_receipt"]
        ),
        "pmu_receipt": require_file(PMU_RECEIPT, sha256=EXPECTED["pmu_receipt"]),
        "v10": require_file(
            P0 / "artifacts/v13_typed_peak.v10.rs",
            sha256=EXPECTED["v10_source"],
            mode="0444",
            size=91_518,
        ),
        "active_v11": require_file(ACTIVE_V11, sha256=EXPECTED["active_v11"]),
        "fragment": require_file(FRAGMENT),
        "controller": require_file(CONTROLLER),
    }
    route_receipt = load_json(ROUTE_RECEIPT)
    require(route_receipt.get("verdict") == "PASS", "A1 structural route is not PASS")
    require(route_receipt.get("authority_ready") is False, "A1 route gained authority")
    preflight = load_json(PREFLIGHT_RECEIPT)
    require(preflight.get("verdict") == "READY_TO_IMPLEMENT", "A1 preflight is not ready")
    require(preflight.get("safe_to_implement") is True, "A1 preflight is unsafe")
    require(not preflight.get("blockers"), "A1 preflight has blockers")
    require(load_json(PMU_RECEIPT).get("v12_admitted") is False, "PMU receipt admits V12")
    return files


def consume_marker(name: str) -> pathlib.Path:
    markers = REMOTE_STATE / "markers"
    available = markers / f"{name}.available"
    consumed = markers / f"{name}.consumed-before-exec"
    require(available.is_file(), f"{name} marker unavailable")
    require(not consumed.exists(), f"{name} marker already consumed")
    os.rename(available, consumed)
    fsync_directory(markers)
    return consumed


def fsync_directory(path: pathlib.Path) -> None:
    descriptor = os.open(path, os.O_RDONLY | os.O_DIRECTORY)
    try:
        os.fsync(descriptor)
    finally:
        os.close(descriptor)


def make_tree_writable(root: pathlib.Path) -> None:
    for path in sorted(root.rglob("*")):
        path.chmod(0o700 if path.is_dir() else 0o600)
    root.chmod(0o700)


def seal_tree(root: pathlib.Path) -> None:
    for path in sorted(root.rglob("*"), reverse=True):
        path.chmod(0o555 if path.is_dir() else 0o444)
    root.chmod(0o555)


def remove_tree(path: pathlib.Path) -> None:
    if not path.exists():
        return
    make_tree_writable(path)
    shutil.rmtree(path)


def manifest_rows(root: pathlib.Path, *, exclude: set[str] | None = None) -> list[dict[str, Any]]:
    excluded = exclude or set()
    rows = []
    for path in sorted(candidate for candidate in root.rglob("*") if candidate.is_file()):
        relative = path.relative_to(root).as_posix()
        if relative in excluded:
            continue
        rows.append(
            {
                "path": relative,
                "sha256": sha256_file(path),
                "size_bytes": path.stat().st_size,
                "mode": mode_string(path),
            }
        )
    return rows


def write_sha256sums(root: pathlib.Path) -> None:
    rows = manifest_rows(root, exclude={"SHA256SUMS"})
    lines = "".join(f"{row['sha256']}  {row['path']}\n" for row in rows)
    write_new_bytes(root / "SHA256SUMS", lines.encode(), 0o444)


def verify_sha256sums(root: pathlib.Path) -> int:
    manifest = root / "SHA256SUMS"
    require(manifest.is_file(), f"missing SHA256SUMS: {root}")
    count = 0
    for line in manifest.read_text(encoding="utf-8").splitlines():
        digest, relative = line.split("  ", 1)
        path = root / relative
        require(path.is_file(), f"manifest file missing: {path}")
        require(sha256_file(path) == digest, f"manifest SHA mismatch: {path}")
        count += 1
    return count


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
    output = run(["readelf", "-n", str(path)]).stdout
    match = re.search(rb"Build ID:\s*([0-9a-f]+)", output)
    require(match is not None, "ELF Build ID missing")
    return match.group(1).decode()


def controlled_environment() -> dict[str, str]:
    return {
        "HOME": "/home/e",
        "PATH": "/home/e/.cargo/bin:/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin",
        "LANG": "C.UTF-8",
        "LC_ALL": "C.UTF-8",
        "TZ": "Europe/Tallinn",
    }


def remote_machine_identity() -> dict[str, str]:
    value = {
        "hostname": os.uname().nodename,
        "machine_id_sha256": sha256_file(pathlib.Path("/etc/machine-id")),
    }
    require(value["hostname"] == REMOTE_HOSTNAME, "remote hostname mismatch")
    require(
        value["machine_id_sha256"] == REMOTE_MACHINE_ID_SHA256,
        "remote machine identity mismatch",
    )
    return value


def environment_snapshot() -> dict[str, Any]:
    def text(path: str) -> str | None:
        with contextlib.suppress(OSError):
            return pathlib.Path(path).read_text(encoding="utf-8").strip()
        return None

    process = run(
        ["ps", "-eo", "pid=,comm=,pcpu=,psr=", "--sort=-pcpu"], check=False
    ).stdout.decode(errors="replace").splitlines()[:30]
    temperatures = {}
    for path in sorted(pathlib.Path("/sys/class/thermal").glob("thermal_zone*/temp")):
        with contextlib.suppress(OSError, ValueError):
            temperatures[str(path)] = int(path.read_text().strip())
    return {
        "loadavg": text("/proc/loadavg"),
        "cpu_pressure": text("/proc/pressure/cpu"),
        "io_pressure": text("/proc/pressure/io"),
        "memory_pressure": text("/proc/pressure/memory"),
        "temperatures_millicelsius": temperatures,
        "top_processes": process,
    }


@contextlib.contextmanager
def remote_lock() -> Iterable[None]:
    lock = REMOTE_STATE / "route.lock"
    require(lock.is_file(), "A1 route lock missing")
    descriptor = os.open(lock, os.O_RDONLY)
    try:
        try:
            fcntl.flock(descriptor, fcntl.LOCK_EX | fcntl.LOCK_NB)
        except BlockingIOError as error:
            raise GateError("another A1 owner holds the route lock") from error
        yield
    finally:
        with contextlib.suppress(OSError):
            fcntl.flock(descriptor, fcntl.LOCK_UN)
        os.close(descriptor)


def initialize_remote_state() -> None:
    require(not REMOTE_PARENT.exists(), "A1 remote parent already exists")
    require(not REMOTE_STATE.exists(), "A1 remote state already exists")
    REMOTE_PARENT.mkdir(parents=True, mode=0o700)
    markers = REMOTE_STATE / "markers"
    markers.mkdir(parents=True, mode=0o700)
    write_new_bytes(markers / "build.available", b"one guarded build\n", 0o400)
    write_new_bytes(markers / "run.available", b"one structural run\n", 0o400)
    write_new_bytes(REMOTE_STATE / "route.lock", b"A1\n", 0o400)
    fsync_directory(markers)
    fsync_directory(REMOTE_STATE)


def validate_bootstrap(bootstrap: pathlib.Path) -> dict[str, Any]:
    expected = {
        "controller.py": sha256_file(bootstrap / "controller.py"),
        "fragment.inc": sha256_file(bootstrap / "fragment.inc"),
        "contract.md": EXPECTED["contract"],
        "route.md": EXPECTED["route"],
        "route-receipt.json": EXPECTED["route_receipt"],
        "preflight.json": EXPECTED["preflight"],
        "preflight-receipt.json": EXPECTED["preflight_receipt"],
        "combined-pmu.json": EXPECTED["pmu_receipt"],
    }
    rows = {}
    for name, digest in expected.items():
        rows[name] = require_file(bootstrap / name, sha256=digest)
    require(load_json(bootstrap / "route-receipt.json").get("verdict") == "PASS", "route receipt drift")
    require(
        load_json(bootstrap / "preflight-receipt.json").get("verdict")
        == "READY_TO_IMPLEMENT",
        "preflight receipt drift",
    )
    return rows


def remote_build(bootstrap: pathlib.Path) -> None:
    remote_machine_identity()
    require(REMOTE_B0A.is_dir(), "sealed B0a closure missing")
    require(REMOTE_B0B.is_dir(), "sealed B0b closure missing")
    require_file(REMOTE_B0A / "SHA256SUMS", sha256=EXPECTED["b0a_manifest"], mode="0444")
    require_file(REMOTE_B0A / "INPUT_CLOSURE.json", sha256=EXPECTED["b0a_receipt"], mode="0444")
    require_file(REMOTE_B0B / "query-schedule.json", sha256=EXPECTED["b0b_schedule"], mode="0444")
    bootstrap_rows = validate_bootstrap(bootstrap)
    initialize_remote_state()
    stage = REMOTE_PARENT / f"build-v1.stage-{os.getpid()}-{time.time_ns()}"
    workspace = REMOTE_PARENT / f"workspace-{os.getpid()}-{time.time_ns()}"
    stage.mkdir(mode=0o700)
    workspace.mkdir(mode=0o700)
    marker_consumed = False
    try:
        source_root = REMOTE_B0A / "inputs/surviving-source-closure"
        shutil.copytree(source_root, workspace, dirs_exist_ok=True)
        make_tree_writable(workspace)
        artifacts = REMOTE_B0A / "inputs/slice8b-v10-f6178f/artifacts"
        shutil.copyfile(artifacts / "Cargo.toml", workspace / "Cargo.toml")
        shutil.copyfile(artifacts / "Cargo.lock", workspace / "Cargo.lock")
        (workspace / "scripts").mkdir(exist_ok=True)
        shutil.copyfile(
            REMOTE_B0A / "inputs/controller/cargo-guard.sh",
            workspace / "scripts/cargo-guard.sh",
        )
        (workspace / "scripts/cargo-guard.sh").chmod(0o775)
        v10_path = artifacts / "v13_typed_peak.v10.rs"
        require_file(v10_path, sha256=EXPECTED["v10_source"], mode="0444", size=91_518)
        fragment = (bootstrap / "fragment.inc").read_bytes()
        final_source = assemble_source(v10_path.read_bytes(), fragment)
        source_path = workspace / "src/nanda_wave/l2_field/v13_typed_peak.rs"
        source_path.parent.mkdir(parents=True, exist_ok=True)
        source_path.write_bytes(final_source)
        source_path.chmod(0o444)
        inputs = stage / "inputs"
        inputs.mkdir()
        for name in bootstrap_rows:
            shutil.copyfile(bootstrap / name, inputs / name)
            (inputs / name).chmod(0o444)
        write_new_bytes(stage / "diagnostic-source.rs", final_source)
        cargo = run(["cargo", "-V"], env=controlled_environment()).stdout.decode().strip()
        rustc = run(["rustc", "-Vv"], env=controlled_environment()).stdout.decode().strip()
        require(cargo == EXPECTED_CARGO_VERSION, f"Cargo drift: {cargo}")
        for expected in EXPECTED_RUSTC:
            require(expected in rustc, f"rustc drift: missing {expected}")
        prerequisite = {
            "host": remote_machine_identity(),
            "cargo": cargo,
            "rustc_vv": rustc,
            "cargo_incremental": "0",
            "cargo_net_offline": "true",
            "rustflags": "",
            "build_jobs": "20",
            "loaded_host_is_blocker": False,
        }
        write_new_json(
            stage / "PREBUILD_PROVENANCE.json",
            {
                "schema": "lay.v10.structural-work-a1-prebuild.v1",
                "task_id": TASK_ID,
                "recovered_v10_sha256": EXPECTED["v10_source"],
                "production_prefix_bytes": 39_047,
                "production_prefix_sha256": EXPECTED["production_prefix"],
                "fragment_sha256": sha256_bytes(fragment),
                "final_source_sha256": sha256_bytes(final_source),
                "prerequisites": prerequisite,
                "cargo_started_when_written": False,
            },
        )
        consume_marker("build")
        marker_consumed = True
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
            TEST_NAME,
        ]
        with (stage / "cargo.log").open("wb") as log:
            result = run(
                command,
                cwd=workspace,
                env=environment,
                check=False,
                stdout=log,
                stderr=subprocess.STDOUT,
            )
            log.flush()
            os.fsync(log.fileno())
        require(result.returncode == 0, f"guarded A1 build failed with {result.returncode}")
        executable = find_test_executable(workspace / "target")
        shutil.copyfile(executable, stage / "diagnostic-test-elf")
        (stage / "diagnostic-test-elf").chmod(0o555)
        write_new_json(
            stage / "EXECUTABLE_PROVENANCE.json",
            {
                "schema": "lay.v10.structural-work-a1-executable.v1",
                "task_id": TASK_ID,
                "source": {
                    "recovered_v10_sha256": EXPECTED["v10_source"],
                    "production_prefix_bytes": 39_047,
                    "production_prefix_sha256": EXPECTED["production_prefix"],
                    "fragment_sha256": sha256_bytes(fragment),
                    "final_source_sha256": sha256_bytes(final_source),
                    "full_source_closure": "WATCH",
                },
                "build": {
                    "command": command,
                    "build_marker_consumed_before_cargo": True,
                    "retry_permitted": False,
                    "prerequisites": prerequisite,
                },
                "executable": {
                    "sha256": sha256_file(stage / "diagnostic-test-elf"),
                    "size_bytes": (stage / "diagnostic-test-elf").stat().st_size,
                    "build_id": elf_build_id(stage / "diagnostic-test-elf"),
                    "test_entrypoint": TEST_NAME,
                },
                "executed": False,
                "perf_invoked": False,
                "pmu_event_opened": False,
                "latency_authority": False,
                "runtime_authority_changed": False,
                "installed_lay_changed": False,
            },
        )
        write_sha256sums(stage)
        seal_tree(stage)
        os.rename(stage, REMOTE_BUILD)
        fsync_directory(REMOTE_PARENT)
        print(json.dumps({"state": "A1_EXECUTABLE_SEALED", "build": str(REMOTE_BUILD)}))
    except Exception as error:
        if marker_consumed:
            with contextlib.suppress(Exception):
                write_new_json(
                    stage / "FAILURE.json",
                    {
                        "schema": "lay.v10.structural-work-a1-build-failure.v1",
                        "error": str(error),
                        "build_marker_consumed": True,
                        "retry_permitted": False,
                    },
                )
                write_sha256sums(stage)
                seal_tree(stage)
                os.rename(stage, REMOTE_PARENT / "build-failure-v1")
        raise
    finally:
        remove_tree(workspace)


def validate_subject(value: dict[str, Any]) -> dict[str, Any]:
    require(value.get("verdict") == "STRUCTURAL_WORK_OBSERVED_NO_PROMOTION", "A1 subject did not pass")
    require(value.get("records") == 382, "A1 record count mismatch")
    parity = value.get("production_parity", {})
    for field in (
        "terminal_mismatches",
        "peak_mismatches",
        "completeness_mismatches",
        "expanded_state_mismatches",
        "scratch_mismatches",
        "unresolved",
        "false_certificates",
    ):
        require(parity.get(field) == 0, f"A1 parity mismatch: {field}")
    require(parity.get("target_form_retained") == 382, "target form retention mismatch")
    require(parity.get("target_lemma_retained") == 382, "target lemma retention mismatch")
    require(parity.get("maximum_product_states") == 35_590, "maximum states mismatch")
    require(parity.get("maximum_scratch_bytes") == 6_656, "maximum scratch mismatch")
    aggregate = value.get("aggregate", {})
    require(aggregate.get("additive_identities_pass") is True, "additive identities failed")
    require(aggregate.get("allocation_conservation_failures") == 0, "allocation conservation failed")
    counters = aggregate.get("counters", {})
    require(counters.get("expanded_states") == 8_059_788, "expanded-state total mismatch")
    require(counters.get("terminal_refs_post_dedup") == 17_600, "terminal total mismatch")
    require(counters.get("edges_examined", 0) > counters.get("expanded_states", 0), "edge denominator invalid")
    require(value.get("formal_b_pass") is False, "A1 promoted formal B")
    require(value.get("v12_admitted") is False, "A1 admitted V12")
    require(value.get("runtime_authority_changed") is False, "A1 changed runtime authority")
    return counters


def physical_model(subject: dict[str, Any], pmu: dict[str, Any]) -> dict[str, Any]:
    counters = subject["aggregate"]["counters"]
    requests = subject["records"]
    b5_instructions = pmu["g0"]["b5"]["instructions_per_request"]
    b6_instructions = pmu["g0"]["b6"]["instructions_per_request"]

    def per_request(field: str) -> float:
        return counters[field] / requests

    edges = per_request("edges_examined")
    cells = per_request("band_cells_evaluated")
    survivors = per_request("surviving_edges")
    return {
        "schema": "lay.v10.structural-work-a1-physical-model.v1",
        "requests": requests,
        "structural_per_request": {
            field: per_request(field)
            for field in (
                "expanded_states",
                "edges_examined",
                "transition_calls",
                "band_cells_evaluated",
                "query_symbol_comparisons",
                "surviving_edges",
                "pruned_edges",
                "stack_pushes",
                "stack_pops",
                "terminal_refs_post_dedup",
                "certificate_calls",
            )
        },
        "derived": {
            "edges_per_expanded_state": counters["edges_examined"] / counters["expanded_states"],
            "survival_rate": counters["surviving_edges"] / counters["edges_examined"],
            "band_cells_per_transition": counters["band_cells_evaluated"] / counters["transition_calls"],
            "b5_instructions_per_examined_edge": b5_instructions / edges,
            "b6_instructions_per_examined_edge": b6_instructions / edges,
            "b5_instructions_per_transition": b5_instructions / edges,
            "b5_instructions_per_band_cell": b5_instructions / cells,
            "b5_instructions_per_surviving_edge": b5_instructions / survivors,
        },
        "pmu_input": {
            "receipt_sha256": EXPECTED["pmu_receipt"],
            "b5_instructions_per_request": b5_instructions,
            "b6_instructions_per_request": b6_instructions,
        },
        "claim_boundary": {
            "latency_proven": False,
            "causal_attribution": False,
            "formal_b_pass": False,
            "v12_admitted": False,
            "runtime_authority_changed": False,
        },
    }


def remote_run() -> None:
    remote_machine_identity()
    require(REMOTE_BUILD.is_dir(), "sealed A1 build missing")
    require(not REMOTE_RESULT.exists(), "A1 result already exists")
    verify_sha256sums(REMOTE_BUILD)
    provenance = load_json(REMOTE_BUILD / "EXECUTABLE_PROVENANCE.json")
    executable = REMOTE_BUILD / "diagnostic-test-elf"
    require_file(
        executable,
        sha256=provenance["executable"]["sha256"],
        mode="0444",
        size=provenance["executable"]["size_bytes"],
    )
    require(elf_build_id(executable) == provenance["executable"]["build_id"], "Build ID drift")
    require_file(REMOTE_LOADER, sha256=EXPECTED["loader"], mode="0755", size=240_936)
    run_controller = pathlib.Path(__file__).resolve()
    run_controller_identity = require_file(run_controller)
    stage = REMOTE_PARENT / f"result-v1.stage-{os.getpid()}-{time.time_ns()}"
    stage.mkdir(mode=0o700)
    marker_consumed = False
    try:
        before = environment_snapshot()
        write_new_json(stage / "ENVIRONMENT_BEFORE.json", before)
        consume_marker("run")
        marker_consumed = True
        output = stage / "SUBJECT_RESULT.json"
        environment = controlled_environment()
        artifacts = REMOTE_B0A / "inputs/slice8b-v10-f6178f/artifacts"
        environment.update(
            {
                "LAY_V10_A1_V13_PACKAGE": str(REMOTE_B0A / "inputs/LAY-L2-RU-FULL-v13.bin"),
                "LAY_V10_A1_SIDECAR": str(artifacts / "LAY-L2-RU-FULL-v13.dafsa"),
                "LAY_V10_A1_V7": str(artifacts / "slice8b-v7-fixed-13x100.json"),
                "LAY_V10_A1_SCHEDULE": str(REMOTE_B0B / "query-schedule.json"),
                "LAY_V10_A1_OUTPUT": str(output),
            }
        )
        command = [
            str(REMOTE_LOADER),
            str(executable),
            TEST_NAME,
            "--ignored",
            "--exact",
            "--nocapture",
            "--test-threads=1",
        ]
        with (stage / "subject.stdout").open("wb") as stdout, (
            stage / "subject.stderr"
        ).open("wb") as stderr:
            result = run(command, env=environment, check=False, stdout=stdout, stderr=stderr)
            stdout.flush()
            stderr.flush()
            os.fsync(stdout.fileno())
            os.fsync(stderr.fileno())
        require(result.returncode == 0, f"A1 subject failed with {result.returncode}")
        require(output.is_file(), "A1 subject result missing")
        subject = load_json(output)
        counters = validate_subject(subject)
        pmu = load_json(REMOTE_BUILD / "inputs/combined-pmu.json")
        model = physical_model(subject, pmu)
        write_new_json(stage / "PHYSICAL_MODEL.json", model)
        after = environment_snapshot()
        write_new_json(stage / "ENVIRONMENT_AFTER.json", after)
        write_new_json(
            stage / "RUN_PROVENANCE.json",
            {
                "schema": "lay.v10.structural-work-a1-run.v1",
                "task_id": TASK_ID,
                "command": command,
                "subject_elf_sha256": provenance["executable"]["sha256"],
                "subject_elf_build_id": provenance["executable"]["build_id"],
                "loader": {
                    "path": str(REMOTE_LOADER),
                    "sha256": EXPECTED["loader"],
                    "size_bytes": 240_936,
                    "mode": "0755",
                },
                "run_controller": run_controller_identity,
                "run_marker_consumed_before_subject": True,
                "retry_permitted": False,
                "records": subject["records"],
                "edges_examined": counters["edges_examined"],
                "transition_calls": counters["transition_calls"],
                "band_cells_evaluated": counters["band_cells_evaluated"],
                "loaded_host_is_blocker": False,
                "foreign_process_control": False,
                "perf_invoked": False,
                "pmu_event_opened": False,
                "latency_authority": False,
                "formal_b_pass": False,
                "v12_admitted": False,
                "runtime_authority_changed": False,
                "installed_lay_changed": False,
            },
        )
        write_sha256sums(stage)
        seal_tree(stage)
        os.rename(stage, REMOTE_RESULT)
        fsync_directory(REMOTE_PARENT)
        print(json.dumps({"state": "STRUCTURAL_WORK_OBSERVED_NO_PROMOTION", "result": str(REMOTE_RESULT)}))
    except Exception as error:
        if marker_consumed:
            with contextlib.suppress(Exception):
                write_new_json(
                    stage / "FAILURE.json",
                    {
                        "schema": "lay.v10.structural-work-a1-run-failure.v1",
                        "error": str(error),
                        "run_marker_consumed": True,
                        "retry_permitted": False,
                    },
                )
                write_sha256sums(stage)
                seal_tree(stage)
                os.rename(stage, REMOTE_PARENT / "run-failure-v1")
        raise


def local_runtime_snapshot() -> dict[str, Any]:
    def pids(name: str) -> list[int]:
        result = run(["pgrep", "-x", name], check=False)
        return sorted(int(value) for value in result.stdout.split())

    version = run([str(pathlib.Path.home() / ".local/bin/lay"), "--version"]).stdout.decode().strip()
    return {
        "lay_version": version,
        "active_v11_sha256": sha256_file(ACTIVE_V11),
        "ibus_daemon_pid": pids("ibus-daemon"),
        "lay_daemon_pid": pids("lay-daemon"),
        "lay_ibus_engine_pid": pids("lay-ibus-engine"),
    }


def local_build() -> None:
    verify_local_admission()
    require(not LOCAL_RESULT.exists(), "local A1 result already exists")
    probe = ssh(
        [
            "python3",
            "-c",
            (
                "import hashlib,os,pathlib;"
                "p=pathlib.Path('/etc/machine-id');"
                "print(os.uname().nodename);"
                "print(hashlib.sha256(p.read_bytes()).hexdigest());"
                f"print(int(pathlib.Path('{REMOTE_PARENT}').exists()));"
                f"print(int(pathlib.Path('{REMOTE_STATE}').exists()))"
            ),
        ]
    ).stdout.decode().splitlines()
    require(probe == [REMOTE_HOSTNAME, REMOTE_MACHINE_ID_SHA256, "0", "0"], f"remote build probe failed: {probe}")
    temporary = ssh(["mktemp", "-d", "/tmp/lay-v10-a1.XXXXXX"]).stdout.decode().strip()
    require(temporary.startswith("/tmp/lay-v10-a1."), "unexpected remote bootstrap path")
    files = {
        CONTROLLER: "controller.py",
        FRAGMENT: "fragment.inc",
        CONTRACT: "contract.md",
        ROUTE: "route.md",
        ROUTE_RECEIPT: "route-receipt.json",
        PREFLIGHT: "preflight.json",
        PREFLIGHT_RECEIPT: "preflight-receipt.json",
        PMU_RECEIPT: "combined-pmu.json",
    }
    try:
        for source, name in files.items():
            scp(source, f"{temporary}/{name}")
        result = ssh(["python3", f"{temporary}/controller.py", "remote-build", temporary], check=False)
        require(result.returncode == 0, result.stderr.decode(errors="replace")[-5000:])
        print(result.stdout.decode().strip())
    finally:
        ssh(["rm", "-rf", "--", temporary], check=False)


def local_run() -> None:
    verify_local_admission()
    require(not LOCAL_RESULT.exists(), "local A1 result already exists")
    before = local_runtime_snapshot()
    temporary = ssh(["mktemp", "-d", "/tmp/lay-v10-a2-run.XXXXXX"]).stdout.decode().strip()
    require(temporary.startswith("/tmp/lay-v10-a2-run."), "unexpected run-controller path")
    try:
        remote_controller = f"{temporary}/controller.py"
        scp(CONTROLLER, remote_controller)
        remote_sha = ssh(["sha256sum", remote_controller]).stdout.decode().split()[0]
        require(remote_sha == sha256_file(CONTROLLER), "remote run-controller SHA mismatch")
        result = ssh(["python3", remote_controller, "remote-run"], check=False)
        require(result.returncode == 0, result.stderr.decode(errors="replace")[-5000:])
    finally:
        ssh(["rm", "-rf", "--", temporary], check=False)
    after = local_runtime_snapshot()
    require(before == after, f"installed runtime stable projection changed: {before} != {after}")
    stage = pathlib.Path(f"{LOCAL_RESULT}.stage-{os.getpid()}-{time.time_ns()}")
    try:
        run(["scp", "-q", "-pr", f"{REMOTE}:{REMOTE_RESULT}", str(stage)])
        count = verify_sha256sums(stage)
        subject = load_json(stage / "SUBJECT_RESULT.json")
        validate_subject(subject)
        require(not any(path.stat().st_mode & 0o222 for path in stage.rglob("*")), "remote result contains writable objects")
        seal_tree(stage)
        os.rename(stage, LOCAL_RESULT)
        fsync_directory(LOCAL_RESULT.parent)
        print(
            json.dumps(
                {
                    "state": "STRUCTURAL_WORK_OBSERVED_NO_PROMOTION",
                    "local_result": str(LOCAL_RESULT),
                    "manifest_entries": count,
                    "runtime_stable": True,
                },
                sort_keys=True,
            )
        )
    except Exception:
        remove_tree(stage)
        raise


def local_self_check() -> None:
    files = verify_local_admission()
    controller = CONTROLLER.read_text(encoding="utf-8")
    compile(controller, str(CONTROLLER), "exec")
    fragment = FRAGMENT.read_bytes()
    final = assemble_source((P0 / "artifacts/v13_typed_peak.v10.rs").read_bytes(), fragment)
    required_fragment_tokens = (
        "edges_examined",
        "transition_calls",
        "band_cells_evaluated",
        "surviving_edges",
        "stack_pushes",
        "certificate_calls",
        "A1CountingAllocator",
        "allocation_conservation_failures",
        "v10_structural_work_a1",
    )
    text = fragment.decode()
    for token in required_fragment_tokens:
        require(token in text, f"fragment lacks {token}")
    forbidden_commands = (
        "perf stat",
        "/usr/bin/perf",
        "cpupower",
        "systemctl stop",
        "systemctl restart",
        "pkill",
        "killall",
        "taskset -pc",
        "renice",
    )
    tree = ast.parse(controller)
    invoked_literals = []
    for node in ast.walk(tree):
        if not isinstance(node, ast.Call) or not isinstance(node.func, ast.Name):
            continue
        if node.func.id not in {"run", "ssh", "scp"}:
            continue
        invoked_literals.extend(
            child.value
            for child in ast.walk(node)
            if isinstance(child, ast.Constant) and isinstance(child.value, str)
        )
    invoked_text = " ".join(invoked_literals)
    for token in forbidden_commands:
        require(token not in invoked_text, f"controller invokes forbidden command {token}")
    require("search_elapsed_us" not in text, "latency field leaked into A1 fragment")
    require("total_elapsed_us" not in text, "latency field leaked into A1 fragment")
    with tempfile.TemporaryDirectory(prefix="lay-a1-self-check-") as directory:
        state = pathlib.Path(directory)
        markers = state / "markers"
        markers.mkdir()
        write_new_bytes(markers / "build.available", b"one\n", 0o400)
        available = markers / "build.available"
        consumed = markers / "build.consumed-before-exec"
        os.rename(available, consumed)
        require(not available.exists() and consumed.is_file(), "marker consumption failed")
        try:
            os.rename(available, consumed)
        except FileNotFoundError:
            pass
        else:
            raise GateError("second marker consumption unexpectedly succeeded")
    print(
        json.dumps(
            {
                "verdict": "PASS",
                "admission_files": len(files),
                "controller_sha256": sha256_file(CONTROLLER),
                "fragment_sha256": sha256_file(FRAGMENT),
                "final_source_sha256": sha256_bytes(final),
                "production_prefix_sha256": sha256_bytes(final[:39_047]),
                "test_entrypoint": TEST_NAME,
                "remote_actions_executed": 0,
                "perf_invocations": 0,
                "latency_measurements": 0,
            },
            sort_keys=True,
        )
    )


def remote_status() -> None:
    remote_machine_identity()
    value = {
        "parent": REMOTE_PARENT.exists(),
        "build": REMOTE_BUILD.exists(),
        "result": REMOTE_RESULT.exists(),
        "state": REMOTE_STATE.exists(),
        "markers": sorted(path.name for path in (REMOTE_STATE / "markers").glob("*"))
        if (REMOTE_STATE / "markers").is_dir()
        else [],
    }
    print(json.dumps(value, sort_keys=True))


def parser() -> argparse.ArgumentParser:
    value = argparse.ArgumentParser()
    value.add_argument(
        "action",
        choices=("self-check", "build", "run", "status", "remote-build", "remote-run", "remote-status"),
    )
    value.add_argument("argument", nargs="?")
    return value


def main() -> int:
    arguments = parser().parse_args()
    try:
        if arguments.action == "self-check":
            local_self_check()
        elif arguments.action == "build":
            local_build()
        elif arguments.action == "run":
            local_run()
        elif arguments.action == "status":
            result = ssh(["python3", str(REMOTE_BUILD / "inputs/controller.py"), "remote-status"], check=False)
            if result.returncode == 0:
                print(result.stdout.decode().strip())
            else:
                probe = ssh(["test", "-e", str(REMOTE_PARENT)], check=False)
                print(json.dumps({"parent": probe.returncode == 0, "build": False, "result": False}))
        elif arguments.action == "remote-build":
            require(os.uname().nodename == REMOTE_HOSTNAME, "remote-build on wrong host")
            require(arguments.argument is not None, "remote bootstrap path missing")
            remote_build(pathlib.Path(arguments.argument))
        elif arguments.action == "remote-run":
            require(os.uname().nodename == REMOTE_HOSTNAME, "remote-run on wrong host")
            with remote_lock():
                remote_run()
        elif arguments.action == "remote-status":
            remote_status()
        return 0
    except Exception as error:
        print(f"ERROR: {error}", file=sys.stderr)
        return 1


if __name__ == "__main__":
    raise SystemExit(main())
