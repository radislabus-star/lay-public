#!/usr/bin/env python3
"""Focused regression tests for architecture receipt freshness semantics."""

from __future__ import annotations

import importlib.util
import os
import tempfile
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
    def graph_freshness(
        self,
        sources: dict[str, str],
        manifest: dict[str, object],
        graph: dict[str, object] | None = None,
    ) -> list[str]:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            for relative, content in sources.items():
                path = root / relative
                path.parent.mkdir(parents=True, exist_ok=True)
                path.write_text(content, encoding="utf-8")
            previous_root = gate.ROOT
            gate.ROOT = root
            try:
                return gate.graph_freshness_violations(manifest, graph)
            finally:
                gate.ROOT = previous_root

    def test_graph_source_freshness_uses_content_not_mtime(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            source = Path(directory) / "source.rs"
            source.write_text("fn stable() {}\n", encoding="utf-8")
            entry = {"ast_hash": gate.hashlib.md5(source.read_bytes()).hexdigest()}
            os.utime(source, (1, 1))

            self.assertIsNone(
                gate.graph_source_freshness_violation(
                    source, "src/source.rs", entry
                )
            )

    def test_graph_source_freshness_rejects_changed_content(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            source = Path(directory) / "source.rs"
            source.write_text("fn before() {}\n", encoding="utf-8")
            entry = {"ast_hash": gate.hashlib.md5(source.read_bytes()).hexdigest()}
            source.write_text("fn after() {}\n", encoding="utf-8")

            self.assertEqual(
                "graph_stale_source:src/source.rs",
                gate.graph_source_freshness_violation(
                    source, "src/source.rs", entry
                ),
            )

    def test_graph_freshness_accepts_unchanged_source_set(self) -> None:
        source = "fn stable() {}\n"
        manifest = {
            "src/stable.rs": {
                "ast_hash": gate.hashlib.md5(source.encode()).hexdigest()
            }
        }

        self.assertEqual([], self.graph_freshness({"src/stable.rs": source}, manifest))

    def test_graph_freshness_rejects_added_source(self) -> None:
        self.assertEqual(
            ["graph_missing_source:src/added.rs"],
            self.graph_freshness({"src/added.rs": "fn added() {}\n"}, {}),
        )

    def test_graph_freshness_rejects_deleted_source(self) -> None:
        self.assertEqual(
            ["graph_removed_source:src/deleted.rs"],
            self.graph_freshness(
                {},
                {"src/deleted.rs": {"ast_hash": "obsolete"}},
            ),
        )

    def test_graph_freshness_rejects_renamed_source(self) -> None:
        source = "fn renamed() {}\n"
        self.assertEqual(
            [
                "graph_missing_source:src/new.rs",
                "graph_removed_source:src/old.rs",
            ],
            self.graph_freshness(
                {"src/new.rs": source},
                {"src/old.rs": {"ast_hash": gate.hashlib.md5(source.encode()).hexdigest()}},
            ),
        )

    def test_graph_freshness_rejects_stale_graph_after_manifest_repair(self) -> None:
        source = "fn current() {}\n"
        manifest = {
            "src/current.rs": {
                "ast_hash": gate.hashlib.md5(source.encode()).hexdigest()
            }
        }
        graph = {
            "nodes": [{"source_file": "src/deleted.rs"}],
            "links": [],
        }

        self.assertEqual(
            [
                "graph_missing_source_reference:src/current.rs",
                "graph_removed_source_reference:src/deleted.rs",
            ],
            self.graph_freshness({"src/current.rs": source}, manifest, graph),
        )

    def test_graph_binding_rejects_same_path_source_drift(self) -> None:
        binding = {
            "schema": gate.GRAPH_BINDING_SCHEMA,
            "graph_fingerprint": "graph-a",
            "manifest_fingerprint": "manifest-current",
            "rust_sources": {"src/current.rs": "old-source"},
        }

        self.assertEqual(
            ["graph_binding_rust_sources_mismatch"],
            gate.graph_binding_violations(
                binding,
                "graph-a",
                "manifest-current",
                {"src/current.rs": "new-source"},
            ),
        )

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

    def test_external_import_with_ambiguous_graph_target_is_not_forbidden(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            path = root / "src/nanda_wave/example.rs"
            path.parent.mkdir(parents=True)
            path.write_text("use std::io::Cursor;\n", encoding="utf-8")
            graph = gate.ArchitectureGraph(
                {
                    "nodes": [],
                    "links": [
                        {
                            "relation": "imports_from",
                            "source_file": "src/nanda_wave/example.rs",
                            "source_location": "L1",
                            "target": "src_text_edit_cursor",
                        }
                    ],
                }
            )
            previous_root = gate.ROOT
            gate.ROOT = root
            try:
                self.assertEqual(
                    [],
                    graph.source_imports("src/nanda_wave", ("src_text_edit",)),
                )
            finally:
                gate.ROOT = previous_root

    def test_internal_forbidden_import_still_fails(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            path = root / "src/nanda_wave/example.rs"
            path.parent.mkdir(parents=True)
            path.write_text("use crate::text_edit::cursor::Cursor;\n", encoding="utf-8")
            graph = gate.ArchitectureGraph(
                {
                    "nodes": [],
                    "links": [
                        {
                            "relation": "imports_from",
                            "source_file": "src/nanda_wave/example.rs",
                            "source_location": "L1",
                            "target": "src_text_edit_cursor",
                        }
                    ],
                }
            )
            previous_root = gate.ROOT
            gate.ROOT = root
            try:
                self.assertEqual(
                    [
                        "forbidden_import:src/nanda_wave/example.rs:L1:src_text_edit_cursor"
                    ],
                    graph.source_imports("src/nanda_wave", ("src_text_edit",)),
                )
            finally:
                gate.ROOT = previous_root

    def test_bare_internal_import_is_not_hidden_as_external(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            path = root / "src/nanda_wave/example.rs"
            path.parent.mkdir(parents=True)
            path.write_text("use text_edit::cursor::Cursor;\n", encoding="utf-8")
            graph = gate.ArchitectureGraph(
                {
                    "nodes": [],
                    "links": [
                        {
                            "relation": "imports_from",
                            "source_file": "src/nanda_wave/example.rs",
                            "source_location": "L1",
                            "target": "src_text_edit_cursor",
                        }
                    ],
                }
            )
            previous_root = gate.ROOT
            gate.ROOT = root
            try:
                self.assertEqual(
                    [
                        "forbidden_import:src/nanda_wave/example.rs:L1:src_text_edit_cursor"
                    ],
                    graph.source_imports("src/nanda_wave", ("src_text_edit",)),
                )
            finally:
                gate.ROOT = previous_root

    def test_production_rows_exclude_terminal_cfg_test_module(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "source.rs"
            path.write_text(
                "production();\n#[cfg(test)]\nmod tests {\ntest_only();\n}\n",
                encoding="utf-8",
            )

            self.assertEqual([(1, "production();")], gate.production_source_rows(path))

    def test_struct_body_excludes_ephemeral_return_types(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "source.rs"
            path.write_text(
                "struct Memory {\n  bytes: Vec<u8>,\n}\n"
                "fn output() -> Vec<String> { Vec::new() }\n",
                encoding="utf-8",
            )

            self.assertNotIn("Vec<String>", gate.struct_body(path, "Memory"))

    def test_struct_body_includes_persistent_string_storage(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "source.rs"
            path.write_text(
                "struct Memory {\n  words: Vec<String>,\n}\n",
                encoding="utf-8",
            )

            self.assertIn("Vec<String>", gate.struct_body(path, "Memory"))

    def test_struct_body_ignores_braces_in_comments(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "source.rs"
            path.write_text(
                "struct Memory {\n"
                "  /// A misleading closing brace: }\n"
                "  marker: u8,\n"
                "  words: Vec<String>,\n"
                "}\n",
                encoding="utf-8",
            )

            self.assertIn("Vec<String>", gate.struct_body(path, "Memory"))


if __name__ == "__main__":
    unittest.main()
