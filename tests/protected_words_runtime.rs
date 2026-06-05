use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

fn temp_home() -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time must be after unix epoch")
        .as_nanos();
    std::env::temp_dir().join(format!("lay-protected-{}-{nanos}", std::process::id()))
}

#[test]
fn user_protected_ascii_word_is_not_overridden_by_layout_scoring() {
    let home = temp_home();
    let config = home.join(".config/lay");
    fs::create_dir_all(&config).expect("create test config dir");
    fs::write(config.join("protected_words.txt"), "vs\n").expect("write protected words");

    let output = Command::new(env!("CARGO_BIN_EXE_lay"))
        .env("HOME", &home)
        .arg("--explain-correct")
        .arg("vs ")
        .output()
        .expect("run lay explain-correct");

    let _ = fs::remove_dir_all(&home);

    assert!(
        output.status.success(),
        "lay failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("output: none"),
        "protected token must stay unchanged, got:\n{stdout}"
    );
}
