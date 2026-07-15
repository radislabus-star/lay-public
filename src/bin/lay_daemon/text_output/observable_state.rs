use lay::word_buffer::WordBuffer;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::sync::atomic::{AtomicU64, Ordering};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct DaemonTextContext {
    pub(crate) field_identity: Option<String>,
    pub(crate) field_epoch: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct DaemonBufferedSuffix {
    pub(crate) context: DaemonTextContext,
    pub(crate) suffix: String,
    pub(crate) fingerprint: u64,
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct DaemonTextContextObserver<'a> {
    field_identity: Option<&'a str>,
    field_epoch: &'a AtomicU64,
}

#[derive(Debug, Clone)]
pub(crate) struct DaemonTextObservation<'a> {
    expected_context: DaemonTextContext,
    observer: DaemonTextContextObserver<'a>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum DaemonInputObservation {
    ExclusiveInputObserved,
    Unobservable { reason: &'static str },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum DaemonMutationPolicy {
    AutomaticDestructive,
    ExplicitManualUserIntent,
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct DaemonMutationLease {
    expected: DaemonBufferedSuffix,
    policy: DaemonMutationPolicy,
    consumed: bool,
}

pub(crate) struct DaemonMutationPreflight<'obs, 'buf> {
    lease: DaemonMutationLease,
    observer: DaemonTextContextObserver<'obs>,
    buffer: &'buf WordBuffer,
    input_observation: DaemonInputObservation,
}

impl DaemonTextContext {
    pub(crate) fn new(field_identity: Option<String>, field_epoch: u64) -> Self {
        Self {
            field_identity,
            field_epoch,
        }
    }
}

impl DaemonBufferedSuffix {
    pub(crate) fn new(context: DaemonTextContext, suffix: String) -> Self {
        let fingerprint = buffered_suffix_fingerprint(
            context.field_identity.as_deref(),
            context.field_epoch,
            &suffix,
        );
        Self {
            context,
            suffix,
            fingerprint,
        }
    }
}

impl<'a> DaemonTextContextObserver<'a> {
    pub(crate) fn new(field_identity: Option<&'a str>, field_epoch: &'a AtomicU64) -> Self {
        Self {
            field_identity,
            field_epoch,
        }
    }

    fn field_epoch(&self) -> u64 {
        self.field_epoch.load(Ordering::Acquire)
    }
}

impl<'a> DaemonTextObservation<'a> {
    pub(crate) fn new(
        expected_context: DaemonTextContext,
        observer: DaemonTextContextObserver<'a>,
    ) -> Self {
        Self {
            expected_context,
            observer,
        }
    }

    pub(crate) fn explicit_manual_preflight<'buf>(
        &self,
        buffer: &'buf WordBuffer,
        suffix: impl Into<String>,
        input_isolated: bool,
    ) -> DaemonMutationPreflight<'a, 'buf> {
        self.preflight(
            buffer,
            suffix,
            DaemonMutationPolicy::ExplicitManualUserIntent,
            DaemonInputObservation::for_input_isolation(input_isolated),
        )
    }

    pub(crate) fn automatic_destructive_preflight<'buf>(
        &self,
        buffer: &'buf WordBuffer,
        suffix: impl Into<String>,
        input_isolated: bool,
    ) -> DaemonMutationPreflight<'a, 'buf> {
        self.preflight(
            buffer,
            suffix,
            DaemonMutationPolicy::AutomaticDestructive,
            DaemonInputObservation::for_input_isolation(input_isolated),
        )
    }

    fn preflight<'buf>(
        &self,
        buffer: &'buf WordBuffer,
        suffix: impl Into<String>,
        policy: DaemonMutationPolicy,
        input_observation: DaemonInputObservation,
    ) -> DaemonMutationPreflight<'a, 'buf> {
        let expected = expected_buffered_suffix(self.expected_context.clone(), suffix);
        let lease = DaemonMutationLease::new(expected, policy);
        DaemonMutationPreflight::new(lease, self.observer, buffer, input_observation)
    }
}

impl DaemonInputObservation {
    pub(crate) fn for_input_isolation(input_isolated: bool) -> Self {
        if input_isolated {
            Self::ExclusiveInputObserved
        } else {
            Self::Unobservable {
                reason: "input_stream_not_exclusive",
            }
        }
    }

    fn is_exclusive_input(self) -> bool {
        matches!(self, Self::ExclusiveInputObserved)
    }

    fn name(self) -> &'static str {
        match self {
            Self::ExclusiveInputObserved => "exclusive_input_observed",
            Self::Unobservable { .. } => "unobservable",
        }
    }
}

impl DaemonMutationLease {
    pub(crate) fn new(expected: DaemonBufferedSuffix, policy: DaemonMutationPolicy) -> Self {
        Self {
            expected,
            policy,
            consumed: false,
        }
    }

    fn validate(
        &self,
        observer: DaemonTextContextObserver<'_>,
        current_suffix: &str,
        input_observation: DaemonInputObservation,
    ) -> Result<(), String> {
        if self.policy == DaemonMutationPolicy::AutomaticDestructive
            && !input_observation.is_exclusive_input()
        {
            let reason = match input_observation {
                DaemonInputObservation::Unobservable { reason } => reason,
                DaemonInputObservation::ExclusiveInputObserved => "exclusive_input_observed",
            };
            return Err(format!(
                "automatic destructive edit requires exclusive input observation, got {} ({reason})",
                input_observation.name()
            ));
        }
        if observer.field_identity != self.expected.context.field_identity.as_deref() {
            return Err("stale field identity".to_string());
        }
        if current_suffix != self.expected.suffix {
            return Err(format!(
                "stale daemon buffered suffix: expected {:?}, current {:?}",
                self.expected.suffix, current_suffix
            ));
        }

        let current_fingerprint = buffered_suffix_fingerprint(
            observer.field_identity,
            self.expected.context.field_epoch,
            current_suffix,
        );
        if current_fingerprint != self.expected.fingerprint {
            return Err(format!(
                "stale daemon buffered suffix fingerprint: expected {}, current {}",
                self.expected.fingerprint, current_fingerprint
            ));
        }

        // This load is deliberately last: input exclusivity does not prove pointer/focus stability.
        let current_epoch = observer.field_epoch();
        if current_epoch != self.expected.context.field_epoch {
            return Err(format!(
                "stale field epoch: expected {}, current {}",
                self.expected.context.field_epoch, current_epoch
            ));
        }
        Ok(())
    }
}

impl<'obs, 'buf> DaemonMutationPreflight<'obs, 'buf> {
    pub(crate) fn new(
        lease: DaemonMutationLease,
        observer: DaemonTextContextObserver<'obs>,
        buffer: &'buf WordBuffer,
        input_observation: DaemonInputObservation,
    ) -> Self {
        Self {
            lease,
            observer,
            buffer,
            input_observation,
        }
    }

    pub(crate) fn validate_current(&self) -> Result<(), String> {
        if self.lease.consumed {
            return Err("mutation lease already consumed".to_string());
        }
        self.validate_observed_state()
    }

    pub(crate) fn consume(&mut self) -> Result<(), String> {
        if self.lease.consumed {
            return Err("mutation lease already consumed".to_string());
        }
        self.lease.consumed = true;
        self.validate_observed_state()
    }

    fn validate_observed_state(&self) -> Result<(), String> {
        let Some(visible_tail) = self
            .buffer
            .visible_tail_text(lay::word_buffer::MAX_REPLACE_WORDS)
        else {
            return Err("no daemon buffered suffix available for mutation preflight".to_string());
        };
        let current_suffix = tail_chars(&visible_tail, self.lease.expected.suffix.chars().count());
        self.lease
            .validate(self.observer, &current_suffix, self.input_observation)
    }
}

pub(crate) fn expected_buffered_suffix(
    context: DaemonTextContext,
    suffix: impl Into<String>,
) -> DaemonBufferedSuffix {
    DaemonBufferedSuffix::new(context, suffix.into())
}

fn buffered_suffix_fingerprint(
    field_identity: Option<&str>,
    field_epoch: u64,
    suffix: &str,
) -> u64 {
    let mut hasher = DefaultHasher::new();
    field_identity.hash(&mut hasher);
    field_epoch.hash(&mut hasher);
    suffix.hash(&mut hasher);
    hasher.finish()
}

fn tail_chars(text: &str, char_count: usize) -> String {
    let mut chars = text.chars().rev().take(char_count).collect::<Vec<_>>();
    chars.reverse();
    chars.into_iter().collect()
}
