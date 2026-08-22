from __future__ import annotations

import importlib.util
import hashlib
import json
import tempfile
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
SCRIPT = ROOT / "scripts" / "analyze_ime_immediate_space_trace.py"
SPEC = importlib.util.spec_from_file_location("immediate_space_trace", SCRIPT)
assert SPEC is not None and SPEC.loader is not None
MODULE = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(MODULE)

IME_SCRIPT = ROOT / "scripts" / "runtime_smoke" / "ime.py"
IME_SPEC = importlib.util.spec_from_file_location("runtime_smoke_ime", IME_SCRIPT)
assert IME_SPEC is not None and IME_SPEC.loader is not None
IME_MODULE = importlib.util.module_from_spec(IME_SPEC)
IME_SPEC.loader.exec_module(IME_MODULE)


def route(path: str, epoch: int, projection: str, producers: int) -> dict[str, object]:
    return {
        "kind": "ibus_token_field_route",
        "projection": projection,
        "outcome": "prepared" if projection == "correction" else "applied",
        "worker_generation": epoch,
        "tail_epoch": epoch,
        "engine_path": path,
        "field_producer_count": producers,
        "field_cache_disposition": "producer" if producers else "ready_hit",
        "field_generation": 9,
        "l11_us": 100,
        "productive_v90_us": 200,
        "display_l3_us": 30 if projection == "display" else 0,
        "semantic_l3_us": 10 if projection == "display" else 0,
        "correction_l3_us": 40 if projection == "correction" else 0,
        "space_lookup_wait_us": 0,
        "decision_total_us": 50,
        "correction_total_us": 400,
        "candidates": 2 if projection == "display" else 0,
    }


def lease(path: str, epoch: int, outcome: str = "applied") -> dict[str, object]:
    return {
        "kind": "ibus_space_correction_lease",
        "outcome": outcome,
        "worker_generation": epoch,
        "tail_epoch": epoch,
        "engine_path": path,
        "space_lookup_wait_us": 200,
    }


def timing(kind: str, total_us: int) -> dict[str, object]:
    if kind == "space":
        return {
            "kind": "ibus_space_key_timing",
            "route": "managed_autocorrect",
            "setup_us": 1,
            "autocorrect_us": total_us - 2,
            "commit_us": 1,
            "total_us": total_us,
        }
    return {
        "kind": "ibus_printable_key_timing",
        "route": "managed_commit",
        "total_us": total_us,
    }


class ImmediateSpaceTraceTest(unittest.TestCase):
    def setUp(self) -> None:
        self.temp = tempfile.TemporaryDirectory(prefix="lay-ime-trace-test-")
        self.root = Path(self.temp.name)
        self.manifest = self.root / "manifest.json"
        self.trace = self.root / "trace.jsonl"
        self.harness = self.root / "harness.json"
        self.expected = {"warmup": "warm output", "eligible": "eligible output"}
        self.manifest.write_text(
            json.dumps(
                {
                    "schema": "lay.ime-immediate-space-replay.v2",
                    "warmup_case": "warmup",
                    "warmup_space_events": 1,
                    "eligible_case": "eligible",
                    "eligible_space_events": 2,
                    "eligible_applied_space_events": 2,
                    "expected_text_sha256": {
                        name: hashlib.sha256(value.encode("utf-8")).hexdigest()
                        for name, value in self.expected.items()
                    },
                    "required_projections": ["display", "correction"],
                    "maximum_field_producers_per_frame": 1,
                    "space_p99_us_max": 10000,
                    "space_max_us_max": 20000,
                    "printable_p99_us_max": 5000,
                    "printable_max_us_max": 20000,
                }
            ),
            encoding="utf-8",
        )
        self.write_harness()

    def tearDown(self) -> None:
        self.temp.cleanup()

    def records(self) -> list[dict[str, object]]:
        rows: list[dict[str, object]] = []
        for epoch in range(1, 4):
            path = "/warmup" if epoch == 1 else "/eligible"
            rows.extend(
                [
                    timing("printable", 80 + epoch),
                    route(path, epoch, "display", 1),
                    route(path, epoch, "correction", 0),
                    lease(path, epoch),
                    timing("space", 900 + epoch),
                    {
                        "kind": "ibus_key",
                        "stage": "space_managed_autocorrect",
                        "keyval": 32,
                    },
                ]
            )
        return rows

    def write_records(self, rows: list[dict[str, object]]) -> None:
        self.trace.write_text(
            "".join(json.dumps(row, ensure_ascii=False) + "\n" for row in rows),
            encoding="utf-8",
        )

    def write_harness(self, *, eligible_got: str | None = None) -> None:
        self.harness.write_text(
            json.dumps(
                {
                    "schema": "lay.runtime-smoke-receipt.v1",
                    "all_passed": eligible_got is None,
                    "cases": [
                        {
                            "name": name,
                            "ok": eligible_got is None or name != "eligible",
                            "got": eligible_got if name == "eligible" and eligible_got is not None else value,
                            "expected": value,
                        }
                        for name, value in self.expected.items()
                    ],
                }
            ),
            encoding="utf-8",
        )

    def test_complete_trace_passes(self) -> None:
        self.write_records(self.records())
        receipt = MODULE.analyze(self.trace, self.manifest, self.harness)
        self.assertEqual(receipt["verdict"], "PASS")
        self.assertEqual(receipt["denominators"]["eligible_space_events"], 2)
        self.assertTrue(all(receipt["gates"].values()))

    def test_duplicate_projection_fails_closed(self) -> None:
        rows = self.records()
        rows.insert(9, route("/eligible", 2, "correction", 0))
        self.write_records(rows)
        receipt = MODULE.analyze(self.trace, self.manifest, self.harness)
        self.assertEqual(receipt["verdict"], "FAIL")
        self.assertEqual(receipt["projection_receipt"]["failed_frames"], 1)

    def test_eligible_not_ready_is_a_failure(self) -> None:
        rows = self.records()
        next(
            row
            for row in rows
            if row.get("kind") == "ibus_space_correction_lease"
            and row.get("tail_epoch") == 2
        )["outcome"] = "not_ready"
        self.write_records(rows)
        receipt = MODULE.analyze(self.trace, self.manifest, self.harness)
        self.assertEqual(receipt["verdict"], "FAIL")
        self.assertFalse(receipt["gates"]["eligible_not_ready_zero"])

    def test_ready_without_applied_edit_fails_closed(self) -> None:
        rows = self.records()
        next(
            row
            for row in rows
            if row.get("kind") == "ibus_space_correction_lease"
            and row.get("tail_epoch") == 2
        )["outcome"] = "ready"
        self.write_records(rows)
        receipt = MODULE.analyze(self.trace, self.manifest, self.harness)
        self.assertEqual(receipt["verdict"], "FAIL")
        self.assertFalse(receipt["gates"]["eligible_applied_exact"])

    def test_wrong_captured_output_fails_closed(self) -> None:
        self.write_records(self.records())
        self.write_harness(eligible_got="wrong output")
        receipt = MODULE.analyze(self.trace, self.manifest, self.harness)
        self.assertEqual(receipt["verdict"], "FAIL")
        self.assertFalse(receipt["gates"]["harness_output_parity"])

    def test_malformed_json_is_rejected(self) -> None:
        self.trace.write_text('{"kind":"ibus_key"}\nnot-json\n', encoding="utf-8")
        with self.assertRaises(MODULE.AnalysisError):
            MODULE.analyze(self.trace, self.manifest, self.harness)

    def test_type_space_has_no_post_letter_sleep(self) -> None:
        typing_source = (
            ROOT / "src/bin/lay_test_input/scenarios/typing.rs"
        ).read_text(encoding="utf-8")
        helper = typing_source.split("pub(super) fn type_physical_before_boundary", 1)[1]
        helper = helper.split("pub(super) fn double_shift_manual", 1)[0]
        self.assertIn("if chars.peek().is_some()", helper)

    def test_managed_ime_config_enables_full_space_authority(self) -> None:
        config_path = self.root / "managed-config.json"
        IME_MODULE.write_managed_ime_config(config_path)
        config = json.loads(config_path.read_text(encoding="utf-8"))
        self.assertIs(config["nanda_autocorrect"], True)
        self.assertIs(config["nanda_precognition"], True)


if __name__ == "__main__":
    unittest.main()
