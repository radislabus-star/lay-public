#!/usr/bin/env python3
"""Real-process regressions for the release L1.1 pidfd/peer guard."""

from __future__ import annotations

import hashlib
import importlib.util
import json
import os
import shutil
import signal
import subprocess
import sys
import tempfile
import time
import unittest
from pathlib import Path
from unittest import mock


ROOT = Path(__file__).resolve().parents[1]
GUARD = ROOT / "scripts" / "lay-release-l11-guard.py"
SERVER_CODE = r"""
import socket
import sys
import time

server = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
server.bind(sys.argv[1])
server.listen(1)
connection, _ = server.accept()
with connection:
    request = b""
    while not request.endswith(b"\n"):
        chunk = connection.recv(4096)
        if not chunk:
            break
        request += chunk
    connection.sendall(sys.argv[3].encode("utf-8") + b"\n")
    time.sleep(0.5)
"""


def proc_argv(pid: int) -> tuple[str, ...]:
    raw = Path(f"/proc/{pid}/cmdline").read_bytes()
    return tuple(os.fsdecode(field) for field in raw[:-1].split(b"\0"))


def proc_exe(pid: int) -> str:
    raw = os.readlink(f"/proc/{pid}/exe")
    if raw.endswith(" (deleted)") and not os.path.exists(raw):
        return raw[: -len(" (deleted)")]
    return raw


def proc_hash(pid: int) -> str:
    return hashlib.sha256(Path(f"/proc/{pid}/exe").read_bytes()).hexdigest()


def wait_for_process(pid: int, expected_argv: tuple[str, ...]) -> None:
    deadline = time.monotonic() + 3
    while time.monotonic() < deadline:
        try:
            if proc_argv(pid) == expected_argv:
                return
        except (FileNotFoundError, ProcessLookupError):
            break
        time.sleep(0.01)
    raise AssertionError(f"process {pid} did not reach the expected argv")


def stop_test_process(process: subprocess.Popen[bytes]) -> None:
    if process.poll() is None:
        process.kill()
    process.wait(timeout=3)


def spawn_bash(
    root: Path, *, ignore_term: bool = False, deleted_exe: bool = False,
    literal_deleted_suffix: bool = False
) -> tuple[subprocess.Popen[bytes], tuple[str, ...], str, str]:
    source = Path(shutil.which("bash") or "/bin/bash").resolve()
    executable = source
    if deleted_exe or literal_deleted_suffix:
        name = "bash (deleted)" if literal_deleted_suffix else "bash-copy"
        executable = root / name
        shutil.copy2(source, executable)
        executable.chmod(0o755)
    trap = "trap '' TERM" if ignore_term else "trap 'exit 0' TERM"
    script = f"{trap}; while :; do sleep 0.05; done"
    argv = (
        str(root / "captured-l11-argv0"),
        "-c",
        script,
        "run",
        "--memory",
        str(root / "package.bin"),
    )
    process = subprocess.Popen(
        argv,
        executable=str(executable),
        stdin=subprocess.DEVNULL,
        stdout=subprocess.DEVNULL,
        stderr=subprocess.DEVNULL,
    )
    wait_for_process(process.pid, argv)
    expected_exe = proc_exe(process.pid)
    expected_hash = proc_hash(process.pid)
    if deleted_exe:
        executable.unlink()
        deadline = time.monotonic() + 2
        while time.monotonic() < deadline:
            if os.readlink(f"/proc/{process.pid}/exe").endswith(" (deleted)"):
                break
            time.sleep(0.01)
        else:
            stop_test_process(process)
            raise AssertionError("deleted executable suffix was not observable")
    return process, argv, expected_exe, expected_hash


def guard_command(
    operation: str,
    pid: int,
    expected_exe: str,
    expected_hash: str,
    expected_argv: tuple[str, ...],
    *,
    timeout_ms: int = 1000,
    socket_path: Path | None = None,
    package_path: Path | None = None,
) -> list[str]:
    command = [
        sys.executable,
        str(GUARD),
        operation,
        "--pid",
        str(pid),
        "--expected-exe",
        expected_exe,
        "--expected-sha256",
        expected_hash,
        "--timeout-ms",
        str(timeout_ms),
    ]
    if operation == "health":
        assert socket_path is not None and package_path is not None
        command.extend(["--socket", str(socket_path), "--package", str(package_path)])
    return [*command, "--", *expected_argv]


def run_guard(command: list[str], *, timeout: float = 4) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        command,
        check=False,
        capture_output=True,
        text=True,
        timeout=timeout,
    )


def health_report(socket_path: Path, package_file: Path, **changes: object) -> str:
    report: dict[str, object] = {
        "status": "ready",
        "message": None,
        "socket": str(socket_path),
        "package_path": str(package_file),
        "package_bytes": package_file.stat().st_size,
        "terminal_count": 1,
        "manifest_generation": 0,
        "requests_served": 0,
        "uptime_ms": 1,
    }
    report.update(changes)
    return json.dumps({"type": "health", "report": report}, separators=(",", ":"))


def spawn_health_server(
    root: Path, socket_path: Path, package_path: Path, response: str
) -> tuple[subprocess.Popen[bytes], tuple[str, ...], str, str]:
    argv = (
        str(root / "health-server-argv0"),
        "-c",
        SERVER_CODE,
        str(socket_path),
        str(package_path),
        response,
    )
    process = subprocess.Popen(
        argv,
        executable=sys.executable,
        stdin=subprocess.DEVNULL,
        stdout=subprocess.DEVNULL,
        stderr=subprocess.DEVNULL,
    )
    wait_for_process(process.pid, argv)
    deadline = time.monotonic() + 3
    while time.monotonic() < deadline:
        if socket_path.exists():
            break
        if process.poll() is not None:
            raise AssertionError("health server exited before binding its socket")
        time.sleep(0.01)
    else:
        stop_test_process(process)
        raise AssertionError("health server did not bind its socket")
    return process, argv, proc_exe(process.pid), proc_hash(process.pid)


class ReleaseL11ProcessGuardTest(unittest.TestCase):
    def test_pidfd_terminates_current_deleted_and_literal_suffix_executables(self) -> None:
        for case in ("current", "deleted", "literal_deleted_suffix"):
            with self.subTest(case=case), tempfile.TemporaryDirectory(
                prefix="lay-l11-pidfd-"
            ) as temporary:
                root = Path(temporary)
                (root / "package.bin").write_bytes(b"package")
                process, argv, expected_exe, expected_hash = spawn_bash(
                    root,
                    deleted_exe=case == "deleted",
                    literal_deleted_suffix=case == "literal_deleted_suffix",
                )
                try:
                    result = run_guard(
                        guard_command(
                            "terminate",
                            process.pid,
                            expected_exe,
                            expected_hash,
                            argv,
                        )
                    )
                    self.assertEqual(0, result.returncode, result.stderr)
                    process.wait(timeout=2)
                finally:
                    stop_test_process(process)

    def test_argv_executable_and_hash_mismatches_never_signal(self) -> None:
        with tempfile.TemporaryDirectory(prefix="lay-l11-mismatch-") as temporary:
            root = Path(temporary)
            (root / "package.bin").write_bytes(b"package")
            process, argv, expected_exe, expected_hash = spawn_bash(root)
            try:
                variants = [("argv-5", argv[:-1]), ("argv-7", (*argv, "extra"))]
                variants.extend(
                    (f"argv-field-{index}", (*argv[:index], f"changed-{index}", *argv[index + 1 :]))
                    for index in range(6)
                )
                for label, changed_argv in variants:
                    with self.subTest(label=label):
                        result = run_guard(
                            guard_command(
                                "terminate",
                                process.pid,
                                expected_exe,
                                expected_hash,
                                changed_argv,
                            )
                        )
                        self.assertNotEqual(0, result.returncode)
                        self.assertIsNone(process.poll(), result.stderr)

                for label, changed_exe, changed_hash in (
                    ("exe", str(root / "other-exe"), expected_hash),
                    ("hash", expected_exe, "0" * 64),
                ):
                    with self.subTest(label=label):
                        result = run_guard(
                            guard_command(
                                "terminate",
                                process.pid,
                                changed_exe,
                                changed_hash,
                                argv,
                            )
                        )
                        self.assertNotEqual(0, result.returncode)
                        self.assertIsNone(process.poll(), result.stderr)
            finally:
                stop_test_process(process)

    def test_term_ignoring_child_returns_within_hard_budget(self) -> None:
        with tempfile.TemporaryDirectory(prefix="lay-l11-timeout-") as temporary:
            root = Path(temporary)
            (root / "package.bin").write_bytes(b"package")
            process, argv, expected_exe, expected_hash = spawn_bash(
                root, ignore_term=True
            )
            try:
                started = time.monotonic()
                result = run_guard(
                    guard_command(
                        "terminate",
                        process.pid,
                        expected_exe,
                        expected_hash,
                        argv,
                        timeout_ms=150,
                    )
                )
                elapsed = time.monotonic() - started
                self.assertEqual(4, result.returncode, result.stderr)
                self.assertLess(elapsed, 1.0)
                self.assertIsNone(process.poll())
            finally:
                stop_test_process(process)

    def test_pidfd_process_object_is_used_for_signal_and_poll(self) -> None:
        specification = importlib.util.spec_from_file_location("lay_l11_guard", GUARD)
        assert specification is not None and specification.loader is not None
        module = importlib.util.module_from_spec(specification)
        specification.loader.exec_module(module)

        poller = mock.Mock()
        poller.poll.return_value = [(41, 1)]
        with (
            mock.patch.object(module, "_open_pidfd", return_value=41),
            mock.patch.object(module, "_validate_process_identity") as validate,
            mock.patch.object(module.signal, "pidfd_send_signal") as send_signal,
            mock.patch.object(module.select, "poll", return_value=poller),
            mock.patch.object(module.os, "close") as close,
        ):
            module.terminate_process(123, "/expected", "a" * 64, tuple("abcdef"), 50)

        validate.assert_called_once_with(41, 123, "/expected", "a" * 64, tuple("abcdef"))
        send_signal.assert_called_once_with(41, signal.SIGTERM)
        poller.register.assert_called_once_with(41, module.select.POLLIN)
        poller.poll.assert_called_once_with(50)
        close.assert_called_once_with(41)

    def test_health_accepts_only_the_exact_peer_on_the_same_connection(self) -> None:
        with tempfile.TemporaryDirectory(prefix="lay-l11-health-") as temporary:
            root = Path(temporary)
            package = root / "package.bin"
            package.write_bytes(b"admitted-package")
            socket_path = root / "l11.sock"
            response = health_report(socket_path, package)
            server, argv, expected_exe, expected_hash = spawn_health_server(
                root, socket_path, package, response
            )
            try:
                result = run_guard(
                    guard_command(
                        "health",
                        server.pid,
                        expected_exe,
                        expected_hash,
                        argv,
                        socket_path=socket_path,
                        package_path=package,
                    )
                )
                self.assertEqual(0, result.returncode, result.stderr)
            finally:
                stop_test_process(server)

            socket_path.unlink(missing_ok=True)
            foreign, foreign_argv, foreign_exe, foreign_hash = spawn_health_server(
                root, socket_path, package, response
            )
            decoy, decoy_argv, decoy_exe, decoy_hash = spawn_bash(root)
            try:
                result = run_guard(
                    guard_command(
                        "health",
                        decoy.pid,
                        decoy_exe,
                        decoy_hash,
                        decoy_argv,
                        socket_path=socket_path,
                        package_path=package,
                    )
                )
                self.assertEqual(3, result.returncode, result.stderr)
                self.assertIn("health peer PID", result.stderr)
                self.assertIsNone(decoy.poll())
            finally:
                stop_test_process(foreign)
                stop_test_process(decoy)

    def test_health_rejects_same_size_package_replaced_after_process_start(self) -> None:
        with tempfile.TemporaryDirectory(prefix="lay-l11-health-package-age-") as temporary:
            root = Path(temporary)
            package = root / "package.bin"
            package.write_bytes(b"package-A")
            socket_path = root / "l11.sock"
            response = health_report(socket_path, package)
            server, argv, expected_exe, expected_hash = spawn_health_server(
                root, socket_path, package, response
            )
            try:
                time.sleep(0.01)
                replacement = root / "replacement.bin"
                replacement.write_bytes(b"package-B")
                os.replace(replacement, package)
                result = run_guard(
                    guard_command(
                        "health",
                        server.pid,
                        expected_exe,
                        expected_hash,
                        argv,
                        socket_path=socket_path,
                        package_path=package,
                    )
                )
                self.assertEqual(3, result.returncode, result.stderr)
                self.assertIn("package metadata changed", result.stderr)
            finally:
                stop_test_process(server)

    def test_malformed_or_wrong_health_and_package_bytes_are_rejected(self) -> None:
        with tempfile.TemporaryDirectory(prefix="lay-l11-health-bad-") as temporary:
            root = Path(temporary)
            package = root / "package.bin"
            package.write_bytes(b"admitted-package")
            cases = {
                "malformed": "not-json",
                "wrong-type": json.dumps({"type": "stats", "report": {}}),
                "wrong-status": health_report(
                    root / "status.sock", package, status="loading"
                ),
                "wrong-socket": health_report(
                    root / "unused.sock", package, socket=str(root / "foreign.sock")
                ),
                "wrong-package": health_report(
                    root / "package.sock", package, package_path=str(root / "other.bin")
                ),
                "wrong-bytes": health_report(
                    root / "bytes.sock", package, package_bytes=package.stat().st_size + 1
                ),
                "float-bytes": health_report(
                    root / "float.sock", package, package_bytes=float(package.stat().st_size)
                ),
                "bool-bytes": health_report(
                    root / "bool.sock", package, package_bytes=True
                ),
            }
            for label, response in cases.items():
                with self.subTest(label=label):
                    socket_path = root / f"{label}.sock"
                    if label == "wrong-status":
                        response = health_report(socket_path, package, status="loading")
                    elif label == "wrong-socket":
                        response = health_report(
                            socket_path, package, socket=str(root / "foreign.sock")
                        )
                    elif label == "wrong-package":
                        response = health_report(
                            socket_path, package, package_path=str(root / "other.bin")
                        )
                    elif label == "wrong-bytes":
                        response = health_report(
                            socket_path,
                            package,
                            package_bytes=package.stat().st_size + 1,
                        )
                    elif label == "float-bytes":
                        response = health_report(
                            socket_path,
                            package,
                            package_bytes=float(package.stat().st_size),
                        )
                    elif label == "bool-bytes":
                        response = health_report(
                            socket_path,
                            package,
                            package_bytes=True,
                        )
                    server, argv, expected_exe, expected_hash = spawn_health_server(
                        root, socket_path, package, response
                    )
                    try:
                        result = run_guard(
                            guard_command(
                                "health",
                                server.pid,
                                expected_exe,
                                expected_hash,
                                argv,
                                socket_path=socket_path,
                                package_path=package,
                            )
                        )
                        self.assertNotEqual(0, result.returncode)
                    finally:
                        stop_test_process(server)


if __name__ == "__main__":
    unittest.main()
