pub fn warm_up() {
    warm_up_hot();
    crate::layout_autoswitch::warm_up();
    crate::russian_lexicon::warm_up();
}

pub fn warm_up_hot() {
    crate::lexicon::warm_up_for_ime();
    crate::typing_replacements::warm_up();
    crate::ngram::warm_up_ru();
}
