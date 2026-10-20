"""Exercise install/remove in a disposable user profile, without desktop input."""
import json
import importlib.util
import os
from pathlib import Path
import shlex
import subprocess
import tempfile
import unittest
from unittest.mock import patch

ROOT = Path(__file__).resolve().parents[1]
SCRIPT = ROOT / "scripts/lay-ime-client-setup.py"
DESKTOP = '[Desktop Entry]\nType=Application\nName=Kitty\nExec=kitty --title "my terminal"\nIcon=kitty\n'


class ClientSetupTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.home = Path(self.temp.name)
        self.data = self.home / "user data"
        self.config = self.home / "config"
        self.system = self.home / "system share"
        self.env = dict(os.environ, HOME=str(self.home),
                        XDG_CONFIG_HOME=str(self.config),
                        XDG_DATA_HOME=str(self.data), XDG_DATA_DIRS=str(self.system))
        self.env.pop("GLFW_IM_MODULE", None)
        for key in ("GTK_IM_MODULE", "QT_IM_MODULE", "XMODIFIERS"):
            self.env.pop(key, None)
        self.source = self.system / "applications/kitty.desktop"
        self.source.parent.mkdir(parents=True)
        self.source.write_text(DESKTOP)
        self.desktop = self.data / "applications/kitty.desktop"

    def run_setup(self, *args, ok=True):
        result = subprocess.run(["python3", str(SCRIPT), *args], env=self.env,
                                text=True, capture_output=True)
        self.assertEqual(result.returncode == 0, ok, result.stderr)
        return result

    def test_menu_launch_connects_without_test_environment_and_remove_roundtrip(self):
        self.run_setup()
        first = self.desktop.read_text()
        self.run_setup()
        self.assertEqual(self.desktop.read_text(), first)
        fake_bin = self.home / "bin"
        fake_bin.mkdir()
        kitty = fake_bin / "kitty"
        kitty.write_text('#!/usr/bin/python3\nimport json,os,sys\n'
                         'print(json.dumps([os.getenv("GLFW_IM_MODULE"),sys.argv[1:]]))\n')
        kitty.chmod(0o755)
        command = next(s[5:] for s in first.splitlines() if s.startswith("Exec="))
        env = dict(self.env, PATH=str(fake_bin) + os.pathsep + self.env["PATH"])
        result = subprocess.run(shlex.split(command), env=env, text=True,
                                capture_output=True, check=True)
        self.assertEqual(json.loads(result.stdout), ["ibus", ["--title", "my terminal"]])
        self.run_setup("--remove")
        self.assertFalse(self.desktop.exists())
        self.assertEqual(self.source.read_text(), DESKTOP)
        self.assertFalse((self.config / "environment.d/90-lay-glfw-ime.conf").exists())

    def test_custom_launcher_and_later_user_edit_survive(self):
        self.desktop.parent.mkdir(parents=True)
        original = DESKTOP + '\n[Desktop Action New]\nName=New\nExec=kitty -e bash\n'
        self.desktop.write_text(original)
        self.run_setup()
        self.desktop.write_text(self.desktop.read_text().replace("Name=Kitty", "Name=My Kitty"))
        self.run_setup("--remove")
        self.assertEqual(self.desktop.read_text(), original.replace("Name=Kitty", "Name=My Kitty"))

    def test_explicit_other_input_method_is_preserved_before_any_write(self):
        self.desktop.parent.mkdir(parents=True)
        original = DESKTOP.replace("Exec=kitty", "Exec=env GLFW_IM_MODULE=fcitx kitty")
        self.desktop.write_text(original)
        self.run_setup(ok=False)
        self.assertEqual(self.desktop.read_text(), original)
        self.assertFalse(self.config.exists())

    def test_session_scripts_preserve_explicit_input_method(self):
        self.run_setup()
        script = self.config / "plasma-workspace/env/lay-glfw-ime.sh"
        for value, expected in [(None, "ibus"), ("fcitx", "fcitx")]:
            env = dict(self.env)
            if value:
                env["GLFW_IM_MODULE"] = value
            result = subprocess.run(["sh", "-c", '. "$1"; printf %s "$GLFW_IM_MODULE"',
                                     "sh", str(script)], env=env, text=True,
                                    capture_output=True, check=True)
            self.assertEqual(result.stdout, expected)

    def test_kde_gtk_and_qt_defaults_apply_at_login(self):
        self.run_setup()
        script = self.config / "plasma-workspace/env/lay-glfw-ime.sh"
        env = {k: v for k, v in self.env.items()
               if k not in ("GTK_IM_MODULE", "QT_IM_MODULE", "XMODIFIERS")}
        result = subprocess.run(
            ["sh", "-c", '. "$1"; printf "%s|%s|%s" "$GTK_IM_MODULE" '
             '"$QT_IM_MODULE" "$XMODIFIERS"', "sh", str(script)],
            env=env, text=True, capture_output=True, check=True)
        self.assertEqual(result.stdout, "ibus|ibus|@im=ibus")

    def test_other_toolkit_input_method_is_preserved_without_partial_setup(self):
        self.env["QT_IM_MODULE"] = "fcitx"
        self.run_setup(ok=False)
        self.assertFalse(self.desktop.exists())
        self.assertFalse(self.config.exists())

    def test_generic_session_setup_also_works_without_kitty_installed(self):
        self.source.unlink()
        self.run_setup()
        self.assertFalse(self.desktop.exists())
        self.assertIn("QT_IM_MODULE=${QT_IM_MODULE:-ibus}",
                      (self.config / "environment.d/90-lay-glfw-ime.conf").read_text())
        self.run_setup("--remove")
        self.assertFalse((self.config / "environment.d/90-lay-glfw-ime.conf").exists())

    def test_previous_owned_glfw_only_setup_migrates_and_custom_content_survives(self):
        path = self.config / "environment.d/90-lay-glfw-ime.conf"
        path.parent.mkdir(parents=True)
        path.write_text("# Managed by Lay: GLFW IBus client\nGLFW_IM_MODULE=${GLFW_IM_MODULE:-ibus}\n")
        self.run_setup()
        self.assertIn("QT_IM_MODULE", path.read_text())
        custom = path.read_text() + "# my change\n"
        path.write_text(custom)
        self.run_setup(ok=False)
        self.run_setup("--remove")
        self.assertEqual(path.read_text(), custom)

    def test_read_only_diagnostic_reports_other_ime_and_never_claims_client_pass(self):
        binary = self.home / "bin"
        binary.mkdir()
        ibus = binary / "ibus"
        ibus.write_text('#!/bin/sh\nprintf "%s\\n" lay-ime-ru\n')
        ibus.chmod(0o755)
        self.env["PATH"] = str(binary) + os.pathsep + self.env["PATH"]
        self.env["QT_IM_MODULE"] = "fcitx"
        report = json.loads(self.run_setup("--json").stdout)
        self.assertEqual(report["engine"], "lay-ime-ru")
        self.assertEqual(report["conflicts"], ["QT_IM_MODULE"])
        self.assertEqual(report["client_acceptance"], "NOT_TESTED")
        self.assertFalse(self.config.exists())
        self.assertFalse(self.desktop.exists())

    def test_automatic_setup_does_not_mistake_native_session_defaults_for_failure(self):
        spec = importlib.util.spec_from_file_location("lay_ime_client_setup", SCRIPT)
        module = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(module)
        report = dict(engine="lay-ime-ru", conflicts=[], modules=dict.fromkeys(module.MODULES))
        for desktop, session, expected in [("GNOME", "x11", False),
                                           ("GNOME", "wayland", False),
                                           ("KDE", "wayland", False),
                                           ("KDE", "x11", True)]:
            with patch.dict(os.environ, {"XDG_CURRENT_DESKTOP": desktop,
                                         "XDG_SESSION_TYPE": session}):
                self.assertEqual(module.needs_setup(report), expected, (desktop, session))
        with patch.dict(os.environ, {"XDG_CURRENT_DESKTOP": "GNOME",
                                     "XDG_SESSION_TYPE": "wayland"}):
            self.assertTrue(module.needs_setup(dict(report, engine="")))
            self.assertTrue(module.needs_setup(dict(report, engine="other")))
            self.assertTrue(module.needs_setup(dict(report, conflicts=["QT_IM_MODULE"])))


if __name__ == "__main__":
    unittest.main()
