use super::crystal::{AtomWaveCode, ComplexBasisWave, WordCenter64};
use super::ngram_graph::NGramGraph;
use super::restoration::RestorationCalibration;

pub(super) const COUPLING_FLAG_CHARACTER_ANCHOR: u8 = 1;
pub(super) const CENTER_FLAG_ASCII_SCRIPT: u8 = 1;
pub(super) const CENTER_FLAG_CYRILLIC_SCRIPT: u8 = 2;
pub(super) const CENTER_PHASE_FLAG_PHYSICAL_KEY_GEOMETRY: u8 = 1;

pub(super) fn surface_script_flags(surface: &str) -> u8 {
    let mut flags = 0_u8;
    for character in surface.chars() {
        if character.is_ascii_alphabetic() {
            flags |= CENTER_FLAG_ASCII_SCRIPT;
        } else if matches!(character, '\u{0400}'..='\u{052f}') {
            flags |= CENTER_FLAG_CYRILLIC_SCRIPT;
        }
    }
    flags
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) struct AtomRecord {
    pub(super) wave_code: AtomWaveCode,
    pub(super) coupling_start: u32,
    pub(super) coupling_count: u32,
    pub(super) support: u16,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) struct WaveCoupling {
    pub(super) peer_id: u32,
    pub(super) strength: u8,
    pub(super) phase_relation: i8,
    pub(super) position_mode: u8,
    pub(super) flags: u8,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) struct DecoderNode {
    pub(super) parent: u32,
    pub(super) symbol: u32,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub(super) struct PairKey {
    pub(super) low_terminal: u32,
    pub(super) high_terminal: u32,
}

impl PairKey {
    pub(super) fn new(left: u32, right: u32) -> Option<Self> {
        if left == right {
            return None;
        }
        Some(Self {
            low_terminal: left.min(right),
            high_terminal: left.max(right),
        })
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) struct PairPhaseProfile {
    pub(super) key: PairKey,
    pub(super) low_wins_start: u32,
    pub(super) high_wins_start: u32,
    pub(super) low_wins_count: u16,
    pub(super) high_wins_count: u16,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) struct CenterPhaseProfile {
    pub(super) positive_start: u32,
    pub(super) anti_start: u32,
    pub(super) hard_negative_start: u32,
    pub(super) keyboard_geometry_start: u32,
    pub(super) ambiguity_start: u32,
    pub(super) positive_count: u8,
    pub(super) anti_count: u8,
    pub(super) hard_negative_count: u8,
    pub(super) keyboard_geometry_count: u8,
    pub(super) flags: u8,
    pub(super) ambiguity_count: u8,
    pub(super) min_ambiguity_milli: u16,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(super) struct LexicalGrokkingPackage {
    pub(super) corpus_hash: u64,
    pub(super) graph: NGramGraph,
    pub(super) basis: Vec<ComplexBasisWave>,
    pub(super) atoms: Vec<AtomRecord>,
    pub(super) forward_couplings: Vec<WaveCoupling>,
    pub(super) reverse_couplings: Vec<WaveCoupling>,
    pub(super) anti_centers: Vec<WordCenter64>,
    pub(super) pair_profiles: Vec<PairPhaseProfile>,
    pub(super) pair_centers: Vec<WordCenter64>,
    pub(super) center_phase_profiles: Vec<CenterPhaseProfile>,
    pub(super) positive_subcenters: Vec<WordCenter64>,
    pub(super) anti_subcenters: Vec<WordCenter64>,
    pub(super) hard_negative_subcenters: Vec<WordCenter64>,
    pub(super) ambiguity_subcenters: Vec<WordCenter64>,
    pub(super) keyboard_geometry_units: Vec<u32>,
    pub(super) restoration_calibration: RestorationCalibration,
    pub(super) centers: Vec<WordCenter64>,
    pub(super) decoder_nodes: Vec<DecoderNode>,
}

impl LexicalGrokkingPackage {
    pub(super) fn terminal_count(&self) -> u32 {
        self.centers.len() as u32
    }
}
