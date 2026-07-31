#[path = "l3_online/feedback.rs"]
mod feedback;
#[path = "l3_online/journal.rs"]
mod journal;
#[path = "l3_online/proof_chain.rs"]
mod proof_chain;

use feedback::{
    enforce_relation_bound, insert_relation_observation, relation_observation, ObservationSource,
    OnlineState, UsageEvent, MIN_SCENES, STATE_FORMAT,
};
use proof_chain::attempt_relation;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::thread;
use std::time::Duration;

struct Paths {
    root: PathBuf,
    usage_events: PathBuf,
    corrections: PathBuf,
    base: PathBuf,
    manifest: PathBuf,
    state: PathBuf,
    full_proof_corpus: PathBuf,
    full_proof_surface: PathBuf,
}

impl Paths {
    fn discover() -> io::Result<Self> {
        let home = std::env::var_os("HOME")
            .map(PathBuf::from)
            .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "HOME is not set"))?;
        let wave = home.join(".local/share/lay/nanda_wave");
        let root = wave.join("l3-online");
        let proof_root = wave.join("l3-proof");
        Ok(Self {
            usage_events: wave.join("word_usage_events.jsonl"),
            corrections: home.join(".local/share/lay/corrections.jsonl"),
            base: wave.join("l3_context_phase.nwpc"),
            manifest: wave.join("l3_context_phase.runtime.json"),
            state: root.join("state.json"),
            full_proof_corpus: std::env::var_os("LAY_L3_ONLINE_FULL_PROOF_CORPUS")
                .map(PathBuf::from)
                .unwrap_or_else(|| proof_root.join("fixed-base-corpus-80k.txt")),
            full_proof_surface: std::env::var_os("LAY_L3_ONLINE_FULL_PROOF_SURFACE")
                .map(PathBuf::from)
                .unwrap_or_else(|| proof_root.join("surface-geometry-exact.jsonl")),
            root,
        })
    }
}

pub(super) fn run(args: &[String]) -> io::Result<()> {
    let once = args.iter().any(|arg| arg == "--once");
    let replay_existing = args.iter().any(|arg| arg == "--replay-existing-feedback");
    let poll_ms = arg_u64(args, "--poll-ms").unwrap_or(5_000).max(250);
    let paths = Paths::discover()?;
    fs::create_dir_all(&paths.root)?;
    ensure_manifest(&paths)?;
    let state_exists = paths.state.is_file();
    let mut state = load_state(&paths.state)?;

    if initialize_source_cursor(&paths, &mut state, state_exists)? {
        println!(
            "{}",
            serde_json::json!({
                "kind": "l3_online_initialized",
                "manifest": paths.manifest,
                "source_offset": state.source_offset,
                "source_inode": state.source_inode,
                "historical_events_replayed": false,
            })
        );
    }
    if replay_existing {
        println!("{}", replay_existing_feedback(&paths, &mut state)?);
        save_state(&paths.state, &state)?;
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

fn initialize_source_cursor(
    paths: &Paths,
    state: &mut OnlineState,
    state_exists: bool,
) -> io::Result<bool> {
    if state_exists && state.source_inode != 0 {
        return Ok(false);
    }
    journal::initialize_cursor(&paths.usage_events, state)?;
    save_state(&paths.state, state)?;
    Ok(true)
}

fn process_once(paths: &Paths, state: &mut OnlineState) -> io::Result<()> {
    let batch = journal::read_new_events(&paths.usage_events, state)?;
    match batch.mode {
        journal::JournalReadMode::Append => {}
        journal::JournalReadMode::Compacted => {
            state.feedback.journal_compactions =
                state.feedback.journal_compactions.saturating_add(1);
        }
        journal::JournalReadMode::Reanchored => {
            state.feedback.journal_reanchors_without_overlap = state
                .feedback
                .journal_reanchors_without_overlap
                .saturating_add(1);
        }
    }
    let mut observations = 0_usize;
    let mut direct_observations = 0_usize;
    let mut partial_ime_edit_observations = 0_usize;
    let mut ime_choice_observations = 0_usize;
    for line in batch.text.lines() {
        let Ok(event) = serde_json::from_str::<UsageEvent>(line) else {
            continue;
        };
        let Some(observation) = relation_observation(state, &event) else {
            continue;
        };
        observations = observations.saturating_add(1);
        match observation.source {
            ObservationSource::DirectCorrection => {
                direct_observations = direct_observations.saturating_add(1)
            }
            ObservationSource::PartialImeEdit => {
                partial_ime_edit_observations = partial_ime_edit_observations.saturating_add(1)
            }
            ObservationSource::CausalImeChoice => {
                ime_choice_observations = ime_choice_observations.saturating_add(1)
            }
        }
        insert_relation_observation(state, observation);
    }
    enforce_relation_bound(state);
    save_state(&paths.state, state)?;

    let ready = state.pending.iter().find_map(|(key, relation)| {
        let count = relation.scenes.len();
        (count >= MIN_SCENES && count > relation.last_attempted_scenes && count.is_power_of_two())
            .then(|| key.clone())
    });
    let Some(key) = ready else {
        if observations > 0 || batch.mode != journal::JournalReadMode::Append {
            println!(
                "{}",
                serde_json::json!({
                    "kind": "l3_online_feedback_recorded",
                    "journal_mode": format!("{:?}", batch.mode),
                    "journal_overlap_lines": batch.overlap_lines,
                    "new_observations": observations,
                    "direct_correction_observations": direct_observations,
                    "partial_ime_edit_observations": partial_ime_edit_observations,
                    "causal_ime_choice_observations": ime_choice_observations,
                    "pending_relations": state.pending.len(),
                    "recent_ime_rejections": state.recent_ime_rejections.len(),
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
        state.admitted_deltas = state.admitted_deltas.saturating_add(1);
    } else if let Some(relation) = state.pending.get_mut(&key) {
        relation.last_attempted_scenes = relation.scenes.len();
    }
    save_state(&paths.state, state)?;
    println!("{report}");
    Ok(())
}

fn replay_existing_feedback(
    paths: &Paths,
    state: &mut OnlineState,
) -> io::Result<serde_json::Value> {
    if state.replayed_source_bytes > 0 {
        return Ok(serde_json::json!({
            "kind": "l3_online_feedback_replay",
            "status": "already_completed",
            "source_bytes": state.replayed_source_bytes,
            "pending_relations": state.pending.len(),
            "runtime_authority": false,
        }));
    }
    let snapshot = journal::read_full_snapshot(&paths.usage_events)?;
    let text = snapshot.text();
    let mut parsed_events = 0_usize;
    let mut observations = 0_usize;
    let mut direct_observations = 0_usize;
    let mut partial_ime_edit_observations = 0_usize;
    let mut ime_choice_observations = 0_usize;
    for line in text.lines() {
        let Ok(event) = serde_json::from_str::<UsageEvent>(line) else {
            continue;
        };
        parsed_events = parsed_events.saturating_add(1);
        state.feedback.replayed_events = state.feedback.replayed_events.saturating_add(1);
        let Some(observation) = relation_observation(state, &event) else {
            continue;
        };
        observations = observations.saturating_add(1);
        match observation.source {
            ObservationSource::DirectCorrection => {
                direct_observations = direct_observations.saturating_add(1)
            }
            ObservationSource::PartialImeEdit => {
                partial_ime_edit_observations = partial_ime_edit_observations.saturating_add(1)
            }
            ObservationSource::CausalImeChoice => {
                ime_choice_observations = ime_choice_observations.saturating_add(1)
            }
        }
        insert_relation_observation(state, observation);
    }
    enforce_relation_bound(state);
    state.replayed_source_bytes = snapshot.complete_bytes();
    snapshot.anchor(state);
    Ok(serde_json::json!({
        "kind": "l3_online_feedback_replay",
        "status": "completed",
        "source_bytes": state.replayed_source_bytes,
        "parsed_events": parsed_events,
        "relation_observations": observations,
        "direct_correction_observations": direct_observations,
        "partial_ime_edit_observations": partial_ime_edit_observations,
        "causal_ime_choice_observations": ime_choice_observations,
        "pending_relations": state.pending.len(),
        "ready_relations": state.pending.values().filter(|relation| relation.scenes.len() >= MIN_SCENES).count(),
        "runtime_authority": false,
    }))
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

fn arg_u64(args: &[String], flag: &str) -> Option<u64> {
    args.iter()
        .position(|arg| arg == flag)
        .and_then(|index| args.get(index + 1))
        .and_then(|value| value.parse().ok())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

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
            full_proof_corpus: wave.join("proof.txt"),
            full_proof_surface: wave.join("surface.jsonl"),
        };
        fs::write(&paths.usage_events, []).unwrap();
        let mut state = OnlineState::default();
        assert!(initialize_source_cursor(&paths, &mut state, false).unwrap());
        assert_eq!(state.source_offset, 0);

        fs::write(&paths.usage_events, b"{\"kind\":\"accepted_fix\"}\n").unwrap();
        assert!(!initialize_source_cursor(&paths, &mut state, true).unwrap());
        assert_eq!(state.source_offset, 0);
        assert!(!journal::read_new_events(&paths.usage_events, &mut state)
            .unwrap()
            .text
            .is_empty());
        let _ = fs::remove_dir_all(root);
    }
}
