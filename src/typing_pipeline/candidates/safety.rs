use crate::typing_rule_graph::ids;
use crate::word_reader::split_ws_segments;

pub(super) fn unsafe_word_count_shrink(original: &str, replacement: &str, rule_id: &str) -> bool {
    if matches!(
        rule_id,
        ids::SPLIT_WORD_PAIR | ids::GLUED_PHRASE | ids::MOVED_PREFIX_PAIR
    ) {
        return false;
    }
    let original_words = split_ws_segments(original)
        .into_iter()
        .filter(|(_, is_ws)| !*is_ws)
        .count();
    let replacement_words = split_ws_segments(replacement)
        .into_iter()
        .filter(|(_, is_ws)| !*is_ws)
        .count();
    original_words >= 2 && replacement_words < original_words
}
