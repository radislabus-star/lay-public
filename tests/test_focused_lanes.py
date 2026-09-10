#!/usr/bin/env python3
"""Adversarial contracts for development-only focused test lanes."""

from __future__ import annotations

import pathlib
import sys
import tempfile
import unittest
from unittest import mock


ROOT = pathlib.Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / "scripts"))

from test_lanes import contracts, discovery, focused


def row(
    target: str,
    name: str,
    lane: str = "correctness",
    isolation: str = "target",
) -> dict[str, str]:
    return {
        "target": target,
        "name": name,
        "kind": "test",
        "lane": lane,
        "isolation": isolation,
    }


def manifest(rows: list[dict[str, str]], targets: list[str]) -> dict[str, object]:
    counts = {lane: 0 for lane in ("correctness", "package", "performance", "ignored")}
    isolation_counts = {kind: 0 for kind in ("process", "target")}
    for test in rows:
        counts[test["lane"]] += 1
        isolation_counts[test["isolation"]] += 1
    return {
        "schema": discovery.SCHEMA,
        "cargo_args": discovery.CARGO_ARGS,
        "cargo_configuration": {"external": "ABSENT", "project": []},
        "toolchain": {"release": "1", "commit": "c", "host": "h"},
        "counts": counts,
        "isolation_counts": isolation_counts,
        "package_fixtures": [
            {"path": "tests/fixtures/pinned", "size": 1, "sha256": "a" * 64}
        ],
        "targets": [{"target": target} for target in sorted(targets)],
        "tests": sorted(rows, key=discovery.canonical),
    }


class FocusedDiscoveryTests(unittest.TestCase):
    def test_exact_target_flags_never_compile_all_targets(self) -> None:
        args = discovery.focused_cargo_args(
            {"test:typing_contract", "bin:lay-ibus-engine", "lib:lay"}
        )
        self.assertNotIn("--all-targets", args)
        self.assertEqual(1, args.count("--lib"))
        self.assertEqual(
            ["lay-ibus-engine"],
            [args[index + 1] for index, value in enumerate(args) if value == "--bin"],
        )
        self.assertEqual(
            ["typing_contract"],
            [args[index + 1] for index, value in enumerate(args) if value == "--test"],
        )
        with (
            mock.patch.object(
                discovery,
                "cargo_configuration_closure",
                return_value={"external": "ABSENT", "project": []},
            ),
            mock.patch.object(discovery, "toolchain_identity", return_value={}),
            mock.patch.object(discovery, "package_fixture_rows", return_value=[]),
        ):
            default = discovery.manifest_payload([], [])
        self.assertEqual(discovery.CARGO_ARGS, default["cargo_args"])

    def test_invalid_target_identity_is_rejected_before_cargo(self) -> None:
        for targets in (set(), {"doc:lay"}, {"bin:--workspace"}, {"lib:not-lay"}):
            with self.subTest(targets=targets):
                with self.assertRaises(discovery.DiscoveryError):
                    discovery.focused_cargo_args(targets)

    def test_focused_registry_is_projected_but_default_remains_global(self) -> None:
        artifacts = [{"target": "bin:one", "executable": "/tmp/one"}]
        performance = {("bin:one", "timed"), ("bin:two", "other-timed")}
        isolated = {("bin:one", "isolated"), ("bin:two", "other-isolated")}

        def names(_executable: str, *arguments: str) -> dict[str, str]:
            if arguments:
                return {}
            return {"timed": "test", "isolated": "test", "ordinary": "test"}

        with (
            mock.patch.object(discovery, "PERFORMANCE_TESTS", performance),
            mock.patch.object(discovery, "PROCESS_ISOLATED_TESTS", isolated),
            mock.patch.object(discovery, "PACKAGE_TARGETS", set()),
            mock.patch.object(discovery, "_harness_names", side_effect=names),
        ):
            tests, targets = discovery.discover_from_artifacts(artifacts, {"bin:one"})
            self.assertEqual([{"target": "bin:one"}], targets)
            self.assertEqual(3, len(tests))
            with self.assertRaises(discovery.DiscoveryError):
                discovery.discover_from_artifacts(artifacts)

    def test_missing_or_unexpected_compiled_target_is_rejected(self) -> None:
        with self.assertRaisesRegex(discovery.DiscoveryError, "missing"):
            discovery.discover_from_artifacts([], {"bin:wanted"})
        with self.assertRaisesRegex(discovery.DiscoveryError, "unexpected"):
            discovery.discover_from_artifacts(
                [{"target": "bin:other", "executable": "/tmp/other"}],
                {"bin:wanted"},
            )


class FocusedExecutionTests(unittest.TestCase):
    TARGET = "bin:lay-ibus-engine"

    def test_invalid_duplicate_and_zero_requested_targets_are_rejected(self) -> None:
        for values in ([], ["doc:missing"], [self.TARGET, self.TARGET]):
            with self.subTest(values=values):
                with self.assertRaises((focused.FocusedError, discovery.DiscoveryError)):
                    focused.normalize_targets(values)

    def test_new_test_is_allowed_but_reported_as_canonical_drift(self) -> None:
        canonical = manifest([row(self.TARGET, "old")], [self.TARGET])
        live = manifest(
            [row(self.TARGET, "old"), row(self.TARGET, "new")], [self.TARGET]
        )
        drift = focused.canonical_drift(live, canonical, {self.TARGET})
        self.assertEqual("DRIFT", drift["status"])
        self.assertEqual(
            [{"target": self.TARGET, "name": "new"}], drift["added"]
        )
        self.assertEqual([], drift["removed"])

    def test_new_target_is_allowed_but_reported_as_canonical_drift(self) -> None:
        new_target = "test:new_contract"
        canonical = manifest([row(self.TARGET, "old")], [self.TARGET])
        live = manifest([row(new_target, "new")], [new_target])
        self.assertEqual([new_target], focused.normalize_targets([new_target]))
        drift = focused.canonical_drift(live, canonical, {new_target})
        self.assertEqual("DRIFT", drift["status"])
        self.assertEqual(
            [{"target": new_target, "name": "new"}], drift["added"]
        )

    def run_mocked(
        self,
        directory: str,
        live_rows: list[dict[str, str]],
        failures: list[dict[str, str]] | None = None,
    ) -> tuple[dict[str, object], pathlib.Path, bytes, mock.Mock]:
        root = pathlib.Path(directory)
        canonical_path = root / "scripts" / "test-lanes" / "manifest.json"
        canonical_path.parent.mkdir(parents=True)
        canonical = manifest([row(self.TARGET, "old")], [self.TARGET])
        canonical_path.write_text(contracts.canonical_json(canonical), encoding="utf-8")
        before = canonical_path.read_bytes()
        live = manifest(live_rows, [self.TARGET])
        artifact = {"target": self.TARGET, "executable": "/tmp/ibus-tests"}
        results = root / "fresh-results"
        target_dir = root / "target"
        run = mock.Mock(return_value=(failures or [], 0.25))
        with (
            mock.patch.object(focused, "MANIFEST", canonical_path),
            mock.patch.object(
                focused, "cargo_discovery", return_value=(live, [artifact])
            ) as cargo,
            mock.patch.object(
                focused, "source_closure_identity", return_value={"sha256": "same"}
            ),
            mock.patch.object(focused, "run_target", run),
        ):
            summary = focused.run_focused([self.TARGET], target_dir, results)
        cargo.assert_called_once_with(target_dir.resolve(), {self.TARGET})
        return summary, results, before, run

    def test_target_filtering_exclusions_and_canonical_immutability(self) -> None:
        live_rows = [
            row(self.TARGET, "old"),
            row(self.TARGET, "new"),
            row(self.TARGET, "isolated", isolation="process"),
            row(self.TARGET, "timed", "performance", "process"),
            row(self.TARGET, "ignored", "ignored"),
        ]
        with tempfile.TemporaryDirectory() as directory:
            summary, results, before, run = self.run_mocked(directory, live_rows)
            selected = run.call_args.args[1]
            self.assertEqual(
                ["isolated", "new", "old"],
                sorted(test["name"] for test in selected),
            )
            self.assertEqual(
                "process",
                next(test for test in selected if test["name"] == "isolated")[
                    "isolation"
                ],
            )
            self.assertEqual("PASS", summary["verdict"])
            self.assertEqual(5, summary["discovered"])
            self.assertEqual(3, summary["executed"])
            self.assertEqual({"ignored": 1, "performance": 1}, summary["excluded"])
            receipt = contracts.load_json(results / focused.DISCOVERED_MANIFEST)
            self.assertEqual("DRIFT", receipt["canonical_manifest"]["drift"]["status"])
            self.assertEqual("development-only-not-release", receipt["scope"])
            self.assertFalse(receipt["release_receipt"])
            canonical = pathlib.Path(directory) / "scripts/test-lanes/manifest.json"
            self.assertEqual(before, canonical.read_bytes())

    def test_any_selected_failure_propagates_to_exit_status(self) -> None:
        failure = {"target": self.TARGET, "test": "old", "failure_block": "boom"}
        with tempfile.TemporaryDirectory() as directory:
            summary, results, _before, _run = self.run_mocked(
                directory, [row(self.TARGET, "old")], [failure]
            )
            self.assertEqual("FAIL", summary["verdict"])
            self.assertEqual([failure], summary["failures"])
            with mock.patch.object(focused, "run_focused", return_value=summary):
                self.assertEqual(
                    1,
                    focused.main(
                        [
                            "--target",
                            self.TARGET,
                            "--target-dir",
                            str(pathlib.Path(directory) / "another-target"),
                            "--results-dir",
                            str(pathlib.Path(directory) / "another-results"),
                        ]
                    ),
                )
            recorded = contracts.load_json(results / focused.SUMMARY)
            self.assertEqual("FAIL", recorded["verdict"])

    def test_zero_selected_tests_on_any_requested_target_is_blocked(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = pathlib.Path(directory)
            canonical_path = root / "canonical.json"
            other = "test:timing_only"
            canonical = manifest([row(self.TARGET, "ordinary")], [self.TARGET])
            live = manifest(
                [
                    row(self.TARGET, "ordinary"),
                    row(other, "timed", "performance", "process"),
                ],
                [self.TARGET, other],
            )
            canonical_path.write_text(
                contracts.canonical_json(canonical), encoding="utf-8"
            )
            results = root / "results"
            with (
                mock.patch.object(focused, "MANIFEST", canonical_path),
                mock.patch.object(
                    focused,
                    "cargo_discovery",
                    return_value=(
                        live,
                        [
                            {"target": self.TARGET, "executable": "/tmp/primary"},
                            {"target": other, "executable": "/tmp/timing"},
                        ],
                    ),
                ),
                mock.patch.object(
                    focused,
                    "source_closure_identity",
                    return_value={"sha256": "same"},
                ),
                mock.patch.object(focused, "run_target") as run,
            ):
                with self.assertRaisesRegex(focused.FocusedError, "zero"):
                    focused.run_focused(
                        [self.TARGET, other], root / "target", results
                    )
            run.assert_not_called()
            self.assertTrue((results / focused.DISCOVERED_MANIFEST).is_file())
            recorded = contracts.load_json(results / focused.SUMMARY)
            self.assertEqual("BLOCKED", recorded["verdict"])

    def test_existing_results_path_is_rejected_not_replaced(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            results = pathlib.Path(directory) / "results"
            results.mkdir()
            sentinel = results / "keep"
            sentinel.write_text("owned", encoding="utf-8")
            with self.assertRaisesRegex(focused.FocusedError, "already exists"):
                focused.run_focused(
                    [self.TARGET], pathlib.Path(directory) / "target", results
                )
            self.assertEqual("owned", sentinel.read_text(encoding="utf-8"))

    def test_public_shell_route_dispatches_the_focused_module(self) -> None:
        wrapper = (ROOT / "scripts/check-lay-tests.sh").read_text(encoding="utf-8")
        self.assertIn("all|performance|focused)", wrapper)
        self.assertIn("-m test_lanes.focused", wrapper)


if __name__ == "__main__":
    unittest.main()
