pub(crate) const L2_PHASE_CELLS: usize = 60;

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct FormCenterRef {
    pub(crate) l1_terminal_id: u32,
    pub(crate) decoder_ref: u32,
    pub(crate) script_flags: u16,
    pub(crate) length_bucket: u8,
    pub(crate) flags: u8,
    pub(crate) reserved: u32,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct LemmaCenter {
    pub(crate) primary_pos: u16,
    pub(crate) flags: u16,
    pub(crate) form_start: u32,
    pub(crate) form_count: u32,
    pub(crate) local_context_start: u32,
    pub(crate) local_context_count: u32,
    pub(crate) competition_start: u32,
    pub(crate) competition_count: u32,
    pub(crate) reserved: u32,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct MorphBinding {
    pub(crate) form_center_ref: u32,
    pub(crate) lemma_center_id: u32,
    pub(crate) feature_mask: u32,
    pub(crate) support: u16,
    pub(crate) phase: i8,
    pub(crate) flags: u8,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct LocalContextMode {
    pub(crate) left_class: u16,
    pub(crate) right_class: u16,
    pub(crate) punctuation_class: u8,
    pub(crate) adjacency_mode: i8,
    pub(crate) position_mode: u8,
    pub(crate) flags: u8,
    pub(crate) lexical_anchor: u32,
    pub(crate) stable_key: u32,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct SlotPhaseCenter {
    pub(crate) cells: [i8; L2_PHASE_CELLS],
    pub(crate) feature_mask: u32,
    pub(crate) context_mode_id: u32,
    pub(crate) support: u16,
    pub(crate) mass: u16,
    pub(crate) polarity: i8,
    pub(crate) flags: u8,
    pub(crate) reserved: u16,
}

impl Default for SlotPhaseCenter {
    fn default() -> Self {
        Self {
            cells: [0; L2_PHASE_CELLS],
            feature_mask: 0,
            context_mode_id: 0,
            support: 0,
            mass: 0,
            polarity: 0,
            flags: 0,
            reserved: 0,
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct NeighborCoupling {
    pub(crate) context_mode_id: u32,
    pub(crate) target_lemma_id: u32,
    pub(crate) target_feature_mask: u32,
    pub(crate) support: i16,
    pub(crate) repel: i16,
    pub(crate) source_anchor: u32,
    pub(crate) flags: u16,
    pub(crate) reserved: u16,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct CompetitionEdge {
    pub(crate) left_form_ref: u32,
    pub(crate) right_form_ref: u32,
    pub(crate) context_mode_id: u32,
    pub(crate) support_delta: i16,
    pub(crate) anti_delta: i16,
    pub(crate) evidence: u32,
    pub(crate) flags: u16,
    pub(crate) reserved: u16,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct TieCalibration {
    pub(crate) minimum_positive: i32,
    pub(crate) minimum_margin: i32,
    pub(crate) tie_window: i32,
    pub(crate) abstain_window: i32,
    pub(crate) false_authority_ceiling_milli: u16,
    pub(crate) flags: u16,
    pub(crate) evidence_count: u32,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct L2FieldPackage {
    pub(crate) l1_package_fingerprint: u64,
    pub(crate) form_refs: Vec<FormCenterRef>,
    pub(crate) lemma_centers: Vec<LemmaCenter>,
    pub(crate) morph_bindings: Vec<MorphBinding>,
    pub(crate) context_modes: Vec<LocalContextMode>,
    pub(crate) slot_centers: Vec<SlotPhaseCenter>,
    pub(crate) neighbor_couplings: Vec<NeighborCoupling>,
    pub(crate) competition_edges: Vec<CompetitionEdge>,
    pub(crate) calibration: TieCalibration,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn canonical_l2_records_keep_fixed_widths() {
        assert_eq!(std::mem::size_of::<FormCenterRef>(), 16);
        assert_eq!(std::mem::size_of::<LemmaCenter>(), 32);
        assert_eq!(std::mem::size_of::<MorphBinding>(), 16);
        assert_eq!(std::mem::size_of::<LocalContextMode>(), 16);
        assert_eq!(std::mem::size_of::<SlotPhaseCenter>(), 76);
        assert_eq!(std::mem::size_of::<NeighborCoupling>(), 24);
        assert_eq!(std::mem::size_of::<CompetitionEdge>(), 24);
        assert_eq!(std::mem::size_of::<TieCalibration>(), 24);
    }
}
