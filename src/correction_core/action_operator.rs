use super::{edit_transition, TypingErrorClass};
use crate::language_action::{
    operator_for_candidate, proof_for_candidate, LanguageActionOperator, LanguageActionProof,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct CorrectionActionOperatorReport {
    pub(super) operator: LanguageActionOperator,
    pub(super) proof: LanguageActionProof,
    pub(super) edit_operator: edit_transition::EditTransitionOperator,
    pub(super) edit_proof: LanguageActionProof,
    pub(super) verifier_required: bool,
    pub(super) verifier_passed: bool,
    pub(super) left_context_changed: bool,
    pub(super) changed_tokens: usize,
    blocker: Option<&'static str>,
}

impl CorrectionActionOperatorReport {
    pub(super) const fn apply_blocker(self) -> Option<&'static str> {
        if !self.verifier_required {
            return None;
        }
        self.blocker
    }
}

pub(super) fn verify_action_operator(
    original: &str,
    replacement: &str,
    error_class: TypingErrorClass,
    source_id: &str,
) -> CorrectionActionOperatorReport {
    let operator = operator_for_candidate(error_class, source_id);
    let proof = proof_for_candidate(error_class, source_id);
    let transition =
        edit_transition::prove_edit_transition(original, replacement, error_class, source_id);

    CorrectionActionOperatorReport {
        operator,
        proof,
        edit_operator: transition.operator,
        edit_proof: transition.language_proof,
        verifier_required: original != replacement,
        verifier_passed: transition.verified,
        left_context_changed: transition.left_context_changed,
        changed_tokens: transition.changed_tokens,
        blocker: transition.reject_apply_reason(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_verified_layout_projection() {
        let report = verify_action_operator(
            "HF<JNF NTCN CFV ",
            "РАБОТА ТЕСТ САМ ",
            TypingErrorClass::WrongLayout,
            crate::typing_rule_graph::ids::LAYOUT_EN_TO_RU,
        );

        assert_eq!(report.operator, LanguageActionOperator::FlipLayout);
        assert_eq!(
            report.edit_operator,
            edit_transition::EditTransitionOperator::LayoutProjection
        );
        assert!(report.verifier_required);
        assert!(report.verifier_passed);
        assert_eq!(report.apply_blocker(), None);
    }

    #[test]
    fn blocks_unverified_left_context_import() {
        let report = verify_action_operator(
            "содержкой ",
            "что получилось вроде хороший ввод и даже фикс был шикарный но с содержать ",
            TypingErrorClass::CompositeTypo,
            "L2SurfaceMotifCell32",
        );

        assert_eq!(report.operator, LanguageActionOperator::FixTypo);
        assert_eq!(
            report.edit_operator,
            edit_transition::EditTransitionOperator::Unknown
        );
        assert!(report.verifier_required);
        assert!(!report.verifier_passed);
        assert_eq!(report.apply_blocker(), Some("edit_transition_not_verified"));
    }
}
