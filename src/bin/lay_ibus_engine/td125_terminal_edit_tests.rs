//! Actual executor payload -> GNU Readline. Not a legacy IBus delivery proof.
use std::io::Write;
use std::process::{Command, Stdio};
use std::sync::{Arc, Mutex};

use lay::config::LayConfig;
use lay::text_edit::{VisibleTailSnapshot, VisibleTailSource};

use crate::engine::LayIbusEngine;
use crate::output::{AtomicEffectBuilder, AtomicProposal, EngineOutput, PROPOSAL_FRAME_READY};
use crate::state::CommittedTailReplaceRequest;

fn terminal(initial: &str) -> LayIbusEngine {
    let mut engine = LayIbusEngine::new(
        "/td125/terminal".to_string(),
        Arc::new(Mutex::new(Default::default())),
        true,
        true,
        LayConfig {
            text_backend: "ime".to_string(),
            ..LayConfig::default()
        },
    );
    engine.set_client_capabilities(0);
    engine.set_content_type_state(10, 0);
    engine.client_context.cursor_cell_width = 11;
    assert!(engine.bind_focus_path());
    for ch in initial.chars() {
        engine.push_tail_char(ch);
    }
    engine
}

fn replace(
    engine: &mut LayIbusEngine,
    request: CommittedTailReplaceRequest,
) -> (bool, AtomicProposal) {
    let mut builder = AtomicEffectBuilder::default();
    let handled = zbus::block_on(
        engine.replace_committed_tail(&mut EngineOutput::atomic(&mut builder), request),
    )
    .expect("committed-tail executor");
    (handled, builder.finish(handled))
}

fn commit_payload(result: (bool, AtomicProposal)) -> String {
    let (handled, proposal) = result;
    assert!(handled);
    assert_eq!(proposal.0, PROPOSAL_FRAME_READY);
    assert!(proposal.1.iter().all(|(tag, _)| *tag != 2));
    let commits: Vec<_> = proposal
        .1
        .into_iter()
        .filter(|(tag, _)| *tag == 1)
        .collect();
    assert_eq!(commits.len(), 1, "one terminal mutation frame");
    String::try_from(commits.into_iter().next().unwrap().1).expect("commit string")
}

fn readline(initial: &str, payload: &str) -> String {
    let mut child = Command::new("/usr/bin/python3")
        .arg(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/scripts/proof/ime-client/readline_consumer.py"
        ))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("real Readline consumer");
    let input = serde_json::json!({"initial": initial, "payload": payload, "marker": "§"});
    child
        .stdin
        .take()
        .unwrap()
        .write_all(input.to_string().as_bytes())
        .expect("consumer request");
    let result = child
        .wait_with_output()
        .expect("bounded consumer completion");
    assert!(
        result.status.success(),
        "Readline failed: {}",
        String::from_utf8_lossy(&result.stderr)
    );
    let result: serde_json::Value = serde_json::from_slice(&result.stdout).unwrap();
    result["line"]
        .as_str()
        .expect("actual line buffer")
        .to_string()
}

#[test]
fn td125_completion_executes_authorized_append_without_erasing_prefix() {
    let mut engine = terminal("метка пров");
    let payload = commit_payload(replace(
        &mut engine,
        CommittedTailReplaceRequest::ime_candidate_accept(4, "проверка ".to_string()),
    ));
    assert_eq!(payload, "ерка ", "execute the append-only authorized plan");
    assert_eq!(engine.committed_tail.buffer, "метка проверка ");
}

#[test]
fn td125_completion_preserves_existing_no_backend_refusal() {
    let mut engine = terminal("метка пров");
    engine.client_context.cursor_cell_width = 0;
    let (handled, proposal) = replace(
        &mut engine,
        CommittedTailReplaceRequest::ime_candidate_accept(4, "проверка ".to_string()),
    );
    assert!(!handled);
    assert!(proposal.1.is_empty());
    assert_eq!(engine.committed_tail.buffer, "метка пров");
}

#[test]
fn td125_readline_completion_then_full_edit_preserves_left_word_and_separator() {
    let initial = "метка пров";
    let mut engine = terminal(initial);
    let first = commit_payload(replace(
        &mut engine,
        CommittedTailReplaceRequest::ime_candidate_accept(4, "проверка".to_string()),
    ));
    assert_eq!(engine.committed_tail.buffer, "метка проверка");
    // Controlled executor sequence, not a claim that Space bypasses the
    // current-word suppression armed by explicit candidate acceptance.
    let second = commit_payload(replace(
        &mut engine,
        CommittedTailReplaceRequest::ime_autocorrect(8, "проверки ".to_string()),
    ));
    assert_eq!(engine.committed_tail.buffer, "метка проверки ");
    assert_eq!(readline(initial, &(first + &second)), "метка проверки §");
}

#[test]
fn td125_readline_full_token_growth_equal_shrink_preserve_left_boundary() {
    for prefix in ["left ", "left  ", "метка "] {
        for (old, new) in [("лово", "слово"), ("слова", "слово"), ("сллово", "слово")]
        {
            let initial = format!("{prefix}{old}");
            let mut engine = terminal(&initial);
            let payload = commit_payload(replace(
                &mut engine,
                CommittedTailReplaceRequest::ime_autocorrect(
                    old.chars().count() as u32,
                    format!("{new} "),
                ),
            ));
            assert_eq!(payload, "\u{7f}".repeat(old.chars().count()) + new + " ");
            assert_eq!(engine.committed_tail.buffer, format!("{prefix}{new} "));
            assert_eq!(readline(&initial, &payload), format!("{prefix}{new} §"));
        }
    }
}

#[test]
fn td125_readline_oracle_detects_intentional_overdelete() {
    let payload = "\u{7f}".repeat(5) + "слово ";
    assert_eq!(readline("метка лово", &payload), "меткаслово §");
}

#[test]
fn td125_stale_completion_refuses_without_output_or_tail_change() {
    let mut engine = terminal("метка пров");
    let expected = VisibleTailSnapshot::new(
        VisibleTailSource::ImeCommittedTail,
        "пров".to_string(),
        Some(engine.path.clone()),
        engine.committed_tail.epoch + 1,
    );
    let (handled, proposal) = replace(
        &mut engine,
        CommittedTailReplaceRequest::ime_candidate_accept(4, "проверка ".to_string())
            .with_expected_tail(expected),
    );
    assert!(!handled);
    assert!(proposal.1.is_empty());
    assert_eq!(engine.committed_tail.buffer, "метка пров");
}
