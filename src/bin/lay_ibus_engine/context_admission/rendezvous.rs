use std::collections::VecDeque;
use std::future::{poll_fn, Future};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Mutex;
use std::task::Poll;
use std::time::{Duration, Instant};

use event_listener::{Event, EventListener};

use super::{ConnectionGeneration, HeaderKey, IngressStamp, OwnerGeneration};

pub(crate) const CALLBACK_RENDEZVOUS_BUDGET: Duration = Duration::from_micros(1_000);
const DEFAULT_STAMP_CAPACITY: usize = 128;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct RendezvousGuard {
    pub(crate) connection: ConnectionGeneration,
    pub(crate) owner: OwnerGeneration,
    pub(crate) revocation: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RendezvousFailure {
    Revoked,
    Evicted,
    ObserverTerminated,
    Poisoned,
    Timeout,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[expect(
    clippy::large_enum_variant,
    reason = "Keep callback stamps inline; the production outcome has a tested 224-byte budget"
)]
pub(crate) enum RendezvousOutcome<P> {
    Stamp(IngressStamp<P>),
    Failed(RendezvousFailure),
}

#[derive(Debug)]
struct StampState<P> {
    connection: ConnectionGeneration,
    owner: OwnerGeneration,
    revocation: u64,
    eviction: u64,
    terminated: bool,
    stamps: VecDeque<IngressStamp<P>>,
}

#[derive(Debug)]
pub(crate) struct CallbackStampStore<P = zbus::message::Sequence> {
    capacity: usize,
    state: Mutex<StampState<P>>,
    changed: Event,
    waiters: AtomicUsize,
}

impl<P> CallbackStampStore<P>
where
    P: Clone,
{
    pub(crate) fn new(connection: ConnectionGeneration, owner: OwnerGeneration) -> Self {
        Self::with_capacity(connection, owner, DEFAULT_STAMP_CAPACITY)
    }

    pub(crate) fn with_capacity(
        connection: ConnectionGeneration,
        owner: OwnerGeneration,
        capacity: usize,
    ) -> Self {
        assert!(capacity > 0, "callback stamp capacity must be positive");
        Self {
            capacity,
            state: Mutex::new(StampState {
                connection,
                owner,
                revocation: 1,
                eviction: 0,
                terminated: false,
                stamps: VecDeque::with_capacity(capacity),
            }),
            changed: Event::new(),
            waiters: AtomicUsize::new(0),
        }
    }

    /// Observer hot path: one short mutex, then notification after unlock.
    pub(crate) fn publish(&self, stamp: IngressStamp<P>) -> bool {
        let (accepted, notify) = {
            let Ok(mut state) = self.state.lock() else {
                self.changed.notify(usize::MAX);
                return false;
            };
            if state.terminated {
                return false;
            }
            if stamp.header.connection != state.connection
                || state
                    .stamps
                    .iter()
                    .any(|existing| existing.header == stamp.header)
            {
                state.revocation = next_generation(state.revocation);
                state.stamps.clear();
                (false, true)
            } else {
                if state.stamps.len() == self.capacity {
                    state.stamps.pop_front();
                    state.eviction = next_generation(state.eviction);
                }
                state.stamps.push_back(stamp);
                (true, true)
            }
        };
        if notify {
            self.changed.notify(usize::MAX);
        }
        accepted
    }

    pub(crate) fn guard(&self) -> Option<RendezvousGuard> {
        let state = self.state.lock().ok()?;
        Some(RendezvousGuard {
            connection: state.connection,
            owner: state.owner,
            revocation: state.revocation,
        })
    }

    pub(crate) fn replace_owner(&self, owner: OwnerGeneration) {
        if let Ok(mut state) = self.state.lock() {
            state.owner = owner;
            state.revocation = next_generation(state.revocation);
            // Retain bounded typed stamps so a handler delayed across owner
            // replacement can identify its old ingress owner and no-op. Any
            // already-waiting rendezvous still fails through the new epoch.
        }
        self.changed.notify(usize::MAX);
    }

    pub(crate) fn revoke(&self) {
        if let Ok(mut state) = self.state.lock() {
            state.revocation = next_generation(state.revocation);
            state.stamps.clear();
        }
        self.changed.notify(usize::MAX);
    }

    pub(crate) fn terminate_observer(&self) {
        if let Ok(mut state) = self.state.lock() {
            state.terminated = true;
            state.revocation = next_generation(state.revocation);
            state.stamps.clear();
        }
        self.changed.notify(usize::MAX);
    }

    pub(crate) fn begin<'a>(
        &'a self,
        key: HeaderKey,
        guard: RendezvousGuard,
    ) -> RendezvousBegin<'a, P> {
        match self.lookup(&key, guard, None) {
            Lookup::Stamp(stamp) => RendezvousBegin::Ready(RendezvousOutcome::Stamp(stamp)),
            Lookup::Failed(failure) => RendezvousBegin::Ready(RendezvousOutcome::Failed(failure)),
            Lookup::Pending(eviction) => RendezvousBegin::Pending(PendingRendezvous {
                store: self,
                key,
                guard,
                eviction,
                listener: None,
                counted: false,
            }),
        }
    }

    #[cfg(test)]
    pub(crate) fn waiter_count(&self) -> usize {
        self.waiters.load(Ordering::Acquire)
    }

    fn lookup(
        &self,
        key: &HeaderKey,
        guard: RendezvousGuard,
        expected_eviction: Option<u64>,
    ) -> Lookup<P> {
        let Ok(state) = self.state.lock() else {
            return Lookup::Failed(RendezvousFailure::Poisoned);
        };
        if state.terminated {
            return Lookup::Failed(RendezvousFailure::ObserverTerminated);
        }
        if state.connection != guard.connection
            || state.owner != guard.owner
            || state.revocation != guard.revocation
        {
            return Lookup::Failed(RendezvousFailure::Revoked);
        }
        if let Some(stamp) = state.stamps.iter().find(|stamp| stamp.header == *key) {
            return Lookup::Stamp(stamp.clone());
        }
        // Publishing this exact stamp may evict an unrelated older entry.
        // Only a missing stamp is ambiguous after eviction; the epoch checks
        // above still revoke retained stamps across ownership changes.
        if expected_eviction.is_some_and(|expected| state.eviction != expected) {
            return Lookup::Failed(RendezvousFailure::Evicted);
        }
        Lookup::Pending(state.eviction)
    }
}

enum Lookup<P> {
    Stamp(IngressStamp<P>),
    Failed(RendezvousFailure),
    Pending(u64),
}

pub(crate) enum RendezvousBegin<'a, P>
where
    P: Clone,
{
    Ready(RendezvousOutcome<P>),
    Pending(PendingRendezvous<'a, P>),
}

pub(crate) struct PendingRendezvous<'a, P>
where
    P: Clone,
{
    store: &'a CallbackStampStore<P>,
    key: HeaderKey,
    guard: RendezvousGuard,
    eviction: u64,
    listener: Option<EventListener>,
    counted: bool,
}

impl<P> PendingRendezvous<'_, P>
where
    P: Clone,
{
    /// Must precede the second lookup.
    pub(crate) fn listen(&mut self) {
        if self.listener.is_none() {
            self.listener = Some(self.store.changed.listen());
            if !self.counted {
                self.store.waiters.fetch_add(1, Ordering::AcqRel);
                self.counted = true;
            }
        }
    }

    pub(crate) fn recheck(&self) -> Option<RendezvousOutcome<P>> {
        match self
            .store
            .lookup(&self.key, self.guard, Some(self.eviction))
        {
            Lookup::Stamp(stamp) => Some(RendezvousOutcome::Stamp(stamp)),
            Lookup::Failed(failure) => Some(RendezvousOutcome::Failed(failure)),
            Lookup::Pending(_) => None,
        }
    }

    /// The deadline future is injected for deterministic tests. It represents
    /// one absolute deadline and is never recreated after unrelated wakes.
    pub(crate) async fn wait_until<F>(&mut self, deadline: F) -> RendezvousOutcome<P>
    where
        F: Future<Output = ()>,
    {
        self.listen();
        if let Some(outcome) = self.recheck() {
            return outcome;
        }

        let mut deadline = Box::pin(deadline);
        loop {
            let mut listener = Box::pin(
                self.listener
                    .take()
                    .unwrap_or_else(|| self.store.changed.listen()),
            );
            let deadline_won = poll_fn(|cx| {
                if deadline.as_mut().poll(cx).is_ready() {
                    return Poll::Ready(true);
                }
                if listener.as_mut().poll(cx).is_ready() {
                    return Poll::Ready(false);
                }
                Poll::Pending
            })
            .await;

            if deadline_won {
                return self
                    .recheck()
                    .unwrap_or(RendezvousOutcome::Failed(RendezvousFailure::Timeout));
            }

            // Listen before rechecking again, including after unrelated wakes.
            self.listener = Some(self.store.changed.listen());
            if let Some(outcome) = self.recheck() {
                return outcome;
            }
        }
    }
}

impl<P> Drop for PendingRendezvous<'_, P>
where
    P: Clone,
{
    fn drop(&mut self) {
        if self.counted {
            self.store.waiters.fetch_sub(1, Ordering::AcqRel);
        }
    }
}

pub(crate) fn callback_deadline(callback_entered: Instant) -> Instant {
    callback_entered
        .checked_add(CALLBACK_RENDEZVOUS_BUDGET)
        .unwrap_or(callback_entered)
}

pub(crate) async fn rendezvous_stamp<P>(
    store: &CallbackStampStore<P>,
    key: HeaderKey,
    guard: RendezvousGuard,
    callback_entered: Instant,
) -> RendezvousOutcome<P>
where
    P: Clone,
{
    match store.begin(key, guard) {
        RendezvousBegin::Ready(outcome) => outcome,
        RendezvousBegin::Pending(mut pending) => {
            let deadline = callback_deadline(callback_entered);
            pending
                .wait_until(async move {
                    let _ = async_io::Timer::at(deadline).await;
                })
                .await
        }
    }
}

fn next_generation(current: u64) -> u64 {
    let next = current.wrapping_add(1);
    if next == 0 {
        1
    } else {
        next
    }
}
