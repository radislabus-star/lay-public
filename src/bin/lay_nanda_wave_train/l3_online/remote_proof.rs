//! Cold execution adapter. Admission and model publication stay with the
//! existing online owner; a configured failure never runs a local full proof.
use super::Paths;
use sha2::{Digest, Sha256};
use std::{fs, io, path::Path, process::Command};

pub(super) fn run(
    runner: &Path,
    paths: &Paths,
    delta: &Path,
    targeted_receipt: &Path,
    receipt: &Path,
) -> io::Result<serde_json::Value> {
    let evaluator_sha256 = hex_digest(&fs::read("/proc/self/exe")?);
    // Remove a receipt from a prior failed request before invoking the runner.
    match fs::remove_file(receipt) {
        Ok(()) => {}
        Err(error) if error.kind() == io::ErrorKind::NotFound => {}
        Err(error) => return Err(error),
    }
    let status = Command::new(runner)
        .arg("--manifest")
        .arg(&paths.manifest)
        .arg("--delta")
        .arg(delta)
        .arg("--corpus")
        .arg(&paths.full_proof_corpus)
        .arg("--surface-evidence")
        .arg(&paths.full_proof_surface)
        .arg("--max-fragments")
        .arg("80000")
        .arg("--min-surface-support")
        .arg("2")
        .arg("--evaluator-sha256")
        .arg(&evaluator_sha256)
        .arg("--out-receipt")
        .arg(receipt)
        .status()?;
    if !status.success() {
        return Err(io::Error::other(
            "configured L3 full-proof runner failed; local proof disabled",
        ));
    }
    let bytes = fs::read(receipt)?;
    let proof: serde_json::Value = serde_json::from_slice(&bytes).map_err(io::Error::other)?;
    let targeted: serde_json::Value =
        serde_json::from_slice(&fs::read(targeted_receipt)?).map_err(io::Error::other)?;
    if proof.get("executor_sha256").and_then(|v| v.as_str()) != Some(&evaluator_sha256)
        || proof.get("delta_sha256").and_then(|v| v.as_str())
            != Some(hex_digest(&fs::read(delta)?).as_str())
        || proof.get("delta_bytes").and_then(|v| v.as_u64()) != Some(fs::metadata(delta)?.len())
        || !remote_path(&proof, "manifest", &paths.manifest)
        || !remote_path(&proof, "delta", delta)
        || ["baseline_sha256", "delta_sha256"].into_iter().any(|key| {
            proof.get(key).and_then(|v| v.as_str()).is_none() || proof.get(key) != targeted.get(key)
        })
    {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "remote L3 receipt is not bound to this evaluator and portable input snapshot",
        ));
    }
    // Full PASS and every regression counter are checked by proof_chain;
    // current baseline and proposed delta hashes are checked by admission.
    Ok(proof)
}

fn remote_path(proof: &serde_json::Value, key: &str, local: &Path) -> bool {
    proof
        .get(key)
        .and_then(|v| v.as_str())
        .is_some_and(|remote| {
            let remote = Path::new(remote);
            remote.is_absolute() && remote != local && !remote.exists()
        })
}

fn hex_digest(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::os::unix::fs::PermissionsExt;

    fn fixture() -> (
        std::path::PathBuf,
        Paths,
        std::path::PathBuf,
        std::path::PathBuf,
    ) {
        let root = std::env::temp_dir().join(format!(
            "lay-l3-remote-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir_all(&root).unwrap();
        let paths = Paths {
            root: root.clone(),
            usage_events: root.join("events"),
            base: root.join("base"),
            manifest: root.join("manifest"),
            state: root.join("state"),
            full_proof_corpus: root.join("corpus"),
            full_proof_surface: root.join("surface"),
            full_proof_runner: None,
        };
        fs::write(&paths.manifest, b"unchanged baseline").unwrap();
        let delta = root.join("delta");
        fs::write(&delta, b"candidate").unwrap();
        let targeted = root.join("targeted.json");
        fs::write(
            &targeted,
            serde_json::to_vec(&serde_json::json!({
                "baseline_sha256":"baseline", "delta_sha256":hex_digest(b"candidate")
            }))
            .unwrap(),
        )
        .unwrap();
        (root, paths, delta, targeted)
    }

    #[test]
    fn failed_configured_runner_keeps_pending_artifacts_and_never_reuses_old_receipt() {
        let (root, paths, delta, targeted) = fixture();
        let receipt = root.join("full.json");
        fs::write(&receipt, b"old PASS").unwrap();
        let error = run(Path::new("/bin/false"), &paths, &delta, &targeted, &receipt).unwrap_err();
        assert!(error.to_string().contains("local proof disabled"));
        assert!(!receipt.exists());
        assert_eq!(fs::read(&paths.manifest).unwrap(), b"unchanged baseline");
        assert_eq!(fs::read(&delta).unwrap(), b"candidate");
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn actual_runner_result_requires_matching_targeted_and_full_content() {
        let (root, paths, delta, targeted) = fixture();
        // Exercise the production subprocess/argv/receipt boundary, including
        // an executable whose filename contains spaces.
        let runner = root.join("cold runner.py");
        fs::write(
            &runner,
            r#"#!/usr/bin/env python3
import sys,json,hashlib,pathlib
a=dict(zip(sys.argv[1::2],sys.argv[2::2]));d=pathlib.Path(a['--delta']).read_bytes()
pathlib.Path(a['--out-receipt']).write_text(json.dumps({
'executor_sha256':a['--evaluator-sha256'],'delta_sha256':hashlib.sha256(d).hexdigest(),
'delta_bytes':len(d),'baseline_sha256':'baseline',
'manifest':'/remote-l3-proof-no-local-file/manifest.json',
'delta':'/remote-l3-proof-no-local-file/delta','verdict':'WATCH'}))
"#,
        )
        .unwrap();
        fs::set_permissions(&runner, fs::Permissions::from_mode(0o700)).unwrap();
        let receipt = root.join("full.json");
        let proof = run(&runner, &paths, &delta, &targeted, &receipt).unwrap();
        assert_eq!(proof["verdict"], "WATCH");
        fs::write(
            &targeted,
            br#"{"baseline_sha256":"other baseline","delta_sha256":"other delta"}"#,
        )
        .unwrap();
        assert!(run(&runner, &paths, &delta, &targeted, &receipt).is_err());
        assert_eq!(fs::read(&paths.manifest).unwrap(), b"unchanged baseline");
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn remote_receipt_cannot_use_local_paths_to_bypass_portable_identity() {
        let local = std::env::current_exe().unwrap();
        assert!(!remote_path(
            &serde_json::json!({"manifest":local}),
            "manifest",
            &local
        ));
        assert!(!remote_path(
            &serde_json::json!({"manifest":"relative.json"}),
            "manifest",
            &local
        ));
        assert!(!remote_path(
            &serde_json::json!({"manifest":"/"}),
            "manifest",
            &local
        ));
        assert!(remote_path(
            &serde_json::json!({"manifest":"/remote-l3-proof-no-local-file/manifest.json"}),
            "manifest",
            &local
        ));
    }
}
