//! Compact learned context relation memory for L3.
//!
//! Cold text is compiled into token-state centers and candidate-specific
//! context centers. The hot package stores hashes, quantized phase vectors,
//! support and learned thresholds; it stores no raw phrase or word strings.

mod compiler;
mod format;

use std::env;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::OnceLock;

use super::phase_field::{
    add_hashed_atom, add_rotated_vector, empty_vector, hash_text, max_coherence,
    phase_center_from_sum, phase_micro, PhaseCell, PhaseCenter,
};

pub(crate) use compiler::{compile_context_phase, prove_context_phase, ContextPhaseCompileInput};
pub(crate) use format::{read_package, write_package};

pub(crate) const MAGIC: &[u8; 8] = b"LAYL3P01";
pub(crate) const CELLS: usize = 64;
pub(crate) const MAX_CONTEXT_TOKENS: usize = 16;

#[derive(Clone, Debug)]
pub(crate) struct TokenSemanticState {
    pub(crate) token_hash: u64,
    pub(crate) support: u32,
    pub(crate) center: Vec<PhaseCell>,
}

#[derive(Clone, Debug)]
pub(crate) struct ContextCandidateProfile {
    pub(crate) token_hash: u64,
    pub(crate) positive_examples: u32,
    pub(crate) negative_examples: u32,
    pub(crate) threshold_micro: i32,
    pub(crate) positive: Vec<PhaseCenter>,
    pub(crate) negative: Vec<PhaseCenter>,
}

#[derive(Clone, Debug, Default)]
pub(crate) struct ContextPhasePackage {
    pub(crate) semantic_states: Vec<TokenSemanticState>,
    pub(crate) profiles: Vec<ContextCandidateProfile>,
    pub(crate) transitions: u64,
    pub(crate) corpus_fragments: u32,
    pub(crate) global_threshold_micro: i32,
    pub(crate) competition_threshold_micro: i32,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) enum ContextPhaseDisposition {
    Support,
    Suppress,
    Neutral,
    #[default]
    Unavailable,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct ContextPhaseReadout {
    pub(crate) package_loaded: bool,
    pub(crate) profile_present: bool,
    pub(crate) disposition: ContextPhaseDisposition,
    pub(crate) positive_micro: i64,
    pub(crate) anti_micro: i64,
    pub(crate) margin_micro: i64,
    pub(crate) threshold_micro: i64,
    pub(crate) competition_margin_micro: i64,
    pub(crate) positive_examples: u32,
    pub(crate) negative_examples: u32,
    pub(crate) positive_centers: u8,
    pub(crate) anti_centers: u8,
    pub(crate) semantic_support: u32,
    pub(crate) relation_class: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ContextPhaseMode {
    Full,
    NoPhase,
    NoAnti,
    NoSemanticState,
}

impl ContextPhasePackage {
    pub(crate) fn is_empty(&self) -> bool {
        self.profiles.is_empty()
    }

    pub(crate) fn score_candidates(
        &self,
        context_tokens: &[String],
        candidates: &[&str],
    ) -> Vec<ContextPhaseReadout> {
        self.score_candidates_with_mode(context_tokens, candidates, ContextPhaseMode::Full)
    }

    pub(crate) fn score_candidates_with_mode(
        &self,
        context_tokens: &[String],
        candidates: &[&str],
        mode: ContextPhaseMode,
    ) -> Vec<ContextPhaseReadout> {
        if self.is_empty() || context_tokens.is_empty() {
            return vec![ContextPhaseReadout::default(); candidates.len()];
        }
        let mut readouts = candidates
            .iter()
            .map(|candidate| {
                let vector = self.candidate_relation_vector(context_tokens, candidate, mode);
                self.raw_readout(&vector, candidate, mode)
            })
            .collect::<Vec<_>>();

        let mut ranked = readouts
            .iter()
            .enumerate()
            .filter(|(_, readout)| readout.profile_present)
            .map(|(index, readout)| (index, readout.margin_micro))
            .collect::<Vec<_>>();
        ranked.sort_by_key(|item| std::cmp::Reverse(item.1));
        let best = ranked.first().copied();
        let runner_up = ranked
            .get(1)
            .map(|(_, score)| *score)
            .unwrap_or(i64::MIN / 2);
        let competition_margin = best
            .map(|(_, score)| score.saturating_sub(runner_up))
            .unwrap_or_default();
        let competition_ready = ranked.len() == 1
            || competition_margin >= i64::from(self.competition_threshold_micro.max(1));

        for (index, readout) in readouts.iter_mut().enumerate() {
            if !readout.profile_present || mode == ContextPhaseMode::NoPhase {
                continue;
            }
            readout.competition_margin_micro = if best.is_some_and(|(best, _)| best == index) {
                competition_margin
            } else {
                best.map(|(_, score)| readout.margin_micro.saturating_sub(score))
                    .unwrap_or_default()
            };
            let above_threshold = readout.margin_micro >= readout.threshold_micro;
            let has_support = readout.positive_examples >= 2;
            readout.disposition = if best.is_some_and(|(best, _)| best == index)
                && competition_ready
                && above_threshold
                && has_support
            {
                ContextPhaseDisposition::Support
            } else if readout.anti_micro > readout.positive_micro
                || best.is_some_and(|(_, score)| {
                    score.saturating_sub(readout.margin_micro)
                        >= i64::from(self.competition_threshold_micro.max(1))
                })
            {
                ContextPhaseDisposition::Suppress
            } else {
                ContextPhaseDisposition::Neutral
            };
        }
        readouts
    }

    fn raw_readout(
        &self,
        vector: &[PhaseCell],
        candidate: &str,
        mode: ContextPhaseMode,
    ) -> ContextPhaseReadout {
        let token = crate::word_reader::last_text_word(candidate).unwrap_or_default();
        let token_hash = hash_text(&token.to_lowercase());
        let Some(profile) = self.profile(token_hash) else {
            return ContextPhaseReadout {
                package_loaded: true,
                ..ContextPhaseReadout::default()
            };
        };
        if mode == ContextPhaseMode::NoPhase {
            return ContextPhaseReadout {
                package_loaded: true,
                profile_present: true,
                threshold_micro: i64::from(profile.threshold_micro),
                positive_examples: profile.positive_examples,
                negative_examples: profile.negative_examples,
                positive_centers: profile.positive.len().min(u8::MAX as usize) as u8,
                anti_centers: profile.negative.len().min(u8::MAX as usize) as u8,
                semantic_support: self
                    .semantic_state(token_hash)
                    .map(|state| state.support)
                    .unwrap_or_default(),
                relation_class: relation_class(token_hash, 0),
                ..ContextPhaseReadout::default()
            };
        }
        let positive = max_coherence(vector, &profile.positive).unwrap_or_default();
        let anti = if mode == ContextPhaseMode::NoAnti {
            0.0
        } else {
            max_coherence(vector, &profile.negative).unwrap_or_default()
        };
        let margin = positive - anti;
        let margin_micro = phase_micro(margin);
        ContextPhaseReadout {
            package_loaded: true,
            profile_present: true,
            disposition: ContextPhaseDisposition::Neutral,
            positive_micro: phase_micro(positive),
            anti_micro: phase_micro(anti),
            margin_micro,
            threshold_micro: i64::from(profile.threshold_micro.max(self.global_threshold_micro)),
            competition_margin_micro: 0,
            positive_examples: profile.positive_examples,
            negative_examples: profile.negative_examples,
            positive_centers: profile.positive.len().min(u8::MAX as usize) as u8,
            anti_centers: profile.negative.len().min(u8::MAX as usize) as u8,
            semantic_support: self
                .semantic_state(token_hash)
                .map(|state| state.support)
                .unwrap_or_default(),
            relation_class: relation_class(token_hash, margin_micro),
        }
    }

    pub(crate) fn context_vector(
        &self,
        context_tokens: &[String],
        mode: ContextPhaseMode,
    ) -> Vec<PhaseCell> {
        let mut vector = empty_vector(CELLS);
        let start = context_tokens.len().saturating_sub(MAX_CONTEXT_TOKENS);
        for (offset, token) in context_tokens[start..].iter().rev().enumerate() {
            let token_hash = hash_text(token);
            let position = offset as u64 + 1;
            let recency = 1.0 / (position as f32).sqrt();
            add_hashed_atom(
                &mut vector,
                token_hash ^ 0x0043_4f4e_5445_5854,
                position ^ token_hash.rotate_left(13),
                recency,
            );
            if mode != ContextPhaseMode::NoSemanticState {
                if let Some(state) = self.semantic_state(token_hash) {
                    add_rotated_vector(
                        &mut vector,
                        &state.center,
                        position ^ 0x0053_454d_414e_5449,
                        recency * 0.80,
                    );
                }
            }
        }
        phase_center_from_sum(&vector)
    }

    pub(super) fn candidate_relation_vector(
        &self,
        context_tokens: &[String],
        candidate: &str,
        mode: ContextPhaseMode,
    ) -> Vec<PhaseCell> {
        let mut vector = self.context_vector(context_tokens, mode);
        if mode != ContextPhaseMode::NoSemanticState {
            let token = crate::word_reader::last_text_word(candidate).unwrap_or_default();
            let token_hash = hash_text(&token.to_lowercase());
            if let Some(state) = self.semantic_state(token_hash) {
                add_rotated_vector(
                    &mut vector,
                    &state.center,
                    token_hash ^ 0x0052_454c_4154_494f,
                    0.85,
                );
            }
        }
        phase_center_from_sum(&vector)
    }

    fn semantic_state(&self, token_hash: u64) -> Option<&TokenSemanticState> {
        self.semantic_states
            .binary_search_by_key(&token_hash, |state| state.token_hash)
            .ok()
            .and_then(|index| self.semantic_states.get(index))
    }

    fn profile(&self, token_hash: u64) -> Option<&ContextCandidateProfile> {
        self.profiles
            .binary_search_by_key(&token_hash, |profile| profile.token_hash)
            .ok()
            .and_then(|index| self.profiles.get(index))
    }
}

fn relation_class(token_hash: u64, margin_micro: i64) -> u64 {
    let band = ((margin_micro / 25_000).clamp(-32, 32) + 32) as u64;
    crate::stable_hash::mix64_golden(token_hash ^ band.rotate_left(19))
}

pub(crate) fn default_memory_path() -> PathBuf {
    env::var_os("LAY_NANDA_L3_CONTEXT_MEMORY")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            env::var_os("HOME")
                .map(PathBuf::from)
                .unwrap_or_else(|| PathBuf::from("."))
                .join(".local/share/lay/nanda_wave/l3_context_phase.nwpc")
        })
}

static DEFAULT_MEMORY: OnceLock<ContextPhasePackage> = OnceLock::new();
static DEFAULT_MEMORY_WARM: AtomicBool = AtomicBool::new(false);

pub(crate) fn warm_default_memory() {
    let _ = default_memory();
}

pub(crate) fn default_memory_is_warm() -> bool {
    DEFAULT_MEMORY_WARM.load(Ordering::Acquire)
}

pub(crate) fn default_memory() -> &'static ContextPhasePackage {
    DEFAULT_MEMORY.get_or_init(|| {
        let memory = read_package(&default_memory_path()).unwrap_or_default();
        DEFAULT_MEMORY_WARM.store(true, Ordering::Release);
        memory
    })
}

pub(crate) fn readout_default_candidates(
    original: &str,
    replacements: &[&str],
) -> Vec<ContextPhaseReadout> {
    let mut context = super::llmwave::tokenize(original);
    context.pop();
    default_memory().score_candidates(&context, replacements)
}

pub(crate) fn package_report(path: &Path) -> serde_json::Value {
    match read_package(path) {
        Ok(package) => serde_json::json!({
            "kind": "l3_context_phase_package",
            "path": path,
            "loaded": true,
            "raw_words_stored": false,
            "cells": CELLS,
            "semantic_states": package.semantic_states.len(),
            "candidate_profiles": package.profiles.len(),
            "positive_centers": package.profiles.iter().map(|profile| profile.positive.len()).sum::<usize>(),
            "anti_centers": package.profiles.iter().map(|profile| profile.negative.len()).sum::<usize>(),
            "transitions": package.transitions,
            "corpus_fragments": package.corpus_fragments,
            "global_threshold_micro": package.global_threshold_micro,
            "competition_threshold_micro": package.competition_threshold_micro,
            "bytes": std::fs::metadata(path).map(|meta| meta.len()).unwrap_or_default(),
        }),
        Err(error) => serde_json::json!({
            "kind": "l3_context_phase_package",
            "path": path,
            "loaded": false,
            "error": error.to_string(),
        }),
    }
}

#[cfg(test)]
mod tests;
