use std::fs;
use std::os::unix::fs::MetadataExt;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use clap::Parser;
use serde::Serialize;
use sha2::{Digest, Sha256};

#[path = "../text_edit/output_transaction.rs"]
mod output_transaction;

use output_transaction::{
    run_output_transaction_fault_matrix, scan_output_transaction_journal, OrderedJournalOwnerV1,
    OutputTransactionIntentV1, OutputTransactionJournalStatsV1, OutputTransactionRecordKindV1,
    OutputTransactionTerminalV1, OUTPUT_TRANSACTION_JOURNAL_CAPACITY_BYTES,
    OUTPUT_TRANSACTION_QUEUE_CAPACITY, OUTPUT_TRANSACTION_RECORD_BYTES,
    OUTPUT_TRANSACTION_SLOT_BYTES, OUTPUT_TRANSACTION_SLOT_CAPACITY,
};

const PREPARE_CO_COMMIT_P99_GATE_US: u64 = 2_000;
const PREPARE_CO_COMMIT_MAX_GATE_US: u64 = 8_000;
const TAIL_FLUSH_DEADLINE_MS: u64 = 20;

#[derive(Parser)]
struct Args {
    #[arg(long)]
    journal_dir: PathBuf,
    #[arg(long)]
    receipt: PathBuf,
    #[arg(long, default_value_t = 1_000)]
    samples: usize,
    #[arg(long, default_value_t = 64)]
    warmup: usize,
}

#[derive(Clone, Debug, Serialize)]
struct LatencySummaryV1 {
    samples: usize,
    p50_us: u64,
    p95_us: u64,
    p99_us: u64,
    maximum_us: u64,
}

#[derive(Debug, Serialize)]
struct StratumReceiptV1 {
    cold_owner_startup_us: u64,
    latency: LatencySummaryV1,
    journal: OutputTransactionJournalStatsV1,
    records_verified: usize,
}

#[derive(Debug, Serialize)]
struct HostReceiptV1 {
    hostname: String,
    filesystem_mount: String,
    filesystem_type: String,
    mount_options: String,
    device_id: u64,
    journal_directory: String,
    clock: &'static str,
    worker_nice: i32,
}

#[derive(Debug, Serialize)]
struct ReceiptV1 {
    schema: &'static str,
    generated_unix_ms: u128,
    verdict: &'static str,
    host: HostReceiptV1,
    record_bytes: usize,
    slot_bytes: usize,
    slot_capacity: usize,
    sync_primitive: &'static str,
    journal_capacity_bytes: usize,
    queue_capacity: usize,
    tail_flush_deadline_ms: u64,
    warmup_samples: usize,
    prepare: StratumReceiptV1,
    co_commit: StratumReceiptV1,
    tail_flush: StratumReceiptV1,
    next_native_wait: StratumReceiptV1,
    fault_matrix: output_transaction::OutputTransactionFaultMatrixV1,
    gates: GateReceiptV1,
    runtime_authority_changed: bool,
    installed_runtime_touched: bool,
}

#[derive(Debug, Serialize)]
struct GateReceiptV1 {
    p99_at_most_us: u64,
    maximum_strictly_below_us: u64,
    prepare_pass: bool,
    co_commit_pass: bool,
}

fn digest(label: &[u8], sequence: usize) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(label);
    hasher.update((sequence as u64).to_le_bytes());
    hasher.finalize().into()
}

fn intent(sequence: usize) -> OutputTransactionIntentV1 {
    OutputTransactionIntentV1 {
        event_id: digest(b"event", sequence),
        lineage_id: digest(b"lineage", 0),
        before_digest: digest(b"before", sequence),
        intended_after_digest: digest(b"after", sequence),
    }
}

fn terminal(sequence: usize) -> OutputTransactionTerminalV1 {
    OutputTransactionTerminalV1::new(
        OutputTransactionRecordKindV1::Succeeded,
        digest(b"after", sequence),
    )
    .expect("Succeeded is terminal")
}

fn summarize(mut samples: Vec<u64>) -> LatencySummaryV1 {
    samples.sort_unstable();
    let percentile = |percent: usize| {
        let index = (samples.len() * percent).div_ceil(100).saturating_sub(1);
        samples[index.min(samples.len().saturating_sub(1))]
    };
    LatencySummaryV1 {
        samples: samples.len(),
        p50_us: percentile(50),
        p95_us: percentile(95),
        p99_us: percentile(99),
        maximum_us: samples.last().copied().unwrap_or_default(),
    }
}

fn journal_path(root: &Path, name: &str) -> PathBuf {
    root.join(format!("{name}.journal"))
}

fn clean_path(path: &Path) {
    let _ = fs::remove_file(path);
}

fn run_prepare(root: &Path, warmup: usize, samples: usize) -> Result<StratumReceiptV1, String> {
    let path = journal_path(root, "prepare");
    clean_path(&path);
    let startup_started = Instant::now();
    let owner = OrderedJournalOwnerV1::start(&path, Duration::from_millis(TAIL_FLUSH_DEADLINE_MS))
        .map_err(|error| format!("prepare owner: {error:?}"))?;
    let cold_owner_startup_us = startup_started.elapsed().as_micros() as u64;
    let mut latency = Vec::with_capacity(samples);
    for sequence in 0..warmup + samples {
        let event = intent(sequence);
        let started = Instant::now();
        let receipt = owner
            .prepare(event)
            .map_err(|error| format!("prepare: {error:?}"))?;
        let elapsed = started.elapsed().as_micros() as u64;
        if receipt.foreground_waits != 1 || receipt.published_previous_terminal {
            return Err("prepare barrier accounting mismatch".to_string());
        }
        owner
            .begin_effect(event.event_id)
            .map_err(|error| format!("begin: {error:?}"))?;
        owner
            .finish(event.event_id, terminal(sequence))
            .map_err(|error| format!("finish: {error:?}"))?;
        owner
            .flush_terminal()
            .map_err(|error| format!("flush: {error:?}"))?;
        if sequence >= warmup {
            latency.push(elapsed);
        }
    }
    let (_, stats) = owner
        .snapshot()
        .map_err(|error| format!("snapshot: {error:?}"))?;
    owner.stop().map_err(|error| format!("stop: {error:?}"))?;
    let records =
        scan_output_transaction_journal(&path).map_err(|error| format!("scan: {error:?}"))?;
    Ok(StratumReceiptV1 {
        cold_owner_startup_us,
        latency: summarize(latency),
        journal: stats,
        records_verified: records,
    })
}

fn run_co_commit(root: &Path, warmup: usize, samples: usize) -> Result<StratumReceiptV1, String> {
    let path = journal_path(root, "co-commit");
    clean_path(&path);
    let startup_started = Instant::now();
    let owner = OrderedJournalOwnerV1::start(&path, Duration::from_millis(TAIL_FLUSH_DEADLINE_MS))
        .map_err(|error| format!("co-commit owner: {error:?}"))?;
    let cold_owner_startup_us = startup_started.elapsed().as_micros() as u64;
    let first = intent(0);
    owner
        .prepare(first)
        .map_err(|error| format!("initial prepare: {error:?}"))?;
    owner
        .begin_effect(first.event_id)
        .map_err(|error| format!("initial begin: {error:?}"))?;
    owner
        .finish(first.event_id, terminal(0))
        .map_err(|error| format!("initial finish: {error:?}"))?;
    let mut latency = Vec::with_capacity(samples);
    for sequence in 1..=warmup + samples {
        let event = intent(sequence);
        let started = Instant::now();
        let receipt = owner
            .prepare(event)
            .map_err(|error| format!("co-commit: {error:?}"))?;
        let elapsed = started.elapsed().as_micros() as u64;
        if receipt.foreground_waits != 1 || !receipt.published_previous_terminal {
            return Err("co-commit did not publish the previous terminal".to_string());
        }
        owner
            .begin_effect(event.event_id)
            .map_err(|error| format!("begin: {error:?}"))?;
        owner
            .finish(event.event_id, terminal(sequence))
            .map_err(|error| format!("finish: {error:?}"))?;
        if sequence > warmup {
            latency.push(elapsed);
        }
    }
    owner
        .flush_terminal()
        .map_err(|error| format!("final flush: {error:?}"))?;
    let (_, stats) = owner
        .snapshot()
        .map_err(|error| format!("snapshot: {error:?}"))?;
    owner.stop().map_err(|error| format!("stop: {error:?}"))?;
    let records =
        scan_output_transaction_journal(&path).map_err(|error| format!("scan: {error:?}"))?;
    Ok(StratumReceiptV1 {
        cold_owner_startup_us,
        latency: summarize(latency),
        journal: stats,
        records_verified: records,
    })
}

fn run_terminal_wait(
    root: &Path,
    name: &str,
    warmup: usize,
    samples: usize,
    native: bool,
) -> Result<StratumReceiptV1, String> {
    let path = journal_path(root, name);
    clean_path(&path);
    let startup_started = Instant::now();
    let owner = OrderedJournalOwnerV1::start(&path, Duration::from_millis(TAIL_FLUSH_DEADLINE_MS))
        .map_err(|error| format!("{name} owner: {error:?}"))?;
    let cold_owner_startup_us = startup_started.elapsed().as_micros() as u64;
    let mut latency = Vec::with_capacity(samples);
    for sequence in 0..warmup + samples {
        let event = intent(sequence);
        owner
            .prepare(event)
            .map_err(|error| format!("prepare: {error:?}"))?;
        owner
            .begin_effect(event.event_id)
            .map_err(|error| format!("begin: {error:?}"))?;
        owner
            .finish(event.event_id, terminal(sequence))
            .map_err(|error| format!("finish: {error:?}"))?;
        let started = Instant::now();
        let receipt = if native {
            owner.wait_before_native_state_change()
        } else {
            owner.flush_terminal()
        }
        .map_err(|error| format!("{name} wait: {error:?}"))?;
        let elapsed = started.elapsed().as_micros() as u64;
        if !receipt.published {
            return Err(format!("{name} returned before terminal publication"));
        }
        if sequence >= warmup {
            latency.push(elapsed);
        }
    }
    let (_, stats) = owner
        .snapshot()
        .map_err(|error| format!("snapshot: {error:?}"))?;
    owner.stop().map_err(|error| format!("stop: {error:?}"))?;
    let records =
        scan_output_transaction_journal(&path).map_err(|error| format!("scan: {error:?}"))?;
    Ok(StratumReceiptV1 {
        cold_owner_startup_us,
        latency: summarize(latency),
        journal: stats,
        records_verified: records,
    })
}

fn mount_info(path: &Path) -> (String, String, String) {
    let canonical = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
    let mut best = (String::new(), String::from("unknown"), String::new());
    let Ok(mountinfo) = fs::read_to_string("/proc/self/mountinfo") else {
        return best;
    };
    for line in mountinfo.lines() {
        let Some((left, right)) = line.split_once(" - ") else {
            continue;
        };
        let left_fields = left.split_whitespace().collect::<Vec<_>>();
        let right_fields = right.split_whitespace().collect::<Vec<_>>();
        if left_fields.len() < 6 || right_fields.len() < 3 {
            continue;
        }
        let mount = Path::new(left_fields[4]);
        if canonical.starts_with(mount) && mount.as_os_str().len() >= best.0.len() {
            best = (
                left_fields[4].to_string(),
                right_fields[0].to_string(),
                left_fields[5].to_string(),
            );
        }
    }
    best
}

fn main() -> Result<(), String> {
    let args = Args::parse();
    if args.samples == 0 {
        return Err("--samples must be positive".to_string());
    }
    fs::create_dir_all(&args.journal_dir).map_err(|error| error.to_string())?;
    let prepare = run_prepare(&args.journal_dir, args.warmup, args.samples)?;
    let co_commit = run_co_commit(&args.journal_dir, args.warmup, args.samples)?;
    let tail_flush = run_terminal_wait(
        &args.journal_dir,
        "tail-flush",
        args.warmup,
        args.samples,
        false,
    )?;
    let next_native_wait = run_terminal_wait(
        &args.journal_dir,
        "native-wait",
        args.warmup,
        args.samples,
        true,
    )?;
    let prepare_pass = prepare.latency.p99_us <= PREPARE_CO_COMMIT_P99_GATE_US
        && prepare.latency.maximum_us < PREPARE_CO_COMMIT_MAX_GATE_US;
    let co_commit_pass = co_commit.latency.p99_us <= PREPARE_CO_COMMIT_P99_GATE_US
        && co_commit.latency.maximum_us < PREPARE_CO_COMMIT_MAX_GATE_US;
    let (mount, filesystem_type, options) = mount_info(&args.journal_dir);
    let device_id = fs::metadata(&args.journal_dir)
        .map_err(|error| error.to_string())?
        .dev();
    let hostname = fs::read_to_string("/etc/hostname")
        .unwrap_or_else(|_| "unknown".to_string())
        .trim()
        .to_string();
    let fault_matrix = run_output_transaction_fault_matrix();
    let receipt = ReceiptV1 {
        schema: "lay.output-transaction-direct-slot-microproof.v2",
        generated_unix_ms: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|error| error.to_string())?
            .as_millis(),
        verdict: if prepare_pass && co_commit_pass && fault_matrix.passes() {
            "PASS_ISOLATED_DURABILITY_STRATEGY"
        } else {
            "FAIL_ISOLATED_DURABILITY_STRATEGY"
        },
        host: HostReceiptV1 {
            hostname,
            filesystem_mount: mount,
            filesystem_type,
            mount_options: options,
            device_id,
            journal_directory: args.journal_dir.display().to_string(),
            clock: "std::time::Instant(CLOCK_MONOTONIC)",
            worker_nice: prepare.journal.owner_nice,
        },
        record_bytes: OUTPUT_TRANSACTION_RECORD_BYTES,
        slot_bytes: OUTPUT_TRANSACTION_SLOT_BYTES,
        slot_capacity: OUTPUT_TRANSACTION_SLOT_CAPACITY,
        sync_primitive: "aligned pwrite on O_DIRECT|O_DSYNC; no buffered fallback",
        journal_capacity_bytes: OUTPUT_TRANSACTION_JOURNAL_CAPACITY_BYTES,
        queue_capacity: OUTPUT_TRANSACTION_QUEUE_CAPACITY,
        tail_flush_deadline_ms: TAIL_FLUSH_DEADLINE_MS,
        warmup_samples: args.warmup,
        prepare,
        co_commit,
        tail_flush,
        next_native_wait,
        fault_matrix,
        gates: GateReceiptV1 {
            p99_at_most_us: PREPARE_CO_COMMIT_P99_GATE_US,
            maximum_strictly_below_us: PREPARE_CO_COMMIT_MAX_GATE_US,
            prepare_pass,
            co_commit_pass,
        },
        runtime_authority_changed: false,
        installed_runtime_touched: false,
    };
    let bytes = serde_json::to_vec_pretty(&receipt).map_err(|error| error.to_string())?;
    if let Some(parent) = args.receipt.parent() {
        fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    }
    fs::write(&args.receipt, bytes).map_err(|error| error.to_string())?;
    println!(
        "{}",
        serde_json::to_string_pretty(&receipt).map_err(|error| error.to_string())?
    );
    if prepare_pass && co_commit_pass && fault_matrix.passes() {
        Ok(())
    } else {
        Err("durability strategy gate failed".to_string())
    }
}
