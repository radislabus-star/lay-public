# Browser delayed-snapshot Space authority source review, 1.0.74

Final source review: **9/10, H0/M0/L0**. This accepts the bounded source
successor only. The complete project gate, release build, installation and
physical Firefox/Chrome acceptance remain separate gates.

The reviewed successor retains one exact managed caret-boundary snapshot while
the engine appends an uninterrupted local CommitText word. The witness binds
focus receipt and serial, runtime owner lease, layout generation, starting tail
epoch and text, exact snapshot geometry and observation revision. It permits a
single source-free rebind before the first character. Later use requires the
same owner/focus/layout lineage and exactly one tail-epoch advance per local
character. The projected snapshot is request-scoped and is never installed as
a client observation. Candidate selection, ranking, DecisionCore, SafetyGate,
edit-plan validation and visible postcondition verification are unchanged.

The first review found one high issue: client-owned command, navigation, Tab
and generic non-printable exits inside `process_pressed_key` retained both the
witness and prepared Space work. The second review confirmed those paths were
fixed, then found one medium issue at wrapper exits that run before managed
dispatch: disabled live composition and unsuccessful standalone
Alt/ISO-level3 completion release. Both checkpoints were rejected.

The final source routes every client-owned mutation/navigation exit through one
revocation helper which clears the managed-word-start witness and invalidates
the whole Space-prefetch path. This includes unmanaged/backend-disabled input,
native exact replay, unhandled Alt/ISO branches, generic releases, failed
Up/Down and Tab, command input, cursor/Backspace/Enter, generic non-printables
and unhandled terminal/Space paths. Handled releases remain internal. Shift-only
legacy and atomic branches remain observation-only so shifted managed typing is
not broken.

The causal negatives begin with a fully prepared old lease. Five managed-route
classes plus backend disable/re-enable and standalone Alt release must leave no
witness, no capturable frame and no ready lease. The following Space must emit
zero DeleteSurroundingText effects and only the ordinary space commit. The
focused remote gate passed 590/590. Result:
`/home/ubu/.cache/lay/development/run-ekb6jas1/RESULT.json`, SHA-256
`ad9442d37e357d27f1c6a9aede2cd0ae6695961dc0efe16c91277da2627dcfa9`;
run log SHA-256
`b9f264e409f61b8779bfe351db34d64d3927cef228b52cf05107d04bc1c4b972`;
source archive SHA-256
`9c22a693763a8f9130b1325eaa7b4f6e77df54055055882b23b2b81695b3e2d1`.

Reviewed source SHA-256 identities:

| Path | SHA-256 |
| --- | --- |
| `src/bin/lay_ibus_engine/composition_commit.rs` | `2f6d78072dc9c4087ad5cea738e6e35d20de93425f8867c94ab46256a62270ed` |
| `src/bin/lay_ibus_engine/ibus_interface.rs` | `14e1749758aee05fe70079405e21503cc8e9a5f870dc705feb0f8419358da107` |
| `src/bin/lay_ibus_engine/managed.rs` | `70113d94e9e513a71d74eb19518e0b38d4edfbb2ef38c4f555330400d3466ee1` |
| `src/bin/lay_ibus_engine/context_admission/adapter/tests/terminal_delivery.rs` | `d6638fd8e369cba54089dff986cd711b60fdbef5cfca64a35feac3251876745e` |

No high, medium or low source finding remains. Runtime-authority classification
for the full successor is **changed**: the exact managed word-start plus its
verified local chain can now authorize the existing bounded correction where a
final client snapshot is delayed. The receipt's generic plan field does not
encode this architecture-specific classification.
