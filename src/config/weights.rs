use super::LayConfig;
impl LayConfig {
    pub fn active_nanda_l2_weight(&self) -> f32 {
        f32::from(self.nanda_l2_weight_percent.clamp(0, 200)) / 20.0
    }

    pub fn active_nanda_l3_weight(&self) -> f32 {
        f32::from(self.nanda_l3_weight_percent.clamp(0, 200)) / 8.0
    }

    pub fn active_nanda_precognition(&self) -> bool {
        self.nanda_precognition
            && self.active_text_backend().should_try_ime()
            && (self.active_nanda_l2_weight() > 0.0 || self.active_nanda_l3_weight() > 0.0)
    }

    pub fn active_nanda_wave_options(&self) -> crate::nanda_wave::WaveOptions {
        crate::nanda_wave::WaveOptions::default()
            .with_layer_weights(self.active_nanda_l2_weight(), self.active_nanda_l3_weight())
            .with_llmwave_shadow(self.llmwave_shadow)
            .with_llmwave_apply(self.llmwave_shadow && self.llmwave_apply)
            .with_l2_phase_shadow(self.nanda_l2_phase_shadow)
            .with_l2_phase_apply(self.nanda_l2_phase_shadow && self.nanda_l2_phase_apply)
            .with_l3_phase_shadow(self.nanda_l3_phase_shadow)
    }
}
