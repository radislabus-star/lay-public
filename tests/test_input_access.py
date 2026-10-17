#!/usr/bin/env python3
"""Run actual installer access provisioning against fake devices/commands."""
import json
import os
from pathlib import Path
import subprocess
import tempfile
import unittest

ROOT = Path(__file__).resolve().parents[1]
HELPER = ROOT / "scripts/setup-input-access.sh"
LEGACY = ROOT / "udev/99-lay-uinput.legacy.rules"


class InputAccessTests(unittest.TestCase):
    def setUp(self):
        temporary = tempfile.TemporaryDirectory(prefix="lay-input-access-")
        self.addCleanup(temporary.cleanup)
        self.work = Path(temporary.name)
        self.system = self.work / "system"
        self.system.mkdir()
        self.fake = self.work / "bin"
        self.fake.mkdir()
        self.record = self.work / "commands.jsonl"
        self.env = dict(os.environ, PATH=str(self.fake) + os.pathsep + os.environ["PATH"],
                        HOME=str(self.work / "home"),
                        LAY_INPUT_ACCESS_SYSTEM_ROOT=str(self.system),
                        LAY_TEST_ACCESS_RECORD=str(self.record), LAY_TEST_ACCESS_FAIL="")
        self.env.pop("BASH_ENV", None)
        self.env.pop("ENV", None)
        self.env["DBUS_SESSION_BUS_ADDRESS"] = "unix:path=" + str(self.work / "absent-bus")
        self.env["XDG_RUNTIME_DIR"] = str(self.work / "absent-runtime")
        # Only install/rm are real, constrained to the temporary root. No host
        # privileged/device command can escape through this sudo replacement.
        sudo = self.fake / "sudo"
        sudo.write_text("""#!/usr/bin/env python3
import json, os, pathlib, subprocess, sys
args=sys.argv[1:]
with open(os.environ['LAY_TEST_ACCESS_RECORD'], 'a') as out:
    out.write(json.dumps(args)+'\\n')
command=args[0]
if ' '.join(args)==os.environ.get('LAY_TEST_ACCESS_FAIL'):
    sys.exit(17)
if command in ('install','rm'):
    root=pathlib.Path(os.environ['LAY_INPUT_ACCESS_SYSTEM_ROOT']).resolve()
    target=pathlib.Path(args[-1]).resolve()
    if not target.is_relative_to(root):
        sys.exit('refusing non-sandbox path')
    sys.exit(subprocess.call(['/usr/bin/'+command,*args[1:]]))
if command not in ('modprobe','udevadm'):
    sys.exit('forbidden privileged command: '+command)
""")
        sudo.chmod(0o755)
        for name in ("cargo", "rustc"):
            path = self.fake / name
            path.write_text("#!/bin/sh\necho installer-test-tool\n")
            path.chmod(0o755)
        # If install.sh ever ignores helper failure, later session/network
        # commands must fail safely, including legacy Ollama service cleanup.
        for name in ("systemctl", "ibus", "gsettings", "gnome-extensions",
                     "dconf", "curl", "wget", "git", "pip", "pip3"):
            path = self.fake / name
            path.write_text("#!/bin/sh\necho forbidden-later-stage >&2\nexit 97\n")
            path.chmod(0o755)

    def run_helper(self):
        return subprocess.run(["bash", str(HELPER)], env=self.env, text=True,
                              capture_output=True, timeout=10)

    def commands(self):
        return [json.loads(line) for line in self.record.read_text().splitlines()]

    def legacy(self, contents=None):
        path = self.system / "etc/udev/rules.d/99-lay-uinput.rules"
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_bytes(LEGACY.read_bytes() if contents is None else contents)
        return path

    def test_fresh_setup_installs_early_session_rules_and_loads_module(self):
        result = self.run_helper()
        self.assertEqual(result.returncode, 0, result.stderr)
        rule = self.system / "etc/udev/rules.d/70-lay-input.rules"
        text = rule.read_text()
        for line in ['SUBSYSTEM=="input", KERNEL=="event*", TAG+="uaccess"',
                     'SUBSYSTEM=="misc", KERNEL=="uinput", TAG+="uaccess"']:
            self.assertIn(line, text)
        self.assertLess(rule.name, "73-seat-late.rules")
        self.assertNotIn('GROUP=', text)
        self.assertNotIn('MODE=', text)
        self.assertEqual(rule.stat().st_mode & 0o777, 0o644)
        self.assertEqual((self.system / "etc/modules-load.d/lay-uinput.conf").read_text(),
                         "uinput\n")
        self.assertIn(["modprobe", "uinput"], self.commands())
        self.assertIn("logind", result.stderr)  # Missing fake devices != ACL PASS.

    def test_both_subsystems_are_applied_in_order_with_bounded_settle(self):
        self.assertEqual(self.run_helper().returncode, 0)
        commands = [row for row in self.commands() if row[0] == "udevadm"]
        self.assertEqual(commands, [
            ["udevadm", "control", "--reload-rules"],
            ["udevadm", "trigger", "--action=change", "--subsystem-match=input", "--sysname-match=event*"],
            ["udevadm", "trigger", "--action=change", "--subsystem-match=misc", "--sysname-match=uinput"],
            ["udevadm", "settle", "--timeout=10"],
        ])

    def test_accessible_devices_do_not_skip_durable_setup_and_repeat_is_safe(self):
        (self.system / "dev/input").mkdir(parents=True)
        (self.system / "dev/input/event0").touch()
        (self.system / "dev/uinput").touch()
        for _ in range(2):
            result = self.run_helper()
            self.assertEqual(result.returncode, 0, result.stderr)
            self.assertIn("доступны текущему процессу", result.stdout)
            self.assertTrue((self.system / "etc/udev/rules.d/70-lay-input.rules").is_file())
        self.assertEqual(sum(row[0] == "install" for row in self.commands()), 4)
        self.assertFalse(any(row[0] in {"usermod", "gpasswd", "chmod", "chgrp"}
                             for row in self.commands()))

    def test_only_the_exact_legacy_rule_is_removed(self):
        path = self.legacy()
        self.assertEqual(self.run_helper().returncode, 0)
        self.assertFalse(path.exists())
        custom = LEGACY.read_bytes() + b"# administrator rule\n"
        path = self.legacy(custom)
        result = self.run_helper()
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(path.read_bytes(), custom)
        self.assertIn("сохраняю", result.stderr)

    def test_modified_new_files_stop_setup_without_overwrite_or_partial_install(self):
        for relative in ["etc/udev/rules.d/70-lay-input.rules",
                         "etc/modules-load.d/lay-uinput.conf"]:
            with self.subTest(relative=relative):
                path = self.system / relative
                path.parent.mkdir(parents=True, exist_ok=True)
                path.write_text("# administrator content\n")
                result = self.run_helper()
                self.assertEqual(result.returncode, 1, result.stdout + result.stderr)
                self.assertEqual(path.read_text(), "# administrator content\n")
                self.assertFalse(self.record.exists())
                path.unlink()

    def test_activation_failures_are_nonzero_and_do_not_claim_access(self):
        for command in ["modprobe uinput", "udevadm control --reload-rules",
                        "udevadm trigger --action=change --subsystem-match=input --sysname-match=event*",
                        "udevadm trigger --action=change --subsystem-match=misc --sysname-match=uinput",
                        "udevadm settle --timeout=10"]:
            with self.subTest(command=command):
                self.env["LAY_TEST_ACCESS_FAIL"] = command
                result = self.run_helper()
                self.assertEqual(result.returncode, 17, result.stderr)
                self.assertNotIn("доступны текущему процессу", result.stdout)
                self.assertTrue((self.system / "etc/udev/rules.d/70-lay-input.rules").exists())

    def test_failed_module_load_still_applies_evdev_and_reports_incomplete_activation(self):
        self.env["LAY_TEST_ACCESS_FAIL"] = "modprobe uinput"
        result = self.run_helper()
        self.assertEqual(result.returncode, 17)
        self.assertIn(["udevadm", "settle", "--timeout=10"], self.commands())
        self.assertIn("активация не подтверждена", result.stderr)

    def test_real_installer_stops_before_cleanup_or_build_on_helper_failure(self):
        self.env["LAY_TEST_ACCESS_FAIL"] = "udevadm control --reload-rules"
        result = subprocess.run(["bash", str(ROOT / "install.sh")], env=self.env,
                                text=True, capture_output=True, timeout=10)
        self.assertEqual(result.returncode, 17, result.stdout + result.stderr)
        self.assertNotIn("legacy Ollama cleanup", result.stdout)
        self.assertIn(["udevadm", "control", "--reload-rules"], self.commands())

    def test_unsafe_filesystem_override_is_rejected_before_any_command(self):
        for root in ["/", "relative", str(self.work / "missing")]:
            with self.subTest(root=root):
                self.env["LAY_INPUT_ACCESS_SYSTEM_ROOT"] = root
                self.assertEqual(self.run_helper().returncode, 2)
                self.assertFalse(self.record.exists())

    def test_modified_new_files_survive_uninstall_and_purge(self):
        for purge in [False, True]:
            with self.subTest(purge=purge):
                paths = [self.system / "etc/udev/rules.d/70-lay-input.rules",
                         self.system / "etc/modules-load.d/lay-uinput.conf"]
                for path in paths:
                    path.parent.mkdir(parents=True, exist_ok=True)
                    path.write_text("# administrator content\n")
                env = dict(self.env, LAY_UNINSTALL_TEST_MODE="1",
                           LAY_UNINSTALL_SYSTEM_ROOT=str(self.system))
                args = ["bash", str(ROOT / "uninstall.sh"), "--keep-source"]
                if purge:
                    args.append("--purge")
                result = subprocess.run(args, env=env, text=True,
                                        capture_output=True, timeout=10)
                self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
                for path in paths:
                    self.assertEqual(path.read_text(), "# administrator content\n")

    def test_uninstall_and_purge_remove_owned_files_and_preserve_custom_rules(self):
        for purge in [False, True]:
            for custom in [False, True]:
                with self.subTest(purge=purge, custom=custom):
                    self.assertEqual(self.run_helper().returncode, 0)
                    path = self.legacy(b"# custom\n" if custom else None)
                    unrelated = path.with_name("80-another-app.rules")
                    unrelated.write_text("# keep\n")
                    env = dict(self.env, LAY_UNINSTALL_TEST_MODE="1",
                               LAY_UNINSTALL_SYSTEM_ROOT=str(self.system))
                    args = ["bash", str(ROOT / "uninstall.sh"), "--keep-source"]
                    if purge:
                        args.append("--purge")
                    result = subprocess.run(args, env=env, text=True,
                                            capture_output=True, timeout=10)
                    self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
                    self.assertFalse((path.parent / "70-lay-input.rules").exists())
                    self.assertFalse((self.system / "etc/modules-load.d/lay-uinput.conf").exists())
                    self.assertEqual(path.exists(), custom)
                    if custom:
                        self.assertEqual(path.read_text(), "# custom\n")
                    self.assertEqual(unrelated.read_text(), "# keep\n")


if __name__ == "__main__":
    unittest.main()
