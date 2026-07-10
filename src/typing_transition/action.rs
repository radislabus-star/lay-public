use super::verifier;
use crate::correction_core::TypingErrorClass;
use crate::correction_source_contract::CandidateOrigin;
use crate::language_action::{
    operator_for_origin, proof_for_origin, LanguageActionOperator, LanguageActionProof,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct CorrectionActionOperatorReport {
    pub(crate) operator: LanguageActionOperator,
    pub(crate) proof: LanguageActionProof,
    pub(crate) edit_operator: verifier::EditTransitionOperator,
    pub(crate) edit_proof: LanguageActionProof,
    pub(crate) verifier_required: bool,
    pub(crate) verifier_passed: bool,
    pub(crate) left_context_changed: bool,
    pub(crate) changed_tokens: usize,
    blocker: Option<&'static str>,
}

impl CorrectionActionOperatorReport {
    pub(crate) const fn apply_blocker(self) -> Option<&'static str> {
        if !self.verifier_required {
            return None;
        }
        self.blocker
    }
}

pub(crate) fn verify_action_operator(
    original: &str,
    replacement: &str,
    error_class: TypingErrorClass,
    origin: CandidateOrigin,
) -> CorrectionActionOperatorReport {
    let operator = operator_for_origin(error_class, origin);
    let proof = proof_for_origin(error_class, origin);
    let transition = verifier::prove_edit_transition(original, replacement, error_class, origin);

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
            crate::correction_source_contract::CandidateOrigin::Layout,
        );

        assert_eq!(report.operator, LanguageActionOperator::FlipLayout);
        assert_eq!(
            report.edit_operator,
            verifier::EditTransitionOperator::LayoutProjection
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
            crate::correction_source_contract::CandidateOrigin::L2Surface,
        );

        assert_eq!(report.operator, LanguageActionOperator::FixTypo);
        assert_eq!(
            report.edit_operator,
            verifier::EditTransitionOperator::Unknown
        );
        assert!(report.verifier_required);
        assert!(!report.verifier_passed);
        assert_eq!(report.apply_blocker(), Some("edit_transition_not_verified"));
    }
}
