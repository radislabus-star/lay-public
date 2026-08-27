
    const C1_PARITY_TEST: &str =
        "nanda_wave::l2_field::v13_typed_peak::tests::v10_c1_semantic_parity";
    const C1_SINGLE_TEST: &str =
        "nanda_wave::l2_field::v13_typed_peak::tests::v10_c1_single_latency";
    const C1_TWENTY_TEST: &str =
        "nanda_wave::l2_field::v13_typed_peak::tests::v10_c1_twenty_worker_latency";
    const C1_V7_SHA256: &str =
        "33fded73e13f565bdc08a83bd440be67c80f6d85a196453050b09bc9f7ef28e4";
    const C1_SIDECAR_SHA256: &str =
        "a1aa95be8c43fae8fedf02d3261dd1265dcbfbc8d618330fcd6449ae829df8cd";
    const C1_SCHEDULE_SHA256: &str =
        "2f8346d97246ac3704434279739fa5ed27705465d628890a367c21875d879e78";
    const C1_SCHEDULE_SCHEMA: &str = "lay.v10.hardware-query-schedule.v1";
    const C1_SAMPLE_SCHEMA: &str = "lay.v10.c1-primitive-samples.v1";
    const C1_SAMPLE_BYTES: usize = 22;
    const C1_SINGLE_ROUNDS: usize = 100;
    const C1_TWENTY_ROUNDS: usize = 250;
    const C1_WORKERS: usize = 20;
    const C1_SINGLE_WORKER_SENTINEL: u8 = u8::MAX;
    const C1_FLAG_ERROR: u8 = 1;
    const C1_FLAG_UNRESOLVED: u8 = 2;

    #[derive(Clone)]
    struct C1ScheduleEntry {
        source_ordinal: usize,
        composite_identity_sha256: String,
        damaged_surface_sha256: String,
        retrieval_lanes_sha256: String,
        worker: usize,
        chunk_start: usize,
        chunk_end: usize,
    }

    struct C1Inputs {
        package: RuntimeL2Package,
        index: V13DafsaView,
        cases: Vec<ProofCase>,
        schedule: Vec<C1ScheduleEntry>,
    }

    #[derive(Clone, Copy)]
    struct C1Sample {
        query_ordinal: u16,
        round: u16,
        worker_id: u8,
        flags: u8,
        search_elapsed_us: u64,
        total_elapsed_us: u64,
    }

    fn c1_required_path(name: &str) -> Result<std::path::PathBuf, String> {
        std::env::var_os(name)
            .map(std::path::PathBuf::from)
            .ok_or_else(|| format!("required environment path {name} is absent"))
    }

    fn c1_required_string(name: &str) -> Result<String, String> {
        std::env::var(name).map_err(|error| format!("required environment {name}: {error}"))
    }

    fn c1_write_new(path: &Path, bytes: &[u8]) -> Result<(), String> {
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

    fn c1_write_json(path: &Path, value: &serde_json::Value) -> Result<(), String> {
        let mut bytes = serde_json::to_vec_pretty(value).map_err(|error| error.to_string())?;
        bytes.push(b'\n');
        c1_write_new(path, &bytes)
    }

    fn c1_sha256_text(value: &[u8]) -> String {
        hex(Sha256::digest(value).into())
    }

    fn c1_compact_lanes(lanes: &[Phase7dRetrievalLane]) -> Result<Vec<u8>, String> {
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

    fn c1_schedule_entries(value: &serde_json::Value) -> Result<Vec<C1ScheduleEntry>, String> {
        if value.get("schema").and_then(serde_json::Value::as_str) != Some(C1_SCHEDULE_SCHEMA) {
            return Err("C1 schedule schema mismatch".to_string());
        }
        let partition = value
            .get("partition")
            .and_then(serde_json::Value::as_object)
            .ok_or_else(|| "C1 schedule partition is absent".to_string())?;
        if partition.get("b5_requests").and_then(serde_json::Value::as_u64) != Some(382)
            || partition.get("b6_workers").and_then(serde_json::Value::as_u64) != Some(20)
            || partition.get("b6_chunk_size").and_then(serde_json::Value::as_u64) != Some(20)
        {
            return Err("C1 schedule partition mismatch".to_string());
        }
        value
            .get("entries")
            .and_then(serde_json::Value::as_array)
            .ok_or_else(|| "C1 schedule entries are absent".to_string())?
            .iter()
            .map(|entry| {
                let number = |name: &str| {
                    entry
                        .get(name)
                        .and_then(serde_json::Value::as_u64)
                        .and_then(|value| usize::try_from(value).ok())
                        .ok_or_else(|| format!("C1 schedule entry lacks {name}"))
                };
                let string = |name: &str| {
                    entry
                        .get(name)
                        .and_then(serde_json::Value::as_str)
                        .map(str::to_string)
                        .ok_or_else(|| format!("C1 schedule entry lacks {name}"))
                };
                Ok(C1ScheduleEntry {
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

    fn c1_validate_schedule(
        cases: &[ProofCase],
        schedule: &[C1ScheduleEntry],
    ) -> Result<(), String> {
        if cases.len() != 382 || schedule.len() != cases.len() {
            return Err("C1 schedule denominator mismatch".to_string());
        }
        for (ordinal, (case, entry)) in cases.iter().zip(schedule).enumerate() {
            if entry.source_ordinal != ordinal {
                return Err(format!("C1 source ordinal mismatch at {ordinal}"));
            }
            let composite = [case.class.as_bytes(), &[0], case.proof_identity.as_bytes()].concat();
            if c1_sha256_text(&composite) != entry.composite_identity_sha256 {
                return Err(format!("C1 composite identity mismatch at {ordinal}"));
            }
            if c1_sha256_text(case.damaged_surface.as_bytes()) != entry.damaged_surface_sha256 {
                return Err(format!("C1 damaged surface mismatch at {ordinal}"));
            }
            let lanes = Phase7dCertificateOracle::new(&case.damaged_surface)?.retrieval_lanes();
            if c1_sha256_text(&c1_compact_lanes(&lanes)?) != entry.retrieval_lanes_sha256 {
                return Err(format!("C1 retrieval lanes mismatch at {ordinal}"));
            }
            let worker = ordinal / 20;
            let start = worker * 20;
            let end = (start + 20).min(cases.len());
            if entry.worker != worker || entry.chunk_start != start || entry.chunk_end != end {
                return Err(format!("C1 worker partition mismatch at {ordinal}"));
            }
        }
        Ok(())
    }

    fn c1_load_inputs() -> Result<C1Inputs, String> {
        let package_path = c1_required_path("LAY_V10_C1_PACKAGE")?;
        let sidecar_path = c1_required_path("LAY_V10_C1_SIDECAR")?;
        let v7_path = c1_required_path("LAY_V10_C1_V7")?;
        let schedule_path = c1_required_path("LAY_V10_C1_SCHEDULE")?;

        let package_bytes = std::fs::metadata(&package_path)
            .map_err(|error| format!("{}: {error}", package_path.display()))?
            .len();
        let package_sha256 = sha256_file(&package_path)?;
        if hex(package_sha256) != EXPECTED_V13_SHA256 || package_bytes != EXPECTED_V13_BYTES {
            return Err("C1 V13 package identity mismatch".to_string());
        }
        if hex(sha256_file(&sidecar_path)?) != C1_SIDECAR_SHA256 {
            return Err("C1 sidecar identity mismatch".to_string());
        }
        if hex(sha256_file(&v7_path)?) != C1_V7_SHA256 {
            return Err("C1 V7 identity mismatch".to_string());
        }
        if hex(sha256_file(&schedule_path)?) != C1_SCHEDULE_SHA256 {
            return Err("C1 schedule identity mismatch".to_string());
        }

        let package = RuntimeL2Package::load(&package_path)?;
        if package.form_count() != EXPECTED_V13_FORMS {
            return Err("C1 V13 form count mismatch".to_string());
        }
        let identity = V13Identity {
            package_sha256,
            package_bytes,
            form_count: u32::try_from(package.form_count())
                .map_err(|_| "C1 form count exceeds u32".to_string())?,
            binding_count: u32::try_from(package.binding_count())
                .map_err(|_| "C1 binding count exceeds u32".to_string())?,
        };
        let index = V13DafsaView::load(&sidecar_path, identity)?;
        let v7: serde_json::Value = serde_json::from_slice(
            &std::fs::read(&v7_path)
                .map_err(|error| format!("{}: {error}", v7_path.display()))?,
        )
        .map_err(|error| error.to_string())?;
        let cases = parse_v7_cases(&v7)?;
        let schedule_value: serde_json::Value = serde_json::from_slice(
            &std::fs::read(&schedule_path)
                .map_err(|error| format!("{}: {error}", schedule_path.display()))?,
        )
        .map_err(|error| error.to_string())?;
        let schedule = c1_schedule_entries(&schedule_value)?;
        c1_validate_schedule(&cases, &schedule)?;
        Ok(C1Inputs {
            package,
            index,
            cases,
            schedule,
        })
    }

    #[cfg(target_os = "linux")]
    fn c1_pin_current_thread(cpu: usize) -> Result<(), String> {
        unsafe {
            let mut set: libc::cpu_set_t = std::mem::zeroed();
            libc::CPU_ZERO(&mut set);
            libc::CPU_SET(cpu, &mut set);
            let result = libc::sched_setaffinity(
                0,
                std::mem::size_of::<libc::cpu_set_t>(),
                &set,
            );
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
    fn c1_pin_current_thread(_cpu: usize) -> Result<(), String> {
        Err("C1 requires Linux affinity".to_string())
    }

    fn c1_current_affinity() -> Result<Vec<usize>, String> {
        #[cfg(target_os = "linux")]
        unsafe {
            let mut set: libc::cpu_set_t = std::mem::zeroed();
            libc::CPU_ZERO(&mut set);
            let result = libc::sched_getaffinity(
                0,
                std::mem::size_of::<libc::cpu_set_t>(),
                &mut set,
            );
            if result != 0 {
                return Err(format!("sched_getaffinity failed: {}", std::io::Error::last_os_error()));
            }
            Ok((0..libc::CPU_SETSIZE as usize)
                .filter(|cpu| libc::CPU_ISSET(*cpu, &set))
                .collect())
        }
        #[cfg(not(target_os = "linux"))]
        {
            Err("C1 requires Linux affinity".to_string())
        }
    }

    fn c1_parse_cpus() -> Result<Vec<usize>, String> {
        let cpus = c1_required_string("LAY_V10_C1_CPUS")?
            .split(',')
            .map(|value| value.parse::<usize>().map_err(|error| error.to_string()))
            .collect::<Result<Vec<_>, _>>()?;
        if cpus.is_empty() || cpus.iter().copied().collect::<BTreeSet<_>>().len() != cpus.len() {
            return Err("C1 CPU list is empty or contains duplicates".to_string());
        }
        Ok(cpus)
    }

    fn c1_thread_migrations() -> Result<u64, String> {
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

    fn c1_sample(
        index: &V13DafsaView,
        package: &RuntimeL2Package,
        case: &ProofCase,
        query_ordinal: usize,
        round: usize,
        worker_id: u8,
    ) -> C1Sample {
        let outcome = search_typed_peaks(
            index,
            package,
            &case.damaged_surface,
            SearchBudget::proof(),
        );
        let sample = match &outcome {
            Ok(result) => C1Sample {
                query_ordinal: query_ordinal as u16,
                round: round as u16,
                worker_id,
                flags: if result.completeness == SearchCompleteness::CertifiedExhaustive {
                    0
                } else {
                    C1_FLAG_UNRESOLVED
                },
                search_elapsed_us: result.search_elapsed_us,
                total_elapsed_us: result.total_elapsed_us,
            },
            Err(_) => C1Sample {
                query_ordinal: query_ordinal as u16,
                round: round as u16,
                worker_id,
                flags: C1_FLAG_ERROR,
                search_elapsed_us: 0,
                total_elapsed_us: 0,
            },
        };
        std::hint::black_box(&outcome);
        drop(outcome);
        sample
    }

    fn c1_warmup(
        index: &V13DafsaView,
        package: &RuntimeL2Package,
        cases: &[ProofCase],
    ) -> Result<(), String> {
        for case in cases {
            let result = search_typed_peaks(
                index,
                package,
                &case.damaged_surface,
                SearchBudget::proof(),
            )?;
            if result.completeness != SearchCompleteness::CertifiedExhaustive {
                return Err("C1 warmup was unresolved".to_string());
            }
            std::hint::black_box(&result);
        }
        Ok(())
    }

    fn c1_encode_samples(samples: &[C1Sample]) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(samples.len() * C1_SAMPLE_BYTES);
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

    fn c1_sample_summary(samples: &[C1Sample]) -> serde_json::Value {
        serde_json::json!({
            "records": samples.len(),
            "errors": samples.iter().filter(|sample| sample.flags & C1_FLAG_ERROR != 0).count(),
            "unresolved": samples.iter().filter(|sample| sample.flags & C1_FLAG_UNRESOLVED != 0).count(),
            "sample_bytes": C1_SAMPLE_BYTES,
            "schema": C1_SAMPLE_SCHEMA,
        })
    }

    fn c1_output_dir() -> Result<std::path::PathBuf, String> {
        let path = c1_required_path("LAY_V10_C1_OUTPUT")?;
        if !path.is_dir() {
            return Err(format!("C1 output directory is absent: {}", path.display()));
        }
        Ok(path)
    }

    fn c1_run_parity() -> Result<serde_json::Value, String> {
        let inputs = c1_load_inputs()?;
        let bindings = ProofBindingIndex::build(&inputs.package)?;
        let mut terminal_mismatches = 0_usize;
        let mut peak_mismatches = 0_usize;
        let mut completeness_mismatches = 0_usize;
        let mut work_mismatches = 0_usize;
        let mut target_form_retained = 0_usize;
        let mut target_lemma_retained = 0_usize;
        let mut false_certificates = 0_usize;
        let mut maximum_product_states = 0_usize;
        let mut maximum_scratch_bytes = 0_usize;

        for case in &inputs.cases {
            let result = search_typed_peaks(
                &inputs.index,
                &inputs.package,
                &case.damaged_surface,
                SearchBudget::proof(),
            )?;
            let full = search_typed_peaks_full_row(
                &inputs.index,
                &inputs.package,
                &case.damaged_surface,
                SearchBudget::proof(),
            )?;
            terminal_mismatches += usize::from(result.retrieved_form_refs != full.retrieved_form_refs);
            peak_mismatches += usize::from(result.peaks != full.peaks);
            completeness_mismatches += usize::from(result.completeness != full.completeness);
            work_mismatches += usize::from(result.expanded_product_states != full.expanded_product_states);
            maximum_product_states = maximum_product_states.max(result.expanded_product_states);
            maximum_scratch_bytes = maximum_scratch_bytes.max(result.maximum_scratch_bytes);

            let frozen_refs = result
                .peaks
                .iter()
                .map(|peak| peak.form_ref)
                .collect::<BTreeSet<_>>();
            let target_ref = inputs
                .package
                .form_ref_for_surface(&case.target_surface)
                .ok_or_else(|| format!("C1 V13 lacks target {}", case.target_surface))?;
            target_form_retained += usize::from(frozen_refs.contains(&target_ref));
            let target_lemmas = bindings.lemma_ids(&inputs.package, target_ref);
            let found_lemmas = frozen_refs
                .iter()
                .flat_map(|form_ref| bindings.lemma_ids(&inputs.package, *form_ref))
                .collect::<BTreeSet<_>>();
            target_lemma_retained += usize::from(
                !target_lemmas.is_empty()
                    && target_lemmas.iter().any(|lemma| found_lemmas.contains(lemma)),
            );
            for peak in &result.peaks {
                let surface = inputs
                    .package
                    .surface(peak.form_ref as usize)
                    .ok_or_else(|| format!("C1 missing emitted form {}", peak.form_ref))?;
                let verified = crate::nanda_wave::lexical_grokking::phase7d_certificate_keys(
                    &case.damaged_surface,
                    surface.as_ref(),
                )?;
                false_certificates += usize::from(
                    verified.is_empty() || verified != peak.certificate_keys,
                );
            }
        }
        let pass = terminal_mismatches == 0
            && peak_mismatches == 0
            && completeness_mismatches == 0
            && work_mismatches == 0
            && target_form_retained == 382
            && target_lemma_retained == 382
            && false_certificates == 0
            && maximum_product_states == 35_590
            && maximum_scratch_bytes == 6_656;
        Ok(serde_json::json!({
            "schema": "lay.v10.c1-semantic-parity.v1",
            "verdict": if pass { "PASS" } else { "FAIL" },
            "test": C1_PARITY_TEST,
            "records": inputs.cases.len(),
            "schedule_records": inputs.schedule.len(),
            "terminal_mismatches": terminal_mismatches,
            "peak_mismatches": peak_mismatches,
            "completeness_mismatches": completeness_mismatches,
            "work_mismatches": work_mismatches,
            "target_form_retained": target_form_retained,
            "target_lemma_retained": target_lemma_retained,
            "false_certificates": false_certificates,
            "maximum_product_states": maximum_product_states,
            "maximum_scratch_bytes": maximum_scratch_bytes,
            "latency_authority": false,
            "runtime_authority_changed": false,
            "installed_lay_changed": false,
        }))
    }

    fn c1_run_single() -> Result<serde_json::Value, String> {
        let inputs = c1_load_inputs()?;
        let cpus = c1_parse_cpus()?;
        if cpus.len() != 1 {
            return Err("C1 S requires exactly one CPU".to_string());
        }
        c1_pin_current_thread(cpus[0])?;
        if c1_current_affinity()? != cpus {
            return Err("C1 S affinity mismatch".to_string());
        }
        let migrations_before = c1_thread_migrations()?;
        c1_warmup(&inputs.index, &inputs.package, &inputs.cases)?;
        let expected = inputs.cases.len() * C1_SINGLE_ROUNDS;
        let mut samples = Vec::with_capacity(expected);
        let capacity = samples.capacity();
        for round in 0..C1_SINGLE_ROUNDS {
            for (query_ordinal, case) in inputs.cases.iter().enumerate() {
                samples.push(c1_sample(
                    &inputs.index,
                    &inputs.package,
                    case,
                    query_ordinal,
                    round,
                    C1_SINGLE_WORKER_SENTINEL,
                ));
            }
        }
        if samples.len() != expected || samples.capacity() != capacity {
            return Err("C1 S sample capacity changed".to_string());
        }
        let migrations_after = c1_thread_migrations()?;
        let output = c1_output_dir()?;
        let raw = c1_encode_samples(&samples);
        c1_write_new(&output.join("samples.bin"), &raw)?;
        Ok(serde_json::json!({
            "schema": "lay.v10.c1-single-process.v1",
            "test": C1_SINGLE_TEST,
            "run_id": c1_required_string("LAY_V10_C1_RUN_ID")?,
            "rounds": C1_SINGLE_ROUNDS,
            "queries": inputs.cases.len(),
            "warmup_queries": inputs.cases.len(),
            "cpus": cpus,
            "sample_capacity": capacity,
            "thread_migrations_before": migrations_before,
            "thread_migrations_after": migrations_after,
            "thread_migrations_delta": migrations_after.saturating_sub(migrations_before),
            "samples_sha256": c1_sha256_text(&raw),
            "samples": c1_sample_summary(&samples),
            "production_timing_authority": ["search_elapsed_us", "total_elapsed_us"],
            "external_call_wall_authority": false,
            "runtime_authority_changed": false,
            "installed_lay_changed": false,
        }))
    }

    fn c1_run_twenty() -> Result<serde_json::Value, String> {
        let inputs = c1_load_inputs()?;
        let cpus = c1_parse_cpus()?;
        if cpus.len() != C1_WORKERS {
            return Err("C1 T requires exactly twenty CPUs".to_string());
        }
        let start = std::sync::Arc::new(std::sync::Barrier::new(C1_WORKERS + 1));
        let end = std::sync::Arc::new(std::sync::Barrier::new(C1_WORKERS + 1));
        let worker_outputs = std::thread::scope(|scope| {
            let handles = (0..C1_WORKERS)
                .map(|worker| {
                    let start = std::sync::Arc::clone(&start);
                    let end = std::sync::Arc::clone(&end);
                    let cpu = cpus[worker];
                    let chunk_start = worker * 20;
                    let chunk_end = (chunk_start + 20).min(inputs.cases.len());
                    let cases = &inputs.cases[chunk_start..chunk_end];
                    let index = &inputs.index;
                    let package = &inputs.package;
                    scope.spawn(move || -> Result<(usize, Vec<C1Sample>, Vec<usize>, u64), String> {
                        let mut failure = c1_pin_current_thread(cpu).err();
                        let affinity = c1_current_affinity().unwrap_or_default();
                        if failure.is_none() && affinity != [cpu] {
                            failure = Some(format!("C1 T worker {worker} affinity mismatch"));
                        }
                        let migrations_before = c1_thread_migrations().unwrap_or_default();
                        let expected = cases.len() * C1_TWENTY_ROUNDS;
                        let mut samples = Vec::with_capacity(expected);
                        let capacity = samples.capacity();
                        for burst in 0..=C1_TWENTY_ROUNDS {
                            start.wait();
                            if failure.is_none() {
                                for (offset, case) in cases.iter().enumerate() {
                                    if burst == 0 {
                                        match search_typed_peaks(
                                            index,
                                            package,
                                            &case.damaged_surface,
                                            SearchBudget::proof(),
                                        ) {
                                            Ok(result)
                                                if result.completeness
                                                    == SearchCompleteness::CertifiedExhaustive =>
                                            {
                                                std::hint::black_box(&result);
                                            }
                                            Ok(_) => {
                                                failure = Some(format!(
                                                    "C1 T worker {worker} warmup unresolved"
                                                ));
                                                break;
                                            }
                                            Err(error) => {
                                                failure = Some(format!(
                                                    "C1 T worker {worker} warmup: {error}"
                                                ));
                                                break;
                                            }
                                        }
                                    } else {
                                        samples.push(c1_sample(
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
                        let migrations_after = c1_thread_migrations()?;
                        if samples.len() != expected || samples.capacity() != capacity {
                            return Err(format!("C1 T worker {worker} capacity changed"));
                        }
                        Ok((
                            capacity,
                            samples,
                            affinity,
                            migrations_after.saturating_sub(migrations_before),
                        ))
                    })
                })
                .collect::<Vec<_>>();
            for _ in 0..=C1_TWENTY_ROUNDS {
                start.wait();
                end.wait();
            }
            handles
                .into_iter()
                .map(|handle| handle.join().map_err(|_| "C1 T worker panicked".to_string())?)
                .collect::<Result<Vec<_>, String>>()
        })?;

        let mut samples = Vec::with_capacity(inputs.cases.len() * C1_TWENTY_ROUNDS);
        let mut capacities = Vec::with_capacity(C1_WORKERS);
        let mut affinities = Vec::with_capacity(C1_WORKERS);
        let mut migration_deltas = Vec::with_capacity(C1_WORKERS);
        for (capacity, mut worker_samples, affinity, migrations) in worker_outputs {
            capacities.push(capacity);
            affinities.push(affinity);
            migration_deltas.push(migrations);
            samples.append(&mut worker_samples);
        }
        if samples.len() != inputs.cases.len() * C1_TWENTY_ROUNDS {
            return Err("C1 T pooled sample count mismatch".to_string());
        }
        let output = c1_output_dir()?;
        let raw = c1_encode_samples(&samples);
        c1_write_new(&output.join("samples.bin"), &raw)?;
        Ok(serde_json::json!({
            "schema": "lay.v10.c1-twenty-worker-process.v1",
            "test": C1_TWENTY_TEST,
            "run_id": c1_required_string("LAY_V10_C1_RUN_ID")?,
            "rounds": C1_TWENTY_ROUNDS,
            "queries": inputs.cases.len(),
            "workers": C1_WORKERS,
            "chunk_size": 20,
            "warmup_bursts": 1,
            "start_barriers": C1_TWENTY_ROUNDS + 1,
            "end_barriers": C1_TWENTY_ROUNDS + 1,
            "cpus": cpus,
            "worker_affinities": affinities,
            "worker_capacities": capacities,
            "worker_migration_deltas": migration_deltas,
            "samples_sha256": c1_sha256_text(&raw),
            "samples": c1_sample_summary(&samples),
            "production_timing_authority": ["search_elapsed_us", "total_elapsed_us"],
            "external_call_wall_authority": false,
            "runtime_authority_changed": false,
            "installed_lay_changed": false,
        }))
    }

    fn c1_subject(test: &str) {
        let output = c1_required_path("LAY_V10_C1_OUTPUT").expect("C1 output path");
        let result = match test {
            C1_PARITY_TEST => c1_run_parity(),
            C1_SINGLE_TEST => c1_run_single(),
            C1_TWENTY_TEST => c1_run_twenty(),
            _ => Err(format!("unknown C1 test {test}")),
        };
        let receipt = result.unwrap_or_else(|error| {
            serde_json::json!({
                "schema": "lay.v10.c1-subject-error.v1",
                "test": test,
                "error": error,
                "runtime_authority_changed": false,
                "installed_lay_changed": false,
            })
        });
        c1_write_json(&output.join("SUBJECT_RECEIPT.json"), &receipt)
            .expect("write C1 subject receipt");
        assert!(
            receipt.get("error").is_none(),
            "{}",
            serde_json::to_string_pretty(&receipt).unwrap()
        );
        if test == C1_PARITY_TEST {
            assert_eq!(receipt.get("verdict").and_then(serde_json::Value::as_str), Some("PASS"));
        }
    }

    #[test]
    #[ignore = "C1 separate semantic prerequisite; admitted only by C1 preflight"]
    fn v10_c1_semantic_parity() {
        c1_subject(C1_PARITY_TEST);
    }

    #[test]
    #[ignore = "C1 direct single-process latency; admitted only after parity and clean admission"]
    fn v10_c1_single_latency() {
        c1_subject(C1_SINGLE_TEST);
    }

    #[test]
    #[ignore = "C1 twenty-worker fixed-shard burst; admitted only after parity and clean admission"]
    fn v10_c1_twenty_worker_latency() {
        c1_subject(C1_TWENTY_TEST);
    }
