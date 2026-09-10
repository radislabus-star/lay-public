#!/usr/bin/env python3
"""Exercise the public build entrypoint without compiling or installing a desktop."""

import fcntl
import json
import os
from pathlib import Path
import signal
import subprocess
import tempfile
import time
import unittest


ROOT = Path(__file__).resolve().parents[1]
BUILDER = ROOT / "scripts/build-release-binaries.sh"
INSTALLER = ROOT / "scripts/install-release-binaries.sh"
EXPECTED_BINARIES = {
    "lay", "lay-daemon", "lay-nanda-wave-eval", "lay-nanda-wave-train",
    "lay-test-input", "lay-ngram-corpus", "lay-ibus-engine", "lay-memory-report",
    "lay-l11-restore", "lay-l11-serve",
}


class PublicReleaseBuildTests(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory(prefix="lay-public-build-")
        self.addCleanup(self.temporary.cleanup)
        self.work = Path(self.temporary.name)
        self.fake = self.work / "commands"
        self.fake.mkdir()
        self.env = dict(os.environ)
        self.env.update(
            PATH=str(self.fake) + os.pathsep + self.env["PATH"],
            XDG_STATE_HOME=str(self.work / "state"),
            CARGO_TARGET_DIR=str(self.work / "target"),
            LAY_TEST_BUILD_RECORD=str(self.work / "cargo.json"),
            LAY_TEST_BUILD_RC="0",
            LAY_L2_OFFLINE="1",
            LAY_L2_PACKAGE_SOURCE=str(self.work / "absent-model.bin"),
            LAY_RESOURCE_GUARD_ACTIVE="0",
            LAY_RESOURCE_LEASE_HELD="0",
            LAY_RESOURCE_PROFILE="workstation",
            CARGO_BUILD_JOBS="2",
            LAY_RESOURCE_LOCK_PATH=str(self.work / "guard.lock"),
        )
        cargo = self.fake / "cargo"
        cargo.write_text(
            "#!/usr/bin/env python3\n"
            "import json, os, pathlib, signal, subprocess, sys\n"
            "pathlib.Path(os.environ['LAY_TEST_BUILD_RECORD']).write_text(\n"
            "    json.dumps({'args': sys.argv[1:], 'cwd': os.getcwd()}))\n"
            "print('Compiling public build fixture', file=sys.stderr, flush=True)\n"
            "print('warning: preserved diagnostic', file=sys.stderr, flush=True)\n"
            "if os.environ.get('LAY_TEST_BUILD_WAIT'):\n"
            "    child = subprocess.Popen(['sleep', '30'])\n"
            "    record = pathlib.Path(os.environ['LAY_TEST_BUILD_WAIT'])\n"
            "    record.write_text(json.dumps([os.getpid(), child.pid]))\n"
            "    child.wait()\n"
            "if os.environ.get('LAY_TEST_BUILD_SIGNAL'):\n"
            "    os.kill(os.getpid(), signal.SIGTERM)\n"
            "code = int(os.environ['LAY_TEST_BUILD_RC'])\n"
            "print('Finished fixture' if code == 0 else 'error: compiler fixture',\n"
            "      file=sys.stderr, flush=True)\n"
            "sys.exit(code)\n"
        )
        cargo.chmod(0o755)

    def build(self):
        # The same fail-fast shell contract as install.sh; a failed build must
        # not allow the following installation stage to run.
        return subprocess.run(
            ["bash", "-c", 'set -e; "$1"; touch "$2"', "_",
             str(BUILDER), str(self.work / "next-install-stage")],
            cwd=self.work, env=self.env, text=True, capture_output=True, timeout=30,
        )

    def transcript(self):
        logs = list((self.work / "state/lay").glob("build-release-*.log"))
        self.assertEqual(len(logs), 1)
        return logs[0], logs[0].read_text()

    def test_only_the_complete_installed_binary_set_is_built_with_progress(self):
        result = self.build()
        self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
        record = json.loads((self.work / "cargo.json").read_text())
        arguments = record["args"]
        selected = [arguments[i + 1] for i, value in enumerate(arguments) if value == "--bin"]
        self.assertEqual(set(selected), EXPECTED_BINARIES)
        self.assertEqual(len(selected), len(EXPECTED_BINARIES))
        self.assertIn("--release", arguments)
        self.assertIn("--locked", arguments)
        self.assertEqual(arguments[arguments.index("--features") + 1], "research-tools")
        self.assertNotIn("--quiet", arguments)
        self.assertNotIn("--bins", arguments)
        self.assertEqual(record["cwd"], str(ROOT))
        log, content = self.transcript()
        self.assertIn("Compiling public build fixture", result.stdout)
        self.assertIn("Finished fixture", result.stdout)
        self.assertIn("warning: preserved diagnostic", content)
        self.assertIn(str(log), result.stdout)
        self.assertTrue((self.work / "next-install-stage").exists())

    def test_compiler_failure_keeps_diagnostic_exit_status_and_stops_install(self):
        self.env["LAY_TEST_BUILD_RC"] = "101"
        result = self.build()
        self.assertEqual(result.returncode, 101, result.stdout + result.stderr)
        log, content = self.transcript()
        self.assertIn("error: compiler fixture", content)
        self.assertIn("101", result.stderr)
        self.assertIn(str(log), result.stderr)
        self.assertFalse((self.work / "next-install-stage").exists())

    def test_terminated_compiler_is_reported_and_stops_install(self):
        self.env["LAY_TEST_BUILD_SIGNAL"] = "TERM"
        result = self.build()
        self.assertEqual(result.returncode, 143, result.stdout + result.stderr)
        log, content = self.transcript()
        self.assertIn("warning: preserved diagnostic", content)
        self.assertIn("143", result.stderr)
        self.assertIn(str(log), result.stderr)
        self.assertFalse((self.work / "next-install-stage").exists())

    def test_transcript_write_failure_is_not_a_successful_install(self):
        tee = self.fake / "tee"
        tee.write_text(
            "#!/usr/bin/env python3\n"
            "import sys\n"
            "sys.stdout.write(sys.stdin.read())\n"
            "sys.exit(74)\n"
        )
        tee.chmod(0o755)
        result = self.build()
        self.assertEqual(result.returncode, 74, result.stdout + result.stderr)
        self.assertIn("Не удалось сохранить журнал", result.stderr)
        self.assertFalse((self.work / "next-install-stage").exists())

    def test_wrapper_cancellation_stops_compiler_descendants_and_releases_lease(self):
        def foreground_signals():
            # A guarded test runner can inherit ignored SIGINT. A terminal
            # foreground job starts with its default signal disposition.
            signal.signal(signal.SIGINT, signal.SIG_DFL)

        def running(pid):
            try:
                # An exited child awaiting its external reaper has no remaining
                # execution or open descriptors; do not mistake it for a worker.
                return Path(f"/proc/{pid}/stat").read_text().split(") ", 1)[1][0] != "Z"
            except FileNotFoundError:
                return False

        for cancellation, whole_group in [(signal.SIGTERM, False),
                                           (signal.SIGHUP, False),
                                           (signal.SIGINT, True)]:
            with self.subTest(signal=cancellation, whole_group=whole_group):
                ready = self.work / f"workers-{cancellation}.json"
                env = {**self.env, "LAY_TEST_BUILD_WAIT": str(ready)}
                process = subprocess.Popen(
                    [str(BUILDER)], cwd=self.work, env=env, text=True,
                    stdout=subprocess.PIPE, stderr=subprocess.PIPE,
                    start_new_session=True,
                    preexec_fn=foreground_signals,
                )
                workers = []
                try:
                    deadline = time.monotonic() + 5
                    while time.monotonic() < deadline:
                        if ready.exists() and ready.stat().st_size:
                            workers = json.loads(ready.read_text())
                            break
                        time.sleep(0.02)
                    self.assertEqual(len(workers), 2, "compiler and child must reach readiness")
                    with (self.work / "guard.lock").open("a") as lease:
                        with self.assertRaises(BlockingIOError):
                            fcntl.flock(lease, fcntl.LOCK_EX | fcntl.LOCK_NB)
                    if whole_group:
                        os.killpg(process.pid, cancellation)
                    else:
                        process.send_signal(cancellation)
                    stdout, stderr = process.communicate(timeout=6)
                    self.assertEqual(process.returncode, 128 + cancellation, stdout + stderr)
                    self.assertFalse(any(running(pid) for pid in workers), workers)
                    with (self.work / "guard.lock").open("a") as lease:
                        fcntl.flock(lease, fcntl.LOCK_EX | fcntl.LOCK_NB)
                    self.assertIn("Журнал:", stderr)
                    self.assertTrue(any("warning: preserved diagnostic" in path.read_text()
                                        for path in (self.work / "state/lay").glob("*.log")))
                finally:
                    # This also makes a failing baseline test safe: the real
                    # guard deliberately places Cargo in a separate session.
                    for pgid in [process.pid, *workers[:1]]:
                        try:
                            os.killpg(pgid, signal.SIGKILL)
                        except ProcessLookupError:
                            pass
                    process.communicate(timeout=5)

    def test_binary_listing_does_not_resolve_models_or_install_files(self):
        self.env.update(
            LAY_INSTALL_LIBEXEC_DIR=str(self.work / "installed"),
            LAY_INSTALL_BIN_DIR=str(self.work / "links"),
            LAY_L2_MODEL_DIR=str(self.work / "models"),
        )
        result = subprocess.run(
            [str(INSTALLER), "--list-source-binaries"], cwd=self.work,
            env=self.env, text=True, capture_output=True, timeout=10,
        )
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(set(result.stdout.splitlines()), EXPECTED_BINARIES)
        for name in ["installed", "links", "models", "cargo.json"]:
            self.assertFalse((self.work / name).exists())

    def test_required_json_tool_is_declared_for_each_package_manager(self):
        for manager in ["apt", "pacman", "rpm-ostree", "dnf", "yum"]:
            with self.subTest(manager=manager):
                env = {**self.env, "LAY_PACKAGE_MANAGER_OVERRIDE": manager}
                result = subprocess.run(
                    ["bash", str(ROOT / "install.sh"), "--check-platform"],
                    env=env, text=True, capture_output=True, timeout=10,
                )
                self.assertEqual(result.returncode, 0, result.stderr)
                packages = next(line for line in result.stdout.splitlines()
                                if line.startswith("base_packages="))
                self.assertIn("jq", packages.partition("=")[2].split())

    def test_bootstrap_requests_json_tool_before_checkout_on_clean_systems(self):
        source = (ROOT / "scripts/install-remote.sh").read_text()
        self.assertTrue(source.endswith('main "$@"\n'))
        # Source the actual bootstrap functions; replace the external package
        # manager boundary so these tests never invoke sudo or install packages.
        definitions = source.removesuffix('main "$@"\n')
        script = definitions + '''
sudo() { printf '%s\n' "$*" >> "$LAY_TEST_PACKAGE_CALLS"; }
kde_available() { return 1; }
rpm_ostree_dependencies_available() { return 1; }
install_system_packages
'''
        for manager in ["apt", "pacman", "rpm-ostree", "dnf", "yum"]:
            with self.subTest(manager=manager):
                calls = self.work / ("packages-" + manager)
                env = {**self.env, "LAY_PACKAGE_MANAGER_OVERRIDE": manager,
                       "LAY_TEST_PACKAGE_CALLS": str(calls)}
                result = subprocess.run(
                    ["bash", "-c", script], env=env, text=True,
                    capture_output=True, timeout=10,
                )
                self.assertEqual(result.returncode, 0, result.stderr)
                self.assertIn("jq", calls.read_text().split())


if __name__ == "__main__":
    unittest.main(verbosity=2)
