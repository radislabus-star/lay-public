//! Optional local model arbiter for already-built layout candidates.
//!
//! The daemon builds deterministic candidates first. A configured backend can
//! only vote between those candidates with a short A/B answer; invalid answers
//! are ignored. The default backend is `off`. Direct GGUF loading requires the
//! `direct-llm` feature.

use crate::llm_backend::{choose_candidate, Choice};
#[cfg(test)]
pub(crate) use crate::llm_backend::{parse_choice, AnthropicResponse, OpenAiChatResponse};
pub use crate::mixed_script_repair::repair_mixed_script;
use crate::text_metrics::{has_cyrillic, has_latin, is_cyrillic_char};
use crate::token_language::{all_tokens_known, is_known_en_token, is_known_ru_token, Lang};
use crate::word_reader::split_ws_segments;
use crate::word_recognizer::is_protected_ascii_token;

pub fn convert(text: &str) -> Result<String, Box<dyn std::error::Error>> {
    let direction = crate::dict::detect_direction(text);
    let converted = crate::dict::convert(text, direction);
    choose_candidate(text, &converted).map(|choice| match choice {
        Some(Choice::Original) => text.to_string(),
        Some(Choice::Converted) | None => converted,
    })
}

pub fn convert_hybrid(
    original: &str,
    converted: &str,
) -> Result<Option<String>, Box<dyn std::error::Error>> {
    if let Some(repaired) = repair_mixed_script(original) {
        return Ok(Some(repaired));
    }

    if let Some(protected) = keep_protected_ascii_tokens(original, converted) {
        return Ok(Some(protected));
    }

    if all_tokens_known(original, Lang::Ru) && !all_tokens_known(converted, Lang::En) {
        return Ok(Some(original.to_string()));
    }

    if let Some(tokenwise) = choose_mixed_token_candidate(original, converted, choose_candidate)? {
        return Ok(Some(tokenwise));
    }

    if has_cyrillic(original) && has_latin(original) {
        return Ok(Some(original.to_string()));
    }

    Ok(Some(match choose_candidate(original, converted)? {
        Some(Choice::Original) | None => original.to_string(),
        Some(Choice::Converted) => converted.to_string(),
    }))
}

pub fn choose_token_hybrid(
    original: &str,
    converted: &str,
) -> Result<Option<String>, Box<dyn std::error::Error>> {
    choose_token_hybrid_with_chooser(original, converted, choose_candidate)
}

pub fn choose_token_consensus(
    original: &str,
    converted: &str,
) -> Result<Option<String>, Box<dyn std::error::Error>> {
    choose_token_consensus_with_chooser(original, converted, choose_candidate)
}

fn choose_token_hybrid_with_chooser<F>(
    original: &str,
    converted: &str,
    _chooser: F,
) -> Result<Option<String>, Box<dyn std::error::Error>>
where
    F: Fn(&str, &str) -> Result<Option<Choice>, Box<dyn std::error::Error>>,
{
    if original == converted {
        return Ok(Some(original.to_string()));
    }

    if let Some(repaired) = repair_mixed_script(original) {
        return Ok(Some(repaired));
    }

    if is_protected_ascii_token(original) {
        return Ok(Some(original.to_string()));
    }

    if let Some(choice) = obvious_token_choice(original, converted) {
        return Ok(Some(match choice {
            Choice::Original => original.to_string(),
            Choice::Converted => converted.to_string(),
        }));
    }

    Ok(Some(original.to_string()))
}

fn choose_token_consensus_with_chooser<F>(
    original: &str,
    converted: &str,
    chooser: F,
) -> Result<Option<String>, Box<dyn std::error::Error>>
where
    F: Fn(&str, &str) -> Result<Option<Choice>, Box<dyn std::error::Error>>,
{
    if original == converted {
        return Ok(Some(original.to_string()));
    }

    if let Some(repaired) = repair_mixed_script(original) {
        return Ok(Some(repaired));
    }

    if is_protected_ascii_token(original) {
        return Ok(Some(original.to_string()));
    }

    let Some(choice) = obvious_token_choice(original, converted) else {
        return Ok(Some(original.to_string()));
    };

    match choice {
        Choice::Original => Ok(Some(original.to_string())),
        Choice::Converted => match chooser(original, converted) {
            Ok(Some(Choice::Converted)) | Err(_) => Ok(Some(converted.to_string())),
            Ok(Some(Choice::Original)) | Ok(None) => Ok(Some(original.to_string())),
        },
    }
}

pub fn warm_up() -> Result<(), Box<dyn std::error::Error>> {
    crate::llm_backend::warm_up()
}

pub fn model_backend_enabled() -> bool {
    crate::llm_backend::model_backend_enabled()
}

fn keep_protected_ascii_tokens(original: &str, converted: &str) -> Option<String> {
    let original_segments = split_ws_segments(original);
    let converted_segments = split_ws_segments(converted);
    if original_segments.len() != converted_segments.len() {
        return None;
    }

    let mut protected_count = 0;
    let mut converted_count = 0;
    let mut out = String::with_capacity(original.len().max(converted.len()));

    for ((orig, orig_ws), (conv, conv_ws)) in
        original_segments.iter().zip(converted_segments.iter())
    {
        if orig_ws != conv_ws {
            return None;
        }
        if *orig_ws {
            out.push_str(orig);
        } else if is_protected_ascii_token(orig) {
            protected_count += 1;
            out.push_str(orig);
        } else {
            match obvious_token_choice(orig, conv).unwrap_or(Choice::Original) {
                Choice::Original => out.push_str(orig),
                Choice::Converted => {
                    if orig != conv {
                        converted_count += 1;
                    }
                    out.push_str(conv);
                }
            }
        }
    }

    if protected_count > 0 && converted_count > 0 && out != original && out != converted {
        Some(out)
    } else {
        None
    }
}

fn choose_mixed_token_candidate<F>(
    original: &str,
    converted: &str,
    mut chooser: F,
) -> Result<Option<String>, Box<dyn std::error::Error>>
where
    F: FnMut(&str, &str) -> Result<Option<Choice>, Box<dyn std::error::Error>>,
{
    let original_segments = split_ws_segments(original);
    let converted_segments = split_ws_segments(converted);
    if original_segments.len() != converted_segments.len() {
        return Ok(None);
    }

    let mut word_count = 0;
    let mut kept_original = false;
    let mut used_converted = false;
    let mut used_chooser = false;
    let mut out = String::with_capacity(original.len().max(converted.len()));

    for ((orig, orig_ws), (conv, conv_ws)) in
        original_segments.iter().zip(converted_segments.iter())
    {
        if orig_ws != conv_ws {
            return Ok(None);
        }
        if *orig_ws {
            out.push_str(orig);
            continue;
        }

        word_count += 1;
        if orig == conv {
            out.push_str(orig);
            continue;
        }

        let choice = match obvious_token_choice(orig, conv) {
            Some(choice) => Some(choice),
            None => {
                used_chooser = true;
                chooser(orig, conv)?
            }
        };

        match choice {
            Some(Choice::Original) => {
                kept_original = true;
                out.push_str(orig);
            }
            Some(Choice::Converted) => {
                used_converted = true;
                out.push_str(conv);
            }
            None => return Ok(None),
        }
    }

    let deterministic_choice = word_count > 0 && !used_chooser;
    let mixed_choice =
        word_count >= 2 && kept_original && used_converted && out != original && out != converted;
    if deterministic_choice || mixed_choice {
        Ok(Some(out))
    } else {
        Ok(None)
    }
}

fn obvious_token_choice(original: &str, converted: &str) -> Option<Choice> {
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

#[cfg(test)]
#[path = "llm_tests.rs"]
mod tests;
