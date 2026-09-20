# Lay 1.0.73 finalization and release evidence

Status: `PUBLISHED_VERIFIED`.

This document owns the final repair, verification, installation and publication
transaction for 1.0.73. The source starts at commit
`02e20854720161a9714aa6dc150edcef8f9ffd67` plus the uncommitted review-fix
patch. The installed runtime remains 1.0.72 until every release gate below has
passed and the exact accepted binaries are installed.

## Demonstrated failures

The fixed review snapshot passed the remote focused `lib:lay +
bin:lay-ibus-engine` lane at 2358/2358 and `bin:lay-daemon` at 266/266, but the
automatic affected check stopped before Rust execution because two new daemon
test identities and one removed identity were absent from the canonical test
manifest. That is a contract failure, not a test PASS.

The bounded physical-input drain forwards typing and a selected navigation
allowlist. `Ctrl+PageUp` and `Ctrl+PageDown` are already classified by the
daemon as context-changing shortcuts, but neither key is advertised by the
virtual keyboard nor accepted by the drain. If their complete press/release
sequence arrives while the physical device is grabbed, the sequence produces
no virtual output.

The pending Firefox exact-receipt regression observes Delete and Commit member
names but accepts any committed replacement other than `abc`. It therefore
does not establish the exact `abc -> фис` surface required by the semantic-test
contract.

## Consequence analysis before production code

### Designs considered

1. Add PageUp and PageDown to the local `match` in `physical_input_grab.rs`.
   This repairs the examples but leaves virtual-device capability and drain
   admission as two independent lists that can drift again.
2. Clone every capability from the physical evdev device into the virtual
   keyboard and forward every captured key. This would broaden the virtual
   device identity, media/system-key authority and multi-device behavior well
   beyond the demonstrated defect.
3. Keep the bounded virtual-key capability set as the single admitted replay
   boundary. Add PageUp/PageDown to that set and have the drain query the same
   capability predicate instead of maintaining its own navigation allowlist.
   This is the selected design.

### Selected route and invariants

The existing daemon remains the only physical Double Shift detector and the
existing `PhysicalInputGrab` remains the only queue owner. No timer, cache,
fallback, detector or text authority is added. Modifier events still update
the captured modifier state; printable keys still update `WordBuffer`; every
other key admitted by the virtual keyboard is forwarded as one closed chord
and resets the daemon word mirror because it may move focus, cursor or visible
content. Unsupported physical keys remain outside this bounded replay
capability and are not silently promoted into new system-key authority.

The 12 ms modifier-settle and 20 ms total drain budgets remain unchanged. The
new membership check is over a small static slice, so it adds no allocation,
I/O, blocking wait or asymptotic cost. PageUp/PageDown add two key bits and at
most the existing small chord frame; package size and RSS impact are expected
to be below measurement noise and must not be described as measured.

Candidate/lattice retention, ranking, L1.1/L2/L3/L4 authority, SafetyGate,
verifier behavior, model packages, delta reloads, lexical caches and learning
feedback do not change. Command replay continues to clear `WordBuffer` rather
than invent typed characters, so a successful PageUp/PageDown chord cannot
become learning input. Emission failure keeps the existing fail-closed closed-
frame cleanup and logs the failure.

Firefox production ownership is unchanged. The test will decode the actual
`CommitText` payload and assert the exact `фис` surface, final tail and target
layout; the contradiction case must continue to emit neither Delete nor
Commit and must clear the pending command without changing `abc`.

The rollback boundary is the connected 1.0.73 commit: revert the shared
virtual-key admission, the two key capabilities and their regression tests
together. Test-manifest refresh is generated only after the final test identity
set is stable. No installed authority changes during implementation or remote
verification.

## Independent review pass 1: closed-frame repair

The first independent source review found that the connected-grab chord replay
constructed a balanced frame but called `VirtualDevice::emit` directly. A
partial uinput failure could therefore leave a replayed Ctrl, Alt, Meta or Shift
pressed even though the drain logged the error and continued. `Drop` releases
the physical grab only; it does not release virtual keys. This contradicted the
closed-frame cleanup claimed above and blocks release.

The repair must route the complete modifier-plus-key chord through the existing
`text_output::key_emit` closed-frame owner. That owner already preserves the
first error and attempts one release frame covering every advertised virtual
key. The physical drain will supply only its captured modifier list and target
key; it will no longer construct or emit an independent uinput frame. A
controlled emitter failure test must exercise the real chord helper with a
command modifier and PageUp/PageDown-capable key, then prove that the next
attempt is the complete release-all frame. No retry, sleep, fallback output or
new runtime authority is added.

## Required proof before installation

- old-code/controlled reasoning: the prior drain rejects PageUp/PageDown after
  modifier capture because neither the boundary allowlist nor `is_typing_key`
  admits them;
- focused daemon and IME/lib lanes with nonzero exact test discovery;
- automatic affected correctness/package lane with canonical manifest MATCH;
- full release gate and release-binary version/hash manifest;
- architecture graph refresh bound to the accepted source;
- native/client checks required by the changed Double Shift and Firefox paths;
- installation snapshot, exact-byte atomic install, Lay-managed restart only,
  source/installed/loaded version and `/proc` executable hash parity, and
  preservation of the global `ibus-daemon` PID;
- publication readback of every pushed ref and public release artifact.

## Results

The final repair routes every captured modifier-plus-key command through the
shared closed-frame emitter. The virtual keyboard now advertises PageUp and
PageDown, and the physical drain queries that same capability set. The new
failure-injection test proves the exact `Ctrl down -> PageDown down/up -> Ctrl
up` command frame, preservation of the first injected error, and a complete
release-all cleanup frame. The Firefox pending-receipt proof decodes the real
`CommitText("фис")`, confirms no second mutation when the client publishes the
final `фис` surface, and applies the layout transition only after that exact
postcondition.

Independent review pass 1 found the direct-emission partial-failure gap above;
the common closed-frame repair resolved it. Review pass 2 inspected the common
cleanup path, PageUp/PageDown capability ownership, exact Firefox commit/tail/
layout proof and mismatch rejection and reported no remaining blocker in this
scope. Production code did not change after that review.

Remote focused verification after the repair passed 2625/2625 across
`bin:lay-daemon`, `bin:lay-ibus-engine` and `lib:lay` at
`run-nbeub7_w`. The final Firefox-specific lane first failed 562/563 because
the new assertion checked layout before the existing exact-visible-
postcondition receipt. The fixture was corrected to model the real two-stage
contract: edit dispatch, then exact final surrounding-text confirmation. The
same lane then passed 563/563 at `run-nct0fz52`. This correction changed only
the proof timing and assertions, not runtime code.

The remotely generated canonical manifest contains 2,890 tests. Relative to
the prior refreshed snapshot, it adds only
`text_output::key_emit::tests::failed_command_chord_preserves_first_error_and_releases_every_virtual_key`;
known failures remain empty and only their manifest hash binding changed.
The final automatic affected lane passed all 2,864 selected tests at
`run-rfz4a08r`; source archive SHA-256 is
`38be2bc85d2b1d7cb2a8f4ed21b76c7e9c4ca2a817a2b2d8d6a7f2229d0d02e5`.

The final frozen release transaction is recorded below. Native Firefox,
installation and publication evidence remains pending. Installed runtime
authority is still 1.0.72 and has not changed.

## Failed release attempt and remediation preflight

The first frozen release attempt used source archive
`b99b88c83ebb7fabb1cc5b8bf8ed083cfec347bc4e0ff753792f6f20bfaf02cb`
in remote run `run-FyjwlB`. Retained clippy passed with zero compiler errors,
the 2,890-test manifest reproduced byte-for-byte and the zero-failure binding
validated. `scripts/update-architecture-graph.sh` then stopped the transaction
before changed/full tests, release binaries, clients, installation or
publication.

The refresh exposed two defects. First, the single-owner check searched for
`pub fn typing_assist_pipeline_for_context` without the opening parenthesis, so
the distinct public function `typing_assist_pipeline_for_context_with_layout`
was counted as a second definition. The repair will make the architecture
assertion match the complete canonical function token ending in `(`. This
preserves both public APIs and keeps the one-owner rule exact.

Second, the new grabbed-input modifier-settle path called `thread::sleep`
inside `physical_input_grab.rs`, which is not an admitted delay owner. Adding
that file to the sleep allowlist or moving the same polling sleep behind a new
wrapper would weaken the ownership contract. The selected repair reuses the
daemon's existing fd-based `wait_for_keyboard_event_or_timeout` owner, waiting
on the captured evdev descriptor only while a modifier is unresolved and only
for the remaining part of the existing 12 ms settle budget. The 20 ms total
drain budget, grab lifetime, command classification, closed-frame output,
Double Shift FSM and word-buffer effects remain unchanged. A timeout simply
continues to the existing bounded decision; an fd error logs and ends the
drain. No retry loop, new timer, fallback output or authority is added.

The failed receipt and architecture log are frozen under
`/home/ubu/.cache/lay/development/release-1.0.73-final-20260920-r2/`.
The next release attempt must use a fresh source archive and fresh remote run.

The first compile after the fd-wait repair failed before discovery because the
temporary evdev iterator retained a mutable device borrow while the error arm
borrowed the device for its fd. The repair now captures the raw fd once before
the drain loop and passes that value to the existing wait owner; it does not
change timing or authority. The final remote daemon lane passed 267/267 at
`run-61e7ytlk`. The architecture scope gate passed with no unowned sleep, and
the final automatic lane passed all 2,864 selected tests at `run-l7eb1wik`.
Both checks used source archive
`2eb7d43822d82dcea448daa730316f2213a6e558066922534cbf63de25c9c552`.

## Full-gate lint failure and remediation preflight

Fresh release run `run-pPIymJ` used archive
`c4082e3c7dba16166b4bf2fba2d5028c6723338ce8a921730ca41d025cda0c7e`.
Architecture refresh and `scripts/check-lay-changed.sh` passed. The full gate
again passed the 2,864-test correctness/package denominator with zero known or
infrastructure failures, then stopped in `scripts/check-lay-lints.sh` before
release artifacts. No runtime, installation or publication changed.

The lint inventory reported three additions. `active_typing_assist` is consumed
only by the non-test decoder configuration; decoder tests intentionally use a
fixed local configuration. It must therefore be compiled only under
`cfg(not(test))`. `forward_queued_typing_key` and `forward_queued_boundary`
became test scaffolds when production queue replay moved to the common
closed-chord helper; they must be compiled only under `cfg(test)`. Updating the
dead-code baseline would admit stale production symbols and is rejected.

These three cfg ownership annotations do not change production code, tests,
timing, input authority or output effects. The next fresh run must reproduce
the same manifest, pass the focused daemon and automatic lanes, and then pass
the lint inventory without a baseline change. The failed run receipt and full
log are frozen under
`/home/ubu/.cache/lay/development/release-1.0.73-final-20260920-r3/`.

The cfg ownership repair passed the remote daemon lane 267/267 at
`run-n5nr_1c8` and the automatic lane 2,864/2,864 at `run-mfeo54is`; both used
archive `1510a6a914817b554f18e22f05699e9bcf1c99283e6c63eaa0ace2ff1b8a56f7`.
The exact guarded `scripts/check-lay-lints.sh` command then passed with default
dead-code inventory 534, research inventory 358, zero non-dead diagnostics and
no baseline modification.

## Frozen release gate and ordinary Firefox native failure

Fresh remote run `run-weqYT4` used source archive SHA-256
`189cc1dbc463d1c409d40ca13cc4d24002ab000963c76851e2f0603e98b8cda1`.
Retained clippy, the byte-exact 2,890-test manifest and zero-known-failure
binding, architecture refresh, changed gate, full gate, compiled receipt,
compatibility-adapter build and all four isolated client profiles passed. The
ten fetched release binaries match the run manifest. This establishes the
frozen release build only; it does not override the native result below.

The exact release bytes were then exercised in an owned fresh Firefox 155.0.1
profile through the ordinary production launcher environment: native Wayland,
asynchronous GTK/IBus, no compatibility-library mapping and no forced
`IBUS_ENABLE_SYNC_MODE`. Main Firefox and global IBus identities were preserved
and installed 1.0.72 was restored after every cell. Receipt
`/home/ubu/.cache/lay/development/release-1.0.73-final-20260920-r4/native/native-control-r5.json`
is **3/4 PASS, overall FAIL**. First word, mixed prefix and trailing Space each
reach the exact target and return surface with two delegations. The completion
cell leaves `про`, publishes no hint, accepts no completion and performs zero
manual delegations.

The first loss is before candidate generation. After the managed commits for
`п`, `р`, `о`, Firefox publishes surrounding snapshots for one and two
characters only. The final three-character client receipt is absent after the
last key release, so `context_word_is_known()` remains false and no
precognition work is scheduled. Alt therefore has no published candidate to
accept and opens the browser menu. Candidate ranking, the Double Shift FSM and
manual replay are downstream of this failure and were not exercised in that
cell.

### Post-release retrieval discriminator preflight

The existing engine `RequireSurroundingText` signal is not selected: the GTK
IBus client disconnects its initial callback and earlier controlled attempts
did not force a fresh Firefox snapshot. Granting authority from the engine's
own committed-tail mirror is also rejected because `CommitText` dispatch is not
an independent visible-delivery receipt. Restoring synchronous IBus is rejected
because the measured native Wayland route can lose modifier transitions.

The bounded discriminator keeps asynchronous IBus and the exact snapshot gate.
In an owned Firefox child only, interpose GTK's existing
`gtk_im_context_filter_keypress`; after the real function successfully handles
a printable key **release**, emit one `retrieve-surrounding` signal on that
same outer `GtkIMContext`. The corresponding key press and its managed
`CommitText` precede the release in the client event stream, so this requests
Firefox's actual post-press text without inspecting or fabricating it. It adds
no detector, retry, sleep, text cache, candidate exception or mutation
authority. A stale, missing, selected or contradictory callback remains
fail-closed under the existing owner, focus, epoch, cursor, selection and exact
snapshot checks.

First compile and run this as a cache-only diagnostic adapter against the same
four frozen native cells with synchronous mode absent. The completion cell must
show an exact three-character receipt before candidate publication and the
required `проверка ` boundary before both Double Shift round trips. The other
three cells must retain their exact intermediate/final surfaces, two
delegations and modifier behavior. Any failure rejects the route without a
production edit. A green diagnostic permits a source adapter change only after
controlled callback tests and an explicit source-level consequence update;
release, installation and publication remain blocked until a fresh full gate
and native rerun pass on the changed source.

The first cache-only discriminator is **3/4 PASS, overall FAIL** at
`/home/ubu/.cache/lay/development/release-1.0.73-final-20260920-r4/native-post-release-retrieve/native-control-post-release.json`.
The exact adapter SHA-256
`013f0f5a64be14883e0a36369e56845a14d2d28ca1116adbc3960e141d039820`
was mapped in all four owned Firefox processes with synchronous mode absent.
First word, mixed prefix and trailing Space retained their prior two-toggle
results; completion again had two stale snapshots, no hint and no delegation.
The adapter emitted no bounded request record in any browser log, so this run
did not exercise the proposed retrieval at all. It rejects that compiled
classifier, but cannot distinguish an ABI/event classification error from an
unreached interposition boundary. Installed 1.0.72 was restored, the temporary
adapter was removed, and protected Firefox/global-IBus identities were
unchanged.

The next discriminator is observation-only: emit a bounded record of the real
function result, event type/keyval/state and outer context type for the
completion cell, without emitting `retrieve-surrounding`. This determines the
actual GTK release shape before another retrieval attempt. It changes no text,
input result, candidate state, timing wait or authority and will run once in an
owned Firefox child with the same cleanup guards.

That observation receipt is
`/home/ubu/.cache/lay/development/release-1.0.73-final-20260920-r4/native-post-release-observe/native-control-observe.json`.
The mapped hook observed only Alt press/release. Both calls used an outer
`GtkIMMulticontext` whose selected module was `wayland`; the printable keys did
not cross `gtk_im_context_filter_keypress`. Completion remained at `про` and no
text effect changed. This rejects the ordinary-Wayland filter hook as an
unreachable notification boundary. The adapter was removed and all protected
runtime identities were restored.

The former Firefox launcher changed three variables together: it selected the
GTK `ibus` module, enabled synchronous IBus processing and loaded the Reset
adapter. The measured modifier defect therefore does not identify which member
of that bundle caused it. The next bounded control selects direct GTK IBus for
the owned child and loads the post-release adapter while leaving
`IBUS_ENABLE_SYNC_MODE` absent. This makes printable release events observable
without synchronous D-Bus waiting. Run the same four cells and a separate
repeated `Shift+1` modifier sequence; require exact environment/mapping proof,
the four text round trips, and alternating shifted/unshifted DOM characters.
The control grants no authority from the hook: it requests actual client text,
and all existing exact receipt checks remain authoritative. A failure rejects
this client route; a PASS still requires controlled adapter tests, source
review, fresh release gates and a final native rerun before installation.

The direct-IBus asynchronous control is **3/4 PASS, overall FAIL** at
`/home/ubu/.cache/lay/development/release-1.0.73-final-20260920-r4/native-direct-ibus-post-release/native-control-direct-ibus.json`.
Environment and mapping proofs confirm `GTK_IM_MODULE=ibus`, absent synchronous
mode and the exact diagnostic adapter in all four children. Completion now
publishes `верка`, accepts exact `проверка ` and completes both required
round trips. Mixed prefix and trailing Space also pass. First word reaches
`привет` but its second toggle times out with passive unknown context, so this
route is not accepted.

The adapter log explains a broader input set than intended: it requests
surrounding text on original managed characters and on the physical
delete/replay characters used by the first toggle. Those replay callbacks
therefore create extra Reset/receipt interleavings before the second gesture.
Do not weaken their existing exact-replay settlement checks. The next
observation-only control records real GTK press/release return values, event
identity and modifier state for one original-first-word replay cell under the
same direct-IBus asynchronous environment. It emits no retrieval request. The
purpose is to find an existing client-visible discriminator between a managed
`CommitText` press/release and unhandled physical replay; if none exists, this
adapter design is rejected rather than made word- or timing-specific.

The direct-IBus observation at
`/home/ubu/.cache/lay/development/release-1.0.73-final-20260920-r4/native-direct-ibus-observe/native-control-direct-observe.json`
found no safe event-only discriminator. Original managed characters and replayed
printable characters both produce successful release callbacks marked with
`IBUS_HANDLED_MASK`; replay additionally produces ignored/duplicate callbacks,
but not in a complete one-to-one form that can classify every handled release.
Key names, replay lengths, Shift counts or timing windows would duplicate the
daemon detector and are rejected. The observation hook emitted no request or
text effect and was removed after cleanup.

The bounded client route to test next combines two actual-text notifications.
The post-release request obtains current text after ordinary managed commits;
the existing Reset adapter requests current text after Firefox/GTK resets the
context during physical replay. This does not classify replay keys or infer text
from them. Both callbacks invoke Firefox's existing `retrieve-surrounding`
handler; either may be stale or missing, and neither grants authority. The
current exact snapshot, owner, focus, epoch, selection, replay-scope and
one-shot checks remain the only bridge to mutation. Use direct GTK IBus with
synchronous mode absent in an owned child, combine the two hooks in one
cache-only library, and rerun the four fixed cells once. The existing four-cell
conjunction and separate modifier control remain mandatory.

The combined cache-only route is **4/4 PASS** at
`/home/ubu/.cache/lay/development/release-1.0.73-final-20260920-r4/native-direct-ibus-combined/native-control-combined.json`.
The adapter source SHA-256 is
`916bb9cf23112448c1e9434c71de4ba9aa410dd02e9e0689049883de9caa1293` and
the guarded shared-library SHA-256 is
`290b4ef5af024e89ba72c63c40ae7ac4070d5fbac075029e99487820258fafba`.
First word, mixed prefix, accepted completion with its closing Space, and a
plain trailing-Space word each reached the exact target and exact return
surface with two manual delegations. Completion published the measured
`овод`, `ивет`, and `верка` surfaces, accepted once, and round-tripped
`проверка ` to `ghjdthrf ` and back. The adapter only requested Firefox's
actual surrounding text after handled printable releases and Reset; the
engine's existing receipt checks remained authoritative.

The first modifier-control invocation was a harness failure before input: its
sender printed an object representation instead of the evdev device path. No
product conclusion is drawn from that receipt. After correcting only the
ephemeral sender, the same asynchronous direct-IBus route passed repeated
`Shift+1`: exact DOM result `!!`, two managed `!` commits and zero manual
toggles. Receipt:
`/home/ubu/.cache/lay/development/release-1.0.73-final-20260920-r4/native-direct-ibus-modifier/native-control-modifier-r2.json`.
This is synthetic client-visible input evidence, not human-keyboard
acceptance. Both accepted controls preserved the user's Firefox and global
IBus identities, restored installed 1.0.72, and removed the temporary library.

## Firefox client adapter source preflight

Promote the measured combined adapter into the existing compatibility source.
The Firefox launcher must select `GTK_IM_MODULE=ibus`, preload the adapter
inside Snap confinement, and explicitly remove inherited
`IBUS_ENABLE_SYNC_MODE`; the synchronous route remains prohibited. Preserve
the real GTK filter/reset calls and return value. Request surrounding text only
after a successfully handled printable release with no Ctrl, Alt, Super,
Hyper, or Meta state, and once after an outer Reset. Do not inspect text,
classify words or replay lengths, retry, sleep, cache a snapshot, or grant edit
authority in the adapter.

Add a controlled ABI-level adapter test with fake GTK symbols. It must prove
one real-call/return path and exactly one request for the admitted release and
Reset, plus zero requests for press, unhandled, command-modified, non-printable
and non-IBus cases. Add a launcher contract assertion for direct IBus,
in-Snap preload, and absent synchronous mode. Compile with the release flags,
run the changed/architecture gates remotely, then freeze a new source archive.
The accepted r4 binaries are invalidated by this source change. Installation
and publication require a fresh full release, the same four native cells, and
the modifier control on the source-owned launcher/library.

The source-owned adapter development check passed all **2,864/2,864** selected
affected tests in 341.5 seconds at
`/home/ubu/.cache/lay/development/run-1p9mnig3/RESULT.json`. The same guarded
transaction compiled and executed the fake-GTK ABI regression, checked the
launcher contract, and refreshed the architecture graph. Adapter source
SHA-256 is
`916bb9cf23112448c1e9434c71de4ba9aa410dd02e9e0689049883de9caa1293`;
launcher SHA-256 is
`eacd6bf1f051c8110c8c2606d7b6cb35980e862feb9acc6c1ed7d564745bc4fb`.
This is development acceptance only. Runtime authority remains installed
1.0.72; the fresh full release and exact-byte native cells remain mandatory.

## r5 release PASS and source-launcher native failure

Frozen archive
`116cfabe37888ff3beb27fcf5bef9c92081d383603076d0f634d55d763a8c0be`
passed retained clippy, the byte-exact 2,890-test manifest and zero-failure
binding, architecture refresh, changed gate in 357.0 seconds, full gate in
416.1 seconds, compiled receipt, adapter build, ten release artifacts and all
four isolated client profiles. Receipt:
`/home/ubu/.cache/lay/development/release-1.0.73-final-20260920-r5/RESULT.json`.
The release adapter SHA-256 is
`ce9c258eec489a95dfadb8d23ba03d6f1afc79ab093ce63b7a3a54589a0887de`.

The first native invocation was a harness failure before browser or input: its
copied runner resolved the release directory one level too shallow. The outer
transaction restored the prior compatibility library byte-for-byte; retained
receipt `native-release-route/native-route-transaction-harness-fail.json` has
no product verdict. After correcting only that cache-local path, exact r5
launcher/adapter/binaries produced **3/4 PASS, overall FAIL** at
`native-release-route/text/native-control-release.json`. First word, mixed
prefix and trailing Space passed both exact surfaces with two delegations.
Completion remained `про`, published no preedit, accepted nothing and made no
delegation. Main Firefox/global IBus were unchanged, installed 1.0.72 and the
prior adapter bytes were restored, and the modifier cell did not run after the
failed text conjunction.

The adapter did run and supplied all four requests. The source launcher,
however, overwrote the `LD_PRELOAD` already established inside Snap. Its owned
Firefox mapped only the Lay adapter, while the accepted cache route retained
both Lay and Snap's `gnome-platform/$LIB/bindtextdomain.so`. Under the source
launcher, three managed characters reached the DOM but GTK emitted a single
late Reset. That Reset armed the three-character receipt, received stale
one-character text first, then rejected the exact three-character callback as
a mismatch. The accepted cache route emitted and confirmed a Reset after each
character, allowing each fresh candidate to replace the prior one.

### Snap preload preservation repair preflight

Preserve the already established in-Snap `LD_PRELOAD` after the Lay adapter,
while continuing to unset `IBUS_ENABLE_SYNC_MODE` and select direct IBus. This
matches the native 4/4 cache launcher exactly and does not add another library;
it stops discarding Snap's own compatibility preload. Extend the launcher
contract to require conditional preservation and ordering. Do not change the C
adapter, engine receipt lifecycle, timing, input script, or native expectations.
The r5 release is invalidated by this shipped-launcher repair; a fresh affected
check, full release, four native cells and modifier control remain mandatory.

The corrected launcher contract passed all **2,864/2,864** selected affected
tests in 340.2 seconds at
`/home/ubu/.cache/lay/development/run-od4752nt/RESULT.json`. Corrected launcher
SHA-256 is
`1ef03e5bdffbbd48645bd215e0b744a8f1463409e5f4e0e18dde5c76120763e8`.
This remains development-only evidence; runtime authority is still 1.0.72.

## r6 release PASS and transient readout-denial native failure

Frozen archive
`5d6d9c267db1c7864df21df9f3a287be38cefa41abb2799574aa31ba9098bd53`
passed retained clippy, the byte-exact 2,890-test manifest and zero-failure
binding, architecture refresh, the unique changed gate in 356.2 seconds, the
full gate in 415.9 seconds, compiled receipt, adapter build, ten release
artifacts and all four isolated client profiles. Remote receipt `run-uyAAv9`
and the fetched aggregate are at
`/home/ubu/.cache/lay/development/release-1.0.73-final-20260920-r6/RESULT.json`.
The source launcher SHA-256 is
`1ef03e5bdffbbd48645bd215e0b744a8f1463409e5f4e0e18dde5c76120763e8`;
the release adapter SHA-256 is
`ce9c258eec489a95dfadb8d23ba03d6f1afc79ab093ce63b7a3a54589a0887de`.

Exact source-route native receipt
`native-release-route/text/native-control-release.json` is **3/4 PASS,
overall FAIL**. Mixed prefix, accepted completion with its closing Space, and
trailing Space each show both exact surfaces and two manual delegations. First
word shows `ghbdtn -> привет`, then the queued second gesture stops with
`context admission denied`, leaving one delegation and final `привет`. Main
Firefox and global IBus were unchanged; installed 1.0.72 and the prior adapter
bytes were restored. The modifier cell did not run after this failed text
conjunction. r6 is therefore rejected for installation and publication.

The first loss is the read-only settlement probe, after the first edit already
completed. In the failing first-word trace, bridge nonce 11 completed Ping,
marker observation, wait and take at rows 437--446. A replay key had entered at
row 440 but its callback had not settled, so the fence correctly carried no
admission token. `finish_bridge_fence` maps that empty token to
`AdapterError::Denied`; `VisibleTailV3` currently converts every adapter error
to a D-Bus failure, and the daemon's already bounded settlement loop aborts.
The callback then settles at rows 447--450. Firefox's Reset sequence retains
the inert replay candidate at rows 480--491 and the exact six-character client
snapshot confirms it at rows 492--497. There is no contradictory text, owner
change, expiry, cancellation, bus failure, timeout or mutation before that
confirmation.

### Passive denied-readout repair preflight

Split the bridge token helper so the read-only `VisibleTailV3` can inspect the
structured `AdapterError`. For this method only, map exactly `Denied` to the
existing `passive:unknown-context` empty reply. Keep `Busy`, `Timeout`,
`Cancelled`, bus/configuration failures and every adapter error in mutation,
suppression, cancellation and manual-toggle methods as hard errors. A passive
reply supplies no token, path, text, epoch, focus receipt or mutation authority.
The existing 80 ms daemon settlement loop may make another fresh fenced read;
no timer, retry loop, deadline, queue or fallback is added. Persistent denial
still ends at that unchanged bound, while cancellation and transport failure
still end immediately.

The controlled proof must first fail on the old implementation by placing a
real P2P `ProcessKeyEvent` ingress before bridge marker settlement. It must then
show an empty passive reply and no text effects, settle that exact callback,
and recover the same authoritative surface on a fresh bridge fence. The
existing duplicate-ingress cancellation proof must continue to return a hard
`Cancelled` error for both readout and mutation. Run focused and affected
closures, a fresh frozen full release, all four exact native text cells and the
modifier control before installation.

Consequence analysis: this changes transport presentation for a read-only
sample, not context admission. It cannot add a lattice candidate, retain a
contradicted candidate, rank a surface, satisfy an exact snapshot, consume a
lease, pass SafetyGate/verifier checks or emit client text. The possible CPU
cost is additional fresh D-Bus fences only inside the existing bounded
settlement window; memory, package/model identity, cache keys, invalidation,
reload, learning and feedback are unchanged. Rollback is the helper split and
single `Denied` match. r6 remains the latest complete release proof but has no
runtime authority.

The first old-code fixture run, `run-y7hgog9t/RESULT.json` (remote
`run-w4Racu`), is a **HARNESS FAIL**, not the causal RED: 563/564 selected tests
passed, but the new case's effect probe read the controlled server's expected
detached-object error reply before its proof marker. The readout assertion had
not executed. Consume that exact transport reply with the existing helper,
without changing production or the intended unsettled callback, then rerun the
old implementation.

Corrected old-code run `run-le3cakpe/RESULT.json` (remote `run-IGsOXA`) is the
causal **RED**: 563/564 selected tests passed, and only
`td121_visible_tail_denial_is_passive_until_the_exact_callback_settles` failed.
The real fence returned `Failed("context admission denied")` at the new passive
readout assertion. The setup/effect checks passed and production was still
unchanged. Apply only the structured readout mapping described above.

The structured repair passed **564/564** focused IME tests in 28.9 seconds at
`run-3uunyqkd/RESULT.json`. The new real-P2P test returns empty
`passive:unknown-context` with zero text effects while its key callback is
unsettled, then returns the identical authoritative tuple after that exact
callback settles. The same unsettled state must also return a hard `Denied`
from `ManualToggleV3`; existing duplicate-ingress cancellation remains a hard
error. This is development proof only; affected/full/native acceptance and
runtime authority remain unchanged.

The first affected-closure attempt `run-b6ancqui/RESULT.json` (remote
`run-gNZSAB`) stopped at the canonical manifest gate before Rust execution.
Discovery reports exactly one addition, the new P2P test, with zero removals or
changed rows. Refresh the manifest on that guarded remote snapshot, verify this
exact delta, and rebind the unchanged zero-failure registry before rerunning.
This is an expected contract stop, not a correctness verdict.

After the exact one-row manifest refresh, `run-wll6ea2b/RESULT.json` passed the
complete affected closure: **2,865/2,865 selected**, 358.3 seconds in the remote
receipt (368.7 seconds including transport). Its source archive SHA-256 is
`229e09c340d7c43390d990064c99727765f7c5c2b27924347828fa3873ae63c4`.
The final same-state mutation assertion was added afterward to the existing
test identity; rerun focused proof and require it in the forthcoming frozen
full gate rather than claiming this affected snapshot covered that assertion.

The first focused run of that assertion, `run-ho7o518l/RESULT.json` (remote
`run-G9aoL8`), is **BLOCKED before discovery**: a test-local variable shadowed
the `bridge` helper on its second call (`E0618`). Direct guarded compilation of
the same snapshot exposed that exact diagnostic. Rename only the two local
bindings and rerun; no runtime code or test semantics change.

Corrected focused receipt `run-c_ir4jlp/RESULT.json` is **564/564 PASS** in
34.3 seconds remotely (46.1 seconds including transport). It covers the
passive readout, same-state hard mutation denial, exact post-settlement recovery
and the existing cancellation controls. Freeze a new r7 source now. Its changed
lane will retain every non-Rust-all command in the hash-bound changed script;
the canonical full gate will execute the 2,865-test correctness/package lane
once, then clippy/lint, architecture receipt, release build and four clients.
No r6 artifact may be installed as r7 evidence.

## Frozen r7 release and exact native acceptance

The frozen r7 source archive SHA-256 is
`09b3b396bfec998764aef6d135797c1f03214ab7f0ad9f09fe557db291237981`.
Remote receipt `run-bJ5E28`, fetched as
`/home/ubu/.cache/lay/development/release-1.0.73-final-20260920-r7/RESULT.json`,
is **PASS**. The hash-bound changed lane passed in 31.3 seconds; the canonical
full gate passed all **2,891/2,891** discovered tests once in 477.4 seconds.
Clippy reported zero compiler errors. Manifest/known-failure binding,
architecture refresh and compiled architecture receipt passed. All four
isolated client profiles passed. The release contains ten binaries, each bound
by the receipt; `lay`, `lay-daemon` and `lay-ibus-engine` report 1.0.73.

The exact r7 Firefox source route then passed all four text scenarios and the
modifier control. First word, mixed prefix, accepted completion with its Space,
and trailing Space each returned to the exact original surface with two manual
delegations and zero malformed trace records. Repeated `Shift+1` produced
exactly `!!`, two managed commits and zero manual toggles. The transaction
receipt is
`release-1.0.73-final-20260920-r7/native-release-route/native-route-transaction.json`;
both child return codes are zero and the temporary adapter swap was restored.
The source-route harness preserved the pre-existing main Firefox, global IBus
and installed 1.0.72 runtime identities. This is synthetic client-visible
native acceptance; a physical human-keyboard run is not claimed.

r7 is the first accepted installation candidate. The accepted adapter SHA-256
is `ce9c258eec489a95dfadb8d23ba03d6f1afc79ab093ce63b7a3a54589a0887de`;
the launcher SHA-256 is
`1ef03e5bdffbbd48645bd215e0b744a8f1463409e5f4e0e18dde5c76120763e8`.
Installation must use the ten exact receipt-bound binaries plus those exact
client files, restart only Lay-managed owners, and prove source/installed/
loaded hash parity before publication.

## Installed runtime acceptance

The rollback-capable installation transaction completed with status
`INSTALLED_LOADED_HASH_VERIFIED_PHYSICAL_PENDING`. Receipt:
`/home/ubu/.cache/lay/development/release-1.0.73-final-20260920-r7/installation-1.0.73.json`.
All ten installed files and their `~/.local/bin` links match the r7 artifact
manifest. The loaded daemon, managed IBus engine, L3 watcher and L1.1 server
match their installed hashes; the L1.1 health contract passed. CLI, daemon and
engine report 1.0.73, and the loaded GNOME extension reports 1.0.73. The exact
adapter and launcher hashes match the accepted r7 values.

The global `ibus-daemon` retained PID 4062416 and start identity. The existing
main Firefox retained PID 2748943 and executable identity; it was not restarted.
GNOME input sources, current source `lay-ime-ru`, configuration bytes, desktop
entries and Firefox CLI wrapper were unchanged. Only Lay-managed runtime owners
were restarted. The pre-install 1.0.72 backup is
`/home/ubu/.local/state/lay/release-backups/1.0.73-preinstall-20260920T054651Z`.
Physical human-keyboard acceptance remains `NOT_TESTED`.

## Publication readback

The accepted private source is commit
`96d6b6697a3583318537738d99066ad4c6206a1f` on
`origin/codex/review-fixes-1.0.73`; live `ls-remote` returned that exact object.
Publication used an isolated child of `public/main`, with no private-history
merge. Its staged tree `0c15612aa0326c0004e5b1ef29f3460056f33317` was
byte-identical to the accepted private commit tree before the public commit was
created. Public installer regressions passed **8/8** on that isolated tree.

GitHub readback reports public `main` commit
`9c9fc3c6de7e119caf7479d956b3080b0e4b2764`. Annotated tag object
`0f44cafcd74e0fc567a94f31f272fda0f79e448e` dereferences to that exact commit,
and both commit and tag resolve to tree
`0c15612aa0326c0004e5b1ef29f3460056f33317`. GitHub Release
[`v1.0.73`](https://github.com/radislabus-star/lay-public/releases/tag/v1.0.73)
is published, non-draft and non-prerelease. Live content readback confirmed the
1.0.73 README, both test denominators and the tagged evidence document. This
closes installation and publication; the physical-keyboard limitation remains
unchanged.
