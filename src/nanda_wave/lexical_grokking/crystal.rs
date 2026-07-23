//! Fixed-width records for the crystallized L1 word memory.

pub(super) const WAVE_DIMENSION: usize = 128;
pub(super) const WORD_WAVE_COMPONENTS: usize = 22;
pub(super) const ATOM_WAVE_COMPONENTS: usize = 4;
pub(super) const WORD_CENTER_BYTES: usize = 64;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) struct BasisComponent8 {
    pub(super) basis: u8,
    pub(super) coefficient: i8,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) struct BasisComponent16 {
    pub(super) basis: u16,
    pub(super) coefficient: i16,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct WordCenter64 {
    pub(super) wave_code: [BasisComponent8; WORD_WAVE_COMPONENTS],
    pub(super) coupling_start: u32,
    pub(super) anti_start: u32,
    pub(super) decoder_terminal: u32,
    pub(super) coupling_count: u16,
    pub(super) crystal_support: u16,
    pub(super) anti_count: u8,
    pub(super) stability: u8,
    pub(super) surface_len: u8,
    pub(super) flags: u8,
}

impl Default for WordCenter64 {
    fn default() -> Self {
        Self {
            wave_code: [BasisComponent8::default(); WORD_WAVE_COMPONENTS],
            coupling_start: 0,
            anti_start: 0,
            decoder_terminal: 0,
            coupling_count: 0,
            crystal_support: 0,
            anti_count: 0,
            stability: 0,
            surface_len: 0,
            flags: 0,
        }
    }
}

impl WordCenter64 {
    pub(super) fn encode(self) -> [u8; WORD_CENTER_BYTES] {
        let mut bytes = [0_u8; WORD_CENTER_BYTES];
        for (index, component) in self.wave_code.into_iter().enumerate() {
            bytes[index * 2] = component.basis;
            bytes[index * 2 + 1] = component.coefficient as u8;
        }
        put_u32(&mut bytes, 44, self.coupling_start);
        put_u32(&mut bytes, 48, self.anti_start);
        put_u32(&mut bytes, 52, self.decoder_terminal);
        put_u16(&mut bytes, 56, self.coupling_count);
        put_u16(&mut bytes, 58, self.crystal_support);
        bytes[60] = self.anti_count;
        bytes[61] = self.stability;
        bytes[62] = self.surface_len;
        bytes[63] = self.flags;
        bytes
    }

    pub(super) fn decode(bytes: &[u8]) -> Result<Self, String> {
        if bytes.len() != WORD_CENTER_BYTES {
            return Err("WordCenter64 requires exactly 64 bytes".to_string());
        }
        let mut wave_code = [BasisComponent8::default(); WORD_WAVE_COMPONENTS];
        for (index, component) in wave_code.iter_mut().enumerate() {
            component.basis = bytes[index * 2];
            component.coefficient = bytes[index * 2 + 1] as i8;
        }
        Ok(Self {
            wave_code,
            coupling_start: read_u32(bytes, 44)?,
            anti_start: read_u32(bytes, 48)?,
            decoder_terminal: read_u32(bytes, 52)?,
            coupling_count: read_u16(bytes, 56)?,
            crystal_support: read_u16(bytes, 58)?,
            anti_count: bytes[60],
            stability: bytes[61],
            surface_len: bytes[62],
            flags: bytes[63],
        })
    }
}

/// Role-specific view of a fixed-width center used for a directed ambiguity
/// relation. Ambiguity centers have no coupling span, so bytes 56..58 store
/// the learned coherence threshold without changing the 64-byte wire record.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct AmbiguityPhaseCenter64(WordCenter64);

impl AmbiguityPhaseCenter64 {
    pub(super) fn from_record(record: WordCenter64) -> Self {
        Self(record)
    }

    pub(super) fn threshold_milli(self) -> u16 {
        self.0.coupling_count
    }

    pub(super) fn with_threshold_milli(mut self, threshold_milli: u16) -> Self {
        self.0.coupling_count = threshold_milli;
        self
    }

    pub(super) fn record(self) -> WordCenter64 {
        self.0
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) struct AtomWaveCode {
    pub(super) components: [BasisComponent16; ATOM_WAVE_COMPONENTS],
}

impl AtomWaveCode {
    pub(super) const BYTES: usize = 16;

    pub(super) fn encode(self) -> [u8; Self::BYTES] {
        let mut bytes = [0_u8; Self::BYTES];
        for (index, component) in self.components.into_iter().enumerate() {
            put_u16(&mut bytes, index * 4, component.basis);
            put_i16(&mut bytes, index * 4 + 2, component.coefficient);
        }
        bytes
    }

    pub(super) fn decode(bytes: &[u8]) -> Result<Self, String> {
        if bytes.len() != Self::BYTES {
            return Err("AtomWaveCode requires exactly 16 bytes".to_string());
        }
        let mut components = [BasisComponent16::default(); ATOM_WAVE_COMPONENTS];
        for (index, component) in components.iter_mut().enumerate() {
            component.basis = read_u16(bytes, index * 4)?;
            component.coefficient = read_i16(bytes, index * 4 + 2)?;
        }
        Ok(Self { components })
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct ComplexBasisWave {
    pub(super) re: [i8; WAVE_DIMENSION],
    pub(super) im: [i8; WAVE_DIMENSION],
}

impl Default for ComplexBasisWave {
    fn default() -> Self {
        Self {
            re: [0; WAVE_DIMENSION],
            im: [0; WAVE_DIMENSION],
        }
    }
}

fn put_u16(bytes: &mut [u8], offset: usize, value: u16) {
    bytes[offset..offset + 2].copy_from_slice(&value.to_le_bytes());
}

fn put_i16(bytes: &mut [u8], offset: usize, value: i16) {
    bytes[offset..offset + 2].copy_from_slice(&value.to_le_bytes());
}

fn put_u32(bytes: &mut [u8], offset: usize, value: u32) {
    bytes[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
}

fn read_u16(bytes: &[u8], offset: usize) -> Result<u16, String> {
    let raw = bytes
        .get(offset..offset + 2)
        .ok_or_else(|| "truncated u16".to_string())?;
    Ok(u16::from_le_bytes([raw[0], raw[1]]))
}

fn read_i16(bytes: &[u8], offset: usize) -> Result<i16, String> {
    let raw = bytes
        .get(offset..offset + 2)
        .ok_or_else(|| "truncated i16".to_string())?;
    Ok(i16::from_le_bytes([raw[0], raw[1]]))
}

fn read_u32(bytes: &[u8], offset: usize) -> Result<u32, String> {
    let raw = bytes
        .get(offset..offset + 4)
        .ok_or_else(|| "truncated u32".to_string())?;
    Ok(u32::from_le_bytes([raw[0], raw[1], raw[2], raw[3]]))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn word_center_contract_is_exactly_sixty_four_bytes() {
        let mut center = WordCenter64 {
            coupling_start: 11,
            coupling_count: 12,
            anti_start: 13,
            anti_count: 14,
            crystal_support: 15,
            stability: 16,
            surface_len: 17,
            decoder_terminal: 18,
            flags: 19,
            ..WordCenter64::default()
        };
        center.wave_code[0] = BasisComponent8 {
            basis: 7,
            coefficient: -31,
        };
        let bytes = center.encode();
        assert_eq!(bytes.len(), WORD_CENTER_BYTES);
        assert_eq!(WordCenter64::decode(&bytes), Ok(center));
    }

    #[test]
    fn ambiguity_center_reuses_the_fixed_record_without_changing_its_size() {
        let relation = AmbiguityPhaseCenter64::from_record(WordCenter64 {
            coupling_count: 731,
            ..WordCenter64::default()
        });
        assert_eq!(
            std::mem::size_of::<AmbiguityPhaseCenter64>(),
            WORD_CENTER_BYTES
        );
        assert_eq!(relation.threshold_milli(), 731);
        assert_eq!(relation.record().coupling_count, 731);
    }

    #[test]
    fn atom_wave_code_is_exactly_sixteen_bytes() {
        let mut code = AtomWaveCode::default();
        code.components[2] = BasisComponent16 {
            basis: 91,
            coefficient: -7_123,
        };
        let bytes = code.encode();
        assert_eq!(bytes.len(), AtomWaveCode::BYTES);
        assert_eq!(AtomWaveCode::decode(&bytes), Ok(code));
    }
}
