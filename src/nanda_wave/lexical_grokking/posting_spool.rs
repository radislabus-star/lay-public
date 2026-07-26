use std::fs::{self, File, OpenOptions};
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use super::model::{AtomRecord, WaveCoupling};
use super::wave_basis::learn_atom_code_iter;

const RECORD_BYTES: usize = 12;
const DEFAULT_SHARDS: usize = 16;
static SPOOL_SEQUENCE: AtomicU64 = AtomicU64::new(0);

pub(super) struct MaterializedPostings {
    pub(super) atoms: Vec<AtomRecord>,
    pub(super) forward_couplings: Vec<WaveCoupling>,
    pub(super) maximum_strengths: Vec<u8>,
    pub(super) relations_before_policy: usize,
    pub(super) relations_dropped: usize,
    pub(super) atoms_above_policy_cap: usize,
    pub(super) max_forward_degree: usize,
}

pub(super) struct PostingSpool {
    root: PathBuf,
    writers: Vec<BufWriter<File>>,
    shard_count: usize,
    atom_count: usize,
    records: usize,
}

impl PostingSpool {
    pub(super) fn create(base: &Path, atom_count: usize) -> Result<Self, String> {
        if atom_count > u32::MAX as usize {
            return Err("L1 posting spool atom count exceeds u32".to_string());
        }
        let sequence = SPOOL_SEQUENCE.fetch_add(1, Ordering::Relaxed);
        let root = base.join(format!("postings-{}-{sequence}", std::process::id()));
        fs::create_dir_all(&root)
            .map_err(|error| format!("create L1 posting spool {}: {error}", root.display()))?;
        let shard_count = DEFAULT_SHARDS.min(atom_count.max(1));
        let mut writers = Vec::new();
        writers
            .try_reserve_exact(shard_count)
            .map_err(|error| format!("allocate L1 posting shard writers: {error}"))?;
        for shard in 0..shard_count {
            let path = root.join(format!("{shard:03}.bin"));
            let file = File::create(&path)
                .map_err(|error| format!("create L1 posting shard {}: {error}", path.display()))?;
            writers.push(BufWriter::with_capacity(64 * 1024, file));
        }
        Ok(Self {
            root,
            writers,
            shard_count,
            atom_count,
            records: 0,
        })
    }

    pub(super) fn push(&mut self, atom_id: u32, coupling: WaveCoupling) -> Result<(), String> {
        if atom_id as usize >= self.atom_count {
            return Err("L1 posting spool atom ID is out of range".to_string());
        }
        let shard = shard_for(atom_id as usize, self.atom_count, self.writers.len());
        let writer = &mut self.writers[shard];
        writer
            .write_all(&atom_id.to_le_bytes())
            .and_then(|_| writer.write_all(&coupling.peer_id.to_le_bytes()))
            .and_then(|_| writer.write_all(&[coupling.strength]))
            .and_then(|_| writer.write_all(&coupling.phase_relation.to_le_bytes()))
            .and_then(|_| writer.write_all(&[coupling.position_mode, coupling.flags]))
            .map_err(|error| format!("write L1 posting shard: {error}"))?;
        self.records = self.records.saturating_add(1);
        Ok(())
    }

    pub(super) fn merge(mut spools: Vec<Self>) -> Result<Self, String> {
        if spools.is_empty() {
            return Err("L1 posting spool merge requires at least one shard".to_string());
        }
        for spool in &mut spools {
            spool.flush_writers()?;
        }
        let mut merged = spools.remove(0);
        for mut spool in spools {
            if spool.atom_count != merged.atom_count || spool.shard_count != merged.shard_count {
                return Err("L1 posting spool shard geometry mismatch".to_string());
            }
            for shard in 0..merged.shard_count {
                let source_path = spool.root.join(format!("{shard:03}.bin"));
                let target_path = merged.root.join(format!("{shard:03}.bin"));
                let mut source = File::open(&source_path).map_err(|error| {
                    format!(
                        "open L1 posting merge source {}: {error}",
                        source_path.display()
                    )
                })?;
                let mut target =
                    OpenOptions::new()
                        .append(true)
                        .open(&target_path)
                        .map_err(|error| {
                            format!(
                                "open L1 posting merge target {}: {error}",
                                target_path.display()
                            )
                        })?;
                std::io::copy(&mut source, &mut target).map_err(|error| {
                    format!(
                        "merge L1 posting shard {} into {}: {error}",
                        source_path.display(),
                        target_path.display()
                    )
                })?;
            }
            merged.records = merged.records.saturating_add(spool.records);
            fs::remove_dir_all(&spool.root).map_err(|error| {
                format!(
                    "remove merged L1 posting spool {}: {error}",
                    spool.root.display()
                )
            })?;
            spool.root.clear();
        }
        Ok(merged)
    }

    pub(super) fn materialize(
        mut self,
        atom_support: &[u32],
        maximum_per_atom: Option<usize>,
    ) -> Result<MaterializedPostings, String> {
        if atom_support.len() != self.atom_count {
            return Err("L1 posting spool support length mismatch".to_string());
        }
        self.flush_writers()?;

        let mut atoms = vec![AtomRecord::default(); self.atom_count];
        let retained_capacity = maximum_per_atom
            .map(|limit| self.atom_count.saturating_mul(limit).min(self.records))
            .unwrap_or(self.records);
        let mut forward_couplings = Vec::new();
        let mut maximum_strengths = vec![0_u8; self.atom_count];
        forward_couplings
            .try_reserve_exact(retained_capacity)
            .map_err(|error| format!("allocate final L1 forward postings: {error}"))?;
        let mut relations_dropped = 0_usize;
        let mut atoms_above_policy_cap = 0_usize;
        let mut max_forward_degree = 0_usize;

        for shard in 0..self.shard_count() {
            let path = self.root.join(format!("{shard:03}.bin"));
            let bytes = fs::read(&path)
                .map_err(|error| format!("read L1 posting shard {}: {error}", path.display()))?;
            if bytes.len() % RECORD_BYTES != 0 {
                return Err(format!("L1 posting shard {} is truncated", path.display()));
            }
            let mut records = Vec::new();
            records
                .try_reserve_exact(bytes.len() / RECORD_BYTES)
                .map_err(|error| format!("allocate L1 posting shard records: {error}"))?;
            for record in bytes.chunks_exact(RECORD_BYTES) {
                records.push(PostingRecord {
                    atom_id: u32::from_le_bytes(record[0..4].try_into().unwrap()),
                    coupling: WaveCoupling {
                        peer_id: u32::from_le_bytes(record[4..8].try_into().unwrap()),
                        strength: record[8],
                        phase_relation: i8::from_le_bytes([record[9]]),
                        position_mode: record[10],
                        flags: record[11],
                    },
                });
            }
            drop(bytes);
            records.sort_unstable_by(|left, right| {
                left.atom_id
                    .cmp(&right.atom_id)
                    .then_with(|| coupling_order(&left.coupling, &right.coupling))
            });

            let (first_atom, end_atom) =
                shard_atom_range(shard, self.atom_count, self.shard_count());
            let mut cursor = 0_usize;
            for atom_id in first_atom..end_atom {
                let start = cursor;
                while cursor < records.len() && records[cursor].atom_id as usize == atom_id {
                    cursor += 1;
                }
                let degree = cursor - start;
                max_forward_degree = max_forward_degree.max(degree);
                if maximum_per_atom.is_some_and(|limit| degree > limit) {
                    atoms_above_policy_cap += 1;
                }
                let retained = maximum_per_atom.map_or(degree, |limit| degree.min(limit));
                let coupling_start = u32::try_from(forward_couplings.len())
                    .map_err(|_| "L1 forward coupling start exceeds u32".to_string())?;
                let retained_records = &mut records[start..start + retained];
                maximum_strengths[atom_id] = retained_records
                    .iter()
                    .map(|record| record.coupling.strength)
                    .max()
                    .unwrap_or_default();
                let wave_code =
                    learn_atom_code_iter(retained_records.iter().map(|record| record.coupling));
                // Runtime scoring is order-independent. Terminal order enables
                // exact WAND skipping during cold anti-center discovery.
                retained_records.sort_unstable_by_key(|record| record.coupling.peer_id);
                forward_couplings.extend(retained_records.iter().map(|record| record.coupling));
                atoms[atom_id] = AtomRecord {
                    wave_code,
                    coupling_start,
                    coupling_count: retained as u32,
                    support: atom_support[atom_id].min(u16::MAX as u32) as u16,
                };
                relations_dropped = relations_dropped.saturating_add(degree - retained);
            }
            if cursor != records.len() {
                return Err(format!(
                    "L1 posting shard {} contains an atom outside its range",
                    path.display()
                ));
            }
        }

        fs::remove_dir_all(&self.root).map_err(|error| {
            format!(
                "remove completed L1 posting spool {}: {error}",
                self.root.display()
            )
        })?;
        self.root.clear();
        Ok(MaterializedPostings {
            atoms,
            forward_couplings,
            maximum_strengths,
            relations_before_policy: self.records,
            relations_dropped,
            atoms_above_policy_cap,
            max_forward_degree,
        })
    }

    fn shard_count(&self) -> usize {
        self.shard_count
    }

    fn flush_writers(&mut self) -> Result<(), String> {
        for writer in &mut self.writers {
            writer
                .flush()
                .map_err(|error| format!("flush L1 posting shard: {error}"))?;
        }
        self.writers.clear();
        Ok(())
    }
}

impl Drop for PostingSpool {
    fn drop(&mut self) {
        if !self.root.as_os_str().is_empty() {
            let _ = fs::remove_dir_all(&self.root);
        }
    }
}

#[derive(Clone, Copy)]
struct PostingRecord {
    atom_id: u32,
    coupling: WaveCoupling,
}

fn shard_for(atom_id: usize, atom_count: usize, shard_count: usize) -> usize {
    atom_id
        .saturating_mul(shard_count)
        .checked_div(atom_count.max(1))
        .unwrap_or_default()
        .min(shard_count - 1)
}

fn shard_atom_range(shard: usize, atom_count: usize, shard_count: usize) -> (usize, usize) {
    (
        shard.saturating_mul(atom_count).div_ceil(shard_count),
        (shard + 1).saturating_mul(atom_count).div_ceil(shard_count),
    )
}

fn coupling_order(left: &WaveCoupling, right: &WaveCoupling) -> std::cmp::Ordering {
    (right.flags != 0)
        .cmp(&(left.flags != 0))
        .then_with(|| {
            if left.flags != 0 && right.flags != 0 {
                left.position_mode.cmp(&right.position_mode)
            } else {
                right.strength.cmp(&left.strength)
            }
        })
        .then_with(|| left.peer_id.cmp(&right.peer_id))
}
