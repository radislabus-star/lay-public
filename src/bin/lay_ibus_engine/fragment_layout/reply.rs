//! Finish a positive native-field letter request after its unchanged key reply.
//!
//! Dispatch is not a client acknowledgement. The existing document observation,
//! decision, verifier and executor must still establish every edit permission.

use std::time::Instant;

use serde::Serialize;
use zbus::object_server::{ResponseDispatchNotifier, SignalEmitter};
use zbus::zvariant::{Signature, Type};

use super::super::context_admission::{AdmissionToken, EngineOwner};
use super::super::engine::LayIbusEngine;
use super::super::output::EngineOutput;
use super::FragmentLetterRelease;

/// The public D-Bus body remains exactly one bool. Ordinary callbacks allocate
/// neither an event nor a continuation.
#[derive(Debug)]
pub(crate) enum KeyEventReply {
    Plain(bool),
    Notified(ResponseDispatchNotifier<bool>),
}

impl Serialize for KeyEventReply {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            Self::Plain(handled) => handled.serialize(serializer),
            Self::Notified(reply) => reply.serialize(serializer),
        }
    }
}

impl Type for KeyEventReply {
    const SIGNATURE: &'static Signature = bool::SIGNATURE;
}

impl KeyEventReply {
    #[cfg(test)]
    pub(crate) fn handled(&self) -> bool {
        match self {
            Self::Plain(handled) => *handled,
            Self::Notified(reply) => *reply.response(),
        }
    }
}

/// Transient caller metadata, not a word owner, queue or permission to write.
struct ReplyBoundFragment {
    path: String,
    owner: Option<EngineOwner>,
    token: Option<AdmissionToken>,
    focus_serial: u64,
    owner_lease: u64,
    tail_epoch: u64,
    tail: String,
    layout_generation: u64,
    layout_is_ru: bool,
    release: FragmentLetterRelease,
    handled: bool,
    request: lay::layout_fragments::FragmentLayoutRequest,
    started: Instant,
}

impl ReplyBoundFragment {
    fn new(
        engine: &LayIbusEngine,
        release: FragmentLetterRelease,
        handled: bool,
        request: lay::layout_fragments::FragmentLayoutRequest,
    ) -> Self {
        Self {
            path: engine.path.clone(),
            owner: engine.context_owner.clone(),
            token: engine.context_token.clone(),
            focus_serial: engine.client_context.focus_serial,
            owner_lease: engine.client_context.runtime_owner_lease_identity,
            tail_epoch: engine.committed_tail.epoch,
            tail: engine.committed_tail.buffer.clone(),
            layout_generation: engine.layout_gesture.layout_generation,
            layout_is_ru: engine.layout_gesture.layout_is_ru,
            release,
            handled,
            request,
            started: Instant::now(),
        }
    }

    fn is_current(&self, engine: &LayIbusEngine) -> bool {
        engine.path == self.path
            && engine.context_owner == self.owner
            && engine.context_token == self.token
            && engine.client_context.focus_serial == self.focus_serial
            && engine.client_context.runtime_owner_lease_identity == self.owner_lease
            && engine.committed_tail.epoch == self.tail_epoch
            && engine.committed_tail.buffer == self.tail
            && engine.layout_gesture.layout_generation == self.layout_generation
            && engine.layout_gesture.layout_is_ru == self.layout_is_ru
            && engine.composition.buffer.is_empty()
            && engine.config.active_layout_backend() == lay::desktop::LayoutBackend::Kde
            && engine.client_context.native_document_binding.is_some()
            && engine.fragment_layout_letter_is_allowed(self.release.letter)
            && engine.fragment_letter_release_is_current(&self.release, self.handled)
    }
}

impl LayIbusEngine {
    pub(crate) async fn fragment_letter_key_reply(
        &mut self,
        emitter: &SignalEmitter<'_>,
        release: FragmentLetterRelease,
        handled: bool,
    ) -> KeyEventReply {
        // Only the existing managed KDE native document's late-read case can
        // defer. GNOME, terminals, owned preedit and already-current frames use
        // the previous synchronous caller unchanged.
        let needs_reply = self.config.active_layout_backend() == lay::desktop::LayoutBackend::Kde
            && self.client_context.managed_input
            && !release.native
            && !self.uses_native_terminal_input()
            && self.client_context.native_document_binding.is_some()
            && self.fragment_letter_release_is_current(&release, handled)
            && self.fragment_native_observation_is_pending(
                self.last_tail_token_text().chars().count() as u32,
            );
        if !needs_reply {
            let mut output = EngineOutput::legacy(emitter);
            self.request_fragment_after_letter_release(&mut output, release, handled)
                .await;
            return KeyEventReply::Plain(handled);
        }
        let Some(request) = self.fragment_layout_letter_request(release.letter) else {
            return KeyEventReply::Plain(handled);
        };
        let pending = ReplyBoundFragment::new(self, release, handled, request);
        let connection = emitter.connection().clone();
        let task_connection = connection.clone();
        let (reply, dispatched) = ResponseDispatchNotifier::new(handled);
        connection
            .executor()
            .spawn(
                async move {
                    dispatched.await;
                    let Ok(interface) = task_connection
                        .object_server()
                        .interface::<_, LayIbusEngine>(pending.path.as_str())
                        .await
                    else {
                        return;
                    };
                    let mut engine = interface.get_mut().await;
                    let mut output = EngineOutput::legacy(interface.signal_emitter());
                    engine
                        .fulfill_fragment_after_reply(&mut output, pending)
                        .await;
                },
                "lay-fragment-letter-reply",
            )
            .detach();
        KeyEventReply::Notified(reply)
    }

    async fn fulfill_fragment_after_reply(
        &mut self,
        output: &mut EngineOutput<'_, '_>,
        pending: ReplyBoundFragment,
    ) {
        if !pending.is_current(self) {
            self.trace_fragment_refusal("reply_scope_changed");
            return;
        }
        if crate::trace::enabled() {
            crate::trace::record(
                serde_json::json!({
                    "kind": "ibus_letter_fragment_reply",
                    "stage": "current_request_needs_actual_observation",
                    "focus_serial": pending.focus_serial,
                    "tail_epoch": pending.tail_epoch,
                })
                .to_string(),
            );
        }
        // The stamp only permits reconsidering this exact request. The existing
        // operation obtains a fresh actual snapshot and verifies it before any
        // mutation; response dispatch itself supplies no edit authority.
        self.apply_fragment_layout_request(output, pending.request, pending.started)
            .await;
    }
}

#[cfg(test)]
#[path = "reply/tests.rs"]
mod tests;
