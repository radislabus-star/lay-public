use super::output::EngineOutput;
use std::time::Instant;
use zbus::fdo;

use lay::typing_cpu::{
    is_command_like_long_tail, preedit_suffix_context_and_word, select_ime_candidate_proposals,
    ImeCandidateProposal, ImeCandidateReadoutRequest,
};
use lay::word_reader::split_last_alphabetic_token;

#[cfg(test)]
use lay::typing_cpu::push_unique_suffix;
#[cfg(test)]
use lay::typing_cpu::{is_allowed_visible_completion_suffix, phrase_candidate_suffix};

use super::engine::{InputFrameIdentity, LayIbusEngine};
use super::precognition_worker::PrecognitionWork;
use super::text::{make_ibus_text, make_preedit_ibus_text};
use super::trace;

const PREEDIT_TAIL_LIMIT: usize = 160;
const PREEDIT_TOKEN_LIMIT: usize = 32;
const PREEDIT_RU_WAVE_CANDIDATE_LIMIT: usize = 12;
const PREEDIT_RU_PREFIX_MIN_CHARS: usize = 1;
const PREEDIT_VISIBLE_PREFIX_MIN_CHARS: usize = 3;
#[cfg(test)]
const PREEDIT_PROBE_SYMBOL: &str = "*";
const PREEDIT_MODE_CLEAR: u32 = 0;

include!("preedit_readout.rs");

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct PrecognitionInput {
    tail: String,
    context_prefix: String,
    partial: String,
    max_suffix_chars: usize,
    active_composition: bool,
    correction_safety: lay::config::CorrectionSafety,
    declined_target_surfaces: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct ObservedPrediction {
    typed_prefix: String,
    target_surface: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ObservedPredictionOutcome {
    ConfirmedAttested,
    EndingChanged,
    DivergedAfterPrefix,
    MatchedUnattested,
    Censored,
}

impl ObservedPredictionOutcome {
    const fn trace_status(self) -> &'static str {
        match self {
            Self::ConfirmedAttested => "confirmed_attested",
            Self::EndingChanged => "ending_changed",
            Self::DivergedAfterPrefix => "diverged_after_prefix_censored",
            Self::MatchedUnattested => "matched_unattested",
            Self::Censored => "rejected_censored",
        }
    }
}

fn observed_prediction_outcome(
    typed_prefix: &str,
    predicted_word: &str,
    observed_word: &str,
    observed_is_attested: bool,
) -> ObservedPredictionOutcome {
    if predicted_word == observed_word {
        return if observed_is_attested {
            ObservedPredictionOutcome::ConfirmedAttested
        } else {
            ObservedPredictionOutcome::MatchedUnattested
        };
    }
    if !observed_is_attested
        || !predicted_word.starts_with(typed_prefix)
        || !observed_word.starts_with(typed_prefix)
    {
        return ObservedPredictionOutcome::Censored;
    }
    if lay::typing_cpu::TypingCpu::completion_edit_geometry_is_linked(
        typed_prefix,
        predicted_word,
        observed_word,
    ) {
        ObservedPredictionOutcome::EndingChanged
    } else {
        ObservedPredictionOutcome::DivergedAfterPrefix
    }
}

#[derive(Clone, Debug, Default)]
pub(crate) struct PreeditFastState {
    token: String,
    target_surface: Option<String>,
    declined_target_surfaces: Vec<String>,
    observed_prediction: Option<ObservedPrediction>,
}

impl PreeditFastState {
    pub(crate) fn reset(&mut self) {
        self.token.clear();
        self.target_surface = None;
        self.declined_target_surfaces.clear();
        self.observed_prediction = None;
    }

    pub(crate) fn push(&mut self, ch: char) {
        if ch.is_whitespace()
            || ch.is_ascii_punctuation() && !self.ascii_layout_symbol_continues_token(ch)
        {
            self.reset();
            return;
        }
        let target_still_matches = self.target_surface.as_deref().is_some_and(|target| {
            let mut continued_token = self.token.clone();
            continued_token.push(ch);
            target
                .to_lowercase()
                .starts_with(&continued_token.to_lowercase())
        });
        // Typing the next suggested character confirms the same target and
        // only shortens its visible suffix. A divergent character declines it.
        if !target_still_matches {
            if let Some(target) = self.target_surface.take() {
                if !self
                    .declined_target_surfaces
                    .iter()
                    .any(|declined| declined == &target)
                {
                    if self.declined_target_surfaces.len() >= PREEDIT_TOKEN_LIMIT {
                        self.declined_target_surfaces.remove(0);
                    }
                    self.declined_target_surfaces.push(target);
                }
            }
        }
        self.token.push(ch);
        trim_tail_buffer_to(&mut self.token, PREEDIT_TOKEN_LIMIT);
    }

    pub(crate) fn backspace(&mut self) {
        self.token.pop();
    }

    fn target_surface(&self) -> Option<&str> {
        self.target_surface.as_deref()
    }

    fn remember_target(&mut self, target: Option<String>) {
        self.target_surface = target;
    }

    /// Keep the first visible full-word prediction until the word boundary.
    /// Readout may change its suffix while the user keeps typing, but that
    /// initial target is the prediction whose outcome must be learned.
    fn observe_prediction_target(&mut self, typed_prefix: &str, target: Option<String>) {
        if self.observed_prediction.is_none() {
            self.observed_prediction =
                target
                    .filter(|target| !target.is_empty())
                    .map(|target_surface| ObservedPrediction {
                        typed_prefix: typed_prefix.to_lowercase(),
                        target_surface,
                    });
        }
    }

    fn observed_prediction_target(&self) -> Option<&str> {
        self.observed_prediction
            .as_ref()
            .map(|prediction| prediction.target_surface.as_str())
    }

    fn clear_target(&mut self) {
        self.target_surface = None;
    }

    fn ascii_layout_symbol_continues_token(&self, ch: char) -> bool {
        lay::typing_cpu::is_ascii_layout_letter_symbol(ch)
            && (self.token.is_empty()
                || self.token.chars().all(|current| {
                    current.is_ascii_alphabetic()
                        || lay::typing_cpu::is_ascii_layout_letter_symbol(current)
                }))
    }

    fn is_ascii_live_candidate_token(&self) -> bool {
        !self.token.is_empty()
            && self.token.chars().any(|ch| ch.is_ascii_alphabetic())
            && self.token.chars().all(|ch| {
                ch.is_ascii_alphabetic() || lay::typing_cpu::is_ascii_layout_letter_symbol(ch)
            })
    }

    pub(crate) fn clear_candidate_tracking(&mut self) {
        self.target_surface = None;
        self.declined_target_surfaces.clear();
        self.observed_prediction = None;
    }

    #[cfg(test)]
    pub(crate) fn token(&self) -> &str {
        &self.token
    }
}

impl LayIbusEngine {
    pub(super) async fn refresh_precognition_after_visible_input(
        &mut self,
        emitter: &mut EngineOutput<'_, '_>,
        frame: Option<InputFrameIdentity>,
    ) -> fdo::Result<()> {
        if self.preedit_waits_for_cursor_ack() {
            self.preedit_dirty = true;
            self.pending_display_frame = frame;
            return Ok(());
        }
        self.preedit_dirty = false;
        self.pending_display_frame = None;
        self.begin_pending_precognition_refresh();
        if !self.schedule_background_precognition(emitter, frame) {
            self.clear_preedit(emitter).await?;
        }
        Ok(())
    }

    pub(super) async fn flush_dirty_preedit(
        &mut self,
        emitter: &mut EngineOutput<'_, '_>,
    ) -> fdo::Result<()> {
        if !self.preedit_dirty {
            return Ok(());
        }
        self.preedit_dirty = false;
        let frame = self.pending_display_frame.take();
        self.begin_pending_precognition_refresh();
        if !self.schedule_background_precognition(emitter, frame) {
            self.clear_preedit(emitter).await?;
        }
        Ok(())
    }

    pub(super) async fn update_precognition_preedit(
        &mut self,
        emitter: &mut EngineOutput<'_, '_>,
    ) -> fdo::Result<()> {
        if !self.precognition_preedit_enabled() {
            return self.clear_preedit(emitter).await;
        }
        self.refresh_precognition_candidates();
        let Some(suffix) = self
            .preedit_candidates
            .get(self.preedit_candidate_index)
            .cloned()
        else {
            return self.clear_preedit(emitter).await;
        };
        self.preedit_suffix = suffix;
        let (preedit_text, cursor_pos) = self.inactive_preedit_payload();
        self.publish_preedit_payload(emitter, preedit_text, cursor_pos)
            .await
    }

    pub(crate) async fn clear_preedit(
        &mut self,
        emitter: &mut EngineOutput<'_, '_>,
    ) -> fdo::Result<()> {
        let was_visible = self.preedit_clear_needed();
        self.preedit_suffix.clear();
        self.preedit_candidates.clear();
        self.preedit_replacement_targets.clear();
        self.preedit_candidate_index = 0;
        self.preedit_fast.clear_target();
        if !was_visible {
            return Ok(());
        }
        trace::record_preedit("clear", false, 0, 0, None);
        emitter
            .update_preedit_text(make_ibus_text(String::new()), 0, false, PREEDIT_MODE_CLEAR)
            .await
            .map_err(|e| fdo::Error::Failed(e.to_string()))?;
        emitter
            .hide_preedit_text()
            .await
            .map_err(|e| fdo::Error::Failed(e.to_string()))?;
        self.preedit_visible = false;
        Ok(())
    }

    pub(super) fn close_precognition_word_boundary(&mut self) {
        self.cancel_precognition_display_generation();
        self.clear_preedit_completion_state();
        self.preedit_fast.reset();
    }

    pub(super) fn cancel_precognition_display_generation(&mut self) {
        super::precognition_worker::cancel();
        self.preedit_dirty = false;
        self.pending_display_frame = None;
    }

    pub(super) fn invalidate_input_frame_background_work(&mut self) {
        self.cancel_precognition_display_generation();
        self.invalidate_space_autocorrect_path();
    }

    fn preedit_clear_needed(&self) -> bool {
        self.preedit_visible
    }

    pub(super) async fn update_composition_preedit(
        &mut self,
        emitter: &mut EngineOutput<'_, '_>,
    ) -> fdo::Result<()> {
        if self.buffer.is_empty() {
            return self.clear_preedit(emitter).await;
        }
        self.refresh_precognition_candidates();
        let (text, cursor_pos) = self.composition_preedit_payload();
        self.publish_preedit_payload(emitter, text, cursor_pos)
            .await
    }

    pub(super) async fn update_composition_preedit_after_visible_input(
        &mut self,
        emitter: &mut EngineOutput<'_, '_>,
        frame: Option<InputFrameIdentity>,
    ) -> fdo::Result<()> {
        if !self.precognition_preedit_enabled() {
            return self.clear_preedit(emitter).await;
        }
        if self.buffer.is_empty() {
            return self.clear_preedit(emitter).await;
        }

        self.clear_visible_precognition_candidates();
        let cursor_pos = self.composition_cursor.min(self.buffer.chars().count()) as u32;
        self.publish_preedit_payload(emitter, self.buffer.clone(), cursor_pos)
            .await?;
        let _ = self.schedule_background_precognition(emitter, frame);
        Ok(())
    }

    async fn publish_preedit_payload(
        &mut self,
        emitter: &mut EngineOutput<'_, '_>,
        text: String,
        cursor_pos: u32,
    ) -> fdo::Result<()> {
        let show_transition = !self.preedit_visible;
        // UpdatePreeditText owns the visible frame. Install the new payload
        // before ShowPreeditText so a client cannot expose an empty or stale
        // frame while a previous completion is being replaced.
        emitter
            .update_preedit_text(
                make_preedit_ibus_text(text.clone()),
                cursor_pos,
                true,
                PREEDIT_MODE_CLEAR,
            )
            .await
            .map_err(|e| fdo::Error::Failed(e.to_string()))?;
        let sensitive = self.content_is_sensitive();
        let trace_text = (!sensitive).then_some(text.as_str());
        let trace_chars = if sensitive { 0 } else { text.chars().count() };
        let trace_cursor = if sensitive { 0 } else { cursor_pos };
        trace::record_preedit("update", true, trace_chars, trace_cursor, trace_text);
        if show_transition {
            emitter
                .show_preedit_text()
                .await
                .map_err(|e| fdo::Error::Failed(e.to_string()))?;
            trace::record_preedit("show", true, trace_chars, trace_cursor, trace_text);
        }
        self.preedit_visible = true;
        Ok(())
    }

    pub(super) fn precognition_suffix(&self) -> Option<String> {
        self.precognition_suffix_candidates().into_iter().next()
    }

    fn inactive_preedit_payload(&self) -> (String, u32) {
        let suffix = if self.preedit_suffix == "*" {
            String::new()
        } else {
            self.preedit_suffix.clone()
        };
        (self.visible_precognition_suffix(suffix), 0)
    }

    fn composition_preedit_payload(&mut self) -> (String, u32) {
        if let Some(replacement) = self
            .selected_precognition_replacement()
            .map(ToOwned::to_owned)
        {
            self.preedit_suffix.clear();
            return (replacement.clone(), replacement.chars().count() as u32);
        }
        let cursor_pos = self.composition_cursor.min(self.buffer.chars().count()) as u32;
        let suffix = if cursor_pos as usize == self.buffer.chars().count() {
            self.selected_visible_completion_suffix()
        } else {
            String::new()
        };
        self.preedit_suffix = suffix.clone();
        let mut text = self.buffer.clone();
        text.push_str(&self.visible_precognition_suffix(suffix));
        (text, cursor_pos)
    }

    fn visible_precognition_suffix(&self, suffix: String) -> String {
        if suffix.is_empty() || !self.config.ime_bracket_candidates {
            return suffix;
        }
        format!("[{suffix}]")
    }

    pub(super) fn selected_precognition_suffix(&self) -> Option<String> {
        self.preedit_candidates
            .get(self.preedit_candidate_index)
            .cloned()
            .or_else(|| self.precognition_suffix())
    }

    pub(super) fn selected_precognition_replacement(&self) -> Option<&str> {
        self.preedit_replacement_targets
            .get(self.preedit_candidate_index)
            .and_then(|target| target.as_deref())
    }

    pub(super) fn refresh_precognition_candidates(&mut self) {
        let proposals = self.precognition_candidates();
        self.install_precognition_candidates(proposals);
    }

    fn install_precognition_candidates(&mut self, proposals: Vec<ImeCandidateProposal>) {
        let proposals = proposals
            .into_iter()
            .filter(|proposal| !proposal.is_replacement())
            .collect::<Vec<_>>();
        let partial = self.live_candidate_partial();
        self.preedit_replacement_targets = proposals
            .iter()
            .map(|proposal| proposal.replacement.clone())
            .collect();
        self.preedit_candidates = proposals
            .into_iter()
            .map(|proposal| proposal.suffix)
            .collect();
        self.preedit_candidate_index = stable_candidate_index(
            self.preedit_fast.target_surface(),
            &partial,
            &self.preedit_candidates,
            &self.preedit_replacement_targets,
        );
        self.remember_selected_target(&partial);
    }

    fn clear_visible_precognition_candidates(&mut self) {
        self.preedit_suffix.clear();
        self.preedit_candidates.clear();
        self.preedit_replacement_targets.clear();
        self.preedit_candidate_index = 0;
    }

    fn begin_pending_precognition_refresh(&mut self) {
        // The previous surface remains visible until the matching worker result
        // replaces or hides it. Its candidates are invalidated immediately, so
        // Tab can never accept a stale completion during that short interval.
        self.clear_visible_precognition_candidates();
    }

    fn schedule_background_precognition(
        &self,
        emitter: &mut EngineOutput<'_, '_>,
        identity: Option<InputFrameIdentity>,
    ) -> bool {
        let Some(identity) = identity else {
            super::precognition_worker::cancel();
            return false;
        };
        if !self.input_frame_identity_matches(&identity) {
            super::precognition_worker::cancel();
            return false;
        }
        if !self.precognition_display_ready() {
            super::precognition_worker::cancel();
            return false;
        }
        let Some(input) = self.precognition_input() else {
            super::precognition_worker::cancel();
            return false;
        };
        let Some(connection) = emitter.connection() else {
            super::precognition_worker::cancel();
            return false;
        };
        super::precognition_worker::schedule(PrecognitionWork {
            identity,
            input,
            connection: connection.clone(),
            scheduled_at: Instant::now(),
        });
        true
    }

    fn precognition_display_ready(&self) -> bool {
        self.live_candidate_partial().chars().count() >= PREEDIT_VISIBLE_PREFIX_MIN_CHARS
    }

    pub(crate) fn precognition_identity_matches(&self, expected: &InputFrameIdentity) -> bool {
        self.input_frame_identity_matches(expected)
    }

    pub(super) fn capture_input_frame_identity(&self) -> Option<InputFrameIdentity> {
        let committed_tail = self.tail_buffer.clone();
        let trimmed_tail = committed_tail.trim_end();
        if trimmed_tail.is_empty() {
            return None;
        }
        let fallback_token = self.last_tail_token_text();
        let (context_prefix, observed_token) =
            self.live_word_readout_input(trimmed_tail).or_else(|| {
                (!fallback_token.is_empty()).then(|| {
                    let context = trimmed_tail
                        .strip_suffix(fallback_token.as_str())
                        .unwrap_or_default();
                    (context, fallback_token.as_str())
                })
            })?;
        let context_prefix = context_prefix.to_string();
        let observed_token = observed_token.to_string();
        let source_scalar_count = u32::try_from(observed_token.chars().count()).ok()?;
        let identity = InputFrameIdentity::new_authoritative(
            self.path.clone(),
            self.focus_receipt.clone(),
            self.tail_epoch,
            committed_tail,
            context_prefix.clone(),
            observed_token.clone(),
            self.live_completion_input_is_active(),
            self.layout_is_ru,
            self.factory_engine_profile,
            self.output_capability_fingerprint(),
            &self.config,
        );
        let (caret_scalar, preedit, preedit_cursor_scalar) = if self.buffer.is_empty() {
            (source_scalar_count, String::new(), 0)
        } else {
            if self.buffer.as_bytes() != observed_token.as_bytes() {
                return Some(identity);
            }
            let cursor =
                u32::try_from(self.composition_cursor.min(self.buffer.chars().count())).ok()?;
            (cursor, self.buffer.clone(), cursor)
        };
        let coordinates = lay::lexical_authority_frame::LexicalAuthorityCoordinatesV1::new(
            self.runtime_owner_lease_identity,
            [self.runtime_owner_lease_identity, self.tail_epoch.max(1)],
            self.focus_serial,
            observed_token.clone(),
            context_prefix.clone(),
            caret_scalar,
            (caret_scalar, caret_scalar),
            preedit,
            preedit_cursor_scalar,
            self.layout_generation,
            identity.config.identity_fingerprint(),
        );
        Some(identity.with_lexical_coordinates(coordinates))
    }

    pub(super) fn input_frame_authority_matches(&self, expected: &InputFrameIdentity) -> bool {
        self.path == expected.path
            && self.focus_receipt == expected.focus_receipt
            && self.focus_serial == expected.lexical_coordinates.as_ref().map_or(
                self.focus_serial,
                lay::lexical_authority_frame::LexicalAuthorityCoordinatesV1::focus_serial,
            )
            && self.runtime_owner_lease_identity
                == expected.lexical_coordinates.as_ref().map_or(
                    self.runtime_owner_lease_identity,
                    lay::lexical_authority_frame::LexicalAuthorityCoordinatesV1::runtime_owner_lease_identity,
                )
            && self.tail_epoch == expected.tail_epoch
            && self.tail_buffer == expected.committed_tail
            && self.layout_is_ru == expected.active_layout_is_ru
            && expected.lexical_coordinates.as_ref().is_none_or(|coordinates| {
                coordinates.layout_generation() == self.layout_generation
            })
            && self.factory_engine_profile == expected.factory_engine_profile
            && self.output_capability_fingerprint() == expected.output_capability_fingerprint
            && expected.config_matches(&self.config)
            && self
                .shared
                .lock()
                .is_ok_and(|state| state.active_path.as_deref() == Some(self.path.as_str()))
    }

    fn output_capability_fingerprint(&self) -> u64 {
        let mut fingerprint = 0xcbf2_9ce4_8422_2325_u64;
        for byte in self
            .cursor_cell_width
            .to_le_bytes()
            .into_iter()
            .chain([u8::from(self.surrounding_text_supported)])
            .chain([u8::from(self.managed_input)])
            .chain([u8::from(self.atomic_route_active)])
        {
            fingerprint ^= u64::from(byte);
            fingerprint = fingerprint.wrapping_mul(0x100_0000_01b3);
        }
        fingerprint
    }

    pub(super) fn input_frame_identity_matches(&self, expected: &InputFrameIdentity) -> bool {
        self.input_frame_authority_matches(expected)
            && self
                .capture_input_frame_identity()
                .as_ref()
                .is_some_and(|current| current == expected)
    }

    pub(crate) async fn apply_background_precognition(
        &mut self,
        emitter: &mut EngineOutput<'_, '_>,
        proposals: Vec<ImeCandidateProposal>,
    ) -> fdo::Result<()> {
        self.install_precognition_candidates(proposals);
        if self.buffer.is_empty() {
            let Some(candidate) = self
                .preedit_candidates
                .get(self.preedit_candidate_index)
                .cloned()
            else {
                return self.clear_preedit(emitter).await;
            };
            self.preedit_suffix = candidate;
            let (preedit_text, cursor_pos) = self.inactive_preedit_payload();
            return self
                .publish_preedit_payload(emitter, preedit_text, cursor_pos)
                .await;
        }

        if self.preedit_candidates.is_empty() {
            self.preedit_suffix.clear();
            let cursor_pos = self.composition_cursor.min(self.buffer.chars().count()) as u32;
            return self
                .publish_preedit_payload(emitter, self.buffer.clone(), cursor_pos)
                .await;
        }
        let (text, cursor_pos) = self.composition_preedit_payload();
        self.publish_preedit_payload(emitter, text, cursor_pos)
            .await
    }

    pub(super) fn cycle_precognition_candidate(&mut self, step: isize) -> bool {
        self.refresh_precognition_candidates();
        self.advance_precognition_candidate(step)
    }

    fn advance_precognition_candidate(&mut self, step: isize) -> bool {
        let len = self.preedit_candidates.len();
        if len < 2 {
            return false;
        }
        let len = len as isize;
        self.preedit_candidate_index =
            (self.preedit_candidate_index as isize + step).rem_euclid(len) as usize;
        let partial = self.live_candidate_partial();
        self.remember_selected_target(&partial);
        true
    }

    fn remember_selected_target(&mut self, partial: &str) {
        let target = self
            .selected_precognition_replacement()
            .map(ToOwned::to_owned)
            .or_else(|| {
                self.preedit_candidates
                    .get(self.preedit_candidate_index)
                    .map(|suffix| format!("{partial}{suffix}"))
            });
        self.preedit_fast
            .observe_prediction_target(partial, target.clone());
        self.preedit_fast.remember_target(target);
    }

    fn live_candidate_partial(&self) -> String {
        if self.preedit_fast.is_ascii_live_candidate_token() {
            return self.preedit_fast.token.to_lowercase();
        }
        split_last_alphabetic_token(self.tail_buffer.trim_end())
            .map(|(_, token)| token.to_lowercase())
            .unwrap_or_default()
    }

    fn precognition_input(&self) -> Option<PrecognitionInput> {
        if !self.precognition_preedit_enabled()
            || self.buffer.is_empty() && self.tail_buffer.ends_with(char::is_whitespace)
        {
            return None;
        }
        if self.buffer.is_empty()
            && self
                .tail_buffer
                .trim_end()
                .chars()
                .last()
                .is_some_and(is_hard_precognition_boundary)
            && !self.preedit_fast.is_ascii_live_candidate_token()
        {
            return None;
        }
        let tail = self.tail_buffer.clone();
        let (context_prefix, partial) = {
            let trimmed = tail.trim_end();
            if is_command_like_long_tail(trimmed) {
                return None;
            }
            let (context_prefix, partial) = self.live_word_readout_input(trimmed)?;
            (context_prefix.to_string(), partial.to_lowercase())
        };
        if partial.is_empty() {
            return None;
        }
        Some(PrecognitionInput {
            tail,
            context_prefix,
            partial,
            max_suffix_chars: self.precognition_max_suffix_chars(),
            active_composition: self.live_completion_input_is_active(),
            correction_safety: self.config.active_correction_safety(),
            declined_target_surfaces: self.preedit_fast.declined_target_surfaces.clone(),
        })
    }

    fn precognition_candidates(&self) -> Vec<ImeCandidateProposal> {
        self.precognition_input()
            .map(|input| materialize_precognition_candidates(&input))
            .unwrap_or_default()
    }

    fn precognition_suffix_candidates(&self) -> Vec<String> {
        self.precognition_candidates()
            .into_iter()
            .filter(|proposal| !proposal.is_replacement())
            .map(|proposal| proposal.suffix)
            .collect()
    }

    fn precognition_max_suffix_chars(&self) -> usize {
        match self.config.active_correction_safety() {
            lay::config::CorrectionSafety::Strict => 3,
            lay::config::CorrectionSafety::Normal => 16,
            lay::config::CorrectionSafety::Experimental => 24,
        }
    }

    fn precognition_preedit_enabled(&self) -> bool {
        self.config.active_nanda_precognition() && self.content_allows_text_assistance()
    }

    pub(super) fn push_tail_char(&mut self, ch: char) {
        if self.content_is_sensitive() {
            self.close_committed_tail_field();
            return;
        }
        let is_boundary = ch.is_whitespace()
            || is_hard_precognition_boundary(ch)
                && !self.preedit_fast.ascii_layout_symbol_continues_token(ch);
        let tail_before_boundary = is_boundary.then(|| self.tail_buffer.clone());
        // Whitespace resets the fast preedit state. Preserve the prediction
        // first so the boundary can turn it into supervised feedback.
        let prediction_before_boundary = is_boundary
            .then(|| self.preedit_fast.observed_prediction.clone())
            .flatten();
        self.surrounding_text_snapshot = None;
        self.tail_buffer.push(ch);
        self.preedit_fast.push(ch);
        self.last_tail_input_at = Some(Instant::now());
        if is_boundary {
            let completion_edit_finalized = tail_before_boundary
                .as_deref()
                .is_some_and(|tail| self.finalize_pending_ime_completion_edit(tail));
            if let Some(tail_before_boundary) = tail_before_boundary.as_deref() {
                if !completion_edit_finalized {
                    self.record_precognition_outcome_at_boundary(
                        tail_before_boundary,
                        prediction_before_boundary.as_ref(),
                    );
                }
            }
            self.close_precognition_word_boundary();
            if ch.is_whitespace() {
                self.word_input_mode = None;
            }
        }
        trim_tail_buffer(&mut self.tail_buffer);
        self.publish_tail_handoff();
    }

    fn record_precognition_outcome_at_boundary(
        &self,
        tail_before_boundary: &str,
        prediction_before_boundary: Option<&ObservedPrediction>,
    ) {
        let Some((prefix, observed_word)) = split_last_alphabetic_token(tail_before_boundary)
        else {
            return;
        };
        let observed_word = observed_word.to_lowercase();
        let context = lay::nanda_wave::llmwave::tokenize(prefix);
        let fallback_prediction;
        let prediction = if let Some(prediction) = prediction_before_boundary {
            prediction
        } else {
            let suffix = self.selected_visible_completion_suffix();
            let Some((_, target_surface)) =
                preedit_suffix_context_and_word(tail_before_boundary, &suffix)
            else {
                return;
            };
            let typed_prefix = target_surface
                .strip_suffix(&suffix)
                .unwrap_or(&observed_word)
                .to_lowercase();
            fallback_prediction = ObservedPrediction {
                typed_prefix,
                target_surface,
            };
            &fallback_prediction
        };
        let predicted_word = prediction.target_surface.to_lowercase();
        let typed_prefix = prediction.typed_prefix.to_lowercase();
        if predicted_word.is_empty() || typed_prefix.is_empty() {
            return;
        }
        let outcome = observed_prediction_outcome(
            &typed_prefix,
            &predicted_word,
            &observed_word,
            lay::typing_cpu::TypingCpu::learning_target_is_attested(&observed_word),
        );
        match outcome {
            ObservedPredictionOutcome::ConfirmedAttested => {
                lay::typing_cpu::TypingCpu::record_confirmed_completion_prediction(
                    &context.join(" "),
                    &predicted_word,
                );
            }
            ObservedPredictionOutcome::EndingChanged => {
                lay::typing_cpu::TypingCpu::record_edited_completion(
                    &context.join(" "),
                    &typed_prefix,
                    &predicted_word,
                    &observed_word,
                );
            }
            ObservedPredictionOutcome::DivergedAfterPrefix
            | ObservedPredictionOutcome::MatchedUnattested
            | ObservedPredictionOutcome::Censored => {
                // Absence of Tab and an unrelated continuation are censored.
                // Neither is evidence against the lemma or visible suggestion.
            }
        }
        let status = outcome.trace_status();
        trace::record(format!(
            r#"{{"kind":"ibus_prediction_outcome","status":{},"typed_prefix":{},"suggested":{},"final":{}}}"#,
            serde_json::to_string(status).unwrap_or_else(|_| "\"rejected_censored\"".to_string()),
            serde_json::to_string(&typed_prefix).unwrap_or_else(|_| "\"\"".to_string()),
            serde_json::to_string(&predicted_word).unwrap_or_else(|_| "\"\"".to_string()),
            serde_json::to_string(&observed_word).unwrap_or_else(|_| "\"\"".to_string()),
        ));
    }

    #[cfg(test)]
    fn preedit_text_for_client(&self) -> (String, u32) {
        self.inactive_preedit_payload()
    }
}

fn candidate_index_for_target(
    previous_target: &str,
    partial: &str,
    candidates: &[String],
    replacements: &[Option<String>],
) -> Option<usize> {
    if let Some(index) = replacements.iter().position(|replacement| {
        replacement
            .as_deref()
            .is_some_and(|replacement| replacement == previous_target)
    }) {
        return Some(index);
    }
    let expected_suffix = previous_target.strip_prefix(partial)?;
    candidates
        .iter()
        .position(|candidate| candidate == expected_suffix)
}

fn stable_candidate_index(
    previous_target: Option<&str>,
    partial: &str,
    candidates: &[String],
    replacements: &[Option<String>],
) -> usize {
    previous_target
        .and_then(|target| candidate_index_for_target(target, partial, candidates, replacements))
        .unwrap_or(0)
}

#[cfg(test)]
fn push_unique_ru_known_suffix(
    candidates: &mut Vec<String>,
    partial: &str,
    word: &str,
    suffix: Option<String>,
) {
    let Some(suffix) = suffix else {
        return;
    };
    if suffix.is_empty() || candidates.iter().any(|candidate| candidate == &suffix) {
        return;
    }
    let single_cyrillic =
        suffix.chars().count() == 1 && suffix.chars().all(|ch| matches!(ch, 'а'..='я' | 'ё'));
    let strong_single_letter_completion = single_cyrillic
        && partial.chars().count() >= 3
        && !is_ime_complete_russian_word(partial)
        && is_ime_candidate_russian_word(word);
    if strong_single_letter_completion || is_allowed_visible_completion_suffix(&suffix) {
        candidates.push(suffix);
    }
}

#[cfg(test)]
fn is_ime_complete_russian_word(word: &str) -> bool {
    lay::lexicon::is_common_ru_word(word)
        || (word.chars().count() >= 5 && lay::russian_lexicon::is_known_russian_word_or_form(word))
}

#[cfg(test)]
fn is_ime_candidate_russian_word(word: &str) -> bool {
    lay::lexicon::is_common_ru_word(word)
        || lay::russian_lexicon::is_known_russian_word_or_form(word)
}

fn trim_tail_buffer(buffer: &mut String) {
    trim_tail_buffer_to(buffer, PREEDIT_TAIL_LIMIT);
}

fn trim_tail_buffer_to(buffer: &mut String, limit: usize) {
    let chars = buffer.chars().count();
    if chars <= limit {
        return;
    }
    let skip = chars - limit;
    let byte_idx = buffer
        .char_indices()
        .nth(skip)
        .map(|(idx, _)| idx)
        .unwrap_or(0);
    buffer.drain(..byte_idx);
}

fn is_hard_precognition_boundary(ch: char) -> bool {
    matches!(
        ch,
        '.' | ',' | '!' | '?' | ':' | ';' | ')' | ']' | '}' | '…'
    )
}

#[cfg(test)]
#[path = "preedit/tests.rs"]
mod tests;
