use std::collections::BTreeMap;
use std::sync::Arc;

use super::super::atoms::AtomChannel;
use super::super::model::{LexicalGrokkingPackage, WaveCoupling};
use super::super::runtime::{ForwardActivation, ObservedAtom};
use super::exact_reverse::{exact_reverse_from_occurrences, resolve_terminal_occurrences};
use super::support::ExactSupportField;

#[derive(Clone, Debug)]
pub(super) struct ImplicitForwardRelation {
    pub(super) atom_id: u32,
    pub(super) channel: AtomChannel,
    pub(super) coupling: WaveCoupling,
}

#[derive(Clone, Debug)]
pub(super) struct ImplicitCandidate {
    pub(super) terminal_id: u32,
    pub(super) relations: Vec<ImplicitForwardRelation>,
    pub(super) exact_reverse: Arc<[WaveCoupling]>,
    pub(super) activation: ForwardActivation,
}

pub(super) fn reconstruct_candidate(
    package: &LexicalGrokkingPackage,
    support: &ExactSupportField,
    observed: &BTreeMap<u32, ObservedAtom>,
    terminal_id: u32,
) -> Result<ImplicitCandidate, String> {
    let mut occurrences = resolve_terminal_occurrences(package, terminal_id)?;
    let exact_reverse = exact_reverse_from_occurrences(package, support, &occurrences)?;
    occurrences.sort_unstable_by_key(|item| (item.atom_id, item.position, item.channel as u8));

    let mut relations = Vec::new();
    let mut cursor = 0_usize;
    while cursor < occurrences.len() {
        let first = occurrences[cursor];
        let mut end = cursor + 1;
        let mut position_sum = u64::from(first.position);
        while end < occurrences.len() && occurrences[end].atom_id == first.atom_id {
            if occurrences[end].channel != first.channel {
                return Err(format!(
                    "one AtomId resolves to multiple channels: atom={} terminal={terminal_id}",
                    first.atom_id
                ));
            }
            position_sum = position_sum.saturating_add(u64::from(occurrences[end].position));
            end += 1;
        }
        if first.channel != AtomChannel::CharacterAnchor {
            let observations = u32::try_from(end - cursor)
                .map_err(|_| "implicit atom observation count exceeds u32".to_string())?;
            let exact_support = support
                .get(first.atom_id)
                .ok_or_else(|| format!("implicit atom lacks exact support: {}", first.atom_id))?;
            let average_position = position_sum / u64::from(observations.max(1));
            let position_mode = (average_position / 257).min(255) as u8;
            relations.push(ImplicitForwardRelation {
                atom_id: first.atom_id,
                channel: first.channel,
                coupling: WaveCoupling {
                    peer_id: terminal_id,
                    strength: coupling_strength(observations, exact_support, package.centers.len()),
                    phase_relation: position_phase(position_mode),
                    position_mode,
                    flags: 0,
                },
            });
        }
        cursor = end;
    }
    let activation = activation_from_relations(&relations, observed);
    Ok(ImplicitCandidate {
        terminal_id,
        relations,
        exact_reverse: exact_reverse.into(),
        activation,
    })
}

pub(super) fn activation_from_relations(
    relations: &[ImplicitForwardRelation],
    observed: &BTreeMap<u32, ObservedAtom>,
) -> ForwardActivation {
    let mut activation = ForwardActivation::default();
    for relation in relations {
        let Some(atom) = observed.get(&relation.atom_id) else {
            continue;
        };
        activation.mass = activation.mass.saturating_add(
            u64::from(relation.coupling.strength)
                .saturating_mul(u64::from(atom.weight))
                .saturating_mul(u64::from(position_coherence(
                    atom.position,
                    relation.coupling.position_mode,
                ))),
        );
        activation.hits = activation.hits.saturating_add(1);
        if is_keyboard_channel(atom.channel) {
            activation.keyboard_hits = activation.keyboard_hits.saturating_add(1);
        } else {
            activation.surface_hits = activation.surface_hits.saturating_add(1);
        }
    }
    activation
}

pub(super) fn activation_equal(left: ForwardActivation, right: ForwardActivation) -> bool {
    left.mass == right.mass
        && left.hits == right.hits
        && left.surface_hits == right.surface_hits
        && left.keyboard_hits == right.keyboard_hits
}

pub(super) fn candidates_equal(left: &[ImplicitCandidate], right: &[ImplicitCandidate]) -> bool {
    left.len() == right.len()
        && left.iter().zip(right).all(|(left, right)| {
            left.terminal_id == right.terminal_id
                && activation_equal(left.activation, right.activation)
                && left.relations.len() == right.relations.len()
                && left.exact_reverse.as_ref() == right.exact_reverse.as_ref()
                && left
                    .relations
                    .iter()
                    .zip(&right.relations)
                    .all(|(left, right)| {
                        left.atom_id == right.atom_id
                            && left.channel == right.channel
                            && left.coupling == right.coupling
                    })
        })
}

fn coupling_strength(observations: u32, atom_support: u32, word_count: usize) -> u8 {
    let reliability = observations.saturating_mul(255);
    let specificity =
        ((word_count as u32 + 1).saturating_mul(32) / atom_support.max(1)).clamp(32, 255);
    ((reliability.saturating_mul(specificity) / 255).clamp(1, 255)) as u8
}

fn position_phase(position: u8) -> i8 {
    (i16::from(position) - 128).clamp(-127, 127) as i8
}

fn position_coherence(observed: u8, expected: u8) -> u16 {
    256_u16.saturating_sub(u16::from(observed.abs_diff(expected)))
}

fn is_keyboard_channel(channel: AtomChannel) -> bool {
    matches!(
        channel,
        AtomChannel::KeyboardGram
            | AtomChannel::KeyboardBigram
            | AtomChannel::KeyboardBagGram
            | AtomChannel::KeyboardSkipGram
    )
}
