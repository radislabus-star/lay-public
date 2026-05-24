use super::candidates::rank_candidates;
use super::cases::Case;
use lay::lem::ScoredCandidate;

pub(crate) struct ProbeReport {
    cases: usize,
    ok: usize,
    by_kind: Vec<KindStats>,
    failures: Vec<Failure>,
    median_margin: f64,
    p10_margin: f64,
}

struct KindStats {
    kind: &'static str,
    passed: usize,
    total: usize,
}

struct Failure {
    case: Case,
    ranked: Vec<ScoredCandidate>,
}

pub(crate) fn run_probe(cases: &[Case]) -> ProbeReport {
    let mut ok = 0usize;
    let mut by_kind = Vec::new();
    let mut failures = Vec::new();
    let mut margins = Vec::new();

    for case in cases {
        let ranked = rank_candidates(case);
        let best = &ranked[0];
        let second = &ranked[1];
        let passed = best.text == case.expected;
        if passed {
            ok += 1;
            margins.push(best.total - second.total);
        } else if failures.len() < 20 {
            failures.push(Failure {
                case: case.clone(),
                ranked: ranked[..ranked.len().min(4)].to_vec(),
            });
        }

        record_kind_stat(&mut by_kind, case.kind, passed);
    }

    margins.sort_by(f64::total_cmp);
    ProbeReport {
        cases: cases.len(),
        ok,
        by_kind,
        failures,
        median_margin: percentile(&margins, 0.50),
        p10_margin: percentile(&margins, 0.10),
    }
}

pub(crate) fn print_report(report: &ProbeReport) {
    let accuracy = report.ok as f64 * 100.0 / report.cases as f64;
    println!("LEM research probe");
    println!("cases: {}", report.cases);
    println!("passed: {}/{} ({accuracy:.1}%)", report.ok, report.cases);
    println!("median winning margin: {:.3}", report.median_margin);
    println!("p10 winning margin: {:.3}", report.p10_margin);
    println!();
    println!("by kind:");
    for stats in &report.by_kind {
        let pct = stats.passed as f64 * 100.0 / stats.total as f64;
        println!(
            "  {:24} {:4}/{:<4} {:5.1}%",
            stats.kind, stats.passed, stats.total, pct
        );
    }

    if !report.failures.is_empty() {
        println!();
        println!("first failures:");
        for failure in &report.failures {
            print_failure(failure);
        }
    }
}

fn record_kind_stat(stats: &mut Vec<KindStats>, kind: &'static str, passed: bool) {
    match stats.iter_mut().find(|item| item.kind == kind) {
        Some(item) => {
            item.passed += usize::from(passed);
            item.total += 1;
        }
        None => stats.push(KindStats {
            kind,
            passed: usize::from(passed),
            total: 1,
        }),
    }
}

fn print_failure(failure: &Failure) {
    println!(
        "  kind={} typed={:?} expected={:?}",
        failure.case.kind, failure.case.typed, failure.case.expected
    );
    for candidate in &failure.ranked {
        println!(
            "    {:>8.3} lang={:>7.3} noise={:>5.2} edit={:>5.2} int={:>5.2} {:?}",
            candidate.total,
            candidate.language,
            candidate.noise,
            candidate.edit,
            candidate.intervention,
            candidate.text
        );
    }
}

fn percentile(values: &[f64], p: f64) -> f64 {
    if values.is_empty() {
        return 0.0;
    }
    let idx = ((values.len() - 1) as f64 * p).round() as usize;
    values[idx]
}
