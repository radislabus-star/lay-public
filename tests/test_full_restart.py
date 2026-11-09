"""Complete recovery executes the existing controller against isolated peers."""
import importlib.util
import json
import os
from pathlib import Path
import socket
import subprocess
import tempfile
import unittest

ROOT = Path(__file__).resolve().parents[1]
spec = importlib.util.spec_from_file_location("full_restart_startup_peers", ROOT / "tests/test_kde_ime_startup.py")
peers = importlib.util.module_from_spec(spec)
spec.loader.exec_module(peers)
# Reuse the existing desktop peers; extend only lifecycle operations.
STUB = peers.STUB.replace("if name == 'systemctl' else", "if name in ('systemctl', 'systemd-run') else")
STUB = STUB.replace("if name == 'systemctl': sys.exit(0)", r'''
if name == 'systemd-run': sys.exit(state.get('launch_status', 0))
if name == 'notify-send': sys.exit(0)
if name == 'gnome-extensions':
    if args == ['list', '--enabled'] and state['extension_enabled']:
        print('lay@radislabus-star.github.io')
    if args[0] == 'enable' and state.get('extension_failure'): sys.exit(7)
    sys.exit(0)
if name == 'systemctl':
    operation = args[1]
    unit = args[-1]
    if operation == 'show': print('loaded' if state['gnome_unit_loaded'] else 'not-found')
    if operation in ('is-active', 'is-enabled'):
        if unit == 'lay-l3-online.service':
            active = state.get('l3_enabled', state['l3_active']) if operation == 'is-enabled' else state['l3_active']
            sys.exit(0 if active else 3)
        if unit == 'lay-kde-tray.service': sys.exit(0 if state.get('tray_active') else 3)
    if operation == 'restart' and unit == state.get('fail_unit'): sys.exit(7)
    if operation == 'restart' and unit == 'org.freedesktop.IBus.session.GNOME.service':
        state['engine'] = ''; state['restarted'] = True; p.write_text(json.dumps(state))
    sys.exit(0)
''')
STUB = STUB.replace("if name == 'ibus':", """if name == 'ibus' and args == ['restart']:
    state['engine'] = ''; state['restarted'] = True; p.write_text(json.dumps(state)); sys.exit(0)
if name == 'ibus' and state.get('restarted') and os.environ.get('IBUS_ADDRESS'):
    sys.exit(89)
if name == 'ibus':""")


class FullRestartTests(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory(prefix="lay-full-restart-")
        self.addCleanup(self.tmp.cleanup)
        self.home = Path(self.tmp.name)
        self.bin = self.home / 'bin'
        self.bin.mkdir()
        for command in ('systemctl', 'systemd-run', 'ibus', 'gdbus', 'gsettings',
                        'pgrep', 'gnome-extensions', 'notify-send'):
            path = self.bin / command
            path.write_text(STUB)
            path.chmod(0o755)
        self.state = self.home / 'state.json'
        self.log = self.home / 'commands.jsonl'
        self.config = self.home / 'lay/config.json'
        self.config.parent.mkdir()
        self.env = dict(os.environ, HOME=str(self.home), PATH=f'{self.bin}:/usr/bin:/bin',
                        XDG_CONFIG_HOME=str(self.home), XDG_RUNTIME_DIR=str(self.home),
                        DISPLAY=':999', WAYLAND_DISPLAY='', XDG_CURRENT_DESKTOP='GNOME',
                        XDG_SESSION_DESKTOP='', DESKTOP_SESSION='',
                        DBUS_SESSION_BUS_ADDRESS='unix:path=/nonexistent-isolated-bus',
                        STUB_STATE=str(self.state), STUB_LOG=str(self.log))

    def run_worker(self, *, kde=False, index=1, backend='ime', **extra):
        state = dict(kde=kde, index=index, layouts=[['us', 'English', ''], ['ru', 'Russian', '']],
                     engine='lay-ime-ru' if index else 'lay-ime-us', system_map=True,
                     l3_active=True, extension_enabled=True, gnome_unit_loaded=True)
        state.update(extra)
        self.state.write_text(json.dumps(state))
        self.config.write_text(json.dumps(dict(text_backend=backend)))
        self.env['XDG_CURRENT_DESKTOP'] = 'KDE' if kde else 'GNOME'
        result = subprocess.run(['bash', str(ROOT / 'scripts/lay-full-restart.sh')],
                                env=self.env, capture_output=True, text=True, timeout=20)
        return result, json.loads(self.state.read_text()), self.commands()

    def commands(self):
        return [json.loads(line) for line in self.log.read_text().splitlines()]

    def test_both_desktops_restore_both_languages_after_fresh_global_engine(self):
        for kde in (False, True):
            for index in (0, 1):
                with self.subTest(kde=kde, index=index):
                    self.log.unlink(missing_ok=True)
                    result, state, calls = self.run_worker(kde=kde, index=index)
                    self.assertEqual(result.returncode, 0, result.stderr)
                    self.assertEqual(state['engine'], 'lay-ime-ru' if index else 'lay-ime-us')
                    restart = (['ibus', 'restart'] if kde else
                               ['systemctl', '--user', 'restart', 'org.freedesktop.IBus.session.GNOME.service'])
                    pos = calls.index(restart)
                    first_engine = next(c for c in calls[pos + 1:] if c[:2] == ['ibus', 'engine'] and len(c) == 3)
                    self.assertEqual(first_engine[2], 'xkb:ru::rus' if index else 'xkb:us::eng')
                    self.assertLess(calls.index(['systemctl', '--user', 'stop', 'lay-daemon.service']), pos)
                    self.assertIn(['systemctl', '--user', 'restart', 'lay-l3-online.service'], calls)
                    self.assertIn(['systemctl', '--user', 'start', 'lay-daemon.service'], calls)
                    self.assertEqual(state['index'], index)
                    if kde:
                        self.assertFalse(any(c[0] == 'gnome-extensions' for c in calls))
                        self.assertFalse(any('org.freedesktop.IBus.session.GNOME.service' in c for c in calls))
                    else:
                        self.assertIn(['gnome-extensions', 'enable', 'lay@radislabus-star.github.io'], calls)

    def test_non_ime_mode_preserves_global_ibus(self):
        result, _, calls = self.run_worker(backend='uinput')
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertFalse(any(c[0] == 'ibus' for c in calls))
        self.assertFalse(any('lay-ibus-engine.service' in c for c in calls))
        self.assertFalse(any('org.freedesktop.IBus.session.GNOME.service' in c for c in calls))

    def test_old_ibus_socket_address_is_discarded_after_restart(self):
        self.env['IBUS_ADDRESS'] = 'unix:path=/retired-ibus-socket'
        result, state, _ = self.run_worker()
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(state['engine'], 'lay-ime-ru')

    def test_inactive_optional_services_are_not_started(self):
        result, _, calls = self.run_worker(kde=True, l3_active=False)
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertNotIn(['systemctl', '--user', 'restart', 'lay-l3-online.service'], calls)
        self.assertNotIn(['systemctl', '--user', 'restart', 'lay-kde-tray.service'], calls)

    def test_failed_ibus_restart_reports_failure_and_restores_previously_active_daemon(self):
        result, _, calls = self.run_worker(fail_unit='org.freedesktop.IBus.session.GNOME.service')
        self.assertNotEqual(result.returncode, 0)
        self.assertNotIn('полный перезапуск завершён', result.stdout)
        self.assertIn(['systemctl', '--user', 'start', 'lay-daemon.service'], calls)
        self.assertIn('полный перезапуск не завершён', result.stderr)

    def test_failed_but_enabled_l3_service_is_recovered(self):
        result, _, calls = self.run_worker(l3_active=False, l3_enabled=True)
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertIn(['systemctl', '--user', 'restart', 'lay-l3-online.service'], calls)

    def test_foreign_socket_peer_is_never_terminated(self):
        with socket.socket(socket.AF_UNIX, socket.SOCK_STREAM) as server:
            server.bind(str(self.home / 'lay-l11.sock'))
            server.listen(1)
            result, _, calls = self.run_worker()
        self.assertNotEqual(result.returncode, 0)
        self.assertIn('foreign L1.1 socket peer', result.stderr)
        self.assertNotIn(['systemctl', '--user', 'restart', 'org.freedesktop.IBus.session.GNOME.service'], calls)

    def test_launcher_is_detached_and_keeps_gui_bus_distinct_from_manager_bus(self):
        self.state.write_text(json.dumps(dict(launch_status=0)))
        result = subprocess.run(['bash', str(ROOT / 'scripts/lay-runtime-control.sh'), 'full-restart'],
                                env=self.env, capture_output=True, text=True, timeout=5)
        self.assertEqual(result.returncode, 0, result.stderr)
        command = self.commands()[-1]
        self.assertEqual(command[0], 'systemd-run')
        self.assertIn('--unit=lay-full-restart.service', command)
        self.assertIn('--no-block', command)
        self.assertIn('--setenv=DBUS_SESSION_BUS_ADDRESS=unix:path=/nonexistent-isolated-bus', command)
        self.assertEqual(command[-1], str(ROOT / 'scripts/lay-full-restart.sh'))

    def test_launcher_failure_is_propagated(self):
        self.state.write_text(json.dumps(dict(launch_status=7)))
        result = subprocess.run(['bash', str(ROOT / 'scripts/lay-runtime-control.sh'), 'full-restart'],
                                env=self.env, capture_output=True, text=True, timeout=5)
        self.assertEqual(result.returncode, 7, result.stderr)


if __name__ == '__main__':
    unittest.main()
