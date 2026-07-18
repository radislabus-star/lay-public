#[test]
fn every_ime_space_route_finishes_the_full_precognition_boundary() {
    let managed = include_str!("../src/bin/lay_ibus_engine/managed.rs");
    let commit = include_str!("../src/bin/lay_ibus_engine/composition_commit.rs");
    let preedit = include_str!("../src/bin/lay_ibus_engine/preedit.rs");

    assert!(
        managed.contains("self.clear_preedit(emitter).await?")
            && managed.contains("self.close_precognition_word_boundary();")
            && commit.contains("self.close_precognition_word_boundary();"),
        "empty and active-composition Space routes must close the same preedit state"
    );
    assert!(
        preedit.contains("Self::hide_preedit_text(emitter)")
            && preedit.contains("pub(super) fn close_precognition_word_boundary")
            && preedit.contains("self.clear_preedit_completion_state();")
            && preedit.contains("self.preedit_fast.reset();"),
        "the boundary pair must hide IBus UI and discard all candidate/tail state"
    );
}
