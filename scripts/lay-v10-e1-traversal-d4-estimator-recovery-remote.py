#!/usr/bin/env python3
"""Remote one-shot producer for D4 single-worker estimator recovery."""

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


TASK_ID = "slice8b-v10-e1-traversal-d4-estimator-recovery-v1-20260826"
TRANSACTION_ID = "2d3002b7cf615459a4250d7e44eb2094863dc422f908080b7afa59551ba4ee26"
HOSTNAME = "e-MEGA-MINI-M1-13th"
MACHINE_ID_SHA256 = "5ac0bb537745673ef80942c2df32d586f5753c85de1c8b2453aa4e7317b77441"

D2_TASK_ID = "slice8b-v10-e1-traversal-d2-primary-only-v2-20260825"
D2_TRANSACTION_ID = "35dbe5c5eaec7dbb5b5b112313a3285dbe2ceaea0a910719ad379255ca5569e7"
D2_PARENT = pathlib.Path("/home/e/.local/share/lay/provenance") / D2_TASK_ID
D2_STATE = pathlib.Path("/home/e/.local/state/lay") / D2_TASK_ID

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
PAPER_SHA256 = "50b5cde0514d356375464f8155905201c395112c88a8d5022c70913b5da4d7b3"
PREFLIGHT_SHA256 = "109d284d0031df29bd1747571690fc410fba72381fe73d4adf647e550ce86a78"
PREFLIGHT_RECEIPT_SHA256 = "21dc062bf9996bd793d9392e46fc747dc89b8c5878275d257e578e0dc501c282"
D2_TERMINAL_SHA256 = "75dc2703e279524a76c49a455b6081634d1c67274106da45f0d7d610af95e608"
D3_TERMINAL_SHA256 = "7f4c0fd9466cced361c21858ea2017910adc7b92db7c1edfe02f3be09d5c4299"
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

ROUTE_ORDER = ("U3-SINGLE", "T3-SINGLE")
ROUTES = {
    "U3-SINGLE": {
        "marker": "u3-single",
        "marker_sha256": "4484826fab0137fcc7e41e146891b110a666a865e7594eb008784ddd2c2154e9",
        "marker_size": 287,
        "pass_state": "U3_SINGLE_PASS",
    },
    "T3-SINGLE": {
        "marker": "t3-single",
        "marker_sha256": "ff975bfdb78ca675903a6ee123134594cd4ec88391c17be7f41098174d45ffd6",
        "marker_size": 287,
        "pass_state": "D4_SINGLE_ESTIMATOR_PASS",
    },
}

SUBJECT_TEST = "nanda_wave::l2_field::v13_typed_peak::tests::v10_d1_component_single"
QUERIES = 382
MEASURED_ROUNDS = 20
SAMPLED_ROUNDS = 21
EDGES_PER_ROUND = 25_145_756
U3_EDGES = EDGES_PER_ROUND * MEASURED_ROUNDS
T3_EDGES = EDGES_PER_ROUND * SAMPLED_ROUNDS
REQUESTS = QUERIES * MEASURED_ROUNDS
STAGING_CPU = 6
SCIENTIFIC_CPU = 0
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
U3_DISPATCH = (
    ("provenance", "BLOCKED_PROVENANCE"),
    ("thermal", "BLOCKED_THERMAL"),
    ("semantic", "BLOCKED_SEMANTIC"),
)
T3_DISPATCH = (
    ("provenance", "BLOCKED_PROVENANCE"),
    ("thermal", "BLOCKED_THERMAL"),
    ("capability", "BLOCKED_CAPABILITY"),
    ("bucket_map", "BLOCKED_BUCKET_MAP"),
    ("perturbation", "BLOCKED_PERTURBATION"),
    ("sample_coverage", "BLOCKED_SAMPLE_COVERAGE"),
)


class D4Error(RuntimeError):
    pass


def need(condition: bool, message: str) -> None:
    if not condition:
        raise D4Error(message)


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
        "schema": "lay.v10.e1-traversal-d4-marker.v1",
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


def verify_host() -> dict[str, Any]:
    need(os.geteuid() == 0, "D4 remote controller requires root")
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
    need(
        preflight.get("task_id")
        == "LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_D4_ESTIMATOR_RECOVERY_IMPLEMENTATION_V2_2026-08-26",
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
    local_source = decode_input(payload, "local_controller_b64", payload["local_controller_sha256"])
    remote_source = decode_input(payload, "remote_controller_b64", payload["remote_controller_sha256"])
    need(paper.startswith(b"# D4 - D2 task-clock estimator recovery V1\n"), "paper identity drift")
    result: dict[str, Any] = {
        "local_controller_sha256": sha256_bytes(local_source),
        "remote_controller_sha256": sha256_bytes(remote_source),
        "preflight_sha256": PREFLIGHT_SHA256,
        "preflight_receipt_sha256": PREFLIGHT_RECEIPT_SHA256,
        "paper_sha256": PAPER_SHA256,
        "d2_terminal_sha256": D2_TERMINAL_SHA256,
        "d3_terminal_sha256": D3_TERMINAL_SHA256,
    }
    if require_bootstrap_audit:
        audit_bytes = base64.b64decode(payload.get("bootstrap_audit_b64", ""), validate=True)
        need(sha256_bytes(audit_bytes) == payload.get("bootstrap_audit_sha256"), "bootstrap audit payload SHA drift")
        audit = json.loads(audit_bytes)
        need(audit.get("verdict") == "D4_UID_ACCESS_AUDIT_PASS_MARKER_CREATION", "bootstrap audit verdict drift")
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
        need(audit.get("verdict") == "D4_MARKER_AUDIT_PASS_U3_ADMITTED", "marker audit verdict drift")
        need(audit.get("task_id") == TASK_ID and audit.get("transaction_id") == TRANSACTION_ID, "marker audit namespace drift")
        need(audit.get("markers_created") == 2 and audit.get("markers_consumed") == 0, "marker audit ledger drift")
        need(audit.get("remote_writes") == 0, "marker audit remote write drift")
        result["marker_audit_sha256"] = sha256_bytes(audit_bytes)
        result["marker_audit"] = audit
    return result


def d4_marker_projection() -> list[dict[str, Any]]:
    values = []
    for path in sorted((STATE / "markers").iterdir()):
        values.append({**row(path), "name": path.name, "value": json.loads(path.read_text())})
    return values


def expected_d4_markers(route: str | None = None, *, post: bool = False) -> dict[str, tuple[str, int]]:
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


def verify_d4_markers(route: str | None = None, *, post: bool = False) -> list[dict[str, Any]]:
    expected = expected_d4_markers(route, post=post)
    values = d4_marker_projection()
    observed = {item["name"]: item for item in values}
    need(set(observed) == set(expected), f"D4 marker membership drift: {sorted(observed)}")
    for name, (digest, size) in expected.items():
        item = observed[name]
        need(
            item["sha256"] == digest and item["size_bytes"] == size and item["mode"] == "0400",
            f"D4 marker identity drift: {name}",
        )
        body = item["value"]
        need(body.get("task_id") == TASK_ID and body.get("transaction_id") == TRANSACTION_ID, f"D4 marker namespace drift: {name}")
        need(body.get("retry_permitted") is False, f"D4 marker retry drift: {name}")
    return values


def verify_bootstrap_base(
    payload: Mapping[str, Any], *, require_bootstrap_audit: bool
) -> dict[str, Any]:
    payload_check = verify_payload(
        payload, require_bootstrap_audit=require_bootstrap_audit
    )
    need(PARENT.is_dir() and STATE.is_dir(), "D4 namespace missing")
    need(mode_string(PARENT) == "0755" and mode_string(STATE) == "0755", "D4 parent mode drift")
    bootstrap = PARENT / "bootstrap-v1"
    need(bootstrap.is_dir(), "D4 bootstrap evidence missing")
    verify_sha256sums(bootstrap)
    receipt_path = bootstrap / "D4_BOOTSTRAP_RECEIPT.json"
    receipt = json.loads(receipt_path.read_text())
    need(receipt.get("verdict") == "D4_UID_PROOF_CREATED_UNAUDITED", "bootstrap receipt verdict drift")
    need(receipt.get("markers_created") == 0, "bootstrap created scientific markers")
    state_path = STATE / "STATE.json"
    state_value = json.loads(state_path.read_text())
    need(state_value.get("state") == "D4_UID_PROOF_CREATED_UNAUDITED", "D4 bootstrap state drift")
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
    need(creation.is_dir(), "D4 marker creation evidence missing")
    verify_sha256sums(creation)
    creation_receipt_path = creation / "D4_MARKER_CREATION_RECEIPT.json"
    creation_receipt = json.loads(creation_receipt_path.read_text())
    need(creation_receipt.get("verdict") == "D4_MARKERS_CREATED_UNAUDITED", "marker creation verdict drift")
    need(
        creation_receipt.get("bootstrap_audit_sha256")
        == payload_check["bootstrap_audit_sha256"],
        "marker creation admission drift",
    )
    marker_state_path = STATE / "MARKER_STATE.json"
    marker_state = json.loads(marker_state_path.read_text())
    need(marker_state.get("state") == "D4_MARKERS_CREATED_UNAUDITED", "marker state drift")
    markers = verify_d4_markers(route, post=post)
    if route == "T3-SINGLE":
        u3_state_path = STATE / "U3_SINGLE_STATE.json"
        need(u3_state_path.is_file(), "U3 state missing before T3")
        u3_state = json.loads(u3_state_path.read_text())
        need(u3_state.get("state") == "U3_SINGLE_PASS", "T3 not admitted by U3 PASS")
        u3_receipt_path = PARENT / "u3-single-v1/D4_ROUTE_RECEIPT.json"
        need(u3_receipt_path.is_file(), "U3 receipt missing before T3")
        need(u3_state.get("receipt_sha256") == sha256_file(u3_receipt_path), "U3 state receipt drift")
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
fixed=b"D4 UID E CAPABILITY PROBE V1\n"
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
receipt={"schema":"lay.v10.e1-traversal-d4-uid-probe.v1","uid":os.getuid(),"gid":os.getgid(),"fixed_sha256":hashlib.sha256(fixed).hexdigest(),"fixed_size":len(fixed),"elf_sha256":elf_sha,"map_sha256":map_sha,"ancestors":ancestors,"operations":["traverse","stat","read","create","write","fsync-file","rename","fsync-directory","reopen","read-exact","unlink","fsync-directory"],"verdict":"D4_UID_E_CAPABILITY_PROOF_PASS"}
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
    need(value.get("verdict") == "D4_UID_E_CAPABILITY_PROOF_PASS", "UID probe verdict drift")
    need(value.get("uid") == pwd.getpwnam("e").pw_uid, "UID probe identity drift")
    need(value.get("elf_sha256") == ELF_SHA256 and value.get("map_sha256") == MAP_SHA256, "UID readable input drift")
    need((subject / "UID_PROBE_BYTES.bin").read_bytes() == b"D4 UID E CAPABILITY PROBE V1\n", "UID probe bytes drift")
    need(json.loads((subject / "UID_PROBE.json").read_text()) == value, "UID probe receipt drift")
    return {"value": value, "stdout_sha256": sha256_bytes(result.stdout), "stderr_sha256": sha256_bytes(result.stderr)}


def bootstrap(payload: Mapping[str, Any]) -> dict[str, Any]:
    host = verify_host()
    d2 = verify_d2_projection()
    payload_check = verify_payload(payload, require_bootstrap_audit=False)
    need(not PARENT.exists() and not STATE.exists(), "D4 namespace already exists")
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
        write_new_bytes(inputs / "preflight-v2.json", decode_input(payload, "preflight_b64", PREFLIGHT_SHA256))
        write_new_bytes(
            inputs / "preflight-v2-receipt.json",
            decode_input(payload, "preflight_receipt_b64", PREFLIGHT_RECEIPT_SHA256),
        )
        write_new_bytes(inputs / "d2-terminal-receipt.json", decode_input(payload, "d2_terminal_b64", D2_TERMINAL_SHA256))
        write_new_bytes(inputs / "d3-terminal-receipt.json", decode_input(payload, "d3_terminal_b64", D3_TERMINAL_SHA256))
        write_new_bytes(
            inputs / "local-controller.py",
            decode_input(payload, "local_controller_b64", payload["local_controller_sha256"]),
        )
        write_new_bytes(
            inputs / "remote-controller.py",
            decode_input(payload, "remote_controller_b64", payload["remote_controller_sha256"]),
        )
        uid_probe = run_uid_probe(subject)
        write_new_bytes(state_stage / "route.lock", b"d4-estimator-recovery-v1\n", 0o400)
        write_new_json(
            state_stage / "STATE.json",
            {
                "schema": "lay.v10.e1-traversal-d4-state.v1",
                "task_id": TASK_ID,
                "transaction_id": TRANSACTION_ID,
                "state": "D4_UID_PROOF_CREATED_UNAUDITED",
                "markers_expected": 2,
                "markers_created": 0,
                "markers_consumed": 0,
                "retry_permitted": False,
            },
            0o400,
        )
        receipt = {
            "schema": "lay.v10.e1-traversal-d4-bootstrap.v1",
            "task_id": TASK_ID,
            "transaction_id": TRANSACTION_ID,
            "verdict": "D4_UID_PROOF_CREATED_UNAUDITED",
            "payload": payload_check,
            "host": host,
            "d2_projection": d2,
            "uid_probe": uid_probe,
            "parent_mode": "0755",
            "subject_uid": account.pw_uid,
            "markers_expected": 2,
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
            "next_action_admitted": "independent D4 bootstrap audit only",
        }
        write_new_json(evidence / "D4_BOOTSTRAP_RECEIPT.json", receipt)
        write_sha256sums(evidence)
        seal_evidence_tree(evidence)
        fsync_directory(parent_stage)
        fsync_directory(state_stage)
        os.rename(parent_stage, PARENT)
        fsync_directory(PARENT.parent)
        os.rename(state_stage, STATE)
        fsync_directory(STATE.parent)
        published = PARENT / "bootstrap-v1/D4_BOOTSTRAP_RECEIPT.json"
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
    base = verify_bootstrap_base(payload, require_bootstrap_audit=True)
    need(not (STATE / "markers").exists(), "D4 marker directory already exists")
    need(not (STATE / "MARKER_STATE.json").exists(), "D4 marker state already exists")
    result = PARENT / "marker-creation-v1"
    need(not result.exists(), "D4 marker creation evidence already exists")
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
            "schema": "lay.v10.e1-traversal-d4-marker-creation.v1",
            "task_id": TASK_ID,
            "transaction_id": TRANSACTION_ID,
            "verdict": "D4_MARKERS_CREATED_UNAUDITED",
            "bootstrap_audit_sha256": base["payload"]["bootstrap_audit_sha256"],
            "markers": marker_rows,
            "markers_expected": 2,
            "markers_created": 2,
            "markers_consumed": 0,
            "host": host,
            "d2_projection": d2,
            "subject_executions": 0,
            "perf_record": 0,
            "perf_stat": 0,
            "pmu_events_opened": 0,
            "retry_permitted": False,
            "next_action_admitted": "independent D4 marker audit only",
        }
        write_new_json(result_stage / "D4_MARKER_CREATION_RECEIPT.json", receipt)
        write_sha256sums(result_stage)
        seal_evidence_tree(result_stage)
        os.rename(marker_stage, STATE / "markers")
        fsync_directory(STATE)
        write_new_json(
            STATE / "MARKER_STATE.json",
            {
                "schema": "lay.v10.e1-traversal-d4-marker-state.v1",
                "task_id": TASK_ID,
                "transaction_id": TRANSACTION_ID,
                "state": "D4_MARKERS_CREATED_UNAUDITED",
                "bootstrap_audit_sha256": base["payload"]["bootstrap_audit_sha256"],
                "markers_created": 2,
                "markers_consumed": 0,
                "retry_permitted": False,
            },
            0o400,
        )
        fsync_directory(STATE)
        os.rename(result_stage, result)
        fsync_directory(PARENT)
        published = result / "D4_MARKER_CREATION_RECEIPT.json"
        return {**receipt, "published_receipt_sha256": sha256_file(published)}
    except BaseException:
        if marker_stage.exists():
            shutil.rmtree(marker_stage)
        if result_stage.exists():
            shutil.rmtree(result_stage)
        raise


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
        "LAY_V10_D1_CPUS": "0",
    }


def subject_command() -> list[str]:
    return [
        str(TASKSET),
        "--cpu-list",
        str(STAGING_CPU),
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
        "mapping": receipt.get("mapping") == "SINGLE",
        "queries": receipt.get("queries") == QUERIES,
        "rounds": receipt.get("rounds") == MEASURED_ROUNDS,
        "workers": receipt.get("workers") == 1,
        "warmup_queries": receipt.get("warmup_queries") == QUERIES,
        "cpus": receipt.get("cpus") == [SCIENTIFIC_CPU],
        "thread_migrations": receipt.get("thread_migrations") == 0,
        "receipt_samples": receipt.get("samples", {}).get("samples") == REQUESTS,
        "receipt_errors": receipt.get("samples", {}).get("errors") == 0,
        "receipt_unresolved": receipt.get("samples", {}).get("unresolved") == 0,
        "phase_order": receipt.get("samples", {}).get("phase_order") == list(PHASES),
        "record_count": samples["records"] == REQUESTS,
        "ordinals": samples["query_ordinals"] == list(range(QUERIES)),
        "round_ids": samples["rounds"] == list(range(MEASURED_ROUNDS)),
        "worker_sentinel": samples["workers"] == [255],
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
        "traversal_thread_cpu_per_edge_ns": samples["traversal_thread_cpu_ns"] / U3_EDGES,
        "u3_measured_rounds": MEASURED_ROUNDS,
        "u3_denominator_edges": U3_EDGES,
        "staging_cpu": STAGING_CPU,
        "scientific_cpu": SCIENTIFIC_CPU,
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


def run_u3(stage: pathlib.Path, environment: dict[str, str]) -> dict[str, Any]:
    subject = stage / "subject"
    command = child_as_e(environment, subject_command())
    before = host_snapshot()
    status = run_command_to_files(command, stage / "subject.stdout", stage / "subject.stderr", timeout=1800)
    after = host_snapshot()
    violations = {name: [] for name, _ in U3_DISPATCH}
    validation: dict[str, Any] = {}
    if status["timed_out"] or status["returncode"] != 0:
        violations["semantic"].append(f"subject status: {status}")
    try:
        validation = validate_subject(subject, "U3-SINGLE")
        if validation["violations"]:
            violations["semantic"].extend(f"subject {name} mismatch" for name in validation["violations"])
    except Exception as error:
        violations["provenance"].append(f"subject evidence unavailable: {type(error).__name__}: {error}")
    drift = thermal_drift(before, after)
    if drift:
        violations["thermal"].append(f"thermal throttle drift: {drift}")
    if before["perf_event_max_sample_rate"] != HOST_SAMPLE_RATE or after["perf_event_max_sample_rate"] != HOST_SAMPLE_RATE:
        violations["provenance"].append("host sample-rate drift during U3")
    return {
        "schema": "lay.v10.e1-traversal-d4-u3-observation.v1",
        "route": "U3-SINGLE",
        "complete": not violations["provenance"],
        "command": command,
        "subject_command": subject_command(),
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
    return {
        "mappings": mappings,
        "lost_records": lost,
        "throttle_records": throttle,
        "unthrottle_records": unthrottle,
        "raw_sample_records": raw_samples,
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


def attribute_t3(
    samples: list[dict[str, Any]], raw: Mapping[str, Any], map_value: Mapping[str, Any]
) -> dict[str, Any]:
    segment = map_value["executable_pt_load"]
    page_size = os.sysconf("SC_PAGE_SIZE")
    expected_offset = int(segment["offset"]) // page_size * page_size
    executable = [
        value for value in raw["mappings"] if "x" in value["permissions"] and value["offset"] == expected_offset
    ]
    need(len(executable) == 1, f"executable D2 mapping count: {len(executable)}")
    mapping = executable[0]
    aligned_vaddr = int(segment["vaddr"]) // page_size * page_size
    load_bias = mapping["start"] - aligned_vaddr
    ranges = map_value["ranges"]
    starts = [int(value["start"]) for value in ranges]
    accepted_bucket_counts: dict[str, int] = {}
    accepted_sub_bucket_counts: dict[str, int] = {}
    d2_samples = 0
    cpu_counts: dict[int, int] = {}
    accepted_tids: set[int] = set()
    staging_traversal_samples = 0
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
        if sample["cpu"] != SCIENTIFIC_CPU:
            if bucket != "OUTSIDE_TRAVERSAL":
                staging_traversal_samples += 1
            continue
        if bucket == "OUTSIDE_TRAVERSAL":
            scientific_outside_traversal += 1
            continue
        need(bucket in TRAVERSAL_BUCKETS, f"unknown traversal bucket: {bucket}")
        sub_bucket = value["sub_bucket"]
        accepted_bucket_counts[bucket] = accepted_bucket_counts.get(bucket, 0) + 1
        accepted_sub_bucket_counts[sub_bucket] = accepted_sub_bucket_counts.get(sub_bucket, 0) + 1
        accepted_tids.add(sample["tid"])
    accepted = sum(accepted_bucket_counts.values())
    unattributed = accepted_bucket_counts.get("UNATTRIBUTED", 0)
    unattributed_percent = 100.0 * unattributed / accepted if accepted else 100.0
    sampled_cpu_ns = accepted * PERIOD
    return {
        "filter": {
            "elf_build_id": ELF_BUILD_ID,
            "sample_cpu": SCIENTIFIC_CPU,
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
        "staging_traversal_samples_excluded": staging_traversal_samples,
        "scientific_outside_traversal_samples": scientific_outside_traversal,
        "outside_mapped_text_samples": outside_mapped_text,
        "accepted_traversal_samples": accepted,
        "accepted_tids": sorted(accepted_tids),
        "accepted_bucket_counts": dict(sorted(accepted_bucket_counts.items())),
        "accepted_sub_bucket_counts": dict(sorted(accepted_sub_bucket_counts.items())),
        "unattributed_samples": unattributed,
        "unattributed_percent": unattributed_percent,
        "sampled_traversal_cpu_ns": sampled_cpu_ns,
        "sampled_traversal_cpu_per_edge_ns": sampled_cpu_ns / T3_EDGES,
        "t3_sampled_rounds": SAMPLED_ROUNDS,
        "t3_denominator_edges": T3_EDGES,
    }


def run_t3(stage: pathlib.Path, environment: dict[str, str]) -> dict[str, Any]:
    subject = stage / "subject"
    data_path = stage / "perf.data"
    record_command = [
        *PERF_RECORD_PREFIX,
        "--output",
        str(data_path),
        "--",
        *child_as_e(environment, subject_command()),
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
    violations = {name: [] for name, _ in T3_DISPATCH}
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
        violations["capability"].append(f"sample parser failed: {type(error).__name__}: {error}")
    try:
        raw = scan_raw_records(stage / "raw-records.stdout")
        if samples and raw["raw_sample_records"] != len(samples):
            violations["provenance"].append("raw and rendered sample count mismatch")
    except Exception as error:
        violations["capability"].append(f"raw parser failed: {type(error).__name__}: {error}")
    try:
        buildids = (stage / "buildids.stdout").read_text(errors="replace")
        if re.search(rf"(?m)^{ELF_BUILD_ID}\s+{re.escape(str(ELF))}$", buildids) is None:
            violations["bucket_map"].append("exact D2 Build ID/path absent")
    except Exception as error:
        violations["bucket_map"].append(f"Build ID validation failed: {type(error).__name__}: {error}")
    try:
        map_value = json.loads(MAP.read_text())
        map_check = verify_map_bytes(map_value)
        if map_check["machine_byte_mismatches"]:
            violations["bucket_map"].append("machine byte mismatch")
        attribution = attribute_t3(samples, raw, map_value)
        attribution["map_check"] = map_check
        if attribution["dso_mismatches"]:
            violations["bucket_map"].append("sample DSO/mapping mismatch")
        if not attribution["accepted_tids"]:
            violations["capability"].append("no CPU0 traversal sample TID")
    except Exception as error:
        violations["bucket_map"].append(f"IP normalization or map join failed: {type(error).__name__}: {error}")
    try:
        subject_validation = validate_subject(subject, "T3-SINGLE")
        if subject_validation["violations"]:
            violations["provenance"].extend(
                f"subject {name} mismatch" for name in subject_validation["violations"]
            )
    except Exception as error:
        violations["provenance"].append(f"subject evidence unavailable: {type(error).__name__}: {error}")

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
        u3_receipt_path = PARENT / "u3-single-v1/D4_ROUTE_RECEIPT.json"
        u3_receipt = json.loads(u3_receipt_path.read_text())
        need(u3_receipt.get("verdict") == "U3_SINGLE_PASS", "paired U3 receipt is not PASS")
        u3_cpu_per_edge = float(
            u3_receipt["observation"]["subject"]["traversal_thread_cpu_per_edge_ns"]
        )
        t3_cpu_per_edge = float(attribution["sampled_traversal_cpu_per_edge_ns"])
        delta = abs(t3_cpu_per_edge - u3_cpu_per_edge) / u3_cpu_per_edge * 100.0
        attribution["paired_u3_receipt_sha256"] = sha256_file(u3_receipt_path)
        attribution["paired_u3_cpu_per_edge_ns"] = u3_cpu_per_edge
        attribution["sampled_vs_paired_u3_delta_percent"] = delta
        if delta > 5.0:
            violations["perturbation"].append(f"sampled-vs-U3 delta {delta:.12f}% exceeds 5%")
        if attribution["accepted_traversal_samples"] < 50_000:
            violations["sample_coverage"].append(
                f"accepted traversal samples {attribution['accepted_traversal_samples']} below 50000"
            )
        if attribution["unattributed_percent"] > 5.0:
            violations["sample_coverage"].append(
                f"UNATTRIBUTED {attribution['unattributed_percent']:.12f}% exceeds 5%"
            )
    else:
        violations["sample_coverage"].append("attribution unavailable")
    return {
        "schema": "lay.v10.e1-traversal-d4-t3-observation.v1",
        "route": "T3-SINGLE",
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
        "perf_reader_invocations": 4,
        "perf_stat_invocations": 0,
        "pmu_events_opened": 1,
        "subject_executions": 1,
        "warmup_time_filter_or_subtraction": False,
    }


def dispatch_observation(observation: Mapping[str, Any], route: str) -> dict[str, Any]:
    priority = U3_DISPATCH if route == "U3-SINGLE" else T3_DISPATCH
    violations = observation.get("violations")
    if not isinstance(violations, dict):
        return {
            "selected_cause": "provenance",
            "selected_rank": 0,
            "verdict": "BLOCKED_PROVENANCE",
            "reason": "dispatch schema missing",
            "all_violations": violations,
        }
    selected = None
    for rank, (cause, terminal) in enumerate(priority):
        reasons = violations.get(cause)
        if not isinstance(reasons, list):
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
        "reason": "all frozen D4 route predicates passed",
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
    return PARENT / ("u3-single-v1" if route == "U3-SINGLE" else "t3-single-v1")


def failure_path(route: str) -> pathlib.Path:
    return PARENT / ("u3-single-failure-v1" if route == "U3-SINGLE" else "t3-single-failure-v1")


def state_path(route: str) -> pathlib.Path:
    return STATE / ("U3_SINGLE_STATE.json" if route == "U3-SINGLE" else "T3_SINGLE_STATE.json")


def publish_route_state(route: str, verdict: str, receipt_sha256: str | None) -> None:
    write_new_json(
        state_path(route),
        {
            "schema": "lay.v10.e1-traversal-d4-route-state.v1",
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
    need(route in ROUTES, f"unknown route: {route}")
    host = verify_host()
    d2 = verify_d2_projection()
    admission = verify_bootstrap(payload, route, post=False)
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
        write_new_bytes(inputs / "preflight-v2.json", decode_input(payload, "preflight_b64", PREFLIGHT_SHA256))
        write_new_bytes(
            inputs / "preflight-v2-receipt.json",
            decode_input(payload, "preflight_receipt_b64", PREFLIGHT_RECEIPT_SHA256),
        )
        write_new_bytes(inputs / "d2-terminal-receipt.json", decode_input(payload, "d2_terminal_b64", D2_TERMINAL_SHA256))
        write_new_bytes(inputs / "d3-terminal-receipt.json", decode_input(payload, "d3_terminal_b64", D3_TERMINAL_SHA256))
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
            "schema": "lay.v10.e1-traversal-d4-preobservation.v1",
            "task_id": TASK_ID,
            "transaction_id": TRANSACTION_ID,
            "route": route,
            "subject_command": subject_command(),
            "environment": environment,
            "admission": admission,
            "host": host,
            "d2_projection": d2,
            "marker_consumed": False,
            "retry_permitted": False,
        }
        if route == "T3-SINGLE":
            data_path = stage / "perf.data"
            preobservation["perf_record_command"] = [
                *PERF_RECORD_PREFIX,
                "--output",
                str(data_path),
                "--",
                *child_as_e(environment, subject_command()),
            ]
            preobservation["reader_commands"] = reader_commands(data_path)
        write_new_json(stage / "PREOBSERVATION.json", preobservation)
        fsync_directory(stage)
        marker = consume_marker(route)
        marker_consumed = True
        write_new_json(stage / "MARKER_CONSUMPTION.json", marker)
        observation = run_u3(stage, environment) if route == "U3-SINGLE" else run_t3(stage, environment)
        dispatch = dispatch_observation(observation, route)
        write_new_json(stage / "OBSERVATION.json", observation)
        receipt = {
            "schema": "lay.v10.e1-traversal-d4-route-receipt.v1",
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
                "T3-SINGLE only"
                if route == "U3-SINGLE" and dispatch["verdict"] == "U3_SINGLE_PASS"
                else (
                    "separate multiworker paper only; no execution or optimization"
                    if route == "T3-SINGLE" and dispatch["verdict"] == "D4_SINGLE_ESTIMATOR_PASS"
                    else "none; terminal D4 blocked verdict"
                )
            ),
        }
        write_new_json(stage / "D4_ROUTE_RECEIPT.json", receipt)
        write_sha256sums(stage)
        seal_evidence_tree(stage)
        os.rename(stage, result)
        fsync_directory(PARENT)
        published = result / "D4_ROUTE_RECEIPT.json"
        publish_route_state(route, dispatch["verdict"], sha256_file(published))
        verify_d4_markers(route, post=True)
        return {
            **receipt,
            "published_receipt_sha256": sha256_file(published),
            "remote_result": str(result),
        }
    except BaseException as error:
        if marker_consumed:
            with contextlib.suppress(BaseException):
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
                seal_evidence_tree(stage)
                os.rename(stage, failure)
                fsync_directory(PARENT)
                publish_route_state(route, "BLOCKED_PROVENANCE", None)
        elif stage.exists():
            shutil.rmtree(stage)
        raise


def probe(payload: Mapping[str, Any], action: str, route: str | None) -> dict[str, Any]:
    host = verify_host()
    d2 = verify_d2_projection()
    if action == "probe-absent":
        verify_payload(payload, require_bootstrap_audit=False)
        need(not PARENT.exists() and not STATE.exists(), "D4 namespace already exists")
        return {
            "verdict": "D4_REMOTE_ABSENT_PROBE_PASS",
            "host": host,
            "d2_projection": d2,
            "d4_parent_exists": False,
            "d4_state_exists": False,
            "remote_writes": 0,
        }
    if action == "probe-bootstrap":
        ready = verify_bootstrap_base(payload, require_bootstrap_audit=False)
        need(not (STATE / "markers").exists(), "markers exist before bootstrap audit")
        return {
            "verdict": "D4_REMOTE_BOOTSTRAP_PROBE_PASS",
            "host": host,
            "d2_projection": d2,
            "d4": ready,
            "remote_writes": 0,
        }
    need(route in ROUTES, "route required for ready/after probe")
    post = action == "probe-after"
    ready = verify_bootstrap(payload, route, post=post)
    result: dict[str, Any] = {
        "verdict": "D4_REMOTE_POST_PROBE_PASS" if post else "D4_REMOTE_READY_PROBE_PASS",
        "route": route,
        "host": host,
        "d2_projection": d2,
        "d4": ready,
        "remote_writes": 0,
    }
    if post:
        receipt_path = result_path(route) / "D4_ROUTE_RECEIPT.json"
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
        elif action in ("run-u3", "run-t3"):
            expected = "U3-SINGLE" if action == "run-u3" else "T3-SINGLE"
            need(route == expected, "action/route mismatch")
            with LOCK.open("rb") as lock:
                fcntl.flock(lock.fileno(), fcntl.LOCK_EX)
                value = route_once(payload, route)
        else:
            raise D4Error(f"unsupported action: {action!r}")
        print(json.dumps(value, sort_keys=True, separators=(",", ":")))
        return 0
    except Exception as error:
        print(f"D4 REMOTE ERROR: {type(error).__name__}: {error}", file=sys.stderr)
        return 1


if __name__ == "__main__":
    raise SystemExit(main())
