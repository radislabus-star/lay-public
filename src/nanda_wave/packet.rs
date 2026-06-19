use std::fs;
use std::io;
use std::path::Path;

use super::mode::CELL32_BYTES;

const MAGIC: &[u8; 8] = b"LAYC32v1";
const HEADER_BYTES: usize = 256;
const SLOT_BYTES: usize = 256;
const SLOT_COUNT: usize = (CELL32_BYTES - HEADER_BYTES) / SLOT_BYTES;
const SLOT_PAYLOAD_BYTES: usize = SLOT_BYTES - 14;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LearnedPacketEntry {
    pub original: String,
    pub expected: String,
    pub operation: String,
    pub count: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PacketWriteReport {
    pub encoded: usize,
    pub skipped: usize,
}

pub fn write_learned_packet(
    path: &Path,
    entries: &[LearnedPacketEntry],
) -> io::Result<PacketWriteReport> {
    let (bytes, report) = encode_learned_packet(entries);
    crate::private_file::write_private_bytes(path, &bytes)?;
    Ok(report)
}

pub fn read_learned_packet(path: &Path) -> io::Result<Vec<LearnedPacketEntry>> {
    decode_learned_packet(&fs::read(path)?)
}

fn encode_learned_packet(entries: &[LearnedPacketEntry]) -> (Vec<u8>, PacketWriteReport) {
    let mut bytes = vec![0_u8; CELL32_BYTES];
    bytes[..MAGIC.len()].copy_from_slice(MAGIC);
    bytes[8..10].copy_from_slice(&(SLOT_COUNT as u16).to_le_bytes());
    bytes[10..12].copy_from_slice(&(SLOT_BYTES as u16).to_le_bytes());

    let mut encoded = 0;
    let mut skipped = 0;
    for entry in entries.iter().take(SLOT_COUNT) {
        let start = HEADER_BYTES + encoded * SLOT_BYTES;
        let end = start + SLOT_BYTES;
        if encode_slot(&mut bytes[start..end], entry) {
            encoded += 1;
        } else {
            skipped += 1;
        }
    }
    bytes[12..14].copy_from_slice(&(encoded as u16).to_le_bytes());
    skipped += entries.len().saturating_sub(SLOT_COUNT);
    (bytes, PacketWriteReport { encoded, skipped })
}

fn decode_learned_packet(bytes: &[u8]) -> io::Result<Vec<LearnedPacketEntry>> {
    if bytes.len() != CELL32_BYTES || &bytes[..MAGIC.len()] != MAGIC {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "invalid learned Cell32 packet",
        ));
    }
    let slots = u16::from_le_bytes([bytes[8], bytes[9]]) as usize;
    let slot_bytes = u16::from_le_bytes([bytes[10], bytes[11]]) as usize;
    if slots > SLOT_COUNT || slot_bytes != SLOT_BYTES {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "unsupported learned Cell32 packet layout",
        ));
    }
    let mut entries = Vec::new();
    for idx in 0..slots {
        let start = HEADER_BYTES + idx * SLOT_BYTES;
        if let Some(entry) = decode_slot(&bytes[start..start + SLOT_BYTES])? {
            entries.push(entry);
        }
    }
    Ok(entries)
}

fn encode_slot(slot: &mut [u8], entry: &LearnedPacketEntry) -> bool {
    let original = entry.original.as_bytes();
    let expected = entry.expected.as_bytes();
    if original.len() > u8::MAX as usize
        || expected.len() > u8::MAX as usize
        || original.len() + expected.len() > SLOT_PAYLOAD_BYTES
    {
        return false;
    }
    slot[0] = 1;
    slot[1] = operation_code(&entry.operation);
    slot[2..4].copy_from_slice(&(entry.count.min(u16::MAX as usize) as u16).to_le_bytes());
    slot[4..12].copy_from_slice(&hash_text(&entry.original).to_le_bytes());
    slot[12] = original.len() as u8;
    slot[13] = expected.len() as u8;
    slot[14..14 + original.len()].copy_from_slice(original);
    slot[14 + original.len()..14 + original.len() + expected.len()].copy_from_slice(expected);
    true
}

fn decode_slot(slot: &[u8]) -> io::Result<Option<LearnedPacketEntry>> {
    if slot.first().copied().unwrap_or_default() == 0 {
        return Ok(None);
    }
    let original_len = slot[12] as usize;
    let expected_len = slot[13] as usize;
    if original_len + expected_len > SLOT_PAYLOAD_BYTES {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "invalid learned Cell32 slot payload",
        ));
    }
    let original_start = 14;
    let expected_start = original_start + original_len;
    let original = String::from_utf8(slot[original_start..expected_start].to_vec())
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "invalid original utf8"))?;
    let expected = String::from_utf8(slot[expected_start..expected_start + expected_len].to_vec())
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "invalid expected utf8"))?;
    let stored_hash = u64::from_le_bytes(slot[4..12].try_into().unwrap_or_default());
    if stored_hash != hash_text(&original) {
        return Ok(None);
    }
    Ok(Some(LearnedPacketEntry {
        original,
        expected,
        operation: operation_name(slot[1]).to_string(),
        count: u16::from_le_bytes([slot[2], slot[3]]) as usize,
    }))
}

fn operation_code(operation: &str) -> u8 {
    match operation {
        "layout" => 1,
        "split" => 2,
        "typo" => 3,
        _ => 9,
    }
}

fn operation_name(code: u8) -> &'static str {
    match code {
        1 => "layout",
        2 => "split",
        3 => "typo",
        _ => "other",
    }
}

fn hash_text(text: &str) -> u64 {
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
    fn learned_packet_is_exactly_cell32_sized() {
        let (bytes, report) = encode_learned_packet(&[LearnedPacketEntry {
            original: "djn".to_string(),
            expected: "вот".to_string(),
            operation: "layout".to_string(),
            count: 1,
        }]);
        assert_eq!(bytes.len(), CELL32_BYTES);
        assert_eq!(report.encoded, 1);
        let decoded = decode_learned_packet(&bytes).unwrap();
        assert_eq!(decoded[0].expected, "вот");
    }
}
