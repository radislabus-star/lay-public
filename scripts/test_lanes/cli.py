"""Command-line orchestration for Lay's test lanes."""

from __future__ import annotations

import argparse
import datetime
import hashlib
import json
import pathlib
import shutil
import subprocess
import sys
import time
from typing import Any

from .contracts import (
    ContractError,
    compare_known_failures,
    compare_manifest,
    file_sha256,
    load_json,
    load_known_failures,
    write_json,
)
from .discovery import DiscoveryError, ROOT, cargo_discovery
from .execution import (
    ExecutionError,
    PerformanceAssertionError,
    run_performance_test,
    run_target,
)


MANIFEST = ROOT / "scripts" / "test-lanes" / "manifest.json"
KNOWN_FAILURES = ROOT / "scripts" / "test-lanes" / "known_failures.json"
DEFAULT_TARGET = ROOT / "target" / "test-lanes"
DEFAULT_RESULTS = ROOT / "target" / "test-lanes-results"


class PerformanceLaneError(RuntimeError):
    pass


SOURCE_ROOTS = (
    ".cargo",
    "Cargo.lock",
    "Cargo.toml",
    "benches",
    "build.rs",
    "data",
    "examples",
    "scripts",
    "src",
    "tests",
)
SOURCE_EXCLUDES = {
    "scripts/test-lanes/known_failures.json",
}


def source_closure_identity() -> dict[str, Any]:
    listed = subprocess.run(
        ["git", "ls-files", "-co", "--exclude-standard", "--", *SOURCE_ROOTS],
        cwd=ROOT,
        check=True,
        capture_output=True,
        text=True,
    ).stdout.splitlines()
    paths = sorted(
        relative
        for relative in set(listed)
        if relative not in SOURCE_EXCLUDES and (ROOT / relative).is_file()
    )
    digest = hashlib.sha256()
    for relative in paths:
        path = ROOT / relative
        data = path.read_bytes()
        digest.update(relative.encode("utf-8"))
        digest.update(b"\0")
        digest.update(str(path.stat().st_mode & 0o777).encode("ascii"))
        digest.update(b"\0")
        digest.update(hashlib.sha256(data).digest())
    head = subprocess.run(
        ["git", "rev-parse", "HEAD"],
        cwd=ROOT,
        check=True,
        capture_output=True,
        text=True,
    ).stdout.strip()
    return {
        "git_head": head,
        "files": len(paths),
        "sha256": digest.hexdigest(),
    }


def artifact_map(artifacts: list[dict[str, str]]) -> dict[str, dict[str, str]]:
    return {row["target"]: row for row in artifacts}


def known_failures_for_lanes(
    known: dict[str, Any], lanes: set[str]
) -> dict[str, Any]:
    return {
        **known,
        "failures": [row for row in known["failures"] if row["lane"] in lanes],
    }


def prepare(args: argparse.Namespace) -> tuple[dict[str, Any], list[dict[str, str]]]:
    current, artifacts = cargo_discovery(args.target_dir)
    expected = load_json(MANIFEST)
    compare_manifest(current, expected)
    print(
        "test_manifest=PASS "
        + " ".join(f"{key}={value}" for key, value in sorted(current["counts"].items()))
        + f" targets={len(current['targets'])} total={len(current['tests'])}"
    )
    return current, artifacts


def run_non_timing(
    args: argparse.Namespace,
    manifest: dict[str, Any],
    artifacts: list[dict[str, str]],
    lanes: set[str],
) -> dict[str, Any]:
    run_started = time.monotonic()
    source_before = source_closure_identity()
    if args.results_dir.exists():
        shutil.rmtree(args.results_dir)
    args.results_dir.mkdir(parents=True)
    by_target: dict[str, list[dict[str, str]]] = {}
    for row in manifest["tests"]:
        by_target.setdefault(row["target"], []).append(row)
    failures = []
    elapsed_by_target = {}
    artifact_by_target = artifact_map(artifacts)
    sandbox_root = args.results_dir / "sandboxes"
    for target in sorted(by_target):
        rows = by_target[target]
        selected = [row for row in rows if row["lane"] in lanes]
        if not selected:
            continue
        target_slug = target.replace(":", "-")
        target_failures, elapsed = run_target(
            artifact_by_target[target],
            selected,
            rows,
            sandbox_root / target_slug,
            args.results_dir / "logs" / f"{target_slug}.log",
        )
        failures.extend(target_failures)
        elapsed_by_target[target] = elapsed
        print(
            f"test_target={target} selected={len(selected)} "
            f"failures={len(target_failures)} elapsed_seconds={elapsed:.3f}",
            flush=True,
        )
    source_after = source_closure_identity()
    if source_after != source_before:
        raise ContractError("source closure changed during hermetic test execution")
    selected_by_lane = {
        lane: sum(1 for row in manifest["tests"] if row["lane"] == lane)
        for lane in sorted(lanes)
    }
    failure_by_identity = {
        (row["target"], row["test"]): row for row in failures
    }
    failures_by_lane = {
        lane: sum(
            1
            for row in manifest["tests"]
            if row["lane"] == lane
            and (row["target"], row["name"]) in failure_by_identity
        )
        for lane in sorted(lanes)
    }
    elapsed_by_lane = {lane: 0.0 for lane in sorted(lanes)}
    for target, elapsed in elapsed_by_target.items():
        target_lanes = {
            row["lane"]
            for row in manifest["tests"]
            if row["target"] == target and row["lane"] in lanes
        }
        if len(target_lanes) != 1:
            raise ContractError(
                f"cannot attribute target elapsed time to one lane: {target} {target_lanes}"
            )
        elapsed_by_lane[next(iter(target_lanes))] += elapsed
    if args.observations_out is not None:
        untested = ["live_desktop_smoke"]
        untested.extend(
            lane
            for lane in ("correctness", "package", "performance", "ignored")
            if lane not in lanes
        )
        write_json(
            args.observations_out,
            {
                "schema": "lay.test-failure-observation.v1",
                "recorded_at_utc": datetime.datetime.now(
                    datetime.timezone.utc
                ).isoformat(),
                "invocation": {
                    "action": args.action,
                    "lanes": sorted(lanes),
                    "target_dir": str(args.target_dir),
                    "results_dir": str(args.results_dir),
                },
                "source_closure": source_before,
                "manifest_sha256": file_sha256(MANIFEST),
                "sandbox": {
                    "filesystem": "host and repository read-only; result sandbox writable",
                    "network": "unshared",
                    "run": "empty tmpfs",
                    "home_xdg": "fresh",
                    "test_threads": 1,
                },
                "lanes": sorted(lanes),
                "selected_by_lane": selected_by_lane,
                "failures_by_lane": failures_by_lane,
                "elapsed_by_lane_seconds": elapsed_by_lane,
                "elapsed_by_target_seconds": elapsed_by_target,
                "failures": failures,
                "untested_scope": untested,
                "runtime_authority_changed": False,
            },
        )
    known = known_failures_for_lanes(
        load_known_failures(KNOWN_FAILURES, MANIFEST), lanes
    )
    compare_known_failures(failures, known)
    summary = {
        "schema": "lay.test-lane-run.v1",
        "lanes": sorted(lanes),
        "selected": sum(
            1 for row in manifest["tests"] if row["lane"] in lanes
        ),
        "selected_by_lane": selected_by_lane,
        "known_semantic_failures": len(failures),
        "known_semantic_failures_by_lane": failures_by_lane,
        "infrastructure_failures": 0,
        "elapsed_by_lane_seconds": elapsed_by_lane,
        "elapsed_by_target_seconds": elapsed_by_target,
        "elapsed_seconds": time.monotonic() - run_started,
        "verdict": "PASS_WITH_EXACT_KNOWN_FAILURES" if failures else "PASS",
    }
    write_json(args.results_dir / "SUMMARY.json", summary)
    print(
        f"test_lanes={','.join(sorted(lanes))} selected={summary['selected']} "
        f"known_semantic_failures={len(failures)} infrastructure_failures=0 "
        f"verdict={summary['verdict']}"
    )
    return summary


def run_performance(
    args: argparse.Namespace,
    manifest: dict[str, Any],
    artifacts: list[dict[str, str]],
) -> None:
    rows = [row for row in manifest["tests"] if row["lane"] == "performance"]
    artifact_by_target = artifact_map(artifacts)
    root = args.results_dir / "performance"
    if root.exists():
        shutil.rmtree(root)
    elapsed = {}
    failures = []
    for row in rows:
        slug = (row["target"] + "-" + row["name"]).replace(":", "-").replace("/", "-")
        identity = f"{row['target']}::{row['name']}"
        try:
            duration = run_performance_test(
                artifact_by_target[row["target"]],
                row,
                root / "sandboxes" / slug,
                root / "logs" / f"{slug}.log",
            )
        except PerformanceAssertionError as error:
            failures.append({"test": identity, "error": str(error)})
            print(f"performance_test={identity} verdict=FAIL", flush=True)
        else:
            elapsed[identity] = duration
            print(
                f"performance_test={identity} elapsed_seconds={duration:.3f} verdict=PASS",
                flush=True,
            )
    verdict = "PASS" if not failures else "BLOCKED_PERFORMANCE"
    write_json(
        root / "SUMMARY.json",
        {
            "schema": "lay.test-performance-run.v1",
            "tests": len(rows),
            "passed": len(rows) - len(failures),
            "failed": len(failures),
            "elapsed_seconds": elapsed,
            "failures": failures,
            "verdict": verdict,
        },
    )
    if failures:
        raise PerformanceLaneError(f"{len(failures)} performance assertions failed")


def write_blocked_summary(
    args: argparse.Namespace, verdict: str, category: str, error: BaseException
) -> None:
    args.results_dir.mkdir(parents=True, exist_ok=True)
    write_json(
        args.results_dir / "SUMMARY.json",
        {
            "schema": "lay.test-lane-run.v1",
            "action": args.action,
            "verdict": verdict,
            "error_category": category,
            "error": str(error),
            "known_semantic_failures": 0,
            "infrastructure_failures": 1 if category == "infrastructure" else 0,
            "contract_failures": 1 if category == "contract" else 0,
            "performance_failures": 1 if category == "performance" else 0,
        },
    )


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument(
        "action",
        choices=("manifest", "write-manifest", "correctness", "package", "all", "performance"),
    )
    parser.add_argument("--target-dir", type=pathlib.Path, default=DEFAULT_TARGET)
    parser.add_argument("--results-dir", type=pathlib.Path, default=DEFAULT_RESULTS)
    parser.add_argument("--observations-out", type=pathlib.Path)
    args = parser.parse_args(argv)
    args.target_dir = args.target_dir.resolve()
    args.results_dir = args.results_dir.resolve()
    try:
        started = time.monotonic()
        if args.action == "write-manifest":
            manifest, _ = cargo_discovery(args.target_dir)
            write_json(MANIFEST, manifest)
            print(f"test_manifest_written={MANIFEST} total={len(manifest['tests'])}")
            return 0
        manifest, artifacts = prepare(args)
        if args.action == "manifest":
            return 0
        if args.action == "performance":
            run_performance(args, manifest, artifacts)
        else:
            lanes = {
                "correctness": {"correctness"},
                "package": {"package"},
                "all": {"correctness", "package"},
            }[args.action]
            run_non_timing(args, manifest, artifacts, lanes)
        print(f"test_lane_action={args.action} elapsed_seconds={time.monotonic() - started:.3f}")
        return 0
    except PerformanceLaneError as error:
        print(f"test lane performance error: {error}", file=sys.stderr)
        return 1
    except ExecutionError as error:
        write_blocked_summary(args, "BLOCKED_INFRASTRUCTURE", "infrastructure", error)
        print(f"test lane infrastructure error: {error}", file=sys.stderr)
        return 1
    except ContractError as error:
        write_blocked_summary(args, "BLOCKED_CONTRACT", "contract", error)
        print(f"test lane contract error: {error}", file=sys.stderr)
        return 1
    except (DiscoveryError, OSError, subprocess.SubprocessError) as error:
        write_blocked_summary(args, "BLOCKED_INFRASTRUCTURE", "infrastructure", error)
        print(f"test lane infrastructure error: {error}", file=sys.stderr)
        return 1


if __name__ == "__main__":
    raise SystemExit(main())
