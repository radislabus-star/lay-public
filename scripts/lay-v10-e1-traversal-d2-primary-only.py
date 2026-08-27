#!/usr/bin/env python3
"""Primary-only D2 controller. This revision stops after immutable D2-A closure."""

from __future__ import annotations

import argparse
import contextlib
import dataclasses
import hashlib
import json
import os
import pathlib
import platform
import re
import shlex
import shutil
import stat
import subprocess
import sys
import tempfile
import time
from typing import Any, Iterable, Mapping, Sequence


TASK_ID = "slice8b-v10-e1-traversal-d2-primary-only-v2-20260825"
REMOTE = "e@192.168.3.94"
REMOTE_HOSTNAME = "e-MEGA-MINI-M1-13th"
REMOTE_MACHINE_ID_SHA256 = "5ac0bb537745673ef80942c2df32d586f5753c85de1c8b2453aa4e7317b77441"
REMOTE_V1_PARENT = pathlib.Path(
    "/home/e/.local/share/lay/provenance/"
    "slice8b-v10-e1-traversal-d2-primary-only-v1-20260825"
)
REMOTE_V1_D2A = REMOTE_V1_PARENT / "d2a-v1"
REMOTE_V1_FAILURE = REMOTE_V1_PARENT / "d2a-failure-v1"
REMOTE_V1_STATE = pathlib.Path(
    "/home/e/.local/state/lay/slice8b-v10-e1-traversal-d2-primary-only-v1-20260825"
)
REMOTE_PARENT = pathlib.Path("/home/e/.local/share/lay/provenance") / TASK_ID
REMOTE_D2A = REMOTE_PARENT / "d2a-v1"
REMOTE_D2A_FAILURE = REMOTE_PARENT / "d2a-failure-v1"
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
PROJECT_ROOT = CONTROLLER.parents[1]
PREFLIGHT = PROJECT_ROOT / (
    "docs/structural_gates/preflights/"
    "LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_D2_"
    "PRIMARY_ONLY_IMPLEMENTATION_V4_2026-08-25.json"
)
PREFLIGHT_RECEIPT = PROJECT_ROOT / (
    "docs/structural_gates/receipts/"
    "LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_D2_"
    "PRIMARY_ONLY_IMPLEMENTATION_V4_PREFLIGHT_2026-08-25.json"
)
REPAIR_PREFLIGHT = PROJECT_ROOT / (
    "docs/structural_gates/preflights/"
    "LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_D2_PRIMARY_ONLY_"
    "D2A_CONTROLLER_REPAIR_V3_2026-08-25.json"
)
REPAIR_PREFLIGHT_RECEIPT = PROJECT_ROOT / (
    "docs/structural_gates/receipts/"
    "LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_D2_PRIMARY_ONLY_"
    "D2A_CONTROLLER_REPAIR_V3_PREFLIGHT_2026-08-25.json"
)
LOCAL_V1_FAILURE = PROJECT_ROOT / (
    "docs/structural_gates/receipts/"
    "LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_D2_PRIMARY_ONLY_"
    "D2A_FAILURE_V1_2026-08-25"
)
FRAGMENT = PROJECT_ROOT / "scripts/lay_v10_e1_remaining_cost_d1_test_module.rs.inc"
LOCAL_V10 = pathlib.Path(
    "/home/ubu/.local/share/lay/provenance/slice8b-v10-f6178f/"
    "artifacts/v13_typed_peak.v10.rs"
)
ACTIVE_V11 = PROJECT_ROOT / "src/nanda_wave/l2_field/v13_typed_peak.rs"
LOCAL_RESULT = PROJECT_ROOT / (
    "docs/structural_gates/receipts/"
    "LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_D2_PRIMARY_ONLY_V2_2026-08-25"
)

EXPECTED = {
    "preflight": "e9d6328b9f610ede73ae2e8d3c819b9728ac0fc5e4c263e9fe25a61978f80f5a",
    "preflight_size": 98_316,
    "preflight_receipt": "740c008d59fb4689826537e46a35da554bde863358d2c18382f315395ee835e0",
    "preflight_receipt_size": 14_171,
    "repair_preflight": "6589ca862a73aca61491c8b43edfc540a08ef96d8b7827a11b7ebf968b686596",
    "repair_preflight_size": 10_908,
    "repair_preflight_receipt": "e532213da5bbe064de38f8dc0da31cc647730316f5765c5cad781e31d2252dc4",
    "repair_preflight_receipt_size": 5_725,
    "v1_failure": "5d8d1db51238adf63f0d01757b4bddca587679200091d09388919b35091d2891",
    "v1_failure_manifest": "a0b5243c6d12383fa35c8b86819cf212b9c8b87f0b3e15c4df76e2fdf9a03473",
    "v10_source": "f6178f3882c1a216a9c2aaf7ff59dd6bdd78798708516b753ea7208177d9718c",
    "v10_source_size": 91_518,
    "production_prefix": "ce9ea2d290602774e2f45444e10e6fdabce6af4a61ae2bdca54c6d2e31a53f26",
    "production_prefix_size": 39_047,
    "d1_fragment": "bbd8b8d318810eec721812f21efbeb5f231dacba774cb5ade854e2201c6c7665",
    "d1_fragment_size": 113_204,
    "assembled_source": "6cd9edece91ac2e0c0e6dda7658e104dcf8953f1c16b1acff6108ea44ada0181",
    "assembled_source_size": 204_722,
    "active_v11": "d9edfe7346b8636096701d4ac38044d5e386e6741a086a4d8da57ac15bffdf3b",
    "package": "cce259fe0ce5dce67702383363b66f0fe9b9ff5a87d8f01c4fcf342d91218d7b",
    "package_size": 140_556_462,
    "sidecar": "a1aa95be8c43fae8fedf02d3261dd1265dcbfbc8d618330fcd6449ae829df8cd",
    "sidecar_size": 3_689_884,
    "v7": "33fded73e13f565bdc08a83bd440be67c80f6d85a196453050b09bc9f7ef28e4",
    "v7_size": 1_606_189,
    "schedule": "2f8346d97246ac3704434279739fa5ed27705465d628890a367c21875d879e78",
    "schedule_size": 174_941,
    "cargo_toml": "90a3ae6b16677ff49b70aab3457b794cd1e114a547f9e113a8b49595f976207b",
    "cargo_toml_size": 2_399,
    "cargo_lock": "e6399be48c393e1557b451fcb04d886f9fbdaa9812109410ac3d9ecdc98b93f1",
    "cargo_lock_size": 70_770,
    "cargo_guard": "a5b01f9a12b32ee2bcfc957a5e9f65efb52fe6e7644850ae3f00e412648243fe",
    "b0a_manifest": "920374efc2e75a021d53235aafea2a74dc7258546219bcae9c6a2bf53e194916",
    "b0a_receipt": "48176ec6faae86f43ddda8404542367be5b6c9d6813762dedb829b4946593eb3",
    "perf_wrapper": "2d0953085bf720a25efbe24f853e97d27b1f12f18a398255ff82cbafde254dad",
    "perf_resolved": "b0741eb0e6e769ba9ee0ae4e27f0c60909b51be4bc560802aef4bcd91130692e",
}

EXPECTED_CARGO = "cargo 1.97.1 (c980f4866 2026-06-30)"
EXPECTED_RUSTC = {
    "release": "1.97.1",
    "commit-hash": "8bab26f4f68e0e26f0bb7960be334d5b520ea452",
    "host": "x86_64-unknown-linux-gnu",
    "LLVM version": "22.1.6",
}
EXPECTED_RUSTC_DISPLAY_COMMIT = "8bab26f4f"
EXPECTED_TARGET_FEATURES = ("fxsr", "sse", "sse2")
EXPECTED_ROUTES = (
    "BUILD",
    "BUCKET-MAP",
    "PARITY",
    "U-SINGLE",
    "U-FIXED",
    "U-REVERSED",
    "V-FIXED-INSTR",
    "V-REVERSED-INSTR",
    "T-SINGLE",
    "T-FIXED",
    "T-REVERSED",
)
MARKERS = (
    "build.available",
    "bucket-map.available",
    "parity.available",
    "u-single.available",
    "u-fixed.available",
    "u-reversed.available",
    "v-fixed-instr.available",
    "v-reversed-instr.available",
    "t-single.available",
    "t-fixed.available",
    "t-reversed.available",
)
MARKER_ROUTE = dict(zip(MARKERS, EXPECTED_ROUTES, strict=True))

PARITY_TEST = "nanda_wave::l2_field::v13_typed_peak::tests::v10_d1_semantic_parity"
SINGLE_TEST = "nanda_wave::l2_field::v13_typed_peak::tests::v10_d1_component_single"
TWENTY_TEST = "nanda_wave::l2_field::v13_typed_peak::tests::v10_d1_component_twenty"
PMU_TEST = "nanda_wave::l2_field::v13_typed_peak::tests::v10_d1_twenty_pmu"
FIXED_CPUS = tuple(range(20))
REVERSED_CPUS = tuple(reversed(FIXED_CPUS))

DISPATCH_TABLES = {
    "U": (
        (0, "provenance", "BLOCKED_PROVENANCE"),
        (1, "thermal", "BLOCKED_THERMAL"),
        (2, "semantic", "BLOCKED_SEMANTIC"),
        (3, "perturbation", "BLOCKED_PERTURBATION"),
    ),
    "V": (
        (0, "provenance", "BLOCKED_PROVENANCE"),
        (1, "thermal", "BLOCKED_THERMAL"),
        (2, "capability", "BLOCKED_CAPABILITY"),
        (3, "denominator", "BLOCKED_DENOMINATOR"),
        (4, "perturbation", "BLOCKED_PERTURBATION"),
    ),
    "T": (
        (0, "provenance", "BLOCKED_PROVENANCE"),
        (1, "thermal", "BLOCKED_THERMAL"),
        (2, "capability", "BLOCKED_CAPABILITY"),
        (3, "bucket_map", "BLOCKED_BUCKET_MAP"),
        (4, "perturbation", "BLOCKED_PERTURBATION"),
        (5, "sample_coverage", "BLOCKED_SAMPLE_COVERAGE"),
    ),
}


class ControllerError(RuntimeError):
    pass


@dataclasses.dataclass(frozen=True)
class RouteSpec:
    route_id: str
    family: str
    kind: str
    test: str | None = None
    cpus: tuple[int, ...] = ()
    commands: tuple[tuple[str, ...], ...] = ()


def require(condition: bool, message: str) -> None:
    if not condition:
        raise ControllerError(message)


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


def remove_tree(root: pathlib.Path) -> None:
    if not root.exists():
        return
    make_tree_writable(root)
    shutil.rmtree(root)


def seal_tree(root: pathlib.Path) -> None:
    for path in sorted(root.rglob("*"), reverse=True):
        path.chmod(0o555 if path.is_dir() else 0o444)
    root.chmod(0o555)


def manifest_rows(root: pathlib.Path, exclude: set[str] | None = None) -> list[dict[str, Any]]:
    excluded = exclude or set()
    rows = []
    for path in sorted(candidate for candidate in root.rglob("*") if candidate.is_file()):
        relative = path.relative_to(root).as_posix()
        if relative in excluded:
            continue
        rows.append(
            {
                "path": relative,
                "mode": mode_string(path),
                "size_bytes": path.stat().st_size,
                "sha256": sha256_file(path),
            }
        )
    return rows


def write_sha256sums(root: pathlib.Path) -> None:
    rows = manifest_rows(root, {"SHA256SUMS"})
    data = "".join(f"{row['sha256']}  {row['path']}\n" for row in rows).encode()
    write_new_bytes(root / "SHA256SUMS", data, 0o444)


def verify_sha256sums(root: pathlib.Path) -> int:
    manifest = root / "SHA256SUMS"
    require(manifest.is_file(), f"missing SHA256SUMS: {root}")
    count = 0
    for line in manifest.read_text(encoding="utf-8").splitlines():
        digest, relative = line.split("  ", 1)
        path = root / relative
        require(path.is_file(), f"manifest member missing: {path}")
        require(sha256_file(path) == digest, f"manifest SHA mismatch: {path}")
        count += 1
    return count


def run(
    command: Sequence[str],
    *,
    env: Mapping[str, str] | None = None,
    check: bool = True,
    cwd: pathlib.Path | None = None,
) -> subprocess.CompletedProcess[bytes]:
    result = subprocess.run(
        list(command),
        env=dict(env) if env is not None else None,
        cwd=cwd,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        check=False,
    )
    if check and result.returncode != 0:
        detail = result.stderr.decode(errors="replace")[-4000:]
        raise ControllerError(
            f"command failed ({result.returncode}): {shlex.join(command)}\n{detail}"
        )
    return result


def ssh(command: Sequence[str], *, check: bool = True) -> subprocess.CompletedProcess[bytes]:
    return run(
        [
            "ssh",
            "-o",
            "BatchMode=yes",
            "-o",
            "ConnectTimeout=10",
            REMOTE,
            shlex.join(list(command)),
        ],
        check=check,
    )


def controlled_environment() -> dict[str, str]:
    return {
        "HOME": "/home/e",
        "PATH": "/home/e/.cargo/bin:/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin",
        "LANG": "C.UTF-8",
        "LC_ALL": "C.UTF-8",
        "TZ": "Europe/Tallinn",
    }


def subject_template(test: str) -> tuple[str, ...]:
    return (
        str(REMOTE_LOADER),
        "<D2-ELF>",
        "--exact",
        test,
        "--ignored",
        "--nocapture",
        "--test-threads=1",
    )


def route_registry() -> dict[str, RouteSpec]:
    build = (
        "<workspace>/scripts/cargo-guard.sh",
        "test",
        "--offline",
        "--locked",
        "--release",
        "--lib",
        "--no-run",
        "nanda_wave::l2_field::v13_typed_peak::tests",
    )
    map_commands = (
        (
            "/usr/bin/readelf",
            "--file-header",
            "--program-headers",
            "--sections",
            "--symbols",
            "--notes",
            "--debug-dump=decodedline",
            "<D2-ELF>",
        ),
        (
            "/usr/bin/objdump",
            "--disassemble",
            "--line-numbers",
            "--demangle",
            "--wide",
            "<D2-ELF>",
        ),
        (
            "/usr/bin/nm",
            "--numeric-sort",
            "--print-size",
            "--demangle",
            "<D2-ELF>",
        ),
        (
            "/usr/bin/addr2line",
            "--exe=<D2-ELF>",
            "--functions",
            "--inlines",
            "--demangle",
            "--addresses",
            "<sealed-address-list>",
        ),
    )
    perf_stat = (
        "/usr/bin/sudo",
        "-n",
        "/usr/bin/perf",
        "stat",
        "--json-output",
        "--no-big-num",
        "--delay=-1",
        "--control=fifo:<control>,<ack>",
        "--event",
        "instructions,cycles,branches,branch-misses",
        "--",
        "<exact-subject-command>",
    )
    perf_record = (
        "/usr/bin/sudo",
        "-n",
        "/usr/bin/perf",
        "record",
        "--buildid-all",
        "--sample-cpu",
        "--timestamp",
        "--event",
        "task-clock:u",
        "--count",
        "100000",
        "--output",
        "<route>/perf.data",
        "--",
        "<exact-subject-command>",
    )
    readers = (
        ("/usr/bin/perf", "evlist", "-v", "-i", "<perf.data>"),
        (
            "/usr/bin/perf",
            "script",
            "-i",
            "<perf.data>",
            "-F",
            "comm,pid,tid,cpu,time,event,ip,dso,period",
        ),
        ("/usr/bin/perf", "script", "-D", "-i", "<perf.data>"),
        ("/usr/bin/perf", "buildid-list", "-i", "<perf.data>"),
    )
    return {
        "BUILD": RouteSpec("BUILD", "BUILD", "cargo", commands=(build,)),
        "BUCKET-MAP": RouteSpec("BUCKET-MAP", "MAP", "map", commands=map_commands),
        "PARITY": RouteSpec(
            "PARITY", "PARITY", "subject", PARITY_TEST, commands=(subject_template(PARITY_TEST),)
        ),
        "U-SINGLE": RouteSpec(
            "U-SINGLE", "U", "subject", SINGLE_TEST, (0,), (subject_template(SINGLE_TEST),)
        ),
        "U-FIXED": RouteSpec(
            "U-FIXED", "U", "subject", TWENTY_TEST, FIXED_CPUS, (subject_template(TWENTY_TEST),)
        ),
        "U-REVERSED": RouteSpec(
            "U-REVERSED", "U", "subject", TWENTY_TEST, REVERSED_CPUS, (subject_template(TWENTY_TEST),)
        ),
        "V-FIXED-INSTR": RouteSpec(
            "V-FIXED-INSTR", "V", "perf-stat", PMU_TEST, FIXED_CPUS, (perf_stat, subject_template(PMU_TEST))
        ),
        "V-REVERSED-INSTR": RouteSpec(
            "V-REVERSED-INSTR", "V", "perf-stat", PMU_TEST, REVERSED_CPUS, (perf_stat, subject_template(PMU_TEST))
        ),
        "T-SINGLE": RouteSpec(
            "T-SINGLE", "T", "perf-record", SINGLE_TEST, (0,), (perf_record, subject_template(SINGLE_TEST), *readers)
        ),
        "T-FIXED": RouteSpec(
            "T-FIXED", "T", "perf-record", TWENTY_TEST, FIXED_CPUS, (perf_record, subject_template(TWENTY_TEST), *readers)
        ),
        "T-REVERSED": RouteSpec(
            "T-REVERSED", "T", "perf-record", TWENTY_TEST, REVERSED_CPUS, (perf_record, subject_template(TWENTY_TEST), *readers)
        ),
    }


ACTION_TO_ROUTE = {
    "build": "BUILD",
    "bucket-map": "BUCKET-MAP",
    "parity": "PARITY",
    "u-single": "U-SINGLE",
    "u-fixed": "U-FIXED",
    "u-reversed": "U-REVERSED",
    "v-fixed-instr": "V-FIXED-INSTR",
    "v-reversed-instr": "V-REVERSED-INSTR",
    "t-single": "T-SINGLE",
    "t-fixed": "T-FIXED",
    "t-reversed": "T-REVERSED",
}


def dispatch_observation(family: str, predicates: Mapping[str, Any]) -> dict[str, Any]:
    table = DISPATCH_TABLES.get(family)
    if table is None:
        return {
            "verdict": "BLOCKED_PROVENANCE",
            "reason": "unknown dispatch family",
            "matched": [],
        }
    expected = {name for _, name, _ in table}
    if set(predicates) != expected or any(type(value) is not bool for value in predicates.values()):
        return {
            "verdict": "BLOCKED_PROVENANCE",
            "reason": "incomplete unknown or non-boolean dispatch evidence",
            "matched": [],
        }
    matched = [
        {"priority": priority, "predicate": name, "terminal": terminal}
        for priority, name, terminal in table
        if predicates[name]
    ]
    if not matched:
        return {"verdict": "PASS", "reason": "no failure predicate matched", "matched": []}
    priorities = [row["priority"] for row in matched]
    if len(priorities) != len(set(priorities)):
        return {
            "verdict": "BLOCKED_PROVENANCE",
            "reason": "multiple causes at one dispatch priority",
            "matched": matched,
        }
    winner = min(matched, key=lambda row: row["priority"])
    return {
        "verdict": winner["terminal"],
        "reason": f"first matched priority {winner['priority']}: {winner['predicate']}",
        "matched": matched,
    }


def resolve_baseline_path(manifest: pathlib.Path, value: str) -> pathlib.Path:
    path = pathlib.Path(value)
    return path if path.is_absolute() else (manifest.parent / path).resolve()


def verify_json_verdicts(rows: Mapping[str, pathlib.Path]) -> None:
    expected = {
        "d2-v4-structural-review": ("D2_PAPER_REVIEWED", None),
        "d1-decision": ("D1_OBSERVED_WITH_CAPABILITY_GAP", None),
        "d1-pmu-correction-v2": (
            "D1_PMU_INTERPRETATION_PASS_FROM_SEALED_EVIDENCE",
            None,
        ),
        "tcap-interpretation-v3": ("T_CAP_RECOVERED_FROM_SEALED_EVIDENCE", None),
        "precise-capability-v3": ("BLOCKED_CAPABILITY", None),
        "secondary-gap-route-v6": ("PASS", None),
        "u-instruction-route-v7": ("PASS", None),
        "d2-p0-static-closure": ("P0_STATIC_PASS_P1_REQUIRED", None),
        "primary-only-v1-receipt": ("READY_TO_IMPLEMENT", True),
        "primary-only-v2-receipt": ("BLOCKED_BEFORE_CODE", False),
        "primary-only-v3-receipt": ("BLOCKED_BEFORE_CODE", False),
    }
    for identity, (verdict, safe) in expected.items():
        value = load_json(rows[identity])
        require(value.get("verdict") == verdict, f"verdict mismatch: {identity}")
        if safe is not None:
            require(value.get("safe_to_implement") is safe, f"safety mismatch: {identity}")
    require(load_json(rows["d2-v4-structural-review"]).get("authority_ready") is False, "D2 V4 paper gained authority")
    require(load_json(rows["secondary-gap-route-v6"]).get("authority_ready") is False, "V6 gained authority")
    require(load_json(rows["u-instruction-route-v7"]).get("authority_ready") is False, "V7 gained authority")
    require(load_json(rows["d2-p0-static-closure"]).get("final_implementation_ready") is False, "P0 became final")


def verify_route_registry(preflight: Mapping[str, Any]) -> dict[str, Any]:
    graph = preflight["execution_route_graph"]
    registry = route_registry()
    require(tuple(graph["route_ids"]) == EXPECTED_ROUTES, "manifest route order mismatch")
    require(set(registry) == set(EXPECTED_ROUTES), "controller route registry mismatch")
    require(tuple(graph["build"]["command"]) == registry["BUILD"].commands[0], "BUILD argv mismatch")
    require(
        tuple(tuple(command) for command in graph["bucket_map"]["commands"])
        == registry["BUCKET-MAP"].commands,
        "BUCKET-MAP argv mismatch",
    )
    require(tuple(graph["subject_argv_suffix"]) == subject_template("<exact-test>"), "subject argv suffix mismatch")
    require(tuple(graph["v_perf_stat_argv"]) == registry["V-FIXED-INSTR"].commands[0], "V perf stat argv mismatch")
    require(tuple(graph["t_perf_record_argv"]) == registry["T-FIXED"].commands[0], "T perf record argv mismatch")
    require(
        tuple(tuple(command) for command in graph["t_readers"])
        == registry["T-FIXED"].commands[2:],
        "T reader argv mismatch",
    )
    for route_id, subject in graph["subjects"].items():
        spec = registry[route_id]
        require(spec.test == subject["test"], f"test mismatch: {route_id}")
        require(spec.cpus == tuple(subject.get("cpus", ())), f"CPU mapping mismatch: {route_id}")
    perf_stat_routes = {key for key, value in registry.items() if value.kind == "perf-stat"}
    perf_record_routes = {key for key, value in registry.items() if value.kind == "perf-record"}
    require(perf_stat_routes == {"V-FIXED-INSTR", "V-REVERSED-INSTR"}, "perf stat reachability mismatch")
    require(perf_record_routes == {"T-SINGLE", "T-FIXED", "T-REVERSED"}, "perf record reachability mismatch")
    route_text = "\n".join(shlex.join(command) for spec in registry.values() for command in spec.commands)
    forbidden = (
        "--pid",
        "SIGINT",
        "cpu_core/event=",
        "cpu_atom/event=",
        "I-ATOM",
        "I-CORE",
        "clean C1",
        "full B",
        "systemctl",
        "pkill",
        "killall",
    )
    for token in forbidden:
        require(token not in route_text, f"forbidden reachable route token: {token}")
    for route_id in perf_record_routes:
        text = "\n".join(shlex.join(value) for value in registry[route_id].commands)
        require("task-clock:u" in text and "100000" in text, f"T event mismatch: {route_id}")
    require(set(ACTION_TO_ROUTE.values()) == set(EXPECTED_ROUTES), "action dispatch mismatch")
    return {
        "routes": list(EXPECTED_ROUTES),
        "route_count": len(registry),
        "perf_stat_routes": sorted(perf_stat_routes),
        "perf_record_routes": sorted(perf_record_routes),
        "forbidden_routes_present": False,
    }


def verify_dispatch_contract(preflight: Mapping[str, Any]) -> dict[str, Any]:
    contract = preflight["failure_dispatch_contract"]
    family_key = {"U": "u", "V": "v", "T": "t"}
    for family, key in family_key.items():
        table = DISPATCH_TABLES[family]
        frozen = contract[key]["priority"]
        require([terminal for _, _, terminal in table] == [row["terminal"] for row in frozen], f"{family} terminal order mismatch")
        require([rank for rank, _, _ in table] == list(range(len(table))), f"{family} priority gap")
        for index, (_, name, terminal) in enumerate(table):
            predicates = {candidate: False for _, candidate, _ in table}
            predicates[name] = True
            require(dispatch_observation(family, predicates)["verdict"] == terminal, f"{family} single dispatch mismatch")
            for lower in table[index + 1 :]:
                combined = dict(predicates)
                combined[lower[1]] = True
                require(dispatch_observation(family, combined)["verdict"] == terminal, f"{family} priority dispatch mismatch")
        clear = {name: False for _, name, _ in table}
        require(dispatch_observation(family, clear)["verdict"] == "PASS", f"{family} PASS dispatch mismatch")
        incomplete = dict(clear)
        incomplete.pop(next(iter(incomplete)))
        require(dispatch_observation(family, incomplete)["verdict"] == "BLOCKED_PROVENANCE", f"{family} incomplete dispatch accepted")
        invalid = dict(clear)
        invalid[next(iter(invalid))] = None
        require(dispatch_observation(family, invalid)["verdict"] == "BLOCKED_PROVENANCE", f"{family} unknown dispatch accepted")
    require(contract.get("dispatch_steps_execute_subject_or_pmu") is False, "dispatch gained execution effect")
    return {
        "tables": {
            family: [
                {"priority": priority, "predicate": predicate, "terminal": terminal}
                for priority, predicate, terminal in table
            ]
            for family, table in DISPATCH_TABLES.items()
        },
        "unknown_is_blocked_provenance": True,
        "dispatch_executes_subject_or_pmu": False,
    }


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
    preflight_row = require_file(
        PREFLIGHT,
        digest=EXPECTED["preflight"],
        size=EXPECTED["preflight_size"],
        mode="0444",
    )
    receipt_row = require_file(
        PREFLIGHT_RECEIPT,
        digest=EXPECTED["preflight_receipt"],
        size=EXPECTED["preflight_receipt_size"],
        mode="0444",
    )
    preflight = load_json(PREFLIGHT)
    receipt = load_json(PREFLIGHT_RECEIPT)
    repair_row = require_file(
        REPAIR_PREFLIGHT,
        digest=EXPECTED["repair_preflight"],
        size=EXPECTED["repair_preflight_size"],
        mode="0444",
    )
    repair_receipt_row = require_file(
        REPAIR_PREFLIGHT_RECEIPT,
        digest=EXPECTED["repair_preflight_receipt"],
        size=EXPECTED["repair_preflight_receipt_size"],
        mode="0444",
    )
    repair_receipt = load_json(REPAIR_PREFLIGHT_RECEIPT)
    require(receipt.get("verdict") == "READY_TO_IMPLEMENT", "effective V4 preflight not READY")
    require(receipt.get("safe_to_implement") is True and not receipt.get("blockers"), "effective V4 preflight unsafe")
    require(repair_receipt.get("verdict") == "READY_TO_IMPLEMENT", "repair V3 preflight not READY")
    require(
        repair_receipt.get("safe_to_implement") is True and not repair_receipt.get("blockers"),
        "repair V3 preflight unsafe",
    )
    require(preflight.get("scoped_positive_verdict") == "READY_TO_IMPLEMENT_PRIMARY_ONLY_D2", "scoped verdict mismatch")
    require(preflight.get("task_id", "").endswith("IMPLEMENTATION_V4_2026-08-25"), "effective V4 task mismatch")
    receipt_rows = {row["id"]: row for row in receipt["baseline_receipts"]}
    require(len(receipt_rows) == len(preflight["baseline_checks"]), "V4 baseline receipt count mismatch")
    existing: dict[str, pathlib.Path] = {}
    baseline = {}
    for check in preflight["baseline_checks"]:
        identity = check["id"]
        recorded = receipt_rows.get(identity)
        require(recorded is not None, f"missing V4 baseline receipt: {identity}")
        if check["kind"] == "absent":
            require(recorded.get("exists") is False, f"historical absence not sealed: {identity}")
            baseline[identity] = {"historical_absence": True, "path": recorded["path"]}
            continue
        path = resolve_baseline_path(PREFLIGHT, check["path"])
        expected = check["expect"]
        row = require_file(
            path,
            digest=expected.get("sha256"),
            size=expected.get("size_bytes"),
            mode=expected.get("mode"),
        )
        require(row["sha256"] == recorded["sha256"], f"V4 receipt SHA drift: {identity}")
        require(row["size_bytes"] == recorded["size_bytes"], f"V4 receipt size drift: {identity}")
        require(row["mode"] == recorded["mode"], f"V4 receipt mode drift: {identity}")
        existing[identity] = path
        baseline[identity] = row
    verify_json_verdicts(existing)
    require(not LOCAL_RESULT.exists(), "local D2-A result already exists")
    v1_failure = require_file(
        LOCAL_V1_FAILURE / "D2A_FAILURE.json",
        digest=EXPECTED["v1_failure"],
        size=406,
        mode="0444",
    )
    v1_failure_manifest = require_file(
        LOCAL_V1_FAILURE / "SHA256SUMS",
        digest=EXPECTED["v1_failure_manifest"],
        size=3_591,
        mode="0444",
    )
    require(verify_sha256sums(LOCAL_V1_FAILURE) == 33, "local V1 failure manifest count mismatch")
    v10 = require_file(
        LOCAL_V10,
        digest=EXPECTED["v10_source"],
        size=EXPECTED["v10_source_size"],
        mode="0444",
    )
    fragment = require_file(
        FRAGMENT,
        digest=EXPECTED["d1_fragment"],
        size=EXPECTED["d1_fragment_size"],
        mode="0664",
    )
    source = assemble_source(LOCAL_V10.read_bytes(), FRAGMENT.read_bytes())
    return {
        "preflight": preflight_row,
        "preflight_receipt": receipt_row,
        "repair_preflight": repair_row,
        "repair_preflight_receipt": repair_receipt_row,
        "v1_failure": v1_failure,
        "v1_failure_manifest": v1_failure_manifest,
        "baseline": baseline,
        "baseline_paths": {identity: str(path) for identity, path in existing.items()},
        "v10": v10,
        "fragment": fragment,
        "assembled_source": {
            "size_bytes": len(source),
            "sha256": sha256_bytes(source),
            "production_prefix_size": EXPECTED["production_prefix_size"],
            "production_prefix_sha256": sha256_bytes(source[: EXPECTED["production_prefix_size"]]),
        },
        "route_registry": verify_route_registry(preflight),
        "failure_dispatch": verify_dispatch_contract(preflight),
    }


def self_check() -> dict[str, Any]:
    admission = verify_local_admission()
    compile(CONTROLLER.read_text(encoding="utf-8"), str(CONTROLLER), "exec")
    return {
        "schema": "lay.v10.e1-traversal-d2-primary-only-controller-self-check.v2",
        "task_id": TASK_ID,
        "verdict": "PRIMARY_ONLY_CONTROLLER_VERIFIED_UNRUN",
        "controller": file_identity(CONTROLLER),
        "admission": admission,
        "markers_expected": len(MARKERS),
        "remote_actions": 0,
        "remote_state_created": False,
        "cargo_invocations": 0,
        "rustc_compilations": 0,
        "perf_record": 0,
        "perf_stat": 0,
        "d2_subject": 0,
        "bucket_map_created": False,
        "runtime_authority_changed": False,
    }


def local_runtime_snapshot() -> dict[str, Any]:
    def pids(name: str) -> list[int]:
        result = run(["pgrep", "-x", name], check=False)
        return sorted(int(value) for value in result.stdout.split())

    version = run([str(pathlib.Path.home() / ".local/bin/lay"), "--version"]).stdout.decode().strip()
    value = {
        "lay_version": version,
        "active_v11_sha256": sha256_file(ACTIVE_V11),
        "ibus_daemon_pid": pids("ibus-daemon"),
        "lay_daemon_pid": pids("lay-daemon"),
        "lay_ibus_engine_pid": pids("lay-ibus-engine"),
    }
    require(value["lay_version"] in ("1.0.43", "lay 1.0.43"), "installed Lay version drift")
    require(value["active_v11_sha256"] == EXPECTED["active_v11"], "active V11 drift")
    require(value["ibus_daemon_pid"] == [2076194], "ibus-daemon PID drift")
    require(value["lay_daemon_pid"] == [3410795], "lay-daemon PID drift")
    require(value["lay_ibus_engine_pid"] == [3410820], "lay-ibus-engine PID drift")
    return value


def copy_preserving(source: pathlib.Path, destination: pathlib.Path) -> None:
    destination.parent.mkdir(parents=True, exist_ok=True)
    shutil.copy2(source, destination)


def prepare_bootstrap(root: pathlib.Path, check: Mapping[str, Any]) -> dict[str, Any]:
    root.mkdir(mode=0o700)
    copy_preserving(CONTROLLER, root / "controller.py")
    copy_preserving(FRAGMENT, root / "fragment.inc")
    copy_preserving(PREFLIGHT, root / "preflight-v4.json")
    copy_preserving(PREFLIGHT_RECEIPT, root / "preflight-v4-receipt.json")
    copy_preserving(REPAIR_PREFLIGHT, root / "repair-preflight-v3.json")
    copy_preserving(REPAIR_PREFLIGHT_RECEIPT, root / "repair-preflight-v3-receipt.json")
    write_new_json(root / "LOCAL_SELF_CHECK.json", check, 0o444)
    predecessor_dir = root / "predecessors"
    predecessor_dir.mkdir()
    predecessors = []
    for index, (identity, source_value) in enumerate(
        sorted(check["admission"]["baseline_paths"].items())
    ):
        source = pathlib.Path(source_value)
        suffix = source.suffix if source.suffix else ".bin"
        relative = pathlib.Path("predecessors") / f"{index:02d}-{identity}{suffix}"
        copy_preserving(source, root / relative)
        row = file_identity(root / relative)
        row.update({"id": identity, "relative_path": relative.as_posix(), "original_path": str(source)})
        predecessors.append(row)
    manifest = {
        "schema": "lay.v10.e1-traversal-d2-primary-only-bootstrap.v1",
        "task_id": TASK_ID,
        "controller": file_identity(root / "controller.py"),
        "fragment": file_identity(root / "fragment.inc"),
        "preflight": file_identity(root / "preflight-v4.json"),
        "preflight_receipt": file_identity(root / "preflight-v4-receipt.json"),
        "repair_preflight": file_identity(root / "repair-preflight-v3.json"),
        "repair_preflight_receipt": file_identity(root / "repair-preflight-v3-receipt.json"),
        "local_self_check": file_identity(root / "LOCAL_SELF_CHECK.json"),
        "predecessors": predecessors,
        "historical_absence_ids": ["primary-only-controller-absent", "primary-only-result-absent"],
    }
    write_new_json(root / "BOOTSTRAP_MANIFEST.json", manifest, 0o444)
    return manifest


def upload_bootstrap(source: pathlib.Path, destination: pathlib.Path) -> None:
    result = run(
        ["scp", "-q", "-p", "-r", f"{source}/.", f"{REMOTE}:{destination}/"],
        check=False,
    )
    require(result.returncode == 0, result.stderr.decode(errors="replace")[-4000:])


def validate_bootstrap(root: pathlib.Path) -> tuple[dict[str, Any], dict[str, pathlib.Path]]:
    manifest = load_json(root / "BOOTSTRAP_MANIFEST.json")
    require(manifest.get("task_id") == TASK_ID, "bootstrap task mismatch")
    for key, name in (
        ("controller", "controller.py"),
        ("fragment", "fragment.inc"),
        ("preflight", "preflight-v4.json"),
        ("preflight_receipt", "preflight-v4-receipt.json"),
        ("repair_preflight", "repair-preflight-v3.json"),
        ("repair_preflight_receipt", "repair-preflight-v3-receipt.json"),
        ("local_self_check", "LOCAL_SELF_CHECK.json"),
    ):
        expected = manifest[key]
        require_file(
            root / name,
            digest=expected["sha256"],
            size=expected["size_bytes"],
            mode=expected["mode"],
        )
    require_file(
        root / "preflight-v4.json",
        digest=EXPECTED["preflight"],
        size=EXPECTED["preflight_size"],
        mode="0444",
    )
    require_file(
        root / "preflight-v4-receipt.json",
        digest=EXPECTED["preflight_receipt"],
        size=EXPECTED["preflight_receipt_size"],
        mode="0444",
    )
    require_file(
        root / "repair-preflight-v3.json",
        digest=EXPECTED["repair_preflight"],
        size=EXPECTED["repair_preflight_size"],
        mode="0444",
    )
    require_file(
        root / "repair-preflight-v3-receipt.json",
        digest=EXPECTED["repair_preflight_receipt"],
        size=EXPECTED["repair_preflight_receipt_size"],
        mode="0444",
    )
    paths = {}
    for row in manifest["predecessors"]:
        path = root / row["relative_path"]
        require_file(path, digest=row["sha256"], size=row["size_bytes"], mode=row["mode"])
        paths[row["id"]] = path
    verify_json_verdicts(paths)
    preflight = load_json(root / "preflight-v4.json")
    receipt = load_json(root / "preflight-v4-receipt.json")
    repair_receipt = load_json(root / "repair-preflight-v3-receipt.json")
    require(receipt.get("verdict") == "READY_TO_IMPLEMENT", "bootstrap V4 not READY")
    require(receipt.get("safe_to_implement") is True and not receipt.get("blockers"), "bootstrap V4 unsafe")
    require(repair_receipt.get("verdict") == "READY_TO_IMPLEMENT", "bootstrap repair V3 not READY")
    require(
        repair_receipt.get("safe_to_implement") is True and not repair_receipt.get("blockers"),
        "bootstrap repair V3 unsafe",
    )
    verify_route_registry(preflight)
    verify_dispatch_contract(preflight)
    return manifest, paths


def remote_input_paths() -> dict[str, pathlib.Path]:
    artifacts = REMOTE_B0A / "inputs/slice8b-v10-f6178f/artifacts"
    return {
        "package": REMOTE_B0A / "inputs/LAY-L2-RU-FULL-v13.bin",
        "sidecar": artifacts / "LAY-L2-RU-FULL-v13.dafsa",
        "v7": artifacts / "slice8b-v7-fixed-13x100.json",
        "schedule": REMOTE_B0B / "query-schedule.json",
        "v10": artifacts / "v13_typed_peak.v10.rs",
        "cargo_toml": artifacts / "Cargo.toml",
        "cargo_lock": artifacts / "Cargo.lock",
        "cargo_guard": REMOTE_B0A / "inputs/controller/cargo-guard.sh",
    }


def remote_inputs_closure() -> dict[str, Any]:
    require(REMOTE_B0A.is_dir(), "B0a immutable closure missing")
    require(REMOTE_B0B.is_dir(), "B0b immutable closure missing")
    require((REMOTE_B0A / "inputs/surviving-source-closure").is_dir(), "source closure missing")
    rows = {
        "b0a_manifest": require_file(
            REMOTE_B0A / "SHA256SUMS", digest=EXPECTED["b0a_manifest"], mode="0444"
        ),
        "b0a_receipt": require_file(
            REMOTE_B0A / "INPUT_CLOSURE.json", digest=EXPECTED["b0a_receipt"], mode="0444"
        ),
    }
    paths = remote_input_paths()
    rows.update(
        {
            "package": require_file(paths["package"], digest=EXPECTED["package"], size=EXPECTED["package_size"], mode="0444"),
            "sidecar": require_file(paths["sidecar"], digest=EXPECTED["sidecar"], size=EXPECTED["sidecar_size"], mode="0444"),
            "v7": require_file(paths["v7"], digest=EXPECTED["v7"], size=EXPECTED["v7_size"], mode="0444"),
            "schedule": require_file(paths["schedule"], digest=EXPECTED["schedule"], size=EXPECTED["schedule_size"], mode="0444"),
            "v10": require_file(paths["v10"], digest=EXPECTED["v10_source"], size=EXPECTED["v10_source_size"], mode="0444"),
            "cargo_toml": require_file(paths["cargo_toml"], digest=EXPECTED["cargo_toml"], size=EXPECTED["cargo_toml_size"], mode="0444"),
            "cargo_lock": require_file(paths["cargo_lock"], digest=EXPECTED["cargo_lock"], size=EXPECTED["cargo_lock_size"], mode="0444"),
            "cargo_guard": require_file(paths["cargo_guard"], digest=EXPECTED["cargo_guard"]),
        }
    )
    return rows


def read_required(path: pathlib.Path) -> str:
    require(path.is_file(), f"missing host field: {path}")
    return path.read_text(encoding="utf-8").strip()


def parse_rustc_verbose(text: str) -> dict[str, str]:
    rows = {}
    for line in text.splitlines():
        if ": " in line:
            key, value = line.split(": ", 1)
            rows[key] = value
    return rows


def host_toolchain_closure() -> dict[str, Any]:
    machine = {
        "hostname": platform.node(),
        "machine_id_exact_file_sha256": sha256_file(pathlib.Path("/etc/machine-id")),
        "kernel": platform.release(),
        "online_cpus": read_required(pathlib.Path("/sys/devices/system/cpu/online")),
        "cpu_core": {
            "type": int(read_required(pathlib.Path("/sys/bus/event_source/devices/cpu_core/type"))),
            "cpus": read_required(pathlib.Path("/sys/bus/event_source/devices/cpu_core/cpus")),
        },
        "cpu_atom": {
            "type": int(read_required(pathlib.Path("/sys/bus/event_source/devices/cpu_atom/type"))),
            "cpus": read_required(pathlib.Path("/sys/bus/event_source/devices/cpu_atom/cpus")),
        },
    }
    require(machine["hostname"] == REMOTE_HOSTNAME, "remote hostname mismatch")
    require(machine["machine_id_exact_file_sha256"] == REMOTE_MACHINE_ID_SHA256, "remote machine identity mismatch")
    require(machine["kernel"] == "6.8.0-124-generic", "remote kernel drift")
    require(machine["online_cpus"] == "0-19", "online CPU drift")
    require(machine["cpu_core"] == {"type": 4, "cpus": "0-11"}, "cpu_core PMU drift")
    require(machine["cpu_atom"] == {"type": 10, "cpus": "12-19"}, "cpu_atom PMU drift")
    environment = controlled_environment()
    cargo = run(["cargo", "-V"], env=environment).stdout.decode().strip()
    rustc_text = run(["rustc", "-Vv"], env=environment).stdout.decode().strip()
    rustc = parse_rustc_verbose(rustc_text)
    features_text = run(
        ["rustc", "--print", "cfg", "--target", "x86_64-unknown-linux-gnu"],
        env=environment,
    ).stdout.decode()
    features = tuple(
        sorted(
            match.group(1)
            for line in features_text.splitlines()
            if (match := re.fullmatch(r'target_feature="([^"]+)"', line))
        )
    )
    require(cargo == EXPECTED_CARGO, "Cargo version drift")
    for key, value in EXPECTED_RUSTC.items():
        require(rustc.get(key) == value, f"rustc {key} drift")
    require(features == EXPECTED_TARGET_FEATURES, f"target feature drift: {features}")
    perf_wrapper = pathlib.Path("/usr/bin/perf")
    perf_resolved = pathlib.Path("/usr/lib/linux-tools/6.8.0-124-generic/perf").resolve()
    perf = {
        "wrapper": require_file(perf_wrapper, digest=EXPECTED["perf_wrapper"], size=1_622, mode="0755"),
        "resolved": require_file(perf_resolved, digest=EXPECTED["perf_resolved"], size=11_627_592, mode="0755"),
    }
    return {
        "host": machine,
        "toolchain": {
            "cargo": cargo,
            "rustc": (
                f"rustc {rustc['release']} "
                f"({EXPECTED_RUSTC_DISPLAY_COMMIT} 2026-07-14)"
            ),
            "rustc_vv": rustc_text,
            "llvm": rustc["LLVM version"],
            "host": rustc["host"],
            "target_features": list(features),
            "rustflags": [],
            "version_queries_only": True,
        },
        "perf_files_read_without_execution": perf,
    }


def prepare_state_stage(stage: pathlib.Path, transaction_id: str, controller_sha: str) -> None:
    markers = stage / "markers"
    markers.mkdir(parents=True, mode=0o700)
    for marker in MARKERS:
        write_new_json(
            markers / marker,
            {
                "schema": "lay.v10.e1-traversal-d2-primary-only-marker.v1",
                "task_id": TASK_ID,
                "transaction_id": transaction_id,
                "route_id": MARKER_ROUTE[marker],
                "state": "available",
                "retry_permitted": False,
                "controller_sha256": controller_sha,
                "preflight_sha256": EXPECTED["preflight"],
            },
            0o400,
        )
    marker_rows = [file_identity(markers / marker) for marker in MARKERS]
    write_new_json(
        stage / "STATE.json",
        {
            "schema": "lay.v10.e1-traversal-d2-primary-only-state.v1",
            "task_id": TASK_ID,
            "transaction_id": transaction_id,
            "state": "D2A_CLOSED_ALL_MARKERS_AVAILABLE",
            "markers_expected": len(MARKERS),
            "markers_created": len(marker_rows),
            "markers_consumed": 0,
            "marker_rows": marker_rows,
            "retry_permitted": False,
        },
        0o400,
    )
    write_new_json(
        stage / "route.lock",
        {
            "schema": "lay.v10.e1-traversal-d2-primary-only-route-lock.v1",
            "task_id": TASK_ID,
            "transaction_id": transaction_id,
            "state": "unlocked",
        },
        0o400,
    )
    fsync_directory(markers)
    fsync_directory(stage)


def remote_marker_rows() -> list[dict[str, Any]]:
    markers = REMOTE_STATE / "markers"
    require(markers.is_dir(), "published marker directory missing")
    require(sorted(path.name for path in markers.iterdir()) == sorted(MARKERS), "published marker set mismatch")
    rows = []
    for marker in MARKERS:
        row = require_file(markers / marker, mode="0400")
        value = load_json(markers / marker)
        require(value.get("state") == "available", f"marker not available: {marker}")
        require(value.get("route_id") == MARKER_ROUTE[marker], f"marker route mismatch: {marker}")
        row["marker"] = marker
        row["route_id"] = value["route_id"]
        rows.append(row)
    require(not list(markers.glob("*.consumed*")), "a route marker was consumed during D2-A")
    return rows


def remote_v1_failure_closure() -> dict[str, Any]:
    require(REMOTE_V1_PARENT.is_dir(), "sealed V1 parent missing")
    require(not REMOTE_V1_D2A.exists(), "V1 unexpectedly has D2-A PASS")
    require(not REMOTE_V1_STATE.exists(), "V1 unexpectedly published marker state")
    failure = require_file(
        REMOTE_V1_FAILURE / "D2A_FAILURE.json",
        digest=EXPECTED["v1_failure"],
        size=406,
        mode="0444",
    )
    manifest = require_file(
        REMOTE_V1_FAILURE / "SHA256SUMS",
        digest=EXPECTED["v1_failure_manifest"],
        size=3_591,
        mode="0444",
    )
    require(verify_sha256sums(REMOTE_V1_FAILURE) == 33, "remote V1 failure manifest count mismatch")
    value = load_json(REMOTE_V1_FAILURE / "D2A_FAILURE.json")
    require(value.get("error") == "rustc commit-hash drift", "V1 failure reason drift")
    require(value.get("state_published") is False, "V1 unexpectedly published state")
    require(value.get("retry_permitted") is False, "V1 unexpectedly permits retry")
    return {
        "verdict": "V1_D2A_FAILURE_PRESERVED",
        "failure": failure,
        "manifest": manifest,
        "manifest_entries": 33,
        "state_published": False,
        "markers_created": 0,
        "retry_permitted": False,
    }


def remote_d2a(bootstrap: pathlib.Path) -> None:
    require(platform.node() == REMOTE_HOSTNAME, "remote-d2a executed on wrong host")
    require(not REMOTE_PARENT.exists(), "D2 remote parent already exists")
    require(not REMOTE_STATE.exists(), "D2 remote state already exists")
    require(not REMOTE_D2A.exists() and not REMOTE_D2A_FAILURE.exists(), "D2-A terminal evidence already exists")
    v1_failure = remote_v1_failure_closure()
    manifest, predecessor_paths = validate_bootstrap(bootstrap)
    preflight = load_json(bootstrap / "preflight-v4.json")
    controller_row = file_identity(pathlib.Path(__file__).resolve())
    require(controller_row["sha256"] == manifest["controller"]["sha256"], "remote controller identity drift")
    REMOTE_PARENT.mkdir(parents=True, mode=0o700)
    result_stage = REMOTE_PARENT / f"d2a-v1.stage-{os.getpid()}-{time.time_ns()}"
    state_stage = REMOTE_STATE.parent / f".{TASK_ID}.stage-{os.getpid()}-{time.time_ns()}"
    result_stage.mkdir(mode=0o700)
    state_published = False
    try:
        inputs = result_stage / "inputs"
        inputs.mkdir()
        shutil.copy2(bootstrap / "controller.py", inputs / "controller.py")
        shutil.copy2(bootstrap / "fragment.inc", inputs / "fragment.inc")
        shutil.copy2(bootstrap / "preflight-v4.json", inputs / "preflight-v4.json")
        shutil.copy2(bootstrap / "preflight-v4-receipt.json", inputs / "preflight-v4-receipt.json")
        shutil.copy2(bootstrap / "repair-preflight-v3.json", inputs / "repair-preflight-v3.json")
        shutil.copy2(
            bootstrap / "repair-preflight-v3-receipt.json",
            inputs / "repair-preflight-v3-receipt.json",
        )
        shutil.copy2(bootstrap / "BOOTSTRAP_MANIFEST.json", inputs / "BOOTSTRAP_MANIFEST.json")
        shutil.copy2(bootstrap / "LOCAL_SELF_CHECK.json", inputs / "LOCAL_SELF_CHECK.json")
        shutil.copytree(bootstrap / "predecessors", inputs / "predecessors")
        remote_inputs = remote_inputs_closure()
        host_toolchain = host_toolchain_closure()
        source = assemble_source(
            remote_input_paths()["v10"].read_bytes(),
            (bootstrap / "fragment.inc").read_bytes(),
        )
        write_new_bytes(inputs / "assembled_d2_source.rs", source, 0o444)
        source_closure = {
            "v10": remote_inputs["v10"],
            "fragment": file_identity(bootstrap / "fragment.inc"),
            "production_prefix_bytes": EXPECTED["production_prefix_size"],
            "production_prefix_sha256": sha256_bytes(source[: EXPECTED["production_prefix_size"]]),
            "assembled_source": file_identity(inputs / "assembled_d2_source.rs"),
            "compiled": False,
        }
        command_graph = verify_route_registry(preflight)
        failure_dispatch = verify_dispatch_contract(preflight)
        transaction_id = sha256_bytes(
            canonical_json_bytes(
                {
                    "task_id": TASK_ID,
                    "controller": controller_row["sha256"],
                    "preflight": EXPECTED["preflight"],
                    "source": EXPECTED["assembled_source"],
                    "host": host_toolchain["host"],
                }
            )
        )
        state_stage.mkdir(mode=0o700)
        prepare_state_stage(state_stage, transaction_id, controller_row["sha256"])
        fsync_directory(state_stage.parent)
        os.rename(state_stage, REMOTE_STATE)
        fsync_directory(REMOTE_STATE.parent)
        state_published = True
        markers = remote_marker_rows()
        paper_rows = {
            identity: file_identity(path) for identity, path in sorted(predecessor_paths.items())
        }
        receipt = {
            "schema": "lay.v10.e1-traversal-d2-primary-only-d2a.v2",
            "task_id": TASK_ID,
            "transaction_id": transaction_id,
            "verdict": "D2A_CLOSED_ALL_MARKERS_AVAILABLE",
            "controller_state": "PRIMARY_ONLY_CONTROLLER_VERIFIED_UNRUN",
            "preflight": {
                "manifest": file_identity(bootstrap / "preflight-v4.json"),
                "receipt": file_identity(bootstrap / "preflight-v4-receipt.json"),
                "engine_verdict": "READY_TO_IMPLEMENT",
                "project_verdict": "READY_TO_IMPLEMENT_PRIMARY_ONLY_D2",
            },
            "repair_admission": {
                "manifest": file_identity(bootstrap / "repair-preflight-v3.json"),
                "receipt": file_identity(bootstrap / "repair-preflight-v3-receipt.json"),
                "verdict": "READY_TO_IMPLEMENT",
                "safe_to_implement": True,
            },
            "v1_failure_preservation": v1_failure,
            "paper_preflight_chain": paper_rows,
            "paper_preflight_chain_count": len(paper_rows),
            "source_closure": source_closure,
            "remote_inputs": remote_inputs,
            "host_toolchain_closure": host_toolchain,
            "command_graph": command_graph,
            "failure_dispatch": failure_dispatch,
            "markers_expected": len(MARKERS),
            "markers_created": len(markers),
            "markers_consumed": 0,
            "markers": markers,
            "state": file_identity(REMOTE_STATE / "STATE.json"),
            "route_lock": file_identity(REMOTE_STATE / "route.lock"),
            "cargo_invocations": 0,
            "cargo_version_queries": 1,
            "rustc_compilations": 0,
            "rustc_version_or_cfg_queries": 2,
            "perf_record": 0,
            "perf_stat": 0,
            "pmu_events_opened": 0,
            "d2_subject": 0,
            "d2_elf_created": False,
            "bucket_map_created": False,
            "parity_executed": False,
            "u_routes_executed": 0,
            "v_routes_executed": 0,
            "t_routes_executed": 0,
            "foreign_process_control": False,
            "host_tuning": False,
            "runtime_authority_changed": False,
            "installed_lay_changed": False,
            "retry_permitted": False,
            "next_action_admitted": "independent D2-A audit only; build remains unexecuted",
        }
        write_new_json(result_stage / "D2A_RECEIPT.json", receipt, 0o444)
        write_new_json(
            result_stage / "ZERO_EXECUTION_LEDGER.json",
            {
                "cargo_invocations": 0,
                "rustc_compilations": 0,
                "perf_record": 0,
                "perf_stat": 0,
                "pmu_events_opened": 0,
                "d2_subject": 0,
                "bucket_map_created": False,
                "markers_consumed": 0,
                "runtime_authority_changed": False,
            },
            0o444,
        )
        write_sha256sums(result_stage)
        seal_tree(result_stage)
        os.rename(result_stage, REMOTE_D2A)
        fsync_directory(REMOTE_PARENT)
        print(
            json.dumps(
                {
                    "state": "D2A_CLOSED_ALL_MARKERS_AVAILABLE",
                    "controller": "PRIMARY_ONLY_CONTROLLER_VERIFIED_UNRUN",
                    "result": str(REMOTE_D2A),
                    "markers": len(markers),
                },
                sort_keys=True,
            )
        )
    except Exception as error:
        if not state_published:
            remove_tree(state_stage)
        with contextlib.suppress(Exception):
            if result_stage.exists():
                write_new_json(
                    result_stage / "D2A_FAILURE.json",
                    {
                        "schema": "lay.v10.e1-traversal-d2-primary-only-d2a-failure.v1",
                        "task_id": TASK_ID,
                        "error": str(error),
                        "state_published": state_published,
                        "markers_consumed": 0,
                        "cargo_invocations": 0,
                        "rustc_compilations": 0,
                        "perf_record": 0,
                        "perf_stat": 0,
                        "d2_subject": 0,
                        "runtime_authority_changed": False,
                        "retry_permitted": False,
                    },
                    0o444,
                )
                write_sha256sums(result_stage)
                seal_tree(result_stage)
                os.rename(result_stage, REMOTE_D2A_FAILURE)
                fsync_directory(REMOTE_PARENT)
        raise


def remote_status() -> dict[str, Any]:
    command = (
        "import hashlib,json,os,pathlib;"
        "m=pathlib.Path('/etc/machine-id');"
        f"p=pathlib.Path('{REMOTE_PARENT}');"
        f"d=pathlib.Path('{REMOTE_D2A}');"
        f"f=pathlib.Path('{REMOTE_D2A_FAILURE}');"
        f"s=pathlib.Path('{REMOTE_STATE}');"
        "markers=s/'markers';"
        "print(json.dumps({'host':os.uname().nodename,'machine':hashlib.sha256(m.read_bytes()).hexdigest(),"
        "'parent':p.exists(),'d2a':d.exists(),'failure':f.exists(),'state':s.exists(),"
        "'markers':sorted(x.name for x in markers.glob('*')) if markers.is_dir() else []}))"
    )
    return json.loads(ssh(["python3", "-c", command]).stdout)


def local_d2a() -> dict[str, Any]:
    check = self_check()
    status = remote_status()
    require(
        status
        == {
            "host": REMOTE_HOSTNAME,
            "machine": REMOTE_MACHINE_ID_SHA256,
            "parent": False,
            "d2a": False,
            "failure": False,
            "state": False,
            "markers": [],
        },
        f"D2-A remote pre-state mismatch: {status}",
    )
    before = local_runtime_snapshot()
    remote_bootstrap = pathlib.Path(
        ssh(["mktemp", "-d", "/tmp/lay-v10-d2a.XXXXXX"]).stdout.decode().strip()
    )
    require(str(remote_bootstrap).startswith("/tmp/lay-v10-d2a."), "unexpected bootstrap path")
    with tempfile.TemporaryDirectory(prefix="lay-v10-d2a-local-") as temporary:
        local_bootstrap = pathlib.Path(temporary) / "bootstrap"
        prepare_bootstrap(local_bootstrap, check)
        try:
            upload_bootstrap(local_bootstrap, remote_bootstrap)
            result = ssh(
                ["python3", f"{remote_bootstrap}/controller.py", "remote-d2a", str(remote_bootstrap)],
                check=False,
            )
            require(result.returncode == 0, result.stderr.decode(errors="replace")[-8000:])
            remote_output = result.stdout.decode(errors="replace").strip().splitlines()
        finally:
            ssh(["rm", "-rf", "--", str(remote_bootstrap)], check=False)
    after = local_runtime_snapshot()
    require(before == after, f"installed runtime changed: {before} != {after}")
    final_status = remote_status()
    require(final_status["d2a"] is True and final_status["state"] is True, "D2-A remote publication missing")
    require(final_status["failure"] is False, "D2-A failure evidence exists")
    require(sorted(final_status["markers"]) == sorted(MARKERS), "remote marker set mismatch")
    stage = pathlib.Path(f"{LOCAL_RESULT}.stage-{os.getpid()}-{time.time_ns()}")
    stage.mkdir(parents=True, mode=0o700)
    try:
        remote_evidence = stage / "REMOTE_EVIDENCE"
        scp_result = run(
            ["scp", "-q", "-p", "-r", f"{REMOTE}:{REMOTE_D2A}", str(remote_evidence)],
            check=False,
        )
        require(scp_result.returncode == 0, scp_result.stderr.decode(errors="replace")[-4000:])
        remote_entries = verify_sha256sums(remote_evidence)
        receipt = load_json(remote_evidence / "D2A_RECEIPT.json")
        require(receipt.get("verdict") == "D2A_CLOSED_ALL_MARKERS_AVAILABLE", "D2-A receipt verdict mismatch")
        require(receipt.get("markers_created") == 11 and receipt.get("markers_consumed") == 0, "D2-A marker ledger mismatch")
        require(receipt.get("cargo_invocations") == 0 and receipt.get("rustc_compilations") == 0, "D2-A compilation occurred")
        require(receipt.get("perf_record") == 0 and receipt.get("perf_stat") == 0, "D2-A perf occurred")
        require(receipt.get("d2_subject") == 0 and receipt.get("bucket_map_created") is False, "D2 subject or map occurred")
        shutil.copy2(remote_evidence / "D2A_RECEIPT.json", stage / "D2A_RECEIPT.json")
        write_new_json(
            stage / "LOCAL_D2A_RECEIPT.json",
            {
                "schema": "lay.v10.e1-traversal-d2-primary-only-local-d2a.v1",
                "task_id": TASK_ID,
                "verdict": "D2A_CLOSED_ALL_MARKERS_AVAILABLE",
                "controller_state": "PRIMARY_ONLY_CONTROLLER_VERIFIED_UNRUN",
                "controller": file_identity(CONTROLLER),
                "remote_receipt_sha256": sha256_file(remote_evidence / "D2A_RECEIPT.json"),
                "remote_manifest_entries": remote_entries,
                "remote_output": remote_output[-2:],
                "runtime_before": before,
                "runtime_after": after,
                "runtime_stable": True,
                "markers_expected": 11,
                "markers_created": 11,
                "markers_consumed": 0,
                "cargo_invocations": 0,
                "rustc_compilations": 0,
                "perf_record": 0,
                "perf_stat": 0,
                "d2_subject": 0,
                "bucket_map_created": False,
                "runtime_authority_changed": False,
                "stop_before_cargo": True,
            },
            0o444,
        )
        write_sha256sums(stage)
        seal_tree(stage)
        os.rename(stage, LOCAL_RESULT)
        fsync_directory(LOCAL_RESULT.parent)
    except Exception:
        remove_tree(stage)
        raise
    return {
        "verdict": "D2A_CLOSED_ALL_MARKERS_AVAILABLE",
        "controller_state": "PRIMARY_ONLY_CONTROLLER_VERIFIED_UNRUN",
        "local_result": str(LOCAL_RESULT),
        "markers_created": 11,
        "markers_consumed": 0,
        "cargo_invocations": 0,
        "perf_record": 0,
        "perf_stat": 0,
        "d2_subject": 0,
        "stop_before_cargo": True,
    }


def verify_execution_gate(route_id: str) -> dict[str, Any]:
    require(route_id in route_registry(), f"unknown route: {route_id}")
    require(LOCAL_RESULT.is_dir(), "sealed local D2-A PASS is absent")
    require(not any(path.stat().st_mode & 0o222 for path in LOCAL_RESULT.rglob("*")), "local D2-A evidence is writable")
    receipt = load_json(LOCAL_RESULT / "D2A_RECEIPT.json")
    require(receipt.get("verdict") == "D2A_CLOSED_ALL_MARKERS_AVAILABLE", "D2-A gate verdict mismatch")
    require(receipt.get("markers_consumed") == 0, "a D2 route marker is already consumed")
    status = remote_status()
    require(status["d2a"] is True and status["state"] is True, "remote D2-A gate missing")
    require(sorted(status["markers"]) == sorted(MARKERS), "remote D2-A marker state mismatch")
    return {"route_id": route_id, "d2a": receipt["verdict"], "marker_state": "all available"}


def execution_action(route_id: str) -> None:
    verify_execution_gate(route_id)
    raise ControllerError(
        "execution route is outside this D2-A-only implementation pass; independent D2-A audit and next admission are required"
    )


def parser() -> argparse.ArgumentParser:
    value = argparse.ArgumentParser()
    value.add_argument(
        "action",
        choices=("self-check", "d2a", "status", "remote-d2a", *ACTION_TO_ROUTE.keys()),
    )
    value.add_argument("argument", nargs="?")
    return value


def main() -> int:
    arguments = parser().parse_args()
    try:
        if arguments.action == "self-check":
            print(json.dumps(self_check(), ensure_ascii=False, sort_keys=True))
        elif arguments.action == "d2a":
            print(json.dumps(local_d2a(), ensure_ascii=False, sort_keys=True))
        elif arguments.action == "status":
            print(json.dumps(remote_status(), ensure_ascii=False, sort_keys=True))
        elif arguments.action == "remote-d2a":
            require(arguments.argument is not None, "remote-d2a requires bootstrap path")
            remote_d2a(pathlib.Path(arguments.argument))
        else:
            execution_action(ACTION_TO_ROUTE[arguments.action])
        return 0
    except Exception as error:
        print(f"D2 PRIMARY-ONLY ERROR: {error}", file=sys.stderr)
        return 1


if __name__ == "__main__":
    raise SystemExit(main())
