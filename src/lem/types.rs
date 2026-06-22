#[derive(Clone, Debug)]
pub struct ScoredCandidate {
    pub text: String,
    pub total: f64,
    pub language: f64,
    pub noise: f64,
    pub edit: f64,
    pub intervention: f64,
}

impl ScoredCandidate {
    pub(crate) fn with_language_weight(mut self, weight: f64) -> Self {
        let weight = weight.clamp(0.0, 2.0);
        self.total += self.language * (weight - 1.0);
        self.language *= weight;
        self
    }
}
