
    const E1_PARITY_TEST: &str =
        "nanda_wave::l2_field::v13_typed_peak::tests::v10_e1_semantic_parity";
    const E1_SINGLE_TEST: &str =
        "nanda_wave::l2_field::v13_typed_peak::tests::v10_e1_single_latency";
    const E1_TWENTY_TEST: &str =
        "nanda_wave::l2_field::v13_typed_peak::tests::v10_e1_twenty_worker_latency";
    const E1_E0_PHYSICAL_TEST: &str =
        "nanda_wave::l2_field::v13_typed_peak::tests::v10_e1_e0_physical";
    const E1_E1_PHYSICAL_TEST: &str =
        "nanda_wave::l2_field::v13_typed_peak::tests::v10_e1_e1_physical";
    const E1_V7_SHA256: &str = "33fded73e13f565bdc08a83bd440be67c80f6d85a196453050b09bc9f7ef28e4";
    const E1_SIDECAR_SHA256: &str =
        "a1aa95be8c43fae8fedf02d3261dd1265dcbfbc8d618330fcd6449ae829df8cd";
    const E1_SCHEDULE_SHA256: &str =
        "2f8346d97246ac3704434279739fa5ed27705465d628890a367c21875d879e78";
    const E1_SCHEDULE_SCHEMA: &str = "lay.v10.hardware-query-schedule.v1";
    const E1_SAMPLE_SCHEMA: &str = "lay.v10.e1-primitive-samples.v1";
    const E1_SAMPLE_BYTES: usize = 22;
    const E1_SINGLE_ROUNDS: usize = 100;
    const E1_TWENTY_ROUNDS: usize = 250;
    const E1_WORKERS: usize = 20;
    const E1_SINGLE_WORKER_SENTINEL: u8 = u8::MAX;
    const E1_FLAG_ERROR: u8 = 1;
    const E1_FLAG_UNRESOLVED: u8 = 2;
    const E1_ALPHABET_SYMBOLS: usize = 34;
    const E1_EXPECTED_TRANSITIONS: usize = 25_145_756;
    const E1_EXPECTED_EXPANDED: usize = 8_059_788;

    #[derive(Clone)]
    struct E1ScheduleEntry {
        source_ordinal: usize,
        composite_identity_sha256: String,
        damaged_surface_sha256: String,
        retrieval_lanes_sha256: String,
        worker: usize,
        chunk_start: usize,
        chunk_end: usize,
    }

    struct E1Inputs {
        package: RuntimeL2Package,
        index: V13DafsaView,
        cases: Vec<ProofCase>,
        schedule: Vec<E1ScheduleEntry>,
    }

    #[derive(Clone, Copy)]
    struct E1Sample {
        query_ordinal: u16,
        round: u16,
        worker_id: u8,
        flags: u8,
        search_elapsed_us: u64,
        total_elapsed_us: u64,
    }

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    struct E1FusedOutput {
        state: u64,
        minimum: u8,
    }

    #[derive(Clone, Copy)]
    struct E1PackedNode {
        state_id: u32,
        rank_prefix: u32,
        row: u64,
    }

    #[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
    struct E1Work {
        expanded_states: usize,
        examined_edges: usize,
        surviving_edges: usize,
        pruned_edges: usize,
        stack_pushes: usize,
        stack_pops: usize,
        terminal_hits: usize,
    }

    impl E1Work {
        fn add(&mut self, other: Self) {
            self.expanded_states = self.expanded_states.saturating_add(other.expanded_states);
            self.examined_edges = self.examined_edges.saturating_add(other.examined_edges);
            self.surviving_edges = self.surviving_edges.saturating_add(other.surviving_edges);
            self.pruned_edges = self.pruned_edges.saturating_add(other.pruned_edges);
            self.stack_pushes = self.stack_pushes.saturating_add(other.stack_pushes);
            self.stack_pops = self.stack_pops.saturating_add(other.stack_pops);
            self.terminal_hits = self.terminal_hits.saturating_add(other.terminal_hits);
        }
    }

    struct E1Observed {
        result: SearchResult,
        work: E1Work,
        rank_prefixes: Vec<u32>,
        terminal_ranks: Vec<u32>,
    }

    fn e1_required_path(name: &str) -> Result<std::path::PathBuf, String> {
        std::env::var_os(name)
            .map(std::path::PathBuf::from)
            .ok_or_else(|| format!("required environment path {name} is absent"))
    }

    fn e1_required_string(name: &str) -> Result<String, String> {
        std::env::var(name).map_err(|error| format!("required environment {name}: {error}"))
    }

    fn e1_write_new(path: &Path, bytes: &[u8]) -> Result<(), String> {
        use std::io::Write;
        let mut output = std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(path)
            .map_err(|error| format!("{}: {error}", path.display()))?;
        output
            .write_all(bytes)
            .map_err(|error| format!("{}: {error}", path.display()))?;
        output
            .sync_all()
            .map_err(|error| format!("{}: {error}", path.display()))
    }

    fn e1_write_json(path: &Path, value: &serde_json::Value) -> Result<(), String> {
        let mut bytes = serde_json::to_vec_pretty(value).map_err(|error| error.to_string())?;
        bytes.push(b'\n');
        e1_write_new(path, &bytes)
    }

    fn e1_sha256_text(value: &[u8]) -> String {
        hex(Sha256::digest(value).into())
    }

    fn e1_compact_lanes(lanes: &[Phase7dRetrievalLane]) -> Result<Vec<u8>, String> {
        let values = lanes
            .iter()
            .map(|lane| {
                serde_json::json!({
                    "symbols": lane.symbols,
                    "maximum_levenshtein_distance": lane.maximum_levenshtein_distance,
                })
            })
            .collect::<Vec<_>>();
        serde_json::to_vec(&values).map_err(|error| error.to_string())
    }

    fn e1_schedule_entries(value: &serde_json::Value) -> Result<Vec<E1ScheduleEntry>, String> {
        if value.get("schema").and_then(serde_json::Value::as_str) != Some(E1_SCHEDULE_SCHEMA) {
            return Err("E1 schedule schema mismatch".to_string());
        }
        let partition = value
            .get("partition")
            .and_then(serde_json::Value::as_object)
            .ok_or_else(|| "E1 schedule partition is absent".to_string())?;
        if partition
            .get("b5_requests")
            .and_then(serde_json::Value::as_u64)
            != Some(382)
            || partition
                .get("b6_workers")
                .and_then(serde_json::Value::as_u64)
                != Some(20)
            || partition
                .get("b6_chunk_size")
                .and_then(serde_json::Value::as_u64)
                != Some(20)
        {
            return Err("E1 schedule partition mismatch".to_string());
        }
        value
            .get("entries")
            .and_then(serde_json::Value::as_array)
            .ok_or_else(|| "E1 schedule entries are absent".to_string())?
            .iter()
            .map(|entry| {
                let number = |name: &str| {
                    entry
                        .get(name)
                        .and_then(serde_json::Value::as_u64)
                        .and_then(|value| usize::try_from(value).ok())
                        .ok_or_else(|| format!("E1 schedule entry lacks {name}"))
                };
                let string = |name: &str| {
                    entry
                        .get(name)
                        .and_then(serde_json::Value::as_str)
                        .map(str::to_string)
                        .ok_or_else(|| format!("E1 schedule entry lacks {name}"))
                };
                Ok(E1ScheduleEntry {
                    source_ordinal: number("source_ordinal")?,
                    composite_identity_sha256: string("composite_identity_sha256")?,
                    damaged_surface_sha256: string("damaged_surface_sha256")?,
                    retrieval_lanes_sha256: string("retrieval_lanes_sha256")?,
                    worker: number("b6_worker")?,
                    chunk_start: number("b6_chunk_start")?,
                    chunk_end: number("b6_chunk_end_exclusive")?,
                })
            })
            .collect()
    }

    fn e1_validate_schedule(
        cases: &[ProofCase],
        schedule: &[E1ScheduleEntry],
    ) -> Result<(), String> {
        if cases.len() != 382 || schedule.len() != cases.len() {
            return Err("E1 schedule denominator mismatch".to_string());
        }
        for (ordinal, (case, entry)) in cases.iter().zip(schedule).enumerate() {
            if entry.source_ordinal != ordinal {
                return Err(format!("E1 source ordinal mismatch at {ordinal}"));
            }
            let composite = [case.class.as_bytes(), &[0], case.proof_identity.as_bytes()].concat();
            if e1_sha256_text(&composite) != entry.composite_identity_sha256 {
                return Err(format!("E1 composite identity mismatch at {ordinal}"));
            }
            if e1_sha256_text(case.damaged_surface.as_bytes()) != entry.damaged_surface_sha256 {
                return Err(format!("E1 damaged surface mismatch at {ordinal}"));
            }
            let lanes = Phase7dCertificateOracle::new(&case.damaged_surface)?.retrieval_lanes();
            if e1_sha256_text(&e1_compact_lanes(&lanes)?) != entry.retrieval_lanes_sha256 {
                return Err(format!("E1 retrieval lanes mismatch at {ordinal}"));
            }
            let worker = ordinal / 20;
            let start = worker * 20;
            let end = (start + 20).min(cases.len());
            if entry.worker != worker || entry.chunk_start != start || entry.chunk_end != end {
                return Err(format!("E1 worker partition mismatch at {ordinal}"));
            }
        }
        Ok(())
    }

    fn e1_load_inputs() -> Result<E1Inputs, String> {
        let package_path = e1_required_path("LAY_V10_E1_PACKAGE")?;
        let sidecar_path = e1_required_path("LAY_V10_E1_SIDECAR")?;
        let v7_path = e1_required_path("LAY_V10_E1_V7")?;
        let schedule_path = e1_required_path("LAY_V10_E1_SCHEDULE")?;

        let package_bytes = std::fs::metadata(&package_path)
            .map_err(|error| format!("{}: {error}", package_path.display()))?
            .len();
        let package_sha256 = sha256_file(&package_path)?;
        if hex(package_sha256) != EXPECTED_V13_SHA256 || package_bytes != EXPECTED_V13_BYTES {
            return Err("E1 V13 package identity mismatch".to_string());
        }
        if hex(sha256_file(&sidecar_path)?) != E1_SIDECAR_SHA256 {
            return Err("E1 sidecar identity mismatch".to_string());
        }
        if hex(sha256_file(&v7_path)?) != E1_V7_SHA256 {
            return Err("E1 V7 identity mismatch".to_string());
        }
        if hex(sha256_file(&schedule_path)?) != E1_SCHEDULE_SHA256 {
            return Err("E1 schedule identity mismatch".to_string());
        }

        let package = RuntimeL2Package::load(&package_path)?;
        if package.form_count() != EXPECTED_V13_FORMS {
            return Err("E1 V13 form count mismatch".to_string());
        }
        let identity = V13Identity {
            package_sha256,
            package_bytes,
            form_count: u32::try_from(package.form_count())
                .map_err(|_| "E1 form count exceeds u32".to_string())?,
            binding_count: u32::try_from(package.binding_count())
                .map_err(|_| "E1 binding count exceeds u32".to_string())?,
        };
        let index = V13DafsaView::load(&sidecar_path, identity)?;
        e1_validate_dense_alphabet(&index)?;
        let v7: serde_json::Value = serde_json::from_slice(
            &std::fs::read(&v7_path).map_err(|error| format!("{}: {error}", v7_path.display()))?,
        )
        .map_err(|error| error.to_string())?;
        let cases = parse_v7_cases(&v7)?;
        let schedule_value: serde_json::Value = serde_json::from_slice(
            &std::fs::read(&schedule_path)
                .map_err(|error| format!("{}: {error}", schedule_path.display()))?,
        )
        .map_err(|error| error.to_string())?;
        let schedule = e1_schedule_entries(&schedule_value)?;
        e1_validate_schedule(&cases, &schedule)?;
        Ok(E1Inputs {
            package,
            index,
            cases,
            schedule,
        })
    }

    #[inline(always)]
    fn e1_alphabet_id(symbol: u32) -> Option<usize> {
        match symbol {
            0x002d => Some(0),
            0x0430..=0x044f => Some((symbol - 0x0430 + 1) as usize),
            0x0451 => Some(33),
            _ => None,
        }
    }

    fn e1_validate_dense_alphabet(index: &V13DafsaView) -> Result<(), String> {
        let mut seen = [false; E1_ALPHABET_SYMBOLS];
        for edge_id in 0..index.edge_count as usize {
            let edge = index.edge(edge_id)?;
            let alphabet_id = e1_alphabet_id(edge.symbol).ok_or_else(|| {
                format!("E1 sidecar edge has unknown symbol U+{:04X}", edge.symbol)
            })?;
            seen[alphabet_id] = true;
        }
        if seen.iter().any(|present| !present) {
            return Err("E1 sidecar does not contain the complete dense alphabet".to_string());
        }
        Ok(())
    }

    #[inline(always)]
    fn e1_pack(row: &BandedLevenshteinRow) -> u64 {
        let mut value = 0_u64;
        for (index, cell) in row.cells.iter().copied().enumerate() {
            value |= u64::from(cell) << (index * 3);
        }
        value
            | (u64::from(row.depth) << 21)
            | (u64::from(row.start) << 29)
            | (u64::from(row.len) << 37)
    }

    fn e1_unpack(value: u64) -> BandedLevenshteinRow {
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
    fn e1_packed_value(value: u64, column: usize, outside: u8) -> u8 {
        let start = ((value >> 29) & 0xff) as usize;
        let len = ((value >> 37) & 7) as usize;
        if column >= start && column < start + len {
            ((value >> ((column - start) * 3)) & 7) as u8
        } else {
            outside
        }
    }

    #[inline(always)]
    fn e1_initial(query_len: usize, radius: u8) -> u64 {
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

    fn e1_eq_masks(query: &[u32]) -> [[u64; 2]; E1_ALPHABET_SYMBOLS] {
        let mut masks = [[0_u64; 2]; E1_ALPHABET_SYMBOLS];
        for (position, symbol) in query.iter().copied().enumerate() {
            if let Some(alphabet_id) = e1_alphabet_id(symbol) {
                masks[alphabet_id][position / 64] |= 1_u64 << (position % 64);
            }
        }
        masks
    }

    #[inline(always)]
    fn e1_extract_seven(mask: [u64; 2], position: usize) -> u8 {
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
    fn e1_equality_window(previous: u64, mask: [u64; 2], radius: u8, query_len: usize) -> u8 {
        let previous_depth = ((previous >> 21) & 0xff) as u8;
        let depth = previous_depth.checked_add(1).expect("bounded query depth");
        let start = depth.saturating_sub(radius) as usize;
        let end = (depth as usize + radius as usize).min(query_len);
        let len = end.checked_sub(start).map_or(0, |width| width + 1);
        if len == 0 {
            return 0;
        }
        let equality = if start == 0 {
            (e1_extract_seven(mask, 0) << 1) & 0x7f
        } else {
            e1_extract_seven(mask, start - 1)
        };
        equality & ((1_u16 << len) - 1) as u8
    }

    #[inline(always)]
    fn e1_u1_cell<const INDEX: usize>(
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
            let above = e1_packed_value(previous, column, outside)
                .saturating_add(1)
                .min(outside);
            let diagonal = e1_packed_value(previous, column - 1, outside)
                .saturating_add(u8::from(equality & (1 << INDEX) == 0))
                .min(outside);
            left.min(above).min(diagonal)
        }
    }

    #[inline(always)]
    fn e1_u1_advance(previous: u64, equality: u8, radius: u8, query_len: usize) -> E1FusedOutput {
        let outside = radius.saturating_add(1);
        let previous_depth = ((previous >> 21) & 0xff) as u8;
        let depth = previous_depth.checked_add(1).expect("bounded query depth");
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
            return E1FusedOutput {
                state,
                minimum: outside,
            };
        }
        let c0 = e1_u1_cell::<0>(previous, equality, radius, depth, start, outside);
        let c1 = if len > 1 {
            e1_u1_cell::<1>(previous, equality, radius, depth, start, c0)
        } else {
            outside
        };
        let c2 = if len > 2 {
            e1_u1_cell::<2>(previous, equality, radius, depth, start, c1)
        } else {
            outside
        };
        let c3 = if len > 3 {
            e1_u1_cell::<3>(previous, equality, radius, depth, start, c2)
        } else {
            outside
        };
        let c4 = if len > 4 {
            e1_u1_cell::<4>(previous, equality, radius, depth, start, c3)
        } else {
            outside
        };
        let c5 = if len > 5 {
            e1_u1_cell::<5>(previous, equality, radius, depth, start, c4)
        } else {
            outside
        };
        let c6 = if len > 6 {
            e1_u1_cell::<6>(previous, equality, radius, depth, start, c5)
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
        E1FusedOutput { state, minimum }
    }

    #[inline(always)]
    fn e1_terminal_distance(state: u64, query_len: usize, radius: u8) -> u8 {
        e1_packed_value(state, query_len, radius.saturating_add(1))
    }

    fn e1_scratch_bytes(stack: &Vec<E1PackedNode>, terminals: &Vec<u32>) -> usize {
        stack.capacity() * std::mem::size_of::<E1PackedNode>()
            + terminals.capacity() * std::mem::size_of::<u32>()
    }

    fn e1_enumerate_lane<const TRACE: bool>(
        index: &V13DafsaView,
        lane: &Phase7dRetrievalLane,
        budget: SearchBudget,
        started: Instant,
        reverse_schedule: bool,
        rank_prefixes: &mut Vec<u32>,
        terminal_ranks: &mut Vec<u32>,
    ) -> Result<(LaneOutcome, E1Work), String> {
        let query = lane.symbols.as_ref();
        if query.len() > MAX_QUERY_SYMBOLS {
            return Ok((
                LaneOutcome {
                    form_refs: Vec::new(),
                    expanded: 0,
                    maximum_scratch: 0,
                    unresolved: Some("query_symbol_budget"),
                },
                E1Work::default(),
            ));
        }
        let radius = lane.maximum_levenshtein_distance;
        if radius > MAX_LEVENSHTEIN_RADIUS {
            return Ok((
                LaneOutcome {
                    form_refs: Vec::new(),
                    expanded: 0,
                    maximum_scratch: 0,
                    unresolved: Some("lane_radius_budget"),
                },
                E1Work::default(),
            ));
        }
        let masks = e1_eq_masks(query);
        let mut stack = vec![E1PackedNode {
            state_id: index.root_state,
            rank_prefix: 0,
            row: e1_initial(query.len(), radius),
        }];
        let mut form_refs = Vec::new();
        let mut expanded = 0_usize;
        let mut work = E1Work::default();
        if TRACE {
            work.stack_pushes = 1;
        }
        let mut maximum_scratch = e1_scratch_bytes(&stack, &form_refs);
        while let Some(node) = stack.pop() {
            expanded += 1;
            if TRACE {
                work.expanded_states += 1;
                work.stack_pops += 1;
            }
            if expanded > budget.maximum_product_states {
                return Ok((
                    LaneOutcome {
                        form_refs,
                        expanded,
                        maximum_scratch,
                        unresolved: Some("product_state_budget"),
                    },
                    work,
                ));
            }
            if budget
                .maximum_elapsed
                .is_some_and(|maximum| started.elapsed() > maximum)
            {
                return Ok((
                    LaneOutcome {
                        form_refs,
                        expanded,
                        maximum_scratch,
                        unresolved: Some("wall_deadline"),
                    },
                    work,
                ));
            }
            let state = index.state(node.state_id)?;
            if state.terminal() && e1_terminal_distance(node.row, query.len(), radius) <= radius {
                if TRACE {
                    work.terminal_hits += 1;
                    terminal_ranks.push(node.rank_prefix);
                }
                form_refs.push(node.rank_prefix);
                if form_refs.len() > budget.maximum_terminals {
                    return Ok((
                        LaneOutcome {
                            form_refs,
                            expanded,
                            maximum_scratch,
                            unresolved: Some("terminal_budget"),
                        },
                        work,
                    ));
                }
            }
            if reverse_schedule {
                let mut children = Vec::new();
                for edge_id in index.edge_range(state)? {
                    let edge = index.edge(edge_id)?;
                    let rank_prefix = node
                        .rank_prefix
                        .checked_add(edge.rank_delta)
                        .ok_or_else(|| "V13 search rank overflows u32".to_string())?;
                    if TRACE {
                        work.examined_edges += 1;
                        rank_prefixes.push(rank_prefix);
                    }
                    let alphabet_id = e1_alphabet_id(edge.symbol).ok_or_else(|| {
                        format!("E1 edge has unknown symbol U+{:04X}", edge.symbol)
                    })?;
                    let equality =
                        e1_equality_window(node.row, masks[alphabet_id], radius, query.len());
                    let next = e1_u1_advance(node.row, equality, radius, query.len());
                    if next.minimum <= radius {
                        if TRACE {
                            work.surviving_edges += 1;
                        }
                        children.push(E1PackedNode {
                            state_id: edge.target,
                            rank_prefix,
                            row: next.state,
                        });
                    } else if TRACE {
                        work.pruned_edges += 1;
                    }
                }
                if TRACE {
                    work.stack_pushes += children.len();
                }
                stack.extend(children.into_iter().rev());
            } else {
                for edge_id in index.edge_range(state)? {
                    let edge = index.edge(edge_id)?;
                    let rank_prefix = node
                        .rank_prefix
                        .checked_add(edge.rank_delta)
                        .ok_or_else(|| "V13 search rank overflows u32".to_string())?;
                    if TRACE {
                        work.examined_edges += 1;
                        rank_prefixes.push(rank_prefix);
                    }
                    let alphabet_id = e1_alphabet_id(edge.symbol).ok_or_else(|| {
                        format!("E1 edge has unknown symbol U+{:04X}", edge.symbol)
                    })?;
                    let equality =
                        e1_equality_window(node.row, masks[alphabet_id], radius, query.len());
                    let next = e1_u1_advance(node.row, equality, radius, query.len());
                    if next.minimum <= radius {
                        if TRACE {
                            work.surviving_edges += 1;
                            work.stack_pushes += 1;
                        }
                        stack.push(E1PackedNode {
                            state_id: edge.target,
                            rank_prefix,
                            row: next.state,
                        });
                    } else if TRACE {
                        work.pruned_edges += 1;
                    }
                }
            }
            maximum_scratch = maximum_scratch.max(e1_scratch_bytes(&stack, &form_refs));
            if maximum_scratch > budget.maximum_scratch_bytes {
                return Ok((
                    LaneOutcome {
                        form_refs,
                        expanded,
                        maximum_scratch,
                        unresolved: Some("scratch_budget"),
                    },
                    work,
                ));
            }
        }
        Ok((
            LaneOutcome {
                form_refs,
                expanded,
                maximum_scratch,
                unresolved: None,
            },
            work,
        ))
    }

    fn e1_search<const TRACE: bool>(
        index: &V13DafsaView,
        package: &RuntimeL2Package,
        observed: &str,
        budget: SearchBudget,
        reverse_schedule: bool,
    ) -> Result<E1Observed, String> {
        let started = Instant::now();
        let oracle = Phase7dCertificateOracle::new(observed)?;
        let lanes = oracle.retrieval_lanes();
        let mut terminal_refs = Vec::new();
        let mut expanded = 0_usize;
        let mut maximum_scratch = 0_usize;
        let mut work = E1Work::default();
        let mut rank_prefixes = Vec::new();
        let mut terminal_ranks = Vec::new();
        for lane in lanes {
            let (outcome, lane_work) = e1_enumerate_lane::<TRACE>(
                index,
                &lane,
                budget,
                started,
                reverse_schedule,
                &mut rank_prefixes,
                &mut terminal_ranks,
            )?;
            if TRACE {
                work.add(lane_work);
            }
            expanded = expanded.saturating_add(outcome.expanded);
            maximum_scratch = maximum_scratch.max(outcome.maximum_scratch);
            if let Some(reason) = outcome.unresolved {
                let elapsed = elapsed_us(started);
                return Ok(E1Observed {
                    result: SearchResult {
                        retrieved_form_refs: Vec::new(),
                        peaks: Vec::new(),
                        completeness: SearchCompleteness::Unresolved(reason),
                        expanded_product_states: expanded,
                        maximum_scratch_bytes: maximum_scratch,
                        search_elapsed_us: elapsed,
                        material_elapsed_us: 0,
                        total_elapsed_us: elapsed,
                    },
                    work,
                    rank_prefixes,
                    terminal_ranks,
                });
            }
            terminal_refs.extend(outcome.form_refs);
        }
        terminal_refs.sort_unstable();
        terminal_refs.dedup();
        let search_elapsed_us = elapsed_us(started);
        let retrieved_form_refs = terminal_refs.clone();
        let mut peaks = Vec::new();
        for form_ref in terminal_refs {
            let surface = package
                .surface(form_ref as usize)
                .ok_or_else(|| format!("V13 terminal rank {form_ref} cannot be decoded"))?;
            let certificate_keys = oracle.certificate_keys(surface.as_ref())?;
            if !certificate_keys.is_empty() {
                peaks.push(V13TypedPeak {
                    form_ref,
                    certificate_keys,
                });
            }
        }
        let total_elapsed_us = elapsed_us(started);
        Ok(E1Observed {
            result: SearchResult {
                retrieved_form_refs,
                peaks,
                completeness: SearchCompleteness::CertifiedExhaustive,
                expanded_product_states: expanded,
                maximum_scratch_bytes: maximum_scratch,
                search_elapsed_us,
                material_elapsed_us: total_elapsed_us.saturating_sub(search_elapsed_us),
                total_elapsed_us,
            },
            work,
            rank_prefixes,
            terminal_ranks,
        })
    }

    fn e1_e0_trace_lane(
        index: &V13DafsaView,
        lane: &Phase7dRetrievalLane,
        reverse_schedule: bool,
        rank_prefixes: &mut Vec<u32>,
        terminal_ranks: &mut Vec<u32>,
    ) -> Result<(LaneOutcome, E1Work), String> {
        let query = lane.symbols.as_ref();
        let radius = lane.maximum_levenshtein_distance;
        if query.len() > MAX_QUERY_SYMBOLS || radius > MAX_LEVENSHTEIN_RADIUS {
            return Err("E1 E0 trace received an out-of-contract lane".to_string());
        }
        let mut stack = vec![SearchNode {
            state_id: index.root_state,
            rank_prefix: 0,
            row: BandedLevenshteinRow::initial(query.len(), radius),
        }];
        let mut form_refs = Vec::new();
        let mut expanded = 0_usize;
        let mut work = E1Work {
            stack_pushes: 1,
            ..E1Work::default()
        };
        let mut maximum_scratch = search_scratch_bytes(&stack, &form_refs);
        while let Some(node) = stack.pop() {
            expanded += 1;
            work.expanded_states += 1;
            work.stack_pops += 1;
            let state = index.state(node.state_id)?;
            if state.terminal() && node.row.terminal_distance(query.len(), radius) <= radius {
                work.terminal_hits += 1;
                terminal_ranks.push(node.rank_prefix);
                form_refs.push(node.rank_prefix);
            }
            if reverse_schedule {
                let mut children = Vec::new();
                for edge_id in index.edge_range(state)? {
                    let edge = index.edge(edge_id)?;
                    let rank_prefix = node
                        .rank_prefix
                        .checked_add(edge.rank_delta)
                        .ok_or_else(|| "V13 search rank overflows u32".to_string())?;
                    work.examined_edges += 1;
                    rank_prefixes.push(rank_prefix);
                    if let Some(child) = search_child(&node, query, edge, rank_prefix, lane) {
                        work.surviving_edges += 1;
                        children.push(child);
                    } else {
                        work.pruned_edges += 1;
                    }
                }
                work.stack_pushes += children.len();
                stack.extend(children.into_iter().rev());
            } else {
                for edge_id in index.edge_range(state)? {
                    let edge = index.edge(edge_id)?;
                    let rank_prefix = node
                        .rank_prefix
                        .checked_add(edge.rank_delta)
                        .ok_or_else(|| "V13 search rank overflows u32".to_string())?;
                    work.examined_edges += 1;
                    rank_prefixes.push(rank_prefix);
                    if let Some(child) = search_child(&node, query, edge, rank_prefix, lane) {
                        work.surviving_edges += 1;
                        work.stack_pushes += 1;
                        stack.push(child);
                    } else {
                        work.pruned_edges += 1;
                    }
                }
            }
            maximum_scratch = maximum_scratch.max(search_scratch_bytes(&stack, &form_refs));
        }
        Ok((
            LaneOutcome {
                form_refs,
                expanded,
                maximum_scratch,
                unresolved: None,
            },
            work,
        ))
    }

    fn e1_e0_trace(
        index: &V13DafsaView,
        observed: &str,
        reverse_schedule: bool,
    ) -> Result<(Vec<u32>, usize, usize, E1Work, Vec<u32>, Vec<u32>), String> {
        let oracle = Phase7dCertificateOracle::new(observed)?;
        let mut terminal_refs = Vec::new();
        let mut expanded = 0_usize;
        let mut maximum_scratch = 0_usize;
        let mut work = E1Work::default();
        let mut rank_prefixes = Vec::new();
        let mut terminal_ranks = Vec::new();
        for lane in oracle.retrieval_lanes() {
            let (outcome, lane_work) = e1_e0_trace_lane(
                index,
                &lane,
                reverse_schedule,
                &mut rank_prefixes,
                &mut terminal_ranks,
            )?;
            terminal_refs.extend(outcome.form_refs);
            expanded = expanded.saturating_add(outcome.expanded);
            maximum_scratch = maximum_scratch.max(outcome.maximum_scratch);
            work.add(lane_work);
        }
        terminal_refs.sort_unstable();
        terminal_refs.dedup();
        Ok((
            terminal_refs,
            expanded,
            maximum_scratch,
            work,
            rank_prefixes,
            terminal_ranks,
        ))
    }

    fn e1_transition_stress() -> (usize, usize, usize) {
        let mut cases = 0_usize;
        let mut transition_mismatches = 0_usize;
        let mut packed_state_mismatches = 0_usize;
        for query_len in 23_usize..=96 {
            for radius in 0_u8..=3 {
                let query = vec![1_u32; query_len];
                let mut next_query = vec![2_u32; query_len];
                let mut previous = BandedLevenshteinRow::initial(query_len, radius);
                for _ in 0..=query_len + radius as usize {
                    let packed = e1_pack(&previous);
                    packed_state_mismatches += usize::from(e1_pack(&e1_unpack(packed)) != packed);
                    let depth = previous.depth.saturating_add(1);
                    let start = depth.saturating_sub(radius) as usize;
                    let end = (depth as usize + radius as usize).min(query_len);
                    let len = end.checked_sub(start).map_or(0, |width| width + 1);
                    for equality in 0_u16..(1_u16 << len) {
                        next_query.fill(2);
                        for index in 0..len {
                            let column = start + index;
                            if column > 0 && equality & (1 << index) != 0 {
                                next_query[column - 1] = 1;
                            }
                        }
                        let generic = previous.advance(&next_query, 1, radius);
                        let fused = e1_u1_advance(packed, equality as u8, radius, query_len);
                        transition_mismatches += usize::from(
                            e1_pack(&generic) != fused.state
                                || generic.minimum(radius) != fused.minimum
                                || generic.terminal_distance(query_len, radius)
                                    != e1_terminal_distance(fused.state, query_len, radius),
                        );
                        cases += 1;
                    }
                    previous = previous.advance(&query, 1, radius);
                }
            }
        }
        (cases, transition_mismatches, packed_state_mismatches)
    }

    fn e1_focused_fixtures() -> Result<serde_json::Value, String> {
        let query = (0..96)
            .map(|index| {
                if index % 5 == 0 {
                    0x0451
                } else {
                    0x0430 + (index % 32) as u32
                }
            })
            .collect::<Vec<_>>();
        let masks = e1_eq_masks(&query);
        let symbol = 0x0451;
        let alphabet_id =
            e1_alphabet_id(symbol).ok_or_else(|| "fixture alphabet lookup".to_string())?;
        let mut equality_mismatches = 0_usize;
        for depth in 0_u8..=99 {
            for radius in 0_u8..=3 {
                let mut row = BandedLevenshteinRow::initial(query.len(), radius);
                row.depth = depth;
                row.start = depth.saturating_sub(radius);
                row.len = ((depth as usize + radius as usize)
                    .min(query.len())
                    .checked_sub(row.start as usize)
                    .map_or(0, |width| width + 1)) as u8;
                let packed = e1_pack(&row);
                let actual = e1_equality_window(packed, masks[alphabet_id], radius, query.len());
                let next_depth = depth.saturating_add(1);
                let start = next_depth.saturating_sub(radius) as usize;
                let end = (next_depth as usize + radius as usize).min(query.len());
                let len = end.checked_sub(start).map_or(0, |width| width + 1);
                let mut expected = 0_u8;
                for index in 0..len {
                    let column = start + index;
                    if column > 0 && query[column - 1] == symbol {
                        expected |= 1 << index;
                    }
                }
                equality_mismatches += usize::from(actual != expected);
            }
        }
        let initial = BandedLevenshteinRow::initial(96, 3);
        let initial_packed = e1_initial(96, 3);
        let terminal_edge_mismatches = usize::from(e1_pack(&initial) != initial_packed)
            + usize::from(
                e1_terminal_distance(initial_packed, 0, 3) != initial.terminal_distance(0, 3),
            )
            + usize::from(
                e1_terminal_distance(initial_packed, 96, 3) != initial.terminal_distance(96, 3),
            );
        let stack = Vec::<E1PackedNode>::with_capacity(17);
        let terminals = Vec::<u32>::with_capacity(11);
        let expected_scratch =
            17 * std::mem::size_of::<E1PackedNode>() + 11 * std::mem::size_of::<u32>();
        let scratch_mismatches =
            usize::from(e1_scratch_bytes(&stack, &terminals) != expected_scratch);
        let unknown_symbol_rejected = e1_alphabet_id(u32::from('A')).is_none();
        Ok(serde_json::json!({
            "equality_window_cases": 400,
            "equality_window_mismatches": equality_mismatches,
            "crosses_position_64": true,
            "unknown_symbol_rejected": unknown_symbol_rejected,
            "terminal_edge_mismatches": terminal_edge_mismatches,
            "stack_budget_mismatches": scratch_mismatches,
            "packed_node_bytes": std::mem::size_of::<E1PackedNode>(),
            "pass": equality_mismatches == 0 && unknown_symbol_rejected
                && terminal_edge_mismatches == 0 && scratch_mismatches == 0,
        }))
    }

    #[cfg(target_os = "linux")]
    fn e1_pin_current_thread(cpu: usize) -> Result<(), String> {
        unsafe {
            let mut set: libc::cpu_set_t = std::mem::zeroed();
            libc::CPU_ZERO(&mut set);
            libc::CPU_SET(cpu, &mut set);
            let result = libc::sched_setaffinity(0, std::mem::size_of::<libc::cpu_set_t>(), &set);
            if result != 0 {
                return Err(format!(
                    "sched_setaffinity({cpu}) failed: {}",
                    std::io::Error::last_os_error()
                ));
            }
        }
        Ok(())
    }

    #[cfg(not(target_os = "linux"))]
    fn e1_pin_current_thread(_cpu: usize) -> Result<(), String> {
        Err("E1 requires Linux affinity".to_string())
    }

    fn e1_current_affinity() -> Result<Vec<usize>, String> {
        #[cfg(target_os = "linux")]
        unsafe {
            let mut set: libc::cpu_set_t = std::mem::zeroed();
            libc::CPU_ZERO(&mut set);
            let result =
                libc::sched_getaffinity(0, std::mem::size_of::<libc::cpu_set_t>(), &mut set);
            if result != 0 {
                return Err(format!(
                    "sched_getaffinity failed: {}",
                    std::io::Error::last_os_error()
                ));
            }
            Ok((0..libc::CPU_SETSIZE as usize)
                .filter(|cpu| libc::CPU_ISSET(*cpu, &set))
                .collect())
        }
        #[cfg(not(target_os = "linux"))]
        {
            Err("E1 requires Linux affinity".to_string())
        }
    }

    fn e1_parse_cpus() -> Result<Vec<usize>, String> {
        let cpus = e1_required_string("LAY_V10_E1_CPUS")?
            .split(',')
            .map(|value| value.parse::<usize>().map_err(|error| error.to_string()))
            .collect::<Result<Vec<_>, _>>()?;
        if cpus.is_empty() || cpus.iter().copied().collect::<BTreeSet<_>>().len() != cpus.len() {
            return Err("E1 CPU list is empty or contains duplicates".to_string());
        }
        Ok(cpus)
    }

    fn e1_thread_migrations() -> Result<u64, String> {
        let source = std::fs::read_to_string("/proc/thread-self/sched")
            .map_err(|error| format!("/proc/thread-self/sched: {error}"))?;
        source
            .lines()
            .find_map(|line| {
                let (name, value) = line.split_once(':')?;
                (name.trim() == "se.nr_migrations" || name.trim() == "nr_migrations")
                    .then(|| value.trim().parse::<u64>().ok())
                    .flatten()
            })
            .ok_or_else(|| "thread migration counter is absent".to_string())
    }

    fn e1_sample(
        index: &V13DafsaView,
        package: &RuntimeL2Package,
        case: &ProofCase,
        query_ordinal: usize,
        round: usize,
        worker_id: u8,
    ) -> E1Sample {
        let outcome = e1_search::<false>(
            index,
            package,
            &case.damaged_surface,
            SearchBudget::proof(),
            false,
        );
        let sample = match &outcome {
            Ok(observed) => E1Sample {
                query_ordinal: query_ordinal as u16,
                round: round as u16,
                worker_id,
                flags: if observed.result.completeness == SearchCompleteness::CertifiedExhaustive {
                    0
                } else {
                    E1_FLAG_UNRESOLVED
                },
                search_elapsed_us: observed.result.search_elapsed_us,
                total_elapsed_us: observed.result.total_elapsed_us,
            },
            Err(_) => E1Sample {
                query_ordinal: query_ordinal as u16,
                round: round as u16,
                worker_id,
                flags: E1_FLAG_ERROR,
                search_elapsed_us: 0,
                total_elapsed_us: 0,
            },
        };
        std::hint::black_box(&outcome);
        drop(outcome);
        sample
    }

    fn e1_warmup(
        index: &V13DafsaView,
        package: &RuntimeL2Package,
        cases: &[ProofCase],
    ) -> Result<(), String> {
        for case in cases {
            let observed = e1_search::<false>(
                index,
                package,
                &case.damaged_surface,
                SearchBudget::proof(),
                false,
            )?;
            if observed.result.completeness != SearchCompleteness::CertifiedExhaustive {
                return Err("E1 warmup was unresolved".to_string());
            }
            std::hint::black_box(&observed.result);
        }
        Ok(())
    }

    fn e1_encode_samples(samples: &[E1Sample]) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(samples.len() * E1_SAMPLE_BYTES);
        for sample in samples {
            bytes.extend_from_slice(&sample.query_ordinal.to_le_bytes());
            bytes.extend_from_slice(&sample.round.to_le_bytes());
            bytes.push(sample.worker_id);
            bytes.push(sample.flags);
            bytes.extend_from_slice(&sample.search_elapsed_us.to_le_bytes());
            bytes.extend_from_slice(&sample.total_elapsed_us.to_le_bytes());
        }
        bytes
    }

    fn e1_sample_summary(samples: &[E1Sample]) -> serde_json::Value {
        serde_json::json!({
            "records": samples.len(),
            "errors": samples.iter().filter(|sample| sample.flags & E1_FLAG_ERROR != 0).count(),
            "unresolved": samples.iter().filter(|sample| sample.flags & E1_FLAG_UNRESOLVED != 0).count(),
            "sample_bytes": E1_SAMPLE_BYTES,
            "schema": E1_SAMPLE_SCHEMA,
        })
    }

    fn e1_output_dir() -> Result<std::path::PathBuf, String> {
        let path = e1_required_path("LAY_V10_E1_OUTPUT")?;
        if !path.is_dir() {
            return Err(format!("E1 output directory is absent: {}", path.display()));
        }
        Ok(path)
    }

    fn e1_run_parity() -> Result<serde_json::Value, String> {
        let inputs = e1_load_inputs()?;
        let bindings = ProofBindingIndex::build(&inputs.package)?;
        let mut terminal_mismatches = 0_usize;
        let mut peak_mismatches = 0_usize;
        let mut completeness_mismatches = 0_usize;
        let mut work_mismatches = 0_usize;
        let mut rank_prefix_mismatches = 0_usize;
        let mut terminal_rank_mismatches = 0_usize;
        let mut trace_authority_mismatches = 0_usize;
        let mut reverse_terminal_mismatches = 0_usize;
        let mut reverse_peak_mismatches = 0_usize;
        let mut reverse_completeness_mismatches = 0_usize;
        let mut reverse_work_mismatches = 0_usize;
        let mut reverse_rank_prefix_mismatches = 0_usize;
        let mut reverse_terminal_rank_mismatches = 0_usize;
        let mut full_row_terminal_mismatches = 0_usize;
        let mut full_row_peak_mismatches = 0_usize;
        let mut full_row_completeness_mismatches = 0_usize;
        let mut full_row_work_mismatches = 0_usize;
        let mut target_form_retained = 0_usize;
        let mut target_lemma_retained = 0_usize;
        let mut false_certificates = 0_usize;
        let mut maximum_product_states = 0_usize;
        let mut e0_maximum_scratch_bytes = 0_usize;
        let mut e1_maximum_scratch_bytes = 0_usize;
        let mut e0_work_total = E1Work::default();
        let mut e1_work_total = E1Work::default();
        let mut reverse_e0_work_total = E1Work::default();
        let mut reverse_e1_work_total = E1Work::default();

        for case in &inputs.cases {
            let e0 = search_typed_peaks(
                &inputs.index,
                &inputs.package,
                &case.damaged_surface,
                SearchBudget::proof(),
            )?;
            let e1 = e1_search::<true>(
                &inputs.index,
                &inputs.package,
                &case.damaged_surface,
                SearchBudget::proof(),
                false,
            )?;
            let full = search_typed_peaks_full_row(
                &inputs.index,
                &inputs.package,
                &case.damaged_surface,
                SearchBudget::proof(),
            )?;
            let (
                e0_trace_refs,
                e0_trace_expanded,
                e0_trace_scratch,
                e0_work,
                e0_ranks,
                e0_terminals,
            ) = e1_e0_trace(&inputs.index, &case.damaged_surface, false)?;
            terminal_mismatches +=
                usize::from(e0.retrieved_form_refs != e1.result.retrieved_form_refs);
            peak_mismatches += usize::from(e0.peaks != e1.result.peaks);
            completeness_mismatches += usize::from(e0.completeness != e1.result.completeness);
            work_mismatches +=
                usize::from(e0.expanded_product_states != e1.result.expanded_product_states)
                    + usize::from(e0_work != e1.work);
            rank_prefix_mismatches += usize::from(e0_ranks != e1.rank_prefixes);
            terminal_rank_mismatches += usize::from(e0_terminals != e1.terminal_ranks);
            trace_authority_mismatches += usize::from(e0.retrieved_form_refs != e0_trace_refs)
                + usize::from(e0.expanded_product_states != e0_trace_expanded)
                + usize::from(e0.maximum_scratch_bytes != e0_trace_scratch);
            full_row_terminal_mismatches +=
                usize::from(e0.retrieved_form_refs != full.retrieved_form_refs);
            full_row_peak_mismatches += usize::from(e0.peaks != full.peaks);
            full_row_completeness_mismatches += usize::from(e0.completeness != full.completeness);
            full_row_work_mismatches +=
                usize::from(e0.expanded_product_states != full.expanded_product_states);
            maximum_product_states = maximum_product_states.max(e1.result.expanded_product_states);
            e0_maximum_scratch_bytes = e0_maximum_scratch_bytes.max(e0.maximum_scratch_bytes);
            e1_maximum_scratch_bytes =
                e1_maximum_scratch_bytes.max(e1.result.maximum_scratch_bytes);
            e0_work_total.add(e0_work);
            e1_work_total.add(e1.work);

            let e1_reverse = e1_search::<true>(
                &inputs.index,
                &inputs.package,
                &case.damaged_surface,
                SearchBudget::proof(),
                true,
            )?;
            let (
                reverse_e0_refs,
                reverse_e0_expanded,
                _,
                reverse_e0_work,
                reverse_e0_ranks,
                reverse_e0_terminals,
            ) = e1_e0_trace(&inputs.index, &case.damaged_surface, true)?;
            reverse_terminal_mismatches +=
                usize::from(reverse_e0_refs != e1_reverse.result.retrieved_form_refs)
                    + usize::from(e0.retrieved_form_refs != e1_reverse.result.retrieved_form_refs);
            reverse_peak_mismatches += usize::from(e0.peaks != e1_reverse.result.peaks);
            reverse_completeness_mismatches +=
                usize::from(e0.completeness != e1_reverse.result.completeness);
            reverse_work_mismatches +=
                usize::from(reverse_e0_expanded != e1_reverse.result.expanded_product_states)
                    + usize::from(reverse_e0_work != e1_reverse.work);
            reverse_rank_prefix_mismatches +=
                usize::from(reverse_e0_ranks != e1_reverse.rank_prefixes);
            reverse_terminal_rank_mismatches +=
                usize::from(reverse_e0_terminals != e1_reverse.terminal_ranks);
            reverse_e0_work_total.add(reverse_e0_work);
            reverse_e1_work_total.add(e1_reverse.work);

            let frozen_refs = e1
                .result
                .peaks
                .iter()
                .map(|peak| peak.form_ref)
                .collect::<BTreeSet<_>>();
            let target_ref = inputs
                .package
                .form_ref_for_surface(&case.target_surface)
                .ok_or_else(|| format!("E1 V13 lacks target {}", case.target_surface))?;
            target_form_retained += usize::from(frozen_refs.contains(&target_ref));
            let target_lemmas = bindings.lemma_ids(&inputs.package, target_ref);
            let found_lemmas = frozen_refs
                .iter()
                .flat_map(|form_ref| bindings.lemma_ids(&inputs.package, *form_ref))
                .collect::<BTreeSet<_>>();
            target_lemma_retained += usize::from(
                !target_lemmas.is_empty()
                    && target_lemmas
                        .iter()
                        .any(|lemma| found_lemmas.contains(lemma)),
            );
            for peak in &e1.result.peaks {
                let surface = inputs
                    .package
                    .surface(peak.form_ref as usize)
                    .ok_or_else(|| format!("E1 missing emitted form {}", peak.form_ref))?;
                let verified = crate::nanda_wave::lexical_grokking::phase7d_certificate_keys(
                    &case.damaged_surface,
                    surface.as_ref(),
                )?;
                false_certificates +=
                    usize::from(verified.is_empty() || verified != peak.certificate_keys);
            }
        }
        let (stress_cases, stress_transition_mismatches, stress_packed_state_mismatches) =
            e1_transition_stress();
        let fixtures = e1_focused_fixtures()?;
        let pass = terminal_mismatches == 0
            && peak_mismatches == 0
            && completeness_mismatches == 0
            && work_mismatches == 0
            && rank_prefix_mismatches == 0
            && terminal_rank_mismatches == 0
            && trace_authority_mismatches == 0
            && reverse_terminal_mismatches == 0
            && reverse_peak_mismatches == 0
            && reverse_completeness_mismatches == 0
            && reverse_work_mismatches == 0
            && reverse_rank_prefix_mismatches == 0
            && reverse_terminal_rank_mismatches == 0
            && full_row_terminal_mismatches == 0
            && full_row_peak_mismatches == 0
            && full_row_completeness_mismatches == 0
            && full_row_work_mismatches == 0
            && target_form_retained == 382
            && target_lemma_retained == 382
            && false_certificates == 0
            && maximum_product_states == 35_590
            && e0_maximum_scratch_bytes == 6_656
            && e1_maximum_scratch_bytes <= 6_656
            && e0_work_total == e1_work_total
            && reverse_e0_work_total == reverse_e1_work_total
            && e0_work_total.expanded_states == E1_EXPECTED_EXPANDED
            && e0_work_total.examined_edges == E1_EXPECTED_TRANSITIONS
            && stress_cases == 714_026
            && stress_transition_mismatches == 0
            && stress_packed_state_mismatches == 0
            && fixtures.get("pass").and_then(serde_json::Value::as_bool) == Some(true);
        Ok(serde_json::json!({
            "schema": "lay.v10.e1-semantic-parity.v1",
            "verdict": if pass { "PASS" } else { "FAIL" },
            "test": E1_PARITY_TEST,
            "records": inputs.cases.len(),
            "schedule_records": inputs.schedule.len(),
            "terminal_mismatches": terminal_mismatches,
            "peak_mismatches": peak_mismatches,
            "completeness_mismatches": completeness_mismatches,
            "work_mismatches": work_mismatches,
            "rank_prefix_mismatches": rank_prefix_mismatches,
            "terminal_rank_mismatches": terminal_rank_mismatches,
            "trace_authority_mismatches": trace_authority_mismatches,
            "reverse_terminal_mismatches": reverse_terminal_mismatches,
            "reverse_peak_mismatches": reverse_peak_mismatches,
            "reverse_completeness_mismatches": reverse_completeness_mismatches,
            "reverse_work_mismatches": reverse_work_mismatches,
            "reverse_rank_prefix_mismatches": reverse_rank_prefix_mismatches,
            "reverse_terminal_rank_mismatches": reverse_terminal_rank_mismatches,
            "full_row_terminal_mismatches": full_row_terminal_mismatches,
            "full_row_peak_mismatches": full_row_peak_mismatches,
            "full_row_completeness_mismatches": full_row_completeness_mismatches,
            "full_row_work_mismatches": full_row_work_mismatches,
            "target_form_retained": target_form_retained,
            "target_lemma_retained": target_lemma_retained,
            "false_certificates": false_certificates,
            "maximum_product_states": maximum_product_states,
            "e0_maximum_scratch_bytes": e0_maximum_scratch_bytes,
            "e1_maximum_scratch_bytes": e1_maximum_scratch_bytes,
            "e0_work": {
                "expanded_states": e0_work_total.expanded_states,
                "examined_edges": e0_work_total.examined_edges,
                "surviving_edges": e0_work_total.surviving_edges,
                "pruned_edges": e0_work_total.pruned_edges,
                "stack_pushes": e0_work_total.stack_pushes,
                "stack_pops": e0_work_total.stack_pops,
                "terminal_hits": e0_work_total.terminal_hits,
            },
            "e1_work": {
                "expanded_states": e1_work_total.expanded_states,
                "examined_edges": e1_work_total.examined_edges,
                "surviving_edges": e1_work_total.surviving_edges,
                "pruned_edges": e1_work_total.pruned_edges,
                "stack_pushes": e1_work_total.stack_pushes,
                "stack_pops": e1_work_total.stack_pops,
                "terminal_hits": e1_work_total.terminal_hits,
            },
            "stress": {
                "cases": stress_cases,
                "transition_mismatches": stress_transition_mismatches,
                "packed_state_mismatches": stress_packed_state_mismatches,
            },
            "fixtures": fixtures,
            "latency_authority": false,
            "runtime_authority_changed": false,
            "installed_lay_changed": false,
        }))
    }

    fn e1_run_single() -> Result<serde_json::Value, String> {
        let inputs = e1_load_inputs()?;
        let cpus = e1_parse_cpus()?;
        if cpus.len() != 1 {
            return Err("E1 S requires exactly one CPU".to_string());
        }
        e1_pin_current_thread(cpus[0])?;
        if e1_current_affinity()? != cpus {
            return Err("E1 S affinity mismatch".to_string());
        }
        let migrations_before = e1_thread_migrations()?;
        e1_warmup(&inputs.index, &inputs.package, &inputs.cases)?;
        let expected = inputs.cases.len() * E1_SINGLE_ROUNDS;
        let mut samples = Vec::with_capacity(expected);
        let capacity = samples.capacity();
        for round in 0..E1_SINGLE_ROUNDS {
            for (query_ordinal, case) in inputs.cases.iter().enumerate() {
                samples.push(e1_sample(
                    &inputs.index,
                    &inputs.package,
                    case,
                    query_ordinal,
                    round,
                    E1_SINGLE_WORKER_SENTINEL,
                ));
            }
        }
        if samples.len() != expected || samples.capacity() != capacity {
            return Err("E1 S sample capacity changed".to_string());
        }
        let migrations_after = e1_thread_migrations()?;
        let output = e1_output_dir()?;
        let raw = e1_encode_samples(&samples);
        e1_write_new(&output.join("samples.bin"), &raw)?;
        Ok(serde_json::json!({
            "schema": "lay.v10.e1-single-process.v1",
            "test": E1_SINGLE_TEST,
            "run_id": e1_required_string("LAY_V10_E1_RUN_ID")?,
            "rounds": E1_SINGLE_ROUNDS,
            "queries": inputs.cases.len(),
            "warmup_queries": inputs.cases.len(),
            "cpus": cpus,
            "sample_capacity": capacity,
            "thread_migrations_before": migrations_before,
            "thread_migrations_after": migrations_after,
            "thread_migrations_delta": migrations_after.saturating_sub(migrations_before),
            "samples_sha256": e1_sha256_text(&raw),
            "samples": e1_sample_summary(&samples),
            "production_timing_authority": ["search_elapsed_us", "total_elapsed_us"],
            "external_call_wall_authority": false,
            "runtime_authority_changed": false,
            "installed_lay_changed": false,
        }))
    }

    fn e1_run_twenty() -> Result<serde_json::Value, String> {
        let inputs = e1_load_inputs()?;
        let cpus = e1_parse_cpus()?;
        if cpus.len() != E1_WORKERS {
            return Err("E1 T requires exactly twenty CPUs".to_string());
        }
        let start = std::sync::Arc::new(std::sync::Barrier::new(E1_WORKERS + 1));
        let end = std::sync::Arc::new(std::sync::Barrier::new(E1_WORKERS + 1));
        let worker_outputs = std::thread::scope(|scope| {
            let handles = (0..E1_WORKERS)
                .map(|worker| {
                    let start = std::sync::Arc::clone(&start);
                    let end = std::sync::Arc::clone(&end);
                    let cpu = cpus[worker];
                    let chunk_start = worker * 20;
                    let chunk_end = (chunk_start + 20).min(inputs.cases.len());
                    let cases = &inputs.cases[chunk_start..chunk_end];
                    let index = &inputs.index;
                    let package = &inputs.package;
                    scope.spawn(
                        move || -> Result<(usize, Vec<E1Sample>, Vec<usize>, u64), String> {
                            let mut failure = e1_pin_current_thread(cpu).err();
                            let affinity = e1_current_affinity().unwrap_or_default();
                            if failure.is_none() && affinity != [cpu] {
                                failure = Some(format!("E1 T worker {worker} affinity mismatch"));
                            }
                            let migrations_before = e1_thread_migrations().unwrap_or_default();
                            let expected = cases.len() * E1_TWENTY_ROUNDS;
                            let mut samples = Vec::with_capacity(expected);
                            let capacity = samples.capacity();
                            for burst in 0..=E1_TWENTY_ROUNDS {
                                start.wait();
                                if failure.is_none() {
                                    for (offset, case) in cases.iter().enumerate() {
                                        if burst == 0 {
                                            match e1_search::<false>(
                                            index,
                                            package,
                                            &case.damaged_surface,
                                            SearchBudget::proof(),
                                            false,
                                        ) {
                                            Ok(observed)
                                                if observed.result.completeness
                                                    == SearchCompleteness::CertifiedExhaustive =>
                                            {
                                                std::hint::black_box(&observed.result);
                                            }
                                            Ok(_) => {
                                                failure = Some(format!(
                                                    "E1 T worker {worker} warmup unresolved"
                                                ));
                                                break;
                                            }
                                            Err(error) => {
                                                failure = Some(format!(
                                                    "E1 T worker {worker} warmup: {error}"
                                                ));
                                                break;
                                            }
                                        }
                                        } else {
                                            samples.push(e1_sample(
                                                index,
                                                package,
                                                case,
                                                chunk_start + offset,
                                                burst - 1,
                                                worker as u8,
                                            ));
                                        }
                                    }
                                }
                                end.wait();
                            }
                            if let Some(error) = failure {
                                return Err(error);
                            }
                            let migrations_after = e1_thread_migrations()?;
                            if samples.len() != expected || samples.capacity() != capacity {
                                return Err(format!("E1 T worker {worker} capacity changed"));
                            }
                            Ok((
                                capacity,
                                samples,
                                affinity,
                                migrations_after.saturating_sub(migrations_before),
                            ))
                        },
                    )
                })
                .collect::<Vec<_>>();
            for _ in 0..=E1_TWENTY_ROUNDS {
                start.wait();
                end.wait();
            }
            handles
                .into_iter()
                .map(|handle| {
                    handle
                        .join()
                        .map_err(|_| "E1 T worker panicked".to_string())?
                })
                .collect::<Result<Vec<_>, String>>()
        })?;

        let mut samples = Vec::with_capacity(inputs.cases.len() * E1_TWENTY_ROUNDS);
        let mut capacities = Vec::with_capacity(E1_WORKERS);
        let mut affinities = Vec::with_capacity(E1_WORKERS);
        let mut migration_deltas = Vec::with_capacity(E1_WORKERS);
        for (capacity, mut worker_samples, affinity, migrations) in worker_outputs {
            capacities.push(capacity);
            affinities.push(affinity);
            migration_deltas.push(migrations);
            samples.append(&mut worker_samples);
        }
        if samples.len() != inputs.cases.len() * E1_TWENTY_ROUNDS {
            return Err("E1 T pooled sample count mismatch".to_string());
        }
        let output = e1_output_dir()?;
        let raw = e1_encode_samples(&samples);
        e1_write_new(&output.join("samples.bin"), &raw)?;
        Ok(serde_json::json!({
            "schema": "lay.v10.e1-twenty-worker-process.v1",
            "test": E1_TWENTY_TEST,
            "run_id": e1_required_string("LAY_V10_E1_RUN_ID")?,
            "rounds": E1_TWENTY_ROUNDS,
            "queries": inputs.cases.len(),
            "workers": E1_WORKERS,
            "chunk_size": 20,
            "warmup_bursts": 1,
            "start_barriers": E1_TWENTY_ROUNDS + 1,
            "end_barriers": E1_TWENTY_ROUNDS + 1,
            "cpus": cpus,
            "worker_affinities": affinities,
            "worker_capacities": capacities,
            "worker_migration_deltas": migration_deltas,
            "samples_sha256": e1_sha256_text(&raw),
            "samples": e1_sample_summary(&samples),
            "production_timing_authority": ["search_elapsed_us", "total_elapsed_us"],
            "external_call_wall_authority": false,
            "runtime_authority_changed": false,
            "installed_lay_changed": false,
        }))
    }

    fn e1_wait_for(path: &Path) -> Result<(), String> {
        let started = Instant::now();
        while !path.is_file() {
            if started.elapsed() > Duration::from_secs(120) {
                return Err(format!("timed out waiting for {}", path.display()));
            }
            std::thread::sleep(Duration::from_millis(1));
        }
        Ok(())
    }

    fn e1_run_physical(candidate: bool) -> Result<serde_json::Value, String> {
        let inputs = e1_load_inputs()?;
        let cpus = e1_parse_cpus()?;
        if cpus.len() != 1 {
            return Err("E1 physical route requires exactly one CPU".to_string());
        }
        e1_pin_current_thread(cpus[0])?;
        if e1_current_affinity()? != cpus {
            return Err("E1 physical affinity mismatch".to_string());
        }
        for case in &inputs.cases {
            if candidate {
                let observed = e1_search::<false>(
                    &inputs.index,
                    &inputs.package,
                    &case.damaged_surface,
                    SearchBudget::proof(),
                    false,
                )?;
                if observed.result.completeness != SearchCompleteness::CertifiedExhaustive {
                    return Err("E1 physical candidate warmup unresolved".to_string());
                }
                std::hint::black_box(&observed.result);
            } else {
                let result = search_typed_peaks(
                    &inputs.index,
                    &inputs.package,
                    &case.damaged_surface,
                    SearchBudget::proof(),
                )?;
                if result.completeness != SearchCompleteness::CertifiedExhaustive {
                    return Err("E1 physical E0 warmup unresolved".to_string());
                }
                std::hint::black_box(&result);
            }
        }
        let control = e1_required_path("LAY_V10_E1_CONTROL_DIR")?;
        if !control.is_dir() {
            return Err(format!(
                "E1 control directory is absent: {}",
                control.display()
            ));
        }
        let ready = control.join("subject-ready");
        let enabled = control.join("controller-enabled");
        let done = control.join("subject-done");
        let disabled = control.join("controller-disabled");
        e1_write_new(&ready, b"ready\n")?;
        e1_wait_for(&enabled)?;
        let measured_started = Instant::now();
        let mut checksum = 0xcbf29ce484222325_u64;
        let mut search_elapsed_sum_us = 0_u64;
        let mut total_elapsed_sum_us = 0_u64;
        let mut errors = 0_usize;
        let mut unresolved = 0_usize;
        for case in &inputs.cases {
            if candidate {
                match e1_search::<false>(
                    &inputs.index,
                    &inputs.package,
                    &case.damaged_surface,
                    SearchBudget::proof(),
                    false,
                ) {
                    Ok(observed) => {
                        let result = &observed.result;
                        unresolved += usize::from(
                            result.completeness != SearchCompleteness::CertifiedExhaustive,
                        );
                        checksum = checksum.wrapping_mul(0x100000001b3)
                            ^ result.expanded_product_states as u64
                            ^ ((result.retrieved_form_refs.len() as u64) << 32)
                            ^ result.peaks.len() as u64;
                        search_elapsed_sum_us =
                            search_elapsed_sum_us.saturating_add(result.search_elapsed_us);
                        total_elapsed_sum_us =
                            total_elapsed_sum_us.saturating_add(result.total_elapsed_us);
                        std::hint::black_box(result);
                    }
                    Err(_) => errors += 1,
                }
            } else {
                match search_typed_peaks(
                    &inputs.index,
                    &inputs.package,
                    &case.damaged_surface,
                    SearchBudget::proof(),
                ) {
                    Ok(result) => {
                        unresolved += usize::from(
                            result.completeness != SearchCompleteness::CertifiedExhaustive,
                        );
                        checksum = checksum.wrapping_mul(0x100000001b3)
                            ^ result.expanded_product_states as u64
                            ^ ((result.retrieved_form_refs.len() as u64) << 32)
                            ^ result.peaks.len() as u64;
                        search_elapsed_sum_us =
                            search_elapsed_sum_us.saturating_add(result.search_elapsed_us);
                        total_elapsed_sum_us =
                            total_elapsed_sum_us.saturating_add(result.total_elapsed_us);
                        std::hint::black_box(&result);
                    }
                    Err(_) => errors += 1,
                }
            }
        }
        let measured_wall_ns = measured_started.elapsed().as_nanos();
        e1_write_new(&done, b"done\n")?;
        e1_wait_for(&disabled)?;
        if errors != 0 || unresolved != 0 {
            return Err(format!(
                "E1 physical errors={errors} unresolved={unresolved}"
            ));
        }
        Ok(serde_json::json!({
            "schema": "lay.v10.exact-fused-executor-e1-physical.v1",
            "verdict": "PASS",
            "test": if candidate { E1_E1_PHYSICAL_TEST } else { E1_E0_PHYSICAL_TEST },
            "executor": if candidate { "E1" } else { "E0" },
            "queries": inputs.cases.len(),
            "warmup_queries": inputs.cases.len(),
            "checksum": checksum,
            "errors": errors,
            "unresolved": unresolved,
            "search_elapsed_sum_us_diagnostic": search_elapsed_sum_us,
            "total_elapsed_sum_us_diagnostic": total_elapsed_sum_us,
            "measured_wall_ns_diagnostic": measured_wall_ns,
            "cpus": cpus,
            "control_protocol": ["subject-ready", "controller-enabled", "subject-done", "controller-disabled"],
            "runtime_authority_changed": false,
            "installed_lay_changed": false,
        }))
    }

    fn e1_subject(test: &str) {
        let output = e1_required_path("LAY_V10_E1_OUTPUT").expect("E1 output path");
        let result = match test {
            E1_PARITY_TEST => e1_run_parity(),
            E1_SINGLE_TEST => e1_run_single(),
            E1_TWENTY_TEST => e1_run_twenty(),
            E1_E0_PHYSICAL_TEST => e1_run_physical(false),
            E1_E1_PHYSICAL_TEST => e1_run_physical(true),
            _ => Err(format!("unknown E1 test {test}")),
        };
        let receipt = result.unwrap_or_else(|error| {
            serde_json::json!({
                "schema": "lay.v10.e1-subject-error.v1",
                "test": test,
                "error": error,
                "runtime_authority_changed": false,
                "installed_lay_changed": false,
            })
        });
        e1_write_json(&output.join("SUBJECT_RECEIPT.json"), &receipt)
            .expect("write E1 subject receipt");
        assert!(
            receipt.get("error").is_none(),
            "{}",
            serde_json::to_string_pretty(&receipt).unwrap()
        );
        if test == E1_PARITY_TEST || test == E1_E0_PHYSICAL_TEST || test == E1_E1_PHYSICAL_TEST {
            assert_eq!(
                receipt.get("verdict").and_then(serde_json::Value::as_str),
                Some("PASS")
            );
        }
    }

    #[test]
    #[ignore = "E1 separate semantic prerequisite; admitted only by E1 preflight"]
    fn v10_e1_semantic_parity() {
        e1_subject(E1_PARITY_TEST);
    }

    #[test]
    #[ignore = "E1 direct single-process latency; admitted only after parity and clean admission"]
    fn v10_e1_single_latency() {
        e1_subject(E1_SINGLE_TEST);
    }

    #[test]
    #[ignore = "E1 twenty-worker fixed-shard burst; admitted only after parity and clean admission"]
    fn v10_e1_twenty_worker_latency() {
        e1_subject(E1_TWENTY_TEST);
    }

    #[test]
    #[ignore = "E1 exact V10 E0 full-executor PMU denominator"]
    fn v10_e1_e0_physical() {
        e1_subject(E1_E0_PHYSICAL_TEST);
    }

    #[test]
    #[ignore = "E1 packed full-executor PMU candidate"]
    fn v10_e1_e1_physical() {
        e1_subject(E1_E1_PHYSICAL_TEST);
    }
