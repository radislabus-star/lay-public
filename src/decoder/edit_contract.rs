use super::edit_plan::DecoderEditPlan;

impl DecoderEditPlan {
    pub fn matches_text_edit_contract_boundary_shift(&self) -> bool {
        self.transition.operator.as_deref() == Some("boundary_shift")
            && self.transition.proof.as_deref() == Some("boundary")
            && self.transition.verified == Some(true)
            && self.transition.changed_tokens == Some(2)
    }
}
