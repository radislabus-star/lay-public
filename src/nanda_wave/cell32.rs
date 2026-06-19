use super::mode::{mix64, Mode8, ModeRole, MODES_PER_CELL32};
use super::signal::{ActiveMode, WavePacket};

pub const DEFAULT_TOP_K: usize = 8;
pub const SPARSE_MODE_PROBES: usize = 64;

#[derive(Debug, Clone, PartialEq)]
pub struct NandaCell32 {
    pub id: &'static str,
    pub role: ModeRole,
    seed: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SymbolStimulus {
    pub ch: char,
    pub prev: Option<char>,
    pub index_in_token: usize,
}

impl NandaCell32 {
    pub const fn new(id: &'static str, role: ModeRole, seed: u64) -> Self {
        Self { id, role, seed }
    }

    pub fn observe_symbol(&self, stimulus: SymbolStimulus, top_k: usize) -> WavePacket {
        let stimulus_hash = self.stimulus_hash(stimulus);
        let transition = self.transition(stimulus.prev, stimulus.ch);
        let mut modes = (0..SPARSE_MODE_PROBES)
            .map(|probe| {
                let idx = sparse_mode_index(stimulus_hash, probe);
                let mode = Mode8::seeded(self.seed, idx, self.role);
                let energy = mode.energy_for(stimulus_hash ^ idx as u64, transition);
                ActiveMode {
                    cell: self.id,
                    mode_id: idx,
                    role: mode.role,
                    energy,
                    phase: mode.phase,
                    coherence: coherence(mode.phase, energy),
                }
            })
            .collect::<Vec<_>>();
        modes.sort_by(|left, right| {
            right
                .energy
                .total_cmp(&left.energy)
                .then_with(|| left.mode_id.cmp(&right.mode_id))
        });
        modes.truncate(top_k);
        WavePacket {
            layer: "L1",
            cell: self.id,
            modes,
        }
    }

    fn stimulus_hash(&self, stimulus: SymbolStimulus) -> u64 {
        let mut value = self.seed ^ stimulus.ch as u32 as u64;
        value ^= (stimulus.index_in_token as u64) << 32;
        if let Some(prev) = stimulus.prev {
            value ^= (prev as u32 as u64) << 16;
        }
        value ^= class_bits(stimulus.ch);
        mix64(value)
    }

    fn transition(&self, prev: Option<char>, ch: char) -> i8 {
        let Some(prev) = prev else {
            return 0;
        };
        let same_class = class_bits(prev) == class_bits(ch);
        let boundary = ch.is_whitespace() || prev.is_whitespace();
        match (same_class, boundary) {
            (true, false) => 48,
            (false, true) => 24,
            (false, false) => -16,
            (true, true) => 8,
        }
    }
}

fn sparse_mode_index(stimulus_hash: u64, probe: usize) -> usize {
    (super::mode::mix64(stimulus_hash ^ ((probe as u64) << 32)) as usize) % MODES_PER_CELL32
}

fn class_bits(ch: char) -> u64 {
    if ch.is_ascii_alphabetic() {
        0x10
    } else if ('а'..='я').contains(&ch) || ('А'..='Я').contains(&ch) || ch == 'ё' || ch == 'Ё'
    {
        0x20
    } else if ch.is_ascii_digit() {
        0x30
    } else if ch.is_whitespace() {
        0x40
    } else if ch.is_ascii_punctuation() {
        0x50
    } else {
        0x60
    }
}

fn coherence(phase: i8, energy: f32) -> f32 {
    let centered = 1.0 - (phase.unsigned_abs() as f32 / 128.0);
    (0.65 * energy + 0.35 * centered).clamp(0.0, 1.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cell_emits_top_k_modes() {
        let cell = NandaCell32::new("Utf8Cell32", ModeRole::Utf8, 1);
        let packet = cell.observe_symbol(
            SymbolStimulus {
                ch: 'д',
                prev: None,
                index_in_token: 0,
            },
            8,
        );
        assert_eq!(packet.modes.len(), 8);
        assert!(packet
            .modes
            .windows(2)
            .all(|pair| pair[0].energy >= pair[1].energy));
    }
}
