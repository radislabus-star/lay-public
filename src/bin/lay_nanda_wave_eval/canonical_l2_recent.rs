use lay::nanda_wave::eval::{canonical_l2_candidate_report, CanonicalL2Candidate};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;

use super::real_suite;

const RECENT_ACTIONS_PATH: &str = ".local/share/lay/recent_actions.jsonl";
const HARVEST_PATH: &str = ".local/share/lay/nanda_wave/canonical_l2_harvest.jsonl";

#[derive(Debug, Deserialize)]
struct RecentAction {
    #[serde(default)]
    ts: u64,
    #[serde(default)]
    kind: String,
    #[serde(default)]
    from: String,
    #[serde(default)]
    to: String,
    #[serde(default)]
    input_gate: Option<RecentActionGate>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct HarvestRecord {
    ts: u64,
    from: String,
    to: String,
    input: String,
    live_kind: String,
    live_source: Option<String>,
    live_source_id: Option<String>,
    live_error_class: Option<String>,
    canonical_top: Option<String>,
    canonical_score: Option<u32>,
    canonical_l1_overlap: Option<usize>,
    canonical_l2_overlap: Option<usize>,
    canonical_motif_overlap: Option<usize>,
    canonical_prefix_match: Option<bool>,
    verdict: String,
}

#[derive(Debug, Deserialize)]
struct RecentActionGate {
    #[serde(default)]
    selected_source: Option<String>,
    #[serde(default)]
    selected_source_id: Option<String>,
    #[serde(default)]
    selected_error_class: Option<String>,
}

pub(crate) fn print_recent(args: &[String]) -> io::Result<()> {
    let limit = super::arg_value(args, "--limit")
        .and_then(|value| value.parse::<usize>().ok())
        .unwrap_or(30)
        .clamp(1, 200);
    let candidate_limit = super::arg_value(args, "--candidate-limit")
        .and_then(|value| value.parse::<usize>().ok())
        .unwrap_or(5)
        .clamp(1, 20);
    let path = super::arg_value(args, "--recent-actions")
        .map(PathBuf::from)
        .or_else(default_recent_actions_path)
        .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "HOME is not set"))?;
    let words = clean_words()?;
    let actions = load_recent_actions(&path, limit)?;

    println!("canonical_l2_recent:");
    println!("  source: {}", path.display());
    println!("  clean_words: {}", words.len());
    println!("  actions: {}", actions.len());
    println!("  live_authority: false");
    println!("  rows:");
    let mut counts = VerdictCounts::default();
    for action in actions {
        let input = last_word(&action.from);
        if input.is_empty() {
            continue;
        }
        let report = canonical_l2_candidate_report(&words, &input, candidate_limit);
        let top = report.candidates.first();
        let verdict = verdict(&action, top);
        counts.add(verdict);
        println!(
            "    {} -> {} | live={}{}{} | canonical={} | verdict={}",
            compact(&action.from),
            compact(&action.to),
            action.kind,
            action
                .input_gate
                .as_ref()
                .and_then(|gate| gate.selected_source.as_deref())
                .map(|source| format!("/{source}"))
                .unwrap_or_default(),
            action
                .input_gate
                .as_ref()
                .and_then(|gate| gate.selected_source_id.as_deref())
                .map(|source_id| format!("/{source_id}"))
                .unwrap_or_default(),
            format_top(top),
            verdict
        );
    }
    println!("  summary:");
    println!("    canonical_agrees_live: {}", counts.agrees_live);
    println!("    canonical_conflicts_live: {}", counts.conflicts_live);
    println!("    canonical_would_keep: {}", counts.would_keep);
    println!("    canonical_watch: {}", counts.watch);
    println!("    canonical_weak: {}", counts.weak);
    println!("    canonical_no_candidate: {}", counts.no_candidate);
    println!("    layout_route_skip: {}", counts.layout_route_skip);

    Ok(())
}

pub(crate) fn harvest_recent(args: &[String]) -> io::Result<()> {
    let limit = super::arg_value(args, "--limit")
        .and_then(|value| value.parse::<usize>().ok())
        .unwrap_or(200)
        .clamp(1, 2_000);
    let candidate_limit = super::arg_value(args, "--candidate-limit")
        .and_then(|value| value.parse::<usize>().ok())
        .unwrap_or(5)
        .clamp(1, 20);
    let max_records = super::arg_value(args, "--max-records")
        .and_then(|value| value.parse::<usize>().ok())
        .unwrap_or(50_000)
        .clamp(100, 500_000);
    let recent_path = super::arg_value(args, "--recent-actions")
        .map(PathBuf::from)
        .or_else(default_recent_actions_path)
        .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "HOME is not set"))?;
    let out_path = super::arg_value(args, "--out")
        .map(PathBuf::from)
        .or_else(default_harvest_path)
        .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "HOME is not set"))?;

    let words = clean_words()?;
    let actions = load_recent_actions(&recent_path, limit)?;
    let mut records = load_harvest_records(&out_path)?;
    let before = records.len();
    for action in actions {
        let input = last_word(&action.from);
        if input.is_empty() {
            continue;
        }
        let report = canonical_l2_candidate_report(&words, &input, candidate_limit);
        let top = report.candidates.first();
        let record = harvest_record(&action, &input, top);
        records.insert(harvest_key(&record), record);
    }
    trim_records(&mut records, max_records);
    write_harvest_records(&out_path, records.values())?;

    println!("canonical_l2_harvest:");
    println!("  recent_source: {}", recent_path.display());
    println!("  output: {}", out_path.display());
    println!("  clean_words: {}", words.len());
    println!("  before: {}", before);
    println!("  after: {}", records.len());
    println!("  added: {}", records.len().saturating_sub(before));
    println!("  max_records: {}", max_records);
    println!("  live_authority: false");

    Ok(())
}

pub(crate) fn print_harvest_summary(args: &[String]) -> io::Result<()> {
    let path = super::arg_value(args, "--harvest")
        .map(PathBuf::from)
        .or_else(default_harvest_path)
        .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "HOME is not set"))?;
    let records = load_harvest_records(&path)?;
    let mut counts = VerdictCounts::default();
    for record in records.values() {
        counts.add(&record.verdict);
    }

    println!("canonical_l2_harvest_summary:");
    println!("  source: {}", path.display());
    println!("  records: {}", records.len());
    println!("  canonical_agrees_live: {}", counts.agrees_live);
    println!("  canonical_conflicts_live: {}", counts.conflicts_live);
    println!("  canonical_would_keep: {}", counts.would_keep);
    println!("  canonical_watch: {}", counts.watch);
    println!("  canonical_weak: {}", counts.weak);
    println!("  canonical_no_candidate: {}", counts.no_candidate);
    println!("  layout_route_skip: {}", counts.layout_route_skip);
    println!("  top_conflicts:");
    for record in records
        .values()
        .filter(|record| record.verdict == "canonical_conflicts_live")
        .take(12)
    {
        println!(
            "    {} -> {} | input={} | canonical={}:{}",
            compact(&record.from),
            compact(&record.to),
            record.input,
            record.canonical_top.as_deref().unwrap_or("none"),
            record.canonical_score.unwrap_or(0)
        );
    }

    Ok(())
}

pub(crate) fn replay_harvest(args: &[String]) -> io::Result<()> {
    let path = super::arg_value(args, "--harvest")
        .map(PathBuf::from)
        .or_else(default_harvest_path)
        .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "HOME is not set"))?;
    let limit = super::arg_value(args, "--limit")
        .and_then(|value| value.parse::<usize>().ok())
        .unwrap_or(40)
        .clamp(1, 500);
    let min_score = super::arg_value(args, "--min-score")
        .and_then(|value| value.parse::<u32>().ok())
        .unwrap_or(650);
    let records = load_harvest_records(&path)?;
    let mut rows = records
        .values()
        .filter(|record| !matches!(record.live_error_class.as_deref(), Some("wrong_layout")))
        .filter(|record| record.canonical_score.unwrap_or(0) >= min_score)
        .collect::<Vec<_>>();
    rows.sort_by(|left, right| {
        right
            .canonical_score
            .unwrap_or(0)
            .cmp(&left.canonical_score.unwrap_or(0))
            .then_with(|| left.input.cmp(&right.input))
    });
    rows.truncate(limit);

    let mut live_match = 0usize;
    let mut live_conflict = 0usize;
    println!("canonical_l2_replay:");
    println!("  source: {}", path.display());
    println!("  records: {}", records.len());
    println!("  min_score: {}", min_score);
    println!("  rows: {}", rows.len());
    println!("  live_authority: false");
    for record in &rows {
        let simulated = simulate_replace_last_word(&record.from, record.canonical_top.as_deref());
        let matches_live = normalize_sentence(&simulated) == normalize_sentence(&record.to);
        if matches_live {
            live_match += 1;
        } else {
            live_conflict += 1;
        }
        println!(
            "    {} => {} | live={} | canonical={}:{} | {}",
            compact(&record.from),
            compact(&simulated),
            compact(&record.to),
            record.canonical_top.as_deref().unwrap_or("none"),
            record.canonical_score.unwrap_or(0),
            if matches_live {
                "matches_live"
            } else {
                "differs_from_live"
            }
        );
    }
    println!("  summary:");
    println!("    matches_live: {}", live_match);
    println!("    differs_from_live: {}", live_conflict);

    Ok(())
}

pub(crate) fn replay_harvest_with_morphology(args: &[String]) -> io::Result<()> {
    let path = super::arg_value(args, "--harvest")
        .map(PathBuf::from)
        .or_else(default_harvest_path)
        .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "HOME is not set"))?;
    let limit = super::arg_value(args, "--limit")
        .and_then(|value| value.parse::<usize>().ok())
        .unwrap_or(40)
        .clamp(1, 500);
    let min_score = super::arg_value(args, "--min-score")
        .and_then(|value| value.parse::<u32>().ok())
        .unwrap_or(650);
    let records = load_harvest_records(&path)?;
    let words = learned_clean_words(&records)?;
    let mut rows = records
        .values()
        .filter(|record| !matches!(record.live_error_class.as_deref(), Some("wrong_layout")))
        .filter(|record| record.canonical_score.unwrap_or(0) >= min_score)
        .collect::<Vec<_>>();
    rows.sort_by(|left, right| {
        right
            .canonical_score
            .unwrap_or(0)
            .cmp(&left.canonical_score.unwrap_or(0))
            .then_with(|| left.input.cmp(&right.input))
    });
    rows.truncate(limit);

    let mut raw_match = 0usize;
    let mut raw_conflict = 0usize;
    let mut morph_match = 0usize;
    let mut morph_conflict = 0usize;
    let mut morph_changed = 0usize;
    let mut morph_missing = 0usize;
    println!("canonical_l2_morph_replay:");
    println!("  source: {}", path.display());
    println!("  records: {}", records.len());
    println!("  clean_words: {}", words.len());
    println!("  learned_from_harvest: true");
    println!("  replay_isolation: optimistic_same_harvest");
    println!("  min_score: {}", min_score);
    println!("  rows: {}", rows.len());
    println!("  live_authority: false");
    for record in &rows {
        let raw_simulated =
            simulate_replace_last_word(&record.from, record.canonical_top.as_deref());
        let morph = morphology_candidate(&record.input, record.canonical_top.as_deref(), &words);
        let morph_simulated = simulate_replace_last_word(&record.from, morph.as_deref());
        let raw_matches_live = normalize_sentence(&raw_simulated) == normalize_sentence(&record.to);
        let morph_matches_live =
            normalize_sentence(&morph_simulated) == normalize_sentence(&record.to);
        if raw_matches_live {
            raw_match += 1;
        } else {
            raw_conflict += 1;
        }
        if morph_matches_live {
            morph_match += 1;
        } else {
            morph_conflict += 1;
        }
        if morph.is_none() {
            morph_missing += 1;
        }
        if morph.as_deref() != record.canonical_top.as_deref() {
            morph_changed += 1;
        }
        println!(
            "    {} => raw:{} => morph:{} | live={} | canonical={}:{} | morph={} | {}",
            compact(&record.from),
            compact(&raw_simulated),
            compact(&morph_simulated),
            compact(&record.to),
            record.canonical_top.as_deref().unwrap_or("none"),
            record.canonical_score.unwrap_or(0),
            morph.as_deref().unwrap_or("none"),
            if morph_matches_live {
                "morph_matches_live"
            } else if raw_matches_live {
                "raw_matches_live"
            } else {
                "differs_from_live"
            }
        );
    }
    println!("  summary:");
    println!("    raw_matches_live: {}", raw_match);
    println!("    raw_differs_from_live: {}", raw_conflict);
    println!("    morph_matches_live: {}", morph_match);
    println!("    morph_differs_from_live: {}", morph_conflict);
    println!("    morph_changed: {}", morph_changed);
    println!("    morph_missing: {}", morph_missing);

    Ok(())
}

#[derive(Default)]
struct VerdictCounts {
    agrees_live: usize,
    conflicts_live: usize,
    would_keep: usize,
    watch: usize,
    weak: usize,
    no_candidate: usize,
    layout_route_skip: usize,
}

impl VerdictCounts {
    fn add(&mut self, verdict: &str) {
        match verdict {
            "canonical_agrees_live" => self.agrees_live += 1,
            "canonical_conflicts_live" => self.conflicts_live += 1,
            "canonical_would_keep" => self.would_keep += 1,
            "canonical_watch" => self.watch += 1,
            "canonical_weak" => self.weak += 1,
            "canonical_no_candidate" => self.no_candidate += 1,
            "layout_route_skip" => self.layout_route_skip += 1,
            _ => {}
        }
    }
}

fn clean_words() -> io::Result<Vec<String>> {
    let suite = real_suite::load()?;
    let mut words = BTreeSet::new();
    for case in &suite.cases {
        collect_words(&case.expected, &mut words);
    }
    for word in [
        "и", "в", "не", "на", "с", "по", "для", "как", "что", "я", "ты", "мы", "он", "она", "они",
    ] {
        words.insert(word.to_string());
    }
    Ok(words.into_iter().collect())
}

fn learned_clean_words(records: &BTreeMap<String, HarvestRecord>) -> io::Result<Vec<String>> {
    let mut words = clean_words()?.into_iter().collect::<BTreeSet<_>>();
    for record in records.values() {
        collect_words(&record.to, &mut words);
    }
    Ok(words.into_iter().collect())
}

fn collect_words(text: &str, out: &mut BTreeSet<String>) {
    for token in text.split_whitespace() {
        let word = normalize_word(token);
        if !word.is_empty() {
            out.insert(word);
        }
    }
}

fn load_recent_actions(path: &PathBuf, limit: usize) -> io::Result<Vec<RecentAction>> {
    let text = fs::read_to_string(path)?;
    let mut actions = text
        .lines()
        .rev()
        .filter_map(|line| serde_json::from_str::<RecentAction>(line).ok())
        .filter(|action| action.kind == "typing-assist" || action.kind == "layout-replay")
        .take(limit)
        .collect::<Vec<_>>();
    actions.reverse();
    Ok(actions)
}

fn load_harvest_records(path: &PathBuf) -> io::Result<BTreeMap<String, HarvestRecord>> {
    let Ok(text) = fs::read_to_string(path) else {
        return Ok(BTreeMap::new());
    };
    Ok(text
        .lines()
        .filter_map(|line| serde_json::from_str::<HarvestRecord>(line).ok())
        .map(|record| (harvest_key(&record), record))
        .collect())
}

fn write_harvest_records<'a, I>(path: &PathBuf, records: I) -> io::Result<()>
where
    I: IntoIterator<Item = &'a HarvestRecord>,
{
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let mut file = fs::File::create(path)?;
    for record in records {
        serde_json::to_writer(&mut file, record)?;
        file.write_all(b"\n")?;
    }
    Ok(())
}

fn harvest_record(
    action: &RecentAction,
    input: &str,
    candidate: Option<&CanonicalL2Candidate>,
) -> HarvestRecord {
    let verdict = verdict(action, candidate).to_string();
    HarvestRecord {
        ts: action.ts,
        from: action.from.clone(),
        to: action.to.clone(),
        input: input.to_string(),
        live_kind: action.kind.clone(),
        live_source: action
            .input_gate
            .as_ref()
            .and_then(|gate| gate.selected_source.clone()),
        live_source_id: action
            .input_gate
            .as_ref()
            .and_then(|gate| gate.selected_source_id.clone()),
        live_error_class: action
            .input_gate
            .as_ref()
            .and_then(|gate| gate.selected_error_class.clone()),
        canonical_top: candidate.map(|candidate| candidate.word.clone()),
        canonical_score: candidate.map(|candidate| candidate.score),
        canonical_l1_overlap: candidate.map(|candidate| candidate.l1_overlap),
        canonical_l2_overlap: candidate.map(|candidate| candidate.l2_overlap),
        canonical_motif_overlap: candidate.map(|candidate| candidate.motif_overlap),
        canonical_prefix_match: candidate.map(|candidate| candidate.prefix_match),
        verdict,
    }
}

fn harvest_key(record: &HarvestRecord) -> String {
    format!("{}:{}:{}", record.ts, record.from, record.to)
}

fn trim_records(records: &mut BTreeMap<String, HarvestRecord>, max_records: usize) {
    while records.len() > max_records {
        let Some(key) = records.keys().next().cloned() else {
            break;
        };
        records.remove(&key);
    }
}

fn default_recent_actions_path() -> Option<PathBuf> {
    std::env::var_os("HOME")
        .map(PathBuf::from)
        .map(|home| home.join(RECENT_ACTIONS_PATH))
}

fn default_harvest_path() -> Option<PathBuf> {
    std::env::var_os("HOME")
        .map(PathBuf::from)
        .map(|home| home.join(HARVEST_PATH))
}

fn last_word(text: &str) -> String {
    text.split_whitespace()
        .rev()
        .map(normalize_word)
        .find(|word| !word.is_empty())
        .unwrap_or_default()
}

fn normalize_word(text: &str) -> String {
    text.chars()
        .filter(|ch| ch.is_alphabetic() || *ch == '-')
        .flat_map(char::to_lowercase)
        .collect()
}

fn normalize_sentence(text: &str) -> String {
    text.split_whitespace()
        .map(normalize_word)
        .filter(|word| !word.is_empty())
        .collect::<Vec<_>>()
        .join(" ")
}

fn morphology_candidate(
    input: &str,
    canonical_top: Option<&str>,
    clean_words: &[String],
) -> Option<String> {
    let input = normalize_word(input);
    let canonical_top = canonical_top.map(normalize_word);
    if input.is_empty() {
        return canonical_top;
    }
    let input_len = char_len(&input);
    let max_distance = (input_len / 3 + 1).max(2);
    let mut best: Option<(i32, String)> = None;
    for word in clean_words {
        let word = normalize_word(word);
        if word.is_empty() || word == input {
            continue;
        }
        let word_len = char_len(&word);
        if word_len + 2 < input_len || word_len > input_len + 4 {
            continue;
        }
        let input_distance = edit_distance(&input, &word);
        if input_distance > max_distance {
            continue;
        }
        let canonical_distance = canonical_top
            .as_deref()
            .map(|canonical| edit_distance(canonical, &word))
            .unwrap_or(input_distance);
        let input_prefix = common_prefix_len(&input, &word);
        let canonical_prefix = canonical_top
            .as_deref()
            .map(|canonical| common_prefix_len(canonical, &word))
            .unwrap_or(0);
        if input_prefix < 2 && canonical_prefix < 2 {
            continue;
        }
        let input_suffix = common_suffix_len(&input, &word);
        let len_delta = input_len.abs_diff(word_len);
        let mut score = 10_000i32;
        score -= input_distance as i32 * 1_000;
        score -= canonical_distance as i32 * 25;
        score -= len_delta as i32 * 160;
        score += input_prefix.min(5) as i32 * 120;
        score += canonical_prefix.min(5) as i32 * 60;
        score += input_suffix.min(5) as i32 * 80;
        if canonical_top.as_deref() == Some(word.as_str()) {
            score += 250;
        }
        if canonical_top
            .as_deref()
            .is_some_and(|canonical| word.starts_with(canonical) || canonical.starts_with(&word))
        {
            score += 120;
        }
        let should_replace = match best.as_ref() {
            Some((best_score, best_word)) => {
                score > *best_score || score == *best_score && word < *best_word
            }
            None => true,
        };
        if should_replace {
            best = Some((score, word));
        }
    }
    best.map(|(_, word)| word).or(canonical_top)
}

fn simulate_replace_last_word(text: &str, replacement: Option<&str>) -> String {
    let Some(replacement) = replacement else {
        return text.to_string();
    };
    let Some((start, end)) = last_word_span(text) else {
        return text.to_string();
    };
    let mut output = String::with_capacity(text.len() + replacement.len());
    output.push_str(&text[..start]);
    output.push_str(replacement);
    output.push_str(&text[end..]);
    output
}

fn char_len(text: &str) -> usize {
    text.chars().count()
}

fn common_prefix_len(left: &str, right: &str) -> usize {
    left.chars()
        .zip(right.chars())
        .take_while(|(left, right)| left == right)
        .count()
}

fn common_suffix_len(left: &str, right: &str) -> usize {
    left.chars()
        .rev()
        .zip(right.chars().rev())
        .take_while(|(left, right)| left == right)
        .count()
}

fn edit_distance(left: &str, right: &str) -> usize {
    let left = left.chars().collect::<Vec<_>>();
    let right = right.chars().collect::<Vec<_>>();
    let mut prev = (0..=right.len()).collect::<Vec<_>>();
    let mut curr = vec![0usize; right.len() + 1];
    for (left_idx, left_ch) in left.iter().enumerate() {
        curr[0] = left_idx + 1;
        for (right_idx, right_ch) in right.iter().enumerate() {
            let replace_cost = usize::from(left_ch != right_ch);
            curr[right_idx + 1] = (prev[right_idx + 1] + 1)
                .min(curr[right_idx] + 1)
                .min(prev[right_idx] + replace_cost);
        }
        std::mem::swap(&mut prev, &mut curr);
    }
    prev[right.len()]
}

fn last_word_span(text: &str) -> Option<(usize, usize)> {
    let mut end = None;
    for (idx, ch) in text.char_indices().rev() {
        if ch.is_alphabetic() || ch == '-' {
            end.get_or_insert(idx + ch.len_utf8());
        } else if let Some(end) = end {
            return Some((idx + ch.len_utf8(), end));
        }
    }
    end.map(|end| (0, end))
}

fn format_top(candidate: Option<&CanonicalL2Candidate>) -> String {
    candidate
        .map(|candidate| {
            format!(
                "{}:{} l1={} l2={} motif={} prefix={}",
                candidate.word,
                candidate.score,
                candidate.l1_overlap,
                candidate.l2_overlap,
                candidate.motif_overlap,
                candidate.prefix_match
            )
        })
        .unwrap_or_else(|| "none".to_string())
}

fn verdict(action: &RecentAction, candidate: Option<&CanonicalL2Candidate>) -> &'static str {
    let from_last = last_word(&action.from);
    let to_last = last_word(&action.to);
    let Some(candidate) = candidate else {
        return "canonical_no_candidate";
    };
    let candidate_word = normalize_word(&candidate.word);
    let live_error_class = action
        .input_gate
        .as_ref()
        .and_then(|gate| gate.selected_error_class.as_deref());
    if live_error_class == Some("wrong_layout") {
        return "layout_route_skip";
    }
    if candidate_word == to_last && to_last != from_last {
        return "canonical_agrees_live";
    }
    if candidate_word == from_last && to_last != from_last {
        return "canonical_would_keep";
    }
    if !to_last.is_empty() && candidate_word != to_last && candidate.score >= 700 {
        return "canonical_conflicts_live";
    }
    if candidate.score < 250 {
        return "canonical_weak";
    }
    "canonical_watch"
}

fn compact(text: &str) -> String {
    let compact = text.split_whitespace().collect::<Vec<_>>().join(" ");
    let chars = compact.chars().collect::<Vec<_>>();
    if chars.len() <= 28 {
        return compact;
    }
    let tail = chars[chars.len() - 28..].iter().collect::<String>();
    format!("...{tail}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn last_word_normalizes_tail() {
        assert_eq!(last_word("ну и что видешь "), "видешь");
        assert_eq!(last_word("html djn "), "djn");
    }

    #[test]
    fn simulate_replaces_last_word_only() {
        assert_eq!(
            simulate_replace_last_word("сейчас видешь ", Some("видишь")),
            "сейчас видишь "
        );
        assert_eq!(
            simulate_replace_last_word("в предлажение?!", Some("предложения")),
            "в предложения?!"
        );
    }

    #[test]
    fn morphology_replay_recovers_known_forms() {
        let words = [
            "предлогами",
            "предложение",
            "словарные",
            "видишь",
            "правильно",
        ]
        .into_iter()
        .map(str::to_string)
        .collect::<Vec<_>>();
        assert_eq!(
            morphology_candidate("предлгами", Some("предлог"), &words).as_deref(),
            Some("предлогами")
        );
        assert_eq!(
            morphology_candidate("словарыне", Some("слова"), &words).as_deref(),
            Some("словарные")
        );
        assert_eq!(
            morphology_candidate("предлажение", Some("предложения"), &words).as_deref(),
            Some("предложение")
        );
        assert_eq!(
            morphology_candidate("видешь", Some("видишь"), &words).as_deref(),
            Some("видишь")
        );
    }
}
