#!/usr/bin/python3
"""Run the versioned V2 actual-client IME proof in a private remote sandbox."""

from __future__ import annotations

import argparse
from dataclasses import dataclass, replace
import hashlib
import json
import os
from pathlib import Path, PurePosixPath
import re
import shutil
import subprocess
import sys
import uuid
import xml.etree.ElementTree as ET


SCHEMA = "lay.ime-client-harness.v1"
PROOF_CONTRACT = "lay.ime-client.actual-input-context.v2"
RUN_METADATA_SCHEMA = "lay.ime-client-harness.run-metadata.v2"
V2_DRIVER_SHA256 = "d80447f21db4d689ea49d39742feb36e2202b23979d820380c1fcef916b88c12"
V1_DRIVER_PROVENANCE = {
    "version": "v1",
    "git_commit": "708245298a3f553ac3c52243728c02ba6344a140",
    "git_blob": "04f7dbac56c92dbe0238c0754bcb6db4e242256c",
    "sha256": "9ece223f6689323e5cae3fc5f27cf990d37b86dff5e9f6e0ff3212d4dd488750",
    "normalized_sha256": "669e3ef88cc2639794fde79dcb9132b99ebcaf142cafe39065376f42c4a056dc",
}
SANDBOX_CANDIDATE = Path("/tmp/candidate/lay-ibus-engine")
SANDBOX_DEPLOYED_ROOT = Path("/tmp/deployed")
SANDBOX_DEPS_ROOT = Path("/tmp/deps")
SANDBOX_PROOF_ROOT = Path("/tmp/proof")
RESOURCE_ENVELOPE_ID = "td124-ime-client-cpu200-mem1536-swap0-tasks128-runtime90"
REQUIRED_GUARD = {
    "LAY_RESOURCE_GUARD_ACTIVE": "1",
    "LAY_RESOURCE_LEASE_HELD": "1",
    "LAY_RESOURCE_PROFILE": "dedicated-20cpu",
    "CARGO_BUILD_JOBS": "20",
    "RUST_TEST_THREADS": "1",
}
ROLE_PATHS = {
    "l11_service": PurePosixPath("bin/lay-l1.1-serve"),
    "l11_package": PurePosixPath(
        "l1.1/LAY-L1.1-RU-COMPOSITE-EN300K-PHASE8I-v9.v9.bin"
    ),
    "l11_proof": PurePosixPath(
        "l1.1/LAY-L1.1-RU-COMPOSITE-EN300K-PHASE8I-v9."
        "4983930495e793c1d28c7558fe006ddf8097ee575bebb1afd3f1dba4ddb1d01d.proof.json"
    ),
    "l11_active_receipt": PurePosixPath("l1.1/active.installed.json"),
    "l2_lexical_phase": PurePosixPath("l2/l2_lexical_phase_v2.bin"),
    "l2_v13": PurePosixPath("l2/LAY-L2-RU-FULL-v13.bin"),
    "l2_v13_dafsa": PurePosixPath("l2/LAY-L2-RU-FULL-v13.dafsa"),
    "productive_v90_package": PurePosixPath(
        "l2/LAY-L2-PRODUCTIVE-PARADIGM-v90.p2m"
    ),
    "productive_v90_recovery": PurePosixPath(
        "l2/LAY-L2-PRODUCTIVE-PARADIGM-v90.p2r"
    ),
}
HOST_TOOLS = (
    Path("/usr/bin/systemd-run"),
    Path("/usr/bin/bwrap"),
    Path("/usr/bin/dbus-run-session"),
    Path("/usr/bin/python3"),
    Path("/usr/bin/ibus-daemon"),
    Path("/usr/libexec/ibus-engine-simple"),
)


class HarnessError(RuntimeError):
    def __init__(self, message: str, code: int = 64) -> None:
        super().__init__(message)
        self.code = code


@dataclass(frozen=True)
class Dependency:
    role: str
    relative_path: PurePosixPath
    host_path: Path
    sha256: str
    size_bytes: int


@dataclass(frozen=True)
class Plan:
    config_path: Path
    config_sha256: str
    origin_machine_id_sha256: str
    execution_lease_id: str
    candidate: Path
    candidate_sha256: str
    deployed_ibus_root: Path
    deps_root: Path
    manifest_path: Path
    manifest_sha256: str
    receipt_embedded_mount_path: Path
    dependencies: dict[str, Dependency]
    startup_schedule: str = "immediate"
    scenario_set: str = "restoration"
    startup_proof_profile: str = "legacy"


def sha256(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as handle:
        for block in iter(lambda: handle.read(1024 * 1024), b""):
            digest.update(block)
    return digest.hexdigest()


def _exact_hash(value: object, label: str) -> str:
    if not isinstance(value, str) or re.fullmatch(r"[0-9a-f]{64}", value) is None:
        raise HarnessError(f"{label} must be an exact lowercase 64-hex SHA-256")
    return value


def _object(value: object, label: str) -> dict[str, object]:
    if not isinstance(value, dict):
        raise HarnessError(f"{label} must be a JSON object")
    return value


def _exact_keys(value: dict[str, object], expected: set[str], label: str) -> None:
    actual = set(value)
    if actual != expected:
        raise HarnessError(
            f"{label} keys mismatch: missing={sorted(expected - actual)} "
            f"unexpected={sorted(actual - expected)}"
        )


def _absolute_path(value: object, label: str) -> Path:
    if not isinstance(value, str) or not value or "\x00" in value or "\n" in value:
        raise HarnessError(f"{label} must be a nonempty filesystem path")
    path = Path(value)
    if not path.is_absolute():
        raise HarnessError(f"{label} must be absolute: {value!r}")
    return path


def _read_json(path: Path, label: str) -> dict[str, object]:
    try:
        return _object(json.loads(path.read_text(encoding="utf-8")), label)
    except (OSError, UnicodeError, json.JSONDecodeError) as error:
        raise HarnessError(f"cannot read {label} {path}: {error}", 65) from error


def _require_file(path: Path, label: str, executable: bool = False) -> None:
    if not path.is_file():
        raise HarnessError(f"missing {label}: {path}", 65)
    if executable and not os.access(path, os.X_OK):
        raise HarnessError(f"{label} is not executable: {path}", 65)


def _require_hash(path: Path, expected: str, label: str) -> None:
    actual = sha256(path)
    if actual != expected:
        raise HarnessError(
            f"{label} hash mismatch: expected={expected} actual={actual} path={path}",
            65,
        )


def verify_driver_identity(harness_root: Path | None = None) -> str:
    root = harness_root or Path(__file__).resolve().parent
    actual = sha256(root / "driver.py")
    if actual != V2_DRIVER_SHA256:
        raise HarnessError(
            "V2 driver contract drift: "
            f"expected SHA-256 {V2_DRIVER_SHA256}, got {actual}"
        )
    return actual


def _manifest_relative(entry_path: object, remote_root: Path) -> PurePosixPath:
    path = _absolute_path(entry_path, "dependency manifest files[].path")
    try:
        relative = path.relative_to(remote_root)
    except ValueError as error:
        raise HarnessError(
            f"dependency path is outside manifest remote_root: {path}"
        ) from error
    pure = PurePosixPath(relative.as_posix())
    if not pure.parts or any(part in ("", ".", "..") for part in pure.parts):
        raise HarnessError(f"unsafe dependency relative path: {pure}")
    return pure


def load_plan(config_path: Path) -> Plan:
    config_path = config_path.resolve(strict=True)
    config = _read_json(config_path, "harness config")
    _exact_keys(
        config,
        {
            "schema",
            "origin_machine_id_sha256",
            "execution_lease_id",
            "candidate",
            "deployed_ibus_root",
            "dependencies",
        },
        "harness config",
    )
    if config["schema"] != SCHEMA:
        raise HarnessError(f"unsupported harness config schema: {config['schema']!r}")
    origin_hash = _exact_hash(
        config["origin_machine_id_sha256"], "origin_machine_id_sha256"
    )
    lease = config["execution_lease_id"]
    if not isinstance(lease, str) or re.fullmatch(r"[A-Za-z0-9._:-]{1,128}", lease) is None:
        raise HarnessError("execution_lease_id must match [A-Za-z0-9._:-]{1,128}")

    candidate_config = _object(config["candidate"], "candidate")
    _exact_keys(candidate_config, {"path", "sha256"}, "candidate")
    candidate = _absolute_path(candidate_config["path"], "candidate.path")
    candidate_hash = _exact_hash(candidate_config["sha256"], "candidate.sha256")

    deps_config = _object(config["dependencies"], "dependencies")
    _exact_keys(
        deps_config,
        {"root", "manifest_path", "manifest_sha256", "receipt_embedded_mount_path"},
        "dependencies",
    )
    deps_root = _absolute_path(deps_config["root"], "dependencies.root")
    manifest_path = _absolute_path(
        deps_config["manifest_path"], "dependencies.manifest_path"
    )
    if manifest_path != deps_root / "dependency-manifest.json":
        raise HarnessError("dependencies.manifest_path must be root/dependency-manifest.json")
    manifest_hash = _exact_hash(
        deps_config["manifest_sha256"], "dependencies.manifest_sha256"
    )
    embedded = _absolute_path(
        deps_config["receipt_embedded_mount_path"],
        "dependencies.receipt_embedded_mount_path",
    )
    try:
        embedded.relative_to("/home")
    except ValueError as error:
        raise HarnessError("receipt embedded mount path must be below private /home") from error

    deployed = _absolute_path(config["deployed_ibus_root"], "deployed_ibus_root")
    _require_file(candidate, "candidate", executable=True)
    _require_hash(candidate, candidate_hash, "candidate")
    _require_file(manifest_path, "dependency manifest")
    _require_hash(manifest_path, manifest_hash, "dependency manifest")
    if not deps_root.is_dir():
        raise HarnessError(f"missing dependency root: {deps_root}", 65)
    if not deployed.is_dir():
        raise HarnessError(f"missing deployed IBus root: {deployed}", 65)
    _require_file(deployed / "bundle/bin/ibus-daemon", "deployed private ibus-daemon", True)
    _require_file(deployed / "bundle/lib/ld-linux-x86-64.so.2", "deployed loader", True)

    manifest = _read_json(manifest_path, "dependency manifest")
    if manifest.get("schema") != "lay.td121.authority-dependency-manifest.v1":
        raise HarnessError(f"unsupported dependency manifest schema: {manifest.get('schema')!r}")
    remote_root = _absolute_path(manifest.get("remote_root"), "manifest remote_root")
    topology = _object(manifest.get("receipt_topology"), "manifest receipt_topology")
    if topology.get("runtime_model_dir") != str(embedded):
        raise HarnessError("configured receipt mount differs from manifest runtime_model_dir")
    if topology.get("runtime_receipt") != str(embedded / "active.installed.json"):
        raise HarnessError("manifest runtime_receipt is not embedded active.installed.json")
    if topology.get("receipt_bytes_rewritten") is not False:
        raise HarnessError("dependency receipt bytes must remain exact and unmodified")
    if topology.get("direct_lay_l11_package_env_forbidden") is not True:
        raise HarnessError("dependency manifest does not preserve receipt-only L1.1 admission")

    files = manifest.get("files")
    if not isinstance(files, list) or not files:
        raise HarnessError("dependency manifest files must be a nonempty array")
    by_relative: dict[PurePosixPath, tuple[Path, str, int]] = {}
    for index, raw_entry in enumerate(files):
        entry = _object(raw_entry, f"dependency manifest files[{index}]")
        relative = _manifest_relative(entry.get("path"), remote_root)
        if relative in by_relative:
            raise HarnessError(f"duplicate dependency manifest path: {relative}")
        expected = _exact_hash(entry.get("sha256"), f"dependency {relative} sha256")
        size = entry.get("size_bytes")
        if not isinstance(size, int) or size < 0:
            raise HarnessError(f"dependency {relative} has invalid size_bytes")
        host_path = deps_root.joinpath(*relative.parts)
        _require_file(host_path, f"dependency {relative}", executable=(relative == ROLE_PATHS["l11_service"]))
        if host_path.stat().st_size != size:
            raise HarnessError(
                f"dependency {relative} size mismatch: expected={size} "
                f"actual={host_path.stat().st_size}",
                65,
            )
        _require_hash(host_path, expected, f"dependency {relative}")
        by_relative[relative] = (host_path, expected, size)
    if set(by_relative) != set(ROLE_PATHS.values()):
        raise HarnessError(
            "dependency manifest role set mismatch: "
            f"missing={sorted(str(path) for path in set(ROLE_PATHS.values()) - set(by_relative))} "
            f"unexpected={sorted(str(path) for path in set(by_relative) - set(ROLE_PATHS.values()))}"
        )
    dependencies = {
        role: Dependency(role, relative, *by_relative[relative])
        for role, relative in ROLE_PATHS.items()
    }
    return Plan(
        config_path=config_path,
        config_sha256=sha256(config_path),
        origin_machine_id_sha256=origin_hash,
        execution_lease_id=lease,
        candidate=candidate,
        candidate_sha256=candidate_hash,
        deployed_ibus_root=deployed,
        deps_root=deps_root,
        manifest_path=manifest_path,
        manifest_sha256=manifest_hash,
        receipt_embedded_mount_path=embedded,
        dependencies=dependencies,
    )


def validate_remote_guard(remote_worker: bool, origin_hash: str, machine_id: Path = Path("/etc/machine-id")) -> None:
    if not remote_worker:
        raise HarnessError("actual-client harness is remote-only; --remote-worker is required")
    _require_file(machine_id, "worker machine-id")
    current = sha256(machine_id)
    if current == origin_hash:
        raise HarnessError("refusing actual-client harness on the origin machine")
    if any(os.environ.get(key) != value for key, value in REQUIRED_GUARD.items()):
        raise HarnessError("existing remote resource guard and heavy lease required; use scripts/dev-check.py client")


def validate_host_prerequisites() -> None:
    for path in HOST_TOOLS:
        _require_file(path, "host prerequisite", executable=True)


def _fresh_output_path(value: str) -> Path:
    output = _absolute_path(value, "--output")
    if os.path.lexists(output):
        raise HarnessError(f"output already exists; refusing overwrite: {output}", 65)
    if not output.parent.is_dir():
        raise HarnessError(f"output parent does not exist: {output.parent}", 65)
    return output


def _write_generated_component(template_path: Path, target: Path) -> None:
    tree = ET.parse(template_path)
    exec_nodes = tree.getroot().findall("exec")
    if len(exec_nodes) != 1 or (exec_nodes[0].text or "").strip():
        raise HarnessError("Lay component template must have one empty exec element")
    exec_nodes[0].text = f"{SANDBOX_CANDIDATE} --ibus --managed"
    ET.indent(tree, space="  ")
    tree.write(target, encoding="utf-8", xml_declaration=True)


def prepare_output(plan: Plan, output: Path, harness_root: Path | None = None) -> None:
    root = harness_root or Path(__file__).resolve().parent
    driver_sha256 = verify_driver_identity(root)
    output.mkdir(mode=0o700)
    try:
        component = output / "component"
        component.mkdir(mode=0o700)
        (output / "home").mkdir(mode=0o700)
        for name in ("driver.py", "dbus.conf", "config.json", "readline_consumer.py"):
            shutil.copyfile(root / name, output / name)
        if (plan.scenario_set in ("manual-toggle", "terminal-delivery", "first-word", "first-word-us", "first-word-ru", "fresh-preedit")
                or plan.startup_proof_profile != "legacy"):
            config_path = output / "config.json"
            config = json.loads(config_path.read_text(encoding="utf-8"))
            config["nanda_precognition"] = True
            if plan.startup_proof_profile == "off":
                config.update({
                    "nanda_autocorrect": False,
                    "auto_replace": False,
                    "auto_switch_layout": False,
                    "typing_assist": False,
                })
            config_path.write_text(json.dumps(config, indent=2) + "\n", encoding="utf-8")
        shutil.copyfile(root / "component/seed-simple.xml", component / "seed-simple.xml")
        _write_generated_component(root / "component/lay.xml.template", component / "lay.xml")
        shutil.copyfile(plan.config_path, output / "harness-config.json")
        metadata = {
            "schema": RUN_METADATA_SCHEMA,
            "proof_contract": PROOF_CONTRACT,
            "configuration_schema": SCHEMA,
            "startup_schedule": plan.startup_schedule,
            "scenario_set": plan.scenario_set,
            "startup_proof_profile": plan.startup_proof_profile,
            "readline_consumer_sha256": sha256(root / "readline_consumer.py"),
            "private_config_sha256": sha256(output / "config.json"),
            "runtime_authority_changed": False,
            "config_sha256": plan.config_sha256,
            "candidate": {
                "configured_path": str(plan.candidate),
                "sandbox_path": str(SANDBOX_CANDIDATE),
                "sha256": plan.candidate_sha256,
            },
            "dependency_manifest": {
                "configured_path": str(plan.manifest_path),
                "sha256": plan.manifest_sha256,
                "validated_file_count": len(plan.dependencies),
                "receipt_embedded_mount_path": str(plan.receipt_embedded_mount_path),
            },
            "driver": {
                "version": "v2",
                "sha256": driver_sha256,
                "predecessor": V1_DRIVER_PROVENANCE,
                "comparison": "SUCCESSOR_CONTRACT_NOT_BASELINE_PARITY",
                "contract_changes": [
                    "advertise and publish SurroundingText",
                    "apply native-unhandled fixture input in the client",
                ],
            },
            "resource_envelope": {
                "id": RESOURCE_ENVELOPE_ID,
                "cpu_quota": "200%",
                "memory_max": "1536M",
                "memory_swap_max": 0,
                "tasks_max": 128,
                "runtime_max": "90s",
            },
        }
        (output / "run-metadata.json").write_text(
            json.dumps(metadata, ensure_ascii=False, indent=2) + "\n", encoding="utf-8"
        )
    except Exception:
        shutil.rmtree(output)
        raise


def _embedded_directory_args(path: Path) -> list[str]:
    args: list[str] = []
    current = Path("/home")
    relative = path.relative_to(current)
    for part in relative.parts:
        current /= part
        args.extend(("--dir", str(current)))
    return args


def _sandbox_dependency(dependency: Dependency) -> str:
    return str(SANDBOX_DEPS_ROOT.joinpath(*dependency.relative_path.parts))


def build_command(plan: Plan, output: Path) -> list[str]:
    unit = f"lay-ime-client-{os.getpid()}-{uuid.uuid4().hex[:12]}"
    embedded = plan.receipt_embedded_mount_path
    role = plan.dependencies
    packages_absent = plan.startup_proof_profile == "absent"
    dependency_mount_args = [] if packages_absent else [
        "--ro-bind", str(plan.deps_root), str(SANDBOX_DEPS_ROOT),
        "--ro-bind", str(plan.deps_root / "l1.1"), str(embedded),
    ]
    dependency_env_args = [] if packages_absent else [
        "--setenv", "LAY_L11_SERVICE_BIN", _sandbox_dependency(role["l11_service"]),
        "--setenv", "EXPECTED_L11_SERVICE_SHA256", role["l11_service"].sha256,
        "--setenv", "TD121_L11_PACKAGE_PATH", str(embedded / role["l11_package"].relative_path.name),
        "--setenv", "EXPECTED_TD121_L11_PACKAGE_SHA256", role["l11_package"].sha256,
        "--setenv", "TD121_L11_PROOF_PATH", str(embedded / role["l11_proof"].relative_path.name),
        "--setenv", "EXPECTED_TD121_L11_PROOF_SHA256", role["l11_proof"].sha256,
        "--setenv", "LAY_L11_RECEIPT", str(embedded / "active.installed.json"),
        "--setenv", "EXPECTED_LAY_L11_RECEIPT_SHA256", role["l11_active_receipt"].sha256,
        "--setenv", "LAY_L2_LEXICAL_PHASE_MEMORY", _sandbox_dependency(role["l2_lexical_phase"]),
        "--setenv", "LAY_L2_PACKAGE", _sandbox_dependency(role["l2_v13"]),
        "--setenv", "EXPECTED_LAY_L2_PACKAGE_SHA256", role["l2_v13"].sha256,
        "--setenv", "LAY_L2_PRODUCTIVE_V1_PACKAGE", _sandbox_dependency(role["productive_v90_package"]),
        "--setenv", "EXPECTED_LAY_L2_PRODUCTIVE_V1_PACKAGE_SHA256", role["productive_v90_package"].sha256,
        "--setenv", "TD121_PRODUCTIVE_V90_RECOVERY_PATH", _sandbox_dependency(role["productive_v90_recovery"]),
        "--setenv", "EXPECTED_TD121_PRODUCTIVE_V90_RECOVERY_SHA256", role["productive_v90_recovery"].sha256,
        "--setenv", "LAY_L2_V13_DAFSA", _sandbox_dependency(role["l2_v13_dafsa"]),
        "--setenv", "EXPECTED_LAY_L2_V13_DAFSA_SHA256", role["l2_v13_dafsa"].sha256,
    ]
    authority_env_names = (
        "LAY_L11_SERVICE_BIN", "EXPECTED_L11_SERVICE_SHA256",
        "TD121_L11_PACKAGE_PATH", "EXPECTED_TD121_L11_PACKAGE_SHA256",
        "TD121_L11_PROOF_PATH", "EXPECTED_TD121_L11_PROOF_SHA256",
        "LAY_L11_RECEIPT", "EXPECTED_LAY_L11_RECEIPT_SHA256",
        "LAY_L2_LEXICAL_PHASE_MEMORY", "LAY_L2_PACKAGE",
        "EXPECTED_LAY_L2_PACKAGE_SHA256", "LAY_L2_PRODUCTIVE_V1_PACKAGE",
        "EXPECTED_LAY_L2_PRODUCTIVE_V1_PACKAGE_SHA256",
        "TD121_PRODUCTIVE_V90_RECOVERY_PATH",
        "EXPECTED_TD121_PRODUCTIVE_V90_RECOVERY_SHA256",
        "LAY_L2_V13_DAFSA", "EXPECTED_LAY_L2_V13_DAFSA_SHA256",
    )
    inherited_authority_unsets = [argument for name in authority_env_names
                                  for argument in ("--unsetenv", name)] if packages_absent else []
    absence_env_args = [
        "--setenv", "LAY_L11_MODEL_DIR", "/tmp/proof/no-models/l1.1",
        "--setenv", "LAY_L2_MODEL_DIR", "/tmp/proof/no-models/l2",
    ] if packages_absent else []
    command = [
        "/usr/bin/systemd-run",
        "--user",
        f"--unit={unit}",
        "--collect",
        "--wait",
        "--pipe",
        "--property=CPUQuota=200%",
        "--property=MemoryMax=1536M",
        "--property=MemorySwapMax=0",
        "--property=TasksMax=128",
        "--property=RuntimeMaxSec=90s",
        "--property=Nice=19",
        "/usr/bin/bwrap",
        "--unshare-all",
        "--die-with-parent",
        "--new-session",
        "--ro-bind",
        "/",
        "/",
        "--tmpfs",
        "/tmp",
        "--tmpfs",
        "/run",
        "--proc",
        "/proc",
        "--dev",
        "/dev",
        "--tmpfs",
        "/home",
        "--dir",
        "/tmp/candidate",
        "--dir",
        "/tmp/deps",
        *_embedded_directory_args(embedded),
        "--ro-bind",
        str(plan.candidate),
        str(SANDBOX_CANDIDATE),
        "--ro-bind",
        str(plan.deployed_ibus_root),
        str(SANDBOX_DEPLOYED_ROOT),
        *dependency_mount_args,
        "--bind",
        str(output),
        str(SANDBOX_PROOF_ROOT),
        "--chdir",
        str(SANDBOX_PROOF_ROOT),
        "--unsetenv",
        "DISPLAY",
        "--unsetenv",
        "WAYLAND_DISPLAY",
        "--unsetenv",
        "DBUS_SESSION_BUS_ADDRESS",
        "--unsetenv",
        "IBUS_ADDRESS",
        "--unsetenv",
        "DBUS_STARTER_ADDRESS",
        "--unsetenv",
        "DBUS_STARTER_BUS_TYPE",
        "--unsetenv",
        "LD_LIBRARY_PATH",
        "--unsetenv",
        "LD_PRELOAD",
        *inherited_authority_unsets,
        "--setenv",
        "HOME",
        "/tmp/proof/home",
        "--setenv",
        "IME_CLIENT_CANDIDATE",
        str(SANDBOX_CANDIDATE),
        "--setenv",
        "IME_CLIENT_STARTUP_SCHEDULE",
        plan.startup_schedule,
        "--setenv",
        "IME_CLIENT_SCENARIO_SET",
        plan.scenario_set,
        "--setenv",
        "IME_CLIENT_STARTUP_PROOF_PROFILE",
        plan.startup_proof_profile,
        "--setenv",
        "EXPECTED_CANDIDATE_SHA256",
        plan.candidate_sha256,
        "--setenv",
        "TD121_EXECUTION_LEASE_ID",
        plan.execution_lease_id,
        "--setenv",
        "TD121_RESOURCE_ENVELOPE_ID",
        RESOURCE_ENVELOPE_ID,
        *dependency_env_args,
        *absence_env_args,
        "--setenv",
        "XDG_RUNTIME_DIR",
        "/tmp/proof/runtime",
        "--setenv",
        "XDG_CACHE_HOME",
        "/tmp/proof/cache",
        "--setenv",
        "XDG_DATA_HOME",
        "/tmp/proof/data",
        "--setenv",
        "XDG_CONFIG_HOME",
        "/tmp/proof/config",
        "/usr/bin/dbus-run-session",
        "--config-file=/tmp/proof/dbus.conf",
        "--",
        "/usr/bin/python3",
        "/tmp/proof/driver.py",
    ]
    return command


def execute(command: list[str], output: Path) -> int:
    with (output / "run.log").open("xb") as log:
        process = subprocess.Popen(command, stdout=subprocess.PIPE, stderr=subprocess.STDOUT)
        assert process.stdout is not None
        for block in iter(lambda: process.stdout.read(64 * 1024), b""):
            log.write(block)
            log.flush()
            sys.stdout.buffer.write(block)
            sys.stdout.buffer.flush()
        return process.wait()


def validate_startup_proof_combination(plan: Plan) -> None:
    allowed = {
        "legacy": {"restoration", "lifecycle", "manual-toggle", "terminal-delivery", "first-word", "first-word-us", "first-word-ru"},
        "on": {"first-word", "first-word-us", "first-word-ru", "fresh-preedit", "startup-only"},
        "off": {"fresh-preedit", "startup-only"},
        "absent": {"packages-absent-literal", "startup-only"},
    }
    if (plan.scenario_set not in allowed[plan.startup_proof_profile]
            or (plan.startup_proof_profile != "legacy" and plan.startup_schedule != "immediate")):
        raise HarnessError("unsupported startup proof profile/scenario/schedule combination")


def parser() -> argparse.ArgumentParser:
    result = argparse.ArgumentParser(description=__doc__)
    result.add_argument("--remote-worker", action="store_true")
    result.add_argument("--config", required=True)
    result.add_argument("--output", required=True)
    result.add_argument("--startup-schedule", choices=("immediate", "post-exact-ready"),
                        default="immediate")
    result.add_argument("--scenario-set", choices=("restoration", "lifecycle", "manual-toggle", "terminal-delivery", "first-word",
                                                   "first-word-us", "first-word-ru", "fresh-preedit", "startup-only", "packages-absent-literal"),
                        default="restoration")
    result.add_argument("--startup-proof-profile", choices=("legacy", "on", "off", "absent"), default="legacy")
    return result


def main(argv: list[str] | None = None) -> int:
    args = parser().parse_args(argv)
    try:
        output = _fresh_output_path(args.output)
        plan = replace(load_plan(Path(args.config)), startup_schedule=args.startup_schedule,
                       scenario_set=args.scenario_set, startup_proof_profile=args.startup_proof_profile)
        validate_startup_proof_combination(plan)
        validate_remote_guard(args.remote_worker, plan.origin_machine_id_sha256)
        validate_host_prerequisites()
        prepare_output(plan, output)
        return execute(build_command(plan, output), output)
    except HarnessError as error:
        print(f"ime-client harness refused: {error}", file=sys.stderr)
        return error.code


if __name__ == "__main__":
    raise SystemExit(main())
