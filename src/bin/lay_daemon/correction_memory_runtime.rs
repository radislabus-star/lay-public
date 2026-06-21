use lay::keyboard::KeyEvent;
use lay::text_edit::TextReplacement;
use lay::typing_assist::typing_correction_should_skip_auto_undo;
use lay::word_buffer::WordBuffer;

pub(super) struct ManualTextCorrectionMemory<'a> {
    pub(super) events: &'a [KeyEvent],
    pub(super) plan: &'a TextReplacement,
    pub(super) original: &'a str,
    pub(super) replacement: &'a str,
    pub(super) kind: &'a str,
    pub(super) replace_words: usize,
    pub(super) words: usize,
    pub(super) inserted_layout_is_ru: Option<bool>,
}

pub(super) fn remember_manual_text_correction(
    buf: &mut WordBuffer,
    correction: ManualTextCorrectionMemory<'_>,
) {
    buf.remember_pending_learning_correction(
        correction.kind,
        correction.original,
        correction.replacement,
        correction.replace_words,
        correction.words,
    );
    let remembered = buf.remember_replacement_last_word_for_replay(
        correction.events,
        correction.plan,
        correction.replacement,
    ) || correction
        .inserted_layout_is_ru
        .is_some_and(|layout_is_ru| {
            buf.remember_inserted_tail_for_replay(correction.events, correction.plan, layout_is_ru)
        })
        || (correction.inserted_layout_is_ru.is_some()
            && buf.remember_inserted_last_word_for_replay(correction.events, correction.plan));
    if !remembered {
        buf.reset_all();
    }
    remember_pending_auto_undo(buf, &correction);
}

pub(super) struct AssistedCorrectionMemory<'a> {
    pub(super) events: &'a [KeyEvent],
    pub(super) plan: &'a TextReplacement,
    pub(super) original: &'a str,
    pub(super) replacement: &'a str,
    pub(super) kind: &'a str,
    pub(super) rule_id: Option<&'a str>,
    pub(super) replace_words: usize,
    pub(super) words: usize,
    pub(super) cursor_offset: u32,
}

pub(super) fn remember_assisted_text_correction(
    buf: &mut WordBuffer,
    correction: AssistedCorrectionMemory<'_>,
) {
    buf.remember_pending_learning_correction(
        correction.kind,
        correction.original,
        correction.replacement,
        correction.replace_words,
        correction.words,
    );
    let remembered = if correction.cursor_offset > 0 {
        buf.remember_visible_replacement_tail_for_replay(correction.events, correction.replacement)
    } else {
        buf.remember_replacement_last_word_for_replay(
            correction.events,
            correction.plan,
            correction.replacement,
        )
    };
    if !remembered && correction.cursor_offset == 0 {
        buf.reset_all();
    }
    if typing_correction_should_skip_auto_undo(
        correction.rule_id,
        correction.original(),
        correction.replacement(),
    ) {
        // Layout-only typing assists are intentionally not auto-undone on the
        // next edit, but explicit manual double-Shift remains a real user
        // command and must use the normal replay path.
    } else {
        remember_pending_auto_undo(buf, &correction);
    }
}

trait PendingUndoCorrection {
    fn kind(&self) -> &str;
    fn original(&self) -> &str;
    fn replacement(&self) -> &str;
    fn replace_words(&self) -> usize;
    fn words(&self) -> usize;
}

impl PendingUndoCorrection for ManualTextCorrectionMemory<'_> {
    fn kind(&self) -> &str {
        self.kind
    }

    fn original(&self) -> &str {
        self.original
    }

    fn replacement(&self) -> &str {
        self.replacement
    }

    fn replace_words(&self) -> usize {
        self.replace_words
    }

    fn words(&self) -> usize {
        self.words
    }
}

impl PendingUndoCorrection for AssistedCorrectionMemory<'_> {
    fn kind(&self) -> &str {
        self.kind
    }

    fn original(&self) -> &str {
        self.original
    }

    fn replacement(&self) -> &str {
        self.replacement
    }

    fn replace_words(&self) -> usize {
        self.replace_words
    }

    fn words(&self) -> usize {
        self.words
    }
}

fn remember_pending_auto_undo<T: PendingUndoCorrection>(buf: &mut WordBuffer, correction: &T) {
    buf.remember_pending_auto_undo(
        correction.kind(),
        correction.original(),
        correction.replacement(),
        correction.replace_words(),
        correction.words(),
    );
}
