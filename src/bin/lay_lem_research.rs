//! Research-only probe for LEM (Layout Error Metric).
//!
//! This binary does not participate in the daemon path. It builds synthetic
//! candidate sets and checks whether a scoring function can choose the intended
//! text from noisy layout/typo variants.

#[path = "lay_lem_research/candidates.rs"]
mod candidates;
#[path = "lay_lem_research/cases.rs"]
mod cases;
#[path = "lay_lem_research/report.rs"]
mod report;

use cases::build_cases;
use report::{print_report, run_probe};

const TARGET_CASES: usize = 12_000;

fn main() {
    let cases = build_cases(TARGET_CASES);
    let report = run_probe(&cases);
    print_report(&report);
}
