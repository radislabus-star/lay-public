use std::cmp::Ordering;
use std::collections::HashMap;
use std::path::Path;

#[cfg(test)]
use std::collections::BTreeMap;

use super::compositional::{
    prepared_similarity_to_normalized_surface_with_workspace_milli, surface_scoring_profile,
    SurfaceGeometryWorkspace,
};
use super::context::context_evidence_lanes;
#[cfg(test)]
use super::context::{context_evidence_keys, scoped_context_evidence_key};
use super::package_bytes::PackageBytes;
use super::productive::{
    apply_edge_transform_prepared, context_pair_evidence_from, productive_birth_rank,
    productive_family_lane_starts, productive_family_suffixes, productive_slot_features_for_scope,
    retain_best_productive_birth, PreparedProductiveGeneration, ProductiveBirthStatus,
    ProductiveContextPairEvidence, ProductiveContextSlotEvidence, ProductiveFormBirth,
    ProductiveIndexReport, ProductiveMorphologyIndex, ProductiveMorphologySource,
};

const MAGIC: &[u8; 8] = b"LAYL2P01";
const LEGACY_VERSION: u32 = 1;
const VERSION: u32 = 2;
const LEGACY_HEADER_BYTES: usize = 128;
const HEADER_BYTES: usize = 160;
const RULE_BYTES: usize = 44;
const TARGET_BYTES: usize = 12;
const CONTEXT_SLOT_BYTES: usize = 20;
const KNOWN_CONTEXT_BYTES: usize = 8;
const CONTEXT_PAIR_BYTES: usize = 16;

#[derive(Clone, Debug, PartialEq, Eq)]
struct Header {
    version: u32,
    header_bytes: usize,
    l2_fingerprint: u64,
    rule_count: usize,
    target_count: usize,
    context_slot_count: usize,
    known_context_count: usize,
    context_pair_count: usize,
    payload_bytes: usize,
    rule_offset: usize,
    target_offset: usize,
    context_slot_offset: usize,
    known_context_offset: usize,
    context_pair_offset: usize,
    payload_offset: usize,
    report: ProductiveIndexReport,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) struct ProductiveFormatStats {
    pub(super) version: u32,
    pub(super) rules: usize,
    pub(super) target_features: usize,
    pub(super) context_slots: usize,
    pub(super) known_contexts: usize,
    pub(super) context_pairs: usize,
    pub(super) payload_bytes: usize,
}

#[derive(Clone, Debug)]
pub(super) struct CompactProductiveMorphologyView {
    backing: PackageBytes,
    header: Header,
    rule_transition_ranges: std::sync::Arc<[RuleTransitionRange]>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct RuleTransitionRange {
    primary_pos: u16,
    source_feature_mask: u32,
    target_feature_mask: u32,
    start: u32,
    end: u32,
}

impl RuleTransitionRange {
    fn key(self) -> (u16, u32, u32) {
        (
            self.primary_pos,
            self.source_feature_mask,
            self.target_feature_mask,
        )
    }
}

#[derive(Clone, Copy)]
struct RuleView<'a> {
    family_specificity: u8,
    remove_prefix_chars: u8,
    prepend: &'a str,
    remove_chars: u8,
    append: &'a str,
    positive_support: u32,
    anti_support: u32,
}

pub(super) fn encode_index(
    index: &ProductiveMorphologyIndex,
    l2_fingerprint: u64,
) -> Result<(Vec<u8>, ProductiveFormatStats), String> {
    let rule_count = index
        .rules_by_family
        .values()
        .try_fold(0_usize, |count, rules| {
            count
                .checked_add(rules.len())
                .ok_or_else(|| "productive rule count overflow".to_string())
        })?;
    let target_count =
        index
            .target_features_by_source
            .values()
            .try_fold(0_usize, |count, targets| {
                count
                    .checked_add(targets.len())
                    .ok_or_else(|| "productive target count overflow".to_string())
            })?;
    let rule_offset = HEADER_BYTES;
    let target_offset = checked_section_end(rule_offset, rule_count, RULE_BYTES)?;
    let context_slot_offset = checked_section_end(target_offset, target_count, TARGET_BYTES)?;
    let known_context_offset = checked_section_end(
        context_slot_offset,
        index.context_slots.len(),
        CONTEXT_SLOT_BYTES,
    )?;
    let context_pair_offset = checked_section_end(
        known_context_offset,
        index.known_contexts.len(),
        KNOWN_CONTEXT_BYTES,
    )?;
    let payload_offset = checked_section_end(
        context_pair_offset,
        index.context_pair_support.len(),
        CONTEXT_PAIR_BYTES,
    )?;

    let mut rule_bytes = Vec::with_capacity(rule_count.saturating_mul(RULE_BYTES));
    let mut payload = Vec::new();
    for ((primary_pos, source_feature_mask, target_feature_mask, family_suffix), rules) in
        &index.rules_by_family
    {
        for rule in rules {
            let family = append_payload(&mut payload, family_suffix)?;
            let prepend = append_payload(&mut payload, &rule.prepend)?;
            let append = append_payload(&mut payload, &rule.append)?;
            put_u16(&mut rule_bytes, *primary_pos);
            rule_bytes.push(family_suffix.chars().count().min(u8::MAX as usize) as u8);
            rule_bytes.push(rule.remove_prefix_chars);
            rule_bytes.push(rule.remove_chars);
            rule_bytes.extend_from_slice(&[0; 3]);
            put_u32(&mut rule_bytes, *source_feature_mask);
            put_u32(&mut rule_bytes, *target_feature_mask);
            put_u32(&mut rule_bytes, rule.positive_support);
            put_u32(&mut rule_bytes, rule.anti_support);
            put_u32(&mut rule_bytes, family.0);
            put_u16(&mut rule_bytes, family.1);
            put_u16(&mut rule_bytes, prepend.1);
            put_u32(&mut rule_bytes, prepend.0);
            put_u32(&mut rule_bytes, append.0);
            put_u16(&mut rule_bytes, append.1);
            put_u16(&mut rule_bytes, 0);
        }
    }
    if rule_bytes.len() != rule_count.saturating_mul(RULE_BYTES) {
        return Err("productive sidecar rule encoding length mismatch".to_string());
    }

    let mut target_bytes = Vec::with_capacity(target_count.saturating_mul(TARGET_BYTES));
    for ((primary_pos, source_feature_mask), targets) in &index.target_features_by_source {
        for target in targets {
            put_u16(&mut target_bytes, *primary_pos);
            put_u16(&mut target_bytes, 0);
            put_u32(&mut target_bytes, *source_feature_mask);
            put_u32(&mut target_bytes, *target);
        }
    }

    let mut context_slot_bytes =
        Vec::with_capacity(index.context_slots.len().saturating_mul(CONTEXT_SLOT_BYTES));
    for ((context_key, slot), evidence) in &index.context_slots {
        put_u32(&mut context_slot_bytes, *context_key);
        put_u32(&mut context_slot_bytes, *slot);
        put_u32(&mut context_slot_bytes, evidence.positive_support);
        put_u32(
            &mut context_slot_bytes,
            evidence.unlabeled_alternative_support,
        );
        put_u16(&mut context_slot_bytes, evidence.posterior_milli);
        put_u16(
            &mut context_slot_bytes,
            u16::from(evidence.context_observed),
        );
    }

    let mut known_context_bytes = Vec::with_capacity(
        index
            .known_contexts
            .len()
            .saturating_mul(KNOWN_CONTEXT_BYTES),
    );
    for (context_key, support) in &index.known_contexts {
        put_u32(&mut known_context_bytes, *context_key);
        put_u32(&mut known_context_bytes, *support);
    }

    let mut context_pair_bytes = Vec::with_capacity(
        index
            .context_pair_support
            .len()
            .saturating_mul(CONTEXT_PAIR_BYTES),
    );
    for ((context_key, preferred_slot, competitor_slot), support) in &index.context_pair_support {
        put_u32(&mut context_pair_bytes, *context_key);
        put_u32(&mut context_pair_bytes, *preferred_slot);
        put_u32(&mut context_pair_bytes, *competitor_slot);
        put_u32(&mut context_pair_bytes, *support);
    }

    let total_bytes = payload_offset
        .checked_add(payload.len())
        .ok_or_else(|| "productive sidecar exceeds address space".to_string())?;
    let mut bytes = vec![0_u8; HEADER_BYTES];
    bytes.extend_from_slice(&rule_bytes);
    bytes.extend_from_slice(&target_bytes);
    bytes.extend_from_slice(&context_slot_bytes);
    bytes.extend_from_slice(&known_context_bytes);
    bytes.extend_from_slice(&context_pair_bytes);
    bytes.extend_from_slice(&payload);
    if bytes.len() != total_bytes {
        return Err("productive sidecar section length mismatch".to_string());
    }

    bytes[..8].copy_from_slice(MAGIC);
    write_u32(&mut bytes, 8, VERSION)?;
    write_u32(&mut bytes, 12, HEADER_BYTES as u32)?;
    write_u64(&mut bytes, 16, l2_fingerprint)?;
    write_u32(&mut bytes, 24, as_u32(rule_count, "rule count")?)?;
    write_u32(&mut bytes, 28, as_u32(target_count, "target count")?)?;
    write_u32(
        &mut bytes,
        32,
        as_u32(index.context_slots.len(), "context slot count")?,
    )?;
    write_u32(
        &mut bytes,
        36,
        as_u32(index.known_contexts.len(), "known context count")?,
    )?;
    write_u32(&mut bytes, 40, as_u32(payload.len(), "payload bytes")?)?;
    write_u32(
        &mut bytes,
        44,
        as_u32(index.context_pair_support.len(), "context pair count")?,
    )?;
    write_u64(&mut bytes, 48, rule_offset as u64)?;
    write_u64(&mut bytes, 56, target_offset as u64)?;
    write_u64(&mut bytes, 64, context_slot_offset as u64)?;
    write_u64(&mut bytes, 72, known_context_offset as u64)?;
    write_u64(&mut bytes, 80, payload_offset as u64)?;
    write_u64(&mut bytes, 128, context_pair_offset as u64)?;
    write_report(&mut bytes, &index.report)?;

    Ok((
        bytes,
        ProductiveFormatStats {
            version: VERSION,
            rules: rule_count,
            target_features: target_count,
            context_slots: index.context_slots.len(),
            known_contexts: index.known_contexts.len(),
            context_pairs: index.context_pair_support.len(),
            payload_bytes: payload.len(),
        },
    ))
}

impl CompactProductiveMorphologyView {
    pub(super) fn load(path: &Path) -> Result<Self, String> {
        Self::from_backing(PackageBytes::load(path)?)
    }

    pub(super) fn from_bytes(bytes: Vec<u8>) -> Result<Self, String> {
        Self::from_backing(PackageBytes::from_vec(bytes))
    }

    fn from_backing(backing: PackageBytes) -> Result<Self, String> {
        let header = decode_header(backing.as_slice())?;
        validate_sections(backing.as_slice(), &header)?;
        let rule_transition_ranges =
            index_rule_transition_ranges(backing.as_slice(), &header)?.into();
        Ok(Self {
            backing,
            header,
            rule_transition_ranges,
        })
    }

    pub(super) fn l2_fingerprint(&self) -> u64 {
        self.header.l2_fingerprint
    }

    pub(super) fn backing_bytes(&self) -> usize {
        self.backing.len()
    }

    pub(super) fn mmap_backed(&self) -> bool {
        self.backing.is_mapped()
    }

    pub(super) fn report(&self) -> ProductiveIndexReport {
        self.header.report.clone()
    }

    fn rule_range(
        &self,
        primary_pos: u16,
        source_feature_mask: u32,
        target_feature_mask: u32,
        family_suffix: &str,
    ) -> std::ops::Range<usize> {
        let transition_key = (primary_pos, source_feature_mask, target_feature_mask);
        let Ok(transition_index) = self
            .rule_transition_ranges
            .binary_search_by_key(&transition_key, |range| range.key())
        else {
            return 0..0;
        };
        let transition = self.rule_transition_ranges[transition_index];
        let key = (
            primary_pos,
            source_feature_mask,
            target_feature_mask,
            family_suffix,
        );
        let mut low = transition.start as usize;
        let mut high = transition.end as usize;
        while low < high {
            let middle = low + (high - low) / 2;
            if self
                .compare_rule_key(middle, key)
                .is_some_and(|ordering| ordering == Ordering::Less)
            {
                low = middle + 1;
            } else {
                high = middle;
            }
        }
        let start = low;
        while low < transition.end as usize
            && self
                .compare_rule_key(low, key)
                .is_some_and(|ordering| ordering == Ordering::Equal)
        {
            low += 1;
        }
        start..low
    }

    fn compare_rule_key(&self, index: usize, right: (u16, u32, u32, &str)) -> Option<Ordering> {
        let record = self.rule_record(index)?;
        let family = self.payload_str(
            read_u32(record, 24)? as usize,
            read_u16(record, 28)? as usize,
        )?;
        Some(
            (
                read_u16(record, 0)?,
                read_u32(record, 8)?,
                read_u32(record, 12)?,
                family,
            )
                .cmp(&right),
        )
    }

    fn rule(&self, index: usize) -> Option<RuleView<'_>> {
        let record = self.rule_record(index)?;
        Some(RuleView {
            family_specificity: *record.get(2)?,
            remove_prefix_chars: *record.get(3)?,
            prepend: self.payload_str(
                read_u32(record, 32)? as usize,
                read_u16(record, 30)? as usize,
            )?,
            remove_chars: *record.get(4)?,
            append: self.payload_str(
                read_u32(record, 36)? as usize,
                read_u16(record, 40)? as usize,
            )?,
            positive_support: read_u32(record, 16)?,
            anti_support: read_u32(record, 20)?,
        })
    }

    fn rule_record(&self, index: usize) -> Option<&[u8]> {
        fixed_record(
            self.backing.as_slice(),
            self.header.rule_offset,
            self.header.rule_count,
            RULE_BYTES,
            index,
        )
    }

    fn payload_str(&self, offset: usize, length: usize) -> Option<&str> {
        let start = self.header.payload_offset.checked_add(offset)?;
        let end = start.checked_add(length)?;
        std::str::from_utf8(self.backing.as_slice().get(start..end)?).ok()
    }

    fn context_evidence(
        &self,
        context_key: u32,
        slot: u32,
    ) -> Option<ProductiveContextSlotEvidence> {
        let mut low = 0_usize;
        let mut high = self.header.context_slot_count;
        while low < high {
            let middle = low + (high - low) / 2;
            let record = fixed_record(
                self.backing.as_slice(),
                self.header.context_slot_offset,
                self.header.context_slot_count,
                CONTEXT_SLOT_BYTES,
                middle,
            )?;
            match (read_u32(record, 0)?, read_u32(record, 4)?).cmp(&(context_key, slot)) {
                Ordering::Less => low = middle + 1,
                Ordering::Greater => high = middle,
                Ordering::Equal => {
                    return Some(ProductiveContextSlotEvidence {
                        positive_support: read_u32(record, 8)?,
                        unlabeled_alternative_support: read_u32(record, 12)?,
                        posterior_milli: read_u16(record, 16)?,
                        context_observed: read_u16(record, 18)? != 0,
                    });
                }
            }
        }
        None
    }

    fn known_context_support(&self, context_key: u32) -> Option<u32> {
        let mut low = 0_usize;
        let mut high = self.header.known_context_count;
        while low < high {
            let middle = low + (high - low) / 2;
            let record = fixed_record(
                self.backing.as_slice(),
                self.header.known_context_offset,
                self.header.known_context_count,
                KNOWN_CONTEXT_BYTES,
                middle,
            )?;
            match read_u32(record, 0)?.cmp(&context_key) {
                Ordering::Less => low = middle + 1,
                Ordering::Greater => high = middle,
                Ordering::Equal => return read_u32(record, 4),
            }
        }
        None
    }

    fn context_pair_support(
        &self,
        context_key: u32,
        preferred_slot: u32,
        competitor_slot: u32,
    ) -> u32 {
        let key = (context_key, preferred_slot, competitor_slot);
        let mut low = 0_usize;
        let mut high = self.header.context_pair_count;
        while low < high {
            let middle = low + (high - low) / 2;
            let Some(record) = fixed_record(
                self.backing.as_slice(),
                self.header.context_pair_offset,
                self.header.context_pair_count,
                CONTEXT_PAIR_BYTES,
                middle,
            ) else {
                return 0;
            };
            let Some(record_key) = read_u32(record, 0)
                .zip(read_u32(record, 4))
                .zip(read_u32(record, 8))
                .map(|((context_key, preferred_slot), competitor_slot)| {
                    (context_key, preferred_slot, competitor_slot)
                })
            else {
                return 0;
            };
            match record_key.cmp(&key) {
                Ordering::Less => low = middle + 1,
                Ordering::Greater => high = middle,
                Ordering::Equal => return read_u32(record, 12).unwrap_or_default(),
            }
        }
        0
    }
}

fn index_rule_transition_ranges(
    bytes: &[u8],
    header: &Header,
) -> Result<Vec<RuleTransitionRange>, String> {
    let mut ranges = Vec::<RuleTransitionRange>::new();
    let mut current_key = None::<(u16, u32, u32)>;
    let mut current_start = 0_usize;
    for index in 0..header.rule_count {
        let record = fixed_record(
            bytes,
            header.rule_offset,
            header.rule_count,
            RULE_BYTES,
            index,
        )
        .ok_or_else(|| format!("missing productive rule record {index}"))?;
        let key = (
            read_u16(record, 0).ok_or("missing productive rule POS")?,
            read_u32(record, 8).ok_or("missing productive source feature mask")?,
            read_u32(record, 12).ok_or("missing productive target feature mask")?,
        );
        if let Some(previous) = current_key {
            if key < previous {
                return Err("productive rule transitions are not sorted".to_string());
            }
            if key != previous {
                ranges.push(RuleTransitionRange {
                    primary_pos: previous.0,
                    source_feature_mask: previous.1,
                    target_feature_mask: previous.2,
                    start: u32::try_from(current_start)
                        .map_err(|_| "productive rule transition start overflow")?,
                    end: u32::try_from(index)
                        .map_err(|_| "productive rule transition end overflow")?,
                });
                current_start = index;
            }
        }
        current_key = Some(key);
    }
    if let Some(key) = current_key {
        ranges.push(RuleTransitionRange {
            primary_pos: key.0,
            source_feature_mask: key.1,
            target_feature_mask: key.2,
            start: u32::try_from(current_start)
                .map_err(|_| "productive rule transition start overflow")?,
            end: u32::try_from(header.rule_count)
                .map_err(|_| "productive rule transition end overflow")?,
        });
    }
    Ok(ranges)
}

impl ProductiveMorphologySource for CompactProductiveMorphologyView {
    fn target_features_vec(&self, primary_pos: u16, source_feature_mask: u32) -> Vec<u32> {
        let mut low = 0_usize;
        let mut high = self.header.target_count;
        while low < high {
            let middle = low + (high - low) / 2;
            let Some(record) = fixed_record(
                self.backing.as_slice(),
                self.header.target_offset,
                self.header.target_count,
                TARGET_BYTES,
                middle,
            ) else {
                return Vec::new();
            };
            let Some(key) = read_u16(record, 0).zip(read_u32(record, 4)) else {
                return Vec::new();
            };
            if key < (primary_pos, source_feature_mask) {
                low = middle + 1;
            } else {
                high = middle;
            }
        }
        let mut targets = Vec::new();
        while low < self.header.target_count {
            let Some(record) = fixed_record(
                self.backing.as_slice(),
                self.header.target_offset,
                self.header.target_count,
                TARGET_BYTES,
                low,
            ) else {
                break;
            };
            if read_u16(record, 0).zip(read_u32(record, 4))
                != Some((primary_pos, source_feature_mask))
            {
                break;
            }
            if let Some(target) = read_u32(record, 8) {
                targets.push(target);
            }
            low += 1;
        }
        targets
    }

    fn context_slot_evidence_for(
        &self,
        context: &str,
        target_feature_mask: u32,
    ) -> ProductiveContextSlotEvidence {
        let primary_pos =
            crate::nanda_wave::morphology_phase::feature_primary_pos(target_feature_mask);
        for lane in context_evidence_lanes(context) {
            let slot = productive_slot_features_for_scope(target_feature_mask, lane.scope);
            for context_key in [
                super::context::scoped_context_evidence_key(lane.key, primary_pos),
                lane.key,
            ] {
                if let Some(context_total) = self.known_context_support(context_key) {
                    return self.context_evidence(context_key, slot).unwrap_or_else(|| {
                        ProductiveContextSlotEvidence {
                            positive_support: 0,
                            unlabeled_alternative_support: context_total,
                            posterior_milli: laplace_posterior_milli(0, context_total),
                            context_observed: true,
                        }
                    });
                }
            }
        }
        ProductiveContextSlotEvidence {
            positive_support: 0,
            unlabeled_alternative_support: 0,
            posterior_milli: laplace_posterior_milli(0, 0),
            context_observed: false,
        }
    }

    fn context_pair_evidence_for(
        &self,
        context: &str,
        preferred_feature_mask: u32,
        competitor_feature_mask: u32,
    ) -> ProductiveContextPairEvidence {
        context_pair_evidence_from(
            context,
            preferred_feature_mask,
            competitor_feature_mask,
            |context_key| self.known_context_support(context_key).is_some(),
            |context_key, preferred_slot, competitor_slot| {
                self.context_pair_support(context_key, preferred_slot, competitor_slot)
            },
        )
    }

    fn generate_forms(
        &self,
        observed_surface: &str,
        primary_pos: u16,
        source_surface: &str,
        source_feature_mask: u32,
        target_feature_mask: u32,
        limit: usize,
    ) -> Vec<ProductiveFormBirth> {
        let source_chars = source_surface.chars().collect::<Vec<_>>();
        let observed_profile = surface_scoring_profile(observed_surface);
        let family_suffixes = productive_family_suffixes(&source_chars);
        let family_lane_starts = productive_family_lane_starts(source_surface, observed_surface);
        let mut geometry_by_surface = HashMap::new();
        let mut geometry_workspace = SurfaceGeometryWorkspace::default();
        self.generate_forms_prepared(
            &PreparedProductiveGeneration {
                observed_surface,
                observed_profile: &observed_profile,
                source_surface,
                source_chars: &source_chars,
                family_suffixes: &family_suffixes,
                family_lane_starts: &family_lane_starts,
            },
            primary_pos,
            source_feature_mask,
            target_feature_mask,
            limit,
            &mut geometry_by_surface,
            &mut geometry_workspace,
        )
    }

    fn generate_forms_prepared(
        &self,
        prepared: &PreparedProductiveGeneration<'_>,
        primary_pos: u16,
        source_feature_mask: u32,
        target_feature_mask: u32,
        limit: usize,
        geometry_by_surface: &mut HashMap<String, u16>,
        geometry_workspace: &mut SurfaceGeometryWorkspace,
    ) -> Vec<ProductiveFormBirth> {
        if limit == 0 {
            return Vec::new();
        }
        let mut by_surface = Vec::<ProductiveFormBirth>::new();
        for &lane_start in prepared.family_lane_starts {
            for specificity in (0..=lane_start).rev() {
                let Some(family_suffix) = prepared.family_suffixes.get(specificity) else {
                    continue;
                };
                let range = self.rule_range(
                    primary_pos,
                    source_feature_mask,
                    target_feature_mask,
                    family_suffix,
                );
                if range.is_empty() {
                    continue;
                }
                for index in range {
                    let Some(rule) = self.rule(index) else {
                        continue;
                    };
                    let Some(surface) = apply_edge_transform_prepared(
                        prepared.source_surface,
                        prepared.source_chars,
                        usize::from(rule.remove_prefix_chars),
                        rule.prepend,
                        usize::from(rule.remove_chars),
                        rule.append,
                    ) else {
                        continue;
                    };
                    if surface == prepared.source_surface || surface.is_empty() {
                        continue;
                    }
                    let geometry_evidence_milli =
                        if let Some(geometry) = geometry_by_surface.get(&surface) {
                            *geometry
                        } else {
                            let geometry =
                                prepared_similarity_to_normalized_surface_with_workspace_milli(
                                    prepared.observed_profile,
                                    &surface,
                                    geometry_workspace,
                                );
                            geometry_by_surface.insert(surface.clone(), geometry);
                            geometry
                        };
                    let candidate = ProductiveFormBirth {
                        surface,
                        source_feature_mask,
                        target_feature_mask,
                        geometry_evidence_milli,
                        profile_evidence_milli: laplace_posterior_milli(
                            rule.positive_support,
                            rule.anti_support,
                        ),
                        positive_support: rule.positive_support,
                        anti_support: rule.anti_support,
                        family_specificity: rule.family_specificity,
                        status: ProductiveBirthStatus::ShadowUnverified,
                    };
                    retain_best_productive_birth(&mut by_surface, candidate);
                }
                break;
            }
        }
        if by_surface.is_empty() {
            return Vec::new();
        }
        let mut births = by_surface;
        births.sort_by(|left, right| {
            productive_birth_rank(right)
                .cmp(&productive_birth_rank(left))
                .then_with(|| left.surface.cmp(&right.surface))
        });
        births.truncate(limit);
        births
    }
}

fn decode_header(bytes: &[u8]) -> Result<Header, String> {
    if bytes.len() < LEGACY_HEADER_BYTES || bytes.get(..8) != Some(MAGIC) {
        return Err("invalid productive morphology sidecar magic".to_string());
    }
    let version = read_u32(bytes, 8).ok_or("missing productive format version")?;
    let header_bytes = read_usize_u32(bytes, 12)?;
    if !matches!(
        (version, header_bytes),
        (LEGACY_VERSION, LEGACY_HEADER_BYTES) | (VERSION, HEADER_BYTES)
    ) {
        return Err("unsupported productive morphology sidecar version".to_string());
    }
    Ok(Header {
        version,
        header_bytes,
        l2_fingerprint: read_u64(bytes, 16).ok_or("missing L2 fingerprint")?,
        rule_count: read_usize_u32(bytes, 24)?,
        target_count: read_usize_u32(bytes, 28)?,
        context_slot_count: read_usize_u32(bytes, 32)?,
        known_context_count: read_usize_u32(bytes, 36)?,
        context_pair_count: if version >= VERSION {
            read_usize_u32(bytes, 44)?
        } else {
            0
        },
        payload_bytes: read_usize_u32(bytes, 40)?,
        rule_offset: read_usize_u64(bytes, 48)?,
        target_offset: read_usize_u64(bytes, 56)?,
        context_slot_offset: read_usize_u64(bytes, 64)?,
        known_context_offset: read_usize_u64(bytes, 72)?,
        context_pair_offset: if version >= VERSION {
            read_usize_u64(bytes, 128)?
        } else {
            read_usize_u64(bytes, 80)?
        },
        payload_offset: read_usize_u64(bytes, 80)?,
        report: read_report(bytes, version)?,
    })
}

fn validate_sections(bytes: &[u8], header: &Header) -> Result<(), String> {
    let target_offset = checked_section_end(header.rule_offset, header.rule_count, RULE_BYTES)?;
    let context_slot_offset =
        checked_section_end(header.target_offset, header.target_count, TARGET_BYTES)?;
    let known_context_offset = checked_section_end(
        header.context_slot_offset,
        header.context_slot_count,
        CONTEXT_SLOT_BYTES,
    )?;
    let context_pair_offset = checked_section_end(
        header.known_context_offset,
        header.known_context_count,
        KNOWN_CONTEXT_BYTES,
    )?;
    let payload_offset = checked_section_end(
        header.context_pair_offset,
        header.context_pair_count,
        CONTEXT_PAIR_BYTES,
    )?;
    let end = header
        .payload_offset
        .checked_add(header.payload_bytes)
        .ok_or("productive sidecar length overflow")?;
    if header.rule_offset != header.header_bytes
        || header.target_offset != target_offset
        || header.context_slot_offset != context_slot_offset
        || header.known_context_offset != known_context_offset
        || header.context_pair_offset != context_pair_offset
        || header.payload_offset != payload_offset
        || end != bytes.len()
    {
        return Err("productive sidecar section bounds are inconsistent".to_string());
    }

    Ok(())
}

fn append_payload(payload: &mut Vec<u8>, value: &str) -> Result<(u32, u16), String> {
    let offset = as_u32(payload.len(), "payload offset")?;
    let length = u16::try_from(value.len())
        .map_err(|_| "productive payload string exceeds u16".to_string())?;
    payload.extend_from_slice(value.as_bytes());
    Ok((offset, length))
}

fn fixed_record(
    bytes: &[u8],
    offset: usize,
    count: usize,
    width: usize,
    index: usize,
) -> Option<&[u8]> {
    if index >= count {
        return None;
    }
    let start = offset.checked_add(index.checked_mul(width)?)?;
    bytes.get(start..start.checked_add(width)?)
}

fn checked_section_end(offset: usize, count: usize, width: usize) -> Result<usize, String> {
    offset
        .checked_add(
            count
                .checked_mul(width)
                .ok_or("productive section size overflow")?,
        )
        .ok_or_else(|| "productive section offset overflow".to_string())
}

fn laplace_posterior_milli(positive_support: u32, anti_support: u32) -> u16 {
    let numerator = u64::from(positive_support).saturating_add(1) * 1_000;
    let denominator = u64::from(positive_support)
        .saturating_add(u64::from(anti_support))
        .saturating_add(2);
    (numerator / denominator).min(1_000) as u16
}

fn write_report(bytes: &mut [u8], report: &ProductiveIndexReport) -> Result<(), String> {
    for (offset, value, label) in [
        (88, report.observed_lemmas, "observed lemmas"),
        (92, report.admitted_lemmas, "admitted lemmas"),
        (96, report.observed_transforms, "observed transforms"),
        (100, report.admitted_profiles, "admitted profiles"),
        (
            104,
            report.rejected_low_support_profiles,
            "rejected profiles",
        ),
        (108, report.observed_context_rows, "observed context rows"),
        (112, report.admitted_context_rows, "admitted context rows"),
        (116, report.excluded_context_rows, "excluded context rows"),
        (120, report.rejected_context_rows, "rejected context rows"),
        (
            136,
            report.observed_competitor_rows,
            "observed competitor rows",
        ),
        (
            140,
            report.observed_competitor_surfaces,
            "observed competitor surfaces",
        ),
        (
            144,
            report.same_lemma_competitor_surfaces,
            "same-lemma competitor surfaces",
        ),
        (
            148,
            report.admitted_pair_observations,
            "admitted pair observations",
        ),
        (152, report.context_pairs, "context pairs"),
    ] {
        write_u32(bytes, offset, as_u32(value, label)?)?;
    }
    Ok(())
}

fn read_report(bytes: &[u8], version: u32) -> Result<ProductiveIndexReport, String> {
    Ok(ProductiveIndexReport {
        observed_lemmas: read_usize_u32(bytes, 88)?,
        admitted_lemmas: read_usize_u32(bytes, 92)?,
        observed_transforms: read_usize_u32(bytes, 96)?,
        admitted_profiles: read_usize_u32(bytes, 100)?,
        rejected_low_support_profiles: read_usize_u32(bytes, 104)?,
        observed_context_rows: read_usize_u32(bytes, 108)?,
        admitted_context_rows: read_usize_u32(bytes, 112)?,
        excluded_context_rows: read_usize_u32(bytes, 116)?,
        rejected_context_rows: read_usize_u32(bytes, 120)?,
        context_modes: read_usize_u32(bytes, 36)?,
        context_slots: read_usize_u32(bytes, 32)?,
        observed_competitor_rows: if version >= VERSION {
            read_usize_u32(bytes, 136)?
        } else {
            0
        },
        observed_competitor_surfaces: if version >= VERSION {
            read_usize_u32(bytes, 140)?
        } else {
            0
        },
        same_lemma_competitor_surfaces: if version >= VERSION {
            read_usize_u32(bytes, 144)?
        } else {
            0
        },
        admitted_pair_observations: if version >= VERSION {
            read_usize_u32(bytes, 148)?
        } else {
            0
        },
        context_pairs: if version >= VERSION {
            read_usize_u32(bytes, 152)?
        } else {
            0
        },
    })
}

fn as_u32(value: usize, label: &str) -> Result<u32, String> {
    u32::try_from(value).map_err(|_| format!("productive {label} exceeds u32"))
}

fn put_u16(bytes: &mut Vec<u8>, value: u16) {
    bytes.extend_from_slice(&value.to_le_bytes());
}

fn put_u32(bytes: &mut Vec<u8>, value: u32) {
    bytes.extend_from_slice(&value.to_le_bytes());
}

fn write_u32(bytes: &mut [u8], offset: usize, value: u32) -> Result<(), String> {
    bytes
        .get_mut(offset..offset + 4)
        .ok_or_else(|| "productive header write overflow".to_string())?
        .copy_from_slice(&value.to_le_bytes());
    Ok(())
}

fn write_u64(bytes: &mut [u8], offset: usize, value: u64) -> Result<(), String> {
    bytes
        .get_mut(offset..offset + 8)
        .ok_or_else(|| "productive header write overflow".to_string())?
        .copy_from_slice(&value.to_le_bytes());
    Ok(())
}

fn read_u16(bytes: &[u8], offset: usize) -> Option<u16> {
    Some(u16::from_le_bytes(
        bytes.get(offset..offset + 2)?.try_into().ok()?,
    ))
}

fn read_u32(bytes: &[u8], offset: usize) -> Option<u32> {
    Some(u32::from_le_bytes(
        bytes.get(offset..offset + 4)?.try_into().ok()?,
    ))
}

fn read_u64(bytes: &[u8], offset: usize) -> Option<u64> {
    Some(u64::from_le_bytes(
        bytes.get(offset..offset + 8)?.try_into().ok()?,
    ))
}

fn read_usize_u32(bytes: &[u8], offset: usize) -> Result<usize, String> {
    Ok(read_u32(bytes, offset).ok_or("missing productive u32")? as usize)
}

fn read_usize_u64(bytes: &[u8], offset: usize) -> Result<usize, String> {
    usize::try_from(read_u64(bytes, offset).ok_or("missing productive u64")?)
        .map_err(|_| "productive offset exceeds address space".to_string())
}

#[cfg(test)]
mod tests {
    use super::super::productive::ProductiveRule;
    use super::*;

    fn fixture() -> ProductiveMorphologyIndex {
        let source = crate::nanda_wave::morphology_phase::parse_features("noun:nom:sg")
            .expect("source features");
        let target = crate::nanda_wave::morphology_phase::parse_features("noun:gen:sg")
            .expect("target features");
        let competitor = crate::nanda_wave::morphology_phase::parse_features("noun:dat:sg")
            .expect("competitor features");
        let context_key =
            scoped_context_evidence_key(context_evidence_keys("пока нет _ дома")[1], 1);
        let slot = crate::nanda_wave::morphology_phase::productive_context_slot_features(target);
        let competitor_slot =
            crate::nanda_wave::morphology_phase::productive_neighbor_context_slot_features(
                competitor,
            );
        ProductiveMorphologyIndex {
            rules_by_family: BTreeMap::from([(
                (1, source, target, "а".to_string()),
                vec![ProductiveRule {
                    primary_pos: 1,
                    source_feature_mask: source,
                    target_feature_mask: target,
                    family_suffix: "а".to_string(),
                    remove_prefix_chars: 0,
                    prepend: String::new(),
                    remove_chars: 1,
                    append: "и".to_string(),
                    positive_support: 7,
                    anti_support: 1,
                }],
            )]),
            target_features_by_source: BTreeMap::from([((1, source), vec![target])]),
            context_slots: BTreeMap::from([(
                (context_key, slot),
                ProductiveContextSlotEvidence {
                    positive_support: 5,
                    unlabeled_alternative_support: 2,
                    posterior_milli: laplace_posterior_milli(5, 2),
                    context_observed: true,
                },
            )]),
            known_contexts: BTreeMap::from([(context_key, 7)]),
            context_pair_support: BTreeMap::from([
                ((context_key, slot, competitor_slot), 4),
                ((context_key, competitor_slot, slot), 1),
            ]),
            report: ProductiveIndexReport {
                observed_lemmas: 8,
                admitted_lemmas: 7,
                observed_transforms: 12,
                admitted_profiles: 1,
                observed_context_rows: 7,
                admitted_context_rows: 7,
                context_modes: 1,
                context_slots: 1,
                observed_competitor_rows: 5,
                observed_competitor_surfaces: 5,
                same_lemma_competitor_surfaces: 5,
                admitted_pair_observations: 5,
                context_pairs: 2,
                ..ProductiveIndexReport::default()
            },
        }
    }

    #[test]
    fn compact_productive_sidecar_preserves_generation_and_context_evidence() {
        let index = fixture();
        let (first, stats) = encode_index(&index, 77).expect("encode");
        let (second, _) = encode_index(&index, 77).expect("deterministic encode");
        let view = CompactProductiveMorphologyView::from_bytes(first.clone()).expect("view");
        let source = crate::nanda_wave::morphology_phase::parse_features("noun:nom:sg")
            .expect("source features");
        let target = crate::nanda_wave::morphology_phase::parse_features("noun:gen:sg")
            .expect("target features");
        let competitor = crate::nanda_wave::morphology_phase::parse_features("noun:dat:sg")
            .expect("competitor features");

        assert_eq!(first, second);
        assert_eq!(stats.rules, 1);
        assert_eq!(stats.context_pairs, 2);
        assert_eq!(view.l2_fingerprint(), 77);
        assert_eq!(view.report(), index.report);
        assert_eq!(
            view.target_features_vec(1, source),
            index.target_features_vec(1, source)
        );
        assert_eq!(
            view.context_slot_evidence_for("нет _", target),
            index.context_slot_evidence_for("нет _", target)
        );
        assert_eq!(
            view.context_pair_evidence_for("нет _", target, competitor),
            index.context_pair_evidence_for("нет _", target, competitor)
        );
        assert_eq!(
            view.generate_forms("книги", 1, "книга", source, target, 8),
            index.generate_forms("книги", 1, "книга", source, target, 8)
        );
    }

    #[test]
    fn compact_productive_sidecar_rejects_truncated_sections() {
        let (mut bytes, _) = encode_index(&fixture(), 77).expect("encode");
        bytes.pop();

        assert!(CompactProductiveMorphologyView::from_bytes(bytes).is_err());
    }
}
