#!/usr/bin/env python3
"""Hermetic contracts for Lay's host-wide verification resource guard."""

from __future__ import annotations

import fcntl
import json
import os
import pathlib
import signal
import subprocess
import tempfile
import time
import unittest


ROOT = pathlib.Path(__file__).resolve().parents[1]
RESOURCE_GUARD = ROOT / "scripts" / "lay-resource-guard.sh"
CARGO_GUARD = ROOT / "scripts" / "cargo-guard.sh"


class ResourceGuardTests(unittest.TestCase):
    def setUp(self) -> None:
        self.temporary = tempfile.TemporaryDirectory(prefix="lay-resource-guard-test-")
        self.root = pathlib.Path(self.temporary.name)
        self.meminfo = self.root / "meminfo"
        self.memory_pressure = self.root / "memory.pressure"
        self.io_pressure = self.root / "io.pressure"
        self.lock_path = self.root / "lay-verification.lock"
        self.systemd_capture = self.root / "systemd-argv.txt"
        self.fake_systemd = self.root / "systemd-run"
        self.fake_systemd.write_text(
            """#!/usr/bin/env bash
set -euo pipefail
printf '%s\\n' "$@" > "$LAY_RESOURCE_TEST_SYSTEMD_CAPTURE"
while [[ "$#" -gt 0 ]]; do
  case "$1" in
    --user|--scope|--quiet|--collect|--unit=*) shift ;;
    -p|--property) shift 2 ;;
    *) exec "$@" ;;
  esac
done
exit 64
""",
            encoding="utf-8",
        )
        self.fake_systemd.chmod(0o755)
        self.base_nice = os.getpriority(os.PRIO_PROCESS, 0)
        self.write_host_state()

    def tearDown(self) -> None:
        self.temporary.cleanup()

    def write_host_state(
        self,
        *,
        available_kib: int = 16 * 1024 * 1024,
        swap_total_kib: int = 40 * 1024 * 1024,
        swap_free_kib: int = 30 * 1024 * 1024,
        memory_full_avg10: float = 0.0,
        io_full_avg10: float = 0.0,
    ) -> None:
        self.meminfo.write_text(
            "\n".join(
                (
                    "MemTotal:       31457280 kB",
                    f"MemAvailable:  {available_kib} kB",
                    f"SwapTotal:     {swap_total_kib} kB",
                    f"SwapFree:      {swap_free_kib} kB",
                    "",
                )
            ),
            encoding="utf-8",
        )
        self.memory_pressure.write_text(
            "some avg10=0.00 avg60=0.00 avg300=0.00 total=0\n"
            f"full avg10={memory_full_avg10:.2f} avg60=0.00 avg300=0.00 total=0\n",
            encoding="utf-8",
        )
        self.io_pressure.write_text(
            "some avg10=0.00 avg60=0.00 avg300=0.00 total=0\n"
            f"full avg10={io_full_avg10:.2f} avg60=0.00 avg300=0.00 total=0\n",
            encoding="utf-8",
        )

    def guard_environment(self) -> dict[str, str]:
        environment = {
            **{
                name: value
                for name, value in os.environ.items()
                if not name.startswith("LAY_RESOURCE_")
                and name not in {"CARGO_BUILD_JOBS", "RUST_TEST_THREADS", "CI"}
            },
            "LAY_RESOURCE_MEMINFO_PATH": str(self.meminfo),
            "LAY_RESOURCE_MEMORY_PRESSURE_PATH": str(self.memory_pressure),
            "LAY_RESOURCE_IO_PRESSURE_PATH": str(self.io_pressure),
            "LAY_RESOURCE_LOCK_PATH": str(self.lock_path),
            "LAY_RESOURCE_SYSTEMD_RUN": str(self.fake_systemd),
            "LAY_RESOURCE_TEST_SYSTEMD_CAPTURE": str(self.systemd_capture),
            "LAY_RESOURCE_TERM_GRACE_TICKS": "4",
            "LAY_RESOURCE_TERM_GRACE_SLEEP": "0.02",
        }
        return environment

    def run_guard(
        self,
        command: list[str],
        *,
        environment: dict[str, str] | None = None,
        timeout: float = 10,
    ) -> subprocess.CompletedProcess[str]:
        return subprocess.run(
            [str(RESOURCE_GUARD), "--", *command],
            cwd=ROOT,
            env=environment or self.guard_environment(),
            capture_output=True,
            text=True,
            check=False,
            timeout=timeout,
        )

    def test_pressure_preflight_fails_closed_before_spawn(self) -> None:
        cases = (
            (
                "memory_available",
                {"available_kib": 2 * 1024 * 1024},
                "reason=memory_available",
            ),
            (
                "swap_free",
                {"swap_total_kib": 8 * 1024 * 1024, "swap_free_kib": 512 * 1024},
                "reason=swap_free",
            ),
            (
                "memory_pressure",
                {"memory_full_avg10": 9.0},
                "reason=memory_pressure",
            ),
            (
                "io_pressure",
                {"io_full_avg10": 25.0},
                "reason=io_pressure",
            ),
        )
        for label, values, expected in cases:
            with self.subTest(label=label):
                self.systemd_capture.unlink(missing_ok=True)
                self.write_host_state(**values)
                marker = self.root / f"child-{label}"
                completed = self.run_guard(
                    ["/usr/bin/python3", "-c", "import pathlib,sys; pathlib.Path(sys.argv[1]).touch()", str(marker)]
                )
                self.assertEqual(75, completed.returncode, completed.stderr)
                self.assertIn(expected, completed.stderr)
                self.assertFalse(marker.exists())
                self.assertFalse(self.systemd_capture.exists())

    def test_swapless_host_does_not_fail_swap_preflight(self) -> None:
        self.write_host_state(swap_total_kib=0, swap_free_kib=0)
        completed = self.run_guard(["/usr/bin/true"])
        self.assertEqual(0, completed.returncode, completed.stderr)

    def test_preheld_host_lease_rejects_before_spawn(self) -> None:
        with self.lock_path.open("w", encoding="utf-8") as lease:
            fcntl.flock(lease.fileno(), fcntl.LOCK_EX | fcntl.LOCK_NB)
            completed = self.run_guard(["/usr/bin/true"])
        self.assertEqual(75, completed.returncode, completed.stderr)
        self.assertIn("reason=lock_busy", completed.stderr)
        self.assertFalse(self.systemd_capture.exists())

    def test_scope_has_complete_limits_and_child_environment(self) -> None:
        observed = self.root / "observed.json"
        program = """
import json, os, pathlib, sys
pathlib.Path(sys.argv[1]).write_text(json.dumps({
    'active': os.environ.get('LAY_RESOURCE_GUARD_ACTIVE'),
    'lease': os.environ.get('LAY_RESOURCE_LEASE_HELD'),
    'jobs': os.environ.get('CARGO_BUILD_JOBS'),
    'threads': os.environ.get('RUST_TEST_THREADS'),
    'nice': os.getpriority(os.PRIO_PROCESS, 0),
}), encoding='utf-8')
"""
        completed = self.run_guard(
            ["/usr/bin/python3", "-c", program, str(observed)]
        )
        self.assertEqual(0, completed.returncode, completed.stderr)
        payload = json.loads(observed.read_text(encoding="utf-8"))
        self.assertEqual(
            {
                "active": "1",
                "lease": "1",
                "jobs": "2",
                "threads": "1",
                "nice": min(19, self.base_nice + 10),
            },
            payload,
        )
        arguments = self.systemd_capture.read_text(encoding="utf-8").splitlines()
        for expected in (
            "CPUQuota=200%",
            "CPUWeight=10",
            "MemoryHigh=4G",
            "MemoryMax=6G",
            "MemorySwapMax=1G",
            "TasksMax=128",
            "IOWeight=10",
            "KillMode=control-group",
        ):
            self.assertIn(expected, arguments)
        # OOMPolicy is a service-unit setting. systemd 249 rejects every value
        # of that property for the transient scope used here.
        self.assertFalse(
            [argument for argument in arguments if argument.startswith("OOMPolicy=")],
            arguments,
        )
        self.assertIn("--scope", arguments)
        self.assertIn("--collect", arguments)
        self.assertIn("--inside", arguments)

    def test_ci_marker_does_not_disable_cgroup_scope(self) -> None:
        environment = self.guard_environment()
        environment["CI"] = "true"
        completed = self.run_guard(["/usr/bin/true"], environment=environment)
        self.assertEqual(0, completed.returncode, completed.stderr)
        self.assertIn("mode=scope", completed.stderr)
        self.assertTrue(self.systemd_capture.exists())

    def test_workstation_profile_rejects_twenty_jobs(self) -> None:
        environment = self.guard_environment()
        environment["CARGO_BUILD_JOBS"] = "20"
        completed = self.run_guard(["/usr/bin/true"], environment=environment)
        self.assertEqual(75, completed.returncode, completed.stderr)
        self.assertIn("reason=profile_jobs", completed.stderr)
        self.assertFalse(self.systemd_capture.exists())

    def test_dedicated_profile_scales_jobs_and_cgroup_together(self) -> None:
        fake_bin = self.root / "bin"
        fake_bin.mkdir()
        fake_nproc = fake_bin / "nproc"
        fake_nproc.write_text("#!/usr/bin/env bash\nprintf '20\\n'\n", encoding="utf-8")
        fake_nproc.chmod(0o755)
        environment = self.guard_environment()
        environment["PATH"] = f"{fake_bin}:{environment['PATH']}"
        environment["LAY_RESOURCE_PROFILE"] = "dedicated-20cpu"
        observed = self.root / "dedicated.json"
        program = (
            "import json,os,pathlib,sys; "
            "pathlib.Path(sys.argv[1]).write_text(json.dumps({"
            "'jobs':os.environ.get('CARGO_BUILD_JOBS'),"
            "'profile':os.environ.get('LAY_RESOURCE_PROFILE'),"
            "'nice':os.getpriority(os.PRIO_PROCESS,0)}),encoding='utf-8')"
        )
        completed = self.run_guard(
            ["/usr/bin/python3", "-c", program, str(observed)],
            environment=environment,
        )
        self.assertEqual(0, completed.returncode, completed.stderr)
        self.assertEqual(
            {"jobs": "20", "profile": "dedicated-20cpu", "nice": self.base_nice},
            json.loads(observed.read_text(encoding="utf-8")),
        )
        arguments = self.systemd_capture.read_text(encoding="utf-8").splitlines()
        for expected in (
            "CPUQuota=2000%",
            "CPUWeight=100",
            "MemoryHigh=24G",
            "MemoryMax=28G",
            "MemorySwapMax=1G",
            "TasksMax=512",
            "IOWeight=100",
        ):
            self.assertIn(expected, arguments)

    def test_normal_child_exit_status_is_preserved(self) -> None:
        completed = self.run_guard(["/usr/bin/python3", "-c", "raise SystemExit(37)"])
        self.assertEqual(37, completed.returncode, completed.stderr)

    def test_normal_leader_exit_cleans_leftover_descendant_group(self) -> None:
        descendant_pid_path = self.root / "descendant.pid"
        spawner = self.root / "spawner.sh"
        spawner.write_text(
            "#!/usr/bin/env bash\n"
            "sleep 60 &\n"
            "printf '%s\\n' \"$!\" > \"$1\"\n",
            encoding="utf-8",
        )
        spawner.chmod(0o755)
        completed = self.run_guard([str(spawner), str(descendant_pid_path)])
        self.assertEqual(0, completed.returncode, completed.stderr)
        descendant_pid = int(descendant_pid_path.read_text(encoding="utf-8"))
        deadline = time.monotonic() + 1.0
        while pathlib.Path(f"/proc/{descendant_pid}").exists() and time.monotonic() < deadline:
            time.sleep(0.01)
        self.assertFalse(pathlib.Path(f"/proc/{descendant_pid}").exists())

    def test_direct_mode_keeps_lease_jobs_and_priority_without_systemd(self) -> None:
        observed = self.root / "direct.json"
        environment = self.guard_environment()
        environment["LAY_RESOURCE_GUARD_MODE"] = "direct"
        program = """
import json, os, pathlib, sys
pathlib.Path(sys.argv[1]).write_text(json.dumps({
    'active': os.environ.get('LAY_RESOURCE_GUARD_ACTIVE'),
    'lease': os.environ.get('LAY_RESOURCE_LEASE_HELD'),
    'jobs': os.environ.get('CARGO_BUILD_JOBS'),
    'nice': os.getpriority(os.PRIO_PROCESS, 0),
}), encoding='utf-8')
"""
        completed = self.run_guard(
            ["/usr/bin/python3", "-c", program, str(observed)],
            environment=environment,
        )
        self.assertEqual(0, completed.returncode, completed.stderr)
        self.assertEqual(
            {
                "active": "1",
                "lease": "1",
                "jobs": "2",
                "nice": min(19, self.base_nice + 10),
            },
            json.loads(observed.read_text(encoding="utf-8")),
        )
        self.assertFalse(self.systemd_capture.exists())

    def test_signal_terminates_descendant_group_and_releases_lease(self) -> None:
        leader_pid_path = self.root / "leader.pid"
        descendant_pid_path = self.root / "signal-descendant.pid"
        spawner = self.root / "signal-spawner.sh"
        spawner.write_text(
            "#!/usr/bin/env bash\n"
            "printf '%s\\n' \"$$\" > \"$1\"\n"
            "sleep 60 &\n"
            "printf '%s\\n' \"$!\" > \"$2\"\n"
            "wait\n",
            encoding="utf-8",
        )
        spawner.chmod(0o755)
        process = subprocess.Popen(
            [str(RESOURCE_GUARD), "--", str(spawner), str(leader_pid_path), str(descendant_pid_path)],
            cwd=ROOT,
            env=self.guard_environment(),
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            text=True,
        )
        deadline = time.monotonic() + 3.0
        while not descendant_pid_path.exists() and time.monotonic() < deadline:
            time.sleep(0.01)
        self.assertTrue(descendant_pid_path.exists(), "synthetic descendant did not start")
        process.send_signal(signal.SIGTERM)
        _, stderr = process.communicate(timeout=5)
        self.assertEqual(143, process.returncode, stderr)
        pids = (
            int(leader_pid_path.read_text(encoding="utf-8")),
            int(descendant_pid_path.read_text(encoding="utf-8")),
        )
        deadline = time.monotonic() + 1.0
        while any(pathlib.Path(f"/proc/{pid}").exists() for pid in pids) and time.monotonic() < deadline:
            time.sleep(0.01)
        self.assertFalse([pid for pid in pids if pathlib.Path(f"/proc/{pid}").exists()])
        completed = self.run_guard(["/usr/bin/true"])
        self.assertEqual(0, completed.returncode, completed.stderr)

    @unittest.skipUnless(
        os.environ.get("LAY_RESOURCE_REAL_CGROUP_TEST") == "1",
        "explicit real user-cgroup integration only",
    )
    def test_real_cgroup_kills_new_session_descendant(self) -> None:
        descendant_pid_path = self.root / "escaped-descendant.pid"
        spawner = self.root / "escaped-spawner.sh"
        spawner.write_text(
            "#!/usr/bin/env bash\n"
            "setsid sleep 60 >/dev/null 2>&1 &\n"
            "printf '%s\\n' \"$!\" > \"$1\"\n",
            encoding="utf-8",
        )
        spawner.chmod(0o755)
        environment = {**os.environ, "LAY_RESOURCE_LOCK_PATH": str(self.lock_path)}
        for name in (
            "CI",
            "LAY_RESOURCE_GUARD_MODE",
            "LAY_RESOURCE_PROFILE",
            "LAY_RESOURCE_SYSTEMD_RUN",
            "LAY_RESOURCE_SYSTEMCTL",
        ):
            environment.pop(name, None)
        completed = self.run_guard(
            [str(spawner), str(descendant_pid_path)],
            environment=environment,
        )
        self.assertEqual(0, completed.returncode, completed.stderr)
        descendant_pid = int(descendant_pid_path.read_text(encoding="utf-8"))
        deadline = time.monotonic() + 2.0
        while pathlib.Path(f"/proc/{descendant_pid}").exists() and time.monotonic() < deadline:
            time.sleep(0.01)
        self.assertFalse(pathlib.Path(f"/proc/{descendant_pid}").exists())


class CargoGuardResourceTests(unittest.TestCase):
    def setUp(self) -> None:
        self.temporary = tempfile.TemporaryDirectory(prefix="lay-cargo-guard-test-")
        self.root = pathlib.Path(self.temporary.name)
        self.bin = self.root / "bin"
        self.bin.mkdir()
        self.marker = self.root / "cargo.json"
        self.lock_path = self.root / "lay-verification.lock"
        cargo = self.bin / "cargo"
        cargo.write_text(
            "#!/usr/bin/env bash\n"
            "if [[ -n \"${FAKE_CARGO_DESCENDANT_PID:-}\" ]]; then\n"
            "  sleep 30 </dev/null >/dev/null 2>&1 &\n"
            "  printf '%s\\n' \"$!\" > \"$FAKE_CARGO_DESCENDANT_PID\"\n"
            "  exit 0\n"
            "fi\n"
            "if [[ -n \"${FAKE_CARGO_HOLD_SECONDS:-}\" ]]; then\n"
            "  /usr/bin/sleep \"$FAKE_CARGO_HOLD_SECONDS\"\n"
            "fi\n"
            "python3 -c 'import json,os,pathlib; pathlib.Path(os.environ[\"FAKE_CARGO_MARKER\"]).write_text(json.dumps({\"jobs\": os.environ.get(\"CARGO_BUILD_JOBS\"), \"argv\": os.sys.argv[1:]}), encoding=\"utf-8\")' \"$@\"\n",
            encoding="utf-8",
        )
        cargo.chmod(0o755)
        sleep = self.bin / "sleep"
        sleep.write_text(
            "#!/usr/bin/env bash\n"
            "if [[ -n \"${FAKE_MONITOR_SLEEP_PID:-}\" ]] "
            "&& [[ \"${1:-}\" == \"${LAY_CARGO_TARGET_POLL_SECONDS:-}\" ]]; then\n"
            "  printf '%s\\n' \"$$\" > \"$FAKE_MONITOR_SLEEP_PID\"\n"
            "  if [[ -e /proc/$$/fd/8 ]]; then\n"
            "    printf 'open\\n' > \"$FAKE_MONITOR_SLEEP_FD8_STATE\"\n"
            "  else\n"
            "    printf 'closed\\n' > \"$FAKE_MONITOR_SLEEP_FD8_STATE\"\n"
            "  fi\n"
            "fi\n"
            "exec /usr/bin/sleep \"$@\" </dev/null >/dev/null 2>&1\n",
            encoding="utf-8",
        )
        sleep.chmod(0o755)

    def tearDown(self) -> None:
        self.temporary.cleanup()

    def environment(self) -> dict[str, str]:
        environment = {
            **{
                name: value
                for name, value in os.environ.items()
                if not name.startswith("LAY_RESOURCE_")
                and name not in {"CARGO_BUILD_JOBS", "RUST_TEST_THREADS", "CI"}
            },
            "PATH": f"{self.bin}:/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin",
            "CARGO_TARGET_DIR": str(self.root / "target"),
            "FAKE_CARGO_MARKER": str(self.marker),
            "LAY_RESOURCE_LOCK_PATH": str(self.lock_path),
            "LAY_CARGO_TARGET_POLL_SECONDS": "0.02",
            "LAY_RUST_TOOLCHAIN": "",
        }
        return environment

    def run_cargo_guard(
        self,
        environment: dict[str, str],
        arguments: list[str] | None = None,
    ) -> subprocess.CompletedProcess[str]:
        return subprocess.run(
            [str(CARGO_GUARD), *(arguments or ["check", "--locked"])],
            cwd=ROOT,
            env=environment,
            capture_output=True,
            text=True,
            check=False,
            timeout=10,
        )

    def terminate_and_wait(self, pid: int | None) -> None:
        if pid is None:
            return
        process_path = pathlib.Path(f"/proc/{pid}")
        try:
            os.kill(pid, signal.SIGTERM)
        except ProcessLookupError:
            return
        deadline = time.monotonic() + 2.0
        while process_path.exists() and time.monotonic() < deadline:
            time.sleep(0.01)
        if process_path.exists():
            try:
                os.kill(pid, signal.SIGKILL)
            except ProcessLookupError:
                return
            deadline = time.monotonic() + 2.0
            while process_path.exists() and time.monotonic() < deadline:
                time.sleep(0.01)
        self.assertFalse(
            process_path.exists(),
            f"synthetic descendant {pid} survived cleanup",
        )

    def test_cargo_guard_defaults_to_two_jobs(self) -> None:
        environment = self.environment()
        environment.pop("CARGO_BUILD_JOBS", None)
        completed = self.run_cargo_guard(environment)
        self.assertEqual(0, completed.returncode, completed.stderr)
        self.assertEqual("2", json.loads(self.marker.read_text(encoding="utf-8"))["jobs"])

    def test_cargo_guard_preserves_explicit_dedicated_host_jobs(self) -> None:
        environment = self.environment()
        environment["CARGO_BUILD_JOBS"] = "20"
        environment["LAY_RESOURCE_PROFILE"] = "dedicated-20cpu"
        environment["LAY_RESOURCE_GUARD_ACTIVE"] = "1"
        environment["LAY_RESOURCE_LEASE_HELD"] = "1"
        completed = self.run_cargo_guard(environment)
        self.assertEqual(0, completed.returncode, completed.stderr)
        self.assertEqual("20", json.loads(self.marker.read_text(encoding="utf-8"))["jobs"])

    def test_cargo_guard_rejects_unscoped_twenty_job_override(self) -> None:
        environment = self.environment()
        environment["CARGO_BUILD_JOBS"] = "20"
        completed = self.run_cargo_guard(environment)
        self.assertEqual(75, completed.returncode, completed.stderr)
        self.assertIn("reason=profile_jobs", completed.stderr)
        self.assertFalse(self.marker.exists())

    def test_cargo_guard_rejects_cli_job_ceiling_bypasses(self) -> None:
        for arguments in (
            ["check", "-j20"],
            ["check", "-j", "20"],
            ["check", "--jobs=20"],
            ["check", "--jobs", "20"],
            ["check", "--config", "build.jobs=20"],
            ["check", "--config=build.jobs=20"],
        ):
            with self.subTest(arguments=arguments):
                self.marker.unlink(missing_ok=True)
                completed = self.run_cargo_guard(self.environment(), arguments)
                self.assertEqual(75, completed.returncode, completed.stderr)
                self.assertFalse(self.marker.exists())

    def test_cargo_guard_uses_same_host_lease_and_fails_before_cargo(self) -> None:
        environment = self.environment()
        with self.lock_path.open("w", encoding="utf-8") as lease:
            fcntl.flock(lease.fileno(), fcntl.LOCK_EX | fcntl.LOCK_NB)
            completed = self.run_cargo_guard(environment)
        self.assertEqual(75, completed.returncode, completed.stderr)
        self.assertIn("reason=lock_busy", completed.stderr)
        self.assertFalse(self.marker.exists())

    def test_cargo_descendants_cannot_inherit_the_host_lease(self) -> None:
        descendant_pid_path = self.root / "descendant.pid"
        environment = self.environment()
        environment["FAKE_CARGO_DESCENDANT_PID"] = str(descendant_pid_path)

        descendant_pid = None
        try:
            completed = self.run_cargo_guard(environment)
            self.assertEqual(0, completed.returncode, completed.stderr)
            descendant_pid = int(descendant_pid_path.read_text(encoding="utf-8"))
            with self.lock_path.open("w", encoding="utf-8") as lease:
                fcntl.flock(lease.fileno(), fcntl.LOCK_EX | fcntl.LOCK_NB)
        finally:
            if descendant_pid is None and descendant_pid_path.exists():
                descendant_pid = int(descendant_pid_path.read_text(encoding="utf-8"))
            self.terminate_and_wait(descendant_pid)

    def test_budget_monitor_descendants_cannot_inherit_the_host_lease(self) -> None:
        monitor_sleep_pid_path = self.root / "monitor-sleep.pid"
        monitor_sleep_fd8_state_path = self.root / "monitor-sleep-fd8-state"
        environment = self.environment()
        environment["FAKE_CARGO_HOLD_SECONDS"] = "0.2"
        environment["FAKE_MONITOR_SLEEP_PID"] = str(monitor_sleep_pid_path)
        environment["FAKE_MONITOR_SLEEP_FD8_STATE"] = str(
            monitor_sleep_fd8_state_path
        )
        environment["LAY_CARGO_TARGET_POLL_SECONDS"] = "30"

        monitor_sleep_pid = None
        try:
            completed = self.run_cargo_guard(environment)
            self.assertEqual(0, completed.returncode, completed.stderr)
            monitor_sleep_pid = int(
                monitor_sleep_pid_path.read_text(encoding="utf-8")
            )
            self.assertEqual(
                "closed",
                monitor_sleep_fd8_state_path.read_text(encoding="utf-8").strip(),
            )
            with self.lock_path.open("w", encoding="utf-8") as lease:
                fcntl.flock(lease.fileno(), fcntl.LOCK_EX | fcntl.LOCK_NB)
        finally:
            if monitor_sleep_pid is None and monitor_sleep_pid_path.exists():
                monitor_sleep_pid = int(
                    monitor_sleep_pid_path.read_text(encoding="utf-8")
                )
            self.terminate_and_wait(monitor_sleep_pid)


class ResourceEntrypointContractTests(unittest.TestCase):
    def test_heavy_entrypoints_enter_one_resource_owner(self) -> None:
        changed = (ROOT / "scripts" / "check-lay-changed.sh").read_text(encoding="utf-8")
        full = (ROOT / "scripts" / "check-lay-full.sh").read_text(encoding="utf-8")
        lanes = (ROOT / "scripts" / "check-lay-tests.sh").read_text(encoding="utf-8")
        for source in (changed, full, lanes):
            self.assertIn("LAY_RESOURCE_GUARD_ACTIVE", source)
            self.assertIn("lay-resource-guard.sh", source)
        self.assertIn("self-test)", lanes)
        self.assertIn("live)", lanes)

    def test_changed_and_full_semantic_commands_remain_present(self) -> None:
        changed = (ROOT / "scripts" / "check-lay-changed.sh").read_text(encoding="utf-8")
        full = (ROOT / "scripts" / "check-lay-full.sh").read_text(encoding="utf-8")
        for marker in (
            "scripts/check-lay-tests.sh all",
            "cargo check --lib --bins",
            "--transition-replay",
            "--unsafe-gate",
        ):
            self.assertIn(marker, changed)
        for marker in (
            "scripts/check-lay-tests.sh all",
            "scripts/check-lay-lints.sh",
            "cargo build --release --bins --features research-tools",
        ):
            self.assertIn(marker, full)

    def test_guard_changes_run_resource_tests_and_ci_direct_mode_is_explicit(self) -> None:
        changed = (ROOT / "scripts" / "check-lay-changed.sh").read_text(encoding="utf-8")
        workflow = (ROOT / ".github" / "workflows" / "ci.yml").read_text(encoding="utf-8")
        for marker in (
            "lay-resource-guard",
            "cargo-guard",
            "resource_guard",
            "scripts/check-lay-tests.sh self-test",
        ):
            self.assertIn(marker, changed)
        self.assertIn("LAY_RESOURCE_GUARD_MODE: direct", workflow)

    def test_changed_shell_files_have_valid_syntax(self) -> None:
        completed = subprocess.run(
            [
                "bash",
                "-n",
                str(RESOURCE_GUARD),
                str(CARGO_GUARD),
                str(ROOT / "scripts" / "check-lay-changed.sh"),
                str(ROOT / "scripts" / "check-lay-full.sh"),
                str(ROOT / "scripts" / "check-lay-tests.sh"),
            ],
            cwd=ROOT,
            capture_output=True,
            text=True,
            check=False,
        )
        self.assertEqual(0, completed.returncode, completed.stderr)


if __name__ == "__main__":
    unittest.main(verbosity=2)
