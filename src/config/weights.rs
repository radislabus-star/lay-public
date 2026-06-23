use super::LayConfig;
use crate::text_backend::TextBackendPreference;

impl LayConfig {
    pub fn active_lem_weight(&self) -> f64 {
        if !self.lem_enabled {
            return 0.0;
        }
        f64::from(self.lem_weight_percent.clamp(0, 200)) / 80.0
    }

    pub fn active_nanda_l2_weight(&self) -> f32 {
        f32::from(self.nanda_l2_weight_percent.clamp(0, 200)) / 20.0
    }

    pub fn active_nanda_l3_weight(&self) -> f32 {
        f32::from(self.nanda_l3_weight_percent.clamp(0, 200)) / 8.0
    }

    pub fn active_nanda_precognition(&self) -> bool {
        self.nanda_precognition
            && self.active_text_backend() == TextBackendPreference::Ime
            && (self.active_nanda_l2_weight() > 0.0 || self.active_nanda_l3_weight() > 0.0)
    }
}
