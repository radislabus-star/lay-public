use std::cmp::Ordering;
use std::collections::{BinaryHeap, HashMap};
use std::fs::File;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use crate::lexical_surface_atoms::SurfaceFieldEncoder;
use crate::text_metrics::damerau_levenshtein;

use super::format::{
    atom_center_keys, normalize_surface, phase_coherence_milli, read_arc, read_center,
    read_decoder_arc, read_decoder_state, read_header, read_node, read_posting, read_terminal,
    surface_phase, ArtifactHeader, CenterRecord, NO_INDEX,
};

const MAX_PHASE_FRONTIER: usize = 768;
const MAX_POSTINGS_PER_QUERY_CENTER: usize = 512;
const MAX_COMPLETION_FRONTIER: usize = 4_096;
const MAX_DECODE_EDITS: u8 = 3;
const MAX_DAFSA_VISITS: usize = 300_000;
const RECONSTRUCTION_LANE_RESERVE: usize = 4;
const PREFIX_COMPOSITION_FRONTIER_PER_PREFIX: usize = 16;

static DEFAULT_MEMORY: OnceLock<Option<LexicalPhaseMemory>> = OnceLock::new();

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct LexicalPhaseStats {
    pub(crate) source_words: usize,
    pub(crate) l1_centers: usize,
    pub(crate) l1_postings: usize,
    pub(crate) l2_word_centers: usize,
    pub(crate) grapheme_nodes: usize,
    pub(crate) grapheme_arcs: usize,
    pub(crate) decoder_states: usize,
    pub(crate) decoder_arcs: usize,
    pub(crate) training_surfaces: usize,
    pub(crate) hot_bytes: usize,
    pub(crate) mmap_backed: bool,
    pub(crate) raw_word_table: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct LexicalPhaseCandidate {
    pub(crate) word: String,
    pub(crate) score: u32,
    pub(crate) l1_overlap: usize,
    pub(crate) l2_overlap: usize,
    pub(crate) motif_overlap: usize,
    pub(crate) prefix_match: bool,
    pub(crate) rank: usize,
    pub(crate) phase_coherence_milli: u16,
    pub(crate) reconstructed: bool,
}

pub(crate) struct LexicalPhaseMemory {
    bytes: ArtifactBytes,
    header: ArtifactHeader,
    path: PathBuf,
}

impl std::fmt::Debug for LexicalPhaseMemory {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("LexicalPhaseMemory")
            .field("path", &self.path)
            .field("stats", &self.stats())
            .finish()
    }
}

pub(crate) fn default_memory() -> Option<&'static LexicalPhaseMemory> {
    DEFAULT_MEMORY
        .get_or_init(|| {
            let mut last_error = None;
            for path in default_artifact_candidates() {
                match LexicalPhaseMemory::load(&path) {
                    Ok(memory) => {
                        memory.bytes.prefetch_all();
                        return Some(memory);
                    }
                    Err(error) => last_error = Some((path, error)),
                }
            }
            if std::env::var_os("LAY_NANDA_L2_TIMING").is_some() {
                if let Some((path, error)) = last_error {
                    eprintln!(
                        "lay_lexical_phase load_failed path={} error={error}",
                        path.display()
                    );
                }
            }
            None
        })
        .as_ref()
}

#[cfg(test)]
fn default_artifact_candidates() -> [PathBuf; 2] {
    [repository_artifact_path(), default_artifact_path()]
}

#[cfg(not(test))]
fn default_artifact_candidates() -> [PathBuf; 2] {
    [default_artifact_path(), repository_artifact_path()]
}

pub(crate) fn default_artifact_path() -> PathBuf {
    if let Some(path) = std::env::var_os("LAY_L2_LEXICAL_PHASE_MEMORY") {
        return PathBuf::from(path);
    }
    if let Some(data) = std::env::var_os("XDG_DATA_HOME") {
        return PathBuf::from(data)
            .join("lay")
            .join("nanda_wave")
            .join("l2_lexical_phase_v2.bin");
    }
    let home = std::env::var_os("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."));
    home.join(".local")
        .join("share")
        .join("lay")
        .join("nanda_wave")
        .join("l2_lexical_phase_v2.bin")
}

fn repository_artifact_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("data")
        .join("lexicon")
        .join("l2_lexical_phase_v2.bin")
}

impl LexicalPhaseMemory {
    pub(crate) fn load(path: &Path) -> Result<Self, String> {
        let bytes = ArtifactBytes::load(path)?;
        let header = read_header(bytes.as_slice())?;
        Ok(Self {
            bytes,
            header,
            path: path.to_path_buf(),
        })
    }

    pub(crate) fn from_bytes(bytes: Vec<u8>) -> Result<Self, String> {
        let bytes = ArtifactBytes::Owned(bytes.into_boxed_slice());
        let header = read_header(bytes.as_slice())?;
        Ok(Self {
            bytes,
            header,
            path: PathBuf::from("<memory>"),
        })
    }

    pub(crate) fn stats(&self) -> LexicalPhaseStats {
        LexicalPhaseStats {
            source_words: self.header.source_words as usize,
            l1_centers: self.header.center_count as usize,
            l1_postings: self.header.posting_count as usize,
            l2_word_centers: self.header.terminal_count as usize,
            grapheme_nodes: self.header.node_count as usize,
            grapheme_arcs: self.header.arc_count as usize,
            decoder_states: self.header.decoder_state_count as usize,
            decoder_arcs: self.header.decoder_arc_count as usize,
            training_surfaces: self.header.training_surfaces as usize,
            hot_bytes: self.bytes.as_slice().len(),
            mmap_backed: self.bytes.is_mapped(),
            raw_word_table: false,
        }
    }

    pub(crate) fn contains_surface(&self, surface: &str) -> bool {
        self.terminal_for_surface(surface).is_some()
    }

    pub(crate) fn contains_decoded_surface(&self, surface: &str) -> bool {
        normalize_surface(surface).is_some_and(|surface| self.decoder_contains_surface(&surface))
    }

    pub(crate) fn adjacent_transposition_candidates(
        &self,
        surface: &str,
    ) -> Vec<LexicalPhaseCandidate> {
        let Some(surface) = normalize_surface(surface) else {
            return Vec::new();
        };
        let query_field = SurfaceFieldEncoder::encode(&surface);
        let (query_phase, _) = surface_phase(&query_field);
        let query_keys = atom_center_keys(&query_field);
        let mut chars: Vec<char> = surface.chars().collect();
        let mut candidates = Vec::new();
        for index in 0..chars.len().saturating_sub(1) {
            if chars[index] == chars[index + 1] {
                continue;
            }
            chars.swap(index, index + 1);
            let candidate: String = chars.iter().collect();
            chars.swap(index, index + 1);
            if !self.decoder_contains_surface(&candidate)
                || candidates
                    .iter()
                    .any(|existing: &LexicalPhaseCandidate| existing.word == candidate)
            {
                continue;
            }
            let rank = self
                .terminal_for_normalized_surface(&candidate)
                .and_then(|terminal| read_terminal(self.bytes(), self.header, terminal))
                .map_or(u32::MAX, |terminal| terminal.rank);
            let candidate_field = SurfaceFieldEncoder::encode(&candidate);
            let (candidate_phase, atom_count) = surface_phase(&candidate_field);
            let coherence = phase_coherence_milli(&query_phase, &candidate_phase);
            let candidate_keys = atom_center_keys(&candidate_field);
            let overlap = sorted_overlap(&query_keys, &candidate_keys);
            let rank_boost = if rank == u32::MAX {
                0
            } else {
                corpus_rank_boost(rank).saturating_mul(2)
            };
            let score = 1_100u32
                .saturating_add(u32::from(coherence))
                .saturating_add(overlap.min(24) as u32 * 64)
                .saturating_add(rank_boost)
                .saturating_sub(240);
            candidates.push(LexicalPhaseCandidate {
                rank: if rank == u32::MAX {
                    usize::MAX
                } else {
                    rank as usize
                },
                score,
                l1_overlap: overlap,
                l2_overlap: usize::from(coherence) / 40,
                motif_overlap: atom_count as usize,
                prefix_match: false,
                phase_coherence_milli: coherence,
                reconstructed: true,
                word: candidate,
            });
        }
        sort_candidates(&mut candidates);
        candidates
    }

    pub(crate) fn surface_rank(&self, surface: &str) -> Option<usize> {
        let terminal = self.terminal_for_surface(surface)?;
        read_terminal(self.bytes(), self.header, terminal).map(|record| record.rank as usize)
    }

    pub(crate) fn phase_readout(&self, surface: &str) -> LexicalPhaseReadout {
        let Some(surface) = normalize_surface(surface) else {
            return LexicalPhaseReadout::default();
        };
        let field = SurfaceFieldEncoder::encode(&surface);
        let keys = atom_center_keys(&field);
        let center_hits = keys
            .iter()
            .filter(|key| self.center_by_key(**key).is_some())
            .count();
        let exact_terminal = self.terminal_for_normalized_surface(&surface);
        let (query_phase, atom_count) = surface_phase(&field);
        let phase_coherence_milli = exact_terminal
            .and_then(|terminal| read_terminal(self.bytes(), self.header, terminal))
            .map(|terminal| phase_coherence_milli(&query_phase, &terminal.phase))
            .unwrap_or_default();
        LexicalPhaseReadout {
            exact_center: exact_terminal.is_some(),
            atom_count: atom_count as usize,
            center_hits,
            phase_coherence_milli,
        }
    }

    pub(crate) fn surface_candidates(
        &self,
        surface: &str,
        limit: usize,
    ) -> Vec<LexicalPhaseCandidate> {
        if limit == 0 {
            return Vec::new();
        }
        let Some(surface) = normalize_surface(surface) else {
            return Vec::new();
        };
        if self.terminal_for_normalized_surface(&surface).is_some() {
            return Vec::new();
        }
        let mut candidates = self.field_surface_candidates_normalized(&surface, limit);
        let reconstructed =
            self.reconstructed_surface_candidates(&surface, limit.saturating_mul(2));
        let prefixed_reconstructed =
            self.prefixed_reconstructed_surface_candidates(&surface, limit.saturating_mul(2));
        let reconstruction_reserve = reconstruction_lane_reserve(
            &surface,
            &reconstructed,
            limit,
            RECONSTRUCTION_LANE_RESERVE,
        );
        let prefixed_reconstruction_reserve = reconstruction_lane_reserve(
            &surface,
            &prefixed_reconstructed,
            limit,
            RECONSTRUCTION_LANE_RESERVE,
        );
        let mut reconstructed = reconstructed;
        reconstructed.extend(prefixed_reconstructed);
        candidates.extend(reconstructed);
        sort_candidates(&mut candidates);
        let mut reserved = candidates
            .iter()
            .filter(|candidate| {
                crate::text_metrics::sparse_internal_omission_count(&surface, &candidate.word)
                    .is_some()
            })
            .take(4)
            .cloned()
            .collect::<Vec<_>>();
        for candidate in reconstruction_reserve {
            if !reserved.iter().any(|item| item.word == candidate.word) {
                reserved.push(candidate);
            }
        }
        for candidate in prefixed_reconstruction_reserve {
            if !reserved.iter().any(|item| item.word == candidate.word) {
                reserved.push(candidate);
            }
        }
        candidates.truncate(limit);
        for candidate in reserved {
            if candidates.iter().any(|item| item.word == candidate.word) {
                continue;
            }
            if candidates.len() == limit {
                candidates.pop();
            }
            candidates.push(candidate);
        }
        sort_candidates(&mut candidates);
        candidates
    }

    pub(crate) fn field_surface_candidates(
        &self,
        surface: &str,
        limit: usize,
    ) -> Vec<LexicalPhaseCandidate> {
        if limit == 0 {
            return Vec::new();
        }
        let Some(surface) = normalize_surface(surface) else {
            return Vec::new();
        };
        if self.terminal_for_normalized_surface(&surface).is_some() {
            return Vec::new();
        }
        let mut candidates = self.field_surface_candidates_normalized(&surface, limit);
        sort_candidates(&mut candidates);
        candidates.truncate(limit);
        candidates
    }

    fn field_surface_candidates_normalized(
        &self,
        surface: &str,
        limit: usize,
    ) -> Vec<LexicalPhaseCandidate> {
        let input_len = surface.chars().count();
        let field = SurfaceFieldEncoder::encode(surface);
        let keys = atom_center_keys(&field);
        let (query_phase, _) = surface_phase(&field);
        let mut votes = HashMap::<u32, u16>::new();
        for key in keys {
            let Some(center) = self.center_by_key(key) else {
                continue;
            };
            let posting_limit = usize::from(center.posting_len).min(MAX_POSTINGS_PER_QUERY_CENTER);
            for offset in 0..posting_limit {
                let posting_index = center.posting_start.saturating_add(offset as u32);
                let Some(terminal) = read_posting(self.bytes(), self.header, posting_index) else {
                    continue;
                };
                if let Some(vote) = votes.get_mut(&terminal) {
                    *vote = vote.saturating_add(1);
                } else if votes.len() < MAX_PHASE_FRONTIER {
                    votes.insert(terminal, 1);
                }
            }
        }

        let mut frontier = votes.into_iter().collect::<Vec<_>>();
        frontier.sort_by(|(left_id, left_votes), (right_id, right_votes)| {
            right_votes.cmp(left_votes).then_with(|| {
                self.terminal_rank(*left_id)
                    .cmp(&self.terminal_rank(*right_id))
            })
        });
        frontier.truncate(limit.saturating_mul(48).max(192));

        let candidates = frontier
            .into_iter()
            .filter_map(|(terminal_id, votes)| {
                let terminal = read_terminal(self.bytes(), self.header, terminal_id)?;
                let word = self.reconstruct_terminal(terminal_id)?;
                if word == surface {
                    return None;
                }
                let word_len = terminal.char_len as usize;
                let prefix_match = word.starts_with(surface) || surface.starts_with(&word);
                let distance = damerau_levenshtein(surface, &word);
                let distance_limit = if input_len >= 9 {
                    3
                } else if input_len >= 5 {
                    2
                } else {
                    1
                };
                if !prefix_match && distance > distance_limit {
                    return None;
                }
                if prefix_match && input_len.abs_diff(word_len) > 16 {
                    return None;
                }
                let coherence = phase_coherence_milli(&query_phase, &terminal.phase);
                let score = lexical_score(
                    input_len,
                    word_len,
                    distance,
                    votes,
                    coherence,
                    terminal.rank,
                    prefix_match,
                );
                Some(LexicalPhaseCandidate {
                    word,
                    score,
                    l1_overlap: votes as usize,
                    l2_overlap: usize::from(coherence) / 40,
                    motif_overlap: votes.saturating_sub(1) as usize,
                    prefix_match,
                    rank: terminal.rank as usize,
                    phase_coherence_milli: coherence,
                    reconstructed: false,
                })
            })
            .collect::<Vec<_>>();
        candidates
    }

    pub(crate) fn completion_candidates(
        &self,
        prefix: &str,
        result_limit: usize,
        material_limit: usize,
    ) -> Vec<LexicalPhaseCandidate> {
        if result_limit == 0 || material_limit == 0 {
            return Vec::new();
        }
        let Some(prefix) = normalize_surface(prefix) else {
            return Vec::new();
        };
        let Some(prefix_node) = self.node_for_normalized_surface(&prefix) else {
            return Vec::new();
        };
        let prefix_len = prefix.chars().count();
        let field = SurfaceFieldEncoder::encode(&prefix);
        let (query_phase, _) = surface_phase(&field);
        let mut heap = BinaryHeap::new();
        self.push_frontier(&mut heap, prefix_node);
        let mut visited = 0usize;
        let mut emitted = std::collections::BTreeSet::new();
        let mut candidates = Vec::new();
        while let Some(entry) = heap.pop() {
            if visited >= MAX_COMPLETION_FRONTIER
                || candidates.len() >= material_limit.max(result_limit)
            {
                break;
            }
            visited += 1;
            if !emitted.insert(entry.best_terminal) {
                continue;
            }
            if !entry.terminal_only {
                self.partition_frontier(&mut heap, entry.node, entry.best_terminal);
            }
            let Some(terminal) = read_terminal(self.bytes(), self.header, entry.best_terminal)
            else {
                continue;
            };
            if terminal.char_len as usize <= prefix_len {
                continue;
            }
            if let Some(word) = self.reconstruct_terminal(entry.best_terminal) {
                let coherence = phase_coherence_milli(&query_phase, &terminal.phase);
                candidates.push(LexicalPhaseCandidate {
                    score: completion_score(
                        terminal.rank,
                        terminal.support,
                        coherence,
                        terminal.char_len as usize - prefix_len,
                    ),
                    word,
                    l1_overlap: prefix_len,
                    l2_overlap: usize::from(coherence) / 40,
                    motif_overlap: usize::from(terminal.atom_count),
                    prefix_match: true,
                    rank: terminal.rank as usize,
                    phase_coherence_milli: coherence,
                    reconstructed: false,
                });
            }
        }
        sort_candidates(&mut candidates);
        candidates.truncate(result_limit);
        candidates
    }

    fn reconstructed_surface_candidates(
        &self,
        surface: &str,
        limit: usize,
    ) -> Vec<LexicalPhaseCandidate> {
        if limit == 0 || self.header.decoder_state_count == 0 {
            return Vec::new();
        }
        let input = surface.chars().collect::<Vec<_>>();
        let max_edits = if input.len() >= 6 {
            MAX_DECODE_EDITS
        } else {
            2
        };
        let max_depth = input.len().saturating_add(max_edits as usize);
        let row_width = input.len() + 1;
        let mut rows = vec![u8::MAX; (max_depth + 1).saturating_mul(row_width)];
        for (column, value) in rows[..row_width].iter_mut().enumerate() {
            *value = column as u8;
        }
        let mut output = Vec::with_capacity(max_depth);
        let mut visited = 0usize;
        let mut completed = Vec::<(String, usize)>::new();
        self.collect_reconstructed_matches(
            0,
            &input,
            max_edits,
            max_depth,
            row_width,
            0,
            &mut rows,
            &mut output,
            &mut visited,
            &mut completed,
        );

        let query_field = SurfaceFieldEncoder::encode(surface);
        let (query_phase, _) = surface_phase(&query_field);
        let query_keys = atom_center_keys(&query_field);
        let mut candidates = completed
            .into_iter()
            .filter_map(|(word, distance)| {
                if word == surface {
                    return None;
                }
                let rank = self
                    .terminal_for_normalized_surface(&word)
                    .and_then(|terminal| read_terminal(self.bytes(), self.header, terminal))
                    .map_or(u32::MAX, |terminal| terminal.rank);
                let candidate_field = SurfaceFieldEncoder::encode(&word);
                let (candidate_phase, atom_count) = surface_phase(&candidate_field);
                let coherence = phase_coherence_milli(&query_phase, &candidate_phase);
                let candidate_keys = atom_center_keys(&candidate_field);
                let overlap = sorted_overlap(&query_keys, &candidate_keys);
                let prefix_match = word.starts_with(surface) || surface.starts_with(&word);
                let rank_boost = if rank == u32::MAX {
                    0
                } else {
                    corpus_rank_boost(rank).saturating_mul(2)
                };
                let score = 1_100u32
                    .saturating_add(u32::from(coherence))
                    .saturating_add(overlap.min(24) as u32 * 64)
                    .saturating_add(rank_boost)
                    .saturating_sub(distance as u32 * 240)
                    .saturating_add(if prefix_match { 120 } else { 0 });
                Some(LexicalPhaseCandidate {
                    word,
                    score,
                    l1_overlap: overlap,
                    l2_overlap: usize::from(coherence) / 40,
                    motif_overlap: atom_count as usize,
                    prefix_match,
                    rank: if rank == u32::MAX {
                        usize::MAX
                    } else {
                        rank as usize
                    },
                    phase_coherence_milli: coherence,
                    reconstructed: true,
                })
            })
            .collect::<Vec<_>>();
        sort_candidates(&mut candidates);
        candidates.truncate(limit);
        candidates
    }

    fn prefixed_reconstructed_surface_candidates(
        &self,
        surface: &str,
        limit: usize,
    ) -> Vec<LexicalPhaseCandidate> {
        let mut candidates = Vec::new();
        let query_field = SurfaceFieldEncoder::encode(surface);
        let (query_phase, _) = surface_phase(&query_field);
        let query_keys = atom_center_keys(&query_field);
        for prefix in crate::russian_prefixes::derivational_prefixes() {
            let Some(base_surface) = surface.strip_prefix(prefix) else {
                continue;
            };
            if base_surface.chars().count() < 4 {
                continue;
            }
            let mut prefix_candidates = Vec::new();
            for base in self.reconstructed_surface_candidates(base_surface, limit.saturating_mul(2))
            {
                let word = format!("{prefix}{}", base.word);
                let distance = damerau_levenshtein(surface, &word);
                if distance > usize::from(MAX_DECODE_EDITS) {
                    continue;
                }
                let candidate_field = SurfaceFieldEncoder::encode(&word);
                let (candidate_phase, atom_count) = surface_phase(&candidate_field);
                let coherence = phase_coherence_milli(&query_phase, &candidate_phase);
                let candidate_keys = atom_center_keys(&candidate_field);
                let overlap = sorted_overlap(&query_keys, &candidate_keys);
                let rank_boost = if base.rank == usize::MAX {
                    0
                } else {
                    corpus_rank_boost(base.rank.min(u32::MAX as usize) as u32).saturating_mul(2)
                };
                let score = 1_100u32
                    .saturating_add(u32::from(coherence))
                    .saturating_add(overlap.min(24) as u32 * 64)
                    .saturating_add(rank_boost)
                    .saturating_sub(distance as u32 * 240);
                prefix_candidates.push(LexicalPhaseCandidate {
                    word,
                    score,
                    l1_overlap: overlap,
                    l2_overlap: usize::from(coherence) / 40,
                    motif_overlap: atom_count as usize,
                    prefix_match: false,
                    rank: base.rank,
                    phase_coherence_milli: coherence,
                    reconstructed: true,
                });
            }
            // Composition is a distinct operator path. A globally strong
            // reconstruction from another prefix must not erase the only
            // viable decoded continuation for this prefix before L2 can
            // compare the completed word centers.
            candidates.extend(reconstruction_lane_reserve(
                surface,
                &prefix_candidates,
                PREFIX_COMPOSITION_FRONTIER_PER_PREFIX,
                PREFIX_COMPOSITION_FRONTIER_PER_PREFIX,
            ));
        }
        sort_candidates(&mut candidates);
        candidates.truncate(limit);
        candidates
    }

    #[allow(clippy::too_many_arguments)]
    fn collect_reconstructed_matches(
        &self,
        state_id: u32,
        input: &[char],
        max_edits: u8,
        max_depth: usize,
        row_width: usize,
        depth: usize,
        rows: &mut [u8],
        output: &mut Vec<char>,
        visited: &mut usize,
        completed: &mut Vec<(String, usize)>,
    ) {
        if *visited >= MAX_DAFSA_VISITS {
            return;
        }
        *visited += 1;
        let Some(state) = read_decoder_state(self.bytes(), self.header, state_id) else {
            return;
        };
        let row_start = depth * row_width;
        let row = &rows[row_start..row_start + row_width];
        let distance = usize::from(row[row_width - 1]);
        if state.is_final() && distance <= max_edits as usize && !output.is_empty() {
            completed.push((output.iter().collect(), distance));
        }
        if depth >= max_depth
            || row
                .iter()
                .copied()
                .min()
                .map_or(true, |minimum| minimum > max_edits)
        {
            return;
        }

        for offset in 0..state.arc_len {
            if *visited >= MAX_DAFSA_VISITS {
                return;
            }
            let Some(arc) = read_decoder_arc(
                self.bytes(),
                self.header,
                state.first_arc.saturating_add(u32::from(offset)),
            ) else {
                continue;
            };
            let Some(ch) = char::from_u32(arc.ch) else {
                continue;
            };
            let target_start = (depth + 1) * row_width;
            let row_is_live = {
                let (prior_rows, target_rows) = rows.split_at_mut(target_start);
                let target = &mut target_rows[..row_width];
                let previous = &prior_rows[row_start..row_start + row_width];
                let previous_previous = (depth > 0).then(|| {
                    let start = (depth - 1) * row_width;
                    &prior_rows[start..start + row_width]
                });
                next_damerau_row_into(
                    input,
                    ch,
                    previous,
                    previous_previous,
                    output.last().copied(),
                    target,
                );
                target
                    .iter()
                    .copied()
                    .min()
                    .is_some_and(|minimum| minimum <= max_edits)
            };
            if !row_is_live {
                continue;
            }
            output.push(ch);
            self.collect_reconstructed_matches(
                arc.child,
                input,
                max_edits,
                max_depth,
                row_width,
                depth + 1,
                rows,
                output,
                visited,
                completed,
            );
            output.pop();
        }
    }

    fn decoder_contains_surface(&self, surface: &str) -> bool {
        let mut state_id = 0u32;
        for ch in surface.chars() {
            state_id = match self.decoder_child(state_id, ch) {
                Some(child) => child,
                None => return false,
            };
        }
        read_decoder_state(self.bytes(), self.header, state_id)
            .is_some_and(|state| state.is_final())
    }

    fn decoder_child(&self, state_id: u32, ch: char) -> Option<u32> {
        let state = read_decoder_state(self.bytes(), self.header, state_id)?;
        let mut low = 0usize;
        let mut high = state.arc_len as usize;
        while low < high {
            let mid = low + (high - low) / 2;
            let arc = read_decoder_arc(
                self.bytes(),
                self.header,
                state.first_arc.saturating_add(mid as u32),
            )?;
            match arc.ch.cmp(&(ch as u32)) {
                Ordering::Less => low = mid + 1,
                Ordering::Greater => high = mid,
                Ordering::Equal => return Some(arc.child),
            }
        }
        None
    }

    fn bytes(&self) -> &[u8] {
        self.bytes.as_slice()
    }

    fn terminal_for_surface(&self, surface: &str) -> Option<u32> {
        let surface = normalize_surface(surface)?;
        self.terminal_for_normalized_surface(&surface)
    }

    fn terminal_for_normalized_surface(&self, surface: &str) -> Option<u32> {
        let node = self.node_for_normalized_surface(surface)?;
        let terminal = read_node(self.bytes(), self.header, node)?.terminal;
        (terminal != NO_INDEX).then_some(terminal)
    }

    fn node_for_normalized_surface(&self, surface: &str) -> Option<u32> {
        let mut node_id = 0u32;
        for ch in surface.chars() {
            node_id = self.child_for_char(node_id, ch)?;
        }
        Some(node_id)
    }

    fn child_for_char(&self, node_id: u32, ch: char) -> Option<u32> {
        let node = read_node(self.bytes(), self.header, node_id)?;
        let mut low = 0usize;
        let mut high = node.arc_len as usize;
        while low < high {
            let mid = low + (high - low) / 2;
            let arc = read_arc(
                self.bytes(),
                self.header,
                node.first_arc.saturating_add(mid as u32),
            )?;
            match arc.ch.cmp(&(ch as u32)) {
                Ordering::Less => low = mid + 1,
                Ordering::Greater => high = mid,
                Ordering::Equal => return Some(arc.child),
            }
        }
        None
    }

    fn reconstruct_terminal(&self, terminal_id: u32) -> Option<String> {
        let terminal = read_terminal(self.bytes(), self.header, terminal_id)?;
        let mut node_id = terminal.node;
        let mut chars = Vec::with_capacity(terminal.char_len as usize);
        while node_id != 0 {
            let node = read_node(self.bytes(), self.header, node_id)?;
            chars.push(char::from_u32(node.incoming)?);
            node_id = node.parent;
            if chars.len() > super::format::MAX_WORD_CHARS {
                return None;
            }
        }
        chars.reverse();
        Some(chars.into_iter().collect())
    }

    fn center_by_key(&self, key: u64) -> Option<CenterRecord> {
        let mut low = 0u32;
        let mut high = self.header.center_count;
        while low < high {
            let mid = low + (high - low) / 2;
            let center = read_center(self.bytes(), self.header, mid)?;
            match center.key.cmp(&key) {
                Ordering::Less => low = mid + 1,
                Ordering::Greater => high = mid,
                Ordering::Equal => return Some(center),
            }
        }
        None
    }

    fn terminal_rank(&self, terminal_id: u32) -> u32 {
        read_terminal(self.bytes(), self.header, terminal_id)
            .map(|terminal| terminal.rank)
            .unwrap_or(u32::MAX)
    }

    fn push_frontier(&self, heap: &mut BinaryHeap<CompletionFrontier>, node: u32) {
        let Some(node_record) = read_node(self.bytes(), self.header, node) else {
            return;
        };
        if node_record.best_terminal == NO_INDEX {
            return;
        }
        heap.push(CompletionFrontier {
            node,
            best_terminal: node_record.best_terminal,
            best_rank: self.terminal_rank(node_record.best_terminal),
            terminal_only: false,
        });
    }

    fn push_terminal_frontier(&self, heap: &mut BinaryHeap<CompletionFrontier>, terminal: u32) {
        let Some(record) = read_terminal(self.bytes(), self.header, terminal) else {
            return;
        };
        heap.push(CompletionFrontier {
            node: record.node,
            best_terminal: terminal,
            best_rank: record.rank,
            terminal_only: true,
        });
    }

    fn partition_frontier(
        &self,
        heap: &mut BinaryHeap<CompletionFrontier>,
        root: u32,
        excluded_terminal: u32,
    ) {
        let Some(terminal) = read_terminal(self.bytes(), self.header, excluded_terminal) else {
            return;
        };
        let mut reverse_path = vec![terminal.node];
        let mut current = terminal.node;
        while current != root {
            let Some(node) = read_node(self.bytes(), self.header, current) else {
                return;
            };
            current = node.parent;
            reverse_path.push(current);
            if reverse_path.len() > super::format::MAX_WORD_CHARS + 1 {
                return;
            }
        }
        reverse_path.reverse();

        for (index, node_id) in reverse_path.iter().copied().enumerate() {
            let Some(node) = read_node(self.bytes(), self.header, node_id) else {
                continue;
            };
            if node.terminal != NO_INDEX && node.terminal != excluded_terminal {
                self.push_terminal_frontier(heap, node.terminal);
            }
            let path_child = reverse_path.get(index + 1).copied();
            for arc_offset in 0..node.arc_len {
                let Some(arc) = read_arc(
                    self.bytes(),
                    self.header,
                    node.first_arc.saturating_add(u32::from(arc_offset)),
                ) else {
                    continue;
                };
                if Some(arc.child) != path_child {
                    self.push_frontier(heap, arc.child);
                }
            }
        }
    }
}

/// Keeps a small independent decode lane alive through the final lattice cut.
/// Field postings describe local motif pressure, while decoder reconstruction
/// describes a complete attested surface. Neither is allowed to erase the
/// other before L2/L3 phase competition sees both alternatives.
fn reconstruction_lane_reserve(
    surface: &str,
    candidates: &[LexicalPhaseCandidate],
    limit: usize,
    max_reserve: usize,
) -> Vec<LexicalPhaseCandidate> {
    let input_len = surface.chars().count();
    let mut reserve = candidates
        .iter()
        .filter(|candidate| candidate.reconstructed)
        .cloned()
        .collect::<Vec<_>>();
    reserve.sort_by(|left, right| {
        let left_distance = damerau_levenshtein(surface, &left.word);
        let right_distance = damerau_levenshtein(surface, &right.word);
        left_distance
            .cmp(&right_distance)
            .then_with(|| {
                right
                    .word
                    .chars()
                    .count()
                    .abs_diff(input_len)
                    .cmp(&left.word.chars().count().abs_diff(input_len))
            })
            .then_with(|| right.score.cmp(&left.score))
            .then_with(|| right.phase_coherence_milli.cmp(&left.phase_coherence_milli))
            .then_with(|| left.rank.cmp(&right.rank))
            .then_with(|| left.word.cmp(&right.word))
    });
    let capacity = limit.min(max_reserve);
    let mut selected = Vec::with_capacity(capacity);
    let mut covered_distances = Vec::new();

    // Preserve one center from each edit-distance wave. Otherwise the many
    // cheap one-letter variants can erase a real two-letter restoration before
    // L2/L3 has a chance to compare complete lexical centers.
    for candidate in &reserve {
        let distance = damerau_levenshtein(surface, &candidate.word);
        if !covered_distances.contains(&distance) {
            covered_distances.push(distance);
            selected.push(candidate.clone());
            if selected.len() == capacity {
                return selected;
            }
        }
    }
    for candidate in reserve {
        if selected
            .iter()
            .any(|existing| existing.word == candidate.word)
        {
            continue;
        }
        selected.push(candidate);
        if selected.len() == capacity {
            break;
        }
    }
    selected
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct LexicalPhaseReadout {
    pub(crate) exact_center: bool,
    pub(crate) atom_count: usize,
    pub(crate) center_hits: usize,
    pub(crate) phase_coherence_milli: u16,
}

fn next_damerau_row_into(
    input: &[char],
    ch: char,
    previous: &[u8],
    previous_previous: Option<&[u8]>,
    previous_char: Option<char>,
    row: &mut [u8],
) {
    row[0] = previous[0].saturating_add(1);
    for column in 1..=input.len() {
        let insertion = row[column - 1].saturating_add(1);
        let deletion = previous[column].saturating_add(1);
        let substitution = previous[column - 1].saturating_add(u8::from(input[column - 1] != ch));
        let mut value = insertion.min(deletion).min(substitution);
        if column > 1 && ch == input[column - 2] && previous_char == Some(input[column - 1]) {
            if let Some(previous_previous) = previous_previous {
                value = value.min(previous_previous[column - 2].saturating_add(1));
            }
        }
        row[column] = value;
    }
}

fn sorted_overlap(left: &[u64], right: &[u64]) -> usize {
    let mut left_index = 0usize;
    let mut right_index = 0usize;
    let mut overlap = 0usize;
    while left_index < left.len() && right_index < right.len() {
        match left[left_index].cmp(&right[right_index]) {
            Ordering::Less => left_index += 1,
            Ordering::Greater => right_index += 1,
            Ordering::Equal => {
                overlap += 1;
                left_index += 1;
                right_index += 1;
            }
        }
    }
    overlap
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct CompletionFrontier {
    node: u32,
    best_terminal: u32,
    best_rank: u32,
    terminal_only: bool,
}

impl Ord for CompletionFrontier {
    fn cmp(&self, other: &Self) -> Ordering {
        other
            .best_rank
            .cmp(&self.best_rank)
            .then_with(|| other.terminal_only.cmp(&self.terminal_only))
            .then_with(|| other.best_terminal.cmp(&self.best_terminal))
            .then_with(|| other.node.cmp(&self.node))
    }
}

impl PartialOrd for CompletionFrontier {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

fn lexical_score(
    input_len: usize,
    word_len: usize,
    distance: usize,
    votes: u16,
    coherence: u16,
    rank: u32,
    prefix_match: bool,
) -> u32 {
    let distance_bonus = 1_200u32.saturating_sub(distance.min(4) as u32 * 280);
    let rank_bonus = corpus_rank_boost(rank);
    let prefix_bonus = if prefix_match { 240 } else { 0 };
    u32::from(votes)
        .saturating_mul(96)
        .saturating_add(u32::from(coherence))
        .saturating_add(distance_bonus)
        .saturating_add(rank_bonus)
        .saturating_add(prefix_bonus)
        .saturating_sub(input_len.abs_diff(word_len).min(16) as u32 * 24)
}

fn completion_score(rank: u32, support: u32, coherence: u16, generated: usize) -> u32 {
    corpus_rank_boost(rank)
        .saturating_add(u32::from(coherence))
        .saturating_add(support.min(128))
        .saturating_add(360)
        .saturating_sub(generated.min(16) as u32 * 10)
}

fn corpus_rank_boost(rank: u32) -> u32 {
    match rank {
        0..=999 => 640,
        1_000..=4_999 => 500,
        5_000..=19_999 => 340,
        20_000..=59_999 => 180,
        _ => 80,
    }
}

fn sort_candidates(candidates: &mut Vec<LexicalPhaseCandidate>) {
    candidates.sort_by(|left, right| {
        right
            .score
            .cmp(&left.score)
            .then_with(|| right.phase_coherence_milli.cmp(&left.phase_coherence_milli))
            .then_with(|| right.l1_overlap.cmp(&left.l1_overlap))
            .then_with(|| left.rank.cmp(&right.rank))
            .then_with(|| left.word.cmp(&right.word))
    });
    candidates.dedup_by(|left, right| left.word == right.word);
}

enum ArtifactBytes {
    #[cfg(target_os = "linux")]
    Mapped(MappedFile),
    Owned(Box<[u8]>),
}

impl ArtifactBytes {
    fn load(path: &Path) -> Result<Self, String> {
        #[cfg(target_os = "linux")]
        {
            MappedFile::open(path).map(Self::Mapped)
        }
        #[cfg(not(target_os = "linux"))]
        {
            std::fs::read(path)
                .map(|bytes| Self::Owned(bytes.into_boxed_slice()))
                .map_err(|error| format!("{}: {error}", path.display()))
        }
    }

    fn as_slice(&self) -> &[u8] {
        match self {
            #[cfg(target_os = "linux")]
            Self::Mapped(mapped) => mapped.as_slice(),
            Self::Owned(bytes) => bytes,
        }
    }

    fn is_mapped(&self) -> bool {
        #[cfg(target_os = "linux")]
        {
            matches!(self, Self::Mapped(_))
        }
        #[cfg(not(target_os = "linux"))]
        {
            false
        }
    }

    fn prefetch_all(&self) {
        match self {
            #[cfg(target_os = "linux")]
            Self::Mapped(mapped) => mapped.prefetch_all(),
            Self::Owned(bytes) => touch_pages(bytes),
        }
    }
}

fn touch_pages(bytes: &[u8]) {
    const PAGE_BYTES: usize = 4096;
    let mut checksum = 0_u8;
    for offset in (0..bytes.len()).step_by(PAGE_BYTES) {
        checksum ^= unsafe { std::ptr::read_volatile(bytes.as_ptr().add(offset)) };
    }
    if let Some(last) = bytes.last() {
        checksum ^= *last;
    }
    std::hint::black_box(checksum);
}

#[cfg(target_os = "linux")]
struct MappedFile {
    ptr: *mut libc::c_void,
    len: usize,
}

#[cfg(target_os = "linux")]
impl MappedFile {
    fn open(path: &Path) -> Result<Self, String> {
        use std::os::fd::AsRawFd;

        let file = File::open(path).map_err(|error| format!("{}: {error}", path.display()))?;
        let len = file
            .metadata()
            .map_err(|error| format!("{}: {error}", path.display()))?
            .len() as usize;
        if len == 0 {
            return Err(format!("{}: empty lexical phase artifact", path.display()));
        }
        let ptr = unsafe {
            libc::mmap(
                std::ptr::null_mut(),
                len,
                libc::PROT_READ,
                libc::MAP_PRIVATE,
                file.as_raw_fd(),
                0,
            )
        };
        if ptr == libc::MAP_FAILED {
            return Err(format!(
                "{}: mmap failed: {}",
                path.display(),
                std::io::Error::last_os_error()
            ));
        }
        Ok(Self { ptr, len })
    }

    fn as_slice(&self) -> &[u8] {
        unsafe { std::slice::from_raw_parts(self.ptr.cast::<u8>(), self.len) }
    }

    fn prefetch_all(&self) {
        unsafe {
            libc::madvise(self.ptr, self.len, libc::MADV_WILLNEED);
        }
        touch_pages(self.as_slice());
    }
}

#[cfg(target_os = "linux")]
unsafe impl Send for MappedFile {}
#[cfg(target_os = "linux")]
unsafe impl Sync for MappedFile {}

#[cfg(target_os = "linux")]
impl Drop for MappedFile {
    fn drop(&mut self) {
        unsafe {
            libc::munmap(self.ptr, self.len);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::nanda_wave::lexical_phase::compiler::{compile_words, compile_words_with_training};

    fn memory() -> LexicalPhaseMemory {
        let bytes = compile_words([
            "загрузи",
            "загрузить",
            "выгрузи",
            "пункт",
            "пункты",
            "проверка",
            "проверить",
            "проверяем",
            "привет",
        ])
        .expect("fixture compiles");
        LexicalPhaseMemory::from_bytes(bytes).expect("fixture loads")
    }

    #[test]
    fn graph_reconstructs_surface_without_raw_word_table() {
        let memory = memory();

        assert!(memory.contains_surface("проверка"));
        assert_eq!(memory.surface_rank("загрузи"), Some(0));
        assert!(!memory.stats().raw_word_table);
    }

    #[test]
    fn phase_field_recovers_corrupted_surface() {
        let memory = memory();
        let candidates = memory.surface_candidates("звгрузи", 4);

        assert!(
            candidates
                .iter()
                .any(|candidate| candidate.word == "загрузи"),
            "candidates={candidates:?}"
        );
    }

    #[test]
    fn decoder_recovers_adjacent_transposition_as_typed_operator() {
        let memory = memory();
        let candidates = memory.adjacent_transposition_candidates("пукнт");

        assert_eq!(
            candidates
                .iter()
                .map(|candidate| candidate.word.as_str())
                .collect::<Vec<_>>(),
            vec!["пункт"]
        );
        assert!(memory.adjacent_transposition_candidates("пункт").is_empty());
    }

    #[test]
    fn grapheme_graph_completes_prefix() {
        let memory = memory();
        let candidates = memory.completion_candidates("пров", 4, 24);

        assert!(
            candidates
                .iter()
                .any(|candidate| candidate.word.starts_with("провер")),
            "candidates={candidates:?}"
        );
    }

    #[test]
    fn decoder_reconstructs_training_form_absent_from_terminal_graph() {
        let base = ["проверка", "проверить", "загрузить"];
        let training = [
            "проверка",
            "проверить",
            "загрузить",
            "работает",
            "работаем",
            "работал",
            "работала",
            "работали",
        ];
        let bytes = compile_words_with_training(base, training).expect("artifact compiles");
        let memory = LexicalPhaseMemory::from_bytes(bytes).expect("artifact loads");

        assert!(!memory.contains_surface("работает"));
        let candidates = memory.surface_candidates("рабоатет", 12);
        assert!(
            candidates
                .iter()
                .any(|candidate| candidate.word == "работает" && candidate.reconstructed),
            "candidates={candidates:?}"
        );
    }

    #[test]
    fn reconstruction_lane_survives_field_frontier_cut() {
        let bytes =
            compile_words(["про", "при", "приз", "прим", "прям"]).expect("fixture compiles");
        let memory = LexicalPhaseMemory::from_bytes(bytes).expect("fixture loads");
        let candidates = memory.surface_candidates("прм", 4);

        assert!(
            candidates
                .iter()
                .any(|candidate| candidate.word == "прям" && candidate.reconstructed),
            "complete decoded surface must survive the bounded lattice: {candidates:?}"
        );
    }

    #[test]
    fn production_decoder_does_not_accept_dirty_probe_surfaces() {
        let memory = default_memory().expect("production lexical phase artifact loads");
        for dirty in ["пукнт", "звгрузи", "эсперемнт", "труссс", "поянл"]
        {
            assert!(
                !memory.decoder_contains_surface(dirty),
                "decoder accepted dirty probe as a training surface: {dirty:?}"
            );
        }
        assert!(memory.decoder_contains_surface("можем"));
        assert!(memory.decoder_contains_surface("понял"));
    }

    #[test]
    fn production_decoder_contains_generated_base_form_for_prefix_composition() {
        let memory = default_memory().expect("production lexical phase artifact loads");

        assert!(memory.decoder_contains_surface("подключаю"));
        let candidates = memory.surface_candidates("подлчаю", 64);
        assert!(
            candidates
                .iter()
                .any(|candidate| candidate.word == "подключаю"),
            "candidates={candidates:?}"
        );

        let candidates = memory.surface_candidates("переподлчаю", 32);
        assert!(
            candidates
                .iter()
                .any(|candidate| candidate.word == "переподключаю" && candidate.reconstructed),
            "candidates={candidates:?}"
        );
    }
}
