//! Compact nonlinear L2 candidate admission memory.
//!
//! This is a shadow field: candidate generators may propose words, but this
//! package learns whether a replacement transition has a stable positive phase
//! center and a weak negative/trap phase center.
#![allow(dead_code)]

use std::env;
use std::f64::consts::TAU;
use std::io;
use std::path::{Path, PathBuf};

use super::mode::mix64_golden;

const MAGIC: &[u8; 8] = b"LAYPC001";
const CELLS: usize = 256;
const HEADER_BYTES: usize = 12;
const CELL_BYTES: usize = 16;
const ADMISSION_THRESHOLD_MICRO: i64 = 50_000;

#[derive(Clone, Copy, Debug, Default, PartialEq)]
struct PhaseCell {
    re: f64,
    im: f64,
}

#[derive(Clone, Debug)]
struct PhaseRuntime {
    positive: Vec<PhaseCell>,
    negative: Vec<PhaseCell>,
}

#[derive(Clone, Debug)]
struct PhaseCompiler {
    positive_sum: Vec<PhaseCell>,
    negative_sum: Vec<PhaseCell>,
    positive_count: usize,
    negative_count: usize,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct L2PhaseExample {
    original: String,
    expected: String,
    operation: String,
    count: usize,
}

pub(super) fn default_phase_memory_path() -> PathBuf {
    env::var_os("LAY_NANDA_L2_PHASE_MEMORY")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            env::var_os("HOME")
                .map(PathBuf::from)
                .unwrap_or_else(|| PathBuf::from("."))
                .join(".local/share/lay/nanda_wave/l2_candidate_phase.nwpc")
        })
}

pub(super) fn write_phase_memory_from_entries<I>(path: &Path, entries: I) -> io::Result<usize>
where
    I: IntoIterator<Item = (String, String, String, usize)>,
{
    let examples = entries
        .into_iter()
        .map(|(original, expected, operation, count)| L2PhaseExample {
            original,
            expected,
            operation,
            count,
        })
        .collect::<Vec<_>>();
    let runtime = train_phase_runtime(examples)?;
    let bytes = runtime.to_bytes();
    crate::private_file::write_private_bytes(path, &bytes)?;
    Ok(bytes.len())
}

fn train_phase_runtime<I>(examples: I) -> io::Result<PhaseRuntime>
where
    I: IntoIterator<Item = L2PhaseExample>,
{
    let mut compiler = PhaseCompiler::new();
    let mut trained = 0usize;
    for example in examples {
        if !is_trainable_pair(&example.original, &example.expected) {
            continue;
        }
        let positive_atoms =
            candidate_phase_atoms(&example.original, &example.expected, &example.operation);
        for _ in 0..example.count.clamp(1, 8) {
            compiler.add_atoms(&positive_atoms, true);
        }
        for trap in generated_negative_candidates(&example.original, &example.expected) {
            let negative_atoms =
                candidate_phase_atoms(&example.original, &trap, &example.operation);
            compiler.add_atoms(&negative_atoms, false);
        }
        trained += 1;
    }
    if trained == 0 || compiler.positive_count == 0 || compiler.negative_count == 0 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "not enough L2 phase examples",
        ));
    }
    Ok(compiler.compile())
}

impl PhaseCompiler {
    fn new() -> Self {
        Self {
            positive_sum: vec![PhaseCell::default(); CELLS],
            negative_sum: vec![PhaseCell::default(); CELLS],
            positive_count: 0,
            negative_count: 0,
        }
    }

    fn add_atoms(&mut self, atoms: &[String], positive: bool) {
        let vector = phase_vector_from_atoms(atoms);
        let target = if positive {
            self.positive_count += 1;
            &mut self.positive_sum
        } else {
            self.negative_count += 1;
            &mut self.negative_sum
        };
        for (target, source) in target.iter_mut().zip(vector) {
            target.re += source.re;
            target.im += source.im;
        }
    }

    fn compile(self) -> PhaseRuntime {
        PhaseRuntime {
            positive: phase_center_from_sum(&self.positive_sum),
            negative: phase_center_from_sum(&self.negative_sum),
        }
    }
}

impl PhaseRuntime {
    fn admission_margin_micro(&self, original: &str, candidate: &str, operation: &str) -> i64 {
        let atoms = candidate_phase_atoms(original, candidate, operation);
        let vector = phase_vector_from_atoms(&atoms);
        let positive = phase_coherence(&vector, &self.positive);
        let negative = phase_coherence(&vector, &self.negative);
        ((positive - negative) * 1_000_000.0).round() as i64
    }

    fn admits(&self, original: &str, candidate: &str, operation: &str) -> bool {
        self.admission_margin_micro(original, candidate, operation) >= ADMISSION_THRESHOLD_MICRO
    }

    fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(HEADER_BYTES + CELLS * CELL_BYTES * 2);
        bytes.extend_from_slice(MAGIC);
        bytes.extend_from_slice(&(CELLS as u32).to_le_bytes());
        write_cells(&mut bytes, &self.positive);
        write_cells(&mut bytes, &self.negative);
        bytes
    }

    fn from_bytes(bytes: &[u8]) -> io::Result<Self> {
        if bytes.len() != HEADER_BYTES + CELLS * CELL_BYTES * 2 || &bytes[..MAGIC.len()] != MAGIC {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "invalid L2 phase package",
            ));
        }
        let cells = u32::from_le_bytes(bytes[8..12].try_into().unwrap_or_default()) as usize;
        if cells != CELLS {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "unsupported L2 phase width",
            ));
        }
        let mut offset = HEADER_BYTES;
        let positive = read_cells(bytes, &mut offset)?;
        let negative = read_cells(bytes, &mut offset)?;
        Ok(Self { positive, negative })
    }
}

fn candidate_phase_atoms(original: &str, candidate: &str, operation: &str) -> Vec<String> {
    let original = normalize_phrase(original);
    let candidate = normalize_phrase(candidate);
    let original_last = last_word(&original);
    let candidate_last = last_word(&candidate);
    let original_len = original_last.chars().count();
    let candidate_len = candidate_last.chars().count();
    let prefix = common_prefix_chars(original_last, candidate_last);
    let suffix = common_suffix_chars(original_last, candidate_last);
    let distance = crate::text_metrics::damerau_levenshtein(original_last, candidate_last);

    let mut atoms = vec![
        "field:l2-candidate-admission".to_string(),
        format!("op:{}", normalize_operation(operation)),
        format!(
            "word-count:{}->{}",
            word_count(&original),
            word_count(&candidate)
        ),
        format!(
            "script:{}->{}",
            script_class(original_last),
            script_class(candidate_last)
        ),
        format!(
            "len:{}->{}",
            len_bucket(original_len),
            len_bucket(candidate_len)
        ),
        format!(
            "delta:{}",
            signed_bucket(candidate_len as isize - original_len as isize)
        ),
        format!("prefix:{}", small_bucket(prefix)),
        format!("suffix:{}", small_bucket(suffix)),
        format!("edit:{}", small_bucket(distance)),
        format!(
            "boundary:{}",
            if word_count(&original) == word_count(&candidate) {
                "same"
            } else {
                "changed"
            }
        ),
    ];

    push_shape_atoms(&mut atoms, "from", original_last);
    push_shape_atoms(&mut atoms, "to", candidate_last);
    atoms
}

fn phase_vector_from_atoms(atoms: &[String]) -> Vec<PhaseCell> {
    let mut vector = vec![PhaseCell::default(); CELLS];
    for atom in atoms {
        for lane in 0..3_u64 {
            let hash = stable_hash64(atom.as_bytes(), lane);
            let cell = (hash as usize) % CELLS;
            let angle =
                (mix64_golden(hash ^ 0x9e37_79b9_7f4a_7c15) as f64 / (u64::MAX as f64 + 1.0)) * TAU;
            vector[cell].re += angle.cos();
            vector[cell].im += angle.sin();
        }
    }
    vector.into_iter().map(phase_unit).collect()
}

fn phase_center_from_sum(values: &[PhaseCell]) -> Vec<PhaseCell> {
    values.iter().copied().map(phase_unit).collect()
}

fn phase_unit(value: PhaseCell) -> PhaseCell {
    let norm = (value.re * value.re + value.im * value.im).sqrt();
    if norm == 0.0 {
        PhaseCell::default()
    } else {
        PhaseCell {
            re: value.re / norm,
            im: value.im / norm,
        }
    }
}

fn phase_coherence(vector: &[PhaseCell], center: &[PhaseCell]) -> f64 {
    let mut score = 0.0;
    let mut active = 0usize;
    for (left, right) in vector.iter().zip(center) {
        if left.re != 0.0 || left.im != 0.0 {
            active += 1;
            score += left.re * right.re + left.im * right.im;
        }
    }
    if active == 0 {
        0.0
    } else {
        score / active as f64
    }
}

fn stable_hash64(bytes: &[u8], lane: u64) -> u64 {
    let hash = bytes.iter().fold(
        0xcbf2_9ce4_8422_2325_u64 ^ lane.wrapping_mul(0x1000_0000_01b3),
        |hash, byte| (hash ^ u64::from(*byte)).wrapping_mul(0x0000_0100_0000_01b3),
    );
    mix64_golden(hash)
}

fn write_cells(bytes: &mut Vec<u8>, cells: &[PhaseCell]) {
    for cell in cells {
        bytes.extend_from_slice(&cell.re.to_le_bytes());
        bytes.extend_from_slice(&cell.im.to_le_bytes());
    }
}

fn read_cells(bytes: &[u8], offset: &mut usize) -> io::Result<Vec<PhaseCell>> {
    let mut cells = Vec::with_capacity(CELLS);
    for _ in 0..CELLS {
        let re = read_f64(bytes, offset)?;
        let im = read_f64(bytes, offset)?;
        cells.push(PhaseCell { re, im });
    }
    Ok(cells)
}

fn read_f64(bytes: &[u8], offset: &mut usize) -> io::Result<f64> {
    let end = offset.saturating_add(8);
    let Some(slice) = bytes.get(*offset..end) else {
        return Err(io::Error::new(
            io::ErrorKind::UnexpectedEof,
            "truncated L2 phase package",
        ));
    };
    *offset = end;
    Ok(f64::from_le_bytes(slice.try_into().unwrap_or_default()))
}

fn generated_negative_candidates(original: &str, expected: &str) -> Vec<String> {
    let mut traps = Vec::new();
    if original != expected {
        traps.push(original.to_string());
    }
    if let Some(trap) = first_char_splice(original, expected) {
        traps.push(trap);
    }
    if let Some(trap) = last_char_splice(original, expected) {
        traps.push(trap);
    }
    traps.extend(last_char_substitution_traps(expected));
    traps.sort();
    traps.dedup();
    traps
}

fn is_trainable_pair(original: &str, expected: &str) -> bool {
    let original = original.trim();
    let expected = expected.trim();
    !original.is_empty()
        && !expected.is_empty()
        && original != expected
        && original.chars().count() <= 96
        && expected.chars().count() <= 96
        && !original.chars().any(char::is_control)
        && !expected.chars().any(char::is_control)
}

fn normalize_phrase(text: &str) -> String {
    text.trim().to_lowercase()
}

fn normalize_operation(operation: &str) -> &'static str {
    match operation {
        "layout" => "layout",
        "split" => "split",
        "typo" => "typo",
        "completion" => "completion",
        _ => "other",
    }
}

fn last_word(text: &str) -> &str {
    text.split_whitespace().last().unwrap_or(text)
}

fn word_count(text: &str) -> usize {
    text.split_whitespace().count().clamp(1, 6)
}

fn script_class(text: &str) -> &'static str {
    let has_ascii = text.chars().any(|ch| ch.is_ascii_alphabetic());
    let has_cyr = text.chars().any(crate::keyboard::is_cyrillic_letter);
    match (has_ascii, has_cyr) {
        (true, true) => "mixed",
        (true, false) => "latin",
        (false, true) => "cyr",
        _ => "other",
    }
}

fn len_bucket(len: usize) -> &'static str {
    match len {
        0..=2 => "tiny",
        3..=4 => "short",
        5..=7 => "mid",
        8..=11 => "long",
        _ => "wide",
    }
}

fn signed_bucket(value: isize) -> &'static str {
    match value {
        isize::MIN..=-4 => "minus-wide",
        -3..=-2 => "minus-mid",
        -1 => "minus-one",
        0 => "zero",
        1 => "plus-one",
        2..=3 => "plus-mid",
        _ => "plus-wide",
    }
}

fn small_bucket(value: usize) -> &'static str {
    match value {
        0 => "0",
        1 => "1",
        2 => "2",
        3..=4 => "3-4",
        _ => "5+",
    }
}

fn common_prefix_chars(left: &str, right: &str) -> usize {
    left.chars()
        .zip(right.chars())
        .take_while(|(left, right)| left == right)
        .count()
}

fn common_suffix_chars(left: &str, right: &str) -> usize {
    left.chars()
        .rev()
        .zip(right.chars().rev())
        .take_while(|(left, right)| left == right)
        .count()
}

fn push_shape_atoms(atoms: &mut Vec<String>, role: &str, word: &str) {
    let chars = word.chars().collect::<Vec<_>>();
    for (idx, window) in chars.windows(2).take(6).enumerate() {
        atoms.push(format!("{role}:bigram:{idx}:{}{}", window[0], window[1]));
    }
    for (idx, ch) in chars.iter().take(3).enumerate() {
        atoms.push(format!("{role}:head:{idx}:{ch}"));
    }
    for (idx, ch) in chars.iter().rev().take(3).enumerate() {
        atoms.push(format!("{role}:tail:{idx}:{ch}"));
    }
}

fn first_char_splice(original: &str, expected: &str) -> Option<String> {
    let first = original.chars().next()?;
    let expected_tail = expected.chars().skip(1).collect::<String>();
    let trap = format!("{first}{expected_tail}");
    (trap != expected).then_some(trap)
}

fn last_char_splice(original: &str, expected: &str) -> Option<String> {
    let last = original.chars().last()?;
    let mut trap = expected
        .chars()
        .take(expected.chars().count().saturating_sub(1))
        .collect::<String>();
    trap.push(last);
    (trap != expected).then_some(trap)
}

fn last_char_substitution_traps(expected: &str) -> Vec<String> {
    let len = expected.chars().count();
    if len < 3 {
        return Vec::new();
    }
    let stem = expected
        .chars()
        .take(len.saturating_sub(1))
        .collect::<String>();
    ['а', 'е', 'и', 'й', 'о', 'у', 'ы', 'ь', 'я', 'з']
        .into_iter()
        .filter_map(|ch| {
            let mut trap = stem.clone();
            trap.push(ch);
            (trap != expected).then_some(trap)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn l2_phase_field_admits_positive_over_splice_trap() {
        let runtime = train_phase_runtime(vec![
            L2PhaseExample {
                original: "звгрузи".to_string(),
                expected: "загрузи".to_string(),
                operation: "typo".to_string(),
                count: 4,
            },
            L2PhaseExample {
                original: "пукнт".to_string(),
                expected: "пункт".to_string(),
                operation: "typo".to_string(),
                count: 3,
            },
            L2PhaseExample {
                original: "gbitv".to_string(),
                expected: "ищем".to_string(),
                operation: "layout".to_string(),
                count: 5,
            },
        ])
        .expect("phase field trains");

        let good = runtime.admission_margin_micro("звгрузи", "загрузи", "typo");
        let trap = runtime.admission_margin_micro("звгрузи", "загрузз", "typo");
        assert!(
            runtime.admits("звгрузи", "загрузи", "typo"),
            "good={good} threshold={ADMISSION_THRESHOLD_MICRO}"
        );
        assert!(good > trap, "good={good} trap={trap}");
    }

    #[test]
    fn l2_phase_field_roundtrips_as_binary_package() {
        let runtime = train_phase_runtime(vec![L2PhaseExample {
            original: "тоесть".to_string(),
            expected: "то есть".to_string(),
            operation: "split".to_string(),
            count: 2,
        }])
        .expect("phase field trains");
        let bytes = runtime.to_bytes();
        let loaded = PhaseRuntime::from_bytes(&bytes).expect("field loads");
        assert_eq!(
            loaded.admission_margin_micro("тоесть", "то есть", "split"),
            runtime.admission_margin_micro("тоесть", "то есть", "split")
        );
    }
}
