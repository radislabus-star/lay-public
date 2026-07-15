use super::edit_plan::DecoderEditPlan;

impl DecoderEditPlan {
    pub fn matches_text_edit_contract_boundary_shift(&self) -> bool {
        self.transition.operator() == Some(crate::text_edit::TransitionOperator::BoundaryShift)
            && self.transition.proof() == Some(crate::text_edit::TransitionProof::Boundary)
            && self.transition.verified() == Some(true)
            && self.transition.changed_tokens() == Some(2)
    }
}
