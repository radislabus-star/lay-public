"""Prove that lost entrypoints and undeclared rule changes are rejected."""

from __future__ import annotations

import importlib.util
import json
import subprocess
import tempfile
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
spec = importlib.util.spec_from_file_location("canon_gate", ROOT / "scripts/check_architecture_canon.py")
assert spec is not None and spec.loader is not None
gate = importlib.util.module_from_spec(spec)
spec.loader.exec_module(gate)


class CanonGuardTest(unittest.TestCase):
    def setUp(self):
        temporary = tempfile.TemporaryDirectory()
        self.addCleanup(temporary.cleanup)
        self.root = Path(temporary.name)
        self.write("ARCHITECTURE.md", "\n".join(f"## C{n:02} — rule" for n in range(1, 11)))
        self.write("AGENTS.md", "Read [canon](ARCHITECTURE.md).")
        self.write("CLAUDE.md", "Read [rules](AGENTS.md) and [canon](ARCHITECTURE.md).")
        self.write("GEMINI.md", "Read [rules](AGENTS.md) and [canon](ARCHITECTURE.md).")
        self.write(".github/copilot-instructions.md",
                   "Read [rules](../AGENTS.md) and [canon](../ARCHITECTURE.md).")

    def write(self, name, text):
        path = self.root / name
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(text, encoding="utf-8")

    def decision(self, paths, **overrides):
        name = "docs/architecture/decisions/new.json"
        record = {
            "schema": "lay.architecture-decision.v1",
            "decision": "Keep one explicit contract.",
            "reason": "A bounded contract change.",
            "invariants": ["C03"],
            "protected_paths": paths,
            "verification": "Focused ownership and refusal tests.",
            "not_tested": "Real client behavior.",
            "runtime_authority_changed": False,
            **overrides,
        }
        self.write(name, json.dumps(record))
        return name

    def test_all_model_entrypoints_reach_one_canon(self):
        self.assertEqual([], gate.check_links(self.root))

    def test_missing_model_entrypoint_is_rejected(self):
        (self.root / "CLAUDE.md").unlink()
        self.assertIn("missing_file:CLAUDE.md", gate.check_links(self.root))

    def test_diverted_model_entrypoint_is_rejected(self):
        self.write("GEMINI.md", "Read the old session.")
        self.assertIn("entrypoint_missing_link:GEMINI.md:ARCHITECTURE.md", gate.check_links(self.root))

    def test_broken_owner_document_link_is_rejected(self):
        with (self.root / "ARCHITECTURE.md").open("a") as stream:
            stream.write("\n[owner](docs/missing.md)\n")
        self.assertIn("broken_link:ARCHITECTURE.md:docs/missing.md", gate.check_links(self.root))

    def test_removed_invariant_is_rejected(self):
        path = self.root / "ARCHITECTURE.md"
        path.write_text(path.read_text().replace("## C08", "## Removed"))
        self.assertIn("canon_rule_ids:expected_C01_through_C10_once", gate.check_links(self.root))

    def test_protected_change_needs_new_decision(self):
        self.assertIn("protected_change_without_new_decision:ARCHITECTURE.md",
                      gate.check_decisions(self.root, {"ARCHITECTURE.md"}, set()))

    def test_editing_old_decision_does_not_authorize_new_change(self):
        record = self.decision(["ARCHITECTURE.md"])
        errors = gate.check_decisions(self.root, {"ARCHITECTURE.md", record}, set())
        self.assertIn("protected_change_without_new_decision:ARCHITECTURE.md", errors)

    def test_new_decision_covers_only_declared_protected_files(self):
        record = self.decision(["ARCHITECTURE.md"])
        changed = {"ARCHITECTURE.md", "AGENTS.md", record}
        self.assertEqual(["protected_change_without_new_decision:AGENTS.md"],
                         gate.check_decisions(self.root, changed, {record}))

    def test_invalid_runtime_flag_cannot_claim_a_valid_record(self):
        record = self.decision(["ARCHITECTURE.md"], runtime_authority_changed="false")
        errors = gate.check_decisions(self.root, {"ARCHITECTURE.md", record}, {record})
        self.assertTrue(any(error.startswith("invalid_new_decision:") for error in errors))
        self.assertIn("protected_change_without_new_decision:ARCHITECTURE.md", errors)

    def test_valid_new_record_admits_rule_change_without_runtime_claim(self):
        record = self.decision(["ARCHITECTURE.md"])
        self.assertEqual([], gate.check_decisions(self.root, {"ARCHITECTURE.md", record}, {record}))

    def test_workflow_deletion_requires_a_decision(self):
        name = ".github/workflows/ci.yml"
        self.assertEqual([f"protected_change_without_new_decision:{name}"],
                         gate.check_decisions(self.root, {name}, set()))

    def test_regular_source_change_remains_owned_by_semantic_gates(self):
        self.assertEqual([], gate.check_decisions(self.root, {"src/input_gate.rs"}, set()))

    def test_git_diff_includes_deletions_and_untracked_records(self):
        def git(*args):
            return subprocess.run(["git", "-C", str(self.root), *args], check=True,
                                  capture_output=True, text=True)
        git("init", "-q")
        git("add", ".")
        git("-c", "user.name=Canon Test", "-c", "user.email=canon@example.invalid",
            "-c", "commit.gpgsign=false", "commit", "-qm", "fixture")
        (self.root / "CLAUDE.md").unlink()
        self.write("AGENTS.md", "changed")
        record = self.decision(["CLAUDE.md", "AGENTS.md"])
        changed, added = gate.changes_since(self.root, "HEAD")
        self.assertEqual({"CLAUDE.md", "AGENTS.md", record}, changed)
        self.assertEqual({record}, added)
        self.assertEqual([], gate.check_decisions(self.root, changed, added))


if __name__ == "__main__":
    unittest.main()
