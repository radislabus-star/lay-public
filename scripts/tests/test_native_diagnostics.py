"""Evidence integrity tests; no input, desktop services or external packages."""
import copy
import json
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest

SCRIPTS = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(SCRIPTS))
from native_diagnostics import receipts, wire


def bundle():
    rows = [dict(window="normal", field="text", name="rapid-five", word="space", status="FAIL",
                 error="{'late_targets': []}", native_trace_complete=False, latencies_ms=[150, 180]),
            dict(window="normal", field="email", name="roundtrip", word="active", status="BLOCKED",
                 error="native/DOM text disagreement")]
    runtime = {"daemon": dict(pid=12, sha256="a" * 64, start_ticks="123")}
    return dict(actor=dict(scope="ACTUAL_NATIVE_RFB_BROWSER", browser="firefox", expected_rows=2,
                           rows=rows, counts=dict(PASS=0, FAIL=1, BLOCKED=1), status="FAIL", held_after=[]),
                verification=dict(runtime_before=runtime, runtime_after=copy.deepcopy(runtime),
                    runtime_preserved=True, config_restored=True, source=dict(sha256="b" * 64),
                    actor_sha256="c" * 64, finished_utc="2026-10-29T17:40:00Z", controller_exit_code=1),
                controller=dict(actor_sha256="c" * 64, trace_complete=False,
                    owned_exact_refresh_observed=True, owned_production_launcher_alignment=dict(
                        library_sha256="d" * 64, gtk_ime="ibus", actor_unchanged=True, host_environment_changed=False)))


def call(serial=1, stamp=100, member="VisibleTailV4", sender=":1.2"):
    return dict(phase="call", member=member, sender=sender, serial=serial, at_unix_us=stamp)


def reply(serial=1, stamp=150, member="VisibleTailV4", sender=":1.2", **values):
    return dict(phase="return", member=member, destination=sender, reply_serial=serial,
                at_unix_us=stamp, **values)


class ReceiptTests(unittest.TestCase):
    def test_keeps_failure_block_and_missing_latency_coverage(self):
        result = receipts.summarize(bundle())
        self.assertEqual(result["counts"], dict(PASS=0, FAIL=1, BLOCKED=1, NOT_TESTED=0))
        self.assertEqual(result["rapid_timing"]["space"]["expected_samples"], 5)
        self.assertEqual(result["rapid_timing"]["space"]["observed_samples"], 2)
        self.assertEqual(result["causal_evidence"]["first_failed_transition"], "UNKNOWN_REQUIRES_CORRELATED_EVENTS")
        self.assertIn("CAUSAL_CAPTURE_INCOMPLETE", result["warnings"])

    def test_partial_run_never_becomes_a_complete_matrix(self):
        data = bundle(); data["actor"]["expected_rows"] = 5
        result = receipts.summarize(data)
        self.assertEqual((result["coverage"], result["unrecorded_rows"], result["counts"]["NOT_TESTED"]), ("PARTIAL", 3, 3))

    def test_counts_and_duplicate_cases_are_rejected(self):
        data = bundle(); data["actor"]["counts"]["FAIL"] = 0
        with self.assertRaisesRegex(receipts.EvidenceError, "counts disagree"):
            receipts.summarize(data)
        data = bundle(); data["actor"]["rows"][1] = copy.deepcopy(data["actor"]["rows"][0])
        with self.assertRaisesRegex(receipts.EvidenceError, "duplicate"):
            receipts.summarize(data)

    def test_false_pass_and_held_keys_are_rejected(self):
        for key, value in (("status", "PASS"), ("held_after", [50])):
            data = bundle(); data["actor"][key] = value
            with self.assertRaises(receipts.EvidenceError):
                receipts.summarize(data)

    def test_source_test_result_cannot_be_native_evidence(self):
        data = bundle(); data["actor"]["scope"] = "SOURCE_TESTS"
        with self.assertRaisesRegex(receipts.EvidenceError, "native actor"):
            receipts.summarize(data)

    def test_claimed_runtime_preservation_is_checked_against_snapshots(self):
        data = bundle(); data["verification"]["runtime_after"]["daemon"]["sha256"] = "e" * 64
        result = receipts.summarize(data)
        self.assertFalse(result["provenance"]["runtime_snapshots_consistent"])
        self.assertIn("RUNTIME_SNAPSHOTS_MISSING_OR_CHANGED", result["warnings"])

    def test_actor_and_controller_mismatch_is_visible(self):
        data = bundle(); data["controller"]["actor_sha256"] = "e" * 64
        self.assertIn("ACTOR_IDENTITY_MISSING_OR_MISMATCHED", receipts.summarize(data)["warnings"])

    def test_trace_anchor_does_not_claim_causal_completeness(self):
        data = bundle(); data["controller"]["trace_complete"] = True
        data["actor"]["rows"][0]["native_trace_complete"] = True
        self.assertEqual(receipts.summarize(data)["causal_evidence"]["first_failed_transition"],
                         "UNKNOWN_REQUIRES_CORRELATED_EVENTS")

    def test_changed_launcher_prevents_implicit_patch_attribution(self):
        old = receipts.summarize(bundle()); data = bundle()
        data["controller"]["owned_production_launcher_alignment"]["library_sha256"] = "e" * 64
        result = receipts.compare(old, receipts.summarize(data))
        self.assertIn("launcher", result["changed_dimensions"])
        self.assertEqual(result["causal_attribution"], "NOT_ESTABLISHED_BY_COUNT_DELTA")
        self.assertTrue(result["same_failure_fingerprint"])

    def test_raw_text_and_errors_are_excluded(self):
        data = bundle(); data["actor"]["rows"][0].update(error="private-user-text", actual={"raw_text": "private-user-text"})
        data["controller"]["owned_production_launcher_alignment"]["library_sha256"] = "private-user-text"
        self.assertNotIn("private-user-text", json.dumps(receipts.summarize(data)))

    def test_bad_shapes_and_nonfinite_samples_are_visible(self):
        data = bundle(); data["verification"]["runtime_before"] = []
        with self.assertRaises(receipts.EvidenceError):
            receipts.summarize(data)
        data = bundle(); data["actor"]["rows"][0]["latencies_ms"] = [float("nan"), True, -1, 12]
        self.assertEqual(receipts.summarize(data)["rapid_timing"]["space"]["invalid_samples"], 3)

    def test_missing_config_hash_does_not_assert_matching_setup(self):
        result = receipts.summarize(bundle())
        self.assertIn("CONFIG_IDENTITY_UNRECORDED", result["warnings"])
        self.assertIn("config_sha256", receipts.compare(result, result)["missing_identity_dimensions"])

    def test_changed_config_is_a_separate_comparison_dimension(self):
        data = bundle(); data["verification"]["config_sha256"] = "e" * 64
        before = receipts.summarize(data)
        data["verification"]["config_sha256"] = "f" * 64
        self.assertIn("config_sha256", receipts.compare(before, receipts.summarize(data))["changed_dimensions"])


class WireTests(unittest.TestCase):
    def test_pairing_duration_and_text_free_export(self):
        end = reply(latency_us=50, text_chars=7, text_sha256="private-user-text", path="private-user-text")
        result, trace = wire.analyze([call(), end])
        self.assertEqual(result["pairing"], "COMPLETE")
        self.assertEqual(result["rpc_timing"]["VisibleTailV4"]["median_us"], 50)
        self.assertNotIn("private-user-text", json.dumps([result, trace]))
        self.assertEqual([e["dur"] for e in trace["traceEvents"] if e["ph"] == "X"], [50])

    def test_overlapping_calls_use_distinct_timeline_lanes(self):
        result, trace = wire.analyze([call(1, 100), call(2, 110), reply(1, 140), reply(2, 150)])
        slices = [e for e in trace["traceEvents"] if e["ph"] == "X"]
        self.assertEqual(len({e["tid"] for e in slices}), 2)
        self.assertEqual(result["paired_calls"], 2)

    def test_same_serial_on_different_senders_is_not_conflated(self):
        result, _ = wire.analyze([call(), call(sender=":1.3"), reply(), reply(sender=":1.3")])
        self.assertEqual(result["paired_calls"], 2)

    def test_unpaired_and_orphan_records_remain_incomplete(self):
        result, _ = wire.analyze([call(), reply(2)])
        self.assertEqual(result["pairing"], "INCOMPLETE")
        self.assertEqual({g["kind"] for g in result["gaps"]}, {"UNPAIRED_CALL", "ORPHAN_RETURN"})

    def test_clock_latency_and_duplicate_serial_errors_are_rejected(self):
        for events in ([call(), reply(stamp=99)], [call(), reply(latency_us=49)], [call(), call()]):
            with self.subTest(events=events), self.assertRaises(receipts.EvidenceError):
                wire.analyze(events)

    def test_false_ack_is_observation_not_automatic_defect(self):
        first = call(member="NativeReplayProcessedV1"); first["epoch"] = 5
        result, _ = wire.analyze([first, reply(member="NativeReplayProcessedV1", value=False)])
        self.assertEqual(result["observed_false_ack_excluding_empty_epoch_probe"], 1)
        self.assertEqual(result["first_failed_transition"], "UNKNOWN_WITHOUT_OPERATION_EXPECTATIONS")

    def test_wire_comparison_exposes_counts_and_missing_provenance(self):
        before, _ = wire.analyze([call(), reply(), call(2, 200), reply(2, 250)])
        after, _ = wire.analyze([call(), reply()])
        result = wire.compare(before, after)
        self.assertEqual(result["calls"]["VisibleTailV4"], {"before": 2, "after": 1})
        self.assertIn("NOT_PRESENT", result["runtime_and_actor_identity"])

    def test_wire_comparison_keeps_incomplete_previous_capture_visible(self):
        before, _ = wire.analyze([call()]); after, _ = wire.analyze([call(), reply()])
        self.assertFalse(wire.compare(before, after)["pairing_complete"])

    def test_cli_keeps_product_verdict_separate_and_never_overwrites(self):
        with tempfile.TemporaryDirectory() as folder:
            path = Path(folder); source = path / "receipt.json"; source.write_text(json.dumps(bundle()))
            command = [sys.executable, str(SCRIPTS / "lay-diagnose.py"), "report", str(source), "--output", str(path / "report")]
            first = subprocess.run(command, capture_output=True, text=True)
            self.assertEqual(first.returncode, 0, first.stderr + first.stdout)
            self.assertEqual(json.loads(first.stdout)["counts"]["FAIL"], 1)
            second = subprocess.run(command, capture_output=True, text=True)
            self.assertEqual(second.returncode, 2)
            self.assertEqual(json.loads((path / "report/report.json").read_text())["counts"]["FAIL"], 1)


if __name__ == "__main__":
    unittest.main()
