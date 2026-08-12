use std::collections::BTreeMap;

use super::induce::{EditOperationV1, EditTemplateV1, SourceAnchorV1, COPY_TO_RETAINED_EDGE};

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(super) struct ProductiveTerminalAttributionV1 {
    pub(super) program_id: u32,
    pub(super) target_slot_id: u32,
    pub(super) variant_id: u16,
    pub(super) decoder_ref: u32,
    pub(super) evidence_ref: u32,
    pub(super) calibration_class: u16,
    pub(super) provenance_ref: u32,
    pub(super) stable_identity_hash: u32,
}

#[derive(Clone, Debug)]
pub(super) struct TrieProgramInputV1 {
    pub(super) paradigm_id: u32,
    pub(super) anchor_scalar_len: u16,
    pub(super) template: EditTemplateV1,
    pub(super) exact_allomorph_surface: Option<String>,
    pub(super) terminal: ProductiveTerminalAttributionV1,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum AtomicTrieTokenV1 {
    CopySourceScalar {
        source_anchor: SourceAnchorV1,
        source_delta: i16,
    },
    CopyToRetainedEdge {
        source_anchor: SourceAnchorV1,
        source_delta: i16,
        retained_end_delta: i16,
    },
    DropSourcePrefix {
        scalar_count: u16,
    },
    DropSourceSuffix {
        scalar_count: u16,
    },
    EmitScalar {
        scalar: char,
    },
    ReplaceSourceStart {
        end_relative_offset: i16,
        delete_count: u16,
    },
    EmitExactAllomorph {
        form_ref: u32,
    },
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(super) enum ProductiveTrieArcActionV1 {
    CopySourceRange {
        source_anchor: SourceAnchorV1,
        source_delta: i16,
        scalar_count: u16,
    },
    CopyToRetainedEdge {
        source_anchor: SourceAnchorV1,
        source_delta: i16,
        retained_end_delta: i16,
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
    ReplaceSourceStart {
        end_relative_offset: i16,
        delete_count: u16,
    },
    EmitExactAllomorph {
        form_ref: u32,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct ProductiveTrieArcV1 {
    pub(super) action: ProductiveTrieArcActionV1,
    pub(super) child_node: u32,
    pub(super) order_target_slot_id: u32,
    pub(super) order_variant_id: u16,
    pub(super) stable_order: u32,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(super) struct ProductiveTrieNodeV1 {
    pub(super) parent: Option<u32>,
    pub(super) arcs: Vec<ProductiveTrieArcV1>,
    pub(super) terminals: Vec<ProductiveTerminalAttributionV1>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(super) struct ProductiveTrieForestV1 {
    pub(super) roots_by_paradigm: BTreeMap<u32, u32>,
    pub(super) nodes: Vec<ProductiveTrieNodeV1>,
    exact_allomorph_surfaces: BTreeMap<u32, String>,
}

#[derive(Clone, Debug, Default)]
struct AtomicBuildNodeV1 {
    children: BTreeMap<AtomicTrieTokenV1, u32>,
    terminals: Vec<ProductiveTerminalAttributionV1>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct InstantiatedProductiveTerminalV1 {
    pub(super) normalized_surface: String,
    pub(super) terminal: ProductiveTerminalAttributionV1,
}

pub(super) fn compile_productive_trie(
    programs: &[TrieProgramInputV1],
) -> Result<ProductiveTrieForestV1, &'static str> {
    let mut ordered = programs.iter().collect::<Vec<_>>();
    ordered.sort_by(|left, right| {
        left.paradigm_id
            .cmp(&right.paradigm_id)
            .then_with(|| {
                left.template
                    .source_slot_id
                    .cmp(&right.template.source_slot_id)
            })
            .then_with(|| {
                left.template
                    .target_slot_id
                    .cmp(&right.template.target_slot_id)
            })
            .then_with(|| left.template.variant_id.cmp(&right.template.variant_id))
            .then_with(|| left.terminal.program_id.cmp(&right.terminal.program_id))
    });
    if ordered.iter().any(|program| {
        program.paradigm_id == 0
            || program.terminal.program_id == 0
            || program.terminal.target_slot_id != program.template.target_slot_id
            || program.terminal.variant_id != program.template.variant_id
    }) {
        return Err("productive trie input contains an unowned or inconsistent identity");
    }

    let mut atomic_nodes = Vec::<AtomicBuildNodeV1>::new();
    let mut atomic_roots = BTreeMap::<u32, u32>::new();
    let mut exact_allomorph_surfaces = BTreeMap::<u32, String>::new();
    for program in ordered {
        let root = *atomic_roots.entry(program.paradigm_id).or_insert_with(|| {
            let root = atomic_nodes.len() as u32;
            atomic_nodes.push(AtomicBuildNodeV1::default());
            root
        });
        if let Some(surface) = &program.exact_allomorph_surface {
            let exact_ref = program
                .template
                .operations
                .iter()
                .find_map(|operation| match operation {
                    EditOperationV1::EmitExactAllomorph { form_ref } => Some(*form_ref),
                    _ => None,
                })
                .ok_or("exact allomorph surface lacks an exact allomorph operation")?;
            if exact_allomorph_surfaces
                .insert(exact_ref, surface.clone())
                .is_some_and(|previous| previous != *surface)
            {
                return Err("one exact allomorph reference resolves to multiple surfaces");
            }
        }
        let tokens = atomic_tokens(program)?;
        let mut node = root;
        for token in tokens {
            let child = if let Some(child) = atomic_nodes[node as usize].children.get(&token) {
                *child
            } else {
                let child = atomic_nodes.len() as u32;
                atomic_nodes.push(AtomicBuildNodeV1 {
                    ..AtomicBuildNodeV1::default()
                });
                atomic_nodes[node as usize].children.insert(token, child);
                child
            };
            node = child;
        }
        atomic_nodes[node as usize]
            .terminals
            .push(program.terminal.clone());
    }
    for node in &mut atomic_nodes {
        node.terminals.sort_unstable();
        node.terminals.dedup();
    }

    let mut forest = ProductiveTrieForestV1 {
        roots_by_paradigm: BTreeMap::new(),
        nodes: Vec::new(),
        exact_allomorph_surfaces,
    };
    for (paradigm_id, atomic_root) in atomic_roots {
        let root = emit_compacted_node(&atomic_nodes, atomic_root, None, &mut forest.nodes)?;
        forest.roots_by_paradigm.insert(paradigm_id, root);
    }
    forest.validate()?;
    Ok(forest)
}

fn atomic_tokens(program: &TrieProgramInputV1) -> Result<Vec<AtomicTrieTokenV1>, &'static str> {
    if program.anchor_scalar_len == u16::MAX {
        return Err("productive trie anchor reaches the wire ceiling");
    }
    let source_len = usize::from(program.anchor_scalar_len);
    let suffix_drop = program
        .template
        .operations
        .iter()
        .filter_map(|operation| match operation {
            EditOperationV1::DropSourceSuffix { scalar_count } => Some(usize::from(*scalar_count)),
            _ => None,
        })
        .sum::<usize>();
    let retained_end_delta = i16::try_from(
        -isize::try_from(suffix_drop).map_err(|_| "productive suffix drop exceeds isize")?,
    )
    .map_err(|_| "productive suffix drop exceeds signed wire offset")?;
    let mut tokens = Vec::new();
    let mut terminated = false;
    for (operation_index, operation) in program.template.operations.iter().enumerate() {
        if terminated {
            return Err("productive trie program contains operations after terminate");
        }
        match operation {
            EditOperationV1::CopySourceRange {
                start_anchor,
                start_delta,
                scalar_count,
            } => {
                let start = resolve_source_offset(source_len, *start_anchor, *start_delta)
                    .ok_or("productive trie source offset is outside the anchor")?;
                let end = if *scalar_count == COPY_TO_RETAINED_EDGE {
                    source_len
                        .checked_sub(suffix_drop)
                        .ok_or("productive suffix drop exceeds source length")?
                } else {
                    start
                        .checked_add(usize::from(*scalar_count))
                        .ok_or("productive trie copy range overflow")?
                };
                if *scalar_count == 0 || end <= start || end > source_len {
                    return Err("productive trie copy range is outside the anchor");
                }
                if *scalar_count == COPY_TO_RETAINED_EDGE {
                    let dynamic_retained_end_delta =
                        match program.template.operations.get(operation_index + 1) {
                            Some(EditOperationV1::ReplaceSourceRange {
                                end_relative_offset,
                                ..
                            }) => *end_relative_offset,
                            _ => retained_end_delta,
                        };
                    tokens.push(AtomicTrieTokenV1::CopyToRetainedEdge {
                        source_anchor: *start_anchor,
                        source_delta: *start_delta,
                        retained_end_delta: dynamic_retained_end_delta,
                    });
                    continue;
                }
                for offset in 0..usize::from(*scalar_count) {
                    tokens.push(AtomicTrieTokenV1::CopySourceScalar {
                        source_anchor: *start_anchor,
                        source_delta: start_delta
                            .checked_add(
                                i16::try_from(offset)
                                    .map_err(|_| "productive trie copy offset exceeds i16")?,
                            )
                            .ok_or("productive trie copy delta overflow")?,
                    });
                }
            }
            EditOperationV1::DropSourcePrefix { scalar_count } => {
                tokens.push(AtomicTrieTokenV1::DropSourcePrefix {
                    scalar_count: *scalar_count,
                });
            }
            EditOperationV1::DropSourceSuffix { scalar_count } => {
                tokens.push(AtomicTrieTokenV1::DropSourceSuffix {
                    scalar_count: *scalar_count,
                });
            }
            EditOperationV1::EmitSegment { segment } => {
                if segment.is_empty() {
                    return Err("productive trie cannot encode an empty segment");
                }
                tokens.extend(
                    segment
                        .chars()
                        .map(|scalar| AtomicTrieTokenV1::EmitScalar { scalar }),
                );
            }
            EditOperationV1::ReplaceSourceRange {
                end_relative_offset,
                delete_count,
                segment,
            } => {
                let start = source_len
                    .checked_add_signed(isize::from(*end_relative_offset))
                    .ok_or("productive trie replacement offset is outside the anchor")?;
                if start
                    .checked_add(usize::from(*delete_count))
                    .is_none_or(|end| end > source_len)
                {
                    return Err("productive trie replacement range is outside the anchor");
                }
                tokens.push(AtomicTrieTokenV1::ReplaceSourceStart {
                    end_relative_offset: *end_relative_offset,
                    delete_count: *delete_count,
                });
                tokens.extend(
                    segment
                        .chars()
                        .map(|scalar| AtomicTrieTokenV1::EmitScalar { scalar }),
                );
            }
            EditOperationV1::EmitExactAllomorph { form_ref } => {
                if program.template.transferable || program.exact_allomorph_surface.is_none() {
                    return Err("exact allomorph is transferable or lacks its lemma-local decoder");
                }
                tokens.push(AtomicTrieTokenV1::EmitExactAllomorph {
                    form_ref: *form_ref,
                });
            }
            EditOperationV1::Terminate {
                slot_id,
                variant_id,
            } => {
                if *slot_id != program.terminal.target_slot_id
                    || *variant_id != program.terminal.variant_id
                {
                    return Err("productive trie terminate identity disagrees with terminal");
                }
                terminated = true;
            }
        }
    }
    if !terminated {
        return Err("productive trie program lacks terminate");
    }
    Ok(tokens)
}

fn emit_compacted_node(
    atomic_nodes: &[AtomicBuildNodeV1],
    atomic_node: u32,
    parent: Option<u32>,
    output: &mut Vec<ProductiveTrieNodeV1>,
) -> Result<u32, &'static str> {
    let output_node = output.len() as u32;
    output.push(ProductiveTrieNodeV1 {
        parent,
        terminals: atomic_nodes[atomic_node as usize].terminals.clone(),
        arcs: Vec::new(),
    });
    let mut arcs = Vec::new();
    for (token, child) in &atomic_nodes[atomic_node as usize].children {
        let mut tokens = vec![token.clone()];
        let mut endpoint = *child;
        loop {
            let node = &atomic_nodes[endpoint as usize];
            if !node.terminals.is_empty() || node.children.len() != 1 {
                break;
            }
            let (next_token, next_child) = node.children.first_key_value().expect("one child");
            if !can_compact(&tokens, next_token) {
                break;
            }
            tokens.push(next_token.clone());
            endpoint = *next_child;
        }
        let child_node = emit_compacted_node(atomic_nodes, endpoint, Some(output_node), output)?;
        let descendant_order = minimum_terminal_identity(atomic_nodes, endpoint)
            .ok_or("productive trie branch has no terminal descendant")?;
        arcs.push(ProductiveTrieArcV1 {
            action: compact_action(&tokens)?,
            child_node,
            order_target_slot_id: descendant_order.target_slot_id,
            order_variant_id: descendant_order.variant_id,
            stable_order: descendant_order.stable_identity_hash,
        });
    }
    arcs.sort_by(|left, right| {
        left.action
            .cmp(&right.action)
            .then_with(|| left.order_target_slot_id.cmp(&right.order_target_slot_id))
            .then_with(|| left.order_variant_id.cmp(&right.order_variant_id))
            .then_with(|| left.stable_order.cmp(&right.stable_order))
    });
    output[output_node as usize].arcs = arcs;
    Ok(output_node)
}

fn can_compact(prefix: &[AtomicTrieTokenV1], next: &AtomicTrieTokenV1) -> bool {
    match (prefix.first(), prefix.last(), next) {
        (
            Some(AtomicTrieTokenV1::CopySourceScalar { .. }),
            Some(AtomicTrieTokenV1::CopySourceScalar {
                source_anchor: previous_anchor,
                source_delta: previous_delta,
            }),
            AtomicTrieTokenV1::CopySourceScalar {
                source_anchor: current_anchor,
                source_delta: current_delta,
            },
        ) => {
            previous_anchor == current_anchor
                && previous_delta.checked_add(1) == Some(*current_delta)
        }
        (
            Some(AtomicTrieTokenV1::EmitScalar { .. }),
            Some(AtomicTrieTokenV1::EmitScalar { .. }),
            AtomicTrieTokenV1::EmitScalar { .. },
        ) => true,
        _ => false,
    }
}

fn compact_action(tokens: &[AtomicTrieTokenV1]) -> Result<ProductiveTrieArcActionV1, &'static str> {
    match tokens.first().ok_or("productive trie arc has no token")? {
        AtomicTrieTokenV1::CopySourceScalar {
            source_anchor,
            source_delta,
        } => {
            if !tokens
                .iter()
                .all(|token| matches!(token, AtomicTrieTokenV1::CopySourceScalar { .. }))
            {
                return Err("productive trie copy arc mixes token kinds");
            }
            Ok(ProductiveTrieArcActionV1::CopySourceRange {
                source_anchor: *source_anchor,
                source_delta: *source_delta,
                scalar_count: u16::try_from(tokens.len())
                    .map_err(|_| "productive trie copy arc exceeds u16")?,
            })
        }
        AtomicTrieTokenV1::CopyToRetainedEdge {
            source_anchor,
            source_delta,
            retained_end_delta,
        } if tokens.len() == 1 => Ok(ProductiveTrieArcActionV1::CopyToRetainedEdge {
            source_anchor: *source_anchor,
            source_delta: *source_delta,
            retained_end_delta: *retained_end_delta,
        }),
        AtomicTrieTokenV1::EmitScalar { .. } => {
            let segment = tokens
                .iter()
                .map(|token| match token {
                    AtomicTrieTokenV1::EmitScalar { scalar } => Ok(*scalar),
                    _ => Err("productive trie segment arc mixes token kinds"),
                })
                .collect::<Result<String, _>>()?;
            Ok(ProductiveTrieArcActionV1::EmitSegment { segment })
        }
        AtomicTrieTokenV1::DropSourcePrefix { scalar_count } if tokens.len() == 1 => {
            Ok(ProductiveTrieArcActionV1::DropSourcePrefix {
                scalar_count: *scalar_count,
            })
        }
        AtomicTrieTokenV1::DropSourceSuffix { scalar_count } if tokens.len() == 1 => {
            Ok(ProductiveTrieArcActionV1::DropSourceSuffix {
                scalar_count: *scalar_count,
            })
        }
        AtomicTrieTokenV1::ReplaceSourceStart {
            end_relative_offset,
            delete_count,
        } if tokens.len() == 1 => Ok(ProductiveTrieArcActionV1::ReplaceSourceStart {
            end_relative_offset: *end_relative_offset,
            delete_count: *delete_count,
        }),
        AtomicTrieTokenV1::EmitExactAllomorph { form_ref } if tokens.len() == 1 => {
            Ok(ProductiveTrieArcActionV1::EmitExactAllomorph {
                form_ref: *form_ref,
            })
        }
        _ => Err("productive trie arc cannot be compacted canonically"),
    }
}

fn minimum_terminal_identity(
    nodes: &[AtomicBuildNodeV1],
    node: u32,
) -> Option<ProductiveTerminalAttributionV1> {
    let current = &nodes[node as usize];
    current
        .terminals
        .iter()
        .cloned()
        .chain(
            current
                .children
                .values()
                .filter_map(|child| minimum_terminal_identity(nodes, *child)),
        )
        .min()
}

impl ProductiveTrieForestV1 {
    pub(super) fn exact_allomorph_surface(&self, form_ref: u32) -> Option<&str> {
        self.exact_allomorph_surfaces
            .get(&form_ref)
            .map(String::as_str)
    }

    pub(super) fn validate(&self) -> Result<(), &'static str> {
        for (index, node) in self.nodes.iter().enumerate() {
            if node.parent == Some(index as u32) {
                return Err("productive trie node is its own parent");
            }
            if index > 0
                && node.parent.is_none()
                && !self
                    .roots_by_paradigm
                    .values()
                    .any(|root| *root == index as u32)
            {
                return Err("productive trie non-root node lacks a parent");
            }
            if node
                .arcs
                .windows(2)
                .any(|pair| arc_order(&pair[0], &pair[1]).is_gt())
            {
                return Err("productive trie arcs are not in canonical order");
            }
            for arc in &node.arcs {
                let child = self
                    .nodes
                    .get(arc.child_node as usize)
                    .ok_or("productive trie arc child is outside the node bank")?;
                if child.parent != Some(index as u32) {
                    return Err("productive trie child has more than one logical parent");
                }
            }
            if node.terminals.windows(2).any(|pair| pair[0] >= pair[1]) {
                return Err("productive trie terminals are duplicated or unsorted");
            }
        }
        for root in self.roots_by_paradigm.values() {
            if self
                .nodes
                .get(*root as usize)
                .is_none_or(|node| node.parent.is_some())
            {
                return Err("productive trie root is invalid");
            }
        }
        Ok(())
    }

    pub(super) fn instantiate(
        &self,
        paradigm_id: u32,
        canonical_source: &str,
    ) -> Result<Vec<InstantiatedProductiveTerminalV1>, &'static str> {
        let root = *self
            .roots_by_paradigm
            .get(&paradigm_id)
            .ok_or("productive trie lacks the requested paradigm")?;
        let source = canonical_source.chars().collect::<Vec<_>>();
        let mut output = Vec::new();
        let mut stack = vec![(root, 0_usize, String::new(), false)];
        while let Some((node_id, cursor, surface, exact)) = stack.pop() {
            let node = self
                .nodes
                .get(node_id as usize)
                .ok_or("productive trie traversal reached an invalid node")?;
            if exact || cursor == source.len() {
                output.extend(node.terminals.iter().cloned().map(|terminal| {
                    InstantiatedProductiveTerminalV1 {
                        normalized_surface: surface.clone(),
                        terminal,
                    }
                }));
            }
            for arc in node.arcs.iter().rev() {
                if let Some(next) = apply_arc(
                    &arc.action,
                    &source,
                    cursor,
                    &surface,
                    exact,
                    &self.exact_allomorph_surfaces,
                )? {
                    stack.push((arc.child_node, next.0, next.1, next.2));
                }
            }
        }
        output.sort_by(|left, right| {
            left.terminal
                .cmp(&right.terminal)
                .then_with(|| left.normalized_surface.cmp(&right.normalized_surface))
        });
        output.dedup();
        Ok(output)
    }
}

fn arc_order(left: &ProductiveTrieArcV1, right: &ProductiveTrieArcV1) -> std::cmp::Ordering {
    left.action
        .cmp(&right.action)
        .then_with(|| left.order_target_slot_id.cmp(&right.order_target_slot_id))
        .then_with(|| left.order_variant_id.cmp(&right.order_variant_id))
        .then_with(|| left.stable_order.cmp(&right.stable_order))
}

fn apply_arc(
    action: &ProductiveTrieArcActionV1,
    source: &[char],
    cursor: usize,
    surface: &str,
    exact: bool,
    exact_surfaces: &BTreeMap<u32, String>,
) -> Result<Option<(usize, String, bool)>, &'static str> {
    if exact {
        return Ok(None);
    }
    let mut next_cursor = cursor;
    let mut next_surface = surface.to_string();
    let mut next_exact = false;
    match action {
        ProductiveTrieArcActionV1::CopySourceRange {
            source_anchor,
            source_delta,
            scalar_count,
        } => {
            let Some(start) = resolve_source_offset(source.len(), *source_anchor, *source_delta)
            else {
                return Ok(None);
            };
            let end = start
                .checked_add(usize::from(*scalar_count))
                .ok_or("productive trie runtime copy overflow")?;
            if start != cursor || end > source.len() {
                return Ok(None);
            }
            next_surface.extend(source[start..end].iter());
            next_cursor = end;
        }
        ProductiveTrieArcActionV1::CopyToRetainedEdge {
            source_anchor,
            source_delta,
            retained_end_delta,
        } => {
            let Some(start) = resolve_source_offset(source.len(), *source_anchor, *source_delta)
            else {
                return Ok(None);
            };
            let Some(end) = source
                .len()
                .checked_add_signed(isize::from(*retained_end_delta))
            else {
                return Ok(None);
            };
            if start != cursor || end <= start || end > source.len() {
                return Ok(None);
            }
            next_surface.extend(source[start..end].iter());
            next_cursor = end;
        }
        ProductiveTrieArcActionV1::DropSourcePrefix { scalar_count } => {
            if cursor != 0 || *scalar_count == 0 || usize::from(*scalar_count) > source.len() {
                return Ok(None);
            }
            next_cursor = usize::from(*scalar_count);
        }
        ProductiveTrieArcActionV1::DropSourceSuffix { scalar_count } => {
            if *scalar_count == 0
                || cursor.checked_add(usize::from(*scalar_count)) != Some(source.len())
            {
                return Ok(None);
            }
            next_cursor = source.len();
        }
        ProductiveTrieArcActionV1::EmitSegment { segment } => {
            next_surface.push_str(segment);
        }
        ProductiveTrieArcActionV1::ReplaceSourceStart {
            end_relative_offset,
            delete_count,
        } => {
            let Some(start) = source
                .len()
                .checked_add_signed(isize::from(*end_relative_offset))
            else {
                return Ok(None);
            };
            let end = start
                .checked_add(usize::from(*delete_count))
                .ok_or("productive trie runtime replacement overflow")?;
            if start != cursor || end > source.len() {
                return Ok(None);
            }
            next_cursor = end;
        }
        ProductiveTrieArcActionV1::EmitExactAllomorph { form_ref } => {
            next_surface = exact_surfaces
                .get(form_ref)
                .cloned()
                .ok_or("productive trie exact decoder reference is unavailable")?;
            next_cursor = source.len();
            next_exact = true;
        }
    }
    Ok(Some((next_cursor, next_surface, next_exact)))
}

fn resolve_source_offset(source_len: usize, anchor: SourceAnchorV1, delta: i16) -> Option<usize> {
    let base = match anchor {
        SourceAnchorV1::Start => 0,
        SourceAnchorV1::End => source_len,
    };
    base.checked_add_signed(isize::from(delta))
}

#[cfg(test)]
mod tests {
    use super::super::induce::{derive_edit_template, CanonicalFormObservationV1};
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

    fn program(
        paradigm_id: u32,
        program_id: u32,
        source: &CanonicalFormObservationV1,
        target: &CanonicalFormObservationV1,
    ) -> TrieProgramInputV1 {
        let mut template = derive_edit_template(source, target).expect("template");
        template.transferable = true;
        TrieProgramInputV1 {
            paradigm_id,
            anchor_scalar_len: source.normalized_surface.chars().count() as u16,
            template,
            exact_allomorph_surface: None,
            terminal: ProductiveTerminalAttributionV1 {
                program_id,
                target_slot_id: target.slot_id,
                variant_id: target.variant_id,
                decoder_ref: target.form_ref,
                evidence_ref: program_id,
                calibration_class: 1,
                provenance_ref: 1,
                stable_identity_hash: program_id,
            },
        }
    }

    #[test]
    fn trie_shares_scalar_prefixes_and_keeps_one_parent() {
        let source = form(1, 1, "кот", 1);
        let plural = form(2, 2, "коты", 2);
        let adjectival = form(3, 3, "котов", 3);
        let forest = compile_productive_trie(&[
            program(1, 1, &source, &plural),
            program(1, 2, &source, &adjectival),
        ])
        .expect("compile");
        forest.validate().expect("validate");
        assert_eq!(forest.roots_by_paradigm.len(), 1);
        assert!(forest
            .nodes
            .iter()
            .skip(1)
            .all(|node| node.parent.is_some()));
        let root = forest.roots_by_paradigm[&1];
        assert_eq!(forest.nodes[root as usize].arcs.len(), 1);
    }

    #[test]
    fn trie_instantiation_matches_edit_program_candidates_and_order() {
        let source = form(1, 1, "кот", 1);
        let targets = [form(2, 2, "коты", 2), form(3, 3, "котов", 3)];
        let programs = targets
            .iter()
            .enumerate()
            .map(|(index, target)| program(1, index as u32 + 1, &source, target))
            .collect::<Vec<_>>();
        let forest = compile_productive_trie(&programs).expect("compile");
        let instantiated = forest.instantiate(1, "кот").expect("instantiate");
        assert_eq!(
            instantiated
                .iter()
                .map(|candidate| candidate.normalized_surface.as_str())
                .collect::<Vec<_>>(),
            vec!["коты", "котов"]
        );
    }

    #[test]
    fn trie_compile_is_byte_order_deterministic_under_input_permutation() {
        let source = form(1, 1, "кот", 1);
        let first = program(1, 1, &source, &form(2, 2, "коты", 2));
        let second = program(1, 2, &source, &form(3, 3, "котов", 3));
        let forward = compile_productive_trie(&[first.clone(), second.clone()]).expect("forward");
        let reverse = compile_productive_trie(&[second, first]).expect("reverse");
        assert_eq!(forward, reverse);
    }

    #[test]
    fn relative_anchor_program_instantiates_for_a_different_length_lemma() {
        let source = form(1, 1, "кот", 1);
        let target = form(2, 2, "коты", 2);
        let forest = compile_productive_trie(&[program(1, 1, &source, &target)]).expect("trie");
        let instantiated = forest.instantiate(1, "слон").expect("instantiate");
        assert_eq!(instantiated[0].normalized_surface, "слоны");
    }

    #[test]
    fn end_relative_replacement_keeps_the_variable_stem_length() {
        let source = form(1, 1, "prefixabc", 1);
        let target = form(2, 2, "prefixac", 2);
        let forest = compile_productive_trie(&[program(1, 1, &source, &target)]).expect("trie");

        let instantiated = forest.instantiate(1, "xabc").expect("instantiate");

        assert_eq!(instantiated[0].normalized_surface, "xac");
    }
}
