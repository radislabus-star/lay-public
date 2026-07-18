use std::time::Instant;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum WordInputMode {
    ManagedCommit,
    TerminalPassthrough,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ManualToggleAuthority {
    ImeActiveComposition,
    ImeCommittedTail,
    DaemonWordBuffer,
}

#[derive(Debug, Clone)]
pub(crate) struct RecentCommittedTailReplace {
    pub(crate) backspaces: u32,
    pub(crate) text: String,
    pub(crate) at: Instant,
}

#[derive(Debug, Clone)]
pub(crate) struct PendingVisiblePostcondition {
    pub(crate) expected_suffix: String,
    pub(crate) dispatched_epoch: u64,
    pub(crate) dispatched_at: Instant,
}

/// A Tab completion is provisional until the user starts the next word.
/// Deleting the accepted tail first means the candidate was not actually useful.
#[derive(Debug, Clone)]
pub(crate) struct PendingImeCompletionLearning {
    pub(crate) context_tail: String,
    pub(crate) accepted_word: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SurroundingTextSnapshot {
    pub(crate) text: String,
    pub(crate) cursor_pos: u32,
    pub(crate) anchor_pos: u32,
}

impl SurroundingTextSnapshot {
    pub(crate) fn new(text: String, cursor_pos: u32, anchor_pos: u32) -> Self {
        Self {
            text,
            cursor_pos,
            anchor_pos,
        }
    }

    pub(crate) fn suffix_before_cursor(&self, chars: usize) -> Option<String> {
        if chars == 0 {
            return Some(String::new());
        }
        let cursor = self.cursor_pos as usize;
        if cursor < chars || self.text.chars().count() < cursor {
            return None;
        }
        Some(
            self.text
                .chars()
                .take(cursor)
                .skip(cursor - chars)
                .collect(),
        )
    }

    pub(crate) fn has_selection(&self) -> bool {
        self.cursor_pos != self.anchor_pos
    }
}
