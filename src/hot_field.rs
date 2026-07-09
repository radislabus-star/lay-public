//! Hot runtime field contract.
//!
//! The hot input path must read compact field state and tiny renderer metadata.
//! Full dictionaries, generated forms and full NANDA traces are cold/reference
//! authority. They may build or verify the field, but they must not become the
//! live daemon's default memory object.

use crate::text_backend::TextBackendPreference;
use std::sync::atomic::{AtomicU8, Ordering};

const ROUTE_DAEMON: u8 = 0;
const ROUTE_IME: u8 = 1;
const AUTHORITY_FIELD_SNAPSHOT_ONLY: u8 = 0;
const AUTHORITY_FULL_REFERENCE_ALLOWED: u8 = 1;

static PROCESS_ROUTE: AtomicU8 = AtomicU8::new(ROUTE_DAEMON);
static PROCESS_AUTHORITY: AtomicU8 = AtomicU8::new(AUTHORITY_FULL_REFERENCE_ALLOWED);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HotRuntimeRoute {
    Daemon,
    Ime,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HotAuthority {
    FieldSnapshotOnly,
    FullReferenceAllowed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HotFieldPolicy {
    route: HotRuntimeRoute,
    authority: HotAuthority,
}

impl HotFieldPolicy {
    pub fn daemon_for_text_backend(backend: TextBackendPreference) -> Self {
        let authority = if backend.should_try_ime() {
            HotAuthority::FieldSnapshotOnly
        } else {
            HotAuthority::FullReferenceAllowed
        };
        Self {
            route: HotRuntimeRoute::Daemon,
            authority,
        }
    }

    pub const fn ime() -> Self {
        Self {
            route: HotRuntimeRoute::Ime,
            authority: HotAuthority::FieldSnapshotOnly,
        }
    }

    pub const fn route(self) -> HotRuntimeRoute {
        self.route
    }

    pub const fn authority(self) -> HotAuthority {
        self.authority
    }

    pub const fn allows_full_reference_authority(self) -> bool {
        matches!(self.authority, HotAuthority::FullReferenceAllowed)
    }

    pub const fn allows_full_nanda_authority(self) -> bool {
        self.allows_full_reference_authority()
    }
}

pub fn set_process_policy(policy: HotFieldPolicy) {
    PROCESS_ROUTE.store(encode_route(policy.route), Ordering::Relaxed);
    PROCESS_AUTHORITY.store(encode_authority(policy.authority), Ordering::Relaxed);
}

pub fn process_policy() -> HotFieldPolicy {
    HotFieldPolicy {
        route: decode_route(PROCESS_ROUTE.load(Ordering::Relaxed)),
        authority: decode_authority(PROCESS_AUTHORITY.load(Ordering::Relaxed)),
    }
}

pub fn process_allows_full_reference_authority() -> bool {
    process_policy().allows_full_reference_authority()
}

const fn encode_route(route: HotRuntimeRoute) -> u8 {
    match route {
        HotRuntimeRoute::Daemon => ROUTE_DAEMON,
        HotRuntimeRoute::Ime => ROUTE_IME,
    }
}

const fn decode_route(value: u8) -> HotRuntimeRoute {
    match value {
        ROUTE_IME => HotRuntimeRoute::Ime,
        _ => HotRuntimeRoute::Daemon,
    }
}

const fn encode_authority(authority: HotAuthority) -> u8 {
    match authority {
        HotAuthority::FieldSnapshotOnly => AUTHORITY_FIELD_SNAPSHOT_ONLY,
        HotAuthority::FullReferenceAllowed => AUTHORITY_FULL_REFERENCE_ALLOWED,
    }
}

const fn decode_authority(value: u8) -> HotAuthority {
    match value {
        AUTHORITY_FULL_REFERENCE_ALLOWED => HotAuthority::FullReferenceAllowed,
        _ => HotAuthority::FieldSnapshotOnly,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HotWordAuthority {
    Unknown,
    CommonSurface,
    UserUsage,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HotWordReadout {
    pub authority: HotWordAuthority,
}

impl HotWordReadout {
    pub const fn is_known(self) -> bool {
        !matches!(self.authority, HotWordAuthority::Unknown)
    }
}

#[derive(Debug, Default)]
pub struct HotFieldSnapshot;

impl HotFieldSnapshot {
    pub const fn current() -> Self {
        Self
    }

    pub fn word_readout(&self, word: &str) -> HotWordReadout {
        let lower = word.trim().to_lowercase();
        let authority = if lower.is_empty() {
            HotWordAuthority::Unknown
        } else if crate::lexicon::is_common_ru_word(&lower)
            || crate::lexicon::is_ime_hot_ru_word(&lower)
        {
            HotWordAuthority::CommonSurface
        } else if crate::nanda_wave::cached_usage_prior_snapshot().word_prior(&lower) >= 0.020 {
            HotWordAuthority::UserUsage
        } else {
            HotWordAuthority::Unknown
        };
        HotWordReadout { authority }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn daemon_auto_backend_uses_field_snapshot_only() {
        let policy = HotFieldPolicy::daemon_for_text_backend(TextBackendPreference::Auto);
        assert_eq!(policy.route(), HotRuntimeRoute::Daemon);
        assert_eq!(policy.authority(), HotAuthority::FieldSnapshotOnly);
        assert!(!policy.allows_full_nanda_authority());
    }

    #[test]
    fn daemon_uinput_backend_can_use_full_reference_authority() {
        let policy = HotFieldPolicy::daemon_for_text_backend(TextBackendPreference::Uinput);
        assert_eq!(policy.authority(), HotAuthority::FullReferenceAllowed);
        assert!(policy.allows_full_nanda_authority());
    }

    #[test]
    fn process_policy_tracks_hot_authority() {
        let original = process_policy();
        set_process_policy(HotFieldPolicy::ime());

        assert_eq!(process_policy().route(), HotRuntimeRoute::Ime);
        assert_eq!(
            process_policy().authority(),
            HotAuthority::FieldSnapshotOnly
        );
        assert!(!process_allows_full_reference_authority());

        set_process_policy(original);
    }

    #[test]
    fn hot_word_readout_does_not_need_full_dictionary_for_common_words() {
        let snapshot = HotFieldSnapshot::current();
        assert!(snapshot.word_readout("это").is_known());
    }
}
