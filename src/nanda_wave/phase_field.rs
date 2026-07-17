//! Shared phase-center arithmetic for learned relation memories.
//!
//! A phase vector is an observation. A center is the circular mean of several
//! observations. Positive and negative banks provide constructive and
//! destructive interference; neither bank owns runtime edit authority.

use std::f32::consts::TAU;

use crate::stable_hash::mix64_golden;

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(crate) struct PhaseCell {
    pub(crate) re: f32,
    pub(crate) im: f32,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct PhaseCenter {
    pub(crate) sum: Vec<PhaseCell>,
    pub(crate) center: Vec<PhaseCell>,
    pub(crate) support: u32,
}

impl PhaseCenter {
    pub(crate) fn from_center(center: Vec<PhaseCell>, support: u32) -> Self {
        Self {
            sum: center.clone(),
            center,
            support,
        }
    }
}

pub(crate) fn empty_vector(cells: usize) -> Vec<PhaseCell> {
    vec![PhaseCell::default(); cells]
}

pub(crate) fn stable_hash64(bytes: &[u8], lane: u64) -> u64 {
    let hash = bytes.iter().fold(
        0xcbf2_9ce4_8422_2325_u64 ^ lane.wrapping_mul(0x1000_0000_01b3),
        |hash, byte| (hash ^ u64::from(*byte)).wrapping_mul(0x0000_0100_0000_01b3),
    );
    mix64_golden(hash)
}

pub(crate) fn hash_text(text: &str) -> u64 {
    stable_hash64(text.as_bytes(), 0x0054_4f4b_454e)
}

pub(crate) fn add_hashed_atom(
    vector: &mut [PhaseCell],
    identity: u64,
    phase_key: u64,
    weight: f32,
) {
    if vector.is_empty() || weight == 0.0 {
        return;
    }
    for lane in 0..3_u64 {
        let cell_hash = mix64_golden(identity ^ lane.wrapping_mul(0x9e37_79b9_7f4a_7c15));
        let phase_hash = mix64_golden(
            phase_key ^ lane.wrapping_mul(0xbf58_476d_1ce4_e5b9) ^ identity.rotate_left(17),
        );
        let cell = (cell_hash as usize) % vector.len();
        let angle = (phase_hash as f32 / u64::MAX as f32) * TAU;
        vector[cell].re += angle.cos() * weight;
        vector[cell].im += angle.sin() * weight;
    }
}

pub(crate) fn add_rotated_vector(
    target: &mut [PhaseCell],
    source: &[PhaseCell],
    phase_key: u64,
    weight: f32,
) {
    let angle = (mix64_golden(phase_key) as f32 / u64::MAX as f32) * TAU;
    let cos = angle.cos() * weight;
    let sin = angle.sin() * weight;
    for (target, source) in target.iter_mut().zip(source) {
        target.re += source.re * cos - source.im * sin;
        target.im += source.re * sin + source.im * cos;
    }
}

pub(crate) fn add_phase_vector(target: &mut [PhaseCell], source: &[PhaseCell]) {
    for (target, source) in target.iter_mut().zip(source) {
        target.re += source.re;
        target.im += source.im;
    }
}

pub(crate) fn phase_center_from_sum(values: &[PhaseCell]) -> Vec<PhaseCell> {
    values.iter().copied().map(phase_unit).collect()
}

fn phase_unit(value: PhaseCell) -> PhaseCell {
    let norm = value.re.hypot(value.im);
    if norm == 0.0 {
        PhaseCell::default()
    } else {
        PhaseCell {
            re: value.re / norm,
            im: value.im / norm,
        }
    }
}

pub(crate) fn vector_phase_coherence(vector: &[PhaseCell], center: &[PhaseCell]) -> f32 {
    let mut score = 0.0;
    let mut active = 0usize;
    for (left, right) in vector.iter().zip(center) {
        if left.re != 0.0 || left.im != 0.0 {
            active += 1;
            score += left.re * right.re + left.im * right.im;
        }
    }
    if active == 0 {
        0.0
    } else {
        score / active as f32
    }
}

pub(crate) fn max_coherence(vector: &[PhaseCell], centers: &[PhaseCenter]) -> Option<f32> {
    centers
        .iter()
        .map(|center| vector_phase_coherence(vector, &center.center))
        .max_by(f32::total_cmp)
}

pub(crate) fn add_cluster(
    centers: &mut Vec<PhaseCenter>,
    vector: &[PhaseCell],
    max_centers: usize,
    split_coherence: f32,
) {
    let best = centers
        .iter()
        .enumerate()
        .map(|(index, center)| (index, vector_phase_coherence(vector, &center.center)))
        .max_by(|left, right| left.1.total_cmp(&right.1));
    if let Some((index, coherence)) = best {
        if coherence >= split_coherence || centers.len() >= max_centers {
            let center = &mut centers[index];
            add_phase_vector(&mut center.sum, vector);
            center.center = phase_center_from_sum(&center.sum);
            center.support = center.support.saturating_add(1);
            return;
        }
    }
    centers.push(PhaseCenter {
        sum: vector.to_vec(),
        center: phase_center_from_sum(vector),
        support: 1,
    });
}

pub(crate) fn margin(
    vector: &[PhaseCell],
    positive: &[PhaseCenter],
    negative: &[PhaseCenter],
) -> f32 {
    max_coherence(vector, positive).unwrap_or_default()
        - max_coherence(vector, negative).unwrap_or_default()
}

pub(crate) fn quantize(value: f32) -> i8 {
    (value.clamp(-1.0, 1.0) * 127.0).round() as i8
}

pub(crate) fn dequantize(value: i8) -> f32 {
    f32::from(value) / 127.0
}

pub(crate) fn phase_micro(value: f32) -> i64 {
    (value.clamp(-1.0, 1.0) * 1_000_000.0).round() as i64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn positive_and_negative_centers_create_signed_margin() {
        let mut positive_vector = empty_vector(32);
        add_hashed_atom(&mut positive_vector, 11, 17, 1.0);
        let positive_vector = phase_center_from_sum(&positive_vector);
        let mut negative_vector = empty_vector(32);
        add_hashed_atom(&mut negative_vector, 29, 31, 1.0);
        let negative_vector = phase_center_from_sum(&negative_vector);

        let positive = vec![PhaseCenter::from_center(positive_vector.clone(), 3)];
        let negative = vec![PhaseCenter::from_center(negative_vector, 3)];

        assert!(margin(&positive_vector, &positive, &negative) > 0.5);
    }
}
