use lay::dict::{convert, Direction};
use std::collections::{BTreeMap, BTreeSet};
use std::env;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

const DEFAULT_OUT: &str = "data/nanda_training/generated_cases.tsv";

#[derive(Debug, Clone)]
struct Case {
    original: String,
    expected: String,
    operation: String,
    source: String,
    reason: String,
}

#[derive(Debug, Clone)]
struct Row {
    group_id: String,
    context: String,
    original: String,
    candidate: String,
    operation: String,
    label: bool,
    source: String,
    reason: String,
}

fn main() -> io::Result<()> {
    let out = output_path();
    let cases = build_cases()?;
    let rows = build_rows(&cases);
    write_rows(&out, &rows)?;
    print_summary(&out, &cases, &rows);
    Ok(())
}

fn output_path() -> PathBuf {
    let args = env::args().collect::<Vec<_>>();
    args.windows(2)
        .find_map(|pair| (pair[0] == "--out").then(|| PathBuf::from(&pair[1])))
        .unwrap_or_else(|| PathBuf::from(DEFAULT_OUT))
}

fn build_cases() -> io::Result<Vec<Case>> {
    let mut cases = Vec::new();
    add_nanda_seed_cases(&mut cases)?;
    add_short_alternating_cases(&mut cases)?;
    add_technical_layout_cases(&mut cases)?;
    add_keep_lines(
        &mut cases,
        "tests/fixtures/typing_assist_clean_mixed.txt",
        "clean_mixed_keep",
    )?;
    add_keep_lines(
        &mut cases,
        "tests/fixtures/typing_assist_shell_keep.txt",
        "shell_keep",
    )?;
    dedupe_cases(cases)
}

fn add_nanda_seed_cases(cases: &mut Vec<Case>) -> io::Result<()> {
    let path = "data/nanda_wave_synthetic_cases.tsv";
    for line in read_data_lines(path)? {
        let cols = line.split('\t').collect::<Vec<_>>();
        if cols.len() < 3 {
            continue;
        }
        cases.push(Case {
            original: decode_fixture(cols[0]),
            expected: decode_fixture(cols[1]),
            operation: operation_from_reason(cols[2]).to_string(),
            source: path.to_string(),
            reason: cols[2].to_string(),
        });
    }
    Ok(())
}

fn add_short_alternating_cases(cases: &mut Vec<Case>) -> io::Result<()> {
    let path = "tests/fixtures/typing_assist_short_alternating_pairs.tsv";
    let pairs = read_data_lines(path)?
        .into_iter()
        .filter_map(|line| {
            let cols = line.split('\t').collect::<Vec<_>>();
            (cols.len() >= 2).then(|| (cols[0].to_string(), cols[1].to_string()))
        })
        .collect::<Vec<_>>();
    for (idx, chunk) in pairs.chunks(12).enumerate() {
        let expected = alternating_expected(chunk);
        cases.push(Case {
            original: chunk
                .iter()
                .flat_map(|(ru, en)| [convert(ru, Direction::Ru2Us), en.clone()])
                .collect::<Vec<_>>()
                .join(" ")
                + " ",
            expected: expected.clone(),
            operation: "layout".to_string(),
            source: path.to_string(),
            reason: format!("short_alternating_ru_wrong_{idx}"),
        });
        cases.push(Case {
            original: chunk
                .iter()
                .flat_map(|(ru, en)| [ru.clone(), convert(en, Direction::Us2Ru)])
                .collect::<Vec<_>>()
                .join(" ")
                + " ",
            expected: expected.clone(),
            operation: "layout".to_string(),
            source: path.to_string(),
            reason: format!("short_alternating_en_wrong_{idx}"),
        });
        cases.push(Case {
            original: chunk
                .iter()
                .flat_map(|(ru, en)| [convert(ru, Direction::Ru2Us), convert(en, Direction::Us2Ru)])
                .collect::<Vec<_>>()
                .join(" ")
                + " ",
            expected,
            operation: "layout".to_string(),
            source: path.to_string(),
            reason: format!("short_alternating_all_wrong_{idx}"),
        });
    }
    Ok(())
}

fn alternating_expected(chunk: &[(String, String)]) -> String {
    chunk
        .iter()
        .flat_map(|(ru, en)| [ru.as_str(), en.as_str()])
        .collect::<Vec<_>>()
        .join(" ")
        + " "
}

fn add_technical_layout_cases(cases: &mut Vec<Case>) -> io::Result<()> {
    let path = "tests/fixtures/typing_assist_ru_to_en_synthetic.txt";
    for token in read_data_lines(path)? {
        cases.push(Case {
            original: convert(&token, Direction::Us2Ru) + " ",
            expected: token + " ",
            operation: "layout".to_string(),
            source: path.to_string(),
            reason: "technical_ru_layout".to_string(),
        });
    }
    Ok(())
}

fn add_keep_lines(cases: &mut Vec<Case>, path: &str, reason: &str) -> io::Result<()> {
    for line in read_data_lines(path)? {
        let text = decode_fixture(&line);
        cases.push(Case {
            original: text.clone(),
            expected: text,
            operation: "keep".to_string(),
            source: path.to_string(),
            reason: reason.to_string(),
        });
    }
    Ok(())
}

fn dedupe_cases(cases: Vec<Case>) -> io::Result<Vec<Case>> {
    let mut seen = BTreeSet::new();
    Ok(cases
        .into_iter()
        .filter(|case| seen.insert((case.original.clone(), case.expected.clone())))
        .collect())
}

fn build_rows(cases: &[Case]) -> Vec<Row> {
    let mut rows = Vec::new();
    for (idx, case) in cases.iter().enumerate() {
        let group_id = format!("generated:{idx:04}:{}", case.reason);
        let mut seen = BTreeSet::new();
        push_candidate(
            &mut rows,
            &mut seen,
            &group_id,
            case,
            &case.expected,
            &case.operation,
            true,
        );
        if case.original != case.expected {
            push_candidate(
                &mut rows,
                &mut seen,
                &group_id,
                case,
                &case.original,
                "keep",
                false,
            );
        }
        push_generic_negatives(&mut rows, &mut seen, &group_id, case);
    }
    rows
}

fn push_generic_negatives(
    rows: &mut Vec<Row>,
    seen: &mut BTreeSet<String>,
    group_id: &str,
    case: &Case,
) {
    let flipped = convert(&case.original, Direction::Ru2Us);
    push_candidate(rows, seen, group_id, case, &flipped, "layout", false);
    let flipped = convert(&case.original, Direction::Us2Ru);
    push_candidate(rows, seen, group_id, case, &flipped, "layout", false);
    let glued = case.original.replace(' ', "");
    push_candidate(rows, seen, group_id, case, &glued, "glue", false);
}

fn push_candidate(
    rows: &mut Vec<Row>,
    seen: &mut BTreeSet<String>,
    group_id: &str,
    case: &Case,
    candidate: &str,
    operation: &str,
    label: bool,
) {
    if candidate.is_empty() || !seen.insert(candidate.to_string()) {
        return;
    }
    rows.push(Row {
        group_id: group_id.to_string(),
        context: String::new(),
        original: case.original.clone(),
        candidate: candidate.to_string(),
        operation: operation.to_string(),
        label,
        source: case.source.clone(),
        reason: case.reason.clone(),
    });
}

fn write_rows(path: &Path, rows: &[Row]) -> io::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let mut text =
        "group_id\tcontext\toriginal\tcandidate\toperation\tlabel\tsource\treason\n".to_string();
    for row in rows {
        text.push_str(&format!(
            "{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\n",
            clean_field(&row.group_id),
            clean_field(&row.context),
            clean_field(&row.original),
            clean_field(&row.candidate),
            clean_field(&row.operation),
            usize::from(row.label),
            clean_field(&row.source),
            clean_field(&row.reason)
        ));
    }
    fs::write(path, text)
}

fn print_summary(path: &Path, cases: &[Case], rows: &[Row]) {
    let mut by_operation = BTreeMap::<&str, usize>::new();
    for case in cases {
        *by_operation.entry(&case.operation).or_default() += 1;
    }
    println!("wrote {}", path.display());
    println!("cases: {}", cases.len());
    println!("rows: {}", rows.len());
    for (operation, count) in by_operation {
        println!("  {operation}: {count}");
    }
}

fn read_data_lines(path: &str) -> io::Result<Vec<String>> {
    Ok(fs::read_to_string(path)?
        .lines()
        .map(str::trim_end)
        .filter(|line| !line.trim().is_empty() && !line.trim_start().starts_with('#'))
        .map(ToOwned::to_owned)
        .collect())
}

fn operation_from_reason(reason: &str) -> &str {
    if reason.contains("split") {
        "split"
    } else if reason.contains("typo") || reason.contains("grammar") {
        "typo"
    } else if reason.contains("veto") || reason.contains("protected") || reason.contains("keep") {
        "keep"
    } else {
        "layout"
    }
}

fn decode_fixture(value: &str) -> String {
    value.replace("\\s", " ")
}

fn clean_field(value: &str) -> String {
    value.replace(['\t', '\n', '\r'], " ")
}
