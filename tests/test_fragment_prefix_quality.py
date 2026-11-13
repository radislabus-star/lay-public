"""Identity-level safeguards for the offline streaming proof comparator."""
import importlib.util
from pathlib import Path
import unittest

SPEC = importlib.util.spec_from_file_location(
    "fragment_prefix_quality",
    Path(__file__).resolve().parents[1] / "scripts/proof/fragment-prefix-quality.py",
)
PROOF = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(PROOF)


class FragmentPrefixQualityTests(unittest.TestCase):
    def test_new_reversal_from_zero_is_detected_even_when_final_text_is_correct(self):
        self.assertEqual(
            PROOF.transition_regressions(
                {"flips": 0, "actual": "word"}, {"flips": 2, "actual": "word"}, "word"
            ),
            {"new_repeated_flips"},
        )

    def test_new_reversal_after_one_or_more_flips_is_detected(self):
        for previous in (1, 2):
            with self.subTest(previous=previous):
                self.assertIn(
                    "new_repeated_flips",
                    PROOF.transition_regressions(
                        {"flips": previous, "actual": "wrong"},
                        {"flips": previous + 1, "actual": "wrong"}, "word",
                    ),
                )

    def test_improvement_of_another_word_cannot_hide_a_new_wrong_flip(self):
        fixed = PROOF.transition_regressions(
            {"flips": 1, "actual": "wrong"}, {"flips": 0, "actual": "first"}, "first"
        )
        broken = PROOF.transition_regressions(
            {"flips": 0, "actual": "second"}, {"flips": 1, "actual": "wrong"}, "second"
        )
        self.assertEqual(fixed, set())
        self.assertEqual(broken, {"new_wrong_after_flip"})

    def test_new_correct_single_flip_is_an_improvement(self):
        self.assertEqual(
            PROOF.transition_regressions(
                {"flips": 0, "actual": "wrong"}, {"flips": 1, "actual": "word"}, "word"
            ),
            set(),
        )


if __name__ == "__main__":
    unittest.main()
