#!/usr/bin/env python3
"""Regression tests for the live release transaction controller."""

from __future__ import annotations

import difflib
import hashlib
import os
import re
import shutil
import stat
import subprocess
import tempfile
import time
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
DONOR_CONTROLLER = ROOT / "scripts" / "install-live-release-1.0.65.sh"
CONTROLLER = ROOT / "scripts" / "install-live-release-1.0.66.sh"
RUNTIME_CONTROLLER = ROOT / "scripts" / "lay-runtime-control.sh"
DONOR_SHA256 = "962f4b7500758cf1cf421e72695b10695d5b49d29e696782bd4060144ea96e66"
PREPARED_CONTROLLER_SHA256 = (
    "abe1d7f1b35bfc6b7456699326bb6137675bfd5c8eafe2e7c52fd034f0255ae9"
)
CONTROLLER_SHA256 = "d8a8e8b82fc9ec94f7b985e310f5831684461f536a53c0c68ea4d1e40726129a"
CONTROLLER_DELTA_SHA256 = (
    "e3ec4f97d914c752f00fbff3250bc1be622aa27d7bcbcc1f7cb43f810d587321"
)
RUNTIME_CONTROLLER_SHA256 = (
    "746d077115277775acb275e077b57823ace9e416d2575eaf0bc53d3327d64e6d"
)


def command_output(*command: str) -> str | None:
    result = subprocess.run(
        command,
        check=False,
        capture_output=True,
        text=True,
        timeout=5,
    )
    value = result.stdout.strip()
    return value or None


def file_sha256(path: Path) -> str | None:
    if not path.is_file():
        return None
    return hashlib.sha256(path.read_bytes()).hexdigest()


def tree_projection(root: Path) -> tuple[tuple[str, bytes, int], ...]:
    return tuple(
        (
            path.relative_to(root).as_posix(),
            path.read_bytes(),
            stat.S_IMODE(path.stat().st_mode),
        )
        for path in sorted(root.rglob("*"))
        if path.is_file()
    )


def run_controller(snapshot: Path, home: Path) -> subprocess.CompletedProcess[str]:
    environment = os.environ.copy()
    environment["HOME"] = str(home)
    return subprocess.run(
        [str(CONTROLLER), "--snapshot", str(snapshot)],
        cwd=ROOT,
        env=environment,
        check=False,
        capture_output=True,
        text=True,
        timeout=10,
    )


def create_isolated_live_snapshot(home: Path) -> tuple[Path, dict[str, Path]]:
    live_trees = {
        "bin": home / ".local/lib/lay/bin",
        "extension": home
        / ".local/share/gnome-shell/extensions/lay@radislabus-star.github.io",
        "l2": home / ".local/share/lay/nanda_wave/l2",
        "systemd": home / ".config/systemd/user",
    }
    expected_counts = {"bin": 19, "extension": 9, "l2": 7, "systemd": 1}
    expected_modes = {
        "bin": 0o755,
        "extension": 0o644,
        "l2": 0o644,
        "systemd": 0o644,
    }
    for tree_name, live_tree in live_trees.items():
        live_tree.mkdir(parents=True)
        for index in range(expected_counts[tree_name]):
            path = (
                live_tree / "lay-l3-online.service"
                if tree_name == "systemd"
                else live_tree / f"file-{index:02d}"
            )
            path.write_text(f"{tree_name}-{index}\n", encoding="utf-8")
            path.chmod(expected_modes[tree_name])

    snapshot = (
        home
        / ".local/state/lay/release-backups/1.0.66-preinstall-controller-test"
    )
    for tree_name, live_tree in live_trees.items():
        shutil.copytree(live_tree, snapshot / tree_name)
    return snapshot, live_trees


def live_projection() -> tuple[str | None, ...]:
    return (
        command_output("pgrep", "-xo", "ibus-daemon"),
        command_output(
            "systemctl", "--user", "show", "lay-daemon.service", "-p", "MainPID", "--value"
        ),
        command_output(
            "systemctl", "--user", "show", "lay-l3-online.service", "-p", "MainPID", "--value"
        ),
        file_sha256(Path.home() / ".local/lib/lay/bin/lay"),
        file_sha256(Path.home() / ".local/lib/lay/bin/lay-daemon"),
        file_sha256(Path.home() / ".local/lib/lay/bin/lay-ibus-engine"),
    )


def l11_lifecycle_functions() -> str:
    source = CONTROLLER.read_text(encoding="utf-8")
    return source[
        source.index("l11_normalize_exe_path() {") : source.index(
            "# End release-local L1.1 lifecycle"
        )
    ]


def l11_installed_recovery_function() -> str:
    source = CONTROLLER.read_text(encoding="utf-8")
    return source[
        source.index("l11_run_installed_recovery() {") : source.index(
            "verify_process_parity() {"
        )
    ]


def tree_fingerprint_function() -> str:
    source = CONTROLLER.read_text(encoding="utf-8")
    return source[
        source.index("tree_fingerprint() {") : source.index("verify_tree_parity() {")
    ]


def installed_recovery_tree_functions() -> str:
    source = CONTROLLER.read_text(encoding="utf-8")
    return source[
        source.index("capture_installed_recovery_fingerprints() {") : source.index(
            "installed_recovery_projection_unchanged() {"
        )
    ]


def forward_rollback_function() -> str:
    source = CONTROLLER.read_text(encoding="utf-8")
    return source[
        source.index("run_forward_rollback() {") : source.index(
            '[[ -d "$SNAPSHOT_DIR"'
        )
    ]


def forward_transaction_dispatch() -> str:
    source = CONTROLLER.read_text(encoding="utf-8")
    start = source.index("(\n    set -euo pipefail")
    return source[start : source.index("exit $?", start) + len("exit $?")]


def release_file_functions() -> str:
    source = CONTROLLER.read_text(encoding="utf-8")
    return source[source.index("file_count() {") : source.index("self_test() {")]


def installed_exact_v13_verifier() -> str:
    source = CONTROLLER.read_text(encoding="utf-8")
    verifier = source[
        source.index("verify_installed_exact_v13_sidecar() {") : source.index(
            "self_test() {"
        )
    ]
    return verifier.replace(
        "verify_installed_exact_v13_sidecar() {",
        "verify_installed_exact_v13_sidecar_production() {",
        1,
    )


def controller_delta(prepared_source: str, source: str) -> str:
    return "".join(
        difflib.unified_diff(
            prepared_source.splitlines(keepends=True),
            source.splitlines(keepends=True),
            fromfile="prepared-1.0.66",
            tofile="remote-sidecar-1.0.66",
        )
    )


def write_exact_v13_receipt(
    receipt: Path, package: Path, compiler: Path, sidecar: Path
) -> None:
    receipt.write_text(
        "\n".join(
            (
                "schema=lay.release.exact-v13-sidecar.v1",
                "release_version=1.0.66",
                "compile_operation=target/release/lay-nanda-wave-train "
                "--compile-v13-exact-sidecar LAY-L2-RU-FULL-v13.bin "
                "--out LAY-L2-RU-FULL-v13.dafsa",
                "compile_exit_status=0",
                "package_name=LAY-L2-RU-FULL-v13.bin",
                f"package_bytes={package.stat().st_size}",
                f"package_sha256={file_sha256(package)}",
                "compiler_name=lay-nanda-wave-train",
                f"compiler_bytes={compiler.stat().st_size}",
                f"compiler_sha256={file_sha256(compiler)}",
                "sidecar_name=LAY-L2-RU-FULL-v13.dafsa",
                f"sidecar_bytes={sidecar.stat().st_size}",
                f"sidecar_sha256={file_sha256(sidecar)}",
            )
        )
        + "\n",
        encoding="utf-8",
    )
    receipt.chmod(0o644)


def create_exact_v13_fixture(root: Path) -> dict[str, Path]:
    scripts = root / "scripts"
    release = root / "target/release"
    l2 = root / "live/l2"
    scripts.mkdir(parents=True)
    release.mkdir(parents=True)
    l2.mkdir(parents=True)
    package = root / "input/LAY-L2-RU-FULL-v13.bin"
    package.parent.mkdir(parents=True)
    package.write_bytes(b"canonical-package")
    compiler = release / "lay-nanda-wave-train"
    compiler.write_bytes(b"#!/usr/bin/env bash\nexit 97\n")
    compiler.chmod(0o755)
    sidecar = release / "LAY-L2-RU-FULL-v13.dafsa"
    sidecar.write_bytes(b"remotely-compiled-exact-v13")
    sidecar.chmod(0o644)
    installed = l2 / sidecar.name
    installed.write_bytes(b"rollback-sidecar")
    installed.chmod(0o640)
    contract = scripts / "l2-package-contract.sh"
    contract.write_text(
        "\n".join(
            (
                '#!/usr/bin/env bash',
                'LAY_CANONICAL_L2_PACKAGE_NAME="LAY-L2-RU-FULL-v13.bin"',
                f'LAY_CANONICAL_L2_PACKAGE_BYTES="{package.stat().st_size}"',
                f'LAY_CANONICAL_L2_PACKAGE_SHA256="{file_sha256(package)}"',
            )
        )
        + "\n",
        encoding="utf-8",
    )
    resolver = scripts / "resolve-l2-package.sh"
    resolver.write_text(
        f"#!/usr/bin/env bash\nprintf '%s\\n' '{package}'\n", encoding="utf-8"
    )
    resolver.chmod(0o755)
    receipt = release / "LAY-L2-RU-FULL-v13.dafsa.release-receipt"
    write_exact_v13_receipt(receipt, package, compiler, sidecar)
    return {
        "root": root,
        "l2": l2,
        "package": package,
        "compiler": compiler,
        "sidecar": sidecar,
        "installed": installed,
        "receipt": receipt,
    }


def run_exact_v13_harness(
    fixture: dict[str, Path], body: str
) -> subprocess.CompletedProcess[str]:
    harness = f"""
set -u -o pipefail
ROOT="$TEST_ROOT"
EXPECTED_VERSION=1.0.66
L2_DIR="$TEST_L2"
EXACT_V13_SIDECAR_NAME=LAY-L2-RU-FULL-v13.dafsa
EXACT_V13_SIDECAR_SOURCE="$ROOT/target/release/$EXACT_V13_SIDECAR_NAME"
EXACT_V13_SIDECAR_RECEIPT="$EXACT_V13_SIDECAR_SOURCE.release-receipt"
EXACT_V13_COMPILER="$ROOT/target/release/lay-nanda-wave-train"
EXACT_V13_COMPILE_OPERATION="target/release/lay-nanda-wave-train --compile-v13-exact-sidecar LAY-L2-RU-FULL-v13.bin --out LAY-L2-RU-FULL-v13.dafsa"
EXACT_V13_PACKAGE_SOURCE=
EXACT_V13_PACKAGE_BYTES=
EXACT_V13_PACKAGE_SHA256=
EXACT_V13_COMPILER_BYTES=
EXACT_V13_COMPILER_SHA256=
EXACT_V13_SIDECAR_BYTES=
EXACT_V13_SIDECAR_SHA256=
EXACT_V13_RECEIPT_SHA256=
{release_file_functions()}
{body}
"""
    environment = os.environ.copy()
    environment.update(
        {"TEST_ROOT": str(fixture["root"]), "TEST_L2": str(fixture["l2"])}
    )
    return subprocess.run(
        ["bash", "-c", harness],
        env=environment,
        check=False,
        capture_output=True,
        text=True,
        timeout=10,
    )


class ReleaseLiveInstallControllerTest(unittest.TestCase):
    def test_controller_delta_is_scoped_and_donor_is_pinned(self) -> None:
        source = CONTROLLER.read_text(encoding="utf-8")
        donor_source = DONOR_CONTROLLER.read_text(encoding="utf-8")
        self.assertEqual(1, donor_source.count("EXPECTED_VERSION=1.0.65"))
        self.assertEqual(1, donor_source.count("ROLLBACK_VERSION=1.0.64"))
        self.assertEqual(1, donor_source.count("1.0.65-preinstall-"))
        prepared_source = donor_source.replace(
            "EXPECTED_VERSION=1.0.65", "EXPECTED_VERSION=1.0.66", 1
        ).replace(
            "ROLLBACK_VERSION=1.0.64", "ROLLBACK_VERSION=1.0.65", 1
        ).replace(
            "1.0.65-preinstall-", "1.0.66-preinstall-", 1
        )
        self.assertEqual(DONOR_SHA256, file_sha256(DONOR_CONTROLLER))
        self.assertEqual(
            PREPARED_CONTROLLER_SHA256,
            hashlib.sha256(prepared_source.encode()).hexdigest(),
        )
        self.assertEqual(CONTROLLER_SHA256, file_sha256(CONTROLLER))
        self.assertEqual(
            CONTROLLER_DELTA_SHA256,
            hashlib.sha256(controller_delta(prepared_source, source).encode()).hexdigest(),
        )
        self.assertNotEqual(prepared_source, source)
        self.assertEqual(1, source.count("EXPECTED_VERSION=1.0.66"))
        self.assertEqual(1, source.count("ROLLBACK_VERSION=1.0.65"))
        self.assertEqual(
            1,
            source.count(
                '"${HOME}/.local/state/lay/release-backups/1.0.66-preinstall-"*'
            ),
        )
        for required in (
            "capture_exact_v13_sidecar_admission",
            "exact_v13_sidecar_sources_unchanged",
            "verify_installed_exact_v13_sidecar",
            "LAY_SKIP_EXACT_V13_SIDECAR=1",
        ):
            self.assertIn(required, source)
        self.assertTrue(os.access(CONTROLLER, os.X_OK))

    def test_remote_sidecar_preflight_copy_and_readback_are_exact(self) -> None:
        with tempfile.TemporaryDirectory(prefix="lay-exact-v13-valid-") as temporary:
            fixture = create_exact_v13_fixture(Path(temporary))
            result = run_exact_v13_harness(
                fixture,
                """
capture_exact_v13_sidecar_admission || exit 41
exact_v13_sidecar_sources_unchanged || exit 42
install_file_atomic "$EXACT_V13_SIDECAR_SOURCE" "$L2_DIR/$EXACT_V13_SIDECAR_NAME" 0644 || exit 43
verify_installed_exact_v13_sidecar || exit 44
""",
            )
            self.assertEqual(0, result.returncode, result.stderr)
            self.assertEqual(fixture["sidecar"].read_bytes(), fixture["installed"].read_bytes())
            self.assertEqual(0o644, stat.S_IMODE(fixture["installed"].stat().st_mode))

            fixture["installed"].write_bytes(b"post-copy-drift")
            rejected = run_exact_v13_harness(
                fixture,
                """
capture_exact_v13_sidecar_admission || exit 51
verify_installed_exact_v13_sidecar
""",
            )
            self.assertNotEqual(0, rejected.returncode)

    def test_remote_inputs_missing_empty_and_symlink_fail_before_mutation(self) -> None:
        for target in ("package", "compiler", "sidecar", "receipt"):
            for fault in ("missing", "empty", "symlink"):
                with self.subTest(
                    target=target, fault=fault
                ), tempfile.TemporaryDirectory(
                    prefix=f"lay-exact-v13-{target}-{fault}-"
                ) as temporary:
                    fixture = create_exact_v13_fixture(Path(temporary))
                    artifact = fixture[target]
                    if fault == "missing":
                        artifact.unlink()
                    elif fault == "empty":
                        artifact.write_bytes(b"")
                    else:
                        external = fixture["root"] / f"external-{target}"
                        external.write_bytes(f"foreign-{target}".encode())
                        if target == "compiler":
                            external.chmod(0o755)
                        artifact.unlink()
                        artifact.symlink_to(external)
                    marker = fixture["root"] / "mutation"
                    result = run_exact_v13_harness(
                        fixture,
                        """
capture_exact_v13_sidecar_admission || exit 61
printf mutation >"$TEST_ROOT/mutation"
""",
                    )
                    self.assertNotEqual(0, result.returncode)
                    self.assertFalse(marker.exists(), result.stderr)

    def test_remote_sidecar_receipt_is_strict_and_not_caller_overridable(self) -> None:
        mutations = {
            "wrong_package_hash": lambda fixture: fixture["receipt"].write_text(
                fixture["receipt"].read_text(encoding="utf-8").replace(
                    f"package_sha256={file_sha256(fixture['package'])}",
                    "package_sha256=" + "2" * 64,
                ),
                encoding="utf-8",
            ),
            "wrong_sidecar_hash": lambda fixture: fixture["receipt"].write_text(
                fixture["receipt"].read_text(encoding="utf-8").replace(
                    f"sidecar_sha256={file_sha256(fixture['sidecar'])}",
                    "sidecar_sha256=" + "0" * 64,
                ),
                encoding="utf-8",
            ),
            "wrong_compiler_hash": lambda fixture: fixture["receipt"].write_text(
                fixture["receipt"].read_text(encoding="utf-8").replace(
                    f"compiler_sha256={file_sha256(fixture['compiler'])}",
                    "compiler_sha256=" + "1" * 64,
                ),
                encoding="utf-8",
            ),
            "compile_not_successful": lambda fixture: fixture["receipt"].write_text(
                fixture["receipt"].read_text(encoding="utf-8").replace(
                    "compile_exit_status=0", "compile_exit_status=1"
                ),
                encoding="utf-8",
            ),
            "unknown_field": lambda fixture: fixture["receipt"].write_text(
                fixture["receipt"].read_text(encoding="utf-8")
                + "expected_sha256=caller-value\n",
                encoding="utf-8",
            ),
            "duplicate_field": lambda fixture: fixture["receipt"].write_text(
                fixture["receipt"].read_text(encoding="utf-8") + "sidecar_bytes=1\n",
                encoding="utf-8",
            ),
            "nul_in_key": lambda fixture: fixture["receipt"].write_bytes(
                fixture["receipt"].read_bytes().replace(
                    b"compile_exit_status", b"compile_exit_\0status", 1
                )
            ),
            "nul_in_value": lambda fixture: fixture["receipt"].write_bytes(
                fixture["receipt"].read_bytes().replace(
                    b"compile_exit_status=0", b"compile_exit_status=0\0", 1
                )
            ),
        }
        for fault, mutate in mutations.items():
            with self.subTest(fault=fault), tempfile.TemporaryDirectory(
                prefix=f"lay-exact-v13-receipt-{fault}-"
            ) as temporary:
                fixture = create_exact_v13_fixture(Path(temporary))
                mutate(fixture)
                fixture["receipt"].chmod(0o644)
                result = run_exact_v13_harness(
                    fixture,
                    """
capture_exact_v13_sidecar_admission || exit 71
printf mutation >"$TEST_ROOT/mutation"
""",
                )
                self.assertNotEqual(0, result.returncode)
                self.assertFalse((fixture["root"] / "mutation").exists(), result.stderr)

    def test_remote_sidecar_all_source_identity_drift_is_rejected_before_copy(self) -> None:
        for source_name in ("package", "compiler", "sidecar", "receipt"):
            with self.subTest(source=source_name), tempfile.TemporaryDirectory(
                prefix=f"lay-exact-v13-drift-{source_name}-"
            ) as temporary:
                fixture = create_exact_v13_fixture(Path(temporary))
                result = run_exact_v13_harness(
                    fixture,
                    f"""
capture_exact_v13_sidecar_admission || exit 81
printf drift >>"{fixture[source_name]}"
exact_v13_sidecar_sources_unchanged && exit 82
[[ ! -e "$TEST_ROOT/copy" ]]
""",
                )
                self.assertEqual(0, result.returncode, result.stderr)
                self.assertFalse((fixture["root"] / "copy").exists())

    def test_remote_sidecar_preflight_precedes_services_and_forward_never_runs_train(self) -> None:
        source = CONTROLLER.read_text(encoding="utf-8")
        preflight = source.index("capture_exact_v13_sidecar_admission || {")
        first_service_observation = source.index(
            "systemctl --user is-active --quiet lay-daemon.service", preflight
        )
        forward_start = source.index("(\n    set -euo pipefail")
        forward_end = source.index(")\nforward_rc=$?", forward_start)
        forward = source[forward_start:forward_end]
        verifier = source[
            source.index("verify_forward() {") : source.index(
                "verify_rollback_runtime() {"
            )
        ]

        self.assertLess(preflight, first_service_observation)
        self.assertIn("LAY_SKIP_EXACT_V13_SIDECAR=1", forward)
        for binding in (
            'LAY_L2_PACKAGE_NAME="$LAY_CANONICAL_L2_PACKAGE_NAME"',
            'LAY_L2_PACKAGE_BYTES="$LAY_CANONICAL_L2_PACKAGE_BYTES"',
            'LAY_L2_PACKAGE_SHA256="$LAY_CANONICAL_L2_PACKAGE_SHA256"',
            'LAY_L2_MODEL_DIR="$L2_DIR"',
            'LAY_RELEASE_SOURCE_DIR="$ROOT/target/release"',
            'LAY_INSTALL_LIBEXEC_DIR="$INSTALL_DIR"',
            'LAY_INSTALL_BIN_DIR="$LINK_DIR"',
        ):
            self.assertIn(binding, forward)
        self.assertEqual(2, forward.count("exact_v13_sidecar_sources_unchanged"))
        first_revalidation = forward.index("exact_v13_sidecar_sources_unchanged")
        installer = forward.index("LAY_SKIP_EXACT_V13_SIDECAR=1")
        second_revalidation = forward.index(
            "exact_v13_sidecar_sources_unchanged", first_revalidation + 1
        )
        copy = forward.index("install_file_atomic", second_revalidation)
        self.assertLess(first_revalidation, installer)
        self.assertLess(installer, second_revalidation)
        self.assertLess(second_revalidation, copy)
        self.assertIn("install_file_atomic", forward)
        self.assertIn("verify_installed_exact_v13_sidecar", forward)
        self.assertNotIn("--compile-v13-exact-sidecar", forward)
        self.assertNotIn("--compile-v13-exact-sidecar", verifier)
        self.assertNotRegex(
            forward,
            re.compile(r'\"\$EXACT_V13_COMPILER\"|lay-nanda-wave-train\s+--compile'),
        )

    def test_remote_sidecar_post_copy_failure_restores_seven_l2_files_and_modes(self) -> None:
        with tempfile.TemporaryDirectory(prefix="lay-exact-v13-rollback-") as temporary:
            root = Path(temporary)
            fixture = create_exact_v13_fixture(root / "release")
            snapshot = root / "snapshot"
            live = fixture["root"] / "live"
            for tree in ("bin", "extension", "l2"):
                (snapshot / tree).mkdir(parents=True)
                (live / tree).mkdir(parents=True, exist_ok=True)
            (snapshot / "systemd").mkdir(parents=True)
            (live / "systemd").mkdir(parents=True)
            for index in range(7):
                name = "LAY-L2-RU-FULL-v13.dafsa" if index == 0 else f"model-{index}"
                path = snapshot / "l2" / name
                path.write_bytes(f"rollback-{index}".encode())
                path.chmod(0o640 if index == 0 else 0o644)
            shutil.copytree(snapshot / "l2", fixture["l2"], dirs_exist_ok=True)
            for tree in ("bin", "extension"):
                path = snapshot / tree / "artifact"
                path.write_bytes(f"rollback-{tree}".encode())
                path.chmod(0o755 if tree == "bin" else 0o640)
                shutil.copy2(path, live / tree / "artifact")
            snapshot_unit = snapshot / "systemd/lay-l3-online.service"
            live_unit = live / "systemd/lay-l3-online.service"
            snapshot_unit.write_bytes(b"rollback-unit")
            snapshot_unit.chmod(0o640)
            shutil.copy2(snapshot_unit, live_unit)
            before = tree_projection(snapshot / "l2")
            installer = fixture["root"] / "scripts/install-release-binaries.sh"
            installer.write_text(
                """#!/usr/bin/env bash
set -u
[[ "$LAY_SKIP_EXACT_V13_SIDECAR" == 1 ]] || exit 31
[[ "$LAY_L2_OFFLINE" == 1 ]] || exit 32
printf '%s\\n' "$LAY_SKIP_EXACT_V13_SIDECAR" "$LAY_L2_OFFLINE" \\
    "$LAY_L2_PACKAGE_SOURCE" "$LAY_L2_PACKAGE_NAME" "$LAY_L2_PACKAGE_BYTES" \\
    "$LAY_L2_PACKAGE_SHA256" "$LAY_L2_MODEL_DIR" "$LAY_RELEASE_SOURCE_DIR" \\
    "$LAY_INSTALL_LIBEXEC_DIR" "$LAY_INSTALL_BIN_DIR" >"$TEST_INSTALLER_ENV"
""",
                encoding="utf-8",
            )
            installer.chmod(0o755)
            extension_check = fixture["root"] / "scripts/check-gnome-extension-runtime.sh"
            extension_check.write_text(
                "#!/usr/bin/env bash\nprintf 'extension:%s\\n' \"$*\" >>\"$TEST_CALLS\"\n",
                encoding="utf-8",
            )
            extension_check.chmod(0o755)
            runtime_control = fixture["root"] / "scripts/lay-runtime-control.sh"
            runtime_control.write_text(
                "#!/usr/bin/env bash\nprintf 'runtime:%s\\n' \"$1\" >>\"$TEST_CALLS\"\n",
                encoding="utf-8",
            )
            runtime_control.chmod(0o755)
            l3_source = fixture["root"] / "systemd/lay-l3-online.service"
            l3_source.parent.mkdir()
            l3_source.write_bytes(b"forward-unit")
            l3_source.chmod(0o644)
            calls = root / "calls"
            installer_environment = root / "installer-environment"
            copy_marker = root / "post-copy-marker"
            harness = f"""
set -u -o pipefail
ROOT="$TEST_ROOT"
EXPECTED_VERSION=1.0.66
ROLLBACK_VERSION=1.0.65
SNAPSHOT_DIR="$TEST_SNAPSHOT"
INSTALL_DIR="$TEST_LIVE/bin"
LINK_DIR="$TEST_LIVE/links"
EXTENSION_DIR="$TEST_LIVE/extension"
L2_DIR="$TEST_L2"
L3_UNIT_SOURCE="$ROOT/systemd/lay-l3-online.service"
L3_UNIT_PATH="$TEST_LIVE/systemd/lay-l3-online.service"
EXACT_V13_SIDECAR_NAME=LAY-L2-RU-FULL-v13.dafsa
EXACT_V13_SIDECAR_SOURCE="$ROOT/target/release/$EXACT_V13_SIDECAR_NAME"
EXACT_V13_SIDECAR_RECEIPT="$EXACT_V13_SIDECAR_SOURCE.release-receipt"
EXACT_V13_COMPILER="$ROOT/target/release/lay-nanda-wave-train"
EXACT_V13_COMPILE_OPERATION="target/release/lay-nanda-wave-train --compile-v13-exact-sidecar LAY-L2-RU-FULL-v13.bin --out LAY-L2-RU-FULL-v13.dafsa"
EXACT_V13_PACKAGE_SOURCE=
EXACT_V13_PACKAGE_BYTES=
EXACT_V13_PACKAGE_SHA256=
EXACT_V13_COMPILER_BYTES=
EXACT_V13_COMPILER_SHA256=
EXACT_V13_SIDECAR_BYTES=
EXACT_V13_SIDECAR_SHA256=
EXACT_V13_RECEIPT_SHA256=
L11_BINARY_NAME=lay-l1.1-serve
L11_ROLLBACK_HASH=rollback-l11
L11_RELEASE_HASH=release-l11
L11_UNVERIFIED_RC=70
{release_file_functions()}
{installed_exact_v13_verifier()}
verify_installed_exact_v13_sidecar() {{
    verify_file_bytes_mode "$EXACT_V13_SIDECAR_SOURCE" "$L2_DIR/$EXACT_V13_SIDECAR_NAME" 644 || return 91
    printf 'copied:%s:%s\\n' "$EXACT_V13_SIDECAR_BYTES" "$(stat -c '%a' -- "$L2_DIR/$EXACT_V13_SIDECAR_NAME")" >"$TEST_COPY_MARKER"
    printf 'post-copy-verifier-fault' >"$L2_DIR/$EXACT_V13_SIDECAR_NAME"
    chmod 0644 -- "$L2_DIR/$EXACT_V13_SIDECAR_NAME"
    verify_installed_exact_v13_sidecar_production
}}
{forward_rollback_function()}
systemctl() {{ printf 'systemctl:%s\\n' "$*" >>"$TEST_CALLS"; }}
select_verified_xkb() {{ printf 'select:xkb\\n' >>"$TEST_CALLS"; }}
l11_stop_current_process() {{ printf 'l11:stop\\n' >>"$TEST_CALLS"; }}
l11_transition_to_binary() {{ printf 'l11:transition\\n' >>"$TEST_CALLS"; }}
l11_package_unchanged() {{ return 0; }}
l11_find_current_process() {{ return 1; }}
l11_start_process() {{ printf 'l11:start\\n' >>"$TEST_CALLS"; }}
global_ibus_unchanged() {{ return 0; }}
reload_lay_extension() {{ printf 'extension:reload\\n' >>"$TEST_CALLS"; }}
restore_captured_engine_if_safe() {{ return 0; }}
wait_runtime_version() {{ return 0; }}
verify_rollback_runtime() {{ return 0; }}
rollback_abort() {{ printf 'abort:%s\\n' "$1" >>"$TEST_CALLS"; exit "${{2:-99}}"; }}
capture_exact_v13_sidecar_admission || exit 201
{forward_transaction_dispatch()}
"""
            environment = os.environ.copy()
            environment.update(
                {
                    "TEST_ROOT": str(fixture["root"]),
                    "TEST_SNAPSHOT": str(snapshot),
                    "TEST_LIVE": str(live),
                    "TEST_L2": str(fixture["l2"]),
                    "TEST_CALLS": str(calls),
                    "TEST_INSTALLER_ENV": str(installer_environment),
                    "TEST_COPY_MARKER": str(copy_marker),
                }
            )
            result = subprocess.run(
                ["bash", "-c", harness],
                env=environment,
                check=False,
                capture_output=True,
                text=True,
                timeout=10,
            )
            self.assertEqual(1, result.returncode, result.stderr)
            self.assertEqual(
                f"copied:{fixture['sidecar'].stat().st_size}:644\n",
                copy_marker.read_text(encoding="utf-8"),
            )
            self.assertEqual(
                [
                    "1",
                    "1",
                    str(fixture["package"]),
                    "LAY-L2-RU-FULL-v13.bin",
                    str(fixture["package"].stat().st_size),
                    file_sha256(fixture["package"]),
                    str(fixture["l2"]),
                    str(fixture["root"] / "target/release"),
                    str(live / "bin"),
                    str(live / "links"),
                ],
                installer_environment.read_text(encoding="utf-8").splitlines(),
            )
            self.assertNotIn("extension:--fix --reload", calls.read_text(encoding="utf-8"))
            self.assertNotIn("l11:transition", calls.read_text(encoding="utf-8"))
            self.assertEqual(7, len(tree_projection(fixture["l2"])))
            self.assertEqual(before, tree_projection(fixture["l2"]))

    def test_isolated_rollback_restores_release_files_without_live_mutation(self) -> None:
        before = live_projection()

        result = subprocess.run(
            [str(CONTROLLER), "--self-test"],
            cwd=ROOT,
            check=False,
            capture_output=True,
            text=True,
            timeout=15,
        )

        self.assertEqual(0, result.returncode, result.stderr)
        self.assertIn("release live install controller self-test: PASS", result.stdout)
        self.assertEqual(before, live_projection())

    def test_snapshot_faults_abort_before_mutation(self) -> None:
        with tempfile.TemporaryDirectory(prefix="lay-controller-test-") as temporary:
            home = Path(temporary)
            snapshot = (
                home
                / ".local/state/lay/release-backups/1.0.66-preinstall-missing"
            )
            snapshot.parent.mkdir(parents=True)
            result = run_controller(snapshot, home)
            self.assertEqual(2, result.returncode)
            self.assertIn("snapshot directory is missing", result.stderr)

        with tempfile.TemporaryDirectory(prefix="lay-controller-test-") as temporary:
            home = Path(temporary)
            snapshot = (
                home
                / ".local/state/lay/release-backups/1.0.66-preinstall-incomplete"
            )
            (snapshot / "bin").mkdir(parents=True)
            result = run_controller(snapshot, home)
            self.assertEqual(2, result.returncode)
            self.assertIn("snapshot tree missing: extension", result.stderr)

        for fault in ("bytes", "mode"):
            with self.subTest(fault=fault), tempfile.TemporaryDirectory(
                prefix="lay-controller-test-"
            ) as temporary:
                home = Path(temporary)
                snapshot, live_trees = create_isolated_live_snapshot(home)
                before = {
                    name: tree_projection(tree) for name, tree in live_trees.items()
                }
                damaged = snapshot / "bin/file-00"
                if fault == "bytes":
                    damaged.write_text("different snapshot bytes\n", encoding="utf-8")
                else:
                    damaged.chmod(0o700)

                result = run_controller(snapshot, home)

                self.assertEqual(2, result.returncode)
                self.assertIn("binary snapshot does not match live baseline", result.stderr)
                self.assertEqual(
                    before,
                    {
                        name: tree_projection(tree)
                        for name, tree in live_trees.items()
                    },
                )

    def test_snapshot_systemd_parent_symlink_is_rejected(self) -> None:
        with tempfile.TemporaryDirectory(prefix="lay-controller-test-") as temporary:
            home = Path(temporary)
            snapshot = (
                home
                / ".local/state/lay/release-backups/1.0.66-preinstall-unit-alias"
            )
            for tree in ("bin", "extension", "l2"):
                (snapshot / tree).mkdir(parents=True)
            external = home / "external-systemd"
            external.mkdir()
            (external / "lay-l3-online.service").write_text(
                "aliased snapshot\n", encoding="utf-8"
            )
            (snapshot / "systemd").symlink_to(external, target_is_directory=True)
            live_unit = home / ".config/systemd/user/lay-l3-online.service"
            live_unit.parent.mkdir(parents=True)
            live_unit.write_text("aliased snapshot\n", encoding="utf-8")
            live_unit.chmod(0o644)

            result = run_controller(snapshot, home)

            self.assertEqual(2, result.returncode)
            self.assertIn(
                "snapshot systemd directory is missing or is a symlink",
                result.stderr,
            )

    def test_global_ibus_identity_is_dynamic(self) -> None:
        source = CONTROLLER.read_text(encoding="utf-8")

        self.assertIn("pgrep -x ibus-daemon", source)
        self.assertIn('"${#current_pids[@]}" == 1', source)
        self.assertIn('== "$ibus_before"', source)
        self.assertNotRegex(
            source,
            re.compile(r"(?:ibus_before|ibus_pid)\s*=\s*['\"]?[0-9]+"),
        )

    def test_selected_engine_identity_is_transactional_on_failure(self) -> None:
        source = CONTROLLER.read_text(encoding="utf-8")
        runtime = RUNTIME_CONTROLLER.read_text(encoding="utf-8")

        self.assertEqual(
            1,
            source.count(
                'engine_before="$(timeout 2s ibus engine 2>/dev/null || true)"'
            ),
        )
        self.assertIn('[[ "$1" == lay-ime-us || "$1" == lay-ime-ru ]]', source)
        self.assertIn('is_supported_lay_engine "$engine_before" || {', source)
        self.assertIn("select_xkb() {", runtime)
        self.assertIn("stop_ime() {", runtime)
        self.assertIn("select_xkb", runtime[runtime.index("stop_ime() {") :])

        recovery = source[
            source.index("restore_captured_engine_if_safe() {") : source.index(
                "rollback_abort() {"
            )
        ]
        self.assertLess(
            recovery.index("rollback_engine_mutation_is_safe"),
            recovery.index('ibus engine "$engine_before"'),
        )
        mutation_guard = source[
            source.index("rollback_engine_mutation_is_safe() {") : source.index(
                "current_ibus_engine() {"
            )
        ]
        self.assertIn("global_ibus_unchanged", mutation_guard)
        self.assertIn("rollback_snapshot_matches_live", mutation_guard)
        self.assertGreaterEqual(recovery.count("rollback_engine_mutation_is_safe"), 5)
        self.assertIn('current_engine" == "$engine_before"', recovery)
        self.assertIn("select_verified_xkb() {", source)

        rollback = forward_rollback_function()
        self.assertEqual(6, rollback.count("rollback_abort \"ROLLBACK_"))
        self.assertIn(
            'if ! restore_captured_engine_if_safe; then',
            rollback,
        )
        self.assertNotRegex(source, re.compile(r"\bibus\s+engine\s+lay-ime-(?:us|ru)\b"))
        self.assertIn("engine state UNVERIFIED", source)

    def test_engine_recovery_requires_verified_snapshot_parity(self) -> None:
        source = CONTROLLER.read_text(encoding="utf-8")
        recovery_functions = source[
            source.index("rollback_snapshot_matches_live() {") : source.index(
                "rollback_abort() {"
            )
        ]
        harness = f"""
set -u -o pipefail
verify_tree_parity() {{ diff -qr -- "$1" "$2" >/dev/null; }}
is_supported_lay_engine() {{ [[ "$1" == lay-ime-us || "$1" == lay-ime-ru ]]; }}
global_ibus_unchanged() {{ return 0; }}
gdbus() {{
    if [[ "${{TEST_DAMAGE_AFTER_GDBUS:-0}}" == 1 ]]; then
        printf 'mixed-during-engine-recovery\n' >"$TEST_LIVE/bin/artifact"
    fi
    return 1
}}
sleep() {{ :; }}
timeout() {{
    shift
    if [[ "$1" == ibus && "$2" == engine ]]; then
        if [[ "$#" == 3 ]]; then
            printf '%s' "$3" >"$TEST_ENGINE_STATE"
        else
            cat "$TEST_ENGINE_STATE"
        fi
        return 0
    fi
    return 1
}}
SNAPSHOT_DIR="$TEST_SNAPSHOT"
INSTALL_DIR="$TEST_LIVE/bin"
EXTENSION_DIR="$TEST_LIVE/extension"
L2_DIR="$TEST_LIVE/l2"
engine_before=lay-ime-ru
{recovery_functions}
restore_captured_engine_if_safe
"""

        with tempfile.TemporaryDirectory(prefix="lay-engine-recovery-") as temporary:
            root = Path(temporary)
            snapshot = root / "snapshot"
            live = root / "live"
            for tree in ("bin", "extension", "l2"):
                (snapshot / tree).mkdir(parents=True)
                (live / tree).mkdir(parents=True)
                (snapshot / tree / "artifact").write_text(
                    f"{tree}-rollback\n", encoding="utf-8"
                )
                shutil.copy2(snapshot / tree / "artifact", live / tree / "artifact")
            engine_state = root / "engine-state"
            engine_state.write_text("xkb:ru::rus", encoding="utf-8")
            environment = os.environ.copy()
            environment.update(
                {
                    "TEST_SNAPSHOT": str(snapshot),
                    "TEST_LIVE": str(live),
                    "TEST_ENGINE_STATE": str(engine_state),
                    "TEST_DAMAGE_AFTER_GDBUS": "0",
                }
            )

            recovered = subprocess.run(
                ["bash", "-c", harness],
                env=environment,
                check=False,
                capture_output=True,
                text=True,
                timeout=5,
            )
            self.assertEqual(0, recovered.returncode, recovered.stderr)
            self.assertEqual("lay-ime-ru", engine_state.read_text(encoding="utf-8"))

            (live / "bin/artifact").write_text("mixed-forward-bytes\n", encoding="utf-8")
            engine_state.write_text("xkb:ru::rus", encoding="utf-8")
            rejected = subprocess.run(
                ["bash", "-c", harness],
                env=environment,
                check=False,
                capture_output=True,
                text=True,
                timeout=5,
            )
            self.assertNotEqual(0, rejected.returncode)
            self.assertEqual("xkb:ru::rus", engine_state.read_text(encoding="utf-8"))

            shutil.copy2(snapshot / "bin/artifact", live / "bin/artifact")
            environment["TEST_DAMAGE_AFTER_GDBUS"] = "1"
            changed_mid_attempt = subprocess.run(
                ["bash", "-c", harness],
                env=environment,
                check=False,
                capture_output=True,
                text=True,
                timeout=5,
            )
            self.assertNotEqual(0, changed_mid_attempt.returncode)
            self.assertEqual("xkb:ru::rus", engine_state.read_text(encoding="utf-8"))

    def test_failed_xkb_selection_reports_engine_state_unverified(self) -> None:
        source = CONTROLLER.read_text(encoding="utf-8")
        recovery_functions = source[
            source.index("rollback_snapshot_matches_live() {") : source.index(
                "reload_lay_extension() {"
            )
        ]
        harness = f"""
set -u -o pipefail
verify_tree_parity() {{ return 1; }}
is_supported_lay_engine() {{ [[ "$1" == lay-ime-us || "$1" == lay-ime-ru ]]; }}
global_ibus_unchanged() {{ return 0; }}
gdbus() {{ printf 'gdbus:%s\n' "$*" >>"$TEST_CALLS"; return 1; }}
sleep() {{ :; }}
timeout() {{
    shift
    if [[ "$1" == ibus && "$2" == engine ]]; then
        if [[ "$#" == 3 ]]; then
            printf 'ibus-set:%s\n' "$3" >>"$TEST_CALLS"
            return 1
        fi
        cat "$TEST_ENGINE_STATE"
        return 0
    fi
    return 1
}}
SNAPSHOT_DIR="$TEST_ROOT/snapshot"
INSTALL_DIR="$TEST_ROOT/live/bin"
EXTENSION_DIR="$TEST_ROOT/live/extension"
L2_DIR="$TEST_ROOT/live/l2"
engine_before=lay-ime-ru
{recovery_functions}
rollback_abort "ROLLBACK_TEST=FAIL"
"""

        with tempfile.TemporaryDirectory(prefix="lay-xkb-failure-") as temporary:
            root = Path(temporary)
            engine_state = root / "engine-state"
            calls = root / "calls"
            engine_state.write_text("lay-ime-ru", encoding="utf-8")
            environment = os.environ.copy()
            environment.update(
                {
                    "TEST_ROOT": str(root),
                    "TEST_ENGINE_STATE": str(engine_state),
                    "TEST_CALLS": str(calls),
                }
            )

            result = subprocess.run(
                ["bash", "-c", harness],
                env=environment,
                check=False,
                capture_output=True,
                text=True,
                timeout=5,
            )

            self.assertEqual(2, result.returncode)
            self.assertIn("engine state UNVERIFIED", result.stderr)
            self.assertNotIn("engine verified fail-closed", result.stderr)
            self.assertNotIn("captured engine restored", result.stderr)
            self.assertEqual(
                ["ibus-set:xkb:ru::rus", "ibus-set:xkb:us::eng"],
                calls.read_text(encoding="utf-8").splitlines(),
            )

    def test_mid_retry_ibus_pid_change_stops_all_engine_mutation(self) -> None:
        source = CONTROLLER.read_text(encoding="utf-8")
        recovery_functions = source[
            source.index("rollback_snapshot_matches_live() {") : source.index(
                "reload_lay_extension() {"
            )
        ]
        harness = f"""
set -u -o pipefail
verify_tree_parity() {{ return 0; }}
is_supported_lay_engine() {{ [[ "$1" == lay-ime-us || "$1" == lay-ime-ru ]]; }}
ibus_checks=0
global_ibus_unchanged() {{
    ibus_checks=$((ibus_checks + 1))
    [[ "$ibus_checks" -le 2 ]]
}}
gdbus() {{ printf 'gdbus:%s\n' "$*" >>"$TEST_CALLS"; return 1; }}
sleep() {{ :; }}
timeout() {{
    shift
    if [[ "$1" == ibus && "$2" == engine && "$#" == 3 ]]; then
        printf 'ibus-set:%s\n' "$3" >>"$TEST_CALLS"
    fi
    return 1
}}
SNAPSHOT_DIR="$TEST_ROOT/snapshot"
INSTALL_DIR="$TEST_ROOT/live/bin"
EXTENSION_DIR="$TEST_ROOT/live/extension"
L2_DIR="$TEST_ROOT/live/l2"
engine_before=lay-ime-ru
{recovery_functions}
rollback_abort "ROLLBACK_TEST=FAIL"
"""

        with tempfile.TemporaryDirectory(prefix="lay-ibus-pid-change-") as temporary:
            root = Path(temporary)
            calls = root / "calls"
            environment = os.environ.copy()
            environment.update({"TEST_ROOT": str(root), "TEST_CALLS": str(calls)})

            result = subprocess.run(
                ["bash", "-c", harness],
                env=environment,
                check=False,
                capture_output=True,
                text=True,
                timeout=5,
            )

            self.assertEqual(2, result.returncode)
            self.assertIn("engine state UNVERIFIED", result.stderr)
            calls_made = calls.read_text(encoding="utf-8").splitlines()
            self.assertEqual(1, len(calls_made))
            self.assertTrue(calls_made[0].startswith("gdbus:"), calls_made)

    def test_failed_file_restore_uses_fail_closed_engine_recovery_before_reactivation(
        self,
    ) -> None:
        rollback = forward_rollback_function()

        stop_l11 = rollback.index("l11_stop_current_process")
        restore = rollback.index('rollback_files "$SNAPSHOT_DIR"')
        fail_closed = rollback.index(
            'if [[ "$rollback_prepare_rc" -ne 0 ]]', restore
        )
        start_l11 = rollback.index("l11_start_process", fail_closed)
        reload_extension = rollback.index("if ! reload_lay_extension")
        restart_l3 = rollback.index("if ! systemctl --user start lay-l3-online.service")
        restart_lay = rollback.index('if ! "$ROOT/scripts/lay-runtime-control.sh" start')

        self.assertLess(stop_l11, restore)
        self.assertLess(restore, fail_closed)
        self.assertLess(fail_closed, start_l11)
        self.assertLess(start_l11, reload_extension)
        self.assertLess(reload_extension, restart_l3)
        self.assertLess(restart_l3, restart_lay)
        self.assertIn(
            "FAIL after L1.1 stop; bytes or projection require inspection",
            rollback[fail_closed:reload_extension],
        )
        self.assertIn("rollback_abort", rollback[fail_closed:reload_extension])

    def test_forward_verification_failure_uses_the_same_rollback(self) -> None:
        source = CONTROLLER.read_text(encoding="utf-8")
        forward_start = source.index("(\n    set -euo pipefail")
        forward_end = source.index(")\nforward_rc=$?", forward_start)
        forward = source[forward_start:forward_end]
        failure_start = source.index(
            'echo "FORWARD_INSTALL_${EXPECTED_VERSION//./_}=FAIL', forward_end
        )
        failure_route = source[failure_start:]
        rollback = forward_rollback_function()

        self.assertIn("scripts/install-release-binaries.sh", forward)
        self.assertIn("scripts/check-gnome-extension-runtime.sh --fix --reload", forward)
        self.assertIn('wait_runtime_version "$EXPECTED_VERSION"', forward)
        self.assertIn("verify_forward", forward)
        self.assertIn('run_forward_rollback "$forward_rc"', failure_route)
        self.assertIn('exit "$L11_UNVERIFIED_RC"', failure_route)
        self.assertIn(
            'rollback_files "$SNAPSHOT_DIR" "$INSTALL_DIR" "$EXTENSION_DIR" "$L2_DIR"',
            rollback,
        )
        self.assertIn('wait_runtime_version "$ROLLBACK_VERSION"', rollback)
        self.assertIn("verify_rollback_runtime", rollback)

    def test_l3_unit_is_snapshot_installed_verified_and_rollback_restored(self) -> None:
        source = CONTROLLER.read_text(encoding="utf-8")
        forward = source[
            source.index("(\n    set -euo pipefail") : source.index(
                ")\nforward_rc=$?"
            )
        ]
        rollback = forward_rollback_function()
        verifier = source[
            source.index("verify_forward() {") : source.index(
                "verify_rollback_runtime() {"
            )
        ]
        resource_verifier = source[
            source.index("verify_l3_forward_resource_contract() {") : source.index(
                "verify_forward() {"
            )
        ]

        self.assertIn('L3_UNIT_SOURCE="$ROOT/systemd/lay-l3-online.service"', source)
        self.assertIn(
            'L3_UNIT_PATH="${HOME}/.config/systemd/user/lay-l3-online.service"',
            source,
        )
        install = forward.index(
            'install_file_atomic "$L3_UNIT_SOURCE" "$L3_UNIT_PATH" 0644'
        )
        reload_forward = forward.index("systemctl --user daemon-reload", install)
        restart = forward.index("systemctl --user restart lay-l3-online.service")
        self.assertLess(install, reload_forward)
        self.assertLess(reload_forward, restart)
        self.assertIn("verify_l3_forward_resource_contract", verifier)
        for required in (
            "CPUQuotaPerSecUSec",
            '[[ "$quota" == 1.500000s ]]',
            'LAY_L3_PROOF_WORKERS=2',
            '"/proc/$l3_pid/environ"',
            'verify_file_bytes_mode "$L3_UNIT_SOURCE" "$L3_UNIT_PATH" 644',
        ):
            self.assertIn(required, resource_verifier)

        restore = rollback.index(
            'rollback_files "$SNAPSHOT_DIR" "$INSTALL_DIR" "$EXTENSION_DIR" '
            '"$L2_DIR" "$L3_UNIT_PATH"'
        )
        reload_rollback = rollback.index("systemctl --user daemon-reload", restore)
        start = rollback.index("systemctl --user start lay-l3-online.service")
        self.assertLess(restore, reload_rollback)
        self.assertLess(reload_rollback, start)
        self.assertIn(
            'verify_file_bytes_mode "$SNAPSHOT_DIR/systemd/lay-l3-online.service" '
            '"$L3_UNIT_PATH" 644',
            rollback,
        )

    def test_release_runtime_parity_contract_is_complete(self) -> None:
        source = CONTROLLER.read_text(encoding="utf-8")
        verifier = source[
            source.index("verify_forward() {") : source.index(
                "verify_rollback_runtime() {"
            )
        ]
        specs_start = verifier.index("    local specs=(")
        specs_end = verifier.index("    )", specs_start)
        binary_pairs = set(
            re.findall(
                r'^\s*"([^"\n]+:[^"\n]+)"$',
                verifier[specs_start:specs_end],
                flags=re.MULTILINE,
            )
        )
        self.assertEqual(
            {
                "lay:lay",
                "lay-daemon:lay-daemon",
                "lay-nanda-wave-eval:lay-nanda-wave-eval",
                "lay-nanda-wave-train:lay-nanda-wave-train",
                "lay-test-input:lay-test-input",
                "lay-ngram-corpus:lay-ngram-corpus",
                "lay-ibus-engine:lay-ibus-engine",
                "lay-memory-report:lay-memory-report",
                "lay-l11-restore:lay-l1.1-restore",
                "lay-l11-serve:lay-l1.1-serve",
            },
            binary_pairs,
        )
        for required in (
            '[[ "$(file_count "$EXTENSION_SOURCE")" == 9 ]]',
            '[[ "$(file_count "$EXTENSION_DIR")" == 9 ]]',
            'diff -qr -- "$EXTENSION_SOURCE" "$EXTENSION_DIR"',
            '[[ "$source_hash" == "$installed_hash" ]]',
            '[[ "$installed_mode" == 755 ]]',
            '[[ "$resolved_link" == "$installed_file" ]]',
            'source "$ROOT/scripts/l2-package-contract.sh"',
            '[[ "$package_hash" == "$LAY_CANONICAL_L2_PACKAGE_SHA256" ]]',
            '== "$LAY_CANONICAL_L2_PACKAGE_BYTES"',
            "exact_v13_sidecar_sources_unchanged",
            "verify_installed_exact_v13_sidecar",
            '[[ "$version" == "$EXPECTED_VERSION" ]]',
            '[[ "$ping" == *"pong from lay-extension"* ]]',
            '[[ "$(timeout 2s ibus engine)" == "$engine_before" ]]',
            "global_ibus_unchanged",
            "systemctl --user is-active --quiet lay-daemon.service",
            "systemctl --user is-active --quiet lay-l3-online.service",
            "verify_process_parity",
        ):
            self.assertIn(required, verifier)

        process_verifier = source[
            source.index("verify_process_parity() {") : source.index(
                "verify_forward() {"
            )
        ]
        for required in (
            '"$daemon_pid:lay-daemon:daemon"',
            '"$l3_pid:lay-nanda-wave-train:l3"',
            '"$ime_pid:lay-ibus-engine:ime"',
            '[[ "${#ime_pids[@]}" == 1 ]]',
            'readlink -f "/proc/$pid/exe"',
            'sha256sum "/proc/$pid/exe"',
            'sha256sum "$INSTALL_DIR/$installed_name"',
        ):
            self.assertIn(required, process_verifier)

    def test_l11_lifecycle_is_release_local_and_in_all_process_parity(self) -> None:
        source = CONTROLLER.read_text(encoding="utf-8")

        lifecycle = l11_lifecycle_functions()
        process_verifier = source[
            source.index("verify_process_parity() {") : source.index(
                "verify_forward() {"
            )
        ]
        forward = source[
            source.index("(\n    set -euo pipefail") : source.index(
                ")\nforward_rc=$?"
            )
        ]
        rollback = source[source.index('echo "FORWARD_INSTALL_') :]

        self.assertIn("--repair-l11", source)
        self.assertIn("l11_capture_process", source)
        self.assertIn(
            'l11_transition_to_binary "$INSTALL_DIR/$L11_BINARY_NAME" "$L11_RELEASE_HASH"',
            forward,
        )
        self.assertIn(
            'l11_start_process',
            rollback,
        )
        self.assertLess(
            rollback.index("l11_stop_current_process"),
            rollback.index('rollback_files "$SNAPSHOT_DIR"'),
        )
        self.assertLess(
            rollback.index('rollback_files "$SNAPSHOT_DIR"'),
            rollback.index('"$L11_ROLLBACK_HASH"'),
        )
        self.assertIn('"$l11_pid:lay-l1.1-serve:l11"', process_verifier)
        self.assertIn("l11_health_ready", process_verifier)
        self.assertIn("l11_package_unchanged", process_verifier)
        self.assertIn('python3 "$L11_GUARD" terminate', lifecycle)
        self.assertIn('python3 "$L11_GUARD" health', lifecycle)
        self.assertIn("systemd-run --user", lifecycle)
        self.assertIn("l11_started_unit_owns_process", lifecycle)
        self.assertNotIn('kill -TERM "$pid"', lifecycle)
        self.assertNotIn('wait "$pid"', lifecycle)
        self.assertNotIn("pkill", lifecycle)
        self.assertNotRegex(
            lifecycle,
            re.compile(r"rm[^\n]+(?:L11_CAPTURED_SOCKET|lay-l11[.]sock)"),
        )
        self.assertEqual(RUNTIME_CONTROLLER_SHA256, file_sha256(RUNTIME_CONTROLLER))

    def test_l11_term_requires_exact_executable_hash_and_six_field_argv(self) -> None:
        lifecycle = l11_lifecycle_functions()

        with tempfile.TemporaryDirectory(prefix="lay-l11-identity-") as temporary:
            root = Path(temporary)
            proc_root = root / "proc"
            process_dir = proc_root / "101"
            install_dir = root / "live/bin"
            link_dir = root / "links"
            snapshot_dir = root / "snapshot"
            memory = root / "memory.bin"
            socket = root / "lay-l11.sock"
            calls = root / "calls"
            process_dir.mkdir(parents=True)
            install_dir.mkdir(parents=True)
            link_dir.mkdir(parents=True)
            (snapshot_dir / "bin").mkdir(parents=True)
            memory.write_bytes(b"admitted-package")
            binary = install_dir / "lay-l1.1-serve"
            binary.write_bytes(b"rollback-l11-binary")
            binary.chmod(0o755)
            invocation = link_dir / "lay-l1.1-serve"
            invocation.symlink_to(binary)
            (process_dir / "exe").symlink_to(binary)
            argv = (
                str(invocation),
                "run",
                "--memory",
                str(memory),
                "--socket",
                str(socket),
            )
            (process_dir / "cmdline").write_bytes(
                b"\0".join(value.encode() for value in argv) + b"\0"
            )
            binary_hash = hashlib.sha256(binary.read_bytes()).hexdigest()
            memory_hash = hashlib.sha256(memory.read_bytes()).hexdigest()
            environment = os.environ.copy()
            environment.update(
                {
                    "TEST_PROC": str(proc_root),
                    "TEST_INSTALL": str(install_dir),
                    "TEST_LINK": str(link_dir),
                    "TEST_SNAPSHOT": str(snapshot_dir),
                    "TEST_MEMORY": str(memory),
                    "TEST_SOCKET": str(socket),
                    "TEST_CALLS": str(calls),
                    "TEST_HASH": binary_hash,
                    "TEST_MEMORY_HASH": memory_hash,
                }
            )
            harness = f"""
set -u -o pipefail
L11_PROC_ROOT="$TEST_PROC"
INSTALL_DIR="$TEST_INSTALL"
LINK_DIR="$TEST_LINK"
SNAPSHOT_DIR="$TEST_SNAPSHOT"
L11_BINARY_NAME=lay-l1.1-serve
L11_GUARD=/test/lay-release-l11-guard.py
L11_UNVERIFIED_RC=70
L11_TERM_TIMEOUT_MS=5000
L11_CAPTURED_PID=101
L11_CAPTURED_ARGV=("$TEST_LINK/lay-l1.1-serve" run --memory "$TEST_MEMORY" --socket "$TEST_SOCKET")
L11_CAPTURED_MEMORY="$TEST_MEMORY"
L11_CAPTURED_SOCKET="$TEST_SOCKET"
L11_MEMORY_HASH="$TEST_MEMORY_HASH"
{lifecycle}
python3() {{
    [[ "$1" == "$L11_GUARD" && "$2" == terminate ]] || return 1
    l11_process_identity_matches "$4" "$6" "$8" || return 1
    printf 'TERM:%s\n' "$4" >>"$TEST_CALLS"
    rm -rf -- "$L11_PROC_ROOT/$4"
}}
kill() {{
    if [[ "$1" == -TERM ]]; then
        printf 'TERM:%s\n' "$2" >>"$TEST_CALLS"
        rm -rf -- "$L11_PROC_ROOT/$2"
        return 0
    fi
    if [[ "$1" == -0 ]]; then
        [[ -d "$L11_PROC_ROOT/$2" ]]
        return
    fi
    return 1
}}
sleep() {{ :; }}
[[ "$(l11_normalize_exe_path "$TEST_INSTALL/lay-l1.1-serve (deleted)")" == "$TEST_INSTALL/lay-l1.1-serve" ]] || exit 90
l11_stop_process 101 "$TEST_INSTALL/lay-l1.1-serve" "$TEST_HASH"
"""

            accepted = subprocess.run(
                ["bash", "-c", harness],
                env=environment,
                check=False,
                capture_output=True,
                text=True,
                timeout=5,
            )
            self.assertEqual(0, accepted.returncode, accepted.stderr)
            self.assertEqual(["TERM:101"], calls.read_text(encoding="utf-8").splitlines())

            process_dir.mkdir(parents=True)
            (process_dir / "exe").symlink_to(binary)
            damaged_argv = (*argv[:-1], str(root / "foreign.sock"))
            (process_dir / "cmdline").write_bytes(
                b"\0".join(value.encode() for value in damaged_argv) + b"\0"
            )
            calls.unlink()
            rejected = subprocess.run(
                ["bash", "-c", harness],
                env=environment,
                check=False,
                capture_output=True,
                text=True,
                timeout=5,
            )
            self.assertNotEqual(0, rejected.returncode)
            self.assertFalse(calls.exists(), rejected.stderr)

            duplicate_or_changed = subprocess.run(
                [
                    "bash",
                    "-c",
                    harness.replace(
                        'l11_stop_process 101 "$TEST_INSTALL/lay-l1.1-serve" "$TEST_HASH"',
                        'l11_find_current_process\n[[ "$?" == 2 ]]',
                    ),
                ],
                env=environment,
                check=False,
                capture_output=True,
                text=True,
                timeout=5,
            )
            self.assertEqual(0, duplicate_or_changed.returncode, duplicate_or_changed.stderr)
            self.assertFalse(calls.exists(), duplicate_or_changed.stderr)

    def test_l11_start_fault_only_stops_a_proven_new_process(self) -> None:
        lifecycle = l11_lifecycle_functions()
        harness = f"""
set -u -o pipefail
L11_CAPTURED_ARGV=(/installed/lay-l1.1-serve run --memory /data/memory.bin --socket /run/lay-l11.sock)
L11_CAPTURED_MEMORY=/data/memory.bin
L11_CAPTURED_SOCKET=/run/lay-l11.sock
L11_STARTED_PID=
L11_STARTED_UNIT=
L11_UNVERIFIED_RC=70
{lifecycle}
l11_package_unchanged() {{ return 0; }}
l11_binary_hash_matches() {{ return 0; }}
l11_spawn_process() {{
    [[ "$TEST_SPAWN_RC" == 0 ]] || return "$TEST_SPAWN_RC"
    L11_STARTED_PID=222
    L11_STARTED_UNIT=lay-l11-release-123-456.service
}}
l11_wait_process_ready() {{ [[ "$TEST_READY" == 1 ]]; }}
l11_started_unit_owns_process() {{ return 0; }}
l11_process_identity_matches() {{ return 0; }}
l11_stop_process() {{
    printf 'stop:%s:%s:%s\n' "$1" "$2" "$3" >>"$TEST_CALLS"
    return "$TEST_STOP_RC"
}}
l11_start_process /installed/lay-l1.1-serve /installed/lay-l1.1-serve wanted-hash
"""

        for spawn_rc, ready, stop_rc, expected_rc, expected_calls in (
            ("1", "0", "0", 1, []),
            ("70", "0", "0", 70, []),
            ("0", "1", "0", 0, []),
            ("0", "0", "0", 1, ["stop:222:/installed/lay-l1.1-serve:wanted-hash"]),
            ("0", "0", "70", 70, ["stop:222:/installed/lay-l1.1-serve:wanted-hash"]),
        ):
            with self.subTest(
                spawn_rc=spawn_rc, ready=ready, stop_rc=stop_rc
            ), tempfile.TemporaryDirectory(
                prefix="lay-l11-start-fault-"
            ) as temporary:
                calls = Path(temporary) / "calls"
                environment = os.environ.copy()
                environment.update(
                    {
                        "TEST_SPAWN_RC": spawn_rc,
                        "TEST_READY": ready,
                        "TEST_STOP_RC": stop_rc,
                        "TEST_CALLS": str(calls),
                    }
                )
                result = subprocess.run(
                    ["bash", "-c", harness],
                    env=environment,
                    check=False,
                    capture_output=True,
                    text=True,
                    timeout=5,
                )
                self.assertEqual(expected_rc, result.returncode, result.stderr)
                actual_calls = (
                    calls.read_text(encoding="utf-8").splitlines()
                    if calls.exists()
                    else []
                )
                self.assertEqual(expected_calls, actual_calls)

    def test_l11_cleanup_uncertainty_blocks_start_and_fallback(self) -> None:
        lifecycle = l11_lifecycle_functions()
        harness = f"""
set -u -o pipefail
L11_CAPTURED_ARGV=(/installed/lay-l1.1-serve run --memory /data/memory.bin --socket /run/lay-l11.sock)
L11_CAPTURED_MEMORY=/data/memory.bin
L11_CAPTURED_SOCKET=/run/lay-l11.sock
L11_CAPTURED_HASH=rollback-hash
L11_ROLLBACK_HASH=rollback-hash
L11_RELEASE_HASH=release-hash
L11_CURRENT_PID=
L11_CURRENT_EXE=
L11_CURRENT_HASH=
L11_UNVERIFIED_RC=70
{lifecycle}
global_ibus_unchanged() {{ return 0; }}
l11_package_unchanged() {{ return 0; }}
l11_binary_hash_matches() {{ return 0; }}
l11_find_current_process() {{
    L11_CURRENT_PID=111
    L11_CURRENT_EXE=/installed/lay-l1.1-serve
    L11_CURRENT_HASH=rollback-hash
}}
l11_health_ready() {{ return 1; }}
l11_stop_process() {{
    printf 'stop:%s\n' "$1" >>"$TEST_CALLS"
    return 70
}}
l11_start_process() {{ printf 'start:%s\n' "$1" >>"$TEST_CALLS"; }}
l11_transition_to_binary /installed/lay-l1.1-serve release-hash
"""
        with tempfile.TemporaryDirectory(prefix="lay-l11-unverified-") as temporary:
            calls = Path(temporary) / "calls"
            environment = os.environ.copy()
            environment["TEST_CALLS"] = str(calls)
            result = subprocess.run(
                ["bash", "-c", harness],
                env=environment,
                check=False,
                capture_output=True,
                text=True,
                timeout=5,
            )
            self.assertEqual(70, result.returncode, result.stderr)
            self.assertEqual(["stop:111"], calls.read_text(encoding="utf-8").splitlines())

            calls.unlink()
            start_cleanup_harness = f"""
set -u -o pipefail
L11_CAPTURED_ARGV=(/installed/lay-l1.1-serve run --memory /data/memory.bin --socket /run/lay-l11.sock)
L11_CAPTURED_MEMORY=/data/memory.bin
L11_CAPTURED_SOCKET=/run/lay-l11.sock
L11_CAPTURED_HASH=rollback-hash
L11_ROLLBACK_HASH=rollback-hash
L11_RELEASE_HASH=release-hash
L11_CURRENT_PID=
L11_CURRENT_EXE=
L11_CURRENT_HASH=
L11_UNVERIFIED_RC=70
{lifecycle}
global_ibus_unchanged() {{ return 0; }}
l11_package_unchanged() {{ return 0; }}
l11_binary_hash_matches() {{ return 0; }}
l11_find_current_process() {{ return 1; }}
l11_start_process() {{ printf 'start:%s\n' "$1" >>"$TEST_CALLS"; return 70; }}
l11_transition_to_binary /installed/lay-l1.1-serve release-hash
"""
            start_cleanup = subprocess.run(
                ["bash", "-c", start_cleanup_harness],
                env=environment,
                check=False,
                capture_output=True,
                text=True,
                timeout=5,
            )
            self.assertEqual(70, start_cleanup.returncode, start_cleanup.stderr)
            self.assertEqual(
                ["start:/installed/lay-l1.1-serve"],
                calls.read_text(encoding="utf-8").splitlines(),
            )

    def test_l11_package_disk_hash_detects_same_size_mutation(self) -> None:
        lifecycle = l11_lifecycle_functions()
        with tempfile.TemporaryDirectory(prefix="lay-l11-package-hash-") as temporary:
            package = Path(temporary) / "package.bin"
            package.write_bytes(b"package-A")
            expected_hash = hashlib.sha256(package.read_bytes()).hexdigest()
            environment = os.environ.copy()
            environment.update(
                {"TEST_PACKAGE": str(package), "TEST_PACKAGE_HASH": expected_hash}
            )
            harness = f"""
set -u -o pipefail
L11_CAPTURED_MEMORY="$TEST_PACKAGE"
L11_MEMORY_HASH="$TEST_PACKAGE_HASH"
{lifecycle}
l11_package_unchanged || exit 90
printf package-B >"$TEST_PACKAGE"
if l11_package_unchanged; then exit 91; fi
"""
            result = subprocess.run(
                ["bash", "-c", harness],
                env=environment,
                check=False,
                capture_output=True,
                text=True,
                timeout=5,
            )
            self.assertEqual(0, result.returncode, result.stderr)

    def test_installed_recovery_tree_fingerprints_detect_bytes_and_mode_drift(
        self,
    ) -> None:
        fingerprint = tree_fingerprint_function()
        recovery_trees = installed_recovery_tree_functions()

        with tempfile.TemporaryDirectory(
            prefix="lay-l11-tree-fingerprint-"
        ) as temporary:
            root = Path(temporary)
            environment = os.environ.copy()
            environment["TEST_ROOT"] = str(root)
            harness = f"""
set -u -o pipefail
INSTALL_DIR="$TEST_ROOT/installed/bin"
EXTENSION_DIR="$TEST_ROOT/installed/extension"
L2_DIR="$TEST_ROOT/installed/l2"
SNAPSHOT_DIR="$TEST_ROOT/snapshot"
L11_INSTALLED_BIN_TREE_FINGERPRINT=
L11_INSTALLED_EXTENSION_TREE_FINGERPRINT=
L11_INSTALLED_L2_TREE_FINGERPRINT=
L11_SNAPSHOT_BIN_TREE_FINGERPRINT=
L11_SNAPSHOT_EXTENSION_TREE_FINGERPRINT=
L11_SNAPSHOT_L2_TREE_FINGERPRINT=
{fingerprint}
{recovery_trees}
for tree in "$INSTALL_DIR" "$EXTENSION_DIR" "$L2_DIR" \
    "$SNAPSHOT_DIR/bin" "$SNAPSHOT_DIR/extension" "$SNAPSHOT_DIR/l2"; do
    mkdir -p "$tree/subdir"
    printf alpha >"$tree/subdir/artifact"
    chmod 0644 "$tree/subdir/artifact"
done
capture_installed_recovery_fingerprints || exit 90
printf omega >"$INSTALL_DIR/subdir/artifact"
if installed_recovery_trees_unchanged; then exit 91; fi
printf alpha >"$INSTALL_DIR/subdir/artifact"
capture_installed_recovery_fingerprints || exit 92
chmod 0600 "$SNAPSHOT_DIR/extension/subdir/artifact"
if installed_recovery_trees_unchanged; then exit 93; fi
"""
            result = subprocess.run(
                ["bash", "-c", harness],
                env=environment,
                check=False,
                capture_output=True,
                text=True,
                timeout=5,
            )
            self.assertEqual(0, result.returncode, result.stderr)

    def test_installed_recovery_has_distinct_success_fallback_and_unverified_routes(
        self,
    ) -> None:
        recovery = l11_installed_recovery_function()
        harness = f"""
set -u -o pipefail
EXPECTED_VERSION=1.0.66
INSTALL_DIR=/installed
SNAPSHOT_DIR=/snapshot
L11_BINARY_NAME=lay-l1.1-serve
L11_RELEASE_HASH=release-hash
L11_ROLLBACK_HASH=rollback-hash
L11_UNVERIFIED_RC=70
{recovery}
l11_transition_to_binary() {{
    if [[ "$1" == /installed/lay-l1.1-serve ]]; then
        printf 'transition:release\n' >>"$TEST_CALLS"
        return "$TEST_RELEASE_RC"
    fi
    printf 'transition:snapshot\n' >>"$TEST_CALLS"
    return "$TEST_SNAPSHOT_RC"
}}
verify_process_parity() {{ printf 'verify:release\n' >>"$TEST_CALLS"; return "$TEST_VERIFY_RC"; }}
TEST_PROJECTION_INDEX=0
installed_recovery_projection_unchanged() {{
    local -a results
    local result
    IFS=, read -r -a results <<<"$TEST_PROJECTION_RESULTS"
    result="${{results[$TEST_PROJECTION_INDEX]:-${{results[${{#results[@]}}-1]}}}}"
    TEST_PROJECTION_INDEX=$((TEST_PROJECTION_INDEX + 1))
    printf 'verify:projection\n' >>"$TEST_CALLS"
    return "$result"
}}
l11_run_installed_recovery
"""
        cases = (
            ("success", "0", "0", "0", "0,0", 0, ["transition:release", "verify:projection", "verify:release", "verify:projection"]),
            ("release-unverified", "70", "0", "0", "0", 70, ["transition:release"]),
            ("release-projection-unverified", "0", "0", "0", "1", 70, ["transition:release", "verify:projection"]),
            ("post-verifier-projection-unverified", "0", "0", "0", "0,1", 70, ["transition:release", "verify:projection", "verify:release", "verify:projection"]),
            ("release-absent-fallback", "1", "0", "0", "0,0", 2, ["transition:release", "verify:projection", "transition:snapshot", "verify:projection"]),
            ("pre-fallback-projection-unverified", "1", "0", "0", "1", 70, ["transition:release", "verify:projection"]),
            ("verify-fallback", "0", "1", "0", "0,0,0", 2, ["transition:release", "verify:projection", "verify:release", "verify:projection", "transition:snapshot", "verify:projection"]),
            ("pre-fallback-drift-after-verify", "0", "1", "0", "0,1", 70, ["transition:release", "verify:projection", "verify:release", "verify:projection"]),
            ("snapshot-unverified", "1", "0", "70", "0", 70, ["transition:release", "verify:projection", "transition:snapshot"]),
            ("post-snapshot-projection-unverified", "1", "0", "0", "0,1", 70, ["transition:release", "verify:projection", "transition:snapshot", "verify:projection"]),
        )
        for label, release_rc, verify_rc, snapshot_rc, projection_results, expected_rc, expected_calls in cases:
            with self.subTest(label=label), tempfile.TemporaryDirectory(
                prefix="lay-l11-installed-route-"
            ) as temporary:
                calls = Path(temporary) / "calls"
                environment = os.environ.copy()
                environment.update(
                    {
                        "TEST_CALLS": str(calls),
                        "TEST_RELEASE_RC": release_rc,
                        "TEST_VERIFY_RC": verify_rc,
                        "TEST_SNAPSHOT_RC": snapshot_rc,
                        "TEST_PROJECTION_RESULTS": projection_results,
                    }
                )
                result = subprocess.run(
                    ["bash", "-c", harness],
                    env=environment,
                    check=False,
                    capture_output=True,
                    text=True,
                    timeout=5,
                )
                self.assertEqual(expected_rc, result.returncode, result.stderr)
                self.assertEqual(
                    expected_calls, calls.read_text(encoding="utf-8").splitlines()
                )

    def test_forward_rollback_stops_l11_before_restoring_and_restarting(
        self,
    ) -> None:
        rollback = forward_rollback_function()

        with tempfile.TemporaryDirectory(
            prefix="lay-l11-forward-rollback-order-"
        ) as temporary:
            root = Path(temporary)
            runtime_controller = root / "scripts/lay-runtime-control.sh"
            runtime_controller.parent.mkdir(parents=True)
            runtime_controller.write_text(
                "#!/usr/bin/env bash\nprintf 'runtime:%s\\n' \"$1\" >>\"$TEST_CALLS\"\n",
                encoding="utf-8",
            )
            runtime_controller.chmod(0o755)
            calls = root / "calls"
            environment = os.environ.copy()
            environment.update(
                {"TEST_ROOT": str(root), "TEST_CALLS": str(calls)}
            )
            harness = f"""
set -u -o pipefail
EXPECTED_VERSION=1.0.66
ROLLBACK_VERSION=1.0.65
ROOT="$TEST_ROOT"
SNAPSHOT_DIR=/snapshot
INSTALL_DIR=/installed
EXTENSION_DIR=/extension
L2_DIR=/l2
L3_UNIT_PATH=/lay-l3-online.service
L11_BINARY_NAME=lay-l1.1-serve
L11_ROLLBACK_HASH=rollback-hash
L11_UNVERIFIED_RC=70
{rollback}
systemctl() {{ printf 'systemctl:%s\n' "$*" >>"$TEST_CALLS"; }}
select_verified_xkb() {{ printf 'select:xkb\n' >>"$TEST_CALLS"; }}
l11_stop_current_process() {{ printf 'stop:release\n' >>"$TEST_CALLS"; }}
rollback_files() {{ printf 'restore:trees\n' >>"$TEST_CALLS"; }}
verify_tree_parity() {{ return 0; }}
verify_file_bytes_mode() {{ return 0; }}
global_ibus_unchanged() {{ return 0; }}
l11_package_unchanged() {{ return 0; }}
l11_find_current_process() {{ return 1; }}
l11_start_process() {{ printf 'start:snapshot:%s\n' "$1" >>"$TEST_CALLS"; }}
reload_lay_extension() {{ printf 'reload:extension\n' >>"$TEST_CALLS"; }}
restore_captured_engine_if_safe() {{ return 0; }}
wait_runtime_version() {{ return 0; }}
verify_rollback_runtime() {{ return 0; }}
rollback_abort() {{ printf 'abort:%s\n' "$1" >>"$TEST_CALLS"; exit "${{2:-99}}"; }}
run_forward_rollback 17
"""
            result = subprocess.run(
                ["bash", "-c", harness],
                env=environment,
                check=False,
                capture_output=True,
                text=True,
                timeout=5,
            )
            self.assertEqual(17, result.returncode, result.stderr)
            actual_calls = calls.read_text(encoding="utf-8").splitlines()
            stop = actual_calls.index("stop:release")
            restore = actual_calls.index("restore:trees")
            start = actual_calls.index("start:snapshot:/installed/lay-l1.1-serve")
            self.assertLess(stop, restore)
            self.assertLess(restore, start)

    def test_l11_spawn_is_owned_by_a_detached_systemd_service(self) -> None:
        lifecycle = l11_lifecycle_functions()

        with tempfile.TemporaryDirectory(prefix="lay-l11-systemd-spawn-") as temporary:
            root = Path(temporary)
            proc_root = root / "proc"
            (proc_root / "222").mkdir(parents=True)
            calls = root / "calls"
            arguments = root / "arguments"
            environment = os.environ.copy()
            environment.update(
                {
                    "TEST_PROC": str(proc_root),
                    "TEST_CALLS": str(calls),
                    "TEST_ARGUMENTS": str(arguments),
                }
            )
            harness = f"""
set -u -o pipefail
L11_CAPTURED_ARGV=(/links/lay-l1.1-serve run --memory /data/package.bin --socket /run/lay-l11.sock)
L11_STARTED_PID=
L11_STARTED_UNIT=
L11_UNVERIFIED_RC=70
L11_PROC_ROOT="$TEST_PROC"
{lifecycle}
systemd-run() {{ printf '%s\n' "$@" >"$TEST_ARGUMENTS"; }}
systemctl() {{
    case "$2" in
        show) printf '222\n' ;;
        is-active) return 0 ;;
        *) return 1 ;;
    esac
}}
sleep() {{ :; }}
l11_spawn_process /installed/lay-l1.1-serve || exit $?
l11_started_unit_owns_process "$L11_STARTED_PID" || exit 91
printf 'pid:%s\nunit:%s\n' "$L11_STARTED_PID" "$L11_STARTED_UNIT" >>"$TEST_CALLS"
"""
            result = subprocess.run(
                ["bash", "-c", harness],
                env=environment,
                check=False,
                capture_output=True,
                text=True,
                timeout=5,
            )
            self.assertEqual(0, result.returncode, result.stderr)
            actual = calls.read_text(encoding="utf-8")
            actual_arguments = arguments.read_text(encoding="utf-8").splitlines()
            self.assertEqual(
                ["--user", "--quiet", "--collect", "--service-type=exec"],
                actual_arguments[:4],
            )
            self.assertEqual(
                [
                    "/bin/bash",
                    "-c",
                    'exec -a "$1" "$2" "$3" "$4" "$5" "$6" "$7"',
                    "_",
                    "/links/lay-l1.1-serve",
                    "/installed/lay-l1.1-serve",
                    "run",
                    "--memory",
                    "/data/package.bin",
                    "--socket",
                    "/run/lay-l11.sock",
                ],
                actual_arguments[-11:],
            )
            self.assertIn("pid:222", actual)
            self.assertRegex(
                actual, r"unit:lay-l11-release-[0-9]+-[0-9]+[.]service"
            )

        spawn = lifecycle[
            lifecycle.index("l11_spawn_process() {") : lifecycle.index(
                "l11_monotonic_millis() {"
            )
        ]
        self.assertIn("systemd-run --user", spawn)
        self.assertIn("--collect", spawn)
        self.assertNotIn("L11_STARTED_PID=$!", spawn)

    def test_l11_spawn_mainpid_uncertainty_cleans_only_the_created_unit(self) -> None:
        lifecycle = l11_lifecycle_functions()
        harness = f"""
set -u -o pipefail
L11_CAPTURED_ARGV=(/links/lay-l1.1-serve run --memory /data/package.bin --socket /run/lay-l11.sock)
L11_STARTED_PID=
L11_STARTED_UNIT=
L11_UNVERIFIED_RC=70
L11_PROC_ROOT=/proc
{lifecycle}
systemd-run() {{ return 0; }}
systemctl() {{
    case "$2" in
        show) return 1 ;;
        stop)
            printf 'stop:%s\n' "$3" >>"$TEST_CALLS"
            printf '%s' "$TEST_POST_STOP_STATE" >"$TEST_STATE"
            return 0
            ;;
        is-active)
            cat "$TEST_STATE"
            [[ "$(cat "$TEST_STATE")" == active ]]
            ;;
        *) return 1 ;;
    esac
}}
timeout() {{ shift; "$@"; }}
sleep() {{ :; }}
l11_spawn_process /installed/lay-l1.1-serve
printf 'rc:%s\n' "$?" >>"$TEST_CALLS"
"""

        for post_stop_state, expected_rc in (
            ("inactive", "1"),
            ("failed", "70"),
            ("active", "70"),
        ):
            with self.subTest(
                post_stop_state=post_stop_state
            ), tempfile.TemporaryDirectory(
                prefix="lay-l11-unbound-unit-"
            ) as temporary:
                root = Path(temporary)
                state = root / "state"
                state.write_text("active", encoding="ascii")
                calls = root / "calls"
                environment = os.environ.copy()
                environment.update(
                    {
                        "TEST_STATE": str(state),
                        "TEST_CALLS": str(calls),
                        "TEST_POST_STOP_STATE": post_stop_state,
                    }
                )
                result = subprocess.run(
                    ["bash", "-c", harness],
                    env=environment,
                    check=False,
                    capture_output=True,
                    text=True,
                    timeout=5,
                )
                self.assertEqual(0, result.returncode, result.stderr)
                actual_calls = calls.read_text(encoding="utf-8").splitlines()
                self.assertEqual(2, len(actual_calls))
                self.assertRegex(
                    actual_calls[0],
                    r"^stop:lay-l11-release-[0-9]+-[0-9]+[.]service$",
                )
                self.assertEqual(f"rc:{expected_rc}", actual_calls[1])

    @unittest.skipUnless(
        os.environ.get("LAY_L11_SYSTEMD_INTEGRATION") == "1",
        "opt-in transient user-systemd integration proof",
    )
    def test_l11_spawn_survives_controller_parent_exit(self) -> None:
        lifecycle = l11_lifecycle_functions()

        with tempfile.TemporaryDirectory(
            prefix="lay-l11-systemd-parent-exit-"
        ) as temporary:
            root = Path(temporary)
            state = root / "state"
            environment = os.environ.copy()
            environment["TEST_STATE"] = str(state)
            harness = f"""
set -u -o pipefail
L11_CAPTURED_ARGV=(/tmp/lay-l11-parent-survival -c "trap 'exit 0' TERM; while :; do sleep 1; done" marker-a marker-b marker-c)
L11_STARTED_PID=
L11_STARTED_UNIT=
L11_UNVERIFIED_RC=70
L11_PROC_ROOT=/proc
{lifecycle}
l11_spawn_process /bin/bash || exit $?
printf '%s\n%s\n' "$L11_STARTED_UNIT" "$L11_STARTED_PID" >"$TEST_STATE"
"""
            launch = subprocess.run(
                ["bash", "-c", harness],
                env=environment,
                check=False,
                capture_output=True,
                text=True,
                timeout=10,
            )
            self.assertEqual(0, launch.returncode, launch.stderr)
            unit, pid_text = state.read_text(encoding="utf-8").splitlines()
            self.assertRegex(unit, r"^lay-l11-release-[0-9]+-[0-9]+[.]service$")
            self.assertRegex(pid_text, r"^[1-9][0-9]*$")
            pid = int(pid_text)
            try:
                time.sleep(0.25)
                active = subprocess.run(
                    ["systemctl", "--user", "is-active", "--quiet", unit],
                    check=False,
                    timeout=5,
                )
                self.assertEqual(0, active.returncode)
                main_pid = subprocess.run(
                    [
                        "systemctl",
                        "--user",
                        "show",
                        unit,
                        "-p",
                        "MainPID",
                        "--value",
                    ],
                    check=True,
                    capture_output=True,
                    text=True,
                    timeout=5,
                ).stdout.strip()
                self.assertEqual(pid_text, main_pid)
                self.assertTrue(Path(f"/proc/{pid}").is_dir())
                raw_argv = Path(f"/proc/{pid}/cmdline").read_bytes()
                self.assertTrue(raw_argv.endswith(b"\0"))
                argv = tuple(raw_argv[:-1].split(b"\0"))
                self.assertEqual(
                    (
                        b"/tmp/lay-l11-parent-survival",
                        b"-c",
                        b"trap 'exit 0' TERM; while :; do sleep 1; done",
                        b"marker-a",
                        b"marker-b",
                        b"marker-c",
                    ),
                    argv,
                )
                proc_exe = Path(f"/proc/{pid}/exe")
                expected_exe = Path("/bin/bash").resolve(strict=True)
                self.assertEqual(expected_exe, proc_exe.resolve(strict=True))
                self.assertEqual(file_sha256(expected_exe), file_sha256(proc_exe))
            finally:
                subprocess.run(
                    ["systemctl", "--user", "stop", unit],
                    check=False,
                    stdout=subprocess.DEVNULL,
                    stderr=subprocess.DEVNULL,
                    timeout=10,
                )

    def test_l11_readiness_uses_one_bounded_deadline(self) -> None:
        lifecycle = l11_lifecycle_functions()

        with tempfile.TemporaryDirectory(
            prefix="lay-l11-ready-deadline-"
        ) as temporary:
            root = Path(temporary)
            proc_root = root / "proc"
            (proc_root / "123").mkdir(parents=True)
            clock = root / "clock"
            clock.write_text("1000\n1010\n1050\n", encoding="ascii")
            calls = root / "calls"
            environment = os.environ.copy()
            environment.update(
                {
                    "TEST_PROC": str(proc_root),
                    "TEST_CLOCK": str(clock),
                    "TEST_CALLS": str(calls),
                }
            )
            harness = f"""
set -u -o pipefail
L11_READY_TIMEOUT_MS=100
L11_HEALTH_TIMEOUT_MS=200
L11_CAPTURED_SOCKET=/run/test.sock
L11_CAPTURED_MEMORY=/data/package.bin
L11_PROC_ROOT="$TEST_PROC"
{lifecycle}
l11_monotonic_millis() {{
    local value
    value="$(head -n1 "$TEST_CLOCK")"
    tail -n +2 "$TEST_CLOCK" >"$TEST_CLOCK.next"
    mv "$TEST_CLOCK.next" "$TEST_CLOCK"
    printf '%s\n' "$value"
}}
l11_process_identity_matches() {{ return 0; }}
l11_health_ready() {{ printf 'health-timeout:%s\n' "$6" >>"$TEST_CALLS"; return 1; }}
sleep() {{ printf 'sleep\n' >>"$TEST_CALLS"; }}
if l11_wait_process_ready 123 /installed/lay-l1.1-serve wanted-hash; then
    exit 91
fi
"""
            result = subprocess.run(
                ["bash", "-c", harness],
                env=environment,
                check=False,
                capture_output=True,
                text=True,
                timeout=5,
            )
            self.assertEqual(0, result.returncode, result.stderr)
            self.assertEqual(
                ["health-timeout:90"],
                calls.read_text(encoding="utf-8").splitlines(),
            )

    def test_l11_failed_forward_transition_uses_snapshot_fallback(self) -> None:
        lifecycle = l11_lifecycle_functions()
        harness = f"""
set -u -o pipefail
L11_CAPTURED_ARGV=(/installed/lay-l1.1-serve run --memory /data/memory.bin --socket /run/lay-l11.sock)
L11_CAPTURED_MEMORY=/data/memory.bin
L11_CAPTURED_SOCKET=/run/lay-l11.sock
L11_CAPTURED_HASH=rollback-hash
L11_ROLLBACK_HASH=rollback-hash
L11_RELEASE_HASH=release-hash
L11_CURRENT_PID=
L11_CURRENT_EXE=
L11_CURRENT_HASH=
TEST_STATE=old
{lifecycle}
global_ibus_unchanged() {{ return 0; }}
l11_package_unchanged() {{ return 0; }}
l11_binary_hash_matches() {{ return 0; }}
l11_health_ready() {{ return 0; }}
l11_find_current_process() {{
    case "$TEST_STATE" in
        old)
            L11_CURRENT_PID=111
            L11_CURRENT_EXE=/installed/lay-l1.1-serve
            L11_CURRENT_HASH=rollback-hash
            return 0
            ;;
        unknown)
            L11_CURRENT_PID=333
            L11_CURRENT_EXE=/installed/lay-l1.1-serve
            L11_CURRENT_HASH=foreign-hash
            return 0
            ;;
        none) return 1 ;;
    esac
}}
l11_stop_process() {{
    printf 'stop:%s:%s:%s\n' "$1" "$2" "$3" >>"$TEST_CALLS"
    TEST_STATE=none
}}
l11_start_process() {{
    printf 'start:%s:%s\n' "$1" "$3" >>"$TEST_CALLS"
    if [[ "$1" == /installed/lay-l1.1-serve ]]; then
        return 1
    fi
    TEST_STATE=snapshot
    return 0
}}
if l11_transition_to_binary /installed/lay-l1.1-serve release-hash; then
    exit 91
fi
l11_transition_to_binary /snapshot/lay-l1.1-serve rollback-hash
"""
        with tempfile.TemporaryDirectory(prefix="lay-l11-fallback-") as temporary:
            calls = Path(temporary) / "calls"
            environment = os.environ.copy()
            environment["TEST_CALLS"] = str(calls)
            result = subprocess.run(
                ["bash", "-c", harness],
                env=environment,
                check=False,
                capture_output=True,
                text=True,
                timeout=5,
            )
            self.assertEqual(0, result.returncode, result.stderr)
            self.assertEqual(
                [
                    "stop:111:/installed/lay-l1.1-serve:rollback-hash",
                    "start:/installed/lay-l1.1-serve:release-hash",
                    "start:/snapshot/lay-l1.1-serve:rollback-hash",
                ],
                calls.read_text(encoding="utf-8").splitlines(),
            )

            calls.unlink()
            rejected = subprocess.run(
                ["bash", "-c", harness.replace("TEST_STATE=old", "TEST_STATE=unknown")],
                env=environment,
                check=False,
                capture_output=True,
                text=True,
                timeout=5,
            )
            self.assertNotEqual(0, rejected.returncode)
            self.assertFalse(calls.exists(), rejected.stderr)


if __name__ == "__main__":
    unittest.main()
