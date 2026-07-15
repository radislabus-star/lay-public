#!/usr/bin/env python3
"""Focused regression tests for architecture receipt freshness semantics."""

from __future__ import annotations

import importlib.util
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
GATE_PATH = ROOT / "scripts" / "architecture_graph_gate.py"

spec = importlib.util.spec_from_file_location("architecture_graph_gate", GATE_PATH)
assert spec is not None and spec.loader is not None
gate = importlib.util.module_from_spec(spec)
spec.loader.exec_module(gate)


def receipt(source_fingerprint: str) -> dict[str, object]:
    return {
        "schema": gate.SCHEMA,
        "verdict": "PASS",
        "source_fingerprint": source_fingerprint,
        "graph_fingerprint": "graph-a",
        "graph_nodes": 1,
        "graph_links": 1,
        "checks": [],
        "duplicate_symbols": {},
    }


class ArchitectureReceiptFreshnessTest(unittest.TestCase):
    def test_head_only_metadata_change_does_not_stale_receipt(self) -> None:
        existing = receipt("source-a")
        expected = receipt("source-a")
        existing["git_head"] = "old-head"
        expected["git_head"] = "new-head"
        existing["graph_built_at_commit"] = "old-head"
        expected["graph_built_at_commit"] = "new-head"

        self.assertEqual([], gate.receipt_staleness_violations(existing, expected))

    def test_source_fingerprint_change_stales_receipt(self) -> None:
        violations = gate.receipt_staleness_violations(
            receipt("source-a"),
            receipt("source-b"),
        )

        self.assertIn("receipt_payload_mismatch", violations)
        self.assertTrue(
            any(item.startswith("source_fingerprint:") for item in violations)
        )

    def test_graph_fingerprint_change_stales_receipt(self) -> None:
        existing = receipt("source-a")
        expected = receipt("source-a")
        expected["graph_fingerprint"] = "graph-b"

        self.assertIn(
            "receipt_payload_mismatch",
            gate.receipt_staleness_violations(existing, expected),
        )


if __name__ == "__main__":
    unittest.main()
