//! A small synthetic membership fixture for semantic/execution contract tests.
//! No L1 links, ranking weights or context edges; this is not corpus quality.
pub fn warm_lexical_fixture() {
    static WARM: std::sync::Once = std::sync::Once::new();
    WARM.call_once(|| {
        if lay::nanda_wave::discover_installed_l2_package()
            .expect("discover lexical fixture")
            .is_none()
        {
            let path = std::env::temp_dir().join(format!(
                "lay-space-boundary-lexical-{}.bin",
                std::process::id(),
            ));
            std::fs::write(
                &path,
                include_bytes!("../fixtures/space_boundary_lexical_v2.bin"),
            )
            .expect("write lexical fixture");
            // The lane starts one test identity per isolated process. Setup
            // precedes correction workers and never changes a warmed provider.
            std::env::set_var("LAY_L2_PACKAGE", &path);
        }
        let status = lay::nanda_wave::canonical_l2_status();
        assert_eq!(status["status"], "ready", "{status}");
    });
}
