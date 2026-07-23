//! Shared Fourier basis used by compact atom and word wave codes.

use std::f32::consts::TAU;

use super::crystal::{
    AtomWaveCode, BasisComponent16, BasisComponent8, ComplexBasisWave, WordCenter64,
    WAVE_DIMENSION, WORD_WAVE_COMPONENTS,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(super) struct PairResidualAtom {
    pub(super) atom_id: u32,
    pub(super) position_mode: u8,
    pub(super) coefficient: i32,
}

pub(super) fn pair_residual_atoms(
    observed: impl IntoIterator<Item = (u32, u8)>,
    owner_expected: impl IntoIterator<Item = (u32, u8)>,
    competitor_expected: impl IntoIterator<Item = (u32, u8)>,
) -> Vec<PairResidualAtom> {
    let observed = observed
        .into_iter()
        .collect::<std::collections::BTreeSet<_>>();
    let owner = owner_expected
        .into_iter()
        .collect::<std::collections::BTreeSet<_>>();
    let competitor = competitor_expected
        .into_iter()
        .collect::<std::collections::BTreeSet<_>>();
    let mut all = observed.clone();
    all.extend(owner.iter().copied());
    all.extend(competitor.iter().copied());
    all.into_iter()
        .filter_map(|(atom_id, position_mode)| {
            let key = (atom_id, position_mode);
            let coefficient = 2 * i32::from(observed.contains(&key))
                - i32::from(owner.contains(&key))
                - i32::from(competitor.contains(&key));
            (coefficient != 0).then_some(PairResidualAtom {
                atom_id,
                position_mode,
                coefficient,
            })
        })
        .collect()
}

pub(super) fn positioned_atom_code(mut code: AtomWaveCode, position_mode: u8) -> AtomWaveCode {
    let shift = u16::from(position_mode) * (WAVE_DIMENSION as u16 - 1) / u16::from(u8::MAX);
    for component in &mut code.components {
        component.basis = (component.basis + shift) % WAVE_DIMENSION as u16;
    }
    code
}

pub(super) fn compile_basis() -> Vec<ComplexBasisWave> {
    (0..WAVE_DIMENSION)
        .map(|frequency| {
            let mut wave = ComplexBasisWave::default();
            for cell in 0..WAVE_DIMENSION {
                let angle = TAU * frequency as f32 * cell as f32 / WAVE_DIMENSION as f32;
                wave.re[cell] = quantize(angle.cos());
                wave.im[cell] = quantize(angle.sin());
            }
            wave
        })
        .collect()
}

pub(super) fn learn_atom_code(couplings: &[super::model::WaveCoupling]) -> AtomWaveCode {
    let mut mass = [0_i64; WAVE_DIMENSION];
    for coupling in couplings {
        for projection in 0..4_u32 {
            let mixed = u64::from(coupling.peer_id)
                .wrapping_mul(0x9e37_79b9)
                .rotate_left(projection * 11)
                ^ u64::from(projection).wrapping_mul(0x85eb_ca6b);
            let basis = (mixed as usize) % WAVE_DIMENSION;
            let sign = if mixed & (1_u64 << 63) == 0 {
                1_i64
            } else {
                -1_i64
            };
            mass[basis] =
                mass[basis].saturating_add(sign.saturating_mul(i64::from(coupling.strength)));
        }
    }
    let mut ranked = mass
        .into_iter()
        .enumerate()
        .filter(|(_, value)| *value != 0)
        .collect::<Vec<_>>();
    ranked.sort_unstable_by(|left, right| {
        right
            .1
            .unsigned_abs()
            .cmp(&left.1.unsigned_abs())
            .then_with(|| left.0.cmp(&right.0))
    });
    let max = ranked
        .first()
        .map(|(_, value)| value.unsigned_abs())
        .unwrap_or(1)
        .max(1);
    let mut code = AtomWaveCode::default();
    for (component, (basis, value)) in code.components.iter_mut().zip(ranked) {
        *component = BasisComponent16 {
            basis: basis as u16,
            coefficient: (value.saturating_mul(16_383) / max as i64).clamp(-16_383, 16_383) as i16,
        };
    }
    code
}

pub(super) fn settle_word_code(
    center: &mut WordCenter64,
    components: impl IntoIterator<Item = (u16, i32)>,
) {
    let mut mass = [0_i64; WAVE_DIMENSION];
    for (basis, coefficient) in components {
        if let Some(slot) = mass.get_mut(basis as usize) {
            *slot = slot.saturating_add(i64::from(coefficient));
        }
    }
    let mut ranked = mass
        .into_iter()
        .enumerate()
        .filter(|(_, value)| *value != 0)
        .collect::<Vec<_>>();
    ranked.sort_unstable_by(|left, right| {
        right
            .1
            .unsigned_abs()
            .cmp(&left.1.unsigned_abs())
            .then_with(|| left.0.cmp(&right.0))
    });
    let max = ranked
        .first()
        .map(|(_, value)| value.unsigned_abs())
        .unwrap_or(1)
        .max(1);
    for (slot, (basis, value)) in center
        .wave_code
        .iter_mut()
        .zip(ranked.into_iter().take(WORD_WAVE_COMPONENTS))
    {
        let scaled = (value.saturating_mul(127) / max as i64).clamp(-127, 127) as i8;
        *slot = BasisComponent8 {
            basis: basis as u8,
            coefficient: scaled,
        };
    }
}

pub(super) fn expand_atom(
    basis: &[ComplexBasisWave],
    code: AtomWaveCode,
    re: &mut [i32; WAVE_DIMENSION],
    im: &mut [i32; WAVE_DIMENSION],
    scale: i32,
) {
    for component in code.components {
        let Some(wave) = basis.get(component.basis as usize) else {
            continue;
        };
        let coefficient = i32::from(component.coefficient).saturating_mul(scale);
        for cell in 0..WAVE_DIMENSION {
            re[cell] =
                re[cell].saturating_add(i32::from(wave.re[cell]).saturating_mul(coefficient));
            im[cell] =
                im[cell].saturating_add(i32::from(wave.im[cell]).saturating_mul(coefficient));
        }
    }
}

pub(super) fn expand_word(
    basis: &[ComplexBasisWave],
    center: WordCenter64,
) -> ([i32; WAVE_DIMENSION], [i32; WAVE_DIMENSION]) {
    let mut re = [0_i32; WAVE_DIMENSION];
    let mut im = [0_i32; WAVE_DIMENSION];
    for component in center.wave_code {
        if component.coefficient == 0 {
            continue;
        }
        let Some(wave) = basis.get(component.basis as usize) else {
            continue;
        };
        let coefficient = i32::from(component.coefficient);
        for cell in 0..WAVE_DIMENSION {
            re[cell] = re[cell].saturating_add(i32::from(wave.re[cell]) * coefficient);
            im[cell] = im[cell].saturating_add(i32::from(wave.im[cell]) * coefficient);
        }
    }
    (re, im)
}

pub(super) fn complex_coherence_milli(
    left_re: &[i32; WAVE_DIMENSION],
    left_im: &[i32; WAVE_DIMENSION],
    right_re: &[i32; WAVE_DIMENSION],
    right_im: &[i32; WAVE_DIMENSION],
) -> u16 {
    let mut dot = 0_i128;
    let mut left_mass = 0_u128;
    let mut right_mass = 0_u128;
    for cell in 0..WAVE_DIMENSION {
        let lr = i128::from(left_re[cell]);
        let li = i128::from(left_im[cell]);
        let rr = i128::from(right_re[cell]);
        let ri = i128::from(right_im[cell]);
        dot = dot.saturating_add(lr.saturating_mul(rr) + li.saturating_mul(ri));
        left_mass = left_mass.saturating_add((lr * lr + li * li) as u128);
        right_mass = right_mass.saturating_add((rr * rr + ri * ri) as u128);
    }
    if left_mass == 0 || right_mass == 0 {
        return 0;
    }
    let denominator = integer_sqrt(left_mass.saturating_mul(right_mass)).max(1) as i128;
    ((dot.saturating_mul(500) / denominator) + 500).clamp(0, 1_000) as u16
}

fn integer_sqrt(value: u128) -> u128 {
    if value < 2 {
        return value;
    }
    let mut x = 1_u128 << ((128 - value.leading_zeros() as usize).div_ceil(2));
    loop {
        let next = (x + value / x) / 2;
        if next >= x {
            return x;
        }
        x = next;
    }
}

fn quantize(value: f32) -> i8 {
    (value.clamp(-1.0, 1.0) * 127.0).round() as i8
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn complex_coherence_distinguishes_aligned_and_opposed_waves() {
        let mut positive = [0_i32; WAVE_DIMENSION];
        positive[0] = 100;
        let mut negative = [0_i32; WAVE_DIMENSION];
        negative[0] = -100;
        let zero = [0_i32; WAVE_DIMENSION];
        assert_eq!(
            complex_coherence_milli(&positive, &zero, &positive, &zero),
            1_000
        );
        assert_eq!(
            complex_coherence_milli(&positive, &zero, &negative, &zero),
            0
        );
    }

    #[test]
    fn pair_residual_keeps_position_as_phase_evidence() {
        assert_eq!(
            pair_residual_atoms([(7, 10)], [(7, 10)], [(7, 90)]),
            vec![
                PairResidualAtom {
                    atom_id: 7,
                    position_mode: 10,
                    coefficient: 1,
                },
                PairResidualAtom {
                    atom_id: 7,
                    position_mode: 90,
                    coefficient: -1,
                },
            ]
        );
        let mut code = AtomWaveCode::default();
        code.components[0].basis = 9;
        assert_ne!(
            positioned_atom_code(code, 0),
            positioned_atom_code(code, 255)
        );
    }
}
