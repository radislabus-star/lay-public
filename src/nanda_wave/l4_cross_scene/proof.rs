use std::collections::BTreeSet;
use std::fs;
use std::io;
use std::path::Path;
use std::time::Instant;

use serde::Serialize;

use super::compiler::{compile_observations, CrossSceneCompileConfig};
use super::encoder::{
    candidate_relation_id, context_signal_from_text, keep_relation_id, relation_class_from_context,
};
use super::format::{encode_package, read_package, write_package};
use super::model::{
    L4CrossSceneDisposition, L4CrossSceneL2Signal, L4CrossSceneObservation, L4CrossSceneProfileKey,
};
use super::runtime::readout;
use crate::nanda_wave::phase_field::stable_hash64;
use crate::transition_relation::{TransitionOperatorKind, TransitionRelationAtoms};
use crate::typing_memory::{LayoutProjectionDirection, LayoutProjectionScope, TypingMemoryOutcome};

const MAX_WORDS_PER_LANGUAGE: usize = 512;
const HOT_READOUT_REPEATS: usize = 32;
const HOT_READOUT_P99_LIMIT_NS: u64 = 500_000;

#[derive(Clone, Debug)]
struct ProofCase {
    observation: L4CrossSceneObservation,
    expected_support: bool,
    class: &'static str,
}

struct LoadedWords {
    selected: Vec<String>,
    eligible_tokens: usize,
}

#[derive(Clone, Debug, Default, Serialize)]
struct ClassScore {
    total: usize,
    expected_support: usize,
    supported: usize,
    repelled: usize,
    ambiguous: usize,
    unknown: usize,
    correct: usize,
    false_support: usize,
    automatic_apply: usize,
    percent: f32,
}

#[derive(Clone, Debug, Default, Serialize)]
struct AblationScore {
    negative_cases: usize,
    full_false_supports: usize,
    without_anti_false_supports: usize,
    anti_prevented_false_supports: usize,
    shuffled_sign_positive_supports: usize,
    full_positive_supports: usize,
    no_context_positive_supports: usize,
    shuffled_direction_positive_supports: usize,
}

#[derive(Clone, Debug, Default, Serialize)]
struct HotReadoutLatency {
    samples: usize,
    repeats: usize,
    p50_us: f64,
    p99_us: f64,
    max_us: f64,
    gate_us: u64,
    pass: bool,
}

pub(crate) fn prove_cross_scene_word_lists(
    russian_words: &Path,
    english_words: &Path,
    output: &Path,
) -> io::Result<serde_json::Value> {
    let russian = load_words(russian_words, true)?;
    let english = load_words(english_words, false)?;
    if russian.selected.len() < 40 || english.selected.len() < 40 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "L4 cross-scene proof requires at least 40 clean words per language",
        ));
    }
    let (ru_train, ru_heldout) = split_words(&russian.selected);
    let (en_train, en_heldout) = split_words(&english.selected);
    let train_overlap =
        overlap_count(&ru_train, &ru_heldout) + overlap_count(&en_train, &en_heldout);

    let mut receipt = 1_u64;
    let mut training = Vec::new();
    build_word_observations(&ru_train, &en_train, &mut receipt, &mut training, false);
    let mut cases = Vec::new();
    build_word_cases(&ru_heldout, &en_heldout, &mut receipt, &mut cases);
    let (grapheme_train, grapheme_cases) = grapheme_data(&mut receipt);
    training.extend(grapheme_train);
    cases.extend(grapheme_cases);

    let (package, compile_report) =
        compile_observations(&training, CrossSceneCompileConfig::default());
    write_package(output, &package)?;
    let restored = read_package(output)?;
    let package_bytes = fs::read(output)?;
    let roundtrip_bytes = encode_package(&restored);

    let mut reversed_training = training.clone();
    reversed_training.reverse();
    let (reordered, _) =
        compile_observations(&reversed_training, CrossSceneCompileConfig::default());
    let deterministic_order = encode_package(&package) == encode_package(&reordered);
    let package_roundtrip = package_bytes == roundtrip_bytes;
    let runtime_evaluator_parity = cases.iter().all(|case| {
        readout(&package, case.observation.input()) == readout(&restored, case.observation.input())
    });
    let candidate_order_parity = candidate_permutation_parity(&restored, &cases);
    let raw_text_absent = raw_text_is_absent(&package_bytes, &ru_train, &en_train);

    let mut by_class = std::collections::BTreeMap::<&'static str, ClassScore>::new();
    for case in &cases {
        let result = readout(&restored, case.observation.input());
        let score = by_class.entry(case.class).or_default();
        score.total += 1;
        score.expected_support += usize::from(case.expected_support);
        score.supported += usize::from(result.disposition == L4CrossSceneDisposition::Supported);
        score.repelled += usize::from(result.disposition == L4CrossSceneDisposition::Repelled);
        score.ambiguous += usize::from(result.disposition == L4CrossSceneDisposition::Ambiguous);
        score.unknown += usize::from(result.disposition == L4CrossSceneDisposition::Unknown);
        let correct = if case.expected_support {
            result.disposition == L4CrossSceneDisposition::Supported
        } else {
            result.disposition != L4CrossSceneDisposition::Supported
        };
        score.correct += usize::from(correct);
        score.false_support += usize::from(
            !case.expected_support && result.disposition == L4CrossSceneDisposition::Supported,
        );
        score.automatic_apply += usize::from(result.recommendation.automatic_apply());
    }
    for score in by_class.values_mut() {
        score.percent = percent(score.correct, score.total);
    }

    let (without_anti, _) = compile_observations(
        &training,
        CrossSceneCompileConfig {
            include_anti_centers: false,
            ..CrossSceneCompileConfig::default()
        },
    );
    let mut shuffled = training.clone();
    for observation in &mut shuffled {
        observation.outcome = match observation.outcome {
            TypingMemoryOutcome::ConfirmedPositive => TypingMemoryOutcome::Reverted,
            TypingMemoryOutcome::ConfirmedNegative | TypingMemoryOutcome::Reverted => {
                TypingMemoryOutcome::ConfirmedPositive
            }
            value => value,
        };
    }
    let (shuffled_sign, _) = compile_observations(&shuffled, CrossSceneCompileConfig::default());
    let ablation = evaluate_ablations(&cases, &restored, &without_anti, &shuffled_sign);
    let hot_readout_latency = measure_hot_readout_latency(&restored, &cases);

    let all_classes_pass = by_class.values().all(|score| score.percent > 95.0);
    let false_automatic_projection = by_class
        .values()
        .map(|score| score.automatic_apply)
        .sum::<usize>();
    let separate_directions_pass = [
        "whole_token_en_to_ru_positive",
        "whole_token_en_to_ru_negative",
        "whole_token_ru_to_en_positive",
        "whole_token_ru_to_en_negative",
    ]
    .iter()
    .all(|class| {
        by_class
            .get(class)
            .is_some_and(|score| score.percent > 95.0)
    });
    let anti_ablation_pass = ablation.anti_prevented_false_supports > 0;
    let hot_readout_latency_pass = hot_readout_latency.pass;
    let verdict = if all_classes_pass
        && separate_directions_pass
        && anti_ablation_pass
        && false_automatic_projection == 0
        && train_overlap == 0
        && deterministic_order
        && candidate_order_parity
        && package_roundtrip
        && runtime_evaluator_parity
        && raw_text_absent
        && hot_readout_latency_pass
    {
        "PASS_SHADOW"
    } else {
        "WATCH"
    };

    Ok(serde_json::json!({
        "kind": "l4_cross_scene_candidate_relative_heldout_proof",
        "verdict": verdict,
        "runtime_authority": "shadow_suggest_only",
        "runtime_authority_changed": false,
        "automatic_apply_possible": false,
        "input": {
            "russian": russian_words,
            "english": english_words,
            "russian_source_eligible_tokens": russian.eligible_tokens,
            "english_source_eligible_tokens": english.eligible_tokens,
            "russian_sampled_words": russian.selected.len(),
            "english_sampled_words": english.selected.len(),
            "russian_train": ru_train.len(),
            "russian_heldout": ru_heldout.len(),
            "english_train": en_train.len(),
            "english_heldout": en_heldout.len(),
            "train_heldout_word_overlap": train_overlap,
        },
        "training_observations": training.len(),
        "heldout_cases": cases.len(),
        "classes": by_class,
        "ablation": ablation,
        "gates": {
            "every_class_strictly_above_95_percent": all_classes_pass,
            "directions_pass_separately": separate_directions_pass,
            "anti_centers_measurably_help": anti_ablation_pass,
            "false_automatic_layout_projection": false_automatic_projection,
            "package_deterministic": deterministic_order,
            "candidate_order_parity": candidate_order_parity,
            "package_roundtrip_exact": package_roundtrip,
            "runtime_evaluator_parity": runtime_evaluator_parity,
            "raw_text_absent": raw_text_absent,
            "hot_readout_p99_under_500_us": hot_readout_latency_pass,
        },
        "hot_readout_latency": hot_readout_latency,
        "package": {
            "path": output,
            "bytes": package_bytes.len(),
            "profiles": restored.profiles.len(),
            "pair_profiles": restored.pair_profiles.len(),
            "encoder_cells": super::CELLS,
            "encoder_version": super::ENCODER_VERSION,
            "encoder_hash": format!("{:016x}", super::ENCODER_HASH),
        },
        "compile": compile_report,
        "not_tested": [
            "automatic edit promotion",
            "application-specific rules",
            "semantic truth outside layout projection",
            "organic negative receipts absent from the current live journal"
        ]
    }))
}

fn build_word_observations(
    russian: &[String],
    english: &[String],
    receipt: &mut u64,
    output: &mut Vec<L4CrossSceneObservation>,
    ambiguous: bool,
) {
    let count = russian.len().min(english.len());
    for index in 0..count {
        let ru_context = context_at(russian, index);
        let en_context = context_at(english, index);
        if let Some(wrong_en) = opposite_layout(&russian[index]) {
            output.push(make_observation(
                &wrong_en,
                &russian[index],
                ru_context,
                LayoutProjectionDirection::EnToRu,
                LayoutProjectionScope::CurrentToken,
                if ambiguous {
                    TypingMemoryOutcome::Ambiguous
                } else {
                    TypingMemoryOutcome::ConfirmedPositive
                },
                next_receipt(receipt),
            ));
            output.push(make_observation(
                &wrong_en,
                &russian[index],
                &negative_control_context(ru_context),
                LayoutProjectionDirection::EnToRu,
                LayoutProjectionScope::CurrentToken,
                TypingMemoryOutcome::Reverted,
                next_receipt(receipt),
            ));
            output.push(make_observation(
                &russian[index],
                &wrong_en,
                context_at(russian, index + 7),
                LayoutProjectionDirection::RuToEn,
                LayoutProjectionScope::CurrentToken,
                TypingMemoryOutcome::Reverted,
                next_receipt(receipt),
            ));
        }
        if let Some(wrong_ru) = opposite_layout(&english[index]) {
            output.push(make_observation(
                &wrong_ru,
                &english[index],
                en_context,
                LayoutProjectionDirection::RuToEn,
                LayoutProjectionScope::CurrentToken,
                if ambiguous {
                    TypingMemoryOutcome::Ambiguous
                } else {
                    TypingMemoryOutcome::ConfirmedPositive
                },
                next_receipt(receipt),
            ));
            output.push(make_observation(
                &wrong_ru,
                &english[index],
                &negative_control_context(en_context),
                LayoutProjectionDirection::RuToEn,
                LayoutProjectionScope::CurrentToken,
                TypingMemoryOutcome::Reverted,
                next_receipt(receipt),
            ));
            output.push(make_observation(
                &english[index],
                &wrong_ru,
                context_at(english, index + 7),
                LayoutProjectionDirection::EnToRu,
                LayoutProjectionScope::CurrentToken,
                TypingMemoryOutcome::Reverted,
                next_receipt(receipt),
            ));
        }
    }
}

fn build_word_cases(
    russian: &[String],
    english: &[String],
    receipt: &mut u64,
    output: &mut Vec<ProofCase>,
) {
    let count = russian.len().min(english.len());
    for index in 0..count {
        if let Some(wrong_en) = opposite_layout(&russian[index]) {
            output.push(ProofCase {
                observation: make_observation(
                    &wrong_en,
                    &russian[index],
                    context_at(russian, index),
                    LayoutProjectionDirection::EnToRu,
                    LayoutProjectionScope::CurrentToken,
                    TypingMemoryOutcome::Censored,
                    next_receipt(receipt),
                ),
                expected_support: true,
                class: "whole_token_en_to_ru_positive",
            });
            output.push(ProofCase {
                observation: make_observation(
                    &wrong_en,
                    &russian[index],
                    &negative_control_context(context_at(russian, index)),
                    LayoutProjectionDirection::EnToRu,
                    LayoutProjectionScope::CurrentToken,
                    TypingMemoryOutcome::Censored,
                    next_receipt(receipt),
                ),
                expected_support: false,
                class: "whole_token_en_to_ru_same_route_negative",
            });
            output.push(ProofCase {
                observation: make_observation(
                    &russian[index],
                    &wrong_en,
                    context_at(russian, index + 5),
                    LayoutProjectionDirection::RuToEn,
                    LayoutProjectionScope::CurrentToken,
                    TypingMemoryOutcome::Censored,
                    next_receipt(receipt),
                ),
                expected_support: false,
                class: "whole_token_ru_to_en_negative",
            });
        }
        if let Some(wrong_ru) = opposite_layout(&english[index]) {
            output.push(ProofCase {
                observation: make_observation(
                    &wrong_ru,
                    &english[index],
                    context_at(english, index),
                    LayoutProjectionDirection::RuToEn,
                    LayoutProjectionScope::CurrentToken,
                    TypingMemoryOutcome::Censored,
                    next_receipt(receipt),
                ),
                expected_support: true,
                class: "whole_token_ru_to_en_positive",
            });
            output.push(ProofCase {
                observation: make_observation(
                    &wrong_ru,
                    &english[index],
                    &negative_control_context(context_at(english, index)),
                    LayoutProjectionDirection::RuToEn,
                    LayoutProjectionScope::CurrentToken,
                    TypingMemoryOutcome::Censored,
                    next_receipt(receipt),
                ),
                expected_support: false,
                class: "whole_token_ru_to_en_same_route_negative",
            });
            output.push(ProofCase {
                observation: make_observation(
                    &english[index],
                    &wrong_ru,
                    context_at(english, index + 5),
                    LayoutProjectionDirection::EnToRu,
                    LayoutProjectionScope::CurrentToken,
                    TypingMemoryOutcome::Censored,
                    next_receipt(receipt),
                ),
                expected_support: false,
                class: "whole_token_en_to_ru_negative",
            });
        }
    }
}

fn grapheme_data(receipt: &mut u64) -> (Vec<L4CrossSceneObservation>, Vec<ProofCase>) {
    let pairs = "abcdefghijklmnopqrstuvwxyz"
        .chars()
        .filter_map(|en| opposite_layout(&en.to_string()).map(|ru| (en.to_string(), ru)))
        .collect::<Vec<_>>();
    let mut training = Vec::new();
    let mut cases = Vec::new();
    let ru_context = ["мы".to_string(), "пишем".to_string(), "слово".to_string()];
    let en_context = ["we".to_string(), "write".to_string(), "word".to_string()];
    for (index, (en, ru)) in pairs.into_iter().enumerate() {
        let train = index % 4 != 0;
        for (from, to, context, direction, expected, class) in [
            (
                en.as_str(),
                ru.as_str(),
                ru_context.as_slice(),
                LayoutProjectionDirection::EnToRu,
                true,
                "grapheme_en_to_ru_positive",
            ),
            (
                en.as_str(),
                ru.as_str(),
                en_context.as_slice(),
                LayoutProjectionDirection::EnToRu,
                false,
                "grapheme_en_to_ru_negative",
            ),
            (
                ru.as_str(),
                en.as_str(),
                en_context.as_slice(),
                LayoutProjectionDirection::RuToEn,
                true,
                "grapheme_ru_to_en_positive",
            ),
            (
                ru.as_str(),
                en.as_str(),
                ru_context.as_slice(),
                LayoutProjectionDirection::RuToEn,
                false,
                "grapheme_ru_to_en_negative",
            ),
        ] {
            let observation = make_observation(
                from,
                to,
                context,
                direction,
                LayoutProjectionScope::Grapheme,
                if train {
                    if expected {
                        TypingMemoryOutcome::ConfirmedPositive
                    } else {
                        TypingMemoryOutcome::Reverted
                    }
                } else {
                    TypingMemoryOutcome::Censored
                },
                next_receipt(receipt),
            );
            if train {
                training.push(observation);
            } else {
                cases.push(ProofCase {
                    observation,
                    expected_support: expected,
                    class,
                });
            }
        }
    }
    (training, cases)
}

fn make_observation(
    from: &str,
    to: &str,
    context: &[String],
    direction: LayoutProjectionDirection,
    scope: LayoutProjectionScope,
    outcome: TypingMemoryOutcome,
    receipt_id: u64,
) -> L4CrossSceneObservation {
    let relation =
        TransitionRelationAtoms::for_operator(from, to, TransitionOperatorKind::LayoutProjection);
    let identity =
        crate::typing_memory::TypingTransitionIdentity::observed(from, to, "replacement");
    let sentence_language = crate::typing_scene::SentenceLanguageEvidence::script_only(context, to);
    L4CrossSceneObservation {
        receipt_id,
        complete_chain: outcome != TypingMemoryOutcome::Censored,
        profile: L4CrossSceneProfileKey::new(
            TransitionOperatorKind::LayoutProjection,
            Some(direction),
            Some(scope),
        )
        .with_scene(identity.scene, sentence_language),
        context: context.to_vec(),
        from_text: from.to_string(),
        to_text: to.to_string(),
        relation_atoms: relation.atoms().to_vec(),
        candidate_relation_id: candidate_relation_id(relation.atoms()),
        keep_relation_id: keep_relation_id(),
        l3_relation_class: relation_class_from_context(context, to),
        context_signal: context_signal_from_text(context, to),
        l2_signal: L4CrossSceneL2Signal::Support,
        sentence_language,
        scene_symbols: identity.scene.known_symbols(),
        outcome,
    }
}

fn evaluate_ablations(
    cases: &[ProofCase],
    full: &super::model::L4CrossScenePackage,
    without_anti: &super::model::L4CrossScenePackage,
    shuffled_sign: &super::model::L4CrossScenePackage,
) -> AblationScore {
    let mut score = AblationScore::default();
    for case in cases {
        let full_readout = readout(full, case.observation.input());
        if case.expected_support {
            score.full_positive_supports +=
                usize::from(full_readout.disposition == L4CrossSceneDisposition::Supported);
            let shuffled = readout(shuffled_sign, case.observation.input());
            score.shuffled_sign_positive_supports +=
                usize::from(shuffled.disposition == L4CrossSceneDisposition::Supported);
            let mut no_context = case.observation.clone();
            no_context.context.clear();
            no_context.context_signal = super::model::L4CrossSceneContextSignal::Unknown;
            no_context.l3_relation_class = relation_class_from_context(&[], &no_context.to_text);
            score.no_context_positive_supports += usize::from(
                readout(full, no_context.input()).disposition == L4CrossSceneDisposition::Supported,
            );
            let mut shuffled_direction = case.observation.clone();
            shuffled_direction.profile.direction = match shuffled_direction.profile.direction {
                Some(LayoutProjectionDirection::EnToRu) => Some(LayoutProjectionDirection::RuToEn),
                Some(LayoutProjectionDirection::RuToEn) => Some(LayoutProjectionDirection::EnToRu),
                value => value,
            };
            score.shuffled_direction_positive_supports += usize::from(
                readout(full, shuffled_direction.input()).disposition
                    == L4CrossSceneDisposition::Supported,
            );
        } else {
            score.negative_cases += 1;
            score.full_false_supports +=
                usize::from(full_readout.disposition == L4CrossSceneDisposition::Supported);
            score.without_anti_false_supports += usize::from(
                readout(without_anti, case.observation.input()).disposition
                    == L4CrossSceneDisposition::Supported,
            );
        }
    }
    score.anti_prevented_false_supports = score
        .without_anti_false_supports
        .saturating_sub(score.full_false_supports);
    score
}

fn measure_hot_readout_latency(
    package: &super::model::L4CrossScenePackage,
    cases: &[ProofCase],
) -> HotReadoutLatency {
    let mut samples = Vec::with_capacity(cases.len().saturating_mul(HOT_READOUT_REPEATS));
    for _ in 0..HOT_READOUT_REPEATS {
        for case in cases {
            let started = Instant::now();
            std::hint::black_box(readout(package, case.observation.input()));
            samples.push(started.elapsed().as_nanos().min(u128::from(u64::MAX)) as u64);
        }
    }
    samples.sort_unstable();
    let p50_ns = percentile(&samples, 50);
    let p99_ns = percentile(&samples, 99);
    let max_ns = samples.last().copied().unwrap_or_default();
    HotReadoutLatency {
        samples: samples.len(),
        repeats: HOT_READOUT_REPEATS,
        p50_us: p50_ns as f64 / 1_000.0,
        p99_us: p99_ns as f64 / 1_000.0,
        max_us: max_ns as f64 / 1_000.0,
        gate_us: HOT_READOUT_P99_LIMIT_NS / 1_000,
        pass: !samples.is_empty() && p99_ns <= HOT_READOUT_P99_LIMIT_NS,
    }
}

fn percentile(sorted: &[u64], percent: usize) -> u64 {
    if sorted.is_empty() {
        return 0;
    }
    let rank = sorted
        .len()
        .saturating_mul(percent.clamp(1, 100))
        .div_ceil(100);
    sorted[rank.saturating_sub(1).min(sorted.len() - 1)]
}

fn load_words(path: &Path, russian: bool) -> io::Result<LoadedWords> {
    let text = fs::read_to_string(path)?;
    let mut words = std::collections::BTreeMap::<(u64, String), ()>::new();
    let mut eligible_tokens = 0_usize;
    for token in text.split_whitespace() {
        let word = token
            .trim_matches(|ch: char| !ch.is_alphabetic())
            .to_lowercase();
        let valid = word.chars().count() >= 4
            && word.chars().all(|ch| {
                if russian {
                    matches!(ch, 'а'..='я' | 'ё')
                } else {
                    ch.is_ascii_alphabetic()
                }
            });
        if valid && opposite_layout(&word).is_some() {
            eligible_tokens = eligible_tokens.saturating_add(1);
            let key = (stable_hash64(word.as_bytes(), 0x4c34_574f_5244_53), word);
            words.insert(key, ());
            if words.len() > MAX_WORDS_PER_LANGUAGE {
                words.pop_last();
            }
        }
    }
    let mut selected = words.into_keys().map(|(_, word)| word).collect::<Vec<_>>();
    selected.sort();
    selected.dedup();
    Ok(LoadedWords {
        selected,
        eligible_tokens,
    })
}

fn split_words(words: &[String]) -> (Vec<String>, Vec<String>) {
    let mut ordered = words.to_vec();
    ordered.sort_by_key(|word| {
        (
            stable_hash64(word.as_bytes(), 0x4c34_5350_4c49_54),
            word.clone(),
        )
    });
    let heldout_count = (ordered.len() / 5).max(16).min(ordered.len() / 2);
    let heldout = ordered.split_off(ordered.len() - heldout_count);
    (ordered, heldout)
}

fn overlap_count(left: &[String], right: &[String]) -> usize {
    let left = left.iter().collect::<BTreeSet<_>>();
    right.iter().filter(|value| left.contains(value)).count()
}

fn context_at(words: &[String], index: usize) -> &[String] {
    if words.len() < 3 {
        return words;
    }
    let start = index % (words.len() - 2);
    &words[start..start + 3]
}

fn negative_control_context(context: &[String]) -> Vec<String> {
    if context.is_empty() {
        return Vec::new();
    }
    context
        .iter()
        .cycle()
        .take(8)
        .map(|token| format!("`{token}`"))
        .collect()
}

fn opposite_layout(text: &str) -> Option<String> {
    let fallback_is_ru = text.chars().any(crate::keyboard::is_cyrillic_letter);
    let events = crate::keyboard::text_to_key_events(text, fallback_is_ru)?;
    let projected = crate::keyboard::map_opposite_events(&events);
    (!projected.is_empty()).then_some(projected)
}

fn next_receipt(receipt: &mut u64) -> u64 {
    let current = *receipt;
    *receipt = receipt.saturating_add(1);
    current
}

fn raw_text_is_absent(bytes: &[u8], russian: &[String], english: &[String]) -> bool {
    russian
        .iter()
        .chain(english)
        .filter(|word| word.len() >= 6)
        .take(32)
        .all(|word| {
            !bytes
                .windows(word.len())
                .any(|window| window == word.as_bytes())
        })
}

fn candidate_permutation_parity(
    package: &super::model::L4CrossScenePackage,
    cases: &[ProofCase],
) -> bool {
    let mut forward = cases
        .iter()
        .map(|case| {
            (
                case.observation.receipt_id,
                readout(package, case.observation.input()),
            )
        })
        .collect::<Vec<_>>();
    let mut reverse = cases
        .iter()
        .rev()
        .map(|case| {
            (
                case.observation.receipt_id,
                readout(package, case.observation.input()),
            )
        })
        .collect::<Vec<_>>();
    forward.sort_by_key(|item| item.0);
    reverse.sort_by_key(|item| item.0);
    forward == reverse
}

fn percent(numerator: usize, denominator: usize) -> f32 {
    if denominator == 0 {
        0.0
    } else {
        numerator as f32 * 100.0 / denominator as f32
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn opposite_layout_roundtrips_both_scripts() {
        assert_eq!(opposite_layout("привет").as_deref(), Some("ghbdtn"));
        assert_eq!(opposite_layout("hello").as_deref(), Some("руддщ"));
    }

    #[test]
    fn split_is_deterministic_and_disjoint() {
        let words = (0..100)
            .map(|index| format!("word{index}"))
            .collect::<Vec<_>>();
        let (train, heldout) = split_words(&words);
        assert_eq!(overlap_count(&train, &heldout), 0);
        assert_eq!(train.len() + heldout.len(), words.len());
    }

    #[test]
    fn same_route_negative_controls_require_signed_evidence() {
        let russian = "абвгдежзий"
            .chars()
            .map(|suffix| format!("слово{suffix}"))
            .collect::<Vec<_>>();
        let english = "abcdefghij"
            .chars()
            .map(|suffix| format!("word{suffix}"))
            .collect::<Vec<_>>();
        let mut receipt = 1;
        let mut training = Vec::new();
        build_word_observations(&russian, &english, &mut receipt, &mut training, false);
        let mut cases = Vec::new();
        build_word_cases(&russian, &english, &mut receipt, &mut cases);
        let (full, _) = compile_observations(&training, CrossSceneCompileConfig::default());
        let (without_anti, _) = compile_observations(
            &training,
            CrossSceneCompileConfig {
                include_anti_centers: false,
                ..CrossSceneCompileConfig::default()
            },
        );
        let score = evaluate_ablations(&cases, &full, &without_anti, &full);
        let positive_cases = cases.iter().filter(|case| case.expected_support).count();

        assert_eq!(score.full_false_supports, 0);
        assert_eq!(score.full_positive_supports, positive_cases);
        assert!(score.anti_prevented_false_supports > 0);
    }
}
