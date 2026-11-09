//! Mode-only completion for the marked, already admitted exact-tail gesture.
//! This entry supplies no text proof, edit, new owner or handoff extension.
use crate::{bridge::LayImeBridge, engine::LayIbusEngine, output::EngineOutput};
use zbus::fdo;

impl LayImeBridge {
    pub(crate) async fn complete_fragment_word_mode_inner(
        &self,
        expected_suffix: String,
        expected_epoch: u64,
        expected_path: String,
        expected_is_ru: bool,
        target_is_ru: bool,
    ) -> fdo::Result<bool> {
        let token = self.bridge_admission_token().await?;
        let Some(path) = self.bridge_engine_path(token.as_ref()) else {
            return Ok(false);
        };
        if path != expected_path || expected_path.is_empty() {
            return Ok(false);
        }
        let iface = self
            .ibus_connection
            .object_server()
            .interface::<_, LayIbusEngine>(path.as_str())
            .await
            .map_err(|error| fdo::Error::Failed(error.to_string()))?;
        let mut engine = iface.get_mut().await;
        if !self.shared_active_path_matches(&path)
            || !self.bridge_token_is_live(&engine, token.as_ref())
            || !engine.fragment_manual_mode_handoff_matches(
                &expected_suffix,
                expected_epoch,
                &expected_path,
                expected_is_ru,
                target_is_ru,
            )
        {
            return Ok(false);
        }
        let mut output = EngineOutput::legacy(iface.signal_emitter());
        engine
            .complete_fragment_word_mode(&mut output, target_is_ru)
            .await?;
        engine.layout_gesture.fragment_word_mode_generation =
            Some(engine.layout_gesture.layout_generation);
        Ok(self.bridge_token_is_live(&engine, token.as_ref())
            && engine.committed_tail.epoch == expected_epoch
            && engine.path == expected_path
            && engine.layout_gesture.layout_is_ru == target_is_ru)
    }
}
