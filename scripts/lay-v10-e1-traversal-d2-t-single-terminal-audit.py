#!/usr/bin/env python3
"""Audit sealed T-SINGLE evidence and publish the terminal D2 interpretation."""

from __future__ import annotations

import argparse
import ast
import base64
import bisect
import contextlib
import hashlib
import json
import os
import pathlib
import re
import shlex
import shutil
import stat
import subprocess
import time
from collections import Counter, defaultdict
from typing import Any, Mapping, Sequence


TASK_ID = "slice8b-v10-e1-traversal-d2-primary-only-v2-20260825"
TRANSACTION_ID = "35dbe5c5eaec7dbb5b5b112313a3285dbe2ceaea0a910719ad379255ca5569e7"
PROJECT_ROOT = pathlib.Path(__file__).resolve().parents[1]
CONTROLLER = pathlib.Path(__file__).resolve()
CORRECTION = PROJECT_ROOT / (
    "docs/structural_gates/"
    "LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_D2_"
    "T_SINGLE_ESTIMATOR_SCOPE_CORRECTION_V1_2026-08-26.md"
)
PREFLIGHT = PROJECT_ROOT / (
    "docs/structural_gates/preflights/"
    "LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_D2_"
    "PRIMARY_ONLY_IMPLEMENTATION_V4_2026-08-25.json"
)
FRAGMENT = PROJECT_ROOT / "scripts/lay_v10_e1_remaining_cost_d1_test_module.rs.inc"
MAP = PROJECT_ROOT / (
    "docs/structural_gates/receipts/"
    "LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_D2_"
    "PRIMARY_ONLY_BUCKET_MAP_V1_2026-08-26/REMOTE_EVIDENCE/D2_BUCKET_MAP.json"
)
MAP_AUDIT = PROJECT_ROOT / (
    "docs/structural_gates/receipts/"
    "LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_D2_"
    "PRIMARY_ONLY_BUCKET_MAP_AUDIT_V1_2026-08-26/D2_BUCKET_MAP_AUDIT_RECEIPT.json"
)
HISTORICAL = PROJECT_ROOT / (
    "docs/structural_gates/receipts/"
    "LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_D2_"
    "PRIMARY_ONLY_T_SINGLE_V1_2026-08-26"
)
REMOTE_EVIDENCE = HISTORICAL / "REMOTE_EVIDENCE"
RESULT = PROJECT_ROOT / (
    "docs/structural_gates/receipts/"
    "LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_D2_"
    "PRIMARY_ONLY_T_SINGLE_TERMINAL_AUDIT_V1_2026-08-26"
)

REMOTE = "e@192.168.3.94"
SSH_IDENTITY = pathlib.Path("/home/ubu/.ssh/mega-mini-admin")
REMOTE_STATE = pathlib.PurePosixPath(
    "/home/e/.local/state/lay/"
    "slice8b-v10-e1-traversal-d2-primary-only-v2-20260825"
)

EXPECTED = {
    "correction": "88c12093bb3cf76395b3f5c37d48991ebb90a3c34a6581d5801f3cd4fb2001f4",
    "preflight": "e9d6328b9f610ede73ae2e8d3c819b9728ac0fc5e4c263e9fe25a61978f80f5a",
    "fragment": "bbd8b8d318810eec721812f21efbeb5f231dacba774cb5ade854e2201c6c7665",
    "map": "2b93aedf11101a8bc5fc7ed4ffaa056b50476d96cdd2eabf9d185f59ee60f846",
    "map_audit": "8d2e52574adca52d5a090067cdf071afba922363ccee8fc9eb4a4158cb92cba7",
    "historical_local_receipt": "8e20921889924dc7967c770bac7827b4e4553a68aecd9c08617a17db9538b10c",
    "historical_remote_receipt": "afaeb7d3caffb1967dd76021e42b94664803cef5d0ed72ec574fb54526a8fa0d",
    "observation": "ff492ade004efff36c0f1200f5a7a02cc34445f08b6d3e9958fcd5e096f7b66a",
    "samples": "883763966d1d0b8e6ad142a4a22214c33db23764b14e96099b1cf2d11b9d78e3",
    "raw_records": "6536eb150de8d27b91b088f2de50b9bd41528e8bad5987ad71be381ce370669f",
    "subject_receipt": "b3e4c3843ccc6f8ef298866cbbbcb82469004b192ad5b8257d0a47975cf65bae",
    "remote_controller": "428452cd2937deb644299d8d1ad0280d649eeaf7da6b3b05452a07f26b11146b",
}

PERIOD_NS = 100_000
ROUNDS = 20
EXAMINED_EDGES = 25_145_756 * ROUNDS
PAIRED_U_CPU_NS = 13_106_179_024
HISTORICAL_REASON = "traversal sample CPU outside frozen mapping"
EFFECTIVE_REASON = (
    "whole-process stream contains pre-pinning d1_load_inputs samples in sealed "
    "traversal ranges, contradicting the frozen one-warmup-plus-twenty-measured estimator"
)
EXPECTED_MARKERS = {
    "build.consumed-before-exec",
    "bucket-map.consumed-before-exec",
    "parity.consumed-before-exec",
    "u-single.consumed-before-exec",
    "u-fixed.consumed-before-exec",
    "u-reversed.consumed-before-exec",
    "v-fixed-instr.consumed-before-exec",
    "v-reversed-instr.consumed-before-exec",
    "t-single.consumed-before-exec",
    "t-fixed.available",
    "t-reversed.available",
}

SAMPLE_PATTERN = re.compile(
    r"^\s*(.*?)\s+(\d+)/(\d+)\s+\[(\d{3})\]\s+([0-9]+\.[0-9]+):\s+"
    r"(\d+)\s+([^\s]+):\s+([0-9a-fA-F]+)\s+\((.*)\)\s*$"
)
MMAP2_PATTERN = re.compile(
    r"PERF_RECORD_MMAP2\s+(\d+)/(\d+):\s+\[0x([0-9a-fA-F]+)\(0x([0-9a-fA-F]+)\)\s+"
    r"@\s+0x([0-9a-fA-F]+).*?\]:\s+([rwxps-]+)\s+(.+)$"
)


class AuditError(RuntimeError):
    pass


def require(condition: bool, message: str) -> None:
    if not condition:
        raise AuditError(message)


def sha256_file(path: pathlib.Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as source:
        for block in iter(lambda: source.read(1024 * 1024), b""):
            digest.update(block)
    return digest.hexdigest()


def mode_string(path: pathlib.Path) -> str:
    return f"{stat.S_IMODE(path.stat().st_mode):04o}"


def row(path: pathlib.Path, expected_sha256: str | None = None) -> dict[str, Any]:
    require(path.is_file() and not path.is_symlink(), f"missing or invalid file: {path}")
    value = {
        "path": str(path),
        "mode": mode_string(path),
        "size_bytes": path.stat().st_size,
        "sha256": sha256_file(path),
    }
    if expected_sha256 is not None:
        require(value["sha256"] == expected_sha256, f"SHA-256 drift: {path}")
    return value


def canonical_json_bytes(value: Any) -> bytes:
    return (json.dumps(value, sort_keys=True, indent=2) + "\n").encode()


def write_new_bytes(path: pathlib.Path, value: bytes, mode: int = 0o444) -> None:
    descriptor = os.open(path, os.O_WRONLY | os.O_CREAT | os.O_EXCL, 0o600)
    try:
        view = memoryview(value)
        while view:
            written = os.write(descriptor, view)
            require(written > 0, "short write made no progress")
            view = view[written:]
        os.fsync(descriptor)
        os.fchmod(descriptor, mode)
    finally:
        os.close(descriptor)


def write_new_json(path: pathlib.Path, value: Any) -> None:
    write_new_bytes(path, canonical_json_bytes(value))


def inventory(root: pathlib.Path) -> list[dict[str, Any]]:
    values = []
    for path in sorted(root.rglob("*")):
        require(not path.is_symlink(), f"symlink in receipt: {path}")
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
    require(manifest.is_file(), f"missing SHA256SUMS: {root}")
    seen: set[str] = set()
    for line in manifest.read_text(encoding="utf-8").splitlines():
        digest, separator, relative = line.partition("  ")
        require(separator == "  " and len(digest) == 64, f"invalid manifest row: {line}")
        pure = pathlib.PurePosixPath(relative)
        require(not pure.is_absolute() and ".." not in pure.parts, f"unsafe manifest row: {relative}")
        require(relative not in seen, f"duplicate manifest row: {relative}")
        seen.add(relative)
        require(sha256_file(root / pure) == digest, f"manifest mismatch: {relative}")
    actual = {value["path"] for value in inventory(root) if value["path"] != "SHA256SUMS"}
    require(seen == actual, f"manifest membership mismatch: {root}")
    return len(seen)


def seal_tree(root: pathlib.Path) -> None:
    write_sha256sums(root)
    for path in sorted(root.rglob("*"), reverse=True):
        path.chmod(0o555 if path.is_dir() or path.stat().st_mode & 0o111 else 0o444)
    root.chmod(0o555)
    verify_sha256sums(root)


def remove_owned_tree(root: pathlib.Path) -> None:
    if not root.exists():
        return
    for path in [root, *root.rglob("*")]:
        with contextlib.suppress(OSError):
            path.chmod(0o700 if path.is_dir() else 0o600)
    shutil.rmtree(root)


def verify_controller_graph() -> dict[str, Any]:
    source = CONTROLLER.read_text(encoding="utf-8")
    tree = ast.parse(source, filename=str(CONTROLLER))
    subprocess_calls = []
    shell_true = False
    for node in ast.walk(tree):
        if not isinstance(node, ast.Call):
            continue
        name = ""
        if isinstance(node.func, ast.Attribute) and isinstance(node.func.value, ast.Name):
            name = f"{node.func.value.id}.{node.func.attr}"
        if name == "subprocess.run":
            subprocess_calls.append(node.lineno)
            shell_true = shell_true or any(
                keyword.arg == "shell"
                and isinstance(keyword.value, ast.Constant)
                and keyword.value.value is True
                for keyword in node.keywords
            )
    require(subprocess_calls == [368], f"external command graph drift: {subprocess_calls}")
    require(not shell_true, "shell=True is forbidden")
    return {
        "external_command": "/usr/bin/ssh only",
        "subprocess_run_lines": subprocess_calls,
        "shell_true": shell_true,
        "cargo_route": False,
        "rustc_route": False,
        "perf_route": False,
        "pmu_route": False,
        "subject_route": False,
        "marker_write_route": False,
        "remote_write_route": False,
    }


def verify_contract() -> dict[str, Any]:
    value = json.loads(PREFLIGHT.read_text(encoding="utf-8"))
    sampling = value["sampling_contract"]
    require(sampling["lifecycle"].startswith("whole-process perf command wrapping"), "T lifecycle drift")
    require(
        sampling["estimator"]["sample_stream"]
        == "one frozen 382-query warmup traversal round plus twenty measured rounds",
        "sampling estimator drift",
    )
    require(sampling["estimator"]["time_filter_or_warmup_subtraction"] is False, "filter rule drift")
    require(sampling["event"] == {
        "name": "task-clock:u",
        "period": 100000,
        "freq": 0,
        "exclude_kernel": 1,
        "precise_ip": 0,
    }, "event contract drift")
    require(sampling["hard_gates"]["throttle_unthrottle"] == "0/0", "coverage gate drift")
    require(
        sampling["hard_gates"]["sampled_traversal_cpu_per_edge_vs_paired_u_delta_percent_max"] == 5.0,
        "perturbation threshold drift",
    )
    priority = value["failure_dispatch_contract"]["t"]["priority"]
    require(
        [item["terminal"] for item in priority]
        == [
            "BLOCKED_PROVENANCE",
            "BLOCKED_THERMAL",
            "BLOCKED_CAPABILITY",
            "BLOCKED_BUCKET_MAP",
            "BLOCKED_PERTURBATION",
            "BLOCKED_SAMPLE_COVERAGE",
        ],
        "T dispatch drift",
    )
    serialized = json.dumps(value, sort_keys=True)
    require("sample CPU outside" not in serialized and "traversal sample CPU" not in serialized, "CPU predicate exists in V4")
    return {
        "lifecycle": sampling["lifecycle"],
        "estimator": sampling["estimator"],
        "hard_gates": sampling["hard_gates"],
        "dispatch": priority,
        "sample_cpu_subset_gate": False,
    }


def verify_source_order() -> dict[str, Any]:
    source = FRAGMENT.read_text(encoding="utf-8")
    function_start = source.index("fn d1_run_component_single()")
    function_end = source.index("fn d1_run_component_twenty()", function_start)
    body = source[function_start:function_end]
    load = body.index("let inputs = d1_load_inputs()?")
    pin = body.index("d1_pin_current_thread(0)?")
    warmup = body.index("d1_warmup_case")
    measured = body.index("for round in 0..D1_DIAGNOSTIC_ROUNDS")
    require(load < pin < warmup < measured, "component route order drift")
    validation_start = source.index("fn d1_validate_dense_alphabet")
    validation_end = source.index("fn d1_pack", validation_start)
    validation = source[validation_start:validation_end]
    require("index.edge(edge_id)?" in validation, "shared edge decoder validation call missing")
    load_start = source.index("fn d1_load_inputs")
    load_end = source.index("fn d1_alphabet_id", load_start)
    require("d1_validate_dense_alphabet(&index)?" in source[load_start:load_end], "load validation call missing")
    return {
        "order": ["d1_load_inputs", "d1_pin_current_thread(0)", "warmup", "twenty measured rounds"],
        "load_calls_dense_alphabet_validation": True,
        "dense_alphabet_validation_calls_shared_edge_decoder": True,
    }


REMOTE_PROJECTION = f'''import hashlib,json,pathlib,socket,stat
root=pathlib.Path({str(REMOTE_STATE)!r})
def sha(path):
 digest=hashlib.sha256()
 with path.open("rb") as source:
  for block in iter(lambda:source.read(1048576),b""): digest.update(block)
 return digest.hexdigest()
markers=[]
for path in sorted((root/"markers").iterdir()):
 if path.is_file():
  markers.append({{"name":path.name,"mode":f"{{stat.S_IMODE(path.stat().st_mode):04o}}","size_bytes":path.stat().st_size,"sha256":sha(path)}})
states={{}}
for path in sorted(root.glob("*_STATE.json")):
 states[path.name]=json.loads(path.read_text())
print(json.dumps({{"hostname":socket.gethostname(),"markers":markers,"states":states}},sort_keys=True,separators=(",",":")))'''


def live_projection() -> dict[str, Any]:
    encoded = base64.b64encode(REMOTE_PROJECTION.encode()).decode()
    decoder = f"import base64;exec(base64.b64decode({encoded!r}))"
    remote_command = shlex.join(["/usr/bin/sudo", "-n", "/usr/bin/python3", "-c", decoder])
    command = [
        "/usr/bin/ssh",
        "-i",
        str(SSH_IDENTITY),
        "-o",
        "BatchMode=yes",
        "-o",
        "ConnectTimeout=8",
        REMOTE,
        remote_command,
    ]
    result = subprocess.run(command, stdout=subprocess.PIPE, stderr=subprocess.PIPE, check=False, timeout=30)
    require(result.returncode == 0, f"read-only live projection failed: {result.stderr[-2000:]!r}")
    return json.loads(result.stdout)


def validate_projection(value: Mapping[str, Any]) -> dict[str, Any]:
    require(value.get("hostname") == "e-MEGA-MINI-M1-13th", "remote hostname drift")
    markers = {item["name"]: item for item in value.get("markers", [])}
    require(set(markers) == EXPECTED_MARKERS, f"remote marker set drift: {sorted(markers)}")
    require(all(item["mode"] == "0400" for item in markers.values()), "remote marker mode drift")
    state = value.get("states", {}).get("T_SINGLE_STATE.json", {})
    require(state.get("state") == "BLOCKED_PROVENANCE", "historical T state drift")
    require(state.get("route") == "T-SINGLE", "historical route drift")
    require(state.get("receipt_sha256") == EXPECTED["historical_remote_receipt"], "historical receipt drift")
    require(state.get("retry_permitted") is False, "historical retry drift")
    return {
        "markers_expected": len(EXPECTED_MARKERS),
        "markers_observed": len(markers),
        "consumed": sorted(name for name in markers if name.endswith(".consumed-before-exec")),
        "available": sorted(name for name in markers if name.endswith(".available")),
        "t_single_state": state,
    }


def parse_samples(path: pathlib.Path) -> list[dict[str, Any]]:
    values = []
    with path.open(encoding="utf-8", errors="replace") as source:
        for line_number, line in enumerate(source, start=1):
            match = SAMPLE_PATTERN.fullmatch(line.rstrip("\n"))
            require(match is not None, f"unparsed sample row {line_number}: {line[:200]!r}")
            values.append(
                {
                    "pid": int(match.group(2)),
                    "tid": int(match.group(3)),
                    "cpu": int(match.group(4)),
                    "time": float(match.group(5)),
                    "period": int(match.group(6)),
                    "event": match.group(7),
                    "runtime_ip": int(match.group(8), 16),
                    "dso": match.group(9),
                }
            )
    return values


def scan_raw(path: pathlib.Path, d2_path: str) -> dict[str, Any]:
    lost = throttle = unthrottle = raw_samples = 0
    mappings = []
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
            if match is not None and match.group(7) == d2_path:
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
        "lost_records": lost,
        "throttle_records": throttle,
        "unthrottle_records": unthrottle,
        "raw_sample_records": raw_samples,
        "mappings": mappings,
    }


def analyze_scope() -> dict[str, Any]:
    observation = json.loads((REMOTE_EVIDENCE / "OBSERVATION.json").read_text(encoding="utf-8"))
    map_value = json.loads(MAP.read_text(encoding="utf-8"))
    samples = parse_samples(REMOTE_EVIDENCE / "samples.stdout")
    require(len(samples) == 127_143, "sample count drift")
    require({sample["event"] for sample in samples} == {"task-clock:u"}, "sample event drift")
    require({sample["period"] for sample in samples} == {PERIOD_NS}, "sample period drift")

    d2_path = observation["attribution"]["mapping"]["path"]
    raw = scan_raw(REMOTE_EVIDENCE / "raw-records.stdout", d2_path)
    require(raw["raw_sample_records"] == len(samples), "raw/sample reader count mismatch")
    require(raw["lost_records"] == 0, "lost record drift")
    require(raw["throttle_records"] == 15_847 and raw["unthrottle_records"] == 15_847, "throttle count drift")

    segment = map_value["executable_pt_load"]
    page_size = os.sysconf("SC_PAGE_SIZE")
    expected_offset = segment["offset"] // page_size * page_size
    mappings = [item for item in raw["mappings"] if "x" in item["permissions"] and item["offset"] == expected_offset]
    require(len(mappings) == 1, f"D2 executable mapping count drift: {len(mappings)}")
    mapping = mappings[0]
    load_bias = mapping["start"] - segment["vaddr"] // page_size * page_size
    require(load_bias == observation["attribution"]["load_bias"], "load bias drift")

    ranges = map_value["ranges"]
    starts = [item["start"] for item in ranges]
    counts: Counter[tuple[int, str, str, str]] = Counter()
    spans: dict[int, list[float]] = defaultdict(list)
    bucket_counts: Counter[str] = Counter()
    sub_bucket_counts: Counter[str] = Counter()
    d2_samples = traversal_samples = outside_samples = outside_text = 0
    for sample in samples:
        if sample["dso"] != d2_path:
            continue
        d2_samples += 1
        spans[sample["cpu"]].append(sample["time"])
        normalized = sample["runtime_ip"] - load_bias
        position = bisect.bisect_right(starts, normalized) - 1
        if position < 0 or not (ranges[position]["start"] <= normalized < ranges[position]["end_exclusive"]):
            outside_text += 1
            continue
        item = ranges[position]
        counts[(sample["cpu"], item["bucket"], item["sub_bucket"], item["symbol"])] += 1
        if item["bucket"] == "OUTSIDE_TRAVERSAL":
            outside_samples += 1
            continue
        traversal_samples += 1
        bucket_counts[item["bucket"]] += 1
        sub_bucket_counts[item["sub_bucket"]] += 1

    historical_attribution = observation["attribution"]
    require(d2_samples == historical_attribution["d2_samples"] == 123_718, "D2 sample count drift")
    require(traversal_samples == historical_attribution["traversal_samples"] == 106_901, "traversal count drift")
    require(outside_samples == historical_attribution["outside_traversal_samples"] == 16_817, "outside count drift")
    require(outside_text == historical_attribution["outside_mapped_text_samples"] == 0, "outside text drift")
    require(dict(sorted(bucket_counts.items())) == historical_attribution["bucket_counts"], "bucket count replay drift")
    require(dict(sorted(sub_bucket_counts.items())) == historical_attribution["sub_bucket_counts"], "sub-bucket replay drift")

    cpu6 = Counter()
    cpu6_d2 = 0
    cpu6_outside = 0
    cpu6_symbols = Counter()
    for (cpu, bucket, sub_bucket, symbol), count in counts.items():
        if cpu != 6:
            continue
        cpu6_d2 += count
        if bucket == "OUTSIDE_TRAVERSAL":
            cpu6_outside += count
        else:
            cpu6[sub_bucket] += count
            cpu6_symbols[symbol] += count
    require(cpu6_d2 == 4_395 and cpu6_outside == 4_379, "CPU6 D2 scope replay drift")
    require(cpu6 == {"EDGE_DECODE": 6, "STATE_DECODE": 8, "SYMBOL_DECODE": 2}, "CPU6 traversal replay drift")
    require(set(cpu6_symbols) == {
        "<lay::nanda_wave::l2_field::v13_typed_peak::V13DafsaView>::edge",
        "<lay::nanda_wave::l2_field::v13_typed_peak::V13DafsaView>::state",
    }, "CPU6 traversal symbol drift")
    worker_pid = mapping["pid"]
    worker_tid = observation["attribution"]["traversal_tids"][0]
    worker_spans: dict[int, list[float]] = defaultdict(list)
    for sample in samples:
        if sample["pid"] == worker_pid and sample["tid"] == worker_tid:
            worker_spans[sample["cpu"]].append(sample["time"])
    require(min(spans[6]) == 2809072.503217 and max(spans[6]) == 2809073.078265, "CPU6 D2 interval drift")
    require(min(worker_spans[6]) == 2809072.503217 and max(worker_spans[6]) == 2809073.079066, "CPU6 worker interval drift")
    require(min(worker_spans[0]) == 2809073.079286 and max(worker_spans[6]) < min(worker_spans[0]), "pre-pin boundary drift")

    subject = json.loads((REMOTE_EVIDENCE / "subject/SUBJECT_RECEIPT.json").read_text(encoding="utf-8"))
    require(subject["cpus"] == [0] and subject["thread_migrations"] == 0, "subject affinity evidence drift")
    sampled_cpu_per_edge = traversal_samples * PERIOD_NS / EXAMINED_EDGES
    paired_u_cpu_per_edge = PAIRED_U_CPU_NS / EXAMINED_EDGES
    delta = abs(sampled_cpu_per_edge - paired_u_cpu_per_edge) / paired_u_cpu_per_edge * 100.0
    require(abs(delta - historical_attribution["sampled_vs_paired_u_delta_percent"]) < 1e-12, "delta replay drift")
    counterfactual_samples = traversal_samples - sum(cpu6.values())
    counterfactual_cpu_per_edge = counterfactual_samples * PERIOD_NS / EXAMINED_EDGES
    counterfactual_delta = abs(counterfactual_cpu_per_edge - paired_u_cpu_per_edge) / paired_u_cpu_per_edge * 100.0

    return {
        "sample_reader_rows": len(samples),
        "d2_samples": d2_samples,
        "traversal_samples": traversal_samples,
        "outside_traversal_samples": outside_samples,
        "outside_mapped_text_samples": outside_text,
        "load_bias": f"0x{load_bias:x}",
        "cpu_spans": {
            str(cpu): {"samples": len(times), "first": min(times), "last": max(times)}
            for cpu, times in sorted(spans.items())
        },
        "worker_cpu_spans": {
            str(cpu): {"samples": len(times), "first": min(times), "last": max(times)}
            for cpu, times in sorted(worker_spans.items())
        },
        "pre_pinning_cpu6": {
            "d2_samples": cpu6_d2,
            "outside_traversal_samples": cpu6_outside,
            "traversal_range_samples": sum(cpu6.values()),
            "sub_bucket_counts": dict(sorted(cpu6.items())),
            "symbols": dict(sorted(cpu6_symbols.items())),
            "d2_interval": {"first": min(spans[6]), "last": max(spans[6])},
            "worker_event_interval": {"first": min(worker_spans[6]), "last": max(worker_spans[6])},
            "first_cpu0_worker_event": min(worker_spans[0]),
        },
        "subject_affinity": {"cpus": subject["cpus"], "thread_migrations": subject["thread_migrations"]},
        "bucket_counts": dict(sorted(bucket_counts.items())),
        "sub_bucket_counts": dict(sorted(sub_bucket_counts.items())),
        "raw_records": {key: raw[key] for key in ("lost_records", "throttle_records", "unthrottle_records", "raw_sample_records")},
        "sampled_cpu_per_edge_ns": sampled_cpu_per_edge,
        "paired_u_cpu_per_edge_ns": paired_u_cpu_per_edge,
        "sampled_vs_paired_u_delta_percent": delta,
        "counterfactual_diagnostic_only": {
            "removed_pre_pinning_traversal_samples": sum(cpu6.values()),
            "sampled_cpu_per_edge_ns": counterfactual_cpu_per_edge,
            "sampled_vs_paired_u_delta_percent": counterfactual_delta,
            "scientific_authority": False,
        },
    }


def self_check() -> dict[str, Any]:
    require(not RESULT.exists(), f"terminal audit already exists: {RESULT}")
    identities = {
        "controller": row(CONTROLLER),
        "correction": row(CORRECTION, EXPECTED["correction"]),
        "preflight": row(PREFLIGHT, EXPECTED["preflight"]),
        "fragment": row(FRAGMENT, EXPECTED["fragment"]),
        "bucket_map": row(MAP, EXPECTED["map"]),
        "bucket_map_audit": row(MAP_AUDIT, EXPECTED["map_audit"]),
        "historical_local_receipt": row(HISTORICAL / "LOCAL_T_ROUTE_RECEIPT.json", EXPECTED["historical_local_receipt"]),
        "historical_remote_receipt": row(REMOTE_EVIDENCE / "D2_T_ROUTE_RECEIPT.json", EXPECTED["historical_remote_receipt"]),
        "observation": row(REMOTE_EVIDENCE / "OBSERVATION.json", EXPECTED["observation"]),
        "samples": row(REMOTE_EVIDENCE / "samples.stdout", EXPECTED["samples"]),
        "raw_records": row(REMOTE_EVIDENCE / "raw-records.stdout", EXPECTED["raw_records"]),
        "subject_receipt": row(REMOTE_EVIDENCE / "subject/SUBJECT_RECEIPT.json", EXPECTED["subject_receipt"]),
        "remote_controller": row(REMOTE_EVIDENCE / "inputs/remote-controller.py", EXPECTED["remote_controller"]),
    }
    historical_entries = verify_sha256sums(HISTORICAL)
    remote_entries = verify_sha256sums(REMOTE_EVIDENCE)
    receipt = json.loads((REMOTE_EVIDENCE / "D2_T_ROUTE_RECEIPT.json").read_text(encoding="utf-8"))
    require(receipt["verdict"] == "BLOCKED_PROVENANCE", "historical verdict drift")
    require(receipt["dispatch"]["reason"] == HISTORICAL_REASON, "historical reason drift")
    require(receipt["retry_permitted"] is False, "historical retry drift")
    remote_source = (REMOTE_EVIDENCE / "inputs/remote-controller.py").read_text(encoding="utf-8")
    require(HISTORICAL_REASON in remote_source, "historical CPU predicate missing")
    return {
        "schema": "lay.v10.e1-traversal-d2-t-single-terminal-audit-self-check.v1",
        "verdict": "PASS_UNRUN",
        "identities": identities,
        "historical_manifest_entries": historical_entries,
        "remote_manifest_entries": remote_entries,
        "command_graph": verify_controller_graph(),
        "frozen_contract": verify_contract(),
        "source_order": verify_source_order(),
        "side_effects": {
            "cargo": 0,
            "rustc": 0,
            "perf_record": 0,
            "perf_stat": 0,
            "pmu": 0,
            "d2_subject": 0,
            "marker_mutations": 0,
            "remote_writes": 0,
        },
    }


def audit() -> pathlib.Path:
    check = self_check()
    live_before = live_projection()
    live_summary = validate_projection(live_before)
    scope = analyze_scope()
    historical = json.loads((REMOTE_EVIDENCE / "D2_T_ROUTE_RECEIPT.json").read_text(encoding="utf-8"))
    all_violations = historical["dispatch"]["all_violations"]
    require(all_violations["provenance"] == [HISTORICAL_REASON], "historical provenance set drift")
    require(all_violations["perturbation"] == ["sampled-vs-U delta 18.434656047164% exceeds 5%"], "historical perturbation drift")
    require(all_violations["sample_coverage"] == ["lost/throttle/unthrottle=0/15847/15847"], "historical coverage drift")

    effective_violations = {
        "provenance": [EFFECTIVE_REASON],
        "thermal": list(all_violations["thermal"]),
        "capability": list(all_violations["capability"]),
        "bucket_map": list(all_violations["bucket_map"]),
        "perturbation": list(all_violations["perturbation"]),
        "sample_coverage": list(all_violations["sample_coverage"]),
    }
    live_after = live_projection()
    validate_projection(live_after)
    require(live_after == live_before, "live marker/state projection changed during audit")

    receipt = {
        "schema": "lay.v10.e1-traversal-d2-t-single-terminal-audit.v1",
        "task_id": TASK_ID,
        "transaction_id": TRANSACTION_ID,
        "verdict": "BLOCKED_PROVENANCE",
        "terminal_scope": "D2 primary-only attribution route",
        "historical_execution": {
            "verdict": historical["verdict"],
            "receipt": check["identities"]["historical_remote_receipt"],
            "selected_reason": historical["dispatch"]["reason"],
            "selected_reason_contractual": False,
            "marker_consumed": True,
            "retry_permitted": False,
            "receipt_modified": False,
        },
        "effective_interpretation": {
            "selected_cause": "provenance",
            "selected_rank": 0,
            "reason": EFFECTIVE_REASON,
            "verdict": "BLOCKED_PROVENANCE",
            "all_violations": effective_violations,
        },
        "scope_analysis": scope,
        "frozen_contract": check["frozen_contract"],
        "source_order": check["source_order"],
        "sealed_evidence": check["identities"],
        "historical_manifest_entries": check["historical_manifest_entries"],
        "remote_manifest_entries": check["remote_manifest_entries"],
        "diagnostic_bucket_values": {
            "scientific_authority": False,
            "reason": "estimator-scope drift, perturbation failure, and throttle/unthrottle records",
            "bucket_counts": scope["bucket_counts"],
            "sub_bucket_counts": scope["sub_bucket_counts"],
        },
        "claim_boundary": {
            "d2_attribution_established": False,
            "concurrency_inflation_cause_established": False,
            "plus_18_77_ns_per_edge_explained": False,
            "secondary_instruction_gap_closed": False,
            "optimization_authority": False,
            "runtime_integration_admitted": False,
            "deployment_admitted": False,
        },
        "live_projection": live_summary,
        "live_projection_stable": True,
        "remaining_routes": {
            "t_fixed_marker": "available and unconsumed",
            "t_reversed_marker": "available and unconsumed",
            "t_fixed_admitted": False,
            "t_reversed_admitted": False,
        },
        "side_effect_ledger": {
            "offline_sample_reader_passes": 1,
            "offline_raw_reader_passes": 1,
            "read_only_remote_projections": 2,
            "cargo_invocations": 0,
            "rustc_compilations": 0,
            "perf_record": 0,
            "perf_stat": 0,
            "pmu_events": 0,
            "d2_subject_executions": 0,
            "marker_mutations": 0,
            "remote_writes": 0,
            "runtime_authority_changed": False,
        },
        "next_action_admitted": "none within D2; a new paper route would be required",
    }

    parent = RESULT.parent
    stage = parent / f".{RESULT.name}.stage-{os.getpid()}-{time.monotonic_ns()}"
    require(not stage.exists(), f"stage exists: {stage}")
    stage.mkdir(mode=0o700)
    try:
        write_new_json(stage / "SELF_CHECK.json", check)
        write_new_json(stage / "LIVE_STATE_BEFORE.json", live_before)
        write_new_json(stage / "LIVE_STATE_AFTER.json", live_after)
        write_new_json(stage / "T_SINGLE_TERMINAL_AUDIT_RECEIPT.json", receipt)
        shutil.copyfile(CONTROLLER, stage / "auditor.py")
        (stage / "auditor.py").chmod(0o444)
        shutil.copyfile(CORRECTION, stage / "correction.md")
        (stage / "correction.md").chmod(0o444)
        shutil.copyfile(REMOTE_EVIDENCE / "D2_T_ROUTE_RECEIPT.json", stage / "historical-remote-receipt.json")
        (stage / "historical-remote-receipt.json").chmod(0o444)
        seal_tree(stage)
        os.rename(stage, RESULT)
        descriptor = os.open(parent, os.O_RDONLY | os.O_DIRECTORY)
        try:
            os.fsync(descriptor)
        finally:
            os.close(descriptor)
    except Exception:
        remove_owned_tree(stage)
        raise
    return RESULT


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("action", choices=("self-check", "audit"))
    args = parser.parse_args()
    try:
        if args.action == "self-check":
            print(json.dumps(self_check(), sort_keys=True, indent=2))
            return 0
        result = audit()
        receipt = result / "T_SINGLE_TERMINAL_AUDIT_RECEIPT.json"
        print(json.dumps({"result": str(result), "receipt_sha256": sha256_file(receipt), "verdict": "BLOCKED_PROVENANCE"}, sort_keys=True))
        return 0
    except Exception as error:
        print(f"BLOCKED_PROVENANCE: {type(error).__name__}: {error}", file=os.sys.stderr)
        return 2


if __name__ == "__main__":
    raise SystemExit(main())
