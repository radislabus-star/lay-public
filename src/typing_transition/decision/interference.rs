use crate::nanda_wave::{PhaseReadout, PhaseVerdict};

#[derive(Debug, Clone, Copy)]
pub(super) struct TransitionInterferenceInput {
    pub(super) l2_rank_energy: f32,
    pub(super) l2_uncertainty: f32,
    pub(super) phase: PhaseReadout,
    pub(super) phase_competition: Option<f32>,
    pub(super) l3_rank_energy: f32,
    pub(super) l4_scene_rank_energy: f32,
    pub(super) l4_signed_rank_energy: f32,
}

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub(super) struct TransitionInterferenceReadout {
    pub(super) signal: f32,
    pub(super) attraction: f32,
    pub(super) repulsion: f32,
    pub(super) uncertainty: f32,
    pub(super) phase_competition: f32,
}

pub(super) fn read_transition_interference(
    input: TransitionInterferenceInput,
) -> TransitionInterferenceReadout {
    let phase_competition = input.phase_competition.unwrap_or_default().clamp(-1.0, 1.0);
    let l2_energy = settle_l2_energy(input.l2_rank_energy, input.phase, input.phase_competition);
    let energies = [
        l2_energy,
        input.l3_rank_energy,
        input.l4_scene_rank_energy,
        input.l4_signed_rank_energy,
    ];
    let attraction = energies.iter().map(|energy| energy.max(0.0)).sum::<f32>();
    let repulsion = energies
        .iter()
        .map(|energy| (-energy).max(0.0))
        .sum::<f32>();
    let phase_uncertainty = (input.phase.package_loaded
        && input.phase.operator_present
        && input.phase.operator_promoted
        && input.phase.verdict == PhaseVerdict::Unknown) as u8 as f32;
    let uncertainty = input.l2_uncertainty.max(phase_uncertainty).clamp(0.0, 1.0);

    TransitionInterferenceReadout {
        signal: (attraction - repulsion).clamp(-1.0, 1.0),
        attraction: attraction.clamp(0.0, 1.0),
        repulsion: repulsion.clamp(0.0, 1.0),
        uncertainty,
        phase_competition,
    }
}

pub(super) fn normalized_phase_support_strength(phase: PhaseReadout) -> Option<f32> {
    if !phase.package_loaded
        || !phase.operator_present
        || !phase.operator_promoted
        || phase.verdict != PhaseVerdict::Support
        || phase.margin_micro < phase.threshold_micro
    {
        return None;
    }
    let threshold_micro = phase.threshold_micro.clamp(-1_000_000, 999_999);
    let threshold = threshold_micro as f64;
    let margin = phase.margin_micro.clamp(threshold_micro, 1_000_000) as f64;
    Some(((margin - threshold) / (1_000_000.0 - threshold)).clamp(0.0, 1.0) as f32)
}

fn settle_l2_energy(
    surface_energy: f32,
    phase: PhaseReadout,
    phase_competition: Option<f32>,
) -> f32 {
    let Some(competition) = phase_competition else {
        return surface_energy;
    };
    if !phase.package_loaded
        || !phase.operator_present
        || !phase.operator_promoted
        || phase.verdict != PhaseVerdict::Support
    {
        return surface_energy;
    }

    // The phase field redistributes the released L2 ranking budget. It cannot
    // create a second bonus beside L2 and therefore cannot grow authority by
    // merely adding another scorer.
    let budget = surface_energy.abs().max(f32::EPSILON);
    let phase_energy = competition.clamp(-1.0, 1.0) * budget;
    (surface_energy + phase_energy) * 0.5
}

#[cfg(test)]
mod tests {
    use super::*;

    fn promoted_phase(verdict: PhaseVerdict, margin_micro: i64) -> PhaseReadout {
        PhaseReadout {
            package_loaded: true,
            operator_present: true,
            operator_promoted: true,
            margin_micro,
            verdict,
            ..PhaseReadout::default()
        }
    }

    fn input(phase_competition: Option<f32>) -> TransitionInterferenceInput {
        TransitionInterferenceInput {
            l2_rank_energy: 0.28,
            l2_uncertainty: 0.10,
            phase: promoted_phase(PhaseVerdict::Support, 240_000),
            phase_competition,
            l3_rank_energy: 0.08,
            l4_scene_rank_energy: 0.03,
            l4_signed_rank_energy: 0.04,
        }
    }

    #[test]
    fn no_phase_competition_preserves_the_released_energy_budget() {
        let readout = read_transition_interference(input(None));

        assert!((readout.signal - 0.43).abs() < 0.0001, "{readout:?}");
        assert_eq!(readout.repulsion, 0.0);
    }

    #[test]
    fn learned_phase_competition_redistributes_l2_energy_without_growing_it() {
        let strongest = read_transition_interference(input(Some(1.0)));
        let weakest = read_transition_interference(input(Some(-1.0)));

        assert!(
            strongest.signal > weakest.signal,
            "{strongest:?} {weakest:?}"
        );
        assert!(strongest.signal <= 0.43, "{strongest:?}");
        assert!(weakest.signal >= 0.15, "{weakest:?}");
    }

    #[test]
    fn repel_does_not_receive_positive_competition_authority() {
        let mut repelled = input(Some(1.0));
        repelled.phase = promoted_phase(PhaseVerdict::Repel, -240_000);
        let readout = read_transition_interference(repelled);

        assert!((readout.signal - 0.43).abs() < 0.0001, "{readout:?}");
    }

    #[test]
    fn phase_strength_is_normalized_against_each_profiles_learned_threshold() {
        let mut low_threshold = promoted_phase(PhaseVerdict::Support, 200_000);
        low_threshold.threshold_micro = 100_000;
        let mut high_threshold = promoted_phase(PhaseVerdict::Support, 550_000);
        high_threshold.threshold_micro = 500_000;

        let low = normalized_phase_support_strength(low_threshold).unwrap();
        let high = normalized_phase_support_strength(high_threshold).unwrap();

        assert!(low > high, "low={low} high={high}");
        assert!(
            normalized_phase_support_strength(promoted_phase(PhaseVerdict::Repel, -200_000))
                .is_none()
        );
    }

    #[test]
    fn corrupt_threshold_is_bounded_instead_of_panicking() {
        let mut phase = promoted_phase(PhaseVerdict::Support, i64::MAX);
        phase.threshold_micro = i64::MAX;

        assert_eq!(normalized_phase_support_strength(phase), Some(1.0));
    }
}
