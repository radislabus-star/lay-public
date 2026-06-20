#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct WaveOptions {
    disabled: Vec<String>,
    llmwave_shadow: bool,
    llmwave_apply: bool,
}

impl WaveOptions {
    pub fn with_disabled(disabled: &[String]) -> Self {
        Self {
            disabled: disabled.to_vec(),
            llmwave_shadow: false,
            llmwave_apply: false,
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
}
