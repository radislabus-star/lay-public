use sha2::{Digest, Sha256};

use super::types::{MorphologySlotKeyV1, AXIS_INAPPLICABLE};
use super::L2_SCENE_PHASE_CELLS;

const L2_SCENE_V1_SEED: u64 = 0x4c32_5343_454e_4531;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
pub(super) enum BoundaryKindV1 {
    #[default]
    None = 0,
    Token = 1,
    Phrase = 2,
    Sentence = 3,
    Line = 4,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
pub(super) enum PunctuationKindV1 {
    #[default]
    None = 0,
    Separator = 1,
    Terminal = 2,
    Opening = 3,
    Closing = 4,
    Connector = 5,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(super) struct LocalTokenObservationV1 {
    pub(super) normalized_surface: String,
    pub(super) lemma_id: Option<u32>,
    pub(super) morphology_slot: Option<MorphologySlotKeyV1>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub(super) struct TypedLocalMorphologyObservationV1 {
    pub(super) position: i8,
    pub(super) lemma_id: Option<u32>,
    pub(super) slot: MorphologySlotKeyV1,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) struct ScriptLayoutObservationV1 {
    pub(super) script_id: u16,
    pub(super) layout_id: u16,
    pub(super) flags: u32,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) struct PreeditContinuationStateV1 {
    pub(super) committed_scalars: u16,
    pub(super) preedit_scalars: u16,
    pub(super) flags: u32,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(super) struct L2LocalSceneV1 {
    pub(super) current_token: String,
    pub(super) current_normalized_scalars: Vec<u32>,
    pub(super) left_tokens: [Option<LocalTokenObservationV1>; 2],
    pub(super) right_tokens: [Option<LocalTokenObservationV1>; 2],
    pub(super) boundary_before: BoundaryKindV1,
    pub(super) boundary_after: BoundaryKindV1,
    pub(super) punctuation_before: PunctuationKindV1,
    pub(super) punctuation_after: PunctuationKindV1,
    pub(super) punctuation_adjacency_flags: u8,
    pub(super) adjacency_shape: u64,
    pub(super) script_layout: ScriptLayoutObservationV1,
    pub(super) morphology: Vec<TypedLocalMorphologyObservationV1>,
    pub(super) continuation: PreeditContinuationStateV1,
}

impl L2LocalSceneV1 {
    pub(super) fn validate(&self) -> Result<(), &'static str> {
        if self.current_token.len() > u32::MAX as usize
            || self.current_normalized_scalars.len() > u32::MAX as usize
            || self.morphology.len() > u32::MAX as usize
            || self
                .left_tokens
                .iter()
                .chain(self.right_tokens.iter())
                .flatten()
                .any(|token| token.normalized_surface.len() > u32::MAX as usize)
        {
            return Err("local scene exceeds canonical u32 sequence width");
        }
        if self.current_normalized_scalars
            != self
                .current_token
                .chars()
                .map(u32::from)
                .collect::<Vec<_>>()
        {
            return Err("current token bytes and normalized scalar sequence disagree");
        }
        if self
            .morphology
            .iter()
            .any(|observation| !(-2..=2).contains(&observation.position))
        {
            return Err("local morphology observation lies outside the two-token window");
        }
        Ok(())
    }

    pub(super) fn canonical_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        push_string(&mut bytes, &self.current_token);
        push_u32(&mut bytes, self.current_normalized_scalars.len() as u32);
        for scalar in &self.current_normalized_scalars {
            push_u32(&mut bytes, *scalar);
        }
        for token in self.left_tokens.iter().chain(self.right_tokens.iter()) {
            match token {
                Some(token) => {
                    bytes.push(1);
                    push_string(&mut bytes, &token.normalized_surface);
                    push_optional_u32(&mut bytes, token.lemma_id);
                    match token.morphology_slot {
                        Some(slot) => {
                            bytes.push(1);
                            bytes.extend_from_slice(&slot.to_bytes());
                        }
                        None => bytes.push(0),
                    }
                }
                None => bytes.push(0),
            }
        }
        bytes.extend_from_slice(&[
            self.boundary_before as u8,
            self.boundary_after as u8,
            self.punctuation_before as u8,
            self.punctuation_after as u8,
            self.punctuation_adjacency_flags,
        ]);
        bytes.extend_from_slice(&self.adjacency_shape.to_le_bytes());
        bytes.extend_from_slice(&self.script_layout.script_id.to_le_bytes());
        bytes.extend_from_slice(&self.script_layout.layout_id.to_le_bytes());
        bytes.extend_from_slice(&self.script_layout.flags.to_le_bytes());
        let mut morphology = self.morphology.clone();
        morphology.sort_unstable();
        morphology.dedup();
        push_u32(&mut bytes, morphology.len() as u32);
        for observation in morphology {
            bytes.push(observation.position as u8);
            push_optional_u32(&mut bytes, observation.lemma_id);
            bytes.extend_from_slice(&observation.slot.to_bytes());
        }
        bytes.extend_from_slice(&self.continuation.committed_scalars.to_le_bytes());
        bytes.extend_from_slice(&self.continuation.preedit_scalars.to_le_bytes());
        bytes.extend_from_slice(&self.continuation.flags.to_le_bytes());
        bytes
    }

    pub(super) fn decode_canonical_bytes(bytes: &[u8]) -> Result<Self, &'static str> {
        let mut input = SceneInputV1::new(bytes);
        let current_token = input.string()?;
        let scalar_count = input.u32()? as usize;
        let mut current_normalized_scalars = Vec::with_capacity(scalar_count);
        for _ in 0..scalar_count {
            current_normalized_scalars.push(input.u32()?);
        }
        let mut tokens = [None, None, None, None];
        for token in &mut tokens {
            *token = match input.u8()? {
                0 => None,
                1 => Some(LocalTokenObservationV1 {
                    normalized_surface: input.string()?,
                    lemma_id: input.optional_u32()?,
                    morphology_slot: match input.u8()? {
                        0 => None,
                        1 => Some(input.slot()?),
                        _ => return Err("local scene token slot presence flag is invalid"),
                    },
                }),
                _ => return Err("local scene token presence flag is invalid"),
            };
        }
        let boundary_before = decode_boundary(input.u8()?)?;
        let boundary_after = decode_boundary(input.u8()?)?;
        let punctuation_before = decode_punctuation(input.u8()?)?;
        let punctuation_after = decode_punctuation(input.u8()?)?;
        let punctuation_adjacency_flags = input.u8()?;
        let adjacency_shape = input.u64()?;
        let script_layout = ScriptLayoutObservationV1 {
            script_id: input.u16()?,
            layout_id: input.u16()?,
            flags: input.u32()?,
        };
        let morphology_count = input.u32()? as usize;
        let mut morphology = Vec::with_capacity(morphology_count);
        for _ in 0..morphology_count {
            morphology.push(TypedLocalMorphologyObservationV1 {
                position: input.u8()? as i8,
                lemma_id: input.optional_u32()?,
                slot: input.slot()?,
            });
        }
        let continuation = PreeditContinuationStateV1 {
            committed_scalars: input.u16()?,
            preedit_scalars: input.u16()?,
            flags: input.u32()?,
        };
        if !input.is_empty() {
            return Err("local scene canonical bytes have an unowned suffix");
        }
        let [left_far, left_near, right_near, right_far] = tokens;
        let scene = Self {
            current_token,
            current_normalized_scalars,
            left_tokens: [left_far, left_near],
            right_tokens: [right_near, right_far],
            boundary_before,
            boundary_after,
            punctuation_before,
            punctuation_after,
            punctuation_adjacency_flags,
            adjacency_shape,
            script_layout,
            morphology,
            continuation,
        };
        scene.validate()?;
        if scene.canonical_bytes() != bytes {
            return Err("local scene bytes are valid but not canonically ordered");
        }
        Ok(scene)
    }
}

pub(super) fn directional_scene_key(scene: &L2LocalSceneV1) -> Result<u32, &'static str> {
    scene.validate()?;
    let mut hasher = Sha256::new();
    hasher.update(b"lay-productive-directional-scene-v1\0");
    hasher.update(scene.canonical_bytes());
    let digest = hasher.finalize();
    let key = u32::from_le_bytes(digest[0..4].try_into().expect("SHA-256 prefix"));
    if key == 0 {
        return Err("productive directional scene key is zero");
    }
    Ok(key)
}

fn decode_boundary(value: u8) -> Result<BoundaryKindV1, &'static str> {
    match value {
        0 => Ok(BoundaryKindV1::None),
        1 => Ok(BoundaryKindV1::Token),
        2 => Ok(BoundaryKindV1::Phrase),
        3 => Ok(BoundaryKindV1::Sentence),
        4 => Ok(BoundaryKindV1::Line),
        _ => Err("local scene boundary kind is invalid"),
    }
}

fn decode_punctuation(value: u8) -> Result<PunctuationKindV1, &'static str> {
    match value {
        0 => Ok(PunctuationKindV1::None),
        1 => Ok(PunctuationKindV1::Separator),
        2 => Ok(PunctuationKindV1::Terminal),
        3 => Ok(PunctuationKindV1::Opening),
        4 => Ok(PunctuationKindV1::Closing),
        5 => Ok(PunctuationKindV1::Connector),
        _ => Err("local scene punctuation kind is invalid"),
    }
}

struct SceneInputV1<'a> {
    bytes: &'a [u8],
    offset: usize,
}

impl<'a> SceneInputV1<'a> {
    const fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, offset: 0 }
    }

    fn take<const N: usize>(&mut self) -> Result<[u8; N], &'static str> {
        let end = self
            .offset
            .checked_add(N)
            .ok_or("local scene canonical read overflow")?;
        let value = self
            .bytes
            .get(self.offset..end)
            .ok_or("local scene canonical bytes are truncated")?
            .try_into()
            .expect("fixed scene field");
        self.offset = end;
        Ok(value)
    }

    fn bytes(&mut self, count: usize) -> Result<&'a [u8], &'static str> {
        let end = self
            .offset
            .checked_add(count)
            .ok_or("local scene variable read overflow")?;
        let value = self
            .bytes
            .get(self.offset..end)
            .ok_or("local scene variable bytes are truncated")?;
        self.offset = end;
        Ok(value)
    }

    fn u8(&mut self) -> Result<u8, &'static str> {
        Ok(self.take::<1>()?[0])
    }

    fn u16(&mut self) -> Result<u16, &'static str> {
        Ok(u16::from_le_bytes(self.take()?))
    }

    fn u32(&mut self) -> Result<u32, &'static str> {
        Ok(u32::from_le_bytes(self.take()?))
    }

    fn u64(&mut self) -> Result<u64, &'static str> {
        Ok(u64::from_le_bytes(self.take()?))
    }

    fn string(&mut self) -> Result<String, &'static str> {
        let count = self.u32()? as usize;
        std::str::from_utf8(self.bytes(count)?)
            .map(str::to_owned)
            .map_err(|_| "local scene canonical string is not UTF-8")
    }

    fn optional_u32(&mut self) -> Result<Option<u32>, &'static str> {
        match self.u8()? {
            0 => Ok(None),
            1 => Ok(Some(self.u32()?)),
            _ => Err("local scene optional u32 presence flag is invalid"),
        }
    }

    fn slot(&mut self) -> Result<MorphologySlotKeyV1, &'static str> {
        MorphologySlotKeyV1::from_bytes(self.take()?)
    }

    fn is_empty(&self) -> bool {
        self.offset == self.bytes.len()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(super) struct SceneFeatureRecordV1 {
    pub(super) kind: u16,
    pub(super) position: i8,
    pub(super) flags: u8,
    pub(super) value: u64,
}

impl SceneFeatureRecordV1 {
    fn bytes(self) -> [u8; 12] {
        let mut bytes = [0_u8; 12];
        bytes[0..2].copy_from_slice(&self.kind.to_le_bytes());
        bytes[2] = self.position as u8;
        bytes[3] = self.flags;
        bytes[4..12].copy_from_slice(&self.value.to_le_bytes());
        bytes
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct SceneWaveV1(pub(super) [i8; L2_SCENE_PHASE_CELLS]);

impl Default for SceneWaveV1 {
    fn default() -> Self {
        Self([0; L2_SCENE_PHASE_CELLS])
    }
}

pub(super) fn scene_feature_records(scene: &L2LocalSceneV1) -> Vec<SceneFeatureRecordV1> {
    let mut records = Vec::new();
    for (token, position) in scene
        .left_tokens
        .iter()
        .zip([-2_i8, -1_i8])
        .chain(scene.right_tokens.iter().zip([1_i8, 2_i8]))
    {
        let Some(token) = token else {
            continue;
        };
        if let Some(lemma_id) = token.lemma_id {
            records.push(SceneFeatureRecordV1 {
                kind: 1,
                position,
                flags: 0,
                value: u64::from(lemma_id),
            });
        }
        if let Some(slot) = token.morphology_slot {
            push_slot_features(&mut records, position, slot);
        }
    }
    for observation in &scene.morphology {
        if let Some(lemma_id) = observation.lemma_id {
            records.push(SceneFeatureRecordV1 {
                kind: 1,
                position: observation.position,
                flags: 0,
                value: u64::from(lemma_id),
            });
        }
        push_slot_features(&mut records, observation.position, observation.slot);
    }
    if scene.boundary_before != BoundaryKindV1::None {
        records.push(SceneFeatureRecordV1 {
            kind: 14,
            position: -1,
            flags: 0,
            value: scene.boundary_before as u64,
        });
    }
    if scene.boundary_after != BoundaryKindV1::None {
        records.push(SceneFeatureRecordV1 {
            kind: 14,
            position: 1,
            flags: 0,
            value: scene.boundary_after as u64,
        });
    }
    if scene.punctuation_before != PunctuationKindV1::None {
        records.push(SceneFeatureRecordV1 {
            kind: 15,
            position: -1,
            flags: scene.punctuation_adjacency_flags,
            value: scene.punctuation_before as u64,
        });
    }
    if scene.punctuation_after != PunctuationKindV1::None {
        records.push(SceneFeatureRecordV1 {
            kind: 15,
            position: 1,
            flags: scene.punctuation_adjacency_flags,
            value: scene.punctuation_after as u64,
        });
    }
    if scene.adjacency_shape != 0 {
        records.push(SceneFeatureRecordV1 {
            kind: 16,
            position: 0,
            flags: 0,
            value: scene.adjacency_shape,
        });
    }
    if scene.script_layout != ScriptLayoutObservationV1::default() {
        records.push(SceneFeatureRecordV1 {
            kind: 17,
            position: 0,
            flags: 0,
            value: u64::from(scene.script_layout.script_id)
                | (u64::from(scene.script_layout.layout_id) << 16)
                | (u64::from(scene.script_layout.flags) << 32),
        });
    }
    if scene.continuation != PreeditContinuationStateV1::default() {
        records.push(SceneFeatureRecordV1 {
            kind: 18,
            position: 0,
            flags: 0,
            value: u64::from(scene.continuation.committed_scalars)
                | (u64::from(scene.continuation.preedit_scalars) << 16)
                | (u64::from(scene.continuation.flags) << 32),
        });
    }
    records.sort_unstable();
    records.dedup();
    records
}

pub(super) fn encode_scene_wave(scene: &L2LocalSceneV1) -> SceneWaveV1 {
    let mut accumulator = [0_i32; L2_SCENE_PHASE_CELLS];
    for feature in scene_feature_records(scene) {
        let hash =
            crate::nanda_wave::phase_field::stable_hash64(&feature.bytes(), L2_SCENE_V1_SEED);
        let cell_a = (hash as usize) % L2_SCENE_PHASE_CELLS;
        let cell_b = (cell_a + 17) % L2_SCENE_PHASE_CELLS;
        let sign = if hash & (1 << 8) == 0 { 1 } else { -1 };
        accumulator[cell_a] = accumulator[cell_a].saturating_add(9 * sign);
        accumulator[cell_b] = accumulator[cell_b].saturating_add(5 * sign);
    }
    let maximum = accumulator
        .iter()
        .map(|value| value.unsigned_abs())
        .max()
        .unwrap_or_default();
    if maximum == 0 {
        return SceneWaveV1::default();
    }
    let mut wave = [0_i8; L2_SCENE_PHASE_CELLS];
    for (output, value) in wave.iter_mut().zip(accumulator) {
        let magnitude =
            (u64::from(value.unsigned_abs()) * 120 + u64::from(maximum) / 2) / u64::from(maximum);
        *output = if value < 0 {
            -(magnitude as i8)
        } else {
            magnitude as i8
        };
    }
    SceneWaveV1(wave)
}

fn push_slot_features(
    records: &mut Vec<SceneFeatureRecordV1>,
    position: i8,
    slot: MorphologySlotKeyV1,
) {
    for (axis, value) in slot.axes().into_iter().take(12).enumerate() {
        if value == AXIS_INAPPLICABLE {
            continue;
        }
        records.push(SceneFeatureRecordV1 {
            kind: 2 + axis as u16,
            position,
            flags: 0,
            value: u64::from(value),
        });
    }
}

fn push_u32(bytes: &mut Vec<u8>, value: u32) {
    bytes.extend_from_slice(&value.to_le_bytes());
}

fn push_string(bytes: &mut Vec<u8>, value: &str) {
    push_u32(bytes, value.len() as u32);
    bytes.extend_from_slice(value.as_bytes());
}

fn push_optional_u32(bytes: &mut Vec<u8>, value: Option<u32>) {
    match value {
        Some(value) => {
            bytes.push(1);
            push_u32(bytes, value);
        }
        None => bytes.push(0),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_scene_is_sixty_zero_cells() {
        let scene = L2LocalSceneV1::default();
        assert_eq!(encode_scene_wave(&scene), SceneWaveV1([0; 60]));
    }

    #[test]
    fn scene_wave_is_order_independent_after_feature_sort() {
        let slot = MorphologySlotKeyV1::new(2, 2, 3, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0);
        let mut left = L2LocalSceneV1 {
            current_token: "test".to_string(),
            current_normalized_scalars: "test".chars().map(u32::from).collect(),
            morphology: vec![
                TypedLocalMorphologyObservationV1 {
                    position: -1,
                    lemma_id: Some(7),
                    slot,
                },
                TypedLocalMorphologyObservationV1 {
                    position: 1,
                    lemma_id: Some(9),
                    slot,
                },
            ],
            ..L2LocalSceneV1::default()
        };
        let mut right = left.clone();
        right.morphology.reverse();
        assert_eq!(encode_scene_wave(&left), encode_scene_wave(&right));
        left.morphology.push(left.morphology[0]);
        assert_eq!(encode_scene_wave(&left), encode_scene_wave(&right));
    }
}
