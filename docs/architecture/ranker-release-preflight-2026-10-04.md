# Native agreement ranker release preflight — 2026-10-04

Status: INSTALLED_EXPERIMENTAL_1_0_78; broad physical-client testing in progress.
User explicitly requests ranker release/install, then continued improvement.
Source base:79b09d06306a018961ace783e2542eb817de7b30, dirty isolated branch
`codex/lay-ranker-release-20261004`. Previous accepted1.0.77 IME58dcaacc… is
preserved for rollback. Current installed experimental1.0.78 C03 IME SHA-256
`a1bdffe17771e241234b6bf130babde052e4bac31578741c0d7de780df6ebaee`;
receipt `revision4/c03-install/INSTALL.json` under the release evidence root.
C06 and C09 post-install-publication source fixes are imported but NOT_INSTALLED.
Publication/tag1.0.78 remains pending. Earlier entries below are historical.
The authoritative research roadmap/owner remain in
`/workspace/local/lay-syntax-agreement-20260929`; this owns deployment only.

## Facts and first missing transition

COMPLETED_101 is Python dictionary completion and factorial candidate reranking,
not a ready native artifact. Offline R0 is a rejected LightGBM/MLM/raw38 control,
not the live scorer. The measured research hybrid97.4% prepared and70.7%
natural rank are offline only.
The runtime computes candidate admission before ordering in
`src/typing_transition/decision.rs:241–333`. Admission itself consumes ranks: a
pre-admission score mutation could alter Apply/Keep. `ClosedExact` returns first;
retained-exact authority overrides the ordinary ranking. Existing canonical
`morphology_slot_identities_for_surface` supplies complete lemma/feature masks,
while generated MorphologySlotEvidence is a lossy projection. Current IME passes
prefix+last token, with no future noun on the right.

First missing transition: observed source/neighbor/candidate full readings ->
portable compatible-reading selection -> ordering of already admitted candidates.
Canonical-provider equivalence to OLDTRAIN+OpenCorpora is NOT_PROVEN.

## Designs considered

| Design | Assessment |
|---|---|
| Native portable agreement policy after existing admissions, existing canonical full-mask provider | Selected: bounded lookup, same decision owner and package identity, no new authority or process. New native provider/scores scope requires proof; historical percentages cannot be transferred. |
| Complete Python R0+MLM+dictionary serving pipeline | Viable for later offline/shadow work; missing live provider, process lifecycle, full causal prefix and latency/RSS proof. It expands deployment/dependencies and cannot honestly be installed as a ready artifact. |
| Modify current rank_score before admission | Rejected: those scores feed margin/authority gates and would change Keep/Edit. |

## Consequence analysis before code

- Candidate retention: all original candidates/evaluations remain. The ranker
  observes only already admitted indices, adds/removes none. Empty admissions,
  one admitted candidate, ClosedExact and retained-exact results stay original.
- Rank/false authority: mirror the observed compatible-signature policy using
  existing score preference, source-family identity, unique nearest witness,
  tie refusal and old-support veto/person1/2 exception. The ranker supplies
  order only; it cannot create an admission or replace L1 retained authority.
  A correct old candidate may still be displaced by an incorrectly recognized
  grammatical cue: source/provider/native comparison and clean outcomes matter.
- Context/time: exact existing event text/current last-word focus, bounded left8
  and available right4 only, punctuation boundaries respected. Current Space
  has no right words; adjective branch receives no fabricated evidence. Unknown
  readings/roles return original ranking, no field/word uncertainty veto.
- Latency/CPU/RSS: existing warmed canonical package, point lookups only, no
  model inference/corpus scan/lexeme search. Bound candidate/context/reading
  work explicitly. Tail/allocations/native timings remain NOT_MEASURED and must
  be checked remotely; additional dictionary lookup on a cold package can be
  expensive, so avoid adding a new lazy load inside this rank path.
- Cache/reload: reuse installed L2 identity/lifecycle only. No second cache,
  package path, worker, timer, online weights or source of truth. If existing
  provider is cold/unavailable, preserve old order without loading a package.
- Learning: chosen existing candidate retains its original evaluation, action
  and receipt; score permutation never becomes positive feedback. Existing
  observed-result feedback remains sole learning outcome authority.
- Concurrency/staleness: synchronous bounded evidence within existing decision
  evaluation; no new async request. Existing field/revision lease and verifier
  apply to selected original candidate. Provider is immutable within its
  current existing generation; no token context retained across focus change.
- Failure/rollback: missing/ambiguous/nonfinite/incompatible evidence returns
  original order. Preserve exact old binaries/config/packages before install;
  atomic replacement and existing managed-engine activation only after proof.
  User authorization covers this install, not global IBus restart or sources
  migration. Rollback restores exact old bytes, not a rebuilt substitute.
- Compatibility: common core serves IME/daemon, no gesture/output protocol
  edits. Integrate ordinary correction order first. IME preedit ranking has a
  separate existing core interface; do not silently call it improved if not
  connected and measured.
- Maintenance: one native policy module under decision owner and full-mask
  adapter. No phrase/suffix/client-specific exceptions or copied corpus as code.
  Offline parity and native scope are separately versioned; future refinement
  replaces this same policy/provider rather than adding a second selector.

## Implementation and proof boundary

Implement the generic policy plus native adapter after immutable admissions,
inside TransitionDecisionCore; carry a bounded reason for the chosen order.
Preserve exact authority override and every verifier/SafetyGate input. First
remote proof: actual existing core admission/selection effects, portable
policy parity, ambiguity/ties/unknown/technical text, candidate conservation
and Keep/Edit invariance. Required architecture refresh and affected source
gates/full release/client gates remain truthful, with no weakened assertion.
No new research fit/model inference or sealed tuning. Changed native runtime
requires exact binary/installation evidence and its own physical scope.

This is a native adaptation of the measured policy, not installation of the
research R0 model. No quality claim from prepared97.4% or tree-shape100%.
Independent per-class>95%, clean>=99.99%, full natural quality remain unmet.
User's release request permits preparation/install while these remain explicitly
unmet; they are never relabeled PASS. Prototype/experimental release status
must remain visible until the complete contract is satisfied.


## Connected source — 2026-10-04

The native policy is connected after `surface_authority_admissions` in
`src/typing_transition/decision.rs`; retained-exact precedence is unchanged.
`decision/agreement_order.rs` owns ordering, with 11 new contract tests in its
`tests.rs`. The canonical full-mask adapter in `l2_field/mod.rs` reads the
existing OnceLock via `get()` only; `l2_field/runtime.rs` adds a bounded imported
binding iterator. At most64 candidates,64 readings per surface,64 surface
characters and8 left tokens participate; at most73 warmed point lookups.
Current right-context availability is zero. No model/package or config changes.

Source inspection and whitespace/format preparation are complete; executable
proof, native per-class quality and resource measurement remain untested.
Version surfaces are1.0.78, intended as an experimental GitHub prerelease.
Accepted1.0.77 bytes/config are backed up at
`/home/local-user/.cache/lay/development/ranker-release-20261004/ROLLBACK.json`.
No installed bytes or process have changed.


## Remote attempt1 — 2026-10-04

Frozen archive9415fbfd65bee0a278a22aa2aeca2b1ed96535f23f4b15b3e3d7ac55a3cd6cc4
(1444 files) staged at
`/workspace/worker/lay-development-runner/ranker-release-20261004-jyCWP8`.
Independent held-source review: zero authority/source design blockers, no claim
of native quality/latency. Architecture refresh PASS44.906s;8 canonical graph
files delivered (optional vocabulary cache was not generated). An initial
delivery helper expected that optional cache and failed; graph execution itself
had passed, and eight-file hash-verified delivery completed without rerunning it.
Full gate FAIL50.270s at compile before test discovery: private morphology
module access (E0603) and fixture `&&str` versus `Into<String>` (E0277).
Receipt: `/home/local-user/.cache/lay/development/ranker-release-20261004/ATTEMPT1_FULL_EXECUTION.json`.
No functional test or quality percentage established; no live authority/bytes
changed. Next exact revision fixes the interface/reference and adds the
canonical inventory using the existing classifier and verifies it against actual
Cargo discovery, without dropping old tests.

Revision2 uses narrow existing `l2_field` crate re-exports; the morphology
module remains private. The fixture reference is fixed and unused future-noun
extraction removed. Policy semantics/admission/receipt/exact precedence stay
unchanged. Canonical test inventory update is authorized only as an
additive change: every existing test, classification and assertion must remain.
Its eleven added rows are derived with the existing classification/isolation
functions and must match actual Cargo discovery in the full gate.


## Revision2 graph relocation diagnosis — 2026-10-04

Cargo compilation and actual manifest discovery succeeded:2977 tests total,
2915 correctness,36 package,11 performance,15 ignored. Full non-timing proof
is executing separately; no final verdict is asserted yet.

Graph refresh FAIL28.518s (WATCH): all three mutation sinks lacked graph
parameter_type references to AuthorizedEdit. Exact sink source and guard SHAs
are unchanged. The first failed transition was copied Graphify AST cache
`v0.9.11`: its absolute IDs/origin_file stayed at the old jyCWP8 checkout while
only source_file was reanchored. The resolver expected the new vriyGa root;
type-reference edges fell21038->5103, AuthorizedEdit refs13->2.
Independent diagnosis located this in cache.py:378/extract.py:4247–4310.
Fix: preserve rejected graph/cache outside active source and rerun the existing
wrapper with a fresh AST cache under the same resource guard. No manual
capability edge, relaxed rule or runtime authority change is permitted.
Receipt remains `/workspace/worker/lay-development-runner/ranker-release-20261004-r2-vriyGa/GRAPH_EXECUTION.json`;
new attempt uses a separate GRAPH_REPAIR_EXECUTION.json.

Native non-timing target logs report zero functional failures; all eleven new
ranker tests passed. The lane's final result is BLOCKED_CONTRACT, solely because
the known-failure ledger still binds the old inventory SHA. Rebind only that
SHA to the additive manifest; keep zero failure entries and historical
observation bytes untouched. This does not promote that historical observation
to proof of the new source; the complete full gate will execute afresh.
Fresh AST-cache graph repair PASS44.410s; source-backed capability references
are restored by extraction. Rejected graph/cache and original WATCH remain.


## Guarded full gate and user test-install priority

The full non-timing lane passed2951 selected tests (2915 correctness+36 package),
including all11 native ordering tests; measured execution397.074s. Full release
script stopped later at lint: the new test function ended in uppercase `_C`.
Only that cfg(test) identifier is renamed `_c`, with its canonical inventory
row and ledger SHA updated; every assertion and production Rust byte remain
unchanged. Test-name metadata is not a runtime algorithm or safety bypass.
Full receipt: `/home/local-user/.cache/lay/development/ranker-release-20261004/revision2/FULL_EXECUTION.json`.

User explicitly now requests replacing the runtime with1.0.78 for their own
manual test. Test-install is distinct from final release acceptance. Run the
strict graph/lint checks and build the exact production candidate remotely,
then install the verified bytes with rollback and loaded-process proof.
No new automatic window matrix is selected before that user-requested test.
Publication still needs the final full gate; its quality stage remains
experimental, with native per-class and physical-client acceptance pending.


## Exact experimental installation and broad client proof — 2026-10-04

Release build PASS232.249s, strict canon and format PASS. Native functional
proof:2951 selected PASS, with unchanged production and assertions; only one
test identifier/inventory row renamed. The embedded architecture receipt is
reused only after schema/verdict/sorted check id/status projection parity and
old receipt SHA verification. Independent review caught the initial overly
broad receipt whitelist; it was tightened before installation.

Candidate2 full lint FAIL: stale dead_code baseline (4 added/2 removed in
preedit.rs/adapter.rs). Candidate3 strict Clippy FAIL on two production style
issues in observation.rs and three immutable-reference test call sites in
residuals.rs. All four files match accepted77 source bytes; no lint flag,
baseline, assertion, runtime input route or canon guard was relaxed. These FAIL
receipts remain FAIL. Publication acceptance is BLOCKED_LINT. The user's
explicit immediate experimental78 test-install priority is recorded separately
in TEST_INSTALL_SCOPE.json, not promoted to full acceptance.

Ten exact candidate binaries and two version surfaces installed atomically,
loaded SHA parity verified for IME/daemon/L3/L11; global IBus PID270775,
configuration and selected Lay source preserved. Exact rollback77 is retained.
Receipt: /home/local-user/.cache/lay/development/ranker-release-20261004/INSTALL.json.
IME SHA:e64ccde4e7e495ac7ea4479c3f353fb4b64e54cc087be17ebe67daa31f33adac.
Runtime bytes changed:true; ranking activated, authority/gesture/output
contracts unchanged. Native quality percentages remain unmeasured.

The user subsequently explicitly requested broad local-window testing, including
eight rapid Double Shift with visible word/mode after each, next letter, Space,
Tab, held Shift, numeric/punctuation hint denial, and focus boundaries. Local
physical testing is authorized; Rust/model/build work remains remote. A single
virtual keyboard is attached to the installed managed daemon by a scoped
managed-daemon restart; no second daemon or global IBus restart is used.

GTK p->Tab plus one space/focus retained and held Shift:2/2 actual cases PASS.
The first new wide harness had a D-Bus boolean parsing defect and early focus
observation; its196 blocked attempts are not product tests or a quality
denominator. The next narrow/wide attempts preserve separate receipts.
Current valid seven-field native matrix (GTK3/GTK4 Entry/TextView, Qt line/plain/
rich edits):437 recorded groups/steps,423 PASS,14 recorded misses, pending
classification of focus/setup/timing versus product failures. This is not a
ranker accuracy percentage. Repeated rapid Shift showed a changed mode while
the word remained old in GTK; a Qt word appeared just after the500ms observation
limit. Plain `кторое` remained uncorrected; no claim of restored typo coverage.
Broad browsers/actual app fields and visual GNOME icon pixel proof are pending.
Receipts: windows/GTK_TAB_SHIFT.json, windows/NATIVE_MATRIX_*.json and private
IME trace snapshots under the same local release evidence directory.
Required next proof: complete current client matrix, locate first failed shared
authority/observation transition, and retain the scope of every missing test.


## Physical publication failure and observer repair — 2026-10-04

These observations concern installed IME
`e64ccde4e7e495ac7ea4479c3f353fb4b64e54cc087be17ebe67daa31f33adac`.
The upcoming source repair is not installed or accepted yet. The ten installed
binary hashes and original rollback remain bound to INSTALL.json.

- First wide native receipt:437 observed groups/steps,423 PASS,14 misses. These
  counts are client acceptance, not grammatical-ranker accuracy. The causal
  audit separates3 Shift deadlines,6 unchanged standalone typos,1 Space word,
  3 focus/observation failures and1 setup failure. The original196-case invalid
  observer run remains excluded, not relabeled as functional PASS.
- Chrome input/textarea/contenteditable:198 executed groups/steps,190 original
  PASS,8 misses in WIDE_MATRIX_V2_20261004T060845.json. Three held-Shift misses
  sampled active completion (`ПРОв` with `ести` ghost) as committed text; they
  require the new Escape-dismissal proof, not retrospective PASS. Three typo
  misses preserve `кторое `. Contenteditable additionally loses published mode
  after a correct visible conversion/Space. Remaining counts are still scoped
  to this original observer receipt.
- Independent native Shift+held-Shift proof with the actual visible AT-SPI Lay
  panel label:14 executed groups,12 PASS,2 misses, receipt
  WIDE_MATRIX_V2_20261004T062628.json.500ms was an original harness observation
  bound, not a user SLA. The new observer saves samples/times and independently
  checks a1.5s convergence bound before the next pair; old misses remain misses.
- CurrentInputMode reads the GNOME published property, not Rust's private
  decoder. The old harness key `mode.decoder` names that published observation;
  only a subsequent physical character proves actual decoding.

First proved C09 failure: Chrome contenteditable changes `ghbdtn` to `привет`,
but published InputMode and the actual showing/visible panel label stay EN.
The next physical KEY_F yields `привета`, proving actual RU decoding.
Wire receipt IBUS_PROPERTY_MONITOR_BASELINE.log observes EN registrations then
RU RegisterProperties on the same engine `/lay_ime_us/1056`, serial28608; across
its bounded interval there are17 RegisterProperties and0 UpdateProperty.
GNOME's installed ibusManager accepts initial registration once and subscribes
persistently to update-property. The existing output.register_input_mode emits
registration only. This is the first established publication mismatch; no
ranker/word-edit authority failure is inferred from it.

Repair owner: existing IME output publisher and standard Engine property signal.
Add standard UpdateProperty for changes of InputMode; preserve initial
registration, existing ContextAdmissionReducer/ManualToggle/preedit ownership,
verifier/SafetyGate and source IDs. No separate owner, route, queue or timer.
Prove the real Legacy transport through two actual manual toggles and wire
RU/EN properties; record exact new source/build bytes before runtime promotion.

Firefox observer failure is separate: the loopback fixture's no-referrer policy
produces `Origin:null` POSTs rejected400 by its strict origin guard. The bounded
server metadata receipt BROWSER_TRANSPORT_NO_REFERRER_CAPTURE.json confirms
actual Firefox400|null counts without recording bodies or credentials. Using
same-origin Referrer-Policy on this owned localhost page preserves the exact
Origin guard. No Firefox user preference or Lay route changed. Normal-profile
physical testing resumes on the repaired observer; earlier setup BLOCKED rows
remain BLOCKED. The temporary fresh-profile retry created no product proof.

Evidence root: /home/local-user/.cache/lay/development/ranker-release-20261004/windows.
Owning audits: NATIVE_FAILURE_CAUSAL_AUDIT.md, CHROME_SHIFT_CAUSAL_AUDIT.md and
ICON_READBACK_AUDIT.md; exact receipt paths/hashes are inside those files.
Runtime authority contract: unchanged. Full release remains BLOCKED_LINT;
experimental install is distinct from public tag/release and native quality.

## C09 publication source proof and remaining observation limits

The connected production repair changes only the existing Legacy mode
publication to the standard IBus `UpdateProperty` signal. Initial
`RegisterProperties`, the decoder owner, Atomic transport, failure rollback,
manual edit admission and verifier authority remain in their existing owners.
The existing real P2P transport test now invokes two production manual toggles
and checks the native preedit surface, cursor, RU/EN property shape and wire
member. It does not substitute a mocked property helper for this path.

Remote run: `/workspace/worker/lay-development-runner/ranker-release-20261004-r3-ef2jV3`.
`C09_PROBE_FMT1.json` records fmt PASS, the changed native transport test PASS
(24.028s), and the expected failure when only the old publication function is
temporarily restored (7.503s). The source was restored exactly after that
mutation. This is a controlled function mutation, not a complete historical
checkout comparison. The original format failure is retained separately.

That probe's unfiltered IME command records637 PASS/4 FAIL. All four first fail
at the same lexical prerequisite (`про` selects `просто` instead of
`проверка`), before the changed publication. Their canonical inventory requires
process isolation, which this diagnostic command bypassed. The exact global
cache/package contribution remains UNKNOWN; this FAIL receipt is not rewritten.
The saved source audit is
`/home/local-user/.cache/lay/development/ranker-release-20261004/revision3/TD121_ISOLATION_SOURCE_AUDIT.md`.

The unchanged R3 source then passed the existing hermetic focused runner:
638/638 correctness tests, no canonical inventory drift or failures, 63.871s
including discovery. Each disputed TD121 test and the changed native transport
test has its own exact-process PASS log. Proof:
`C09_CANONICAL_IME_EXECUTION.json` and `C09_CANONICAL_IME/SUMMARY.json` in the
remote run. Source manifest SHA256:
`48aea1e44019787a353ef42da5f7ca73b3e6e84a3ff27a15b75464dd311deb99`.
This proves the affected IME source scope, not a new installed runtime or public
release. At this entry, the installed IME remains `e64ccde4e7e495ac7ea4479c3f353fb4b64e54cc087be17ebe67daa31f33adac`.

The actual Kitty baseline passed5/6 executed groups, including two Tab cases
with exact post-Enter whitespace, held Shift, eight visible word/icon pairs
and the next physical letter, and Space layout conversion. The miss is the
unchanged `кторое` spelling case. Receipt:
`windows/WIDE_MATRIX_V2_20261004T063826.json` under the evidence root. Its runner
was frozen before launch. The earlier Firefox150-row receipt's runner was
edited during execution; its final harness hash therefore cannot bind the
executed observer. Preserve that receipt as diagnostic evidence and obtain a
fresh frozen-runner result before promotion.

Qt.Entry's first proven loss precedes Double Shift: Ctrl+A/Backspace clears
the client field while IME retains tail3; six later letters yield tail9 against
SurroundingText6. The gesture reaches its existing owner, and
`context_authority` correctly refuses before edit-plan/verifier/application.
The missing empty/selection receipt and exact admission subpredicate remain
UNKNOWN. Its held Shift trace decodes correctly but records preedit separately
from empty committed text; rendered acceptance is UNKNOWN. Audit:
`windows/QT_ENTRY_CAUSAL_AUDIT.md`. A separate Escape-first fixture setup can
isolate clean-field gestures but cannot certify the original Ctrl+A-delete path.

Runtime authority changed: no. New C09 bytes, live GNOME property acceptance,
all-window completion and native ranker quality remain pending. The preexisting
strict Clippy/lint failures still block full release acceptance.

## Installed C09 bytes and first physical repair receipt

Guarded graph refresh, architecture canon and format PASS; guarded ten-binary
release build PASS217.504s with no source drift. Packaging initially stopped
because the copied checkout's git-derived file list omitted two existing,
ignored historical receipt JSON files. The exact two filesystem hashes match
the1444-file pinned source; the final package manifest explicitly includes
them. The failure is retained as `revision3/C09_PACKAGE_ATTEMPT1.json`; no source
file, test or gate was removed. Independent review confirmed zero proof-scope
blockers. Fresh638 IME correctness results are combined with reuse of2313
unchanged non-IME results, explicitly not reported as a new full2951 run.

Installation PASS: `revision3/C09_INSTALL.json` under the release evidence root.
Installed/loaded IME SHA256:
`71ad8f2e5a2d72df813881c89a5a26085ce8d047a1df60d2f98597d26dbd69f6`.
Ten installed hashes and four loaded service hashes match the package. Global
IBus PID270775, config, input-source list and selected source are preserved;
GNOME extension was not reloaded or replaced. Exact current78 rollback is in
`revision3/C09_ROLLBACK_CURRENT_78.json`; the original77 rollback remains intact.
The authority and input protocol contracts are unchanged.

The first bound prospective physical replay passed Chrome contenteditable:
eight consecutive Double Shift pairs, each word and independent visible
GNOME RU/EN label before the next pair, then the correct next physical letter.
The group took3991ms including trace capture; the complete run7.88s. Scope is
that owned Chrome field on these exact bytes, not every window. Matrix:
`windows/V3_RUN_20261004T074054_8790fce49b72/WIDE_MATRIX_V3_20261004T074054_8790fce49b72.json`;
its adjacent `V3_BINDING.json` binds the immutable runner, imported helpers,
install receipt and matrix hashes before/after. Earlier failed Chrome evidence
is retained. The V3 selector rejects zero-match scopes and partial alternating
sequences, and skips unselected alternating setup before any keyboard event.
No selected assertion is weakened.

Other physical fields, original active-preedit Ctrl+A-delete, strict lint,
resources and native ranker quality remain outside this first PASS scope.

## Interrupted C09 broad run and observer identity defect

The next frozen V3 run, `windows/V3_RUN_20261004T074612_e3d9f5b99f43`,
recorded263 diagnostic rows:174 PASS,5 FAIL,84 BLOCKED. Its browser close
raised a focus error again from the outer cleanup; no final matrix was written
and `V3_BINDING.json` correctly records REJECT_BINDING. These counts are not
an accepted all-window result. The actual runtime remains the installed C09
IME `71ad8f2e5a2d72df813881c89a5a26085ce8d047a1df60d2f98597d26dbd69f6`.

First shared observer defect: the localhost server keys state only by
client/field kind and accepts every changed page_id. Passive metadata proves
two old/background Firefox pages alternating ownership of that alias within
1.6s. The constructor pins an alias snapshot, while later focus checks reject
another page_id. Chrome editable first blocks before Escape; Firefox input
reads an old page at its first block. The exact rejected focus predicate and
constructor-pinned identity were not saved and remain UNKNOWN; alias collision
is not retroactively asserted as the cause of every missed product assertion.
Audit: `windows/BROWSER_PAGE_IDENTITY_AUDIT.md`, SHA256
`25db664ae8340ede3c6ffa8d17f2dbf494a50e46e580902f787ceed1a1b2293d`.

Prospective repair belongs to the owned test observer: unique launch UUID,
separate per-page state, pinned fresh snapshot and nonthrowing field close.
Old helpers and receipts remain immutable. No Lay route, text admission,
application preference or product assertion changes for this repair.

Explicit cleanup then PASS: `revision3/C09_ABORT_TEST_DEVICE_CLEANUP.json`.
The departed test keyboard is absent; the existing managed daemon was restarted
once to complete teardown. IME PID3296914/loaded hash, global IBus PID270775,
input sources and the prior Lay RU source are preserved. No IME/global-IBus
restart or version rollback occurred. Runtime authority changed: no.

## Bound C09 native and terminal physical matrix

Frozen runner run `windows/V3_RUN_20261004T081804_4cd0996bdfee` completed in
317.670s with BOUND source/install/matrix hashes:463 PASS/476 executed,
13 FAIL,20 BLOCKED. Its `WIDE_MATRIX_V3_20261004T081804_4cd0996bdfee.json`
preserves every row; global IBus PID270775 is preserved and the physical test
keyboard is absent after cleanup. Exact installed IME remains `71ad8f2e…dbd69f6`.

All eight owned fields (GTK3 Entry/multiline, GTK4 Entry/multiline, Qt
Entry/multiline/rich, Kitty readline) pass eight visible word/icon pairs and
the correct next physical letter:64 intermediate manual transitions. All pass
first/second-word Tab with one space and held Shift. Kitty Tab/manual results
add exact physical Enter/readline submission equality. Qt held Shift uses the
explicit physical Escape-to-commit observer; this is not evidence for the
original active-preedit Ctrl+A/Backspace sequence. The20 blocked Kitty rows
cannot independently observe external IME hint absence.

Failures by shared mechanism: eight unchanged spelling misses preserve
`кторое `; two alternating sequences show correct visible text but the public
mode stays opposite the requested next-letter mode (GTK3 Entry and Kitty).
Three setup activations do not settle (Qt rich twice, Kitty once). These last
five are C09 synchronization observations, not established private decoder
or ranker failures. No diagnostic next letter was typed for those failed
Space rows. The saved traces must establish the first failed publication or
activation transition before a source repair. Selected source, IBus engine,
public InputMode and actual next physical decoding remain distinct evidence.

Runtime authority changed: no. Whole-matrix verdict FAIL_OR_BLOCKED; the64
passing manual transitions do not certify all windows, all gestures or a
public release. Existing strict Clippy gate remains FAIL.

## Pinned browser observer and C09 browser core result

The separately reviewed V4 observer gives every owned launch a UUID and every
GET a server-issued page UUID. Per-page sequence/freshness, exact body/query
identity and PID/starttick/title are checked before physical key-down. The
foreground alias serves first binding only; all later reads use the pinned
per-page file. Close errors cannot skip UInput teardown, managed daemon detach,
partial matrix or binding output. No Lay input route, application preference,
assertion or existing helper changed. Server/helper SHA256:
`cc0bdf3e0fcc989b6cb1b844868fe51b97562168d710518eea6acc8882e7c705` /
`b729a413d64ca79fc44607c48d0a4ee42571ccf7ae0cf6e0c70a3d1b9abfa056`.
The old own server was stopped after the completed native run; shared browsers
were not terminated. This source/AST review alone did not prove acceptance.

Actual run `windows/V4_RUN_20261004T082916_2ad7a5df3262` then completed BOUND,
21 PASS/24 executed/3 FAIL/0 BLOCKED,76.251s, no fatal cleanup error. The adjacent
`V4_BINDING.json` pins nine helpers, the executed runner, installation and final
matrix. It ran the sealed `wide_matrix_v4.py`, byte-identical to the prepared
frozen runner copy; the copy was not the execution path for this first run.
All three Chrome and all three Firefox fields pass held Shift and eight visible
word/icon pairs plus the next physical letter. Both Tab cases pass in Chrome;
Firefox first-word Tab passes. In all three Firefox fields second-word Tab
preserves active `почему` preedit at caret6 in `тест почему`, without the
required committed space. These are newly bound physical failures, no longer
alias-only diagnostic counts. Exact first failing admission/publication step
requires the saved per-case traces; no ranker blame or verifier weakening is
inferred. Broader/browser-negative, raw selection, other available application
fields and all-window acceptance remain pending. Runtime authority changed: no.


## C09 owned Gost, Tor and GNOME core observations

Bound run `windows/V4_RUN_20261004T083236_32bcb5458b3c` records 6 PASS/8
executed/2 FAIL/4 BLOCKED in 94.694s. Gost textarea and contenteditable pass
both Tab cases and held Shift. Their first Double Shift pair fails: the word
stays `ghbdtn`, and physical KEY_F confirms US decoding. Input setup and all
three Tor fields fail fresh owned-page binding before input; these BLOCKED
rows do not establish product failure. No browser preference/proxy changed.
The finite C03 audit proves stale committed tail survives client selection
and deletion; live exact WordInputMode and native empty-selection admission
remain UNKNOWN. Audit SHA `ba6fdd911b3990b38dcdffb9ee71c2c6897bd21464b0bb9cec8919be28bfeefd`.

Bound `windows/V4_RUN_20261004T083458_970bb85e1ca3` records 1 PASS/4
executed/3 FAIL/1 BLOCKED in 31.331s. The existing GNOME Text Editor inserts
literal Tab (`п\t`, `тест п\t`), passes held Shift, and fails first manual
pair. That proves visible behavior, not the cause or that IBus received those
keys. GNOME Terminal setup reports failure to open a PTY peer (too many open
files); no test text entered. Its server FD count1581, RLIMIT_NOFILE524288,
and kernel PTY nr19 do not establish a simple process limit cause. No limits
or working application settings changed.

Prospective standalone GTK automatic-vs-forced-IBus control is observer
BLOCKED: `windows/V5_RUN_20261004T085353_ec26db88223d`, 0 executed/2 BLOCKED,
25.686s, owned focused editable Text missing/ambiguous. Both own standalone
processes were closed. No key event reached either test field, so this is
neither IBus integration proof nor a successful repair. The v1 helper SHA is
`92e470d907ad5b0693acb2639ec97f58d2b5b29eb0008f2598a31062c68960e7`.

All three adjacent binding receipts are BOUND, fatal=null, UInput teardown,
managed-route detach and source restoration completed; global IBus PID270775
and installed C09 IME bytes are preserved. WPS, Telegram, WeChat and other
unexecuted clients remain availability only. Whole-window acceptance is not
established. Runtime authority changed: false.

## Retained Clippy and C03 format failures; prospective source cleanup

R2 `CANDIDATE3.log` reports five style lints already present at accepted77:
`redundant_pattern_matching` and `needless_option_as_deref` in
`window_interaction/observation.rs`, plus three `unnecessary_mut_passed`
references in `context_admission/adapter/tests/residuals.rs`. The original
command uses `-D warnings -A dead-code`; dead-code is governed by a separate
baseline check, which also failed and remains a separate release blocker.
The five style lints are not physical input or ranker failure evidence.

Source cleanup replaces only those operations; actual Timer poll result is
Instant (no Drop), polling/owner destruction order remains unchanged, and the
last Option<&mut EngineOutput> reference is moved after its earlier reborrow
ends. Three test calls use shared references. No new allow/waiver was added.
Clippy rerun has not yet established PASS.

The first C03 remote source probe stopped at fmt before any functional test:
`revision4/remote_c03_probe.py`, remote
`ranker-release-20261004-r4-nmoimizp/C03_PROBE_EXECUTION.json`. The exact
1445-file snapshot remained unchanged. Eight rustfmt layout diffs affected
only the two new tests. A separate clone `ranker-release-20261004-r4c-qayjcpz5`
retains an initial resource-wrapper CLI rejection (`FORMAT_0.log`, no source
mutation) and a corrected env-selected dedicated-20cpu formatter
(`FORMAT_1.log`), 2.758s PASS. Formatting touched only residuals.rs; its
formatted SHA is `8268d3e5ed75efb1f625f4f777ff143cc0807f6b464830c7fc8a69bf07f786ad`.
No tests, runtime installation or physical acceptance follow from fmt alone.
Original failures remain immutable. Runtime authority changed: false.


## Measured five-lint cleanup and discovered C03 focused proof

Dedicated-20cpu `ranker-release-20261004-r4c-qayjcpz5` verifies all1445
source hashes against manifest `d633dbd2866d68ef778c1633d6ce2c0e7453c282077a45e16c088142d17d5fac`.
Both original style-policy Clippy commands now PASS: default20.660s and
research-tools21.523s. Flags remain `-D warnings -A dead-code`, matching the
existing split policy; no extra suppression was introduced. This resolves
the five style lints, not the independent dead-code baseline failure or
physical release acceptance. Fmt check also PASS2.732s.

The first focused call used an incorrect module prefix and executed zero
of643 discovered tests despite process exit0. Its nonzero execution guard
correctly returned FAIL, retained in `STYLE_C03_EXECUTION.json`; it is not
a functional PASS. Actual discovery identifies the missing `word_scope`
module. The three exact discovered callback tests then each execute1 and
PASS (0.248/0.221/0.238s), recorded in `C03_FOCUSED_V2_EXECUTION.json` and
`C03_DISCOVERY.json`. No source changed during either worker. Controlled
original-guard mutation and canonical full affected IME lane remain pending.
Test proof does not certify live Gost guard eligibility or physical repair.
Runtime bytes/authority changed: false.

## Standalone GNOME owned observer V2: active-window blocker isolated

`windows/V6_RUN_20261004T092312_898f53dd01bd` is BOUND, 0 executed/2 BLOCKED,
25.526s, fatal=null, teardown/detach/source restoration completed, unchanged
C09 IME and global IBus PID270775. The separately frozen V6 runner SHA is
`440ac0721dca850cc8a9a983f583a2b57818419ab0b69e68da40fdfaacaed44a`,11
helper pins. New standalone observer V2 SHA is
`821b9d2aff1b94f75fde5d90d895196f373074cb4bf535255316f55e92da80bf`.

First bounded discovery reports no text reads/nodes and zero GrabFocus calls:
`DISCOVERY_FIRST.json` in `/run/user/1000/lay78-gnome.bbruxz_d` and
`/run/user/1000/lay78-gnome.k1tj84p2` fail `exact owned window is not the active
window`. Later constructor retries exhaust the discovery budget and are
not the first cause. Thus this attempt isolates missing owned-window
activation before node discovery; it does not prove missing Text capability
or evaluate automatic versus forced IBus. No test key was sent. Subsequent
observer work must preserve exact owner checks before activation and actual
active/editable focus proof before keys. Original V1/V2 receipts stay intact.
Runtime authority changed: false.


## Standalone GNOME activation V3 and C06 audit scope

Root `windows/V7_RUN_20261004T093340_a179b1a36887` completes BOUND,0 executed/
2 BLOCKED in25.819s, fatal=null, unchanged C09/global IBus with full teardown.
Frozen runner SHA `3f5065f8adac4473ec0b98e69e079e97fb102dd162b020dfac6f771d0ba8e732`
pins12 helpers; new V3 activation helper SHA
`86d8746b17abf92f7863bd825a348789e85f16a06c5cfc9d5f89a145c1672805`.
Owner PID/starttick/exe/UUID-window/file are verified before the single window
Component.GrabFocus request, and original active-window guard remains mandatory
before Text discovery or keys. FIRST metadata under own dirs
`/run/user/1000/lay78-gnome.63rfhkeg` and `lay78-gnome.n789x9j2` records
own ACTIVE=false, shell own PID=true, own title=false. The request raises
AT_SPI_OR_PROCESS_ERROR with reply null, so it does not prove compositor
activation. No automatic-vs-forced-IBus comparison executes, and no test key
is sent. Original failures remain retained. No working-app setting changes.

The finite C06 audit of bound Firefox traces establishes shared first-/second-
word route divergence across all three fields. Owned preedit is shown but
append admission excludes the exact-refresh capability that the starter allows
at an observed boundary. The precise internal boolean/return reason remains
UNKNOWN/reconstructed. See the matching stabilization entry and
`windows/FIREFOX_SECOND_WORD_TAB_CAUSAL_AUDIT.md`, SHA
`36a7a47904ec4b0d9a115b7e5b85f2a75604fef08c9a8a2215233317bfd749c9`.
No C06 source repair, model fit, runtime installation or authority change.


## Finite C03 original-guard witness and actual79 canonical closure

The source regression detects the original SurroundingText-only predicate:
remote `ranker-release-20261004-c03-mutation-8eofbpl3` executes1, returns101
at residuals.rs8189 `engine.committed_tail.buffer.is_empty()`. Managed bytes
restore exactly to `925d5ac19adab8037e781049c1d8604d3c431f76ebfab49fbfe0201d8e241d5a`.
The first worker remains FAIL: its parser incorrectly required the test name
and FAILED on one line although nocapture inserts panic lines. No old-guard
mutation rerun is substituted. Fresh continuation binds the saved exact witness
and restored regression actually PASS1,23.575s.

Actual manifest generation PASS2979 identities, preserving all2977 prior rows
and adding only the two C03 cases. Knownfailure count remains0; only its
manifest SHA metadata binding changes. The first canonical continuation
`ranker-release-20261004-c03-canonical-tvg1146n` remains BLOCKED before any
canonical test because Git/index metadata were missing (source closure rc128).
A copy-only attempt also retains a mode-assumption rejection: graph.json
bytes match, but actual0664 differs from previously recorded0644; no source
chmod or test rerun follows from that preparation failure.

Earlier R3/R4C snapshot Git metadata have synthetic HEAD
`9a3091db29f0a0a6e6745c244da664079937b92a`, message identifying accepted79
through source manifest. Their Git root is the exact source directory; no
source/ prefix defect occurred. Source hash/build/physical receipts remain
bounded to their actual bytes, but this metadata is not the original accepted
Git commit and is not reused as actual79 governance comparison proof.

A fresh private continuation obtains actual accepted79 commit/tree/blob
objects using native Git pack (1commit/171trees/1446blobs,58,887,638B), copies
the original worktree HEAD/index unchanged, and excludes config/credentials/
other worktree metadata. Actual HEAD is
`79b09d06306a018961ace783e2542eb817de7b30`; Git root equals its exact source.
This is transport of existing objects, not a new commit or regenerated index.

`ranker-release-20261004-c03-canon79-3ppevcj2` then executes the one actual
canonical affected-IME proof:643 discovered/640 selected/640 executed/640
PASS/0 FAIL, drift MATCH;3 performance tests excluded by the existing lane.
It runs185 isolated processes and455 target tests, canonical62.986s including
37.909s execution; source and original R4C are unchanged. Actual donor/index/
pack binding and before1445-file manifest are recorded separately. Dedicated
20cpu guard terminates; no remaining worker descendants. This certifies the
affected IME source lane, not a full release, physical repair, ranker quality
promotion, dead-code baseline or newly installed bytes. Mandatory graph/canon
refresh of the latest owning-document snapshot follows separately.
Runtime bytes/authority changed: false.


## Latest exact source graph/canon milestone; installation still separate

Dedicated-20cpu remote `ranker-release-20261004-c03-graph-8n3b9t26` validates
all1446 local source file hashes against `e164d425…`. Its exact Git root/HEAD
is the actual accepted79 source plus dirty snapshot. `update-architecture-graph.sh`
performs the real AST-only update, pruning, binding, receipt and architecture
checks: PASS44.801s; protected-change canon against actual79 PASS0.979s.
No source/test rule was weakened, API/model request or Cargo build was added.
Only six generated graph/receipt files changed. Exported graph files are
individually SHA verified and imported exactly into the authoritative release
checkout. Receipts `revision4/graph-final/GRAPH_EXECUTION.json` and
`LOCAL_GRAPH_IMPORT.json` distinguish architecture proof from quality/runtime.

All code changes now have owning entries and explicit decisions, including
`2026-10-04-ime-clippy-style-cleanup.json`. Five inherited style warnings are
resolved in source and both original Clippy style-policy commands PASS. The
C03 original-predicate witness/restored regression and640 affected-IME closure
are source evidence. Independent dead-code baseline, fixed heldout quality,
changed-byte packaging/build/install and all-window physical acceptance remain
separate. Current installed C09 IME remains `71ad8f2e…dbd69f6`.

Next runtime work: exact-build/bind the C03 candidate before any installation;
prove live conditional eligibility and Gost/Qt selection repair on new bytes;
resolve Firefox second-word Tab using the existing append authority contract;
finish C09 Space activation/publication evidence and GNOME/Tor owned setup.
No new model fit or agreement research resumed. The own V4 HTTP fixture server
is terminated after PID/starttick/startup argv/cwd/source SHA validation;
`windows/BROWSER_SERVER_V4_STOP.json`, exec exit143. Shared applications and
global IBus are not terminated. No keyboard driver/server/test worker remains
from this finite milestone. Runtime authority changed: false.


## C03 candidate installed and raw-command physical result — 2026-10-04T09:50:20.929980+00:00

Exact source1446 manifest `079852a3…` and actual79 metadata build only
`lay-ibus-engine --locked --release --features research-tools`; release build
PASS195.144s, source unchanged. Existing640 affected-IME source tests reused
with production/dependency identity; graph receipt metadata delta recorded
separately. Independent baseline/full release gates remain blocked.

User-authorized single-IME transaction `revision4/c03-install/INSTALL.json`
PASS installs/loads `a1bdffe17771e241234b6bf130babde052e4bac31578741c0d7de780df6ebaee`.
Exact C09 rollback saved. Daemon/L3/L11/global IBus PIDs/bytes and input sources,
config and extension preserved. No input protocol/authority policy change.

Physical raw Ctrl+A/Backspace core: Gost textarea and Qt entry/multiline/rich
16/16 executed PASS, 0 functional FAIL; all32 intermediate Double Shift
transitions show word/native GNOME icon before next pair, correct next letter,
first/second Tab exactly one space, held Shift. Qt held-Shift proof retains
explicit Escape settlement, not every active-preedit shortcut scenario.
Gost contenteditable setup BLOCKED before input by OWNER_BIND_TIMEOUT; this
is not physical PASS. Exact receipt `/home/local-user/.cache/lay/development/ranker-release-20261004/windows/V8_RUN_20261004T124423_bb1e553d2a8d/WIDE_MATRIX_V7_20261004T124423_bb1e553d2a8d.json`, binding BOUND;
unconditional own device teardown and global IBus preservation confirmed.
Live direct guard-entry tuple is still not exported; physical repair plus
source/callback proofs establish this tested sequence, not all client states.
Firefox second-word Tab/C09 Space/default Wayland GNOME/Tor/other applications
remain separate acceptance work. Runtime bytes changed:true; model research
remains paused.


## Additional C03 physical/setup evidence 2026-10-04T10:02:45.851519+00:00

Installed byte identity remains `a1bdffe…baee`. These results do not promote all-window acceptance.
- `/home/local-user/.cache/lay/development/ranker-release-20261004/windows/V9_RUN_20261004T124859_73b2e1520ed5/WIDE_MATRIX_V7_20261004T124859_73b2e1520ed5.json`: 0/0 PASS; 8 BLOCKED; BOUND, own teardown/global IBus preserved.
- `/home/local-user/.cache/lay/development/ranker-release-20261004/windows/V9_RUN_20261004T125101_558d61b342d8/WIDE_MATRIX_V7_20261004T125101_558d61b342d8.json`: 2/8 PASS; 0 BLOCKED; BOUND, own teardown/global IBus preserved.
- `/home/local-user/.cache/lay/development/ranker-release-20261004/windows/V10_RUN_20261004T125525_943d930b82f5/WIDE_MATRIX_V7_20261004T125525_943d930b82f5.json`: 0/0 PASS; 1 BLOCKED; BOUND, own teardown/global IBus preserved.
- `/home/local-user/.cache/lay/development/ranker-release-20261004/windows/V10_RUN_20261004T125719_5b9235682f7f/WIDE_MATRIX_V7_20261004T125719_5b9235682f7f.json`: 32/35 PASS; 0 BLOCKED; BOUND, own teardown/global IBus preserved.

GNOME Text Editor fresh standalone X11 automatic/forced IBus: exact own-XID activation succeeds. Escape-clean setup cannot observe preedit/composition, so8cases BLOCKED before keys. Separate raw run firstTab PASS both fields; next3cases each fail already at Ctrl+A/Backspace setup: previous `почему ` loses one character, not entire field. Their Tab/held/DoubleShift actions were not reached. This is a shortcut/clear first-transition audit, not three independent functional regressions. Default Wayland is NOT_TESTED by X11 evidence.
Gedit standalone X11 owner-tree discovery exceeds its bound before keyboard input:0executed/1BLOCKED. No partial tree accepted or user document touched.
Space capture with whole20 requested for3/4-letter alternation: GTK3Entry passes all20four-letter words, 3letterword11 (`sun`) remains `ыгт `; Kitty text replacement succeeds at3letterword3 and4letterword1 but public InputMode remainsRU with source/engineUS.35executed/32PASS/3FAIL; early failures stop those sequences, so incomplete20sets do not PASS. Private decoder at Kitty failure UNKNOWN (no additional key).
Metadata-only capture `C03_SPACE_WIRE_20261004_A/CAPTURE_RECEIPT.json` observed115native RegisterProperties/36GlobalEngineChanged and zero UpdateProperty; forwarding/private-peer coverage not implied. Root stopped exact owned helper after the sequence. Captured mode/path/order is being joined with exact failure traces; no C09 patch or route change made. Runtime authority policy unchanged.


## C03 fresh GNOME/manual and browser negative proof 2026-10-04T10:15:48.892466+00:00

`/home/local-user/.cache/lay/development/ranker-release-20261004/windows/V10_RUN_20261004T125951_4e2ac0f46807/WIDE_MATRIX_V7_20261004T125951_4e2ac0f46807.json`: fresh separate GNOME X11 automatic/forced IBus fields, only Double Shift case requested to avoid earlier clear cascade.0/2PASS. Pair1 AT-SPI text reports `gпривет`, RU native icon/source/engine correct; diagnostic physical nextKEY_F reports `gпривета`. Rendered pixels and committed/preedit separation remain UNKNOWN; this is a current accessible-text failure, not a proven committed-duplicate cause. Exact raw traces are under that receipt; source ownership/commit/preedit path audit follows. Own app/device cleanup and global IBus preservation confirmed.

`/home/local-user/.cache/lay/development/ranker-release-20261004/windows/V10_RUN_20261004T130939_2391832ad083/WIDE_MATRIX_V7_20261004T130939_2391832ad083.json`: browser forbidden-prefix proof in Chrome/Firefox input/textarea/contenteditable119PASS/120attempted,0BLOCKED. One reported FAIL at Chrome input after-word `@g` occurs already in initial mode settlement, before that token is typed; not a observed forbidden hint. Other119 checks preserve expected text and no observable IME preedit for digits/special prefixes in first/later words. No all120PASS claim or all-window promotion. Installed bytes remain C03 `a1bdffe…baee`. Runtime policy/authority unchanged; no model fit.


## C06 observed-start append proof imported; NOT_INSTALLED — 2026-10-04T10:17:23.595665+00:00

Six exact formatted source paths imported after root base/final SHA checks from
`c06-owned-preedit-boundary/final/C06_FINAL_CODE_MANIFEST.json`. Focused11/11
actual legacy/P2P tests PASS; both controlled original exact-refresh predicate
and omitted local Backspace receipt FAIL precisely at actual Tab; both restored
positives PASS. Actual accepted79 HEAD/index/pack metadata preserved.
`C06_COMPACT_PROOF.json`/execution83ff3e6a distinguish this focused proof from
canonical/full/physical acceptance. Initial absent-witness source attempt and
infrastructure prelaunch rejects are retained. No weakened assertion/manifest.

The existing CompositionState holds an existing ManagedWordStartWitness captured
from an exact zero-current-token start snapshot BEFORE first owned preedit key;
old managed witness construction remains unchanged. Tab only extends this
current wholly-owned uncommitted word, with original live candidate/token/
selection/revision/owner/backend verifier checks. Exact-refresh no-boundary
refusal remains. Nonempty end-caret local Backspace may reanchor only after an
exact single local deletion receipt; client/selection/epoch/owner/layout drift
and whole-word erase revoke. No additional controller/cache/timer/fallback or
manual/mutation grant. Two explicit decisions record both scope changes.

Canonical affected-IME plus graph/canon and release-build proof still required
before changed-byte physical testing. C09 isolated metadata publication repair
will be integrated only after its own regression proof; one combined build
avoids duplicate compile/closure. Live IME stays C03a1bdffe…baee, no runtime
bytes or authority policy changed by this source import. Research paused.


## C09 focused proof failure and bounded fixture replanning — 2026-10-04

Actual continuation `/workspace/worker/lay-development-runner/ranker-release-20261004-c09-mode-continuation-O3I6VHDV/C09_FOCUSED_EXECUTION.json`: fmt PASS, 652 discovered, two actual native release callback positives PASS. The third negative fixture panicked at `context_owner.unwrap()` before its stale-owner assertion; fourth was not executed. `start_source_free_unknown` prepares but does not install engine authority. This is a fixture preparation failure, not a passing negative proof. Earlier temporary-borrow compiler failure is retained; no further unreported repair pass.

Explicit replan: retain the unchanged bounded C09 production proposal, use the existing activation-install path to establish an asserted live owner before negative corruption, and establish installed pending publication without draining it before transport-failure injection. Compare preparation through an actual release callback with the direct existing installation path; choose the latter for the transport-failure fixture to avoid falsely consuming the pending flag. Native SourceFree/Transfer positive fixtures retain actual production callback and output ordering. No new reducer/owner/callback/timer or bypass is added; all stale-owner and failed-transport assertions remain required. Controlled removal of the publication must fail a positive fixture. Four nonzero focused positives, existing canonical closure, original style policy and fresh graph/canon precede one combined IME build.

Consequences: this new variant changes only fixture preparation, does not change candidates/ranking/SafetyGate/text authority/decoder/deadlines/cache identity/packages/learning/concurrency or runtime CPU/RSS. A bad preparation could hide the real refusal, hence explicit installed-owner/token/pending assertions and exact callback positives are mandatory. Current installed IME remains C03 `a1bdffe17771e241234b6bf130babde052e4bac31578741c0d7de780df6ebaee`; no runtime authority changed. Owner: h7_full_pool isolated source/focused worker; root imports and installs only after proof. Canonical/build lease remains blocked until this variant passes.


## C09 source import and explicit native-output fixture adaptation — 2026-10-04

Imported the exact three files listed in `c09-post-install-mode-patch/combined-c06/C09_CODE_HANDOFF.json`, against all base hashes. Focused execution `focused_replan_pass/C09_FOCUSED_EXECUTION.json`: 652 discovered, four exact actual tests PASS, controlled missing-callback publication fails, restored source PASS in43.303s. SourceFree/Transfer native release publication, revoked/mismatched-owner refusal and failed-transport authority/pending preservation are separately covered. C09 metadata remains a causal treatment for native InputMode publication; GNOME private forwarding loss is UNKNOWN. It changes no input gesture detector, layout initiator, text executor, candidate/ranking authority, or model package. Current installed C03 bytes remain unchanged.

Before the combined gate, adapt existing test output expectations explicitly under `2026-10-04-native-input-mode-test-output-sequence.json`. Measured inventory:38 static first-CommitText helper calls, plus direct native-transfer/preedit-aware readers and a Shift→RPC marker fixture. Chosen design validates at most one expected standard InputMode metadata message with exact engine path/interface/schema/key/type/state/label+symbol and installed owner before requiring the unchanged next CommitText. Actual publication must precede text; arbitrary signals, duplicates and malformed/mismatched property still fail. Existing unfiltered legacy_effects and no-output/array assertions remain real. The Shift marker fixture consumes only this explicitly expected signal before reading its RPC. A prior preedit-aware reader retains its preedit semantics while explicitly validating InputMode.

Consequence analysis: fixture adaptation only; production candidates, scores, Keep/Edit/SafetyGate, key decoding, latency/CPU/RSS, cache/package identities, learning, concurrency, stale-result races and rollback behavior remain unchanged. A global signal filter would hide authority/output defects and is rejected. Manually clearing pending or prepublishing outside actual callbacks would conceal the activation behavior and is rejected. Validate current metadata identity using the installed engine and keep the4 native callback regressions mandatory. One frozen combined source then gets the original style policy, canonical nonzero IME proof, required graph/canon, and onlyIME release build. Root owns installation and changed-byte physical acceptance; source PASS is not client PASS.


## Physical scope additions and GNOME replay first-loss analysis — 2026-10-04

Fresh Writer attempt on current C03 `windows/V11_RUN_20261004T134616_140ac2a8f1f5/WIDE_MATRIX_V7_20261004T134616_140ac2a8f1f5.json`: 0 executed,1setup BLOCKED,16.746s, no user text/input. The own temporary UserInstallation helper pinned an early native PID before an owned AT-SPI window existed, then rejected changed PID/starttick. This is observer setup, not a functional Writer result; exact bootstrap lifecycle UNKNOWN. Agent read-only process check found zero remaining processes with the exact own profile argv. A prospective helper will bind only the native candidate proven by the actual UUID window, with no post-binding rebind.

Saved C03 GNOME V10 traces prove the first replay consistency loss: auto owner128, PRESS4005/4006, epoch778 and tail6→5→4, then Reset4007 at epoch780 while client length/cursor/anchor5/5/5. Forcedibus owner133 similarly PRESS4201/4202, epoch811→813, Reset4203, client5/5/5. Both callbacks are exact_replay_native handled=false; replay later rejects a Backspace while a client character remains before insertion. Origin of the two PRESS callbacks is UNKNOWN because raw state/sender/native event ID/monotonic and daemon output counts were not captured. Do not deduplicate by keycode/time, weaken the exact replay lease, or merely permit rejected Backspace. Root needs a single owned-window causal metadata probe. Rendered pixels/committed-preedit separation remain UNKNOWN. V9 clear independently used RU keyval1734 for Ctrl+physicalA with no recorded selection, and Backspace shortened7→6; observe exact selections before attributing selection/deletion failure.

Ranker boundary is unchanged: `surface_authority_admissions` precedes `agreement_order::select`; retained-exact selection follows and retains precedence. No ranker detector/layout initiator/output executor was added. Whether observed Tab/replay defects existed in77 or regressed in78 has not been established by an exact old-byte physical comparison. Runtime authority policy unchanged; no deployment or quality promotion in this entry.


## User live Firefox Tab observation — 2026-10-04

User reports «сейчас в firefox работает tab» on current C03 installed IME. Receipt `windows/USER_FIREFOX_TAB_WORKING_20261004.json` stores report/time/current installed SHA separately from synthetic proof. Exact current field and prior focus/Reset/typing sequence are not established. This confirms a working user-field observation; the separately captured refusal and offline C06 regressions remain state-dependent, not a universal Firefox Tab failure. No keys, deployment, full-release acceptance or runtime authority change in this observation.


## C09 explicit fixture sequence imported before combined freeze — 2026-10-04

Hash-checked and imported the final remote-formatted four test files from `c09-test-output-sequence/C09_TEST_OUTPUT_SEQUENCE.json`. Root reviewed max-one exact InputMode signal validation, path/interface/type/state/label+symbol and current installed-owner/token/shared checks.38 helper sites capture the expected mode before the actual key callback. Two first-Shift preparations consume and validate the exact expected property before their RPC/fence read; legacy_effects/no-output/count collectors remain unfiltered. The pre-existing preedit loop is unchanged apart from explicit metadata validation. No production delta, new owner, input detector, layout initiator, edit authority or runtime deployment in this variant. Combined executable gate/build remains required; no source PASS claimed for these new fixture bytes yet.

Latest user clarifies the live Firefox failure trigger: ordinary Tab works now, while Backspace→retype→Tab does not («Backspace убивает tab»). `windows/USER_BACKSPACE_RETYPE_TAB_FAIL_20261004.json` is user observation, not synthetic acceptance. Installed IME remains current C03 and lacks the separately proved C06 boundary-preservation repair. First physical scenario after combined installation will assert this exact trigger, separately from ordinary first/second-word Tab.


## Combined source gate first pass: style failure before canonical/build — 2026-10-04

Actual frozen combined1458-file source stage `/workspace/worker/lay-development-runner/ranker-release-20261004-combined-c06-c09-0j6crwhh/PIPELINE_EXECUTION.json`: canonical inventory discovery PASS (all old rows literal,9 added), default strict original style Clippy FAIL at five concrete sites; no canonical correctness test or release build executed. Total53.91s, discovery29.95s. Findings: now-unused ObservationReceipt re-export after C09 consumer replacement; three C06 Option early-return let-else sites; one unnecessary &source argument in a borrowed-engine test helper. No runtime bytes/authority changed.

First mechanical repair pass is permitted: make the obsolete re-export test-only if still required by tests, replace the exact None-return sites with equivalent Option ?, and remove only the redundant test reference. No lint waiver or suppression is added, no semantic assertion weakened, and no owner/token/lease/display/decoder/ranker behavior changes. Candidate retention/ranks/false authority/deadlines/caches/packages/learning/concurrency/failure rollback remain semantically identical. Rebind exact changed bytes plus owning docs, continue the original two style commands then full selected canonical IME, mandatory graph/canon and onlyIME build. Reuse inventory discovery only if names/manifest semantics remain unchanged. No silent repair loop or zero-test PASS.


## First combined style-only repair imported — 2026-10-04

Hash-checked three formatted files against `combined-c06-c09-pipeline/style-fix/STYLE_ONLY.json`; diff05272905…a4440. Five lint-site edits only: ObservationReceipt export retained under cfg(test), three equivalent Option None early-returns written with ?, and one redundant borrowed-source reference removed. Current installed C03 bytes remain unchanged. Original style commands, canonical nonzero correctness, graph/canon and exact build remain pending for these bytes. No lint waiver, authority or behavioral change.

Additional owned observers: V12 Writer bound exact own nativePID/window after an actually measured startup PID change, then generic full-window text discovery exceeded its bound (0keys/0tests). Four exact own-session Glycin loader children were individually closed through freshly revalidated pidfds; `windows/LO_WRITER_V12_ROOT_HELPER_CLEANUP.json` PASS, own profile removed, no live process remains in that owned session. V14 GNOME selection attempt blocked before any key because exact child X11 environment was not proved;0functional tests. Neither blocked setup is an IME PASS or FAIL. Root prospective V13 six browser Backspace cases are ready but NOT_RUN.


## Combined canonical failure and explicit activation-fixture replan — 2026-10-04

Continuation `combined-c06-c09-pipeline/continuation/failure/FAILURE_PACKET.json` (SHA256 `c37d277e849f8befd9c5177c732d4dc94df5f5575a705b971bfbd967425b3df5`): format and both original Clippy configurations PASS; canonical 649 selected/executed,628PASS,21FAIL, all9new C06/C09 tests PASS. Four failing consumer categories:11Ping markers,5preedit allow lists,2strict proof drains,3strict empty legacy-effect assertions. Graph/canon/build were not run; failed evidence remains immutable. Signals without printed bodies are not yet proven to be the expected metadata.

Explicit replan recorded before further test edits in `decisions/2026-10-04-native-input-mode-activation-fixture-replan.json`: validate exactly one expected standard InputMode at the proven actual authenticated activation preparation boundary, then retain the original text/preedit/Ping/no-output assertion. First-iteration-only consumption for looped activation fixtures. Generic marker/effect/drain/count collectors stay strict and unfiltered. Compare with global filtering and manual pending clearing: both rejected because they conceal order, duplicate or authority violations. Production code, test identities and SafetyGate are unchanged. Owner h7_full_pool isolated fixture variant; root review/import and sole installer/GUI owner. First run the exact nonzero21focused identities, then full649canonical plus original style, graph/canon and onlyIME build.

Consequences: fixture-only repair has no change to candidate retention, ranking/false authority, runtime latency/CPU/RSS, cache invalidation, package/reload identities, learning/feedback, concurrency/stale-result rules or rollback. The material risk is hiding output defects in test preparation; exact current owner/path/mode/property checks and unchanged downstream negative assertions bound it. No new owner, detector, layout initiator, executor, cache or fallback. Installed C03 `a1bdffe17771e241234b6bf130babde052e4bac31578741c0d7de780df6ebaee` remains unchanged and lacks the C06 Backspace repair.

Physical C03 baseline `windows/V13_RUN_20261004T141910_cc6d6365a9a5/WIDE_MATRIX_V7_20261004T141910_cc6d6365a9a5.json`: Firefox input Backspace→partial retype→Tab actually executed and FAIL (1executed,0PASS),6.735s. Its independent observed result must be read before attributing the exact refusal. No changed-byte success or broad acceptance inferred.


## Explicit activation fixture replan imported — 2026-10-04

Root reviewed and hash-checked three test files from `c09-activation-fixture-replan/FORMATTED_FIXTURE_HANDOFF.json` SHA12374ac06d646f517545f4f74b003343259e1f22d67bc4eb32cebada29307cc6. Actual nonzero21focused failures now21/21PASS in31.071s; executionSHA01c7198f3a823f50e7ab6d34111d04a8fe1771b136e7b0ea18ff33452e5384e8.15explicit activation boundaries consume exactly one fully validated expected InputMode before original assertions; first iteration only where looped. Generic collectors, existing validator and production source remain unchanged. Original canonical649/style closure required. Graph/build held pending the separate native-prefix Backspace coverage gap; no changed-byte installation or client acceptance.


## Native Backspace scope gap and causal rebind proposal — 2026-10-04

Independent review proves C06 owns only nonempty local composition/handled=true. Actual V13 owner369 uses client-committed prefix and virtual suffix; native Backspace18212 correctly hides composition, shortens mirror2→1 and returns false. Reset18213 rejects `missing_predecessor_or_post_reset_token`; the combined reason does not distinguish its operands. Rereceipt after exact surrounding and next letter remains unavailable. Tab was not sent. Nearby exact_backspace=false belongs to owner368 setup and is excluded. Current uninstalled owned-path11PASS is valid within its scope and does not repair this native sequence.

Before runtime edits, `decisions/2026-10-04-native-backspace-rereceipt-rebind.json` records the bounded hypothesis/design and scaled consequence analysis. Start with a production native callback regression of the saved order on current bytes. Existing advance_context_reset_rereceipt_after_key unconditionally retires prior confirmed pending state for native Backspace; investigate preserving only inert shortened provenance, using existing PendingContextResetRereceipt/reducer/snapshot checks. Confirmed exact old boundary, no selection/end-caret, one scalar/epoch and same owner/focus/layout/lineage are required. Never leave it confirmed after a client-owned edit. New exact surrounding receipt must prove the shortened prefix before Tab authority. No old hint reuse or native Backspace republishing; keep the verified hide→mirror→handled=false contract. Full erase revokes old provenance. No new owner/route/executor/gesture/layout initiator/queue/timer/fallback. Alternatives that reconstruct arbitrary text authority or retain confirmed old snapshots are rejected.

Consequences and required negative proof: existing candidates/rank/SafetyGate/package/learning remain unchanged; bounded receipt state only, no corpus work/waits. Primary risks are a client selection/caret/failed deletion, stale callback/receipt and duplicate display before native deletion. Test absence of hint/text output during native Backspace, stale/no client receipt, changed owner/focus/layout/epoch/selection, and partial/whole first/later-word recovery. Actual native old-code failure and restored PASS precede fixed canonical/style/mandatory graph/canon/onlyIME build. h7_full_pool remains connected implementation and remote-heavy owner; grammar_label_audit reviews; root alone imports/installs/tests physical fields. Live C03 unchanged; no new acceptance or runtime authority change in this entry.


## Native Backspace causal old-source regression — 2026-10-04

Root independently read remote `ranker-release-20261004-native-bsp-iaDvavnl/NATIVE_BASELINE_EXECUTION.json` and bounded `FOCUSED_1.log` SHA53483da8bf069f5283fcb2f97ab0c38e0cdd8d84a624eda56abbe21f4fe128c9. Existing guarded exact bin test selected/executed1, failed1 (rc101), compile succeeded in23.44s; total24.097s. Fixture proves confirmed old pending range2 with reset scope0; native Backspace(false) clears/hides once in exact order, shortens mirror and advances one epoch, but pending becomesNone. Next actual Reset leaves a live post-reset token (assertion passed), pending remainsNone, then exact shortened client snapshot cannot restore completion authority. Failure occurs at the required restoration assertion, not at preparation or an unrelated panic. This distinguishes missing predecessor from missing post-Reset token for this controlled sequence.

The wrapper's textual final-name assertion also rejected nocapture diagnostics interleaved before FAILED; its status remains FAIL and is not relabelled a passing test. Raw exact command, identity and nonzero1failed summary independently establish the old-code negative control. No redundant numerical rerun required. Source-proved mechanism matches the physical owner369 native-delete/reset/receipt gap; broader client behavior still awaits changed bytes. h7 implements the recorded inert receipt rebind and native Backspace refresh suppression; no global IBus/runtime/model change. Independent pre-code review PASS_WITH_LIMITS identified the false-BSP postkey display hazard and requires absence of nonempty preedit/Commit/Delete during that callback.


## Native receipt revision repair and exact-client confirmation scope — 2026-10-04

First private production proposal a99a56ab compiled, but exact1positive FAIL with pendingNone. Root/independent review found a functional guard error: capture compared old pending.armed_revision with current revision even though existing confirmation succeeds at arm+1 without rewriting arm. Remove this equality; existing capture records the current revision, and post-key candidate-revision equality still guards drift. This is not a passing repair receipt. A second explicit clarification `decisions/2026-10-04-native-backspace-exact-client-confirmation.json` records the interpretation and unchanged consequences.

Fresh exact same-owner client confirmation is mandatory for new acceptance. Another Reset is part of the measured Firefox order and tested; existing fresh SetSurroundingText verification may also confirm the inert carry without an extra Reset. The proposal does not add a Reset requirement or another route. Current conservative support is end-of-document, single scalar, shortened word nonempty, confirmed old exact snapshot/no selection and unchanged owner/focus/layout/lineage/epoch. Mid-document caret support remains unproved. New pending stays unconfirmed/publicationNone; nativefalseBackspace skips all postkeyprefetch/refresh and retains hide→mirror→client deletion. Complete erase/repeated unconfirmed/delete or identity drift cannot accumulate speculation. No candidate/rank/package/learning/deadline/resource/SafetyGate change; exact C03 rollback preserved. Source-focused/canonical/build/physical proof still required; current runtime unchanged.


## Native no-confirmation fixture replan — 2026-10-04

Actual suite native-bsp-suite-PVsXJgDP: first2focused identities PASS, including core native deletion and four first/later RU/EN delete/retype/receipt/Tab routes. Third identity correctly refused no-receipt Tab, emitted no text edit/learning, then failed an invalid expectation that a subsequent receipt could revive its provenance. Existing unhandled Tab is an unproven external input/navigation gap; scope is revoked. Remaining identities/control were not executed. Production c118fece remains unchanged.

Explicit test-only replan recorded before further fixture edits in `decisions/2026-10-04-native-backspace-negative-fixture-replan.json`: two independent preparations. Direct Backspace→fresh exact receipt confirms through the existing gate; Backspace→unconfirmed Tab refuses edit/learning and remains refused after a later receipt. Preserve all negative output/provenance assertions. Changing runtime to revive after that Tab is rejected because it would erase the input-gap safety contract. Fixture-only preparation has no candidate/rank/runtime resource/cache/package/learning/concurrency/rollback effects. Complete exact focused/controlled suppression then canonical/style/graph/build before root install. No live bytes or authority changed.


## Prospective GNOME observer setup proved, no keys — 2026-10-04

Root constructed only an owned empty GNOME auto-input editor through frozen gnome_standalone_fields_v5 SHA4e8d9a061d944e91074125877d73b812d33cfcd8f4c46237a63ce035108e401c, then closed it. SAME nativePID1766808/start273696908, UUID38dad5dd82eb4006b92ee079575bd8ba, early and final GDK_BACKEND=[x11]/GTK_IM_MODULE=[] were independently observed. Unique UUID window/file/XID and final environment proved before any input;0keys/0functional cases. Durable own receipts `windows/GNOME_V5_SETUP_NO_KEYS_20261004T120715/ROOT_SETUP_RECEIPT.json`. Earlier V14 startup cause remainsUNKNOWN and BLOCKED scope unchanged; this establishes usable observer setup only, never GNOME Tab/DoubleShift PASS. Prospective V16 physical wrapper pins this observer plus bounded Writer V3 and preserves all original product assertions; NOT_RUN. CurrentC03/runtime/globalIBus unchanged.


## Native Backspace final focused proof and source import — 2026-10-04

Hash-checked and imported exactly two files from `native-backspace-rereceipt/NATIVE_CODE_HANDOFF.json` SHA750f1c09648891a21b21c93cf6468aab7e0aabd7783d02d8067d3995d6f8c612: observation.rs c118fecef5e641ec504d0b943fcdbec15120321e66711081d291a63e4c93edbf; residuals.rs bb19ee2454593c4d045cd4396e4e8c1ae0a9124c447fe3a86f8802b8a90f4d68. Both root bases and formatted private targets matched before import. `NATIVE_FINAL_EXECUTION.json` SHA4f3e32436a50ff4e9dfc403d321f236539487e62601b924785a5ca99bcb9aa09 establishes nine exact focused PASS (four new systemic regressions, five unchanged strict contracts), a controlled removal of native Backspace background suppression fails at the actual required display-latch assertion, and restored exact retype PASS in8.228s with unchanged final source. The controlled wrapper's later expected-output-name mismatch remains FAIL and is not rewritten; the actual assertion failure is separately retained. Old-source tiny fixture body is byte-identical to the final equivalent body (SHA40ab3bef4410d653c6a21ed745d8ec1779dd7c3f8d823f8f45147de2ed9dfcc0), so its earlier actual one-test failure remains reusable.

Independent grammar_label_audit review PASS_WITH_LIMITS, zero blockers, exact final production c118fece: four first/later RU/EN retype paths, fresh-client confirmation without mandatory extra Reset, refusal after unconfirmed Tab, nine drift/selection/erase classes, native empty-update→hide with no Commit/Delete/background latch. Native deletion remains client-owned and handled=false; only existing inert pending predecessor is carried, confirmed=false/publicationNone until a new exact same-owner client receipt. No new owner, persistent field, detector, layout initiator, executor, model/package or SafetyGate change. Conservative end-of-document/nonempty shortened single-scalar scope remains explicit.

Installed C03 IME a1bdffe17771e241234b6bf130babde052e4bac31578741c0d7de780df6ebaee is unchanged; no physical acceptance or full release PASS in this source import. Root source now freezes for one original-style/nonzero canonical/mandatory graph+canon/onlyIME release pipeline; fresh discovery must retain every old test row and all13 C06/C09 additions. Exact final generated closure must match before installation. Root alone installs with exact C03 rollback, then first checks the reported Firefox Backspace→retype→Tab trigger and all six first/later-word browser variants. Research remains paused.


## Final canonical PASS and decision-format gate repair — 2026-10-04

Actual guarded native-final-PTmtHdcv pipeline:653 selected/executed/PASS,0failed;656 discovered with unchanged3performance exclusions; fresh manifest2992rows=2979original+13required additions, original rows and known-failure semantics literal exact. Original default/research-tools Clippy PASS20.717/21.916s, canonical39.276s. Mandatory graph refresh PASS45.664s, then actual protected-change canon rejected eight decision records for invalid schema or missing required fields; no build/install executed. Failure receipt and logs remain FAIL, never reinterpreted as full pipeline PASS.

Root repaired only the record serialization: exact `lay.architecture-decision.v1`, missing nonempty decision/reason/verification/not_tested, explicit invariant/protected-path lists and authority-change Boolean. All existing historical fields retained, eight originals archived with before/after hashes in `decision-format-repair/ROOT_RECORD_FORMAT_REPAIR.json`. No guard/schema rule, production/test assertion, source identity, detector/layout initiator/edit owner/SafetyGate, candidate/rank/model/package/learning/resource/deadline/concurrency behavior changed. This is documentation/metadata only; root source code remains byte-identical to the actual653/styles proof.

Continue with an explicit proof-reuse hash comparison of all code/config/toolchain/test dependencies, root latest metadata capture, fresh graph/canon and onlyIME build. Do not repeat unchanged correctness/style just to recreate receipts. A fresh final source closure and unchanged post-build identity remain installation gates. Current C03/live bytes, global IBus and model processes unchanged; changed-byte physical acceptance and full release remain unestablished.


## Graph/build continuation wrapper mode check failure — 2026-10-04

Actual guarded native-record-format-3p0Ua3XR continuation rejected before any graph/build/test/style command in0.214s. Worker line81 compared generated graph artifact modes with their pre-graph modes; the preceding recorded graph producer legitimately changed graphify-out/graph.json0644→0664 (recorded source_delta, SHA30b3689e→59a1d58b). This is an overly broad continuation-reuse wrapper predicate, not a production or canon result. Original wrapper rejection remains FAIL and no live bytes changed.

One bounded wrapper-only repair is authorized: compare every non-generated code/dependency/config/guard/test path in SHA and mode exactly with tested canonical/style source; allow only the preceding graph producer's explicitly recorded generated delta, and require those bytes and modes to equal its exact GRAPH_AFTER_SOURCE_MANIFEST rows. Do not allow arbitrary generated changes or relax source coverage. Preserve exactly ten metadata overlay paths (eight repaired records plus two owning docs). Rerun only the narrow reused-proof/fresh graph/canon/onlyIME continuation, no unchanged tests/style. Root freeze resumes after this owning entry. Candidate/authority/resource/learning/rollback contracts remain unchanged.


## Native Backspace repair installed and first physical Firefox trigger PASS — 2026-10-04

Final graph/canon continuation PASS46.977s, onlyIME build195.283s (pipeline244.526s); exact653/styles proof reused only across recorded metadata and generated graph changes. Root hash+mode verified all1462 final source rows, imported exact8graph+2canonical artifacts with backups, and verified exactly five local manifest path translations. `delivery/ROOT_EXACT_SOURCE_IMPORT.json` SHA63a5fae0b3b8a48e2f0c0f2cc2ae07e304377b348fab65821088e57a2e756fab.

Root fresh owned empty Chrome wrapper and unchanged singleIME transaction PASS: `revision5/combined-c06-c09-install/INSTALL.json`. Installed/loaded IME2f813878b8cb86d7144ee845f462acde49ffa3be35fd72ee0a5862296ff4aac8. Exact C03 rollback retained in that transaction. Daemon/L3/L11/globalIBus PIDs and bytes, configuration/input-source identities preserved; no detector/layout initiator/model package/authority-policy change. This is user-authorized experimental78 repair, not published/full-release acceptance.

Root actual Firefox input first reported trigger: `windows/V16_RUN_20261004T154831_7a172eb24b75/WIDE_MATRIX_V16_20261004T154831_7a172eb24b75.json`,1executed/1PASS8.232s. Physical по→Backspace→п→о→Tab: independently observed current prefixпо/suffixчему, exact final почему plus one ASCII space, caret7, ended composition, same owned field focus. UUID observer/loaded bytes bound, own field/device cleanup and original input-source restore; globalIBus preserved. Historical C03 V13 failure remains FAIL. This establishes only this exact changed-byte first-word end-caret Firefox input sequence. Remaining first/later-word partial/whole deletion and other field/client acceptance follow separately; no all-window PASS.


## Changed-byte broad browser Tab/Backspace observer first failure — 2026-10-04

Root actual V16 broad UUID Firefox/Chrome input/textarea/contenteditable series on installed2f813878:48rows,47executed,21PASS,26FAIL,1BLOCKED,113.145s. Receipt `windows/V16_RUN_20261004T154910_f244db2cd6f6/WIDE_MATRIX_V16_20261004T154910_f244db2cd6f6.json` SHA9bc0845d27dd38822dec3e741038df96ddff225659a5c980d110d0433b579f76. Exact own fields/device detached and original source restored, globalIBus preserved. The receipt stays FAIL_OR_BLOCKED; no all-field PASS.

Compact root analysis `ROOT_V16_OBSERVER_FAILURE_PACKET.json`: all27 nonpassing rows have no after_Tab phase. Five stop after actual whole-word Backspace and retype,21 before_Backspace/pre-source validation. Current full owned preedit has DOM text=base+fullCandidate, cursor at base+typedPrefix, active composition.data=fullCandidate. The existing observer assumed composition.data was always a suffix and demanded DOM text=base+prefix+data, correct only for the native committed-prefix route. First native deletion/retype and partial-delete Tab succeed in the actual saved fields; the broad failures do not prove rejected Tab because no Tab was sent there. One ordinary second-word editable case was BLOCKED before_p delivery; current vs snapshot timing remains bounded observer scope, not runtime failure.

Prospective observer-only V17 must distinguish exactly two unambiguous independently exported DOM forms: suffix text=base+current+data; full composition text=base+data, data starts with current and has a nonempty alphabetical remaining suffix. Preserve collapsed UTF16 caret, active current compositionupdate and event generation after Backspace, same UUID/PID/focus, immediate fresh recheck before physical Tab, exact resulting current hint plus one space/ended composition/caret/focus. An empty erased prefix must observe actual ended empty field. No old/ended-hint fallback, runtime return code or caller-specific runtime branch is allowed. Independent review precedes a prospective changed-byte rerun; historical failure is never retrospectively promoted. This changes no source/runtime/reducer/ranker/SafetyGate/authority/model/resource contract; root sole GUI owner.


## Changed-byte eight-pair regression scope — 2026-10-04

Root actual V16 own GTK3/GTK4/Qt/GNOMEauto/GNOMEibus/Kitty eight-pair series: `windows/V16_RUN_20261004T155641_64e75f3a804b/WIDE_MATRIX_V16_20261004T155641_64e75f3a804b.json`,6executed,1PASS/5FAIL39.09s on2f813878 IME. GTK4entry proves all8 word/mode/native rendered RU/EN transitions plus next physical letter. GTK3 stops at seed observation ghbdjtn; Qt stops at seed observation empty; first pair was not established there. GNOMEauto and forcedibus first pair produce gпривет and nextletter gпривета, reproducing the recorded C03 consistency defect. Kitty fails pair3 with correct word/decoder but public-mode assertion; first two intermediate transitions are in its saved progress receipt. Full route/next-mode causality for new failures remains UNKNOWN; no client-specific bypass or unsafe change follows.

All own fields/process/device cleanup and managed route detach, originalsource restore/globalIBus preservation recorded. Failure receipts remain FAIL; GTK4PASS does not certify any other client. These are separate residual DoubleShift/initial-seed/icon acceptance limits, not evidence that the now-proved native Firefox Backspace retype/Tab fix failed. Installed IME/model/daemon bytes remain unchanged by this experiment. Pending broad Backspace observer replan continues with prospective strict dual DOM representation review.


## Prospective V17 dual browser representation source review — 2026-10-04

Private frozen wide_matrix_v17 SHAabc94b064c8b2834d8aa09e870e401b8716bb36347790a39b752cda9e543158a reviewed by root and independent grammar_label_audit: PASS_WITH_LIMITS, zero blockers. Finite V16 diff adds exact mutually exclusive suffix/full DOM representation, permits nonempty current prefix in prefix observations, requires a nonempty alphabetical effective suffix for an actual proposal, and derives final expectation from the exact current observed full surface plus one space. Empty whole-erased prefix requires ended composition/exact base; current-event/owner/focus/collapsed UTF16/recheck/final ended/caret guards remain. Metadata labels describe browser DOM representation and do not independently prove runtime ownership or rendered pixels.

Ordinary p_Tab keeps fixed почему plus one-space quality predicate. Existing one-second bounded readiness now establishes exact base/collapsed UTF16/ended composition before KEY_G and immediately rechecks; actual saved BLOCKED prefix snapshot was тес/caret3 while the later read was тест-space/caret5. No timer/retry-until-green or runtime change. V16 original rows and FAIL_OR_BLOCKED receipt remain immutable. Root prospective changed-byte Firefox after-word retype then full48 browsercases follows; helper source review is not physical acceptance.


## Prospective full-composition Backspace trigger physical PASS — 2026-10-04

Root V17 actual Firefox contenteditable after-word delete/retype/current-hint/Tab1executed/1PASS4.195s: `windows/V17_RUN_20261004T160139_edbbd8babd02/WIDE_MATRIX_V17_20261004T160139_edbbd8babd02.json`. Independently exported FULL_COMPOSITION prefixпо/dataпочему, exact observed DOMтест-почему without assuming suffix duplication; Tab produces тест-space-почему-space, caret12, ended composition, same field focus. Exact2f813878 loaded/installed bytes and V17abc94 source pins bound. Owned field/device cleanup and originalsource/globalIBus preservation recorded. This proves the previously blocked representation in this one actual field only; it does not retrospectively alter V16 or establish pixels/runtime-owner proof. No new source/runtime change. Root full48Firefox/Chrome first/later-word ordinaryTab and sixBackspacevariants is the next prospective acceptance scope.


## Prospective V17 browser series measured result — 2026-10-04

Root actual installed2f813878 V17 series `windows/V17_RUN_20261004T160211_002f138e849b/WIDE_MATRIX_V17_20261004T160211_002f138e849b.json`:48rows=44PASS/3FAIL/1BLOCKED,47executed,74.322s, nofatal, allownfield/device cleanup/source restore/globalIBus preserved. Chromeinput/textarea/contenteditable8/8each; Firefoxinput7PASS/1FAIL,textarea6PASS/1FAIL/1BLOCKED,contenteditable7PASS/1FAIL. Sixteen successful Backspace cases used exactFULL_COMPOSITION and16SUFFIX DOM forms. These are changed-byte final-effect receipts, not pixels/runtimeownership or all-windowPASS.

Three Firefox after_word_partial_delete_Tab cases stop BEFORE actualTab: exact full proposal remains visible after Backspace with collapsed caret shortening, but no new compositionupdate sequence exceeds the pre-Backspace floor. Internal candidate recomputation/authority is UNKNOWN from DOM alone; read-only saved trace/source analysis follows. Preserve the original fresh-event requirement/result. One Firefoxtextarea afterword retype case stopped on actual same-window/title focus loss; no actualTab sent and no bypass/retry-until-green.

Prospective distinct retained-visible-hint probe may use only the exact same active full proposal/event identity/data observed before physicalBackspace, actual one-character shortening and new collapsed caret/samebase/currentprefix/focus/UUID, no ended/olderhint fallback. It must record internal freshnessUNKNOWN and prove only actualTab final exact observedword+one-space/endedcomp/caret/focus. This observational variant does not relax runtime verifier/authority/SafetyGate and never retrospectively promotes the fresh-event failure. Isolated repeat of the focus-interrupted original retype remains a separate receipt. No runtime/code/model/route change in this experiment.


## Isolated repeat of focus-interrupted Firefox retype — 2026-10-04

Root `windows/V17_RUN_20261004T160818_5852c60671c3/WIDE_MATRIX_V17_20261004T160818_5852c60671c3.json`: original unchanged V17 afterword partial-delete/retype/Tab identity in fresh own Firefoxtextarea1executed/1PASS; exact итогтест-space-почему-space, current full proposal+onespace/caret/endedcomposition/focus confirmed. Previous matrix's focus-loss BLOCKED stays BLOCKED and is not overwritten. No retry-until-green or source/runtime change; this one independent bounded replay resolves the unexecuted original scenario for this field. Owned field/device cleanup/source restore/globalIBus preserved. Remaining three retained-fullhint/no-retype probes are separate observational scope.


## Retained-visible full hint V18 reviewed before physical Tab — 2026-10-04

Root and independent grammar_label_audit finite review PASS_WITH_LIMITS/zero blockers for frozen wide_matrix_v18 SHAedd727dd8a3cd135a30710880990bb8bdc1e1e1f37f39a20ba34632e64fe06ac (diffd1a569a61b41527bdf51604c719b88a7f368358be973acb8d6172b293cd58cfe). Exact CLI only afterword_partial_delete_Tab in three Firefox field kinds; V17 fresh proposal/prefix/ordinaryTab predicates literal unchanged. Retained branch requires actual before/after/immediate-preTab same UUID/page/title/file, active fullcomposition identical current event/data/DOM, snapshot sequence/time progression, physical one-character deletion and exact UTF16 caret shortening, alphabetical nonempty extension of the shortened prefix. ActualTab must produce observedfullsurface+onespace, endedcomposition/currentfocus/collapsedcaret. Internal candidate freshness remainsUNKNOWN from DOM alone; no runtime-return-code or endedhint fallback. Historical V17 stays44PASS/3FAIL/1BLOCKED. Root executes only this separate bounded prospective physical scope; no production/SafetyGate/route or runtime change.


## Backspace/Tab six-browser-field final-effect closure — 2026-10-04

Root retained-visible-hint V18 actual `windows/V18_RUN_20261004T161509_ceed0b35ecf3/WIDE_MATRIX_V18_20261004T161509_ceed0b35ecf3.json`:2executed/2PASS and1initial-focus BLOCKED7.199s; Firefoxtextarea/contenteditable Backspace→Tab without retype yields exact тест-space-почему-space, caret12, ended composition, samefocus. One independently repeated unchanged observer Firefoxinput `windows/V18_RUN_20261004T161735_19372221999f/WIDE_MATRIX_V18_20261004T161735_19372221999f.json`:1executed/1PASS3.801s same exact effect. Earlier BLOCKED remains immutable, no retries-until-green. Own fields/device/source cleanup and globalIBus preservation in each receipt.

Causal source/trace analysis `windows/V17_FRESH_HINT_CAUSAL_RECEIPT.json` SHA7f84bf17d88b7e0c239ee4d369f0a639ef9c2f448f4bc54611f0964b793f8d54 establishes all3 earlier ownedBSP handled=true/accepted with stable owners150/160/171; new epochs1441→1442,1546→1547,1648→1649 and applied generations2878→2880,3126→3128,3373→3375. Actual UpdatePreedit publishes same fullword with cursor2→1 and new prefixп/suffixочему, while browser event105 remains105. Browser implementation's exact event dedup branch is UNKNOWN; the V17 first failed transition was its event105>105 predicate before anyTab. V18 independently proves physical final Tab effect, not inferred internal authority/pixels.

Aggregate `windows/ROOT_TAB_BACKSPACE_ACCEPTANCE_20261004.json` SHA09ff89f234244aac79960cae19fd67fb06473a1709f624343e186c44189f48da binds48 UNIQUE successful final-effect identities on exact installed IME2f813878b8cb86d7144ee845f462acde49ffa3be35fd72ee0a5862296ff4aac8:8each Firefox/Chrome input,textarea,contenteditable. The8 consist of ordinary first/second-word Tab and partialdelete/partialdelete+retype/wholeerase+retype at first/second word; actual current observed hint completed with exactly one space and correct ended composition/caret/focus. Three identities use explicitly declared retained-visible full-surface observation; all others original V17. Union uses four exact separately bound receipts, never claims one uninterrupted48PASS or retrospectively converts any historical FAIL/BLOCKED. No private WhatsApp/GitHub field, human keyboard, pixels or other-window acceptance follows.

Current Backspace→retype→Tab user defect is repaired in installed experimental78 within this six-field/end-caret scope. Canonical653/653/styles/source and exact rollback retained. Other changed-byte DoubleShift failures (GNOME g-prefix, Kitty mode/icon, GTK3/Qt seed) remain OPEN, not silently promoted. No commit/tag/push/full release publication in this task. Research stays paused. One mandatory remote AST/source-binding/canon refresh of these owning-document changes remains; it must not rebuild/reinstall/restart or rerun unchanged tests. Root final runtime health verifies sameIME and unchanged globalIBus/models/config/source identities.


## Final documentation graph result and current runtime — 2026-10-04

Mandatory remote graph-only refresh completed PASS: `tab-backspace-final-graph/delivery/GRAPH_EXECUTION.json` SHA43bed571f2fef3339014750a4915116a88f524ed0538655e61e723c5b0e6c7fc, AST+binding45.267s and actual protected canon79 0.990s. Root imported exact8generated members/6changes and independently verified all1462 after hashes+modes: `ROOT_GRAPH_EXACT_IMPORT.json` SHA62f137af63ca02a2e1c40ab05853ba176784952697951ac5684e3f3a0f00dae0. No build/test/style/fit/install/GUI/runtime call. This establishes architecture documentation/canon only; the installed artifact remains bound to its original deployed build manifest. This result report is the only new documentation delta; final AST/binding refresh will bind this report without changing functional source or repeating its checks.

Read-only final runtime `windows/ROOT_TAB_BACKSPACE_FINAL_RUNTIME_HEALTH.json` PASS: IME1956960 SHA2f813878…4aac8, globalIBus270775, L3/L11 original PIDs3296775/3296699 and bytes; daemon2173433 changed only for owned keyboard attach/detach, same original daemon bytes. GNOME/IBus both lay-ime-ru, config/inputsource identities unchanged.48unique Tab/Backspace final-effect successes across6ownedbrowserfields retained. DoubleShift residuals OPEN, actual privateclients/humankeyboard/fullrelease unproved, research paused. This is the current milestone; no commit/tag/push in this repair.


## Current-word IME decoration regression — 2026-10-04

User clarified that Firefox decorates the whole current word, not previous words. Actual installed/loaded IME remains `2f813878b8cb86d7144ee845f462acde49ffa3be35fd72ee0a5862296ff4aac8`, HEAD79b09d06306a018961ace783e2542eb817de7b30, dirty source preserved. The first demonstrated loss is rendering: `text.rs` passes total payload length to `attrs::preedit`, which applies gray and single underline to `[0,total)`. `preedit.rs::composition_preedit_payload` deliberately contains typed buffer plus suggested suffix, and pending/no-candidate publications contain only typed buffer. The suffix-only fixture cannot distinguish these cases. Previous48 browser checks certify text, caret, ended composition and Tab spacing, explicitly not pixels; their receipts remain unchanged.

Decision `decisions/2026-10-04-preedit-suggestion-visual-boundary.json` records C01/C03/C05/C06/C07/C10 and consequence analysis before implementation. Preserve existing owner, payload, cursor, reducer, Tab/Backspace/Double Shift and all routes. Derive the untyped suffix boundary from an exact buffer prefix; explicit no underline on typed prefix, existing gray/single underline only on suffix. A mid-word caret is not the hint boundary; nonmatching replacement stays neutral. No ranking/candidate/verifier/model/cache/package/reload/learning authority changes; only bounded prefix/scalar work and one extra attribute, no new owner/worker/queue/timer/fallback. Material risk: Firefox client default decoration may ignore the requested neutral prefix, so source wire PASS must be followed by an owned-field pixel observation and unchanged final-effect Tab checks. Root owns implementation/import/install/GUI; agents independently review, h7 owns guarded remote execution. Research stays paused; existing Double Shift residuals remain OPEN. No production change, install or runtime authority change at this preflight.


### Visual-boundary source proof and final pipeline freeze — 2026-10-04

Root imported six remote-formatted files with exact SHA/mode0664, ROOT_FORMAT_IMPORT.json; only two files changed under rustfmt. Independent finite review of all six files and actual legacy raw-attribute assertion is PASS_WITH_LIMITS, zero blockers. Ten exact production-focused tests passed; controlled publisher suggestion_start=0 produced two required FAILs: direct publisher whole range0..6 instead of neutral0..2/hint2..6, and actual legacy wire whole0..3 instead of neutral0..1/hint1..3. Exact restored source hashes and modes produced2/2PASS. This is a controlled reproduction of the old rendering boundary, not a historical runtime rerun. Execution42.977s: `/home/local-user/.cache/lay/development/ranker-release-20261004/preedit-visual-boundary/focused/proof/FOCUSED_EXECUTION.json` SHAe966ca5ce38ad5b53798410f1bd8bc4de9793f824235169490a42f1c8f829b47. Original manifest/ledger remain unchanged.

Final source freeze authorizes one fresh discovery retaining every original row/lane/isolation and adding only the two new visual tests, original default/research-tools style gates, nonzero canonical IME closure, mandatory graph/canon and ONLYIME release build on the guarded remote host. Installer and NoKeyboard owned-empty-Chrome wrapper were independently reviewed source-only,0blockers; prior2f813 exact rollback and all service/config/source identity checks preserved. Root alone executes any later installation/GUI. Wire/source facts do not certify Firefox rendering; current installed2f813 remains unchanged, physical pixels and changed-byte Tab/Backspace checks still NOT_TESTED. No transport, authority, model, ranking or architecture-canon change.


### Visual-boundary installed source and client final effects — 2026-10-04

Original final source gates: fmt PASS2.732s, fresh discovery29.825s retains all2992 old rows/lane/isolation and adds exactly2, both original Clippy configurations PASS20.783s/21.670s, canonical655selected/executed/PASS0failed39.539s, mandatory graph/canon PASS. Parent pipeline records prebuild FileExistsError for reused TARGET_BEFORE.log, before any release Cargo. ROOT_BUILD_NAMESPACE_REPLAN authorizes only a fresh private build namespace with all1463 tested/generated hashes AND modes exact; original FAIL receipts retained. Actual ONLYIME release build194.991s / worker195.487s PASS, binary8,073,960bytes SHA8ef1f4de69a19d580bd0d9af1ff273294dc1b5ecff14f1f526ecbf9445440616. Build manifest `/home/local-user/.cache/lay/development/ranker-release-20261004/preedit-visual-boundary/final/delivery/BUILD_MANIFEST.json` SHA0f03cc78853010cd69f7a7d73924f5242a0161e61f79dfeb2ef9802d6596be8a; original remote manifest and exact five path translations preserved. Root imported exact10 graph/canonical files and checked all1463 source hashes/modes before installation; ROOT_EXACT_IMPORT.json. Inherited build-wrapper baseline/full-release labels are historical scope labels, not failures of these two measured style configurations and not an all-window release certificate.

Root-only NoKeyboard owned-empty-Chrome installation PASS: `/home/local-user/.cache/lay/development/ranker-release-20261004/preedit-visual-boundary/install/INSTALL.json` SHA09a640c10040734e17a7fd29c01ebb126bf34bfdb0efec593c124f73d0d1ef77. Only existing IME channel reloaded; exact installed2f813 rollback saved and hashed. Daemon/L3/L11/globalIBus PIDs/bytes, config/input sources, extension/model packages remained unchanged during the transaction; no input protocol, ownership, SafetyGate, gesture or layout route changed. Subsequent root owned acceptance device attach/detach changes daemon PID only, bytes unchanged. Final read-only `/home/local-user/.cache/lay/development/ranker-release-20261004/preedit-visual-boundary/ROOT_FINAL_RUNTIME_HEALTH.json` SHA38dd1f41ebdc12b5f2888c19c9a0e12f2efa6d467ca08329abce1086839f0ac1: installed and loaded8ef1 match, IME2598471/daemon2637875/L3 3296775/L11 3296699/globalIBus270775, GNOME/IBus coherent, sources/config preserved.

Physical final effects: new-byte V17 48executed/45PASS/3FAIL76.319s; three Firefox after-word partial-delete cases stop before actual Tab because the same full-composition event remains unchanged. Exact preexisting V18 retained-full observer separately executes those three and yields3/3PASS8.447s. Root union covers48unique successful scenarios,8each in Firefox/Chrome input/textarea/editable; never claim one uninterrupted48PASS or rewrite oldFAIL. Each successful physical Tab observes exact current surface plus one ASCII space, ended composition, collapsed correct UTF16 caret and same owned field. Aggregate `/home/local-user/.cache/lay/development/ranker-release-20261004/preedit-visual-boundary/ROOT_TAB_BACKSPACE_ACCEPTANCE.json` SHAf61ea8e6593945122c2c30c472cc75d4415a6affd7f0dd2a3f83fceb8066ea3b binds the two immutable receipts and8ef1 bytes.

Firefox rendered decoration remains UNKNOWN, awaiting user observation requested asynchronously after physical input cleanup. Wire proof establishes neutral typed prefix and gray/single-underlined untyped suffix; payload/cursor/owned composition are intentionally unchanged. Four bounded baseline capture attempts never sent Tab and never established pixels: first helper print injection error0cases; gnome-screenshot returns0/noPNG; ordinary application name already owned; normal direct ScreenshotWindow raises an error. Last stderr is truncated, so exact GNOME rejection cause is UNKNOWN. ROOT_SCREENSHOT_TRANSPORT_REPLAN, ROOT_DIRECT_SCREENSHOT_API_REPLAN and ROOT_CAPTURE_STOPPED preserve failures; no screenshot permission/unsafe-mode or trusted owner change. All owned fields/devices closed and globalIBus preserved. Source/Tab final-effect PASS does not establish visual Firefox, private WhatsApp/GitHub, other windows, complete ranker quality or remaining Double Shift acceptance. Research stays paused. Final owning-document AST/source binding refresh follows without tests/build/runtime change.
