use std::cmp::Ordering;
use std::collections::{BTreeMap, BTreeSet};

use super::types::{MorphologyApplicabilityMaskV1, MorphologySlotKeyV1};

const MAX_WIRE_SCALARS: usize = u16::MAX as usize - 1;
const MAX_SIGNED_OFFSET: usize = i16::MAX as usize;
pub(super) const COPY_TO_RETAINED_EDGE: u16 = u16::MAX;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct CanonicalFormObservationV1 {
    pub(super) form_ref: u32,
    pub(super) slot_id: u32,
    pub(super) slot: MorphologySlotKeyV1,
    pub(super) applicability: MorphologyApplicabilityMaskV1,
    pub(super) normalized_surface: String,
    pub(super) support: u32,
    pub(super) provenance_id: u32,
    pub(super) variant_id: u16,
}

pub(super) fn select_canonical_anchor(
    forms: &[CanonicalFormObservationV1],
) -> Result<&CanonicalFormObservationV1, &'static str> {
    if forms.is_empty() {
        return Err("canonical anchor requires at least one train form");
    }
    for form in forms {
        if form.support == 0 || form.slot_id == 0 {
            return Err("canonical anchor input contains an unowned form");
        }
        form.slot
            .validate(form.applicability)
            .map_err(|_| "canonical anchor input contains an invalid slot")?;
        checked_scalar_len(&form.normalized_surface)?;
    }
    forms
        .iter()
        .min_by(|left, right| canonical_anchor_order(left, right))
        .ok_or("canonical anchor selection failed")
}

fn canonical_anchor_order(
    left: &CanonicalFormObservationV1,
    right: &CanonicalFormObservationV1,
) -> Ordering {
    right
        .support
        .cmp(&left.support)
        .then_with(|| {
            right
                .slot
                .annotation_complete(right.applicability)
                .cmp(&left.slot.annotation_complete(left.applicability))
        })
        .then_with(|| left.slot.to_bytes().cmp(&right.slot.to_bytes()))
        .then_with(|| {
            left.normalized_surface
                .chars()
                .count()
                .cmp(&right.normalized_surface.chars().count())
        })
        .then_with(|| {
            left.normalized_surface
                .as_bytes()
                .cmp(right.normalized_surface.as_bytes())
        })
        .then_with(|| left.provenance_id.cmp(&right.provenance_id))
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
pub(super) enum SourceAnchorV1 {
    Start = 1,
    End = 2,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(super) enum EditOperationV1 {
    CopySourceRange {
        start_anchor: SourceAnchorV1,
        start_delta: i16,
        scalar_count: u16,
    },
    DropSourcePrefix {
        scalar_count: u16,
    },
    DropSourceSuffix {
        scalar_count: u16,
    },
    EmitSegment {
        segment: String,
    },
    ReplaceSourceRange {
        end_relative_offset: i16,
        delete_count: u16,
        segment: String,
    },
    EmitExactAllomorph {
        form_ref: u32,
    },
    Terminate {
        slot_id: u32,
        variant_id: u16,
    },
}

impl EditOperationV1 {
    pub(super) fn canonical_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        match self {
            Self::CopySourceRange {
                start_anchor,
                start_delta,
                scalar_count,
            } => {
                bytes.extend_from_slice(&[1, *start_anchor as u8]);
                bytes.extend_from_slice(&start_delta.to_le_bytes());
                bytes.extend_from_slice(&scalar_count.to_le_bytes());
            }
            Self::DropSourcePrefix { scalar_count } => {
                bytes.push(2);
                bytes.extend_from_slice(&scalar_count.to_le_bytes());
            }
            Self::DropSourceSuffix { scalar_count } => {
                bytes.push(3);
                bytes.extend_from_slice(&scalar_count.to_le_bytes());
            }
            Self::EmitSegment { segment } => {
                bytes.push(4);
                push_bytes(&mut bytes, segment.as_bytes());
            }
            Self::ReplaceSourceRange {
                end_relative_offset,
                delete_count,
                segment,
            } => {
                bytes.push(5);
                bytes.extend_from_slice(&end_relative_offset.to_le_bytes());
                bytes.extend_from_slice(&delete_count.to_le_bytes());
                push_bytes(&mut bytes, segment.as_bytes());
            }
            Self::EmitExactAllomorph { form_ref } => {
                bytes.push(6);
                bytes.extend_from_slice(&form_ref.to_le_bytes());
            }
            Self::Terminate {
                slot_id,
                variant_id,
            } => {
                bytes.push(7);
                bytes.extend_from_slice(&slot_id.to_le_bytes());
                bytes.extend_from_slice(&variant_id.to_le_bytes());
            }
        }
        bytes
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct EditTemplateV1 {
    pub(super) source_slot_id: u32,
    pub(super) target_slot_id: u32,
    pub(super) source_slot: MorphologySlotKeyV1,
    pub(super) target_slot: MorphologySlotKeyV1,
    pub(super) variant_id: u16,
    pub(super) operations: Vec<EditOperationV1>,
    pub(super) transferable: bool,
}

impl EditTemplateV1 {
    pub(super) fn reconstruct(
        &self,
        source: &str,
        exact_allomorph: Option<&str>,
    ) -> Result<String, &'static str> {
        let source = source.chars().collect::<Vec<_>>();
        let mut output = String::new();
        let mut cursor = 0_usize;
        let suffix_drop = self
            .operations
            .iter()
            .filter_map(|operation| match operation {
                EditOperationV1::DropSourceSuffix { scalar_count } => {
                    Some(usize::from(*scalar_count))
                }
                _ => None,
            })
            .try_fold(0_usize, |total, count| total.checked_add(count))
            .ok_or("source suffix drop count overflow")?;
        let retained_end = source
            .len()
            .checked_sub(suffix_drop)
            .ok_or("source suffix drops exceed the source")?;
        let exact_count = self
            .operations
            .iter()
            .filter(|operation| matches!(operation, EditOperationV1::EmitExactAllomorph { .. }))
            .count();
        if exact_count != 0 {
            if exact_count != 1
                || self.operations.len() != 2
                || !matches!(
                    self.operations.last(),
                    Some(EditOperationV1::Terminate { .. })
                )
            {
                return Err("exact allomorph must be the only emitting instruction");
            }
            return exact_allomorph
                .map(str::to_owned)
                .ok_or("exact allomorph decoder form is unavailable");
        }

        let mut terminated = false;
        for (operation_index, operation) in self.operations.iter().enumerate() {
            if terminated {
                return Err("edit program contains an operation after terminate");
            }
            match operation {
                EditOperationV1::CopySourceRange {
                    start_anchor,
                    start_delta,
                    scalar_count,
                } => {
                    let start = resolve_source_offset(source.len(), *start_anchor, *start_delta)?;
                    let end = if *scalar_count == COPY_TO_RETAINED_EDGE {
                        match self.operations.get(operation_index + 1) {
                            Some(EditOperationV1::ReplaceSourceRange {
                                end_relative_offset,
                                ..
                            }) => resolve_source_offset(
                                source.len(),
                                SourceAnchorV1::End,
                                *end_relative_offset,
                            )?,
                            _ => retained_end,
                        }
                    } else {
                        start
                            .checked_add(usize::from(*scalar_count))
                            .ok_or("copy range overflow")?
                    };
                    if start != cursor || end <= start || end > retained_end {
                        return Err("copy range is non-monotonic or outside the source");
                    }
                    output.extend(source[start..end].iter());
                    cursor = end;
                }
                EditOperationV1::DropSourcePrefix { scalar_count } => {
                    if cursor != 0
                        || *scalar_count == 0
                        || usize::from(*scalar_count) > source.len()
                    {
                        return Err("source prefix drop is invalid");
                    }
                    cursor = usize::from(*scalar_count);
                }
                EditOperationV1::DropSourceSuffix { scalar_count } => {
                    let count = usize::from(*scalar_count);
                    if count == 0 || cursor.checked_add(count) != Some(source.len()) {
                        return Err("source suffix drop is invalid");
                    }
                    cursor = source.len();
                }
                EditOperationV1::EmitSegment { segment } => {
                    if segment.is_empty() {
                        return Err("edit program contains an empty emitted segment");
                    }
                    output.push_str(segment);
                }
                EditOperationV1::ReplaceSourceRange {
                    end_relative_offset,
                    delete_count,
                    segment,
                } => {
                    let start = source
                        .len()
                        .checked_add_signed(isize::from(*end_relative_offset))
                        .ok_or("replacement range offset is outside the source")?;
                    let end = start
                        .checked_add(usize::from(*delete_count))
                        .ok_or("replacement range overflow")?;
                    if start != cursor || end > source.len() {
                        return Err("replacement range is non-monotonic or outside the source");
                    }
                    output.push_str(segment);
                    cursor = end;
                }
                EditOperationV1::EmitExactAllomorph { .. } => unreachable!("handled above"),
                EditOperationV1::Terminate {
                    slot_id,
                    variant_id,
                } => {
                    if *slot_id != self.target_slot_id || *variant_id != self.variant_id {
                        return Err("terminate identity disagrees with edit template");
                    }
                    if cursor != source.len() {
                        return Err(
                            "edit program terminates before accounting for every source scalar",
                        );
                    }
                    terminated = true;
                }
            }
        }
        if !terminated {
            return Err("edit program lacks terminate");
        }
        Ok(output)
    }

    pub(super) fn source_ranges_valid(&self, source_scalar_len: usize) -> bool {
        if source_scalar_len > MAX_WIRE_SCALARS {
            return false;
        }
        let placeholder = std::iter::repeat_n('x', source_scalar_len).collect::<String>();
        self.reconstruct(&placeholder, Some("x")).is_ok()
    }

    pub(super) fn transition_key(&self) -> ParadigmTransitionKeyV1 {
        ParadigmTransitionKeyV1 {
            source_slot: self.source_slot,
            target_slot: self.target_slot,
            operations: self
                .operations
                .iter()
                .filter(|operation| !matches!(operation, EditOperationV1::Terminate { .. }))
                .cloned()
                .collect(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct AlignmentCandidateV1 {
    edit_cost: u32,
    internal_replacements: u32,
    emitted_segments: u32,
    operations: Vec<EditOperationV1>,
}

impl AlignmentCandidateV1 {
    fn extended(
        &self,
        operation: EditOperationV1,
        edit_cost: usize,
        internal_replacement: bool,
        emitted_segment: bool,
    ) -> Result<Self, &'static str> {
        let mut next = self.clone();
        next.edit_cost = next
            .edit_cost
            .checked_add(u32::try_from(edit_cost).map_err(|_| "edit cost exceeds u32")?)
            .ok_or("edit cost overflow")?;
        next.internal_replacements += u32::from(internal_replacement);
        next.emitted_segments += u32::from(emitted_segment);
        next.operations.push(operation);
        Ok(next)
    }

    fn order(&self, other: &Self) -> Ordering {
        self.edit_cost
            .cmp(&other.edit_cost)
            .then_with(|| self.internal_replacements.cmp(&other.internal_replacements))
            .then_with(|| self.emitted_segments.cmp(&other.emitted_segments))
            .then_with(|| self.operations.len().cmp(&other.operations.len()))
            .then_with(|| {
                canonical_program_bytes(&self.operations)
                    .cmp(&canonical_program_bytes(&other.operations))
            })
    }
}

pub(super) fn derive_edit_template(
    source: &CanonicalFormObservationV1,
    target: &CanonicalFormObservationV1,
) -> Result<EditTemplateV1, &'static str> {
    if source.slot.pos_domain() != target.slot.pos_domain() {
        return Err("source and target slots cross POS domains");
    }
    if source.slot_id == 0 || target.slot_id == 0 {
        return Err("edit template requires owned slot IDs");
    }
    let source_scalars = source.normalized_surface.chars().collect::<Vec<_>>();
    let target_scalars = target.normalized_surface.chars().collect::<Vec<_>>();
    checked_scalar_count(source_scalars.len())?;
    checked_scalar_count(target_scalars.len())?;
    let width = target_scalars.len() + 1;
    let mut cells = vec![None::<AlignmentCandidateV1>; (source_scalars.len() + 1) * width];
    cells[0] = Some(AlignmentCandidateV1 {
        edit_cost: 0,
        internal_replacements: 0,
        emitted_segments: 0,
        operations: Vec::new(),
    });

    for source_index in 0..=source_scalars.len() {
        for target_index in 0..=target_scalars.len() {
            let Some(current) = cells[source_index * width + target_index].clone() else {
                continue;
            };

            let mut copy_count = 0_usize;
            while source_index + copy_count < source_scalars.len()
                && target_index + copy_count < target_scalars.len()
                && source_scalars[source_index + copy_count]
                    == target_scalars[target_index + copy_count]
            {
                copy_count += 1;
            }
            if copy_count != 0 {
                let operation = copy_operation(source_index, copy_count, source_scalars.len())?;
                admit_alignment(
                    &mut cells,
                    width,
                    source_index + copy_count,
                    target_index + copy_count,
                    current.extended(operation, 0, false, false)?,
                );
            }

            if source_index == 0 {
                for count in 1..=source_scalars.len() {
                    let operation = EditOperationV1::DropSourcePrefix {
                        scalar_count: checked_scalar_count(count)?,
                    };
                    admit_alignment(
                        &mut cells,
                        width,
                        count,
                        target_index,
                        current.extended(operation, count, false, false)?,
                    );
                }
            }

            if target_index == target_scalars.len() && source_index < source_scalars.len() {
                let count = source_scalars.len() - source_index;
                let operation = EditOperationV1::DropSourceSuffix {
                    scalar_count: checked_scalar_count(count)?,
                };
                admit_alignment(
                    &mut cells,
                    width,
                    source_scalars.len(),
                    target_index,
                    current.extended(operation, count, false, false)?,
                );
            }

            if (source_index == 0 || source_index == source_scalars.len())
                && target_index < target_scalars.len()
            {
                for count in 1..=target_scalars.len() - target_index {
                    let segment =
                        scalars_to_string(&target_scalars[target_index..target_index + count]);
                    let operation = EditOperationV1::EmitSegment { segment };
                    admit_alignment(
                        &mut cells,
                        width,
                        source_index,
                        target_index + count,
                        current.extended(operation, count, false, true)?,
                    );
                }
            }

            if source_index < source_scalars.len() {
                for delete_count in 1..=source_scalars.len() - source_index {
                    for emit_count in 0..=target_scalars.len() - target_index {
                        if delete_count == emit_count
                            && emit_count != 0
                            && source_scalars[source_index..source_index + delete_count]
                                == target_scalars[target_index..target_index + emit_count]
                        {
                            continue;
                        }
                        let Some(end_relative_offset) =
                            end_relative_offset(source_index, source_scalars.len())
                        else {
                            continue;
                        };
                        let segment = scalars_to_string(
                            &target_scalars[target_index..target_index + emit_count],
                        );
                        let operation = EditOperationV1::ReplaceSourceRange {
                            end_relative_offset,
                            delete_count: checked_scalar_count(delete_count)?,
                            segment,
                        };
                        admit_alignment(
                            &mut cells,
                            width,
                            source_index + delete_count,
                            target_index + emit_count,
                            current.extended(
                                operation,
                                delete_count.max(emit_count),
                                true,
                                emit_count != 0,
                            )?,
                        );
                    }
                }
            }

            if source_index > 0
                && source_index < source_scalars.len()
                && target_index < target_scalars.len()
            {
                let Some(end_relative_offset) =
                    end_relative_offset(source_index, source_scalars.len())
                else {
                    continue;
                };
                for emit_count in 1..=target_scalars.len() - target_index {
                    let segment =
                        scalars_to_string(&target_scalars[target_index..target_index + emit_count]);
                    let operation = EditOperationV1::ReplaceSourceRange {
                        end_relative_offset,
                        delete_count: 0,
                        segment,
                    };
                    admit_alignment(
                        &mut cells,
                        width,
                        source_index,
                        target_index + emit_count,
                        current.extended(operation, emit_count, true, true)?,
                    );
                }
            }
        }
    }

    let mut best = cells
        .pop()
        .flatten()
        .ok_or("no representable scalar edit program")?;
    generalize_retained_copy(&mut best.operations, source_scalars.len())?;
    best.operations.push(EditOperationV1::Terminate {
        slot_id: target.slot_id,
        variant_id: target.variant_id,
    });
    let template = EditTemplateV1 {
        source_slot_id: source.slot_id,
        target_slot_id: target.slot_id,
        source_slot: source.slot,
        target_slot: target.slot,
        variant_id: target.variant_id,
        operations: best.operations,
        transferable: false,
    };
    if template.reconstruct(&source.normalized_surface, None)? != target.normalized_surface {
        return Err("edit template does not reconstruct its train target byte-exactly");
    }
    Ok(template)
}

fn generalize_retained_copy(
    operations: &mut [EditOperationV1],
    source_len: usize,
) -> Result<(), &'static str> {
    let prefix_drop = operations
        .iter()
        .filter_map(|operation| match operation {
            EditOperationV1::DropSourcePrefix { scalar_count } => Some(usize::from(*scalar_count)),
            _ => None,
        })
        .sum::<usize>();
    let suffix_drop = operations
        .iter()
        .filter_map(|operation| match operation {
            EditOperationV1::DropSourceSuffix { scalar_count } => Some(usize::from(*scalar_count)),
            _ => None,
        })
        .sum::<usize>();
    let retained_end = source_len
        .checked_sub(suffix_drop)
        .ok_or("source suffix drops exceed the source")?;
    if prefix_drop >= retained_end {
        return Ok(());
    }
    let original = operations.to_vec();
    for (index, operation) in original.iter().enumerate() {
        let EditOperationV1::CopySourceRange {
            start_anchor,
            start_delta,
            scalar_count,
        } = operation
        else {
            continue;
        };
        let start = resolve_source_offset(source_len, *start_anchor, *start_delta)?;
        let end = start
            .checked_add(usize::from(*scalar_count))
            .ok_or("copy range overflow")?;

        let mut generalized_anchor = *start_anchor;
        let mut generalized_delta = *start_delta;
        if index > 0 {
            if let Some(EditOperationV1::ReplaceSourceRange {
                end_relative_offset,
                delete_count,
                ..
            }) = original.get(index - 1)
            {
                let replacement_start =
                    resolve_source_offset(source_len, SourceAnchorV1::End, *end_relative_offset)?;
                let replacement_end = replacement_start
                    .checked_add(usize::from(*delete_count))
                    .ok_or("replacement range overflow")?;
                if start == replacement_end {
                    generalized_anchor = SourceAnchorV1::End;
                    generalized_delta =
                        i16::try_from(i32::from(*end_relative_offset) + i32::from(*delete_count))
                            .map_err(|_| "replacement-relative copy start exceeds i16")?;
                }
            }
        }

        let dynamic_end = match original.get(index + 1) {
            Some(EditOperationV1::ReplaceSourceRange {
                end_relative_offset,
                ..
            }) => resolve_source_offset(source_len, SourceAnchorV1::End, *end_relative_offset)?,
            _ => retained_end,
        };
        operations[index] = EditOperationV1::CopySourceRange {
            start_anchor: generalized_anchor,
            start_delta: generalized_delta,
            scalar_count: if end == dynamic_end {
                COPY_TO_RETAINED_EDGE
            } else {
                *scalar_count
            },
        };
    }
    Ok(())
}

fn admit_alignment(
    cells: &mut [Option<AlignmentCandidateV1>],
    width: usize,
    source_index: usize,
    target_index: usize,
    candidate: AlignmentCandidateV1,
) {
    let cell = &mut cells[source_index * width + target_index];
    if cell
        .as_ref()
        .is_none_or(|current| candidate.order(current).is_lt())
    {
        *cell = Some(candidate);
    }
}

fn copy_operation(
    start: usize,
    count: usize,
    source_len: usize,
) -> Result<EditOperationV1, &'static str> {
    let scalar_count = checked_scalar_count(count)?;
    if start <= MAX_SIGNED_OFFSET {
        Ok(EditOperationV1::CopySourceRange {
            start_anchor: SourceAnchorV1::Start,
            start_delta: i16::try_from(start).map_err(|_| "start-relative copy offset overflow")?,
            scalar_count,
        })
    } else {
        let delta = isize::try_from(start).map_err(|_| "copy offset exceeds isize")?
            - isize::try_from(source_len).map_err(|_| "source length exceeds isize")?;
        Ok(EditOperationV1::CopySourceRange {
            start_anchor: SourceAnchorV1::End,
            start_delta: i16::try_from(delta).map_err(|_| "end-relative copy offset overflow")?,
            scalar_count,
        })
    }
}

fn end_relative_offset(start: usize, source_len: usize) -> Option<i16> {
    let start = isize::try_from(start).ok()?;
    let source_len = isize::try_from(source_len).ok()?;
    i16::try_from(start - source_len).ok()
}

fn resolve_source_offset(
    source_len: usize,
    anchor: SourceAnchorV1,
    delta: i16,
) -> Result<usize, &'static str> {
    let base = match anchor {
        SourceAnchorV1::Start => 0,
        SourceAnchorV1::End => source_len,
    };
    base.checked_add_signed(isize::from(delta))
        .ok_or("copy source offset is outside the source")
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(super) struct ParadigmTransitionKeyV1 {
    pub(super) source_slot: MorphologySlotKeyV1,
    pub(super) target_slot: MorphologySlotKeyV1,
    pub(super) operations: Vec<EditOperationV1>,
}

impl ParadigmTransitionKeyV1 {
    pub(super) fn canonical_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&self.source_slot.to_bytes());
        bytes.extend_from_slice(&self.target_slot.to_bytes());
        bytes.extend_from_slice(&(self.operations.len() as u32).to_le_bytes());
        for operation in &self.operations {
            push_bytes(&mut bytes, &operation.canonical_bytes());
        }
        bytes
    }

    fn source_ranges_valid(&self, source_scalar_len: usize) -> bool {
        let template = EditTemplateV1 {
            source_slot_id: 1,
            target_slot_id: 1,
            source_slot: self.source_slot,
            target_slot: self.target_slot,
            variant_id: 1,
            operations: self
                .operations
                .iter()
                .cloned()
                .chain([EditOperationV1::Terminate {
                    slot_id: 1,
                    variant_id: 1,
                }])
                .collect(),
            transferable: true,
        };
        template.source_ranges_valid(source_scalar_len)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct ParadigmSignatureV1 {
    pub(super) pos_domain: u8,
    pub(super) transitions: Vec<ParadigmTransitionKeyV1>,
}

impl ParadigmSignatureV1 {
    pub(super) fn new(
        pos_domain: u8,
        transitions: impl IntoIterator<Item = ParadigmTransitionKeyV1>,
    ) -> Result<Self, &'static str> {
        let mut transitions = transitions.into_iter().collect::<Vec<_>>();
        transitions.sort_unstable();
        transitions.dedup();
        if transitions.iter().any(|transition| {
            transition.source_slot.pos_domain() != pos_domain
                || transition.target_slot.pos_domain() != pos_domain
        }) {
            return Err("paradigm transition crosses its POS domain");
        }
        Ok(Self {
            pos_domain,
            transitions,
        })
    }

    pub(super) fn semantically_equivalent(&self, other: &Self) -> bool {
        self == other
    }
}

#[derive(Clone, Debug)]
pub(super) struct LemmaTransitionObservationV1 {
    pub(super) lemma_id: u32,
    pub(super) target_form_ref: u32,
    pub(super) template: EditTemplateV1,
}

pub(super) fn apply_transfer_support_gate(
    observations: &mut [LemmaTransitionObservationV1],
) -> Result<BTreeMap<ParadigmTransitionKeyV1, u32>, &'static str> {
    let mut support = BTreeMap::<ParadigmTransitionKeyV1, BTreeSet<u32>>::new();
    for observation in observations.iter() {
        support
            .entry(observation.template.transition_key())
            .or_default()
            .insert(observation.lemma_id);
    }
    let support_counts = support
        .into_iter()
        .map(|(key, lemmas)| (key, lemmas.len() as u32))
        .collect::<BTreeMap<_, _>>();
    for observation in observations {
        let key = observation.template.transition_key();
        if support_counts.get(&key).copied().unwrap_or_default() >= 2 {
            observation.template.transferable = true;
        } else {
            observation.template.transferable = false;
            observation.template.operations = vec![
                EditOperationV1::EmitExactAllomorph {
                    form_ref: observation.target_form_ref,
                },
                EditOperationV1::Terminate {
                    slot_id: observation.template.target_slot_id,
                    variant_id: observation.template.variant_id,
                },
            ];
        }
    }
    Ok(support_counts)
}

pub(super) fn compatible_with_paradigm(
    incomplete: &ParadigmSignatureV1,
    paradigm: &ParadigmSignatureV1,
    anchor_scalar_len: usize,
) -> bool {
    if incomplete.pos_domain != paradigm.pos_domain || anchor_scalar_len > MAX_WIRE_SCALARS {
        return false;
    }
    let mut slot_pairs =
        BTreeMap::<(MorphologySlotKeyV1, MorphologySlotKeyV1), &ParadigmTransitionKeyV1>::new();
    for transition in &incomplete.transitions {
        let pair = (transition.source_slot, transition.target_slot);
        if slot_pairs
            .insert(pair, transition)
            .is_some_and(|previous| previous != transition)
        {
            return false;
        }
        if paradigm.transitions.binary_search(transition).is_err() {
            return false;
        }
    }
    paradigm
        .transitions
        .iter()
        .all(|transition| transition.source_ranges_valid(anchor_scalar_len))
}

fn canonical_program_bytes(operations: &[EditOperationV1]) -> Vec<u8> {
    let mut bytes = Vec::new();
    for operation in operations {
        let operation = operation.canonical_bytes();
        push_bytes(&mut bytes, &operation);
    }
    bytes
}

fn push_bytes(output: &mut Vec<u8>, bytes: &[u8]) {
    output.extend_from_slice(&(bytes.len() as u32).to_le_bytes());
    output.extend_from_slice(bytes);
}

fn checked_scalar_len(surface: &str) -> Result<u16, &'static str> {
    checked_scalar_count(surface.chars().count())
}

fn checked_scalar_count(count: usize) -> Result<u16, &'static str> {
    if count > MAX_WIRE_SCALARS {
        return Err("normalized scalar sequence reaches the wire ceiling");
    }
    u16::try_from(count).map_err(|_| "normalized scalar sequence exceeds u16")
}

fn scalars_to_string(scalars: &[char]) -> String {
    scalars.iter().collect()
}

#[cfg(test)]
mod tests {
    use super::super::types::{AXIS_INAPPLICABLE, AXIS_UNKNOWN_OR_UNANNOTATED};
    use super::*;

    fn slot(pos: u8, number: u8) -> MorphologySlotKeyV1 {
        MorphologySlotKeyV1::new(
            pos,
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
        )
    }

    fn form(
        form_ref: u32,
        slot_id: u32,
        surface: &str,
        support: u32,
        number: u8,
    ) -> CanonicalFormObservationV1 {
        CanonicalFormObservationV1 {
            form_ref,
            slot_id,
            slot: slot(2, number),
            applicability: MorphologyApplicabilityMaskV1::new(0b11).expect("mask"),
            normalized_surface: surface.to_string(),
            support,
            provenance_id: form_ref,
            variant_id: 1,
        }
    }

    #[test]
    fn canonical_anchor_uses_the_normative_total_order() {
        let incomplete = form(1, 1, "длиннее", 10, AXIS_UNKNOWN_OR_UNANNOTATED);
        let complete = form(2, 2, "короче", 10, 2);
        let less_supported = form(3, 3, "x", 9, 2);
        let forms = [incomplete, complete, less_supported];
        assert_eq!(select_canonical_anchor(&forms).expect("anchor").form_ref, 2);
    }

    #[test]
    fn scalar_dp_reconstructs_edge_and_internal_edits_byte_exactly() {
        let source = form(1, 1, "берега", 10, 1);
        let target = form(2, 2, "берёзу", 8, 2);
        let template = derive_edit_template(&source, &target).expect("template");
        assert_eq!(
            template
                .reconstruct(&source.normalized_surface, None)
                .expect("execute"),
            target.normalized_surface
        );
        assert!(matches!(
            template.operations.last(),
            Some(EditOperationV1::Terminate { .. })
        ));

        let internal_deletion =
            derive_edit_template(&form(3, 1, "abc", 10, 1), &form(4, 2, "ac", 8, 2))
                .expect("internal deletion");
        assert!(internal_deletion
            .operations
            .iter()
            .any(|operation| matches!(
                operation,
                EditOperationV1::ReplaceSourceRange {
                    delete_count: 1,
                    segment,
                    ..
                } if segment.is_empty()
            )));
        assert_eq!(
            internal_deletion
                .reconstruct("abc", None)
                .expect("delete reconstruction"),
            "ac"
        );

        let internal_insertion =
            derive_edit_template(&form(5, 1, "ac", 10, 1), &form(6, 2, "abc", 8, 2))
                .expect("internal insertion");
        assert!(internal_insertion
            .operations
            .iter()
            .any(|operation| matches!(
                operation,
                EditOperationV1::ReplaceSourceRange {
                    delete_count: 0,
                    segment,
                    ..
                } if segment == "b"
            )));
        assert_eq!(
            internal_insertion
                .reconstruct("ac", None)
                .expect("insert reconstruction"),
            "abc"
        );
    }

    #[test]
    fn end_relative_replacement_has_one_cross_length_program_identity() {
        let short = derive_edit_template(&form(1, 1, "xabc", 10, 1), &form(2, 2, "xac", 8, 2))
            .expect("short template");
        let long = derive_edit_template(
            &form(3, 1, "longprefixabc", 10, 1),
            &form(4, 2, "longprefixac", 8, 2),
        )
        .expect("long template");

        assert_eq!(short.operations, long.operations);
        assert!(short.operations.iter().any(|operation| matches!(
            operation,
            EditOperationV1::CopySourceRange {
                scalar_count: COPY_TO_RETAINED_EDGE,
                ..
            }
        )));
        assert_eq!(
            short.reconstruct("differentabc", None).expect("transfer"),
            "differentac"
        );
    }

    #[test]
    fn transfer_requires_two_distinct_train_lemmas() {
        let first_source = form(1, 1, "кот", 10, 1);
        let first_target = form(2, 2, "коты", 8, 2);
        let second_source = form(3, 1, "рот", 10, 1);
        let second_target = form(4, 2, "роты", 8, 2);
        let lone_source = form(5, 1, "друг", 10, 1);
        let lone_target = form(6, 3, "друзья", 8, 3);
        let mut observations = vec![
            LemmaTransitionObservationV1 {
                lemma_id: 1,
                target_form_ref: 2,
                template: derive_edit_template(&first_source, &first_target).expect("first"),
            },
            LemmaTransitionObservationV1 {
                lemma_id: 2,
                target_form_ref: 4,
                template: derive_edit_template(&second_source, &second_target).expect("second"),
            },
            LemmaTransitionObservationV1 {
                lemma_id: 3,
                target_form_ref: 6,
                template: derive_edit_template(&lone_source, &lone_target).expect("lone"),
            },
        ];
        apply_transfer_support_gate(&mut observations).expect("support gate");
        assert!(observations[0].template.transferable);
        assert!(observations[1].template.transferable);
        assert!(!observations[2].template.transferable);
        assert!(matches!(
            observations[2].template.operations[0],
            EditOperationV1::EmitExactAllomorph { form_ref: 6 }
        ));
    }

    #[test]
    fn compatibility_is_exact_and_checks_instantiated_ranges() {
        let source = form(1, 1, "кот", 10, 1);
        let target = form(2, 2, "коты", 8, 2);
        let template = derive_edit_template(&source, &target).expect("template");
        let transition = template.transition_key();
        let complete = ParadigmSignatureV1::new(2, [transition.clone()]).expect("complete");
        let incomplete = ParadigmSignatureV1::new(2, [transition]).expect("incomplete");
        assert!(compatible_with_paradigm(&incomplete, &complete, 3));
        assert!(complete.semantically_equivalent(&complete));
        assert!(!compatible_with_paradigm(
            &ParadigmSignatureV1::new(3, []).expect("other POS"),
            &complete,
            3
        ));
        assert!(!compatible_with_paradigm(
            &incomplete,
            &complete,
            MAX_WIRE_SCALARS + 1
        ));
    }
}
