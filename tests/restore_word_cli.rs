use std::io::Write;
use std::process::Command;

fn restore(word: &str) -> String {
    let output = Command::new(env!("CARGO_BIN_EXE_lay"))
        .arg("--restore-word")
        .arg(word)
        .output()
        .expect("run lay --restore-word");

    assert!(
        output.status.success(),
        "lay failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout).expect("UTF-8 output")
}

#[test]
fn restore_word_keeps_unproven_typos_without_a_lexical_authority_frame() {
    assert_eq!(restore("врмея"), "врмея");
    assert_eq!(restore("рабоатет"), "рабоатет");
}

#[test]
fn restore_word_keeps_an_already_valid_word() {
    assert_eq!(restore("короче"), "короче");
}

#[test]
fn restore_word_rejects_phrase_input() {
    let output = Command::new(env!("CARGO_BIN_EXE_lay"))
        .arg("--restore-word")
        .arg("два слова")
        .output()
        .expect("run lay --restore-word with a phrase");

    assert_eq!(output.status.code(), Some(2));
}

#[test]
fn restore_word_stream_reuses_one_loaded_fail_closed_core() {
    let mut child = Command::new(env!("CARGO_BIN_EXE_lay"))
        .arg("--restore-word")
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .spawn()
        .expect("spawn lay --restore-word");
    child
        .stdin
        .as_mut()
        .expect("stdin")
        .write_all("врмея\nрабоатет\nкороче\n".as_bytes())
        .expect("write word stream");
    let output = child.wait_with_output().expect("collect word stream");

    assert!(output.status.success());
    assert_eq!(
        String::from_utf8(output.stdout).expect("UTF-8 output"),
        "врмея\nрабоатет\nкороче"
    );
}
