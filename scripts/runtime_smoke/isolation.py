from __future__ import annotations

import contextlib
import dataclasses
import hashlib
import os
import signal
import subprocess
from pathlib import Path


@dataclasses.dataclass(frozen=True)
class CaseContext:
    run_id: str
    case_id: str
    directory: Path
    config_path: Path
    trace_path: Path

    def environment(self) -> dict[str, str]:
        return {
            **os.environ,
            "LAY_RUNTIME_SMOKE_RUN_ID": self.run_id,
            "LAY_RUNTIME_SMOKE_CASE_ID": self.case_id,
            "LAY_CONFIG_PATH": str(self.config_path),
            "LAY_NANDA_WORD_USAGE_EVENTS": str(self.directory / "events.jsonl"),
            "LAY_NANDA_WORD_USAGE_COUNTS": str(self.directory / "counts.json"),
            "LAY_NANDA_WORD_USAGE_FEEDBACK_COUNTS": str(
                self.directory / "feedback-counts.json"
            ),
            "LAY_IBUS_TRACE_PATH": str(self.trace_path),
        }


OWNERSHIP_SIGNALS = (signal.SIGINT, signal.SIGTERM, signal.SIGHUP)


@contextlib.contextmanager
def defer_ownership_signals():
    previous: dict[signal.Signals, object] = {}
    pending: list[signal.Signals] = []

    def defer(signum, _frame):
        pending.append(signal.Signals(signum))

    for name in OWNERSHIP_SIGNALS:
        previous[name] = signal.getsignal(name)
        signal.signal(name, defer)
    try:
        yield
    finally:
        for name, handler in previous.items():
            signal.signal(name, handler)
        if pending:
            handler = previous[pending[0]]
            if callable(handler):
                handler(pending[0], None)
            raise SmokeInterrupted(
                f"runtime smoke interrupted by signal {pending[0].value}"
            )


class ProcessSupervisor:
    def __init__(self) -> None:
        self._children: list[tuple[str, subprocess.Popen[str]]] = []

    def track(self, role: str, process):
        self._children.append((role, process))
        return process

    def spawn(self, role: str, args, **kwargs):
        # A pending signal may run as soon as the mask is restored. Record the
        # child first so stack unwinding always reaches an owned process handle.
        with defer_ownership_signals():
            process = subprocess.Popen(args, **kwargs)
            return self.track(role, process)

    def stop(self, process, timeout: float = 3.0) -> tuple[str, str]:
        if process.poll() is not None:
            return process.communicate(timeout=timeout)
        try:
            process.terminate()
        except ProcessLookupError:
            return process.communicate(timeout=timeout)
        try:
            return process.communicate(timeout=timeout)
        except subprocess.TimeoutExpired:
            try:
                process.kill()
            except ProcessLookupError:
                pass
            try:
                return process.communicate(timeout=timeout)
            except subprocess.TimeoutExpired as error:
                raise RuntimeError("child pipes remained open after SIGKILL") from error

    def close(self) -> None:
        errors: list[str] = []
        for role, process in reversed(self._children):
            if process.poll() is None:
                try:
                    self.stop(process)
                except Exception as error:
                    errors.append(f"{role}: {type(error).__name__}: {error}")
        if errors:
            raise RuntimeError("child cleanup failed: " + "; ".join(errors))

    def __enter__(self):
        return self

    def __exit__(self, _kind, _error, _traceback) -> None:
        self.close()


def case_id_for(run_id: str, case_name: str) -> str:
    value = hashlib.sha256(f"{run_id}\0{case_name}".encode()).hexdigest()[:20]
    return f"{case_name}-{value}"


def prepare_case_context(root: Path, run_id: str, case_name: str) -> CaseContext:
    case_id = case_id_for(run_id, case_name)
    directory = root / case_id
    directory.mkdir(parents=True, exist_ok=False)
    return CaseContext(
        run_id=run_id,
        case_id=case_id,
        directory=directory,
        config_path=directory / "config.json",
        trace_path=directory / "ibus_engine_debug.jsonl",
    )


class SmokeInterrupted(RuntimeError):
    pass


class CleanupSignalHandlers:
    def __init__(self) -> None:
        self.previous: dict[signal.Signals, object] = {}

    def __enter__(self):
        def interrupt(signum, _frame):
            raise SmokeInterrupted(f"runtime smoke interrupted by signal {signum}")

        for name in OWNERSHIP_SIGNALS:
            self.previous[name] = signal.getsignal(name)
            signal.signal(name, interrupt)
        return self

    def __exit__(self, _kind, _error, _traceback) -> None:
        for name, handler in self.previous.items():
            signal.signal(name, handler)
