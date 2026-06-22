#[derive(Debug, Clone, PartialEq)]
pub struct WaveOptions {
    disabled: Vec<String>,
    llmwave_shadow: bool,
    llmwave_apply: bool,
    l2_weight: f32,
    l3_weight: f32,
}

impl Default for WaveOptions {
    fn default() -> Self {
        Self {
            disabled: Vec::new(),
            llmwave_shadow: false,
            llmwave_apply: false,
            l2_weight: 1.0,
            l3_weight: 1.0,
        }
    }
}

impl WaveOptions {
    pub fn with_disabled(disabled: &[String]) -> Self {
        Self {
            disabled: disabled.to_vec(),
            llmwave_shadow: false,
            llmwave_apply: false,
            l2_weight: 1.0,
            l3_weight: 1.0,
        }
    }

    pub fn with_llmwave_shadow(mut self, enabled: bool) -> Self {
        self.llmwave_shadow = enabled;
        self
    }

    pub fn with_llmwave_apply(mut self, enabled: bool) -> Self {
        self.llmwave_apply = enabled;
        self.llmwave_shadow |= enabled;
        self
    }

    pub fn with_layer_weights(mut self, l2_weight: f32, l3_weight: f32) -> Self {
        self.l2_weight = l2_weight.clamp(0.0, 2.0);
        self.l3_weight = l3_weight.clamp(0.0, 2.0);
        self
    }

    pub fn is_enabled(&self, cell: &str) -> bool {
        !self.disabled.iter().any(|item| item == cell)
    }

    pub fn disabled(&self) -> &[String] {
        &self.disabled
    }

    pub fn llmwave_shadow(&self) -> bool {
        self.llmwave_shadow
    }

    pub fn llmwave_apply(&self) -> bool {
        self.llmwave_apply
    }

    pub fn l2_weight(&self) -> f32 {
        self.l2_weight
    }

    pub fn l3_weight(&self) -> f32 {
        self.l3_weight
    }

    pub fn scale_l2_energy(&self, energy: f32) -> f32 {
        (energy * self.l2_weight).clamp(0.0, 1.0)
    }

    pub fn scale_l3_delta(&self, delta: f32) -> f32 {
        (delta * self.l3_weight).clamp(-1.0, 1.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_weights_keep_existing_strength() {
        let options = WaveOptions::default();
        assert_eq!(options.l2_weight(), 1.0);
        assert_eq!(options.l3_weight(), 1.0);
        assert_eq!(options.scale_l2_energy(0.5), 0.5);
        assert_eq!(options.scale_l3_delta(0.08), 0.08);
    }

    #[test]
    fn layer_weights_are_clamped() {
        let options = WaveOptions::default().with_layer_weights(3.0, -1.0);
        assert_eq!(options.l2_weight(), 2.0);
        assert_eq!(options.l3_weight(), 0.0);
        assert_eq!(options.scale_l2_energy(0.75), 1.0);
        assert_eq!(options.scale_l3_delta(0.08), 0.0);
    }
}
