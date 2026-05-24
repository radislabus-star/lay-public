use lay::decoder::{decode_typing_assist_tail, CorrectionSource, DecoderEditPlan};
use lay::keyboard::KeyEvent;
use lay::typing_context::completed_tail_context;
use lay::word_buffer::{WordBuffer, MAX_REPLACE_WORDS};

#[cfg(not(test))]
use super::super::active_typing_assist_pipeline_for_auto_replace;

pub(crate) struct TypingAssistCorrection {
    pub(crate) events: Vec<KeyEvent>,
    pub(crate) edit: DecoderEditPlan,
}

pub(crate) fn find_typing_assist_correction(
    buf: &WordBuffer,
    allow_layout_auto: bool,
) -> Option<TypingAssistCorrection> {
    completed_tail_scopes(buf)
        .into_iter()
        .find_map(|word_count| {
            let events = buf.last_completed_words_events(word_count)?;
            let context = completed_tail_context(buf, word_count, &events);
            #[cfg(test)]
            let pipeline = lay::typing_context::typing_assist_pipeline_for_context(
                true,
                lay::config::CorrectionSafety::Normal,
                &lay::config::default_typing_assist_pipeline(),
                &context,
            );
            #[cfg(not(test))]
            let pipeline = active_typing_assist_pipeline_for_auto_replace(&context);
            let edit = decode_typing_assist_tail(
                &events,
                allow_layout_auto,
                &pipeline,
                CorrectionSource::TypingAssist,
            )?;
            Some(TypingAssistCorrection { events, edit })
        })
}

fn completed_tail_scopes(buf: &WordBuffer) -> Vec<usize> {
    let max_scope = buf.prev_words_len().min(MAX_REPLACE_WORDS);
    (1..=max_scope).rev().collect()
}
