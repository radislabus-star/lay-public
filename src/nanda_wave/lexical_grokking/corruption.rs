use std::collections::BTreeSet;

use crate::dict::{convert, Direction};
use crate::stable_hash::mix64_golden;

const MAX_TRAINING_AUGMENTATIONS_PER_CLASS: usize = 12;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ScaleTrainingSurfacePolicy {
    LegacyAlphabetical,
    HybridClassConditioned,
}

impl ScaleTrainingSurfacePolicy {
    pub fn parse(value: &str) -> Result<Self, String> {
        match value {
            "legacy-alphabetical" => Ok(Self::LegacyAlphabetical),
            "hybrid-class-conditioned" => Ok(Self::HybridClassConditioned),
            _ => Err(format!(
                "unknown L1.1 training surface policy {value:?}; expected \
                 legacy-alphabetical or hybrid-class-conditioned"
            )),
        }
    }

    pub const fn name(self) -> &'static str {
        match self {
            Self::LegacyAlphabetical => "legacy-alphabetical",
            Self::HybridClassConditioned => "hybrid-class-conditioned",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct DamageExample {
    pub(super) class: &'static str,
    pub(super) surface: String,
}

pub(super) fn split_damages(word: &str) -> (Vec<DamageExample>, Vec<DamageExample>) {
    split_damages_with_training_augmentations(word, true)
}

pub(super) fn split_scale_damages(
    word: &str,
    include_training_augmentations: bool,
) -> (Vec<DamageExample>, Vec<DamageExample>) {
    split_damages_with_training_augmentations(word, include_training_augmentations)
}

fn split_damages_with_training_augmentations(
    word: &str,
    include_training_augmentations: bool,
) -> (Vec<DamageExample>, Vec<DamageExample>) {
    let mut all = Vec::new();
    let chars = word.chars().collect::<Vec<_>>();
    for position in 1..chars.len().saturating_sub(1) {
        push(
            &mut all,
            "missing_letter",
            remove_positions(&chars, &[position]),
        );
        if position + 1 < chars.len() && chars[position] != chars[position + 1] {
            let mut swapped = chars.clone();
            swapped.swap(position, position + 1);
            push(
                &mut all,
                "adjacent_transposition",
                swapped.into_iter().collect(),
            );
        }
        let mut duplicated = String::new();
        for (index, ch) in chars.iter().copied().enumerate() {
            duplicated.push(ch);
            if index == position {
                duplicated.push(ch);
            }
        }
        push(&mut all, "extra_letter", duplicated);
        if let Some(replacement) =
            crate::nanda_wave::surface_damage::alphabet_successor(chars[position])
        {
            let mut substituted = chars.clone();
            substituted[position] = replacement;
            push(
                &mut all,
                "letter_substitution",
                substituted.into_iter().collect(),
            );
        }
    }
    if chars.len() >= 7 {
        let first = chars.len() / 3;
        let second = (chars.len() * 2 / 3).min(chars.len() - 2);
        push(
            &mut all,
            "sparse_multi_omission",
            remove_positions(&chars, &[first, second]),
        );
    }
    if chars.len() >= 5 {
        let first = chars.len() / 3;
        let second = (chars.len() * 2 / 3).min(chars.len() - 2);
        if chars[first] != chars[second] {
            let mut swapped = chars.clone();
            swapped.swap(first, second);
            push(
                &mut all,
                "non_adjacent_transposition",
                swapped.into_iter().collect(),
            );
        }
        let mut double_substitution = chars.clone();
        if let (Some(first_value), Some(second_value)) = (
            crate::nanda_wave::surface_damage::alphabet_successor(chars[first]),
            crate::nanda_wave::surface_damage::alphabet_successor(chars[second]),
        ) {
            double_substitution[first] = first_value;
            double_substitution[second] = second_value;
            push(
                &mut all,
                "double_substitution",
                double_substitution.into_iter().collect(),
            );
        }
        let mut omission_transposition = chars.clone();
        omission_transposition.swap(first, first + 1);
        omission_transposition.remove(second);
        push(
            &mut all,
            "omission_transposition",
            omission_transposition.into_iter().collect(),
        );
        let fragment = chars[first..=first + 1].iter().collect::<String>();
        let repeated_fragment = format!(
            "{}{}{}",
            chars[..second].iter().collect::<String>(),
            fragment,
            chars[second..].iter().collect::<String>()
        );
        push(&mut all, "repeated_fragment", repeated_fragment);
        push(&mut all, "prefix_truncation", chars[1..].iter().collect());
        push(
            &mut all,
            "suffix_truncation",
            chars[..chars.len() - 1].iter().collect(),
        );
    }
    let direction = if chars.iter().any(|ch| ch.is_ascii_alphabetic()) {
        Direction::Us2Ru
    } else {
        Direction::Ru2Us
    };
    push(&mut all, "layout_projection", convert(word, direction));
    for suffix in ["!", ",", ".", "?"] {
        push(&mut all, "punctuation_suffix", format!("{word}{suffix}"));
    }

    let mut seen = BTreeSet::new();
    all.retain(|example| example.surface != word && seen.insert(example.surface.clone()));
    let mut training = Vec::new();
    let mut heldout = Vec::new();
    for example in all {
        if split_hash(word, &example) % 4 == 0 {
            heldout.push(example);
        } else {
            training.push(example);
        }
    }
    if heldout.is_empty() && !training.is_empty() {
        heldout.push(training.remove(0));
    }
    if include_training_augmentations {
        extend_training_damages(word, &chars, &mut seen, &mut training);
    }
    (training, heldout)
}

#[cfg(test)]
pub(super) fn select_scale_training_damages(
    word: &str,
    training: Vec<DamageExample>,
    maximum_surfaces: usize,
) -> Vec<DamageExample> {
    select_scale_training_damages_with_policy(
        word,
        training,
        maximum_surfaces,
        ScaleTrainingSurfacePolicy::LegacyAlphabetical,
    )
}

pub(super) fn select_scale_training_damages_with_policy(
    word: &str,
    training: Vec<DamageExample>,
    maximum_surfaces: usize,
    policy: ScaleTrainingSurfacePolicy,
) -> Vec<DamageExample> {
    let mut by_class = std::collections::BTreeMap::<&'static str, Vec<DamageExample>>::new();
    for example in training {
        by_class.entry(example.class).or_default().push(example);
    }
    for examples in by_class.values_mut() {
        examples.sort_unstable_by_key(|example| split_hash(word, example));
    }

    let mut selected = Vec::with_capacity(maximum_surfaces);
    const HYBRID_CLASSES: &[&str] = &[
        "layout_projection",
        "double_substitution",
        "omission_transposition",
        "sparse_multi_omission",
        "adjacent_transposition",
        "extra_letter",
    ];
    const LEGACY_REFILL_CLASSES: &[&str] = &[
        "sparse_multi_omission",
        "omission_transposition",
        "non_adjacent_transposition",
        "double_substitution",
        "missing_letter",
        "letter_substitution",
        "extra_letter",
        "adjacent_transposition",
    ];
    if policy == ScaleTrainingSurfacePolicy::LegacyAlphabetical {
        for examples in by_class.values_mut() {
            if selected.len() == maximum_surfaces {
                break;
            }
            if let Some(example) = examples.pop() {
                selected.push(example);
            }
        }
    }
    let refill_classes = match policy {
        ScaleTrainingSurfacePolicy::LegacyAlphabetical => LEGACY_REFILL_CLASSES,
        ScaleTrainingSurfacePolicy::HybridClassConditioned => HYBRID_CLASSES,
    };
    while selected.len() < maximum_surfaces {
        let mut added = false;
        for class in refill_classes {
            let Some(examples) = by_class.get_mut(class) else {
                continue;
            };
            if let Some(example) = examples.pop() {
                selected.push(example);
                added = true;
                if selected.len() == maximum_surfaces {
                    break;
                }
            }
        }
        if !added {
            break;
        }
    }
    selected.sort_unstable_by(|left, right| {
        left.class
            .cmp(right.class)
            .then_with(|| left.surface.cmp(&right.surface))
    });
    selected
}

fn extend_training_damages(
    word: &str,
    chars: &[char],
    seen: &mut BTreeSet<String>,
    training: &mut Vec<DamageExample>,
) {
    if chars.len() < 5 {
        return;
    }
    let mut counts = std::collections::BTreeMap::<&'static str, usize>::new();
    let mut position_pairs = (1..chars.len().saturating_sub(2))
        .flat_map(|first| {
            (first + 2..chars.len().saturating_sub(1)).map(move |second| (first, second))
        })
        .collect::<Vec<_>>();
    position_pairs
        .sort_unstable_by_key(|(first, second)| augmentation_pair_hash(word, *first, *second));
    for (first, second) in position_pairs {
        if chars[first] != chars[second] {
            let mut transposed = chars.to_vec();
            transposed.swap(first, second);
            push_training_unique(
                word,
                "non_adjacent_transposition",
                transposed.into_iter().collect(),
                seen,
                training,
                &mut counts,
            );
        }

        if let (Some(first_value), Some(second_value)) = (
            crate::nanda_wave::surface_damage::alphabet_successor(chars[first]),
            crate::nanda_wave::surface_damage::alphabet_successor(chars[second]),
        ) {
            let mut substituted = chars.to_vec();
            substituted[first] = first_value;
            substituted[second] = second_value;
            push_training_unique(
                word,
                "double_substitution",
                substituted.into_iter().collect(),
                seen,
                training,
                &mut counts,
            );
        }

        let mut omission_transposition = chars.to_vec();
        omission_transposition.swap(first, first + 1);
        omission_transposition.remove(second);
        push_training_unique(
            word,
            "omission_transposition",
            omission_transposition.into_iter().collect(),
            seen,
            training,
            &mut counts,
        );

        push_training_unique(
            word,
            "sparse_multi_omission",
            remove_positions(chars, &[first, second]),
            seen,
            training,
            &mut counts,
        );

        let fragment = chars[first..=first + 1].iter().collect::<String>();
        let repeated = format!(
            "{}{}{}",
            chars[..second].iter().collect::<String>(),
            fragment,
            chars[second..].iter().collect::<String>()
        );
        push_training_unique(
            word,
            "repeated_fragment",
            repeated,
            seen,
            training,
            &mut counts,
        );
    }
}

fn augmentation_pair_hash(word: &str, first: usize, second: usize) -> u64 {
    let mut state = 0x4155_474d_454e_5431_u64;
    for byte in word.bytes() {
        state = mix64_golden(state ^ u64::from(byte));
    }
    mix64_golden(state ^ (first as u64).rotate_left(17) ^ (second as u64).rotate_left(41))
}

fn push_training_unique(
    word: &str,
    class: &'static str,
    surface: String,
    seen: &mut BTreeSet<String>,
    training: &mut Vec<DamageExample>,
    counts: &mut std::collections::BTreeMap<&'static str, usize>,
) {
    let count = counts.entry(class).or_default();
    if *count >= MAX_TRAINING_AUGMENTATIONS_PER_CLASS
        || surface.is_empty()
        || surface == word
        || !seen.insert(surface.clone())
    {
        return;
    }
    training.push(DamageExample { class, surface });
    *count += 1;
}

fn push(examples: &mut Vec<DamageExample>, class: &'static str, surface: String) {
    if !surface.is_empty() {
        examples.push(DamageExample { class, surface });
    }
}

fn remove_positions(chars: &[char], positions: &[usize]) -> String {
    chars
        .iter()
        .copied()
        .enumerate()
        .filter_map(|(index, ch)| (!positions.contains(&index)).then_some(ch))
        .collect()
}

fn split_hash(word: &str, example: &DamageExample) -> u64 {
    let mut state = 0x4845_4c44_4f55_5431_u64;
    for byte in word
        .bytes()
        .chain(example.class.bytes())
        .chain(example.surface.bytes())
    {
        state = mix64_golden(state ^ u64::from(byte));
    }
    state
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn composite_damage_training_has_multiple_disjoint_surfaces() {
        let (training, heldout) = split_damages("перезагрузка");
        for class in [
            "double_substitution",
            "non_adjacent_transposition",
            "omission_transposition",
        ] {
            assert!(training.iter().filter(|item| item.class == class).count() > 1);
        }
        let training_surfaces = training
            .iter()
            .map(|item| item.surface.as_str())
            .collect::<BTreeSet<_>>();
        assert!(heldout
            .iter()
            .all(|item| !training_surfaces.contains(item.surface.as_str())));
    }

    #[test]
    fn zero_depth_selects_no_damaged_training_surfaces() {
        let (training, _) = split_damages("перезагрузка");

        assert!(select_scale_training_damages("перезагрузка", training, 0).is_empty());
    }

    #[test]
    fn zero_depth_skips_only_training_augmentations_and_preserves_heldout() {
        let (_, full_heldout) = split_damages("перезагрузка");
        let (base_training, zero_depth_heldout) = split_scale_damages("перезагрузка", false);

        assert_eq!(zero_depth_heldout, full_heldout);
        assert!(!base_training.is_empty());
        assert!(select_scale_training_damages("перезагрузка", base_training, 0).is_empty());
    }

    #[test]
    fn hybrid_policy_preserves_layout_lane_before_refilling_easy_classes() {
        let training = [
            "adjacent_transposition",
            "double_substitution",
            "extra_letter",
            "layout_projection",
            "omission_transposition",
            "sparse_multi_omission",
        ]
        .into_iter()
        .enumerate()
        .map(|(index, class)| DamageExample {
            class,
            surface: format!("training-surface-{index}"),
        })
        .collect();
        let selected = select_scale_training_damages_with_policy(
            "перезагрузка",
            training,
            4,
            ScaleTrainingSurfacePolicy::HybridClassConditioned,
        );
        let classes = selected
            .iter()
            .map(|example| example.class)
            .collect::<Vec<_>>();

        assert_eq!(
            classes,
            [
                "double_substitution",
                "layout_projection",
                "omission_transposition",
                "sparse_multi_omission",
            ]
        );
    }
}
