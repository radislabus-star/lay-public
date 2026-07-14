use super::engine::LayIbusEngine;
use std::time::Instant;

impl LayIbusEngine {
    pub(super) fn selected_visible_completion_suffix(&self) -> String {
        visible_completion_suffix(self.selected_precognition_suffix())
    }

    pub(super) fn last_tail_token_text(&self) -> String {
        last_tail_token(&self.tail_buffer)
    }

    pub(super) fn sync_tail_after_composition_commit(&mut self, text: &str) {
        let trailing_ws = trailing_whitespace_chars(text);
        let committed = text.trim_end_matches(char::is_whitespace);
        if !committed.is_empty() {
            self.replace_last_tail_token_text(committed, self.buffer.chars().count());
        }
        for _ in 0..trailing_ws {
            self.tail_buffer.push(' ');
        }
        if trailing_ws > 0 {
            self.preedit_fast.reset();
        } else {
            self.rebuild_preedit_fast_from_tail();
        }
        trim_committed_tail_buffer(&mut self.tail_buffer);
        self.publish_tail_handoff();
    }

    pub(super) fn replace_last_tail_token_text(&mut self, replacement: &str, fallback_len: usize) {
        let Some((start, end)) = last_tail_token_range(&self.tail_buffer) else {
            self.tail_buffer.push_str(replacement);
            return;
        };
        let range_len = self.tail_buffer[start..end].chars().count();
        if fallback_len > 0 && range_len != fallback_len {
            self.tail_buffer.push_str(replacement);
            return;
        }
        self.tail_buffer.replace_range(start..end, replacement);
    }

    pub(super) fn rebuild_preedit_fast_from_tail(&mut self) {
        self.preedit_fast.reset();
        for ch in self.last_tail_token_text().chars() {
            self.preedit_fast.push(ch);
        }
    }

    pub(super) fn publish_tail_handoff(&self) {
        let Ok(mut state) = self.shared.lock() else {
            return;
        };
        state.handoff_tail_buffer = self.tail_buffer.clone();
    }

    pub(super) fn close_committed_tail_field(&mut self) {
        self.tail_buffer.clear();
        self.preedit_fast.reset();
        self.suppress_next_committed_tail_autocorrect = false;
        self.word_input_mode = None;
        self.last_tail_input_at = None;
        self.last_commit_at = None;
        self.recent_committed_tail_replace = None;
        let Ok(mut state) = self.shared.lock() else {
            return;
        };
        state.handoff_tail_buffer.clear();
        state.suppress_next_committed_tail_autocorrect = false;
        state.preserve_active_path_until = None;
    }

    pub(super) fn refresh_empty_tail_from_handoff(&mut self) {
        if !self.tail_buffer.is_empty() {
            return;
        }
        let Ok(state) = self.shared.lock() else {
            return;
        };
        if state.handoff_tail_buffer.is_empty() {
            return;
        }
        self.tail_buffer.clone_from(&state.handoff_tail_buffer);
        drop(state);
        self.rebuild_preedit_fast_from_tail();
    }

    pub(super) fn publish_autocorrect_suppression_handoff(&self) {
        let Ok(mut state) = self.shared.lock() else {
            return;
        };
        state.suppress_next_committed_tail_autocorrect = true;
    }

    #[cfg(test)]
    pub(super) fn take_autocorrect_suppression_handoff(&self) -> bool {
        let Ok(mut state) = self.shared.lock() else {
            return false;
        };
        let suppress = state.suppress_next_committed_tail_autocorrect;
        state.suppress_next_committed_tail_autocorrect = false;
        suppress
    }

    pub(super) fn clear_autocorrect_suppression_handoff(&self) {
        let Ok(mut state) = self.shared.lock() else {
            return;
        };
        state.suppress_next_committed_tail_autocorrect = false;
    }

    pub(super) fn publish_active_path_preserve_handoff(&self, until: Instant) {
        let Ok(mut state) = self.shared.lock() else {
            return;
        };
        state.preserve_active_path_until = Some(until);
    }

    pub(super) fn shared_active_path_preserved(&self) -> bool {
        let Ok(mut state) = self.shared.lock() else {
            return false;
        };
        let Some(until) = state.preserve_active_path_until else {
            return false;
        };
        if Instant::now() <= until {
            return true;
        }
        state.preserve_active_path_until = None;
        false
    }
}

fn visible_completion_suffix(suffix: Option<String>) -> String {
    suffix.filter(|suffix| suffix != "*").unwrap_or_default()
}

fn last_tail_token(tail: &str) -> String {
    last_tail_token_range(tail)
        .map(|(start, end)| tail[start..end].to_string())
        .unwrap_or_default()
}

fn last_tail_token_range(tail: &str) -> Option<(usize, usize)> {
    let end = tail
        .char_indices()
        .rev()
        .find_map(|(idx, ch)| (!ch.is_whitespace()).then_some(idx + ch.len_utf8()))?;
    let start = tail[..end]
        .char_indices()
        .rev()
        .find_map(|(idx, ch)| ch.is_whitespace().then_some(idx + ch.len_utf8()))
        .unwrap_or(0);
    Some((start, end))
}

fn trailing_whitespace_chars(text: &str) -> usize {
    text.chars()
        .rev()
        .take_while(|ch| ch.is_whitespace())
        .count()
}

fn trim_committed_tail_buffer(buffer: &mut String) {
    const LIMIT: usize = 160;
    let chars = buffer.chars().count();
    if chars <= LIMIT {
        return;
    }
    let remove = chars - LIMIT;
    if let Some((idx, _)) = buffer.char_indices().nth(remove) {
        buffer.drain(..idx);
    }
}

#[cfg(test)]
mod tests {
    use super::{last_tail_token_range, trailing_whitespace_chars, LayIbusEngine};
    use crate::engine::WordInputMode;
    use lay::config::LayConfig;
    use std::sync::{Arc, Mutex};
    use std::time::{Duration, Instant};

    #[test]
    fn committed_tail_range_keeps_separator_outside_token() {
        let tail = "file проверка ";
        let (start, end) = last_tail_token_range(tail).expect("last token");
        assert_eq!(&tail[start..end], "проверка");
        assert_eq!(trailing_whitespace_chars(tail), 1);
    }

    #[test]
    fn committed_space_keeps_next_word_separated_in_tail_memory() {
        let mut engine = LayIbusEngine::new(
            "/test".to_string(),
            Arc::new(Mutex::new(Default::default())),
            true,
            true,
            LayConfig::default(),
        );
        for ch in "печатаетт".chars() {
            engine.insert_composition_char(ch);
        }
        engine.sync_tail_after_composition_commit("печатается ");
        engine.insert_composition_char('т');
        engine.insert_composition_char('ы');
        assert_eq!(engine.tail_buffer, "печатается ты");
        assert_eq!(engine.preedit_fast.token(), "ты");
    }

    #[test]
    fn focus_reset_preserves_just_typed_passthrough_tail() {
        let mut engine = LayIbusEngine::new(
            "/test".to_string(),
            Arc::new(Mutex::new(Default::default())),
            false,
            true,
            LayConfig::default(),
        );

        engine.push_tail_char('g');
        engine.reset_for_ibus_focus_change();

        assert_eq!(engine.tail_buffer, "g");
        assert_eq!(engine.preedit_fast.token(), "g");
    }

    #[test]
    fn focus_reset_clears_stale_passthrough_tail() {
        let mut engine = LayIbusEngine::new(
            "/test".to_string(),
            Arc::new(Mutex::new(Default::default())),
            false,
            true,
            LayConfig::default(),
        );

        engine.push_tail_char('g');
        engine.last_tail_input_at = Some(Instant::now() - Duration::from_millis(900));
        engine.reset_for_ibus_focus_change();

        assert!(engine.tail_buffer.is_empty());
        assert_eq!(engine.preedit_fast.token(), "");
    }

    #[test]
    fn ibus_soft_reset_preserves_tail_for_manual_toggle() {
        let mut engine = LayIbusEngine::new(
            "/test".to_string(),
            Arc::new(Mutex::new(Default::default())),
            false,
            true,
            LayConfig::default(),
        );

        engine.word_input_mode = Some(WordInputMode::ManagedCommit);
        for ch in "ghbdtn".chars() {
            engine.push_tail_char(ch);
        }
        engine.reset_for_ibus_soft_reset();

        assert_eq!(engine.tail_buffer, "ghbdtn");
        assert_eq!(engine.preedit_fast.token(), "ghbdtn");
        assert_eq!(engine.word_input_mode, Some(WordInputMode::ManagedCommit));
    }

    #[test]
    fn ibus_soft_reset_preserves_manual_toggle_autocorrect_suppression() {
        let mut engine = LayIbusEngine::new(
            "/test".to_string(),
            Arc::new(Mutex::new(Default::default())),
            false,
            true,
            LayConfig::default(),
        );

        engine.suppress_next_committed_tail_autocorrect = true;
        engine.reset_for_ibus_soft_reset();

        assert!(engine.suppress_next_committed_tail_autocorrect);
    }

    #[test]
    fn focus_reset_without_preserve_clears_shared_autocorrect_suppression() {
        let shared = Arc::new(Mutex::new(Default::default()));
        let mut engine = LayIbusEngine::new(
            "/test".to_string(),
            shared.clone(),
            false,
            true,
            LayConfig::default(),
        );
        engine.publish_autocorrect_suppression_handoff();

        engine.reset_for_ibus_focus_change();

        let state = shared.lock().expect("lay ime state poisoned");
        assert!(!state.suppress_next_committed_tail_autocorrect);
    }

    #[test]
    fn close_committed_tail_field_clears_shared_tail_and_preserve_window() {
        let shared = Arc::new(Mutex::new(Default::default()));
        let mut engine = LayIbusEngine::new(
            "/test".to_string(),
            shared.clone(),
            false,
            true,
            LayConfig::default(),
        );
        engine.tail_buffer = "file проверка".to_string();
        engine.publish_tail_handoff();
        engine.publish_active_path_preserve_handoff(Instant::now() + Duration::from_millis(100));

        engine.close_committed_tail_field();

        let state = shared.lock().expect("lay ime state poisoned");
        assert!(engine.tail_buffer.is_empty());
        assert!(state.handoff_tail_buffer.is_empty());
        assert!(state.preserve_active_path_until.is_none());
    }

    #[test]
    fn active_path_preserve_handoff_is_shared_between_engine_objects() {
        let shared = Arc::new(Mutex::new(Default::default()));
        let publisher = LayIbusEngine::new(
            "/publisher".to_string(),
            shared.clone(),
            false,
            true,
            LayConfig::default(),
        );
        let reader = LayIbusEngine::new(
            "/reader".to_string(),
            shared,
            false,
            true,
            LayConfig::default(),
        );

        publisher.publish_active_path_preserve_handoff(Instant::now() + Duration::from_millis(100));

        assert!(reader.shared_active_path_preserved());
    }

    #[test]
    fn focus_engine_can_refresh_empty_tail_from_shared_handoff() {
        let shared = Arc::new(Mutex::new(Default::default()));
        let mut publisher = LayIbusEngine::new(
            "/publisher".to_string(),
            shared.clone(),
            false,
            true,
            LayConfig::default(),
        );
        publisher.tail_buffer = "вот ".to_string();
        publisher.publish_tail_handoff();
        let mut reader = LayIbusEngine::new(
            "/reader".to_string(),
            shared,
            true,
            true,
            LayConfig::default(),
        );
        reader.tail_buffer.clear();

        reader.refresh_empty_tail_from_handoff();

        assert_eq!(reader.tail_buffer, "вот ");
        assert_eq!(reader.preedit_fast.token(), "вот");
    }

    #[test]
    fn empty_focus_reset_does_not_overwrite_preserved_shared_tail() {
        let shared = Arc::new(Mutex::new(Default::default()));
        let mut publisher = LayIbusEngine::new(
            "/publisher".to_string(),
            shared.clone(),
            false,
            true,
            LayConfig::default(),
        );
        publisher.tail_buffer = "вот ".to_string();
        publisher.publish_tail_handoff();
        publisher.publish_active_path_preserve_handoff(Instant::now() + Duration::from_millis(100));
        let mut empty_engine = LayIbusEngine::new(
            "/empty".to_string(),
            shared.clone(),
            true,
            true,
            LayConfig::default(),
        );
        empty_engine.tail_buffer.clear();

        empty_engine.reset_for_ibus_focus_change();

        let state = shared.lock().expect("lay ime state poisoned");
        assert_eq!(state.handoff_tail_buffer, "вот ");
    }

    #[test]
    fn whitespace_closes_current_word_input_mode() {
        let mut engine = LayIbusEngine::new(
            "/test".to_string(),
            Arc::new(Mutex::new(Default::default())),
            false,
            true,
            LayConfig::default(),
        );

        engine.word_input_mode = Some(WordInputMode::ManagedCommit);
        engine.push_tail_char('a');
        engine.push_tail_char(' ');

        assert_eq!(engine.word_input_mode, None);
    }

    #[test]
    fn fresh_focus_reset_preserves_current_word_input_mode() {
        let mut engine = LayIbusEngine::new(
            "/test".to_string(),
            Arc::new(Mutex::new(Default::default())),
            false,
            true,
            LayConfig::default(),
        );

        engine.word_input_mode = Some(WordInputMode::ManagedCommit);
        engine.push_tail_char('f');
        engine.reset_for_ibus_focus_change();

        assert_eq!(engine.tail_buffer, "f");
        assert_eq!(engine.word_input_mode, Some(WordInputMode::ManagedCommit));
    }
}
