#!/usr/bin/env python3
"""Remote one-shot producer for D5 multiworker TID attribution."""

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
import struct
import subprocess
import sys
import time
from typing import Any, Mapping, Sequence


TASK_ID = "slice8b-v10-e1-traversal-d5-multiworker-tid-estimator-v1-20260826"
TRANSACTION_ID = "3ee46e2c915677e1b2d3cd6bcc9709e0232252dbc120745b097d736537779036"
HOSTNAME = "e-MEGA-MINI-M1-13th"
MACHINE_ID_SHA256 = "5ac0bb537745673ef80942c2df32d586f5753c85de1c8b2453aa4e7317b77441"

D2_TASK_ID = "slice8b-v10-e1-traversal-d2-primary-only-v2-20260825"
D2_TRANSACTION_ID = "35dbe5c5eaec7dbb5b5b112313a3285dbe2ceaea0a910719ad379255ca5569e7"
D2_PARENT = pathlib.Path("/home/e/.local/share/lay/provenance") / D2_TASK_ID
D2_STATE = pathlib.Path("/home/e/.local/state/lay") / D2_TASK_ID

D4_TASK_ID = "slice8b-v10-e1-traversal-d4-estimator-recovery-v1-20260826"
D4_TRANSACTION_ID = "2d3002b7cf615459a4250d7e44eb2094863dc422f908080b7afa59551ba4ee26"
D4_PARENT = pathlib.Path("/home/e/.local/share/lay/provenance") / D4_TASK_ID
D4_STATE = pathlib.Path("/home/e/.local/state/lay") / D4_TASK_ID
D4_U3_RECEIPT_SHA256 = "db2ba1b3d4e11ac2c4edb24e382f93d282b1b5d605fcbb1636f5e346030dc000"
D4_T3_RECEIPT_SHA256 = "dd4e3b7bb49d368fe1461c36fda0968af629293e801500770ca9dc3715a96f09"

PARENT = pathlib.Path("/home/e/.local/share/lay/provenance") / TASK_ID
STATE = pathlib.Path("/home/e/.local/state/lay") / TASK_ID
LOCK = STATE / "route.lock"
ELF = D2_PARENT / "build-v1/d2-test-elf"
MAP = D2_PARENT / "bucket-map-v1/D2_BUCKET_MAP.json"
LOADER = pathlib.Path("/usr/lib/x86_64-linux-gnu/ld-linux-x86-64.so.2")
TASKSET = pathlib.Path("/usr/bin/taskset")
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
PAPER_SHA256 = "cfe512a162d257ecf5b3c37c2dbb1c39ecacf831b70954f46912035d00169c87"
PREFLIGHT_SHA256 = "4c51d24eaa8cdd1052f5220d453fc3ad5d8e5d0b43fda7c972d3b0c8063ef77a"
PREFLIGHT_RECEIPT_SHA256 = "78cde27fbf1062122fb74e3c0f5028c29784e9d16417cad7ae0638864ef6455b"
D2_TERMINAL_SHA256 = "75dc2703e279524a76c49a455b6081634d1c67274106da45f0d7d610af95e608"
D3_TERMINAL_SHA256 = "7f4c0fd9466cced361c21858ea2017910adc7b92db7c1edfe02f3be09d5c4299"
D4_TERMINAL_SHA256 = "f748a7c448f7f142a342efa2b916a8995969044567a3aaa9a7de3aff6ba8645b"
PERF_WRAPPER_SHA256 = "2d0953085bf720a25efbe24f853e97d27b1f12f18a398255ff82cbafde254dad"
PERF_RESOLVED_SHA256 = "b0741eb0e6e769ba9ee0ae4e27f0c60909b51be4bc560802aef4bcd91130692e"
TASKSET_SHA256 = "63e52c4b99a688ccd7bab6edbc6df2af1acad124eb852adcb4d20043d28eb2d3"
SUDO_SHA256 = "1e000f41739201f030cdc588fbe50d5438570f5386104c9521543824827fb985"
LOADER_SHA256 = "8d06f393f4a93bcf9b81145a259524d66a95522a646bf8d7e05b6ffdf2e63dcc"
D1_STRUCTURE_SHA256 = "90d24adee563be803c390b41b18b41624b999db37b34c26650cb362f03d06712"

INPUTS = {
    PACKAGE: ("cce259fe0ce5dce67702383363b66f0fe9b9ff5a87d8f01c4fcf342d91218d7b", 140_556_462),
    SIDECAR: ("a1aa95be8c43fae8fedf02d3261dd1265dcbfbc8d618330fcd6449ae829df8cd", 3_689_884),
    V7: ("33fded73e13f565bdc08a83bd440be67c80f6d85a196453050b09bc9f7ef28e4", 1_606_189),
    SCHEDULE: ("2f8346d97246ac3704434279739fa5ed27705465d628890a367c21875d879e78", 174_941),
}

D2_MARKERS = {
    "build.consumed-before-exec": ("d21b10eac837b740fa6cb9e84c75ff76bc5ec9dc388f0896575d6f697e0c2964", 478),
    "bucket-map.consumed-before-exec": ("4471cb4edbaa8c7fc7a2f7b2ffeb4894e9b66cc904d81c0e8b1122c506745bb7", 483),
    "parity.consumed-before-exec": ("ef5eef0d5ae91bea7bff2c1077cceb8c77d2f2d5a0e7263a70f044c648115c55", 479),
    "u-single.consumed-before-exec": ("bb7b16f685e7c1a8818bc7185f0d6991f83183d8f035e90688fc66d83ba2a46b", 481),
    "u-fixed.consumed-before-exec": ("58435bf78041efe8a24191551c48062a9a0617d9ac9d468b918138b268ed27a8", 480),
    "u-reversed.consumed-before-exec": ("c13f9e22ead22c1f3afe231848a49673c93690c88ba5bbaa1426e1f46507fe0e", 483),
    "v-fixed-instr.consumed-before-exec": ("760e09bb85418e31732fcf24f93e8e81d683ecaded94e4afd29bc5d44c1c2f82", 486),
    "v-reversed-instr.consumed-before-exec": ("a87b98f363b0c51f1a36896d1892cf0c508997ab91f05980480770c6601583dc", 489),
    "t-single.consumed-before-exec": ("8f9e716a687622cd04f693350371228072c1a303d65834c6b647fd900322fe7b", 481),
    "t-fixed.available": ("7915c483243c7116f2d023895948667cda3708f23afa7bcc7abed614772f49b0", 480),
    "t-reversed.available": ("26eecf8ae977c0428a5371c30d85277e5974e2ec5768629ac95212cd8cd20c9e", 483),
}

FIXED_CPUS = tuple(range(20))
REVERSED_CPUS = tuple(reversed(FIXED_CPUS))
ROUTE_ORDER = ("U4-FIXED", "T4-FIXED", "U4-REVERSED", "T4-REVERSED")
ACTION_TO_ROUTE = {
    "run-u4-fixed": "U4-FIXED",
    "run-t4-fixed": "T4-FIXED",
    "run-u4-reversed": "U4-REVERSED",
    "run-t4-reversed": "T4-REVERSED",
}
ROUTES = {
    "U4-FIXED": {
        "marker": "u4-fixed",
        "marker_sha256": "445b573ba817c87abc345d56bb065c27cfc38ed4f9569dbfd1e91803124fabfd",
        "marker_size": 293,
        "pass_state": "U4_FIXED_PASS",
        "mapping": "FIXED",
        "cpus": FIXED_CPUS,
    },
    "T4-FIXED": {
        "marker": "t4-fixed",
        "marker_sha256": "c3c3711e77121062613ec1fe252e0f44c8ad41744008dd41fc894e68ce5a3c02",
        "marker_size": 293,
        "pass_state": "T4_FIXED_PASS",
        "mapping": "FIXED",
        "cpus": FIXED_CPUS,
    },
    "U4-REVERSED": {
        "marker": "u4-reversed",
        "marker_sha256": "359a675a658f3998c4b5fff181eeb03db67974739471e5b8c100795b86f54c45",
        "marker_size": 296,
        "pass_state": "U4_REVERSED_PASS",
        "mapping": "REVERSED",
        "cpus": REVERSED_CPUS,
    },
    "T4-REVERSED": {
        "marker": "t4-reversed",
        "marker_sha256": "3319d302aa0ed4bafdc23de8f04e5138a9352f6f22ca9934131bdc1d696a7cda",
        "marker_size": 296,
        "pass_state": "D5_MULTIWORKER_ATTRIBUTION_PASS",
        "mapping": "REVERSED",
        "cpus": REVERSED_CPUS,
    },
}

SUBJECT_TEST = "nanda_wave::l2_field::v13_typed_peak::tests::v10_d1_component_twenty"
QUERIES = 382
MEASURED_ROUNDS = 20
SAMPLED_ROUNDS = 21
EDGES_PER_ROUND = 25_145_756
U4_EDGES = EDGES_PER_ROUND * MEASURED_ROUNDS
T4_EDGES = EDGES_PER_ROUND * SAMPLED_ROUNDS
REQUESTS = QUERIES * MEASURED_ROUNDS
EVENT = "task-clock:u"
PERIOD = 200_000
HOST_SAMPLE_RATE = "8000"
PHASES = ("oracle", "lanes", "eqmask", "traversal", "merge", "certificate")
COMPONENT_SAMPLE = struct.Struct("<HHBB" + "Q" * 14)
TRAVERSAL_BUCKETS = {
    "DAFSA_DECODE_MEMORY",
    "TRANSITION",
    "RANK",
    "STACK_CONTROL",
    "TERMINAL",
    "UNATTRIBUTED",
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
U4_DISPATCH = (
    ("provenance", "BLOCKED_PROVENANCE"),
    ("thermal", "BLOCKED_THERMAL"),
    ("semantic", "BLOCKED_SEMANTIC"),
)
T4_DISPATCH = (
    ("provenance", "BLOCKED_PROVENANCE"),
    ("thermal", "BLOCKED_THERMAL"),
    ("capability", "BLOCKED_CAPABILITY"),
    ("bucket_map", "BLOCKED_BUCKET_MAP"),
    ("perturbation", "BLOCKED_PERTURBATION"),
    ("sample_coverage", "BLOCKED_SAMPLE_COVERAGE"),
)


class D5Error(RuntimeError):
    pass


def need(condition: bool, message: str) -> None:
    if not condition:
        raise D5Error(message)


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


def canonical_json_bytes(value: Any) -> bytes:
    return (json.dumps(value, ensure_ascii=False, indent=2, sort_keys=True) + "\n").encode()


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


def read_text(path: pathlib.Path) -> str:
    return path.read_text(encoding="utf-8", errors="replace").strip()


def seal_evidence_tree(root: pathlib.Path) -> None:
    for path in sorted(root.rglob("*"), reverse=True):
        path.chmod(0o555 if path.is_dir() else 0o444)
    root.chmod(0o555)


def write_sha256sums(root: pathlib.Path) -> None:
    entries = []
    for path in sorted(item for item in root.rglob("*") if item.is_file() and item.name != "SHA256SUMS"):
        entries.append(f"{sha256_file(path)}  {path.relative_to(root)}\n")
    write_new_bytes(root / "SHA256SUMS", "".join(entries).encode())


def verify_sha256sums(root: pathlib.Path) -> int:
    manifest = root / "SHA256SUMS"
    need(manifest.is_file(), f"SHA256SUMS missing: {root}")
    count = 0
    for line in manifest.read_text().splitlines():
        digest, relative = line.split("  ", 1)
        path = root / relative
        need(path.is_file() and sha256_file(path) == digest, f"manifest mismatch: {path}")
        count += 1
    actual = sum(1 for path in root.rglob("*") if path.is_file() and path.name != "SHA256SUMS")
    need(count == actual, f"manifest cardinality drift: {count} != {actual}")
    return count


def temperatures() -> list[dict[str, Any]]:
    values = []
    for path in sorted(pathlib.Path("/sys/class/thermal").glob("thermal_zone*/temp")):
        raw = read_text(path)
        if raw.lstrip("-").isdigit():
            values.append({"path": str(path), "millidegrees_c": int(raw)})
    for path in sorted(pathlib.Path("/sys/class/hwmon").glob("hwmon*/temp*_input")):
        raw = read_text(path)
        if raw.lstrip("-").isdigit():
            values.append({"path": str(path), "millidegrees_c": int(raw)})
    return values


def throttle_counters() -> dict[str, int]:
    values = {}
    for path in sorted(pathlib.Path("/sys/devices/system/cpu").glob("cpu*/thermal_throttle/*")):
        raw = read_text(path)
        if raw.isdigit():
            values[str(path)] = int(raw)
    return values


def host_snapshot() -> dict[str, Any]:
    thermal = temperatures()
    return {
        "monotonic_ns": time.perf_counter_ns(),
        "loadavg": read_text(pathlib.Path("/proc/loadavg")),
        "temperatures": thermal,
        "maximum_temperature_c": max((item["millidegrees_c"] for item in thermal), default=0) / 1000.0,
        "throttle_counters": throttle_counters(),
        "online_cpus": read_text(pathlib.Path("/sys/devices/system/cpu/online")),
        "perf_event_max_sample_rate": read_text(pathlib.Path("/proc/sys/kernel/perf_event_max_sample_rate")),
    }


def thermal_drift(before: Mapping[str, Any], after: Mapping[str, Any]) -> dict[str, list[int]]:
    result = {}
    left = before["throttle_counters"]
    right = after["throttle_counters"]
    for key in sorted(set(left) | set(right)):
        if left.get(key) != right.get(key):
            result[key] = [left.get(key, 0), right.get(key, 0)]
    return result


def decode_input(payload: Mapping[str, Any], key: str, digest: str) -> bytes:
    value = base64.b64decode(payload.get(key, ""), validate=True)
    need(sha256_bytes(value) == digest, f"payload SHA drift: {key}")
    return value


def marker_body(route: str) -> dict[str, Any]:
    return {
        "one_shot": True,
        "retry_permitted": False,
        "route": route,
        "schema": "lay.v10.e1-traversal-d5-marker.v1",
        "task_id": TASK_ID,
        "transaction_id": TRANSACTION_ID,
    }


def verify_file(path: pathlib.Path, digest: str, size: int, mode: str) -> dict[str, Any]:
    value = row(path)
    need(
        value["sha256"] == digest and value["size_bytes"] == size and value["mode"] == mode,
        f"file identity drift: {path}",
    )
    return value


def d2_marker_projection() -> list[dict[str, Any]]:
    marker_dir = D2_STATE / "markers"
    values = [{**row(path), "name": path.name} for path in sorted(marker_dir.iterdir())]
    observed = {item["name"]: item for item in values}
    need(set(observed) == set(D2_MARKERS), f"D2 marker membership drift: {sorted(observed)}")
    for name, (digest, size) in D2_MARKERS.items():
        item = observed[name]
        need(
            item["sha256"] == digest and item["size_bytes"] == size and item["mode"] == "0400",
            f"D2 marker identity drift: {name}",
        )
        body = json.loads((marker_dir / name).read_text())
        need(body.get("task_id") == D2_TASK_ID, f"D2 marker task drift: {name}")
        need(body.get("transaction_id") == D2_TRANSACTION_ID, f"D2 marker transaction drift: {name}")
        need(body.get("retry_permitted") is False, f"D2 marker retry drift: {name}")
    return values


def verify_d2_projection() -> dict[str, Any]:
    elf = verify_file(ELF, ELF_SHA256, ELF_SIZE, "0555")
    map_value = verify_file(MAP, MAP_SHA256, 390_324, "0444")
    inputs = []
    for path, (digest, size) in INPUTS.items():
        inputs.append(verify_file(path, digest, size, "0444"))
    state_path = D2_STATE / "T_SINGLE_STATE.json"
    state_value = json.loads(state_path.read_text())
    need(state_value.get("state") == "BLOCKED_PROVENANCE", "D2 terminal state drift")
    need(state_value.get("receipt_sha256") == "afaeb7d3caffb1967dd76021e42b94664803cef5d0ed72ec574fb54526a8fa0d", "D2 T receipt drift")
    need(state_value.get("retry_permitted") is False, "D2 retry authority drift")
    return {
        "elf": elf,
        "map": map_value,
        "inputs": inputs,
        "markers": d2_marker_projection(),
        "t_single_state": {**row(state_path), "value": state_value},
        "retired_unconsumed": ["t-fixed.available", "t-reversed.available"],
    }


def verify_d4_projection() -> dict[str, Any]:
    need(
        sorted(path.name for path in D4_PARENT.iterdir())
        == ["bootstrap-v1", "marker-creation-v1", "t3-single-v1", "u3-single-v1"],
        "D4 evidence membership drift",
    )
    need(
        sorted(path.name for path in D4_STATE.iterdir())
        == [
            "MARKER_STATE.json",
            "STATE.json",
            "T3_SINGLE_STATE.json",
            "U3_SINGLE_STATE.json",
            "markers",
            "route.lock",
        ],
        "D4 state membership drift",
    )
    expected_markers = {
        "u3-single.consumed-before-exec": (
            "4484826fab0137fcc7e41e146891b110a666a865e7594eb008784ddd2c2154e9",
            287,
            "U3-SINGLE",
        ),
        "t3-single.consumed-before-exec": (
            "ff975bfdb78ca675903a6ee123134594cd4ec88391c17be7f41098174d45ffd6",
            287,
            "T3-SINGLE",
        ),
    }
    markers = []
    observed = {path.name: path for path in (D4_STATE / "markers").iterdir()}
    need(set(observed) == set(expected_markers), "D4 marker membership drift")
    for name, (digest, size, route) in expected_markers.items():
        identity = row(observed[name])
        need(
            identity["sha256"] == digest
            and identity["size_bytes"] == size
            and identity["mode"] == "0400",
            f"D4 marker identity drift: {name}",
        )
        body = json.loads(observed[name].read_text())
        need(
            body.get("task_id") == D4_TASK_ID
            and body.get("transaction_id") == D4_TRANSACTION_ID
            and body.get("route") == route
            and body.get("retry_permitted") is False,
            f"D4 marker body drift: {name}",
        )
        markers.append({**identity, "name": name})
    route_rows = {}
    for route, state_name, result_name, verdict, digest in (
        ("U3-SINGLE", "U3_SINGLE_STATE.json", "u3-single-v1", "U3_SINGLE_PASS", D4_U3_RECEIPT_SHA256),
        ("T3-SINGLE", "T3_SINGLE_STATE.json", "t3-single-v1", "D4_SINGLE_ESTIMATOR_PASS", D4_T3_RECEIPT_SHA256),
    ):
        route_state_path = D4_STATE / state_name
        receipt_path = D4_PARENT / result_name / "D4_ROUTE_RECEIPT.json"
        state_value = json.loads(route_state_path.read_text())
        receipt_value = json.loads(receipt_path.read_text())
        need(
            state_value.get("task_id") == D4_TASK_ID
            and state_value.get("transaction_id") == D4_TRANSACTION_ID
            and state_value.get("route") == route
            and state_value.get("state") == verdict
            and state_value.get("receipt_sha256") == digest
            and state_value.get("retry_permitted") is False,
            f"D4 route state drift: {route}",
        )
        need(
            sha256_file(receipt_path) == digest
            and receipt_value.get("verdict") == verdict
            and receipt_value.get("retry_permitted") is False,
            f"D4 route receipt drift: {route}",
        )
        route_rows[route] = {"state": row(route_state_path), "receipt": row(receipt_path)}
    return {"markers": sorted(markers, key=lambda item: item["name"]), "routes": route_rows}


def verify_host() -> dict[str, Any]:
    need(os.geteuid() == 0, "D5 remote controller requires root")
    need(os.uname().nodename == HOSTNAME, "hostname drift")
    need(sha256_file(pathlib.Path("/etc/machine-id")) == MACHINE_ID_SHA256, "machine identity drift")
    need(os.uname().release == "6.8.0-124-generic", "kernel drift")
    need(read_text(pathlib.Path("/sys/devices/system/cpu/online")) == "0-19", "online CPU drift")
    need(read_text(pathlib.Path("/sys/bus/event_source/devices/cpu_core/cpus")) == "0-11", "core CPU drift")
    need(read_text(pathlib.Path("/sys/bus/event_source/devices/cpu_atom/cpus")) == "12-19", "atom CPU drift")
    need(read_text(pathlib.Path("/proc/sys/kernel/perf_event_max_sample_rate")) == HOST_SAMPLE_RATE, "sample-rate drift")
    verify_file(PERF_WRAPPER, PERF_WRAPPER_SHA256, 1_622, "0755")
    verify_file(PERF_RESOLVED, PERF_RESOLVED_SHA256, 11_627_592, "0755")
    verify_file(TASKSET, TASKSET_SHA256, 22_912, "0755")
    need(sha256_file(SUDO) == SUDO_SHA256, "sudo drift")
    need(sha256_file(LOADER) == LOADER_SHA256, "loader drift")
    return {
        "hostname": HOSTNAME,
        "machine_id_sha256": MACHINE_ID_SHA256,
        "kernel": os.uname().release,
        "online_cpus": "0-19",
        "core_pmu_cpus": "0-11",
        "atom_pmu_cpus": "12-19",
        "perf_event_max_sample_rate": HOST_SAMPLE_RATE,
        "perf_wrapper_sha256": PERF_WRAPPER_SHA256,
        "resolved_perf_sha256": PERF_RESOLVED_SHA256,
        "taskset_sha256": TASKSET_SHA256,
        "host_policy_tuning": False,
    }


def active_d5_subjects() -> list[int]:
    route_environment = {f"LAY_V10_D1_RUN_ID={route}".encode() for route in ROUTE_ORDER}
    active = []
    for process in pathlib.Path("/proc").iterdir():
        if not process.name.isdigit():
            continue
        try:
            entries = set((process / "environ").read_bytes().split(b"\0"))
        except (FileNotFoundError, PermissionError, ProcessLookupError):
            continue
        if entries & route_environment:
            active.append(int(process.name))
    return sorted(active)


def verify_payload(
    payload: Mapping[str, Any],
    *,
    require_bootstrap_audit: bool,
    require_marker_audit: bool = False,
) -> dict[str, Any]:
    paper = decode_input(payload, "paper_b64", PAPER_SHA256)
    preflight = json.loads(decode_input(payload, "preflight_b64", PREFLIGHT_SHA256))
    preflight_receipt = json.loads(
        decode_input(payload, "preflight_receipt_b64", PREFLIGHT_RECEIPT_SHA256)
    )
    d2_terminal = json.loads(decode_input(payload, "d2_terminal_b64", D2_TERMINAL_SHA256))
    d3_terminal = json.loads(decode_input(payload, "d3_terminal_b64", D3_TERMINAL_SHA256))
    d4_terminal = json.loads(decode_input(payload, "d4_terminal_b64", D4_TERMINAL_SHA256))
    need(
        preflight.get("task_id")
        == "LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_D5_MULTIWORKER_TID_IMPLEMENTATION_V3_2026-08-26",
        "preflight task drift",
    )
    need(
        preflight_receipt.get("verdict") == "READY_TO_IMPLEMENT"
        and preflight_receipt.get("safe_to_implement") is True,
        "preflight receipt drift",
    )
    need(d2_terminal.get("verdict") == "BLOCKED_PROVENANCE", "D2 terminal receipt verdict drift")
    need(d3_terminal.get("verdict") == "BLOCKED_PROVENANCE", "D3 terminal receipt verdict drift")
    need(d3_terminal.get("retry_permitted") is False, "D3 retry authority drift")
    need(d4_terminal.get("verdict") == "D4_SINGLE_ESTIMATOR_PASS", "D4 terminal receipt verdict drift")
    need(d4_terminal.get("retry_permitted") is False, "D4 retry authority drift")
    need(d4_terminal.get("u3_receipt_sha256") == D4_U3_RECEIPT_SHA256, "D4 U3 receipt drift")
    need(d4_terminal.get("t3_receipt_sha256") == D4_T3_RECEIPT_SHA256, "D4 T3 receipt drift")
    local_source = decode_input(payload, "local_controller_b64", payload["local_controller_sha256"])
    remote_source = decode_input(payload, "remote_controller_b64", payload["remote_controller_sha256"])
    need(paper.startswith(b"# D5 - multiworker TID estimator V1\n"), "paper identity drift")
    result: dict[str, Any] = {
        "local_controller_sha256": sha256_bytes(local_source),
        "remote_controller_sha256": sha256_bytes(remote_source),
        "preflight_sha256": PREFLIGHT_SHA256,
        "preflight_receipt_sha256": PREFLIGHT_RECEIPT_SHA256,
        "paper_sha256": PAPER_SHA256,
        "d2_terminal_sha256": D2_TERMINAL_SHA256,
        "d3_terminal_sha256": D3_TERMINAL_SHA256,
        "d4_terminal_sha256": D4_TERMINAL_SHA256,
    }
    if require_bootstrap_audit:
        audit_bytes = base64.b64decode(payload.get("bootstrap_audit_b64", ""), validate=True)
        need(sha256_bytes(audit_bytes) == payload.get("bootstrap_audit_sha256"), "bootstrap audit payload SHA drift")
        audit = json.loads(audit_bytes)
        need(audit.get("verdict") == "D5_UID_ACCESS_AUDIT_PASS_MARKER_CREATION", "bootstrap audit verdict drift")
        need(audit.get("task_id") == TASK_ID and audit.get("transaction_id") == TRANSACTION_ID, "bootstrap audit namespace drift")
        need(audit.get("local_controller_sha256") == sha256_bytes(local_source), "audited local controller drift")
        need(audit.get("remote_controller_sha256") == sha256_bytes(remote_source), "audited remote controller drift")
        need(
            audit.get("remote_writes") == 0
            and audit.get("markers_created") == 0
            and audit.get("markers_consumed") == 0
            and audit.get("scp_as_e_pass") is True,
            "bootstrap audit side-effect drift",
        )
        result["bootstrap_audit_sha256"] = sha256_bytes(audit_bytes)
        result["bootstrap_audit"] = audit
    if require_marker_audit:
        audit_bytes = base64.b64decode(payload.get("marker_audit_b64", ""), validate=True)
        need(sha256_bytes(audit_bytes) == payload.get("marker_audit_sha256"), "marker audit payload SHA drift")
        audit = json.loads(audit_bytes)
        need(audit.get("verdict") == "D5_MARKER_AUDIT_PASS_U4_FIXED_ADMITTED", "marker audit verdict drift")
        need(audit.get("task_id") == TASK_ID and audit.get("transaction_id") == TRANSACTION_ID, "marker audit namespace drift")
        need(audit.get("markers_created") == 4 and audit.get("markers_consumed") == 0, "marker audit ledger drift")
        need(audit.get("remote_writes") == 0, "marker audit remote write drift")
        need(
            audit.get("local_controller_sha256") == sha256_bytes(local_source)
            and audit.get("remote_controller_sha256") == sha256_bytes(remote_source)
            and audit.get("bootstrap_audit_sha256")
            == result.get("bootstrap_audit_sha256"),
            "marker audit controller or bootstrap provenance drift",
        )
        result["marker_audit_sha256"] = sha256_bytes(audit_bytes)
        result["marker_audit"] = audit
    return result


def d5_marker_projection() -> list[dict[str, Any]]:
    values = []
    for path in sorted((STATE / "markers").iterdir()):
        values.append({**row(path), "name": path.name, "value": json.loads(path.read_text())})
    return values


def expected_d5_markers(route: str | None = None, *, post: bool = False) -> dict[str, tuple[str, int]]:
    expected = {}
    for name in ROUTE_ORDER:
        spec = ROUTES[name]
        consumed = False
        if route is not None:
            current = ROUTE_ORDER.index(route)
            index = ROUTE_ORDER.index(name)
            consumed = index < current or (post and index == current)
        suffix = "consumed-before-exec" if consumed else "available"
        expected[f"{spec['marker']}.{suffix}"] = (spec["marker_sha256"], spec["marker_size"])
    return expected


def verify_d5_markers(route: str | None = None, *, post: bool = False) -> list[dict[str, Any]]:
    expected = expected_d5_markers(route, post=post)
    values = d5_marker_projection()
    observed = {item["name"]: item for item in values}
    need(set(observed) == set(expected), f"D5 marker membership drift: {sorted(observed)}")
    for name, (digest, size) in expected.items():
        item = observed[name]
        need(
            item["sha256"] == digest and item["size_bytes"] == size and item["mode"] == "0400",
            f"D5 marker identity drift: {name}",
        )
        body = item["value"]
        marker_route = name.split(".", 1)[0]
        expected_route = next(
            candidate for candidate in ROUTE_ORDER if ROUTES[candidate]["marker"] == marker_route
        )
        need(
            body.get("task_id") == TASK_ID
            and body.get("transaction_id") == TRANSACTION_ID
            and body.get("route") == expected_route,
            f"D5 marker namespace or route drift: {name}",
        )
        need(body.get("one_shot") is True and body.get("retry_permitted") is False, f"D5 marker authority drift: {name}")
    return values


def verify_bootstrap_base(
    payload: Mapping[str, Any], *, require_bootstrap_audit: bool
) -> dict[str, Any]:
    payload_check = verify_payload(
        payload, require_bootstrap_audit=require_bootstrap_audit
    )
    need(PARENT.is_dir() and STATE.is_dir(), "D5 namespace missing")
    need(mode_string(PARENT) == "0755" and mode_string(STATE) == "0755", "D5 parent mode drift")
    bootstrap = PARENT / "bootstrap-v1"
    need(bootstrap.is_dir(), "D5 bootstrap evidence missing")
    verify_sha256sums(bootstrap)
    receipt_path = bootstrap / "D5_BOOTSTRAP_RECEIPT.json"
    receipt = json.loads(receipt_path.read_text())
    need(
        receipt.get("task_id") == TASK_ID
        and receipt.get("transaction_id") == TRANSACTION_ID
        and receipt.get("verdict") == "D5_UID_PROOF_CREATED_UNAUDITED",
        "bootstrap receipt namespace or verdict drift",
    )
    need(
        receipt.get("markers_expected") == 4
        and receipt.get("markers_created") == 0
        and receipt.get("markers_consumed") == 0,
        "bootstrap scientific marker ledger drift",
    )
    receipt_payload = receipt.get("payload", {})
    need(
        receipt_payload.get("local_controller_sha256")
        == payload_check["local_controller_sha256"]
        and receipt_payload.get("remote_controller_sha256")
        == payload_check["remote_controller_sha256"],
        "bootstrap controller provenance drift",
    )
    if require_bootstrap_audit:
        need(
            payload_check["bootstrap_audit"].get("bootstrap_receipt_sha256")
            == sha256_file(receipt_path),
            "bootstrap audit does not identify the authoritative bootstrap receipt",
        )
    state_path = STATE / "STATE.json"
    state_value = json.loads(state_path.read_text())
    need(
        state_value.get("task_id") == TASK_ID
        and state_value.get("transaction_id") == TRANSACTION_ID
        and state_value.get("state") == "D5_UID_PROOF_CREATED_UNAUDITED"
        and state_value.get("markers_expected") == 4
        and state_value.get("markers_created") == 0
        and state_value.get("markers_consumed") == 0
        and state_value.get("retry_permitted") is False,
        "D5 bootstrap state drift",
    )
    return {
        "payload": payload_check,
        "bootstrap_receipt": row(receipt_path),
        "bootstrap_value": receipt,
        "state": {**row(state_path), "value": state_value},
    }


def verify_bootstrap(payload: Mapping[str, Any], route: str | None = None, *, post: bool = False) -> dict[str, Any]:
    base = verify_bootstrap_base(payload, require_bootstrap_audit=True)
    payload_check = verify_payload(
        payload, require_bootstrap_audit=True, require_marker_audit=True
    )
    creation = PARENT / "marker-creation-v1"
    need(creation.is_dir(), "D5 marker creation evidence missing")
    verify_sha256sums(creation)
    creation_receipt_path = creation / "D5_MARKER_CREATION_RECEIPT.json"
    creation_receipt = json.loads(creation_receipt_path.read_text())
    need(
        creation_receipt.get("task_id") == TASK_ID
        and creation_receipt.get("transaction_id") == TRANSACTION_ID
        and creation_receipt.get("verdict") == "D5_MARKERS_CREATED_UNAUDITED"
        and creation_receipt.get("markers_expected") == 4
        and creation_receipt.get("markers_created") == 4
        and creation_receipt.get("markers_consumed") == 0,
        "marker creation namespace, verdict, or ledger drift",
    )
    need(
        creation_receipt.get("bootstrap_audit_sha256")
        == payload_check["bootstrap_audit_sha256"],
        "marker creation admission drift",
    )
    need(
        payload_check["marker_audit"].get("marker_creation_receipt_sha256")
        == sha256_file(creation_receipt_path),
        "marker audit does not identify the authoritative creation receipt",
    )
    marker_state_path = STATE / "MARKER_STATE.json"
    marker_state = json.loads(marker_state_path.read_text())
    need(
        marker_state.get("task_id") == TASK_ID
        and marker_state.get("transaction_id") == TRANSACTION_ID
        and marker_state.get("state") == "D5_MARKERS_CREATED_UNAUDITED"
        and marker_state.get("bootstrap_audit_sha256")
        == payload_check["bootstrap_audit_sha256"]
        and marker_state.get("markers_created") == 4
        and marker_state.get("markers_consumed") == 0
        and marker_state.get("retry_permitted") is False,
        "marker state drift",
    )
    markers = verify_d5_markers(route, post=post)
    route_index = ROUTE_ORDER.index(route) if route is not None else 0
    if route is not None:
        for previous in ROUTE_ORDER[:route_index]:
            previous_state_path = state_path(previous)
            previous_receipt_path = result_path(previous) / "D5_ROUTE_RECEIPT.json"
            need(previous_state_path.is_file(), f"previous route state missing: {previous}")
            need(previous_receipt_path.is_file(), f"previous route receipt missing: {previous}")
            need(not failure_path(previous).exists(), f"previous route failure evidence exists: {previous}")
            verify_sha256sums(result_path(previous))
            previous_state = json.loads(previous_state_path.read_text())
            previous_receipt = json.loads(previous_receipt_path.read_text())
            expected_verdict = ROUTES[previous]["pass_state"]
            need(
                previous_state.get("task_id") == TASK_ID
                and previous_state.get("transaction_id") == TRANSACTION_ID
                and previous_state.get("route") == previous
                and previous_state.get("state") == expected_verdict
                and previous_state.get("retry_permitted") is False,
                f"previous route state did not close at PASS: {previous}",
            )
            need(
                previous_receipt.get("task_id") == TASK_ID
                and previous_receipt.get("transaction_id") == TRANSACTION_ID
                and previous_receipt.get("route") == previous
                and previous_receipt.get("verdict") == expected_verdict
                and previous_receipt.get("retry_permitted") is False,
                f"previous route receipt did not close at PASS: {previous}",
            )
            need(
                previous_state.get("receipt_sha256") == sha256_file(previous_receipt_path),
                f"previous state receipt drift: {previous}",
            )
            need(
                sha256_file(result_path(previous) / "inputs/local-controller.py")
                == payload_check["local_controller_sha256"]
                and sha256_file(result_path(previous) / "inputs/remote-controller.py")
                == payload_check["remote_controller_sha256"]
                and previous_receipt.get("bootstrap_audit_sha256")
                == payload_check["bootstrap_audit_sha256"]
                and previous_receipt.get("marker_audit_sha256")
                == payload_check["marker_audit_sha256"],
                f"previous route controller or admission provenance drift: {previous}",
            )
        if post:
            current_state_path = state_path(route)
            current_receipt_path = result_path(route) / "D5_ROUTE_RECEIPT.json"
            need(current_state_path.is_file(), f"current route state missing: {route}")
            need(current_receipt_path.is_file(), f"current route receipt missing: {route}")
            need(not failure_path(route).exists(), f"current route failure evidence exists: {route}")
            verify_sha256sums(result_path(route))
            current_state = json.loads(current_state_path.read_text())
            current_receipt = json.loads(current_receipt_path.read_text())
            need(
                current_state.get("state") == current_receipt.get("verdict")
                and current_state.get("receipt_sha256") == sha256_file(current_receipt_path)
                and current_state.get("retry_permitted") is False,
                f"current route state/receipt drift: {route}",
            )
        else:
            for pending in ROUTE_ORDER[route_index:]:
                need(not state_path(pending).exists(), f"pending route state already exists: {pending}")
                need(not result_path(pending).exists(), f"pending route result already exists: {pending}")
                need(not failure_path(pending).exists(), f"pending route failure already exists: {pending}")
    return {
        **base,
        "payload": payload_check,
        "marker_creation_receipt": row(creation_receipt_path),
        "marker_state": {**row(marker_state_path), "value": marker_state},
        "markers": markers,
    }


def run_uid_probe(subject: pathlib.Path) -> dict[str, Any]:
    source = r'''
import hashlib,json,os,pathlib,stat,sys
subject,elf,bucket_map=map(pathlib.Path,sys.argv[1:4])
fixed=b"D5 UID E CAPABILITY PROBE V1\n"
ancestors=[]
cursor=subject
while True:
    value=cursor.stat()
    ancestors.append({"path":str(cursor),"mode":f"{stat.S_IMODE(value.st_mode):04o}","uid":value.st_uid,"gid":value.st_gid})
    if cursor==cursor.parent: break
    cursor=cursor.parent
def digest(path):
    h=hashlib.sha256()
    with path.open("rb") as stream:
        for block in iter(lambda:stream.read(1024*1024),b""): h.update(block)
    return h.hexdigest()
elf_sha=digest(elf); map_sha=digest(bucket_map)
first=subject/"capability-probe.tmp"; renamed=subject/"capability-probe.renamed"
fd=os.open(first,os.O_WRONLY|os.O_CREAT|os.O_EXCL,0o600)
try:
    os.write(fd,fixed); os.fsync(fd)
finally: os.close(fd)
os.rename(first,renamed)
directory=os.open(subject,os.O_RDONLY|os.O_DIRECTORY); os.fsync(directory); os.close(directory)
read_back=renamed.read_bytes()
if read_back!=fixed: raise RuntimeError("renamed probe byte drift")
renamed.unlink()
directory=os.open(subject,os.O_RDONLY|os.O_DIRECTORY); os.fsync(directory); os.close(directory)
bytes_path=subject/"UID_PROBE_BYTES.bin"
fd=os.open(bytes_path,os.O_WRONLY|os.O_CREAT|os.O_EXCL,0o600)
try: os.write(fd,fixed); os.fsync(fd)
finally: os.close(fd)
receipt={"schema":"lay.v10.e1-traversal-d5-uid-probe.v1","uid":os.getuid(),"gid":os.getgid(),"fixed_sha256":hashlib.sha256(fixed).hexdigest(),"fixed_size":len(fixed),"elf_sha256":elf_sha,"map_sha256":map_sha,"ancestors":ancestors,"operations":["traverse","stat","read","create","write","fsync-file","rename","fsync-directory","reopen","read-exact","unlink","fsync-directory"],"verdict":"D5_UID_E_CAPABILITY_PROOF_PASS"}
payload=(json.dumps(receipt,indent=2,sort_keys=True)+"\n").encode()
receipt_path=subject/"UID_PROBE.json"
fd=os.open(receipt_path,os.O_WRONLY|os.O_CREAT|os.O_EXCL,0o600)
try: os.write(fd,payload); os.fsync(fd)
finally: os.close(fd)
directory=os.open(subject,os.O_RDONLY|os.O_DIRECTORY); os.fsync(directory); os.close(directory)
print(json.dumps(receipt,sort_keys=True,separators=(",",":")))
'''
    command = [
        str(SUDO), "-n", "-u", "e", "/usr/bin/python3", "-c", source,
        str(subject), str(ELF), str(MAP),
    ]
    result = subprocess.run(command, stdout=subprocess.PIPE, stderr=subprocess.PIPE, timeout=1800, check=False)
    need(result.returncode == 0, f"UID e probe failed: {result.stderr.decode(errors='replace')[-4000:]}")
    lines = result.stdout.decode().strip().splitlines()
    need(lines, "UID e probe produced no receipt")
    value = json.loads(lines[-1])
    need(value.get("verdict") == "D5_UID_E_CAPABILITY_PROOF_PASS", "UID probe verdict drift")
    need(value.get("uid") == pwd.getpwnam("e").pw_uid, "UID probe identity drift")
    need(value.get("elf_sha256") == ELF_SHA256 and value.get("map_sha256") == MAP_SHA256, "UID readable input drift")
    need((subject / "UID_PROBE_BYTES.bin").read_bytes() == b"D5 UID E CAPABILITY PROBE V1\n", "UID probe bytes drift")
    need(json.loads((subject / "UID_PROBE.json").read_text()) == value, "UID probe receipt drift")
    return {"value": value, "stdout_sha256": sha256_bytes(result.stdout), "stderr_sha256": sha256_bytes(result.stderr)}


def bootstrap(payload: Mapping[str, Any]) -> dict[str, Any]:
    host = verify_host()
    d2 = verify_d2_projection()
    d4 = verify_d4_projection()
    payload_check = verify_payload(payload, require_bootstrap_audit=False)
    need(not PARENT.exists() and not STATE.exists(), "D5 namespace already exists")
    parent_stage = pathlib.Path(f"{PARENT}.stage-{os.getpid()}-{time.time_ns()}")
    state_stage = pathlib.Path(f"{STATE}.stage-{os.getpid()}-{time.time_ns()}")
    parent_stage.mkdir(parents=True, mode=0o755)
    state_stage.mkdir(parents=True, mode=0o755)
    try:
        evidence = parent_stage / "bootstrap-v1"
        inputs = evidence / "inputs"
        subject = evidence / "subject"
        evidence.mkdir(mode=0o755)
        inputs.mkdir(mode=0o700)
        subject.mkdir(mode=0o700)
        account = pwd.getpwnam("e")
        os.chown(subject, account.pw_uid, account.pw_gid)
        write_new_bytes(inputs / "paper.md", decode_input(payload, "paper_b64", PAPER_SHA256))
        write_new_bytes(inputs / "preflight-v3.json", decode_input(payload, "preflight_b64", PREFLIGHT_SHA256))
        write_new_bytes(
            inputs / "preflight-v3-receipt.json",
            decode_input(payload, "preflight_receipt_b64", PREFLIGHT_RECEIPT_SHA256),
        )
        write_new_bytes(inputs / "d2-terminal-receipt.json", decode_input(payload, "d2_terminal_b64", D2_TERMINAL_SHA256))
        write_new_bytes(inputs / "d3-terminal-receipt.json", decode_input(payload, "d3_terminal_b64", D3_TERMINAL_SHA256))
        write_new_bytes(inputs / "d4-terminal-receipt.json", decode_input(payload, "d4_terminal_b64", D4_TERMINAL_SHA256))
        write_new_bytes(
            inputs / "local-controller.py",
            decode_input(payload, "local_controller_b64", payload["local_controller_sha256"]),
        )
        write_new_bytes(
            inputs / "remote-controller.py",
            decode_input(payload, "remote_controller_b64", payload["remote_controller_sha256"]),
        )
        uid_probe = run_uid_probe(subject)
        write_new_bytes(state_stage / "route.lock", b"d5-multiworker-tid-v1\n", 0o400)
        write_new_json(
            state_stage / "STATE.json",
            {
                "schema": "lay.v10.e1-traversal-d5-state.v1",
                "task_id": TASK_ID,
                "transaction_id": TRANSACTION_ID,
                "state": "D5_UID_PROOF_CREATED_UNAUDITED",
                "markers_expected": 4,
                "markers_created": 0,
                "markers_consumed": 0,
                "retry_permitted": False,
            },
            0o400,
        )
        receipt = {
            "schema": "lay.v10.e1-traversal-d5-bootstrap.v1",
            "task_id": TASK_ID,
            "transaction_id": TRANSACTION_ID,
            "verdict": "D5_UID_PROOF_CREATED_UNAUDITED",
            "payload": payload_check,
            "host": host,
            "d2_projection": d2,
            "d4_projection": d4,
            "uid_probe": uid_probe,
            "parent_mode": "0755",
            "subject_uid": account.pw_uid,
            "markers_expected": 4,
            "markers_created": 0,
            "markers_consumed": 0,
            "cargo_invocations": 0,
            "rustc_compilations": 0,
            "perf_record": 0,
            "perf_stat": 0,
            "pmu_events_opened": 0,
            "subject_executions": 0,
            "d2_state_mutations": 0,
            "runtime_authority_changed": False,
            "next_action_admitted": "independent D5 bootstrap audit only",
        }
        write_new_json(evidence / "D5_BOOTSTRAP_RECEIPT.json", receipt)
        write_sha256sums(evidence)
        seal_evidence_tree(evidence)
        fsync_directory(parent_stage)
        fsync_directory(state_stage)
        os.rename(parent_stage, PARENT)
        fsync_directory(PARENT.parent)
        os.rename(state_stage, STATE)
        fsync_directory(STATE.parent)
        published = PARENT / "bootstrap-v1/D5_BOOTSTRAP_RECEIPT.json"
        return {
            **receipt,
            "published_receipt_sha256": sha256_file(published),
            "remote_parent": str(PARENT),
            "remote_state": str(STATE),
        }
    except BaseException:
        if parent_stage.exists():
            shutil.rmtree(parent_stage)
        if state_stage.exists():
            shutil.rmtree(state_stage)
        raise


def create_markers(payload: Mapping[str, Any]) -> dict[str, Any]:
    host = verify_host()
    d2 = verify_d2_projection()
    d4 = verify_d4_projection()
    base = verify_bootstrap_base(payload, require_bootstrap_audit=True)
    need(not (STATE / "markers").exists(), "D5 marker directory already exists")
    need(not (STATE / "MARKER_STATE.json").exists(), "D5 marker state already exists")
    result = PARENT / "marker-creation-v1"
    need(not result.exists(), "D5 marker creation evidence already exists")
    marker_stage = STATE / f"markers.stage-{os.getpid()}-{time.time_ns()}"
    result_stage = pathlib.Path(f"{result}.stage-{os.getpid()}-{time.time_ns()}")
    marker_stage.mkdir(mode=0o700)
    result_stage.mkdir(mode=0o700)
    try:
        marker_rows = []
        for route in ROUTE_ORDER:
            path = marker_stage / f"{ROUTES[route]['marker']}.available"
            write_new_json(path, marker_body(route), 0o400)
            marker_rows.append({**row(path), "name": path.name})
        fsync_directory(marker_stage)
        receipt = {
            "schema": "lay.v10.e1-traversal-d5-marker-creation.v1",
            "task_id": TASK_ID,
            "transaction_id": TRANSACTION_ID,
            "verdict": "D5_MARKERS_CREATED_UNAUDITED",
            "bootstrap_audit_sha256": base["payload"]["bootstrap_audit_sha256"],
            "markers": marker_rows,
            "markers_expected": 4,
            "markers_created": 4,
            "markers_consumed": 0,
            "host": host,
            "d2_projection": d2,
            "d4_projection": d4,
            "subject_executions": 0,
            "perf_record": 0,
            "perf_stat": 0,
            "pmu_events_opened": 0,
            "retry_permitted": False,
            "next_action_admitted": "independent D5 marker audit only",
        }
        write_new_json(result_stage / "D5_MARKER_CREATION_RECEIPT.json", receipt)
        write_sha256sums(result_stage)
        seal_evidence_tree(result_stage)
        os.rename(marker_stage, STATE / "markers")
        fsync_directory(STATE)
        write_new_json(
            STATE / "MARKER_STATE.json",
            {
                "schema": "lay.v10.e1-traversal-d5-marker-state.v1",
                "task_id": TASK_ID,
                "transaction_id": TRANSACTION_ID,
                "state": "D5_MARKERS_CREATED_UNAUDITED",
                "bootstrap_audit_sha256": base["payload"]["bootstrap_audit_sha256"],
                "markers_created": 4,
                "markers_consumed": 0,
                "retry_permitted": False,
            },
            0o400,
        )
        fsync_directory(STATE)
        os.rename(result_stage, result)
        fsync_directory(PARENT)
        published = result / "D5_MARKER_CREATION_RECEIPT.json"
        return {**receipt, "published_receipt_sha256": sha256_file(published)}
    except BaseException:
        if marker_stage.exists():
            shutil.rmtree(marker_stage)
        if result_stage.exists():
            shutil.rmtree(result_stage)
        raise


def controlled_environment(output: pathlib.Path, route: str) -> dict[str, str]:
    spec = ROUTES[route]
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
        "LAY_V10_D1_CPUS": ",".join(str(cpu) for cpu in spec["cpus"]),
    }


def subject_command(route: str) -> list[str]:
    need(route in ROUTES, f"unknown subject route: {route}")
    return [
        str(TASKSET),
        "--cpu-list",
        "0-19",
        str(LOADER),
        str(ELF),
        "--exact",
        SUBJECT_TEST,
        "--ignored",
        "--nocapture",
        "--test-threads=1",
    ]


def child_as_e(environment: Mapping[str, str], command: Sequence[str]) -> list[str]:
    assignments = [f"{key}={value}" for key, value in sorted(environment.items())]
    return ["/usr/bin/sudo", "-n", "-u", "e", "/usr/bin/env", *assignments, *command]


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
    totals = {
        field: sum(int(item.get(field, 0)) for item in queries)
        for field in (
            "expanded_states",
            "examined_edges",
            "surviving_edges",
            "pruned_edges",
            "stack_pushes",
            "stack_pops",
            "terminal_hits_before_merge",
        )
    }
    return {
        "sha256": sha256_file(path),
        "queries": len(queries),
        "totals_per_round": totals,
        "exact_d1_structure": sha256_file(path) == D1_STRUCTURE_SHA256,
    }


def validate_subject(subject: pathlib.Path, route: str) -> dict[str, Any]:
    spec = ROUTES[route]
    receipt_path = subject / "SUBJECT_RECEIPT.json"
    samples_path = subject / "component-samples.bin"
    structure_path = subject / "structure.json"
    need(receipt_path.is_file() and samples_path.is_file() and structure_path.is_file(), "subject evidence incomplete")
    receipt = json.loads(receipt_path.read_text())
    samples = parse_component_samples(samples_path)
    structure = validate_structure(structure_path)
    checks = {
        "schema": receipt.get("schema") == "lay.v10.e1-remaining-cost-d1-component-process.v1",
        "test": receipt.get("test") == SUBJECT_TEST,
        "run_id": receipt.get("run_id") == route,
        "mapping": receipt.get("mapping") == spec["mapping"],
        "queries": receipt.get("queries") == QUERIES,
        "rounds": receipt.get("rounds") == MEASURED_ROUNDS,
        "workers": receipt.get("workers") == 20,
        "warmup_bursts": receipt.get("warmup_bursts") == 1,
        "start_barriers": receipt.get("start_barriers") == SAMPLED_ROUNDS,
        "end_barriers": receipt.get("end_barriers") == SAMPLED_ROUNDS,
        "cpus": receipt.get("cpus") == list(spec["cpus"]),
        "worker_affinities": receipt.get("worker_affinities") == [[cpu] for cpu in spec["cpus"]],
        "worker_migrations": receipt.get("worker_migration_deltas") == [0] * 20,
        "receipt_samples": receipt.get("samples", {}).get("samples") == REQUESTS,
        "receipt_errors": receipt.get("samples", {}).get("errors") == 0,
        "receipt_unresolved": receipt.get("samples", {}).get("unresolved") == 0,
        "phase_order": receipt.get("samples", {}).get("phase_order") == list(PHASES),
        "record_count": samples["records"] == REQUESTS,
        "ordinals": samples["query_ordinals"] == list(range(QUERIES)),
        "round_ids": samples["rounds"] == list(range(MEASURED_ROUNDS)),
        "worker_ids": samples["workers"] == list(range(20)),
        "sample_errors": samples["errors"] == 0,
        "sample_unresolved": samples["unresolved"] == 0,
        "structure_sha": structure["exact_d1_structure"],
        "structure_queries": structure["queries"] == QUERIES,
        "structure_edges": structure["totals_per_round"]["examined_edges"] == EDGES_PER_ROUND,
    }
    return {
        "checks": checks,
        "violations": [name for name, passed in checks.items() if not passed],
        "receipt": receipt,
        "receipt_identity": row(receipt_path),
        "component_samples": row(samples_path),
        "structure": row(structure_path),
        "sample_summary": samples,
        "structure_summary": structure,
        "traversal_thread_cpu_ns": samples["traversal_thread_cpu_ns"],
        "traversal_thread_cpu_per_edge_ns": samples["traversal_thread_cpu_ns"] / U4_EDGES,
        "u4_measured_rounds": MEASURED_ROUNDS,
        "u4_denominator_edges": U4_EDGES,
        "mapping": spec["mapping"],
        "cpus": list(spec["cpus"]),
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


def run_u4(route: str, stage: pathlib.Path, environment: dict[str, str]) -> dict[str, Any]:
    subject = stage / "subject"
    command = child_as_e(environment, subject_command(route))
    before = host_snapshot()
    status = run_command_to_files(command, stage / "subject.stdout", stage / "subject.stderr", timeout=1800)
    after = host_snapshot()
    violations = {name: [] for name, _ in U4_DISPATCH}
    validation: dict[str, Any] = {}
    if status["timed_out"] or status["returncode"] != 0:
        violations["semantic"].append(f"subject status: {status}")
    try:
        validation = validate_subject(subject, route)
        if validation["violations"]:
            violations["semantic"].extend(f"subject {name} mismatch" for name in validation["violations"])
    except Exception as error:
        violations["provenance"].append(f"subject evidence unavailable: {type(error).__name__}: {error}")
    drift = thermal_drift(before, after)
    if drift:
        violations["thermal"].append(f"thermal throttle drift: {drift}")
    if before["perf_event_max_sample_rate"] != HOST_SAMPLE_RATE or after["perf_event_max_sample_rate"] != HOST_SAMPLE_RATE:
        violations["provenance"].append("host sample-rate drift during U4")
    return {
        "schema": "lay.v10.e1-traversal-d5-u4-observation.v1",
        "route": route,
        "complete": not violations["provenance"],
        "command": command,
        "subject_command": subject_command(route),
        "environment": environment,
        "status": status,
        "subject": validation,
        "host_before": before,
        "host_after": after,
        "thermal_throttle_drift": drift,
        "violations": violations,
        "cargo_invocations": 0,
        "perf_record_invocations": 0,
        "perf_reader_invocations": 0,
        "perf_stat_invocations": 0,
        "pmu_events_opened": 0,
        "subject_executions": 1,
    }


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
FORK_PATTERN = re.compile(r"PERF_RECORD_FORK\((\d+):(\d+)\):\((\d+):(\d+)\)")
COMM_PATTERN = re.compile(r"PERF_RECORD_COMM(?: exec)?:\s+(.+):(\d+)/(\d+)")
EXIT_PATTERN = re.compile(r"PERF_RECORD_EXIT\((\d+):(\d+)\):\((\d+):(\d+)\)")


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
    forks = []
    comms = []
    exits = []
    lost = 0
    throttle = 0
    unthrottle = 0
    raw_samples = 0
    with path.open(encoding="utf-8", errors="replace") as source:
        for line in source:
            if "PERF_RECORD_LOST" in line or "PERF_RECORD_LOST_SAMPLES" in line:
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
            match = FORK_PATTERN.search(line)
            if match is not None:
                forks.append(
                    {
                        "pid": int(match.group(1)),
                        "tid": int(match.group(2)),
                        "parent_pid": int(match.group(3)),
                        "parent_tid": int(match.group(4)),
                    }
                )
            match = COMM_PATTERN.search(line)
            if match is not None:
                comms.append(
                    {"comm": match.group(1), "pid": int(match.group(2)), "tid": int(match.group(3))}
                )
            match = EXIT_PATTERN.search(line)
            if match is not None:
                exits.append(
                    {
                        "pid": int(match.group(1)),
                        "tid": int(match.group(2)),
                        "parent_pid": int(match.group(3)),
                        "parent_tid": int(match.group(4)),
                    }
                )
    return {
        "mappings": mappings,
        "forks": forks,
        "comms": comms,
        "exits": exits,
        "lost_records": lost,
        "throttle_records": throttle,
        "unthrottle_records": unthrottle,
        "raw_sample_records": raw_samples,
    }


def reconstruct_worker_tids(
    raw: Mapping[str, Any], mapping: Mapping[str, Any], samples: Sequence[Mapping[str, Any]]
) -> dict[str, Any]:
    subject_pid = int(mapping["pid"])
    subject_forks = [
        value
        for value in raw["forks"]
        if value["pid"] == subject_pid and value["parent_pid"] == subject_pid
    ]
    children: dict[int, list[int]] = {}
    for value in subject_forks:
        children.setdefault(int(value["parent_tid"]), []).append(int(value["tid"]))
    direct = sorted(set(children.get(subject_pid, [])))
    candidates = []
    for parent_tid in direct:
        worker_tids = sorted(set(children.get(parent_tid, [])))
        if len(worker_tids) == 20:
            candidates.append((parent_tid, worker_tids))
    need(len(candidates) == 1, f"twenty-worker parent candidate count: {len(candidates)}")
    test_parent_tid, worker_tids = candidates[0]
    need(direct == [test_parent_tid], f"subject leader direct children drift: {direct}")
    need(
        all(not children.get(worker_tid) for worker_tid in worker_tids),
        "worker child-thread graph is not terminal",
    )
    need(len(subject_forks) == 21, f"subject FORK record count: {len(subject_forks)}")
    worker_exits = [
        value
        for value in raw["exits"]
        if value["pid"] == subject_pid and int(value["tid"]) in worker_tids
    ]
    exit_tids = [int(value["tid"]) for value in worker_exits]
    need(
        len(worker_exits) == 20 and sorted(exit_tids) == worker_tids,
        f"worker EXIT record closure drift: {sorted(exit_tids)}",
    )
    subject_tids = {subject_pid, test_parent_tid, *worker_tids}
    subject_comms = [value for value in raw["comms"] if value["pid"] == subject_pid]
    unknown_comm_tids = sorted({int(value["tid"]) for value in subject_comms} - subject_tids)
    need(not unknown_comm_tids, f"subject COMM records reference unknown TIDs: {unknown_comm_tids}")
    need(
        any(int(value["tid"]) == test_parent_tid for value in subject_comms),
        "libtest parent COMM record missing",
    )
    sample_cpus: dict[int, set[int]] = {tid: set() for tid in worker_tids}
    sample_counts: dict[int, int] = {tid: 0 for tid in worker_tids}
    for sample in samples:
        tid = int(sample["tid"])
        if tid in sample_cpus:
            sample_cpus[tid].add(int(sample["cpu"]))
            sample_counts[tid] += 1
    need(all(sample_counts.values()), "one or more workers emitted no samples")
    need(
        all(len(cpus) == 1 for cpus in sample_cpus.values()),
        f"worker sample CPU sets are not singleton: {sample_cpus}",
    )
    worker_cpu = {tid: next(iter(cpus)) for tid, cpus in sample_cpus.items()}
    need(
        sorted(worker_cpu.values()) == list(range(20)),
        f"worker sample CPU closure drift: {sorted(worker_cpu.values())}",
    )
    return {
        "subject_pid": subject_pid,
        "test_parent_tid": test_parent_tid,
        "worker_tids": worker_tids,
        "worker_count": len(worker_tids),
        "worker_sample_counts": {str(key): sample_counts[key] for key in worker_tids},
        "worker_sample_cpus": {str(key): worker_cpu[key] for key in worker_tids},
        "subject_fork_records": subject_forks,
        "subject_comm_records": subject_comms,
        "worker_exit_records": worker_exits,
        "unique": True,
    }


def verify_map_bytes(map_value: Mapping[str, Any]) -> dict[str, Any]:
    need(map_value.get("elf_sha256") == ELF_SHA256, "map ELF SHA drift")
    need(map_value.get("build_id") == ELF_BUILD_ID, "map Build ID drift")
    need(map_value.get("address_space") == "ELF virtual address", "map address space drift")
    segment = map_value.get("executable_pt_load", {})
    ranges = map_value.get("ranges")
    need(isinstance(ranges, list) and len(ranges) == 46, "map range count drift")
    text = map_value.get("text", {})
    cursor = text.get("start")
    mismatches = []
    with ELF.open("rb") as source:
        for index, value in enumerate(ranges):
            start = value.get("start")
            end = value.get("end_exclusive")
            need(
                isinstance(start, int) and isinstance(end, int) and start == cursor and end > start,
                f"map range geometry drift: {index}",
            )
            need(value.get("length_bytes") == end - start, f"map range length drift: {index}")
            need(
                value.get("bucket") in TRAVERSAL_BUCKETS | {"OUTSIDE_TRAVERSAL"},
                f"map bucket drift: {index}",
            )
            need(
                segment["vaddr"] <= start < end <= segment["end_exclusive"],
                f"map range outside executable segment: {index}",
            )
            offset = int(segment["offset"]) + start - int(segment["vaddr"])
            source.seek(offset)
            actual = sha256_bytes(source.read(end - start))
            if actual != value.get("machine_bytes_sha256"):
                mismatches.append(
                    {
                        "index": index,
                        "start": start,
                        "expected": value.get("machine_bytes_sha256"),
                        "actual": actual,
                    }
                )
            cursor = end
    need(cursor == text.get("end_exclusive"), "map text coverage drift")
    return {
        "ranges": len(ranges),
        "machine_byte_mismatches": mismatches,
        "covered_bytes": cursor - text["start"],
    }


def attribute_t4(
    samples: list[dict[str, Any]],
    raw: Mapping[str, Any],
    map_value: Mapping[str, Any],
    tid_graph: Mapping[str, Any],
) -> dict[str, Any]:
    segment = map_value["executable_pt_load"]
    page_size = os.sysconf("SC_PAGE_SIZE")
    expected_offset = int(segment["offset"]) // page_size * page_size
    executable = [
        value for value in raw["mappings"] if "x" in value["permissions"] and value["offset"] == expected_offset
    ]
    need(len(executable) == 1, f"executable D2 mapping count: {len(executable)}")
    mapping = executable[0]
    worker_tids = set(tid_graph["worker_tids"])
    aligned_vaddr = int(segment["vaddr"]) // page_size * page_size
    load_bias = mapping["start"] - aligned_vaddr
    ranges = map_value["ranges"]
    starts = [int(value["start"]) for value in ranges]
    accepted_bucket_counts: dict[str, int] = {}
    accepted_sub_bucket_counts: dict[str, int] = {}
    d2_samples = 0
    cpu_counts: dict[int, int] = {}
    accepted_tids: set[int] = set()
    accepted_tid_counts: dict[int, int] = {tid: 0 for tid in worker_tids}
    excluded_non_worker_traversal_samples = 0
    scientific_outside_traversal = 0
    outside_mapped_text = 0
    dso_mismatches = []
    for sample in samples:
        inside_mapping = mapping["start"] <= sample["runtime_ip"] < mapping["start"] + mapping["length"]
        rendered_exact = sample["dso"] == str(ELF)
        if not inside_mapping and not rendered_exact:
            continue
        d2_samples += 1
        cpu_counts[sample["cpu"]] = cpu_counts.get(sample["cpu"], 0) + 1
        if not inside_mapping or not rendered_exact:
            dso_mismatches.append({"runtime_ip": sample["runtime_ip"], "dso": sample["dso"]})
            continue
        normalized = sample["runtime_ip"] - load_bias
        position = bisect.bisect_right(starts, normalized) - 1
        if position < 0 or not (
            int(ranges[position]["start"]) <= normalized < int(ranges[position]["end_exclusive"])
        ):
            outside_mapped_text += 1
            continue
        value = ranges[position]
        bucket = value["bucket"]
        if sample["tid"] not in worker_tids:
            if bucket != "OUTSIDE_TRAVERSAL":
                excluded_non_worker_traversal_samples += 1
            continue
        if bucket == "OUTSIDE_TRAVERSAL":
            scientific_outside_traversal += 1
            continue
        need(bucket in TRAVERSAL_BUCKETS, f"unknown traversal bucket: {bucket}")
        sub_bucket = value["sub_bucket"]
        accepted_bucket_counts[bucket] = accepted_bucket_counts.get(bucket, 0) + 1
        accepted_sub_bucket_counts[sub_bucket] = accepted_sub_bucket_counts.get(sub_bucket, 0) + 1
        accepted_tids.add(sample["tid"])
        accepted_tid_counts[sample["tid"]] += 1
    accepted = sum(accepted_bucket_counts.values())
    unattributed = accepted_bucket_counts.get("UNATTRIBUTED", 0)
    unattributed_percent = 100.0 * unattributed / accepted if accepted else 100.0
    sampled_cpu_ns = accepted * PERIOD
    return {
        "filter": {
            "elf_build_id": ELF_BUILD_ID,
            "worker_tid_graph": "one libtest parent with exactly twenty direct worker children",
            "normalized_ip": "sealed traversal range",
            "timestamp_filter": False,
            "warmup_subtraction": False,
        },
        "mapping": mapping,
        "load_bias": load_bias,
        "load_bias_hex": f"0x{load_bias:x}",
        "normalization_unique": True,
        "d2_samples": d2_samples,
        "d2_samples_by_cpu": {str(key): value for key, value in sorted(cpu_counts.items())},
        "dso_mismatches": dso_mismatches[:32],
        "tid_graph": tid_graph,
        "excluded_non_worker_traversal_samples": excluded_non_worker_traversal_samples,
        "scientific_outside_traversal_samples": scientific_outside_traversal,
        "outside_mapped_text_samples": outside_mapped_text,
        "accepted_traversal_samples": accepted,
        "accepted_tids": sorted(accepted_tids),
        "accepted_samples_by_tid": {
            str(key): accepted_tid_counts[key] for key in sorted(accepted_tid_counts)
        },
        "accepted_bucket_counts": dict(sorted(accepted_bucket_counts.items())),
        "accepted_sub_bucket_counts": dict(sorted(accepted_sub_bucket_counts.items())),
        "unattributed_samples": unattributed,
        "unattributed_percent": unattributed_percent,
        "sampled_traversal_cpu_ns": sampled_cpu_ns,
        "sampled_traversal_cpu_per_edge_ns": sampled_cpu_ns / T4_EDGES,
        "t4_sampled_bursts": SAMPLED_ROUNDS,
        "t4_denominator_edges": T4_EDGES,
    }


def run_t4(route: str, stage: pathlib.Path, environment: dict[str, str]) -> dict[str, Any]:
    subject = stage / "subject"
    data_path = stage / "perf.data"
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
        record_command, stage / "record.stdout", stage / "record.stderr", timeout=1800
    )
    after_record = host_snapshot()
    commands = reader_commands(data_path)
    write_new_json(stage / "reader-commands.json", commands)
    reader_status = {}
    for name in READER_KINDS:
        if data_path.is_file() and data_path.stat().st_size > 0:
            reader_status[name] = run_command_to_files(
                commands[name], stage / f"{name}.stdout", stage / f"{name}.stderr", timeout=1800
            )
        else:
            write_new_bytes(stage / f"{name}.stdout", b"")
            write_new_bytes(stage / f"{name}.stderr", b"perf.data missing or empty\n")
            reader_status[name] = {"returncode": None, "timed_out": False, "not_run": True}
    after = host_snapshot()
    violations = {name: [] for name, _ in T4_DISPATCH}
    record_status_invalid = record_status["timed_out"] or record_status["returncode"] != 0
    if not data_path.is_file() or data_path.stat().st_size == 0:
        violations["capability"].append("perf.data missing or empty")
    for name, status_value in reader_status.items():
        if status_value.get("timed_out") or status_value.get("returncode") != 0:
            violations["capability"].append(f"{name} reader status: {status_value}")

    event_validation: dict[str, Any] = {}
    samples: list[dict[str, Any]] = []
    raw: dict[str, Any] = {
        "mappings": [],
        "forks": [],
        "comms": [],
        "exits": [],
        "lost_records": 0,
        "throttle_records": 0,
        "unthrottle_records": 0,
        "raw_sample_records": 0,
    }
    attribution: dict[str, Any] = {}
    tid_graph: dict[str, Any] = {}
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
        expected = {
            "type": 1,
            "config": 1,
            "sample_period": PERIOD,
            "exclude_kernel": 1,
            "inherit": 1,
            "freq": 0,
            "precise_ip": 0,
        }
        if any(event_validation[key] != value for key, value in expected.items()):
            violations["capability"].append(f"task-clock event identity mismatch: {event_validation}")
    except Exception as error:
        violations["capability"].append(f"event validation failed: {type(error).__name__}: {error}")
    try:
        samples = parse_samples(stage / "samples.stdout")
        if not samples:
            violations["capability"].append("perf script emitted no samples")
        if samples and ({item["event"] for item in samples} != {EVENT} or {item["period"] for item in samples} != {PERIOD}):
            violations["sample_coverage"].append("sample event or fixed period drift")
    except Exception as error:
        violations["provenance"].append(f"sample parser failed: {type(error).__name__}: {error}")
    try:
        raw = scan_raw_records(stage / "raw-records.stdout")
        if samples and raw["raw_sample_records"] != len(samples):
            violations["provenance"].append("raw and rendered sample count mismatch")
    except Exception as error:
        violations["provenance"].append(f"raw parser failed: {type(error).__name__}: {error}")
    try:
        buildids = (stage / "buildids.stdout").read_text(errors="replace")
        if re.search(rf"(?m)^{ELF_BUILD_ID}\s+{re.escape(str(ELF))}$", buildids) is None:
            violations["bucket_map"].append("exact D2 Build ID/path absent")
    except Exception as error:
        violations["bucket_map"].append(f"Build ID validation failed: {type(error).__name__}: {error}")
    map_value: dict[str, Any] = {}
    map_check: dict[str, Any] = {}
    executable: list[dict[str, Any]] = []
    try:
        map_value = json.loads(MAP.read_text())
        map_check = verify_map_bytes(map_value)
        if map_check["machine_byte_mismatches"]:
            violations["bucket_map"].append("machine byte mismatch")
        segment = map_value["executable_pt_load"]
        page_size = os.sysconf("SC_PAGE_SIZE")
        expected_offset = int(segment["offset"]) // page_size * page_size
        executable = [
            value
            for value in raw["mappings"]
            if "x" in value["permissions"] and value["offset"] == expected_offset
        ]
        need(len(executable) == 1, f"executable D2 mapping count: {len(executable)}")
    except Exception as error:
        violations["bucket_map"].append(f"IP normalization or map identity failed: {type(error).__name__}: {error}")
    if not violations["capability"] and not violations["provenance"]:
        try:
            need(bool(map_value), "map unavailable before TID reconstruction")
            need(len(executable) == 1, "executable mapping unavailable before TID reconstruction")
            tid_graph = reconstruct_worker_tids(raw, executable[0], samples)
        except Exception as error:
            violations["provenance"].append(f"worker TID graph failed: {type(error).__name__}: {error}")
    if tid_graph:
        try:
            attribution = attribute_t4(samples, raw, map_value, tid_graph)
            attribution["map_check"] = map_check
            if attribution["dso_mismatches"]:
                violations["bucket_map"].append("sample DSO/mapping mismatch")
            if attribution["accepted_tids"] != tid_graph["worker_tids"]:
                violations["sample_coverage"].append("one or more worker TIDs have no accepted traversal sample")
        except Exception as error:
            violations["bucket_map"].append(f"map join failed: {type(error).__name__}: {error}")
    try:
        subject_validation = validate_subject(subject, route)
        if subject_validation["violations"]:
            violations["provenance"].extend(
                f"subject {name} mismatch" for name in subject_validation["violations"]
            )
    except Exception as error:
        violations["provenance"].append(f"subject evidence unavailable: {type(error).__name__}: {error}")
    if record_status_invalid:
        if not data_path.is_file() or any(
            status_value.get("not_run")
            or status_value.get("timed_out")
            or status_value.get("returncode") != 0
            for status_value in reader_status.values()
        ):
            violations["capability"].append(f"perf record status: {record_status}")
        else:
            violations["provenance"].append(
                f"nonzero or timed-out record status with retained perf evidence: {record_status}"
            )

    drift = thermal_drift(before, after)
    if drift:
        violations["thermal"].append(f"thermal throttle drift: {drift}")
    if any(
        snapshot["perf_event_max_sample_rate"] != HOST_SAMPLE_RATE
        for snapshot in (before, after_record, after)
    ):
        violations["provenance"].append("perf_event_max_sample_rate drift")
    if raw["lost_records"] or raw["throttle_records"] or raw["unthrottle_records"]:
        violations["sample_coverage"].append(
            f"lost/throttle/unthrottle={raw['lost_records']}/{raw['throttle_records']}/{raw['unthrottle_records']}"
        )
    if attribution:
        paired_route = "U4-FIXED" if route == "T4-FIXED" else "U4-REVERSED"
        u4_receipt_path = result_path(paired_route) / "D5_ROUTE_RECEIPT.json"
        u4_receipt = json.loads(u4_receipt_path.read_text())
        need(u4_receipt.get("verdict") == ROUTES[paired_route]["pass_state"], "paired U4 receipt is not PASS")
        u4_cpu_per_edge = float(
            u4_receipt["observation"]["subject"]["traversal_thread_cpu_per_edge_ns"]
        )
        t4_cpu_per_edge = float(attribution["sampled_traversal_cpu_per_edge_ns"])
        delta = abs(t4_cpu_per_edge - u4_cpu_per_edge) / u4_cpu_per_edge * 100.0
        attribution["paired_u4_route"] = paired_route
        attribution["paired_u4_receipt_sha256"] = sha256_file(u4_receipt_path)
        attribution["paired_u4_cpu_per_edge_ns"] = u4_cpu_per_edge
        attribution["sampled_vs_paired_u4_delta_percent"] = delta
        if delta > 5.0:
            violations["perturbation"].append(f"sampled-vs-U4 delta {delta:.12f}% exceeds 5%")
        if attribution["accepted_traversal_samples"] < 50_000:
            violations["sample_coverage"].append(
                f"accepted traversal samples {attribution['accepted_traversal_samples']} below 50000"
            )
        if attribution["unattributed_percent"] > 5.0:
            violations["sample_coverage"].append(
                f"UNATTRIBUTED {attribution['unattributed_percent']:.12f}% exceeds 5%"
            )
    elif not violations["capability"] and not violations["provenance"]:
        violations["sample_coverage"].append("attribution unavailable")
    return {
        "schema": "lay.v10.e1-traversal-d5-t4-observation.v1",
        "route": route,
        "complete": not violations["provenance"],
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
        "cargo_invocations": 0,
        "perf_record_invocations": 1,
        "perf_reader_invocations": sum(
            1 for status_value in reader_status.values() if not status_value.get("not_run")
        ),
        "perf_stat_invocations": 0,
        "pmu_events_opened": 1 if event_validation else 0,
        "subject_executions": 1,
        "warmup_time_filter_or_subtraction": False,
    }


def dispatch_observation(observation: Mapping[str, Any], route: str) -> dict[str, Any]:
    priority = U4_DISPATCH if route.startswith("U4-") else T4_DISPATCH
    violations = observation.get("violations")
    expected_causes = {cause for cause, _ in priority}
    if (
        observation.get("route") != route
        or not isinstance(observation.get("complete"), bool)
        or not isinstance(violations, dict)
        or set(violations) != expected_causes
    ):
        return {
            "selected_cause": "provenance",
            "selected_rank": 0,
            "verdict": "BLOCKED_PROVENANCE",
            "reason": "dispatch observation schema mismatch",
            "all_violations": violations,
        }
    if observation.get("complete") is False and not violations.get("provenance"):
        return {
            "selected_cause": "provenance",
            "selected_rank": 0,
            "verdict": "BLOCKED_PROVENANCE",
            "reason": "observation declared incomplete without a provenance cause",
            "all_violations": violations,
        }
    selected = None
    for rank, (cause, terminal) in enumerate(priority):
        reasons = violations.get(cause)
        if (
            not isinstance(reasons, list)
            or any(not isinstance(reason, str) or not reason for reason in reasons)
        ):
            return {
                "selected_cause": "provenance",
                "selected_rank": 0,
                "verdict": "BLOCKED_PROVENANCE",
                "reason": f"dispatch schema invalid: {cause}",
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
        "reason": "all frozen D5 route predicates passed",
        "all_violations": violations,
    }


def consume_marker(route: str) -> dict[str, Any]:
    spec = ROUTES[route]
    available = STATE / f"markers/{spec['marker']}.available"
    consumed = STATE / f"markers/{spec['marker']}.consumed-before-exec"
    before = row(available)
    need(
        before["sha256"] == spec["marker_sha256"]
        and before["size_bytes"] == spec["marker_size"]
        and before["mode"] == "0400",
        f"marker identity drift: {route}",
    )
    need(not consumed.exists(), f"marker already consumed: {route}")
    os.rename(available, consumed)
    fsync_directory(available.parent)
    after = row(consumed)
    need(after["sha256"] == before["sha256"], "marker rename drift")
    return {"before": before, "after": after, "consumed_before_effect": True}


def result_path(route: str) -> pathlib.Path:
    return PARENT / f"{route.lower()}-v1"


def failure_path(route: str) -> pathlib.Path:
    return PARENT / f"{route.lower()}-failure-v1"


def state_path(route: str) -> pathlib.Path:
    return STATE / f"{route.replace('-', '_')}_STATE.json"


def publish_route_state(route: str, verdict: str, receipt_sha256: str | None) -> None:
    write_new_json(
        state_path(route),
        {
            "schema": "lay.v10.e1-traversal-d5-route-state.v1",
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


def consumed_marker_evidence(route: str) -> dict[str, Any]:
    spec = ROUTES[route]
    available = STATE / f"markers/{spec['marker']}.available"
    consumed = STATE / f"markers/{spec['marker']}.consumed-before-exec"
    need(not available.exists() and consumed.is_file(), f"consumed marker projection missing: {route}")
    after = row(consumed)
    need(
        after["sha256"] == spec["marker_sha256"]
        and after["size_bytes"] == spec["marker_size"]
        and after["mode"] == "0400",
        f"consumed marker identity drift: {route}",
    )
    body = json.loads(consumed.read_text())
    need(body == marker_body(route), f"consumed marker body drift: {route}")
    before = {**after, "path": str(available)}
    return {
        "before": before,
        "after": after,
        "consumed_before_effect": True,
        "reconstructed_after_controller_failure": True,
    }


def controller_failure_observation(route: str, error: BaseException) -> dict[str, Any]:
    priority = U4_DISPATCH if route.startswith("U4-") else T4_DISPATCH
    violations = {cause: [] for cause, _ in priority}
    violations["provenance"].append(
        f"controller failure after marker consumption: {type(error).__name__}: {error}"
    )
    return {
        "schema": "lay.v10.e1-traversal-d5-controller-failure-observation.v1",
        "route": route,
        "complete": False,
        "controller_failure": {
            "type": type(error).__name__,
            "message": str(error),
        },
        "violations": violations,
        "cargo_invocations": 0,
        "perf_record_invocations": None,
        "perf_reader_invocations": None,
        "perf_stat_invocations": 0,
        "pmu_events_opened": None,
        "subject_executions": None,
        "execution_ledger_complete": False,
    }


def recover_consumed_route_failure(
    stage: pathlib.Path,
    result: pathlib.Path,
    payload: Mapping[str, Any],
    route: str,
    error: BaseException,
) -> dict[str, Any]:
    need(stage.is_dir() and not result.exists(), "route failure recovery stage drift")
    for path in [stage, *stage.rglob("*")]:
        path.chmod(0o700 if path.is_dir() else 0o600)
    checksum = stage / "SHA256SUMS"
    if checksum.exists():
        checksum.unlink()
    retained = {}
    for name, retained_name in (
        ("OBSERVATION.json", "SCIENTIFIC_OBSERVATION.partial.json"),
        ("D5_ROUTE_RECEIPT.json", "SCIENTIFIC_ROUTE_RECEIPT.partial.json"),
    ):
        source = stage / name
        if source.exists():
            target = stage / retained_name
            need(not target.exists(), f"route recovery retained-file collision: {target}")
            os.rename(source, target)
            retained[name] = row(target)
    marker_path = stage / "MARKER_CONSUMPTION.json"
    authoritative_marker = consumed_marker_evidence(route)
    marker = authoritative_marker
    if marker_path.is_file():
        staged_marker = json.loads(marker_path.read_text())
        need(
            staged_marker.get("consumed_before_effect") is True
            and staged_marker.get("after") == authoritative_marker["after"]
            and staged_marker.get("before", {}).get("sha256")
            == authoritative_marker["before"]["sha256"],
            f"staged marker consumption drift: {route}",
        )
        marker = staged_marker
    observation = controller_failure_observation(route, error)
    dispatch = dispatch_observation(observation, route)
    need(dispatch["verdict"] == "BLOCKED_PROVENANCE", "controller failure dispatch drift")
    write_new_json(stage / "OBSERVATION.json", observation)
    write_new_json(
        stage / "CONTROLLER_FAILURE.json",
        {
            "schema": "lay.v10.e1-traversal-d5-controller-failure.v1",
            "task_id": TASK_ID,
            "transaction_id": TRANSACTION_ID,
            "route": route,
            "verdict": "BLOCKED_PROVENANCE",
            "error_type": type(error).__name__,
            "error": str(error),
            "marker_consumed": True,
            "retained_partial_files": retained,
            "retry_permitted": False,
        },
    )
    receipt = {
        "schema": "lay.v10.e1-traversal-d5-route-receipt.v1",
        "task_id": TASK_ID,
        "transaction_id": TRANSACTION_ID,
        "route": route,
        "verdict": "BLOCKED_PROVENANCE",
        "dispatch": dispatch,
        "marker": marker,
        "observation": observation,
        "elf": row(ELF),
        "map": row(MAP),
        "paper_sha256": PAPER_SHA256,
        "preflight_sha256": PREFLIGHT_SHA256,
        "preflight_receipt_sha256": PREFLIGHT_RECEIPT_SHA256,
        "bootstrap_audit_sha256": payload["bootstrap_audit_sha256"],
        "marker_audit_sha256": payload["marker_audit_sha256"],
        "d2_terminal_sha256": D2_TERMINAL_SHA256,
        "d3_terminal_sha256": D3_TERMINAL_SHA256,
        "d4_terminal_sha256": D4_TERMINAL_SHA256,
        "cargo_invocations": 0,
        "perf_record": None,
        "perf_readers": None,
        "perf_stat": 0,
        "pmu_events_opened": None,
        "subject_executions": None,
        "execution_ledger_complete": False,
        "d2_marker_mutations": 0,
        "runtime_authority_changed": False,
        "retry_permitted": False,
        "next_action_admitted": "none; terminal D5 blocked verdict",
        "controller_failure_recovered": True,
    }
    write_new_json(stage / "D5_ROUTE_RECEIPT.json", receipt)
    write_sha256sums(stage)
    seal_evidence_tree(stage)
    os.rename(stage, result)
    fsync_directory(PARENT)
    published = result / "D5_ROUTE_RECEIPT.json"
    published_sha256 = sha256_file(published)
    publish_route_state(route, "BLOCKED_PROVENANCE", published_sha256)
    return {
        **receipt,
        "published_receipt_sha256": published_sha256,
        "remote_result": str(result),
    }


def route_once(payload: Mapping[str, Any], route: str) -> dict[str, Any]:
    need(route in ROUTES, f"unknown route: {route}")
    host = verify_host()
    d2 = verify_d2_projection()
    d4 = verify_d4_projection()
    admission = verify_bootstrap(payload, route, post=False)
    need(not active_d5_subjects(), "D5 subject or perf descendant active before route")
    result = result_path(route)
    failure = failure_path(route)
    need(not result.exists() and not failure.exists(), f"route result already exists: {route}")
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
        write_new_bytes(inputs / "paper.md", decode_input(payload, "paper_b64", PAPER_SHA256))
        write_new_bytes(inputs / "preflight-v3.json", decode_input(payload, "preflight_b64", PREFLIGHT_SHA256))
        write_new_bytes(
            inputs / "preflight-v3-receipt.json",
            decode_input(payload, "preflight_receipt_b64", PREFLIGHT_RECEIPT_SHA256),
        )
        write_new_bytes(inputs / "d2-terminal-receipt.json", decode_input(payload, "d2_terminal_b64", D2_TERMINAL_SHA256))
        write_new_bytes(inputs / "d3-terminal-receipt.json", decode_input(payload, "d3_terminal_b64", D3_TERMINAL_SHA256))
        write_new_bytes(inputs / "d4-terminal-receipt.json", decode_input(payload, "d4_terminal_b64", D4_TERMINAL_SHA256))
        write_new_bytes(
            inputs / "local-controller.py",
            decode_input(payload, "local_controller_b64", payload["local_controller_sha256"]),
        )
        write_new_bytes(
            inputs / "remote-controller.py",
            decode_input(payload, "remote_controller_b64", payload["remote_controller_sha256"]),
        )
        audit_bytes = base64.b64decode(payload["bootstrap_audit_b64"], validate=True)
        need(sha256_bytes(audit_bytes) == payload["bootstrap_audit_sha256"], "audit SHA drift")
        write_new_bytes(inputs / "bootstrap-audit.json", audit_bytes)
        marker_audit_bytes = base64.b64decode(payload["marker_audit_b64"], validate=True)
        need(sha256_bytes(marker_audit_bytes) == payload["marker_audit_sha256"], "marker audit SHA drift")
        write_new_bytes(inputs / "marker-audit.json", marker_audit_bytes)
        environment = controlled_environment(subject, route)
        preobservation = {
            "schema": "lay.v10.e1-traversal-d5-preobservation.v1",
            "task_id": TASK_ID,
            "transaction_id": TRANSACTION_ID,
            "route": route,
            "subject_command": subject_command(route),
            "environment": environment,
            "admission": admission,
            "host": host,
            "d2_projection": d2,
            "d4_projection": d4,
            "marker_consumed": False,
            "retry_permitted": False,
        }
        if route.startswith("T4-"):
            data_path = stage / "perf.data"
            preobservation["perf_record_command"] = [
                *PERF_RECORD_PREFIX,
                "--output",
                str(data_path),
                "--",
                *child_as_e(environment, subject_command(route)),
            ]
            preobservation["reader_commands"] = reader_commands(data_path)
        write_new_json(stage / "PREOBSERVATION.json", preobservation)
        fsync_directory(stage)
        marker = consume_marker(route)
        marker_consumed = True
        write_new_json(stage / "MARKER_CONSUMPTION.json", marker)
        verify_d5_markers(route, post=True)
        observation = run_u4(route, stage, environment) if route.startswith("U4-") else run_t4(route, stage, environment)
        dispatch = dispatch_observation(observation, route)
        write_new_json(stage / "OBSERVATION.json", observation)
        receipt = {
            "schema": "lay.v10.e1-traversal-d5-route-receipt.v1",
            "task_id": TASK_ID,
            "transaction_id": TRANSACTION_ID,
            "route": route,
            "verdict": dispatch["verdict"],
            "dispatch": dispatch,
            "marker": marker,
            "observation": observation,
            "elf": row(ELF),
            "map": row(MAP),
            "paper_sha256": PAPER_SHA256,
            "preflight_sha256": PREFLIGHT_SHA256,
            "preflight_receipt_sha256": PREFLIGHT_RECEIPT_SHA256,
            "bootstrap_audit_sha256": payload["bootstrap_audit_sha256"],
            "marker_audit_sha256": payload["marker_audit_sha256"],
            "d2_terminal_sha256": D2_TERMINAL_SHA256,
            "d3_terminal_sha256": D3_TERMINAL_SHA256,
            "d4_terminal_sha256": D4_TERMINAL_SHA256,
            "cargo_invocations": 0,
            "perf_record": observation["perf_record_invocations"],
            "perf_readers": observation["perf_reader_invocations"],
            "perf_stat": 0,
            "pmu_events_opened": observation["pmu_events_opened"],
            "subject_executions": 1,
            "d2_marker_mutations": 0,
            "runtime_authority_changed": False,
            "retry_permitted": False,
            "next_action_admitted": (
                f"{ROUTE_ORDER[ROUTE_ORDER.index(route) + 1]} only"
                if dispatch["verdict"] == ROUTES[route]["pass_state"]
                and ROUTE_ORDER.index(route) + 1 < len(ROUTE_ORDER)
                else (
                    "independent D5 terminal audit only; no optimization"
                    if route == ROUTE_ORDER[-1] and dispatch["verdict"] == ROUTES[route]["pass_state"]
                    else "none; terminal D5 blocked verdict"
                )
            ),
        }
        write_new_json(stage / "D5_ROUTE_RECEIPT.json", receipt)
        write_sha256sums(stage)
        seal_evidence_tree(stage)
        os.rename(stage, result)
        fsync_directory(PARENT)
        published = result / "D5_ROUTE_RECEIPT.json"
        published_sha256 = sha256_file(published)
        response = {
            **receipt,
            "published_receipt_sha256": published_sha256,
            "remote_result": str(result),
        }
        publish_route_state(route, dispatch["verdict"], published_sha256)
        return response
    except BaseException as error:
        spec = ROUTES[route]
        consumed_path = STATE / f"markers/{spec['marker']}.consumed-before-exec"
        available_path = STATE / f"markers/{spec['marker']}.available"
        marker_consumed = marker_consumed or (
            consumed_path.is_file() and not available_path.exists()
        )
        recovery_error = None
        if marker_consumed and stage.exists() and not result.exists():
            try:
                return recover_consumed_route_failure(stage, result, payload, route, error)
            except BaseException as failure_recovery_error:
                recovery_error = (
                    f"{type(failure_recovery_error).__name__}: {failure_recovery_error}"
                )
        if marker_consumed:
            with contextlib.suppress(BaseException):
                failure_stage = stage
                provisional_result = None
                if not failure_stage.exists():
                    failure_stage = pathlib.Path(
                        f"{failure}.stage-{os.getpid()}-{time.time_ns()}"
                    )
                    failure_stage.mkdir(mode=0o700)
                    if result.is_dir():
                        provisional_receipt = result / "D5_ROUTE_RECEIPT.json"
                        provisional_result = {
                            "path": str(result),
                            "manifest": row(result / "SHA256SUMS"),
                            "receipt": row(provisional_receipt),
                            "receipt_verdict": json.loads(provisional_receipt.read_text()).get(
                                "verdict"
                            ),
                        }
                for path in [failure_stage, *failure_stage.rglob("*")]:
                    path.chmod(0o700 if path.is_dir() else 0o600)
                checksum = failure_stage / "SHA256SUMS"
                if checksum.exists():
                    checksum.unlink()
                write_new_json(
                    failure_stage / "FAILURE.json",
                    {
                        "verdict": "BLOCKED_PROVENANCE",
                        "route": route,
                        "error": f"{type(error).__name__}: {error}",
                        "recovery_error": recovery_error,
                        "marker_consumed": True,
                        "provisional_result": provisional_result,
                        "retry_permitted": False,
                    },
                )
                write_sha256sums(failure_stage)
                seal_evidence_tree(failure_stage)
                os.rename(failure_stage, failure)
                fsync_directory(PARENT)
                if not state_path(route).exists():
                    publish_route_state(route, "BLOCKED_PROVENANCE", None)
        elif stage.exists():
            shutil.rmtree(stage)
        raise


def probe(payload: Mapping[str, Any], action: str, route: str | None) -> dict[str, Any]:
    host = verify_host()
    d2 = verify_d2_projection()
    d4 = verify_d4_projection()
    if action == "probe-absent":
        verify_payload(payload, require_bootstrap_audit=False)
        need(not PARENT.exists() and not STATE.exists(), "D5 namespace already exists")
        need(not active_d5_subjects(), "D5 subject active before namespace creation")
        return {
            "verdict": "D5_REMOTE_ABSENT_PROBE_PASS",
            "host": host,
            "d2_projection": d2,
            "d5_parent_exists": False,
            "d5_state_exists": False,
            "d4_projection": d4,
            "remote_writes": 0,
        }
    if action == "probe-bootstrap":
        ready = verify_bootstrap_base(payload, require_bootstrap_audit=False)
        need(not (STATE / "markers").exists(), "markers exist before bootstrap audit")
        need(not active_d5_subjects(), "D5 subject active before marker creation")
        return {
            "verdict": "D5_REMOTE_BOOTSTRAP_PROBE_PASS",
            "host": host,
            "d2_projection": d2,
            "d5": ready,
            "d4_projection": d4,
            "remote_writes": 0,
        }
    need(route in ROUTES, "route required for ready/after probe")
    post = action == "probe-after"
    ready = verify_bootstrap(payload, route, post=post)
    need(not active_d5_subjects(), "D5 subject or perf descendant active at route probe")
    result: dict[str, Any] = {
        "verdict": "D5_REMOTE_POST_PROBE_PASS" if post else "D5_REMOTE_READY_PROBE_PASS",
        "route": route,
        "host": host,
        "d2_projection": d2,
        "d5": ready,
        "d4_projection": d4,
        "remote_writes": 0,
    }
    if post:
        receipt_path = result_path(route) / "D5_ROUTE_RECEIPT.json"
        result["route_receipt"] = row(receipt_path)
        result["route_verdict"] = json.loads(receipt_path.read_text()).get("verdict")
    return result


def main() -> int:
    try:
        need(len(sys.argv) == 2, "expected one base64 payload")
        payload = json.loads(base64.b64decode(sys.argv[1], validate=True))
        action = payload.get("action")
        route = payload.get("route")
        if action == "probe-absent":
            value = probe(payload, action, None)
        elif action == "bootstrap":
            value = bootstrap(payload)
        elif action == "probe-bootstrap":
            value = probe(payload, action, None)
        elif action == "create-markers":
            with LOCK.open("rb") as lock:
                fcntl.flock(lock.fileno(), fcntl.LOCK_EX)
                value = create_markers(payload)
        elif action in ("probe-ready", "probe-after"):
            need(isinstance(route, str), "route missing")
            value = probe(payload, action, route)
        elif action in ACTION_TO_ROUTE:
            expected = ACTION_TO_ROUTE[action]
            need(route == expected, "action/route mismatch")
            with LOCK.open("rb") as lock:
                fcntl.flock(lock.fileno(), fcntl.LOCK_EX)
                value = route_once(payload, route)
        else:
            raise D5Error(f"unsupported action: {action!r}")
        print(json.dumps(value, sort_keys=True, separators=(",", ":")))
        return 0
    except Exception as error:
        print(f"D5 REMOTE ERROR: {type(error).__name__}: {error}", file=sys.stderr)
        return 1


if __name__ == "__main__":
    raise SystemExit(main())
