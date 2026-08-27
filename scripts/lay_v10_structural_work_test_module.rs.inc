
    const A1_STRUCTURAL_TEST: &str =
        "nanda_wave::l2_field::v13_typed_peak::tests::v10_structural_work_a1";
    const A1_SCHEDULE_SHA256: &str =
        "2f8346d97246ac3704434279739fa5ed27705465d628890a367c21875d879e78";
    const A1_SIDECAR_SHA256: &str =
        "a1aa95be8c43fae8fedf02d3261dd1265dcbfbc8d618330fcd6449ae829df8cd";

    macro_rules! a1_counter_type {
        ($($field:ident),+ $(,)?) => {
            #[derive(Clone, Copy, Debug, Default)]
            struct A1Counters {
                $($field: u64,)+
            }

            impl A1Counters {
                fn add_assign(&mut self, other: Self) {
                    $(self.$field = self.$field.saturating_add(other.$field);)+
                }

                fn json(self) -> serde_json::Value {
                    let mut value = serde_json::Map::new();
                    $(value.insert(
                        stringify!($field).to_string(),
                        serde_json::Value::from(self.$field),
                    );)+
                    serde_json::Value::Object(value)
                }
            }
        };
    }

    a1_counter_type!(
        oracle_constructions,
        retrieval_lane_calls,
        retrieval_lanes,
        expanded_states,
        state_decodes,
        edge_range_calls,
        edges_examined,
        edge_decodes,
        rank_delta_additions,
        transition_calls,
        band_cells_evaluated,
        query_symbol_comparisons,
        band_value_lookups,
        minimum_calls,
        minimum_cells_scanned,
        surviving_edges,
        pruned_edges,
        stack_pushes,
        stack_pops,
        stack_capacity_growths,
        terminal_state_checks,
        terminal_state_hits,
        terminal_distance_calls,
        terminal_refs_collected,
        terminal_vector_capacity_growths,
        terminal_refs_pre_dedup,
        terminal_refs_post_dedup,
        terminal_sort_calls,
        terminal_dedup_calls,
        surface_decodes,
        certificate_calls,
        certified_peaks,
        scratch_observations,
    );

    #[derive(Clone, Copy, Debug, Default)]
    struct A1AllocationCounts {
        alloc_calls: u64,
        alloc_zeroed_calls: u64,
        realloc_calls: u64,
        dealloc_calls: u64,
        allocated_bytes: u64,
        zeroed_bytes: u64,
        realloc_old_bytes: u64,
        realloc_new_bytes: u64,
        deallocated_bytes: u64,
        current_live_requested_bytes: i64,
        maximum_live_requested_bytes: i64,
        minimum_live_requested_bytes: i64,
    }

    impl A1AllocationCounts {
        fn add_assign(&mut self, other: Self) {
            self.alloc_calls = self.alloc_calls.saturating_add(other.alloc_calls);
            self.alloc_zeroed_calls = self
                .alloc_zeroed_calls
                .saturating_add(other.alloc_zeroed_calls);
            self.realloc_calls = self.realloc_calls.saturating_add(other.realloc_calls);
            self.dealloc_calls = self.dealloc_calls.saturating_add(other.dealloc_calls);
            self.allocated_bytes = self.allocated_bytes.saturating_add(other.allocated_bytes);
            self.zeroed_bytes = self.zeroed_bytes.saturating_add(other.zeroed_bytes);
            self.realloc_old_bytes = self
                .realloc_old_bytes
                .saturating_add(other.realloc_old_bytes);
            self.realloc_new_bytes = self
                .realloc_new_bytes
                .saturating_add(other.realloc_new_bytes);
            self.deallocated_bytes = self
                .deallocated_bytes
                .saturating_add(other.deallocated_bytes);
            self.current_live_requested_bytes = self
                .current_live_requested_bytes
                .saturating_add(other.current_live_requested_bytes);
            self.maximum_live_requested_bytes = self
                .maximum_live_requested_bytes
                .max(other.maximum_live_requested_bytes);
            self.minimum_live_requested_bytes = self
                .minimum_live_requested_bytes
                .min(other.minimum_live_requested_bytes);
        }

        fn conservation_expected(self) -> i64 {
            let positive = self
                .allocated_bytes
                .saturating_add(self.zeroed_bytes)
                .saturating_add(self.realloc_new_bytes);
            let negative = self
                .realloc_old_bytes
                .saturating_add(self.deallocated_bytes);
            i64::try_from(positive).unwrap_or(i64::MAX)
                .saturating_sub(i64::try_from(negative).unwrap_or(i64::MAX))
        }

        fn json(self) -> serde_json::Value {
            serde_json::json!({
                "alloc_calls": self.alloc_calls,
                "alloc_zeroed_calls": self.alloc_zeroed_calls,
                "realloc_calls": self.realloc_calls,
                "dealloc_calls": self.dealloc_calls,
                "allocated_bytes": self.allocated_bytes,
                "zeroed_bytes": self.zeroed_bytes,
                "realloc_old_bytes": self.realloc_old_bytes,
                "realloc_new_bytes": self.realloc_new_bytes,
                "deallocated_bytes": self.deallocated_bytes,
                "current_live_requested_bytes": self.current_live_requested_bytes,
                "maximum_live_requested_bytes": self.maximum_live_requested_bytes,
                "minimum_live_requested_bytes": self.minimum_live_requested_bytes,
                "conservation_expected_live_bytes": self.conservation_expected(),
                "conservation_pass": self.conservation_expected()
                    == self.current_live_requested_bytes,
            })
        }
    }

    thread_local! {
        static A1_ALLOC_ENABLED: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
        static A1_ALLOC_COUNTS: std::cell::Cell<A1AllocationCounts> =
            const { std::cell::Cell::new(A1AllocationCounts {
                alloc_calls: 0,
                alloc_zeroed_calls: 0,
                realloc_calls: 0,
                dealloc_calls: 0,
                allocated_bytes: 0,
                zeroed_bytes: 0,
                realloc_old_bytes: 0,
                realloc_new_bytes: 0,
                deallocated_bytes: 0,
                current_live_requested_bytes: 0,
                maximum_live_requested_bytes: 0,
                minimum_live_requested_bytes: 0,
            }) };
    }

    struct A1CountingAllocator;

    fn a1_record_allocation(update: impl FnOnce(&mut A1AllocationCounts)) {
        A1_ALLOC_ENABLED.with(|enabled| {
            if enabled.get() {
                A1_ALLOC_COUNTS.with(|counts| {
                    let mut value = counts.get();
                    update(&mut value);
                    value.maximum_live_requested_bytes = value
                        .maximum_live_requested_bytes
                        .max(value.current_live_requested_bytes);
                    value.minimum_live_requested_bytes = value
                        .minimum_live_requested_bytes
                        .min(value.current_live_requested_bytes);
                    counts.set(value);
                });
            }
        });
    }

    unsafe impl std::alloc::GlobalAlloc for A1CountingAllocator {
        unsafe fn alloc(&self, layout: std::alloc::Layout) -> *mut u8 {
            let pointer = unsafe { std::alloc::GlobalAlloc::alloc(&std::alloc::System, layout) };
            if !pointer.is_null() {
                a1_record_allocation(|counts| {
                    counts.alloc_calls = counts.alloc_calls.saturating_add(1);
                    counts.allocated_bytes = counts
                        .allocated_bytes
                        .saturating_add(layout.size() as u64);
                    counts.current_live_requested_bytes = counts
                        .current_live_requested_bytes
                        .saturating_add(layout.size() as i64);
                });
            }
            pointer
        }

        unsafe fn alloc_zeroed(&self, layout: std::alloc::Layout) -> *mut u8 {
            let pointer = unsafe {
                std::alloc::GlobalAlloc::alloc_zeroed(&std::alloc::System, layout)
            };
            if !pointer.is_null() {
                a1_record_allocation(|counts| {
                    counts.alloc_zeroed_calls = counts.alloc_zeroed_calls.saturating_add(1);
                    counts.zeroed_bytes = counts.zeroed_bytes.saturating_add(layout.size() as u64);
                    counts.current_live_requested_bytes = counts
                        .current_live_requested_bytes
                        .saturating_add(layout.size() as i64);
                });
            }
            pointer
        }

        unsafe fn dealloc(&self, pointer: *mut u8, layout: std::alloc::Layout) {
            a1_record_allocation(|counts| {
                counts.dealloc_calls = counts.dealloc_calls.saturating_add(1);
                counts.deallocated_bytes = counts
                    .deallocated_bytes
                    .saturating_add(layout.size() as u64);
                counts.current_live_requested_bytes = counts
                    .current_live_requested_bytes
                    .saturating_sub(layout.size() as i64);
            });
            unsafe { std::alloc::GlobalAlloc::dealloc(&std::alloc::System, pointer, layout) };
        }

        unsafe fn realloc(
            &self,
            pointer: *mut u8,
            layout: std::alloc::Layout,
            new_size: usize,
        ) -> *mut u8 {
            let output = unsafe {
                std::alloc::GlobalAlloc::realloc(&std::alloc::System, pointer, layout, new_size)
            };
            if !output.is_null() {
                a1_record_allocation(|counts| {
                    counts.realloc_calls = counts.realloc_calls.saturating_add(1);
                    counts.realloc_old_bytes = counts
                        .realloc_old_bytes
                        .saturating_add(layout.size() as u64);
                    counts.realloc_new_bytes =
                        counts.realloc_new_bytes.saturating_add(new_size as u64);
                    counts.current_live_requested_bytes = counts
                        .current_live_requested_bytes
                        .saturating_sub(layout.size() as i64)
                        .saturating_add(new_size as i64);
                });
            }
            output
        }
    }

    #[global_allocator]
    static A1_GLOBAL_ALLOCATOR: A1CountingAllocator = A1CountingAllocator;

    fn a1_with_allocation_counts<T>(operation: impl FnOnce() -> T) -> (T, A1AllocationCounts) {
        A1_ALLOC_ENABLED.with(|enabled| assert!(!enabled.get(), "nested A1 allocator scope"));
        A1_ALLOC_COUNTS.with(|counts| counts.set(A1AllocationCounts::default()));
        A1_ALLOC_ENABLED.with(|enabled| enabled.set(true));
        let output = operation();
        A1_ALLOC_ENABLED.with(|enabled| enabled.set(false));
        let counts = A1_ALLOC_COUNTS.with(std::cell::Cell::get);
        (output, counts)
    }

    struct A1LaneOutcome {
        form_refs: Vec<u32>,
        expanded: usize,
        maximum_scratch: usize,
        unresolved: Option<&'static str>,
        counters: A1Counters,
    }

    struct A1ObservedSearch {
        retrieved_form_refs: Vec<u32>,
        peaks: Vec<V13TypedPeak>,
        completeness: SearchCompleteness,
        expanded_product_states: usize,
        maximum_scratch_bytes: usize,
        counters: A1Counters,
        allocations: A1AllocationCounts,
    }

    fn a1_advance(
        previous: &BandedLevenshteinRow,
        query: &[u32],
        symbol: u32,
        radius: u8,
        counters: &mut A1Counters,
    ) -> BandedLevenshteinRow {
        let outside = radius.saturating_add(1);
        let depth = previous.depth.checked_add(1).expect("bounded query depth");
        let start = depth.saturating_sub(radius) as usize;
        let end = (depth as usize + radius as usize).min(query.len());
        let len = end.checked_sub(start).map_or(0, |width| width + 1);
        let mut cells = [outside; MAX_BAND_CELLS];
        for column in start..start + len {
            counters.band_cells_evaluated += 1;
            let value = if column == 0 {
                depth.min(outside)
            } else {
                counters.query_symbol_comparisons += 1;
                counters.band_value_lookups += 2;
                let left = if column > start {
                    cells[column - 1 - start]
                } else {
                    outside
                }
                .saturating_add(1)
                .min(outside);
                let above = previous
                    .value(column, outside)
                    .saturating_add(1)
                    .min(outside);
                let diagonal = previous
                    .value(column - 1, outside)
                    .saturating_add(u8::from(query[column - 1] != symbol))
                    .min(outside);
                left.min(above).min(diagonal)
            };
            cells[column - start] = value;
        }
        BandedLevenshteinRow {
            cells,
            depth,
            start: start as u8,
            len: len as u8,
        }
    }

    fn a1_minimum(row: &BandedLevenshteinRow, radius: u8, counters: &mut A1Counters) -> u8 {
        counters.minimum_calls += 1;
        counters.minimum_cells_scanned += row.len as u64;
        row.cells[..row.len as usize]
            .iter()
            .copied()
            .min()
            .unwrap_or_else(|| radius.saturating_add(1))
    }

    fn a1_push<T>(vector: &mut Vec<T>, value: T, growths: &mut u64) {
        let capacity = vector.capacity();
        vector.push(value);
        *growths += u64::from(vector.capacity() != capacity);
    }

    fn a1_enumerate_lane(
        index: &V13DafsaView,
        lane: &Phase7dRetrievalLane,
        budget: SearchBudget,
    ) -> Result<A1LaneOutcome, String> {
        let query = lane.symbols.as_ref();
        if query.len() > MAX_QUERY_SYMBOLS {
            return Ok(A1LaneOutcome {
                form_refs: Vec::new(),
                expanded: 0,
                maximum_scratch: 0,
                unresolved: Some("query_symbol_budget"),
                counters: A1Counters::default(),
            });
        }
        if lane.maximum_levenshtein_distance > MAX_LEVENSHTEIN_RADIUS {
            return Ok(A1LaneOutcome {
                form_refs: Vec::new(),
                expanded: 0,
                maximum_scratch: 0,
                unresolved: Some("lane_radius_budget"),
                counters: A1Counters::default(),
            });
        }
        let mut counters = A1Counters::default();
        let mut stack = Vec::new();
        a1_push(
            &mut stack,
            SearchNode {
                state_id: index.root_state,
                rank_prefix: 0,
                row: BandedLevenshteinRow::initial(
                    query.len(),
                    lane.maximum_levenshtein_distance,
                ),
            },
            &mut counters.stack_capacity_growths,
        );
        counters.stack_pushes += 1;
        let mut form_refs = Vec::new();
        let mut expanded = 0_usize;
        let mut maximum_scratch = search_scratch_bytes(&stack, &form_refs);
        counters.scratch_observations += 1;
        while let Some(node) = stack.pop() {
            counters.stack_pops += 1;
            counters.expanded_states += 1;
            expanded += 1;
            if expanded > budget.maximum_product_states {
                return Ok(A1LaneOutcome {
                    form_refs,
                    expanded,
                    maximum_scratch,
                    unresolved: Some("product_state_budget"),
                    counters,
                });
            }
            let state = index.state(node.state_id)?;
            counters.state_decodes += 1;
            counters.terminal_state_checks += 1;
            if state.terminal() {
                counters.terminal_state_hits += 1;
                counters.terminal_distance_calls += 1;
                counters.band_value_lookups += 1;
                if node.row.terminal_distance(
                    query.len(),
                    lane.maximum_levenshtein_distance,
                ) <= lane.maximum_levenshtein_distance
                {
                    a1_push(
                        &mut form_refs,
                        node.rank_prefix,
                        &mut counters.terminal_vector_capacity_growths,
                    );
                    counters.terminal_refs_collected += 1;
                    if form_refs.len() > budget.maximum_terminals {
                        return Ok(A1LaneOutcome {
                            form_refs,
                            expanded,
                            maximum_scratch,
                            unresolved: Some("terminal_budget"),
                            counters,
                        });
                    }
                }
            }
            counters.edge_range_calls += 1;
            for edge_id in index.edge_range(state)? {
                counters.edges_examined += 1;
                let edge = index.edge(edge_id)?;
                counters.edge_decodes += 1;
                let rank_prefix = node
                    .rank_prefix
                    .checked_add(edge.rank_delta)
                    .ok_or_else(|| "V13 search rank overflows u32".to_string())?;
                counters.rank_delta_additions += 1;
                counters.transition_calls += 1;
                let next_row = a1_advance(
                    &node.row,
                    query,
                    edge.symbol,
                    lane.maximum_levenshtein_distance,
                    &mut counters,
                );
                if a1_minimum(
                    &next_row,
                    lane.maximum_levenshtein_distance,
                    &mut counters,
                ) <= lane.maximum_levenshtein_distance
                {
                    a1_push(
                        &mut stack,
                        SearchNode {
                            state_id: edge.target,
                            rank_prefix,
                            row: next_row,
                        },
                        &mut counters.stack_capacity_growths,
                    );
                    counters.stack_pushes += 1;
                    counters.surviving_edges += 1;
                } else {
                    counters.pruned_edges += 1;
                }
            }
            maximum_scratch = maximum_scratch.max(search_scratch_bytes(&stack, &form_refs));
            counters.scratch_observations += 1;
            if maximum_scratch > budget.maximum_scratch_bytes {
                return Ok(A1LaneOutcome {
                    form_refs,
                    expanded,
                    maximum_scratch,
                    unresolved: Some("scratch_budget"),
                    counters,
                });
            }
        }
        Ok(A1LaneOutcome {
            form_refs,
            expanded,
            maximum_scratch,
            unresolved: None,
            counters,
        })
    }

    fn a1_search_unscoped(
        index: &V13DafsaView,
        package: &RuntimeL2Package,
        observed: &str,
        budget: SearchBudget,
    ) -> Result<A1ObservedSearch, String> {
        let oracle = Phase7dCertificateOracle::new(observed)?;
        let lanes = oracle.retrieval_lanes();
        let mut counters = A1Counters {
            oracle_constructions: 1,
            retrieval_lane_calls: 1,
            retrieval_lanes: lanes.len() as u64,
            ..A1Counters::default()
        };
        let mut terminal_refs = Vec::new();
        let mut expanded = 0_usize;
        let mut maximum_scratch = 0_usize;
        for lane in lanes {
            let outcome = a1_enumerate_lane(index, &lane, budget)?;
            counters.add_assign(outcome.counters);
            expanded = expanded.saturating_add(outcome.expanded);
            maximum_scratch = maximum_scratch.max(outcome.maximum_scratch);
            if let Some(reason) = outcome.unresolved {
                return Ok(A1ObservedSearch {
                    retrieved_form_refs: Vec::new(),
                    peaks: Vec::new(),
                    completeness: SearchCompleteness::Unresolved(reason),
                    expanded_product_states: expanded,
                    maximum_scratch_bytes: maximum_scratch,
                    counters,
                    allocations: A1AllocationCounts::default(),
                });
            }
            terminal_refs.extend(outcome.form_refs);
        }
        counters.terminal_refs_pre_dedup = terminal_refs.len() as u64;
        counters.terminal_sort_calls = 1;
        terminal_refs.sort_unstable();
        counters.terminal_dedup_calls = 1;
        terminal_refs.dedup();
        counters.terminal_refs_post_dedup = terminal_refs.len() as u64;
        let retrieved_form_refs = terminal_refs.clone();
        let mut peaks = Vec::new();
        for form_ref in terminal_refs {
            let surface = package
                .surface(form_ref as usize)
                .ok_or_else(|| format!("V13 terminal rank {form_ref} cannot be decoded"))?;
            counters.surface_decodes += 1;
            let certificate_keys = oracle.certificate_keys(surface.as_ref())?;
            counters.certificate_calls += 1;
            if !certificate_keys.is_empty() {
                peaks.push(V13TypedPeak {
                    form_ref,
                    certificate_keys,
                });
                counters.certified_peaks += 1;
            }
        }
        Ok(A1ObservedSearch {
            retrieved_form_refs,
            peaks,
            completeness: SearchCompleteness::CertifiedExhaustive,
            expanded_product_states: expanded,
            maximum_scratch_bytes: maximum_scratch,
            counters,
            allocations: A1AllocationCounts::default(),
        })
    }

    fn a1_search(
        index: &V13DafsaView,
        package: &RuntimeL2Package,
        observed: &str,
    ) -> Result<A1ObservedSearch, String> {
        let (result, allocations) = a1_with_allocation_counts(|| {
            a1_search_unscoped(index, package, observed, SearchBudget::proof())
        });
        let mut result = result?;
        result.allocations = allocations;
        Ok(result)
    }

    fn a1_required_path(name: &str) -> Result<std::path::PathBuf, String> {
        std::env::var_os(name)
            .map(std::path::PathBuf::from)
            .ok_or_else(|| format!("missing required environment variable {name}"))
    }

    fn a1_digest(bytes: &[u8]) -> String {
        hex(Sha256::digest(bytes).into())
    }

    fn a1_lanes_digest(lanes: &[Phase7dRetrievalLane]) -> Result<String, String> {
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
            .map(|bytes| a1_digest(&bytes))
            .map_err(|error| error.to_string())
    }

    fn a1_write_new_json(path: &Path, value: &serde_json::Value) -> Result<(), String> {
        if !path.parent().is_some_and(Path::is_dir) {
            return Err(format!("output parent does not exist for {}", path.display()));
        }
        let mut output = std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(path)
            .map_err(|error| format!("{}: {error}", path.display()))?;
        serde_json::to_writer_pretty(&mut output, value).map_err(|error| error.to_string())?;
        use std::io::Write;
        output.write_all(b"\n").map_err(|error| error.to_string())?;
        output.sync_all().map_err(|error| error.to_string())
    }

    fn a1_load_assets(
    ) -> Result<(RuntimeL2Package, V13DafsaView, Vec<ProofCase>, serde_json::Value), String> {
        let package_path = a1_required_path("LAY_V10_A1_V13_PACKAGE")?;
        let sidecar_path = a1_required_path("LAY_V10_A1_SIDECAR")?;
        let v7_path = a1_required_path("LAY_V10_A1_V7")?;
        let schedule_path = a1_required_path("LAY_V10_A1_SCHEDULE")?;
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
        if hex(sha256_file(&sidecar_path)?) != A1_SIDECAR_SHA256 {
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
        if hex(sha256_file(&schedule_path)?) != A1_SCHEDULE_SHA256 {
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
            if entry.get("source_ordinal").and_then(serde_json::Value::as_u64)
                != Some(source_ordinal as u64)
            {
                return Err(format!("schedule ordinal mismatch at {source_ordinal}"));
            }
            if entry
                .get("damaged_surface_sha256")
                .and_then(serde_json::Value::as_str)
                != Some(a1_digest(case.damaged_surface.as_bytes()).as_str())
            {
                return Err(format!("schedule surface mismatch at {source_ordinal}"));
            }
            let oracle = Phase7dCertificateOracle::new(&case.damaged_surface)?;
            let digest = a1_lanes_digest(&oracle.retrieval_lanes())?;
            if entry
                .get("retrieval_lanes_sha256")
                .and_then(serde_json::Value::as_str)
                != Some(digest.as_str())
            {
                return Err(format!("schedule lanes mismatch at {source_ordinal}"));
            }
        }
        Ok((package, index, cases, schedule))
    }

    fn a1_run() -> Result<serde_json::Value, String> {
        let (package, index, cases, _schedule) = a1_load_assets()?;
        let bindings = ProofBindingIndex::build(&package)?;
        let mut aggregate_counters = A1Counters::default();
        let mut aggregate_allocations = A1AllocationCounts::default();
        let mut queries = Vec::with_capacity(cases.len());
        let mut terminal_mismatches = 0_usize;
        let mut peak_mismatches = 0_usize;
        let mut completeness_mismatches = 0_usize;
        let mut expanded_state_mismatches = 0_usize;
        let mut scratch_mismatches = 0_usize;
        let mut unresolved = 0_usize;
        let mut target_form_retained = 0_usize;
        let mut target_lemma_retained = 0_usize;
        let mut false_certificates = 0_usize;
        let mut maximum_product_states = 0_usize;
        let mut maximum_scratch_bytes = 0_usize;
        let mut allocation_conservation_failures = 0_usize;
        for (source_ordinal, case) in cases.iter().enumerate() {
            let production = search_typed_peaks(
                &index,
                &package,
                &case.damaged_surface,
                SearchBudget::proof(),
            )?;
            let observed = a1_search(&index, &package, &case.damaged_surface)?;
            terminal_mismatches +=
                usize::from(production.retrieved_form_refs != observed.retrieved_form_refs);
            peak_mismatches += usize::from(production.peaks != observed.peaks);
            completeness_mismatches +=
                usize::from(production.completeness != observed.completeness);
            expanded_state_mismatches += usize::from(
                production.expanded_product_states != observed.expanded_product_states,
            );
            scratch_mismatches += usize::from(
                production.maximum_scratch_bytes != observed.maximum_scratch_bytes,
            );
            unresolved += usize::from(
                observed.completeness != SearchCompleteness::CertifiedExhaustive,
            );
            maximum_product_states =
                maximum_product_states.max(observed.expanded_product_states);
            maximum_scratch_bytes = maximum_scratch_bytes.max(observed.maximum_scratch_bytes);
            allocation_conservation_failures += usize::from(
                observed.allocations.conservation_expected()
                    != observed.allocations.current_live_requested_bytes,
            );
            let target_ref = package
                .form_ref_for_surface(&case.target_surface)
                .ok_or_else(|| format!("V13 lacks target {}", case.target_surface))?;
            let found_refs = observed
                .peaks
                .iter()
                .map(|peak| peak.form_ref)
                .collect::<BTreeSet<_>>();
            target_form_retained += usize::from(found_refs.contains(&target_ref));
            let target_lemmas = bindings.lemma_ids(&package, target_ref);
            let found_lemmas = found_refs
                .iter()
                .flat_map(|form_ref| bindings.lemma_ids(&package, *form_ref))
                .collect::<BTreeSet<_>>();
            target_lemma_retained += usize::from(
                !target_lemmas.is_empty()
                    && target_lemmas
                        .iter()
                        .any(|lemma| found_lemmas.contains(lemma)),
            );
            for peak in &observed.peaks {
                let surface = package
                    .surface(peak.form_ref as usize)
                    .ok_or_else(|| format!("missing emitted form {}", peak.form_ref))?;
                let keys = crate::nanda_wave::lexical_grokking::phase7d_certificate_keys(
                    &case.damaged_surface,
                    surface.as_ref(),
                )?;
                false_certificates +=
                    usize::from(keys.is_empty() || keys != peak.certificate_keys);
            }
            aggregate_counters.add_assign(observed.counters);
            aggregate_allocations.add_assign(observed.allocations);
            queries.push(serde_json::json!({
                "source_ordinal": source_ordinal,
                "damaged_surface_sha256": a1_digest(case.damaged_surface.as_bytes()),
                "expanded_product_states": observed.expanded_product_states,
                "maximum_scratch_bytes": observed.maximum_scratch_bytes,
                "terminal_refs": observed.retrieved_form_refs.len(),
                "certified_peaks": observed.peaks.len(),
                "counters": observed.counters.json(),
                "allocations": observed.allocations.json(),
            }));
        }
        let additive_identities_pass = aggregate_counters.edges_examined
            == aggregate_counters.surviving_edges + aggregate_counters.pruned_edges
            && aggregate_counters.edges_examined == aggregate_counters.edge_decodes
            && aggregate_counters.edges_examined == aggregate_counters.rank_delta_additions
            && aggregate_counters.edges_examined == aggregate_counters.transition_calls
            && aggregate_counters.stack_pops == aggregate_counters.expanded_states
            && aggregate_counters.stack_pushes
                == aggregate_counters.retrieval_lanes + aggregate_counters.surviving_edges
            && aggregate_counters.terminal_refs_collected
                == aggregate_counters.terminal_refs_pre_dedup
            && aggregate_counters.surface_decodes == aggregate_counters.certificate_calls;
        let pass = terminal_mismatches == 0
            && peak_mismatches == 0
            && completeness_mismatches == 0
            && expanded_state_mismatches == 0
            && scratch_mismatches == 0
            && unresolved == 0
            && target_form_retained == 382
            && target_lemma_retained == 382
            && false_certificates == 0
            && maximum_product_states == 35_590
            && maximum_scratch_bytes == 6_656
            && aggregate_counters.expanded_states == 8_059_788
            && aggregate_counters.terminal_refs_post_dedup == 17_600
            && allocation_conservation_failures == 0
            && additive_identities_pass;
        Ok(serde_json::json!({
            "schema": "lay.v10.structural-work-a1.v1",
            "verdict": if pass { "STRUCTURAL_WORK_OBSERVED_NO_PROMOTION" } else { "FAIL" },
            "test": A1_STRUCTURAL_TEST,
            "records": cases.len(),
            "production_parity": {
                "terminal_mismatches": terminal_mismatches,
                "peak_mismatches": peak_mismatches,
                "completeness_mismatches": completeness_mismatches,
                "expanded_state_mismatches": expanded_state_mismatches,
                "scratch_mismatches": scratch_mismatches,
                "unresolved": unresolved,
                "target_form_retained": target_form_retained,
                "target_lemma_retained": target_lemma_retained,
                "false_certificates": false_certificates,
                "maximum_product_states": maximum_product_states,
                "maximum_scratch_bytes": maximum_scratch_bytes,
            },
            "aggregate": {
                "counters": aggregate_counters.json(),
                "allocations": aggregate_allocations.json(),
                "allocation_conservation_failures": allocation_conservation_failures,
                "additive_identities_pass": additive_identities_pass,
            },
            "queries": queries,
            "measurement_boundary": {
                "perf_invoked": false,
                "pmu_event_opened": false,
                "latency_authority": false,
                "loaded_host_is_blocker": false,
                "foreign_process_control": false,
                "allocator_scope": "current observer thread from oracle construction through peak materialization",
            },
            "formal_b_pass": false,
            "v12_admitted": false,
            "runtime_authority_changed": false,
            "installed_lay_changed": false,
        }))
    }

    #[test]
    #[ignore = "one-shot deterministic V10 structural work observer; no latency or PMU authority"]
    fn v10_structural_work_a1() {
        let receipt = a1_run().unwrap_or_else(|error| {
            serde_json::json!({
                "schema": "lay.v10.structural-work-a1.v1",
                "verdict": "ERROR",
                "error": error,
                "formal_b_pass": false,
                "v12_admitted": false,
                "runtime_authority_changed": false,
                "installed_lay_changed": false,
            })
        });
        let output = a1_required_path("LAY_V10_A1_OUTPUT").expect("A1 output path");
        a1_write_new_json(&output, &receipt).expect("write A1 receipt");
        assert_eq!(
            receipt.get("verdict").and_then(serde_json::Value::as_str),
            Some("STRUCTURAL_WORK_OBSERVED_NO_PROMOTION"),
            "{}",
            serde_json::to_string_pretty(&receipt).unwrap(),
        );
    }
