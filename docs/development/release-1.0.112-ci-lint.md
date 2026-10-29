# Release 1.0.112: explicit hint-frame lint contract

## Before-edit boundary

Baseline: `59dd712a427c30510415d97a5f72fbabb47e930f`.
[CI run 40970843062](https://github.com/radislabus-star/lay-public/actions/runs/40970843062)
passed all correctness/package targets after the ledger-binding repair, then
failed the default Clippy route at `hint_view.rs:85`: eight arguments exceed
the seven-argument recommendation. The 117-item default dead-code inventory
passed. This is a lint-only task, not a field integration or rendering change.

Keep the accepted `HintViewFrame::replace_with_style` signature, body, caller
order, ownership checks, revision update and optional style serialization.
Owner, text, coordinates and presentation belong to one checked frame update;
splitting it solely to satisfy an argument count is unnecessary scope.
Use the existing item-local `#[expect(clippy::too_many_arguments, reason = ...)]`
convention from `context_admission.rs` and `composition_commit.rs`.
Unfulfilled expectations must remain errors; no crate-wide allowance, Clippy
threshold, CI gate or validator change is permitted.

Allowed source changes: that single attribute on that single method, and the
equivalent character-pattern expression described below. Derived
AST graph/binding/receipt metadata and the owning documents may be refreshed.
Do not change test identities, the manifest, dead-code inventories, GNOME/KDE
behavior, any delivery route, installed bytes, desktop configuration or services.

## Verification

The real CI diagnostic is the red evidence. Verify that removing only the
new attribute yields the exact baseline file. Run the existing default and
research-tools lint contract on the dedicated guarded remote worker, refresh
the graph metadata, and verify its receipt. Do not rebuild/install the accepted
runtime or rerun the native browser matrix for a lint attribute. Push the
repair and read back automatic CI, including release binaries.

## Result

The first guarded follow-up passed formatting and the 117-item default
dead-code inventory. The argument-count error was absent; default Clippy then
reported `manual_pattern_char_comparison` at
`src/bin/lay_daemon/kde/firefox_replay_receipt.rs:133`. Saved failure:
CROOT/release112-hint-frame-lint/result.json and lint.log (52.240s lint route).

Before the second edit, extend the exact scope to that existing KDE/Firefox
receipt leaf. Replace only its trailing ASCII-Space/NBSP predicate with
`trim_end_matches([' ', '\u{a0}'])`. Preserve its body/count checks and all other
code. Existing native receipt tests cover closing-space identity/count and
prefix preservation; execute those identities before the complete lint route.
Do not suppress this lint or change the receipt admission rule.

Guarded second follow-up: five exact existing receipt tests PASS (18.877s);
complete default/research-tools lint contract PASS (54.597s); formatting PASS;
AST graph update, pruning, source binding and architecture receipt PASS.
Receipt: CROOT/release112-ci-lint-v2/result.json, worker SHA
1b56c97d9b47fc3c74d60c62e2b0058e3aebf41e7c437074aa64a76c9e8aaaf9.
Lint/test source closure44e71598; derived graph metadata closure2067c53b.
Both closures contain1318 files. Running daemon, IME, tray, IBus and original
Firefox process identities remained unchanged. No release build or installation
was performed by this worker. Automatic CI after push must verify the final
post-graph snapshot and release build. Keep v1.0.112 immutable.
