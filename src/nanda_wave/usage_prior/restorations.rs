//! Explicit user preferences share the existing usage writer and hot snapshot.
//! Keep their three episode identities outside the rotating activity journal.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use super::{UsageEvent, UsageEventKind, TYPED_EVENT_SCHEMA_V3};
use crate::typing_memory::normalized_words;

pub(super) const THRESHOLD: usize = 3;
pub(super) type Episodes = BTreeMap<String, Vec<String>>;

pub(super) fn restored_word(event: &UsageEvent) -> Option<String> {
    if event.schema != Some(TYPED_EVENT_SCHEMA_V3)
        || !event.typed_v3_is_consistent()
        || event.episode_id.as_deref().is_none_or(str::is_empty)
        || !event.restoration_observed
    {
        return None;
    }
    let original = event.from.as_deref()?;
    let target = event.to.as_deref()?;
    let proposal = match event.kind {
        UsageEventKind::AcceptedFix
            if event.source.as_deref() == Some("user_correction")
                && event.outcome.as_deref() == Some("confirmed_positive")
                && original.trim() == target.trim() =>
        {
            event.proposal.as_deref()?
        }
        UsageEventKind::RejectedCandidate
            if event.source.as_deref() == Some("autocorrect")
                && event.outcome.as_deref() == Some("reverted")
                && event.operation.as_deref() == Some("replacement") =>
        {
            target
        }
        _ => return None,
    };
    let original_words = normalized_words(original);
    let proposal_words = normalized_words(proposal);
    if original_words.len() != proposal_words.len() {
        return None;
    }
    let mut changed = original_words
        .iter()
        .zip(&proposal_words)
        .filter(|(left, right)| left != right);
    let (restored, rejected) = changed.next()?;
    if changed.next().is_some()
        || restored.chars().count() > 128
        || !restored.chars().all(char::is_alphabetic)
        || event
            .word
            .as_deref()
            .map(super::normalize_memory_word)
            .as_ref()
            != Some(if event.kind == UsageEventKind::RejectedCandidate {
                rejected
            } else {
                restored
            })
    {
        return None;
    }
    Some(restored.clone())
}

pub(super) fn observe(episodes: &mut Episodes, event: &UsageEvent) -> bool {
    let Some(word) = restored_word(event) else {
        return false;
    };
    let episode = event.episode_id.as_ref().expect("validated episode");
    let votes = episodes.entry(word).or_default();
    if votes.len() >= THRESHOLD || votes.contains(episode) {
        return false;
    }
    votes.push(episode.clone());
    true
}

pub(super) fn merge(target: &mut Episodes, source: Episodes) {
    for (word, votes) in source {
        let current = target.entry(word).or_default();
        for vote in votes {
            if current.len() < THRESHOLD && !current.contains(&vote) {
                current.push(vote);
            }
        }
    }
}

fn path(events: &Path) -> PathBuf {
    events.with_file_name("user_word_restorations.json")
}

pub(super) fn load(events: &Path) -> Episodes {
    read(events).unwrap_or_default()
}

fn read(events: &Path) -> std::io::Result<Episodes> {
    let text = match std::fs::read_to_string(path(events)) {
        Ok(text) => text,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(Episodes::new()),
        Err(error) => return Err(error),
    };
    let source: Episodes = serde_json::from_str(&text)?;
    let mut result = Episodes::new();
    // Canonicalize persisted episode sets before constructing numeric hot state.
    for (word, episodes) in source {
        if word.is_empty() || word.chars().count() > 128 || !word.chars().all(char::is_alphabetic) {
            continue;
        }
        let mut unique = Vec::new();
        for episode in episodes {
            if !episode.is_empty() && !unique.contains(&episode) && unique.len() < THRESHOLD {
                unique.push(episode);
            }
        }
        result.insert(super::normalize_memory_word(&word), unique);
    }
    Ok(result)
}

/// Runs only on the existing persistence worker, bound to the queued root.
pub(super) fn persist(events: &Path, text: &str) -> std::io::Result<()> {
    let observations = text
        .lines()
        .filter_map(|line| serde_json::from_str::<UsageEvent>(line).ok())
        .filter(|event| restored_word(event).is_some())
        .collect::<Vec<_>>();
    if observations.is_empty() {
        return Ok(());
    }
    let _lock = lock_preferences(events)?;
    let mut votes = read(events)?;
    let mut changed = false;
    for event in &observations {
        changed |= observe(&mut votes, event);
    }
    if changed {
        let text = serde_json::to_vec(&votes)?;
        crate::private_file::write_private_bytes_atomic(&path(events), &text)?;
    }
    Ok(())
}

pub(super) fn lock_preferences(events: &Path) -> std::io::Result<std::fs::File> {
    crate::private_file::lock_private_file(&events.with_file_name(".user_word_restorations.lock"))
}
