//! Actual document observations for widgets without an IBus text callback.
//!
//! This adapter owns transport resources only. The existing admission token,
//! desktop adapter, transition verifier and local executor retain authority.
#[path = "native_document/bus.rs"]
mod bus;
#[path = "native_document/client_presentation.rs"]
mod client_presentation;
#[path = "native_document/observer.rs"]
mod observer;
#[path = "native_document/preparation.rs"]
mod preparation;
#[cfg(test)]
#[path = "native_document/tests.rs"]
mod tests;

use super::desktop::FocusedWindow;
use super::SurroundingTextSnapshot;
use crate::context_admission::AdmissionToken;
use crate::engine::LayIbusEngine;
use std::future::{poll_fn, Future};
use std::task::Poll;
use std::time::{Duration, Instant};

// Equal to the existing bridge admission I/O budget. It is an adapter deadline,
// not a second admission timeout, retry loop or authority lifetime.
const IO_BUDGET: Duration = Duration::from_millis(20);
// Actual Chrome document observations include two canonical KWin reads and
// multiple native text round trips. Cold same-field traces have valid character
// bounds followed by two40ms deadline failures. Give that complete read80ms;
// this is a maximum wait, independent of admission and selection lifetimes.
const OBSERVATION_BUDGET: Duration = Duration::from_millis(80);
// Chrome may accept AddSelection before the exact range is observable. Native
// burst traces exceed 20 ms in that acknowledgement phase. Retain the complete
// range and foreground proof, with a separate bound; never retry a partial edit.
const SELECTION_BUDGET: Duration = Duration::from_millis(40);
const MAX_DOCUMENT_CHARS: i32 = 4096;
const MAX_APPLICATIONS: usize = 128;
type NativeResult<T> = Result<T, &'static str>;

impl LayIbusEngine {
    pub(crate) async fn kde_chrome_activation_deadline(
        backend: lay::desktop::LayoutBackend,
    ) -> Option<Instant> {
        if backend != lay::desktop::LayoutBackend::Kde {
            return None;
        }
        // A fresh canonical window and its actual process select timing only.
        // They never substitute for the IBus context/marker admission proof.
        let focused =
            super::desktop::focused_window_until(backend, Instant::now() + IO_BUDGET).await?;
        crate::kde::display::chrome::activation_deadline(focused.pid, Instant::now())
    }
}

#[derive(Clone, Debug)]
pub(crate) struct NativeFieldIdentity {
    desktop: FocusedWindow,
    bus: String,
    owner: String,
    application: zbus::zvariant::OwnedObjectPath,
    field: zbus::zvariant::OwnedObjectPath,
}

// A physical client field is independent of the local KWin query transport.
// Receipt freshness remains explicit at installation; notification ownership
// keeps the exact receipt, rather than weakening canonical word/edit leases.
impl PartialEq for NativeFieldIdentity {
    fn eq(&self, other: &Self) -> bool {
        super::desktop::kde::same_native_window(&self.desktop, &other.desktop)
            && self.bus == other.bus
            && self.owner == other.owner
            && self.application == other.application
            && self.field == other.field
    }
}

impl Eq for NativeFieldIdentity {}

impl NativeFieldIdentity {
    fn same_receipt_as(&self, other: &Self) -> bool {
        self == other && self.desktop == other.desktop
    }

    fn continues_from(&self, previous: &Self) -> bool {
        self == previous
            && super::desktop::kde::native_window_continues(&previous.desktop, &self.desktop)
    }
}

#[derive(Clone, Debug)]
pub(crate) struct NativeDocumentBinding {
    identity: NativeFieldIdentity,
    connection: zbus::Connection,
    token: AdmissionToken,
    focus_serial: u64,
    owner_lease: u64,
    layout_generation: u64,
    tail_epoch: u64,
    observer: Option<std::sync::Arc<observer::Subscription>>,
    // Coalesce display requests only. This never witnesses a document edit.
    last_display_frame: Option<crate::engine::InputFrameIdentity>,
    // Read-only character bounds travel with the actual document they describe.
    // Reusing a transport for another tail must never reuse its old geometry.
    hint_geometry: Option<NativeHintGeometry>,
}

#[derive(Clone, Debug)]
struct NativeHintGeometry {
    snapshot: SurroundingTextSnapshot,
    rect: [i32; 4],
}

pub(super) struct NativeSnapshot {
    snapshot: SurroundingTextSnapshot,
    hint_rect: Option<[i32; 4]>,
}

pub(crate) struct Observation {
    binding: NativeDocumentBinding,
    pub(super) snapshot: SurroundingTextSnapshot,
}

impl NativeDocumentBinding {
    // A transport subscription survives ordinary keys, not a field, focus,
    // lease or layout change. It never carries an edit or word-scope grant.
    fn same_observation_owner(&self, other: &Self) -> bool {
        self.identity.same_receipt_as(&other.identity)
            && self.focus_serial == other.focus_serial
            && self.owner_lease == other.owner_lease
            && self.layout_generation == other.layout_generation
    }

    pub(crate) fn identity(&self) -> &NativeFieldIdentity {
        &self.identity
    }

    pub(crate) fn is_current(&self, engine: &LayIbusEngine) -> bool {
        self.observation_owner_is_current(engine) && engine.committed_tail.epoch == self.tail_epoch
    }

    fn observation_owner_is_current(&self, engine: &LayIbusEngine) -> bool {
        self.observation_owner_is_current_for_token(engine, &self.token)
    }

    /// Keep the original field/focus coordinates while the pending Space
    /// adapter separately proves its single canonical word-token transition.
    fn observation_owner_is_current_for_token(
        &self,
        engine: &LayIbusEngine,
        token: &AdmissionToken,
    ) -> bool {
        engine.config.active_layout_backend() == lay::desktop::LayoutBackend::Kde
            && !engine.atomic.active
            && !engine.content_is_sensitive()
            && !engine.uses_native_terminal_input()
            && engine.context_token.as_ref() == Some(token)
            && engine
                .context_word_scope
                .as_ref()
                .is_some_and(|scope| token.matches_word_scope(scope))
            && engine
                .context_admission
                .as_ref()
                .is_some_and(|a| a.revalidate(token))
            && engine.client_context.focus_serial == self.focus_serial
            && engine.client_context.runtime_owner_lease_identity == self.owner_lease
            && engine.layout_gesture.layout_generation == self.layout_generation
    }

    pub(crate) fn for_result(&self, engine: &LayIbusEngine) -> Self {
        let mut result = self.clone();
        result.hint_geometry = None;
        result.tail_epoch = engine.committed_tail.epoch;
        result.layout_generation = engine.layout_gesture.layout_generation;
        result
    }
}

pub(super) fn bounded<T>(
    deadline: Instant,
    work: impl Future<Output = NativeResult<T>>,
) -> impl Future<Output = NativeResult<T>> {
    // Allocate before constructing the returned future. Boxing inside an
    // async fn still leaves its unpolled transport argument in that future,
    // propagating the large D-Bus frame into enclosing key/focus operations.
    let mut work = Box::pin(work);
    async move {
        if Instant::now() >= deadline {
            return Err("native_deadline");
        }
        let mut timer = std::pin::pin!(async_io::Timer::at(deadline));
        poll_fn(|cx| {
            if Instant::now() >= deadline {
                return Poll::Ready(Err("native_deadline"));
            }
            if let Poll::Ready(result) = work.as_mut().poll(cx) {
                return Poll::Ready(if Instant::now() < deadline {
                    result
                } else {
                    Err("native_deadline")
                });
            }
            if timer.as_mut().poll(cx).is_ready() {
                return Poll::Ready(Err("native_deadline"));
            }
            Poll::Pending
        })
        .await
    }
}

/// Existing read coordination only: no snapshot installation or text output.
async fn read_with_one_owned_append_resample<R, F, P>(
    deadline: Instant,
    mut read: R,
    is_pending: P,
    resample_changed: bool,
) -> NativeResult<Observation>
where
    R: FnMut(Instant, bool) -> F,
    F: Future<Output = NativeResult<Observation>>,
    P: Fn(&SurroundingTextSnapshot) -> bool,
{
    let observation = read(deadline, true).await;
    let resample = match &observation {
        Ok(observation) => is_pending(&observation.snapshot),
        Err("native_document_changed") => resample_changed,
        Err(_) => false,
    };
    if resample {
        crate::trace::record(
            serde_json::json!({
                "kind": "ibus_native_document", "stage": "owned_append_resample",
                "reason": observation.as_ref().err().copied().unwrap_or("older_owned_prefix"),
            })
            .to_string(),
        );
        return read(deadline, false).await;
    }
    observation
}

/// Native coordinates are Unicode scalars, never byte or UTF-16 offsets.
fn checked_snapshot(
    text: String,
    count: i32,
    caret: i32,
    selections: i32,
) -> NativeResult<SurroundingTextSnapshot> {
    if !(0..=MAX_DOCUMENT_CHARS).contains(&count)
        || text.chars().count() != count as usize
        || !(0..=count).contains(&caret)
        || selections != 0
    {
        return Err("native_document_shape");
    }
    Ok(SurroundingTextSnapshot::new(
        text,
        caret as u32,
        caret as u32,
    ))
}

fn exact_range(snapshot: &SurroundingTextSnapshot, backspaces: u32) -> NativeResult<(i32, i32)> {
    if snapshot.has_selection()
        || backspaces == 0
        || snapshot.cursor_pos as usize > snapshot.text.chars().count()
    {
        return Err("native_range_shape");
    }
    let start = snapshot
        .cursor_pos
        .checked_sub(backspaces)
        .ok_or("native_range_start")?;
    Ok((
        i32::try_from(start).map_err(|_| "native_range_start")?,
        i32::try_from(snapshot.cursor_pos).map_err(|_| "native_range_end")?,
    ))
}

impl LayIbusEngine {
    pub(crate) async fn refresh_native_document_precognition(
        &mut self,
        output: &mut crate::output::EngineOutput<'_, '_>,
    ) -> zbus::fdo::Result<()> {
        if !self.native_document_is_current()
            || self.context_handoff_sealed
            || !self.composition.buffer.is_empty()
        {
            return Ok(());
        }
        // The actual native receipt can complete the canonical Space frame
        // after the key callback. Queue that independent correction proof even
        // when display recovery below must retain an UnknownStart suffix.
        let space_frame = self.capture_space_autocorrect_frame_identity();
        if crate::trace::enabled() {
            let prepared = self.managed_word_start_preparation_snapshot();
            crate::trace::record(serde_json::json!({
                "kind": "ibus_native_document", "stage": "space_preparation",
                "frame_available": space_frame.is_some(),
                "pair_in_frame": space_frame.as_ref().is_some_and(|frame| frame.space_boundary_pair.is_some()),
                "pair_live": self.capture_space_boundary_pair_scope().is_some(),
                "native_preparation": self.native_word_preparation_identity().is_some(),
                "managed_identity_in_frame": space_frame.as_ref().is_some_and(|frame| frame.space_autocorrect_managed_start_identity.is_some()),
                "prepared_tail_matches": prepared.as_ref().map(|snapshot| snapshot.suffix_before_cursor(self.committed_tail.buffer.chars().count()).as_deref() == Some(self.committed_tail.buffer.as_str())),
                "manual_word_current": self.native_manual_word_is_current(),
                "actual_revision": self.client_context.surrounding_observation_revision,
                "commit_revision": self.client_context.managed_commit_snapshot_floor.map(|floor| floor.observation_revision),
                "tail_epoch": self.committed_tail.epoch,
            }).to_string());
        }
        if let Some(frame) = space_frame {
            self.schedule_space_autocorrect_prefetch(&frame);
        }
        // A complete current-word proof is independent of permission over
        // older field text. Keep the existing manual/display scope below.
        if !self.native_manual_word_is_current() {
            return Ok(());
        }
        if !self.context_word_is_known() {
            return self.refresh_observed_suffix_precognition(output).await;
        }
        // An actual read can arrive after literal-preedit transfer cancelled
        // its old display task. KnownStart then needs the ordinary canonical
        // input frame, rather than the common suffix-only recovery branch.
        let Some(frame) = self
            .capture_input_frame_identity()
            .filter(|frame| self.input_frame_identity_matches(frame))
        else {
            return Ok(());
        };
        if self
            .client_context
            .native_document_binding
            .as_ref()
            .is_some_and(|binding| binding.last_display_frame.as_ref() == Some(&frame))
        {
            return Ok(());
        }
        self.refresh_precognition_after_visible_input(output, Some(frame.clone()))
            .await?;
        if let Some(binding) = self.client_context.native_document_binding.as_mut() {
            binding.last_display_frame = Some(frame);
        }
        Ok(())
    }

    pub(crate) fn document_source_provenance(
        &self,
    ) -> crate::context_admission::ExactManualSourceProvenance {
        if self.client_context.native_document_binding.is_some() {
            crate::context_admission::ExactManualSourceProvenance::NativeDocument
        } else {
            crate::context_admission::ExactManualSourceProvenance::ClientSnapshot
        }
    }

    pub(crate) fn native_document_is_current(&self) -> bool {
        self.client_context
            .native_document_binding
            .as_ref()
            .is_some_and(|binding| binding.is_current(self))
            && self.client_context.surrounding_text_snapshot.is_some()
    }

    /// Some(None) suppresses an old IBus rectangle for a native Chrome field.
    /// Geometry is presentation only; it never supplies a document/edit proof.
    pub(crate) fn native_document_hint_rect(&self) -> Option<Option<[i32; 4]>> {
        if self.config.active_layout_backend() != lay::desktop::LayoutBackend::Kde {
            return None;
        }
        let binding = self.client_context.native_document_binding.as_ref()?;
        Some(binding.hint_geometry.as_ref().and_then(|geometry| {
            ((self.native_manual_word_is_current() || self.native_managed_readout_is_current())
                && self.client_context.surrounding_text_snapshot.as_ref()
                    == Some(&geometry.snapshot))
            .then_some(geometry.rect)
        }))
    }

    fn native_display_observation_needed(&self) -> bool {
        self.config.active_layout_backend() == lay::desktop::LayoutBackend::Kde
            && (!self.context_observed_suffix_readout_is_current()
                || self.native_document_hint_rect() == Some(None))
    }

    /// A complete actual document range can witness an explicit manual edit
    /// without upgrading UnknownStart to a known word for autocorrection.
    pub(crate) fn native_manual_word_is_current(&self) -> bool {
        self.native_document_is_current()
            && !self.context_handoff_sealed
            && (self.current_external_snapshot_agrees_with_owned_tail()
                || self.known_native_active_word_agrees())
    }

    fn known_native_active_word_agrees(&self) -> bool {
        // A KnownStart owns this active word, including after its verified
        // layout handoff. Older model text is outside the manual edit span.
        // Keep UnknownStart and closed-word permissions on their old proof.
        if !self.context_word_is_known()
            || self.committed_tail.buffer.ends_with(char::is_whitespace)
        {
            return false;
        }
        let token = self.last_tail_token_text();
        let chars = token.chars().count();
        let Some(snapshot) = self.client_context.surrounding_text_snapshot.as_ref() else {
            return false;
        };
        if chars == 0
            || snapshot.has_selection()
            || snapshot.suffix_before_cursor(chars).as_deref() != Some(token.as_str())
        {
            return false;
        }
        let start = snapshot.cursor_pos as usize - chars;
        (start == 0
            || snapshot
                .text
                .chars()
                .nth(start - 1)
                .is_some_and(crate::preedit::is_observed_word_boundary))
            && snapshot
                .text
                .chars()
                .nth(snapshot.cursor_pos as usize)
                .is_none_or(crate::preedit::is_observed_word_boundary)
    }

    /// A fresh post-CommitText Chrome observation can prove display/append of
    /// an acquisition-retained suffix without inventing its missing key history.
    /// UnknownStart, observed counts and whole-word authority remain unchanged.
    pub(crate) fn native_managed_readout_is_current(&self) -> bool {
        // Display and explicit append need the current locally committed word,
        // not authority over older field text. Chrome may normalize an older
        // separator while that word and its fresh receipt remain exact.
        self.native_document_is_current()
            && !self.context_handoff_sealed
            && self.exact_managed_surrounding_word_is_current()
    }

    /// This observation cannot set an IBus capability or callback-observed bit.
    pub(crate) fn read_native_document(
        &self,
    ) -> impl Future<Output = NativeResult<Observation>> + '_ {
        Box::pin(async move {
            let deadline = Instant::now() + OBSERVATION_BUDGET;
            // Both attempts retain the original deadline and field checks;
            // projected append data never becomes an actual source receipt.
            read_with_one_owned_append_resample(
                deadline,
                |deadline, geometry| self.read_native_document_until(deadline, geometry),
                |snapshot| self.native_owned_append_observation_is_pending(snapshot),
                self.native_owned_append_preparation_is_current(),
            )
            .await
        })
    }

    async fn read_native_document_until(
        &self,
        deadline: Instant,
        with_hint_geometry: bool,
    ) -> NativeResult<Observation> {
        if self.config.active_layout_backend() != lay::desktop::LayoutBackend::Kde
            || self.atomic.active
            || self.content_is_sensitive()
            || self.uses_native_terminal_input()
            || self.client_context.exact_surrounding_refresh_available
            || self.client_context.surrounding_text_callback_observed
        {
            return Err("native_not_eligible");
        }
        let token = self
            .context_token
            .clone()
            .filter(|t| {
                self.context_admission
                    .as_ref()
                    .is_some_and(|a| a.revalidate(t))
            })
            .ok_or("native_no_admission")?;
        bounded(deadline, async {
            let desktop = super::desktop::kde::observe_focused_window_until(deadline)?;
            if !crate::kde::display::chrome::supports_native_prefix(desktop.pid) {
                return Err("native_not_chrome");
            }
            // Keep native transport frames out of enclosing key/focus futures.
            // Cancellation still drops the same bounded operation and resources.
            let reusable = self
                .client_context
                .native_document_binding
                .as_ref()
                .filter(|binding| {
                    super::desktop::kde::native_window_continues(
                        &binding.identity.desktop,
                        &desktop,
                    ) && binding.focus_serial == self.client_context.focus_serial
                        && binding.owner_lease == self.client_context.runtime_owner_lease_identity
                        && binding.layout_generation == self.layout_gesture.layout_generation
                });
            let (connection, mut identity, snapshot) = if let Some(binding) = reusable {
                let snapshot =
                    bus::read_bound(&binding.connection, &binding.identity, with_hint_geometry)
                        .await?;
                (
                    binding.connection.clone(),
                    binding.identity.clone(),
                    snapshot,
                )
            } else {
                Box::pin(bus::read_focused(desktop.clone(), with_hint_geometry)).await?
            };
            let final_desktop = super::desktop::kde::observe_focused_window_until(deadline)?;
            if !super::desktop::kde::native_window_continues(&desktop, &final_desktop) {
                return Err("native_desktop_changed");
            }
            // The final authenticated window observation owns this receipt.
            identity.desktop = final_desktop;
            let NativeSnapshot {
                snapshot,
                hint_rect,
            } = snapshot;
            let hint_geometry = hint_rect.map(|rect| NativeHintGeometry {
                snapshot: snapshot.clone(),
                rect,
            });
            let binding = NativeDocumentBinding {
                identity,
                connection,
                token,
                focus_serial: self.client_context.focus_serial,
                owner_lease: self.client_context.runtime_owner_lease_identity,
                layout_generation: self.layout_gesture.layout_generation,
                tail_epoch: self.committed_tail.epoch,
                observer: None,
                last_display_frame: None,
                hint_geometry,
            };
            if !binding.is_current(self) {
                return Err("native_admission_changed");
            }
            Ok(Observation { binding, snapshot })
        })
        .await
    }

    pub(crate) fn install_native_document(&mut self, mut observation: Observation) -> bool {
        if !observation.binding.is_current(self) {
            crate::trace::record(
                serde_json::json!({
                    "kind": "ibus_native_document",
                    "stage": "install_refused",
                    "reason": "admission_changed",
                    "token_matches": self.context_token.as_ref() == Some(&observation.binding.token),
                    "scope_matches": self.context_word_scope.as_ref().is_some_and(|scope| observation.binding.token.matches_word_scope(scope)),
                    "admitted": self.context_admission.as_ref().is_some_and(|admission| admission.revalidate(&observation.binding.token)),
                    "focus_matches": self.client_context.focus_serial == observation.binding.focus_serial,
                    "lease_matches": self.client_context.runtime_owner_lease_identity == observation.binding.owner_lease,
                    "layout_matches": self.layout_gesture.layout_generation == observation.binding.layout_generation,
                    "tail_matches": self.committed_tail.epoch == observation.binding.tail_epoch,
                })
                .to_string(),
            );
            return false;
        }
        if self
            .client_context
            .native_document_binding
            .as_ref()
            .is_some_and(|previous| {
                !observation
                    .binding
                    .identity
                    .continues_from(&previous.identity)
            })
        {
            let previous = self
                .client_context
                .native_document_binding
                .as_ref()
                .unwrap();
            if previous.identity == observation.binding.identity {
                // An older receipt is not a different field. Discard that
                // completion without revoking the newer installed owner.
                crate::trace::record(
                    r#"{"kind":"ibus_native_document","stage":"install_refused","reason":"stale_window_receipt"}"#,
                );
                return false;
            }
            crate::trace::record(
                serde_json::json!({
                    "kind": "ibus_native_document",
                    "stage": "install_refused",
                    "reason": "field_identity_changed",
                    "desktop_matches": previous.identity.desktop == observation.binding.identity.desktop,
                    "bus_matches": previous.identity.bus == observation.binding.identity.bus,
                    "owner_matches": previous.identity.owner == observation.binding.identity.owner,
                    "application_matches": previous.identity.application == observation.binding.identity.application,
                    "field_matches": previous.identity.field == observation.binding.identity.field,
                })
                .to_string(),
            );
            // Reuse canonical revocation; a matching string/path on a later
            // A-B-A visit cannot resurrect the old input lease.
            self.client_context.native_document_binding = None;
            self.client_context.surrounding_text_snapshot = None;
            self.revoke_context_word();
            return false;
        }
        if self
            .client_context
            .native_document_binding
            .as_ref()
            .is_some_and(|previous| {
                !previous
                    .identity
                    .same_receipt_as(&observation.binding.identity)
            })
        {
            crate::trace::record(
                r#"{"kind":"ibus_native_document","stage":"field_receipt_rebound","same_physical_field":true,"forward_provider_lifetime":true}"#,
            );
        }
        let unchanged_snapshot = self
            .client_context
            .native_document_binding
            .as_ref()
            .is_some_and(|previous| {
                previous.is_current(self)
                    && previous.same_observation_owner(&observation.binding)
                    && self.client_context.surrounding_text_snapshot.as_ref()
                        == Some(&observation.snapshot)
            });
        if let Some(previous) = self.client_context.native_document_binding.as_ref() {
            if previous.same_observation_owner(&observation.binding) {
                observation.binding.observer = previous.observer.clone();
                if previous.token == observation.binding.token {
                    observation.binding.last_display_frame = previous.last_display_frame.clone();
                }
            }
        }
        // Data preparation follows an already admitted physical field across
        // a forward provider reconnect. Subscription/cache reuse above still
        // requires the exact transport receipt through same_observation_owner.
        let same_native_input_owner = self
            .client_context
            .native_document_binding
            .as_ref()
            .is_some_and(|previous| {
                observation
                    .binding
                    .identity
                    .continues_from(&previous.identity)
                    && previous.focus_serial == observation.binding.focus_serial
                    && previous.owner_lease == observation.binding.owner_lease
                    && previous.layout_generation == observation.binding.layout_generation
            });
        self.client_context.native_document_binding = Some(observation.binding);
        if same_native_input_owner {
            self.acknowledge_native_word_separator_reflow(&observation.snapshot);
        }
        // Re-reading one current document is not another input receipt. A new
        // revision would invalidate the displayed word's pair frame and start
        // pending prediction again just before Tab. Still replace the binding
        // and geometry; changed input/owner/text/caret keeps the ordinary path.
        if !unchanged_snapshot {
            self.observe_document_snapshot(Some(observation.snapshot));
        }
        if same_native_input_owner {
            self.acknowledge_native_closing_space();
            self.arm_native_observed_word_boundary();
            self.arm_native_observed_first_append_boundary();
        }
        true
    }

    fn observe_native_document_postcondition(&mut self) {
        if self.space_boundary_is_pending() {
            return;
        }
        self.acknowledge_native_closing_space();
        self.acknowledge_native_layout_space();
        let native_result = self
            .committed_tail
            .pending_visible_postcondition
            .as_ref()
            .is_some_and(|pending| pending.native_document.is_some());
        let token = (native_result
            && self.context_bridge_token.is_none()
            && self.native_document_is_current())
        .then(|| self.live_context_token())
        .flatten();
        if let Some(token) = token {
            // The original RPC has returned; its actual native result is now
            // observed under the same live owner. Keep the canonical result
            // verifier and settle its deferred layout publication in admission.
            let mut bridge = self.begin_context_bridge_output(Some(&token));
            let _ = bridge.observe_visible_postcondition();
            bridge.complete();
        } else {
            let _ = self.observe_visible_postcondition();
        }
    }

    fn settle_native_literal_input_mode(&mut self) {
        // Ready may preserve a literal preedit consumed during acquisition,
        // while resetting its per-word mode. This owned legacy preedit could
        // only start through ManagedCommit. Rejoin that same transport after
        // a fresh native observation confirms the entire owned tail; do not
        // migrate an established mode or manufacture any document/edit proof.
        if self.composition.word_input_mode.is_none()
            && self.native_document_is_current()
            && self.exact_owned_legacy_preedit_is_current()
            && self.current_external_snapshot_agrees_with_owned_tail()
            && self.initial_word_input_mode() == crate::engine::WordInputMode::ManagedCommit
        {
            self.composition.word_input_mode = Some(crate::engine::WordInputMode::ManagedCommit);
        }
    }

    fn native_observation_failed(&mut self, reason: &str) -> bool {
        // AddSelection and CommitText are observed asynchronously. While an
        // exact native result is already pending for this owner and field,
        // its intermediate selection is not another input owner. It supplies
        // no edit proof: discard the snapshot and await the unchanged final
        // verifier, which still requires the full actual unselected document.
        let awaiting_native_selection = reason == "native_selection_present"
            && self
                .committed_tail
                .pending_visible_postcondition
                .as_ref()
                .is_some_and(|pending| {
                    pending.dispatched_epoch == self.committed_tail.epoch
                        && pending.expected_external_snapshot.is_some()
                        && pending.native_document.as_ref().is_some_and(|expected| {
                            expected.is_current(self)
                                && self
                                    .client_context
                                    .native_document_binding
                                    .as_ref()
                                    .is_some_and(|actual| actual.identity == expected.identity)
                        })
                });
        // An unavailable native observation is not an input gap for the same
        // canonical IBus owner. Discard every document-derived edit proof,
        // but let the existing admitted key path continue literal input and
        // the bound observer await real text. It must authenticate the full
        // document again before any edit.
        let transient_read = matches!(
            reason,
            "native_deadline"
                | "native_document_changed"
                | "native_document_shape"
                | "native_desktop_unobserved"
        ) || awaiting_native_selection;
        let mut refused_at = "error_class";
        let waiting_for_observation = transient_read
            && {
                refused_at = "handoff";
                !self.context_handoff_sealed
            }
            && {
                refused_at = "admission_token";
                self.live_context_token().is_some_and(|token| {
                    refused_at = "word_scope";
                    self.context_word_scope
                        .as_ref()
                        .is_some_and(|scope| token.matches_word_scope(scope))
                })
            }
            && {
                refused_at = "native_binding";
                self.client_context
                    .native_document_binding
                    .as_ref()
                    .is_some_and(|binding| {
                        refused_at = "notification_owner";
                        observer::current(self, binding)
                    })
            };
        if crate::trace::enabled() && transient_read {
            let binding = self.client_context.native_document_binding.as_ref();
            crate::trace::record(serde_json::json!({
                "kind": "ibus_native_document",
                "stage": "transient_input_gate",
                "reason": reason,
                "allowed": waiting_for_observation,
                "refused_at": (!waiting_for_observation).then_some(refused_at),
                "callback_entered": self.context_callback_entered.is_some(),
                "word_scope_present": self.context_word_scope.is_some(),
                "tail_chars": self.committed_tail.buffer.chars().count(),
                "focus_matches": binding.map(|b| b.focus_serial == self.client_context.focus_serial),
                "lease_matches": binding.map(|b| b.owner_lease == self.client_context.runtime_owner_lease_identity),
                "layout_matches": binding.map(|b| b.layout_generation == self.layout_gesture.layout_generation),
                "active_path_matches": self.shared.lock().is_ok_and(|s| s.active_path.as_deref() == Some(self.path.as_str())),
            }).to_string());
        }
        if waiting_for_observation {
            // Keep only append provenance for this same notification owner.
            // The ordinary observation still invalidates every actual edit
            // proof; a new complete matching document is mandatory to edit.
            let preparation = self.client_context.managed_word_start.clone();
            self.observe_document_snapshot(None);
            if self
                .client_context
                .native_document_binding
                .as_ref()
                .is_some_and(|binding| observer::current(self, binding))
            {
                self.client_context.managed_word_start = preparation;
            }
            return true;
        }
        self.client_context.native_document_binding = None;
        self.client_context.surrounding_text_snapshot = None;
        self.revoke_context_word();
        false
    }

    /// Called only by the existing AuthorizedEdit executor. The caller marks
    /// CursorOrPreedit before entering: selection errors are partial effects.
    pub(crate) fn select_native_document_range<'a>(
        &'a self,
        expected: &'a SurroundingTextSnapshot,
        backspaces: u32,
    ) -> impl Future<Output = NativeResult<()>> + 'a {
        Box::pin(async move {
            let binding = self
                .client_context
                .native_document_binding
                .as_ref()
                .filter(|b| b.is_current(self))
                .ok_or("native_stale_binding")?;
            let range = exact_range(expected, backspaces)?;
            // A source observation has its own bounded I/O phase. Consuming
            // that time must not leave an accepted asynchronous selection
            // without its existing acknowledgement budget.
            let fresh = self
                .read_native_document_until(Instant::now() + OBSERVATION_BUDGET, false)
                .await?;
            if !fresh.binding.identity.continues_from(&binding.identity)
                || &fresh.snapshot != expected
                || !binding.is_current(self)
            {
                return Err("native_source_changed");
            }
            let deadline = Instant::now() + SELECTION_BUDGET;
            bounded(deadline, async {
                // Presentation I/O must never become a prerequisite for an
                // exact edit, including this outer source preflight.
                Box::pin(bus::select_exact(
                    &binding.connection,
                    &binding.identity,
                    expected,
                    range,
                ))
                .await?;
                if !binding.is_current(self)
                    || super::desktop::focused_window_until(
                        self.config.active_layout_backend(),
                        deadline,
                    )
                    .await
                    .as_ref()
                        != Some(&binding.identity.desktop)
                {
                    return Err("native_selected_target_changed");
                }
                Ok(())
            })
            .await
        })
    }
}

impl super::WindowInteraction {
    /// Cursor and completed-readout callbacks can wake an actual Chrome read.
    /// Neither callback proves text: the adapter authenticates the document.
    /// A current binding alone does not mean its suffix already matches input.
    pub(crate) async fn refresh_native_document_for_display(
        engine: &mut LayIbusEngine,
        output: &mut crate::output::EngineOutput<'_, '_>,
    ) -> zbus::fdo::Result<()> {
        Box::pin(async move {
            if !engine.native_display_observation_needed() {
                return Ok(());
            }
            // A key-time read may miss Chrome's first focused field or time
            // out before it publishes the composition. The subsequent real
            // cursor callback must also be able to establish the first binding;
            // requiring one here otherwise strands the literal preedit.
            // The full bounded read below still authenticates Chrome, field,
            // privacy, focus, admission and actual document text.
            if Self::refresh_native_document(engine, output).await?
                && engine.native_document_is_current()
            {
                engine.refresh_native_document_precognition(output).await?;
            }
            Ok(())
        })
        .await
    }

    /// False means an already-native field lost its current observation.
    /// Ordinary input may pass to the app; stale edits and retries may not.
    pub(crate) async fn refresh_native_document(
        engine: &mut LayIbusEngine,
        output: &mut crate::output::EngineOutput<'_, '_>,
    ) -> zbus::fdo::Result<bool> {
        // Bound the entire observation/transfer frame, including the existing
        // literal-commit helper, before it enters enclosing focus/key futures.
        Box::pin(async move {
        if engine.config.active_layout_backend() != lay::desktop::LayoutBackend::Kde
            || engine.atomic.active
            || engine.content_is_sensitive()
            || engine.uses_native_terminal_input()
            || !engine.client_context.surrounding_text_supported
            || engine.client_context.exact_surrounding_refresh_available
            || engine.client_context.surrounding_text_callback_observed
        {
            if engine.client_context.native_document_binding.take().is_some() {
                engine.client_context.surrounding_text_snapshot = None;
                engine.revoke_context_word();
            }
            return Ok(true);
        }
        let was_native = engine.client_context.native_document_binding.is_some();
        let started = Instant::now();
        let observation = engine.read_native_document().await;
        crate::trace::record(format!(
            r#"{{"kind":"ibus_native_document","stage":"source_read","ok":{},"elapsed_us":{},"reason":"{}"}}"#,
            observation.is_ok(),
            started.elapsed().as_micros(),
            observation
                .as_ref()
                .err()
                .copied()
                .unwrap_or("actual_document"),
        ));
        match observation {
            Ok(observation) => {
                if !engine.install_native_document(observation) {
                    return Ok(false);
                }
                let boundary_applied = engine.apply_space_boundary_after_client_commit(output).await?;
                if !boundary_applied {
                    engine.observe_native_document_postcondition();
                }
                observer::arm(engine, output).await;
                engine.settle_native_literal_input_mode();
                if crate::trace::enabled() && !engine.composition.buffer.is_empty() {
                    crate::trace::record(serde_json::json!({
                        "kind": "ibus_native_document", "stage": "literal_transfer_ready",
                        "composition_chars": engine.composition.buffer.chars().count(),
                        "owned": engine.exact_owned_legacy_preedit_is_current(),
                        "visible": engine.composition.preedit_visible,
                        "dirty": engine.composition.preedit_dirty,
                        "at_end": engine.composition.cursor == engine.composition.buffer.chars().count(),
                        "managed": engine.composition.word_input_mode == Some(crate::engine::WordInputMode::ManagedCommit),
                        "sealed": engine.context_handoff_sealed,
                        "native_current": engine.native_document_is_current(),
                        "scope_current": engine.context_word_scope.as_ref().is_some_and(|scope| engine.live_context_token().is_some_and(|token| token.matches_word_scope(scope))),
                        "tail_agrees": engine.current_external_snapshot_agrees_with_owned_tail(),
                        "known": engine.context_word_is_known(),
                        "pending": engine.composition.preedit_display_only_pending,
                    }).to_string());
                }
                let transferred = engine.commit_observed_literal_preedit(output).await?;
                if crate::trace::enabled() {
                crate::trace::record(serde_json::json!({
                    "kind": "ibus_native_document", "stage": "refresh_complete",
                    "transferred": transferred,
                    "known": engine.context_word_is_known(),
                    "pending": engine.composition.preedit_display_only_pending,
                    "exact_managed": engine.exact_managed_surrounding_word_is_current(),
                    "frame_available": engine.capture_input_frame_identity().is_some(),
                    "suffix_readout": engine.context_observed_suffix_readout_is_current(),
                    "observed_suffix_chars": engine.context_word_scope.map(|scope| scope.lineage().observed_suffix_chars),
                    "tail_token_chars": engine.last_tail_token_text().chars().count(),
                    "composition_chars": engine.composition.buffer.chars().count(),
                }).to_string());
                }
                Ok(true)
            }
            Err(reason) if was_native => Ok(engine.native_observation_failed(reason)),
            Err(_) => Ok(true),
        }
        })
        .await
    }
}
