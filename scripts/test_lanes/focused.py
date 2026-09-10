"""Run a development-only exact target set without weakening release lanes."""

from __future__ import annotations

import argparse
import os
import pathlib
import subprocess
import sys
import time
from typing import Any, Iterable

from .cli import MANIFEST, artifact_map, source_closure_identity
from .contracts import (
    ContractError,
    file_sha256,
    load_json,
    validate_manifest_shape,
    write_json,
)
from .discovery import DiscoveryError, cargo_discovery, focused_cargo_args
from .execution import ExecutionError, run_target


DISCOVERED_MANIFEST = "DISCOVERED_MANIFEST.json"
SUMMARY = "SUMMARY.json"
SELECTED_LANES = {"correctness", "package"}


class FocusedError(RuntimeError):
    pass


def normalize_targets(values: Iterable[str]) -> list[str]:
    requested = list(values)
    if not requested:
        raise FocusedError("focused execution requires at least one --target")
    if len(requested) != len(set(requested)):
        raise FocusedError("focused targets must not be repeated")
    normalized = sorted(requested)
    focused_cargo_args(set(normalized))
    return normalized


def identity(row: dict[str, str]) -> dict[str, str]:
    return {"target": row["target"], "name": row["name"]}


def canonical_drift(
    discovered: dict[str, Any], canonical: dict[str, Any], targets: set[str]
) -> dict[str, Any]:
    live = {
        (row["target"], row["name"]): row
        for row in discovered["tests"]
        if row["target"] in targets
    }
    expected = {
        (row["target"], row["name"]): row
        for row in canonical["tests"]
        if row["target"] in targets
    }
    added = [identity(live[key]) for key in sorted(live.keys() - expected.keys())]
    removed = [identity(expected[key]) for key in sorted(expected.keys() - live.keys())]
    changed = [
        {
            "target": key[0],
            "name": key[1],
            "canonical": expected[key],
            "discovered": live[key],
        }
        for key in sorted(live.keys() & expected.keys())
        if live[key] != expected[key]
    ]
    return {
        "status": "DRIFT" if added or removed or changed else "MATCH",
        "added": added,
        "removed": removed,
        "changed": changed,
    }


def reserve_results(path: pathlib.Path) -> None:
    if os.path.lexists(path):
        raise FocusedError(f"focused results path already exists: {path}")
    try:
        path.mkdir(parents=True, exist_ok=False)
    except FileExistsError as error:
        raise FocusedError(f"focused results path already exists: {path}") from error


def write_blocked_summary(
    results_dir: pathlib.Path, targets: list[str], category: str, error: BaseException
) -> None:
    write_json(
        results_dir / SUMMARY,
        {
            "schema": "lay.focused-test-run.v1",
            "scope": "development-only-not-release",
            "requested_targets": targets,
            "verdict": "BLOCKED",
            "error_category": category,
            "error": str(error),
            "runtime_authority_changed": False,
        },
    )


def run_focused(
    requested_values: Iterable[str],
    target_dir: pathlib.Path,
    results_dir: pathlib.Path,
) -> dict[str, Any]:
    target_dir = target_dir.resolve()
    results_dir = pathlib.Path(os.path.abspath(results_dir))
    if (
        target_dir == results_dir
        or target_dir in results_dir.parents
        or results_dir in target_dir.parents
    ):
        raise FocusedError("target and focused results paths must not overlap")
    reserve_results(results_dir)
    requested_input = list(requested_values)
    started = time.monotonic()
    try:
        canonical_sha_before = file_sha256(MANIFEST)
        canonical = load_json(MANIFEST)
        validate_manifest_shape(canonical)
        requested = normalize_targets(requested_input)
        requested_set = set(requested)
        source_before = source_closure_identity()

        discovery_started = time.monotonic()
        discovered, artifacts = cargo_discovery(target_dir, requested_set)
        discovery_elapsed = time.monotonic() - discovery_started
        drift = canonical_drift(discovered, canonical, requested_set)
        selected = [
            row for row in discovered["tests"] if row["lane"] in SELECTED_LANES
        ]
        by_target = {
            target: [row for row in discovered["tests"] if row["target"] == target]
            for target in requested
        }
        selected_by_target = {
            target: [row for row in rows if row["lane"] in SELECTED_LANES]
            for target, rows in by_target.items()
        }
        zero_selected_targets = sorted(
            target for target, rows in selected_by_target.items() if not rows
        )
        excluded = {
            lane: [
                identity(row) for row in discovered["tests"] if row["lane"] == lane
            ]
            for lane in ("performance", "ignored")
        }

        discovered_receipt = {
            **discovered,
            "scope": "development-only-not-release",
            "release_receipt": False,
            "requested_targets": requested,
            "selected_lanes": sorted(SELECTED_LANES),
            "selected_tests": [identity(row) for row in selected],
            "selected_by_lane": {
                lane: sum(1 for row in selected if row["lane"] == lane)
                for lane in sorted(SELECTED_LANES)
            },
            "excluded_tests": excluded,
            "canonical_manifest": {
                "path": str(MANIFEST.relative_to(MANIFEST.parents[2])),
                "sha256": canonical_sha_before,
                "drift": drift,
                "mutated": False,
            },
            "timings_seconds": {"discovery": discovery_elapsed},
            "runtime_authority_changed": False,
        }
        write_json(results_dir / DISCOVERED_MANIFEST, discovered_receipt)
        if zero_selected_targets:
            raise FocusedError(
                "focused discovery selected zero correctness/package tests for "
                f"targets={zero_selected_targets}"
            )

        execution_started = time.monotonic()
        artifacts_by_target = artifact_map(artifacts)
        failures: list[dict[str, str]] = []
        elapsed_by_target: dict[str, float] = {}
        for target in requested:
            target_slug = target.replace(":", "-")
            target_failures, elapsed = run_target(
                artifacts_by_target[target],
                selected_by_target[target],
                by_target[target],
                results_dir / "sandboxes" / target_slug,
                results_dir / "logs" / f"{target_slug}.log",
            )
            failures.extend(target_failures)
            elapsed_by_target[target] = elapsed
            print(
                f"focused_test_target={target} "
                f"selected={len(selected_by_target[target])} "
                f"failures={len(target_failures)} elapsed_seconds={elapsed:.3f}",
                flush=True,
            )
        execution_elapsed = time.monotonic() - execution_started
        source_after = source_closure_identity()
        if source_after != source_before:
            raise FocusedError("source closure changed during focused execution")
        canonical_sha_after = file_sha256(MANIFEST)
        if canonical_sha_after != canonical_sha_before:
            raise FocusedError("canonical test manifest changed during focused execution")

        summary = {
            "schema": "lay.focused-test-run.v1",
            "scope": "development-only-not-release",
            "release_receipt": False,
            "requested_targets": requested,
            "selected_lanes": sorted(SELECTED_LANES),
            "discovered": len(discovered["tests"]),
            "discovered_tests": [identity(row) for row in discovered["tests"]],
            "selected": len(selected),
            "selected_tests": [identity(row) for row in selected],
            "selected_by_lane": {
                lane: sum(1 for row in selected if row["lane"] == lane)
                for lane in sorted(SELECTED_LANES)
            },
            "executed": len(selected),
            "executed_tests": [identity(row) for row in selected],
            "excluded": {lane: len(rows) for lane, rows in excluded.items()},
            "excluded_tests": excluded,
            "canonical_manifest": {
                "path": str(MANIFEST.relative_to(MANIFEST.parents[2])),
                "sha256": canonical_sha_before,
                "drift": drift,
                "mutated": False,
            },
            "failures": failures,
            "failed": len(failures),
            "passed": len(selected) - len(failures),
            "elapsed_by_target_seconds": elapsed_by_target,
            "timings_seconds": {
                "discovery": discovery_elapsed,
                "execution": execution_elapsed,
                "total": time.monotonic() - started,
            },
            "verdict": "PASS" if not failures else "FAIL",
            "runtime_authority_changed": False,
        }
        write_json(results_dir / SUMMARY, summary)
        return summary
    except (FocusedError, ContractError) as error:
        write_blocked_summary(results_dir, requested_input, "contract", error)
        raise
    except (DiscoveryError, ExecutionError, OSError, subprocess.SubprocessError) as error:
        write_blocked_summary(results_dir, requested_input, "infrastructure", error)
        raise


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--target", action="append", required=True)
    parser.add_argument("--target-dir", type=pathlib.Path, required=True)
    parser.add_argument("--results-dir", type=pathlib.Path, required=True)
    args = parser.parse_args(argv)
    try:
        summary = run_focused(args.target, args.target_dir, args.results_dir)
    except (
        FocusedError,
        ContractError,
        DiscoveryError,
        ExecutionError,
        OSError,
        subprocess.SubprocessError,
    ) as error:
        print(f"focused test error: {error}", file=sys.stderr)
        return 1
    print(
        f"focused_tests={summary['selected']} failures={summary['failed']} "
        f"verdict={summary['verdict']} scope=development-only-not-release"
    )
    return 0 if summary["verdict"] == "PASS" else 1


if __name__ == "__main__":
    raise SystemExit(main())
