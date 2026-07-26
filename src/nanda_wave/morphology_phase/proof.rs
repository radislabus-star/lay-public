use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;
use std::time::Instant;

use serde::Serialize;

use super::corpus::{parse_corpus, MorphCorpus, MorphExample};
use super::field::{MorphReadout, MorphSlotScore, MorphologyField};
use super::{feature_name, NUMBER_PLURAL, NUMBER_SINGULAR};

const SMALL_RUSSIAN_CORPUS: &str =
    include_str!("../../../data/morphology/russian_noun_cases_small.tsv");

#[derive(Clone, Debug, Default, Serialize)]
struct SplitMetrics {
    cases: usize,
    top1_correct: usize,
    top1_percent: f64,
    exact_surface_top1_correct: usize,
    exact_surface_top1_percent: f64,
    authority_correct: usize,
    authority_percent: f64,
    false_authority: usize,
    tied: usize,
    abstain: usize,
    per_case: BTreeMap<String, CaseMetrics>,
}

#[derive(Clone, Debug, Default, Serialize)]
struct CaseMetrics {
    cases: usize,
    top1_correct: usize,
    top1_percent: f64,
    authority_correct: usize,
    authority_percent: f64,
    false_authority: usize,
}

#[derive(Clone, Debug, Default, Serialize)]
struct MorphologyCoverage {
    dual_number_lemmas: usize,
    plural_only_lemmas: usize,
    singular_only_lemmas: usize,
    singular_form_centers: usize,
    plural_form_centers: usize,
    multi_surface_slots: usize,
    maximum_surfaces_per_slot: usize,
}

#[derive(Clone, Debug, Serialize)]
struct MorphologyProofReport {
    architecture: &'static str,
    status: &'static str,
    scope: &'static str,
    corpus_bytes: usize,
    compile_ms: u128,
    proof_ms: u128,
    lemma_centers: usize,
    form_centers: usize,
    form_lemma_bindings: usize,
    binding_bytes: usize,
    coverage: MorphologyCoverage,
    positive_subcenters: usize,
    anti_subcenters: usize,
    minimum_positive: i32,
    minimum_authority_margin: i32,
    train: SplitMetrics,
    heldout: SplitMetrics,
    heldout_without_anti_top1_percent: f64,
    candidate_permutation_parity: bool,
    candidate_permutation_cases: usize,
    runtime_connected: bool,
    verdict: &'static str,
}

pub fn run_embedded_russian_morphology_proof() -> Result<serde_json::Value, String> {
    run_russian_morphology_proof_text(SMALL_RUSSIAN_CORPUS)
}

pub fn run_russian_morphology_proof_path(path: &Path) -> Result<serde_json::Value, String> {
    let text = std::fs::read_to_string(path).map_err(|error| {
        format!(
            "failed to read morphology corpus {}: {error}",
            path.display()
        )
    })?;
    run_russian_morphology_proof_text(&text)
}

fn run_russian_morphology_proof_text(text: &str) -> Result<serde_json::Value, String> {
    let compile_started = Instant::now();
    let corpus = parse_corpus(text)?;
    let field = MorphologyField::train(&corpus)?;
    let compile_ms = compile_started.elapsed().as_millis();
    let proof_started = Instant::now();
    let train = evaluate(&field, &corpus, &corpus.train, true);
    let heldout = evaluate(&field, &corpus, &corpus.heldout, true);
    let heldout_without_anti = evaluate(&field, &corpus, &corpus.heldout, false);
    let permutation_examples = stratified_permutation_examples(&corpus.heldout);
    let candidate_permutation_cases = permutation_examples.len();
    let candidate_permutation_parity = permutation_parity(&field, &permutation_examples);
    let proof_ms = proof_started.elapsed().as_millis();
    let verdict = if heldout.top1_percent >= 95.0
        && heldout
            .per_case
            .values()
            .all(|metrics| metrics.top1_percent >= 95.0)
        && heldout.false_authority == 0
        && candidate_permutation_parity
    {
        "PASS_shadow"
    } else {
        "WATCH_shadow"
    };
    let calibration = field.calibration();
    let coverage = morphology_coverage(&corpus);
    serde_json::to_value(MorphologyProofReport {
        architecture: "L1 FormCenter -> L2 LemmaCenter -> positive/anti MorphSlot field",
        status: "cold-teacher supervised corpus proof",
        scope:
            "Russian noun declension: six core cases plus partitive, second locative and vocative",
        corpus_bytes: text.len(),
        compile_ms,
        proof_ms,
        lemma_centers: corpus.lemmas.len(),
        form_centers: corpus.surfaces.len(),
        form_lemma_bindings: field.binding_count(),
        binding_bytes: field.binding_count() * std::mem::size_of::<super::MorphBinding16>(),
        coverage,
        positive_subcenters: field.positive_center_count(),
        anti_subcenters: field.anti_center_count(),
        minimum_positive: calibration.minimum_positive,
        minimum_authority_margin: calibration.minimum_margin,
        train,
        heldout,
        heldout_without_anti_top1_percent: heldout_without_anti.top1_percent,
        candidate_permutation_parity,
        candidate_permutation_cases,
        runtime_connected: false,
        verdict,
    })
    .map_err(|error| error.to_string())
}

fn evaluate(
    field: &MorphologyField,
    _corpus: &MorphCorpus,
    examples: &[MorphExample],
    with_anti: bool,
) -> SplitMetrics {
    let mut metrics = SplitMetrics::default();
    let mut score_cache = BTreeMap::<String, BTreeMap<u32, MorphSlotScore>>::new();
    for example in examples {
        metrics.cases += 1;
        let scores = score_cache
            .entry(example.context.clone())
            .or_insert_with(|| field.score_slots(&example.context, with_anti));
        let ranked = field.ranked_for_lemma_with_scores(example.lemma_id, scores);
        let top1_correct = ranked
            .first()
            .is_some_and(|candidate| candidate.features == example.features);
        let exact_surface_top1 = ranked.first().is_some_and(|candidate| {
            candidate.features == example.features && candidate.surface == example.surface
        });
        metrics.top1_correct += usize::from(top1_correct);
        metrics.exact_surface_top1_correct += usize::from(exact_surface_top1);

        let readout = if with_anti {
            field.readout_for_lemma_with_scores(example.lemma_id, scores)
        } else {
            MorphReadout::Abstain(ranked)
        };
        let (authority_correct, false_authority) = match readout {
            MorphReadout::Winner(candidate) => (
                usize::from(candidate.features == example.features),
                usize::from(candidate.features != example.features),
            ),
            MorphReadout::Tied(_) => {
                metrics.tied += 1;
                (0, 0)
            }
            MorphReadout::Abstain(_) => {
                metrics.abstain += 1;
                (0, 0)
            }
        };
        metrics.authority_correct += authority_correct;
        metrics.false_authority += false_authority;

        let case = metrics
            .per_case
            .entry(feature_name(example.features).to_string())
            .or_default();
        case.cases += 1;
        case.top1_correct += usize::from(top1_correct);
        case.authority_correct += authority_correct;
        case.false_authority += false_authority;
    }
    metrics.top1_percent = percent(metrics.top1_correct, metrics.cases);
    metrics.exact_surface_top1_percent = percent(metrics.exact_surface_top1_correct, metrics.cases);
    metrics.authority_percent = percent(metrics.authority_correct, metrics.cases);
    for case in metrics.per_case.values_mut() {
        case.top1_percent = percent(case.top1_correct, case.cases);
        case.authority_percent = percent(case.authority_correct, case.cases);
    }
    metrics
}

fn morphology_coverage(corpus: &MorphCorpus) -> MorphologyCoverage {
    let mut numbers_by_lemma = vec![0_u32; corpus.lemmas.len()];
    let mut forms_by_slot = BTreeMap::<(u32, u32), BTreeSet<u32>>::new();
    let mut singular_forms = BTreeSet::new();
    let mut plural_forms = BTreeSet::new();
    for binding in &corpus.bindings {
        let number = binding.features & (NUMBER_SINGULAR | NUMBER_PLURAL);
        numbers_by_lemma[binding.lemma_center_id as usize] |= number;
        forms_by_slot
            .entry((binding.lemma_center_id, binding.features))
            .or_default()
            .insert(binding.form_center_id);
        if number == NUMBER_SINGULAR {
            singular_forms.insert(binding.form_center_id);
        } else if number == NUMBER_PLURAL {
            plural_forms.insert(binding.form_center_id);
        }
    }
    let both = NUMBER_SINGULAR | NUMBER_PLURAL;
    MorphologyCoverage {
        dual_number_lemmas: numbers_by_lemma
            .iter()
            .filter(|numbers| **numbers == both)
            .count(),
        plural_only_lemmas: numbers_by_lemma
            .iter()
            .filter(|numbers| **numbers == NUMBER_PLURAL)
            .count(),
        singular_only_lemmas: numbers_by_lemma
            .iter()
            .filter(|numbers| **numbers == NUMBER_SINGULAR)
            .count(),
        singular_form_centers: singular_forms.len(),
        plural_form_centers: plural_forms.len(),
        multi_surface_slots: forms_by_slot
            .values()
            .filter(|forms| forms.len() > 1)
            .count(),
        maximum_surfaces_per_slot: forms_by_slot.values().map(BTreeSet::len).max().unwrap_or(0),
    }
}

fn permutation_parity(field: &MorphologyField, examples: &[MorphExample]) -> bool {
    examples.iter().all(|example| {
        let mut form_ids = field
            .candidate_form_ids_for_lemma(example.lemma_id)
            .to_vec();
        let forward = field.readout_form_ids(&example.context, &form_ids);
        form_ids.reverse();
        forward == field.readout_form_ids(&example.context, &form_ids)
    })
}

fn stratified_permutation_examples(examples: &[MorphExample]) -> Vec<MorphExample> {
    const CASES_PER_SLOT: usize = 224;

    let mut selected = Vec::new();
    let mut counts = BTreeMap::<u32, usize>::new();
    for example in examples {
        let count = counts.entry(example.features).or_default();
        if *count < CASES_PER_SLOT {
            selected.push(example.clone());
            *count += 1;
        }
    }
    selected
}

fn percent(numerator: usize, denominator: usize) -> f64 {
    if denominator == 0 {
        0.0
    } else {
        numerator as f64 * 100.0 / denominator as f64
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn embedded_small_corpus_proves_cross_lemma_declension() {
        let report = run_embedded_russian_morphology_proof().expect("proof");
        assert_eq!(report["candidate_permutation_parity"], true);
        assert_eq!(report["heldout"]["false_authority"], 0);
        assert!(
            report["heldout"]["top1_percent"]
                .as_f64()
                .unwrap_or_default()
                >= 95.0,
            "{report:#}"
        );
    }
}
