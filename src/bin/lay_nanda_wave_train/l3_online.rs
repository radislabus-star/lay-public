use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs;
use std::io::{self, Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

const STATE_FORMAT: &str = "lay-l3-online-v1";
const MIN_SCENES: usize = 2;
const MAX_SCENES: usize = 8;
const MAX_RELATIONS: usize = 128;

#[derive(Clone, Debug, Deserialize)]
struct UsageEvent {
    #[serde(default)]
    kind: String,
    #[serde(default)]
    outcome: String,
    #[serde(default)]
    source: String,
    #[serde(default)]
    word: Option<String>,
    #[serde(default)]
    context: Vec<String>,
    #[serde(default)]
    from: Option<String>,
    #[serde(default)]
    to: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct PendingRelation {
    rejected: String,
    expected: String,
    scenes: Vec<String>,
    last_attempted_scenes: usize,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct OnlineState {
    format: String,
    source_offset: u64,
    generation: u64,
    #[serde(default)]
    pending: BTreeMap<String, PendingRelation>,
}

impl Default for OnlineState {
    fn default() -> Self {
        Self {
            format: STATE_FORMAT.to_string(),
            source_offset: 0,
            generation: 0,
            pending: BTreeMap::new(),
        }
    }
}

struct Paths {
    root: PathBuf,
    usage_events: PathBuf,
    corrections: PathBuf,
    base: PathBuf,
    manifest: PathBuf,
    state: PathBuf,
}

impl Paths {
    fn discover() -> io::Result<Self> {
        let home = std::env::var_os("HOME")
            .map(PathBuf::from)
            .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "HOME is not set"))?;
        let wave = home.join(".local/share/lay/nanda_wave");
        let root = wave.join("l3-online");
        Ok(Self {
            usage_events: wave.join("word_usage_events.jsonl"),
            corrections: home.join(".local/share/lay/corrections.jsonl"),
            base: wave.join("l3_context_phase.nwpc"),
            manifest: wave.join("l3_context_phase.runtime.json"),
            state: root.join("state.json"),
            root,
        })
    }
}

pub(super) fn run(args: &[String]) -> io::Result<()> {
    let once = args.iter().any(|arg| arg == "--once");
    let poll_ms = arg_u64(args, "--poll-ms").unwrap_or(5_000).max(250);
    let paths = Paths::discover()?;
    fs::create_dir_all(&paths.root)?;
    ensure_manifest(&paths)?;
    let state_exists = paths.state.is_file();
    let mut state = load_state(&paths.state)?;

    if initialize_source_offset(&paths, &mut state, state_exists)? {
        println!(
            "{}",
            serde_json::json!({
                "kind": "l3_online_initialized",
                "manifest": paths.manifest,
                "source_offset": state.source_offset,
                "historical_events_replayed": false,
            })
        );
    }

    loop {
        if let Err(error) = process_once(&paths, &mut state) {
            eprintln!("lay-l3-online cycle failed: {error}");
        }
        if once {
            return Ok(());
        }
        thread::sleep(Duration::from_millis(poll_ms));
    }
}

fn initialize_source_offset(
    paths: &Paths,
    state: &mut OnlineState,
    state_exists: bool,
) -> io::Result<bool> {
    if state_exists {
        return Ok(false);
    }
    state.source_offset = fs::metadata(&paths.usage_events)
        .map(|metadata| metadata.len())
        .unwrap_or_default();
    save_state(&paths.state, state)?;
    Ok(true)
}

fn process_once(paths: &Paths, state: &mut OnlineState) -> io::Result<()> {
    let appended = read_appended_text(&paths.usage_events, &mut state.source_offset)?;
    let mut observations = 0_usize;
    for line in appended.lines() {
        let Ok(event) = serde_json::from_str::<UsageEvent>(line) else {
            continue;
        };
        let Some((key, rejected, expected, scene)) = relation_observation(&event) else {
            continue;
        };
        observations = observations.saturating_add(1);
        let relation = state.pending.entry(key).or_insert_with(|| PendingRelation {
            rejected,
            expected,
            scenes: Vec::new(),
            last_attempted_scenes: 0,
        });
        if !relation.scenes.contains(&scene) && relation.scenes.len() < MAX_SCENES {
            relation.scenes.push(scene);
        }
    }
    while state.pending.len() > MAX_RELATIONS {
        let Some(key) = state.pending.keys().next().cloned() else {
            break;
        };
        state.pending.remove(&key);
    }
    save_state(&paths.state, state)?;

    let ready = state.pending.iter().find_map(|(key, relation)| {
        let count = relation.scenes.len();
        (count >= MIN_SCENES && count > relation.last_attempted_scenes && count.is_power_of_two())
            .then(|| key.clone())
    });
    let Some(key) = ready else {
        if observations > 0 {
            println!(
                "{}",
                serde_json::json!({
                    "kind": "l3_online_feedback_recorded",
                    "new_observations": observations,
                    "pending_relations": state.pending.len(),
                })
            );
        }
        return Ok(());
    };

    let relation = state.pending.get(&key).cloned().expect("ready relation");
    let generation = state.generation.saturating_add(1);
    let report = attempt_relation(paths, generation, &relation)?;
    let passed = report.get("verdict").and_then(|value| value.as_str()) == Some("PASS");
    state.generation = generation;
    if passed {
        state.pending.remove(&key);
    } else if let Some(relation) = state.pending.get_mut(&key) {
        relation.last_attempted_scenes = relation.scenes.len();
    }
    save_state(&paths.state, state)?;
    println!("{report}");
    Ok(())
}

fn attempt_relation(
    paths: &Paths,
    generation: u64,
    relation: &PendingRelation,
) -> io::Result<serde_json::Value> {
    let stem = format!("delta-{generation:08}-{}", unix_time());
    let corpus = paths.root.join(format!("{stem}.txt"));
    let cases = paths.root.join(format!("{stem}.cases.tsv"));
    let delta = paths.root.join(format!("{stem}.nwpc"));
    let receipt = paths.root.join(format!("{stem}.proof.json"));
    // Two distinct scenes establish independence; replaying each scene once
    // more gives the online learner its separate profile-admission and
    // coherent-center observations without making a second corpus pass.
    let corpus_text = relation
        .scenes
        .iter()
        .flat_map(|scene| [scene.as_str(), scene.as_str()])
        .collect::<Vec<_>>()
        .join("\n");
    write_private(&corpus, &format!("{corpus_text}\n"))?;
    write_private(&cases, &targeted_cases(relation))?;

    let compile = lay::nanda_wave::compile_l3_context_delta_for_manifest(
        &paths.manifest,
        &corpus,
        &paths.corrections,
        &delta,
        2,
        2,
    )?;
    let proof = lay::nanda_wave::prove_l3_context_delta_targeted(
        &paths.manifest,
        &delta,
        &cases,
        &receipt,
    )?;
    let passed = proof_passed(&proof);
    let admission = if passed {
        Some(lay::nanda_wave::admit_l3_context_delta(
            &paths.manifest,
            &delta,
            Some(&receipt),
            Some("local-online"),
        )?)
    } else {
        None
    };
    Ok(serde_json::json!({
        "kind": "l3_online_delta_attempt",
        "generation": generation,
        "rejected": relation.rejected,
        "expected": relation.expected,
        "independent_scenes": relation.scenes.len(),
        "corpus_passes": compile.get("corpus_passes"),
        "delta": delta,
        "proof_receipt": receipt,
        "false_supports": proof.get("false_supports"),
        "verdict": proof.get("verdict"),
        "admission": admission,
        "base_rewritten": false,
    }))
}

fn proof_passed(proof: &serde_json::Value) -> bool {
    proof.get("verdict").and_then(serde_json::Value::as_str) == Some("PASS")
        && proof
            .get("false_supports")
            .and_then(serde_json::Value::as_u64)
            == Some(0)
}

fn relation_observation(event: &UsageEvent) -> Option<(String, String, String, String)> {
    if event.kind != "accepted_fix"
        || event.outcome != "confirmed_positive"
        || event.source != "user_correction"
        || event.context.len() < 2
    {
        return None;
    }
    let rejected = rejected_word(event)?;
    let expected = normalize_word(event.word.as_deref()?)?;
    if rejected == expected {
        return None;
    }
    let context = event
        .context
        .iter()
        .filter_map(|token| normalize_word(token))
        .collect::<Vec<_>>();
    if context.len() < 2 {
        return None;
    }
    let scene = context
        .iter()
        .cloned()
        .chain(std::iter::once(expected.clone()))
        .collect::<Vec<_>>()
        .join(" ");
    let key = format!("{rejected}\u{1f}{expected}");
    Some((key, rejected, expected, scene))
}

fn rejected_word(event: &UsageEvent) -> Option<String> {
    let from_words = normalized_words(event.from.as_deref()?);
    let to_words = normalized_words(event.to.as_deref()?);
    if from_words.len() != to_words.len() {
        return None;
    }
    let changed = from_words
        .iter()
        .zip(&to_words)
        .enumerate()
        .filter_map(|(index, (left, right))| (left != right).then_some(index))
        .collect::<Vec<_>>();
    let index = *changed.first()?;
    (changed.len() == 1 && index + 1 == to_words.len()).then(|| from_words[index].clone())
}

fn normalized_words(text: &str) -> Vec<String> {
    text.split_whitespace().filter_map(normalize_word).collect()
}

fn normalize_word(raw: &str) -> Option<String> {
    let word = raw
        .trim_matches(|ch: char| !ch.is_alphanumeric() && ch != '-')
        .to_lowercase();
    (!word.is_empty()
        && word.chars().count() <= 48
        && word.chars().all(|ch| ch.is_alphabetic() || ch == '-'))
    .then_some(word)
}

fn targeted_cases(relation: &PendingRelation) -> String {
    let mut rows = vec!["# kind\tcontext\tcandidates\texpected".to_string()];
    for scene in &relation.scenes {
        let mut words = scene.split_whitespace().collect::<Vec<_>>();
        let _ = words.pop();
        rows.push(format!(
            "improve\t{}\t{}|{}\t{}",
            words.join(" "),
            relation.expected,
            relation.rejected,
            relation.expected
        ));
    }
    for context in [
        "компилятор проверяет исходный код",
        "система сохраняет чистый сигнал",
        "пользователь открыл новое окно",
        "модель читает другой контекст",
    ] {
        rows.push(format!(
            "safety\t{context}\t{}|{}\t{}",
            relation.expected, relation.rejected, relation.rejected
        ));
    }
    rows.push(format!(
        "safety\tизолированная проверка\t{}|{}\t-",
        relation.expected, relation.rejected
    ));
    format!("{}\n", rows.join("\n"))
}

fn ensure_manifest(paths: &Paths) -> io::Result<()> {
    if paths.manifest.is_file() {
        return Ok(());
    }
    if !paths.base.is_file() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!("L3 base package is missing: {}", paths.base.display()),
        ));
    }
    lay::nanda_wave::initialize_l3_context_composite_manifest(&paths.manifest, &paths.base)?;
    Ok(())
}

fn read_appended_text(path: &Path, offset: &mut u64) -> io::Result<String> {
    let mut file = match fs::File::open(path) {
        Ok(file) => file,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(String::new()),
        Err(error) => return Err(error),
    };
    let len = file.metadata()?.len();
    if len < *offset {
        *offset = 0;
    }
    file.seek(SeekFrom::Start(*offset))?;
    let mut bytes = Vec::new();
    file.read_to_end(&mut bytes)?;
    *offset = len;
    Ok(String::from_utf8_lossy(&bytes).into_owned())
}

fn load_state(path: &Path) -> io::Result<OnlineState> {
    let Ok(bytes) = fs::read(path) else {
        return Ok(OnlineState::default());
    };
    let state: OnlineState = serde_json::from_slice(&bytes).map_err(io::Error::other)?;
    if state.format != STATE_FORMAT {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("unsupported L3 online state format: {}", state.format),
        ));
    }
    Ok(state)
}

fn save_state(path: &Path, state: &OnlineState) -> io::Result<()> {
    let mut bytes = serde_json::to_vec_pretty(state).map_err(io::Error::other)?;
    bytes.push(b'\n');
    lay::private_file::write_private_bytes(path, &bytes)
}

fn write_private(path: &Path, text: &str) -> io::Result<()> {
    lay::private_file::write_private_text(path, text)
}

fn arg_u64(args: &[String], flag: &str) -> Option<u64> {
    args.iter()
        .position(|arg| arg == flag)
        .and_then(|index| args.get(index + 1))
        .and_then(|value| value.parse().ok())
}

fn unix_time() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn accepted_fix_requires_context_and_one_tail_change() {
        let event: UsageEvent = serde_json::from_str(
            r#"{"kind":"accepted_fix","outcome":"confirmed_positive","source":"user_correction","word":"ходу","context":["обновлять","модель","по"],"from":"обновлять модель по ход","to":"обновлять модель по ходу"}"#,
        )
        .unwrap();
        let (key, _, _, scene) = relation_observation(&event).unwrap();
        assert_eq!(key, "ход\u{1f}ходу");
        assert_eq!(scene, "обновлять модель по ходу");
    }

    #[test]
    fn targeted_proof_has_improvements_and_sentinels() {
        let relation = PendingRelation {
            rejected: "ход".to_string(),
            expected: "ходу".to_string(),
            scenes: vec![
                "обновлять модель по ходу".to_string(),
                "менять параметры по ходу".to_string(),
            ],
            last_attempted_scenes: 0,
        };
        let cases = targeted_cases(&relation);
        assert_eq!(
            cases
                .lines()
                .filter(|line| line.starts_with("improve\t"))
                .count(),
            2
        );
        assert_eq!(
            cases
                .lines()
                .filter(|line| line.starts_with("safety\t"))
                .count(),
            5
        );
    }

    #[test]
    fn watch_or_false_support_never_opens_admission() {
        assert!(!proof_passed(
            &serde_json::json!({"verdict": "WATCH", "false_supports": 0})
        ));
        assert!(!proof_passed(
            &serde_json::json!({"verdict": "PASS", "false_supports": 1})
        ));
        assert!(proof_passed(
            &serde_json::json!({"verdict": "PASS", "false_supports": 0})
        ));
    }

    #[test]
    fn empty_journal_initialization_does_not_skip_first_appended_events() {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root = std::env::temp_dir().join(format!("lay-l3-online-empty-{unique}"));
        let wave = root.join("wave");
        let online = wave.join("l3-online");
        fs::create_dir_all(&online).unwrap();
        let paths = Paths {
            root: online.clone(),
            usage_events: wave.join("events.jsonl"),
            corrections: wave.join("corrections.jsonl"),
            base: wave.join("base.nwpc"),
            manifest: wave.join("runtime.json"),
            state: online.join("state.json"),
        };
        fs::write(&paths.usage_events, []).unwrap();
        let mut state = OnlineState::default();
        assert!(initialize_source_offset(&paths, &mut state, false).unwrap());
        assert_eq!(state.source_offset, 0);

        fs::write(&paths.usage_events, b"{\"kind\":\"accepted_fix\"}\n").unwrap();
        assert!(!initialize_source_offset(&paths, &mut state, true).unwrap());
        assert_eq!(state.source_offset, 0);
        assert!(
            !read_appended_text(&paths.usage_events, &mut state.source_offset)
                .unwrap()
                .is_empty()
        );
        let _ = fs::remove_dir_all(root);
    }
}
