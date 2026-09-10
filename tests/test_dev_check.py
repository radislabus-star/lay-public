"""Development orchestration contracts; subprocesses are mocked, never live."""
import importlib.util
import json
from pathlib import Path
import subprocess
import tarfile
import tempfile
import unittest
from unittest import mock

SCRIPT = Path(__file__).resolve().parents[1] / "scripts/dev-check.py"
SPEC = importlib.util.spec_from_file_location("lay_dev_check", SCRIPT)
dev = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(dev)


class PlannerTests(unittest.TestCase):
    def setUp(self):
        self.directory = tempfile.TemporaryDirectory()
        self.addCleanup(self.directory.cleanup)
        self.root = Path(self.directory.name)
        (self.root / "src/bin/lay_ibus_engine").mkdir(parents=True)
        (self.root / "tests").mkdir()
        (self.root / "Cargo.toml").write_text('[package]\nname="lay"\n')
        (self.root / "tests/one.rs").write_text("#[test] fn one() {}")
        (self.root / "tests/dynamic.rs").write_text("// dynamic source contract")

    def test_ime_includes_every_integration_consumer_not_only_literal_matches(self):
        result = dev.plan(["src/bin/lay_ibus_engine/engine.rs"], self.root)
        self.assertEqual("focused", result["mode"])
        self.assertEqual(["bin:lay-ibus-engine", "test:dynamic", "test:one"], result["targets"])

    def test_ime_deletion_keeps_owner_and_contracts(self):
        self.assertEqual("focused", dev.plan(["src/bin/lay_ibus_engine/deleted.rs"], self.root)["mode"])

    def test_shared_manifest_unknown_and_mixed_paths_broaden(self):
        for paths in (["src/lib.rs"], ["Cargo.toml"], ["tests/deleted.rs"],
                      ["some-new-tool"], ["src/bin/lay_ibus_engine.rs", "src/shared.rs"]):
            with self.subTest(paths=paths):
                self.assertEqual("all", dev.plan(paths, self.root)["mode"])

    def test_cross_binary_source_import_broadens(self):
        (self.root / "src/bin/other.rs").write_text('#[path="lay_ibus_engine/state.rs"] mod state;')
        self.assertEqual("all", dev.plan(["src/bin/lay_ibus_engine/state.rs"], self.root)["mode"])

    def test_dynamic_source_include_broadens(self):
        (self.root / "src/lib.rs").write_text('include!(concat!(env!("OUT_DIR"), "/code.rs"));')
        self.assertEqual("all", dev.plan(["src/bin/lay_ibus_engine/state.rs"], self.root)["mode"])

    def test_no_changes_is_not_a_test_pass(self):
        self.assertEqual("none", dev.plan([], self.root)["mode"])

    def test_git_change_detection_preserves_both_sides_and_nul_names(self):
        with mock.patch.object(dev, "git", side_effect=["old.rs\0new.rs\0with\nnewline.rs\0", "added.rs\0"] ) as call:
            self.assertEqual(["added.rs", "new.rs", "old.rs", "with\nnewline.rs"], dev.changed_paths(self.root))
        self.assertIn("--no-renames", call.call_args_list[0].args)
        self.assertNotIn("--diff-filter=ACMRTUXB", call.call_args_list[0].args)

    def test_snapshot_is_complete_and_sha_matches_extracted_bytes(self):
        names = ["Cargo.toml", "tests/one.rs", "tests/dynamic.rs"]
        archive = self.root / "source.tar"
        with mock.patch.object(dev, "file_list", return_value=names), mock.patch.object(dev, "git", return_value="abc123\n"), mock.patch.object(dev, "changed_paths", return_value=[]):
            receipt = dev.snapshot(self.root, archive, {"mode": "all", "source_head": "abc123", "changed": []})
        self.assertEqual(dev.digest(archive), receipt["archive_sha256"])
        with tarfile.open(archive) as bundle:
            self.assertEqual(names, bundle.getnames())
        self.assertEqual("abc123", receipt["source_head"])

    def test_snapshot_rejects_symlink_even_inside_repo(self):
        (self.root / "link").symlink_to("Cargo.toml")
        with mock.patch.object(dev, "file_list", return_value=["link"]), mock.patch.object(dev, "git", return_value="abc"), mock.patch.object(dev, "changed_paths", return_value=[]):
            with self.assertRaisesRegex(ValueError, "unsupported snapshot"):
                dev.snapshot(self.root, self.root / "source.tar", {"source_head": "abc", "changed": []})

    def test_snapshot_refuses_source_drift(self):
        with mock.patch.object(dev, "file_list", return_value=["Cargo.toml"]), mock.patch.object(dev, "digest", return_value="changed"), mock.patch.object(dev, "git", return_value="abc"), mock.patch.object(dev, "changed_paths", return_value=[]):
            with self.assertRaisesRegex(ValueError, "source changed"):
                dev.snapshot(self.root, self.root / "source.tar", {"source_head": "abc", "changed": []})

    def test_new_shared_edit_after_planning_refuses_before_archiving(self):
        with mock.patch.object(dev, "git", return_value="abc"), mock.patch.object(dev, "changed_paths", return_value=["src/lib.rs"]):
            with self.assertRaisesRegex(ValueError, "planning inputs changed"):
                dev.snapshot(self.root, self.root / "source.tar", {"source_head": "abc", "changed": []})
        self.assertFalse((self.root / "source.tar").exists())

    def test_head_change_after_planning_is_not_misattributed(self):
        with mock.patch.object(dev, "git", return_value="new-head"):
            with self.assertRaisesRegex(ValueError, "planning inputs changed"):
                dev.snapshot(self.root, self.root / "source.tar", {"source_head": "old-head", "changed": []})

    def test_changed_path_set_drift_during_snapshot_is_rejected(self):
        with mock.patch.object(dev, "git", return_value="abc"), mock.patch.object(dev, "changed_paths", side_effect=[[], ["new-shared.rs"]]), mock.patch.object(dev, "file_list", return_value=[]):
            with self.assertRaisesRegex(ValueError, "planning inputs changed"):
                dev.snapshot(self.root, self.root / "source.tar", {"source_head": "abc", "changed": []})

    def test_configuration_rejects_shell_options_and_broad_paths(self):
        config = self.root / "config.json"
        base = {"remote": "e@worker", "runs_dir": "/home/e/projects/lay-dev", "target_dir": "/home/e/projects/lay/target"}
        config.write_text(json.dumps(base))
        self.assertEqual(base, dev.load_config(config))
        for key, value in (("remote", "-oProxyCommand=bad"), ("remote", "host;touch bad"),
                           ("runs_dir", "/"), ("target_dir", "/home/e/../target"),
                           ("runs_dir", "/home/e/.cache/lay/development"),
                           ("target_dir", "/home/e/.local/share/lay/target")):
            config.write_text(json.dumps({**base, key: value}))
            with self.assertRaises(ValueError):
                dev.load_config(config)

    def test_same_machine_is_refused_before_execution(self):
        with mock.patch.object(Path, "read_text", return_value="machine\n"):
            with self.assertRaisesRegex(ValueError, "refusing local"):
                dev.assert_remote("machine")

    def test_worker_without_resource_guard_cannot_execute(self):
        request = self.root / "request.json"
        request.write_text(json.dumps({"origin_machine_id": "remote-origin", "target_dir": "/some/target"}))
        with mock.patch.object(dev, "assert_remote"), mock.patch.dict(dev.os.environ, {}, clear=True), mock.patch.object(subprocess, "run") as run:
            with self.assertRaisesRegex(ValueError, "resource guard"):
                dev.worker(request)
            run.assert_not_called()

    def test_existing_result_is_never_overwritten(self):
        output = self.root / "result.json"
        dev.write_json(output, {"first": True})
        with self.assertRaises(FileExistsError):
            dev.write_json(output, {"second": True})
        self.assertEqual({"first": True}, json.loads(output.read_text()))

    def test_existing_unowned_workspace_is_never_synced(self):
        run = self.root / "run-123"
        run.mkdir()
        workspace = self.root / "workspace"
        workspace.mkdir()
        with mock.patch.object(subprocess, "run") as command:
            with self.assertRaisesRegex(ValueError, "unowned"):
                dev.prepare_workspace({"workspace_dir": str(workspace)}, run / "request.json")
            command.assert_not_called()

    def test_ssh_uses_argument_array_and_quotes_remote_arguments(self):
        with mock.patch.object(subprocess, "run") as run:
            dev.ssh("e@worker", ["python3", "path with spaces", "x; echo unsafe"])
        argv = run.call_args.args[0]
        self.assertEqual("e@worker", argv[-2])
        self.assertIn("'x; echo unsafe'", argv[-1])
        self.assertNotIn("shell", run.call_args.kwargs)


if __name__ == "__main__":
    unittest.main()
