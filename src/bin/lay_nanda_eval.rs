use lay::config::CorrectionSafety;
use lay::microbrain::{default_expert_names, default_writer_names, MicrobrainOptions};
use lay::nanda_eval::{
    evaluate, evaluate_report, read_cases, render_eval_report, EvalCase, EvalResult, EvalStats,
};
use std::env;
use std::io;
use std::path::PathBuf;

#[derive(Debug, Clone)]
struct EnsembleMode {
    name: &'static str,
    cells: &'static [&'static str],
    reasons: &'static [&'static str],
}

fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();
    let case_paths = arg_values(&args, "--cases")
        .into_iter()
        .map(PathBuf::from)
        .collect::<Vec<_>>();
    let case_paths = if case_paths.is_empty() {
        vec![PathBuf::from("data/neural_arbiter/holdout.tsv")]
    } else {
        case_paths
    };
    let disabled = arg_values(&args, "--disable-expert");
    let safety = arg_value(&args, "--safety")
        .map(parse_safety)
        .unwrap_or(CorrectionSafety::Experimental);
    let show_changes = args.iter().any(|arg| arg == "--show-changes");
    let ensemble_sweep = args.iter().any(|arg| arg == "--ensemble-sweep");
    let compare_64x3 = args.iter().any(|arg| arg == "--compare-64x3-vs-192");

    if compare_64x3 {
        print_synthetic_64x3_vs_192();
        return Ok(());
    }

    let mut cases = Vec::new();
    for path in &case_paths {
        cases.extend(read_cases(path)?);
    }

    let options = MicrobrainOptions::with_disabled(&disabled);
    let report = evaluate_report(&cases, safety, &options);
    print!("{}", render_eval_report(&report, false));
    if show_changes {
        print_changed_cases(&cases, &report.baseline, &report.nanda);
    }

    if disabled.is_empty() {
        print_ablation(&cases, safety, show_changes);
    }
    if ensemble_sweep {
        print_ensemble_sweep(&cases, safety);
    }

    Ok(())
}

fn print_ablation(cases: &[EvalCase], safety: CorrectionSafety, show_changes: bool) {
    let default = evaluate(cases, safety, Some(&MicrobrainOptions::default()));
    let default_ok = summarize(&default).ok;
    println!("ablation:");
    for expert in default_expert_names() {
        let disabled = vec![expert.to_string()];
        let result = evaluate(
            cases,
            safety,
            Some(&MicrobrainOptions::with_disabled(&disabled)),
        );
        let ok = summarize(&result).ok;
        let delta = ok as isize - default_ok as isize;
        let changed = default
            .iter()
            .zip(&result)
            .filter(|(left, right)| left.output != right.output)
            .count();
        println!(
            "  without {expert}: ok={ok}/{} delta={delta:+} changed={changed}",
            cases.len()
        );
        if show_changes && changed > 0 {
            print_changed_cases(cases, &default, &result);
        }
    }
}

fn print_changed_cases(cases: &[EvalCase], left: &[EvalResult], right: &[EvalResult]) {
    for ((idx, case), (left, right)) in cases.iter().enumerate().zip(left.iter().zip(right)) {
        if left.output == right.output && left.ok == right.ok {
            continue;
        }
        println!(
            "  case#{idx} reason={} original={:?} expected={:?} left={:?} right={:?} left_ok={} right_ok={}",
            case.reason, case.original, case.expected, left.output, right.output, left.ok, right.ok
        );
    }
}

fn print_ensemble_sweep(cases: &[EvalCase], safety: CorrectionSafety) {
    println!("ensemble_sweep:");
    for mode in ensemble_modes() {
        let mode_cases: Vec<EvalCase> = cases
            .iter()
            .filter(|case| mode.reasons.iter().any(|reason| *reason == case.reason))
            .cloned()
            .collect();
        if mode_cases.is_empty() {
            println!("  mode {}: no cases", mode.name);
            continue;
        }
        let non_mode_cases: Vec<EvalCase> = cases
            .iter()
            .filter(|case| !mode.reasons.iter().any(|reason| *reason == case.reason))
            .cloned()
            .collect();
        print_ensemble_mode(mode, &mode_cases, &non_mode_cases, safety);
    }
}

fn print_ensemble_mode(
    mode: EnsembleMode,
    cases: &[EvalCase],
    non_mode_cases: &[EvalCase],
    safety: CorrectionSafety,
) {
    println!("  mode {}", mode.name);
    println!("    cells: {}", mode.cells.join(" + "));
    println!("    classes: {}", mode.reasons.join(", "));

    let full = evaluate_enabled(cases, safety, mode.cells);
    let full_score = score_results(&full);
    let mut best_single = 0.0f64;
    let mut best_pair = 0.0f64;

    for cell in mode.cells {
        let single = evaluate_enabled(cases, safety, &[*cell]);
        let score = score_results(&single);
        best_single = best_single.max(score);
        println!(
            "    single {cell}: {}/{} {:.1}%",
            ok_count(&single),
            cases.len(),
            score
        );
    }

    for pair in cell_pairs(mode.cells) {
        let pair_refs: Vec<&str> = pair.iter().map(|cell| cell.as_str()).collect();
        let pair_result = evaluate_enabled(cases, safety, &pair_refs);
        let score = score_results(&pair_result);
        best_pair = best_pair.max(score);
        println!(
            "    pair {}: {}/{} {:.1}%",
            pair.join("+"),
            ok_count(&pair_result),
            cases.len(),
            score
        );
    }

    println!(
        "    ensemble: {}/{} {:.1}%",
        ok_count(&full),
        cases.len(),
        full_score
    );
    println!(
        "    synergy_vs_best_single: {:+.1} pp",
        full_score - best_single
    );
    println!(
        "    synergy_vs_best_pair: {:+.1} pp",
        full_score - best_pair
    );
    let mut largest_drop = 0.0f64;
    println!("    ablation_drop:");
    for cell in mode.cells {
        let without: Vec<&str> = mode
            .cells
            .iter()
            .copied()
            .filter(|candidate| candidate != cell)
            .collect();
        let result = evaluate_enabled(cases, safety, &without);
        let score = score_results(&result);
        let drop = score - full_score;
        largest_drop = largest_drop.min(drop);
        println!("      without {cell}: {drop:+.1} pp");
    }
    let false_positive_delta = false_positive_delta(non_mode_cases, safety, mode.cells);
    let stability = split_stability(cases, safety, mode.cells);
    let locality = locality_status(full_score - best_single, false_positive_delta);
    println!("    false_positive_delta: {false_positive_delta:+.1} pp");
    println!("    stability: {stability:.2}");
    println!("    locality: {locality}");
    println!(
        "    mode_status: {}",
        mode_status(
            full_score - best_single,
            largest_drop,
            false_positive_delta,
            stability,
            locality
        )
    );
    print_representative_trajectory(cases, &full);
}

fn false_positive_delta(cases: &[EvalCase], safety: CorrectionSafety, enabled: &[&str]) -> f64 {
    if cases.is_empty() {
        return 0.0;
    }
    let baseline = evaluate(cases, safety, None);
    let ensemble = evaluate_enabled(cases, safety, enabled);
    let false_positives = baseline
        .iter()
        .zip(&ensemble)
        .filter(|(base, full)| base.ok && !full.ok)
        .count();
    percent(false_positives, cases.len())
}

fn split_stability(cases: &[EvalCase], safety: CorrectionSafety, enabled: &[&str]) -> f64 {
    if cases.len() < 6 {
        return 1.0;
    }
    let mut stable = 0usize;
    let mut shards = 0usize;
    for shard_idx in 0..3 {
        let shard: Vec<EvalCase> = cases
            .iter()
            .enumerate()
            .filter(|(idx, _case)| idx % 3 == shard_idx)
            .map(|(_idx, case)| case.clone())
            .collect();
        if shard.is_empty() {
            continue;
        }
        shards += 1;
        let full = evaluate_enabled(&shard, safety, enabled);
        let full_score = score_results(&full);
        let best_single = enabled
            .iter()
            .map(|cell| score_results(&evaluate_enabled(&shard, safety, &[*cell])))
            .fold(0.0f64, f64::max);
        if full_score >= best_single {
            stable += 1;
        }
    }
    stable as f64 / shards.max(1) as f64
}

fn locality_status(synergy_vs_best_single: f64, false_positive_delta: f64) -> &'static str {
    if false_positive_delta > 1.0 {
        "failed_false_positives"
    } else if synergy_vs_best_single >= 5.0 {
        "confirmed"
    } else {
        "not_proven"
    }
}

fn mode_status(
    synergy_vs_best_single: f64,
    largest_ablation_drop: f64,
    false_positive_delta: f64,
    stability: f64,
    locality: &str,
) -> &'static str {
    if false_positive_delta > 1.0 {
        "rejected_false_positives"
    } else if synergy_vs_best_single >= 5.0
        && largest_ablation_drop <= -5.0
        && stability >= 0.67
        && locality == "confirmed"
    {
        "ensemble_mode_found"
    } else if synergy_vs_best_single <= 0.0 && largest_ablation_drop >= 0.0 {
        "too_easy_or_redundant"
    } else if stability < 0.67 {
        "unstable"
    } else {
        "inconclusive"
    }
}

fn evaluate_enabled(
    cases: &[EvalCase],
    safety: CorrectionSafety,
    enabled: &[&str],
) -> Vec<EvalResult> {
    let enabled = enabled
        .iter()
        .map(|cell| cell.to_string())
        .collect::<Vec<_>>();
    evaluate(
        cases,
        safety,
        Some(&MicrobrainOptions::with_enabled_only(&enabled)),
    )
}

fn print_representative_trajectory(cases: &[EvalCase], results: &[EvalResult]) {
    let Some((case, result)) = cases.iter().zip(results).find(|(_case, result)| {
        result
            .trace
            .as_ref()
            .is_some_and(|trace| !trace.candidates.is_empty())
    }) else {
        return;
    };
    let Some(trace) = &result.trace else {
        return;
    };
    let Some(candidate) = trace
        .candidates
        .iter()
        .find(|candidate| trace.chosen.as_deref() == Some(candidate.candidate.as_str()))
        .or_else(|| trace.candidates.first())
    else {
        return;
    };
    println!(
        "    trajectory original={:?} expected={:?} chosen={:?}",
        case.original, case.expected, trace.chosen
    );
    for tick in &candidate.mesh_ticks {
        println!(
            "      tick {}: layout={:.2} space={:.2} technical={:.2} undo={:.2} coherence={:.2} confidence={:.2} reason={}",
            tick.tick,
            tick.board.layout_energy,
            tick.board.space_energy,
            tick.board.technical_risk,
            tick.board.undo_risk,
            tick.board.coherence,
            tick.confidence,
            tick.reason_code
        );
    }
}

fn ensemble_modes() -> Vec<EnsembleMode> {
    vec![
        EnsembleMode {
            name: "safe_layout_mode",
            cells: &[
                "layout_writer_64k_stub",
                "layout_signal_16k_stub",
                "layout_signal_64k_trained",
                "context_tail_32k_stub",
                "sentence_mesh_64k_stub",
            ],
            reasons: &["ru_layout_to_technical_en", "en_layout_to_ru", "layout"],
        },
        EnsembleMode {
            name: "space_repair_mode",
            cells: &[
                "context_tail_32k_stub",
                "user_memory_64k_stub",
                "sentence_mesh_64k_stub",
            ],
            reasons: &["split_glued_phrase"],
        },
        EnsembleMode {
            name: "technical_protect_mode",
            cells: &[
                "protected_token_16k_stub",
                "cli_guard_16k_stub",
                "context_tail_32k_stub",
                "sentence_mesh_64k_stub",
            ],
            reasons: &["technical_keep", "keep_russian_phrase", "keep"],
        },
        EnsembleMode {
            name: "typo_repair_mode",
            cells: &[
                "context_tail_32k_stub",
                "user_memory_64k_stub",
                "layout_signal_64k_trained",
                "sentence_mesh_64k_stub",
            ],
            reasons: &["ru_typo"],
        },
        EnsembleMode {
            name: "mixed_context_mode",
            cells: &[
                "layout_writer_64k_stub",
                "layout_signal_16k_stub",
                "context_tail_32k_stub",
                "sentence_mesh_64k_stub",
            ],
            reasons: &["mixed_context"],
        },
    ]
}

fn cell_pairs(cells: &[&str]) -> Vec<Vec<String>> {
    let mut pairs = Vec::new();
    for left in 0..cells.len() {
        for right in left + 1..cells.len() {
            pairs.push(vec![cells[left].to_string(), cells[right].to_string()]);
        }
    }
    pairs
}

fn score_results(results: &[EvalResult]) -> f64 {
    percent(ok_count(results), results.len())
}

fn ok_count(results: &[EvalResult]) -> usize {
    results.iter().filter(|result| result.ok).count()
}

fn summarize(results: &[EvalResult]) -> EvalStats {
    EvalStats {
        cases: results.len(),
        ok: results.iter().filter(|result| result.ok).count(),
        ..EvalStats::default()
    }
}

#[allow(dead_code)]
fn known_microbrain_cells() -> Vec<&'static str> {
    default_expert_names()
        .into_iter()
        .chain(default_writer_names())
        .collect()
}

fn parse_safety(value: &str) -> CorrectionSafety {
    match value {
        "strict" => CorrectionSafety::Strict,
        "normal" => CorrectionSafety::Normal,
        "experimental" => CorrectionSafety::Experimental,
        _ => CorrectionSafety::Experimental,
    }
}

fn arg_value<'a>(args: &'a [String], name: &str) -> Option<&'a str> {
    args.windows(2)
        .find(|window| window[0] == name)
        .map(|window| window[1].as_str())
}

fn arg_values(args: &[String], name: &str) -> Vec<String> {
    args.windows(2)
        .filter(|window| window[0] == name)
        .map(|window| window[1].clone())
        .collect()
}

fn percent(num: usize, den: usize) -> f64 {
    if den == 0 {
        0.0
    } else {
        num as f64 * 100.0 / den as f64
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ToyClass {
    SafeLayout,
    CliVeto,
    TechnicalVeto,
    SpaceRepair,
    PrefixKeep,
    MixedContext,
    NormalKeep,
}

#[derive(Debug, Clone)]
struct ToyCase {
    class: ToyClass,
    layout_signal: bool,
    guard_risk: bool,
    space_signal: bool,
    prefix_risk: bool,
    mixed_context: bool,
    apply: bool,
}

#[derive(Debug, Clone)]
struct ToyLinearCell {
    weights: Vec<i16>,
    features: Vec<usize>,
}

#[derive(Debug, Clone, Copy)]
struct ToyStats {
    cases: usize,
    ok: usize,
    false_positive: usize,
    false_negative: usize,
}

fn print_synthetic_64x3_vs_192() {
    let train = synthetic_train_cases();
    let test = synthetic_test_cases();
    let hard = synthetic_hard_cases();

    let layout_64 = train_cell_by(&train, &[0, 1, 5], 64 * 1024, 12, |case| case.layout_signal);
    let guard_64 = train_cell_by(&train, &[0, 2, 4, 5], 64 * 1024, 12, |case| {
        !case.guard_risk && !case.prefix_risk
    });
    let context_64 = train_cell_by(&train, &[0, 3, 5], 64 * 1024, 12, |case| case.space_signal);
    let mono_192 = train_cell_by(&train, &[0, 1, 2, 3, 4, 5], 192 * 1024, 12, |case| {
        case.apply
    });

    println!("synthetic_64x3_vs_192:");
    println!("  goal: compositional control, not production proof");
    println!("  train_cases: {}", train.len());
    println!("  test_cases: {}", test.len());
    println!("  hard_cases: {}", hard.len());
    println!("  ensemble_budget: {} KB", 64 * 3);
    println!("  monolith_budget: 192 KB");
    println!("  cells: layout_64 + guard_64 + context_64");

    let ensemble_test = eval_toy(&test, |case| {
        predict_64x3(case, &layout_64, &guard_64, &context_64)
    });
    let mono_test = eval_toy(&test, |case| predict_cell(case, &mono_192));
    let ensemble_hard = eval_toy(&hard, |case| {
        predict_64x3(case, &layout_64, &guard_64, &context_64)
    });
    let mono_hard = eval_toy(&hard, |case| predict_cell(case, &mono_192));

    print_toy_stats("  test 64x3_mesh", ensemble_test);
    print_toy_stats("  test mono_192", mono_test);
    print_toy_stats("  hard 64x3_mesh", ensemble_hard);
    print_toy_stats("  hard mono_192", mono_hard);
    print_toy_by_class("  hard_by_class 64x3_mesh", &hard, |case| {
        predict_64x3(case, &layout_64, &guard_64, &context_64)
    });
    print_toy_by_class("  hard_by_class mono_192", &hard, |case| {
        predict_cell(case, &mono_192)
    });

    let hard_gain =
        percent(ensemble_hard.ok, ensemble_hard.cases) - percent(mono_hard.ok, mono_hard.cases);
    println!("  hard_gain_64x3_vs_192: {hard_gain:+.1} pp");

    let without_layout = eval_toy(&hard, |case| {
        predict_64x3_without(case, None, Some(&guard_64), Some(&context_64))
    });
    let without_guard = eval_toy(&hard, |case| {
        predict_64x3_without(case, Some(&layout_64), None, Some(&context_64))
    });
    let without_context = eval_toy(&hard, |case| {
        predict_64x3_without(case, Some(&layout_64), Some(&guard_64), None)
    });
    println!("  hard_ablation:");
    println!(
        "    without layout_64: {:+.1} pp",
        percent(without_layout.ok, without_layout.cases)
            - percent(ensemble_hard.ok, ensemble_hard.cases)
    );
    println!(
        "    without guard_64: {:+.1} pp",
        percent(without_guard.ok, without_guard.cases)
            - percent(ensemble_hard.ok, ensemble_hard.cases)
    );
    println!(
        "    without context_64: {:+.1} pp",
        percent(without_context.ok, without_context.cases)
            - percent(ensemble_hard.ok, ensemble_hard.cases)
    );

    let mode_found = hard_gain >= 5.0
        && ensemble_hard.false_positive <= mono_hard.false_positive
        && without_guard.ok < ensemble_hard.ok;
    println!(
        "  synthetic_mode_status: {}",
        if mode_found {
            "64x3_compositional_mode_found"
        } else {
            "not_proven"
        }
    );
}

fn print_toy_stats(name: &str, stats: ToyStats) {
    println!(
        "{name}: ok={}/{} {:.1}% fp={} fn={}",
        stats.ok,
        stats.cases,
        percent(stats.ok, stats.cases),
        stats.false_positive,
        stats.false_negative
    );
}

fn eval_toy(cases: &[ToyCase], mut predict: impl FnMut(&ToyCase) -> bool) -> ToyStats {
    let mut stats = ToyStats {
        cases: cases.len(),
        ok: 0,
        false_positive: 0,
        false_negative: 0,
    };
    for case in cases {
        let predicted = predict(case);
        stats.ok += usize::from(predicted == case.apply);
        stats.false_positive += usize::from(predicted && !case.apply);
        stats.false_negative += usize::from(!predicted && case.apply);
    }
    stats
}

fn print_toy_by_class(name: &str, cases: &[ToyCase], mut predict: impl FnMut(&ToyCase) -> bool) {
    println!("{name}:");
    for class in [
        ToyClass::SafeLayout,
        ToyClass::CliVeto,
        ToyClass::TechnicalVeto,
        ToyClass::SpaceRepair,
        ToyClass::PrefixKeep,
        ToyClass::MixedContext,
        ToyClass::NormalKeep,
    ] {
        let class_cases: Vec<ToyCase> = cases
            .iter()
            .filter(|case| case.class == class)
            .cloned()
            .collect();
        if class_cases.is_empty() {
            continue;
        }
        let stats = eval_toy(&class_cases, |case| predict(case));
        println!(
            "    {}: {}/{} {:.1}% fp={} fn={}",
            toy_class_name(class),
            stats.ok,
            stats.cases,
            percent(stats.ok, stats.cases),
            stats.false_positive,
            stats.false_negative
        );
    }
}

fn toy_class_name(class: ToyClass) -> &'static str {
    match class {
        ToyClass::SafeLayout => "safe_layout",
        ToyClass::CliVeto => "cli_veto",
        ToyClass::TechnicalVeto => "technical_veto",
        ToyClass::SpaceRepair => "space_repair",
        ToyClass::PrefixKeep => "prefix_keep",
        ToyClass::MixedContext => "mixed_context",
        ToyClass::NormalKeep => "normal_keep",
    }
}

fn train_cell_by(
    cases: &[ToyCase],
    features: &[usize],
    budget_bytes: usize,
    epochs: usize,
    label: impl Fn(&ToyCase) -> bool,
) -> ToyLinearCell {
    let mut cell = ToyLinearCell {
        weights: vec![0; budget_bytes],
        features: features.to_vec(),
    };
    for _ in 0..epochs {
        for case in cases {
            let predicted = predict_cell(case, &cell);
            let expected = label(case);
            if predicted == expected {
                continue;
            }
            let delta = if expected { 1 } else { -1 };
            update_toy_cell(case, &mut cell, delta);
        }
    }
    cell
}

fn update_toy_cell(case: &ToyCase, cell: &mut ToyLinearCell, delta: i16) {
    for feature in toy_features(case, &cell.features) {
        let idx = stable_hash(feature.as_bytes()) % cell.weights.len();
        cell.weights[idx] = (cell.weights[idx] + delta).clamp(-127, 127);
    }
}

fn predict_cell(case: &ToyCase, cell: &ToyLinearCell) -> bool {
    let sum: i32 = toy_features(case, &cell.features)
        .into_iter()
        .map(|feature| {
            let idx = stable_hash(feature.as_bytes()) % cell.weights.len();
            cell.weights[idx] as i32
        })
        .sum();
    sum > 0
}

fn predict_64x3(
    case: &ToyCase,
    layout: &ToyLinearCell,
    guard: &ToyLinearCell,
    context: &ToyLinearCell,
) -> bool {
    predict_64x3_without(case, Some(layout), Some(guard), Some(context))
}

fn predict_64x3_without(
    case: &ToyCase,
    layout: Option<&ToyLinearCell>,
    guard: Option<&ToyLinearCell>,
    context: Option<&ToyLinearCell>,
) -> bool {
    let layout_apply = layout.is_some_and(|cell| predict_cell(case, cell));
    let guard_keep = guard.is_some_and(|cell| !predict_cell(case, cell));
    let context_apply = context.is_some_and(|cell| predict_cell(case, cell));

    if guard_keep {
        return false;
    }
    layout_apply || context_apply
}

fn toy_features(case: &ToyCase, selected: &[usize]) -> Vec<&'static str> {
    let all = [
        "bias",
        if case.layout_signal {
            "layout:high"
        } else {
            "layout:low"
        },
        if case.guard_risk {
            "guard:risk"
        } else {
            "guard:clear"
        },
        if case.space_signal {
            "space:repair"
        } else {
            "space:normal"
        },
        if case.prefix_risk {
            "prefix:risk"
        } else {
            "prefix:clear"
        },
        if case.mixed_context {
            "context:mixed"
        } else {
            "context:plain"
        },
    ];
    selected
        .iter()
        .filter_map(|idx| all.get(*idx).copied())
        .collect()
}

fn stable_hash(bytes: &[u8]) -> usize {
    let mut hash = 0xcbf2_9ce4_8422_2325u64;
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x1000_0000_01b3);
    }
    hash as usize
}

fn synthetic_train_cases() -> Vec<ToyCase> {
    let mut cases = Vec::new();
    for _ in 0..12 {
        cases.push(toy_case(
            ToyClass::SafeLayout,
            true,
            false,
            false,
            false,
            false,
            true,
        ));
        cases.push(toy_case(
            ToyClass::SpaceRepair,
            false,
            false,
            true,
            false,
            false,
            true,
        ));
        cases.push(toy_case(
            ToyClass::NormalKeep,
            false,
            false,
            false,
            false,
            false,
            false,
        ));
        cases.push(toy_case(
            ToyClass::CliVeto,
            false,
            true,
            false,
            false,
            false,
            false,
        ));
        cases.push(toy_case(
            ToyClass::TechnicalVeto,
            false,
            true,
            false,
            false,
            true,
            false,
        ));
        cases.push(toy_case(
            ToyClass::PrefixKeep,
            false,
            false,
            false,
            true,
            false,
            false,
        ));
    }
    cases
}

fn synthetic_test_cases() -> Vec<ToyCase> {
    let mut cases = synthetic_train_cases();
    cases.extend([
        toy_case(
            ToyClass::MixedContext,
            true,
            false,
            false,
            false,
            true,
            true,
        ),
        toy_case(
            ToyClass::MixedContext,
            false,
            false,
            true,
            false,
            true,
            true,
        ),
        toy_case(
            ToyClass::TechnicalVeto,
            false,
            true,
            true,
            false,
            true,
            false,
        ),
        toy_case(ToyClass::PrefixKeep, true, false, false, true, false, false),
    ]);
    cases
}

fn synthetic_hard_cases() -> Vec<ToyCase> {
    vec![
        toy_case(ToyClass::CliVeto, true, true, false, false, false, false),
        toy_case(
            ToyClass::TechnicalVeto,
            true,
            true,
            false,
            false,
            true,
            false,
        ),
        toy_case(
            ToyClass::TechnicalVeto,
            false,
            true,
            true,
            false,
            true,
            false,
        ),
        toy_case(ToyClass::PrefixKeep, true, false, false, true, false, false),
        toy_case(ToyClass::PrefixKeep, false, false, true, true, false, false),
        toy_case(
            ToyClass::MixedContext,
            true,
            false,
            false,
            false,
            true,
            true,
        ),
        toy_case(
            ToyClass::MixedContext,
            false,
            false,
            true,
            false,
            true,
            true,
        ),
        toy_case(ToyClass::SafeLayout, true, false, false, false, false, true),
        toy_case(
            ToyClass::SpaceRepair,
            false,
            false,
            true,
            false,
            false,
            true,
        ),
    ]
}

fn toy_case(
    class: ToyClass,
    layout_signal: bool,
    guard_risk: bool,
    space_signal: bool,
    prefix_risk: bool,
    mixed_context: bool,
    apply: bool,
) -> ToyCase {
    ToyCase {
        class,
        layout_signal,
        guard_risk,
        space_signal,
        prefix_risk,
        mixed_context,
        apply,
    }
}
