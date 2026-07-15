use std::collections::{BTreeMap, HashMap};

use super::format::{
    atom_center_keys, checksum, normalize_surface, put_u16, put_u32, put_u64, surface_phase,
    ArcRecord, CenterRecord, DecoderArcRecord, DecoderStateRecord, NodeRecord, TerminalRecord,
    ARC_BYTES, CENTER_BYTES, DECODER_ARC_BYTES, DECODER_STATE_BYTES, HEADER_BYTES, MAGIC,
    NODE_BYTES, NO_INDEX, PHASE_CELLS, POSTING_BYTES, TERMINAL_BYTES, VERSION,
};

const MAX_POSTINGS_PER_CENTER: usize = 512;

pub(crate) fn compile_words<'a, I>(words: I) -> Result<Vec<u8>, String>
where
    I: IntoIterator<Item = &'a str>,
{
    let words = words.into_iter().map(str::to_string).collect::<Vec<_>>();
    compile_words_with_training(
        words.iter().map(String::as_str),
        words.iter().map(String::as_str),
    )
}

pub(crate) fn compile_words_with_training<'a, 'b, I, J>(
    base_words: I,
    training_words: J,
) -> Result<Vec<u8>, String>
where
    I: IntoIterator<Item = &'a str>,
    J: IntoIterator<Item = &'b str>,
{
    let mut unique = Vec::<SourceWord>::new();
    let mut indexes = HashMap::<String, usize>::new();
    for raw in base_words {
        let Some(word) = normalize_surface(raw) else {
            continue;
        };
        if let Some(index) = indexes.get(&word).copied() {
            unique[index].support = unique[index].support.saturating_add(1);
            continue;
        }
        let index = unique.len();
        indexes.insert(word.clone(), index);
        unique.push(SourceWord { word, support: 1 });
    }
    if unique.is_empty() {
        return Err("lexical phase compiler received no valid words".to_string());
    }

    let mut nodes = vec![TempNode::root()];
    let mut terminals = Vec::<TerminalRecord>::with_capacity(unique.len());
    for (rank, source) in unique.iter().enumerate() {
        let node = insert_word(&mut nodes, &source.word)?;
        let terminal_id = terminals.len() as u32;
        if nodes[node as usize].terminal != NO_INDEX {
            return Err(format!(
                "duplicate terminal after normalization: {}",
                source.word
            ));
        }
        nodes[node as usize].terminal = terminal_id;
        let (phase, atom_count) = surface_phase(&source.word);
        terminals.push(TerminalRecord {
            node,
            rank: rank.min(u32::MAX as usize) as u32,
            support: source.support,
            char_len: source.word.chars().count().min(u16::MAX as usize) as u16,
            atom_count,
            phase,
        });
    }

    assign_best_terminals(&mut nodes, &terminals);
    let (node_records, arcs) = flatten_nodes(&nodes)?;
    let (centers, postings) = compile_centers(&unique);
    let training = normalized_training_surfaces(training_words);
    let (decoder_states, decoder_arcs) = compile_decoder(&training)?;
    let corpus_hash = corpus_hash(&unique, &training);
    serialize_artifact(ArtifactParts {
        nodes: &node_records,
        arcs: &arcs,
        terminals: &terminals,
        centers: &centers,
        postings: &postings,
        decoder_states: &decoder_states,
        decoder_arcs: &decoder_arcs,
        corpus_hash,
        source_words: unique.len(),
        training_surfaces: training.len(),
    })
}

#[derive(Clone, Debug)]
struct SourceWord {
    word: String,
    support: u32,
}

#[derive(Clone, Debug, Default)]
struct DecoderMutableState {
    final_state: bool,
    arcs: BTreeMap<u32, u32>,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
struct DecoderStateSignature {
    final_state: bool,
    arcs: Vec<(u32, u32)>,
}

fn normalized_training_surfaces<'a, I>(words: I) -> Vec<String>
where
    I: IntoIterator<Item = &'a str>,
{
    let mut surfaces = words
        .into_iter()
        .filter_map(normalize_surface)
        .collect::<Vec<_>>();
    surfaces.sort_unstable();
    surfaces.dedup();
    surfaces
}

fn compile_decoder(
    surfaces: &[String],
) -> Result<(Vec<DecoderStateRecord>, Vec<DecoderArcRecord>), String> {
    let mut path = vec![DecoderMutableState::default()];
    let mut previous = Vec::<u32>::new();
    let mut registry = HashMap::<DecoderStateSignature, u32>::new();
    let mut registered = Vec::<DecoderStateSignature>::new();

    for surface in surfaces {
        let current = surface.chars().map(|ch| ch as u32).collect::<Vec<_>>();
        let common = previous
            .iter()
            .zip(&current)
            .take_while(|(left, right)| left == right)
            .count();
        minimize_decoder_suffix(&mut path, &previous, common, &mut registry, &mut registered)?;
        for _ in common..current.len() {
            path.push(DecoderMutableState::default());
        }
        path.last_mut()
            .expect("decoder path always contains root")
            .final_state = true;
        previous = current;
    }

    minimize_decoder_suffix(&mut path, &previous, 0, &mut registry, &mut registered)?;
    let root = intern_decoder_state(
        path.pop().expect("decoder root remains after minimization"),
        &mut registry,
        &mut registered,
    )?;
    flatten_decoder(root, &registered)
}

fn minimize_decoder_suffix(
    path: &mut Vec<DecoderMutableState>,
    previous: &[u32],
    common: usize,
    registry: &mut HashMap<DecoderStateSignature, u32>,
    registered: &mut Vec<DecoderStateSignature>,
) -> Result<(), String> {
    while path.len().saturating_sub(1) > common {
        let depth = path.len() - 1;
        let child = path.pop().expect("decoder suffix state exists");
        let child_id = intern_decoder_state(child, registry, registered)?;
        let ch = *previous
            .get(depth - 1)
            .ok_or_else(|| "decoder path/surface depth mismatch".to_string())?;
        path.last_mut()
            .expect("decoder parent state exists")
            .arcs
            .insert(ch, child_id);
    }
    Ok(())
}

fn intern_decoder_state(
    state: DecoderMutableState,
    registry: &mut HashMap<DecoderStateSignature, u32>,
    registered: &mut Vec<DecoderStateSignature>,
) -> Result<u32, String> {
    let signature = DecoderStateSignature {
        final_state: state.final_state,
        arcs: state.arcs.into_iter().collect(),
    };
    if let Some(existing) = registry.get(&signature) {
        return Ok(*existing);
    }
    let id = u32::try_from(registered.len())
        .map_err(|_| "lexical decoder state table exceeds u32".to_string())?;
    registered.push(signature.clone());
    registry.insert(signature, id);
    Ok(id)
}

fn flatten_decoder(
    root: u32,
    registered: &[DecoderStateSignature],
) -> Result<(Vec<DecoderStateRecord>, Vec<DecoderArcRecord>), String> {
    let mut old_to_new = HashMap::<u32, u32>::new();
    let mut order = Vec::<u32>::new();
    let mut stack = vec![root];
    while let Some(old) = stack.pop() {
        if old_to_new.contains_key(&old) {
            continue;
        }
        let new = u32::try_from(order.len())
            .map_err(|_| "lexical decoder state table exceeds u32".to_string())?;
        old_to_new.insert(old, new);
        order.push(old);
        let state = registered
            .get(old as usize)
            .ok_or_else(|| "lexical decoder state reference is invalid".to_string())?;
        for (_, child) in state.arcs.iter().rev() {
            stack.push(*child);
        }
    }

    let mut states = Vec::with_capacity(order.len());
    let mut arcs = Vec::new();
    for old in order {
        let state = &registered[old as usize];
        if arcs.len() > u32::MAX as usize || state.arcs.len() > u16::MAX as usize {
            return Err("lexical decoder table exceeds artifact format".to_string());
        }
        let first_arc = arcs.len() as u32;
        for (ch, child) in &state.arcs {
            arcs.push(DecoderArcRecord {
                ch: *ch,
                child: *old_to_new
                    .get(child)
                    .ok_or_else(|| "lexical decoder child was not remapped".to_string())?,
            });
        }
        states.push(DecoderStateRecord {
            first_arc,
            arc_len: (arcs.len() as u32 - first_arc) as u16,
            flags: u16::from(state.final_state),
        });
    }
    Ok((states, arcs))
}

#[derive(Clone, Debug)]
struct TempNode {
    parent: u32,
    incoming: char,
    depth: u16,
    terminal: u32,
    best_terminal: u32,
    children: BTreeMap<char, u32>,
}

impl TempNode {
    fn root() -> Self {
        Self {
            parent: 0,
            incoming: '\0',
            depth: 0,
            terminal: NO_INDEX,
            best_terminal: NO_INDEX,
            children: BTreeMap::new(),
        }
    }
}

fn insert_word(nodes: &mut Vec<TempNode>, word: &str) -> Result<u32, String> {
    let mut node_id = 0u32;
    for ch in word.chars() {
        let child = nodes[node_id as usize].children.get(&ch).copied();
        node_id = if let Some(child) = child {
            child
        } else {
            let next = nodes.len();
            if next > u32::MAX as usize {
                return Err("lexical grapheme graph exceeds u32".to_string());
            }
            let child = next as u32;
            let depth = nodes[node_id as usize].depth.saturating_add(1);
            nodes.push(TempNode {
                parent: node_id,
                incoming: ch,
                depth,
                terminal: NO_INDEX,
                best_terminal: NO_INDEX,
                children: BTreeMap::new(),
            });
            nodes[node_id as usize].children.insert(ch, child);
            child
        };
    }
    Ok(node_id)
}

fn assign_best_terminals(nodes: &mut [TempNode], terminals: &[TerminalRecord]) {
    for node_index in (0..nodes.len()).rev() {
        let mut best = nodes[node_index].terminal;
        for child in nodes[node_index].children.values().copied() {
            let child_best = nodes[child as usize].best_terminal;
            best = lower_rank_terminal(best, child_best, terminals);
        }
        nodes[node_index].best_terminal = best;
    }
}

fn lower_rank_terminal(left: u32, right: u32, terminals: &[TerminalRecord]) -> u32 {
    match (left, right) {
        (NO_INDEX, other) | (other, NO_INDEX) => other,
        (left, right) => {
            let left_rank = terminals
                .get(left as usize)
                .map(|terminal| terminal.rank)
                .unwrap_or(u32::MAX);
            let right_rank = terminals
                .get(right as usize)
                .map(|terminal| terminal.rank)
                .unwrap_or(u32::MAX);
            if left_rank <= right_rank {
                left
            } else {
                right
            }
        }
    }
}

fn flatten_nodes(nodes: &[TempNode]) -> Result<(Vec<NodeRecord>, Vec<ArcRecord>), String> {
    let mut records = Vec::with_capacity(nodes.len());
    let mut arcs = Vec::new();
    for node in nodes {
        let first_arc = arcs.len();
        for (ch, child) in &node.children {
            arcs.push(ArcRecord {
                ch: *ch as u32,
                child: *child,
            });
        }
        if first_arc > u32::MAX as usize || node.children.len() > u16::MAX as usize {
            return Err("lexical grapheme arc table exceeds format".to_string());
        }
        records.push(NodeRecord {
            parent: node.parent,
            incoming: node.incoming as u32,
            first_arc: first_arc as u32,
            arc_len: node.children.len() as u16,
            depth: node.depth,
            terminal: node.terminal,
            best_terminal: node.best_terminal,
        });
    }
    Ok((records, arcs))
}

fn compile_centers(words: &[SourceWord]) -> (Vec<CenterRecord>, Vec<u32>) {
    let mut center_postings = BTreeMap::<u64, Vec<u32>>::new();
    for (terminal, source) in words.iter().enumerate() {
        for key in atom_center_keys(&source.word) {
            center_postings
                .entry(key)
                .or_default()
                .push(terminal as u32);
        }
    }
    let mut centers = Vec::with_capacity(center_postings.len());
    let mut postings = Vec::new();
    for (key, mut terminal_ids) in center_postings {
        terminal_ids.sort_unstable();
        terminal_ids.dedup();
        let support = terminal_ids.len();
        terminal_ids.truncate(MAX_POSTINGS_PER_CENTER);
        let start = postings.len();
        postings.extend(terminal_ids.iter().copied());
        centers.push(CenterRecord {
            key,
            posting_start: start.min(u32::MAX as usize) as u32,
            posting_len: terminal_ids.len().min(u16::MAX as usize) as u16,
            support: support.min(u16::MAX as usize) as u16,
        });
    }
    (centers, postings)
}

fn corpus_hash(words: &[SourceWord], training: &[String]) -> u64 {
    let mut bytes = Vec::new();
    for source in words {
        bytes.extend_from_slice(source.word.as_bytes());
        bytes.push(0xff);
        bytes.extend_from_slice(&source.support.to_le_bytes());
    }
    bytes.extend_from_slice(b"\0decoder\0");
    for surface in training {
        bytes.extend_from_slice(surface.as_bytes());
        bytes.push(0xfe);
    }
    checksum(&bytes)
}

struct ArtifactParts<'a> {
    nodes: &'a [NodeRecord],
    arcs: &'a [ArcRecord],
    terminals: &'a [TerminalRecord],
    centers: &'a [CenterRecord],
    postings: &'a [u32],
    decoder_states: &'a [DecoderStateRecord],
    decoder_arcs: &'a [DecoderArcRecord],
    corpus_hash: u64,
    source_words: usize,
    training_surfaces: usize,
}

fn serialize_artifact(parts: ArtifactParts<'_>) -> Result<Vec<u8>, String> {
    let ArtifactParts {
        nodes,
        arcs,
        terminals,
        centers,
        postings,
        decoder_states,
        decoder_arcs,
        corpus_hash,
        source_words,
        training_surfaces,
    } = parts;
    let nodes_offset = HEADER_BYTES;
    let arcs_offset = nodes_offset + nodes.len() * NODE_BYTES;
    let terminals_offset = arcs_offset + arcs.len() * ARC_BYTES;
    let centers_offset = terminals_offset + terminals.len() * TERMINAL_BYTES;
    let postings_offset = centers_offset + centers.len() * CENTER_BYTES;
    let decoder_states_offset = postings_offset + postings.len() * POSTING_BYTES;
    let decoder_arcs_offset = decoder_states_offset + decoder_states.len() * DECODER_STATE_BYTES;
    let file_bytes = decoder_arcs_offset + decoder_arcs.len() * DECODER_ARC_BYTES;
    if [
        nodes.len(),
        arcs.len(),
        terminals.len(),
        centers.len(),
        postings.len(),
        decoder_states.len(),
        decoder_arcs.len(),
        source_words,
        training_surfaces,
    ]
    .iter()
    .any(|value| *value > u32::MAX as usize)
    {
        return Err("lexical phase artifact exceeds u32 format".to_string());
    }

    let mut bytes = vec![0u8; file_bytes];
    bytes[..8].copy_from_slice(MAGIC);
    put_u32(&mut bytes, 8, VERSION);
    put_u32(&mut bytes, 12, HEADER_BYTES as u32);
    put_u64(&mut bytes, 16, file_bytes as u64);
    put_u32(&mut bytes, 24, nodes.len() as u32);
    put_u32(&mut bytes, 28, arcs.len() as u32);
    put_u32(&mut bytes, 32, terminals.len() as u32);
    put_u32(&mut bytes, 36, centers.len() as u32);
    put_u32(&mut bytes, 40, postings.len() as u32);
    put_u16(&mut bytes, 44, PHASE_CELLS as u16);
    put_u64(&mut bytes, 48, nodes_offset as u64);
    put_u64(&mut bytes, 56, arcs_offset as u64);
    put_u64(&mut bytes, 64, terminals_offset as u64);
    put_u64(&mut bytes, 72, centers_offset as u64);
    put_u64(&mut bytes, 80, postings_offset as u64);
    put_u64(&mut bytes, 96, corpus_hash);
    put_u32(&mut bytes, 104, source_words as u32);
    put_u32(&mut bytes, 108, decoder_states.len() as u32);
    put_u32(&mut bytes, 112, decoder_arcs.len() as u32);
    put_u64(&mut bytes, 116, decoder_states_offset as u64);
    put_u32(&mut bytes, 124, training_surfaces as u32);

    for (index, node) in nodes.iter().enumerate() {
        let offset = nodes_offset + index * NODE_BYTES;
        put_u32(&mut bytes, offset, node.parent);
        put_u32(&mut bytes, offset + 4, node.incoming);
        put_u32(&mut bytes, offset + 8, node.first_arc);
        put_u16(&mut bytes, offset + 12, node.arc_len);
        put_u16(&mut bytes, offset + 14, node.depth);
        put_u32(&mut bytes, offset + 16, node.terminal);
        put_u32(&mut bytes, offset + 20, node.best_terminal);
    }
    for (index, arc) in arcs.iter().enumerate() {
        let offset = arcs_offset + index * ARC_BYTES;
        put_u32(&mut bytes, offset, arc.ch);
        put_u32(&mut bytes, offset + 4, arc.child);
    }
    for (index, terminal) in terminals.iter().enumerate() {
        let offset = terminals_offset + index * TERMINAL_BYTES;
        put_u32(&mut bytes, offset, terminal.node);
        put_u32(&mut bytes, offset + 4, terminal.rank);
        put_u32(&mut bytes, offset + 8, terminal.support);
        put_u16(&mut bytes, offset + 12, terminal.char_len);
        put_u16(&mut bytes, offset + 14, terminal.atom_count);
        for (target, value) in bytes[offset + 16..offset + TERMINAL_BYTES]
            .iter_mut()
            .zip(terminal.phase)
        {
            *target = value as u8;
        }
    }
    for (index, center) in centers.iter().enumerate() {
        let offset = centers_offset + index * CENTER_BYTES;
        put_u64(&mut bytes, offset, center.key);
        put_u32(&mut bytes, offset + 8, center.posting_start);
        put_u16(&mut bytes, offset + 12, center.posting_len);
        put_u16(&mut bytes, offset + 14, center.support);
    }
    for (index, posting) in postings.iter().enumerate() {
        put_u32(
            &mut bytes,
            postings_offset + index * POSTING_BYTES,
            *posting,
        );
    }
    for (index, state) in decoder_states.iter().enumerate() {
        let offset = decoder_states_offset + index * DECODER_STATE_BYTES;
        put_u32(&mut bytes, offset, state.first_arc);
        put_u16(&mut bytes, offset + 4, state.arc_len);
        put_u16(&mut bytes, offset + 6, state.flags);
    }
    for (index, arc) in decoder_arcs.iter().enumerate() {
        let offset = decoder_arcs_offset + index * DECODER_ARC_BYTES;
        put_u32(&mut bytes, offset, arc.ch);
        put_u32(&mut bytes, offset + 4, arc.child);
    }
    let artifact_checksum = checksum(&bytes[HEADER_BYTES..]);
    put_u64(&mut bytes, 88, artifact_checksum);
    Ok(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compiler_emits_valid_stringless_artifact() {
        let bytes = compile_words(["привет", "проверка", "проверить", "проверка"])
            .expect("artifact compiles");
        let header = super::super::format::read_header(&bytes).expect("artifact validates");

        assert_eq!(header.source_words, 3);
        assert_eq!(header.terminal_count, 3);
        assert!(header.center_count > 0);
        assert!(header.node_count > 3);
    }

    #[test]
    fn compiler_is_bit_reproducible_for_same_surfaces() {
        let first = compile_words_with_training(
            ["проверка", "загрузить", "слово"],
            ["проверка", "проверить", "загрузить", "загрузи", "слово"],
        )
        .expect("first artifact compiles");
        let second = compile_words_with_training(
            ["проверка", "загрузить", "слово"],
            ["слово", "загрузи", "загрузить", "проверить", "проверка"],
        )
        .expect("second artifact compiles");

        assert_eq!(first, second);
    }
}
