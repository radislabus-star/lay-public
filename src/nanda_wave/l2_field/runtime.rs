use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use crate::correction_core::UnifiedCorrectionCandidate;

use super::context::{context_mode, scene_wave};
use super::format::decode_package;
use super::model::{L2FieldPackage, MorphBinding, SlotPhaseCenter, TieCalibration, L2_PHASE_CELLS};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum L2FieldBridgeKind {
    Shadow,
}

impl L2FieldBridgeKind {
    pub(crate) const fn surface_source_id(self) -> &'static str {
        "L2FieldShadowSurface"
    }

    #[cfg(test)]
    pub(crate) const fn morph_source_id(self) -> &'static str {
        "L2FieldShadowMorphology"
    }

    #[cfg(test)]
    pub(crate) const fn near_neighbor_source_id(self) -> &'static str {
        "L2FieldShadowNearNeighbor"
    }

    pub(crate) const fn readout_source_id(self) -> &'static str {
        "L2FieldShadowReadout"
    }
}

#[derive(Debug, Default)]
pub(crate) struct L2FieldShadowReadout {
    pub(crate) candidates: Vec<UnifiedCorrectionCandidate>,
}

impl L2FieldShadowReadout {
    pub(crate) fn new(candidates: Vec<UnifiedCorrectionCandidate>) -> Self {
        Self { candidates }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct L2LexicalSeed {
    pub(crate) terminal_id: u32,
    pub(crate) evidence_milli: i32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct L2LocalCandidate {
    pub(crate) terminal_id: u32,
    pub(crate) l1_evidence_milli: i32,
    pub(crate) slot_phase_milli: i32,
    pub(crate) neighbor_pressure: i32,
    pub(crate) competition_pressure: i32,
    pub(crate) local_score: i32,
    pub(crate) lemma_ids: Vec<u32>,
    pub(crate) feature_masks: Vec<u32>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum L2LocalVerdict {
    Winner { terminal_id: u32 },
    Tied { terminal_ids: Vec<u32> },
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

    pub(crate) fn package_counts(&self) -> (usize, usize, usize, usize) {
        (
            self.package.form_refs.len(),
            self.package.lemma_centers.len(),
            self.package.morph_bindings.len(),
            self.package.competition_edges.len(),
        )
    }

    pub(crate) fn form_terminal_ids(&self) -> impl Iterator<Item = u32> + '_ {
        self.package
            .form_refs
            .iter()
            .map(|form| form.l1_terminal_id)
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
        let seed_evidence = seeds
            .iter()
            .filter_map(|seed| {
                Some((
                    *self.form_by_terminal.get(&seed.terminal_id)?,
                    seed.evidence_milli,
                ))
            })
            .collect::<BTreeMap<_, _>>();
        let mut active_forms = seed_evidence.keys().copied().collect::<BTreeSet<_>>();
        let mut lemma_seed_counts = BTreeMap::<u32, usize>::new();
        for form_ref in &active_forms {
            for lemma_id in self.bindings_by_form[*form_ref as usize]
                .iter()
                .map(|binding| binding.lemma_center_id)
                .collect::<BTreeSet<_>>()
            {
                *lemma_seed_counts.entry(lemma_id).or_default() += 1;
            }
        }
        let strongest_lemma_count = lemma_seed_counts
            .values()
            .copied()
            .max()
            .unwrap_or_default();
        let seed_lemmas = lemma_seed_counts
            .into_iter()
            .filter_map(|(lemma_id, count)| (count == strongest_lemma_count).then_some(lemma_id))
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
            .saturating_sub(240);
        let mut candidates = active_forms
            .into_iter()
            .filter_map(|form_ref| {
                self.score_form(
                    form_ref,
                    context_mode_id,
                    &wave,
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
                .then_with(|| left.terminal_id.cmp(&right.terminal_id))
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
        l1_evidence_milli: i32,
    ) -> Option<L2LocalCandidate> {
        let form = self.package.form_refs.get(form_ref as usize)?;
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
        let competition_pressure = context_mode_id
            .map(|context_mode_id| {
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
                            .map(|edge| {
                                if edge.left_form_ref == form_ref {
                                    i32::from(edge.support_delta) * 24
                                } else if edge.right_form_ref == form_ref {
                                    -i32::from(edge.anti_delta) * 24
                                } else {
                                    0
                                }
                            })
                            .sum::<i32>()
                    })
                    .max()
                    .unwrap_or_default()
            })
            .unwrap_or_default();
        Some(L2LocalCandidate {
            terminal_id: form.l1_terminal_id,
            l1_evidence_milli,
            slot_phase_milli,
            neighbor_pressure,
            competition_pressure,
            local_score: l1_evidence_milli
                .saturating_add(slot_phase_milli)
                .saturating_add(neighbor_pressure)
                .saturating_add(competition_pressure),
            lemma_ids,
            feature_masks,
        })
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
        return L2LocalVerdict::Abstain;
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
            same_lemma && (equivalent_positive_slot || inclusive_imperative_tie)
        })
        .map(|candidate| candidate.terminal_id)
        .collect::<Vec<_>>();
    if equivalent_slot.len() > 1 {
        return L2LocalVerdict::Tied {
            terminal_ids: equivalent_slot,
        };
    }
    let margin = winner.local_score.saturating_sub(
        candidates
            .get(1)
            .map(|candidate| candidate.local_score)
            .unwrap_or_default(),
    );
    if margin <= calibration.tie_window.max(calibration.minimum_margin - 1) {
        return L2LocalVerdict::Tied {
            terminal_ids: candidates
                .iter()
                .take_while(|candidate| {
                    winner.local_score.saturating_sub(candidate.local_score)
                        <= calibration.tie_window
                })
                .map(|candidate| candidate.terminal_id)
                .collect(),
        };
    }
    L2LocalVerdict::Winner {
        terminal_id: winner.terminal_id,
    }
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
                terminal_id: 17,
                evidence_milli: 900,
            }],
            8,
        );

        assert_eq!(field.l1_package_fingerprint(), 99);
        assert_eq!(readout.verdict, L2LocalVerdict::Winner { terminal_id: 23 });
        assert_eq!(readout.candidates[0].terminal_id, 23);
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
                terminal_id: 17,
                evidence_milli: 900,
            }],
            8,
        );
        assert_eq!(readout.verdict, L2LocalVerdict::Abstain);
    }

    #[test]
    fn homonymous_seed_does_not_expand_a_weaker_foreign_lemma() {
        let corpus = L2TeacherCorpus::parse_tsv(
            "F\tдом\tдом\tnoun:nom:sg\n\
             F\tдом\tдома\tnoun:gen:sg\n\
             F\tдома\tdома\tnoun:nom:sg\n\
             F\tдома\tдомик\tnoun:acc:sg\n\
             T\tдома\tдомик\tnoun:acc:sg\tвижу _\n\
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
                    terminal_id: 17,
                    evidence_milli: 1_000,
                },
                L2LexicalSeed {
                    terminal_id: 23,
                    evidence_milli: 1_000,
                },
            ],
            8,
        );

        assert!(readout
            .candidates
            .iter()
            .all(|candidate| candidate.terminal_id != 31));
    }

    #[test]
    fn equal_slot_evidence_within_one_lemma_cannot_become_false_singleton() {
        let candidates = vec![
            L2LocalCandidate {
                terminal_id: 17,
                l1_evidence_milli: 1_000,
                slot_phase_milli: 1_128,
                neighbor_pressure: 0,
                competition_pressure: 128,
                local_score: 2_256,
                lemma_ids: vec![3, 7],
                feature_masks: vec![11],
            },
            L2LocalCandidate {
                terminal_id: 23,
                l1_evidence_milli: 1_000,
                slot_phase_milli: 1_128,
                neighbor_pressure: 0,
                competition_pressure: 0,
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
                }
            ),
            L2LocalVerdict::Tied {
                terminal_ids: vec![17, 23]
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
                terminal_id: 17,
                l1_evidence_milli: 1_000,
                slot_phase_milli: 1_088,
                neighbor_pressure: 0,
                competition_pressure: 0,
                local_score: 2_088,
                lemma_ids: vec![7],
                feature_masks: vec![inclusive_singular],
            },
            L2LocalCandidate {
                terminal_id: 23,
                l1_evidence_milli: 1_000,
                slot_phase_milli: 0,
                neighbor_pressure: 0,
                competition_pressure: 0,
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
                }
            ),
            L2LocalVerdict::Tied {
                terminal_ids: vec![17, 23]
            }
        );
    }
}
