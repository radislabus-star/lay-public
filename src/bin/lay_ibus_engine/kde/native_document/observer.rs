//! Bound Chrome notifications wake an actual read; their payload is not proof.
use super::{bounded, bus, LayIbusEngine, NativeDocumentBinding, IO_BUDGET};
use crate::output::EngineOutput;
use crate::window_interaction::WindowInteraction;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use std::time::Instant;

#[derive(Debug)]
pub(super) struct Subscription {
    _task: zbus::Task<()>,
    live: Arc<AtomicBool>,
}

struct Completion(Arc<AtomicBool>);

impl Drop for Completion {
    fn drop(&mut self) {
        self.0.store(false, Ordering::Release);
    }
}

impl Subscription {
    fn spawn(
        executor: &zbus::Executor<'_>,
        work: impl std::future::Future<Output = ()> + Send + 'static,
    ) -> Self {
        let live = Arc::new(AtomicBool::new(true));
        // Capture before spawning: cancellation before the first poll must
        // retire the subscription too. This is resource state, not authority.
        let completion = Completion(live.clone());
        let task = executor.spawn(
            async move {
                let _completion = completion;
                work.await;
            },
            "lay-kde-chrome-text-observer",
        );
        Self { _task: task, live }
    }

    fn is_live(&self) -> bool {
        self.live.load(Ordering::Acquire)
    }
}

pub(super) async fn watch_changes<F, Fut>(mut events: zbus::MessageStream, mut refresh: F)
where
    F: FnMut() -> Fut,
    Fut: std::future::Future<Output = bool>,
{
    let dirty = AtomicBool::new(false);
    let changed = event_listener::Event::new();
    // A full zbus signal stream blocks the shared socket reader, including
    // method replies. Keep draining while refresh waits for the engine lock
    // or reads the actual document. Signal payloads never authorize an edit.
    let notifications = async {
        loop {
            let next =
                crate::context_admission::next_ordered_message(std::pin::Pin::new(&mut events))
                    .await;
            match next {
                Some(Ok(_event)) => {}
                Some(Err(_)) => {
                    crate::trace::record(
                        r#"{"kind":"ibus_native_document","stage":"observer_stream_end","reason":"stream_error"}"#,
                    );
                    return;
                }
                None => {
                    crate::trace::record(
                        r#"{"kind":"ibus_native_document","stage":"observer_stream_end","reason":"stream_closed"}"#,
                    );
                    return;
                }
            }
            dirty.store(true, Ordering::Release);
            changed.notify(1);
        }
    };
    let observations = async {
        loop {
            let listener = changed.listen();
            if !dirty.swap(false, Ordering::AcqRel) {
                listener.await;
                continue;
            }
            drop(listener);
            if !refresh().await {
                return;
            }
        }
    };
    // Stream end, stale ownership or cancellation retires both futures.
    futures_lite::future::race(notifications, observations).await;
}

pub(super) fn current(engine: &LayIbusEngine, expected: &NativeDocumentBinding) -> bool {
    engine.config.active_layout_backend() == lay::desktop::LayoutBackend::Kde
        && !engine.atomic.active
        && !engine.content_is_sensitive()
        && engine.client_context.focus_serial == expected.focus_serial
        && engine.client_context.runtime_owner_lease_identity == expected.owner_lease
        && engine.layout_gesture.layout_generation == expected.layout_generation
        && engine
            .client_context
            .native_document_binding
            .as_ref()
            .is_some_and(|binding| binding.same_observation_owner(expected))
        && engine
            .shared
            .lock()
            .is_ok_and(|state| state.active_path.as_deref() == Some(engine.path.as_str()))
}

pub(super) fn needs_subscription(engine: &LayIbusEngine, binding: &NativeDocumentBinding) -> bool {
    let available = binding
        .observer
        .as_ref()
        .is_none_or(|observer| !observer.is_live());
    let owner_current = current(engine, binding);
    if available && crate::trace::enabled() {
        crate::trace::record(
            serde_json::json!({
                "kind": "ibus_native_document", "stage": "observer_arm",
                "notification_owner_current": owner_current,
                "edit_receipt_current": binding.is_current(engine),
                "tail_epoch_matches": binding.tail_epoch == engine.committed_tail.epoch,
                "focus_matches": binding.focus_serial == engine.client_context.focus_serial,
                "lease_matches": binding.owner_lease == engine.client_context.runtime_owner_lease_identity,
                "layout_matches": binding.layout_generation == engine.layout_gesture.layout_generation,
            })
            .to_string(),
        );
    }
    // CommitText expires the old word receipt. Keep listening within the same
    // observation owner so its actual result can arrive; the refresh below
    // still acquires a new snapshot and retains every edit-authority guard.
    available && owner_current
}

pub(super) fn arm<'a>(
    engine: &'a mut LayIbusEngine,
    output: &'a EngineOutput<'_, '_>,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = ()> + Send + 'a>> {
    // The observer calls the same adapter after a signal. Give that recursive
    // async boundary an explicit boxed Send contract, as native reads do.
    Box::pin(async move {
        let Some(mut binding) = engine
            .client_context
            .native_document_binding
            .as_ref()
            .filter(|binding| needs_subscription(engine, binding))
            .cloned()
        else {
            return;
        };
        binding.observer = None;
        let Some(connection) = output.connection().cloned() else {
            return;
        };
        let started = Instant::now();
        let events = bounded(
            Instant::now() + IO_BUDGET,
            bus::observe_text_changes(&binding.connection, &binding.identity),
        )
        .await;
        crate::trace::record(
            serde_json::json!({
                "kind": "ibus_native_document", "stage": "observer_subscribe",
                "ok": events.is_ok(), "reason": events.as_ref().err().copied(),
                "elapsed_us": started.elapsed().as_micros(),
                "field": binding.identity.field.as_str(),
            })
            .to_string(),
        );
        let Ok(events) = events else { return };
        let path = engine.path.clone();
        let executor = connection.executor().clone();
        let task = Subscription::spawn(&executor, async move {
            watch_changes(events, move || {
                let connection = connection.clone();
                let path = path.clone();
                let binding = binding.clone();
                async move {
                    let Ok(iface) = connection
                        .object_server()
                        .interface::<_, LayIbusEngine>(path.as_str())
                        .await
                    else {
                        crate::trace::record(r#"{"kind":"ibus_native_document","stage":"observer_event","reason":"interface_unavailable"}"#);
                        return false;
                    };
                    let emitter = iface.signal_emitter();
                    let mut engine = iface.get_mut().await;
                    if !current(&engine, &binding) {
                        crate::trace::record(serde_json::json!({
                            "kind": "ibus_native_document", "stage": "observer_event",
                            "reason": "owner_changed",
                            "focus_matches": engine.client_context.focus_serial == binding.focus_serial,
                            "lease_matches": engine.client_context.runtime_owner_lease_identity == binding.owner_lease,
                            "layout_matches": engine.layout_gesture.layout_generation == binding.layout_generation,
                            "binding_matches": engine.client_context.native_document_binding.as_ref().is_some_and(|actual| actual.same_observation_owner(&binding)),
                        }).to_string());
                        return false;
                    }
                    crate::trace::record(r#"{"kind":"ibus_native_document","stage":"observer_event","reason":"actual_read"}"#);
                    let mut output = EngineOutput::legacy(emitter);
                    // Revalidate desktop, peer, focused field, scope and actual text.
                    // The signal is neither a snapshot nor a successful-edit receipt.
                    if matches!(
                        WindowInteraction::refresh_native_document(&mut engine, &mut output).await,
                        Ok(true)
                    ) && engine.native_document_is_current()
                    {
                        let _ = engine
                            .refresh_native_document_precognition(&mut output)
                            .await;
                    }
                    true
                }
            })
            .await;
        });
        if let Some(binding) = engine.client_context.native_document_binding.as_mut() {
            binding.observer = Some(Arc::new(task));
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::context_admission::{ContextAdmissionAdapter, WordCompleteness};

    #[test]
    fn native_text_observer_completed_handle_does_not_prevent_rearming() {
        let (_, _, peer) = ContextAdmissionAdapter::test_established(
            "/test/completed_native_observer",
            "/org/freedesktop/IBus/InputContext_observer",
            WordCompleteness::UnknownStart,
            17,
        );
        let mut finished = Subscription::spawn(peer.executor(), async {});
        zbus::block_on(&mut finished._task).unwrap();
        assert!(!finished.is_live());
        // Keeping the completed handle must not make a replacement live.
        let replacement = Subscription::spawn(peer.executor(), std::future::pending());
        assert!(replacement.is_live());
        assert!(!finished.is_live());
    }

    #[test]
    fn native_text_observer_cancellation_retires_liveness() {
        let live = Arc::new(AtomicBool::new(true));
        let completion = Completion(live.clone());
        let unpolled = async move {
            let _completion = completion;
            std::future::pending::<()>().await;
        };
        drop(unpolled);
        assert!(!live.load(Ordering::Acquire));
    }
}
