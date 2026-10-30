//! Read-only, bounds-checked view of the compact fragment trie. No heap graph.

use super::{Boundary, Score, Status};

const HEADER: usize = 24;
const NODE_BYTES: usize = 8;
const EDGE_BYTES: usize = 5;
const RECORD_BYTES: usize = 4;
const NO_RECORD: u16 = u16::MAX;
const KNOWN: u8 = 1;
const CASE_SENSITIVE: u8 = 2;

pub(super) struct Model<'a> {
    bytes: &'a [u8],
    edge_offset: usize,
    record_offset: usize,
    max_key: usize,
    has_pair: bool,
}

fn u16_at(bytes: &[u8], at: usize) -> Option<u16> {
    Some(u16::from_le_bytes(
        bytes.get(at..at.checked_add(2)?)?.try_into().ok()?,
    ))
}

fn u32_at(bytes: &[u8], at: usize) -> Option<usize> {
    Some(u32::from_le_bytes(bytes.get(at..at.checked_add(4)?)?.try_into().ok()?) as usize)
}

impl<'a> Model<'a> {
    pub(super) fn parse(bytes: &'a [u8]) -> Option<Self> {
        if bytes.get(..8)? != b"LAYFRG01" {
            return None;
        }
        let nodes = u32_at(bytes, 8)?;
        let edges = u32_at(bytes, 12)?;
        let records = u32_at(bytes, 16)?;
        let max_key = u32_at(bytes, 20)?;
        if nodes == 0
            || records == 0
            || records >= usize::from(NO_RECORD)
            || !(1..=128).contains(&max_key)
        {
            return None;
        }
        let edge_offset = HEADER.checked_add(nodes.checked_mul(NODE_BYTES)?)?;
        let record_offset = edge_offset.checked_add(edges.checked_mul(EDGE_BYTES)?)?;
        if record_offset.checked_add(records.checked_mul(RECORD_BYTES)?)? != bytes.len() {
            return None;
        }
        let mut consumed = 0;
        for node in 0..nodes {
            let at = HEADER + node * NODE_BYTES;
            let first = u32_at(bytes, at)?;
            let count = usize::from(u16_at(bytes, at + 4)?);
            let record = u16_at(bytes, at + 6)?;
            if first != consumed
                || first.checked_add(count)? > edges
                || (record != NO_RECORD && usize::from(record) >= records)
            {
                return None;
            }
            let mut previous = None;
            for edge in first..first + count {
                let at = edge_offset + edge * EDGE_BYTES;
                let byte = bytes[at];
                let target = u32_at(bytes, at + 1)?;
                if previous.is_some_and(|old| byte <= old) || target <= node || target >= nodes {
                    return None;
                }
                previous = Some(byte);
            }
            consumed += count;
        }
        if consumed != edges {
            return None;
        }
        let mut has_pair = false;
        for rec in 0..records {
            let at = record_offset + rec * RECORD_BYTES;
            if bytes[at + 1] & !7 != 0 {
                return None;
            }
            has_pair |= u16_at(bytes, at + 2)? == 0;
        }
        Some(Self {
            bytes,
            edge_offset,
            record_offset,
            max_key,
            has_pair,
        })
    }

    fn next(&self, node: usize, byte: u8) -> Option<usize> {
        let at = HEADER + node * NODE_BYTES;
        let first = u32_at(self.bytes, at)?;
        let mut low = 0;
        let mut high = usize::from(u16_at(self.bytes, at + 4)?);
        while low < high {
            let middle = low + (high - low) / 2;
            let edge = self.edge_offset + (first + middle) * EDGE_BYTES;
            match self.bytes[edge].cmp(&byte) {
                std::cmp::Ordering::Equal => return u32_at(self.bytes, edge + 1),
                std::cmp::Ordering::Less => low = middle + 1,
                std::cmp::Ordering::Greater => high = middle,
            }
        }
        None
    }

    fn scan(&self, data: &[u8], sensitive: bool, score: &mut Score) {
        for start in 0..data.len() {
            let mut node = 0;
            for &byte in data[start..].iter().take(self.max_key) {
                let Some(next) = self.next(node, byte) else {
                    break;
                };
                node = next;
                let Some(id) = u16_at(self.bytes, HEADER + node * NODE_BYTES + 6) else {
                    break;
                };
                if id == NO_RECORD {
                    continue;
                }
                let at = self.record_offset + usize::from(id) * RECORD_BYTES;
                let flags = self.bytes[at + 1];
                if (flags & CASE_SENSITIVE != 0) != sensitive
                    || u16_at(self.bytes, at + 2) != Some(0)
                {
                    continue;
                }
                let weight = i32::from(self.bytes[at] as i8);
                score.total = score.total.saturating_add(weight);
                score.matches += 1;
                if weight == 0 {
                    let status = if flags & KNOWN != 0 {
                        Status::Known
                    } else {
                        Status::Special
                    };
                    if status as u8 > score.status as u8 {
                        score.status = status;
                    }
                }
            }
        }
    }

    pub(super) fn score(&self, text: &str, boundary: Boundary) -> Score {
        if !self.has_pair || text.len() > 128 {
            return Score::unavailable();
        }
        let mut result = Score {
            total: 0,
            status: Status::Normal,
            matches: 0,
        };
        let wrap = |value: &str| {
            let mut bytes = Vec::with_capacity(value.len() + 2);
            bytes.push(2);
            bytes.extend_from_slice(value.as_bytes());
            if boundary == Boundary::Closed {
                bytes.push(3);
            }
            bytes
        };
        self.scan(&wrap(&text.to_lowercase()), false, &mut result);
        self.scan(&wrap(text), true, &mut result);
        result
    }
}
