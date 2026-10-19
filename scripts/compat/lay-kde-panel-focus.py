#!/usr/bin/env python3
"""Install/remove one KWin rule without replacing the user's other window rules."""
import argparse
import configparser
import os
from pathlib import Path
import shutil
import subprocess
import tempfile

RULE_ID = 'lay-ibus-candidate-focus'
RULE_FILE = Path(__file__).resolve().parents[2] / 'data/kde/ibus-candidate-focus.kwinrules'


def configure(target, remove=False):
    target = target.expanduser().resolve()
    before = target.read_bytes() if target.exists() else None
    if remove and before is None:
        return False
    pair = next(((shutil.which('kreadconfig' + n), shutil.which('kwriteconfig' + n))
                 for n in ('6', '5') if shutil.which('kreadconfig' + n) and shutil.which('kwriteconfig' + n)), None)
    if pair is None:
        raise RuntimeError('KDE KConfig utilities are unavailable')
    template = configparser.ConfigParser(interpolation=None)
    template.optionxform = str
    template.read(RULE_FILE)
    values = dict(template[RULE_ID])
    target.parent.mkdir(parents=True, exist_ok=True)
    fd, name = tempfile.mkstemp(prefix='.lay-kwinrules-', dir=target.parent)
    temporary = Path(name)
    try:
        with os.fdopen(fd, 'wb') as stream:
            stream.write(before or b'')
        if before is not None:
            temporary.chmod(target.stat().st_mode & 0o777)

        def read(group, key, default=''):
            return subprocess.check_output([pair[0], '--file', str(temporary), '--group', group,
                                            '--key', key, '--default', default], text=True).rstrip('\n')

        def write(group, key, value=None):
            command = [pair[1], '--file', str(temporary), '--group', group, '--key', key]
            subprocess.run(command + (['--delete'] if value is None else [value]), check=True)

        raw = read('General', 'rules', '__LAY_MISSING_RULE_LIST__')
        rules = ([str(i) for i in range(1, int(read('General', 'count', '0')) + 1)]
                 if raw == '__LAY_MISSING_RULE_LIST__' else raw.split(',') if raw else [])
        # KWin 5 rewrites named groups to numbered groups on first startup.
        # Follow the installed rule's exact description and popup match after
        # that rewrite; its old group name is not a durable registration ID.
        owned = [group for group in rules if group == RULE_ID or all(
            read(group, key) == values[key]
            for key in ('Description', 'wmclass', 'title', 'types'))]
        if remove:
            groups = list(dict.fromkeys([*owned, RULE_ID]))
            if not owned and not any(read(RULE_ID, key) for key in values):
                return False
            rules = [item for item in rules if item not in owned]
            for group in groups:
                for key, value in values.items():
                    if read(group, key) == value:
                        write(group, key)
        else:
            if len(owned) > 1:
                raise RuntimeError('Multiple Lay panel rules are registered; leaving them untouched')
            group = owned[0] if owned else RULE_ID
            for key, value in values.items():
                write(group, key, value)
            if group not in rules:
                rules.append(group)
        write('General', 'rules', ','.join(rules))
        write('General', 'count', str(len(rules)))
        after = temporary.read_bytes()
        if after == before:
            return False
        current = target.read_bytes() if target.exists() else None
        if current != before:
            raise RuntimeError('KWin rules changed during installation; leaving them untouched')
        os.replace(temporary, target)
        return True
    finally:
        temporary.unlink(missing_ok=True)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--remove', action='store_true')
    parser.add_argument('--config', type=Path, default=Path(os.environ.get('XDG_CONFIG_HOME', Path.home() / '.config')) / 'kwinrulesrc')
    args = parser.parse_args()
    try:
        changed = configure(args.config, args.remove)
    except (OSError, ValueError, KeyError, configparser.Error, RuntimeError, subprocess.SubprocessError) as error:
        parser.exit(1, 'KDE IBus panel focus: ' + str(error) + '\n')
    print('KDE IBus panel focus: ' + ('removed' if args.remove else 'installed') if changed else 'KDE IBus panel focus: unchanged')


if __name__ == '__main__':
    main()
