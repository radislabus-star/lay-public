use super::*;
use crate::text_edit::{
    authorize_backend_edit, TextEditBackend, TransitionOperator, TransitionProof,
};

#[test]
fn fragment_letter_request_uses_automatic_exact_layout_authority() {
    for (input, replacement, current_is_ru) in
        [("ghbdtn", "привет", false), ("руддщ", "hello", true)]
    {
        let request = on_letter(input, input.chars().last().unwrap(), current_is_ru)
            .unwrap_or_else(|| {
                panic!(
                    "no letter request: {input}: {:?}",
                    analyze(input, Boundary::Open)
                )
            });
        let action = request.action();
        assert_eq!(action.from_text(), input);
        assert_eq!(action.to_text(), replacement);
        assert_eq!(
            action.transition().operator(),
            Some(TransitionOperator::LayoutProjection)
        );
        assert_eq!(action.transition().proof(), Some(TransitionProof::Layout));
        assert_eq!(request.target_layout_is_ru(), !current_is_ru);
        assert!(authorize_backend_edit(TextEditBackend::Ime, action.clone())
            .into_authorized()
            .is_some());
        assert!(on_letter(
            replacement,
            replacement.chars().last().unwrap(),
            !current_is_ru
        )
        .is_none());
    }
}

#[test]
fn fragment_letter_request_ignores_boundaries_short_mixed_and_technical_tokens() {
    for (token, ch) in [
        ("ghbdtn", ' '),
        ("ghbdtn", '\n'),
        ("ghbdtn", '\t'),
        ("ghbdtn", '!'),
        ("ghbdtn", 'x'),
        ("", 'a'),
        ("g", 'g'),
        ("gh", 'h'),
        ("ghbdtn ", ' '),
        ("path/ghbdtn", 'n'),
        ("ghбdtn", 'n'),
        ("123", '3'),
        ("🙂", '🙂'),
    ] {
        assert!(on_letter(token, ch, false).is_none(), "{token:?} {ch:?}");
    }
    assert!(on_letter(&"a".repeat(33), 'a', true).is_none());
}

#[test]
fn fragment_letter_request_changes_decoder_during_both_directions_without_space() {
    for (keys, expected, initial_ru) in [("ghbdtn", "привет", false), ("руддщ", "hello", true)]
    {
        let mut text = String::new();
        let mut is_ru = initial_ru;
        let mut flips = 0;
        for source in keys.chars() {
            let letter = if is_ru == initial_ru {
                source
            } else {
                crate::manual_toggle::project_to_opposite_layout(&source.to_string(), initial_ru)
                    .chars()
                    .next()
                    .unwrap()
            };
            text.push(letter);
            if let Some(request) = on_letter(&text, letter, is_ru) {
                text = request.action().to_text().to_owned();
                is_ru = request.target_layout_is_ru();
                flips += 1;
            }
        }
        assert_eq!(text, expected);
        assert_eq!(
            flips, 1,
            "no reversal or repeated application inside a word"
        );
        assert_eq!(is_ru, !initial_ru);
    }
}

#[test]
fn fragment_letter_decision_preserves_current_support_and_requires_known_target() {
    use crate::typing_transition::decision::TransitionDecisionCore;

    for evidence in [
        "unopposed-known-target",
        "known-current",
        "positive-current",
        "normal-target",
    ] {
        let mut analysis = analyze("руддщ", Boundary::Open).unwrap();
        analysis.russian_score = Score {
            total: 0,
            status: Status::Normal,
            matches: 2,
        };
        analysis.english_score = Score {
            total: 100,
            status: Status::Known,
            matches: 4,
        };
        match evidence {
            "known-current" => analysis.russian_score.status = Status::Known,
            "positive-current" => analysis.russian_score.total = 1,
            "normal-target" => analysis.english_score.status = Status::Normal,
            _ => {}
        }
        analysis.selection = select(analysis.russian_score, analysis.english_score);
        let request = TransitionDecisionCore::decide_fragment_layout("руддщ", true, &analysis);
        assert_eq!(
            request.is_some(),
            evidence == "unopposed-known-target",
            "{evidence}"
        );
        if let Some(request) = request {
            assert_eq!(request.action().to_text(), "hello");
            assert!(
                authorize_backend_edit(TextEditBackend::Ime, request.action().clone())
                    .into_authorized()
                    .is_some()
            );
        }
    }
}
