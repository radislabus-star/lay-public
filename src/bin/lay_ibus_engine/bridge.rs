use zbus::fdo;
use zbus::{interface, Connection};

use super::protocol::Shared;
use crate::bridge_policy::should_suppress_next_autocorrect;

pub(crate) struct LayImeBridge {
    pub(crate) ibus_connection: Connection,
    pub(crate) shared: Shared,
}

#[interface(name = "io.github.radislabus_star.LayIme")]
impl LayImeBridge {
    #[zbus(name = "Ping")]
    fn ping(&self) -> String {
        let state = self.shared.lock().expect("lay ime state poisoned");
        match state.active_path.as_deref() {
            Some(path) => format!("lay-ibus-engine-rs focused {path}"),
            None => "lay-ibus-engine-rs no-focus".to_string(),
        }
    }

    #[zbus(name = "Focused")]
    fn focused(&self) -> bool {
        self.shared
            .lock()
            .expect("lay ime state poisoned")
            .active_path
            .is_some()
    }

    #[zbus(name = "OwnsActiveText")]
    async fn owns_active_text(&self) -> fdo::Result<bool> {
        self.owns_active_text_inner().await
    }

    #[zbus(name = "InputState")]
    async fn input_state(&self) -> fdo::Result<String> {
        self.input_state_inner().await
    }

    #[zbus(name = "VisibleTailV1")]
    async fn visible_tail_v1(&self) -> fdo::Result<(String, String, bool)> {
        self.visible_tail_v1_inner().await
    }

    #[zbus(name = "VisibleTailV2")]
    async fn visible_tail_v2(&self) -> fdo::Result<(String, String, bool, u64, String)> {
        self.visible_tail_v2_inner().await
    }

    #[zbus(name = "CanReplaceCommittedTail")]
    async fn can_replace_committed_tail(&self, backspaces: u32) -> fdo::Result<bool> {
        self.can_replace_committed_tail_inner(backspaces).await
    }

    #[zbus(name = "SuppressNextAutocorrect")]
    async fn suppress_next_autocorrect(&self) -> fdo::Result<bool> {
        self.suppress_next_autocorrect_inner().await
    }

    #[zbus(name = "ManualToggle")]
    async fn manual_toggle(&self) -> fdo::Result<bool> {
        self.manual_toggle_inner().await
    }

    #[zbus(name = "ManualToggleV2")]
    async fn manual_toggle_v2(&self) -> fdo::Result<(bool, bool)> {
        self.manual_toggle_v2_inner().await
    }

    #[zbus(name = "ReplaceTail")]
    async fn replace_tail(&self, backspaces: u32, text: String) -> fdo::Result<bool> {
        self.replace_tail_inner(backspaces, text, false, None, None)
            .await
    }

    #[zbus(name = "ReplaceTailV2")]
    async fn replace_tail_v2(
        &self,
        backspaces: u32,
        text: String,
        kind: String,
    ) -> fdo::Result<bool> {
        self.replace_tail_inner(
            backspaces,
            text,
            should_suppress_next_autocorrect(&kind),
            None,
            None,
        )
        .await
    }

    #[zbus(name = "ReplaceTailV3")]
    async fn replace_tail_v3(
        &self,
        backspaces: u32,
        text: String,
        kind: String,
        expected_original_tail: String,
    ) -> fdo::Result<bool> {
        self.replace_tail_inner(
            backspaces,
            text,
            should_suppress_next_autocorrect(&kind),
            Some(expected_original_tail),
            None,
        )
        .await
    }

    #[zbus(name = "ReplaceTailV4")]
    async fn replace_tail_v4(
        &self,
        backspaces: u32,
        text: String,
        kind: String,
        expected_original_tail: String,
        expected_epoch: u64,
        expected_focus: String,
    ) -> fdo::Result<bool> {
        self.replace_tail_inner(
            backspaces,
            text,
            should_suppress_next_autocorrect(&kind),
            Some(expected_original_tail),
            Some((expected_epoch, expected_focus)),
        )
        .await
    }
}
