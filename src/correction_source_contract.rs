//! Source ownership contract for correction candidates.
//!
//! Candidate generators may use many local cell names, but gate/safety code
//! should reason through these roles instead of open-coded string lists.

use crate::typing_rule_graph::ids;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CorrectionSourceRole {
    Layout,
    Boundary,
    Completion,
    L2Surface,
    L3Context,
    DeterministicTypo,
    Technical,
    Unknown,
}

/// Typed origin carried by a candidate after it leaves its legacy producer.
/// `source_id` remains useful for logs and local diagnostics, but decision
/// authority must use this stable semantic category.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CandidateOrigin {
    Layout,
    LayoutThenTypo,
    Boundary,
    Completion,
    L2Surface,
    L3Context,
    DeterministicTypo,
    Technical,
    Unknown,
}

impl CandidateOrigin {
    pub(crate) const fn source_role(self) -> CorrectionSourceRole {
        match self {
            Self::Layout | Self::LayoutThenTypo => CorrectionSourceRole::Layout,
            Self::Boundary => CorrectionSourceRole::Boundary,
            Self::Completion => CorrectionSourceRole::Completion,
            Self::L2Surface => CorrectionSourceRole::L2Surface,
            Self::L3Context => CorrectionSourceRole::L3Context,
            Self::DeterministicTypo => CorrectionSourceRole::DeterministicTypo,
            Self::Technical => CorrectionSourceRole::Technical,
            Self::Unknown => CorrectionSourceRole::Unknown,
        }
    }

    pub(crate) const fn memory_key(self) -> &'static str {
        match self {
            Self::Layout => "layout",
            Self::LayoutThenTypo => "layout_then_typo",
            Self::Boundary => "boundary",
            Self::Completion => "completion",
            Self::L2Surface => "l2_surface",
            Self::L3Context => "l3_context",
            Self::DeterministicTypo => "deterministic_typo",
            Self::Technical => "technical",
            Self::Unknown => "unknown",
        }
    }
}

pub(crate) fn candidate_origin(source_id: &str) -> CandidateOrigin {
    if is_layout_then_typo_source(source_id) {
        CandidateOrigin::LayoutThenTypo
    } else {
        match source_role(source_id) {
            CorrectionSourceRole::Layout => CandidateOrigin::Layout,
            CorrectionSourceRole::Boundary => CandidateOrigin::Boundary,
            CorrectionSourceRole::Completion => CandidateOrigin::Completion,
            CorrectionSourceRole::L2Surface => CandidateOrigin::L2Surface,
            CorrectionSourceRole::L3Context => CandidateOrigin::L3Context,
            CorrectionSourceRole::DeterministicTypo => CandidateOrigin::DeterministicTypo,
            CorrectionSourceRole::Technical => CandidateOrigin::Technical,
            CorrectionSourceRole::Unknown => CandidateOrigin::Unknown,
        }
    }
}

pub(crate) fn source_role(source_id: &str) -> CorrectionSourceRole {
    if is_boundary_source(source_id) {
        CorrectionSourceRole::Boundary
    } else if is_completion_source(source_id) {
        CorrectionSourceRole::Completion
    } else if is_l3_context_source(source_id) {
        CorrectionSourceRole::L3Context
    } else if is_l2_surface_source(source_id) {
        CorrectionSourceRole::L2Surface
    } else if is_deterministic_typo_source(source_id) {
        CorrectionSourceRole::DeterministicTypo
    } else if is_technical_source(source_id) {
        CorrectionSourceRole::Technical
    } else if is_layout_source(source_id) {
        CorrectionSourceRole::Layout
    } else {
        CorrectionSourceRole::Unknown
    }
}

pub(crate) fn is_layout_source(source_id: &str) -> bool {
    source_id.contains("layout")
        || matches!(
            source_id,
            "LayoutWordCell32"
                | ids::FAST_LAYOUT_EN_TO_RU
                | ids::LAYOUT_RU_TO_EN
                | ids::LAYOUT_EN_TO_RU
                | ids::CONTEXTUAL_LAYOUT_EN_TO_RU
                | ids::EXPERIMENTAL_LAYOUT_EN_TO_RU
                | ids::EXPERIMENTAL_LAYOUT_RU_TO_EN
                | ids::VISUAL_B
                | ids::MIXED_SCRIPT_LAYOUT
                | ids::DUPLICATE_LAYOUT_PREFIX
                | ids::MOVED_PREFIX_PAIR
        )
}

pub(crate) fn is_layout_then_typo_source(source_id: &str) -> bool {
    source_id.starts_with("layout_then_")
}

pub(crate) fn is_boundary_source(source_id: &str) -> bool {
    matches!(
        source_id,
        "BoundaryCell32" | "layout_phrase" | ids::SPLIT_WORD_PAIR | ids::GLUED_PHRASE
    )
}

pub(crate) fn is_completion_source(source_id: &str) -> bool {
    matches!(
        source_id,
        "PhraseForecastCell32" | "L2SurfaceCompletionCell32"
    )
}

pub(crate) fn is_l2_surface_source(source_id: &str) -> bool {
    matches!(source_id, "L2SurfaceMotifCell32" | "L2WordAttractorCell32")
}

pub(crate) fn is_l3_context_source(source_id: &str) -> bool {
    matches!(
        source_id,
        "PhraseCell32" | "PhraseMemoryCell32" | "SemanticWordCell32"
    )
}

pub(crate) fn is_surface_or_context_source(source_id: &str) -> bool {
    is_l2_surface_source(source_id) || is_l3_context_source(source_id)
}

pub(crate) fn is_deterministic_typo_source(source_id: &str) -> bool {
    matches!(
        source_id,
        "composite_ru_typo"
            | ids::ADJACENT_TRANSPOSITION
            | ids::MISSING_LETTER
            | ids::REPEATED_LETTER
            | ids::EXTRA_LETTERS
            | ids::SINGLE_LETTER_SUBSTITUTION
            | ids::VOWEL_CONFUSION
            | ids::VERB_ENDING
            | ids::HARD_SIGN
    ) || is_layout_then_typo_source(source_id)
}

pub(crate) fn is_technical_source(source_id: &str) -> bool {
    matches!(
        source_id,
        "TechTokenCell32" | "TechnicalContextCell32" | ids::LAYOUT_TECHNICAL
    )
}

#[cfg(test)]
mod tests {
    use super::{source_role, CorrectionSourceRole};
    use crate::typing_rule_graph::ids;

    #[test]
    fn classifies_nanda_surface_roles() {
        assert_eq!(
            source_role("L2SurfaceMotifCell32"),
            CorrectionSourceRole::L2Surface
        );
        assert_eq!(
            source_role("L2WordAttractorCell32"),
            CorrectionSourceRole::L2Surface
        );
        assert_eq!(
            source_role("L2SurfaceCompletionCell32"),
            CorrectionSourceRole::Completion
        );
        assert_eq!(
            source_role("SemanticWordCell32"),
            CorrectionSourceRole::L3Context
        );
        assert_eq!(
            source_role("BoundaryCell32"),
            CorrectionSourceRole::Boundary
        );
        assert_eq!(source_role("layout_phrase"), CorrectionSourceRole::Boundary);
        assert_eq!(
            source_role("layout_then_missing-letter"),
            CorrectionSourceRole::DeterministicTypo
        );
        assert_eq!(
            source_role(ids::EXTRA_LETTERS),
            CorrectionSourceRole::DeterministicTypo
        );
    }
}
