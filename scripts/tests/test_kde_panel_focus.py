"""Exercise the real KConfig tools against disposable files, never a live profile."""
import importlib.util
import os
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile
import unittest

SCRIPT = Path(__file__).resolve().parents[1] / 'compat/lay-kde-panel-focus.py'
spec = importlib.util.spec_from_file_location('kde_panel_focus', SCRIPT)
policy = importlib.util.module_from_spec(spec)
spec.loader.exec_module(policy)
TOOLS = any(shutil.which('kreadconfig' + n) and shutil.which('kwriteconfig' + n) for n in ('5', '6'))


class RemovalWithoutKde(unittest.TestCase):
    def test_absent_configuration_does_not_require_kde(self):
        with tempfile.TemporaryDirectory() as directory:
            target = Path(directory) / 'kwinrulesrc'
            run = subprocess.run([sys.executable, str(SCRIPT), '--remove', '--config', str(target)],
                                 env={**os.environ, 'PATH': '/nonexistent'}, capture_output=True, text=True)
            self.assertEqual(run.returncode, 0, run.stderr)
            self.assertFalse(target.exists())


@unittest.skipUnless(TOOLS, 'real KDE KConfig tools required')
class KdePanelFocus(unittest.TestCase):
    def setUp(self):
        self.directory = tempfile.TemporaryDirectory()
        self.addCleanup(self.directory.cleanup)
        self.path = Path(self.directory.name) / 'kwinrulesrc'

    def test_new_profile_uses_exact_nonfocus_match(self):
        self.assertTrue(policy.configure(self.path))
        text = self.path.read_text()
        for line in ('wmclass=ibus-ui-gtk3', 'title=ibus-ui-gtk3', 'types=1',
                     'acceptfocus=false', 'acceptfocusrule=2', 'rules=' + policy.RULE_ID):
            self.assertIn(line, text)

    def test_preserves_existing_rules_and_is_idempotent(self):
        self.path.write_text('[General]\ncount=1\nrules=user-rule\n\n[user-rule]\nDescription=My window\nwmclass=konsole\nacceptfocus=true\n')
        policy.configure(self.path)
        installed = self.path.read_bytes()
        self.assertIn(b'rules=user-rule,' + policy.RULE_ID.encode(), installed)
        self.assertIn(b'wmclass=konsole\n', installed)
        self.assertIn(b'acceptfocus=true\n', installed)
        self.assertFalse(policy.configure(self.path))
        self.assertEqual(self.path.read_bytes(), installed)
        self.assertTrue(policy.configure(self.path, remove=True))
        removed = self.path.read_text()
        self.assertIn('rules=user-rule\n', removed)
        self.assertIn('Description=My window\n', removed)
        self.assertNotIn('[' + policy.RULE_ID + ']', removed)

    def test_legacy_numbered_rules_remain_registered(self):
        self.path.write_text('[General]\ncount=2\n\n[1]\nDescription=First\n\n[2]\nDescription=Second\n')
        policy.configure(self.path)
        self.assertIn('rules=1,2,' + policy.RULE_ID, self.path.read_text())
        policy.configure(self.path, remove=True)
        self.assertIn('rules=1,2\n', self.path.read_text())
        self.assertIn('count=2\n', self.path.read_text())

    def test_remove_preserves_user_edits_and_other_owned_group_keys(self):
        policy.configure(self.path)
        self.path.write_text(self.path.read_text().replace('acceptfocus=false', 'acceptfocus=true') + 'custom-user-key=keep\n')
        policy.configure(self.path, remove=True)
        self.assertIn('acceptfocus=true\n', self.path.read_text())
        self.assertIn('custom-user-key=keep\n', self.path.read_text())

    def test_remove_absent_rule_leaves_file_untouched(self):
        self.assertFalse(policy.configure(self.path, remove=True))
        self.assertFalse(self.path.exists())
        self.path.write_text('[General]\ncount=0\n')
        before = self.path.read_bytes()
        self.assertFalse(policy.configure(self.path, remove=True))
        self.assertEqual(self.path.read_bytes(), before)

    def test_missing_tools_does_not_change_configuration(self):
        self.path.write_text('[General]\ncount=0\n')
        before = self.path.read_bytes()
        run = subprocess.run([sys.executable, str(SCRIPT), '--config', str(self.path)],
                             env={**os.environ, 'PATH': '/nonexistent'}, capture_output=True, text=True)
        self.assertNotEqual(run.returncode, 0)
        self.assertEqual(self.path.read_bytes(), before)

    def test_reinstall_tracks_kwin_migrated_numbered_rule(self):
        fixture = Path(__file__).with_name('fixtures') / 'kde-kwinrules-migrated.ini'
        self.path.write_bytes(fixture.read_bytes())
        policy.configure(self.path)
        text = self.path.read_text()
        self.assertIn('rules=1\n', text)
        self.assertIn('count=1\n', text)
        self.assertNotIn('rules=1,' + policy.RULE_ID, text)
        self.assertFalse(policy.configure(self.path))

    def test_remove_tracks_migrated_rule_and_cleans_original_orphan(self):
        fixture = Path(__file__).with_name('fixtures') / 'kde-kwinrules-migrated.ini'
        self.path.write_bytes(fixture.read_bytes())
        policy.configure(self.path, remove=True)
        text = self.path.read_text()
        self.assertIn('count=0\n', text)
        self.assertIn('rules=\n', text)
        self.assertNotIn('wmclass=ibus-ui-gtk3', text)
        self.assertFalse(policy.configure(self.path, remove=True))


if __name__ == '__main__':
    unittest.main()
