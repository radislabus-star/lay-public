use super::super::atoms::normalize_lexical_surface;

pub(super) const MAX_ANCHOR_SEQUENCE: usize = 32;
pub(in crate::nanda_wave::lexical_grokking) const RECONSTRUCTION_MODE_DELETION: u8 = 1;
pub(in crate::nanda_wave::lexical_grokking) const RECONSTRUCTION_MODE_DELETION_TRANSPOSITION: u8 =
    2;
pub(in crate::nanda_wave::lexical_grokking) const RECONSTRUCTION_MODE_SUFFIX_TRUNCATION: u8 = 4;
pub(in crate::nanda_wave::lexical_grokking) const RECONSTRUCTION_MODE_PREFIX_TRUNCATION: u8 = 8;
pub(in crate::nanda_wave::lexical_grokking) const RECONSTRUCTION_MODE_SINGLE_DELETION: u8 = 16;
pub(in crate::nanda_wave::lexical_grokking) const RECONSTRUCTION_MODE_SINGLE_SUBSTITUTION: u8 = 32;
pub(in crate::nanda_wave::lexical_grokking) const RECONSTRUCTION_MODE_DOUBLE_SUBSTITUTION: u8 = 64;
pub(in crate::nanda_wave::lexical_grokking) const RECONSTRUCTION_MODE_NON_ADJACENT_TRANSPOSITION:
    u8 = 128;

pub(in crate::nanda_wave::lexical_grokking) fn damerau_distance(
    left: &[u32],
    right: &[u32],
) -> usize {
    const STACK_WIDTH: usize = MAX_ANCHOR_SEQUENCE + 1;
    if right.len() >= STACK_WIDTH {
        return damerau_distance_heap(left, right);
    }
    let mut previous_previous = [0_usize; STACK_WIDTH];
    let mut previous = [0_usize; STACK_WIDTH];
    let mut current = [0_usize; STACK_WIDTH];
    for (column, slot) in previous.iter_mut().take(right.len() + 1).enumerate() {
        *slot = column;
    }
    for row in 1..=left.len() {
        current[0] = row;
        for column in 1..=right.len() {
            let substitution = usize::from(left[row - 1] != right[column - 1]);
            let mut distance = previous[column]
                .saturating_add(1)
                .min(current[column - 1].saturating_add(1))
                .min(previous[column - 1].saturating_add(substitution));
            if row > 1
                && column > 1
                && left[row - 1] == right[column - 2]
                && left[row - 2] == right[column - 1]
            {
                distance = distance.min(previous_previous[column - 2].saturating_add(1));
            }
            current[column] = distance;
        }
        std::mem::swap(&mut previous_previous, &mut previous);
        std::mem::swap(&mut previous, &mut current);
    }
    previous[right.len()]
}

fn damerau_distance_heap(left: &[u32], right: &[u32]) -> usize {
    let width = right.len() + 1;
    let mut matrix = vec![0_usize; (left.len() + 1) * width];
    for row in 0..=left.len() {
        matrix[row * width] = row;
    }
    for (column, slot) in matrix.iter_mut().take(width).enumerate() {
        *slot = column;
    }
    for row in 1..=left.len() {
        for column in 1..=right.len() {
            let substitution = usize::from(left[row - 1] != right[column - 1]);
            let mut distance = matrix[(row - 1) * width + column]
                .saturating_add(1)
                .min(matrix[row * width + column - 1].saturating_add(1))
                .min(matrix[(row - 1) * width + column - 1].saturating_add(substitution));
            if row > 1
                && column > 1
                && left[row - 1] == right[column - 2]
                && left[row - 2] == right[column - 1]
            {
                distance = distance.min(matrix[(row - 2) * width + column - 2].saturating_add(1));
            }
            matrix[row * width + column] = distance;
        }
    }
    matrix[left.len() * width + right.len()]
}

pub(in crate::nanda_wave::lexical_grokking) fn reconstruction_modes(
    observed: &[u32],
    expected: &[u32],
) -> u8 {
    let missing = expected.len().saturating_sub(observed.len());
    if !(1..=2).contains(&missing) {
        return 0;
    }
    let mut modes = 0;
    let ordered_subsequence = is_ordered_subsequence(observed, expected);
    if missing == 1 && ordered_subsequence {
        if observed == &expected[..expected.len() - 1] {
            modes |= RECONSTRUCTION_MODE_SUFFIX_TRUNCATION;
        } else if observed == &expected[1..] {
            modes |= RECONSTRUCTION_MODE_PREFIX_TRUNCATION;
        } else {
            modes |= RECONSTRUCTION_MODE_SINGLE_DELETION;
        }
    }
    if missing == 2 && ordered_subsequence {
        modes |= RECONSTRUCTION_MODE_DELETION;
    }
    if missing == 1
        && !ordered_subsequence
        && is_subsequence_after_one_adjacent_swap(observed, expected)
    {
        modes |= RECONSTRUCTION_MODE_DELETION_TRANSPOSITION;
    }
    modes
}

pub(in crate::nanda_wave::lexical_grokking) fn surface_operator_reconstruction_modes(
    observed: &str,
    expected: &str,
) -> u8 {
    let observed = normalize_lexical_surface(observed)
        .chars()
        .collect::<Vec<_>>();
    let expected = normalize_lexical_surface(expected)
        .chars()
        .collect::<Vec<_>>();
    if observed.len() != expected.len() || observed == expected {
        return 0;
    }

    let mismatches = observed
        .iter()
        .zip(&expected)
        .enumerate()
        .filter_map(|(index, (observed, expected))| (observed != expected).then_some(index))
        .collect::<Vec<_>>();
    match mismatches.as_slice() {
        [index]
            if crate::nanda_wave::surface_damage::alphabet_successor(expected[*index])
                == Some(observed[*index]) =>
        {
            RECONSTRUCTION_MODE_SINGLE_SUBSTITUTION
        }
        [first, second] => {
            let mut modes = 0;
            if crate::nanda_wave::surface_damage::alphabet_successor(expected[*first])
                == Some(observed[*first])
                && crate::nanda_wave::surface_damage::alphabet_successor(expected[*second])
                    == Some(observed[*second])
            {
                modes |= RECONSTRUCTION_MODE_DOUBLE_SUBSTITUTION;
            }
            if expected[*first] == observed[*second] && expected[*second] == observed[*first] {
                modes |= RECONSTRUCTION_MODE_NON_ADJACENT_TRANSPOSITION;
            }
            modes
        }
        _ => 0,
    }
}

pub(super) fn is_ordered_subsequence(needle: &[u32], haystack: &[u32]) -> bool {
    let mut next = 0;
    for symbol in haystack {
        if needle.get(next) == Some(symbol) {
            next += 1;
        }
    }
    next == needle.len()
}

pub(super) fn is_subsequence_after_one_adjacent_swap(observed: &[u32], expected: &[u32]) -> bool {
    if observed.len() < 2
        || expected.len() != observed.len().saturating_add(1)
        || observed.len() > MAX_ANCHOR_SEQUENCE
        || expected.len() > MAX_ANCHOR_SEQUENCE
    {
        return false;
    }

    fn visit(
        observed: &[u32],
        expected: &[u32],
        observed_index: usize,
        expected_index: usize,
        skipped: bool,
        swapped: bool,
    ) -> bool {
        if observed_index == observed.len() && expected_index == expected.len() {
            skipped && swapped
        } else {
            (!skipped
                && expected_index < expected.len()
                && visit(
                    observed,
                    expected,
                    observed_index,
                    expected_index + 1,
                    true,
                    swapped,
                ))
                || (observed_index < observed.len()
                    && expected_index < expected.len()
                    && observed[observed_index] == expected[expected_index]
                    && visit(
                        observed,
                        expected,
                        observed_index + 1,
                        expected_index + 1,
                        skipped,
                        swapped,
                    ))
                || (!swapped
                    && observed_index + 1 < observed.len()
                    && expected_index + 1 < expected.len()
                    && observed[observed_index] == expected[expected_index + 1]
                    && observed[observed_index + 1] == expected[expected_index]
                    && visit(
                        observed,
                        expected,
                        observed_index + 2,
                        expected_index + 2,
                        skipped,
                        true,
                    ))
                || (!skipped
                    && !swapped
                    && observed_index + 1 < observed.len()
                    && expected_index + 2 < expected.len()
                    && observed[observed_index] == expected[expected_index + 2]
                    && observed[observed_index + 1] == expected[expected_index]
                    && visit(
                        observed,
                        expected,
                        observed_index + 2,
                        expected_index + 3,
                        true,
                        true,
                    ))
        }
    }

    visit(observed, expected, 0, 0, false, false)
}

pub(in crate::nanda_wave::lexical_grokking) fn ambiguity_geometry_link(
    owner_distance: u8,
    competitor_distance: u8,
    max_geometry_distance: u8,
) -> bool {
    competitor_distance <= max_geometry_distance
        && owner_distance.abs_diff(competitor_distance) <= 1
}
