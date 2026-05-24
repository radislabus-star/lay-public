use super::token_choice::obvious_token_choice;
use crate::llm::repair_mixed_script;
use crate::llm_backend::Choice;
use crate::word_recognizer::is_protected_ascii_token;

pub fn choose_token_hybrid(
    original: &str,
    converted: &str,
) -> Result<Option<String>, Box<dyn std::error::Error>> {
    choose_token_hybrid_with_chooser(original, converted, crate::llm_backend::choose_candidate)
}

pub fn choose_token_consensus(
    original: &str,
    converted: &str,
) -> Result<Option<String>, Box<dyn std::error::Error>> {
    choose_token_consensus_with_chooser(original, converted, crate::llm_backend::choose_candidate)
}

pub(super) fn choose_token_hybrid_with_chooser<F>(
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

pub(super) fn choose_token_consensus_with_chooser<F>(
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
