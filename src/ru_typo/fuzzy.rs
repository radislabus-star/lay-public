#[cfg(test)]
use crate::text_metrics::damerau_levenshtein;

pub(crate) fn fuzzy_known_word_candidates(lower: &str) -> Vec<String> {
    crate::nanda_wave::l2::l2_center_near_surfaces(lower, 32)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn center_memory_recovers_typical_dirty_surfaces() {
        let mut missed = Vec::new();
        for (dirty, expected) in [
            ("ппоникаешь", "понимаешь"),
            ("эсперемнт", "эксперимент"),
            ("эаффективная", "эффективная"),
            ("руских", "русских"),
            ("звгрузи", "загрузи"),
            ("пукнт", "пункт"),
        ] {
            let candidates = fuzzy_known_word_candidates(dirty);
            if !candidates.iter().any(|candidate| candidate == expected) {
                missed.push((dirty, expected, candidates));
            }
        }
        assert!(missed.is_empty(), "missed center repairs: {missed:#?}");
    }

    #[test]
    fn center_memory_never_returns_the_dirty_surface_as_its_own_repair() {
        assert!(fuzzy_known_word_candidates("досвкйо")
            .iter()
            .all(|candidate| candidate != "досвкйо"));
    }

    #[test]
    fn damerau_counts_adjacent_swap_as_one_edit() {
        assert_eq!(damerau_levenshtein("йо", "ой"), 1);
    }
}
