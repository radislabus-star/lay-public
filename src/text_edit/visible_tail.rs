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
pub struct SnapshotIdentity {
    /// Adapter boundary that observed the text. A lease is never transferable
    /// between the daemon buffer, active IBus composition, and committed tail.
    pub source: VisibleTailSource,
    pub focus_id: Option<String>,
    pub revision: u64,
    pub caret: Option<u32>,
    pub selection: Option<(u32, u32)>,
    pub composition_generation: Option<u64>,
    pub layout_epoch: Option<u64>,
    pub visible_tail_hash: u64,
}

impl SnapshotIdentity {
    fn from_snapshot(snapshot: &VisibleTailSnapshot) -> Self {
        Self {
            source: snapshot.source,
            focus_id: snapshot.focus_id.clone(),
            revision: snapshot.epoch,
            caret: snapshot.caret,
            selection: snapshot.selection,
            composition_generation: snapshot.composition_generation,
            layout_epoch: snapshot.layout_epoch,
            visible_tail_hash: stable_tail_hash(&snapshot.expected_suffix),
        }
    }
}

/// Immutable lease for one visible-text observation.
///
/// Candidate generation can use a snapshot freely, but an output adapter must
/// present this lease again before it is allowed to dispatch an edit.  Fields
/// unavailable on a particular adapter remain `None`; they cannot be invented
/// later by a backend.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VisibleTailSnapshot {
    pub source: VisibleTailSource,
    pub expected_suffix: String,
    pub focus_id: Option<String>,
    pub epoch: u64,
    pub caret: Option<u32>,
    pub selection: Option<(u32, u32)>,
    pub composition_generation: Option<u64>,
    pub layout_epoch: Option<u64>,
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
            caret: None,
            selection: None,
            composition_generation: None,
            layout_epoch: None,
        }
    }

    pub fn with_runtime_coordinates(
        mut self,
        caret: Option<u32>,
        selection: Option<(u32, u32)>,
        composition_generation: Option<u64>,
        layout_epoch: Option<u64>,
    ) -> Self {
        self.caret = caret;
        self.selection = selection;
        self.composition_generation = composition_generation;
        self.layout_epoch = layout_epoch;
        self
    }

    pub fn identity(&self) -> SnapshotIdentity {
        SnapshotIdentity::from_snapshot(self)
    }

    pub fn matches_source_focus_and_epoch(
        &self,
        source: VisibleTailSource,
        focus_id: Option<&str>,
        epoch: u64,
    ) -> bool {
        self.source == source
            && self.epoch == epoch
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

fn stable_tail_hash(text: &str) -> u64 {
    // FNV-1a is deterministic across processes; this is an identity guard,
    // not a cryptographic boundary. The exact suffix remains the final check.
    text.as_bytes()
        .iter()
        .fold(0xcbf2_9ce4_8422_2325, |hash, byte| {
            (hash ^ u64::from(*byte)).wrapping_mul(0x0000_0100_0000_01b3)
        })
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

        assert!(snapshot.matches_source_focus_and_epoch(
            VisibleTailSource::DaemonWordBuffer,
            Some("/ime/focus"),
            0
        ));
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

        assert!(!snapshot.matches_source_focus_and_epoch(
            VisibleTailSource::DaemonWordBuffer,
            Some("/ime/focus-b"),
            0
        ));
        assert!(!snapshot.matches_source_focus_and_epoch(
            VisibleTailSource::DaemonWordBuffer,
            Some("/ime/focus-a"),
            1
        ));
    }

    #[test]
    fn snapshot_identity_keeps_runtime_coordinates_and_tail_hash() {
        let snapshot = VisibleTailSnapshot::new(
            VisibleTailSource::ImeActiveComposition,
            "провер",
            Some("/ime/focus".to_string()),
            9,
        )
        .with_runtime_coordinates(Some(7), None, Some(12), Some(4));

        let identity = snapshot.identity();
        assert_eq!(identity.source, VisibleTailSource::ImeActiveComposition);
        assert_eq!(identity.revision, 9);
        assert_eq!(identity.caret, Some(7));
        assert_eq!(identity.composition_generation, Some(12));
        assert_eq!(identity.layout_epoch, Some(4));
        assert_ne!(identity.visible_tail_hash, 0);
    }
}
