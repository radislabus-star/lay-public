use std::cmp::Ordering;
use std::collections::BTreeMap;

use super::geometry::{
    GeometryPathIdentityV1, GeometryTerminalEvidenceV1, GeometryTraversalStateV1,
    ObservedGeometryV1,
};
use super::induce::SourceAnchorV1;
use super::trie::{
    ProductiveTerminalAttributionV1, ProductiveTrieArcActionV1, ProductiveTrieForestV1,
};

pub(super) const PRODUCTIVE_PHYSICAL_TOP_K: usize = 32;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) struct SpeedParityEvidenceV1 {
    pub(super) family_specificity: u8,
    pub(super) profile_evidence_milli: u16,
    pub(super) positive_support: u32,
    pub(super) anti_support: u32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct SpeedParityCandidateV1 {
    pub(super) normalized_surface: String,
    pub(super) terminal: ProductiveTerminalAttributionV1,
    pub(super) evidence: SpeedParityEvidenceV1,
    pub(super) geometry: GeometryTerminalEvidenceV1,
}

#[derive(Clone, Copy, Debug)]
struct ScalarTraceNodeV1 {
    parent: Option<u32>,
    scalar: u32,
    length: u16,
}

#[derive(Default)]
pub(super) struct ScalarTraceArenaV1 {
    nodes: Vec<ScalarTraceNodeV1>,
}

impl ScalarTraceArenaV1 {
    pub(super) fn append(&mut self, parent: Option<u32>, scalar: u32) -> Result<u32, &'static str> {
        let length = parent
            .map(|parent| {
                self.nodes
                    .get(parent as usize)
                    .map(|node| node.length)
                    .ok_or("productive decoder trace parent is outside the arena")
            })
            .transpose()?
            .unwrap_or_default()
            .checked_add(1)
            .ok_or("productive decoder trace reaches u16 ceiling")?;
        if length == u16::MAX {
            return Err("productive decoder trace reaches the wire ceiling");
        }
        let reference = u32::try_from(self.nodes.len())
            .map_err(|_| "productive decoder trace arena exceeds u32")?;
        self.nodes.push(ScalarTraceNodeV1 {
            parent,
            scalar,
            length,
        });
        Ok(reference)
    }

    pub(super) fn scalars(&self, reference: Option<u32>) -> Result<Vec<u32>, &'static str> {
        let Some(mut reference) = reference else {
            return Ok(Vec::new());
        };
        let length = usize::from(
            self.nodes
                .get(reference as usize)
                .ok_or("productive decoder trace is outside the arena")?
                .length,
        );
        let mut scalars = vec![0_u32; length];
        for index in (0..length).rev() {
            let node = self
                .nodes
                .get(reference as usize)
                .ok_or("productive decoder trace chain is outside the arena")?;
            scalars[index] = node.scalar;
            if index > 0 {
                reference = node
                    .parent
                    .ok_or("productive decoder trace chain terminates early")?;
            }
        }
        Ok(scalars)
    }
}

#[derive(Clone)]
pub(super) struct TraversalFrameV1 {
    pub(super) node_id: u32,
    pub(super) source_cursor: usize,
    pub(super) trace_ref: Option<u32>,
    pub(super) geometry: GeometryTraversalStateV1,
    pub(super) exact_allomorph: bool,
}

#[derive(Clone)]
struct BoundedCandidateV1 {
    surface_scalars: Vec<u32>,
    terminal: ProductiveTerminalAttributionV1,
    evidence: SpeedParityEvidenceV1,
    geometry: GeometryTerminalEvidenceV1,
}

pub(super) fn traverse_speed_parity(
    forest: &ProductiveTrieForestV1,
    paradigm_id: u32,
    canonical_source: &str,
    observed: &ObservedGeometryV1,
    evidence_by_ref: &BTreeMap<u32, SpeedParityEvidenceV1>,
) -> Result<Vec<SpeedParityCandidateV1>, &'static str> {
    let root = *forest
        .roots_by_paradigm
        .get(&paradigm_id)
        .ok_or("productive speed-parity trie lacks the requested paradigm")?;
    let source = canonical_source.chars().collect::<Vec<_>>();
    if source.len() >= u16::MAX as usize {
        return Err("productive canonical source reaches the wire ceiling");
    }
    let mut trace_arena = ScalarTraceArenaV1::default();
    let mut selected = Vec::<BoundedCandidateV1>::with_capacity(PRODUCTIVE_PHYSICAL_TOP_K);
    let mut stack = vec![TraversalFrameV1 {
        node_id: root,
        source_cursor: 0,
        trace_ref: None,
        geometry: GeometryTraversalStateV1::new(
            observed,
            GeometryPathIdentityV1 {
                paradigm_id,
                ..GeometryPathIdentityV1::default()
            },
        )?,
        exact_allomorph: false,
    }];

    while let Some(mut frame) = stack.pop() {
        let node = forest
            .nodes
            .get(frame.node_id as usize)
            .ok_or("productive speed-parity traversal reached an invalid node")?;
        if frame.exact_allomorph || frame.source_cursor == source.len() {
            for terminal in &node.terminals {
                let evidence = *evidence_by_ref
                    .get(&terminal.evidence_ref)
                    .ok_or("productive terminal lacks speed-parity evidence")?;
                frame.geometry.identity.slot_id = terminal.target_slot_id;
                frame.geometry.identity.program_id = terminal.program_id;
                frame.geometry.identity.variant_id = terminal.variant_id;
                frame.geometry.identity.decoder_trace_ref = frame.trace_ref.unwrap_or(u32::MAX);
                let geometry = frame.geometry.terminal_evidence();
                let surface_scalars = trace_arena.scalars(frame.trace_ref)?;
                retain_bounded_candidate(
                    &mut selected,
                    BoundedCandidateV1 {
                        surface_scalars,
                        terminal: terminal.clone(),
                        evidence,
                        geometry,
                    },
                );
            }
        }
        for arc in node.arcs.iter().rev() {
            let mut child = frame.clone();
            child.node_id = arc.child_node;
            if advance_arc(&arc.action, forest, &source, &mut child, &mut trace_arena)? {
                stack.push(child);
            }
        }
    }
    selected.sort_by(|left, right| candidate_order(right, left));
    selected
        .into_iter()
        .map(|candidate| {
            let normalized_surface = candidate
                .surface_scalars
                .into_iter()
                .map(|scalar| {
                    char::from_u32(scalar).ok_or("decoder trace contains an invalid scalar")
                })
                .collect::<Result<String, _>>()?;
            Ok(SpeedParityCandidateV1 {
                normalized_surface,
                terminal: candidate.terminal,
                evidence: candidate.evidence,
                geometry: candidate.geometry,
            })
        })
        .collect()
}

fn retain_bounded_candidate(selected: &mut Vec<BoundedCandidateV1>, candidate: BoundedCandidateV1) {
    if let Some(existing) = selected
        .iter_mut()
        .find(|existing| existing.surface_scalars == candidate.surface_scalars)
    {
        if candidate_order(&candidate, existing).is_gt() {
            *existing = candidate;
        }
        return;
    }
    if selected.len() < PRODUCTIVE_PHYSICAL_TOP_K {
        selected.push(candidate);
        return;
    }
    let worst = selected
        .iter()
        .enumerate()
        .min_by(|(_, left), (_, right)| candidate_order(left, right))
        .map(|(index, _)| index)
        .expect("bounded candidate set is nonempty");
    if candidate_order(&candidate, &selected[worst]).is_gt() {
        selected[worst] = candidate;
    }
}

fn candidate_order(left: &BoundedCandidateV1, right: &BoundedCandidateV1) -> Ordering {
    left.evidence
        .family_specificity
        .cmp(&right.evidence.family_specificity)
        .then_with(|| {
            left.evidence
                .profile_evidence_milli
                .cmp(&right.evidence.profile_evidence_milli)
        })
        .then_with(|| {
            left.evidence
                .positive_support
                .cmp(&right.evidence.positive_support)
        })
        .then_with(|| right.evidence.anti_support.cmp(&left.evidence.anti_support))
        .then_with(|| {
            left.geometry
                .geometry_milli
                .cmp(&right.geometry.geometry_milli)
        })
        .then_with(|| right.surface_scalars.cmp(&left.surface_scalars))
        .then_with(|| right.terminal.cmp(&left.terminal))
}

fn advance_arc(
    action: &ProductiveTrieArcActionV1,
    forest: &ProductiveTrieForestV1,
    source: &[char],
    frame: &mut TraversalFrameV1,
    trace_arena: &mut ScalarTraceArenaV1,
) -> Result<bool, &'static str> {
    if frame.exact_allomorph {
        return Ok(false);
    }
    match action {
        ProductiveTrieArcActionV1::CopySourceRange {
            source_anchor,
            source_delta,
            scalar_count,
        } => {
            let Some(start) = resolve_source_offset(source.len(), *source_anchor, *source_delta)
            else {
                return Ok(false);
            };
            let end = start
                .checked_add(usize::from(*scalar_count))
                .ok_or("productive runtime copy range overflow")?;
            if start != frame.source_cursor || end > source.len() {
                return Ok(false);
            }
            for scalar in &source[start..end] {
                emit_scalar(frame, trace_arena, *scalar)?;
            }
            frame.source_cursor = end;
        }
        ProductiveTrieArcActionV1::CopyToRetainedEdge {
            source_anchor,
            source_delta,
            retained_end_delta,
        } => {
            let Some(start) = resolve_source_offset(source.len(), *source_anchor, *source_delta)
            else {
                return Ok(false);
            };
            let Some(end) = source
                .len()
                .checked_add_signed(isize::from(*retained_end_delta))
            else {
                return Ok(false);
            };
            if start != frame.source_cursor || end <= start || end > source.len() {
                return Ok(false);
            }
            for scalar in &source[start..end] {
                emit_scalar(frame, trace_arena, *scalar)?;
            }
            frame.source_cursor = end;
        }
        ProductiveTrieArcActionV1::DropSourcePrefix { scalar_count } => {
            if frame.source_cursor != 0
                || *scalar_count == 0
                || usize::from(*scalar_count) > source.len()
            {
                return Ok(false);
            }
            frame.source_cursor = usize::from(*scalar_count);
        }
        ProductiveTrieArcActionV1::DropSourceSuffix { scalar_count } => {
            if *scalar_count == 0
                || frame.source_cursor.checked_add(usize::from(*scalar_count)) != Some(source.len())
            {
                return Ok(false);
            }
            frame.source_cursor = source.len();
        }
        ProductiveTrieArcActionV1::EmitSegment { segment } => {
            for scalar in segment.chars() {
                emit_scalar(frame, trace_arena, scalar)?;
            }
        }
        ProductiveTrieArcActionV1::ReplaceSourceStart {
            end_relative_offset,
            delete_count,
        } => {
            let Some(start) = source
                .len()
                .checked_add_signed(isize::from(*end_relative_offset))
            else {
                return Ok(false);
            };
            let end = start
                .checked_add(usize::from(*delete_count))
                .ok_or("productive runtime replacement range overflow")?;
            if start != frame.source_cursor || end > source.len() {
                return Ok(false);
            }
            frame.source_cursor = end;
        }
        ProductiveTrieArcActionV1::EmitExactAllomorph { form_ref } => {
            let surface = forest
                .exact_allomorph_surface(*form_ref)
                .ok_or("productive runtime exact allomorph decoder is unavailable")?;
            frame.trace_ref = None;
            frame.geometry.reset_generated();
            for scalar in surface.chars() {
                emit_scalar(frame, trace_arena, scalar)?;
            }
            frame.source_cursor = source.len();
            frame.exact_allomorph = true;
        }
    }
    Ok(true)
}

pub(super) fn emit_scalar(
    frame: &mut TraversalFrameV1,
    trace_arena: &mut ScalarTraceArenaV1,
    scalar: char,
) -> Result<(), &'static str> {
    frame.geometry.emit_normalized_scalar(scalar)?;
    frame.trace_ref = Some(trace_arena.append(frame.trace_ref, u32::from(scalar))?);
    Ok(())
}

pub(super) fn resolve_source_offset(
    source_len: usize,
    anchor: SourceAnchorV1,
    delta: i16,
) -> Option<usize> {
    let base = match anchor {
        SourceAnchorV1::Start => 0,
        SourceAnchorV1::End => source_len,
    };
    base.checked_add_signed(isize::from(delta))
}

#[cfg(test)]
mod tests {
    use super::super::super::compositional::{
        prepared_similarity_to_normalized_surface_milli, surface_scoring_profile,
    };
    use super::super::super::productive::{
        productive_birth_rank, ProductiveBirthStatus, ProductiveFormBirth,
    };
    use super::super::induce::{derive_edit_template, CanonicalFormObservationV1};
    use super::super::trie::{compile_productive_trie, TrieProgramInputV1};
    use super::super::types::{
        MorphologyApplicabilityMaskV1, MorphologySlotKeyV1, AXIS_INAPPLICABLE,
    };
    use super::*;

    fn form(form_ref: u32, slot_id: u32, surface: &str, number: u8) -> CanonicalFormObservationV1 {
        CanonicalFormObservationV1 {
            form_ref,
            slot_id,
            slot: MorphologySlotKeyV1::new(
                2,
                number,
                AXIS_INAPPLICABLE,
                AXIS_INAPPLICABLE,
                AXIS_INAPPLICABLE,
                AXIS_INAPPLICABLE,
                AXIS_INAPPLICABLE,
                AXIS_INAPPLICABLE,
                AXIS_INAPPLICABLE,
                AXIS_INAPPLICABLE,
                AXIS_INAPPLICABLE,
                AXIS_INAPPLICABLE,
                AXIS_INAPPLICABLE,
            ),
            applicability: MorphologyApplicabilityMaskV1::new(0b11).expect("mask"),
            normalized_surface: surface.to_string(),
            support: 10,
            provenance_id: form_ref,
            variant_id: 1,
        }
    }

    #[test]
    fn speed_parity_runtime_uses_v39_tuple_and_keeps_top_32_bounded() {
        let source = form(1, 1, "кот", 1);
        let targets = (0..40)
            .map(|index| form(index + 2, index + 2, &format!("кот{index:02}"), 2))
            .collect::<Vec<_>>();
        let programs = targets
            .iter()
            .enumerate()
            .map(|(index, target)| {
                let mut template = derive_edit_template(&source, target).expect("template");
                template.transferable = true;
                TrieProgramInputV1 {
                    paradigm_id: 1,
                    anchor_scalar_len: 3,
                    template,
                    exact_allomorph_surface: None,
                    terminal: ProductiveTerminalAttributionV1 {
                        program_id: index as u32 + 1,
                        target_slot_id: target.slot_id,
                        variant_id: 1,
                        decoder_ref: target.form_ref,
                        evidence_ref: index as u32 + 1,
                        calibration_class: 1,
                        provenance_ref: 1,
                        stable_identity_hash: index as u32 + 1,
                    },
                }
            })
            .collect::<Vec<_>>();
        let forest = compile_productive_trie(&programs).expect("trie");
        let evidence = programs
            .iter()
            .map(|program| {
                (
                    program.terminal.evidence_ref,
                    SpeedParityEvidenceV1 {
                        family_specificity: (program.terminal.program_id % 8) as u8,
                        profile_evidence_milli: 900,
                        positive_support: program.terminal.program_id,
                        anti_support: 0,
                    },
                )
            })
            .collect::<BTreeMap<_, _>>();
        let candidates = traverse_speed_parity(
            &forest,
            1,
            "кот",
            &ObservedGeometryV1::new("кот39").expect("observed"),
            &evidence,
        )
        .expect("traverse");
        assert_eq!(candidates.len(), PRODUCTIVE_PHYSICAL_TOP_K);
        assert_eq!(candidates[0].evidence.family_specificity, 7);
        assert!(candidates.windows(2).all(|pair| {
            let left = BoundedCandidateV1 {
                surface_scalars: pair[0].normalized_surface.chars().map(u32::from).collect(),
                terminal: pair[0].terminal.clone(),
                evidence: pair[0].evidence,
                geometry: pair[0].geometry,
            };
            let right = BoundedCandidateV1 {
                surface_scalars: pair[1].normalized_surface.chars().map(u32::from).collect(),
                terminal: pair[1].terminal.clone(),
                evidence: pair[1].evidence,
                geometry: pair[1].geometry,
            };
            !candidate_order(&left, &right).is_lt()
        }));

        let observed_profile = surface_scoring_profile("кот39");
        let mut v39 = targets
            .iter()
            .enumerate()
            .map(|(index, target)| {
                let evidence = evidence[&(index as u32 + 1)];
                ProductiveFormBirth {
                    surface: target.normalized_surface.clone(),
                    source_feature_mask: 1,
                    target_feature_mask: target.slot_id,
                    geometry_evidence_milli: prepared_similarity_to_normalized_surface_milli(
                        &observed_profile,
                        &target.normalized_surface,
                    ),
                    profile_evidence_milli: evidence.profile_evidence_milli,
                    positive_support: evidence.positive_support,
                    anti_support: evidence.anti_support,
                    family_specificity: evidence.family_specificity,
                    status: ProductiveBirthStatus::ShadowUnverified,
                }
            })
            .collect::<Vec<_>>();
        v39.sort_by(|left, right| {
            productive_birth_rank(right)
                .cmp(&productive_birth_rank(left))
                .then_with(|| left.surface.cmp(&right.surface))
        });
        v39.truncate(PRODUCTIVE_PHYSICAL_TOP_K);
        assert_eq!(
            candidates
                .iter()
                .map(|candidate| candidate.normalized_surface.as_str())
                .collect::<Vec<_>>(),
            v39.iter()
                .map(|candidate| candidate.surface.as_str())
                .collect::<Vec<_>>()
        );
    }

    #[test]
    fn source_length_incompatible_arc_is_pruned_without_integrity_failure() {
        let source = ['а'];
        let observed = ObservedGeometryV1::new("а").expect("observed");
        let mut trace_arena = ScalarTraceArenaV1::default();
        let forest = ProductiveTrieForestV1::default();
        let mut frame = TraversalFrameV1 {
            node_id: 0,
            source_cursor: 0,
            trace_ref: None,
            geometry: GeometryTraversalStateV1::new(&observed, GeometryPathIdentityV1::default())
                .expect("geometry"),
            exact_allomorph: false,
        };

        let applicable = advance_arc(
            &ProductiveTrieArcActionV1::ReplaceSourceStart {
                end_relative_offset: -2,
                delete_count: 1,
            },
            &forest,
            &source,
            &mut frame,
            &mut trace_arena,
        )
        .expect("incompatible branch is not an integrity failure");

        assert!(!applicable);
        assert_eq!(frame.source_cursor, 0);
    }
}
