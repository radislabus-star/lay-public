use std::cmp::Ordering;
use std::collections::{BTreeMap, BTreeSet};

use crate::stable_hash::mix64_golden;

use super::corpus::{MorphCorpus, MorphExample};
use super::{MorphBinding16, MorphPhaseCenter64, MAX_SUBCENTERS, PHASE_CELLS, POS_NOUN};

const SUBCENTER_MERGE_COHERENCE: i32 = 720;

#[derive(Clone, Debug, Default)]
struct MorphSlotProfile {
    positive: Vec<MorphPhaseCenter64>,
    anti: Vec<MorphPhaseCenter64>,
}

#[derive(Clone, Copy, Debug, Default)]
pub(super) struct MorphCalibration {
    pub(super) minimum_positive: i32,
    pub(super) minimum_margin: i32,
}

#[derive(Clone, Debug)]
pub(super) struct MorphologyField {
    bindings: Vec<MorphBinding16>,
    surfaces: Vec<String>,
    bindings_by_form: Vec<Vec<MorphBinding16>>,
    forms_by_lemma: Vec<Vec<u32>>,
    form_ids_by_surface: BTreeMap<String, Vec<u32>>,
    profiles: BTreeMap<u32, MorphSlotProfile>,
    calibration: MorphCalibration,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct ScoredSurface {
    pub(super) form_center_id: u32,
    pub(super) surface: String,
    pub(super) features: u32,
    pub(super) positive: i32,
    pub(super) anti: i32,
    pub(super) score: i32,
}

#[derive(Clone, Copy, Debug, Default)]
pub(super) struct MorphSlotScore {
    positive: i32,
    anti: i32,
    score: i32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum MorphReadout {
    Winner(ScoredSurface),
    Tied(Vec<ScoredSurface>),
    Abstain(Vec<ScoredSurface>),
}

impl MorphologyField {
    pub(super) fn train(corpus: &MorphCorpus) -> Result<Self, String> {
        let mut bindings = corpus.bindings.clone();
        let form_ids_by_surface = corpus
            .surfaces
            .iter()
            .enumerate()
            .map(|(form_id, surface)| (surface.as_str(), form_id as u32))
            .collect::<BTreeMap<_, _>>();
        let mut profiles = BTreeMap::<u32, MorphSlotProfile>::new();
        let slots = bindings
            .iter()
            .map(|binding| binding.features)
            .collect::<BTreeSet<_>>();

        for example in &corpus.train {
            let scene = scene_wave(&example.context);
            profiles
                .entry(example.features)
                .or_default()
                .observe_positive(&scene);
            for slot in slots
                .iter()
                .copied()
                .filter(|slot| *slot != example.features && *slot & POS_NOUN != 0)
            {
                profiles.entry(slot).or_default().observe_anti(&scene);
            }
            let form_center_id = form_ids_by_surface
                .get(example.surface.as_str())
                .copied()
                .ok_or_else(|| format!("unknown train surface {:?}", example.surface))?;
            let binding_key = (example.lemma_id, form_center_id, example.features);
            let binding_index = bindings
                .binary_search_by_key(&binding_key, |binding| {
                    (
                        binding.lemma_center_id,
                        binding.form_center_id,
                        binding.features,
                    )
                })
                .map_err(|_| {
                    format!(
                        "missing train binding lemma={} surface={:?} features={:#x}",
                        example.lemma_id, example.surface, example.features
                    )
                })?;
            let binding = &mut bindings[binding_index];
            binding.support = binding.support.saturating_add(1);
            binding.phase = 1;
        }

        let mut bindings_by_form = vec![Vec::new(); corpus.surfaces.len()];
        let mut forms_by_lemma = vec![Vec::new(); corpus.lemmas.len()];
        let mut form_ids_by_surface = BTreeMap::<String, Vec<u32>>::new();
        for binding in &bindings {
            bindings_by_form[binding.form_center_id as usize].push(*binding);
            forms_by_lemma[binding.lemma_center_id as usize].push(binding.form_center_id);
            form_ids_by_surface
                .entry(corpus.surfaces[binding.form_center_id as usize].clone())
                .or_default()
                .push(binding.form_center_id);
        }
        for forms in &mut forms_by_lemma {
            forms.sort_unstable();
            forms.dedup();
        }
        for form_ids in form_ids_by_surface.values_mut() {
            form_ids.sort_unstable();
            form_ids.dedup();
        }

        let mut field = Self {
            bindings,
            surfaces: corpus.surfaces.clone(),
            bindings_by_form,
            forms_by_lemma,
            form_ids_by_surface,
            profiles,
            calibration: MorphCalibration::default(),
        };
        field.calibration = calibrate(&field, corpus)?;
        Ok(field)
    }

    pub(super) fn calibration(&self) -> MorphCalibration {
        self.calibration
    }

    pub(super) fn binding_count(&self) -> usize {
        self.bindings.len()
    }

    pub(super) fn positive_center_count(&self) -> usize {
        self.profiles
            .values()
            .map(|profile| profile.positive.len())
            .sum()
    }

    pub(super) fn anti_center_count(&self) -> usize {
        self.profiles
            .values()
            .map(|profile| profile.anti.len())
            .sum()
    }

    #[cfg(test)]
    pub(super) fn candidate_surfaces_for(&self, target_surface: &str) -> Vec<String> {
        let Some(target_form_id) = self
            .form_ids_by_surface
            .get(target_surface)
            .and_then(|form_ids| form_ids.first())
            .copied()
        else {
            return Vec::new();
        };
        let Some(target_binding) = self
            .bindings_by_form
            .get(target_form_id as usize)
            .and_then(|bindings| bindings.first())
        else {
            return Vec::new();
        };
        self.candidate_form_ids_for_lemma(target_binding.lemma_center_id)
            .iter()
            .filter_map(|form_id| self.surfaces.get(*form_id as usize))
            .cloned()
            .collect()
    }

    pub(super) fn candidate_form_ids_for_lemma(&self, lemma_id: u32) -> &[u32] {
        self.forms_by_lemma
            .get(lemma_id as usize)
            .map(Vec::as_slice)
            .unwrap_or_default()
    }

    #[cfg(test)]
    pub(super) fn readout(&self, context: &str, candidates: &[String]) -> MorphReadout {
        let form_ids = candidates
            .iter()
            .filter_map(|surface| self.form_ids_by_surface.get(surface))
            .flatten()
            .copied()
            .collect::<Vec<_>>();
        self.readout_form_ids(context, &form_ids)
    }

    pub(super) fn readout_for_lemma_with_scores(
        &self,
        lemma_id: u32,
        scores: &BTreeMap<u32, MorphSlotScore>,
    ) -> MorphReadout {
        let ranked = self.ranked_for_lemma_with_scores(lemma_id, scores);
        classify_scored(ranked, self.calibration)
    }

    pub(super) fn readout_form_ids(&self, context: &str, form_ids: &[u32]) -> MorphReadout {
        self.readout_with_calibration(context, form_ids, self.calibration, true)
    }

    pub(super) fn ranked_for_lemma(
        &self,
        context: &str,
        lemma_id: u32,
        with_anti: bool,
    ) -> Vec<ScoredSurface> {
        self.ranked_form_ids(
            context,
            self.candidate_form_ids_for_lemma(lemma_id),
            with_anti,
        )
    }

    pub(super) fn score_slots(
        &self,
        context: &str,
        with_anti: bool,
    ) -> BTreeMap<u32, MorphSlotScore> {
        let scene = scene_wave(context);
        self.profiles
            .iter()
            .map(|(features, profile)| {
                let positive = profile.positive_score(&scene);
                let anti = if with_anti {
                    profile.anti_score(&scene)
                } else {
                    0
                };
                (
                    *features,
                    MorphSlotScore {
                        positive,
                        anti,
                        score: positive.saturating_sub(anti),
                    },
                )
            })
            .collect()
    }

    pub(super) fn ranked_for_lemma_with_scores(
        &self,
        lemma_id: u32,
        scores: &BTreeMap<u32, MorphSlotScore>,
    ) -> Vec<ScoredSurface> {
        self.ranked_form_ids_with_scores(self.candidate_form_ids_for_lemma(lemma_id), scores)
    }

    pub(super) fn ranked_form_ids(
        &self,
        context: &str,
        form_ids: &[u32],
        with_anti: bool,
    ) -> Vec<ScoredSurface> {
        let scores = self.score_slots(context, with_anti);
        self.ranked_form_ids_with_scores(form_ids, &scores)
    }

    fn ranked_form_ids_with_scores(
        &self,
        form_ids: &[u32],
        scores: &BTreeMap<u32, MorphSlotScore>,
    ) -> Vec<ScoredSurface> {
        let mut best_by_form = BTreeMap::<u32, ScoredSurface>::new();
        for form_id in form_ids {
            let Some(surface) = self.surfaces.get(*form_id as usize) else {
                continue;
            };
            let Some(bindings) = self.bindings_by_form.get(*form_id as usize) else {
                continue;
            };
            for binding in bindings {
                let Some(slot) = scores.get(&binding.features) else {
                    continue;
                };
                let scored = ScoredSurface {
                    form_center_id: binding.form_center_id,
                    surface: surface.clone(),
                    features: binding.features,
                    positive: slot.positive,
                    anti: slot.anti,
                    score: slot.score,
                };
                let replace = best_by_form
                    .get(form_id)
                    .is_none_or(|current| score_order(&scored, current) == Ordering::Less);
                if replace {
                    best_by_form.insert(*form_id, scored);
                }
            }
        }
        let mut scored = best_by_form.into_values().collect::<Vec<_>>();
        scored.sort_by(score_order);
        scored
    }

    fn readout_with_calibration(
        &self,
        context: &str,
        form_ids: &[u32],
        calibration: MorphCalibration,
        with_anti: bool,
    ) -> MorphReadout {
        let scored = self.ranked_form_ids(context, form_ids, with_anti);
        classify_scored(scored, calibration)
    }
}

fn classify_scored(scored: Vec<ScoredSurface>, calibration: MorphCalibration) -> MorphReadout {
    let Some(winner) = scored.first() else {
        return MorphReadout::Abstain(Vec::new());
    };
    let margin = winner
        .score
        .saturating_sub(scored.get(1).map(|candidate| candidate.score).unwrap_or(0));
    if winner.positive < calibration.minimum_positive || winner.score <= 0 {
        return MorphReadout::Abstain(scored);
    }
    if margin < calibration.minimum_margin {
        return MorphReadout::Tied(scored);
    }
    MorphReadout::Winner(winner.clone())
}

impl MorphSlotProfile {
    fn observe_positive(&mut self, scene: &[i8; PHASE_CELLS]) {
        observe_bank(&mut self.positive, scene);
    }

    fn observe_anti(&mut self, scene: &[i8; PHASE_CELLS]) {
        observe_bank(&mut self.anti, scene);
    }

    fn positive_score(&self, scene: &[i8; PHASE_CELLS]) -> i32 {
        bank_score(&self.positive, scene)
    }

    fn anti_score(&self, scene: &[i8; PHASE_CELLS]) -> i32 {
        bank_score(&self.anti, scene)
    }
}

fn calibrate(field: &MorphologyField, corpus: &MorphCorpus) -> Result<MorphCalibration, String> {
    let observations = corpus
        .train
        .iter()
        .map(|example| calibration_observation(field, example))
        .collect::<Result<Vec<_>, _>>()?;
    let minimum_positive = observations
        .iter()
        .filter(|observation| observation.correct)
        .map(|observation| observation.positive)
        .min()
        .unwrap_or(1);
    let maximum_wrong_margin = observations
        .iter()
        .filter(|observation| !observation.correct)
        .map(|observation| observation.margin)
        .max()
        .unwrap_or(0);
    let minimum_margin = maximum_wrong_margin.saturating_add(1).max(1);
    Ok(MorphCalibration {
        minimum_positive,
        minimum_margin,
    })
}

struct CalibrationObservation {
    correct: bool,
    positive: i32,
    margin: i32,
}

fn calibration_observation(
    field: &MorphologyField,
    example: &MorphExample,
) -> Result<CalibrationObservation, String> {
    let scored = field.ranked_for_lemma(&example.context, example.lemma_id, true);
    let winner = scored
        .first()
        .ok_or_else(|| format!("no morphology candidates for {:?}", example.surface))?;
    Ok(CalibrationObservation {
        correct: winner.features == example.features,
        positive: winner.positive,
        margin: winner
            .score
            .saturating_sub(scored.get(1).map(|candidate| candidate.score).unwrap_or(0)),
    })
}

fn observe_bank(bank: &mut Vec<MorphPhaseCenter64>, scene: &[i8; PHASE_CELLS]) {
    if bank.is_empty() {
        bank.push(center_from_scene(scene));
        return;
    }
    let (best_index, best_coherence) = bank
        .iter()
        .enumerate()
        .map(|(index, center)| (index, coherence(&center.cells, scene)))
        .max_by_key(|(_, score)| *score)
        .unwrap_or((0, i32::MIN));
    if best_coherence < SUBCENTER_MERGE_COHERENCE && bank.len() < MAX_SUBCENTERS {
        bank.push(center_from_scene(scene));
    } else {
        merge_center(&mut bank[best_index], scene);
    }
}

fn center_from_scene(scene: &[i8; PHASE_CELLS]) -> MorphPhaseCenter64 {
    MorphPhaseCenter64 {
        cells: *scene,
        support: 1,
        mass: scene
            .iter()
            .map(|cell| u16::from(cell.unsigned_abs()))
            .sum(),
    }
}

fn merge_center(center: &mut MorphPhaseCenter64, scene: &[i8; PHASE_CELLS]) {
    let old_support = i32::from(center.support.max(1));
    let new_support = old_support.saturating_add(1);
    for (cell, observed) in center.cells.iter_mut().zip(scene) {
        let mixed = (i32::from(*cell) * old_support + i32::from(*observed)) / new_support;
        *cell = mixed.clamp(i32::from(i8::MIN), i32::from(i8::MAX)) as i8;
    }
    center.support = center.support.saturating_add(1);
    center.mass = center
        .cells
        .iter()
        .map(|cell| u16::from(cell.unsigned_abs()))
        .sum();
}

fn bank_score(bank: &[MorphPhaseCenter64], scene: &[i8; PHASE_CELLS]) -> i32 {
    bank.iter()
        .map(|center| coherence(&center.cells, scene))
        .max()
        .unwrap_or(0)
}

fn coherence(left: &[i8; PHASE_CELLS], right: &[i8; PHASE_CELLS]) -> i32 {
    let mut dot = 0_i64;
    let mut left_norm = 0_i64;
    let mut right_norm = 0_i64;
    for (left, right) in left.iter().zip(right) {
        let left = i64::from(*left);
        let right = i64::from(*right);
        dot = dot.saturating_add(left.saturating_mul(right));
        left_norm = left_norm.saturating_add(left.saturating_mul(left));
        right_norm = right_norm.saturating_add(right.saturating_mul(right));
    }
    if left_norm == 0 || right_norm == 0 {
        return 0;
    }
    let denominator = ((left_norm as f64).sqrt() * (right_norm as f64).sqrt()).max(1.0);
    ((dot as f64 * 1_000.0) / denominator)
        .round()
        .clamp(-1_000.0, 1_000.0) as i32
}

fn scene_wave(context: &str) -> [i8; PHASE_CELLS] {
    let tokens = context.split_whitespace().collect::<Vec<_>>();
    let placeholder = tokens.iter().position(|token| *token == "_").unwrap_or(0);
    let mut cells = [0_i16; PHASE_CELLS];
    for (index, token) in tokens.iter().enumerate() {
        if *token == "_" {
            continue;
        }
        let relative = index as isize - placeholder as isize;
        let relative = relative.clamp(-3, 3);
        add_feature(&mut cells, &format!("token:{relative}:{token}"), 9);
        add_feature(&mut cells, &format!("token:any:{token}"), 5);
        add_feature(&mut cells, &format!("occupied:{relative}"), 3);
        let characters = token.chars().collect::<Vec<_>>();
        for width in 1..=3.min(characters.len()) {
            let suffix = characters[characters.len() - width..]
                .iter()
                .collect::<String>();
            add_feature(&mut cells, &format!("suffix:{relative}:{suffix}"), 2);
        }
    }
    let max = cells
        .iter()
        .map(|cell| cell.unsigned_abs())
        .max()
        .unwrap_or(1);
    let mut normalized = [0_i8; PHASE_CELLS];
    for (target, source) in normalized.iter_mut().zip(cells) {
        *target = (i32::from(source) * 120 / i32::from(max.max(1)))
            .clamp(i32::from(i8::MIN), i32::from(i8::MAX)) as i8;
    }
    normalized
}

fn add_feature(cells: &mut [i16; PHASE_CELLS], feature: &str, weight: i16) {
    let mut state = hash_text(feature);
    for lane in 0..4_u64 {
        state = mix64_golden(state ^ lane.wrapping_mul(0x9e37_79b9));
        let index = state as usize % PHASE_CELLS;
        let signed = if state & (1 << 63) == 0 {
            weight
        } else {
            -weight
        };
        cells[index] = cells[index].saturating_add(signed);
    }
}

fn hash_text(text: &str) -> u64 {
    text.as_bytes().iter().fold(
        mix64_golden(0x4d4f_5250_485f_4c32 ^ text.len() as u64),
        |state, byte| mix64_golden(state ^ u64::from(*byte)),
    )
}

fn score_order(left: &ScoredSurface, right: &ScoredSurface) -> Ordering {
    right
        .score
        .cmp(&left.score)
        .then_with(|| right.positive.cmp(&left.positive))
        .then_with(|| left.form_center_id.cmp(&right.form_center_id))
        .then_with(|| left.features.cmp(&right.features))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::nanda_wave::morphology_phase::corpus::parse_corpus;

    const MICRO: &str = "F\tдом\tдом\tnoun:nom:sg\n\
                         F\tдом\tдома\tnoun:gen:sg\n\
                         T\tдом\tnoun:nom:sg\t_ стоит\n\
                         T\tдома\tnoun:gen:sg\tнет _\n\
                         H\tдом\tnoun:nom:sg\t_ открыт\n";

    #[test]
    fn candidate_order_does_not_change_morphology_readout() {
        let corpus = parse_corpus(MICRO).expect("valid corpus");
        let field = MorphologyField::train(&corpus).expect("trained field");
        let mut candidates = field.candidate_surfaces_for("дом");
        let forward = field.readout("_ открыт", &candidates);
        candidates.reverse();
        assert_eq!(forward, field.readout("_ открыт", &candidates));
    }

    #[test]
    fn same_slot_surface_variants_remain_tied() {
        let corpus = parse_corpus(
            "F\tучитель\tучители\tnoun:nom:pl\n\
             F\tучитель\tучителя\tnoun:nom:pl\n\
             T\tучитель\tучители\tnoun:nom:pl\t_ появились\n\
             H\tучитель\tучителя\tnoun:nom:pl\t_ находятся здесь\n",
        )
        .expect("valid variant corpus");
        let field = MorphologyField::train(&corpus).expect("trained field");
        let candidates = field.candidate_surfaces_for("учители");
        assert!(
            matches!(
                field.readout("_ появились", &candidates),
                MorphReadout::Tied(_)
            ),
            "same-slot variants require more context, not arbitrary authority"
        );
    }
}
