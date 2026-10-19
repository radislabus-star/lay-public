#!/usr/bin/env python3
"""Run the actual updater against isolated release metadata and Git tags."""
import fcntl
import json
import os
from pathlib import Path
import shutil
import subprocess
import tempfile
import unittest

ROOT = Path(__file__).resolve().parents[1]


class UpdateMenuTests(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory(prefix='lay update $(literal) ')
        self.addCleanup(self.tmp.cleanup)
        self.base = Path(self.tmp.name)
        self.home = self.base / 'home'
        self.bin = self.home / '.local/bin'
        self.bin.mkdir(parents=True)
        self.repo = self.base / 'source with spaces'
        self.repo.mkdir()
        self.api = self.base / 'release.json'
        self.api.write_text(json.dumps(dict(tag_name='v0.0.2', draft=False, prerelease=False)))
        self.env = dict(os.environ, HOME=str(self.home),
                        XDG_STATE_HOME=str(self.home / '.state'),
                        XDG_DATA_HOME=str(self.home / '.data'),
                        XDG_CURRENT_DESKTOP='', XDG_SESSION_DESKTOP='', DESKTOP_SESSION='',
                        PATH=str(self.bin) + os.pathsep + os.environ['PATH'],
                        GIT_CONFIG_GLOBAL='/dev/null', GIT_CONFIG_NOSYSTEM='1',
                        LAY_UPDATE_RELEASE_API=self.api.as_uri())
        self.git('init', '-q', '-b', 'main')
        self.git('config', 'user.name', 'lay-test')
        self.git('config', 'user.email', 'lay@example.invalid')
        shutil.copy2(ROOT / 'update.sh', self.repo / 'update.sh')
        (self.repo / 'Cargo.toml').write_text('[package]\nname = "lay"\nversion = "0.0.1"\n')
        self.git('add', '.')
        self.git('commit', '-qm', 'old developer branch')
        self.origin = self.base / 'origin.git'
        self.git('clone', '-q', '--bare', str(self.repo), str(self.origin))
        self.git('remote', 'add', 'origin', str(self.origin))
        self.git('fetch', '-q', 'origin')
        self.git('branch', '--set-upstream-to=origin/main')
        self.env['LAY_UPDATE_REPOSITORY_URL'] = str(self.origin)
        seed = self.base / 'release-seed'
        self.git('clone', '-q', str(self.origin), str(seed))
        (seed / 'Cargo.toml').write_text('[package]\nname = "lay"\nversion = "0.0.2"\n')
        (seed / 'install.sh').write_text(
            'test "$1" = --update || exit 66\n'
            'printf "%s %s" "$LAY_KEEP_OLLAMA" "$1" > "$HOME/installed"\n'
            'printf \'#!/bin/bash\\necho "lay 0.0.2"\\n\' > "$HOME/.local/bin/lay"\n'
            'chmod +x "$HOME/.local/bin/lay"\n')
        self.git('add', '.', cwd=seed)
        self.git('-c', 'user.name=test', '-c', 'user.email=test@example.invalid',
                 'commit', '-qm', 'stable release', cwd=seed)
        self.git('-c', 'user.name=test', '-c', 'user.email=test@example.invalid',
                 'tag', '-a', 'v0.0.2', '-m', 'stable', cwd=seed)
        self.git('push', '-q', 'origin', 'refs/tags/v0.0.2', cwd=seed)
        self.script('lay', 'echo "lay 0.0.1"')
        self.script('systemctl', 'printf "%s\\n" "$*" >> "$HOME/runtime-calls"\n'
                    'case "$*" in\n'
                    '*show*) cat "$HOME/timer-state" 2>/dev/null || true ;;\n'
                    '*enable*) echo enabled > "$HOME/timer-state" ;;\n'
                    '*disable*) echo disabled > "$HOME/timer-state" ;;\n'
                    '*is-active*) exit 3 ;;\n'
                    'esac')
        (self.bin / 'lay-update').symlink_to(self.repo / 'update.sh')
        self.cache = self.home / '.data/lay/update-source'

    def git(self, *args, cwd=None):
        return subprocess.check_output(['git', *args], cwd=cwd or self.repo,
                                       env=self.env, stderr=subprocess.DEVNULL, text=True).strip()

    def script(self, name, body):
        path = self.bin / name
        path.write_text('#!/bin/bash\n' + body + '\n')
        path.chmod(0o755)

    def update(self, *args, input='\n', expected=0):
        result = subprocess.run(['bash', str(self.bin / 'lay-update'), *args], env=self.env,
                                cwd=self.base, input=input, text=True, capture_output=True, timeout=15)
        self.assertEqual(result.returncode, expected, result.stdout + result.stderr)
        return result

    def test_old_upstream_with_zero_new_commits_still_discovers_public_release(self):
        self.assertEqual(self.git('rev-parse', 'HEAD'), self.git('rev-parse', '@{u}'))
        result = self.update('--check')
        self.assertIn('0.0.1 → 0.0.2', result.stdout)
        self.assertNotIn('обновлений нет', result.stdout)
        self.assertFalse(self.cache.exists())
        self.assertFalse((self.home / 'installed').exists())
        self.assertFalse((self.home / 'runtime-calls').exists())

    def test_current_and_newer_versions_do_not_install(self):
        for version in ('0.0.2', '0.0.10'):
            with self.subTest(version=version):
                self.script('lay', f'echo "lay {version}"')
                self.update('--automatic')
                self.assertFalse((self.home / 'installed').exists())

    def test_network_error_never_reports_no_updates(self):
        self.env['LAY_UPDATE_RELEASE_API'] = (self.base / 'missing.json').as_uri()
        result = self.update('--check', expected=1)
        self.assertIn('Ошибка проверки', result.stderr)
        self.assertNotIn('обновлений нет', result.stdout)

    def test_invalid_release_and_installed_versions_are_errors(self):
        for data in (dict(tag_name='v0.0.2', draft=True, prerelease=False),
                     dict(tag_name='v0.0.2', draft=False, prerelease=True),
                     dict(tag_name='v0.0.2;touch bad', draft=False, prerelease=False), []):
            self.api.write_text(json.dumps(data))
            self.update('--check', expected=1)
        self.api.write_text(json.dumps(dict(tag_name='v0.0.2', draft=False, prerelease=False)))
        self.script('lay', 'echo broken-version')
        self.update('--check', expected=1)

    def test_declining_preserves_dirty_developer_work(self):
        head = self.git('rev-parse', 'HEAD')
        (self.repo / 'Cargo.toml').write_text('user edits')
        result = self.update('--interactive', input='n\n')
        self.assertIn('Обновление отменено', result.stdout)
        self.assertEqual(head, self.git('rev-parse', 'HEAD'))
        self.assertEqual('', self.git('stash', 'list'))
        self.assertEqual('user edits', (self.repo / 'Cargo.toml').read_text())
        self.assertFalse(self.cache.exists())

    def test_automatic_fetches_exact_tag_and_calls_restricted_installer(self):
        head = self.git('rev-parse', 'HEAD')
        self.update('--automatic', input='')
        self.assertEqual('1 --update', (self.home / 'installed').read_text())
        self.assertEqual(head, self.git('rev-parse', 'HEAD'))
        self.assertEqual(self.git('rev-parse', 'v0.0.2^{}', cwd=self.origin),
                         self.git('rev-parse', 'HEAD', cwd=self.cache))
        self.assertTrue((self.home / '.state/lay/installed-release').read_text().startswith('v0.0.2 '))
        self.assertEqual(['--user try-restart lay-daemon'],
                         (self.home / 'runtime-calls').read_text().splitlines())

    def test_unowned_source_directory_is_not_overwritten(self):
        self.cache.mkdir(parents=True)
        (self.cache / 'important').write_text('keep')
        self.update('--automatic', expected=1)
        self.assertEqual('keep', (self.cache / 'important').read_text())

    def test_dirty_owned_source_is_not_reset(self):
        self.update('--automatic')
        self.script('lay', 'echo "lay 0.0.1"')
        (self.cache / 'Cargo.toml').write_text('local changes')
        self.update('--automatic', expected=1)
        self.assertEqual('local changes', (self.cache / 'Cargo.toml').read_text())

    def test_missing_tag_and_wrong_manifest_are_errors(self):
        self.api.write_text(json.dumps(dict(tag_name='v0.0.3', draft=False, prerelease=False)))
        self.update('--automatic', expected=128)
        self.git('tag', 'v0.0.3', 'v0.0.2^{}', cwd=self.origin)
        self.update('--automatic', expected=1)
        self.assertFalse((self.home / 'installed').exists())

    def terminal(self, desktop):
        self.env['XDG_CURRENT_DESKTOP'] = desktop
        name = 'konsole' if desktop == 'KDE' else 'gnome-terminal'
        self.script(name, 'printf "%s\\n" "$@" > "$HOME/terminal-args"\n'
                         'shift; shift; exec "$@"')

    def test_both_desktops_launch_same_owner_with_quoted_arguments(self):
        for desktop, prefix in [('GNOME', ['--wait', '--']), ('KDE', ['--separate', '-e'])]:
            with self.subTest(desktop=desktop):
                self.terminal(desktop)
                self.update('--menu', input='n\n')
                argv = (self.home / 'terminal-args').read_text().splitlines()
                self.assertEqual(argv[:2], prefix)
                self.assertEqual(argv[-1], str(self.repo / 'update.sh'))
                self.assertIn('--interactive', argv[-3])
                self.assertFalse((self.home / 'installed').exists())

    def test_missing_terminal_fails_clearly(self):
        limited = self.base / 'limited-bin'
        limited.mkdir()
        for binary in ['bash', 'readlink']:
            (limited / binary).symlink_to(shutil.which(binary))
        self.env['PATH'] = str(limited)
        self.assertIn('Не найден терминал', self.update('--menu', expected=1).stderr)

    def test_terminal_fallbacks_keep_the_existing_supported_clients(self):
        limited = self.base / 'terminal-bin'
        limited.mkdir()
        for binary in ['bash', 'readlink']:
            (limited / binary).symlink_to(shutil.which(binary))
        self.env['PATH'] = str(limited)
        for name, prefix in [('kgx', ['--']), ('wezterm', ['start', '--']),
                             ('kitty', ['-e']), ('xterm', ['-e'])]:
            with self.subTest(terminal=name):
                terminal = limited / name
                terminal.write_text('#!/bin/bash\nprintf "%s\\n" "$@"\n')
                terminal.chmod(0o755)
                argv = self.update('--menu').stdout.splitlines()
                self.assertEqual(prefix, argv[:len(prefix)])
                self.assertEqual(str(self.repo / 'update.sh'), argv[-1])
                terminal.unlink()

    def test_terminal_keeps_check_failure_exit_code(self):
        self.terminal('GNOME')
        self.env['LAY_UPDATE_RELEASE_API'] = (self.base / 'missing.json').as_uri()
        result = self.update('--menu', expected=1)
        self.assertIn('Нажми Enter', result.stdout)
        self.assertNotIn('обновлений нет', result.stdout)

    def test_timer_switches_share_state_without_input_restart(self):
        self.assertIn('disabled', self.update('--auto-status').stdout)
        self.update('--auto-enable')
        self.assertIn('enabled', self.update('--auto-status').stdout)
        self.update('--auto-disable')
        self.assertIn('disabled', self.update('--auto-status').stdout)
        calls = (self.home / 'runtime-calls').read_text()
        self.assertNotIn('restart', calls)
        self.assertNotIn('lay-daemon', calls)

    def test_timer_control_failure_is_an_error(self):
        self.script('systemctl', 'echo "bus unavailable" >&2; exit 1')
        self.assertIn('bus unavailable', self.update('--auto-enable', expected=1).stderr)
        self.update('--auto-status', expected=1)

    def test_source_only_is_explicit_and_never_installs(self):
        self.update('--source-only')
        self.assertFalse((self.home / 'installed').exists())

    def test_concurrent_update_is_excluded(self):
        state = self.home / '.state/lay'
        state.mkdir(parents=True)
        with (state / 'update.lock').open('w') as lock:
            fcntl.flock(lock, fcntl.LOCK_EX | fcntl.LOCK_NB)
            self.assertIn('уже выполняется', self.update('--check', expected=75).stderr)


if __name__ == '__main__':
    unittest.main()
