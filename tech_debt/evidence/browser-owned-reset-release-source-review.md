# Browser owned Reset and release source review, 1.0.74

Source review: **8/10, H0/M0/L0**. This reviews the bounded managed-word
snapshot and Reset/release change after the accepted delayed-snapshot
checkpoint. It does not replace that historical review. The full fixed gate,
final release bytes and same-binary Firefox/Chrome acceptance are separate.

The first Firefox physical result retained the suggestion but did not correct
the word. An exact six-character snapshot still matched the witness after
`Reset`; the next key's release was unhandled because reset cleanup had cleared
the keycode of its already handled press. That release revoked the witness and
prepared Space path. This is evidenced by
`/home/ubu/.cache/lay/development/browser-autocorrect-20260922/physical-firefox-reconcile-diag/receipt.json`
(SHA-256 `fa54f2d51ada151403ded3d7ee1bee34ffec8bff6f02e1200ae45644f15058fc`).

The new snapshot arm requires managed input, exact surrounding refresh,
non-sensitive content, no atomic or active composition, no selection, an exact
retained tail before the caret and word boundaries on both sides. Its start
identity records focus, owner lease, layout, observation revision, tail epoch,
token length and snapshot geometry. Later projection requires an uninterrupted
one-epoch-per-character managed CommitText chain. A foreign key or changed
focus/owner/layout/text cannot reuse the witness.

Only a current witness can arm the Reset echo ticket after CommitText. The
ticket is one-use, tied to that tail epoch and expires after 700 ms. An owned
Reset can retain the current Space frame and handled-press keycodes, so the
release of the same handled key stays internal. Other Resets clear those
keycodes and advance the tail authority boundary. Client-owned key exits still
revoke the witness and Space path; SafetyGate, candidate ranking, edit-plan
validation and visible postcondition verifier are unchanged.

The focused callback test now uses `CommitText -> Reset -> release` order and
asserts an exact correction. It also rejects a duplicate or late Reset. It
passed 1/1 in
`/home/ubu/.cache/lay/development/browser-autocorrect-20260922/reset-release-focused-final.log`
(SHA-256 `c0a52ab44597169721e670b2a7f9b0164dea87f138a21572cca9418a17e9820c`).
The diagnostic Firefox run kept the suggestion continuous and changed
`публекует ` to `публикует `, with zero composition events:
`/home/ubu/.cache/lay/development/browser-autocorrect-20260922/physical-firefox-reset-release/receipt.json`
(SHA-256 `c0cb13a12e698c278426d3ea5a7ad53093c2ca156baabbb108e1623c10b0101b`).

Reviewed source identities (SHA-256):

| Path | SHA-256 |
| --- | --- |
| `src/bin/lay_ibus_engine/composition_commit.rs` | `c7e8f604ced77488302e510bfb4c3df87ed05ac60efef67e78b568a4fe769358` |
| `src/bin/lay_ibus_engine/window_interaction/observation.rs` | `21af33a344e75b53f1bacfde6c6fe8e77efa485714671466b7e31d97d3d015bd` |
| `src/bin/lay_ibus_engine/state.rs` | `205e096f699845539812d2f93ec99e964ccc33c4bc04b266336430a62bfe72a3` |
| `src/bin/lay_ibus_engine/managed.rs` | `70113d94e9e513a71d74eb19518e0b38d4edfbb2ef38c4f555330400d3466ee1` |
| `src/bin/lay_ibus_engine/ibus_interface.rs` | `14e1749758aee05fe70079405e21503cc8e9a5f870dc705feb0f8419358da107` |
| `src/bin/lay_ibus_engine/context_admission/adapter/tests/terminal_delivery.rs` | `f5c2b92104f963ba110df4cc81e56bf6031dce4fb3b99f69da8c5dcf07c1f3bc` |

Verdict scope: source and focused causal route **PASS**. Runtime edit
authority changed only in the exact midword snapshot and owned Reset/release
path. Quality in the full test set and physical Chrome is still **UNKNOWN** at
this review checkpoint.
