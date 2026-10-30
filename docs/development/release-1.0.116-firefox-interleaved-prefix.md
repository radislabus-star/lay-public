# Lay 1.0.116 preflight — Firefox interleaved owned prefix receipts

Baseline21f82bb20f4ee932186520a55ecc14e614233963 (115 source/build verified,
committed and pushed; its independent publication checks remain separate).
One active implementation,SOLO/noHarness/noagents. No localGNOME installation.

Observed failure:114 private iframe-rich consecutive-two active row155 refuses
Double Shift context_authority after actual SSI prefixes4/5/6 and an owned key
between the first two. Native trace is complete,but lengths alone did not prove
the mechanism. Real production-adapter/selector RED now confirms it: exactly1
selected isolated test fails at final exact manual handoff. Receipt
CROOT/release116-prefix-red-v1/result.json. Source and runtime restored/unchanged.

Map (existing owners retained):
context_admission/adapter/tests/active_composition_space_receipt.rs
 -> window_interaction/observation.rs::advance_context_reset_rereceipt_after_key
 -> kde/firefox/preedit_echo.rs (pure whole-field prefix progression)
 -> observation.rs::observe_context_reset_rereceipt_surrounding_text
 -> existing authority.rs manual admission -> verifier/executor.

First failed transition: after a confirmed managed append receives a delayed
strict prefix,it becomes inert. The next exactly owned character clears the
one-shot prefix revision and drops owned_append_actual_before because confirmed
is false. A second delayed prefix cannot retain the original whole field;
the final complete snapshot at revisiongap2 is rejected. This is not an output
delivery failure. Do not weaken UnknownStart or recreate proof from equal text.

Selected design9/10: reuse the existing bounded whole-field anchor in the
existing pending receipt. Only a freshly identified KDEFirefox callback and
existing managed identity may retain it through a strict prefix progression.
Later exactly owned appends may carry that already inert anchor. Final actual
full field still uses the existing exact receipt,manual admission and verifier.
Pure semantic checks live in the existing KDEFirefox leaf; common code only
plumbs the same receipt fields/selector. No new owner,route,cache or fallback.

Rejected designs: an extra manual-gesture SSI retry5/10 adds latency and masks
the first proof loss; suffix equality1/10 cannot distinguish changed external
text. A new receipt owner/type is unnecessary for a single bounded predecessor.

Scope: runtime preedit_echo.rs and observation.rs, plus the existing adapter
test file above. Current scope guard has no Firefox component; declare these
exact files rather than allowing all KDE edits. Freeze neighboring integration
files. Version metadata,actual manifest/zero-failure observation and generated
graph are administrative release outputs. GNOME/Chrome/terminal policy,shared
decision,decoder,output,all deadlines and global IBus are frozen.

Consequence check: no candidate/lattice/ranking,learning,package/delta reload or
model behavior changes. No mutation right on partial snapshots. The anchor is
one existing bounded snapshot,not a growing history. Added Unicode whole-field
comparison/formatting is O(snapshot) on the candidate callback; no new I/O,
timeout or retry. Normal exact full snapshots use the old path. Configured
desktop alone is insufficient: actual Firefox dispatch and current owner must
both match. Every intervening owned key still proves exact character append,
same word generation/owner and next tail epoch. Contradictory observations
spend the anchor as before; focus,Reset,layout/lease changes cannot restore it.

Second-order risk: a foreign external prefix followed by consistent growth
must never regain authority. Require the original whole actual anchor,not only
the previous partial or a KnownStart token. The generic path spends the anchor;
only the scoped positive Firefox prefix path retains it while unconfirmed.
Recheck all assignments to owned_append_actual_before before relying on this
distinction. Multiple owned appends may carry the same anchor; none grant edits.
Snapshots/selection/caret/predecessor identity must remain consistent. Future
packages do not change this protocol; persistent caches are not introduced.

Proof: actual selector+adapter RED above; GREEN regression plus foreign first
partial followed by internally consistent foreign partial/full,unknown or
missing source,unowned append,selected/changed/duplicate/regressing snapshots,
focus/layout/owner contradictions and no Firefox dispatch. Reuse existing
queued-caret/append negative fixtures and hermetic executor. Then full source,
all installed binaries,scoped actual KDEFirefox native scenario and unchanged
Chrome controls,CI/Security and installed updater readback. Keep original FAIL
until a new native result establishes its exact scope; no global0FAIL claim.

Rollback the pure leaf,thin common retention/dispatch and tests together. No
migration or compatibility mode. Remove this special retention only when its
observed Firefox callback ordering no longer occurs and fixed negatives prove
equivalent behavior. Source115 remains immutable during this independent work.

Results: RED only;116 is not built,installed,published or physically accepted.

Primary GREEN CROOT/release116-prefix-green-v2/result.json:1/1 actual isolated
production-adapter/selector testPASS;15.35s compile,0.047s test. Firstcandidate
v1 failed before key-bound proof retention;diagnosticv1/v2 retained asFAIL,
not retried as acceptance. Their state showed the first inert receipt had
spent its anchor. The new scoped branch initially omitted the neighboring
caret branch's use of the saved actual anchor when managed input intentionally
invalidated the current snapshot. Reusing that exact existing pre-effect
source fixed the primary regression. New guard KnownStart remains;no admission
or output safeguard was removed. Negative/multi-append closure still pending.

Scoped closure GREEN CROOT/release116-prefix-closure-v2/result.json:37/37
actual module test identitiesPASS in3.558s after15.29s compile. Three new test
identities include two positive schedules (single owned key and queued/repeated
interleaving),11 actual adapter negatives and20 Unicode/geometry negatives.
The first closure retained36/37PASS with a new duplicate/regressing-prefix
failure. Corrected retention advances the SAME existing anchor to each checked
whole actual field,with the same owner/known predecessor. The next comparison
must advance from that newest actual field,so a duplicate cannot masquerade as
progress after managed input invalidates the live view. Each accepted hop
preserves all external text from its predecessor; no history or extra owner.
Original source receipt establishes the chain; partial hops remain inert.

The original preflight's 'original anchor' means original whole-field provenance,
not an immutable old snapshot that would permit duplicate progress. This
refinement changes only the same existing pending receipt. All earlier failed
candidate/diagnostic receipts remainFAIL;none was installed. Full3614source
proof,release build and actual native acceptance remain pending.

## Verified source and deployment — 2026-10-30

Full source receipt: `evidence/release-1.0.116-source-gates.json`. Actual3614/3614 Rust tests,34 targets and490 process-isolated cases passed. The first wrapper exit1 was an obsolete manifest binding after3 new test identities; raw exit retained, current manifest/actual observation/zero-failure ledger renewed, independent CI still required. Lints,439 Python tests (10 skips),MSRV,architecture and all10 installing release binaries passed.

Source closure1327 files, SHA-256 `545f941b2b3cc8fa18b25600fc03ec607d6c0e630015bcc0f76e872fd74efcbe` matches canonical imported source. Remote KDE116 loaded and `/proc` bytes verified: daemon1751547/41acd9b9,IME1751630/1399f098,tray1751764. Exact config6e62eab3 preserved; global IBus,models and localGNOME untouched.

Private Firefox iframe-rich and text consecutive-two active cases eachPASS1/1. Their traces did not exercise the newly retained interleaved-prefix branch, so these are controls,not an independent causal proof. Full220 native Firefox running; release/native acceptance pending. Harness wrapper v1 stopped before input on receipt status spelling; v2 multi-selector produced0rows because frozen r19 actor only accepts one selector. Both nonacceptance artifacts retained. Valid scoped runs used one selector each and unchanged r19 actor.

## Final source/native verdict before publication

Native receipt `evidence/release-1.0.116-native.json`: full Firefox165PASS/15FAIL/40BLOCKED/220. Exact private iframe-rich consecutive-two active row155 changedFAIL→PASS. Real newly retained prefix branch occurred once in normal plain Tab row80,whose own trace is complete and resultPASS. Global debug trace rotated:211 complete row traces,9 incomplete; no complete-global-trace claim.

Compared with114:10FAIL→PASS,4PASS→FAIL,2PASS→BLOCKED,2BLOCKED→PASS. Remaining15FAIL are all rapid-five after Space late-target deadlines. This unpaired run does not prove speedup or unchanged per-case timing. Fresh unchanged Chrome/Firefox normal roundtrip controls2/2 cases(4flips)PASS; exact config restored. Chrome59.3/37.5ms,Firefox138.4/166.8ms. Standard full-run recorded transitions n102,median141.44ms,p95173.90ms,max213.19ms,nearest-rank p95; distinct passing subsets prohibit causal timing comparison.

Self review checked the two runtime files against the declaration and34 frozen adjacent hashes. No independent-agent review: user requires solo/noHarness. Source/common verifier and executor,GNOME/Chrome/terminal policy and all deadlines unchanged. Verdict is the bounded source defect plus native controls,not all-client acceptance. CI/Security/publication/updater pending. TD162 remains a separate following candidate,not included in runtime116.

CI first attempt41102635991 failed at field scope before compilation: new TD162/163 Markdown notes were not included in the declaration. Metadata-only follow-up adds those exact two paths to scope/owning ADR. Guard implementation and all runtime/test/model bytes unchanged; retained source/build/native evidence remains applicable. No tag was created on the failed attempt.
