use super::{
    BandedLevenshteinRow, LevenshteinRow, Phase7dRetrievalLane, SearchBudget, V13DafsaView,
    V13Identity, MAX_BAND_CELLS, TERMINAL_FLAG,
};

pub(super) const ALPHABET_SYMBOLS: usize = 34;
pub(super) const EXPECTED_STATE_COUNT: usize = 81_128;
pub(super) const EXPECTED_EDGE_COUNT: usize = 226_341;
pub(super) const EXPECTED_SYMBOL_COUNT: usize = 34;
pub(super) const EXPECTED_ROOT_STATE: u32 = 81_127;
pub(super) const EXPECTED_TYPED_PAYLOAD_BYTES: usize = 3_689_628;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct TypedState {
    first_edge: u32,
    suffix_count: u32,
    edge_count: u16,
    flags: u16,
}

impl TypedState {
    #[inline(always)]
    fn terminal(self) -> bool {
        self.flags & TERMINAL_FLAG != 0
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct TypedEdge {
    symbol: u32,
    target: u32,
    rank_delta: u32,
}

pub(super) struct TypedExactView {
    states: Box<[TypedState]>,
    edges: Box<[TypedEdge]>,
    root_state: u32,
    identity: V13Identity,
    symbol_digest: [u8; 32],
}

impl TypedExactView {
    #[inline(always)]
    fn state(&self, state_id: u32) -> Result<TypedState, String> {
        self.states
            .get(state_id as usize)
            .copied()
            .ok_or_else(|| format!("typed V13 state {state_id} is out of range"))
    }

    #[inline(always)]
    fn edges(&self, state: TypedState) -> Result<&[TypedEdge], String> {
        let start = state.first_edge as usize;
        let end = start
            .checked_add(state.edge_count as usize)
            .ok_or_else(|| "typed V13 edge range overflows usize".to_string())?;
        self.edges
            .get(start..end)
            .ok_or_else(|| "typed V13 edge range is out of bounds".to_string())
    }

    pub(super) fn state_count(&self) -> usize {
        self.states.len()
    }

    pub(super) fn edge_count(&self) -> usize {
        self.edges.len()
    }

    pub(super) fn root_state(&self) -> u32 {
        self.root_state
    }

    pub(super) fn identity(&self) -> V13Identity {
        self.identity
    }

    pub(super) fn symbol_digest(&self) -> [u8; 32] {
        self.symbol_digest
    }
}

pub(super) struct TypedMaterialization {
    view: TypedExactView,
    payload_bytes: usize,
    states_checked: usize,
    edges_checked: usize,
}

impl TypedMaterialization {
    pub(super) fn from_validated(index: &V13DafsaView) -> Result<Self, String> {
        if std::mem::size_of::<TypedState>() != 12 || std::mem::size_of::<TypedEdge>() != 12 {
            return Err("typed V13 record width drift".to_string());
        }
        if index.state_count as usize != EXPECTED_STATE_COUNT
            || index.edge_count as usize != EXPECTED_EDGE_COUNT
            || index.symbol_count as usize != EXPECTED_SYMBOL_COUNT
            || index.root_state != EXPECTED_ROOT_STATE
        {
            return Err(format!(
                "typed V13 fixed identity drift: states={} edges={} symbols={} root={}",
                index.state_count, index.edge_count, index.symbol_count, index.root_state
            ));
        }

        let mut states = Vec::with_capacity(EXPECTED_STATE_COUNT);
        for state_id in 0..index.state_count {
            let decoded = index.state(state_id)?;
            let typed = TypedState {
                first_edge: decoded.first_edge,
                suffix_count: decoded.suffix_count,
                edge_count: decoded.edge_count,
                flags: decoded.flags,
            };
            if typed.first_edge != decoded.first_edge
                || typed.suffix_count != decoded.suffix_count
                || typed.edge_count != decoded.edge_count
                || typed.flags != decoded.flags
                || typed.terminal() != decoded.terminal()
            {
                return Err(format!("typed V13 state field mismatch at {state_id}"));
            }
            states.push(typed);
        }

        let mut seen_alphabet = [false; ALPHABET_SYMBOLS];
        let mut edges = Vec::with_capacity(EXPECTED_EDGE_COUNT);
        for edge_id in 0..index.edge_count as usize {
            let decoded = index.edge(edge_id)?;
            let alphabet_id = alphabet_id(decoded.symbol).ok_or_else(|| {
                format!(
                    "typed V13 edge {edge_id} has unsupported U+{:04X}",
                    decoded.symbol
                )
            })?;
            seen_alphabet[alphabet_id] = true;
            let typed = TypedEdge {
                symbol: decoded.symbol,
                target: decoded.target,
                rank_delta: decoded.rank_delta,
            };
            if typed.symbol != decoded.symbol
                || typed.target != decoded.target
                || typed.rank_delta != decoded.rank_delta
            {
                return Err(format!("typed V13 edge field mismatch at {edge_id}"));
            }
            edges.push(typed);
        }
        if seen_alphabet.iter().any(|seen| !seen) {
            return Err("typed V13 dense alphabet is incomplete".to_string());
        }

        let payload_bytes = states
            .len()
            .checked_mul(std::mem::size_of::<TypedState>())
            .and_then(|state_bytes| {
                edges
                    .len()
                    .checked_mul(std::mem::size_of::<TypedEdge>())
                    .and_then(|edge_bytes| state_bytes.checked_add(edge_bytes))
            })
            .ok_or_else(|| "typed V13 payload size overflows usize".to_string())?;
        if payload_bytes != EXPECTED_TYPED_PAYLOAD_BYTES {
            return Err(format!("typed V13 payload drift: {payload_bytes}"));
        }

        let view = TypedExactView {
            states: states.into_boxed_slice(),
            edges: edges.into_boxed_slice(),
            root_state: index.root_state,
            identity: index.identity,
            symbol_digest: index.symbol_digest,
        };
        if view.identity != index.identity
            || view.symbol_digest != index.symbol_digest
            || view.state(view.root_state)?.suffix_count
                != index.state(index.root_state)?.suffix_count
        {
            return Err("typed V13 root or identity closure drift".to_string());
        }

        Ok(Self {
            view,
            payload_bytes,
            states_checked: EXPECTED_STATE_COUNT,
            edges_checked: EXPECTED_EDGE_COUNT,
        })
    }

    pub(super) fn view(&self) -> &TypedExactView {
        &self.view
    }

    pub(super) fn payload_bytes(&self) -> usize {
        self.payload_bytes
    }

    pub(super) fn states_checked(&self) -> usize {
        self.states_checked
    }

    pub(super) fn edges_checked(&self) -> usize {
        self.edges_checked
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) struct StructuralWork {
    pub(super) expanded_states: usize,
    pub(super) examined_edges: usize,
    pub(super) surviving_edges: usize,
    pub(super) pruned_edges: usize,
    pub(super) stack_pushes: usize,
    pub(super) stack_pops: usize,
    pub(super) terminal_hits: usize,
}

impl StructuralWork {
    pub(super) fn add(&mut self, other: Self) {
        self.expanded_states = self.expanded_states.saturating_add(other.expanded_states);
        self.examined_edges = self.examined_edges.saturating_add(other.examined_edges);
        self.surviving_edges = self.surviving_edges.saturating_add(other.surviving_edges);
        self.pruned_edges = self.pruned_edges.saturating_add(other.pruned_edges);
        self.stack_pushes = self.stack_pushes.saturating_add(other.stack_pushes);
        self.stack_pops = self.stack_pops.saturating_add(other.stack_pops);
        self.terminal_hits = self.terminal_hits.saturating_add(other.terminal_hits);
    }
}

pub(super) struct ExactObservation {
    pub(super) retrieved_form_refs: Vec<u32>,
    pub(super) unresolved: Option<&'static str>,
    pub(super) expanded_product_states: usize,
    pub(super) maximum_scratch_bytes: usize,
    pub(super) work: StructuralWork,
    pub(super) rank_prefixes: Vec<u32>,
    pub(super) terminal_ranks: Vec<u32>,
    pub(super) transition_checks: usize,
    pub(super) terminal_distance_checks: usize,
}

struct LaneObservation {
    form_refs: Vec<u32>,
    expanded: usize,
    maximum_scratch: usize,
    unresolved: Option<&'static str>,
    work: StructuralWork,
    transition_checks: usize,
    terminal_distance_checks: usize,
}

#[derive(Clone, Copy)]
pub(super) struct PackedNode {
    pub(super) state_id: u32,
    pub(super) rank_prefix: u32,
    pub(super) row: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct FusedOutput {
    pub(super) state: u64,
    pub(super) minimum: u8,
}

pub(super) fn search<F>(
    index: &TypedExactView,
    lanes: &[Phase7dRetrievalLane],
    budget: SearchBudget,
    reverse_schedule: bool,
    verify_transitions: bool,
    mut deadline_exceeded: F,
) -> Result<ExactObservation, String>
where
    F: FnMut() -> bool,
{
    let mut terminal_refs = Vec::new();
    let mut expanded = 0_usize;
    let mut maximum_scratch = 0_usize;
    let mut work = StructuralWork::default();
    let mut rank_prefixes = Vec::new();
    let mut terminal_ranks = Vec::new();
    let mut transition_checks = 0_usize;
    let mut terminal_distance_checks = 0_usize;

    for lane in lanes {
        let masks = equality_masks(lane.symbols.as_ref());
        let outcome = enumerate_lane(
            index,
            lane,
            &masks,
            budget,
            reverse_schedule,
            verify_transitions,
            &mut deadline_exceeded,
            &mut rank_prefixes,
            &mut terminal_ranks,
        )?;
        work.add(outcome.work);
        expanded = expanded.saturating_add(outcome.expanded);
        maximum_scratch = maximum_scratch.max(outcome.maximum_scratch);
        transition_checks = transition_checks.saturating_add(outcome.transition_checks);
        terminal_distance_checks =
            terminal_distance_checks.saturating_add(outcome.terminal_distance_checks);
        if let Some(reason) = outcome.unresolved {
            return Ok(ExactObservation {
                retrieved_form_refs: Vec::new(),
                unresolved: Some(reason),
                expanded_product_states: expanded,
                maximum_scratch_bytes: maximum_scratch,
                work,
                rank_prefixes,
                terminal_ranks,
                transition_checks,
                terminal_distance_checks,
            });
        }
        terminal_refs.extend(outcome.form_refs);
    }

    terminal_refs.sort_unstable();
    terminal_refs.dedup();
    Ok(ExactObservation {
        retrieved_form_refs: terminal_refs,
        unresolved: None,
        expanded_product_states: expanded,
        maximum_scratch_bytes: maximum_scratch,
        work,
        rank_prefixes,
        terminal_ranks,
        transition_checks,
        terminal_distance_checks,
    })
}

#[allow(clippy::too_many_arguments)]
fn enumerate_lane<F>(
    index: &TypedExactView,
    lane: &Phase7dRetrievalLane,
    masks: &[[u64; 2]; ALPHABET_SYMBOLS],
    budget: SearchBudget,
    reverse_schedule: bool,
    verify_transitions: bool,
    deadline_exceeded: &mut F,
    rank_prefixes: &mut Vec<u32>,
    terminal_ranks: &mut Vec<u32>,
) -> Result<LaneObservation, String>
where
    F: FnMut() -> bool,
{
    if let Some(reason) = super::validate_lane(lane) {
        return Ok(LaneObservation {
            form_refs: Vec::new(),
            expanded: 0,
            maximum_scratch: 0,
            unresolved: Some(reason),
            work: StructuralWork::default(),
            transition_checks: 0,
            terminal_distance_checks: 0,
        });
    }

    let query = lane.symbols.as_ref();
    let radius = lane.maximum_levenshtein_distance;
    let initial = initial_row(query.len(), radius);
    if verify_transitions {
        let generic = BandedLevenshteinRow::initial(query.len(), radius);
        if pack_row(&generic) != initial {
            return Err("typed V13 initial packed row differs from banded oracle".to_string());
        }
    }
    let mut stack = vec![PackedNode {
        state_id: index.root_state,
        rank_prefix: 0,
        row: initial,
    }];
    let mut form_refs = Vec::new();
    let mut expanded = 0_usize;
    let mut work = StructuralWork {
        stack_pushes: 1,
        ..StructuralWork::default()
    };
    let mut transition_checks = 0_usize;
    let mut terminal_distance_checks = 0_usize;
    let mut maximum_scratch = scratch_bytes(&stack, &form_refs);

    while let Some(node) = stack.pop() {
        expanded = expanded.saturating_add(1);
        work.expanded_states = work.expanded_states.saturating_add(1);
        work.stack_pops = work.stack_pops.saturating_add(1);
        if expanded > budget.maximum_product_states {
            return Ok(lane_unresolved(
                form_refs,
                expanded,
                maximum_scratch,
                "product_state_budget",
                work,
                transition_checks,
                terminal_distance_checks,
            ));
        }
        if budget.maximum_elapsed.is_some() && deadline_exceeded() {
            return Ok(lane_unresolved(
                form_refs,
                expanded,
                maximum_scratch,
                "wall_deadline",
                work,
                transition_checks,
                terminal_distance_checks,
            ));
        }

        let state = index.state(node.state_id)?;
        let distance = terminal_distance(node.row, query.len(), radius);
        if verify_transitions {
            let generic_distance = unpack_row(node.row).terminal_distance(query.len(), radius);
            if distance != generic_distance {
                return Err("typed V13 terminal distance differs from banded oracle".to_string());
            }
            terminal_distance_checks = terminal_distance_checks.saturating_add(1);
        }
        if state.terminal() && distance <= radius {
            work.terminal_hits = work.terminal_hits.saturating_add(1);
            terminal_ranks.push(node.rank_prefix);
            form_refs.push(node.rank_prefix);
            if form_refs.len() > budget.maximum_terminals {
                return Ok(lane_unresolved(
                    form_refs,
                    expanded,
                    maximum_scratch,
                    "terminal_budget",
                    work,
                    transition_checks,
                    terminal_distance_checks,
                ));
            }
        }

        if reverse_schedule {
            let mut children = Vec::new();
            for edge in index.edges(state)? {
                let rank_prefix = node
                    .rank_prefix
                    .checked_add(edge.rank_delta)
                    .ok_or_else(|| "typed V13 rank overflows u32".to_string())?;
                work.examined_edges = work.examined_edges.saturating_add(1);
                rank_prefixes.push(rank_prefix);
                let next = advance_for_symbol(node.row, masks, edge.symbol, radius, query.len())?;
                if verify_transitions {
                    verify_transition(node.row, query, edge.symbol, radius, next)?;
                    transition_checks = transition_checks.saturating_add(1);
                }
                if next.minimum <= radius {
                    work.surviving_edges = work.surviving_edges.saturating_add(1);
                    children.push(PackedNode {
                        state_id: edge.target,
                        rank_prefix,
                        row: next.state,
                    });
                } else {
                    work.pruned_edges = work.pruned_edges.saturating_add(1);
                }
            }
            work.stack_pushes = work.stack_pushes.saturating_add(children.len());
            stack.extend(children.into_iter().rev());
        } else {
            for edge in index.edges(state)? {
                let rank_prefix = node
                    .rank_prefix
                    .checked_add(edge.rank_delta)
                    .ok_or_else(|| "typed V13 rank overflows u32".to_string())?;
                work.examined_edges = work.examined_edges.saturating_add(1);
                rank_prefixes.push(rank_prefix);
                let next = advance_for_symbol(node.row, masks, edge.symbol, radius, query.len())?;
                if verify_transitions {
                    verify_transition(node.row, query, edge.symbol, radius, next)?;
                    transition_checks = transition_checks.saturating_add(1);
                }
                if next.minimum <= radius {
                    work.surviving_edges = work.surviving_edges.saturating_add(1);
                    work.stack_pushes = work.stack_pushes.saturating_add(1);
                    stack.push(PackedNode {
                        state_id: edge.target,
                        rank_prefix,
                        row: next.state,
                    });
                } else {
                    work.pruned_edges = work.pruned_edges.saturating_add(1);
                }
            }
        }

        maximum_scratch = maximum_scratch.max(scratch_bytes(&stack, &form_refs));
        if maximum_scratch > budget.maximum_scratch_bytes {
            return Ok(lane_unresolved(
                form_refs,
                expanded,
                maximum_scratch,
                "scratch_budget",
                work,
                transition_checks,
                terminal_distance_checks,
            ));
        }
    }

    Ok(LaneObservation {
        form_refs,
        expanded,
        maximum_scratch,
        unresolved: None,
        work,
        transition_checks,
        terminal_distance_checks,
    })
}

fn lane_unresolved(
    form_refs: Vec<u32>,
    expanded: usize,
    maximum_scratch: usize,
    reason: &'static str,
    work: StructuralWork,
    transition_checks: usize,
    terminal_distance_checks: usize,
) -> LaneObservation {
    LaneObservation {
        form_refs,
        expanded,
        maximum_scratch,
        unresolved: Some(reason),
        work,
        transition_checks,
        terminal_distance_checks,
    }
}

fn verify_transition(
    previous: u64,
    query: &[u32],
    symbol: u32,
    radius: u8,
    candidate: FusedOutput,
) -> Result<(), String> {
    let generic = unpack_row(previous).advance(query, symbol, radius);
    if pack_row(&generic) != candidate.state || generic.minimum(radius) != candidate.minimum {
        return Err(format!(
            "typed V13 packed transition differs from banded oracle for U+{symbol:04X}"
        ));
    }
    Ok(())
}

pub(super) fn alphabet_id(symbol: u32) -> Option<usize> {
    match symbol {
        0x002d => Some(0),
        0x0430..=0x044f => Some((symbol - 0x0430 + 1) as usize),
        0x0451 => Some(33),
        _ => None,
    }
}

pub(super) fn equality_masks(query: &[u32]) -> [[u64; 2]; ALPHABET_SYMBOLS] {
    let mut masks = [[0_u64; 2]; ALPHABET_SYMBOLS];
    for (position, symbol) in query.iter().copied().enumerate() {
        if let Some(alphabet_id) = alphabet_id(symbol) {
            masks[alphabet_id][position / 64] |= 1_u64 << (position % 64);
        }
    }
    masks
}

pub(super) fn initial_row(query_len: usize, radius: u8) -> u64 {
    let outside = radius.saturating_add(1);
    let len = query_len.min(radius as usize) + 1;
    let c0 = 0_u8;
    let c1 = if len > 1 { 1 } else { outside };
    let c2 = if len > 2 { 2 } else { outside };
    let c3 = if len > 3 { 3 } else { outside };
    u64::from(c0)
        | (u64::from(c1) << 3)
        | (u64::from(c2) << 6)
        | (u64::from(c3) << 9)
        | (u64::from(outside) << 12)
        | (u64::from(outside) << 15)
        | (u64::from(outside) << 18)
        | ((len as u64) << 37)
}

pub(super) fn advance_for_symbol(
    previous: u64,
    masks: &[[u64; 2]; ALPHABET_SYMBOLS],
    symbol: u32,
    radius: u8,
    query_len: usize,
) -> Result<FusedOutput, String> {
    let alphabet_id = alphabet_id(symbol)
        .ok_or_else(|| format!("typed V13 transition has unsupported U+{symbol:04X}"))?;
    let equality = equality_window(previous, masks[alphabet_id], radius, query_len)?;
    fused_advance(previous, equality, radius, query_len)
}

pub(super) fn terminal_distance(state: u64, query_len: usize, radius: u8) -> u8 {
    packed_value(state, query_len, radius.saturating_add(1))
}

fn pack_row(row: &BandedLevenshteinRow) -> u64 {
    let mut value = 0_u64;
    for (index, cell) in row.cells.iter().copied().enumerate() {
        value |= u64::from(cell) << (index * 3);
    }
    value | (u64::from(row.depth) << 21) | (u64::from(row.start) << 29) | (u64::from(row.len) << 37)
}

fn unpack_row(value: u64) -> BandedLevenshteinRow {
    let mut cells = [0_u8; MAX_BAND_CELLS];
    for (index, cell) in cells.iter_mut().enumerate() {
        *cell = ((value >> (index * 3)) & 7) as u8;
    }
    BandedLevenshteinRow {
        cells,
        depth: ((value >> 21) & 0xff) as u8,
        start: ((value >> 29) & 0xff) as u8,
        len: ((value >> 37) & 7) as u8,
    }
}

#[inline(always)]
fn packed_value(value: u64, column: usize, outside: u8) -> u8 {
    let start = ((value >> 29) & 0xff) as usize;
    let len = ((value >> 37) & 7) as usize;
    if column >= start && column < start + len {
        ((value >> ((column - start) * 3)) & 7) as u8
    } else {
        outside
    }
}

#[inline(always)]
fn extract_seven(mask: [u64; 2], position: usize) -> u8 {
    if position < 64 {
        let low = mask[0] >> position;
        let high = if position == 0 {
            0
        } else {
            mask[1] << (64 - position)
        };
        (low | high) as u8 & 0x7f
    } else if position < 128 {
        (mask[1] >> (position - 64)) as u8 & 0x7f
    } else {
        0
    }
}

#[inline(always)]
fn equality_window(
    previous: u64,
    mask: [u64; 2],
    radius: u8,
    query_len: usize,
) -> Result<u8, String> {
    let previous_depth = ((previous >> 21) & 0xff) as u8;
    let depth = previous_depth
        .checked_add(1)
        .ok_or_else(|| "typed V13 query depth overflows u8".to_string())?;
    let start = depth.saturating_sub(radius) as usize;
    let end = (depth as usize + radius as usize).min(query_len);
    let len = end.checked_sub(start).map_or(0, |width| width + 1);
    if len == 0 {
        return Ok(0);
    }
    let equality = if start == 0 {
        (extract_seven(mask, 0) << 1) & 0x7f
    } else {
        extract_seven(mask, start - 1)
    };
    Ok(equality & ((1_u16 << len) - 1) as u8)
}

#[inline(always)]
fn fused_cell<const INDEX: usize>(
    previous: u64,
    equality: u8,
    radius: u8,
    depth: u8,
    start: usize,
    left_value: u8,
) -> u8 {
    let outside = radius.saturating_add(1);
    let column = start + INDEX;
    if column == 0 {
        depth.min(outside)
    } else {
        let left = if INDEX == 0 { outside } else { left_value }
            .saturating_add(1)
            .min(outside);
        let above = packed_value(previous, column, outside)
            .saturating_add(1)
            .min(outside);
        let diagonal = packed_value(previous, column - 1, outside)
            .saturating_add(u8::from(equality & (1 << INDEX) == 0))
            .min(outside);
        left.min(above).min(diagonal)
    }
}

#[inline(always)]
fn fused_advance(
    previous: u64,
    equality: u8,
    radius: u8,
    query_len: usize,
) -> Result<FusedOutput, String> {
    let outside = radius.saturating_add(1);
    let previous_depth = ((previous >> 21) & 0xff) as u8;
    let depth = previous_depth
        .checked_add(1)
        .ok_or_else(|| "typed V13 query depth overflows u8".to_string())?;
    let start = depth.saturating_sub(radius) as usize;
    let end = (depth as usize + radius as usize).min(query_len);
    let len = end.checked_sub(start).map_or(0, |width| width + 1);
    if len == 0 {
        let state = u64::from(outside)
            | (u64::from(outside) << 3)
            | (u64::from(outside) << 6)
            | (u64::from(outside) << 9)
            | (u64::from(outside) << 12)
            | (u64::from(outside) << 15)
            | (u64::from(outside) << 18)
            | (u64::from(depth) << 21)
            | ((start as u64) << 29);
        return Ok(FusedOutput {
            state,
            minimum: outside,
        });
    }

    let c0 = fused_cell::<0>(previous, equality, radius, depth, start, outside);
    let c1 = if len > 1 {
        fused_cell::<1>(previous, equality, radius, depth, start, c0)
    } else {
        outside
    };
    let c2 = if len > 2 {
        fused_cell::<2>(previous, equality, radius, depth, start, c1)
    } else {
        outside
    };
    let c3 = if len > 3 {
        fused_cell::<3>(previous, equality, radius, depth, start, c2)
    } else {
        outside
    };
    let c4 = if len > 4 {
        fused_cell::<4>(previous, equality, radius, depth, start, c3)
    } else {
        outside
    };
    let c5 = if len > 5 {
        fused_cell::<5>(previous, equality, radius, depth, start, c4)
    } else {
        outside
    };
    let c6 = if len > 6 {
        fused_cell::<6>(previous, equality, radius, depth, start, c5)
    } else {
        outside
    };
    let cells = [c0, c1, c2, c3, c4, c5, c6];
    let minimum = cells[..len].iter().copied().min().unwrap_or(outside);
    let state = u64::from(c0)
        | (u64::from(c1) << 3)
        | (u64::from(c2) << 6)
        | (u64::from(c3) << 9)
        | (u64::from(c4) << 12)
        | (u64::from(c5) << 15)
        | (u64::from(c6) << 18)
        | (u64::from(depth) << 21)
        | ((start as u64) << 29)
        | ((len as u64) << 37);
    Ok(FusedOutput { state, minimum })
}

fn scratch_bytes(stack: &Vec<PackedNode>, terminals: &Vec<u32>) -> usize {
    stack
        .capacity()
        .saturating_mul(std::mem::size_of::<PackedNode>())
        .saturating_add(
            terminals
                .capacity()
                .saturating_mul(std::mem::size_of::<u32>()),
        )
}
