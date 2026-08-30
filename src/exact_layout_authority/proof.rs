//! Runtime side of the V27 independent-oracle proof.
//!
//! This module is test-only. Expected rows are produced by the separately
//! compiled standalone oracle under `scripts/proof/`; this code only observes
//! the production authority route and writes actual rows for byte comparison.

use std::env;
use std::fs::{self, File};
use std::io::{BufWriter, Write};
use std::path::Path;
use std::thread;
use std::time::Duration;

use rayon::prelude::*;

use super::{
    exact_authority_snapshot_if_warm, warm_up_exact_layout_authority_for_ibus, ActiveDecoderLayout,
    ExactAuthoritySnapshot, ExactLayoutFrame, FactoryEngineProfile,
};
use crate::config::LayConfig;
use crate::ime_correction::{
    decide_active_composition_autocorrect_observed_with_exact,
    prepare_exact_layout_active_composition_autocorrect_observed,
    ActiveCompositionAutocorrectRequest,
};

const ROW_SCHEMA: &str = "lay-v27-exact-layout-oracle-rows-v1";
// V28 adds the reverse keyboard map and Cyrillic protection set to the same
// immutable authority owner. Keep a fixed 1 MiB envelope for that admitted
// direction instead of treating a few pages of deterministic data as drift.
const ENGINE_EXACT_GUARD_RSS_BUDGET_KIB: u64 = 15 * 1_024;
const ACTIVE_LAY_EXACT_GUARD_PSS_BUDGET_KIB: u64 = 16 * 1_024;

#[derive(Clone, Debug)]
struct OracleRow {
    class_id: String,
    operation: String,
    input: String,
    target: String,
    expected: String,
    profile: String,
    decoder: String,
    active_composition: bool,
    auto_replace: bool,
    auto_switch: bool,
    context: String,
    snapshot: String,
}

impl OracleRow {
    fn parse(line: &str) -> Result<Self, String> {
        let fields = line.split('\t').collect::<Vec<_>>();
        if fields.len() != 12 {
            return Err(format!(
                "oracle row has {} fields instead of 12: {line:?}",
                fields.len()
            ));
        }
        Ok(Self {
            class_id: fields[0].to_string(),
            operation: fields[1].to_string(),
            input: fields[2].to_string(),
            target: fields[3].to_string(),
            expected: fields[4].to_string(),
            profile: fields[5].to_string(),
            decoder: fields[6].to_string(),
            active_composition: parse_bool(fields[7])?,
            auto_replace: parse_bool(fields[8])?,
            auto_switch: parse_bool(fields[9])?,
            context: fields[10].to_string(),
            snapshot: fields[11].to_string(),
        })
    }

    fn actual_line(&self, actual_target: &str, actual: &str) -> String {
        format!(
            "{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\n",
            self.class_id,
            self.operation,
            self.input,
            actual_target,
            actual,
            self.profile,
            self.decoder,
            u8::from(self.active_composition),
            u8::from(self.auto_replace),
            u8::from(self.auto_switch),
            self.context,
            self.snapshot,
        )
    }
}

struct ObservedRow {
    line: String,
    error: Option<String>,
}

#[test]
#[ignore = "requires independently generated V27 oracle rows"]
fn v27_independent_oracle_runtime_parity() {
    let rows_path = env::var("LAY_V27_ORACLE_ROWS").expect("LAY_V27_ORACLE_ROWS is required");
    let actual_path =
        env::var("LAY_V27_RUNTIME_ROWS_OUT").expect("LAY_V27_RUNTIME_ROWS_OUT is required");
    let scope = env::var("LAY_V27_RUNTIME_SCOPE").unwrap_or_else(|_| "certificate".to_string());
    assert!(matches!(scope.as_str(), "certificate" | "full"));

    let rows = read_rows(Path::new(&rows_path)).expect("parse independent oracle rows");
    assert!(!rows.is_empty(), "oracle denominator must be nonempty");
    warm_up_exact_layout_authority_for_ibus().expect("warm production exact authority");

    let observed = rows
        .par_iter()
        .map(|row| observe_row(row, &scope))
        .collect::<Vec<_>>();
    write_actual_rows(Path::new(&actual_path), &observed).expect("write runtime actual rows");

    let errors = observed
        .iter()
        .filter_map(|row| row.error.as_deref())
        .take(40)
        .collect::<Vec<_>>();
    let divergence_count = observed.iter().filter(|row| row.error.is_some()).count();
    eprintln!(
        "V27_RUNTIME_ORACLE scope={} rows={} divergences={}",
        scope,
        observed.len(),
        divergence_count
    );
    assert_eq!(
        divergence_count, 0,
        "runtime/oracle divergences={} first={errors:#?}",
        divergence_count
    );
}

#[test]
fn v27_exact_en_guard_controlled_resource_budget() {
    // Match the engine's already-admitted baseline: these compact/shared
    // dependencies are not allocations introduced by the exact EN guard.
    let _ = crate::dict::warm_up_us_to_ru();
    crate::nanda_wave::warm_up_exact_layout_terminal_authority()
        .expect("warm exact terminal authority baseline");
    let before = settled_process_memory();

    let receipt =
        warm_up_exact_layout_authority_for_ibus().expect("warm production exact authority");
    let after = settled_process_memory();
    let repeat_receipt =
        warm_up_exact_layout_authority_for_ibus().expect("reuse production exact authority");
    let repeated = settled_process_memory();

    let rss_delta_kib = after.rss_kib.saturating_sub(before.rss_kib);
    let pss_delta_kib = after.pss_kib.saturating_sub(before.pss_kib);
    let pss_anon_delta_kib = after.pss_anon_kib.saturating_sub(before.pss_anon_kib);
    let repeat_pss_anon_delta_kib = repeated.pss_anon_kib.saturating_sub(after.pss_anon_kib);

    eprintln!(
        "LAY_V27_EXACT_RESOURCE english_entries={} protection_entries={} resident_bytes={} authority_fingerprint={} baseline_rss_kib={} final_rss_kib={} rss_delta_kib={} baseline_pss_kib={} final_pss_kib={} pss_delta_kib={} baseline_pss_anon_kib={} final_pss_anon_kib={} pss_anon_delta_kib={} repeat_pss_anon_delta_kib={}",
        receipt.english_entries,
        receipt.protection_entries,
        receipt.resident_bytes,
        receipt.authority_fingerprint,
        before.rss_kib,
        after.rss_kib,
        rss_delta_kib,
        before.pss_kib,
        after.pss_kib,
        pss_delta_kib,
        before.pss_anon_kib,
        after.pss_anon_kib,
        pss_anon_delta_kib,
        repeat_pss_anon_delta_kib,
    );

    assert!(receipt.english_entries > 0);
    assert!(receipt.protection_entries > 0);
    assert_ne!(receipt.authority_fingerprint, 0);
    assert_eq!(repeat_receipt, receipt, "warm snapshot must be immutable");
    assert!(
        rss_delta_kib <= ENGINE_EXACT_GUARD_RSS_BUDGET_KIB,
        "exact EN guard RSS delta {rss_delta_kib} KiB exceeds {ENGINE_EXACT_GUARD_RSS_BUDGET_KIB} KiB"
    );
    assert!(
        pss_anon_delta_kib <= ENGINE_EXACT_GUARD_RSS_BUDGET_KIB,
        "exact EN guard Pss_Anon delta {pss_anon_delta_kib} KiB exceeds {ENGINE_EXACT_GUARD_RSS_BUDGET_KIB} KiB"
    );
    assert!(
        pss_delta_kib <= ACTIVE_LAY_EXACT_GUARD_PSS_BUDGET_KIB,
        "single-owner aggregate PSS delta {pss_delta_kib} KiB exceeds {ACTIVE_LAY_EXACT_GUARD_PSS_BUDGET_KIB} KiB"
    );
    assert!(
        repeat_pss_anon_delta_kib <= 256,
        "second warmup retained {repeat_pss_anon_delta_kib} KiB; snapshot is not allocation-stable"
    );
}

#[derive(Clone, Copy)]
struct ProcessMemorySample {
    rss_kib: u64,
    pss_kib: u64,
    pss_anon_kib: u64,
}

fn settled_process_memory() -> ProcessMemorySample {
    thread::sleep(Duration::from_millis(100));
    process_memory_sample()
}

fn process_memory_sample() -> ProcessMemorySample {
    let status = fs::read_to_string("/proc/self/status").expect("read process status");
    let rollup = fs::read_to_string("/proc/self/smaps_rollup").expect("read process smaps rollup");
    ProcessMemorySample {
        rss_kib: proc_kib(&status, "VmRSS").expect("VmRSS in process status"),
        pss_kib: proc_kib(&rollup, "Pss").expect("Pss in process smaps rollup"),
        pss_anon_kib: proc_kib(&rollup, "Pss_Anon").expect("Pss_Anon in process smaps rollup"),
    }
}

fn proc_kib(text: &str, key: &str) -> Option<u64> {
    let prefix = format!("{key}:");
    text.lines().find_map(|line| {
        line.trim_start()
            .strip_prefix(&prefix)?
            .split_whitespace()
            .next()?
            .parse()
            .ok()
    })
}

fn observe_row(row: &OracleRow, scope: &str) -> ObservedRow {
    if row.operation == "guard" {
        let guarded = crate::word_recognizer::exact_english_word_if_warm(&row.input);
        let actual = match guarded {
            Some(true) => "guarded",
            Some(false) => "unguarded",
            None => "unavailable",
        };
        let error = (actual != row.expected).then(|| {
            format!(
                "class={} input={:?} expected={} actual={}",
                row.class_id, row.input, row.expected, actual
            )
        });
        return ObservedRow {
            line: row.actual_line("", actual),
            error,
        };
    }

    let profile = parse_profile(&row.profile);
    let decoder = parse_decoder(&row.decoder);
    let snapshot = snapshot_for(row, profile, decoder);
    let frame = ExactLayoutFrame {
        frame_revision: 27,
        frame_fingerprint: 0x27_00_00_01,
        observed_token: row.input.clone(),
        active_composition: row.active_composition,
        factory_engine_profile: profile,
        active_decoder_layout: decoder,
        authority_snapshot: snapshot,
    };
    let config = LayConfig {
        text_backend: "ime".to_string(),
        auto_replace: row.auto_replace,
        typing_assist: true,
        auto_switch_layout: row.auto_switch,
        correction_safety: "experimental".to_string(),
        nanda_autocorrect: true,
        nanda_precognition: true,
        nanda_l2_phase_apply: false,
        ..LayConfig::default()
    };
    let text = format!("{} ", row.input);
    let committed_tail = committed_tail(row);
    let request = || ActiveCompositionAutocorrectRequest {
        text: &text,
        committed_tail: &committed_tail,
        config: &config,
        lexical_authority_frame: None,
        active_layout_is_ru: Some(decoder == ActiveDecoderLayout::Ru),
    };
    let prepared =
        prepare_exact_layout_active_composition_autocorrect_observed(request(), &frame).prepared;
    let actual_target = prepared
        .as_ref()
        .map(|prepared| prepared.certificate.projected_token().to_string())
        .unwrap_or_else(|| row.target.clone());
    let actual = if prepared.is_some() {
        "apply"
    } else {
        "no_apply"
    };
    let mut errors = Vec::new();
    if actual != row.expected {
        errors.push(format!(
            "disposition expected={} actual={actual}",
            row.expected
        ));
    }
    if prepared.is_some() && actual_target != row.target {
        errors.push(format!(
            "target expected={:?} actual={actual_target:?}",
            row.target
        ));
    }

    if let Some(prepared) = prepared {
        if let Some(decision) = prepared.decision.as_ref() {
            if !decision.action.allow_apply() {
                errors.push("closed exact decision did not authorize apply".to_string());
            }
            let expected_live = format!("{} ", row.target);
            if decision.replacement != expected_live {
                errors.push(format!(
                    "closed replacement expected={expected_live:?} actual={:?}",
                    decision.replacement
                ));
            }
        } else {
            errors.push("certificate had no closed exact decision".to_string());
        }

        if scope == "full" && row.expected == "apply" {
            let full = decide_active_composition_autocorrect_observed_with_exact(
                request(),
                &prepared.certificate,
            )
            .decision;
            match (prepared.decision.as_ref(), full.as_ref()) {
                (Some(exact), Some(full)) => {
                    if exact.replacement != full.replacement {
                        errors.push(format!(
                            "full/exact replacement differs exact={:?} full={:?}",
                            exact.replacement, full.replacement
                        ));
                    }
                    if exact.action.allow_apply() != full.action.allow_apply() {
                        errors.push("full/exact allow_apply differs".to_string());
                    }
                    if exact.action.transition().proof() != full.action.transition().proof() {
                        errors.push("full/exact transition proof differs".to_string());
                    }
                }
                _ => errors.push("full/exact terminal disposition differs".to_string()),
            }
        }
    }

    ObservedRow {
        line: row.actual_line(&actual_target, actual),
        error: (!errors.is_empty()).then(|| {
            format!(
                "class={} input={:?} target={:?}: {}",
                row.class_id,
                row.input,
                row.target,
                errors.join("; ")
            )
        }),
    }
}

fn snapshot_for(
    row: &OracleRow,
    profile: FactoryEngineProfile,
    decoder: ActiveDecoderLayout,
) -> Option<ExactAuthoritySnapshot> {
    let mut snapshot = exact_authority_snapshot_if_warm(profile, decoder);
    match row.snapshot.as_str() {
        "current" => snapshot,
        "none" => None,
        "corrupt_keyboard" => {
            if let Some(snapshot) = snapshot.as_mut() {
                snapshot.keyboard_map_fingerprint ^= 1;
            }
            snapshot
        }
        value => panic!("unknown oracle snapshot mode {value:?}"),
    }
}

fn committed_tail(row: &OracleRow) -> String {
    match row.context.as_str() {
        "first" => row.input.clone(),
        "ru" => format!("проверь {}", row.input),
        "ascii" => format!("check {}", row.input),
        "punct" => format!("check: {}", row.input),
        value => panic!("unknown oracle context {value:?}"),
    }
}

fn parse_profile(value: &str) -> FactoryEngineProfile {
    match value {
        "us_qwerty" => FactoryEngineProfile::UsQwerty,
        "ru" => FactoryEngineProfile::Ru,
        "unknown" => FactoryEngineProfile::Unknown,
        _ => panic!("unknown oracle profile {value:?}"),
    }
}

fn parse_decoder(value: &str) -> ActiveDecoderLayout {
    match value {
        "us" => ActiveDecoderLayout::Us,
        "ru" => ActiveDecoderLayout::Ru,
        _ => panic!("unknown oracle decoder {value:?}"),
    }
}

fn parse_bool(value: &str) -> Result<bool, String> {
    match value {
        "0" => Ok(false),
        "1" => Ok(true),
        _ => Err(format!("invalid boolean field {value:?}")),
    }
}

fn read_rows(path: &Path) -> Result<Vec<OracleRow>, String> {
    let text = fs::read_to_string(path)
        .map_err(|error| format!("cannot read {}: {error}", path.display()))?;
    let mut lines = text.lines();
    if lines.next() != Some(&format!("#{ROW_SCHEMA}")) {
        return Err("oracle row schema mismatch".to_string());
    }
    let header = lines.next().ok_or("oracle row header is missing")?;
    if !header.starts_with("#class\toperation\tinput\ttarget\texpected") {
        return Err("oracle row column header mismatch".to_string());
    }
    lines
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .map(OracleRow::parse)
        .collect()
}

fn write_actual_rows(path: &Path, rows: &[ObservedRow]) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|error| format!("cannot create {}: {error}", parent.display()))?;
    }
    let file =
        File::create(path).map_err(|error| format!("cannot create {}: {error}", path.display()))?;
    let mut writer = BufWriter::new(file);
    writeln!(writer, "#{ROW_SCHEMA}").map_err(|error| error.to_string())?;
    writeln!(
        writer,
        "#class\toperation\tinput\ttarget\texpected\tprofile\tdecoder\tactive_composition\tauto_replace\tauto_switch\tcontext\tsnapshot"
    )
    .map_err(|error| error.to_string())?;
    for row in rows {
        writer
            .write_all(row.line.as_bytes())
            .map_err(|error| error.to_string())?;
    }
    writer.flush().map_err(|error| error.to_string())
}
