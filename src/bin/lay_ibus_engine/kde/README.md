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
Client names never grant text-edit authority. Exact OS process qualification
belongs to its client adapter and still requires the common admission/verifier.

`chrome.rs` owns the existing exact process qualification and100ms activation
budget. It does not render text. `display/` owns KDE presentation only; client
qualification is not hidden in it. Native observation and range execution keep
using the same common contracts. Add another client adapter only for a proven
difference; do not create placeholders or copy transports.

This relocation does not fix or certify the Chrome underline bug. Compilation,
source checks, installed bytes and physical client acceptance are separate facts.

`native_document/preparation.rs` owns all three native boundary/separator
preparation entrypoints. The common observation module retains the sole word
witness and its inert projection/constructors; KDE never copies that owner.

Read-only append resampling validates the current admitted key token against
the existing native field/focus/lease/layout transport and the owned append
witness. Its two attempts share the original80ms deadline. Edit preparation
retains the original native observation token and requires an actual complete
snapshot; a read request never rebinds that edit receipt or installs projection.

`terminal.rs` owns the existing Kitty/Konsole qualification and probe. The
`desktop.rs` facade preserves the selected-desktop API and test identities;
the common terminal receipt and executor remain unchanged. No terminal name
adds authority, and an unavailable KDE probe never invokes GNOME.

`chrome/presentation.rs` owns the existing Chrome-only native separator and
presentation acknowledgements. Its original private Rust parent and facade
remain `window_interaction::native_document`; the file is compiled once.
A Chrome presentation edit no longer shares its source file with observation
transport. Common formatting and GNOME presentation remain separate.

GTK/VTE GNOME Terminal on KDE qualifies through `terminal.rs` with the exact
observed canonical window class and coherent desktop identity. This selects
the existing current-field terminal input contract; it does not provide a
source snapshot or edit authority. GNOME Desktop dispatch is unchanged.
