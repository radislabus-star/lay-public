from __future__ import annotations

import csv
from dataclasses import replace
import importlib.util
import os
import stat
import sys
import tempfile
import threading
import unittest
from unittest import mock
from pathlib import Path


SCRIPT = Path(__file__).resolve().parents[1] / "research-evidence-store.py"
SPEC = importlib.util.spec_from_file_location("research_evidence_store", SCRIPT)
assert SPEC is not None and SPEC.loader is not None
MODULE = importlib.util.module_from_spec(SPEC)
sys.modules[SPEC.name] = MODULE
SPEC.loader.exec_module(MODULE)


class EvidenceStoreTests(unittest.TestCase):
    def setUp(self) -> None:
        self.temporary = tempfile.TemporaryDirectory()
        self.root = Path(self.temporary.name)
        self.repo = self.root / "repo"
        self.store_root = self.root / "store"
        self.repo.mkdir()
        payloads = {
            "evidence/a.bin": (b"alpha payload\n" * 31, 0o444),
            "evidence/a-copy.bin": (b"alpha payload\n" * 31, 0o444),
            "evidence/tool.elf": (b"ELF fixture\0" * 37, 0o555),
        }
        rows = []
        for relative, (payload, mode) in payloads.items():
            path = self.repo / relative
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_bytes(payload)
            path.chmod(mode)
            rows.append(
                {
                    "path": relative,
                    "size_bytes": str(len(payload)),
                    "mode": f"{mode:o}",
                    "sha256": MODULE.sha256_file(path),
                    "git_state": "ignored",
                    "selection": "fixture",
                    "independent_backup_count_before": "0",
                }
            )
        self.inventory = self.repo / "inventory.tsv"
        with self.inventory.open("w", newline="", encoding="utf-8") as handle:
            writer = csv.DictWriter(handle, fieldnames=rows[0].keys(), delimiter="\t")
            writer.writeheader()
            writer.writerows(rows)
        self.entries = MODULE.load_inventory(self.inventory)
        evidence_parent = self.repo / "evidence"
        evidence_parent.chmod(0o555)
        parent_stat = evidence_parent.stat()
        self.parent_inventory = self.repo / "parent-modes.tsv"
        self.parent_inventory.write_text(
            "path\tmode\tuid\tgid\n"
            f"evidence\t0555\t{parent_stat.st_uid}\t{parent_stat.st_gid}\n",
            encoding="utf-8",
        )
        self.parent_contracts = MODULE.load_parent_inventory(self.parent_inventory)

    def tearDown(self) -> None:
        for path in self.root.rglob("*"):
            if path.is_dir() and not path.is_symlink():
                path.chmod(0o700)
        for path in self.root.rglob("*"):
            if path.is_file() and not path.is_symlink():
                path.chmod(0o600)
        self.temporary.cleanup()

    def new_store(self, fault: str | None = None):
        return MODULE.EvidenceStore(
            self.repo,
            self.store_root,
            self.entries,
            self.parent_contracts,
            MODULE.FaultInjector(fault),
        )

    def test_stage_deduplicates_and_preserves_modes(self) -> None:
        store = self.new_store()
        result = store.stage(self.entries)
        self.assertEqual(result["objects_created"], 2)
        self.assertEqual(store.verify(self.entries)["objects_verified"], 2)
        for entry in self.entries:
            mode = stat.S_IMODE(store.object_path(entry).stat().st_mode)
            self.assertEqual(mode, entry.mode)

    def test_failure_before_object_publish_leaves_original(self) -> None:
        entry = self.entries[0]
        store = self.new_store("before-object-publish")
        with self.assertRaises(MODULE.InjectedFailure):
            store.ensure_object(entry)
        source = store.source_path(entry)
        self.assertTrue(source.is_file())
        self.assertFalse(source.is_symlink())
        store.verify_file(source, entry)
        self.assertFalse(store.object_path(entry).exists())

    def test_recovery_after_backup_rename(self) -> None:
        entry = self.entries[0]
        store = self.new_store()
        store.ensure_object(entry)
        store.fault = MODULE.FaultInjector("after-backup-rename")
        with self.assertRaises(MODULE.InjectedFailure):
            store.externalize_entry(entry)
        source = store.source_path(entry)
        backup = source.with_name(f".{source.name}.td103-backup")
        self.assertFalse(source.exists())
        self.assertTrue(backup.is_file())

        store.fault = MODULE.FaultInjector()
        self.assertEqual(store.externalize_entry(entry), "externalized")
        self.assertTrue(source.is_symlink())
        self.assertFalse(backup.exists())
        store.verify_file(source, entry)

    def test_parent_mode_is_restored_after_handled_failure(self) -> None:
        entry = self.entries[0]
        store = self.new_store()
        store.ensure_object(entry)
        store.fault = MODULE.FaultInjector("after-parent-chmod")
        with self.assertRaises(MODULE.InjectedFailure):
            store.externalize_entry(entry)
        self.assertEqual(stat.S_IMODE(store.source_path(entry).parent.stat().st_mode), 0o555)

    def test_parent_mode_is_recovered_from_durable_intent(self) -> None:
        store = self.new_store()
        parent = self.repo / "evidence"
        store.journal(
            "parent-write-intent",
            parent="evidence",
            original_mode="0555",
        )
        parent.chmod(0o755)
        self.assertEqual(store.recover_parent_modes(), 1)
        self.assertEqual(stat.S_IMODE(parent.stat().st_mode), 0o555)
        self.assertEqual(store.recover_parent_modes(), 0)

    def test_first_journal_write_fsyncs_transaction_directory(self) -> None:
        store = self.new_store()
        transaction_directory = self.store_root / "transactions"
        real_fsync = MODULE.fsync_directory
        with mock.patch.object(MODULE, "fsync_directory", wraps=real_fsync) as fsync:
            store.journal("fixture-event")
        self.assertIn(mock.call(transaction_directory), fsync.call_args_list)

    def test_lock_and_journal_symlinks_are_rejected(self) -> None:
        store = self.new_store()
        self.store_root.mkdir()
        victim = self.root / "victim"
        victim.write_text("protected", encoding="utf-8")
        os.symlink(victim, self.store_root / "LOCK")
        with self.assertRaises(MODULE.EvidenceError):
            with store.lock():
                pass
        (self.store_root / "LOCK").unlink()
        transactions = self.store_root / "transactions"
        transactions.mkdir()
        os.symlink(victim, transactions / "td103-v1.jsonl")
        with self.assertRaises(MODULE.EvidenceError):
            store.journal("fixture-event")
        self.assertEqual(victim.read_text(encoding="utf-8"), "protected")

    def test_recovery_after_link_publish(self) -> None:
        entry = self.entries[1]
        store = self.new_store()
        store.ensure_object(entry)
        store.fault = MODULE.FaultInjector("after-link-publish")
        with self.assertRaises(MODULE.InjectedFailure):
            store.externalize_entry(entry)
        source = store.source_path(entry)
        backup = source.with_name(f".{source.name}.td103-backup")
        self.assertTrue(source.is_symlink())
        self.assertTrue(backup.is_file())

        store.fault = MODULE.FaultInjector()
        self.assertEqual(store.externalize_entry(entry), "already-externalized")
        self.assertTrue(source.is_symlink())
        self.assertFalse(backup.exists())
        store.verify_file(source, entry)

    def test_materialize_and_externalize_round_trip(self) -> None:
        store = self.new_store()
        store.externalize(self.entries)
        entry = self.entries[-1]
        source = store.source_path(entry)
        self.assertTrue(source.is_symlink())
        self.assertEqual(store.materialize_entry(entry), "materialized")
        self.assertTrue(source.is_file())
        self.assertFalse(source.is_symlink())
        self.assertEqual(stat.S_IMODE(source.stat().st_mode), entry.mode)
        self.assertEqual(store.externalize_entry(entry), "externalized")
        self.assertTrue(source.is_symlink())
        store.verify_file(source, entry)

    def test_catalog_publish_fault_retains_no_partial_file(self) -> None:
        store = self.new_store()
        self.store_root.mkdir()
        output = self.root / "catalog.json"
        with self.assertRaises(MODULE.InjectedFailure):
            MODULE.atomic_json(
                output,
                store.catalog(),
                MODULE.FaultInjector("before-catalog-publish"),
            )
        self.assertFalse(output.exists())
        self.assertEqual(list(output.parent.glob(f".{output.name}.*")), [])

    def test_inventory_rejects_repository_escape(self) -> None:
        invalid = self.repo / "invalid.tsv"
        invalid.write_text(
            "path\tsize_bytes\tmode\tsha256\tgit_state\tselection\tindependent_backup_count_before\n"
            f"../escape\t1\t444\t{'0' * 64}\tignored\tfixture\t0\n",
            encoding="utf-8",
        )
        with self.assertRaises(MODULE.EvidenceError):
            MODULE.load_inventory(invalid)

    def test_content_object_symlink_is_rejected(self) -> None:
        entry = self.entries[0]
        store = self.new_store()
        store.ensure_object(entry)
        object_path = store.object_path(entry)
        object_path.unlink()
        os.symlink(store.source_path(entry), object_path)
        with self.assertRaises(MODULE.EvidenceError):
            store.ensure_object(entry)

    def test_object_tree_ancestor_symlink_is_rejected(self) -> None:
        entry = self.entries[0]
        outside = self.root / "outside"
        outside.mkdir()
        self.store_root.mkdir()
        os.symlink(outside, self.store_root / "objects", target_is_directory=True)
        store = self.new_store()
        with self.assertRaises(MODULE.EvidenceError):
            store.ensure_object(entry)
        self.assertEqual(list(outside.iterdir()), [])

    def test_absolute_projection_target_is_rejected(self) -> None:
        entry = self.entries[0]
        store = self.new_store()
        store.externalize([entry])
        source = store.source_path(entry)
        parent = source.parent
        parent.chmod(0o755)
        source.unlink()
        os.symlink(store.object_path(entry), source)
        parent.chmod(0o555)
        with self.assertRaises(MODULE.EvidenceError):
            store.verify_projection(entry)

    def test_stage_rejects_source_symlink_before_copy(self) -> None:
        entry = self.entries[0]
        source = self.repo / entry.path
        outside = self.root / "outside-source"
        outside.write_bytes(source.read_bytes())
        outside.chmod(entry.mode)
        source.parent.chmod(0o755)
        source.unlink()
        os.symlink(outside, source)
        source.parent.chmod(0o555)
        store = self.new_store()
        with self.assertRaises(MODULE.EvidenceError):
            store.ensure_object(entry)
        self.assertFalse(store.object_path(entry).exists())

    def test_unknown_backup_collision_is_preserved_and_blocked(self) -> None:
        entry = self.entries[0]
        store = self.new_store()
        store.ensure_object(entry)
        source = store.source_path(entry)
        backup = source.with_name(f".{source.name}.td103-backup")
        source.parent.chmod(0o755)
        backup.write_bytes(source.read_bytes())
        backup.chmod(entry.mode)
        source.parent.chmod(0o555)
        with self.assertRaises(MODULE.EvidenceError):
            store.externalize_entry(entry)
        self.assertTrue(source.is_file())
        self.assertTrue(backup.is_file())

    def test_backup_symlink_is_preserved_and_blocked(self) -> None:
        entry = self.entries[0]
        store = self.new_store()
        store.ensure_object(entry)
        source = store.source_path(entry)
        backup = source.with_name(f".{source.name}.td103-backup")
        source.parent.chmod(0o755)
        os.symlink(source, backup)
        source.parent.chmod(0o555)
        with self.assertRaises(MODULE.EvidenceError):
            store.externalize_entry(entry)
        self.assertTrue(backup.is_symlink())

    def test_missing_projection_is_reconstructed_from_verified_object(self) -> None:
        entry = self.entries[0]
        store = self.new_store()
        store.ensure_object(entry)
        source = store.source_path(entry)
        source.parent.chmod(0o755)
        source.unlink()
        source.parent.chmod(0o555)
        self.assertEqual(store.externalize_entry(entry), "reconstructed")
        self.assertEqual(store.verify_projection(entry), "symlink")

    def test_seal_makes_every_object_directory_read_only(self) -> None:
        store = self.new_store()
        store.externalize(self.entries)
        result = store.seal_objects()
        self.assertEqual(result["verdict"], "PASS")
        self.assertTrue(store.object_tree_sealed())
        for directory in store.expected_object_directories():
            self.assertEqual(stat.S_IMODE(directory.stat().st_mode), 0o555)

    def test_existing_output_is_never_replaced_with_different_bytes(self) -> None:
        output = self.root / "receipt.json"
        original = b'{"protected": true}\n'
        output.write_bytes(original)
        with self.assertRaises(MODULE.EvidenceError):
            MODULE.atomic_json(output, {"different": True})
        self.assertEqual(output.read_bytes(), original)

    def test_concurrent_outputs_never_replace_the_winner(self) -> None:
        output = self.root / "receipt.json"
        barrier = threading.Barrier(2)
        successes: list[dict[str, int]] = []
        failures: list[Exception] = []

        def publish(payload: dict[str, int]) -> None:
            barrier.wait()
            try:
                MODULE.atomic_json(output, payload)
                successes.append(payload)
            except MODULE.EvidenceError as error:
                failures.append(error)

        threads = [
            threading.Thread(target=publish, args=({"writer": writer},))
            for writer in (1, 2)
        ]
        for thread in threads:
            thread.start()
        for thread in threads:
            thread.join()
        self.assertEqual(len(successes), 1)
        self.assertEqual(len(failures), 1)
        self.assertEqual(MODULE.json.loads(output.read_text(encoding="utf-8")), successes[0])

    def test_cli_output_is_confined_to_td103_evidence_names(self) -> None:
        with self.assertRaises(MODULE.EvidenceError):
            MODULE.validate_output_path(self.repo, Path("../victim.json"))

    def test_torn_journal_tail_is_truncated_before_next_append(self) -> None:
        store = self.new_store()
        store.journal("first")
        with store.journal_path.open("ab") as handle:
            handle.write(b'{"event":"torn"')
            handle.flush()
            os.fsync(handle.fileno())
        removed = store.repair_journal_tail()
        self.assertGreater(removed, 0)
        store.journal("after-repair")
        events = [record["event"] for record in store.journal_records()]
        self.assertEqual(events, ["first", "journal-tail-truncated", "after-repair"])

    def test_crash_after_backup_unlink_converges_and_clears_state(self) -> None:
        entry = self.entries[0]
        store = self.new_store()
        store.ensure_object(entry)
        store.fault = MODULE.FaultInjector("after-backup-unlink")
        with self.assertRaises(MODULE.InjectedFailure):
            store.externalize_entry(entry)
        source = store.source_path(entry)
        self.assertTrue(source.is_symlink())
        self.assertEqual(store.backup_state(entry), "removing")
        store.fault = MODULE.FaultInjector()
        self.assertEqual(store.externalize_entry(entry), "already-externalized")
        self.assertIsNone(store.backup_state(entry))
        self.assertEqual(store.materialize_entry(entry), "materialized")
        self.assertEqual(store.externalize_entry(entry), "externalized")

    def test_journal_intent_cannot_authorize_different_backup_inode(self) -> None:
        entry = self.entries[0]
        store = self.new_store()
        store.ensure_object(entry)
        source = store.source_path(entry)
        backup = source.with_name(f".{source.name}.td103-backup")
        store.journal("source-rename-intent", entry, backup=backup.name)
        source.parent.chmod(0o755)
        backup.write_bytes(source.read_bytes())
        backup.chmod(entry.mode)
        source.parent.chmod(0o555)
        with self.assertRaises(MODULE.EvidenceError):
            store.externalize_entry(entry)
        self.assertTrue(source.is_file())
        self.assertTrue(backup.is_file())

    def test_source_parent_symlink_is_rejected_before_staging(self) -> None:
        entry = self.entries[0]
        evidence = self.repo / "evidence"
        evidence.chmod(0o755)
        outside = self.root / "outside-evidence"
        evidence.rename(outside)
        os.symlink(outside, evidence, target_is_directory=True)
        store = self.new_store()
        with self.assertRaises(MODULE.EvidenceError):
            store.ensure_object(entry)
        self.assertFalse(store.object_path(entry).exists())

    def test_seal_rejects_unexpected_object_directory(self) -> None:
        store = self.new_store()
        store.stage(self.entries)
        extra = self.store_root / "objects" / "sha256" / "unexpected"
        extra.mkdir()
        with self.assertRaises(MODULE.EvidenceError):
            store.seal_objects()
        self.assertTrue(extra.is_dir())

    def test_catalog_backup_count_is_derived_and_must_be_consistent(self) -> None:
        self.store_root.mkdir()
        backed_up = [replace(entry, independent_backup_count_before=1) for entry in self.entries]
        catalog = MODULE.EvidenceStore(
            self.repo,
            self.store_root,
            backed_up,
            self.parent_contracts,
        ).catalog()
        self.assertEqual(catalog["independent_backup_count_before"], 1)
        inconsistent = [*backed_up]
        inconsistent[0] = replace(inconsistent[0], independent_backup_count_before=0)
        with self.assertRaises(MODULE.EvidenceError):
            MODULE.EvidenceStore(
                self.repo,
                self.store_root,
                inconsistent,
                self.parent_contracts,
            ).catalog()

    def test_inventory_rejects_mode_conflict_for_same_object(self) -> None:
        rows = self.inventory.read_text(encoding="utf-8").splitlines()
        fields = rows[2].split("\t")
        fields[2] = "555"
        invalid = self.repo / "mode-conflict.tsv"
        invalid.write_text("\n".join([rows[0], rows[1], "\t".join(fields)]) + "\n", encoding="utf-8")
        with self.assertRaises(MODULE.EvidenceError):
            MODULE.load_inventory(invalid)


if __name__ == "__main__":
    unittest.main()
