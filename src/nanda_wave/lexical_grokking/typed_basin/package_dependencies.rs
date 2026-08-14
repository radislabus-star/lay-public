use super::super::model::LexicalGrokkingPackage;

#[derive(Clone, Debug, Default)]
pub(super) struct PackageDependencyAudit {
    pub(super) center_phase_profiles: usize,
    pub(super) positive_subcenters: usize,
    pub(super) anti_subcenters: usize,
    pub(super) hard_negative_subcenters: usize,
    pub(super) ambiguity_subcenters: usize,
    pub(super) keyboard_geometry_units: usize,
    pub(super) directional_anti_centers: usize,
    pub(super) pair_profiles: usize,
    pub(super) pair_centers: usize,
    pub(super) referenced_positive: usize,
    pub(super) referenced_anti: usize,
    pub(super) referenced_hard_negative: usize,
    pub(super) referenced_ambiguity: usize,
    pub(super) referenced_keyboard_geometry: usize,
    pub(super) referenced_directional_anti: usize,
    pub(super) referenced_pair_centers: usize,
    pub(super) invalid_references: usize,
    pub(super) max_geometry_distance: u8,
    pub(super) min_positive_milli: u16,
    pub(super) min_backward_milli: u16,
    pub(super) min_tied_energy_margin: u16,
    pub(super) unresolved: Vec<String>,
}

impl PackageDependencyAudit {
    pub(super) fn inspect(package: &LexicalGrokkingPackage) -> Self {
        let mut audit = Self {
            center_phase_profiles: package.center_phase_profiles.len(),
            positive_subcenters: package.positive_subcenters.len(),
            anti_subcenters: package.anti_subcenters.len(),
            hard_negative_subcenters: package.hard_negative_subcenters.len(),
            ambiguity_subcenters: package.ambiguity_subcenters.len(),
            keyboard_geometry_units: package.keyboard_geometry_units.len(),
            directional_anti_centers: package.anti_centers.len(),
            pair_profiles: package.pair_profiles.len(),
            pair_centers: package.pair_centers.len(),
            max_geometry_distance: package.restoration_calibration.max_geometry_distance,
            min_positive_milli: package.restoration_calibration.min_positive_milli,
            min_backward_milli: package.restoration_calibration.min_backward_milli,
            min_tied_energy_margin: package.restoration_calibration.min_tied_energy_margin,
            ..Self::default()
        };
        for profile in &package.center_phase_profiles {
            audit.referenced_positive = audit
                .referenced_positive
                .saturating_add(profile.positive_count as usize);
            audit.referenced_anti = audit
                .referenced_anti
                .saturating_add(profile.anti_count as usize);
            audit.referenced_hard_negative = audit
                .referenced_hard_negative
                .saturating_add(profile.hard_negative_count as usize);
            audit.referenced_ambiguity = audit
                .referenced_ambiguity
                .saturating_add(profile.ambiguity_count as usize);
            audit.referenced_keyboard_geometry = audit
                .referenced_keyboard_geometry
                .saturating_add(profile.keyboard_geometry_count as usize);
            audit.invalid_references += usize::from(!valid_range(
                profile.positive_start,
                profile.positive_count,
                package.positive_subcenters.len(),
            ));
            audit.invalid_references += usize::from(!valid_range(
                profile.anti_start,
                profile.anti_count,
                package.anti_subcenters.len(),
            ));
            audit.invalid_references += usize::from(!valid_range(
                profile.hard_negative_start,
                profile.hard_negative_count,
                package.hard_negative_subcenters.len(),
            ));
            audit.invalid_references += usize::from(!valid_range(
                profile.ambiguity_start,
                profile.ambiguity_count,
                package.ambiguity_subcenters.len(),
            ));
            audit.invalid_references += usize::from(!valid_range(
                profile.keyboard_geometry_start,
                profile.keyboard_geometry_count,
                package.keyboard_geometry_units.len(),
            ));
        }
        for center in &package.centers {
            audit.referenced_directional_anti = audit
                .referenced_directional_anti
                .saturating_add(center.anti_count as usize);
            audit.invalid_references += usize::from(!valid_range(
                center.anti_start,
                center.anti_count,
                package.anti_centers.len(),
            ));
        }
        for profile in &package.pair_profiles {
            audit.referenced_pair_centers = audit
                .referenced_pair_centers
                .saturating_add(profile.low_wins_count as usize)
                .saturating_add(profile.high_wins_count as usize);
            audit.invalid_references += usize::from(!valid_range(
                profile.low_wins_start,
                profile.low_wins_count,
                package.pair_centers.len(),
            ));
            audit.invalid_references += usize::from(!valid_range(
                profile.high_wins_start,
                profile.high_wins_count,
                package.pair_centers.len(),
            ));
        }
        audit.record_unresolved();
        audit
    }

    fn record_unresolved(&mut self) {
        if self.invalid_references != 0 {
            self.unresolved
                .push("invalid package bank references".to_string());
        }
        if self.positive_subcenters != 0 || self.referenced_positive != 0 {
            self.unresolved
                .push("positive subcenter bank requires an A2 adapter".to_string());
        }
        if self.anti_subcenters != 0 || self.referenced_anti != 0 {
            self.unresolved
                .push("anti subcenter bank requires an A2 adapter".to_string());
        }
        if self.hard_negative_subcenters != 0 || self.referenced_hard_negative != 0 {
            self.unresolved
                .push("hard-negative bank requires an A2 adapter".to_string());
        }
        if self.ambiguity_subcenters != 0 || self.referenced_ambiguity != 0 {
            self.unresolved
                .push("ambiguity bank requires exact owner/competitor reverse".to_string());
        }
        if self.directional_anti_centers != 0 || self.referenced_directional_anti != 0 {
            self.unresolved
                .push("directional anti bank is unresolved in A2".to_string());
        }
        if self.pair_profiles != 0 || self.pair_centers != 0 || self.referenced_pair_centers != 0 {
            self.unresolved
                .push("pairwise residual bank is unresolved in A2".to_string());
        }
    }

    pub(super) fn resolved(&self) -> bool {
        self.unresolved.is_empty()
    }

    pub(super) fn report(&self) -> serde_json::Value {
        serde_json::json!({
            "physical_vectors": {
                "center_phase_profiles": self.center_phase_profiles,
                "positive_subcenters": self.positive_subcenters,
                "anti_subcenters": self.anti_subcenters,
                "hard_negative_subcenters": self.hard_negative_subcenters,
                "ambiguity_subcenters": self.ambiguity_subcenters,
                "keyboard_geometry_units": self.keyboard_geometry_units,
                "directional_anti_centers": self.directional_anti_centers,
                "pair_profiles": self.pair_profiles,
                "pair_centers": self.pair_centers,
            },
            "summed_references": {
                "positive": self.referenced_positive,
                "anti": self.referenced_anti,
                "hard_negative": self.referenced_hard_negative,
                "ambiguity": self.referenced_ambiguity,
                "keyboard_geometry": self.referenced_keyboard_geometry,
                "directional_anti": self.referenced_directional_anti,
                "pair_centers": self.referenced_pair_centers,
            },
            "restoration_calibration": {
                "max_geometry_distance": self.max_geometry_distance,
                "min_positive_milli": self.min_positive_milli,
                "min_backward_milli": self.min_backward_milli,
                "min_tied_energy_margin": self.min_tied_energy_margin,
            },
            "invalid_references": self.invalid_references,
            "unresolved": self.unresolved,
            "a2_resolved": self.resolved(),
            "keyboard_geometry_policy": "package-backed_or_lazy_generated_non_reverse_dependency",
        })
    }
}

fn valid_range(start: u32, count: impl Into<usize>, len: usize) -> bool {
    (start as usize)
        .checked_add(count.into())
        .is_some_and(|end| end <= len)
}
