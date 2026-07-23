const BYTE_GRAM: usize = 4;
const CHARACTER_BIGRAM: usize = 2;
const CHARACTER_GRAM: usize = 3;
const KEYBOARD_BIGRAM: usize = 2;
const KEYBOARD_GRAM: usize = 3;
const MAX_BYTE_ATOMS: usize = 24;
const MAX_CHARACTER_BIGRAM_ATOMS: usize = 16;
const MAX_CHARACTER_ATOMS: usize = 24;
const MAX_KEYBOARD_BIGRAM_ATOMS: usize = 16;
const MAX_KEYBOARD_ATOMS: usize = 24;
const MAX_BAG_ATOMS: usize = 24;
const MAX_SKIP_ATOMS: usize = 32;
const POSITION_BUCKETS: usize = 16;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
pub(super) enum AtomChannel {
    ByteGram = 1,
    CharacterGram = 2,
    KeyboardGram = 3,
    BoundaryPosition = 4,
    CharacterBigram = 5,
    KeyboardBigram = 6,
    CharacterBagGram = 7,
    KeyboardBagGram = 8,
    CharacterSkipGram = 9,
    KeyboardSkipGram = 10,
    CharacterAnchor = 11,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(super) struct NGramKey {
    pub(super) channel: AtomChannel,
    pub(super) len: u8,
    pub(super) units: [u32; 4],
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(super) struct WaveSurfaceAtom {
    pub(super) key: NGramKey,
    pub(super) position: u16,
    pub(super) weight: u8,
}

pub(super) fn encode_wave_surface(text: &str) -> Vec<WaveSurfaceAtom> {
    let normalized = normalize_lexical_surface(text);
    let mut atoms = Vec::new();
    append_byte_wave_atoms(normalized.as_bytes(), &mut atoms);
    append_character_wave_atoms(&normalized, &mut atoms);
    append_keyboard_wave_atoms(&normalized, &mut atoms);
    append_boundary_wave_atoms(&normalized, &mut atoms);
    atoms.sort_unstable();
    atoms.dedup();
    atoms
}

pub(super) fn normalize_lexical_surface(text: &str) -> String {
    text.trim()
        .trim_matches(|ch: char| matches!(ch, '!' | ',' | '.' | '?' | ';' | ':'))
        .to_lowercase()
}

pub(super) fn physical_key_sequence(text: &str) -> Vec<u32> {
    let normalized = text.trim().to_lowercase();
    crate::keyboard::text_to_key_events(&normalized, false)
        .unwrap_or_default()
        .into_iter()
        .take(32)
        .map(|event| u32::from(event.keycode) | if event.shift { 1 << 16 } else { 0 })
        .collect()
}

pub(super) fn channel_weight(channel: AtomChannel) -> u16 {
    match channel {
        AtomChannel::ByteGram => 2,
        AtomChannel::CharacterGram | AtomChannel::KeyboardGram | AtomChannel::BoundaryPosition => 3,
        AtomChannel::CharacterBigram | AtomChannel::KeyboardBigram => 1,
        AtomChannel::CharacterBagGram | AtomChannel::KeyboardBagGram => 3,
        AtomChannel::CharacterSkipGram | AtomChannel::KeyboardSkipGram => 2,
        AtomChannel::CharacterAnchor => 1,
    }
}

fn append_byte_wave_atoms(bytes: &[u8], output: &mut Vec<WaveSurfaceAtom>) {
    if bytes.is_empty() {
        return;
    }
    if bytes.len() < BYTE_GRAM {
        push_wave_atom(
            output,
            AtomChannel::ByteGram,
            0,
            bytes.iter().copied().map(u32::from),
        );
        return;
    }
    let total = bytes.len() - BYTE_GRAM + 1;
    for position in sampled_positions(total, MAX_BYTE_ATOMS) {
        push_wave_atom(
            output,
            AtomChannel::ByteGram,
            relative_position(position, total),
            bytes[position..position + BYTE_GRAM]
                .iter()
                .copied()
                .map(u32::from),
        );
    }
}

fn append_character_wave_atoms(text: &str, output: &mut Vec<WaveSurfaceAtom>) {
    let mut units = vec![0x11_0001, 0x11_0001];
    units.extend(text.chars().map(|ch| ch as u32));
    units.extend([0x11_0002, 0x11_0002]);
    append_wave_unit_grams(
        &units,
        CHARACTER_BIGRAM,
        MAX_CHARACTER_BIGRAM_ATOMS,
        AtomChannel::CharacterBigram,
        output,
    );
    append_wave_unit_grams(
        &units,
        CHARACTER_GRAM,
        MAX_CHARACTER_ATOMS,
        AtomChannel::CharacterGram,
        output,
    );
    append_bag_grams(&units, AtomChannel::CharacterBagGram, output);
    append_skip_grams(&units, AtomChannel::CharacterSkipGram, output);
    append_anchor_atoms(
        &units[2..units.len() - 2],
        AtomChannel::CharacterAnchor,
        output,
    );
}

fn append_keyboard_wave_atoms(text: &str, output: &mut Vec<WaveSurfaceAtom>) {
    let physical = physical_key_sequence(text);
    if physical.is_empty() {
        return;
    }
    let mut units = vec![0x20_0001, 0x20_0001];
    units.extend(physical);
    units.extend([0x20_0002, 0x20_0002]);
    append_wave_unit_grams(
        &units,
        KEYBOARD_BIGRAM,
        MAX_KEYBOARD_BIGRAM_ATOMS,
        AtomChannel::KeyboardBigram,
        output,
    );
    append_wave_unit_grams(
        &units,
        KEYBOARD_GRAM,
        MAX_KEYBOARD_ATOMS,
        AtomChannel::KeyboardGram,
        output,
    );
    append_bag_grams(&units, AtomChannel::KeyboardBagGram, output);
    append_skip_grams(&units, AtomChannel::KeyboardSkipGram, output);
}

fn append_bag_grams(units: &[u32], channel: AtomChannel, output: &mut Vec<WaveSurfaceAtom>) {
    let total = units.len().saturating_sub(2);
    for position in sampled_positions(total, MAX_BAG_ATOMS) {
        let mut gram = [units[position], units[position + 1], units[position + 2]];
        gram.sort_unstable();
        push_wave_atom(output, channel, relative_position(position, total), gram);
    }
}

fn append_skip_grams(units: &[u32], channel: AtomChannel, output: &mut Vec<WaveSurfaceAtom>) {
    let mut emitted = 0;
    for distance in 2..=4 {
        let total = units.len().saturating_sub(distance);
        let remaining = MAX_SKIP_ATOMS.saturating_sub(emitted);
        for position in sampled_positions(total, remaining) {
            push_wave_atom(
                output,
                channel,
                relative_position(position, total),
                [units[position], units[position + distance]],
            );
            emitted += 1;
            if emitted >= MAX_SKIP_ATOMS {
                return;
            }
        }
    }
}

fn append_anchor_atoms(units: &[u32], channel: AtomChannel, output: &mut Vec<WaveSurfaceAtom>) {
    for (position, unit) in units.iter().copied().enumerate() {
        push_wave_atom(
            output,
            channel,
            sequence_position(position, units.len()),
            [unit],
        );
    }
}

fn append_wave_unit_grams(
    units: &[u32],
    gram: usize,
    budget: usize,
    channel: AtomChannel,
    output: &mut Vec<WaveSurfaceAtom>,
) {
    let total = units.len().saturating_sub(gram) + 1;
    for position in sampled_positions(total, budget) {
        push_wave_atom(
            output,
            channel,
            relative_position(position, total),
            units[position..position + gram].iter().copied(),
        );
    }
}

fn append_boundary_wave_atoms(text: &str, output: &mut Vec<WaveSurfaceAtom>) {
    let chars = text.chars().map(|ch| ch as u32).collect::<Vec<_>>();
    if chars.is_empty() {
        return;
    }
    let edge = chars.len().min(3);
    for length in 1..=edge {
        let mut begin = [0_u32; 4];
        begin[0] = 0x30_0001;
        begin[1..=length].copy_from_slice(&chars[..length]);
        output.push(WaveSurfaceAtom {
            key: NGramKey {
                channel: AtomChannel::BoundaryPosition,
                len: (length + 1) as u8,
                units: begin,
            },
            position: 0,
            weight: channel_weight(AtomChannel::BoundaryPosition) as u8,
        });
        let mut end = [0_u32; 4];
        end[0] = 0x30_0002;
        end[1..=length].copy_from_slice(&chars[chars.len() - length..]);
        output.push(WaveSurfaceAtom {
            key: NGramKey {
                channel: AtomChannel::BoundaryPosition,
                len: (length + 1) as u8,
                units: end,
            },
            position: u16::MAX,
            weight: channel_weight(AtomChannel::BoundaryPosition) as u8,
        });
    }
}

fn push_wave_atom(
    output: &mut Vec<WaveSurfaceAtom>,
    channel: AtomChannel,
    position: u16,
    units: impl IntoIterator<Item = u32>,
) {
    let mut packed = [0_u32; 4];
    let mut len = 0;
    for unit in units.into_iter().take(packed.len()) {
        packed[len] = unit;
        len += 1;
    }
    output.push(WaveSurfaceAtom {
        key: NGramKey {
            channel,
            len: len as u8,
            units: packed,
        },
        position,
        weight: channel_weight(channel) as u8,
    });
}

fn sampled_positions(total: usize, budget: usize) -> Vec<usize> {
    if total == 0 || budget == 0 {
        return Vec::new();
    }
    if budget == 1 {
        return vec![0];
    }
    if total <= budget {
        return (0..total).collect();
    }
    let mut positions = (0..budget)
        .map(|index| index * (total - 1) / (budget - 1))
        .collect::<Vec<_>>();
    positions.dedup();
    positions
}

fn relative_position(position: usize, total: usize) -> u16 {
    if total <= 1 {
        return 0;
    }
    let bucket = position.saturating_mul(POSITION_BUCKETS - 1) / (total - 1);
    (bucket.saturating_mul(u16::MAX as usize) / (POSITION_BUCKETS - 1)) as u16
}

fn sequence_position(position: usize, total: usize) -> u16 {
    if total <= 1 {
        0
    } else {
        (position.saturating_mul(u16::MAX as usize) / (total - 1)) as u16
    }
}
