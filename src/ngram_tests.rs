use super::*;

fn ru_test_model() -> CharNgramModel {
    CharNgramModel::train(
        Lang::Ru,
        [
            "привет",
            "проверка",
            "работает",
            "ошибка",
            "ошибся",
            "явно",
            "ладно",
            "можно",
            "дальше",
            "плохо",
            "правильно",
            "исправлено",
            "исправляет",
            "текст",
            "слово",
        ],
    )
}

#[test]
fn scores_good_word_above_transposed_typo() {
    let model = ru_test_model();
    assert!(
        model.score_text("работает") > model.score_text("рабоатет"),
        "работает={} рабоатет={}",
        model.score_text("работает"),
        model.score_text("рабоатет")
    );
}

#[test]
fn scores_common_word_above_rare_transposition() {
    let model = ru_test_model();
    assert!(
        model.score_text("ладно") > model.score_text("ландо"),
        "ладно={} ландо={}",
        model.score_text("ладно"),
        model.score_text("ландо")
    );
}

#[test]
fn scores_merged_word_above_accidental_split() {
    let model = ru_test_model();
    assert!(
        model.score_text("явно") > model.score_text("я вно"),
        "явно={} я вно={}",
        model.score_text("явно"),
        model.score_text("я вно")
    );
}

#[test]
fn global_ru_model_can_rank_local_words() {
    assert!(ru_candidate_is_better("правильно", "првильно", 0.0));
    assert!(ru_candidate_margin("исправлено", "исправленно") > -0.50);
    assert!(ru_candidate_margin("явно", "я вно") > -1.00);
    assert!(ru_candidate_margin("плохо", "плозо") > -0.50);
}
