import importlib.util
import json
from pathlib import Path
import unittest

ROOT = Path(__file__).resolve().parents[2]
spec = importlib.util.spec_from_file_location("lay_kde_tray", ROOT / "scripts/lay-kde-tray.py")
tray = importlib.util.module_from_spec(spec)
spec.loader.exec_module(tray)


class ProjectionTests(unittest.TestCase):
    def test_optional_outline_is_typed_and_cannot_consume_revision(self):
        projection = tray.InlineHintProjection()
        projection.reset(":1.8")
        frame = dict(version=1, stream=":1.8", revision="1", owner="/field",
                     text="вет", rect=[100, 100, 0, 22], monospace=False)
        self.assertTrue(projection.apply(json.dumps(frame))[0])
        frame["revision"] = "2"
        for invalid in (None, "true", 1, [], {}):
            with self.subTest(invalid=invalid):
                self.assertEqual(projection.apply(json.dumps(dict(frame, outlined=invalid))),
                                 (False, None))
                self.assertEqual(projection.revision, 1)
        accepted, shown = projection.apply(json.dumps(dict(frame, outlined=True)))
        self.assertTrue(accepted)
        self.assertEqual(shown["text"], "вет")
        self.assertTrue(shown["outlined"])
        frame["revision"] = "3"
        accepted, shown = projection.apply(json.dumps(dict(frame, outlined=False)))
        self.assertTrue(accepted)
        self.assertFalse(shown["outlined"])

    def test_shared_protocol_owner_restart_and_reordering(self):
        projection = tray.InlineHintProjection()
        cases = json.loads((ROOT / "scripts/tests/fixtures/inline-hint-frames-v1.json").read_text())
        for index, case in enumerate(cases):
            with self.subTest(index=index):
                if "owner" in case:
                    projection.reset(case["owner"])
                    continue
                accepted, frame = projection.apply(json.dumps(case["frame"], ensure_ascii=False))
                self.assertEqual(accepted, case["accepted"])
                if accepted:
                    self.assertEqual(frame is not None, case["visible"])
                    if frame is not None:
                        self.assertEqual(frame["text"], case["frame"]["text"])

    def test_invalid_data_cannot_consume_revision_or_mutate_view(self):
        projection = tray.InlineHintProjection()
        projection.reset(":1.8")
        for raw in ("null", "[]", "{", "x" * 16385,
                    json.dumps({"version": 1, "stream": ":1.8", "revision": "9" * 21})):
            self.assertEqual(projection.apply(raw), (False, None))
        self.assertEqual(projection.revision, -1)


if __name__ == "__main__":
    unittest.main()
