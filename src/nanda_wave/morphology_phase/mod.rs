mod corpus;
mod field;
mod proof;
mod runtime;

pub use proof::{run_embedded_russian_morphology_proof, run_russian_morphology_proof_path};
pub(crate) use runtime::{shadow_same_lemma_surface_readout, SameLemmaSurfaceReadout};

const PHASE_CELLS: usize = 60;
const MAX_SUBCENTERS: usize = 4;

pub(crate) const POS_NOUN: u32 = 1 << 0;
pub(crate) const POS_VERB: u32 = 1 << 1;
pub(crate) const POS_ADJECTIVE: u32 = 1 << 2;
pub(crate) const POS_PRONOUN: u32 = 1 << 3;
const POS_MASK: u32 = POS_NOUN | POS_VERB | POS_ADJECTIVE | POS_PRONOUN;
const NUMBER_SINGULAR: u32 = 1 << 4;
const NUMBER_PLURAL: u32 = 1 << 5;
const NUMBER_MASK: u32 = NUMBER_SINGULAR | NUMBER_PLURAL;
const IMPERATIVE_INCLUSIVE: u32 = 1 << 6;
const IMPERATIVE_EXCLUSIVE: u32 = 1 << 7;
const IMPERATIVE_KIND_MASK: u32 = IMPERATIVE_INCLUSIVE | IMPERATIVE_EXCLUSIVE;
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
const GENDER_MASCULINE: u32 = 1 << 17;
const GENDER_FEMININE: u32 = 1 << 18;
const GENDER_NEUTER: u32 = 1 << 19;
const GENDER_MASK: u32 = GENDER_MASCULINE | GENDER_FEMININE | GENDER_NEUTER;
const PERSON_FIRST: u32 = 1 << 20;
const PERSON_SECOND: u32 = 1 << 21;
const PERSON_THIRD: u32 = 1 << 22;
const PERSON_MASK: u32 = PERSON_FIRST | PERSON_SECOND | PERSON_THIRD;
const TENSE_PAST: u32 = 1 << 23;
const TENSE_PRESENT: u32 = 1 << 24;
const TENSE_FUTURE: u32 = 1 << 25;
const TENSE_MASK: u32 = TENSE_PAST | TENSE_PRESENT | TENSE_FUTURE;
const MOOD_INDICATIVE: u32 = 1 << 26;
const MOOD_IMPERATIVE: u32 = 1 << 27;
const MOOD_MASK: u32 = MOOD_INDICATIVE | MOOD_IMPERATIVE;
const ASPECT_PERFECTIVE: u32 = 1 << 28;
const ASPECT_IMPERFECTIVE: u32 = 1 << 29;
const ASPECT_MASK: u32 = ASPECT_PERFECTIVE | ASPECT_IMPERFECTIVE;
const FORM_KIND_A: u32 = 1 << 30;
const FORM_KIND_B: u32 = 1 << 31;

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

pub(crate) fn parse_features(raw: &str) -> Result<u32, String> {
    let mut features = 0_u32;
    for part in raw.split(':') {
        features |= match part {
            "noun" => POS_NOUN,
            "verb" | "aux" => POS_VERB,
            "adj" => POS_ADJECTIVE,
            "pron" => POS_PRONOUN,
            "sg" => NUMBER_SINGULAR,
            "pl" => NUMBER_PLURAL,
            "imp_incl" => IMPERATIVE_INCLUSIVE,
            "imp_excl" => IMPERATIVE_EXCLUSIVE,
            "nom" => CASE_NOMINATIVE,
            "gen" => CASE_GENITIVE,
            "dat" => CASE_DATIVE,
            "acc" => CASE_ACCUSATIVE,
            "ins" => CASE_INSTRUMENTAL,
            "prep" => CASE_PREPOSITIONAL,
            "part" => CASE_PARTITIVE,
            "loc2" => CASE_SECOND_LOCATIVE,
            "voc" => CASE_VOCATIVE,
            "masc" => GENDER_MASCULINE,
            "fem" => GENDER_FEMININE,
            "neut" => GENDER_NEUTER,
            "p1" => PERSON_FIRST,
            "p2" => PERSON_SECOND,
            "p3" => PERSON_THIRD,
            "past" => TENSE_PAST,
            "pres" => TENSE_PRESENT,
            "fut" => TENSE_FUTURE,
            "ind" => MOOD_INDICATIVE,
            "imp" => MOOD_IMPERATIVE,
            "perf" => ASPECT_PERFECTIVE,
            "imperf" => ASPECT_IMPERFECTIVE,
            "inf" | "short" => FORM_KIND_A,
            "ger" | "comp" => FORM_KIND_B,
            other => return Err(format!("unknown morphology feature {other:?}")),
        };
    }
    if (features & POS_MASK).count_ones() != 1 || !valid_feature_shape(features) {
        return Err(format!("incomplete morphology feature set {raw:?}"));
    }
    Ok(features)
}

pub(crate) fn feature_primary_pos(features: u32) -> u16 {
    match features & POS_MASK {
        POS_NOUN => 1,
        POS_VERB => 2,
        POS_ADJECTIVE => 3,
        POS_PRONOUN => 4,
        _ => 0,
    }
}

pub(crate) fn contextual_slot_features(features: u32) -> u32 {
    if features & POS_MASK == POS_PRONOUN {
        POS_PRONOUN | (features & CASE_MASK)
    } else {
        features
    }
}

pub(crate) fn same_inclusive_imperative_family(left: u32, right: u32) -> bool {
    let required = POS_VERB | MOOD_IMPERATIVE | IMPERATIVE_INCLUSIVE;
    left & required == required
        && right & required == required
        && left & ASPECT_MASK == right & ASPECT_MASK
}

pub(crate) fn same_finite_agreement_family(left: u32, right: u32) -> bool {
    let finite_person = |features: u32| {
        let explicit = features & PERSON_MASK;
        if explicit == 0 && features & MOOD_IMPERATIVE != 0 && features & IMPERATIVE_EXCLUSIVE != 0
        {
            PERSON_SECOND
        } else {
            explicit
        }
    };
    let left_person = finite_person(left);
    let right_person = finite_person(right);
    let left_number = left & NUMBER_MASK;
    let right_number = right & NUMBER_MASK;
    left & POS_MASK == POS_VERB
        && right & POS_MASK == POS_VERB
        && left_person != 0
        && left_person == right_person
        && left_number != 0
        && left_number == right_number
}

fn valid_feature_shape(features: u32) -> bool {
    let at_most_one = |mask: u32| (features & mask).count_ones() <= 1;
    if !at_most_one(NUMBER_MASK)
        || !at_most_one(CASE_MASK)
        || !at_most_one(GENDER_MASK)
        || !at_most_one(PERSON_MASK)
        || !at_most_one(TENSE_MASK)
        || !at_most_one(MOOD_MASK)
        || !at_most_one(ASPECT_MASK)
        || !at_most_one(IMPERATIVE_KIND_MASK)
    {
        return false;
    }
    if features & IMPERATIVE_KIND_MASK != 0 && features & MOOD_IMPERATIVE == 0 {
        return false;
    }
    match features & POS_MASK {
        POS_NOUN => {
            (features & NUMBER_MASK).count_ones() == 1 && (features & CASE_MASK).count_ones() == 1
        }
        POS_VERB => {
            if features & FORM_KIND_A != 0 {
                features & (NUMBER_MASK | PERSON_MASK | TENSE_MASK | MOOD_MASK) == 0
            } else if features & FORM_KIND_B != 0 {
                true
            } else {
                (features & (TENSE_MASK | MOOD_MASK)).count_ones() >= 1
            }
        }
        POS_ADJECTIVE => {
            features & FORM_KIND_B != 0
                || ((features & NUMBER_MASK).count_ones() == 1
                    && (features & (CASE_MASK | FORM_KIND_A)).count_ones() >= 1)
        }
        POS_PRONOUN => features & (NUMBER_MASK | CASE_MASK | GENDER_MASK | PERSON_MASK) != 0,
        _ => false,
    }
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
    fn feature_parser_requires_complete_pos_specific_slots() {
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
        assert!(parse_features("verb:inf:perf").is_ok());
        assert!(parse_features("verb:pres:ind:p1:sg:imperf").is_ok());
        assert!(parse_features("verb:pres:past:ind:sg").is_err());
        assert!(parse_features("verb:pl:imp:imp_excl:perf").is_ok());
        assert!(parse_features("verb:sg:imp:imp_incl:imperf").is_ok());
        assert!(parse_features("verb:sg:imp_incl:imperf").is_err());
        assert!(parse_features("verb:sg:imp:imp_incl:imp_excl:imperf").is_err());
        assert!(parse_features("adj:nom:sg:masc").is_ok());
        assert!(parse_features("adj:comp").is_ok());
        assert!(parse_features("pron:dat:sg:p1").is_ok());
        assert!(parse_features("verb:adj:pres").is_err());
    }

    #[test]
    fn pronoun_context_slot_projects_to_case_without_identity_features() {
        let first_singular = parse_features("pron:dat:sg:p1").expect("first singular");
        let third_plural = parse_features("pron:dat:pl:p3").expect("third plural");
        assert_eq!(
            contextual_slot_features(first_singular),
            contextual_slot_features(third_plural)
        );
        assert_ne!(first_singular, third_plural);
    }

    #[test]
    fn finite_agreement_family_ignores_tense_and_mood() {
        let future = parse_features("verb:fut:ind:p2:sg:imperf").expect("future second singular");
        let imperative = parse_features("verb:imp:imp_excl:sg:imperf")
            .expect("exclusive imperative second singular");
        let first_person =
            parse_features("verb:fut:ind:p1:sg:imperf").expect("future first singular");
        assert!(same_finite_agreement_family(future, imperative));
        assert!(!same_finite_agreement_family(future, first_person));
    }
}
