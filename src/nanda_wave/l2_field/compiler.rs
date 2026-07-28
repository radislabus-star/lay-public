use std::collections::{BTreeMap, BTreeSet};

use super::context::{context_mode, scene_wave};
use super::model::{
    CompetitionEdge, FormCenterRef, L2FieldPackage, LemmaCenter, LocalContextMode, MorphBinding,
    NeighborCoupling, SlotPhaseCenter, TieCalibration, L2_PHASE_CELLS,
};
use super::teacher::{L2TeacherCorpus, TeacherNeighborScene, TeacherScene};

const MAX_COMPETITORS_PER_SCENE: usize = 4;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct L2CompileReport {
    pub(crate) source_forms: usize,
    pub(crate) source_unique_surfaces: usize,
    pub(crate) admitted_forms: usize,
    pub(crate) missing_l1_forms: usize,
    pub(crate) lemma_centers: usize,
    pub(crate) morph_bindings: usize,
    pub(crate) context_modes: usize,
    pub(crate) slot_centers: usize,
    pub(crate) neighbor_couplings: usize,
    pub(crate) competition_edges: usize,
    pub(crate) train_scenes: usize,
    pub(crate) heldout_scenes: usize,
}

pub(crate) fn compile_l2_package(
    corpus: &L2TeacherCorpus,
    l1_package_fingerprint: u64,
    mut resolve_terminal: impl FnMut(&str) -> Option<u32>,
) -> Result<(L2FieldPackage, L2CompileReport), String> {
    let mut report = L2CompileReport {
        source_forms: corpus.forms.len(),
        train_scenes: corpus.scenes.iter().filter(|scene| !scene.heldout).count()
            + corpus
                .neighbor_scenes
                .iter()
                .filter(|scene| !scene.heldout)
                .count(),
        heldout_scenes: corpus.scenes.iter().filter(|scene| scene.heldout).count()
            + corpus
                .neighbor_scenes
                .iter()
                .filter(|scene| scene.heldout)
                .count(),
        ..L2CompileReport::default()
    };

    let mut terminal_by_surface = BTreeMap::<String, u32>::new();
    let unique_surfaces = corpus
        .forms
        .iter()
        .map(|form| form.surface.as_str())
        .collect::<BTreeSet<_>>();
    report.source_unique_surfaces = unique_surfaces.len();
    for surface in unique_surfaces {
        if let Some(terminal_id) = resolve_terminal(surface) {
            terminal_by_surface.insert(surface.to_string(), terminal_id);
        } else {
            report.missing_l1_forms += 1;
        }
    }
    if terminal_by_surface.is_empty() {
        return Err("no teacher forms resolve to L1.1 terminal IDs".to_string());
    }

    let admitted_lemmas = corpus
        .forms
        .iter()
        .filter(|form| terminal_by_surface.contains_key(&form.surface))
        .map(|form| form.lemma.clone())
        .collect::<BTreeSet<_>>();
    let lemma_ids = admitted_lemmas
        .into_iter()
        .enumerate()
        .map(|(index, lemma)| (lemma, index as u32))
        .collect::<BTreeMap<_, _>>();
    let form_ids = terminal_by_surface
        .iter()
        .enumerate()
        .map(|(index, (surface, _))| (surface.clone(), index as u32))
        .collect::<BTreeMap<_, _>>();
    let form_refs = terminal_by_surface
        .iter()
        .map(|(surface, terminal_id)| FormCenterRef {
            l1_terminal_id: *terminal_id,
            decoder_ref: *terminal_id,
            script_flags: script_flags(surface),
            length_bucket: surface.chars().count().min(u8::MAX as usize) as u8,
            flags: 0,
            reserved: 0,
        })
        .collect::<Vec<_>>();

    let mut morph_bindings = corpus
        .forms
        .iter()
        .filter_map(|form| {
            Some(MorphBinding {
                form_center_ref: *form_ids.get(&form.surface)?,
                lemma_center_id: *lemma_ids.get(&form.lemma)?,
                feature_mask: form.feature_mask,
                support: 0,
                phase: 0,
                flags: 0,
            })
        })
        .collect::<Vec<_>>();
    morph_bindings.sort_by_key(|binding| {
        (
            binding.lemma_center_id,
            binding.form_center_ref,
            binding.feature_mask,
        )
    });
    morph_bindings.dedup_by_key(|binding| {
        (
            binding.lemma_center_id,
            binding.form_center_ref,
            binding.feature_mask,
        )
    });

    let train_scenes = corpus
        .scenes
        .iter()
        .filter(|scene| !scene.heldout)
        .filter(|scene| {
            lemma_ids.contains_key(&scene.lemma) && form_ids.contains_key(&scene.surface)
        })
        .collect::<Vec<_>>();
    accumulate_binding_support(&mut morph_bindings, &lemma_ids, &form_ids, &train_scenes);

    let train_neighbor_scenes = corpus
        .neighbor_scenes
        .iter()
        .filter(|scene| !scene.heldout)
        .collect::<Vec<_>>();
    let (context_modes, context_ids) = compile_context_modes(&train_scenes, &train_neighbor_scenes);
    let slot_centers = compile_slot_centers(&train_scenes, &context_ids);
    let neighbor_couplings = compile_neighbor_couplings(&train_scenes, &lemma_ids, &context_ids);
    let (competition_edges, competition_lemma_ids) = compile_competition_edges(
        &train_scenes,
        &lemma_ids,
        &form_ids,
        &morph_bindings,
        &context_ids,
        &train_neighbor_scenes,
    );
    let lemma_centers = compile_lemma_centers(
        lemma_ids.len(),
        &morph_bindings,
        &neighbor_couplings,
        &competition_lemma_ids,
    );
    let calibration = calibrate_from_evidence(corpus, &terminal_by_surface);

    let package = L2FieldPackage {
        l1_package_fingerprint,
        form_refs,
        lemma_centers,
        morph_bindings,
        context_modes,
        slot_centers,
        neighbor_couplings,
        competition_edges,
        calibration,
    };
    report.admitted_forms = package.form_refs.len();
    report.lemma_centers = package.lemma_centers.len();
    report.morph_bindings = package.morph_bindings.len();
    report.context_modes = package.context_modes.len();
    report.slot_centers = package.slot_centers.len();
    report.neighbor_couplings = package.neighbor_couplings.len();
    report.competition_edges = package.competition_edges.len();
    Ok((package, report))
}

fn accumulate_binding_support(
    bindings: &mut [MorphBinding],
    lemma_ids: &BTreeMap<String, u32>,
    form_ids: &BTreeMap<String, u32>,
    scenes: &[&TeacherScene],
) {
    let observations = scenes
        .iter()
        .filter_map(|scene| {
            Some((
                *lemma_ids.get(&scene.lemma)?,
                *form_ids.get(&scene.surface)?,
                scene.feature_mask,
            ))
        })
        .fold(BTreeMap::<(u32, u32, u32), u16>::new(), |mut map, key| {
            let count = map.entry(key).or_default();
            *count = count.saturating_add(1);
            map
        });
    for binding in bindings {
        binding.support = observations
            .get(&(
                binding.lemma_center_id,
                binding.form_center_ref,
                binding.feature_mask,
            ))
            .copied()
            .unwrap_or_default();
        binding.phase = i8::from(binding.support > 0);
    }
}

fn compile_context_modes(
    scenes: &[&TeacherScene],
    neighbor_scenes: &[&TeacherNeighborScene],
) -> (Vec<LocalContextMode>, BTreeMap<u32, u32>) {
    let unique = scenes
        .iter()
        .map(|scene| scene.context.as_str())
        .chain(neighbor_scenes.iter().map(|scene| scene.context.as_str()))
        .map(context_mode)
        .map(|mode| (mode.stable_key, mode))
        .collect::<BTreeMap<_, _>>();
    let ids = unique
        .keys()
        .enumerate()
        .map(|(index, key)| (*key, index as u32))
        .collect();
    (unique.into_values().collect(), ids)
}

fn compile_slot_centers(
    scenes: &[&TeacherScene],
    context_ids: &BTreeMap<u32, u32>,
) -> Vec<SlotPhaseCenter> {
    let mut observations = BTreeMap::<(u32, u32), (u16, [i32; L2_PHASE_CELLS])>::new();
    for scene in scenes {
        let mode = context_mode(&scene.context);
        let Some(context_mode_id) = context_ids.get(&mode.stable_key).copied() else {
            continue;
        };
        let wave = scene_wave(&scene.context);
        let slot_features =
            crate::nanda_wave::morphology_phase::contextual_slot_features(scene.feature_mask);
        let (support, sum) = observations
            .entry((context_mode_id, slot_features))
            .or_insert((0, [0; L2_PHASE_CELLS]));
        *support = support.saturating_add(1);
        for (target, observed) in sum.iter_mut().zip(wave) {
            *target = target.saturating_add(i32::from(observed));
        }
    }
    observations
        .into_iter()
        .map(|((context_mode_id, feature_mask), (support, sum))| {
            let mut cells = [0_i8; L2_PHASE_CELLS];
            for (target, total) in cells.iter_mut().zip(sum) {
                *target = (total / i32::from(support.max(1)))
                    .clamp(i32::from(i8::MIN), i32::from(i8::MAX)) as i8;
            }
            SlotPhaseCenter {
                cells,
                feature_mask,
                context_mode_id,
                support,
                mass: cells
                    .iter()
                    .map(|cell| u16::from(cell.unsigned_abs()))
                    .sum(),
                polarity: 1,
                flags: 0,
                reserved: 0,
            }
        })
        .collect()
}

fn compile_neighbor_couplings(
    scenes: &[&TeacherScene],
    lemma_ids: &BTreeMap<String, u32>,
    context_ids: &BTreeMap<u32, u32>,
) -> Vec<NeighborCoupling> {
    let mut support = BTreeMap::<(u32, u32, u32, u32), u16>::new();
    for scene in scenes {
        let Some(lemma_id) = lemma_ids.get(&scene.lemma).copied() else {
            continue;
        };
        let mode = context_mode(&scene.context);
        let Some(context_mode_id) = context_ids.get(&mode.stable_key).copied() else {
            continue;
        };
        let key = (
            context_mode_id,
            lemma_id,
            scene.feature_mask,
            mode.lexical_anchor,
        );
        let count = support.entry(key).or_default();
        *count = count.saturating_add(1);
    }
    let mut couplings = support
        .into_iter()
        .map(
            |((context_mode_id, target_lemma_id, target_feature_mask, source_anchor), count)| {
                NeighborCoupling {
                    context_mode_id,
                    target_lemma_id,
                    target_feature_mask,
                    support: count.min(i16::MAX as u16) as i16,
                    repel: 0,
                    source_anchor,
                    flags: 0,
                    reserved: 0,
                }
            },
        )
        .collect::<Vec<_>>();
    couplings.sort_by_key(|coupling| {
        (
            coupling.target_lemma_id,
            coupling.context_mode_id,
            coupling.target_feature_mask,
            coupling.source_anchor,
        )
    });
    couplings
}

fn compile_competition_edges(
    scenes: &[&TeacherScene],
    lemma_ids: &BTreeMap<String, u32>,
    form_ids: &BTreeMap<String, u32>,
    bindings: &[MorphBinding],
    context_ids: &BTreeMap<u32, u32>,
    neighbor_scenes: &[&TeacherNeighborScene],
) -> (Vec<CompetitionEdge>, Vec<u32>) {
    let mut forms_by_lemma = BTreeMap::<u32, Vec<u32>>::new();
    for binding in bindings {
        forms_by_lemma
            .entry(binding.lemma_center_id)
            .or_default()
            .push(binding.form_center_ref);
    }
    for forms in forms_by_lemma.values_mut() {
        forms.sort_unstable();
        forms.dedup();
    }
    let mut evidence = BTreeMap::<(u32, u32, u32, u32), u32>::new();
    for scene in scenes {
        let Some(lemma_id) = lemma_ids.get(&scene.lemma).copied() else {
            continue;
        };
        let Some(winner) = form_ids.get(&scene.surface).copied() else {
            continue;
        };
        let mode = context_mode(&scene.context);
        let Some(context_mode_id) = context_ids.get(&mode.stable_key).copied() else {
            continue;
        };
        for competitor in forms_by_lemma
            .get(&lemma_id)
            .into_iter()
            .flatten()
            .copied()
            .filter(|candidate| *candidate != winner)
            .take(MAX_COMPETITORS_PER_SCENE)
        {
            let count = evidence
                .entry((lemma_id, winner, competitor, context_mode_id))
                .or_default();
            *count = count.saturating_add(1);
        }
    }
    let mut lemmas_by_form = BTreeMap::<u32, BTreeSet<u32>>::new();
    for binding in bindings {
        lemmas_by_form
            .entry(binding.form_center_ref)
            .or_default()
            .insert(binding.lemma_center_id);
    }
    for scene in neighbor_scenes {
        let (Some(target_lemma), Some(winner)) = (
            lemma_ids.get(&scene.lemma).copied(),
            form_ids.get(&scene.surface).copied(),
        ) else {
            continue;
        };
        let mode = context_mode(&scene.context);
        let Some(context_mode_id) = context_ids.get(&mode.stable_key).copied() else {
            continue;
        };
        for competitor in scene
            .competitors
            .iter()
            .filter_map(|surface| form_ids.get(surface).copied())
            .filter(|competitor| *competitor != winner)
        {
            let mut owners = BTreeSet::from([target_lemma]);
            owners.extend(
                lemmas_by_form
                    .get(&competitor)
                    .into_iter()
                    .flatten()
                    .copied(),
            );
            for owner in owners {
                let count = evidence
                    .entry((owner, winner, competitor, context_mode_id))
                    .or_default();
                *count = count.saturating_add(1);
            }
        }
    }
    let compiled = evidence
        .into_iter()
        .map(
            |((lemma_id, left_form_ref, right_form_ref, context_mode_id), evidence)| {
                (
                    lemma_id,
                    CompetitionEdge {
                        left_form_ref,
                        right_form_ref,
                        context_mode_id,
                        support_delta: evidence.min(i16::MAX as u32) as i16,
                        anti_delta: evidence.min(i16::MAX as u32) as i16,
                        evidence,
                        flags: 0,
                        reserved: 0,
                    },
                )
            },
        )
        .collect::<Vec<_>>();
    let lemma_ids = compiled.iter().map(|(lemma_id, _)| *lemma_id).collect();
    let edges = compiled.into_iter().map(|(_, edge)| edge).collect();
    (edges, lemma_ids)
}

fn compile_lemma_centers(
    lemma_count: usize,
    bindings: &[MorphBinding],
    couplings: &[NeighborCoupling],
    edge_lemma_ids: &[u32],
) -> Vec<LemmaCenter> {
    (0..lemma_count as u32)
        .map(|lemma_id| {
            let form_range = equal_range(bindings, |binding| binding.lemma_center_id, lemma_id);
            let context_range =
                equal_range(couplings, |coupling| coupling.target_lemma_id, lemma_id);
            let competition_start = edge_lemma_ids.partition_point(|owner| *owner < lemma_id);
            let competition_end = edge_lemma_ids.partition_point(|owner| *owner <= lemma_id);
            let primary_pos = bindings
                .get(form_range.0 as usize..form_range.0 as usize + form_range.1 as usize)
                .into_iter()
                .flatten()
                .map(|binding| {
                    crate::nanda_wave::morphology_phase::feature_primary_pos(binding.feature_mask)
                })
                .find(|pos| *pos != 0)
                .unwrap_or_default();
            LemmaCenter {
                primary_pos,
                flags: 0,
                form_start: form_range.0,
                form_count: form_range.1,
                local_context_start: context_range.0,
                local_context_count: context_range.1,
                competition_start: competition_start as u32,
                competition_count: competition_end.saturating_sub(competition_start) as u32,
                reserved: 0,
            }
        })
        .collect()
}

fn equal_range<T>(values: &[T], key: impl Fn(&T) -> u32, target: u32) -> (u32, u32) {
    let start = values.partition_point(|value| key(value) < target);
    let end = values.partition_point(|value| key(value) <= target);
    (start as u32, end.saturating_sub(start) as u32)
}

fn calibrate_from_evidence(
    corpus: &L2TeacherCorpus,
    terminal_by_surface: &BTreeMap<String, u32>,
) -> TieCalibration {
    let train = corpus
        .scenes
        .iter()
        .filter(|scene| !scene.heldout && terminal_by_surface.contains_key(&scene.surface))
        .count();
    let heldout = corpus
        .scenes
        .iter()
        .filter(|scene| scene.heldout && terminal_by_surface.contains_key(&scene.surface))
        .count();
    TieCalibration {
        minimum_positive: 1,
        minimum_margin: 1,
        tie_window: i32::from((heldout > train).then_some(2).unwrap_or(1)),
        abstain_window: 0,
        false_authority_ceiling_milli: 0,
        flags: 0,
        evidence_count: train.saturating_add(heldout).min(u32::MAX as usize) as u32,
    }
}

fn script_flags(surface: &str) -> u16 {
    let cyrillic = surface.chars().any(crate::keyboard::is_cyrillic_letter);
    let latin = surface.chars().any(|ch| ch.is_ascii_alphabetic());
    u16::from(cyrillic) | (u16::from(latin) << 1)
}

#[cfg(test)]
mod tests {
    use super::super::format::{decode_package, encode_package};
    use super::*;

    #[test]
    fn compiler_reuses_l1_terminal_ids_and_builds_all_memory_sections() {
        let corpus = L2TeacherCorpus::parse_tsv(
            "F\tдом\tдом\tnoun:nom:sg\n\
             F\tдом\tдома\tnoun:gen:sg\n\
             T\tдом\tдом\tnoun:nom:sg\t_ стоит\n\
             T\tдом\tдома\tnoun:gen:sg\tнет _\n\
             H\tдом\tдома\tnoun:gen:sg\tоколо _\n",
        )
        .expect("teacher");
        let terminals = BTreeMap::from([("дом", 17), ("дома", 23)]);
        let (package, report) =
            compile_l2_package(&corpus, 99, |surface| terminals.get(surface).copied())
                .expect("compile");

        assert_eq!(
            package
                .form_refs
                .iter()
                .map(|form| form.l1_terminal_id)
                .collect::<Vec<_>>(),
            vec![17, 23]
        );
        assert_eq!(report.admitted_forms, 2);
        assert_eq!(report.lemma_centers, 1);
        assert_eq!(report.morph_bindings, 2);
        assert!(!package.context_modes.is_empty());
        assert!(!package.slot_centers.is_empty());
        assert!(!package.neighbor_couplings.is_empty());
        assert!(!package.competition_edges.is_empty());
        let encoded = encode_package(&package).expect("encode");
        assert_eq!(decode_package(&encoded), Ok(package));
    }
}
