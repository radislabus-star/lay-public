#!/usr/bin/env python3
"""Read-only forensic interpretation of the sealed D5 T4-FIXED evidence."""

from __future__ import annotations

import argparse
import bisect
import collections
import hashlib
import json
import os
import pathlib
import re
import shutil
import stat
import sys
import time
from typing import Any, Mapping, Sequence


AUDITOR = pathlib.Path(__file__).resolve()
ROOT = AUDITOR.parents[1]
TASK_ID = "slice8b-v10-e1-traversal-d5-t4-offline-forensic-v1-20260826"
D5_TASK_ID = "slice8b-v10-e1-traversal-d5-multiworker-tid-estimator-v1-20260826"
D5_TRANSACTION_ID = "3ee46e2c915677e1b2d3cd6bcc9709e0232252dbc120745b097d736537779036"
REMOTE_ELF = pathlib.PurePosixPath(
    "/home/e/.local/share/lay/provenance/"
    "slice8b-v10-e1-traversal-d2-primary-only-v2-20260825/build-v1/d2-test-elf"
)
PERIOD = 200_000
T4_DENOMINATOR_EDGES = 528_060_876
TRAVERSAL_BUCKETS = {
    "DAFSA_DECODE_MEMORY",
    "TRANSITION",
    "RANK",
    "STACK_CONTROL",
    "TERMINAL",
    "UNATTRIBUTED",
}

PAPER = ROOT / (
    "docs/structural_gates/"
    "LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_D5_T4_"
    "OFFLINE_FORENSIC_INTERPRETATION_V1_2026-08-26.md"
)
STRUCTURAL_REVIEW = ROOT / (
    "docs/structural_gates/receipts/"
    "LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_D5_T4_"
    "OFFLINE_FORENSIC_STRUCTURAL_REVIEW_V1_2026-08-26.json"
)
PREFLIGHT = ROOT / (
    "docs/structural_gates/preflights/"
    "LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_D5_T4_"
    "OFFLINE_FORENSIC_IMPLEMENTATION_V2_2026-08-26.json"
)
PREFLIGHT_RECEIPT = ROOT / (
    "docs/structural_gates/receipts/"
    "LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_D5_T4_"
    "OFFLINE_FORENSIC_IMPLEMENTATION_V2_PREFLIGHT_2026-08-26.json"
)
D5_TERMINAL_ROOT = ROOT / (
    "docs/structural_gates/receipts/"
    "LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_D5_"
    "TERMINAL_AUDIT_V1_2026-08-26"
)
D5_TERMINAL = D5_TERMINAL_ROOT / "D5_TERMINAL_AUDIT_RECEIPT.json"
U4_ROOT = ROOT / (
    "docs/structural_gates/receipts/"
    "LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_D5_"
    "U4_FIXED_V1_2026-08-26"
)
U4_EVIDENCE = U4_ROOT / "REMOTE_EVIDENCE"
U4_RECEIPT = U4_EVIDENCE / "D5_ROUTE_RECEIPT.json"
T4_ROOT = ROOT / (
    "docs/structural_gates/receipts/"
    "LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_D5_"
    "T4_FIXED_V1_2026-08-26"
)
T4_EVIDENCE = T4_ROOT / "REMOTE_EVIDENCE"
T4_RECEIPT = T4_EVIDENCE / "D5_ROUTE_RECEIPT.json"
OBSERVATION = T4_EVIDENCE / "OBSERVATION.json"
SAMPLES = T4_EVIDENCE / "samples.stdout"
RAW_RECORDS = T4_EVIDENCE / "raw-records.stdout"
PERF_DATA = T4_EVIDENCE / "perf.data"
SUBJECT_RECEIPT = T4_EVIDENCE / "subject/SUBJECT_RECEIPT.json"
MAP_ROOT = ROOT / (
    "docs/structural_gates/receipts/"
    "LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_D2_"
    "PRIMARY_ONLY_BUCKET_MAP_V1_2026-08-26"
)
MAP_EVIDENCE = MAP_ROOT / "REMOTE_EVIDENCE"
BUCKET_MAP = MAP_EVIDENCE / "D2_BUCKET_MAP.json"
RESULT = ROOT / (
    "docs/structural_gates/receipts/"
    "LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_D5_T4_"
    "OFFLINE_FORENSIC_AUDIT_V1_2026-08-26"
)

PINNED_INPUTS: dict[pathlib.Path, tuple[str, int, str]] = {
    PAPER: ("0444", 5313, "5624f6337c8df385e317f5a08b7886d3aa30768c10920bb34fa94639a2063856"),
    STRUCTURAL_REVIEW: (
        "0444",
        1134,
        "318a8c8327373f7067ec793a00c4ad8f153aaf0255aa8e30f15aec4a1a0545c4",
    ),
    PREFLIGHT: ("0444", 13601, "24fa7822e4eeb7bba11ea7bd7f2f9db39a10d71da51e3bfbe9ca98aff71441d4"),
    PREFLIGHT_RECEIPT: (
        "0444",
        9025,
        "4025eb23fbcf459eb2b5d20babdc1aac8a2688795bad5e936331bd16bd2fcc14",
    ),
    D5_TERMINAL: (
        "0444",
        2964,
        "b37e24bd87d063063d83dd30f084d7fda81fc9bdd4f1759a1643b2e6809c741a",
    ),
    U4_RECEIPT: (
        "0444",
        45133,
        "229b901d65516d7eb6041668d2974409a721e601a6acb03d52d66929895903e4",
    ),
    T4_RECEIPT: (
        "0444",
        68243,
        "d337dcddcd74e95e8009e520347f6e3cf6c1319c46bf7e896862200af8f5cbbf",
    ),
    OBSERVATION: (
        "0444",
        61044,
        "55501b090feb7d525d5942e732795006c3bc9e626386bf381b7174364ab3fe44",
    ),
    SAMPLES: (
        "0444",
        30432151,
        "e50f8c8f002ea6ea7433c4375239ba101eb54bd3a6b4bde44156e43210ac21a0",
    ),
    RAW_RECORDS: (
        "0444",
        151281354,
        "bdfd0b7bf9786ad20586606156df78a810c72d0c4fca128b30c9215d461cb89a",
    ),
    PERF_DATA: (
        "0444",
        7126578,
        "02c140c235641ed177139e8fd5b37eb4cdcc5fc952443326d944c2c7322d62b6",
    ),
    SUBJECT_RECEIPT: (
        "0444",
        1737,
        "341fdfb60d4874f2a0ec0b7458561bdba22706286aa2be6492733ff1d4a5119f",
    ),
    BUCKET_MAP: (
        "0444",
        390324,
        "2b93aedf11101a8bc5fc7ed4ffaa056b50476d96cdd2eabf9d185f59ee60f846",
    ),
}

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
RAW_PREFIX_PATTERN = re.compile(r"^(\d+)\s+(\d+)\s+")
THROTTLE_PATTERN = re.compile(r"PERF_RECORD_(THROTTLE|UNTHROTTLE)")


class ForensicError(RuntimeError):
    pass


def require(value: bool, message: str) -> None:
    if not value:
        raise ForensicError(message)


def sha256_file(path: pathlib.Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as source:
        for block in iter(lambda: source.read(1024 * 1024), b""):
            digest.update(block)
    return digest.hexdigest()


def mode(path: pathlib.Path) -> str:
    return f"{stat.S_IMODE(path.stat().st_mode):04o}"


def row(path: pathlib.Path) -> dict[str, Any]:
    require(path.is_file() and not path.is_symlink(), f"missing or invalid file: {path}")
    return {
        "path": str(path),
        "mode": mode(path),
        "size_bytes": path.stat().st_size,
        "sha256": sha256_file(path),
    }


def inventory(root: pathlib.Path) -> list[dict[str, Any]]:
    values = []
    for path in sorted(root.rglob("*")):
        require(not path.is_symlink(), f"symlink in evidence tree: {path}")
        if path.is_file():
            values.append(
                {
                    "path": path.relative_to(root).as_posix(),
                    "mode": mode(path),
                    "size_bytes": path.stat().st_size,
                    "sha256": sha256_file(path),
                }
            )
    return values


def verify_sums(root: pathlib.Path) -> int:
    manifest = root / "SHA256SUMS"
    require(manifest.is_file(), f"missing SHA256SUMS: {root}")
    expected: dict[str, str] = {}
    for line in manifest.read_text().splitlines():
        digest, separator, relative = line.partition("  ")
        require(separator == "  " and len(digest) == 64, f"bad manifest row: {line}")
        pure = pathlib.PurePosixPath(relative)
        require(not pure.is_absolute() and ".." not in pure.parts, f"unsafe manifest path: {relative}")
        require(relative not in expected, f"duplicate manifest path: {relative}")
        expected[relative] = digest
    actual = {
        item["path"]: item["sha256"]
        for item in inventory(root)
        if item["path"] != "SHA256SUMS"
    }
    unlisted = set(actual) - set(expected)
    require(
        all(pathlib.PurePosixPath(path).name == "SHA256SUMS" for path in unlisted),
        f"unlisted payload outside SHA256SUMS: {root}",
    )
    require(
        set(expected) <= set(actual)
        and all(actual[path] == digest for path, digest in expected.items()),
        f"SHA256SUMS membership or digest mismatch: {root}",
    )
    return len(expected)


def write_new(path: pathlib.Path, value: bytes, file_mode: int = 0o444) -> None:
    descriptor = os.open(path, os.O_WRONLY | os.O_CREAT | os.O_EXCL, 0o600)
    try:
        view = memoryview(value)
        while view:
            written = os.write(descriptor, view)
            require(written > 0, "short write made no progress")
            view = view[written:]
        os.fsync(descriptor)
        os.fchmod(descriptor, file_mode)
    finally:
        os.close(descriptor)


def write_json(path: pathlib.Path, value: Any) -> None:
    write_new(path, json.dumps(value, sort_keys=True, indent=2).encode() + b"\n")


def write_sums(root: pathlib.Path) -> None:
    entries = [
        item
        for item in inventory(root)
        if pathlib.PurePosixPath(item["path"]).name != "SHA256SUMS"
    ]
    write_new(
        root / "SHA256SUMS",
        "".join(f"{item['sha256']}  {item['path']}\n" for item in entries).encode(),
    )


def seal(root: pathlib.Path) -> None:
    for path in sorted(root.rglob("*"), reverse=True):
        path.chmod(0o555 if path.is_dir() or path.stat().st_mode & 0o111 else 0o444)
    root.chmod(0o555)


def fsync_dir(path: pathlib.Path) -> None:
    descriptor = os.open(path, os.O_RDONLY | os.O_DIRECTORY)
    try:
        os.fsync(descriptor)
    finally:
        os.close(descriptor)


def parse_time_ns(value: str) -> int:
    seconds, dot, fraction = value.partition(".")
    require(dot == "." and seconds.isdigit() and fraction.isdigit(), f"bad sample time: {value}")
    require(len(fraction) <= 9, f"sample precision exceeds nanoseconds: {value}")
    return int(seconds) * 1_000_000_000 + int(fraction.ljust(9, "0"))


def parse_samples(path: pathlib.Path) -> list[dict[str, Any]]:
    values = []
    with path.open(encoding="utf-8", errors="replace") as source:
        for line_number, line in enumerate(source, start=1):
            match = SAMPLE_PATTERN.fullmatch(line.rstrip("\n"))
            require(match is not None, f"unparsed sample row {line_number}: {line[:240]!r}")
            values.append(
                {
                    "line_number": line_number,
                    "comm": match.group(1).strip(),
                    "pid": int(match.group(2)),
                    "tid": int(match.group(3)),
                    "cpu": int(match.group(4)),
                    "time": match.group(5),
                    "time_ns": parse_time_ns(match.group(5)),
                    "period": int(match.group(6)),
                    "event": match.group(7),
                    "runtime_ip": int(match.group(8), 16),
                    "dso": match.group(9),
                }
            )
    return values


def scan_raw(path: pathlib.Path) -> dict[str, Any]:
    mappings: list[dict[str, Any]] = []
    forks: list[dict[str, Any]] = []
    comms: list[dict[str, Any]] = []
    exits: list[dict[str, Any]] = []
    open_throttle: dict[int, int] = {}
    throttle_pairs: list[dict[str, int]] = []
    lost = 0
    samples = 0
    with path.open(encoding="utf-8", errors="replace") as source:
        for line_number, line in enumerate(source, start=1):
            prefix = RAW_PREFIX_PATTERN.match(line)
            event_cpu = int(prefix.group(1)) if prefix is not None else None
            event_time_ns = int(prefix.group(2)) if prefix is not None else None
            if "PERF_RECORD_LOST" in line or "PERF_RECORD_LOST_SAMPLES" in line:
                lost += 1
            if "PERF_RECORD_SAMPLE" in line:
                samples += 1
            throttle = THROTTLE_PATTERN.search(line)
            if throttle is not None:
                require(event_cpu is not None and event_time_ns is not None, f"throttle prefix missing: {line_number}")
                kind = throttle.group(1)
                if kind == "THROTTLE":
                    require(event_cpu not in open_throttle, f"nested throttle on CPU {event_cpu}")
                    open_throttle[event_cpu] = event_time_ns
                else:
                    require(event_cpu in open_throttle, f"unmatched unthrottle on CPU {event_cpu}")
                    start = open_throttle.pop(event_cpu)
                    require(event_time_ns > start, f"non-positive throttle duration on CPU {event_cpu}")
                    throttle_pairs.append(
                        {
                            "cpu": event_cpu,
                            "start_time_ns": start,
                            "end_time_ns": event_time_ns,
                            "duration_ns": event_time_ns - start,
                        }
                    )
            match = MMAP2_PATTERN.search(line)
            if match is not None and match.group(7) == str(REMOTE_ELF):
                mappings.append(
                    {
                        "pid": int(match.group(1)),
                        "tid": int(match.group(2)),
                        "start": int(match.group(3), 16),
                        "length": int(match.group(4), 16),
                        "offset": int(match.group(5), 16),
                        "permissions": match.group(6),
                        "path": match.group(7),
                        "event_cpu": event_cpu,
                        "event_time_ns": event_time_ns,
                    }
                )
            match = FORK_PATTERN.search(line)
            if match is not None:
                require(event_cpu is not None and event_time_ns is not None, f"FORK prefix missing: {line_number}")
                forks.append(
                    {
                        "pid": int(match.group(1)),
                        "tid": int(match.group(2)),
                        "parent_pid": int(match.group(3)),
                        "parent_tid": int(match.group(4)),
                        "event_cpu": event_cpu,
                        "event_time_ns": event_time_ns,
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
    require(not open_throttle, f"unclosed throttle CPUs: {sorted(open_throttle)}")
    return {
        "mappings": mappings,
        "forks": forks,
        "comms": comms,
        "exits": exits,
        "lost_records": lost,
        "raw_sample_records": samples,
        "throttle_pairs": throttle_pairs,
    }


def verify_inputs() -> dict[str, Any]:
    inputs = {}
    for path, (expected_mode, expected_size, expected_sha) in PINNED_INPUTS.items():
        value = row(path)
        require(value["mode"] == expected_mode, f"input mode drift: {path}")
        require(value["size_bytes"] == expected_size, f"input size drift: {path}")
        require(value["sha256"] == expected_sha, f"input SHA drift: {path}")
        inputs[path.relative_to(ROOT).as_posix()] = value
    manifests = {
        "d5-terminal": verify_sums(D5_TERMINAL_ROOT),
        "u4-local": verify_sums(U4_ROOT),
        "u4-remote": verify_sums(U4_EVIDENCE),
        "t4-local": verify_sums(T4_ROOT),
        "t4-remote": verify_sums(T4_EVIDENCE),
        "map-local": verify_sums(MAP_ROOT),
        "map-remote": verify_sums(MAP_EVIDENCE),
    }
    preflight = json.loads(PREFLIGHT_RECEIPT.read_text())
    require(
        preflight.get("verdict") == "READY_TO_IMPLEMENT"
        and preflight.get("safe_to_implement") is True,
        "effective preflight is not READY_TO_IMPLEMENT",
    )
    structural = json.loads(STRUCTURAL_REVIEW.read_text())
    require(
        structural.get("verdict") == "STRUCTURALLY_ACCEPTED_WITH_SPLIT"
        and structural.get("all_routes_pass") is True
        and structural.get("authority_ready") is False,
        "structural split review drift",
    )
    return {"inputs": inputs, "manifests": manifests}


def verify_history() -> dict[str, Any]:
    terminal = json.loads(D5_TERMINAL.read_text())
    require(
        terminal.get("verdict") == "BLOCKED_PROVENANCE"
        and terminal.get("terminal_route") == "T4-FIXED"
        and terminal.get("terminal_cause") == "provenance"
        and terminal.get("claim_valid") is False
        and terminal.get("optimization_authority") is False
        and terminal.get("retry_permitted") is False,
        "D5 terminal authority drift",
    )
    u4 = json.loads(U4_RECEIPT.read_text())
    require(u4.get("verdict") == "U4_FIXED_PASS" and u4.get("route") == "U4-FIXED", "U4 receipt drift")
    u4_cpu = u4.get("observation", {}).get("subject", {}).get("traversal_thread_cpu_per_edge_ns")
    require(isinstance(u4_cpu, (int, float)) and not isinstance(u4_cpu, bool), "U4 CPU/edge missing")
    t4 = json.loads(T4_RECEIPT.read_text())
    require(
        t4.get("verdict") == "BLOCKED_PROVENANCE"
        and t4.get("route") == "T4-FIXED"
        and t4.get("retry_permitted") is False,
        "T4 receipt drift",
    )
    observation = json.loads(OBSERVATION.read_text())
    require(observation.get("complete") is False and observation.get("route") == "T4-FIXED", "T4 observation drift")
    subject = json.loads(SUBJECT_RECEIPT.read_text())
    require(
        subject.get("mapping") == "FIXED"
        and subject.get("workers") == 20
        and subject.get("rounds") == 20
        and subject.get("warmup_bursts") == 1
        and subject.get("queries") == 382,
        "T4 subject envelope drift",
    )
    return {
        "d5_terminal": terminal,
        "u4_cpu_per_edge_ns": float(u4_cpu),
        "t4_dispatch": t4["dispatch"],
        "t4_original_violations": observation["violations"],
        "subject": subject,
    }


def reconstruct_lifecycle(raw: Mapping[str, Any], subject_pid: int) -> dict[str, Any]:
    subject_forks = [
        value
        for value in raw["forks"]
        if value["pid"] == subject_pid and value["parent_pid"] == subject_pid
    ]
    children: dict[int, list[int]] = collections.defaultdict(list)
    for value in subject_forks:
        children[int(value["parent_tid"])].append(int(value["tid"]))
    direct = sorted(set(children.get(subject_pid, [])))
    candidates = []
    for parent_tid in direct:
        worker_tids = sorted(set(children.get(parent_tid, [])))
        if len(worker_tids) == 20:
            candidates.append((parent_tid, worker_tids))
    require(len(candidates) == 1, f"twenty-worker parent candidate count: {len(candidates)}")
    parent_tid, worker_tids = candidates[0]
    require(direct == [parent_tid], f"subject leader direct children drift: {direct}")
    require(all(not children.get(tid) for tid in worker_tids), "worker graph is not terminal")
    require(len(subject_forks) == 21, f"subject FORK count: {len(subject_forks)}")
    exits = [value for value in raw["exits"] if value["pid"] == subject_pid and value["tid"] in worker_tids]
    require(len(exits) == 20 and sorted(value["tid"] for value in exits) == worker_tids, "worker EXIT closure drift")
    subject_tids = {subject_pid, parent_tid, *worker_tids}
    comms = [value for value in raw["comms"] if value["pid"] == subject_pid]
    require(not sorted({value["tid"] for value in comms} - subject_tids), "unknown subject COMM TID")
    require(any(value["tid"] == parent_tid for value in comms), "libtest parent COMM missing")
    fork_by_tid = {value["tid"]: value for value in subject_forks if value["tid"] in worker_tids}
    return {
        "subject_pid": subject_pid,
        "test_parent_tid": parent_tid,
        "worker_tids": worker_tids,
        "worker_count": 20,
        "subject_fork_count": len(subject_forks),
        "worker_exit_count": len(exits),
        "fork_by_tid": fork_by_tid,
        "unique": True,
    }


def projection(rows_by_tid: Mapping[int, Sequence[Mapping[str, Any]]], worker_tids: Sequence[int]) -> dict[str, Any]:
    workers = {}
    for tid in worker_tids:
        rows = list(rows_by_tid.get(tid, []))
        workers[str(tid)] = {
            "samples": len(rows),
            "cpus": sorted({int(value["cpu"]) for value in rows}),
            "first_time_ns": min((int(value["time_ns"]) for value in rows), default=None),
            "last_time_ns": max((int(value["time_ns"]) for value in rows), default=None),
        }
    anomalies = {tid: value for tid, value in workers.items() if len(value["cpus"]) != 1}
    singleton_cpus = [value["cpus"][0] for value in workers.values() if len(value["cpus"]) == 1]
    return {
        "workers": workers,
        "anomalous_tids": anomalies,
        "all_workers_sampled": all(value["samples"] > 0 for value in workers.values()),
        "singleton_cpu_closure": sorted(singleton_cpus),
    }


def interpret(
    samples: list[dict[str, Any]], raw: Mapping[str, Any], map_value: Mapping[str, Any], history: Mapping[str, Any]
) -> dict[str, Any]:
    require(len(samples) == raw["raw_sample_records"], "raw and rendered sample count mismatch")
    require({value["event"] for value in samples} == {"task-clock:u"}, "event identity drift")
    require({value["period"] for value in samples} == {PERIOD}, "sample period drift")

    segment = map_value.get("executable_pt_load", {})
    ranges = map_value.get("ranges")
    require(isinstance(ranges, list) and len(ranges) == 46, "bucket map range count drift")
    starts = [int(value["start"]) for value in ranges]
    require(starts == sorted(starts), "bucket map starts are not sorted")
    page_size = os.sysconf("SC_PAGE_SIZE")
    expected_offset = int(segment["offset"]) // page_size * page_size
    mappings = [
        value for value in raw["mappings"] if "x" in value["permissions"] and value["offset"] == expected_offset
    ]
    require(len(mappings) == 1, f"executable D2 mapping count: {len(mappings)}")
    mapping = mappings[0]
    aligned_vaddr = int(segment["vaddr"]) // page_size * page_size
    load_bias = int(mapping["start"]) - aligned_vaddr
    lifecycle = reconstruct_lifecycle(raw, int(mapping["pid"]))
    worker_tids = lifecycle["worker_tids"]
    worker_set = set(worker_tids)

    all_worker: dict[int, list[dict[str, Any]]] = collections.defaultdict(list)
    exact_d2_worker: dict[int, list[dict[str, Any]]] = collections.defaultdict(list)
    traversal_worker: dict[int, list[dict[str, Any]]] = collections.defaultdict(list)
    buckets: collections.Counter[str] = collections.Counter()
    sub_buckets: collections.Counter[str] = collections.Counter()
    accepted_by_tid: collections.Counter[int] = collections.Counter()
    d2_by_cpu: collections.Counter[int] = collections.Counter()
    dso_mismatches = []
    excluded_non_worker_traversal = 0
    scientific_outside_traversal = 0
    outside_mapped_text = 0

    for sample in samples:
        tid = int(sample["tid"])
        if tid in worker_set:
            all_worker[tid].append(sample)
        inside = mapping["start"] <= sample["runtime_ip"] < mapping["start"] + mapping["length"]
        rendered_exact = sample["dso"] == str(REMOTE_ELF)
        if not inside and not rendered_exact:
            continue
        if not inside or not rendered_exact:
            dso_mismatches.append(
                {"line_number": sample["line_number"], "runtime_ip": sample["runtime_ip"], "dso": sample["dso"]}
            )
            continue
        d2_by_cpu[int(sample["cpu"])] += 1
        if tid in worker_set:
            exact_d2_worker[tid].append(sample)
        normalized = int(sample["runtime_ip"]) - load_bias
        position = bisect.bisect_right(starts, normalized) - 1
        if position < 0 or not (
            int(ranges[position]["start"]) <= normalized < int(ranges[position]["end_exclusive"])
        ):
            outside_mapped_text += 1
            continue
        mapped = ranges[position]
        bucket = mapped["bucket"]
        if tid not in worker_set:
            if bucket != "OUTSIDE_TRAVERSAL":
                excluded_non_worker_traversal += 1
            continue
        if bucket == "OUTSIDE_TRAVERSAL":
            scientific_outside_traversal += 1
            continue
        require(bucket in TRAVERSAL_BUCKETS, f"unknown traversal bucket: {bucket}")
        traversal_worker[tid].append(sample)
        buckets[bucket] += 1
        sub_buckets[mapped["sub_bucket"]] += 1
        accepted_by_tid[tid] += 1

    require(not dso_mismatches, "D2 DSO and mapping identity mismatch")
    all_projection = projection(all_worker, worker_tids)
    d2_projection = projection(exact_d2_worker, worker_tids)
    traversal_projection = projection(traversal_worker, worker_tids)
    require(traversal_projection["all_workers_sampled"], "one or more workers lack traversal samples")
    require(not traversal_projection["anomalous_tids"], "traversal worker CPU set is not singleton")
    require(
        traversal_projection["singleton_cpu_closure"] == list(range(20)),
        f"traversal CPU closure drift: {traversal_projection['singleton_cpu_closure']}",
    )

    expected_cpu = {
        tid: traversal_projection["workers"][str(tid)]["cpus"][0] for tid in worker_tids
    }
    first_d2 = {
        tid: min((value["time_ns"] for value in exact_d2_worker[tid]), default=None) for tid in worker_tids
    }
    first_traversal = {
        tid: min((value["time_ns"] for value in traversal_worker[tid]), default=None) for tid in worker_tids
    }
    foreign_samples = []
    for tid in worker_tids:
        fork_time = lifecycle["fork_by_tid"][tid]["event_time_ns"]
        for sample in all_worker[tid]:
            if int(sample["cpu"]) == expected_cpu[tid]:
                continue
            foreign_samples.append(
                {
                    "line_number": sample["line_number"],
                    "pid": sample["pid"],
                    "tid": tid,
                    "cpu": sample["cpu"],
                    "expected_traversal_cpu": expected_cpu[tid],
                    "time": sample["time"],
                    "time_ns": sample["time_ns"],
                    "runtime_ip": sample["runtime_ip"],
                    "runtime_ip_hex": f"0x{sample['runtime_ip']:x}",
                    "dso": sample["dso"],
                    "delta_from_fork_ns": sample["time_ns"] - fork_time,
                    "delta_to_first_exact_d2_ns": first_d2[tid] - sample["time_ns"],
                    "delta_to_first_traversal_ns": first_traversal[tid] - sample["time_ns"],
                    "exact_d2": sample["dso"] == str(REMOTE_ELF),
                    "scientific_traversal": sample in traversal_worker[tid],
                }
            )

    accepted = sum(buckets.values())
    unattributed = buckets.get("UNATTRIBUTED", 0)
    sampled_cpu_ns = accepted * PERIOD
    sampled_cpu_per_edge = sampled_cpu_ns / T4_DENOMINATOR_EDGES
    u4_cpu_per_edge = float(history["u4_cpu_per_edge_ns"])
    diagnostic_delta = abs(sampled_cpu_per_edge - u4_cpu_per_edge) / u4_cpu_per_edge * 100.0
    projections = []
    for period in (250_000, 300_000, 350_000, 400_000):
        nominal = u4_cpu_per_edge * T4_DENOMINATOR_EDGES / period
        projections.append(
            {
                "period_ns": period,
                "nominal_samples_from_u4": nominal,
                "samples_at_minus_5_percent": nominal * 0.95,
                "samples_at_plus_5_percent": nominal * 1.05,
                "minimum_50000_margin_at_minus_5_percent": nominal * 0.95 - 50_000,
                "zero_throttle_proven": False,
            }
        )

    pairs = list(raw["throttle_pairs"])
    pair_count_by_cpu = collections.Counter(value["cpu"] for value in pairs)
    duration_by_cpu = collections.Counter()
    for value in pairs:
        duration_by_cpu[value["cpu"]] += value["duration_ns"]
    throttle = {
        "pairs": len(pairs),
        "records": len(pairs),
        "unthrottle_records": len(pairs),
        "pair_count_by_cpu": {str(key): value for key, value in sorted(pair_count_by_cpu.items())},
        "duration_ns_by_cpu": {str(key): value for key, value in sorted(duration_by_cpu.items())},
        "total_duration_ns": sum(value["duration_ns"] for value in pairs),
        "max_duration_ns": max((value["duration_ns"] for value in pairs), default=0),
        "first_start_time_ns": min((value["start_time_ns"] for value in pairs), default=None),
        "last_end_time_ns": max((value["end_time_ns"] for value in pairs), default=None),
        "complete_same_cpu_pairs": True,
        "pairs_detail": pairs,
    }

    return {
        "sample_stream": {
            "rendered_samples": len(samples),
            "raw_sample_records": raw["raw_sample_records"],
            "lost_records": raw["lost_records"],
            "event": "task-clock:u",
            "period_ns": PERIOD,
        },
        "mapping": {
            **mapping,
            "load_bias": load_bias,
            "load_bias_hex": f"0x{load_bias:x}",
            "normalization_unique": True,
            "d2_samples_by_cpu": {str(key): value for key, value in sorted(d2_by_cpu.items())},
            "outside_mapped_text_samples": outside_mapped_text,
            "dso_mismatches": dso_mismatches,
        },
        "lifecycle": lifecycle,
        "cpu_projections": {
            "all_worker_samples": all_projection,
            "exact_d2_worker_samples": d2_projection,
            "traversal_worker_samples": traversal_projection,
            "foreign_cpu_samples": foreign_samples,
        },
        "diagnostic_attribution": {
            "retrospective_only": True,
            "accepted_traversal_samples": accepted,
            "accepted_samples_by_tid": {
                str(key): accepted_by_tid[key] for key in sorted(accepted_by_tid)
            },
            "accepted_bucket_counts": dict(sorted(buckets.items())),
            "accepted_sub_bucket_counts": dict(sorted(sub_buckets.items())),
            "unattributed_samples": unattributed,
            "unattributed_percent": 100.0 * unattributed / accepted if accepted else 100.0,
            "excluded_non_worker_traversal_samples": excluded_non_worker_traversal,
            "scientific_outside_traversal_samples": scientific_outside_traversal,
            "sampled_traversal_cpu_ns": sampled_cpu_ns,
            "sampled_traversal_cpu_per_edge_ns": sampled_cpu_per_edge,
            "paired_u4_cpu_per_edge_ns": u4_cpu_per_edge,
            "diagnostic_sampled_vs_u4_delta_percent": diagnostic_delta,
            "historical_zero_throttle_gate_pass": False,
            "d5_claim_valid": False,
        },
        "throttle": throttle,
        "fixed_period_projections": projections,
    }


def self_check() -> dict[str, Any]:
    require(not RESULT.exists(), f"forensic result already exists: {RESULT}")
    inputs = verify_inputs()
    compile(AUDITOR.read_text(), str(AUDITOR), "exec")
    source = AUDITOR.read_text()
    forbidden = (
        "sub" + "process",
        "sock" + "et",
        "para" + "miko",
        "perf " + "record",
        "perf " + "script",
        "perf " + "stat",
        "ss" + "h ",
        "sc" + "p ",
        "car" + "go ",
        "rust" + "c ",
    )
    hits = [token for token in forbidden if token in source]
    require(not hits, f"external command graph token present: {hits}")
    return {
        "schema": "lay.v10.e1-traversal-d5-t4-offline-forensic-self-check.v1",
        "task_id": TASK_ID,
        "verdict": "D5_T4_FORENSIC_AUDITOR_VERIFIED_UNRUN",
        "auditor": row(AUDITOR),
        "input_count": len(PINNED_INPUTS),
        "manifest_count": len(inputs["manifests"]),
        "external_commands": 0,
        "remote_reads": 0,
        "remote_writes": 0,
        "perf_readers": 0,
        "perf_record": 0,
        "perf_stat": 0,
        "subject_executions": 0,
        "marker_mutations": 0,
        "runtime_authority_changed": False,
    }


def audit() -> dict[str, Any]:
    check = self_check()
    inputs = verify_inputs()
    history = verify_history()
    samples = parse_samples(SAMPLES)
    raw = scan_raw(RAW_RECORDS)
    map_value = json.loads(BUCKET_MAP.read_text())
    detail = interpret(samples, raw, map_value, history)
    foreign = detail["cpu_projections"]["foreign_cpu_samples"]
    diagnostic = detail["diagnostic_attribution"]
    throttle = detail["throttle"]
    receipt = {
        "schema": "lay.v10.e1-traversal-d5-t4-offline-forensic-audit.v1",
        "task_id": TASK_ID,
        "d5_task_id": D5_TASK_ID,
        "d5_transaction_id": D5_TRANSACTION_ID,
        "verdict": "D5_T4_FORENSIC_DIAGNOSTIC_COMPLETE",
        "retrospective": True,
        "d5_terminal_verdict": "BLOCKED_PROVENANCE",
        "d5_terminal_receipt_sha256": PINNED_INPUTS[D5_TERMINAL][2],
        "d5_claim_valid": False,
        "d5_optimization_authority": False,
        "d5_retry_permitted": False,
        "auditor_sha256": check["auditor"]["sha256"],
        "paper_sha256": PINNED_INPUTS[PAPER][2],
        "preflight_sha256": PINNED_INPUTS[PREFLIGHT][2],
        "preflight_receipt_sha256": PINNED_INPUTS[PREFLIGHT_RECEIPT][2],
        "rendered_samples": detail["sample_stream"]["rendered_samples"],
        "raw_sample_records": detail["sample_stream"]["raw_sample_records"],
        "worker_tids": detail["lifecycle"]["worker_tids"],
        "all_sample_cpu_anomalies": len(
            detail["cpu_projections"]["all_worker_samples"]["anomalous_tids"]
        ),
        "exact_d2_cpu_anomalies": len(
            detail["cpu_projections"]["exact_d2_worker_samples"]["anomalous_tids"]
        ),
        "traversal_cpu_anomalies": len(
            detail["cpu_projections"]["traversal_worker_samples"]["anomalous_tids"]
        ),
        "foreign_cpu_samples": len(foreign),
        "foreign_cpu_sample_scientific": sum(bool(value["scientific_traversal"]) for value in foreign),
        "accepted_traversal_samples": diagnostic["accepted_traversal_samples"],
        "unattributed_percent": diagnostic["unattributed_percent"],
        "diagnostic_sampled_vs_u4_delta_percent": diagnostic[
            "diagnostic_sampled_vs_u4_delta_percent"
        ],
        "lost_records": detail["sample_stream"]["lost_records"],
        "throttle_unthrottle": [throttle["records"], throttle["unthrottle_records"]],
        "historical_zero_throttle_gate_pass": False,
        "diagnostic_attribution_valid_for_d5": False,
        "new_perf_reader_invocations": 0,
        "perf_record": 0,
        "perf_stat": 0,
        "pmu_events_opened": 0,
        "subject_executions": 0,
        "marker_mutations": 0,
        "remote_reads": 0,
        "remote_writes": 0,
        "runtime_authority_changed": False,
        "optimization_authority": False,
        "next_action_admitted": "separate D6 paper design only",
    }
    stage = pathlib.Path(f"{RESULT}.stage-{os.getpid()}-{time.time_ns()}")
    stage.mkdir(parents=True, mode=0o700)
    try:
        write_json(stage / "D5_T4_FORENSIC_AUDIT_RECEIPT.json", receipt)
        write_json(stage / "FORENSIC_DETAIL.json", detail)
        write_json(stage / "SELF_CHECK.json", check)
        write_json(stage / "INPUT_IDENTITIES.json", inputs)
        write_new(stage / "auditor.py", AUDITOR.read_bytes(), 0o555)
        write_new(stage / "paper.md", PAPER.read_bytes())
        write_new(stage / "preflight-v2.json", PREFLIGHT.read_bytes())
        write_new(stage / "preflight-v2-receipt.json", PREFLIGHT_RECEIPT.read_bytes())
        write_new(stage / "structural-review.json", STRUCTURAL_REVIEW.read_bytes())
        write_sums(stage)
        seal(stage)
        os.rename(stage, RESULT)
        fsync_dir(RESULT.parent)
    except BaseException:
        if stage.exists():
            for path in [stage, *stage.rglob("*")]:
                path.chmod(0o700 if path.is_dir() else 0o600)
            shutil.rmtree(stage)
        raise
    output = dict(receipt)
    output["receipt_sha256"] = sha256_file(RESULT / "D5_T4_FORENSIC_AUDIT_RECEIPT.json")
    output["result"] = str(RESULT)
    return output


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("action", choices=("self-check", "audit"))
    args = parser.parse_args()
    try:
        value = self_check() if args.action == "self-check" else audit()
        print(json.dumps(value, sort_keys=True))
        return 0
    except Exception as error:
        print(f"D5 T4 FORENSIC ERROR: {type(error).__name__}: {error}", file=sys.stderr)
        return 1


if __name__ == "__main__":
    raise SystemExit(main())
