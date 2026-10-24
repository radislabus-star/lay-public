"""Execute real runtime-control startup with isolated desktop command peers."""
import json
import os
from pathlib import Path
import subprocess
import tempfile
import time
import unittest

ROOT = Path(__file__).resolve().parents[1]
SCRIPT = Path(os.environ.get('LAY_TEST_RUNTIME_CONTROL', ROOT / 'scripts/lay-runtime-control.sh'))
STUB = r'''#!/usr/bin/python3
import json, os, sys
from pathlib import Path
p = Path(os.environ['STUB_STATE']); state = json.loads(p.read_text())
name, args = Path(sys.argv[0]).name, sys.argv[1:]
expected_bus = ('unix:path=' + os.environ['XDG_RUNTIME_DIR'] + '/bus'
                if name == 'systemctl' else 'unix:path=/nonexistent-isolated-bus')
if os.environ.get('DBUS_SESSION_BUS_ADDRESS') != expected_bus:
    sys.exit(88)
with Path(os.environ['STUB_LOG']).open('a') as f:
    f.write(json.dumps([name, *args]) + '\n')
if name == 'pgrep': sys.exit(1)
if name == 'lay-ime-setup': sys.exit(0)
if name == 'systemctl': sys.exit(0)
if name == 'gsettings':
    if args[0] == 'set':
        if state.get('write_failure'): sys.exit(7)
        state['system_map'] = True; p.write_text(json.dumps(state))
    else: print('false' if state.get('readback_failure') else str(state['system_map']).lower())
    sys.exit(0)
if name == 'gdbus':
    method = args[args.index('--method') + 1]
    if method.endswith('.getLayoutsList'):
        if state.get('map_failure'): sys.exit(8)
        layouts = state['layouts']
        if state.get('transient_map_reads', 0):
            state['transient_map_reads'] -= 1
            p.write_text(json.dumps(state))
            layouts = [['us', 'English', '']]
        print(repr(([tuple(row) for row in layouts],)))
    elif method.endswith('.getLayout'):
        print('(uint32 %d,)' % state['index'])
    elif method.endswith('.CurrentLayout'):
        if state['kde']: sys.exit(1)
        print(repr((state['engine'],)))
    elif method.endswith('.ActivateLayout'):
        if state['kde']: sys.exit(1)
        state['engine'] = args[-1]; p.write_text(json.dumps(state)); print('()')
    else: sys.exit(9)
    sys.exit(0)
if name == 'ibus':
    if len(args) > 1:
        state['engine'] = args[1]
        if state.get('concurrent_group_change') and args[1].startswith('lay-ime-'):
            state['index'] = 1 - state['index']
        p.write_text(json.dumps(state))
    else: print(state['engine'])
    sys.exit(0)
sys.exit(10)
'''


class KdeImeStartupTests(unittest.TestCase):
    def run_start(self, *, layouts=None, index=0, kde=True, action=('channel', 'ime'),
                  setup=False, **extra):
        with tempfile.TemporaryDirectory(prefix='lay-kde-startup-') as directory:
            home = Path(directory)
            bin_dir = home / 'bin'; bin_dir.mkdir()
            for command in ('systemctl', 'gdbus', 'ibus', 'gsettings', 'pgrep'):
                path = bin_dir / command; path.write_text(STUB); path.chmod(0o755)
            if setup:
                helper = home / '.local/bin/lay-ime-setup'
                helper.parent.mkdir(parents=True)
                helper.write_text(STUB)
                helper.chmod(0o755)
            state = dict(layouts=layouts or [['us', 'English', ''], ['ru', 'Russian', '']],
                         index=index, kde=kde, engine='lay-ime-us', system_map=False, **extra)
            state_path, log = home / 'state.json', home / 'commands.jsonl'
            state_path.write_text(json.dumps(state))
            env = dict(os.environ, HOME=str(home), PATH=f'{bin_dir}:/usr/bin:/bin',
                       XDG_CONFIG_HOME=str(home), XDG_RUNTIME_DIR=str(home),
                       XDG_CURRENT_DESKTOP='KDE' if kde else 'GNOME',
                       XDG_SESSION_DESKTOP='', DESKTOP_SESSION='', DISPLAY=':999',
                       XAUTHORITY=str(home / 'authority'),
                       DBUS_SESSION_BUS_ADDRESS='unix:path=/nonexistent-isolated-bus',
                       STUB_STATE=str(state_path), STUB_LOG=str(log))
            result = subprocess.run(['bash', str(SCRIPT), *action], env=env,
                                    text=True, capture_output=True, timeout=12)
            if setup and action[0] not in ('daemon-start', 'daemon-stop', 'daemon-restart'):
                deadline = time.monotonic() + 1
                while 'lay-ime-setup' not in log.read_text() and time.monotonic() < deadline:
                    time.sleep(0.01)
            return result, json.loads(state_path.read_text()), [json.loads(s) for s in log.read_text().splitlines()]

    def test_current_named_layout_selects_matching_profile_without_setting_kde(self):
        for index, expected in ((0, 'lay-ime-us'), (1, 'lay-ime-ru')):
            with self.subTest(index=index):
                result, state, calls = self.run_start(index=index)
                self.assertEqual(result.returncode, 0, result.stderr)
                self.assertEqual(state['engine'], expected)
                self.assertTrue(state['system_map'])
                self.assertEqual(state['index'], index)
                self.assertFalse(any('setLayout' in ' '.join(c) for c in calls))
                write = next(i for i, c in enumerate(calls) if c[:2] == ['gsettings', 'set'])
                first_source = next(i for i, c in enumerate(calls) if c[:2] == ['ibus', 'engine'] and len(c) == 3)
                self.assertLess(write, first_source)

    def test_reordered_and_additional_layouts_keep_the_user_map(self):
        layouts = [['de', 'German', ''], ['ru', 'Russian', 'phonetic'], ['us', 'English', 'intl']]
        result, state, _ = self.run_start(layouts=layouts, index=1)
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(state['engine'], 'lay-ime-ru')
        self.assertEqual(state['layouts'], layouts)

    def test_transient_out_of_range_snapshot_settles_before_source_mutation(self):
        result, state, calls = self.run_start(index=1, transient_map_reads=2)
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(state['engine'], 'lay-ime-ru')
        self.assertEqual(state['transient_map_reads'], 0)
        first_source = next(i for i, c in enumerate(calls)
                            if c[:2] == ['ibus', 'engine'] and len(c) == 3)
        reads = [c for c in calls[:first_source] if 'getLayoutsList' in ' '.join(c)]
        self.assertGreaterEqual(len(reads), 4)

    def test_map_change_between_list_and_index_is_retried(self):
        result, state, calls = self.run_start(index=0, transient_map_reads=1)
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(state['engine'], 'lay-ime-us')
        self.assertEqual(state['transient_map_reads'], 0)
        first_source = next(i for i, c in enumerate(calls)
                            if c[:2] == ['ibus', 'engine'] and len(c) == 3)
        reads = [c for c in calls[:first_source] if 'getLayoutsList' in ' '.join(c)]
        self.assertGreaterEqual(len(reads), 4)

    def test_unsupported_ambiguous_invalid_or_unavailable_map_refuses_before_source_mutation(self):
        cases = [dict(layouts=[['de', 'German', '']]), dict(index=99),
                 dict(layouts=[['ru', 'Russian', ''], ['ru', 'Russian', 'phonetic']]),
                 dict(map_failure=True), dict(layouts=[['us', 'invalid']])]
        for case in cases:
            with self.subTest(case=case):
                result, _, calls = self.run_start(**case)
                self.assertNotEqual(result.returncode, 0)
                self.assertFalse(any(c[:2] == ['ibus', 'engine'] and len(c) == 3 for c in calls))

    def test_failed_preference_write_or_readback_prevents_source_mutation(self):
        for option in ('write_failure', 'readback_failure'):
            with self.subTest(option=option):
                result, _, calls = self.run_start(**{option: True})
                self.assertNotEqual(result.returncode, 0)
                self.assertFalse(any(c[:2] == ['ibus', 'engine'] and len(c) == 3 for c in calls))

    def test_concurrent_group_change_is_not_reported_as_startup_success(self):
        result, _, _ = self.run_start(concurrent_group_change=True)
        self.assertNotEqual(result.returncode, 0)

    def test_gnome_uses_existing_activation_without_kde_preparation(self):
        result, state, calls = self.run_start(kde=False)
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(state['engine'], 'lay-ime-us')
        self.assertFalse(any(c[0] == 'gsettings' or 'org.kde.keyboard' in c for c in calls))

    def test_daemon_only_controls_do_not_prepare_or_change_sources(self):
        for action in ('daemon-start', 'daemon-stop', 'daemon-restart'):
            result, _, calls = self.run_start(action=(action,))
            self.assertEqual(result.returncode, 0, result.stderr)
            self.assertTrue(all(c[0] == 'systemctl' for c in calls))

    def test_incomplete_ime_start_still_opens_setup_and_preserves_failure(self):
        result, _, calls = self.run_start(setup=True, map_failure=True)
        self.assertNotEqual(result.returncode, 0)
        self.assertEqual([c for c in calls if c[0] == 'lay-ime-setup'],
                         [['lay-ime-setup', '--show-if-needed']])

    def test_auto_channel_runs_the_same_setup_check(self):
        result, _, calls = self.run_start(setup=True, action=('channel', 'auto'))
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual([c for c in calls if c[0] == 'lay-ime-setup'],
                         [['lay-ime-setup', '--show-if-needed']])

    def test_daemon_only_control_does_not_open_setup(self):
        result, _, calls = self.run_start(setup=True, action=('daemon-restart',))
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertTrue(all(c[0] == 'systemctl' for c in calls))


if __name__ == '__main__':
    unittest.main()
