//! Canonical surface-wave primitives for the NANDA L1/L2 shadow path.
//!
//! This mirrors the stable `nando-wave` L1 idea: surface form is represented as
//! UTF-8 byte 4-grams plus boundary/service atoms. It is deliberately below
//! semantics and below live IME behavior.

pub(super) const SURFACE_WAVE_DIM: usize = 4_096;
pub(super) const SURFACE_WAVE_NGRAM: usize = 4;
pub(super) const SURFACE_WAVE_TRITS: usize = 3;
const SURFACE_WAVE_SHORT_TOKEN_IDENTITY_ATOMS: usize = 4;
pub(super) const SURFACE_WAVE_BYTES: usize =
    SURFACE_WAVE_DIM * std::mem::size_of::<SurfaceWaveLane>();

type SurfaceWaveLane = i16;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct SurfaceAtom {
    pub(super) position: u64,
    pub(super) bytes: Vec<u8>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct SurfaceWaveTrit {
    pub(super) lane: u16,
    pub(super) value: i8,
}

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

#[must_use]
pub(super) fn surface_atoms(text: &str) -> Vec<SurfaceAtom> {
    let mut atoms = raw_byte_atoms(text.as_bytes());
    append_boundary_atoms(text, &mut atoms);
    atoms
}

fn raw_byte_atoms(bytes: &[u8]) -> Vec<SurfaceAtom> {
    if bytes.len() < SURFACE_WAVE_NGRAM {
        return Vec::new();
    }
    bytes
        .windows(SURFACE_WAVE_NGRAM)
        .enumerate()
        .map(|(position, gram)| SurfaceAtom {
            position: position as u64,
            bytes: gram.to_vec(),
        })
        .collect()
}

fn append_boundary_atoms(text: &str, atoms: &mut Vec<SurfaceAtom>) {
    for raw_token in text.split_whitespace() {
        let chars = lower_token_chars(raw_token);
        if chars.is_empty() {
            continue;
        }

        let mut padded = Vec::with_capacity(chars.len() + 2 * (SURFACE_WAVE_NGRAM - 1));
        for _ in 0..SURFACE_WAVE_NGRAM - 1 {
            padded.push(BoundarySlot::Begin);
        }
        padded.extend(chars.into_iter().map(BoundarySlot::Text));
        for _ in 0..SURFACE_WAVE_NGRAM - 1 {
            padded.push(BoundarySlot::End);
        }

        for (local_position, window) in padded.windows(SURFACE_WAVE_NGRAM).enumerate() {
            if !window
                .iter()
                .any(|slot| matches!(slot, BoundarySlot::Begin | BoundarySlot::End))
            {
                continue;
            }
            atoms.push(SurfaceAtom {
                position: local_position as u64,
                bytes: encode_boundary_atom(window),
            });
        }

        let service_token = normalize_service_token(raw_token);
        append_short_token_identity_atoms(&service_token, atoms);
        if is_service_word(&service_token) {
            atoms.push(SurfaceAtom {
                position: 0,
                bytes: encode_service_atom(&service_token),
            });
        }
    }
}

fn append_short_token_identity_atoms(token: &str, atoms: &mut Vec<SurfaceAtom>) {
    if token.is_empty() || token.chars().count() >= SURFACE_WAVE_NGRAM {
        return;
    }

    for salt in 0..SURFACE_WAVE_SHORT_TOKEN_IDENTITY_ATOMS {
        atoms.push(SurfaceAtom {
            position: 0,
            bytes: encode_short_token_identity_atom(token, salt as u8),
        });
    }
}

fn lower_token_chars(token: &str) -> Vec<char> {
    token
        .chars()
        .filter(|ch| !ch.is_control())
        .flat_map(char::to_lowercase)
        .collect()
}

fn normalize_service_token(token: &str) -> String {
    token
        .chars()
        .filter(|ch| ch.is_alphanumeric() || *ch == '_')
        .flat_map(char::to_lowercase)
        .collect()
}

fn is_service_word(token: &str) -> bool {
    matches!(
        token,
        "и" | "а"
            | "но"
            | "или"
            | "в"
            | "во"
            | "на"
            | "к"
            | "ко"
            | "с"
            | "со"
            | "из"
            | "у"
            | "о"
            | "об"
            | "от"
            | "до"
            | "по"
            | "за"
            | "над"
            | "под"
            | "при"
            | "для"
            | "без"
            | "не"
            | "ни"
            | "да"
            | "же"
            | "ли"
            | "бы"
            | "то"
            | "это"
            | "как"
            | "что"
            | "где"
            | "кто"
            | "мы"
            | "я"
            | "ты"
            | "он"
            | "она"
            | "они"
            | "the"
            | "a"
            | "an"
            | "and"
            | "or"
            | "but"
            | "not"
            | "no"
            | "to"
            | "of"
            | "in"
            | "on"
            | "at"
            | "by"
            | "for"
            | "from"
            | "with"
            | "as"
            | "is"
            | "are"
            | "was"
            | "were"
            | "be"
            | "do"
            | "does"
            | "did"
            | "if"
            | "then"
            | "than"
            | "that"
            | "this"
            | "it"
            | "we"
            | "you"
            | "he"
            | "she"
            | "they"
    )
}

#[derive(Clone, Copy, Debug)]
enum BoundarySlot {
    Begin,
    End,
    Text(char),
}

fn encode_boundary_atom(slots: &[BoundarySlot]) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(32);
    bytes.extend_from_slice(b"\x1Fbd4\0");
    for slot in slots {
        match slot {
            BoundarySlot::Begin => bytes.push(0x01),
            BoundarySlot::End => bytes.push(0x02),
            BoundarySlot::Text(ch) => {
                let mut buffer = [0u8; 4];
                let encoded = ch.encode_utf8(&mut buffer);
                bytes.push(0x10);
                bytes.push(encoded.len() as u8);
                bytes.extend_from_slice(encoded.as_bytes());
            }
        }
    }
    bytes
}

fn encode_service_atom(token: &str) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(8 + token.len());
    bytes.extend_from_slice(b"\x1Fsvc\0");
    bytes.extend_from_slice(token.as_bytes());
    bytes
}

fn encode_short_token_identity_atom(token: &str, salt: u8) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(10 + token.len());
    bytes.extend_from_slice(b"\x1Fsid\0");
    bytes.push(salt);
    bytes.extend_from_slice(token.as_bytes());
    bytes
}

#[must_use]
pub(super) fn surface_atom_projection(
    position: u64,
    atom: &[u8],
) -> [SurfaceWaveTrit; SURFACE_WAVE_TRITS] {
    let seed = stable_hash_with_position(position, atom);
    [
        SurfaceWaveTrit {
            lane: (mix64(seed) % SURFACE_WAVE_DIM as u64) as u16,
            value: if seed & 1 == 0 { 1 } else { -1 },
        },
        SurfaceWaveTrit {
            lane: (mix64(seed ^ 0x9E37_79B9_7F4A_7C15) % SURFACE_WAVE_DIM as u64) as u16,
            value: if seed & 2 == 0 { 1 } else { -1 },
        },
        SurfaceWaveTrit {
            lane: (mix64(seed ^ 0xD1B5_4A32_D192_ED03) % SURFACE_WAVE_DIM as u64) as u16,
            value: if seed & 4 == 0 { 1 } else { -1 },
        },
    ]
}

fn stable_hash_with_position(position: u64, bytes: &[u8]) -> u64 {
    let mut state = 0x5355_5246_4143_4531u64 ^ position.rotate_left(17);
    for byte in bytes {
        state ^= u64::from(*byte).wrapping_mul(0x9E37_79B9_7F4A_7C15);
        state = mix64(state);
    }
    mix64(state ^ bytes.len() as u64)
}

fn mix64(mut value: u64) -> u64 {
    value = value.wrapping_add(0x9E37_79B9_7F4A_7C15);
    value = (value ^ (value >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    value = (value ^ (value >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    value ^ (value >> 31)
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
        assert!(
            atoms.len() >= SURFACE_WAVE_SHORT_TOKEN_IDENTITY_ATOMS,
            "atoms={atoms:?}"
        );
        assert!(SurfaceWave4096::compile("сыч").active_lanes() > 0);
    }
}
