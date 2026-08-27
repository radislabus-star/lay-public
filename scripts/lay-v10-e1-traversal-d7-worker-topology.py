#!/usr/bin/env python3
"""One-shot D7 worker/topology sweep for the sealed V10 E1 traversal."""

from __future__ import annotations

import argparse
import ast
import contextlib
import fcntl
import hashlib
import json
import math
import os
import pathlib
import re
import shlex
import shutil
import signal
import stat
import struct
import subprocess
import sys
import tempfile
import time
from collections.abc import Iterable, Sequence
from typing import Any


TASK_ID = "slice8b-v10-e1-traversal-d7-worker-topology-sweep-v1-20260826"
TRANSACTION_ID = "d0982f48bba3090a155713c32a73bbc71f7ef79a0f5fa1eccbea4423563102e0"
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

CONTROLLER = pathlib.Path(__file__).resolve()
REMOTE_BOOTSTRAP = CONTROLLER.name == "controller.py"
PROJECT_ROOT = (
    pathlib.Path("/home/ubu/projects/lay-l1-exact-peak-search")
    if REMOTE_BOOTSTRAP
    else CONTROLLER.parents[1]
)
BOOTSTRAP_ROOT = CONTROLLER.parent if REMOTE_BOOTSTRAP else None
P0 = pathlib.Path("/home/ubu/.local/share/lay/provenance/slice8b-v10-f6178f")
V10_SOURCE = (
    BOOTSTRAP_ROOT / "v13_typed_peak.v10.rs"
    if REMOTE_BOOTSTRAP
    else P0 / "artifacts/v13_typed_peak.v10.rs"
)
FRAGMENT = (
    BOOTSTRAP_ROOT / "fragment.inc"
    if REMOTE_BOOTSTRAP
    else PROJECT_ROOT / "scripts/lay_v10_e1_traversal_d7_worker_topology_test_module.rs.inc"
)
D1_FRAGMENT = PROJECT_ROOT / "scripts/lay_v10_e1_remaining_cost_d1_test_module.rs.inc"
PAPER = (
    BOOTSTRAP_ROOT / "paper.md"
    if REMOTE_BOOTSTRAP
    else PROJECT_ROOT
    / "docs/structural_gates/"
    "LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_D7_WORKER_TOPOLOGY_SWEEP_V1_2026-08-26.md"
)
STRUCTURAL_REVIEW = (
    BOOTSTRAP_ROOT / "structural-review.json"
    if REMOTE_BOOTSTRAP
    else PROJECT_ROOT
    / "docs/structural_gates/receipts/"
    "LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_D7_WORKER_TOPOLOGY_SWEEP_STRUCTURAL_REVIEW_V1_2026-08-26.json"
)
PREFLIGHT = (
    BOOTSTRAP_ROOT / "preflight-v2.json"
    if REMOTE_BOOTSTRAP
    else PROJECT_ROOT
    / "docs/structural_gates/preflights/"
    "LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_D7_WORKER_TOPOLOGY_SWEEP_IMPLEMENTATION_V2_2026-08-26.json"
)
PREFLIGHT_RECEIPT = (
    BOOTSTRAP_ROOT / "preflight-v2-receipt.json"
    if REMOTE_BOOTSTRAP
    else PROJECT_ROOT
    / "docs/structural_gates/receipts/"
    "LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_D7_WORKER_TOPOLOGY_SWEEP_IMPLEMENTATION_V2_PREFLIGHT_2026-08-26.json"
)
D6_RECEIPT = (
    BOOTSTRAP_ROOT / "d6-receipt.json"
    if REMOTE_BOOTSTRAP
    else PROJECT_ROOT
    / "docs/structural_gates/receipts/"
    "LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_D6_CONCURRENCY_ACCOUNTING_V1_2026-08-26/"
    "D6_CONCURRENCY_ACCOUNTING_RECEIPT.json"
)
D1_DECISION = (
    BOOTSTRAP_ROOT / "d1-decision.json"
    if REMOTE_BOOTSTRAP
    else PROJECT_ROOT
    / "docs/structural_gates/receipts/"
    "LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_REMAINING_COST_D1_2026-08-25/"
    "D1_DECISION.json"
)
E1_DECISION = (
    BOOTSTRAP_ROOT / "e1-decision.json"
    if REMOTE_BOOTSTRAP
    else PROJECT_ROOT
    / "docs/structural_gates/receipts/"
    "LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_EXACT_FUSED_EXECUTOR_E1_2026-08-25/"
    "E1_DECISION.json"
)
LOCAL_RESULT = PROJECT_ROOT / (
    "docs/structural_gates/receipts/"
    "LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_D7_WORKER_TOPOLOGY_SWEEP_V1_2026-08-26"
)
TERMINAL_AUDITOR = PROJECT_ROOT / "scripts/lay-v10-e1-traversal-d7-terminal-audit.py"
IMPLEMENTATION_RECEIPT = PROJECT_ROOT / (
    "docs/structural_gates/receipts/"
    "LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_D7_WORKER_TOPOLOGY_SWEEP_"
    "IMPLEMENTATION_SELF_CHECK_V1_2026-08-26.json"
)

EXPECTED = {
    "paper": "090c39efec9916c1a9bea050c6385a17c00b95532e08592730b3b83ede591a23",
    "structural_review": "c94d5d927e27c6f0aeaa5be331b4894bbd6d41f3aaab16a1f8bfe0a7c233da3e",
    "preflight": "c35e144ed81bd244bbf7d2233f0f8da86fb6fd4aedd1f6d5e25608b9bd0cc700",
    "preflight_receipt": "bb92b6d1059090b0da024cca686202c19386b85b97fc2563adcca84107400c20",
    "d6_receipt": "cc1fc1c7e74258cd7fec7eed5a113bbaeb3a4bf8ee3b269825f4cd282f5755dc",
    "d1_decision": "80530f9f5787f846ce2cf222c1b60e3ae42887ce95a11ac153ec7271cce98baf",
    "e1_decision": "b334c047d29b21c27923fba9b38bbf17bb642cc72c9b112add1c38d8c9b0beab",
    "d1_fragment": "bbd8b8d318810eec721812f21efbeb5f231dacba774cb5ade854e2201c6c7665",
    "fragment": "8c9ff3aaf43942aff6090b1350cef1828e24ea5664d312bde2ebdf29be6687ce",
    "fragment_suffix": "c15d7f55339560ed3b6182b9f7caf68617f4a7f45bedeac4a6fc6334d1f80952",
    "v10_source": "f6178f3882c1a216a9c2aaf7ff59dd6bdd78798708516b753ea7208177d9718c",
    "production_prefix": "ce9ea2d290602774e2f45444e10e6fdabce6af4a61ae2bdca54c6d2e31a53f26",
    "assembled_source": "c9f6400304eed2d84717cbef84b17e5ba1817e08f8d9e68580b5d441d1f13803",
    "b0a_manifest": "920374efc2e75a021d53235aafea2a74dc7258546219bcae9c6a2bf53e194916",
    "b0a_receipt": "48176ec6faae86f43ddda8404542367be5b6c9d6813762dedb829b4946593eb3",
    "schedule": "2f8346d97246ac3704434279739fa5ed27705465d628890a367c21875d879e78",
    "package": "cce259fe0ce5dce67702383363b66f0fe9b9ff5a87d8f01c4fcf342d91218d7b",
    "sidecar": "a1aa95be8c43fae8fedf02d3261dd1265dcbfbc8d618330fcd6449ae829df8cd",
    "v7": "33fded73e13f565bdc08a83bd440be67c80f6d85a196453050b09bc9f7ef28e4",
    "cargo_guard": "a5b01f9a12b32ee2bcfc957a5e9f65efb52fe6e7644850ae3f00e412648243fe",
    "cargo_toml": "90a3ae6b16677ff49b70aab3457b794cd1e114a547f9e113a8b49595f976207b",
    "cargo_lock": "e6399be48c393e1557b451fcb04d886f9fbdaa9812109410ac3d9ecdc98b93f1",
    "loader": "8d06f393f4a93bcf9b81145a259524d66a95522a646bf8d7e05b6ffdf2e63dcc",
}

EXPECTED_CARGO_VERSION = "cargo 1.97.1 (c980f4866 2026-06-30)"
EXPECTED_RUSTC = (
    "release: 1.97.1",
    "commit-hash: 8bab26f4f68e0e26f0bb7960be334d5b520ea452",
    "host: x86_64-unknown-linux-gnu",
    "LLVM version: 22.1.6",
)
PARITY_TEST = "nanda_wave::l2_field::v13_typed_peak::tests::v10_d1_semantic_parity"
SWEEP_TEST = "nanda_wave::l2_field::v13_typed_peak::tests::v10_d7_worker_topology_sweep"
ROUTE_CPUS: dict[str, tuple[int, ...]] = {
    "W1": (0,),
    "W6": (0, 2, 4, 6, 8, 10),
    "W12": tuple(range(12)),
    "W14": (0, 2, 4, 6, 8, 10, 12, 13, 14, 15, 16, 17, 18, 19),
    "W20": tuple(range(20)),
}
ROUTE_ORDER = tuple(ROUTE_CPUS)
MARKER_ROUTES = ("build", "parity", *ROUTE_ORDER)
HARDWARE_EVENTS = ("instructions", "cycles", "branches", "branch-misses")
ALL_EVENTS = (*HARDWARE_EVENTS, "task-clock")
QUERIES = 382
ROUNDS = 20
EDGES_PER_ROUND = 25_145_756
MEASURED_EDGES = EDGES_PER_ROUND * ROUNDS
COMPONENT_SAMPLE = struct.Struct("<HHBB" + "Q" * 14)
PHASES = ("oracle", "lanes", "eqmask", "traversal", "merge", "certificate")
W1_BASELINE_NS_PER_EDGE = 25.96501044152341
W20_BASELINE_NS_PER_EDGE = 44.735012045372585


class D7Error(RuntimeError):
    pass


class TerminalRouteError(D7Error):
    def __init__(self, verdict: str, route: str, detail: str) -> None:
        super().__init__(detail)
        self.verdict = verdict
        self.route = route
        self.detail = detail


def need(condition: bool, message: str) -> None:
    if not condition:
        raise D7Error(message)


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
    need(path.is_file(), f"missing file: {path}")
    return {
        "path": str(path),
        "sha256": sha256_file(path),
        "size_bytes": path.stat().st_size,
        "mode": mode_string(path),
    }


def require_file(
    path: pathlib.Path,
    *,
    sha256: str | None = None,
    size: int | None = None,
    mode: str | None = None,
) -> dict[str, Any]:
    value = file_identity(path)
    if sha256 is not None:
        need(value["sha256"] == sha256, f"SHA-256 mismatch: {path}")
    if size is not None:
        need(value["size_bytes"] == size, f"size mismatch: {path}")
    if mode is not None:
        need(value["mode"] == mode, f"mode mismatch: {path}")
    return value


def load_json(path: pathlib.Path) -> Any:
    with path.open("rb") as source:
        return json.load(source)


def canonical_json_bytes(value: Any) -> bytes:
    return (
        json.dumps(value, ensure_ascii=False, sort_keys=True, separators=(",", ":")) + "\n"
    ).encode()


def write_new_bytes(path: pathlib.Path, value: bytes, mode: int = 0o444) -> None:
    descriptor = os.open(path, os.O_WRONLY | os.O_CREAT | os.O_EXCL, 0o600)
    try:
        view = memoryview(value)
        while view:
            written = os.write(descriptor, view)
            need(written > 0, f"short write made no progress: {path}")
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


def fsync_directory(path: pathlib.Path) -> None:
    descriptor = os.open(path, os.O_RDONLY | os.O_DIRECTORY)
    try:
        os.fsync(descriptor)
    finally:
        os.close(descriptor)


def make_tree_writable(root: pathlib.Path) -> None:
    if not root.exists():
        return
    for path in sorted(root.rglob("*")):
        path.chmod(0o700 if path.is_dir() else 0o600)
    root.chmod(0o700)


def remove_owned_tree(root: pathlib.Path) -> None:
    if root.exists():
        make_tree_writable(root)
        shutil.rmtree(root)


def seal_tree(root: pathlib.Path) -> None:
    for path in sorted(root.rglob("*"), reverse=True):
        path.chmod(0o555 if path.is_dir() else 0o444)
    root.chmod(0o555)


def seal_remote_state() -> None:
    markers = REMOTE_STATE / "markers"
    for path in markers.glob("*"):
        if path.is_file():
            path.chmod(0o400)
    if markers.is_dir():
        markers.chmod(0o500)
    for path in REMOTE_STATE.glob("STATE-*.json"):
        path.chmod(0o444)
    lock = REMOTE_STATE / "route.lock"
    if lock.is_file():
        lock.chmod(0o400)
    REMOTE_STATE.chmod(0o500)


def manifest_rows(root: pathlib.Path, exclude: set[str] | None = None) -> list[dict[str, Any]]:
    excluded = exclude or set()
    rows = []
    for path in sorted(candidate for candidate in root.rglob("*") if candidate.is_file()):
        relative = path.relative_to(root).as_posix()
        if relative in excluded:
            continue
        value = file_identity(path)
        value["path"] = relative
        rows.append(value)
    return rows


def write_sha256sums(root: pathlib.Path) -> None:
    rows = manifest_rows(root, {"SHA256SUMS"})
    lines = "".join(f"{row['sha256']}  {row['path']}\n" for row in rows)
    write_new_bytes(root / "SHA256SUMS", lines.encode())


def verify_sha256sums(root: pathlib.Path) -> int:
    manifest = root / "SHA256SUMS"
    need(manifest.is_file(), f"missing SHA256SUMS: {root}")
    expected_paths: set[str] = set()
    count = 0
    for line in manifest.read_text(encoding="utf-8").splitlines():
        digest, relative = line.split("  ", 1)
        need(relative not in expected_paths, f"duplicate manifest path: {relative}")
        expected_paths.add(relative)
        path = root / relative
        need(path.is_file(), f"manifest file missing: {path}")
        need(sha256_file(path) == digest, f"manifest SHA mismatch: {path}")
        count += 1
    actual_paths = {
        path.relative_to(root).as_posix()
        for path in root.rglob("*")
        if path.is_file() and path != manifest
    }
    need(actual_paths == expected_paths, "SHA256SUMS inventory mismatch")
    return count


def run_command(
    command: Sequence[str],
    *,
    cwd: pathlib.Path | None = None,
    env: dict[str, str] | None = None,
    check: bool = True,
    stdout: int | Any = subprocess.PIPE,
    stderr: int | Any = subprocess.PIPE,
    timeout: float | None = None,
) -> subprocess.CompletedProcess[bytes]:
    result = subprocess.run(
        list(command),
        cwd=cwd,
        env=env,
        check=False,
        stdout=stdout,
        stderr=stderr,
        timeout=timeout,
    )
    if check and result.returncode != 0:
        detail = result.stderr.decode(errors="replace")[-5000:] if result.stderr else ""
        raise D7Error(f"command failed ({result.returncode}): {shlex.join(command)}\n{detail}")
    return result


def ssh(command: Sequence[str], *, check: bool = True, timeout: float | None = None) -> subprocess.CompletedProcess[bytes]:
    return run_command(
        [
            "ssh",
            "-i",
            "/home/ubu/.ssh/mega-mini-admin",
            "-o",
            "BatchMode=yes",
            "-o",
            "ConnectTimeout=10",
            "-o",
            "ServerAliveInterval=15",
            "-o",
            "ServerAliveCountMax=6",
            REMOTE,
            shlex.join(list(command)),
        ],
        check=check,
        timeout=timeout,
    )


def scp_to_remote(source: pathlib.Path, destination: pathlib.Path) -> None:
    run_command(
        [
            "scp",
            "-q",
            "-p",
            "-i",
            "/home/ubu/.ssh/mega-mini-admin",
            "-o",
            "ConnectTimeout=10",
            str(source),
            f"{REMOTE}:{destination}",
        ]
    )


def scp_from_remote(source: pathlib.Path, destination: pathlib.Path, *, recursive: bool = False) -> None:
    command = [
        "scp",
        "-q",
        "-p",
        "-i",
        "/home/ubu/.ssh/mega-mini-admin",
        "-o",
        "ConnectTimeout=10",
    ]
    if recursive:
        command.append("-r")
    command.extend([f"{REMOTE}:{source}", str(destination)])
    run_command(command)


def controlled_environment() -> dict[str, str]:
    return {
        "HOME": "/home/e",
        "PATH": "/home/e/.cargo/bin:/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin",
        "LANG": "C.UTF-8",
        "LC_ALL": "C.UTF-8",
        "TZ": "Europe/Tallinn",
    }


def assemble_source(v10: bytes, fragment: bytes) -> bytes:
    need(sha256_bytes(v10) == EXPECTED["v10_source"], "V10 source identity mismatch")
    need(v10.endswith(b"}\n"), "V10 terminal brace mismatch")
    need(len(fragment) == 123_880, "D7 fragment size mismatch")
    need(sha256_bytes(fragment) == EXPECTED["fragment"], "D7 fragment SHA mismatch")
    need(sha256_bytes(fragment[:113_204]) == EXPECTED["d1_fragment"], "D1 fragment prefix drift")
    need(sha256_bytes(fragment[113_204:]) == EXPECTED["fragment_suffix"], "D7 suffix drift")
    final = v10[:-2] + fragment + b"}\n"
    need(len(final) == 215_398, "assembled D7 source size mismatch")
    need(sha256_bytes(final) == EXPECTED["assembled_source"], "assembled D7 source SHA mismatch")
    need(final[:39_047] == v10[:39_047], "V10 production prefix changed")
    need(sha256_bytes(final[:39_047]) == EXPECTED["production_prefix"], "production prefix SHA mismatch")
    return final


def verify_admission() -> dict[str, Any]:
    files = {
        "paper": require_file(PAPER, sha256=EXPECTED["paper"], mode="0444", size=8_540),
        "structural_review": require_file(
            STRUCTURAL_REVIEW,
            sha256=EXPECTED["structural_review"],
            mode="0444",
            size=1_040,
        ),
        "preflight": require_file(PREFLIGHT, sha256=EXPECTED["preflight"], mode="0444", size=19_001),
        "preflight_receipt": require_file(
            PREFLIGHT_RECEIPT,
            sha256=EXPECTED["preflight_receipt"],
            mode="0444",
            size=7_774,
        ),
        "d6_receipt": require_file(D6_RECEIPT, sha256=EXPECTED["d6_receipt"], mode="0444", size=1_280),
        "d1_decision": require_file(D1_DECISION, sha256=EXPECTED["d1_decision"], mode="0444", size=5_361_257),
        "e1_decision": require_file(E1_DECISION, sha256=EXPECTED["e1_decision"], mode="0444", size=202_202),
        "v10_source": require_file(V10_SOURCE, sha256=EXPECTED["v10_source"], mode="0444", size=91_518),
        "fragment": require_file(FRAGMENT, sha256=EXPECTED["fragment"], size=123_880),
    }
    review = load_json(STRUCTURAL_REVIEW)
    need(review.get("verdict") == "STRUCTURALLY_ACCEPTED_WITH_SPLIT", "D7 structural review not accepted")
    need(all(route.get("verdict") == "PASS" for route in review.get("routes", {}).values()), "D7 route review drift")
    preflight = load_json(PREFLIGHT_RECEIPT)
    need(preflight.get("verdict") == "READY_TO_IMPLEMENT", "D7 V2 preflight not ready")
    need(preflight.get("safe_to_implement") is True and not preflight.get("blockers"), "D7 V2 preflight unsafe")
    need(load_json(D6_RECEIPT).get("verdict") == "D6_CONCURRENCY_ACCOUNTING_COMPLETE", "D6 predecessor drift")
    need(load_json(E1_DECISION).get("verdict") == "E1_REJECT", "E1 predecessor drift")
    return files


def local_self_check(*, emit: bool = True) -> dict[str, Any]:
    files = verify_admission()
    files["terminal_auditor"] = require_file(TERMINAL_AUDITOR)
    source = assemble_source(V10_SOURCE.read_bytes(), FRAGMENT.read_bytes())
    controller_text = CONTROLLER.read_text(encoding="utf-8")
    compile(controller_text, str(CONTROLLER), "exec")
    auditor_text = TERMINAL_AUDITOR.read_text(encoding="utf-8")
    compile(auditor_text, str(TERMINAL_AUDITOR), "exec")
    fragment_text = FRAGMENT.read_text(encoding="utf-8")
    required = (
        "d7_expected_cpus",
        "D7_ROUNDS: usize = 20",
        "d7_run_worker_topology_sweep",
        "v10_d7_worker_topology_sweep",
        "subject-ready",
        "controller-disabled",
        "CLOCK_THREAD_CPUTIME_ID",
        "d1_component_sample",
    )
    for token in required:
        need(token in fragment_text, f"D7 fragment missing token: {token}")
    for token in ("perf record", "precise_ip", "systemctl", "pkill", "killall", "V12Executor"):
        need(token not in fragment_text, f"D7 fragment contains forbidden token: {token}")
    need(set(ROUTE_CPUS) == {"W1", "W6", "W12", "W14", "W20"}, "D7 Python registry drift")
    tree = ast.parse(controller_text)
    execution_functions = {
        "remote_build_once",
        "run_parity_route",
        "run_worker_route",
        "subject_command",
        "child_as_e",
    }
    execution_nodes = [
        node
        for node in ast.walk(tree)
        if isinstance(node, (ast.FunctionDef, ast.AsyncFunctionDef))
        and node.name in execution_functions
    ]
    need({node.name for node in execution_nodes} == execution_functions, "D7 executable function set drift")
    literals = [
        node.value
        for function in execution_nodes
        for node in ast.walk(function)
        if isinstance(node, ast.Constant) and isinstance(node.value, str)
    ]
    invoked = " ".join(literals)
    need("perf record" not in invoked and "--pid" not in invoked, "sampling or attach route reachable")
    need("SIGINT" not in invoked, "signal shutdown route reachable")
    need("systemctl restart" not in invoked and "install-release" not in invoked, "runtime mutation reachable")
    command_graph = {
        "BUILD": {"function": "remote_build_once", "external": ["cargo-guard"]},
        "PARITY": {"function": "run_parity_route", "external": ["diagnostic-test-elf"]},
        **{
            route: {"function": "run_worker_route", "external": ["perf stat", "diagnostic-test-elf"]}
            for route in ROUTE_ORDER
        },
        "TERMINAL-AUDIT": {"function": "run_independent_terminal_audit", "external": ["terminal-auditor.py"]},
    }
    need(set(command_graph) == {"BUILD", "PARITY", *ROUTE_ORDER, "TERMINAL-AUDIT"}, "D7 command graph drift")
    for function in ("remote_build_once", "run_parity_route", "run_worker_route", "run_independent_terminal_audit"):
        need(f"def {function}(" in controller_text, f"D7 command producer missing: {function}")
    for token in ("independent_remote_probe", "audit_samples", "audit_perf", "calculate_frontiers"):
        need(token in auditor_text, f"D7 terminal auditor missing: {token}")
    with tempfile.NamedTemporaryFile(suffix=".rs") as temporary:
        temporary.write(source)
        temporary.flush()
        parsed = run_command(
            ["rustfmt", "--edition", "2024", "--emit", "stdout", temporary.name],
            check=False,
        )
    need(parsed.returncode == 0, "assembled D7 source failed rustfmt parse:\n" + parsed.stderr.decode(errors="replace")[-4000:])
    with tempfile.NamedTemporaryFile() as sample_file:
        sample_file.write(COMPONENT_SAMPLE.pack(0, 0, 0, 0, *range(14)) * (QUERIES * ROUNDS))
        sample_file.flush()
        parsed_samples = parse_component_samples(pathlib.Path(sample_file.name))
        need(
            len(parsed_samples) == QUERIES * ROUNDS
            and parsed_samples[0]["outer_thread_cpu_ns"] == 1
            and parsed_samples[0]["phases"][3]["thread_cpu_ns"] == 9,
            "D7 component sample layout self-check failed",
        )
    core_rows = []
    mixed_rows = []
    for event in HARDWARE_EVENTS:
        core_rows.extend(
            [
                {"counter-value": "100", "event": f"cpu_core/{event}/", "event-runtime": 100, "pcnt-running": 100.0},
                {"counter-value": "<not counted>", "event": f"cpu_atom/{event}/", "event-runtime": 0, "pcnt-running": 0.0},
            ]
        )
        mixed_rows.extend(
            [
                {"counter-value": "100", "event": f"cpu_core/{event}/", "event-runtime": 60, "pcnt-running": 60.0},
                {"counter-value": "200", "event": f"cpu_atom/{event}/", "event-runtime": 40, "pcnt-running": 40.0},
            ]
        )
    task_row = {"counter-value": "10", "unit": "msec", "event": "task-clock", "event-runtime": 100, "pcnt-running": 100.0}
    core_perf = parse_perf(b"".join(canonical_json_bytes(row) for row in [*core_rows, task_row]), "W1")
    mixed_perf = parse_perf(b"".join(canonical_json_bytes(row) for row in [*mixed_rows, task_row]), "W20")
    need(core_perf["counters"]["instructions"]["runtime_weighted_value"] == 100.0, "core-only perf fixture failed")
    need(mixed_perf["counters"]["instructions"]["runtime_weighted_value"] == 140.0, "mixed perf fixture failed")
    with tempfile.TemporaryDirectory(prefix="lay-d7-marker-check-") as directory:
        markers = pathlib.Path(directory)
        marker = marker_payload("W1")
        write_new_bytes(markers / "w1.available", marker, 0o400)
        os.rename(markers / "w1.available", markers / "w1.consumed-before-exec")
        fsync_directory(markers)
        need(not (markers / "w1.available").exists(), "marker remained available")
        need((markers / "w1.consumed-before-exec").read_bytes() == marker, "marker bytes changed")
    result = {
        "verdict": "D7_CONTROLLER_VERIFIED_UNRUN",
        "admission_files": files,
        "controller_sha256": sha256_file(CONTROLLER),
        "terminal_auditor_sha256": sha256_file(TERMINAL_AUDITOR),
        "fragment_sha256": EXPECTED["fragment"],
        "assembled_source_sha256": EXPECTED["assembled_source"],
        "production_prefix_sha256": EXPECTED["production_prefix"],
        "routes": {route: list(cpus) for route, cpus in ROUTE_CPUS.items()},
        "command_graph": command_graph,
        "marker_identities": {
            route: {
                "sha256": sha256_bytes(marker_payload(route)),
                "size_bytes": len(marker_payload(route)),
                "mode": "0400",
            }
            for route in MARKER_ROUTES
        },
        "cargo_invocations": 0,
        "perf_stat_invocations": 0,
        "subject_executions": 0,
        "runtime_authority_changed": False,
    }
    if emit:
        print(json.dumps(result, sort_keys=True))
    return result


def read_text(path: pathlib.Path) -> str | None:
    with contextlib.suppress(OSError):
        return path.read_text(encoding="utf-8", errors="replace").strip()
    return None


def parse_cpu_list(value: str) -> tuple[int, ...]:
    cpus: list[int] = []
    for field in value.split(","):
        field = field.strip()
        if not field:
            continue
        if "-" in field:
            start, end = (int(part) for part in field.split("-", 1))
            need(start <= end, f"invalid CPU range: {field}")
            cpus.extend(range(start, end + 1))
        else:
            cpus.append(int(field))
    need(len(cpus) == len(set(cpus)), f"duplicate CPU in list: {value}")
    return tuple(cpus)


def remote_machine_identity() -> dict[str, str]:
    machine_id = pathlib.Path("/etc/machine-id")
    value = {
        "hostname": os.uname().nodename,
        "machine_id_sha256": sha256_file(machine_id),
    }
    need(value["hostname"] == REMOTE_HOSTNAME, "remote hostname mismatch")
    need(value["machine_id_sha256"] == REMOTE_MACHINE_ID_SHA256, "remote machine identity mismatch")
    return value


def remote_topology() -> dict[str, Any]:
    online_text = read_text(pathlib.Path("/sys/devices/system/cpu/online"))
    core_text = read_text(pathlib.Path("/sys/bus/event_source/devices/cpu_core/cpus"))
    atom_text = read_text(pathlib.Path("/sys/bus/event_source/devices/cpu_atom/cpus"))
    need(online_text is not None and core_text is not None and atom_text is not None, "hybrid topology files missing")
    online = parse_cpu_list(online_text)
    core = parse_cpu_list(core_text)
    atom = parse_cpu_list(atom_text)
    need(online == tuple(range(20)), f"online CPU drift: {online}")
    need(core == tuple(range(12)), f"core PMU CPU drift: {core}")
    need(atom == tuple(range(12, 20)), f"atom PMU CPU drift: {atom}")
    sibling_sets: dict[int, tuple[int, ...]] = {}
    for cpu in online:
        value = read_text(pathlib.Path(f"/sys/devices/system/cpu/cpu{cpu}/topology/thread_siblings_list"))
        need(value is not None, f"CPU{cpu} sibling topology missing")
        sibling_sets[cpu] = parse_cpu_list(value)
    expected = {
        0: (0, 1), 1: (0, 1), 2: (2, 3), 3: (2, 3), 4: (4, 5), 5: (4, 5),
        6: (6, 7), 7: (6, 7), 8: (8, 9), 9: (8, 9), 10: (10, 11), 11: (10, 11),
        12: (12,), 13: (13,), 14: (14,), 15: (15,), 16: (16,), 17: (17,), 18: (18,), 19: (19,),
    }
    need(sibling_sets == expected, f"thread sibling topology drift: {sibling_sets}")
    return {
        "online": list(online),
        "cpu_core": list(core),
        "cpu_atom": list(atom),
        "thread_siblings": {str(cpu): list(value) for cpu, value in sibling_sets.items()},
    }


def artifacts() -> tuple[pathlib.Path, pathlib.Path, pathlib.Path, pathlib.Path]:
    root = REMOTE_B0A / "inputs/slice8b-v10-f6178f/artifacts"
    return (
        REMOTE_B0A / "inputs/LAY-L2-RU-FULL-v13.bin",
        root / "LAY-L2-RU-FULL-v13.dafsa",
        root / "slice8b-v7-fixed-13x100.json",
        REMOTE_B0B / "query-schedule.json",
    )


def pressure(path: pathlib.Path) -> dict[str, dict[str, float]]:
    rows: dict[str, dict[str, float]] = {}
    for line in (read_text(path) or "").splitlines():
        fields = line.split()
        if fields:
            rows[fields[0]] = {
                key: float(value)
                for key, value in (field.split("=", 1) for field in fields[1:])
            }
    return rows


def temperatures() -> list[dict[str, Any]]:
    values = []
    for path in sorted(pathlib.Path("/sys/class/hwmon").glob("hwmon*/temp*_input")):
        raw = read_text(path)
        if raw and raw.lstrip("-").isdigit():
            values.append(
                {
                    "path": str(path),
                    "label": read_text(path.with_name(path.name.replace("_input", "_label"))),
                    "millidegrees_c": int(raw),
                }
            )
    return values


def throttle_counters() -> dict[str, int]:
    values = {}
    for path in sorted(pathlib.Path("/sys/devices/system/cpu").glob("cpu*/thermal_throttle/*")):
        raw = read_text(path)
        if raw and raw.isdigit():
            values[str(path)] = int(raw)
    return values


def environment_snapshot() -> dict[str, Any]:
    thermal = temperatures()
    processes = run_command(
        ["ps", "-eo", "pid=,comm=,pcpu=,psr=,args=", "--sort=-pcpu"], check=False
    ).stdout.decode(errors="replace").splitlines()[:40]
    return {
        "monotonic_ns": time.perf_counter_ns(),
        "loadavg": read_text(pathlib.Path("/proc/loadavg")),
        "cpu_pressure": pressure(pathlib.Path("/proc/pressure/cpu")),
        "io_pressure": pressure(pathlib.Path("/proc/pressure/io")),
        "memory_pressure": pressure(pathlib.Path("/proc/pressure/memory")),
        "temperatures": thermal,
        "maximum_temperature_c": max(
            (value["millidegrees_c"] for value in thermal), default=0
        )
        / 1000.0,
        "throttle_counters": throttle_counters(),
        "online_cpus": read_text(pathlib.Path("/sys/devices/system/cpu/online")),
        "top_processes": processes,
    }


def thermal_drift(before: dict[str, Any], after: dict[str, Any]) -> dict[str, list[int]]:
    left = before["throttle_counters"]
    right = after["throttle_counters"]
    return {
        key: [left.get(key, 0), right.get(key, 0)]
        for key in sorted(set(left) | set(right))
        if left.get(key, 0) != right.get(key, 0)
    }


def local_runtime_snapshot() -> dict[str, Any]:
    binary_rows = {}
    for path in sorted(pathlib.Path("/home/ubu/.local/bin").glob("lay*")):
        with contextlib.suppress(OSError):
            resolved = path.resolve(strict=True)
            if resolved.is_file():
                binary_rows[path.name] = {
                    "target": str(resolved),
                    "sha256": sha256_file(resolved),
                }
    processes = run_command(
        ["pgrep", "-a", "-f", "(?:lay-daemon|lay-ibus-engine|ibus-daemon)"],
        check=False,
    ).stdout.decode(errors="replace").splitlines()
    managed = sorted(line for line in processes if "lay-daemon" in line or "lay-ibus-engine" in line)
    ibus = sorted(line for line in processes if "ibus-daemon" in line and "lay-ibus-engine" not in line)
    return {
        "installed_lay_hashes": binary_rows,
        "managed_lay_process_ids": [line.split(maxsplit=1)[0] for line in managed],
        "global_ibus_pid": ibus[0].split(maxsplit=1)[0] if ibus else None,
    }


def remote_runtime_snapshot() -> dict[str, Any]:
    binary_rows = {}
    for path in sorted(pathlib.Path("/home/e/.local/bin").glob("lay*")):
        with contextlib.suppress(OSError):
            resolved = path.resolve(strict=True)
            if resolved.is_file():
                binary_rows[path.name] = {
                    "target": str(resolved),
                    "sha256": sha256_file(resolved),
                }
    processes = run_command(
        ["pgrep", "-a", "-f", "(?:lay-daemon|lay-ibus-engine|ibus-daemon)"],
        check=False,
    ).stdout.decode(errors="replace").splitlines()
    managed = sorted(
        line for line in processes if "lay-daemon" in line or "lay-ibus-engine" in line
    )
    ibus = sorted(
        line
        for line in processes
        if "ibus-daemon" in line and "lay-ibus-engine" not in line
    )
    return {
        "installed_lay_hashes": binary_rows,
        "managed_lay_process_ids": [line.split(maxsplit=1)[0] for line in managed],
        "global_ibus_pid": ibus[0].split(maxsplit=1)[0] if ibus else None,
    }


def active_d7_processes() -> list[str]:
    rows = run_command(["ps", "-eo", "pid=,args="], check=False).stdout.decode(errors="replace").splitlines()
    needles = (TASK_ID, "v10_d7_worker_topology_sweep")
    return [row.strip() for row in rows if any(needle in row for needle in needles) and str(os.getpid()) not in row.split(maxsplit=1)[:1]]


@contextlib.contextmanager
def remote_lock() -> Iterable[None]:
    lock = REMOTE_STATE / "route.lock"
    need(lock.is_file(), "D7 route lock missing")
    descriptor = os.open(lock, os.O_RDONLY)
    try:
        try:
            fcntl.flock(descriptor, fcntl.LOCK_EX | fcntl.LOCK_NB)
        except BlockingIOError as error:
            raise D7Error("another D7 owner holds route.lock") from error
        yield
    finally:
        with contextlib.suppress(OSError):
            fcntl.flock(descriptor, fcntl.LOCK_UN)
        os.close(descriptor)


def marker_name(route: str, suffix: str) -> str:
    return f"{route.lower()}.{suffix}"


def marker_payload(route: str) -> bytes:
    return canonical_json_bytes(
        {
            "schema": "lay.v10.e1-traversal-d7-marker.v1",
            "task_id": TASK_ID,
            "transaction_id": TRANSACTION_ID,
            "route": route,
            "retry_permitted": False,
        }
    )


def marker_inventory() -> dict[str, list[str]]:
    markers = REMOTE_STATE / "markers"
    available = sorted(path.name for path in markers.glob("*.available")) if markers.is_dir() else []
    consumed = sorted(path.name for path in markers.glob("*.consumed-before-exec")) if markers.is_dir() else []
    return {"available": available, "consumed": consumed}


def consume_marker(route: str) -> dict[str, Any]:
    markers = REMOTE_STATE / "markers"
    available = markers / marker_name(route, "available")
    consumed = markers / marker_name(route, "consumed-before-exec")
    expected = marker_payload(route)
    need(available.is_file() and not consumed.exists(), f"{route} marker unavailable")
    need(available.read_bytes() == expected, f"{route} marker bytes drift")
    need(mode_string(available) == "0400", f"{route} marker mode drift")
    before = file_identity(available)
    os.rename(available, consumed)
    fsync_directory(markers)
    after = file_identity(consumed)
    need(after["sha256"] == before["sha256"] and after["size_bytes"] == before["size_bytes"], "marker identity changed")
    return {"before": before, "after": after, "consumed_before_execution": True}


def append_state(sequence: int, state: str, **extra: Any) -> pathlib.Path:
    path = REMOTE_STATE / f"STATE-{sequence:02d}-{state}.json"
    value = {
        "schema": "lay.v10.e1-traversal-d7-state.v1",
        "task_id": TASK_ID,
        "transaction_id": TRANSACTION_ID,
        "sequence": sequence,
        "state": state,
        "markers": marker_inventory(),
        "runtime_authority_changed": False,
        **extra,
    }
    write_new_json(path, value)
    fsync_directory(REMOTE_STATE)
    return path


def initialize_remote_state() -> dict[str, Any]:
    need(not REMOTE_PARENT.exists(), "D7 remote parent already exists")
    need(not REMOTE_STATE.exists(), "D7 remote state already exists")
    REMOTE_PARENT.mkdir(parents=True, mode=0o700)
    REMOTE_STATE.mkdir(parents=True, mode=0o700)
    markers = REMOTE_STATE / "markers"
    marker_stage = REMOTE_STATE / f".markers.stage-{os.getpid()}-{time.time_ns()}"
    marker_stage.mkdir(mode=0o700)
    identities = {}
    try:
        for route in MARKER_ROUTES:
            path = marker_stage / marker_name(route, "available")
            write_new_bytes(path, marker_payload(route), 0o400)
            identities[route] = file_identity(path)
        fsync_directory(marker_stage)
        write_new_bytes(REMOTE_STATE / "route.lock", b"D7\n", 0o400)
        os.rename(marker_stage, markers)
        fsync_directory(REMOTE_STATE)
        identities = {
            route: file_identity(markers / marker_name(route, "available"))
            for route in MARKER_ROUTES
        }
    except Exception:
        remove_owned_tree(marker_stage)
        raise
    append_state(0, "ALL_MARKERS_AVAILABLE", markers_expected=len(MARKER_ROUTES))
    return {"markers": identities, "inventory": marker_inventory()}


def validate_bootstrap(root: pathlib.Path) -> dict[str, Any]:
    expected = {
        "controller.py": sha256_file(CONTROLLER),
        "fragment.inc": EXPECTED["fragment"],
        "v13_typed_peak.v10.rs": EXPECTED["v10_source"],
        "paper.md": EXPECTED["paper"],
        "structural-review.json": EXPECTED["structural_review"],
        "preflight-v2.json": EXPECTED["preflight"],
        "preflight-v2-receipt.json": EXPECTED["preflight_receipt"],
        "d6-receipt.json": EXPECTED["d6_receipt"],
        "d1-decision.json": EXPECTED["d1_decision"],
        "e1-decision.json": EXPECTED["e1_decision"],
    }
    rows = {name: require_file(root / name, sha256=digest) for name, digest in expected.items()}
    need(load_json(root / "preflight-v2-receipt.json").get("verdict") == "READY_TO_IMPLEMENT", "bootstrap preflight drift")
    need(load_json(root / "d6-receipt.json").get("verdict") == "D6_CONCURRENCY_ACCOUNTING_COMPLETE", "bootstrap D6 drift")
    assemble_source((root / "v13_typed_peak.v10.rs").read_bytes(), (root / "fragment.inc").read_bytes())
    return rows


def remote_pre_marker_probe(root: pathlib.Path) -> dict[str, Any]:
    host = remote_machine_identity()
    topology = remote_topology()
    need(not REMOTE_PARENT.exists() and not REMOTE_STATE.exists(), "D7 namespace exists before markers")
    need(not active_d7_processes(), "D7 process exists before markers")
    bootstrap = validate_bootstrap(root)
    b0a = require_file(REMOTE_B0A / "SHA256SUMS", sha256=EXPECTED["b0a_manifest"], mode="0444")
    b0a_receipt = require_file(REMOTE_B0A / "INPUT_CLOSURE.json", sha256=EXPECTED["b0a_receipt"], mode="0444")
    package, sidecar, v7, schedule = artifacts()
    inputs = {
        "package": require_file(package, sha256=EXPECTED["package"], mode="0444", size=140_556_462),
        "sidecar": require_file(sidecar, sha256=EXPECTED["sidecar"], mode="0444", size=3_689_884),
        "v7": require_file(v7, sha256=EXPECTED["v7"], mode="0444", size=1_606_189),
        "schedule": require_file(schedule, sha256=EXPECTED["schedule"], mode="0444", size=174_941),
        "loader": require_file(REMOTE_LOADER, sha256=EXPECTED["loader"]),
    }
    source_artifacts = REMOTE_B0A / "inputs/slice8b-v10-f6178f/artifacts"
    inputs["cargo_toml"] = require_file(source_artifacts / "Cargo.toml", sha256=EXPECTED["cargo_toml"], mode="0444", size=2_399)
    inputs["cargo_lock"] = require_file(source_artifacts / "Cargo.lock", sha256=EXPECTED["cargo_lock"], mode="0444", size=70_770)
    inputs["cargo_guard"] = require_file(REMOTE_B0A / "inputs/controller/cargo-guard.sh", sha256=EXPECTED["cargo_guard"])
    source_root = REMOTE_B0A / "inputs/surviving-source-closure"
    need(source_root.is_dir() and not (source_root / "target").exists(), "source closure is not fresh")
    cargo = run_command(["cargo", "-V"], env=controlled_environment()).stdout.decode().strip()
    rustc = run_command(["rustc", "-Vv"], env=controlled_environment()).stdout.decode().strip()
    need(cargo == EXPECTED_CARGO_VERSION, f"Cargo drift: {cargo}")
    for expected in EXPECTED_RUSTC:
        need(expected in rustc, f"rustc drift: missing {expected}")
    sudo = run_command(["sudo", "-n", "true"], check=False)
    need(sudo.returncode == 0, "noninteractive sudo unavailable")
    disk = shutil.disk_usage(REMOTE_PARENT.parent)
    need(disk.free >= 20 * 1024**3, "less than 20 GiB free before D7 build")
    return {
        "schema": "lay.v10.e1-traversal-d7-pre-marker-probe.v1",
        "task_id": TASK_ID,
        "transaction_id": TRANSACTION_ID,
        "verdict": "D7_PRE_MARKER_PROBE_PASS",
        "host": host,
        "topology": topology,
        "bootstrap": bootstrap,
        "b0a_manifest": b0a,
        "b0a_receipt": b0a_receipt,
        "inputs": inputs,
        "cargo": cargo,
        "rustc_vv": rustc,
        "disk_free_bytes": disk.free,
        "markers_created": 0,
        "markers_consumed": 0,
        "cargo_invocations": 0,
        "perf_stat_invocations": 0,
        "subject_executions": 0,
        "remote_writes": 0,
    }


def find_test_executable(target: pathlib.Path) -> pathlib.Path:
    candidates = []
    for path in (target / "release/deps").glob("lay-*"):
        if not path.is_file() or not os.access(path, os.X_OK) or path.name.endswith((".d", ".rlib", ".rmeta")):
            continue
        with path.open("rb") as source:
            if source.read(4) == b"\x7fELF":
                candidates.append(path)
    need(len(candidates) == 1, f"expected one release test ELF, found {candidates}")
    return candidates[0]


def elf_build_id(path: pathlib.Path) -> str:
    output = run_command(["readelf", "-n", str(path)]).stdout
    match = re.search(rb"Build ID:\s*([0-9a-f]+)", output)
    need(match is not None, "ELF Build ID missing")
    return match.group(1).decode()


def remote_build_once(bootstrap: pathlib.Path) -> dict[str, Any]:
    require_file(REMOTE_B0A / "SHA256SUMS", sha256=EXPECTED["b0a_manifest"], mode="0444")
    require_file(REMOTE_B0A / "INPUT_CLOSURE.json", sha256=EXPECTED["b0a_receipt"], mode="0444")
    require_file(REMOTE_LOADER, sha256=EXPECTED["loader"])
    package, sidecar, v7, schedule = artifacts()
    package_identity = require_file(package, sha256=EXPECTED["package"], mode="0444", size=140_556_462)
    sidecar_identity = require_file(sidecar, sha256=EXPECTED["sidecar"], mode="0444", size=3_689_884)
    v7_identity = require_file(v7, sha256=EXPECTED["v7"], mode="0444", size=1_606_189)
    schedule_identity = require_file(schedule, sha256=EXPECTED["schedule"], mode="0444", size=174_941)
    source_artifacts = REMOTE_B0A / "inputs/slice8b-v10-f6178f/artifacts"
    cargo_toml_identity = require_file(source_artifacts / "Cargo.toml", sha256=EXPECTED["cargo_toml"], mode="0444", size=2_399)
    cargo_lock_identity = require_file(source_artifacts / "Cargo.lock", sha256=EXPECTED["cargo_lock"], mode="0444", size=70_770)
    cargo_guard = REMOTE_B0A / "inputs/controller/cargo-guard.sh"
    cargo_guard_identity = require_file(cargo_guard, sha256=EXPECTED["cargo_guard"])

    stage = REMOTE_PARENT / f"build-v1.stage-{os.getpid()}-{time.time_ns()}"
    workspace = REMOTE_PARENT / f"workspace-{os.getpid()}-{time.time_ns()}"
    stage.mkdir(mode=0o700)
    workspace.mkdir(mode=0o700)
    marker: dict[str, Any] | None = None
    cargo_started = False
    try:
        source_root = REMOTE_B0A / "inputs/surviving-source-closure"
        shutil.copytree(source_root, workspace, dirs_exist_ok=True)
        make_tree_writable(workspace)
        shutil.copyfile(source_artifacts / "Cargo.toml", workspace / "Cargo.toml")
        shutil.copyfile(source_artifacts / "Cargo.lock", workspace / "Cargo.lock")
        (workspace / "scripts").mkdir(exist_ok=True)
        shutil.copyfile(cargo_guard, workspace / "scripts/cargo-guard.sh")
        (workspace / "scripts/cargo-guard.sh").chmod(0o775)
        need(not (workspace / "target").exists(), "diagnostic target directory is not fresh")
        final_source = assemble_source(
            (bootstrap / "v13_typed_peak.v10.rs").read_bytes(),
            (bootstrap / "fragment.inc").read_bytes(),
        )
        source_path = workspace / "src/nanda_wave/l2_field/v13_typed_peak.rs"
        source_path.parent.mkdir(parents=True, exist_ok=True)
        source_path.write_bytes(final_source)
        source_path.chmod(0o444)
        inputs = stage / "inputs"
        inputs.mkdir()
        for source in sorted(path for path in bootstrap.iterdir() if path.is_file()):
            shutil.copyfile(source, inputs / source.name)
            (inputs / source.name).chmod(0o444)
        write_new_bytes(stage / "diagnostic-source.rs", final_source)

        cargo = run_command(["cargo", "-V"], env=controlled_environment()).stdout.decode().strip()
        rustc = run_command(["rustc", "-Vv"], env=controlled_environment()).stdout.decode().strip()
        need(cargo == EXPECTED_CARGO_VERSION, f"Cargo drift: {cargo}")
        for expected in EXPECTED_RUSTC:
            need(expected in rustc, f"rustc drift: missing {expected}")
        prebuild = {
            "schema": "lay.v10.e1-traversal-d7-prebuild.v1",
            "task_id": TASK_ID,
            "transaction_id": TRANSACTION_ID,
            "controller_sha256": sha256_file(CONTROLLER),
            "fragment_sha256": EXPECTED["fragment"],
            "assembled_source_sha256": EXPECTED["assembled_source"],
            "production_prefix_bytes": 39_047,
            "production_prefix_sha256": EXPECTED["production_prefix"],
            "cargo": cargo,
            "rustc_vv": rustc,
            "cargo_started": False,
            "retry_permitted": False,
            "inputs": {
                "package": package_identity,
                "sidecar": sidecar_identity,
                "v7": v7_identity,
                "schedule": schedule_identity,
                "cargo_toml": cargo_toml_identity,
                "cargo_lock": cargo_lock_identity,
                "cargo_guard": cargo_guard_identity,
            },
        }
        write_new_json(stage / "PREBUILD.json", prebuild)
        marker = consume_marker("build")
        write_new_json(stage / "BUILD_MARKER_CONSUMED.json", marker)
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
            SWEEP_TEST,
        ]
        write_new_json(
            stage / "BUILD_INVOCATION.json",
            {"command": command, "environment": environment, "cargo_invocations": 1},
        )
        cargo_started = True
        with (stage / "cargo.log").open("wb") as log:
            result = run_command(
                command,
                cwd=workspace,
                env=environment,
                check=False,
                stdout=log,
                stderr=subprocess.STDOUT,
            )
            log.flush()
            os.fsync(log.fileno())
        need(result.returncode == 0, f"guarded D7 build failed with {result.returncode}")
        executable = find_test_executable(workspace / "target")
        candidate = stage / "diagnostic-test-elf"
        shutil.copyfile(executable, candidate)
        candidate.chmod(0o555)
        provenance = {
            "schema": "lay.v10.e1-traversal-d7-build.v1",
            "task_id": TASK_ID,
            "transaction_id": TRANSACTION_ID,
            "source": {
                "v10_sha256": EXPECTED["v10_source"],
                "d1_fragment_sha256": EXPECTED["d1_fragment"],
                "d7_fragment_sha256": EXPECTED["fragment"],
                "assembled_source_sha256": EXPECTED["assembled_source"],
                "production_prefix_bytes": 39_047,
                "production_prefix_sha256": EXPECTED["production_prefix"],
            },
            "build": {
                "command": command,
                "environment": {key: environment[key] for key in (
                    "CARGO_BUILD_JOBS", "CARGO_INCREMENTAL", "CARGO_NET_OFFLINE", "CARGO_TARGET_DIR", "RUSTFLAGS"
                )},
                "cargo_invocations": 1,
                "exit_code": result.returncode,
                "marker": marker,
                "retry_permitted": False,
                "inputs": prebuild["inputs"],
            },
            "executable": {
                "sha256": sha256_file(candidate),
                "size_bytes": candidate.stat().st_size,
                "build_id": elf_build_id(candidate),
                "mode_before_seal": mode_string(candidate),
                "test_entrypoints": [PARITY_TEST, SWEEP_TEST],
            },
            "executed": False,
            "perf_invocations": 0,
            "runtime_authority_changed": False,
        }
        write_new_json(stage / "BUILD_PROVENANCE.json", provenance)
        write_sha256sums(stage)
        seal_tree(stage)
        os.rename(stage, REMOTE_BUILD)
        fsync_directory(REMOTE_PARENT)
        append_state(1, "BUILD_CREATED", build_provenance_sha256=sha256_file(REMOTE_BUILD / "BUILD_PROVENANCE.json"))
        return provenance
    except Exception as error:
        if marker is not None:
            with contextlib.suppress(Exception):
                write_new_json(
                    stage / "BUILD_FAILURE.json",
                    {
                        "schema": "lay.v10.e1-traversal-d7-build-failure.v1",
                        "verdict": "BLOCKED_BUILD",
                        "error": str(error),
                        "cargo_started": cargo_started,
                        "marker": marker,
                        "retry_permitted": False,
                    },
                )
                write_sha256sums(stage)
                seal_tree(stage)
                os.rename(stage, REMOTE_PARENT / "build-failure-v1")
                fsync_directory(REMOTE_PARENT)
        else:
            remove_owned_tree(stage)
        raise TerminalRouteError("BLOCKED_BUILD", "BUILD", str(error)) from error
    finally:
        remove_owned_tree(workspace)


def subject_environment(output: pathlib.Path, route: str, cpus: Sequence[int]) -> dict[str, str]:
    package, sidecar, v7, schedule = artifacts()
    environment = controlled_environment()
    environment.update(
        {
            "LAY_V10_D1_PACKAGE": str(package),
            "LAY_V10_D1_SIDECAR": str(sidecar),
            "LAY_V10_D1_V7": str(v7),
            "LAY_V10_D1_SCHEDULE": str(schedule),
            "LAY_V10_D1_OUTPUT": str(output),
            "LAY_V10_D1_RUN_ID": route,
            "LAY_V10_D1_CPUS": ",".join(map(str, cpus)),
        }
    )
    return environment


def subject_command(test: str) -> list[str]:
    return [
        str(REMOTE_LOADER),
        str(REMOTE_BUILD / "diagnostic-test-elf"),
        "--exact",
        test,
        "--ignored",
        "--nocapture",
        "--test-threads=1",
    ]


def child_as_e(environment: dict[str, str], command: Sequence[str]) -> list[str]:
    assignments = [f"{key}={value}" for key, value in sorted(environment.items())]
    return ["/usr/bin/sudo", "-n", "-u", "e", "/usr/bin/env", *assignments, *command]


def terminate_owned(process: subprocess.Popen[bytes] | None) -> None:
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


def wait_for_file(process: subprocess.Popen[bytes], path: pathlib.Path, timeout: float) -> None:
    deadline = time.monotonic() + timeout
    while time.monotonic() < deadline:
        if path.is_file():
            return
        need(process.poll() is None, f"subject exited before {path.name}")
        time.sleep(0.002)
    raise D7Error(f"timeout waiting for {path}")


def open_fifo(path: pathlib.Path, flags: int, deadline: float) -> int:
    while time.monotonic() < deadline:
        try:
            return os.open(path, flags | os.O_NONBLOCK)
        except OSError as error:
            if error.errno not in (6, 11):
                raise
            time.sleep(0.005)
    raise D7Error(f"FIFO open timeout: {path}")


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
    raise D7Error("perf control acknowledgement timeout")


def validate_parity(value: dict[str, Any]) -> None:
    zero_fields = (
        "terminal_mismatches",
        "peak_mismatches",
        "completeness_mismatches",
        "work_mismatches",
        "rank_prefix_mismatches",
        "terminal_rank_mismatches",
        "trace_authority_mismatches",
        "reverse_terminal_mismatches",
        "reverse_peak_mismatches",
        "reverse_completeness_mismatches",
        "reverse_work_mismatches",
        "reverse_rank_prefix_mismatches",
        "reverse_terminal_rank_mismatches",
        "full_row_terminal_mismatches",
        "full_row_peak_mismatches",
        "full_row_completeness_mismatches",
        "full_row_work_mismatches",
        "false_certificates",
    )
    need(value.get("verdict") == "PASS", "D7 parity subject did not PASS")
    need(value.get("records") == QUERIES and value.get("schedule_records") == QUERIES, "D7 parity record mismatch")
    for field in zero_fields:
        need(value.get(field) == 0, f"D7 parity mismatch: {field}")
    need(value.get("target_form_retained") == QUERIES, "D7 target form retention mismatch")
    need(value.get("target_lemma_retained") == QUERIES, "D7 target lemma retention mismatch")
    need(value.get("maximum_product_states") == 35_590, "D7 maximum states mismatch")
    need(value.get("e0_work") == value.get("d1_work"), "D7 E0/D1 work mismatch")
    need(value.get("d1_work", {}).get("examined_edges") == EDGES_PER_ROUND, "D7 parity edge denominator mismatch")
    need(value.get("stress", {}).get("cases") == 714_026, "D7 stress denominator mismatch")
    need(value.get("stress", {}).get("transition_mismatches") == 0, "D7 stress transition mismatch")
    need(value.get("stress", {}).get("packed_state_mismatches") == 0, "D7 stress packed mismatch")
    need(value.get("fixtures", {}).get("pass") is True, "D7 fixture invariant mismatch")


def run_parity_route(stage: pathlib.Path) -> dict[str, Any]:
    root = stage / "PARITY"
    subject = root / "subject"
    subject.mkdir(parents=True, mode=0o700)
    marker = consume_marker("parity")
    environment = subject_environment(subject, "PARITY", [0])
    command = child_as_e(environment, subject_command(PARITY_TEST))
    before = environment_snapshot()
    process = subprocess.Popen(
        command,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        start_new_session=True,
    )
    timeout_error = None
    try:
        stdout, stderr = process.communicate(timeout=3_600)
    except subprocess.TimeoutExpired:
        timeout_error = "parity timed out after 3600 seconds"
        terminate_owned(process)
        stdout, stderr = process.communicate()
    after = environment_snapshot()
    write_new_bytes(root / "stdout.log", stdout)
    write_new_bytes(root / "stderr.log", stderr)
    receipt_path = subject / "SUBJECT_RECEIPT.json"
    verdict = "PASS"
    detail = None
    try:
        need(timeout_error is None, timeout_error or "parity timeout")
        need(process.returncode == 0, f"parity exited {process.returncode}")
        need(receipt_path.is_file(), "parity receipt missing")
        validate_parity(load_json(receipt_path))
    except Exception as error:
        verdict = "BLOCKED_PARITY"
        detail = str(error)
    drift = thermal_drift(before, after)
    if verdict == "PASS" and drift:
        verdict = "BLOCKED_THERMAL"
        detail = f"parity thermal throttle counters changed: {drift}"
    wrapper = {
        "schema": "lay.v10.e1-traversal-d7-parity-wrapper.v1",
        "route": "PARITY",
        "verdict": verdict,
        "detail": detail,
        "marker": marker,
        "command": command,
        "exit_code": process.returncode,
        "subject_receipt": load_json(receipt_path) if receipt_path.is_file() else None,
        "subject_receipt_sha256": sha256_file(receipt_path) if receipt_path.is_file() else None,
        "environment_before": before,
        "environment_after": after,
        "thermal_throttle_drift": drift,
        "perf_invocations": 0,
        "runtime_authority_changed": False,
    }
    write_new_json(root / "PARITY_WRAPPER.json", wrapper)
    if verdict != "PASS":
        raise TerminalRouteError(verdict, "PARITY", detail or verdict)
    append_state(2, "PARITY_PASS", parity_wrapper_sha256=sha256_file(root / "PARITY_WRAPPER.json"))
    return wrapper


def parse_component_samples(path: pathlib.Path) -> list[dict[str, Any]]:
    raw = path.read_bytes()
    need(len(raw) == QUERIES * ROUNDS * COMPONENT_SAMPLE.size, "component sample byte denominator mismatch")
    samples = []
    for offset in range(0, len(raw), COMPONENT_SAMPLE.size):
        values = COMPONENT_SAMPLE.unpack_from(raw, offset)
        phases = []
        cursor = 6
        for phase in PHASES:
            phases.append(
                {
                    "phase": phase,
                    "wall_ns": values[cursor],
                    "thread_cpu_ns": values[cursor + 1],
                }
            )
            cursor += 2
        samples.append(
            {
                "query_ordinal": values[0],
                "round": values[1],
                "worker_id": values[2],
                "flags": values[3],
                "outer_wall_ns": values[4],
                "outer_thread_cpu_ns": values[5],
                "phases": phases,
            }
        )
    return samples


def expected_chunks(workers: int) -> list[tuple[int, int]]:
    chunk_size = math.ceil(QUERIES / workers)
    return [
        (worker * chunk_size, min((worker + 1) * chunk_size, QUERIES))
        for worker in range(workers)
    ]


def validate_component_route(
    route: str,
    receipt: dict[str, Any],
    samples: list[dict[str, Any]],
    structures: list[dict[str, Any]],
) -> dict[str, Any]:
    cpus = ROUTE_CPUS[route]
    chunks = expected_chunks(len(cpus))
    need(receipt.get("schema") == "lay.v10.e1-traversal-d7-worker-topology-subject.v1", "D7 subject schema mismatch")
    need(receipt.get("verdict") == "PASS" and receipt.get("route") == route, "D7 subject route mismatch")
    need(receipt.get("test") == SWEEP_TEST, "D7 subject test mismatch")
    need(receipt.get("queries") == QUERIES and receipt.get("rounds") == ROUNDS, "D7 subject denominator mismatch")
    need(receipt.get("workers") == len(cpus) and receipt.get("cpus") == list(cpus), "D7 subject CPU registry mismatch")
    need(receipt.get("warmup_bursts") == 1, "D7 warmup mismatch")
    need(receipt.get("start_barriers") == ROUNDS + 1 and receipt.get("end_barriers") == ROUNDS + 1, "D7 barrier count mismatch")
    expected_chunk_rows = [
        {"worker": worker, "start": start, "end": end, "queries": end - start}
        for worker, (start, end) in enumerate(chunks)
    ]
    need(receipt.get("worker_chunks") == expected_chunk_rows, "D7 worker chunks mismatch")
    need(receipt.get("worker_affinities") == [[cpu] for cpu in cpus], "D7 worker affinity mismatch")
    need(receipt.get("worker_migration_deltas") == [0] * len(cpus), "D7 worker migration detected")
    need(receipt.get("parent_affinity") == [cpus[0]], "D7 parent affinity mismatch")
    need(receipt.get("parent_migration_delta") == 0, "D7 parent migration detected")
    summary = receipt.get("samples", {})
    need(summary.get("sample_bytes") == COMPONENT_SAMPLE.size, "D7 sample width receipt mismatch")
    need(summary.get("samples") == QUERIES * ROUNDS, "D7 sample count receipt mismatch")
    need(summary.get("errors") == 0 and summary.get("unresolved") == 0, "D7 semantic error or unresolved")
    need(len(samples) == QUERIES * ROUNDS, "D7 parsed sample count mismatch")
    owner = {}
    for worker, (start, end) in enumerate(chunks):
        for query in range(start, end):
            owner[query] = worker
    need(set(owner) == set(range(QUERIES)), "D7 query partition is not complete")
    seen = set()
    for sample in samples:
        query = sample["query_ordinal"]
        round_id = sample["round"]
        need(0 <= query < QUERIES and 0 <= round_id < ROUNDS, "D7 sample coordinate out of range")
        need(sample["worker_id"] == owner[query], "D7 sample worker ownership mismatch")
        need(sample["flags"] == 0, "D7 sample error flag")
        coordinate = (query, round_id)
        need(coordinate not in seen, "D7 duplicate sample coordinate")
        seen.add(coordinate)
    need(len(seen) == QUERIES * ROUNDS, "D7 sample coordinate coverage mismatch")
    need(len(structures) == QUERIES, "D7 structure row count mismatch")
    need(sum(int(row.get("examined_edges", 0)) for row in structures) == EDGES_PER_ROUND, "D7 structure edge denominator mismatch")
    need(all(int(row.get("query_ordinal", -1)) == ordinal for ordinal, row in enumerate(structures)), "D7 structure ordinal mismatch")

    traversal_cpu_ns = sum(sample["phases"][3]["thread_cpu_ns"] for sample in samples)
    outer_cpu_ns = sum(sample["outer_thread_cpu_ns"] for sample in samples)
    phase_cpu_ns = {
        phase: sum(sample["phases"][index]["thread_cpu_ns"] for sample in samples)
        for index, phase in enumerate(PHASES)
    }
    measured_wall_ns = int(receipt.get("measured_wall_ns", 0))
    need(measured_wall_ns > 0 and traversal_cpu_ns > 0, "D7 timing denominator absent")
    return {
        "records": len(samples),
        "measured_edges": MEASURED_EDGES,
        "traversal_thread_cpu_ns": traversal_cpu_ns,
        "traversal_ns_per_edge": traversal_cpu_ns / MEASURED_EDGES,
        "outer_thread_cpu_ns": outer_cpu_ns,
        "outer_ns_per_edge": outer_cpu_ns / MEASURED_EDGES,
        "phase_thread_cpu_ns": phase_cpu_ns,
        "measured_wall_ns": measured_wall_ns,
        "aggregate_edges_per_second": MEASURED_EDGES * 1_000_000_000 / measured_wall_ns,
    }


def numeric_counter(value: Any) -> float | None:
    if isinstance(value, (int, float)):
        return float(value)
    if not isinstance(value, str):
        return None
    compact = value.strip().replace(",", "")
    if not compact or compact.startswith("<") or compact.lower() in {"not counted", "not supported"}:
        return None
    try:
        return float(compact)
    except ValueError:
        return None


def event_identity(event: str) -> tuple[str | None, str]:
    normalized = event.strip().lower().replace(":u", "")
    match = re.fullmatch(r"cpu_(core|atom)/([^/]+)/", normalized)
    if match is not None:
        return match.group(1), match.group(2)
    return None, normalized.strip("/")


def task_clock_ns(row: dict[str, Any], counter: float) -> float:
    unit = str(row.get("unit", "")).strip().lower()
    if unit in {"msec", "ms", "milliseconds"}:
        return counter * 1_000_000
    if unit in {"usec", "us", "microseconds"}:
        return counter * 1_000
    if unit in {"nsec", "ns", "nanoseconds"}:
        return counter
    if unit in {"sec", "s", "seconds"}:
        return counter * 1_000_000_000
    raise D7Error(f"unsupported task-clock unit: {unit!r}")


def parse_perf(raw: bytes, route: str) -> dict[str, Any]:
    rows = []
    diagnostics = []
    for line in raw.decode(errors="replace").splitlines():
        try:
            value = json.loads(line)
        except json.JSONDecodeError:
            if line.strip():
                diagnostics.append(line)
            continue
        if isinstance(value, dict) and isinstance(value.get("event"), str):
            rows.append(value)
        elif line.strip():
            diagnostics.append(line)
    need(rows, "perf produced no JSON rows")
    expected_pmus = {"core"} | ({"atom"} if any(cpu >= 12 for cpu in ROUTE_CPUS[route]) else set())
    counters = {}
    for expected in HARDWARE_EVENTS:
        matched = [row for row in rows if event_identity(row["event"])[1] == expected]
        need(len(matched) == 2, f"perf hybrid row count mismatch for {expected}: {len(matched)}")
        need(
            {event_identity(row["event"])[0] for row in matched} == {"core", "atom"},
            f"perf hybrid owner coverage mismatch for {expected}",
        )
        active = []
        inactive = []
        for row in matched:
            pmu, _ = event_identity(row["event"])
            need(pmu in {"core", "atom"}, f"hardware event lacks hybrid PMU: {row['event']}")
            counter = numeric_counter(row.get("counter-value"))
            runtime = numeric_counter(row.get("event-runtime"))
            running = numeric_counter(row.get("pcnt-running"))
            if counter is None or runtime is None or runtime <= 0:
                need(
                    str(row.get("counter-value", "")).strip().lower() == "<not counted>",
                    f"unsupported or malformed inactive PMU row: {row}",
                )
                need((runtime or 0) == 0, f"partial inactive PMU row: {row}")
                need((running or 0) == 0, f"inactive PMU row has running time: {row}")
                inactive.append({"pmu": pmu, "row": row})
            else:
                need(running is not None and running > 0, f"active PMU row lacks running percent: {row}")
                active.append({"pmu": pmu, "counter": counter, "runtime": runtime, "running": running, "row": row})
        need({item["pmu"] for item in active} == expected_pmus, f"{expected} active PMU coverage mismatch")
        need(len(active) == len(expected_pmus), f"{expected} duplicate active PMU row")
        need(not ({item["pmu"] for item in inactive} & expected_pmus), f"{expected} expected PMU is inactive")
        need(
            {item["pmu"] for item in inactive} == {"core", "atom"} - expected_pmus,
            f"{expected} inactive PMU coverage mismatch",
        )
        runtime_sum = sum(item["runtime"] for item in active)
        need(runtime_sum > 0, f"{expected} runtime sum absent")
        running_sum = sum(item["running"] for item in active)
        need(98.9 <= running_sum <= 101.1, f"{expected} running-percent partition mismatch")
        weighted = 0.0
        parts = []
        for item in active:
            share = item["runtime"] / runtime_sum
            need(abs(item["running"] - share * 100.0) <= 1.1, f"{expected} hybrid runtime percentage mismatch")
            contribution = item["counter"] * share
            weighted += contribution
            parts.append({**item, "runtime_share": share, "weighted_contribution": contribution})
        counters[expected] = {
            "runtime_weighted_value": weighted,
            "runtime_sum_ns": runtime_sum,
            "running_percent_sum": running_sum,
            "active_pmus": sorted(expected_pmus),
            "parts": parts,
            "inactive_rows": inactive,
        }
    task_rows = [row for row in rows if event_identity(row["event"])[1] == "task-clock"]
    need(len(task_rows) == 1, f"task-clock row count mismatch: {len(task_rows)}")
    need(event_identity(task_rows[0]["event"])[0] is None, "task-clock unexpectedly PMU-qualified")
    numeric_task = [(row, numeric_counter(row.get("counter-value"))) for row in task_rows]
    numeric_task = [(row, value) for row, value in numeric_task if value is not None]
    need(len(numeric_task) == 1, f"task-clock numeric row count mismatch: {len(numeric_task)}")
    task_row, task_value = numeric_task[0]
    task_runtime = numeric_counter(task_row.get("event-runtime"))
    task_running = numeric_counter(task_row.get("pcnt-running"))
    need(task_runtime is not None and task_runtime > 0, "task-clock event runtime absent")
    need(task_running is not None and abs(task_running - 100.0) <= 0.01, "task-clock scaled")
    task_ns = task_clock_ns(task_row, task_value)
    need(task_ns > 0, "task-clock denominator absent")
    instructions = counters["instructions"]["runtime_weighted_value"]
    cycles = counters["cycles"]["runtime_weighted_value"]
    branches = counters["branches"]["runtime_weighted_value"]
    branch_misses = counters["branch-misses"]["runtime_weighted_value"]
    need(instructions > 0 and cycles > 0 and branches > 0, "perf physical denominator absent")
    return {
        "rows": rows,
        "diagnostics": diagnostics,
        "counters": counters,
        "task_clock": {
            "value_ns": task_ns,
            "event_runtime": task_runtime,
            "pcnt_running": task_running,
            "row": task_row,
        },
        "derived": {
            "instructions_per_edge": instructions / MEASURED_EDGES,
            "cycles_per_edge": cycles / MEASURED_EDGES,
            "branches_per_edge": branches / MEASURED_EDGES,
            "branch_miss_rate": branch_misses / branches,
            "ipc": instructions / cycles,
            "effective_frequency_ghz": cycles / task_ns,
            "task_clock_ns_per_edge": task_ns / MEASURED_EDGES,
        },
    }


def run_worker_route(stage: pathlib.Path, route: str, sequence: int) -> dict[str, Any]:
    cpus = ROUTE_CPUS[route]
    root = stage / route
    subject = root / "subject"
    control = root / "control"
    subject.mkdir(parents=True, mode=0o700)
    control.mkdir(mode=0o700)
    control_fifo = root / "perf-control.fifo"
    ack_fifo = root / "perf-ack.fifo"
    os.mkfifo(control_fifo, 0o600)
    os.mkfifo(ack_fifo, 0o600)
    marker = consume_marker(route)
    environment = subject_environment(subject, route, cpus)
    environment["LAY_V10_D7_ROUTE"] = route
    environment["LAY_V10_D7_CONTROL_DIR"] = str(control)
    child = child_as_e(environment, subject_command(SWEEP_TEST))
    command = [
        "/usr/bin/sudo",
        "-n",
        "/usr/bin/perf",
        "stat",
        "--json-output",
        "--no-big-num",
        "--delay=-1",
        f"--control=fifo:{control_fifo},{ack_fifo}",
        "--event",
        ",".join(ALL_EVENTS),
        "--",
        *child,
    ]
    before = environment_snapshot()
    write_new_json(root / "OBSERVATION_PLAN.json", {
        "route": route,
        "cpus": list(cpus),
        "events": list(ALL_EVENTS),
        "command": command,
        "marker": marker,
        "perf_stat_invocations": 1,
        "retry_permitted": False,
    })
    process: subprocess.Popen[bytes] | None = None
    control_fd: int | None = None
    ack_fd: int | None = None
    stdout = b""
    stderr = b""
    enable_ack: str | None = None
    disable_ack: str | None = None
    controller_measured_wall_ns: int | None = None
    try:
        process = subprocess.Popen(
            command,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            start_new_session=True,
        )
        wait_for_file(process, control / "subject-ready", 3_600.0)
        deadline = time.monotonic() + 15.0
        control_fd = open_fifo(control_fifo, os.O_WRONLY, deadline)
        ack_fd = open_fifo(ack_fifo, os.O_RDONLY, deadline)
        os.write(control_fd, b"enable\n")
        enable_ack = read_fifo_line(ack_fd, time.monotonic() + 15.0)
        write_new_bytes(control / "controller-enabled", b"enabled\n")
        measured_started = time.perf_counter_ns()
        wait_for_file(process, control / "subject-done", 1_800.0)
        controller_measured_wall_ns = time.perf_counter_ns() - measured_started
        os.write(control_fd, b"disable\n")
        disable_ack = read_fifo_line(ack_fd, time.monotonic() + 15.0)
        write_new_bytes(control / "controller-disabled", b"disabled\n")
        stdout, stderr = process.communicate(timeout=300)
    except BaseException as error:
        terminate_owned(process)
        if process is not None and process.stdout is not None and process.stderr is not None:
            with contextlib.suppress(Exception):
                stdout, stderr = process.communicate(timeout=1)
        write_new_bytes(root / "stdout.log", stdout)
        write_new_bytes(root / "perf.raw", stderr)
        failure = {
            "schema": "lay.v10.e1-traversal-d7-route-failure.v1",
            "route": route,
            "verdict": "BLOCKED_PROVENANCE",
            "error": str(error),
            "marker": marker,
            "command": command,
            "observation_complete": False,
            "retry_permitted": False,
            "runtime_authority_changed": False,
        }
        write_new_json(root / "ROUTE_FAILURE.json", failure)
        raise TerminalRouteError("BLOCKED_PROVENANCE", route, str(error)) from error
    finally:
        if control_fd is not None:
            os.close(control_fd)
        if ack_fd is not None:
            os.close(ack_fd)
        with contextlib.suppress(FileNotFoundError):
            control_fifo.unlink()
        with contextlib.suppress(FileNotFoundError):
            ack_fifo.unlink()

    write_new_bytes(root / "stdout.log", stdout)
    write_new_bytes(root / "perf.raw", stderr)
    after = environment_snapshot()
    receipt_path = subject / "SUBJECT_RECEIPT.json"
    samples_path = subject / "component-samples.bin"
    structure_path = subject / "structure.json"
    provenance_errors = []
    capability_errors = []
    measurement_errors = []
    thermal_errors = []
    receipt: dict[str, Any] | None = None
    statistics: dict[str, Any] | None = None
    perf: dict[str, Any] | None = None
    required_outputs = (receipt_path, samples_path, structure_path)
    if not all(path.is_file() for path in required_outputs):
        provenance_errors.append("subject output set incomplete")
    else:
        try:
            receipt = load_json(receipt_path)
            samples = parse_component_samples(samples_path)
            structures = load_json(structure_path).get("queries", [])
            statistics = validate_component_route(route, receipt, samples, structures)
        except Exception as error:
            measurement_errors.append(str(error))
    if process is None or process.returncode != 0:
        if receipt is not None and receipt.get("error"):
            measurement_errors.append(f"subject exited {process.returncode if process else None}: {receipt['error']}")
        else:
            capability_errors.append(f"perf/subject process exited {process.returncode if process else None}")
    if not (isinstance(enable_ack, str) and enable_ack.startswith("ack")):
        capability_errors.append(f"perf enable acknowledgement invalid: {enable_ack!r}")
    if not (isinstance(disable_ack, str) and disable_ack.startswith("ack")):
        capability_errors.append(f"perf disable acknowledgement invalid: {disable_ack!r}")
    try:
        perf = parse_perf(stderr, route)
    except Exception as error:
        capability_errors.append(str(error))
    drift = thermal_drift(before, after)
    if drift:
        thermal_errors.append(f"thermal throttle counters changed: {drift}")
    if statistics is not None and route == "W1":
        delta = abs(statistics["traversal_ns_per_edge"] - W1_BASELINE_NS_PER_EDGE) / W1_BASELINE_NS_PER_EDGE
        if delta > 0.05:
            measurement_errors.append(f"W1 D1 baseline delta {delta:.9%} exceeds 5%")
    if statistics is not None and route == "W20":
        delta = abs(statistics["traversal_ns_per_edge"] - W20_BASELINE_NS_PER_EDGE) / W20_BASELINE_NS_PER_EDGE
        if delta > 0.05:
            measurement_errors.append(f"W20 D1 baseline delta {delta:.9%} exceeds 5%")

    if provenance_errors:
        verdict = "BLOCKED_PROVENANCE"
        failures = provenance_errors
    elif thermal_errors:
        verdict = "BLOCKED_THERMAL"
        failures = thermal_errors
    elif capability_errors:
        verdict = "BLOCKED_CAPABILITY"
        failures = capability_errors
    elif measurement_errors:
        verdict = "BLOCKED_MEASUREMENT"
        failures = measurement_errors
    else:
        verdict = "PASS"
        failures = []
    wrapper = {
        "schema": "lay.v10.e1-traversal-d7-worker-route-wrapper.v1",
        "route": route,
        "verdict": verdict,
        "failures": failures,
        "all_failure_sets": {
            "provenance": provenance_errors,
            "thermal": thermal_errors,
            "capability": capability_errors,
            "measurement": measurement_errors,
        },
        "marker": marker,
        "command": command,
        "environment": {key: environment[key] for key in sorted(environment)},
        "exit_code": process.returncode if process is not None else None,
        "enable_ack": enable_ack,
        "disable_ack": disable_ack,
        "controller_measured_wall_ns": controller_measured_wall_ns,
        "subject_receipt": receipt,
        "subject_receipt_sha256": sha256_file(receipt_path) if receipt_path.is_file() else None,
        "samples_sha256": sha256_file(samples_path) if samples_path.is_file() else None,
        "structure_sha256": sha256_file(structure_path) if structure_path.is_file() else None,
        "statistics": statistics,
        "perf": perf,
        "perf_raw_sha256": sha256_file(root / "perf.raw"),
        "environment_before": before,
        "environment_after": after,
        "thermal_throttle_drift": drift,
        "perf_stat_invocations": 1,
        "perf_record_invocations": 0,
        "runtime_authority_changed": False,
    }
    write_new_json(root / "ROUTE_WRAPPER.json", wrapper)
    if verdict != "PASS":
        raise TerminalRouteError(verdict, route, "; ".join(failures))
    append_state(
        sequence,
        f"{route}_PASS",
        route_wrapper_sha256=sha256_file(root / "ROUTE_WRAPPER.json"),
    )
    return wrapper


def calculate_frontiers(routes: dict[str, dict[str, Any]]) -> dict[str, Any]:
    w1 = routes["W1"]["statistics"]["traversal_ns_per_edge"]
    points = []
    for route in ROUTE_ORDER:
        wrapper = routes[route]
        statistics = wrapper["statistics"]
        perf = wrapper["perf"]["derived"]
        point = {
            "route": route,
            "workers": len(ROUTE_CPUS[route]),
            "cpus": list(ROUTE_CPUS[route]),
            "traversal_ns_per_edge": statistics["traversal_ns_per_edge"],
            "delta_from_W1_ns_per_edge": statistics["traversal_ns_per_edge"] - w1,
            "inflation_from_W1_percent": (statistics["traversal_ns_per_edge"] / w1 - 1.0) * 100.0,
            "aggregate_edges_per_second": statistics["aggregate_edges_per_second"],
            "throughput_scaling_from_W1": statistics["aggregate_edges_per_second"]
            / routes["W1"]["statistics"]["aggregate_edges_per_second"],
            "parallel_efficiency_from_W1": (
                statistics["aggregate_edges_per_second"]
                / routes["W1"]["statistics"]["aggregate_edges_per_second"]
                / len(ROUTE_CPUS[route])
            ),
            **perf,
        }
        points.append(point)
    latency_candidates = [point for point in points if point["traversal_ns_per_edge"] <= w1 * 1.05]
    latency_capacity = max(latency_candidates, key=lambda point: point["workers"]) if latency_candidates else None
    throughput_point = max(points, key=lambda point: point["aggregate_edges_per_second"])
    pareto = []
    for point in points:
        dominated = any(
            other["route"] != point["route"]
            and other["traversal_ns_per_edge"] <= point["traversal_ns_per_edge"]
            and other["aggregate_edges_per_second"] >= point["aggregate_edges_per_second"]
            and (
                other["traversal_ns_per_edge"] < point["traversal_ns_per_edge"]
                or other["aggregate_edges_per_second"] > point["aggregate_edges_per_second"]
            )
            for other in points
        )
        if not dominated:
            pareto.append(point["route"])
    by_route = {point["route"]: point for point in points}

    def delta(right: str, left: str) -> dict[str, float]:
        return {
            "traversal_ns_per_edge": by_route[right]["traversal_ns_per_edge"]
            - by_route[left]["traversal_ns_per_edge"],
            "ipc": by_route[right]["ipc"] - by_route[left]["ipc"],
            "effective_frequency_ghz": by_route[right]["effective_frequency_ghz"]
            - by_route[left]["effective_frequency_ghz"],
            "instructions_per_edge": by_route[right]["instructions_per_edge"]
            - by_route[left]["instructions_per_edge"],
        }

    return {
        "schema": "lay.v10.e1-traversal-d7-frontiers.v1",
        "points": points,
        "latency_preserving_capacity": latency_capacity,
        "throughput_point": throughput_point,
        "pareto_routes": pareto,
        "topology_interventions": {
            "W1_to_W6_package_and_all_P_cores": delta("W6", "W1"),
            "W6_to_W12_P_core_SMT": delta("W12", "W6"),
            "W6_to_W14_add_E_cores_without_P_SMT": delta("W14", "W6"),
            "W14_to_W20_add_P_SMT_at_full_physical_saturation": delta("W20", "W14"),
        },
        "full_W20_minus_W1_ns_per_edge": by_route["W20"]["traversal_ns_per_edge"]
        - by_route["W1"]["traversal_ns_per_edge"],
        "historical_target_delta_ns_per_edge": 18.770001603849174,
        "production_policy_admitted": False,
    }


def publish_remote_result(
    stage: pathlib.Path,
    *,
    verdict: str,
    build: dict[str, Any] | None,
    parity: dict[str, Any] | None,
    routes: dict[str, dict[str, Any]],
    failure: dict[str, Any] | None,
    runtime_before: dict[str, Any],
    runtime_after: dict[str, Any],
) -> dict[str, Any]:
    frontiers = (
        calculate_frontiers(routes)
        if verdict == "D7_WORKER_TOPOLOGY_SWEEP_CREATED_UNAUDITED"
        else None
    )
    decision = {
        "schema": "lay.v10.e1-traversal-d7-decision.v1",
        "task_id": TASK_ID,
        "transaction_id": TRANSACTION_ID,
        "verdict": verdict,
        "failure": failure,
        "build": build,
        "parity": parity,
        "routes": routes,
        "frontiers": frontiers,
        "markers": marker_inventory(),
        "remote_runtime_before": runtime_before,
        "remote_runtime_after": runtime_after,
        "execution_ledger": {
            "cargo_invocations": 1 if build is not None else 0,
            "subject_executions": (1 if parity is not None else 0) + len(routes),
            "perf_stat_invocations": len(routes),
            "perf_record_invocations": 0,
            "installed_elf_executions": 0,
            "runtime_authority_changed": False,
        },
        "claim_boundary": {
            "diagnostic_only": True,
            "production_concurrency_established": False,
            "production_policy_admitted": False,
            "SWAR_or_decoder_rewrite_admitted": False,
            "integration_or_deployment_admitted": False,
            "runtime_authority_changed": False,
            "retry_permitted": False,
        },
    }
    write_new_json(stage / "D7_DECISION.json", decision)
    write_new_json(
        stage / "INPUT_IDENTITIES.json",
        {
            "paper_sha256": EXPECTED["paper"],
            "preflight_sha256": EXPECTED["preflight"],
            "preflight_receipt_sha256": EXPECTED["preflight_receipt"],
            "d6_receipt_sha256": EXPECTED["d6_receipt"],
            "d1_decision_sha256": EXPECTED["d1_decision"],
            "e1_decision_sha256": EXPECTED["e1_decision"],
            "controller_sha256": sha256_file(CONTROLLER),
            "fragment_sha256": EXPECTED["fragment"],
            "assembled_source_sha256": EXPECTED["assembled_source"],
        },
    )
    write_sha256sums(stage)
    seal_tree(stage)
    os.rename(stage, REMOTE_RESULT)
    fsync_directory(REMOTE_PARENT)
    return decision


def remote_run(bootstrap: pathlib.Path) -> dict[str, Any]:
    host = remote_machine_identity()
    topology = remote_topology()
    need(not REMOTE_PARENT.exists() and not REMOTE_STATE.exists(), "D7 namespace already exists")
    bootstrap_rows = validate_bootstrap(bootstrap)
    runtime_before = remote_runtime_snapshot()
    initialize_remote_state()
    result_stage = REMOTE_PARENT / f"result-v1.stage-{os.getpid()}-{time.time_ns()}"
    result_stage.mkdir(mode=0o700)
    build: dict[str, Any] | None = None
    parity: dict[str, Any] | None = None
    routes: dict[str, dict[str, Any]] = {}
    with remote_lock():
        try:
            write_new_json(
                result_stage / "BOOTSTRAP.json",
                {
                    "host": host,
                    "topology": topology,
                    "bootstrap": bootstrap_rows,
                    "markers": marker_inventory(),
                    "controller_sha256": sha256_file(CONTROLLER),
                },
            )
            build = remote_build_once(bootstrap)
            parity = run_parity_route(result_stage)
            for sequence, route in enumerate(ROUTE_ORDER, start=3):
                routes[route] = run_worker_route(result_stage, route, sequence)
            need(not active_d7_processes(), "owned D7 subject remains active after routes")
            runtime_after = remote_runtime_snapshot()
            need(runtime_before == runtime_after, "remote installed Lay authority changed during D7")
            decision = publish_remote_result(
                result_stage,
                verdict="D7_WORKER_TOPOLOGY_SWEEP_CREATED_UNAUDITED",
                build=build,
                parity=parity,
                routes=routes,
                failure=None,
                runtime_before=runtime_before,
                runtime_after=runtime_after,
            )
            seal_remote_state()
            return decision
        except TerminalRouteError as error:
            runtime_after = remote_runtime_snapshot()
            verdict = error.verdict
            detail = error.detail
            if runtime_before != runtime_after:
                verdict = "BLOCKED_PROVENANCE"
                detail = f"{detail}; remote installed Lay authority changed during D7"
            failure = {"route": error.route, "verdict": verdict, "detail": detail}
            with contextlib.suppress(Exception):
                write_new_json(result_stage / "TERMINAL_FAILURE.json", failure)
            decision = publish_remote_result(
                result_stage,
                verdict=verdict,
                build=build,
                parity=parity,
                routes=routes,
                failure=failure,
                runtime_before=runtime_before,
                runtime_after=runtime_after,
            )
            with contextlib.suppress(Exception):
                append_state(8, verdict, failure=failure, result_sha256=sha256_file(REMOTE_RESULT / "D7_DECISION.json"))
                seal_remote_state()
            return decision
        except Exception as error:
            runtime_after = remote_runtime_snapshot()
            failure = {"route": "CONTROLLER", "verdict": "BLOCKED_PROVENANCE", "detail": str(error)}
            with contextlib.suppress(Exception):
                write_new_json(result_stage / "TERMINAL_FAILURE.json", failure)
            decision = publish_remote_result(
                result_stage,
                verdict="BLOCKED_PROVENANCE",
                build=build,
                parity=parity,
                routes=routes,
                failure=failure,
                runtime_before=runtime_before,
                runtime_after=runtime_after,
            )
            with contextlib.suppress(Exception):
                append_state(8, "BLOCKED_PROVENANCE", failure=failure, result_sha256=sha256_file(REMOTE_RESULT / "D7_DECISION.json"))
                seal_remote_state()
            return decision


def remote_status() -> dict[str, Any]:
    host = remote_machine_identity()
    topology = remote_topology()
    inventory = marker_inventory()
    marker_rows: dict[str, Any] = {}
    markers = REMOTE_STATE / "markers"
    for route in MARKER_ROUTES:
        candidates = [
            markers / marker_name(route, "available"),
            markers / marker_name(route, "consumed-before-exec"),
        ]
        present = [path for path in candidates if path.is_file()]
        if present:
            need(len(present) == 1, f"duplicate marker states for {route}")
            path = present[0]
            marker_rows[route] = {
                **file_identity(path),
                "payload_exact": path.read_bytes() == marker_payload(route),
            }
    state_files = []
    if REMOTE_STATE.is_dir():
        state_files = [
            file_identity(path)
            for path in sorted(REMOTE_STATE.glob("STATE-*.json"))
            if path.is_file()
        ]
    build_manifest_files = verify_sha256sums(REMOTE_BUILD) if REMOTE_BUILD.is_dir() else None
    build_failure = REMOTE_PARENT / "build-failure-v1"
    build_failure_manifest_files = verify_sha256sums(build_failure) if build_failure.is_dir() else None
    result_manifest_files = verify_sha256sums(REMOTE_RESULT) if REMOTE_RESULT.is_dir() else None
    decision = load_json(REMOTE_RESULT / "D7_DECISION.json") if (REMOTE_RESULT / "D7_DECISION.json").is_file() else None
    return {
        "schema": "lay.v10.e1-traversal-d7-remote-status.v1",
        "task_id": TASK_ID,
        "transaction_id": TRANSACTION_ID,
        "host": host,
        "topology": topology,
        "parent_exists": REMOTE_PARENT.is_dir(),
        "state_exists": REMOTE_STATE.is_dir(),
        "build_exists": REMOTE_BUILD.is_dir(),
        "build_failure_exists": build_failure.is_dir(),
        "result_exists": REMOTE_RESULT.is_dir(),
        "inventory": inventory,
        "marker_rows": marker_rows,
        "route_lock": file_identity(REMOTE_STATE / "route.lock") if (REMOTE_STATE / "route.lock").is_file() else None,
        "state_files": state_files,
        "build_manifest_files_verified": build_manifest_files,
        "build_failure_manifest_files_verified": build_failure_manifest_files,
        "result_manifest_files_verified": result_manifest_files,
        "decision_sha256": sha256_file(REMOTE_RESULT / "D7_DECISION.json") if decision is not None else None,
        "decision_verdict": decision.get("verdict") if decision is not None else None,
        "active_d7_processes": active_d7_processes(),
        "runtime": remote_runtime_snapshot(),
        "remote_writes": 0,
    }


def parse_last_json(raw: bytes, label: str) -> dict[str, Any]:
    for line in reversed(raw.decode(errors="replace").splitlines()):
        with contextlib.suppress(json.JSONDecodeError):
            value = json.loads(line)
            if isinstance(value, dict):
                return value
    raise D7Error(f"{label} returned no JSON object")


def upload_bootstrap() -> pathlib.Path:
    created = ssh(["mktemp", "-d", "/tmp/lay-d7.XXXXXX"]).stdout.decode().strip()
    root = pathlib.Path(created)
    need(root.parent == pathlib.Path("/tmp") and root.name.startswith("lay-d7."), "unsafe remote bootstrap path")
    sources = {
        "controller.py": CONTROLLER,
        "fragment.inc": FRAGMENT,
        "v13_typed_peak.v10.rs": V10_SOURCE,
        "paper.md": PAPER,
        "structural-review.json": STRUCTURAL_REVIEW,
        "preflight-v2.json": PREFLIGHT,
        "preflight-v2-receipt.json": PREFLIGHT_RECEIPT,
        "d6-receipt.json": D6_RECEIPT,
        "d1-decision.json": D1_DECISION,
        "e1-decision.json": E1_DECISION,
    }
    try:
        for name, source in sources.items():
            scp_to_remote(source, root / name)
        ssh(["chmod", "0555", str(root / "controller.py")])
        return root
    except Exception:
        with contextlib.suppress(Exception):
            cleanup_remote_bootstrap(root)
        raise


def cleanup_remote_bootstrap(root: pathlib.Path) -> None:
    need(root.parent == pathlib.Path("/tmp") and root.name.startswith("lay-d7."), "unsafe bootstrap cleanup path")
    code = (
        "import pathlib,shutil,sys;"
        "p=pathlib.Path(sys.argv[1]);"
        "assert p.parent==pathlib.Path('/tmp') and p.name.startswith('lay-d7.');"
        "shutil.rmtree(p)"
    )
    ssh(["python3", "-c", code, str(root)])


def remote_controller_call(
    bootstrap: pathlib.Path,
    action: str,
    *,
    timeout: float | None = None,
) -> dict[str, Any]:
    command = ["python3", str(bootstrap / "controller.py"), action]
    if action in {"remote-probe", "remote-run"}:
        command.append(str(bootstrap))
    result = ssh(command, check=False, timeout=timeout)
    value = parse_last_json(result.stdout, action)
    if result.returncode != 0:
        detail = result.stderr.decode(errors="replace")[-5000:]
        raise D7Error(f"remote {action} failed ({result.returncode}): {detail}")
    return value


def copy_local_input(stage: pathlib.Path, name: str, source: pathlib.Path) -> None:
    write_new_bytes(stage / name, source.read_bytes())


def mirror_remote_evidence(
    stage: pathlib.Path,
    *,
    self_check: dict[str, Any],
    remote_response: dict[str, Any],
    remote_after: dict[str, Any],
    runtime_before: dict[str, Any],
    runtime_after: dict[str, Any],
) -> None:
    scp_from_remote(REMOTE_RESULT, stage / "REMOTE_RESULT", recursive=True)
    if remote_after.get("build_exists"):
        metadata = stage / "REMOTE_BUILD_METADATA"
        metadata.mkdir(mode=0o700)
        for name in (
            "PREBUILD.json",
            "BUILD_MARKER_CONSUMED.json",
            "BUILD_INVOCATION.json",
            "BUILD_PROVENANCE.json",
            "SHA256SUMS",
        ):
            scp_from_remote(REMOTE_BUILD / name, metadata / name)
    elif remote_after.get("build_failure_exists"):
        scp_from_remote(
            REMOTE_PARENT / "build-failure-v1",
            stage / "REMOTE_BUILD_FAILURE",
            recursive=True,
        )
    write_new_json(stage / "SELF_CHECK.json", self_check)
    write_new_json(stage / "REMOTE_RESPONSE.json", remote_response)
    write_new_json(stage / "REMOTE_STATUS_AFTER.json", remote_after)
    write_new_json(stage / "LOCAL_RUNTIME_BEFORE.json", runtime_before)
    write_new_json(stage / "LOCAL_RUNTIME_AFTER.json", runtime_after)
    copy_local_input(stage, "controller.py", CONTROLLER)
    copy_local_input(stage, "terminal-auditor.py", TERMINAL_AUDITOR)
    copy_local_input(stage, "fragment.inc", FRAGMENT)
    copy_local_input(stage, "paper.md", PAPER)
    copy_local_input(stage, "preflight-v2.json", PREFLIGHT)
    copy_local_input(stage, "preflight-v2-receipt.json", PREFLIGHT_RECEIPT)


def run_independent_terminal_audit(stage: pathlib.Path) -> dict[str, Any]:
    result = run_command(
        [sys.executable, str(TERMINAL_AUDITOR), "--stage", str(stage)],
        check=False,
        timeout=3_600,
    )
    need(result.returncode == 0, "independent D7 terminal audit failed:\n" + result.stderr.decode(errors="replace")[-5000:])
    return parse_last_json(result.stdout, "independent terminal audit")


def local_status() -> dict[str, Any]:
    result = {
        "schema": "lay.v10.e1-traversal-d7-local-status.v1",
        "task_id": TASK_ID,
        "transaction_id": TRANSACTION_ID,
        "local_result_exists": LOCAL_RESULT.is_dir(),
        "local_result_manifest_files_verified": verify_sha256sums(LOCAL_RESULT) if LOCAL_RESULT.is_dir() else None,
        "local_runtime": local_runtime_snapshot(),
    }
    return result


def seal_implementation_self_check() -> dict[str, Any]:
    need(not IMPLEMENTATION_RECEIPT.exists(), "D7 implementation self-check receipt already exists")
    check = local_self_check(emit=False)
    runtime_before = local_runtime_snapshot()
    bootstrap = upload_bootstrap()
    try:
        status = remote_controller_call(bootstrap, "remote-status", timeout=600)
        need(
            not status["parent_exists"]
            and not status["state_exists"]
            and not status["build_exists"]
            and not status["result_exists"]
            and not status["active_d7_processes"],
            "D7 remote state exists before implementation seal",
        )
        probe = remote_controller_call(bootstrap, "remote-probe", timeout=3_600)
        need(probe.get("verdict") == "D7_PRE_MARKER_PROBE_PASS", "D7 pre-marker probe did not PASS")
    finally:
        cleanup_remote_bootstrap(bootstrap)
    runtime_after = local_runtime_snapshot()
    need(runtime_before == runtime_after, "local runtime changed during implementation self-check")
    receipt = {
        "schema": "lay.v10.e1-traversal-d7-implementation-self-check.v1",
        "task_id": TASK_ID,
        "transaction_id": TRANSACTION_ID,
        "verdict": "D7_IMPLEMENTATION_VERIFIED_UNRUN",
        "controller_sha256": sha256_file(CONTROLLER),
        "controller_mode": mode_string(CONTROLLER),
        "terminal_auditor_sha256": sha256_file(TERMINAL_AUDITOR),
        "terminal_auditor_mode": mode_string(TERMINAL_AUDITOR),
        "fragment_sha256": EXPECTED["fragment"],
        "fragment_mode": mode_string(FRAGMENT),
        "assembled_source_sha256": EXPECTED["assembled_source"],
        "production_prefix_sha256": EXPECTED["production_prefix"],
        "self_check": check,
        "remote_status": status,
        "remote_pre_marker_probe": probe,
        "runtime_before": runtime_before,
        "runtime_after": runtime_after,
        "markers_created": 0,
        "markers_consumed": 0,
        "cargo_invocations": 0,
        "perf_stat_invocations": 0,
        "perf_record_invocations": 0,
        "subject_executions": 0,
        "runtime_authority_changed": False,
        "next_action_admitted": "one D7 run through this exact controller; no retry",
    }
    write_new_json(IMPLEMENTATION_RECEIPT, receipt)
    return {
        **receipt,
        "receipt_path": str(IMPLEMENTATION_RECEIPT),
        "receipt_sha256": sha256_file(IMPLEMENTATION_RECEIPT),
        "receipt_mode": mode_string(IMPLEMENTATION_RECEIPT),
    }


def verify_implementation_receipt() -> dict[str, Any]:
    identity = require_file(IMPLEMENTATION_RECEIPT, mode="0444")
    value = load_json(IMPLEMENTATION_RECEIPT)
    need(value.get("verdict") == "D7_IMPLEMENTATION_VERIFIED_UNRUN", "D7 implementation receipt verdict drift")
    need(value.get("task_id") == TASK_ID and value.get("transaction_id") == TRANSACTION_ID, "D7 implementation receipt namespace drift")
    need(value.get("controller_sha256") == sha256_file(CONTROLLER), "D7 controller changed after implementation seal")
    need(value.get("terminal_auditor_sha256") == sha256_file(TERMINAL_AUDITOR), "D7 auditor changed after implementation seal")
    need(value.get("fragment_sha256") == sha256_file(FRAGMENT), "D7 fragment changed after implementation seal")
    need(
        value.get("markers_created") == 0
        and value.get("markers_consumed") == 0
        and value.get("cargo_invocations") == 0
        and value.get("perf_stat_invocations") == 0
        and value.get("subject_executions") == 0,
        "D7 implementation receipt execution ledger drift",
    )
    return {**identity, "value": value}


def local_run() -> dict[str, Any]:
    need(not LOCAL_RESULT.exists(), "local D7 result already exists")
    verify_implementation_receipt()
    check = local_self_check(emit=False)
    runtime_before = local_runtime_snapshot()
    bootstrap = upload_bootstrap()
    stage = LOCAL_RESULT.with_name(f"{LOCAL_RESULT.name}.stage-{os.getpid()}-{time.time_ns()}")
    stage.mkdir(parents=True, mode=0o700)
    published = False
    try:
        initial = remote_controller_call(bootstrap, "remote-status", timeout=600)
        need(
            not initial["parent_exists"]
            and not initial["state_exists"]
            and not initial["build_exists"]
            and not initial["result_exists"]
            and not initial["active_d7_processes"],
            "D7 remote namespace or process exists before one-shot run",
        )
        probe = remote_controller_call(bootstrap, "remote-probe", timeout=3_600)
        need(probe.get("verdict") == "D7_PRE_MARKER_PROBE_PASS", "D7 pre-marker probe did not PASS")
        remote_response = remote_controller_call(bootstrap, "remote-run", timeout=21_600)
        remote_after = remote_controller_call(bootstrap, "remote-status", timeout=3_600)
        need(remote_after.get("result_exists") is True, "D7 remote result was not published")
        need(not remote_after.get("active_d7_processes"), "D7 process remains active after publication")
        runtime_after = local_runtime_snapshot()
        mirror_remote_evidence(
            stage,
            self_check=check,
            remote_response=remote_response,
            remote_after=remote_after,
            runtime_before=runtime_before,
            runtime_after=runtime_after,
        )
        write_new_json(stage / "REMOTE_STATUS_BEFORE.json", initial)
        write_new_json(stage / "REMOTE_PRE_MARKER_PROBE.json", probe)
        copy_local_input(stage, "implementation-self-check.json", IMPLEMENTATION_RECEIPT)
        audit = run_independent_terminal_audit(stage)
        write_new_json(stage / "D7_TERMINAL_AUDIT.json", audit)
        write_sha256sums(stage)
        seal_tree(stage)
        os.rename(stage, LOCAL_RESULT)
        fsync_directory(LOCAL_RESULT.parent)
        published = True
        cleanup_remote_bootstrap(bootstrap)
        return {
            **audit,
            "local_result": str(LOCAL_RESULT),
            "local_result_manifest_files": verify_sha256sums(LOCAL_RESULT),
            "local_result_sha256sums_sha256": sha256_file(LOCAL_RESULT / "SHA256SUMS"),
        }
    finally:
        if not published:
            remove_owned_tree(stage)


def parser() -> argparse.ArgumentParser:
    value = argparse.ArgumentParser()
    value.add_argument(
        "action",
        choices=("self-check", "seal-self-check", "status", "run", "remote-status", "remote-probe", "remote-run"),
    )
    value.add_argument("bootstrap", nargs="?")
    return value


def main() -> int:
    arguments = parser().parse_args()
    try:
        if arguments.action == "self-check":
            need(not REMOTE_BOOTSTRAP, "self-check is local-only")
            value = local_self_check(emit=False)
        elif arguments.action == "seal-self-check":
            need(not REMOTE_BOOTSTRAP, "seal-self-check is local-only")
            value = seal_implementation_self_check()
        elif arguments.action == "status":
            need(not REMOTE_BOOTSTRAP, "status is local-only")
            value = local_status()
        elif arguments.action == "run":
            need(not REMOTE_BOOTSTRAP, "run is local-only")
            value = local_run()
        elif arguments.action == "remote-status":
            need(REMOTE_BOOTSTRAP, "remote-status requires bootstrap controller name")
            value = remote_status()
        elif arguments.action == "remote-probe":
            need(REMOTE_BOOTSTRAP, "remote-probe requires bootstrap controller name")
            need(arguments.bootstrap is not None, "remote-probe bootstrap path missing")
            bootstrap = pathlib.Path(arguments.bootstrap).resolve()
            need(bootstrap == CONTROLLER.parent, "remote-probe bootstrap/controller mismatch")
            value = remote_pre_marker_probe(bootstrap)
        else:
            need(REMOTE_BOOTSTRAP, "remote-run requires bootstrap controller name")
            need(arguments.bootstrap is not None, "remote-run bootstrap path missing")
            bootstrap = pathlib.Path(arguments.bootstrap).resolve()
            need(bootstrap == CONTROLLER.parent, "remote-run bootstrap/controller mismatch")
            value = remote_run(bootstrap)
        print(json.dumps(value, ensure_ascii=False, sort_keys=True))
        return 0
    except Exception as error:
        print(f"D7 ERROR: {type(error).__name__}: {error}", file=sys.stderr)
        return 1


if __name__ == "__main__":
    raise SystemExit(main())
