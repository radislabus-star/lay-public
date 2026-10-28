# KDE daemon policy

`focus_policy.rs` owns KDE foreground-policy conversion and its single
unavailable-state flag. The existing `focus_guard` facade selects it only for
the resolved KDE backend; GNOME logic stays outside here. Rust helper names
for current callers/tests are preserved through re-exports.

The authenticated transport remains `../kde_focus.rs`, shared by the existing
daemon/IME integration. This directory does not copy it, create a second
provider/owner, authorize an edit, or change keyboard capture/output.
