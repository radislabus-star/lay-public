#[derive(Clone, Debug)]
pub struct ScoredCandidate {
    pub text: String,
    pub total: f64,
    pub language: f64,
    pub noise: f64,
    pub edit: f64,
    pub intervention: f64,
}
