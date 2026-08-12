use std::collections::{HashMap, HashSet};
use std::hash::{BuildHasherDefault, Hasher};
use std::sync::Arc;

use super::super::compositional::{
    append_atom_family, atom_weight, hash_atom, normalize_surface, typed_atom_key,
};
#[cfg(test)]
use super::super::compositional::{
    prepared_similarity_to_normalized_surface_with_workspace_milli, surface_scoring_profile,
    SurfaceGeometryWorkspace,
};

const MAX_GEOMETRY_UNITS: usize = u16::MAX as usize - 1;

const CHARACTER_START_MARKER: u32 = 0x11_0001;
const CHARACTER_END_MARKER: u32 = 0x11_0002;
const KEYBOARD_START_MARKER: u32 = 0x20_0001;
const KEYBOARD_END_MARKER: u32 = 0x20_0002;

const CHARACTER_BIGRAM_CHANNEL: u8 = 1;
const CHARACTER_TRIGRAM_CHANNEL: u8 = 2;
const KEYBOARD_BIGRAM_CHANNEL: u8 = 3;
const KEYBOARD_TRIGRAM_CHANNEL: u8 = 4;
const CHARACTER_BAG_TRIGRAM_CHANNEL: u8 = 5;
const KEYBOARD_BAG_TRIGRAM_CHANNEL: u8 = 6;
const CHARACTER_SKIP_GRAM_CHANNEL: u8 = 7;
const KEYBOARD_SKIP_GRAM_CHANNEL: u8 = 8;
const SIMHASH_COUNTER_PLANES: usize = 20;

#[derive(Clone, Default)]
struct AtomKeyHasher(u64);

impl Hasher for AtomKeyHasher {
    fn finish(&self) -> u64 {
        self.0
    }

    fn write(&mut self, bytes: &[u8]) {
        self.0 = bytes.iter().fold(0xcbf2_9ce4_8422_2325_u64, |hash, byte| {
            (hash ^ u64::from(*byte)).wrapping_mul(0x100_0000_01b3)
        });
    }

    fn write_u64(&mut self, value: u64) {
        self.0 = value;
    }
}

type AtomKeyBuildHasher = BuildHasherDefault<AtomKeyHasher>;
type AtomKeySet = HashSet<u64, AtomKeyBuildHasher>;
type AtomRefcountMap = HashMap<u64, u32, AtomKeyBuildHasher>;

#[derive(Clone, Copy, Debug)]
struct AtomLaneConfigV1 {
    start_marker: u32,
    end_marker: u32,
    bigram_channel: u8,
    trigram_channel: u8,
    bag_channel: u8,
    skip_channel: u8,
    simhash_domain: u64,
}

const CHARACTER_ATOM_CONFIG: AtomLaneConfigV1 = AtomLaneConfigV1 {
    start_marker: CHARACTER_START_MARKER,
    end_marker: CHARACTER_END_MARKER,
    bigram_channel: CHARACTER_BIGRAM_CHANNEL,
    trigram_channel: CHARACTER_TRIGRAM_CHANNEL,
    bag_channel: CHARACTER_BAG_TRIGRAM_CHANNEL,
    skip_channel: CHARACTER_SKIP_GRAM_CHANNEL,
    simhash_domain: 1,
};

const KEYBOARD_ATOM_CONFIG: AtomLaneConfigV1 = AtomLaneConfigV1 {
    start_marker: KEYBOARD_START_MARKER,
    end_marker: KEYBOARD_END_MARKER,
    bigram_channel: KEYBOARD_BIGRAM_CHANNEL,
    trigram_channel: KEYBOARD_TRIGRAM_CHANNEL,
    bag_channel: KEYBOARD_BAG_TRIGRAM_CHANNEL,
    skip_channel: KEYBOARD_SKIP_GRAM_CHANNEL,
    simhash_domain: 2,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct ObservedGeometryV1 {
    pub(super) normalized: String,
    pub(super) characters: Vec<u32>,
    pub(super) keyboard: Vec<u32>,
}

impl ObservedGeometryV1 {
    pub(super) fn new(surface: &str) -> Result<Self, &'static str> {
        let normalized = normalize_surface(surface);
        let characters = normalized.chars().map(u32::from).collect::<Vec<_>>();
        let keyboard = physical_keys(&normalized);
        checked_unit_len(characters.len())?;
        checked_unit_len(keyboard.len())?;
        Ok(Self {
            normalized,
            characters,
            keyboard,
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct OsaLaneV1 {
    observed: Vec<u32>,
    previous_row: Vec<u16>,
    current_row: Vec<u16>,
    next_row: Vec<u16>,
    previous_generated_unit: Option<u32>,
    generated_length: u16,
}

impl OsaLaneV1 {
    fn new(observed: &[u32]) -> Result<Self, &'static str> {
        checked_unit_len(observed.len())?;
        let current_row = (0..=observed.len())
            .map(|value| u16::try_from(value).expect("checked observed length"))
            .collect::<Vec<_>>();
        Ok(Self {
            observed: observed.to_vec(),
            previous_row: current_row.clone(),
            next_row: vec![0_u16; current_row.len()],
            current_row,
            previous_generated_unit: None,
            generated_length: 0,
        })
    }

    fn reset_generated(&mut self) {
        self.current_row
            .iter_mut()
            .enumerate()
            .for_each(|(index, value)| *value = index as u16);
        self.previous_row.clone_from(&self.current_row);
        self.previous_generated_unit = None;
        self.generated_length = 0;
    }

    fn emit(&mut self, generated: u32) -> Result<(), &'static str> {
        if usize::from(self.generated_length) >= MAX_GEOMETRY_UNITS {
            return Err("generated OSA path reaches the u16 wire ceiling");
        }
        let next_length = self
            .generated_length
            .checked_add(1)
            .ok_or("generated OSA length overflow")?;
        self.next_row[0] = next_length;
        for observed_index in 1..=self.observed.len() {
            let substitution = self.current_row[observed_index - 1]
                .saturating_add(u16::from(generated != self.observed[observed_index - 1]));
            let insertion = self.next_row[observed_index - 1].saturating_add(1);
            let deletion = self.current_row[observed_index].saturating_add(1);
            let mut distance = substitution.min(insertion).min(deletion);
            if self.generated_length > 0
                && observed_index > 1
                && generated == self.observed[observed_index - 2]
                && self.previous_generated_unit == Some(self.observed[observed_index - 1])
            {
                distance = distance.min(self.previous_row[observed_index - 2].saturating_add(1));
            }
            self.next_row[observed_index] = distance;
        }
        std::mem::swap(&mut self.previous_row, &mut self.current_row);
        std::mem::swap(&mut self.current_row, &mut self.next_row);
        self.previous_generated_unit = Some(generated);
        self.generated_length = next_length;
        Ok(())
    }

    fn distance(&self) -> u16 {
        self.current_row
            .last()
            .copied()
            .unwrap_or(self.generated_length)
    }

    fn similarity_milli(&self) -> u16 {
        let denominator = self.observed.len().max(usize::from(self.generated_length));
        if denominator == 0 {
            return 1_000;
        }
        let distance = usize::from(self.distance()).min(denominator);
        ((denominator - distance) * 1_000 / denominator) as u16
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) struct GeometryPathIdentityV1 {
    pub(super) lemma_id: u32,
    pub(super) paradigm_id: u32,
    pub(super) slot_id: u32,
    pub(super) program_id: u32,
    pub(super) variant_id: u16,
    pub(super) decoder_trace_ref: u32,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) struct GeometryTerminalEvidenceV1 {
    pub(super) character_distance: u16,
    pub(super) keyboard_distance: u16,
    pub(super) character_similarity_milli: u16,
    pub(super) keyboard_similarity_milli: u16,
    pub(super) geometry_milli: u16,
    pub(super) atom_similarity_milli: u16,
    pub(super) character_simhash: u64,
    pub(super) keyboard_simhash: u64,
}

#[derive(Clone, Debug)]
pub(super) struct GeometryTraversalStateV1 {
    character: OsaLaneV1,
    keyboard: OsaLaneV1,
    keyboard_normalization_valid: bool,
    keyboard_events: Vec<crate::keyboard::KeyEvent>,
    atoms: AtomAccumulatorV1,
    pub(super) identity: GeometryPathIdentityV1,
}

impl GeometryTraversalStateV1 {
    pub(super) fn new(
        observed: &ObservedGeometryV1,
        identity: GeometryPathIdentityV1,
    ) -> Result<Self, &'static str> {
        Ok(Self {
            character: OsaLaneV1::new(&observed.characters)?,
            keyboard: OsaLaneV1::new(&observed.keyboard)?,
            keyboard_normalization_valid: true,
            keyboard_events: Vec::with_capacity(observed.characters.len()),
            atoms: AtomAccumulatorV1::new(observed),
            identity,
        })
    }

    pub(super) fn emit_normalized_scalar(&mut self, scalar: char) -> Result<(), &'static str> {
        let scalar_unit = u32::from(scalar);
        self.character.emit(scalar_unit)?;
        self.atoms.emit_character(scalar_unit)?;

        if self.keyboard_normalization_valid {
            let mut encoded = [0_u8; 4];
            let scalar_text = scalar.encode_utf8(&mut encoded);
            match crate::keyboard::text_to_key_events_into(
                scalar_text,
                false,
                &mut self.keyboard_events,
            ) {
                Some(()) => {
                    for event in self.keyboard_events.iter().copied() {
                        let unit = u32::from(event.keycode) | (u32::from(event.shift) << 16);
                        self.keyboard.emit(unit)?;
                        self.atoms.emit_keyboard(unit)?;
                    }
                }
                None => {
                    self.keyboard_normalization_valid = false;
                    self.keyboard.reset_generated();
                    self.atoms.reset_keyboard();
                }
            }
        }
        Ok(())
    }

    pub(super) fn emit_normalized_str(&mut self, normalized: &str) -> Result<(), &'static str> {
        for scalar in normalized.chars() {
            let scalar_unit = u32::from(scalar);
            self.character.emit(scalar_unit)?;
            self.atoms.emit_character(scalar_unit)?;
        }
        if self.keyboard_normalization_valid {
            match crate::keyboard::text_to_key_events_into(
                normalized,
                false,
                &mut self.keyboard_events,
            ) {
                Some(()) => {
                    for event in self.keyboard_events.iter().copied() {
                        let unit = u32::from(event.keycode) | (u32::from(event.shift) << 16);
                        self.keyboard.emit(unit)?;
                        self.atoms.emit_keyboard(unit)?;
                    }
                }
                None => {
                    self.keyboard_normalization_valid = false;
                    self.keyboard.reset_generated();
                    self.atoms.reset_keyboard();
                }
            }
        }
        Ok(())
    }

    pub(super) fn reset_generated(&mut self) {
        self.character.reset_generated();
        self.keyboard.reset_generated();
        self.keyboard_normalization_valid = true;
        self.atoms.reset_generated();
    }

    pub(super) fn terminal_evidence(&mut self) -> GeometryTerminalEvidenceV1 {
        let atom = self.atoms.terminal_evidence();
        let character_similarity_milli = self.character.similarity_milli();
        let keyboard_similarity_milli = self.keyboard.similarity_milli();
        GeometryTerminalEvidenceV1 {
            character_distance: self.character.distance(),
            keyboard_distance: self.keyboard.distance(),
            character_similarity_milli,
            keyboard_similarity_milli,
            geometry_milli: character_similarity_milli.max(keyboard_similarity_milli),
            atom_similarity_milli: atom.similarity_milli,
            character_simhash: atom.character_simhash,
            keyboard_simhash: atom.keyboard_simhash,
        }
    }
}

#[derive(Debug)]
pub(super) struct BatchGeometryEvaluatorV1 {
    character: OsaLaneV1,
    keyboard: OsaLaneV1,
    observed_atoms: Vec<u64>,
    observed_atom_weight: u32,
    characters: Vec<u32>,
    keyboard_units: Vec<u32>,
    keyboard_events: Vec<crate::keyboard::KeyEvent>,
    generated_atoms: Vec<u64>,
    padded_units: Vec<u32>,
    simhash_atoms: Vec<u64>,
    evidence_by_surface: HashMap<Arc<str>, CachedGeometryEvidenceV1>,
}

#[derive(Clone, Copy, Debug)]
struct CachedGeometryEvidenceV1 {
    evidence: GeometryTerminalEvidenceV1,
    simhash_ready: bool,
}

impl BatchGeometryEvaluatorV1 {
    pub(super) fn new(observed: &ObservedGeometryV1) -> Result<Self, &'static str> {
        let mut observed_atoms = Vec::with_capacity(
            observed
                .characters
                .len()
                .saturating_add(observed.keyboard.len())
                .saturating_mul(16)
                .saturating_add(32),
        );
        append_atom_family(
            &observed.characters,
            CHARACTER_ATOM_CONFIG.start_marker,
            CHARACTER_ATOM_CONFIG.end_marker,
            CHARACTER_ATOM_CONFIG.bigram_channel,
            CHARACTER_ATOM_CONFIG.trigram_channel,
            CHARACTER_ATOM_CONFIG.bag_channel,
            CHARACTER_ATOM_CONFIG.skip_channel,
            &mut observed_atoms,
        );
        append_atom_family(
            &observed.keyboard,
            KEYBOARD_ATOM_CONFIG.start_marker,
            KEYBOARD_ATOM_CONFIG.end_marker,
            KEYBOARD_ATOM_CONFIG.bigram_channel,
            KEYBOARD_ATOM_CONFIG.trigram_channel,
            KEYBOARD_ATOM_CONFIG.bag_channel,
            KEYBOARD_ATOM_CONFIG.skip_channel,
            &mut observed_atoms,
        );
        observed_atoms.sort_unstable();
        observed_atoms.dedup();
        let observed_atom_weight = observed_atoms
            .iter()
            .copied()
            .map(atom_weight)
            .map(u32::from)
            .sum();
        let workspace_capacity = observed.characters.len().saturating_mul(2).max(32);
        Ok(Self {
            character: OsaLaneV1::new(&observed.characters)?,
            keyboard: OsaLaneV1::new(&observed.keyboard)?,
            observed_atoms,
            observed_atom_weight,
            characters: Vec::with_capacity(workspace_capacity),
            keyboard_units: Vec::with_capacity(workspace_capacity),
            keyboard_events: Vec::with_capacity(workspace_capacity),
            generated_atoms: Vec::with_capacity(workspace_capacity.saturating_mul(16)),
            padded_units: Vec::with_capacity(workspace_capacity.saturating_add(4)),
            simhash_atoms: Vec::with_capacity(workspace_capacity.saturating_mul(8)),
            evidence_by_surface: HashMap::new(),
        })
    }

    pub(super) fn evaluate(
        &mut self,
        normalized: &str,
    ) -> Result<GeometryTerminalEvidenceV1, &'static str> {
        let _ = self.evaluate_for_ranking(normalized)?;
        if let Some(cached) = self
            .evidence_by_surface
            .get(normalized)
            .filter(|cached| cached.simhash_ready)
        {
            return Ok(cached.evidence);
        }
        self.prepare_generated_units(normalized)?;
        let character_simhash = batch_simhash(
            &self.characters,
            CHARACTER_ATOM_CONFIG.simhash_domain,
            &mut self.padded_units,
            &mut self.simhash_atoms,
        );
        let keyboard_simhash = batch_simhash(
            &self.keyboard_units,
            KEYBOARD_ATOM_CONFIG.simhash_domain,
            &mut self.padded_units,
            &mut self.simhash_atoms,
        );
        let cached = self
            .evidence_by_surface
            .get_mut(normalized)
            .expect("ranking evidence inserted before simhash completion");
        cached.evidence.character_simhash = character_simhash;
        cached.evidence.keyboard_simhash = keyboard_simhash;
        cached.simhash_ready = true;
        Ok(cached.evidence)
    }

    pub(super) fn evaluate_for_ranking(
        &mut self,
        normalized: &str,
    ) -> Result<GeometryTerminalEvidenceV1, &'static str> {
        self.evaluate_for_ranking_interned(normalized)
            .map(|(evidence, _)| evidence)
    }

    pub(super) fn evaluate_for_ranking_interned(
        &mut self,
        normalized: &str,
    ) -> Result<(GeometryTerminalEvidenceV1, Arc<str>), &'static str> {
        if let Some((surface, cached)) = self.evidence_by_surface.get_key_value(normalized) {
            let mut evidence = cached.evidence;
            evidence.character_simhash = 0;
            evidence.keyboard_simhash = 0;
            return Ok((evidence, Arc::clone(surface)));
        }
        let evidence = self.evaluate_uncached_for_ranking(normalized)?;
        let surface = Arc::<str>::from(normalized);
        self.evidence_by_surface.insert(
            Arc::clone(&surface),
            CachedGeometryEvidenceV1 {
                evidence,
                simhash_ready: false,
            },
        );
        Ok((evidence, surface))
    }

    fn prepare_generated_units(&mut self, normalized: &str) -> Result<(), &'static str> {
        self.characters.clear();
        self.characters.extend(normalized.chars().map(u32::from));
        checked_unit_len(self.characters.len())?;

        self.keyboard_units.clear();
        if crate::keyboard::text_to_key_events_into(normalized, false, &mut self.keyboard_events)
            .is_some()
        {
            self.keyboard_units.extend(
                self.keyboard_events
                    .iter()
                    .map(|event| u32::from(event.keycode) | (u32::from(event.shift) << 16)),
            );
        }
        checked_unit_len(self.keyboard_units.len())?;
        Ok(())
    }

    fn evaluate_uncached_for_ranking(
        &mut self,
        normalized: &str,
    ) -> Result<GeometryTerminalEvidenceV1, &'static str> {
        self.prepare_generated_units(normalized)?;

        self.character.reset_generated();
        for unit in self.characters.iter().copied() {
            self.character.emit(unit)?;
        }
        self.keyboard.reset_generated();
        for unit in self.keyboard_units.iter().copied() {
            self.keyboard.emit(unit)?;
        }

        self.generated_atoms.clear();
        append_atom_family_reused(
            &self.characters,
            CHARACTER_ATOM_CONFIG,
            &mut self.padded_units,
            &mut self.generated_atoms,
        );
        append_atom_family_reused(
            &self.keyboard_units,
            KEYBOARD_ATOM_CONFIG,
            &mut self.padded_units,
            &mut self.generated_atoms,
        );
        self.generated_atoms.sort_unstable();
        self.generated_atoms.dedup();
        let generated_atom_weight = self
            .generated_atoms
            .iter()
            .copied()
            .map(atom_weight)
            .map(u32::from)
            .sum::<u32>();
        let shared_atom_weight =
            sorted_shared_atom_weight(&self.observed_atoms, &self.generated_atoms);
        let atom_denominator = self
            .observed_atom_weight
            .saturating_add(generated_atom_weight);
        let atom_similarity_milli = if atom_denominator == 0 {
            1_000
        } else {
            shared_atom_weight
                .saturating_mul(2_000)
                .checked_div(atom_denominator)
                .unwrap_or_default()
                .min(1_000) as u16
        };

        let character_similarity_milli = self.character.similarity_milli();
        let keyboard_similarity_milli = self.keyboard.similarity_milli();
        Ok(GeometryTerminalEvidenceV1 {
            character_distance: self.character.distance(),
            keyboard_distance: self.keyboard.distance(),
            character_similarity_milli,
            keyboard_similarity_milli,
            geometry_milli: character_similarity_milli.max(keyboard_similarity_milli),
            atom_similarity_milli,
            character_simhash: 0,
            keyboard_simhash: 0,
        })
    }
}

fn append_atom_family_reused(
    units: &[u32],
    config: AtomLaneConfigV1,
    padded: &mut Vec<u32>,
    output: &mut Vec<u64>,
) {
    if units.is_empty() {
        return;
    }
    padded.clear();
    padded.extend([config.start_marker, config.start_marker]);
    padded.extend_from_slice(units);
    padded.extend([config.end_marker, config.end_marker]);
    for window in padded.windows(2) {
        output.push(typed_atom_key(
            config.bigram_channel,
            u64::from(config.bigram_channel),
            window,
        ));
    }
    for window in padded.windows(3) {
        output.push(typed_atom_key(
            config.trigram_channel,
            u64::from(config.trigram_channel),
            window,
        ));
        let bag = sort_three([window[0], window[1], window[2]]);
        output.push(typed_atom_key(
            config.bag_channel,
            u64::from(config.bag_channel),
            &bag,
        ));
    }
    for distance in 2..=4 {
        for position in 0..padded.len().saturating_sub(distance) {
            output.push(typed_atom_key(
                config.skip_channel,
                u64::from(config.skip_channel) * 16 + distance as u64,
                &[padded[position], padded[position + distance]],
            ));
        }
    }
}

fn sorted_shared_atom_weight(observed: &[u64], generated: &[u64]) -> u32 {
    let mut observed_index = 0_usize;
    let mut generated_index = 0_usize;
    let mut shared_weight = 0_u32;
    while observed_index < observed.len() && generated_index < generated.len() {
        match observed[observed_index].cmp(&generated[generated_index]) {
            std::cmp::Ordering::Less => observed_index += 1,
            std::cmp::Ordering::Greater => generated_index += 1,
            std::cmp::Ordering::Equal => {
                shared_weight =
                    shared_weight.saturating_add(u32::from(atom_weight(observed[observed_index])));
                observed_index += 1;
                generated_index += 1;
            }
        }
    }
    shared_weight
}

fn batch_simhash(units: &[u32], domain: u64, padded: &mut Vec<u32>, atoms: &mut Vec<u64>) -> u64 {
    if units.is_empty() {
        return 0;
    }
    padded.clear();
    padded.push(0x11_0000 + domain as u32);
    padded.extend_from_slice(units);
    padded.push(0x12_0000 + domain as u32);
    atoms.clear();
    for gram in [2_usize, 3] {
        for window in padded.windows(gram) {
            atoms.push(hash_atom(domain * 16 + gram as u64, window));
        }
    }
    for distance in 2..=4 {
        for position in 0..padded.len().saturating_sub(distance) {
            atoms.push(hash_atom(
                domain * 16 + 8 + distance as u64,
                &[padded[position], padded[position + distance]],
            ));
        }
    }
    for window in padded.windows(3) {
        let bag = sort_three([window[0], window[1], window[2]]);
        atoms.push(hash_atom(domain * 16 + 15, &bag));
    }
    atoms.sort_unstable();
    atoms.dedup();
    simhash_from_sorted_unique_atoms(atoms)
}

fn simhash_from_sorted_unique_atoms(atoms: &[u64]) -> u64 {
    let mut planes = [0_u64; SIMHASH_COUNTER_PLANES];
    for atom in atoms.iter().copied() {
        let mut carry = atom;
        for plane in &mut planes {
            let next_carry = *plane & carry;
            *plane ^= carry;
            carry = next_carry;
        }
        debug_assert_eq!(carry, 0, "simhash bit-sliced counter overflow");
    }
    let threshold = atoms.len().div_ceil(2);
    debug_assert!(threshold < (1_usize << SIMHASH_COUNTER_PLANES));
    let mut greater = 0_u64;
    let mut equal = u64::MAX;
    for (bit, plane) in planes.iter().enumerate().rev() {
        if threshold & (1_usize << bit) == 0 {
            greater |= equal & *plane;
            equal &= !*plane;
        } else {
            equal &= *plane;
        }
    }
    greater | equal
}

#[inline]
fn sort_three(mut values: [u32; 3]) -> [u32; 3] {
    if values[0] > values[1] {
        values.swap(0, 1);
    }
    if values[1] > values[2] {
        values.swap(1, 2);
    }
    if values[0] > values[1] {
        values.swap(0, 1);
    }
    values
}

#[derive(Clone, Copy, Debug)]
struct AtomLaneCheckpointV1 {
    padded_len: usize,
    atom_undo_len: usize,
    real_units: u16,
    simhash_padded_len: usize,
    simhash_undo_len: usize,
}

#[derive(Clone, Debug)]
struct AtomLaneAccumulatorV1 {
    config: AtomLaneConfigV1,
    observed_atoms: AtomKeySet,
    observed_weight: u32,
    generated_atom_refcounts: AtomRefcountMap,
    generated_weight: u32,
    shared_weight: u32,
    padded: Vec<u32>,
    atom_undo: Vec<u64>,
    real_units: u16,
    simhash_padded: Vec<u32>,
    simhash_refcounts: AtomRefcountMap,
    simhash_support: [i32; 64],
    simhash_undo: Vec<u64>,
}

impl AtomLaneAccumulatorV1 {
    fn new(observed: &[u32], config: AtomLaneConfigV1) -> Self {
        let mut atoms = Vec::new();
        append_atom_family(
            observed,
            config.start_marker,
            config.end_marker,
            config.bigram_channel,
            config.trigram_channel,
            config.bag_channel,
            config.skip_channel,
            &mut atoms,
        );
        atoms.sort_unstable();
        atoms.dedup();
        let mut observed_atoms =
            AtomKeySet::with_capacity_and_hasher(atoms.len(), AtomKeyBuildHasher::default());
        observed_atoms.extend(atoms);
        let observed_weight = observed_atoms
            .iter()
            .copied()
            .map(atom_weight)
            .map(u32::from)
            .sum();
        Self {
            config,
            observed_atoms,
            observed_weight,
            generated_atom_refcounts: AtomRefcountMap::with_capacity_and_hasher(
                128,
                AtomKeyBuildHasher::default(),
            ),
            generated_weight: 0,
            shared_weight: 0,
            padded: Vec::new(),
            atom_undo: Vec::new(),
            real_units: 0,
            simhash_padded: Vec::new(),
            simhash_refcounts: AtomRefcountMap::with_capacity_and_hasher(
                128,
                AtomKeyBuildHasher::default(),
            ),
            simhash_support: [0; 64],
            simhash_undo: Vec::new(),
        }
    }

    fn reset_generated(&mut self) {
        self.generated_atom_refcounts.clear();
        self.generated_weight = 0;
        self.shared_weight = 0;
        self.padded.clear();
        self.atom_undo.clear();
        self.real_units = 0;
        self.simhash_padded.clear();
        self.simhash_refcounts.clear();
        self.simhash_support.fill(0);
        self.simhash_undo.clear();
    }

    fn checkpoint(&self) -> AtomLaneCheckpointV1 {
        AtomLaneCheckpointV1 {
            padded_len: self.padded.len(),
            atom_undo_len: self.atom_undo.len(),
            real_units: self.real_units,
            simhash_padded_len: self.simhash_padded.len(),
            simhash_undo_len: self.simhash_undo.len(),
        }
    }

    fn restore(&mut self, checkpoint: AtomLaneCheckpointV1) {
        while self.atom_undo.len() > checkpoint.atom_undo_len {
            let atom = self.atom_undo.pop().expect("checked undo length");
            let remove = {
                let count = self
                    .generated_atom_refcounts
                    .get_mut(&atom)
                    .expect("undo atom exists");
                *count -= 1;
                *count == 0
            };
            if remove {
                self.generated_atom_refcounts.remove(&atom);
                self.generated_weight -= u32::from(atom_weight(atom));
                if self.observed_atoms.contains(&atom) {
                    self.shared_weight -= u32::from(atom_weight(atom));
                }
            }
        }
        while self.simhash_undo.len() > checkpoint.simhash_undo_len {
            let atom = self
                .simhash_undo
                .pop()
                .expect("checked simhash undo length");
            let remove = {
                let count = self
                    .simhash_refcounts
                    .get_mut(&atom)
                    .expect("undo simhash atom exists");
                *count -= 1;
                *count == 0
            };
            if remove {
                self.simhash_refcounts.remove(&atom);
                for (bit, support) in self.simhash_support.iter_mut().enumerate() {
                    *support -= if atom & (1_u64 << bit) == 0 { -1 } else { 1 };
                }
            }
        }
        self.padded.truncate(checkpoint.padded_len);
        self.simhash_padded.truncate(checkpoint.simhash_padded_len);
        self.real_units = checkpoint.real_units;
    }

    fn emit_real(&mut self, unit: u32) -> Result<(), &'static str> {
        if usize::from(self.real_units) >= MAX_GEOMETRY_UNITS {
            return Err("atom path reaches the u16 wire ceiling");
        }
        if self.real_units == 0 {
            self.padded
                .extend([self.config.start_marker, self.config.start_marker]);
            self.add_typed_atom(super::super::compositional::typed_atom_key(
                self.config.bigram_channel,
                u64::from(self.config.bigram_channel),
                &[self.config.start_marker, self.config.start_marker],
            ));
            self.simhash_padded
                .push(0x11_0000 + self.config.simhash_domain as u32);
        }
        self.real_units += 1;
        self.append_typed_unit(unit);
        self.append_simhash_unit(unit);
        Ok(())
    }

    fn terminal_evidence(&mut self) -> AtomLaneTerminalV1 {
        if self.real_units == 0 {
            return AtomLaneTerminalV1 {
                observed_weight: self.observed_weight,
                ..AtomLaneTerminalV1::default()
            };
        }
        let checkpoint = self.checkpoint();
        self.append_typed_unit(self.config.end_marker);
        self.append_typed_unit(self.config.end_marker);
        self.append_simhash_unit(0x12_0000 + self.config.simhash_domain as u32);
        let simhash = self
            .simhash_support
            .iter()
            .enumerate()
            .fold(0_u64, |bits, (bit, support)| {
                bits | (u64::from(*support >= 0) << bit)
            });
        let evidence = AtomLaneTerminalV1 {
            observed_weight: self.observed_weight,
            generated_weight: self.generated_weight,
            shared_weight: self.shared_weight,
            simhash,
        };
        self.restore(checkpoint);
        evidence
    }

    fn append_typed_unit(&mut self, unit: u32) {
        self.padded.push(unit);
        let length = self.padded.len();
        if length >= 2 {
            self.add_typed_atom(super::super::compositional::typed_atom_key(
                self.config.bigram_channel,
                u64::from(self.config.bigram_channel),
                &self.padded[length - 2..],
            ));
        }
        if length >= 3 {
            self.add_typed_atom(super::super::compositional::typed_atom_key(
                self.config.trigram_channel,
                u64::from(self.config.trigram_channel),
                &self.padded[length - 3..],
            ));
            let bag = sort_three([
                self.padded[length - 3],
                self.padded[length - 2],
                self.padded[length - 1],
            ]);
            self.add_typed_atom(super::super::compositional::typed_atom_key(
                self.config.bag_channel,
                u64::from(self.config.bag_channel),
                &bag,
            ));
        }
        for distance in 2..=4 {
            if length > distance {
                self.add_typed_atom(super::super::compositional::typed_atom_key(
                    self.config.skip_channel,
                    u64::from(self.config.skip_channel) * 16 + distance as u64,
                    &[self.padded[length - 1 - distance], unit],
                ));
            }
        }
    }

    fn add_typed_atom(&mut self, atom: u64) {
        let count = self.generated_atom_refcounts.entry(atom).or_default();
        if *count == 0 {
            self.generated_weight += u32::from(atom_weight(atom));
            if self.observed_atoms.contains(&atom) {
                self.shared_weight += u32::from(atom_weight(atom));
            }
        }
        *count += 1;
        self.atom_undo.push(atom);
    }

    fn append_simhash_unit(&mut self, unit: u32) {
        self.simhash_padded.push(unit);
        let length = self.simhash_padded.len();
        for gram in [2_usize, 3] {
            if length >= gram {
                self.add_simhash_atom(hash_atom(
                    self.config.simhash_domain * 16 + gram as u64,
                    &self.simhash_padded[length - gram..],
                ));
            }
        }
        for distance in 2..=4 {
            if length > distance {
                self.add_simhash_atom(hash_atom(
                    self.config.simhash_domain * 16 + 8 + distance as u64,
                    &[self.simhash_padded[length - 1 - distance], unit],
                ));
            }
        }
        if length >= 3 {
            let bag = sort_three([
                self.simhash_padded[length - 3],
                self.simhash_padded[length - 2],
                self.simhash_padded[length - 1],
            ]);
            self.add_simhash_atom(hash_atom(self.config.simhash_domain * 16 + 15, &bag));
        }
    }

    fn add_simhash_atom(&mut self, atom: u64) {
        let count = self.simhash_refcounts.entry(atom).or_default();
        if *count == 0 {
            for (bit, support) in self.simhash_support.iter_mut().enumerate() {
                *support += if atom & (1_u64 << bit) == 0 { -1 } else { 1 };
            }
        }
        *count += 1;
        self.simhash_undo.push(atom);
    }
}

#[derive(Clone, Copy, Debug, Default)]
struct AtomLaneTerminalV1 {
    observed_weight: u32,
    generated_weight: u32,
    shared_weight: u32,
    simhash: u64,
}

#[derive(Clone, Copy, Debug, Default)]
struct AtomTerminalEvidenceV1 {
    similarity_milli: u16,
    character_simhash: u64,
    keyboard_simhash: u64,
}

#[derive(Clone, Debug)]
struct AtomAccumulatorV1 {
    character: AtomLaneAccumulatorV1,
    keyboard: AtomLaneAccumulatorV1,
}

impl AtomAccumulatorV1 {
    fn new(observed: &ObservedGeometryV1) -> Self {
        Self {
            character: AtomLaneAccumulatorV1::new(&observed.characters, CHARACTER_ATOM_CONFIG),
            keyboard: AtomLaneAccumulatorV1::new(&observed.keyboard, KEYBOARD_ATOM_CONFIG),
        }
    }

    fn emit_character(&mut self, unit: u32) -> Result<(), &'static str> {
        self.character.emit_real(unit)
    }

    fn emit_keyboard(&mut self, unit: u32) -> Result<(), &'static str> {
        self.keyboard.emit_real(unit)
    }

    fn reset_keyboard(&mut self) {
        self.keyboard.reset_generated();
    }

    fn reset_generated(&mut self) {
        self.character.reset_generated();
        self.keyboard.reset_generated();
    }

    fn terminal_evidence(&mut self) -> AtomTerminalEvidenceV1 {
        let character = self.character.terminal_evidence();
        let keyboard = self.keyboard.terminal_evidence();
        let observed_weight = character.observed_weight + keyboard.observed_weight;
        let generated_weight = character.generated_weight + keyboard.generated_weight;
        let denominator = observed_weight.saturating_add(generated_weight);
        let shared_weight = character.shared_weight + keyboard.shared_weight;
        let similarity_milli = if denominator == 0 {
            1_000
        } else {
            shared_weight
                .saturating_mul(2_000)
                .checked_div(denominator)
                .unwrap_or_default()
                .min(1_000) as u16
        };
        AtomTerminalEvidenceV1 {
            similarity_milli,
            character_simhash: character.simhash,
            keyboard_simhash: keyboard.simhash,
        }
    }
}

fn physical_keys(surface: &str) -> Vec<u32> {
    crate::keyboard::text_to_key_events(surface, false)
        .unwrap_or_default()
        .into_iter()
        .map(|event| u32::from(event.keycode) | (u32::from(event.shift) << 16))
        .collect()
}

fn checked_unit_len(length: usize) -> Result<u16, &'static str> {
    if length > MAX_GEOMETRY_UNITS {
        return Err("geometry sequence reaches the u16 wire ceiling");
    }
    u16::try_from(length).map_err(|_| "geometry sequence exceeds u16")
}

#[cfg(test)]
mod tests {
    use super::super::super::compositional::{
        surface_atom_profile, surface_atom_similarity_milli, surface_wave_code,
    };
    use super::*;

    #[test]
    fn incremental_osa_is_score_identical_to_v39_adapter() {
        let pairs = [
            ("кот", "кот"),
            ("кот", "кто"),
            ("прекрасный", "прекрастный"),
            ("hello", "hlelo"),
            ("собака", "cj,frf"),
            ("", ""),
            ("мало!", "мло"),
        ];
        for (observed, generated) in pairs {
            let observed_geometry = ObservedGeometryV1::new(observed).expect("observed");
            let mut state = GeometryTraversalStateV1::new(
                &observed_geometry,
                GeometryPathIdentityV1::default(),
            )
            .expect("state");
            let normalized_generated = normalize_surface(generated);
            state
                .emit_normalized_str(&normalized_generated)
                .expect("emit");
            let incremental = state.terminal_evidence().geometry_milli;
            let profile = surface_scoring_profile(observed);
            let mut workspace = SurfaceGeometryWorkspace::default();
            let exhaustive = prepared_similarity_to_normalized_surface_with_workspace_milli(
                &profile,
                &normalized_generated,
                &mut workspace,
            );
            assert_eq!(incremental, exhaustive, "{observed:?} -> {generated:?}");
        }
    }

    #[test]
    fn incremental_atom_and_simhash_match_v39_materialized_surface() {
        let pairs = [
            ("кот", "коты"),
            ("прекрасный", "прекрасно"),
            ("hello", "hlelo"),
            ("собака", "cj,frf"),
            ("", ""),
        ];
        for (observed, generated) in pairs {
            let observed_geometry = ObservedGeometryV1::new(observed).expect("observed");
            let mut state = GeometryTraversalStateV1::new(
                &observed_geometry,
                GeometryPathIdentityV1::default(),
            )
            .expect("state");
            let normalized_generated = normalize_surface(generated);
            state
                .emit_normalized_str(&normalized_generated)
                .expect("emit");
            let evidence = state.terminal_evidence();
            let expected_atom = surface_atom_similarity_milli(
                &surface_atom_profile(observed),
                &normalized_generated,
            );
            let expected_wave = surface_wave_code(&normalized_generated);
            assert_eq!(
                evidence.atom_similarity_milli, expected_atom,
                "atom parity {observed:?} -> {generated:?}"
            );
            assert_eq!(
                evidence.character_simhash, expected_wave.character,
                "character wave parity {observed:?} -> {generated:?}"
            );
            assert_eq!(
                evidence.keyboard_simhash, expected_wave.keyboard,
                "keyboard wave parity {observed:?} -> {generated:?}"
            );
        }
    }

    #[test]
    fn cloned_branch_state_preserves_complete_parent_path_identity() {
        let observed = ObservedGeometryV1::new("коты").expect("observed");
        let mut prefix = GeometryTraversalStateV1::new(
            &observed,
            GeometryPathIdentityV1 {
                lemma_id: 7,
                paradigm_id: 11,
                ..GeometryPathIdentityV1::default()
            },
        )
        .expect("state");
        prefix.emit_normalized_str("кот").expect("prefix");
        let mut plural = prefix.clone();
        plural.emit_normalized_str("ы").expect("plural");
        let mut genitive = prefix;
        genitive.emit_normalized_str("а").expect("genitive");
        assert_eq!(plural.terminal_evidence().geometry_milli, 1_000);
        assert!(
            plural.terminal_evidence().geometry_milli > genitive.terminal_evidence().geometry_milli
        );
        assert_eq!(plural.identity.lemma_id, 7);
        assert_eq!(plural.identity.paradigm_id, 11);
    }

    #[test]
    fn batched_keyboard_emission_matches_scalar_emission() {
        for generated in ["мало!", "hello!", "-кот", "кот-", "123", "ёж"] {
            let observed = ObservedGeometryV1::new(generated).expect("observed");
            let mut batched =
                GeometryTraversalStateV1::new(&observed, GeometryPathIdentityV1::default())
                    .expect("batched state");
            batched
                .emit_normalized_str(generated)
                .expect("batched emit");

            let mut scalar =
                GeometryTraversalStateV1::new(&observed, GeometryPathIdentityV1::default())
                    .expect("scalar state");
            for unit in generated.chars() {
                scalar.emit_normalized_scalar(unit).expect("scalar emit");
            }

            assert_eq!(
                batched.terminal_evidence(),
                scalar.terminal_evidence(),
                "keyboard batch parity for {generated:?}"
            );
        }
    }

    #[test]
    fn reset_reuses_atom_tables_without_changing_terminal_evidence() {
        let observed = ObservedGeometryV1::new("коты").expect("observed");
        let mut state = GeometryTraversalStateV1::new(&observed, GeometryPathIdentityV1::default())
            .expect("state");
        state.emit_normalized_str("коты").expect("first emit");
        let first = state.terminal_evidence();

        state.reset_generated();
        state.emit_normalized_str("коты").expect("second emit");

        assert_eq!(state.terminal_evidence(), first);
    }

    #[test]
    fn batch_geometry_is_exactly_equal_to_incremental_geometry() {
        let observed_surfaces = ["кот", "прекрасный", "hello", "собака", "мало!", ""];
        let generated_surfaces = [
            "кот",
            "кто",
            "прекрастный",
            "hlelo",
            "cj,frf",
            "мло",
            "мало!",
            "",
        ];
        for observed_surface in observed_surfaces {
            let observed = ObservedGeometryV1::new(observed_surface).expect("observed");
            let mut batch = BatchGeometryEvaluatorV1::new(&observed).expect("batch");
            for generated_surface in generated_surfaces {
                let normalized = normalize_surface(generated_surface);
                let mut incremental =
                    GeometryTraversalStateV1::new(&observed, GeometryPathIdentityV1::default())
                        .expect("incremental");
                incremental
                    .emit_normalized_str(&normalized)
                    .expect("incremental emit");
                let expected = incremental.terminal_evidence();
                let mut expected_ranking = expected;
                expected_ranking.character_simhash = 0;
                expected_ranking.keyboard_simhash = 0;
                assert_eq!(
                    batch
                        .evaluate_for_ranking(&normalized)
                        .expect("ranking batch evaluate"),
                    expected_ranking,
                    "ranking parity for {observed_surface:?} -> {generated_surface:?}"
                );
                assert_eq!(
                    batch.evaluate(&normalized).expect("batch evaluate"),
                    expected,
                    "batch parity for {observed_surface:?} -> {generated_surface:?}"
                );
                let cached_entries = batch.evidence_by_surface.len();
                assert_eq!(
                    batch.evaluate(&normalized).expect("cached batch evaluate"),
                    expected,
                    "cache-hit parity for {observed_surface:?} -> {generated_surface:?}"
                );
                assert_eq!(batch.evidence_by_surface.len(), cached_entries);
            }
        }
    }

    #[test]
    fn batch_geometry_interns_duplicate_terminal_surfaces_per_request() {
        let observed = ObservedGeometryV1::new("коты").expect("observed");
        let mut batch = BatchGeometryEvaluatorV1::new(&observed).expect("batch");

        let (_, first) = batch
            .evaluate_for_ranking_interned("коты")
            .expect("first terminal");
        let (_, duplicate) = batch
            .evaluate_for_ranking_interned("коты")
            .expect("duplicate terminal");

        assert!(Arc::ptr_eq(&first, &duplicate));
        assert_eq!(batch.evidence_by_surface.len(), 1);
    }
}
