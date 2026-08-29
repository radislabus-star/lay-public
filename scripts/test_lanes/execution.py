"""Run selected Rust test lanes in a filesystem and network sandbox."""

from __future__ import annotations

import hashlib
import os
import pathlib
import re
import shutil
import subprocess
import time
from typing import Any


ROOT = pathlib.Path(__file__).resolve().parents[2]
STATUS = re.compile(r"^test (.+) \.\.\. (ok|FAILED|ignored(?:,.*)?)$")
SUCCESS_SUMMARY = re.compile(
    r"^test result: ok\. 1 passed; 0 failed; 0 ignored;", re.MULTILINE
)


class ExecutionError(RuntimeError):
    pass


class PerformanceAssertionError(RuntimeError):
    pass


def clean_environment(sandbox: pathlib.Path, inherited: dict[str, str] | None = None) -> dict[str, str]:
    environment: dict[str, str] = {}
    home = sandbox / "home"
    config = sandbox / "config"
    data = sandbox / "data"
    cache = sandbox / "cache"
    runtime = sandbox / "runtime"
    for path in (home, config, data, cache, runtime):
        path.mkdir(parents=True, exist_ok=True)
    runtime.chmod(0o700)
    environment.update(
        {
            "PATH": "/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin",
            "HOME": str(home),
            "XDG_CONFIG_HOME": str(config),
            "XDG_DATA_HOME": str(data),
            "XDG_CACHE_HOME": str(cache),
            "XDG_RUNTIME_DIR": str(runtime),
            "CARGO_NET_OFFLINE": "true",
            "LAY_CONFIG_PATH": str(config / "lay" / "config.toml"),
            "LAY_LLM_BACKEND": "off",
            "RUST_TEST_THREADS": "1",
            "TMPDIR": "/tmp",
            "LANG": "C.UTF-8",
            "LC_ALL": "C.UTF-8",
            "TZ": "UTC",
            "http_proxy": "http://127.0.0.1:9",
            "https_proxy": "http://127.0.0.1:9",
            "ALL_PROXY": "http://127.0.0.1:9",
            "NO_PROXY": "",
        }
    )
    return environment


def sandbox_command(
    command: list[str], sandbox: pathlib.Path, real_home: pathlib.Path | None = None
) -> list[str]:
    bubblewrap = shutil.which("bwrap")
    if bubblewrap is None:
        raise ExecutionError("bubblewrap is required for hermetic test execution")
    empty = sandbox / "masked"
    empty.mkdir(parents=True, exist_ok=True)
    wrapped = [
        bubblewrap,
        "--die-with-parent",
        "--new-session",
        "--unshare-net",
        "--unshare-ipc",
        "--unshare-pid",
        "--ro-bind",
        "/",
        "/",
        "--dev",
        "/dev",
        "--proc",
        "/proc",
        "--tmpfs",
        "/run",
        "--tmpfs",
        "/tmp",
        "--bind",
        str(sandbox),
        str(sandbox),
        "--chdir",
        str(ROOT),
    ]
    home = real_home or pathlib.Path(os.environ.get("HOME", ""))
    for relative in (".local/share/lay", ".config/lay", ".cache/lay"):
        target = home / relative
        if home.is_absolute() and target.is_dir():
            wrapped.extend(("--ro-bind", str(empty), str(target)))
    return [*wrapped, *command]


def parse_statuses(output: str) -> dict[str, str]:
    statuses = {}
    for line in output.splitlines():
        match = STATUS.fullmatch(line)
        if match:
            statuses[match.group(1)] = match.group(2)
    return statuses


def performance_test_succeeded(output: str, test: str) -> bool:
    """Accept a one-test success even when --nocapture splits its status line."""
    status = parse_statuses(output).get(test)
    if status is not None:
        return status == "ok"
    return (
        "running 1 test" in output
        and f"test {test} ..." in output
        and SUCCESS_SUMMARY.search(output) is not None
    )


def failure_block(output: str, test: str) -> str:
    marker = f"---- {test} stdout ----"
    start = output.find(marker)
    if start < 0:
        return ""
    start += len(marker)
    candidates = [
        offset
        for token in ("\n---- ", "\nfailures:", "\ntest result:")
        if (offset := output.find(token, start)) >= 0
    ]
    end = min(candidates) if candidates else len(output)
    return output[start:end].strip()


def run_harness(
    artifact: dict[str, str],
    command: list[str],
    expected: set[str],
    sandbox: pathlib.Path,
    log_path: pathlib.Path,
) -> tuple[list[dict[str, str]], float]:
    environment = clean_environment(sandbox)
    started = time.monotonic()
    completed = subprocess.run(
        sandbox_command(command, sandbox),
        cwd=ROOT,
        env=environment,
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
        text=True,
        check=False,
    )
    elapsed = time.monotonic() - started
    log_path.parent.mkdir(parents=True, exist_ok=True)
    log_path.write_text(completed.stdout, encoding="utf-8")
    statuses = parse_statuses(completed.stdout)
    observed = {name for name in statuses if name in expected}
    if observed != expected:
        missing = sorted(expected - observed)
        raise ExecutionError(
            f"{artifact['target']}: harness status coverage mismatch; missing={missing[:20]}"
        )
    failures = [
        {
            "target": artifact["target"],
            "test": name,
            "failure_block": failure_block(completed.stdout, name),
        }
        for name in sorted(expected)
        if statuses[name] == "FAILED"
    ]
    if completed.returncode not in ({0} if not failures else {101}):
        raise ExecutionError(
            f"{artifact['target']}: unsupported harness exit {completed.returncode}"
        )
    return failures, elapsed


def partition_selected(
    selected: list[dict[str, str]],
) -> tuple[list[dict[str, str]], list[dict[str, str]]]:
    target = [row for row in selected if row["isolation"] == "target"]
    process = [row for row in selected if row["isolation"] == "process"]
    return target, process


def run_target(
    artifact: dict[str, str],
    selected: list[dict[str, str]],
    rows: list[dict[str, str]],
    sandbox: pathlib.Path,
    log_path: pathlib.Path,
) -> tuple[list[dict[str, str]], float]:
    bulk, isolated = partition_selected(selected)
    failures: list[dict[str, str]] = []
    elapsed = 0.0
    bulk_names = {row["name"] for row in bulk}
    if bulk:
        command = [artifact["executable"], "--test-threads=1"]
        for row in rows:
            if row["name"] not in bulk_names and row["lane"] != "ignored":
                command.extend(("--skip", row["name"]))
        observed, duration = run_harness(
            artifact,
            command,
            bulk_names,
            sandbox / "target",
            log_path,
        )
        failures.extend(observed)
        elapsed += duration
    for row in isolated:
        digest = hashlib.sha256(row["name"].encode("utf-8")).hexdigest()[:16]
        command = [
            artifact["executable"],
            "--exact",
            row["name"],
            "--test-threads=1",
        ]
        observed, duration = run_harness(
            artifact,
            command,
            {row["name"]},
            sandbox / "process" / digest,
            log_path.parent / "process" / f"{log_path.stem}-{digest}.log",
        )
        failures.extend(observed)
        elapsed += duration
    return failures, elapsed


def run_performance_test(
    artifact: dict[str, str], row: dict[str, str], sandbox: pathlib.Path, log_path: pathlib.Path
) -> float:
    environment = clean_environment(sandbox)
    environment["LAY_ENFORCE_IME_LATENCY_BUDGET"] = "1"
    environment["LAY_ENFORCE_CANONICAL_L2_FIELD_LATENCY_BUDGET"] = "1"
    environment["LAY_ENFORCE_DAEMON_LATENCY_BUDGET"] = "1"
    command = [
        artifact["executable"],
        "--exact",
        row["name"],
        "--nocapture",
        "--test-threads=1",
    ]
    started = time.monotonic()
    completed = subprocess.run(
        sandbox_command(command, sandbox),
        cwd=ROOT,
        env=environment,
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
        text=True,
        check=False,
    )
    elapsed = time.monotonic() - started
    log_path.parent.mkdir(parents=True, exist_ok=True)
    log_path.write_text(completed.stdout, encoding="utf-8")
    if completed.returncode != 0 or not performance_test_succeeded(
        completed.stdout, row["name"]
    ):
        raise PerformanceAssertionError(
            f"performance test failed: {row['target']}::{row['name']} (see {log_path})"
        )
    return elapsed
