#!/usr/bin/env python3
"""Exercise graphical-session installation without touching the user's manager."""
import os
from pathlib import Path
import subprocess
import tempfile
import unittest
import shutil

ROOT = Path(__file__).resolve().parents[1]
INSTALLER = ROOT / 'scripts/install-session-services.sh'
UNITS = ('lay-daemon.service', 'lay-l3-online.service')


class SessionServicesTests(unittest.TestCase):
    def test_graphical_services_are_not_pulled_into_default_target(self):
        for unit in UNITS:
            with self.subTest(unit=unit):
                source = (ROOT / 'systemd' / unit).read_text()
                self.assertIn('WantedBy=graphical-session.target', source)
                self.assertIn('PartOf=graphical-session.target', source)
                self.assertNotIn('WantedBy=default.target', source)

    def exercise(self, mode, enabled):
        with tempfile.TemporaryDirectory(prefix='lay-session-install-') as directory:
            home = Path(directory)
            commands = home / 'commands'
            commands.mkdir()
            systemctl = commands / 'systemctl'
            systemctl.write_text('''#!/usr/bin/env python3
import json, os, pathlib, sys
args = sys.argv[1:]
with open(os.environ['CALLS'], 'a') as log:
    log.write(json.dumps(args) + '\\n')
if 'is-enabled' in args:
    sys.exit(0 if args[-1] in os.environ['ENABLED'].split(',') else 1)
''')
            systemctl.chmod(0o755)
            env = {**os.environ, 'HOME': str(home),
                   'XDG_CONFIG_HOME': str(home / 'config'),
                   'PATH': str(commands) + os.pathsep + os.environ['PATH'],
                   'CALLS': str(home / 'calls'), 'ENABLED': ','.join(enabled)}
            result = subprocess.run(['bash', str(INSTALLER), mode], env=env,
                                    text=True, capture_output=True, timeout=10)
            self.assertEqual(result.returncode, 0, result.stderr)
            import json
            calls = [json.loads(line) for line in (home / 'calls').read_text().splitlines()]
            reenabled = {call[-1] for call in calls if 'reenable' in call}
            self.assertEqual(reenabled, set(UNITS if mode == '--fresh' else enabled))
            self.assertFalse(any(set(call) & {'start', 'stop', 'restart', '--now'}
                                 for call in calls), calls)
            for unit in UNITS:
                self.assertEqual((home / '.config/systemd/user' / unit).read_bytes(),
                                 (ROOT / 'systemd' / unit).read_bytes())
            self.assertFalse((home / 'config/systemd/user').exists())

    def test_upgrade_migrates_enabled_services_without_restarting(self):
        self.exercise('--update', UNITS)

    def test_upgrade_preserves_disabled_services(self):
        self.exercise('--update', ('lay-daemon.service',))
        self.exercise('--update', ())

    def test_fresh_install_enables_both_services(self):
        self.exercise('--fresh', ())

    def test_real_systemctl_reenable_removes_legacy_links(self):
        self.assertIsNotNone(shutil.which('systemctl'))
        with tempfile.TemporaryDirectory(prefix='lay-offline-units-') as directory:
            root = Path(directory)
            units = root / 'etc/systemd/user'
            legacy = units / 'default.target.wants'
            legacy.mkdir(parents=True)
            for unit in UNITS:
                shutil.copyfile(ROOT / 'systemd' / unit, units / unit)
                (legacy / unit).symlink_to('../' + unit)
            for unit in UNITS:
                result = subprocess.run(['systemctl', '--global', '--root', str(root),
                                         '--no-reload', 'reenable', unit],
                                        text=True, capture_output=True, timeout=10)
                self.assertEqual(result.returncode, 0, result.stderr)
                self.assertFalse((legacy / unit).exists())
                self.assertTrue((units / 'graphical-session.target.wants' / unit).is_symlink())


if __name__ == '__main__':
    unittest.main()
