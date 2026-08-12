use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::fs::File;
use std::io::{self, BufRead, BufReader};
use std::path::Path;

use super::compositional::{
    prepared_similarity_to_normalized_surface_with_workspace_milli, surface_scoring_profile,
    SurfaceGeometryWorkspace, SurfaceScoringProfile,
};
use super::context::{context_evidence_lanes, scoped_context_evidence_key, ContextEvidenceScope};
use super::runtime_storage::RuntimeL2Package;

const MAX_FAMILY_SUFFIX_CHARS: usize = 8;
const MAX_PREFIX_CHARS: usize = 4;
const MAX_REMOVE_CHARS: usize = 8;
const MAX_APPEND_CHARS: usize = 12;
const MIN_SHARED_STEM_CHARS: usize = 2;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum ProductiveBirthStatus {
    ShadowUnverified,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct ProductiveFormBirth {
    pub(super) surface: String,
    pub(super) source_feature_mask: u32,
    pub(super) target_feature_mask: u32,
    pub(super) geometry_evidence_milli: u16,
    pub(super) profile_evidence_milli: u16,
    pub(super) positive_support: u32,
    pub(super) anti_support: u32,
    pub(super) family_specificity: u8,
    pub(super) status: ProductiveBirthStatus,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(super) struct ProductiveIndexReport {
    pub(super) observed_lemmas: usize,
    pub(super) admitted_lemmas: usize,
    pub(super) observed_transforms: usize,
    pub(super) admitted_profiles: usize,
    pub(super) rejected_low_support_profiles: usize,
    pub(super) observed_context_rows: usize,
    pub(super) admitted_context_rows: usize,
    pub(super) excluded_context_rows: usize,
    pub(super) rejected_context_rows: usize,
    pub(super) context_modes: usize,
    pub(super) context_slots: usize,
    pub(super) observed_competitor_rows: usize,
    pub(super) observed_competitor_surfaces: usize,
    pub(super) same_lemma_competitor_surfaces: usize,
    pub(super) admitted_pair_observations: usize,
    pub(super) context_pairs: usize,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) struct ProductiveContextSlotEvidence {
    pub(super) positive_support: u32,
    pub(super) unlabeled_alternative_support: u32,
    pub(super) posterior_milli: u16,
    pub(super) context_observed: bool,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) struct ProductiveContextPairEvidence {
    pub(super) positive_support: u32,
    pub(super) anti_support: u32,
    pub(super) posterior_milli: u16,
    pub(super) context_observed: bool,
    pub(super) exact_positive_support: u32,
    pub(super) exact_anti_support: u32,
    pub(super) supporting_neighbor_lanes: u8,
    pub(super) contradicting_neighbor_lanes: u8,
    pub(super) tied_neighbor_lanes: u8,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct ProductiveRuleKey {
    primary_pos: u16,
    source_feature_mask: u32,
    target_feature_mask: u32,
    family_suffix: String,
    remove_prefix_chars: u8,
    prepend: String,
    remove_chars: u8,
    append: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct ProductiveRule {
    pub(super) primary_pos: u16,
    pub(super) source_feature_mask: u32,
    pub(super) target_feature_mask: u32,
    pub(super) family_suffix: String,
    pub(super) remove_prefix_chars: u8,
    pub(super) prepend: String,
    pub(super) remove_chars: u8,
    pub(super) append: String,
    pub(super) positive_support: u32,
    pub(super) anti_support: u32,
}

impl ProductiveRule {
    fn family_specificity(&self) -> u8 {
        self.family_suffix.chars().count().min(u8::MAX as usize) as u8
    }

    fn profile_evidence_milli(&self) -> u16 {
        laplace_posterior_milli(self.positive_support, self.anti_support)
    }
}

#[derive(Clone, Debug, Default)]
pub(super) struct ProductiveMorphologyIndex {
    pub(super) rules_by_family: BTreeMap<(u16, u32, u32, String), Vec<ProductiveRule>>,
    pub(super) target_features_by_source: BTreeMap<(u16, u32), Vec<u32>>,
    pub(super) context_slots: BTreeMap<(u32, u32), ProductiveContextSlotEvidence>,
    pub(super) known_contexts: BTreeMap<u32, u32>,
    pub(super) context_pair_support: BTreeMap<(u32, u32, u32), u32>,
    pub(super) report: ProductiveIndexReport,
}

pub(super) trait ProductiveMorphologySource: Sync {
    fn target_features_vec(&self, primary_pos: u16, source_feature_mask: u32) -> Vec<u32>;

    fn context_slot_evidence_for(
        &self,
        context: &str,
        target_feature_mask: u32,
    ) -> ProductiveContextSlotEvidence;

    fn context_pair_evidence_for(
        &self,
        context: &str,
        preferred_feature_mask: u32,
        competitor_feature_mask: u32,
    ) -> ProductiveContextPairEvidence;

    fn generate_forms(
        &self,
        observed_surface: &str,
        primary_pos: u16,
        source_surface: &str,
        source_feature_mask: u32,
        target_feature_mask: u32,
        limit: usize,
    ) -> Vec<ProductiveFormBirth>;

    fn generate_forms_prepared(
        &self,
        prepared: &PreparedProductiveGeneration<'_>,
        primary_pos: u16,
        source_feature_mask: u32,
        target_feature_mask: u32,
        limit: usize,
        _geometry_by_surface: &mut HashMap<String, u16>,
        _geometry_workspace: &mut SurfaceGeometryWorkspace,
    ) -> Vec<ProductiveFormBirth> {
        self.generate_forms(
            prepared.observed_surface,
            primary_pos,
            prepared.source_surface,
            source_feature_mask,
            target_feature_mask,
            limit,
        )
    }
}

pub(super) struct PreparedProductiveGeneration<'a> {
    pub(super) observed_surface: &'a str,
    pub(super) observed_profile: &'a SurfaceScoringProfile,
    pub(super) source_surface: &'a str,
    pub(super) source_chars: &'a [char],
    pub(super) family_suffixes: &'a [String],
    pub(super) family_lane_starts: &'a [usize],
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct ProductiveForm {
    pub(super) form_ref: u32,
    pub(super) surface: String,
    pub(super) feature_mask: u32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct ProductiveLemma {
    pub(super) lemma_id: u32,
    pub(super) primary_pos: u16,
    pub(super) forms: Vec<ProductiveForm>,
}

#[derive(Debug)]
struct PendingContextCompetition {
    lemma: String,
    context: String,
    primary_pos: u16,
    preferred_feature_mask: u32,
    competitors: Vec<String>,
}

#[derive(Default)]
struct ContextTrainingAccumulator {
    context_totals: BTreeMap<u32, u32>,
    positive: BTreeMap<(u32, u32), u32>,
    pending_competitions: Vec<PendingContextCompetition>,
    wanted_lemma_surfaces: BTreeMap<String, BTreeSet<String>>,
}

impl ProductiveMorphologyIndex {
    pub(super) fn train_from_package(
        package: &RuntimeL2Package,
        include_lemma: impl Fn(u32) -> bool,
        minimum_support: u32,
    ) -> Result<Self, String> {
        let mut observations = BTreeMap::<ProductiveRuleKey, u32>::new();
        let mut family_totals = BTreeMap::<(u16, u32, u32, String), u32>::new();
        let mut report = ProductiveIndexReport {
            observed_lemmas: package.lemma_centers().len(),
            ..ProductiveIndexReport::default()
        };
        for lemma_id in 0..package.lemma_centers().len() as u32 {
            if !include_lemma(lemma_id) {
                continue;
            }
            let lemma = package_lemma(package, lemma_id)?;
            observe_lemma(&lemma, &mut observations, &mut family_totals, &mut report);
        }
        Ok(finish_training(
            observations,
            family_totals,
            report,
            minimum_support,
        ))
    }

    fn train_lemmas(
        lemmas: &[ProductiveLemma],
        include_lemma: impl Fn(u32) -> bool,
        minimum_support: u32,
    ) -> Self {
        let mut observations = BTreeMap::<ProductiveRuleKey, u32>::new();
        let mut family_totals = BTreeMap::<(u16, u32, u32, String), u32>::new();
        let mut report = ProductiveIndexReport {
            observed_lemmas: lemmas.len(),
            ..ProductiveIndexReport::default()
        };

        for lemma in lemmas.iter().filter(|lemma| include_lemma(lemma.lemma_id)) {
            observe_lemma(lemma, &mut observations, &mut family_totals, &mut report);
        }
        finish_training(observations, family_totals, report, minimum_support)
    }

    pub(super) fn generate(
        &self,
        observed_surface: &str,
        primary_pos: u16,
        source_surface: &str,
        source_feature_mask: u32,
        target_feature_mask: u32,
        limit: usize,
    ) -> Vec<ProductiveFormBirth> {
        let source_chars = source_surface.chars().collect::<Vec<_>>();
        let observed_profile = surface_scoring_profile(observed_surface);
        let family_suffixes = productive_family_suffixes(&source_chars);
        let family_lane_starts = productive_family_lane_starts(source_surface, observed_surface);
        let mut geometry_by_surface = HashMap::new();
        let mut geometry_workspace = SurfaceGeometryWorkspace::default();
        self.generate_prepared(
            &PreparedProductiveGeneration {
                observed_surface,
                observed_profile: &observed_profile,
                source_surface,
                source_chars: &source_chars,
                family_suffixes: &family_suffixes,
                family_lane_starts: &family_lane_starts,
            },
            primary_pos,
            source_feature_mask,
            target_feature_mask,
            limit,
            &mut geometry_by_surface,
            &mut geometry_workspace,
        )
    }

    fn generate_prepared(
        &self,
        prepared: &PreparedProductiveGeneration<'_>,
        primary_pos: u16,
        source_feature_mask: u32,
        target_feature_mask: u32,
        limit: usize,
        geometry_by_surface: &mut HashMap<String, u16>,
        geometry_workspace: &mut SurfaceGeometryWorkspace,
    ) -> Vec<ProductiveFormBirth> {
        if limit == 0 {
            return Vec::new();
        }
        let mut by_surface = Vec::<ProductiveFormBirth>::new();
        for &lane_start in prepared.family_lane_starts {
            for specificity in (0..=lane_start).rev() {
                let Some(family_suffix) = prepared.family_suffixes.get(specificity) else {
                    continue;
                };
                let Some(rules) = self.rules_by_family.get(&(
                    primary_pos,
                    source_feature_mask,
                    target_feature_mask,
                    family_suffix.clone(),
                )) else {
                    continue;
                };
                for rule in rules {
                    let Some(surface) = apply_edge_transform_prepared(
                        prepared.source_surface,
                        prepared.source_chars,
                        usize::from(rule.remove_prefix_chars),
                        &rule.prepend,
                        usize::from(rule.remove_chars),
                        &rule.append,
                    ) else {
                        continue;
                    };
                    if surface == prepared.source_surface || surface.is_empty() {
                        continue;
                    }
                    let geometry_evidence_milli =
                        if let Some(geometry) = geometry_by_surface.get(&surface) {
                            *geometry
                        } else {
                            let geometry =
                                prepared_similarity_to_normalized_surface_with_workspace_milli(
                                    prepared.observed_profile,
                                    &surface,
                                    geometry_workspace,
                                );
                            geometry_by_surface.insert(surface.clone(), geometry);
                            geometry
                        };
                    let candidate = ProductiveFormBirth {
                        surface,
                        source_feature_mask,
                        target_feature_mask,
                        geometry_evidence_milli,
                        profile_evidence_milli: rule.profile_evidence_milli(),
                        positive_support: rule.positive_support,
                        anti_support: rule.anti_support,
                        family_specificity: rule.family_specificity(),
                        status: ProductiveBirthStatus::ShadowUnverified,
                    };
                    retain_best_productive_birth(&mut by_surface, candidate);
                }
                break;
            }
        }
        if by_surface.is_empty() {
            return Vec::new();
        }
        let mut births = by_surface;
        births.sort_by(|left, right| {
            productive_birth_rank(right)
                .cmp(&productive_birth_rank(left))
                .then_with(|| left.surface.cmp(&right.surface))
        });
        births.truncate(limit);
        births
    }

    pub(super) fn report(&self) -> &ProductiveIndexReport {
        &self.report
    }

    pub(super) fn train_context_slots_from_corpus(
        &mut self,
        corpus_path: &Path,
        excluded_lemmas: &BTreeSet<String>,
    ) -> io::Result<()> {
        let training = collect_context_training(
            BufReader::with_capacity(1024 * 1024, File::open(corpus_path)?),
            excluded_lemmas,
            &mut self.report,
        )?;
        let form_features = collect_wanted_form_features(
            BufReader::with_capacity(1024 * 1024, File::open(corpus_path)?),
            &training.wanted_lemma_surfaces,
        )?;
        self.finish_context_training(training, &form_features);
        Ok(())
    }

    fn train_context_slots(
        &mut self,
        mut reader: impl BufRead,
        excluded_lemmas: &BTreeSet<String>,
    ) -> io::Result<()> {
        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes)?;
        let training = collect_context_training(
            std::io::Cursor::new(&bytes),
            excluded_lemmas,
            &mut self.report,
        )?;
        let form_features = collect_wanted_form_features(
            std::io::Cursor::new(&bytes),
            &training.wanted_lemma_surfaces,
        )?;
        self.finish_context_training(training, &form_features);
        Ok(())
    }

    fn finish_context_training(
        &mut self,
        training: ContextTrainingAccumulator,
        form_features: &BTreeMap<String, BTreeMap<String, BTreeSet<u32>>>,
    ) {
        self.context_slots = training
            .positive
            .into_iter()
            .map(|(key @ (context_key, _), positive_support)| {
                let total = training
                    .context_totals
                    .get(&context_key)
                    .copied()
                    .unwrap_or_default();
                let unlabeled_alternative_support = total.saturating_sub(positive_support);
                (
                    key,
                    ProductiveContextSlotEvidence {
                        positive_support,
                        unlabeled_alternative_support,
                        posterior_milli: laplace_posterior_milli(
                            positive_support,
                            unlabeled_alternative_support,
                        ),
                        context_observed: true,
                    },
                )
            })
            .collect();
        self.known_contexts = training.context_totals;
        self.context_pair_support.clear();
        for pending in training.pending_competitions {
            let mut row_pairs = BTreeSet::new();
            for competitor in pending.competitors {
                let Some(features) = form_features
                    .get(&pending.lemma)
                    .and_then(|surfaces| surfaces.get(&competitor))
                else {
                    continue;
                };
                self.report.same_lemma_competitor_surfaces += 1;
                for lane in context_evidence_lanes(&pending.context) {
                    let context_key = scoped_context_evidence_key(lane.key, pending.primary_pos);
                    let preferred_slot = productive_slot_features_for_scope(
                        pending.preferred_feature_mask,
                        lane.scope,
                    );
                    for competitor_feature_mask in features {
                        if crate::nanda_wave::morphology_phase::feature_primary_pos(
                            *competitor_feature_mask,
                        ) != pending.primary_pos
                        {
                            continue;
                        }
                        let competitor_slot = productive_slot_features_for_scope(
                            *competitor_feature_mask,
                            lane.scope,
                        );
                        if preferred_slot != competitor_slot {
                            row_pairs.insert((context_key, preferred_slot, competitor_slot));
                        }
                    }
                }
            }
            self.report.admitted_pair_observations += row_pairs.len();
            for pair in row_pairs {
                *self.context_pair_support.entry(pair).or_default() += 1;
            }
        }
        self.report.context_modes = self.known_contexts.len();
        self.report.context_slots = self.context_slots.len();
        self.report.context_pairs = self.context_pair_support.len();
    }

    pub(super) fn context_slot_evidence(
        &self,
        context: &str,
        target_feature_mask: u32,
    ) -> ProductiveContextSlotEvidence {
        let primary_pos =
            crate::nanda_wave::morphology_phase::feature_primary_pos(target_feature_mask);
        for lane in context_evidence_lanes(context) {
            let slot = productive_slot_features_for_scope(target_feature_mask, lane.scope);
            for context_key in [scoped_context_evidence_key(lane.key, primary_pos), lane.key] {
                if let Some(context_total) = self.known_contexts.get(&context_key).copied() {
                    return self
                        .context_slots
                        .get(&(context_key, slot))
                        .copied()
                        .unwrap_or_else(|| ProductiveContextSlotEvidence {
                            positive_support: 0,
                            unlabeled_alternative_support: context_total,
                            posterior_milli: laplace_posterior_milli(0, context_total),
                            context_observed: true,
                        });
                }
            }
        }
        ProductiveContextSlotEvidence {
            positive_support: 0,
            unlabeled_alternative_support: 0,
            posterior_milli: laplace_posterior_milli(0, 0),
            context_observed: false,
        }
    }

    pub(super) fn context_pair_evidence(
        &self,
        context: &str,
        preferred_feature_mask: u32,
        competitor_feature_mask: u32,
    ) -> ProductiveContextPairEvidence {
        context_pair_evidence_from(
            context,
            preferred_feature_mask,
            competitor_feature_mask,
            |context_key| self.known_contexts.contains_key(&context_key),
            |context_key, preferred_slot, competitor_slot| {
                self.context_pair_support
                    .get(&(context_key, preferred_slot, competitor_slot))
                    .copied()
                    .unwrap_or_default()
            },
        )
    }

    pub(super) fn target_features(
        &self,
        primary_pos: u16,
        source_feature_mask: u32,
    ) -> impl Iterator<Item = u32> + '_ {
        self.target_features_by_source
            .get(&(primary_pos, source_feature_mask))
            .into_iter()
            .flatten()
            .copied()
    }
}

pub(super) fn context_pair_evidence_from(
    context: &str,
    preferred_feature_mask: u32,
    competitor_feature_mask: u32,
    mut context_observed: impl FnMut(u32) -> bool,
    mut pair_support: impl FnMut(u32, u32, u32) -> u32,
) -> ProductiveContextPairEvidence {
    let primary_pos =
        crate::nanda_wave::morphology_phase::feature_primary_pos(preferred_feature_mask);
    if primary_pos
        != crate::nanda_wave::morphology_phase::feature_primary_pos(competitor_feature_mask)
    {
        return ProductiveContextPairEvidence::default();
    }
    let mut evidence = ProductiveContextPairEvidence::default();
    let mut neighbor_positive_support = 0_u32;
    let mut neighbor_anti_support = 0_u32;
    for lane in context_evidence_lanes(context) {
        let context_key = scoped_context_evidence_key(lane.key, primary_pos);
        if !context_observed(context_key) {
            continue;
        }
        evidence.context_observed = true;
        let preferred_slot = productive_slot_features_for_scope(preferred_feature_mask, lane.scope);
        let competitor_slot =
            productive_slot_features_for_scope(competitor_feature_mask, lane.scope);
        if preferred_slot == competitor_slot {
            continue;
        }
        let positive_support = pair_support(context_key, preferred_slot, competitor_slot);
        let anti_support = pair_support(context_key, competitor_slot, preferred_slot);
        match lane.scope {
            ContextEvidenceScope::Exact => {
                evidence.exact_positive_support = positive_support;
                evidence.exact_anti_support = anti_support;
            }
            ContextEvidenceScope::Neighbor => {
                neighbor_positive_support =
                    neighbor_positive_support.saturating_add(positive_support);
                neighbor_anti_support = neighbor_anti_support.saturating_add(anti_support);
                match positive_support.cmp(&anti_support) {
                    std::cmp::Ordering::Greater => {
                        evidence.supporting_neighbor_lanes =
                            evidence.supporting_neighbor_lanes.saturating_add(1);
                    }
                    std::cmp::Ordering::Less => {
                        evidence.contradicting_neighbor_lanes =
                            evidence.contradicting_neighbor_lanes.saturating_add(1);
                    }
                    std::cmp::Ordering::Equal if positive_support > 0 => {
                        evidence.tied_neighbor_lanes =
                            evidence.tied_neighbor_lanes.saturating_add(1);
                    }
                    std::cmp::Ordering::Equal => {}
                }
            }
        }
    }
    if evidence.exact_positive_support > 0 || evidence.exact_anti_support > 0 {
        evidence.positive_support = evidence.exact_positive_support;
        evidence.anti_support = evidence.exact_anti_support;
    } else {
        evidence.positive_support = neighbor_positive_support;
        evidence.anti_support = neighbor_anti_support;
    }
    evidence.posterior_milli =
        laplace_posterior_milli(evidence.positive_support, evidence.anti_support);
    evidence
}

pub(super) fn directional_evidence_margin(
    evidence: ProductiveContextPairEvidence,
) -> std::cmp::Ordering {
    if evidence.exact_positive_support > 0 || evidence.exact_anti_support > 0 {
        return evidence
            .exact_positive_support
            .cmp(&evidence.exact_anti_support);
    }
    // Neighbor lanes are useful retention evidence, but the independent NH
    // gate showed that even two agreeing lexical neighbors can reverse a real
    // target. Only an independently observed exact competitor scene owns
    // morphology-slot authority; lower-specificity evidence stays tied for L3.
    std::cmp::Ordering::Equal
}

fn collect_context_training(
    mut reader: impl BufRead,
    excluded_lemmas: &BTreeSet<String>,
    report: &mut ProductiveIndexReport,
) -> io::Result<ContextTrainingAccumulator> {
    let mut training = ContextTrainingAccumulator::default();
    let mut line = String::with_capacity(256);
    let mut line_number = 0_usize;
    loop {
        line.clear();
        if reader.read_line(&mut line)? == 0 {
            break;
        }
        line_number += 1;
        if !line.starts_with("T\t") && !line.starts_with("NT\t") {
            continue;
        }
        report.observed_context_rows += 1;
        let raw = line.trim_end_matches(['\r', '\n']);
        let fields = raw.split('\t').collect::<Vec<_>>();
        let (lemma, feature_text, context, competitor_text) = match fields.as_slice() {
            ["T", lemma, _surface, feature_text, context] => {
                (*lemma, *feature_text, *context, None)
            }
            ["NT", lemma, _surface, feature_text, context, competitors] => {
                report.observed_competitor_rows += 1;
                (*lemma, *feature_text, *context, Some(*competitors))
            }
            _ => {
                // Legacy rows without a lemma identity cannot participate in a
                // leave-lemmas-out proof or directional slot competition.
                report.rejected_context_rows += 1;
                continue;
            }
        };
        let lemma = lemma.trim().to_lowercase();
        if excluded_lemmas.contains(&lemma) {
            report.excluded_context_rows += 1;
            continue;
        }
        if context
            .split_whitespace()
            .filter(|token| *token == "_")
            .count()
            != 1
        {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("line {line_number}: productive context requires one _ slot"),
            ));
        }
        let feature_mask = crate::nanda_wave::morphology_phase::parse_features(feature_text)
            .map_err(|error| {
                io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!("line {line_number}: {error}"),
                )
            })?;
        let primary_pos = crate::nanda_wave::morphology_phase::feature_primary_pos(feature_mask);
        for lane in context_evidence_lanes(context) {
            let slot = productive_slot_features_for_scope(feature_mask, lane.scope);
            let context_key = scoped_context_evidence_key(lane.key, primary_pos);
            *training.context_totals.entry(context_key).or_default() += 1;
            *training.positive.entry((context_key, slot)).or_default() += 1;
        }
        if let Some(competitor_text) = competitor_text {
            let competitors = competitor_text
                .split(',')
                .map(str::trim)
                .filter(|surface| !surface.is_empty())
                .map(str::to_lowercase)
                .collect::<BTreeSet<_>>()
                .into_iter()
                .collect::<Vec<_>>();
            report.observed_competitor_surfaces += competitors.len();
            if !competitors.is_empty() {
                training
                    .wanted_lemma_surfaces
                    .entry(lemma.clone())
                    .or_default()
                    .extend(competitors.iter().cloned());
                training
                    .pending_competitions
                    .push(PendingContextCompetition {
                        lemma,
                        context: context.to_string(),
                        primary_pos,
                        preferred_feature_mask: feature_mask,
                        competitors,
                    });
            }
        }
        report.admitted_context_rows += 1;
    }
    Ok(training)
}

pub(super) fn collect_wanted_form_features(
    mut reader: impl BufRead,
    wanted: &BTreeMap<String, BTreeSet<String>>,
) -> io::Result<BTreeMap<String, BTreeMap<String, BTreeSet<u32>>>> {
    let mut features = BTreeMap::<String, BTreeMap<String, BTreeSet<u32>>>::new();
    let mut line = String::with_capacity(128);
    let mut line_number = 0_usize;
    loop {
        line.clear();
        if reader.read_line(&mut line)? == 0 {
            break;
        }
        line_number += 1;
        if !line.starts_with("F\t") {
            continue;
        }
        let raw = line.trim_end_matches(['\r', '\n']);
        let fields = raw.split('\t').collect::<Vec<_>>();
        let ["F", lemma, surface, feature_text] = fields.as_slice() else {
            continue;
        };
        let lemma = lemma.trim().to_lowercase();
        let Some(wanted_surfaces) = wanted.get(&lemma) else {
            continue;
        };
        let surface = surface.trim().to_lowercase();
        if !wanted_surfaces.contains(&surface) {
            continue;
        }
        let feature_mask = crate::nanda_wave::morphology_phase::parse_features(feature_text)
            .map_err(|error| {
                io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!("line {line_number}: {error}"),
                )
            })?;
        features
            .entry(lemma)
            .or_default()
            .entry(surface)
            .or_default()
            .insert(feature_mask);
    }
    Ok(features)
}

pub(super) fn productive_slot_features_for_scope(
    features: u32,
    scope: ContextEvidenceScope,
) -> u32 {
    match scope {
        ContextEvidenceScope::Exact => {
            crate::nanda_wave::morphology_phase::productive_context_slot_features(features)
        }
        ContextEvidenceScope::Neighbor => {
            crate::nanda_wave::morphology_phase::productive_neighbor_context_slot_features(features)
        }
    }
}

impl ProductiveMorphologySource for ProductiveMorphologyIndex {
    fn target_features_vec(&self, primary_pos: u16, source_feature_mask: u32) -> Vec<u32> {
        self.target_features(primary_pos, source_feature_mask)
            .collect()
    }

    fn context_slot_evidence_for(
        &self,
        context: &str,
        target_feature_mask: u32,
    ) -> ProductiveContextSlotEvidence {
        self.context_slot_evidence(context, target_feature_mask)
    }

    fn context_pair_evidence_for(
        &self,
        context: &str,
        preferred_feature_mask: u32,
        competitor_feature_mask: u32,
    ) -> ProductiveContextPairEvidence {
        self.context_pair_evidence(context, preferred_feature_mask, competitor_feature_mask)
    }

    fn generate_forms(
        &self,
        observed_surface: &str,
        primary_pos: u16,
        source_surface: &str,
        source_feature_mask: u32,
        target_feature_mask: u32,
        limit: usize,
    ) -> Vec<ProductiveFormBirth> {
        self.generate(
            observed_surface,
            primary_pos,
            source_surface,
            source_feature_mask,
            target_feature_mask,
            limit,
        )
    }

    fn generate_forms_prepared(
        &self,
        prepared: &PreparedProductiveGeneration<'_>,
        primary_pos: u16,
        source_feature_mask: u32,
        target_feature_mask: u32,
        limit: usize,
        geometry_by_surface: &mut HashMap<String, u16>,
        geometry_workspace: &mut SurfaceGeometryWorkspace,
    ) -> Vec<ProductiveFormBirth> {
        self.generate_prepared(
            prepared,
            primary_pos,
            source_feature_mask,
            target_feature_mask,
            limit,
            geometry_by_surface,
            geometry_workspace,
        )
    }
}

fn observe_lemma(
    lemma: &ProductiveLemma,
    observations: &mut BTreeMap<ProductiveRuleKey, u32>,
    family_totals: &mut BTreeMap<(u16, u32, u32, String), u32>,
    report: &mut ProductiveIndexReport,
) {
    let Some(source) = canonical_source(&lemma.forms) else {
        return;
    };
    report.admitted_lemmas += 1;
    let mut lemma_observations = BTreeSet::new();
    let mut lemma_family_totals = BTreeSet::new();
    for target in &lemma.forms {
        if target.feature_mask == source.feature_mask && target.surface == source.surface {
            continue;
        }
        let Some((remove_prefix_chars, prepend, remove_chars, append)) =
            edge_transform(&source.surface, &target.surface)
        else {
            continue;
        };
        let source_chars = source.surface.chars().collect::<Vec<_>>();
        let max_specificity = source_chars.len().min(MAX_FAMILY_SUFFIX_CHARS);
        for specificity in 0..=max_specificity {
            let family_suffix = source_chars[source_chars.len() - specificity..]
                .iter()
                .collect::<String>();
            lemma_family_totals.insert((
                lemma.primary_pos,
                source.feature_mask,
                target.feature_mask,
                family_suffix.clone(),
            ));
            lemma_observations.insert(ProductiveRuleKey {
                primary_pos: lemma.primary_pos,
                source_feature_mask: source.feature_mask,
                target_feature_mask: target.feature_mask,
                family_suffix,
                remove_prefix_chars,
                prepend: prepend.clone(),
                remove_chars,
                append: append.clone(),
            });
            report.observed_transforms += 1;
        }
    }
    for key in lemma_family_totals {
        *family_totals.entry(key).or_default() += 1;
    }
    for key in lemma_observations {
        *observations.entry(key).or_default() += 1;
    }
}

fn finish_training(
    observations: BTreeMap<ProductiveRuleKey, u32>,
    family_totals: BTreeMap<(u16, u32, u32, String), u32>,
    mut report: ProductiveIndexReport,
    minimum_support: u32,
) -> ProductiveMorphologyIndex {
    let mut rules_by_family = BTreeMap::<(u16, u32, u32, String), Vec<ProductiveRule>>::new();
    let mut target_features_by_source = BTreeMap::<(u16, u32), BTreeSet<u32>>::new();
    for (key, positive_support) in observations {
        if positive_support < minimum_support.max(1) {
            report.rejected_low_support_profiles += 1;
            continue;
        }
        let family_total = family_totals
            .get(&(
                key.primary_pos,
                key.source_feature_mask,
                key.target_feature_mask,
                key.family_suffix.clone(),
            ))
            .copied()
            .unwrap_or_default();
        let rule = ProductiveRule {
            primary_pos: key.primary_pos,
            source_feature_mask: key.source_feature_mask,
            target_feature_mask: key.target_feature_mask,
            family_suffix: key.family_suffix,
            remove_prefix_chars: key.remove_prefix_chars,
            prepend: key.prepend,
            remove_chars: key.remove_chars,
            append: key.append,
            positive_support,
            anti_support: family_total.saturating_sub(positive_support),
        };
        target_features_by_source
            .entry((rule.primary_pos, rule.source_feature_mask))
            .or_default()
            .insert(rule.target_feature_mask);
        rules_by_family
            .entry((
                rule.primary_pos,
                rule.source_feature_mask,
                rule.target_feature_mask,
                rule.family_suffix.clone(),
            ))
            .or_default()
            .push(rule);
        report.admitted_profiles += 1;
    }
    for rules in rules_by_family.values_mut() {
        rules.sort_by(|left, right| productive_rule_order(left, right));
    }
    let target_features_by_source = target_features_by_source
        .into_iter()
        .map(|(key, targets)| (key, targets.into_iter().collect()))
        .collect();
    ProductiveMorphologyIndex {
        rules_by_family,
        target_features_by_source,
        context_slots: BTreeMap::new(),
        known_contexts: BTreeMap::new(),
        context_pair_support: BTreeMap::new(),
        report,
    }
}

fn laplace_posterior_milli(positive_support: u32, anti_support: u32) -> u16 {
    let numerator = u64::from(positive_support).saturating_add(1) * 1_000;
    let denominator = u64::from(positive_support)
        .saturating_add(u64::from(anti_support))
        .saturating_add(2);
    (numerator / denominator).min(1_000) as u16
}

pub(super) fn package_canonical_source(
    package: &RuntimeL2Package,
    lemma_id: u32,
) -> Result<(u16, ProductiveForm), String> {
    let center = package
        .lemma_centers()
        .get(lemma_id as usize)
        .copied()
        .ok_or_else(|| format!("missing productive lemma center {lemma_id}"))?;
    let start = center.form_start as usize;
    let end = start.saturating_add(center.form_count as usize);
    let minimum_priority = (start..end)
        .filter_map(|binding_index| {
            package
                .binding_for_lemma(binding_index, lemma_id as usize)
                .map(|binding| {
                    crate::nanda_wave::morphology_phase::productive_source_priority(
                        binding.feature_mask,
                    )
                })
        })
        .min()
        .ok_or_else(|| format!("productive lemma {lemma_id} has no morphology bindings"))?;
    let mut best = None::<ProductiveForm>;
    for binding_index in start..end {
        let binding = package
            .binding_for_lemma(binding_index, lemma_id as usize)
            .ok_or_else(|| format!("missing productive morphology binding {binding_index}"))?;
        if crate::nanda_wave::morphology_phase::productive_source_priority(binding.feature_mask)
            != minimum_priority
        {
            continue;
        }
        let surface = package
            .surface(binding.form_center_ref as usize)
            .ok_or_else(|| {
                format!(
                    "missing productive morphology surface {}",
                    binding.form_center_ref
                )
            })?
            .into_owned();
        let candidate = ProductiveForm {
            form_ref: binding.form_center_ref,
            surface,
            feature_mask: binding.feature_mask,
        };
        if best.as_ref().is_none_or(|current| {
            (
                candidate.surface.chars().count(),
                candidate.feature_mask,
                &candidate.surface,
            ) < (
                current.surface.chars().count(),
                current.feature_mask,
                &current.surface,
            )
        }) {
            best = Some(candidate);
        }
    }
    best.map(|source| (center.primary_pos, source))
        .ok_or_else(|| format!("productive lemma {lemma_id} has no canonical source"))
}

pub(super) fn package_lemma(
    package: &RuntimeL2Package,
    lemma_id: u32,
) -> Result<ProductiveLemma, String> {
    let center = package
        .lemma_centers()
        .get(lemma_id as usize)
        .copied()
        .ok_or_else(|| format!("missing productive lemma center {lemma_id}"))?;
    let start = center.form_start as usize;
    let end = start.saturating_add(center.form_count as usize);
    let mut forms = Vec::with_capacity(end.saturating_sub(start));
    for binding_index in start..end {
        let binding = package
            .binding_for_lemma(binding_index, lemma_id as usize)
            .ok_or_else(|| format!("missing productive morphology binding {binding_index}"))?;
        let surface = package
            .surface(binding.form_center_ref as usize)
            .ok_or_else(|| {
                format!(
                    "missing productive morphology surface {}",
                    binding.form_center_ref
                )
            })?
            .into_owned();
        forms.push(ProductiveForm {
            form_ref: binding.form_center_ref,
            surface,
            feature_mask: binding.feature_mask,
        });
    }
    forms.sort_by(|left, right| {
        (left.feature_mask, &left.surface, left.form_ref).cmp(&(
            right.feature_mask,
            &right.surface,
            right.form_ref,
        ))
    });
    forms.dedup();
    Ok(ProductiveLemma {
        lemma_id,
        primary_pos: center.primary_pos,
        forms,
    })
}

pub(super) fn canonical_source(forms: &[ProductiveForm]) -> Option<&ProductiveForm> {
    forms.iter().min_by(|left, right| {
        (
            crate::nanda_wave::morphology_phase::productive_source_priority(left.feature_mask),
            left.surface.chars().count(),
            left.feature_mask,
            &left.surface,
        )
            .cmp(&(
                crate::nanda_wave::morphology_phase::productive_source_priority(right.feature_mask),
                right.surface.chars().count(),
                right.feature_mask,
                &right.surface,
            ))
    })
}

fn edge_transform(source: &str, target: &str) -> Option<(u8, String, u8, String)> {
    let source_chars = source.chars().collect::<Vec<_>>();
    let target_chars = target.chars().collect::<Vec<_>>();
    let direct_shared = source_chars
        .iter()
        .zip(&target_chars)
        .take_while(|(left, right)| left == right)
        .count();
    let direct_remove = source_chars.len().saturating_sub(direct_shared);
    let direct_append = target_chars.len().saturating_sub(direct_shared);
    if direct_shared >= MIN_SHARED_STEM_CHARS
        && direct_remove <= MAX_REMOVE_CHARS
        && direct_append <= MAX_APPEND_CHARS
    {
        return Some((
            0,
            String::new(),
            direct_remove as u8,
            target_chars[direct_shared..].iter().collect::<String>(),
        ));
    }
    let mut best = None::<(usize, usize, usize, usize, usize)>;
    for source_start in 0..=source_chars.len().min(MAX_PREFIX_CHARS) {
        for target_start in 0..=target_chars.len().min(MAX_PREFIX_CHARS) {
            let shared = source_chars[source_start..]
                .iter()
                .zip(&target_chars[target_start..])
                .take_while(|(left, right)| left == right)
                .count();
            let remove_chars = source_chars.len().saturating_sub(source_start + shared);
            let append_chars = target_chars.len().saturating_sub(target_start + shared);
            if shared < MIN_SHARED_STEM_CHARS
                || remove_chars > MAX_REMOVE_CHARS
                || append_chars > MAX_APPEND_CHARS
            {
                continue;
            }
            let edit_width = source_start
                .saturating_add(target_start)
                .saturating_add(remove_chars)
                .saturating_add(append_chars);
            let candidate = (shared, edit_width, source_start, target_start, remove_chars);
            if best.is_none_or(|current| {
                candidate.0 > current.0
                    || (candidate.0 == current.0 && candidate.1 < current.1)
                    || (candidate.0 == current.0
                        && candidate.1 == current.1
                        && candidate.2 < current.2)
                    || (candidate.0 == current.0
                        && candidate.1 == current.1
                        && candidate.2 == current.2
                        && candidate.3 < current.3)
            }) {
                best = Some(candidate);
            }
        }
    }
    let (shared, _, source_start, target_start, remove_chars) = best?;
    Some((
        source_start as u8,
        target_chars[..target_start].iter().collect::<String>(),
        remove_chars as u8,
        target_chars[target_start + shared..]
            .iter()
            .collect::<String>(),
    ))
}

pub(super) fn apply_edge_transform(
    source: &str,
    remove_prefix_chars: usize,
    prepend: &str,
    remove_chars: usize,
    append: &str,
) -> Option<String> {
    let chars = source.chars().collect::<Vec<_>>();
    apply_edge_transform_prepared(
        source,
        &chars,
        remove_prefix_chars,
        prepend,
        remove_chars,
        append,
    )
}

pub(super) fn apply_edge_transform_prepared(
    source: &str,
    chars: &[char],
    remove_prefix_chars: usize,
    prepend: &str,
    remove_chars: usize,
    append: &str,
) -> Option<String> {
    if remove_prefix_chars.saturating_add(remove_chars) > chars.len() {
        return None;
    }
    let mut surface = String::with_capacity(prepend.len() + source.len() + append.len());
    surface.push_str(prepend);
    surface.extend(
        chars[remove_prefix_chars..chars.len() - remove_chars]
            .iter()
            .copied(),
    );
    surface.push_str(append);
    Some(surface)
}

fn productive_rule_order(left: &ProductiveRule, right: &ProductiveRule) -> std::cmp::Ordering {
    right
        .family_specificity()
        .cmp(&left.family_specificity())
        .then_with(|| {
            right
                .profile_evidence_milli()
                .cmp(&left.profile_evidence_milli())
        })
        .then_with(|| right.positive_support.cmp(&left.positive_support))
        .then_with(|| {
            left.remove_prefix_chars
                .saturating_add(left.remove_chars)
                .cmp(&right.remove_prefix_chars.saturating_add(right.remove_chars))
        })
        .then_with(|| left.prepend.cmp(&right.prepend))
        .then_with(|| left.remove_chars.cmp(&right.remove_chars))
        .then_with(|| left.append.cmp(&right.append))
}

pub(super) fn productive_birth_rank(
    birth: &ProductiveFormBirth,
) -> (u8, u16, u32, std::cmp::Reverse<u32>, u16) {
    (
        birth.family_specificity,
        birth.profile_evidence_milli,
        birth.positive_support,
        std::cmp::Reverse(birth.anti_support),
        birth.geometry_evidence_milli,
    )
}

pub(super) fn retain_best_productive_birth(
    births: &mut Vec<ProductiveFormBirth>,
    candidate: ProductiveFormBirth,
) {
    if let Some(current) = births
        .iter_mut()
        .find(|current| current.surface == candidate.surface)
    {
        if productive_birth_rank(&candidate) > productive_birth_rank(current) {
            *current = candidate;
        }
    } else {
        births.push(candidate);
    }
}

pub(super) fn productive_family_lane_starts(
    source_surface: &str,
    observed_surface: &str,
) -> Vec<usize> {
    let maximum = source_surface.chars().count().min(MAX_FAMILY_SUFFIX_CHARS);
    let directed = productive_family_specificities(source_surface, observed_surface)
        .first()
        .copied()
        .unwrap_or(maximum);
    let mut starts = vec![maximum];
    if directed != maximum {
        starts.push(directed);
    }
    starts
}

pub(super) fn productive_family_suffixes(source_chars: &[char]) -> Vec<String> {
    let maximum = source_chars.len().min(MAX_FAMILY_SUFFIX_CHARS);
    (0..=maximum)
        .map(|specificity| {
            source_chars[source_chars.len() - specificity..]
                .iter()
                .collect()
        })
        .collect()
}

pub(super) fn productive_family_specificities(
    source_surface: &str,
    observed_surface: &str,
) -> Vec<usize> {
    let source = source_surface.chars().collect::<Vec<_>>();
    let observed = observed_surface.chars().collect::<Vec<_>>();
    let maximum = source.len().min(MAX_FAMILY_SUFFIX_CHARS);
    if maximum == 0 {
        return vec![0];
    }
    let shared_prefix = source
        .iter()
        .zip(&observed)
        .take_while(|(left, right)| left == right)
        .count();
    let changed_source_suffix = source.len().saturating_sub(shared_prefix).clamp(1, maximum);
    (0..=changed_source_suffix).rev().collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    const NOUN_NOM_SG: u32 = (1 << 0) | (1 << 4) | (1 << 8);
    const NOUN_GEN_SG: u32 = (1 << 0) | (1 << 4) | (1 << 9);
    const NOUN_GEN_PL: u32 = (1 << 0) | (1 << 5) | (1 << 9);
    const NOUN_DAT_SG: u32 = (1 << 0) | (1 << 4) | (1 << 10);
    const NOUN_ACC_SG: u32 = (1 << 0) | (1 << 4) | (1 << 11);
    const NOUN_PREP_SG: u32 = (1 << 0) | (1 << 4) | (1 << 13);

    fn lemma(id: u32, nominative: &str, genitive: &str) -> ProductiveLemma {
        ProductiveLemma {
            lemma_id: id,
            primary_pos: 1,
            forms: vec![
                ProductiveForm {
                    form_ref: 0,
                    surface: nominative.to_string(),
                    feature_mask: NOUN_NOM_SG,
                },
                ProductiveForm {
                    form_ref: 1,
                    surface: genitive.to_string(),
                    feature_mask: NOUN_GEN_SG,
                },
            ],
        }
    }

    #[test]
    fn leave_one_lemma_out_generates_an_unseen_form_without_authority() {
        let lemmas = vec![
            lemma(0, "рамка", "рамки"),
            lemma(1, "лапка", "лапки"),
            lemma(2, "шапка", "шапки"),
        ];
        let index = ProductiveMorphologyIndex::train_lemmas(&lemmas, |lemma_id| lemma_id != 2, 2);

        let births = index.generate("шпки", 1, "шапка", NOUN_NOM_SG, NOUN_GEN_SG, 8);

        assert_eq!(
            births.first().map(|birth| birth.surface.as_str()),
            Some("шапки")
        );
        assert!(births
            .iter()
            .all(|birth| { birth.status == ProductiveBirthStatus::ShadowUnverified }));
        assert!(index.report().admitted_profiles > 0);
    }

    #[test]
    fn damaged_ending_uses_the_matching_family_scale_and_retains_a_general_transform() {
        const ADJ_NOM: u32 = (1 << 2) | (1 << 4) | (1 << 8);
        const ADJ_GEN: u32 = (1 << 2) | (1 << 4) | (1 << 9);
        let adjective = |id, source: &str, target: &str| ProductiveLemma {
            lemma_id: id,
            primary_pos: 3,
            forms: vec![
                ProductiveForm {
                    form_ref: id * 2,
                    surface: source.to_string(),
                    feature_mask: ADJ_NOM,
                },
                ProductiveForm {
                    form_ref: id * 2 + 1,
                    surface: target.to_string(),
                    feature_mask: ADJ_GEN,
                },
            ],
        };
        let lemmas = vec![
            adjective(0, "красный", "краснейшей"),
            adjective(1, "опасный", "опаснейшей"),
            adjective(2, "пионовидный", "пионовидной"),
        ];
        let index = ProductiveMorphologyIndex::train_lemmas(&lemmas, |_| true, 2);

        let births = index.generate("пионовиднейешй", 3, "пионовидный", ADJ_NOM, ADJ_GEN, 8);

        assert!(births.iter().any(|birth| birth.surface == "пионовиднейшей"));
        assert_eq!(
            productive_family_specificities("пионовидный", "пионовиднейешй"),
            vec![2, 1, 0]
        );
    }

    #[test]
    fn irregular_stem_rewrites_are_not_misrepresented_as_suffix_rules() {
        assert_eq!(edge_transform("идти", "шел"), None);
        assert_eq!(
            edge_transform("дом", "дома"),
            Some((0, String::new(), 0, "а".to_string()))
        );
    }

    #[test]
    fn productive_edge_transform_can_learn_a_bounded_prefix_and_suffix() {
        let transform = edge_transform("ходячий", "походячее").expect("edge transform");
        assert_eq!(transform, (0, "по".to_string(), 2, "ее".to_string()));
        assert_eq!(
            apply_edge_transform(
                "ходячий",
                usize::from(transform.0),
                &transform.1,
                usize::from(transform.2),
                &transform.3,
            ),
            Some("походячее".to_string())
        );
    }

    #[test]
    fn context_slot_lattice_uses_train_rows_and_excludes_heldout_lemmas() {
        let lemmas = vec![
            lemma(0, "рамка", "рамки"),
            lemma(1, "лапка", "лапки"),
            lemma(2, "шапка", "шапки"),
        ];
        let mut index = ProductiveMorphologyIndex::train_lemmas(&lemmas, |_| true, 1);
        let corpus = concat!(
            "T\tрамка\tрамки\tnoun:gen:sg\tнет _\n",
            "T\tлапка\tлапки\tnoun:gen:sg\tнет _\n",
            "T\tдом\tдом\tnoun:nom:sg\tнет _\n",
            "T\tшапка\tшапка\tnoun:nom:sg\tнет _\n",
        );
        let excluded = BTreeSet::from(["шапка".to_string()]);
        index
            .train_context_slots(std::io::Cursor::new(corpus), &excluded)
            .expect("context slot training");

        let genitive = index.context_slot_evidence("нет _", NOUN_GEN_SG);
        let nominative = index.context_slot_evidence("нет _", NOUN_NOM_SG);
        assert_eq!(genitive.positive_support, 2);
        assert_eq!(nominative.positive_support, 1);
        assert!(genitive.context_observed);
        assert!(nominative.context_observed);
        assert_eq!(index.report().excluded_context_rows, 1);
    }

    #[test]
    fn context_slot_lattice_learns_a_neighbor_backoff_without_literal_rules() {
        let lemmas = vec![lemma(0, "рамка", "рамки"), lemma(1, "лапка", "лапки")];
        let mut index = ProductiveMorphologyIndex::train_lemmas(&lemmas, |_| true, 1);
        let corpus = concat!(
            "T\tрамка\tramке\tnoun:dat:sg\tподошел к _ окну\n",
            "T\tлапка\tлапке\tnoun:dat:sg\tдвигаюсь к _ дому\n",
        );
        index
            .train_context_slots(std::io::Cursor::new(corpus), &BTreeSet::new())
            .expect("context slot training");

        let dative = index.context_slot_evidence("к _", NOUN_DAT_SG);
        let genitive = index.context_slot_evidence("к _", NOUN_GEN_SG);
        assert_eq!(dative.positive_support, 2);
        assert_eq!(dative.unlabeled_alternative_support, 0);
        assert_eq!(genitive.positive_support, 0);
        assert_eq!(genitive.unlabeled_alternative_support, 2);
    }

    #[test]
    fn noun_context_evidence_does_not_create_number_authority() {
        let lemmas = vec![lemma(0, "рамка", "рамки")];
        let mut index = ProductiveMorphologyIndex::train_lemmas(&lemmas, |_| true, 1);
        index
            .train_context_slots(
                std::io::Cursor::new("T\tрамка\tрамки\tnoun:gen:sg\tнет _\n"),
                &BTreeSet::new(),
            )
            .expect("context slot training");

        assert_eq!(
            index.context_slot_evidence("нет _", NOUN_GEN_SG),
            index.context_slot_evidence("нет _", NOUN_GEN_PL)
        );
    }

    #[test]
    fn neighbor_context_evidence_does_not_create_adjective_number_authority() {
        let lemmas = vec![lemma(0, "рамка", "рамки")];
        let mut index = ProductiveMorphologyIndex::train_lemmas(&lemmas, |_| true, 1);
        index
            .train_context_slots(
                std::io::Cursor::new(
                    "T\tусловный\tусловным\tadj:ins:sg:masc\tговорю с _ человеком\n",
                ),
                &BTreeSet::new(),
            )
            .expect("context slot training");
        let singular = crate::nanda_wave::morphology_phase::parse_features("adj:ins:sg:masc")
            .expect("singular adjective");
        let plural = crate::nanda_wave::morphology_phase::parse_features("adj:ins:pl")
            .expect("plural adjective");

        assert_eq!(
            index.context_slot_evidence("с _", singular),
            index.context_slot_evidence("с _", plural)
        );
    }

    #[test]
    fn real_neighbor_train_rows_contribute_without_admitting_heldout_rows() {
        let lemmas = vec![lemma(0, "рамка", "рамки")];
        let mut index = ProductiveMorphologyIndex::train_lemmas(&lemmas, |_| true, 1);
        let corpus = concat!(
            "NT\tрамка\tрамке\tnoun:dat:sg\tподошел к _ дому\tлапке\n",
            "NH\tрамка\tрамки\tnoun:gen:sg\tнет _ дома\tлапки\n",
        );
        index
            .train_context_slots(std::io::Cursor::new(corpus), &BTreeSet::new())
            .expect("context slot training");

        assert_eq!(
            index
                .context_slot_evidence("к _", NOUN_DAT_SG)
                .positive_support,
            1
        );
        assert!(
            !index
                .context_slot_evidence("нет _", NOUN_GEN_SG)
                .context_observed
        );
        assert_eq!(index.report().observed_context_rows, 1);
        assert_eq!(index.report().admitted_context_rows, 1);
    }

    #[test]
    fn real_competitor_rows_train_directional_same_lemma_pair_evidence_only() {
        let lemmas = vec![lemma(0, "рамка", "рамки")];
        let mut index = ProductiveMorphologyIndex::train_lemmas(&lemmas, |_| true, 1);
        let corpus = concat!(
            "F\tрамка\tрамки\tnoun:gen:sg\n",
            "F\tрамка\tрамке\tnoun:dat:sg\n",
            "F\tлапка\tлапки\tnoun:gen:sg\n",
            "NT\tрамка\tрамке\tnoun:dat:sg\tподошел к _ дому\tрамки,лапки\n",
            "NH\tрамка\tрамки\tnoun:gen:sg\tподошел к _ дому\tрамке\n",
        );
        index
            .train_context_slots(std::io::Cursor::new(corpus), &BTreeSet::new())
            .expect("context pair training");

        let preferred = index.context_pair_evidence("подошел к _ дому", NOUN_DAT_SG, NOUN_GEN_SG);
        let reverse = index.context_pair_evidence("подошел к _ дому", NOUN_GEN_SG, NOUN_DAT_SG);
        assert_eq!((preferred.positive_support, preferred.anti_support), (1, 0));
        assert_eq!((reverse.positive_support, reverse.anti_support), (0, 1));
        assert_eq!(index.report().observed_competitor_rows, 1);
        assert_eq!(index.report().observed_competitor_surfaces, 2);
        assert_eq!(index.report().same_lemma_competitor_surfaces, 1);
        assert!(index.report().admitted_pair_observations > 0);
        assert!(index.report().context_pairs > 0);
    }

    #[test]
    fn directional_pair_evidence_reaches_an_independent_neighbor_context() {
        let lemmas = vec![lemma(0, "рамка", "рамки")];
        let mut index = ProductiveMorphologyIndex::train_lemmas(&lemmas, |_| true, 1);
        let corpus = concat!(
            "F\tрамка\tрамки\tnoun:gen:sg\n",
            "F\tрамка\tрамке\tnoun:dat:sg\n",
            "NT\tрамка\tрамке\tnoun:dat:sg\tподошел к _ дому\tрамки\n",
            "NH\tрамка\tрамке\tnoun:dat:sg\tпошел к _ окну\tрамки\n",
        );
        index
            .train_context_slots(std::io::Cursor::new(corpus), &BTreeSet::new())
            .expect("context pair training");

        let evidence = index.context_pair_evidence("пошел к _ окну", NOUN_DAT_SG, NOUN_GEN_SG);

        assert_eq!((evidence.positive_support, evidence.anti_support), (1, 0));
        assert!(evidence.context_observed);
        assert_eq!(
            directional_evidence_margin(evidence),
            std::cmp::Ordering::Equal
        );
    }

    #[test]
    fn two_neighbor_lanes_remain_non_authoritative_without_exact_observation() {
        let lemmas = vec![lemma(0, "рамка", "рамки")];
        let mut index = ProductiveMorphologyIndex::train_lemmas(&lemmas, |_| true, 1);
        let corpus = concat!(
            "F\tрамка\tрамки\tnoun:gen:sg\n",
            "F\tрамка\tрамке\tnoun:dat:sg\n",
            "NT\tрамка\tрамке\tnoun:dat:sg\tподошел к _ окну\tрамки\n",
            "NH\tрамка\tрамке\tnoun:dat:sg\tпошел к _ окну\tрамки\n",
        );
        index
            .train_context_slots(std::io::Cursor::new(corpus), &BTreeSet::new())
            .expect("context pair training");

        let evidence = index.context_pair_evidence("пошел к _ окну", NOUN_DAT_SG, NOUN_GEN_SG);

        assert_eq!(evidence.supporting_neighbor_lanes, 2);
        assert_eq!(evidence.contradicting_neighbor_lanes, 0);
        assert_eq!(
            directional_evidence_margin(evidence),
            std::cmp::Ordering::Equal
        );
    }

    #[test]
    fn ambiguous_neighbor_backoff_keeps_competing_slots_visible() {
        let lemmas = vec![lemma(0, "рамка", "рамки")];
        let mut index = ProductiveMorphologyIndex::train_lemmas(&lemmas, |_| true, 1);
        let corpus = concat!(
            "T\tрамка\tрамку\tnoun:acc:sg\tположил на _ стол\n",
            "T\tрамка\tрамке\tnoun:prep:sg\tлежит на _ столе\n",
        );
        index
            .train_context_slots(std::io::Cursor::new(corpus), &BTreeSet::new())
            .expect("context slot training");

        let accusative = index.context_slot_evidence("на _", NOUN_ACC_SG);
        let prepositional = index.context_slot_evidence("на _", NOUN_PREP_SG);
        assert_eq!(
            (
                accusative.positive_support,
                accusative.unlabeled_alternative_support
            ),
            (1, 1)
        );
        assert_eq!(
            (
                prepositional.positive_support,
                prepositional.unlabeled_alternative_support
            ),
            (1, 1)
        );
    }
}
