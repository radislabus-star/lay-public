#!/usr/bin/env python3
"""One-build exact V10 fused-band transition M1 microproof controller."""

from __future__ import annotations

import argparse
import contextlib
import importlib.util
import json
import os
import pathlib
import re
import signal
import subprocess
import sys
import tempfile
import time
from typing import Any, Sequence


TASK_ID = "slice8b-v10-exact-fused-band-transition-m1-20260825"
REMOTE = "e@192.168.3.94"
REMOTE_HOSTNAME = "e-MEGA-MINI-M1-13th"
REMOTE_MACHINE_ID_SHA256 = "5ac0bb537745673ef80942c2df32d586f5753c85de1c8b2453aa4e7317b77441"
REMOTE_PARENT = pathlib.Path("/home/e/.local/share/lay/provenance") / TASK_ID
REMOTE_BUILD = REMOTE_PARENT / "build-v1"
REMOTE_RESULT = REMOTE_PARENT / "result-v1"
REMOTE_STATE = pathlib.Path("/home/e/.local/state/lay") / TASK_ID

CONTROLLER = pathlib.Path(__file__).resolve()
BASE_REMOTE = CONTROLLER.with_name("base.py")
REMOTE_BOOTSTRAP = BASE_REMOTE.is_file()
PROJECT_ROOT = (
    pathlib.Path("/home/ubu/projects/lay-l1-exact-peak-search")
    if REMOTE_BOOTSTRAP
    else CONTROLLER.parents[1]
)
BASE_LOCAL = PROJECT_ROOT / "scripts/lay-v10-structural-work.py"
FRAGMENT = (
    CONTROLLER.with_name("fragment.inc")
    if REMOTE_BOOTSTRAP
    else PROJECT_ROOT / "scripts/lay_v10_fused_band_transition_m1_test_module.rs.inc"
)
CONTRACT = (CONTROLLER.with_name("contract.md") if REMOTE_BOOTSTRAP else PROJECT_ROOT / (
    "docs/structural_gates/"
    "LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_EXACT_FUSED_BAND_TRANSITION_M1_CONTRACT_2026-08-25.md"
))
ROUTE = (CONTROLLER.with_name("route.md") if REMOTE_BOOTSTRAP else PROJECT_ROOT / (
    "docs/structural_gates/"
    "LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_EXACT_FUSED_BAND_TRANSITION_M1_ROUTE_V6.md"
))
ROUTE_RECEIPT = (CONTROLLER.with_name("route-receipt.json") if REMOTE_BOOTSTRAP else PROJECT_ROOT / (
    "docs/structural_gates/receipts/"
    "LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_EXACT_FUSED_BAND_TRANSITION_M1_ROUTE_V6_RECEIPT_2026-08-25.json"
))
PREFLIGHT = (CONTROLLER.with_name("preflight.json") if REMOTE_BOOTSTRAP else PROJECT_ROOT / (
    "docs/structural_gates/preflights/"
    "LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_EXACT_FUSED_BAND_TRANSITION_M1_IMPLEMENTATION_V2_2026-08-25.json"
))
PREFLIGHT_RECEIPT = (CONTROLLER.with_name("preflight-receipt.json") if REMOTE_BOOTSTRAP else PROJECT_ROOT / (
    "docs/structural_gates/receipts/"
    "LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_EXACT_FUSED_BAND_TRANSITION_M1_IMPLEMENTATION_V2_PREFLIGHT_2026-08-25.json"
))
PMU_RECEIPT = (CONTROLLER.with_name("combined-pmu.json") if REMOTE_BOOTSTRAP else PROJECT_ROOT / (
    "docs/structural_gates/receipts/"
    "LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_LOADED_PMU_DIAGNOSIS_COMBINED_V3_V4_2026-08-25.json"
))
DECISION_RECEIPT = (CONTROLLER.with_name("decision.json") if REMOTE_BOOTSTRAP else PROJECT_ROOT / (
    "docs/structural_gates/receipts/"
    "LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_STRUCTURAL_PMU_DECISION_2026-08-25.json"
))
LOCAL_RESULT = PROJECT_ROOT / (
    "docs/structural_gates/receipts/"
    "LAY_IME_TARGET_AUTHORITY_SLICE8B_V10_EXACT_FUSED_BAND_TRANSITION_M1_2026-08-25"
)
ACTIVE_V11 = PROJECT_ROOT / "src/nanda_wave/l2_field/v13_typed_peak.rs"

EXPECTED = {
    "contract": "891a5ee2aa11840d8692d1ab110738bdd50d7bdca2b4873e92ab2ce125d2298b",
    "route": "76ab8038d118fd250af965922266a888b2ff7fd3cc6c5b359f6df5df561a8b70",
    "route_receipt": "757463bffd63dcca470818bfdf5d30d60d621d4390fe7dc7948723dcb61a7806",
    "preflight": "459e65c3c7a7f252c2b1e5b4bb5474497763ccde26fc000ae547f4bf2e9565f2",
    "preflight_receipt": "a6846348a4576b9c006f969c38ad8d28173f1aacf149bccdd7f615dde4cc4d54",
    "decision": "98f7f606e3a21448e92137091f75c53c1cb210049895f0c9a18f7745da3afd09",
    "pmu_receipt": "ea9a19cace1eab5418f783dfb6c18a4de2adb7281356afffd12bb2b28cdacbd1",
    "v10_source": "f6178f3882c1a216a9c2aaf7ff59dd6bdd78798708516b753ea7208177d9718c",
    "production_prefix": "ce9ea2d290602774e2f45444e10e6fdabce6af4a61ae2bdca54c6d2e31a53f26",
    "active_v11": "d9edfe7346b8636096701d4ac38044d5e386e6741a086a4d8da57ac15bffdf3b",
    "fragment": "bdef992d1f9bec095b3f683b384f1e7d23323823625cf3547dc44480511f0d76",
}

TEST_NAME = "nanda_wave::l2_field::v13_typed_peak::tests::v10_exact_fused_band_transition_m1"
EVENTS = ("instructions", "cycles", "branches", "branch-misses")
MODES = ("G0", "G1", "U1")
TRANSITIONS = 25_145_756
TRANSITIONS_PER_QUERY = 65_826.58638743455
BASELINE_INSTRUCTIONS_PER_QUERY = 42_378_604.08638743
PROJECTED_SAVING_GATE = 0.15


class M1Error(RuntimeError):
    pass


def require(condition: bool, message: str) -> None:
    if not condition:
        raise M1Error(message)


def load_base() -> Any:
    path = BASE_REMOTE if BASE_REMOTE.is_file() else BASE_LOCAL
    require(path.is_file(), f"missing structural-work base controller: {path}")
    spec = importlib.util.spec_from_file_location("lay_v10_structural_work_base", path)
    require(spec is not None and spec.loader is not None, "cannot load base controller")
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    configure_base(module)
    return module


def assemble_source(base: Any, v10: bytes, fragment: bytes) -> bytes:
    require(v10.endswith(b"}\n"), "V10 terminal brace mismatch")
    require(fragment.startswith(b"\n    const M1_TEST"), "M1 fragment prefix mismatch")
    final = v10[:-2] + fragment + b"}\n"
    require(final[:39_047] == v10[:39_047], "V10 production prefix changed")
    require(
        base.sha256_bytes(final[:39_047]) == EXPECTED["production_prefix"],
        "V10 production prefix SHA mismatch",
    )
    return final


def initialize_remote_state(base: Any) -> None:
    require(not REMOTE_PARENT.exists(), "M1 remote parent already exists")
    require(not REMOTE_STATE.exists(), "M1 remote state already exists")
    REMOTE_PARENT.mkdir(parents=True, mode=0o700)
    markers = REMOTE_STATE / "markers"
    markers.mkdir(parents=True, mode=0o700)
    for name in ("build", "parity", "g0", "g1", "u1"):
        base.write_new_json(
            markers / f"{name}.available",
            {"task_id": TASK_ID, "route": name, "retry_permitted": False},
            0o400,
        )
    base.write_new_bytes(REMOTE_STATE / "route.lock", b"M1\n", 0o400)
    base.fsync_directory(markers)
    base.fsync_directory(REMOTE_STATE)


def configure_base(base: Any) -> None:
    base.TASK_ID = TASK_ID
    base.REMOTE = REMOTE
    base.REMOTE_HOSTNAME = REMOTE_HOSTNAME
    base.REMOTE_MACHINE_ID_SHA256 = REMOTE_MACHINE_ID_SHA256
    base.REMOTE_PARENT = REMOTE_PARENT
    base.REMOTE_BUILD = REMOTE_BUILD
    base.REMOTE_RESULT = REMOTE_RESULT
    base.REMOTE_STATE = REMOTE_STATE
    base.PROJECT_ROOT = PROJECT_ROOT
    base.FRAGMENT = FRAGMENT
    base.CONTROLLER = CONTROLLER
    base.CONTRACT = CONTRACT
    base.ROUTE = ROUTE
    base.ROUTE_RECEIPT = ROUTE_RECEIPT
    base.PREFLIGHT = PREFLIGHT
    base.PREFLIGHT_RECEIPT = PREFLIGHT_RECEIPT
    base.PMU_RECEIPT = PMU_RECEIPT
    base.LOCAL_RESULT = LOCAL_RESULT
    base.ACTIVE_V11 = ACTIVE_V11
    base.TEST_NAME = TEST_NAME
    base.EXPECTED.update(
        {
            "contract": EXPECTED["contract"],
            "route": EXPECTED["route"],
            "route_receipt": EXPECTED["route_receipt"],
            "preflight": EXPECTED["preflight"],
            "preflight_receipt": EXPECTED["preflight_receipt"],
            "pmu_receipt": EXPECTED["pmu_receipt"],
            "v10_source": EXPECTED["v10_source"],
            "production_prefix": EXPECTED["production_prefix"],
            "active_v11": EXPECTED["active_v11"],
        }
    )
    base.assemble_source = lambda v10, fragment: assemble_source(base, v10, fragment)
    base.initialize_remote_state = lambda: initialize_remote_state(base)


def verify_admission(base: Any) -> dict[str, Any]:
    files = base.verify_local_admission()
    files["decision"] = base.require_file(
        DECISION_RECEIPT, sha256=EXPECTED["decision"], mode="0444"
    )
    files["base"] = base.require_file(BASE_LOCAL)
    files["fragment"] = base.require_file(FRAGMENT, sha256=EXPECTED["fragment"])
    route = base.load_json(ROUTE_RECEIPT)
    require(route.get("verdict") == "PASS", "M1 route is not PASS")
    require(route.get("authority_ready") is False, "M1 route gained authority")
    preflight = base.load_json(PREFLIGHT_RECEIPT)
    require(preflight.get("verdict") == "READY_TO_IMPLEMENT", "M1 preflight not ready")
    require(preflight.get("safe_to_implement") is True, "M1 preflight unsafe")
    require(not preflight.get("blockers"), "M1 preflight has blockers")
    decision = base.load_json(DECISION_RECEIPT)
    require(
        decision.get("measured_decisions", {}).get("exact_fused_band_transition_microproof")
        == "ADMIT_NEXT_PAPER_GATE",
        "M1 decision admission missing",
    )
    require(
        decision.get("claim_boundary", {}).get("v12_admitted") is False,
        "decision admits V12",
    )
    return files


def local_self_check(base: Any) -> None:
    files = verify_admission(base)
    source = assemble_source(
        base,
        (base.P0 / "artifacts/v13_typed_peak.v10.rs").read_bytes(),
        FRAGMENT.read_bytes(),
    )
    required = (
        "m1_g1_advance",
        "m1_u1_advance",
        "M1_EXPECTED_TRANSITIONS",
        "equality_window",
        "query_len",
        "S1_swar_candidate",
    )
    for token in required:
        require(token.lower() in source.decode(errors="replace").lower(), f"missing {token}")
    forbidden = (
        "LAY_V10_C1_",
        "systemctl restart",
        "pkill",
        "killall",
        "scaling_governor",
        "ExactFusedBandTransitionV12",
    )
    controller = CONTROLLER.read_text(encoding="utf-8")
    fragment = FRAGMENT.read_text(encoding="utf-8")
    for token in forbidden:
        require(token not in fragment, f"forbidden fragment token: {token}")
    compile(controller, str(CONTROLLER), "exec")
    with tempfile.NamedTemporaryFile(suffix=".rs") as temporary:
        temporary.write(b"mod x {" + FRAGMENT.read_bytes() + b"}\n")
        temporary.flush()
        formatted = subprocess.run(
            ["rustfmt", "--edition", "2024", "--check", temporary.name],
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            check=False,
        )
    require(
        formatted.returncode == 0,
        "M1 test-only fragment failed rustfmt check:\n"
        + formatted.stdout.decode(errors="replace")[-4000:]
        + formatted.stderr.decode(errors="replace")[-4000:],
    )
    require("S1" not in MODES, "S1 candidate unexpectedly enabled")
    print(
        json.dumps(
            {
                "verdict": "PASS",
                "task_id": TASK_ID,
                "files": files,
                "fragment_sha256": base.sha256_file(FRAGMENT),
                "assembled_source_sha256": base.sha256_bytes(source),
                "production_prefix_sha256": base.sha256_bytes(source[:39_047]),
                "modes": MODES,
                "s1_swar_candidate": "OMITTED",
                "formal_b_pass": False,
                "v12_admitted": False,
            },
            sort_keys=True,
        )
    )


def upload_bootstrap(base: Any) -> str:
    temporary = base.ssh(["mktemp", "-d", "/tmp/lay-v10-m1.XXXXXX"]).stdout.decode().strip()
    require(temporary.startswith("/tmp/lay-v10-m1."), "unexpected bootstrap path")
    files = {
        CONTROLLER: "controller.py",
        BASE_LOCAL: "base.py",
        FRAGMENT: "fragment.inc",
        CONTRACT: "contract.md",
        ROUTE: "route.md",
        ROUTE_RECEIPT: "route-receipt.json",
        PREFLIGHT: "preflight.json",
        PREFLIGHT_RECEIPT: "preflight-receipt.json",
        PMU_RECEIPT: "combined-pmu.json",
        DECISION_RECEIPT: "decision.json",
    }
    for source, name in files.items():
        base.scp(source, f"{temporary}/{name}")
    return temporary


def local_build(base: Any) -> None:
    verify_admission(base)
    require(not LOCAL_RESULT.exists(), "local M1 result already exists")
    probe = base.ssh(
        [
            "python3",
            "-c",
            (
                "import hashlib,os,pathlib;"
                "p=pathlib.Path('/etc/machine-id');"
                "print(os.uname().nodename);"
                "print(hashlib.sha256(p.read_bytes()).hexdigest());"
                f"print(int(pathlib.Path('{REMOTE_PARENT}').exists()));"
                f"print(int(pathlib.Path('{REMOTE_STATE}').exists()))"
            ),
        ]
    ).stdout.decode().splitlines()
    require(
        probe == [REMOTE_HOSTNAME, REMOTE_MACHINE_ID_SHA256, "0", "0"],
        f"remote build probe failed: {probe}",
    )
    temporary = upload_bootstrap(base)
    try:
        result = base.ssh(
            ["python3", f"{temporary}/controller.py", "remote-build", temporary],
            check=False,
        )
        require(result.returncode == 0, result.stderr.decode(errors="replace")[-6000:])
        print(result.stdout.decode().strip())
    finally:
        base.ssh(["rm", "-rf", "--", temporary], check=False)


def consume_marker(base: Any, name: str) -> pathlib.Path:
    return base.consume_marker(name.lower())


def subject_environment(base: Any, output: pathlib.Path, mode: str) -> dict[str, str]:
    artifacts = base.REMOTE_B0A / "inputs/slice8b-v10-f6178f/artifacts"
    environment = base.controlled_environment()
    environment.update(
        {
            "LAY_V10_M1_MODE": mode,
            "LAY_V10_M1_OUTPUT": str(output),
            "LAY_V10_M1_V13_PACKAGE": str(base.REMOTE_B0A / "inputs/LAY-L2-RU-FULL-v13.bin"),
            "LAY_V10_M1_SIDECAR": str(artifacts / "v13-typed-peak-dafsa.bin"),
            "LAY_V10_M1_V7": str(base.REMOTE_B0A / "inputs/denominator-v7.json"),
            "LAY_V10_M1_SCHEDULE": str(base.REMOTE_B0B / "query-schedule.json"),
        }
    )
    return environment


def subject_command(base: Any) -> list[str]:
    executable = REMOTE_BUILD / "diagnostic-test-elf"
    return [
        str(base.REMOTE_LOADER),
        str(executable),
        "--exact",
        TEST_NAME,
        "--ignored",
        "--nocapture",
        "--test-threads=1",
    ]


def terminate_owned(process: subprocess.Popen[bytes] | None) -> None:
    if process is None or process.poll() is not None:
        return
    with contextlib.suppress(ProcessLookupError):
        os.killpg(process.pid, signal.SIGTERM)
    with contextlib.suppress(subprocess.TimeoutExpired):
        process.wait(timeout=3)
    if process.poll() is None:
        with contextlib.suppress(ProcessLookupError):
            os.killpg(process.pid, signal.SIGKILL)
        process.wait()


def wait_for_file(process: subprocess.Popen[bytes], path: pathlib.Path, timeout: float) -> None:
    deadline = time.monotonic() + timeout
    while time.monotonic() < deadline:
        if path.is_file():
            return
        require(process.poll() is None, f"subject exited before {path.name}")
        time.sleep(0.002)
    raise M1Error(f"timeout waiting for {path}")


def open_fifo(path: pathlib.Path, flags: int, deadline: float) -> int:
    while time.monotonic() < deadline:
        try:
            return os.open(path, flags | os.O_NONBLOCK)
        except OSError as error:
            if error.errno not in (6, 11):
                raise
            time.sleep(0.005)
    raise M1Error(f"FIFO open timeout: {path}")


def read_fifo_line(descriptor: int, deadline: float) -> str:
    value = bytearray()
    while time.monotonic() < deadline:
        try:
            chunk = os.read(descriptor, 4096)
        except BlockingIOError:
            chunk = b""
        if chunk:
            value.extend(chunk)
            if b"\n" in value:
                return bytes(value).decode(errors="replace").strip()
        time.sleep(0.002)
    raise M1Error("perf control acknowledgement timeout")


def event_pmu(observed: str, expected: str) -> str | None:
    normalized = observed.strip().lower().replace(":u", "")
    expected = expected.lower()
    if normalized == expected:
        return None
    match = re.fullmatch(r"cpu_(core|atom)/([^/]+)/", normalized)
    if match is not None and match.group(2) == expected:
        return match.group(1)
    return "unknown"


def numeric_counter(value: Any) -> float | None:
    if isinstance(value, (int, float)):
        return float(value)
    if not isinstance(value, str):
        return None
    compact = value.strip().replace(",", "")
    if compact.startswith("<") or compact in ("", "not counted", "not supported"):
        return None
    try:
        return float(compact)
    except ValueError:
        return None


def parse_perf(raw: bytes) -> dict[str, Any]:
    rows = []
    diagnostics = []
    for line in raw.decode(errors="replace").splitlines():
        try:
            value = json.loads(line)
        except json.JSONDecodeError:
            if line.strip():
                diagnostics.append(line)
            continue
        if isinstance(value, dict) and isinstance(value.get("event"), str):
            rows.append(value)
        elif line.strip():
            diagnostics.append(line)
    require(rows, "perf produced no JSON counter rows")
    counters: dict[str, Any] = {}
    for expected in EVENTS:
        matched = [row for row in rows if event_pmu(row["event"], expected) != "unknown"]
        require(matched, f"perf event missing: {expected}")
        counted = []
        inactive = []
        for row in matched:
            counter = numeric_counter(row.get("counter-value"))
            running = numeric_counter(row.get("pcnt-running"))
            runtime = numeric_counter(row.get("event-runtime"))
            pmu = event_pmu(row["event"], expected)
            if counter is None:
                require(
                    str(row.get("counter-value", "")).strip().lower() == "<not counted>",
                    f"unsupported perf event: {row}",
                )
                require(runtime == 0 and running == 0, f"inactive PMU row ran: {expected}")
                inactive.append(row)
                continue
            require(pmu == "core", f"physical replay escaped core PMU: {row['event']}")
            require(running is not None and abs(running - 100.0) <= 0.01, f"scaled event: {expected}")
            require(runtime is not None and runtime > 0, f"missing event runtime: {expected}")
            counted.append((counter, row))
        require(len(counted) == 1, f"expected one core counter for {expected}")
        counters[expected] = {
            "value": counted[0][0],
            "row": counted[0][1],
            "inactive_rows": inactive,
        }
    unknown = [
        row["event"]
        for row in rows
        if not any(event_pmu(row["event"], expected) != "unknown" for expected in EVENTS)
    ]
    require(not unknown, f"unexpected perf events: {unknown}")
    return {"counters": counters, "diagnostics": diagnostics, "rows": rows}


def validate_parity(value: dict[str, Any]) -> dict[str, Any]:
    require(value.get("verdict") == "PASS", f"M1 parity failed: {value}")
    parity = value.get("parity", {})
    expected = {
        "records": 382,
        "transitions": TRANSITIONS,
        "expanded_states": 8_059_788,
        "terminal_mismatches": 0,
        "peak_mismatches": 0,
        "completeness_mismatches": 0,
        "work_mismatches": 0,
        "scratch_mismatches": 0,
        "transition_mismatches": 0,
        "packed_state_mismatches": 0,
        "maximum_product_states": 35_590,
        "maximum_scratch_bytes": 6_656,
    }
    for key, expected_value in expected.items():
        require(parity.get(key) == expected_value, f"parity mismatch: {key}")
    stress = value.get("stress", {})
    require(stress.get("transition_mismatches") == 0, "stress transition mismatch")
    require(stress.get("packed_state_mismatches") == 0, "stress pack mismatch")
    require(value.get("s1_swar_candidate") == "OMITTED", "S1 unexpectedly present")
    require(value.get("formal_b_pass") is False, "M1 claimed formal B")
    require(value.get("v12_admitted") is False, "M1 admitted V12")
    return parity


def run_parity(base: Any, stage: pathlib.Path) -> dict[str, Any]:
    root = stage / "parity"
    root.mkdir(mode=0o700)
    marker = consume_marker(base, "parity")
    output = root / "SUBJECT_RESULT.json"
    environment = subject_environment(base, output, "PARITY")
    process = subprocess.run(
        subject_command(base),
        env=environment,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        timeout=1800,
        check=False,
    )
    base.write_new_bytes(root / "stdout.log", process.stdout)
    base.write_new_bytes(root / "stderr.log", process.stderr)
    require(process.returncode == 0, f"parity exited {process.returncode}: {process.stderr[-4000:]!r}")
    value = base.load_json(output)
    parity = validate_parity(value)
    wrapper = {
        "schema": "lay.v10.exact-fused-band-transition-m1-parity-wrapper.v1",
        "verdict": "PASS",
        "marker": str(marker),
        "subject_sha256": base.sha256_file(output),
        "trace_sha256": parity["trace_sha256"],
        "transitions": parity["transitions"],
        "perf_invoked": False,
        "runtime_authority_changed": False,
    }
    base.write_new_json(root / "PARITY_WRAPPER.json", wrapper)
    return wrapper


def child_as_e(environment: dict[str, str], command: Sequence[str]) -> list[str]:
    assignments = [f"{key}={value}" for key, value in sorted(environment.items())]
    return ["/usr/bin/sudo", "-n", "-u", "e", "/usr/bin/env", *assignments, *command]


def run_physical(
    base: Any,
    stage: pathlib.Path,
    mode: str,
    parity: dict[str, Any],
) -> dict[str, Any]:
    root = stage / mode.lower()
    root.mkdir(mode=0o700)
    control = root / "control"
    control.mkdir(mode=0o700)
    marker = consume_marker(base, mode)
    control_fifo = root / "perf-control.fifo"
    ack_fifo = root / "perf-ack.fifo"
    os.mkfifo(control_fifo, 0o600)
    os.mkfifo(ack_fifo, 0o600)
    output = root / "SUBJECT_RESULT.json"
    environment = subject_environment(base, output, mode)
    environment["LAY_V10_M1_CONTROL_DIR"] = str(control)
    child = child_as_e(
        environment,
        ["/usr/bin/taskset", "-c", "0", *subject_command(base)],
    )
    command = [
        "/usr/bin/sudo",
        "-n",
        "/usr/bin/perf",
        "stat",
        "--json-output",
        "--no-big-num",
        "--delay=-1",
        f"--control=fifo:{control_fifo},{ack_fifo}",
        "--event",
        ",".join(EVENTS),
        "--",
        *child,
    ]
    process: subprocess.Popen[bytes] | None = None
    control_fd: int | None = None
    ack_fd: int | None = None
    try:
        process = subprocess.Popen(
            command,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            start_new_session=True,
        )
        wait_for_file(process, control / "subject-ready", 1800.0)
        deadline = time.monotonic() + 10.0
        control_fd = open_fifo(control_fifo, os.O_WRONLY, deadline)
        ack_fd = open_fifo(ack_fifo, os.O_RDONLY, deadline)
        os.write(control_fd, b"enable\n")
        enable_ack = read_fifo_line(ack_fd, time.monotonic() + 10.0)
        base.write_new_bytes(control / "controller-enabled", b"enabled\n")
        started_ns = time.perf_counter_ns()
        wait_for_file(process, control / "subject-done", 600.0)
        ended_ns = time.perf_counter_ns()
        os.write(control_fd, b"disable\n")
        disable_ack = read_fifo_line(ack_fd, time.monotonic() + 10.0)
        base.write_new_bytes(control / "controller-disabled", b"disabled\n")
        stdout, stderr = process.communicate(timeout=120)
        base.write_new_bytes(root / "stdout.log", stdout)
        base.write_new_bytes(root / "perf.raw", stderr)
        require(process.returncode == 0, f"{mode} exited {process.returncode}: {stderr[-4000:]!r}")
        subject = base.load_json(output)
        require(subject.get("verdict") == "PHYSICAL_REPLAY_OBSERVED", f"{mode} subject failed")
        require(subject.get("mode") == mode, f"{mode} subject mode mismatch")
        subject_parity = subject.get("parity", {})
        require(subject_parity.get("trace_sha256") == parity["trace_sha256"], f"{mode} trace drift")
        require(subject_parity.get("transitions") == TRANSITIONS, f"{mode} transition drift")
        require(subject.get("s1_swar_candidate") == "OMITTED", f"{mode} enabled S1")
        parsed = parse_perf(stderr)
        counters = parsed["counters"]
        values = {event: counters[event]["value"] for event in EVENTS}
        receipt = {
            "schema": "lay.v10.exact-fused-band-transition-m1-physical-wrapper.v1",
            "verdict": "PHYSICAL_REPLAY_OBSERVED",
            "mode": mode,
            "marker": str(marker),
            "command": command,
            "enable_ack": enable_ack,
            "disable_ack": disable_ack,
            "transitions": TRANSITIONS,
            "trace_sha256": parity["trace_sha256"],
            "trace_event_bytes": subject_parity["trace_event_bytes"],
            "counters": counters,
            "derived": {
                "instructions_per_transition": values["instructions"] / TRANSITIONS,
                "cycles_per_transition": values["cycles"] / TRANSITIONS,
                "branches_per_transition": values["branches"] / TRANSITIONS,
                "branch_miss_rate": values["branch-misses"] / values["branches"],
                "controller_wall_ns_diagnostic": ended_ns - started_ns,
                "subject_wall_ns_diagnostic": subject["subject_wall_ns_diagnostic"],
            },
            "checksum": subject["checksum"],
            "subject_sha256": base.sha256_file(output),
            "raw_perf_sha256": base.sha256_file(root / "perf.raw"),
            "pmu_event_opened": True,
            "environment_intentionally_loaded": True,
            "formal_b_pass": False,
            "v12_admitted": False,
            "runtime_authority_changed": False,
        }
        base.write_new_json(root / "PHYSICAL_WRAPPER.json", receipt)
        return receipt
    except BaseException:
        terminate_owned(process)
        raise
    finally:
        if control_fd is not None:
            os.close(control_fd)
        if ack_fd is not None:
            os.close(ack_fd)


def remote_run(base: Any) -> None:
    base.remote_machine_identity()
    require(REMOTE_BUILD.is_dir(), "sealed M1 build missing")
    require(not REMOTE_RESULT.exists(), "M1 result already exists")
    base.verify_sha256sums(REMOTE_BUILD)
    provenance = base.load_json(REMOTE_BUILD / "EXECUTABLE_PROVENANCE.json")
    executable = REMOTE_BUILD / "diagnostic-test-elf"
    require(
        base.sha256_file(executable) == provenance["executable"]["sha256"],
        "M1 executable SHA drift",
    )
    require(
        base.elf_build_id(executable) == provenance["executable"]["build_id"],
        "M1 executable Build ID drift",
    )
    base.require_file(
        base.REMOTE_LOADER,
        sha256=base.EXPECTED["loader"],
        mode="0755",
        size=240_936,
    )
    stage = REMOTE_PARENT / f"result-v1.stage-{os.getpid()}-{time.time_ns()}"
    stage.mkdir(mode=0o700)
    before = base.environment_snapshot()
    try:
        parity = run_parity(base, stage)
        physical = {mode: run_physical(base, stage, mode, parity) for mode in MODES}
        g0 = physical["G0"]["derived"]["instructions_per_transition"]
        g1 = physical["G1"]["derived"]["instructions_per_transition"]
        u1 = physical["U1"]["derived"]["instructions_per_transition"]
        delta_per_query = (g0 - u1) * TRANSITIONS_PER_QUERY
        projected_saving = delta_per_query / BASELINE_INSTRUCTIONS_PER_QUERY
        passed = u1 < g0 and projected_saving >= PROJECTED_SAVING_GATE
        after = base.environment_snapshot()
        decision = {
            "schema": "lay.v10.exact-fused-band-transition-m1-decision.v1",
            "verdict": "M1_PASS" if passed else "M1_REJECT_FUSED",
            "task_id": TASK_ID,
            "subject": provenance["executable"],
            "production_prefix_bytes": 39_047,
            "production_prefix_sha256": EXPECTED["production_prefix"],
            "parity": parity,
            "physical": physical,
            "comparison": {
                "g0_instructions_per_transition": g0,
                "g1_instructions_per_transition": g1,
                "u1_instructions_per_transition": u1,
                "g0_to_g1_percent": 100.0 * (g0 - g1) / g0,
                "g1_to_u1_percent": 100.0 * (g1 - u1) / g1,
                "g0_to_u1_percent": 100.0 * (g0 - u1) / g0,
                "projected_instruction_delta_per_query": delta_per_query,
                "projected_whole_query_instruction_saving": projected_saving,
                "projected_saving_gate": PROJECTED_SAVING_GATE,
            },
            "environment_before": before,
            "environment_after": after,
            "loaded_host_is_blocker": False,
            "s1_swar_candidate": "OMITTED",
            "claim_boundary": {
                "latency_prediction": False,
                "latency_pass": False,
                "full_executor_admitted": False,
                "formal_b_pass": False,
                "v12_admitted": False,
                "runtime_authority_changed": False,
            },
        }
        base.write_new_json(stage / "M1_DECISION.json", decision)
        base.write_new_json(
            stage / "RUN_PROVENANCE.json",
            {
                "schema": "lay.v10.exact-fused-band-transition-m1-run.v1",
                "task_id": TASK_ID,
                "controller_sha256": base.sha256_file(CONTROLLER),
                "base_controller_sha256": base.sha256_file(BASE_REMOTE),
                "fragment_sha256": base.sha256_file(FRAGMENT),
                "markers_consumed": ["parity", "g0", "g1", "u1"],
                "adaptive_rerun": False,
                "third_loaded_c1_run": False,
                "clean_c1_marker_consumed": False,
                "foreign_process_control": False,
                "host_tuning": False,
                "installed_lay_changed": False,
                "runtime_authority_changed": False,
            },
        )
        base.write_sha256sums(stage)
        base.seal_tree(stage)
        os.rename(stage, REMOTE_RESULT)
        base.fsync_directory(REMOTE_PARENT)
        print(json.dumps({"state": decision["verdict"], "result": str(REMOTE_RESULT)}))
    except Exception as error:
        with contextlib.suppress(Exception):
            base.write_new_json(
                stage / "FAILURE.json",
                {
                    "schema": "lay.v10.exact-fused-band-transition-m1-failure.v1",
                    "error": str(error),
                    "retry_permitted": False,
                    "runtime_authority_changed": False,
                },
            )
            base.write_sha256sums(stage)
            base.seal_tree(stage)
            os.rename(stage, REMOTE_PARENT / "run-failure-v1")
        raise


def local_run(base: Any) -> None:
    verify_admission(base)
    require(not LOCAL_RESULT.exists(), "local M1 result already exists")
    before = base.local_runtime_snapshot()
    temporary = upload_bootstrap(base)
    try:
        result = base.ssh(
            ["python3", f"{temporary}/controller.py", "remote-run"],
            check=False,
        )
        require(result.returncode == 0, result.stderr.decode(errors="replace")[-8000:])
        remote_state = result.stdout.decode().strip()
    finally:
        base.ssh(["rm", "-rf", "--", temporary], check=False)
    after = base.local_runtime_snapshot()
    require(before == after, f"installed runtime changed: {before} != {after}")
    stage = pathlib.Path(f"{LOCAL_RESULT}.stage-{os.getpid()}-{time.time_ns()}")
    try:
        base.run(["scp", "-q", "-pr", f"{REMOTE}:{REMOTE_RESULT}", str(stage)])
        entries = base.verify_sha256sums(stage)
        decision = base.load_json(stage / "M1_DECISION.json")
        require(decision.get("verdict") in ("M1_PASS", "M1_REJECT_FUSED"), "invalid M1 decision")
        require(
            decision.get("claim_boundary", {}).get("v12_admitted") is False,
            "M1 local result admits V12",
        )
        require(
            not any(path.stat().st_mode & 0o222 for path in stage.rglob("*")),
            "remote M1 result contains writable objects",
        )
        base.seal_tree(stage)
        os.rename(stage, LOCAL_RESULT)
        base.fsync_directory(LOCAL_RESULT.parent)
        print(
            json.dumps(
                {
                    "state": decision["verdict"],
                    "remote": remote_state,
                    "local_result": str(LOCAL_RESULT),
                    "manifest_entries": entries,
                    "runtime_stable": True,
                },
                sort_keys=True,
            )
        )
    except Exception:
        base.remove_tree(stage)
        raise


def remote_status(base: Any) -> None:
    markers = REMOTE_STATE / "markers"
    print(
        json.dumps(
            {
                "parent": REMOTE_PARENT.exists(),
                "build": REMOTE_BUILD.exists(),
                "result": REMOTE_RESULT.exists(),
                "state": REMOTE_STATE.exists(),
                "markers": sorted(path.name for path in markers.glob("*")) if markers.is_dir() else [],
            },
            sort_keys=True,
        )
    )


def parser() -> argparse.ArgumentParser:
    value = argparse.ArgumentParser()
    value.add_argument(
        "action",
        choices=("self-check", "build", "run", "status", "remote-build", "remote-run"),
    )
    value.add_argument("argument", nargs="?")
    return value


def main() -> int:
    arguments = parser().parse_args()
    try:
        base = load_base()
        if arguments.action == "self-check":
            local_self_check(base)
        elif arguments.action == "build":
            local_build(base)
        elif arguments.action == "run":
            local_run(base)
        elif arguments.action == "status":
            result = base.ssh(["python3", "-c", (
                "import json,pathlib;"
                f"p=pathlib.Path('{REMOTE_PARENT}');s=pathlib.Path('{REMOTE_STATE}');"
                "m=s/'markers';"
                "print(json.dumps({'parent':p.exists(),'build':(p/'build-v1').exists(),"
                "'result':(p/'result-v1').exists(),'state':s.exists(),"
                "'markers':sorted(x.name for x in m.glob('*')) if m.is_dir() else []}))"
            )])
            print(result.stdout.decode().strip())
        elif arguments.action == "remote-build":
            require(os.uname().nodename == REMOTE_HOSTNAME, "remote-build on wrong host")
            require(arguments.argument is not None, "remote-build requires bootstrap path")
            base.remote_build(pathlib.Path(arguments.argument))
        elif arguments.action == "remote-run":
            require(os.uname().nodename == REMOTE_HOSTNAME, "remote-run on wrong host")
            remote_run(base)
        return 0
    except Exception as error:
        print(f"M1 ERROR: {error}", file=sys.stderr)
        return 1


if __name__ == "__main__":
    raise SystemExit(main())
