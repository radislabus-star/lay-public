use super::super::phase_field::{add_hashed_atom, phase_center_from_sum, stable_hash64, PhaseCell};
use super::model::{EncodedL4Scene, L4CrossSceneContextSignal, L4CrossSceneInput};
use super::CELLS;

pub(crate) fn encode_scene(input: L4CrossSceneInput<'_>) -> EncodedL4Scene {
    let mut vector = vec![PhaseCell::default(); CELLS];
    push_code(&mut vector, "operator", input.profile.operator as u64, 1.40);
    push_code(
        &mut vector,
        "direction",
        input
            .profile
            .direction
            .map_or(0, |value| u64::from(value.code())),
        1.25,
    );
    push_code(
        &mut vector,
        "scope",
        input
            .profile
            .scope
            .map_or(0, |value| u64::from(value.code())),
        1.20,
    );
    push_code(
        &mut vector,
        "context-signal",
        input.context_signal as u64,
        1.25,
    );
    push_code(
        &mut vector,
        "l2-signal",
        (input.l2_signal as i8 as i16 + 2) as u64,
        0.75,
    );
    push_code(
        &mut vector,
        "context-count",
        small_bucket(input.context.len()) as u64,
        0.75,
    );
    push_code(
        &mut vector,
        "from-script",
        script_code(last_token(input.from_text)),
        1.0,
    );
    push_code(
        &mut vector,
        "to-script",
        script_code(last_token(input.to_text)),
        1.0,
    );
    push_code(
        &mut vector,
        "from-length",
        small_bucket(last_token(input.from_text).chars().count()) as u64,
        0.55,
    );
    push_code(
        &mut vector,
        "to-length",
        small_bucket(last_token(input.to_text).chars().count()) as u64,
        0.55,
    );
    push_code(
        &mut vector,
        "edit-distance",
        small_bucket(crate::text_metrics::damerau_levenshtein(
            last_token(input.from_text),
            last_token(input.to_text),
        )) as u64,
        0.75,
    );
    push_code(
        &mut vector,
        "candidate-relation",
        input.candidate_relation_id,
        0.45,
    );
    push_code(&mut vector, "keep-relation", input.keep_relation_id, 0.35);
    if input.l3_relation_class != 0 {
        push_code(&mut vector, "l3-relation", input.l3_relation_class, 0.60);
    }

    for (position, token) in input.context.iter().rev().take(4).rev().enumerate() {
        let role = stable_hash64(b"context-script", position as u64 + 1);
        add_hashed_atom(
            &mut vector,
            role,
            script_code(token),
            if position >= 2 { 0.85 } else { 0.65 },
        );
        let punctuation = punctuation_code(token);
        if punctuation != 0 {
            add_hashed_atom(&mut vector, role.rotate_left(11), punctuation, 0.45);
        }
    }
    for (position, atom) in input.relation_atoms.iter().enumerate() {
        let (role, value) = atom.split_once(':').unwrap_or(("relation", atom.as_str()));
        add_hashed_atom(
            &mut vector,
            stable_hash64(role.as_bytes(), position as u64 + 17),
            stable_hash64(value.as_bytes(), 0x4c34_5245_4c41_5445),
            0.42,
        );
    }

    let vector = phase_center_from_sum(&vector);
    let mut compact = Vec::with_capacity(CELLS * 2);
    for cell in &vector {
        compact.push(super::super::phase_field::quantize(cell.re) as u8);
        compact.push(super::super::phase_field::quantize(cell.im) as u8);
    }
    EncodedL4Scene {
        fingerprint: stable_hash64(&compact, super::ENCODER_HASH),
        vector,
        candidate_relation_id: input.candidate_relation_id,
        keep_relation_id: input.keep_relation_id,
    }
}

pub(crate) fn candidate_relation_id(atoms: &[String]) -> u64 {
    atoms.iter().enumerate().fold(
        stable_hash64(b"l4-candidate-relation-v1", 1),
        |state, (index, atom)| {
            crate::stable_hash::mix64_golden(
                state ^ stable_hash64(atom.as_bytes(), index as u64 + 1),
            )
        },
    )
}

pub(crate) fn keep_relation_id() -> u64 {
    stable_hash64(b"l4-keep-relation-v1", 1)
}

pub(crate) fn relation_class_from_context(context: &[String], target: &str) -> u64 {
    let mut bytes = Vec::with_capacity(context.len() + 2);
    bytes.push(script_code(target) as u8);
    bytes.push(small_bucket(context.len()) as u8);
    bytes.extend(
        context
            .iter()
            .rev()
            .take(4)
            .map(|token| script_code(token) as u8),
    );
    stable_hash64(&bytes, 0x4c34_4354_5854)
}

pub(crate) fn context_signal_from_text(
    context: &[String],
    target: &str,
) -> L4CrossSceneContextSignal {
    let target_script = script_code(last_token(target));
    let mut matching = 0usize;
    let mut conflicting = 0usize;
    for token in context.iter().rev().take(4) {
        let script = script_code(token);
        if script == 0 || target_script == 0 {
            continue;
        }
        if script == target_script {
            matching += 1;
        } else if script != 3 && target_script != 3 {
            conflicting += 1;
        }
    }
    if matching > conflicting && matching > 0 {
        L4CrossSceneContextSignal::Support
    } else if conflicting > matching && conflicting > 0 {
        L4CrossSceneContextSignal::Suppress
    } else if matching + conflicting > 0 {
        L4CrossSceneContextSignal::Neutral
    } else {
        L4CrossSceneContextSignal::Unknown
    }
}

fn push_code(vector: &mut [PhaseCell], role: &str, code: u64, weight: f32) {
    let identity = stable_hash64(role.as_bytes(), 0x4c34_5343_454e_45);
    add_hashed_atom(vector, identity, code, weight);
}

fn last_token(text: &str) -> &str {
    text.split_whitespace().last().unwrap_or(text)
}

fn script_code(text: &str) -> u64 {
    let mut ru = false;
    let mut en = false;
    for ch in text.chars() {
        ru |= matches!(ch, 'а'..='я' | 'А'..='Я' | 'ё' | 'Ё');
        en |= ch.is_ascii_alphabetic();
    }
    match (ru, en) {
        (false, false) => 0,
        (true, false) => 1,
        (false, true) => 2,
        (true, true) => 3,
    }
}

fn punctuation_code(text: &str) -> u64 {
    let first = text
        .chars()
        .next()
        .is_some_and(|ch| ch.is_ascii_punctuation());
    let last = text
        .chars()
        .last()
        .is_some_and(|ch| ch.is_ascii_punctuation());
    match (first, last) {
        (false, false) => 0,
        (true, false) => 1,
        (false, true) => 2,
        (true, true) => 3,
    }
}

const fn small_bucket(value: usize) -> usize {
    match value {
        0 => 0,
        1 => 1,
        2 => 2,
        3..=4 => 3,
        5..=8 => 4,
        _ => 5,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::nanda_wave::l4_cross_scene::model::L4CrossSceneL2Signal;
    use crate::transition_relation::{TransitionOperatorKind, TransitionRelationAtoms};
    use crate::typing_memory::{LayoutProjectionDirection, LayoutProjectionScope};

    #[test]
    fn encoder_is_candidate_relative_but_application_independent() {
        let relation = TransitionRelationAtoms::for_operator(
            "ghbdtn",
            "привет",
            TransitionOperatorKind::LayoutProjection,
        );
        let context = vec!["мы".to_string(), "пишем".to_string()];
        let input = L4CrossSceneInput {
            profile: super::super::L4CrossSceneProfileKey::new(
                TransitionOperatorKind::LayoutProjection,
                Some(LayoutProjectionDirection::EnToRu),
                Some(LayoutProjectionScope::CurrentToken),
            ),
            context: &context,
            from_text: "ghbdtn",
            to_text: "привет",
            relation_atoms: relation.atoms(),
            candidate_relation_id: candidate_relation_id(relation.atoms()),
            keep_relation_id: keep_relation_id(),
            l3_relation_class: relation_class_from_context(&context, "привет"),
            context_signal: context_signal_from_text(&context, "привет"),
            l2_signal: L4CrossSceneL2Signal::Support,
        };
        let first = encode_scene(input);
        let second = encode_scene(input);

        assert_eq!(first.fingerprint, second.fingerprint);
        assert_eq!(first.vector, second.vector);
        assert_ne!(first.candidate_relation_id, first.keep_relation_id);
    }
}
