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

#[cfg(test)]
mod tests {
    use super::VisibleTailSource;

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
}
