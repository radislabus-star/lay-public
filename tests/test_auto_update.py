"""Isolated update-mode install and actual frontend command adapters."""
import configparser
import importlib.util
import json
import os
from pathlib import Path
import shutil
import subprocess
import tempfile
import unittest
from unittest.mock import patch

ROOT = Path(__file__).resolve().parents[1]
UUID = "lay@radislabus-star.github.io"


class AutoUpdateResourceTests(unittest.TestCase):
    def test_build_can_use_its_existing_bounded_memory_budget(self):
        unit = configparser.ConfigParser(interpolation=None, strict=False)
        unit.read(ROOT / "systemd/lay-auto-update.service")
        service = unit["Service"]
        # A 3GiB soft ceiling made the actual compiler reclaim continuously
        # while its unchanged 4GiB hard budget still had room.
        self.assertIn(service.get("MemoryHigh", "infinity"),
                      ("infinity", service["MemoryMax"]))
        self.assertEqual(service["MemoryMax"], "4G")
        self.assertEqual(service["MemorySwapMax"], "256M")
        self.assertEqual(service["CPUQuota"], "100%")


class UpdateInstallTests(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory(prefix="lay-update-install-")
        self.addCleanup(self.tmp.cleanup)
        self.base = Path(self.tmp.name)
        self.home = self.base / "home"
        self.bin = self.home / ".local/bin"
        self.bin.mkdir(parents=True)
        self.source = self.base / "source"
        self.source.mkdir()
        self.env = dict(os.environ, HOME=str(self.home),
                        PATH=str(self.bin) + os.pathsep + os.environ["PATH"])
        shutil.copy2(ROOT / "install.sh", self.source / "install.sh")
        shutil.copy2(ROOT / "update.sh", self.source / "update.sh")
        self.shell(self.bin / "cargo", "exit 0")
        self.shell(self.bin / "rustc", "echo rust-test")
        self.shell(self.bin / "systemctl",
                   'test "$*" = "--user daemon-reload" || exit 93\necho reload >> "$HOME/calls"')
        self.shell(self.bin / "update-desktop-database", "exit 0")
        for forbidden in ("sudo", "ibus", "gnome-extensions"):
            self.shell(self.bin / forbidden, 'echo forbidden >> "$HOME/forbidden"; exit 94')
        scripts = {
            "resolve-l2-package.sh": 'echo resolve >> "$HOME/calls"; echo unused-package',
            "build-release-binaries.sh": 'echo build >> "$HOME/calls"; mkdir -p target/release; touch target/release/lay target/release/lay-daemon',
            "install-l2-transition-phase.sh": 'echo transition >> "$HOME/calls"',
            "install-l2-lexical-phase.sh": 'echo lexical >> "$HOME/calls"',
            "install-l3-context-phase.sh": 'echo context >> "$HOME/calls"',
            "install-release-binaries.sh": 'echo binaries >> "$HOME/calls"',
            "install-session-services.sh": 'test "$1" = --update; echo session-services >> "$HOME/calls"',
        }
        for name, body in scripts.items():
            self.shell(self.source / "scripts" / name, body)
        for name in ("lay-runtime-control.sh", "lay-kde-tray.py", "lay-host-vm-guard.sh"):
            self.shell(self.source / "scripts" / name, "exit 0")
        for name in ("lay-auto-update.service", "lay-auto-update.timer"):
            path = self.source / "systemd" / name
            path.parent.mkdir(exist_ok=True)
            shutil.copy2(ROOT / "systemd" / name, path)
        extension = self.source / "extension" / UUID
        extension.mkdir(parents=True)
        (extension / "metadata.json").write_text("{}")
        (extension / "extension.js").write_text("// entry")
        (extension / "tray_support.js").write_text("// helper")
        (self.source / "extension/lay-settings.desktop").write_text("[Desktop Entry]\n")
        self.personal = self.home / ".local/share/lay/personal.json"
        self.personal.parent.mkdir(parents=True)
        self.personal.write_text('{"manual_corrections": 3}')

    def shell(self, path, body):
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text("#!/bin/bash\n" + body + "\n")
        path.chmod(0o755)

    def install(self):
        return subprocess.run(["bash", str(self.source / "install.sh"), "--update"],
                              env=self.env, text=True, capture_output=True, timeout=15)

    def test_update_reuses_install_owners_without_initial_setup_or_learning_cleanup(self):
        result = self.install()
        self.assertEqual(0, result.returncode, result.stdout + result.stderr)
        self.assertEqual(["reload", "resolve", "build", "transition", "lexical", "context", "binaries", "session-services"],
                         (self.home / "calls").read_text().splitlines())
        self.assertFalse((self.home / "forbidden").exists())
        self.assertFalse((self.home / ".local/share/ibus").exists())
        self.assertEqual('{"manual_corrections": 3}', self.personal.read_text())
        owner = self.home / ".local/lib/lay/update.sh"
        self.assertTrue(owner.is_file())
        self.assertFalse(owner.is_symlink())
        self.assertEqual(owner, (self.bin / "lay-update").resolve())
        self.assertEqual((ROOT / "update.sh").read_bytes(), owner.read_bytes())
        self.assertTrue((self.home / ".config/systemd/user/lay-auto-update.timer").exists())

    def test_legacy_soft_budget_is_replaced_before_compilation(self):
        old = self.home / '.config/systemd/user/lay-auto-update.service'
        old.parent.mkdir(parents=True)
        old.write_text((self.source / 'systemd/lay-auto-update.service').read_text()
                       + '\nMemoryHigh=3G\n')
        policy = {'MemoryHigh': '3G', 'MemoryMax': '4G',
                  'MemorySwapMax': '256M', 'CPUQuota': '100%'}
        (self.home / 'live-budget.json').write_text(json.dumps(policy))
        self.shell(self.bin / 'systemctl', '''set -e
test "$*" = "--user daemon-reload" || exit 93
python3 - "$HOME/.config/systemd/user/lay-auto-update.service" "$HOME/live-budget.json" <<'UNIT_BUDGET_PY'
import configparser,json,sys
unit=configparser.ConfigParser(interpolation=None,strict=False)
unit.read(sys.argv[1]);service=unit['Service']
keys=('MemoryHigh','MemoryMax','MemorySwapMax','CPUQuota')
with open(sys.argv[2],'w') as stream:
    json.dump({key:service.get(key,'infinity') for key in keys},stream)
UNIT_BUDGET_PY
echo reload >> "$HOME/calls"
''')
        self.shell(self.source / 'scripts/build-release-binaries.sh', '''set -e
python3 - "$HOME/live-budget.json" <<'BUILD_BUDGET_PY'
import json,sys
with open(sys.argv[1]) as stream:budget=json.load(stream)
assert budget['MemoryHigh'] in ('infinity','4G'), 'legacy soft limit still active before compilation'
assert budget['MemoryMax']=='4G'
assert budget['MemorySwapMax']=='256M'
assert budget['CPUQuota']=='100%'
BUILD_BUDGET_PY
echo build >> "$HOME/calls"
mkdir -p target/release
touch target/release/lay target/release/lay-daemon
''')
        result = self.install()
        self.assertEqual(0, result.returncode, result.stdout + result.stderr)
        self.assertFalse((self.home / 'forbidden').exists())
        self.assertEqual('{"manual_corrections": 3}', self.personal.read_text())
        calls = (self.home / 'calls').read_text().splitlines()
        self.assertLess(calls.index('reload'), calls.index('build'))

    def test_existing_rustup_environment_precedes_system_cargo(self):
        self.shell(self.home / ".cargo/bin/cargo", "exit 0")
        self.shell(self.home / ".cargo/bin/rustc", "echo rustup-existing")
        (self.home / ".cargo/env").write_text(
            'export PATH="$HOME/.cargo/bin:$PATH"\n')
        self.shell(self.source / "scripts/build-release-binaries.sh", '''set -e
if [ "$(command -v cargo)" != "$HOME/.cargo/bin/cargo" ]; then
    echo 'existing rustup environment not loaded before compiler' >&2
    exit 92
fi
test "$(rustc --version)" = rustup-existing
echo build >> "$HOME/calls"
mkdir -p target/release
touch target/release/lay target/release/lay-daemon
''')
        result = self.install()
        self.assertEqual(0, result.returncode, result.stdout + result.stderr)
        self.assertFalse((self.home / "forbidden").exists())
        self.assertEqual('{"manual_corrections": 3}', self.personal.read_text())

    def test_build_failure_precedes_binary_and_updater_replacement(self):
        self.shell(self.source / "scripts/build-release-binaries.sh", "exit 44")
        result = self.install()
        self.assertEqual(44, result.returncode, result.stdout + result.stderr)
        self.assertFalse((self.home / ".local/lib/lay/update.sh").exists())
        self.assertNotIn("binaries", (self.home / "calls").read_text())
        self.assertFalse((self.home / "forbidden").exists())


class LoadedExtensionTests(unittest.TestCase):
    def check(self, mode, reload=False):
        with tempfile.TemporaryDirectory(prefix="lay-loaded-menu-") as directory:
            home = Path(directory)
            extension = ROOT / "extension" / UUID
            installed = home / ".local/share/gnome-shell/extensions" / UUID
            shutil.copytree(extension, installed)
            commands = home / "bin"
            commands.mkdir()
            version = json.loads((extension / "metadata.json").read_text())["version-name"]
            stubs = {
                "gnome-extensions": "exit 0",
                "sleep": "exit 0",
                "gdbus": (
                    'n=$(cat "$HOME/probes" 2>/dev/null || echo 0); n=$((n+1)); '
                    'echo "$n" > "$HOME/probes"\n'
                    'case "$TEST_ENDPOINT" in\n'
                    'missing) exit 1;;\n'
                    'delayed) test "$n" -ge 3 || exit 1;;\n'
                    'wrong) echo "(\x271.0.0\x27,)"; exit 0;;\n'
                    'esac\nprintf "(\x27%s\x27,)\\n" "$TEST_VERSION"'
                ),
            }
            for name, body in stubs.items():
                path = commands / name
                path.write_text("#!/bin/bash\n" + body + "\n")
                path.chmod(0o755)
            return subprocess.run(
                ["bash", str(ROOT / "scripts/check-gnome-extension-runtime.sh")]
                + (["--reload"] if reload else []),
                env=dict(os.environ, HOME=str(home), TEST_ENDPOINT=mode,
                         TEST_VERSION=version, PATH=str(commands) + os.pathsep + os.environ["PATH"]),
                text=True, capture_output=True, timeout=15)

    def test_missing_loaded_endpoint_is_failure_even_when_files_match(self):
        for reload in (False, True):
            with self.subTest(reload=reload):
                result = self.check("missing", reload=reload)
                self.assertNotEqual(0, result.returncode, result.stdout + result.stderr)
                self.assertIn("DBus endpoint did not respond", result.stderr)
                self.assertNotIn("runtime: OK", result.stdout)

    def test_async_reload_waits_for_loaded_version(self):
        result = self.check("delayed", reload=True)
        self.assertEqual(0, result.returncode, result.stdout + result.stderr)
        self.assertIn("runtime: OK", result.stdout)

    def test_stale_loaded_version_is_failure(self):
        result = self.check("wrong", reload=True)
        self.assertNotEqual(0, result.returncode, result.stdout + result.stderr)
        self.assertIn("loaded extension version drift", result.stderr)


class FrontendAdapters(unittest.TestCase):
    @unittest.skipUnless(shutil.which("gjs"), "requires GJS on the remote test host")
    def test_old_cached_loader_imports_missing_hint_and_each_generation_is_fresh(self):
        with tempfile.TemporaryDirectory(prefix="lay-old-loader-") as directory:
            home = Path(directory)
            installed = home / ".local/share/gnome-shell/extensions" / UUID / "ime_hint_style.js"
            installed.parent.mkdir(parents=True)
            installed.write_text("export const marker = 'one';\n")
            first, second = home / "first", home / "second"
            first.mkdir()
            second.mkdir()
            for name in ("lay-impl.js", "tray_support.js", "dbus_service.js", "recent_actions_menu.js"):
                (first / name).write_text("// original loader stages this module\n")
            (second / "lay-impl.js").touch()
            module = (ROOT / "extension" / UUID / "tray_support.js").as_uri()
            script = home / "check.js"
            script.write_text(
                "import Gio from 'gi://Gio';\n"
                f"import {{importImeHintStyle}} from {json.dumps(module)};\n"
                f"const first = {json.dumps((first / 'lay-impl.js').as_uri())};\n"
                f"const second = {json.dumps((second / 'lay-impl.js').as_uri())};\n"
                "const values = [(await importImeHintStyle(first)).marker];\n"
                f"Gio.File.new_for_path({json.dumps(str(installed))}).replace_contents(\n"
                "new TextEncoder().encode(\"export const marker = 'two';\\n\"),\n"
                "null, false, Gio.FileCreateFlags.REPLACE_DESTINATION, null);\n"
                "values.push((await importImeHintStyle(first)).marker);\n"
                "values.push((await importImeHintStyle(second)).marker);\n"
                "print(JSON.stringify(values));\n")
            result = subprocess.run(["gjs", "-m", str(script)],
                                    env=dict(os.environ, HOME=str(home)),
                                    text=True, capture_output=True, timeout=15)
            self.assertEqual(0, result.returncode, result.stdout + result.stderr)
            self.assertEqual(["one", "one", "two"], json.loads(result.stdout))
            self.assertEqual("export const marker = 'one';\n", (first / "ime_hint_style.js").read_text())

    def test_kde_adapter_executes_one_installed_owner_and_propagates_errors(self):
        spec = importlib.util.spec_from_file_location("lay_kde_update", ROOT / "scripts/lay-kde-tray.py")
        module = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(module)
        for action, output, error, code, expected in (
            ("--auto-status", "enabled\n", "", 0, True),
            ("--auto-disable", "disabled\n", "", 0, False),
            ("--auto-enable", "", "bus failed", 1, None),
            ("--auto-status", "unexpected", "", 0, None),
        ):
            with self.subTest(action=action, output=output):
                result = subprocess.CompletedProcess([], code, output, error)
                with patch.object(module.subprocess, "run", return_value=result) as run:
                    state, message = module.auto_update_control(action)
                    self.assertIs(state, expected)
                    self.assertEqual([str(Path.home() / ".local/bin/lay-update"), action],
                                     run.call_args.args[0])
                    self.assertEqual(5, run.call_args.kwargs["timeout"])
                    if expected is None:
                        self.assertTrue(message)

    @unittest.skipUnless(shutil.which("gjs"), "requires GJS on the remote test host")
    def test_gnome_async_adapter_executes_installed_owner_and_propagates_errors(self):
        with tempfile.TemporaryDirectory(prefix="lay-gnome-update-") as directory:
            home = Path(directory)
            updater = home / ".local/bin/lay-update"
            updater.parent.mkdir(parents=True)
            updater.write_text('#!/bin/bash\ncase "$1" in\n'
                               '--auto-status) echo enabled ;;\n'
                               '--auto-disable) echo disabled ;;\n'
                               '--auto-enable) echo "bus failed" >&2; exit 1 ;;\n'
                               '*) echo invalid ;;\nesac\n')
            updater.chmod(0o755)
            module = (ROOT / "extension" / UUID / "tray_support.js").as_uri()
            script = home / "check.js"
            script.write_text(
                "import GLib from 'gi://GLib';\n"
                f"import {{autoUpdateControl}} from {json.dumps(module)};\n"
                "const results = []; const loop = GLib.MainLoop.new(null, false);\n"
                "const actions = ['--auto-status', '--auto-disable', '--auto-enable', '--invalid'];\n"
                "function next() { if (!actions.length) { print(JSON.stringify(results)); loop.quit(); return; }\n"
                "autoUpdateControl(actions.shift(), state => {results.push(state); next();},\n"
                "error => {results.push(error); next();}); }\nnext(); loop.run();\n")
            result = subprocess.run(["gjs", "-m", str(script)],
                                    env=dict(os.environ, HOME=str(home)),
                                    text=True, capture_output=True, timeout=15)
            self.assertEqual(0, result.returncode, result.stdout + result.stderr)
            states = json.loads(result.stdout)
            self.assertEqual([True, False], states[:2])
            self.assertIn("bus failed", states[2])
            self.assertTrue(states[3])


if __name__ == "__main__":
    unittest.main()
