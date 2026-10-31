#!/usr/bin/env python3
"""Named component checks using the canonical discovery and hermetic executor."""
from __future__ import annotations

import argparse
import json
import os
from pathlib import Path
import subprocess
import sys
import time

from test_lanes.cli import artifact_map, source_closure_identity
from test_lanes.contracts import ContractError, file_sha256, load_json, write_json
from test_lanes.discovery import DiscoveryError, cargo_discovery
from test_lanes.execution import ExecutionError, run_target
from test_lanes.focused import FocusedError, canonical_drift, reserve_results

ROOT = Path(__file__).resolve().parents[1]
MANIFEST = ROOT / "scripts/test-lanes/manifest.json"
ENGINE, DAEMON = "bin:lay-ibus-engine", "bin:lay-daemon"
# These are source test closures, not new runtime routes or release gates.
RECIPES = {
    "kde-chrome": ((ENGINE, "window_interaction::native_document::"),
                   (ENGINE, "kde::chrome::tests::"), (ENGINE, "display::tests::")),
    "kde-terminal": ((ENGINE, "terminal"), (ENGINE, "display::tests::")),
    "kde-focus": ((DAEMON, "focus_guard::"), (DAEMON, "tests::layout_backend::"),
                  (DAEMON, "kde_focus::tests::"), (ENGINE, "kde_focus::tests::"),
                  (DAEMON, "layout_controller::kde_exact::tests::")),
    "shared-replay": ((DAEMON, "layout_controller::ime_bridge::"),),
    "gnome": ((ENGINE, "display::tests::"),
              (DAEMON, "layout_controller::gnome_dbus::tests::"),
              (DAEMON, "focused_window_identity_prefers_gnome_stable_sequence")),
}


def select_rows(manifest: dict, component: str) -> list[dict]:
    if component not in RECIPES:
        raise ValueError("unknown component: " + component)
    selected = {}
    for target, fragment in RECIPES[component]:
        rows = [row for row in manifest["tests"] if row["target"] == target
                and row["lane"] in {"correctness", "package"} and fragment in row["name"]]
        if not rows:
            raise ValueError("empty component selector: " + target + " / " + fragment)
        selected.update({(row["target"], row["name"]): row for row in rows})
    return [selected[key] for key in sorted(selected)]


def run(component: str, target_dir: Path, results: Path) -> dict:
    if os.environ.get("LAY_RESOURCE_GUARD_ACTIVE") != "1":
        raise ValueError("use scripts/check-lay-component.sh under the resource guard")
    target_dir, results = target_dir.resolve(), results.absolute()
    if target_dir == results or target_dir in results.parents or results in target_dir.parents:
        raise ValueError("target and results paths must not overlap")
    reserve_results(results)
    started = time.monotonic()
    source = source_closure_identity()
    manifest_sha = file_sha256(MANIFEST)
    canonical = load_json(MANIFEST)
    expected = select_rows(canonical, component)
    targets = {row["target"] for row in expected}
    discovered, artifacts = cargo_discovery(target_dir, targets)
    drift = canonical_drift(discovered, canonical, targets)
    if drift["status"] != "MATCH":
        raise ValueError("component discovery differs from canonical manifest: " + repr(drift))
    selected = select_rows(discovered, component)
    if selected != expected:
        raise ValueError("component test identities changed during discovery")
    by_target = artifact_map(artifacts)
    failures = []
    for target in sorted(targets):
        slug = target.replace(":", "-")
        failed, _ = run_target(by_target[target],
                               [row for row in selected if row["target"] == target],
                               [row for row in discovered["tests"] if row["target"] == target],
                               results / "sandboxes" / slug, results / "logs" / (slug + ".log"))
        failures.extend(failed)
    if source != source_closure_identity() or manifest_sha != file_sha256(MANIFEST):
        raise ValueError("component source/manifest changed during execution")
    summary = {"schema": "lay.component-check.v1", "component": component,
               "scope": "development-only-not-release-or-native-acceptance", "source": source,
               "selected_tests": selected, "selected": len(selected), "executed": len(selected),
               "passed": len(selected) - len(failures), "failed": len(failures),
               "failures": failures, "elapsed_seconds": time.monotonic() - started,
               "verdict": "PASS" if not failures else "FAIL"}
    write_json(results / "SUMMARY.json", summary)
    return summary


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("component", choices=sorted(RECIPES))
    parser.add_argument("--list", action="store_true", help="show exact selected identities; do not run")
    parser.add_argument("--target-dir", type=Path, default=Path(os.environ.get("CARGO_TARGET_DIR", ROOT / "target/component")))
    parser.add_argument("--results-dir", type=Path)
    args = parser.parse_args()
    try:
        if args.list:
            rows = select_rows(load_json(MANIFEST), args.component)
            print(json.dumps({"status": "SELECTION_ONLY_NOT_TEST_PROOF", "selected": len(rows), "tests": rows}))
            return 0
        if args.results_dir is None:
            raise ValueError("--results-dir must name a fresh evidence directory")
        summary = run(args.component, args.target_dir, args.results_dir)
        print(json.dumps({key: summary[key] for key in ("component", "verdict", "selected", "executed", "passed", "failed")}))
        return 0 if summary["verdict"] == "PASS" else 1
    except (ValueError, OSError, ContractError, DiscoveryError, ExecutionError,
            FocusedError, subprocess.SubprocessError) as error:
        print("component check: " + str(error), file=sys.stderr)
        return 1


if __name__ == "__main__":
    raise SystemExit(main())
