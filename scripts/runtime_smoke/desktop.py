from __future__ import annotations

import ast
import contextlib
import dataclasses
import os
import signal
import subprocess
import time
from pathlib import Path


@dataclasses.dataclass(frozen=True)
class ProcessIdentity:
    pid: int
    start_time: int
    executable: str
    argv: tuple[str, ...]


@dataclasses.dataclass(frozen=True)
class ServiceSnapshot:
    active_state: str
    sub_state: str
    unit_file_state: str
    main_pid: int
    main_process: ProcessIdentity | None


@dataclasses.dataclass(frozen=True)
class DesktopSnapshot:
    service: ServiceSnapshot
    active_layout: str
    active_engine: str
    ibus_daemons: tuple[ProcessIdentity, ...]
    lay_engines: tuple[ProcessIdentity, ...]
    lay_engine_trace_paths: tuple[str, ...]
    harness_trace_path: str | None


def same_process(left: ProcessIdentity, right: ProcessIdentity) -> bool:
    return left == right


def same_process_command(left: ProcessIdentity, right: ProcessIdentity) -> bool:
    return left.executable == right.executable and left.argv == right.argv


def read_process_identity(
    pid: int, proc_root: Path = Path("/proc")
) -> ProcessIdentity | None:
    process_root = proc_root / str(pid)
    try:
        if process_root.stat().st_uid != os.getuid():
            return None
        stat = (process_root / "stat").read_text(encoding="utf-8")
        after_name = stat.rsplit(") ", 1)[1].split()
        start_time = int(after_name[19])
        executable = os.readlink(process_root / "exe").removesuffix(" (deleted)")
        argv = tuple(
            value.decode("utf-8", errors="replace")
            for value in (process_root / "cmdline").read_bytes().split(b"\0")
            if value
        )
    except (FileNotFoundError, OSError, ValueError, IndexError):
        return None
    return ProcessIdentity(pid, start_time, executable, argv)


def discover_processes(
    executable_name: str, proc_root: Path = Path("/proc")
) -> tuple[ProcessIdentity, ...]:
    identities: list[ProcessIdentity] = []
    try:
        children = list(proc_root.iterdir())
    except OSError:
        return ()
    for child in children:
        if not child.name.isdigit():
            continue
        identity = read_process_identity(int(child.name), proc_root)
        if identity is None or Path(identity.executable).name != executable_name:
            continue
        identities.append(identity)
    return tuple(sorted(identities, key=lambda item: item.pid))


def discover_lay_ibus_engines(
    proc_root: Path = Path("/proc"),
) -> tuple[ProcessIdentity, ...]:
    return tuple(
        identity
        for identity in discover_processes("lay-ibus-engine", proc_root)
        if len(identity.argv) >= 2
        and Path(identity.argv[0]).name == "lay-ibus-engine"
        and identity.argv[1] == "--ibus"
    )


def read_process_environment(
    pid: int, proc_root: Path = Path("/proc")
) -> dict[str, str]:
    try:
        raw = (proc_root / str(pid) / "environ").read_bytes()
    except OSError as error:
        raise RuntimeError(f"could not read environment for PID {pid}") from error
    environment: dict[str, str] = {}
    for entry in raw.split(b"\0"):
        if not entry or b"=" not in entry:
            continue
        key, value = entry.split(b"=", 1)
        environment[key.decode("utf-8", errors="replace")] = value.decode(
            "utf-8", errors="replace"
        )
    return environment


def effective_engine_trace_path(
    identity: ProcessIdentity, proc_root: Path = Path("/proc")
) -> str:
    environment = read_process_environment(identity.pid, proc_root)
    configured = environment.get("LAY_IBUS_TRACE_PATH")
    if configured:
        return str(Path(configured).expanduser())
    home = environment.get("HOME") or str(Path.home())
    return str(Path(home) / ".local/share/lay/ibus_engine_debug.jsonl")


def engine_trace_paths(
    identities: tuple[ProcessIdentity, ...], proc_root: Path = Path("/proc")
) -> tuple[str, ...]:
    return tuple(effective_engine_trace_path(identity, proc_root) for identity in identities)


def process_is_current(
    expected: ProcessIdentity, proc_root: Path = Path("/proc")
) -> bool:
    current = read_process_identity(expected.pid, proc_root)
    return current is not None and same_process(expected, current)


def terminate_captured_process(
    expected: ProcessIdentity,
    timeout: float = 3.0,
    proc_root: Path = Path("/proc"),
) -> None:
    current = read_process_identity(expected.pid, proc_root)
    if current is None:
        return
    if not same_process(expected, current):
        raise RuntimeError(f"refusing to signal reused or changed PID {expected.pid}")
    os.kill(expected.pid, signal.SIGTERM)
    deadline = time.monotonic() + timeout
    while time.monotonic() < deadline:
        if not process_is_current(expected, proc_root):
            return
        time.sleep(0.05)
    if not process_is_current(expected, proc_root):
        return
    os.kill(expected.pid, signal.SIGKILL)
    deadline = time.monotonic() + timeout
    while time.monotonic() < deadline:
        if not process_is_current(expected, proc_root):
            return
        time.sleep(0.05)
    raise RuntimeError(f"captured PID {expected.pid} survived SIGKILL")


def systemctl(*args: str, check: bool = True) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        ["systemctl", "--user", *args],
        check=check,
        text=True,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
    )


def service_snapshot() -> ServiceSnapshot:
    result = systemctl(
        "show",
        "lay-daemon",
        "--property=ActiveState",
        "--property=SubState",
        "--property=UnitFileState",
        "--property=MainPID",
    )
    values = dict(
        line.split("=", 1)
        for line in result.stdout.splitlines()
        if "=" in line
    )
    main_pid = int(values.get("MainPID", "0"))
    main_process = read_process_identity(main_pid) if main_pid else None
    active_state = values.get("ActiveState", "unknown")
    if active_state == "active" and (
        main_process is None or Path(main_process.executable).name != "lay-daemon"
    ):
        raise RuntimeError("active lay-daemon MainPID identity is ambiguous")
    if active_state == "inactive" and (main_pid != 0 or main_process is not None):
        raise RuntimeError("inactive lay-daemon unexpectedly has a MainPID")
    return ServiceSnapshot(
        active_state=active_state,
        sub_state=values.get("SubState", "unknown"),
        unit_file_state=values.get("UnitFileState", "unknown"),
        main_pid=main_pid,
        main_process=main_process,
    )


def gnome_layout_call(method: str, *arguments: str) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        [
            "gdbus",
            "call",
            "--session",
            "--dest",
            "org.gnome.Shell",
            "--object-path",
            "/io/github/radislabus_star/LayDaemon",
            "--method",
            f"io.github.radislabus_star.LayDaemon.{method}",
            *arguments,
        ],
        check=False,
        text=True,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
    )


def current_desktop_layout() -> str:
    result = gnome_layout_call("CurrentLayout")
    if result.returncode != 0:
        raise RuntimeError(
            "GNOME current layout is unavailable: " + result.stderr.strip()
        )
    try:
        payload = ast.literal_eval(result.stdout.strip())
    except (SyntaxError, ValueError) as error:
        raise RuntimeError(
            f"could not parse GNOME current layout: {result.stdout!r}"
        ) from error
    if (
        not isinstance(payload, tuple)
        or len(payload) != 1
        or not isinstance(payload[0], str)
        or not payload[0]
    ):
        raise RuntimeError(f"ambiguous GNOME current layout: {payload!r}")
    return payload[0]


def set_desktop_layout(layout: str) -> None:
    if current_desktop_layout() == layout:
        return
    for _ in range(8):
        result = gnome_layout_call("ActivateLayout", layout)
        if result.returncode == 0 and current_desktop_layout() == layout:
            return
        time.sleep(0.15)
    raise RuntimeError(f"could not restore GNOME layout {layout!r}")


def current_ibus_engine() -> str:
    result = subprocess.run(
        ["ibus", "engine"],
        check=False,
        text=True,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
    )
    return result.stdout.strip()


def set_ibus_engine(engine: str) -> None:
    target = engine or "xkb:us::eng"
    if current_ibus_engine() == target:
        return
    for _ in range(8):
        subprocess.run(
            ["ibus", "engine", target],
            check=False,
            stdout=subprocess.DEVNULL,
            stderr=subprocess.DEVNULL,
        )
        if current_ibus_engine() == target:
            return
        time.sleep(0.15)
    raise RuntimeError(f"could not activate IBus engine {target!r}")


def capture_desktop_snapshot() -> DesktopSnapshot:
    active_layout = current_desktop_layout()
    active_engine = current_ibus_engine()
    if not active_engine:
        raise RuntimeError("active IBus engine is unknown; refusing desktop mutation")
    lay_engines = discover_lay_ibus_engines()
    return DesktopSnapshot(
        service=service_snapshot(),
        active_layout=active_layout,
        active_engine=active_engine,
        ibus_daemons=discover_processes("ibus-daemon"),
        lay_engines=lay_engines,
        lay_engine_trace_paths=engine_trace_paths(lay_engines),
        harness_trace_path=os.environ.get("LAY_IBUS_TRACE_PATH"),
    )


def verify_ibus_daemons_unchanged(expected: tuple[ProcessIdentity, ...]) -> None:
    current = discover_processes("ibus-daemon")
    if current != expected:
        raise RuntimeError("global ibus-daemon process identity changed during smoke")


def restore_service(snapshot: ServiceSnapshot) -> None:
    if snapshot.active_state == "active":
        systemctl("start", "lay-daemon")
    else:
        systemctl("stop", "lay-daemon")
    deadline = time.monotonic() + 3.0
    restored: ServiceSnapshot | None = None
    last_error: Exception | None = None
    while time.monotonic() < deadline:
        try:
            candidate = service_snapshot()
            if candidate.active_state == snapshot.active_state:
                restored = candidate
                break
        except RuntimeError as error:
            last_error = error
        time.sleep(0.05)
    if restored is None:
        if last_error is not None:
            raise RuntimeError("lay-daemon did not reach its restored state") from last_error
        raise RuntimeError("lay-daemon did not reach its restored state")
    if restored.active_state != snapshot.active_state:
        raise RuntimeError(
            "lay-daemon active state was not restored: "
            f"{restored.active_state!r} != {snapshot.active_state!r}"
        )
    if restored.unit_file_state != snapshot.unit_file_state:
        raise RuntimeError("lay-daemon unit-file state changed during smoke")
    if restored.sub_state != snapshot.sub_state:
        raise RuntimeError("lay-daemon sub-state was not restored")
    if snapshot.main_process is not None:
        if restored.main_process is None or not same_process_command(
            snapshot.main_process, restored.main_process
        ):
            raise RuntimeError("restored lay-daemon command identity changed")
    elif restored.main_process is not None:
        raise RuntimeError("inactive lay-daemon unexpectedly has a MainPID")


def verify_service_unchanged(snapshot: ServiceSnapshot) -> None:
    current = service_snapshot()
    if current.active_state != snapshot.active_state:
        raise RuntimeError("lay-daemon active state changed during smoke")
    if current.unit_file_state != snapshot.unit_file_state:
        raise RuntimeError("lay-daemon unit-file state changed during smoke")
    if current.sub_state != snapshot.sub_state:
        raise RuntimeError("lay-daemon sub-state changed during smoke")
    if current.main_process != snapshot.main_process:
        raise RuntimeError("unmanaged lay-daemon MainPID changed during smoke")


def validate_replacement_snapshot(
    snapshot: DesktopSnapshot,
    *,
    replace_service: bool,
    replace_lay_engines: bool,
) -> None:
    if replace_service and snapshot.service.active_state not in {"active", "inactive"}:
        raise RuntimeError(
            "lay-daemon is not in a replaceable active/inactive state: "
            f"{snapshot.service.active_state!r}"
        )
    if not replace_lay_engines:
        return
    active_is_lay = snapshot.active_engine.startswith("lay-ime-")
    if active_is_lay and len(snapshot.lay_engines) != 1:
        raise RuntimeError("active Lay IBus engine ownership is ambiguous")
    if not active_is_lay and snapshot.lay_engines:
        raise RuntimeError("inactive Lay IBus engine ownership is ambiguous")


def verify_lay_engines_restored(
    snapshot: DesktopSnapshot, *, replaced: bool
) -> None:
    current = discover_lay_ibus_engines()
    if not replaced:
        if current != snapshot.lay_engines:
            raise RuntimeError("unmanaged Lay IBus engine identity changed during smoke")
        if engine_trace_paths(current) != snapshot.lay_engine_trace_paths:
            raise RuntimeError("unmanaged Lay IBus trace destination changed")
        return
    if not snapshot.active_engine.startswith("lay-ime-"):
        if current:
            raise RuntimeError("Lay IBus engine remained after restoring an XKB engine")
        return
    if len(snapshot.lay_engines) != 1 or len(current) != 1:
        raise RuntimeError("Lay IBus engine cardinality was not restored")
    if not same_process_command(snapshot.lay_engines[0], current[0]):
        raise RuntimeError("restored Lay IBus engine command identity changed")
    if engine_trace_paths(current) != snapshot.lay_engine_trace_paths:
        raise RuntimeError("restored Lay IBus trace destination changed")


def verify_harness_trace_path_unchanged(expected: str | None) -> None:
    if os.environ.get("LAY_IBUS_TRACE_PATH") != expected:
        raise RuntimeError("global IBus trace destination changed during smoke")


def xkb_fallback_for(engine: str) -> str:
    return "xkb:ru::rus" if engine in {"lay-ime-ru", "xkb:ru::rus"} else "xkb:us::eng"


def run_cleanup_step(errors: list[str], label: str, action) -> None:
    try:
        action()
    except Exception as error:
        errors.append(f"{label}: {type(error).__name__}: {error}")


@contextlib.contextmanager
def managed_desktop_session(
    *,
    admitted: bool,
    replace_service: bool,
    replace_lay_engines: bool,
    snapshot: DesktopSnapshot | None = None,
):
    if not admitted:
        raise RuntimeError("managed desktop mutation was not admitted")
    snapshot = snapshot or capture_desktop_snapshot()
    validate_replacement_snapshot(
        snapshot,
        replace_service=replace_service,
        replace_lay_engines=replace_lay_engines,
    )
    try:
        if replace_service and snapshot.service.active_state == "active":
            verify_service_unchanged(snapshot.service)
            systemctl("stop", "lay-daemon")
        if replace_lay_engines:
            fallback = xkb_fallback_for(snapshot.active_engine)
            set_ibus_engine(fallback)
            for identity in snapshot.lay_engines:
                terminate_captured_process(identity)
        yield snapshot
    finally:
        errors: list[str] = []
        if replace_service:
            run_cleanup_step(
                errors, "service", lambda: restore_service(snapshot.service)
            )
        else:
            run_cleanup_step(
                errors,
                "service",
                lambda: verify_service_unchanged(snapshot.service),
            )
        run_cleanup_step(
            errors, "layout", lambda: set_desktop_layout(snapshot.active_layout)
        )
        run_cleanup_step(
            errors, "engine", lambda: set_ibus_engine(snapshot.active_engine)
        )
        run_cleanup_step(
            errors,
            "lay-engines",
            lambda: verify_lay_engines_restored(
                snapshot, replaced=replace_lay_engines
            ),
        )
        run_cleanup_step(
            errors,
            "ibus-daemon",
            lambda: verify_ibus_daemons_unchanged(snapshot.ibus_daemons),
        )
        run_cleanup_step(
            errors,
            "trace-path",
            lambda: verify_harness_trace_path_unchanged(
                snapshot.harness_trace_path
            ),
        )
        if errors:
            raise RuntimeError("desktop restoration failed: " + "; ".join(errors))
