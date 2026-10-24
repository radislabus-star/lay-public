# GNOME IME integration

- `display.rs` owns the user-accepted inline policy and GNOME hint style.
- `desktop.rs` is the existing GNOME Shell focused-window observer, relocated
  without a new namespace or observation authority.
- `layout_sync.rs` is the existing ActivateLayout transport. The common layout
  worker still owns ordering, stale-request checks and decoder promotion.

The resolved `LayConfig::active_layout_backend` selects this integration.
The common display module selects a profile; `text` constructs Unicode scalar
ranges and serializes IBus payloads. KDE changes belong in `../kde/`.
Correction, word/context ownership, learning, verification and delivery stay
shared. No KDE failure may retry through GNOME.

GNOME inline appearance was accepted on engine `fa5e8950...` on 2026-10-23.
The known Firefox saved-composition-on-blur limitation is separate; this file
relocation does not fix it or certify a native client.
