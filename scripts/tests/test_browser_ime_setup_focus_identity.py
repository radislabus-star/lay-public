import importlib.util
from pathlib import Path
import unittest


PATH = Path(__file__).resolve().parents[2] / "tests/manual/browser_ime_setup_focus_identity.py"
SPEC = importlib.util.spec_from_file_location("browser_ime_setup_focus_identity", PATH)
MODULE = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(MODULE)
same_identity = MODULE.same_engine_focus_identity


class FocusIdentityTests(unittest.TestCase):
    def setUp(self):
        self.focus = {"client": "gnome-shell", "engine_object_path": "/engine/ru/1", "wire_header": "serial=35 time=1"}

    def test_repeated_same_focus_with_new_packet_header(self):
        fresh = dict(self.focus, wire_header="serial=47 time=2")
        self.assertTrue(same_identity(self.focus, fresh))

    def test_changed_engine_path_is_refused(self):
        self.assertFalse(same_identity(self.focus, dict(self.focus, engine_object_path="/engine/ru/2")))

    def test_changed_client_is_refused(self):
        self.assertFalse(same_identity(self.focus, dict(self.focus, client="other-client")))

    def test_missing_observation_or_identity_is_refused(self):
        for bad in (None, {}, {"client": "gnome-shell"}, {"client": "", "engine_object_path": "/engine/ru/1"}):
            self.assertFalse(same_identity(self.focus, bad))
            self.assertFalse(same_identity(bad, self.focus))

    def test_malformed_identity_is_refused(self):
        self.assertFalse(same_identity(self.focus, dict(self.focus, client=["gnome-shell"])))


if __name__ == "__main__":
    unittest.main()
