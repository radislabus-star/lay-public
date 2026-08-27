
    const HARDWARE_SCHEDULE_SCHEMA: &str = "lay.v10.hardware-query-schedule.v1";
    const HARDWARE_FREEZER_TEST: &str =
        "nanda_wave::l2_field::v13_typed_peak::tests::v10_hardware_schedule_freezer";
    const HARDWARE_PARITY_TEST: &str =
        "nanda_wave::l2_field::v13_typed_peak::tests::v10_hardware_semantic_parity";
    const HARDWARE_B5_TEST: &str =
        "nanda_wave::l2_field::v13_typed_peak::tests::v10_hardware_b5_proxy";
    const HARDWARE_B6_TEST: &str =
        "nanda_wave::l2_field::v13_typed_peak::tests::v10_hardware_b6_proxy";

    #[derive(Clone)]
    struct HardwarePreparedQuery {
        source_ordinal: usize,
        composite_identity_sha256: String,
        damaged_surface_sha256: String,
        retrieval_lanes_sha256: String,
        damaged_surface: String,
        lanes: Vec<Phase7dRetrievalLane>,
    }

    struct HardwareQueryOutcome {
        terminal_refs: Vec<u32>,
        expanded_product_states: usize,
        maximum_scratch_bytes: usize,
    }

    fn hardware_required_path(name: &str) -> Result<std::path::PathBuf, String> {
        std::env::var_os(name)
            .map(std::path::PathBuf::from)
            .ok_or_else(|| format!("missing required environment variable {name}"))
    }

    fn hardware_digest(bytes: &[u8]) -> String {
        let digest: [u8; 32] = Sha256::digest(bytes).into();
        hex(digest)
    }

    fn hardware_lanes_value(lanes: &[Phase7dRetrievalLane]) -> serde_json::Value {
        serde_json::Value::Array(
            lanes
                .iter()
                .map(|lane| {
                    serde_json::json!({
                        "symbols": lane.symbols.as_ref(),
                        "maximum_levenshtein_distance": lane.maximum_levenshtein_distance,
                    })
                })
                .collect(),
        )
    }

    fn hardware_prepare_queries(cases: &[ProofCase]) -> Result<Vec<HardwarePreparedQuery>, String> {
        let mut prepared = Vec::with_capacity(cases.len());
        let mut identities = BTreeSet::new();
        for (source_ordinal, case) in cases.iter().enumerate() {
            let oracle = Phase7dCertificateOracle::new(&case.damaged_surface)?;
            let lanes = oracle.retrieval_lanes();
            let lanes_bytes = serde_json::to_vec(&hardware_lanes_value(&lanes))
                .map_err(|error| error.to_string())?;
            let mut composite = Vec::with_capacity(case.class.len() + case.proof_identity.len() + 1);
            composite.extend_from_slice(case.class.as_bytes());
            composite.push(0);
            composite.extend_from_slice(case.proof_identity.as_bytes());
            let composite_identity_sha256 = hardware_digest(&composite);
            if !identities.insert(composite_identity_sha256.clone()) {
                return Err(format!(
                    "duplicate composite identity at source ordinal {source_ordinal}"
                ));
            }
            prepared.push(HardwarePreparedQuery {
                source_ordinal,
                composite_identity_sha256,
                damaged_surface_sha256: hardware_digest(case.damaged_surface.as_bytes()),
                retrieval_lanes_sha256: hardware_digest(&lanes_bytes),
                damaged_surface: case.damaged_surface.clone(),
                lanes,
            });
        }
        if prepared.len() != 382 {
            return Err(format!("expected 382 fixed queries, found {}", prepared.len()));
        }
        Ok(prepared)
    }

    fn hardware_schedule_value(
        prepared: &[HardwarePreparedQuery],
        v7_sha256: &str,
    ) -> serde_json::Value {
        serde_json::json!({
            "schema": HARDWARE_SCHEDULE_SCHEMA,
            "source": {
                "v7_sha256": v7_sha256,
                "phase7d_semantics_sha256": hex(phase7d_semantics_digest()),
                "composite_identity_encoding": "sha256(class_utf8 || NUL || proof_identity_json_utf8)",
                "damaged_surface_encoding": "sha256(utf8)",
                "retrieval_lanes_encoding": "sha256(compact_json([{symbols,maximum_levenshtein_distance}]))",
            },
            "partition": {
                "b5_requests": 382,
                "b6_workers": 20,
                "b6_chunk_size": 20,
            },
            "entries": prepared.iter().map(|query| {
                let worker = query.source_ordinal / 20;
                let start = worker * 20;
                serde_json::json!({
                    "source_ordinal": query.source_ordinal,
                    "composite_identity_sha256": query.composite_identity_sha256,
                    "damaged_surface_sha256": query.damaged_surface_sha256,
                    "retrieval_lanes_sha256": query.retrieval_lanes_sha256,
                    "b5_ordinal": query.source_ordinal,
                    "b6_worker": worker,
                    "b6_chunk_start": start,
                    "b6_chunk_end_exclusive": (start + 20).min(prepared.len()),
                })
            }).collect::<Vec<_>>(),
        })
    }

    fn hardware_write_new_json(path: &Path, value: &serde_json::Value) -> Result<(), String> {
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

    fn hardware_load_cases() -> Result<(Vec<ProofCase>, String), String> {
        let v7_path = hardware_required_path("LAY_V10_HW_V7")?;
        let v7_sha256 = hex(sha256_file(&v7_path)?);
        if v7_sha256 != EXPECTED_V7_SHA256 {
            return Err(format!("unexpected V7 SHA-256 {v7_sha256}"));
        }
        let source: serde_json::Value = serde_json::from_slice(
            &std::fs::read(&v7_path).map_err(|error| format!("{}: {error}", v7_path.display()))?,
        )
        .map_err(|error| error.to_string())?;
        let cases = parse_v7_cases(&source)?;
        Ok((cases, v7_sha256))
    }

    fn hardware_load_assets(
    ) -> Result<(RuntimeL2Package, V13DafsaView, Vec<ProofCase>, String), String> {
        let package_path = hardware_required_path("LAY_V10_HW_V13_PACKAGE")?;
        let sidecar_path = hardware_required_path("LAY_V10_HW_SIDECAR")?;
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
        let sidecar_sha256 = hex(sha256_file(&sidecar_path)?);
        if sidecar_sha256
            != "a1aa95be8c43fae8fedf02d3261dd1265dcbfbc8d618330fcd6449ae829df8cd"
        {
            return Err(format!("unexpected sidecar SHA-256 {sidecar_sha256}"));
        }
        let index = V13DafsaView::load(&sidecar_path, identity)?;
        let (cases, v7_sha256) = hardware_load_cases()?;
        Ok((package, index, cases, v7_sha256))
    }

    fn hardware_run_executor(
        index: &V13DafsaView,
        query: &HardwarePreparedQuery,
    ) -> Result<HardwareQueryOutcome, String> {
        let started = Instant::now();
        let mut terminal_refs = Vec::new();
        let mut expanded_product_states = 0_usize;
        let mut maximum_scratch_bytes = 0_usize;
        for lane in &query.lanes {
            let outcome = enumerate_lane::<BandedLevenshteinRow>(
                index,
                lane,
                SearchBudget::proof(),
                started,
                false,
            )?;
            if let Some(reason) = outcome.unresolved {
                return Err(format!(
                    "query {} unresolved: {reason}",
                    query.source_ordinal
                ));
            }
            expanded_product_states = expanded_product_states.saturating_add(outcome.expanded);
            maximum_scratch_bytes = maximum_scratch_bytes.max(outcome.maximum_scratch);
            terminal_refs.extend(outcome.form_refs);
        }
        terminal_refs.sort_unstable();
        terminal_refs.dedup();
        Ok(HardwareQueryOutcome {
            terminal_refs,
            expanded_product_states,
            maximum_scratch_bytes,
        })
    }

    fn hardware_terminal_digest(terminal_refs: &[u32]) -> String {
        let mut bytes = Vec::with_capacity(terminal_refs.len() * 4);
        for form_ref in terminal_refs {
            bytes.extend_from_slice(&form_ref.to_le_bytes());
        }
        hardware_digest(&bytes)
    }

    fn hardware_materialize_peaks(
        package: &RuntimeL2Package,
        query: &HardwarePreparedQuery,
        terminal_refs: &[u32],
    ) -> Result<Vec<V13TypedPeak>, String> {
        let oracle = Phase7dCertificateOracle::new(&query.damaged_surface)?;
        let mut peaks = Vec::new();
        for form_ref in terminal_refs {
            let surface = package
                .surface(*form_ref as usize)
                .ok_or_else(|| format!("missing V13 terminal {form_ref}"))?;
            let certificate_keys = oracle.certificate_keys(surface.as_ref())?;
            if !certificate_keys.is_empty() {
                peaks.push(V13TypedPeak {
                    form_ref: *form_ref,
                    certificate_keys,
                });
            }
        }
        Ok(peaks)
    }

    fn hardware_reject_target_keys(value: &serde_json::Value) -> Result<(), String> {
        match value {
            serde_json::Value::Object(fields) => {
                for (key, child) in fields {
                    if key.to_ascii_lowercase().contains("target") {
                        return Err(format!("schedule contains forbidden key {key}"));
                    }
                    hardware_reject_target_keys(child)?;
                }
            }
            serde_json::Value::Array(values) => {
                for child in values {
                    hardware_reject_target_keys(child)?;
                }
            }
            _ => {}
        }
        Ok(())
    }

    fn hardware_validate_schedule(
        path: &Path,
        prepared: &[HardwarePreparedQuery],
        v7_sha256: &str,
    ) -> Result<String, String> {
        let bytes = std::fs::read(path).map_err(|error| format!("{}: {error}", path.display()))?;
        let value: serde_json::Value =
            serde_json::from_slice(&bytes).map_err(|error| error.to_string())?;
        hardware_reject_target_keys(&value)?;
        if value.get("schema").and_then(serde_json::Value::as_str) != Some(HARDWARE_SCHEDULE_SCHEMA)
        {
            return Err("hardware schedule schema mismatch".to_string());
        }
        if value.pointer("/source/v7_sha256").and_then(serde_json::Value::as_str)
            != Some(v7_sha256)
        {
            return Err("hardware schedule V7 binding mismatch".to_string());
        }
        let expected = hardware_schedule_value(prepared, v7_sha256);
        if value != expected {
            return Err("hardware schedule does not match exact prepared queries".to_string());
        }
        Ok(hardware_digest(&bytes))
    }

    fn hardware_control_path(name: &str) -> Result<std::path::PathBuf, String> {
        let directory = hardware_required_path("LAY_V10_HW_CONTROL_DIR")?;
        if !directory.is_dir() {
            return Err(format!("control directory does not exist: {}", directory.display()));
        }
        Ok(directory.join(name))
    }

    fn hardware_write_marker(path: &Path, bytes: &[u8]) -> Result<(), String> {
        let mut output = std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(path)
            .map_err(|error| format!("{}: {error}", path.display()))?;
        use std::io::Write;
        output.write_all(bytes).map_err(|error| error.to_string())?;
        output.sync_all().map_err(|error| error.to_string())
    }

    fn hardware_wait_for_marker(name: &str) -> Result<(), String> {
        let path = hardware_control_path(name)?;
        wait_for_path(&path, Duration::from_secs(60))
    }

    #[cfg(target_os = "linux")]
    fn hardware_pin_current_thread(cpu: usize) -> Result<(), String> {
        let mut set: libc::cpu_set_t = unsafe { std::mem::zeroed() };
        unsafe {
            libc::CPU_ZERO(&mut set);
            libc::CPU_SET(cpu, &mut set);
        }
        let result = unsafe {
            libc::sched_setaffinity(
                0,
                std::mem::size_of::<libc::cpu_set_t>(),
                &set as *const libc::cpu_set_t,
            )
        };
        if result != 0 {
            return Err(format!(
                "sched_setaffinity({cpu}) failed: {}",
                std::io::Error::last_os_error()
            ));
        }
        Ok(())
    }

    #[cfg(not(target_os = "linux"))]
    fn hardware_pin_current_thread(_cpu: usize) -> Result<(), String> {
        Err("hardware proxy requires Linux affinity".to_string())
    }

    fn hardware_cpu_list(expected: usize) -> Result<Vec<usize>, String> {
        let raw = std::env::var("LAY_V10_HW_CPU_LIST")
            .map_err(|_| "missing LAY_V10_HW_CPU_LIST".to_string())?;
        let cpus = raw
            .split(',')
            .map(|value| value.parse::<usize>().map_err(|error| error.to_string()))
            .collect::<Result<Vec<_>, _>>()?;
        if cpus.len() != expected || cpus.iter().copied().collect::<BTreeSet<_>>().len() != expected
        {
            return Err(format!("expected {expected} unique CPUs, got {raw}"));
        }
        Ok(cpus)
    }

    fn hardware_query_receipt(query: &HardwarePreparedQuery, result: HardwareQueryOutcome) -> serde_json::Value {
        serde_json::json!({
            "source_ordinal": query.source_ordinal,
            "composite_identity_sha256": query.composite_identity_sha256,
            "terminal_refs_sha256": hardware_terminal_digest(&result.terminal_refs),
            "terminal_count": result.terminal_refs.len(),
            "expanded_product_states": result.expanded_product_states,
            "maximum_scratch_bytes": result.maximum_scratch_bytes,
        })
    }

    fn hardware_warmup(
        index: &V13DafsaView,
        prepared: &[HardwarePreparedQuery],
    ) -> Result<(), String> {
        for query in prepared {
            std::hint::black_box(hardware_run_executor(index, query)?);
        }
        Ok(())
    }

    #[test]
    #[ignore = "unmeasured B schedule freezer; one admitted execution only"]
    fn v10_hardware_schedule_freezer() {
        let result = (|| -> Result<(), String> {
            let output_path = hardware_required_path("LAY_V10_HW_FREEZER_OUTPUT")?;
            let receipt_path = hardware_required_path("LAY_V10_HW_FREEZER_RECEIPT")?;
            let (cases, v7_sha256) = hardware_load_cases()?;
            let prepared = hardware_prepare_queries(&cases)?;
            let schedule = hardware_schedule_value(&prepared, &v7_sha256);
            hardware_reject_target_keys(&schedule)?;
            hardware_write_new_json(&output_path, &schedule)?;
            let schedule_sha256 = hex(sha256_file(&output_path)?);
            hardware_write_new_json(
                &receipt_path,
                &serde_json::json!({
                    "schema": "lay.v10.hardware-schedule-freezer-receipt.v1",
                    "test": HARDWARE_FREEZER_TEST,
                    "schedule_schema": HARDWARE_SCHEDULE_SCHEMA,
                    "schedule_sha256": schedule_sha256,
                    "entries": prepared.len(),
                    "unique_composite_identities": prepared.iter()
                        .map(|query| query.composite_identity_sha256.as_str())
                        .collect::<BTreeSet<_>>().len(),
                    "perf_invoked": false,
                    "pmu_event_opened": false,
                    "search_enumeration_executed": false,
                    "latency_observed": false,
                    "runtime_authority_changed": false,
                    "installed_lay_changed": false,
                }),
            )
        })();
        assert!(result.is_ok(), "{}", result.unwrap_err());
    }

    #[test]
    #[ignore = "unmeasured semantic parity prerequisite for later B5/B6"]
    fn v10_hardware_semantic_parity() {
        let result = (|| -> Result<serde_json::Value, String> {
            let (package, index, cases, v7_sha256) = hardware_load_assets()?;
            let prepared = hardware_prepare_queries(&cases)?;
            let schedule_path = hardware_required_path("LAY_V10_HW_SCHEDULE")?;
            let schedule_sha256 = hardware_validate_schedule(&schedule_path, &prepared, &v7_sha256)?;
            let bindings = ProofBindingIndex::build(&package)?;
            let mut terminal_mismatches = 0_usize;
            let mut peak_mismatches = 0_usize;
            let mut completeness_mismatches = 0_usize;
            let mut work_mismatches = 0_usize;
            let mut full_row_mismatches = 0_usize;
            let mut target_form_retained = 0_usize;
            let mut target_lemma_retained = 0_usize;
            let mut false_certificates = 0_usize;
            let mut maximum_product_states = 0_usize;
            let mut maximum_scratch_bytes = 0_usize;
            for (case, query) in cases.iter().zip(&prepared) {
                let production = search_typed_peaks(
                    &index,
                    &package,
                    &case.damaged_surface,
                    SearchBudget::proof(),
                )?;
                let full_row = search_typed_peaks_full_row(
                    &index,
                    &package,
                    &case.damaged_surface,
                    SearchBudget::proof(),
                )?;
                let proxy = hardware_run_executor(&index, query)?;
                let proxy_peaks = hardware_materialize_peaks(&package, query, &proxy.terminal_refs)?;
                terminal_mismatches += usize::from(proxy.terminal_refs != production.retrieved_form_refs);
                peak_mismatches += usize::from(proxy_peaks != production.peaks);
                completeness_mismatches += usize::from(
                    production.completeness != SearchCompleteness::CertifiedExhaustive,
                );
                work_mismatches += usize::from(
                    proxy.expanded_product_states != production.expanded_product_states
                        || proxy.maximum_scratch_bytes != production.maximum_scratch_bytes,
                );
                full_row_mismatches += usize::from(
                    production.retrieved_form_refs != full_row.retrieved_form_refs
                        || production.peaks != full_row.peaks
                        || production.completeness != full_row.completeness
                        || production.expanded_product_states != full_row.expanded_product_states,
                );
                maximum_product_states =
                    maximum_product_states.max(production.expanded_product_states);
                maximum_scratch_bytes = maximum_scratch_bytes.max(production.maximum_scratch_bytes);
                let target_ref = package
                    .form_ref_for_surface(&case.target_surface)
                    .ok_or_else(|| format!("V13 lacks target {}", case.target_surface))?;
                let found_refs = production
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
                        && target_lemmas.iter().any(|lemma| found_lemmas.contains(lemma)),
                );
                for peak in &production.peaks {
                    let surface = package
                        .surface(peak.form_ref as usize)
                        .ok_or_else(|| format!("missing emitted form {}", peak.form_ref))?;
                    let keys = crate::nanda_wave::lexical_grokking::phase7d_certificate_keys(
                        &case.damaged_surface,
                        surface.as_ref(),
                    )?;
                    false_certificates += usize::from(
                        keys.is_empty() || keys != peak.certificate_keys,
                    );
                }
            }
            let pass = terminal_mismatches == 0
                && peak_mismatches == 0
                && completeness_mismatches == 0
                && work_mismatches == 0
                && full_row_mismatches == 0
                && target_form_retained == 382
                && target_lemma_retained == 382
                && false_certificates == 0
                && maximum_product_states == 35_590
                && maximum_scratch_bytes == 6_656;
            Ok(serde_json::json!({
                "schema": "lay.v10.hardware-diagnostic-proxy-parity.v1",
                "test": HARDWARE_PARITY_TEST,
                "verdict": if pass { "PASS" } else { "FAIL" },
                "schedule_sha256": schedule_sha256,
                "records": cases.len(),
                "terminal_mismatches": terminal_mismatches,
                "peak_mismatches": peak_mismatches,
                "completeness_mismatches": completeness_mismatches,
                "work_mismatches": work_mismatches,
                "full_row_mismatches": full_row_mismatches,
                "target_form_retained": target_form_retained,
                "target_lemma_retained": target_lemma_retained,
                "false_certificates": false_certificates,
                "maximum_product_states": maximum_product_states,
                "maximum_scratch_bytes": maximum_scratch_bytes,
                "historical_executable_identity_proven": false,
                "full_source_closure": "WATCH",
                "runtime_authority_changed": false,
                "installed_lay_changed": false,
            }))
        })();
        let receipt = result.unwrap_or_else(|error| serde_json::json!({
            "schema": "lay.v10.hardware-diagnostic-proxy-parity.v1",
            "verdict": "ERROR",
            "error": error,
            "runtime_authority_changed": false,
            "installed_lay_changed": false,
        }));
        let receipt_path = hardware_required_path("LAY_V10_HW_PARITY_RECEIPT")
            .expect("parity receipt path");
        hardware_write_new_json(&receipt_path, &receipt).expect("write parity receipt");
        assert_eq!(receipt.get("verdict").and_then(serde_json::Value::as_str), Some("PASS"));
    }

    fn hardware_run_b5() -> Result<serde_json::Value, String> {
        let (package, index, cases, v7_sha256) = hardware_load_assets()?;
        let prepared = hardware_prepare_queries(&cases)?;
        let schedule_path = hardware_required_path("LAY_V10_HW_SCHEDULE")?;
        let schedule_sha256 = hardware_validate_schedule(&schedule_path, &prepared, &v7_sha256)?;
        let cpus = hardware_cpu_list(1)?;
        hardware_pin_current_thread(cpus[0])?;
        hardware_warmup(&index, &prepared)?;
        hardware_write_marker(&hardware_control_path("subject-ready")?, b"ready\n")?;
        hardware_wait_for_marker("controller-enabled")?;
        let mut outputs = Vec::with_capacity(prepared.len());
        for query in &prepared {
            outputs.push(hardware_query_receipt(query, hardware_run_executor(&index, query)?));
        }
        hardware_write_marker(&hardware_control_path("subject-done")?, b"done\n")?;
        hardware_wait_for_marker("controller-disabled")?;
        std::hint::black_box(package);
        Ok(serde_json::json!({
            "schema": "lay.v10.hardware-diagnostic-proxy-b5.v1",
            "test": HARDWARE_B5_TEST,
            "route": "B5",
            "schedule_sha256": schedule_sha256,
            "cpu": cpus[0],
            "workers": 1,
            "requests": outputs.len(),
            "queries": outputs,
            "in_process_latency_observed": false,
            "runtime_authority_changed": false,
            "installed_lay_changed": false,
        }))
    }

    fn hardware_run_b6() -> Result<serde_json::Value, String> {
        let (package, index, cases, v7_sha256) = hardware_load_assets()?;
        let prepared = hardware_prepare_queries(&cases)?;
        let schedule_path = hardware_required_path("LAY_V10_HW_SCHEDULE")?;
        let schedule_sha256 = hardware_validate_schedule(&schedule_path, &prepared, &v7_sha256)?;
        let cpus = hardware_cpu_list(20)?;
        hardware_pin_current_thread(cpus[0])?;
        hardware_warmup(&index, &prepared)?;
        let ready = std::sync::Arc::new(std::sync::Barrier::new(20));
        let start = std::sync::Arc::new(std::sync::Barrier::new(20));
        let worker_results = std::thread::scope(|scope| -> Result<Vec<Vec<serde_json::Value>>, String> {
            let mut handles = Vec::new();
            for worker in 1..20 {
                let ready = std::sync::Arc::clone(&ready);
                let start = std::sync::Arc::clone(&start);
                let index = &index;
                let prepared = &prepared;
                let cpu = cpus[worker];
                handles.push(scope.spawn(move || -> Result<Vec<serde_json::Value>, String> {
                    let affinity = hardware_pin_current_thread(cpu);
                    ready.wait();
                    start.wait();
                    affinity?;
                    let range_start = worker * 20;
                    let range_end = (range_start + 20).min(prepared.len());
                    prepared[range_start..range_end]
                        .iter()
                        .map(|query| {
                            hardware_run_executor(index, query)
                                .map(|result| hardware_query_receipt(query, result))
                        })
                        .collect()
                }));
            }
            ready.wait();
            hardware_write_marker(&hardware_control_path("subject-ready")?, b"ready\n")?;
            hardware_wait_for_marker("controller-enabled")?;
            start.wait();
            let mut outputs = Vec::with_capacity(20);
            outputs.push(
                prepared[..20]
                    .iter()
                    .map(|query| {
                        hardware_run_executor(&index, query)
                            .map(|result| hardware_query_receipt(query, result))
                    })
                    .collect::<Result<Vec<_>, _>>()?,
            );
            for handle in handles {
                outputs.push(
                    handle
                        .join()
                        .map_err(|_| "B6 worker panicked".to_string())??,
                );
            }
            Ok(outputs)
        })?;
        hardware_write_marker(&hardware_control_path("subject-done")?, b"done\n")?;
        hardware_wait_for_marker("controller-disabled")?;
        std::hint::black_box(package);
        Ok(serde_json::json!({
            "schema": "lay.v10.hardware-diagnostic-proxy-b6.v1",
            "test": HARDWARE_B6_TEST,
            "route": "B6",
            "schedule_sha256": schedule_sha256,
            "cpus": cpus,
            "workers": 20,
            "chunk_size": 20,
            "requests": worker_results.iter().map(Vec::len).sum::<usize>(),
            "worker_queries": worker_results,
            "in_process_latency_observed": false,
            "runtime_authority_changed": false,
            "installed_lay_changed": false,
        }))
    }

    fn hardware_proxy_test(route: &str) {
        let result = match route {
            "B5" => hardware_run_b5(),
            "B6" => hardware_run_b6(),
            _ => Err(format!("unknown hardware proxy route {route}")),
        };
        let receipt = result.unwrap_or_else(|error| serde_json::json!({
            "schema": "lay.v10.hardware-diagnostic-proxy-error.v1",
            "route": route,
            "error": error,
            "runtime_authority_changed": false,
            "installed_lay_changed": false,
        }));
        let receipt_path = hardware_required_path("LAY_V10_HW_SUBJECT_RECEIPT")
            .expect("subject receipt path");
        hardware_write_new_json(&receipt_path, &receipt).expect("write subject receipt");
        assert!(receipt.get("error").is_none(), "{}", serde_json::to_string_pretty(&receipt).unwrap());
    }

    #[test]
    #[ignore = "later B5 executor-core subject; not admitted by B0-B2 preflight"]
    fn v10_hardware_b5_proxy() {
        hardware_proxy_test("B5");
    }

    #[test]
    #[ignore = "later B6 executor-core subject; not admitted by B0-B2 preflight"]
    fn v10_hardware_b6_proxy() {
        hardware_proxy_test("B6");
    }
