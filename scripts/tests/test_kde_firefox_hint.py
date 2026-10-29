"""Optional display style must not alter legacy projection or revision rules."""
import importlib.util
import json
from pathlib import Path
import unittest

ROOT = Path(__file__).resolve().parents[2]
spec = importlib.util.spec_from_file_location("lay_kde_tray", ROOT / "scripts/lay-kde-tray.py")
tray = importlib.util.module_from_spec(spec)
spec.loader.exec_module(tray)


class FirefoxSuffixStyleTests(unittest.TestCase):
    def test_optional_font_data_cannot_consume_revision_when_invalid(self):
        projection = tray.InlineHintProjection()
        projection.reset(":1.8")
        frame = dict(version=1, stream=":1.8", revision="1", owner="/field",
                     text="вет", rect=[100, 100, 0, 22], monospace=False)
        for style in ([], "font", 1, {"font_family": 12}, {"font_family": ""},
                      {"font_family": "bad\nfont"}, {"font_family": "x" * 129}):
            with self.subTest(style=style):
                self.assertEqual(projection.apply(json.dumps(dict(frame, suffix_style=style))), (False, None))
                self.assertEqual(projection.revision, -1)
        accepted, shown = projection.apply(json.dumps(dict(frame, suffix_style={"font_family": "Observed Font"})))
        self.assertTrue(accepted)
        self.assertEqual(shown["suffix_style"]["font_family"], "Observed Font")
        frame["revision"] = "2"
        accepted, shown = projection.apply(json.dumps(frame))
        self.assertTrue(accepted)
        self.assertNotIn("suffix_style", shown)


if __name__ == "__main__":
    unittest.main()
