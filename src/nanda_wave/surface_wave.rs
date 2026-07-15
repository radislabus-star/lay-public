//! Canonical surface-wave primitives for the NANDA L1/L2 shadow path.
//!
//! This mirrors the stable `nando-wave` L1 idea: surface form is represented as
//! UTF-8 byte 4-grams plus boundary/service atoms. It is deliberately below
//! semantics and below live IME behavior.

#[cfg(test)]
use crate::lexical_surface_atoms::SURFACE_WAVE_NGRAM;
use crate::lexical_surface_atoms::{surface_atom_projection, surface_atoms, SURFACE_WAVE_DIM};

pub(super) const SURFACE_WAVE_BYTES: usize =
    SURFACE_WAVE_DIM * std::mem::size_of::<SurfaceWaveLane>();

type SurfaceWaveLane = i16;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct SurfaceWave4096 {
    lanes: [SurfaceWaveLane; SURFACE_WAVE_DIM],
}

impl SurfaceWave4096 {
    #[must_use]
    pub(super) const fn zero() -> Self {
        Self {
            lanes: [0; SURFACE_WAVE_DIM],
        }
    }

    #[must_use]
    pub(super) fn compile(text: &str) -> Self {
        let mut wave = Self::zero();
        for atom in surface_atoms(text) {
            wave.add_atom(atom.position, &atom.bytes);
        }
        wave
    }

    #[must_use]
    pub(super) fn active_lanes(&self) -> usize {
        self.lanes.iter().filter(|value| **value != 0).count()
    }

    fn add_atom(&mut self, position: u64, atom: &[u8]) {
        for trit in surface_atom_projection(position, atom) {
            if trit.value == 0 {
                continue;
            }
            let lane = usize::from(trit.lane);
            self.lanes[lane] = self.lanes[lane].saturating_add(i16::from(trit.value));
        }
    }
}

impl Default for SurfaceWave4096 {
    fn default() -> Self {
        Self::zero()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn surface_wave_uses_four_byte_grams() {
        assert_eq!(SURFACE_WAVE_NGRAM, 4);
        assert!(surface_atoms("проверка").len() >= 4);
        assert!(SurfaceWave4096::compile("проверка").active_lanes() > 0);
    }

    #[test]
    fn short_service_words_still_emit_atoms() {
        for word in ["и", "в", "не", "a", "to"] {
            assert!(!surface_atoms(word).is_empty(), "word={word}");
        }
    }

    #[test]
    fn short_non_service_words_emit_identity_atoms() {
        let atoms = surface_atoms("сыч");
        assert!(!atoms.is_empty(), "atoms={atoms:?}");
        assert!(SurfaceWave4096::compile("сыч").active_lanes() > 0);
    }
}
