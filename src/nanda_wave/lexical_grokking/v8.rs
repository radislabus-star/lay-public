//! Lossless lazy L1.1 package.
//!
//! V8 keeps the compact V7 crystal as its immutable base and adds a complete,
//! independently addressable forward-posting section. Runtime code can mmap
//! the package and decode only postings touched by the current surface.

use std::collections::{HashMap, VecDeque};
use std::fs::{self, File};
use std::io;
use std::path::Path;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex, OnceLock};

use super::model::{LexicalGrokkingPackage, WaveCoupling};
use super::{format, posting_codec};
use rayon::prelude::*;

const MAGIC: [u8; 8] = *b"LAYL1V8\0";
const VERSION: u32 = 8;
const HEADER_BYTES: usize = 128;
const INDEX_ENTRY_BYTES: usize = 16;
const SHARD_ENTRY_BYTES: usize = 16;
const DEFAULT_ATOMS_PER_SHARD: usize = 1;
const MAX_ATOMS_PER_SHARD: usize = 4_096;
const CHECKSUM_OFFSET: usize = 88;
const DEFAULT_POSTING_CACHE_MIB: usize = 32;
const DEFAULT_SHARD_CACHE_MIB: usize = 0;
const MAX_RUNTIME_CACHE_MIB: usize = 128;
const MAX_PREFETCH_WORKERS: usize = 32;

pub(super) fn runtime_pool_install<OP, R>(operation: OP) -> R
where
    OP: FnOnce() -> R + Send,
    R: Send,
{
    static POOL: OnceLock<rayon::ThreadPool> = OnceLock::new();
    POOL.get_or_init(|| {
        let workers = std::env::var("LAY_L11_RUNTIME_WORKERS")
            .ok()
            .and_then(|value| value.parse::<usize>().ok())
            .unwrap_or_else(|| {
                std::thread::available_parallelism()
                    .map(usize::from)
                    .unwrap_or(1)
                    .saturating_sub(1)
                    .max(1)
                    .min(14)
            })
            .clamp(1, MAX_PREFETCH_WORKERS);
        rayon::ThreadPoolBuilder::new()
            .num_threads(workers)
            .thread_name(|index| format!("lay-l11-v8-{index}"))
            .build()
            .expect("build L1.1 V8 runtime worker pool")
    })
    .install(operation)
}

fn runtime_cache_bytes(variable: &str, default_mib: usize) -> usize {
    std::env::var(variable)
        .ok()
        .and_then(|value| value.parse::<usize>().ok())
        .unwrap_or(default_mib)
        .min(MAX_RUNTIME_CACHE_MIB)
        .saturating_mul(1024 * 1024)
}

fn posting_cache_bytes() -> usize {
    static BYTES: OnceLock<usize> = OnceLock::new();
    *BYTES.get_or_init(|| {
        runtime_cache_bytes("LAY_L11_V8_POSTING_CACHE_MIB", DEFAULT_POSTING_CACHE_MIB)
    })
}

fn shard_cache_bytes() -> usize {
    static BYTES: OnceLock<usize> = OnceLock::new();
    *BYTES
        .get_or_init(|| runtime_cache_bytes("LAY_L11_V8_SHARD_CACHE_MIB", DEFAULT_SHARD_CACHE_MIB))
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) struct V8PostingIndex {
    pub(super) shard_id: u32,
    pub(super) offset: u32,
    pub(super) byte_len: u32,
    pub(super) relation_count: u32,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct V8ShardIndex {
    offset: u64,
    compressed_len: u32,
    raw_len: u32,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) struct V8Header {
    pub(super) file_bytes: u64,
    pub(super) base_offset: u64,
    pub(super) base_bytes: u64,
    pub(super) index_offset: u64,
    pub(super) index_count: u32,
    pub(super) forward_relations: u64,
    pub(super) postings_offset: u64,
    pub(super) postings_bytes: u64,
    pub(super) corpus_hash: u64,
    pub(super) checksum: u64,
    pub(super) reverse_relations: u64,
    pub(super) shard_index_offset: u64,
    pub(super) shard_count: u32,
}

pub(super) struct V8Artifact {
    bytes: ArtifactBytes,
    header: V8Header,
    posting_cache: Mutex<PostingCache>,
    shard_cache: Mutex<ShardCache>,
    shard_locks: Vec<Mutex<()>>,
}

#[derive(Default)]
struct PostingCache {
    bytes: usize,
    order: VecDeque<u32>,
    entries: HashMap<u32, Arc<[WaveCoupling]>>,
}

#[derive(Default)]
struct ShardCache {
    bytes: usize,
    order: VecDeque<u32>,
    entries: HashMap<u32, Arc<[u8]>>,
}

impl V8Artifact {
    pub(super) fn load(path: &Path) -> Result<Self, String> {
        validate_file_checksum(path)?;
        let bytes = ArtifactBytes::load(path)?;
        let header = read_header_inner(bytes.as_slice(), false)?;
        let shard_locks = (0..header.shard_count).map(|_| Mutex::new(())).collect();
        Ok(Self {
            bytes,
            header,
            posting_cache: Mutex::new(PostingCache::default()),
            shard_cache: Mutex::new(ShardCache::default()),
            shard_locks,
        })
    }

    pub(super) fn decode_base(&self) -> Result<LexicalGrokkingPackage, String> {
        let package = format::decode_compact_base(base_bytes(self.bytes.as_slice(), self.header)?)?;
        self.bytes.discard_resident_pages();
        Ok(package)
    }

    pub(super) fn forward_relation_count(&self) -> usize {
        self.header.forward_relations as usize
    }

    pub(super) fn reverse_relation_count(&self) -> usize {
        self.header.reverse_relations as usize
    }

    pub(super) fn posting_degree(&self, atom_id: u32) -> usize {
        read_index(self.bytes.as_slice(), self.header, atom_id)
            .map(|item| item.relation_count as usize)
            .unwrap_or_default()
    }

    pub(super) fn posting(&self, atom_id: u32) -> Result<Arc<[WaveCoupling]>, String> {
        let cache_budget = posting_cache_bytes();
        if cache_budget != 0 {
            if let Ok(cache) = self.posting_cache.lock() {
                if let Some(posting) = cache.entries.get(&atom_id) {
                    return Ok(Arc::clone(posting));
                }
            }
        }
        let posting = self.decode_posting_uncached(atom_id)?;
        self.admit_posting(atom_id, posting, cache_budget)
    }

    fn decode_posting_uncached(&self, atom_id: u32) -> Result<Arc<[WaveCoupling]>, String> {
        let item = read_index(self.bytes.as_slice(), self.header, atom_id)?;
        let shard = self.shard(item.shard_id)?;
        let start = item.offset as usize;
        let end = start.saturating_add(item.byte_len as usize);
        let decoded = posting_codec::decode_posting(
            shard
                .get(start..end)
                .ok_or_else(|| "invalid V8 posting range inside shard".to_string())?,
            item.relation_count as usize,
        )?;
        Ok(decoded.into())
    }

    fn admit_posting(
        &self,
        atom_id: u32,
        posting: Arc<[WaveCoupling]>,
        cache_budget: usize,
    ) -> Result<Arc<[WaveCoupling]>, String> {
        let posting_bytes = posting
            .len()
            .saturating_mul(std::mem::size_of::<WaveCoupling>());
        if posting_bytes <= cache_budget {
            let mut cache = self
                .posting_cache
                .lock()
                .map_err(|_| "V8 posting cache is poisoned".to_string())?;
            if let Some(existing) = cache.entries.get(&atom_id) {
                return Ok(Arc::clone(existing));
            }
            while cache.bytes.saturating_add(posting_bytes) > cache_budget {
                let Some(evicted_id) = cache.order.pop_front() else {
                    break;
                };
                let Some(evicted) = cache.entries.remove(&evicted_id) else {
                    continue;
                };
                cache.bytes = cache.bytes.saturating_sub(
                    evicted
                        .len()
                        .saturating_mul(std::mem::size_of::<WaveCoupling>()),
                );
            }
            cache.bytes = cache.bytes.saturating_add(posting_bytes);
            cache.order.push_back(atom_id);
            cache.entries.insert(atom_id, Arc::clone(&posting));
        }
        Ok(posting)
    }

    pub(super) fn postings(&self, atom_ids: &[u32]) -> Result<Vec<Arc<[WaveCoupling]>>, String> {
        if atom_ids.len() <= 1 {
            return atom_ids
                .iter()
                .map(|atom_id| self.posting(*atom_id))
                .collect();
        }
        let cache_budget = posting_cache_bytes();
        let mut resolved = vec![None; atom_ids.len()];
        if cache_budget != 0 {
            let cache = self
                .posting_cache
                .lock()
                .map_err(|_| "V8 posting cache is poisoned".to_string())?;
            for (index, atom_id) in atom_ids.iter().copied().enumerate() {
                if let Some(posting) = cache.entries.get(&atom_id) {
                    resolved[index] = Some(Arc::clone(posting));
                }
            }
        }
        let missing = atom_ids
            .iter()
            .copied()
            .enumerate()
            .filter(|(index, _)| resolved[*index].is_none())
            .collect::<Vec<_>>();
        let decoded = runtime_pool_install(|| {
            missing
                .par_iter()
                .map(|(index, atom_id)| {
                    self.decode_posting_uncached(*atom_id)
                        .map(|posting| (*index, *atom_id, posting))
                })
                .collect::<Result<Vec<_>, _>>()
        })?;
        if cache_budget == 0 {
            for (index, _, posting) in decoded {
                resolved[index] = Some(posting);
            }
        } else {
            let mut cache = self
                .posting_cache
                .lock()
                .map_err(|_| "V8 posting cache is poisoned".to_string())?;
            for (index, atom_id, posting) in decoded {
                if let Some(existing) = cache.entries.get(&atom_id) {
                    resolved[index] = Some(Arc::clone(existing));
                    continue;
                }
                let posting_bytes = posting
                    .len()
                    .saturating_mul(std::mem::size_of::<WaveCoupling>());
                if posting_bytes <= cache_budget {
                    while cache.bytes.saturating_add(posting_bytes) > cache_budget {
                        let Some(evicted_id) = cache.order.pop_front() else {
                            break;
                        };
                        let Some(evicted) = cache.entries.remove(&evicted_id) else {
                            continue;
                        };
                        cache.bytes = cache.bytes.saturating_sub(
                            evicted
                                .len()
                                .saturating_mul(std::mem::size_of::<WaveCoupling>()),
                        );
                    }
                    cache.bytes = cache.bytes.saturating_add(posting_bytes);
                    cache.order.push_back(atom_id);
                    cache.entries.insert(atom_id, Arc::clone(&posting));
                }
                resolved[index] = Some(posting);
            }
        }
        resolved
            .into_iter()
            .map(|posting| posting.ok_or_else(|| "V8 posting batch is incomplete".to_string()))
            .collect()
    }

    fn shard(&self, shard_id: u32) -> Result<Arc<[u8]>, String> {
        let cache_budget = shard_cache_bytes();
        if cache_budget != 0 {
            if let Ok(cache) = self.shard_cache.lock() {
                if let Some(shard) = cache.entries.get(&shard_id) {
                    return Ok(Arc::clone(shard));
                }
            }
        }
        let _shard_guard = self
            .shard_locks
            .get(shard_id as usize)
            .ok_or_else(|| "V8 shard ID exceeds lock table".to_string())?
            .lock()
            .map_err(|_| "V8 shard lock is poisoned".to_string())?;
        if cache_budget != 0 {
            if let Ok(cache) = self.shard_cache.lock() {
                if let Some(shard) = cache.entries.get(&shard_id) {
                    return Ok(Arc::clone(shard));
                }
            }
        }
        let item = read_shard_index(self.bytes.as_slice(), self.header, shard_id)?;
        let compressed = compressed_shard_bytes(self.bytes.as_slice(), self.header, item)?;
        let decoded = zstd::bulk::decompress(compressed, item.raw_len as usize)
            .map_err(|error| format!("V8 shard decompression failed: {error}"))?;
        if decoded.len() != item.raw_len as usize {
            return Err("V8 shard raw length mismatch".to_string());
        }
        let shard: Arc<[u8]> = decoded.into();
        if shard.len() <= cache_budget {
            let mut cache = self
                .shard_cache
                .lock()
                .map_err(|_| "V8 shard cache is poisoned".to_string())?;
            while cache.bytes.saturating_add(shard.len()) > cache_budget {
                let Some(evicted_id) = cache.order.pop_front() else {
                    break;
                };
                let Some(evicted) = cache.entries.remove(&evicted_id) else {
                    continue;
                };
                cache.bytes = cache.bytes.saturating_sub(evicted.len());
            }
            cache.bytes = cache.bytes.saturating_add(shard.len());
            cache.order.push_back(shard_id);
            cache.entries.insert(shard_id, Arc::clone(&shard));
        }
        Ok(shard)
    }
}

pub(super) fn is_v8(bytes: &[u8]) -> bool {
    bytes.get(..MAGIC.len()) == Some(MAGIC.as_slice())
}

pub fn build_lazy_v8_package(input: &Path, output: &Path) -> io::Result<serde_json::Value> {
    build_lazy_v8_package_with_shard_size(input, output, DEFAULT_ATOMS_PER_SHARD)
}

pub fn build_lazy_v8_package_with_shard_size(
    input: &Path,
    output: &Path,
    atoms_per_shard: usize,
) -> io::Result<serde_json::Value> {
    if !(1..=MAX_ATOMS_PER_SHARD).contains(&atoms_per_shard) {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("atoms per V8 shard must be in 1..={MAX_ATOMS_PER_SHARD}"),
        ));
    }
    let base = fs::read(input)?;
    let package = format::decode(&base).map_err(io::Error::other)?;
    let mut index = Vec::with_capacity(package.atoms.len());
    let mut postings = Vec::new();
    let mut shards = Vec::new();
    for (shard_id, built) in build_shards_parallel(&package, atoms_per_shard)?
        .into_iter()
        .enumerate()
    {
        index.extend(built.index);
        shards.push(V8ShardIndex {
            offset: postings.len() as u64,
            compressed_len: u32::try_from(built.compressed.len())
                .map_err(|_| io::Error::other("V8 compressed shard exceeds u32 bytes"))?,
            raw_len: built.raw_len,
        });
        debug_assert!(index
            .iter()
            .rev()
            .take(package.atoms.len().min(atoms_per_shard))
            .all(|item| item.shard_id <= shard_id as u32));
        postings.extend_from_slice(&built.compressed);
    }

    let base_offset = HEADER_BYTES;
    let index_offset = align8(base_offset.saturating_add(base.len()));
    let shard_index_offset = align8(index_offset.saturating_add(index.len() * INDEX_ENTRY_BYTES));
    let postings_offset =
        align8(shard_index_offset.saturating_add(shards.len() * SHARD_ENTRY_BYTES));
    let file_bytes = postings_offset.saturating_add(postings.len());
    let mut bytes = vec![0_u8; file_bytes];
    bytes[base_offset..base_offset + base.len()].copy_from_slice(&base);
    for (position, item) in index.iter().copied().enumerate() {
        write_index(
            &mut bytes,
            index_offset + position * INDEX_ENTRY_BYTES,
            item,
        );
    }
    for (position, item) in shards.iter().copied().enumerate() {
        write_shard_index(
            &mut bytes,
            shard_index_offset + position * SHARD_ENTRY_BYTES,
            item,
        );
    }
    bytes[postings_offset..].copy_from_slice(&postings);

    bytes[..8].copy_from_slice(&MAGIC);
    put_u32(&mut bytes, 8, VERSION);
    put_u32(&mut bytes, 12, HEADER_BYTES as u32);
    put_u64(&mut bytes, 16, file_bytes as u64);
    put_u64(&mut bytes, 24, base_offset as u64);
    put_u64(&mut bytes, 32, base.len() as u64);
    put_u64(&mut bytes, 40, index_offset as u64);
    put_u32(&mut bytes, 48, index.len() as u32);
    put_u64(&mut bytes, 56, package.forward_couplings.len() as u64);
    put_u64(&mut bytes, 64, postings_offset as u64);
    put_u64(&mut bytes, 72, postings.len() as u64);
    put_u64(&mut bytes, 80, package.corpus_hash);
    put_u64(&mut bytes, 96, package.reverse_couplings.len() as u64);
    put_u64(&mut bytes, 104, shard_index_offset as u64);
    put_u32(&mut bytes, 112, shards.len() as u32);
    let package_checksum = checksum(&bytes);
    put_u64(&mut bytes, CHECKSUM_OFFSET, package_checksum);

    if let Some(parent) = output.parent() {
        fs::create_dir_all(parent)?;
    }
    let temporary = output.with_extension("tmp");
    fs::write(&temporary, &bytes)?;
    fs::rename(&temporary, output)?;

    Ok(serde_json::json!({
        "format": "V8 lazy-sharded-postings",
        "input": input,
        "output": output,
        "terminal_count": package.terminal_count(),
        "atom_count": package.atoms.len(),
        "forward_relations": package.forward_couplings.len(),
        "base_bytes": base.len(),
        "posting_index_bytes": index.len() * INDEX_ENTRY_BYTES,
        "atoms_per_shard": atoms_per_shard,
        "posting_shards": shards.len(),
        "posting_shard_index_bytes": shards.len() * SHARD_ENTRY_BYTES,
        "compressed_posting_bytes": postings.len(),
        "output_bytes": bytes.len(),
        "average_posting_bytes_per_relation": if package.forward_couplings.is_empty() {
            0.0
        } else {
            postings.len() as f64 / package.forward_couplings.len() as f64
        },
    }))
}

struct BuiltShard {
    index: Vec<V8PostingIndex>,
    compressed: Vec<u8>,
    raw_len: u32,
}

fn build_shards_parallel(
    package: &LexicalGrokkingPackage,
    atoms_per_shard: usize,
) -> io::Result<Vec<BuiltShard>> {
    let shard_count = package.atoms.len().div_ceil(atoms_per_shard);
    let workers = std::env::var("LAY_L11_BUILD_WORKERS")
        .ok()
        .and_then(|value| value.parse::<usize>().ok())
        .unwrap_or_else(|| {
            std::thread::available_parallelism()
                .map(usize::from)
                .unwrap_or(1)
                .saturating_sub(1)
                .max(1)
        })
        .clamp(1, MAX_PREFETCH_WORKERS)
        .min(shard_count.max(1));
    let next = AtomicUsize::new(0);
    let partial = std::thread::scope(|scope| {
        (0..workers)
            .map(|_| {
                scope.spawn(|| {
                    let mut built = Vec::new();
                    loop {
                        let shard_id = next.fetch_add(1, Ordering::Relaxed);
                        if shard_id >= shard_count {
                            break;
                        }
                        built.push((shard_id, build_shard(package, shard_id, atoms_per_shard)?));
                    }
                    Ok::<_, io::Error>(built)
                })
            })
            .collect::<Vec<_>>()
            .into_iter()
            .map(|handle| {
                handle
                    .join()
                    .map_err(|_| io::Error::other("V8 shard worker panicked"))?
            })
            .collect::<io::Result<Vec<_>>>()
    })?;
    let mut built = partial.into_iter().flatten().collect::<Vec<_>>();
    built.sort_unstable_by_key(|(shard_id, _)| *shard_id);
    if built.len() != shard_count {
        return Err(io::Error::other("V8 shard build is incomplete"));
    }
    Ok(built.into_iter().map(|(_, shard)| shard).collect())
}

fn build_shard(
    package: &LexicalGrokkingPackage,
    shard_id: usize,
    atoms_per_shard: usize,
) -> io::Result<BuiltShard> {
    let atom_start = shard_id.saturating_mul(atoms_per_shard);
    let atom_end = atom_start
        .saturating_add(atoms_per_shard)
        .min(package.atoms.len());
    let mut raw = Vec::new();
    let mut index = Vec::with_capacity(atom_end.saturating_sub(atom_start));
    for atom in &package.atoms[atom_start..atom_end] {
        let start = atom.coupling_start as usize;
        let end = start.saturating_add(atom.coupling_count as usize);
        let relations = package
            .forward_couplings
            .get(start..end)
            .ok_or_else(|| io::Error::other("invalid V8 source posting range"))?;
        let encoded = posting_codec::encode_posting(relations).map_err(io::Error::other)?;
        index.push(V8PostingIndex {
            shard_id: shard_id as u32,
            offset: u32::try_from(raw.len())
                .map_err(|_| io::Error::other("V8 raw shard exceeds u32 bytes"))?,
            byte_len: u32::try_from(encoded.bytes.len())
                .map_err(|_| io::Error::other("V8 posting exceeds u32 bytes"))?,
            relation_count: atom.coupling_count,
        });
        raw.extend_from_slice(&encoded.bytes);
    }
    let raw_len =
        u32::try_from(raw.len()).map_err(|_| io::Error::other("V8 raw shard exceeds u32 bytes"))?;
    let compressed = zstd::bulk::compress(&raw, 19)
        .map_err(|error| io::Error::other(format!("V8 shard compression failed: {error}")))?;
    Ok(BuiltShard {
        index,
        compressed,
        raw_len,
    })
}

pub(super) fn read_header(bytes: &[u8]) -> Result<V8Header, String> {
    read_header_inner(bytes, true)
}

fn read_header_inner(bytes: &[u8], verify_checksum: bool) -> Result<V8Header, String> {
    if bytes.get(..8) != Some(MAGIC.as_slice())
        || read_u32(bytes, 8)? != VERSION
        || read_u32(bytes, 12)? as usize != HEADER_BYTES
    {
        return Err("invalid L1.1 V8 header".to_string());
    }
    let header = V8Header {
        file_bytes: read_u64(bytes, 16)?,
        base_offset: read_u64(bytes, 24)?,
        base_bytes: read_u64(bytes, 32)?,
        index_offset: read_u64(bytes, 40)?,
        index_count: read_u32(bytes, 48)?,
        forward_relations: read_u64(bytes, 56)?,
        postings_offset: read_u64(bytes, 64)?,
        postings_bytes: read_u64(bytes, 72)?,
        corpus_hash: read_u64(bytes, 80)?,
        checksum: read_u64(bytes, CHECKSUM_OFFSET)?,
        reverse_relations: read_u64(bytes, 96)?,
        shard_index_offset: read_u64(bytes, 104)?,
        shard_count: read_u32(bytes, 112)?,
    };
    if header.file_bytes as usize != bytes.len()
        || (verify_checksum && header.checksum != checksum(bytes))
        || checked_range(bytes, header.base_offset, header.base_bytes).is_none()
        || checked_range(
            bytes,
            header.index_offset,
            u64::from(header.index_count) * INDEX_ENTRY_BYTES as u64,
        )
        .is_none()
        || checked_range(
            bytes,
            header.shard_index_offset,
            u64::from(header.shard_count) * SHARD_ENTRY_BYTES as u64,
        )
        .is_none()
        || checked_range(bytes, header.postings_offset, header.postings_bytes).is_none()
    {
        return Err("invalid L1.1 V8 ranges or checksum".to_string());
    }
    Ok(header)
}

fn validate_file_checksum(path: &Path) -> Result<(), String> {
    use std::io::Read;

    let expected = {
        let mut file = File::open(path).map_err(|error| format!("{}: {error}", path.display()))?;
        let mut header = [0_u8; HEADER_BYTES];
        file.read_exact(&mut header)
            .map_err(|error| format!("{}: {error}", path.display()))?;
        read_u64(&header, CHECKSUM_OFFSET)?
    };
    let mut file = File::open(path).map_err(|error| format!("{}: {error}", path.display()))?;
    let mut buffer = vec![0_u8; 1024 * 1024];
    let mut state = 0xcbf2_9ce4_8422_2325_u64;
    let mut global_offset = 0_usize;
    loop {
        let count = file
            .read(&mut buffer)
            .map_err(|error| format!("{}: {error}", path.display()))?;
        if count == 0 {
            break;
        }
        for (local_offset, byte) in buffer[..count].iter().copied().enumerate() {
            let index = global_offset.saturating_add(local_offset);
            let value = if (CHECKSUM_OFFSET..CHECKSUM_OFFSET + 8).contains(&index) {
                0
            } else {
                byte
            };
            state = (state ^ u64::from(value)).wrapping_mul(0x100_0000_01b3);
        }
        global_offset = global_offset.saturating_add(count);
    }
    if state != expected {
        return Err("invalid L1.1 V8 checksum".to_string());
    }
    Ok(())
}

pub(super) fn read_index(
    bytes: &[u8],
    header: V8Header,
    atom_id: u32,
) -> Result<V8PostingIndex, String> {
    if atom_id >= header.index_count {
        return Err("V8 atom ID exceeds posting index".to_string());
    }
    let start = header.index_offset as usize + atom_id as usize * INDEX_ENTRY_BYTES;
    Ok(V8PostingIndex {
        shard_id: read_u32(bytes, start)?,
        offset: read_u32(bytes, start + 4)?,
        byte_len: read_u32(bytes, start + 8)?,
        relation_count: read_u32(bytes, start + 12)?,
    })
}

pub(super) fn base_bytes(bytes: &[u8], header: V8Header) -> Result<&[u8], String> {
    checked_range(bytes, header.base_offset, header.base_bytes)
        .ok_or_else(|| "invalid V8 base range".to_string())
}

fn read_shard_index(bytes: &[u8], header: V8Header, shard_id: u32) -> Result<V8ShardIndex, String> {
    if shard_id >= header.shard_count {
        return Err("V8 shard ID exceeds shard index".to_string());
    }
    let start = header.shard_index_offset as usize + shard_id as usize * SHARD_ENTRY_BYTES;
    Ok(V8ShardIndex {
        offset: read_u64(bytes, start)?,
        compressed_len: read_u32(bytes, start + 8)?,
        raw_len: read_u32(bytes, start + 12)?,
    })
}

fn compressed_shard_bytes(
    bytes: &[u8],
    header: V8Header,
    item: V8ShardIndex,
) -> Result<&[u8], String> {
    let offset = header
        .postings_offset
        .checked_add(item.offset)
        .ok_or_else(|| "V8 shard offset overflow".to_string())?;
    checked_range(bytes, offset, u64::from(item.compressed_len))
        .ok_or_else(|| "invalid V8 compressed shard range".to_string())
}

fn write_index(bytes: &mut [u8], offset: usize, item: V8PostingIndex) {
    put_u32(bytes, offset, item.shard_id);
    put_u32(bytes, offset + 4, item.offset);
    put_u32(bytes, offset + 8, item.byte_len);
    put_u32(bytes, offset + 12, item.relation_count);
}

fn write_shard_index(bytes: &mut [u8], offset: usize, item: V8ShardIndex) {
    put_u64(bytes, offset, item.offset);
    put_u32(bytes, offset + 8, item.compressed_len);
    put_u32(bytes, offset + 12, item.raw_len);
}

fn checked_range(bytes: &[u8], offset: u64, len: u64) -> Option<&[u8]> {
    let start = usize::try_from(offset).ok()?;
    let end = start.checked_add(usize::try_from(len).ok()?)?;
    bytes.get(start..end)
}

fn align8(value: usize) -> usize {
    value.saturating_add(7) & !7
}

fn checksum(bytes: &[u8]) -> u64 {
    bytes
        .iter()
        .enumerate()
        .fold(0xcbf2_9ce4_8422_2325_u64, |state, (index, byte)| {
            let value = if (CHECKSUM_OFFSET..CHECKSUM_OFFSET + 8).contains(&index) {
                0
            } else {
                *byte
            };
            (state ^ u64::from(value)).wrapping_mul(0x100_0000_01b3)
        })
}

fn put_u32(bytes: &mut [u8], offset: usize, value: u32) {
    bytes[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
}

fn put_u64(bytes: &mut [u8], offset: usize, value: u64) {
    bytes[offset..offset + 8].copy_from_slice(&value.to_le_bytes());
}

fn read_u32(bytes: &[u8], offset: usize) -> Result<u32, String> {
    let raw = bytes
        .get(offset..offset + 4)
        .ok_or_else(|| "truncated V8 u32".to_string())?;
    Ok(u32::from_le_bytes([raw[0], raw[1], raw[2], raw[3]]))
}

fn read_u64(bytes: &[u8], offset: usize) -> Result<u64, String> {
    let raw = bytes
        .get(offset..offset + 8)
        .ok_or_else(|| "truncated V8 u64".to_string())?;
    Ok(u64::from_le_bytes([
        raw[0], raw[1], raw[2], raw[3], raw[4], raw[5], raw[6], raw[7],
    ]))
}

enum ArtifactBytes {
    #[cfg(target_os = "linux")]
    Mapped(MappedFile),
    #[cfg(not(target_os = "linux"))]
    Owned(Box<[u8]>),
}

impl ArtifactBytes {
    fn load(path: &Path) -> Result<Self, String> {
        #[cfg(target_os = "linux")]
        {
            MappedFile::open(path).map(Self::Mapped)
        }
        #[cfg(not(target_os = "linux"))]
        {
            fs::read(path)
                .map(Vec::into_boxed_slice)
                .map(Self::Owned)
                .map_err(|error| format!("{}: {error}", path.display()))
        }
    }

    fn as_slice(&self) -> &[u8] {
        match self {
            #[cfg(target_os = "linux")]
            Self::Mapped(mapped) => mapped.as_slice(),
            #[cfg(not(target_os = "linux"))]
            Self::Owned(bytes) => bytes,
        }
    }

    fn discard_resident_pages(&self) {
        #[cfg(target_os = "linux")]
        {
            let Self::Mapped(mapped) = self;
            mapped.discard_resident_pages();
        }
    }
}

#[cfg(target_os = "linux")]
struct MappedFile {
    ptr: *mut libc::c_void,
    len: usize,
}

#[cfg(target_os = "linux")]
impl MappedFile {
    fn open(path: &Path) -> Result<Self, String> {
        use std::os::fd::AsRawFd;

        let file = File::open(path).map_err(|error| format!("{}: {error}", path.display()))?;
        let len = file
            .metadata()
            .map_err(|error| format!("{}: {error}", path.display()))?
            .len() as usize;
        if len == 0 {
            return Err(format!("{}: empty L1.1 V8 artifact", path.display()));
        }
        let ptr = unsafe {
            libc::mmap(
                std::ptr::null_mut(),
                len,
                libc::PROT_READ,
                libc::MAP_PRIVATE,
                file.as_raw_fd(),
                0,
            )
        };
        if ptr == libc::MAP_FAILED {
            return Err(format!(
                "{}: mmap failed: {}",
                path.display(),
                io::Error::last_os_error()
            ));
        }
        Ok(Self { ptr, len })
    }

    fn as_slice(&self) -> &[u8] {
        unsafe { std::slice::from_raw_parts(self.ptr.cast::<u8>(), self.len) }
    }

    fn discard_resident_pages(&self) {
        unsafe {
            libc::madvise(self.ptr, self.len, libc::MADV_DONTNEED);
        }
    }
}

#[cfg(target_os = "linux")]
unsafe impl Send for MappedFile {}
#[cfg(target_os = "linux")]
unsafe impl Sync for MappedFile {}

#[cfg(target_os = "linux")]
impl Drop for MappedFile {
    fn drop(&mut self) {
        unsafe {
            libc::munmap(self.ptr, self.len);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::nanda_wave::lexical_grokking::compiler::{
        compile_with_policy, ForwardPostingPolicy, TrainingWord,
    };
    use crate::nanda_wave::lexical_grokking::runtime::{LexicalGrokkingMemory, ReadoutMode};

    #[test]
    fn v8_keeps_every_forward_relation_addressable() {
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
        let v7 = format::encode_compact_depth0(&package).expect("encode V7 fixture");
        let directory = std::env::temp_dir().join(format!("lay-v8-{}", std::process::id()));
        fs::create_dir_all(&directory).expect("create V8 test directory");
        let input = directory.join("fixture.v7.bin");
        let output = directory.join("fixture.v8.bin");
        fs::write(&input, &v7).expect("write V7 fixture");
        build_lazy_v8_package(&input, &output).expect("build V8 fixture");

        let bytes = fs::read(&output).expect("read V8 fixture");
        let header = read_header(&bytes).expect("read V8 header");
        let base = format::decode_compact_base(base_bytes(&bytes, header).expect("V8 base"))
            .expect("decode compact V8 base");
        assert_eq!(base.centers.len(), package.centers.len());
        assert!(base.forward_couplings.is_empty());
        let artifact = V8Artifact::load(&output).expect("load mmap V8 fixture");
        let atom_ids = (0..header.index_count).collect::<Vec<_>>();
        artifact.postings(&atom_ids).expect("load V8 postings");
        let mut decoded_relations = 0_usize;
        for atom_id in 0..header.index_count {
            let item = read_index(&bytes, header, atom_id).expect("read V8 index");
            let decoded = artifact.posting(atom_id).expect("decode V8 posting");
            let source = &package.forward_couplings[package.atoms[atom_id as usize].coupling_start
                as usize
                ..package.atoms[atom_id as usize].coupling_start as usize
                    + package.atoms[atom_id as usize].coupling_count as usize];
            assert_eq!(decoded.as_ref(), posting_codec::canonical_relations(source));
            decoded_relations += decoded.len();
            assert_eq!(decoded.len(), item.relation_count as usize);
        }
        assert_eq!(decoded_relations as u64, header.forward_relations);

        let eager = LexicalGrokkingMemory::from_bytes(&v7).expect("load eager V7");
        let lazy = LexicalGrokkingMemory::load(&output).expect("load mmap V8");
        for terminal_id in 0..package.terminal_count() {
            assert_eq!(
                lazy.character_anchors(terminal_id),
                eager.character_anchors(terminal_id),
                "DecoderGraph anchor reconstruction differs for terminal {terminal_id}"
            );
        }
        for surface in ["время", "вреям", "работат", "downlod"] {
            assert_eq!(
                lazy.readout(surface, 8, ReadoutMode::Full),
                eager.readout(surface, 8, ReadoutMode::Full),
                "V8 readout differs for {surface}"
            );
        }
        let _ = fs::remove_dir_all(directory);
    }
}
