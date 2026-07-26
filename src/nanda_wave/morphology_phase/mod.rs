mod corpus;
mod field;
mod proof;
mod runtime;

pub use proof::{run_embedded_russian_morphology_proof, run_russian_morphology_proof_path};
pub(crate) use runtime::{shadow_same_lemma_surface_readout, SameLemmaSurfaceReadout};

const PHASE_CELLS: usize = 60;
const MAX_SUBCENTERS: usize = 4;

const POS_NOUN: u32 = 1 << 0;
const NUMBER_SINGULAR: u32 = 1 << 4;
const NUMBER_PLURAL: u32 = 1 << 5;
const NUMBER_MASK: u32 = NUMBER_SINGULAR | NUMBER_PLURAL;
const CASE_NOMINATIVE: u32 = 1 << 8;
const CASE_GENITIVE: u32 = 1 << 9;
const CASE_DATIVE: u32 = 1 << 10;
const CASE_ACCUSATIVE: u32 = 1 << 11;
const CASE_INSTRUMENTAL: u32 = 1 << 12;
const CASE_PREPOSITIONAL: u32 = 1 << 13;
const CASE_PARTITIVE: u32 = 1 << 14;
const CASE_SECOND_LOCATIVE: u32 = 1 << 15;
const CASE_VOCATIVE: u32 = 1 << 16;
const CASE_MASK: u32 = CASE_NOMINATIVE
    | CASE_GENITIVE
    | CASE_DATIVE
    | CASE_ACCUSATIVE
    | CASE_INSTRUMENTAL
    | CASE_PREPOSITIONAL
    | CASE_PARTITIVE
    | CASE_SECOND_LOCATIVE
    | CASE_VOCATIVE;

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct MorphBinding16 {
    form_center_id: u32,
    lemma_center_id: u32,
    features: u32,
    support: u16,
    phase: i8,
    flags: u8,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct MorphPhaseCenter64 {
    cells: [i8; PHASE_CELLS],
    support: u16,
    mass: u16,
}

impl Default for MorphPhaseCenter64 {
    fn default() -> Self {
        Self {
            cells: [0; PHASE_CELLS],
            support: 0,
            mass: 0,
        }
    }
}

fn parse_features(raw: &str) -> Result<u32, String> {
    let mut features = 0_u32;
    for part in raw.split(':') {
        features |= match part {
            "noun" => POS_NOUN,
            "sg" => NUMBER_SINGULAR,
            "pl" => NUMBER_PLURAL,
            "nom" => CASE_NOMINATIVE,
            "gen" => CASE_GENITIVE,
            "dat" => CASE_DATIVE,
            "acc" => CASE_ACCUSATIVE,
            "ins" => CASE_INSTRUMENTAL,
            "prep" => CASE_PREPOSITIONAL,
            "part" => CASE_PARTITIVE,
            "loc2" => CASE_SECOND_LOCATIVE,
            "voc" => CASE_VOCATIVE,
            other => return Err(format!("unknown morphology feature {other:?}")),
        };
    }
    if features & POS_NOUN == 0
        || (features & NUMBER_MASK).count_ones() != 1
        || (features & CASE_MASK).count_ones() != 1
    {
        return Err(format!("incomplete morphology feature set {raw:?}"));
    }
    Ok(features)
}

fn feature_name(features: u32) -> &'static str {
    match (features & CASE_MASK, features & NUMBER_MASK) {
        (CASE_NOMINATIVE, NUMBER_SINGULAR) => "nominative_singular",
        (CASE_GENITIVE, NUMBER_SINGULAR) => "genitive_singular",
        (CASE_DATIVE, NUMBER_SINGULAR) => "dative_singular",
        (CASE_ACCUSATIVE, NUMBER_SINGULAR) => "accusative_singular",
        (CASE_INSTRUMENTAL, NUMBER_SINGULAR) => "instrumental_singular",
        (CASE_PREPOSITIONAL, NUMBER_SINGULAR) => "prepositional_singular",
        (CASE_PARTITIVE, NUMBER_SINGULAR) => "partitive_singular",
        (CASE_SECOND_LOCATIVE, NUMBER_SINGULAR) => "second_locative_singular",
        (CASE_VOCATIVE, NUMBER_SINGULAR) => "vocative_singular",
        (CASE_NOMINATIVE, NUMBER_PLURAL) => "nominative_plural",
        (CASE_GENITIVE, NUMBER_PLURAL) => "genitive_plural",
        (CASE_DATIVE, NUMBER_PLURAL) => "dative_plural",
        (CASE_ACCUSATIVE, NUMBER_PLURAL) => "accusative_plural",
        (CASE_INSTRUMENTAL, NUMBER_PLURAL) => "instrumental_plural",
        (CASE_PREPOSITIONAL, NUMBER_PLURAL) => "prepositional_plural",
        (CASE_PARTITIVE, NUMBER_PLURAL) => "partitive_plural",
        (CASE_SECOND_LOCATIVE, NUMBER_PLURAL) => "second_locative_plural",
        (CASE_VOCATIVE, NUMBER_PLURAL) => "vocative_plural",
        _ => "unknown",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compact_morphology_records_keep_the_promised_width() {
        assert_eq!(std::mem::size_of::<MorphBinding16>(), 16);
        assert_eq!(std::mem::size_of::<MorphPhaseCenter64>(), 64);
    }

    #[test]
    fn feature_parser_requires_one_complete_noun_slot() {
        assert_eq!(
            parse_features("noun:dat:sg"),
            Ok(POS_NOUN | CASE_DATIVE | NUMBER_SINGULAR)
        );
        assert_eq!(
            parse_features("noun:prep:pl"),
            Ok(POS_NOUN | CASE_PREPOSITIONAL | NUMBER_PLURAL)
        );
        assert_eq!(
            parse_features("noun:loc2:sg"),
            Ok(POS_NOUN | CASE_SECOND_LOCATIVE | NUMBER_SINGULAR)
        );
        assert!(parse_features("noun:sg").is_err());
        assert!(parse_features("noun:nom:sg:pl").is_err());
        assert!(parse_features("noun:nom:acc:sg").is_err());
    }
}
