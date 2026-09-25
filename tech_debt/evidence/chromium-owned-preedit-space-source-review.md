# Chromium owned-preedit Space authority source review, 1.0.74

Source checkpoint review: **9/10, H0/M0/L1**. This accepts the bounded source
successor only. The changed-source gate has passed; optimized build,
installation and ordinary-Chromium physical acceptance remain separate gates.

The reviewed change gives Space autocorrection a dedicated input-frame token
for a complete `UnknownStart` word only while that word is still owned either
as an exact native-terminal suffix or as an active legacy preedit equal to the
current tail token. Capture and consumption retain the existing path, focus,
owner, epoch, layout, configuration, output-capability, lexical-coordinate and
material-generation checks. The display-only suffix token remains ineligible
for mutation.

Native-terminal and owned legacy-preedit scheduling now occur in the successful
post-settlement callback. This is required because settlement advances the
observed-suffix lineage used by the `UnknownStart` Space token. The first source
checkpoint left legacy-preedit scheduling inside the printable handler; an
ordinary-Chromium run then proved that its fully prepared lease was stale at
Space. The repaired route keeps exact recapture and every existing authority
check. Space still consumes the common prepared lease, `SafetyGate`, authorized
edit plan and existing backend executor; no candidate, ranking, verifier or
learning rule changed.

No high or medium finding remains. The gate exposed one fixture defect: two
full-correction tests depended on the workstation's installed lexical package.
Their route proof now uses the built-in deterministic `рабоает -> работает`
case and passes with home/config/cache masked. The user's concrete
`публекует -> публикует` case is reserved for the separately required physical
Chromium acceptance with the installed package.

The remaining low finding at source review was that the first installed
Chromium experiment failed before this scheduling repair, while source tests
could not prove repaired client timing or the installed package answer. Its
bounded trace recorded worker generation 8 as fully prepared, then
`stale_lease` and `space_legacy_preedit_commit`; it also preserved physical
Double Shift. The stable runtime was restored after that failed experiment.
The later promotion run resolved the bounded live gate: ordinary Chrome
produced exact `публикует публикует ` and physical Double Shift produced exact
`привет`; receipt SHA-256 is
`0f4275f1a15e1ad081a6aee0c25c6b70afc55bb05a2d32e38a4b3998dceb04cb`.
Sub-80 ms Space timing remains unmeasured. The fixed L1 quality proof was not
rerun because candidate production, scoring and verifier authority are
unchanged; answer quality remains `UNKNOWN` until independently measured.

Focused evidence before this review passed the native route 2/2, Chromium
owned-preedit route 1/1, TD-125 legacy preedit 17/17, context runtime 17/17,
preedit 89/89, committed tail 13/13, Space prefetch 11/11 and active route 3/3.
After the fixture repair, both full-correction routes pass in separate hermetic
processes with network disabled and home/config/cache masked. Process isolation
removed shared worker state, but repeated runs showed that semantic callback
fixtures still raced the real 1 ms stamp rendezvous and failed different cases.
One repeated clean case measured 19 PASS and 1 FAIL. The common semantic helper
now observes the already received ingress before invoking the engine callback;
dedicated adapter tests retain pending, timeout, marker and revocation
schedules. Callback fixtures that explicitly model an immediate Reset bind
their recency timestamp at that callback, and synthetic callbacks suppress
unrelated transport replies. No product deadline changed.

The repaired Chromium fixture additionally checks the identity actually
scheduled by the background worker against the post-settlement frame and fails
on `Stale` before installing its deterministic full-correction lease. That
causal fixture passed 1/1. After the fixture-ordering repair, the complete
focused engine target passed 586/586; its summary SHA-256 is
`ec0f2ea5470913d8f1c2f3152c1168fbf767b732cc146f2fb3f68298835b94d6`.
The complete changed-source gate then passed all 2,887 selected correctness and
package cases with zero semantic or infrastructure failures, plus `cargo
check`, transition replay and the unsafe-edit scoreboard at zero gate failures.
Its log SHA-256 is
`9c403e6323d324779bbbe156f46645cebf16ca7ca131deaa2636f242c853cba5`.

Reviewed source SHA-256:

| Path | SHA-256 |
| --- | --- |
| `src/bin/lay_ibus_engine/committed_tail.rs` | `c84297534abeb0981a8f707b0e5adaf2347ee67da0be347276d08fd16930b7f0` |
| `src/bin/lay_ibus_engine/composition_commit.rs` | `f615fc1af80c9a8c64b64dfb435ce55776d80321e3e650435a84aa2a1fc04659` |
| `src/bin/lay_ibus_engine/engine/types.rs` | `338d7a8f6e7d8603f972efe83b5ff0665638137c69b0d17b7dc1e1349bd78d34` |
| `src/bin/lay_ibus_engine/managed.rs` | `cb28450cd58bc5ee9178a886ae027e4d89dea8076a26def9532629932d37a879` |
| `src/bin/lay_ibus_engine/preedit.rs` | `b84206cd53e0b2ba83346aa8d0499c47f2e038cc470b5027b9c2387fc1715f58` |
| `src/bin/lay_ibus_engine/window_interaction/observation.rs` | `8d172645c86ab7146ac928baee2e2c2355b7ec9a5b58f4663487cfd8b37fb11b` |
| `src/bin/lay_ibus_engine/context_admission/adapter/tests/residuals.rs` | `e48e22ca596eb9662dfc6e6f9c2623e4d59897048365b0958aea54e59d808161` |
| `src/bin/lay_ibus_engine/context_admission/adapter/tests/terminal_delivery.rs` | `e8714542eacfe4cc0eba81c20fcfd364e5efcb25ad98dbab69da1576a68429a0` |
| `src/bin/lay_ibus_engine/context_admission/adapter/tests/word_scope.rs` | `4b94f9efa4ad4ab95e422e7407aa5876dc259388ee6593f54814ae0089de2953` |
| `src/bin/lay_ibus_engine/space_autocorrect_prefetch/proof.rs` | `5845b694c18f1ed47cc05daa1eab09436c3bf6c9d76ed379f4751940d0d4554c` |

Runtime authority changed by this source-review checkpoint: **false**.
