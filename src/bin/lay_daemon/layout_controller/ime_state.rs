pub(super) fn has_text_authority(state: &str) -> bool {
    state == "active:composition" || state == "passive:committed-tail"
}
