import importlib.util
from pathlib import Path
import unittest


PATH = Path(__file__).resolve().parents[2] / "tests/manual/browser_ime_native_surfaces.py"
SPEC = importlib.util.spec_from_file_location("browser_ime_native_surfaces", PATH)
MODULE = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(MODULE)
matches_typed_seed = MODULE.matches_typed_seed
native_dom_text_state = MODULE.native_dom_text_state


class NativeSeedTests(unittest.TestCase):
    def test_plain_exact_seed(self):
        self.assertTrue(matches_typed_seed("тест, привет", 12, 0, "тест, привет"))

    def test_empty_seed(self):
        self.assertTrue(matches_typed_seed("", 0, 0, ""))

    def test_rich_typed_space_can_be_nbsp(self):
        self.assertTrue(matches_typed_seed("тест,\u00a0привет", 12, 0, "тест, привет", rich_text=True))

    def test_multiple_spaces_keep_their_positions(self):
        for actual in ("а б в", "а\u00a0б в", "а б\u00a0в", "а\u00a0б\u00a0в"):
            with self.subTest(actual=actual):
                self.assertTrue(matches_typed_seed(actual, 5, 0, "а б в", rich_text=True))

    def test_plain_field_does_not_allow_nbsp(self):
        self.assertFalse(matches_typed_seed("тест\u00a0привет", 11, 0, "тест привет"))

    def test_expected_nbsp_remains_exact(self):
        self.assertTrue(matches_typed_seed("а\u00a0б", 3, 0, "а\u00a0б", rich_text=True))
        self.assertFalse(matches_typed_seed("а б", 3, 0, "а\u00a0б", rich_text=True))

    def test_other_whitespace_is_not_a_typed_space(self):
        for separator in ("\t", "\n", "\r", "\u2009", "\u202f", "\u200b"):
            with self.subTest(separator=repr(separator)):
                self.assertFalse(matches_typed_seed("а" + separator + "б", 3, 0, "а б", rich_text=True))

    def test_changed_text_or_separator_is_rejected(self):
        for actual in ("тест привет", "Тест, привет", "тест; привет", "тест,\u00a0превет",
                       "тест,  привет", "тест,привет", "тест, приветx", "тест, привет\n"):
            with self.subTest(actual=actual):
                self.assertFalse(matches_typed_seed(actual, len(actual), 0, "тест, привет", rich_text=True))

    def test_interior_caret_or_hidden_suffix_is_rejected(self):
        self.assertFalse(matches_typed_seed("тест привет", 5, 0, "тест привет", rich_text=True))
        self.assertFalse(matches_typed_seed("приветx", 6, 0, "привет", rich_text=True))

    def test_any_selection_is_rejected(self):
        for selections in (-1, 1, 2):
            with self.subTest(selections=selections):
                self.assertFalse(matches_typed_seed("а б", 3, selections, "а б", rich_text=True))

    def test_malformed_snapshot_is_rejected(self):
        for raw, caret, selections, expected, rich in (
            (None, 0, 0, "", True), ("", 0, 0, None, True),
            ("", False, 0, "", True), ("", 0, False, "", True),
            ("", 0, 0, "", 1), ("а б", "3", 0, "а б", True),
            ("а б", 4, 0, "а б", True), ("а б", -1, 0, "а б", True),
        ):
            with self.subTest(values=(raw, caret, selections, expected, rich)):
                self.assertFalse(matches_typed_seed(raw, caret, selections, expected, rich_text=rich))


class NativeDomStateTests(unittest.TestCase):
    def snapshot(self):
        return dict(url="file:///fixture?browser=test", field_id="search",
                    active_id="search", type="search", value="привет",
                    selection_start=6, selection_end=6,
                    selection_direction="none", document_focused=True)

    def observe(self, snapshot):
        return native_dom_text_state(
            snapshot, expected_url="file:///fixture?browser=test",
            expected_field="search", expected_type="search")

    def changed(self, **changes):
        return self.snapshot() | changes

    def test_complete_raw_state(self):
        state = self.observe(self.snapshot())
        self.assertEqual((state["committed"], state["caret"], state["selection"]),
                         ("привет", 6, False))

    def test_invalid_expected_identity_is_rejected(self):
        for field in ("", None, 1):
            with self.subTest(field=field), self.assertRaises(ValueError):
                native_dom_text_state(self.snapshot(), expected_url="file:///fixture?browser=test",
                                      expected_field=field, expected_type="search")

    def test_empty_field(self):
        self.assertEqual(self.observe(self.changed(value="", selection_start=0,
                                                   selection_end=0))["committed"], "")

    def test_wrong_identity_or_focus_is_rejected(self):
        for key, value in (("url", "file:///other"), ("field_id", "text"),
                           ("active_id", "text"), ("type", "text"),
                           ("document_focused", False), ("document_focused", 1)):
            with self.subTest(key=key, value=value), self.assertRaises(ValueError):
                self.observe(self.changed(**{key: value}))

    def test_missing_fields_and_malformed_snapshots_are_rejected(self):
        for key in self.snapshot():
            snapshot = self.snapshot()
            del snapshot[key]
            with self.subTest(missing=key), self.assertRaises(ValueError):
                self.observe(snapshot)
        for snapshot in (None, [], "привет", True):
            with self.subTest(snapshot=snapshot), self.assertRaises(ValueError):
                self.observe(snapshot)

    def test_invalid_selection_is_rejected(self):
        for start, end, direction in ((-1, 6, "none"), (0, 7, "forward"),
                                      (4, 3, "backward"), (True, 6, "none"),
                                      (0, None, "none"), (0, 6, "unknown")):
            with self.subTest(values=(start, end, direction)), self.assertRaises(ValueError):
                self.observe(self.changed(selection_start=start, selection_end=end,
                                          selection_direction=direction))

    def test_selection_and_interior_caret_remain_visible(self):
        for direction, caret in (("forward", 5), ("backward", 1)):
            state = self.observe(self.changed(selection_start=1, selection_end=5,
                                              selection_direction=direction))
            self.assertEqual((state["caret"], state["selection"], state["selection_start"],
                              state["selection_end"]), (caret, True, 1, 5))
        self.assertEqual(self.observe(self.changed(selection_start=3, selection_end=3))["caret"], 3)

    def test_no_text_normalization_or_expected_word_substitution(self):
        for value in ("приве", "Привет", "превет", "приветx", "а\u00a0б"):
            state = self.observe(self.changed(value=value, selection_start=len(value),
                                              selection_end=len(value)))
            self.assertEqual(state["committed"], value)
            self.assertFalse(matches_typed_seed(state["committed"], state["caret"], 0, "привет"))

    def test_utf16_offsets_are_converted_without_changing_text(self):
        state = self.observe(self.changed(value="а😀б", selection_start=4, selection_end=4))
        self.assertEqual((state["committed"], state["caret"]), ("а😀б", 3))
        with self.assertRaises(ValueError):
            self.observe(self.changed(value="а😀б", selection_start=2, selection_end=2))

    def test_unbounded_or_invalid_text_is_rejected(self):
        for value in (None, "а" * 2049, "а\ud800б"):
            with self.subTest(length=len(value) if value is not None else None), self.assertRaises(ValueError):
                self.observe(self.changed(value=value))


if __name__ == "__main__":
    unittest.main()
