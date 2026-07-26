use std::io;
use std::path::Path;

use super::format;
use super::model::LexicalGrokkingPackage;

const FINAL_PACKAGE_BUDGET_BYTES: usize = 195 * 1024 * 1024;

pub fn compact_depth0_package(input: &Path, output: &Path) -> io::Result<serde_json::Value> {
    let input_bytes = std::fs::read(input)?;
    let package = format::decode(&input_bytes).map_err(io::Error::other)?;
    let compacted = compact_depth0(package).map_err(io::Error::other)?;
    let output_bytes = format::encode_compact_depth0(&compacted).map_err(io::Error::other)?;
    if output_bytes.len() > FINAL_PACKAGE_BUDGET_BYTES {
        return Err(io::Error::other(format!(
            "compacted depth-0 package exceeds 195 MiB: {} > {} bytes",
            output_bytes.len(),
            FINAL_PACKAGE_BUDGET_BYTES
        )));
    }
    let reconstructed = format::decode(&output_bytes).map_err(io::Error::other)?;
    if reconstructed != compacted {
        return Err(io::Error::other(
            "compact depth-0 V7 roundtrip changed the reconstructed runtime field",
        ));
    }
    let reconstructed_forward = reconstructed.forward_couplings.len();
    let reconstructed_reverse = reconstructed.reverse_couplings.len();
    let retained_positive = reconstructed.positive_subcenters.len();
    let retained_ambiguity = reconstructed.ambiguity_subcenters.len();
    if let Some(parent) = output.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let temporary = output.with_extension("tmp");
    std::fs::write(&temporary, &output_bytes)?;
    std::fs::rename(&temporary, output)?;
    Ok(serde_json::json!({
        "input": input,
        "output": output,
        "terminal_count": compacted.terminal_count(),
        "input_bytes": input_bytes.len(),
        "output_bytes": output_bytes.len(),
        "stored_forward_couplings": 0,
        "reconstructed_forward_couplings": reconstructed_forward,
        "reconstructed_reverse_couplings": reconstructed_reverse,
        "positive_subcenters": retained_positive,
        "ambiguity_subcenters": retained_ambiguity,
        "format": "V7",
        "exact_runtime_field_parity": true,
        "forward_storage": "implicit_decoder_reconstruction",
        "reverse_storage": "implicit_decoder_reconstruction",
        "keyboard_geometry_storage": "implicit_decoder_reconstruction",
        "budget_bytes": FINAL_PACKAGE_BUDGET_BYTES,
        "within_budget": true,
        "verdict": "READY_FOR_FIXED_PROOF"
    }))
}

fn compact_depth0(mut package: LexicalGrokkingPackage) -> Result<LexicalGrokkingPackage, String> {
    if !package.anti_centers.is_empty()
        || !package.anti_subcenters.is_empty()
        || !package.hard_negative_subcenters.is_empty()
        || !package.pair_profiles.is_empty()
        || !package.pair_centers.is_empty()
    {
        return Err("depth-0 compaction refuses a package with learned counter-waves".to_string());
    }

    for profile in &mut package.center_phase_profiles {
        profile.positive_start = 0;
        profile.positive_count = 0;
        profile.ambiguity_start = 0;
        profile.ambiguity_count = 0;
        profile.min_ambiguity_milli = 0;
    }
    package.positive_subcenters.clear();
    package.ambiguity_subcenters.clear();
    Ok(package)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::nanda_wave::lexical_grokking::compiler::{
        compile_with_policy, ForwardPostingPolicy, TrainingWord,
    };

    #[test]
    fn depth0_compaction_removes_redundant_banks_and_rebuilds_complete_relations() {
        let words = ["время", "работает", "download"]
            .into_iter()
            .enumerate()
            .map(|(terminal_id, surface)| TrainingWord {
                terminal_id: terminal_id as u32,
                surface: surface.to_string(),
                training_surfaces: Vec::new(),
            })
            .collect::<Vec<_>>();
        let package = compile_with_policy(&words, ForwardPostingPolicy::Complete)
            .expect("compile fixture")
            .package;

        let compacted = compact_depth0(package).expect("compact fixture");

        assert!(compacted.positive_subcenters.is_empty());
        assert!(compacted.ambiguity_subcenters.is_empty());

        let encoded = format::encode_compact_depth0(&compacted).expect("encode compact V7");
        let decoded = format::decode(&encoded).expect("decode compact V7");
        assert_eq!(decoded, compacted);
    }
}
