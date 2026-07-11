//! Generative L2 surface decoder.
//!
//! This is intentionally not a `prefix -> words` table. Training text is used
//! to compile a hot grapheme-state field; runtime reads transitions and decodes
//! a surface form from the current prefix state.

use std::collections::HashMap;
use std::sync::OnceLock;

use crate::keyboard::is_cyrillic_letter;

use super::surface_bank::normalize_l2_surface_word;

const L2_SURFACE_FOUNDATION_RU_DATA: &str =
    include_str!("../../data/lexicon/l2_surface_foundation_ru_100k.txt");
const L2_SURFACE_HOT_RU_DATA: &str = include_str!("../../data/lexicon/l2_surface_hot_ru.txt");
const COMMON_RU_DATA: &str = include_str!("../../data/lexicon/common_ru.txt");

const MAX_TRAIN_WORDS: usize = 120_000;
const MAX_WORD_CHARS: usize = 24;
const MAX_ARCS_PER_STATE: usize = 12;
const MAX_BEAM: usize = 32;
const MAX_EXTRA_CHARS: usize = 16;
const MIN_TERMINAL_SUPPORT: u16 = 2;
const MIN_GENERATED_CHARS: usize = 1;

static DECODER: OnceLock<L2SurfaceDecoder> = OnceLock::new();

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct L2SurfaceDecoderStats {
    pub(super) source_words: usize,
    pub(super) states: usize,
    pub(super) arcs: usize,
    pub(super) hot_bytes: usize,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct DecodedSurfaceCandidate {
    pub(super) word: String,
    pub(super) score: u32,
    pub(super) support: u16,
    pub(super) generated_chars: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
struct DecoderState {
    prev2: u32,
    prev1: u32,
    pos: u8,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct DecoderArc {
    ch: char,
    weight_milli: u16,
    support: u16,
}

#[derive(Clone, Debug, Default)]
struct RawNode {
    arcs: HashMap<char, u32>,
    terminal: u32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct DecoderNode {
    arcs: Vec<DecoderArc>,
    terminal_weight_milli: u16,
    terminal_support: u16,
}

#[derive(Clone, Debug)]
struct L2SurfaceDecoder {
    nodes: HashMap<DecoderState, DecoderNode>,
    source_words: usize,
    arc_count: usize,
}

#[derive(Clone, Debug)]
struct Beam {
    word: String,
    state: DecoderState,
    score: u32,
    support: u16,
    generated_chars: usize,
}

pub(super) fn warm_up() {
    let _ = decoder().stats();
}

pub(super) fn is_warm() -> bool {
    DECODER.get().is_some()
}

pub(super) fn stats() -> L2SurfaceDecoderStats {
    decoder().stats()
}

pub(super) fn completion_candidates(prefix: &str, limit: usize) -> Vec<DecodedSurfaceCandidate> {
    decoder().completion_candidates(prefix, limit)
}

fn decoder() -> &'static L2SurfaceDecoder {
    DECODER.get_or_init(|| {
        let mut seen = std::collections::HashSet::<String>::new();
        let mut words = Vec::<String>::new();
        for data in [
            COMMON_RU_DATA,
            L2_SURFACE_HOT_RU_DATA,
            L2_SURFACE_FOUNDATION_RU_DATA,
        ] {
            for line in data.lines().map(str::trim) {
                if words.len() >= MAX_TRAIN_WORDS {
                    break;
                }
                if line.is_empty() || line.starts_with('#') {
                    continue;
                }
                let Some(word) = normalize_l2_surface_word(line) else {
                    continue;
                };
                let len = word.chars().count();
                if !(3..=MAX_WORD_CHARS).contains(&len) || !word.chars().all(is_cyrillic_letter) {
                    continue;
                }
                if seen.insert(word.clone()) {
                    words.push(word);
                }
            }
        }
        let decoder = L2SurfaceDecoder::build(words.iter().map(String::as_str));
        drop(words);
        decoder
    })
}

impl L2SurfaceDecoder {
    fn build<'a, I>(words: I) -> Self
    where
        I: IntoIterator<Item = &'a str>,
    {
        let mut raw = HashMap::<DecoderState, RawNode>::new();
        let mut source_words = 0usize;
        for word in words {
            let chars = word.chars().collect::<Vec<_>>();
            if chars.is_empty() {
                continue;
            }
            source_words += 1;
            let mut prev2 = 0;
            let mut prev1 = 0;
            for (idx, ch) in chars.iter().copied().enumerate() {
                let state = DecoderState {
                    prev2,
                    prev1,
                    pos: position_bucket(idx),
                };
                *raw.entry(state).or_default().arcs.entry(ch).or_default() += 1;
                prev2 = prev1;
                prev1 = ch as u32;
            }
            raw.entry(DecoderState {
                prev2,
                prev1,
                pos: position_bucket(chars.len()),
            })
            .or_default()
            .terminal += 1;
        }

        let mut arc_count = 0usize;
        let nodes = raw
            .into_iter()
            .filter_map(|(state, node)| {
                let total = node
                    .arcs
                    .values()
                    .copied()
                    .sum::<u32>()
                    .saturating_add(node.terminal);
                if total == 0 {
                    return None;
                }
                let mut arcs = node
                    .arcs
                    .into_iter()
                    .map(|(ch, support)| DecoderArc {
                        ch,
                        weight_milli: weight_milli(support, total),
                        support: support.min(u16::MAX as u32) as u16,
                    })
                    .collect::<Vec<_>>();
                arcs.sort_by(|left, right| {
                    right
                        .support
                        .cmp(&left.support)
                        .then_with(|| right.weight_milli.cmp(&left.weight_milli))
                        .then_with(|| left.ch.cmp(&right.ch))
                });
                arcs.truncate(MAX_ARCS_PER_STATE);
                arc_count += arcs.len();
                Some((
                    state,
                    DecoderNode {
                        arcs,
                        terminal_weight_milli: weight_milli(node.terminal, total),
                        terminal_support: node.terminal.min(u16::MAX as u32) as u16,
                    },
                ))
            })
            .collect::<HashMap<_, _>>();

        Self {
            nodes,
            source_words,
            arc_count,
        }
    }

    fn stats(&self) -> L2SurfaceDecoderStats {
        L2SurfaceDecoderStats {
            source_words: self.source_words,
            states: self.nodes.len(),
            arcs: self.arc_count,
            hot_bytes: self.hot_bytes(),
        }
    }

    fn hot_bytes(&self) -> usize {
        self.nodes.len()
            * (std::mem::size_of::<DecoderState>() + std::mem::size_of::<DecoderNode>())
            + self.arc_count * std::mem::size_of::<DecoderArc>()
    }

    fn completion_candidates(&self, prefix: &str, limit: usize) -> Vec<DecodedSurfaceCandidate> {
        if limit == 0 {
            return Vec::new();
        }
        let Some(prefix) = normalize_l2_surface_word(prefix) else {
            return Vec::new();
        };
        let prefix_len = prefix.chars().count();
        if !(2..=18).contains(&prefix_len) || !prefix.chars().all(is_cyrillic_letter) {
            return Vec::new();
        }
        let mut beams = vec![Beam {
            word: prefix.clone(),
            state: state_after_prefix(&prefix),
            score: 0,
            support: u16::MAX,
            generated_chars: 0,
        }];
        let mut out = Vec::<DecodedSurfaceCandidate>::new();
        for _ in 0..MAX_EXTRA_CHARS {
            let mut next = Vec::<Beam>::new();
            for beam in &beams {
                let Some(node) = self.nodes.get(&beam.state) else {
                    continue;
                };
                if beam.generated_chars >= MIN_GENERATED_CHARS
                    && node.terminal_support >= MIN_TERMINAL_SUPPORT
                {
                    out.push(DecodedSurfaceCandidate {
                        word: beam.word.clone(),
                        score: terminal_score(beam.score, beam.generated_chars, node),
                        support: beam.support.min(node.terminal_support),
                        generated_chars: beam.generated_chars,
                    });
                }
                for arc in node.arcs.iter().take(8) {
                    if arc.weight_milli < 16 && next.len() >= limit.saturating_mul(2).max(limit) {
                        continue;
                    }
                    let mut word = beam.word.clone();
                    word.push(arc.ch);
                    next.push(Beam {
                        word,
                        state: advance_state(beam.state, arc.ch),
                        score: beam.score.saturating_add(arc.weight_milli as u32),
                        support: beam.support.min(arc.support),
                        generated_chars: beam.generated_chars + 1,
                    });
                }
            }
            if next.is_empty() {
                break;
            }
            next.sort_by(|left, right| {
                right
                    .score
                    .cmp(&left.score)
                    .then_with(|| right.support.cmp(&left.support))
                    .then_with(|| left.generated_chars.cmp(&right.generated_chars))
                    .then_with(|| left.word.cmp(&right.word))
            });
            next.truncate(MAX_BEAM);
            beams = next;
            if out.len() >= limit.saturating_mul(4).max(limit) {
                break;
            }
        }
        out.sort_by(|left, right| {
            right
                .score
                .cmp(&left.score)
                .then_with(|| right.support.cmp(&left.support))
                .then_with(|| left.generated_chars.cmp(&right.generated_chars))
                .then_with(|| left.word.cmp(&right.word))
        });
        out.dedup_by(|left, right| left.word == right.word);
        out.truncate(limit);
        out
    }
}

fn state_after_prefix(prefix: &str) -> DecoderState {
    let chars = prefix.chars().collect::<Vec<_>>();
    let len = chars.len();
    DecoderState {
        prev2: len
            .checked_sub(2)
            .and_then(|idx| chars.get(idx).copied())
            .map(|ch| ch as u32)
            .unwrap_or(0),
        prev1: chars.last().copied().map(|ch| ch as u32).unwrap_or(0),
        pos: position_bucket(len),
    }
}

fn advance_state(state: DecoderState, ch: char) -> DecoderState {
    DecoderState {
        prev2: state.prev1,
        prev1: ch as u32,
        pos: position_bucket(state.pos as usize + 1),
    }
}

fn position_bucket(pos: usize) -> u8 {
    pos.min(18) as u8
}

fn weight_milli(count: u32, total: u32) -> u16 {
    if total == 0 {
        return 0;
    }
    ((count.saturating_mul(1000) / total).min(u16::MAX as u32)) as u16
}

fn terminal_score(path_score: u32, generated_chars: usize, node: &DecoderNode) -> u32 {
    if generated_chars == 0 {
        return 0;
    }
    let avg_path = path_score / generated_chars as u32;
    avg_path
        .saturating_add(node.terminal_weight_milli as u32)
        .saturating_add(node.terminal_support.min(256) as u32)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decoder_generates_surface_without_word_bucket_lookup() {
        let candidates = completion_candidates("пров", 8);

        assert!(
            candidates
                .iter()
                .any(|candidate| candidate.word.starts_with("пров")
                    && candidate.word.chars().count() > 4),
            "decoder candidates: {candidates:?}"
        );
    }

    #[test]
    fn decoder_has_compact_hot_state() {
        let stats = stats();

        assert!(stats.source_words >= 10_000, "{stats:?}");
        assert!(stats.states > 0, "{stats:?}");
        assert!(stats.arcs > 0, "{stats:?}");
        assert!(
            stats.hot_bytes < 16 * 1024 * 1024,
            "decoder hot state should stay compact: {stats:?}"
        );
    }
}
