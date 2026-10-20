# Reserved IBus daemon sender compatibility

Upstream: zbus_names 4.3.2, MIT (see LICENSE), from
https://static.crates.io/crates/zbus_names/zbus_names-4.3.2.crate

Original crate archive SHA-256:
`7074f3e50b894eac91750142016d30d0a89be8e67dbfd9704fb875825760e52d`.
Registry metadata, upstream lockfile and original pre-normalization manifest
are omitted; production source, normalized Cargo manifest and license are kept.

The only production source change is the opt-in `ibus-bus-daemon` feature:
UniqueName validation additionally admits exactly `org.freedesktop.IBus`.
IBus 1.5.26 emits GlobalEngineChanged with that Sender. Without this patch,
zbus's Value conversion accepts the name but cached header reconstruction
panics when validating it as a string. Lay release builds abort on that panic.
The feature preserves the wire identity; it neither maps it to DBus's name nor
admits other well-known clients. Lay's existing authenticated connection,
bootstrap-bound sender roles, path/member and lifecycle checks remain required.

Regression: `tests/ibus_wire_compat.rs` reconstructs the stock equal-length
wire header, checks its payload and rejects near-match service names. It was
RED (the same reconstruction panic) on the unpatched dependency.

This fork is temporary. On a future zbus update, replace it with upstream
support only after this wire regression, role/admission checks and installed
native IBus acceptance pass. Do not remove the patch merely for a version bump.
