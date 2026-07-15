pub const CELL32_BYTES: usize = 32 * 1024;
pub const CELL32_HEADER_BYTES: usize = 256;
pub const CELL32_PROJECTION_BYTES: usize = 4 * 1024;
pub const CELL32_MODE_BANK_BYTES: usize = 16 * 1024;
pub const CELL32_TRANSITION_BYTES: usize = 4 * 1024;
pub const CELL32_INTERFERENCE_BYTES: usize = 4 * 1024;
pub const CELL32_CALIBRATION_BYTES: usize = 2 * 1024;
pub const CELL32_SCRATCH_BYTES: usize = 1792;
pub const MODE8_BYTES: usize = 8;
pub const MODES_PER_CELL32: usize = CELL32_MODE_BANK_BYTES / MODE8_BYTES;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModeRole {
    Utf8,
    Script,
    Keyboard,
    Boundary,
    Word,
    Layout,
    Typo,
    Technical,
    Phrase,
    Sentence,
    Guard,
    Mesh,
}

impl ModeRole {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Utf8 => "utf8",
            Self::Script => "script",
            Self::Keyboard => "keyboard",
            Self::Boundary => "boundary",
            Self::Word => "word",
            Self::Layout => "layout",
            Self::Typo => "typo",
            Self::Technical => "technical",
            Self::Phrase => "phrase",
            Self::Sentence => "sentence",
            Self::Guard => "guard",
            Self::Mesh => "mesh",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Mode8 {
    pub frequency_id: u16,
    pub sin_weight: i8,
    pub cos_weight: i8,
    pub amplitude: i8,
    pub phase: i8,
    pub damping: u8,
    pub role: ModeRole,
}

impl Mode8 {
    pub fn seeded(seed: u64, idx: usize, role: ModeRole) -> Self {
        let mixed = crate::stable_hash::mix64_avalanche(seed ^ idx as u64);
        Self {
            frequency_id: mixed as u16,
            sin_weight: byte_to_i8(mixed >> 8),
            cos_weight: byte_to_i8(mixed >> 16),
            amplitude: byte_to_i8(mixed >> 24).saturating_abs().max(8),
            phase: byte_to_i8(mixed >> 32),
            damping: ((mixed >> 40) as u8).max(1),
            role,
        }
    }

    pub fn energy_for(self, stimulus: u64, transition: i8) -> f32 {
        let phase = ((stimulus as u16) ^ self.frequency_id).count_ones() as f32;
        let phase_fit = 1.0 - (phase / 16.0);
        let wave = (self.sin_weight as f32 * phase_fit
            + self.cos_weight as f32 * (1.0 - phase_fit.abs()))
            / 127.0;
        let amp = self.amplitude.unsigned_abs() as f32 / 127.0;
        let transition = transition as f32 / 127.0;
        (0.55 * amp + 0.35 * wave.abs() + 0.10 * transition.max(0.0)).clamp(0.0, 1.0)
    }
}

fn byte_to_i8(value: u64) -> i8 {
    (value as u8).wrapping_sub(128) as i8
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cell32_layout_is_exact() {
        assert_eq!(
            CELL32_HEADER_BYTES
                + CELL32_PROJECTION_BYTES
                + CELL32_MODE_BANK_BYTES
                + CELL32_TRANSITION_BYTES
                + CELL32_INTERFERENCE_BYTES
                + CELL32_CALIBRATION_BYTES
                + CELL32_SCRATCH_BYTES,
            CELL32_BYTES
        );
        assert_eq!(MODES_PER_CELL32, 2048);
    }
}
