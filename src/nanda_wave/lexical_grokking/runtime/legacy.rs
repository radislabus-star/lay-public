use super::*;
use std::cell::RefCell;

use super::contract::ForwardScratch;

thread_local! {
    static FORWARD_SCRATCH: RefCell<ForwardScratch> = RefCell::new(ForwardScratch::default());
}

pub(super) fn select_birth_atoms(
    birth_by_channel: &mut [Vec<BirthAtom>],
    atoms_per_channel: usize,
    posting_budget: usize,
) -> Vec<BirthAtom> {
    let mut eligible = Vec::new();
    for atoms in birth_by_channel {
        atoms.sort_unstable_by(|left, right| {
            left.0
                .cmp(&right.0)
                .then_with(|| right.2.weight.cmp(&left.2.weight))
                .then_with(|| left.1.cmp(&right.1))
        });
        eligible.extend(atoms.iter().take(atoms_per_channel).copied());
    }
    eligible.sort_unstable_by(|left, right| {
        left.0
            .cmp(&right.0)
            .then_with(|| right.2.weight.cmp(&left.2.weight))
            .then_with(|| left.1.cmp(&right.1))
    });
    let mut selected = Vec::with_capacity(eligible.len());
    let mut posting_count = 0_usize;
    for atom in eligible {
        let next = posting_count.saturating_add(atom.0);
        if !selected.is_empty() && next > posting_budget {
            continue;
        }
        posting_count = next;
        selected.push(atom);
    }
    selected
}

pub(super) fn should_expand_operator_lattice(exact_terminal_count: usize, limit: usize) -> bool {
    exact_terminal_count == 0 || limit > exact_terminal_count
}

impl LexicalGrokkingMemory {
    pub(super) fn prepare_readout(&self, surface: &str, limit: usize) -> Option<PreparedReadout> {
        let trace_started = Instant::now();
        let observed = self.resolve_surface(surface);
        if observed.is_empty() {
            return None;
        }
        let resolve_us = trace_started.elapsed().as_micros();
        let character_sequence = observed_sequence(&observed, AtomChannel::CharacterAnchor);
        let exact_terminals = self.exact_terminals(character_sequence.as_slice());
        let observed_char_count = normalize_lexical_surface(surface)
            .chars()
            .count()
            .min(u8::MAX as usize) as u8;
        let lexical_observed = observed
            .iter()
            .filter(|(_, atom)| !is_anchor_channel(atom.channel))
            .copied()
            .collect::<BTreeMap<_, _>>();
        let mut birth_by_channel: [Vec<BirthAtom>; 12] = std::array::from_fn(|_| Vec::new());
        for (atom_id, atom) in &lexical_observed {
            birth_by_channel[atom.channel as usize].push((
                self.forward_degree(*atom_id),
                *atom_id,
                *atom,
            ));
        }
        let birth_atoms = select_birth_atoms(
            &mut birth_by_channel,
            birth_atoms_per_channel(),
            birth_posting_budget(),
        );
        let birth_postings = birth_atoms
            .iter()
            .map(|(degree, _, _)| *degree)
            .sum::<usize>();
        let birth_atom_ids = birth_atoms
            .iter()
            .map(|(_, atom_id, _)| *atom_id)
            .collect::<Vec<_>>();
        let birth_couplings = self.forward_coupling_views(&birth_atom_ids);
        let prefetch_us = trace_started.elapsed().as_micros();
        let (surface_re, surface_im, mut frontier) = FORWARD_SCRATCH.with_borrow_mut(|scratch| {
            if scratch.activations.len() != self.package.terminal_count() as usize {
                scratch.activations =
                    vec![ForwardActivation::default(); self.package.terminal_count() as usize];
                scratch.activation_epochs = vec![0; self.package.terminal_count() as usize];
                scratch.epoch = 1;
                scratch.touched.clear();
            } else {
                scratch.epoch = scratch.epoch.wrapping_add(1);
                if scratch.epoch == 0 {
                    scratch.activation_epochs.fill(0);
                    scratch.epoch = 1;
                }
                scratch.touched.clear();
            }
            let epoch = scratch.epoch;
            let mut surface_re = [0_i32; WAVE_DIMENSION];
            let mut surface_im = [0_i32; WAVE_DIMENSION];
            for (atom_id, atom) in &lexical_observed {
                let Some(record) = self.package.atoms.get(*atom_id as usize) else {
                    continue;
                };
                expand_atom(
                    &self.package.basis,
                    record.wave_code,
                    &mut surface_re,
                    &mut surface_im,
                    i32::from(atom.weight),
                );
            }
            for ((_, _, atom), couplings) in birth_atoms.iter().zip(&birth_couplings) {
                let atom_weight = u64::from(atom.weight);
                let keyboard_channel = is_keyboard_channel(atom.channel);
                for coupling in couplings.iter() {
                    let contribution = u64::from(coupling.strength)
                        * atom_weight
                        * u64::from(position_coherence(atom.position, coupling.position_mode));
                    let terminal_id = coupling.peer_id as usize;
                    if terminal_id >= scratch.activations.len() {
                        continue;
                    }
                    if scratch.activation_epochs[terminal_id] != epoch {
                        scratch.activation_epochs[terminal_id] = epoch;
                        scratch.activations[terminal_id] = ForwardActivation::default();
                        scratch.touched.push(coupling.peer_id);
                    }
                    let activation = &mut scratch.activations[terminal_id];
                    activation.mass += contribution;
                    activation.hits += 1;
                    if keyboard_channel {
                        activation.keyboard_hits += 1;
                    } else {
                        activation.surface_hits += 1;
                    }
                }
            }
            let frontier = scratch
                .touched
                .iter()
                .map(|terminal_id| (*terminal_id, scratch.activations[*terminal_id as usize]))
                .collect::<Vec<_>>();
            (surface_re, surface_im, frontier)
        });
        let forward_us = trace_started.elapsed().as_micros();
        let operator_reserve = if should_expand_operator_lattice(exact_terminals.len(), limit) {
            self.operator_reserve(surface, &lexical_observed, !exact_terminals.is_empty())
        } else {
            Vec::new()
        };
        let operator_us = trace_started.elapsed().as_micros();
        let frontier_order = |left: &(u32, ForwardActivation), right: &(u32, ForwardActivation)| {
            exact_terminals
                .contains(&right.0)
                .cmp(&exact_terminals.contains(&left.0))
                .then_with(|| right.1.mass.cmp(&left.1.mass))
                .then_with(|| right.1.hits.cmp(&left.1.hits))
                .then_with(|| left.0.cmp(&right.0))
        };
        let touched_count = frontier.len();
        if let Some(trace_terminal) = readout_trace_terminal() {
            let selected_activation = frontier
                .iter()
                .find_map(|(terminal_id, activation)| {
                    (*terminal_id == trace_terminal).then_some(*activation)
                })
                .unwrap_or_default();
            let full_activation = self.activation_for_terminal(trace_terminal, &lexical_observed);
            let expected = self.character_anchors(trace_terminal);
            let reconstruction_modes =
                reconstruction_modes(character_sequence.as_slice(), expected);
            let selected_support_atoms = birth_atoms
                .iter()
                .filter(|(_, atom_id, _)| {
                    self.forward_couplings(*atom_id)
                        .iter()
                        .any(|coupling| coupling.peer_id == trace_terminal)
                })
                .count();
            let observed_support_atoms = lexical_observed
                .keys()
                .filter(|atom_id| {
                    self.forward_couplings(**atom_id)
                        .iter()
                        .any(|coupling| coupling.peer_id == trace_terminal)
                })
                .count();
            eprintln!(
                "l11_trace_terminal terminal={} touched={} selected_hits={} selected_mass={} \
                 full_hits={} full_mass={} reconstruction_modes={} observed_support_atoms={} \
                 selected_support_atoms={}",
                trace_terminal,
                selected_activation.hits != 0,
                selected_activation.hits,
                selected_activation.mass,
                full_activation.hits,
                full_activation.mass,
                reconstruction_modes,
                observed_support_atoms,
                selected_support_atoms,
            );
        }
        if frontier.len() > MAX_RECONSTRUCTION_SCAN {
            frontier.select_nth_unstable_by(MAX_RECONSTRUCTION_SCAN, frontier_order);
            frontier.truncate(MAX_RECONSTRUCTION_SCAN);
        }
        let reconstruction_reserve =
            self.reconstruction_lane_reserve(&frontier, character_sequence.as_slice());
        let reconstruction_us = trace_started.elapsed().as_micros();
        if frontier.len() > MAX_GEOMETRY_SCAN {
            frontier.select_nth_unstable_by(MAX_GEOMETRY_SCAN, frontier_order);
            frontier.truncate(MAX_GEOMETRY_SCAN);
        }
        frontier.sort_unstable_by(frontier_order);
        let geometry_reserve = if exact_terminals.is_empty() {
            self.geometry_reserve(&frontier, character_sequence.as_slice())
        } else {
            Vec::new()
        };
        let geometry_us = trace_started.elapsed().as_micros();
        let operator_reserve_count = operator_reserve.len();
        let reconstruction_reserve_count = reconstruction_reserve.len();
        let geometry_reserve_count = geometry_reserve.len();
        let geometry_reserve_ids = operator_reserve
            .iter()
            .chain(&reconstruction_reserve)
            .chain(&geometry_reserve)
            .map(|(terminal_id, _)| *terminal_id)
            .collect::<BTreeSet<_>>();
        frontier.truncate(MAX_PHASE_FRONTIER.max(limit));
        let primary_ids = frontier
            .iter()
            .map(|(terminal_id, _)| *terminal_id)
            .collect::<BTreeSet<_>>();
        let reconstruction_only_ids = reconstruction_reserve
            .iter()
            .map(|(terminal_id, _)| *terminal_id)
            .filter(|terminal_id| !primary_ids.contains(terminal_id))
            .collect::<BTreeSet<_>>();
        let mut retained = primary_ids;
        frontier.extend(
            operator_reserve
                .into_iter()
                .filter(|(terminal_id, _)| retained.insert(*terminal_id)),
        );
        frontier.extend(
            reconstruction_reserve
                .into_iter()
                .filter(|(terminal_id, _)| retained.insert(*terminal_id)),
        );
        frontier.extend(
            geometry_reserve
                .into_iter()
                .filter(|(terminal_id, _)| retained.insert(*terminal_id)),
        );
        let mut frontier_reverse =
            self.refresh_frontier_activations(&mut frontier, &lexical_observed);
        let activation_us = trace_started.elapsed().as_micros();
        if let Some(reverse) = frontier_reverse.take() {
            let mut retained_frontier = Vec::with_capacity(frontier.len());
            let mut retained_reverse = Vec::with_capacity(reverse.len());
            for (candidate, relations) in frontier.into_iter().zip(reverse) {
                if candidate.1.hits != 0 {
                    retained_frontier.push(candidate);
                    retained_reverse.push(relations);
                }
            }
            frontier = retained_frontier;
            frontier_reverse = Some(retained_reverse);
        } else {
            frontier.retain(|(_, activation)| activation.hits != 0);
        }
        let max_forward = frontier
            .iter()
            .map(|(_, activation)| activation.mass)
            .max()
            .unwrap_or(1)
            .max(1);
        if readout_trace_enabled() {
            eprintln!(
                "l11_readout_trace resolve_us={resolve_us} prefetch_us={} forward_us={} operator_us={} \
                 reconstruction_us={} geometry_us={} activation_us={} prepare_us={} touched={} retained={} \
                 operator_reserve={} reconstruction_reserve={} geometry_reserve={} birth_atoms={} \
                 birth_postings={}",
                prefetch_us.saturating_sub(resolve_us),
                forward_us.saturating_sub(prefetch_us),
                operator_us.saturating_sub(forward_us),
                reconstruction_us.saturating_sub(operator_us),
                geometry_us.saturating_sub(reconstruction_us),
                activation_us.saturating_sub(geometry_us),
                trace_started.elapsed().as_micros(),
                touched_count,
                retained.len(),
                operator_reserve_count,
                reconstruction_reserve_count,
                geometry_reserve_count,
                birth_atoms.len(),
                birth_postings,
            );
        }
        Some(PreparedReadout {
            observed: lexical_observed,
            character_sequence,
            observed_char_count,
            surface_re,
            surface_im,
            max_forward,
            frontier,
            frontier_reverse,
            geometry_reserve_ids,
            reconstruction_only_ids,
        })
    }

    pub(super) fn exact_singleton_readout(&self, surface: &str) -> Option<GrokkingCandidate> {
        let terminal_id = self.exact_terminal_for_surface(surface)?;
        let observed = self.resolve_surface(surface);
        let character_sequence = observed_sequence(&observed, AtomChannel::CharacterAnchor);
        self.package.centers.get(terminal_id as usize)?;
        let reverse = self.reverse_couplings(terminal_id);
        let observed_char_count = normalize_lexical_surface(surface)
            .chars()
            .count()
            .min(u8::MAX as usize) as u8;
        let mut surface_re = [0_i32; WAVE_DIMENSION];
        let mut surface_im = [0_i32; WAVE_DIMENSION];
        let mut activation = ForwardActivation::default();
        for (atom_id, atom) in &observed {
            if is_anchor_channel(atom.channel) {
                continue;
            }
            let record = self.package.atoms.get(*atom_id as usize)?;
            expand_atom(
                &self.package.basis,
                record.wave_code,
                &mut surface_re,
                &mut surface_im,
                i32::from(atom.weight),
            );
            let coupling = reverse
                .iter()
                .find(|coupling| coupling.flags == 0 && coupling.peer_id == *atom_id);
            let Some(coupling) = coupling else {
                continue;
            };
            let contribution = u64::from(coupling.strength)
                .saturating_mul(u64::from(atom.weight))
                .saturating_mul(u64::from(position_coherence(
                    atom.position,
                    coupling.position_mode,
                )));
            activation.mass = activation.mass.saturating_add(contribution);
            activation.hits = activation.hits.saturating_add(1);
            if is_keyboard_channel(atom.channel) {
                activation.keyboard_hits = activation.keyboard_hits.saturating_add(1);
            } else {
                activation.surface_hits = activation.surface_hits.saturating_add(1);
            }
        }
        if activation.hits == 0 {
            return None;
        }
        let observed = observed
            .into_iter()
            .filter(|(_, atom)| !is_anchor_channel(atom.channel))
            .collect::<BTreeMap<_, _>>();
        let candidate = self.settle_candidate(
            terminal_id,
            activation,
            activation.mass.max(1),
            &observed,
            &surface_re,
            &surface_im,
            &character_sequence,
            observed_char_count,
            ReadoutMode::Full,
        )?;
        let mut candidates = vec![candidate];
        self.finalize_candidates(
            surface,
            1,
            ReadoutMode::Full,
            &surface_re,
            &surface_im,
            &mut candidates,
        );
        candidates.into_iter().next()
    }

    pub(super) fn resolve_surface(&self, surface: &str) -> Vec<(u32, ObservedAtom)> {
        encode_wave_surface(surface)
            .into_iter()
            .filter_map(|atom| {
                self.package.graph.atom_id(atom.key).map(|atom_id| {
                    (
                        atom_id,
                        ObservedAtom {
                            position: (atom.position / 257).min(255) as u8,
                            weight: atom.weight,
                            channel: atom.key.channel,
                        },
                    )
                })
            })
            .collect()
    }

    pub(super) fn refresh_frontier_activations(
        &self,
        frontier: &mut [(u32, ForwardActivation)],
        observed: &BTreeMap<u32, ObservedAtom>,
    ) -> Option<Vec<Arc<[WaveCoupling]>>> {
        if matches!(self.relations, RelationStore::Eager) || frontier.len() < 2 {
            for (terminal_id, activation) in frontier {
                *activation = self.activation_for_terminal(*terminal_id, observed);
            }
            return None;
        }
        let (reverse, all_cached) = self.frontier_reverse_batch(frontier);
        if all_cached {
            for ((terminal_id, activation), relations) in frontier.iter_mut().zip(&reverse) {
                *activation =
                    self.activation_for_terminal_with_reverse(*terminal_id, observed, relations);
            }
            return Some(reverse);
        }
        v8::runtime_pool_install(|| {
            frontier.par_iter_mut().zip(reverse.par_iter()).for_each(
                |((terminal_id, activation), relations)| {
                    *activation = self.activation_for_terminal_with_reverse(
                        *terminal_id,
                        observed,
                        relations,
                    );
                },
            );
        });
        Some(reverse)
    }
    pub(super) fn exact_terminal_for_surface(&self, surface: &str) -> Option<u32> {
        let observed = self.resolve_surface(surface);
        if observed.is_empty() {
            return None;
        }
        let character_sequence = observed_sequence(&observed, AtomChannel::CharacterAnchor);
        let exact_terminals = self.exact_terminals(character_sequence.as_slice());
        (exact_terminals.len() == 1).then(|| *exact_terminals.first().expect("one terminal"))
    }

    pub(super) fn exact_terminals(&self, observed: &[u32]) -> BTreeSet<u32> {
        let hash = anchor_sequence_hash(observed);
        self.exact_surface_index
            .get(&hash)
            .into_iter()
            .chain(
                self.exact_surface_collisions
                    .get(&hash)
                    .into_iter()
                    .flatten(),
            )
            .filter_map(|terminal| {
                (self.character_anchors(*terminal) == observed).then_some(*terminal)
            })
            .collect()
    }

    pub(super) fn record_exact_terminals_for_chars(
        &self,
        chars: &[char],
        rank: u8,
        candidates: &mut BTreeMap<u32, u8>,
    ) {
        let Some(anchors) = self.anchor_sequence_for_chars(chars) else {
            return;
        };
        self.record_exact_terminals_for_anchors(anchors.as_slice(), rank, candidates);
    }

    pub(super) fn anchor_sequence_for_chars(&self, chars: &[char]) -> Option<AnchorSequence> {
        if chars.len() > MAX_ANCHOR_SEQUENCE {
            return None;
        }
        let mut anchors = AnchorSequence::default();
        for (index, ch) in chars.iter().enumerate() {
            anchors.atoms[index] = self.character_anchor_by_char.get(ch).copied()?;
        }
        anchors.len = chars.len() as u8;
        Some(anchors)
    }

    pub(super) fn record_exact_terminals_for_anchors(
        &self,
        anchors: &[u32],
        rank: u8,
        candidates: &mut BTreeMap<u32, u8>,
    ) {
        let hash = anchor_sequence_hash(anchors);
        for terminal_id in self
            .exact_surface_index
            .get(&hash)
            .into_iter()
            .chain(
                self.exact_surface_collisions
                    .get(&hash)
                    .into_iter()
                    .flatten(),
            )
            .filter_map(|terminal_id| {
                (self.character_anchors(*terminal_id) == anchors).then_some(*terminal_id)
            })
        {
            candidates
                .entry(terminal_id)
                .and_modify(|current| *current = (*current).min(rank))
                .or_insert(rank);
        }
    }

    pub(super) fn reconstruction_lane_reserve(
        &self,
        frontier: &[(u32, ForwardActivation)],
        observed: &[u32],
    ) -> Vec<(u32, ForwardActivation)> {
        let mut reserve = frontier
            .iter()
            .filter_map(|(terminal_id, activation)| {
                let expected = self.character_anchors(*terminal_id);
                let modes = reconstruction_modes(observed, &expected);
                (modes != 0).then_some((modes, *terminal_id, *activation))
            })
            .collect::<Vec<_>>();
        reserve.sort_unstable_by(|left, right| {
            reconstruction_mode_rank(right.0)
                .cmp(&reconstruction_mode_rank(left.0))
                .then_with(|| right.2.mass.cmp(&left.2.mass))
                .then_with(|| right.2.hits.cmp(&left.2.hits))
                .then_with(|| left.1.cmp(&right.1))
        });
        reserve.truncate(MAX_RECONSTRUCTION_RESERVE);
        reserve
            .into_iter()
            .map(|(_, terminal_id, activation)| (terminal_id, activation))
            .collect()
    }

    pub(super) fn operator_reserve(
        &self,
        surface: &str,
        observed: &BTreeMap<u32, ObservedAtom>,
        expand_inverse_lattice: bool,
    ) -> Vec<(u32, ForwardActivation)> {
        let normalized = normalize_lexical_surface(surface);
        let chars = normalized.chars().collect::<Vec<_>>();
        if chars.is_empty() || chars.len() > MAX_ANCHOR_SEQUENCE {
            return Vec::new();
        }

        let mut ranked = BTreeMap::<u32, u8>::new();
        let raw_projected = crate::dict::convert(
            surface.trim(),
            crate::dict::detect_direction(surface.trim()),
        );
        let raw_projected = normalize_lexical_surface(&raw_projected);
        let raw_projected_chars = raw_projected.chars().collect::<Vec<_>>();
        if raw_projected_chars != chars {
            self.record_exact_operator_candidates(&raw_projected_chars, 0, &mut ranked);
        }
        let projected =
            crate::dict::convert(&normalized, crate::dict::detect_direction(&normalized));
        let projected_chars = projected.chars().collect::<Vec<_>>();
        if projected_chars != chars {
            self.record_exact_operator_candidates(&projected_chars, 0, &mut ranked);
        }

        let predecessors = chars
            .iter()
            .copied()
            .map(crate::nanda_wave::surface_damage::alphabet_predecessor)
            .collect::<Vec<_>>();
        for first in 0..chars.len() {
            let Some(first_value) = predecessors[first] else {
                continue;
            };
            for second in first + 1..chars.len() {
                let Some(second_value) = predecessors[second] else {
                    continue;
                };
                let mut repaired = chars.clone();
                repaired[first] = first_value;
                repaired[second] = second_value;
                self.record_exact_operator_candidates(&repaired, 1, &mut ranked);
            }
        }
        for (index, predecessor) in predecessors.into_iter().enumerate() {
            let Some(predecessor) = predecessor else {
                continue;
            };
            let mut repaired = chars.clone();
            repaired[index] = predecessor;
            self.record_exact_operator_candidates(&repaired, 2, &mut ranked);
        }
        for first in 0..chars.len() {
            for second in first + 1..chars.len() {
                if chars[first] == chars[second] {
                    continue;
                }
                let mut repaired = chars.clone();
                repaired.swap(first, second);
                self.record_exact_operator_candidates(&repaired, 3, &mut ranked);
            }
        }
        for index in 0..chars.len() {
            let mut repaired = chars.clone();
            repaired.remove(index);
            self.record_exact_operator_candidates(&repaired, 4, &mut ranked);
        }
        if chars.len() >= 2 {
            for index in 0..chars.len() - 1 {
                let mut repaired = chars.clone();
                repaired.drain(index..=index + 1);
                self.record_exact_operator_candidates(&repaired, 5, &mut ranked);
            }
        }
        if expand_inverse_lattice && chars.len() <= MAX_EXACT_COLLISION_OPERATOR_CHARS {
            self.record_inverse_operator_candidates(&chars, &mut ranked);
        }
        if let Some(trace_terminal) = readout_trace_terminal() {
            eprintln!(
                "l11_trace_operator terminal={} raw_projected={} ranked={:?}",
                trace_terminal,
                raw_projected,
                ranked.get(&trace_terminal),
            );
        }

        let mut reserve = ranked
            .into_iter()
            .filter_map(|(terminal_id, rank)| {
                let activation = self.activation_for_terminal(terminal_id, observed);
                (activation.hits != 0).then_some((rank, terminal_id, activation))
            })
            .collect::<Vec<_>>();
        reserve.sort_unstable_by(|left, right| {
            left.0
                .cmp(&right.0)
                .then_with(|| right.2.mass.cmp(&left.2.mass))
                .then_with(|| right.2.hits.cmp(&left.2.hits))
                .then_with(|| left.1.cmp(&right.1))
        });
        reserve.truncate(MAX_OPERATOR_RESERVE);
        reserve
            .into_iter()
            .map(|(_, terminal_id, activation)| (terminal_id, activation))
            .collect()
    }

    pub(super) fn record_inverse_operator_candidates(
        &self,
        chars: &[char],
        candidates: &mut BTreeMap<u32, u8>,
    ) {
        let Some(alphabet) = chars
            .iter()
            .find_map(|ch| crate::nanda_wave::surface_damage::alphabet_for(*ch))
        else {
            return;
        };
        let Some(base) = self.anchor_sequence_for_chars(chars) else {
            return;
        };
        if base.as_slice().len() >= MAX_ANCHOR_SEQUENCE {
            return;
        }
        let inserted_atoms = alphabet
            .chars()
            .filter_map(|ch| self.character_anchor_by_char.get(&ch).copied())
            .collect::<Vec<_>>();
        for insert_at in 0..=base.as_slice().len() {
            for inserted in &inserted_atoms {
                let mut repaired = AnchorSequence::default();
                repaired.len = base.len.saturating_add(1);
                repaired.atoms[..insert_at].copy_from_slice(&base.as_slice()[..insert_at]);
                repaired.atoms[insert_at] = *inserted;
                repaired.atoms[insert_at + 1..usize::from(repaired.len)]
                    .copy_from_slice(&base.as_slice()[insert_at..]);
                self.record_exact_terminals_for_anchors(repaired.as_slice(), 3, candidates);
                for swap_at in 0..repaired.as_slice().len().saturating_sub(1) {
                    if repaired.atoms[swap_at] == repaired.atoms[swap_at + 1] {
                        continue;
                    }
                    repaired.atoms.swap(swap_at, swap_at + 1);
                    self.record_exact_terminals_for_anchors(repaired.as_slice(), 4, candidates);
                    repaired.atoms.swap(swap_at, swap_at + 1);
                }
            }
        }
    }

    pub(super) fn record_exact_operator_candidates(
        &self,
        chars: &[char],
        rank: u8,
        candidates: &mut BTreeMap<u32, u8>,
    ) {
        self.record_exact_terminals_for_chars(chars, rank, candidates);
    }

    pub(super) fn activation_for_terminal(
        &self,
        terminal_id: u32,
        observed: &BTreeMap<u32, ObservedAtom>,
    ) -> ForwardActivation {
        let reverse = self.reverse_couplings(terminal_id);
        self.activation_for_terminal_with_reverse(terminal_id, observed, &reverse)
    }

    pub(super) fn activation_for_terminal_with_reverse(
        &self,
        terminal_id: u32,
        observed: &BTreeMap<u32, ObservedAtom>,
        reverse: &[WaveCoupling],
    ) -> ForwardActivation {
        let Some(_) = self.package.centers.get(terminal_id as usize) else {
            return ForwardActivation::default();
        };
        let mut activation = ForwardActivation::default();
        for coupling in reverse.iter().filter(|coupling| coupling.flags == 0) {
            let Some(atom) = observed.get(&coupling.peer_id) else {
                continue;
            };
            let contribution = u64::from(coupling.strength)
                .saturating_mul(u64::from(atom.weight))
                .saturating_mul(u64::from(position_coherence(
                    atom.position,
                    coupling.position_mode,
                )));
            activation.mass = activation.mass.saturating_add(contribution);
            activation.hits = activation.hits.saturating_add(1);
            if is_keyboard_channel(atom.channel) {
                activation.keyboard_hits = activation.keyboard_hits.saturating_add(1);
            } else {
                activation.surface_hits = activation.surface_hits.saturating_add(1);
            }
        }
        activation
    }

    pub(super) fn geometry_reserve(
        &self,
        frontier: &[(u32, ForwardActivation)],
        observed: &[u32],
    ) -> Vec<(u32, ForwardActivation)> {
        let maximum_distance =
            usize::from(self.package.restoration_calibration.max_geometry_distance);
        let mut minimum_distance = usize::MAX;
        let mut measured = Vec::new();
        for (terminal_id, activation) in frontier {
            let expected = self.character_anchors(*terminal_id);
            if expected.len().abs_diff(observed.len()) > maximum_distance {
                continue;
            }
            let distance = damerau_distance(observed, &expected);
            if distance > maximum_distance {
                continue;
            }
            minimum_distance = minimum_distance.min(distance);
            measured.push((distance, *terminal_id, *activation));
        }
        let ambiguity_shell = minimum_distance.saturating_add(1).min(maximum_distance);
        measured.retain(|(distance, _, _)| *distance <= ambiguity_shell);
        measured.sort_unstable_by(|left, right| {
            left.0
                .cmp(&right.0)
                .then_with(|| right.2.mass.cmp(&left.2.mass))
                .then_with(|| right.2.hits.cmp(&left.2.hits))
                .then_with(|| left.1.cmp(&right.1))
        });
        measured.truncate(MAX_GEOMETRY_RESERVE);
        measured
            .into_iter()
            .map(|(_, terminal_id, activation)| (terminal_id, activation))
            .collect()
    }
}
