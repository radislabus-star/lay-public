use std::collections::{BTreeMap, BTreeSet};
use std::io;
use std::path::Path;
use std::time::Instant;

use serde::Serialize;

use super::runtime::{
    CompositionalFormBirth, L2LexicalSeed, L2LexicalSeedOrigin, L2LocalVerdict, StandaloneL2Field,
};
use crate::nanda_wave::lexical_grokking::{split_damages, DamageExample};

const EXPECTED_DAMAGE_CLASSES: usize = 13;
const AUTHORITY_PROBE_FORM_LIMIT: usize = 16;

#[derive(Clone, Debug)]
struct ProofCase {
    target_form_ref: u32,
    target_surface: String,
    class: &'static str,
    damaged_surface: String,
}

#[derive(Clone, Debug, Default, Serialize)]
struct ClassMetrics {
    cases: usize,
    unique_birth_top1: usize,
    unique_birth_top1_percent: f64,
    top8_retained: usize,
    top8_retained_percent: f64,
    top16_retained: usize,
    top16_retained_percent: f64,
    empty_lattice: usize,
    false_authority: usize,
    exact_surface_collisions: usize,
    collision_target_retained: usize,
    non_collision_cases: usize,
    non_collision_unique_birth_top1: usize,
    non_collision_unique_birth_top1_percent: f64,
    target_geometry_top: usize,
    target_geometry_top_ties: usize,
}

impl ClassMetrics {
    fn merge(&mut self, other: Self) {
        self.cases += other.cases;
        self.unique_birth_top1 += other.unique_birth_top1;
        self.top8_retained += other.top8_retained;
        self.top16_retained += other.top16_retained;
        self.empty_lattice += other.empty_lattice;
        self.false_authority += other.false_authority;
        self.exact_surface_collisions += other.exact_surface_collisions;
        self.collision_target_retained += other.collision_target_retained;
        self.non_collision_cases += other.non_collision_cases;
        self.non_collision_unique_birth_top1 += other.non_collision_unique_birth_top1;
        self.target_geometry_top += other.target_geometry_top;
        self.target_geometry_top_ties += other.target_geometry_top_ties;
    }

    fn finish(&mut self) {
        self.unique_birth_top1_percent = percent(self.unique_birth_top1, self.cases);
        self.top8_retained_percent = percent(self.top8_retained, self.cases);
        self.top16_retained_percent = percent(self.top16_retained, self.cases);
        self.non_collision_unique_birth_top1_percent = percent(
            self.non_collision_unique_birth_top1,
            self.non_collision_cases,
        );
    }
}

#[derive(Default)]
struct ProofShard {
    classes: BTreeMap<&'static str, ClassMetrics>,
    birth_latency_us: Vec<u64>,
    lemma_birth_latency_us: Vec<u64>,
    form_birth_latency_us: Vec<u64>,
    readout_latency_us: Vec<u64>,
    failure_examples: BTreeMap<&'static str, Vec<serde_json::Value>>,
    false_authority_examples: Vec<serde_json::Value>,
}

impl ProofShard {
    fn merge(&mut self, other: Self) {
        for (class, metrics) in other.classes {
            self.classes.entry(class).or_default().merge(metrics);
        }
        self.birth_latency_us.extend(other.birth_latency_us);
        self.lemma_birth_latency_us
            .extend(other.lemma_birth_latency_us);
        self.form_birth_latency_us
            .extend(other.form_birth_latency_us);
        self.readout_latency_us.extend(other.readout_latency_us);
        for (class, examples) in other.failure_examples {
            let retained = self.failure_examples.entry(class).or_default();
            retained.extend(examples);
            retained.truncate(8);
        }
        self.false_authority_examples
            .extend(other.false_authority_examples);
        self.false_authority_examples.truncate(32);
    }
}

struct SampledCases {
    cases: Vec<ProofCase>,
    scanned_forms: usize,
    eligible_l2_only_forms: usize,
    target_forms: usize,
}

pub(super) fn prove_package(
    l1_package_path: &Path,
    l2_package_path: &Path,
    heldout_per_class: usize,
    requested_workers: usize,
    lemma_limit: usize,
    form_limit: usize,
    atom_relation_limit: usize,
) -> io::Result<serde_json::Value> {
    if heldout_per_class == 0 || lemma_limit == 0 || form_limit < 16 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "compositional L2 proof requires heldout-per-class > 0, lemma-limit > 0, and form-limit >= 16",
        ));
    }

    let l2_started = Instant::now();
    let field = StandaloneL2Field::load(l2_package_path).map_err(io::Error::other)?;
    let l2_cold_load_us = l2_started.elapsed().as_micros() as u64;
    let rss_after_l2_load_kib = proc_status_kib("VmRSS:");
    let l1_started = Instant::now();
    let l1 = crate::nanda_wave::L1RestorationHost::load(l1_package_path)?;
    let l1_cold_load_us = l1_started.elapsed().as_micros() as u64;
    if field.l1_package_fingerprint() != l1.corpus_fingerprint() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "L2 package was compiled for a different L1.1 corpus fingerprint",
        ));
    }

    let sampling_started = Instant::now();
    let sampled = sample_fixed_cases(&field, heldout_per_class)?;
    let sampling_us = sampling_started.elapsed().as_micros() as u64;
    let workers = requested_workers
        .max(1)
        .min(sampled.cases.len().max(1))
        .min(
            std::thread::available_parallelism()
                .map(usize::from)
                .unwrap_or(1),
        );
    let chunk_size = sampled.cases.len().div_ceil(workers).max(1);
    let proof_started = Instant::now();
    let partials = std::thread::scope(|scope| {
        sampled
            .cases
            .chunks(chunk_size)
            .map(|cases| {
                let field = &field;
                scope.spawn(move || {
                    evaluate_cases(cases, field, lemma_limit, form_limit, atom_relation_limit)
                })
            })
            .collect::<Vec<_>>()
            .into_iter()
            .map(|worker| {
                worker
                    .join()
                    .expect("compositional L2 proof worker panicked")
            })
            .collect::<Vec<_>>()
    });
    let mut proof = ProofShard::default();
    for partial in partials {
        proof.merge(partial);
    }
    let proof_us = proof_started.elapsed().as_micros() as u64;
    for metrics in proof.classes.values_mut() {
        metrics.finish();
    }
    proof.birth_latency_us.sort_unstable();
    proof.lemma_birth_latency_us.sort_unstable();
    proof.form_birth_latency_us.sort_unstable();
    proof.readout_latency_us.sort_unstable();

    let clean_targets = sampled
        .cases
        .iter()
        .map(|case| (case.target_form_ref, case.target_surface.clone()))
        .collect::<BTreeMap<_, _>>()
        .into_iter()
        .collect::<Vec<_>>();
    let clean = evaluate_clean_targets(
        &field,
        &clean_targets,
        workers,
        lemma_limit,
        form_limit,
        atom_relation_limit,
    );
    let clean_unique_top1_percent = percent(clean.unique_top1, clean.cases);
    let clean_top16_retained_percent = percent(clean.top16_retained, clean.cases);
    let false_authority = proof
        .classes
        .values()
        .map(|metrics| metrics.false_authority)
        .sum::<usize>();
    let all_classes_pass = proof.classes.len() == EXPECTED_DAMAGE_CLASSES
        && proof.classes.values().all(|metrics| {
            metrics.cases == heldout_per_class
                && metrics.unique_birth_top1_percent > 95.0
                && metrics.top8_retained_percent >= 99.0
                && metrics.top16_retained_percent >= 99.0
        });
    let passed = all_classes_pass
        && clean_unique_top1_percent >= 99.9
        && clean_top16_retained_percent >= 99.9
        && false_authority == 0;
    let (package_storage, package_resident_bytes) = field.package_storage();
    let package_bytes = std::fs::metadata(l2_package_path)?.len();
    let failure_examples = proof
        .failure_examples
        .values()
        .flatten()
        .cloned()
        .collect::<Vec<_>>();

    Ok(serde_json::json!({
        "kind": "canonical_l2_compositional_restoration_fixed_proof",
        "verdict": if passed { "PASS" } else { "FAIL" },
        "scope": "L2-only exact form birth and lattice retention over the shared L1.1 damage generator",
        "l1_package": l1_package_path,
        "l2_package": l2_package_path,
        "l1_package_fingerprint": l1.corpus_fingerprint(),
        "forms": field.form_count(),
        "heldout_per_class": heldout_per_class,
        "class_count": proof.classes.len(),
        "evaluated": proof.classes.values().map(|metrics| metrics.cases).sum::<usize>(),
        "classes": proof.classes,
        "clean": {
            "cases": clean.cases,
            "unique_birth_top1": clean.unique_top1,
            "unique_birth_top1_percent": clean_unique_top1_percent,
            "top16_retained": clean.top16_retained,
            "top16_retained_percent": clean_top16_retained_percent,
        },
        "false_authority": false_authority,
        "failure_examples": failure_examples,
        "false_authority_examples": proof.false_authority_examples,
        "sampling": {
            "strategy": "deterministic coprime form permutation; at most one heldout damage per target and class",
            "scanned_forms": sampled.scanned_forms,
            "eligible_l2_only_forms": sampled.eligible_l2_only_forms,
            "selected_target_forms": sampled.target_forms,
            "sampling_us": sampling_us,
        },
        "limits": {
            "lemma_frontier": lemma_limit,
            "form_lattice": form_limit,
            "atom_relation_budget": atom_relation_limit,
        },
        "latency": {
            "birth_p50_us": percentile(&proof.birth_latency_us, 50),
            "birth_p99_us": percentile(&proof.birth_latency_us, 99),
            "lemma_birth_p50_us": percentile(&proof.lemma_birth_latency_us, 50),
            "lemma_birth_p99_us": percentile(&proof.lemma_birth_latency_us, 99),
            "form_birth_p50_us": percentile(&proof.form_birth_latency_us, 50),
            "form_birth_p99_us": percentile(&proof.form_birth_latency_us, 99),
            "composition_only_readout_p50_us": percentile(&proof.readout_latency_us, 50),
            "composition_only_readout_p99_us": percentile(&proof.readout_latency_us, 99),
            "proof_us": proof_us,
            "workers": workers,
        },
        "memory": {
            "package_bytes": package_bytes,
            "package_storage": package_storage,
            "package_resident_bytes": package_resident_bytes,
            "package_mmap_backed": field.package_mmap_backed(),
            "compositional_index_source": field.compositional_index_source(),
            "compositional_index_resident_bytes": field.compositional_index_bytes(),
            "compositional_index_view_bytes": field.compositional_index_view_bytes(),
            "rss_after_l2_load_kib": rss_after_l2_load_kib,
            "rss_peak_kib": proc_status_kib("VmHWM:"),
        },
        "cold_load": {
            "l2_us": l2_cold_load_us,
            "l1_us": l1_cold_load_us,
        },
        "gates": {
            "each_unique_birth_top1_strictly_above_percent": 95.0,
            "each_top8_retained_at_least_percent": 99.0,
            "each_top16_retained_at_least_percent": 99.0,
            "clean_unique_birth_top1_at_least_percent": 99.9,
            "clean_top16_retained_at_least_percent": 99.9,
            "false_authority_equals": 0,
        },
        "not_tested": [
            "sentence-context ranking between valid morphology cells",
            "final L3/L4/DecisionCore apply authority",
            "daemon and IBus latency",
        ],
        "runtime_authority_changed_by_proof": false,
    }))
}

fn sample_fixed_cases(
    field: &StandaloneL2Field,
    heldout_per_class: usize,
) -> io::Result<SampledCases> {
    let form_count = field.form_count();
    if form_count == 0 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "compositional L2 proof found no exact forms",
        ));
    }
    let mut step = (0x9e37_79b9_usize % form_count).max(1);
    while greatest_common_divisor(step, form_count) != 1 {
        step = if step + 1 == form_count { 1 } else { step + 1 };
    }
    let start = 0x51ed_270b_usize % form_count;
    let mut by_class = BTreeMap::<&'static str, Vec<ProofCase>>::new();
    let mut scanned_forms = 0_usize;
    let mut eligible_l2_only_forms = 0_usize;
    let mut target_forms = BTreeSet::new();
    for iteration in 0..form_count {
        let form_ref =
            ((start as u128 + iteration as u128 * step as u128) % form_count as u128) as u32;
        scanned_forms += 1;
        if field.l1_terminal_for_form_ref(form_ref).is_some() {
            continue;
        }
        let Some(surface) = field.decode_form_ref(form_ref) else {
            continue;
        };
        let surface = surface.into_owned();
        if !eligible_surface(&surface) {
            continue;
        }
        eligible_l2_only_forms += 1;
        let (_, heldout) = split_damages(&surface);
        let mut one_per_class = BTreeMap::<&'static str, DamageExample>::new();
        for example in heldout {
            one_per_class
                .entry(example.class)
                .and_modify(|selected| {
                    if case_hash(&surface, &example) < case_hash(&surface, selected) {
                        *selected = example.clone();
                    }
                })
                .or_insert(example);
        }
        for (class, example) in one_per_class {
            let bucket = by_class.entry(class).or_default();
            if bucket.len() == heldout_per_class {
                continue;
            }
            bucket.push(ProofCase {
                target_form_ref: form_ref,
                target_surface: surface.clone(),
                class,
                damaged_surface: example.surface,
            });
            target_forms.insert(form_ref);
        }
        if by_class.len() == EXPECTED_DAMAGE_CLASSES
            && by_class
                .values()
                .all(|cases| cases.len() == heldout_per_class)
        {
            break;
        }
    }

    if by_class.len() != EXPECTED_DAMAGE_CLASSES
        || by_class
            .values()
            .any(|cases| cases.len() != heldout_per_class)
    {
        let counts = by_class
            .iter()
            .map(|(class, cases)| format!("{class}={}", cases.len()))
            .collect::<Vec<_>>()
            .join(", ");
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!(
                "compositional L2 proof cannot fill {heldout_per_class} cases for all 13 classes: {counts}"
            ),
        ));
    }
    let cases = by_class.into_values().flatten().collect::<Vec<_>>();
    Ok(SampledCases {
        cases,
        scanned_forms,
        eligible_l2_only_forms,
        target_forms: target_forms.len(),
    })
}

fn evaluate_cases(
    cases: &[ProofCase],
    field: &StandaloneL2Field,
    lemma_limit: usize,
    form_limit: usize,
    atom_relation_limit: usize,
) -> ProofShard {
    let mut shard = ProofShard::default();
    for case in cases {
        let birth_started = Instant::now();
        let lemma_birth_started = Instant::now();
        let lemma_births = field.compositional_lemma_births_with_atom_relation_limit(
            &case.damaged_surface,
            lemma_limit,
            atom_relation_limit,
        );
        shard
            .lemma_birth_latency_us
            .push(lemma_birth_started.elapsed().as_micros() as u64);
        let form_birth_started = Instant::now();
        let births = field.compositional_form_births_from_lemmas(
            &case.damaged_surface,
            &lemma_births,
            form_limit,
        );
        shard
            .form_birth_latency_us
            .push(form_birth_started.elapsed().as_micros() as u64);
        shard
            .birth_latency_us
            .push(birth_started.elapsed().as_micros() as u64);
        let unique_top1 = unique_target_top1(&births, case.target_form_ref);
        let exact_surface_collision = field
            .form_ref_for_surface(&case.damaged_surface)
            .is_some_and(|form_ref| form_ref != case.target_form_ref);
        let top8 = births
            .iter()
            .take(8)
            .any(|birth| birth.form_ref == case.target_form_ref);
        let top16 = births
            .iter()
            .any(|birth| birth.form_ref == case.target_form_ref);
        let best_geometry = births
            .iter()
            .map(|birth| birth.geometry_evidence_milli)
            .max();
        let target_geometry = births
            .iter()
            .find(|birth| birth.form_ref == case.target_form_ref)
            .map(|birth| birth.geometry_evidence_milli);
        let target_geometry_top = target_geometry.is_some() && target_geometry == best_geometry;
        let target_geometry_top_tie = target_geometry_top
            && births
                .iter()
                .filter(|birth| Some(birth.geometry_evidence_milli) == best_geometry)
                .count()
                > 1;
        let seeds = births
            .iter()
            .take(AUTHORITY_PROBE_FORM_LIMIT)
            .filter_map(|birth| {
                Some(L2LexicalSeed {
                    terminal_id: field.l1_terminal_for_form_ref(birth.form_ref),
                    surface: Some(field.decode_form_ref(birth.form_ref)?.into_owned()),
                    evidence_milli: i32::from(birth.evidence_milli),
                    origin: L2LexicalSeedOrigin::CompositionalMorphology,
                })
            })
            .collect::<Vec<_>>();
        let readout_started = Instant::now();
        let readout = field.readout("_", &seeds, AUTHORITY_PROBE_FORM_LIMIT);
        shard
            .readout_latency_us
            .push(readout_started.elapsed().as_micros() as u64);
        let false_authority = !matches!(readout.verdict, L2LocalVerdict::Abstain);
        let metrics = shard.classes.entry(case.class).or_default();
        metrics.cases += 1;
        metrics.unique_birth_top1 += usize::from(unique_top1);
        metrics.top8_retained += usize::from(top8);
        metrics.top16_retained += usize::from(top16);
        metrics.empty_lattice += usize::from(births.is_empty());
        metrics.false_authority += usize::from(false_authority);
        metrics.exact_surface_collisions += usize::from(exact_surface_collision);
        metrics.collision_target_retained += usize::from(exact_surface_collision && top16);
        metrics.non_collision_cases += usize::from(!exact_surface_collision);
        metrics.non_collision_unique_birth_top1 +=
            usize::from(!exact_surface_collision && unique_top1);
        metrics.target_geometry_top += usize::from(target_geometry_top);
        metrics.target_geometry_top_ties += usize::from(target_geometry_top_tie);

        let class_failures = shard.failure_examples.entry(case.class).or_default();
        if (!unique_top1 || !top8 || !top16) && class_failures.len() < 8 {
            class_failures.push(serde_json::json!({
                "class": case.class,
                "target": case.target_surface,
                "damaged": case.damaged_surface,
                "unique_top1": unique_top1,
                "top8": top8,
                "top16": top16,
                "exact_surface_collision": exact_surface_collision,
                "target_geometry_top": target_geometry_top,
                "target_geometry_top_tie": target_geometry_top_tie,
                "births": birth_surfaces(field, &births, 8),
            }));
        }
        if false_authority && shard.false_authority_examples.len() < 32 {
            shard.false_authority_examples.push(serde_json::json!({
                "class": case.class,
                "target": case.target_surface,
                "damaged": case.damaged_surface,
                "verdict": format!("{:?}", readout.verdict),
            }));
        }
    }
    shard
}

#[derive(Default)]
struct CleanMetrics {
    cases: usize,
    unique_top1: usize,
    top16_retained: usize,
}

impl CleanMetrics {
    fn merge(&mut self, other: Self) {
        self.cases += other.cases;
        self.unique_top1 += other.unique_top1;
        self.top16_retained += other.top16_retained;
    }
}

fn evaluate_clean_targets(
    field: &StandaloneL2Field,
    targets: &[(u32, String)],
    workers: usize,
    lemma_limit: usize,
    form_limit: usize,
    atom_relation_limit: usize,
) -> CleanMetrics {
    let chunk_size = targets.len().div_ceil(workers.max(1)).max(1);
    let partials = std::thread::scope(|scope| {
        targets
            .chunks(chunk_size)
            .map(|chunk| {
                scope.spawn(move || {
                    let mut metrics = CleanMetrics::default();
                    for (target_form_ref, surface) in chunk {
                        let births = field.compositional_form_births_with_atom_relation_limit(
                            surface,
                            lemma_limit,
                            form_limit,
                            atom_relation_limit,
                        );
                        metrics.cases += 1;
                        metrics.unique_top1 +=
                            usize::from(unique_target_top1(&births, *target_form_ref));
                        metrics.top16_retained += usize::from(
                            births
                                .iter()
                                .any(|birth| birth.form_ref == *target_form_ref),
                        );
                    }
                    metrics
                })
            })
            .collect::<Vec<_>>()
            .into_iter()
            .map(|worker| {
                worker
                    .join()
                    .expect("compositional clean proof worker panicked")
            })
            .collect::<Vec<_>>()
    });
    let mut metrics = CleanMetrics::default();
    for partial in partials {
        metrics.merge(partial);
    }
    metrics
}

fn unique_target_top1(births: &[CompositionalFormBirth], target_form_ref: u32) -> bool {
    let Some(first) = births.first() else {
        return false;
    };
    first.form_ref == target_form_ref
        && births
            .get(1)
            .is_none_or(|second| first.rank_evidence() != second.rank_evidence())
}

fn birth_surfaces(
    field: &StandaloneL2Field,
    births: &[CompositionalFormBirth],
    limit: usize,
) -> Vec<serde_json::Value> {
    births
        .iter()
        .take(limit)
        .map(|birth| {
            serde_json::json!({
                "surface": field.decode_form_ref(birth.form_ref).as_deref(),
                "evidence_milli": birth.evidence_milli,
                "geometry_evidence_milli": birth.geometry_evidence_milli,
                "atom_evidence_milli": birth.atom_evidence_milli,
                "lemma_evidence_milli": birth.lemma_evidence_milli,
                "wave_distance": birth.wave_distance,
            })
        })
        .collect()
}

fn eligible_surface(surface: &str) -> bool {
    let length = surface.chars().count();
    (7..=48).contains(&length)
        && (surface.chars().all(crate::keyboard::is_cyrillic_letter)
            || surface.chars().all(|ch| ch.is_ascii_alphabetic()))
}

fn case_hash(target: &str, example: &DamageExample) -> u64 {
    target
        .bytes()
        .chain(example.class.bytes())
        .chain(example.surface.bytes())
        .fold(0x6a09_e667_f3bc_c909, |state, byte| {
            crate::stable_hash::mix64_golden(state ^ u64::from(byte))
        })
}

fn greatest_common_divisor(mut left: usize, mut right: usize) -> usize {
    while right != 0 {
        (left, right) = (right, left % right);
    }
    left
}

fn percent(numerator: usize, denominator: usize) -> f64 {
    if denominator == 0 {
        0.0
    } else {
        numerator as f64 * 100.0 / denominator as f64
    }
}

fn percentile(values: &[u64], percentile: usize) -> u64 {
    if values.is_empty() {
        return 0;
    }
    values[(values.len() - 1) * percentile.min(100) / 100]
}

fn proc_status_kib(field: &str) -> Option<u64> {
    std::fs::read_to_string("/proc/self/status")
        .ok()?
        .lines()
        .find(|line| line.starts_with(field))?
        .split_whitespace()
        .nth(1)?
        .parse()
        .ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unique_top1_rejects_a_score_tie_even_when_target_sorts_first() {
        let target = CompositionalFormBirth {
            form_ref: 3,
            lemma_id: 1,
            evidence_milli: 900,
            geometry_evidence_milli: 900,
            atom_evidence_milli: 900,
            lemma_evidence_milli: 800,
            wave_distance: 7,
        };
        let tied = CompositionalFormBirth {
            form_ref: 4,
            lemma_id: 2,
            evidence_milli: 900,
            geometry_evidence_milli: 900,
            atom_evidence_milli: 900,
            lemma_evidence_milli: 800,
            wave_distance: 7,
        };
        let lower = CompositionalFormBirth {
            evidence_milli: 899,
            ..tied
        };

        assert!(!unique_target_top1(&[target, tied], 3));
        assert!(unique_target_top1(&[target, lower], 3));
    }

    #[test]
    fn coprime_permutation_visits_every_form_once() {
        let form_count = 42_usize;
        let mut step = 17_usize;
        while greatest_common_divisor(step, form_count) != 1 {
            step += 1;
        }
        let visited = (0..form_count)
            .map(|index| (11 + index * step) % form_count)
            .collect::<BTreeSet<_>>();
        assert_eq!(visited.len(), form_count);
    }
}
