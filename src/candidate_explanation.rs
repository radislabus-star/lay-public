//! Candidate explanation scoring.
//!
//! This module scores whether a correction candidate explains the observed
//! noisy input. Frequency alone is not enough authority to apply a correction.

use crate::correction_core::TypingErrorClass;
use crate::language_action::{proof_for_candidate, LanguageActionProof};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CandidateExplanation {
    pub edit_shape: &'static str,
    pub preservation_milli: i16,
    pub lost_mass_milli: i16,
    pub added_mass_milli: i16,
    pub operator_fit_milli: i16,
    pub shortcut_risk_milli: i16,
    pub anti_wave_milli: i16,
    pub explanation_score_milli: i16,
}

impl CandidateExplanation {
    pub fn blocks_apply(self) -> bool {
        self.anti_wave_milli >= 500 && self.operator_fit_milli < 500
    }
}

pub fn explain_candidate(
    original: &str,
    replacement: &str,
    error_class: TypingErrorClass,
    source_id: &str,
) -> CandidateExplanation {
    let original_surface = surface_mass(original);
    let replacement_surface = surface_mass(replacement);
    let common = lcs_len(&original_surface, &replacement_surface);
    let original_len = original_surface.len();
    let replacement_len = replacement_surface.len();
    let proof = proof_for_candidate(error_class, source_id);
    let layout_projection_preserves_key_mass =
        proof == LanguageActionProof::Layout && original_len > 0 && original_len == replacement_len;
    let (preservation_milli, lost_mass_milli, added_mass_milli) =
        if layout_projection_preserves_key_mass {
            (1000, 0, 0)
        } else {
            (
                ratio_milli(common, original_len),
                ratio_milli(original_len.saturating_sub(common), original_len),
                ratio_milli(
                    replacement_len.saturating_sub(common),
                    replacement_len.max(original_len),
                ),
            )
        };
    let boundary_delta = boundary_signature(original) != boundary_signature(replacement);
    let edit_shape = edit_shape(
        &original_surface,
        &replacement_surface,
        common,
        boundary_delta,
        error_class,
    );
    let operator_fit_milli = operator_fit_milli(
        proof,
        error_class,
        edit_shape,
        lost_mass_milli,
        added_mass_milli,
        boundary_delta,
    );
    let shortcut_risk_milli =
        shortcut_risk_milli(source_id, proof, lost_mass_milli, operator_fit_milli);
    let anti_wave_milli = ((shortcut_risk_milli as i32 * (1000 - operator_fit_milli as i32)) / 1000)
        .clamp(0, 1000) as i16;
    // Preserve the full geometry instead of saturating every plausible edit at
    // 1000. The decision core needs to distinguish a mass-preserving operator
    // from a shortcut that happens to end at a frequent word.
    let explanation_score_milli =
        (200 + preservation_milli as i32 * 45 / 100 + operator_fit_milli as i32 * 35 / 100
            - lost_mass_milli as i32 * 30 / 100
            - anti_wave_milli as i32 * 25 / 100)
            .clamp(0, 1000) as i16;

    CandidateExplanation {
        edit_shape,
        preservation_milli,
        lost_mass_milli,
        added_mass_milli,
        operator_fit_milli,
        shortcut_risk_milli,
        anti_wave_milli,
        explanation_score_milli,
    }
}

fn surface_mass(text: &str) -> Vec<char> {
    text.chars()
        .filter(|ch| !ch.is_whitespace() && !is_soft_punctuation(*ch))
        .map(|ch| ch.to_lowercase().next().unwrap_or(ch))
        .collect()
}

fn boundary_signature(text: &str) -> Vec<bool> {
    text.chars().map(char::is_whitespace).collect()
}

fn is_soft_punctuation(ch: char) -> bool {
    matches!(
        ch,
        '.' | ',' | '!' | '?' | ':' | ';' | '"' | '\'' | '`' | '«' | '»' | '(' | ')' | '[' | ']'
    )
}

fn edit_shape(
    original: &[char],
    replacement: &[char],
    common: usize,
    boundary_delta: bool,
    error_class: TypingErrorClass,
) -> &'static str {
    if matches!(
        error_class,
        TypingErrorClass::WrongLayout
            | TypingErrorClass::PartialLayout
            | TypingErrorClass::MixedScript
    ) {
        return "layout_projection";
    }
    if boundary_delta && original == replacement {
        return "boundary_only";
    }
    if is_adjacent_transposition(original, replacement) {
        return "transpose_adjacent";
    }
    match replacement.len() as isize - original.len() as isize {
        0 if common + 1 >= original.len() => "replace_char",
        1 if common == original.len() => "insert_char",
        -1 if common == replacement.len() => "delete_char",
        diff if diff > 0 => "insert_span",
        diff if diff < 0 => "delete_span",
        _ => "composite_edit",
    }
}

fn operator_fit_milli(
    proof: LanguageActionProof,
    error_class: TypingErrorClass,
    edit_shape: &str,
    lost_mass_milli: i16,
    added_mass_milli: i16,
    boundary_delta: bool,
) -> i16 {
    match proof {
        LanguageActionProof::Layout => 1000,
        LanguageActionProof::Boundary => {
            if boundary_delta || matches!(edit_shape, "boundary_only") {
                1000
            } else {
                650
            }
        }
        LanguageActionProof::Completion => {
            if lost_mass_milli == 0 && added_mass_milli > 0 {
                900
            } else {
                300
            }
        }
        LanguageActionProof::Grammar => {
            if lost_mass_milli <= 300 {
                700
            } else {
                350
            }
        }
        LanguageActionProof::Context => {
            if lost_mass_milli <= 200 {
                700
            } else {
                350
            }
        }
        LanguageActionProof::Typo => typo_operator_fit(edit_shape, error_class, lost_mass_milli),
        LanguageActionProof::SafetyVeto | LanguageActionProof::None => 0,
    }
}

fn typo_operator_fit(edit_shape: &str, error_class: TypingErrorClass, lost_mass_milli: i16) -> i16 {
    match (error_class, edit_shape) {
        (TypingErrorClass::AdjacentTransposition, "transpose_adjacent") => 1000,
        (TypingErrorClass::MissingLetter, "insert_char" | "insert_span") => 850,
        (TypingErrorClass::ExtraLetter | TypingErrorClass::RepeatedLetter, "delete_char") => 850,
        (
            TypingErrorClass::LetterSubstitution | TypingErrorClass::CompositeTypo,
            "replace_char",
        ) => 800,
        (_, "delete_span") if lost_mass_milli >= 250 => 150,
        (_, "composite_edit") if lost_mass_milli >= 250 => 250,
        _ if lost_mass_milli <= 200 => 650,
        _ => 350,
    }
}

fn shortcut_risk_milli(
    source_id: &str,
    proof: LanguageActionProof,
    lost_mass_milli: i16,
    operator_fit_milli: i16,
) -> i16 {
    if matches!(
        proof,
        LanguageActionProof::Layout | LanguageActionProof::Boundary
    ) {
        return 0;
    }
    let source_penalty =
        if crate::correction_source_contract::is_surface_or_context_source(source_id) {
            150
        } else {
            0
        };
    (lost_mass_milli as i32 + source_penalty + (500 - operator_fit_milli as i32).max(0) / 2)
        .clamp(0, 1000) as i16
}

fn ratio_milli(part: usize, total: usize) -> i16 {
    if total == 0 {
        return 0;
    }
    ((part as f32 / total as f32) * 1000.0).round() as i16
}

fn lcs_len(left: &[char], right: &[char]) -> usize {
    if left.is_empty() || right.is_empty() {
        return 0;
    }
    let mut prev = vec![0usize; right.len() + 1];
    let mut curr = vec![0usize; right.len() + 1];
    for left_ch in left {
        for (idx, right_ch) in right.iter().enumerate() {
            curr[idx + 1] = if left_ch == right_ch {
                prev[idx] + 1
            } else {
                curr[idx].max(prev[idx + 1])
            };
        }
        std::mem::swap(&mut prev, &mut curr);
        curr.fill(0);
    }
    prev[right.len()]
}

fn is_adjacent_transposition(left: &[char], right: &[char]) -> bool {
    if left.len() != right.len() || left.len() < 2 {
        return false;
    }
    let diffs: Vec<usize> = left
        .iter()
        .zip(right.iter())
        .enumerate()
        .filter_map(|(idx, (a, b))| (a != b).then_some(idx))
        .collect();
    matches!(diffs.as_slice(), [a, b] if *b == *a + 1 && left[*a] == right[*b] && left[*b] == right[*a])
}

#[cfg(test)]
mod tests {
    use super::explain_candidate;
    use crate::correction_core::TypingErrorClass;

    #[test]
    fn explanation_prefers_boundary_preservation_over_shortcut_loss() {
        let split = explain_candidate(
            "тоесть ",
            "то есть ",
            TypingErrorClass::GluedWords,
            "BoundaryCell32",
        );
        let shortcut = explain_candidate(
            "тоесть ",
            "есть ",
            TypingErrorClass::CompositeTypo,
            "L2SurfaceMotifCell32",
        );

        assert_eq!(split.preservation_milli, 1000);
        assert_eq!(split.anti_wave_milli, 0);
        assert!(shortcut.lost_mass_milli >= 300);
        assert!(shortcut.blocks_apply());
    }
}
