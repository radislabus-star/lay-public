use std::sync::Once;

use lay::typing_assist::remember_promoted_replacement;

use super::fixtures::fixture_rows;
use super::harness::apply_typing_assist_to_text_tail_with;

pub(super) fn select_typing_assist_exact(text: &str) -> Option<String> {
    lay::lem::set_runtime_enabled(true);
    seed_test_replacements();
    lay::typing_assist::select_typing_assist_exact(text)
}

pub(super) fn select_typing_assist(text: &str, allow_layout_auto: bool) -> Option<String> {
    lay::lem::set_runtime_enabled(true);
    seed_test_replacements();
    lay::typing_assist::select_typing_assist(text, allow_layout_auto)
}

pub(super) fn apply_auto_replace(original: &str, target: &str) -> Option<String> {
    lay::lem::set_runtime_enabled(true);
    seed_test_replacements();
    lay::typing_assist::apply_auto_replace(original, target)
}

pub(super) fn apply_typing_assist_to_text_tail(text: &str) -> Option<String> {
    apply_typing_assist_to_text_tail_with(text, select_typing_assist_exact)
}

fn seed_test_replacements() {
    static ONCE: Once = Once::new();
    ONCE.call_once(|| {
        for row in fixture_rows("daemon_seed_replacements.tsv") {
            assert_eq!(row.len(), 2, "seed replacement fixture must be TSV");
            remember_promoted_replacement(&row[0], &row[1]);
        }
    });
}
