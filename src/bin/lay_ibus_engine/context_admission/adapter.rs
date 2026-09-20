//! Same-IBus-connection metadata adapter for TD-121.
//!
//! This module owns no text and performs no engine, Shared, layout or model
//! work. The observer awaits only ordered zbus input, then publishes bounded
//! metadata after releasing its short locks.

use std::fmt;
use std::future::{poll_fn, Future};
use std::pin::Pin;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::task::Poll;
use std::time::{Duration, Instant};

use event_listener::Event;
use ordered_stream::OrderedStream;
use zbus::message::{Sequence, Type};
use zbus::zvariant::{OwnedObjectPath, OwnedValue, Value};
use zbus::{Connection, MatchRule, Message, MessageStream};

use super::*;
use crate::trace;

const DBUS_NAME: &str = "org.freedesktop.DBus";
const DBUS_PATH: &str = "/org/freedesktop/DBus";
const DBUS_INTERFACE: &str = "org.freedesktop.DBus";
const PROPERTIES_INTERFACE: &str = "org.freedesktop.DBus.Properties";
const IBUS_NAME: &str = "org.freedesktop.IBus";
const IBUS_PATH: &str = "/org/freedesktop/IBus";
const IBUS_INTERFACE: &str = "org.freedesktop.IBus";
const FACTORY_INTERFACE: &str = "org.freedesktop.IBus.Factory";
const ENGINE_INTERFACE: &str = "org.freedesktop.IBus.Engine";
const ENGINE_PATH_PREFIX: &str = "/io/github/radislabus_star/LayIme/engine/";
const MARKER_PATH: &str = "/io/github/radislabus_star/LayIme/ContextAdmission";
const MARKER_INTERFACE: &str = "io.github.radislabus_star.LayIme.ContextAdmission";
const MARKER_MEMBER: &str = "Barrier";
const LIFECYCLE_QUEUE: usize = 8;
const KEY_QUEUE: usize = 64;
const MAX_LAY_PROFILES: usize = 8;
pub(crate) const ACQUISITION_BUDGET: Duration = Duration::from_millis(5);
// Focus acquisition is asynchronous and includes a full Get/marker round trip.
// Keep its end-to-end deadline separate from text-mutation callback fencing.
const ACTIVATION_BUDGET: Duration = Duration::from_millis(50);

const ENGINE_CALLBACK_MEMBERS: &[&str] = &[
    "FocusIn",
    "FocusInId",
    "FocusOut",
    "FocusOutId",
    "Enable",
    "Disable",
    "Reset",
    "ProcessKeyEvent",
    "ProcessKeyEventAtomicV1",
];

fn word_completeness_trace(completeness: WordCompleteness) -> &'static str {
    match completeness {
        WordCompleteness::KnownStart => "known_start",
        WordCompleteness::UnknownStart => "unknown_start",
    }
}

fn rendezvous_failure_trace(failure: RendezvousFailure) -> &'static str {
    match failure {
        RendezvousFailure::Revoked => "refused_revoked",
        RendezvousFailure::Evicted => "refused_evicted",
        RendezvousFailure::ObserverTerminated => "refused_observer_terminated",
        RendezvousFailure::Poisoned => "refused_poisoned",
        RendezvousFailure::Timeout => "refused_timeout",
    }
}

fn context_admission_trace_disposition(
    disposition: &IngressDisposition,
) -> (&'static str, Option<u64>) {
    match disposition {
        IngressDisposition::Passive => ("passive", None),
        IngressDisposition::Key { owner, .. } => ("key", Some(owner.generation.0)),
        IngressDisposition::Factory { .. } => ("factory", None),
        IngressDisposition::FocusOut { owner, .. } => ("focus_out", Some(owner.generation.0)),
        IngressDisposition::Disable { owner } => ("disable", Some(owner.generation.0)),
        IngressDisposition::Revocation { owner } => ("revocation", Some(owner.generation.0)),
        IngressDisposition::UnchangedContentType { token, .. } => {
            ("unchanged_content_type", Some(token.owner.generation.0))
        }
        IngressDisposition::StaleContext => ("stale_context", None),
    }
}

type JoinedMessages =
    Pin<Box<dyn OrderedStream<Ordering = Sequence, Data = zbus::Result<Message>> + Send + 'static>>;

#[derive(Debug)]
pub(crate) enum AdapterError {
    Bus(zbus::Error),
    MissingUniqueName,
    InvalidConfiguration(&'static str),
    InvalidBootstrap(&'static str),
    Busy,
    Denied,
    Timeout,
    Cancelled,
}

impl fmt::Display for AdapterError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Bus(error) => write!(formatter, "zbus: {error}"),
            Self::MissingUniqueName => formatter.write_str("IBus connection has no unique name"),
            Self::InvalidConfiguration(reason) => {
                write!(formatter, "invalid adapter configuration: {reason}")
            }
            Self::InvalidBootstrap(reason) => write!(formatter, "invalid bootstrap: {reason}"),
            Self::Busy => formatter.write_str("another context fence is pending"),
            Self::Denied => formatter.write_str("context admission denied"),
            Self::Timeout => formatter.write_str("context admission deadline expired"),
            Self::Cancelled => formatter.write_str("metadata observer cancelled"),
        }
    }
}

impl std::error::Error for AdapterError {}

impl From<zbus::Error> for AdapterError {
    fn from(error: zbus::Error) -> Self {
        Self::Bus(error)
    }
}

#[derive(Debug, Clone)]
pub(crate) struct AdapterConfig {
    connection: ConnectionGeneration,
    lay_profiles: Vec<EngineProfile>,
    acquisition_budget: Duration,
    activation_budget: Duration,
}

impl AdapterConfig {
    pub(crate) fn new(
        connection: ConnectionGeneration,
        lay_profiles: Vec<EngineProfile>,
    ) -> Result<Self, AdapterError> {
        if lay_profiles.is_empty()
            || lay_profiles.len() > MAX_LAY_PROFILES
            || lay_profiles
                .iter()
                .enumerate()
                .any(|(index, profile)| lay_profiles[..index].contains(profile))
        {
            return Err(AdapterError::InvalidConfiguration(
                "Lay profiles must be unique and contain 1..=8 entries",
            ));
        }
        Ok(Self {
            connection,
            lay_profiles,
            acquisition_budget: ACQUISITION_BUDGET,
            activation_budget: ACTIVATION_BUDGET,
        })
    }

    #[cfg(test)]
    pub(super) fn with_acquisition_budget(mut self, budget: Duration) -> Self {
        self.acquisition_budget = budget;
        self.activation_budget = budget;
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct BootstrapIdentity {
    pub(crate) ibus_owner: String,
    pub(crate) own_sender: String,
    pub(crate) engine_callback_sender: String,
    pub(crate) mode: GlobalEngineMode,
    pub(crate) profile: GlobalProfile,
}

pub(crate) struct PendingContextAdapter {
    connection: Connection,
    config: AdapterConfig,
    streams: JoinedMessages,
}

impl PendingContextAdapter {
    /// Installs every narrow stream before any bootstrap method/property call.
    pub(crate) async fn subscribe(
        connection: Connection,
        config: AdapterConfig,
    ) -> Result<Self, AdapterError> {
        let mut streams = Vec::new();
        streams.push(
            MessageStream::for_match_rule(
                signal_rule(IBUS_INTERFACE, "GlobalEngineChanged")?,
                &connection,
                Some(LIFECYCLE_QUEUE),
            )
            .await?,
        );
        streams.push(
            MessageStream::for_match_rule(
                signal_rule(MARKER_INTERFACE, MARKER_MEMBER)?,
                &connection,
                Some(LIFECYCLE_QUEUE),
            )
            .await?,
        );
        streams.push(
            MessageStream::for_match_rule(owner_loss_rule()?, &connection, Some(LIFECYCLE_QUEUE))
                .await?,
        );
        streams.push(
            MessageStream::for_match_rule(
                method_rule(FACTORY_INTERFACE, "CreateEngine")?,
                &connection,
                Some(LIFECYCLE_QUEUE),
            )
            .await?,
        );
        for member in ENGINE_CALLBACK_MEMBERS {
            let capacity = if matches!(*member, "ProcessKeyEvent" | "ProcessKeyEventAtomicV1") {
                KEY_QUEUE
            } else {
                LIFECYCLE_QUEUE
            };
            streams.push(
                MessageStream::for_match_rule(
                    method_rule(ENGINE_INTERFACE, member)?,
                    &connection,
                    Some(capacity),
                )
                .await?,
            );
        }
        streams.push(
            MessageStream::for_match_rule(
                method_rule(PROPERTIES_INTERFACE, "Set")?,
                &connection,
                Some(LIFECYCLE_QUEUE),
            )
            .await?,
        );
        let streams = join_all(streams).ok_or(AdapterError::InvalidConfiguration(
            "observer requires at least one stream",
        ))?;
        Ok(Self {
            connection,
            config,
            streams,
        })
    }

    pub(crate) async fn bootstrap(
        self,
    ) -> Result<
        (
            ContextAdmissionAdapter,
            ContextAdmissionObserver,
            BootstrapIdentity,
        ),
        AdapterError,
    > {
        let own_sender = self
            .connection
            .unique_name()
            .ok_or(AdapterError::MissingUniqueName)?
            .as_str()
            .to_owned();
        let ibus_owner = get_name_owner(&self.connection, IBUS_NAME).await?;
        let engine_callback_sender = get_name_owner(&self.connection, DBUS_NAME).await?;
        let mode_reply = self
            .connection
            .call_method(
                Some(IBUS_NAME),
                IBUS_PATH,
                Some(IBUS_INTERFACE),
                "GetUseGlobalEngine",
                &(),
            )
            .await?;
        let global_mode = mode_reply
            .body()
            .deserialize::<bool>()
            .map_err(|_| AdapterError::InvalidBootstrap("GetUseGlobalEngine was not bool"))?;
        let profile_reply = self
            .connection
            .call_method(
                Some(IBUS_NAME),
                IBUS_PATH,
                Some(PROPERTIES_INTERFACE),
                "Get",
                &(IBUS_INTERFACE, "GlobalEngine"),
            )
            .await?;
        let profile_value = profile_reply
            .body()
            .deserialize::<OwnedValue>()
            .map_err(|_| AdapterError::InvalidBootstrap("GlobalEngine was not a variant"))?;
        let profile_name = engine_name_from_global_value(&profile_value).ok_or(
            AdapterError::InvalidBootstrap("GlobalEngine descriptor had no name"),
        )?;
        let profile = classify_profile(&self.config.lay_profiles, profile_name)?;
        let mode = if global_mode {
            GlobalEngineMode::Verified
        } else {
            GlobalEngineMode::UnsupportedOrFalse
        };
        let identity = BootstrapIdentity {
            ibus_owner: ibus_owner.clone(),
            own_sender: own_sender.clone(),
            engine_callback_sender: engine_callback_sender.clone(),
            mode,
            profile: profile.clone(),
        };
        let bindings = SenderBindings::new(ibus_owner, own_sender, engine_callback_sender).ok_or(
            AdapterError::InvalidBootstrap("sender roles were not distinct"),
        )?;
        let reducer = ContextAdmissionReducer::new(self.config.connection, mode, profile);
        let shared = Arc::new(AdapterState {
            connection: self.connection,
            connection_generation: self.config.connection,
            lay_profiles: self.config.lay_profiles,
            acquisition_budget: self.config.acquisition_budget,
            activation_budget: self.config.activation_budget,
            bindings,
            reducer: Mutex::new(reducer),
            stamps: CallbackStampStore::new(self.config.connection, OwnerGeneration(0)),
            pending: Mutex::new(None),
            ready_activation: Mutex::new(None),
            changed: Event::new(),
            cancellation: Event::new(),
            next_nonce: AtomicU64::new(1),
            cancelled: AtomicBool::new(false),
        });
        let adapter = ContextAdmissionAdapter {
            shared: Arc::clone(&shared),
        };
        let observer = ContextAdmissionObserver {
            shared,
            streams: Some(self.streams),
        };
        Ok((adapter, observer, identity))
    }
}

#[derive(Clone)]
pub(crate) struct ContextAdmissionAdapter {
    shared: Arc<AdapterState>,
}

struct AdapterState {
    connection: Connection,
    connection_generation: ConnectionGeneration,
    lay_profiles: Vec<EngineProfile>,
    acquisition_budget: Duration,
    activation_budget: Duration,
    bindings: SenderBindings,
    reducer: Mutex<ContextAdmissionReducer<Sequence>>,
    stamps: CallbackStampStore<Sequence>,
    pending: Mutex<Option<PendingFenceState>>,
    ready_activation: Mutex<Option<ReadyActivationState>>,
    changed: Event,
    cancellation: Event,
    next_nonce: AtomicU64,
    cancelled: AtomicBool,
}

#[derive(Debug, Clone)]
enum FenceKind {
    Acquisition {
        request: RequestGeneration,
        target_path: EnginePath,
    },
    Bridge {
        ping_position: Sequence,
    },
}

#[derive(Debug, Clone)]
struct PendingFenceState {
    nonce: BarrierNonce,
    deadline: Instant,
    kind: FenceKind,
    ready_token: Option<AdmissionToken>,
    marker_observed: bool,
    ready: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct PendingFence {
    nonce: BarrierNonce,
    deadline: Instant,
}

#[derive(Debug, Clone)]
struct ReadyActivationState {
    fence: PendingFence,
    // Only the test-side snapshot/consume witness compares request identities.
    #[cfg(test)]
    request: RequestGeneration,
    target_path: EnginePath,
    owner: EngineOwner,
    outcome: ActivationOutcome,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum ActivationOutcome {
    Transfer(TransferGrant),
    SourceFree(ActivationGrant),
    /// An uninstalled, already-fenced context whose word was explicitly reset.
    /// This is not a new focus receipt and carries no transferable source tail.
    ResetUnknown(ActivationGrant),
}

impl ActivationOutcome {
    fn token(&self) -> AdmissionToken {
        let (owner, activation, lineage, revocation) = match self {
            Self::Transfer(grant) => (
                &grant.target_owner,
                &grant.target_activation,
                grant.lineage,
                grant.revocation,
            ),
            Self::SourceFree(grant) | Self::ResetUnknown(grant) => (
                &grant.target_owner,
                &grant.target_activation,
                grant.lineage,
                grant.revocation,
            ),
        };
        AdmissionToken {
            connection: activation.context.connection,
            revocation,
            owner: owner.clone(),
            activation: activation.clone(),
            lineage,
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct ObservedCallback {
    pub(crate) header: HeaderKey,
    pub(crate) position: Sequence,
    disposition: IngressDisposition,
    guard: RendezvousGuard,
}

#[derive(Debug, Clone)]
pub(crate) struct KeyCallback {
    observed: ObservedCallback,
    owner: EngineOwner,
}

#[derive(Debug, Clone)]
pub(crate) struct FactoryCallback {
    pub(crate) ticket: TicketId,
    #[cfg(test)]
    pub(crate) position: Sequence,
}

fn record_admission_diagnostic(
    flow: &str,
    phase: &str,
    reason: &str,
    (request_generation, nonce): (Option<RequestGeneration>, BarrierNonce),
    snapshot: AdmissionDiagnosticSnapshot,
    fence_present: bool,
    marker_observed: bool,
) {
    trace::record_admission_diagnostic(trace::AdmissionDiagnosticTrace {
        flow,
        phase,
        reason,
        request_generation: request_generation.map(|generation| generation.0),
        nonce: nonce.0,
        reducer_status: snapshot.reducer_status,
        post_decision_unsettled_count: snapshot.unsettled_count,
        owner_generation: snapshot.owner_generation,
        activation_generation: snapshot.activation_generation,
        fence_present,
        marker_observed,
    });
}

impl ContextAdmissionAdapter {
    #[cfg(test)]
    pub(crate) fn test_established(
        path: &str,
        context_path: &str,
        completeness: WordCompleteness,
        tail_epoch: u64,
    ) -> (Self, ActivationGrant, Connection) {
        let (server_socket, peer_socket) =
            std::os::unix::net::UnixStream::pair().expect("context test p2p sockets");
        let guid = zbus::Guid::generate();
        let server = std::thread::spawn(move || {
            zbus::block_on(
                zbus::connection::Builder::unix_stream(server_socket)
                    .server(guid)
                    .expect("context test server identity")
                    .p2p()
                    .build(),
            )
            .expect("context test server connection")
        });
        let peer = zbus::block_on(
            zbus::connection::Builder::unix_stream(peer_socket)
                .p2p()
                .build(),
        )
        .expect("context test peer connection");
        let connection = server.join().expect("context test server handshake");

        let connection_generation = ConnectionGeneration(991);
        let profile = EngineProfile::new("lay-ime-us").expect("context test profile");
        let mut reducer = ContextAdmissionReducer::new(
            connection_generation,
            GlobalEngineMode::Verified,
            GlobalProfile::Lay(profile.clone()),
        );
        let owner = reducer
            .establish_source(
                EnginePath::new(path).expect("context test engine path"),
                ContextKey::new(connection_generation, context_path)
                    .expect("context test input context"),
                completeness,
                tail_epoch,
            )
            .expect("context test established owner");
        let target_activation = reducer
            .activation()
            .expect("context test established activation")
            .clone();
        let lineage = reducer.lineage();
        let revocation = reducer.revocation_generation();
        let bindings =
            SenderBindings::new(":1.0", ":1.1", ":1.2").expect("context test sender roles");
        let shared = Arc::new(AdapterState {
            connection,
            connection_generation,
            lay_profiles: vec![profile],
            acquisition_budget: ACQUISITION_BUDGET,
            activation_budget: ACTIVATION_BUDGET,
            bindings,
            reducer: Mutex::new(reducer),
            stamps: CallbackStampStore::new(connection_generation, owner.generation),
            pending: Mutex::new(None),
            ready_activation: Mutex::new(None),
            changed: Event::new(),
            cancellation: Event::new(),
            next_nonce: AtomicU64::new(1),
            cancelled: AtomicBool::new(false),
        });
        (
            Self { shared },
            ActivationGrant {
                revocation,
                target_owner: owner,
                target_activation,
                lineage,
                tail_epoch,
                frame_generation: FrameGeneration(1),
                receipt_origin: ReceiptOrigin::Native,
            },
            peer,
        )
    }

    #[cfg(test)]
    pub(crate) fn test_set_word_scope(
        &self,
        owner: &EngineOwner,
        tail_epoch: u64,
        scope: &WordScope,
    ) -> bool {
        self.shared.reducer.lock().is_ok_and(|mut reducer| {
            if !reducer.is_current_owner(owner) {
                return false;
            }
            reducer.latest_tail_epoch = tail_epoch;
            reducer.lineage = scope.lineage();
            true
        })
    }

    pub(crate) async fn observe_callback(
        &self,
        header: &zbus::message::Header<'_>,
        callback_entered: Instant,
    ) -> Result<ObservedCallback, AdapterError> {
        if self.shared.cancelled.load(Ordering::Acquire) {
            return Err(AdapterError::Cancelled);
        }
        let key = HeaderKey::from_zbus_header(self.shared.connection_generation, header)
            .ok_or(AdapterError::Denied)?;
        if !self
            .shared
            .bindings
            .accepts(HeaderRole::EngineCallback, &key)
        {
            trace::record_context_admission(
                "callback_stamp",
                &key.member,
                key.serial,
                "refused_binding",
                None,
                None,
                None,
            );
            return Err(AdapterError::Denied);
        }
        let Some(guard) = self.shared.stamps.guard() else {
            trace::record_context_admission(
                "callback_stamp",
                &key.member,
                key.serial,
                "guard_absent",
                None,
                None,
                None,
            );
            return Err(AdapterError::Denied);
        };
        match rendezvous_stamp(&self.shared.stamps, key.clone(), guard, callback_entered).await {
            RendezvousOutcome::Stamp(stamp) => {
                if trace::enabled() {
                    let (disposition, owner_generation) =
                        context_admission_trace_disposition(&stamp.disposition);
                    trace::record_context_admission(
                        "callback_stamp",
                        &stamp.header.member,
                        stamp.header.serial,
                        disposition,
                        owner_generation,
                        self.current_activation_generation(),
                        None,
                    );
                }
                Ok(ObservedCallback {
                    header: stamp.header,
                    position: stamp.position,
                    disposition: stamp.disposition,
                    guard,
                })
            }
            RendezvousOutcome::Failed(failure) => {
                trace::record_context_admission(
                    "callback_stamp",
                    &key.member,
                    key.serial,
                    rendezvous_failure_trace(failure),
                    None,
                    None,
                    None,
                );
                Err(AdapterError::Denied)
            }
        }
    }

    pub(crate) async fn begin_key_callback(
        &self,
        expected_owner: &EngineOwner,
        header: &zbus::message::Header<'_>,
        callback_entered: Instant,
    ) -> Result<KeyCallback, AdapterError> {
        let observed = self.observe_callback(header, callback_entered).await?;
        self.key_callback_from_observed(expected_owner, observed, false)
    }

    pub(crate) async fn begin_atomic_key_callback(
        &self,
        expected_owner: &EngineOwner,
        header: &zbus::message::Header<'_>,
        callback_entered: Instant,
    ) -> Result<KeyCallback, AdapterError> {
        let observed = self.observe_callback(header, callback_entered).await?;
        self.key_callback_from_observed(expected_owner, observed, true)
    }

    pub(crate) fn key_callback_from_observed(
        &self,
        expected_owner: &EngineOwner,
        observed: ObservedCallback,
        atomic: bool,
    ) -> Result<KeyCallback, AdapterError> {
        let member = if atomic {
            "ProcessKeyEventAtomicV1"
        } else {
            "ProcessKeyEvent"
        };
        if observed.header.member != member {
            return Err(AdapterError::Denied);
        }
        let owner = match &observed.disposition {
            IngressDisposition::Key { owner, .. } => owner.clone(),
            _ => return Err(AdapterError::Denied),
        };
        if &owner != expected_owner || observed.guard.owner != owner.generation {
            return Err(AdapterError::Denied);
        }
        Ok(KeyCallback { observed, owner })
    }

    pub(crate) async fn observe_content_type_callback(
        &self,
        path: &EnginePath,
        value: (u32, u32),
        header: &zbus::message::Header<'_>,
        callback_entered: Instant,
    ) -> Result<Option<bool>, AdapterError> {
        let observed = self.observe_callback(header, callback_entered).await?;
        let path_matches = header
            .path()
            .is_some_and(|observed_path| observed_path.as_str() == path.as_str());
        if !header
            .interface()
            .is_some_and(|interface| interface.as_str() == PROPERTIES_INTERFACE)
            || observed.header.member != "Set"
            || !path_matches
        {
            return Err(AdapterError::Denied);
        }
        match observed.disposition {
            IngressDisposition::UnchangedContentType {
                token,
                value: recorded,
            } => {
                if recorded != value {
                    return Err(AdapterError::Denied);
                }
                let live = self
                    .shared
                    .reducer
                    .lock()
                    .is_ok_and(|reducer| reducer.revalidate_content_type(&token, path, value));
                // A delayed equality witness cannot touch a newer context.
                Ok(live.then_some(true))
            }
            IngressDisposition::Revocation { .. } | IngressDisposition::StaleContext => {
                Ok(Some(false))
            }
            _ => Err(AdapterError::Denied),
        }
    }

    /// The observer already cancelled this older key at a later revocation.
    /// A caller proving no text effect must leave the successor state alone.
    pub(crate) fn key_observation_was_revoked(
        &self,
        callback: &KeyCallback,
        tail_epoch: u64,
    ) -> bool {
        let IngressDisposition::Key { revocation, .. } = &callback.observed.disposition else {
            return false;
        };
        let retired = self.shared.reducer.lock().is_ok_and(|reducer| {
            reducer.owner() == Some(&callback.owner)
                && reducer.revocation_generation() != *revocation
                && reducer.latest_tail_epoch == tail_epoch
        });
        if retired {
            trace::record_context_admission(
                "callback_settlement",
                &callback.observed.header.member,
                callback.observed.header.serial,
                "retired_after_revocation",
                Some(callback.owner.generation.0),
                trace::enabled()
                    .then(|| self.current_activation_generation())
                    .flatten(),
                None,
            );
        }
        retired
    }

    pub(crate) fn settle_key_callback(
        &self,
        callback: &KeyCallback,
        settled_state: SettledWordState,
    ) -> bool {
        let accepted = self.shared.reducer.lock().is_ok_and(|mut reducer| {
            reducer.settle_key(&callback.owner, &callback.observed.header, settled_state)
        });
        let activation_generation = trace::enabled()
            .then(|| self.current_activation_generation())
            .flatten();
        trace::record_context_admission(
            "callback_settlement",
            &callback.observed.header.member,
            callback.observed.header.serial,
            if accepted { "accepted" } else { "refused" },
            Some(callback.owner.generation.0),
            activation_generation,
            accepted.then_some(word_completeness_trace(settled_state.lineage.completeness)),
        );
        if accepted {
            self.refresh_acquisition_fence_ready();
        }
        accepted
    }

    pub(crate) fn abandon_observed_key(&self, observed: &ObservedCallback) {
        if let IngressDisposition::Key { owner, .. } = &observed.disposition {
            if self
                .shared
                .reducer
                .lock()
                .is_ok_and(|mut reducer| reducer.abandon_key(owner, &observed.header))
            {
                self.shared.stamps.revoke();
                self.refresh_acquisition_fence_ready();
            }
        }
    }

    pub(crate) async fn begin_factory_callback(
        &self,
        header: &zbus::message::Header<'_>,
        callback_entered: Instant,
        expected_profile: EngineProfile,
    ) -> Result<FactoryCallback, AdapterError> {
        let observed = self.observe_callback(header, callback_entered).await?;
        if observed.header.member != "CreateEngine" {
            return Err(AdapterError::Denied);
        }
        let IngressDisposition::Factory {
            ticket,
            expected_profile: observed_profile,
        } = &observed.disposition
        else {
            return Err(AdapterError::Denied);
        };
        if observed_profile != &expected_profile {
            return Err(AdapterError::Denied);
        }
        Ok(FactoryCallback {
            ticket: *ticket,
            #[cfg(test)]
            position: observed.position,
        })
    }

    pub(crate) fn bind_factory_target(
        &self,
        callback: &FactoryCallback,
        target_path: EnginePath,
    ) -> bool {
        self.shared
            .reducer
            .lock()
            .is_ok_and(|mut reducer| reducer.bind_factory_target(callback.ticket, target_path))
    }

    pub(crate) fn focus_out(&self, owner: &EngineOwner, stamp: &ObservedCallback) -> bool {
        let ticket = match &stamp.disposition {
            IngressDisposition::FocusOut {
                owner: observed_owner,
                ticket,
            } if observed_owner == owner => *ticket,
            _ => return false,
        };
        self.shared.reducer.lock().is_ok_and(|reducer| {
            reducer.ticket.as_ref().is_some_and(|current| {
                current.id == ticket
                    && current.status == TicketStatus::Pending
                    && current.source_owner == *owner
                    && current.focus_out_position.as_ref() == Some(&stamp.position)
            })
        })
    }

    pub(crate) fn disable(&self, owner: &EngineOwner, stamp: &ObservedCallback) -> bool {
        if !matches!(
            &stamp.disposition,
            IngressDisposition::Disable { owner: observed_owner } if observed_owner == owner
        ) {
            return false;
        }
        self.shared.reducer.lock().is_ok_and(|reducer| {
            reducer.ticket.as_ref().is_some_and(|current| {
                current.status == TicketStatus::Pending
                    && current.source_owner == *owner
                    && current.disable_position.as_ref() == Some(&stamp.position)
            })
        })
    }

    pub(crate) fn callback_is_stale(&self, stamp: &ObservedCallback) -> bool {
        matches!(stamp.disposition, IngressDisposition::StaleContext)
    }

    pub(crate) fn callback_is_stale_for(
        &self,
        owner: &EngineOwner,
        stamp: &ObservedCallback,
    ) -> bool {
        match &stamp.disposition {
            IngressDisposition::StaleContext => true,
            IngressDisposition::Key {
                owner: observed_owner,
                ..
            }
            | IngressDisposition::FocusOut {
                owner: observed_owner,
                ..
            }
            | IngressDisposition::Disable {
                owner: observed_owner,
            }
            | IngressDisposition::Revocation {
                owner: observed_owner,
            } => observed_owner != owner,
            IngressDisposition::UnchangedContentType { token, .. } => &token.owner != owner,
            IngressDisposition::Passive | IngressDisposition::Factory { .. } => false,
        }
    }

    pub(crate) fn revocation_matches(&self, owner: &EngineOwner, stamp: &ObservedCallback) -> bool {
        let current_owner = self.current_owner();
        matches!(
            &stamp.disposition,
            IngressDisposition::Revocation {
                owner: observed_owner,
            } if observed_owner == owner
        ) && (current_owner.as_ref() == Some(owner)
            || (current_owner.is_none() && stamp.guard.owner == owner.generation))
    }

    pub(crate) fn seal_source(
        &self,
        owner: &EngineOwner,
        tail_epoch: u64,
        stamp: &ObservedCallback,
    ) -> bool {
        let sealed = self
            .shared
            .reducer
            .lock()
            .is_ok_and(|mut reducer| reducer.seal_source(owner, tail_epoch, stamp.position));
        if sealed {
            self.refresh_acquisition_fence_ready();
        }
        sealed
    }

    #[cfg(test)]
    pub(crate) async fn begin_compatibility_activation(
        &self,
        target_path: EnginePath,
        focus_position: Sequence,
    ) -> Result<PendingFence, AdapterError> {
        let deadline = self.activation_deadline();
        let nonce = self.next_nonce();
        let request = self.begin_activation_request(
            target_path.clone(),
            nonce,
            ReceiptOrigin::CompatibilityProperty,
            focus_position,
        )?;
        self.finish_compatibility_activation(target_path, request, nonce, deadline)
            .await
    }

    pub(crate) fn start_compatibility_activation(
        &self,
        target_path: EnginePath,
        focus_position: Sequence,
    ) -> Result<(), AdapterError> {
        let deadline = self.activation_deadline();
        let nonce = self.next_nonce();
        let request = self.begin_activation_request(
            target_path.clone(),
            nonce,
            ReceiptOrigin::CompatibilityProperty,
            focus_position,
        )?;
        let adapter = self.clone();
        self.shared
            .connection
            .executor()
            .spawn(
                async move {
                    if let Ok(fence) = adapter
                        .finish_compatibility_activation(target_path, request, nonce, deadline)
                        .await
                    {
                        async_io::Timer::at(fence.deadline).await;
                        adapter.expire_fence(fence);
                    }
                },
                "lay-context-compatibility-acquisition",
            )
            .detach();
        Ok(())
    }

    async fn finish_compatibility_activation(
        &self,
        target_path: EnginePath,
        request: RequestGeneration,
        nonce: BarrierNonce,
        deadline: Instant,
    ) -> Result<PendingFence, AdapterError> {
        trace::record_admission_timing("get_started", Some(request.0), nonce.0, Some(deadline));
        let result = async {
            let mut operation = Box::pin(self.shared.connection.call_method(
                Some(IBUS_NAME),
                IBUS_PATH,
                Some(PROPERTIES_INTERFACE),
                "Get",
                &(IBUS_INTERFACE, "CurrentInputContext"),
            ));
            let mut timer = Box::pin(async move {
                let _ = async_io::Timer::at(deadline).await;
            });
            let mut changed = Box::pin(self.shared.changed.listen());
            let reply = poll_fn(|task| {
                let request_state = self.shared.reducer.lock().ok().and_then(|reducer| {
                    reducer.request.as_ref().and_then(|current| {
                        (current.generation == request && current.nonce == nonce)
                            .then_some((current.origin, current.reply.is_some()))
                    })
                });
                match request_state {
                    Some((ReceiptOrigin::Native, true)) => return Poll::Ready(Ok(None)),
                    Some(_) => {}
                    None => return Poll::Ready(Err(AdapterError::Denied)),
                }
                if let Poll::Ready(result) = operation.as_mut().poll(task) {
                    return Poll::Ready(result.map(Some).map_err(AdapterError::Bus));
                }
                if timer.as_mut().poll(task).is_ready() {
                    return Poll::Ready(Err(AdapterError::Timeout));
                }
                if changed.as_mut().poll(task).is_ready() {
                    changed = Box::pin(self.shared.changed.listen());
                    task.waker().wake_by_ref();
                }
                Poll::Pending
            })
            .await?;
            if let Some(reply) = reply {
                let context_value = reply
                    .body()
                    .deserialize::<OwnedValue>()
                    .map_err(|_| AdapterError::Denied)?;
                let context_path =
                    OwnedObjectPath::try_from(context_value).map_err(|_| AdapterError::Denied)?;
                let context =
                    ContextKey::new(self.shared.connection_generation, context_path.as_str())
                        .ok_or(AdapterError::Denied)?;
                if !self.shared.reducer.lock().is_ok_and(|mut reducer| {
                    reducer.context_reply(request, nonce, context, reply.recv_position())
                }) {
                    return Err(AdapterError::Denied);
                }
            }
            trace::record_admission_timing(
                "reply_accepted",
                Some(request.0),
                nonce.0,
                Some(deadline),
            );
            self.arm_and_emit_marker(target_path, request, nonce, deadline)
                .await
        }
        .await;
        if result.is_err() {
            if let Ok(mut reducer) = self.shared.reducer.lock() {
                reducer.context_acquisition_failed(request);
            }
        }
        result
    }

    #[cfg(test)]
    pub(crate) async fn begin_native_activation(
        &self,
        target_path: EnginePath,
        context: ContextKey,
        focus_position: Sequence,
    ) -> Result<PendingFence, AdapterError> {
        let deadline = self.activation_deadline();
        let nonce = self.next_nonce();
        let request = self.begin_activation_request(
            target_path.clone(),
            nonce,
            ReceiptOrigin::Native,
            focus_position,
        )?;
        self.finish_native_activation(
            target_path,
            context,
            focus_position,
            request,
            nonce,
            deadline,
        )
        .await
    }

    pub(crate) fn start_native_activation(
        &self,
        target_path: EnginePath,
        context: ContextKey,
        focus_position: Sequence,
    ) -> Result<(), AdapterError> {
        let deadline = self.activation_deadline();
        let nonce = self.next_nonce();
        let request = self.begin_activation_request(
            target_path.clone(),
            nonce,
            ReceiptOrigin::Native,
            focus_position,
        )?;
        let adapter = self.clone();
        self.shared
            .connection
            .executor()
            .spawn(
                async move {
                    if let Ok(fence) = adapter
                        .finish_native_activation(
                            target_path,
                            context,
                            focus_position,
                            request,
                            nonce,
                            deadline,
                        )
                        .await
                    {
                        async_io::Timer::at(fence.deadline).await;
                        adapter.expire_fence(fence);
                    }
                },
                "lay-context-native-acquisition",
            )
            .detach();
        Ok(())
    }

    async fn finish_native_activation(
        &self,
        target_path: EnginePath,
        context: ContextKey,
        focus_position: Sequence,
        request: RequestGeneration,
        nonce: BarrierNonce,
        deadline: Instant,
    ) -> Result<PendingFence, AdapterError> {
        let trace_target = trace::enabled().then(|| target_path.as_str().to_owned());
        let trace_context = trace::enabled().then(|| context.path.as_str().to_owned());
        let result = async {
            let (accepted, revocation) = match self.shared.reducer.lock() {
                Ok(mut reducer) => {
                    let accepted = reducer.context_reply(request, nonce, context, focus_position);
                    (accepted, Some(reducer.revocation_generation()))
                }
                Err(error) => {
                    drop(error);
                    (false, None)
                }
            };
            if let Some(target) = trace_target.as_deref() {
                trace::record_native_activation(trace::NativeActivationTrace {
                    stage: "context_reply",
                    outcome: if accepted { "accepted" } else { "refused" },
                    target_path: target,
                    request_generation: Some(request.0),
                    nonce: nonce.0,
                    context_path: trace_context.as_deref(),
                    owner_generation: None,
                    revocation,
                    profile: None,
                    route: None,
                });
            }
            if !accepted {
                return Err(AdapterError::Denied);
            }
            self.arm_and_emit_marker(target_path, request, nonce, deadline)
                .await
        }
        .await;
        if result.is_err() {
            if let Ok(mut reducer) = self.shared.reducer.lock() {
                reducer.context_acquisition_failed(request);
            }
        }
        result
    }

    #[cfg(test)]
    pub(crate) fn finish_activation(
        &self,
        fence: PendingFence,
    ) -> Result<ActivationOutcome, AdapterError> {
        self.take_ready_activation(Some(fence), None)
            .ok_or(AdapterError::Denied)
    }

    #[cfg(test)]
    pub(crate) fn try_finish_activation_for(
        &self,
        target_path: &EnginePath,
    ) -> Result<Option<ActivationOutcome>, AdapterError> {
        Ok(self.take_ready_activation(None, Some(target_path)))
    }

    pub(crate) fn pending_activation_for(
        &self,
        target_path: &EnginePath,
    ) -> Option<ActivationOutcome> {
        self.discard_stale_ready_activation();
        self.shared
            .ready_activation
            .lock()
            .ok()?
            .as_ref()
            .filter(|ready| &ready.target_path == target_path)
            .map(|ready| ready.outcome.clone())
    }

    pub(crate) fn acknowledge_activation(&self, outcome: &ActivationOutcome) -> bool {
        // Same ready-before-reducer order as publication. Never acknowledge a
        // Reset replacement using a previously peeked transferable outcome.
        let Ok(mut ready) = self.shared.ready_activation.lock() else {
            return false;
        };
        let Ok(reducer) = self.shared.reducer.lock() else {
            return false;
        };
        if !reducer.revalidate(&outcome.token()) {
            return false;
        }
        if !ready
            .as_ref()
            .is_some_and(|current| &current.outcome == outcome)
        {
            return false;
        }
        *ready = None;
        drop(reducer);
        drop(ready);
        self.refresh_acquisition_fence_ready();
        true
    }

    pub(crate) async fn begin_bridge_fence(&self) -> Result<PendingFence, AdapterError> {
        let deadline = self.acquisition_deadline();
        let nonce = self.next_nonce();
        trace::record_admission_timing("bridge_begin", None, nonce.0, Some(deadline));
        let sent = OwnedValue::from(nonce.0);
        let reply = before_deadline(
            self.shared.connection.call_method(
                Some(IBUS_NAME),
                IBUS_PATH,
                Some(IBUS_INTERFACE),
                "Ping",
                &sent,
            ),
            deadline,
        )
        .await?;
        trace::record_admission_timing("bridge_ping_complete", None, nonce.0, Some(deadline));
        let echoed = reply
            .body()
            .deserialize::<OwnedValue>()
            .map_err(|_| AdapterError::Denied)?;
        if echoed != sent {
            return Err(AdapterError::Denied);
        }
        self.arm_fence(PendingFenceState {
            nonce,
            deadline,
            kind: FenceKind::Bridge {
                ping_position: reply.recv_position(),
            },
            ready_token: None,
            marker_observed: false,
            ready: false,
        })?;
        if before_deadline(
            self.shared.connection.emit_signal(
                None::<()>,
                MARKER_PATH,
                MARKER_INTERFACE,
                MARKER_MEMBER,
                &nonce.0,
            ),
            deadline,
        )
        .await
        .is_err()
        {
            self.clear_fence(nonce);
            return Err(AdapterError::Timeout);
        }
        trace::record_admission_timing("bridge_marker_emitted", None, nonce.0, Some(deadline));
        Ok(PendingFence { nonce, deadline })
    }

    pub(crate) fn finish_bridge_fence(
        &self,
        fence: PendingFence,
    ) -> Result<AdmissionToken, AdapterError> {
        self.take_ready_fence(fence, true)?
            .ok_or(AdapterError::Denied)
    }

    pub(crate) fn revalidate(&self, token: &AdmissionToken) -> bool {
        self.shared
            .reducer
            .lock()
            .is_ok_and(|reducer| reducer.revalidate(token))
    }

    pub(crate) fn revalidate_bridge(&self, token: &AdmissionToken) -> bool {
        self.shared
            .reducer
            .lock()
            .is_ok_and(|reducer| reducer.revalidate_bridge(token))
    }

    pub(crate) fn settle_soft_reset_rereceipt(
        &self,
        owner: &EngineOwner,
        reset_token: &AdmissionToken,
        settled: SettledWordState,
    ) -> Option<AdmissionToken> {
        self.shared.reducer.lock().ok().and_then(|mut reducer| {
            reducer
                .settle_soft_reset_rereceipt(owner, reset_token, settled)
                .then(|| reducer.admission_token())
                .flatten()
        })
    }

    pub(crate) fn settle_bridge_output(
        &self,
        token: &AdmissionToken,
        settled: SettledWordState,
    ) -> bool {
        self.shared
            .reducer
            .lock()
            .is_ok_and(|mut reducer| reducer.settle_bridge_output(token, settled))
    }

    pub(crate) fn publish_exact_manual_snapshot(
        &self,
        token: &AdmissionToken,
        tail_epoch: u64,
        tail: String,
        observation_revision: u64,
        expires_at: Instant,
    ) -> bool {
        self.shared.reducer.lock().is_ok_and(|mut reducer| {
            reducer.publish_exact_manual_snapshot(
                token,
                tail_epoch,
                tail,
                observation_revision,
                expires_at,
            )
        })
    }

    pub(crate) fn invalidate_exact_manual_snapshot(&self, owner: &EngineOwner) {
        if let Ok(mut reducer) = self.shared.reducer.lock() {
            reducer.invalidate_exact_manual_snapshot(owner);
        }
    }

    pub(crate) fn current_token(&self) -> Option<AdmissionToken> {
        self.shared
            .reducer
            .lock()
            .ok()
            .and_then(|reducer| reducer.admission_token())
    }

    pub(crate) fn current_layout_intent_token_for(
        &self,
        owner: &EngineOwner,
    ) -> Option<LayoutIntentToken> {
        self.shared
            .reducer
            .lock()
            .ok()
            .and_then(|reducer| reducer.layout_intent_token_for(owner))
    }

    pub(crate) fn revalidate_layout_intent(&self, token: &LayoutIntentToken) -> bool {
        self.shared
            .reducer
            .lock()
            .is_ok_and(|reducer| reducer.revalidate_layout_intent(token))
    }

    pub(crate) fn revoke_layout_intent(&self, token: &LayoutIntentToken) -> bool {
        self.shared
            .reducer
            .lock()
            .is_ok_and(|mut reducer| reducer.revoke_layout_intent(token))
    }

    pub(crate) fn revoke_current_owner(&self, owner: &EngineOwner) -> bool {
        let Ok(mut reducer) = self.shared.reducer.lock() else {
            return false;
        };
        if !reducer.is_current_owner(owner) {
            return false;
        }
        reducer.malformed_or_unobserved_lifecycle();
        self.shared.stamps.revoke();
        true
    }

    pub(crate) fn ready_reset_replaces(&self, owner: &EngineOwner) -> bool {
        let Ok(ready) = self.shared.ready_activation.lock() else {
            return false;
        };
        let Some(current) = ready.as_ref() else {
            return false;
        };
        let Ok(reducer) = self.shared.reducer.lock() else {
            return false;
        };
        matches!(
            &current.outcome,
            ActivationOutcome::ResetUnknown(grant) if &grant.target_owner == owner
        ) && reducer.revalidate(&current.outcome.token())
    }

    #[cfg(test)]
    pub(crate) async fn complete_activation(
        &self,
        fence: PendingFence,
    ) -> Result<ActivationOutcome, AdapterError> {
        if let Err(error) = self.wait_for_fence(fence, false).await {
            self.expire_fence(fence);
            return Err(error);
        }
        self.finish_activation(fence)
    }

    pub(crate) async fn complete_bridge_fence(
        &self,
        fence: PendingFence,
    ) -> Result<AdmissionToken, AdapterError> {
        if let Err(error) = self.wait_for_fence(fence, true).await {
            trace::record_admission_timing(
                "bridge_wait_failed",
                None,
                fence.nonce.0,
                Some(fence.deadline),
            );
            self.expire_fence(fence);
            return Err(error);
        }
        trace::record_admission_timing(
            "bridge_wait_complete",
            None,
            fence.nonce.0,
            Some(fence.deadline),
        );
        self.finish_bridge_fence(fence)
    }

    pub(crate) fn current_owner(&self) -> Option<EngineOwner> {
        self.shared
            .reducer
            .lock()
            .ok()
            .and_then(|reducer| reducer.owner().cloned())
    }

    pub(crate) fn current_activation_generation(&self) -> Option<u64> {
        self.shared.reducer.lock().ok().and_then(|reducer| {
            reducer
                .activation()
                .map(|activation| activation.generation.0)
        })
    }

    pub(crate) fn activation_outcome_is_current(&self, outcome: &ActivationOutcome) -> bool {
        self.activation_outcome_token(outcome).is_some()
    }

    pub(crate) fn activation_outcome_token(
        &self,
        outcome: &ActivationOutcome,
    ) -> Option<AdmissionToken> {
        let token = outcome.token();
        self.revalidate(&token).then_some(token)
    }

    pub(crate) fn bind_source_free_tail_epoch(
        &self,
        grant: &ActivationGrant,
        tail_epoch: u64,
    ) -> bool {
        self.shared
            .reducer
            .lock()
            .is_ok_and(|mut reducer| reducer.bind_source_free_tail_epoch(grant, tail_epoch))
    }

    pub(crate) fn bind_reset_tail_epoch(&self, grant: &ActivationGrant, tail_epoch: u64) -> bool {
        self.shared
            .reducer
            .lock()
            .is_ok_and(|mut reducer| reducer.bind_reset_tail_epoch(grant, tail_epoch))
    }

    #[cfg(test)]
    pub(crate) fn test_bind_activation_outcome(&self, outcome: &ActivationOutcome) {
        let (owner, activation, lineage) = match outcome {
            ActivationOutcome::Transfer(grant) => (
                grant.target_owner.clone(),
                grant.target_activation.clone(),
                grant.lineage,
            ),
            ActivationOutcome::SourceFree(grant) | ActivationOutcome::ResetUnknown(grant) => (
                grant.target_owner.clone(),
                grant.target_activation.clone(),
                grant.lineage,
            ),
        };
        let mut reducer = self.shared.reducer.lock().expect("context test reducer");
        reducer.owner = Some(owner);
        reducer.activation = Some(activation);
        reducer.lineage = lineage;
    }

    pub(crate) fn context_key(&self, path: impl Into<String>) -> Option<ContextKey> {
        ContextKey::new(self.shared.connection_generation, path)
    }

    pub(crate) fn enrich_native_activation(
        &self,
        expected_owner: &EngineOwner,
        target_path: &EnginePath,
        context: &ContextKey,
        observed: &ObservedCallback,
    ) -> Option<bool> {
        if observed.header.member != "FocusInId"
            || observed.guard.owner != expected_owner.generation
            || self.callback_is_stale_for(expected_owner, observed)
        {
            return Some(false);
        }
        match self.shared.reducer.lock() {
            Ok(mut reducer) => {
                reducer.enrich_native_activation(expected_owner, target_path, context)
            }
            Err(_) => Some(false),
        }
    }

    pub(crate) fn enrich_pending_native_activation(
        &self,
        target_path: &EnginePath,
        context: ContextKey,
        observed: &ObservedCallback,
    ) -> bool {
        if observed.header.member != "FocusInId"
            || observed.header.path != target_path.as_str()
            || self.callback_is_stale(observed)
        {
            return false;
        }
        let enriched = self.shared.reducer.lock().is_ok_and(|mut reducer| {
            reducer.enrich_pending_native_activation(target_path, context, observed.position)
        });
        if enriched {
            self.shared.changed.notify(usize::MAX);
        }
        enriched
    }

    #[cfg(test)]
    pub(crate) fn cancel_observer(&self) {
        self.shared.cancelled.store(true, Ordering::Release);
        self.shared.cancellation.notify(usize::MAX);
        self.shared.changed.notify(usize::MAX);
    }

    fn begin_activation_request(
        &self,
        target_path: EnginePath,
        nonce: BarrierNonce,
        origin: ReceiptOrigin,
        focus_position: Sequence,
    ) -> Result<RequestGeneration, AdapterError> {
        let trace_target = (origin == ReceiptOrigin::Native && trace::enabled())
            .then(|| target_path.as_str().to_owned());
        // Serialize reducer begin with completed-result publication. Reuse
        // the pending-slot lock; never await or prune ready state while held.
        let publication = match self.shared.pending.lock() {
            Ok(publication) => publication,
            Err(error) => {
                drop(error);
                if let Some(target) = trace_target.as_deref() {
                    trace::record_native_activation(trace::NativeActivationTrace {
                        stage: "begin",
                        outcome: "refused_pending_lock",
                        target_path: target,
                        request_generation: None,
                        nonce: nonce.0,
                        context_path: None,
                        owner_generation: None,
                        revocation: None,
                        profile: None,
                        route: None,
                    });
                }
                return Err(AdapterError::Denied);
            }
        };
        if publication.is_some() {
            drop(publication);
            if let Some(target) = trace_target.as_deref() {
                trace::record_native_activation(trace::NativeActivationTrace {
                    stage: "begin",
                    outcome: "busy",
                    target_path: target,
                    request_generation: None,
                    nonce: nonce.0,
                    context_path: None,
                    owner_generation: None,
                    revocation: None,
                    profile: None,
                    route: Some("pending_occupied"),
                });
            }
            return Err(AdapterError::Busy);
        }
        let mut reducer = match self.shared.reducer.lock() {
            Ok(reducer) => reducer,
            Err(error) => {
                drop(error);
                drop(publication);
                if let Some(target) = trace_target.as_deref() {
                    trace::record_native_activation(trace::NativeActivationTrace {
                        stage: "begin",
                        outcome: "refused_reducer_lock",
                        target_path: target,
                        request_generation: None,
                        nonce: nonce.0,
                        context_path: None,
                        owner_generation: None,
                        revocation: None,
                        profile: None,
                        route: None,
                    });
                }
                return Err(AdapterError::Denied);
            }
        };
        let has_transfer = reducer.ticket.as_ref().is_some_and(|ticket| {
            ticket.status == TicketStatus::Pending
                && ticket.target_path.as_ref() == Some(&target_path)
        });
        let trace_profile = trace_target.as_ref().map(|_| match &reducer.profile {
            GlobalProfile::Lay(profile) | GlobalProfile::Foreign(profile) => {
                profile.as_str().to_owned()
            }
        });
        let trace_revocation = reducer.revocation_generation();
        let trace_route = if has_transfer {
            "transfer"
        } else {
            "source_free"
        };
        let request = if has_transfer {
            reducer.focus_in(&target_path, nonce, origin, focus_position)
        } else {
            reducer.begin_source_free_activation(target_path, nonce, origin, focus_position)
        }
        .ok_or(AdapterError::Denied);
        drop(reducer);
        drop(publication);
        if let Some(target) = trace_target.as_deref() {
            trace::record_native_activation(trace::NativeActivationTrace {
                stage: "begin",
                outcome: if request.is_ok() {
                    "accepted"
                } else {
                    "refused"
                },
                target_path: target,
                request_generation: request.as_ref().ok().map(|request| request.0),
                nonce: nonce.0,
                context_path: None,
                owner_generation: None,
                revocation: Some(trace_revocation),
                profile: trace_profile.as_deref(),
                route: Some(trace_route),
            });
        }
        self.discard_stale_ready_activation();
        request
    }

    fn discard_stale_ready_activation(&self) {
        // Reset replaces an outcome inside the same request/owner slot. Keep
        // validation and removal atomic so a stale snapshot cannot erase its
        // current ResetUnknown replacement.
        let Ok(mut ready) = self.shared.ready_activation.lock() else {
            return;
        };
        let Some(current) = ready.as_ref() else {
            return;
        };
        let Ok(reducer) = self.shared.reducer.lock() else {
            return;
        };
        if reducer.revalidate(&current.outcome.token()) {
            return;
        }
        let ready_fence = ready.take().map(|ready| ready.fence);
        drop(reducer);
        drop(ready);
        if let Some(fence) = ready_fence {
            self.clear_fence(fence.nonce);
        }
    }

    async fn arm_and_emit_marker(
        &self,
        target_path: EnginePath,
        request: RequestGeneration,
        nonce: BarrierNonce,
        deadline: Instant,
    ) -> Result<PendingFence, AdapterError> {
        let trace_target = trace::enabled().then(|| target_path.as_str().to_owned());
        let arm_result = self.arm_fence(PendingFenceState {
            nonce,
            deadline,
            kind: FenceKind::Acquisition {
                request,
                target_path,
            },
            ready_token: None,
            marker_observed: false,
            ready: false,
        });
        if let Some(target) = trace_target.as_deref() {
            trace::record_native_activation(trace::NativeActivationTrace {
                stage: "marker_arm",
                outcome: match &arm_result {
                    Ok(()) => "armed",
                    Err(AdapterError::Busy) => "refused_busy",
                    Err(AdapterError::Denied) => "refused_denied",
                    Err(_) => "refused_error",
                },
                target_path: target,
                request_generation: Some(request.0),
                nonce: nonce.0,
                context_path: None,
                owner_generation: None,
                revocation: None,
                profile: None,
                route: None,
            });
        }
        arm_result?;
        if before_deadline(
            self.shared.connection.emit_signal(
                None::<()>,
                MARKER_PATH,
                MARKER_INTERFACE,
                MARKER_MEMBER,
                &nonce.0,
            ),
            deadline,
        )
        .await
        .is_err()
        {
            self.clear_fence(nonce);
            if let Ok(mut reducer) = self.shared.reducer.lock() {
                reducer.context_acquisition_failed(request);
            }
            if let Some(target) = trace_target.as_deref() {
                trace::record_native_activation(trace::NativeActivationTrace {
                    stage: "marker_emission",
                    outcome: "timeout",
                    target_path: target,
                    request_generation: Some(request.0),
                    nonce: nonce.0,
                    context_path: None,
                    owner_generation: None,
                    revocation: None,
                    profile: None,
                    route: None,
                });
            }
            return Err(AdapterError::Timeout);
        }
        if let Some(target) = trace_target.as_deref() {
            trace::record_native_activation(trace::NativeActivationTrace {
                stage: "marker_emission",
                outcome: "emitted",
                target_path: target,
                request_generation: Some(request.0),
                nonce: nonce.0,
                context_path: None,
                owner_generation: None,
                revocation: None,
                profile: None,
                route: None,
            });
        }
        trace::record_admission_timing("marker_emitted", Some(request.0), nonce.0, Some(deadline));
        Ok(PendingFence { nonce, deadline })
    }

    fn arm_fence(&self, pending: PendingFenceState) -> Result<(), AdapterError> {
        let mut slot = self
            .shared
            .pending
            .lock()
            .map_err(|_| AdapterError::Denied)?;
        if slot.is_some() {
            return Err(AdapterError::Busy);
        }
        if let FenceKind::Acquisition {
            request,
            target_path,
        } = &pending.kind
        {
            let reducer = self
                .shared
                .reducer
                .lock()
                .map_err(|_| AdapterError::Denied)?;
            let current = reducer.request.as_ref().is_some_and(|current| {
                current.generation == *request
                    && current.nonce == pending.nonce
                    && current.lifecycle_revision == reducer.revocation_generation()
                    && match current.target {
                        RequestTarget::Transfer(ticket_id) => {
                            reducer.ticket.as_ref().is_some_and(|ticket| {
                                ticket.id == ticket_id
                                    && matches!(
                                        ticket.status,
                                        TicketStatus::Pending | TicketStatus::Ready
                                    )
                                    && ticket.target_path.as_ref() == Some(target_path)
                            })
                        }
                        RequestTarget::SourceFree => {
                            reducer.source_free.as_ref().is_some_and(|activation| {
                                matches!(
                                    activation.status,
                                    TicketStatus::Pending | TicketStatus::Ready
                                ) && &activation.target_path == target_path
                            })
                        }
                    }
            });
            if !current {
                return Err(AdapterError::Denied);
            }
            *slot = Some(pending);
            drop(reducer);
            return Ok(());
        }
        *slot = Some(pending);
        Ok(())
    }

    fn clear_fence(&self, nonce: BarrierNonce) {
        if let Ok(mut slot) = self.shared.pending.lock() {
            if slot.as_ref().is_some_and(|pending| pending.nonce == nonce) {
                *slot = None;
            }
        }
        self.shared.changed.notify(usize::MAX);
    }

    fn expire_fence(&self, fence: PendingFence) {
        let diagnostics_enabled = trace::diagnostics_enabled_cached();
        let (removed, observed_at_expiry, matching_request) = self
            .shared
            .pending
            .lock()
            .ok()
            .map(|mut slot| {
                let matches = slot.as_ref().is_some_and(|pending| {
                    pending.nonce == fence.nonce && pending.deadline == fence.deadline
                });
                let marker_observed =
                    matches && slot.as_ref().is_some_and(|pending| pending.marker_observed);
                let matching_request = matches
                    .then(|| {
                        slot.as_ref().and_then(|pending| match pending.kind {
                            FenceKind::Acquisition { request, .. } => Some(request),
                            FenceKind::Bridge { .. } => None,
                        })
                    })
                    .flatten();
                // Bridge has no ready-activation consumer after its wait
                // returns an error. Only Acquisition retains observed work.
                let observed_acquisition = marker_observed && matching_request.is_some();
                let removed = if matches && !observed_acquisition {
                    slot.take()
                } else {
                    None
                };
                (removed, marker_observed, matching_request)
            })
            .unwrap_or((None, false, None));
        let removed_request = removed.as_ref().and_then(|pending| match pending.kind {
            FenceKind::Acquisition { request, .. } => Some(request),
            FenceKind::Bridge { .. } => None,
        });
        if let Some(request) = removed_request {
            if let Ok(mut reducer) = self.shared.reducer.lock() {
                reducer.context_acquisition_failed(request);
            }
        }
        self.shared.changed.notify(usize::MAX);
        if diagnostics_enabled && matching_request.is_some() {
            trace::record_admission_timing(
                "fence_expiry",
                matching_request.map(|request| request.0),
                fence.nonce.0,
                Some(fence.deadline),
            );
            let diagnostic = self.shared.reducer.lock().ok().map(|reducer| {
                let reason = if observed_at_expiry {
                    matching_request
                        .map(|generation| {
                            let reason = reducer.compatibility_readiness_reason(generation);
                            if reason == "ready" {
                                "publication_not_completed"
                            } else {
                                reason
                            }
                        })
                        .unwrap_or("request_not_current")
                } else {
                    "marker_not_observed_before_expiry"
                };
                (matching_request, reason, reducer.diagnostic_snapshot())
            });
            if let Some((request, reason, snapshot)) = diagnostic {
                record_admission_diagnostic(
                    "compatibility",
                    if observed_at_expiry {
                        "expiry_post_decision_snapshot"
                    } else {
                        "expiry"
                    },
                    reason,
                    (request, fence.nonce),
                    snapshot,
                    observed_at_expiry,
                    observed_at_expiry,
                );
            }
        }
    }

    fn refresh_acquisition_fence_ready(&self) {
        self.discard_stale_ready_activation();
        // A current predecessor must be installed before its successor can
        // consume this single ready slot. No second queue/owner is needed.
        if self
            .shared
            .ready_activation
            .lock()
            .map_or(true, |ready| ready.is_some())
        {
            return;
        }
        // Keep begin from observing a consumed result with its old pending
        // fence still occupied. This critical section contains no await.
        let Ok(mut slot) = self.shared.pending.lock() else {
            return;
        };
        let Some(pending) = slot.clone() else {
            return;
        };
        let FenceKind::Acquisition {
            request,
            target_path,
        } = pending.kind.clone()
        else {
            return;
        };
        let request_observation = self
            .shared
            .reducer
            .lock()
            .map(|reducer| {
                (
                    reducer
                        .request
                        .as_ref()
                        .is_some_and(|current| current.generation == request),
                    Some(reducer.revocation_generation()),
                )
            })
            .ok();
        let (request_is_current, rejection_revocation) =
            request_observation.unwrap_or((false, None));
        if !request_is_current {
            *slot = None;
            drop(slot);
            trace::record_native_activation(trace::NativeActivationTrace {
                stage: "publication",
                outcome: "refused_not_current",
                target_path: target_path.as_str(),
                request_generation: Some(request.0),
                nonce: pending.nonce.0,
                context_path: None,
                owner_generation: None,
                revocation: rejection_revocation,
                profile: None,
                route: None,
            });
            self.shared.changed.notify(usize::MAX);
            return;
        }
        if !pending.marker_observed {
            return;
        }
        // Global order: pending -> ready -> reducer. Reset and ACK need only
        // ready -> reducer. Publish the witness before Reset can observe its
        // newly consumed owner; never leave owner present with ready absent.
        let Ok(mut ready) = self.shared.ready_activation.lock() else {
            return;
        };
        if ready.is_some() {
            return;
        }
        let promoted = self.shared.reducer.lock().ok().and_then(|mut reducer| {
            if reducer.status() != AdmissionStatus::Ready
                || reducer
                    .request
                    .as_ref()
                    .is_none_or(|current| current.generation != request)
            {
                return None;
            }
            let outcome = reducer
                .consume()
                .map(ActivationOutcome::Transfer)
                .or_else(|| {
                    reducer
                        .consume_source_free_activation()
                        .map(ActivationOutcome::SourceFree)
                })?;
            let owner = match &outcome {
                ActivationOutcome::Transfer(grant) => grant.target_owner.clone(),
                ActivationOutcome::SourceFree(grant) | ActivationOutcome::ResetUnknown(grant) => {
                    grant.target_owner.clone()
                }
            };
            // Classifying a key for this owner must never overtake publication
            // of that same owner's stamp guard (existing reducer->stamps order).
            self.shared.stamps.replace_owner(owner.generation);
            Some((owner, outcome))
        });
        let Some((owner, outcome)) = promoted else {
            return;
        };
        let (activation_generation, completeness, outcome_kind) = match &outcome {
            ActivationOutcome::Transfer(grant) => (
                grant.target_activation.generation.0,
                grant.lineage.completeness,
                "transfer_ready",
            ),
            ActivationOutcome::SourceFree(grant) | ActivationOutcome::ResetUnknown(grant) => (
                grant.target_activation.generation.0,
                grant.lineage.completeness,
                "source_free_ready",
            ),
        };
        let owner_generation = owner.generation.0;
        let revocation = outcome.token().revocation;
        let trace_target = trace::enabled().then(|| target_path.as_str().to_owned());
        *ready = Some(ReadyActivationState {
            fence: PendingFence {
                nonce: pending.nonce,
                deadline: pending.deadline,
            },
            #[cfg(test)]
            request,
            target_path,
            owner,
            outcome,
        });
        // Completed acquisition lives in ready_activation, not in the
        // in-progress fence slot needed by the next target or bridge.
        if slot.as_ref().is_some_and(|current| {
            current.nonce == pending.nonce && current.deadline == pending.deadline
        }) {
            *slot = None;
        }
        drop(ready);
        drop(slot);
        if let Some(target) = trace_target.as_deref() {
            trace::record_native_activation(trace::NativeActivationTrace {
                stage: "publication",
                outcome: "published",
                target_path: target,
                request_generation: Some(request.0),
                nonce: pending.nonce.0,
                context_path: None,
                owner_generation: Some(owner_generation),
                revocation: Some(revocation),
                profile: None,
                route: Some(outcome_kind),
            });
        }
        trace::record_context_admission(
            "activation_ready",
            "",
            0,
            outcome_kind,
            Some(owner_generation),
            Some(activation_generation),
            Some(word_completeness_trace(completeness)),
        );
        self.shared.changed.notify(usize::MAX);
    }

    #[cfg(test)]
    fn take_ready_activation(
        &self,
        fence: Option<PendingFence>,
        target_path: Option<&EnginePath>,
    ) -> Option<ActivationOutcome> {
        let candidate = self
            .shared
            .ready_activation
            .lock()
            .ok()
            .and_then(|ready| ready.clone())?;
        if fence.is_some_and(|expected| expected != candidate.fence)
            || target_path.is_some_and(|expected| expected != &candidate.target_path)
        {
            return None;
        }
        let outcome_is_current = self.activation_outcome_is_current(&candidate.outcome);
        let mut ready = self.shared.ready_activation.lock().ok()?;
        if ready.as_ref().is_none_or(|current| {
            current.request != candidate.request || current.owner != candidate.owner
        }) {
            return None;
        }
        let taken = ready.take().expect("matching ready activation");
        drop(ready);
        self.clear_fence(taken.fence.nonce);
        self.refresh_acquisition_fence_ready();
        outcome_is_current.then_some(taken.outcome)
    }

    fn take_ready_fence(
        &self,
        fence: PendingFence,
        bridge: bool,
    ) -> Result<Option<AdmissionToken>, AdapterError> {
        let mut slot = self
            .shared
            .pending
            .lock()
            .map_err(|_| AdapterError::Denied)?;
        let pending = slot.as_ref().ok_or(AdapterError::Denied)?;
        if pending.nonce != fence.nonce || pending.deadline != fence.deadline {
            return Err(AdapterError::Denied);
        }
        if Instant::now() > pending.deadline {
            let kind = pending.kind.clone();
            *slot = None;
            drop(slot);
            if bridge {
                trace::record_admission_timing(
                    "bridge_expired_at_take",
                    None,
                    fence.nonce.0,
                    Some(fence.deadline),
                );
            }
            if let FenceKind::Acquisition { request, .. } = kind {
                if let Ok(mut reducer) = self.shared.reducer.lock() {
                    reducer.context_acquisition_failed(request);
                }
            }
            return Err(AdapterError::Timeout);
        }
        if !pending.ready || bridge != matches!(pending.kind, FenceKind::Bridge { .. }) {
            return Err(AdapterError::Denied);
        }
        let token = pending.ready_token.clone();
        *slot = None;
        drop(slot);
        if bridge {
            trace::record_admission_timing(
                "bridge_taken",
                None,
                fence.nonce.0,
                Some(fence.deadline),
            );
        }
        Ok(token)
    }

    fn acquisition_deadline(&self) -> Instant {
        let now = Instant::now();
        now.checked_add(self.shared.acquisition_budget)
            .unwrap_or(now)
    }

    fn activation_deadline(&self) -> Instant {
        let now = Instant::now();
        now.checked_add(self.shared.activation_budget)
            .unwrap_or(now)
    }

    fn next_nonce(&self) -> BarrierNonce {
        let nonce = self.shared.next_nonce.fetch_add(1, Ordering::AcqRel);
        BarrierNonce(if nonce == 0 { 1 } else { nonce })
    }

    async fn wait_for_fence(&self, fence: PendingFence, bridge: bool) -> Result<(), AdapterError> {
        loop {
            let mut changed = Box::pin(self.shared.changed.listen());
            let mut cancellation = Box::pin(self.shared.cancellation.listen());
            if self.shared.cancelled.load(Ordering::Acquire) {
                return Err(AdapterError::Cancelled);
            }
            if !bridge {
                let ready = self
                    .shared
                    .ready_activation
                    .lock()
                    .map_err(|_| AdapterError::Denied)?;
                if ready.as_ref().is_some_and(|ready| ready.fence == fence) {
                    return Ok(());
                }
            }
            {
                let slot = self
                    .shared
                    .pending
                    .lock()
                    .map_err(|_| AdapterError::Denied)?;
                let pending = slot.as_ref().ok_or(AdapterError::Denied)?;
                if pending.nonce != fence.nonce
                    || pending.deadline != fence.deadline
                    || bridge != matches!(pending.kind, FenceKind::Bridge { .. })
                {
                    return Err(AdapterError::Denied);
                }
                if pending.ready {
                    return Ok(());
                }
            }
            if Instant::now() >= fence.deadline {
                return Err(AdapterError::Timeout);
            }
            let mut timer = Box::pin(async_io::Timer::at(fence.deadline));
            poll_fn(|context| {
                if changed.as_mut().poll(context).is_ready() {
                    return Poll::Ready(Ok(()));
                }
                if cancellation.as_mut().poll(context).is_ready()
                    || self.shared.cancelled.load(Ordering::Acquire)
                {
                    return Poll::Ready(Err(AdapterError::Cancelled));
                }
                if timer.as_mut().poll(context).is_ready() {
                    return Poll::Ready(Err(AdapterError::Timeout));
                }
                Poll::Pending
            })
            .await?;
        }
    }
}

pub(crate) struct ContextAdmissionObserver {
    shared: Arc<AdapterState>,
    streams: Option<JoinedMessages>,
}

impl ContextAdmissionObserver {
    fn refresh_acquisition_fence_ready(&self) {
        ContextAdmissionAdapter {
            shared: Arc::clone(&self.shared),
        }
        .refresh_acquisition_fence_ready();
    }

    pub(crate) async fn process_next(&mut self) -> Result<bool, AdapterError> {
        let mut cancellation = Box::pin(self.shared.cancellation.listen());
        if self.shared.cancelled.load(Ordering::Acquire) {
            self.fail_closed();
            return Err(AdapterError::Cancelled);
        }
        let shared = Arc::clone(&self.shared);
        let message = {
            let streams = self.streams.as_mut().ok_or(AdapterError::Cancelled)?;
            let mut next_message = Box::pin(next_ordered_message(streams.as_mut()));
            poll_fn(|context| {
                if let Poll::Ready(message) = next_message.as_mut().poll(context) {
                    return Poll::Ready(Ok(message));
                }
                if cancellation.as_mut().poll(context).is_ready()
                    || shared.cancelled.load(Ordering::Acquire)
                {
                    return Poll::Ready(Err(AdapterError::Cancelled));
                }
                Poll::Pending
            })
            .await?
        };
        let Some(message) = message else {
            self.fail_closed();
            return Ok(false);
        };
        let message = message?;
        self.process_message(message)?;
        Ok(true)
    }

    pub(crate) async fn run(mut self) -> Result<(), AdapterError> {
        while self.process_next().await? {}
        Ok(())
    }

    fn process_message(&self, message: Message) -> Result<(), AdapterError> {
        let header = message.header();
        let member = header.member().map(|member| member.as_str()).unwrap_or("");
        let interface = header
            .interface()
            .map(|interface| interface.as_str())
            .unwrap_or("");
        let path = header.path().map(|path| path.as_str()).unwrap_or("");
        let key = HeaderKey::from_zbus_header(self.shared.connection_generation, &header)
            .ok_or(AdapterError::Denied)?;

        if header.message_type() == Type::MethodCall
            && ((interface == FACTORY_INTERFACE && member == "CreateEngine")
                || (interface == ENGINE_INTERFACE && ENGINE_CALLBACK_MEMBERS.contains(&member))
                || (interface == PROPERTIES_INTERFACE
                    && member == "Set"
                    && path.starts_with(ENGINE_PATH_PREFIX)))
        {
            if !self
                .shared
                .bindings
                .accepts(HeaderRole::EngineCallback, &key)
            {
                self.fail_closed();
                return Err(AdapterError::Denied);
            }
            let mut stamp = IngressStamp::from_message(self.shared.connection_generation, &message)
                .ok_or(AdapterError::Denied)?;
            stamp.disposition = match self.apply_ingress(&message, interface, member, path, &stamp)
            {
                Ok(disposition) => disposition,
                Err(error) => {
                    if trace::enabled() {
                        trace::record_context_admission(
                            "observer_ingress_refused",
                            member,
                            stamp.header.serial,
                            "denied",
                            None,
                            None,
                            None,
                        );
                    }
                    self.fail_closed();
                    return Err(error);
                }
            };
            if trace::enabled() {
                let (disposition, owner_generation) =
                    context_admission_trace_disposition(&stamp.disposition);
                trace::record_context_admission(
                    "observer_ingress",
                    member,
                    stamp.header.serial,
                    disposition,
                    owner_generation,
                    ContextAdmissionAdapter {
                        shared: Arc::clone(&self.shared),
                    }
                    .current_activation_generation(),
                    None,
                );
            }
            self.refresh_acquisition_fence_ready();
            if !self.shared.stamps.publish(stamp) {
                self.fail_closed();
                return Err(AdapterError::Denied);
            }
            return Ok(());
        }

        if header.message_type() == Type::Signal
            && interface == IBUS_INTERFACE
            && member == "GlobalEngineChanged"
            && path == IBUS_PATH
        {
            if !self
                .shared
                .bindings
                .accepts(HeaderRole::IbusLifecycle, &key)
            {
                self.fail_closed();
                return Err(AdapterError::Denied);
            }
            let name = message
                .body()
                .deserialize::<String>()
                .map_err(|_| AdapterError::Denied)?;
            let profile = classify_profile(&self.shared.lay_profiles, &name)?;
            let revoked = if let Ok(mut reducer) = self.shared.reducer.lock() {
                let before = reducer.revocation_generation();
                reducer.global_engine_changed(profile);
                reducer.revocation_generation() != before
            } else {
                true
            };
            if revoked {
                self.shared.stamps.revoke();
            }
            self.refresh_acquisition_fence_ready();
            self.shared.changed.notify(usize::MAX);
            return Ok(());
        }

        if header.message_type() == Type::Signal
            && interface == MARKER_INTERFACE
            && member == MARKER_MEMBER
            && path == MARKER_PATH
        {
            if !self
                .shared
                .bindings
                .accepts(HeaderRole::PrivateMarker, &key)
            {
                return Ok(());
            }
            let nonce = BarrierNonce(
                message
                    .body()
                    .deserialize::<u64>()
                    .map_err(|_| AdapterError::Denied)?,
            );
            return self.process_marker(nonce, message.recv_position());
        }

        if header.message_type() == Type::Signal
            && interface == DBUS_INTERFACE
            && member == "NameOwnerChanged"
            && path == DBUS_PATH
        {
            if !self
                .shared
                .bindings
                .accepts(HeaderRole::EngineCallback, &key)
            {
                self.fail_closed();
                return Err(AdapterError::Denied);
            }
            let (name, old_owner, new_owner) = message
                .body()
                .deserialize::<(String, String, String)>()
                .map_err(|_| AdapterError::Denied)?;
            if name == IBUS_NAME
                && old_owner == self.shared.bindings.ibus_owner
                && new_owner.is_empty()
            {
                self.fail_closed();
            }
        }
        Ok(())
    }

    /// Advances only bounded admission metadata in bus receive order. Engine
    /// handlers later correlate this disposition and settle effects; they do
    /// not recreate ingress order from their execution order.
    fn apply_ingress(
        &self,
        message: &Message,
        interface: &str,
        member: &str,
        path: &str,
        stamp: &IngressStamp,
    ) -> Result<IngressDisposition, AdapterError> {
        if interface == PROPERTIES_INTERFACE && member == "Set" {
            let body = message.body();
            let (target_interface, property, value) = body
                .deserialize::<(String, String, Value<'_>)>()
                .map_err(|_| AdapterError::Denied)?;
            if target_interface != ENGINE_INTERFACE || property != "ContentType" {
                return Err(AdapterError::Denied);
            }
            let value = <(u32, u32)>::try_from(value).map_err(|_| AdapterError::Denied)?;
            let target = EnginePath::new(path).ok_or(AdapterError::Denied)?;
            let unchanged = self
                .shared
                .reducer
                .lock()
                .map_err(|_| AdapterError::Denied)?
                .unchanged_content_type_token(&target, value);
            if let Some(token) = unchanged {
                return Ok(IngressDisposition::UnchangedContentType { token, value });
            }
            return self.apply_word_reset(path);
        }
        if interface == ENGINE_INTERFACE && member == "Reset" {
            message
                .body()
                .deserialize::<()>()
                .map_err(|_| AdapterError::Denied)?;
            return self.apply_word_reset(path);
        }
        let mut reducer = self
            .shared
            .reducer
            .lock()
            .map_err(|_| AdapterError::Denied)?;

        if interface == FACTORY_INTERFACE && member == "CreateEngine" {
            let requested = message
                .body()
                .deserialize::<String>()
                .map_err(|_| AdapterError::Denied)?;
            let GlobalProfile::Lay(expected_profile) =
                classify_profile(&self.shared.lay_profiles, &requested)?
            else {
                reducer.malformed_or_unobserved_lifecycle();
                return Ok(IngressDisposition::Passive);
            };
            let diagnostic = trace::enabled().then(|| {
                let state = match (
                    reducer.ticket.as_ref().map(|ticket| ticket.status),
                    reducer
                        .source_free_factory
                        .as_ref()
                        .map(|reservation| reservation.status),
                    reducer
                        .source_free
                        .as_ref()
                        .map(|activation| activation.status),
                ) {
                    (Some(TicketStatus::Pending), _, _) => "transfer_pending",
                    (Some(TicketStatus::Ready), _, _) => "transfer_ready",
                    (Some(TicketStatus::Revoked), _, _) => "transfer_revoked",
                    (_, Some(TicketStatus::Pending), _) => "source_free_factory_pending",
                    (_, Some(TicketStatus::Ready), _) => "source_free_factory_ready",
                    (_, Some(TicketStatus::Revoked), _) => "source_free_factory_revoked",
                    (_, _, Some(TicketStatus::Pending)) => "source_free_pending",
                    (_, _, Some(TicketStatus::Ready)) => "source_free_ready",
                    _ => "no_live_acquisition",
                };
                (
                    state,
                    reducer.owner().map(|owner| owner.generation.0),
                    reducer
                        .activation()
                        .map(|activation| activation.generation.0),
                )
            });
            let ticket = reducer.open_factory_request(expected_profile.clone(), stamp.position);
            drop(reducer);
            if let Some((state, owner, activation)) = diagnostic {
                trace::record_context_admission(
                    "factory_acquisition_state",
                    member,
                    stamp.header.serial,
                    state,
                    owner,
                    activation,
                    None,
                );
            }
            let Some(ticket) = ticket else {
                // A reducer-declined factory transition is lifecycle state, not
                // observer transport loss; Passive cannot authorize its callback.
                return Ok(IngressDisposition::Passive);
            };
            return Ok(IngressDisposition::Factory {
                ticket,
                expected_profile,
            });
        }

        let path_owner = reducer
            .owner()
            .filter(|owner| owner.path.as_str() == path)
            .cloned();

        if interface != ENGINE_INTERFACE {
            return Ok(IngressDisposition::Passive);
        }

        match member {
            "ProcessKeyEvent" | "ProcessKeyEventAtomicV1" => {
                if let Some(owner) = path_owner {
                    let word_effect = if member == "ProcessKeyEvent"
                        && message
                            .body()
                            .deserialize::<(u32, u32, u32)>()
                            .is_ok_and(|(keyval, _, _)| crate::protocol::is_shift_key(keyval))
                    {
                        KeyWordEffect::LegacyShiftObservation
                    } else {
                        KeyWordEffect::PossibleMutation
                    };
                    if reducer.observe_key_start(&owner, stamp.header.clone(), word_effect) {
                        Ok(IngressDisposition::Key {
                            owner,
                            revocation: reducer.revocation_generation(),
                        })
                    } else {
                        Err(AdapterError::Denied)
                    }
                } else {
                    let target = EnginePath::new(path).ok_or(AdapterError::Denied)?;
                    let pending_target = reducer.ticket.as_ref().is_some_and(|ticket| {
                        matches!(ticket.status, TicketStatus::Pending | TicketStatus::Ready)
                            && ticket.target_path.as_ref() == Some(&target)
                    });
                    if pending_target {
                        let _ = reducer.target_input_before_admission(&target);
                    }
                    Ok(IngressDisposition::Passive)
                }
            }
            "FocusOut" | "FocusOutId" => {
                let Some(owner) = path_owner else {
                    let target = EnginePath::new(path).ok_or(AdapterError::Denied)?;
                    let pending_target = reducer.request.as_ref().and_then(|request| match request
                        .target
                    {
                        RequestTarget::Transfer(_) => reducer.pending_transfer_target(),
                        RequestTarget::SourceFree => reducer
                            .source_free
                            .as_ref()
                            .map(|activation| activation.target_path.clone()),
                    });
                    if pending_target.as_ref() == Some(&target) {
                        reducer.malformed_or_unobserved_lifecycle();
                    }
                    return Ok(IngressDisposition::StaleContext);
                };
                if member == "FocusOutId" {
                    let context_path = message
                        .body()
                        .deserialize::<String>()
                        .map_err(|_| AdapterError::Denied)?;
                    if reducer
                        .activation()
                        .is_none_or(|activation| activation.context.path.as_str() != context_path)
                    {
                        return Ok(IngressDisposition::StaleContext);
                    }
                }
                match reducer.focus_out(&owner, stamp.position) {
                    Some(ticket) => Ok(IngressDisposition::FocusOut { owner, ticket }),
                    None => {
                        // A repeated/revoked handoff is a context refusal, not
                        // a broken metadata connection. Keep observing future
                        // focus while granting no transfer of the old word.
                        reducer.malformed_or_unobserved_lifecycle();
                        Ok(IngressDisposition::Revocation { owner })
                    }
                }
            }
            "Disable" => {
                let Some(owner) = path_owner else {
                    return Ok(IngressDisposition::StaleContext);
                };
                if !reducer.disable(&owner, stamp.position) {
                    reducer.malformed_or_unobserved_lifecycle();
                    return Ok(IngressDisposition::Revocation { owner });
                }
                Ok(IngressDisposition::Disable { owner })
            }
            _ => Ok(IngressDisposition::Passive),
        }
    }

    fn apply_word_reset(&self, path: &str) -> Result<IngressDisposition, AdapterError> {
        // Keep the same uninstalled witness, not a remembered current owner.
        // Other revocations cannot enter this authenticated Reset/Set route.
        let mut ready = self
            .shared
            .ready_activation
            .lock()
            .map_err(|_| AdapterError::Denied)?;
        let mut reducer = self
            .shared
            .reducer
            .lock()
            .map_err(|_| AdapterError::Denied)?;
        let target = EnginePath::new(path).ok_or(AdapterError::Denied)?;
        let pending_source_owner = (reducer.pending_transfer_target().as_ref() == Some(&target))
            .then(|| reducer.owner().cloned())
            .flatten();
        if let Some(source_owner) = pending_source_owner {
            if reducer.discard_pending_target_word(&target) {
                return Ok(IngressDisposition::Revocation {
                    owner: source_owner,
                });
            }
        }
        let Some(owner) = reducer
            .owner()
            .filter(|owner| owner.path.as_str() == path)
            .cloned()
        else {
            return Ok(IngressDisposition::StaleContext);
        };
        let retained = ready
            .as_ref()
            .filter(|current| {
                current.owner == owner
                    && current.target_path == owner.path
                    && reducer.lifecycle_is_settled()
                    && matches!(reducer.profile, GlobalProfile::Lay(_))
                    && reducer.revalidate(&current.outcome.token())
            })
            .map(|current| match &current.outcome {
                ActivationOutcome::Transfer(grant) => {
                    (grant.target_activation.clone(), grant.receipt_origin)
                }
                ActivationOutcome::SourceFree(grant) | ActivationOutcome::ResetUnknown(grant) => {
                    (grant.target_activation.clone(), grant.receipt_origin)
                }
            });
        reducer.malformed_or_unobserved_lifecycle();
        if let (Some((activation, origin)), Some(current)) = (retained, ready.as_mut()) {
            current.outcome = ActivationOutcome::ResetUnknown(ActivationGrant {
                revocation: reducer.revocation,
                target_owner: owner.clone(),
                target_activation: activation,
                lineage: reducer.lineage,
                tail_epoch: reducer.latest_tail_epoch,
                frame_generation: FrameGeneration(reducer.take_frame_generation()),
                receipt_origin: origin,
            });
        }
        Ok(IngressDisposition::Revocation { owner })
    }

    fn process_marker(&self, nonce: BarrierNonce, position: Sequence) -> Result<(), AdapterError> {
        trace::record_admission_timing("marker_ingress", None, nonce.0, None);
        let diagnostics_enabled = trace::diagnostics_enabled_cached();
        let pending = self
            .shared
            .pending
            .lock()
            .map_err(|_| AdapterError::Denied)?
            .clone();
        let Some(pending) = pending else {
            return Ok(());
        };
        if pending.nonce != nonce {
            return Ok(());
        }
        if Instant::now() > pending.deadline {
            let request = match pending.kind {
                FenceKind::Acquisition { request, .. } => Some(request),
                FenceKind::Bridge { .. } => None,
            };
            if let FenceKind::Acquisition { request, .. } = pending.kind {
                if let Ok(mut reducer) = self.shared.reducer.lock() {
                    reducer.context_acquisition_failed(request);
                }
            }
            self.clear_pending(nonce);
            if diagnostics_enabled {
                if let Ok(reducer) = self.shared.reducer.lock() {
                    let snapshot = reducer.diagnostic_snapshot();
                    drop(reducer);
                    record_admission_diagnostic(
                        if request.is_some() {
                            "compatibility"
                        } else {
                            "bridge"
                        },
                        "marker",
                        "deadline_exceeded_before_observation",
                        (request, nonce),
                        snapshot,
                        false,
                        false,
                    );
                }
            }
            return Ok(());
        }
        let bridge = matches!(&pending.kind, FenceKind::Bridge { .. });
        let mut bridge_diagnostic = None;
        let ready_token = match pending.kind {
            FenceKind::Acquisition { request, .. } => {
                let decision = self.shared.reducer.lock().ok().map(|mut reducer| {
                    let _ = reducer.marker(request, nonce, position);
                    let accepted = reducer.request.as_ref().is_some_and(|current| {
                        current.generation == request
                            && current.nonce == nonce
                            && current.marker_position.as_ref() == Some(&position)
                    });
                    let diagnostic = diagnostics_enabled.then(|| {
                        (
                            reducer.compatibility_readiness_reason(request),
                            reducer.diagnostic_snapshot(),
                        )
                    });
                    (accepted, diagnostic)
                });
                let Some((marker_accepted, diagnostic)) = decision else {
                    return Ok(());
                };
                if !marker_accepted {
                    return Ok(());
                }
                if let Ok(mut slot) = self.shared.pending.lock() {
                    if let Some(current) = slot.as_mut().filter(|current| {
                        current.nonce == nonce && current.deadline == pending.deadline
                    }) {
                        current.marker_observed = true;
                    }
                }
                self.refresh_acquisition_fence_ready();
                if let Some((reason, snapshot)) = diagnostic {
                    if reason != "ready" {
                        let post_fence_present = self.shared.pending.lock().is_ok_and(|slot| {
                            slot.as_ref().is_some_and(|current| current.nonce == nonce)
                        });
                        record_admission_diagnostic(
                            "compatibility",
                            "marker",
                            reason,
                            (Some(request), nonce),
                            snapshot,
                            post_fence_present,
                            true,
                        );
                    }
                }
                return Ok(());
            }
            FenceKind::Bridge { ping_position } => {
                if position <= ping_position {
                    self.fail_closed();
                    return Err(AdapterError::Denied);
                }
                let decision = self.shared.reducer.lock().ok().map(|reducer| {
                    let token = reducer.bridge_admission_token();
                    let diagnostic = (diagnostics_enabled && token.is_none()).then(|| {
                        (
                            reducer.bridge_readiness_reason(),
                            reducer.diagnostic_snapshot(),
                        )
                    });
                    (token, diagnostic)
                });
                match decision {
                    Some((token, diagnostic)) => {
                        bridge_diagnostic = diagnostic;
                        token
                    }
                    None => None,
                }
            }
        };
        let mut slot = self
            .shared
            .pending
            .lock()
            .map_err(|_| AdapterError::Denied)?;
        let marker_ready = if let Some(current) = slot
            .as_mut()
            .filter(|current| current.nonce == nonce && current.deadline == pending.deadline)
        {
            current.ready = true;
            current.ready_token = ready_token;
            current.marker_observed = true;
            true
        } else {
            false
        };
        drop(slot);
        self.shared.changed.notify(usize::MAX);
        if bridge && marker_ready {
            trace::record_admission_timing(
                "bridge_marker_ready",
                None,
                nonce.0,
                Some(pending.deadline),
            );
        }
        if let Some((reason, snapshot)) = bridge_diagnostic {
            record_admission_diagnostic(
                "bridge",
                "marker",
                reason,
                (None, nonce),
                snapshot,
                true,
                true,
            );
        }
        Ok(())
    }

    fn clear_pending(&self, nonce: BarrierNonce) {
        if let Ok(mut pending) = self.shared.pending.lock() {
            if pending
                .as_ref()
                .is_some_and(|pending| pending.nonce == nonce)
            {
                *pending = None;
            }
        }
        self.shared.changed.notify(usize::MAX);
    }

    fn fail_closed(&self) {
        if !self.shared.cancelled.swap(true, Ordering::AcqRel) {
            self.shared.stamps.terminate_observer();
            if let Ok(mut reducer) = self.shared.reducer.lock() {
                reducer.owner_loss_or_disconnect();
            }
            if let Ok(mut pending) = self.shared.pending.lock() {
                *pending = None;
            }
            if let Ok(mut ready) = self.shared.ready_activation.lock() {
                *ready = None;
            }
            self.shared.cancellation.notify(usize::MAX);
            self.shared.changed.notify(usize::MAX);
        }
    }
}

impl Drop for ContextAdmissionObserver {
    fn drop(&mut self) {
        self.streams.take();
        self.fail_closed();
    }
}

fn join_all(streams: Vec<MessageStream>) -> Option<JoinedMessages> {
    let mut streams = streams.into_iter();
    let first = streams.next()?;
    let mut joined: JoinedMessages = Box::pin(first);
    for stream in streams {
        joined = Box::pin(join_ordered_streams(joined, stream));
    }
    Some(joined)
}

fn signal_rule(interface: &'static str, member: &'static str) -> zbus::Result<MatchRule<'static>> {
    Ok(MatchRule::builder()
        .msg_type(Type::Signal)
        .interface(interface)?
        .member(member)?
        .build())
}

fn method_rule(interface: &'static str, member: &'static str) -> zbus::Result<MatchRule<'static>> {
    Ok(MatchRule::builder()
        .msg_type(Type::MethodCall)
        .interface(interface)?
        .member(member)?
        .build())
}

fn owner_loss_rule() -> zbus::Result<MatchRule<'static>> {
    Ok(MatchRule::builder()
        .msg_type(Type::Signal)
        .interface(DBUS_INTERFACE)?
        .member("NameOwnerChanged")?
        .arg(0, IBUS_NAME)?
        .build())
}

async fn get_name_owner(connection: &Connection, name: &str) -> Result<String, AdapterError> {
    let reply = connection
        .call_method(
            Some(DBUS_NAME),
            DBUS_PATH,
            Some(DBUS_INTERFACE),
            "GetNameOwner",
            &name,
        )
        .await?;
    reply
        .body()
        .deserialize::<String>()
        .map_err(|_| AdapterError::InvalidBootstrap("GetNameOwner was not string"))
}

fn classify_profile(
    lay_profiles: &[EngineProfile],
    name: &str,
) -> Result<GlobalProfile, AdapterError> {
    let profile = EngineProfile::new(name).ok_or(AdapterError::Denied)?;
    if lay_profiles.contains(&profile) {
        Ok(GlobalProfile::Lay(profile))
    } else {
        Ok(GlobalProfile::Foreign(profile))
    }
}

fn engine_name_from_global_value(value: &OwnedValue) -> Option<&str> {
    fn peel<'a, 'v>(mut value: &'a Value<'v>) -> &'a Value<'v> {
        while let Value::Value(inner) = value {
            value = inner;
        }
        value
    }

    let Value::Structure(structure) = peel(value) else {
        return None;
    };
    match structure.fields().first().map(peel) {
        Some(Value::Str(name)) => Some(name.as_str()),
        _ => None,
    }
}

async fn before_deadline<F, T>(future: F, deadline: Instant) -> Result<T, AdapterError>
where
    F: Future<Output = zbus::Result<T>>,
{
    let mut operation = Box::pin(future);
    let mut timer = Box::pin(async move {
        let _ = async_io::Timer::at(deadline).await;
    });
    poll_fn(|context| {
        if let Poll::Ready(result) = operation.as_mut().poll(context) {
            return Poll::Ready(result.map_err(AdapterError::Bus));
        }
        if timer.as_mut().poll(context).is_ready() {
            return Poll::Ready(Err(AdapterError::Timeout));
        }
        Poll::Pending
    })
    .await
}

#[cfg(test)]
#[path = "adapter/tests.rs"]
pub(crate) mod tests;
