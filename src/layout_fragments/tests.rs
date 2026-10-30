use super::*;
use std::collections::BTreeMap;

fn package(entries: &[(&[u8], i8, u8, u16)]) -> Vec<u8> {
    let mut nodes = vec![(BTreeMap::<u8, usize>::new(), u16::MAX)];
    let mut records = Vec::new();
    for &(key, weight, flags, pair) in entries {
        let record = (weight, flags, pair);
        let id = records
            .iter()
            .position(|old| *old == record)
            .unwrap_or_else(|| {
                records.push(record);
                records.len() - 1
            });
        let mut node = 0;
        for &byte in key {
            node = if let Some(&child) = nodes[node].0.get(&byte) {
                child
            } else {
                let child = nodes.len();
                nodes.push((BTreeMap::new(), u16::MAX));
                nodes[node].0.insert(byte, child);
                child
            };
        }
        nodes[node].1 = id as u16;
    }
    let mut bytes = b"LAYFRG01".to_vec();
    for value in [
        nodes.len(),
        nodes.len() - 1,
        records.len(),
        entries.iter().map(|r| r.0.len()).max().unwrap(),
    ] {
        bytes.extend_from_slice(&(value as u32).to_le_bytes());
    }
    let mut first = 0_u32;
    for (edges, record) in &nodes {
        bytes.extend_from_slice(&first.to_le_bytes());
        bytes.extend_from_slice(&(edges.len() as u16).to_le_bytes());
        bytes.extend_from_slice(&record.to_le_bytes());
        first += edges.len() as u32;
    }
    for (edges, _) in &nodes {
        for (&byte, &target) in edges {
            bytes.push(byte);
            bytes.extend_from_slice(&(target as u32).to_le_bytes());
        }
    }
    for (weight, flags, pair) in records {
        bytes.extend_from_slice(&[weight as u8, flags]);
        bytes.extend_from_slice(&pair.to_le_bytes());
    }
    bytes
}

fn score(total: i32, status: Status) -> Score {
    Score {
        total,
        status,
        matches: 0,
    }
}

#[test]
fn fragments_count_overlapping_occurrences_even_with_shared_records() {
    let bytes = package(&[(b"hap", 2, 0, 0), (b"app", 2, 0, 0)]);
    let model = Model::parse(&bytes).unwrap();
    for input in ["happy", "Happy", "HAPPY"] {
        assert_eq!(
            model.score(input, Boundary::Closed),
            Score {
                total: 4,
                status: Status::Normal,
                matches: 2
            }
        );
    }
    assert_eq!(model.score("hap", Boundary::Closed).total, 2);
}

#[test]
fn fragments_distinguish_open_closed_and_known_prefix_boundaries() {
    let bytes = package(&[(b"\x02abc", 0, 1, 0), (b"\x02go\x03", 5, 0, 0)]);
    let model = Model::parse(&bytes).unwrap();
    assert_eq!(model.score("abcdef", Boundary::Open).status, Status::Known);
    assert_eq!(model.score("xabc", Boundary::Closed).status, Status::Normal);
    assert_eq!(model.score("go", Boundary::Open).total, 0);
    assert_eq!(model.score("go", Boundary::Closed).total, 5);
    assert_eq!(model.score("going", Boundary::Closed).total, 0);
}

#[test]
fn fragments_filter_case_and_language_pair_without_double_counting() {
    let bytes = package(&[(b"ha", 2, 0, 0), (b"Ha", 3, 2, 0), (b"ap", 100, 0, 9)]);
    let model = Model::parse(&bytes).unwrap();
    assert_eq!(model.score("Happy", Boundary::Closed).total, 5);
    assert_eq!(model.score("happy", Boundary::Closed).total, 2);
    assert_eq!(model.score("HAPPY", Boundary::Closed).total, 2);
    let unavailable = package(&[(b"ha", 2, 0, 1)]);
    assert_eq!(
        Model::parse(&unavailable)
            .unwrap()
            .score("happy", Boundary::Closed)
            .status,
        Status::Unavailable
    );
}

#[test]
fn fragments_special_zero_flag_precedes_known_and_signed_sum() {
    let bytes = package(&[(b"ab", 0, 1, 0), (b"bc", 0, 0, 0), (b"cd", -4, 0, 0)]);
    let actual = Model::parse(&bytes)
        .unwrap()
        .score("abcd", Boundary::Closed);
    assert_eq!(
        actual,
        Score {
            total: -4,
            status: Status::Special,
            matches: 3
        }
    );
    assert_eq!(
        select(actual, score(100, Status::Normal)),
        Selection::Special
    );
}

#[test]
fn fragments_selector_keeps_reported_precedence_and_tie_behavior() {
    assert_eq!(
        select(score(-2, Status::Normal), score(-3, Status::Normal)),
        Selection::Russian
    );
    assert_eq!(
        select(score(0, Status::Known), score(9, Status::Normal)),
        Selection::Russian
    );
    assert_eq!(
        select(score(9, Status::Normal), score(0, Status::Known)),
        Selection::English
    );
    assert_eq!(
        select(score(0, Status::Known), score(0, Status::Known)),
        Selection::English
    );
    assert_eq!(
        select(score(2, Status::Normal), score(2, Status::Normal)),
        Selection::Russian
    );
    assert_eq!(
        select(score(0, Status::Normal), score(0, Status::Normal)),
        Selection::Ambiguous
    );
    assert_eq!(
        select(score(0, Status::Unavailable), score(9, Status::Normal)),
        Selection::Unavailable
    );
}

#[test]
fn fragments_reject_truncation_cycles_and_out_of_range_records() {
    let original = package(&[(b"ab", 2, 0, 0)]);
    for length in 0..original.len() {
        assert!(Model::parse(&original[..length]).is_none());
    }
    let mut bad = original.clone();
    bad[30..32].copy_from_slice(&42_u16.to_le_bytes());
    assert!(Model::parse(&bad).is_none());
    let mut cycle = original;
    let edge_offset = 24 + 3 * 8;
    cycle[edge_offset + 1..edge_offset + 5].copy_from_slice(&0_u32.to_le_bytes());
    assert!(Model::parse(&cycle).is_none());
}

#[test]
fn fragments_embedded_artifact_matches_manifest_and_budget() {
    use sha2::{Digest, Sha256};
    let metadata: serde_json::Value =
        serde_json::from_str(include_str!("../../data/layout_fragments/model.json")).unwrap();
    assert_eq!(
        crate::sha256_hex::encode(Sha256::digest(PACKAGE).into()),
        metadata["model_sha256"]
    );
    assert_eq!(
        model_bytes(),
        metadata["model_bytes"].as_u64().unwrap() as usize
    );
    assert!(model_bytes() <= 1024 * 1024);
    assert!(Model::parse(PACKAGE).is_some());
}

#[test]
fn fragments_project_exact_keys_both_ways_and_preserve_case() {
    for (input, expected) in [
        ("ghbdtn", "привет"),
        ("Ghbdtn", "Привет"),
        ("РУДДЩ", "HELLO"),
    ] {
        assert_eq!(
            analyze(input, Boundary::Closed)
                .unwrap()
                .replacement(input)
                .as_deref(),
            Some(expected)
        );
    }
    for input in ["привет", "hello"] {
        assert_eq!(
            analyze(input, Boundary::Closed).unwrap().replacement(input),
            None
        );
    }
    // Projection is a keyboard operation; the layout provider does not repair typos.
    let typo = analyze("ghtdtn", Boundary::Closed).unwrap();
    assert_eq!(typo.russian, "превет");
}

#[test]
fn fragments_decline_mixed_scripts_urls_and_unbounded_inputs() {
    for input in ["", "привet", "https://example.org", "123", "🙂"] {
        assert!(analyze(input, Boundary::Closed).is_none());
    }
    assert!(analyze(&"a".repeat(33), Boundary::Closed).is_none());
}

#[test]
fn fragments_config_is_optional_and_old_json_keeps_it_disabled() {
    let old: crate::config::LayConfig = serde_json::from_str("{}").unwrap();
    assert!(!old.fragment_layout_autoflip);
    let enabled: crate::config::LayConfig =
        serde_json::from_str("{\"fragment_layout_autoflip\":true}").unwrap();
    assert!(enabled.fragment_layout_autoflip);
}
