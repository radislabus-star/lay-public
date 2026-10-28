# KDE IME integration

This directory owns KDE-specific integration. Select the desktop through the
existing `LayConfig::active_layout_backend`; never probe GNOME after a KDE error.

Preserve configured field behavior under the
[field integration contract](../../../../docs/architecture/field-integration-isolation-2026-10-28.md).
A Chrome task must declare its exact integration scope. A change in shared
observation/state/ownership code is shared work, even with a KDE-only caller;
do not describe it as isolated to this folder or as correction-only.

- `desktop.rs`: the existing KWin observations for terminal qualification and
  focused-window identity. The common desktop dispatcher only selects it.
- `layout_sync.rs` and `layout_sync/tests.rs`: the existing KDE layout operation
  and its tests. Their Rust module names stay unchanged through `#[path]` at
  their existing callers; this is a file relocation, not a second controller.
- `display/common.rs`: KDE's auxiliary presentation policy and black-on-white
  hint style, independent of GNOME. The common display dispatcher selects it;
  `text` alone constructs scalar ranges and serializes IBus attributes.
- `native_document.rs` and `native_document/`: the bounded actual-field
  observation and native range adapter under review for Chrome's forced
  composition underline. Its Rust namespace remains
  `window_interaction::native_document`; the common reducer and executor retain
  authority. Observation, refresh and binding revalidation require the existing
  canonical KDE backend. No GNOME transport is entered through this adapter.

GNOME integration lives in `../gnome/`, selected by the same dispatcher. Correction,
context/word ownership, edit verification, learning and execution remain shared.
No client-name detection or separate mutation authority belongs in this folder.

The next functional task is Chrome only. Put a demonstrated client presentation
adaptation in `display/chrome.rs` when its requirements are established. Reuse
the existing working mechanisms first. Firefox and other clients follow later,
one at a time; do not add unused placeholder adapters or copy the common rules.

This relocation does not fix or certify the Chrome underline bug. Compilation,
source checks, installed bytes and physical client acceptance are separate facts.

`native_document/preparation.rs` owns all three native boundary/separator
preparation entrypoints. The common observation module retains the sole word
witness and its inert projection/constructors; KDE never copies that owner.
