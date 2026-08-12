use std::cmp::Ordering;
use std::collections::BTreeMap;

use sha2::{Digest, Sha256};

use super::geometry::GeometryTerminalEvidenceV1;

pub(super) const PRODUCTIVE_FEATURE_COUNT: usize = 15;
const Q16_SCALE: f64 = 65_536.0;
const CONVERGENCE_DELTA: f64 = 1.0 / 16_777_216.0;
const MAX_OPTIMIZER_SWEEPS: usize = 128;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u16)]
pub(super) enum ProductiveFeatureIdV1 {
    LemmaPositiveLogEvidence = 1,
    LemmaContradictionMagnitude = 2,
    ParadigmCompatibilityLogEvidence = 3,
    SlotPositiveLogEvidence = 4,
    SlotExplicitAntiMagnitude = 5,
    PositiveCenterCoherence = 6,
    AntiCenterCoherence = 7,
    HardNegativeCenterCoherence = 8,
    CharacterGeometry = 9,
    KeyboardGeometry = 10,
    AtomPhaseCoherence = 11,
    DirectionalPositiveResidual = 12,
    DirectionalAntiResidual = 13,
    LogSupport = 14,
    Stability = 15,
}

impl ProductiveFeatureIdV1 {
    const ALL: [Self; PRODUCTIVE_FEATURE_COUNT] = [
        Self::LemmaPositiveLogEvidence,
        Self::LemmaContradictionMagnitude,
        Self::ParadigmCompatibilityLogEvidence,
        Self::SlotPositiveLogEvidence,
        Self::SlotExplicitAntiMagnitude,
        Self::PositiveCenterCoherence,
        Self::AntiCenterCoherence,
        Self::HardNegativeCenterCoherence,
        Self::CharacterGeometry,
        Self::KeyboardGeometry,
        Self::AtomPhaseCoherence,
        Self::DirectionalPositiveResidual,
        Self::DirectionalAntiResidual,
        Self::LogSupport,
        Self::Stability,
    ];

    const fn index(self) -> usize {
        self as usize - 1
    }

    const fn supportive(self) -> bool {
        matches!(
            self,
            Self::LemmaPositiveLogEvidence
                | Self::ParadigmCompatibilityLogEvidence
                | Self::SlotPositiveLogEvidence
                | Self::PositiveCenterCoherence
                | Self::CharacterGeometry
                | Self::KeyboardGeometry
                | Self::AtomPhaseCoherence
                | Self::DirectionalPositiveResidual
                | Self::LogSupport
                | Self::Stability
        )
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(super) struct CountEvidenceV1 {
    pub(super) positive: u32,
    pub(super) contradiction: u32,
    pub(super) train_positive_prior: f64,
    pub(super) train_contradiction_prior: f64,
}

impl CountEvidenceV1 {
    fn magnitudes(self) -> Result<(f64, f64), &'static str> {
        if self.positive == 0 && self.contradiction == 0 {
            return Ok((0.0, 0.0));
        }
        if !self.train_positive_prior.is_finite()
            || !self.train_contradiction_prior.is_finite()
            || self.train_positive_prior <= 0.0
            || self.train_contradiction_prior <= 0.0
        {
            return Err("productive count evidence has an invalid TRAIN prior");
        }
        let candidate_odds =
            (f64::from(self.positive) + 0.5) / (f64::from(self.contradiction) + 0.5);
        let prior_odds = self.train_positive_prior / self.train_contradiction_prior;
        let centered = libm::log(candidate_odds) - libm::log(prior_odds);
        if !centered.is_finite() {
            return Err("productive centered log evidence is not finite");
        }
        Ok((centered.max(0.0), (-centered).max(0.0)))
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub(super) struct TerminalFeatureInputV1 {
    pub(super) lemma: Option<CountEvidenceV1>,
    pub(super) paradigm: Option<CountEvidenceV1>,
    pub(super) slot: Option<CountEvidenceV1>,
    pub(super) directional: Option<CountEvidenceV1>,
    pub(super) positive_center_cosine: Option<i64>,
    pub(super) anti_center_cosine: Option<i64>,
    pub(super) hard_negative_center_cosine: Option<i64>,
    pub(super) geometry: GeometryTerminalEvidenceV1,
    pub(super) support: Option<u32>,
    pub(super) stability: Option<u16>,
}

#[derive(Clone, Debug, PartialEq)]
pub(super) struct FeatureVectorV1(pub(super) [f64; PRODUCTIVE_FEATURE_COUNT]);

impl Default for FeatureVectorV1 {
    fn default() -> Self {
        Self([0.0; PRODUCTIVE_FEATURE_COUNT])
    }
}

impl FeatureVectorV1 {
    pub(super) fn quantize(&self) -> Result<QuantizedFeatureVectorV1, &'static str> {
        let mut quantized = [0_i32; PRODUCTIVE_FEATURE_COUNT];
        for (output, value) in quantized.iter_mut().zip(self.0) {
            *output = quantize_q16(value)?;
        }
        Ok(QuantizedFeatureVectorV1(quantized))
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct QuantizedFeatureVectorV1(pub(super) [i32; PRODUCTIVE_FEATURE_COUNT]);

impl QuantizedFeatureVectorV1 {
    pub(super) fn with_geometry(
        mut self,
        geometry: GeometryTerminalEvidenceV1,
    ) -> Result<Self, &'static str> {
        self.0[ProductiveFeatureIdV1::CharacterGeometry.index()] =
            quantize_q16(f64::from(geometry.character_similarity_milli) / 1_000.0)?;
        self.0[ProductiveFeatureIdV1::KeyboardGeometry.index()] =
            quantize_q16(f64::from(geometry.keyboard_similarity_milli) / 1_000.0)?;
        self.0[ProductiveFeatureIdV1::AtomPhaseCoherence.index()] =
            quantize_q16(f64::from(geometry.atom_similarity_milli) / 1_000.0)?;
        Ok(self)
    }
}

pub(super) fn extract_feature_vector(
    input: TerminalFeatureInputV1,
) -> Result<FeatureVectorV1, &'static str> {
    let mut features = FeatureVectorV1::default();
    if let Some(evidence) = input.lemma {
        let (positive, negative) = evidence.magnitudes()?;
        features.0[ProductiveFeatureIdV1::LemmaPositiveLogEvidence.index()] = positive;
        features.0[ProductiveFeatureIdV1::LemmaContradictionMagnitude.index()] = negative;
    }
    if let Some(evidence) = input.paradigm {
        features.0[ProductiveFeatureIdV1::ParadigmCompatibilityLogEvidence.index()] =
            evidence.magnitudes()?.0;
    }
    if let Some(evidence) = input.slot {
        let (positive, negative) = evidence.magnitudes()?;
        features.0[ProductiveFeatureIdV1::SlotPositiveLogEvidence.index()] = positive;
        features.0[ProductiveFeatureIdV1::SlotExplicitAntiMagnitude.index()] = negative;
    }
    features.0[ProductiveFeatureIdV1::PositiveCenterCoherence.index()] =
        normalized_cosine(input.positive_center_cosine);
    features.0[ProductiveFeatureIdV1::AntiCenterCoherence.index()] =
        normalized_cosine(input.anti_center_cosine);
    features.0[ProductiveFeatureIdV1::HardNegativeCenterCoherence.index()] =
        normalized_cosine(input.hard_negative_center_cosine);
    features.0[ProductiveFeatureIdV1::CharacterGeometry.index()] =
        f64::from(input.geometry.character_similarity_milli) / 1_000.0;
    features.0[ProductiveFeatureIdV1::KeyboardGeometry.index()] =
        f64::from(input.geometry.keyboard_similarity_milli) / 1_000.0;
    features.0[ProductiveFeatureIdV1::AtomPhaseCoherence.index()] =
        f64::from(input.geometry.atom_similarity_milli) / 1_000.0;
    if let Some(evidence) = input.directional {
        let (positive, negative) = evidence.magnitudes()?;
        features.0[ProductiveFeatureIdV1::DirectionalPositiveResidual.index()] = positive;
        features.0[ProductiveFeatureIdV1::DirectionalAntiResidual.index()] = negative;
    }
    if let Some(support) = input.support.filter(|support| *support != 0) {
        features.0[ProductiveFeatureIdV1::LogSupport.index()] = libm::log1p(f64::from(support));
    }
    if let Some(stability) = input.stability.filter(|stability| *stability != 0) {
        features.0[ProductiveFeatureIdV1::Stability.index()] =
            f64::from(stability) / f64::from(u16::MAX);
    }
    if features
        .0
        .iter()
        .any(|value| !value.is_finite() || *value < 0.0)
    {
        return Err("productive feature vector contains a negative or non-finite magnitude");
    }
    Ok(features)
}

fn normalized_cosine(value: Option<i64>) -> f64 {
    value.unwrap_or_default().max(0) as f64 / 1_000_000.0
}

#[derive(Clone, Debug, PartialEq)]
pub(super) struct PairwiseTrainingPairV1 {
    pub(super) group_identity: [u8; 32],
    pub(super) stable_event_identity: [u8; 32],
    pub(super) inner_fold: u8,
    pub(super) valid: FeatureVectorV1,
    pub(super) contradicted: FeatureVectorV1,
}

#[derive(Clone, Debug)]
struct WeightedPairV1 {
    source: PairwiseTrainingPairV1,
    weight: f64,
}

#[derive(Clone, Debug, PartialEq)]
pub(super) struct FittedEvidenceModelV1 {
    pub(super) selected_lambda: f64,
    pub(super) coefficients: [f64; PRODUCTIVE_FEATURE_COUNT],
    pub(super) coefficients_q16: [i32; PRODUCTIVE_FEATURE_COUNT],
    pub(super) sweeps: usize,
    pub(super) training_pair_count: u32,
}

pub(super) fn productive_feature_schema_hash_low() -> Result<u32, &'static str> {
    let mut canonical = Vec::new();
    canonical.extend_from_slice(b"lay-productive-features-v1\0");
    for feature in ProductiveFeatureIdV1::ALL {
        canonical.extend_from_slice(&(feature as u16).to_le_bytes());
        canonical.push(u8::from(feature.supportive()));
    }
    let digest = Sha256::digest(canonical);
    let hash = u32::from_le_bytes(digest[0..4].try_into().expect("SHA-256 prefix"));
    if hash == 0 {
        return Err("productive feature schema hash is zero");
    }
    Ok(hash)
}

pub(super) fn fit_evidence_model(
    pairs: &[PairwiseTrainingPairV1],
) -> Result<FittedEvidenceModelV1, &'static str> {
    let weighted = validate_and_weight_pairs(pairs)?;
    let lambdas = lambda_grid();
    let mut selected: Option<(f64, f64, Vec<u8>)> = None;
    for lambda in lambdas {
        let mut total_loss = 0.0;
        let mut serialized_folds = Vec::new();
        let mut valid = true;
        for fold in 0..super::PRODUCTIVE_V1_INNER_FOLDS as u8 {
            let train = weighted
                .iter()
                .filter(|pair| pair.source.inner_fold != fold)
                .cloned()
                .collect::<Vec<_>>();
            let validation = weighted
                .iter()
                .filter(|pair| pair.source.inner_fold == fold)
                .cloned()
                .collect::<Vec<_>>();
            if train.is_empty() || validation.is_empty() {
                return Err("productive inner fold has no train or validation pairs");
            }
            let Ok((coefficients, _)) = optimize_coefficients(&train, lambda) else {
                valid = false;
                break;
            };
            total_loss += pairwise_log_loss(&validation, &coefficients)?;
            serialize_coefficients(&coefficients, &mut serialized_folds);
        }
        if !valid {
            continue;
        }
        let mean_loss = total_loss / super::PRODUCTIVE_V1_INNER_FOLDS as f64;
        let replace = match &selected {
            None => true,
            Some((best_loss, best_lambda, best_serialized)) => mean_loss
                .total_cmp(best_loss)
                .then_with(|| lambda.total_cmp(best_lambda).reverse())
                .then_with(|| serialized_folds.cmp(best_serialized))
                .is_lt(),
        };
        if replace {
            selected = Some((mean_loss, lambda, serialized_folds));
        }
    }
    let (_, selected_lambda, _) = selected.ok_or("no productive lambda converged")?;
    let (coefficients, sweeps) = optimize_coefficients(&weighted, selected_lambda)?;
    let mut coefficients_q16 = [0_i32; PRODUCTIVE_FEATURE_COUNT];
    for (output, coefficient) in coefficients_q16.iter_mut().zip(coefficients) {
        *output = quantize_q16(coefficient)?;
        if *output < 0 {
            return Err("productive coefficient quantized below zero");
        }
    }
    let model = FittedEvidenceModelV1 {
        selected_lambda,
        coefficients,
        coefficients_q16,
        sweeps,
        training_pair_count: u32::try_from(pairs.len())
            .map_err(|_| "productive training pair count exceeds u32")?,
    };
    verify_quantized_pair_order(&model, pairs)?;
    Ok(model)
}

fn validate_and_weight_pairs(
    pairs: &[PairwiseTrainingPairV1],
) -> Result<Vec<WeightedPairV1>, &'static str> {
    if pairs.is_empty() {
        return Err("productive evidence training requires explicit pairs");
    }
    let mut ordered = pairs.to_vec();
    ordered.sort_by(|left, right| {
        left.group_identity
            .cmp(&right.group_identity)
            .then_with(|| left.stable_event_identity.cmp(&right.stable_event_identity))
            .then_with(|| feature_order(&left.valid, &right.valid))
            .then_with(|| feature_order(&left.contradicted, &right.contradicted))
    });
    let mut group_counts = BTreeMap::<[u8; 32], (usize, u8)>::new();
    for pair in &ordered {
        if pair.inner_fold >= super::PRODUCTIVE_V1_INNER_FOLDS as u8 {
            return Err("productive pair has an invalid inner fold");
        }
        if pair
            .valid
            .0
            .iter()
            .chain(pair.contradicted.0.iter())
            .any(|value| !value.is_finite() || *value < 0.0)
        {
            return Err("productive pair contains an invalid feature magnitude");
        }
        let entry = group_counts
            .entry(pair.group_identity)
            .or_insert((0, pair.inner_fold));
        if entry.1 != pair.inner_fold {
            return Err("one productive scene group crosses lemma-owned folds");
        }
        entry.0 += 1;
    }
    Ok(ordered
        .into_iter()
        .map(|source| WeightedPairV1 {
            weight: 1.0 / group_counts[&source.group_identity].0 as f64,
            source,
        })
        .collect())
}

fn feature_order(left: &FeatureVectorV1, right: &FeatureVectorV1) -> Ordering {
    left.0
        .iter()
        .zip(right.0.iter())
        .find_map(|(left, right)| {
            let order = left.total_cmp(right);
            (!order.is_eq()).then_some(order)
        })
        .unwrap_or(Ordering::Equal)
}

fn lambda_grid() -> Vec<f64> {
    std::iter::once(0.0)
        .chain((-12..=8).map(|exponent| libm::pow(2.0, f64::from(exponent))))
        .collect()
}

fn optimize_coefficients(
    pairs: &[WeightedPairV1],
    lambda: f64,
) -> Result<([f64; PRODUCTIVE_FEATURE_COUNT], usize), &'static str> {
    let mut coefficients = [0.0_f64; PRODUCTIVE_FEATURE_COUNT];
    for sweep in 0..MAX_OPTIMIZER_SWEEPS {
        let mut maximum_delta = 0.0_f64;
        for feature in ProductiveFeatureIdV1::ALL {
            let feature_index = feature.index();
            let mut gradient = 2.0 * lambda * coefficients[feature_index];
            let mut hessian = 2.0 * lambda;
            for pair in pairs {
                let difference = signed_feature_difference(
                    feature,
                    pair.source.valid.0[feature_index],
                    pair.source.contradicted.0[feature_index],
                );
                if difference == 0.0 {
                    continue;
                }
                let margin = reference_margin(&coefficients, &pair.source);
                let negative_probability = logistic_negative_margin(margin);
                gradient -= pair.weight * difference * negative_probability;
                hessian += pair.weight
                    * difference
                    * difference
                    * negative_probability
                    * (1.0 - negative_probability);
            }
            if hessian == 0.0 {
                continue;
            }
            if !gradient.is_finite() || !hessian.is_finite() || hessian < 0.0 {
                return Err("productive Newton reducer produced invalid curvature");
            }
            let previous = coefficients[feature_index];
            let next = (previous - gradient / hessian).max(0.0);
            if !next.is_finite() {
                return Err("productive Newton reducer produced a non-finite coefficient");
            }
            coefficients[feature_index] = next;
            maximum_delta = maximum_delta.max((next - previous).abs());
        }
        if maximum_delta < CONVERGENCE_DELTA {
            return Ok((coefficients, sweep + 1));
        }
    }
    Err("productive Newton reducer did not converge within 128 sweeps")
}

fn reference_margin(
    coefficients: &[f64; PRODUCTIVE_FEATURE_COUNT],
    pair: &PairwiseTrainingPairV1,
) -> f64 {
    ProductiveFeatureIdV1::ALL
        .into_iter()
        .map(|feature| {
            coefficients[feature.index()]
                * signed_feature_difference(
                    feature,
                    pair.valid.0[feature.index()],
                    pair.contradicted.0[feature.index()],
                )
        })
        .sum()
}

fn signed_feature_difference(feature: ProductiveFeatureIdV1, valid: f64, other: f64) -> f64 {
    let difference = valid - other;
    if feature.supportive() {
        difference
    } else {
        -difference
    }
}

fn logistic_negative_margin(margin: f64) -> f64 {
    if margin >= 0.0 {
        let exponential = libm::exp(-margin);
        exponential / (1.0 + exponential)
    } else {
        1.0 / (1.0 + libm::exp(margin))
    }
}

fn pairwise_log_loss(
    pairs: &[WeightedPairV1],
    coefficients: &[f64; PRODUCTIVE_FEATURE_COUNT],
) -> Result<f64, &'static str> {
    let mut loss = 0.0;
    for pair in pairs {
        let margin = reference_margin(coefficients, &pair.source);
        let contribution = if margin >= 0.0 {
            libm::log1p(libm::exp(-margin))
        } else {
            -margin + libm::log1p(libm::exp(margin))
        };
        loss += pair.weight * contribution;
    }
    if loss.is_finite() {
        Ok(loss)
    } else {
        Err("productive validation loss is not finite")
    }
}

fn serialize_coefficients(coefficients: &[f64; PRODUCTIVE_FEATURE_COUNT], output: &mut Vec<u8>) {
    for coefficient in coefficients {
        output.extend_from_slice(&coefficient.to_bits().to_le_bytes());
    }
}

pub(super) fn fixed_point_score_q16(
    coefficients: &[i32; PRODUCTIVE_FEATURE_COUNT],
    features: QuantizedFeatureVectorV1,
) -> Result<i64, &'static str> {
    let mut accumulator_q32 = 0_i64;
    for feature in ProductiveFeatureIdV1::ALL {
        let product = i64::from(coefficients[feature.index()])
            .checked_mul(i64::from(features.0[feature.index()]))
            .ok_or("productive fixed-point product overflow")?;
        let signed = if feature.supportive() {
            product
        } else {
            product
                .checked_neg()
                .ok_or("productive fixed-point polarity overflow")?
        };
        accumulator_q32 = accumulator_q32
            .checked_add(signed)
            .ok_or("productive fixed-point score overflow")?;
    }
    Ok(accumulator_q32 / 65_536)
}

pub(super) fn verify_quantized_pair_order(
    model: &FittedEvidenceModelV1,
    pairs: &[PairwiseTrainingPairV1],
) -> Result<(), &'static str> {
    for pair in pairs {
        let reference = reference_margin(&model.coefficients, pair);
        let valid = fixed_point_score_q16(&model.coefficients_q16, pair.valid.quantize()?)?;
        let contradicted =
            fixed_point_score_q16(&model.coefficients_q16, pair.contradicted.quantize()?)?;
        let quantized = valid.cmp(&contradicted);
        let reference = reference.total_cmp(&0.0);
        if (reference.is_gt() && !quantized.is_gt())
            || (reference.is_lt() && !quantized.is_lt())
            || (reference.is_eq() && !quantized.is_eq())
        {
            return Err("productive Q16 score changes fitted pair ordering");
        }
    }
    Ok(())
}

fn quantize_q16(value: f64) -> Result<i32, &'static str> {
    if !value.is_finite() {
        return Err("productive Q16 input is not finite");
    }
    let scaled = value * Q16_SCALE;
    if scaled < f64::from(i32::MIN) || scaled > f64::from(i32::MAX) {
        return Err("productive Q16 input exceeds i32");
    }
    let sign = if scaled < 0.0 { -1.0 } else { 1.0 };
    let magnitude = scaled.abs();
    let floor = libm::floor(magnitude);
    let fraction = magnitude - floor;
    let mut rounded = floor;
    if fraction > 0.5 || (fraction == 0.5 && (floor as i64) & 1 == 1) {
        rounded += 1.0;
    }
    Ok((sign * rounded) as i32)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn prepared_invariant_features_reproduce_full_geometry_quantization() {
        for geometry in [
            GeometryTerminalEvidenceV1::default(),
            GeometryTerminalEvidenceV1 {
                character_similarity_milli: 1,
                keyboard_similarity_milli: 499,
                atom_similarity_milli: 999,
                ..GeometryTerminalEvidenceV1::default()
            },
            GeometryTerminalEvidenceV1 {
                character_similarity_milli: 1_000,
                keyboard_similarity_milli: 500,
                atom_similarity_milli: 333,
                ..GeometryTerminalEvidenceV1::default()
            },
        ] {
            let mut input = TerminalFeatureInputV1 {
                lemma: Some(CountEvidenceV1 {
                    positive: 31,
                    contradiction: 3,
                    train_positive_prior: 0.7,
                    train_contradiction_prior: 0.3,
                }),
                paradigm: Some(CountEvidenceV1 {
                    positive: 71,
                    contradiction: 0,
                    train_positive_prior: 0.8,
                    train_contradiction_prior: 0.2,
                }),
                slot: Some(CountEvidenceV1 {
                    positive: 23,
                    contradiction: 5,
                    train_positive_prior: 0.6,
                    train_contradiction_prior: 0.4,
                }),
                directional: Some(CountEvidenceV1 {
                    positive: 17,
                    contradiction: 2,
                    train_positive_prior: 0.55,
                    train_contradiction_prior: 0.45,
                }),
                positive_center_cosine: Some(812_345),
                anti_center_cosine: Some(123_456),
                hard_negative_center_cosine: Some(234_567),
                support: Some(19),
                stability: Some(41_000),
                ..TerminalFeatureInputV1::default()
            };
            let invariant = extract_feature_vector(input)
                .expect("invariant features")
                .quantize()
                .expect("quantized invariant features");
            input.geometry = geometry;
            let complete = extract_feature_vector(input)
                .expect("complete features")
                .quantize()
                .expect("quantized complete features");

            assert_eq!(
                invariant.with_geometry(geometry).expect("geometry overlay"),
                complete
            );
        }
    }

    fn vector(supportive: f64, anti: f64) -> FeatureVectorV1 {
        let mut vector = FeatureVectorV1::default();
        vector.0[ProductiveFeatureIdV1::LemmaPositiveLogEvidence.index()] = supportive;
        vector.0[ProductiveFeatureIdV1::LemmaContradictionMagnitude.index()] = anti;
        vector
    }

    #[test]
    fn missing_evidence_is_exact_neutral_zero() {
        let vector = extract_feature_vector(TerminalFeatureInputV1::default()).expect("features");
        assert_eq!(vector, FeatureVectorV1::default());
        let absent = CountEvidenceV1 {
            train_positive_prior: 100.0,
            train_contradiction_prior: 1.0,
            ..CountEvidenceV1::default()
        };
        assert_eq!(absent.magnitudes().expect("magnitudes"), (0.0, 0.0));
    }

    #[test]
    fn fixed_point_polarity_subtracts_only_explicit_negative_features() {
        let coefficients = [65_536_i32; PRODUCTIVE_FEATURE_COUNT];
        let positive = vector(1.0, 0.0).quantize().expect("positive");
        let anti = vector(0.0, 1.0).quantize().expect("anti");
        assert_eq!(
            fixed_point_score_q16(&coefficients, positive).expect("score"),
            65_536
        );
        assert_eq!(
            fixed_point_score_q16(&coefficients, anti).expect("score"),
            -65_536
        );
    }

    #[test]
    fn projected_training_is_deterministic_and_nonnegative() {
        let mut pairs = Vec::new();
        for fold in 0..5_u8 {
            for group in 0..3_u8 {
                let mut group_identity = [0_u8; 32];
                group_identity[0] = fold;
                group_identity[1] = group;
                let mut event_identity = group_identity;
                event_identity[2] = 1;
                pairs.push(PairwiseTrainingPairV1 {
                    group_identity,
                    stable_event_identity: event_identity,
                    inner_fold: fold,
                    valid: vector(1.0 + f64::from(group) / 10.0, 0.0),
                    contradicted: vector(0.0, 1.0 + f64::from(fold) / 10.0),
                });
            }
        }
        let forward = fit_evidence_model(&pairs).expect("forward model");
        pairs.reverse();
        let reverse = fit_evidence_model(&pairs).expect("reverse model");
        assert_eq!(forward.coefficients_q16, reverse.coefficients_q16);
        assert_eq!(
            forward.selected_lambda.to_bits(),
            reverse.selected_lambda.to_bits()
        );
        assert!(forward
            .coefficients
            .iter()
            .all(|coefficient| *coefficient >= 0.0));
        verify_quantized_pair_order(&forward, &pairs).expect("quantized parity");
    }

    #[test]
    fn q16_rounding_is_ties_to_even() {
        assert_eq!(quantize_q16(0.5 / Q16_SCALE).expect("half"), 0);
        assert_eq!(quantize_q16(1.5 / Q16_SCALE).expect("one and half"), 2);
        assert_eq!(quantize_q16(2.5 / Q16_SCALE).expect("two and half"), 2);
    }
}
