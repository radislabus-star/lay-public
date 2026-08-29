#!/usr/bin/env python3
"""Regression tests for architecture scan ownership and evidence scope."""

from __future__ import annotations

import importlib.util
import tempfile
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
GATE_PATH = ROOT / "scripts" / "architecture_scope_gate.py"

spec = importlib.util.spec_from_file_location("architecture_scope_gate", GATE_PATH)
assert spec is not None and spec.loader is not None
gate = importlib.util.module_from_spec(spec)
spec.loader.exec_module(gate)


class ArchitectureScopeGateTest(unittest.TestCase):
    def fixture(self, files: dict[str, str]) -> Path:
        directory = tempfile.TemporaryDirectory()
        self.addCleanup(directory.cleanup)
        root = Path(directory.name)
        for relative, content in files.items():
            path = root / relative
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_text(content, encoding="utf-8")
        return root

    def test_abusive_runtime_source_is_rejected(self) -> None:
        root = self.fixture({"src/live.rs": "const MESSAGE: &str = \"пиздец\";\n"})

        self.assertEqual(
            ["src/live.rs:1:abusive_snippet"],
            gate.find_abusive_snippets(root),
        )

    def test_sealed_receipt_and_proof_text_are_outside_runtime_scope(self) -> None:
        root = self.fixture(
            {
                "docs/structural_gates/receipts/sealed.json": '{"surface":"пиздец"}\n',
                "src/exact_layout_authority/proof.rs":
                    'const FIXTURE: &str = "пиздец";\n',
            }
        )

        self.assertEqual([], gate.find_abusive_snippets(root))

    def test_sleep_in_live_correction_owner_is_rejected(self) -> None:
        root = self.fixture(
            {"src/typing_transition/live.rs": "std::thread::sleep(DELAY);\n"}
        )

        self.assertEqual(
            ["src/typing_transition/live.rs:1:unowned_thread_sleep"],
            gate.find_sleep_violations(root),
        )

    def test_explicit_background_and_proof_sleep_owners_are_accepted(self) -> None:
        root = self.fixture(
            {
                "src/nanda_wave/context_phase/runtime_refresh.rs":
                    "std::thread::sleep(REFRESH);\n",
                "src/exact_layout_authority/proof.rs": "thread::sleep(SETTLE);\n",
                "src/bin/lay_nanda_wave_train/watch.rs": "thread::sleep(POLL);\n",
            }
        )

        self.assertEqual([], gate.find_sleep_violations(root))

    def test_sleep_scan_crosses_comments_and_newlines(self) -> None:
        root = self.fixture(
            {
                "src/live.rs": (
                    "fn live() {\n"
                    "    std::thread /* no bypass */\n"
                    "        :: sleep(DELAY);\n"
                    "}\n"
                )
            }
        )

        self.assertEqual(
            ["src/live.rs:2:unowned_thread_sleep"],
            gate.find_sleep_violations(root),
        )

    def test_unknown_private_open_owner_is_rejected(self) -> None:
        root = self.fixture({"src/new_owner.rs": "use std::os::unix::fs::OpenOptionsExt;\n"})

        self.assertEqual(
            ["src/new_owner.rs:1:unowned_OpenOptionsExt"],
            gate.find_file_operation_violations(root),
        )

    def test_distinct_private_package_and_socket_owners_are_accepted(self) -> None:
        root = self.fixture(
            {
                "src/private_file.rs": "use std::os::unix::fs::OpenOptionsExt;\n",
                "src/nanda_wave/context_phase/composite.rs":
                    "use std::os::unix::fs::OpenOptionsExt;\n",
                "src/bin/lay_l1_1_serve.rs": (
                    "use std::os::unix::fs::PermissionsExt;\n"
                    "fs::set_permissions(socket, mode)?;\n"
                ),
            }
        )

        self.assertEqual([], gate.find_file_operation_violations(root))

    def test_runtime_tmp_lay_path_is_rejected(self) -> None:
        root = self.fixture({"src/live.rs": 'const PATH: &str = "/tmp/lay-live";\n'})

        self.assertEqual(
            ["src/live.rs:1:runtime_tmp_lay_path"],
            gate.find_runtime_tmp_path_violations(root),
        )

    def test_test_module_tmp_lay_path_is_accepted(self) -> None:
        root = self.fixture(
            {
                "src/live.rs": (
                    "pub fn live() {}\n"
                    "#[cfg(test)]\n"
                    "mod tests {\n"
                    'const PATH: &str = "/tmp/lay-test";\n'
                    "}\n"
                )
            }
        )

        self.assertEqual([], gate.find_runtime_tmp_path_violations(root))

    def test_nonterminal_test_item_is_excluded_without_hiding_later_production(self) -> None:
        root = self.fixture(
            {
                "src/live.rs": (
                    "pub fn before() {}\n"
                    "#[cfg(test)]\n"
                    "mod fixtures {\n"
                    '    const SAMPLE: &str = "bad-example";\n'
                    '    const BRACES: &str = "{ not code }";\n'
                    "}\n"
                    'pub fn after() { let _ = "bad-example"; }\n'
                )
            }
        )

        self.assertEqual(
            ["src/live.rs:7:production_literal:bad-example"],
            gate.find_production_literal_violations("bad-example", root),
        )

    def test_cfg_test_function_is_excluded_without_hiding_neighbors(self) -> None:
        root = self.fixture(
            {
                "src/live.rs": (
                    "pub fn before() {}\n"
                    "#[cfg(test)]\n"
                    'fn fixture() { let _ = "bad-example"; }\n'
                    "pub fn after() {}\n"
                )
            }
        )

        self.assertEqual([], gate.find_production_literal_violations("bad-example", root))

    def test_call_owner_matching_does_not_confuse_prefixed_identifier(self) -> None:
        root = self.fixture(
            {
                "src/live.rs": (
                    "fn live() {\n"
                    "    can_replace_committed_tail(3);\n"
                    "    replace_committed_tail(3);\n"
                    "}\n"
                )
            }
        )

        self.assertEqual(
            ["src/live.rs:3:unowned_call:replace_committed_tail"],
            gate.find_call_owner_violations("replace_committed_tail(", (), root),
        )

    def test_call_owner_matching_crosses_comments_and_newlines(self) -> None:
        root = self.fixture(
            {
                "src/live.rs": (
                    "fn live() {\n"
                    "    replace_committed_tail /* reason */\n"
                    "        (3);\n"
                    "}\n"
                )
            }
        )

        self.assertEqual(
            ["src/live.rs:2:unowned_call:replace_committed_tail"],
            gate.find_call_owner_violations("replace_committed_tail(", (), root),
        )

    def test_call_under_cfg_test_module_is_not_a_runtime_owner(self) -> None:
        root = self.fixture(
            {
                "src/live.rs": (
                    "#[cfg(test)]\n"
                    "mod candidate_sources_tests {\n"
                    "    fn fixture() { resolve_text_correction(request()); }\n"
                    "}\n"
                )
            }
        )

        self.assertEqual(
            [],
            gate.find_call_owner_violations("resolve_text_correction(", (), root),
        )

    def test_runtime_rule_ids_keep_typing_and_wave_taxonomies_separate(self) -> None:
        root = self.fixture(
            {
                "src/live.rs": 'const RULE: &str = "missing_letter";\n',
                "src/typing_rule_graph/ids.rs":
                    'const MISSING: &str = "missing_letter";\n',
                "src/nanda_wave/lexical_grokking/corruption.rs":
                    'const CLASS: &str = "missing_letter";\n',
            }
        )

        self.assertEqual(
            ["src/live.rs:1:unowned_runtime_rule_id"],
            gate.find_runtime_rule_id_violations(root),
        )

    def test_unknown_proof_filename_is_not_implicitly_excluded(self) -> None:
        root = self.fixture(
            {"src/new_route/proof.rs": "std::thread::sleep(DELAY);\n"}
        )

        self.assertEqual(
            ["src/new_route/proof.rs:1:unowned_thread_sleep"],
            gate.find_sleep_violations(root),
        )

    def test_lifetimes_and_labels_do_not_hide_production_after_test_item(self) -> None:
        root = self.fixture(
            {
                "src/live.rs": (
                    "#[cfg(test)]\n"
                    "fn fixture<'a>() { 'label: loop { break 'label; } }\n"
                    'pub fn after() { let _ = "bad-example"; }\n'
                )
            }
        )

        self.assertEqual(
            ["src/live.rs:3:production_literal:bad-example"],
            gate.find_production_literal_violations("bad-example", root),
        )

    def test_cfg_test_field_cannot_hide_following_production(self) -> None:
        root = self.fixture(
            {
                "src/live.rs": (
                    "struct Example {\n"
                    "    #[cfg(test)]\n"
                    "    fixture_only: u8,\n"
                    "    production: u8,\n"
                    "}\n"
                    'pub fn after() { let _ = "bad-example"; }\n'
                )
            }
        )

        self.assertEqual(
            ["src/live.rs:6:production_literal:bad-example"],
            gate.find_production_literal_violations("bad-example", root),
        )


if __name__ == "__main__":
    unittest.main()
