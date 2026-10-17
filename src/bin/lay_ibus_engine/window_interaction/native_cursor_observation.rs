// The existing pending receipt owns all editing provenance. These helpers
// cannot grant ordinary reset confirmation or change WordCompleteness.
impl LayIbusEngine {
    // Diagnostic metadata only: never publish text, create a receipt or settle
    // authority. The existing trace switch owns its storage and lifetime.
    fn trace_native_cursor_phase(&self, stage: &str, keyval: u32, keycode: u32, state: u32) {
        if !trace::enabled() {
            return;
        }
        let receipt = self.native_cursor_edit_receipt();
        let actual = self.client_context.surrounding_text_snapshot.as_ref();
        trace::record(format!(
            r#"{{"kind":"ibus_native_cursor_phase","stage":{stage:?},"keyval":{keyval},"keycode":{keycode},"state":{state},"phase_present":{},"identity_current":{},"awaits_effect":{},"last_key":{},"reset_spent":{},"actual_cursor":{},"actual_anchor":{},"actual_chars":{},"snapshot_matches":{},"tail_chars":{},"preedit_chars":{},"preedit_visible":{},"legacy_word_preedit_active":{},"local_token_live":{},"observed_manual_grant":{},"reset_manual_grant":{},"manual_ready":{},"atomic_active":{},"sensitive":{},"handoff_sealed":{},"replay_quarantine_active":{},"replay_tail_change_quarantined":{}}}"#,
            receipt.is_some(),
            receipt.is_some_and(|receipt| self.native_cursor_identity_is_current(receipt)),
            receipt.is_some_and(|receipt| receipt.awaits_effect()),
            receipt.and_then(|receipt| receipt.last_key).map_or_else(
                || "null".to_string(),
                |(value, code)| format!("[{value},{code}]")
            ),
            receipt.is_some_and(|receipt| receipt.end_reset_spent),
            actual
                .map(|actual| actual.cursor_pos)
                .map_or_else(|| "null".to_string(), |value| value.to_string()),
            actual
                .map(|actual| actual.anchor_pos)
                .map_or_else(|| "null".to_string(), |value| value.to_string()),
            actual
                .map(|actual| actual.text.chars().count())
                .map_or_else(|| "null".to_string(), |value| value.to_string()),
            receipt.is_some_and(|receipt| actual == Some(&receipt.actual)),
            self.committed_tail.buffer.chars().count(),
            self.composition.buffer.chars().count(),
            self.composition.preedit_visible,
            self.composition.legacy_word_preedit_active,
            self.live_context_token().is_some(),
            self.context_observed_suffix_exact_manual_handoff_allowed(),
            self.context_reset_rereceipt_exact_manual_handoff_allowed(),
            self.native_cursor_manual_source_is_current(),
            self.atomic.active,
            self.content_is_sensitive(),
            self.context_handoff_sealed,
            self.exact_replay_quarantine_active(),
            self.exact_replay_tail_change_quarantined,
        ));
    }

    fn native_cursor_edit_receipt(&self) -> Option<&ManualCursorEditReceipt> {
        match self
            .context_reset_rereceipt
            .as_ref()?
            .native_mutation_reset_evidence
            .as_ref()?
        {
            NativeMutationResetEvidence::CursorEditing(receipt) => Some(receipt),
            _ => None,
        }
    }

    fn native_cursor_identity_is_current(&self, receipt: &ManualCursorEditReceipt) -> bool {
        !self.context_handoff_sealed
            && !self.atomic.active
            && !self.content_is_sensitive()
            && !self.composition.legacy_word_preedit_active
            && self.composition.buffer.is_empty()
            && self.client_context.surrounding_text_supported
            && (
                receipt.focus_serial,
                receipt.runtime_owner_lease_identity,
                receipt.layout_generation,
            ) == (
                self.client_context.focus_serial,
                self.client_context.runtime_owner_lease_identity,
                self.layout_gesture.layout_generation,
            )
            && self.context_token.as_ref() == Some(&receipt.token)
            && self
                .context_admission
                .as_ref()
                .is_some_and(|admission| admission.revalidate(&receipt.token))
    }

    fn prepare_native_cursor_edit(&mut self, keyval: u32, keycode: u32, state: u32) -> bool {
        if !is_key_press(state) || is_shift_key(keyval) {
            return false;
        }
        if has_command_modifier(state) || self.uses_native_terminal_input() {
            if self.native_cursor_edit_receipt().is_some() {
                self.discard_context_reset_rereceipt();
            }
            return false;
        }
        let operation = match keyval {
            KEY_LEFT => Some(CursorEditOperation::Left),
            KEY_RIGHT => Some(CursorEditOperation::Right),
            0xff57 => Some(CursorEditOperation::End),
            KEY_BACKSPACE => Some(CursorEditOperation::Backspace),
            _ => self
                .physical_char(keyval, keycode)
                .filter(|ch| !crate::preedit::is_observed_word_boundary(*ch))
                .map(CursorEditOperation::OwnedInsert),
        };
        let Some(operation) = operation else {
            if self.native_cursor_edit_receipt().is_some() {
                self.discard_context_reset_rereceipt();
            }
            return false;
        };
        let existing = self.native_cursor_edit_receipt().cloned();
        let mut receipt = if let Some(receipt) = existing {
            if !self.native_cursor_identity_is_current(&receipt) {
                self.discard_context_reset_rereceipt();
                return false;
            }
            receipt
        } else {
            // Only a passive horizontal move can start editing provenance.
            // SSI presence alone is never a substitute for this source grant.
            if !matches!(
                operation,
                CursorEditOperation::Left | CursorEditOperation::Right
            ) || !(self.context_observed_suffix_exact_manual_handoff_allowed()
                || self.context_reset_rereceipt_exact_manual_handoff_allowed())
                || self.atomic.active
                || self.content_is_sensitive()
                || !self.composition.buffer.is_empty()
                || self.composition.legacy_word_preedit_active
                || self.exact_replay_quarantine_active()
                || self.exact_replay_tail_change_quarantined
            {
                return false;
            }
            let Some(token) = self.live_context_token() else {
                return false;
            };
            let Some(actual) = self.client_context.surrounding_text_snapshot.clone() else {
                return false;
            };
            let suffix = self.last_tail_token_text();
            let Some(receipt) = ManualCursorEditReceipt::from_admitted_suffix(
                token,
                actual,
                &suffix,
                (
                    self.client_context.focus_serial,
                    self.client_context.runtime_owner_lease_identity,
                    self.layout_gesture.layout_generation,
                ),
            ) else {
                return false;
            };
            receipt
        };
        if !receipt.prepare(operation, (keyval, keycode)) {
            self.discard_context_reset_rereceipt();
            return false;
        }
        let token = receipt.token.clone();
        let token_text = receipt.owned_text();
        self.context_reset_rereceipt = Some(PendingContextResetRereceipt {
            completed_replay: None,
            confirmed_native_replay_reset_ack: None,
            native_mutation_reset_evidence: Some(NativeMutationResetEvidence::CursorEditing(
                Box::new(receipt),
            )),
            token: token.clone(),
            predecessor_token: token,
            tail_epoch: self.committed_tail.epoch,
            token_text,
            observed_suffix_chars: 0,
            armed_revision: self.client_context.surrounding_observation_revision,
            confirmed: false,
            awaiting_owned_append_receipt: false,
            strict_prefix_echo_revision: None,
            published_preedit: None,
        });
        true
    }

    fn rebind_native_cursor_after_key(
        &mut self,
        keyval: u32,
        keycode: u32,
        state: u32,
        handled: bool,
        tail_before: &str,
    ) {
        let Some(mut receipt) = self.native_cursor_edit_receipt().cloned() else {
            return;
        };
        if is_key_press(state) {
            if handled && receipt.last_key == Some((keyval, keycode)) {
                if let Some(appended) = self.committed_tail.buffer.strip_prefix(tail_before) {
                    let mut chars = appended.chars();
                    if let (Some(ch), None) = (chars.next(), chars.next()) {
                        let _ = receipt.confirm_owned_insert(ch);
                    }
                }
            }
        } else if !is_shift_key(keyval) && receipt.last_key == Some((keyval, keycode)) {
            receipt.last_key = None;
        }
        let Some(token) = self.live_context_token() else {
            self.discard_context_reset_rereceipt();
            return;
        };
        receipt.token = token.clone();
        if let Some(pending) = self.context_reset_rereceipt.as_mut() {
            pending.token = token;
            pending.tail_epoch = self.committed_tail.epoch;
            pending.native_mutation_reset_evidence = Some(
                NativeMutationResetEvidence::CursorEditing(Box::new(receipt)),
            );
        }
    }

    fn observe_native_cursor_edit_receipt(&mut self) -> bool {
        let Some(mut receipt) = self.native_cursor_edit_receipt().cloned() else {
            return false;
        };
        if !self.native_cursor_identity_is_current(&receipt)
            || !self
                .client_context
                .surrounding_text_snapshot
                .as_ref()
                .is_some_and(|actual| receipt.observe(actual))
        {
            self.trace_native_cursor_phase("actual_receipt_rejected", 0, 0, 0);
            self.discard_context_reset_rereceipt();
            return true;
        }
        if let Some(pending) = self.context_reset_rereceipt.as_mut() {
            pending.token_text = receipt.owned_text();
            pending.armed_revision = self.client_context.surrounding_observation_revision;
            pending.native_mutation_reset_evidence = Some(
                NativeMutationResetEvidence::CursorEditing(Box::new(receipt)),
            );
        }
        true
    }

    fn capture_native_cursor_reset(&self) -> Option<ManualCursorEditReceipt> {
        let receipt = self.native_cursor_edit_receipt()?;
        // The handler may run before canonical Reset ingress. Pin only the
        // existing live source in that order, or its reducer-owned Reset
        // provenance when ingress has already retired it. This clone is inert:
        // restoration still requires the independently matched actual Reset.
        if receipt.end_reset_spent
            || self.atomic.active
            || self.content_is_sensitive()
            || (
                receipt.focus_serial,
                receipt.runtime_owner_lease_identity,
                receipt.layout_generation,
            ) != (
                self.client_context.focus_serial,
                self.client_context.runtime_owner_lease_identity,
                self.layout_gesture.layout_generation,
            )
            || self.context_token.as_ref() != Some(&receipt.token)
            || !(self.native_cursor_identity_is_current(receipt)
                || self.context_admission.as_ref().is_some_and(|admission| {
                    admission.exact_replay_reset_provenance_is_current(&receipt.token)
                }))
        {
            return None;
        }
        Some(receipt.clone())
    }

    fn restore_native_cursor_after_reset(&mut self, mut receipt: ManualCursorEditReceipt) {
        let Some(token) = self.live_context_token() else {
            self.trace_native_cursor_phase("reset_restore_missing_token", 0, 0, 0);
            return;
        };
        if !token.shares_input_activation(&receipt.token)
            || !self.context_admission.as_ref().is_some_and(|admission| {
                admission.exact_replay_reset_provenance_is_current(&receipt.token)
            })
            || self.atomic.active
            || self.content_is_sensitive()
            || (
                receipt.focus_serial,
                receipt.runtime_owner_lease_identity,
                receipt.layout_generation,
            ) != (
                self.client_context.focus_serial,
                self.client_context.runtime_owner_lease_identity,
                self.layout_gesture.layout_generation,
            )
        {
            self.trace_native_cursor_phase("reset_restore_rejected", 0, 0, 0);
            return;
        }
        let predecessor_token = receipt.token.clone();
        receipt.token = token.clone();
        receipt.end_reset_spent = true;
        if !receipt.awaits_effect() {
            // Retain this exact actual observation only after the matched
            // Reset. This is neither an SSI dispatch nor a revision increment.
            self.client_context.surrounding_text_snapshot = Some(receipt.actual.clone());
        }
        self.context_reset_rereceipt = Some(PendingContextResetRereceipt {
            completed_replay: None,
            confirmed_native_replay_reset_ack: None,
            token,
            predecessor_token,
            tail_epoch: self.committed_tail.epoch,
            token_text: receipt.owned_text(),
            observed_suffix_chars: 0,
            armed_revision: self.client_context.surrounding_observation_revision,
            confirmed: false,
            awaiting_owned_append_receipt: false,
            strict_prefix_echo_revision: None,
            published_preedit: None,
            native_mutation_reset_evidence: Some(NativeMutationResetEvidence::CursorEditing(
                Box::new(receipt),
            )),
        });
    }

    fn native_cursor_release_is_current(&self, keyval: u32, keycode: u32) -> bool {
        self.native_cursor_edit_receipt().is_some_and(|receipt| {
            self.native_cursor_identity_is_current(receipt)
                && receipt.last_key == Some((keyval, keycode))
                && !receipt.awaits_effect()
                && self.client_context.surrounding_text_snapshot.as_ref() == Some(&receipt.actual)
        })
    }

    fn restore_native_cursor_observation_after_reset(&mut self) {
        let Some(receipt) = self.native_cursor_edit_receipt() else {
            return;
        };
        if !receipt.awaits_effect() && self.native_cursor_identity_is_current(receipt) {
            self.client_context.surrounding_text_snapshot = Some(receipt.actual.clone());
        }
    }

    pub(crate) fn native_cursor_manual_source_is_current(&self) -> bool {
        self.native_cursor_edit_receipt().is_some_and(|receipt| {
            self.native_cursor_identity_is_current(receipt)
                && receipt.last_key.is_none()
                && receipt.exact_manual_suffix().is_some()
                && self.client_context.surrounding_text_snapshot.as_ref() == Some(&receipt.actual)
        })
    }

    pub(crate) fn discard_native_cursor_manual_source(&mut self) {
        if self.native_cursor_edit_receipt().is_some() {
            self.discard_context_reset_rereceipt();
        }
    }

    pub(crate) fn consume_native_cursor_manual_source(&mut self) -> bool {
        if !self.native_cursor_manual_source_is_current() {
            return false;
        }
        let receipt = self
            .native_cursor_edit_receipt()
            .cloned()
            .expect("current manual cursor receipt");
        if self.context_bridge_token.as_ref() != Some(&receipt.token) {
            return false;
        }
        let Some(text) = receipt.exact_manual_suffix() else {
            return false;
        };
        let (Some(admission), Some(owner)) =
            (self.context_admission.clone(), self.context_owner.clone())
        else {
            return false;
        };
        let Some(settled) = self.context_word_scope.as_mut().and_then(|scope| {
            scope
                .observe_soft_reset_rereceipt(text.chars().count() as u32)
                .then(|| {
                    SettledWordState::from_scope(self.committed_tail.epoch, scope)
                        .with_content_type(
                            self.client_context.content_purpose,
                            self.client_context.content_hints,
                        )
                })
        }) else {
            return false;
        };
        let Some(token) = admission.settle_soft_reset_rereceipt(&owner, &receipt.token, settled)
        else {
            return false;
        };
        // The explicit manual consumer is the sole point that restores the
        // proved edited suffix into the ordinary tail and spends this phase.
        self.committed_tail.buffer = text;
        self.context_reset_rereceipt = None;
        self.context_token = Some(token.clone());
        self.context_bridge_token = Some(token);
        true
    }
}
