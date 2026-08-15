use super::*;

#[test]
fn decoded_surface_pool_preserves_terminal_utf8_bytes() {
    let surfaces = ["alpha", "бета", ""];
    let mut pool = DecodedSurfacePool::new(surfaces.len());
    for surface in surfaces {
        pool.push(Some(surface));
    }

    pool.validate(surfaces.len()).expect("complete pool");
    for (terminal_id, expected) in surfaces.into_iter().enumerate() {
        assert_eq!(pool.get(terminal_id as u32), Some(expected));
    }
    assert_eq!(pool.get(surfaces.len() as u32), None);
}

#[test]
fn decoded_surface_pool_rejects_incomplete_terminal_materialization() {
    let mut pool = DecodedSurfacePool::new(1);
    pool.push(None);
    assert!(pool.validate(1).is_err());
}

#[test]
fn candidate_birth_keeps_a_rare_budgeted_channel_frontier() {
    let mut channels: [Vec<BirthAtom>; 12] = std::array::from_fn(|_| Vec::new());
    channels[AtomChannel::CharacterGram as usize] = (0_u32..40)
        .map(|atom_id| {
            (
                (40 - atom_id) as usize,
                atom_id,
                ObservedAtom {
                    position: atom_id as u8,
                    weight: 1,
                    channel: AtomChannel::CharacterGram,
                },
            )
        })
        .collect();

    let selected = select_birth_atoms(
        &mut channels,
        DEFAULT_BIRTH_ATOMS_PER_CHANNEL,
        DEFAULT_BIRTH_POSTING_BUDGET,
    );

    assert_eq!(DEFAULT_BIRTH_ATOMS_PER_CHANNEL, 4);
    assert_eq!(selected.len(), 4);
    assert_eq!(selected.first().map(|atom| atom.1), Some(39));
    assert_eq!(selected.last().map(|atom| atom.1), Some(36));
}

#[test]
fn candidate_birth_stays_within_the_global_posting_budget() {
    let mut channels: [Vec<BirthAtom>; 12] = std::array::from_fn(|_| Vec::new());
    for (channel, atoms) in channels.iter_mut().take(3).enumerate() {
        *atoms = (0_u32..4)
            .map(|atom_id| {
                (
                    50_000,
                    atom_id + channel as u32 * 10,
                    ObservedAtom {
                        position: atom_id as u8,
                        weight: 1,
                        channel: AtomChannel::CharacterGram,
                    },
                )
            })
            .collect();
    }

    let selected = select_birth_atoms(&mut channels, 4, DEFAULT_BIRTH_POSTING_BUDGET);

    assert_eq!(selected.len(), 2);
    assert!(selected.iter().map(|atom| atom.0).sum::<usize>() <= DEFAULT_BIRTH_POSTING_BUDGET);
}

#[test]
fn geometry_reserve_keeps_the_nearest_basin_and_ambiguity_shell() {
    let anchor_sequences = [[1_u32, 9, 9], [1, 2, 4], [1, 2, 5]];
    let reverse_couplings = anchor_sequences
        .iter()
        .flatten()
        .map(|atom_id| WaveCoupling {
            peer_id: *atom_id,
            flags: COUPLING_FLAG_CHARACTER_ANCHOR,
            ..WaveCoupling::default()
        })
        .collect::<Vec<_>>();
    let centers = (0..anchor_sequences.len())
        .map(|terminal_id| super::super::crystal::WordCenter64 {
            coupling_start: (terminal_id * 3) as u32,
            coupling_count: 3,
            ..Default::default()
        })
        .collect();
    let memory = LexicalGrokkingMemory {
        package: LexicalGrokkingPackage {
            centers,
            reverse_couplings,
            restoration_calibration: super::super::restoration::RestorationCalibration {
                max_geometry_distance: 2,
                ..Default::default()
            },
            ..Default::default()
        },
        exact_surface_index: HashMap::new(),
        exact_surface_collisions: HashMap::new(),
        character_anchor_by_char: HashMap::new(),
        character_anchor_offsets: vec![0, 3, 6, 9],
        character_anchor_atoms: anchor_sequences.into_iter().flatten().collect(),
        relations: RelationStore::Eager,
        reverse_cache: Mutex::new(ReverseCache::default()),
        typed_basin: None,
        decoded_surface_pool: None,
    };
    let frontier = vec![
        (
            0,
            ForwardActivation {
                mass: 10_000,
                hits: 10,
                ..Default::default()
            },
        ),
        (
            1,
            ForwardActivation {
                mass: 100,
                hits: 2,
                ..Default::default()
            },
        ),
        (
            2,
            ForwardActivation {
                mass: 90,
                hits: 2,
                ..Default::default()
            },
        ),
    ];

    let reserve = memory.geometry_reserve(&frontier, &[1, 2, 3]);

    assert_eq!(
        reserve
            .into_iter()
            .map(|(terminal_id, _)| terminal_id)
            .collect::<Vec<_>>(),
        vec![1, 2, 0]
    );
}

#[test]
fn reconstruction_origin_does_not_override_geometry_evidence() {
    let primary = GrokkingCandidate {
        terminal_id: 1,
        geometry_distance: 2,
        settled_energy: 900,
        ..Default::default()
    };
    let reconstructed_tail = GrokkingCandidate {
        terminal_id: 2,
        reconstruction_only: true,
        reconstruction_modes: RECONSTRUCTION_MODE_DELETION,
        geometry_distance: 1,
        settled_energy: 1_000,
        ..Default::default()
    };
    let mut candidates = [primary, reconstructed_tail];

    apply_geometry_certificate_interference(&mut candidates);

    assert_eq!(candidates[0].terminal_id, reconstructed_tail.terminal_id);
    assert_eq!(candidates[1].terminal_id, primary.terminal_id);
}

#[test]
fn exact_two_omission_inverse_operator_can_cross_raw_edit_distance() {
    let nearer_incumbent = GrokkingCandidate {
        terminal_id: 1,
        geometry_distance: 1,
        settled_energy: 7_200,
        ..Default::default()
    };
    let two_omission_reconstruction = GrokkingCandidate {
        terminal_id: 2,
        reconstruction_modes: RECONSTRUCTION_MODE_DELETION,
        sequence_milli: 1_000,
        geometry_distance: 2,
        settled_energy: 6_000,
        ..Default::default()
    };
    let mut candidates = [nearer_incumbent, two_omission_reconstruction];

    apply_geometry_certificate_interference(&mut candidates);

    assert_eq!(
        candidates[0].terminal_id,
        two_omission_reconstruction.terminal_id
    );
}

#[test]
fn inverse_operator_cannot_spend_unbounded_energy_to_cross_distance() {
    let nearer_incumbent = GrokkingCandidate {
        terminal_id: 1,
        geometry_distance: 1,
        settled_energy: 8_000,
        ..Default::default()
    };
    let weak_two_omission_reconstruction = GrokkingCandidate {
        terminal_id: 2,
        reconstruction_modes: RECONSTRUCTION_MODE_DELETION,
        sequence_milli: 1_000,
        geometry_distance: 2,
        settled_energy: 3_500,
        ..Default::default()
    };
    let mut candidates = [nearer_incumbent, weak_two_omission_reconstruction];

    apply_geometry_certificate_interference(&mut candidates);

    assert_eq!(candidates[0].terminal_id, nearer_incumbent.terminal_id);
}

#[test]
fn rejected_high_rank_geometry_cannot_hide_an_admitted_later_challenger() {
    let incumbent = GrokkingCandidate {
        terminal_id: 1,
        geometry_distance: 1,
        settled_energy: 8_000,
        ..Default::default()
    };
    let rejected_high_rank = GrokkingCandidate {
        terminal_id: 2,
        reconstruction_modes: RECONSTRUCTION_MODE_DELETION_TRANSPOSITION,
        geometry_distance: 2,
        settled_energy: 6_000,
        ..Default::default()
    };
    let admitted_later = GrokkingCandidate {
        terminal_id: 3,
        reconstruction_modes: RECONSTRUCTION_MODE_DELETION,
        sequence_milli: 1_000,
        geometry_distance: 2,
        settled_energy: 7_000,
        ..Default::default()
    };
    let mut forward = vec![incumbent, rejected_high_rank, admitted_later];
    let mut permuted = vec![admitted_later, incumbent, rejected_high_rank];
    forward.sort_unstable_by(candidate_order);
    permuted.sort_unstable_by(candidate_order);

    apply_geometry_certificate_interference(&mut forward);
    apply_geometry_certificate_interference(&mut permuted);

    let forward_ids = forward
        .iter()
        .map(|candidate| candidate.terminal_id)
        .collect::<Vec<_>>();
    let permuted_ids = permuted
        .iter()
        .map(|candidate| candidate.terminal_id)
        .collect::<Vec<_>>();
    assert_eq!(
        forward_ids,
        vec![
            admitted_later.terminal_id,
            incumbent.terminal_id,
            rejected_high_rank.terminal_id
        ]
    );
    assert_eq!(permuted_ids, forward_ids);
}

#[test]
fn two_omission_operator_does_not_displace_a_stronger_one_omission_inverse() {
    let one_omission_inverse = GrokkingCandidate {
        terminal_id: 1,
        reconstruction_modes: RECONSTRUCTION_MODE_SINGLE_DELETION,
        sequence_milli: 1_000,
        geometry_distance: 1,
        settled_energy: 8_200,
        ..Default::default()
    };
    let weaker_two_omission_inverse = GrokkingCandidate {
        terminal_id: 2,
        reconstruction_modes: RECONSTRUCTION_MODE_DELETION,
        sequence_milli: 1_000,
        geometry_distance: 2,
        settled_energy: 7_800,
        ..Default::default()
    };
    let mut candidates = [one_omission_inverse, weaker_two_omission_inverse];

    apply_geometry_certificate_interference(&mut candidates);

    assert_eq!(candidates[0].terminal_id, one_omission_inverse.terminal_id);
}

#[test]
fn exact_boundary_truncation_outranks_a_two_omission_completion() {
    let two_omission_completion = GrokkingCandidate {
        terminal_id: 1,
        reconstruction_modes: RECONSTRUCTION_MODE_DELETION,
        sequence_milli: 1_000,
        geometry_distance: 2,
        settled_energy: 8_000,
        ..Default::default()
    };
    let suffix_completion = GrokkingCandidate {
        terminal_id: 2,
        reconstruction_modes: RECONSTRUCTION_MODE_SUFFIX_TRUNCATION,
        sequence_milli: 1_000,
        geometry_distance: 1,
        settled_energy: 7_500,
        ..Default::default()
    };
    let mut candidates = [two_omission_completion, suffix_completion];

    apply_geometry_certificate_interference(&mut candidates);

    assert_eq!(candidates[0].terminal_id, suffix_completion.terminal_id);
}

#[test]
fn direct_surface_operator_outranks_cross_script_keyboard_projection() {
    let keyboard_projection = GrokkingCandidate {
        terminal_id: 1,
        keyboard_hits: 20,
        surface_hits: 2,
        geometry_distance: 2,
        settled_energy: 1_000,
        ..Default::default()
    };
    let double_substitution = GrokkingCandidate {
        terminal_id: 2,
        reconstruction_modes: RECONSTRUCTION_MODE_DOUBLE_SUBSTITUTION,
        keyboard_hits: 10,
        surface_hits: 20,
        geometry_distance: 2,
        settled_energy: 900,
        ..Default::default()
    };
    let mut candidates = [keyboard_projection, double_substitution];

    apply_geometry_certificate_interference(&mut candidates);

    assert_eq!(candidates[0].terminal_id, double_substitution.terminal_id);
}

#[test]
fn deletion_transposition_operator_outranks_cross_script_keyboard_projection() {
    let keyboard_projection = GrokkingCandidate {
        terminal_id: 1,
        keyboard_hits: 20,
        surface_hits: 2,
        geometry_distance: 1,
        settled_energy: 1_000,
        ..Default::default()
    };
    let deletion_transposition = GrokkingCandidate {
        terminal_id: 2,
        reconstruction_modes: RECONSTRUCTION_MODE_DELETION_TRANSPOSITION,
        keyboard_hits: 10,
        surface_hits: 20,
        geometry_distance: 2,
        settled_energy: 900,
        ..Default::default()
    };
    let mut candidates = [keyboard_projection, deletion_transposition];

    apply_geometry_certificate_interference(&mut candidates);

    assert_eq!(
        candidates[0].terminal_id,
        deletion_transposition.terminal_id
    );
}

#[test]
fn reconstruction_evidence_survives_bounded_lattice_without_reordering_primary() {
    let mut candidates = (0..6)
        .map(|terminal_id| GrokkingCandidate {
            terminal_id,
            reconstruction_modes: if terminal_id >= 4 {
                RECONSTRUCTION_MODE_DELETION
            } else {
                0
            },
            ..Default::default()
        })
        .collect::<Vec<_>>();

    truncate_with_reconstruction_tail(&mut candidates, 4);

    assert_eq!(
        candidates
            .iter()
            .map(|candidate| candidate.terminal_id)
            .collect::<Vec<_>>(),
        vec![0, 1, 4, 5]
    );
}

#[test]
fn geometry_shell_evidence_survives_bounded_lattice() {
    let mut candidates = (0..6)
        .map(|terminal_id| GrokkingCandidate {
            terminal_id,
            ambiguity_shell: terminal_id == 5,
            geometry_distance: if terminal_id == 5 { 0 } else { 1 },
            settled_energy: 1_000 - terminal_id as i32,
            ..Default::default()
        })
        .collect::<Vec<_>>();

    truncate_with_reconstruction_tail(&mut candidates, 4);

    assert_eq!(candidates.len(), 4);
    assert!(candidates
        .iter()
        .any(|candidate| candidate.terminal_id == 5));
}

#[test]
fn adjacent_swap_plus_omission_matcher_is_exact_without_heap_storage() {
    assert!(is_subsequence_after_one_adjacent_swap(
        &[1, 3, 2, 4],
        &[1, 2, 3, 4, 5]
    ));
    assert!(is_subsequence_after_one_adjacent_swap(
        &[2, 1, 4],
        &[1, 2, 3, 4]
    ));
    assert!(is_subsequence_after_one_adjacent_swap(
        &[1, 4, 2],
        &[1, 2, 3, 4]
    ));
    assert!(!is_subsequence_after_one_adjacent_swap(
        &[1, 2, 3],
        &[1, 2, 3, 4]
    ));
}

#[test]
fn adjacent_swap_plus_omission_matcher_preserves_reference_semantics() {
    fn reference(observed: &[u32], expected: &[u32]) -> bool {
        if observed.len() < 2 {
            return false;
        }
        let mut repaired = observed.to_vec();
        for index in 0..observed.len() - 1 {
            repaired.swap(index, index + 1);
            if is_ordered_subsequence(&repaired, expected) {
                return true;
            }
            repaired.swap(index, index + 1);
        }
        false
    }

    fn surfaces(length: usize) -> Vec<Vec<u32>> {
        let count = 3_usize.pow(length as u32);
        (0..count)
            .map(|mut encoded| {
                let mut surface = vec![0; length];
                for symbol in &mut surface {
                    *symbol = (encoded % 3) as u32;
                    encoded /= 3;
                }
                surface
            })
            .collect()
    }

    for observed_length in 2..=4 {
        for observed in surfaces(observed_length) {
            for expected in surfaces(observed_length + 1) {
                assert_eq!(
                    is_subsequence_after_one_adjacent_swap(&observed, &expected),
                    reference(&observed, &expected),
                    "observed={observed:?} expected={expected:?}"
                );
            }
        }
    }
}

#[test]
fn adjacent_transposition_is_an_exact_surface_operator() {
    assert_eq!(
        surface_operator_reconstruction_modes("ba", "ab"),
        RECONSTRUCTION_MODE_NON_ADJACENT_TRANSPOSITION
    );
}

#[test]
fn bounded_tail_keeps_the_strongest_operator_evidence() {
    let mut candidates = (0..100)
        .map(|terminal_id| GrokkingCandidate {
            terminal_id,
            reconstruction_modes: if terminal_id >= 64 {
                RECONSTRUCTION_MODE_SINGLE_DELETION
            } else {
                0
            },
            geometry_distance: 1,
            settled_energy: 2_000 - terminal_id as i32,
            ..Default::default()
        })
        .collect::<Vec<_>>();
    candidates[99].reconstruction_modes = RECONSTRUCTION_MODE_DOUBLE_SUBSTITUTION;

    truncate_with_reconstruction_tail(&mut candidates, 64);

    assert_eq!(candidates.len(), 64);
    assert!(candidates
        .iter()
        .any(|candidate| candidate.terminal_id == 99));
}

#[test]
fn exact_surface_keeps_clean_fast_path_but_expands_lattice_readout() {
    assert!(!should_expand_operator_lattice(1, 1));
    assert!(should_expand_operator_lattice(1, 64));
    assert!(should_expand_operator_lattice(0, 1));
}

#[test]
fn bounded_tail_does_not_evict_operator_evidence_already_inside_limit() {
    let mut candidates = (0..100)
        .map(|terminal_id| GrokkingCandidate {
            terminal_id,
            reconstruction_modes: if terminal_id == 42 || terminal_id >= 64 {
                RECONSTRUCTION_MODE_SINGLE_DELETION
            } else {
                0
            },
            geometry_distance: 1,
            settled_energy: 2_000 - terminal_id as i32,
            ..Default::default()
        })
        .collect::<Vec<_>>();

    truncate_with_reconstruction_tail(&mut candidates, 64);

    assert_eq!(candidates.len(), 64);
    assert!(candidates
        .iter()
        .any(|candidate| candidate.terminal_id == 42));
    assert!(candidates
        .iter()
        .any(|candidate| candidate.terminal_id >= 64));
}
