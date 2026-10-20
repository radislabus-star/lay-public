#!/usr/bin/env python3
"""Exercise the real shared GTK settings with an isolated config and commands."""
import json
import os
from pathlib import Path
import shutil
import subprocess
import tempfile
import unittest

ROOT = Path(__file__).resolve().parents[1]


@unittest.skipUnless(shutil.which('gjs') and os.environ.get('DISPLAY'),
                     'requires GJS, libadwaita and an X display')
class SettingsUiTests(unittest.TestCase):
    def test_real_widgets_save_to_runtime_config_and_keep_default_config(self):
        with tempfile.TemporaryDirectory(prefix='lay-settings-ui-') as directory:
            home = Path(directory)
            default = home / '.config/lay/config.json'
            default.parent.mkdir(parents=True)
            default.write_text('{"preserve": "default"}\n')
            selected = home / 'session/config.json'
            selected.parent.mkdir()
            selected.write_text('{"preserve": "session", "tap_max_ms": 200}\n')
            fake = home / '.local/bin'
            fake.mkdir(parents=True)
            for name in ('systemctl', 'ibus', 'lay-runtime-control'):
                path = fake / name
                path.write_text('#!/bin/sh\nexit 0\n')
                path.chmod(0o755)
            script = home / 'check.mjs'
            script.write_text('''
import Adw from 'gi://Adw';
import Gtk from 'gi://Gtk';
import {LaySettingsView} from MODULE;
Adw.init();
const view = new LaySettingsView();
function findRow(widget, title) {
    if (widget instanceof Adw.ActionRow && widget.title === title)
        return widget;
    for (let child = widget.get_first_child(); child; child = child.get_next_sibling()) {
        const found = findRow(child, title);
        if (found) return found;
    }
    return null;
}
function row(title) {
    const found = findRow(view.page, title);
    if (!found) throw new Error('Missing setting: ' + title);
    return found;
}
const learn = row('Запоминать ручные правки').activatable_widget;
if (!(learn instanceof Gtk.Switch)) throw new Error('No learning switch');
learn.active = true;
row('Подробный журнал действий').activatable_widget.active = true;
const ru = row('Русский');
const en = row('Английский');
if (ru.sensitive || en.sensitive) throw new Error('Force keys initially enabled');
const force = row('Отдельные клавиши RU / EN').activatable_widget;
force.active = true;
if (!ru.sensitive || !en.sensitive) throw new Error('Force keys not enabled');
force.active = false;
if (ru.sensitive || en.sensitive) throw new Error('Force keys not disabled');
const timing = row('Длительность одиночной клавиши').activatable_widget;
if (!(timing instanceof Gtk.SpinButton)) throw new Error('No timing control');
timing.value = 275;
if (view.cfg.tap_max_ms !== 275) throw new Error('Timing not saved');
print(JSON.stringify({adwaita: Adw.get_major_version() + '.' + Adw.get_minor_version()}));
'''.replace('MODULE', json.dumps((ROOT / 'extension/lay@radislabus-star.github.io/settings_view.js').as_uri())))
            env = dict(os.environ, HOME=str(home), LAY_CONFIG_PATH=str(selected),
                       PATH=str(fake) + os.pathsep + os.environ['PATH'])
            result = subprocess.run(['gjs', '-m', str(script)], env=env,
                                    text=True, capture_output=True, timeout=15)
            self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
            self.assertIn('adwaita', json.loads(result.stdout.strip()))
            saved = json.loads(selected.read_text())
            self.assertEqual(saved['preserve'], 'session')
            self.assertTrue(saved['learning_log'])
            self.assertTrue(saved['debug_action_log'])
            self.assertTrue(saved['nanda_trace'])
            self.assertTrue(saved['nanda_trace_text'])
            self.assertFalse(saved['force_layout_hotkeys'])
            self.assertEqual(saved['tap_max_ms'], 275)
            self.assertEqual(default.read_text(), '{"preserve": "default"}\n')


if __name__ == '__main__':
    unittest.main()
