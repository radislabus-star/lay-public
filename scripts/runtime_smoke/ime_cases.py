from __future__ import annotations


def make_ime_cases(Case):
    immediate_space_expected = " ".join(
        ["да норм"] * 10 + ["мне сбросили"] * 5 + ["Елена просит"] * 5
    )
    return {
        "ime_worked_nj_space_enter": Case("ime_worked_nj_space_enter", "worked это"),
        "ime_sleduyuschiy_next_enter": Case(
            "ime_sleduyuschiy_next_enter", "следующий слово", start_layout="ru"
        ),
        "ime_short_alt_all_wrong_enter": Case(
            "ime_short_alt_all_wrong_enter",
            "я git и api мы css ты cpu он gpu в html к json с llm не log на md ну pdf "
            "по ram за sql вот ssh это vpn да go как in дом up мир to код on тут off "
            "там file для test при push ход bash",
        ),
        "ime_file_ghjdthrf_shift_enter": Case(
            "ime_file_ghjdthrf_shift_enter", "file проверка"
        ),
        "ime_file_ghjdthrf_shift_immediate_a_enter": Case(
            "ime_file_ghjdthrf_shift_immediate_a_enter",
            "file проверкаа",
            expected_manual_toggles=1,
        ),
        "ime_file_ghjdthrf_shift_twice_enter": Case(
            "ime_file_ghjdthrf_shift_twice_enter", "file ghjdthrf"
        ),
        "ime_file_ghjdthrf_alt_enter": Case("ime_file_ghjdthrf_alt_enter", "file проверка"),
        "ime_file_ghjdthrf_alt_twice_enter": Case(
            "ime_file_ghjdthrf_alt_twice_enter", "file проверка"
        ),
        "ime_ntrcn_space_shift_enter": Case("ime_ntrcn_space_shift_enter", "ntrcn"),
        "ime_raw_ghjdthrf_right_shift_enter": Case(
            "ime_raw_ghjdthrf_right_shift_enter", "проверка"
        ),
        "ime_raw_ghjdthrf_alt_enter": Case("ime_raw_ghjdthrf_alt_enter", "проверка"),
        "ime_prefix_proverka_ghjdthrf_shift_enter": Case(
            "ime_prefix_proverka_ghjdthrf_shift_enter", "проверка проверка"
        ),
        "ime_prefix_proverka_ghjdthrf_alt_enter": Case(
            "ime_prefix_proverka_ghjdthrf_alt_enter", "проверка проверка"
        ),
        "ime_ctrl_a_passthrough_enter": Case(
            "ime_ctrl_a_passthrough_enter", "ч", start_layout="ru"
        ),
        "ime_ctrl_l_passthrough_enter": Case(
            "ime_ctrl_l_passthrough_enter", "фисч", start_layout="ru"
        ),
        "ime_quote_nj_space_enter": Case("ime_quote_nj_space_enter", "это"),
        "ime_lfdfq_space_enter": Case("ime_lfdfq_space_enter", "давай"),
        "ime_autocorrect_en_ru_double_shift_back_layout_enter": Case(
            "ime_autocorrect_en_ru_double_shift_back_layout_enter", "djn file"
        ),
        "ime_autocorrect_typo_double_shift_undo_enter": Case(
            "ime_autocorrect_typo_double_shift_undo_enter",
            "доллора",
            start_layout="ru",
        ),
        "ime_known_inflection_preserved_enter": Case(
            "ime_known_inflection_preserved_enter",
            "в коде",
            start_layout="ru",
        ),
        "ime_prefix_prov_completion_alt_enter": Case(
            "ime_prefix_prov_completion_alt_enter", "проверка", start_layout="ru"
        ),
        "ime_autocomplete_then_double_shift_enter": Case(
            "ime_autocomplete_then_double_shift_enter",
            "ghjdthrf",
            start_layout="ru",
            expected_manual_toggles=1,
        ),
        "ime_alt_left_right_passthrough_enter": Case(
            "ime_alt_left_right_passthrough_enter", "фисч", start_layout="ru"
        ),
        "ime_cursor_backspace_inside_composition_enter": Case(
            "ime_cursor_backspace_inside_composition_enter", "фичв", start_layout="ru"
        ),
        "ime_backspace_after_shift_commit_enter": Case(
            "ime_backspace_after_shift_commit_enter", "проверк"
        ),
        "ime_immediate_space_warmup": Case(
            "ime_immediate_space_warmup",
            "да норм мне сбросили Елена просит коде",
            start_layout="ru",
        ),
        "ime_immediate_space_replay": Case(
            "ime_immediate_space_replay",
            immediate_space_expected,
            start_layout="ru",
        ),
    }
