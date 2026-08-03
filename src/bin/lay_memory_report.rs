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
            let executable = fs::read_link(proc_dir.join("exe"))
                .ok()?
                .file_name()?
                .to_string_lossy()
                .to_string();
            let command = fs::read(proc_dir.join("cmdline")).unwrap_or_default();
            let name = runtime_role(&executable, &command)?;
            Some(read_process_memory(pid, name, &proc_dir))
        })
        .collect()
}

fn runtime_role(executable: &str, command: &[u8]) -> Option<String> {
    // Linux appends this marker to /proc/<pid>/exe after an atomic binary
    // upgrade. The old process is still alive and must remain observable.
    let executable = executable.strip_suffix(" (deleted)").unwrap_or(executable);
    let role = match executable {
        "lay-daemon" => "lay-daemon",
        "lay-ibus-engine" => "lay-ibus-engine",
        "lay-l11-serve" | "lay-l1.1-serve" => "lay-l1.1-serve",
        "lay-nanda-wave-train"
            if command.split(|byte| *byte == 0).any(|argument| {
                argument == b"--watch-l3-context-online"
                    || argument == b"--l3-online"
                    || argument == b"--l3-online-service"
            }) =>
        {
            "lay-l3-online"
        }
        _ => return None,
    };
    Some(role.to_string())
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
        println!(
            "TOTAL processes={} RSS={} PSS={} PrivateDirty={}",
            processes.len(),
            kb(sum(processes, |process| process.rss_kb)),
            kb(sum(processes, |process| process.pss_kb)),
            kb(sum(processes, |process| process.private_dirty_kb)),
        );
    }
    println!();
    println!("startup policy:");
    println!("  daemon hot startup: lexicon guards, replacements, ngram");
    println!("  daemon/IME hot field: mmap lexical phase + compact L3 base");
    println!("  full dictionaries and delta merge: cold learner only");
    println!("  L1.1 restoration: one sidecar shared by live clients");
    println!();
    println!("budgets:");
    println!("  lay-daemon settled target: <= 200 MB PSS");
    println!("  lay-ibus-engine settled target: <= 200 MB PSS");
    println!("  lay-l1.1-serve settled target: <= 350 MB PSS");
    println!("  complete runtime settled target: <= 750 MB PSS");
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
    println!("  \"totals\": {{");
    println!("    \"processes\": {},", processes.len());
    println!(
        "    \"rss_kb\": {},",
        json_opt(sum(processes, |process| process.rss_kb))
    );
    println!(
        "    \"pss_kb\": {},",
        json_opt(sum(processes, |process| process.pss_kb))
    );
    println!(
        "    \"private_dirty_kb\": {}",
        json_opt(sum(processes, |process| process.private_dirty_kb))
    );
    println!("  }},");
    println!("  \"startup_policy\": {{");
    println!(
        "    \"daemon_hot_startup\": [\"lexical_phase_mmap\", \"lexicon_guards\", \"typing_replacements\", \"ngram\"],"
    );
    println!("    \"live_context\": [\"compact_l3_base\", \"delta_free_manifest\"],");
    println!("    \"cold_learner_only\": [\"full_dictionaries\", \"delta_merge\"],");
    println!("    \"shared_sidecar\": [\"l1.1_restoration\"]");
    println!("  }},");
    println!("  \"budgets_kb\": {{");
    println!("    \"daemon_settled_pss\": 204800,");
    println!("    \"ibus_engine_settled_pss\": 204800,");
    println!("    \"l11_sidecar_settled_pss\": 358400,");
    println!("    \"complete_runtime_settled_pss\": 768000");
    println!("  }}");
    println!("}}");
}

fn sum(processes: &[ProcessMemory], value: impl Fn(&ProcessMemory) -> Option<u64>) -> Option<u64> {
    processes.iter().try_fold(0_u64, |total, process| {
        Some(total.saturating_add(value(process)?))
    })
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

#[cfg(test)]
mod tests {
    use super::runtime_role;

    #[test]
    fn atomic_upgrade_suffix_does_not_hide_live_runtime_processes() {
        assert_eq!(
            runtime_role("lay-daemon (deleted)", b"lay-daemon\0"),
            Some("lay-daemon".to_string())
        );
        assert_eq!(
            runtime_role(
                "lay-nanda-wave-train (deleted)",
                b"lay-nanda-wave-train\0--watch-l3-context-online\0"
            ),
            Some("lay-l3-online".to_string())
        );
    }
}
