
    const D1_PARITY_TEST: &str =
        "nanda_wave::l2_field::v13_typed_peak::tests::v10_d1_semantic_parity";
    const D1_SINGLE_TEST: &str =
        "nanda_wave::l2_field::v13_typed_peak::tests::v10_d1_single_latency";
    const D1_TWENTY_TEST: &str =
        "nanda_wave::l2_field::v13_typed_peak::tests::v10_d1_twenty_worker_latency";
    const D1_E0_PHYSICAL_TEST: &str =
        "nanda_wave::l2_field::v13_typed_peak::tests::v10_d1_e0_physical";
    const D1_D1_PHYSICAL_TEST: &str =
        "nanda_wave::l2_field::v13_typed_peak::tests::v10_d1_d1_physical";
    const D1_V7_SHA256: &str = "33fded73e13f565bdc08a83bd440be67c80f6d85a196453050b09bc9f7ef28e4";
    const D1_SIDECAR_SHA256: &str =
        "a1aa95be8c43fae8fedf02d3261dd1265dcbfbc8d618330fcd6449ae829df8cd";
    const D1_SCHEDULE_SHA256: &str =
        "2f8346d97246ac3704434279739fa5ed27705465d628890a367c21875d879e78";
    const D1_SCHEDULE_SCHEMA: &str = "lay.v10.hardware-query-schedule.v1";
    const D1_SAMPLE_SCHEMA: &str = "lay.v10.e1-primitive-samples.v1";
    const D1_SAMPLE_BYTES: usize = 22;
    const D1_SINGLE_ROUNDS: usize = 100;
    const D1_TWENTY_ROUNDS: usize = 250;
    const D1_WORKERS: usize = 20;
    const D1_SINGLE_WORKER_SENTINEL: u8 = u8::MAX;
    const D1_FLAG_ERROR: u8 = 1;
    const D1_FLAG_UNRESOLVED: u8 = 2;
    const D1_ALPHABET_SYMBOLS: usize = 34;
    const D1_EXPECTED_TRANSITIONS: usize = 25_145_756;
    const D1_EXPECTED_EXPANDED: usize = 8_059_788;

    #[derive(Clone)]
    struct D1ScheduleEntry {
        source_ordinal: usize,
        composite_identity_sha256: String,
        damaged_surface_sha256: String,
        retrieval_lanes_sha256: String,
        worker: usize,
        chunk_start: usize,
        chunk_end: usize,
    }

    struct D1Inputs {
        package: RuntimeL2Package,
        index: V13DafsaView,
        cases: Vec<ProofCase>,
        schedule: Vec<D1ScheduleEntry>,
    }

    #[derive(Clone, Copy)]
    struct D1Sample {
        query_ordinal: u16,
        round: u16,
        worker_id: u8,
        flags: u8,
        search_elapsed_us: u64,
        total_elapsed_us: u64,
    }

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    struct D1FusedOutput {
        state: u64,
        minimum: u8,
    }

    #[derive(Clone, Copy)]
    struct D1PackedNode {
        state_id: u32,
        rank_prefix: u32,
        row: u64,
    }

    #[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
    struct D1Work {
        expanded_states: usize,
        examined_edges: usize,
        surviving_edges: usize,
        pruned_edges: usize,
        stack_pushes: usize,
        stack_pops: usize,
        terminal_hits: usize,
    }

    impl D1Work {
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

    struct D1Observed {
        result: SearchResult,
        work: D1Work,
        rank_prefixes: Vec<u32>,
        terminal_ranks: Vec<u32>,
    }

    fn d1_required_path(name: &str) -> Result<std::path::PathBuf, String> {
        std::env::var_os(name)
            .map(std::path::PathBuf::from)
            .ok_or_else(|| format!("required environment path {name} is absent"))
    }

    fn d1_required_string(name: &str) -> Result<String, String> {
        std::env::var(name).map_err(|error| format!("required environment {name}: {error}"))
    }

    fn d1_write_new(path: &Path, bytes: &[u8]) -> Result<(), String> {
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

    fn d1_write_json(path: &Path, value: &serde_json::Value) -> Result<(), String> {
        let mut bytes = serde_json::to_vec_pretty(value).map_err(|error| error.to_string())?;
        bytes.push(b'\n');
        d1_write_new(path, &bytes)
    }

    fn d1_sha256_text(value: &[u8]) -> String {
        hex(Sha256::digest(value).into())
    }

    fn d1_compact_lanes(lanes: &[Phase7dRetrievalLane]) -> Result<Vec<u8>, String> {
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

    fn d1_schedule_entries(value: &serde_json::Value) -> Result<Vec<D1ScheduleEntry>, String> {
        if value.get("schema").and_then(serde_json::Value::as_str) != Some(D1_SCHEDULE_SCHEMA) {
            return Err("D1 schedule schema mismatch".to_string());
        }
        let partition = value
            .get("partition")
            .and_then(serde_json::Value::as_object)
            .ok_or_else(|| "D1 schedule partition is absent".to_string())?;
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
            return Err("D1 schedule partition mismatch".to_string());
        }
        value
            .get("entries")
            .and_then(serde_json::Value::as_array)
            .ok_or_else(|| "D1 schedule entries are absent".to_string())?
            .iter()
            .map(|entry| {
                let number = |name: &str| {
                    entry
                        .get(name)
                        .and_then(serde_json::Value::as_u64)
                        .and_then(|value| usize::try_from(value).ok())
                        .ok_or_else(|| format!("D1 schedule entry lacks {name}"))
                };
                let string = |name: &str| {
                    entry
                        .get(name)
                        .and_then(serde_json::Value::as_str)
                        .map(str::to_string)
                        .ok_or_else(|| format!("D1 schedule entry lacks {name}"))
                };
                Ok(D1ScheduleEntry {
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

    fn d1_validate_schedule(
        cases: &[ProofCase],
        schedule: &[D1ScheduleEntry],
    ) -> Result<(), String> {
        if cases.len() != 382 || schedule.len() != cases.len() {
            return Err("D1 schedule denominator mismatch".to_string());
        }
        for (ordinal, (case, entry)) in cases.iter().zip(schedule).enumerate() {
            if entry.source_ordinal != ordinal {
                return Err(format!("D1 source ordinal mismatch at {ordinal}"));
            }
            let composite = [case.class.as_bytes(), &[0], case.proof_identity.as_bytes()].concat();
            if d1_sha256_text(&composite) != entry.composite_identity_sha256 {
                return Err(format!("D1 composite identity mismatch at {ordinal}"));
            }
            if d1_sha256_text(case.damaged_surface.as_bytes()) != entry.damaged_surface_sha256 {
                return Err(format!("D1 damaged surface mismatch at {ordinal}"));
            }
            let lanes = Phase7dCertificateOracle::new(&case.damaged_surface)?.retrieval_lanes();
            if d1_sha256_text(&d1_compact_lanes(&lanes)?) != entry.retrieval_lanes_sha256 {
                return Err(format!("D1 retrieval lanes mismatch at {ordinal}"));
            }
            let worker = ordinal / 20;
            let start = worker * 20;
            let end = (start + 20).min(cases.len());
            if entry.worker != worker || entry.chunk_start != start || entry.chunk_end != end {
                return Err(format!("D1 worker partition mismatch at {ordinal}"));
            }
        }
        Ok(())
    }

    fn d1_load_inputs() -> Result<D1Inputs, String> {
        let package_path = d1_required_path("LAY_V10_D1_PACKAGE")?;
        let sidecar_path = d1_required_path("LAY_V10_D1_SIDECAR")?;
        let v7_path = d1_required_path("LAY_V10_D1_V7")?;
        let schedule_path = d1_required_path("LAY_V10_D1_SCHEDULE")?;

        let package_bytes = std::fs::metadata(&package_path)
            .map_err(|error| format!("{}: {error}", package_path.display()))?
            .len();
        let package_sha256 = sha256_file(&package_path)?;
        if hex(package_sha256) != EXPECTED_V13_SHA256 || package_bytes != EXPECTED_V13_BYTES {
            return Err("D1 V13 package identity mismatch".to_string());
        }
        if hex(sha256_file(&sidecar_path)?) != D1_SIDECAR_SHA256 {
            return Err("D1 sidecar identity mismatch".to_string());
        }
        if hex(sha256_file(&v7_path)?) != D1_V7_SHA256 {
            return Err("D1 V7 identity mismatch".to_string());
        }
        if hex(sha256_file(&schedule_path)?) != D1_SCHEDULE_SHA256 {
            return Err("D1 schedule identity mismatch".to_string());
        }

        let package = RuntimeL2Package::load(&package_path)?;
        if package.form_count() != EXPECTED_V13_FORMS {
            return Err("D1 V13 form count mismatch".to_string());
        }
        let identity = V13Identity {
            package_sha256,
            package_bytes,
            form_count: u32::try_from(package.form_count())
                .map_err(|_| "D1 form count exceeds u32".to_string())?,
            binding_count: u32::try_from(package.binding_count())
                .map_err(|_| "D1 binding count exceeds u32".to_string())?,
        };
        let index = V13DafsaView::load(&sidecar_path, identity)?;
        d1_validate_dense_alphabet(&index)?;
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
        let schedule = d1_schedule_entries(&schedule_value)?;
        d1_validate_schedule(&cases, &schedule)?;
        Ok(D1Inputs {
            package,
            index,
            cases,
            schedule,
        })
    }

    #[inline(always)]
    fn d1_alphabet_id(symbol: u32) -> Option<usize> {
        match symbol {
            0x002d => Some(0),
            0x0430..=0x044f => Some((symbol - 0x0430 + 1) as usize),
            0x0451 => Some(33),
            _ => None,
        }
    }

    fn d1_validate_dense_alphabet(index: &V13DafsaView) -> Result<(), String> {
        let mut seen = [false; D1_ALPHABET_SYMBOLS];
        for edge_id in 0..index.edge_count as usize {
            let edge = index.edge(edge_id)?;
            let alphabet_id = d1_alphabet_id(edge.symbol).ok_or_else(|| {
                format!("D1 sidecar edge has unknown symbol U+{:04X}", edge.symbol)
            })?;
            seen[alphabet_id] = true;
        }
        if seen.iter().any(|present| !present) {
            return Err("D1 sidecar does not contain the complete dense alphabet".to_string());
        }
        Ok(())
    }

    #[inline(always)]
    fn d1_pack(row: &BandedLevenshteinRow) -> u64 {
        let mut value = 0_u64;
        for (index, cell) in row.cells.iter().copied().enumerate() {
            value |= u64::from(cell) << (index * 3);
        }
        value
            | (u64::from(row.depth) << 21)
            | (u64::from(row.start) << 29)
            | (u64::from(row.len) << 37)
    }

    fn d1_unpack(value: u64) -> BandedLevenshteinRow {
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
    fn d1_packed_value(value: u64, column: usize, outside: u8) -> u8 {
        let start = ((value >> 29) & 0xff) as usize;
        let len = ((value >> 37) & 7) as usize;
        if column >= start && column < start + len {
            ((value >> ((column - start) * 3)) & 7) as u8
        } else {
            outside
        }
    }

    #[inline(always)]
    fn d1_initial(query_len: usize, radius: u8) -> u64 {
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

    fn d1_eq_masks(query: &[u32]) -> [[u64; 2]; D1_ALPHABET_SYMBOLS] {
        let mut masks = [[0_u64; 2]; D1_ALPHABET_SYMBOLS];
        for (position, symbol) in query.iter().copied().enumerate() {
            if let Some(alphabet_id) = d1_alphabet_id(symbol) {
                masks[alphabet_id][position / 64] |= 1_u64 << (position % 64);
            }
        }
        masks
    }

    #[inline(always)]
    fn d1_extract_seven(mask: [u64; 2], position: usize) -> u8 {
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
    fn d1_equality_window(previous: u64, mask: [u64; 2], radius: u8, query_len: usize) -> u8 {
        let previous_depth = ((previous >> 21) & 0xff) as u8;
        let depth = previous_depth.checked_add(1).expect("bounded query depth");
        let start = depth.saturating_sub(radius) as usize;
        let end = (depth as usize + radius as usize).min(query_len);
        let len = end.checked_sub(start).map_or(0, |width| width + 1);
        if len == 0 {
            return 0;
        }
        let equality = if start == 0 {
            (d1_extract_seven(mask, 0) << 1) & 0x7f
        } else {
            d1_extract_seven(mask, start - 1)
        };
        equality & ((1_u16 << len) - 1) as u8
    }

    #[inline(always)]
    fn d1_u1_cell<const INDEX: usize>(
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
            let above = d1_packed_value(previous, column, outside)
                .saturating_add(1)
                .min(outside);
            let diagonal = d1_packed_value(previous, column - 1, outside)
                .saturating_add(u8::from(equality & (1 << INDEX) == 0))
                .min(outside);
            left.min(above).min(diagonal)
        }
    }

    #[inline(always)]
    fn d1_u1_advance(previous: u64, equality: u8, radius: u8, query_len: usize) -> D1FusedOutput {
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
            return D1FusedOutput {
                state,
                minimum: outside,
            };
        }
        let c0 = d1_u1_cell::<0>(previous, equality, radius, depth, start, outside);
        let c1 = if len > 1 {
            d1_u1_cell::<1>(previous, equality, radius, depth, start, c0)
        } else {
            outside
        };
        let c2 = if len > 2 {
            d1_u1_cell::<2>(previous, equality, radius, depth, start, c1)
        } else {
            outside
        };
        let c3 = if len > 3 {
            d1_u1_cell::<3>(previous, equality, radius, depth, start, c2)
        } else {
            outside
        };
        let c4 = if len > 4 {
            d1_u1_cell::<4>(previous, equality, radius, depth, start, c3)
        } else {
            outside
        };
        let c5 = if len > 5 {
            d1_u1_cell::<5>(previous, equality, radius, depth, start, c4)
        } else {
            outside
        };
        let c6 = if len > 6 {
            d1_u1_cell::<6>(previous, equality, radius, depth, start, c5)
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
        D1FusedOutput { state, minimum }
    }

    #[inline(always)]
    fn d1_terminal_distance(state: u64, query_len: usize, radius: u8) -> u8 {
        d1_packed_value(state, query_len, radius.saturating_add(1))
    }

    fn d1_scratch_bytes(stack: &Vec<D1PackedNode>, terminals: &Vec<u32>) -> usize {
        stack.capacity() * std::mem::size_of::<D1PackedNode>()
            + terminals.capacity() * std::mem::size_of::<u32>()
    }

    fn d1_enumerate_lane<const TRACE: bool>(
        index: &V13DafsaView,
        lane: &Phase7dRetrievalLane,
        budget: SearchBudget,
        started: Instant,
        reverse_schedule: bool,
        rank_prefixes: &mut Vec<u32>,
        terminal_ranks: &mut Vec<u32>,
    ) -> Result<(LaneOutcome, D1Work), String> {
        let query = lane.symbols.as_ref();
        if query.len() > MAX_QUERY_SYMBOLS {
            return Ok((
                LaneOutcome {
                    form_refs: Vec::new(),
                    expanded: 0,
                    maximum_scratch: 0,
                    unresolved: Some("query_symbol_budget"),
                },
                D1Work::default(),
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
                D1Work::default(),
            ));
        }
        let masks = d1_eq_masks(query);
        let mut stack = vec![D1PackedNode {
            state_id: index.root_state,
            rank_prefix: 0,
            row: d1_initial(query.len(), radius),
        }];
        let mut form_refs = Vec::new();
        let mut expanded = 0_usize;
        let mut work = D1Work::default();
        if TRACE {
            work.stack_pushes = 1;
        }
        let mut maximum_scratch = d1_scratch_bytes(&stack, &form_refs);
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
            if state.terminal() && d1_terminal_distance(node.row, query.len(), radius) <= radius {
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
                    let alphabet_id = d1_alphabet_id(edge.symbol).ok_or_else(|| {
                        format!("D1 edge has unknown symbol U+{:04X}", edge.symbol)
                    })?;
                    let equality =
                        d1_equality_window(node.row, masks[alphabet_id], radius, query.len());
                    let next = d1_u1_advance(node.row, equality, radius, query.len());
                    if next.minimum <= radius {
                        if TRACE {
                            work.surviving_edges += 1;
                        }
                        children.push(D1PackedNode {
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
                    let alphabet_id = d1_alphabet_id(edge.symbol).ok_or_else(|| {
                        format!("D1 edge has unknown symbol U+{:04X}", edge.symbol)
                    })?;
                    let equality =
                        d1_equality_window(node.row, masks[alphabet_id], radius, query.len());
                    let next = d1_u1_advance(node.row, equality, radius, query.len());
                    if next.minimum <= radius {
                        if TRACE {
                            work.surviving_edges += 1;
                            work.stack_pushes += 1;
                        }
                        stack.push(D1PackedNode {
                            state_id: edge.target,
                            rank_prefix,
                            row: next.state,
                        });
                    } else if TRACE {
                        work.pruned_edges += 1;
                    }
                }
            }
            maximum_scratch = maximum_scratch.max(d1_scratch_bytes(&stack, &form_refs));
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

    #[inline(never)]
    fn d1_enumerate_lane_prepared<const TRACE: bool>(
        index: &V13DafsaView,
        lane: &Phase7dRetrievalLane,
        masks: &[[u64; 2]; D1_ALPHABET_SYMBOLS],
        budget: SearchBudget,
        started: Instant,
        reverse_schedule: bool,
        rank_prefixes: &mut Vec<u32>,
        terminal_ranks: &mut Vec<u32>,
    ) -> Result<(LaneOutcome, D1Work), String> {
        let query = lane.symbols.as_ref();
        if query.len() > MAX_QUERY_SYMBOLS {
            return Ok((
                LaneOutcome {
                    form_refs: Vec::new(),
                    expanded: 0,
                    maximum_scratch: 0,
                    unresolved: Some("query_symbol_budget"),
                },
                D1Work::default(),
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
                D1Work::default(),
            ));
        }
        let mut stack = vec![D1PackedNode {
            state_id: index.root_state,
            rank_prefix: 0,
            row: d1_initial(query.len(), radius),
        }];
        let mut form_refs = Vec::new();
        let mut expanded = 0_usize;
        let mut work = D1Work::default();
        if TRACE {
            work.stack_pushes = 1;
        }
        let mut maximum_scratch = d1_scratch_bytes(&stack, &form_refs);
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
            if state.terminal() && d1_terminal_distance(node.row, query.len(), radius) <= radius {
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
                    let alphabet_id = d1_alphabet_id(edge.symbol).ok_or_else(|| {
                        format!("D1 edge has unknown symbol U+{:04X}", edge.symbol)
                    })?;
                    let equality =
                        d1_equality_window(node.row, masks[alphabet_id], radius, query.len());
                    let next = d1_u1_advance(node.row, equality, radius, query.len());
                    if next.minimum <= radius {
                        if TRACE {
                            work.surviving_edges += 1;
                        }
                        children.push(D1PackedNode {
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
                    let alphabet_id = d1_alphabet_id(edge.symbol).ok_or_else(|| {
                        format!("D1 edge has unknown symbol U+{:04X}", edge.symbol)
                    })?;
                    let equality =
                        d1_equality_window(node.row, masks[alphabet_id], radius, query.len());
                    let next = d1_u1_advance(node.row, equality, radius, query.len());
                    if next.minimum <= radius {
                        if TRACE {
                            work.surviving_edges += 1;
                            work.stack_pushes += 1;
                        }
                        stack.push(D1PackedNode {
                            state_id: edge.target,
                            rank_prefix,
                            row: next.state,
                        });
                    } else if TRACE {
                        work.pruned_edges += 1;
                    }
                }
            }
            maximum_scratch = maximum_scratch.max(d1_scratch_bytes(&stack, &form_refs));
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
    fn d1_search<const TRACE: bool>(
        index: &V13DafsaView,
        package: &RuntimeL2Package,
        observed: &str,
        budget: SearchBudget,
        reverse_schedule: bool,
    ) -> Result<D1Observed, String> {
        let started = Instant::now();
        let oracle = Phase7dCertificateOracle::new(observed)?;
        let lanes = oracle.retrieval_lanes();
        let mut terminal_refs = Vec::new();
        let mut expanded = 0_usize;
        let mut maximum_scratch = 0_usize;
        let mut work = D1Work::default();
        let mut rank_prefixes = Vec::new();
        let mut terminal_ranks = Vec::new();
        for lane in lanes {
            let (outcome, lane_work) = d1_enumerate_lane::<TRACE>(
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
                return Ok(D1Observed {
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
        Ok(D1Observed {
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

    fn d1_e0_trace_lane(
        index: &V13DafsaView,
        lane: &Phase7dRetrievalLane,
        reverse_schedule: bool,
        rank_prefixes: &mut Vec<u32>,
        terminal_ranks: &mut Vec<u32>,
    ) -> Result<(LaneOutcome, D1Work), String> {
        let query = lane.symbols.as_ref();
        let radius = lane.maximum_levenshtein_distance;
        if query.len() > MAX_QUERY_SYMBOLS || radius > MAX_LEVENSHTEIN_RADIUS {
            return Err("D1 E0 trace received an out-of-contract lane".to_string());
        }
        let mut stack = vec![SearchNode {
            state_id: index.root_state,
            rank_prefix: 0,
            row: BandedLevenshteinRow::initial(query.len(), radius),
        }];
        let mut form_refs = Vec::new();
        let mut expanded = 0_usize;
        let mut work = D1Work {
            stack_pushes: 1,
            ..D1Work::default()
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

    fn d1_e0_trace(
        index: &V13DafsaView,
        observed: &str,
        reverse_schedule: bool,
    ) -> Result<(Vec<u32>, usize, usize, D1Work, Vec<u32>, Vec<u32>), String> {
        let oracle = Phase7dCertificateOracle::new(observed)?;
        let mut terminal_refs = Vec::new();
        let mut expanded = 0_usize;
        let mut maximum_scratch = 0_usize;
        let mut work = D1Work::default();
        let mut rank_prefixes = Vec::new();
        let mut terminal_ranks = Vec::new();
        for lane in oracle.retrieval_lanes() {
            let (outcome, lane_work) = d1_e0_trace_lane(
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

    fn d1_transition_stress() -> (usize, usize, usize) {
        let mut cases = 0_usize;
        let mut transition_mismatches = 0_usize;
        let mut packed_state_mismatches = 0_usize;
        for query_len in 23_usize..=96 {
            for radius in 0_u8..=3 {
                let query = vec![1_u32; query_len];
                let mut next_query = vec![2_u32; query_len];
                let mut previous = BandedLevenshteinRow::initial(query_len, radius);
                for _ in 0..=query_len + radius as usize {
                    let packed = d1_pack(&previous);
                    packed_state_mismatches += usize::from(d1_pack(&d1_unpack(packed)) != packed);
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
                        let fused = d1_u1_advance(packed, equality as u8, radius, query_len);
                        transition_mismatches += usize::from(
                            d1_pack(&generic) != fused.state
                                || generic.minimum(radius) != fused.minimum
                                || generic.terminal_distance(query_len, radius)
                                    != d1_terminal_distance(fused.state, query_len, radius),
                        );
                        cases += 1;
                    }
                    previous = previous.advance(&query, 1, radius);
                }
            }
        }
        (cases, transition_mismatches, packed_state_mismatches)
    }

    fn d1_focused_fixtures() -> Result<serde_json::Value, String> {
        let query = (0..96)
            .map(|index| {
                if index % 5 == 0 {
                    0x0451
                } else {
                    0x0430 + (index % 32) as u32
                }
            })
            .collect::<Vec<_>>();
        let masks = d1_eq_masks(&query);
        let symbol = 0x0451;
        let alphabet_id =
            d1_alphabet_id(symbol).ok_or_else(|| "fixture alphabet lookup".to_string())?;
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
                let packed = d1_pack(&row);
                let actual = d1_equality_window(packed, masks[alphabet_id], radius, query.len());
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
        let initial_packed = d1_initial(96, 3);
        let terminal_edge_mismatches = usize::from(d1_pack(&initial) != initial_packed)
            + usize::from(
                d1_terminal_distance(initial_packed, 0, 3) != initial.terminal_distance(0, 3),
            )
            + usize::from(
                d1_terminal_distance(initial_packed, 96, 3) != initial.terminal_distance(96, 3),
            );
        let stack = Vec::<D1PackedNode>::with_capacity(17);
        let terminals = Vec::<u32>::with_capacity(11);
        let expected_scratch =
            17 * std::mem::size_of::<D1PackedNode>() + 11 * std::mem::size_of::<u32>();
        let scratch_mismatches =
            usize::from(d1_scratch_bytes(&stack, &terminals) != expected_scratch);
        let unknown_symbol_rejected = d1_alphabet_id(u32::from('A')).is_none();
        Ok(serde_json::json!({
            "equality_window_cases": 400,
            "equality_window_mismatches": equality_mismatches,
            "crosses_position_64": true,
            "unknown_symbol_rejected": unknown_symbol_rejected,
            "terminal_edge_mismatches": terminal_edge_mismatches,
            "stack_budget_mismatches": scratch_mismatches,
            "packed_node_bytes": std::mem::size_of::<D1PackedNode>(),
            "pass": equality_mismatches == 0 && unknown_symbol_rejected
                && terminal_edge_mismatches == 0 && scratch_mismatches == 0,
        }))
    }

    #[cfg(target_os = "linux")]
    fn d1_pin_current_thread(cpu: usize) -> Result<(), String> {
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
    fn d1_pin_current_thread(_cpu: usize) -> Result<(), String> {
        Err("D1 requires Linux affinity".to_string())
    }

    fn d1_current_affinity() -> Result<Vec<usize>, String> {
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
            Err("D1 requires Linux affinity".to_string())
        }
    }

    fn d1_parse_cpus() -> Result<Vec<usize>, String> {
        let cpus = d1_required_string("LAY_V10_D1_CPUS")?
            .split(',')
            .map(|value| value.parse::<usize>().map_err(|error| error.to_string()))
            .collect::<Result<Vec<_>, _>>()?;
        if cpus.is_empty() || cpus.iter().copied().collect::<BTreeSet<_>>().len() != cpus.len() {
            return Err("D1 CPU list is empty or contains duplicates".to_string());
        }
        Ok(cpus)
    }

    fn d1_thread_migrations() -> Result<u64, String> {
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

    fn d1_sample(
        index: &V13DafsaView,
        package: &RuntimeL2Package,
        case: &ProofCase,
        query_ordinal: usize,
        round: usize,
        worker_id: u8,
    ) -> D1Sample {
        let outcome = d1_search::<false>(
            index,
            package,
            &case.damaged_surface,
            SearchBudget::proof(),
            false,
        );
        let sample = match &outcome {
            Ok(observed) => D1Sample {
                query_ordinal: query_ordinal as u16,
                round: round as u16,
                worker_id,
                flags: if observed.result.completeness == SearchCompleteness::CertifiedExhaustive {
                    0
                } else {
                    D1_FLAG_UNRESOLVED
                },
                search_elapsed_us: observed.result.search_elapsed_us,
                total_elapsed_us: observed.result.total_elapsed_us,
            },
            Err(_) => D1Sample {
                query_ordinal: query_ordinal as u16,
                round: round as u16,
                worker_id,
                flags: D1_FLAG_ERROR,
                search_elapsed_us: 0,
                total_elapsed_us: 0,
            },
        };
        std::hint::black_box(&outcome);
        drop(outcome);
        sample
    }

    fn d1_warmup(
        index: &V13DafsaView,
        package: &RuntimeL2Package,
        cases: &[ProofCase],
    ) -> Result<(), String> {
        for case in cases {
            let observed = d1_search::<false>(
                index,
                package,
                &case.damaged_surface,
                SearchBudget::proof(),
                false,
            )?;
            if observed.result.completeness != SearchCompleteness::CertifiedExhaustive {
                return Err("D1 warmup was unresolved".to_string());
            }
            std::hint::black_box(&observed.result);
        }
        Ok(())
    }

    fn d1_encode_samples(samples: &[D1Sample]) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(samples.len() * D1_SAMPLE_BYTES);
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

    fn d1_sample_summary(samples: &[D1Sample]) -> serde_json::Value {
        serde_json::json!({
            "records": samples.len(),
            "errors": samples.iter().filter(|sample| sample.flags & D1_FLAG_ERROR != 0).count(),
            "unresolved": samples.iter().filter(|sample| sample.flags & D1_FLAG_UNRESOLVED != 0).count(),
            "sample_bytes": D1_SAMPLE_BYTES,
            "schema": D1_SAMPLE_SCHEMA,
        })
    }

    fn d1_output_dir() -> Result<std::path::PathBuf, String> {
        let path = d1_required_path("LAY_V10_D1_OUTPUT")?;
        if !path.is_dir() {
            return Err(format!("D1 output directory is absent: {}", path.display()));
        }
        Ok(path)
    }

    fn d1_run_parity() -> Result<serde_json::Value, String> {
        let inputs = d1_load_inputs()?;
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
        let mut d1_maximum_scratch_bytes = 0_usize;
        let mut e0_work_total = D1Work::default();
        let mut d1_work_total = D1Work::default();
        let mut reverse_e0_work_total = D1Work::default();
        let mut reverse_d1_work_total = D1Work::default();

        for case in &inputs.cases {
            let e0 = search_typed_peaks(
                &inputs.index,
                &inputs.package,
                &case.damaged_surface,
                SearchBudget::proof(),
            )?;
            let e1 = d1_search::<true>(
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
            ) = d1_e0_trace(&inputs.index, &case.damaged_surface, false)?;
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
            d1_maximum_scratch_bytes =
                d1_maximum_scratch_bytes.max(e1.result.maximum_scratch_bytes);
            e0_work_total.add(e0_work);
            d1_work_total.add(e1.work);

            let d1_reverse = d1_search::<true>(
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
            ) = d1_e0_trace(&inputs.index, &case.damaged_surface, true)?;
            reverse_terminal_mismatches +=
                usize::from(reverse_e0_refs != d1_reverse.result.retrieved_form_refs)
                    + usize::from(e0.retrieved_form_refs != d1_reverse.result.retrieved_form_refs);
            reverse_peak_mismatches += usize::from(e0.peaks != d1_reverse.result.peaks);
            reverse_completeness_mismatches +=
                usize::from(e0.completeness != d1_reverse.result.completeness);
            reverse_work_mismatches +=
                usize::from(reverse_e0_expanded != d1_reverse.result.expanded_product_states)
                    + usize::from(reverse_e0_work != d1_reverse.work);
            reverse_rank_prefix_mismatches +=
                usize::from(reverse_e0_ranks != d1_reverse.rank_prefixes);
            reverse_terminal_rank_mismatches +=
                usize::from(reverse_e0_terminals != d1_reverse.terminal_ranks);
            reverse_e0_work_total.add(reverse_e0_work);
            reverse_d1_work_total.add(d1_reverse.work);

            let frozen_refs = e1
                .result
                .peaks
                .iter()
                .map(|peak| peak.form_ref)
                .collect::<BTreeSet<_>>();
            let target_ref = inputs
                .package
                .form_ref_for_surface(&case.target_surface)
                .ok_or_else(|| format!("D1 V13 lacks target {}", case.target_surface))?;
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
                    .ok_or_else(|| format!("D1 missing emitted form {}", peak.form_ref))?;
                let verified = crate::nanda_wave::lexical_grokking::phase7d_certificate_keys(
                    &case.damaged_surface,
                    surface.as_ref(),
                )?;
                false_certificates +=
                    usize::from(verified.is_empty() || verified != peak.certificate_keys);
            }
        }
        let (stress_cases, stress_transition_mismatches, stress_packed_state_mismatches) =
            d1_transition_stress();
        let fixtures = d1_focused_fixtures()?;
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
            && d1_maximum_scratch_bytes <= 6_656
            && e0_work_total == d1_work_total
            && reverse_e0_work_total == reverse_d1_work_total
            && e0_work_total.expanded_states == D1_EXPECTED_EXPANDED
            && e0_work_total.examined_edges == D1_EXPECTED_TRANSITIONS
            && stress_cases == 714_026
            && stress_transition_mismatches == 0
            && stress_packed_state_mismatches == 0
            && fixtures.get("pass").and_then(serde_json::Value::as_bool) == Some(true);
        Ok(serde_json::json!({
            "schema": "lay.v10.e1-semantic-parity.v1",
            "verdict": if pass { "PASS" } else { "FAIL" },
            "test": D1_PARITY_TEST,
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
            "d1_maximum_scratch_bytes": d1_maximum_scratch_bytes,
            "e0_work": {
                "expanded_states": e0_work_total.expanded_states,
                "examined_edges": e0_work_total.examined_edges,
                "surviving_edges": e0_work_total.surviving_edges,
                "pruned_edges": e0_work_total.pruned_edges,
                "stack_pushes": e0_work_total.stack_pushes,
                "stack_pops": e0_work_total.stack_pops,
                "terminal_hits": e0_work_total.terminal_hits,
            },
            "d1_work": {
                "expanded_states": d1_work_total.expanded_states,
                "examined_edges": d1_work_total.examined_edges,
                "surviving_edges": d1_work_total.surviving_edges,
                "pruned_edges": d1_work_total.pruned_edges,
                "stack_pushes": d1_work_total.stack_pushes,
                "stack_pops": d1_work_total.stack_pops,
                "terminal_hits": d1_work_total.terminal_hits,
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

    fn d1_run_single() -> Result<serde_json::Value, String> {
        let inputs = d1_load_inputs()?;
        let cpus = d1_parse_cpus()?;
        if cpus.len() != 1 {
            return Err("D1 S requires exactly one CPU".to_string());
        }
        d1_pin_current_thread(cpus[0])?;
        if d1_current_affinity()? != cpus {
            return Err("D1 S affinity mismatch".to_string());
        }
        let migrations_before = d1_thread_migrations()?;
        d1_warmup(&inputs.index, &inputs.package, &inputs.cases)?;
        let expected = inputs.cases.len() * D1_SINGLE_ROUNDS;
        let mut samples = Vec::with_capacity(expected);
        let capacity = samples.capacity();
        for round in 0..D1_SINGLE_ROUNDS {
            for (query_ordinal, case) in inputs.cases.iter().enumerate() {
                samples.push(d1_sample(
                    &inputs.index,
                    &inputs.package,
                    case,
                    query_ordinal,
                    round,
                    D1_SINGLE_WORKER_SENTINEL,
                ));
            }
        }
        if samples.len() != expected || samples.capacity() != capacity {
            return Err("D1 S sample capacity changed".to_string());
        }
        let migrations_after = d1_thread_migrations()?;
        let output = d1_output_dir()?;
        let raw = d1_encode_samples(&samples);
        d1_write_new(&output.join("samples.bin"), &raw)?;
        Ok(serde_json::json!({
            "schema": "lay.v10.e1-single-process.v1",
            "test": D1_SINGLE_TEST,
            "run_id": d1_required_string("LAY_V10_D1_RUN_ID")?,
            "rounds": D1_SINGLE_ROUNDS,
            "queries": inputs.cases.len(),
            "warmup_queries": inputs.cases.len(),
            "cpus": cpus,
            "sample_capacity": capacity,
            "thread_migrations_before": migrations_before,
            "thread_migrations_after": migrations_after,
            "thread_migrations_delta": migrations_after.saturating_sub(migrations_before),
            "samples_sha256": d1_sha256_text(&raw),
            "samples": d1_sample_summary(&samples),
            "production_timing_authority": ["search_elapsed_us", "total_elapsed_us"],
            "external_call_wall_authority": false,
            "runtime_authority_changed": false,
            "installed_lay_changed": false,
        }))
    }

    fn d1_run_twenty() -> Result<serde_json::Value, String> {
        let inputs = d1_load_inputs()?;
        let cpus = d1_parse_cpus()?;
        if cpus.len() != D1_WORKERS {
            return Err("D1 T requires exactly twenty CPUs".to_string());
        }
        let start = std::sync::Arc::new(std::sync::Barrier::new(D1_WORKERS + 1));
        let end = std::sync::Arc::new(std::sync::Barrier::new(D1_WORKERS + 1));
        let worker_outputs = std::thread::scope(|scope| {
            let handles = (0..D1_WORKERS)
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
                        move || -> Result<(usize, Vec<D1Sample>, Vec<usize>, u64), String> {
                            let mut failure = d1_pin_current_thread(cpu).err();
                            let affinity = d1_current_affinity().unwrap_or_default();
                            if failure.is_none() && affinity != [cpu] {
                                failure = Some(format!("D1 T worker {worker} affinity mismatch"));
                            }
                            let migrations_before = d1_thread_migrations().unwrap_or_default();
                            let expected = cases.len() * D1_TWENTY_ROUNDS;
                            let mut samples = Vec::with_capacity(expected);
                            let capacity = samples.capacity();
                            for burst in 0..=D1_TWENTY_ROUNDS {
                                start.wait();
                                if failure.is_none() {
                                    for (offset, case) in cases.iter().enumerate() {
                                        if burst == 0 {
                                            match d1_search::<false>(
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
                                                    "D1 T worker {worker} warmup unresolved"
                                                ));
                                                break;
                                            }
                                            Err(error) => {
                                                failure = Some(format!(
                                                    "D1 T worker {worker} warmup: {error}"
                                                ));
                                                break;
                                            }
                                        }
                                        } else {
                                            samples.push(d1_sample(
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
                            let migrations_after = d1_thread_migrations()?;
                            if samples.len() != expected || samples.capacity() != capacity {
                                return Err(format!("D1 T worker {worker} capacity changed"));
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
            for _ in 0..=D1_TWENTY_ROUNDS {
                start.wait();
                end.wait();
            }
            handles
                .into_iter()
                .map(|handle| {
                    handle
                        .join()
                        .map_err(|_| "D1 T worker panicked".to_string())?
                })
                .collect::<Result<Vec<_>, String>>()
        })?;

        let mut samples = Vec::with_capacity(inputs.cases.len() * D1_TWENTY_ROUNDS);
        let mut capacities = Vec::with_capacity(D1_WORKERS);
        let mut affinities = Vec::with_capacity(D1_WORKERS);
        let mut migration_deltas = Vec::with_capacity(D1_WORKERS);
        for (capacity, mut worker_samples, affinity, migrations) in worker_outputs {
            capacities.push(capacity);
            affinities.push(affinity);
            migration_deltas.push(migrations);
            samples.append(&mut worker_samples);
        }
        if samples.len() != inputs.cases.len() * D1_TWENTY_ROUNDS {
            return Err("D1 T pooled sample count mismatch".to_string());
        }
        let output = d1_output_dir()?;
        let raw = d1_encode_samples(&samples);
        d1_write_new(&output.join("samples.bin"), &raw)?;
        Ok(serde_json::json!({
            "schema": "lay.v10.e1-twenty-worker-process.v1",
            "test": D1_TWENTY_TEST,
            "run_id": d1_required_string("LAY_V10_D1_RUN_ID")?,
            "rounds": D1_TWENTY_ROUNDS,
            "queries": inputs.cases.len(),
            "workers": D1_WORKERS,
            "chunk_size": 20,
            "warmup_bursts": 1,
            "start_barriers": D1_TWENTY_ROUNDS + 1,
            "end_barriers": D1_TWENTY_ROUNDS + 1,
            "cpus": cpus,
            "worker_affinities": affinities,
            "worker_capacities": capacities,
            "worker_migration_deltas": migration_deltas,
            "samples_sha256": d1_sha256_text(&raw),
            "samples": d1_sample_summary(&samples),
            "production_timing_authority": ["search_elapsed_us", "total_elapsed_us"],
            "external_call_wall_authority": false,
            "runtime_authority_changed": false,
            "installed_lay_changed": false,
        }))
    }

    const D1_COMPONENT_SINGLE_TEST: &str =
        "nanda_wave::l2_field::v13_typed_peak::tests::v10_d1_component_single";
    const D1_COMPONENT_TWENTY_TEST: &str =
        "nanda_wave::l2_field::v13_typed_peak::tests::v10_d1_component_twenty";
    const D1_TWENTY_PMU_TEST: &str =
        "nanda_wave::l2_field::v13_typed_peak::tests::v10_d1_twenty_pmu";
    const D1_DIAGNOSTIC_ROUNDS: usize = 20;
    const D1_COMPONENT_PHASES: usize = 6;
    const D1_COMPONENT_SAMPLE_BYTES: usize = 118;

    #[derive(Clone, Copy, Debug, Default)]
    struct D1PhaseTiming {
        wall_ns: u64,
        thread_cpu_ns: u64,
    }

    impl D1PhaseTiming {
        fn add(&mut self, wall_ns: u64, thread_cpu_ns: u64) {
            self.wall_ns = self.wall_ns.saturating_add(wall_ns);
            self.thread_cpu_ns = self.thread_cpu_ns.saturating_add(thread_cpu_ns);
        }
    }

    #[derive(Clone, Copy, Debug, Default)]
    struct D1ComponentTiming {
        outer_wall_ns: u64,
        outer_thread_cpu_ns: u64,
        phases: [D1PhaseTiming; D1_COMPONENT_PHASES],
    }

    #[derive(Clone, Copy, Debug, Default)]
    struct D1ComponentSample {
        query_ordinal: u16,
        round: u16,
        worker_id: u8,
        flags: u8,
        timing: D1ComponentTiming,
    }

    struct D1ComponentObserved {
        result: SearchResult,
        timing: D1ComponentTiming,
    }

    #[cfg(target_os = "linux")]
    fn d1_thread_cpu_ns() -> Result<u64, String> {
        unsafe {
            let mut value: libc::timespec = std::mem::zeroed();
            if libc::clock_gettime(libc::CLOCK_THREAD_CPUTIME_ID, &mut value) != 0 {
                return Err(format!(
                    "CLOCK_THREAD_CPUTIME_ID failed: {}",
                    std::io::Error::last_os_error()
                ));
            }
            let seconds = u64::try_from(value.tv_sec)
                .map_err(|_| "negative thread CPU seconds".to_string())?;
            let nanoseconds = u64::try_from(value.tv_nsec)
                .map_err(|_| "negative thread CPU nanoseconds".to_string())?;
            Ok(seconds
                .saturating_mul(1_000_000_000)
                .saturating_add(nanoseconds))
        }
    }

    #[cfg(not(target_os = "linux"))]
    fn d1_thread_cpu_ns() -> Result<u64, String> {
        Err("D1 component timing requires Linux thread CPU clocks".to_string())
    }

    fn d1_measure_phase<T>(
        timing: &mut D1PhaseTiming,
        operation: impl FnOnce() -> Result<T, String>,
    ) -> Result<T, String> {
        let cpu_started = d1_thread_cpu_ns()?;
        let wall_started = Instant::now();
        let result = operation();
        let wall_ns = u64::try_from(wall_started.elapsed().as_nanos()).unwrap_or(u64::MAX);
        let cpu_ns = d1_thread_cpu_ns()?.saturating_sub(cpu_started);
        timing.add(wall_ns, cpu_ns);
        result
    }

    fn d1_component_search(
        index: &V13DafsaView,
        package: &RuntimeL2Package,
        observed: &str,
        budget: SearchBudget,
    ) -> Result<D1ComponentObserved, String> {
        let outer_cpu_started = d1_thread_cpu_ns()?;
        let outer_wall_started = Instant::now();
        let mut timing = D1ComponentTiming::default();
        let oracle = d1_measure_phase(&mut timing.phases[0], || {
            Phase7dCertificateOracle::new(observed)
        })?;
        let lanes = d1_measure_phase(&mut timing.phases[1], || {
            Ok(oracle.retrieval_lanes())
        })?;
        let mut terminal_refs = Vec::new();
        let mut expanded = 0_usize;
        let mut maximum_scratch = 0_usize;
        let mut rank_prefixes = Vec::new();
        let mut terminal_ranks = Vec::new();
        for lane in lanes {
            let masks = d1_measure_phase(&mut timing.phases[2], || {
                Ok(d1_eq_masks(lane.symbols.as_ref()))
            })?;
            let (outcome, _) = d1_measure_phase(&mut timing.phases[3], || {
                d1_enumerate_lane_prepared::<false>(
                    index,
                    &lane,
                    &masks,
                    budget,
                    outer_wall_started,
                    false,
                    &mut rank_prefixes,
                    &mut terminal_ranks,
                )
            })?;
            expanded = expanded.saturating_add(outcome.expanded);
            maximum_scratch = maximum_scratch.max(outcome.maximum_scratch);
            if let Some(reason) = outcome.unresolved {
                return Err(format!("D1 component unresolved: {reason}"));
            }
            terminal_refs.extend(outcome.form_refs);
        }
        d1_measure_phase(&mut timing.phases[4], || {
            terminal_refs.sort_unstable();
            terminal_refs.dedup();
            Ok(())
        })?;
        let retrieved_form_refs = terminal_refs.clone();
        let peaks = d1_measure_phase(&mut timing.phases[5], || {
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
            Ok(peaks)
        })?;
        let result = SearchResult {
            retrieved_form_refs,
            peaks,
            completeness: SearchCompleteness::CertifiedExhaustive,
            expanded_product_states: expanded,
            maximum_scratch_bytes: maximum_scratch,
            search_elapsed_us: 0,
            material_elapsed_us: 0,
            total_elapsed_us: 0,
        };
        timing.outer_wall_ns =
            u64::try_from(outer_wall_started.elapsed().as_nanos()).unwrap_or(u64::MAX);
        timing.outer_thread_cpu_ns = d1_thread_cpu_ns()?.saturating_sub(outer_cpu_started);
        Ok(D1ComponentObserved { result, timing })
    }

    fn d1_component_sample(
        index: &V13DafsaView,
        package: &RuntimeL2Package,
        case: &ProofCase,
        query_ordinal: usize,
        round: usize,
        worker_id: u8,
    ) -> D1ComponentSample {
        match d1_component_search(
            index,
            package,
            &case.damaged_surface,
            SearchBudget::proof(),
        ) {
            Ok(observed) => {
                std::hint::black_box(&observed.result);
                D1ComponentSample {
                    query_ordinal: query_ordinal as u16,
                    round: round as u16,
                    worker_id,
                    flags: u8::from(
                        observed.result.completeness
                            != SearchCompleteness::CertifiedExhaustive,
                    ) * D1_FLAG_UNRESOLVED,
                    timing: observed.timing,
                }
            }
            Err(_) => D1ComponentSample {
                query_ordinal: query_ordinal as u16,
                round: round as u16,
                worker_id,
                flags: D1_FLAG_ERROR,
                timing: D1ComponentTiming::default(),
            },
        }
    }

    fn d1_encode_component_samples(samples: &[D1ComponentSample]) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(samples.len() * D1_COMPONENT_SAMPLE_BYTES);
        for sample in samples {
            bytes.extend_from_slice(&sample.query_ordinal.to_le_bytes());
            bytes.extend_from_slice(&sample.round.to_le_bytes());
            bytes.push(sample.worker_id);
            bytes.push(sample.flags);
            bytes.extend_from_slice(&sample.timing.outer_wall_ns.to_le_bytes());
            bytes.extend_from_slice(&sample.timing.outer_thread_cpu_ns.to_le_bytes());
            for phase in sample.timing.phases {
                bytes.extend_from_slice(&phase.wall_ns.to_le_bytes());
                bytes.extend_from_slice(&phase.thread_cpu_ns.to_le_bytes());
            }
        }
        bytes
    }

    fn d1_structure(
        index: &V13DafsaView,
        package: &RuntimeL2Package,
        case: &ProofCase,
        query_ordinal: usize,
    ) -> Result<serde_json::Value, String> {
        let oracle = Phase7dCertificateOracle::new(&case.damaged_surface)?;
        let lanes = oracle.retrieval_lanes();
        let retrieval_lanes = lanes.len();
        let mut terminal_refs = Vec::new();
        let mut work = D1Work::default();
        let mut expanded = 0_usize;
        let mut maximum_scratch = 0_usize;
        let mut rank_prefixes = Vec::new();
        let mut terminal_ranks = Vec::new();
        for lane in lanes {
            let (outcome, lane_work) = d1_enumerate_lane::<true>(
                index,
                &lane,
                SearchBudget::proof(),
                Instant::now(),
                false,
                &mut rank_prefixes,
                &mut terminal_ranks,
            )?;
            if let Some(reason) = outcome.unresolved {
                return Err(format!("D1 structural trace unresolved: {reason}"));
            }
            work.add(lane_work);
            expanded = expanded.saturating_add(outcome.expanded);
            maximum_scratch = maximum_scratch.max(outcome.maximum_scratch);
            terminal_refs.extend(outcome.form_refs);
        }
        terminal_refs.sort_unstable();
        terminal_refs.dedup();
        let certificate_calls = terminal_refs.len();
        let mut peaks = 0_usize;
        for form_ref in &terminal_refs {
            let surface = package
                .surface(*form_ref as usize)
                .ok_or_else(|| format!("V13 terminal rank {form_ref} cannot be decoded"))?;
            peaks += usize::from(!oracle.certificate_keys(surface.as_ref())?.is_empty());
        }
        Ok(serde_json::json!({
            "query_ordinal": query_ordinal,
            "retrieval_lanes": retrieval_lanes,
            "eqmask_builds": retrieval_lanes,
            "expanded_states": expanded,
            "examined_edges": work.examined_edges,
            "surviving_edges": work.surviving_edges,
            "pruned_edges": work.pruned_edges,
            "stack_pushes": work.stack_pushes,
            "stack_pops": work.stack_pops,
            "terminal_hits_before_merge": work.terminal_hits,
            "terminal_refs_after_merge": terminal_refs.len(),
            "certificate_calls": certificate_calls,
            "materialized_peaks": peaks,
            "maximum_scratch_bytes": maximum_scratch,
        }))
    }

    fn d1_structures(inputs: &D1Inputs) -> Result<Vec<serde_json::Value>, String> {
        inputs
            .cases
            .iter()
            .enumerate()
            .map(|(ordinal, case)| d1_structure(&inputs.index, &inputs.package, case, ordinal))
            .collect()
    }

    fn d1_component_summary(samples: &[D1ComponentSample]) -> serde_json::Value {
        serde_json::json!({
            "schema": "lay.v10.e1-remaining-cost-d1-component-samples.v1",
            "sample_bytes": D1_COMPONENT_SAMPLE_BYTES,
            "samples": samples.len(),
            "errors": samples.iter().filter(|sample| sample.flags & D1_FLAG_ERROR != 0).count(),
            "unresolved": samples.iter().filter(|sample| sample.flags & D1_FLAG_UNRESOLVED != 0).count(),
            "phase_order": ["oracle", "lanes", "eqmask", "traversal", "merge", "certificate"],
            "timing_units": "nanoseconds",
            "wall_clock": "monotonic Instant",
            "thread_cpu_clock": "CLOCK_THREAD_CPUTIME_ID",
        })
    }

    fn d1_write_component_outputs(
        output: &Path,
        samples: &[D1ComponentSample],
        structures: &[serde_json::Value],
    ) -> Result<String, String> {
        let raw = d1_encode_component_samples(samples);
        d1_write_new(&output.join("component-samples.bin"), &raw)?;
        d1_write_json(
            &output.join("structure.json"),
            &serde_json::json!({
                "schema": "lay.v10.e1-remaining-cost-d1-structure.v1",
                "queries": structures,
            }),
        )?;
        Ok(d1_sha256_text(&raw))
    }

    fn d1_validate_mapping(run_id: &str, cpus: &[usize]) -> Result<&'static str, String> {
        let fixed = (0..D1_WORKERS).collect::<Vec<_>>();
        let reversed = (0..D1_WORKERS).rev().collect::<Vec<_>>();
        if run_id.contains("FIXED") && cpus == fixed {
            Ok("FIXED")
        } else if run_id.contains("REVERSED") && cpus == reversed {
            Ok("REVERSED")
        } else {
            Err(format!("D1 mapping mismatch for {run_id}: {cpus:?}"))
        }
    }

    fn d1_warmup_case(
        index: &V13DafsaView,
        package: &RuntimeL2Package,
        case: &ProofCase,
    ) -> Result<(), String> {
        let observed = d1_search::<false>(
            index,
            package,
            &case.damaged_surface,
            SearchBudget::proof(),
            false,
        )?;
        if observed.result.completeness != SearchCompleteness::CertifiedExhaustive {
            return Err("D1 warmup unresolved".to_string());
        }
        std::hint::black_box(&observed.result);
        Ok(())
    }

    fn d1_run_component_single() -> Result<serde_json::Value, String> {
        let inputs = d1_load_inputs()?;
        let cpus = d1_parse_cpus()?;
        if cpus != [0] {
            return Err(format!("D1 C-SINGLE requires CPU 0: {cpus:?}"));
        }
        d1_pin_current_thread(0)?;
        if d1_current_affinity()? != cpus {
            return Err("D1 C-SINGLE affinity mismatch".to_string());
        }
        for case in &inputs.cases {
            d1_warmup_case(&inputs.index, &inputs.package, case)?;
        }
        let structures = d1_structures(&inputs)?;
        let migrations_before = d1_thread_migrations()?;
        let expected = inputs.cases.len() * D1_DIAGNOSTIC_ROUNDS;
        let mut samples = Vec::with_capacity(expected);
        let capacity = samples.capacity();
        for round in 0..D1_DIAGNOSTIC_ROUNDS {
            for (ordinal, case) in inputs.cases.iter().enumerate() {
                samples.push(d1_component_sample(
                    &inputs.index,
                    &inputs.package,
                    case,
                    ordinal,
                    round,
                    D1_SINGLE_WORKER_SENTINEL,
                ));
            }
        }
        let migrations = d1_thread_migrations()?.saturating_sub(migrations_before);
        if samples.len() != expected || samples.capacity() != capacity {
            return Err("D1 C-SINGLE sample capacity changed".to_string());
        }
        let output = d1_output_dir()?;
        let samples_sha256 = d1_write_component_outputs(&output, &samples, &structures)?;
        Ok(serde_json::json!({
            "schema": "lay.v10.e1-remaining-cost-d1-component-process.v1",
            "test": D1_COMPONENT_SINGLE_TEST,
            "run_id": d1_required_string("LAY_V10_D1_RUN_ID")?,
            "mapping": "SINGLE",
            "rounds": D1_DIAGNOSTIC_ROUNDS,
            "queries": inputs.cases.len(),
            "workers": 1,
            "warmup_queries": inputs.cases.len(),
            "cpus": cpus,
            "thread_migrations": migrations,
            "samples_sha256": samples_sha256,
            "samples": d1_component_summary(&samples),
            "structural_trace_measured": false,
            "runtime_authority_changed": false,
            "installed_lay_changed": false,
        }))
    }

    fn d1_run_component_twenty() -> Result<serde_json::Value, String> {
        let inputs = d1_load_inputs()?;
        let cpus = d1_parse_cpus()?;
        if cpus.len() != D1_WORKERS {
            return Err("D1 component T requires twenty CPUs".to_string());
        }
        let run_id = d1_required_string("LAY_V10_D1_RUN_ID")?;
        let mapping = d1_validate_mapping(&run_id, &cpus)?;
        let structures = d1_structures(&inputs)?;
        let start = std::sync::Arc::new(std::sync::Barrier::new(D1_WORKERS + 1));
        let end = std::sync::Arc::new(std::sync::Barrier::new(D1_WORKERS + 1));
        let worker_outputs = std::thread::scope(|scope| {
            let handles = (0..D1_WORKERS)
                .map(|worker| {
                    let start = std::sync::Arc::clone(&start);
                    let end = std::sync::Arc::clone(&end);
                    let cpu = cpus[worker];
                    let chunk_start = worker * 20;
                    let chunk_end = (chunk_start + 20).min(inputs.cases.len());
                    let cases = &inputs.cases[chunk_start..chunk_end];
                    let index = &inputs.index;
                    let package = &inputs.package;
                    scope.spawn(move || -> Result<(Vec<D1ComponentSample>, Vec<usize>, u64), String> {
                        d1_pin_current_thread(cpu)?;
                        let affinity = d1_current_affinity()?;
                        if affinity != [cpu] {
                            return Err(format!("D1 component worker {worker} affinity mismatch"));
                        }
                        let migrations_before = d1_thread_migrations()?;
                        let expected = cases.len() * D1_DIAGNOSTIC_ROUNDS;
                        let mut samples = Vec::with_capacity(expected);
                        let capacity = samples.capacity();
                        for burst in 0..=D1_DIAGNOSTIC_ROUNDS {
                            start.wait();
                            for (offset, case) in cases.iter().enumerate() {
                                if burst == 0 {
                                    d1_warmup_case(index, package, case)?;
                                } else {
                                    samples.push(d1_component_sample(
                                        index,
                                        package,
                                        case,
                                        chunk_start + offset,
                                        burst - 1,
                                        worker as u8,
                                    ));
                                }
                            }
                            end.wait();
                        }
                        if samples.len() != expected || samples.capacity() != capacity {
                            return Err(format!("D1 component worker {worker} capacity changed"));
                        }
                        Ok((
                            samples,
                            affinity,
                            d1_thread_migrations()?.saturating_sub(migrations_before),
                        ))
                    })
                })
                .collect::<Vec<_>>();
            for _ in 0..=D1_DIAGNOSTIC_ROUNDS {
                start.wait();
                end.wait();
            }
            handles
                .into_iter()
                .map(|handle| {
                    handle
                        .join()
                        .map_err(|_| "D1 component worker panicked".to_string())?
                })
                .collect::<Result<Vec<_>, String>>()
        })?;
        let mut samples = Vec::with_capacity(inputs.cases.len() * D1_DIAGNOSTIC_ROUNDS);
        let mut affinities = Vec::with_capacity(D1_WORKERS);
        let mut migrations = Vec::with_capacity(D1_WORKERS);
        for (mut worker_samples, affinity, worker_migrations) in worker_outputs {
            samples.append(&mut worker_samples);
            affinities.push(affinity);
            migrations.push(worker_migrations);
        }
        if samples.len() != inputs.cases.len() * D1_DIAGNOSTIC_ROUNDS {
            return Err("D1 component pooled sample count mismatch".to_string());
        }
        let output = d1_output_dir()?;
        let samples_sha256 = d1_write_component_outputs(&output, &samples, &structures)?;
        Ok(serde_json::json!({
            "schema": "lay.v10.e1-remaining-cost-d1-component-process.v1",
            "test": D1_COMPONENT_TWENTY_TEST,
            "run_id": run_id,
            "mapping": mapping,
            "rounds": D1_DIAGNOSTIC_ROUNDS,
            "queries": inputs.cases.len(),
            "workers": D1_WORKERS,
            "warmup_bursts": 1,
            "start_barriers": D1_DIAGNOSTIC_ROUNDS + 1,
            "end_barriers": D1_DIAGNOSTIC_ROUNDS + 1,
            "cpus": cpus,
            "worker_affinities": affinities,
            "worker_migration_deltas": migrations,
            "samples_sha256": samples_sha256,
            "samples": d1_component_summary(&samples),
            "structural_trace_measured": false,
            "runtime_authority_changed": false,
            "installed_lay_changed": false,
        }))
    }

    fn d1_run_twenty_pmu() -> Result<serde_json::Value, String> {
        let inputs = d1_load_inputs()?;
        let cpus = d1_parse_cpus()?;
        if cpus.len() != D1_WORKERS {
            return Err("D1 PMU requires twenty CPUs".to_string());
        }
        let run_id = d1_required_string("LAY_V10_D1_RUN_ID")?;
        let mapping = d1_validate_mapping(&run_id, &cpus)?;
        let control = d1_required_path("LAY_V10_D1_CONTROL_DIR")?;
        if !control.is_dir() {
            return Err(format!("D1 control directory absent: {}", control.display()));
        }
        let start = std::sync::Arc::new(std::sync::Barrier::new(D1_WORKERS + 1));
        let end = std::sync::Arc::new(std::sync::Barrier::new(D1_WORKERS + 1));
        let worker_outputs = std::thread::scope(|scope| {
            let handles = (0..D1_WORKERS)
                .map(|worker| {
                    let start = std::sync::Arc::clone(&start);
                    let end = std::sync::Arc::clone(&end);
                    let cpu = cpus[worker];
                    let chunk_start = worker * 20;
                    let chunk_end = (chunk_start + 20).min(inputs.cases.len());
                    let cases = &inputs.cases[chunk_start..chunk_end];
                    let index = &inputs.index;
                    let package = &inputs.package;
                    scope.spawn(move || -> Result<(u64, usize, usize, Vec<usize>, u64), String> {
                        d1_pin_current_thread(cpu)?;
                        let affinity = d1_current_affinity()?;
                        if affinity != [cpu] {
                            return Err(format!("D1 PMU worker {worker} affinity mismatch"));
                        }
                        let migrations_before = d1_thread_migrations()?;
                        let mut checksum = 0xcbf29ce484222325_u64 ^ worker as u64;
                        let mut errors = 0_usize;
                        let mut unresolved = 0_usize;
                        for burst in 0..=D1_DIAGNOSTIC_ROUNDS {
                            start.wait();
                            for case in cases {
                                match d1_search::<false>(
                                    index,
                                    package,
                                    &case.damaged_surface,
                                    SearchBudget::proof(),
                                    false,
                                ) {
                                    Ok(observed) => {
                                        unresolved += usize::from(
                                            observed.result.completeness
                                                != SearchCompleteness::CertifiedExhaustive,
                                        );
                                        checksum = checksum.wrapping_mul(0x100000001b3)
                                            ^ observed.result.expanded_product_states as u64
                                            ^ ((observed.result.retrieved_form_refs.len() as u64) << 32)
                                            ^ observed.result.peaks.len() as u64;
                                        std::hint::black_box(&observed.result);
                                    }
                                    Err(_) => errors += 1,
                                }
                            }
                            end.wait();
                        }
                        Ok((
                            checksum,
                            errors,
                            unresolved,
                            affinity,
                            d1_thread_migrations()?.saturating_sub(migrations_before),
                        ))
                    })
                })
                .collect::<Vec<_>>();
            start.wait();
            end.wait();
            d1_write_new(&control.join("subject-ready"), b"ready\n")?;
            d1_wait_for(&control.join("controller-enabled"))?;
            let measured_started = Instant::now();
            for _ in 0..D1_DIAGNOSTIC_ROUNDS {
                start.wait();
                end.wait();
            }
            let measured_wall_ns = measured_started.elapsed().as_nanos();
            d1_write_new(&control.join("subject-done"), b"done\n")?;
            d1_wait_for(&control.join("controller-disabled"))?;
            let outputs = handles
                .into_iter()
                .map(|handle| {
                    handle
                        .join()
                        .map_err(|_| "D1 PMU worker panicked".to_string())?
                })
                .collect::<Result<Vec<_>, String>>()?;
            Ok::<_, String>((outputs, measured_wall_ns))
        })?;
        let (worker_outputs, measured_wall_ns) = worker_outputs;
        let checksum = worker_outputs.iter().fold(0_u64, |value, output| value ^ output.0);
        let errors = worker_outputs.iter().map(|output| output.1).sum::<usize>();
        let unresolved = worker_outputs.iter().map(|output| output.2).sum::<usize>();
        let affinities = worker_outputs
            .iter()
            .map(|output| output.3.clone())
            .collect::<Vec<_>>();
        let migrations = worker_outputs
            .iter()
            .map(|output| output.4)
            .collect::<Vec<_>>();
        if errors != 0 || unresolved != 0 {
            return Err(format!("D1 PMU errors={errors} unresolved={unresolved}"));
        }
        Ok(serde_json::json!({
            "schema": "lay.v10.e1-remaining-cost-d1-twenty-pmu.v1",
            "verdict": "PASS",
            "test": D1_TWENTY_PMU_TEST,
            "run_id": run_id,
            "mapping": mapping,
            "rounds": D1_DIAGNOSTIC_ROUNDS,
            "queries_per_round": inputs.cases.len(),
            "measured_requests": inputs.cases.len() * D1_DIAGNOSTIC_ROUNDS,
            "warmup_requests": inputs.cases.len(),
            "workers": D1_WORKERS,
            "cpus": cpus,
            "worker_affinities": affinities,
            "worker_migration_deltas": migrations,
            "checksum": checksum,
            "errors": errors,
            "unresolved": unresolved,
            "measured_wall_ns_diagnostic": measured_wall_ns,
            "examined_edges": D1_EXPECTED_TRANSITIONS * D1_DIAGNOSTIC_ROUNDS,
            "expanded_states": D1_EXPECTED_EXPANDED * D1_DIAGNOSTIC_ROUNDS,
            "control_protocol": ["subject-ready", "controller-enabled", "subject-done", "controller-disabled"],
            "component_clocks_enabled": false,
            "runtime_authority_changed": false,
            "installed_lay_changed": false,
        }))
    }

    fn d1_wait_for(path: &Path) -> Result<(), String> {
        let started = Instant::now();
        while !path.is_file() {
            if started.elapsed() > Duration::from_secs(120) {
                return Err(format!("timed out waiting for {}", path.display()));
            }
            std::thread::sleep(Duration::from_millis(1));
        }
        Ok(())
    }

    fn d1_run_physical(candidate: bool) -> Result<serde_json::Value, String> {
        let inputs = d1_load_inputs()?;
        let cpus = d1_parse_cpus()?;
        if cpus.len() != 1 {
            return Err("D1 physical route requires exactly one CPU".to_string());
        }
        d1_pin_current_thread(cpus[0])?;
        if d1_current_affinity()? != cpus {
            return Err("D1 physical affinity mismatch".to_string());
        }
        for case in &inputs.cases {
            if candidate {
                let observed = d1_search::<false>(
                    &inputs.index,
                    &inputs.package,
                    &case.damaged_surface,
                    SearchBudget::proof(),
                    false,
                )?;
                if observed.result.completeness != SearchCompleteness::CertifiedExhaustive {
                    return Err("D1 physical candidate warmup unresolved".to_string());
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
                    return Err("D1 physical E0 warmup unresolved".to_string());
                }
                std::hint::black_box(&result);
            }
        }
        let control = d1_required_path("LAY_V10_D1_CONTROL_DIR")?;
        if !control.is_dir() {
            return Err(format!(
                "D1 control directory is absent: {}",
                control.display()
            ));
        }
        let ready = control.join("subject-ready");
        let enabled = control.join("controller-enabled");
        let done = control.join("subject-done");
        let disabled = control.join("controller-disabled");
        d1_write_new(&ready, b"ready\n")?;
        d1_wait_for(&enabled)?;
        let measured_started = Instant::now();
        let mut checksum = 0xcbf29ce484222325_u64;
        let mut search_elapsed_sum_us = 0_u64;
        let mut total_elapsed_sum_us = 0_u64;
        let mut errors = 0_usize;
        let mut unresolved = 0_usize;
        for case in &inputs.cases {
            if candidate {
                match d1_search::<false>(
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
        d1_write_new(&done, b"done\n")?;
        d1_wait_for(&disabled)?;
        if errors != 0 || unresolved != 0 {
            return Err(format!(
                "D1 physical errors={errors} unresolved={unresolved}"
            ));
        }
        Ok(serde_json::json!({
            "schema": "lay.v10.exact-fused-executor-e1-physical.v1",
            "verdict": "PASS",
            "test": if candidate { D1_D1_PHYSICAL_TEST } else { D1_E0_PHYSICAL_TEST },
            "executor": if candidate { "D1" } else { "E0" },
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

    fn d1_subject(test: &str) {
        let output = d1_required_path("LAY_V10_D1_OUTPUT").expect("D1 output path");
        let result = match test {
            D1_PARITY_TEST => d1_run_parity(),
            D1_COMPONENT_SINGLE_TEST => d1_run_component_single(),
            D1_COMPONENT_TWENTY_TEST => d1_run_component_twenty(),
            D1_TWENTY_PMU_TEST => d1_run_twenty_pmu(),
            D1_SINGLE_TEST => d1_run_single(),
            D1_TWENTY_TEST => d1_run_twenty(),
            D1_E0_PHYSICAL_TEST => d1_run_physical(false),
            D1_D1_PHYSICAL_TEST => d1_run_physical(true),
            _ => Err(format!("unknown D1 test {test}")),
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
        d1_write_json(&output.join("SUBJECT_RECEIPT.json"), &receipt)
            .expect("write D1 subject receipt");
        assert!(
            receipt.get("error").is_none(),
            "{}",
            serde_json::to_string_pretty(&receipt).unwrap()
        );
        if test == D1_PARITY_TEST
            || test == D1_TWENTY_PMU_TEST
            || test == D1_E0_PHYSICAL_TEST
            || test == D1_D1_PHYSICAL_TEST
        {
            assert_eq!(
                receipt.get("verdict").and_then(serde_json::Value::as_str),
                Some("PASS")
            );
        }
    }

    #[test]
    #[ignore = "D1 separate semantic prerequisite; admitted only by D1 preflight"]
    fn v10_d1_semantic_parity() {
        d1_subject(D1_PARITY_TEST);
    }

    #[test]
    #[ignore = "D1 single component decomposition; admitted only by D1 preflight"]
    fn v10_d1_component_single() {
        d1_subject(D1_COMPONENT_SINGLE_TEST);
    }

    #[test]
    #[ignore = "D1 fixed or reversed component decomposition; admitted only by D1 preflight"]
    fn v10_d1_component_twenty() {
        d1_subject(D1_COMPONENT_TWENTY_TEST);
    }

    #[test]
    #[ignore = "D1 process-scoped twenty-worker PMU; admitted only by D1 preflight"]
    fn v10_d1_twenty_pmu() {
        d1_subject(D1_TWENTY_PMU_TEST);
    }

    #[test]
    #[ignore = "D1 direct single-process latency; admitted only after parity and clean admission"]
    fn v10_d1_single_latency() {
        d1_subject(D1_SINGLE_TEST);
    }

    #[test]
    #[ignore = "D1 twenty-worker fixed-shard burst; admitted only after parity and clean admission"]
    fn v10_d1_twenty_worker_latency() {
        d1_subject(D1_TWENTY_TEST);
    }

    #[test]
    #[ignore = "D1 exact V10 E0 full-executor PMU denominator"]
    fn v10_d1_e0_physical() {
        d1_subject(D1_E0_PHYSICAL_TEST);
    }

    #[test]
    #[ignore = "D1 packed full-executor PMU candidate"]
    fn v10_d1_d1_physical() {
        d1_subject(D1_D1_PHYSICAL_TEST);
    }
