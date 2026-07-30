use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use crate::correction_core::UnifiedCorrectionCandidate;

use super::context::{context_mode, scene_wave};
use super::format::decode_package;
use super::model::{
    L2FieldPackage, MorphBinding, SlotPhaseCenter, TieCalibration,
    COMPETITION_FLAG_EXPLICIT_NEIGHBOR, L2_PHASE_CELLS, NO_L1_TERMINAL,
};

const MAX_ACTIVE_LEMMAS: usize = 4;
const INHERITED_L1_ATTENUATION_MILLI: i32 = 240;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum L2FieldBridgeKind {
    Shadow,
}

impl L2FieldBridgeKind {
    pub(crate) const fn surface_source_id(self) -> &'static str {
        "L2FieldShadowSurface"
    }

    pub(crate) const fn readout_source_id(self) -> &'static str {
        "L2FieldShadowReadout"
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub(crate) struct L2FieldShadowReadout {
    pub(crate) candidates: Vec<UnifiedCorrectionCandidate>,
    pub(crate) authority: L2FieldAuthority,
}

impl L2FieldShadowReadout {
    pub(crate) fn new(
        candidates: Vec<UnifiedCorrectionCandidate>,
        authority: L2FieldAuthority,
    ) -> Self {
        Self {
            candidates,
            authority,
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) enum L2FieldAuthority {
    #[default]
    Unavailable,
    Winner {
        surface: String,
    },
    Tied,
    Abstain,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct L2LexicalSeed {
    pub(crate) terminal_id: Option<u32>,
    pub(crate) surface: Option<String>,
    pub(crate) evidence_milli: i32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct L2LocalCandidate {
    pub(crate) form_ref: u32,
    pub(crate) l1_terminal_id: Option<u32>,
    pub(crate) surface: String,
    pub(crate) l1_evidence_milli: i32,
    pub(crate) slot_phase_milli: i32,
    pub(crate) neighbor_pressure: i32,
    pub(crate) competition_pressure: i32,
    pub(crate) explicit_competition_pressure: i32,
    pub(crate) local_score: i32,
    pub(crate) lemma_ids: Vec<u32>,
    pub(crate) feature_masks: Vec<u32>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum L2LocalVerdict {
    Winner { form_ref: u32 },
    Tied { form_refs: Vec<u32> },
    Abstain,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct StandaloneL2Readout {
    pub(crate) verdict: L2LocalVerdict,
    pub(crate) candidates: Vec<L2LocalCandidate>,
    pub(crate) context_mode_id: Option<u32>,
}

#[derive(Clone, Debug)]
pub(crate) struct StandaloneL2Field {
    package: L2FieldPackage,
    form_by_terminal: BTreeMap<u32, u32>,
    bindings_by_form: Vec<Vec<MorphBinding>>,
    forms_by_lemma: Vec<Vec<u32>>,
    context_by_key: BTreeMap<u32, u32>,
    slot_centers_by_mode_feature: BTreeMap<(u32, u32), Vec<u32>>,
    neighbor_couplings_by_mode_lemma_feature: BTreeMap<(u32, u32, u32), Vec<u32>>,
}

impl StandaloneL2Field {
    pub(crate) fn load(path: &Path) -> Result<Self, String> {
        let bytes = std::fs::read(path)
            .map_err(|error| format!("failed to read L2 package {}: {error}", path.display()))?;
        Self::from_bytes(&bytes)
    }

    pub(crate) fn from_bytes(bytes: &[u8]) -> Result<Self, String> {
        Self::from_package(decode_package(bytes)?)
    }

    pub(crate) fn from_package(package: L2FieldPackage) -> Result<Self, String> {
        let mut form_by_terminal = BTreeMap::new();
        for (form_ref, form) in package.form_refs.iter().enumerate() {
            if form.l1_terminal_id == NO_L1_TERMINAL {
                continue;
            }
            if form_by_terminal
                .insert(form.l1_terminal_id, form_ref as u32)
                .is_some()
            {
                return Err(format!(
                    "duplicate L1.1 terminal ID {} in L2 package",
                    form.l1_terminal_id
                ));
            }
        }
        let mut bindings_by_form = vec![Vec::new(); package.form_refs.len()];
        let mut forms_by_lemma = vec![Vec::new(); package.lemma_centers.len()];
        for binding in &package.morph_bindings {
            bindings_by_form[binding.form_center_ref as usize].push(*binding);
            forms_by_lemma[binding.lemma_center_id as usize].push(binding.form_center_ref);
        }
        for forms in &mut forms_by_lemma {
            forms.sort_unstable();
            forms.dedup();
        }
        let context_by_key = package
            .context_modes
            .iter()
            .enumerate()
            .map(|(index, mode)| (mode.stable_key, index as u32))
            .collect();
        let mut slot_centers_by_mode_feature = BTreeMap::<(u32, u32), Vec<u32>>::new();
        for (index, center) in package.slot_centers.iter().enumerate() {
            slot_centers_by_mode_feature
                .entry((center.context_mode_id, center.feature_mask))
                .or_default()
                .push(index as u32);
        }
        let mut neighbor_couplings_by_mode_lemma_feature =
            BTreeMap::<(u32, u32, u32), Vec<u32>>::new();
        for (index, coupling) in package.neighbor_couplings.iter().enumerate() {
            neighbor_couplings_by_mode_lemma_feature
                .entry((
                    coupling.context_mode_id,
                    coupling.target_lemma_id,
                    coupling.target_feature_mask,
                ))
                .or_default()
                .push(index as u32);
        }
        Ok(Self {
            package,
            form_by_terminal,
            bindings_by_form,
            forms_by_lemma,
            context_by_key,
            slot_centers_by_mode_feature,
            neighbor_couplings_by_mode_lemma_feature,
        })
    }

    pub(crate) fn l1_package_fingerprint(&self) -> u64 {
        self.package.l1_package_fingerprint
    }

    pub(crate) fn package_counts(&self) -> (usize, usize, usize, usize, usize, usize) {
        (
            self.package.form_refs.len(),
            self.form_by_terminal.len(),
            self.package.lemma_centers.len(),
            self.package.morph_bindings.len(),
            self.package.competition_edges.len(),
            self.package.decoder_bytes.len(),
        )
    }

    pub(crate) fn bound_form_refs(&self) -> impl Iterator<Item = (u32, u32)> + '_ {
        self.package
            .form_refs
            .iter()
            .enumerate()
            .filter_map(|(form_ref, form)| {
                (form.l1_terminal_id != NO_L1_TERMINAL)
                    .then_some((u32::try_from(form_ref).ok()?, form.l1_terminal_id))
            })
    }

    pub(crate) fn form_ref_for_surface(&self, surface: &str) -> Option<u32> {
        self.package
            .form_refs
            .binary_search_by(|form| self.decode_form(*form).unwrap_or_default().cmp(surface))
            .ok()
            .and_then(|index| u32::try_from(index).ok())
    }

    pub(crate) fn decode_form_ref(&self, form_ref: u32) -> Option<&str> {
        self.decode_form(*self.package.form_refs.get(form_ref as usize)?)
    }

    pub(crate) fn readout(
        &self,
        context: &str,
        seeds: &[L2LexicalSeed],
        candidate_limit: usize,
    ) -> StandaloneL2Readout {
        let mode = context_mode(context);
        let context_mode_id = self.context_by_key.get(&mode.stable_key).copied();
        let wave = scene_wave(context);
        let mut seed_evidence = BTreeMap::<u32, i32>::new();
        for seed in seeds {
            let form_ref = seed
                .terminal_id
                .and_then(|terminal_id| self.form_by_terminal.get(&terminal_id).copied())
                .or_else(|| {
                    seed.surface
                        .as_deref()
                        .and_then(|surface| self.form_ref_for_surface(surface))
                });
            let Some(form_ref) = form_ref else {
                continue;
            };
            seed_evidence
                .entry(form_ref)
                .and_modify(|evidence| *evidence = (*evidence).max(seed.evidence_milli))
                .or_insert(seed.evidence_milli);
        }
        let mut direct_seed_common_lemmas = None::<BTreeSet<u32>>;
        for form_ref in seed_evidence.keys() {
            let form_lemmas = self.bindings_by_form[*form_ref as usize]
                .iter()
                .map(|binding| binding.lemma_center_id)
                .collect::<BTreeSet<_>>();
            direct_seed_common_lemmas = Some(match direct_seed_common_lemmas {
                Some(common) => common.intersection(&form_lemmas).copied().collect(),
                None => form_lemmas,
            });
        }
        let direct_seed_common_lemmas = direct_seed_common_lemmas.unwrap_or_default();
        let mut active_forms = seed_evidence.keys().copied().collect::<BTreeSet<_>>();
        let mut lemma_seed_evidence = BTreeMap::<u32, (i32, u16, u16)>::new();
        for form_ref in &active_forms {
            let evidence = seed_evidence.get(form_ref).copied().unwrap_or_default();
            let mut form_lemmas = BTreeMap::<u32, u16>::new();
            for binding in &self.bindings_by_form[*form_ref as usize] {
                form_lemmas
                    .entry(binding.lemma_center_id)
                    .and_modify(|support| *support = (*support).max(binding.support))
                    .or_insert(binding.support);
            }
            for (lemma_id, binding_support) in form_lemmas {
                let hypothesis = lemma_seed_evidence
                    .entry(lemma_id)
                    .or_insert((i32::MIN, 0, 0));
                hypothesis.0 = hypothesis.0.max(evidence);
                hypothesis.1 = hypothesis.1.max(binding_support);
                hypothesis.2 = hypothesis.2.saturating_add(1);
            }
        }
        let mut lemma_hypotheses = lemma_seed_evidence
            .into_iter()
            .map(
                |(lemma_id, (seed_evidence, binding_support, seed_support))| {
                    let context_evidence =
                        self.best_lemma_context_evidence(lemma_id, context_mode_id, &wave);
                    (
                        lemma_id,
                        seed_evidence.saturating_add(context_evidence),
                        seed_evidence,
                        context_evidence,
                        binding_support,
                        seed_support,
                    )
                },
            )
            .collect::<Vec<_>>();
        lemma_hypotheses.sort_by(
            |(left_id, left_total, left_seed, left_context, left_support, left_seed_support),
             (
                right_id,
                right_total,
                right_seed,
                right_context,
                right_support,
                right_seed_support,
            )| {
                right_total
                    .cmp(left_total)
                    .then_with(|| right_context.cmp(left_context))
                    .then_with(|| right_seed.cmp(left_seed))
                    .then_with(|| right_seed_support.cmp(left_seed_support))
                    .then_with(|| right_support.cmp(left_support))
                    .then_with(|| left_id.cmp(right_id))
            },
        );
        let seed_lemmas = lemma_hypotheses
            .into_iter()
            .take(MAX_ACTIVE_LEMMAS)
            .map(|(lemma_id, ..)| lemma_id)
            .collect::<BTreeSet<_>>();
        for lemma_id in &seed_lemmas {
            active_forms.extend(
                self.forms_by_lemma
                    .get(*lemma_id as usize)
                    .into_iter()
                    .flatten()
                    .copied(),
            );
            if let (Some(context_mode_id), Some(lemma)) = (
                context_mode_id,
                self.package.lemma_centers.get(*lemma_id as usize),
            ) {
                let start = lemma.competition_start as usize;
                let end = start.saturating_add(lemma.competition_count as usize);
                for edge in self
                    .package
                    .competition_edges
                    .get(start..end)
                    .unwrap_or_default()
                    .iter()
                    .filter(|edge| edge.context_mode_id == context_mode_id)
                {
                    active_forms.insert(edge.left_form_ref);
                    active_forms.insert(edge.right_form_ref);
                }
            }
        }
        let inherited_l1_floor = seeds
            .iter()
            .map(|seed| seed.evidence_milli)
            .max()
            .unwrap_or_default()
            .saturating_sub(INHERITED_L1_ATTENUATION_MILLI);
        let mut candidates = active_forms
            .into_iter()
            .filter_map(|form_ref| {
                self.score_form(
                    form_ref,
                    context_mode_id,
                    &wave,
                    &seed_evidence,
                    &direct_seed_common_lemmas,
                    seed_evidence
                        .get(&form_ref)
                        .copied()
                        .unwrap_or(inherited_l1_floor),
                )
            })
            .collect::<Vec<_>>();
        candidates.sort_by(|left, right| {
            right
                .local_score
                .cmp(&left.local_score)
                .then_with(|| right.slot_phase_milli.cmp(&left.slot_phase_milli))
                .then_with(|| left.form_ref.cmp(&right.form_ref))
        });
        candidates.truncate(candidate_limit.max(1));
        let verdict = classify_local(&candidates, self.package.calibration);
        StandaloneL2Readout {
            verdict,
            candidates,
            context_mode_id,
        }
    }

    fn score_form(
        &self,
        form_ref: u32,
        context_mode_id: Option<u32>,
        wave: &[i8; L2_PHASE_CELLS],
        direct_seed_evidence: &BTreeMap<u32, i32>,
        direct_seed_common_lemmas: &BTreeSet<u32>,
        l1_evidence_milli: i32,
    ) -> Option<L2LocalCandidate> {
        let form = self.package.form_refs.get(form_ref as usize)?;
        let surface = self.decode_form(*form)?.to_string();
        let bindings = self.bindings_by_form.get(form_ref as usize)?;
        let lemma_ids = bindings
            .iter()
            .map(|binding| binding.lemma_center_id)
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect::<Vec<_>>();
        let feature_masks = bindings
            .iter()
            .map(|binding| binding.feature_mask)
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect::<Vec<_>>();
        let slot_phase_milli = context_mode_id
            .map(|context_mode_id| {
                bindings
                    .iter()
                    .flat_map(|binding| {
                        let slot_features =
                            crate::nanda_wave::morphology_phase::contextual_slot_features(
                                binding.feature_mask,
                            );
                        self.slot_centers_by_mode_feature
                            .get(&(context_mode_id, slot_features))
                            .into_iter()
                            .flatten()
                            .filter_map(|index| self.package.slot_centers.get(*index as usize))
                    })
                    .map(|center| slot_center_score(center, wave))
                    .max()
                    .unwrap_or_default()
            })
            .unwrap_or_default();
        let neighbor_pressure = context_mode_id
            .map(|context_mode_id| {
                bindings
                    .iter()
                    .flat_map(|binding| {
                        self.neighbor_couplings_by_mode_lemma_feature
                            .get(&(
                                context_mode_id,
                                binding.lemma_center_id,
                                binding.feature_mask,
                            ))
                            .into_iter()
                            .flatten()
                            .filter_map(|index| {
                                self.package.neighbor_couplings.get(*index as usize)
                            })
                    })
                    .map(|coupling| i32::from(coupling.support - coupling.repel) * 32)
                    .max()
                    .unwrap_or_default()
            })
            .unwrap_or_default();
        let (competition_pressure, explicit_competition_pressure) = context_mode_id
            .map(|context_mode_id| {
                let competition_unit = INHERITED_L1_ATTENUATION_MILLI
                    .saturating_add(
                        self.package
                            .calibration
                            .minimum_margin
                            .max(self.package.calibration.tie_window)
                            .max(1),
                    )
                    .saturating_add(1);
                lemma_ids
                    .iter()
                    .filter_map(|lemma_id| self.package.lemma_centers.get(*lemma_id as usize))
                    .map(|lemma| {
                        let start = lemma.competition_start as usize;
                        let end = start.saturating_add(lemma.competition_count as usize);
                        self.package
                            .competition_edges
                            .get(start..end)
                            .unwrap_or_default()
                            .iter()
                            .filter(|edge| edge.context_mode_id == context_mode_id)
                            .fold((0_i32, 0_i32), |(total, explicit), edge| {
                                let opposing_form_ref = if edge.left_form_ref == form_ref {
                                    edge.right_form_ref
                                } else if edge.right_form_ref == form_ref {
                                    edge.left_form_ref
                                } else {
                                    return (total, explicit);
                                };
                                let unambiguous_lemma_support = lemma_ids
                                    .iter()
                                    .any(|lemma_id| direct_seed_common_lemmas.contains(lemma_id));
                                if !direct_seed_evidence.contains_key(&opposing_form_ref)
                                    && !unambiguous_lemma_support
                                {
                                    return (total, explicit);
                                }
                                let pressure = if edge.left_form_ref == form_ref {
                                    i32::from(edge.support_delta.min(16))
                                        .saturating_mul(competition_unit)
                                } else if edge.right_form_ref == form_ref {
                                    -i32::from(edge.anti_delta.min(16))
                                        .saturating_mul(competition_unit)
                                } else {
                                    0
                                };
                                (
                                    total.saturating_add(pressure),
                                    explicit.saturating_add(
                                        if edge.flags & COMPETITION_FLAG_EXPLICIT_NEIGHBOR != 0 {
                                            pressure
                                        } else {
                                            0
                                        },
                                    ),
                                )
                            })
                    })
                    .max_by_key(|(total, explicit)| (*total, *explicit))
                    .unwrap_or_default()
            })
            .unwrap_or_default();
        Some(L2LocalCandidate {
            form_ref,
            l1_terminal_id: (form.l1_terminal_id != NO_L1_TERMINAL).then_some(form.l1_terminal_id),
            surface,
            l1_evidence_milli,
            slot_phase_milli,
            neighbor_pressure,
            competition_pressure,
            explicit_competition_pressure,
            local_score: l1_evidence_milli
                .saturating_add(slot_phase_milli)
                .saturating_add(neighbor_pressure)
                .saturating_add(competition_pressure),
            lemma_ids,
            feature_masks,
        })
    }

    fn best_lemma_context_evidence(
        &self,
        lemma_id: u32,
        context_mode_id: Option<u32>,
        wave: &[i8; L2_PHASE_CELLS],
    ) -> i32 {
        let Some(context_mode_id) = context_mode_id else {
            return 0;
        };
        self.forms_by_lemma
            .get(lemma_id as usize)
            .into_iter()
            .flatten()
            .flat_map(|form_ref| &self.bindings_by_form[*form_ref as usize])
            .filter(|binding| binding.lemma_center_id == lemma_id)
            .flat_map(|binding| {
                let slot_features = crate::nanda_wave::morphology_phase::contextual_slot_features(
                    binding.feature_mask,
                );
                self.slot_centers_by_mode_feature
                    .get(&(context_mode_id, slot_features))
                    .into_iter()
                    .flatten()
                    .filter_map(|index| self.package.slot_centers.get(*index as usize))
            })
            .map(|center| slot_center_score(center, wave))
            .max()
            .unwrap_or_default()
    }

    fn decode_form(&self, form: super::model::FormCenterRef) -> Option<&str> {
        let tail = self
            .package
            .decoder_bytes
            .get(form.decoder_ref as usize..)?;
        let length = tail.iter().position(|byte| *byte == 0)?;
        std::str::from_utf8(&tail[..length]).ok()
    }
}

fn classify_local(candidates: &[L2LocalCandidate], calibration: TieCalibration) -> L2LocalVerdict {
    const MAX_SUPPORT_UNCERTAINTY: i32 = 16 * 8;

    let Some(winner) = candidates.first() else {
        return L2LocalVerdict::Abstain;
    };
    if winner
        .slot_phase_milli
        .max(winner.neighbor_pressure)
        .max(winner.competition_pressure)
        < calibration.minimum_positive
    {
        let lexical_peak = winner.l1_evidence_milli;
        let form_refs = candidates
            .iter()
            .take_while(|candidate| candidate.l1_evidence_milli == lexical_peak)
            .map(|candidate| candidate.form_ref)
            .collect::<Vec<_>>();
        return if form_refs.len() > 1 {
            L2LocalVerdict::Tied { form_refs }
        } else {
            L2LocalVerdict::Abstain
        };
    }
    if let Some(verdict) =
        cross_lemma_authority_safety_verdict(candidates, winner, MAX_SUPPORT_UNCERTAINTY)
    {
        return verdict;
    }
    let equivalent_slot = candidates
        .iter()
        .filter(|candidate| {
            let same_lemma = candidate
                .lemma_ids
                .iter()
                .any(|lemma| winner.lemma_ids.contains(lemma));
            let equivalent_positive_slot = candidate.slot_phase_milli
                >= calibration.minimum_positive
                && winner.slot_phase_milli.abs_diff(candidate.slot_phase_milli)
                    <= MAX_SUPPORT_UNCERTAINTY as u32;
            let inclusive_imperative_tie =
                candidate.feature_masks.iter().any(|candidate_features| {
                    winner.feature_masks.iter().any(|winner_features| {
                        crate::nanda_wave::morphology_phase::same_inclusive_imperative_family(
                            *winner_features,
                            *candidate_features,
                        )
                    })
                });
            let finite_agreement_tie = winner.neighbor_pressure <= 0
                && winner.competition_pressure == 0
                && candidate.feature_masks.iter().any(|candidate_features| {
                    winner.feature_masks.iter().any(|winner_features| {
                        crate::nanda_wave::morphology_phase::same_finite_agreement_family(
                            *winner_features,
                            *candidate_features,
                        )
                    })
                });
            same_lemma
                && (equivalent_positive_slot || inclusive_imperative_tie || finite_agreement_tie)
        })
        .map(|candidate| candidate.form_ref)
        .collect::<Vec<_>>();
    if equivalent_slot.len() > 1 {
        let mut form_refs = equivalent_slot;
        for form_ref in candidates
            .iter()
            .take_while(|candidate| {
                winner.local_score.saturating_sub(candidate.local_score) <= calibration.tie_window
            })
            .map(|candidate| candidate.form_ref)
        {
            if !form_refs.contains(&form_ref) {
                form_refs.push(form_ref);
            }
        }
        return L2LocalVerdict::Tied { form_refs };
    }
    let margin = winner.local_score.saturating_sub(
        candidates
            .get(1)
            .map(|candidate| candidate.local_score)
            .unwrap_or_default(),
    );
    if margin <= calibration.tie_window.max(calibration.minimum_margin - 1) {
        return L2LocalVerdict::Tied {
            form_refs: candidates
                .iter()
                .take_while(|candidate| {
                    winner.local_score.saturating_sub(candidate.local_score)
                        <= calibration.tie_window
                })
                .map(|candidate| candidate.form_ref)
                .collect(),
        };
    }
    L2LocalVerdict::Winner {
        form_ref: winner.form_ref,
    }
}

fn cross_lemma_authority_safety_verdict(
    candidates: &[L2LocalCandidate],
    winner: &L2LocalCandidate,
    support_uncertainty: i32,
) -> Option<L2LocalVerdict> {
    let strongest_lexical_seed = candidates
        .iter()
        .map(|candidate| candidate.l1_evidence_milli)
        .max()
        .unwrap_or_default();
    let winner_has_independent_lemma_seed = candidates.iter().any(|candidate| {
        candidate.l1_evidence_milli == strongest_lexical_seed
            && candidate
                .lemma_ids
                .iter()
                .any(|lemma_id| winner.lemma_ids.contains(lemma_id))
    });
    if winner.explicit_competition_pressure > 0 && winner_has_independent_lemma_seed {
        return None;
    }
    let independent_score = |candidate: &L2LocalCandidate| {
        candidate
            .l1_evidence_milli
            .saturating_add(candidate.neighbor_pressure)
    };
    let winner_independent = independent_score(winner);
    let strongest_foreign = candidates
        .iter()
        .filter(|candidate| {
            !candidate
                .lemma_ids
                .iter()
                .any(|lemma_id| winner.lemma_ids.contains(lemma_id))
        })
        .map(independent_score)
        .max()?;
    if winner_independent.saturating_sub(strongest_foreign) > support_uncertainty {
        return None;
    }
    let form_refs = candidates
        .iter()
        .map(|candidate| candidate.form_ref)
        .collect::<Vec<_>>();
    Some(if form_refs.len() > 1 {
        L2LocalVerdict::Tied { form_refs }
    } else {
        L2LocalVerdict::Abstain
    })
}

fn slot_center_score(center: &SlotPhaseCenter, wave: &[i8; L2_PHASE_CELLS]) -> i32 {
    coherence_milli(&center.cells, wave)
        .saturating_mul(i32::from(center.polarity))
        .saturating_add(i32::from(center.support.min(16)) * 8)
}

fn coherence_milli(left: &[i8; L2_PHASE_CELLS], right: &[i8; L2_PHASE_CELLS]) -> i32 {
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

#[cfg(test)]
mod standalone_tests {
    use super::super::compiler::compile_l2_package;
    use super::super::format::encode_package;
    use super::super::teacher::L2TeacherCorpus;
    use super::*;

    #[test]
    fn standalone_field_walks_from_l1_seed_to_contextual_same_lemma_form() {
        let corpus = L2TeacherCorpus::parse_tsv(
            "F\tдом\tдом\tnoun:nom:sg\n\
             F\tдом\tдома\tnoun:gen:sg\n\
             T\tдом\tдом\tnoun:nom:sg\t_ стоит\n\
             T\tдом\tдома\tnoun:gen:sg\tнет _\n\
             H\tдом\tдома\tnoun:gen:sg\tоколо _\n",
        )
        .expect("teacher");
        let terminals = BTreeMap::from([("дом", 17), ("дома", 23)]);
        let (package, _) =
            compile_l2_package(&corpus, 99, |surface| terminals.get(surface).copied())
                .expect("compile");
        let bytes = encode_package(&package).expect("encode");
        let field = StandaloneL2Field::from_bytes(&bytes).expect("load");
        let readout = field.readout(
            "нет _",
            &[L2LexicalSeed {
                terminal_id: Some(17),
                surface: None,
                evidence_milli: 900,
            }],
            8,
        );

        assert_eq!(field.l1_package_fingerprint(), 99);
        assert_eq!(readout.verdict, L2LocalVerdict::Winner { form_ref: 1 });
        assert_eq!(readout.candidates[0].surface, "дома");
    }

    #[test]
    fn standalone_field_materializes_a_form_that_is_absent_from_l1() {
        let corpus = L2TeacherCorpus::parse_tsv(
            "F\tдом\tдом\tnoun:nom:sg\n\
             F\tдом\tдома\tnoun:gen:sg\n\
             T\tдом\tдом\tnoun:nom:sg\t_ стоит\n\
             T\tдом\tдома\tnoun:gen:sg\tнет _\n\
             H\tдом\tдома\tnoun:gen:sg\tоколо _\n",
        )
        .expect("teacher");
        let (package, report) =
            compile_l2_package(&corpus, 99, |surface| (surface == "дом").then_some(17))
                .expect("compile");
        assert_eq!(report.l1_bound_forms, 1);
        assert_eq!(report.admitted_forms, 2);
        let field = StandaloneL2Field::from_package(package).expect("load");
        let readout = field.readout(
            "нет _",
            &[L2LexicalSeed {
                terminal_id: Some(17),
                surface: None,
                evidence_milli: 900,
            }],
            8,
        );

        let L2LocalVerdict::Winner { form_ref } = readout.verdict else {
            panic!("context should settle the generated form");
        };
        let winner = readout
            .candidates
            .iter()
            .find(|candidate| candidate.form_ref == form_ref)
            .expect("winner candidate");
        assert_eq!(winner.surface, "дома");
        assert_eq!(winner.l1_terminal_id, None);
    }

    #[test]
    fn standalone_field_resolves_append_only_l1_seed_by_surface() {
        let corpus = L2TeacherCorpus::parse_tsv(
            "F\tрефакторинг\tрефакторинг\tnoun:nom:sg\n\
             F\tрефакторинг\tрефакторинга\tnoun:gen:sg\n\
             T\tрефакторинг\tрефакторинг\tnoun:nom:sg\t_ нужен\n\
             H\tрефакторинг\tрефакторинга\tnoun:gen:sg\tпроект _\n",
        )
        .expect("teacher");
        let (package, _) = compile_l2_package(&corpus, 99, |surface| {
            (surface == "рефакторинг").then_some(17)
        })
        .expect("compile");
        let field = StandaloneL2Field::from_package(package).expect("load");
        let readout = field.readout(
            "проект _",
            &[L2LexicalSeed {
                terminal_id: Some(900_000),
                surface: Some("рефакторинга".to_string()),
                evidence_milli: 1_000,
            }],
            8,
        );

        assert!(readout
            .candidates
            .iter()
            .any(|candidate| candidate.surface == "рефакторинга"));
    }

    #[test]
    fn learned_competition_overcomes_one_reconstruction_attenuation() {
        let corpus = L2TeacherCorpus::parse_tsv(
            "F\tпосмотреть\tпосмотреть\tverb:inf:perf\n\
             F\tпосмотреть\tпосмотри\tverb:imp_excl:sg:imp:perf\n\
             F\tпросмотреть\tпросмотреть\tverb:inf:perf\n\
             F\tпросмотреть\tпросмотри\tverb:imp_excl:sg:imp:perf\n\
             T\tпосмотреть\tпосмотреть\tverb:inf:perf\tхочу _\n\
             H\tпосмотреть\tпосмотри\tverb:imp_excl:sg:imp:perf\t_ сюда\n\
             NT\tпосмотреть\tпосмотри\tverb:imp_excl:sg:imp:perf\t_ сюда\tпросмотри\n\
             NH\tпосмотреть\tпосмотри\tverb:imp_excl:sg:imp:perf\t_ сюда\tпросмотри\n",
        )
        .expect("teacher");
        let terminals = BTreeMap::from([("посмотреть", 17), ("просмотреть", 23)]);
        let (package, _) =
            compile_l2_package(&corpus, 99, |surface| terminals.get(surface).copied())
                .expect("compile");
        let field = StandaloneL2Field::from_package(package).expect("load");
        let readout = field.readout(
            "_ сюда",
            &[L2LexicalSeed {
                terminal_id: Some(17),
                surface: None,
                evidence_milli: 1_000,
            }],
            8,
        );

        let L2LocalVerdict::Winner { form_ref } = readout.verdict else {
            panic!("learned competition should settle one reconstruction: {readout:#?}");
        };
        assert_eq!(field.decode_form_ref(form_ref), Some("посмотри"));
    }

    #[test]
    fn standalone_field_abstains_when_context_mode_is_unknown() {
        let corpus = L2TeacherCorpus::parse_tsv(
            "F\tдом\tдом\tnoun:nom:sg\n\
             F\tдом\tдома\tnoun:gen:sg\n\
             T\tдом\tдом\tnoun:nom:sg\t_ стоит\n\
             H\tдом\tдома\tnoun:gen:sg\tнет _\n",
        )
        .expect("teacher");
        let terminals = BTreeMap::from([("дом", 17), ("дома", 23)]);
        let (package, _) =
            compile_l2_package(&corpus, 99, |surface| terminals.get(surface).copied())
                .expect("compile");
        let field = StandaloneL2Field::from_package(package).expect("load");
        let readout = field.readout(
            "совсем неизвестная сцена _",
            &[L2LexicalSeed {
                terminal_id: Some(17),
                surface: None,
                evidence_milli: 900,
            }],
            8,
        );
        assert_eq!(readout.verdict, L2LocalVerdict::Abstain);
    }

    #[test]
    fn unknown_context_keeps_multiple_direct_surface_seeds_tied() {
        let corpus = L2TeacherCorpus::parse_tsv(
            "F\tкод\tкод\tnoun:nom:sg\n\
             F\tкот\tкот\tnoun:nom:sg\n\
             T\tкод\tкод\tnoun:nom:sg\t_ работает\n\
             T\tкот\tкот\tnoun:nom:sg\t_ спит\n\
             H\tкод\tкод\tnoun:nom:sg\tпроверяю _\n",
        )
        .expect("teacher");
        let terminals = BTreeMap::from([("код", 17), ("кот", 23)]);
        let (package, _) =
            compile_l2_package(&corpus, 99, |surface| terminals.get(surface).copied())
                .expect("compile");
        let field = StandaloneL2Field::from_package(package).expect("load");
        let readout = field.readout(
            "совсем неизвестная сцена _",
            &[
                L2LexicalSeed {
                    terminal_id: None,
                    surface: Some("код".to_string()),
                    evidence_milli: 1_000,
                },
                L2LexicalSeed {
                    terminal_id: None,
                    surface: Some("кот".to_string()),
                    evidence_milli: 1_000,
                },
            ],
            8,
        );

        assert_eq!(
            readout.verdict,
            L2LocalVerdict::Tied {
                form_refs: vec![0, 1]
            }
        );
    }

    #[test]
    fn contextual_multi_lemma_birth_can_select_a_weaker_seeded_lemma() {
        let corpus = L2TeacherCorpus::parse_tsv(
            "F\tдом\tдом\tnoun:nom:sg\n\
             F\tдом\tдома\tnoun:gen:sg\n\
             F\tдома\tдома\tnoun:nom:sg\n\
             F\tдома\tдомик\tnoun:acc:sg\n\
             T\tдома\tдомик\tnoun:acc:sg\tвижу _\n\
             NT\tдома\tдомик\tnoun:acc:sg\tвижу _\tдома\n\
             H\tдом\tдома\tnoun:gen:sg\tнет _\n",
        )
        .expect("teacher");
        let terminals = BTreeMap::from([("дом", 17), ("дома", 23), ("домик", 31)]);
        let (package, _) =
            compile_l2_package(&corpus, 99, |surface| terminals.get(surface).copied())
                .expect("compile");
        let field = StandaloneL2Field::from_package(package).expect("load");
        let readout = field.readout(
            "вижу _",
            &[
                L2LexicalSeed {
                    terminal_id: Some(17),
                    surface: None,
                    evidence_milli: 1_000,
                },
                L2LexicalSeed {
                    terminal_id: Some(23),
                    surface: None,
                    evidence_milli: 1_000,
                },
            ],
            8,
        );

        assert!(
            readout
                .candidates
                .iter()
                .any(|candidate| candidate.surface == "домик"),
            "{readout:#?}"
        );
        let L2LocalVerdict::Winner { form_ref } = readout.verdict else {
            panic!("contextual slot should settle the weaker seeded lemma");
        };
        assert_eq!(
            readout
                .candidates
                .iter()
                .find(|candidate| candidate.form_ref == form_ref)
                .map(|candidate| candidate.surface.as_str()),
            Some("домик")
        );
    }

    #[test]
    fn equal_slot_evidence_within_one_lemma_cannot_become_false_singleton() {
        let candidates = vec![
            L2LocalCandidate {
                form_ref: 17,
                l1_terminal_id: Some(17),
                surface: "первый".to_string(),
                l1_evidence_milli: 1_000,
                slot_phase_milli: 1_128,
                neighbor_pressure: 0,
                competition_pressure: 128,
                explicit_competition_pressure: 0,
                local_score: 2_256,
                lemma_ids: vec![3, 7],
                feature_masks: vec![11],
            },
            L2LocalCandidate {
                form_ref: 23,
                l1_terminal_id: Some(23),
                surface: "второй".to_string(),
                l1_evidence_milli: 1_000,
                slot_phase_milli: 1_128,
                neighbor_pressure: 0,
                competition_pressure: 0,
                explicit_competition_pressure: 0,
                local_score: 2_128,
                lemma_ids: vec![7],
                feature_masks: vec![11],
            },
        ];

        assert_eq!(
            classify_local(
                &candidates,
                TieCalibration {
                    minimum_positive: 1,
                    minimum_margin: 1,
                    tie_window: 1,
                    ..TieCalibration::default()
                },
            ),
            L2LocalVerdict::Tied {
                form_refs: vec![17, 23]
            }
        );
    }

    #[test]
    fn competition_alone_cannot_create_cross_lemma_authority() {
        let candidates = vec![
            L2LocalCandidate {
                form_ref: 17,
                l1_terminal_id: Some(17),
                surface: "чужая".to_string(),
                l1_evidence_milli: 1_000,
                slot_phase_milli: 1_000,
                neighbor_pressure: 0,
                competition_pressure: 500,
                explicit_competition_pressure: 0,
                local_score: 2_500,
                lemma_ids: vec![2],
                feature_masks: vec![11],
            },
            L2LocalCandidate {
                form_ref: 23,
                l1_terminal_id: Some(23),
                surface: "целевая".to_string(),
                l1_evidence_milli: 1_000,
                slot_phase_milli: 1_000,
                neighbor_pressure: 0,
                competition_pressure: 0,
                explicit_competition_pressure: 0,
                local_score: 2_000,
                lemma_ids: vec![1],
                feature_masks: vec![11],
            },
        ];

        assert_eq!(
            classify_local(
                &candidates,
                TieCalibration {
                    minimum_positive: 1,
                    minimum_margin: 1,
                    tie_window: 1,
                    ..TieCalibration::default()
                },
            ),
            L2LocalVerdict::Tied {
                form_refs: vec![17, 23]
            }
        );
    }

    #[test]
    fn explicit_competition_without_a_winner_lemma_seed_stays_tied() {
        let candidates = vec![
            L2LocalCandidate {
                form_ref: 17,
                l1_terminal_id: None,
                surface: "чужая".to_string(),
                l1_evidence_milli: 760,
                slot_phase_milli: 1_000,
                neighbor_pressure: 0,
                competition_pressure: 500,
                explicit_competition_pressure: 500,
                local_score: 2_260,
                lemma_ids: vec![2],
                feature_masks: vec![11],
            },
            L2LocalCandidate {
                form_ref: 23,
                l1_terminal_id: Some(23),
                surface: "целевая".to_string(),
                l1_evidence_milli: 1_000,
                slot_phase_milli: 1_000,
                neighbor_pressure: 0,
                competition_pressure: 0,
                explicit_competition_pressure: 0,
                local_score: 2_000,
                lemma_ids: vec![1],
                feature_masks: vec![11],
            },
        ];

        assert_eq!(
            classify_local(
                &candidates,
                TieCalibration {
                    minimum_positive: 1,
                    minimum_margin: 1,
                    tie_window: 1,
                    ..TieCalibration::default()
                },
            ),
            L2LocalVerdict::Tied {
                form_refs: vec![17, 23]
            }
        );
    }

    #[test]
    fn inclusive_imperative_variants_in_one_lemma_are_tied() {
        let inclusive_singular =
            crate::nanda_wave::morphology_phase::parse_features("verb:imp_incl:sg:imp:perf")
                .expect("inclusive singular");
        let inclusive_plural =
            crate::nanda_wave::morphology_phase::parse_features("verb:imp_incl:pl:imp:perf")
                .expect("inclusive plural");
        let candidates = vec![
            L2LocalCandidate {
                form_ref: 17,
                l1_terminal_id: Some(17),
                surface: "первый".to_string(),
                l1_evidence_milli: 1_000,
                slot_phase_milli: 1_088,
                neighbor_pressure: 0,
                competition_pressure: 0,
                explicit_competition_pressure: 0,
                local_score: 2_088,
                lemma_ids: vec![7],
                feature_masks: vec![inclusive_singular],
            },
            L2LocalCandidate {
                form_ref: 23,
                l1_terminal_id: Some(23),
                surface: "второй".to_string(),
                l1_evidence_milli: 1_000,
                slot_phase_milli: 0,
                neighbor_pressure: 0,
                competition_pressure: 0,
                explicit_competition_pressure: 0,
                local_score: 1_000,
                lemma_ids: vec![7],
                feature_masks: vec![inclusive_plural],
            },
        ];

        assert_eq!(
            classify_local(
                &candidates,
                TieCalibration {
                    minimum_positive: 1,
                    minimum_margin: 1,
                    tie_window: 1,
                    ..TieCalibration::default()
                },
            ),
            L2LocalVerdict::Tied {
                form_refs: vec![17, 23]
            }
        );
    }
}
