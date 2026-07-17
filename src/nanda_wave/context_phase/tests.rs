use super::*;
use std::time::Instant;

#[test]
fn learned_context_phase_separates_same_surface_family_by_scene() {
    let corpus = concat!(
        "на улице утром идет дождь. на улице вечером идет дождь. ",
        "в комнате утром горит свет. в комнате вечером горит свет. ",
        "сегодня на улице идет дождь. сегодня в комнате горит свет."
    );
    let (package, _) = compile_context_phase(ContextPhaseCompileInput {
        corpus_text: corpus,
        lexicon_text: "дождь дожди свет света",
        max_fragments: 0,
    });
    let context = super::super::llmwave::tokenize("сегодня на улице идет");
    let readouts = package.score_candidates(&context, &["дождь", "свет"]);

    assert!(readouts[0].profile_present);
    assert!(readouts[0].margin_micro > readouts[1].margin_micro);
}

#[test]
fn no_phase_ablation_removes_context_authority() {
    let (package, _) = compile_context_phase(ContextPhaseCompileInput {
        corpus_text:
            "на улице опять идет дождь. вечером на улице идет дождь. утром на улице идет дождь.",
        lexicon_text: "дождь дожди домик",
        max_fragments: 0,
    });
    let context = super::super::llmwave::tokenize("вечером на улице идет");
    let readouts = package.score_candidates_with_mode(
        &context,
        &["дождь", "домик"],
        ContextPhaseMode::NoPhase,
    );

    assert!(readouts
        .iter()
        .all(|readout| readout.disposition == ContextPhaseDisposition::Unavailable));
}

#[test]
fn compiled_hot_context_readout_stays_inside_microsecond_budget() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("data/lexicon/l3_context_phase_v1.nwpc");
    let package = read_package(&path).expect("tracked L3 context phase package");
    let context = super::super::llmwave::tokenize("на улице снова идет");
    let candidates = ["дождь", "день", "дом"];
    let _ = package.score_candidates(&context, &candidates);
    let mut elapsed = Vec::with_capacity(1_200);
    for _ in 0..1_200 {
        let started = Instant::now();
        let _ = package.score_candidates(&context, &candidates);
        elapsed.push(started.elapsed().as_micros());
    }
    elapsed.sort_unstable();
    let p99 = elapsed[elapsed.len() * 99 / 100];
    let max = *elapsed.last().unwrap_or(&0);
    eprintln!("l3 context phase hot readout: p99={p99}us max={max}us");
    let budget = if cfg!(debug_assertions) { 1_000 } else { 250 };
    assert!(p99 <= budget, "L3 hot readout p99={p99}us > {budget}us");
}
