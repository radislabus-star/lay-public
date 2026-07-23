use super::crystal::{
    AtomWaveCode, ComplexBasisWave, WordCenter64, WAVE_DIMENSION, WORD_CENTER_BYTES,
};
use super::model::{
    AtomRecord, CenterPhaseProfile, DecoderNode, LexicalGrokkingPackage, PairKey, PairPhaseProfile,
    WaveCoupling, CENTER_PHASE_FLAG_PHYSICAL_KEY_GEOMETRY,
};
use super::ngram_graph::{NGramArc, NGramGraph, NGramNode};
use super::posting_codec::{decode_posting, encode_posting};
use super::restoration::RestorationCalibration;

const MAGIC_V2: &[u8; 8] = b"LAYL1C02";
const MAGIC_V3: &[u8; 8] = b"LAYL1C03";
const MAGIC_V4: &[u8; 8] = b"LAYL1C04";
const MAGIC_V5: &[u8; 8] = b"LAYL1C05";
const VERSION_V2: u32 = 2;
const VERSION_V3: u32 = 3;
const VERSION_V4: u32 = 4;
const VERSION_V5: u32 = 5;
const HEADER_BYTES: usize = 192;
const NODE_BYTES: usize = 12;
const ARC_BYTES: usize = 8;
const BASIS_BYTES: usize = WAVE_DIMENSION * 2;
const ATOM_BYTES: usize = 24;
const COUPLING_BYTES: usize = 8;
const DECODER_NODE_BYTES: usize = 8;
const PAIR_PROFILE_BYTES: usize = 24;
const L11_EXTENSION_MAGIC: &[u8; 8] = b"LAYL11E2";
const L11_EXTENSION_HEADER_BYTES: usize = 40;
const CENTER_PHASE_PROFILE_BYTES: usize = 24;

pub(super) fn encode(package: &LexicalGrokkingPackage) -> Result<Vec<u8>, String> {
    let counts = counts(package)?;
    let (compressed_forward, atom_forward_offsets) = encode_forward(package)?;
    let offsets = offsets_v4(counts, compressed_forward.len());
    let base_bytes = offsets.decoder_nodes + package.decoder_nodes.len() * DECODER_NODE_BYTES;
    let has_l11 = !package.center_phase_profiles.is_empty();
    if !has_l11
        && (!package.positive_subcenters.is_empty()
            || !package.anti_subcenters.is_empty()
            || !package.hard_negative_subcenters.is_empty()
            || !package.keyboard_geometry_units.is_empty())
    {
        return Err("L1.1 extension banks require primary phase profiles".to_string());
    }
    let file_bytes = if has_l11 {
        base_bytes.saturating_add(l11_extension_bytes(package)?)
    } else {
        base_bytes
    };
    let mut bytes = vec![0_u8; file_bytes];
    bytes[..8].copy_from_slice(if has_l11 { MAGIC_V5 } else { MAGIC_V4 });
    put_u32(&mut bytes, 8, if has_l11 { VERSION_V5 } else { VERSION_V4 });
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
    write_atoms(
        &mut bytes,
        offsets.atoms,
        &package.atoms,
        Some(&atom_forward_offsets),
    );
    bytes[offsets.forward..offsets.reverse].copy_from_slice(&compressed_forward);
    write_couplings(&mut bytes, offsets.reverse, &package.reverse_couplings);
    write_centers(&mut bytes, offsets.anti, &package.anti_centers);
    write_pair_profiles(&mut bytes, offsets.pair_profiles, &package.pair_profiles);
    write_centers(&mut bytes, offsets.pair_centers, &package.pair_centers);
    write_centers(&mut bytes, offsets.centers, &package.centers);
    for (index, node) in package.decoder_nodes.iter().copied().enumerate() {
        let start = offsets.decoder_nodes + index * DECODER_NODE_BYTES;
        put_u32(&mut bytes, start, node.parent);
        put_u32(&mut bytes, start + 4, node.symbol);
    }
    if has_l11 {
        write_l11_extension(&mut bytes, base_bytes, package)?;
    }
    let checksum = checksum(&bytes);
    put_u64(&mut bytes, 176, checksum);
    Ok(bytes)
}

pub(super) fn decode(bytes: &[u8]) -> Result<LexicalGrokkingPackage, String> {
    if bytes.len() < HEADER_BYTES {
        return Err("invalid L1 crystal package magic".to_string());
    }
    let version = read_u32(bytes, 8)?;
    let valid_magic = match version {
        VERSION_V2 => bytes.get(..8) == Some(MAGIC_V2.as_slice()),
        VERSION_V3 => bytes.get(..8) == Some(MAGIC_V3.as_slice()),
        VERSION_V4 => bytes.get(..8) == Some(MAGIC_V4.as_slice()),
        VERSION_V5 => bytes.get(..8) == Some(MAGIC_V5.as_slice()),
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
    let mut atoms = read_atoms(bytes, stored_offsets.atoms, counts.atoms as usize)?;
    let forward_couplings = if version == VERSION_V2 {
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
    let reverse_couplings = read_couplings(bytes, stored_offsets.reverse, counts.reverse as usize)?;
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
    let centers = read_centers(bytes, stored_offsets.centers, counts.centers as usize)?;
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
        keyboard_geometry_units,
        restoration_calibration,
    ) = if version >= VERSION_V5 {
        read_l11_extension(bytes, base_bytes, centers.len(), atoms.len())?
    } else {
        (
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
    offsets_with_forward_bytes(counts, counts.forward as usize * COUPLING_BYTES)
}

fn offsets_v3(counts: Counts, forward_bytes: usize) -> Offsets {
    offsets_with_forward_bytes(counts, forward_bytes)
}

fn offsets_v4(counts: Counts, forward_bytes: usize) -> Offsets {
    offsets_with_forward_bytes(counts, forward_bytes)
}

fn offsets_with_forward_bytes(counts: Counts, forward_bytes: usize) -> Offsets {
    let nodes = HEADER_BYTES;
    let arcs = nodes + counts.nodes as usize * NODE_BYTES;
    let basis = arcs + counts.arcs as usize * ARC_BYTES;
    let atoms = basis + counts.basis as usize * BASIS_BYTES;
    let forward = atoms + counts.atoms as usize * ATOM_BYTES;
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
) {
    for (index, atom) in atoms.iter().copied().enumerate() {
        let start = offset + index * ATOM_BYTES;
        bytes[start..start + AtomWaveCode::BYTES].copy_from_slice(&atom.wave_code.encode());
        put_u32(
            bytes,
            start + 16,
            coupling_offsets
                .and_then(|offsets| offsets.get(index).copied())
                .unwrap_or(atom.coupling_start),
        );
        put_u16(bytes, start + 20, atom.coupling_count);
        put_u16(bytes, start + 22, atom.support);
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

fn read_atoms(bytes: &[u8], offset: usize, count: usize) -> Result<Vec<AtomRecord>, String> {
    (0..count)
        .map(|index| {
            let start = offset + index * ATOM_BYTES;
            Ok(AtomRecord {
                wave_code: AtomWaveCode::decode(&bytes[start..start + AtomWaveCode::BYTES])?,
                coupling_start: read_u32(bytes, start + 16)?,
                coupling_count: read_u16(bytes, start + 20)?,
                support: read_u16(bytes, start + 22)?,
            })
        })
        .collect()
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
    Ok(())
}

#[allow(clippy::type_complexity)]
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
        .ok_or_else(|| "L1.1 extension allocation overflow".to_string())?;
    if bytes.len() != offset.saturating_add(expected_bytes) {
        return Err("invalid L1.1 extension size or trailing bytes".to_string());
    }

    let calibration = RestorationCalibration {
        max_geometry_distance: header[28],
        min_positive_milli: read_u16(bytes, offset + 30)?,
        min_backward_milli: read_u16(bytes, offset + 32)?,
    };
    let mut cursor = offset + L11_EXTENSION_HEADER_BYTES;
    let profiles = (0..profile_count)
        .map(|_| {
            let profile = CenterPhaseProfile {
                positive_start: read_u32(bytes, cursor)?,
                anti_start: read_u32(bytes, cursor + 4)?,
                hard_negative_start: read_u32(bytes, cursor + 8)?,
                keyboard_geometry_start: read_u32(bytes, cursor + 12)?,
                positive_count: bytes[cursor + 16],
                anti_count: bytes[cursor + 17],
                hard_negative_count: bytes[cursor + 18],
                keyboard_geometry_count: bytes[cursor + 19],
                flags: bytes[cursor + 20],
            };
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
    validate_l11_ranges(
        &profiles,
        positive.len(),
        anti.len(),
        hard_negative.len(),
        &keyboard_geometry,
        primary_center_count,
        atom_count,
    )?;
    Ok((
        profiles,
        positive,
        anti,
        hard_negative,
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

fn validate_l11_ranges(
    profiles: &[CenterPhaseProfile],
    positive_len: usize,
    anti_len: usize,
    hard_negative_len: usize,
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
                || profile.keyboard_geometry_start as usize
                    + profile.keyboard_geometry_count as usize
                    > keyboard_geometry.len()
                || profile.flags & !CENTER_PHASE_FLAG_PHYSICAL_KEY_GEOMETRY != 0
        })
    {
        return Err("L1.1 phase profile references invalid bank range".to_string());
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
