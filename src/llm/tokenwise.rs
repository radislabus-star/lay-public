use super::token_choice::obvious_token_choice;
use crate::llm_backend::Choice;
use crate::word_reader::split_ws_segments;
use crate::word_recognizer::is_protected_ascii_token;

pub(super) fn keep_protected_ascii_tokens(original: &str, converted: &str) -> Option<String> {
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

pub(super) fn choose_mixed_token_candidate<F>(
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
