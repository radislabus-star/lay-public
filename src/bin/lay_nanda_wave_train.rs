use std::collections::BTreeMap;
use std::env;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use lay::nanda_wave::packet::{write_learned_packet, LearnedPacketEntry};

const DEFAULT_DATASET: &str = "data/nanda_training/generated_cases.tsv";

#[derive(Debug, Clone)]
struct Learned {
    expected: String,
    operation: String,
    count: usize,
    conflicts: usize,
}

fn main() -> io::Result<()> {
    let args = env::args().collect::<Vec<_>>();
    let dataset = arg_path(&args, "--dataset").unwrap_or_else(|| PathBuf::from(DEFAULT_DATASET));
    let out =
        arg_path(&args, "--out").unwrap_or_else(lay::nanda_wave::learned::default_memory_path);
    let learned = learn(&dataset)?;
    write_memory(&out, &learned)?;
    print_summary(&dataset, &out, &learned);
    Ok(())
}

fn learn(path: &Path) -> io::Result<BTreeMap<String, Learned>> {
    let text = fs::read_to_string(path)?;
    let mut map = BTreeMap::<String, Learned>::new();
    for (idx, line) in text.lines().enumerate() {
        if idx == 0 || line.trim().is_empty() {
            continue;
        }
        let cols = line.split('\t').collect::<Vec<_>>();
        if cols.len() < 8 || cols[5] != "1" || cols[2] == cols[3] {
            continue;
        }
        let original = cols[2].trim_end();
        let expected = cols[3].trim_end();
        if original == expected {
            continue;
        }
        let entry = map.entry(original.to_string()).or_insert_with(|| Learned {
            expected: expected.to_string(),
            operation: cols[4].to_string(),
            count: 0,
            conflicts: 0,
        });
        if entry.expected == expected {
            entry.count += 1;
        } else {
            entry.conflicts += 1;
        }
    }
    map.retain(|_, item| item.count > 0 && item.conflicts == 0);
    Ok(map)
}

fn write_memory(path: &Path, learned: &BTreeMap<String, Learned>) -> io::Result<()> {
    let entries = learned
        .iter()
        .map(|(original, item)| LearnedPacketEntry {
            original: original.clone(),
            expected: item.expected.clone(),
            operation: item.operation.clone(),
            count: item.count,
        })
        .collect::<Vec<_>>();
    let report = write_learned_packet(path, &entries)?;
    println!(
        "cell32_packet: bytes={} encoded={} skipped={}",
        lay::nanda_wave::CELL32_BYTES,
        report.encoded,
        report.skipped
    );
    Ok(())
}

fn print_summary(dataset: &Path, out: &Path, learned: &BTreeMap<String, Learned>) {
    let mut by_operation = BTreeMap::<&str, usize>::new();
    for item in learned.values() {
        *by_operation.entry(&item.operation).or_default() += 1;
    }
    println!("dataset: {}", dataset.display());
    println!("out: {}", out.display());
    println!("learned_corrections: {}", learned.len());
    for (operation, count) in by_operation {
        println!("  {operation}: {count}");
    }
}

fn arg_path(args: &[String], name: &str) -> Option<PathBuf> {
    args.windows(2)
        .find_map(|pair| (pair[0] == name).then(|| PathBuf::from(&pair[1])))
}
