use super::*;

impl OnlineContextPhaseLearner {
    pub(in crate::nanda_wave::context_phase) fn prepare_supervised_sentence_basis(
        &mut self,
        scene_tokens: &[String],
    ) {
        let exact_hashes = scene_tokens
            .iter()
            .map(|token| super::super::context_exact_hash(token))
            .collect::<Vec<_>>();
        let frequencies = exact_hashes
            .iter()
            .map(|hash| self.admission_frequency.observe(*hash))
            .collect::<Vec<_>>();
        self.update_semantic_states(&exact_hashes, &frequencies);
    }

    /// Learns one causally labelled sentence slot against the complete bounded
    /// L2 competitor set. The caller owns episode independence; this method
    /// never infers a target from a shown or automatically applied candidate.
    pub(in crate::nanda_wave::context_phase) fn ingest_supervised_sentence(
        &mut self,
        scene_tokens: &[String],
        pair_views: &[Vec<String>],
        target: &str,
        competitors: &[String],
    ) {
        if scene_tokens.is_empty() || target.trim().is_empty() {
            return;
        }
        self.prepare_supervised_sentence_basis(scene_tokens);
        self.ingest_supervised_sentence_on_prepared_basis(
            scene_tokens,
            pair_views,
            target,
            competitors,
        );
    }

    /// Reduces labelled evidence against a basis prepared from the complete
    /// bounded episode batch. This keeps phase coordinates stationary while
    /// independent observations reinforce or split directional subcenters.
    pub(in crate::nanda_wave::context_phase) fn ingest_supervised_sentence_on_prepared_basis(
        &mut self,
        scene_tokens: &[String],
        pair_views: &[Vec<String>],
        target: &str,
        competitors: &[String],
    ) {
        if scene_tokens.is_empty() || target.trim().is_empty() {
            return;
        }
        self.stats.fragments = self.stats.fragments.saturating_add(1);

        let target = target.to_lowercase();
        let target_hash = hash_text(&target);
        let target_frequency = self.admission_frequency.observe(target_hash);
        // A supervised episode already carries an explicit causal label. Count
        // it from the first observation instead of spending that observation
        // on the unsupervised frequency warm-up; profile/RSS bounds and the
        // snapshot support threshold remain unchanged.
        let supervised_frequency =
            target_frequency.max(self.config.min_profile_support.min(u32::from(u16::MAX)) as u16);
        if !self.ensure_profile(target_hash, supervised_frequency) {
            return;
        }
        let context_hashes =
            super::super::context_atom_hashes(scene_tokens, self.config.signature_schema);
        let target_relation_role = self.config.signature_schema
            >= super::super::SIGNATURE_SCHEMA_RELATION_ROLES
            && super::super::relation_role_candidate(&target);
        let target_vector =
            self.relation_vector(&context_hashes, target_hash, target_relation_role);
        if !self.update_profile_bank(target_hash, ProfileBank::Positive, &target_vector) {
            return;
        }
        let target_examples = {
            let profile = self.profiles.get_mut(&target_hash).expect("profile exists");
            profile.positive_examples = profile.positive_examples.saturating_add(1);
            profile.positive_examples
        };
        self.stats.transitions = self.stats.transitions.saturating_add(1);
        let target_margin = self
            .profile_margin_micro(target_hash, &target_vector)
            .unwrap_or_default();
        self.profiles
            .get_mut(&target_hash)
            .expect("profile exists")
            .positive_calibration
            .observe(
                micro_i32(target_margin),
                target_hash ^ self.stats.transitions,
            );

        let mut seen = BTreeSet::new();
        let competitors = competitors
            .iter()
            .filter_map(|competitor| {
                let competitor = competitor.to_lowercase();
                let hash = hash_text(&competitor);
                (hash != target_hash && seen.insert(hash)).then(|| {
                    let relation_role = self.config.signature_schema
                        >= super::super::SIGNATURE_SCHEMA_RELATION_ROLES
                        && super::super::relation_role_candidate(&competitor);
                    (
                        hash,
                        super::super::candidate_l2_signature_for_schema(
                            &competitor,
                            self.config.signature_schema,
                        ),
                        relation_role,
                    )
                })
            })
            .take(MAX_COMPETITORS)
            .collect::<Vec<_>>();
        self.stats.l2_lattice_probes = self.stats.l2_lattice_probes.saturating_add(1);
        self.stats.l2_lattice_negative_examples = self
            .stats
            .l2_lattice_negative_examples
            .saturating_add(competitors.len() as u64);
        self.stats.l2_lattice_max_competitors = self
            .stats
            .l2_lattice_max_competitors
            .max(competitors.len() as u32);
        if competitors.is_empty() {
            self.stats.l2_lattice_empty_results =
                self.stats.l2_lattice_empty_results.saturating_add(1);
            return;
        }
        let competitor_roles = competitors
            .iter()
            .map(|(hash, _, role)| (*hash, *role))
            .collect::<Vec<_>>();
        self.competition_calibration.observe(
            CompetitionCalibrationCase::new(
                target_hash,
                target_relation_role,
                &context_hashes,
                &competitor_roles,
            ),
            target_hash ^ self.stats.transitions.rotate_left(17),
        );
        let relation_scene = target_relation_role
            || competitors
                .iter()
                .any(|(_, _, relation_role)| *relation_role);
        let scene = self.relation_vector(&context_hashes, 0, relation_scene);
        let pair_scenes = pair_views
            .iter()
            .filter(|view| view.len() >= 2)
            .map(|view| {
                let hashes = super::super::context_atom_hashes(view, self.config.signature_schema);
                self.relation_vector(&hashes, 0, relation_scene)
            })
            .collect::<Vec<_>>();
        let target_signature =
            super::super::candidate_l2_signature_for_schema(&target, self.config.signature_schema);
        self.update_signature_positive(target_signature, &scene);
        for (hash, signature, _) in competitors {
            if relation_scene {
                self.update_signature_negative(signature, &scene);
            }
            if pair_scenes.is_empty() {
                self.update_pair_winner(target_hash, hash, &scene, false);
                self.update_pair_relation(target_hash, target_signature, hash, signature, &scene);
            } else {
                for (view_index, pair_scene) in pair_scenes.iter().enumerate() {
                    self.update_pair_view_winner(target_hash, hash, pair_scene, view_index);
                    self.update_pair_view_relation(
                        target_hash,
                        target_signature,
                        hash,
                        signature,
                        pair_scene,
                        view_index,
                    );
                }
            }
        }
        debug_assert!(target_examples > 0);
    }

    fn update_pair_view_winner(
        &mut self,
        winner: u64,
        loser: u64,
        scene: &[PhaseCell],
        view_index: usize,
    ) {
        let winner = super::super::pair_view_hash(winner, view_index);
        let loser = super::super::pair_view_hash(loser, view_index);
        let Some(key) = PairKey::new(winner, loser) else {
            return;
        };
        self.update_pair_key(key, winner == key.low_hash, scene, false, None);
    }

    fn update_pair_view_relation(
        &mut self,
        winner: u64,
        winner_signature: u64,
        loser: u64,
        loser_signature: u64,
        scene: &[PhaseCell],
        view_index: usize,
    ) {
        let source = PairKey::new(winner, loser).map(pair_key_hash);
        let winner = super::super::pair_view_hash(winner, view_index);
        let loser = super::super::pair_view_hash(loser, view_index);
        let winner_signature = super::super::pair_view_hash(winner_signature, view_index);
        let loser_signature = super::super::pair_view_hash(loser_signature, view_index);
        let Some(key) = PairKey::relation(winner, winner_signature, loser, loser_signature) else {
            return;
        };
        self.update_pair_key(
            key,
            winner_signature < loser_signature,
            scene,
            false,
            source,
        );
    }
}
