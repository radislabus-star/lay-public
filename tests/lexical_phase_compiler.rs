#![cfg(feature = "lexical-compiler")]

use std::fs;
use std::path::PathBuf;
use std::process::Command;

#[test]
fn cold_compiler_writes_a_loadable_artifact_and_matching_manifest() {
    let root = unique_temp_dir();
    fs::create_dir_all(&root).expect("create compiler test directory");
    let corpus = root.join("words.txt");
    let artifact = root.join("lexical.bin");
    fs::write(&corpus, "проверка\nпроверить\nслово\n").expect("write corpus");

    let output = Command::new(env!("CARGO_BIN_EXE_lay-nanda-wave-train"))
        .arg("--compile-lexical-phase")
        .arg("--out")
        .arg(&artifact)
        .arg(&corpus)
        .output()
        .expect("run lexical phase compiler");
    assert!(
        output.status.success(),
        "compiler stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let report: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("parse compiler report");
    let manifest_path = artifact.with_extension("manifest.json");
    let manifest: serde_json::Value =
        serde_json::from_slice(&fs::read(&manifest_path).expect("read compiler manifest"))
            .expect("parse compiler manifest");

    assert_eq!(report["output"], artifact.display().to_string());
    assert_eq!(report["manifest"], manifest_path.display().to_string());
    assert_eq!(manifest["artifact_checksum"], report["artifact_checksum"]);
    assert_eq!(manifest["corpus_hash"], report["corpus_hash"]);
    assert_eq!(manifest["raw_word_table"], false);
    assert!(artifact.metadata().expect("artifact metadata").len() > 128);

    fs::remove_dir_all(root).expect("remove compiler test directory");
}

fn unique_temp_dir() -> PathBuf {
    std::env::temp_dir().join(format!(
        "lay-lexical-phase-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("system clock")
            .as_nanos()
    ))
}
