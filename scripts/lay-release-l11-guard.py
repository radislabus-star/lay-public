#!/usr/bin/env python3
"""Fail-closed process and health guard for the release-local L1.1 lifecycle."""

from __future__ import annotations

import argparse
import hashlib
import json
import os
import select
import signal
import socket
import stat
import struct
import sys
import time
from pathlib import Path


MAX_HEALTH_LINE_BYTES = 1 << 20
EXIT_MISMATCH = 3
EXIT_TIMEOUT = 4
EXIT_RUNTIME = 5


class GuardMismatch(Exception):
    """Observed process or service identity differs from the contract."""


class ProcessGone(GuardMismatch):
    """The pidfd-bound process exited before the operation completed."""


def _pidfd_pid(pidfd: int) -> int:
    try:
        lines = Path(f"/proc/self/fdinfo/{pidfd}").read_text(encoding="ascii").splitlines()
    except OSError as error:
        raise GuardMismatch(f"cannot read pidfd identity: {error}") from error
    for line in lines:
        if line.startswith("Pid:"):
            value = int(line.split(":", 1)[1].strip())
            if value < 0:
                raise ProcessGone
            return value
    raise GuardMismatch("pidfd identity has no Pid field")


def _read_exact_argv(pid: int) -> tuple[bytes, ...]:
    try:
        raw = Path(f"/proc/{pid}/cmdline").read_bytes()
    except FileNotFoundError as error:
        raise ProcessGone from error
    except OSError as error:
        raise GuardMismatch(f"cannot read process argv: {error}") from error
    if not raw.endswith(b"\0"):
        raise GuardMismatch("process argv is not NUL terminated")
    fields = tuple(raw[:-1].split(b"\0"))
    if len(fields) != 6 or any(not field for field in fields):
        raise GuardMismatch(f"process argv field count is {len(fields)}, expected 6")
    return fields


def _normalize_exe_path(raw_path: str) -> str:
    suffix = " (deleted)"
    if raw_path.endswith(suffix) and not os.path.exists(raw_path):
        return raw_path[: -len(suffix)]
    return raw_path


def _hash_proc_exe(pid: int) -> str:
    digest = hashlib.sha256()
    try:
        with open(f"/proc/{pid}/exe", "rb", buffering=0) as executable:
            while chunk := executable.read(1024 * 1024):
                digest.update(chunk)
    except FileNotFoundError as error:
        raise ProcessGone from error
    except OSError as error:
        raise GuardMismatch(f"cannot hash process executable: {error}") from error
    return digest.hexdigest()


def _validate_process_identity(
    pidfd: int,
    pid: int,
    expected_exe: str,
    expected_sha256: str,
    expected_argv: tuple[str, ...],
) -> None:
    if _pidfd_pid(pidfd) != pid:
        raise GuardMismatch("pidfd does not identify the expected PID")
    observed_argv = _read_exact_argv(pid)
    encoded_expected = tuple(os.fsencode(value) for value in expected_argv)
    if observed_argv != encoded_expected:
        raise GuardMismatch("process argv differs from the captured six-field invocation")
    try:
        raw_exe = os.readlink(f"/proc/{pid}/exe")
    except FileNotFoundError as error:
        raise ProcessGone from error
    except OSError as error:
        raise GuardMismatch(f"cannot read process executable path: {error}") from error
    if _normalize_exe_path(raw_exe) != expected_exe:
        raise GuardMismatch("process executable path differs from the expected path")
    if _hash_proc_exe(pid) != expected_sha256:
        raise GuardMismatch("process executable SHA-256 differs from the expected hash")
    if _pidfd_pid(pidfd) != pid:
        raise GuardMismatch("pidfd identity changed during process validation")


def _open_pidfd(pid: int) -> int:
    if not hasattr(os, "pidfd_open") or not hasattr(signal, "pidfd_send_signal"):
        raise RuntimeError("Python pidfd APIs are unavailable")
    try:
        return os.pidfd_open(pid, 0)
    except ProcessLookupError as error:
        raise ProcessGone from error


def _require_package_older_than_process(pid: int, package_stat: os.stat_result) -> None:
    try:
        process_stat = os.stat(f"/proc/{pid}", follow_symlinks=False)
    except FileNotFoundError as error:
        raise ProcessGone from error
    except OSError as error:
        raise GuardMismatch(f"cannot stat process identity: {error}") from error
    if package_stat.st_ctime_ns > process_stat.st_ctime_ns:
        raise GuardMismatch("package metadata changed after the process started")


def _same_file_snapshot(left: os.stat_result, right: os.stat_result) -> bool:
    fields = ("st_dev", "st_ino", "st_mode", "st_size", "st_mtime_ns", "st_ctime_ns")
    return all(getattr(left, field) == getattr(right, field) for field in fields)


def terminate_process(
    pid: int,
    expected_exe: str,
    expected_sha256: str,
    expected_argv: tuple[str, ...],
    timeout_ms: int,
) -> None:
    try:
        pidfd = _open_pidfd(pid)
    except ProcessGone:
        return
    try:
        try:
            _validate_process_identity(
                pidfd, pid, expected_exe, expected_sha256, expected_argv
            )
        except ProcessGone:
            return
        try:
            signal.pidfd_send_signal(pidfd, signal.SIGTERM)
        except ProcessLookupError:
            return
        poller = select.poll()
        poller.register(pidfd, select.POLLIN)
        if not poller.poll(timeout_ms):
            raise TimeoutError(f"process did not exit within {timeout_ms} ms")
    finally:
        os.close(pidfd)


def _remaining_seconds(deadline: float) -> float:
    remaining = deadline - time.monotonic()
    if remaining <= 0:
        raise TimeoutError("health deadline expired")
    return remaining


def _read_health_line(connection: socket.socket, deadline: float) -> bytes:
    chunks: list[bytes] = []
    received = 0
    while True:
        connection.settimeout(_remaining_seconds(deadline))
        chunk = connection.recv(min(65536, MAX_HEALTH_LINE_BYTES + 1 - received))
        if not chunk:
            raise GuardMismatch("health connection closed before a complete line")
        chunks.append(chunk)
        received += len(chunk)
        joined = b"".join(chunks)
        if b"\n" in chunk:
            line, remainder = joined.split(b"\n", 1)
            if remainder.strip():
                raise GuardMismatch("health response contains more than one JSON value")
            return line
        if received > MAX_HEALTH_LINE_BYTES:
            raise GuardMismatch("health response exceeds the bounded line limit")


def verify_health(
    pid: int,
    expected_exe: str,
    expected_sha256: str,
    expected_argv: tuple[str, ...],
    socket_path: str,
    package_path: str,
    timeout_ms: int,
) -> None:
    try:
        package_lstat = os.lstat(package_path)
    except OSError as error:
        raise GuardMismatch(f"cannot stat package: {error}") from error
    if stat.S_ISLNK(package_lstat.st_mode) or not stat.S_ISREG(package_lstat.st_mode):
        raise GuardMismatch("package path is not a regular non-symlink file")

    pidfd = _open_pidfd(pid)
    try:
        _validate_process_identity(pidfd, pid, expected_exe, expected_sha256, expected_argv)
        _require_package_older_than_process(pid, package_lstat)
        deadline = time.monotonic() + timeout_ms / 1000
        with socket.socket(socket.AF_UNIX, socket.SOCK_STREAM) as connection:
            connection.settimeout(_remaining_seconds(deadline))
            connection.connect(socket_path)
            peer_raw = connection.getsockopt(
                socket.SOL_SOCKET,
                socket.SO_PEERCRED,
                struct.calcsize("3i"),
            )
            peer_pid, _peer_uid, _peer_gid = struct.unpack("3i", peer_raw)
            if peer_pid != pid:
                raise GuardMismatch(
                    f"health peer PID is {peer_pid}, expected process PID {pid}"
                )
            connection.settimeout(_remaining_seconds(deadline))
            connection.sendall(b'{"type":"health"}\n')
            raw_line = _read_health_line(connection, deadline)
        try:
            response = json.loads(raw_line)
        except (UnicodeDecodeError, json.JSONDecodeError) as error:
            raise GuardMismatch(f"health response is not valid JSON: {error}") from error
        if not isinstance(response, dict):
            raise GuardMismatch("health response is not a JSON object")
        report = response.get("report")
        if response.get("type") != "health" or not isinstance(report, dict):
            raise GuardMismatch("health response type is invalid")
        package_bytes = report.get("package_bytes")
        if (
            report.get("status") != "ready"
            or report.get("socket") != socket_path
            or report.get("package_path") != package_path
            or type(package_bytes) is not int
            or package_bytes != package_lstat.st_size
        ):
            raise GuardMismatch("health report does not match socket/package contract")
        try:
            final_package_lstat = os.lstat(package_path)
        except OSError as error:
            raise GuardMismatch(f"cannot restat package: {error}") from error
        if not _same_file_snapshot(package_lstat, final_package_lstat):
            raise GuardMismatch("package identity changed during health verification")
        _validate_process_identity(pidfd, pid, expected_exe, expected_sha256, expected_argv)
    finally:
        os.close(pidfd)


def _parse_cli(raw_args: list[str]) -> tuple[argparse.Namespace, tuple[str, ...]]:
    try:
        separator = raw_args.index("--")
    except ValueError as error:
        raise GuardMismatch("six expected argv fields must follow --") from error
    expected_argv = tuple(raw_args[separator + 1 :])
    if len(expected_argv) != 6 or any(not value for value in expected_argv):
        raise GuardMismatch("exactly six non-empty expected argv fields are required")

    parser = argparse.ArgumentParser()
    subparsers = parser.add_subparsers(dest="command", required=True)
    for command in ("terminate", "health"):
        subparser = subparsers.add_parser(command)
        subparser.add_argument("--pid", type=int, required=True)
        subparser.add_argument("--expected-exe", required=True)
        subparser.add_argument("--expected-sha256", required=True)
        subparser.add_argument("--timeout-ms", type=int, required=True)
        if command == "health":
            subparser.add_argument("--socket", required=True)
            subparser.add_argument("--package", required=True)
    arguments = parser.parse_args(raw_args[:separator])
    if arguments.pid <= 0:
        raise GuardMismatch("PID must be positive")
    if not os.path.isabs(arguments.expected_exe):
        raise GuardMismatch("expected executable path must be absolute")
    if len(arguments.expected_sha256) != 64 or any(
        character not in "0123456789abcdef" for character in arguments.expected_sha256
    ):
        raise GuardMismatch("expected executable SHA-256 must be lowercase hexadecimal")
    if not 1 <= arguments.timeout_ms <= 30_000:
        raise GuardMismatch("timeout must be between 1 and 30000 ms")
    return arguments, expected_argv


def main(raw_args: list[str]) -> int:
    try:
        arguments, expected_argv = _parse_cli(raw_args)
        if arguments.command == "terminate":
            terminate_process(
                arguments.pid,
                arguments.expected_exe,
                arguments.expected_sha256,
                expected_argv,
                arguments.timeout_ms,
            )
        else:
            verify_health(
                arguments.pid,
                arguments.expected_exe,
                arguments.expected_sha256,
                expected_argv,
                arguments.socket,
                arguments.package,
                arguments.timeout_ms,
            )
        return 0
    except GuardMismatch as error:
        print(f"L1.1 guard mismatch: {error}", file=sys.stderr)
        return EXIT_MISMATCH
    except TimeoutError as error:
        print(f"L1.1 guard timeout: {error}", file=sys.stderr)
        return EXIT_TIMEOUT
    except (OSError, RuntimeError, ValueError) as error:
        print(f"L1.1 guard runtime failure: {error}", file=sys.stderr)
        return EXIT_RUNTIME


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
