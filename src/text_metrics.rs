//! Shared text metrics for scorers and candidate arbitration.

pub(crate) fn has_cyrillic(text: &str) -> bool {
    text.chars().any(is_cyrillic_char)
}

pub(crate) fn has_latin(text: &str) -> bool {
    text.chars().any(|ch| ch.is_ascii_alphabetic())
}

pub(crate) fn is_cyrillic_char(ch: char) -> bool {
    matches!(ch, 'А'..='я' | 'ё' | 'Ё')
}

pub fn without_whitespace(text: &str) -> String {
    text.chars().filter(|ch| !ch.is_whitespace()).collect()
}

pub(crate) fn normalized_edit_distance(left: &str, right: &str) -> f64 {
    let distance = damerau_levenshtein(left, right) as f64;
    let scale = left.chars().count().max(right.chars().count()).max(1) as f64;
    distance / scale
}

pub(crate) fn common_replacement_span(left: &str, right: &str) -> usize {
    let left_chars: Vec<char> = left.chars().collect();
    let right_chars: Vec<char> = right.chars().collect();
    let mut prefix = 0usize;
    while prefix < left_chars.len()
        && prefix < right_chars.len()
        && left_chars[prefix] == right_chars[prefix]
    {
        prefix += 1;
    }
    let mut suffix = 0usize;
    while suffix < left_chars.len().saturating_sub(prefix)
        && suffix < right_chars.len().saturating_sub(prefix)
        && left_chars[left_chars.len() - 1 - suffix] == right_chars[right_chars.len() - 1 - suffix]
    {
        suffix += 1;
    }
    left_chars.len().saturating_sub(prefix + suffix)
}

pub(crate) fn damerau_levenshtein(left: &str, right: &str) -> usize {
    let a: Vec<char> = left.chars().collect();
    let b: Vec<char> = right.chars().collect();
    let mut dp = vec![vec![0usize; b.len() + 1]; a.len() + 1];
    for (i, row) in dp.iter_mut().enumerate() {
        row[0] = i;
    }
    for (j, cell) in dp[0].iter_mut().enumerate() {
        *cell = j;
    }
    for i in 1..=a.len() {
        for j in 1..=b.len() {
            let substitution = usize::from(a[i - 1] != b[j - 1]);
            let mut best = (dp[i - 1][j] + 1)
                .min(dp[i][j - 1] + 1)
                .min(dp[i - 1][j - 1] + substitution);
            if i > 1 && j > 1 && a[i - 1] == b[j - 2] && a[i - 2] == b[j - 1] {
                best = best.min(dp[i - 2][j - 2] + 1);
            }
            dp[i][j] = best;
        }
    }
    dp[a.len()][b.len()]
}
