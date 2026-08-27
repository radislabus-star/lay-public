#!/usr/bin/env python3
"""Offline W1 machine-cost decomposition from sealed D4, D2, and D7 evidence."""

from __future__ import annotations

import argparse
import ast
import bisect
import collections
import hashlib
import json
import math
import os
import pathlib
import re
import shutil
import stat
import subprocess
import sys
import time
from typing import Any, Iterable, Mapping


AUDITOR = pathlib.Path(__file__).resolve()
ROOT = AUDITOR.parents[1]
TASK_ID = "slice8b-v10-e1-traversal-w1-machine-cost-decomposition-v1-20260826"
RESULT_NAME = (
    "LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_"
    "W1_MACHINE_COST_DECOMPOSITION_V1_2026-08-26"
)
RECEIPTS = ROOT / "docs/structural_gates/receipts"

PAPER = ROOT / (
    "docs/structural_gates/LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_"
    "W1_MACHINE_COST_DECOMPOSITION_V1_2026-08-26.md"
)
EVIDENCE_ROUTE = ROOT / (
    "docs/structural_gates/LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_"
    "W1_MACHINE_COST_DECOMPOSITION_V1_ROUTE_A_EVIDENCE.md"
)
BOUNDARY_ROUTE = ROOT / (
    "docs/structural_gates/LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_"
    "W1_MACHINE_COST_DECOMPOSITION_V1_ROUTE_B_BOUNDARY.md"
)
STRUCTURAL_REVIEW = RECEIPTS / (
    "LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_"
    "W1_MACHINE_COST_DECOMPOSITION_STRUCTURAL_REVIEW_V1_2026-08-26.json"
)
PREFLIGHT = ROOT / (
    "docs/structural_gates/preflights/"
    "LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_"
    "W1_MACHINE_COST_DECOMPOSITION_IMPLEMENTATION_V1_2026-08-26.json"
)
PREFLIGHT_RECEIPT = RECEIPTS / (
    "LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_"
    "W1_MACHINE_COST_DECOMPOSITION_IMPLEMENTATION_V1_PREFLIGHT_2026-08-26.json"
)

D7_ROOT = RECEIPTS / (
    "LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_"
    "D7_WORKER_TOPOLOGY_SWEEP_V1_2026-08-26"
)
D7_TERMINAL = D7_ROOT / "D7_TERMINAL_AUDIT.json"
D7_DECISION = D7_ROOT / "REMOTE_RESULT/D7_DECISION.json"
D4_TERMINAL_ROOT = RECEIPTS / (
    "LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_"
    "D4_TERMINAL_AUDIT_V1_2026-08-26"
)
D4_TERMINAL = D4_TERMINAL_ROOT / "D4_TERMINAL_AUDIT_RECEIPT.json"
D4_T3 = D4_TERMINAL_ROOT / "T3_SCIENTIFIC_RECEIPT.json"
D4_U3 = D4_TERMINAL_ROOT / "U3_SCIENTIFIC_RECEIPT.json"
D4_T3_ROOT = RECEIPTS / (
    "LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_"
    "D4_T3_SINGLE_V1_2026-08-26"
)
SAMPLES = D4_T3_ROOT / "REMOTE_EVIDENCE/samples.stdout"

D2_MAP_ROOT = RECEIPTS / (
    "LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_"
    "D2_PRIMARY_ONLY_BUCKET_MAP_V1_2026-08-26"
)
D2_MAP = D2_MAP_ROOT / "REMOTE_EVIDENCE/D2_BUCKET_MAP.json"
D2_MAP_AUDIT_ROOT = RECEIPTS / (
    "LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_"
    "D2_PRIMARY_ONLY_BUCKET_MAP_AUDIT_V1_2026-08-26"
)
D2_MAP_AUDIT = D2_MAP_AUDIT_ROOT / "D2_BUCKET_MAP_AUDIT_RECEIPT.json"
D2_BUILD_ROOT = RECEIPTS / (
    "LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_"
    "D2_PRIMARY_ONLY_BUILD_V1_2026-08-25"
)
D2_ELF = D2_BUILD_ROOT / "REMOTE_EVIDENCE/d2-test-elf"
D2_SOURCE = D2_BUILD_ROOT / "REMOTE_EVIDENCE/assembled_d2_source.rs"
D2_V_FIXED_ROOT = RECEIPTS / (
    "LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_"
    "D2_PRIMARY_ONLY_V_FIXED_INSTR_V1_2026-08-26"
)
D2_V_FIXED = D2_V_FIXED_ROOT / "REMOTE_EVIDENCE/D2_UV_ROUTE_RECEIPT.json"
D2_V_REVERSED_ROOT = RECEIPTS / (
    "LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_E1_TRAVERSAL_"
    "D2_PRIMARY_ONLY_V_REVERSED_INSTR_V1_2026-08-26"
)
D2_V_REVERSED = D2_V_REVERSED_ROOT / "REMOTE_EVIDENCE/D2_UV_ROUTE_RECEIPT.json"

OBJDUMP = pathlib.Path("/usr/bin/objdump")
RESULT = RECEIPTS / RESULT_NAME

D2_ELF_SHA256 = "bb7c68528ec3fec7074919c3c64cce233d48c26efefe9d9b32735c5f5865a178"
D2_BUILD_ID = "eb951f1a7526a9f1cb365040c10989aa5d3fc50f"
D7_ELF_SHA256 = "26316b349d8192c697facf1ed5929fcc7133fc8bde15bdde6eae53a438e0f138"
D7_BUILD_ID = "4d6280e7324975076be3edc4f40802c26910180a"
ACCEPTED_TID = 99_366
ACCEPTED_CPU = 0
STAGING_CPU = 6
LOAD_BIAS = 0x73A003000000
PERIOD_NS = 200_000

EXPECTED_BUCKETS = {
    "DAFSA_DECODE_MEMORY": 11_138,
    "RANK": 9,
    "STACK_CONTROL": 5_682,
    "TERMINAL": 610,
    "TRANSITION": 49_104,
}
EXPECTED_SUB_BUCKETS = {
    "ALPHABET_ID": 765,
    "BUDGET_DEADLINE": 288,
    "EDGE_DECODE": 6_781,
    "EDGE_RANGE_CONTROL": 538,
    "EDGE_RANK_ADD": 9,
    "EQUALITY_WINDOW": 3_417,
    "FORM_REF_COLLECTION": 4,
    "FUSED_SCALAR_U64_ADVANCE": 44_922,
    "PRUNE_AND_LOOP": 934,
    "SCRATCH_BOOKKEEPING": 1_171,
    "STACK_POP": 240,
    "STACK_PUSH": 3_049,
    "STATE_DECODE": 2_673,
    "SYMBOL_DECODE": 1_146,
    "TERMINAL_DISTANCE": 506,
    "TERMINAL_PREDICATE": 100,
}
EXPECTED_FUSED_RANGES = {
    (0x778725, 0x778A7C): 6_492,
    (0x778C2E, 0x778DB7): 27_970,
    (0x778E09, 0x778F26): 10_460,
}
MINIMUM_BLOCKS = {
    "setup": (0x778D17, 0x778D60, 1_124),
    "vector_reduction": (0x778D60, 0x778D9D, 16_739),
    "scalar_tail": (0x778D9D, 0x778DB7, 2_976),
}
EXPECTED_DISASSEMBLY_SPANS = [
    (0x778320, 0x7793AE),
    (0x926520, 0x926643),
    (0x9266B0, 0x926808),
]

PINNED: dict[pathlib.Path, tuple[str, int, str]] = {
    PAPER: ("0444", 8811, "1470dabcabee76f1bac78bab592c0a7c670a2813fb9a190dd40bdbb216f99d2e"),
    EVIDENCE_ROUTE: ("0444", 2944, "dde9f4a0a56e4ac9e7705cfb3cd8dc0aabad6f8784b7c3e4877a8cb9e5bb66fc"),
    BOUNDARY_ROUTE: ("0444", 2804, "2cbfb34a4ac9f9272428c2e629c8b188f14f20a28a543f4c0c8136298e77c70b"),
    STRUCTURAL_REVIEW: ("0444", 1318, "68818825574c8c180c43549040ff1ba9ded9a4e0a776eec722ea37c5fbbe6626"),
    PREFLIGHT: ("0444", 13004, "0808cb28a4655a269b676b1ccfcb3c44625f13f94ff938e35f3f843ef37b0a43"),
    PREFLIGHT_RECEIPT: ("0444", 8936, "1510d2b7424860ecf908c2f4592ea1b50bcbf4d304d8c137a8b8b47bb88debe4"),
    D7_TERMINAL: ("0444", 11633, "db8f8fbb2ab0bbf6ba45ca9b4d2ce7c394c3de826d82961ce938adea79024f3e"),
    D4_TERMINAL: ("0444", 3685, "f748a7c448f7f142a342efa2b916a8995969044567a3aaa9a7de3aff6ba8645b"),
    D4_T3: ("0444", 60563, "dd4e3b7bb49d368fe1461c36fda0968af629293e801500770ca9dc3715a96f09"),
    D4_U3: ("0444", 42897, "db2ba1b3d4e11ac2c4edb24e382f93d282b1b5d605fcbb1636f5e346030dc000"),
    SAMPLES: ("0444", 16442675, "455cc9f2812dd37d79bbdb0abb4a3797f0a7fdfb137772610f3c290798be1233"),
    D2_MAP: ("0444", 390324, "2b93aedf11101a8bc5fc7ed4ffaa056b50476d96cdd2eabf9d185f59ee60f846"),
    D2_MAP_AUDIT: ("0444", 4363, "8d2e52574adca52d5a090067cdf071afba922363ccee8fc9eb4a4158cb92cba7"),
    D2_ELF: ("0555", 317706232, D2_ELF_SHA256),
    D2_SOURCE: ("0444", 204722, "6cd9edece91ac2e0c0e6dda7658e104dcf8953f1c16b1acff6108ea44ada0181"),
    D2_V_FIXED: ("0444", 50090, "56c862759c95de6682571aed3d68098dab084319817fba5af3853042e0396bae"),
    D2_V_REVERSED: ("0444", 50181, "5d75a502ad9e509dc6810b5494067f602d5ace1237e73f4d7913d5b9b1fc9de2"),
    OBJDUMP: ("0755", 415760, "44f07f8da860b15bd4dec909f229dec536595fb170a616fe3ab29c7b21c9736f"),
}

MANIFESTS: dict[pathlib.Path, tuple[int, str]] = {
    D7_ROOT: (8455, "18702c9ec0f5f2030bcaa8833f987b42579d357100f6936399267c2146e986d8"),
    D4_TERMINAL_ROOT: (620, "c16943e01905ebc38cc82c89e864a439ee3e98faf5c623092a006fece151d274"),
    D4_T3_ROOT: (3859, "51fb27081ebb78afb4485ac761bff5ef6d08d51aff651f436445fe60c8b7602b"),
    D2_MAP_ROOT: (3197, "48fce814340a62afdfbfbd62f382539d9e29fe385d0bf8cdcb7f16b0fe8a079d"),
    D2_MAP_AUDIT_ROOT: (787, "97ce0e3a39a22f53c27b5ed3a5c9faade341fd4cf9543fe1822c1154ba914ce3"),
    D2_BUILD_ROOT: (65533, "aead466e48392f22db9394fc601724b7d8e90930f515c993390c37ab4702f28b"),
    D2_V_FIXED_ROOT: (2777, "19d6cc17f7bec1c0f0edcf12a41b9525e23e2d8065407144c7af850654aaefaa"),
    D2_V_REVERSED_ROOT: (2777, "21154082b2f2b4ba9b010eeb29efb55a0a187484f126d4549a934907715f5637"),
}


class DecompositionError(RuntimeError):
    pass


def require(condition: bool, message: str) -> None:
    if not condition:
        raise DecompositionError(message)


def canonical_json_bytes(value: Any) -> bytes:
    return json.dumps(value, sort_keys=True, separators=(",", ":")).encode()


def sha256_bytes(value: bytes) -> str:
    return hashlib.sha256(value).hexdigest()


_SHA_CACHE: dict[pathlib.Path, str] = {}


def sha256_file(path: pathlib.Path) -> str:
    resolved = path.resolve()
    cached = _SHA_CACHE.get(resolved)
    if cached is not None:
        return cached
    digest = hashlib.sha256()
    with path.open("rb") as handle:
        for chunk in iter(lambda: handle.read(1024 * 1024), b""):
            digest.update(chunk)
    value = digest.hexdigest()
    _SHA_CACHE[resolved] = value
    return value


def mode(path: pathlib.Path) -> str:
    return f"{stat.S_IMODE(path.stat().st_mode):04o}"


def path_label(path: pathlib.Path) -> str:
    try:
        return path.relative_to(ROOT).as_posix()
    except ValueError:
        return str(path)


def row(path: pathlib.Path) -> dict[str, Any]:
    return {
        "path": path_label(path),
        "mode": mode(path),
        "size_bytes": path.stat().st_size,
        "sha256": sha256_file(path),
    }


def write_new(path: pathlib.Path, data: bytes, file_mode: int = 0o444) -> None:
    descriptor = os.open(path, os.O_WRONLY | os.O_CREAT | os.O_EXCL, 0o600)
    try:
        view = memoryview(data)
        while view:
            written = os.write(descriptor, view)
            require(written > 0, "short write made no progress")
            view = view[written:]
        os.fsync(descriptor)
        os.fchmod(descriptor, file_mode)
    finally:
        os.close(descriptor)


def write_json(path: pathlib.Path, value: Any) -> None:
    write_new(path, json.dumps(value, indent=2, sort_keys=True).encode() + b"\n")


def fsync_dir(path: pathlib.Path) -> None:
    descriptor = os.open(path, os.O_RDONLY | os.O_DIRECTORY)
    try:
        os.fsync(descriptor)
    finally:
        os.close(descriptor)


def inventory(root: pathlib.Path) -> list[dict[str, Any]]:
    return [
        {
            "path": path.relative_to(root).as_posix(),
            "mode": mode(path),
            "size_bytes": path.stat().st_size,
            "sha256": sha256_file(path),
        }
        for path in sorted(root.rglob("*"))
        if path.is_file()
    ]


def write_sums(root: pathlib.Path) -> None:
    entries = [item for item in inventory(root) if item["path"] != "SHA256SUMS"]
    write_new(
        root / "SHA256SUMS",
        "".join(f"{item['sha256']}  {item['path']}\n" for item in entries).encode(),
    )


def seal(root: pathlib.Path) -> None:
    for path in sorted(root.rglob("*"), reverse=True):
        executable = path.is_dir() or bool(path.stat().st_mode & 0o111)
        path.chmod(0o555 if executable else 0o444)
    root.chmod(0o555)


def copy_input(source: pathlib.Path, destination: pathlib.Path) -> None:
    write_new(destination, source.read_bytes(), 0o555 if source == AUDITOR else 0o444)


def verify_pinned() -> dict[str, dict[str, Any]]:
    values: dict[str, dict[str, Any]] = {}
    for path, (expected_mode, expected_size, expected_sha) in PINNED.items():
        require(path.is_file(), f"missing pinned input: {path}")
        value = row(path)
        require(value["mode"] == expected_mode, f"mode drift: {path}")
        require(value["size_bytes"] == expected_size, f"size drift: {path}")
        require(value["sha256"] == expected_sha, f"SHA drift: {path}")
        values[value["path"]] = value
    return values


def verify_manifest_rows(manifest: pathlib.Path, base: pathlib.Path) -> int:
    expected: dict[str, str] = {}
    for line in manifest.read_text().splitlines():
        digest, separator, relative = line.partition("  ")
        require(separator == "  " and re.fullmatch(r"[0-9a-f]{64}", digest) is not None, f"bad manifest row: {line}")
        pure = pathlib.PurePosixPath(relative)
        require(not pure.is_absolute() and ".." not in pure.parts, f"unsafe manifest path: {relative}")
        require(relative not in expected, f"duplicate manifest path: {relative}")
        expected[relative] = digest
    for relative, digest in expected.items():
        target = base / relative
        require(target.is_file(), f"manifest target missing: {target}")
        require(sha256_file(target) == digest, f"manifest digest drift: {target}")
    return len(expected)


def verify_manifest(root: pathlib.Path, expected_size: int, expected_sha: str) -> dict[str, Any]:
    manifest = root / "SHA256SUMS"
    require(manifest.is_file(), f"manifest missing: {root}")
    require(mode(manifest) == "0444", f"manifest mode drift: {manifest}")
    require(manifest.stat().st_size == expected_size, f"manifest size drift: {manifest}")
    require(sha256_file(manifest) == expected_sha, f"manifest SHA drift: {manifest}")
    entries = verify_manifest_rows(manifest, root)
    listed = {
        line.partition("  ")[2]
        for line in manifest.read_text().splitlines()
        if line
    }
    actual = {
        path.relative_to(root).as_posix()
        for path in root.rglob("*")
        if path.is_file() and path != manifest
    }
    unlisted = sorted(actual - listed)
    require(
        all(pathlib.PurePosixPath(relative).name == "SHA256SUMS" for relative in unlisted),
        f"unlisted non-manifest input: {root}",
    )
    nested = []
    for nested_manifest in sorted(root.rglob("SHA256SUMS")):
        if nested_manifest == manifest:
            continue
        nested.append(
            {
                "manifest": row(nested_manifest),
                "listed_by_parent": nested_manifest.relative_to(root).as_posix() in listed,
                "locally_replayed": False,
                "reason": "sealed remote metadata may describe a deliberately partial local projection",
            }
        )
    return {
        "root": path_label(root),
        "entries": entries,
        "manifest": row(manifest),
        "unlisted_nested_manifests": unlisted,
        "nested_manifests": nested,
    }


def verify_all_manifests() -> dict[str, Any]:
    return {
        path_label(root): verify_manifest(root, expected_size, expected_sha)
        for root, (expected_size, expected_sha) in MANIFESTS.items()
    }


def json_file(path: pathlib.Path) -> dict[str, Any]:
    value = json.loads(path.read_text())
    require(isinstance(value, dict), f"JSON root is not an object: {path}")
    return value


def predecessor_closure() -> dict[str, Any]:
    structural = json_file(STRUCTURAL_REVIEW)
    require(
        structural.get("verdict") == "STRUCTURALLY_ACCEPTED_WITH_SPLIT"
        and structural.get("all_routes_pass") is True
        and structural.get("authority_ready") is False,
        "W1 structural review drift",
    )
    preflight = json_file(PREFLIGHT_RECEIPT)
    require(
        preflight.get("verdict") == "READY_TO_IMPLEMENT"
        and preflight.get("safe_to_implement") is True,
        "W1 implementation preflight drift",
    )

    d7 = json_file(D7_TERMINAL)
    require(d7.get("verdict") == "D7_WORKER_TOPOLOGY_SWEEP_COMPLETE", "D7 verdict drift")
    require(d7.get("production_policy_admitted") is False, "D7 production authority drift")
    require(d7.get("runtime_authority_changed") is False, "D7 runtime authority drift")
    points = d7.get("scientific", {}).get("frontiers", {}).get("points", [])
    w1_matches = [value for value in points if value.get("route") == "W1"]
    require(len(w1_matches) == 1, "D7 W1 point cardinality drift")
    w1 = w1_matches[0]
    expected_w1 = {
        "traversal_ns_per_edge": 25.923669775527927,
        "instructions_per_edge": 361.20658023962375,
        "cycles_per_edge": 103.44625074306774,
        "ipc": 3.491731963652915,
        "effective_frequency_ghz": 3.791036767260203,
    }
    for key, expected in expected_w1.items():
        require(math.isclose(float(w1[key]), expected, rel_tol=0.0, abs_tol=1e-12), f"D7 W1 {key} drift")
    d7_decision = json_file(D7_DECISION)
    encoded_decision = canonical_json_bytes(d7_decision)
    require(D7_ELF_SHA256.encode() in encoded_decision, "D7 ELF SHA missing from decision")
    require(D7_BUILD_ID.encode() in encoded_decision, "D7 Build ID missing from decision")

    d4 = json_file(D4_TERMINAL)
    require(d4.get("verdict") == "D4_SINGLE_ESTIMATOR_PASS", "D4 verdict drift")
    require(d4.get("optimization_authority") is False, "D4 optimization authority drift")
    require(d4.get("accepted_traversal_samples") == 66_543, "D4 accepted sample count drift")
    require(d4.get("accepted_bucket_counts") == EXPECTED_BUCKETS, "D4 bucket count drift")
    require(d4.get("accepted_sub_bucket_counts") == EXPECTED_SUB_BUCKETS, "D4 sub-bucket count drift")
    require(d4.get("lost_records") == 0, "D4 lost-record drift")
    require(d4.get("throttle_records") == 0 and d4.get("unthrottle_records") == 0, "D4 throttle drift")
    require(d4.get("unattributed_samples") == 0, "D4 unattributed drift")
    require(d4.get("normalization_unique") is True, "D4 normalization drift")
    require(d4.get("machine_byte_mismatches") == 0, "D4 byte-map drift")
    require(
        math.isclose(float(d4["sampled_vs_u3_delta_percent"]), 3.343809548379481, abs_tol=1e-12),
        "D4 paired perturbation drift",
    )

    t3 = json_file(D4_T3)
    require(t3.get("route") == "T3-SINGLE" and t3.get("verdict") == "D4_SINGLE_ESTIMATOR_PASS", "T3 receipt drift")
    require(t3.get("elf", {}).get("sha256") == D2_ELF_SHA256, "T3 ELF identity drift")
    require(t3.get("map", {}).get("sha256") == PINNED[D2_MAP][2], "T3 map identity drift")
    event = t3.get("observation", {}).get("event_validation", {})
    require(
        event.get("type") == 1
        and event.get("config") == 1
        and event.get("sample_period") == PERIOD_NS
        and event.get("freq") == 0
        and event.get("inherit") == 1
        and event.get("exclude_kernel") == 1
        and event.get("precise_ip") == 0,
        "T3 event identity drift",
    )
    raw = t3.get("observation", {}).get("raw_records", {})
    require(raw.get("raw_sample_records") == 79_048, "T3 raw sample count drift")
    require(raw.get("lost_records") == 0, "T3 raw lost count drift")
    require(raw.get("throttle_records") == 0 and raw.get("unthrottle_records") == 0, "T3 raw throttle drift")
    attribution = t3.get("observation", {}).get("attribution", {})
    require(attribution.get("accepted_tids") == [ACCEPTED_TID], "T3 accepted TID drift")
    require(attribution.get("load_bias") == LOAD_BIAS, "T3 load bias drift")
    require(attribution.get("filter", {}).get("sample_cpu") == ACCEPTED_CPU, "T3 accepted CPU drift")
    require(attribution.get("filter", {}).get("elf_build_id") == D2_BUILD_ID, "T3 Build ID drift")
    require(attribution.get("accepted_traversal_samples") == 66_543, "T3 accepted sample drift")
    require(attribution.get("staging_traversal_samples_excluded") == 11, "T3 staging exclusion drift")
    require(attribution.get("scientific_outside_traversal_samples") == 7_688, "T3 outside count drift")
    require(attribution.get("d2_samples") == 76_936, "T3 D2 sample count drift")
    require(attribution.get("accepted_bucket_counts") == EXPECTED_BUCKETS, "T3 bucket count drift")
    require(attribution.get("accepted_sub_bucket_counts") == EXPECTED_SUB_BUCKETS, "T3 sub-bucket count drift")

    u3 = json_file(D4_U3)
    require(u3.get("route") == "U3-SINGLE" and u3.get("verdict") == "U3_SINGLE_PASS", "U3 receipt drift")
    u3_ns = float(u3.get("observation", {}).get("subject", {}).get("traversal_thread_cpu_per_edge_ns"))
    require(math.isclose(u3_ns, 26.07466312804435, abs_tol=1e-12), "U3 CPU/edge drift")

    map_audit = json_file(D2_MAP_AUDIT)
    require(map_audit.get("verdict") == "D2_BUCKET_MAP_AUDITED", "D2 map audit verdict drift")
    require(map_audit.get("map", {}).get("range_count") == 46, "D2 map-audit range count drift")
    require(map_audit.get("map", {}).get("instruction_count") == 1_064, "D2 map-audit instruction count drift")
    require(map_audit.get("map", {}).get("overlap_count") == 0, "D2 map overlap drift")
    require(map_audit.get("map", {}).get("machine_byte_hash_mismatches") == 0, "D2 map byte hash drift")
    require(map_audit.get("elf", {}).get("build_id") == D2_BUILD_ID, "D2 map-audit Build ID drift")

    fixed = json_file(D2_V_FIXED)
    reversed_value = json_file(D2_V_REVERSED)
    require(fixed.get("route") == "V-FIXED-INSTR" and fixed.get("verdict") == "V_FIXED_PASS", "V-FIXED drift")
    require(
        reversed_value.get("route") == "V-REVERSED-INSTR"
        and reversed_value.get("verdict") == "ALL_UV_VALIDITY_PASS",
        "V-REVERSED drift",
    )
    fixed_instructions = float(fixed["observation"]["details"]["parsed_g0"]["aggregates"]["instructions"]["per_examined_edge"])
    reversed_instructions = float(reversed_value["observation"]["details"]["parsed_g0"]["aggregates"]["instructions"]["per_examined_edge"])
    require(math.isclose(fixed_instructions, 363.6345951514673, abs_tol=1e-12), "V-FIXED instructions drift")
    require(math.isclose(reversed_instructions, 363.572772180778, abs_tol=1e-12), "V-REVERSED instructions drift")

    return {
        "structural_review_verdict": structural["verdict"],
        "preflight_verdict": preflight["verdict"],
        "d7_verdict": d7["verdict"],
        "d4_verdict": d4["verdict"],
        "d2_map_audit_verdict": map_audit["verdict"],
        "d7_w1": w1,
        "d4_u3_ns_per_edge": u3_ns,
        "d2_v_fixed_instructions_per_edge": fixed_instructions,
        "d2_v_reversed_instructions_per_edge": reversed_instructions,
        "d7_elf_sha256": D7_ELF_SHA256,
        "d7_build_id": D7_BUILD_ID,
        "d2_elf_sha256": D2_ELF_SHA256,
        "d2_build_id": D2_BUILD_ID,
        "d2_and_d7_elf_distinct": D2_ELF_SHA256 != D7_ELF_SHA256 and D2_BUILD_ID != D7_BUILD_ID,
    }


def verify_map() -> tuple[dict[str, Any], list[dict[str, Any]]]:
    value = json_file(D2_MAP)
    require(value.get("schema") == "lay.v10.e1-traversal-d2-bucket-map.v1", "D2 map schema drift")
    require(value.get("build_id") == D2_BUILD_ID, "D2 map Build ID drift")
    require(value.get("elf_sha256") == D2_ELF_SHA256, "D2 map ELF SHA drift")
    require(value.get("join_key") == ["Build ID", "normalized ELF virtual IP"], "D2 map join key drift")
    ranges = value.get("ranges")
    require(isinstance(ranges, list) and len(ranges) == 46, "D2 map range count drift")
    starts = [int(item["start"]) for item in ranges]
    require(starts == sorted(starts) and len(starts) == len(set(starts)), "D2 map range ordering drift")
    text = value.get("text", {})
    cursor = int(text["start"])
    for item in ranges:
        start = int(item["start"])
        end = int(item["end_exclusive"])
        require(start == cursor and end > start, f"D2 map coverage drift at {start:#x}")
        require(item.get("length_bytes") == end - start, f"D2 map range length drift at {start:#x}")
        require(item.get("build_id") == D2_BUILD_ID and item.get("elf_sha256") == D2_ELF_SHA256, "D2 range identity drift")
        cursor = end
    require(cursor == int(text["end_exclusive"]), "D2 map text end drift")
    require(value.get("coverage", {}).get("overlap_count") == 0, "D2 map overlap ledger drift")
    require(value.get("coverage", {}).get("machine_byte_hash_mismatches") == 0, "D2 map byte ledger drift")
    return value, ranges


SAMPLE_PATTERN = re.compile(
    r"^\s*(.*?)\s+(\d+)/(\d+)\s+\[(\d+)\]\s+([0-9]+\.[0-9]+):\s+"
    r"(\d+)\s+task-clock:u:\s+([0-9a-f]+)\s+\((.*)\)$"
)


def range_for_ip(ranges: list[dict[str, Any]], starts: list[int], ip: int) -> dict[str, Any] | None:
    position = bisect.bisect_right(starts, ip) - 1
    if position < 0:
        return None
    value = ranges[position]
    return value if int(value["start"]) <= ip < int(value["end_exclusive"]) else None


def parse_samples(ranges: list[dict[str, Any]], t3: Mapping[str, Any]) -> dict[str, Any]:
    starts = [int(value["start"]) for value in ranges]
    expected_dso = str(t3["elf"]["path"])
    accepted_ips: collections.Counter[int] = collections.Counter()
    buckets: collections.Counter[str] = collections.Counter()
    sub_buckets: collections.Counter[str] = collections.Counter()
    d2_by_cpu: collections.Counter[int] = collections.Counter()
    d2_samples = 0
    outside_map = 0
    staging_traversal = 0
    scientific_outside = 0
    parsed = 0
    period_values: set[int] = set()
    for line_number, line in enumerate(SAMPLES.read_text().splitlines(), 1):
        if not line:
            continue
        match = SAMPLE_PATTERN.fullmatch(line)
        require(match is not None, f"sample parse failure at line {line_number}")
        parsed += 1
        pid = int(match.group(2))
        tid = int(match.group(3))
        cpu = int(match.group(4))
        period = int(match.group(6))
        runtime_ip = int(match.group(7), 16)
        dso = match.group(8)
        period_values.add(period)
        if dso != expected_dso:
            continue
        d2_samples += 1
        d2_by_cpu[cpu] += 1
        normalized_ip = runtime_ip - LOAD_BIAS
        mapped = range_for_ip(ranges, starts, normalized_ip)
        if mapped is None:
            outside_map += 1
            continue
        traversal = mapped["bucket"] != "OUTSIDE_TRAVERSAL"
        if tid == ACCEPTED_TID and cpu == STAGING_CPU and traversal:
            staging_traversal += 1
        if tid == ACCEPTED_TID and cpu == ACCEPTED_CPU and not traversal:
            scientific_outside += 1
        if tid != ACCEPTED_TID or cpu != ACCEPTED_CPU or not traversal:
            continue
        require(pid == int(t3["observation"]["attribution"]["mapping"]["pid"]), "accepted sample PID drift")
        accepted_ips[normalized_ip] += 1
        buckets[str(mapped["bucket"])] += 1
        sub_buckets[str(mapped["sub_bucket"])] += 1

    require(parsed == 79_048, f"rendered sample count drift: {parsed}")
    require(period_values == {PERIOD_NS}, f"sample period drift: {period_values}")
    require(d2_samples == 76_936, f"D2 sample count drift: {d2_samples}")
    require(dict(sorted(d2_by_cpu.items())) == {0: 74_231, 6: 2_705}, f"D2 CPU projection drift: {d2_by_cpu}")
    require(outside_map == 0, f"D2 sample outside map: {outside_map}")
    require(staging_traversal == 11, f"staging traversal exclusion drift: {staging_traversal}")
    require(scientific_outside == 7_688, f"scientific outside-traversal drift: {scientific_outside}")
    require(sum(accepted_ips.values()) == 66_543, "accepted sample total drift")
    require(dict(sorted(buckets.items())) == EXPECTED_BUCKETS, f"bucket reproduction drift: {buckets}")
    require(dict(sorted(sub_buckets.items())) == EXPECTED_SUB_BUCKETS, f"sub-bucket reproduction drift: {sub_buckets}")
    return {
        "rendered_samples": parsed,
        "event": "task-clock:u",
        "fixed_period_ns": PERIOD_NS,
        "d2_samples": d2_samples,
        "d2_samples_by_cpu": {str(key): value for key, value in sorted(d2_by_cpu.items())},
        "outside_mapped_text_samples": outside_map,
        "accepted_tid": ACCEPTED_TID,
        "accepted_cpu": ACCEPTED_CPU,
        "load_bias": LOAD_BIAS,
        "load_bias_hex": f"0x{LOAD_BIAS:x}",
        "staging_traversal_samples_excluded": staging_traversal,
        "scientific_outside_traversal_samples": scientific_outside,
        "accepted_traversal_samples": sum(accepted_ips.values()),
        "accepted_unique_ips": len(accepted_ips),
        "accepted_bucket_counts": dict(sorted(buckets.items())),
        "accepted_sub_bucket_counts": dict(sorted(sub_buckets.items())),
        "accepted_ip_counts": accepted_ips,
    }


def run_objdump(argv: list[str]) -> subprocess.CompletedProcess[bytes]:
    require(bool(argv) and argv[0] == str(OBJDUMP), "non-objdump executable rejected")
    return subprocess.run(argv, check=True, stdout=subprocess.PIPE, stderr=subprocess.PIPE)


def merged_traversal_spans(ranges: Iterable[Mapping[str, Any]]) -> list[tuple[int, int]]:
    spans: list[tuple[int, int]] = []
    for value in ranges:
        if value["bucket"] == "OUTSIDE_TRAVERSAL":
            continue
        start, end = int(value["start"]), int(value["end_exclusive"])
        if spans and spans[-1][1] == start:
            spans[-1] = (spans[-1][0], end)
        else:
            spans.append((start, end))
    return spans


INSTRUCTION_PATTERN = re.compile(r"^\s*([0-9a-fA-F]+):\s+((?:[0-9a-fA-F]{2}\s+)+)\s*(.*)$")


def disassemble(ranges: list[dict[str, Any]]) -> tuple[list[dict[str, Any]], list[dict[str, Any]], bytes, bytes]:
    version_argv = [str(OBJDUMP), "--version"]
    version_process = run_objdump(version_argv)
    version_text = version_process.stdout.decode(errors="replace")
    require(version_text.splitlines()[0] == "GNU objdump (GNU Binutils for Ubuntu) 2.46", "objdump version drift")
    require(version_process.stderr == b"", "objdump version stderr is non-empty")
    command_rows = [
        {
            "argv": version_argv,
            "exit_status": version_process.returncode,
            "stdout_sha256": sha256_bytes(version_process.stdout),
            "stdout_size_bytes": len(version_process.stdout),
            "stderr_sha256": sha256_bytes(version_process.stderr),
            "stderr_size_bytes": len(version_process.stderr),
        }
    ]
    spans = merged_traversal_spans(ranges)
    require(spans == EXPECTED_DISASSEMBLY_SPANS, f"disassembly span drift: {spans}")
    instructions: list[dict[str, Any]] = []
    combined = bytearray()
    for start, end in spans:
        argv = [
            str(OBJDUMP),
            "--disassemble",
            "--demangle",
            "--wide",
            "-M",
            "intel",
            f"--start-address={start:#x}",
            f"--stop-address={end:#x}",
            str(D2_ELF),
        ]
        process = run_objdump(argv)
        require(process.stderr == b"", f"objdump stderr is non-empty for {start:#x}..{end:#x}")
        command_rows.append(
            {
                "argv": argv,
                "exit_status": process.returncode,
                "stdout_sha256": sha256_bytes(process.stdout),
                "stdout_size_bytes": len(process.stdout),
                "stderr_sha256": sha256_bytes(process.stderr),
                "stderr_size_bytes": len(process.stderr),
            }
        )
        combined.extend(f"===== {start:#x}..{end:#x} =====\n".encode())
        combined.extend(process.stdout)
        if not process.stdout.endswith(b"\n"):
            combined.extend(b"\n")
        cursor = start
        for line in process.stdout.decode(errors="replace").splitlines():
            match = INSTRUCTION_PATTERN.fullmatch(line)
            if match is None:
                continue
            address = int(match.group(1), 16)
            if not start <= address < end:
                continue
            machine = bytes.fromhex(match.group(2))
            assembly = match.group(3).strip()
            require(address == cursor, f"instruction gap or overlap at {cursor:#x}")
            require(bool(machine) and bool(assembly), f"empty decoded instruction at {address:#x}")
            mnemonic, _, operands = assembly.partition(" ")
            instructions.append(
                {
                    "address": address,
                    "end_exclusive": address + len(machine),
                    "machine_hex": machine.hex(),
                    "mnemonic": mnemonic.lower(),
                    "operands": operands.strip(),
                    "assembly": assembly,
                }
            )
            cursor += len(machine)
        require(cursor == end, f"instruction stream end drift for {start:#x}..{end:#x}: {cursor:#x}")
    require(len(instructions) == 1_064, f"instruction count drift: {len(instructions)}")
    return instructions, command_rows, bytes(combined), version_process.stdout


def verify_range_bytes(ranges: list[dict[str, Any]], instructions: list[dict[str, Any]]) -> list[dict[str, Any]]:
    addresses = [int(value["address"]) for value in instructions]
    by_address = {int(value["address"]): value for value in instructions}
    summaries = []
    for value in ranges:
        if value["bucket"] == "OUTSIDE_TRAVERSAL":
            continue
        start, end = int(value["start"]), int(value["end_exclusive"])
        first = bisect.bisect_left(addresses, start)
        last = bisect.bisect_left(addresses, end)
        selected = addresses[first:last]
        require(bool(selected) and selected[0] == start, f"range start is not an instruction: {start:#x}")
        require(by_address[selected[-1]]["end_exclusive"] == end, f"range end is not instruction aligned: {end:#x}")
        require(len(selected) == int(value["instruction_count"]), f"range instruction count drift: {start:#x}")
        machine = b"".join(bytes.fromhex(by_address[address]["machine_hex"]) for address in selected)
        require(len(machine) == end - start, f"range machine length drift: {start:#x}")
        require(sha256_bytes(machine) == value["machine_bytes_sha256"], f"range byte SHA drift: {start:#x}")
        frames = value.get("source_inlined_frames", [])
        locations = sorted(
            {
                str(frame.get("location"))
                for stack in frames
                for frame in stack
                if isinstance(frame, dict) and frame.get("location")
            }
        )
        functions = sorted(
            {
                str(frame.get("function"))
                for stack in frames
                for frame in stack
                if isinstance(frame, dict) and frame.get("function")
            }
        )
        summaries.append(
            {
                "start": start,
                "start_hex": f"0x{start:x}",
                "end_exclusive": end,
                "end_exclusive_hex": f"0x{end:x}",
                "length_bytes": end - start,
                "instruction_count": len(selected),
                "machine_bytes_sha256": value["machine_bytes_sha256"],
                "bucket": value["bucket"],
                "sub_bucket": value["sub_bucket"],
                "symbol": value["symbol"],
                "classification_reason": value["classification_reason"],
                "ambiguous": value["ambiguous"],
                "source_ownership_sha256": sha256_bytes(canonical_json_bytes(frames)),
                "source_locations": locations,
                "source_functions": functions,
            }
        )
    return summaries


def source_closure() -> dict[str, Any]:
    lines = D2_SOURCE.read_text().splitlines(keepends=True)
    require(len(lines) >= 2_904, "sealed D2 source is truncated")
    window = "".join(lines[2_838:2_904])
    require(lines[2_838].strip().startswith("fn d1_u1_advance("), "d1_u1_advance source ownership drift")
    require(lines[2_892].strip() == "let minimum = cells[..len].iter().copied().min().unwrap_or(outside);", "minimum source line drift")
    required = [
        "let c0 = d1_u1_cell::<0>",
        "let c1 = if len > 1",
        "let c2 = if len > 2",
        "let c3 = if len > 3",
        "let c4 = if len > 4",
        "let c5 = if len > 5",
        "let c6 = if len > 6",
        "let cells = [c0, c1, c2, c3, c4, c5, c6];",
        "cells[..len].iter().copied().min()",
    ]
    for token in required:
        require(token in window, f"sealed source token missing: {token}")
    return {
        "source": row(D2_SOURCE),
        "function": "d1_u1_advance",
        "line_start": 2_839,
        "line_end": 2_904,
        "window_sha256": sha256_bytes(window.encode()),
        "seven_cell_recurrence_present": True,
        "separate_post_recurrence_minimum_present": True,
        "minimum_line": 2_893,
        "minimum_source": lines[2_892].strip(),
    }


def percentage(count: int, total: int) -> float:
    return count * 100.0 / total


def decomposition_detail(
    closure: Mapping[str, Any],
    ranges: list[dict[str, Any]],
    range_summaries: list[dict[str, Any]],
    sample_data: Mapping[str, Any],
    instructions: list[dict[str, Any]],
) -> dict[str, Any]:
    accepted_ips = sample_data["accepted_ip_counts"]
    instruction_by_address = {int(value["address"]): value for value in instructions}
    missing = sorted(address for address in accepted_ips if address not in instruction_by_address)
    require(not missing, f"accepted sample IP is not an instruction start: {missing[:4]}")
    total = int(sample_data["accepted_traversal_samples"])
    starts = [int(value["start"]) for value in ranges]
    mnemonic_samples: collections.Counter[str] = collections.Counter()
    mnemonic_static: collections.Counter[str] = collections.Counter(value["mnemonic"] for value in instructions)
    sampled_instructions = []
    range_counts: collections.Counter[tuple[int, int]] = collections.Counter()
    for address, count in accepted_ips.items():
        mapped = range_for_ip(ranges, starts, address)
        require(mapped is not None and mapped["bucket"] != "OUTSIDE_TRAVERSAL", f"accepted address map drift: {address:#x}")
        instruction = instruction_by_address[address]
        mnemonic_samples[instruction["mnemonic"]] += count
        range_counts[(int(mapped["start"]), int(mapped["end_exclusive"]))] += count
        sampled_instructions.append(
            {
                **instruction,
                "address_hex": f"0x{address:x}",
                "sample_count": count,
                "sample_percent": percentage(count, total),
                "bucket": mapped["bucket"],
                "sub_bucket": mapped["sub_bucket"],
            }
        )
    sampled_instructions.sort(key=lambda value: (-int(value["sample_count"]), int(value["address"])))

    fused_range_counts = {
        (int(value["start"]), int(value["end_exclusive"])): range_counts[(int(value["start"]), int(value["end_exclusive"]))]
        for value in ranges
        if value["sub_bucket"] == "FUSED_SCALAR_U64_ADVANCE"
    }
    require(fused_range_counts == EXPECTED_FUSED_RANGES, f"fused range count drift: {fused_range_counts}")
    minimum = {}
    for name, (start, end, expected) in MINIMUM_BLOCKS.items():
        count = sum(count for address, count in accepted_ips.items() if start <= address < end)
        require(count == expected, f"minimum block count drift: {name}={count}")
        minimum[name] = {
            "start": start,
            "start_hex": f"0x{start:x}",
            "end_exclusive": end,
            "end_exclusive_hex": f"0x{end:x}",
            "samples": count,
            "share_of_traversal_percent": percentage(count, total),
            "instructions": [
                {
                    **instruction_by_address[address],
                    "address_hex": f"0x{address:x}",
                    "samples": accepted_ips.get(address, 0),
                }
                for address in sorted(instruction_by_address)
                if start <= address < end
            ],
        }
    minimum_total = sum(value["samples"] for value in minimum.values())
    require(minimum_total == 20_839, f"minimum total drift: {minimum_total}")
    fused_total = EXPECTED_SUB_BUCKETS["FUSED_SCALAR_U64_ADVANCE"]
    top_address = 0x778D74
    top = instruction_by_address[top_address]
    require(accepted_ips[top_address] == 11_903, "top pminub sample count drift")
    require(top["mnemonic"] == "pminub", f"top instruction mnemonic drift: {top['mnemonic']}")
    require(sampled_instructions[0]["address"] == top_address, "top sampled instruction identity drift")

    w1 = closure["d7_w1"]
    u3_delta = abs(float(closure["d4_u3_ns_per_edge"]) - float(w1["traversal_ns_per_edge"])) / float(w1["traversal_ns_per_edge"]) * 100.0
    fixed_delta = abs(float(closure["d2_v_fixed_instructions_per_edge"]) - float(w1["instructions_per_edge"])) / float(w1["instructions_per_edge"]) * 100.0
    reversed_delta = abs(float(closure["d2_v_reversed_instructions_per_edge"]) - float(w1["instructions_per_edge"])) / float(w1["instructions_per_edge"]) * 100.0
    require(math.isclose(u3_delta, 0.5824536179633081, abs_tol=1e-12), "D4-to-D7 U delta drift")
    require(math.isclose(fixed_delta, 0.6721956477738577, abs_tol=1e-12), "D2 fixed-to-D7 instruction delta drift")
    require(math.isclose(reversed_delta, 0.6550799654824948, abs_tol=1e-12), "D2 reversed-to-D7 instruction delta drift")

    range_detail = []
    summary_by_key = {(value["start"], value["end_exclusive"]): value for value in range_summaries}
    for key, summary in sorted(summary_by_key.items()):
        count = range_counts[key]
        range_detail.append({**summary, "accepted_samples": count, "accepted_sample_percent": percentage(count, total)})

    return {
        "schema": "lay.v10.e1-traversal-w1-machine-cost-decomposition-detail.v1",
        "task_id": TASK_ID,
        "baseline": {
            "d7_w1": w1,
            "d4_u3_ns_per_edge": closure["d4_u3_ns_per_edge"],
            "d2_v_fixed_instructions_per_edge": closure["d2_v_fixed_instructions_per_edge"],
            "d2_v_reversed_instructions_per_edge": closure["d2_v_reversed_instructions_per_edge"],
            "d4_u3_vs_d7_w1_percent": u3_delta,
            "d2_v_fixed_vs_d7_w1_instructions_percent": fixed_delta,
            "d2_v_reversed_vs_d7_w1_instructions_percent": reversed_delta,
            "largest_instruction_delta_percent": max(fixed_delta, reversed_delta),
        },
        "sample_stream": {key: value for key, value in sample_data.items() if key != "accepted_ip_counts"},
        "instruction_start_closure": {
            "accepted_samples": total,
            "accepted_samples_at_decoded_instruction_start": total,
            "match_percent": 100.0,
            "accepted_unique_ips": len(accepted_ips),
            "missing_instruction_starts": [],
        },
        "machine_ranges": range_detail,
        "sampled_instructions": sampled_instructions,
        "mnemonic_sample_counts": dict(sorted(mnemonic_samples.items(), key=lambda item: (-item[1], item[0]))),
        "mnemonic_static_instruction_counts": dict(sorted(mnemonic_static.items())),
        "fused_scalar_u64_advance": {
            "samples": fused_total,
            "share_of_traversal_percent": percentage(fused_total, total),
            "ranges": [
                {
                    "start": start,
                    "start_hex": f"0x{start:x}",
                    "end_exclusive": end,
                    "end_exclusive_hex": f"0x{end:x}",
                    "samples": count,
                    "share_of_fused_percent": percentage(count, fused_total),
                    "share_of_traversal_percent": percentage(count, total),
                }
                for (start, end), count in sorted(fused_range_counts.items())
            ],
            "post_recurrence_minimum": {
                "blocks": minimum,
                "samples": minimum_total,
                "share_of_traversal_percent": percentage(minimum_total, total),
                "share_of_fused_percent": percentage(minimum_total, fused_total),
                "top_sampled_ip": {
                    **top,
                    "address_hex": f"0x{top_address:x}",
                    "samples": accepted_ips[top_address],
                    "share_of_traversal_percent": percentage(accepted_ips[top_address], total),
                    "interpretation": "non-precise task-clock concentration in the dependency region; not exact instruction latency",
                },
            },
        },
        "sealed_source": source_closure(),
        "claim_boundary": {
            "exact_d4_machine_range_ordering": True,
            "exact_d2_static_machine_instructions": True,
            "bounded_d4_to_d7_mechanism_comparability": True,
            "exact_d7_per_ip_attribution": False,
            "exact_bucket_cycles_from_sample_share": False,
            "single_pminub_latency_share_claim": False,
            "seven_cell_recurrence_removable_claim": False,
            "separate_post_recurrence_minimum_selected_as_hypothesis": True,
            "optimization_authority": False,
        },
    }


def self_check() -> dict[str, Any]:
    require(not RESULT.exists(), f"W1 result already exists: {RESULT}")
    tree = ast.parse(AUDITOR.read_text())
    imported = {
        alias.name.split(".")[0]
        for node in ast.walk(tree)
        if isinstance(node, ast.Import)
        for alias in node.names
    }
    imported.update(
        node.module.split(".")[0]
        for node in ast.walk(tree)
        if isinstance(node, ast.ImportFrom) and node.module
    )
    forbidden_modules = {"socket", "urllib", "http", "requests", "paramiko", "ftplib"}
    require(not imported.intersection(forbidden_modules), "network-capable module imported")
    subprocess_calls = [
        node
        for node in ast.walk(tree)
        if isinstance(node, ast.Call)
        and isinstance(node.func, ast.Attribute)
        and isinstance(node.func.value, ast.Name)
        and node.func.value.id == "subprocess"
    ]
    require(len(subprocess_calls) == 1 and subprocess_calls[0].func.attr == "run", "subprocess graph is not one guarded run call")
    forbidden_calls = []
    for node in ast.walk(tree):
        if not isinstance(node, ast.Call) or not isinstance(node.func, ast.Attribute):
            continue
        owner = node.func.value.id if isinstance(node.func.value, ast.Name) else None
        if owner == "os" and (node.func.attr in {"system", "popen"} or node.func.attr.startswith("spawn")):
            forbidden_calls.append(f"os.{node.func.attr}")
    require(not forbidden_calls, f"forbidden external call graph: {forbidden_calls}")
    require(OBJDUMP.resolve().name == "x86_64-linux-gnu-objdump", "objdump resolution drift")
    return {
        "schema": "lay.v10.e1-traversal-w1-machine-cost-decomposition-self-check.v1",
        "task_id": TASK_ID,
        "verdict": "W1_DECOMPOSITION_AUDITOR_VERIFIED_UNRUN",
        "auditor": row(AUDITOR),
        "actions": ["self-check", "audit"],
        "external_executables": [str(OBJDUMP)],
        "planned_objdump_invocations": 4,
        "network_or_remote": 0,
        "perf_or_pmu": 0,
        "cargo_or_rustc": 0,
        "subject_executions": 0,
        "marker_mutations": 0,
        "runtime_authority_changed": False,
    }


def audit() -> dict[str, Any]:
    require(not RESULT.exists(), f"W1 result already exists: {RESULT}")
    check = self_check()
    pinned = verify_pinned()
    manifests = verify_all_manifests()
    closure = predecessor_closure()
    map_value, ranges = verify_map()
    t3 = json_file(D4_T3)
    sample_data = parse_samples(ranges, t3)
    instructions, commands, objdump_output, version_output = disassemble(ranges)
    range_summaries = verify_range_bytes(ranges, instructions)
    detail = decomposition_detail(closure, ranges, range_summaries, sample_data, instructions)

    stage = RESULT.with_name(f".{RESULT.name}.stage-{os.getpid()}-{time.time_ns()}")
    require(not stage.exists(), "W1 stage collision")
    stage.mkdir(mode=0o700, parents=False)
    try:
        minimum = detail["fused_scalar_u64_advance"]["post_recurrence_minimum"]
        receipt = {
            "schema": "lay.v10.e1-traversal-w1-machine-cost-decomposition-audit.v1",
            "task_id": TASK_ID,
            "verdict": "W1_MACHINE_COST_DECOMPOSITION_COMPLETE",
            "d7_w1_traversal_ns_per_edge": closure["d7_w1"]["traversal_ns_per_edge"],
            "d7_w1_cycles_per_edge": closure["d7_w1"]["cycles_per_edge"],
            "d7_w1_instructions_per_edge": closure["d7_w1"]["instructions_per_edge"],
            "d4_accepted_traversal_samples": sample_data["accepted_traversal_samples"],
            "accepted_samples_at_instruction_starts": detail["instruction_start_closure"]["accepted_samples_at_decoded_instruction_start"],
            "instruction_start_match_percent": detail["instruction_start_closure"]["match_percent"],
            "accepted_bucket_counts": sample_data["accepted_bucket_counts"],
            "accepted_sub_bucket_counts": sample_data["accepted_sub_bucket_counts"],
            "fused_scalar_u64_advance_samples": detail["fused_scalar_u64_advance"]["samples"],
            "fused_scalar_u64_advance_percent": detail["fused_scalar_u64_advance"]["share_of_traversal_percent"],
            "post_recurrence_minimum_samples": minimum["samples"],
            "post_recurrence_minimum_percent_of_traversal": minimum["share_of_traversal_percent"],
            "post_recurrence_minimum_percent_of_fused": minimum["share_of_fused_percent"],
            "top_sampled_ip": minimum["top_sampled_ip"]["address_hex"],
            "top_sampled_ip_mnemonic": minimum["top_sampled_ip"]["mnemonic"],
            "top_sampled_ip_samples": minimum["top_sampled_ip"]["samples"],
            "top_sampled_ip_percent": minimum["top_sampled_ip"]["share_of_traversal_percent"],
            "task_clock_is_non_precise": True,
            "single_instruction_latency_claim": False,
            "d4_ip_identity_bound_to_d2_only": True,
            "d7_exact_ip_attribution": False,
            "d2_and_d7_elf_distinct": closure["d2_and_d7_elf_distinct"],
            "optimization_authority": False,
            "next_action_admitted": "FUSED_MINIMUM_MECHANISM_PAPER_ONLY",
            "external_commands": len(commands),
            "external_executables": [str(OBJDUMP)],
            "network_or_remote": 0,
            "perf_or_pmu": 0,
            "cargo_or_rustc": 0,
            "subject_executions": 0,
            "marker_mutations": 0,
            "runtime_authority_changed": False,
        }
        write_json(stage / "W1_MACHINE_COST_DECOMPOSITION_RECEIPT.json", receipt)
        write_json(stage / "DECOMPOSITION_DETAIL.json", detail)
        write_json(
            stage / "INPUT_IDENTITIES.json",
            {"pinned": pinned, "manifests": manifests, "predecessor_closure": closure, "map_identity": row(D2_MAP)},
        )
        write_json(stage / "OBJDUMP_COMMANDS.json", {"commands": commands})
        write_new(stage / "OBJDUMP.stdout", objdump_output)
        write_new(stage / "OBJDUMP_VERSION.stdout", version_output)
        write_json(stage / "SELF_CHECK.json", check)
        copy_input(AUDITOR, stage / "auditor.py")
        copy_input(PAPER, stage / "paper.md")
        copy_input(EVIDENCE_ROUTE, stage / "route-a-evidence.md")
        copy_input(BOUNDARY_ROUTE, stage / "route-b-boundary.md")
        copy_input(STRUCTURAL_REVIEW, stage / "structural-review.json")
        copy_input(PREFLIGHT, stage / "preflight-v1.json")
        copy_input(PREFLIGHT_RECEIPT, stage / "preflight-v1-receipt.json")
        write_sums(stage)
        fsync_dir(stage)
        seal(stage)
        os.rename(stage, RESULT)
        fsync_dir(RESULT.parent)
    except BaseException:
        if stage.exists():
            shutil.rmtree(stage)
        raise
    receipt_path = RESULT / "W1_MACHINE_COST_DECOMPOSITION_RECEIPT.json"
    return {
        **receipt,
        "result": str(RESULT),
        "receipt_sha256": sha256_file(receipt_path),
        "auditor_sha256": sha256_file(AUDITOR),
    }


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("action", choices=("self-check", "audit"))
    arguments = parser.parse_args()
    try:
        value = self_check() if arguments.action == "self-check" else audit()
    except (
        DecompositionError,
        OSError,
        ValueError,
        KeyError,
        TypeError,
        json.JSONDecodeError,
        subprocess.CalledProcessError,
    ) as error:
        print(f"W1 DECOMPOSITION ERROR: {type(error).__name__}: {error}", file=sys.stderr)
        return 1
    print(json.dumps(value, sort_keys=True))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
