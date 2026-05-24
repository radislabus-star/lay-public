#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CorrectionTrigger {
    Manual,
    AfterSpace,
    AfterPunctuation,
    Enter,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CorrectionSource {
    Replay,
    SmartText,
    AutoReplace,
    TypingAssist,
    EnterAutocorrect,
}

impl CorrectionSource {
    pub fn log_kind(self) -> &'static str {
        match self {
            Self::Replay => "layout-replay",
            Self::SmartText => "smart-text",
            Self::AutoReplace => "auto-replace",
            Self::TypingAssist => "typing-assist",
            Self::EnterAutocorrect => "enter-autocorrect",
        }
    }

    pub fn needs_undo_checkpoint(self) -> bool {
        !matches!(self, Self::Replay)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DecoderAction {
    KeepOriginal,
    ReplayAll,
    ReplaceText {
        replacement: String,
        source: CorrectionSource,
    },
}

impl DecoderAction {
    pub fn replacement_text(&self) -> Option<&str> {
        match self {
            Self::ReplaceText { replacement, .. } => Some(replacement),
            Self::KeepOriginal | Self::ReplayAll => None,
        }
    }
}
