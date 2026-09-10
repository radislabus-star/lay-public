use std::time::{Duration, Instant};

use zbus::fdo;
use zbus::interface;
use zbus::object_server::SignalEmitter;
use zbus::zvariant::Value;

use super::atomic::{AtomicCapability, AtomicEnvelope, AtomicPriorReceipt};
use super::engine::{LayIbusEngine, SurroundingTextSnapshot};
use super::output::{
    AtomicProposal, EngineOutput, PROPOSAL_CONSUMED_NO_EFFECT, PROPOSAL_FRAME_READY,
};
use super::protocol::{
    is_accept_completion_with_space_key, is_key_press, is_shift_key, KEY_LEFT_SHIFT,
};
use super::trace;

#[interface(name = "org.freedesktop.IBus.Engine")]
impl LayIbusEngine {
    #[zbus(name = "ProcessKeyEvent")]
    pub(crate) async fn process_key_event(
        &mut self,
        #[zbus(header)] header: zbus::message::Header<'_>,
        #[zbus(signal_emitter)] emitter: SignalEmitter<'_>,
        keyval: u32,
        keycode: u32,
        state: u32,
    ) -> fdo::Result<bool> {
        if !self.legacy_key_route_allowed() {
            trace::record(r#"{"kind":"ibus_legacy_key_blocked","owner":"atomic"}"#);
            return Ok(false);
        }
        let callback_entered = Instant::now();
        let callback_serial = header.primary().serial_num().get();
        if trace::enabled() {
            let owner_generation = self
                .context_admission
                .as_ref()
                .and_then(|admission| admission.current_owner())
                .map(|owner| owner.generation.0);
            let activation_generation = self
                .context_admission
                .as_ref()
                .and_then(|admission| admission.current_activation_generation());
            trace::record_context_admission(
                "legacy_callback_enter",
                "ProcessKeyEvent",
                callback_serial,
                if is_key_press(state) {
                    "press"
                } else {
                    "release"
                },
                owner_generation,
                activation_generation,
                None,
            );
        }
        let callback = self
            .begin_context_key_callback(&header, callback_entered, false)
            .await;
        if trace::enabled() {
            trace::record_context_admission(
                "legacy_callback_admission",
                "ProcessKeyEvent",
                callback_serial,
                if callback.is_some() {
                    "accepted"
                } else {
                    "refused"
                },
                self.context_owner.as_ref().map(|owner| owner.generation.0),
                self.context_admission
                    .as_ref()
                    .and_then(|admission| admission.current_activation_generation()),
                None,
            );
        }
        let tail_before = self.committed_tail.buffer.clone();
        self.context_callback_entered = Some(callback_entered);
        self.consume_shift_gesture_handoff();
        let mut output = EngineOutput::legacy(&emitter);
        let result = self
            .process_key_event_with_output(&mut output, keyval, keycode, state)
            .await;
        self.context_callback_entered = None;
        if result.is_ok() {
            let handled = result.as_ref().is_ok_and(|handled| *handled);
            self.settle_context_key_callback(
                callback.as_ref(),
                keyval,
                keycode,
                state,
                &tail_before,
                handled,
            );
            if is_key_press(state) && tail_before != self.committed_tail.buffer {
                self.refresh_observed_suffix_precognition(&mut output)
                    .await?;
            }
        } else {
            self.revoke_context_word();
        }
        result
    }

    #[zbus(name = "ProcessKeyEventAtomicV1")]
    #[expect(
        clippy::too_many_arguments,
        reason = "fixed AtomicV1 wire arguments plus the authenticated message header"
    )]
    async fn process_key_event_atomic_v1(
        &mut self,
        #[zbus(header)] header: zbus::message::Header<'_>,
        keyval: u32,
        keycode: u32,
        state: u32,
        envelope: AtomicEnvelope,
        capability: AtomicCapability,
        prior_receipt: AtomicPriorReceipt,
    ) -> fdo::Result<AtomicProposal> {
        self.process_key_event_atomic_callback(
            &header,
            keyval,
            keycode,
            state,
            envelope,
            capability,
            prior_receipt,
        )
        .await
    }

    #[zbus(name = "FocusIn")]
    pub(crate) async fn focus_in_callback(
        &mut self,
        #[zbus(header)] header: zbus::message::Header<'_>,
    ) {
        let changed = if self.context_admission_required {
            self.activate_context_from_header(&header, Instant::now(), None)
                .await
        } else {
            self.bind_focus_path()
        };
        self.finish_focus_in(changed);
    }

    fn finish_focus_in(&mut self, changed: bool) {
        self.discard_atomic_pending();
        self.atomic.active = false;
        self.invalidate_input_frame_background_work();
        trace::record(if changed {
            r#"{"kind":"ibus_focus","stage":"focus_in","receipt":"new_path"}"#
        } else {
            r#"{"kind":"ibus_focus","stage":"focus_in","receipt":"same_path"}"#
        });
        self.config = lay::config::LayConfig::load();
        self.client_context.surrounding_text_snapshot = None;
        if !changed && !self.context_admission_required {
            self.refresh_empty_tail_from_handoff();
        }
    }

    #[zbus(name = "FocusInId")]
    pub(crate) async fn focus_in_id(
        &mut self,
        #[zbus(header)] header: zbus::message::Header<'_>,
        object_path: String,
        client: String,
    ) {
        let changed = self.bind_focus_receipt(object_path, client);
        trace::record(if changed {
            r#"{"kind":"ibus_focus","stage":"focus_in_id","receipt":"new"}"#
        } else {
            r#"{"kind":"ibus_focus","stage":"focus_in_id","receipt":"same"}"#
        });
        let activated = if self.context_admission_required {
            let native_path = self
                .client_context
                .focus_receipt
                .as_deref()
                .and_then(|receipt| receipt.split('\u{1f}').next())
                .map(str::to_owned);
            self.activate_context_from_header(&header, Instant::now(), native_path.as_deref())
                .await
        } else {
            self.bind_focus_path()
        };
        self.finish_focus_in(changed || activated);
    }

    #[zbus(name = "FocusOut")]
    pub(crate) async fn focus_out(&mut self, #[zbus(header)] header: zbus::message::Header<'_>) {
        if self
            .observe_context_focus_out(&header, Instant::now())
            .await
        {
            self.finish_focus_out();
        }
    }

    fn finish_focus_out(&mut self) {
        self.discard_atomic_pending();
        self.atomic.active = false;
        trace::record(r#"{"kind":"ibus_focus","stage":"focus_out"}"#);
        let preserve_active_path = self.context_handoff_sealed
            || !self.context_admission_required
                && (self.should_preserve_focus_handoff() || self.shared_active_path_preserved());
        self.reset_for_ibus_focus_change();
        if preserve_active_path {
            return;
        }
        let mut state = self.shared.lock().expect("lay ime state poisoned");
        if state.active_path.as_deref() == Some(self.path.as_str())
            && (!self.context_admission_required
                || match self.context_owner.as_ref() {
                    Some(owner) => state.context_owner_generation == Some(owner.generation.0),
                    None => state.context_owner_generation.is_none(),
                })
        {
            state.active_path = None;
            state.context_owner_generation = None;
        }
    }

    #[zbus(name = "FocusOutId")]
    async fn focus_out_id(
        &mut self,
        #[zbus(header)] header: zbus::message::Header<'_>,
        _object_path: String,
    ) {
        if self
            .observe_context_focus_out(&header, Instant::now())
            .await
        {
            self.finish_focus_out();
        }
    }

    #[zbus(name = "SetCursorLocation")]
    async fn set_cursor_location(
        &mut self,
        #[zbus(signal_emitter)] emitter: SignalEmitter<'_>,
        x: i32,
        y: i32,
        w: i32,
        h: i32,
    ) -> fdo::Result<()> {
        self.client_context.cursor_cell_width = w;
        trace::record_cursor_location(x, y, w, h);
        if self.atomic.active {
            return Ok(());
        }
        let mut output = EngineOutput::legacy(&emitter);
        self.flush_dirty_preedit(&mut output).await
    }

    #[zbus(name = "ProcessHandWritingEvent")]
    fn process_hand_writing_event(&mut self, _coordinates: Vec<f64>) {}

    #[zbus(name = "CancelHandWriting")]
    fn cancel_hand_writing(&mut self, _n_strokes: u32) {}

    #[zbus(name = "SetCapabilities")]
    fn set_capabilities(&mut self, caps: u32) {
        self.set_client_capabilities(caps);
        trace::record_capabilities(caps, self.client_context.surrounding_text_supported);
    }

    #[zbus(name = "PropertyActivate")]
    fn property_activate(&mut self, _name: String, _state: u32) {}

    #[zbus(name = "PropertyShow")]
    fn property_show(&mut self, _name: String) {}

    #[zbus(name = "PropertyHide")]
    fn property_hide(&mut self, _name: String) {}

    #[zbus(name = "CandidateClicked")]
    fn candidate_clicked(&mut self, _index: u32, _button: u32, _state: u32) {}

    #[zbus(name = "Reset")]
    pub(crate) async fn reset(
        &mut self,
        #[zbus(header)] header: zbus::message::Header<'_>,
        #[zbus(signal_emitter)] emitter: SignalEmitter<'_>,
    ) -> fdo::Result<()> {
        if self
            .observe_context_revocation(&header, Instant::now())
            .await
        {
            self.discard_atomic_pending();
            trace::record(r#"{"kind":"ibus_focus","stage":"reset"}"#);
            let cleared = if self.atomic.active {
                Ok(())
            } else {
                self.clear_preedit(&mut EngineOutput::legacy(&emitter))
                    .await
            };
            self.reset_for_ibus_soft_reset();
            cleared?;
        }
        Ok(())
    }

    #[zbus(name = "Enable")]
    async fn enable(
        &mut self,
        #[zbus(signal_emitter)] emitter: SignalEmitter<'_>,
    ) -> fdo::Result<()> {
        trace::record(r#"{"kind":"ibus_focus","stage":"enable"}"#);
        Self::require_surrounding_text(&emitter)
            .await
            .map_err(|error| fdo::Error::Failed(error.to_string()))
    }

    #[zbus(name = "Disable")]
    pub(crate) async fn disable(&mut self, #[zbus(header)] header: zbus::message::Header<'_>) {
        if self.observe_context_disable(&header, Instant::now()).await {
            self.discard_atomic_pending();
            self.atomic.active = false;
            trace::record(r#"{"kind":"ibus_focus","stage":"disable"}"#);
            self.reset_for_ibus_soft_reset();
        }
    }

    #[zbus(name = "PageUp")]
    fn page_up(&mut self) {}

    #[zbus(name = "PageDown")]
    fn page_down(&mut self) {}

    #[zbus(name = "CursorUp")]
    fn cursor_up(&mut self) {}

    #[zbus(name = "CursorDown")]
    fn cursor_down(&mut self) {}

    #[zbus(name = "SetSurroundingText")]
    pub(crate) async fn set_surrounding_text(
        &mut self,
        #[zbus(signal_emitter)] emitter: SignalEmitter<'_>,
        text: Value<'_>,
        cursor_pos: u32,
        anchor_pos: u32,
    ) -> fdo::Result<()> {
        let snapshot = ibus_text_value_to_string(&text)
            .map(|text| SurroundingTextSnapshot::new(text, cursor_pos, anchor_pos));
        let suffix_snapshot_changed = self
            .client_context
            .surrounding_text_snapshot
            .as_ref()
            .map(|snapshot| (&snapshot.text, snapshot.cursor_pos, snapshot.anchor_pos))
            != snapshot
                .as_ref()
                .map(|snapshot| (&snapshot.text, snapshot.cursor_pos, snapshot.anchor_pos));
        self.observe_external_surrounding_text(snapshot);
        let retry_status = self.pending_ime_auto_undo_retry_status();
        let sensitive = self.content_is_sensitive();
        trace::record_surrounding_text_snapshot(
            self.client_context
                .surrounding_text_snapshot
                .as_ref()
                .map_or(0, |snapshot| snapshot.text.chars().count()),
            if sensitive { 0 } else { cursor_pos },
            if sensitive { 0 } else { anchor_pos },
            retry_status,
        );
        if self.atomic.active {
            self.observe_visible_postcondition();
            return Ok(());
        }
        let mut output = EngineOutput::legacy(&emitter);
        if self
            .apply_pending_manual_toggle_after_surrounding_snapshot(&mut output)
            .await?
        {
            return Ok(());
        }
        if should_apply_auto_undo_before_postcondition(retry_status) {
            let status = if self.undo_last_ime_autocorrect(&mut output).await?.is_some() {
                "applied_after_causal_precondition_snapshot"
            } else {
                "causal_precondition_apply_failed"
            };
            trace::record_auto_undo_retry(status);
        }
        self.observe_visible_postcondition();
        if matches!(retry_status, "ready" | "ready_boundary_elided") {
            let status = if self.undo_last_ime_autocorrect(&mut output).await?.is_some() {
                if retry_status == "ready_boundary_elided" {
                    "applied_after_boundary_elided_snapshot"
                } else {
                    "applied_after_exact_snapshot"
                }
            } else {
                "snapshot_apply_failed"
            };
            trace::record_auto_undo_retry(status);
        }
        if suffix_snapshot_changed {
            self.refresh_observed_suffix_precognition(&mut output)
                .await?;
        }
        Ok(())
    }

    #[zbus(name = "PanelExtensionReceived")]
    fn panel_extension_received(&mut self, _event: Value<'_>) {}

    #[zbus(name = "PanelExtensionRegisterKeys")]
    fn panel_extension_register_keys(&mut self, _data: Value<'_>) {}

    #[zbus(signal, name = "CommitText")]
    pub(crate) async fn commit_text(
        emitter: &SignalEmitter<'_>,
        text: Value<'_>,
    ) -> zbus::Result<()>;

    #[zbus(signal, name = "ForwardKeyEvent")]
    pub(crate) async fn forward_key_event(
        emitter: &SignalEmitter<'_>,
        keyval: u32,
        keycode: u32,
        state: u32,
    ) -> zbus::Result<()>;

    #[zbus(signal, name = "DeleteSurroundingText")]
    pub(crate) async fn delete_surrounding_text(
        emitter: &SignalEmitter<'_>,
        offset: i32,
        nchars: u32,
    ) -> zbus::Result<()>;

    #[zbus(signal, name = "RequireSurroundingText")]
    pub(crate) async fn require_surrounding_text(emitter: &SignalEmitter<'_>) -> zbus::Result<()>;

    #[zbus(signal, name = "UpdatePreeditText")]
    pub(crate) async fn update_preedit_text(
        emitter: &SignalEmitter<'_>,
        text: Value<'_>,
        cursor_pos: u32,
        visible: bool,
        mode: u32,
    ) -> zbus::Result<()>;

    #[zbus(signal, name = "ShowPreeditText")]
    pub(crate) async fn show_preedit_text(emitter: &SignalEmitter<'_>) -> zbus::Result<()>;

    #[zbus(signal, name = "HidePreeditText")]
    pub(crate) async fn hide_preedit_text(emitter: &SignalEmitter<'_>) -> zbus::Result<()>;

    #[zbus(property, name = "ContentType")]
    fn content_type(&self) -> (u32, u32) {
        (
            self.client_context.content_purpose,
            self.client_context.content_hints,
        )
    }

    #[zbus(property, name = "ContentType")]
    pub(crate) async fn set_content_type(
        &mut self,
        value: (u32, u32),
        #[zbus(header)] header: Option<zbus::message::Header<'_>>,
    ) {
        let unchanged = if let (Some(admission), Some(path), Some(header)) = (
            self.context_admission.clone(),
            super::context_admission::EnginePath::new(self.path.clone()),
            header.as_ref(),
        ) {
            match admission
                .observe_content_type_callback(&path, value, header, Instant::now())
                .await
            {
                Ok(Some(unchanged)) => unchanged,
                Ok(None) => return,
                // Failed rendezvous cannot preserve word authority, but the
                // property still owns sensitive-field protection.
                Err(_) => false,
            }
        } else {
            false
        };
        if !unchanged {
            self.revoke_context_word();
        }
        self.set_content_type_state(value.0, value.1);
        trace::record(format!(
            r#"{{"kind":"ibus_content_type","purpose":{},"hints":{},"text_assistance":{}}}"#,
            value.0,
            value.1,
            self.content_allows_text_assistance()
        ));
    }

    #[zbus(property, name = "FocusId")]
    fn focus_id(&self) -> bool {
        self.context_admission.is_some()
    }

    #[zbus(property, name = "ActiveSurroundingText")]
    fn active_surrounding_text(&self) -> bool {
        true
    }
}

impl LayIbusEngine {
    #[expect(
        clippy::too_many_arguments,
        reason = "preserve one-to-one forwarding of authenticated AtomicV1 wire arguments"
    )]
    pub(super) async fn process_key_event_atomic_callback(
        &mut self,
        header: &zbus::message::Header<'_>,
        keyval: u32,
        keycode: u32,
        state: u32,
        envelope: AtomicEnvelope,
        capability: AtomicCapability,
        prior_receipt: AtomicPriorReceipt,
    ) -> fdo::Result<AtomicProposal> {
        let callback_entered = Instant::now();
        let callback = self
            .begin_context_key_callback(header, callback_entered, true)
            .await;
        self.context_callback_entered = Some(callback_entered);
        let result = self
            .process_atomic_key_event_with_context_tail(
                keyval,
                keycode,
                state,
                envelope,
                capability,
                prior_receipt,
            )
            .await;
        self.context_callback_entered = None;
        match result.as_ref() {
            Ok((proposal, _))
                if matches!(
                    proposal.0,
                    PROPOSAL_FRAME_READY | PROPOSAL_CONSUMED_NO_EFFECT
                ) =>
            {
                if !self.bind_atomic_context_callback(callback) {
                    self.revoke_context_word();
                }
            }
            Ok((_, tail_before)) => {
                // Native-unhandled means the client, rather than Lay, applies
                // this exact key. It is still an observed input/boundary and
                // must advance completeness through the ordinary key path.
                self.settle_context_key_callback(
                    callback.as_ref(),
                    keyval,
                    keycode,
                    state,
                    tail_before,
                    false,
                );
            }
            Err(_) => self.revoke_context_word(),
        }
        result.map(|(proposal, _)| proposal)
    }
}

#[cfg(test)]
impl LayIbusEngine {
    fn focus_in(&mut self) {
        let changed = self.bind_focus_path();
        self.finish_focus_in(changed);
    }
}

#[cfg(test)]
mod td121_content_type_tests {
    use super::*;
    use crate::context_admission::{ActivationOutcome, ContextAdmissionAdapter, WordCompleteness};
    use lay::config::LayConfig;
    use std::sync::{Arc, Mutex};
    use std::time::Instant;

    fn unavailable_engine(path: &str, shared: crate::protocol::Shared) -> LayIbusEngine {
        LayIbusEngine::new_from_component(
            path.to_string(),
            shared,
            None,
            "lay-ime-us",
            true,
            LayConfig {
                text_backend: "ime".to_string(),
                ..LayConfig::default()
            },
        )
    }

    #[test]
    fn td121_actual_properties_set_content_type_revokes_known_start() {
        let path = "/io/github/radislabus_star/LayIme/engine/td121_content_type";
        let (adapter, grant, _peer) = ContextAdmissionAdapter::test_established(
            path,
            "/org/freedesktop/IBus/InputContext_td121_content_type",
            WordCompleteness::KnownStart,
            23,
        );
        let mut engine = LayIbusEngine::new_from_component(
            path.to_string(),
            Arc::new(Mutex::new(Default::default())),
            Some(adapter),
            "lay-ime-us",
            true,
            LayConfig {
                text_backend: "ime".to_string(),
                ..LayConfig::default()
            },
        );
        assert!(engine.install_context_activation(ActivationOutcome::SourceFree(grant)));
        engine.committed_tail.buffer = "known".to_string();
        engine.rebuild_preedit_fast_from_tail();
        assert!(engine.context_word_is_known());
        assert!(engine.capture_input_frame_identity().is_some());

        zbus::block_on(engine.set_content_type((0, 1), None));

        assert!(!engine.context_word_is_known());
        assert!(engine.capture_input_frame_identity().is_none());
        assert!(engine.committed_tail.pending_completion_learning.is_none());
    }

    #[test]
    fn td121_missing_content_type_stamp_still_applies_sensitive_metadata() {
        for value in [(8u32, 0u32), (9, 0), (0, 1 << 11), (0, 1 << 12)] {
            let path = "/io/github/radislabus_star/LayIme/engine/td121_sensitive";
            let (adapter, grant, _peer) = ContextAdmissionAdapter::test_established(
                path,
                "/org/freedesktop/IBus/InputContext_td121_sensitive",
                WordCompleteness::KnownStart,
                23,
            );
            let mut engine = LayIbusEngine::new_from_component(
                path.to_string(),
                Arc::new(Mutex::new(Default::default())),
                Some(adapter),
                "lay-ime-us",
                true,
                LayConfig {
                    text_backend: "ime".to_string(),
                    ..LayConfig::default()
                },
            );
            assert!(engine.install_context_activation(ActivationOutcome::SourceFree(grant)));
            engine.committed_tail.buffer = "retained".to_string();
            engine.rebuild_preedit_fast_from_tail();
            assert!(engine.context_word_is_known());
            assert!(engine.capture_input_frame_identity().is_some());
            assert!(!engine.content_is_sensitive());

            // A genuine setter/header with no observer stamp: the bounded
            // rendezvous expires, but the property still must protect text.
            let set = zbus::Message::method_call(path, "Set")
                .unwrap()
                .interface("org.freedesktop.DBus.Properties")
                .unwrap()
                .sender(":1.2")
                .unwrap()
                .build(&(
                    "org.freedesktop.IBus.Engine",
                    "ContentType",
                    zbus::zvariant::Value::from(value),
                ))
                .unwrap();
            zbus::block_on(engine.set_content_type(value, Some(set.header())));

            assert_eq!(engine.content_type(), value);
            assert!(engine.content_is_sensitive());
            assert!(!engine.content_allows_text_assistance());
            assert!(engine.committed_tail.buffer.is_empty());
            assert!(engine.composition.buffer.is_empty());
            assert!(engine.client_context.surrounding_text_snapshot.is_none());
            assert!(!engine.context_word_is_known());
            assert!(engine.capture_input_frame_identity().is_none());
        }
    }

    #[test]
    fn td121_required_mode_without_admission_cannot_inherit_shared_tail() {
        let shared: crate::protocol::Shared = Arc::new(Mutex::new(Default::default()));
        {
            let mut state = shared.lock().expect("TD-121 unavailable shared state");
            state.handoff_tail_buffer = "foreign-field-tail".to_string();
            state.handoff_tail_epoch = 31;
            state.handoff_focus_receipt = Some("foreign-field".to_string());
        }

        let engine = unavailable_engine(
            "/io/github/radislabus_star/LayIme/engine/td121_unavailable",
            shared,
        );

        assert!(engine.committed_tail.buffer.is_empty());
        assert!(!engine.context_word_is_known());
        assert!(engine.capture_input_frame_identity().is_none());
    }

    #[test]
    fn td121_unsealed_focus_out_ignores_legacy_700ms_handoff() {
        let path = "/io/github/radislabus_star/LayIme/engine/td121_unsealed_focus_out";
        let shared: crate::protocol::Shared = Arc::new(Mutex::new(Default::default()));
        let mut engine = unavailable_engine(path, shared.clone());
        engine.committed_tail.buffer = "literal".to_string();
        engine.committed_tail.last_input_at = Some(Instant::now());
        {
            let mut state = shared.lock().expect("TD-121 unsealed shared state");
            state.active_path = Some(path.to_string());
            state.handoff_tail_buffer = "literal".to_string();
            state.handoff_tail_epoch = engine.committed_tail.epoch;
        }

        engine.finish_focus_out();

        assert!(engine.committed_tail.buffer.is_empty());
        let state = shared.lock().expect("TD-121 cleared shared state");
        assert!(state.active_path.is_none());
        assert!(state.handoff_tail_buffer.is_empty());
    }
}

#[cfg(test)]
mod td120_atomic_owner_tests {
    use super::*;
    use lay::config::LayConfig;
    use std::sync::{Arc, Barrier, Mutex};

    fn config() -> LayConfig {
        LayConfig {
            auto_replace: false,
            auto_switch_layout: false,
            text_backend: "ime".to_string(),
            ..LayConfig::default()
        }
    }

    fn engine(path: &str, shared: super::super::protocol::Shared) -> LayIbusEngine {
        LayIbusEngine::new(path.to_string(), shared, false, true, config())
    }

    fn envelope(transaction: u64, focus_epoch: u64) -> AtomicEnvelope {
        (transaction, 2, 12, focus_epoch, 5, 13, vec![8; 32])
    }

    fn prepare_atomic_space(engine: &mut LayIbusEngine, transaction: u64, focus_epoch: u64) {
        // FocusIn reloads the installed configuration. Restore this isolated
        // fixture's deterministic IME settings before exercising the producer.
        engine.config = config();
        let proposal = zbus::block_on(engine.process_atomic_key_event(
            super::super::protocol::KEY_SPACE,
            65,
            0,
            envelope(transaction, focus_epoch),
            super::super::atomic::td120_test_atomic_capability(),
            (0, 0, Vec::new()),
        ))
        .expect("real atomic Space proposal");
        assert_eq!(proposal.0, super::super::output::PROPOSAL_FRAME_READY);
        assert_eq!(proposal.1.iter().filter(|(tag, _)| *tag == 1).count(), 1);
    }

    #[test]
    fn td120_atomic_foreign_owner_and_real_aba_callbacks_reject_stale_settlement() {
        let shared = Arc::new(Mutex::new(Default::default()));
        let mut engine_a = engine("/td120/atomic/a", shared.clone());
        let mut engine_b = engine("/td120/atomic/b", shared.clone());
        engine_a.focus_in();
        engine_a.push_tail_char('x');
        prepare_atomic_space(&mut engine_a, 401, 71);
        super::super::atomic::td120_test_defer_reverted_feedback_on_pending(&engine_a);

        engine_b.focus_in();
        engine_b.push_tail_char('b');
        assert!(engine_b.arm_current_word_autocorrect_suppression());
        let foreign_owner = zbus::block_on(engine_a.process_atomic_key_event(
            super::super::protocol::KEY_LEFT_SHIFT,
            42,
            0,
            envelope(402, 71),
            super::super::atomic::td120_test_atomic_capability(),
            (2, 401, vec![9; 32]),
        ))
        .expect("foreign-owner receipt");
        assert_eq!(
            foreign_owner.0,
            super::super::output::PROPOSAL_NATIVE_UNHANDLED
        );
        assert_eq!(
            shared.lock().expect("shared state").active_path.as_deref(),
            Some("/td120/atomic/b")
        );
        let shared_guard = shared
            .lock()
            .expect("shared state")
            .autocorrect_suppression
            .clone();
        assert!(matches!(
            shared_guard,
            Some(super::super::protocol::AutocorrectSuppression::CurrentWord(
                current_word
            )) if current_word.owner_lease_identity
                == engine_b.client_context.runtime_owner_lease_identity
        ));
        assert_eq!(
            *engine_a
                .atomic
                .settlement_feedback_events
                .lock()
                .expect("atomic feedback events"),
            ["censored"]
        );
        assert!(engine_a.atomic.deferred_learning_actions.is_empty());

        engine_a.focus_in();
        engine_a.push_tail_char('y');
        prepare_atomic_space(&mut engine_a, 403, 72);
        engine_b.focus_in();
        engine_a.focus_in();

        let aba = zbus::block_on(engine_a.process_atomic_key_event(
            super::super::protocol::KEY_LEFT_SHIFT,
            42,
            0,
            envelope(404, 72),
            super::super::atomic::td120_test_atomic_capability(),
            (2, 403, vec![9; 32]),
        ))
        .expect("ABA receipt");
        assert_eq!(aba.0, super::super::output::PROPOSAL_NATIVE_UNHANDLED);
        assert!(aba.1.is_empty());
        assert!(!engine_a.committed_tail.buffer.ends_with(' '));
    }

    #[test]
    fn td120_atomic_capture_refuses_owner_changed_by_focus_in_before_snapshot() {
        let shared = Arc::new(Mutex::new(Default::default()));
        let mut engine_a = engine("/td120/atomic/admission-a", shared.clone());
        let mut engine_b = engine("/td120/atomic/admission-b", shared.clone());
        engine_a.focus_in();
        engine_a.push_tail_char('a');

        let start = Arc::new(Barrier::new(2));
        let finished = Arc::new(Barrier::new(2));
        let hook_start = start.clone();
        let hook_finished = finished.clone();
        engine_a.atomic.before_capture = Some(Arc::new(move || {
            hook_start.wait();
            hook_finished.wait();
        }));
        let callback = std::thread::spawn(move || {
            start.wait();
            engine_b.focus_in();
            engine_b.push_tail_char('b');
            assert!(engine_b.arm_current_word_autocorrect_suppression());
            finished.wait();
            engine_b
        });

        let proposal = zbus::block_on(engine_a.process_atomic_key_event(
            super::super::protocol::KEY_SPACE,
            65,
            0,
            envelope(405, 73),
            super::super::atomic::td120_test_atomic_capability(),
            (0, 0, Vec::new()),
        ))
        .expect("owner-checked atomic admission");
        let engine_b = callback.join().expect("FocusIn callback");

        assert_eq!(proposal.0, super::super::output::PROPOSAL_NATIVE_UNHANDLED);
        assert!(proposal.1.is_empty());
        let state = shared.lock().expect("shared state");
        assert_eq!(state.active_path.as_deref(), Some(engine_b.path.as_str()));
        assert_eq!(state.handoff_tail_buffer, "b");
        assert!(matches!(
            state.autocorrect_suppression.as_ref(),
            Some(super::super::protocol::AutocorrectSuppression::CurrentWord(
                current_word
            )) if current_word.owner_lease_identity
                == engine_b.client_context.runtime_owner_lease_identity
        ));
        drop(state);

        let stale_receipt = zbus::block_on(engine_a.process_atomic_key_event(
            super::super::protocol::KEY_LEFT_SHIFT,
            42,
            0,
            envelope(406, 73),
            super::super::atomic::td120_test_atomic_capability(),
            (2, 405, vec![9; 32]),
        ))
        .expect("receipt for refused admission");
        assert_eq!(
            stale_receipt.0,
            super::super::output::PROPOSAL_NATIVE_UNHANDLED
        );
        assert_eq!(
            shared.lock().expect("shared state").handoff_tail_buffer,
            "b"
        );
    }
}

impl LayIbusEngine {
    fn observe_daemon_owned_legacy_shift(&mut self, keyval: u32, keycode: u32, state: u32) -> bool {
        let pressed = is_key_press(state);
        self.layout_gesture.shift_active = pressed;
        self.layout_gesture.shift_pressed_at = None;
        self.layout_gesture.last_shift_release_at = None;
        if pressed {
            self.layout_gesture.shift_used_as_modifier = false;
            if self.layout_gesture.alt_completion_active {
                self.layout_gesture.alt_used_as_modifier = true;
                self.layout_gesture.shift_used_as_modifier = true;
                return self.toggle_layout_from_modifier_hotkey();
            }
        } else {
            self.layout_gesture.shift_used_as_modifier = false;
        }
        trace::record_key(
            "shift_observed_daemon_owner",
            keyval,
            keycode,
            false,
            None,
            self.committed_tail.buffer.chars().count(),
            self.composition.preedit_suffix.chars().count(),
        );
        false
    }

    async fn process_atomic_shift_gesture(
        &mut self,
        output: &mut EngineOutput<'_, '_>,
        keyval: u32,
        state: u32,
    ) -> fdo::Result<bool> {
        debug_assert!(self.atomic.speculation);
        let pressed = is_key_press(state);
        self.layout_gesture.shift_active = pressed;
        if pressed {
            self.layout_gesture.shift_used_as_modifier = false;
            if self.layout_gesture.alt_completion_active {
                self.layout_gesture.alt_used_as_modifier = true;
                self.layout_gesture.shift_used_as_modifier = true;
                return Ok(self.toggle_layout_from_modifier_hotkey());
            }
        }

        let gesture_key = configured_atomic_double_shift_key(&self.config.trigger, keyval);
        if pressed {
            self.layout_gesture.shift_pressed_at = gesture_key.then(Instant::now);
            if !gesture_key {
                self.layout_gesture.last_shift_release_at = None;
            }
        } else {
            let now = Instant::now();
            let tapped = gesture_key
                && self.layout_gesture.shift_pressed_at.take().is_some()
                && !self.layout_gesture.shift_used_as_modifier;
            let double_tapped = tapped
                && self
                    .layout_gesture
                    .last_shift_release_at
                    .is_some_and(|released_at| {
                        now.duration_since(released_at)
                            <= Duration::from_millis(self.config.shift_window_ms)
                    });
            self.layout_gesture.shift_used_as_modifier = false;
            self.layout_gesture.last_shift_release_at = tapped.then_some(now);
            if double_tapped {
                self.layout_gesture.last_shift_release_at = None;
                if self
                    .manual_toggle_active_text_target(output)
                    .await?
                    .is_some()
                {
                    return Ok(true);
                }
            }
        }
        Ok(false)
    }

    pub(crate) async fn process_key_event_with_output(
        &mut self,
        output: &mut EngineOutput<'_, '_>,
        keyval: u32,
        keycode: u32,
        state: u32,
    ) -> fdo::Result<bool> {
        if !self.client_context.managed_input {
            return Ok(false);
        }
        if !is_key_press(state) && self.consume_handled_release(keycode) {
            trace::record_key(
                "managed_release",
                keyval,
                keycode,
                true,
                None,
                self.committed_tail.buffer.chars().count(),
                self.composition.preedit_suffix.chars().count(),
            );
            return Ok(true);
        }
        if !self.live_composition_enabled() {
            if self.has_live_composition_state() {
                self.reset_for_ibus_focus_change();
                self.clear_preedit(output).await?;
            }
            trace::record_key("composition_disabled", keyval, keycode, false, None, 0, 0);
            return Ok(false);
        }
        if is_shift_key(keyval) {
            if !self.atomic.speculation {
                return Ok(self.observe_daemon_owned_legacy_shift(keyval, keycode, state));
            }
            return self
                .process_atomic_shift_gesture(output, keyval, state)
                .await;
        }
        if is_key_press(state) {
            self.layout_gesture.last_shift_release_at = None;
        }
        if is_accept_completion_with_space_key(keyval) {
            let pressed = is_key_press(state);
            if pressed {
                self.layout_gesture.alt_completion_active = true;
                let retired = self.retire_pending_precognition(output).await?;
                self.layout_gesture.alt_used_as_modifier =
                    self.layout_gesture.shift_active || retired;
                if self.layout_gesture.shift_active {
                    self.layout_gesture.shift_used_as_modifier = true;
                    return Ok(self.toggle_layout_from_modifier_hotkey());
                }
                return Ok(false);
            }
            if self.layout_gesture.alt_completion_active
                && !self.layout_gesture.alt_used_as_modifier
            {
                self.layout_gesture.alt_completion_active = false;
                return self.accept_completion_with_space(output).await;
            }
            self.layout_gesture.alt_completion_active = false;
            self.layout_gesture.alt_used_as_modifier = false;
            return Ok(false);
        }
        if !is_key_press(state) {
            return Ok(false);
        }
        self.layout_gesture.last_shift_release_at = None;
        if self.layout_gesture.shift_active {
            self.layout_gesture.shift_used_as_modifier = true;
        }
        if self.layout_gesture.alt_completion_active {
            self.layout_gesture.alt_used_as_modifier = true;
        }
        let handled = self
            .process_pressed_key(output, keyval, keycode, state)
            .await?;
        self.remember_handled_press(keycode, handled);
        Ok(handled)
    }
}

fn configured_atomic_double_shift_key(trigger: &str, keyval: u32) -> bool {
    trigger == "double-lshift" && keyval == KEY_LEFT_SHIFT
}

fn should_apply_auto_undo_before_postcondition(retry_status: &str) -> bool {
    retry_status == "ready_causal_precondition"
}

#[cfg(test)]
mod causal_precondition_tests {
    use super::{should_apply_auto_undo_before_postcondition, LayIbusEngine};
    use crate::output::{AtomicEffectBuilder, EngineOutput, PROPOSAL_NATIVE_UNHANDLED};
    use crate::protocol::SharedState;
    use crate::protocol::{KEY_LEFT_SHIFT, KEY_RIGHT_SHIFT, RELEASE_MASK};
    use lay::config::LayConfig;
    use std::sync::{Arc, Mutex};

    #[test]
    fn causal_precondition_undo_precedes_stale_postcondition_quarantine() {
        assert!(should_apply_auto_undo_before_postcondition(
            "ready_causal_precondition"
        ));
        assert!(!should_apply_auto_undo_before_postcondition("ready"));
        assert!(!should_apply_auto_undo_before_postcondition(
            "ready_boundary_elided"
        ));
        assert!(!should_apply_auto_undo_before_postcondition(
            "waiting_exact_snapshot"
        ));
    }

    #[test]
    fn physical_double_shift_owner_legacy_route_is_observation_only() {
        let config = LayConfig {
            text_backend: "ime".to_string(),
            ..LayConfig::default()
        };
        let mut engine = LayIbusEngine::new(
            "/engine/legacy-double-shift".to_string(),
            Arc::new(Mutex::new(SharedState::default())),
            false,
            true,
            config,
        );
        engine.composition.buffer = "ghbdtn".to_string();
        engine.composition.cursor = engine.composition.buffer.chars().count();
        engine.committed_tail.buffer = "prefix ghbdtn".to_string();

        for keyval in [KEY_LEFT_SHIFT, KEY_RIGHT_SHIFT] {
            for _ in 0..4 {
                for state in [0, RELEASE_MASK] {
                    let mut builder = AtomicEffectBuilder::default();
                    let mut output = EngineOutput::atomic(&mut builder);
                    let handled = zbus::block_on(engine.process_key_event_with_output(
                        &mut output,
                        keyval,
                        42,
                        state,
                    ))
                    .expect("legacy Shift observation");

                    assert!(!handled);
                    assert_eq!(
                        builder.finish(handled),
                        (PROPOSAL_NATIVE_UNHANDLED, Vec::new())
                    );
                }
            }
        }

        assert_eq!(engine.composition.buffer, "ghbdtn");
        assert_eq!(engine.committed_tail.buffer, "prefix ghbdtn");
        assert!(engine.layout_gesture.shift_pressed_at.is_none());
        assert!(engine.layout_gesture.last_shift_release_at.is_none());
    }

    #[test]
    fn physical_double_shift_owner_legacy_boundary_has_no_edit_capability() {
        let source = include_str!("ibus_interface.rs");
        let legacy_handler = source
            .split("fn observe_daemon_owned_legacy_shift")
            .nth(1)
            .expect("legacy Shift owner boundary")
            .split("async fn process_atomic_shift_gesture")
            .next()
            .expect("atomic Shift boundary");

        for forbidden in [
            "EngineOutput",
            "manual_toggle_active_text_target",
            "replace_committed_tail",
            "commit_text",
            "delete_surrounding_text",
        ] {
            assert!(
                !legacy_handler.contains(forbidden),
                "legacy Shift observer gained edit capability: {forbidden}"
            );
        }
    }
}

pub(crate) fn ibus_text_value_to_string(value: &Value<'_>) -> Option<String> {
    match value {
        Value::Str(text) => Some(text.as_str().to_string()),
        Value::Structure(structure) => {
            let fields = structure.fields();
            match fields.get(2) {
                Some(Value::Str(text)) => Some(text.as_str().to_string()),
                _ => None,
            }
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::{configured_atomic_double_shift_key, ibus_text_value_to_string};
    use crate::protocol::{KEY_LEFT_SHIFT, KEY_RIGHT_SHIFT};
    use crate::text::make_ibus_text;
    use zbus::zvariant::Value;

    #[test]
    fn double_shift_matches_only_two_left_shift_members() {
        assert!(configured_atomic_double_shift_key(
            "double-lshift",
            KEY_LEFT_SHIFT
        ));
        assert!(!configured_atomic_double_shift_key(
            "double-lshift",
            KEY_RIGHT_SHIFT
        ));
        assert!(!configured_atomic_double_shift_key(
            "double-ctrl",
            KEY_LEFT_SHIFT
        ));
    }

    #[test]
    fn parses_plain_string_surrounding_text() {
        assert_eq!(
            ibus_text_value_to_string(&Value::new("привет")),
            Some("привет".to_string())
        );
    }

    #[test]
    fn parses_ibus_text_surrounding_text() {
        let value = make_ibus_text("abc ghbdtn".to_string());

        assert_eq!(
            ibus_text_value_to_string(&value),
            Some("abc ghbdtn".to_string())
        );
    }
}
