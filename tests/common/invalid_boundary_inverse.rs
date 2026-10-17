//! Deliberately wrong action kind for recorded-boundary undo rejection.
//! This fixture is compiled only by the IME test module, never runtime.
pub fn non_boundary_action(original: &str, replacement: &str) -> lay::text_edit::EditAction {
    let plan = lay::text_edit::plan_text_replacement(original, replacement).unwrap();
    lay::text_edit::plan_ime_manual_toggle_edit(original, replacement, plan)
}
