"""Negative controls for the native VNC receipt's settling boundary."""
import importlib.util
from pathlib import Path
import unittest

spec = importlib.util.spec_from_file_location(
    'vnc_smoke', Path(__file__).parent/'manual/remote_x11_vnc_smoke.py')
smoke = importlib.util.module_from_spec(spec)
spec.loader.exec_module(smoke)


class SettlingTest(unittest.TestCase):
    def observe(self, surface):
        tick = [0.0]
        def pump(seconds):
            tick[0] += seconds
        return smoke.wait_stable_target('target', lambda: {'visible': surface(tick[0])},
                                        pump, clock=lambda: tick[0], timeout=.4)

    def test_transient_match_then_reversion_fails(self):
        with self.assertRaisesRegex(AssertionError, 'reverted'):
            self.observe(lambda t: 'target' if .02 <= t < .04 else 'old')

    def test_late_extra_mutation_fails(self):
        with self.assertRaisesRegex(AssertionError, 'reverted'):
            self.observe(lambda t: 'target' if t < .12 else 'targetx')

    def test_waits_for_delayed_result_and_settles(self):
        actual, matched = self.observe(lambda t: 'target' if t >= .08 else 'old')
        self.assertEqual(actual['visible'], 'target')
        self.assertGreaterEqual(matched, .08)

    def test_absent_target_fails(self):
        with self.assertRaisesRegex(AssertionError, 'got'):
            self.observe(lambda t: 'old')
