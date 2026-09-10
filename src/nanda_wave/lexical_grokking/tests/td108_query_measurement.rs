use std::alloc::{GlobalAlloc, Layout, System};
use std::cell::Cell;
use std::hint::black_box;
use std::mem::{size_of, size_of_val};
use std::time::Instant;

use serde::Serialize;
use sha2::{Digest, Sha256};

use super::super::atoms::normalize_lexical_surface;
use super::super::typed_edit_traversal::{
    phase7d_semantics_digest, Phase7dCertificateOracle, Phase7dRetrievalLane,
};
use crate::dict::{detect_direction, project_char};

const BASE_COMMIT: &str = "20445b502856de98f54873819b6edd7f95b66250";
const TYPED_SOURCE_SHA256: &str =
    "c88910fe9080556589c9f5b0fbd8725903f9e5bcd06e832098dfd834d87a6382";
const WARMUP_ITERATIONS: usize = 512;
const LATENCY_SAMPLES: usize = 4_097;

#[derive(Clone, Copy)]
struct MeasurementCase {
    id: &'static str,
    raw: &'static str,
    expected: &'static str,
}

const CASES: &[MeasurementCase] = &[
    MeasurementCase {
        id: "ascii",
        raw: " Hello ",
        expected: "hello",
    },
    MeasurementCase {
        id: "cyrillic",
        raw: " ПрИвЕт ",
        expected: "привет",
    },
    MeasurementCase {
        id: "mixed_layout",
        raw: "ghbdtn",
        expected: "привет",
    },
    MeasurementCase {
        id: "mixed_script",
        raw: "Приvet",
        expected: "приvet",
    },
    MeasurementCase {
        id: "punctuation_only",
        raw: ",!?",
        expected: "б",
    },
    MeasurementCase {
        id: "leading_punctuation",
        raw: ",hello",
        expected: "hello",
    },
    MeasurementCase {
        id: "trailing_punctuation",
        raw: "hello?!",
        expected: "hello",
    },
    MeasurementCase {
        id: "physical_comma_layout",
        raw: "cj,frf",
        expected: "собака",
    },
    MeasurementCase {
        id: "unicode_lowercase_expansion",
        raw: "İ",
        expected: "i\u{307}",
    },
    MeasurementCase {
        id: "empty_normalized",
        raw: "   ,!?   ",
        expected: "",
    },
];

#[derive(Clone, Copy, Debug, Default, Serialize)]
struct AllocationCounts {
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
}

impl AllocationCounts {
    fn conservation_expected(self) -> i64 {
        let value = i128::from(self.allocated_bytes)
            + i128::from(self.zeroed_bytes)
            + i128::from(self.realloc_new_bytes)
            - i128::from(self.realloc_old_bytes)
            - i128::from(self.deallocated_bytes);
        i64::try_from(value).expect("TD-108 allocation counter must fit i64")
    }

    fn assert_conservation(self, case_id: &str, operation: &str) {
        assert_eq!(
            self.current_live_requested_bytes,
            self.conservation_expected(),
            "TD-108 allocation conservation failed for {case_id}/{operation}"
        );
        assert!(
            self.current_live_requested_bytes >= 0,
            "TD-108 measured scope released pre-existing memory for {case_id}/{operation}"
        );
    }
}

thread_local! {
    static ALLOCATION_COUNTING_ENABLED: Cell<bool> = const { Cell::new(false) };
    static ALLOCATION_COUNTS: Cell<AllocationCounts> = const {
        Cell::new(AllocationCounts {
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
        })
    };
}

struct CountingAllocator;

fn update_allocation_counts(update: impl FnOnce(&mut AllocationCounts)) {
    ALLOCATION_COUNTING_ENABLED.with(|enabled| {
        if enabled.get() {
            ALLOCATION_COUNTS.with(|counts| {
                let mut value = counts.get();
                update(&mut value);
                value.maximum_live_requested_bytes = value
                    .maximum_live_requested_bytes
                    .max(value.current_live_requested_bytes);
                counts.set(value);
            });
        }
    });
}

unsafe impl GlobalAlloc for CountingAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let pointer = unsafe { GlobalAlloc::alloc(&System, layout) };
        if !pointer.is_null() {
            update_allocation_counts(|counts| {
                counts.alloc_calls = counts.alloc_calls.saturating_add(1);
                counts.allocated_bytes =
                    counts.allocated_bytes.saturating_add(layout.size() as u64);
                counts.current_live_requested_bytes = counts
                    .current_live_requested_bytes
                    .saturating_add(layout.size() as i64);
            });
        }
        pointer
    }

    unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
        let pointer = unsafe { GlobalAlloc::alloc_zeroed(&System, layout) };
        if !pointer.is_null() {
            update_allocation_counts(|counts| {
                counts.alloc_zeroed_calls = counts.alloc_zeroed_calls.saturating_add(1);
                counts.zeroed_bytes = counts.zeroed_bytes.saturating_add(layout.size() as u64);
                counts.current_live_requested_bytes = counts
                    .current_live_requested_bytes
                    .saturating_add(layout.size() as i64);
            });
        }
        pointer
    }

    unsafe fn dealloc(&self, pointer: *mut u8, layout: Layout) {
        update_allocation_counts(|counts| {
            counts.dealloc_calls = counts.dealloc_calls.saturating_add(1);
            counts.deallocated_bytes = counts
                .deallocated_bytes
                .saturating_add(layout.size() as u64);
            counts.current_live_requested_bytes = counts
                .current_live_requested_bytes
                .saturating_sub(layout.size() as i64);
        });
        unsafe { GlobalAlloc::dealloc(&System, pointer, layout) };
    }

    unsafe fn realloc(&self, pointer: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        let output = unsafe { GlobalAlloc::realloc(&System, pointer, layout, new_size) };
        if !output.is_null() {
            update_allocation_counts(|counts| {
                counts.realloc_calls = counts.realloc_calls.saturating_add(1);
                counts.realloc_old_bytes = counts
                    .realloc_old_bytes
                    .saturating_add(layout.size() as u64);
                counts.realloc_new_bytes = counts.realloc_new_bytes.saturating_add(new_size as u64);
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
static GLOBAL_ALLOCATOR: CountingAllocator = CountingAllocator;

struct DisableAllocationCounting;

impl Drop for DisableAllocationCounting {
    fn drop(&mut self) {
        ALLOCATION_COUNTING_ENABLED.with(|enabled| enabled.set(false));
    }
}

fn with_allocation_counts<T>(operation: impl FnOnce() -> T) -> (T, AllocationCounts) {
    ALLOCATION_COUNTING_ENABLED.with(|enabled| {
        assert!(!enabled.get(), "nested TD-108 allocation measurement");
    });
    ALLOCATION_COUNTS.with(|counts| counts.set(AllocationCounts::default()));
    ALLOCATION_COUNTING_ENABLED.with(|enabled| enabled.set(true));
    let guard = DisableAllocationCounting;
    let output = operation();
    drop(guard);
    let counts = ALLOCATION_COUNTS.with(Cell::get);
    (output, counts)
}

#[derive(Debug, Serialize)]
struct TimingDistribution {
    samples: usize,
    minimum_ns_per_operation: u64,
    p50_ns_per_operation: u64,
    p95_ns_per_operation: u64,
    p99_ns_per_operation: u64,
    maximum_ns_per_operation: u64,
}

fn percentile(sorted: &[u64], percentile: usize) -> u64 {
    let rank = (percentile * sorted.len()).div_ceil(100).max(1);
    sorted[rank - 1]
}

fn measure_latency<T>(mut operation: impl FnMut() -> T) -> TimingDistribution {
    for _ in 0..WARMUP_ITERATIONS {
        black_box(operation());
    }

    let mut samples = Vec::with_capacity(LATENCY_SAMPLES);
    for _ in 0..LATENCY_SAMPLES {
        let started = Instant::now();
        let output = black_box(operation());
        let elapsed = started.elapsed().as_nanos();
        drop(output);
        samples.push(u64::try_from(elapsed).unwrap_or(u64::MAX));
    }
    samples.sort_unstable();

    TimingDistribution {
        samples: samples.len(),
        minimum_ns_per_operation: samples[0],
        p50_ns_per_operation: percentile(&samples, 50),
        p95_ns_per_operation: percentile(&samples, 95),
        p99_ns_per_operation: percentile(&samples, 99),
        maximum_ns_per_operation: samples[samples.len() - 1],
    }
}

#[derive(Debug, Serialize)]
struct LaneSnapshot {
    ordinal: usize,
    maximum_levenshtein_distance: u8,
    symbols: Vec<u32>,
    payload_bytes: usize,
}

fn lane_snapshots(lanes: &[Phase7dRetrievalLane]) -> Vec<LaneSnapshot> {
    lanes
        .iter()
        .enumerate()
        .map(|(ordinal, lane)| LaneSnapshot {
            ordinal,
            maximum_levenshtein_distance: lane.maximum_levenshtein_distance,
            symbols: lane.symbols.to_vec(),
            payload_bytes: lane.symbols.len() * size_of::<u32>(),
        })
        .collect()
}

fn scalars(text: &str) -> Vec<u32> {
    text.chars().map(|character| character as u32).collect()
}

fn representation_surfaces(raw: &str) -> (String, String, String) {
    let raw_surface = raw.trim().to_lowercase();
    let lexical_surface = normalize_lexical_surface(&raw_surface);
    let direction = detect_direction(&raw_surface);
    let projected_raw = raw_surface
        .chars()
        .map(|character| project_char(character, direction))
        .collect::<String>();
    let projected_surface = normalize_lexical_surface(&projected_raw);
    (raw_surface, lexical_surface, projected_surface)
}

fn boundary_punctuation(character: char) -> bool {
    matches!(character, '!' | ',' | '.' | '?' | ';' | ':')
}

#[derive(Debug, Serialize)]
struct CaseMeasurement {
    id: &'static str,
    raw: &'static str,
    expected: &'static str,
    raw_surface: String,
    lexical_surface: String,
    projected_surface: String,
    raw_symbols: Vec<u32>,
    lexical_symbols: Vec<u32>,
    projected_symbols: Vec<u32>,
    lowercase_expands_scalar_count: bool,
    leading_punctuation_len: usize,
    trailing_punctuation_start: usize,
    trailing_punctuation_len: usize,
    lanes: Vec<LaneSnapshot>,
    certificate_keys: Vec<String>,
    oracle_size_bytes: usize,
    oracle_retained_symbol_bytes: usize,
    lane_vector_retained_bytes: usize,
    lane_symbols_retained_bytes: usize,
    oracle_allocations: AllocationCounts,
    retrieval_lane_allocations: AllocationCounts,
    combined_allocations: AllocationCounts,
    oracle_latency: TimingDistribution,
    retrieval_lane_latency: TimingDistribution,
    combined_latency: TimingDistribution,
}

fn measure_case(case: MeasurementCase) -> CaseMeasurement {
    let (raw_surface, lexical_surface, projected_surface) = representation_surfaces(case.raw);
    let raw_symbols = scalars(&raw_surface);
    let lexical_symbols = scalars(&lexical_surface);
    let projected_symbols = scalars(&projected_surface);
    let original_trimmed_scalar_count = case.raw.trim().chars().count();
    let raw_scalar_count = raw_symbols.len();
    let leading_punctuation_len = raw_surface
        .chars()
        .take_while(|character| boundary_punctuation(*character))
        .count();
    let trailing_punctuation_len = raw_surface
        .chars()
        .rev()
        .take_while(|character| boundary_punctuation(*character))
        .count();
    let trailing_punctuation_start = raw_scalar_count.saturating_sub(trailing_punctuation_len);

    let (oracle_for_count, oracle_allocations) = with_allocation_counts(|| {
        Phase7dCertificateOracle::new(black_box(case.raw)).expect("encode TD-108 query")
    });
    oracle_allocations.assert_conservation(case.id, "oracle");
    let oracle_retained_symbol_bytes =
        (raw_symbols.len() + lexical_symbols.len()) * size_of::<u32>();
    assert_eq!(
        oracle_allocations.current_live_requested_bytes, oracle_retained_symbol_bytes as i64,
        "TD-108 oracle retained-byte mismatch for {}",
        case.id
    );
    drop(oracle_for_count);

    let oracle_for_lanes =
        Phase7dCertificateOracle::new(case.raw).expect("encode TD-108 lane query");
    let (lanes_for_count, retrieval_lane_allocations) =
        with_allocation_counts(|| oracle_for_lanes.retrieval_lanes());
    retrieval_lane_allocations.assert_conservation(case.id, "retrieval_lanes");
    let lane_vector_retained_bytes = lanes_for_count.capacity() * size_of::<Phase7dRetrievalLane>();
    let lane_symbols_retained_bytes = lanes_for_count
        .iter()
        .map(|lane| lane.symbols.len() * size_of::<u32>())
        .sum::<usize>();
    assert_eq!(
        retrieval_lane_allocations.current_live_requested_bytes,
        (lane_vector_retained_bytes + lane_symbols_retained_bytes) as i64,
        "TD-108 lane retained-byte mismatch for {}",
        case.id
    );
    drop(lanes_for_count);
    drop(oracle_for_lanes);

    let (combined, combined_allocations) = with_allocation_counts(|| {
        let oracle =
            Phase7dCertificateOracle::new(black_box(case.raw)).expect("encode combined query");
        let lanes = oracle.retrieval_lanes();
        (oracle, lanes)
    });
    combined_allocations.assert_conservation(case.id, "combined");
    assert_eq!(
        combined_allocations.current_live_requested_bytes,
        (oracle_retained_symbol_bytes
            + combined.1.capacity() * size_of::<Phase7dRetrievalLane>()
            + combined
                .1
                .iter()
                .map(|lane| lane.symbols.len() * size_of::<u32>())
                .sum::<usize>()) as i64,
        "TD-108 combined retained-byte mismatch for {}",
        case.id
    );
    drop(combined);

    let oracle = Phase7dCertificateOracle::new(case.raw).expect("encode snapshot query");
    let lanes = oracle.retrieval_lanes();
    let actual_lane_identity = lanes
        .iter()
        .map(|lane| (lane.symbols.to_vec(), lane.maximum_levenshtein_distance))
        .collect::<Vec<_>>();
    let mut expected_lane_identity = Vec::new();
    if !lexical_symbols.is_empty() {
        expected_lane_identity.push((lexical_symbols.clone(), 3));
    }
    if !projected_symbols.is_empty()
        && expected_lane_identity
            .iter()
            .all(|(symbols, _)| symbols != &projected_symbols)
    {
        expected_lane_identity.push((projected_symbols.clone(), 0));
    }
    assert_eq!(
        actual_lane_identity, expected_lane_identity,
        "TD-108 lane identity mismatch for {}",
        case.id
    );
    let certificate_keys = oracle
        .certificate_keys(case.expected)
        .expect("build TD-108 certificate keys");
    if !case.expected.is_empty() {
        assert!(
            !certificate_keys.is_empty(),
            "TD-108 expected surface lacks certificate for {}",
            case.id
        );
    }
    let lanes = lane_snapshots(&lanes);

    let oracle_latency = measure_latency(|| {
        Phase7dCertificateOracle::new(black_box(case.raw)).expect("time oracle construction")
    });
    let latency_oracle =
        Phase7dCertificateOracle::new(case.raw).expect("encode latency lane query");
    let retrieval_lane_latency = measure_latency(|| latency_oracle.retrieval_lanes());
    let combined_latency = measure_latency(|| {
        let oracle =
            Phase7dCertificateOracle::new(black_box(case.raw)).expect("time combined query");
        let lanes = oracle.retrieval_lanes();
        (oracle, lanes)
    });

    CaseMeasurement {
        id: case.id,
        raw: case.raw,
        expected: case.expected,
        raw_surface,
        lexical_surface,
        projected_surface,
        raw_symbols,
        lexical_symbols,
        projected_symbols,
        lowercase_expands_scalar_count: raw_scalar_count != original_trimmed_scalar_count,
        leading_punctuation_len,
        trailing_punctuation_start,
        trailing_punctuation_len,
        lanes,
        certificate_keys,
        oracle_size_bytes: size_of_val(&oracle),
        oracle_retained_symbol_bytes,
        lane_vector_retained_bytes,
        lane_symbols_retained_bytes,
        oracle_allocations,
        retrieval_lane_allocations,
        combined_allocations,
        oracle_latency,
        retrieval_lane_latency,
        combined_latency,
    }
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

fn source_sha256(relative: &str) -> String {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(relative);
    let bytes = std::fs::read(&path).unwrap_or_else(|error| panic!("{}: {error}", path.display()));
    hex(&Sha256::digest(bytes))
}

#[test]
#[ignore = "TD-108 measurement-only release baseline; run explicitly with --ignored --exact"]
fn td108_query_representation_measurement_baseline() {
    assert!(
        !cfg!(debug_assertions),
        "TD-108 latency baseline must use cargo test --release"
    );
    assert_eq!(CASES.len(), 10, "TD-108 fixed corpus denominator drift");

    let typed_source_sha256 =
        source_sha256("src/nanda_wave/lexical_grokking/typed_edit_traversal.rs");
    assert_eq!(typed_source_sha256, TYPED_SOURCE_SHA256);
    assert_eq!(hex(&phase7d_semantics_digest()), TYPED_SOURCE_SHA256);

    let cases = CASES.iter().copied().map(measure_case).collect::<Vec<_>>();
    assert!(cases.iter().any(|case| case.id == "physical_comma_layout"));
    assert!(cases.iter().any(|case| case.lowercase_expands_scalar_count));
    assert!(cases.iter().any(|case| case.lexical_symbols.is_empty()));

    let receipt = serde_json::json!({
        "schema": "lay.td108.query-representation-baseline.v2",
        "verdict": "PASS",
        "base_commit": BASE_COMMIT,
        "build_profile": "release-test",
        "fixed_case_count": cases.len(),
        "warmup_iterations_per_operation": WARMUP_ITERATIONS,
        "latency_samples_per_operation": LATENCY_SAMPLES,
        "operations_per_latency_sample": 1,
        "allocation_scope": "exact requested heap bytes and calls on the measured thread only",
        "latency_scope": "individual construction operations measured in-process; returned values are dropped after the timestamp; no refactor admission without same-host old/new comparison",
        "source_route_counts": {
            "encode_raw_lowercase_passes": 1,
            "encode_lexical_normalization_passes": 1,
            "retrieval_lexical_box_clones": 1,
            "retrieval_layout_projection_passes": 1,
            "retrieval_projected_normalization_passes": 1,
        },
        "type_sizes_bytes": {
            "Phase7dCertificateOracle": size_of::<Phase7dCertificateOracle>(),
            "Phase7dRetrievalLane": size_of::<Phase7dRetrievalLane>(),
        },
        "source_identity": {
            "typed_edit_traversal_sha256": typed_source_sha256,
            "phase7d_semantics_digest": hex(&phase7d_semantics_digest()),
        },
        "candidate_comparison": {
            "performed": false,
            "reason": "measurement-only baseline; no representation candidate is admitted",
        },
        "package_bytes_comparison": "NOT_APPLICABLE_NO_CANDIDATE",
        "rss_comparison": "NOT_APPLICABLE_NO_CANDIDATE",
        "runtime_authority_changed": false,
        "public_api_changed": false,
        "package_format_changed": false,
        "installed_state_changed": false,
        "cases": cases,
    });

    println!(
        "TD108_BASELINE_JSON={}",
        serde_json::to_string(&receipt).expect("serialize TD-108 baseline")
    );
}
