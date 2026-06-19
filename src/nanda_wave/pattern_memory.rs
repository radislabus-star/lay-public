use super::learned::{default_memory_path, loaded_memory_entries};
use super::packet::LearnedPacketEntry;
use super::signal::WordCandidate;

pub const PATTERN_MEMORY_CELL: &str = "PatternMemoryCell32";
pub const PACKED_PATTERN_BYTES: usize = 32;
pub const PATTERN_MEMORY_ARENA_BYTES: usize = 512 * 1024;
pub const PATTERN_MEMORY_CAPACITY: usize = PATTERN_MEMORY_ARENA_BYTES / PACKED_PATTERN_BYTES;

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct PackedCorrectionPattern32 {
    pub signature: u64,
    pub context_hash: u32,
    pub candidate_hash: u32,
    pub source_hash: u32,
    pub boost_i16: i16,
    pub penalty_i16: i16,
    pub accepted: u16,
    pub rejected: u16,
    pub flags: u16,
    pub operation_code: u16,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct PatternMemoryReport {
    pub records: usize,
    pub applied: usize,
}

pub fn apply_pattern_memory(
    original: &str,
    candidates: &mut [WordCandidate],
) -> PatternMemoryReport {
    let tail = original.trim_end();
    let path = default_memory_path();
    let entries = loaded_memory_entries(&path);
    apply_pattern_memory_entries(tail, candidates, &entries)
}

pub fn apply_pattern_memory_entries(
    tail: &str,
    candidates: &mut [WordCandidate],
    entries: &[LearnedPacketEntry],
) -> PatternMemoryReport {
    let mut report = PatternMemoryReport {
        records: entries.len(),
        applied: 0,
    };
    for entry in entries.iter().filter(|entry| entry.original == tail) {
        let packed = PackedCorrectionPattern32::from_entry(entry, tail);
        for candidate in candidates.iter_mut() {
            if candidate.text.trim_end() != entry.expected {
                continue;
            }
            let delta = pattern_delta(&packed);
            if delta == 0.0 {
                continue;
            }
            candidate.energy = (candidate.energy + delta).clamp(0.0, 1.0);
            candidate.risk = (candidate.risk - delta * 0.35).clamp(0.0, 1.0);
            candidate.support.push(format!(
                "pattern-memory:signature={:016x} op={} accepted={} rejected={} delta={delta:.3}",
                packed.signature, entry.operation, packed.accepted, packed.rejected
            ));
            report.applied += 1;
        }
    }
    report
}

impl PackedCorrectionPattern32 {
    pub fn from_entry(entry: &LearnedPacketEntry, context: &str) -> Self {
        Self {
            signature: signature(&entry.original, &entry.expected),
            context_hash: hash32(context),
            candidate_hash: hash32(&entry.expected),
            source_hash: hash32("learned-memory"),
            boost_i16: quantize_weight(boost_for(entry)),
            penalty_i16: quantize_weight(penalty_for(entry)),
            accepted: entry.count.min(u16::MAX as usize) as u16,
            rejected: 0,
            flags: flags_for(entry),
            operation_code: operation_code(&entry.operation),
        }
    }
}

fn pattern_delta(pattern: &PackedCorrectionPattern32) -> f32 {
    let accepted = pattern.accepted.max(1) as f32;
    let rejected = pattern.rejected as f32;
    let trust = (accepted / (accepted + rejected + 1.0)).clamp(0.0, 1.0);
    (dequantize_weight(pattern.boost_i16) * trust).clamp(0.0, 0.18)
}

fn boost_for(entry: &LearnedPacketEntry) -> f32 {
    match entry.operation.as_str() {
        "layout" => 0.12,
        "split" => 0.10,
        "typo" => 0.08,
        _ => 0.06,
    }
}

fn penalty_for(entry: &LearnedPacketEntry) -> f32 {
    match entry.operation.as_str() {
        "layout" => 0.10,
        "split" => 0.14,
        "typo" => 0.16,
        _ => 0.18,
    }
}

fn flags_for(entry: &LearnedPacketEntry) -> u16 {
    match entry.operation.as_str() {
        "layout" => 1,
        "split" => 2,
        "typo" => 4,
        _ => 8,
    }
}

fn operation_code(operation: &str) -> u16 {
    match operation {
        "layout" => 1,
        "split" => 2,
        "typo" => 3,
        _ => 9,
    }
}

fn signature(original: &str, expected: &str) -> u64 {
    hash64(&format!("{original}\u{1f}{expected}"))
}

fn quantize_weight(value: f32) -> i16 {
    (value.clamp(-1.0, 1.0) * i16::MAX as f32).round() as i16
}

fn dequantize_weight(value: i16) -> f32 {
    value as f32 / i16::MAX as f32
}

fn hash32(text: &str) -> u32 {
    hash64(text) as u32
}

fn hash64(text: &str) -> u64 {
    text.as_bytes()
        .iter()
        .fold(0xcbf2_9ce4_8422_2325, |hash, byte| {
            (hash ^ u64::from(*byte)).wrapping_mul(0x0000_0100_0000_01b3)
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn packed_correction_pattern_is_32_bytes() {
        assert_eq!(
            std::mem::size_of::<PackedCorrectionPattern32>(),
            PACKED_PATTERN_BYTES
        );
        assert_eq!(PATTERN_MEMORY_CAPACITY, 16_384);
    }

    #[test]
    fn pattern_memory_reinforces_matching_candidate_only() {
        let entries = vec![LearnedPacketEntry {
            original: "djn".to_string(),
            expected: "вот".to_string(),
            operation: "layout".to_string(),
            count: 4,
        }];
        let mut candidates = vec![
            WordCandidate {
                text: "вот".to_string(),
                source: "LayoutWordCell32",
                energy: 0.50,
                risk: 0.20,
                support: vec![],
            },
            WordCandidate {
                text: "дом".to_string(),
                source: "PhraseCell32",
                energy: 0.80,
                risk: 0.10,
                support: vec![],
            },
        ];

        let report = apply_pattern_memory_entries("djn", &mut candidates, &entries);

        assert_eq!(report.records, 1);
        assert_eq!(report.applied, 1);
        assert!(candidates[0].energy > 0.50);
        assert!(candidates[0]
            .support
            .iter()
            .any(|item| item.starts_with("pattern-memory:")));
        assert_eq!(candidates[1].energy, 0.80);
    }
}
