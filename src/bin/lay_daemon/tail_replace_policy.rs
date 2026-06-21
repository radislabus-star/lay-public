pub(crate) fn full_tail_replace_required(original: &str) -> bool {
    original
        .trim_start()
        .chars()
        .next()
        .is_some_and(|ch| !ch.is_alphanumeric())
}

#[cfg(test)]
mod tests {
    use super::full_tail_replace_required;

    #[test]
    fn punctuation_led_tail_requires_full_tail_replacement() {
        assert!(full_tail_replace_required("'nj "));
        assert!(full_tail_replace_required(" -b "));
        assert!(!full_tail_replace_required("lfdfq "));
        assert!(!full_tail_replace_required("вот "));
    }
}
