#!/usr/bin/env python3
"""Exercise the real updater in isolated Git checkouts; never touch live Lay."""
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
        self.env = dict(os.environ, HOME=str(self.home), XDG_STATE_HOME=str(self.home / '.state'),
                        XDG_CURRENT_DESKTOP='GNOME', XDG_SESSION_DESKTOP='', DESKTOP_SESSION='',
                        PATH=str(self.bin) + os.pathsep + os.environ['PATH'],
                        GIT_CONFIG_GLOBAL='/dev/null', GIT_CONFIG_NOSYSTEM='1')
        self.git('init', '-q', '-b', 'main')
        self.git('config', 'user.name', 'lay-test')
        self.git('config', 'user.email', 'lay@example.invalid')
        shutil.copy2(ROOT / 'update.sh', self.repo / 'update.sh')
        (self.repo / 'Cargo.toml').write_text('[package]\nname = "lay"\nversion = "0.0.1"\n')
        (self.repo / 'install.sh').write_text('printf "%s" "$LAY_KEEP_OLLAMA" > "$HOME/installed"\n')
        self.git('add', '.')
        self.git('commit', '-qm', 'initial')
        self.origin = self.base / 'origin.git'
        self.git('clone', '-q', '--bare', str(self.repo), str(self.origin))
        self.git('remote', 'add', 'origin', str(self.origin))
        self.git('fetch', '-q', 'origin')
        self.git('branch', '--set-upstream-to=origin/main')
        self.script('lay', 'echo "lay 0.0.1"')
        self.script('gdbus', 'exit 0')
        self.script('systemctl', 'echo called >> "$HOME/runtime-calls"')
        self.script('pkill', 'exit 0')
        (self.bin / 'lay-update').symlink_to(self.repo / 'update.sh')

    def git(self, *args, cwd=None):
        return subprocess.check_output(['git', *args], cwd=cwd or self.repo,
                                       env=self.env, stderr=subprocess.DEVNULL, text=True).strip()

    def script(self, name, body):
        path = self.bin / name
        path.write_text('#!/bin/bash\n' + body + '\n')
        path.chmod(0o755)

    def update(self, *args, input='\n', expected=0):
        result = subprocess.run(['bash', str(self.bin / 'lay-update'), *args], env=self.env,
                                cwd=self.base, input=input, text=True, capture_output=True, timeout=10)
        self.assertEqual(result.returncode, expected, result.stdout + result.stderr)
        return result

    def upstream_change(self):
        seed = self.base / 'seed'
        self.git('clone', '-q', str(self.origin), str(seed))
        (seed / 'release.txt').write_text('new upstream\n')
        self.git('add', '.', cwd=seed)
        self.git('-c', 'user.name=test', '-c', 'user.email=test@example.invalid',
                 'commit', '-qm', 'release', cwd=seed)
        self.git('push', '-q', 'origin', 'main', cwd=seed)

    def terminal(self, desktop):
        self.env['XDG_CURRENT_DESKTOP'] = desktop
        name = 'konsole' if desktop == 'KDE' else 'gnome-terminal'
        self.script(name, 'printf "%s\\n" "$@" > "$HOME/terminal-args"\n'
                         'if [ "$1" = --wait ]; then shift; fi\nshift\nexec "$@"')

    def test_installed_symlink_resolves_owner_and_check_is_observational(self):
        self.upstream_change()
        head = self.git('rev-parse', 'HEAD')
        result = self.update('--check')
        self.assertIn('новых коммитов: 1', result.stdout)
        self.assertEqual(head, self.git('rev-parse', 'HEAD'))
        self.assertFalse((self.home / 'installed').exists())
        self.assertFalse((self.home / 'runtime-calls').exists())

    def test_source_only_never_repairs_an_installed_version_mismatch(self):
        self.script('lay', 'echo "lay 0.0.0"')
        self.update('--source-only')
        self.assertFalse((self.home / 'installed').exists())
        self.assertFalse((self.home / 'runtime-calls').exists())

    def test_both_desktops_launch_same_owner_with_quoted_arguments(self):
        for desktop, prefix in [('GNOME', ['--wait', '--']), ('KDE', ['-e'])]:
            with self.subTest(desktop=desktop):
                self.terminal(desktop)
                self.update('--menu')
                argv = (self.home / 'terminal-args').read_text().splitlines()
                self.assertEqual(argv[:len(prefix)], prefix)
                self.assertEqual(argv[-1], str(self.repo / 'update.sh'))
                self.assertIn('--interactive', argv[-3])
                self.assertFalse((self.home / 'installed').exists())

    def test_missing_terminal_fails_clearly(self):
        limited = self.base / 'limited-bin'
        limited.mkdir()
        for binary in ['bash', 'dirname', 'readlink']:
            (limited / binary).symlink_to(shutil.which(binary))
        self.env['PATH'] = str(limited)
        result = self.update('--menu', expected=1)
        self.assertIn('Не найден терминал', result.stderr)

    def test_terminal_keeps_updater_error_and_exit_code(self):
        self.terminal('GNOME')
        self.script('git', 'exit 42')
        result = self.update('--menu', expected=1)
        self.assertIn('Ошибка обновления (код 1)', result.stdout)
        self.assertIn('Нажми Enter', result.stdout)

    def test_declining_does_not_stash_pull_or_install(self):
        self.upstream_change()
        head = self.git('rev-parse', 'HEAD')
        (self.repo / 'user-work.txt').write_text('keep me')
        result = self.update('--interactive', input='n\n')
        self.assertIn('Обновление отменено', result.stdout)
        self.assertEqual(head, self.git('rev-parse', 'HEAD'))
        self.assertEqual('', self.git('stash', 'list'))
        self.assertTrue((self.repo / 'user-work.txt').exists())
        self.assertFalse((self.home / 'installed').exists())

    def test_confirmed_reinstall_preserves_ollama_and_uses_kde_refresh(self):
        self.script('lay', 'echo "lay 0.0.0"')
        self.env['XDG_CURRENT_DESKTOP'] = 'KDE'
        self.update('--interactive', input='y\n')
        self.assertEqual('1', (self.home / 'installed').read_text())
        self.assertTrue((self.home / 'runtime-calls').exists())

    def test_no_tracking_uses_the_checked_fallback(self):
        self.upstream_change()
        self.git('branch', '--unset-upstream')
        self.update('--source-only')
        self.assertTrue((self.repo / 'release.txt').exists())
        self.assertFalse((self.home / 'installed').exists())

    def test_divergence_refuses_before_stashing(self):
        self.upstream_change()
        (self.repo / 'local.txt').write_text('local commit')
        self.git('add', 'local.txt')
        self.git('commit', '-qm', 'local')
        (self.repo / 'user-work.txt').write_text('keep me')
        self.update('--source-only', expected=1)
        self.assertEqual('', self.git('stash', 'list'))
        self.assertTrue((self.repo / 'user-work.txt').exists())

    def test_concurrent_update_is_excluded(self):
        state = self.home / '.state/lay'
        state.mkdir(parents=True)
        with (state / 'update.lock').open('w') as lock:
            fcntl.flock(lock, fcntl.LOCK_EX | fcntl.LOCK_NB)
            result = self.update('--check', expected=75)
            self.assertIn('уже выполняется', result.stderr)


if __name__ == '__main__':
    unittest.main()
