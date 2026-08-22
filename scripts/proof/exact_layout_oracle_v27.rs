//! Standalone V27 exact-layout corpus oracle.
//!
//! Compile this file directly with `rustc`. It deliberately has no dependency
//! on the `lay` crate and owns its keyboard table, lexical parsers and corpus
//! classification rules.

use std::collections::{BTreeMap, BTreeSet};
use std::env;
use std::fs::{self, File};
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};
use std::process::Command;

const ROW_SCHEMA: &str = "lay-v27-exact-layout-oracle-rows-v1";
const MANIFEST_SCHEMA: &str = "lay-v27-exact-layout-oracle-manifest-v1";
const NORMALIZATION_VERSION: &str = "trim-comment-lower-script-v1-max24";
const PROJECTION_VERSION: &str = "us-qwerty-to-ru-bijective-v1";
const SAMPLE_LIMIT: usize = 512;

const RU_SOURCE_PATHS: [&str; 3] = [
    "data/lexicon/common_ru.txt",
    "data/lexicon/l2_surface_foundation_ru_100k.txt",
    "data/lexicon/l2_surface_hot_ru.txt",
];
const EN_HUNSPELL_PATH: &str = "/usr/share/hunspell/en_US.dic";
const EN_WORDS_PATH: &str = "/usr/share/dict/words";
const EN_TECHNICAL_PATH: &str = "data/lexicon/common_en_technical.txt";

const KEYBOARD_TABLE: [(char, char); 33] = [
    ('q', 'й'),
    ('w', 'ц'),
    ('e', 'у'),
    ('r', 'к'),
    ('t', 'е'),
    ('y', 'н'),
    ('u', 'г'),
    ('i', 'ш'),
    ('o', 'щ'),
    ('p', 'з'),
    ('[', 'х'),
    (']', 'ъ'),
    ('a', 'ф'),
    ('s', 'ы'),
    ('d', 'в'),
    ('f', 'а'),
    ('g', 'п'),
    ('h', 'р'),
    ('j', 'о'),
    ('k', 'л'),
    ('l', 'д'),
    (';', 'ж'),
    ('\'', 'э'),
    ('z', 'я'),
    ('x', 'ч'),
    ('c', 'с'),
    ('v', 'м'),
    ('b', 'и'),
    ('n', 'т'),
    ('m', 'ь'),
    (',', 'б'),
    ('.', 'ю'),
    ('`', 'ё'),
];

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
struct Row {
    class_id: String,
    operation: &'static str,
    input: String,
    target: String,
    expected: &'static str,
    profile: &'static str,
    decoder: &'static str,
    active_composition: bool,
    auto_replace: bool,
    auto_switch: bool,
    context: &'static str,
    snapshot: &'static str,
}

impl Row {
    fn certificate(class_id: &str, input: String, target: String, expected: &'static str) -> Self {
        Self {
            class_id: class_id.to_string(),
            operation: "certificate",
            input,
            target,
            expected,
            profile: "us_qwerty",
            decoder: "us",
            active_composition: true,
            auto_replace: true,
            auto_switch: true,
            context: "first",
            snapshot: "current",
        }
    }

    fn guard(input: String) -> Self {
        Self {
            class_id: "english_guard_member".to_string(),
            operation: "guard",
            input,
            target: String::new(),
            expected: "guarded",
            profile: "-",
            decoder: "-",
            active_composition: true,
            auto_replace: true,
            auto_switch: true,
            context: "first",
            snapshot: "current",
        }
    }

    fn line(&self) -> String {
        format!(
            "{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\n",
            self.class_id,
            self.operation,
            self.input,
            self.target,
            self.expected,
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

struct Options {
    root: PathBuf,
    rows: PathBuf,
    manifest: PathBuf,
    source: PathBuf,
    en_hunspell: PathBuf,
    en_words: PathBuf,
    swap_keys: bool,
}

fn main() {
    if let Err(error) = run() {
        eprintln!("exact-layout oracle failed: {error}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let options = parse_options()?;
    let root = options
        .root
        .canonicalize()
        .map_err(|error| format!("cannot canonicalize {}: {error}", options.root.display()))?;
    let table = keyboard_table(options.swap_keys);
    let inverse = inverse_keyboard_table(&table)?;

    let mut russian = BTreeSet::new();
    for relative in RU_SOURCE_PATHS {
        russian.extend(load_plain_russian(&root.join(relative))?);
    }

    let mut english = load_hunspell_english(&options.en_hunspell)?;
    english.extend(load_plain_english(&options.en_words)?);
    english.extend(load_plain_technical(&root.join(EN_TECHNICAL_PATH))?);

    let (eligible, collisions, edge_symbols, domain_protected) =
        classify_russian_surfaces(&russian, &english, &inverse);
    if eligible.is_empty() {
        return Err("eligible exact-layout denominator is empty".to_string());
    }
    if !options.swap_keys && collisions.len() < 423 {
        return Err(format!(
            "known-English collision denominator {} is below 423",
            collisions.len()
        ));
    }

    let mut rows = Vec::new();
    for (input, target) in &eligible {
        rows.push(Row::certificate(
            "eligible_lower",
            input.clone(),
            target.clone(),
            "apply",
        ));
    }
    for (input, target) in &collisions {
        rows.push(Row::certificate(
            "known_english_collision",
            input.clone(),
            target.clone(),
            "no_apply",
        ));
    }
    for word in &english {
        rows.push(Row::guard(word.clone()));
    }

    append_positive_variants(&mut rows, &eligible, &table);
    append_negative_variants(
        &mut rows,
        &eligible,
        &edge_symbols,
        &domain_protected,
        &russian,
        &table,
    );
    rows.sort();
    rows.dedup();

    let class_counts = count_classes(&rows);
    validate_required_classes(&class_counts)?;
    write_rows(&options.rows, &rows)?;
    write_manifest(
        &options,
        &root,
        &table,
        &class_counts,
        eligible.len(),
        collisions.len(),
        rows.len(),
    )?;
    println!(
        "V27_ORACLE rows={} eligible={} collisions={} classes={} mutation={}",
        rows.len(),
        eligible.len(),
        collisions.len(),
        class_counts.len(),
        if options.swap_keys {
            "swap-q-w"
        } else {
            "none"
        }
    );
    Ok(())
}

fn parse_options() -> Result<Options, String> {
    let mut root = None;
    let mut rows = None;
    let mut manifest = None;
    let mut source = None;
    let mut en_hunspell = PathBuf::from(EN_HUNSPELL_PATH);
    let mut en_words = PathBuf::from(EN_WORDS_PATH);
    let mut swap_keys = false;
    let mut args = env::args().skip(1);
    while let Some(argument) = args.next() {
        match argument.as_str() {
            "--root" => root = args.next().map(PathBuf::from),
            "--rows" => rows = args.next().map(PathBuf::from),
            "--manifest" => manifest = args.next().map(PathBuf::from),
            "--source" => source = args.next().map(PathBuf::from),
            "--en-hunspell" => {
                en_hunspell = PathBuf::from(args.next().ok_or("--en-hunspell needs a path")?)
            }
            "--en-words" => en_words = PathBuf::from(args.next().ok_or("--en-words needs a path")?),
            "--swap-q-w" => swap_keys = true,
            "--help" | "-h" => {
                println!(
                    "usage: exact_layout_oracle_v27 --root ROOT --rows FILE --manifest FILE --source FILE [--en-hunspell FILE] [--en-words FILE] [--swap-q-w]"
                );
                std::process::exit(0);
            }
            _ => return Err(format!("unknown argument {argument:?}")),
        }
    }
    Ok(Options {
        root: root.ok_or("--root is required")?,
        rows: rows.ok_or("--rows is required")?,
        manifest: manifest.ok_or("--manifest is required")?,
        source: source.ok_or("--source is required")?,
        en_hunspell,
        en_words,
        swap_keys,
    })
}

fn keyboard_table(swap_keys: bool) -> Vec<(char, char)> {
    let mut table = KEYBOARD_TABLE.to_vec();
    if swap_keys {
        let q = table.iter().position(|(source, _)| *source == 'q').unwrap();
        let w = table.iter().position(|(source, _)| *source == 'w').unwrap();
        let q_target = table[q].1;
        table[q].1 = table[w].1;
        table[w].1 = q_target;
    }
    table
}

fn inverse_keyboard_table(table: &[(char, char)]) -> Result<BTreeMap<char, char>, String> {
    let mut inverse = BTreeMap::new();
    for (source, target) in table {
        if inverse.insert(*target, *source).is_some() {
            return Err(format!("keyboard table is not bijective at {target:?}"));
        }
    }
    Ok(inverse)
}

fn classify_russian_surfaces(
    russian: &BTreeSet<String>,
    english: &BTreeSet<String>,
    inverse: &BTreeMap<char, char>,
) -> (
    Vec<(String, String)>,
    Vec<(String, String)>,
    Vec<(String, String)>,
    Vec<(String, String)>,
) {
    let mut eligible = Vec::new();
    let mut collisions = Vec::new();
    let mut edge_symbols = Vec::new();
    let mut domain_protected = Vec::new();
    for target in russian {
        let len = target.chars().count();
        if !(2..=24).contains(&len) || !target.chars().all(is_lower_russian_letter) {
            continue;
        }
        let Some(input) = target
            .chars()
            .map(|ch| inverse.get(&ch).copied())
            .collect::<Option<String>>()
        else {
            continue;
        };
        if !is_ascii_layout_surface(&input) {
            continue;
        }
        if !input
            .chars()
            .next()
            .is_some_and(|ch| ch.is_ascii_alphabetic())
            || !input
                .chars()
                .last()
                .is_some_and(|ch| ch.is_ascii_alphabetic())
        {
            edge_symbols.push((input, target.clone()));
            continue;
        }
        if english.contains(&input) {
            collisions.push((input, target.clone()));
        } else if is_domain_like_ascii(&input) {
            domain_protected.push((input, target.clone()));
        } else {
            eligible.push((input, target.clone()));
        }
    }
    (eligible, collisions, edge_symbols, domain_protected)
}

fn append_positive_variants(
    rows: &mut Vec<Row>,
    eligible: &[(String, String)],
    table: &[(char, char)],
) {
    for (input, target) in eligible.iter().take(SAMPLE_LIMIT) {
        let title_input = title_ascii(input);
        rows.push(Row::certificate(
            "eligible_title",
            title_input.clone(),
            project(&title_input, table),
            "apply",
        ));
        for context in ["ru", "ascii", "punct"] {
            let mut row = Row::certificate(
                &format!("context_{context}"),
                input.clone(),
                target.clone(),
                "apply",
            );
            row.context = context;
            rows.push(row);
        }
    }
    for (input, _) in eligible
        .iter()
        .filter(|(input, _)| input.chars().filter(|ch| ch.is_ascii_alphabetic()).count() > 4)
        .take(SAMPLE_LIMIT)
    {
        let upper_input = input.to_ascii_uppercase();
        rows.push(Row::certificate(
            "eligible_upper",
            upper_input.clone(),
            project(&upper_input, table),
            "apply",
        ));
    }
}

fn append_negative_variants(
    rows: &mut Vec<Row>,
    eligible: &[(String, String)],
    edge_symbols: &[(String, String)],
    domain_protected: &[(String, String)],
    russian: &BTreeSet<String>,
    table: &[(char, char)],
) {
    for (index, (input, target)) in eligible.iter().take(SAMPLE_LIMIT).enumerate() {
        let mut letter_index = 0usize;
        let mixed = input
            .chars()
            .map(|ch| {
                if !ch.is_ascii_alphabetic() {
                    return ch;
                }
                let current = letter_index;
                letter_index += 1;
                if current == 1 {
                    ch.to_ascii_uppercase()
                } else {
                    ch
                }
            })
            .collect();
        rows.push(Row::certificate(
            "mixed_case_source",
            mixed,
            target.clone(),
            "no_apply",
        ));

        let mut typo = input.clone();
        let byte = typo
            .char_indices()
            .nth(typo.chars().count() / 2)
            .map_or(typo.len(), |(byte, _)| byte);
        typo.insert(byte, '-');
        rows.push(Row::certificate(
            "layout_typo",
            typo,
            target.clone(),
            "no_apply",
        ));

        let unknown_input = unknown_surface(input, index, russian, table);
        let unknown_target = project(&unknown_input, table);
        rows.push(Row::certificate(
            "unknown_non_terminal_target",
            unknown_input,
            unknown_target,
            "no_apply",
        ));

        let generated_input = format!("{input}q");
        let generated_target = project(&generated_input, table);
        if !russian.contains(&generated_target) {
            rows.push(Row::certificate(
                "generated_only_morphology_target",
                generated_input,
                generated_target,
                "no_apply",
            ));
        }

        for (class_id, decorated) in [
            ("url_or_domain", format!("{input}.ru")),
            ("cli_option", format!("--{input}")),
            ("digit_mixed", format!("{input}2")),
            ("mixed_script", format!("я{input}")),
        ] {
            rows.push(Row::certificate(
                class_id,
                decorated,
                target.clone(),
                "no_apply",
            ));
        }

        let mut active_ru = Row::certificate(
            "active_layout_mismatch",
            input.clone(),
            target.clone(),
            "no_apply",
        );
        active_ru.decoder = "ru";
        rows.push(active_ru);

        let mut unknown_profile = Row::certificate(
            "unknown_factory_profile",
            input.clone(),
            target.clone(),
            "no_apply",
        );
        unknown_profile.profile = "unknown";
        rows.push(unknown_profile);

        let mut ru_profile = Row::certificate(
            "ru_factory_with_us_decoder",
            input.clone(),
            target.clone(),
            "no_apply",
        );
        ru_profile.profile = "ru";
        rows.push(ru_profile);

        rows.push(Row::certificate(
            "ru_to_en",
            target.clone(),
            input.clone(),
            "no_apply",
        ));

        for (class_id, active, replace, switch) in [
            ("inactive_composition", false, true, true),
            ("auto_replace_off", true, false, true),
            ("auto_switch_off", true, true, false),
        ] {
            let mut row = Row::certificate(class_id, input.clone(), target.clone(), "no_apply");
            row.active_composition = active;
            row.auto_replace = replace;
            row.auto_switch = switch;
            rows.push(row);
        }

        let mut corrupt = Row::certificate(
            "authority_fingerprint_mismatch",
            input.clone(),
            target.clone(),
            "no_apply",
        );
        corrupt.snapshot = "corrupt_keyboard";
        rows.push(corrupt);
    }

    for (input, target) in edge_symbols.iter().take(SAMPLE_LIMIT) {
        rows.push(Row::certificate(
            "edge_symbol_ambiguity",
            input.clone(),
            target.clone(),
            "no_apply",
        ));
    }
    for (input, target) in domain_protected.iter().take(SAMPLE_LIMIT) {
        rows.push(Row::certificate(
            "domain_like_protected",
            input.clone(),
            target.clone(),
            "no_apply",
        ));
    }
    for (input, _) in eligible
        .iter()
        .filter(|(input, _)| {
            let letters = input.chars().filter(|ch| ch.is_ascii_alphabetic()).count();
            (2..=4).contains(&letters)
        })
        .take(SAMPLE_LIMIT)
    {
        let upper_input = input.to_ascii_uppercase();
        rows.push(Row::certificate(
            "upper_acronym_protected",
            upper_input.clone(),
            project(&upper_input, table),
            "no_apply",
        ));
    }
}

fn unknown_surface(
    input: &str,
    index: usize,
    russian: &BTreeSet<String>,
    table: &[(char, char)],
) -> String {
    let keys = ['q', 'w', 'f', 's', 'j', 'l', 'z', 'x'];
    for salt in 0..keys.len() {
        let key = keys[(index + salt) % keys.len()];
        let mut candidate = input.to_string();
        candidate.push(key);
        let target = project(&candidate, table);
        if target.chars().count() <= 24 && !russian.contains(&target) {
            return candidate;
        }
    }
    format!(
        "{}{}",
        input,
        "q".repeat(25usize.saturating_sub(input.chars().count()))
    )
}

fn project(input: &str, table: &[(char, char)]) -> String {
    input
        .chars()
        .map(|ch| {
            let lower = ch.to_ascii_lowercase();
            let projected = table
                .iter()
                .find_map(|(source, target)| (*source == lower).then_some(*target))
                .unwrap_or(ch);
            if ch.is_ascii_uppercase() {
                projected.to_uppercase().next().unwrap_or(projected)
            } else {
                projected
            }
        })
        .collect()
}

fn load_plain_russian(path: &Path) -> Result<BTreeSet<String>, String> {
    let text = read_text(path)?;
    Ok(data_lines(&text)
        .map(str::to_lowercase)
        .filter(|word| {
            let len = word.chars().count();
            (1..=24).contains(&len)
                && word
                    .chars()
                    .all(|ch| is_lower_russian_letter(ch) || ch == '-' || ch == '\'')
        })
        .collect())
}

fn load_hunspell_english(path: &Path) -> Result<BTreeSet<String>, String> {
    let text = read_text(path)?;
    Ok(data_lines(&text)
        .filter_map(|line| line.split('/').next())
        .filter_map(normalize_english)
        .collect())
}

fn load_plain_english(path: &Path) -> Result<BTreeSet<String>, String> {
    let text = read_text(path)?;
    Ok(data_lines(&text).filter_map(normalize_english).collect())
}

fn load_plain_technical(path: &Path) -> Result<BTreeSet<String>, String> {
    let text = read_text(path)?;
    Ok(data_lines(&text).map(str::to_ascii_lowercase).collect())
}

fn normalize_english(word: &str) -> Option<String> {
    let word = word.trim();
    if word.chars().count() < 2 || !word.is_ascii() {
        return None;
    }
    let mut has_letter = false;
    for ch in word.chars() {
        if ch.is_ascii_alphabetic() {
            has_letter = true;
        } else if !matches!(
            ch,
            '-' | '_'
                | '.'
                | '/'
                | '+'
                | ','
                | ';'
                | '\''
                | '['
                | ']'
                | '`'
                | '?'
                | '!'
                | ':'
                | '$'
                | '%'
                | '^'
                | '&'
                | '#'
                | '@'
        ) {
            return None;
        }
    }
    has_letter.then(|| word.to_ascii_lowercase())
}

fn data_lines(text: &str) -> impl Iterator<Item = &str> {
    text.lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
}

fn read_text(path: &Path) -> Result<String, String> {
    fs::read_to_string(path).map_err(|error| format!("cannot read {}: {error}", path.display()))
}

fn is_lower_russian_letter(ch: char) -> bool {
    matches!(ch, 'а'..='я' | 'ё')
}

fn is_ascii_layout_surface(surface: &str) -> bool {
    surface.is_ascii()
        && surface.chars().any(|ch| ch.is_ascii_alphabetic())
        && surface.chars().all(|ch| {
            ch.is_ascii_alphabetic()
                || matches!(
                    ch,
                    '\'' | ';'
                        | '['
                        | ']'
                        | '`'
                        | ','
                        | '.'
                        | '-'
                        | '{'
                        | '}'
                        | ':'
                        | '"'
                        | '<'
                        | '>'
                        | '~'
                )
        })
}

fn is_domain_like_ascii(surface: &str) -> bool {
    surface.rsplit_once('.').is_some_and(|(name, suffix)| {
        name.chars().filter(|ch| ch.is_ascii_alphabetic()).count() >= 2
            && (2..=4).contains(&suffix.chars().count())
            && suffix.chars().all(|ch| ch.is_ascii_alphabetic())
    })
}

fn title_ascii(word: &str) -> String {
    let mut chars = word.chars();
    let Some(first) = chars.next() else {
        return String::new();
    };
    first.to_ascii_uppercase().to_string() + chars.as_str()
}

fn count_classes(rows: &[Row]) -> BTreeMap<String, usize> {
    let mut counts = BTreeMap::new();
    for row in rows {
        *counts.entry(row.class_id.clone()).or_insert(0) += 1;
    }
    counts
}

fn validate_required_classes(counts: &BTreeMap<String, usize>) -> Result<(), String> {
    const REQUIRED: [&str; 25] = [
        "eligible_lower",
        "eligible_title",
        "eligible_upper",
        "known_english_collision",
        "english_guard_member",
        "context_ru",
        "context_ascii",
        "context_punct",
        "layout_typo",
        "unknown_non_terminal_target",
        "generated_only_morphology_target",
        "url_or_domain",
        "cli_option",
        "digit_mixed",
        "mixed_script",
        "mixed_case_source",
        "edge_symbol_ambiguity",
        "domain_like_protected",
        "upper_acronym_protected",
        "active_layout_mismatch",
        "unknown_factory_profile",
        "ru_factory_with_us_decoder",
        "ru_to_en",
        "authority_fingerprint_mismatch",
        "inactive_composition",
    ];
    for class_id in REQUIRED {
        if counts.get(class_id).copied().unwrap_or(0) == 0 {
            return Err(format!(
                "required class {class_id:?} has an empty denominator"
            ));
        }
    }
    Ok(())
}

fn write_rows(path: &Path, rows: &[Row]) -> Result<(), String> {
    ensure_parent(path)?;
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
            .write_all(row.line().as_bytes())
            .map_err(|error| error.to_string())?;
    }
    writer.flush().map_err(|error| error.to_string())
}

fn write_manifest(
    options: &Options,
    root: &Path,
    table: &[(char, char)],
    class_counts: &BTreeMap<String, usize>,
    eligible_count: usize,
    collision_count: usize,
    row_count: usize,
) -> Result<(), String> {
    ensure_parent(&options.manifest)?;
    let executable = env::current_exe().map_err(|error| error.to_string())?;
    let rustc = command_output("rustc", &["-vV"])?;
    let mut source_hashes = BTreeMap::new();
    for relative in RU_SOURCE_PATHS {
        source_hashes.insert(relative.to_string(), sha256(&root.join(relative))?);
    }
    source_hashes.insert(EN_HUNSPELL_PATH.to_string(), sha256(&options.en_hunspell)?);
    source_hashes.insert(EN_WORDS_PATH.to_string(), sha256(&options.en_words)?);
    source_hashes.insert(
        EN_TECHNICAL_PATH.to_string(),
        sha256(&root.join(EN_TECHNICAL_PATH))?,
    );

    let mut json = String::new();
    json.push_str("{\n");
    json.push_str(&format!("  \"schema\": \"{MANIFEST_SCHEMA}\",\n"));
    json.push_str(&format!(
        "  \"normalization_version\": \"{NORMALIZATION_VERSION}\",\n"
    ));
    json.push_str(&format!(
        "  \"projection_version\": \"{PROJECTION_VERSION}\",\n"
    ));
    json.push_str(&format!(
        "  \"keyboard_table_fingerprint\": \"{:016x}\",\n",
        keyboard_fingerprint(table)
    ));
    json.push_str(&format!(
        "  \"mutation\": \"{}\",\n",
        if options.swap_keys {
            "swap-q-w"
        } else {
            "none"
        }
    ));
    json.push_str(&format!(
        "  \"oracle_source_sha256\": \"{}\",\n",
        sha256(&options.source)?
    ));
    json.push_str(&format!(
        "  \"oracle_binary_sha256\": \"{}\",\n",
        sha256(&executable)?
    ));
    json.push_str(&format!(
        "  \"rustc_vv\": \"{}\",\n",
        json_escape(rustc.trim_end())
    ));
    json.push_str(&format!(
        "  \"rows_sha256\": \"{}\",\n",
        sha256(&options.rows)?
    ));
    json.push_str(&format!("  \"row_count\": {row_count},\n"));
    json.push_str(&format!("  \"eligible_count\": {eligible_count},\n"));
    json.push_str(&format!(
        "  \"known_english_collision_count\": {collision_count},\n"
    ));
    json.push_str("  \"source_sha256\": {\n");
    for (index, (path, hash)) in source_hashes.iter().enumerate() {
        let comma = if index + 1 == source_hashes.len() {
            ""
        } else {
            ","
        };
        json.push_str(&format!(
            "    \"{}\": \"{}\"{}\n",
            json_escape(path),
            hash,
            comma
        ));
    }
    json.push_str("  },\n  \"class_counts\": {\n");
    for (index, (class_id, count)) in class_counts.iter().enumerate() {
        let comma = if index + 1 == class_counts.len() {
            ""
        } else {
            ","
        };
        json.push_str(&format!(
            "    \"{}\": {}{}\n",
            json_escape(class_id),
            count,
            comma
        ));
    }
    json.push_str("  }\n}\n");
    fs::write(&options.manifest, json)
        .map_err(|error| format!("cannot write {}: {error}", options.manifest.display()))
}

fn keyboard_fingerprint(table: &[(char, char)]) -> u64 {
    let mut digest = 0xcbf2_9ce4_8422_2325_u64;
    for (source, target) in table {
        for byte in (*source as u32)
            .to_le_bytes()
            .into_iter()
            .chain((*target as u32).to_le_bytes())
        {
            digest ^= u64::from(byte);
            digest = digest.wrapping_mul(0x100_0000_01b3);
        }
    }
    digest
}

fn sha256(path: &Path) -> Result<String, String> {
    let output = Command::new("sha256sum")
        .arg("--")
        .arg(path)
        .output()
        .map_err(|error| format!("cannot hash {}: {error}", path.display()))?;
    if !output.status.success() {
        return Err(format!("sha256sum failed for {}", path.display()));
    }
    String::from_utf8(output.stdout)
        .map_err(|error| error.to_string())?
        .split_whitespace()
        .next()
        .map(str::to_string)
        .ok_or_else(|| format!("sha256sum returned no digest for {}", path.display()))
}

fn command_output(command: &str, args: &[&str]) -> Result<String, String> {
    let output = Command::new(command)
        .args(args)
        .output()
        .map_err(|error| format!("cannot run {command}: {error}"))?;
    if !output.status.success() {
        return Err(format!("{command} exited with {}", output.status));
    }
    String::from_utf8(output.stdout).map_err(|error| error.to_string())
}

fn ensure_parent(path: &Path) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|error| format!("cannot create {}: {error}", parent.display()))?;
    }
    Ok(())
}

fn json_escape(value: &str) -> String {
    let mut escaped = String::new();
    for ch in value.chars() {
        match ch {
            '"' => escaped.push_str("\\\""),
            '\\' => escaped.push_str("\\\\"),
            '\n' => escaped.push_str("\\n"),
            '\r' => escaped.push_str("\\r"),
            '\t' => escaped.push_str("\\t"),
            ch if ch.is_control() => escaped.push_str(&format!("\\u{:04x}", ch as u32)),
            ch => escaped.push(ch),
        }
    }
    escaped
}
