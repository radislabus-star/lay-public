use clap::Parser;
use std::fs;
use std::path::Path;

#[derive(Parser, Debug)]
#[command(
    name = "lay-memory-report",
    version,
    about = "Report lay runtime memory and hot/cold memory policy"
)]
struct Args {
    #[arg(long, default_value_t = false)]
    json: bool,
}

#[derive(Debug, Clone)]
struct ProcessMemory {
    pid: u32,
    name: String,
    rss_kb: Option<u64>,
    pss_kb: Option<u64>,
    private_dirty_kb: Option<u64>,
    rss_anon_kb: Option<u64>,
    rss_file_kb: Option<u64>,
    threads: Option<u64>,
}

fn main() {
    let args = Args::parse();
    let mut processes = find_lay_processes();
    processes.sort_by(|left, right| left.name.cmp(&right.name).then(left.pid.cmp(&right.pid)));

    if args.json {
        print_json(&processes);
    } else {
        print_human(&processes);
    }
}

fn find_lay_processes() -> Vec<ProcessMemory> {
    let Ok(entries) = fs::read_dir("/proc") else {
        return Vec::new();
    };
    entries
        .filter_map(Result::ok)
        .filter_map(|entry| {
            let pid = entry.file_name().to_string_lossy().parse::<u32>().ok()?;
            let proc_dir = entry.path();
            let name = fs::read_to_string(proc_dir.join("comm")).ok()?;
            let name = name.trim().to_string();
            if !matches!(name.as_str(), "lay-daemon" | "lay-ibus-engine") {
                return None;
            }
            Some(read_process_memory(pid, name, &proc_dir))
        })
        .collect()
}

fn read_process_memory(pid: u32, name: String, proc_dir: &Path) -> ProcessMemory {
    let status = fs::read_to_string(proc_dir.join("status")).unwrap_or_default();
    let rollup = fs::read_to_string(proc_dir.join("smaps_rollup")).unwrap_or_default();
    ProcessMemory {
        pid,
        name,
        rss_kb: status_kb(&status, "VmRSS"),
        pss_kb: rollup_kb(&rollup, "Pss"),
        private_dirty_kb: rollup_kb(&rollup, "Private_Dirty"),
        rss_anon_kb: status_kb(&status, "RssAnon"),
        rss_file_kb: status_kb(&status, "RssFile"),
        threads: status_count(&status, "Threads"),
    }
}

fn status_kb(status: &str, key: &str) -> Option<u64> {
    status_value(status, key)
}

fn status_count(status: &str, key: &str) -> Option<u64> {
    status_value(status, key)
}

fn status_value(status: &str, key: &str) -> Option<u64> {
    let needle = format!("{key}:");
    status.lines().find_map(|line| {
        let value = line.strip_prefix(&needle)?.split_whitespace().next()?;
        value.parse::<u64>().ok()
    })
}

fn rollup_kb(rollup: &str, key: &str) -> Option<u64> {
    let needle = format!("{key}:");
    rollup.lines().find_map(|line| {
        let value = line
            .trim_start()
            .strip_prefix(&needle)?
            .split_whitespace()
            .next()?;
        value.parse::<u64>().ok()
    })
}

fn print_human(processes: &[ProcessMemory]) {
    println!("lay memory report");
    if processes.is_empty() {
        println!("processes: none");
    } else {
        for process in processes {
            println!(
                "{} pid={} RSS={} PSS={} PrivateDirty={} RssAnon={} RssFile={} Threads={}",
                process.name,
                process.pid,
                kb(process.rss_kb),
                kb(process.pss_kb),
                kb(process.private_dirty_kb),
                kb(process.rss_anon_kb),
                kb(process.rss_file_kb),
                count(process.threads)
            );
        }
    }
    println!();
    println!("startup policy:");
    println!("  daemon hot startup: lexicon guards, replacements, ngram");
    println!(
        "  daemon lazy cold: russian generated forms, full NANDA context wave, LLMWave, full LEM"
    );
    println!("  IME hot startup: bounded RU/EN completion banks");
    println!();
    println!("budgets:");
    println!("  lay-daemon cold start target: <= 80 MB RSS");
    println!("  lay-daemon after typing target: <= 150 MB RSS");
    println!("  lay-ibus-engine target: <= 30 MB RSS");
}

fn print_json(processes: &[ProcessMemory]) {
    println!("{{");
    println!("  \"processes\": [");
    for (idx, process) in processes.iter().enumerate() {
        let comma = if idx + 1 == processes.len() { "" } else { "," };
        println!("    {{");
        println!("      \"name\": {:?},", process.name);
        println!("      \"pid\": {},", process.pid);
        println!("      \"rss_kb\": {},", json_opt(process.rss_kb));
        println!("      \"pss_kb\": {},", json_opt(process.pss_kb));
        println!(
            "      \"private_dirty_kb\": {},",
            json_opt(process.private_dirty_kb)
        );
        println!("      \"rss_anon_kb\": {},", json_opt(process.rss_anon_kb));
        println!("      \"rss_file_kb\": {},", json_opt(process.rss_file_kb));
        println!("      \"threads\": {}", json_opt(process.threads));
        println!("    }}{comma}");
    }
    println!("  ],");
    println!("  \"startup_policy\": {{");
    println!(
        "    \"daemon_hot_startup\": [\"lexicon_guards\", \"typing_replacements\", \"ngram\"],"
    );
    println!(
        "    \"daemon_lazy_cold\": [\"russian_generated_forms\", \"nanda_context_wave\", \"llmwave\", \"lem_full\"],"
    );
    println!("    \"ime_hot_startup\": [\"bounded_ru_completion\", \"bounded_en_completion\"]");
    println!("  }},");
    println!("  \"budgets_kb\": {{");
    println!("    \"daemon_cold_start_rss\": 81920,");
    println!("    \"daemon_after_typing_rss\": 153600,");
    println!("    \"ibus_engine_rss\": 30720");
    println!("  }}");
    println!("}}");
}

fn kb(value: Option<u64>) -> String {
    value
        .map(|kb| format!("{:.1} MB", kb as f64 / 1024.0))
        .unwrap_or_else(|| "unknown".to_string())
}

fn count(value: Option<u64>) -> String {
    value
        .map(|value| value.to_string())
        .unwrap_or_else(|| "unknown".to_string())
}

fn json_opt(value: Option<u64>) -> String {
    value
        .map(|value| value.to_string())
        .unwrap_or_else(|| "null".to_string())
}
