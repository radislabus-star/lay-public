use crate::correction_core::UnifiedCorrectionCandidate;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum L2FieldBridgeKind {
    CompactL2,
    Shadow,
}

impl L2FieldBridgeKind {
    pub(crate) const fn surface_source_id(self) -> &'static str {
        match self {
            Self::CompactL2 => "L2LexicalPhaseCell32",
            Self::Shadow => "L2FieldShadowSurface",
        }
    }

    pub(crate) const fn boundary_source_id(self) -> &'static str {
        match self {
            Self::CompactL2 => "BoundaryCell32",
            Self::Shadow => "L2FieldShadowBoundary",
        }
    }

    pub(crate) const fn l11_source_id(self) -> &'static str {
        match self {
            Self::CompactL2 => "L11SurfaceRestore",
            Self::Shadow => "L2FieldShadowL11",
        }
    }

    pub(crate) const fn morph_source_id(self) -> &'static str {
        match self {
            Self::CompactL2 => "L2MorphologyPhaseCell32",
            Self::Shadow => "L2FieldShadowMorphology",
        }
    }

    pub(crate) const fn near_neighbor_source_id(self) -> &'static str {
        match self {
            Self::CompactL2 => "L2NearNeighborCell32",
            Self::Shadow => "L2FieldShadowNearNeighbor",
        }
    }
}

#[derive(Debug, Default)]
pub(crate) struct L2FieldShadowReadout {
    pub(crate) candidates: Vec<UnifiedCorrectionCandidate>,
}

impl L2FieldShadowReadout {
    pub(crate) fn new(candidates: Vec<UnifiedCorrectionCandidate>) -> Self {
        Self { candidates }
    }
}
