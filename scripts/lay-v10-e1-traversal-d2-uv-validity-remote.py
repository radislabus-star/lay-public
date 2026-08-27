#!/usr/bin/env python3
"""Remote one-shot U/V validity producer for primary-only D2."""

from __future__ import annotations

import base64
import contextlib
import fcntl
import hashlib
import json
import os
import pathlib
import re
import shutil
import stat
import struct
import subprocess
import sys
import time
from typing import Any, Mapping, Sequence


TASK_ID = "slice8b-v10-e1-traversal-d2-primary-only-v2-20260825"
TRANSACTION_ID = "35dbe5c5eaec7dbb5b5b112313a3285dbe2ceaea0a910719ad379255ca5569e7"
HOSTNAME = "e-MEGA-MINI-M1-13th"
MACHINE_ID_SHA256 = "5ac0bb537745673ef80942c2df32d586f5753c85de1c8b2453aa4e7317b77441"
PARENT = pathlib.Path("/home/e/.local/share/lay/provenance") / TASK_ID
STATE = pathlib.Path("/home/e/.local/state/lay") / TASK_ID
LOCK = STATE / "route.lock"
ELF = PARENT / "build-v1/d2-test-elf"
MAP = PARENT / "bucket-map-v1/D2_BUCKET_MAP.json"
PARITY_RESULT = PARENT / "parity-v1"
PARITY_STATE = STATE / "PARITY_STATE.json"
LOADER = pathlib.Path("/usr/lib/x86_64-linux-gnu/ld-linux-x86-64.so.2")

D1_RESULT = pathlib.Path(
    "/home/e/.local/share/lay/provenance/"
    "slice8b-v10-e1-remaining-cost-d1-20260825/result-v1"
)
B0A = pathlib.Path(
    "/home/e/.local/share/lay/provenance/"
    "slice8b-v10-hardware-b0-b2-v3-20260824/b0a-input-closure-v2"
)
B0B = pathlib.Path(
    "/home/e/.local/share/lay/provenance/"
    "slice8b-v10-hardware-b0-b2-v3-20260824/b0b-schedule-closure-v1"
)
PACKAGE = B0A / "inputs/LAY-L2-RU-FULL-v13.bin"
ARTIFACTS = B0A / "inputs/slice8b-v10-f6178f/artifacts"
SIDECAR = ARTIFACTS / "LAY-L2-RU-FULL-v13.dafsa"
V7 = ARTIFACTS / "slice8b-v7-fixed-13x100.json"
SCHEDULE = B0B / "query-schedule.json"

ELF_SHA256 = "bb7c68528ec3fec7074919c3c64cce233d48c26efefe9d9b32735c5f5865a178"
ELF_SIZE = 317_706_232
MAP_SHA256 = "2b93aedf11101a8bc5fc7ed4ffaa056b50476d96cdd2eabf9d185f59ee60f846"
PARITY_RECEIPT_SHA256 = "d2519797c6b21976dc6246830e187391a344acb9ec63a7b5a7816161cf306f74"
PREFLIGHT_SHA256 = "e9d6328b9f610ede73ae2e8d3c819b9728ac0fc5e4c263e9fe25a61978f80f5a"
PREFLIGHT_RECEIPT_SHA256 = "740c008d59fb4689826537e46a35da554bde863358d2c18382f315395ee835e0"
D1_CORRECTION_SHA256 = "004bc1f5d7cd493525cfb9287e79e8159f983b41a51a2374eaeb7931c72aad38"
U_SINGLE_SALVAGE_SHA256 = "9617502776537ca4181bd9bf195e1fd5b8fbd2679f1dfd00737f128cb88bfe0b"
U_SINGLE_HISTORICAL_SHA256 = "46d52ac863e25da861f803096a6918a47a1f4b7138c0167c1f4724ad7b26dac8"
D1_DECISION_SHA256 = "80530f9f5787f846ce2cf222c1b60e3ae42887ce95a11ac153ec7271cce98baf"
D1_STRUCTURE_SHA256 = "90d24adee563be803c390b41b18b41624b999db37b34c26650cb362f03d06712"
INPUTS = {
    PACKAGE: ("cce259fe0ce5dce67702383363b66f0fe9b9ff5a87d8f01c4fcf342d91218d7b", 140_556_462),
    SIDECAR: ("a1aa95be8c43fae8fedf02d3261dd1265dcbfbc8d618330fcd6449ae829df8cd", 3_689_884),
    V7: ("33fded73e13f565bdc08a83bd440be67c80f6d85a196453050b09bc9f7ef28e4", 1_606_189),
    SCHEDULE: ("2f8346d97246ac3704434279739fa5ed27705465d628890a367c21875d879e78", 174_941),
}

QUERIES = 382
ROUNDS = 20
REQUESTS = QUERIES * ROUNDS
EDGES_PER_ROUND = 25_145_756
EXAMINED_EDGES = EDGES_PER_ROUND * ROUNDS
PHASES = ("oracle", "lanes", "eqmask", "traversal", "merge", "certificate")
COMPONENT_SAMPLE = struct.Struct("<HHBB" + "Q" * 14)
G0_EVENTS = ("instructions", "cycles", "branches", "branch-misses")
PERF_STAT_PREFIX = (
    "/usr/bin/sudo",
    "-n",
    "/usr/bin/perf",
    "stat",
    "--json-output",
    "--no-big-num",
    "--delay=-1",
)

BASE_MARKERS = {
    "build.consumed-before-exec": ("d21b10eac837b740fa6cb9e84c75ff76bc5ec9dc388f0896575d6f697e0c2964", 478),
    "bucket-map.consumed-before-exec": ("4471cb4edbaa8c7fc7a2f7b2ffeb4894e9b66cc904d81c0e8b1122c506745bb7", 483),
    "parity.consumed-before-exec": ("ef5eef0d5ae91bea7bff2c1077cceb8c77d2f2d5a0e7263a70f044c648115c55", 479),
}
T_MARKERS = {
    "t-single.available": ("8f9e716a687622cd04f693350371228072c1a303d65834c6b647fd900322fe7b", 481),
    "t-fixed.available": ("7915c483243c7116f2d023895948667cda3708f23afa7bcc7abed614772f49b0", 480),
    "t-reversed.available": ("26eecf8ae977c0428a5371c30d85277e5974e2ec5768629ac95212cd8cd20c9e", 483),
}

U_SINGLE_TEST = "nanda_wave::l2_field::v13_typed_peak::tests::v10_d1_component_single"
U_TWENTY_TEST = "nanda_wave::l2_field::v13_typed_peak::tests::v10_d1_component_twenty"
V_TWENTY_TEST = "nanda_wave::l2_field::v13_typed_peak::tests::v10_d1_twenty_pmu"

ROUTE_ORDER = (
    "U-SINGLE",
    "U-FIXED",
    "U-REVERSED",
    "V-FIXED-INSTR",
    "V-REVERSED-INSTR",
)
ROUTES: dict[str, dict[str, Any]] = {
    "U-SINGLE": {
        "kind": "U",
        "marker": "u-single",
        "marker_sha256": "bb7b16f685e7c1a8818bc7185f0d6991f83183d8f035e90688fc66d83ba2a46b",
        "marker_size": 481,
        "test": U_SINGLE_TEST,
        "cpus": [0],
        "workers": 1,
        "d1_route": "C-SINGLE",
        "d1_wrapper_sha256": "3f698e4ebe7b1b22d757c24d2e35a8722a1b868d06674b11750096dcded6fded",
        "baseline_traversal_cpu_ns": 13_058_196_342,
        "pass_state": "U_SINGLE_PASS",
        "next_action": "U-FIXED only",
    },
    "U-FIXED": {
        "kind": "U",
        "marker": "u-fixed",
        "marker_sha256": "58435bf78041efe8a24191551c48062a9a0617d9ac9d468b918138b268ed27a8",
        "marker_size": 480,
        "test": U_TWENTY_TEST,
        "cpus": list(range(20)),
        "workers": 20,
        "d1_route": "C-FIXED",
        "d1_wrapper_sha256": "a5f21f12eb4ca4efa35063501f1e0935b16516a46d66112dd54217e497030c2c",
        "baseline_traversal_cpu_ns": 22_497_913_951,
        "pass_state": "U_FIXED_PASS",
        "next_action": "U-REVERSED only",
    },
    "U-REVERSED": {
        "kind": "U",
        "marker": "u-reversed",
        "marker_sha256": "c13f9e22ead22c1f3afe231848a49673c93690c88ba5bbaa1426e1f46507fe0e",
        "marker_size": 483,
        "test": U_TWENTY_TEST,
        "cpus": list(reversed(range(20))),
        "workers": 20,
        "d1_route": "C-REVERSED",
        "d1_wrapper_sha256": "0d8d649fbb8ad95d11d8129bb02c2c519e361b7c6887646b44319817434e8b6b",
        "baseline_traversal_cpu_ns": 22_478_678_187,
        "pass_state": "ALL_U_PASS",
        "next_action": "V-FIXED-INSTR only",
    },
    "V-FIXED-INSTR": {
        "kind": "V",
        "marker": "v-fixed-instr",
        "marker_sha256": "760e09bb85418e31732fcf24f93e8e81d683ecaded94e4afd29bc5d44c1c2f82",
        "marker_size": 486,
        "test": V_TWENTY_TEST,
        "cpus": list(range(20)),
        "workers": 20,
        "baseline_instructions_per_request": 23_934_876.5598414,
        "pass_state": "V_FIXED_PASS",
        "next_action": "V-REVERSED-INSTR only",
    },
    "V-REVERSED-INSTR": {
        "kind": "V",
        "marker": "v-reversed-instr",
        "marker_sha256": "a87b98f363b0c51f1a36896d1892cf0c508997ab91f05980480770c6601583dc",
        "marker_size": 489,
        "test": V_TWENTY_TEST,
        "cpus": list(reversed(range(20))),
        "workers": 20,
        "baseline_instructions_per_request": 23_935_583.225726895,
        "pass_state": "ALL_UV_VALIDITY_PASS",
        "next_action": "separate T-SINGLE controller only",
    },
}

U_DISPATCH = (
    ("provenance", "BLOCKED_PROVENANCE"),
    ("thermal", "BLOCKED_THERMAL"),
    ("semantic", "BLOCKED_SEMANTIC"),
    ("perturbation", "BLOCKED_PERTURBATION"),
)
V_DISPATCH = (
    ("provenance", "BLOCKED_PROVENANCE"),
    ("thermal", "BLOCKED_THERMAL"),
    ("capability", "BLOCKED_CAPABILITY"),
    ("denominator", "BLOCKED_DENOMINATOR"),
    ("perturbation", "BLOCKED_PERTURBATION"),
)


class UVError(RuntimeError):
    pass


def need(condition: bool, message: str) -> None:
    if not condition:
        raise UVError(message)


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


def row(path: pathlib.Path) -> dict[str, Any]:
    need(path.is_file() and not path.is_symlink(), f"missing or invalid file: {path}")
    return {
        "path": str(path),
        "mode": mode_string(path),
        "size_bytes": path.stat().st_size,
        "sha256": sha256_file(path),
    }


def write_new_bytes(path: pathlib.Path, value: bytes, mode: int = 0o444) -> None:
    descriptor = os.open(path, os.O_WRONLY | os.O_CREAT | os.O_EXCL, 0o600)
    try:
        view = memoryview(value)
        while view:
            written = os.write(descriptor, view)
            need(written > 0, "short write made no progress")
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


def inventory(root: pathlib.Path) -> list[dict[str, Any]]:
    values = []
    for path in sorted(root.rglob("*")):
        need(not path.is_symlink(), f"symlink in evidence: {path}")
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
    values = [value for value in inventory(root) if value["path"] != "SHA256SUMS"]
    write_new_bytes(
        root / "SHA256SUMS",
        "".join(f"{value['sha256']}  {value['path']}\n" for value in values).encode(),
    )


def verify_sha256sums(root: pathlib.Path) -> int:
    manifest = root / "SHA256SUMS"
    need(manifest.is_file(), f"missing SHA256SUMS: {root}")
    seen: set[str] = set()
    for line in manifest.read_text().splitlines():
        digest, separator, relative = line.partition("  ")
        need(separator == "  " and len(digest) == 64, f"bad manifest row: {line}")
        pure = pathlib.PurePosixPath(relative)
        need(not pure.is_absolute() and ".." not in pure.parts and relative not in seen, f"unsafe manifest row: {relative}")
        seen.add(relative)
        need(sha256_file(root / pure) == digest, f"manifest mismatch: {relative}")
    actual = {value["path"] for value in inventory(root) if value["path"] != "SHA256SUMS"}
    need(seen == actual, f"manifest membership mismatch: {root}")
    return len(seen)


def seal_tree(root: pathlib.Path) -> None:
    for path in sorted(root.rglob("*"), reverse=True):
        need(not path.is_symlink(), f"symlink before seal: {path}")
        path.chmod(0o555 if path.is_dir() or path.stat().st_mode & 0o111 else 0o444)
    root.chmod(0o555)


def fsync_directory(path: pathlib.Path) -> None:
    descriptor = os.open(path, os.O_RDONLY | os.O_DIRECTORY)
    try:
        os.fsync(descriptor)
    finally:
        os.close(descriptor)


def read_text(path: pathlib.Path) -> str | None:
    with contextlib.suppress(OSError):
        return path.read_text(encoding="utf-8", errors="replace").strip()
    return None


def pressure(path: str) -> dict[str, dict[str, float]]:
    result: dict[str, dict[str, float]] = {}
    for line in (read_text(pathlib.Path(path)) or "").splitlines():
        fields = line.split()
        result[fields[0]] = {
            key: float(value) for key, value in (field.split("=", 1) for field in fields[1:])
        }
    return result


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


def host_snapshot() -> dict[str, Any]:
    thermal = temperatures()
    return {
        "monotonic_ns": time.perf_counter_ns(),
        "loadavg": read_text(pathlib.Path("/proc/loadavg")),
        "cpu_pressure": pressure("/proc/pressure/cpu"),
        "io_pressure": pressure("/proc/pressure/io"),
        "memory_pressure": pressure("/proc/pressure/memory"),
        "temperatures": thermal,
        "maximum_temperature_c": max(
            (value["millidegrees_c"] for value in thermal), default=0
        )
        / 1000.0,
        "throttle_counters": throttle_counters(),
        "online_cpus": read_text(pathlib.Path("/sys/devices/system/cpu/online")),
    }


def thermal_drift(before: Mapping[str, Any], after: Mapping[str, Any]) -> dict[str, list[int]]:
    result = {}
    left = before["throttle_counters"]
    right = after["throttle_counters"]
    for key in sorted(set(left) | set(right)):
        if left.get(key) != right.get(key):
            result[key] = [left.get(key, 0), right.get(key, 0)]
    return result


def route_slug(route: str) -> str:
    return route.lower().replace("-", "_")


def result_path(route: str) -> pathlib.Path:
    return PARENT / f"uv-{route.lower()}-v1"


def failure_path(route: str) -> pathlib.Path:
    return PARENT / f"uv-{route.lower()}-failure-v1"


def state_path(route: str) -> pathlib.Path:
    return STATE / f"{route_slug(route).upper()}_STATE.json"


def receipt_path(route: str) -> pathlib.Path:
    return result_path(route) / "D2_UV_ROUTE_RECEIPT.json"


def decode_input(payload: Mapping[str, Any], key: str, digest: str) -> bytes:
    value = base64.b64decode(payload.get(key, ""), validate=True)
    need(sha256_bytes(value) == digest, f"payload SHA drift: {key}")
    return value


def marker_projection() -> list[dict[str, Any]]:
    return [
        {**row(path), "name": path.name, "value": json.loads(path.read_text())}
        for path in sorted((STATE / "markers").iterdir())
    ]


def expected_markers(route: str, *, post: bool) -> dict[str, tuple[str, int]]:
    index = ROUTE_ORDER.index(route)
    expected = dict(BASE_MARKERS)
    for position, name in enumerate(ROUTE_ORDER):
        spec = ROUTES[name]
        suffix = "consumed-before-exec" if position < index or (post and position == index) else "available"
        expected[f"{spec['marker']}.{suffix}"] = (spec["marker_sha256"], spec["marker_size"])
    expected.update(T_MARKERS)
    return expected


def verify_markers(route: str, *, post: bool) -> list[dict[str, Any]]:
    expected = expected_markers(route, post=post)
    values = marker_projection()
    observed = {value["name"]: value for value in values}
    need(set(observed) == set(expected), f"marker membership drift: {sorted(observed)}")
    for name, (digest, size) in expected.items():
        value = observed[name]
        need(
            value["sha256"] == digest and value["size_bytes"] == size and value["mode"] == "0400",
            f"marker identity drift: {name}",
        )
        body = value["value"]
        need(body.get("task_id") == TASK_ID, f"marker task drift: {name}")
        need(body.get("transaction_id") == TRANSACTION_ID, f"marker transaction drift: {name}")
        need(body.get("retry_permitted") is False, f"marker retry drift: {name}")
    return values


def verify_predecessor_chain(
    route: str, *, post: bool, u_single_salvage: Mapping[str, Any]
) -> None:
    parity = json.loads((PARITY_RESULT / "D2_PARITY_RECEIPT.json").read_text())
    need(sha256_file(PARITY_RESULT / "D2_PARITY_RECEIPT.json") == PARITY_RECEIPT_SHA256, "parity receipt drift")
    need(parity.get("verdict") == "D2_PARITY_PASS", "parity verdict drift")
    parity_state = json.loads(PARITY_STATE.read_text())
    need(
        parity_state.get("state") == "D2_PARITY_PASS"
        and parity_state.get("receipt_sha256") == PARITY_RECEIPT_SHA256,
        "parity state drift",
    )
    current = ROUTE_ORDER.index(route)
    for index, name in enumerate(ROUTE_ORDER):
        result = result_path(name)
        failure = failure_path(name)
        state_file = state_path(name)
        should_exist = index < current or (post and index == current)
        if should_exist:
            need(result.is_dir() and not failure.exists() and state_file.is_file(), f"route projection missing: {name}")
            verify_sha256sums(result)
            receipt = json.loads(receipt_path(name).read_text())
            state_value = json.loads(state_file.read_text())
            need(receipt.get("route") == name, f"route receipt identity drift: {name}")
            need(state_value.get("receipt_sha256") == sha256_file(receipt_path(name)), f"route state receipt drift: {name}")
            if index < current:
                if name == "U-SINGLE" and receipt.get("verdict") == "BLOCKED_SEMANTIC":
                    need(sha256_file(receipt_path(name)) == U_SINGLE_HISTORICAL_SHA256, "historical U-SINGLE receipt drift")
                    need(state_value.get("state") == "BLOCKED_SEMANTIC", "historical U-SINGLE state drift")
                    need(
                        u_single_salvage.get("verdict")
                        == "U_SINGLE_RECOVERED_FROM_SEALED_EVIDENCE"
                        and u_single_salvage.get("effective_route_state") == "U_SINGLE_PASS"
                        and u_single_salvage.get("historical_receipt_sha256")
                        == U_SINGLE_HISTORICAL_SHA256
                        and u_single_salvage.get("historical_state_unchanged") is True
                        and u_single_salvage.get("retry_permitted") is False,
                        "U-SINGLE salvage overlay drift",
                    )
                else:
                    need(receipt.get("verdict") == ROUTES[name]["pass_state"], f"prior route not PASS: {name}")
                    need(state_value.get("state") == ROUTES[name]["pass_state"], f"prior route state not PASS: {name}")
        else:
            need(not result.exists() and not failure.exists() and not state_file.exists(), f"future route already exists: {name}")


def verify_common(payload: Mapping[str, Any], route: str, *, post: bool) -> dict[str, Any]:
    need(route in ROUTES, f"unknown route: {route}")
    need(os.uname().nodename == HOSTNAME, "hostname drift")
    need(sha256_file(pathlib.Path("/etc/machine-id")) == MACHINE_ID_SHA256, "machine identity drift")
    preflight = json.loads(decode_input(payload, "preflight_b64", PREFLIGHT_SHA256))
    preflight_receipt = json.loads(
        decode_input(payload, "preflight_receipt_b64", PREFLIGHT_RECEIPT_SHA256)
    )
    correction = json.loads(decode_input(payload, "d1_correction_b64", D1_CORRECTION_SHA256))
    u_single_salvage = json.loads(
        decode_input(payload, "u_single_salvage_b64", U_SINGLE_SALVAGE_SHA256)
    )
    need(preflight.get("scoped_positive_verdict") == "READY_TO_IMPLEMENT_PRIMARY_ONLY_D2", "preflight verdict drift")
    need(preflight_receipt.get("verdict") == "READY_TO_IMPLEMENT", "preflight receipt drift")
    need(preflight_receipt.get("safe_to_implement") is True, "preflight implementation authority drift")
    need(
        correction.get("verdict") == "D1_PMU_INTERPRETATION_PASS_FROM_SEALED_EVIDENCE",
        "D1 correction verdict drift",
    )
    need(row(ELF)["sha256"] == ELF_SHA256 and ELF.stat().st_size == ELF_SIZE and mode_string(ELF) == "0555", "D2 ELF drift")
    need(row(MAP)["sha256"] == MAP_SHA256 and mode_string(MAP) == "0444", "D2 bucket map drift")
    need(LOADER.is_file(), "ELF loader missing")
    need(sha256_file(D1_RESULT / "D1_DECISION.json") == D1_DECISION_SHA256, "D1 decision drift")
    need(
        sha256_file(D1_RESULT / "C-SINGLE/subject/structure.json") == D1_STRUCTURE_SHA256,
        "D1 structure drift",
    )
    for name in ("C-FIXED", "C-REVERSED"):
        need(
            sha256_file(D1_RESULT / f"{name}/subject/structure.json") == D1_STRUCTURE_SHA256,
            f"D1 structure drift: {name}",
        )
    for path, (digest, size) in INPUTS.items():
        value = row(path)
        need(value["sha256"] == digest and value["size_bytes"] == size and value["mode"] == "0444", f"input drift: {path}")
    verify_predecessor_chain(route, post=post, u_single_salvage=u_single_salvage)
    markers = verify_markers(route, post=post)
    return {
        "hostname": os.uname().nodename,
        "machine_id_sha256": MACHINE_ID_SHA256,
        "route": route,
        "elf": row(ELF),
        "map": row(MAP),
        "parity_receipt_sha256": PARITY_RECEIPT_SHA256,
        "d1_decision_sha256": D1_DECISION_SHA256,
        "d1_correction_sha256": D1_CORRECTION_SHA256,
        "u_single_salvage_sha256": U_SINGLE_SALVAGE_SHA256,
        "markers": markers,
        "remote_writes": 0,
    }


def controlled_environment(output: pathlib.Path, control: pathlib.Path | None, route: str) -> dict[str, str]:
    value = {
        "HOME": "/home/e",
        "PATH": "/home/e/.cargo/bin:/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin",
        "LANG": "C.UTF-8",
        "LC_ALL": "C.UTF-8",
        "TZ": "Europe/Tallinn",
        "LAY_V10_D1_PACKAGE": str(PACKAGE),
        "LAY_V10_D1_SIDECAR": str(SIDECAR),
        "LAY_V10_D1_V7": str(V7),
        "LAY_V10_D1_SCHEDULE": str(SCHEDULE),
        "LAY_V10_D1_OUTPUT": str(output),
        "LAY_V10_D1_RUN_ID": route,
        "LAY_V10_D1_CPUS": ",".join(map(str, ROUTES[route]["cpus"])),
    }
    if control is not None:
        value["LAY_V10_D1_CONTROL_DIR"] = str(control)
    return value


def subject_command(route: str) -> list[str]:
    return [
        str(LOADER),
        str(ELF),
        "--exact",
        ROUTES[route]["test"],
        "--ignored",
        "--nocapture",
        "--test-threads=1",
    ]


def child_as_e(environment: Mapping[str, str], command: Sequence[str]) -> list[str]:
    assignments = [f"{key}={value}" for key, value in sorted(environment.items())]
    return ["/usr/bin/sudo", "-n", "-u", "e", "/usr/bin/env", *assignments, *command]


def consume_marker(route: str) -> dict[str, Any]:
    spec = ROUTES[route]
    available = STATE / f"markers/{spec['marker']}.available"
    consumed = STATE / f"markers/{spec['marker']}.consumed-before-exec"
    before = row(available)
    need(before["sha256"] == spec["marker_sha256"], f"marker SHA drift: {route}")
    need(before["size_bytes"] == spec["marker_size"], f"marker size drift: {route}")
    need(not consumed.exists(), f"marker already consumed: {route}")
    os.rename(available, consumed)
    fsync_directory(available.parent)
    after = row(consumed)
    need(after["sha256"] == before["sha256"] and after["size_bytes"] == before["size_bytes"], "marker rename drift")
    return {"before": before, "after": after, "consumed_before_effect": True}


def parse_component_samples(path: pathlib.Path) -> dict[str, Any]:
    raw = path.read_bytes()
    need(len(raw) % COMPONENT_SAMPLE.size == 0, "component sample width mismatch")
    traversal_cpu_ns = 0
    errors = 0
    unresolved = 0
    ordinals: set[int] = set()
    rounds: set[int] = set()
    workers: set[int] = set()
    for offset in range(0, len(raw), COMPONENT_SAMPLE.size):
        values = COMPONENT_SAMPLE.unpack_from(raw, offset)
        ordinals.add(values[0])
        rounds.add(values[1])
        workers.add(values[2])
        errors += int(bool(values[3] & 1))
        unresolved += int(bool(values[3] & 2))
        traversal_cpu_ns += values[13]
    return {
        "sample_width_bytes": COMPONENT_SAMPLE.size,
        "records": len(raw) // COMPONENT_SAMPLE.size,
        "query_ordinals": sorted(ordinals),
        "rounds": sorted(rounds),
        "workers": sorted(workers),
        "errors": errors,
        "unresolved": unresolved,
        "traversal_thread_cpu_ns": traversal_cpu_ns,
    }


def validate_structure(path: pathlib.Path) -> dict[str, Any]:
    value = json.loads(path.read_text())
    queries = value.get("queries", [])
    fields = (
        "retrieval_lanes",
        "eqmask_builds",
        "expanded_states",
        "examined_edges",
        "surviving_edges",
        "pruned_edges",
        "stack_pushes",
        "stack_pops",
        "terminal_hits_before_merge",
        "terminal_refs_after_merge",
        "certificate_calls",
        "materialized_peaks",
    )
    totals = {field: sum(int(item.get(field, 0)) for item in queries) for field in fields}
    return {
        "sha256": sha256_file(path),
        "queries": len(queries),
        "totals_per_round": totals,
        "exact_d1_structure": sha256_file(path) == D1_STRUCTURE_SHA256,
    }


def validate_u_receipt(receipt: Mapping[str, Any], route: str) -> list[str]:
    spec = ROUTES[route]
    violations = []
    expected_mapping = "SINGLE" if route == "U-SINGLE" else ("FIXED" if route == "U-FIXED" else "REVERSED")
    checks = {
        "schema": receipt.get("schema") == "lay.v10.e1-remaining-cost-d1-component-process.v1",
        "test": receipt.get("test") == spec["test"],
        "run_id": receipt.get("run_id") == route,
        "mapping": receipt.get("mapping") == expected_mapping,
        "rounds": receipt.get("rounds") == ROUNDS,
        "queries": receipt.get("queries") == QUERIES,
        "workers": receipt.get("workers") == spec["workers"],
        "cpus": receipt.get("cpus") == spec["cpus"],
        "samples": receipt.get("samples", {}).get("samples") == REQUESTS,
        "sample_bytes": receipt.get("samples", {}).get("sample_bytes") == COMPONENT_SAMPLE.size,
        "phase_order": receipt.get("samples", {}).get("phase_order") == list(PHASES),
        "errors": receipt.get("samples", {}).get("errors") == 0,
        "unresolved": receipt.get("samples", {}).get("unresolved") == 0,
    }
    if route != "U-SINGLE":
        checks.update(
            {
                "warmup_bursts": receipt.get("warmup_bursts") == 1,
                "worker_affinities": receipt.get("worker_affinities") == [[cpu] for cpu in spec["cpus"]],
                "worker_migrations": receipt.get("worker_migration_deltas") == [0] * spec["workers"],
            }
        )
    else:
        checks.update(
            {
                "warmup_queries": receipt.get("warmup_queries") == QUERIES,
                "thread_migrations": receipt.get("thread_migrations") == 0,
            }
        )
    for name, passed in checks.items():
        if not passed:
            violations.append(f"subject receipt {name} mismatch")
    return violations


def run_u(route: str, stage: pathlib.Path, environment: dict[str, str]) -> dict[str, Any]:
    spec = ROUTES[route]
    subject = stage / "subject"
    before = host_snapshot()
    started_ns = time.perf_counter_ns()
    process = subprocess.run(
        subject_command(route),
        env=environment,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        check=False,
        timeout=3600,
    )
    ended_ns = time.perf_counter_ns()
    after = host_snapshot()
    write_new_bytes(stage / "stdout.log", process.stdout)
    write_new_bytes(stage / "stderr.log", process.stderr)
    semantic: list[str] = []
    complete = True
    details: dict[str, Any] = {}
    try:
        receipt_file = subject / "SUBJECT_RECEIPT.json"
        samples_file = subject / "component-samples.bin"
        structure_file = subject / "structure.json"
        need(receipt_file.is_file() and samples_file.is_file() and structure_file.is_file(), "U output set incomplete")
        receipt = json.loads(receipt_file.read_text())
        samples = parse_component_samples(samples_file)
        structure = validate_structure(structure_file)
        semantic.extend(validate_u_receipt(receipt, route))
        if process.returncode != 0:
            semantic.append(f"subject exit {process.returncode}")
        if samples["records"] != REQUESTS:
            semantic.append("component record denominator mismatch")
        if samples["query_ordinals"] != list(range(QUERIES)):
            semantic.append("component query ordinal mismatch")
        if samples["rounds"] != list(range(ROUNDS)):
            semantic.append("component round mismatch")
        expected_sample_workers = [255] if route == "U-SINGLE" else list(range(spec["workers"]))
        if samples["workers"] != expected_sample_workers:
            semantic.append("component worker coverage mismatch")
        if samples["errors"] != 0 or samples["unresolved"] != 0:
            semantic.append("component errors or unresolved nonzero")
        if not structure["exact_d1_structure"]:
            semantic.append("structural mismatch")
        if structure["queries"] != QUERIES:
            semantic.append("structure denominator mismatch")
        if structure["totals_per_round"]["examined_edges"] != EDGES_PER_ROUND:
            semantic.append("examined-edge structure mismatch")
        observed_cpu = samples["traversal_thread_cpu_ns"]
        baseline_cpu = spec["baseline_traversal_cpu_ns"]
        observed_per_edge = observed_cpu / EXAMINED_EDGES
        baseline_per_edge = baseline_cpu / EXAMINED_EDGES
        delta_percent = abs(observed_per_edge - baseline_per_edge) / baseline_per_edge * 100.0
        details = {
            "subject_receipt": row(receipt_file),
            "component_samples": row(samples_file),
            "structure": row(structure_file),
            "receipt": receipt,
            "sample_summary": samples,
            "structure_summary": structure,
            "baseline_d1_route": spec["d1_route"],
            "baseline_d1_wrapper": row(D1_RESULT / f"{spec['d1_route']}/COMPONENT_WRAPPER.json"),
            "examined_edges": EXAMINED_EDGES,
            "baseline_traversal_thread_cpu_ns": baseline_cpu,
            "observed_traversal_thread_cpu_ns": observed_cpu,
            "baseline_traversal_thread_cpu_per_edge_ns": baseline_per_edge,
            "observed_traversal_thread_cpu_per_edge_ns": observed_per_edge,
            "absolute_delta_percent": delta_percent,
        }
    except Exception as error:
        complete = False
        semantic.append(f"incomplete U observation: {type(error).__name__}: {error}")
    drift = thermal_drift(before, after)
    perturbation = []
    if complete and details["absolute_delta_percent"] > 5.0:
        perturbation.append(f"CPU/edge delta {details['absolute_delta_percent']:.9f}% exceeds 5%")
    return {
        "complete": complete,
        "kind": "U",
        "route": route,
        "command": subject_command(route),
        "environment": environment,
        "exit_code": process.returncode,
        "process_wall_ns_diagnostic": ended_ns - started_ns,
        "environment_before": before,
        "environment_after": after,
        "details": details,
        "violations": {
            "provenance": [] if complete else ["complete sealed U observation unavailable"],
            "thermal": [f"thermal throttle drift: {drift}"] if drift else [],
            "semantic": semantic,
            "perturbation": perturbation,
        },
        "perf_stat_invocations": 0,
        "pmu_events_opened": 0,
    }


def wait_for_file(process: subprocess.Popen[bytes], path: pathlib.Path, timeout: float) -> None:
    deadline = time.monotonic() + timeout
    while time.monotonic() < deadline:
        if path.is_file():
            return
        need(process.poll() is None, f"perf/subject exited before {path.name}")
        time.sleep(0.002)
    raise UVError(f"timeout waiting for {path}")


def open_fifo(path: pathlib.Path, flags: int, deadline: float) -> int:
    while time.monotonic() < deadline:
        try:
            return os.open(path, flags | os.O_NONBLOCK)
        except OSError as error:
            if error.errno not in (6, 11):
                raise
            time.sleep(0.005)
    raise UVError(f"FIFO open timeout: {path}")


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
    raise UVError("perf control acknowledgement timeout")


def terminate_owned(process: subprocess.Popen[bytes] | None) -> None:
    if process is None or process.poll() is not None:
        return
    process.terminate()
    with contextlib.suppress(subprocess.TimeoutExpired):
        process.wait(timeout=3)
    if process.poll() is None:
        process.kill()
        process.wait()


def numeric_counter(value: Any) -> float | None:
    if isinstance(value, (int, float)):
        return float(value)
    if not isinstance(value, str):
        return None
    compact = value.strip().replace(",", "")
    if compact.startswith("<") or compact in ("", "not counted", "not supported"):
        return None
    try:
        return float(compact)
    except ValueError:
        return None


def event_identity(value: str) -> tuple[str, str] | None:
    match = re.fullmatch(r"cpu_(atom|core)/([^/]+)/", value.strip().lower().replace(":u", ""))
    if match is None:
        return None
    return match.group(1), match.group(2)


def parse_hybrid_g0(raw: bytes) -> dict[str, Any]:
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
    capability: list[str] = []
    denominator: list[str] = []
    aggregates: dict[str, Any] = {}
    if not rows:
        capability.append("perf produced no JSON counter rows")
    identities = [event_identity(str(item.get("event", ""))) for item in rows]
    if any(value is None for value in identities):
        denominator.append("foreign or ambiguous PMU row")
    if len(rows) != len(G0_EVENTS) * 2:
        denominator.append(f"expected 8 G0 rows, observed {len(rows)}")
    for event in G0_EVENTS:
        matched = [
            (identity, item)
            for identity, item in zip(identities, rows)
            if identity is not None and identity[1] == event
        ]
        if not matched:
            capability.append(f"event unavailable: {event}")
            continue
        owners = [identity[0] for identity, _ in matched]
        if sorted(owners) != ["atom", "core"] or len(matched) != 2:
            denominator.append(f"hybrid owner coverage mismatch: {event}: {owners}")
            continue
        parsed = []
        unsupported = False
        for identity, item in matched:
            counter = numeric_counter(item.get("counter-value"))
            runtime = numeric_counter(item.get("event-runtime"))
            running = numeric_counter(item.get("pcnt-running"))
            if counter is None or runtime is None or runtime <= 0 or running is None:
                capability.append(f"uncounted or inactive event row: {event}/{identity[0]}")
                unsupported = True
                continue
            parsed.append(
                {
                    "pmu": identity[0],
                    "scaled_count": counter,
                    "event_runtime": runtime,
                    "pcnt_running": running,
                    "row": item,
                }
            )
        if unsupported or len(parsed) != 2:
            continue
        total_runtime = sum(item["event_runtime"] for item in parsed)
        running_sum = sum(item["pcnt_running"] for item in parsed)
        if not 98.9 <= running_sum <= 101.1:
            denominator.append(f"running-percent partition mismatch: {event}: {running_sum}")
        for item in parsed:
            runtime_percent = 100.0 * item["event_runtime"] / total_runtime
            if abs(item["pcnt_running"] - runtime_percent) > 1.1:
                denominator.append(
                    f"runtime share mismatch: {event}/{item['pmu']}: "
                    f"{item['pcnt_running']} vs {runtime_percent}"
                )
        effective = sum(
            item["scaled_count"] * item["event_runtime"] / total_runtime for item in parsed
        )
        aggregates[event] = {
            "effective_count": effective,
            "per_request": effective / REQUESTS,
            "per_examined_edge": effective / EXAMINED_EDGES,
            "runtime_total": total_runtime,
            "running_percent_sum": running_sum,
            "rows": parsed,
            "aggregation": "sum(scaled_count_i * event_runtime_i / sum(event_runtime))",
        }
    return {
        "rows": rows,
        "diagnostics": diagnostics,
        "aggregates": aggregates,
        "capability_violations": capability,
        "denominator_violations": denominator,
    }


def validate_v_receipt(receipt: Mapping[str, Any], route: str) -> list[str]:
    spec = ROUTES[route]
    mapping = "FIXED" if route == "V-FIXED-INSTR" else "REVERSED"
    checks = {
        "schema": receipt.get("schema") == "lay.v10.e1-remaining-cost-d1-twenty-pmu.v1",
        "verdict": receipt.get("verdict") == "PASS",
        "test": receipt.get("test") == spec["test"],
        "run_id": receipt.get("run_id") == route,
        "mapping": receipt.get("mapping") == mapping,
        "rounds": receipt.get("rounds") == ROUNDS,
        "queries_per_round": receipt.get("queries_per_round") == QUERIES,
        "measured_requests": receipt.get("measured_requests") == REQUESTS,
        "warmup_requests": receipt.get("warmup_requests") == QUERIES,
        "workers": receipt.get("workers") == 20,
        "cpus": receipt.get("cpus") == spec["cpus"],
        "worker_affinities": receipt.get("worker_affinities") == [[cpu] for cpu in spec["cpus"]],
        "worker_migrations": receipt.get("worker_migration_deltas") == [0] * 20,
        "errors": receipt.get("errors") == 0,
        "unresolved": receipt.get("unresolved") == 0,
        "examined_edges": receipt.get("examined_edges") == EXAMINED_EDGES,
        "component_clocks": receipt.get("component_clocks_enabled") is False,
        "control_protocol": receipt.get("control_protocol")
        == ["subject-ready", "controller-enabled", "subject-done", "controller-disabled"],
    }
    return [f"subject producer {name} mismatch" for name, passed in checks.items() if not passed]


def run_v(route: str, stage: pathlib.Path, environment: dict[str, str]) -> dict[str, Any]:
    subject = stage / "subject"
    control = stage / "control"
    control_fifo = stage / "perf-control.fifo"
    ack_fifo = stage / "perf-ack.fifo"
    child = child_as_e(environment, subject_command(route))
    command = [
        *PERF_STAT_PREFIX,
        f"--control=fifo:{control_fifo},{ack_fifo}",
        "--event",
        ",".join(G0_EVENTS),
        "--",
        *child,
    ]
    before = host_snapshot()
    process: subprocess.Popen[bytes] | None = None
    control_fd: int | None = None
    ack_fd: int | None = None
    stdout = b""
    stderr = b""
    lifecycle: dict[str, Any] = {
        "subject_ready": False,
        "enable_ack": None,
        "subject_done": False,
        "disable_ack": None,
        "controller_enabled": False,
        "controller_disabled": False,
    }
    controller_errors: list[str] = []
    started_ns = time.perf_counter_ns()
    try:
        process = subprocess.Popen(
            command,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            start_new_session=True,
        )
        wait_for_file(process, control / "subject-ready", 3600.0)
        lifecycle["subject_ready"] = True
        deadline = time.monotonic() + 10.0
        control_fd = open_fifo(control_fifo, os.O_WRONLY, deadline)
        ack_fd = open_fifo(ack_fifo, os.O_RDONLY, deadline)
        os.write(control_fd, b"enable\n")
        lifecycle["enable_ack"] = read_fifo_line(ack_fd, time.monotonic() + 10.0)
        write_new_bytes(control / "controller-enabled", b"enabled\n")
        lifecycle["controller_enabled"] = True
        wait_for_file(process, control / "subject-done", 1200.0)
        lifecycle["subject_done"] = True
        os.write(control_fd, b"disable\n")
        lifecycle["disable_ack"] = read_fifo_line(ack_fd, time.monotonic() + 10.0)
        write_new_bytes(control / "controller-disabled", b"disabled\n")
        lifecycle["controller_disabled"] = True
        stdout, stderr = process.communicate(timeout=300)
    except Exception as error:
        controller_errors.append(f"{type(error).__name__}: {error}")
        terminate_owned(process)
        if process is not None:
            with contextlib.suppress(Exception):
                more_out, more_err = process.communicate(timeout=1)
                stdout += more_out
                stderr += more_err
    finally:
        if control_fd is not None:
            os.close(control_fd)
        if ack_fd is not None:
            os.close(ack_fd)
    ended_ns = time.perf_counter_ns()
    after = host_snapshot()
    write_new_bytes(stage / "stdout.log", stdout)
    write_new_bytes(stage / "perf.raw", stderr)
    process_exit = None if process is None else process.returncode
    capability: list[str] = []
    denominator: list[str] = []
    complete = True
    receipt_identity = None
    receipt_value: dict[str, Any] | None = None
    parsed: dict[str, Any] | None = None
    if process is None:
        complete = False
    elif process_exit != 0:
        capability.append(f"perf stat exited {process_exit}")
    if controller_errors:
        if process_exit is not None and not lifecycle["subject_ready"]:
            capability.extend(controller_errors)
        else:
            complete = False
    receipt_file = subject / "SUBJECT_RECEIPT.json"
    if receipt_file.is_file():
        try:
            receipt_value = json.loads(receipt_file.read_text())
            receipt_identity = row(receipt_file)
            denominator.extend(validate_v_receipt(receipt_value, route))
        except Exception as error:
            complete = False
            denominator.append(f"subject receipt unreadable: {type(error).__name__}: {error}")
    elif not capability:
        complete = False
        denominator.append("subject receipt missing")
    try:
        parsed = parse_hybrid_g0(stderr)
        capability.extend(parsed["capability_violations"])
        denominator.extend(parsed["denominator_violations"])
    except Exception as error:
        complete = False
        denominator.append(f"perf parser failed: {type(error).__name__}: {error}")
    lifecycle_complete = all(
        (
            lifecycle["subject_ready"],
            lifecycle["controller_enabled"],
            lifecycle["subject_done"],
            lifecycle["controller_disabled"],
            bool(lifecycle["enable_ack"]),
            bool(lifecycle["disable_ack"]),
        )
    )
    if not lifecycle_complete and not capability:
        complete = False
        denominator.append("FIFO lifecycle incomplete")
    instructions_per_request = None
    delta_percent = None
    if parsed is not None and "instructions" in parsed["aggregates"]:
        instructions_per_request = parsed["aggregates"]["instructions"]["per_request"]
        baseline = ROUTES[route]["baseline_instructions_per_request"]
        delta_percent = abs(instructions_per_request - baseline) / baseline * 100.0
    elif not capability:
        complete = False
    perturbation = []
    if delta_percent is not None and delta_percent > 1.0:
        perturbation.append(f"instructions/request delta {delta_percent:.9f}% exceeds 1%")
    drift = thermal_drift(before, after)
    return {
        "complete": complete,
        "kind": "V",
        "route": route,
        "command": command,
        "subject_command": subject_command(route),
        "subject_child_as_e": child,
        "environment": environment,
        "exit_code": process_exit,
        "process_wall_ns_diagnostic": ended_ns - started_ns,
        "lifecycle": lifecycle,
        "controller_errors": controller_errors,
        "environment_before": before,
        "environment_after": after,
        "details": {
            "subject_receipt": receipt_identity,
            "receipt": receipt_value,
            "perf_raw": row(stage / "perf.raw"),
            "parsed_g0": parsed,
            "requests": REQUESTS,
            "baseline_instructions_per_request": ROUTES[route]["baseline_instructions_per_request"],
            "observed_instructions_per_request": instructions_per_request,
            "absolute_delta_percent": delta_percent,
        },
        "violations": {
            "provenance": [] if complete else ["complete sealed V observation unavailable"],
            "thermal": [f"thermal throttle drift: {drift}"] if drift else [],
            "capability": capability,
            "denominator": denominator,
            "perturbation": perturbation,
        },
        "perf_stat_invocations": 1,
        "pmu_events_opened": 1 if lifecycle["enable_ack"] else 0,
    }


def dispatch_observation(observation: Mapping[str, Any], route: str) -> dict[str, Any]:
    priority = U_DISPATCH if ROUTES[route]["kind"] == "U" else V_DISPATCH
    violations = observation.get("violations", {})
    selected = None
    for rank, (cause, terminal) in enumerate(priority):
        reasons = violations.get(cause)
        if not isinstance(reasons, list):
            return {
                "selected_cause": "provenance",
                "selected_rank": 0,
                "verdict": "BLOCKED_PROVENANCE",
                "reason": f"dispatch schema missing or invalid: {cause}",
                "all_violations": violations,
            }
        if reasons and selected is None:
            selected = {
                "selected_cause": cause,
                "selected_rank": rank,
                "verdict": terminal,
                "reason": reasons[0],
                "all_violations": violations,
            }
    if selected is not None:
        return selected
    return {
        "selected_cause": None,
        "selected_rank": None,
        "verdict": ROUTES[route]["pass_state"],
        "reason": "all frozen route predicates passed",
        "all_violations": violations,
    }


def publish_state(route: str, verdict: str, receipt_sha256: str | None) -> None:
    write_new_json(
        state_path(route),
        {
            "schema": "lay.v10.e1-traversal-d2-primary-only-uv-route-state.v1",
            "task_id": TASK_ID,
            "transaction_id": TRANSACTION_ID,
            "route": route,
            "state": verdict,
            "marker_consumed": True,
            "receipt_sha256": receipt_sha256,
            "retry_permitted": False,
        },
        0o400,
    )
    fsync_directory(STATE)


def route_once(payload: Mapping[str, Any], route: str) -> dict[str, Any]:
    admission = verify_common(payload, route, post=False)
    result = result_path(route)
    failure = failure_path(route)
    stage = pathlib.Path(f"{result}.stage-{os.getpid()}-{time.time_ns()}")
    stage.mkdir(mode=0o700)
    marker_consumed = False
    try:
        inputs = stage / "inputs"
        subject = stage / "subject"
        inputs.mkdir(mode=0o700)
        subject.mkdir(mode=0o700)
        if ROUTES[route]["kind"] == "V":
            (stage / "control").mkdir(mode=0o700)
            os.mkfifo(stage / "perf-control.fifo", 0o600)
            os.mkfifo(stage / "perf-ack.fifo", 0o600)
        local_controller = decode_input(payload, "local_controller_b64", payload["local_controller_sha256"])
        remote_controller = decode_input(payload, "remote_controller_b64", payload["remote_controller_sha256"])
        write_new_bytes(inputs / "local-controller.py", local_controller)
        write_new_bytes(inputs / "remote-controller.py", remote_controller)
        write_new_bytes(inputs / "preflight-v4.json", decode_input(payload, "preflight_b64", PREFLIGHT_SHA256))
        write_new_bytes(
            inputs / "preflight-v4-receipt.json",
            decode_input(payload, "preflight_receipt_b64", PREFLIGHT_RECEIPT_SHA256),
        )
        write_new_bytes(
            inputs / "d1-pmu-correction-v2.json",
            decode_input(payload, "d1_correction_b64", D1_CORRECTION_SHA256),
        )
        write_new_bytes(
            inputs / "u-single-salvage-v1.json",
            decode_input(payload, "u_single_salvage_b64", U_SINGLE_SALVAGE_SHA256),
        )
        control = stage / "control" if ROUTES[route]["kind"] == "V" else None
        environment = controlled_environment(subject, control, route)
        write_new_json(
            stage / "PREOBSERVATION.json",
            {
                "schema": "lay.v10.e1-traversal-d2-primary-only-uv-preobservation.v1",
                "task_id": TASK_ID,
                "transaction_id": TRANSACTION_ID,
                "route": route,
                "kind": ROUTES[route]["kind"],
                "subject_command": subject_command(route),
                "environment": environment,
                "admission": admission,
                "marker_consumed": False,
                "retry_permitted": False,
            },
        )
        fsync_directory(stage)
        marker = consume_marker(route)
        marker_consumed = True
        write_new_json(stage / "MARKER_CONSUMPTION.json", marker)
        observation = (
            run_u(route, stage, environment)
            if ROUTES[route]["kind"] == "U"
            else run_v(route, stage, environment)
        )
        dispatch = dispatch_observation(observation, route)
        write_new_json(stage / "OBSERVATION.json", observation)
        receipt = {
            "schema": "lay.v10.e1-traversal-d2-primary-only-uv-route-receipt.v1",
            "task_id": TASK_ID,
            "transaction_id": TRANSACTION_ID,
            "route": route,
            "kind": ROUTES[route]["kind"],
            "observation_state": f"{route.replace('-', '_')}_OBSERVED",
            "verdict": dispatch["verdict"],
            "dispatch": dispatch,
            "marker": marker,
            "observation": observation,
            "elf": row(ELF),
            "map": row(MAP),
            "parity_receipt_sha256": PARITY_RECEIPT_SHA256,
            "preflight_sha256": PREFLIGHT_SHA256,
            "preflight_receipt_sha256": PREFLIGHT_RECEIPT_SHA256,
            "d1_decision_sha256": D1_DECISION_SHA256,
            "d1_correction_sha256": D1_CORRECTION_SHA256,
            "u_single_salvage_sha256": U_SINGLE_SALVAGE_SHA256,
            "remaining_available_markers": 11 - (4 + ROUTE_ORDER.index(route)),
            "total_consumed_markers": 4 + ROUTE_ORDER.index(route),
            "cargo_invocations": 0,
            "perf_record": 0,
            "perf_stat": observation["perf_stat_invocations"],
            "pmu_events_opened": observation["pmu_events_opened"],
            "d2_subject_executions": 1,
            "runtime_authority_changed": False,
            "retry_permitted": False,
            "next_action_admitted": ROUTES[route]["next_action"]
            if dispatch["verdict"] == ROUTES[route]["pass_state"]
            else "none; terminal blocked verdict",
        }
        write_new_json(stage / "D2_UV_ROUTE_RECEIPT.json", receipt)
        write_sha256sums(stage)
        seal_tree(stage)
        os.rename(stage, result)
        fsync_directory(PARENT)
        published = receipt_path(route)
        publish_state(route, dispatch["verdict"], sha256_file(published))
        return {
            **receipt,
            "published_receipt_sha256": sha256_file(published),
            "remote_result": str(result),
        }
    except BaseException as error:
        if marker_consumed:
            try:
                for path in [stage, *stage.rglob("*")]:
                    path.chmod(0o700 if path.is_dir() else 0o600)
                checksum = stage / "SHA256SUMS"
                if checksum.exists():
                    checksum.unlink()
                write_new_json(
                    stage / "FAILURE.json",
                    {
                        "verdict": "BLOCKED_PROVENANCE",
                        "route": route,
                        "error": f"{type(error).__name__}: {error}",
                        "marker_consumed": True,
                        "retry_permitted": False,
                    },
                )
                write_sha256sums(stage)
                seal_tree(stage)
                os.rename(stage, failure)
                fsync_directory(PARENT)
                publish_state(route, "BLOCKED_PROVENANCE", None)
            except BaseException:
                pass
        elif stage.exists():
            shutil.rmtree(stage)
        raise


def main() -> int:
    try:
        need(len(sys.argv) == 2, "expected one base64 payload")
        payload = json.loads(base64.b64decode(sys.argv[1], validate=True))
        action = payload.get("action")
        route = payload.get("route")
        need(isinstance(route, str) and route in ROUTES, "route missing or invalid")
        if action == "probe-before":
            value = {
                **verify_common(payload, route, post=False),
                "verdict": "D2_UV_REMOTE_PROBE_PASS",
            }
        elif action == "probe-after":
            value = {
                **verify_common(payload, route, post=True),
                "verdict": "D2_UV_REMOTE_POST_PROBE_PASS",
                "receipt": row(receipt_path(route)),
                "route_verdict": json.loads(receipt_path(route).read_text()).get("verdict"),
            }
        elif action == "run-once":
            with LOCK.open("rb") as lock:
                fcntl.flock(lock.fileno(), fcntl.LOCK_EX)
                value = route_once(payload, route)
        else:
            raise UVError(f"unsupported action: {action!r}")
        print(json.dumps(value, sort_keys=True, separators=(",", ":")))
        return 0
    except Exception as error:
        print(f"D2 U/V REMOTE ERROR: {type(error).__name__}: {error}", file=sys.stderr)
        return 1


if __name__ == "__main__":
    raise SystemExit(main())
