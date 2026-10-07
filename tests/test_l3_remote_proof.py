"""Cold executor security/identity tests. Run only on the remote worker."""
import importlib.util
import json
import os
from pathlib import Path
import subprocess
import signal
import sys
import tempfile
import time
from types import SimpleNamespace
import unittest
from unittest import mock

ROOT = Path(__file__).resolve().parents[1]
spec = importlib.util.spec_from_file_location("l3_remote_proof", ROOT / "scripts/l3-full-proof-remote.py")
m = importlib.util.module_from_spec(spec)
spec.loader.exec_module(m)


class RemoteProofTests(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory()
        self.root = Path(self.temporary.name)
        self.origin = self.root / "origin"
        self.origin.mkdir()
        for name in ["base", "delta", "lexical", "canonical", "corpus", "surface", "usage_events"]:
            (self.origin / name).write_bytes((name + " data").encode())
        (self.origin / "manifest").write_text(json.dumps({
            "format": "lay-l3-composite-v1", "base": "base", "deltas": []}))
        self.args = SimpleNamespace(manifest=self.origin / "manifest", delta=self.origin / "delta",
                                    corpus=self.origin / "corpus", surface_evidence=self.origin / "surface",
                                    evaluator_sha256="a" * 64, max_fragments=80000, min_surface_support=2)
        self.dependencies = {role:self.origin / role for role in m.ENV_ROLES}
        self.env = mock.patch.dict(os.environ, {}, clear=True)
        self.env.start()

    def tearDown(self):
        self.env.stop()
        self.temporary.cleanup()

    def prepare(self):
        target = self.root / "job"
        target.mkdir()
        with mock.patch.object(m, "proof_dependencies", return_value=self.dependencies):
            request = m.prepare_inputs(self.args, target, {})
        return target, request

    def test_snapshot_binds_exact_optional_absence_and_read_only_inputs(self):
        target, request = self.prepare()
        m.verify_inputs(target, request)
        self.assertEqual((target / "inputs/lexical").stat().st_mode & 0o777, 0o400)
        self.assertFalse(request["roles"]["usage_counts"]["present"])
        (target / "inputs/usage_counts").write_text("unexpected worker state")
        with self.assertRaises(ValueError):
            m.verify_inputs(target, request)

    def test_mutated_immutable_origin_refuses_result_but_new_feedback_is_allowed(self):
        target, request = self.prepare()
        (self.origin / "usage_events").write_text("new feedback")
        m.verify_origin(request)
        (self.origin / "base").write_text("changed baseline")
        with self.assertRaises(ValueError):
            m.verify_origin(request)

    def test_current_manifest_mutation_refuses_even_when_referenced_base_is_unchanged(self):
        target, request = self.prepare()
        (self.origin / "manifest").write_text('{"format":"other"}')
        with self.assertRaises(ValueError):
            m.verify_origin(request)

    def test_tampered_transfer_refuses(self):
        target, request = self.prepare()
        (target / "inputs/delta").chmod(0o600)
        (target / "inputs/delta").write_text("different candidate")
        with self.assertRaises(ValueError):
            m.verify_inputs(target, request)

    def test_semantic_real_probe_switch_never_silently_disappears(self):
        os.environ["LAY_L3_REAL_L2_PROBE"] = "0"
        with self.assertRaises(ValueError):
            self.prepare()

    def test_remote_argv_is_shell_quoted_and_ssh_has_no_interactive_fallback(self):
        cfg = {"remote":"builder@worker.example"}
        with mock.patch.object(subprocess, "run") as run:
            m.ssh(cfg, ["python3", "path with spaces", "$(unwanted)"])
        argv = run.call_args.args[0]
        self.assertIn("BatchMode=yes", argv)
        self.assertEqual(argv[-1], "python3 'path with spaces' '$(unwanted)'")

    def test_config_refuses_ssh_options_unsafe_paths_and_unpinned_evaluator(self):
        path = self.root / "config.json"
        cfg = {"remote":"-oProxyCommand=bad", "runs_dir":"/worker/scoped/jobs",
               "binary":"/worker/scoped/bin/trainer", "resource_guard":"/worker/scoped/guard/run",
               "binary_sha256":"a" * 64, "guard_sha256":"b" * 64, "machine_id":"a" * 32}
        path.write_text(json.dumps(cfg))
        with self.assertRaises(ValueError):
            m.load_config(path)
        cfg["remote"] = "builder@worker.example"; cfg["binary"] = "/worker/scoped/../trainer"
        path.write_text(json.dumps(cfg))
        with self.assertRaises(ValueError):
            m.load_config(path)
        cfg["binary"] = "/worker/scoped/bin/trainer"; cfg["binary_sha256"] = "UNKNOWN"
        path.write_text(json.dumps(cfg))
        with self.assertRaises(ValueError):
            m.load_config(path)

    def test_worker_failure_cleans_all_sensitive_inputs(self):
        target, request = self.prepare()
        request["config"] = {"machine_id":"f" * 32}
        m.write_json(target / "request.json", request)
        with mock.patch.object(m.signal, "signal"), self.assertRaises(ValueError):
            m.worker(target)
        self.assertFalse((target / "inputs").exists())

    def test_readonly_snapshot_prevents_usage_cache_rewrite(self):
        target, request = self.prepare()
        inputs = target / "inputs"; inputs.chmod(0o500)
        try:
            with self.assertRaises(PermissionError):
                (inputs / "usage_counts").write_text("side effect")
        finally:
            m.clear_inputs(target)
        self.assertFalse(inputs.exists())

    def test_sigterm_runs_worker_cleanup_and_kills_its_evaluator(self):
        target, request = self.prepare()
        binary = self.root / "evaluator.py"
        binary.write_text('#!' + sys.executable + '\nimport os,time,pathlib\n'
                          + 'pathlib.Path(' + repr(str(target / "started")) + ').write_text(str(os.getpid()))\ntime.sleep(30)\n')
        binary.chmod(0o700)
        guard = self.root / "guard"; guard.write_text("fixture guard identity")
        request["evaluator_sha256"] = m.digest(binary)
        request["config"] = {"machine_id":Path("/etc/machine-id").read_text().strip(),
                             "binary":str(binary), "resource_guard":str(guard),
                             "guard_sha256":m.digest(guard)}
        m.write_json(target / "request.json", request)
        env = {"PATH":os.defpath, "LAY_RESOURCE_GUARD_ACTIVE":"1", "LAY_RESOURCE_LEASE_HELD":"1",
               "LAY_RESOURCE_PROFILE":"dedicated-20cpu"}
        log = (target / "worker.log").open("wb")
        process = subprocess.Popen([sys.executable, str(ROOT / "scripts/l3-full-proof-remote.py"),
                                    "_worker", str(target)], env=env, stdout=subprocess.DEVNULL, stderr=log)
        try:
            deadline = time.monotonic() + 5
            while not (target / "started").exists() and time.monotonic() < deadline:
                if process.poll() is not None:
                    log.flush()
                    details = (target / "failure.json").read_text() if (target / "failure.json").exists() else (target / "worker.log").read_text()
                    self.fail("worker exited before the signal scenario: " + details[-1200:])
                time.sleep(.02)
            self.assertTrue((target / "started").exists())
            evaluator_pid = int((target / "started").read_text())
            process.send_signal(signal.SIGTERM)
            self.assertEqual(process.wait(timeout=5), 128 + signal.SIGTERM)
            self.assertFalse((target / "inputs").exists())
            with self.assertRaises(ProcessLookupError):
                os.kill(evaluator_pid, 0)
        finally:
            if process.poll() is None:
                process.kill(); process.wait(timeout=5)
            log.close()


if __name__ == "__main__":
    unittest.main()
