"""Read-only feedback observation for one private IBus inverse case.

This module never grants removal or recovery authority for learning files.
The connected driver supplies observed input/apply and an exclusive private
context; journal episode IDs alone do not identify a native window.
"""

from dataclasses import dataclass
import hashlib
import json
import re


class CaseMismatch(RuntimeError):
    """Missing or contradictory evidence must stop dependent proof cases."""


def typed_input_surface(records: list, context: str) -> str:
    """Separate real text-key presses from fixture readiness/control events."""
    characters = []
    for row in records:
        if not isinstance(row, dict):
            raise CaseMismatch('input record is not an object')
        if (row.get('kind') != 'ProcessKeyEvent' or row.get('context') != context
                or row.get('phase') != 'press'):
            continue
        character = row.get('character')
        if not isinstance(character, str) or len(character) != 1:
            raise CaseMismatch('text-key press has no exact character')
        characters.append(character)
    return ''.join(characters)


@dataclass(frozen=True)
class InverseCase:
    name: str
    typed: str
    applied: str

    def validate_edit(self, rows: list, *, context: str, prefix: str,
                      visible: str, cursor: int, inverse: bool = False) -> None:
        """Check requested geometry as well as the client's clamped projection."""
        source, target = (self.applied, self.typed) if inverse else (self.typed, self.applied)
        if not source.startswith(prefix) or not target.startswith(prefix):
            raise CaseMismatch('edit prefix differs from the immutable case')
        if [row.get('kind') for row in rows] != ['DeleteSurroundingText', 'CommitText']:
            raise CaseMismatch('edit is not exactly one ordered delete/commit frame')
        deletion, commit = rows
        suffix = source[len(prefix):]
        actual_delete = tuple(deletion.get(key) for key in
                              ('context', 'offset', 'count', 'deleted', 'visible', 'cursor'))
        if actual_delete != (context, -len(suffix), len(suffix), suffix, prefix, len(prefix)):
            raise CaseMismatch('delete differs from the exact case geometry')
        actual_commit = tuple(commit.get(key) for key in ('context', 'text', 'visible', 'cursor'))
        if actual_commit != (context, target[len(prefix):], target, len(target)):
            raise CaseMismatch('commit differs from the exact case target')
        if visible != target or cursor != len(target):
            raise CaseMismatch('final client surface/cursor differs from the exact case target')

    @property
    def changed_words(self) -> frozenset[str]:
        original, target = self.typed.split(), self.applied.split()
        if len(original) != len(target):
            raise CaseMismatch('inverse case changes token count')
        words = [b for a, b in zip(original, target) if a != b]
        if not words or len(set(words)) != len(words):
            raise CaseMismatch('inverse case has no unique changed targets')
        return frozenset(words)

    def bind(self, *, observed_input: str, observed_apply: str, context: str,
             pid: int, starttick: str, candidate_sha256: str,
             began_unix_ns: int, journal_before: bytes) -> 'BoundInverse':
        if observed_input != self.typed or observed_apply != self.applied:
            raise CaseMismatch('observed case differs from the immutable input/apply specification')
        if not context.startswith('/org/freedesktop/IBus/InputContext_'):
            raise CaseMismatch('missing real private IBus context')
        if type(pid) is not int or pid <= 0 or not starttick.isdecimal():
            raise CaseMismatch('missing candidate process identity')
        if not re.fullmatch(r'[a-f0-9]{64}', candidate_sha256):
            raise CaseMismatch('missing exact candidate hash')
        if type(began_unix_ns) is not int or began_unix_ns <= 0:
            raise CaseMismatch('missing inverse interval')
        self.changed_words
        return BoundInverse(self, context, pid, starttick, candidate_sha256,
                            began_unix_ns, journal_before)


@dataclass(frozen=True)
class BoundInverse:
    case: InverseCase
    context: str
    pid: int
    starttick: str
    candidate_sha256: str
    began_unix_ns: int
    journal_before: bytes

    def observe(self, current: bytes, ended_unix_ns: int) -> dict:
        if type(ended_unix_ns) is not int or ended_unix_ns < self.began_unix_ns:
            raise CaseMismatch('inverse interval reversed')
        retained = _retained_suffix(self.journal_before, current)
        delta = current[len(retained):]
        if delta and not delta.endswith(b'\n'):
            raise CaseMismatch('partial journal record')
        selected = []
        try:
            records = [json.loads(line) for line in delta.splitlines()]
        except (ValueError, UnicodeError) as error:
            raise CaseMismatch('invalid journal record') from error
        for row in records:
            if not isinstance(row, dict):
                raise CaseMismatch('journal record is not an object')
            if row.get('kind') != 'rejected_candidate' or row.get('outcome') != 'reverted':
                continue
            episode = row.get('episode_id')
            if not isinstance(episode, str) or not episode.startswith(str(self.pid) + '-'):
                continue
            identity = re.fullmatch(r'\d+-(\d+)-\d+', episode)
            if identity is None:
                raise CaseMismatch('own rejection has an invalid episode identity')
            if not self.began_unix_ns // 1000 <= int(identity[1]) <= ended_unix_ns // 1000:
                continue
            timestamp = row.get('ts')
            if type(timestamp) is not int:
                raise CaseMismatch('own rejection has no timestamp')
            if not self.began_unix_ns // 10**9 <= timestamp <= ended_unix_ns // 10**9:
                continue
            if (row.get('from') != self.case.typed.strip()
                    or row.get('to') != self.case.applied.strip()):
                raise CaseMismatch('own rejection belongs to a different pair')
            if row.get('word') not in self.case.changed_words:
                raise CaseMismatch('own rejection has an unexpected changed target')
            selected.append(row)
        expected = self.case.changed_words
        if len(selected) > len(expected):
            raise CaseMismatch('extra or duplicate own rejection')
        if len({r['episode_id'] for r in selected}) > 1:
            raise CaseMismatch('mixed own rejection episodes')
        if len({r['word'] for r in selected}) != len(selected):
            raise CaseMismatch('duplicate own changed target')
        complete = len(selected) == len(expected)
        return {
            'status': 'MATCHED_CASE_FEEDBACK' if complete else 'PENDING_CASE_FEEDBACK',
            'selected_rows': len(selected),
            'expected_rows': len(expected),
            'episode_id': selected[0]['episode_id'] if selected else None,
            'began_unix_ns': self.began_unix_ns,
            'ended_unix_ns': ended_unix_ns,
            'journal_sha256': hashlib.sha256(current).hexdigest(),
            'journal_before_sha256': hashlib.sha256(self.journal_before).hexdigest(),
            'retained_old_suffix_bytes': len(retained),
            'retained_old_suffix_byte_identical': True,
            'journal_writes': 0,
            'cleanup_authority': False,
        }


def _retained_suffix(before: bytes, current: bytes) -> bytes:
    if current.startswith(before):
        return before
    if not current or len(current) > 500 * 1024 or not before.endswith(b'\n'):
        raise CaseMismatch('missing or unsettled retained journal generation')
    first = current.splitlines(keepends=True)[0]
    matches, offset = [], 0
    for line in before.splitlines(keepends=True):
        if line == first and current.startswith(before[offset:]):
            matches.append(offset)
        offset += len(line)
    if len(matches) != 1:
        raise CaseMismatch('rewritten or ambiguous retained journal generation')
    return before[matches[0]:]
