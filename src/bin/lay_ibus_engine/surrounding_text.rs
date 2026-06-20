use zbus::zvariant::Value;

use super::engine::LayIbusEngine;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct SurroundingTextSnapshot {
    text: String,
    cursor_pos: u32,
    anchor_pos: u32,
}

impl SurroundingTextSnapshot {
    pub(super) fn from_ibus_text(
        text: &Value<'_>,
        cursor_pos: u32,
        anchor_pos: u32,
    ) -> Option<Self> {
        Some(Self {
            text: ibus_text_value(text)?,
            cursor_pos,
            anchor_pos,
        })
    }

    fn last_token_before_cursor(&self) -> Option<String> {
        if self.cursor_pos != self.anchor_pos {
            return None;
        }
        let prefix: String = self.text.chars().take(self.cursor_pos as usize).collect();
        last_token(&prefix)
    }
}

impl LayIbusEngine {
    pub(super) fn update_surrounding_text(
        &mut self,
        text: &Value<'_>,
        cursor_pos: u32,
        anchor_pos: u32,
    ) {
        self.surrounding_text_supported = true;
        self.surrounding_text =
            SurroundingTextSnapshot::from_ibus_text(text, cursor_pos, anchor_pos);
    }

    pub(super) fn clear_surrounding_text_snapshot(&mut self) {
        self.surrounding_text_supported = false;
        self.surrounding_text = None;
    }

    pub(super) fn committed_tail_visible_token_matches(&self, internal_token: &str) -> bool {
        let Some(snapshot) = self.surrounding_text.as_ref() else {
            return true;
        };
        snapshot
            .last_token_before_cursor()
            .is_some_and(|visible_token| visible_token == internal_token)
    }
}

fn ibus_text_value(value: &Value<'_>) -> Option<String> {
    let Value::Structure(structure) = value else {
        return None;
    };
    let fields = structure.fields();
    let text = fields.get(2)?;
    <String>::try_from(text).ok()
}

fn last_token(text: &str) -> Option<String> {
    let end = text
        .char_indices()
        .rev()
        .find_map(|(idx, ch)| (!ch.is_whitespace()).then_some(idx + ch.len_utf8()))?;
    let start = text[..end]
        .char_indices()
        .rev()
        .find_map(|(idx, ch)| ch.is_whitespace().then_some(idx + ch.len_utf8()))
        .unwrap_or(0);
    Some(text[start..end].to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::text::make_ibus_text;

    #[test]
    fn extracts_text_from_ibus_text_value() {
        let value = make_ibus_text("abc привет".to_string());
        let snapshot = SurroundingTextSnapshot::from_ibus_text(&value, 10, 10).unwrap();

        assert_eq!(snapshot.text, "abc привет");
    }

    #[test]
    fn returns_last_token_before_cursor() {
        let value = make_ibus_text("abc привет хвост".to_string());
        let snapshot = SurroundingTextSnapshot::from_ibus_text(&value, 10, 10).unwrap();

        assert_eq!(
            snapshot.last_token_before_cursor().as_deref(),
            Some("привет")
        );
    }

    #[test]
    fn selection_disables_visible_token_match() {
        let value = make_ibus_text("abc привет".to_string());
        let snapshot = SurroundingTextSnapshot::from_ibus_text(&value, 10, 9).unwrap();

        assert_eq!(snapshot.last_token_before_cursor(), None);
    }

    #[test]
    fn extracts_last_token_range_by_whitespace() {
        assert_eq!(last_token("file ghbdtn").as_deref(), Some("ghbdtn"));
        assert_eq!(last_token("file ghbdtn ").as_deref(), Some("ghbdtn"));
    }
}
