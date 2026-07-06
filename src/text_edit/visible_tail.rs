#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VisibleTailSource {
    DaemonWordBuffer,
    ImeActiveComposition,
    ImeCommittedTail,
}

impl VisibleTailSource {
    pub fn source_id(self) -> &'static str {
        match self {
            Self::DaemonWordBuffer => "daemon_word_buffer",
            Self::ImeActiveComposition => "ime_active_composition",
            Self::ImeCommittedTail => "ime_committed_tail",
        }
    }

    pub fn bridge_state(self) -> &'static str {
        match self {
            Self::DaemonWordBuffer => "passive:daemon-word-buffer",
            Self::ImeActiveComposition => "active:composition",
            Self::ImeCommittedTail => "passive:committed-tail",
        }
    }

    pub fn from_bridge_state(state: &str) -> Option<Self> {
        match state {
            "passive:daemon-word-buffer" => Some(Self::DaemonWordBuffer),
            "active:composition" => Some(Self::ImeActiveComposition),
            "passive:committed-tail" => Some(Self::ImeCommittedTail),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VisibleTail<'a> {
    pub text: &'a str,
    pub source: VisibleTailSource,
}

impl<'a> VisibleTail<'a> {
    pub fn daemon_word_buffer(text: &'a str) -> Self {
        Self {
            text,
            source: VisibleTailSource::DaemonWordBuffer,
        }
    }

    pub fn ime_active_composition(text: &'a str) -> Self {
        Self {
            text,
            source: VisibleTailSource::ImeActiveComposition,
        }
    }

    pub fn ime_committed_tail(text: &'a str) -> Self {
        Self {
            text,
            source: VisibleTailSource::ImeCommittedTail,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VisibleTailSnapshot {
    pub source: VisibleTailSource,
    pub expected_suffix: String,
    pub focus_id: Option<String>,
    pub epoch: u64,
}

impl VisibleTailSnapshot {
    pub fn new(
        source: VisibleTailSource,
        expected_suffix: impl Into<String>,
        focus_id: Option<String>,
        epoch: u64,
    ) -> Self {
        Self {
            source,
            expected_suffix: expected_suffix.into(),
            focus_id,
            epoch,
        }
    }

    pub fn matches_source_and_focus(
        &self,
        source: VisibleTailSource,
        focus_id: Option<&str>,
    ) -> bool {
        self.source == source
            && self
                .focus_id
                .as_deref()
                .map_or(true, |expected| Some(expected) == focus_id)
    }

    pub fn matches_current_suffix(&self, current_tail: &str, delete_chars: usize) -> bool {
        if self.expected_suffix.chars().count() != delete_chars {
            return false;
        }
        if current_tail.chars().count() < delete_chars {
            return false;
        }
        tail_suffix(current_tail, delete_chars) == self.expected_suffix
    }
}

fn tail_suffix(text: &str, chars: usize) -> String {
    if chars == 0 {
        return String::new();
    }
    let count = text.chars().count();
    text.chars().skip(count.saturating_sub(chars)).collect()
}

#[cfg(test)]
mod tests {
    use super::{VisibleTailSnapshot, VisibleTailSource};

    #[test]
    fn source_ids_are_stable_for_logs_and_edit_actions() {
        assert_eq!(
            VisibleTailSource::DaemonWordBuffer.source_id(),
            "daemon_word_buffer"
        );
        assert_eq!(
            VisibleTailSource::ImeActiveComposition.source_id(),
            "ime_active_composition"
        );
        assert_eq!(
            VisibleTailSource::ImeCommittedTail.source_id(),
            "ime_committed_tail"
        );
    }

    #[test]
    fn bridge_states_round_trip() {
        for source in [
            VisibleTailSource::DaemonWordBuffer,
            VisibleTailSource::ImeActiveComposition,
            VisibleTailSource::ImeCommittedTail,
        ] {
            assert_eq!(
                VisibleTailSource::from_bridge_state(source.bridge_state()),
                Some(source)
            );
        }
        assert_eq!(
            VisibleTailSource::from_bridge_state("passive:no-focus"),
            None
        );
    }

    #[test]
    fn visible_tail_snapshot_matches_expected_delete_suffix() {
        let snapshot = VisibleTailSnapshot::new(
            VisibleTailSource::DaemonWordBuffer,
            "bdtn",
            Some("/ime/focus".to_string()),
            0,
        );

        assert!(snapshot
            .matches_source_and_focus(VisibleTailSource::DaemonWordBuffer, Some("/ime/focus")));
        assert!(snapshot.matches_current_suffix("ghbdtn", 4));
        assert!(!snapshot.matches_current_suffix("ghjdt", 4));
        assert!(!snapshot.matches_current_suffix("bdtn", 5));
    }

    #[test]
    fn visible_tail_snapshot_rejects_wrong_focus() {
        let snapshot = VisibleTailSnapshot::new(
            VisibleTailSource::DaemonWordBuffer,
            "bdtn",
            Some("/ime/focus-a".to_string()),
            0,
        );

        assert!(!snapshot
            .matches_source_and_focus(VisibleTailSource::DaemonWordBuffer, Some("/ime/focus-b")));
    }
}
