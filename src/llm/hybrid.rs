use super::tokenwise::{choose_mixed_token_candidate, keep_protected_ascii_tokens};
use crate::llm::repair_mixed_script;
use crate::llm_backend::{choose_candidate, Choice};
use crate::text_metrics::{has_cyrillic, has_latin};
use crate::token_language::{all_tokens_known, Lang};

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
