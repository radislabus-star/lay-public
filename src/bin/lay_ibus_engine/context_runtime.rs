use std::time::Instant;

use super::context_admission::{
    ActivationOutcome, AdmissionToken, EnginePath, KeyCallback, LayoutIntentToken,
    SettledWordState, WordCompleteness, WordScope,
};
use super::engine::{next_input_identity, LayIbusEngine};
use super::protocol::{
    has_command_modifier, is_accept_completion_with_space_key, is_key_press, is_shift_key,
    AutocorrectSuppression, KEY_BACKSPACE, KEY_DOWN, KEY_ENTER, KEY_KP_ENTER, KEY_LEFT, KEY_RIGHT,
    KEY_TAB, KEY_UP,
};
use super::trace;

/// Scoped to the bridge's existing exclusive engine guard. A cancelled or
/// failed output cannot leave its admission witness available to later work.
pub(super) struct ContextBridgeOutput<'a> {
    engine: &'a mut LayIbusEngine,
    completed: bool,
}

impl std::ops::Deref for ContextBridgeOutput<'_> {
    type Target = LayIbusEngine;
    fn deref(&self) -> &Self::Target {
        self.engine
    }
}

impl std::ops::DerefMut for ContextBridgeOutput<'_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.engine
    }
}

impl ContextBridgeOutput<'_> {
    pub(super) fn complete(&mut self) {
        self.completed = true;
    }
}

impl Drop for ContextBridgeOutput<'_> {
    fn drop(&mut self) {
        self.engine.context_bridge_token = None;
        if !self.completed {
            self.engine.revoke_context_word();
        }
    }
}

impl LayIbusEngine {
    pub(super) fn begin_context_bridge_output(
        &mut self,
        token: Option<&AdmissionToken>,
    ) -> ContextBridgeOutput<'_> {
        self.context_bridge_token = token.cloned();
        ContextBridgeOutput {
            engine: self,
            completed: false,
        }
    }

    pub(super) fn settle_context_bridge_output(&mut self) -> bool {
        let Some(token) = self.context_bridge_token.as_ref() else {
            return true;
        };
        let accepted = !self.atomic.speculation
            && self.context_word_scope.as_ref().is_some_and(|scope| {
                self.context_admission.as_ref().is_some_and(|admission| {
                    admission.settle_bridge_output(
                        token,
                        SettledWordState::from_scope(self.committed_tail.epoch, scope),
                    )
                })
            });
        if !accepted {
            self.revoke_context_word();
        }
        accepted
    }

    pub(super) async fn observe_context_focus_out(
        &mut self,
        header: &zbus::message::Header<'_>,
        callback_entered: Instant,
    ) -> bool {
        self.try_install_pending_context_activation();
        let Some(admission) = self.context_admission.clone() else {
            if self.context_admission_required {
                self.context_handoff_sealed = false;
                self.revoke_context_word();
            }
            return !self.context_admission_required;
        };
        let Some(owner) = self.context_owner.clone() else {
            self.context_handoff_sealed = false;
            self.revoke_context_word();
            return false;
        };
        let observed = match admission.observe_callback(header, callback_entered).await {
            Ok(observed) => observed,
            Err(_) => {
                self.context_handoff_sealed = false;
                self.revoke_context_word();
                return false;
            }
        };
        if admission.callback_is_stale_for(&owner, &observed) {
            return false;
        }
        if matches!(observed.header.member.as_str(), "FocusOut" | "FocusOutId")
            && admission.revocation_matches(&owner, &observed)
        {
            // Authenticated loss of focus still requires local cleanup. The
            // return value admits that lifecycle event, not a text handoff.
            self.context_handoff_sealed = false;
            self.revoke_context_word();
            return true;
        }
        let accepted = matches!(observed.header.member.as_str(), "FocusOut" | "FocusOutId")
            && admission.focus_out(&owner, &observed)
            && admission.seal_source(&owner, self.committed_tail.epoch, &observed);
        self.context_handoff_sealed = accepted;
        if !accepted {
            self.revoke_context_word();
        }
        accepted
    }

    pub(super) async fn observe_context_disable(
        &mut self,
        header: &zbus::message::Header<'_>,
        callback_entered: Instant,
    ) -> bool {
        self.try_install_pending_context_activation();
        let Some(admission) = self.context_admission.clone() else {
            if self.context_admission_required {
                self.context_handoff_sealed = false;
                self.revoke_context_word();
            }
            return !self.context_admission_required;
        };
        let Some(owner) = self.context_owner.clone() else {
            self.context_handoff_sealed = false;
            self.revoke_context_word();
            return false;
        };
        let observed = match admission.observe_callback(header, callback_entered).await {
            Ok(observed) => observed,
            Err(_) => {
                self.context_handoff_sealed = false;
                self.revoke_context_word();
                return false;
            }
        };
        if admission.callback_is_stale_for(&owner, &observed) {
            return false;
        }
        if observed.header.member == "Disable" && admission.revocation_matches(&owner, &observed) {
            self.context_handoff_sealed = false;
            self.revoke_context_word();
            return true;
        }
        let accepted = observed.header.member == "Disable" && admission.disable(&owner, &observed);
        if !accepted {
            self.context_handoff_sealed = false;
            self.revoke_context_word();
        }
        accepted
    }

    pub(super) async fn observe_context_revocation(
        &mut self,
        header: &zbus::message::Header<'_>,
        callback_entered: Instant,
    ) -> bool {
        if !self.context_admission_required {
            self.fail_context_activation_local();
            return true;
        }
        self.context_handoff_sealed = false;
        if let (Some(admission), Some(owner)) =
            (self.context_admission.clone(), self.context_owner.clone())
        {
            match admission.observe_callback(header, callback_entered).await {
                Ok(observed) if admission.callback_is_stale(&observed) => return false,
                Ok(observed) if admission.revocation_matches(&owner, &observed) => {
                    self.context_token = admission.current_token();
                }
                Ok(_) => return false,
                Err(_) if admission.current_owner().as_ref() != Some(&owner) => return false,
                Err(_) => {
                    admission.revoke_current_owner(&owner);
                    self.context_token = None;
                }
            }
        }
        if let Some(scope) = self.context_word_scope.as_mut() {
            scope.revoke_for_input_gap();
        }
        self.committed_tail.pending_completion_learning = None;
        self.clear_preedit_completion_state();
        true
    }

    pub(super) fn context_word_is_known(&self) -> bool {
        if !self.context_admission_required {
            return true;
        }
        let Some(admission) = self.context_admission.as_ref() else {
            return false;
        };
        self.context_word_scope.as_ref().is_some_and(|scope| {
            scope.lineage().completeness == WordCompleteness::KnownStart
                && self.context_token.as_ref().is_some_and(|token| {
                    token.matches_word_scope(scope) && admission.revalidate(token)
                })
        })
    }

    /// An unknown beginning can support a suffix suggestion and an explicit
    /// append, without authorizing replacement of the complete word.
    pub(super) fn context_observed_suffix_is_current(&self) -> bool {
        if self.context_handoff_sealed
            || self.atomic.active
            || !self.composition.buffer.is_empty()
            || self.content_is_sensitive()
            || self.committed_tail.buffer.ends_with(char::is_whitespace)
        {
            return false;
        }
        let Some(scope) = self.context_word_scope.as_ref() else {
            return false;
        };
        let token_text = self.last_tail_token_text();
        let chars = token_text.chars().count();
        if scope.lineage().completeness != WordCompleteness::UnknownStart
            || chars == 0
            || chars > scope.lineage().observed_suffix_chars as usize
            || !self.context_token.as_ref().is_some_and(|token| {
                token.matches_word_scope(scope)
                    && self
                        .context_admission
                        .as_ref()
                        .is_some_and(|admission| admission.revalidate(token))
            })
        {
            return false;
        }
        let Some(snapshot) = self.client_context.surrounding_text_snapshot.as_ref() else {
            return !self.client_context.surrounding_text_supported
                && self.terminal_committed_tail_executor_available();
        };
        if snapshot.has_selection()
            || snapshot.suffix_before_cursor(chars).as_deref() != Some(token_text.as_str())
        {
            return false;
        }
        let start = snapshot.cursor_pos as usize - chars;
        let left_is_boundary = start == 0
            || snapshot
                .text
                .chars()
                .nth(start - 1)
                .is_some_and(super::preedit::is_observed_word_boundary);
        let right_is_boundary = snapshot
            .text
            .chars()
            .nth(snapshot.cursor_pos as usize)
            .is_none_or(super::preedit::is_observed_word_boundary);
        // A fragment edge permits only suffix readout/append. It does not
        // establish the absolute beginning required for whole-word edits.
        left_is_boundary && right_is_boundary
    }

    /// Unknown word beginnings do not authorize generic edits.
    /// A terminal can still project its complete observed suffix on an explicit
    /// manual request, using the same settled context and erase geometry.
    pub(super) fn context_allows_manual_toggle(&self) -> bool {
        if self.context_word_is_known() {
            return true;
        }
        if self.context_handoff_sealed
            || !self.composition.buffer.is_empty()
            || self.content_is_sensitive()
            || !self.terminal_committed_tail_executor_available()
        {
            return false;
        }
        let Some(scope) = self.context_word_scope.as_ref() else {
            return false;
        };
        let suffix_chars = self.last_tail_token_text().chars().count();
        suffix_chars > 0
            && suffix_chars <= scope.lineage().observed_suffix_chars as usize
            && self.context_token.as_ref().is_some_and(|token| {
                token.matches_word_scope(scope)
                    && self
                        .context_admission
                        .as_ref()
                        .is_some_and(|admission| admission.revalidate(token))
            })
    }

    pub(super) fn live_context_token(&self) -> Option<AdmissionToken> {
        let admission = self.context_admission.as_ref()?;
        let token = self.context_token.as_ref()?;
        admission.revalidate(token).then(|| token.clone())
    }

    pub(super) fn live_layout_intent_token(&self) -> Option<LayoutIntentToken> {
        let admission = self.context_admission.as_ref()?;
        let owner = self.context_owner.as_ref()?;
        admission.current_layout_intent_token_for(owner)
    }

    pub(super) async fn activate_context_from_header(
        &mut self,
        header: &zbus::message::Header<'_>,
        callback_entered: Instant,
        native_context: Option<&str>,
    ) -> bool {
        let Some(admission) = self.context_admission.clone() else {
            if self.context_admission_required {
                self.fail_context_activation();
            }
            return false;
        };
        let Some(target_path) = EnginePath::new(self.path.clone()) else {
            self.fail_context_activation();
            return false;
        };
        let observed = match admission.observe_callback(header, callback_entered).await {
            Ok(observed) => observed,
            Err(_) => {
                self.fail_context_activation();
                return false;
            }
        };
        if native_context.is_some() {
            self.try_install_pending_context_activation();
        }
        if let (Some(owner), Some(context_path)) = (self.context_owner.as_ref(), native_context) {
            let Some(context) = admission.context_key(context_path.to_string()) else {
                self.fail_context_activation();
                return false;
            };
            if let Some(enriched) =
                admission.enrich_native_activation(owner, &target_path, &context, &observed)
            {
                return if enriched {
                    self.context_token = admission.current_token();
                    true
                } else {
                    self.fail_context_activation();
                    false
                };
            }
        }
        if let Some(context_path) = native_context {
            let Some(context) = admission.context_key(context_path.to_string()) else {
                self.fail_context_activation();
                return false;
            };
            if admission.enrich_pending_native_activation(&target_path, context, &observed) {
                self.fail_context_activation_local();
                return true;
            }
        }
        let started = if let Some(context_path) = native_context {
            admission
                .context_key(context_path.to_string())
                .ok_or(())
                .and_then(|context| {
                    admission
                        .start_native_activation(target_path, context, observed.position)
                        .map_err(|_| ())
                })
        } else {
            admission
                .start_compatibility_activation(target_path, observed.position)
                .map_err(|_| ())
        };
        if started.is_err() {
            self.fail_context_activation();
            return false;
        }
        // The adapter owns the one-shot Get/marker future. Until a later
        // callback consumes its target-bound result, this engine has no text
        // authority and all input follows the literal UnknownStart route.
        self.fail_context_activation_local();
        true
    }

    fn try_install_pending_context_activation(&mut self) {
        let Some(admission) = self.context_admission.clone() else {
            return;
        };
        let Some(target_path) = EnginePath::new(self.path.clone()) else {
            self.fail_context_activation();
            return;
        };
        if let Some(outcome) = admission.pending_activation_for(&target_path) {
            if self.install_context_activation(outcome.clone())
                && !admission.acknowledge_activation(&outcome)
            {
                self.discard_context_activation();
            }
        }
    }

    pub(super) fn install_context_activation(&mut self, outcome: ActivationOutcome) -> bool {
        // Keep this exact grant token through installation. A revocation racing
        // with publication must not be replaced by a newer unrelated token.
        let token = self
            .context_admission
            .as_ref()
            .and_then(|admission| admission.activation_outcome_token(&outcome));
        if self.context_admission_required && token.is_none() {
            self.discard_context_activation();
            return false;
        }
        let installed_outcome = outcome.clone();
        let reset_unknown = matches!(&outcome, ActivationOutcome::ResetUnknown(_));
        let next_owner_lease_identity = next_input_identity();
        let (owner, activation_generation, lineage, transferred, source_free_tail_epoch, kind) =
            match outcome {
                ActivationOutcome::Transfer(grant) => {
                    if grant.target_owner.path.as_str() != self.path {
                        self.fail_context_activation();
                        return false;
                    }
                    let transferred = {
                        let Ok(mut shared) = self.shared.lock() else {
                            self.fail_context_activation();
                            return false;
                        };
                        if shared.active_path.as_deref() != Some(grant.source_owner.path.as_str())
                            || shared.context_owner_generation.is_some_and(|generation| {
                                generation != grant.source_owner.generation.0
                            })
                            || shared.handoff_tail_epoch != grant.source_tail_epoch
                        {
                            None
                        } else {
                            let suppression = match shared.autocorrect_suppression.clone() {
                                Some(AutocorrectSuppression::CurrentWord(mut current)) => {
                                    current.owner_lease_identity = next_owner_lease_identity;
                                    let rebound = AutocorrectSuppression::CurrentWord(current);
                                    shared.autocorrect_suppression = Some(rebound.clone());
                                    shared.suppression_revision =
                                        shared.suppression_revision.wrapping_add(1);
                                    Some(rebound)
                                }
                                _ => {
                                    if shared.autocorrect_suppression.take().is_some() {
                                        shared.suppression_revision =
                                            shared.suppression_revision.wrapping_add(1);
                                    }
                                    None
                                }
                            };
                            shared.active_path = Some(self.path.clone());
                            shared.context_owner_generation = Some(grant.target_owner.generation.0);
                            shared.preserve_active_path_until = None;
                            shared.exact_manual_toggle_handoff_epoch = None;
                            shared.exact_manual_toggle_handoff_path = None;
                            Some((
                                shared.handoff_tail_buffer.clone(),
                                shared.handoff_tail_epoch,
                                suppression,
                            ))
                        }
                    };
                    let Some(transferred) = transferred else {
                        self.fail_context_activation();
                        return false;
                    };
                    (
                        grant.target_owner,
                        grant.target_activation.generation.0,
                        grant.lineage,
                        Some(transferred),
                        None,
                        "transfer_installed",
                    )
                }
                ActivationOutcome::SourceFree(grant) | ActivationOutcome::ResetUnknown(grant) => {
                    if grant.target_owner.path.as_str() != self.path {
                        self.fail_context_activation();
                        return false;
                    }
                    if self.context_admission.as_ref().is_none_or(|admission| {
                        admission.current_owner().as_ref() != Some(&grant.target_owner)
                    }) {
                        self.fail_context_activation();
                        return false;
                    }
                    let source_free_tail_epoch = self.shared.lock().ok().and_then(|mut shared| {
                        if shared
                            .context_owner_generation
                            .is_some_and(|generation| generation > grant.target_owner.generation.0)
                        {
                            None
                        } else {
                            let tail_epoch = if reset_unknown {
                                let next_epoch = shared
                                    .handoff_tail_epoch
                                    .max(self.committed_tail.epoch)
                                    .max(grant.tail_epoch)
                                    .checked_add(1)?;
                                if !self.context_admission.as_ref().is_some_and(|admission| {
                                    admission.bind_reset_tail_epoch(&grant, next_epoch)
                                }) {
                                    return None;
                                }
                                next_epoch
                            } else if grant.tail_epoch > 0 {
                                grant.tail_epoch
                            } else {
                                let next_epoch = shared.handoff_tail_epoch.checked_add(1)?;
                                if !self.context_admission.as_ref().is_some_and(|admission| {
                                    admission.bind_source_free_tail_epoch(&grant, next_epoch)
                                }) {
                                    return None;
                                }
                                next_epoch
                            };
                            shared.active_path = Some(self.path.clone());
                            shared.context_owner_generation = Some(grant.target_owner.generation.0);
                            shared.handoff_tail_buffer.clear();
                            shared.handoff_tail_epoch = tail_epoch;
                            shared.handoff_focus_receipt = None;
                            shared.preserve_active_path_until = None;
                            shared.exact_manual_toggle_handoff_epoch = None;
                            shared.exact_manual_toggle_handoff_path = None;
                            if shared.autocorrect_suppression.take().is_some() {
                                shared.suppression_revision =
                                    shared.suppression_revision.wrapping_add(1);
                            }
                            Some(tail_epoch)
                        }
                    });
                    let Some(source_free_tail_epoch) = source_free_tail_epoch else {
                        self.discard_context_activation();
                        return false;
                    };
                    (
                        grant.target_owner,
                        grant.target_activation.generation.0,
                        grant.lineage,
                        None,
                        Some(source_free_tail_epoch),
                        if reset_unknown {
                            "reset_unknown_installed"
                        } else {
                            "source_free_installed"
                        },
                    )
                }
            };

        self.invalidate_input_frame_background_work();
        self.client_context.focus_serial = next_input_identity();
        self.client_context.runtime_owner_lease_identity = next_owner_lease_identity;
        self.composition.buffer.clear();
        self.composition.cursor = 0;
        self.clear_preedit_completion_state();
        self.composition.pending_passthrough_preedit_clear = false;
        if let Some((tail, epoch, suppression)) = transferred {
            self.committed_tail.buffer = tail;
            self.committed_tail.epoch = epoch;
            self.committed_tail.autocorrect_suppression = suppression;
        } else {
            self.committed_tail.buffer.clear();
            self.committed_tail.epoch =
                source_free_tail_epoch.expect("source-free activation owns an empty tail epoch");
            self.committed_tail.autocorrect_suppression = None;
        }
        self.committed_tail.pending_completion_learning = None;
        self.composition.word_input_mode = None;
        self.rebuild_preedit_fast_from_tail();
        let owner_generation = owner.generation.0;
        self.context_owner = Some(owner);
        self.context_word_scope = Some(WordScope::new(lineage));
        self.context_token = token;
        self.context_handoff_sealed = false;
        if self.context_admission_required
            && self.context_admission.as_ref().is_none_or(|admission| {
                !admission.activation_outcome_is_current(&installed_outcome)
            })
        {
            self.discard_context_activation();
            return false;
        }
        trace::record_context_admission(
            "activation_install",
            "",
            0,
            kind,
            Some(owner_generation),
            Some(activation_generation),
            Some(match lineage.completeness {
                WordCompleteness::KnownStart => "known_start",
                WordCompleteness::UnknownStart => "unknown_start",
            }),
        );
        true
    }

    pub(super) fn fail_context_activation(&mut self) {
        if let (Some(admission), Some(owner)) =
            (self.context_admission.as_ref(), self.context_owner.as_ref())
        {
            admission.revoke_current_owner(owner);
        }
        self.discard_context_activation();
    }

    fn discard_context_activation(&mut self) {
        let failed_owner = self.context_owner.clone();
        self.fail_context_activation_local();
        if let (Some(failed_owner), Ok(mut shared)) = (failed_owner, self.shared.lock()) {
            if shared.active_path.as_deref() == Some(self.path.as_str())
                && shared.context_owner_generation == Some(failed_owner.generation.0)
            {
                shared.active_path = None;
                shared.context_owner_generation = None;
                shared.handoff_tail_buffer.clear();
                shared.handoff_focus_receipt = None;
                if shared.autocorrect_suppression.take().is_some() {
                    shared.suppression_revision = shared.suppression_revision.wrapping_add(1);
                }
            }
        }
    }

    fn fail_context_activation_local(&mut self) {
        self.context_owner = None;
        self.context_token = None;
        self.context_word_scope = None;
        self.context_handoff_sealed = false;
        self.committed_tail.buffer.clear();
        self.committed_tail.autocorrect_suppression = None;
        self.committed_tail.pending_completion_learning = None;
        self.clear_preedit_completion_state();
    }

    pub(super) async fn begin_context_key_callback(
        &mut self,
        header: &zbus::message::Header<'_>,
        callback_entered: Instant,
        atomic: bool,
    ) -> Option<KeyCallback> {
        self.try_install_pending_context_activation();
        let admission = self.context_admission.clone()?;
        let Some(owner) = self.context_owner.clone() else {
            // The stamp rendezvous is the only await admitted while the engine
            // interface lock is held. It lets the observer poison a pending
            // transfer to UnknownStart before this key runs literally; it does
            // not wait for the compatibility property or marker fence.
            if let Ok(observed) = admission.observe_callback(header, callback_entered).await {
                self.try_install_pending_context_activation();
                if let Some(owner) = self.context_owner.as_ref() {
                    if let Ok(callback) =
                        admission.key_callback_from_observed(owner, observed.clone(), atomic)
                    {
                        return Some(callback);
                    }
                }
                admission.abandon_observed_key(&observed);
            }
            return None;
        };
        let callback = if atomic {
            admission
                .begin_atomic_key_callback(&owner, header, callback_entered)
                .await
        } else {
            admission
                .begin_key_callback(&owner, header, callback_entered)
                .await
        };
        match callback {
            Ok(callback) => Some(callback),
            Err(_) => {
                self.revoke_context_word();
                None
            }
        }
    }

    pub(super) fn settle_context_key_callback(
        &mut self,
        callback: Option<&KeyCallback>,
        keyval: u32,
        keycode: u32,
        state: u32,
        tail_before: &str,
        handled: bool,
    ) {
        let Some(admission) = self.context_admission.clone() else {
            return;
        };
        let Some(callback) = callback else {
            self.revoke_context_word();
            return;
        };
        self.advance_context_word_scope(keyval, keycode, state, tail_before, handled);
        let Some(scope) = self.context_word_scope.as_ref() else {
            return;
        };
        if !admission.settle_key_callback(
            callback,
            SettledWordState::from_scope(self.committed_tail.epoch, scope).with_content_type(
                self.client_context.content_purpose,
                self.client_context.content_hints,
            ),
        ) {
            self.revoke_context_word();
            return;
        }
        self.context_token = admission.current_token();
    }

    pub(super) fn settle_context_callback_without_input(&mut self, callback: Option<&KeyCallback>) {
        let Some(admission) = self.context_admission.clone() else {
            return;
        };
        let (Some(callback), Some(scope)) = (callback, self.context_word_scope.as_ref()) else {
            self.revoke_context_word();
            return;
        };
        if !admission.settle_key_callback(
            callback,
            SettledWordState::from_scope(self.committed_tail.epoch, scope).with_content_type(
                self.client_context.content_purpose,
                self.client_context.content_hints,
            ),
        ) {
            self.revoke_context_word();
            return;
        }
        self.context_token = admission.current_token();
    }

    fn advance_context_word_scope(
        &mut self,
        keyval: u32,
        keycode: u32,
        state: u32,
        tail_before: &str,
        handled: bool,
    ) {
        if !is_key_press(state) {
            return;
        }
        let is_backspace = keyval == KEY_BACKSPACE;
        let crossed_boundary = is_backspace
            && (tail_before.is_empty()
                || tail_before
                    .chars()
                    .next_back()
                    .is_some_and(super::preedit::is_observed_word_boundary));
        let observed_boundary = !is_backspace
            && (matches!(keyval, KEY_ENTER | KEY_KP_ENTER)
                || self
                    .physical_char(keyval, keycode)
                    .is_some_and(super::preedit::is_observed_word_boundary));
        let passive_navigation = matches!(keyval, KEY_LEFT | KEY_RIGHT | KEY_UP | KEY_DOWN);
        let unproven_external_input = !handled
            && (passive_navigation
                || keyval == KEY_TAB
                || self.physical_char(keyval, keycode).is_none());
        let printable = self.physical_char(keyval, keycode).is_some();
        let track_suffix = self
            .context_word_scope
            .as_ref()
            .is_some_and(|scope| scope.lineage().completeness == WordCompleteness::UnknownStart);
        let observed_append = if track_suffix && printable && !is_backspace {
            observed_tail_append_length(tail_before, &self.committed_tail.buffer)
        } else {
            None
        };
        let exact_backspace = is_backspace
            && tail_before
                .char_indices()
                .next_back()
                .is_some_and(|(last, _)| self.committed_tail.buffer == tail_before[..last]);
        let retained_boundary = if crossed_boundary && exact_backspace {
            self.committed_tail
                .buffer
                .chars()
                .enumerate()
                .filter(|(_, ch)| super::preedit::is_observed_word_boundary(*ch))
                .last()
                .and_then(|(offset, _)| u32::try_from(offset).ok())
        } else {
            None
        };
        let new_boundary = observed_boundary
            .then(|| self.committed_tail.buffer.chars().enumerate().last())
            .flatten()
            .filter(|(_, ch)| super::preedit::is_observed_word_boundary(*ch))
            .and_then(|(offset, _)| u32::try_from(offset).ok());
        let Some(scope) = self.context_word_scope.as_mut() else {
            return;
        };
        // Modifier keys themselves are layout/gesture observations, not text
        // gaps. A command-modified non-modifier key is unproven client-side
        // input and revokes the admitted word.
        if is_shift_key(keyval) || is_accept_completion_with_space_key(keyval) {
            scope.keep_or_revoke_unknown();
            return;
        }
        if has_command_modifier(state) {
            scope.revoke_for_input_gap();
            return;
        }
        if is_backspace {
            if crossed_boundary {
                // Reuse only a separator within this lineage's observed
                // range. An empty or non-exact mirror has no such witness.
                scope.reopen_at_retained_boundary(retained_boundary);
                if trace::enabled() {
                    trace::record(format!(
                        r#"{{"kind":"ibus_context_boundary_backspace","exact_backspace":{exact_backspace},"retained_observed_start":{}}}"#,
                        scope.lineage().completeness == WordCompleteness::KnownStart,
                    ));
                }
            } else if scope.lineage().completeness == WordCompleteness::UnknownStart {
                if exact_backspace {
                    scope.observe_tail_backspace();
                } else {
                    scope.revoke_for_input_gap();
                }
            }
            return;
        }
        if observed_boundary {
            scope.close_at_observed_boundary(self.committed_tail.epoch, new_boundary);
            return;
        }
        if unproven_external_input {
            scope.revoke_for_input_gap();
        } else if printable && scope.lineage().completeness == WordCompleteness::UnknownStart {
            if let Some(retained_chars) = observed_append {
                scope.observe_tail_append(retained_chars);
            } else {
                scope.revoke_for_input_gap();
            }
        }
    }

    pub(super) fn revoke_context_word(&mut self) {
        if let Some(scope) = self.context_word_scope.as_mut() {
            scope.revoke_for_input_gap();
        }
        if let (Some(admission), Some(owner)) =
            (self.context_admission.as_ref(), self.context_owner.as_ref())
        {
            if admission.ready_reset_replaces(owner) {
                // The observer already discarded this word and published its
                // source-free replacement. Do not let the delayed Set handler
                // revoke that successor before it can be installed.
                self.context_token = None;
            } else {
                admission.revoke_current_owner(owner);
                self.context_token = admission.current_token();
            }
        } else {
            self.context_token = None;
        }
        self.committed_tail.pending_completion_learning = None;
        self.clear_preedit_completion_state();
    }
}

fn observed_tail_append_length(before: &str, after: &str) -> Option<u32> {
    let (last, _) = after.char_indices().next_back()?;
    let prefix = &after[..last];
    let retained_chars = after.chars().count();
    (prefix == before
        || (retained_chars == super::preedit::PREEDIT_TAIL_LIMIT && before.ends_with(prefix)))
    .then_some(retained_chars as u32)
}

#[cfg(test)]
#[path = "context_runtime/tests.rs"]
mod tests;
