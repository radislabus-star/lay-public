use lay::desktop::LayoutBackend;

use super::super::{layout_kde, layout_niri};

pub(super) fn detect_auto_layout_backend_hint() -> Option<LayoutBackend> {
    layout_kde::detect_auto_backend_hint().or_else(|| {
        if std::env::var_os("NIRI_SOCKET").is_some() && layout_niri::ping().is_ok() {
            Some(LayoutBackend::Niri)
        } else {
            None
        }
    })
}
