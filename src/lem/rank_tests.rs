use super::*;

#[test]
fn language_weight_changes_scored_totals() {
    let normal = rank_candidates("ghbdtn", ["ghbdtn".to_string(), "привет".to_string()]);
    let muted = rank_candidates_with_language_weight(
        "ghbdtn",
        ["ghbdtn".to_string(), "привет".to_string()],
        0.2,
    );

    assert_ne!(normal[0].total, muted[0].total);
}
