import importlib.util
import json
from pathlib import Path
import unittest

ROOT = Path(__file__).resolve().parents[2]
spec = importlib.util.spec_from_file_location("lay_kde_tray", ROOT / "scripts/lay-kde-tray.py")
tray = importlib.util.module_from_spec(spec)
spec.loader.exec_module(tray)


class ProjectionTests(unittest.TestCase):
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
