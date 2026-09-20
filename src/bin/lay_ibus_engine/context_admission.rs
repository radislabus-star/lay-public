//! Bounded connection-local metadata admission for TD-121.
//!
//! This module deliberately owns no text and emits no edit.  Its successful
//! output is a single-use grant identifying the source tail epoch which later
//! engine wiring must revalidate before copying the existing tail.

#[path = "context_admission/adapter.rs"]
mod adapter;
#[path = "context_admission/ordered_merge.rs"]
mod ordered_merge;
#[path = "context_admission/rendezvous.rs"]
mod rendezvous;

use std::collections::VecDeque;
use std::time::{Duration, Instant};

#[cfg(test)]
pub(crate) use adapter::tests::word_scope::residuals::assert_window_interaction_reset_rereceipt_contract;
pub(crate) use adapter::{
    ActivationOutcome, AdapterConfig, AdapterError, ContextAdmissionAdapter, KeyCallback,
    PendingContextAdapter,
};
pub(crate) use ordered_merge::{join_ordered_streams, next_ordered_message};
pub(crate) use rendezvous::{
    rendezvous_stamp, CallbackStampStore, RendezvousFailure, RendezvousGuard, RendezvousOutcome,
};

const MAX_IDENTIFIER_BYTES: usize = 1_024;
const MAX_UNSETTLED_KEYS: usize = 64;
const SPACE_FULL_WAIT_BUDGET_US: u64 = 3_500;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct ConnectionGeneration(pub(crate) u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct OwnerGeneration(pub(crate) u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct ActivationGeneration(pub(crate) u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct LineageGeneration(pub(crate) u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct RequestGeneration(pub(crate) u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct TicketId(pub(crate) u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct FrameGeneration(pub(crate) u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct BarrierNonce(pub(crate) u64);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct EnginePath(String);

impl EnginePath {
    pub(crate) fn new(value: impl Into<String>) -> Option<Self> {
        bounded_identifier(value).map(Self)
    }

    pub(crate) fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct CanonicalContextPath(String);

impl CanonicalContextPath {
    pub(crate) fn new(value: impl Into<String>) -> Option<Self> {
        bounded_identifier(value).map(Self)
    }

    pub(crate) fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct EngineProfile(String);

impl EngineProfile {
    pub(crate) fn new(value: impl Into<String>) -> Option<Self> {
        bounded_identifier(value).map(Self)
    }

    pub(crate) fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct ContextKey {
    pub(crate) connection: ConnectionGeneration,
    pub(crate) path: CanonicalContextPath,
}

impl ContextKey {
    pub(crate) fn new(connection: ConnectionGeneration, path: impl Into<String>) -> Option<Self> {
        Some(Self {
            connection,
            path: CanonicalContextPath::new(path)?,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct EngineOwner {
    pub(crate) path: EnginePath,
    pub(crate) generation: OwnerGeneration,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct FocusActivation {
    pub(crate) context: ContextKey,
    pub(crate) generation: ActivationGeneration,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum WordCompleteness {
    KnownStart,
    UnknownStart,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct WordLineage {
    pub(crate) generation: LineageGeneration,
    pub(crate) completeness: WordCompleteness,
    /// Contiguous mirrored suffix available only to an explicit manual action.
    /// Zero for KnownStart; this never upgrades word completeness.
    pub(crate) observed_suffix_chars: u32,
    /// Earliest scalar offset whose retained suffix includes an observed
    /// boundary. Prefix trimming may make this floor conservative; it must
    /// never be moved left by inspecting an unproven mirror.
    pub(crate) observed_boundary_floor: Option<u32>,
}

/// Completeness is an authority state, not a convenience boolean.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct WordScope {
    lineage: WordLineage,
    #[cfg(test)]
    last_boundary: Option<u64>,
}

impl WordScope {
    pub(crate) fn new(lineage: WordLineage) -> Self {
        Self {
            lineage,
            #[cfg(test)]
            last_boundary: None,
        }
    }

    pub(crate) fn lineage(&self) -> WordLineage {
        self.lineage
    }

    /// Returns authority for the word just closed, then arms the next word.
    pub(crate) fn close_at_observed_boundary(
        &mut self,
        _boundary_revision: u64,
        retained_boundary: Option<u32>,
    ) -> WordCompleteness {
        let closed = self.lineage.completeness;
        let observed_boundary_floor = retained_boundary.map(|boundary| {
            self.lineage
                .observed_boundary_floor
                .map_or(boundary, |floor| floor.min(boundary))
        });
        self.lineage = WordLineage {
            generation: LineageGeneration(next_generation(self.lineage.generation.0)),
            completeness: WordCompleteness::KnownStart,
            observed_suffix_chars: 0,
            observed_boundary_floor,
        };
        #[cfg(test)]
        {
            self.last_boundary = Some(_boundary_revision);
        }
        closed
    }

    /// Emptying, reset and re-receipt never manufacture a known start.
    pub(crate) fn keep_or_revoke_unknown(&mut self) {
        #[cfg(test)]
        {
            if self.lineage.completeness == WordCompleteness::UnknownStart {
                self.last_boundary = None;
            }
        }
    }

    pub(crate) fn observe_tail_append(&mut self, retained_chars: u32) {
        self.observe_tail_append_span(retained_chars, 1);
    }

    pub(crate) fn observe_tail_append_span(&mut self, retained_chars: u32, appended_chars: u32) {
        if self.lineage.completeness == WordCompleteness::UnknownStart {
            self.lineage.observed_suffix_chars = self
                .lineage
                .observed_suffix_chars
                .saturating_add(appended_chars)
                .min(retained_chars);
        }
    }

    pub(crate) fn observe_tail_backspace(&mut self) {
        self.lineage.observed_suffix_chars = self.lineage.observed_suffix_chars.saturating_sub(1);
    }

    pub(crate) fn observe_soft_reset_rereceipt(&mut self, observed_suffix_chars: u32) -> bool {
        if self.lineage.completeness != WordCompleteness::UnknownStart
            || observed_suffix_chars <= self.lineage.observed_suffix_chars
        {
            return false;
        }
        self.lineage.observed_suffix_chars = observed_suffix_chars;
        true
    }

    /// Reopening an earlier word retires the old token even when its start is
    /// still proved by a separator in the retained observed range.
    pub(crate) fn reopen_at_retained_boundary(&mut self, retained_boundary: Option<u32>) {
        let retained_floor = self
            .lineage
            .observed_boundary_floor
            .filter(|floor| retained_boundary.is_some_and(|boundary| boundary >= *floor));
        self.revoke_for_input_gap();
        if let Some(floor) = retained_floor {
            self.lineage.completeness = WordCompleteness::KnownStart;
            self.lineage.observed_boundary_floor = Some(floor);
        }
    }

    pub(crate) fn revoke_for_input_gap(&mut self) {
        self.lineage = WordLineage {
            generation: LineageGeneration(next_generation(self.lineage.generation.0)),
            completeness: WordCompleteness::UnknownStart,
            observed_suffix_chars: 0,
            observed_boundary_floor: None,
        };
        #[cfg(test)]
        {
            self.last_boundary = None;
        }
    }

    #[cfg(test)]
    pub(crate) fn backspace_crossed_boundary(&mut self, boundary_revision: u64) {
        if self.last_boundary == Some(boundary_revision) {
            self.revoke_for_input_gap();
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct HeaderKey {
    pub(crate) connection: ConnectionGeneration,
    pub(crate) sender: String,
    pub(crate) serial: u32,
    pub(crate) path: String,
    pub(crate) member: String,
}

impl HeaderKey {
    pub(crate) fn new(
        connection: ConnectionGeneration,
        sender: impl Into<String>,
        serial: u32,
        path: impl Into<String>,
        member: impl Into<String>,
    ) -> Option<Self> {
        if serial == 0 {
            return None;
        }
        Some(Self {
            connection,
            sender: bounded_identifier(sender)?,
            serial,
            path: bounded_identifier(path)?,
            member: bounded_identifier(member)?,
        })
    }

    pub(crate) fn from_zbus_header(
        connection: ConnectionGeneration,
        header: &zbus::message::Header<'_>,
    ) -> Option<Self> {
        Self::new(
            connection,
            header.sender()?.as_str(),
            header.primary().serial_num().get(),
            header.path()?.as_str(),
            header.member()?.as_str(),
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum IngressDisposition {
    Passive,
    Key {
        owner: EngineOwner,
        revocation: u64,
    },
    Factory {
        ticket: TicketId,
        expected_profile: EngineProfile,
    },
    FocusOut {
        owner: EngineOwner,
        ticket: TicketId,
    },
    Disable {
        owner: EngineOwner,
    },
    Revocation {
        owner: EngineOwner,
    },
    UnchangedContentType {
        token: AdmissionToken,
        value: (u32, u32),
    },
    StaleContext,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct IngressStamp<P = zbus::message::Sequence> {
    pub(crate) header: HeaderKey,
    pub(crate) position: P,
    pub(crate) disposition: IngressDisposition,
}

impl IngressStamp<zbus::message::Sequence> {
    pub(crate) fn from_message(
        connection: ConnectionGeneration,
        message: &zbus::Message,
    ) -> Option<Self> {
        Some(Self {
            header: HeaderKey::from_zbus_header(connection, &message.header())?,
            position: message.recv_position(),
            disposition: IngressDisposition::Passive,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SenderBindings {
    ibus_owner: String,
    marker_sender: String,
    engine_dispatch_sender: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum HeaderRole {
    IbusLifecycle,
    PrivateMarker,
    EngineCallback,
}

impl SenderBindings {
    /// All names come from authenticated bootstrap state; none is hard-coded.
    pub(crate) fn new(
        ibus_owner: impl Into<String>,
        marker_sender: impl Into<String>,
        engine_dispatch_sender: impl Into<String>,
    ) -> Option<Self> {
        let ibus_owner = bounded_identifier(ibus_owner)?;
        let marker_sender = bounded_identifier(marker_sender)?;
        let engine_dispatch_sender = bounded_identifier(engine_dispatch_sender)?;
        if ibus_owner == marker_sender
            || ibus_owner == engine_dispatch_sender
            || marker_sender == engine_dispatch_sender
        {
            return None;
        }
        Some(Self {
            ibus_owner,
            marker_sender,
            engine_dispatch_sender,
        })
    }

    pub(crate) fn accepts(&self, role: HeaderRole, header: &HeaderKey) -> bool {
        let expected = match role {
            HeaderRole::IbusLifecycle => &self.ibus_owner,
            HeaderRole::PrivateMarker => &self.marker_sender,
            HeaderRole::EngineCallback => &self.engine_dispatch_sender,
        };
        header.sender == *expected
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ReceiptOrigin {
    Native,
    CompatibilityProperty,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum GlobalEngineMode {
    Verified,
    UnsupportedOrFalse,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum GlobalProfile {
    Lay(EngineProfile),
    Foreign(EngineProfile),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum TicketKind {
    Factory,
    Reflexive,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TicketStatus {
    Pending,
    Ready,
    Revoked,
    Consumed,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct SourceSeal<P> {
    tail_epoch: u64,
    lineage: WordLineage,
    position: P,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct HandoffTicket<P> {
    id: TicketId,
    kind: TicketKind,
    source_owner: EngineOwner,
    source_activation: FocusActivation,
    target_path: Option<EnginePath>,
    expected_target_profile: EngineProfile,
    factory_position: P,
    revocation_at_open: u64,
    focus_out_position: Option<P>,
    disable_position: Option<P>,
    target_focus_position: Option<P>,
    source_seal: Option<SourceSeal<P>>,
    status: TicketStatus,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ContextRequest<P> {
    generation: RequestGeneration,
    nonce: BarrierNonce,
    target: RequestTarget,
    origin: ReceiptOrigin,
    lifecycle_revision: u64,
    reply: Option<(ContextKey, P)>,
    marker_position: Option<P>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RequestTarget {
    Transfer(TicketId),
    SourceFree,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct SourceFreeActivation<P> {
    target_path: EnginePath,
    expected_target_profile: EngineProfile,
    focus_position: P,
    status: TicketStatus,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct SourceFreeFactoryReservation<P> {
    id: TicketId,
    target_path: Option<EnginePath>,
    expected_target_profile: EngineProfile,
    factory_position: P,
    status: TicketStatus,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct UnsettledKey {
    owner: EngineOwner,
    header: HeaderKey,
    word_effect: KeyWordEffect,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum KeyWordEffect {
    PossibleMutation,
    LegacyShiftObservation,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ExactManualSourceSnapshotReceipt {
    source_token: AdmissionToken,
    pub(crate) tail_epoch: u64,
    pub(crate) tail: String,
    pub(crate) observation_revision: u64,
    pub(crate) expires_at: Instant,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct TransferGrant {
    /// Snapshot of the existing reducer revision, not a separate generation.
    pub(crate) revocation: u64,
    pub(crate) ticket: TicketId,
    pub(crate) kind: TicketKind,
    pub(crate) source_owner: EngineOwner,
    pub(crate) target_owner: EngineOwner,
    pub(crate) target_activation: FocusActivation,
    pub(crate) lineage: WordLineage,
    pub(crate) source_tail_epoch: u64,
    pub(crate) frame_generation: FrameGeneration,
    /// Candidate/prepared/Tab/feedback authority must be regenerated.
    pub(crate) invalidate_prior_authority: bool,
    pub(crate) receipt_origin: ReceiptOrigin,
    pub(crate) exact_manual_snapshot: Option<Box<ExactManualSourceSnapshotReceipt>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ActivationGrant {
    pub(crate) revocation: u64,
    pub(crate) target_owner: EngineOwner,
    pub(crate) target_activation: FocusActivation,
    pub(crate) lineage: WordLineage,
    pub(crate) tail_epoch: u64,
    pub(crate) frame_generation: FrameGeneration,
    pub(crate) receipt_origin: ReceiptOrigin,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct SettledWordState {
    pub(crate) tail_epoch: u64,
    pub(crate) lineage: WordLineage,
    content_type: Option<(u32, u32)>,
}

impl SettledWordState {
    pub(crate) fn from_scope(tail_epoch: u64, scope: &WordScope) -> Self {
        Self {
            tail_epoch,
            lineage: scope.lineage(),
            content_type: None,
        }
    }

    pub(crate) fn with_content_type(mut self, purpose: u32, hints: u32) -> Self {
        self.content_type = Some((purpose, hints));
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct AdmissionToken {
    connection: ConnectionGeneration,
    revocation: u64,
    owner: EngineOwner,
    activation: FocusActivation,
    lineage: WordLineage,
}

impl AdmissionToken {
    pub(crate) fn matches_word_scope(&self, scope: &WordScope) -> bool {
        self.lineage == scope.lineage()
    }

    pub(crate) fn owner_path(&self) -> &str {
        self.owner.path.as_str()
    }

    pub(crate) fn matches_owner(&self, owner: &EngineOwner) -> bool {
        &self.owner == owner
    }

    pub(crate) fn word_scope(&self) -> WordScope {
        WordScope::new(self.lineage)
    }

    /// Opaque field receipt for bridge-side exact-tail leases. This projects
    /// the admitted IBus context identity without storing a fallback receipt in
    /// engine frame state.
    pub(crate) fn exact_field_receipt(&self) -> String {
        format!(
            "context-admission:{}\u{1f}{}",
            self.activation.context.connection.0,
            self.activation.context.path.as_str()
        )
    }
}

/// Stable identity for layout work owned by one live input context.
///
/// Layout synchronization is allowed to outlive an ordinary word settlement,
/// so this token deliberately excludes `WordLineage`. It still fails closed
/// across connection, revocation, owner, or focus-activation changes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct LayoutIntentToken {
    connection: ConnectionGeneration,
    revocation: u64,
    owner: EngineOwner,
    activation: FocusActivation,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum AdmissionStatus {
    Idle,
    Pending,
    Ready,
    Revoked,
    Consumed,
}

#[derive(Clone, Copy)]
pub(crate) struct AdmissionDiagnosticSnapshot {
    pub(crate) reducer_status: &'static str,
    pub(crate) unsettled_count: usize,
    pub(crate) owner_generation: Option<u64>,
    pub(crate) activation_generation: Option<u64>,
}

/// Actual TD-121 reducer. `P` is zbus's opaque `Sequence` in production and a
/// deterministic order token in pure tests; no conversion between them exists.
#[derive(Debug)]
pub(crate) struct ContextAdmissionReducer<P = zbus::message::Sequence> {
    connection: ConnectionGeneration,
    mode: GlobalEngineMode,
    profile: GlobalProfile,
    revocation: u64,
    owner: Option<EngineOwner>,
    activation: Option<FocusActivation>,
    lineage: WordLineage,
    latest_tail_epoch: u64,
    settled_content_type: Option<(u32, u32)>,
    ticket: Option<HandoffTicket<P>>,
    source_free_factory: Option<SourceFreeFactoryReservation<P>>,
    source_free: Option<SourceFreeActivation<P>>,
    request: Option<ContextRequest<P>>,
    unsettled: VecDeque<UnsettledKey>,
    exact_manual_snapshot: Option<ExactManualSourceSnapshotReceipt>,
    next_owner: u64,
    next_activation: u64,
    next_lineage: u64,
    next_request: u64,
    next_ticket: u64,
    next_frame: u64,
}

impl<P> ContextAdmissionReducer<P>
where
    P: Clone + Ord,
{
    pub(crate) fn new(
        connection: ConnectionGeneration,
        mode: GlobalEngineMode,
        profile: GlobalProfile,
    ) -> Self {
        Self {
            connection,
            mode,
            profile,
            revocation: 1,
            owner: None,
            activation: None,
            lineage: WordLineage {
                generation: LineageGeneration(1),
                completeness: WordCompleteness::UnknownStart,
                observed_suffix_chars: 0,
                observed_boundary_floor: None,
            },
            latest_tail_epoch: 0,
            settled_content_type: None,
            ticket: None,
            source_free_factory: None,
            source_free: None,
            request: None,
            unsettled: VecDeque::with_capacity(MAX_UNSETTLED_KEYS),
            exact_manual_snapshot: None,
            next_owner: 1,
            next_activation: 1,
            next_lineage: 1,
            next_request: 1,
            next_ticket: 1,
            next_frame: 1,
        }
    }

    /// Bootstrap/wiring supplies a proven canonical source; this never reads or
    /// copies the source text.
    #[cfg(test)]
    pub(crate) fn establish_source(
        &mut self,
        path: EnginePath,
        context: ContextKey,
        completeness: WordCompleteness,
        tail_epoch: u64,
    ) -> Option<EngineOwner> {
        if self.mode != GlobalEngineMode::Verified || context.connection != self.connection {
            self.revoke();
            return None;
        }
        let completeness = if self.owner.is_some() || self.ticket.is_some() {
            WordCompleteness::UnknownStart
        } else {
            completeness
        };
        self.ticket = None;
        self.source_free_factory = None;
        self.source_free = None;
        self.request = None;
        self.unsettled.clear();
        let owner = EngineOwner {
            path,
            generation: OwnerGeneration(self.take_owner_generation()),
        };
        let activation = FocusActivation {
            context,
            generation: ActivationGeneration(self.take_activation_generation()),
        };
        let lineage_generation = self.take_lineage_generation();
        self.lineage = WordLineage {
            generation: LineageGeneration(lineage_generation),
            completeness,
            observed_suffix_chars: 0,
            observed_boundary_floor: None,
        };
        self.latest_tail_epoch = tail_epoch;
        self.owner = Some(owner.clone());
        self.activation = Some(activation);
        Some(owner)
    }

    pub(crate) fn owner(&self) -> Option<&EngineOwner> {
        self.owner.as_ref()
    }

    pub(crate) fn activation(&self) -> Option<&FocusActivation> {
        self.activation.as_ref()
    }

    #[cfg(test)]
    pub(crate) fn lineage(&self) -> WordLineage {
        self.lineage
    }

    pub(crate) fn revocation_generation(&self) -> u64 {
        self.revocation
    }

    pub(crate) fn status(&self) -> AdmissionStatus {
        self.ticket
            .as_ref()
            .map(|ticket| ticket.status)
            .or_else(|| {
                self.source_free_factory
                    .as_ref()
                    .map(|reservation| reservation.status)
            })
            .or_else(|| {
                self.source_free
                    .as_ref()
                    .map(|activation| activation.status)
            })
            .map_or(AdmissionStatus::Idle, |status| match status {
                TicketStatus::Pending => AdmissionStatus::Pending,
                TicketStatus::Ready => AdmissionStatus::Ready,
                TicketStatus::Revoked => AdmissionStatus::Revoked,
                TicketStatus::Consumed => AdmissionStatus::Consumed,
            })
    }

    pub(crate) fn diagnostic_snapshot(&self) -> AdmissionDiagnosticSnapshot {
        AdmissionDiagnosticSnapshot {
            reducer_status: match self.status() {
                AdmissionStatus::Idle => "idle",
                AdmissionStatus::Pending => "pending",
                AdmissionStatus::Ready => "ready",
                AdmissionStatus::Revoked => "revoked",
                AdmissionStatus::Consumed => "consumed",
            },
            unsettled_count: self.unsettled.len(),
            owner_generation: self.owner.as_ref().map(|owner| owner.generation.0),
            activation_generation: self
                .activation
                .as_ref()
                .map(|activation| activation.generation.0),
        }
    }

    pub(crate) fn open_factory_request(
        &mut self,
        expected_target_profile: EngineProfile,
        factory_position: P,
    ) -> Option<TicketId> {
        if self.mode != GlobalEngineMode::Verified {
            self.revoke();
            return None;
        }
        // A terminal transfer retains stale identities, not a reusable source.
        // Only a later factory may retire it and reacquire an empty context.
        let retires_revoked_transfer = self.ticket.as_ref().is_some_and(|ticket| {
            ticket.status == TicketStatus::Revoked
                && factory_position > ticket.factory_position
                && [
                    ticket.focus_out_position.as_ref(),
                    ticket.disable_position.as_ref(),
                    ticket.target_focus_position.as_ref(),
                ]
                .into_iter()
                .flatten()
                .all(|position| &factory_position > position)
        });
        if retires_revoked_transfer {
            self.revoke();
            self.ticket = None;
            self.owner = None;
            self.activation = None;
            self.source_free_factory = None;
            self.source_free = None;
        }
        // A strictly later factory may replace only an empty reservation. It
        // carries no owner, activation, source tail, or acquisition request.
        let supersedes_empty_factory = self.owner.is_none()
            && self.activation.is_none()
            && self.ticket.is_none()
            && self.source_free.is_none()
            && self.request.is_none()
            && self
                .source_free_factory
                .as_ref()
                .is_some_and(|reservation| {
                    reservation.status == TicketStatus::Pending
                        && factory_position > reservation.factory_position
                });
        if supersedes_empty_factory {
            self.source_free_factory = None;
        }
        // A later factory may replace an empty acquisition which has not
        // admitted any owner yet. No source tail or transfer authority exists.
        let supersedes_empty_acquisition = self.owner.is_none()
            && self.activation.is_none()
            && self.ticket.is_none()
            && self.source_free.as_ref().is_some_and(|activation| {
                matches!(
                    activation.status,
                    TicketStatus::Pending | TicketStatus::Ready
                ) && factory_position > activation.focus_position
            })
            && self.request.as_ref().is_some_and(|request| {
                matches!(request.target, RequestTarget::SourceFree)
                    && request.lifecycle_revision == self.revocation
            });
        if supersedes_empty_acquisition {
            self.revoke();
            self.source_free = None;
        }
        if self
            .ticket
            .as_ref()
            .is_some_and(|ticket| !matches!(ticket.status, TicketStatus::Consumed))
        {
            self.revoke();
            return None;
        }
        if self.ticket.is_some() {
            self.ticket = None;
            self.request = None;
        }
        if self
            .source_free_factory
            .as_ref()
            .is_some_and(|reservation| {
                !matches!(
                    reservation.status,
                    TicketStatus::Consumed | TicketStatus::Revoked
                )
            })
            || self.source_free.as_ref().is_some_and(|activation| {
                !matches!(
                    activation.status,
                    TicketStatus::Consumed | TicketStatus::Revoked
                )
            })
        {
            self.revoke();
            return None;
        }
        // A published source-free owner may not yet have installed its empty
        // tail epoch. Factory ingress must retain that one-use bind receipt.
        if self
            .source_free
            .as_ref()
            .is_none_or(|activation| activation.status != TicketStatus::Consumed)
        {
            self.source_free = None;
        }
        let id = TicketId(self.take_ticket_id());
        let (Some(source_owner), Some(source_activation)) =
            (self.owner.clone(), self.activation.clone())
        else {
            self.ticket = None;
            self.source_free_factory = Some(SourceFreeFactoryReservation {
                id,
                target_path: None,
                expected_target_profile,
                factory_position,
                status: TicketStatus::Pending,
            });
            return Some(id);
        };
        self.source_free_factory = None;
        self.ticket = Some(HandoffTicket {
            id,
            kind: TicketKind::Factory,
            source_owner,
            source_activation,
            target_path: None,
            expected_target_profile,
            factory_position,
            revocation_at_open: self.revocation,
            focus_out_position: None,
            disable_position: None,
            target_focus_position: None,
            source_seal: None,
            status: TicketStatus::Pending,
        });
        Some(id)
    }

    pub(crate) fn bind_factory_target(
        &mut self,
        ticket_id: TicketId,
        target_path: EnginePath,
    ) -> bool {
        if let Some(ticket) = self.ticket.as_mut() {
            if ticket.id != ticket_id {
                return false;
            }
            if ticket.kind != TicketKind::Factory
                || ticket.status != TicketStatus::Pending
                || ticket.target_path.is_some()
            {
                self.revoke();
                return false;
            }
            ticket.target_path = Some(target_path);
            return true;
        }
        let Some(reservation) = self.source_free_factory.as_mut() else {
            return false;
        };
        if reservation.id != ticket_id {
            return false;
        }
        if reservation.status != TicketStatus::Pending || reservation.target_path.is_some() {
            self.revoke();
            return false;
        }
        reservation.target_path = Some(target_path);
        true
    }

    pub(crate) fn observe_key_start(
        &mut self,
        owner: &EngineOwner,
        header: HeaderKey,
        word_effect: KeyWordEffect,
    ) -> bool {
        if self.owner.as_ref() != Some(owner) {
            return false;
        }
        if self.unsettled.len() == MAX_UNSETTLED_KEYS
            || self.unsettled.iter().any(|entry| entry.header == header)
        {
            self.revoke();
            return false;
        }
        self.unsettled.push_back(UnsettledKey {
            owner: owner.clone(),
            header,
            word_effect,
        });
        true
    }

    pub(crate) fn settle_key(
        &mut self,
        owner: &EngineOwner,
        header: &HeaderKey,
        settled: SettledWordState,
    ) -> bool {
        if self.owner.as_ref() != Some(owner) {
            return false;
        }
        let Some(index) = self
            .unsettled
            .iter()
            .position(|entry| &entry.owner == owner && &entry.header == header)
        else {
            self.revoke();
            return false;
        };
        if settled.tail_epoch < self.latest_tail_epoch
            || settled.lineage.generation < self.lineage.generation
        {
            self.revoke();
            return false;
        }
        self.unsettled.remove(index);
        self.latest_tail_epoch = settled.tail_epoch;
        self.lineage = settled.lineage;
        self.settled_content_type = settled.content_type;
        self.next_lineage = self
            .next_lineage
            .max(next_generation(settled.lineage.generation.0));
        self.try_ready();
        true
    }

    /// A completed but unhandled observation is an input gap, never a seal.
    /// Match the exact outstanding key before revoking this owner's word;
    /// delayed callbacks cannot revoke a successor or an unrelated request.
    pub(crate) fn abandon_key(&mut self, owner: &EngineOwner, header: &HeaderKey) -> bool {
        if self.owner.as_ref() != Some(owner)
            || !self
                .unsettled
                .iter()
                .any(|key| &key.owner == owner && &key.header == header)
        {
            return false;
        }
        if self.discard_pending_target_word(&owner.path) {
            return true;
        }
        self.revoke();
        true
    }

    /// A Firefox soft Reset may revoke the local suffix witness after the
    /// reducer has already moved to the post-Reset token. The next exact manual
    /// action can bind that current zero-count UnknownStart lineage only when
    /// the post-Reset token is still live and no text epoch changed.
    pub(crate) fn settle_soft_reset_rereceipt(
        &mut self,
        owner: &EngineOwner,
        reset_token: &AdmissionToken,
        settled: SettledWordState,
    ) -> bool {
        if self.owner.as_ref() != Some(owner)
            || !self.revalidate_bridge(reset_token)
            || settled.tail_epoch != self.latest_tail_epoch
            || settled.lineage.generation != self.lineage.generation
            || settled.lineage.observed_boundary_floor != self.lineage.observed_boundary_floor
            || self.lineage.completeness != WordCompleteness::UnknownStart
            || settled.lineage.completeness != WordCompleteness::UnknownStart
            || settled.lineage.observed_suffix_chars <= self.lineage.observed_suffix_chars
        {
            return false;
        }
        self.lineage = settled.lineage;
        true
    }

    /// The bridge holds the engine write lock from fence validation through
    /// output publication. It cannot settle a key or change word provenance.
    pub(crate) fn settle_bridge_output(
        &mut self,
        token: &AdmissionToken,
        settled: SettledWordState,
    ) -> bool {
        if !self.revalidate_bridge(token)
            || settled.lineage != token.lineage
            || settled.tail_epoch <= self.latest_tail_epoch
        {
            return false;
        }
        self.latest_tail_epoch = settled.tail_epoch;
        true
    }

    pub(crate) fn publish_exact_manual_snapshot(
        &mut self,
        token: &AdmissionToken,
        tail_epoch: u64,
        tail: String,
        observation_revision: u64,
        expires_at: Instant,
    ) -> bool {
        if tail.is_empty()
            || expires_at <= Instant::now()
            || !self.revalidate_bridge(token)
            || tail_epoch != self.latest_tail_epoch
            || token.lineage != self.lineage
        {
            if self.owner.as_ref() == Some(&token.owner)
                && self.activation.as_ref() == Some(&token.activation)
            {
                self.exact_manual_snapshot = None;
            }
            return false;
        }
        self.exact_manual_snapshot = Some(ExactManualSourceSnapshotReceipt {
            source_token: token.clone(),
            tail_epoch,
            tail,
            observation_revision,
            expires_at,
        });
        true
    }

    pub(crate) fn invalidate_exact_manual_snapshot(&mut self, owner: &EngineOwner) {
        if self.owner.as_ref() == Some(owner) {
            self.exact_manual_snapshot = None;
        }
    }

    /// Repeated metadata is not a new field. A pending target still needs the
    /// exact source seal and same-context receipt before it gains any word.
    fn unchanged_content_type_token(
        &self,
        path: &EnginePath,
        value: (u32, u32),
    ) -> Option<AdmissionToken> {
        let token = self.admission_token()?;
        self.revalidate_content_type(&token, path, value)
            .then_some(token)
    }

    fn revalidate_content_type(
        &self,
        token: &AdmissionToken,
        path: &EnginePath,
        value: (u32, u32),
    ) -> bool {
        if self.mode != GlobalEngineMode::Verified
            || !matches!(self.profile, GlobalProfile::Lay(_))
            || token.connection != self.connection
            || token.revocation != self.revocation
            || token.lineage != self.lineage
            || self.settled_content_type != Some(value)
        {
            return false;
        }
        if self.revalidate(token) && &token.owner.path == path {
            return true;
        }
        self.ticket.as_ref().is_some_and(|ticket| {
            ticket.source_owner == token.owner
                && ticket.source_activation == token.activation
                && ticket.revocation_at_open == self.revocation
                && ticket.target_path.as_ref() == Some(path)
                && match ticket.status {
                    TicketStatus::Pending | TicketStatus::Ready => self.revalidate(token),
                    TicketStatus::Consumed => {
                        self.owner.as_ref().is_some_and(|owner| &owner.path == path)
                    }
                    TicketStatus::Revoked => false,
                }
        })
    }

    fn pending_transfer_target(&self) -> Option<EnginePath> {
        let RequestTarget::Transfer(ticket_id) = self.request.as_ref()?.target else {
            return None;
        };
        self.ticket.as_ref().and_then(|ticket| {
            (ticket.id == ticket_id
                && matches!(ticket.status, TicketStatus::Pending | TicketStatus::Ready))
            .then(|| ticket.target_path.clone())
            .flatten()
        })
    }

    /// Discard only the word carried by the exact in-flight target. The
    /// original Get/native receipt and marker fence retain their identities;
    /// the next grant is the existing source-free UnknownStart outcome.
    pub(crate) fn discard_pending_target_word(&mut self, target_path: &EnginePath) -> bool {
        let Some(request) = self.request.clone() else {
            return false;
        };
        let RequestTarget::Transfer(ticket_id) = request.target else {
            return false;
        };
        let Some(ticket) = self.ticket.as_ref() else {
            return false;
        };
        if ticket.id != ticket_id
            || !matches!(ticket.status, TicketStatus::Pending | TicketStatus::Ready)
            || ticket.target_path.as_ref() != Some(target_path)
        {
            return false;
        }
        let Some(focus_position) = ticket.target_focus_position.clone() else {
            return false;
        };
        let expected_target_profile = ticket.expected_target_profile.clone();
        if self.mode != GlobalEngineMode::Verified
            || !matches!(self.profile, GlobalProfile::Lay(_))
            || request.lifecycle_revision != self.revocation
        {
            self.revoke();
            return false;
        }

        self.revoke();
        self.owner = None;
        self.activation = None;
        self.ticket = None;
        self.source_free_factory = None;
        self.latest_tail_epoch = 0;
        self.source_free = Some(SourceFreeActivation {
            target_path: target_path.clone(),
            expected_target_profile,
            focus_position,
            status: TicketStatus::Pending,
        });
        self.request = Some(ContextRequest {
            target: RequestTarget::SourceFree,
            lifecycle_revision: self.revocation,
            ..request
        });
        self.try_ready();
        true
    }

    pub(crate) fn focus_out(&mut self, owner: &EngineOwner, position: P) -> Option<TicketId> {
        if self.owner.as_ref() != Some(owner) {
            return None;
        }
        if self
            .ticket
            .as_ref()
            .is_some_and(|ticket| ticket.status == TicketStatus::Consumed)
        {
            self.ticket = None;
            self.request = None;
        }
        if self.ticket.is_none() {
            let source_activation = self.activation.clone()?;
            let expected_target_profile = match &self.profile {
                GlobalProfile::Lay(profile) => profile.clone(),
                GlobalProfile::Foreign(_) => {
                    self.revoke();
                    return None;
                }
            };
            let id = TicketId(self.take_ticket_id());
            self.ticket = Some(HandoffTicket {
                id,
                kind: TicketKind::Reflexive,
                source_owner: owner.clone(),
                source_activation,
                target_path: Some(owner.path.clone()),
                expected_target_profile,
                factory_position: position.clone(),
                revocation_at_open: self.revocation,
                focus_out_position: None,
                disable_position: None,
                target_focus_position: None,
                source_seal: None,
                status: TicketStatus::Pending,
            });
        }
        let ticket = self.ticket.as_mut()?;
        if ticket.source_owner != *owner
            || ticket.focus_out_position.is_some()
            || (ticket.kind == TicketKind::Factory && position <= ticket.factory_position)
        {
            self.revoke();
            return None;
        }
        ticket.focus_out_position = Some(position);
        Some(ticket.id)
    }

    pub(crate) fn disable(&mut self, owner: &EngineOwner, position: P) -> bool {
        if self.owner.as_ref() != Some(owner) {
            return false;
        }
        let Some(ticket) = self.ticket.as_mut() else {
            self.revoke();
            return false;
        };
        if ticket.source_owner != *owner
            || ticket.disable_position.is_some()
            || !ticket
                .focus_out_position
                .as_ref()
                .is_some_and(|focus_out| position > *focus_out)
        {
            self.revoke();
            return false;
        }
        ticket.disable_position = Some(position);
        self.try_ready();
        true
    }

    pub(crate) fn seal_source(
        &mut self,
        owner: &EngineOwner,
        tail_epoch: u64,
        position: P,
    ) -> bool {
        if self.owner.as_ref() != Some(owner) {
            return false;
        }
        if !self.unsettled.is_empty() {
            return false;
        }
        let Some(ticket) = self.ticket.as_mut() else {
            return false;
        };
        if ticket.source_owner != *owner
            || ticket.source_seal.is_some()
            || tail_epoch != self.latest_tail_epoch
            || !ticket
                .focus_out_position
                .as_ref()
                .is_some_and(|focus_out| position <= *focus_out)
        {
            self.revoke();
            return false;
        }
        self.latest_tail_epoch = self.latest_tail_epoch.max(tail_epoch);
        ticket.source_seal = Some(SourceSeal {
            tail_epoch: self.latest_tail_epoch,
            lineage: self.lineage,
            position,
        });
        self.try_ready();
        true
    }

    pub(crate) fn focus_in(
        &mut self,
        target_path: &EnginePath,
        nonce: BarrierNonce,
        origin: ReceiptOrigin,
        position: P,
    ) -> Option<RequestGeneration> {
        if nonce.0 == 0 {
            self.revoke();
            return None;
        }
        let ticket_id = {
            let ticket = self.ticket.as_mut()?;
            if ticket.status != TicketStatus::Pending
                || ticket.target_path.as_ref() != Some(target_path)
                || ticket.target_focus_position.is_some()
            {
                self.revoke();
                return None;
            }
            ticket.target_focus_position = Some(position);
            ticket.id
        };
        if self.request.is_some() {
            self.revoke();
            return None;
        }
        let generation = RequestGeneration(self.take_request_generation());
        self.request = Some(ContextRequest {
            generation,
            nonce,
            target: RequestTarget::Transfer(ticket_id),
            origin,
            lifecycle_revision: self.revocation,
            reply: None,
            marker_position: None,
        });
        Some(generation)
    }

    pub(crate) fn begin_source_free_activation(
        &mut self,
        target_path: EnginePath,
        nonce: BarrierNonce,
        origin: ReceiptOrigin,
        position: P,
    ) -> Option<RequestGeneration> {
        if self.mode != GlobalEngineMode::Verified || nonce.0 == 0 {
            self.revoke();
            return None;
        }
        // Existing consumed receipts provide bounded successor provenance.
        // Require a current owner and activation, and reject only a delayed
        // predecessor path; a real transfer uses its pending ticket instead.
        let installed_owner_path =
            self.owner
                .as_ref()
                .zip(self.activation.as_ref())
                .and_then(|(owner, _)| {
                    let consumed_target = self
                        .ticket
                        .as_ref()
                        .filter(|ticket| ticket.status == TicketStatus::Consumed)
                        .and_then(|ticket| ticket.target_path.as_ref())
                        .or_else(|| {
                            self.source_free
                                .as_ref()
                                .filter(|activation| activation.status == TicketStatus::Consumed)
                                .map(|activation| &activation.target_path)
                        });
                    (consumed_target == Some(&owner.path)).then_some(&owner.path)
                });
        if installed_owner_path.is_some_and(|current| current != &target_path) {
            return None;
        }
        let live_source_free_target = self.source_free.as_ref().and_then(|activation| {
            matches!(
                activation.status,
                TicketStatus::Pending | TicketStatus::Ready
            )
            .then_some(&activation.target_path)
        });
        // A late focus from a superseded path cannot borrow or revoke the
        // current reservation/request. A duplicate for the exact path remains
        // a conflicting lifecycle event and fails closed below.
        if live_source_free_target.is_some_and(|current| current != &target_path) {
            return None;
        }
        if self.request.is_some() || live_source_free_target.is_some() {
            self.revoke();
            return None;
        }
        let reservation = self.source_free_factory.clone();
        let expected_target_profile = if let Some(reservation) = reservation.as_ref() {
            if reservation.target_path.as_ref() != Some(&target_path) {
                return None;
            }
            if reservation.status != TicketStatus::Pending
                || position <= reservation.factory_position
            {
                self.revoke();
                return None;
            }
            reservation.expected_target_profile.clone()
        } else {
            match &self.profile {
                GlobalProfile::Lay(profile) => profile.clone(),
                GlobalProfile::Foreign(_) => {
                    self.revoke();
                    return None;
                }
            }
        };
        self.revoke();
        self.owner = None;
        self.activation = None;
        self.ticket = None;
        self.source_free_factory = None;
        self.unsettled.clear();
        self.source_free = Some(SourceFreeActivation {
            target_path,
            expected_target_profile,
            focus_position: position,
            status: TicketStatus::Pending,
        });
        let generation = RequestGeneration(self.take_request_generation());
        self.request = Some(ContextRequest {
            generation,
            nonce,
            target: RequestTarget::SourceFree,
            origin,
            lifecycle_revision: self.revocation,
            reply: None,
            marker_position: None,
        });
        Some(generation)
    }

    pub(crate) fn context_reply(
        &mut self,
        generation: RequestGeneration,
        nonce: BarrierNonce,
        context: ContextKey,
        position: P,
    ) -> bool {
        let Some(request) = self.request.as_ref() else {
            return false;
        };
        if request.generation != generation {
            return false;
        }
        if request.nonce != nonce
            || request.lifecycle_revision != self.revocation
            || context.connection != self.connection
            || request.reply.is_some()
        {
            self.revoke();
            return false;
        }
        if let RequestTarget::Transfer(ticket_id) = request.target {
            let Some(ticket) = self.ticket.as_ref() else {
                self.revoke();
                return false;
            };
            if ticket.id != ticket_id {
                self.revoke();
                return false;
            }
            if ticket.source_activation.context != context {
                let Some(target_path) = ticket.target_path.clone() else {
                    self.revoke();
                    return false;
                };
                let Some(focus_position) = ticket.target_focus_position.clone() else {
                    self.revoke();
                    return false;
                };
                let expected_target_profile = ticket.expected_target_profile.clone();
                let origin = request.origin;
                self.revoke();
                self.owner = None;
                self.activation = None;
                self.ticket = None;
                self.unsettled.clear();
                self.source_free = Some(SourceFreeActivation {
                    target_path,
                    expected_target_profile,
                    focus_position,
                    status: TicketStatus::Pending,
                });
                self.request = Some(ContextRequest {
                    generation,
                    nonce,
                    target: RequestTarget::SourceFree,
                    origin,
                    lifecycle_revision: self.revocation,
                    reply: Some((context, position)),
                    marker_position: None,
                });
                return true;
            }
        }
        let request = self
            .request
            .as_mut()
            .expect("request presence validated above");
        request.reply = Some((context, position));
        true
    }

    pub(crate) fn enrich_pending_native_activation(
        &mut self,
        target_path: &EnginePath,
        context: ContextKey,
        position: P,
    ) -> bool {
        let Some(request) = self.request.as_ref() else {
            return false;
        };
        if request.origin != ReceiptOrigin::CompatibilityProperty {
            return false;
        }
        let matches_target = match request.target {
            RequestTarget::Transfer(ticket_id) => self.ticket.as_ref().is_some_and(|ticket| {
                ticket.id == ticket_id
                    && matches!(ticket.status, TicketStatus::Pending | TicketStatus::Ready)
                    && ticket.target_path.as_ref() == Some(target_path)
            }),
            RequestTarget::SourceFree => self.source_free.as_ref().is_some_and(|activation| {
                matches!(
                    activation.status,
                    TicketStatus::Pending | TicketStatus::Ready
                ) && &activation.target_path == target_path
            }),
        };
        if !matches_target {
            return false;
        }
        if request.lifecycle_revision != self.revocation
            || context.connection != self.connection
            || request
                .reply
                .as_ref()
                .is_some_and(|(observed, _)| observed != &context)
        {
            self.revoke();
            return false;
        }
        let generation = request.generation;
        let nonce = request.nonce;
        let needs_reply = request.reply.is_none();
        let request = self
            .request
            .as_mut()
            .expect("matching request checked above");
        request.origin = ReceiptOrigin::Native;
        // Keep an already ordered Get reply/marker pair intact. With no reply,
        // use the same context transition, including different-field revocation.
        if needs_reply && !self.context_reply(generation, nonce, context, position) {
            return false;
        }
        self.try_ready();
        true
    }

    pub(crate) fn context_acquisition_failed(&mut self, generation: RequestGeneration) -> bool {
        if self
            .request
            .as_ref()
            .is_some_and(|request| request.generation == generation)
        {
            self.revoke();
            true
        } else {
            false
        }
    }

    /// A target key already dispatched before grant consumption remains literal
    /// and poisons completeness without destroying an otherwise valid identity
    /// claim. The helper does not retain or replay that key.
    pub(crate) fn target_input_before_admission(&mut self, target_path: &EnginePath) -> bool {
        let matches_pending_target = self.ticket.as_ref().is_some_and(|ticket| {
            matches!(ticket.status, TicketStatus::Pending | TicketStatus::Ready)
                && ticket.target_path.as_ref() == Some(target_path)
        });
        if !matches_pending_target {
            self.revoke();
            return false;
        }
        let generation = self.take_lineage_generation();
        self.lineage = WordLineage {
            generation: LineageGeneration(generation),
            completeness: WordCompleteness::UnknownStart,
            observed_suffix_chars: 0,
            observed_boundary_floor: None,
        };
        if let Some(seal) = self
            .ticket
            .as_mut()
            .and_then(|ticket| ticket.source_seal.as_mut())
        {
            seal.lineage = self.lineage;
        }
        true
    }

    /// Called only after exact path/interface/nonce and marker sender validation.
    pub(crate) fn marker(
        &mut self,
        generation: RequestGeneration,
        nonce: BarrierNonce,
        position: P,
    ) -> bool {
        let Some(request) = self.request.as_mut() else {
            return false;
        };
        if request.generation != generation || request.nonce != nonce {
            // A marker for another request cannot settle or revoke this request.
            return false;
        }
        let Some((_, reply_position)) = request.reply.as_ref() else {
            return false;
        };
        if request.lifecycle_revision != self.revocation
            || position <= *reply_position
            || request.marker_position.is_some()
        {
            self.revoke();
            return false;
        }
        request.marker_position = Some(position);
        self.try_ready();
        self.status() == AdmissionStatus::Ready
    }

    pub(crate) fn global_engine_changed(&mut self, profile: GlobalProfile) {
        let foreign = !matches!(&profile, GlobalProfile::Lay(_));
        self.profile = profile;
        if foreign {
            self.revoke();
            return;
        }
        let mismatched_lay = self
            .ticket
            .as_ref()
            .map(|ticket| &ticket.expected_target_profile)
            .or_else(|| {
                self.source_free_factory
                    .as_ref()
                    .map(|reservation| &reservation.expected_target_profile)
            })
            .or_else(|| {
                self.source_free
                    .as_ref()
                    .map(|activation| &activation.expected_target_profile)
            })
            .is_some_and(|expected| {
                !matches!(&self.profile, GlobalProfile::Lay(profile) if profile == expected)
            });
        if mismatched_lay {
            self.revoke();
        } else {
            self.try_ready();
        }
    }

    pub(crate) fn owner_loss_or_disconnect(&mut self) {
        self.mode = GlobalEngineMode::UnsupportedOrFalse;
        self.owner = None;
        self.activation = None;
        self.revoke();
    }

    pub(crate) fn malformed_or_unobserved_lifecycle(&mut self) {
        self.revoke();
    }

    pub(crate) fn consume(&mut self) -> Option<TransferGrant> {
        let (ticket_id, kind, source_owner, source_activation, target_path, seal, origin) = {
            let ticket = self.ticket.as_mut()?;
            if ticket.status != TicketStatus::Ready {
                return None;
            }
            ticket.status = TicketStatus::Consumed;
            (
                ticket.id,
                ticket.kind,
                ticket.source_owner.clone(),
                ticket.source_activation.clone(),
                ticket.target_path.clone()?,
                ticket.source_seal.clone()?,
                self.request.as_ref()?.origin,
            )
        };
        let context = self.request.as_ref()?.reply.as_ref()?.0.clone();
        if context != source_activation.context {
            self.revoke();
            return None;
        }
        let exact_manual_snapshot = self.exact_manual_snapshot.take().filter(|receipt| {
            receipt.source_token.connection == self.connection
                && receipt.source_token.revocation == self.revocation
                && receipt.source_token.owner == source_owner
                && receipt.source_token.activation == source_activation
                && receipt.source_token.lineage == seal.lineage
                && receipt.tail_epoch == seal.tail_epoch
                && receipt.expires_at > Instant::now()
        });
        let target_owner = EngineOwner {
            path: target_path,
            generation: OwnerGeneration(self.take_owner_generation()),
        };
        let target_activation = FocusActivation {
            context,
            generation: ActivationGeneration(self.take_activation_generation()),
        };
        // Ownership, activation and frame epochs rotate at handoff, but the
        // admitted word remains the exact lineage sealed by the source.
        self.lineage = seal.lineage;
        let frame_generation = FrameGeneration(self.take_frame_generation());
        self.latest_tail_epoch = seal.tail_epoch;
        self.owner = Some(target_owner.clone());
        self.activation = Some(target_activation.clone());
        self.request = None;
        Some(TransferGrant {
            revocation: self.revocation,
            ticket: ticket_id,
            kind,
            source_owner,
            target_owner,
            target_activation,
            lineage: self.lineage,
            source_tail_epoch: seal.tail_epoch,
            frame_generation,
            invalidate_prior_authority: true,
            receipt_origin: origin,
            exact_manual_snapshot: exact_manual_snapshot.map(Box::new),
        })
    }

    pub(crate) fn is_current_owner(&self, owner: &EngineOwner) -> bool {
        self.owner.as_ref() == Some(owner)
    }

    pub(crate) fn consume_source_free_activation(&mut self) -> Option<ActivationGrant> {
        let (target_path, origin, context) = {
            let activation = self.source_free.as_mut()?;
            if activation.status != TicketStatus::Ready {
                return None;
            }
            let request = self.request.as_ref()?;
            let context = request.reply.as_ref()?.0.clone();
            activation.status = TicketStatus::Consumed;
            (activation.target_path.clone(), request.origin, context)
        };
        let target_owner = EngineOwner {
            path: target_path,
            generation: OwnerGeneration(self.take_owner_generation()),
        };
        let target_activation = FocusActivation {
            context,
            generation: ActivationGeneration(self.take_activation_generation()),
        };
        let lineage_generation = self.take_lineage_generation();
        self.lineage = WordLineage {
            generation: LineageGeneration(lineage_generation),
            completeness: WordCompleteness::UnknownStart,
            observed_suffix_chars: 0,
            observed_boundary_floor: None,
        };
        self.latest_tail_epoch = 0;
        self.settled_content_type = None;
        self.owner = Some(target_owner.clone());
        self.activation = Some(target_activation.clone());
        self.request = None;
        Some(ActivationGrant {
            revocation: self.revocation,
            target_owner,
            target_activation,
            lineage: self.lineage,
            tail_epoch: self.latest_tail_epoch,
            frame_generation: FrameGeneration(self.take_frame_generation()),
            receipt_origin: origin,
        })
    }

    pub(crate) fn bind_source_free_tail_epoch(
        &mut self,
        grant: &ActivationGrant,
        tail_epoch: u64,
    ) -> bool {
        let source_free_is_consumed = self
            .source_free
            .as_ref()
            .is_some_and(|activation| activation.status == TicketStatus::Consumed);
        if !source_free_is_consumed
            || self.revocation != grant.revocation
            || self.owner.as_ref() != Some(&grant.target_owner)
            || self.activation.as_ref() != Some(&grant.target_activation)
            || self.lineage != grant.lineage
            || self.latest_tail_epoch != grant.tail_epoch
            || tail_epoch <= self.latest_tail_epoch
        {
            return false;
        }
        self.latest_tail_epoch = tail_epoch;
        // Retain the consumed receipt as bounded path provenance. The grant is
        // still one-use because the bound epoch no longer matches it.
        true
    }

    pub(crate) fn bind_reset_tail_epoch(
        &mut self,
        grant: &ActivationGrant,
        tail_epoch: u64,
    ) -> bool {
        if !self.lifecycle_is_settled()
            || self.mode != GlobalEngineMode::Verified
            || !matches!(self.profile, GlobalProfile::Lay(_))
            || self.revocation != grant.revocation
            || self.owner.as_ref() != Some(&grant.target_owner)
            || self.activation.as_ref() != Some(&grant.target_activation)
            || self.lineage != grant.lineage
            || self.lineage.completeness != WordCompleteness::UnknownStart
            || self.latest_tail_epoch != grant.tail_epoch
            || tail_epoch <= self.latest_tail_epoch
        {
            return false;
        }
        self.latest_tail_epoch = tail_epoch;
        true
    }

    /// Text reads outside a key callback must not overtake received work.
    /// The callback's own identity check deliberately has no quiescence test.
    fn lifecycle_is_settled(&self) -> bool {
        self.request.is_none()
            && self
                .ticket
                .as_ref()
                .is_none_or(|ticket| ticket.status == TicketStatus::Consumed)
            && self
                .source_free
                .as_ref()
                .is_none_or(|activation| activation.status == TicketStatus::Consumed)
            && self
                .source_free_factory
                .as_ref()
                .is_none_or(|reservation| reservation.status == TicketStatus::Consumed)
    }

    pub(crate) fn bridge_admission_token(&self) -> Option<AdmissionToken> {
        (!self.has_pending_word_input() && self.lifecycle_is_settled())
            .then(|| self.admission_token())
            .flatten()
    }

    pub(crate) fn revalidate_bridge(&self, token: &AdmissionToken) -> bool {
        !self.has_pending_word_input() && self.lifecycle_is_settled() && self.revalidate(token)
    }

    fn has_pending_word_input(&self) -> bool {
        // Legacy Shift still occupies the bounded queue and needs its actual
        // callback settlement. Its protected handler cannot change the word,
        // so it cannot conflict with the daemon-owned manual projection.
        self.unsettled
            .iter()
            .any(|key| key.word_effect == KeyWordEffect::PossibleMutation)
    }

    pub(crate) fn admission_token(&self) -> Option<AdmissionToken> {
        if self.mode != GlobalEngineMode::Verified {
            return None;
        }
        Some(AdmissionToken {
            connection: self.connection,
            revocation: self.revocation,
            owner: self.owner.clone()?,
            activation: self.activation.clone()?,
            lineage: self.lineage,
        })
    }

    pub(crate) fn revalidate(&self, token: &AdmissionToken) -> bool {
        self.mode == GlobalEngineMode::Verified
            && token.connection == self.connection
            && token.revocation == self.revocation
            && self.owner.as_ref() == Some(&token.owner)
            && self.activation.as_ref() == Some(&token.activation)
            && token.lineage == self.lineage
    }

    pub(crate) fn layout_intent_token(&self) -> Option<LayoutIntentToken> {
        if self.mode != GlobalEngineMode::Verified {
            return None;
        }
        Some(LayoutIntentToken {
            connection: self.connection,
            revocation: self.revocation,
            owner: self.owner.clone()?,
            activation: self.activation.clone()?,
        })
    }

    pub(crate) fn layout_intent_token_for(&self, owner: &EngineOwner) -> Option<LayoutIntentToken> {
        (self.owner.as_ref() == Some(owner))
            .then(|| self.layout_intent_token())
            .flatten()
    }

    pub(crate) fn revalidate_layout_intent(&self, token: &LayoutIntentToken) -> bool {
        self.mode == GlobalEngineMode::Verified
            && token.connection == self.connection
            && token.revocation == self.revocation
            && self.owner.as_ref() == Some(&token.owner)
            && self.activation.as_ref() == Some(&token.activation)
    }

    pub(crate) fn enrich_native_activation(
        &mut self,
        owner: &EngineOwner,
        target_path: &EnginePath,
        context: &ContextKey,
    ) -> Option<bool> {
        // A retained owner is not proof that its previous activation is still
        // active: FocusOut/Disable already opened the next lifecycle ticket.
        if !self.lifecycle_is_settled() {
            return None;
        }
        if self.mode != GlobalEngineMode::Verified
            || self.owner.as_ref() != Some(owner)
            || &owner.path != target_path
            || context.connection != self.connection
            || self
                .activation
                .as_ref()
                .is_none_or(|activation| &activation.context != context)
        {
            self.revoke();
            return Some(false);
        }
        Some(true)
    }

    /// Fail closed after a conflicting layout completion while retaining the
    /// physical layout chosen by the single external layout owner.
    pub(crate) fn revoke_layout_intent(&mut self, token: &LayoutIntentToken) -> bool {
        if !self.revalidate_layout_intent(token) {
            return false;
        }
        self.revoke();
        true
    }

    fn try_ready(&mut self) {
        let Some(request) = self.request.as_ref() else {
            return;
        };
        let Some((context, reply_position)) = request.reply.as_ref() else {
            return;
        };
        let Some(marker_position) = request.marker_position.as_ref() else {
            return;
        };
        if request.lifecycle_revision != self.revocation || marker_position <= reply_position {
            return;
        }
        match request.target {
            RequestTarget::Transfer(ticket_id) => {
                let Some(ticket) = self.ticket.as_mut() else {
                    return;
                };
                if ticket.status != TicketStatus::Pending
                    || ticket.id != ticket_id
                    || ticket.revocation_at_open != self.revocation
                    || ticket.focus_out_position.is_none()
                    || (ticket.kind == TicketKind::Factory && ticket.disable_position.is_none())
                    || ticket.source_seal.is_none()
                    || ticket.target_focus_position.is_none()
                    || !self.unsettled.is_empty()
                {
                    return;
                }
                let source_seal_position = &ticket
                    .source_seal
                    .as_ref()
                    .expect("source seal presence checked above")
                    .position;
                let target_focus_position = ticket
                    .target_focus_position
                    .as_ref()
                    .expect("target focus presence checked above");
                let reply_follows_focus = match request.origin {
                    ReceiptOrigin::Native => reply_position >= target_focus_position,
                    ReceiptOrigin::CompatibilityProperty => reply_position > target_focus_position,
                };
                if context == &ticket.source_activation.context
                    && reply_follows_focus
                    && marker_position > source_seal_position
                    && matches!(
                        &self.profile,
                        GlobalProfile::Lay(profile) if profile == &ticket.expected_target_profile
                    )
                {
                    ticket.status = TicketStatus::Ready;
                }
            }
            RequestTarget::SourceFree => {
                let Some(activation) = self.source_free.as_mut() else {
                    return;
                };
                let reply_follows_focus = match request.origin {
                    ReceiptOrigin::Native => reply_position >= &activation.focus_position,
                    ReceiptOrigin::CompatibilityProperty => {
                        reply_position > &activation.focus_position
                    }
                };
                if activation.status == TicketStatus::Pending
                    && reply_follows_focus
                    && self.unsettled.is_empty()
                    && matches!(
                        &self.profile,
                        GlobalProfile::Lay(profile) if profile == &activation.expected_target_profile
                    )
                {
                    activation.status = TicketStatus::Ready;
                }
            }
        }
    }

    fn compatibility_readiness_reason(&self, generation: RequestGeneration) -> &'static str {
        let Some(request) = self
            .request
            .as_ref()
            .filter(|request| request.generation == generation)
        else {
            return "request_not_current";
        };
        let Some((context, reply_position)) = request.reply.as_ref() else {
            return "reply_missing";
        };
        let Some(marker_position) = request.marker_position.as_ref() else {
            return "marker_missing";
        };
        if request.lifecycle_revision != self.revocation {
            return "lifecycle_revision_mismatch";
        }
        if marker_position <= reply_position {
            return "marker_not_after_reply";
        }
        match request.target {
            RequestTarget::Transfer(ticket_id) => {
                let Some(ticket) = self.ticket.as_ref() else {
                    return "transfer_ticket_missing";
                };
                if ticket.status == TicketStatus::Ready {
                    return "ready";
                }
                if ticket.status != TicketStatus::Pending {
                    return "transfer_ticket_not_pending";
                }
                if ticket.id != ticket_id {
                    return "transfer_ticket_identity_mismatch";
                }
                if ticket.revocation_at_open != self.revocation {
                    return "transfer_ticket_revocation_mismatch";
                }
                if ticket.focus_out_position.is_none() {
                    return "source_focus_out_missing";
                }
                if ticket.kind == TicketKind::Factory && ticket.disable_position.is_none() {
                    return "source_disable_missing";
                }
                if ticket.source_seal.is_none() {
                    return "source_seal_missing";
                }
                if ticket.target_focus_position.is_none() {
                    return "target_focus_missing";
                }
                if !self.unsettled.is_empty() {
                    return "unsettled_callbacks";
                }
                let source_seal_position = &ticket
                    .source_seal
                    .as_ref()
                    .expect("source seal presence checked above")
                    .position;
                let target_focus_position = ticket
                    .target_focus_position
                    .as_ref()
                    .expect("target focus presence checked above");
                let reply_follows_focus = match request.origin {
                    ReceiptOrigin::Native => reply_position >= target_focus_position,
                    ReceiptOrigin::CompatibilityProperty => reply_position > target_focus_position,
                };
                if context != &ticket.source_activation.context {
                    return "source_context_mismatch";
                }
                if !reply_follows_focus {
                    return "reply_not_after_target_focus";
                }
                if marker_position <= source_seal_position {
                    return "marker_not_after_source_seal";
                }
                if !matches!(
                    &self.profile,
                    GlobalProfile::Lay(profile) if profile == &ticket.expected_target_profile
                ) {
                    return "target_profile_mismatch";
                }
                "ready"
            }
            RequestTarget::SourceFree => {
                let Some(activation) = self.source_free.as_ref() else {
                    return "source_free_activation_missing";
                };
                if activation.status == TicketStatus::Ready {
                    return "ready";
                }
                if activation.status != TicketStatus::Pending {
                    return "source_free_activation_not_pending";
                }
                let reply_follows_focus = match request.origin {
                    ReceiptOrigin::Native => reply_position >= &activation.focus_position,
                    ReceiptOrigin::CompatibilityProperty => {
                        reply_position > &activation.focus_position
                    }
                };
                if !reply_follows_focus {
                    return "reply_not_after_target_focus";
                }
                if !self.unsettled.is_empty() {
                    return "unsettled_callbacks";
                }
                if !matches!(
                    &self.profile,
                    GlobalProfile::Lay(profile) if profile == &activation.expected_target_profile
                ) {
                    return "target_profile_mismatch";
                }
                "ready"
            }
        }
    }

    fn bridge_readiness_reason(&self) -> &'static str {
        if self.has_pending_word_input() {
            return "unsettled_callbacks";
        }
        if !self.lifecycle_is_settled() {
            return "lifecycle_pending";
        }
        if self.mode != GlobalEngineMode::Verified {
            return "global_mode_unverified";
        }
        if self.owner.is_none() {
            return "owner_missing";
        }
        if self.activation.is_none() {
            return "activation_missing";
        }
        "ready"
    }

    fn revoke(&mut self) {
        self.settled_content_type = None;
        self.exact_manual_snapshot = None;
        self.revocation = next_generation(self.revocation);
        // A consumed receipt identifies a live admitted successor only until
        // revocation. It must not block a later empty recovery after word loss.
        if self
            .ticket
            .as_ref()
            .is_some_and(|ticket| ticket.status == TicketStatus::Consumed)
        {
            self.ticket = None;
        }
        if self
            .source_free
            .as_ref()
            .is_some_and(|activation| activation.status == TicketStatus::Consumed)
        {
            self.source_free = None;
        }
        if let Some(ticket) = self.ticket.as_mut() {
            if ticket.status != TicketStatus::Consumed {
                ticket.status = TicketStatus::Revoked;
            }
        }
        if let Some(activation) = self.source_free.as_mut() {
            if activation.status != TicketStatus::Consumed {
                activation.status = TicketStatus::Revoked;
            }
        }
        if let Some(reservation) = self.source_free_factory.as_mut() {
            if reservation.status != TicketStatus::Consumed {
                reservation.status = TicketStatus::Revoked;
            }
        }
        self.request = None;
        self.unsettled.clear();
        let lineage_generation = self.take_lineage_generation();
        self.lineage = WordLineage {
            generation: LineageGeneration(lineage_generation),
            completeness: WordCompleteness::UnknownStart,
            observed_suffix_chars: 0,
            observed_boundary_floor: None,
        };
    }

    fn take_owner_generation(&mut self) -> u64 {
        take_counter(&mut self.next_owner)
    }

    fn take_activation_generation(&mut self) -> u64 {
        take_counter(&mut self.next_activation)
    }

    fn take_lineage_generation(&mut self) -> u64 {
        take_counter(&mut self.next_lineage)
    }

    fn take_request_generation(&mut self) -> u64 {
        take_counter(&mut self.next_request)
    }

    fn take_ticket_id(&mut self) -> u64 {
        take_counter(&mut self.next_ticket)
    }

    fn take_frame_generation(&mut self) -> u64 {
        take_counter(&mut self.next_frame)
    }
}

/// Remaining allowance for the existing Space wait.  The callback rendezvous
/// is charged to, never added on top of, the original 3,500 us budget.
#[cfg(test)]
pub(crate) fn remaining_space_wait_budget_us(elapsed_since_callback_entry_us: u64) -> u64 {
    SPACE_FULL_WAIT_BUDGET_US.saturating_sub(elapsed_since_callback_entry_us)
}

pub(crate) fn remaining_space_wait_budget(elapsed_since_callback_entry: Duration) -> Duration {
    Duration::from_micros(SPACE_FULL_WAIT_BUDGET_US).saturating_sub(elapsed_since_callback_entry)
}

fn bounded_identifier(value: impl Into<String>) -> Option<String> {
    let value = value.into();
    (!value.is_empty() && value.len() <= MAX_IDENTIFIER_BYTES).then_some(value)
}

fn next_generation(current: u64) -> u64 {
    let next = current.wrapping_add(1);
    if next == 0 {
        1
    } else {
        next
    }
}

fn take_counter(counter: &mut u64) -> u64 {
    let current = *counter;
    *counter = next_generation(*counter);
    if current == 0 {
        1
    } else {
        current
    }
}

#[cfg(test)]
#[path = "context_admission/tests.rs"]
mod tests;
