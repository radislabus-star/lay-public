#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct WaveOptions {
    disabled: Vec<String>,
}

impl WaveOptions {
    pub fn with_disabled(disabled: &[String]) -> Self {
        Self {
            disabled: disabled.to_vec(),
        }
    }

    pub fn is_enabled(&self, cell: &str) -> bool {
        !self.disabled.iter().any(|item| item == cell)
    }

    pub fn disabled(&self) -> &[String] {
        &self.disabled
    }
}
