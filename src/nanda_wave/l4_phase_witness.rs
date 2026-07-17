//! Bounded phase memory for structural typing-transition surfaces.
//!
//! The bank compiles accepted and rejected surface relations into a fixed
//! number of phase centers. Runtime queries contain no corpus strings and no
//! direct edit authority.

use std::collections::HashMap;

use super::phase_field::{
    add_cluster, add_hashed_atom, max_coherence, phase_center_from_sum, PhaseCell, PhaseCenter,
};

const CELLS: usize = 24;
const MAX_CENTERS: usize = 4;
const SPLIT_COHERENCE: f32 = 0.72;
const MAX_COUNT_WEIGHT: u32 = 8;

#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct L4PhaseWitnessBank {
    positive: Vec<PhaseCenter>,
    negative: Vec<PhaseCenter>,
}

impl Eq for L4PhaseWitnessBank {}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(crate) struct L4PhaseWitnessReadout {
    pub(crate) positive: f32,
    pub(crate) negative: f32,
    pub(crate) margin: f32,
    pub(crate) positive_centers: u8,
    pub(crate) negative_centers: u8,
    pub(crate) supported: bool,
}

impl L4PhaseWitnessBank {
    pub(crate) fn compile(
        positive: &HashMap<String, u32>,
        negative: &HashMap<String, u32>,
    ) -> Self {
        Self {
            positive: compile_centers(positive),
            negative: compile_centers(negative),
        }
    }

    pub(crate) fn readout(&self, surface: &str) -> L4PhaseWitnessReadout {
        if surface.trim().is_empty() || (self.positive.is_empty() && self.negative.is_empty()) {
            return L4PhaseWitnessReadout::default();
        }
        let vector = surface_vector(surface);
        let positive = max_coherence(&vector, &self.positive).unwrap_or_default();
        let negative = max_coherence(&vector, &self.negative).unwrap_or_default();
        L4PhaseWitnessReadout {
            positive,
            negative,
            margin: positive - negative,
            positive_centers: self.positive.len().min(u8::MAX as usize) as u8,
            negative_centers: self.negative.len().min(u8::MAX as usize) as u8,
            supported: !self.positive.is_empty() || !self.negative.is_empty(),
        }
    }

    pub(crate) fn logical_payload_bytes(&self) -> usize {
        self.positive
            .iter()
            .chain(&self.negative)
            .map(|center| {
                center
                    .sum
                    .len()
                    .saturating_add(center.center.len())
                    .saturating_mul(std::mem::size_of::<PhaseCell>())
                    .saturating_add(std::mem::size_of::<u32>())
            })
            .sum()
    }
}

fn compile_centers(source: &HashMap<String, u32>) -> Vec<PhaseCenter> {
    let mut entries = source.iter().collect::<Vec<_>>();
    entries.sort_by(|left, right| left.0.cmp(right.0));
    let mut centers = Vec::new();
    for (surface, count) in entries {
        let vector = surface_vector(surface);
        for _ in 0..(*count).clamp(1, MAX_COUNT_WEIGHT) {
            add_cluster(&mut centers, &vector, MAX_CENTERS, SPLIT_COHERENCE);
        }
    }
    centers
}

fn surface_vector(surface: &str) -> [PhaseCell; CELLS] {
    let mut vector = [PhaseCell::default(); CELLS];
    for (position, atom) in surface.split('|').enumerate() {
        let (role, value) = atom.split_once('=').unwrap_or(("atom", atom));
        let identity = super::phase_field::stable_hash64(role.as_bytes(), position as u64 + 1);
        let phase_key = super::phase_field::stable_hash64(value.as_bytes(), identity);
        add_hashed_atom(&mut vector, identity, phase_key, 1.0);
    }
    let center = phase_center_from_sum(&vector);
    vector.copy_from_slice(&center);
    vector
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn positive_and_anti_surfaces_interfere_in_opposite_directions() {
        let positive = HashMap::from([(
            "op=replacement|source=autocorrect|words=1->1|delta=0|edit=1".to_string(),
            5,
        )]);
        let negative = HashMap::from([(
            "op=boundary|source=autocorrect|words=2->1|delta=-1|edit=2".to_string(),
            5,
        )]);
        let bank = L4PhaseWitnessBank::compile(&positive, &negative);

        let accepted = bank.readout("op=replacement|source=autocorrect|words=1->1|delta=0|edit=1");
        let rejected = bank.readout("op=boundary|source=autocorrect|words=2->1|delta=-1|edit=2");
        let without_anti = L4PhaseWitnessBank::compile(&positive, &HashMap::new())
            .readout("op=boundary|source=autocorrect|words=2->1|delta=-1|edit=2");

        assert!(accepted.margin > 0.5);
        assert!(rejected.margin < -0.5);
        assert!(rejected.margin < without_anti.margin);
        assert!(accepted.supported);
        assert!(bank.logical_payload_bytes() < 8 * 1024);
    }
}
