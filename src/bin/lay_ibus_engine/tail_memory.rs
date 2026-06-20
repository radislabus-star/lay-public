use super::engine::LayIbusEngine;

impl LayIbusEngine {
    pub(super) fn selected_visible_completion_suffix(&self) -> String {
        visible_completion_suffix(self.selected_precognition_suffix())
    }

    pub(super) fn last_tail_token_text(&self) -> String {
        last_tail_token(&self.tail_buffer)
    }

    pub(super) fn tail_trailing_whitespace_chars(&self) -> usize {
        trailing_whitespace_chars(&self.tail_buffer)
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
