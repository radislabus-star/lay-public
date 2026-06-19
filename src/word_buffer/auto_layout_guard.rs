use std::time::Instant;

use super::{PendingAutoLayoutGuard, WordBuffer};

const AUTO_LAYOUT_GUARD_MAX_AGE_MS: u128 = 2_500;

impl WordBuffer {
    pub fn remember_pending_auto_layout_guard(&mut self, original: &str, replacement: &str) {
        let Some((original_word, replacement_word)) =
            changed_single_layout_word(original, replacement)
        else {
            self.pending_auto_layout_guard = None;
            return;
        };

        self.pending_auto_layout_guard = Some(PendingAutoLayoutGuard {
            original_word,
            replacement_word,
            started_at: Instant::now(),
        });
    }

    pub fn should_consume_auto_layout_replay_guard(
        &mut self,
        visible_word: &str,
        replay_target: &str,
    ) -> bool {
        let Some(guard) = self.pending_auto_layout_guard.take() else {
            return false;
        };
        if guard.started_at.elapsed().as_millis() > AUTO_LAYOUT_GUARD_MAX_AGE_MS {
            return false;
        }

        guard.replacement_word == visible_word.trim() && guard.original_word == replay_target.trim()
    }
}

fn changed_single_layout_word(original: &str, replacement: &str) -> Option<(String, String)> {
    let original_words: Vec<_> = original.split_whitespace().collect();
    let replacement_words: Vec<_> = replacement.split_whitespace().collect();
    if original_words.len() != replacement_words.len() {
        return None;
    }

    let mut changed = None;
    for (from, to) in original_words.iter().zip(replacement_words.iter()) {
        if from == to {
            continue;
        }
        if changed.is_some() || !is_pure_layout_pair(from, to) {
            return None;
        }
        changed = Some(((*from).to_string(), (*to).to_string()));
    }
    changed
}

fn is_pure_layout_pair(from: &str, to: &str) -> bool {
    if !from.chars().all(|ch| ch.is_alphabetic()) || !to.chars().all(|ch| ch.is_alphabetic()) {
        return false;
    }
    crate::dict::convert(from, crate::dict::Direction::Ru2Us) == to
        || crate::dict::convert(from, crate::dict::Direction::Us2Ru) == to
}
