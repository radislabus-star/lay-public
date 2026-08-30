#[test]
fn every_ime_space_route_finishes_the_full_precognition_boundary() {
    let managed = include_str!("../src/bin/lay_ibus_engine/managed.rs");
    let commit = include_str!("../src/bin/lay_ibus_engine/composition_commit.rs");
    let preedit = include_str!("../src/bin/lay_ibus_engine/preedit.rs");

    assert!(
        managed.contains("self.commit_managed_passthrough_char(emitter, ' ').await?")
            && managed.contains("self.push_tail_char(' ')")
            && managed.contains("self.commit_space(emitter).await?"),
        "every managed Space route must end in a shared boundary-owning operation"
    );
    assert!(
        commit.contains("self.push_tail_char(ch);")
            && commit.contains("self.close_precognition_word_boundary();"),
        "passthrough and active-composition Space routes must close the same boundary state"
    );
    assert!(
        preedit.contains(".hide_preedit_text()")
            && preedit.contains("pub(super) fn close_precognition_word_boundary")
            && preedit.contains("self.clear_preedit_completion_state();")
            && preedit.contains("self.composition.preedit_fast.reset();"),
        "the boundary pair must hide IBus UI and discard all candidate/tail state"
    );
}
