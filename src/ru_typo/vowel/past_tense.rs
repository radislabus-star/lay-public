pub(super) fn has_same_simple_past_tense_tail(left: &str, right: &str) -> bool {
    PAST_TENSE_TAILS
        .iter()
        .any(|tail| left.ends_with(tail) && right.ends_with(tail))
}

const PAST_TENSE_TAILS: &[&str] = &[
    "ился",
    "илась",
    "ились",
    "илось",
    "ался",
    "алась",
    "ались",
    "алось",
    "ила",
    "или",
    "ило",
    "ил",
    "ала",
    "али",
    "ало",
    "ал",
];
