use super::super::{log, read_current_layout_is_ru, switch_to_target_layout};

#[derive(Debug, Clone, Copy)]
pub(crate) struct LayoutCapabilityPreflight {
    initial_layout_is_ru: Option<bool>,
    current_layout_is_ru: Option<bool>,
}

impl LayoutCapabilityPreflight {
    pub(crate) fn run(
        known_initial_layout_is_ru: Option<bool>,
        target_layouts: impl IntoIterator<Item = bool>,
        label: &str,
    ) -> Result<Self, String> {
        let initial_layout_is_ru =
            known_initial_layout_is_ru.or_else(|| read_current_layout_is_ru().ok());
        let mut preflight = Self {
            initial_layout_is_ru,
            current_layout_is_ru: initial_layout_is_ru,
        };
        for target_is_ru in target_layouts {
            if preflight.current_layout_is_ru == Some(target_is_ru) {
                continue;
            }
            if let Err(error) = switch_to_target_layout(target_is_ru) {
                preflight.current_layout_is_ru = None;
                preflight.restore_initial_best_effort(label);
                return Err(format!(
                    "layout capability preflight failed before destructive edit: {error}"
                ));
            }
            preflight.current_layout_is_ru = Some(target_is_ru);
        }
        Ok(preflight)
    }

    pub(crate) fn current_layout_is_ru(self) -> Option<bool> {
        self.current_layout_is_ru
    }

    pub(crate) fn restore_initial_best_effort(&self, label: &str) {
        let Some(initial_layout_is_ru) = self.initial_layout_is_ru else {
            return;
        };
        if self.current_layout_is_ru == Some(initial_layout_is_ru) {
            return;
        }
        match switch_to_target_layout(initial_layout_is_ru) {
            Ok(layout_id) => log(&format!(
                "  {label} layout restored after preflight abort -> {layout_id}"
            )),
            Err(error) => log(&format!(
                "warning: {label} layout restore after preflight abort failed: {error}"
            )),
        }
    }
}
