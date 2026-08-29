use std::collections::{BTreeMap, BTreeSet, BinaryHeap};
use std::fs::File;
use std::io::{self, BufRead, BufReader};
use std::path::Path;
use std::time::Instant;

use serde::Serialize;

use super::runtime::{
    CompositionalFormBirth, L2LexicalSeed, L2LexicalSeedOrigin, L2LocalVerdict, StandaloneL2Field,
};
use crate::nanda_wave::lexical_grokking::{split_damages, DamageExample};

const EXPECTED_DAMAGE_CLASSES: usize = 13;

#[derive(Clone, Debug)]
struct ProofCase {
    target_form_ref: u32,
    target_lemma_ids: Vec<u32>,
    target_surface: String,
    feature_mask: u32,
    context: String,
    class: &'static str,
    damaged_surface: String,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
struct SampledScene {
    hash: u64,
    lemma: String,
    surface: String,
    feature_text: String,
    context: String,
}

#[derive(Default)]
struct SampledCases {
    cases: Vec<ProofCase>,
    scanned_lines: usize,
    heldout_rows: usize,
    eligible_heldout_rows: usize,
    reservoir_rows: usize,
    selected_target_forms: usize,
}

#[derive(Clone, Debug, Default, Serialize)]
struct ClassMetrics {
    cases: usize,
    known_context_modes: usize,
    known_context_modes_percent: f64,
    broad_lemma_retained: usize,
    broad_lemma_retained_percent: f64,
    active_lemma_retained: usize,
    active_lemma_retained_percent: f64,
    form_top8_retained: usize,
    form_top8_retained_percent: f64,
    form_top16_retained: usize,
    form_top16_retained_percent: f64,
    unique_form_top1: usize,
    unique_form_top1_percent: f64,
    readout_target_retained: usize,
    readout_target_retained_percent: f64,
    readout_unique_top1: usize,
    readout_unique_top1_percent: f64,
    false_authority: usize,
    abstain: usize,
}

impl ClassMetrics {
    fn merge(&mut self, other: Self) {
        self.cases += other.cases;
        self.known_context_modes += other.known_context_modes;
        self.broad_lemma_retained += other.broad_lemma_retained;
        self.active_lemma_retained += other.active_lemma_retained;
        self.form_top8_retained += other.form_top8_retained;
        self.form_top16_retained += other.form_top16_retained;
        self.unique_form_top1 += other.unique_form_top1;
        self.readout_target_retained += other.readout_target_retained;
        self.readout_unique_top1 += other.readout_unique_top1;
        self.false_authority += other.false_authority;
        self.abstain += other.abstain;
    }

    fn finish(&mut self) {
        self.known_context_modes_percent = percent(self.known_context_modes, self.cases);
        self.broad_lemma_retained_percent = percent(self.broad_lemma_retained, self.cases);
        self.active_lemma_retained_percent = percent(self.active_lemma_retained, self.cases);
        self.form_top8_retained_percent = percent(self.form_top8_retained, self.cases);
        self.form_top16_retained_percent = percent(self.form_top16_retained, self.cases);
        self.unique_form_top1_percent = percent(self.unique_form_top1, self.cases);
        self.readout_target_retained_percent = percent(self.readout_target_retained, self.cases);
        self.readout_unique_top1_percent = percent(self.readout_unique_top1, self.cases);
    }
}

#[derive(Default)]
struct ProofShard {
    classes: BTreeMap<&'static str, ClassMetrics>,
    lemma_birth_latency_us: Vec<u64>,
    context_reduction_latency_us: Vec<u64>,
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
        self.lemma_birth_latency_us
            .extend(other.lemma_birth_latency_us);
        self.context_reduction_latency_us
            .extend(other.context_reduction_latency_us);
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

#[expect(
    clippy::too_many_arguments,
    reason = "existing explicit boundary contract"
)]
pub(super) fn prove_package(
    l1_package_path: &Path,
    l2_package_path: &Path,
    morphology_corpus_path: &Path,
    heldout_per_class: usize,
    requested_workers: usize,
    broad_lemma_limit: usize,
    active_lemma_limit: usize,
    feature_limit: usize,
    form_limit: usize,
    atom_relation_limit: usize,
) -> io::Result<serde_json::Value> {
    if heldout_per_class == 0
        || broad_lemma_limit == 0
        || active_lemma_limit == 0
        || active_lemma_limit > broad_lemma_limit
        || feature_limit == 0
        || form_limit < 16
    {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "contextual compositional proof requires heldout > 0, 0 < active lemmas <= broad lemmas, feature limit > 0, and form limit >= 16",
        ));
    }

    let l2_started = Instant::now();
    let field = StandaloneL2Field::load(l2_package_path).map_err(io::Error::other)?;
    let l2_cold_load_us = l2_started.elapsed().as_micros() as u64;
    let rss_after_l2_load_kib = proc_status_kib("VmRSS:");
    let (package_storage, package_backing_bytes) = field.package_storage();
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
    let sampled = sample_fixed_context_cases(&field, morphology_corpus_path, heldout_per_class)?;
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
                    evaluate_cases(
                        cases,
                        field,
                        broad_lemma_limit,
                        active_lemma_limit,
                        feature_limit,
                        form_limit,
                        atom_relation_limit,
                    )
                })
            })
            .collect::<Vec<_>>()
            .into_iter()
            .map(|worker| {
                worker
                    .join()
                    .expect("contextual compositional proof worker panicked")
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
    proof.lemma_birth_latency_us.sort_unstable();
    proof.context_reduction_latency_us.sort_unstable();
    proof.form_birth_latency_us.sort_unstable();
    proof.readout_latency_us.sort_unstable();

    let false_authority = proof
        .classes
        .values()
        .map(|metrics| metrics.false_authority)
        .sum::<usize>();
    let retention_passed = proof.classes.len() == EXPECTED_DAMAGE_CLASSES
        && proof.classes.values().all(|metrics| {
            metrics.active_lemma_retained_percent > 95.0
                && metrics.form_top16_retained_percent > 95.0
                && metrics.readout_target_retained_percent > 95.0
        });
    let passed = retention_passed && false_authority == 0;
    let package_bytes = std::fs::metadata(l2_package_path)?.len();
    let failure_examples = proof
        .failure_examples
        .values()
        .flatten()
        .cloned()
        .collect::<Vec<_>>();

    Ok(serde_json::json!({
        "kind": "canonical_l2_contextual_compositional_fixed_proof",
        "verdict": if passed { "PASS_shadow_retention" } else { "FAIL" },
        "scope": "real heldout L2 teacher contexts: typed broad lemma birth, learned context reduction, exact form expansion, and bounded readout retention",
        "l1_package": l1_package_path,
        "l2_package": l2_package_path,
        "morphology_corpus": morphology_corpus_path,
        "forms": field.form_count(),
        "heldout_per_class": heldout_per_class,
        "class_count": proof.classes.len(),
        "evaluated": proof.classes.values().map(|metrics| metrics.cases).sum::<usize>(),
        "classes": proof.classes,
        "false_authority": false_authority,
        "failure_examples": failure_examples,
        "false_authority_examples": proof.false_authority_examples,
        "sampling": {
            "strategy": "bounded deterministic lowest-hash reservoir over real heldout H rows; one deterministic damage per target and class",
            "scanned_lines": sampled.scanned_lines,
            "heldout_rows": sampled.heldout_rows,
            "eligible_heldout_rows": sampled.eligible_heldout_rows,
            "reservoir_rows": sampled.reservoir_rows,
            "selected_target_forms": sampled.selected_target_forms,
            "sampling_us": sampling_us,
        },
        "limits": {
            "broad_lemma_frontier": broad_lemma_limit,
            "active_lemma_frontier": active_lemma_limit,
            "features_per_lemma": feature_limit,
            "geometry": "exact_bounded_damerau",
            "form_lattice": form_limit,
            "atom_relation_budget": atom_relation_limit,
        },
        "latency": {
            "lemma_birth_p50_us": percentile(&proof.lemma_birth_latency_us, 50),
            "lemma_birth_p99_us": percentile(&proof.lemma_birth_latency_us, 99),
            "context_reduction_p50_us": percentile(&proof.context_reduction_latency_us, 50),
            "context_reduction_p99_us": percentile(&proof.context_reduction_latency_us, 99),
            "form_birth_p50_us": percentile(&proof.form_birth_latency_us, 50),
            "form_birth_p99_us": percentile(&proof.form_birth_latency_us, 99),
            "readout_p50_us": percentile(&proof.readout_latency_us, 50),
            "readout_p99_us": percentile(&proof.readout_latency_us, 99),
            "proof_us": proof_us,
            "workers": workers,
        },
        "memory": {
            "package_bytes": package_bytes,
            "package_storage": package_storage,
            "package_backing_bytes": package_backing_bytes,
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
            "each_active_lemma_retained_strictly_above_percent": 95.0,
            "each_form_top16_retained_strictly_above_percent": 95.0,
            "each_readout_target_retained_strictly_above_percent": 95.0,
            "false_authority_equals": 0,
        },
        "not_tested": [
            "clean preservation; owned by the separate fixed word restoration proof",
            "live L1.1 seed authority transfer",
            "L3/L4/DecisionCore final apply authority",
            "daemon and IBus latency",
        ],
        "runtime_authority_changed_by_proof": false,
    }))
}

fn sample_fixed_context_cases(
    field: &StandaloneL2Field,
    morphology_corpus_path: &Path,
    heldout_per_class: usize,
) -> io::Result<SampledCases> {
    let capacity = heldout_per_class
        .saturating_mul(16)
        .max(heldout_per_class.saturating_add(64));
    let file = File::open(morphology_corpus_path)?;
    let mut reader = BufReader::with_capacity(1024 * 1024, file);
    let mut line = Vec::with_capacity(256);
    let mut reservoir = BinaryHeap::<SampledScene>::with_capacity(capacity + 1);
    let mut sampled = SampledCases::default();
    loop {
        line.clear();
        if reader.read_until(b'\n', &mut line)? == 0 {
            break;
        }
        sampled.scanned_lines += 1;
        if !line.starts_with(b"H\t") {
            continue;
        }
        sampled.heldout_rows += 1;
        let raw = std::str::from_utf8(&line).map_err(|error| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                format!("morphology corpus is not UTF-8: {error}"),
            )
        })?;
        let raw = raw.trim_end_matches(['\r', '\n']);
        let fields = raw.split('\t').collect::<Vec<_>>();
        let ["H", lemma, surface, feature_text, context] = fields.as_slice() else {
            continue;
        };
        if !eligible_surface(surface) {
            continue;
        }
        sampled.eligible_heldout_rows += 1;
        let candidate = SampledScene {
            hash: stable_hash(raw.as_bytes()),
            lemma: (*lemma).to_string(),
            surface: (*surface).to_string(),
            feature_text: (*feature_text).to_string(),
            context: (*context).to_string(),
        };
        if reservoir.len() < capacity {
            reservoir.push(candidate);
        } else if reservoir.peek().is_some_and(|largest| candidate < *largest) {
            reservoir.pop();
            reservoir.push(candidate);
        }
    }

    let mut scenes = reservoir.into_sorted_vec();
    scenes.sort_unstable();
    sampled.reservoir_rows = scenes.len();
    let mut by_class = BTreeMap::<&'static str, Vec<ProofCase>>::new();
    let mut target_forms = BTreeSet::new();
    for scene in scenes {
        let feature_mask = crate::nanda_wave::morphology_phase::parse_features(&scene.feature_text)
            .map_err(io::Error::other)?;
        let Some(target_form_ref) = field.form_ref_for_surface(&scene.surface) else {
            continue;
        };
        let target_lemma_ids = field.lemma_ids_for_form_feature(target_form_ref, feature_mask);
        if target_lemma_ids.is_empty() {
            continue;
        }
        let (_, heldout) = split_damages(&scene.surface);
        let mut one_per_class = BTreeMap::<&'static str, DamageExample>::new();
        for example in heldout {
            one_per_class
                .entry(example.class)
                .and_modify(|selected| {
                    if damage_hash(&scene.surface, &scene.context, &example)
                        < damage_hash(&scene.surface, &scene.context, selected)
                    {
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
                target_form_ref,
                target_lemma_ids: target_lemma_ids.clone(),
                target_surface: scene.surface.clone(),
                feature_mask,
                context: scene.context.clone(),
                class,
                damaged_surface: example.surface,
            });
            target_forms.insert(target_form_ref);
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
                "contextual compositional proof cannot fill {heldout_per_class} cases for all 13 classes: {counts}"
            ),
        ));
    }
    sampled.selected_target_forms = target_forms.len();
    sampled.cases = by_class.into_values().flatten().collect();
    Ok(sampled)
}

fn evaluate_cases(
    cases: &[ProofCase],
    field: &StandaloneL2Field,
    broad_lemma_limit: usize,
    active_lemma_limit: usize,
    feature_limit: usize,
    form_limit: usize,
    atom_relation_limit: usize,
) -> ProofShard {
    let mut shard = ProofShard::default();
    for case in cases {
        let lemma_started = Instant::now();
        let broad = field.compositional_lemma_births_with_atom_relation_limit(
            &case.damaged_surface,
            broad_lemma_limit,
            atom_relation_limit,
        );
        shard
            .lemma_birth_latency_us
            .push(lemma_started.elapsed().as_micros() as u64);
        let context_started = Instant::now();
        let active =
            field.contextual_compositional_lemma_births(&case.context, &broad, active_lemma_limit);
        shard
            .context_reduction_latency_us
            .push(context_started.elapsed().as_micros() as u64);
        let form_started = Instant::now();
        let births = field.contextual_compositional_form_births_from_lemmas(
            &case.context,
            &case.damaged_surface,
            &active,
            feature_limit,
            form_limit,
        );
        shard
            .form_birth_latency_us
            .push(form_started.elapsed().as_micros() as u64);

        let broad_retained = broad
            .iter()
            .any(|birth| case.target_lemma_ids.contains(&birth.lemma_id));
        let active_retained = active
            .iter()
            .any(|birth| case.target_lemma_ids.contains(&birth.lemma_id));
        let form_top8 = births
            .iter()
            .take(8)
            .any(|birth| birth.form_ref == case.target_form_ref);
        let form_top16 = births
            .iter()
            .any(|birth| birth.form_ref == case.target_form_ref);
        let unique_form_top1 = unique_target_top1(&births, case.target_form_ref);
        let seeds = births
            .iter()
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
        let readout = field.readout(&case.context, &seeds, form_limit);
        shard
            .readout_latency_us
            .push(readout_started.elapsed().as_micros() as u64);
        let readout_retained = readout
            .candidates
            .iter()
            .any(|candidate| candidate.form_ref == case.target_form_ref);
        let readout_unique_top1 = readout
            .candidates
            .first()
            .is_some_and(|first| first.form_ref == case.target_form_ref)
            && readout
                .candidates
                .get(1)
                .is_none_or(|second| readout.candidates[0].local_score != second.local_score);
        let false_authority = matches!(
            readout.verdict,
            L2LocalVerdict::Winner { form_ref } if form_ref != case.target_form_ref
        );
        let metrics = shard.classes.entry(case.class).or_default();
        metrics.cases += 1;
        metrics.known_context_modes += usize::from(field.context_mode_known(&case.context));
        metrics.broad_lemma_retained += usize::from(broad_retained);
        metrics.active_lemma_retained += usize::from(active_retained);
        metrics.form_top8_retained += usize::from(form_top8);
        metrics.form_top16_retained += usize::from(form_top16);
        metrics.unique_form_top1 += usize::from(unique_form_top1);
        metrics.readout_target_retained += usize::from(readout_retained);
        metrics.readout_unique_top1 += usize::from(readout_unique_top1);
        metrics.false_authority += usize::from(false_authority);
        metrics.abstain += usize::from(matches!(readout.verdict, L2LocalVerdict::Abstain));

        if (!active_retained || !form_top16 || !readout_retained)
            && shard.failure_examples.entry(case.class).or_default().len() < 8
        {
            shard
                .failure_examples
                .entry(case.class)
                .or_default()
                .push(serde_json::json!({
                    "class": case.class,
                    "target": case.target_surface,
                    "damaged": case.damaged_surface,
                    "feature_mask": case.feature_mask,
                    "context": case.context,
                    "context_mode_known": field.context_mode_known(&case.context),
                    "broad_lemma_retained": broad_retained,
                    "active_lemma_retained": active_retained,
                    "form_top16_retained": form_top16,
                    "readout_retained": readout_retained,
                    "births": birth_surfaces(field, &births, 8),
                }));
        }
        if false_authority && shard.false_authority_examples.len() < 32 {
            shard.false_authority_examples.push(serde_json::json!({
                "class": case.class,
                "target": case.target_surface,
                "damaged": case.damaged_surface,
                "context": case.context,
                "verdict": format!("{:?}", readout.verdict),
            }));
        }
    }
    shard
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
                "lemma_id": birth.lemma_id,
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

fn stable_hash(bytes: &[u8]) -> u64 {
    bytes.iter().fold(0x6a09_e667_f3bc_c909, |state, byte| {
        crate::stable_hash::mix64_golden(state ^ u64::from(*byte))
    })
}

fn damage_hash(target: &str, context: &str, example: &DamageExample) -> u64 {
    stable_hash(
        target
            .bytes()
            .chain(context.bytes())
            .chain(example.class.bytes())
            .chain(example.surface.bytes())
            .collect::<Vec<_>>()
            .as_slice(),
    )
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
    let index = (values.len() - 1).saturating_mul(percentile.min(100)) / 100;
    values[index]
}

fn proc_status_kib(prefix: &str) -> u64 {
    std::fs::read_to_string("/proc/self/status")
        .ok()
        .and_then(|status| {
            status.lines().find_map(|line| {
                line.strip_prefix(prefix)?
                    .split_whitespace()
                    .next()?
                    .parse::<u64>()
                    .ok()
            })
        })
        .unwrap_or_default()
}
