#[derive(Debug)]
pub(crate) enum TextReplacementPipelineError {
    Preflight(String),
    Delete(std::io::Error),
    IndeterminateAfterDelete(String),
}

impl TextReplacementPipelineError {
    pub(crate) fn log(self, label: &str, delete_failure_label: &str) {
        let message = match self {
            Self::Preflight(error) => format!("⚠ {label} skipped before delete: {error}"),
            Self::Delete(error) => format!("⚠ {label} {delete_failure_label}: {error}"),
            Self::IndeterminateAfterDelete(error) => format!(
                "⚠ {label} indeterminate after destructive delete; visible text must be reconciled: {error}"
            ),
        };
        crate::log(&message);
    }
}
