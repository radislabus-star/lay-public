use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

use super::super::atoms::AtomChannel;
use super::super::crystal::WAVE_DIMENSION;
use super::super::model::WaveCoupling;
use super::MAX_ANCHOR_SEQUENCE;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::nanda_wave::lexical_grokking) enum ReadoutMode {
    Full,
    WithoutAnti,
    WithoutPhase,
    WithoutSequence,
    WithoutSequenceCertificate,
    LegacySequence,
    WithoutPairwise,
    WithoutPosition,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(in crate::nanda_wave::lexical_grokking) struct GrokkingCandidate {
    pub(in crate::nanda_wave::lexical_grokking) terminal_id: u32,
    pub(in crate::nanda_wave::lexical_grokking) atom_hits: u16,
    pub(in crate::nanda_wave::lexical_grokking) surface_hits: u16,
    pub(in crate::nanda_wave::lexical_grokking) keyboard_hits: u16,
    pub(in crate::nanda_wave::lexical_grokking) structural_milli: u16,
    pub(in crate::nanda_wave::lexical_grokking) position_milli: u16,
    pub(in crate::nanda_wave::lexical_grokking) legacy_sequence_milli: u16,
    pub(in crate::nanda_wave::lexical_grokking) sequence_milli: u16,
    pub(in crate::nanda_wave::lexical_grokking) forward_milli: u16,
    pub(in crate::nanda_wave::lexical_grokking) backward_milli: u16,
    pub(in crate::nanda_wave::lexical_grokking) positive_milli: u16,
    pub(in crate::nanda_wave::lexical_grokking) positive_subcenter_milli: u16,
    pub(in crate::nanda_wave::lexical_grokking) anti_milli: u16,
    pub(in crate::nanda_wave::lexical_grokking) anti_subcenter_milli: u16,
    pub(in crate::nanda_wave::lexical_grokking) hard_negative_milli: u16,
    pub(in crate::nanda_wave::lexical_grokking) ambiguity_milli: u16,
    pub(in crate::nanda_wave::lexical_grokking) ambiguity_threshold_milli: u16,
    pub(in crate::nanda_wave::lexical_grokking) ambiguity_linked: bool,
    pub(in crate::nanda_wave::lexical_grokking) ambiguity_shell: bool,
    pub(in crate::nanda_wave::lexical_grokking) reconstruction_only: bool,
    pub(in crate::nanda_wave::lexical_grokking) pairwise_loss_milli: u16,
    pub(in crate::nanda_wave::lexical_grokking) crystallization_wins: u8,
    pub(in crate::nanda_wave::lexical_grokking) crystallization_required: u8,
    pub(in crate::nanda_wave::lexical_grokking) crystallization_margin_milli: u16,
    pub(in crate::nanda_wave::lexical_grokking) crystallization_complete: bool,
    pub(in crate::nanda_wave::lexical_grokking) crystallization_known_edges: u16,
    pub(in crate::nanda_wave::lexical_grokking) crystallization_unknown_edges: u16,
    pub(in crate::nanda_wave::lexical_grokking) crystallization_tied_edges: u16,
    pub(in crate::nanda_wave::lexical_grokking) crystallization_conflicts: u16,
    pub(in crate::nanda_wave::lexical_grokking) crystallization_cycles: u16,
    pub(in crate::nanda_wave::lexical_grokking) length_milli: u16,
    pub(in crate::nanda_wave::lexical_grokking) geometry_distance: u8,
    pub(in crate::nanda_wave::lexical_grokking) reconstruction_modes: u8,
    pub(in crate::nanda_wave::lexical_grokking) settled_energy: i32,
    pub(in crate::nanda_wave::lexical_grokking) legacy_settled_energy: i32,
    pub(in crate::nanda_wave::lexical_grokking) length_relation: i8,
    pub(in crate::nanda_wave::lexical_grokking) settling_iterations: u8,
    pub(in crate::nanda_wave::lexical_grokking) exact_reconstruction: bool,
}

#[derive(Clone, Copy, Debug)]
pub(in crate::nanda_wave::lexical_grokking) struct AmbiguityObservation {
    pub(in crate::nanda_wave::lexical_grokking) center_index: usize,
    pub(in crate::nanda_wave::lexical_grokking) owner: u32,
    pub(in crate::nanda_wave::lexical_grokking) competitor: u32,
    pub(in crate::nanda_wave::lexical_grokking) coherence_milli: u16,
    pub(in crate::nanda_wave::lexical_grokking) structurally_applicable: bool,
}

#[derive(Clone, Copy, Debug, Default)]
pub(super) struct ForwardActivation {
    pub(super) mass: u64,
    pub(super) hits: u16,
    pub(super) surface_hits: u16,
    pub(super) keyboard_hits: u16,
}

#[derive(Default)]
pub(super) struct ForwardScratch {
    pub(super) activations: Vec<ForwardActivation>,
    pub(super) activation_epochs: Vec<u32>,
    pub(super) epoch: u32,
    pub(super) touched: Vec<u32>,
}

pub(in crate::nanda_wave::lexical_grokking) struct PreparedReadout {
    pub(super) observed: BTreeMap<u32, ObservedAtom>,
    pub(super) character_sequence: AnchorSequence,
    pub(super) observed_char_count: u8,
    pub(super) surface_re: [i32; WAVE_DIMENSION],
    pub(super) surface_im: [i32; WAVE_DIMENSION],
    pub(super) max_forward: u64,
    pub(super) frontier: Vec<(u32, ForwardActivation)>,
    pub(super) frontier_reverse: Option<Vec<Arc<[WaveCoupling]>>>,
    pub(super) geometry_reserve_ids: BTreeSet<u32>,
    pub(super) reconstruction_only_ids: BTreeSet<u32>,
}

#[derive(Clone, Copy, Debug)]
pub(super) struct ObservedAtom {
    pub(super) position: u8,
    pub(super) weight: u8,
    pub(super) channel: AtomChannel,
}

pub(super) type BirthAtom = (usize, u32, ObservedAtom);

#[derive(Clone, Copy)]
pub(super) enum CachePlanOrder {
    Support,
    Degree,
    ObservedUses,
}

pub(super) struct FirstTouchWarmProfile {
    pub(super) atom_ids: Vec<u32>,
    pub(super) sampled_words: usize,
    pub(super) damage_surfaces: usize,
    pub(super) observed_atoms: usize,
    pub(super) protected_budget_bytes: usize,
}

#[derive(Clone, Copy, Debug, Default)]
pub(super) struct AnchorSequence {
    pub(super) atoms: [u32; MAX_ANCHOR_SEQUENCE],
    pub(super) len: u8,
}

impl AnchorSequence {
    pub(super) fn as_slice(&self) -> &[u32] {
        &self.atoms[..usize::from(self.len)]
    }
}
