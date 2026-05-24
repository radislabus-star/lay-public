use crate::llm_backend::Choice;
use crate::text_metrics::{has_cyrillic, has_latin, is_cyrillic_char};
use crate::token_language::{is_known_en_token, is_known_ru_token};

pub(super) fn obvious_token_choice(original: &str, converted: &str) -> Option<Choice> {
    let original_cyr = has_cyrillic(original);
    let original_lat = has_latin(original);
    let converted_cyr = has_cyrillic(converted);
    let converted_lat = has_latin(converted);

    if (original_cyr && original_lat) || (converted_cyr && converted_lat) {
        return Some(Choice::Original);
    }

    if original_cyr && !original_lat && converted_lat && !converted_cyr {
        let original_known = is_known_ru_token(original);
        let converted_known = is_known_en_token(converted);
        if original_known != converted_known {
            return Some(if original_known {
                Choice::Original
            } else {
                Choice::Converted
            });
        }

        let original_ru = crate::quality::score(original, "ru");
        let converted_en = crate::quality::score(converted, "en");
        return obvious_quality_choice(original_ru, converted_en)
            .or_else(|| short_unknown_prefers_original(original, converted));
    }

    if original_lat && !original_cyr && converted_cyr && !converted_lat {
        let original_known = is_known_en_token(original);
        let converted_known = is_known_ru_token(converted);
        if original_known != converted_known {
            return Some(if original_known {
                Choice::Original
            } else {
                Choice::Converted
            });
        }
        if is_single_ascii_letter(original) && is_single_cyrillic_letter(converted) {
            return Some(Choice::Converted);
        }
        if !original_known && is_long_upper_ascii_word(original) {
            let converted_ru = crate::quality::score(converted, "ru");
            return Some(if converted_ru >= 0.7 {
                Choice::Converted
            } else {
                Choice::Original
            });
        }

        let original_en = crate::quality::score(original, "en");
        let converted_ru = crate::quality::score(converted, "ru");
        return obvious_quality_choice(original_en, converted_ru)
            .or_else(|| short_unknown_prefers_original(original, converted));
    }

    None
}

fn short_unknown_prefers_original(original: &str, converted: &str) -> Option<Choice> {
    let original_len = original.chars().filter(|ch| ch.is_alphabetic()).count();
    let converted_len = converted.chars().filter(|ch| ch.is_alphabetic()).count();
    (original_len <= 3 && converted_len <= 3).then_some(Choice::Original)
}

fn obvious_quality_choice(original_score: f32, converted_score: f32) -> Option<Choice> {
    if original_score >= 0.99 && converted_score < 0.7 {
        Some(Choice::Original)
    } else if original_score < 0.7 && converted_score >= 0.99 {
        Some(Choice::Converted)
    } else {
        None
    }
}

fn is_long_upper_ascii_word(token: &str) -> bool {
    token.chars().count() > 4 && token.chars().all(|ch| ch.is_ascii_uppercase())
}

fn is_single_ascii_letter(token: &str) -> bool {
    let mut chars = token.chars();
    matches!((chars.next(), chars.next()), (Some(ch), None) if ch.is_ascii_alphabetic())
}

fn is_single_cyrillic_letter(token: &str) -> bool {
    let mut chars = token.chars();
    matches!((chars.next(), chars.next()), (Some(ch), None) if is_cyrillic_char(ch))
}
