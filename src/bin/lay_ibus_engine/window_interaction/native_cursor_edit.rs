//! Inert, bounded provenance for explicit manual edits after native re-editing.
use super::observation::SurroundingTextSnapshot;
use crate::context_admission::AdmissionToken;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum CursorEditOperation {
    Left,
    Right,
    End,
    Backspace,
    OwnedInsert(char),
}

/// This record is not a word-start grant, a client observation or an edit lease.
/// The owning reducer validates its token; the caller must first establish the
/// original locally owned suffix through the existing exact manual contract.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct ManualCursorEditReceipt {
    pub(super) token: AdmissionToken,
    pub(super) focus_serial: u64,
    pub(super) runtime_owner_lease_identity: u64,
    pub(super) layout_generation: u64,
    pub(super) actual: SurroundingTextSnapshot,
    start: u32,
    end: u32,
    expected: Option<(SurroundingTextSnapshot, u32)>,
    owned_insert: Option<char>,
    pub(super) last_key: Option<(u32, u32)>,
    pub(super) end_reset_spent: bool,
}

impl ManualCursorEditReceipt {
    pub(super) fn from_admitted_suffix(
        token: AdmissionToken,
        actual: SurroundingTextSnapshot,
        suffix: &str,
        identity: (u64, u64, u64),
    ) -> Option<Self> {
        let chars = u32::try_from(suffix.chars().count()).ok()?;
        if chars == 0
            || chars as usize > crate::preedit::PREEDIT_TAIL_LIMIT
            || actual.has_selection()
            || actual.cursor_pos as usize != actual.text.chars().count()
            || actual.suffix_before_cursor(chars as usize).as_deref() != Some(suffix)
        {
            return None;
        }
        let start = actual.cursor_pos.checked_sub(chars)?;
        let end = actual.cursor_pos;
        Some(Self {
            token,
            actual,
            start,
            end,
            expected: None,
            owned_insert: None,
            focus_serial: identity.0,
            runtime_owner_lease_identity: identity.1,
            layout_generation: identity.2,
            last_key: None,
            end_reset_spent: false,
        })
    }

    /// Only one expected effect can be outstanding. Projection cannot confirm
    /// it: the complete actual successor must match, including external text.
    pub(super) fn prepare(&mut self, operation: CursorEditOperation, key: (u32, u32)) -> bool {
        if self.expected.is_some() || self.owned_insert.is_some() || self.actual.has_selection() {
            return false;
        }
        let cursor = self.actual.cursor_pos;
        if cursor < self.start || cursor > self.end {
            return false;
        }
        let mut next = self.actual.clone();
        let mut end = self.end;
        let position = self
            .actual
            .text
            .char_indices()
            .nth(cursor as usize)
            .map_or(self.actual.text.len(), |(position, _)| position);
        let next_cursor = match operation {
            CursorEditOperation::Left if cursor > self.start => cursor - 1,
            CursorEditOperation::Right if cursor < self.end => cursor + 1,
            CursorEditOperation::End => self.end,
            CursorEditOperation::Backspace if cursor > self.start && self.end > self.start + 1 => {
                let Some((before, _)) = self.actual.text.char_indices().nth(cursor as usize - 1)
                else {
                    return false;
                };
                next.text.replace_range(before..position, "");
                end -= 1;
                cursor - 1
            }
            CursorEditOperation::OwnedInsert(ch)
                if !crate::preedit::is_observed_word_boundary(ch)
                    && ((self.end - self.start) as usize) < crate::preedit::PREEDIT_TAIL_LIMIT =>
            {
                next.text.insert(position, ch);
                end += 1;
                self.owned_insert = Some(ch);
                cursor + 1
            }
            _ => return false,
        };
        next.cursor_pos = next_cursor;
        next.anchor_pos = next_cursor;
        self.expected = Some((next, end));
        self.last_key = Some(key);
        self.end_reset_spent = false;
        true
    }

    pub(super) fn observe(&mut self, actual: &SurroundingTextSnapshot) -> bool {
        let Some((expected, end)) = self.expected.as_ref() else {
            // A duplicate actual receipt neither creates a new effect nor
            // rearms the one allowed acknowledgement of the final Reset.
            return actual == &self.actual;
        };
        if actual != expected || actual.has_selection() {
            return false;
        }
        self.actual = actual.clone();
        self.end = *end;
        self.expected = None;
        true
    }

    pub(super) fn confirm_owned_insert(&mut self, ch: char) -> bool {
        if self.owned_insert != Some(ch) {
            return false;
        }
        self.owned_insert = None;
        true
    }

    pub(super) fn awaits_effect(&self) -> bool {
        self.expected.is_some()
    }

    pub(super) fn owned_text(&self) -> String {
        self.actual
            .text
            .chars()
            .skip(self.start as usize)
            .take((self.end - self.start) as usize)
            .collect()
    }

    pub(super) fn exact_manual_suffix(&self) -> Option<String> {
        if self.expected.is_some()
            || self.owned_insert.is_some()
            || self.actual.has_selection()
            || self.actual.cursor_pos != self.end
        {
            return None;
        }
        Some(
            self.actual
                .text
                .chars()
                .skip(self.start as usize)
                .take((self.end - self.start) as usize)
                .collect(),
        )
    }
}
