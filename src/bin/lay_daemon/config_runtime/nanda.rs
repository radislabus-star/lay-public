pub(crate) fn active_nanda_wave_options() -> lay::nanda_wave::WaveOptions {
    let cfg = super::current_config();
    lay::nanda_wave::WaveOptions::default()
        .with_layer_weights(cfg.active_nanda_l2_weight(), cfg.active_nanda_l3_weight())
        .with_llmwave_shadow(cfg.llmwave_shadow)
        .with_llmwave_apply(cfg.llmwave_shadow && cfg.llmwave_apply)
}
