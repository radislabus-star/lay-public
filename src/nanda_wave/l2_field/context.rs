use super::model::{LocalContextMode, L2_PHASE_CELLS};

pub(crate) fn context_mode(context: &str) -> LocalContextMode {
    let tokens = bounded_context_tokens(context);
    let slot = tokens.iter().position(|token| *token == "_").unwrap_or(0);
    let left = slot
        .checked_sub(1)
        .and_then(|index| tokens.get(index))
        .copied();
    let right = tokens.get(slot + 1).copied();
    let context_key = bounded_context_key("l2-context", &tokens, slot);
    LocalContextMode {
        left_class: token_class(left),
        right_class: token_class(right),
        punctuation_class: punctuation_class(left, right),
        adjacency_mode: if left.is_some() && right.is_some() {
            0
        } else {
            1
        },
        position_mode: slot.min(u8::MAX as usize) as u8,
        flags: 0,
        lexical_anchor: bounded_context_key("l2-anchor", &tokens, slot),
        stable_key: context_key,
    }
}

pub(crate) fn scene_wave(context: &str) -> [i8; L2_PHASE_CELLS] {
    let mut cells = [0_i16; L2_PHASE_CELLS];
    for (index, token) in bounded_context_tokens(context).into_iter().enumerate() {
        if token == "_" {
            continue;
        }
        let hash = stable_key(&["scene", &index.to_string(), token]) as usize;
        let cell = hash % L2_PHASE_CELLS;
        let sign = if hash & 1 == 0 { 1 } else { -1 };
        cells[cell] = cells[cell].saturating_add(9 * sign);
        cells[(cell + 17) % L2_PHASE_CELLS] =
            cells[(cell + 17) % L2_PHASE_CELLS].saturating_add(5 * sign);
    }
    let max = cells
        .iter()
        .map(|cell| cell.unsigned_abs())
        .max()
        .unwrap_or(1)
        .max(1);
    let mut wave = [0_i8; L2_PHASE_CELLS];
    for (target, source) in wave.iter_mut().zip(cells) {
        *target = (i32::from(source) * 120 / i32::from(max))
            .clamp(i32::from(i8::MIN), i32::from(i8::MAX)) as i8;
    }
    wave
}

fn bounded_context_tokens(context: &str) -> Vec<&str> {
    let tokens = context.split_whitespace().collect::<Vec<_>>();
    let Some(slot) = tokens.iter().position(|token| *token == "_") else {
        return vec!["_"];
    };
    let start = slot.saturating_sub(2);
    let end = (slot + 2).min(tokens.len());
    tokens[start..end].to_vec()
}

fn bounded_context_key(domain: &str, tokens: &[&str], slot: usize) -> u32 {
    let slot = slot.to_string();
    let mut parts = Vec::with_capacity(tokens.len() + 2);
    parts.push(domain);
    parts.push(&slot);
    parts.extend(tokens.iter().copied());
    stable_key(&parts)
}

fn token_class(token: Option<&str>) -> u16 {
    let Some(token) = token else {
        return 0;
    };
    let lower = token.to_lowercase();
    if crate::lexicon::is_ru_short_preposition(&lower)
        || matches!(lower.as_str(), "в" | "к" | "с" | "о")
    {
        1
    } else if crate::phrase_lexicon::is_short_russian_function_word(&lower) {
        2
    } else if token.chars().all(crate::keyboard::is_cyrillic_letter) {
        3
    } else if token.chars().all(|ch| ch.is_ascii_alphabetic()) {
        4
    } else {
        5
    }
}

fn punctuation_class(left: Option<&str>, right: Option<&str>) -> u8 {
    left.into_iter()
        .chain(right)
        .flat_map(|token| token.chars())
        .find_map(|ch| match ch {
            ',' => Some(1),
            '.' | '!' | '?' => Some(2),
            ':' | ';' => Some(3),
            _ => None,
        })
        .unwrap_or(0)
}

fn stable_key(parts: &[&str]) -> u32 {
    parts
        .iter()
        .flat_map(|part| part.as_bytes().iter().copied().chain([0xff]))
        .fold(0x811c9dc5_u32, |state, byte| {
            state.wrapping_mul(0x01000193) ^ u32::from(byte)
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn context_encoding_is_deterministic_and_position_sensitive() {
        assert_eq!(context_mode("в _"), context_mode("в _"));
        assert_ne!(context_mode("в _"), context_mode("_ в"));
        assert_ne!(scene_wave("в _"), scene_wave("_ в"));
        assert_eq!(
            context_mode("это длинный текст говорю о _"),
            context_mode("говорю о _")
        );
    }

    #[test]
    fn context_mode_keeps_the_governor_before_a_shared_preposition() {
        assert_ne!(
            context_mode("лежит на _"),
            context_mode("сосредоточен на _")
        );
        assert_ne!(context_mode("двигаюсь к _"), context_mode("подошел к _"));
    }
}
