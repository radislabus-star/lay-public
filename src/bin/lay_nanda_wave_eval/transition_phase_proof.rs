use std::collections::BTreeMap;
use std::fs;
use std::io;
use std::path::PathBuf;

use lay::nanda_wave::L2PhaseTrainingEntry;

const DEFAULT_DATASET: &str = "data/nanda_training/generated_cases.tsv";

pub(crate) fn print_json(args: &[String]) -> io::Result<()> {
    let dataset = arg_value(args, "--dataset")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(DEFAULT_DATASET));
    let entries = read_entries(&dataset)?;
    let mut report = lay::nanda_wave::l2_transition_phase_proof_json(&entries);
    if let Some(object) = report.as_object_mut() {
        object.insert("dataset".to_string(), dataset.display().to_string().into());
        object.insert("labeled_rows".to_string(), entries.len().into());
    }
    println!("{}", serde_json::to_string_pretty(&report)?);
    Ok(())
}

fn read_entries(path: &std::path::Path) -> io::Result<Vec<L2PhaseTrainingEntry>> {
    let text = fs::read_to_string(path)?;
    let rows = text
        .lines()
        .skip(1)
        .filter_map(|line| {
            let cols = line.split('\t').collect::<Vec<_>>();
            (cols.len() >= 8 && cols[2] != cols[3]).then(|| {
                (
                    cols[0].to_string(),
                    cols[2].trim_end().to_string(),
                    cols[3].trim_end().to_string(),
                    cols[4].to_string(),
                    cols[5] == "1",
                )
            })
        })
        .collect::<Vec<_>>();
    let group_operators = rows
        .iter()
        .filter(|row| row.4)
        .map(|row| {
            (
                row.0.clone(),
                lay::nanda_wave::infer_l2_transition_operator(&row.1, &row.2, &row.3).to_string(),
            )
        })
        .collect::<BTreeMap<_, _>>();
    Ok(rows
        .into_iter()
        .filter_map(|(group, original, candidate, operation, accepted)| {
            let operation = group_operators.get(&group).cloned().unwrap_or(operation);
            (!original.is_empty() && !candidate.is_empty()).then_some(L2PhaseTrainingEntry {
                original,
                candidate,
                operation,
                accepted,
                count: 1,
            })
        })
        .collect())
}

fn arg_value(args: &[String], name: &str) -> Option<String> {
    args.windows(2)
        .find_map(|pair| (pair[0] == name).then(|| pair[1].clone()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn canonical_dataset_contains_positive_and_negative_phase_rows() {
        let entries = read_entries(std::path::Path::new(DEFAULT_DATASET)).unwrap();
        assert!(entries.iter().any(|entry| entry.accepted));
        assert!(entries.iter().any(|entry| !entry.accepted));
    }

    #[test]
    fn canonical_dataset_proves_heldout_phase_transfer() {
        let entries = read_entries(std::path::Path::new(DEFAULT_DATASET)).unwrap();
        let report = lay::nanda_wave::l2_transition_phase_proof_json(&entries);

        assert_eq!(report["verdict"], "PASS");
        assert_eq!(report["full_phase_false_accepts"], 0);
        assert_eq!(report["exact_memory_rows_after_compile"], 0);
        assert!(report["causal_positive_support_drop"].as_u64().unwrap() > 0);
        let promoted = report["promoted_operators"].as_array().unwrap();
        assert!(promoted.iter().any(|item| item == "layout_projection"));
        assert!(promoted.iter().any(|item| item == "adjacent_transposition"));
        assert!(promoted.iter().any(|item| item == "accept_completion"));
        assert_eq!(promoted.len(), 11);
        assert_eq!(
            report["by_operator"]["composite_typo"]["promotion_verdict"],
            "PROMOTED"
        );
        assert_eq!(report["modes"]["no_phase"]["positive_support"], 0);
        assert_eq!(report["modes"]["magnitude_only"]["positive_support"], 0);
    }
}
