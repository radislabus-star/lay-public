//! Typed candidate identity shared by L2, L3 and the transition core.
//!
//! Producer names are diagnostics. This enum is the stable decision input.

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
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CandidateReadoutRoute {
    CanonicalL2Field,
    FullWave,
}

impl CandidateReadoutRoute {
    pub const fn live_default() -> Self {
        Self::CanonicalL2Field
    }

    pub const fn compare_reference() -> Self {
        Self::FullWave
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CorrectionSourceRole {
    Layout,
    Boundary,
    Completion,
    L2Surface,
    L3Context,
    DeterministicTypo,
    Technical,
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
        }
    }

    pub(crate) const fn is_surface_or_context(self) -> bool {
        matches!(self, Self::L2Surface | Self::L3Context)
    }
}

#[cfg(test)]
mod tests {
    use super::CandidateReadoutRoute;

    #[test]
    fn canonical_l2_field_is_the_only_live_default_route() {
        assert_eq!(
            CandidateReadoutRoute::live_default(),
            CandidateReadoutRoute::CanonicalL2Field
        );
        assert_eq!(
            CandidateReadoutRoute::compare_reference(),
            CandidateReadoutRoute::FullWave
        );
    }
}
