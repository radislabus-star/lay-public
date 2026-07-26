use super::model::{AtomRecord, WaveCoupling};

pub(super) const TERMINAL_BLOCK_SIZE: u32 = 64;

#[derive(Clone, Copy, Debug, Default)]
struct AtomBlockRange {
    start: u32,
    count: u32,
}

#[derive(Clone, Copy, Debug)]
struct AntiPostingBlock {
    terminal_block: u32,
    payload_start: u32,
    occupancy: u64,
}

pub(super) struct AntiPostingIndex {
    atom_ranges: Vec<AtomBlockRange>,
    atom_counts: Vec<u32>,
    blocks: Vec<AntiPostingBlock>,
    payload: Vec<u16>,
}

impl AntiPostingIndex {
    pub(super) fn build(
        atoms: &[AtomRecord],
        forward_couplings: &[WaveCoupling],
    ) -> Result<Self, String> {
        let mut atom_ranges = vec![AtomBlockRange::default(); atoms.len()];
        let mut atom_counts = vec![0_u32; atoms.len()];
        let mut blocks = Vec::new();
        let mut payload = Vec::new();

        for (atom_id, atom) in atoms.iter().enumerate() {
            let start = atom.coupling_start as usize;
            let end = start.saturating_add(atom.coupling_count as usize);
            let postings = forward_couplings
                .get(start..end)
                .ok_or_else(|| "L1 anti posting atom range is invalid".to_string())?;
            let block_start = u32::try_from(blocks.len())
                .map_err(|_| "L1 anti posting block start exceeds u32".to_string())?;
            let mut cursor = 0_usize;
            while cursor < postings.len() {
                let terminal_block = postings[cursor].peer_id / TERMINAL_BLOCK_SIZE;
                let payload_start = u32::try_from(payload.len())
                    .map_err(|_| "L1 anti posting payload start exceeds u32".to_string())?;
                let mut occupancy = 0_u64;
                while cursor < postings.len()
                    && postings[cursor].peer_id / TERMINAL_BLOCK_SIZE == terminal_block
                {
                    let coupling = postings[cursor];
                    let offset = coupling.peer_id % TERMINAL_BLOCK_SIZE;
                    let bit = 1_u64 << offset;
                    if occupancy & bit != 0 {
                        return Err("L1 anti postings contain a duplicate terminal".to_string());
                    }
                    occupancy |= bit;
                    payload.push(
                        u16::from(coupling.strength) | (u16::from(coupling.position_mode) << 8),
                    );
                    cursor += 1;
                }
                blocks.push(AntiPostingBlock {
                    terminal_block,
                    payload_start,
                    occupancy,
                });
            }
            atom_ranges[atom_id] = AtomBlockRange {
                start: block_start,
                count: u32::try_from(blocks.len())
                    .map_err(|_| "L1 anti posting block count exceeds u32".to_string())?
                    .saturating_sub(block_start),
            };
            atom_counts[atom_id] = atom.coupling_count;
        }

        Ok(Self {
            atom_ranges,
            atom_counts,
            blocks,
            payload,
        })
    }

    pub(super) fn cursor(
        &self,
        atom_id: u32,
        observed_position: u8,
        weight: u8,
    ) -> AntiPostingCursor<'_> {
        let range = self
            .atom_ranges
            .get(atom_id as usize)
            .copied()
            .unwrap_or_default();
        let start = range.start as usize;
        let end = start.saturating_add(range.count as usize);
        AntiPostingCursor::new(
            self.blocks.get(start..end).unwrap_or_default(),
            &self.payload,
            self.atom_counts
                .get(atom_id as usize)
                .copied()
                .unwrap_or_default(),
            observed_position,
            weight,
        )
    }

    pub(super) fn block_count(&self) -> usize {
        self.blocks.len()
    }

    pub(super) fn payload_count(&self) -> usize {
        self.payload.len()
    }

    pub(super) fn resident_bytes(&self) -> usize {
        self.atom_ranges
            .len()
            .saturating_mul(std::mem::size_of::<AtomBlockRange>())
            .saturating_add(
                self.atom_counts
                    .len()
                    .saturating_mul(std::mem::size_of::<u32>()),
            )
            .saturating_add(
                self.blocks
                    .len()
                    .saturating_mul(std::mem::size_of::<AntiPostingBlock>()),
            )
            .saturating_add(
                self.payload
                    .len()
                    .saturating_mul(std::mem::size_of::<u16>()),
            )
    }
}

pub(super) struct AntiPostingCursor<'a> {
    blocks: &'a [AntiPostingBlock],
    payload: &'a [u16],
    block_position: usize,
    current_terminal: Option<u32>,
    posting_count: u32,
    observed_position: u8,
    weight: u8,
}

impl AntiPostingCursor<'_> {
    fn new<'a>(
        blocks: &'a [AntiPostingBlock],
        payload: &'a [u16],
        posting_count: u32,
        observed_position: u8,
        weight: u8,
    ) -> AntiPostingCursor<'a> {
        let current_terminal = blocks.first().map(|block| {
            block
                .terminal_block
                .saturating_mul(TERMINAL_BLOCK_SIZE)
                .saturating_add(block.occupancy.trailing_zeros())
        });
        AntiPostingCursor {
            blocks,
            payload,
            block_position: 0,
            current_terminal,
            posting_count,
            observed_position,
            weight,
        }
    }

    pub(super) fn current_terminal(&self) -> Option<u32> {
        self.current_terminal
    }

    pub(super) fn posting_count(&self) -> u32 {
        self.posting_count
    }

    pub(super) fn advance_to(&mut self, terminal_id: u32) -> usize {
        let Some(current) = self.current_terminal else {
            return 0;
        };
        if terminal_id <= current {
            return 0;
        }
        let target_block = terminal_id / TERMINAL_BLOCK_SIZE;
        let target_offset = terminal_id % TERMINAL_BLOCK_SIZE;
        let current_block = current / TERMINAL_BLOCK_SIZE;
        let current_offset = current % TERMINAL_BLOCK_SIZE;
        let mut skipped = 0_usize;

        while let Some(block) = self.blocks.get(self.block_position) {
            let available_start = if block.terminal_block == current_block {
                current_offset
            } else {
                0
            };
            let available = block.occupancy & mask_from(available_start);
            if block.terminal_block < target_block {
                skipped = skipped.saturating_add(available.count_ones() as usize);
                self.block_position += 1;
                continue;
            }
            let desired_offset = if block.terminal_block == target_block {
                target_offset
            } else {
                0
            };
            skipped = skipped
                .saturating_add((available & mask_before(desired_offset)).count_ones() as usize);
            let candidates = available & mask_from(desired_offset);
            if candidates != 0 {
                self.current_terminal = Some(
                    block
                        .terminal_block
                        .saturating_mul(TERMINAL_BLOCK_SIZE)
                        .saturating_add(candidates.trailing_zeros()),
                );
                return skipped;
            }
            self.block_position += 1;
        }
        self.current_terminal = None;
        skipped
    }

    pub(super) fn consume_current(&mut self) -> u32 {
        let terminal = self
            .current_terminal
            .expect("active anti posting cursor has a terminal");
        let packed = self
            .packed_for_current(terminal)
            .expect("active anti posting terminal has a payload");
        let score = score_strength_position(
            self.observed_position,
            self.weight,
            packed as u8,
            (packed >> 8) as u8,
        );
        let _ = self.advance_to(terminal.saturating_add(1));
        score
    }

    pub(super) fn block_upper_bound(&self, terminal_block: u32) -> u64 {
        let Some(current) = self.current_terminal else {
            return 0;
        };
        let Some(block) = self.blocks.get(self.block_position) else {
            return 0;
        };
        if block.terminal_block != terminal_block || current / TERMINAL_BLOCK_SIZE != terminal_block
        {
            return 0;
        }
        let current_offset = current % TERMINAL_BLOCK_SIZE;
        let first_payload = block.payload_start as usize
            + (block.occupancy & mask_before(current_offset)).count_ones() as usize;
        let payload_end = block.payload_start as usize + block.occupancy.count_ones() as usize;
        self.payload
            .get(first_payload..payload_end)
            .unwrap_or_default()
            .iter()
            .copied()
            .map(|packed| {
                u64::from(score_strength_position(
                    self.observed_position,
                    self.weight,
                    packed as u8,
                    (packed >> 8) as u8,
                ))
            })
            .max()
            .unwrap_or_default()
    }

    pub(super) fn score_terminal(&self, terminal_id: u32) -> Option<u32> {
        let packed = self.packed_for_terminal(terminal_id)?;
        Some(score_strength_position(
            self.observed_position,
            self.weight,
            packed as u8,
            (packed >> 8) as u8,
        ))
    }

    fn packed_for_terminal(&self, terminal_id: u32) -> Option<u16> {
        let terminal_block = terminal_id / TERMINAL_BLOCK_SIZE;
        let index = self
            .blocks
            .binary_search_by_key(&terminal_block, |block| block.terminal_block)
            .ok()?;
        let block = self.blocks[index];
        let offset = terminal_id % TERMINAL_BLOCK_SIZE;
        let bit = 1_u64 << offset;
        if block.occupancy & bit == 0 {
            return None;
        }
        let rank = (block.occupancy & mask_before(offset)).count_ones() as usize;
        self.payload
            .get(block.payload_start as usize + rank)
            .copied()
    }

    fn packed_for_current(&self, terminal_id: u32) -> Option<u16> {
        let block = *self.blocks.get(self.block_position)?;
        if block.terminal_block != terminal_id / TERMINAL_BLOCK_SIZE {
            return None;
        }
        let offset = terminal_id % TERMINAL_BLOCK_SIZE;
        let bit = 1_u64 << offset;
        if block.occupancy & bit == 0 {
            return None;
        }
        let rank = (block.occupancy & mask_before(offset)).count_ones() as usize;
        self.payload
            .get(block.payload_start as usize + rank)
            .copied()
    }
}

pub(super) fn score_strength_position(
    observed_position: u8,
    weight: u8,
    strength: u8,
    expected_position: u8,
) -> u32 {
    let position_factor =
        256_u32.saturating_sub(u32::from(observed_position.abs_diff(expected_position)));
    u32::from(strength)
        .saturating_mul(u32::from(weight))
        .saturating_mul(position_factor)
}

fn mask_from(offset: u32) -> u64 {
    if offset >= 64 {
        0
    } else {
        u64::MAX << offset
    }
}

fn mask_before(offset: u32) -> u64 {
    if offset == 0 {
        0
    } else if offset >= 64 {
        u64::MAX
    } else {
        (1_u64 << offset) - 1
    }
}

#[cfg(test)]
mod tests {
    use super::{score_strength_position, AntiPostingIndex};
    use crate::nanda_wave::lexical_grokking::model::{AtomRecord, WaveCoupling};

    #[test]
    fn compact_cursor_preserves_terminal_order_scores_and_skip_counts() {
        let forward = [1_u32, 5, 63, 64, 70, 130]
            .into_iter()
            .enumerate()
            .map(|(index, terminal)| WaveCoupling {
                peer_id: terminal,
                strength: 10 + index as u8,
                phase_relation: 0,
                position_mode: 20 + index as u8,
                flags: 0,
            })
            .collect::<Vec<_>>();
        let atoms = [AtomRecord {
            coupling_start: 0,
            coupling_count: forward.len() as u32,
            ..AtomRecord::default()
        }];
        let index = AntiPostingIndex::build(&atoms, &forward).unwrap();
        let mut cursor = index.cursor(0, 25, 3);

        assert_eq!(cursor.current_terminal(), Some(1));
        assert_eq!(cursor.advance_to(64), 3);
        assert_eq!(cursor.current_terminal(), Some(64));
        assert_eq!(
            cursor.consume_current(),
            score_strength_position(25, 3, 13, 23)
        );
        assert_eq!(cursor.current_terminal(), Some(70));
        assert_eq!(cursor.advance_to(129), 1);
        assert_eq!(cursor.current_terminal(), Some(130));
        assert_eq!(
            cursor.score_terminal(5),
            Some(score_strength_position(25, 3, 11, 21))
        );
        assert_eq!(cursor.advance_to(131), 1);
        assert_eq!(cursor.current_terminal(), None);
    }
}
