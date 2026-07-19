use std::collections::BTreeMap;
use std::env;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use lay::nanda_wave::llmwave;
use lay::nanda_wave::packet::{write_learned_packet, LearnedPacketEntry};
use lay::nanda_wave::L2PhaseTrainingEntry;
use lay::{lexicon, russian_lexicon};
use serde::Deserialize;

#[allow(dead_code)]
#[path = "../nanda_wave/lexical_phase/format.rs"]
mod lexical_phase_format;
#[path = "../lexical_surface_atoms.rs"]
mod lexical_surface_atoms;
#[path = "../stable_hash.rs"]
mod stable_hash;

#[allow(dead_code)]
mod lexical_phase_compiler {
    include!("../nanda_wave/lexical_phase/compiler.rs");
}

use lexical_phase_format as format;
include!("lay_nanda_wave_train/lexical_phase_compile.rs");

const DEFAULT_DATASET: &str = "data/nanda_training/generated_cases.tsv";
const RECENT_ACTIONS: &str = ".local/share/lay/recent_actions.jsonl";
const CORRECTIONS_LOG: &str = ".local/share/lay/corrections.jsonl";
const USAGE_EVENTS: &str = ".local/share/lay/nanda_wave/word_usage_events.jsonl";

#[derive(Debug, Clone)]
struct Learned {
    expected: String,
    operation: String,
    count: usize,
    conflicts: usize,
    live_count: usize,
}

fn main() -> io::Result<()> {
    let args = env::args().collect::<Vec<_>>();
    if args.iter().any(|arg| arg == "--l3-context-phase-status") {
        let memory = arg_path(&args, "--memory");
        println!(
            "{}",
            serde_json::to_string_pretty(&lay::nanda_wave::l3_context_phase_status_json(
                memory.as_deref(),
            ))
            .map_err(io::Error::other)?
        );
        return Ok(());
    }
    if let Some(corpus) = arg_path(&args, "--prove-l3-context-phase") {
        let max_fragments = arg_usize(&args, "--max-fragments").unwrap_or(0);
        let min_profile_support = arg_u32(&args, "--min-profile-support").unwrap_or(2);
        let report = lay::nanda_wave::prove_l3_context_phase_memory(
            &corpus,
            max_fragments,
            min_profile_support,
        )?;
        println!(
            "{}",
            serde_json::to_string_pretty(&report).map_err(io::Error::other)?
        );
        return Ok(());
    }
    if let Some(corpus) = arg_path(&args, "--compile-l3-context-phase") {
        let out = arg_path(&args, "--out")
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "--out is required"))?;
        let max_fragments = arg_usize(&args, "--max-fragments").unwrap_or(0);
        let min_profile_support = arg_u32(&args, "--min-profile-support").unwrap_or(2);
        let report = lay::nanda_wave::compile_l3_context_phase_memory(
            &corpus,
            &out,
            max_fragments,
            min_profile_support,
        )?;
        println!(
            "{}",
            serde_json::to_string_pretty(&report).map_err(io::Error::other)?
        );
        return Ok(());
    }
    if args
        .iter()
        .any(|arg| arg == "--compile-l3-context-feedback-overlay")
    {
        let base = arg_path(&args, "--base")
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "--base is required"))?;
        let usage_events = arg_path(&args, "--usage-events").ok_or_else(|| {
            io::Error::new(io::ErrorKind::InvalidInput, "--usage-events is required")
        })?;
        let out = arg_path(&args, "--out")
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "--out is required"))?;
        let report = lay::nanda_wave::compile_l3_context_feedback_overlay_memory(
            &base,
            &usage_events,
            &out,
        )?;
        println!(
            "{}",
            serde_json::to_string_pretty(&report).map_err(io::Error::other)?
        );
        return Ok(());
    }
    if args.iter().any(|arg| arg == "--compile-lexical-phase") {
        return run_lexical_phase_compile(&args);
    }
    if args.iter().any(|arg| arg == "--l2-surface-status") {
        print_l2_surface_status();
        return Ok(());
    }
    if let Some(corpus) = arg_path(&args, "--llmwave-corpus") {
        let out = arg_path(&args, "--llmwave-out")
            .or_else(|| arg_path(&args, "--out"))
            .or_else(llmwave::default_memory_path)
            .expect("default llmwave memory path");
        train_llmwave_corpus(&corpus, &out)?;
        return Ok(());
    }
    let dataset = arg_path(&args, "--dataset").unwrap_or_else(|| PathBuf::from(DEFAULT_DATASET));
    let out =
        arg_path(&args, "--out").unwrap_or_else(lay::nanda_wave::learned::default_memory_path);
    let phase_out = arg_path(&args, "--phase-out")
        .unwrap_or_else(lay::nanda_wave::default_l2_candidate_phase_memory_path);
    let pack_live = args.iter().any(|arg| arg == "--pack-live");
    let include_live_actions = pack_live || args.iter().any(|arg| arg == "--include-live-actions");
    let include_user_corrections =
        pack_live || args.iter().any(|arg| arg == "--include-user-corrections");
    let phase_only = args.iter().any(|arg| arg == "--phase-only");
    if phase_only {
        let phase_entries =
            phase_training_entries(&dataset, include_live_actions, include_user_corrections)?;
        write_phase_memory(&phase_out, phase_entries)?;
        println!("phase_out: {}", phase_out.display());
        return Ok(());
    }
    let mut learned = learn(&dataset)?;
    let live_report = if include_live_actions || include_user_corrections {
        add_live_learning(&mut learned, include_user_corrections)?
    } else {
        LiveLearningReport::default()
    };
    let phase_entries =
        phase_training_entries(&dataset, include_live_actions, include_user_corrections)?;
    write_memory(&out, &learned)?;
    write_phase_memory(&phase_out, phase_entries)?;
    print_summary(&dataset, &out, &phase_out, &learned, &live_report);
    Ok(())
}

fn arg_usize(args: &[String], name: &str) -> Option<usize> {
    args.windows(2)
        .find(|window| window[0] == name)
        .and_then(|window| window[1].parse().ok())
}

fn arg_u32(args: &[String], name: &str) -> Option<u32> {
    args.windows(2)
        .find(|window| window[0] == name)
        .and_then(|window| window[1].parse().ok())
}

fn print_l2_surface_status() {
    let status = serde_json::to_value(lay::nanda_wave::l2::l2_surface_memory_status())
        .expect("L2 surface status must serialize");
    println!("l2_surface_status:");
    for key in [
        "active_source_target",
        "source_words",
        "l1_centers",
        "l1_postings",
        "l2_word_centers",
        "grapheme_nodes",
        "grapheme_arcs",
        "decoder_states",
        "decoder_arcs",
        "training_surfaces",
        "artifact_bytes",
        "artifact_mmap_backed",
        "raw_word_table",
        "generated_forms_loaded",
        "generated_forms_words",
    ] {
        println!("  {key}: {}", status[key]);
    }
    let phase = lay::nanda_wave::l2_transition_phase_report_json(None);
    println!(
        "l2_transition_phase: loaded={} profiles={} hot_bytes={}",
        phase["loaded"].as_bool().unwrap_or(false),
        phase["profile_count"].as_u64().unwrap_or(0),
        phase["hot_bytes"].as_u64().unwrap_or(0)
    );
}

fn train_llmwave_corpus(corpus: &Path, out: &Path) -> io::Result<()> {
    let text = fs::read_to_string(corpus)?;
    let memory = llmwave::LlmWaveMemory::from_text(&text);
    llmwave::write_memory_packet(out, &memory)?;
    let bytes = fs::metadata(out).map(|meta| meta.len()).unwrap_or_default();
    println!(
        "llmwave_corpus_train: input={} output={} records={} vocabulary={} bytes={} record_bytes={}",
        corpus.display(),
        out.display(),
        memory.len(),
        memory.vocabulary_len(),
        bytes,
        llmwave::LLMWAVE_RECORD_BYTES
    );
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
            live_count: 0,
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

#[derive(Debug, Default)]
struct LiveLearningReport {
    read: usize,
    accepted: usize,
    skipped: usize,
    user_skipped: usize,
}

#[derive(Debug, Deserialize)]
struct LiveAction {
    #[serde(default)]
    kind: String,
    #[serde(default, rename = "from")]
    from_text: String,
    #[serde(default, rename = "to")]
    to_text: String,
    #[serde(default)]
    safety_allow_apply: Option<bool>,
    #[serde(default)]
    lay_from: String,
    #[serde(default)]
    lay_to: String,
}

fn add_live_learning(
    learned: &mut BTreeMap<String, Learned>,
    include_user_corrections: bool,
) -> io::Result<LiveLearningReport> {
    let mut report = LiveLearningReport::default();
    for path in live_paths() {
        add_live_file(learned, &path, include_user_corrections, &mut report)?;
    }
    learned.retain(|_, item| item.count > 0 && item.conflicts == 0);
    Ok(report)
}

fn add_live_file(
    learned: &mut BTreeMap<String, Learned>,
    path: &Path,
    include_user_corrections: bool,
    report: &mut LiveLearningReport,
) -> io::Result<()> {
    let Ok(text) = fs::read_to_string(path) else {
        return Ok(());
    };
    for line in text.lines().filter(|line| !line.trim().is_empty()) {
        report.read += 1;
        let Ok(action) = serde_json::from_str::<LiveAction>(line) else {
            report.skipped += 1;
            continue;
        };
        if !is_learnable_live_kind(&action.kind, include_user_corrections) {
            if action.kind == "user-correction" {
                report.user_skipped += 1;
            } else {
                report.skipped += 1;
            }
            continue;
        }
        let Some((from, to)) = normalized_live_pair(&action.from_text, &action.to_text) else {
            report.skipped += 1;
            continue;
        };
        let operation = operation_from_live_kind(&action.kind, &from, &to).to_string();
        let entry = learned.entry(from).or_insert_with(|| Learned {
            expected: to.clone(),
            operation,
            count: 0,
            conflicts: 0,
            live_count: 0,
        });
        if entry.expected == to {
            entry.count += 1;
            entry.live_count += 1;
            report.accepted += 1;
        } else {
            entry.conflicts += 1;
            report.skipped += 1;
        }
    }
    Ok(())
}

fn is_learnable_live_kind(kind: &str, include_user_corrections: bool) -> bool {
    matches!(
        kind,
        "typing-assist" | "ime-typing-assist" | "layout-replay" | "smart-text"
    ) || (include_user_corrections && kind == "user-correction")
}

fn normalized_live_pair(from: &str, to: &str) -> Option<(String, String)> {
    let from = from.trim_end();
    let to = to.trim_end();
    if from.is_empty()
        || to.is_empty()
        || from == to
        || from.chars().count() > 96
        || to.chars().count() > 96
        || from.chars().any(char::is_control)
        || to.chars().any(char::is_control)
        || from.split_whitespace().count().max(1) > 6
        || to.split_whitespace().count().max(1) > 6
    {
        return None;
    }
    Some((from.to_string(), to.to_string()))
}

fn operation_from_live_kind(kind: &str, from: &str, to: &str) -> &'static str {
    if kind == "layout-replay" || scripts_look_layout_like(from, to) {
        "layout"
    } else if from.split_whitespace().count() != to.split_whitespace().count() {
        "split"
    } else {
        "typo"
    }
}

fn scripts_look_layout_like(from: &str, to: &str) -> bool {
    let from_ascii = from.chars().any(|ch| ch.is_ascii_alphabetic());
    let from_cyr = from
        .chars()
        .any(|ch| ('а'..='я').contains(&ch) || ('А'..='Я').contains(&ch));
    let to_ascii = to.chars().any(|ch| ch.is_ascii_alphabetic());
    let to_cyr = to
        .chars()
        .any(|ch| ('а'..='я').contains(&ch) || ('А'..='Я').contains(&ch));
    (from_ascii && to_cyr) || (from_cyr && to_ascii)
}

fn live_paths() -> Vec<PathBuf> {
    let Some(home) = env::var_os("HOME").map(PathBuf::from) else {
        return Vec::new();
    };
    vec![
        home.join(RECENT_ACTIONS),
        home.join(CORRECTIONS_LOG),
        home.join(USAGE_EVENTS),
    ]
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

fn write_phase_memory(path: &Path, entries: Vec<L2PhaseTrainingEntry>) -> io::Result<()> {
    let bytes = lay::nanda_wave::write_l2_candidate_phase_memory_labeled(path, entries)?;
    println!("l2_candidate_phase_packet: bytes={bytes}");
    let report = lay::nanda_wave::l2_transition_phase_report_json(Some(path));
    println!(
        "l2_transition_phase_profiles: profiles={} raw_words_stored={}",
        report["profile_count"].as_u64().unwrap_or(0),
        report["raw_words_stored"].as_bool().unwrap_or(true)
    );
    Ok(())
}

fn phase_training_entries(
    dataset: &Path,
    include_live_actions: bool,
    include_user_corrections: bool,
) -> io::Result<Vec<L2PhaseTrainingEntry>> {
    let text = fs::read_to_string(dataset)?;
    let rows = text
        .lines()
        .skip(1)
        .filter_map(|line| {
            let cols = line.split('\t').collect::<Vec<_>>();
            (cols.len() >= 8 && cols[2] != cols[3]).then(|| {
                (
                    cols[0].to_string(),
                    cols[2].trim_end().to_string(),
                    cols[3].trim_end().to_string(),
                    cols[4].to_string(),
                    cols[5] == "1",
                )
            })
        })
        .collect::<Vec<_>>();
    let group_operators = rows
        .iter()
        .filter(|row| row.4)
        .map(|row| {
            (
                row.0.clone(),
                lay::nanda_wave::infer_l2_transition_operator(&row.1, &row.2, &row.3).to_string(),
            )
        })
        .collect::<BTreeMap<_, _>>();
    let mut entries = rows
        .into_iter()
        .filter_map(|(group, original, candidate, operation, accepted)| {
            let operation = group_operators.get(&group).cloned().unwrap_or(operation);
            (!original.is_empty() && !candidate.is_empty()).then_some(L2PhaseTrainingEntry {
                original,
                candidate,
                operation,
                accepted,
                count: 1,
            })
        })
        .collect::<Vec<_>>();

    if include_live_actions || include_user_corrections {
        for path in live_paths() {
            append_live_phase_entries(&mut entries, &path, include_user_corrections)?;
        }
    }
    Ok(entries)
}

fn append_live_phase_entries(
    entries: &mut Vec<L2PhaseTrainingEntry>,
    path: &Path,
    include_user_corrections: bool,
) -> io::Result<()> {
    let Ok(text) = fs::read_to_string(path) else {
        return Ok(());
    };
    for line in text.lines().filter(|line| !line.trim().is_empty()) {
        let Ok(action) = serde_json::from_str::<LiveAction>(line) else {
            continue;
        };
        if action.kind == "layout-replay" {
            push_phase_entry(
                entries,
                &action.from_text,
                &action.to_text,
                &action.kind,
                true,
            );
        } else if action.kind == "candidate_before_apply"
            && action.safety_allow_apply == Some(false)
        {
            push_phase_entry(
                entries,
                &action.from_text,
                &action.to_text,
                &action.kind,
                false,
            );
        } else if include_user_corrections && action.kind == "user-correction" {
            push_causal_user_phase_entries(entries, &action);
        }
    }
    Ok(())
}

fn push_causal_user_phase_entries(entries: &mut Vec<L2PhaseTrainingEntry>, action: &LiveAction) {
    // A generic user-correction record can describe later typing at another
    // caret position. Train only an observed local chain: raw input -> Lay
    // proposal -> immediate user replacement of that exact proposal.
    let Some((original, automatic)) = normalized_live_pair(&action.lay_from, &action.lay_to) else {
        return;
    };
    let Some((applied, target)) = normalized_live_pair(&action.from_text, &action.to_text) else {
        return;
    };
    if applied != automatic
        || original.split_whitespace().count() != 1
        || automatic.split_whitespace().count() != 1
        || target.split_whitespace().count() != 1
        || automatic == target
    {
        return;
    }
    let operator =
        lay::nanda_wave::infer_l2_transition_operator(&original, &target, "user-correction");
    let automatic_operator =
        lay::nanda_wave::infer_l2_transition_operator(&original, &automatic, "user-correction");
    if !matches!(
        operator,
        "adjacent_transposition"
            | "missing_letter_repair"
            | "repeated_letter_repair"
            | "extra_letter_repair"
            | "letter_substitution"
    ) || automatic_operator != operator
    {
        return;
    }
    entries.push(L2PhaseTrainingEntry {
        original: original.clone(),
        candidate: target,
        operation: operator.to_string(),
        accepted: true,
        count: 1,
    });
    entries.push(L2PhaseTrainingEntry {
        original,
        candidate: automatic,
        operation: operator.to_string(),
        accepted: false,
        count: 1,
    });
}

fn push_phase_entry(
    entries: &mut Vec<L2PhaseTrainingEntry>,
    from: &str,
    to: &str,
    kind: &str,
    accepted: bool,
) {
    let Some((original, candidate)) = normalized_live_pair(from, to) else {
        return;
    };
    entries.push(L2PhaseTrainingEntry {
        operation: operation_from_live_kind(kind, &original, &candidate).to_string(),
        original,
        candidate,
        accepted,
        count: 1,
    });
}

fn print_summary(
    dataset: &Path,
    out: &Path,
    phase_out: &Path,
    learned: &BTreeMap<String, Learned>,
    live_report: &LiveLearningReport,
) {
    let mut by_operation = BTreeMap::<&str, usize>::new();
    let mut live_entries = 0usize;
    for item in learned.values() {
        *by_operation.entry(&item.operation).or_default() += 1;
        if item.live_count > 0 {
            live_entries += 1;
        }
    }
    println!("dataset: {}", dataset.display());
    println!("out: {}", out.display());
    println!("phase_out: {}", phase_out.display());
    println!("learned_corrections: {}", learned.len());
    if live_report.read > 0 {
        println!(
            "live_actions: read={} accepted={} skipped={} user_skipped={} live_entries={}",
            live_report.read,
            live_report.accepted,
            live_report.skipped,
            live_report.user_skipped,
            live_entries
        );
    }
    for (operation, count) in by_operation {
        println!("  {operation}: {count}");
    }
}

fn arg_path(args: &[String], name: &str) -> Option<PathBuf> {
    args.windows(2)
        .find_map(|pair| (pair[0] == name).then(|| PathBuf::from(&pair[1])))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn live_typing_action_is_learnable() {
        let pair = normalized_live_pair("fавтозамена ", "автозамена ").unwrap();
        assert_eq!(pair, ("fавтозамена".to_string(), "автозамена".to_string()));
        assert!(is_learnable_live_kind("ime-typing-assist", false));
        assert_eq!(
            operation_from_live_kind("ime-typing-assist", &pair.0, &pair.1),
            "layout"
        );
    }

    #[test]
    fn user_corrections_are_opt_in_for_training() {
        assert!(!is_learnable_live_kind("user-correction", false));
        assert!(is_learnable_live_kind("user-correction", true));
    }

    #[test]
    fn causal_user_correction_compiles_a_positive_and_candidate_specific_anti() {
        let action = LiveAction {
            kind: "user-correction".to_string(),
            from_text: "провекра ".to_string(),
            to_text: "проверка ".to_string(),
            safety_allow_apply: None,
            lay_from: "провека ".to_string(),
            lay_to: "провекра ".to_string(),
        };
        let mut entries = Vec::new();

        push_causal_user_phase_entries(&mut entries, &action);

        assert_eq!(entries.len(), 2);
        assert!(entries
            .iter()
            .any(|entry| entry.accepted && entry.candidate == "проверка"));
        assert!(entries
            .iter()
            .any(|entry| !entry.accepted && entry.candidate == "провекра"));
    }

    #[test]
    fn unrelated_later_user_text_is_not_a_phase_label() {
        let action = LiveAction {
            kind: "user-correction".to_string(),
            from_text: "потом ".to_string(),
            to_text: "предложение ".to_string(),
            safety_allow_apply: None,
            lay_from: "птом ".to_string(),
            lay_to: "потом ".to_string(),
        };
        let mut entries = Vec::new();

        push_causal_user_phase_entries(&mut entries, &action);

        assert!(entries.is_empty());
    }

    #[test]
    fn llmwave_corpus_training_writes_phrase_memory_packet() {
        let dir =
            std::env::temp_dir().join(format!("lay-llmwave-corpus-train-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        let corpus = dir.join("book.txt");
        let out = dir.join("phrase_memory.llmw.bin");
        fs::write(
            &corpus,
            "на улице опять идёт дождь\nя хочу проверить автозамену\n",
        )
        .unwrap();

        train_llmwave_corpus(&corpus, &out).unwrap();

        let memory = llmwave::read_memory_packet(&out).unwrap();
        assert!(!memory.is_empty());
        assert!(memory.vocabulary_len() >= 6);
        let _ = fs::remove_dir_all(&dir);
    }
}
