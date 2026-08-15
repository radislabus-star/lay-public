//! Exact typed traversal shared by proof and the V9 runtime owner.

use std::borrow::Cow;
use std::collections::{BTreeMap, BTreeSet};
#[cfg(any(test, feature = "lexical-compiler"))]
use std::io;
#[cfg(any(test, feature = "lexical-compiler"))]
use std::path::Path;
#[cfg(any(test, feature = "lexical-compiler"))]
use std::thread;
#[cfg(any(test, feature = "lexical-compiler"))]
use std::time::Instant;

use serde::Serialize;
#[cfg(any(test, feature = "lexical-compiler"))]
use sha2::{Digest, Sha256};

use crate::dict::{detect_direction, project_char, Direction};

use super::atoms::normalize_lexical_surface;
#[cfg(any(test, feature = "lexical-compiler"))]
use super::format;
#[cfg(any(test, feature = "lexical-compiler"))]
use super::forward_decoder_index::file_sha256;
use super::forward_decoder_index::{ForwardChild, ForwardDecoderIndex};
use super::model::DecoderNode;
#[cfg(any(test, feature = "lexical-compiler"))]
use super::model::LexicalGrokkingPackage;
#[cfg(any(test, feature = "lexical-compiler"))]
use super::proof::{
    corpus_words_from_lines, prepare_fixed_heldout_cases, proof_worker_count, FixedHeldoutCase,
};
#[cfg(any(test, feature = "lexical-compiler"))]
use super::runtime::LexicalGrokkingMemory;

#[cfg(any(test, feature = "lexical-compiler"))]
const PHASE7A_CLASSES: [&str; 3] = [
    "prefix_truncation",
    "suffix_truncation",
    "punctuation_suffix",
];
#[cfg(any(test, feature = "lexical-compiler"))]
const PHASE7B_CLASSES: [&str; 7] = [
    "missing_letter",
    "extra_letter",
    "letter_substitution",
    "prefix_truncation",
    "suffix_truncation",
    "layout_projection",
    "punctuation_suffix",
];
#[cfg(any(test, feature = "lexical-compiler"))]
const PHASE7C_CLASSES: [&str; 10] = [
    "missing_letter",
    "extra_letter",
    "adjacent_transposition",
    "letter_substitution",
    "prefix_truncation",
    "suffix_truncation",
    "layout_projection",
    "punctuation_suffix",
    "non_adjacent_transposition",
    "repeated_fragment",
];
#[cfg(any(test, feature = "lexical-compiler"))]
const PHASE7D_CLASSES: [&str; 13] = [
    "missing_letter",
    "extra_letter",
    "adjacent_transposition",
    "letter_substitution",
    "double_substitution",
    "prefix_truncation",
    "suffix_truncation",
    "layout_projection",
    "punctuation_suffix",
    "non_adjacent_transposition",
    "sparse_multi_omission",
    "omission_transposition",
    "repeated_fragment",
];

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "snake_case")]
enum LayoutDirection {
    UsToRu,
    RuToUs,
}

impl LayoutDirection {
    fn from_raw(raw: &str) -> Self {
        match detect_direction(raw) {
            Direction::Us2Ru => Self::UsToRu,
            Direction::Ru2Us => Self::RuToUs,
        }
    }

    const fn as_dict_direction(self) -> Direction {
        match self {
            Self::UsToRu => Direction::Us2Ru,
            Self::RuToUs => Direction::Ru2Us,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum TraversalScope {
    Phase7A,
    Phase7B,
    Phase7C,
    Phase7D,
}

impl TraversalScope {
    const fn admits_phase7b(self) -> bool {
        matches!(self, Self::Phase7B | Self::Phase7C | Self::Phase7D)
    }

    const fn admits_phase7c(self) -> bool {
        matches!(self, Self::Phase7C | Self::Phase7D)
    }

    const fn admits_phase7d(self) -> bool {
        matches!(self, Self::Phase7D)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct L1TypedQueryField {
    lexical_symbols: Box<[u32]>,
    raw_layout_symbols: Box<[u32]>,
    layout_direction: LayoutDirection,
    leading_punctuation_len: u8,
    trailing_punctuation_start: u8,
    trailing_punctuation_len: u8,
}

impl L1TypedQueryField {
    fn encode(raw: &str) -> Result<Self, String> {
        let raw = raw.trim().to_lowercase();
        let raw_chars = raw.chars().collect::<Vec<_>>();
        let leading_punctuation_len = raw_chars
            .iter()
            .take_while(|character| is_boundary_punctuation(**character))
            .count();
        let trailing_punctuation_len = raw_chars
            .iter()
            .rev()
            .take_while(|character| is_boundary_punctuation(**character))
            .count();
        let trailing_punctuation_start = raw_chars.len().saturating_sub(trailing_punctuation_len);
        let as_u8 = |value: usize, name: &str| {
            u8::try_from(value).map_err(|_| format!("{name} exceeds the typed query limit"))
        };
        let layout_direction = LayoutDirection::from_raw(&raw);
        Ok(Self {
            lexical_symbols: normalize_lexical_surface(&raw)
                .chars()
                .map(|character| character as u32)
                .collect(),
            raw_layout_symbols: raw_chars
                .into_iter()
                .map(|character| character as u32)
                .collect(),
            layout_direction,
            leading_punctuation_len: as_u8(leading_punctuation_len, "leading punctuation")?,
            trailing_punctuation_start: as_u8(
                trailing_punctuation_start,
                "trailing punctuation start",
            )?,
            trailing_punctuation_len: as_u8(trailing_punctuation_len, "trailing punctuation")?,
        })
    }
}

fn is_boundary_punctuation(character: char) -> bool {
    matches!(character, '!' | ',' | '.' | '?' | ';' | ':')
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
enum TypedCertificate {
    Identity,
    PunctuationSuffix {
        raw_start: u8,
        raw_len: u8,
    },
    PrefixTruncation {
        target_position: u8,
    },
    SuffixTruncation {
        target_position: u8,
    },
    MissingLetter {
        target_position: u8,
    },
    ExtraLetter {
        input_position: u8,
    },
    SingleSubstitution {
        position: u8,
    },
    KeyboardLayout {
        direction: LayoutDirection,
    },
    AdjacentTransposition {
        position: u8,
    },
    NonAdjacentTransposition {
        first: u8,
        second: u8,
    },
    RepeatedFragment {
        input_start: u8,
        source_target_start: u8,
        len: u8,
    },
    DoubleSubstitution {
        first: u8,
        second: u8,
    },
    SparseMultiOmission {
        first: u8,
        second: u8,
    },
    OmissionTransposition {
        omitted_target: u8,
        transposed_target: u8,
    },
}

impl TypedCertificate {
    fn class(&self) -> &'static str {
        match self {
            Self::Identity => "identity",
            Self::PunctuationSuffix { .. } => "punctuation_suffix",
            Self::PrefixTruncation { .. } => "prefix_truncation",
            Self::SuffixTruncation { .. } => "suffix_truncation",
            Self::MissingLetter { .. } => "missing_letter",
            Self::ExtraLetter { .. } => "extra_letter",
            Self::SingleSubstitution { .. } => "letter_substitution",
            Self::KeyboardLayout { .. } => "layout_projection",
            Self::AdjacentTransposition { .. } => "adjacent_transposition",
            Self::NonAdjacentTransposition { .. } => "non_adjacent_transposition",
            Self::RepeatedFragment { .. } => "repeated_fragment",
            Self::DoubleSubstitution { .. } => "double_substitution",
            Self::SparseMultiOmission { .. } => "sparse_multi_omission",
            Self::OmissionTransposition { .. } => "omission_transposition",
        }
    }
}

type TerminalEvents = BTreeMap<u32, BTreeSet<TypedCertificate>>;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum LexicalEditState {
    None,
    TargetInsertion {
        target_position: u8,
    },
    InputDeletion {
        input_position: u8,
    },
    Substitution {
        position: u8,
    },
    AdjacentTransposition {
        position: u8,
    },
    NonAdjacentPending {
        first_position: u8,
        saved_target_symbol: u32,
        saved_observed_symbol: u32,
    },
    NonAdjacentTransposition {
        first: u8,
        second: u8,
    },
    RepeatedFragment {
        input_start: u8,
        source_target_start: u8,
    },
    DoubleSubstitutionPending {
        first: u8,
    },
    DoubleSubstitution {
        first: u8,
        second: u8,
    },
    SparseMultiOmissionPending {
        first: u8,
    },
    SparseMultiOmission {
        first: u8,
        second: u8,
    },
    OmissionBeforeTransposition {
        omitted_target: u8,
    },
    TranspositionBeforeOmission {
        transposed_target: u8,
    },
    OmissionTransposition {
        omitted_target: u8,
        transposed_target: u8,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum OperatorProgram {
    Lexical(LexicalEditState),
    Layout {
        direction: LayoutDirection,
        changed: bool,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct TraversalState {
    decoder_node: u32,
    input_position: u8,
    target_depth: u8,
    program: OperatorProgram,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum TraversalSchedule {
    Forward,
    Reverse,
    Permuted(u64),
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize)]
struct TraversalMetrics {
    states_generated: u64,
    states_deduplicated: u64,
    states_expanded: u64,
    queue_peak: usize,
    decoder_edges_examined: u64,
    terminal_nodes_reached: u64,
    wordcenter_terminal_events: u64,
    unique_wordcenter_ids: usize,
    certificates_emitted: usize,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
struct TraversalResult {
    terminals: TerminalEvents,
    metrics: TraversalMetrics,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct Phase7dTerminalEvidence {
    pub(super) terminal_ids: Vec<u32>,
    pub(super) states_expanded: u64,
    pub(super) queue_peak: usize,
    pub(super) terminal_events: u64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct Phase7dTerminalDiagnosticEvidence {
    pub(super) terminal: Phase7dTerminalEvidence,
    pub(super) certificate_classes: BTreeMap<u32, Vec<&'static str>>,
}

struct L1TypedEditTraversal<'a> {
    index: &'a ForwardDecoderIndex,
    decoder_nodes: &'a [DecoderNode],
}

impl<'a> L1TypedEditTraversal<'a> {
    fn traverse(
        &self,
        query: &L1TypedQueryField,
        scope: TraversalScope,
        schedule: TraversalSchedule,
    ) -> TraversalResult {
        let lexical_root = TraversalState {
            decoder_node: 0,
            input_position: 0,
            target_depth: 0,
            program: OperatorProgram::Lexical(LexicalEditState::None),
        };
        let mut frontier = vec![lexical_root];
        if scope.admits_phase7b() {
            frontier.push(TraversalState {
                decoder_node: 0,
                input_position: 0,
                target_depth: 0,
                program: OperatorProgram::Layout {
                    direction: query.layout_direction,
                    changed: false,
                },
            });
        }
        let mut seen = frontier.iter().copied().collect::<BTreeSet<_>>();
        let mut terminals = TerminalEvents::new();
        let mut metrics = TraversalMetrics {
            states_generated: frontier.len() as u64,
            queue_peak: frontier.len(),
            ..TraversalMetrics::default()
        };

        while !frontier.is_empty() {
            reorder_states(&mut frontier, schedule);
            let mut next = Vec::new();
            for state in frontier.drain(..) {
                metrics.states_expanded += 1;
                self.record_terminal_events(query, scope, state, &mut terminals, &mut metrics);
                match state.program {
                    OperatorProgram::Lexical(edit) => self.advance_lexical(
                        query,
                        scope,
                        schedule,
                        state,
                        edit,
                        &mut seen,
                        &mut next,
                        &mut metrics,
                    ),
                    OperatorProgram::Layout { direction, changed } => self.advance_layout(
                        query,
                        state,
                        direction,
                        changed,
                        &mut seen,
                        &mut next,
                        &mut metrics,
                    ),
                }
            }
            frontier = next;
            metrics.queue_peak = metrics.queue_peak.max(frontier.len());
        }

        metrics.unique_wordcenter_ids = terminals.len();
        metrics.certificates_emitted = terminals.values().map(BTreeSet::len).sum();
        TraversalResult { terminals, metrics }
    }

    #[allow(clippy::too_many_arguments)]
    fn advance_lexical(
        &self,
        query: &L1TypedQueryField,
        scope: TraversalScope,
        schedule: TraversalSchedule,
        state: TraversalState,
        edit: LexicalEditState,
        seen: &mut BTreeSet<TraversalState>,
        next: &mut Vec<TraversalState>,
        metrics: &mut TraversalMetrics,
    ) {
        let observed = query
            .lexical_symbols
            .get(usize::from(state.input_position))
            .copied();
        if let Some(symbol) = observed {
            metrics.decoder_edges_examined += 1;
            if let Some(child) = self.index.child(state.decoder_node, symbol) {
                self.push_state(
                    TraversalState {
                        decoder_node: child,
                        input_position: state.input_position + 1,
                        target_depth: state.target_depth + 1,
                        program: OperatorProgram::Lexical(edit),
                    },
                    seen,
                    next,
                    metrics,
                );
            }
        }

        if let LexicalEditState::NonAdjacentPending {
            first_position,
            saved_target_symbol,
            saved_observed_symbol,
        } = edit
        {
            if state.input_position >= first_position.saturating_add(2)
                && observed == Some(saved_target_symbol)
            {
                metrics.decoder_edges_examined += 1;
                if let Some(child) = self.index.child(state.decoder_node, saved_observed_symbol) {
                    self.push_state(
                        TraversalState {
                            decoder_node: child,
                            input_position: state.input_position + 1,
                            target_depth: state.target_depth + 1,
                            program: OperatorProgram::Lexical(
                                LexicalEditState::NonAdjacentTransposition {
                                    first: first_position,
                                    second: state.target_depth,
                                },
                            ),
                        },
                        seen,
                        next,
                        metrics,
                    );
                }
            }
            return;
        }
        if let LexicalEditState::DoubleSubstitutionPending { first } = edit {
            let Some(observed_symbol) = observed else {
                return;
            };
            let children = ordered_children(self.index.children(state.decoder_node), schedule);
            metrics.decoder_edges_examined += children.len() as u64;
            for child in children
                .iter()
                .copied()
                .filter(|child| child.symbol != observed_symbol)
            {
                self.push_state(
                    TraversalState {
                        decoder_node: child.node_id,
                        input_position: state.input_position + 1,
                        target_depth: state.target_depth + 1,
                        program: OperatorProgram::Lexical(LexicalEditState::DoubleSubstitution {
                            first,
                            second: state.target_depth,
                        }),
                    },
                    seen,
                    next,
                    metrics,
                );
            }
            return;
        }
        if let LexicalEditState::SparseMultiOmissionPending { first } = edit {
            let children = ordered_children(self.index.children(state.decoder_node), schedule);
            metrics.decoder_edges_examined += children.len() as u64;
            for child in children.iter().copied() {
                self.push_state(
                    TraversalState {
                        decoder_node: child.node_id,
                        input_position: state.input_position,
                        target_depth: state.target_depth + 1,
                        program: OperatorProgram::Lexical(LexicalEditState::SparseMultiOmission {
                            first,
                            second: state.target_depth,
                        }),
                    },
                    seen,
                    next,
                    metrics,
                );
            }
            return;
        }
        if let LexicalEditState::OmissionBeforeTransposition { omitted_target } = edit {
            if let (Some(first), Some(second)) = (
                observed,
                query
                    .lexical_symbols
                    .get(usize::from(state.input_position) + 1)
                    .copied(),
            ) {
                if first != second {
                    metrics.decoder_edges_examined += 1;
                    if let Some(first_child) = self.index.child(state.decoder_node, second) {
                        metrics.decoder_edges_examined += 1;
                        if let Some(second_child) = self.index.child(first_child, first) {
                            self.push_state(
                                TraversalState {
                                    decoder_node: second_child,
                                    input_position: state.input_position + 2,
                                    target_depth: state.target_depth + 2,
                                    program: OperatorProgram::Lexical(
                                        LexicalEditState::OmissionTransposition {
                                            omitted_target,
                                            transposed_target: state.target_depth,
                                        },
                                    ),
                                },
                                seen,
                                next,
                                metrics,
                            );
                        }
                    }
                }
            }
            return;
        }
        if let LexicalEditState::TranspositionBeforeOmission { transposed_target } = edit {
            let children = ordered_children(self.index.children(state.decoder_node), schedule);
            metrics.decoder_edges_examined += children.len() as u64;
            for child in children.iter().copied() {
                self.push_state(
                    TraversalState {
                        decoder_node: child.node_id,
                        input_position: state.input_position,
                        target_depth: state.target_depth + 1,
                        program: OperatorProgram::Lexical(
                            LexicalEditState::OmissionTransposition {
                                omitted_target: state.target_depth,
                                transposed_target,
                            },
                        ),
                    },
                    seen,
                    next,
                    metrics,
                );
            }
            return;
        }
        if edit != LexicalEditState::None {
            return;
        }
        if scope.admits_phase7b() && observed.is_some() {
            self.push_state(
                TraversalState {
                    decoder_node: state.decoder_node,
                    input_position: state.input_position + 1,
                    target_depth: state.target_depth,
                    program: OperatorProgram::Lexical(LexicalEditState::InputDeletion {
                        input_position: state.input_position,
                    }),
                },
                seen,
                next,
                metrics,
            );
        }
        if scope.admits_phase7c() {
            if let (Some(first), Some(second)) = (
                observed,
                query
                    .lexical_symbols
                    .get(usize::from(state.input_position) + 1)
                    .copied(),
            ) {
                if first != second {
                    metrics.decoder_edges_examined += 1;
                    if let Some(first_child) = self.index.child(state.decoder_node, second) {
                        metrics.decoder_edges_examined += 1;
                        if let Some(second_child) = self.index.child(first_child, first) {
                            self.push_state(
                                TraversalState {
                                    decoder_node: second_child,
                                    input_position: state.input_position + 2,
                                    target_depth: state.target_depth + 2,
                                    program: OperatorProgram::Lexical(
                                        LexicalEditState::AdjacentTransposition {
                                            position: state.target_depth,
                                        },
                                    ),
                                },
                                seen,
                                next,
                                metrics,
                            );
                            if scope.admits_phase7d() {
                                self.push_state(
                                    TraversalState {
                                        decoder_node: second_child,
                                        input_position: state.input_position + 2,
                                        target_depth: state.target_depth + 2,
                                        program: OperatorProgram::Lexical(
                                            LexicalEditState::TranspositionBeforeOmission {
                                                transposed_target: state.target_depth,
                                            },
                                        ),
                                    },
                                    seen,
                                    next,
                                    metrics,
                                );
                            }
                        }
                    }
                }

                for source_target_start in
                    self.prefix_pair_sources(state.decoder_node, state.target_depth, first, second)
                {
                    self.push_state(
                        TraversalState {
                            decoder_node: state.decoder_node,
                            input_position: state.input_position + 2,
                            target_depth: state.target_depth,
                            program: OperatorProgram::Lexical(LexicalEditState::RepeatedFragment {
                                input_start: state.input_position,
                                source_target_start,
                            }),
                        },
                        seen,
                        next,
                        metrics,
                    );
                }
            }
        }

        if scope.admits_phase7d() {
            if let (Some(first), Some(second)) = (
                observed,
                query
                    .lexical_symbols
                    .get(usize::from(state.input_position) + 1)
                    .copied(),
            ) {
                if first != second {
                    metrics.decoder_edges_examined += 1;
                    if let Some(left) = self.index.child(state.decoder_node, second) {
                        let omitted_children =
                            ordered_children(self.index.children(left), schedule);
                        metrics.decoder_edges_examined += omitted_children.len() as u64;
                        for omitted in omitted_children.iter().copied() {
                            metrics.decoder_edges_examined += 1;
                            if let Some(right) = self.index.child(omitted.node_id, first) {
                                self.push_state(
                                    TraversalState {
                                        decoder_node: right,
                                        input_position: state.input_position + 2,
                                        target_depth: state.target_depth + 3,
                                        program: OperatorProgram::Lexical(
                                            LexicalEditState::OmissionTransposition {
                                                omitted_target: state.target_depth + 1,
                                                transposed_target: state.target_depth,
                                            },
                                        ),
                                    },
                                    seen,
                                    next,
                                    metrics,
                                );
                            }
                        }
                    }
                }
            }
        }

        let children = ordered_children(self.index.children(state.decoder_node), schedule);
        metrics.decoder_edges_examined += children.len() as u64;
        for child in children.iter().copied() {
            self.push_state(
                TraversalState {
                    decoder_node: child.node_id,
                    input_position: state.input_position,
                    target_depth: state.target_depth + 1,
                    program: OperatorProgram::Lexical(LexicalEditState::TargetInsertion {
                        target_position: state.target_depth,
                    }),
                },
                seen,
                next,
                metrics,
            );
            if scope.admits_phase7d() {
                self.push_state(
                    TraversalState {
                        decoder_node: child.node_id,
                        input_position: state.input_position,
                        target_depth: state.target_depth + 1,
                        program: OperatorProgram::Lexical(
                            LexicalEditState::SparseMultiOmissionPending {
                                first: state.target_depth,
                            },
                        ),
                    },
                    seen,
                    next,
                    metrics,
                );
                self.push_state(
                    TraversalState {
                        decoder_node: child.node_id,
                        input_position: state.input_position,
                        target_depth: state.target_depth + 1,
                        program: OperatorProgram::Lexical(
                            LexicalEditState::OmissionBeforeTransposition {
                                omitted_target: state.target_depth,
                            },
                        ),
                    },
                    seen,
                    next,
                    metrics,
                );
            }
            if scope.admits_phase7b() && observed.is_some_and(|symbol| symbol != child.symbol) {
                self.push_state(
                    TraversalState {
                        decoder_node: child.node_id,
                        input_position: state.input_position + 1,
                        target_depth: state.target_depth + 1,
                        program: OperatorProgram::Lexical(LexicalEditState::Substitution {
                            position: state.target_depth,
                        }),
                    },
                    seen,
                    next,
                    metrics,
                );
                if scope.admits_phase7d()
                    && usize::from(state.input_position) + 1 < query.lexical_symbols.len()
                {
                    self.push_state(
                        TraversalState {
                            decoder_node: child.node_id,
                            input_position: state.input_position + 1,
                            target_depth: state.target_depth + 1,
                            program: OperatorProgram::Lexical(
                                LexicalEditState::DoubleSubstitutionPending {
                                    first: state.target_depth,
                                },
                            ),
                        },
                        seen,
                        next,
                        metrics,
                    );
                }
            }
            if let Some(observed_symbol) = observed.filter(|symbol| {
                scope.admits_phase7c()
                    && usize::from(state.input_position) + 2 < query.lexical_symbols.len()
                    && *symbol != child.symbol
            }) {
                self.push_state(
                    TraversalState {
                        decoder_node: child.node_id,
                        input_position: state.input_position + 1,
                        target_depth: state.target_depth + 1,
                        program: OperatorProgram::Lexical(LexicalEditState::NonAdjacentPending {
                            first_position: state.target_depth,
                            saved_target_symbol: child.symbol,
                            saved_observed_symbol: observed_symbol,
                        }),
                    },
                    seen,
                    next,
                    metrics,
                );
            }
        }
    }

    fn prefix_pair_sources(
        &self,
        decoder_node: u32,
        target_depth: u8,
        first: u32,
        second: u32,
    ) -> Vec<u8> {
        if target_depth < 2 {
            return Vec::new();
        }
        let mut sources = Vec::new();
        let mut right_node = decoder_node;
        let mut right_position = target_depth - 1;
        while right_position > 0 {
            let Some(right) = self.decoder_nodes.get(right_node as usize) else {
                break;
            };
            let left_node = right.parent;
            let Some(left) = self.decoder_nodes.get(left_node as usize) else {
                break;
            };
            if left.symbol == first && right.symbol == second {
                sources.push(right_position - 1);
            }
            right_node = left_node;
            right_position -= 1;
        }
        sources
    }

    #[allow(clippy::too_many_arguments)]
    fn advance_layout(
        &self,
        query: &L1TypedQueryField,
        state: TraversalState,
        direction: LayoutDirection,
        changed: bool,
        seen: &mut BTreeSet<TraversalState>,
        next: &mut Vec<TraversalState>,
        metrics: &mut TraversalMetrics,
    ) {
        let Some(&raw_symbol) = query
            .raw_layout_symbols
            .get(usize::from(state.input_position))
        else {
            return;
        };
        let Some(raw_character) = char::from_u32(raw_symbol) else {
            return;
        };
        let projected = project_char(raw_character, direction.as_dict_direction()) as u32;
        metrics.decoder_edges_examined += 1;
        if let Some(child) = self.index.child(state.decoder_node, projected) {
            self.push_state(
                TraversalState {
                    decoder_node: child,
                    input_position: state.input_position + 1,
                    target_depth: state.target_depth + 1,
                    program: OperatorProgram::Layout {
                        direction,
                        changed: changed || projected != raw_symbol,
                    },
                },
                seen,
                next,
                metrics,
            );
        }
    }

    fn push_state(
        &self,
        state: TraversalState,
        seen: &mut BTreeSet<TraversalState>,
        next: &mut Vec<TraversalState>,
        metrics: &mut TraversalMetrics,
    ) {
        metrics.states_generated += 1;
        if seen.insert(state) {
            next.push(state);
        } else {
            metrics.states_deduplicated += 1;
        }
    }

    fn record_terminal_events(
        &self,
        query: &L1TypedQueryField,
        scope: TraversalScope,
        state: TraversalState,
        terminals: &mut TerminalEvents,
        metrics: &mut TraversalMetrics,
    ) {
        let input_len = match state.program {
            OperatorProgram::Lexical(_) => query.lexical_symbols.len(),
            OperatorProgram::Layout { .. } => query.raw_layout_symbols.len(),
        };
        if usize::from(state.input_position) != input_len {
            return;
        }
        let center_ids = self.index.terminals(state.decoder_node);
        if center_ids.is_empty() {
            return;
        }
        let mut certificates = BTreeSet::new();
        match state.program {
            OperatorProgram::Lexical(LexicalEditState::None) => {
                certificates.insert(TypedCertificate::Identity);
                if query.trailing_punctuation_len > 0 {
                    certificates.insert(TypedCertificate::PunctuationSuffix {
                        raw_start: query.trailing_punctuation_start,
                        raw_len: query.trailing_punctuation_len,
                    });
                }
            }
            OperatorProgram::Lexical(LexicalEditState::TargetInsertion { target_position: 0 }) => {
                let target_position = 0;
                certificates.insert(TypedCertificate::PrefixTruncation { target_position });
            }
            OperatorProgram::Lexical(LexicalEditState::TargetInsertion { target_position })
                if target_position + 1 == state.target_depth =>
            {
                certificates.insert(TypedCertificate::SuffixTruncation { target_position });
            }
            OperatorProgram::Lexical(LexicalEditState::TargetInsertion { target_position })
                if scope.admits_phase7b() =>
            {
                certificates.insert(TypedCertificate::MissingLetter { target_position });
            }
            OperatorProgram::Lexical(LexicalEditState::InputDeletion { input_position }) => {
                certificates.insert(TypedCertificate::ExtraLetter { input_position });
            }
            OperatorProgram::Lexical(LexicalEditState::Substitution { position }) => {
                certificates.insert(TypedCertificate::SingleSubstitution { position });
            }
            OperatorProgram::Lexical(LexicalEditState::AdjacentTransposition { position })
                if scope.admits_phase7c() =>
            {
                certificates.insert(TypedCertificate::AdjacentTransposition { position });
            }
            OperatorProgram::Lexical(LexicalEditState::NonAdjacentTransposition {
                first,
                second,
            }) if scope.admits_phase7c() => {
                certificates.insert(TypedCertificate::NonAdjacentTransposition { first, second });
            }
            OperatorProgram::Lexical(LexicalEditState::RepeatedFragment {
                input_start,
                source_target_start,
            }) if scope.admits_phase7c() => {
                certificates.insert(TypedCertificate::RepeatedFragment {
                    input_start,
                    source_target_start,
                    len: 2,
                });
            }
            OperatorProgram::Lexical(LexicalEditState::DoubleSubstitution { first, second })
                if scope.admits_phase7d() =>
            {
                certificates.insert(TypedCertificate::DoubleSubstitution { first, second });
            }
            OperatorProgram::Lexical(LexicalEditState::SparseMultiOmission { first, second })
                if scope.admits_phase7d() =>
            {
                certificates.insert(TypedCertificate::SparseMultiOmission { first, second });
            }
            OperatorProgram::Lexical(LexicalEditState::OmissionTransposition {
                omitted_target,
                transposed_target,
            }) if scope.admits_phase7d() => {
                certificates.insert(TypedCertificate::OmissionTransposition {
                    omitted_target,
                    transposed_target,
                });
            }
            OperatorProgram::Layout {
                direction,
                changed: true,
            } => {
                certificates.insert(TypedCertificate::KeyboardLayout { direction });
            }
            _ => {}
        }
        if certificates.is_empty() {
            return;
        }
        metrics.terminal_nodes_reached += 1;
        for &center_id in center_ids {
            let events = terminals.entry(center_id).or_default();
            for certificate in &certificates {
                metrics.wordcenter_terminal_events +=
                    usize::from(events.insert(certificate.clone())) as u64;
            }
        }
    }
}

pub(super) fn phase7d_terminal_evidence(
    index: &ForwardDecoderIndex,
    decoder_nodes: &[DecoderNode],
    surface: &str,
) -> Result<Phase7dTerminalEvidence, String> {
    let result = traverse_phase7d(index, decoder_nodes, surface)?;
    Ok(project_phase7d_terminal_evidence(&result))
}

pub(super) fn phase7d_terminal_diagnostic_evidence(
    index: &ForwardDecoderIndex,
    decoder_nodes: &[DecoderNode],
    surface: &str,
) -> Result<Phase7dTerminalDiagnosticEvidence, String> {
    let result = traverse_phase7d(index, decoder_nodes, surface)?;
    let terminal = project_phase7d_terminal_evidence(&result);
    let certificate_classes = result
        .terminals
        .iter()
        .map(|(terminal_id, certificates)| {
            let classes = certificates
                .iter()
                .map(TypedCertificate::class)
                .collect::<BTreeSet<_>>()
                .into_iter()
                .collect();
            (*terminal_id, classes)
        })
        .collect();
    Ok(Phase7dTerminalDiagnosticEvidence {
        terminal,
        certificate_classes,
    })
}

fn traverse_phase7d(
    index: &ForwardDecoderIndex,
    decoder_nodes: &[DecoderNode],
    surface: &str,
) -> Result<TraversalResult, String> {
    let query = L1TypedQueryField::encode(surface)?;
    Ok(L1TypedEditTraversal {
        index,
        decoder_nodes,
    }
    .traverse(&query, TraversalScope::Phase7D, TraversalSchedule::Forward))
}

fn project_phase7d_terminal_evidence(result: &TraversalResult) -> Phase7dTerminalEvidence {
    Phase7dTerminalEvidence {
        terminal_ids: result.terminals.keys().copied().collect(),
        states_expanded: result.metrics.states_expanded,
        queue_peak: result.metrics.queue_peak,
        terminal_events: result.metrics.wordcenter_terminal_events,
    }
}

fn reorder_states(states: &mut [TraversalState], schedule: TraversalSchedule) {
    match schedule {
        TraversalSchedule::Forward => states.sort_unstable(),
        TraversalSchedule::Reverse => states.sort_unstable_by(|left, right| right.cmp(left)),
        TraversalSchedule::Permuted(seed) => states.sort_unstable_by_key(|state| {
            permutation_key(
                seed ^ u64::from(state.decoder_node)
                    ^ u64::from(state.input_position).rotate_left(17)
                    ^ u64::from(state.target_depth).rotate_left(31)
                    ^ operator_program_key(state.program).rotate_left(43),
            )
        }),
    }
}

fn operator_program_key(program: OperatorProgram) -> u64 {
    match program {
        OperatorProgram::Lexical(LexicalEditState::None) => 0,
        OperatorProgram::Lexical(LexicalEditState::TargetInsertion { target_position }) => {
            1 | (u64::from(target_position) << 8)
        }
        OperatorProgram::Lexical(LexicalEditState::InputDeletion { input_position }) => {
            2 | (u64::from(input_position) << 8)
        }
        OperatorProgram::Lexical(LexicalEditState::Substitution { position }) => {
            3 | (u64::from(position) << 8)
        }
        OperatorProgram::Lexical(LexicalEditState::AdjacentTransposition { position }) => {
            4 | (u64::from(position) << 8)
        }
        OperatorProgram::Lexical(LexicalEditState::NonAdjacentPending {
            first_position,
            saved_target_symbol,
            saved_observed_symbol,
        }) => {
            5 | (u64::from(first_position) << 8)
                | (u64::from(saved_target_symbol) << 16)
                    ^ u64::from(saved_observed_symbol).rotate_left(37)
        }
        OperatorProgram::Lexical(LexicalEditState::NonAdjacentTransposition { first, second }) => {
            6 | (u64::from(first) << 8) | (u64::from(second) << 16)
        }
        OperatorProgram::Lexical(LexicalEditState::RepeatedFragment {
            input_start,
            source_target_start,
        }) => 7 | (u64::from(input_start) << 8) | (u64::from(source_target_start) << 16),
        OperatorProgram::Lexical(LexicalEditState::DoubleSubstitutionPending { first }) => {
            8 | (u64::from(first) << 8)
        }
        OperatorProgram::Lexical(LexicalEditState::DoubleSubstitution { first, second }) => {
            9 | (u64::from(first) << 8) | (u64::from(second) << 16)
        }
        OperatorProgram::Lexical(LexicalEditState::SparseMultiOmissionPending { first }) => {
            10 | (u64::from(first) << 8)
        }
        OperatorProgram::Lexical(LexicalEditState::SparseMultiOmission { first, second }) => {
            11 | (u64::from(first) << 8) | (u64::from(second) << 16)
        }
        OperatorProgram::Lexical(LexicalEditState::OmissionBeforeTransposition {
            omitted_target,
        }) => 12 | (u64::from(omitted_target) << 8),
        OperatorProgram::Lexical(LexicalEditState::TranspositionBeforeOmission {
            transposed_target,
        }) => 13 | (u64::from(transposed_target) << 8),
        OperatorProgram::Lexical(LexicalEditState::OmissionTransposition {
            omitted_target,
            transposed_target,
        }) => 14 | (u64::from(omitted_target) << 8) | (u64::from(transposed_target) << 16),
        OperatorProgram::Layout { direction, changed } => {
            15 | ((direction as u64) << 8) | ((changed as u64) << 16)
        }
    }
}

fn reorder_children(children: &mut [ForwardChild], schedule: TraversalSchedule) {
    match schedule {
        TraversalSchedule::Forward => {}
        TraversalSchedule::Reverse => children.reverse(),
        TraversalSchedule::Permuted(seed) => children.sort_unstable_by_key(|child| {
            permutation_key(
                seed ^ u64::from(child.node_id) ^ u64::from(child.symbol).rotate_left(23),
            )
        }),
    }
}

fn ordered_children(
    children: &[ForwardChild],
    schedule: TraversalSchedule,
) -> Cow<'_, [ForwardChild]> {
    if schedule == TraversalSchedule::Forward {
        return Cow::Borrowed(children);
    }
    let mut ordered = children.to_vec();
    reorder_children(&mut ordered, schedule);
    Cow::Owned(ordered)
}

fn permutation_key(mut value: u64) -> u64 {
    value ^= value >> 30;
    value = value.wrapping_mul(0xbf58_476d_1ce4_e5b9);
    value ^= value >> 27;
    value = value.wrapping_mul(0x94d0_49bb_1331_11eb);
    value ^ (value >> 31)
}

#[cfg(any(test, feature = "lexical-compiler"))]
fn direct_typed_oracle(
    package: &LexicalGrokkingPackage,
    query: &L1TypedQueryField,
    scope: TraversalScope,
) -> Result<TerminalEvents, String> {
    let mut events = TerminalEvents::new();
    for (center_id, center) in package.centers.iter().copied().enumerate() {
        let target = format::decode_center_surface(center, &package.decoder_nodes)?;
        let target = normalize_lexical_surface(&target)
            .chars()
            .map(|character| character as u32)
            .collect::<Vec<_>>();
        let mut certificates = BTreeSet::new();
        if target.as_slice() == query.lexical_symbols.as_ref() {
            certificates.insert(TypedCertificate::Identity);
            if query.trailing_punctuation_len > 0 {
                certificates.insert(TypedCertificate::PunctuationSuffix {
                    raw_start: query.trailing_punctuation_start,
                    raw_len: query.trailing_punctuation_len,
                });
            }
        }
        if target.len() == query.lexical_symbols.len() + 1 {
            for target_position in 0..target.len() {
                if target
                    .iter()
                    .enumerate()
                    .filter_map(|(index, symbol)| (index != target_position).then_some(*symbol))
                    .eq(query.lexical_symbols.iter().copied())
                {
                    let target_position = target_position as u8;
                    if target_position == 0 {
                        certificates.insert(TypedCertificate::PrefixTruncation { target_position });
                    } else if usize::from(target_position) + 1 == target.len() {
                        certificates.insert(TypedCertificate::SuffixTruncation { target_position });
                    } else if scope.admits_phase7b() {
                        certificates.insert(TypedCertificate::MissingLetter { target_position });
                    }
                }
            }
            if scope.admits_phase7d() {
                for transposed_target in 0..target.len().saturating_sub(1) {
                    if target[transposed_target] == target[transposed_target + 1] {
                        continue;
                    }
                    for omitted_target in 0..target.len() {
                        if omitted_target == transposed_target
                            || omitted_target == transposed_target + 1
                        {
                            continue;
                        }
                        let matches = (0..target.len())
                            .filter(|position| *position != omitted_target)
                            .map(|position| {
                                if position == transposed_target {
                                    target[transposed_target + 1]
                                } else if position == transposed_target + 1 {
                                    target[transposed_target]
                                } else {
                                    target[position]
                                }
                            })
                            .eq(query.lexical_symbols.iter().copied());
                        if matches {
                            certificates.insert(TypedCertificate::OmissionTransposition {
                                omitted_target: omitted_target as u8,
                                transposed_target: transposed_target as u8,
                            });
                        }
                    }
                }
                for transposed_target in 0..target.len().saturating_sub(2) {
                    let omitted_target = transposed_target + 1;
                    if target[transposed_target] == target[transposed_target + 2] {
                        continue;
                    }
                    let matches = target[..transposed_target]
                        .iter()
                        .copied()
                        .chain(std::iter::once(target[transposed_target + 2]))
                        .chain(std::iter::once(target[transposed_target]))
                        .chain(target[transposed_target + 3..].iter().copied())
                        .eq(query.lexical_symbols.iter().copied());
                    if matches {
                        certificates.insert(TypedCertificate::OmissionTransposition {
                            omitted_target: omitted_target as u8,
                            transposed_target: transposed_target as u8,
                        });
                    }
                }
            }
        }
        if scope.admits_phase7b() && query.lexical_symbols.len() == target.len() + 1 {
            for input_position in 0..query.lexical_symbols.len() {
                if query
                    .lexical_symbols
                    .iter()
                    .enumerate()
                    .filter_map(|(index, symbol)| (index != input_position).then_some(*symbol))
                    .eq(target.iter().copied())
                {
                    certificates.insert(TypedCertificate::ExtraLetter {
                        input_position: input_position as u8,
                    });
                }
            }
        }
        if scope.admits_phase7b() && query.lexical_symbols.len() == target.len() {
            let mismatches = query
                .lexical_symbols
                .iter()
                .zip(&target)
                .enumerate()
                .filter_map(|(position, (observed, expected))| {
                    (observed != expected).then_some(position)
                })
                .collect::<Vec<_>>();
            if let [position] = mismatches.as_slice() {
                certificates.insert(TypedCertificate::SingleSubstitution {
                    position: *position as u8,
                });
            }
            if scope.admits_phase7d() {
                if let [first, second] = mismatches.as_slice() {
                    certificates.insert(TypedCertificate::DoubleSubstitution {
                        first: *first as u8,
                        second: *second as u8,
                    });
                }
            }
            if scope.admits_phase7c() {
                if let [first, second] = mismatches.as_slice() {
                    if query.lexical_symbols[*first] == target[*second]
                        && query.lexical_symbols[*second] == target[*first]
                    {
                        if *second == *first + 1 {
                            certificates.insert(TypedCertificate::AdjacentTransposition {
                                position: *first as u8,
                            });
                        } else if *second > *first + 1 {
                            certificates.insert(TypedCertificate::NonAdjacentTransposition {
                                first: *first as u8,
                                second: *second as u8,
                            });
                        }
                    }
                }
            }
        }
        if scope.admits_phase7b() && query.raw_layout_symbols.len() == target.len() {
            let direction = query.layout_direction;
            let mut changed = false;
            let projected_matches =
                query
                    .raw_layout_symbols
                    .iter()
                    .zip(&target)
                    .all(|(raw, expected)| {
                        let Some(raw_character) = char::from_u32(*raw) else {
                            return false;
                        };
                        let projected =
                            project_char(raw_character, direction.as_dict_direction()) as u32;
                        changed |= projected != *raw;
                        projected == *expected
                    });
            if projected_matches && changed {
                certificates.insert(TypedCertificate::KeyboardLayout { direction });
            }
        }
        if scope.admits_phase7c() && query.lexical_symbols.len() == target.len() + 2 {
            for input_start in 0..=target.len() {
                if !query.lexical_symbols[..input_start]
                    .iter()
                    .chain(&query.lexical_symbols[input_start + 2..])
                    .copied()
                    .eq(target.iter().copied())
                {
                    continue;
                }
                let first = query.lexical_symbols[input_start];
                let second = query.lexical_symbols[input_start + 1];
                for source_target_start in 0..target.len().saturating_sub(1) {
                    if source_target_start + 2 <= input_start
                        && target[source_target_start] == first
                        && target[source_target_start + 1] == second
                    {
                        certificates.insert(TypedCertificate::RepeatedFragment {
                            input_start: input_start as u8,
                            source_target_start: source_target_start as u8,
                            len: 2,
                        });
                    }
                }
            }
        }
        if scope.admits_phase7d() && query.lexical_symbols.len() + 2 == target.len() {
            for first in 0..target.len().saturating_sub(1) {
                for second in first + 1..target.len() {
                    if target
                        .iter()
                        .enumerate()
                        .filter_map(|(position, symbol)| {
                            (position != first && position != second).then_some(*symbol)
                        })
                        .eq(query.lexical_symbols.iter().copied())
                    {
                        certificates.insert(TypedCertificate::SparseMultiOmission {
                            first: first as u8,
                            second: second as u8,
                        });
                    }
                }
            }
        }
        if !certificates.is_empty() {
            events.insert(center_id as u32, certificates);
        }
    }
    Ok(events)
}

#[cfg(any(test, feature = "lexical-compiler"))]
#[derive(Clone, Debug, Default, Serialize)]
struct ClassProof {
    cases: usize,
    target_retained: usize,
    certificate_retained: usize,
    schedule_parity: usize,
    states_p50: u64,
    states_p95: u64,
    states_p99: u64,
    states_max: u64,
    queue_p50: u64,
    queue_p95: u64,
    queue_p99: u64,
    queue_max: u64,
    terminal_events_p50: u64,
    terminal_events_p95: u64,
    terminal_events_p99: u64,
    terminal_events_max: u64,
}

#[cfg(any(test, feature = "lexical-compiler"))]
#[derive(Default)]
struct ClassAccumulator {
    cases: usize,
    target_retained: usize,
    certificate_retained: usize,
    schedule_parity: usize,
    states: Vec<u64>,
    queues: Vec<u64>,
    terminal_events: Vec<u64>,
}

#[cfg(any(test, feature = "lexical-compiler"))]
impl ClassAccumulator {
    fn record(&mut self, case: &FixedHeldoutCase, forward: &TraversalResult, parity: bool) {
        let target = forward.terminals.get(&case.terminal_id);
        self.cases += 1;
        self.target_retained += usize::from(target.is_some());
        self.certificate_retained += usize::from(target.is_some_and(|certificates| {
            certificates
                .iter()
                .any(|certificate| certificate.class() == case.class)
        }));
        self.schedule_parity += usize::from(parity);
        self.states.push(forward.metrics.states_expanded);
        self.queues.push(forward.metrics.queue_peak as u64);
        self.terminal_events
            .push(forward.metrics.wordcenter_terminal_events);
    }

    fn merge(&mut self, source: Self) {
        self.cases += source.cases;
        self.target_retained += source.target_retained;
        self.certificate_retained += source.certificate_retained;
        self.schedule_parity += source.schedule_parity;
        self.states.extend(source.states);
        self.queues.extend(source.queues);
        self.terminal_events.extend(source.terminal_events);
    }

    fn finish(mut self) -> ClassProof {
        self.states.sort_unstable();
        self.queues.sort_unstable();
        self.terminal_events.sort_unstable();
        ClassProof {
            cases: self.cases,
            target_retained: self.target_retained,
            certificate_retained: self.certificate_retained,
            schedule_parity: self.schedule_parity,
            states_p50: percentile(&self.states, 50),
            states_p95: percentile(&self.states, 95),
            states_p99: percentile(&self.states, 99),
            states_max: self.states.last().copied().unwrap_or_default(),
            queue_p50: percentile(&self.queues, 50),
            queue_p95: percentile(&self.queues, 95),
            queue_p99: percentile(&self.queues, 99),
            queue_max: self.queues.last().copied().unwrap_or_default(),
            terminal_events_p50: percentile(&self.terminal_events, 50),
            terminal_events_p95: percentile(&self.terminal_events, 95),
            terminal_events_p99: percentile(&self.terminal_events, 99),
            terminal_events_max: self.terminal_events.last().copied().unwrap_or_default(),
        }
    }
}

#[cfg(any(test, feature = "lexical-compiler"))]
fn percentile(values: &[u64], percentile: usize) -> u64 {
    if values.is_empty() {
        return 0;
    }
    let index = values.len().saturating_sub(1).saturating_mul(percentile) / 100;
    values[index]
}

#[cfg(any(test, feature = "lexical-compiler"))]
pub fn prove_l1_typed_edit_phase7a(
    corpus_path: &Path,
    package_path: &Path,
    max_words: usize,
    heldout_per_class: usize,
) -> io::Result<serde_json::Value> {
    prove_l1_typed_edit_phase(
        corpus_path,
        package_path,
        max_words,
        heldout_per_class,
        TraversalScope::Phase7A,
        &PHASE7A_CLASSES,
        "phase7a",
    )
}

#[cfg(any(test, feature = "lexical-compiler"))]
pub fn prove_l1_typed_edit_phase7b(
    corpus_path: &Path,
    package_path: &Path,
    max_words: usize,
    heldout_per_class: usize,
) -> io::Result<serde_json::Value> {
    prove_l1_typed_edit_phase(
        corpus_path,
        package_path,
        max_words,
        heldout_per_class,
        TraversalScope::Phase7B,
        &PHASE7B_CLASSES,
        "phase7b",
    )
}

#[cfg(any(test, feature = "lexical-compiler"))]
pub fn prove_l1_typed_edit_phase7c(
    corpus_path: &Path,
    package_path: &Path,
    max_words: usize,
    heldout_per_class: usize,
) -> io::Result<serde_json::Value> {
    prove_l1_typed_edit_phase(
        corpus_path,
        package_path,
        max_words,
        heldout_per_class,
        TraversalScope::Phase7C,
        &PHASE7C_CLASSES,
        "phase7c",
    )
}

#[cfg(any(test, feature = "lexical-compiler"))]
pub fn prove_l1_typed_edit_phase7d(
    corpus_path: &Path,
    package_path: &Path,
    max_words: usize,
    heldout_per_class: usize,
) -> io::Result<serde_json::Value> {
    prove_l1_typed_edit_phase(
        corpus_path,
        package_path,
        max_words,
        heldout_per_class,
        TraversalScope::Phase7D,
        &PHASE7D_CLASSES,
        "phase7d",
    )
}

#[cfg(any(test, feature = "lexical-compiler"))]
fn prove_l1_typed_edit_phase(
    corpus_path: &Path,
    package_path: &Path,
    max_words: usize,
    heldout_per_class: usize,
    traversal_scope: TraversalScope,
    proof_classes: &[&'static str],
    phase_name: &'static str,
) -> io::Result<serde_json::Value> {
    let started = Instant::now();
    let package_sha256_before = file_sha256(package_path)?;
    let words = corpus_words_from_lines(&std::fs::read_to_string(corpus_path)?, max_words);
    let memory = LexicalGrokkingMemory::load(package_path).map_err(io::Error::other)?;
    if words.len() != memory.package.centers.len() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!(
                "corpus/package terminal count differs: {} != {}",
                words.len(),
                memory.package.centers.len()
            ),
        ));
    }
    let index = ForwardDecoderIndex::build(&memory.package).map_err(io::Error::other)?;
    let traversal = L1TypedEditTraversal {
        index: &index,
        decoder_nodes: &memory.package.decoder_nodes,
    };

    let clean_started = Instant::now();
    let clean_workers = proof_worker_count(words.len());
    let clean_chunk = words.len().div_ceil(clean_workers);
    let clean = thread::scope(|scope| {
        words
            .chunks(clean_chunk)
            .enumerate()
            .map(|(chunk_index, chunk)| {
                let traversal = &traversal;
                let package = &memory.package;
                scope.spawn(move || -> Result<(usize, usize), String> {
                    let mut dictionary_matches = 0;
                    let mut identities = 0;
                    for (offset, word) in chunk.iter().enumerate() {
                        let terminal_id = chunk_index.saturating_mul(clean_chunk) + offset;
                        let center =
                            package.centers.get(terminal_id).copied().ok_or_else(|| {
                                "clean terminal is absent from package".to_string()
                            })?;
                        let decoded =
                            format::decode_center_surface(center, &package.decoder_nodes)?;
                        dictionary_matches += usize::from(decoded == *word);
                        let query = L1TypedQueryField::encode(word)?;
                        let result =
                            traversal.traverse(&query, traversal_scope, TraversalSchedule::Forward);
                        identities +=
                            usize::from(result.terminals.get(&(terminal_id as u32)).is_some_and(
                                |certificates| certificates.contains(&TypedCertificate::Identity),
                            ));
                    }
                    Ok((dictionary_matches, identities))
                })
            })
            .collect::<Vec<_>>()
            .into_iter()
            .map(|handle| {
                handle
                    .join()
                    .map_err(|_| format!("{phase_name} clean worker panicked"))?
            })
            .collect::<Result<Vec<_>, String>>()
    })
    .map_err(io::Error::other)?;
    let dictionary_matches = clean.iter().map(|item| item.0).sum::<usize>();
    let clean_identity_retained = clean.iter().map(|item| item.1).sum::<usize>();
    let clean_ms = clean_started.elapsed().as_millis();

    let heldout_started = Instant::now();
    let heldout = prepare_fixed_heldout_cases(&words, heldout_per_class, 0)?
        .into_iter()
        .filter(|case| proof_classes.contains(&case.class))
        .collect::<Vec<_>>();
    let heldout_sha256 = sha256_hex(
        heldout
            .iter()
            .flat_map(|case| {
                format!("{}\t{}\t{}\n", case.class, case.terminal_id, case.surface).into_bytes()
            })
            .collect::<Vec<_>>()
            .as_slice(),
    );
    let workers = proof_worker_count(heldout.len());
    let partial = thread::scope(|scope| {
        (0..workers)
            .map(|worker| {
                let traversal = &traversal;
                let heldout = &heldout;
                scope.spawn(
                    move || -> Result<BTreeMap<&'static str, ClassAccumulator>, String> {
                        let mut classes = BTreeMap::<&'static str, ClassAccumulator>::new();
                        for case in heldout.iter().skip(worker).step_by(workers) {
                            let query = L1TypedQueryField::encode(&case.surface)?;
                            let forward = traversal.traverse(
                                &query,
                                traversal_scope,
                                TraversalSchedule::Forward,
                            );
                            let reverse = traversal.traverse(
                                &query,
                                traversal_scope,
                                TraversalSchedule::Reverse,
                            );
                            let permuted = traversal.traverse(
                                &query,
                                traversal_scope,
                                TraversalSchedule::Permuted(0x7a11_c0de),
                            );
                            let parity = forward == reverse && forward == permuted;
                            classes
                                .entry(case.class)
                                .or_default()
                                .record(case, &forward, parity);
                        }
                        Ok(classes)
                    },
                )
            })
            .collect::<Vec<_>>()
            .into_iter()
            .map(|handle| {
                handle
                    .join()
                    .map_err(|_| format!("{phase_name} heldout worker panicked"))?
            })
            .collect::<Result<Vec<_>, String>>()
    })
    .map_err(io::Error::other)?;
    let mut classes = BTreeMap::<&'static str, ClassAccumulator>::new();
    for shard in partial {
        for (class, accumulator) in shard {
            classes.entry(class).or_default().merge(accumulator);
        }
    }
    let classes = classes
        .into_iter()
        .map(|(class, accumulator)| (class, accumulator.finish()))
        .collect::<BTreeMap<_, _>>();
    let heldout_ms = heldout_started.elapsed().as_millis();

    let package_sha256_after = file_sha256(package_path)?;
    let package_unchanged = package_sha256_before == package_sha256_after;
    let all_class_counts_exact = proof_classes.iter().all(|class| {
        classes
            .get(class)
            .is_some_and(|metrics| metrics.cases == heldout_per_class)
    });
    let target_recall_complete = classes
        .values()
        .all(|metrics| metrics.target_retained == metrics.cases);
    let certificate_recall_complete = classes
        .values()
        .all(|metrics| metrics.certificate_retained == metrics.cases);
    let schedule_parity_complete = classes
        .values()
        .all(|metrics| metrics.schedule_parity == metrics.cases);
    let passed = package_unchanged
        && dictionary_matches == words.len()
        && clean_identity_retained == words.len()
        && all_class_counts_exact
        && target_recall_complete
        && certificate_recall_complete
        && schedule_parity_complete;

    Ok(serde_json::json!({
        "schema": format!("lay.l11.typed-edit-{phase_name}-proof.v1"),
        "phase": phase_name,
        "verdict": if passed { "PASS" } else { "FAIL" },
        "corpus": corpus_path,
        "package": package_path,
        "package_sha256_before": package_sha256_before,
        "package_sha256_after": package_sha256_after,
        "package_bytes_unchanged": package_unchanged,
        "index_fingerprint_sha256": index.fingerprint(),
        "primary_centers": memory.package.centers.len(),
        "dictionary_roundtripped": dictionary_matches,
        "clean_identity_cases": words.len(),
        "clean_identity_retained": clean_identity_retained,
        "heldout_per_class": heldout_per_class,
        "heldout_cases": heldout.len(),
        "heldout_sha256": heldout_sha256,
        "classes": classes,
        "all_class_counts_exact": all_class_counts_exact,
        "target_recall_complete": target_recall_complete,
        "certificate_recall_complete": certificate_recall_complete,
        "schedule_parity_complete": schedule_parity_complete,
        "generated_target_strings": 0,
        "queue_truncations": 0,
        "runtime_authority_changed": false,
        "package_format_changed": false,
        "clean_ms": clean_ms,
        "heldout_ms": heldout_ms,
        "wall_ms": started.elapsed().as_millis(),
    }))
}

#[cfg(any(test, feature = "lexical-compiler"))]
fn sha256_hex(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::nanda_wave::lexical_grokking::compiler::compile;
    use crate::nanda_wave::lexical_grokking::training_corpus::TrainingWord;

    fn tiny_package() -> LexicalGrokkingPackage {
        let words = [
            "a", "ab", "aba", "baa", "cab", "caba", "ф", "фи", "б", "abcab",
        ]
        .into_iter()
        .enumerate()
        .map(|(terminal_id, surface)| TrainingWord {
            terminal_id: terminal_id as u32,
            surface: surface.to_string(),
            training_surfaces: Vec::new(),
        })
        .collect::<Vec<_>>();
        compile(&words).expect("compile tiny typed traversal package")
    }

    fn traversal<'a>(
        package: &'a LexicalGrokkingPackage,
        index: &'a ForwardDecoderIndex,
    ) -> L1TypedEditTraversal<'a> {
        L1TypedEditTraversal {
            index,
            decoder_nodes: &package.decoder_nodes,
        }
    }

    fn tiny_queries() -> Vec<String> {
        let mut queries = BTreeSet::new();
        queries.insert(String::new());
        for length in 1_usize..=4 {
            let count = 3_usize.pow(length as u32);
            for mut ordinal in 0..count {
                let mut symbols = vec!['a'; length];
                for symbol in symbols.iter_mut().rev() {
                    *symbol = ['a', 'b', 'c'][ordinal % 3];
                    ordinal /= 3;
                }
                let surface = symbols.into_iter().collect::<String>();
                queries.insert(surface.clone());
                queries.insert(format!("{surface}!"));
                queries.insert(format!("{surface},?"));
            }
        }
        queries.into_iter().collect()
    }

    fn phase7b_queries() -> Vec<String> {
        let mut queries = tiny_queries().into_iter().collect::<BTreeSet<_>>();
        queries.extend(
            [",", "ф", "фи", "б", "и", "aa", "abba", "cbb"]
                .into_iter()
                .map(str::to_string),
        );
        queries.into_iter().collect()
    }

    fn phase7c_queries() -> Vec<String> {
        let mut queries = phase7b_queries().into_iter().collect::<BTreeSet<_>>();
        for length in 1_usize..=7 {
            let count = 3_usize.pow(length as u32);
            for mut ordinal in 0..count {
                let mut symbols = vec!['a'; length];
                for symbol in symbols.iter_mut().rev() {
                    *symbol = ['a', 'b', 'c'][ordinal % 3];
                    ordinal /= 3;
                }
                queries.insert(symbols.into_iter().collect());
            }
        }
        queries.into_iter().collect()
    }

    fn phase7d_queries() -> Vec<String> {
        phase7c_queries()
    }

    #[test]
    fn phase7a_traversal_matches_independent_dense_oracle_exhaustively() {
        let package = tiny_package();
        let index = ForwardDecoderIndex::build(&package).unwrap();
        let traversal = traversal(&package, &index);
        for raw in tiny_queries() {
            let query = L1TypedQueryField::encode(&raw).unwrap();
            let expected = direct_typed_oracle(&package, &query, TraversalScope::Phase7A).unwrap();
            let actual =
                traversal.traverse(&query, TraversalScope::Phase7A, TraversalSchedule::Forward);
            assert_eq!(actual.terminals, expected, "query {raw:?}");
        }
    }

    #[test]
    fn phase7a_schedule_cannot_change_terminal_or_metric_bytes() {
        let package = tiny_package();
        let index = ForwardDecoderIndex::build(&package).unwrap();
        let traversal = traversal(&package, &index);
        for raw in tiny_queries() {
            let query = L1TypedQueryField::encode(&raw).unwrap();
            let forward =
                traversal.traverse(&query, TraversalScope::Phase7A, TraversalSchedule::Forward);
            let reverse =
                traversal.traverse(&query, TraversalScope::Phase7A, TraversalSchedule::Reverse);
            let permuted = traversal.traverse(
                &query,
                TraversalScope::Phase7A,
                TraversalSchedule::Permuted(0x7a11_c0de),
            );
            assert_eq!(forward, reverse, "reverse query {raw:?}");
            assert_eq!(forward, permuted, "permuted query {raw:?}");
        }
    }

    #[test]
    fn phase7b_traversal_matches_independent_dense_oracle_exhaustively() {
        let package = tiny_package();
        let index = ForwardDecoderIndex::build(&package).unwrap();
        let traversal = traversal(&package, &index);
        for raw in phase7b_queries() {
            let query = L1TypedQueryField::encode(&raw).unwrap();
            let expected = direct_typed_oracle(&package, &query, TraversalScope::Phase7B).unwrap();
            let actual =
                traversal.traverse(&query, TraversalScope::Phase7B, TraversalSchedule::Forward);
            assert_eq!(actual.terminals, expected, "query {raw:?}");
        }
    }

    #[test]
    fn phase7b_schedule_cannot_change_terminal_or_metric_bytes() {
        let package = tiny_package();
        let index = ForwardDecoderIndex::build(&package).unwrap();
        let traversal = traversal(&package, &index);
        for raw in phase7b_queries() {
            let query = L1TypedQueryField::encode(&raw).unwrap();
            let forward =
                traversal.traverse(&query, TraversalScope::Phase7B, TraversalSchedule::Forward);
            let reverse =
                traversal.traverse(&query, TraversalScope::Phase7B, TraversalSchedule::Reverse);
            let permuted = traversal.traverse(
                &query,
                TraversalScope::Phase7B,
                TraversalSchedule::Permuted(0x7b11_c0de),
            );
            assert_eq!(forward, reverse, "reverse query {raw:?}");
            assert_eq!(forward, permuted, "permuted query {raw:?}");
        }
    }

    #[test]
    fn phase7b_operator_witnesses_emit_exact_typed_certificates() {
        let package = tiny_package();
        let index = ForwardDecoderIndex::build(&package).unwrap();
        let traversal = traversal(&package, &index);
        let cases = [
            (
                "aa",
                2,
                TypedCertificate::MissingLetter { target_position: 1 },
            ),
            (
                "abba",
                2,
                TypedCertificate::ExtraLetter { input_position: 1 },
            ),
            (
                "cbb",
                4,
                TypedCertificate::SingleSubstitution { position: 1 },
            ),
            (
                "ab",
                7,
                TypedCertificate::KeyboardLayout {
                    direction: LayoutDirection::UsToRu,
                },
            ),
            (
                ",",
                8,
                TypedCertificate::KeyboardLayout {
                    direction: LayoutDirection::UsToRu,
                },
            ),
            (
                "ф",
                0,
                TypedCertificate::KeyboardLayout {
                    direction: LayoutDirection::RuToUs,
                },
            ),
        ];
        for (raw, center_id, expected) in cases {
            let result = traversal.traverse(
                &L1TypedQueryField::encode(raw).unwrap(),
                TraversalScope::Phase7B,
                TraversalSchedule::Forward,
            );
            assert!(
                result
                    .terminals
                    .get(&center_id)
                    .is_some_and(|certificates| certificates.contains(&expected)),
                "query {raw:?} lacks {expected:?}"
            );
        }
    }

    #[test]
    fn phase7c_traversal_matches_independent_dense_oracle_exhaustively() {
        let package = tiny_package();
        let index = ForwardDecoderIndex::build(&package).unwrap();
        let traversal = traversal(&package, &index);
        for raw in phase7c_queries() {
            let query = L1TypedQueryField::encode(&raw).unwrap();
            let expected = direct_typed_oracle(&package, &query, TraversalScope::Phase7C).unwrap();
            let actual =
                traversal.traverse(&query, TraversalScope::Phase7C, TraversalSchedule::Forward);
            assert_eq!(actual.terminals, expected, "query {raw:?}");
        }
    }

    #[test]
    fn phase7c_schedule_cannot_change_terminal_or_metric_bytes() {
        let package = tiny_package();
        let index = ForwardDecoderIndex::build(&package).unwrap();
        let traversal = traversal(&package, &index);
        for raw in phase7c_queries() {
            let query = L1TypedQueryField::encode(&raw).unwrap();
            let forward =
                traversal.traverse(&query, TraversalScope::Phase7C, TraversalSchedule::Forward);
            let reverse =
                traversal.traverse(&query, TraversalScope::Phase7C, TraversalSchedule::Reverse);
            let permuted = traversal.traverse(
                &query,
                TraversalScope::Phase7C,
                TraversalSchedule::Permuted(0x7c11_c0de),
            );
            assert_eq!(forward, reverse, "reverse query {raw:?}");
            assert_eq!(forward, permuted, "permuted query {raw:?}");
        }
    }

    #[test]
    fn phase7c_operator_witnesses_emit_exact_typed_certificates() {
        let package = tiny_package();
        let index = ForwardDecoderIndex::build(&package).unwrap();
        let traversal = traversal(&package, &index);
        let cases = [
            (
                "acbab",
                TypedCertificate::AdjacentTransposition { position: 1 },
            ),
            (
                "aacbb",
                TypedCertificate::NonAdjacentTransposition {
                    first: 1,
                    second: 3,
                },
            ),
            (
                "abcabab",
                TypedCertificate::RepeatedFragment {
                    input_start: 3,
                    source_target_start: 0,
                    len: 2,
                },
            ),
        ];
        for (raw, expected) in cases {
            let result = traversal.traverse(
                &L1TypedQueryField::encode(raw).unwrap(),
                TraversalScope::Phase7C,
                TraversalSchedule::Forward,
            );
            assert!(
                result
                    .terminals
                    .get(&9)
                    .is_some_and(|certificates| certificates.contains(&expected)),
                "query {raw:?} lacks {expected:?}"
            );
        }
    }

    #[test]
    fn phase7d_traversal_matches_independent_dense_oracle_exhaustively() {
        let package = tiny_package();
        let index = ForwardDecoderIndex::build(&package).unwrap();
        let traversal = traversal(&package, &index);
        for raw in phase7d_queries() {
            let query = L1TypedQueryField::encode(&raw).unwrap();
            let expected = direct_typed_oracle(&package, &query, TraversalScope::Phase7D).unwrap();
            let actual =
                traversal.traverse(&query, TraversalScope::Phase7D, TraversalSchedule::Forward);
            assert_eq!(actual.terminals, expected, "query {raw:?}");
        }
    }

    #[test]
    fn phase7d_schedule_cannot_change_terminal_or_metric_bytes() {
        let package = tiny_package();
        let index = ForwardDecoderIndex::build(&package).unwrap();
        let traversal = traversal(&package, &index);
        for raw in phase7d_queries() {
            let query = L1TypedQueryField::encode(&raw).unwrap();
            let forward =
                traversal.traverse(&query, TraversalScope::Phase7D, TraversalSchedule::Forward);
            let reverse =
                traversal.traverse(&query, TraversalScope::Phase7D, TraversalSchedule::Reverse);
            let permuted = traversal.traverse(
                &query,
                TraversalScope::Phase7D,
                TraversalSchedule::Permuted(0x7d11_c0de),
            );
            assert_eq!(forward, reverse, "reverse query {raw:?}");
            assert_eq!(forward, permuted, "permuted query {raw:?}");
        }
    }

    #[test]
    fn phase7d_operator_witnesses_emit_exact_typed_certificates() {
        let package = tiny_package();
        let index = ForwardDecoderIndex::build(&package).unwrap();
        let traversal = traversal(&package, &index);
        let cases = [
            (
                "accbb",
                TypedCertificate::DoubleSubstitution {
                    first: 1,
                    second: 3,
                },
            ),
            (
                "acb",
                TypedCertificate::SparseMultiOmission {
                    first: 1,
                    second: 3,
                },
            ),
            (
                "bacb",
                TypedCertificate::OmissionTransposition {
                    omitted_target: 3,
                    transposed_target: 0,
                },
            ),
            (
                "caab",
                TypedCertificate::OmissionTransposition {
                    omitted_target: 1,
                    transposed_target: 0,
                },
            ),
        ];
        for (raw, expected) in cases {
            let result = traversal.traverse(
                &L1TypedQueryField::encode(raw).unwrap(),
                TraversalScope::Phase7D,
                TraversalSchedule::Forward,
            );
            assert!(
                result
                    .terminals
                    .get(&9)
                    .is_some_and(|certificates| certificates.contains(&expected)),
                "query {raw:?} lacks {expected:?}"
            );
        }
    }

    #[test]
    fn phase7d_projection_preserves_terminal_ids_and_certificate_classes() {
        let package = tiny_package();
        let index = ForwardDecoderIndex::build(&package).unwrap();
        let query = L1TypedQueryField::encode("acb").unwrap();
        let complete = traversal(&package, &index).traverse(
            &query,
            TraversalScope::Phase7D,
            TraversalSchedule::Forward,
        );
        let baseline = phase7d_terminal_evidence(&index, &package.decoder_nodes, "acb").unwrap();
        let projected =
            phase7d_terminal_diagnostic_evidence(&index, &package.decoder_nodes, "acb").unwrap();

        assert_eq!(
            baseline.terminal_ids,
            complete.terminals.keys().copied().collect::<Vec<_>>()
        );
        assert_eq!(projected.terminal, baseline);
        assert_eq!(
            projected.certificate_classes.get(&9),
            Some(&vec!["sparse_multi_omission"])
        );
    }

    #[test]
    fn punctuation_and_boundary_insertions_have_canonical_certificates() {
        let package = tiny_package();
        let index = ForwardDecoderIndex::build(&package).unwrap();
        let traversal = traversal(&package, &index);
        let punctuated = traversal.traverse(
            &L1TypedQueryField::encode("cab,?").unwrap(),
            TraversalScope::Phase7A,
            TraversalSchedule::Forward,
        );
        assert_eq!(
            punctuated.terminals.get(&4).unwrap(),
            &BTreeSet::from([
                TypedCertificate::Identity,
                TypedCertificate::PunctuationSuffix {
                    raw_start: 3,
                    raw_len: 2,
                },
            ])
        );
        let prefix = traversal.traverse(
            &L1TypedQueryField::encode("ab").unwrap(),
            TraversalScope::Phase7A,
            TraversalSchedule::Forward,
        );
        assert!(prefix
            .terminals
            .get(&4)
            .unwrap()
            .contains(&TypedCertificate::PrefixTruncation { target_position: 0 }));
        let suffix = traversal.traverse(
            &L1TypedQueryField::encode("cab").unwrap(),
            TraversalScope::Phase7A,
            TraversalSchedule::Forward,
        );
        assert!(suffix
            .terminals
            .get(&5)
            .unwrap()
            .contains(&TypedCertificate::SuffixTruncation { target_position: 3 }));
    }

    #[test]
    fn raw_layout_lane_preserves_punctuation_that_lexical_normalization_strips() {
        let field = L1TypedQueryField::encode(",").unwrap();
        assert!(field.lexical_symbols.is_empty());
        assert_eq!(field.raw_layout_symbols.as_ref(), &[',' as u32]);
        assert_eq!(field.leading_punctuation_len, 1);
        assert_eq!(field.trailing_punctuation_len, 1);
    }

    #[test]
    fn production_owners_do_not_import_typed_edit_traversal() {
        for source in [
            include_str!("runtime.rs"),
            include_str!("service.rs"),
            include_str!("peak_search/mod.rs"),
        ] {
            assert!(!source.contains("typed_edit_traversal"));
            assert!(!source.contains("L1TypedEditTraversal"));
        }
    }
}
