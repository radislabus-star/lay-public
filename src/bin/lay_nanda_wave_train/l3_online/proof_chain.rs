use super::feedback::PendingRelation;
use super::Paths;
use std::fs;
use std::io;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

const FULL_PROOF_MAX_FRAGMENTS: usize = 80_000;
const FULL_PROOF_MIN_SURFACE_SUPPORT: u32 = 2;

pub(super) fn attempt_relation(
    paths: &Paths,
    generation: u64,
    relation: &PendingRelation,
) -> io::Result<serde_json::Value> {
    let stem = format!("delta-{generation:08}-{}", unix_time());
    let corpus = paths.root.join(format!("{stem}.txt"));
    let cases = paths.root.join(format!("{stem}.cases.tsv"));
    let delta = paths.root.join(format!("{stem}.nwpc"));
    let targeted_receipt = paths.root.join(format!("{stem}.targeted-proof.json"));
    let full_receipt = paths.root.join(format!("{stem}.full-proof.json"));
    // Two distinct scenes establish independence; replaying each scene once
    // more gives the learner separate profile and coherent-center observations.
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
        false,
    )?;
    let proof = lay::nanda_wave::prove_l3_context_delta_targeted(
        &paths.manifest,
        &delta,
        &cases,
        &targeted_receipt,
    )?;
    let targeted_passed = proof_passed(&proof);
    let full_proof = if targeted_passed {
        Some(prove_full_differential(
            paths,
            &stem,
            &delta,
            &targeted_receipt,
            &full_receipt,
        )?)
    } else {
        None
    };
    let full_passed = full_proof.as_ref().is_some_and(full_proof_passed);
    let (admission, compaction) = if targeted_passed && full_passed {
        let admission = lay::nanda_wave::admit_l3_context_delta_with_full_proof(
            &paths.manifest,
            &delta,
            &targeted_receipt,
            &full_receipt,
            Some("local-online"),
        )?;
        // A manifest containing deltas is a cold learner representation: its
        // shard merge materializes every compact phase center. Never leave that
        // representation for live daemon/IME reload. Fold the admitted evidence
        // once in this background worker and publish a delta-free compact base.
        let compact_base = inactive_compact_base(paths)?;
        let compaction =
            lay::nanda_wave::compact_l3_context_composite(&paths.manifest, &compact_base)?;
        (Some(admission), Some(compaction))
    } else {
        (None, None)
    };
    let verdict = if targeted_passed && full_passed {
        "PASS"
    } else {
        "WATCH"
    };
    let base_rewritten = compaction.is_some();
    Ok(serde_json::json!({
        "kind": "l3_online_delta_attempt",
        "generation": generation,
        "rejected": relation.rejected,
        "expected": relation.expected,
        "independent_episodes": relation.independent_episodes(),
        "distinct_scenes": relation.distinct_scenes(),
        "independent_scenes": relation.distinct_scenes(),
        "selected_relations": 1,
        "selector": "minimal_single_relation_then_targeted_and_full_impact_proof",
        "corpus_passes": compile.get("corpus_passes"),
        "delta": delta,
        "targeted_proof_receipt": targeted_receipt,
        "full_proof_receipt": full_proof.as_ref().map(|_| &full_receipt),
        "false_supports": proof.get("false_supports"),
        "targeted_verdict": proof.get("verdict"),
        "full_differential_verdict": full_proof.as_ref().and_then(|value| value.get("verdict")),
        "lost_target_profiles": full_proof.as_ref().and_then(|value| value.get("lost_target_profiles")),
        "lost_supports": full_proof.as_ref().and_then(|value| value.get("lost_supports")),
        "lost_top1": full_proof.as_ref().and_then(|value| value.get("lost_top1")),
        "new_false_supports": full_proof.as_ref().and_then(|value| value.get("new_false_supports")),
        "new_false_top1": full_proof.as_ref().and_then(|value| value.get("new_false_top1")),
        "verdict": verdict,
        "admission": admission,
        "compaction": compaction,
        "base_rewritten": base_rewritten,
    }))
}

fn inactive_compact_base(paths: &Paths) -> io::Result<std::path::PathBuf> {
    let manifest: serde_json::Value =
        serde_json::from_slice(&fs::read(&paths.manifest)?).map_err(io::Error::other)?;
    let current = manifest
        .get("base")
        .and_then(serde_json::Value::as_str)
        .unwrap_or_default();
    let name = if current.ends_with("compact-base-a.nwpc") {
        "compact-base-b.nwpc"
    } else {
        "compact-base-a.nwpc"
    };
    Ok(paths.root.join(name))
}

fn prove_full_differential(
    paths: &Paths,
    _stem: &str,
    delta: &Path,
    _targeted_receipt: &Path,
    full_receipt: &Path,
) -> io::Result<serde_json::Value> {
    if !paths.full_proof_corpus.is_file() || !paths.full_proof_surface.is_file() {
        return persist_bound_full_proof(
            serde_json::json!({
                "kind": "l3_context_phase_full_differential_proof",
                "verdict": "WATCH",
                "reason": "frozen_full_proof_sources_unavailable",
                "corpus": paths.full_proof_corpus,
                "surface_evidence": paths.full_proof_surface,
                "runtime_authority": false,
            }),
            &paths.manifest,
            delta,
            full_receipt,
        );
    }
    lay::nanda_wave::prove_l3_context_composite_delta_full(
        &paths.full_proof_corpus,
        &paths.manifest,
        delta,
        &paths.full_proof_surface,
        FULL_PROOF_MAX_FRAGMENTS,
        FULL_PROOF_MIN_SURFACE_SUPPORT,
        full_receipt,
    )
}

fn persist_bound_full_proof(
    mut proof: serde_json::Value,
    manifest: &Path,
    delta: &Path,
    receipt: &Path,
) -> io::Result<serde_json::Value> {
    if let Some(object) = proof.as_object_mut() {
        object.insert("manifest".to_string(), serde_json::json!(manifest));
        object.insert("delta".to_string(), serde_json::json!(delta));
        object.insert(
            "delta_bytes".to_string(),
            serde_json::json!(fs::metadata(delta)?.len()),
        );
    }
    let mut bytes = serde_json::to_vec_pretty(&proof).map_err(io::Error::other)?;
    bytes.push(b'\n');
    lay::private_file::write_private_bytes(receipt, &bytes)?;
    Ok(proof)
}

fn proof_passed(proof: &serde_json::Value) -> bool {
    proof.get("verdict").and_then(serde_json::Value::as_str) == Some("PASS")
        && proof
            .get("false_supports")
            .and_then(serde_json::Value::as_u64)
            == Some(0)
}

fn full_proof_passed(proof: &serde_json::Value) -> bool {
    proof.get("kind").and_then(serde_json::Value::as_str)
        == Some("l3_context_phase_full_differential_proof")
        && proof.get("verdict").and_then(serde_json::Value::as_str) == Some("PASS")
        && [
            "lost_target_profiles",
            "lost_supports",
            "lost_top1",
            "new_false_supports",
            "new_false_top1",
        ]
        .into_iter()
        .all(|field| proof.get(field).and_then(serde_json::Value::as_u64) == Some(0))
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

fn write_private(path: &Path, text: &str) -> io::Result<()> {
    lay::private_file::write_private_text(path, text)
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

    #[test]
    fn targeted_proof_has_improvements_and_sentinels() {
        let relation = PendingRelation {
            rejected: "ход".to_string(),
            expected: "ходу".to_string(),
            scenes: vec![
                "обновлять модель по ходу".to_string(),
                "менять параметры по ходу".to_string(),
            ],
            episode_ids: vec!["episode-1".to_string(), "episode-2".to_string()],
            last_attempted_episodes: 0,
            last_observed_ordinal: 2,
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
    fn watch_or_regression_never_opens_admission() {
        assert!(!proof_passed(
            &serde_json::json!({"verdict": "WATCH", "false_supports": 0})
        ));
        assert!(!proof_passed(
            &serde_json::json!({"verdict": "PASS", "false_supports": 1})
        ));
        assert!(proof_passed(
            &serde_json::json!({"verdict": "PASS", "false_supports": 0})
        ));
        assert!(!full_proof_passed(&serde_json::json!({
            "kind": "l3_context_phase_full_differential_proof",
            "verdict": "PASS",
            "lost_target_profiles": 0,
            "lost_supports": 1,
            "lost_top1": 0,
            "new_false_supports": 0,
            "new_false_top1": 0
        })));
        assert!(full_proof_passed(&serde_json::json!({
            "kind": "l3_context_phase_full_differential_proof",
            "verdict": "PASS",
            "lost_target_profiles": 0,
            "lost_supports": 0,
            "lost_top1": 0,
            "new_false_supports": 0,
            "new_false_top1": 0
        })));
    }

    #[test]
    fn compaction_alternates_between_two_inactive_base_slots() {
        let root = std::env::temp_dir().join(format!(
            "lay-l3-compact-slot-{}",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let online = root.join("l3-online");
        fs::create_dir_all(&online).unwrap();
        let manifest = root.join("runtime.json");
        fs::write(
            &manifest,
            br#"{"format":"lay-l3-composite-v1","base":"l3-online/compact-base-a.nwpc","deltas":[]}"#,
        )
        .unwrap();
        let paths = Paths {
            root: online.clone(),
            usage_events: root.join("events.jsonl"),
            corrections: root.join("corrections.jsonl"),
            base: root.join("base.nwpc"),
            manifest: manifest.clone(),
            state: root.join("state.json"),
            full_proof_corpus: root.join("proof.txt"),
            full_proof_surface: root.join("surface.jsonl"),
        };
        assert_eq!(
            inactive_compact_base(&paths).unwrap(),
            online.join("compact-base-b.nwpc")
        );
        fs::write(
            &manifest,
            br#"{"format":"lay-l3-composite-v1","base":"l3-online/compact-base-b.nwpc","deltas":[]}"#,
        )
        .unwrap();
        assert_eq!(
            inactive_compact_base(&paths).unwrap(),
            online.join("compact-base-a.nwpc")
        );
        let _ = fs::remove_dir_all(root);
    }
}
