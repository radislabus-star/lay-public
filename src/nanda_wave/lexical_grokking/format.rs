use std::collections::BTreeMap;

use super::atoms::{encode_wave_surface, physical_key_sequence, AtomChannel};
use super::crystal::{
    AtomWaveCode, ComplexBasisWave, WordCenter64, WAVE_DIMENSION, WORD_CENTER_BYTES,
};
use super::model::{
    AtomRecord, CenterPhaseProfile, DecoderNode, LexicalGrokkingPackage, PairKey, PairPhaseProfile,
    WaveCoupling, CENTER_PHASE_FLAG_PHYSICAL_KEY_GEOMETRY, COUPLING_FLAG_CHARACTER_ANCHOR,
};
use super::ngram_graph::{NGramArc, NGramGraph, NGramNode};
use super::posting_codec::{decode_posting, encode_posting};
use super::restoration::RestorationCalibration;

const MAGIC_V2: &[u8; 8] = b"LAYL1C02";
const MAGIC_V3: &[u8; 8] = b"LAYL1C03";
const MAGIC_V4: &[u8; 8] = b"LAYL1C04";
const MAGIC_V5: &[u8; 8] = b"LAYL1C05";
const MAGIC_V6: &[u8; 8] = b"LAYL1C06";
const MAGIC_V7: &[u8; 8] = b"LAYL1C07";
const VERSION_V2: u32 = 2;
const VERSION_V3: u32 = 3;
const VERSION_V4: u32 = 4;
const VERSION_V5: u32 = 5;
const VERSION_V6: u32 = 6;
const VERSION_V7: u32 = 7;
const HEADER_BYTES: usize = 192;
const NODE_BYTES: usize = 12;
const ARC_BYTES: usize = 8;
const BASIS_BYTES: usize = WAVE_DIMENSION * 2;
const LEGACY_ATOM_BYTES: usize = 24;
const ATOM_BYTES_V6: usize = 28;
const COUPLING_BYTES: usize = 8;
const DECODER_NODE_BYTES: usize = 8;
const PAIR_PROFILE_BYTES: usize = 24;
const L11_EXTENSION_MAGIC: &[u8; 8] = b"LAYL11E2";
const L11_EXTENSION_HEADER_BYTES: usize = 40;
const CENTER_PHASE_PROFILE_BYTES: usize = 24;
const MAX_REVERSE_LEXICAL_COUPLINGS: usize = 96;

pub(super) fn inspect_header(bytes: &[u8]) -> Result<(u64, u32, u64), String> {
    if bytes.len() < HEADER_BYTES {
        return Err("truncated L1 crystal header".to_string());
    }
    let version = read_u32(bytes, 8)?;
    let valid_magic = match version {
        VERSION_V2 => bytes.get(..8) == Some(MAGIC_V2.as_slice()),
        VERSION_V3 => bytes.get(..8) == Some(MAGIC_V3.as_slice()),
        VERSION_V4 => bytes.get(..8) == Some(MAGIC_V4.as_slice()),
        VERSION_V5 => bytes.get(..8) == Some(MAGIC_V5.as_slice()),
        VERSION_V6 => bytes.get(..8) == Some(MAGIC_V6.as_slice()),
        VERSION_V7 => bytes.get(..8) == Some(MAGIC_V7.as_slice()),
        _ => false,
    };
    if !valid_magic || read_u32(bytes, 12)? as usize != HEADER_BYTES {
        return Err("invalid L1 crystal header".to_string());
    }
    Ok((
        read_u64(bytes, 24)?,
        read_u32(bytes, 36)?,
        read_u64(bytes, 16)?,
    ))
}

pub(super) fn encode(package: &LexicalGrokkingPackage) -> Result<Vec<u8>, String> {
    let version = if package.center_phase_profiles.is_empty() {
        VERSION_V4
    } else {
        VERSION_V6
    };
    encode_version(package, version)
}

pub(super) fn encode_compact_depth0(package: &LexicalGrokkingPackage) -> Result<Vec<u8>, String> {
    validate_compact_depth0(package)?;
    encode_version(package, VERSION_V7)
}

#[cfg(test)]
pub(super) fn encode_v5_compat(package: &LexicalGrokkingPackage) -> Result<Vec<u8>, String> {
    encode_version(package, VERSION_V5)
}

fn encode_version(package: &LexicalGrokkingPackage, version: u32) -> Result<Vec<u8>, String> {
    let compact_depth0 = version == VERSION_V7;
    let mut counts = counts(package)?;
    if compact_depth0 {
        counts.forward = 0;
        counts.reverse = 0;
    }
    let (compressed_forward, atom_forward_offsets) = if compact_depth0 {
        (Vec::new(), vec![0_u32; package.atoms.len()])
    } else {
        encode_forward(package)?
    };
    let offsets = match version {
        VERSION_V4 | VERSION_V5 => offsets_v4(counts, compressed_forward.len()),
        VERSION_V6 => offsets_v6(counts, compressed_forward.len()),
        VERSION_V7 => offsets_v7(counts, compressed_forward.len()),
        _ => return Err("unsupported L1 crystal encoding version".to_string()),
    };
    let base_bytes = offsets.decoder_nodes + package.decoder_nodes.len() * DECODER_NODE_BYTES;
    let has_l11 = !package.center_phase_profiles.is_empty();
    if !has_l11
        && (!package.positive_subcenters.is_empty()
            || !package.anti_subcenters.is_empty()
            || !package.hard_negative_subcenters.is_empty()
            || !package.ambiguity_subcenters.is_empty()
            || !package.keyboard_geometry_units.is_empty())
    {
        return Err("L1.1 extension banks require primary phase profiles".to_string());
    }
    if (version >= VERSION_V5) != has_l11 {
        return Err("L1 crystal version and L1.1 extension disagree".to_string());
    }
    let file_bytes = if compact_depth0 {
        base_bytes.saturating_add(L11_EXTENSION_HEADER_BYTES)
    } else if has_l11 {
        base_bytes.saturating_add(l11_extension_bytes(package)?)
    } else {
        base_bytes
    };
    let mut bytes = vec![0_u8; file_bytes];
    let magic = match version {
        VERSION_V4 => MAGIC_V4,
        VERSION_V5 => MAGIC_V5,
        VERSION_V6 => MAGIC_V6,
        VERSION_V7 => MAGIC_V7,
        _ => unreachable!(),
    };
    bytes[..8].copy_from_slice(magic);
    put_u32(&mut bytes, 8, version);
    put_u32(&mut bytes, 12, HEADER_BYTES as u32);
    put_u64(&mut bytes, 16, file_bytes as u64);
    put_u64(&mut bytes, 24, package.corpus_hash);
    put_u32(&mut bytes, 32, WAVE_DIMENSION as u32);
    put_u32(&mut bytes, 36, package.terminal_count());
    for (index, count) in counts.as_array().into_iter().enumerate() {
        put_u32(&mut bytes, 40 + index * 4, count);
    }
    put_u32(
        &mut bytes,
        76,
        as_u32(compressed_forward.len(), "compressed forward byte")?,
    );
    put_u32(&mut bytes, 168, counts.pair_profiles);
    put_u32(&mut bytes, 172, counts.pair_centers);
    for (index, offset) in offsets.as_array().into_iter().enumerate() {
        put_u64(&mut bytes, 80 + index * 8, offset as u64);
    }
    put_u64(&mut bytes, 152, offsets.pair_profiles as u64);
    put_u64(&mut bytes, 160, offsets.pair_centers as u64);

    write_nodes(&mut bytes, offsets.nodes, &package.graph.nodes);
    write_arcs(&mut bytes, offsets.arcs, &package.graph.arcs);
    write_basis(&mut bytes, offsets.basis, &package.basis);
    if compact_depth0 {
        write_compact_depth0_atoms(&mut bytes, offsets.atoms, &package.atoms);
    } else {
        write_atoms(
            &mut bytes,
            offsets.atoms,
            &package.atoms,
            Some(&atom_forward_offsets),
            version,
        )?;
    }
    bytes[offsets.forward..offsets.reverse].copy_from_slice(&compressed_forward);
    if !compact_depth0 {
        write_couplings(&mut bytes, offsets.reverse, &package.reverse_couplings);
    }
    write_centers(&mut bytes, offsets.anti, &package.anti_centers);
    write_pair_profiles(&mut bytes, offsets.pair_profiles, &package.pair_profiles);
    write_centers(&mut bytes, offsets.pair_centers, &package.pair_centers);
    if compact_depth0 {
        write_compact_depth0_centers(&mut bytes, offsets.centers, &package.centers);
    } else {
        write_centers(&mut bytes, offsets.centers, &package.centers);
    }
    for (index, node) in package.decoder_nodes.iter().copied().enumerate() {
        let start = offsets.decoder_nodes + index * DECODER_NODE_BYTES;
        put_u32(&mut bytes, start, node.parent);
        put_u32(&mut bytes, start + 4, node.symbol);
    }
    if compact_depth0 {
        write_compact_depth0_extension(&mut bytes, base_bytes, package)?;
    } else if has_l11 {
        write_l11_extension(&mut bytes, base_bytes, package)?;
    }
    let checksum = checksum(&bytes);
    put_u64(&mut bytes, 176, checksum);
    Ok(bytes)
}

pub(super) fn decode(bytes: &[u8]) -> Result<LexicalGrokkingPackage, String> {
    decode_with_compact_views(bytes, true)
}

pub(super) fn decode_compact_base(bytes: &[u8]) -> Result<LexicalGrokkingPackage, String> {
    decode_with_compact_views(bytes, false)
}

fn decode_with_compact_views(
    bytes: &[u8],
    rebuild_compact_views: bool,
) -> Result<LexicalGrokkingPackage, String> {
    if bytes.len() < HEADER_BYTES {
        return Err("invalid L1 crystal package magic".to_string());
    }
    let version = read_u32(bytes, 8)?;
    let valid_magic = match version {
        VERSION_V2 => bytes.get(..8) == Some(MAGIC_V2.as_slice()),
        VERSION_V3 => bytes.get(..8) == Some(MAGIC_V3.as_slice()),
        VERSION_V4 => bytes.get(..8) == Some(MAGIC_V4.as_slice()),
        VERSION_V5 => bytes.get(..8) == Some(MAGIC_V5.as_slice()),
        VERSION_V6 => bytes.get(..8) == Some(MAGIC_V6.as_slice()),
        VERSION_V7 => bytes.get(..8) == Some(MAGIC_V7.as_slice()),
        _ => false,
    };
    if !valid_magic || read_u32(bytes, 12)? as usize != HEADER_BYTES {
        return Err("unsupported L1 crystal package version".to_string());
    }
    if read_u64(bytes, 16)? as usize != bytes.len() {
        return Err("invalid L1 crystal package size".to_string());
    }
    if read_u32(bytes, 32)? as usize != WAVE_DIMENSION {
        return Err("unsupported L1 crystal wave dimension".to_string());
    }
    let terminal_count = read_u32(bytes, 36)?;
    let counts = Counts::read(bytes, version)?;
    let stored_offsets = Offsets::read(bytes, version)?;
    let expected_offsets = match version {
        VERSION_V2 => offsets_v2(counts),
        VERSION_V3 => offsets_v3(counts, read_u32(bytes, 76)? as usize),
        VERSION_V4 => offsets_v4(counts, read_u32(bytes, 76)? as usize),
        VERSION_V5 => offsets_v4(counts, read_u32(bytes, 76)? as usize),
        VERSION_V6 => offsets_v6(counts, read_u32(bytes, 76)? as usize),
        VERSION_V7 => offsets_v7(counts, read_u32(bytes, 76)? as usize),
        _ => unreachable!(),
    };
    let base_bytes =
        stored_offsets.decoder_nodes + counts.decoder_nodes as usize * DECODER_NODE_BYTES;
    if stored_offsets != expected_offsets || bytes.len() < base_bytes {
        return Err("invalid L1 crystal package offsets".to_string());
    }
    if version < VERSION_V5 && bytes.len() != base_bytes {
        return Err("invalid L1 crystal package trailing bytes".to_string());
    }
    if checksum(bytes) != read_u64(bytes, 176)? {
        return Err("L1 crystal package checksum mismatch".to_string());
    }

    let graph = NGramGraph {
        nodes: read_nodes(bytes, stored_offsets.nodes, counts.nodes as usize)?,
        arcs: read_arcs(bytes, stored_offsets.arcs, counts.arcs as usize)?,
        atom_count: counts.atoms,
    };
    validate_graph(&graph)?;
    let basis = read_basis(bytes, stored_offsets.basis, counts.basis as usize)?;
    let mut atoms = read_atoms(bytes, stored_offsets.atoms, counts.atoms as usize, version)?;
    let mut forward_couplings = if version == VERSION_V2 {
        read_couplings(bytes, stored_offsets.forward, counts.forward as usize)?
    } else {
        read_compressed_forward(
            bytes,
            stored_offsets.forward,
            stored_offsets.reverse,
            &mut atoms,
            counts.forward as usize,
        )?
    };
    let mut reverse_couplings =
        read_couplings(bytes, stored_offsets.reverse, counts.reverse as usize)?;
    let anti_centers = read_centers(bytes, stored_offsets.anti, counts.anti as usize)?;
    let pair_profiles = read_pair_profiles(
        bytes,
        stored_offsets.pair_profiles,
        counts.pair_profiles as usize,
    )?;
    let pair_centers = read_centers(
        bytes,
        stored_offsets.pair_centers,
        counts.pair_centers as usize,
    )?;
    let mut centers = read_centers(bytes, stored_offsets.centers, counts.centers as usize)?;
    if centers.len() != terminal_count as usize {
        return Err("L1 crystal terminal and center counts differ".to_string());
    }
    let decoder_nodes = read_decoder_nodes(
        bytes,
        stored_offsets.decoder_nodes,
        counts.decoder_nodes as usize,
    )?;
    let (
        center_phase_profiles,
        positive_subcenters,
        anti_subcenters,
        hard_negative_subcenters,
        ambiguity_subcenters,
        keyboard_geometry_units,
        restoration_calibration,
    ) = if version == VERSION_V7 && rebuild_compact_views {
        let restoration_calibration = read_compact_depth0_extension(bytes, base_bytes)?;
        let (rebuilt_forward, rebuilt_reverse, rebuilt_profiles, rebuilt_keyboard_geometry) =
            rebuild_compact_depth0_views(&graph, &mut atoms, &mut centers, &decoder_nodes)?;
        forward_couplings = rebuilt_forward;
        reverse_couplings = rebuilt_reverse;
        (
            rebuilt_profiles,
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            rebuilt_keyboard_geometry,
            restoration_calibration,
        )
    } else if version == VERSION_V7 {
        (
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            read_compact_depth0_extension(bytes, base_bytes)?,
        )
    } else if version >= VERSION_V5 {
        read_l11_extension(bytes, base_bytes, centers.len(), atoms.len())?
    } else {
        (
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            RestorationCalibration::LEGACY_PERMISSIVE,
        )
    };
    validate_ranges(
        &atoms,
        &centers,
        forward_couplings.len(),
        reverse_couplings.len(),
        anti_centers.len(),
        &pair_profiles,
        pair_centers.len(),
        decoder_nodes.len(),
    )?;
    validate_l11_ranges(
        &center_phase_profiles,
        positive_subcenters.len(),
        anti_subcenters.len(),
        hard_negative_subcenters.len(),
        ambiguity_subcenters.len(),
        &keyboard_geometry_units,
        centers.len(),
        atoms.len(),
    )?;
    Ok(LexicalGrokkingPackage {
        corpus_hash: read_u64(bytes, 24)?,
        graph,
        basis,
        atoms,
        forward_couplings,
        reverse_couplings,
        anti_centers,
        pair_profiles,
        pair_centers,
        center_phase_profiles,
        positive_subcenters,
        anti_subcenters,
        hard_negative_subcenters,
        ambiguity_subcenters,
        keyboard_geometry_units,
        restoration_calibration,
        centers,
        decoder_nodes,
    })
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Counts {
    nodes: u32,
    arcs: u32,
    basis: u32,
    atoms: u32,
    forward: u32,
    reverse: u32,
    anti: u32,
    centers: u32,
    decoder_nodes: u32,
    pair_profiles: u32,
    pair_centers: u32,
}

impl Counts {
    fn as_array(self) -> [u32; 9] {
        [
            self.nodes,
            self.arcs,
            self.basis,
            self.atoms,
            self.forward,
            self.reverse,
            self.anti,
            self.centers,
            self.decoder_nodes,
        ]
    }

    fn read(bytes: &[u8], version: u32) -> Result<Self, String> {
        let mut values = [0_u32; 9];
        for (index, value) in values.iter_mut().enumerate() {
            *value = read_u32(bytes, 40 + index * 4)?;
        }
        Ok(Self {
            nodes: values[0],
            arcs: values[1],
            basis: values[2],
            atoms: values[3],
            forward: values[4],
            reverse: values[5],
            anti: values[6],
            centers: values[7],
            decoder_nodes: values[8],
            pair_profiles: if version >= VERSION_V4 {
                read_u32(bytes, 168)?
            } else {
                0
            },
            pair_centers: if version >= VERSION_V4 {
                read_u32(bytes, 172)?
            } else {
                0
            },
        })
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Offsets {
    nodes: usize,
    arcs: usize,
    basis: usize,
    atoms: usize,
    forward: usize,
    reverse: usize,
    anti: usize,
    centers: usize,
    decoder_nodes: usize,
    pair_profiles: usize,
    pair_centers: usize,
}

impl Offsets {
    fn as_array(self) -> [usize; 9] {
        [
            self.nodes,
            self.arcs,
            self.basis,
            self.atoms,
            self.forward,
            self.reverse,
            self.anti,
            self.centers,
            self.decoder_nodes,
        ]
    }

    fn read(bytes: &[u8], version: u32) -> Result<Self, String> {
        let mut values = [0_usize; 9];
        for (index, value) in values.iter_mut().enumerate() {
            *value = read_u64(bytes, 80 + index * 8)? as usize;
        }
        Ok(Self {
            nodes: values[0],
            arcs: values[1],
            basis: values[2],
            atoms: values[3],
            forward: values[4],
            reverse: values[5],
            anti: values[6],
            centers: values[7],
            decoder_nodes: values[8],
            pair_profiles: if version >= VERSION_V4 {
                read_u64(bytes, 152)? as usize
            } else {
                values[7]
            },
            pair_centers: if version >= VERSION_V4 {
                read_u64(bytes, 160)? as usize
            } else {
                values[7]
            },
        })
    }
}

fn counts(package: &LexicalGrokkingPackage) -> Result<Counts, String> {
    Ok(Counts {
        nodes: as_u32(package.graph.nodes.len(), "node")?,
        arcs: as_u32(package.graph.arcs.len(), "arc")?,
        basis: as_u32(package.basis.len(), "basis")?,
        atoms: as_u32(package.atoms.len(), "atom")?,
        forward: as_u32(package.forward_couplings.len(), "forward coupling")?,
        reverse: as_u32(package.reverse_couplings.len(), "reverse coupling")?,
        anti: as_u32(package.anti_centers.len(), "anti center")?,
        centers: as_u32(package.centers.len(), "word center")?,
        decoder_nodes: as_u32(package.decoder_nodes.len(), "decoder node")?,
        pair_profiles: as_u32(package.pair_profiles.len(), "pair profile")?,
        pair_centers: as_u32(package.pair_centers.len(), "pair center")?,
    })
}

fn offsets_v2(counts: Counts) -> Offsets {
    offsets_with_forward_bytes(
        counts,
        counts.forward as usize * COUPLING_BYTES,
        LEGACY_ATOM_BYTES,
    )
}

fn offsets_v3(counts: Counts, forward_bytes: usize) -> Offsets {
    offsets_with_forward_bytes(counts, forward_bytes, LEGACY_ATOM_BYTES)
}

fn offsets_v4(counts: Counts, forward_bytes: usize) -> Offsets {
    offsets_with_forward_bytes(counts, forward_bytes, LEGACY_ATOM_BYTES)
}

fn offsets_v6(counts: Counts, forward_bytes: usize) -> Offsets {
    offsets_with_forward_bytes(counts, forward_bytes, ATOM_BYTES_V6)
}

fn offsets_v7(counts: Counts, forward_bytes: usize) -> Offsets {
    offsets_with_forward_bytes(counts, forward_bytes, ATOM_BYTES_V6)
}

fn offsets_with_forward_bytes(counts: Counts, forward_bytes: usize, atom_bytes: usize) -> Offsets {
    let nodes = HEADER_BYTES;
    let arcs = nodes + counts.nodes as usize * NODE_BYTES;
    let basis = arcs + counts.arcs as usize * ARC_BYTES;
    let atoms = basis + counts.basis as usize * BASIS_BYTES;
    let forward = atoms + counts.atoms as usize * atom_bytes;
    let reverse = forward + forward_bytes;
    let anti = reverse + counts.reverse as usize * COUPLING_BYTES;
    let pair_profiles = anti + counts.anti as usize * WORD_CENTER_BYTES;
    let pair_centers = pair_profiles + counts.pair_profiles as usize * PAIR_PROFILE_BYTES;
    let centers = pair_centers + counts.pair_centers as usize * WORD_CENTER_BYTES;
    let decoder_nodes = centers + counts.centers as usize * WORD_CENTER_BYTES;
    Offsets {
        nodes,
        arcs,
        basis,
        atoms,
        forward,
        reverse,
        anti,
        pair_profiles,
        pair_centers,
        centers,
        decoder_nodes,
    }
}

fn write_nodes(bytes: &mut [u8], offset: usize, nodes: &[NGramNode]) {
    for (index, node) in nodes.iter().copied().enumerate() {
        let start = offset + index * NODE_BYTES;
        put_u32(bytes, start, node.first_arc);
        put_u16(bytes, start + 4, node.arc_count);
        put_u16(bytes, start + 6, 0);
        put_u32(bytes, start + 8, node.atom_id);
    }
}

fn write_arcs(bytes: &mut [u8], offset: usize, arcs: &[NGramArc]) {
    for (index, arc) in arcs.iter().copied().enumerate() {
        let start = offset + index * ARC_BYTES;
        put_u32(bytes, start, arc.symbol);
        put_u32(bytes, start + 4, arc.next_node);
    }
}

fn write_basis(bytes: &mut [u8], offset: usize, basis: &[ComplexBasisWave]) {
    for (index, wave) in basis.iter().enumerate() {
        let start = offset + index * BASIS_BYTES;
        for cell in 0..WAVE_DIMENSION {
            bytes[start + cell * 2] = wave.re[cell] as u8;
            bytes[start + cell * 2 + 1] = wave.im[cell] as u8;
        }
    }
}

fn write_atoms(
    bytes: &mut [u8],
    offset: usize,
    atoms: &[AtomRecord],
    coupling_offsets: Option<&[u32]>,
    version: u32,
) -> Result<(), String> {
    let atom_bytes = atom_bytes(version);
    for (index, atom) in atoms.iter().copied().enumerate() {
        let start = offset + index * atom_bytes;
        bytes[start..start + AtomWaveCode::BYTES].copy_from_slice(&atom.wave_code.encode());
        put_u32(
            bytes,
            start + 16,
            coupling_offsets
                .and_then(|offsets| offsets.get(index).copied())
                .unwrap_or(atom.coupling_start),
        );
        if version >= VERSION_V6 {
            put_u32(bytes, start + 20, atom.coupling_count);
            put_u16(bytes, start + 24, atom.support);
        } else {
            let coupling_count = u16::try_from(atom.coupling_count)
                .map_err(|_| "legacy L1 atom coupling count exceeds u16".to_string())?;
            put_u16(bytes, start + 20, coupling_count);
            put_u16(bytes, start + 22, atom.support);
        }
    }
    Ok(())
}

fn write_compact_depth0_atoms(bytes: &mut [u8], offset: usize, atoms: &[AtomRecord]) {
    for (index, atom) in atoms.iter().copied().enumerate() {
        let start = offset + index * ATOM_BYTES_V6;
        bytes[start..start + AtomWaveCode::BYTES].copy_from_slice(&atom.wave_code.encode());
        put_u32(bytes, start + 16, 0);
        put_u32(bytes, start + 20, 0);
        put_u16(bytes, start + 24, atom.support);
    }
}

fn write_couplings(bytes: &mut [u8], offset: usize, couplings: &[WaveCoupling]) {
    for (index, coupling) in couplings.iter().copied().enumerate() {
        let start = offset + index * COUPLING_BYTES;
        put_u32(bytes, start, coupling.peer_id);
        bytes[start + 4] = coupling.strength;
        bytes[start + 5] = coupling.phase_relation as u8;
        bytes[start + 6] = coupling.position_mode;
        bytes[start + 7] = coupling.flags;
    }
}

fn write_centers(bytes: &mut [u8], offset: usize, centers: &[WordCenter64]) {
    for (index, center) in centers.iter().copied().enumerate() {
        let start = offset + index * WORD_CENTER_BYTES;
        bytes[start..start + WORD_CENTER_BYTES].copy_from_slice(&center.encode());
    }
}

fn write_compact_depth0_centers(bytes: &mut [u8], offset: usize, centers: &[WordCenter64]) {
    for (index, center) in centers.iter().copied().enumerate() {
        let start = offset + index * WORD_CENTER_BYTES;
        let mut stored = center;
        stored.coupling_start = 0;
        stored.coupling_count = 0;
        bytes[start..start + WORD_CENTER_BYTES].copy_from_slice(&stored.encode());
    }
}

fn write_pair_profiles(bytes: &mut [u8], offset: usize, profiles: &[PairPhaseProfile]) {
    for (index, profile) in profiles.iter().copied().enumerate() {
        let start = offset + index * PAIR_PROFILE_BYTES;
        put_u32(bytes, start, profile.key.low_terminal);
        put_u32(bytes, start + 4, profile.key.high_terminal);
        put_u32(bytes, start + 8, profile.low_wins_start);
        put_u32(bytes, start + 12, profile.high_wins_start);
        put_u16(bytes, start + 16, profile.low_wins_count);
        put_u16(bytes, start + 18, profile.high_wins_count);
    }
}

fn read_nodes(bytes: &[u8], offset: usize, count: usize) -> Result<Vec<NGramNode>, String> {
    (0..count)
        .map(|index| {
            let start = offset + index * NODE_BYTES;
            Ok(NGramNode {
                first_arc: read_u32(bytes, start)?,
                arc_count: read_u16(bytes, start + 4)?,
                atom_id: read_u32(bytes, start + 8)?,
            })
        })
        .collect()
}

fn read_arcs(bytes: &[u8], offset: usize, count: usize) -> Result<Vec<NGramArc>, String> {
    (0..count)
        .map(|index| {
            let start = offset + index * ARC_BYTES;
            Ok(NGramArc {
                symbol: read_u32(bytes, start)?,
                next_node: read_u32(bytes, start + 4)?,
            })
        })
        .collect()
}

fn read_basis(bytes: &[u8], offset: usize, count: usize) -> Result<Vec<ComplexBasisWave>, String> {
    (0..count)
        .map(|index| {
            let start = offset + index * BASIS_BYTES;
            let raw = bytes
                .get(start..start + BASIS_BYTES)
                .ok_or_else(|| "truncated complex basis".to_string())?;
            let mut wave = ComplexBasisWave::default();
            for cell in 0..WAVE_DIMENSION {
                wave.re[cell] = raw[cell * 2] as i8;
                wave.im[cell] = raw[cell * 2 + 1] as i8;
            }
            Ok(wave)
        })
        .collect()
}

fn read_atoms(
    bytes: &[u8],
    offset: usize,
    count: usize,
    version: u32,
) -> Result<Vec<AtomRecord>, String> {
    let atom_bytes = atom_bytes(version);
    (0..count)
        .map(|index| {
            let start = offset + index * atom_bytes;
            let raw = bytes
                .get(start..start + atom_bytes)
                .ok_or_else(|| "truncated atom record".to_string())?;
            Ok(AtomRecord {
                wave_code: AtomWaveCode::decode(&raw[..AtomWaveCode::BYTES])?,
                coupling_start: read_u32(raw, 16)?,
                coupling_count: if version >= VERSION_V6 {
                    read_u32(raw, 20)?
                } else {
                    u32::from(read_u16(raw, 20)?)
                },
                support: read_u16(raw, if version >= VERSION_V6 { 24 } else { 22 })?,
            })
        })
        .collect()
}

fn atom_bytes(version: u32) -> usize {
    if version >= VERSION_V6 {
        ATOM_BYTES_V6
    } else {
        LEGACY_ATOM_BYTES
    }
}

fn read_couplings(bytes: &[u8], offset: usize, count: usize) -> Result<Vec<WaveCoupling>, String> {
    (0..count)
        .map(|index| {
            let start = offset + index * COUPLING_BYTES;
            let raw = bytes
                .get(start..start + COUPLING_BYTES)
                .ok_or_else(|| "truncated wave coupling".to_string())?;
            Ok(WaveCoupling {
                peer_id: u32::from_le_bytes([raw[0], raw[1], raw[2], raw[3]]),
                strength: raw[4],
                phase_relation: raw[5] as i8,
                position_mode: raw[6],
                flags: raw[7],
            })
        })
        .collect()
}

fn encode_forward(package: &LexicalGrokkingPackage) -> Result<(Vec<u8>, Vec<u32>), String> {
    let mut bytes = Vec::new();
    let mut starts = Vec::with_capacity(package.atoms.len());
    for atom in &package.atoms {
        starts.push(as_u32(bytes.len(), "compressed forward offset")?);
        let start = atom.coupling_start as usize;
        let end = start.saturating_add(atom.coupling_count as usize);
        let relations = package
            .forward_couplings
            .get(start..end)
            .ok_or_else(|| "invalid forward posting range during encoding".to_string())?;
        bytes.extend_from_slice(&encode_posting(relations)?.bytes);
    }
    Ok((bytes, starts))
}

fn read_compressed_forward(
    bytes: &[u8],
    offset: usize,
    end: usize,
    atoms: &mut [AtomRecord],
    expected_relations: usize,
) -> Result<Vec<WaveCoupling>, String> {
    let section = bytes
        .get(offset..end)
        .ok_or_else(|| "truncated compressed forward section".to_string())?;
    let declared_relations = atoms.iter().try_fold(0_usize, |total, atom| {
        total
            .checked_add(atom.coupling_count as usize)
            .ok_or_else(|| "compressed forward relation count overflow".to_string())
    })?;
    if declared_relations != expected_relations {
        return Err("compressed forward atom relation total mismatch".to_string());
    }
    let minimum_bytes = expected_relations
        .checked_mul(3)
        .and_then(|payload| payload.checked_add(expected_relations.div_ceil(32).checked_mul(8)?))
        .ok_or_else(|| "compressed forward allocation overflow".to_string())?;
    if minimum_bytes > section.len() {
        return Err("compressed forward count exceeds section capacity".to_string());
    }
    let file_starts = atoms
        .iter()
        .map(|atom| atom.coupling_start as usize)
        .collect::<Vec<_>>();
    let mut relations = Vec::with_capacity(expected_relations);
    for index in 0..atoms.len() {
        let start = file_starts[index];
        let posting_end = file_starts.get(index + 1).copied().unwrap_or(section.len());
        if start > posting_end || posting_end > section.len() {
            return Err("invalid compressed forward posting range".to_string());
        }
        let decoded = decode_posting(
            &section[start..posting_end],
            atoms[index].coupling_count as usize,
        )?;
        atoms[index].coupling_start = as_u32(relations.len(), "decoded forward relation")?;
        relations.extend(decoded);
    }
    if relations.len() != expected_relations {
        return Err("compressed forward relation total mismatch".to_string());
    }
    Ok(relations)
}

fn read_centers(bytes: &[u8], offset: usize, count: usize) -> Result<Vec<WordCenter64>, String> {
    (0..count)
        .map(|index| {
            let start = offset + index * WORD_CENTER_BYTES;
            WordCenter64::decode(
                bytes
                    .get(start..start + WORD_CENTER_BYTES)
                    .ok_or_else(|| "truncated wave center".to_string())?,
            )
        })
        .collect()
}

fn read_pair_profiles(
    bytes: &[u8],
    offset: usize,
    count: usize,
) -> Result<Vec<PairPhaseProfile>, String> {
    let profiles = (0..count)
        .map(|index| {
            let start = offset + index * PAIR_PROFILE_BYTES;
            let profile = PairPhaseProfile {
                key: PairKey {
                    low_terminal: read_u32(bytes, start)?,
                    high_terminal: read_u32(bytes, start + 4)?,
                },
                low_wins_start: read_u32(bytes, start + 8)?,
                high_wins_start: read_u32(bytes, start + 12)?,
                low_wins_count: read_u16(bytes, start + 16)?,
                high_wins_count: read_u16(bytes, start + 18)?,
            };
            if profile.key.low_terminal >= profile.key.high_terminal {
                return Err("L1 pair profile key is not canonical".to_string());
            }
            Ok(profile)
        })
        .collect::<Result<Vec<_>, _>>()?;
    if !profiles.windows(2).all(|pair| pair[0].key < pair[1].key) {
        return Err("L1 pair profile keys are not sorted and unique".to_string());
    }
    Ok(profiles)
}

fn read_decoder_nodes(
    bytes: &[u8],
    offset: usize,
    count: usize,
) -> Result<Vec<DecoderNode>, String> {
    (0..count)
        .map(|index| {
            let start = offset + index * DECODER_NODE_BYTES;
            Ok(DecoderNode {
                parent: read_u32(bytes, start)?,
                symbol: read_u32(bytes, start + 4)?,
            })
        })
        .collect()
}

fn validate_compact_depth0(package: &LexicalGrokkingPackage) -> Result<(), String> {
    if package.center_phase_profiles.len() != package.centers.len() {
        return Err("compact depth-0 requires one primary profile per center".to_string());
    }
    if !package.anti_centers.is_empty()
        || !package.pair_profiles.is_empty()
        || !package.pair_centers.is_empty()
        || !package.positive_subcenters.is_empty()
        || !package.anti_subcenters.is_empty()
        || !package.hard_negative_subcenters.is_empty()
        || !package.ambiguity_subcenters.is_empty()
    {
        return Err("compact depth-0 cannot encode learned residual banks".to_string());
    }
    if package.center_phase_profiles.iter().any(|profile| {
        profile.positive_count != 0
            || profile.anti_count != 0
            || profile.hard_negative_count != 0
            || profile.ambiguity_count != 0
            || profile.min_ambiguity_milli != 0
    }) {
        return Err("compact depth-0 profile still references a residual bank".to_string());
    }
    Ok(())
}

fn write_compact_depth0_extension(
    bytes: &mut [u8],
    offset: usize,
    package: &LexicalGrokkingPackage,
) -> Result<(), String> {
    validate_compact_depth0(package)?;
    bytes[offset..offset + 8].copy_from_slice(L11_EXTENSION_MAGIC);
    put_u32(bytes, offset + 8, 0);
    put_u32(bytes, offset + 12, 0);
    put_u32(bytes, offset + 16, 0);
    put_u32(bytes, offset + 20, 0);
    put_u32(bytes, offset + 24, 0);
    bytes[offset + 28] = package.restoration_calibration.max_geometry_distance;
    put_u16(
        bytes,
        offset + 30,
        package.restoration_calibration.min_positive_milli,
    );
    put_u16(
        bytes,
        offset + 32,
        package.restoration_calibration.min_backward_milli,
    );
    put_u16(
        bytes,
        offset + 34,
        package.restoration_calibration.min_tied_energy_margin,
    );
    put_u32(bytes, offset + 36, 0);
    Ok(())
}

fn read_compact_depth0_extension(
    bytes: &[u8],
    offset: usize,
) -> Result<RestorationCalibration, String> {
    let header = bytes
        .get(offset..offset + L11_EXTENSION_HEADER_BYTES)
        .ok_or_else(|| "truncated compact depth-0 extension".to_string())?;
    if bytes.len() != offset + L11_EXTENSION_HEADER_BYTES
        || header.get(..8) != Some(L11_EXTENSION_MAGIC.as_slice())
        || [8, 12, 16, 20, 24, 36]
            .into_iter()
            .any(|field| read_u32(header, field).unwrap_or(u32::MAX) != 0)
    {
        return Err("invalid compact depth-0 extension".to_string());
    }
    Ok(RestorationCalibration {
        max_geometry_distance: header[28],
        min_positive_milli: read_u16(header, 30)?,
        min_backward_milli: read_u16(header, 32)?,
        min_tied_energy_margin: read_u16(header, 34)?,
    })
}

type CompactDepth0Views = (
    Vec<WaveCoupling>,
    Vec<WaveCoupling>,
    Vec<CenterPhaseProfile>,
    Vec<u32>,
);

fn rebuild_compact_depth0_views(
    graph: &NGramGraph,
    atoms: &mut [AtomRecord],
    centers: &mut [WordCenter64],
    decoder_nodes: &[DecoderNode],
) -> Result<CompactDepth0Views, String> {
    let mut support = vec![0_u32; atoms.len()];
    let mut forward_degrees = vec![0_u32; atoms.len()];
    for center in centers.iter().copied() {
        let surface = decode_center_surface(center, decoder_nodes)?;
        let resolved = encode_wave_surface(&surface)
            .into_iter()
            .filter_map(|atom| {
                graph
                    .atom_id(atom.key)
                    .map(|atom_id| (atom_id, atom.key.channel))
            })
            .collect::<Vec<_>>();
        for (atom_id, _) in &resolved {
            let slot = support
                .get_mut(*atom_id as usize)
                .ok_or_else(|| "compact depth-0 atom exceeds support field".to_string())?;
            *slot = slot.saturating_add(1);
        }
        let mut lexical_atoms = BTreeMap::<u32, ()>::new();
        for (atom_id, channel) in resolved {
            if channel != AtomChannel::CharacterAnchor {
                lexical_atoms.insert(atom_id, ());
            }
        }
        for atom_id in lexical_atoms.into_keys() {
            let degree = forward_degrees
                .get_mut(atom_id as usize)
                .ok_or_else(|| "compact depth-0 atom exceeds forward field".to_string())?;
            *degree = degree.saturating_add(1);
        }
    }
    for (atom_id, (atom, exact_support)) in atoms.iter().zip(&support).enumerate() {
        if atom.support != (*exact_support).min(u32::from(u16::MAX)) as u16 {
            return Err(format!(
                "compact depth-0 atom support differs from decoder field: \
                 atom={atom_id} stored={} rebuilt={exact_support}",
                atom.support
            ));
        }
    }

    let mut forward_count = 0_usize;
    for (atom, degree) in atoms.iter_mut().zip(&forward_degrees) {
        atom.coupling_start = as_u32(forward_count, "rebuilt forward relation")?;
        atom.coupling_count = *degree;
        forward_count = forward_count
            .checked_add(*degree as usize)
            .ok_or_else(|| "rebuilt forward relation count overflow".to_string())?;
    }
    let mut forward = vec![WaveCoupling::default(); forward_count];
    let mut forward_cursors = atoms
        .iter()
        .map(|atom| atom.coupling_start as usize)
        .collect::<Vec<_>>();
    let word_count = centers.len();
    let mut reverse = Vec::new();
    let mut profiles = Vec::with_capacity(centers.len());
    let mut keyboard_geometry = Vec::new();
    for (terminal_id, center) in centers.iter_mut().enumerate() {
        let surface = decode_center_surface(*center, decoder_nodes)?;
        let resolved = encode_wave_surface(&surface)
            .into_iter()
            .filter_map(|atom| {
                graph
                    .atom_id(atom.key)
                    .map(|atom_id| (atom_id, atom.position, atom.key.channel))
            })
            .collect::<Vec<_>>();
        let mut stats = BTreeMap::<u32, (u32, u64, AtomChannel)>::new();
        for (atom_id, position, channel) in &resolved {
            let entry = stats.entry(*atom_id).or_insert((0, 0, *channel));
            entry.0 = entry.0.saturating_add(1);
            entry.1 = entry.1.saturating_add(u64::from(*position));
        }
        for (atom_id, (observation_count, position_sum, channel)) in &stats {
            if *channel == AtomChannel::CharacterAnchor {
                continue;
            }
            let average_position = position_sum / u64::from((*observation_count).max(1));
            let position_mode = (average_position / 257).min(255) as u8;
            let cursor = forward_cursors
                .get_mut(*atom_id as usize)
                .ok_or_else(|| "rebuilt forward atom is invalid".to_string())?;
            let slot = forward
                .get_mut(*cursor)
                .ok_or_else(|| "rebuilt forward range is invalid".to_string())?;
            *slot = WaveCoupling {
                peer_id: terminal_id as u32,
                strength: reconstructed_coupling_strength(
                    *observation_count,
                    support[*atom_id as usize],
                    word_count,
                ),
                phase_relation: position_phase(position_mode),
                position_mode,
                flags: 0,
            };
            *cursor = cursor.saturating_add(1);
        }
        let mut center_reverse = resolved
            .into_iter()
            .map(|(atom_id, position, channel)| {
                let observation_count =
                    stats.get(&atom_id).map(|stats| stats.0).unwrap_or_default();
                let position_mode = (position / 257).min(255) as u8;
                WaveCoupling {
                    peer_id: atom_id,
                    strength: reconstructed_coupling_strength(
                        observation_count,
                        support[atom_id as usize],
                        word_count,
                    ),
                    phase_relation: position_phase(position_mode),
                    position_mode,
                    flags: if channel == AtomChannel::CharacterAnchor {
                        COUPLING_FLAG_CHARACTER_ANCHOR
                    } else {
                        0
                    },
                }
            })
            .collect::<Vec<_>>();
        center_reverse.sort_unstable_by(coupling_order);
        let anchor_count = center_reverse
            .iter()
            .take_while(|relation| relation.flags != 0)
            .count();
        center_reverse.truncate(anchor_count.saturating_add(MAX_REVERSE_LEXICAL_COUPLINGS));
        center.coupling_start = as_u32(reverse.len(), "rebuilt reverse relation")?;
        center.coupling_count = u16::try_from(center_reverse.len())
            .map_err(|_| "rebuilt reverse center degree exceeds u16".to_string())?;
        reverse.extend(center_reverse);

        let keyboard_geometry_start = as_u32(keyboard_geometry.len(), "rebuilt keyboard geometry")?;
        let physical_keys = physical_key_sequence(&surface);
        let keyboard_geometry_count = u8::try_from(physical_keys.len())
            .map_err(|_| "rebuilt keyboard geometry exceeds u8".to_string())?;
        keyboard_geometry.extend(physical_keys);
        profiles.push(CenterPhaseProfile {
            keyboard_geometry_start,
            keyboard_geometry_count,
            flags: CENTER_PHASE_FLAG_PHYSICAL_KEY_GEOMETRY,
            ..CenterPhaseProfile::default()
        });
    }
    for (atom, cursor) in atoms.iter().zip(forward_cursors) {
        if cursor != atom.coupling_start as usize + atom.coupling_count as usize {
            return Err("rebuilt forward posting is incomplete".to_string());
        }
    }
    Ok((forward, reverse, profiles, keyboard_geometry))
}

pub(super) fn decode_center_surface(
    center: WordCenter64,
    decoder_nodes: &[DecoderNode],
) -> Result<String, String> {
    let mut node = center.decoder_terminal;
    let mut symbols = Vec::new();
    while node != 0 {
        let item = decoder_nodes
            .get(node as usize)
            .ok_or_else(|| "compact depth-0 decoder terminal is invalid".to_string())?;
        symbols.push(
            char::from_u32(item.symbol)
                .ok_or_else(|| "compact depth-0 decoder symbol is invalid".to_string())?,
        );
        node = item.parent;
    }
    symbols.reverse();
    Ok(symbols.into_iter().collect())
}

pub(super) fn reconstruct_compact_center_reverse(
    package: &LexicalGrokkingPackage,
    terminal_id: u32,
) -> Result<Vec<WaveCoupling>, String> {
    let center = *package
        .centers
        .get(terminal_id as usize)
        .ok_or_else(|| "compact depth-0 terminal is invalid".to_string())?;
    let surface = decode_center_surface(center, &package.decoder_nodes)?;
    let resolved = encode_wave_surface(&surface)
        .into_iter()
        .filter_map(|atom| {
            package
                .graph
                .atom_id(atom.key)
                .map(|atom_id| (atom_id, atom.position, atom.key.channel))
        })
        .collect::<Vec<_>>();
    let mut stats = BTreeMap::<u32, (u32, u64, AtomChannel)>::new();
    for (atom_id, position, channel) in &resolved {
        let entry = stats.entry(*atom_id).or_insert((0, 0, *channel));
        entry.0 = entry.0.saturating_add(1);
        entry.1 = entry.1.saturating_add(u64::from(*position));
    }
    let mut reverse = resolved
        .into_iter()
        .map(|(atom_id, position, channel)| {
            let observation_count = stats.get(&atom_id).map(|item| item.0).unwrap_or_default();
            let position_mode = (position / 257).min(255) as u8;
            WaveCoupling {
                peer_id: atom_id,
                strength: reconstructed_coupling_strength(
                    observation_count,
                    u32::from(package.atoms[atom_id as usize].support),
                    package.centers.len(),
                ),
                phase_relation: position_phase(position_mode),
                position_mode,
                flags: if channel == AtomChannel::CharacterAnchor {
                    COUPLING_FLAG_CHARACTER_ANCHOR
                } else {
                    0
                },
            }
        })
        .collect::<Vec<_>>();
    reverse.sort_unstable_by(coupling_order);
    let anchor_count = reverse
        .iter()
        .take_while(|relation| relation.flags != 0)
        .count();
    reverse.truncate(anchor_count.saturating_add(MAX_REVERSE_LEXICAL_COUPLINGS));
    Ok(reverse)
}

fn reconstructed_coupling_strength(observations: u32, atom_support: u32, word_count: usize) -> u8 {
    let reliability = observations.saturating_mul(255);
    let specificity =
        ((word_count as u32 + 1).saturating_mul(32) / atom_support.max(1)).clamp(32, 255);
    ((reliability.saturating_mul(specificity) / 255).clamp(1, 255)) as u8
}

fn position_phase(position: u8) -> i8 {
    (i16::from(position) - 128).clamp(-127, 127) as i8
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

fn l11_extension_bytes(package: &LexicalGrokkingPackage) -> Result<usize, String> {
    if package.center_phase_profiles.len() != package.centers.len() {
        return Err("L1.1 requires exactly one phase profile per primary center".to_string());
    }
    Ok(L11_EXTENSION_HEADER_BYTES
        .saturating_add(
            package
                .center_phase_profiles
                .len()
                .saturating_mul(CENTER_PHASE_PROFILE_BYTES),
        )
        .saturating_add(
            package
                .positive_subcenters
                .len()
                .saturating_mul(WORD_CENTER_BYTES),
        )
        .saturating_add(
            package
                .anti_subcenters
                .len()
                .saturating_mul(WORD_CENTER_BYTES),
        )
        .saturating_add(
            package
                .hard_negative_subcenters
                .len()
                .saturating_mul(WORD_CENTER_BYTES),
        )
        .saturating_add(
            package
                .keyboard_geometry_units
                .len()
                .saturating_mul(std::mem::size_of::<u32>()),
        )
        .saturating_add(
            package
                .ambiguity_subcenters
                .len()
                .saturating_mul(WORD_CENTER_BYTES),
        ))
}

fn write_l11_extension(
    bytes: &mut [u8],
    offset: usize,
    package: &LexicalGrokkingPackage,
) -> Result<(), String> {
    validate_l11_ranges(
        &package.center_phase_profiles,
        package.positive_subcenters.len(),
        package.anti_subcenters.len(),
        package.hard_negative_subcenters.len(),
        package.ambiguity_subcenters.len(),
        &package.keyboard_geometry_units,
        package.centers.len(),
        package.atoms.len(),
    )?;
    bytes[offset..offset + 8].copy_from_slice(L11_EXTENSION_MAGIC);
    put_u32(
        bytes,
        offset + 8,
        as_u32(package.center_phase_profiles.len(), "L1.1 phase profile")?,
    );
    put_u32(
        bytes,
        offset + 12,
        as_u32(package.positive_subcenters.len(), "L1.1 positive subcenter")?,
    );
    put_u32(
        bytes,
        offset + 16,
        as_u32(package.anti_subcenters.len(), "L1.1 anti subcenter")?,
    );
    put_u32(
        bytes,
        offset + 20,
        as_u32(
            package.hard_negative_subcenters.len(),
            "L1.1 hard-negative subcenter",
        )?,
    );
    put_u32(
        bytes,
        offset + 24,
        as_u32(
            package.keyboard_geometry_units.len(),
            "L1.1 keyboard geometry unit",
        )?,
    );
    bytes[offset + 28] = package.restoration_calibration.max_geometry_distance;
    put_u16(
        bytes,
        offset + 30,
        package.restoration_calibration.min_positive_milli,
    );
    put_u16(
        bytes,
        offset + 32,
        package.restoration_calibration.min_backward_milli,
    );
    put_u16(
        bytes,
        offset + 34,
        package.restoration_calibration.min_tied_energy_margin,
    );
    put_u32(
        bytes,
        offset + 36,
        as_u32(
            package.ambiguity_subcenters.len(),
            "L1.1 ambiguity subcenter",
        )?,
    );

    let mut cursor = offset + L11_EXTENSION_HEADER_BYTES;
    for profile in &package.center_phase_profiles {
        put_u32(bytes, cursor, profile.positive_start);
        put_u32(bytes, cursor + 4, profile.anti_start);
        put_u32(bytes, cursor + 8, profile.hard_negative_start);
        put_u32(bytes, cursor + 12, profile.keyboard_geometry_start);
        bytes[cursor + 16] = profile.positive_count;
        bytes[cursor + 17] = profile.anti_count;
        bytes[cursor + 18] = profile.hard_negative_count;
        bytes[cursor + 19] = profile.keyboard_geometry_count;
        bytes[cursor + 20] = profile.flags;
        bytes[cursor + 21] = profile.ambiguity_count;
        put_u16(bytes, cursor + 22, profile.min_ambiguity_milli);
        cursor += CENTER_PHASE_PROFILE_BYTES;
    }
    write_centers(bytes, cursor, &package.positive_subcenters);
    cursor += package.positive_subcenters.len() * WORD_CENTER_BYTES;
    write_centers(bytes, cursor, &package.anti_subcenters);
    cursor += package.anti_subcenters.len() * WORD_CENTER_BYTES;
    write_centers(bytes, cursor, &package.hard_negative_subcenters);
    cursor += package.hard_negative_subcenters.len() * WORD_CENTER_BYTES;
    for atom_id in &package.keyboard_geometry_units {
        put_u32(bytes, cursor, *atom_id);
        cursor += std::mem::size_of::<u32>();
    }
    write_centers(bytes, cursor, &package.ambiguity_subcenters);
    Ok(())
}

#[expect(clippy::type_complexity, reason = "existing explicit type contract")]
fn read_l11_extension(
    bytes: &[u8],
    offset: usize,
    primary_center_count: usize,
    atom_count: usize,
) -> Result<
    (
        Vec<CenterPhaseProfile>,
        Vec<WordCenter64>,
        Vec<WordCenter64>,
        Vec<WordCenter64>,
        Vec<WordCenter64>,
        Vec<u32>,
        RestorationCalibration,
    ),
    String,
> {
    let header = bytes
        .get(offset..offset + L11_EXTENSION_HEADER_BYTES)
        .ok_or_else(|| "truncated L1.1 extension header".to_string())?;
    if header.get(..8) != Some(L11_EXTENSION_MAGIC.as_slice()) {
        return Err("invalid L1.1 extension magic".to_string());
    }
    let profile_count = read_u32(bytes, offset + 8)? as usize;
    let positive_count = read_u32(bytes, offset + 12)? as usize;
    let anti_count = read_u32(bytes, offset + 16)? as usize;
    let hard_negative_count = read_u32(bytes, offset + 20)? as usize;
    let keyboard_geometry_count = read_u32(bytes, offset + 24)? as usize;
    let ambiguity_count = read_u32(bytes, offset + 36)? as usize;
    if profile_count != primary_center_count {
        return Err("L1.1 phase profile count differs from primary centers".to_string());
    }
    let expected_bytes = L11_EXTENSION_HEADER_BYTES
        .checked_add(
            profile_count
                .checked_mul(CENTER_PHASE_PROFILE_BYTES)
                .ok_or_else(|| "L1.1 profile allocation overflow".to_string())?,
        )
        .and_then(|value| {
            value.checked_add(
                positive_count
                    .checked_add(anti_count)?
                    .checked_add(hard_negative_count)?
                    .checked_mul(WORD_CENTER_BYTES)?,
            )
        })
        .and_then(|value| {
            value.checked_add(keyboard_geometry_count.checked_mul(std::mem::size_of::<u32>())?)
        })
        .and_then(|value| value.checked_add(ambiguity_count.checked_mul(WORD_CENTER_BYTES)?))
        .ok_or_else(|| "L1.1 extension allocation overflow".to_string())?;
    if bytes.len() != offset.saturating_add(expected_bytes) {
        return Err("invalid L1.1 extension size or trailing bytes".to_string());
    }

    let calibration = RestorationCalibration {
        max_geometry_distance: header[28],
        min_positive_milli: read_u16(bytes, offset + 30)?,
        min_backward_milli: read_u16(bytes, offset + 32)?,
        // E2 packages written before tied-basin calibration leave these
        // reserved bytes at zero, which keeps crystallization disabled.
        min_tied_energy_margin: read_u16(bytes, offset + 34)?,
    };
    let mut cursor = offset + L11_EXTENSION_HEADER_BYTES;
    let mut ambiguity_start = 0_u32;
    let profiles = (0..profile_count)
        .map(|_| {
            let ambiguity_profile_count = bytes[cursor + 21];
            let profile = CenterPhaseProfile {
                positive_start: read_u32(bytes, cursor)?,
                anti_start: read_u32(bytes, cursor + 4)?,
                hard_negative_start: read_u32(bytes, cursor + 8)?,
                keyboard_geometry_start: read_u32(bytes, cursor + 12)?,
                ambiguity_start,
                positive_count: bytes[cursor + 16],
                anti_count: bytes[cursor + 17],
                hard_negative_count: bytes[cursor + 18],
                keyboard_geometry_count: bytes[cursor + 19],
                flags: bytes[cursor + 20],
                ambiguity_count: ambiguity_profile_count,
                min_ambiguity_milli: read_u16(bytes, cursor + 22)?,
            };
            ambiguity_start = ambiguity_start.saturating_add(u32::from(ambiguity_profile_count));
            cursor += CENTER_PHASE_PROFILE_BYTES;
            Ok(profile)
        })
        .collect::<Result<Vec<_>, String>>()?;
    let positive = read_centers(bytes, cursor, positive_count)?;
    cursor += positive_count * WORD_CENTER_BYTES;
    let anti = read_centers(bytes, cursor, anti_count)?;
    cursor += anti_count * WORD_CENTER_BYTES;
    let hard_negative = read_centers(bytes, cursor, hard_negative_count)?;
    cursor += hard_negative_count * WORD_CENTER_BYTES;
    let keyboard_geometry = (0..keyboard_geometry_count)
        .map(|_| {
            let atom_id = read_u32(bytes, cursor)?;
            cursor += std::mem::size_of::<u32>();
            Ok(atom_id)
        })
        .collect::<Result<Vec<_>, String>>()?;
    let ambiguity = read_centers(bytes, cursor, ambiguity_count)?;
    validate_l11_ranges(
        &profiles,
        positive.len(),
        anti.len(),
        hard_negative.len(),
        ambiguity.len(),
        &keyboard_geometry,
        primary_center_count,
        atom_count,
    )?;
    Ok((
        profiles,
        positive,
        anti,
        hard_negative,
        ambiguity,
        keyboard_geometry,
        calibration,
    ))
}

fn validate_graph(graph: &NGramGraph) -> Result<(), String> {
    for node in &graph.nodes {
        let start = node.first_arc as usize;
        let end = start.saturating_add(node.arc_count as usize);
        let arcs = graph
            .arcs
            .get(start..end)
            .ok_or_else(|| "n-gram node references invalid arcs".to_string())?;
        if !arcs.windows(2).all(|pair| pair[0].symbol < pair[1].symbol)
            || arcs
                .iter()
                .any(|arc| arc.next_node as usize >= graph.nodes.len())
        {
            return Err("invalid n-gram graph topology".to_string());
        }
    }
    Ok(())
}

#[expect(
    clippy::too_many_arguments,
    reason = "sealed format sections remain explicit"
)]
fn validate_ranges(
    atoms: &[AtomRecord],
    centers: &[WordCenter64],
    forward_len: usize,
    reverse_len: usize,
    anti_len: usize,
    pair_profiles: &[PairPhaseProfile],
    pair_center_len: usize,
    decoder_len: usize,
) -> Result<(), String> {
    if atoms
        .iter()
        .any(|atom| atom.coupling_start as usize + atom.coupling_count as usize > forward_len)
        || centers.iter().any(|center| {
            center.coupling_start as usize + center.coupling_count as usize > reverse_len
                || center.anti_start as usize + center.anti_count as usize > anti_len
                || center.decoder_terminal as usize >= decoder_len
        })
        || pair_profiles.iter().any(|profile| {
            profile.low_wins_start as usize + profile.low_wins_count as usize > pair_center_len
                || profile.high_wins_start as usize + profile.high_wins_count as usize
                    > pair_center_len
        })
    {
        return Err("L1 crystal record references invalid range".to_string());
    }
    Ok(())
}

#[expect(
    clippy::too_many_arguments,
    reason = "sealed format sections remain explicit"
)]
fn validate_l11_ranges(
    profiles: &[CenterPhaseProfile],
    positive_len: usize,
    anti_len: usize,
    hard_negative_len: usize,
    ambiguity_len: usize,
    keyboard_geometry: &[u32],
    primary_center_count: usize,
    atom_count: usize,
) -> Result<(), String> {
    if profiles.is_empty() {
        return Ok(());
    }
    if profiles.len() != primary_center_count
        || profiles.iter().any(|profile| {
            profile.positive_start as usize + profile.positive_count as usize > positive_len
                || profile.anti_start as usize + profile.anti_count as usize > anti_len
                || profile.hard_negative_start as usize + profile.hard_negative_count as usize
                    > hard_negative_len
                || profile.ambiguity_start as usize + profile.ambiguity_count as usize
                    > ambiguity_len
                || profile.keyboard_geometry_start as usize
                    + profile.keyboard_geometry_count as usize
                    > keyboard_geometry.len()
                || profile.flags & !CENTER_PHASE_FLAG_PHYSICAL_KEY_GEOMETRY != 0
        })
    {
        return Err("L1.1 phase profile references invalid bank range".to_string());
    }
    let mut expected_ambiguity_start = 0_usize;
    for profile in profiles {
        if profile.ambiguity_start as usize != expected_ambiguity_start {
            return Err("L1.1 ambiguity profiles are not densely ordered".to_string());
        }
        expected_ambiguity_start =
            expected_ambiguity_start.saturating_add(profile.ambiguity_count as usize);
    }
    if expected_ambiguity_start != ambiguity_len {
        return Err("L1.1 ambiguity profile count differs from bank".to_string());
    }
    for profile in profiles {
        if profile.flags & CENTER_PHASE_FLAG_PHYSICAL_KEY_GEOMETRY != 0 {
            continue;
        }
        let start = profile.keyboard_geometry_start as usize;
        let end = start + profile.keyboard_geometry_count as usize;
        if keyboard_geometry[start..end]
            .iter()
            .any(|atom_id| *atom_id as usize >= atom_count)
        {
            return Err("L1.1 phase profile references invalid bank range".to_string());
        }
    }
    Ok(())
}

fn checksum(bytes: &[u8]) -> u64 {
    let mut state = 0xcbf2_9ce4_8422_2325_u64;
    for (index, byte) in bytes.iter().copied().enumerate() {
        let value = if (176..184).contains(&index) { 0 } else { byte };
        state ^= u64::from(value);
        state = state.wrapping_mul(0x100_0000_01b3);
    }
    state
}

#[cfg(test)]
pub(super) fn refresh_checksum(bytes: &mut [u8]) {
    let value = checksum(bytes);
    put_u64(bytes, 176, value);
}

fn as_u32(value: usize, name: &str) -> Result<u32, String> {
    u32::try_from(value).map_err(|_| format!("L1 crystal {name} count exceeds u32"))
}

fn put_u16(bytes: &mut [u8], offset: usize, value: u16) {
    bytes[offset..offset + 2].copy_from_slice(&value.to_le_bytes());
}

fn put_u32(bytes: &mut [u8], offset: usize, value: u32) {
    bytes[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
}

fn put_u64(bytes: &mut [u8], offset: usize, value: u64) {
    bytes[offset..offset + 8].copy_from_slice(&value.to_le_bytes());
}

fn read_u16(bytes: &[u8], offset: usize) -> Result<u16, String> {
    let raw = bytes
        .get(offset..offset + 2)
        .ok_or_else(|| "truncated u16".to_string())?;
    Ok(u16::from_le_bytes([raw[0], raw[1]]))
}

fn read_u32(bytes: &[u8], offset: usize) -> Result<u32, String> {
    let raw = bytes
        .get(offset..offset + 4)
        .ok_or_else(|| "truncated u32".to_string())?;
    Ok(u32::from_le_bytes([raw[0], raw[1], raw[2], raw[3]]))
}

fn read_u64(bytes: &[u8], offset: usize) -> Result<u64, String> {
    let raw = bytes
        .get(offset..offset + 8)
        .ok_or_else(|| "truncated u64".to_string())?;
    Ok(u64::from_le_bytes([
        raw[0], raw[1], raw[2], raw[3], raw[4], raw[5], raw[6], raw[7],
    ]))
}
