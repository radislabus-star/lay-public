//! Physical Phase 8I package: compact V7 base plus exact-support overflow.

use std::fs::{self, File};
use std::io::{self, Write};
use std::path::{Path, PathBuf};

use sha2::{Digest, Sha256};

use super::format;
use super::model::LexicalGrokkingPackage;
use super::typed_basin::ExactSupportField;
use super::v8;

const MAGIC: [u8; 8] = *b"LAYL1V9\0";
const VERSION: u32 = 9;
const HEADER_BYTES: usize = 32;
const CHECKSUM_OFFSET: usize = 24;
const OVERFLOW_ENTRY_BYTES: usize = 8;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct V9Header {
    pub(super) file_bytes: u64,
    pub(super) base_bytes: u64,
    pub(super) overflow_count: u32,
    pub(super) checksum: u64,
}

pub(super) struct LoadedV9 {
    pub(super) package: LexicalGrokkingPackage,
    pub(super) support: ExactSupportField,
    pub(super) header: V9Header,
}

pub(super) fn is_v9(bytes: &[u8]) -> bool {
    bytes.get(..MAGIC.len()) == Some(MAGIC.as_slice())
}

pub(super) fn load(path: &Path) -> Result<LoadedV9, String> {
    let bytes = fs::read(path).map_err(|error| format!("{}: {error}", path.display()))?;
    decode(&bytes)
}

pub(super) fn read_header(bytes: &[u8]) -> Result<V9Header, String> {
    if bytes.len() < HEADER_BYTES
        || !is_v9(bytes)
        || read_u32(bytes, 8)? != VERSION
        || read_u32(bytes, 12)? as usize != HEADER_BYTES
    {
        return Err("invalid L1.1 V9 header".to_string());
    }
    let base_bytes = read_u64(bytes, 16)?;
    let payload_bytes = bytes
        .len()
        .checked_sub(HEADER_BYTES)
        .and_then(|value| value.checked_sub(base_bytes as usize))
        .ok_or_else(|| "invalid L1.1 V9 base length".to_string())?;
    if payload_bytes % OVERFLOW_ENTRY_BYTES != 0 {
        return Err("invalid L1.1 V9 overflow length".to_string());
    }
    let overflow_count = u32::try_from(payload_bytes / OVERFLOW_ENTRY_BYTES)
        .map_err(|_| "L1.1 V9 overflow count exceeds u32".to_string())?;
    Ok(V9Header {
        file_bytes: bytes.len() as u64,
        base_bytes,
        overflow_count,
        checksum: read_u64(bytes, CHECKSUM_OFFSET)?,
    })
}

fn decode(bytes: &[u8]) -> Result<LoadedV9, String> {
    let header = read_header(bytes)?;
    if checksum(bytes) != header.checksum {
        return Err("L1.1 V9 checksum mismatch".to_string());
    }
    let base_start = HEADER_BYTES;
    let base_end = base_start
        .checked_add(header.base_bytes as usize)
        .ok_or_else(|| "L1.1 V9 base range overflow".to_string())?;
    let base = bytes
        .get(base_start..base_end)
        .ok_or_else(|| "truncated L1.1 V9 compact base".to_string())?;
    let package = format::decode_compact_base(base)?;
    let overflow = read_overflow(bytes, base_end, header.overflow_count as usize)?;
    let support = ExactSupportField::from_compact_overflow(&package, &overflow)?;
    if support.metrics.exact_overflow_atoms != overflow.len() {
        return Err("L1.1 V9 exact-support overflow cardinality differs".to_string());
    }
    Ok(LoadedV9 {
        package,
        support,
        header,
    })
}

pub fn build_exact_v9_package(input: &Path, output: &Path) -> io::Result<serde_json::Value> {
    let input_bytes = fs::read(input)?;
    if !v8::is_v8(&input_bytes) {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Phase 8I V9 builder requires a V8 package",
        ));
    }
    let v8_header = v8::read_header(&input_bytes).map_err(io::Error::other)?;
    let base = v8::base_bytes(&input_bytes, v8_header).map_err(io::Error::other)?;
    let package = format::decode_compact_base(base).map_err(io::Error::other)?;
    let support = ExactSupportField::rebuild_decoded(&package).map_err(io::Error::other)?;
    if support.metrics.stored_support_mismatches != 0 {
        return Err(io::Error::other(format!(
            "compact base support differs from decoded exact support for {} atoms",
            support.metrics.stored_support_mismatches
        )));
    }
    let overflow = support.overflow_entries();
    let bytes = encode(base, &overflow)?;
    let temporary = temporary_path(output);
    if let Some(parent) = output.parent() {
        fs::create_dir_all(parent)?;
    }
    let write_result = (|| -> io::Result<()> {
        let mut file = File::create(&temporary)?;
        file.write_all(&bytes)?;
        file.sync_all()?;
        let loaded = load(&temporary).map_err(io::Error::other)?;
        if loaded.support.values() != support.values() {
            return Err(io::Error::other(
                "V9 support changed during physical round-trip",
            ));
        }
        if loaded.package.corpus_hash != package.corpus_hash
            || loaded.package.terminal_count() != package.terminal_count()
            || loaded.package.atoms.len() != package.atoms.len()
        {
            return Err(io::Error::other(
                "V9 compact base changed during physical round-trip",
            ));
        }
        fs::rename(&temporary, output)?;
        Ok(())
    })();
    if write_result.is_err() {
        let _ = fs::remove_file(&temporary);
    }
    write_result?;

    Ok(serde_json::json!({
        "schema": "lay.l11.phase8i.v9-build.v1",
        "format": "V9 compact-exact-typed-basin",
        "input": input,
        "output": output,
        "input_v8_bytes": input_bytes.len(),
        "compact_v7_base_bytes": base.len(),
        "header_bytes": HEADER_BYTES,
        "overflow_entries": overflow.len(),
        "overflow_bytes": overflow.len().saturating_mul(OVERFLOW_ENTRY_BYTES),
        "output_bytes": bytes.len(),
        "output_sha256": sha256_hex(&bytes),
        "corpus_fingerprint": package.corpus_hash,
        "terminal_count": package.terminal_count(),
        "atom_count": package.atoms.len(),
        "stored_saturated_atoms": support.metrics.stored_saturated_atoms,
        "maximum_exact_support": support.metrics.maximum_exact_support,
        "stored_support_mismatches": support.metrics.stored_support_mismatches,
        "physical_roundtrip": true,
    }))
}

fn encode(base: &[u8], overflow: &[(u32, u32)]) -> io::Result<Vec<u8>> {
    let overflow_bytes = overflow
        .len()
        .checked_mul(OVERFLOW_ENTRY_BYTES)
        .ok_or_else(|| io::Error::other("V9 overflow byte count exceeds usize"))?;
    let file_bytes = HEADER_BYTES
        .checked_add(base.len())
        .and_then(|value| value.checked_add(overflow_bytes))
        .ok_or_else(|| io::Error::other("V9 file byte count exceeds usize"))?;
    let mut bytes = vec![0_u8; file_bytes];
    bytes[..8].copy_from_slice(&MAGIC);
    put_u32(&mut bytes, 8, VERSION);
    put_u32(&mut bytes, 12, HEADER_BYTES as u32);
    put_u64(&mut bytes, 16, base.len() as u64);
    bytes[HEADER_BYTES..HEADER_BYTES + base.len()].copy_from_slice(base);
    let mut cursor = HEADER_BYTES + base.len();
    for &(atom_id, exact_support) in overflow {
        put_u32(&mut bytes, cursor, atom_id);
        put_u32(&mut bytes, cursor + 4, exact_support);
        cursor += OVERFLOW_ENTRY_BYTES;
    }
    let checksum = checksum(&bytes);
    put_u64(&mut bytes, CHECKSUM_OFFSET, checksum);
    Ok(bytes)
}

fn read_overflow(bytes: &[u8], offset: usize, count: usize) -> Result<Vec<(u32, u32)>, String> {
    (0..count)
        .map(|index| {
            let start = offset
                .checked_add(index.saturating_mul(OVERFLOW_ENTRY_BYTES))
                .ok_or_else(|| "L1.1 V9 overflow range exceeds usize".to_string())?;
            Ok((read_u32(bytes, start)?, read_u32(bytes, start + 4)?))
        })
        .collect()
}

fn checksum(bytes: &[u8]) -> u64 {
    let mut hasher = Sha256::new();
    hasher.update(&bytes[..CHECKSUM_OFFSET.min(bytes.len())]);
    hasher.update([0_u8; 8]);
    if bytes.len() > CHECKSUM_OFFSET + 8 {
        hasher.update(&bytes[CHECKSUM_OFFSET + 8..]);
    }
    let digest = hasher.finalize();
    u64::from_le_bytes(digest[..8].try_into().expect("SHA-256 prefix"))
}

fn sha256_hex(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn temporary_path(output: &Path) -> PathBuf {
    let file_name = output
        .file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_else(|| "l11-v9.bin".to_string());
    output.with_file_name(format!(".{file_name}.tmp-{}", std::process::id()))
}

fn read_u32(bytes: &[u8], offset: usize) -> Result<u32, String> {
    let raw = bytes
        .get(offset..offset + 4)
        .ok_or_else(|| "truncated L1.1 V9 u32".to_string())?;
    Ok(u32::from_le_bytes(raw.try_into().expect("four bytes")))
}

fn read_u64(bytes: &[u8], offset: usize) -> Result<u64, String> {
    let raw = bytes
        .get(offset..offset + 8)
        .ok_or_else(|| "truncated L1.1 V9 u64".to_string())?;
    Ok(u64::from_le_bytes(raw.try_into().expect("eight bytes")))
}

fn put_u32(bytes: &mut [u8], offset: usize, value: u32) {
    bytes[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
}

fn put_u64(bytes: &mut [u8], offset: usize, value: u64) {
    bytes[offset..offset + 8].copy_from_slice(&value.to_le_bytes());
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::nanda_wave::lexical_grokking::compiler::{
        compile_with_policy, ForwardPostingPolicy, TrainingWord,
    };
    use crate::nanda_wave::lexical_grokking::restoration::RestorationReadout;
    use crate::nanda_wave::lexical_grokking::runtime::{
        L1RestorationHost, LexicalGrokkingMemory, ReadoutMode,
    };

    #[test]
    fn v9_roundtrips_exact_support_and_rejects_corruption() {
        let (root, v9_path) = build_fixture("roundtrip", &["время", "работает", "download"]);
        let loaded = load(&v9_path).expect("load V9 fixture");
        let rebuilt = ExactSupportField::rebuild_decoded(&loaded.package)
            .expect("rebuild fixture exact support");

        assert_eq!(loaded.support.values(), rebuilt.values());
        assert_eq!(
            loaded.header.file_bytes,
            fs::metadata(&v9_path).unwrap().len()
        );
        assert!(loaded.package.forward_couplings.is_empty());
        assert!(loaded.package.reverse_couplings.is_empty());

        let mut corrupted = fs::read(&v9_path).expect("read V9 fixture");
        let last = corrupted.last_mut().expect("non-empty V9 fixture");
        *last ^= 0x80;
        let corrupted_path = root.join("corrupted.v9.bin");
        fs::write(&corrupted_path, corrupted).expect("write corrupted V9 fixture");
        let error = load(&corrupted_path)
            .err()
            .expect("corrupted V9 must fail closed");
        assert!(error.contains("checksum mismatch"));

        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn v9_runtime_never_falls_back_to_legacy_modes() {
        let (root, v9_path) = build_fixture("no-fallback", &["время", "работает", "download"]);
        let memory = LexicalGrokkingMemory::load(&v9_path).expect("load V9 runtime");

        assert!(!memory.readout("вреям", 8, ReadoutMode::Full).is_empty());
        assert!(memory
            .readout("вреям", 8, ReadoutMode::WithoutAnti)
            .is_empty());

        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn v9_bounded_projection_cannot_manufacture_authority() {
        let (root, v9_path) = build_fixture("authority", &["кот", "кот"]);
        let memory = LexicalGrokkingMemory::load(&v9_path).expect("load V9 runtime");
        let (projected, readout) = memory.restoration_readout("кот", 1);

        assert_eq!(projected.len(), 1);
        assert!(matches!(
            readout,
            RestorationReadout::Tied { .. } | RestorationReadout::TiedOverflow { .. }
        ));

        let host = L1RestorationHost::load(&v9_path).expect("load V9 host");
        let restored = host.restore("кот", 1);
        assert_eq!(restored["result"]["authority"], false);
        assert!(matches!(
            restored["result"]["verdict"].as_str(),
            Some("tied" | "tied_overflow")
        ));

        let _ = fs::remove_dir_all(root);
    }

    fn build_fixture(label: &str, surfaces: &[&str]) -> (PathBuf, PathBuf) {
        let words = surfaces
            .iter()
            .enumerate()
            .map(|(terminal_id, surface)| TrainingWord {
                terminal_id: terminal_id as u32,
                surface: (*surface).to_string(),
                training_surfaces: Vec::new(),
            })
            .collect::<Vec<_>>();
        let package = compile_with_policy(&words, ForwardPostingPolicy::Complete)
            .expect("compile V9 fixture")
            .package;
        let root = std::env::temp_dir().join(format!("lay-v9-{label}-{}", std::process::id()));
        fs::create_dir_all(&root).expect("create V9 fixture directory");
        let v7_path = root.join("fixture.v7.bin");
        let v8_path = root.join("fixture.v8.bin");
        let v9_path = root.join("fixture.v9.bin");
        fs::write(
            &v7_path,
            format::encode_compact_depth0(&package).expect("encode compact V7 fixture"),
        )
        .expect("write compact V7 fixture");
        v8::build_lazy_v8_package_with_shard_size(&v7_path, &v8_path, 32)
            .expect("build V8 fixture");
        build_exact_v9_package(&v8_path, &v9_path).expect("build V9 fixture");
        (root, v9_path)
    }
}
