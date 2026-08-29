#!/usr/bin/env python3
"""Adversarial tests for Lay's test-lane contracts."""

from __future__ import annotations

import argparse
import json
import pathlib
import subprocess
import sys
import tempfile
import unittest


ROOT = pathlib.Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / "scripts"))

from test_lanes import cli, contracts, discovery, execution


class DiscoveryTests(unittest.TestCase):
    def test_cargo_stream_requires_terminal_success(self) -> None:
        artifact = {
            "reason": "compiler-artifact",
            "target": {"kind": ["lib"], "name": "lay"},
            "profile": {"test": True},
            "executable": "/tmp/lay-test",
        }
        with tempfile.TemporaryDirectory() as directory:
            path = pathlib.Path(directory) / "cargo.jsonl"
            path.write_text(
                json.dumps(artifact)
                + "\n"
                + json.dumps({"reason": "build-finished", "success": True})
                + "\n",
                encoding="utf-8",
            )
            self.assertEqual("lib:lay", discovery.parse_cargo_artifacts(path)[0]["target"])
            for payload in (
                "",
                json.dumps(artifact) + "\n",
                json.dumps({"reason": "build-finished", "success": False}) + "\n",
                json.dumps({"reason": "build-finished", "success": True})
                + "\n"
                + json.dumps(artifact)
                + "\n",
            ):
                path.write_text(payload, encoding="utf-8")
                with self.assertRaises(discovery.DiscoveryError):
                    discovery.parse_cargo_artifacts(path)

    def test_lane_classification_is_disjoint(self) -> None:
        perf = next(iter(discovery.PERFORMANCE_TESTS))
        self.assertEqual("performance", discovery.classify(*perf, "test", False))
        self.assertEqual("ignored", discovery.classify(*perf, "test", True))
        package_target = next(iter(discovery.PACKAGE_TARGETS))
        self.assertEqual(
            "package", discovery.classify(package_target, "case", "test", False)
        )
        self.assertEqual(
            "correctness", discovery.classify("test:contract", "case", "test", False)
        )
        self.assertEqual("correctness", discovery.classify("lib:lay", "case", "test", False))
        isolated = next(iter(discovery.PROCESS_ISOLATED_TESTS))
        self.assertEqual("process", discovery.isolation(*isolated, "correctness"))
        self.assertEqual("target", discovery.isolation("lib:lay", "case", "correctness"))


class ContractTests(unittest.TestCase):
    def manifest(self, rows: list[dict[str, str]]) -> dict[str, object]:
        counts = {lane: 0 for lane in ("correctness", "package", "performance", "ignored")}
        isolation_counts = {kind: 0 for kind in ("process", "target")}
        for row in rows:
            counts[row["lane"]] += 1
            isolation_counts[row["isolation"]] += 1
        return {
            "schema": discovery.SCHEMA,
            "cargo_args": discovery.CARGO_ARGS,
            "cargo_configuration": {"external": "ABSENT", "project": []},
            "toolchain": {"release": "1", "commit": "c", "host": "h"},
            "counts": counts,
            "isolation_counts": isolation_counts,
            "package_fixtures": [
                {"path": "tests/fixtures/pinned.txt", "size": 1, "sha256": "a" * 64}
            ],
            "targets": [{"target": "lib:lay"}],
            "tests": sorted(rows, key=discovery.canonical),
        }

    def test_manifest_rejects_add_remove_rename_and_lane_drift(self) -> None:
        row = {
            "target": "lib:lay",
            "name": "old",
            "kind": "test",
            "lane": "correctness",
            "isolation": "target",
        }
        baseline = self.manifest([row])
        contracts.compare_manifest(baseline, baseline)
        variants = [
            self.manifest([row, {**row, "name": "new"}]),
            self.manifest([]),
            self.manifest([{**row, "name": "renamed"}]),
            self.manifest([{**row, "lane": "performance"}]),
            self.manifest([{**row, "isolation": "process"}]),
        ]
        for variant in variants:
            with self.assertRaises(contracts.ContractError):
                contracts.compare_manifest(variant, baseline)

    def test_known_failures_reject_new_fixed_and_signature_drift(self) -> None:
        known_row = {
            "target": "lib:lay",
            "test": "case",
            "lane": "correctness",
            "cluster": "remaining_semantic",
            "owner": "TD-007",
            "signature_sha256": contracts.failure_signature(
                "prefix expected marker suffix"
            ),
            "signature_excerpt": "prefix expected marker suffix",
        }
        known = {"failures": [known_row]}
        observed = [
            {
                "target": "lib:lay",
                "test": "case",
                "failure_block": "prefix expected marker suffix",
            }
        ]
        contracts.compare_known_failures(observed, known)
        for changed in (
            [],
            [{**observed[0], "test": "new"}],
            [{**observed[0], "failure_block": "different"}],
        ):
            with self.assertRaises(contracts.ContractError):
                contracts.compare_known_failures(changed, known)

    def test_known_failure_manifest_rejects_count_and_observation_drift(self) -> None:
        row = {
            "target": "lib:lay",
            "test": "case",
            "lane": "correctness",
            "cluster": "remaining_semantic",
            "owner": "TD-007",
            "signature_sha256": contracts.failure_signature("failure"),
            "signature_excerpt": "failure",
        }
        manifest = self.manifest(
            [
                {
                    "target": "lib:lay",
                    "name": "case",
                    "kind": "test",
                    "lane": "correctness",
                    "isolation": "target",
                }
            ]
        )
        with tempfile.TemporaryDirectory() as directory:
            root = pathlib.Path(directory) / "repo"
            manifest_path = root / "scripts" / "test-lanes" / "manifest.json"
            known_path = root / "scripts" / "test-lanes" / "known.json"
            observation_path = root / "evidence" / "observation.json"
            manifest_path.parent.mkdir(parents=True)
            observation_path.parent.mkdir(parents=True)
            manifest_path.write_text(contracts.canonical_json(manifest), encoding="utf-8")
            observation_path.write_text(
                contracts.canonical_json(
                    {
                        "schema": "lay.test-failure-observation.v1",
                        "lanes": ["correctness"],
                        "failures": [
                            {
                                "target": "lib:lay",
                                "test": "case",
                                "failure_block": "failure",
                            }
                        ],
                    }
                ),
                encoding="utf-8",
            )
            base = {
                "schema": contracts.KNOWN_SCHEMA,
                "manifest_sha256": contracts.file_sha256(manifest_path),
                "observation_path": "evidence/observation.json",
                "observation_sha256": contracts.file_sha256(observation_path),
                "failure_count": 1,
                "failures": [row],
            }
            known_path.write_text(contracts.canonical_json(base), encoding="utf-8")
            contracts.load_known_failures(known_path, manifest_path)
            for changed in (
                {**base, "failure_count": 0},
                {**base, "observation_sha256": "invalid"},
            ):
                known_path.write_text(contracts.canonical_json(changed), encoding="utf-8")
                with self.assertRaises(contracts.ContractError):
                    contracts.load_known_failures(known_path, manifest_path)

    def test_failure_signature_ignores_harness_location_but_not_outcome(self) -> None:
        first = "\n".join(
            (
                "thread 'case' (3) panicked at src/lib.rs:10:2:",
                "assertion `left == right` failed",
                "  left: 1",
                " right: 2",
                "note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace",
            )
        )
        moved = first.replace("src/lib.rs:10:2", "src/lib.rs:999:7")
        changed = first.replace("right: 2", "right: 3")
        self.assertEqual(
            contracts.failure_signature(first), contracts.failure_signature(moved)
        )
        self.assertNotEqual(
            contracts.failure_signature(first), contracts.failure_signature(changed)
        )

    def test_failure_signature_ignores_event_identity_but_not_payload(self) -> None:
        first = (
            '{"ts":100,"episode_id":"2-100-1",'
            '"kind":"confirmed_ime_prediction","word":"ну"}'
        )
        repeated = first.replace("100", "999").replace("2-999-1", "7-999-4")
        changed = first.replace('"word":"ну"', '"word":"да"')
        self.assertEqual(
            contracts.failure_signature(first), contracts.failure_signature(repeated)
        )
        self.assertNotEqual(
            contracts.failure_signature(first), contracts.failure_signature(changed)
        )

    def test_known_failure_comparison_is_lane_local(self) -> None:
        known = {
            "failures": [
                {"lane": "correctness", "target": "lib:lay", "test": "unit"},
                {"lane": "package", "target": "test:contract", "test": "package"},
            ]
        }
        selected = cli.known_failures_for_lanes(known, {"correctness"})
        self.assertEqual(
            [("lib:lay", "unit")],
            [(row["target"], row["test"]) for row in selected["failures"]],
        )


class ExecutionTests(unittest.TestCase):
    def test_process_isolated_rows_are_partitioned_from_target_batch(self) -> None:
        rows = [
            {"name": "bulk", "isolation": "target"},
            {"name": "isolated", "isolation": "process"},
        ]
        bulk, isolated = execution.partition_selected(rows)
        self.assertEqual(["bulk"], [row["name"] for row in bulk])
        self.assertEqual(["isolated"], [row["name"] for row in isolated])

    def test_status_and_failure_block_parsing(self) -> None:
        output = "\n".join(
            (
                "running 2 tests",
                "test one ... ok",
                "test two ... FAILED",
                "",
                "---- two stdout ----",
                "thread 'two' panicked at expected marker",
                "",
                "failures:",
                "    two",
            )
        )
        self.assertEqual({"one": "ok", "two": "FAILED"}, execution.parse_statuses(output))
        self.assertIn("expected marker", execution.failure_block(output, "two"))

    def test_nocapture_output_cannot_hide_one_test_success_or_failure(self) -> None:
        success = "\n".join(
            (
                "running 1 test",
                "test timed_case ... timing p99=42us",
                "ok",
                "",
                "test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 3 filtered out",
            )
        )
        failure = success.replace(
            "test result: ok. 1 passed; 0 failed; 0 ignored;",
            "test result: FAILED. 0 passed; 1 failed; 0 ignored;",
        )
        self.assertTrue(execution.performance_test_succeeded(success, "timed_case"))
        self.assertFalse(execution.performance_test_succeeded(failure, "timed_case"))

    def test_environment_removes_live_authority(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            env = execution.clean_environment(
                pathlib.Path(directory),
                {
                    "HOME": "/real",
                    "LAY_CONFIG_PATH": "/live/config",
                    "DBUS_SESSION_BUS_ADDRESS": "unix:path=/live",
                    "DISPLAY": ":0",
                    "NANDA_PACKAGE": "/live/package",
                    "LD_PRELOAD": "/live/inject.so",
                    "PATH": "/live/bin",
                },
            )
            self.assertEqual(
                "/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin",
                env["PATH"],
            )
            self.assertNotEqual("/real", env["HOME"])
            self.assertNotEqual("/live/config", env["LAY_CONFIG_PATH"])
            self.assertNotIn("DBUS_SESSION_BUS_ADDRESS", env)
            self.assertNotIn("DISPLAY", env)
            self.assertNotIn("NANDA_PACKAGE", env)
            self.assertNotIn("LD_PRELOAD", env)

    def test_bubblewrap_masks_seeded_real_lay_state_and_network(self) -> None:
        if not pathlib.Path("/usr/bin/bwrap").is_file():
            self.skipTest("bubblewrap unavailable")
        with tempfile.TemporaryDirectory(dir=ROOT / "target") as directory:
            sandbox = pathlib.Path(directory) / "sandbox"
            real_home = pathlib.Path(directory) / "real-home"
            live = real_home / ".local" / "share" / "lay"
            live.mkdir(parents=True)
            (live / "poison").write_text("must stay hidden", encoding="utf-8")
            sandbox.mkdir()
            command = execution.sandbox_command(
                [
                    "/bin/sh",
                    "-c",
                    (
                        f"test ! -e {live / 'poison'}"
                        " && test ! -e /run/dbus/system_bus_socket"
                        " && test ! -e /run/user/$(id -u)/bus"
                        " && test ! -e /run/user/$(id -u)/wayland-0"
                    ),
                ],
                sandbox,
                real_home=real_home,
            )
            completed = subprocess.run(
                command,
                env=execution.clean_environment(sandbox, {"PATH": "/usr/bin"}),
                check=False,
            )
            self.assertEqual(0, completed.returncode)
            self.assertIn("--unshare-net", command)

    def test_cargo_discovery_is_networkless_with_only_target_writable(self) -> None:
        if not pathlib.Path("/usr/bin/bwrap").is_file():
            self.skipTest("bubblewrap unavailable")
        with tempfile.TemporaryDirectory(dir=ROOT / "target") as directory:
            target = pathlib.Path(directory)
            command = discovery.cargo_sandbox_command(["/bin/true"], target)
            self.assertIn("--unshare-net", command)
            run_mount = command.index("/run")
            self.assertEqual("--tmpfs", command[run_mount - 1])
            bind = command.index("--bind")
            self.assertEqual([str(target), str(target)], command[bind + 1 : bind + 3])
            self.assertEqual(0, subprocess.run(command, check=False).returncode)

    def test_external_cargo_configuration_is_rejected(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = pathlib.Path(directory) / "home" / "projects" / "lay"
            cargo_home = pathlib.Path(directory) / "home" / ".cargo"
            root.mkdir(parents=True)
            cargo_home.mkdir(parents=True)
            self.assertEqual(
                {"external": "ABSENT", "project": []},
                discovery.cargo_configuration_closure(root, cargo_home),
            )
            (cargo_home / "config.toml").write_text(
                "[build]\nrustflags = ['--cfg', 'poison']\n", encoding="utf-8"
            )
            with self.assertRaises(discovery.DiscoveryError):
                discovery.cargo_configuration_closure(root, cargo_home)


class CliTests(unittest.TestCase):
    def test_blocked_summary_separates_infrastructure_and_contract(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            args = argparse.Namespace(
                action="all", results_dir=pathlib.Path(directory)
            )
            cli.write_blocked_summary(
                args, "BLOCKED_INFRASTRUCTURE", "infrastructure", RuntimeError("boom")
            )
            summary = json.loads(
                (pathlib.Path(directory) / "SUMMARY.json").read_text(encoding="utf-8")
            )
            self.assertEqual(1, summary["infrastructure_failures"])
            self.assertEqual(0, summary["contract_failures"])
            cli.write_blocked_summary(
                args, "BLOCKED_CONTRACT", "contract", RuntimeError("drift")
            )
            summary = json.loads(
                (pathlib.Path(directory) / "SUMMARY.json").read_text(encoding="utf-8")
            )
            self.assertEqual(0, summary["infrastructure_failures"])
            self.assertEqual(1, summary["contract_failures"])


if __name__ == "__main__":
    unittest.main(verbosity=2)
