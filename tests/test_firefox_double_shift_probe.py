"""Source checks of the manual observer; no browser or production IME launch."""
from html.parser import HTMLParser
from pathlib import Path
import subprocess
import re
import unittest

ROOT = Path(__file__).resolve().parents[1]
PAGE = ROOT / "tests/manual/firefox_double_shift.html"
PROOF = ROOT / "tests/manual/firefox_double_shift.test.cjs"


class Fields(HTMLParser):
    def __init__(self):
        super().__init__()
        self.fields = []
        self.elements = {}

    def handle_starttag(self, tag, attrs):
        record = dict(attrs)
        if "data-field" in record:
            self.fields.append(record["data-field"])
        if "id" in record:
            if record["id"] in self.elements:
                raise ValueError("Duplicate editor ID: " + record["id"])
            self.elements[record["id"]] = (tag, record)


class FirefoxDoubleShiftProbeTests(unittest.TestCase):
    def test_seven_stage_observer_contract(self):
        result = subprocess.run(["node", "--test", "--test-reporter=tap", str(PROOF)], cwd=ROOT,
                                capture_output=True, text=True, timeout=30)
        self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
        counts = dict(re.findall(r"^# (tests|pass|fail|skipped) (\d+)$", result.stdout, re.M))
        self.assertEqual(counts, {"tests": "15", "pass": "15", "fail": "0", "skipped": "0"}, result.stdout)

    def test_page_has_all_ten_expected_field_cards(self):
        parser = Fields()
        parser.feed(PAGE.read_text())
        self.assertEqual(parser.fields, ["text", "search", "url", "tel", "email",
                                        "textarea", "rich", "plain", "iframe-input", "iframe-rich"])
        for field in ["text", "search", "url", "tel", "email"]:
            tag, attrs = parser.elements[field]
            self.assertEqual((tag, attrs.get("type")), ("input", field))
        self.assertEqual(parser.elements["textarea"][0], "textarea")
        for field, mode in [("rich", "true"), ("plain", "plaintext-only")]:
            tag, attrs = parser.elements[field]
            self.assertEqual((tag, attrs.get("contenteditable")), ("div", mode))
        for field, expected in [("iframe-input", ("input", "text", None)),
                                ("iframe-rich", ("div", None, "true"))]:
            tag, attrs = parser.elements[field]
            self.assertEqual(tag, "iframe")
            nested = Fields()
            nested.feed(attrs["srcdoc"])
            editor_tag, editor_attrs = nested.elements["field"]
            self.assertEqual((editor_tag, editor_attrs.get("type"), editor_attrs.get("contenteditable")), expected)


if __name__ == "__main__":
    unittest.main()
