#!/usr/bin/env python3
"""Remote one-shot task-clock producer for primary-only D2 attribution."""

from __future__ import annotations

import base64
import bisect
import contextlib
import fcntl
import hashlib
import json
import os
import pathlib
import pwd
import re
import shutil
import stat
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
LOADER = pathlib.Path("/usr/lib/x86_64-linux-gnu/ld-linux-x86-64.so.2")
PERF_WRAPPER = pathlib.Path("/usr/bin/perf")
PERF_RESOLVED = pathlib.Path("/usr/lib/linux-hwe-6.8-tools-6.8.0-124/perf")
SUDO = pathlib.Path("/usr/bin/sudo")

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
ELF_BUILD_ID = "eb951f1a7526a9f1cb365040c10989aa5d3fc50f"
MAP_SHA256 = "2b93aedf11101a8bc5fc7ed4ffaa056b50476d96cdd2eabf9d185f59ee60f846"
PREFLIGHT_SHA256 = "e9d6328b9f610ede73ae2e8d3c819b9728ac0fc5e4c263e9fe25a61978f80f5a"
PREFLIGHT_RECEIPT_SHA256 = "740c008d59fb4689826537e46a35da554bde863358d2c18382f315395ee835e0"
U_SINGLE_SALVAGE_SHA256 = "9617502776537ca4181bd9bf195e1fd5b8fbd2679f1dfd00737f128cb88bfe0b"
PERF_WRAPPER_SHA256 = "2d0953085bf720a25efbe24f853e97d27b1f12f18a398255ff82cbafde254dad"
PERF_RESOLVED_SHA256 = "b0741eb0e6e769ba9ee0ae4e27f0c60909b51be4bc560802aef4bcd91130692e"
SUDO_SHA256 = "1e000f41739201f030cdc588fbe50d5438570f5386104c9521543824827fb985"
LOADER_SHA256 = "8d06f393f4a93bcf9b81145a259524d66a95522a646bf8d7e05b6ffdf2e63dcc"
INPUTS = {
    PACKAGE: ("cce259fe0ce5dce67702383363b66f0fe9b9ff5a87d8f01c4fcf342d91218d7b", 140_556_462),
    SIDECAR: ("a1aa95be8c43fae8fedf02d3261dd1265dcbfbc8d618330fcd6449ae829df8cd", 3_689_884),
    V7: ("33fded73e13f565bdc08a83bd440be67c80f6d85a196453050b09bc9f7ef28e4", 1_606_189),
    SCHEDULE: ("2f8346d97246ac3704434279739fa5ed27705465d628890a367c21875d879e78", 174_941),
}

QUERIES = 382
ROUNDS = 20
EXAMINED_EDGES = 25_145_756 * ROUNDS
EVENT = "task-clock:u"
PERIOD = 100_000
TRAVERSAL_BUCKETS = {
    "DAFSA_DECODE_MEMORY",
    "TRANSITION",
    "RANK",
    "STACK_CONTROL",
    "TERMINAL",
    "UNATTRIBUTED",
}

CONSUMED_MARKERS = {
    "build.consumed-before-exec": ("d21b10eac837b740fa6cb9e84c75ff76bc5ec9dc388f0896575d6f697e0c2964", 478),
    "bucket-map.consumed-before-exec": ("4471cb4edbaa8c7fc7a2f7b2ffeb4894e9b66cc904d81c0e8b1122c506745bb7", 483),
    "parity.consumed-before-exec": ("ef5eef0d5ae91bea7bff2c1077cceb8c77d2f2d5a0e7263a70f044c648115c55", 479),
    "u-single.consumed-before-exec": ("bb7b16f685e7c1a8818bc7185f0d6991f83183d8f035e90688fc66d83ba2a46b", 481),
    "u-fixed.consumed-before-exec": ("58435bf78041efe8a24191551c48062a9a0617d9ac9d468b918138b268ed27a8", 480),
    "u-reversed.consumed-before-exec": ("c13f9e22ead22c1f3afe231848a49673c93690c88ba5bbaa1426e1f46507fe0e", 483),
    "v-fixed-instr.consumed-before-exec": ("760e09bb85418e31732fcf24f93e8e81d683ecaded94e4afd29bc5d44c1c2f82", 486),
    "v-reversed-instr.consumed-before-exec": ("a87b98f363b0c51f1a36896d1892cf0c508997ab91f05980480770c6601583dc", 489),
}

UV_RESULTS = {
    "U-SINGLE": {
        "result": "uv-u-single-v1",
        "state": "U_SINGLE_STATE.json",
        "receipt_sha256": "46d52ac863e25da861f803096a6918a47a1f4b7138c0167c1f4724ad7b26dac8",
        "state_value": "BLOCKED_SEMANTIC",
        "receipt_value": "BLOCKED_SEMANTIC",
    },
    "U-FIXED": {
        "result": "uv-u-fixed-v1",
        "state": "U_FIXED_STATE.json",
        "receipt_sha256": "0ef07db4b8a07efb2ed09c3c47d6b0a9ef88e4529dd436d5d375cecf62339b59",
        "state_value": "U_FIXED_PASS",
        "receipt_value": "U_FIXED_PASS",
    },
    "U-REVERSED": {
        "result": "uv-u-reversed-v1",
        "state": "U_REVERSED_STATE.json",
        "receipt_sha256": "080917d52f3e36abffb6eab47b9e56c4a1e771b4da21caaf6fa3e2cfe686a0fb",
        "state_value": "ALL_U_PASS",
        "receipt_value": "ALL_U_PASS",
    },
    "V-FIXED-INSTR": {
        "result": "uv-v-fixed-instr-v1",
        "state": "V_FIXED_INSTR_STATE.json",
        "receipt_sha256": "56c862759c95de6682571aed3d68098dab084319817fba5af3853042e0396bae",
        "state_value": "V_FIXED_PASS",
        "receipt_value": "V_FIXED_PASS",
    },
    "V-REVERSED-INSTR": {
        "result": "uv-v-reversed-instr-v1",
        "state": "V_REVERSED_INSTR_STATE.json",
        "receipt_sha256": "5d75a502ad9e509dc6810b5494067f602d5ace1237e73f4d7913d5b9b1fc9de2",
        "state_value": "ALL_UV_VALIDITY_PASS",
        "receipt_value": "ALL_UV_VALIDITY_PASS",
    },
}

ROUTE_ORDER = ("T-SINGLE", "T-FIXED", "T-REVERSED")
ROUTES: dict[str, dict[str, Any]] = {
    "T-SINGLE": {
        "marker": "t-single",
        "marker_sha256": "8f9e716a687622cd04f693350371228072c1a303d65834c6b647fd900322fe7b",
        "marker_size": 481,
        "test": "nanda_wave::l2_field::v13_typed_peak::tests::v10_d1_component_single",
        "cpus": [0],
        "workers": 1,
        "paired_u_cpu_ns": 13_106_179_024,
        "paired_u_identity": U_SINGLE_SALVAGE_SHA256,
        "pass_state": "T_SINGLE_PASS",
        "next_action": "T-FIXED only",
    },
    "T-FIXED": {
        "marker": "t-fixed",
        "marker_sha256": "7915c483243c7116f2d023895948667cda3708f23afa7bcc7abed614772f49b0",
        "marker_size": 480,
        "test": "nanda_wave::l2_field::v13_typed_peak::tests::v10_d1_component_twenty",
        "cpus": list(range(20)),
        "workers": 20,
        "paired_u_cpu_ns": 22_761_208_185,
        "paired_u_identity": UV_RESULTS["U-FIXED"]["receipt_sha256"],
        "pass_state": "T_FIXED_PASS",
        "next_action": "T-REVERSED only",
    },
    "T-REVERSED": {
        "marker": "t-reversed",
        "marker_sha256": "26eecf8ae977c0428a5371c30d85277e5974e2ec5768629ac95212cd8cd20c9e",
        "marker_size": 483,
        "test": "nanda_wave::l2_field::v13_typed_peak::tests::v10_d1_component_twenty",
        "cpus": list(reversed(range(20))),
        "workers": 20,
        "paired_u_cpu_ns": 22_379_438_513,
        "paired_u_identity": UV_RESULTS["U-REVERSED"]["receipt_sha256"],
        "pass_state": "ALL_T_PASS",
        "next_action": "separate attribution publication only",
    },
}

PERF_RECORD_PREFIX = (
    "/usr/bin/sudo",
    "-n",
    "/usr/bin/perf",
    "record",
    "--buildid-all",
    "--sample-cpu",
    "--timestamp",
    "--event",
    EVENT,
    "--count",
    str(PERIOD),
)
READER_KINDS = ("evlist", "samples", "raw-records", "buildids")
T_DISPATCH = (
    ("provenance", "BLOCKED_PROVENANCE"),
    ("thermal", "BLOCKED_THERMAL"),
    ("capability", "BLOCKED_CAPABILITY"),
    ("bucket_map", "BLOCKED_BUCKET_MAP"),
    ("perturbation", "BLOCKED_PERTURBATION"),
    ("sample_coverage", "BLOCKED_SAMPLE_COVERAGE"),
)


class TError(RuntimeError):
    pass


def need(condition: bool, message: str) -> None:
    if not condition:
        raise TError(message)


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
        "maximum_temperature_c": max((value["millidegrees_c"] for value in thermal), default=0) / 1000.0,
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
    return PARENT / f"t-{route_slug(route)[2:]}-v1"


def failure_path(route: str) -> pathlib.Path:
    return PARENT / f"t-{route_slug(route)[2:]}-failure-v1"


def state_path(route: str) -> pathlib.Path:
    return STATE / f"{route_slug(route).upper()}_STATE.json"


def receipt_path(route: str) -> pathlib.Path:
    return result_path(route) / "D2_T_ROUTE_RECEIPT.json"


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
    expected = dict(CONSUMED_MARKERS)
    current = ROUTE_ORDER.index(route)
    for index, name in enumerate(ROUTE_ORDER):
        spec = ROUTES[name]
        suffix = "consumed-before-exec" if index < current or (post and index == current) else "available"
        expected[f"{spec['marker']}.{suffix}"] = (spec["marker_sha256"], spec["marker_size"])
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


def verify_uv_chain(salvage: Mapping[str, Any]) -> None:
    need(
        salvage.get("verdict") == "U_SINGLE_RECOVERED_FROM_SEALED_EVIDENCE"
        and salvage.get("effective_route_state") == "U_SINGLE_PASS"
        and salvage.get("historical_receipt_sha256") == UV_RESULTS["U-SINGLE"]["receipt_sha256"]
        and salvage.get("retry_permitted") is False,
        "U-SINGLE salvage overlay drift",
    )
    for route, spec in UV_RESULTS.items():
        result = PARENT / spec["result"]
        receipt_file = result / "D2_UV_ROUTE_RECEIPT.json"
        state_file = STATE / spec["state"]
        need(result.is_dir() and state_file.is_file(), f"UV projection missing: {route}")
        verify_sha256sums(result)
        need(sha256_file(receipt_file) == spec["receipt_sha256"], f"UV receipt drift: {route}")
        receipt = json.loads(receipt_file.read_text())
        state_value = json.loads(state_file.read_text())
        need(receipt.get("route") == route and receipt.get("verdict") == spec["receipt_value"], f"UV verdict drift: {route}")
        need(state_value.get("state") == spec["state_value"], f"UV state drift: {route}")
        need(state_value.get("receipt_sha256") == spec["receipt_sha256"], f"UV state receipt drift: {route}")


def verify_t_chain(route: str, *, post: bool) -> None:
    current = ROUTE_ORDER.index(route)
    for index, name in enumerate(ROUTE_ORDER):
        result = result_path(name)
        failure = failure_path(name)
        state_file = state_path(name)
        should_exist = index < current or (post and index == current)
        if should_exist:
            need(result.is_dir() and not failure.exists() and state_file.is_file(), f"T projection missing: {name}")
            verify_sha256sums(result)
            receipt = json.loads(receipt_path(name).read_text())
            state_value = json.loads(state_file.read_text())
            need(receipt.get("route") == name, f"T receipt route drift: {name}")
            need(state_value.get("receipt_sha256") == sha256_file(receipt_path(name)), f"T state receipt drift: {name}")
            if index < current:
                need(receipt.get("verdict") == ROUTES[name]["pass_state"], f"prior T route not PASS: {name}")
                need(state_value.get("state") == ROUTES[name]["pass_state"], f"prior T state not PASS: {name}")
            else:
                allowed = {terminal for _cause, terminal in T_DISPATCH} | {ROUTES[name]["pass_state"]}
                need(receipt.get("verdict") in allowed, f"current T verdict invalid: {name}")
                need(state_value.get("state") == receipt.get("verdict"), f"current T state drift: {name}")
        else:
            need(not result.exists() and not failure.exists() and not state_file.exists(), f"future T route exists: {name}")


def verify_map_bytes(map_value: Mapping[str, Any]) -> dict[str, Any]:
    need(map_value.get("elf_sha256") == ELF_SHA256, "map ELF identity drift")
    need(map_value.get("build_id") == ELF_BUILD_ID, "map Build ID drift")
    need(map_value.get("address_space") == "ELF virtual address", "map address space drift")
    ranges = map_value.get("ranges")
    need(isinstance(ranges, list) and len(ranges) == 46, "map range count drift")
    text = map_value.get("text", {})
    segment = map_value.get("executable_pt_load", {})
    cursor = text.get("start")
    mismatches = []
    with ELF.open("rb") as source:
        for index, value in enumerate(ranges):
            start = value.get("start")
            end = value.get("end_exclusive")
            need(isinstance(start, int) and isinstance(end, int) and start == cursor and end > start, f"map order drift: {index}")
            need(value.get("length_bytes") == end - start, f"map length drift: {index}")
            need(value.get("bucket") in TRAVERSAL_BUCKETS | {"OUTSIDE_TRAVERSAL"}, f"map bucket drift: {index}")
            need(segment["vaddr"] <= start < end <= segment["end_exclusive"], f"map range outside executable segment: {index}")
            offset = segment["offset"] + start - segment["vaddr"]
            source.seek(offset)
            data = source.read(end - start)
            actual = sha256_bytes(data)
            if actual != value.get("machine_bytes_sha256"):
                mismatches.append({"index": index, "start": start, "expected": value.get("machine_bytes_sha256"), "actual": actual})
            cursor = end
    need(cursor == text.get("end_exclusive"), "map text coverage drift")
    return {"range_count": len(ranges), "machine_byte_mismatches": mismatches, "covered_bytes": cursor - text["start"]}


def verify_common(payload: Mapping[str, Any], route: str, *, post: bool) -> dict[str, Any]:
    need(os.geteuid() == 0, "T remote controller requires root")
    need(route in ROUTES, f"unknown route: {route}")
    need(os.uname().nodename == HOSTNAME, "hostname drift")
    need(sha256_file(pathlib.Path("/etc/machine-id")) == MACHINE_ID_SHA256, "machine identity drift")
    need(os.uname().release == "6.8.0-124-generic", "kernel drift")
    need(read_text(pathlib.Path("/sys/devices/system/cpu/online")) == "0-19", "online CPU drift")
    need(read_text(pathlib.Path("/sys/bus/event_source/devices/cpu_core/cpus")) == "0-11", "core PMU CPU drift")
    need(read_text(pathlib.Path("/sys/bus/event_source/devices/cpu_atom/cpus")) == "12-19", "atom PMU CPU drift")
    need(read_text(pathlib.Path("/sys/bus/event_source/devices/cpu_core/type")) == "4", "core PMU type drift")
    need(read_text(pathlib.Path("/sys/bus/event_source/devices/cpu_atom/type")) == "10", "atom PMU type drift")
    need(sha256_file(PERF_WRAPPER) == PERF_WRAPPER_SHA256, "perf wrapper drift")
    need(sha256_file(PERF_RESOLVED) == PERF_RESOLVED_SHA256, "resolved perf drift")
    need(sha256_file(SUDO) == SUDO_SHA256, "sudo drift")
    need(sha256_file(LOADER) == LOADER_SHA256, "loader drift")
    preflight = json.loads(decode_input(payload, "preflight_b64", PREFLIGHT_SHA256))
    preflight_receipt = json.loads(decode_input(payload, "preflight_receipt_b64", PREFLIGHT_RECEIPT_SHA256))
    salvage = json.loads(decode_input(payload, "u_single_salvage_b64", U_SINGLE_SALVAGE_SHA256))
    need(preflight.get("scoped_positive_verdict") == "READY_TO_IMPLEMENT_PRIMARY_ONLY_D2", "preflight verdict drift")
    need(
        preflight_receipt.get("verdict") == "READY_TO_IMPLEMENT"
        and preflight_receipt.get("safe_to_implement") is True,
        "preflight receipt drift",
    )
    need(row(ELF)["sha256"] == ELF_SHA256 and ELF.stat().st_size == ELF_SIZE and mode_string(ELF) == "0555", "D2 ELF drift")
    need(row(MAP)["sha256"] == MAP_SHA256 and mode_string(MAP) == "0444", "D2 map drift")
    for path, (digest, size) in INPUTS.items():
        value = row(path)
        need(value["sha256"] == digest and value["size_bytes"] == size and value["mode"] == "0444", f"input drift: {path}")
    verify_uv_chain(salvage)
    verify_t_chain(route, post=post)
    markers = verify_markers(route, post=post)
    map_check = verify_map_bytes(json.loads(MAP.read_text()))
    need(not map_check["machine_byte_mismatches"], "sealed map machine bytes drift")
    return {
        "hostname": HOSTNAME,
        "machine_id_sha256": MACHINE_ID_SHA256,
        "route": route,
        "elf": row(ELF),
        "map": row(MAP),
        "map_check": map_check,
        "perf_wrapper_sha256": PERF_WRAPPER_SHA256,
        "resolved_perf_sha256": PERF_RESOLVED_SHA256,
        "markers": markers,
        "remote_writes": 0,
    }


def controlled_environment(output: pathlib.Path, route: str) -> dict[str, str]:
    return {
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


def reader_commands(data_path: pathlib.Path) -> dict[str, list[str]]:
    return {
        "evlist": ["/usr/bin/perf", "evlist", "-v", "-i", str(data_path)],
        "samples": [
            "/usr/bin/perf",
            "script",
            "-i",
            str(data_path),
            "-F",
            "comm,pid,tid,cpu,time,event,ip,dso,period",
        ],
        "raw-records": ["/usr/bin/perf", "script", "-D", "-i", str(data_path)],
        "buildids": ["/usr/bin/perf", "buildid-list", "-i", str(data_path)],
    }


def consume_marker(route: str) -> dict[str, Any]:
    spec = ROUTES[route]
    available = STATE / f"markers/{spec['marker']}.available"
    consumed = STATE / f"markers/{spec['marker']}.consumed-before-exec"
    before = row(available)
    need(before["sha256"] == spec["marker_sha256"] and before["size_bytes"] == spec["marker_size"], f"marker identity drift: {route}")
    need(not consumed.exists(), f"marker already consumed: {route}")
    os.rename(available, consumed)
    fsync_directory(available.parent)
    after = row(consumed)
    need(after["sha256"] == before["sha256"] and after["size_bytes"] == before["size_bytes"], "marker rename drift")
    return {"before": before, "after": after, "consumed_before_effect": True}


def exact_event_line(text: str) -> str:
    lines = [line for line in text.splitlines() if line.startswith(f"{EVENT}:")]
    need(len(lines) == 1, f"task-clock evlist line count: {len(lines)}")
    return lines[0]


def parse_required_attr(line: str, key: str) -> int:
    match = re.search(rf"\b{re.escape(key)}\s*:\s*(0x[0-9a-fA-F]+|\d+)", line)
    if match is None and key == "sample_period":
        match = re.search(r"\{\s*sample_period\s*,\s*sample_freq\s*\}\s*:\s*(0x[0-9a-fA-F]+|\d+)", line)
    need(match is not None, f"task-clock line lacks {key}")
    return int(match.group(1), 0)


SAMPLE_PATTERN = re.compile(
    r"^\s*(.*?)\s+(\d+)/(\d+)\s+\[(\d{3})\]\s+([0-9]+\.[0-9]+):\s+"
    r"(\d+)\s+([^\s]+):\s+([0-9a-fA-F]+)\s+\((.*)\)\s*$"
)
MMAP2_PATTERN = re.compile(
    r"PERF_RECORD_MMAP2\s+(\d+)/(\d+):\s+\[0x([0-9a-fA-F]+)\(0x([0-9a-fA-F]+)\)\s+"
    r"@\s+0x([0-9a-fA-F]+).*?\]:\s+([rwxps-]+)\s+(.+)$"
)


def parse_samples(path: pathlib.Path) -> list[dict[str, Any]]:
    values = []
    with path.open(encoding="utf-8", errors="replace") as source:
        for line_number, line in enumerate(source, start=1):
            match = SAMPLE_PATTERN.fullmatch(line.rstrip("\n"))
            need(match is not None, f"unparsed sample row {line_number}: {line[:240]!r}")
            values.append(
                {
                    "comm": match.group(1).strip(),
                    "pid": int(match.group(2)),
                    "tid": int(match.group(3)),
                    "cpu": int(match.group(4)),
                    "time": match.group(5),
                    "period": int(match.group(6)),
                    "event": match.group(7),
                    "runtime_ip": int(match.group(8), 16),
                    "dso": match.group(9),
                }
            )
    return values


def scan_raw_records(path: pathlib.Path) -> dict[str, Any]:
    mappings = []
    lost = 0
    throttle = 0
    unthrottle = 0
    raw_samples = 0
    with path.open(encoding="utf-8", errors="replace") as source:
        for line in source:
            if re.search(r"PERF_RECORD_LOST(?:_SAMPLES)?\b", line):
                lost += 1
            if "PERF_RECORD_THROTTLE" in line:
                throttle += 1
            if "PERF_RECORD_UNTHROTTLE" in line:
                unthrottle += 1
            if "PERF_RECORD_SAMPLE" in line:
                raw_samples += 1
            match = MMAP2_PATTERN.search(line)
            if match is not None and match.group(7) == str(ELF):
                mappings.append(
                    {
                        "pid": int(match.group(1)),
                        "tid": int(match.group(2)),
                        "start": int(match.group(3), 16),
                        "length": int(match.group(4), 16),
                        "offset": int(match.group(5), 16),
                        "permissions": match.group(6),
                        "path": match.group(7),
                    }
                )
    return {
        "mappings": mappings,
        "lost_records": lost,
        "throttle_records": throttle,
        "unthrottle_records": unthrottle,
        "raw_sample_records": raw_samples,
    }


def validate_subject(route: str, subject: pathlib.Path) -> dict[str, Any]:
    receipt_file = subject / "SUBJECT_RECEIPT.json"
    structure_file = subject / "structure.json"
    samples_file = subject / "component-samples.bin"
    need(receipt_file.is_file() and structure_file.is_file() and samples_file.is_file(), "subject evidence incomplete")
    receipt = json.loads(receipt_file.read_text())
    spec = ROUTES[route]
    need(receipt.get("run_id") == route, "subject run ID drift")
    need(receipt.get("test") == spec["test"], "subject test drift")
    need(receipt.get("queries") == QUERIES and receipt.get("rounds") == ROUNDS, "subject geometry drift")
    need(receipt.get("workers") == spec["workers"], "subject worker count drift")
    need(receipt.get("cpus") == spec["cpus"], "subject CPU mapping drift")
    need(receipt.get("samples", {}).get("samples") == QUERIES * ROUNDS, "subject sample count drift")
    need(receipt.get("samples", {}).get("errors") == 0 and receipt.get("samples", {}).get("unresolved") == 0, "subject semantic failure")
    if spec["workers"] == 1:
        need(receipt.get("warmup_queries") == QUERIES, "single subject warmup drift")
        need(receipt.get("thread_migrations") == 0, "single subject thread migration")
    else:
        need(receipt.get("warmup_bursts") == 1, "twenty subject warmup drift")
        need(receipt.get("worker_migration_deltas") == [0] * spec["workers"], "subject worker migration")
    need(sha256_file(structure_file) == "90d24adee563be803c390b41b18b41624b999db37b34c26650cb362f03d06712", "subject structure drift")
    return {
        "receipt": receipt,
        "receipt_identity": row(receipt_file),
        "structure": row(structure_file),
        "component_samples": row(samples_file),
    }


def attribute_samples(
    samples: list[dict[str, Any]], raw: Mapping[str, Any], map_value: Mapping[str, Any]
) -> dict[str, Any]:
    segment = map_value["executable_pt_load"]
    page_size = os.sysconf("SC_PAGE_SIZE")
    expected_offset = segment["offset"] // page_size * page_size
    executable = [
        value
        for value in raw["mappings"]
        if "x" in value["permissions"] and value["offset"] == expected_offset
    ]
    need(len(executable) == 1, f"executable D2 mapping count: {len(executable)}")
    mapping = executable[0]
    aligned_vaddr = segment["vaddr"] // page_size * page_size
    load_bias = mapping["start"] - aligned_vaddr
    ranges = map_value["ranges"]
    starts = [value["start"] for value in ranges]
    bucket_counts: dict[str, int] = {}
    sub_bucket_counts: dict[str, int] = {}
    d2_samples = 0
    exact_dso_samples = 0
    traversal_samples = 0
    outside_samples = 0
    outside_text_samples = 0
    dso_mismatches = []
    normalized_examples = []
    traversal_tids: set[int] = set()
    traversal_pids: set[int] = set()
    traversal_cpus: set[int] = set()
    for sample in samples:
        inside_mapping = mapping["start"] <= sample["runtime_ip"] < mapping["start"] + mapping["length"]
        rendered_exact = sample["dso"] == str(ELF)
        if not inside_mapping and not rendered_exact:
            continue
        d2_samples += 1
        exact_dso_samples += int(rendered_exact)
        if not inside_mapping or not rendered_exact:
            dso_mismatches.append(
                {
                    "runtime_ip": f"0x{sample['runtime_ip']:x}",
                    "dso": sample["dso"],
                    "inside_mapping": inside_mapping,
                }
            )
            continue
        normalized = sample["runtime_ip"] - load_bias
        if len(normalized_examples) < 16:
            normalized_examples.append(
                {
                    "runtime_ip": f"0x{sample['runtime_ip']:x}",
                    "normalized_ip": f"0x{normalized:x}",
                    "pid": sample["pid"],
                    "tid": sample["tid"],
                    "cpu": sample["cpu"],
                }
            )
        position = bisect.bisect_right(starts, normalized) - 1
        if position < 0 or not (ranges[position]["start"] <= normalized < ranges[position]["end_exclusive"]):
            outside_text_samples += 1
            continue
        value = ranges[position]
        bucket = value["bucket"]
        sub_bucket = value["sub_bucket"]
        if bucket == "OUTSIDE_TRAVERSAL":
            outside_samples += 1
            continue
        need(bucket in TRAVERSAL_BUCKETS, f"unknown traversal bucket: {bucket}")
        traversal_samples += 1
        bucket_counts[bucket] = bucket_counts.get(bucket, 0) + 1
        sub_bucket_counts[sub_bucket] = sub_bucket_counts.get(sub_bucket, 0) + 1
        traversal_tids.add(sample["tid"])
        traversal_pids.add(sample["pid"])
        traversal_cpus.add(sample["cpu"])
    need(len(traversal_pids) == 1, f"traversal PID count: {len(traversal_pids)}")
    main_pid = next(iter(traversal_pids))
    worker_tids = sorted(tid for tid in traversal_tids if tid != main_pid)
    unattributed = bucket_counts.get("UNATTRIBUTED", 0)
    unattributed_percent = 100.0 * unattributed / traversal_samples if traversal_samples else 100.0
    sampled_cpu_ns = traversal_samples * PERIOD
    sampled_cpu_per_edge = sampled_cpu_ns / EXAMINED_EDGES
    return {
        "page_size": page_size,
        "mapping": mapping,
        "aligned_pt_load_vaddr": aligned_vaddr,
        "load_bias": load_bias,
        "load_bias_hex": f"0x{load_bias:x}",
        "normalization_unique": True,
        "normalized_examples": normalized_examples,
        "d2_samples": d2_samples,
        "exact_dso_samples": exact_dso_samples,
        "dso_mismatches": dso_mismatches[:32],
        "traversal_samples": traversal_samples,
        "outside_traversal_samples": outside_samples,
        "outside_mapped_text_samples": outside_text_samples,
        "bucket_counts": dict(sorted(bucket_counts.items())),
        "sub_bucket_counts": dict(sorted(sub_bucket_counts.items())),
        "bucket_cpu_ns_per_edge": {
            key: value * PERIOD / EXAMINED_EDGES for key, value in sorted(bucket_counts.items())
        },
        "sub_bucket_cpu_ns_per_edge": {
            key: value * PERIOD / EXAMINED_EDGES for key, value in sorted(sub_bucket_counts.items())
        },
        "unattributed_samples": unattributed,
        "unattributed_percent": unattributed_percent,
        "sampled_traversal_cpu_ns": sampled_cpu_ns,
        "sampled_traversal_cpu_per_edge_ns": sampled_cpu_per_edge,
        "traversal_pids": sorted(traversal_pids),
        "traversal_tids": sorted(traversal_tids),
        "worker_tids": worker_tids,
        "traversal_cpus": sorted(traversal_cpus),
    }


def run_command_to_files(
    command: Sequence[str], stdout_path: pathlib.Path, stderr_path: pathlib.Path, *, timeout: int
) -> dict[str, Any]:
    with stdout_path.open("xb") as stdout, stderr_path.open("xb") as stderr:
        try:
            result = subprocess.run(list(command), stdout=stdout, stderr=stderr, check=False, timeout=timeout)
            return {"returncode": result.returncode, "timed_out": False}
        except subprocess.TimeoutExpired:
            return {"returncode": None, "timed_out": True}


def run_t(route: str, stage: pathlib.Path, environment: dict[str, str]) -> dict[str, Any]:
    data_path = stage / "perf.data"
    subject = stage / "subject"
    record_command = [
        *PERF_RECORD_PREFIX,
        "--output",
        str(data_path),
        "--",
        *child_as_e(environment, subject_command(route)),
    ]
    write_new_json(stage / "record-command.json", record_command)
    before = host_snapshot()
    record_status = run_command_to_files(
        record_command, stage / "record-stdout.log", stage / "record-stderr.log", timeout=900
    )
    after_record = host_snapshot()
    reader_status: dict[str, Any] = {}
    commands = reader_commands(data_path)
    write_new_json(stage / "reader-commands.json", commands)
    for name in READER_KINDS:
        output = stage / f"{name}.stdout"
        error = stage / f"{name}.stderr"
        if data_path.is_file() and data_path.stat().st_size > 0:
            reader_status[name] = run_command_to_files(commands[name], output, error, timeout=1800)
        else:
            write_new_bytes(output, b"")
            write_new_bytes(error, b"perf.data missing or empty\n")
            reader_status[name] = {"returncode": None, "timed_out": False, "not_run": True}
    after = host_snapshot()

    violations = {name: [] for name, _ in T_DISPATCH}
    if record_status["timed_out"] or record_status["returncode"] != 0:
        violations["capability"].append(f"perf record status: {record_status}")
    if not data_path.is_file() or data_path.stat().st_size == 0:
        violations["capability"].append("perf.data missing or empty")
    for name, status_value in reader_status.items():
        if status_value.get("timed_out") or status_value.get("returncode") != 0:
            violations["capability"].append(f"{name} reader status: {status_value}")

    event_validation: dict[str, Any] = {}
    samples: list[dict[str, Any]] = []
    raw: dict[str, Any] = {
        "mappings": [],
        "lost_records": 0,
        "throttle_records": 0,
        "unthrottle_records": 0,
        "raw_sample_records": 0,
    }
    attribution: dict[str, Any] = {}
    subject_validation: dict[str, Any] = {}
    try:
        event_line = exact_event_line((stage / "evlist.stdout").read_text(errors="replace"))
        event_validation = {
            "line": event_line,
            "type": parse_required_attr(event_line, "type"),
            "config": parse_required_attr(event_line, "config"),
            "sample_period": parse_required_attr(event_line, "sample_period"),
            "exclude_kernel": parse_required_attr(event_line, "exclude_kernel"),
            "inherit": parse_required_attr(event_line, "inherit"),
            "freq": 0 if re.search(r"\bfreq\s*:", event_line) is None else parse_required_attr(event_line, "freq"),
            "precise_ip": 0 if re.search(r"\bprecise_ip\s*:", event_line) is None else parse_required_attr(event_line, "precise_ip"),
        }
        expected_event = {
            "type": 1,
            "config": 1,
            "sample_period": PERIOD,
            "exclude_kernel": 1,
            "inherit": 1,
            "freq": 0,
            "precise_ip": 0,
        }
        if any(event_validation[key] != value for key, value in expected_event.items()):
            violations["capability"].append(f"task-clock event identity mismatch: {event_validation}")
    except Exception as error:
        violations["capability"].append(f"event validation failed: {type(error).__name__}: {error}")
    try:
        samples = parse_samples(stage / "samples.stdout")
        if not samples:
            violations["capability"].append("perf script emitted no samples")
        if samples and ({value["event"] for value in samples} != {EVENT} or {value["period"] for value in samples} != {PERIOD}):
            violations["capability"].append("sample event or period drift")
    except Exception as error:
        violations["capability"].append(f"sample parser failed: {type(error).__name__}: {error}")
    try:
        raw = scan_raw_records(stage / "raw-records.stdout")
    except Exception as error:
        violations["capability"].append(f"raw reader parser failed: {type(error).__name__}: {error}")
    try:
        buildids = (stage / "buildids.stdout").read_text(errors="replace")
        pattern = rf"(?m)^{ELF_BUILD_ID}\s+{re.escape(str(ELF))}$"
        if re.search(pattern, buildids) is None:
            violations["bucket_map"].append("exact D2 Build ID/path absent from buildid-list")
    except Exception as error:
        violations["bucket_map"].append(f"Build ID validation failed: {type(error).__name__}: {error}")
    try:
        map_value = json.loads(MAP.read_text())
        map_check = verify_map_bytes(map_value)
        if map_check["machine_byte_mismatches"]:
            violations["bucket_map"].append("machine byte mismatch")
        attribution = attribute_samples(samples, raw, map_value)
        if attribution["dso_mismatches"]:
            violations["bucket_map"].append("sample DSO/mapping identity mismatch")
        if not attribution["worker_tids"]:
            violations["capability"].append("no inherited worker-thread traversal samples")
        if not set(attribution["traversal_cpus"]).issubset(set(ROUTES[route]["cpus"])):
            violations["provenance"].append("traversal sample CPU outside frozen mapping")
    except Exception as error:
        violations["bucket_map"].append(f"IP normalization or map join failed: {type(error).__name__}: {error}")
    try:
        subject_validation = validate_subject(route, subject)
    except Exception as error:
        violations["provenance"].append(f"subject evidence validation failed: {type(error).__name__}: {error}")

    drift = thermal_drift(before, after)
    if drift:
        violations["thermal"].append(f"thermal throttle drift: {drift}")
    if raw["lost_records"] or raw["throttle_records"] or raw["unthrottle_records"]:
        violations["sample_coverage"].append(
            f"lost/throttle/unthrottle={raw['lost_records']}/{raw['throttle_records']}/{raw['unthrottle_records']}"
        )
    if samples and any(value["period"] != PERIOD for value in samples):
        violations["sample_coverage"].append("adaptive sample period observed")
    if attribution:
        paired_cpu_per_edge = ROUTES[route]["paired_u_cpu_ns"] / EXAMINED_EDGES
        sampled_cpu_per_edge = attribution["sampled_traversal_cpu_per_edge_ns"]
        delta = abs(sampled_cpu_per_edge - paired_cpu_per_edge) / paired_cpu_per_edge * 100.0
        attribution["paired_u_cpu_ns"] = ROUTES[route]["paired_u_cpu_ns"]
        attribution["paired_u_cpu_per_edge_ns"] = paired_cpu_per_edge
        attribution["paired_u_identity"] = ROUTES[route]["paired_u_identity"]
        attribution["sampled_vs_paired_u_delta_percent"] = delta
        if delta > 5.0:
            violations["perturbation"].append(f"sampled-vs-U delta {delta:.12f}% exceeds 5%")
        if attribution["traversal_samples"] < 50_000:
            violations["sample_coverage"].append(
                f"traversal samples {attribution['traversal_samples']} below 50000"
            )
        if attribution["unattributed_percent"] > 5.0:
            violations["sample_coverage"].append(
                f"UNATTRIBUTED {attribution['unattributed_percent']:.12f}% exceeds 5%"
            )
    else:
        violations["sample_coverage"].append("attribution summary unavailable")

    return {
        "schema": "lay.v10.e1-traversal-d2-primary-only-t-observation.v1",
        "route": route,
        "complete": True,
        "record_command": record_command,
        "record_status": record_status,
        "reader_commands": commands,
        "reader_status": reader_status,
        "perf_data": row(data_path) if data_path.is_file() else None,
        "event_validation": event_validation,
        "sample_count": len(samples),
        "raw_records": raw,
        "attribution": attribution,
        "subject": subject_validation,
        "host_before": before,
        "host_after_record": after_record,
        "host_after": after,
        "thermal_throttle_drift": drift,
        "violations": violations,
        "perf_record_invocations": 1,
        "perf_reader_invocations": 4,
        "perf_stat_invocations": 0,
        "pmu_events_opened": 1,
        "warmup_time_filter_or_subtraction": False,
    }


def dispatch_observation(observation: Mapping[str, Any], route: str) -> dict[str, Any]:
    violations = observation.get("violations", {})
    selected = None
    for rank, (cause, terminal) in enumerate(T_DISPATCH):
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
            "schema": "lay.v10.e1-traversal-d2-primary-only-t-route-state.v1",
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
    stage.mkdir(mode=0o755)
    marker_consumed = False
    try:
        inputs = stage / "inputs"
        subject = stage / "subject"
        inputs.mkdir(mode=0o700)
        account = pwd.getpwnam("e")
        subject.mkdir(mode=0o700)
        os.chown(subject, account.pw_uid, account.pw_gid)
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
            inputs / "u-single-salvage-v1.json",
            decode_input(payload, "u_single_salvage_b64", U_SINGLE_SALVAGE_SHA256),
        )
        environment = controlled_environment(subject, route)
        data_path = stage / "perf.data"
        write_new_json(
            stage / "PREOBSERVATION.json",
            {
                "schema": "lay.v10.e1-traversal-d2-primary-only-t-preobservation.v1",
                "task_id": TASK_ID,
                "transaction_id": TRANSACTION_ID,
                "route": route,
                "subject_command": subject_command(route),
                "perf_record_command": [
                    *PERF_RECORD_PREFIX,
                    "--output",
                    str(data_path),
                    "--",
                    *child_as_e(environment, subject_command(route)),
                ],
                "reader_commands": reader_commands(data_path),
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
        observation = run_t(route, stage, environment)
        dispatch = dispatch_observation(observation, route)
        write_new_json(stage / "OBSERVATION.json", observation)
        receipt = {
            "schema": "lay.v10.e1-traversal-d2-primary-only-t-route-receipt.v1",
            "task_id": TASK_ID,
            "transaction_id": TRANSACTION_ID,
            "route": route,
            "observation_state": f"{route.replace('-', '_')}_OBSERVED",
            "verdict": dispatch["verdict"],
            "dispatch": dispatch,
            "marker": marker,
            "observation": observation,
            "elf": row(ELF),
            "map": row(MAP),
            "preflight_sha256": PREFLIGHT_SHA256,
            "preflight_receipt_sha256": PREFLIGHT_RECEIPT_SHA256,
            "u_single_salvage_sha256": U_SINGLE_SALVAGE_SHA256,
            "remaining_available_markers": 2 - ROUTE_ORDER.index(route),
            "total_consumed_markers": 9 + ROUTE_ORDER.index(route),
            "cargo_invocations": 0,
            "perf_record": observation["perf_record_invocations"],
            "perf_readers": observation["perf_reader_invocations"],
            "perf_stat": 0,
            "pmu_events_opened": observation["pmu_events_opened"],
            "d2_subject_executions": 1,
            "runtime_authority_changed": False,
            "retry_permitted": False,
            "next_action_admitted": ROUTES[route]["next_action"]
            if dispatch["verdict"] == ROUTES[route]["pass_state"]
            else "none; terminal blocked verdict",
        }
        write_new_json(stage / "D2_T_ROUTE_RECEIPT.json", receipt)
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
            value = {**verify_common(payload, route, post=False), "verdict": "D2_T_REMOTE_PROBE_PASS"}
        elif action == "probe-after":
            value = {
                **verify_common(payload, route, post=True),
                "verdict": "D2_T_REMOTE_POST_PROBE_PASS",
                "receipt": row(receipt_path(route)),
                "route_verdict": json.loads(receipt_path(route).read_text()).get("verdict"),
            }
        elif action == "run-once":
            with LOCK.open("rb") as lock:
                fcntl.flock(lock.fileno(), fcntl.LOCK_EX)
                value = route_once(payload, route)
        else:
            raise TError(f"unsupported action: {action!r}")
        print(json.dumps(value, sort_keys=True, separators=(",", ":")))
        return 0
    except Exception as error:
        print(f"D2 T REMOTE ERROR: {type(error).__name__}: {error}", file=sys.stderr)
        return 1


if __name__ == "__main__":
    raise SystemExit(main())
