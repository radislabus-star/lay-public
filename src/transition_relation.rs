//! Adapter-neutral relation atoms for proposed text transitions.
//!
//! Atoms describe transition shape and proof, never concrete word identity.
//! They are the shared input for DecisionCore diagnostics and L2 phase memory.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct TransitionRelationInput<'a> {
    pub(crate) action_operator: &'a str,
    pub(crate) edit_operator: &'a str,
    pub(crate) proof: &'a str,
    pub(crate) verifier_passed: bool,
    pub(crate) left_context_changed: bool,
    pub(crate) changed_tokens: usize,
}

pub(crate) fn transition_state_id(text: &str) -> String {
    let tokens = text.split_whitespace().collect::<Vec<_>>();
    let Some(last) = tokens.last() else {
        return "empty".to_string();
    };
    let mut bytes = Vec::new();
    if let Some(previous) = tokens.get(tokens.len().saturating_sub(2)) {
        if previous.chars().all(|ch| !ch.is_alphanumeric()) {
            bytes.extend(previous.to_lowercase().as_bytes());
            bytes.push(0x1e);
        }
    }
    bytes.extend(last.to_lowercase().as_bytes());
    let hash = bytes.iter().fold(0xcbf2_9ce4_8422_2325_u64, |hash, byte| {
        (hash ^ u64::from(*byte)).wrapping_mul(0x0000_0100_0000_01b3)
    });
    format!("{hash:016x}")
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
pub(crate) enum TransitionOperatorKind {
    LayoutProjection = 1,
    AdjacentTransposition = 2,
    MissingLetterRepair = 3,
    RepeatedLetterRepair = 4,
    ExtraLetterRepair = 5,
    LetterSubstitution = 6,
    BoundarySplit = 7,
    BoundaryMerge = 8,
    AcceptCompletion = 9,
    CompositeTypo = 10,
    ContextChoice = 11,
    ManualToggle = 12,
    Other = 255,
}

impl TransitionOperatorKind {
    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Self::LayoutProjection => "layout_projection",
            Self::AdjacentTransposition => "adjacent_transposition",
            Self::MissingLetterRepair => "missing_letter_repair",
            Self::RepeatedLetterRepair => "repeated_letter_repair",
            Self::ExtraLetterRepair => "extra_letter_repair",
            Self::LetterSubstitution => "letter_substitution",
            Self::BoundarySplit => "boundary_split",
            Self::BoundaryMerge => "boundary_merge",
            Self::AcceptCompletion => "accept_completion",
            Self::CompositeTypo => "composite_typo",
            Self::ContextChoice => "context_choice",
            Self::ManualToggle => "manual_toggle",
            Self::Other => "other",
        }
    }

    pub(crate) fn from_code(code: u8) -> Option<Self> {
        Some(match code {
            1 => Self::LayoutProjection,
            2 => Self::AdjacentTransposition,
            3 => Self::MissingLetterRepair,
            4 => Self::RepeatedLetterRepair,
            5 => Self::ExtraLetterRepair,
            6 => Self::LetterSubstitution,
            7 => Self::BoundarySplit,
            8 => Self::BoundaryMerge,
            9 => Self::AcceptCompletion,
            10 => Self::CompositeTypo,
            11 => Self::ContextChoice,
            12 => Self::ManualToggle,
            255 => Self::Other,
            _ => return None,
        })
    }

    pub(crate) fn from_action_operator(action: &str) -> Self {
        match action {
            "flip_layout" | "fix_mixed_layout" => Self::LayoutProjection,
            "fix_transposition" => Self::AdjacentTransposition,
            "restore_missing_letter" => Self::MissingLetterRepair,
            "remove_repeated_letter" => Self::RepeatedLetterRepair,
            "remove_extra_letter" => Self::ExtraLetterRepair,
            "replace_letter" => Self::LetterSubstitution,
            "split_glued_words" => Self::BoundarySplit,
            "join_broken_word" => Self::BoundaryMerge,
            "complete_word" => Self::AcceptCompletion,
            "apply_context_choice" | "fix_grammar_form" => Self::ContextChoice,
            "manual_toggle" | "sync_layout_state" => Self::ManualToggle,
            "fix_typo" => Self::CompositeTypo,
            _ => Self::Other,
        }
    }

    pub(crate) fn infer(original: &str, replacement: &str, operation: &str) -> Self {
        match operation {
            "layout" | "layout_projection" | "flip_layout" => return Self::LayoutProjection,
            "transposition" | "adjacent_transposition" => {
                return Self::AdjacentTransposition;
            }
            "missing" | "missing_letter" | "missing_letter_repair" => {
                return Self::MissingLetterRepair;
            }
            "repeated" | "repeated_letter" | "repeated_letter_repair" => {
                return Self::RepeatedLetterRepair;
            }
            "extra" | "extra_letter" | "extra_letter_repair" => {
                return Self::ExtraLetterRepair;
            }
            "substitution" | "letter_substitution" => return Self::LetterSubstitution,
            "split" | "boundary_split" => return Self::BoundarySplit,
            "merge" | "glue" | "boundary_merge" => return Self::BoundaryMerge,
            "completion" | "accept_completion" => return Self::AcceptCompletion,
            "context" | "grammar" | "context_choice" => return Self::ContextChoice,
            "composite_typo" => return Self::CompositeTypo,
            "manual_toggle" => return Self::ManualToggle,
            _ => {}
        }

        let original_words = lowercase_whitespace_tokens(original);
        let replacement_words = lowercase_whitespace_tokens(replacement);
        if replacement_words.len() == original_words.len() + 1 {
            return Self::BoundarySplit;
        }
        if original_words.len() == replacement_words.len() + 1 {
            return Self::BoundaryMerge;
        }
        let original = original_words
            .last()
            .map(String::as_str)
            .unwrap_or_default();
        let replacement = replacement_words
            .last()
            .map(String::as_str)
            .unwrap_or_default();
        if script_class(original) != script_class(replacement)
            && matches!(
                (script_class(original), script_class(replacement)),
                ("en", "ru") | ("ru", "en") | ("mixed", _) | (_, "mixed")
            )
        {
            return Self::LayoutProjection;
        }
        if crate::text_metrics::is_adjacent_transposition(original, replacement) {
            return Self::AdjacentTransposition;
        }
        let original_len = original.chars().count();
        let replacement_len = replacement.chars().count();
        if replacement_len == original_len + 1 {
            return Self::MissingLetterRepair;
        }
        if original_len == replacement_len + 1 {
            return if has_repeated_letter(original) {
                Self::RepeatedLetterRepair
            } else {
                Self::ExtraLetterRepair
            };
        }
        if original_len == replacement_len
            && crate::text_metrics::damerau_levenshtein(original, replacement) == 1
        {
            return Self::LetterSubstitution;
        }
        Self::CompositeTypo
    }

    pub(crate) const fn action(self) -> &'static str {
        match self {
            Self::LayoutProjection => "flip_layout",
            Self::AdjacentTransposition => "fix_transposition",
            Self::MissingLetterRepair => "restore_missing_letter",
            Self::RepeatedLetterRepair => "remove_repeated_letter",
            Self::ExtraLetterRepair => "remove_extra_letter",
            Self::LetterSubstitution => "replace_letter",
            Self::BoundarySplit => "split_glued_words",
            Self::BoundaryMerge => "join_broken_word",
            Self::AcceptCompletion => "complete_word",
            Self::ContextChoice => "apply_context_choice",
            Self::ManualToggle => "manual_toggle",
            Self::CompositeTypo => "fix_typo",
            Self::Other => "suggest_only",
        }
    }

    pub(crate) const fn edit_operator(self) -> &'static str {
        match self {
            Self::LayoutProjection | Self::ManualToggle => "layout_projection",
            Self::BoundarySplit | Self::BoundaryMerge => "boundary_shift",
            Self::AcceptCompletion => "completion",
            Self::ContextChoice => "phrase_token_repair",
            Self::AdjacentTransposition
            | Self::MissingLetterRepair
            | Self::RepeatedLetterRepair
            | Self::ExtraLetterRepair
            | Self::LetterSubstitution
            | Self::CompositeTypo => "replace_current_word",
            Self::Other => "unknown",
        }
    }

    pub(crate) const fn proof(self) -> &'static str {
        match self {
            Self::LayoutProjection | Self::ManualToggle => "layout",
            Self::BoundarySplit | Self::BoundaryMerge => "boundary",
            Self::AcceptCompletion => "completion",
            Self::ContextChoice => "context",
            Self::AdjacentTransposition
            | Self::MissingLetterRepair
            | Self::RepeatedLetterRepair
            | Self::ExtraLetterRepair
            | Self::LetterSubstitution
            | Self::CompositeTypo => "typo",
            Self::Other => "none",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct TransitionRelationAtoms {
    atoms: Vec<String>,
    surface_key: String,
    verifier_passed: bool,
}

impl TransitionRelationAtoms {
    pub(crate) fn inferred(
        original: &str,
        replacement: &str,
        operation: &str,
    ) -> (TransitionOperatorKind, Self) {
        let operator = TransitionOperatorKind::infer(original, replacement, operation);
        let atoms = Self::for_operator(original, replacement, operator);
        (operator, atoms)
    }

    pub(crate) fn for_operator(
        original: &str,
        replacement: &str,
        operator: TransitionOperatorKind,
    ) -> Self {
        let left_context_changed = left_context(original) != left_context(replacement);
        let changed_tokens = changed_token_count(original, replacement);
        let verifier_passed = operator_shape_verified(original, replacement, operator)
            && (!left_context_changed
                || matches!(
                    operator,
                    TransitionOperatorKind::LayoutProjection
                        | TransitionOperatorKind::BoundarySplit
                        | TransitionOperatorKind::BoundaryMerge
                        | TransitionOperatorKind::ContextChoice
                        | TransitionOperatorKind::ManualToggle
                ));
        Self::encode(
            original,
            replacement,
            TransitionRelationInput {
                action_operator: operator.action(),
                edit_operator: operator.edit_operator(),
                proof: operator.proof(),
                verifier_passed,
                left_context_changed,
                changed_tokens,
            },
        )
    }

    pub(crate) fn encode(
        original: &str,
        replacement: &str,
        input: TransitionRelationInput<'_>,
    ) -> Self {
        let original_last = last_word(original);
        let replacement_last = last_word(replacement);
        let original_len = original_last.chars().count();
        let replacement_len = replacement_last.chars().count();
        let prefix = crate::text_metrics::common_prefix_char_len(original_last, replacement_last);
        let suffix = common_suffix_chars(original_last, replacement_last);
        let distance = crate::text_metrics::damerau_levenshtein(original_last, replacement_last);
        let observed_edit =
            observed_edit_shape(original, replacement, original_last, replacement_last);
        let changed_region = changed_region(original, replacement);
        let boundary = if input.edit_operator.contains("boundary") || input.changed_tokens > 1 {
            "changed"
        } else {
            "same"
        };
        let atoms = vec![
            "field:typing-transition-v1".to_string(),
            format!("action:{}", input.action_operator),
            format!("edit-operator:{}", input.edit_operator),
            format!("proof:{}", input.proof),
            format!("verified:{}", input.verifier_passed),
            format!(
                "left-context:{}",
                if input.left_context_changed {
                    "touched"
                } else {
                    "preserved"
                }
            ),
            format!(
                "word-count:{}->{}",
                word_count(original),
                word_count(replacement)
            ),
            format!("changed-tokens:{}", small_bucket(input.changed_tokens)),
            format!("boundary:{boundary}"),
            format!("observed-edit:{observed_edit}"),
            format!("changed-region:{changed_region}"),
            format!(
                "current-token:{}",
                if original_last == replacement_last {
                    "preserved"
                } else {
                    "changed"
                }
            ),
            format!(
                "script:{}->{}",
                script_class(original_last),
                script_class(replacement_last)
            ),
            format!(
                "len:{}->{}",
                small_bucket(original_len),
                small_bucket(replacement_len)
            ),
            format!(
                "delta:{}",
                signed_bucket(replacement_len as isize - original_len as isize)
            ),
            format!("prefix:{}", small_bucket(prefix)),
            format!("suffix:{}", small_bucket(suffix)),
            format!("edit:{}", small_bucket(distance)),
        ];
        let surface_key = atoms
            .iter()
            .filter(|atom| !atom.starts_with("verified:"))
            .cloned()
            .collect::<Vec<_>>()
            .join("|");
        Self {
            atoms,
            surface_key,
            verifier_passed: input.verifier_passed,
        }
    }

    pub(crate) fn atoms(&self) -> &[String] {
        &self.atoms
    }

    pub(crate) fn surface_key(&self) -> &str {
        &self.surface_key
    }

    pub(crate) const fn verifier_passed(&self) -> bool {
        self.verifier_passed
    }
}

fn operator_shape_verified(
    original: &str,
    replacement: &str,
    operator: TransitionOperatorKind,
) -> bool {
    let original_words = lowercase_whitespace_tokens(original);
    let replacement_words = lowercase_whitespace_tokens(replacement);
    let original_last = original_words
        .last()
        .map(String::as_str)
        .unwrap_or_default();
    let replacement_last = replacement_words
        .last()
        .map(String::as_str)
        .unwrap_or_default();
    let original_len = original_last.chars().count();
    let replacement_len = replacement_last.chars().count();
    let distance = crate::text_metrics::damerau_levenshtein(original_last, replacement_last);
    match operator {
        TransitionOperatorKind::LayoutProjection => {
            original_last != replacement_last
                && aligned_changed_tokens_are_layout_projections(
                    &original_words,
                    &replacement_words,
                )
        }
        TransitionOperatorKind::ManualToggle => {
            aligned_changed_tokens_are_layout_projections(&original_words, &replacement_words)
        }
        TransitionOperatorKind::AdjacentTransposition => {
            crate::text_metrics::is_adjacent_transposition(original_last, replacement_last)
        }
        TransitionOperatorKind::MissingLetterRepair => replacement_len == original_len + 1,
        TransitionOperatorKind::RepeatedLetterRepair => {
            original_len == replacement_len + 1 && has_repeated_letter(original_last)
        }
        TransitionOperatorKind::ExtraLetterRepair => {
            original_len == replacement_len + 1 && !has_repeated_letter(original_last)
        }
        TransitionOperatorKind::LetterSubstitution => {
            original_len == replacement_len && distance == 1
        }
        TransitionOperatorKind::BoundarySplit => {
            replacement_words.len() == original_words.len() + 1
        }
        TransitionOperatorKind::BoundaryMerge => {
            original_words.len() == replacement_words.len() + 1
        }
        TransitionOperatorKind::AcceptCompletion => {
            replacement_len > original_len && replacement_last.starts_with(original_last)
        }
        TransitionOperatorKind::CompositeTypo => {
            original_words.len() == replacement_words.len()
                && script_class(original_last) == script_class(replacement_last)
                && distance >= 2
        }
        TransitionOperatorKind::ContextChoice => {
            original_words.len() == replacement_words.len()
                && changed_token_count(original, replacement) == 1
        }
        TransitionOperatorKind::Other => false,
    }
}

fn aligned_changed_tokens_are_layout_projections(
    original: &[String],
    replacement: &[String],
) -> bool {
    original.len() == replacement.len()
        && original.iter().zip(replacement).any(|(left, right)| {
            left != right
                && matches!(
                    (script_class(left), script_class(right)),
                    ("en", "ru") | ("ru", "en") | ("mixed", "ru") | ("mixed", "en")
                )
        })
        && original.iter().zip(replacement).all(|(left, right)| {
            left == right
                || matches!(
                    (script_class(left), script_class(right)),
                    ("en", "ru") | ("ru", "en") | ("mixed", "ru") | ("mixed", "en")
                )
        })
}

fn changed_region(original: &str, replacement: &str) -> &'static str {
    let original = lowercase_whitespace_tokens(original);
    let replacement = lowercase_whitespace_tokens(replacement);
    if original.len() != replacement.len() {
        return "boundary";
    }
    let changed = original
        .iter()
        .zip(&replacement)
        .enumerate()
        .filter_map(|(index, (left, right))| (left != right).then_some(index))
        .collect::<Vec<_>>();
    if changed.is_empty() {
        "none"
    } else if changed.len() == original.len() {
        "all"
    } else if changed.as_slice() == [original.len().saturating_sub(1)] {
        "current-only"
    } else if !changed.contains(&original.len().saturating_sub(1)) {
        "left-only"
    } else {
        "mixed"
    }
}

fn observed_edit_shape(
    original: &str,
    replacement: &str,
    original_last: &str,
    replacement_last: &str,
) -> &'static str {
    if word_count(original) != word_count(replacement) {
        return "boundary-change";
    }
    if crate::text_metrics::is_adjacent_transposition(original_last, replacement_last) {
        return "adjacent-transposition";
    }
    let original_len = original_last.chars().count();
    let replacement_len = replacement_last.chars().count();
    if replacement_len == original_len + 1 {
        return "insert-one";
    }
    if original_len == replacement_len + 1 {
        return "delete-one";
    }
    let distance = crate::text_metrics::damerau_levenshtein(original_last, replacement_last);
    if original_len == replacement_len && distance == 1 {
        return "substitute-one";
    }
    if original_last == replacement_last {
        "keep"
    } else {
        "composite"
    }
}

fn lowercase_whitespace_tokens(text: &str) -> Vec<String> {
    text.split_whitespace()
        .map(|word| word.to_lowercase())
        .collect()
}

fn left_context(text: &str) -> Vec<String> {
    let mut words = lowercase_whitespace_tokens(text);
    words.pop();
    words
}

fn changed_token_count(original: &str, replacement: &str) -> usize {
    let original = lowercase_whitespace_tokens(original);
    let replacement = lowercase_whitespace_tokens(replacement);
    if original.len() != replacement.len() {
        return original.len().max(replacement.len());
    }
    original
        .iter()
        .zip(replacement)
        .filter(|(left, right)| left != &right)
        .count()
}

fn has_repeated_letter(text: &str) -> bool {
    text.chars()
        .zip(text.chars().skip(1))
        .any(|(left, right)| left == right)
}

fn last_word(text: &str) -> &str {
    text.split_whitespace().last().unwrap_or_default()
}

fn word_count(text: &str) -> usize {
    text.split_whitespace().count()
}

fn common_suffix_chars(left: &str, right: &str) -> usize {
    left.chars()
        .rev()
        .zip(right.chars().rev())
        .take_while(|(left, right)| left == right)
        .count()
}

fn script_class(text: &str) -> &'static str {
    let has_cyrillic = text.chars().any(|ch| matches!(ch, 'А'..='я' | 'Ё' | 'ё'));
    let has_ascii = text.chars().any(|ch| ch.is_ascii_alphabetic());
    match (has_cyrillic, has_ascii) {
        (true, true) => "mixed",
        (true, false) => "ru",
        (false, true) => "en",
        (false, false) => "other",
    }
}

fn small_bucket(value: usize) -> &'static str {
    match value {
        0 => "0",
        1 => "1",
        2 => "2",
        3..=4 => "3-4",
        5..=8 => "5-8",
        _ => "9+",
    }
}

fn signed_bucket(value: isize) -> &'static str {
    match value {
        ..=-3 => "-3+",
        -2 => "-2",
        -1 => "-1",
        0 => "0",
        1 => "+1",
        2 => "+2",
        _ => "+3+",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::candidate_contract::CandidateOrigin;
    use crate::correction_core::TypingErrorClass;
    use crate::typing_transition::action::verify_action_operator;

    fn encode(
        original: &str,
        replacement: &str,
        error_class: TypingErrorClass,
        origin: CandidateOrigin,
    ) -> TransitionRelationAtoms {
        let action = verify_action_operator(original, replacement, error_class, origin);
        TransitionRelationAtoms::encode(
            original,
            replacement,
            TransitionRelationInput {
                action_operator: action.operator.as_str(),
                edit_operator: action.edit_operator.as_str(),
                proof: action.edit_proof.as_str(),
                verifier_passed: action.verifier_passed,
                left_context_changed: action.left_context_changed,
                changed_tokens: action.changed_tokens,
            },
        )
    }

    #[test]
    fn same_operator_on_different_words_has_the_same_surface_key() {
        let left = encode(
            "пукнт ",
            "пункт ",
            TypingErrorClass::AdjacentTransposition,
            CandidateOrigin::DeterministicTypo,
        );
        let right = encode(
            "слвоо ",
            "слово ",
            TypingErrorClass::AdjacentTransposition,
            CandidateOrigin::DeterministicTypo,
        );

        assert_eq!(left.surface_key(), right.surface_key());
        assert!(left.atoms().iter().all(|atom| !atom.contains("пукнт")));
    }

    #[test]
    fn unsafe_left_context_transition_has_distinct_negative_atoms() {
        let atoms = encode(
            "содержкой ",
            "что получилось содержать ",
            TypingErrorClass::CompositeTypo,
            CandidateOrigin::L2Surface,
        );

        assert!(atoms
            .atoms()
            .iter()
            .any(|atom| atom == "left-context:touched"));
        assert!(atoms.atoms().iter().any(|atom| atom == "verified:false"));
    }

    #[test]
    fn state_id_distinguishes_adjacent_symbol_scene_without_storing_text() {
        let plain = transition_state_id("xnj ");
        let question = transition_state_id("? xnj ");
        let same_question = transition_state_id("? XNJ ");

        assert_ne!(plain, question);
        assert_eq!(question, same_question);
        assert!(!question.contains("xnj"));
        assert_eq!(question.len(), 16);
    }
}
