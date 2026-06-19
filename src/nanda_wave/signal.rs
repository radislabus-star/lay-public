use super::mode::ModeRole;

#[derive(Debug, Clone, PartialEq)]
pub struct ActiveMode {
    pub cell: &'static str,
    pub mode_id: usize,
    pub role: ModeRole,
    pub energy: f32,
    pub phase: i8,
    pub coherence: f32,
}

impl ActiveMode {
    pub fn label(&self) -> String {
        format!(
            "{}#{}:{}:{:.3}",
            self.cell,
            self.mode_id,
            self.role.as_str(),
            self.energy
        )
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WavePacket {
    pub layer: &'static str,
    pub cell: &'static str,
    pub modes: Vec<ActiveMode>,
}

impl WavePacket {
    pub fn top_energy(&self) -> f32 {
        self.modes
            .iter()
            .map(|mode| mode.energy)
            .fold(0.0, f32::max)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WordCandidate {
    pub text: String,
    pub source: &'static str,
    pub energy: f32,
    pub risk: f32,
    pub support: Vec<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum WaveDecision {
    Apply { text: String, confidence: f32 },
    Keep { reason: &'static str },
    Veto { reason: &'static str },
}

#[derive(Debug, Clone, PartialEq)]
pub struct LayerTrace {
    pub name: &'static str,
    pub summary: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct WaveTrace {
    pub original: String,
    pub l1: Vec<WavePacket>,
    pub l2_candidates: Vec<WordCandidate>,
    pub l3: Vec<LayerTrace>,
    pub decision: WaveDecision,
}

impl WaveTrace {
    pub fn output(&self) -> Option<&str> {
        match &self.decision {
            WaveDecision::Apply { text, .. } => Some(text.as_str()),
            WaveDecision::Keep { .. } | WaveDecision::Veto { .. } => None,
        }
    }
}
