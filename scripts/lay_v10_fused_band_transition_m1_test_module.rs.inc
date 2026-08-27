
    const M1_TEST: &str =
        "nanda_wave::l2_field::v13_typed_peak::tests::v10_exact_fused_band_transition_m1";
    const M1_SCHEDULE_SHA256: &str =
        "2f8346d97246ac3704434279739fa5ed27705465d628890a367c21875d879e78";
    const M1_SIDECAR_SHA256: &str =
        "a1aa95be8c43fae8fedf02d3261dd1265dcbfbc8d618330fcd6449ae829df8cd";
    const M1_EXPECTED_TRANSITIONS: usize = 25_145_756;
    const M1_EXPECTED_EXPANDED: usize = 8_059_788;

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    struct M1FusedOutput {
        state: u64,
        minimum: u8,
    }

    #[repr(C)]
    #[derive(Clone, Copy)]
    struct M1TraceEvent {
        previous: BandedLevenshteinRow,
        symbol: u32,
        packed_previous: u64,
        equality_window: u8,
        radius: u8,
        query_len: u8,
    }

    struct M1Trace {
        events: Vec<M1TraceEvent>,
        lanes: Vec<Box<[u32]>>,
        lane_ranges: Vec<std::ops::Range<usize>>,
        digest: String,
        records: usize,
        expanded_states: usize,
        terminal_mismatches: usize,
        peak_mismatches: usize,
        completeness_mismatches: usize,
        work_mismatches: usize,
        scratch_mismatches: usize,
        transition_mismatches: usize,
        packed_state_mismatches: usize,
        maximum_product_states: usize,
        maximum_scratch_bytes: usize,
    }

    struct M1TraceBuilder {
        events: Vec<M1TraceEvent>,
        lanes: Vec<Box<[u32]>>,
        lane_ranges: Vec<std::ops::Range<usize>>,
        digest: Sha256,
        expanded_states: usize,
        transition_mismatches: usize,
        packed_state_mismatches: usize,
    }

    struct M1ObservedSearch {
        retrieved_form_refs: Vec<u32>,
        peaks: Vec<V13TypedPeak>,
        completeness: SearchCompleteness,
        expanded_product_states: usize,
        maximum_scratch_bytes: usize,
    }

    fn m1_required_path(name: &str) -> Result<std::path::PathBuf, String> {
        std::env::var_os(name)
            .map(std::path::PathBuf::from)
            .ok_or_else(|| format!("missing required environment variable {name}"))
    }

    fn m1_digest(bytes: &[u8]) -> String {
        hex(Sha256::digest(bytes).into())
    }

    fn m1_write_new(path: &std::path::Path, bytes: &[u8]) -> Result<(), String> {
        if !path.parent().is_some_and(std::path::Path::is_dir) {
            return Err(format!(
                "output parent does not exist for {}",
                path.display()
            ));
        }
        let mut output = std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(path)
            .map_err(|error| format!("{}: {error}", path.display()))?;
        use std::io::Write;
        output.write_all(bytes).map_err(|error| error.to_string())?;
        output.sync_all().map_err(|error| error.to_string())
    }

    fn m1_write_json(path: &std::path::Path, value: &serde_json::Value) -> Result<(), String> {
        let mut bytes = serde_json::to_vec_pretty(value).map_err(|error| error.to_string())?;
        bytes.push(b'\n');
        m1_write_new(path, &bytes)
    }

    fn m1_wait_for(path: &std::path::Path) -> Result<(), String> {
        let deadline = Instant::now() + Duration::from_secs(120);
        while !path.is_file() {
            if Instant::now() >= deadline {
                return Err(format!("timeout waiting for {}", path.display()));
            }
            std::thread::sleep(Duration::from_millis(1));
        }
        Ok(())
    }

    fn m1_lanes_digest(lanes: &[Phase7dRetrievalLane]) -> Result<String, String> {
        let value = serde_json::Value::Array(
            lanes
                .iter()
                .map(|lane| {
                    serde_json::json!({
                        "symbols": lane.symbols.as_ref(),
                        "maximum_levenshtein_distance": lane.maximum_levenshtein_distance,
                    })
                })
                .collect(),
        );
        serde_json::to_vec(&value)
            .map(|bytes| m1_digest(&bytes))
            .map_err(|error| error.to_string())
    }

    fn m1_load_assets() -> Result<(RuntimeL2Package, V13DafsaView, Vec<ProofCase>), String> {
        let package_path = m1_required_path("LAY_V10_M1_V13_PACKAGE")?;
        let sidecar_path = m1_required_path("LAY_V10_M1_SIDECAR")?;
        let v7_path = m1_required_path("LAY_V10_M1_V7")?;
        let schedule_path = m1_required_path("LAY_V10_M1_SCHEDULE")?;
        let package_bytes = std::fs::metadata(&package_path)
            .map_err(|error| format!("{}: {error}", package_path.display()))?
            .len();
        let package_sha256 = sha256_file(&package_path)?;
        if package_bytes != EXPECTED_V13_BYTES || hex(package_sha256) != EXPECTED_V13_SHA256 {
            return Err("V13 package identity mismatch".to_string());
        }
        let package = RuntimeL2Package::load(&package_path)?;
        let identity = V13Identity {
            package_sha256,
            package_bytes,
            form_count: u32::try_from(package.form_count())
                .map_err(|_| "V13 form count exceeds u32".to_string())?,
            binding_count: u32::try_from(package.binding_count())
                .map_err(|_| "V13 binding count exceeds u32".to_string())?,
        };
        if hex(sha256_file(&sidecar_path)?) != M1_SIDECAR_SHA256 {
            return Err("V10 sidecar identity mismatch".to_string());
        }
        let index = V13DafsaView::load(&sidecar_path, identity)?;
        if hex(sha256_file(&v7_path)?) != EXPECTED_V7_SHA256 {
            return Err("V7 denominator identity mismatch".to_string());
        }
        let v7: serde_json::Value = serde_json::from_slice(
            &std::fs::read(&v7_path).map_err(|error| format!("{}: {error}", v7_path.display()))?,
        )
        .map_err(|error| error.to_string())?;
        let cases = parse_v7_cases(&v7)?;
        if cases.len() != 382 {
            return Err(format!("expected 382 V7 cases, found {}", cases.len()));
        }
        if hex(sha256_file(&schedule_path)?) != M1_SCHEDULE_SHA256 {
            return Err("schedule identity mismatch".to_string());
        }
        let schedule: serde_json::Value = serde_json::from_slice(
            &std::fs::read(&schedule_path)
                .map_err(|error| format!("{}: {error}", schedule_path.display()))?,
        )
        .map_err(|error| error.to_string())?;
        let entries = schedule
            .get("entries")
            .and_then(serde_json::Value::as_array)
            .ok_or_else(|| "schedule entries missing".to_string())?;
        if entries.len() != cases.len() {
            return Err("schedule case count mismatch".to_string());
        }
        for (source_ordinal, (case, entry)) in cases.iter().zip(entries).enumerate() {
            if entry
                .get("source_ordinal")
                .and_then(serde_json::Value::as_u64)
                != Some(source_ordinal as u64)
            {
                return Err(format!("schedule ordinal mismatch at {source_ordinal}"));
            }
            let surface_digest = m1_digest(case.damaged_surface.as_bytes());
            if entry
                .get("damaged_surface_sha256")
                .and_then(serde_json::Value::as_str)
                != Some(surface_digest.as_str())
            {
                return Err(format!("schedule surface mismatch at {source_ordinal}"));
            }
            let oracle = Phase7dCertificateOracle::new(&case.damaged_surface)?;
            let lanes_digest = m1_lanes_digest(&oracle.retrieval_lanes())?;
            if entry
                .get("retrieval_lanes_sha256")
                .and_then(serde_json::Value::as_str)
                != Some(lanes_digest.as_str())
            {
                return Err(format!("schedule lanes mismatch at {source_ordinal}"));
            }
        }
        Ok((package, index, cases))
    }

    fn m1_pack(row: &BandedLevenshteinRow) -> u64 {
        let mut value = 0_u64;
        for (index, cell) in row.cells.iter().copied().enumerate() {
            value |= u64::from(cell) << (index * 3);
        }
        value
            | (u64::from(row.depth) << 21)
            | (u64::from(row.start) << 29)
            | (u64::from(row.len) << 37)
    }

    fn m1_unpack(value: u64) -> BandedLevenshteinRow {
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
    fn m1_packed_value(value: u64, column: usize, outside: u8) -> u8 {
        let start = ((value >> 29) & 0xff) as usize;
        let len = ((value >> 37) & 7) as usize;
        if column >= start && column < start + len {
            ((value >> ((column - start) * 3)) & 7) as u8
        } else {
            outside
        }
    }

    fn m1_equality_window(
        previous: &BandedLevenshteinRow,
        query: &[u32],
        symbol: u32,
        radius: u8,
    ) -> u8 {
        let depth = previous.depth.checked_add(1).expect("bounded query depth");
        let start = depth.saturating_sub(radius) as usize;
        let end = (depth as usize + radius as usize).min(query.len());
        let len = end.checked_sub(start).map_or(0, |width| width + 1);
        let mut equality = 0_u8;
        for index in 0..len {
            let column = start + index;
            if column > 0 && query[column - 1] == symbol {
                equality |= 1 << index;
            }
        }
        equality
    }

    fn m1_g1_advance(
        previous: &BandedLevenshteinRow,
        equality: u8,
        radius: u8,
        query_len: usize,
    ) -> BandedLevenshteinRow {
        let outside = radius.saturating_add(1);
        let depth = previous.depth.checked_add(1).expect("bounded query depth");
        let start = depth.saturating_sub(radius) as usize;
        let end = (depth as usize + radius as usize).min(query_len);
        let len = end.checked_sub(start).map_or(0, |width| width + 1);
        let mut cells = [outside; MAX_BAND_CELLS];
        for index in 0..len {
            let column = start + index;
            let value = if column == 0 {
                depth.min(outside)
            } else {
                let left = if index > 0 { cells[index - 1] } else { outside }
                    .saturating_add(1)
                    .min(outside);
                let above = previous
                    .value(column, outside)
                    .saturating_add(1)
                    .min(outside);
                let diagonal = previous
                    .value(column - 1, outside)
                    .saturating_add(u8::from(equality & (1 << index) == 0))
                    .min(outside);
                left.min(above).min(diagonal)
            };
            cells[index] = value;
        }
        BandedLevenshteinRow {
            cells,
            depth,
            start: start as u8,
            len: len as u8,
        }
    }

    #[inline(always)]
    fn m1_u1_cell<const INDEX: usize>(
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
            let above = m1_packed_value(previous, column, outside)
                .saturating_add(1)
                .min(outside);
            let diagonal = m1_packed_value(previous, column - 1, outside)
                .saturating_add(u8::from(equality & (1 << INDEX) == 0))
                .min(outside);
            left.min(above).min(diagonal)
        }
    }

    #[inline(always)]
    fn m1_u1_advance(previous: u64, equality: u8, radius: u8, query_len: usize) -> M1FusedOutput {
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
            return M1FusedOutput {
                state,
                minimum: outside,
            };
        }

        let c0 = m1_u1_cell::<0>(previous, equality, radius, depth, start, outside);
        let c1 = if len > 1 {
            m1_u1_cell::<1>(previous, equality, radius, depth, start, c0)
        } else {
            outside
        };
        let c2 = if len > 2 {
            m1_u1_cell::<2>(previous, equality, radius, depth, start, c1)
        } else {
            outside
        };
        let c3 = if len > 3 {
            m1_u1_cell::<3>(previous, equality, radius, depth, start, c2)
        } else {
            outside
        };
        let c4 = if len > 4 {
            m1_u1_cell::<4>(previous, equality, radius, depth, start, c3)
        } else {
            outside
        };
        let c5 = if len > 5 {
            m1_u1_cell::<5>(previous, equality, radius, depth, start, c4)
        } else {
            outside
        };
        let c6 = if len > 6 {
            m1_u1_cell::<6>(previous, equality, radius, depth, start, c5)
        } else {
            outside
        };

        let mut minimum = c0;
        if len > 1 {
            minimum = minimum.min(c1);
        }
        if len > 2 {
            minimum = minimum.min(c2);
        }
        if len > 3 {
            minimum = minimum.min(c3);
        }
        if len > 4 {
            minimum = minimum.min(c4);
        }
        if len > 5 {
            minimum = minimum.min(c5);
        }
        if len > 6 {
            minimum = minimum.min(c6);
        }

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
        M1FusedOutput { state, minimum }
    }

    impl M1TraceBuilder {
        fn new() -> Self {
            Self {
                events: Vec::with_capacity(M1_EXPECTED_TRANSITIONS),
                lanes: Vec::with_capacity(764),
                lane_ranges: Vec::with_capacity(764),
                digest: Sha256::new(),
                expanded_states: 0,
                transition_mismatches: 0,
                packed_state_mismatches: 0,
            }
        }

        fn start_lane(&mut self, query: &[u32], radius: u8) -> usize {
            let lane_id = self.lanes.len();
            self.lanes.push(query.to_vec().into_boxed_slice());
            self.digest.update(b"lane");
            self.digest.update((lane_id as u32).to_le_bytes());
            self.digest.update((query.len() as u32).to_le_bytes());
            self.digest.update([radius]);
            for symbol in query {
                self.digest.update(symbol.to_le_bytes());
            }
            lane_id
        }

        fn finish_lane(&mut self, start: usize) {
            self.lane_ranges.push(start..self.events.len());
        }

        fn push_event(&mut self, event: M1TraceEvent) {
            self.digest.update(event.packed_previous.to_le_bytes());
            self.digest.update(event.symbol.to_le_bytes());
            self.digest
                .update([event.equality_window, event.radius, event.query_len]);
            self.events.push(event);
        }
    }

    fn m1_enumerate_lane(
        index: &V13DafsaView,
        lane: &Phase7dRetrievalLane,
        builder: &mut M1TraceBuilder,
    ) -> Result<(Vec<u32>, usize, usize), String> {
        let query = lane.symbols.as_ref();
        if query.len() > MAX_QUERY_SYMBOLS {
            return Err("query_symbol_budget".to_string());
        }
        let radius = lane.maximum_levenshtein_distance;
        if radius > MAX_LEVENSHTEIN_RADIUS {
            return Err("lane_radius_budget".to_string());
        }
        let lane_start = builder.events.len();
        builder.start_lane(query, radius);
        let mut stack = vec![SearchNode {
            state_id: index.root_state,
            rank_prefix: 0,
            row: BandedLevenshteinRow::initial(query.len(), radius),
        }];
        let mut form_refs = Vec::new();
        let mut expanded = 0_usize;
        let mut maximum_scratch = search_scratch_bytes(&stack, &form_refs);
        while let Some(node) = stack.pop() {
            expanded += 1;
            builder.expanded_states += 1;
            if expanded > SearchBudget::proof().maximum_product_states {
                return Err("product_state_budget".to_string());
            }
            let state = index.state(node.state_id)?;
            if state.terminal() && node.row.terminal_distance(query.len(), radius) <= radius {
                form_refs.push(node.rank_prefix);
                if form_refs.len() > SearchBudget::proof().maximum_terminals {
                    return Err("terminal_budget".to_string());
                }
            }
            for edge_id in index.edge_range(state)? {
                let edge = index.edge(edge_id)?;
                let rank_prefix = node
                    .rank_prefix
                    .checked_add(edge.rank_delta)
                    .ok_or_else(|| "V13 search rank overflows u32".to_string())?;
                let packed_previous = m1_pack(&node.row);
                if m1_pack(&m1_unpack(packed_previous)) != packed_previous {
                    builder.packed_state_mismatches += 1;
                }
                let equality_window = m1_equality_window(&node.row, query, edge.symbol, radius);
                let event = M1TraceEvent {
                    previous: node.row,
                    symbol: edge.symbol,
                    packed_previous,
                    equality_window,
                    radius,
                    query_len: query.len() as u8,
                };
                builder.push_event(event);
                let g0 = node.row.advance(query, edge.symbol, radius);
                let g0_minimum = g0.minimum(radius);
                let g1 = m1_g1_advance(&node.row, equality_window, radius, query.len());
                let g1_minimum = g1.minimum(radius);
                let u1 = m1_u1_advance(packed_previous, equality_window, radius, query.len());
                let u1_row = m1_unpack(u1.state);
                let terminal_column = query.len();
                let mismatch = m1_pack(&g0) != m1_pack(&g1)
                    || m1_pack(&g0) != m1_pack(&u1_row)
                    || g0_minimum != g1_minimum
                    || g0_minimum != u1.minimum
                    || g0.terminal_distance(terminal_column, radius)
                        != g1.terminal_distance(terminal_column, radius)
                    || g0.terminal_distance(terminal_column, radius)
                        != u1_row.terminal_distance(terminal_column, radius)
                    || (g0_minimum <= radius) != (u1.minimum <= radius);
                builder.transition_mismatches += usize::from(mismatch);
                if g0_minimum <= radius {
                    stack.push(SearchNode {
                        state_id: edge.target,
                        rank_prefix,
                        row: g0,
                    });
                }
            }
            maximum_scratch = maximum_scratch.max(search_scratch_bytes(&stack, &form_refs));
            if maximum_scratch > SearchBudget::proof().maximum_scratch_bytes {
                return Err("scratch_budget".to_string());
            }
        }
        builder.finish_lane(lane_start);
        Ok((form_refs, expanded, maximum_scratch))
    }

    fn m1_observed_search(
        index: &V13DafsaView,
        package: &RuntimeL2Package,
        observed: &str,
        builder: &mut M1TraceBuilder,
    ) -> Result<M1ObservedSearch, String> {
        let oracle = Phase7dCertificateOracle::new(observed)?;
        let mut terminal_refs = Vec::new();
        let mut expanded = 0_usize;
        let mut maximum_scratch = 0_usize;
        for lane in oracle.retrieval_lanes() {
            let (form_refs, lane_expanded, lane_scratch) =
                m1_enumerate_lane(index, &lane, builder)?;
            terminal_refs.extend(form_refs);
            expanded = expanded.saturating_add(lane_expanded);
            maximum_scratch = maximum_scratch.max(lane_scratch);
        }
        terminal_refs.sort_unstable();
        terminal_refs.dedup();
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
        Ok(M1ObservedSearch {
            retrieved_form_refs,
            peaks,
            completeness: SearchCompleteness::CertifiedExhaustive,
            expanded_product_states: expanded,
            maximum_scratch_bytes: maximum_scratch,
        })
    }

    fn m1_stress() -> (usize, usize, usize) {
        let mut cases = 0_usize;
        let mut transition_mismatches = 0_usize;
        let mut packed_state_mismatches = 0_usize;
        for query_len in 23_usize..=96 {
            for radius in 0_u8..=3 {
                let query = vec![1_u32; query_len];
                let mut previous = BandedLevenshteinRow::initial(query_len, radius);
                for _ in 0..=query_len + radius as usize {
                    let packed = m1_pack(&previous);
                    packed_state_mismatches += usize::from(m1_pack(&m1_unpack(packed)) != packed);
                    let depth = previous.depth.saturating_add(1);
                    let start = depth.saturating_sub(radius) as usize;
                    let end = (depth as usize + radius as usize).min(query_len);
                    let len = end.checked_sub(start).map_or(0, |width| width + 1);
                    for equality in 0_u16..(1_u16 << len) {
                        let g1 = m1_g1_advance(&previous, equality as u8, radius, query_len);
                        let u1 = m1_u1_advance(packed, equality as u8, radius, query_len);
                        transition_mismatches += usize::from(
                            m1_pack(&g1) != m1_pack(&m1_unpack(u1.state))
                                || g1.minimum(radius) != u1.minimum,
                        );
                        cases += 1;
                    }
                    previous = previous.advance(&query, 1, radius);
                }
            }
        }
        (cases, transition_mismatches, packed_state_mismatches)
    }

    fn m1_build_trace() -> Result<M1Trace, String> {
        let (package, index, cases) = m1_load_assets()?;
        let mut builder = M1TraceBuilder::new();
        let mut terminal_mismatches = 0_usize;
        let mut peak_mismatches = 0_usize;
        let mut completeness_mismatches = 0_usize;
        let mut work_mismatches = 0_usize;
        let mut scratch_mismatches = 0_usize;
        let mut maximum_product_states = 0_usize;
        let mut maximum_scratch_bytes = 0_usize;
        for case in &cases {
            let production = search_typed_peaks(
                &index,
                &package,
                &case.damaged_surface,
                SearchBudget::proof(),
            )?;
            let observed =
                m1_observed_search(&index, &package, &case.damaged_surface, &mut builder)?;
            terminal_mismatches +=
                usize::from(production.retrieved_form_refs != observed.retrieved_form_refs);
            peak_mismatches += usize::from(production.peaks != observed.peaks);
            completeness_mismatches +=
                usize::from(production.completeness != observed.completeness);
            work_mismatches +=
                usize::from(production.expanded_product_states != observed.expanded_product_states);
            scratch_mismatches +=
                usize::from(production.maximum_scratch_bytes != observed.maximum_scratch_bytes);
            maximum_product_states = maximum_product_states.max(observed.expanded_product_states);
            maximum_scratch_bytes = maximum_scratch_bytes.max(observed.maximum_scratch_bytes);
        }
        let digest = hex(builder.digest.finalize().into());
        Ok(M1Trace {
            events: builder.events,
            lanes: builder.lanes,
            lane_ranges: builder.lane_ranges,
            digest,
            records: cases.len(),
            expanded_states: builder.expanded_states,
            terminal_mismatches,
            peak_mismatches,
            completeness_mismatches,
            work_mismatches,
            scratch_mismatches,
            transition_mismatches: builder.transition_mismatches,
            packed_state_mismatches: builder.packed_state_mismatches,
            maximum_product_states,
            maximum_scratch_bytes,
        })
    }

    #[inline(never)]
    fn m1_replay_g0_event(event: &M1TraceEvent, query: &[u32]) -> u64 {
        let next = event.previous.advance(query, event.symbol, event.radius);
        let minimum = next.minimum(event.radius);
        let next = std::hint::black_box(next);
        u64::from(minimum)
            ^ (u64::from(next.depth) << 8)
            ^ (u64::from(next.start) << 16)
            ^ (u64::from(next.len) << 24)
            ^ (u64::from(next.cells[0]) << 32)
    }

    #[inline(never)]
    fn m1_replay_g1_event(event: &M1TraceEvent) -> u64 {
        let next = m1_g1_advance(
            &event.previous,
            event.equality_window,
            event.radius,
            event.query_len as usize,
        );
        let minimum = next.minimum(event.radius);
        let next = std::hint::black_box(next);
        u64::from(minimum)
            ^ (u64::from(next.depth) << 8)
            ^ (u64::from(next.start) << 16)
            ^ (u64::from(next.len) << 24)
            ^ (u64::from(next.cells[0]) << 32)
    }

    #[inline(never)]
    fn m1_replay_u1_event(event: &M1TraceEvent) -> u64 {
        let next = std::hint::black_box(m1_u1_advance(
            event.packed_previous,
            event.equality_window,
            event.radius,
            event.query_len as usize,
        ));
        next.state ^ (u64::from(next.minimum) << 56)
    }

    fn m1_replay(trace: &M1Trace, mode: &str) -> Result<(u64, u128), String> {
        let started = Instant::now();
        let mut checksum = 0_u64;
        for (lane_index, range) in trace.lane_ranges.iter().enumerate() {
            let query = trace
                .lanes
                .get(lane_index)
                .ok_or_else(|| "lane range/query mismatch".to_string())?;
            for event in &trace.events[range.clone()] {
                let value = match mode {
                    "G0" => m1_replay_g0_event(event, query),
                    "G1" => m1_replay_g1_event(event),
                    "U1" => m1_replay_u1_event(event),
                    _ => return Err(format!("unsupported M1 mode {mode}")),
                };
                checksum = checksum.rotate_left(7) ^ value;
            }
        }
        Ok((std::hint::black_box(checksum), started.elapsed().as_nanos()))
    }

    fn m1_parity_json(trace: &M1Trace) -> serde_json::Value {
        serde_json::json!({
            "records": trace.records,
            "transitions": trace.events.len(),
            "expanded_states": trace.expanded_states,
            "trace_sha256": trace.digest,
            "trace_event_bytes": std::mem::size_of::<M1TraceEvent>(),
            "trace_bytes": trace.events.len() * std::mem::size_of::<M1TraceEvent>(),
            "lanes": trace.lanes.len(),
            "terminal_mismatches": trace.terminal_mismatches,
            "peak_mismatches": trace.peak_mismatches,
            "completeness_mismatches": trace.completeness_mismatches,
            "work_mismatches": trace.work_mismatches,
            "scratch_mismatches": trace.scratch_mismatches,
            "transition_mismatches": trace.transition_mismatches,
            "packed_state_mismatches": trace.packed_state_mismatches,
            "maximum_product_states": trace.maximum_product_states,
            "maximum_scratch_bytes": trace.maximum_scratch_bytes,
        })
    }

    fn m1_run() -> Result<serde_json::Value, String> {
        let mode = std::env::var("LAY_V10_M1_MODE").unwrap_or_else(|_| "PARITY".to_string());
        let trace = m1_build_trace()?;
        let (stress_cases, stress_transition_mismatches, stress_packed_state_mismatches) =
            m1_stress();
        let parity_pass = trace.records == 382
            && trace.events.len() == M1_EXPECTED_TRANSITIONS
            && trace.expanded_states == M1_EXPECTED_EXPANDED
            && trace.terminal_mismatches == 0
            && trace.peak_mismatches == 0
            && trace.completeness_mismatches == 0
            && trace.work_mismatches == 0
            && trace.scratch_mismatches == 0
            && trace.transition_mismatches == 0
            && trace.packed_state_mismatches == 0
            && stress_transition_mismatches == 0
            && stress_packed_state_mismatches == 0
            && trace.maximum_product_states == 35_590
            && trace.maximum_scratch_bytes == 6_656;
        if mode == "PARITY" {
            return Ok(serde_json::json!({
                "schema": "lay.v10.exact-fused-band-transition-m1-parity.v1",
                "verdict": if parity_pass { "PASS" } else { "FAIL" },
                "test": M1_TEST,
                "parity": m1_parity_json(&trace),
                "stress": {
                    "length_min": 23,
                    "length_max": 96,
                    "radius_min": 0,
                    "radius_max": 3,
                    "cases": stress_cases,
                    "transition_mismatches": stress_transition_mismatches,
                    "packed_state_mismatches": stress_packed_state_mismatches,
                },
                "s1_swar_candidate": "OMITTED",
                "formal_b_pass": false,
                "v12_admitted": false,
                "runtime_authority_changed": false,
            }));
        }
        if !parity_pass {
            return Err("M1 parity failed before physical replay".to_string());
        }
        let control = m1_required_path("LAY_V10_M1_CONTROL_DIR")?;
        let ready = control.join("subject-ready");
        let enabled = control.join("controller-enabled");
        let done = control.join("subject-done");
        let disabled = control.join("controller-disabled");
        m1_write_new(&ready, b"ready\n")?;
        m1_wait_for(&enabled)?;
        let (checksum, wall_ns) = m1_replay(&trace, &mode)?;
        m1_write_new(&done, b"done\n")?;
        m1_wait_for(&disabled)?;
        Ok(serde_json::json!({
            "schema": "lay.v10.exact-fused-band-transition-m1-physical.v1",
            "verdict": "PHYSICAL_REPLAY_OBSERVED",
            "test": M1_TEST,
            "mode": mode,
            "parity": m1_parity_json(&trace),
            "stress": {
                "cases": stress_cases,
                "transition_mismatches": stress_transition_mismatches,
                "packed_state_mismatches": stress_packed_state_mismatches,
            },
            "checksum": checksum,
            "subject_wall_ns_diagnostic": wall_ns,
            "s1_swar_candidate": "OMITTED",
            "formal_b_pass": false,
            "v12_admitted": false,
            "runtime_authority_changed": false,
        }))
    }

    #[test]
    #[ignore = "one-shot exact V10 fused-band transition microproof"]
    fn v10_exact_fused_band_transition_m1() {
        let receipt = m1_run().unwrap_or_else(|error| {
            serde_json::json!({
                "schema": "lay.v10.exact-fused-band-transition-m1-error.v1",
                "verdict": "ERROR",
                "error": error,
                "formal_b_pass": false,
                "v12_admitted": false,
                "runtime_authority_changed": false,
            })
        });
        let output = m1_required_path("LAY_V10_M1_OUTPUT").expect("M1 output path");
        m1_write_json(&output, &receipt).expect("write M1 receipt");
        assert!(
            matches!(
                receipt.get("verdict").and_then(serde_json::Value::as_str),
                Some("PASS" | "PHYSICAL_REPLAY_OBSERVED")
            ),
            "{}",
            serde_json::to_string_pretty(&receipt).unwrap(),
        );
    }
