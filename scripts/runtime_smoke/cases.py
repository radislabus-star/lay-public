from __future__ import annotations

import dataclasses

from runtime_smoke.ime_cases import make_ime_cases


SHORT_ALT_50_EXPECTED = (
    "я git и api мы css ты cpu он gpu в html к json с llm не log на md ну pdf "
    "по ram за sql вот ssh это vpn да go как in дом up мир to код on тут off "
    "там file для test при push ход bash"
)


@dataclasses.dataclass(frozen=True)
class Case:
    name: str
    expected: str
    start_layout: str = "us"
    config_overrides: dict[str, object] | None = None
    expected_manual_toggles: int | None = None
    expected_preedit_updates: tuple[str, ...] | None = None
    expected_managed_commits: tuple[str, ...] | None = None
    expected_pending_shortens: int | None = None
    expected_completion_accepts: int | None = None
    minimum_ibus_keys: int = 0
    minimum_preedit_clears: int = 0


def normal_autocorrect_config() -> dict[str, object]:
    return {
        "auto_replace": True,
        "typing_assist": True,
        "correction_safety": "normal",
    }


def nanda_experimental_config() -> dict[str, object]:
    return {
        "auto_replace": True,
        "typing_assist": True,
        "correction_safety": "experimental",
        "auto_switch_layout": True,
        "typing_assist_words": 2,
    }


CASES = {
    "ghbdtn_enter": Case("ghbdtn_enter", "привет"),
    "ghbdtn_enter_autocorrect": Case(
        "ghbdtn_enter_autocorrect",
        "ghbdtn",
        config_overrides={"enter_autocorrect": True},
    ),
    "ghbdtn_fast_lshift_enter": Case(
        "ghbdtn_fast_lshift_enter", "привет", expected_manual_toggles=1
    ),
    "ghbdtn_long_lshift_enter": Case("ghbdtn_long_lshift_enter", "привет"),
    "ghbdtn_extra_lshift_enter": Case(
        "ghbdtn_extra_lshift_enter", "ghbdtn", expected_manual_toggles=2
    ),
    "ctrl_plus_ghbdtn_enter": Case("ctrl_plus_ghbdtn_enter", "привет"),
    "dhtvz_toggle_enter": Case("dhtvz_toggle_enter", "dhtvz"),
    "dhtvz_toggle3_enter": Case("dhtvz_toggle3_enter", "время"),
    "g_to_ru_enter": Case("g_to_ru_enter", "п"),
    "eng_ru_to_us_enter": Case("eng_ru_to_us_enter", "eng", start_layout="ru"),
    "plain_layout_ashdu_space_enter": Case(
        "plain_layout_ashdu_space_enter",
        "file",
        start_layout="ru",
        config_overrides=normal_autocorrect_config(),
    ),
    "plain_layout_cargo_space_enter": Case(
        "plain_layout_cargo_space_enter",
        "cargo",
        start_layout="ru",
        config_overrides=normal_autocorrect_config(),
    ),
    "plain_layout_abkt_space_enter": Case(
        "plain_layout_abkt_space_enter",
        "abkt",
        start_layout="us",
        config_overrides=normal_autocorrect_config(),
    ),
    "good_toggle4_enter": Case("good_toggle4_enter", "good"),
    "good_ntrcn_enter": Case("good_ntrcn_enter", "good текст"),
    "good_text_enter": Case("good_text_enter", "good текст", start_layout="ru"),
    "good_vshgidu_enter": Case("good_vshgidu_enter", "good Double"),
    "mixed_word": Case("mixed_word", "при"),
    "mixed_coke_enter": Case("mixed_coke_enter", "слово кока-колу", start_layout="ru"),
    "mixed_coke_toggle3_enter": Case(
        "mixed_coke_toggle3_enter", "слово кока-колу", start_layout="ru"
    ),
    "n_teper_mixed_enter": Case("n_teper_mixed_enter", "Теперь"),
    "auto_switch_words_enter": Case("auto_switch_words_enter", "njkmrj yt hf,jnftn"),
    "worked_nj_space_enter": Case(
        "worked_nj_space_enter",
        "worked это",
        config_overrides=normal_autocorrect_config(),
    ),
    "html_djn_spacing_enter": Case(
        "html_djn_spacing_enter",
        "html вот",
        start_layout="ru",
        config_overrides=nanda_experimental_config(),
    ),
    "no_ne_ty_enter": Case("no_ne_ty_enter", "но не ты", start_layout="ru"),
    "preparatov_typo_enter": Case(
        "preparatov_typo_enter", "препаратов", start_layout="ru"
    ),
    "proverka_ntrcn_enter": Case(
        "proverka_ntrcn_enter", "проверка текст", start_layout="ru"
    ),
    "glued_toesamoe_next_enter": Case(
        "glued_toesamoe_next_enter", "тоже самое склено", start_layout="ru"
    ),
    "glued_tozhesamoe_next_enter": Case(
        "glued_tozhesamoe_next_enter", "тоже самое склено", start_layout="ru"
    ),
    "glued_yanebudu_next_enter": Case(
        "glued_yanebudu_next_enter", "я не буду склено", start_layout="ru"
    ),
    "glued_context_yanebudu_next_enter": Case(
        "glued_context_yanebudu_next_enter",
        "тоже самое я не буду склено",
        start_layout="ru",
    ),
    "glued_long_phrase_next_enter": Case(
        "glued_long_phrase_next_enter",
        "я не буду за вас тоже самое склено",
        start_layout="ru",
    ),
    "ru_p_enter": Case("ru_p_enter", "п", start_layout="ru"),
    "ru_p_to_g_enter": Case("ru_p_to_g_enter", "g", start_layout="ru"),
    "ru_p_toggle2_enter": Case("ru_p_toggle2_enter", "п", start_layout="ru"),
    "slovo_ru_to_us_fast_lshift_enter": Case(
        "slovo_ru_to_us_fast_lshift_enter",
        "ckjdj",
        start_layout="ru",
        expected_manual_toggles=1,
    ),
    "slovo_ru_to_us_extra_lshift_enter": Case(
        "slovo_ru_to_us_extra_lshift_enter",
        "слово",
        start_layout="ru",
        expected_manual_toggles=2,
    ),
    "vyvodim_dva_enter": Case("vyvodim_dva_enter", "выводим два"),
    "wifi_ye_enter": Case("wifi_ye_enter", "wi-fi ну"),
    "short_alt_ru_wrong_us_enter": Case(
        "short_alt_ru_wrong_us_enter",
        SHORT_ALT_50_EXPECTED,
        config_overrides=nanda_experimental_config(),
    ),
    "short_alt_en_wrong_ru_enter": Case(
        "short_alt_en_wrong_ru_enter",
        SHORT_ALT_50_EXPECTED,
        start_layout="ru",
        config_overrides=nanda_experimental_config(),
    ),
    "short_alt_all_wrong_enter": Case(
        "short_alt_all_wrong_enter",
        SHORT_ALT_50_EXPECTED,
        config_overrides={**nanda_experimental_config(), "text_backend": "ime"},
    ),
}

CASES.update(make_ime_cases(Case))
