use crate::stable_hash::mix64_golden;

pub(crate) const SURFACE_WAVE_DIM: usize = 4_096;
pub(crate) const SURFACE_WAVE_NGRAM: usize = 4;
pub(crate) const SURFACE_WAVE_TRITS: usize = 3;
const SHORT_TOKEN_IDENTITY_ATOMS: usize = 4;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct SurfaceAtom {
    pub(crate) position: u64,
    pub(crate) bytes: Vec<u8>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct SurfaceWaveTrit {
    pub(crate) lane: u16,
    pub(crate) value: i8,
}

#[must_use]
pub(crate) fn surface_atoms(text: &str) -> Vec<SurfaceAtom> {
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

        append_short_token_identity_atoms(&normalize_surface_token(raw_token), atoms);
    }
}

fn append_short_token_identity_atoms(token: &str, atoms: &mut Vec<SurfaceAtom>) {
    if token.is_empty() || token.chars().count() >= SURFACE_WAVE_NGRAM {
        return;
    }

    for salt in 0..SHORT_TOKEN_IDENTITY_ATOMS {
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

fn normalize_surface_token(token: &str) -> String {
    token
        .chars()
        .filter(|ch| ch.is_alphanumeric() || *ch == '_')
        .flat_map(char::to_lowercase)
        .collect()
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

fn encode_short_token_identity_atom(token: &str, salt: u8) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(10 + token.len());
    bytes.extend_from_slice(b"\x1Fsid\0");
    bytes.push(salt);
    bytes.extend_from_slice(token.as_bytes());
    bytes
}

#[must_use]
pub(crate) fn surface_atom_projection(
    position: u64,
    atom: &[u8],
) -> [SurfaceWaveTrit; SURFACE_WAVE_TRITS] {
    let seed = stable_hash_with_position(position, atom);
    [
        SurfaceWaveTrit {
            lane: (mix64_golden(seed) % SURFACE_WAVE_DIM as u64) as u16,
            value: if seed & 1 == 0 { 1 } else { -1 },
        },
        SurfaceWaveTrit {
            lane: (mix64_golden(seed ^ 0x9E37_79B9_7F4A_7C15) % SURFACE_WAVE_DIM as u64) as u16,
            value: if seed & 2 == 0 { 1 } else { -1 },
        },
        SurfaceWaveTrit {
            lane: (mix64_golden(seed ^ 0xD1B5_4A32_D192_ED03) % SURFACE_WAVE_DIM as u64) as u16,
            value: if seed & 4 == 0 { 1 } else { -1 },
        },
    ]
}

fn stable_hash_with_position(position: u64, bytes: &[u8]) -> u64 {
    let mut state = 0x5355_5246_4143_4531u64 ^ position.rotate_left(17);
    for byte in bytes {
        state ^= u64::from(*byte).wrapping_mul(0x9E37_79B9_7F4A_7C15);
        state = mix64_golden(state);
    }
    mix64_golden(state ^ bytes.len() as u64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn four_gram_atoms_cover_long_and_short_words() {
        assert_eq!(SURFACE_WAVE_NGRAM, 4);
        assert!(surface_atoms("проверка").len() >= 4);
        for word in ["и", "в", "не", "a", "to", "сыч"] {
            assert!(!surface_atoms(word).is_empty(), "word={word}");
        }
    }
}
