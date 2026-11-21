# TD-123 — Single internal omissions and readiness at Space

Baseline `v1.0.126` at `cd61dadc689b7819b34c23163a7759fd7fe15ab0`, dedicated branch
`codex/td123-internal-omission-20261120`. The loaded/installed IME matched
on the 20 November audit. The paused browser checkpoint `1d19fba9` stays
separate. This is correction-only work for all desktops and clients.
Existing input, field ownership, selection, Tab, Space deadline, display,
execution, desktop adapters, installation and services are protected.

`~/.cache/lay/development/typo-audit-20261120-vewjcxjt/analysis.md` owns the
sanitized terminal evidence. The last word's Space lease waited 3,386 µs;
prefetch for its exact epoch recorded 117,145 µs and was superseded. A
separate zero-wait refusal needs separate attribution. The correct
restoration exists in L1.1's candidate pool for all five tested one-letter
omissions, with no L1.1 authority awarded; one context-free word is
ambiguous between insertion, deletion and substitution. A shared field
material entry already serves display and correction; the live trace
shows one producer and one waiting consumer.

Before each code step, record the owner and exact path here and in the
active scope decision. Compare the same inputs and surface/safety verdicts.
A faster calculation does not by itself establish a correct replacement.
For normal and short last-letter/Space intervals, report coverage and time
from the input event to usable decision, with missing decisions visible.

## First step — reuse immutable work inside reconstruction

`space_autocorrect_prefetch.rs` schedules the existing worker →
`ime_correction.rs` uses the common decision →
`nanda_wave/lexical_phase/runtime.rs::reconstruction_lane_reserve`
sorts and reserves candidates by edit distance and other stable keys.
The sort and lane selection recalculate the same distance from unchanged
strings. The sole runtime path allowed in this step is
`src/nanda_wave/lexical_phase/runtime.rs`.

The 13 November preflight already measured this hunk, 224/224 identical
ordered field outputs, paired full-word computation median 19,714 →
13,248 µs, and independent review 9/10. Those receipts describe the
preceding candidate. The short native Chrome case still missed the
deadline at a roughly 15.6 ms last-letter/Space interval, so the old
measurement is not a claim of native repair here.

Alternatives: add a second token cache (2/10; existing producer reuse is
already observed), extend Space wait (1/10; changes protected behavior),
or reuse derived values in the existing reconstruction invocation (9/10).
Exact patch origin: `1d19fba9`, sole file above. Measurements and verdicts
will be recorded separately for the released base.

## Second step — selection

Trace the one-token candidate and the boundary competitor through L1.1,
L2, L3/L4 and the common decision before changing any rule. Preserve the
existing lexical authority, SafetyGate and verifier. Compare `услить`
with other one-letter omissions, genuine ambiguous ties, clean forms
including `криптой`, and the working `должн ыбыть` boundary control.
Keep the fixed quality gate and the original failed data. No model fit.

The released CLI RED is retained under
`~/.cache/lay/development/td123-decision-evidence-e_1kygrl/case-0.log`.
For the unsupported boundary candidate the exact new left center is absent,
mass gain is 0, direct apply is false, and the original left word is clean.
The typed producer sends it anyway. The first failure is the handoff from
`phrase_reader/moved_prefix.rs::boundary_shift_proposal` into
`correction_core/candidate_sources.rs::boundary_shift_transition_candidate`.
`typing_transition/decision/admission.rs::attested_space_boundary_admission`
can then promote the unsupported shift. The good single-token candidate was
retained, but its L2 tie still needs contextual confirmation. Compare the
true control: the invalid left word in `должн ыбыть` provides a positive
boundary mass gain of 1186 and direct permission.

This stage allows exactly `src/phrase_reader/moved_prefix.rs`, including an
assertion in its existing regression body. Protect the already certified
left word before emitting the same two-word proposal. Reuse the existing
exceptions and positive pair evidence and preserve the working boundary
corrections. Do not change the shared rank gate, SafetyGate, verifier,
bound text delivery, config or native adapter. The prior performance hunk
remains separately identified. Owner decision:
`docs/architecture/decisions/2026-11-20-protect-clean-left-in-boundary-proposals.json`.

## Comparative results

Speed-only canonical library result: 1871/1871 PASS, local
`run-w4kagf1l/RESULT.json`. Initial `run-hio48bs5` was blocked by the
remote swap admission guard before tests. A task-owned temporary swap
file restored headroom without changing guard limits.

The two immutable diagnostic binaries were built from the exact released
source archive and the exact two-file candidate diff. Identity, capabilities,
guarded test commands, all 69 inputs and every verdict are retained at
`/home/ubu/.cache/lay/development/td123-internal-omission-20261120-7VhusW/RESULT.json`; fixture SHA-256
`2513efef9baf3fe2b52814a84cfe3ad92e64282309f1e3332dd603753e7699b5`.
The fixed suite ran three alternating comparisons. The 20 one-letter
omissions give 60 timing samples per arm. Median was 62.204 → 58.746 ms,
p95 101.506 → 88.417 ms, maximum 114.061 → 101.799 ms.

| Time from computation start | Baseline finished | Candidate finished | Correct apply by that time |
| --- | --- | --- | --- |
| 20 ms | 0/60 | 0/60 | 0 → 0 |
| 80 ms | 48/60 | 51/60 | 26 → 27 |
| 120 ms | 60/60 | 60/60 | 27 → 27 |

These are computation envelopes for controlled gaps. They exclude native
callback, scheduling before work starts, publication and text delivery.
The original actual-key Space lease and deadline remain the same.

One selection changed out of 69: unsupported «что нужноу слить» became
Keep. All four positive boundary rows stayed correct, as did three
additional CLI controls. For one-character omissions, 9/20 isolated and
3/8 contextual targets apply on both arms. The other decisions in those
two groups remain Keep. Clean preservation remains 26/28; «криптой» and
«плачу криптой» still produce the pre-existing wrong «криптон» in both
synthetic arms. A scoped correction-quality improvement is measured; the
general TD-123 class and native acceptance remain open. A phase threshold
or rule that forces every target from this set would need independently
measured discrimination against its valid competitors.

## Producer guard rejected on the fixed corpus

`run-bp_sqpmp/RESULT.json` and remote `run-NcS20Y/tests` retained
3089/3090 PASS and one blocking error. Corpus: 220 clean pairs, 0 false
applies, 188 damaged pairs, 168 proposals and 156 direct recoveries.
89.4% proposal recall fails the existing 98% threshold. Do not carry
the generator guard or its test hunks forward. Independent review at
5/10 confirmed the failure. Its 69-row A/B above remains evidence for
that rejected variant only.

The held trace reports `ranked=Some(0) selected=Some(0)` for the wrong
shift; the ordinary decision is the first admitting lane. If it refuses,
the existing attested boundary fallback can still consider the candidate.
Expose the same evidence rule to both admissions. The ordinary and
attested routes must keep unrelated controls and both remain subordinate
to the existing verifier and SafetyGate.

Only `src/typing_transition/decision/admission.rs` is added to the
exact correction-only scope. Preserve all producer output, including the
old forms lost in the rejected variant. Require a verified one-character
internal insertion with no left change as a competitor, a certified
original left token, an uncertified expanded left token and absent
independent contextual evidence to refuse the weak shift. This does not
upgrade the competing token's own L2/L3/L4 evidence. Keep the speed hunk
already tested independently. Owner decision:
`docs/architecture/decisions/2026-11-20-compare-boundary-against-internal-omission.json`.

First selection-only diagnostic within this revision left all 69 verdicts
unchanged. The targeted clean-certificate trace proves why the condition
did not activate. The expanded left has `exact=false morphology=true`,
and is neither a common form nor user-protected. The original left and
the competing one-word target have exact dictionary evidence. Therefore
judge the proposed expansion by the existing exact dictionary, exact
surface center or user certificate. An independently certified L3 pair
or exact positive L4 state continues to admit a context-supported shift.
Do not change global morphology or discard the proposal.

Existing private IBus/Readline `terminal-delivery` reported the same
first short-form failure on released 126 and the diagnostic candidate.
Both received a ready lease, with `full_no_apply/rank`, and the stock actor
recorded `published correction must consume Space`. Do not reuse this
as a PASS or compare its different binary profiles as a speed benchmark.
The bounded evidence remains in remote `td123-internal-omission-20261120/
refined/PRIVATE-CLIENT.json`. No physical desktop test was run.

## Final selection comparison

Final two-file source hashes are in
`/home/ubu/.cache/lay/development/td123-exact-left-20261120-iQP6PU/BUILD.json`.
The unchanged 69-case fixture, three alternating rounds and exact 126
baseline are tracked in `/home/ubu/.cache/lay/development/td123-exact-left-20261120-iQP6PU/RESULT.json`.
The same input-and-target list lives in
`td123-internal-omission-cases-2026-11-20.jsonl` for future comparisons.
The only changed decision is the same unsupported two-word replacement
changing from Wrong to Keep. The generator matches the release
byte-for-byte. All 4 positive boundary cases and all 3 CLI boundary
controls still correct. Isolated omission yield is 9/20 and contextual
yield 3/8 on both arms. Clean preservation is 26/28 on both arms.

| Synthetic interval | Before finished | After finished | Correct apply before → after |
| --- | --- | --- | --- |
| 20 ms | 0/60 | 0/60 | 0 → 0 |
| 80 ms | 48/60 | 50/60 | 27 → 26 |
| 120 ms | 60/60 | 59/60 | 27 → 27 |

The median on single internal omissions is 60.490 → 57.535 ms, p95
98.400 → 92.087 ms, and maximum 115.480 → 130.876 ms. The longest
observation is still the ambiguous example, with 58.986 ms in the
productive stage and 25.491 ms in the decision stage. This is a median
and p95 improvement for this compute sample, with a worse observed
maximum. It does not establish improved fast-input coverage or
Space-to-visible-text latency.

Final independent source review 8/10, no blocking defect observed in the
diff. Potential coverage limitation: a real morph-only left expansion
can remain unchanged in the presence of a viable one-letter omission.
The current plain-Cyrillic L3 projection covers one changed slot or an
unchanged prefix. For a true two-word shift, the effective positive
escapes today are the existing exact left form or exact positive L4.
All tested positive controls passed. General quality, actual physical
typing acceptance, installation and publication remain separate.

Canonical development verification finished **3090/3090 PASS**, including
the unchanged boundary corpus gate, library, IME, text mutation monopoly
and typing transition authority. Receipt
`~/.cache/lay/development/run-sbkd1lx4/RESULT.json`, source archive
`2120a5a7449742872cfc72850a069fd9f8176ee80ddf616da4392cf032e740c5`.
Both runtime file hashes match the request. Architecture graph, scope
and canon pass. The task's temporary remote swap was removed.
Development checkpoint; the private-client baseline failure and
fast-input issue remain open for release acceptance.

## Complement aspect from the observed left context

Correction-only preflight and baseline C01. Source is clean at `263b325ecfc9f1a47489eba61f2777dfdb3ae1dc`. Primary GNOME runs Lay 1.0.126. Installed and live IME SHA-256 is `5cf26855fa8deead77c6dee59b013182f78791fe6081380a3ad1b885db77dc9d`. Remote proof and primary L3 base SHA-256 is `a71d58a0a01f9c5f8fae4328e1e5011043f3e95ac1d5ee760a0dc56b81cd9ad7`.

Space assembles the observed prefix in `src/ime_correction.rs`. The existing DecisionCore passes original and replacements to L3. The first missing evidence is in `src/nanda_wave/l3_phrase_gate.rs`. Both aspect alternatives survive candidate collection. The relevant L3 profiles are absent or neutral in 13 contextual probes. Two positive recurrence controls do edit and two ambiguous probes keep. Raw traces and timings are in remote `td123-internal-omission-20261120/left-context-rorkdb7e/BASELINE.json`. The earlier 9/20 isolated target count measures hand-labelled fixture agreement only.

Options considered. A per-surface L3 delta needs exact independently labelled target and rival scenes. No such scenes for the requested pair were found in the indexed natural corpus. A broader syntax ranker could learn longer scope and semantic context, but needs independent calibration and would expand the protected proof. Choose a bounded context signal in the current L3 owner for retained one-insertion infinitives with opposed aspect. The source is corpus observations plus canonical V13 feature readings. An unavailable, ambiguous or contradictory readout keeps the existing decision. Train observes adjacent infinitives and at least 16 distinct xcomp lemmas. The independent statistics and heldout cases are in remote `td123-internal-omission-20261120/aspect-proof-h85amytw/CORPUS.json`. The five frozen profiles had no contrary adjacent infinitive in 96 independent observations. This is evidence for a bounded contrast, not an estimate of all typing errors.

Allowed runtime seams are `src/nanda_wave/l3_phrase_gate.rs`, `src/nanda_wave/l3_phrase_gate/infinitive_aspect.rs` and `data/lexicon/russian_infinitive_aspect_context.tsv`. L2 retention and existing DecisionCore, SafetyGate, verifier, owner and writer stay in their current paths. Preserve all context evidence already sufficient for a decision. The lookup uses bounded features in the existing warm immutable V13 provider. It has no new cache, background job, package reload or learning side effect. A different dictionary revision can remove support through ambiguous readings. Governor ambiguity and false support are the main regression risks. The existing certificate can change a SuggestOnly verdict, so measure written-plan authorization rather than rank alone. Both consuming processes need matching bytes for deployment. Preserve desktop, daemon/IME routes, preedit, gestures, config and packages.

First compare the frozen 22 contrast/control probes, 146 corpus rows and 69 prior controls. Reject new wrong edits or changes to unrelated controls. Keep actual timing separate from decision latency. Run canonical affected-source tests and an exact release-profile build before a primary install. User authorized primary installation after checks. Rollback is the immediate installed 1.0.126 binaries and their established restart path. Revert this scope and data as one unit if the signal fails acceptance.

The first contextual candidate selected five previously uncorrected contrasts and seven heldout repairs. The remaining clear probe used a governor with both a first-person plural finite reading and a short singular reading under the same V13 lemma. A bounded personal subject check resolves that specific ambiguity. Person mismatch and paragraph boundaries still keep the input. Exact V13 evidence is in remote `aspect-candidate-TxvPOR/READINGS.json`.

The final source candidate adds 13 correct edit authorizations over the same accelerated baseline in 242 measured requests. Six are contrast fixtures and seven are independently labelled corpus rows. There were zero new wrong replacements. No decision changed on the 120 unprofiled heldout cases or previous 69 controls. The first failed transition remains contextual evidence. Strong semantic contexts beyond the observed governors, including the one-evening example, stay outside the new authorization. The synthetic gate is at remote `aspect-final-XMV74r/RESULT.json`. The package remains an experiment pending source and install checks.

Final source verification completed with 3,652 selected checks and zero failures. The earlier compile failure was limited to two older unit test calls to the renamed helper. Only those `cfg(test)` calls changed after the A/B. Runtime prefix parity is recorded at `/home/ubu/.cache/lay/development/td123-context-final-qzlk37nt/PROOF-SOURCE-PARITY.json`. The full result is at `/home/ubu/.cache/lay/development/run-e9hg7r2m/RESULT.json`. All 13 new corrections completed within 80 ms. On 233 frame requests, compute p95 was 59.12 ms versus 56.51 ms on the baseline. These timings omit field delivery.

Primary GNOME deployment verified at 2026-11-20T23:52:46+03:00. The three exact-profile binaries were installed from the sealed source. The complete installed-to-live checks, versions and rollback hashes are in `/home/ubu/.cache/lay/development/td123-context-final-qzlk37nt/INSTALL.json`. The running IME SHA-256 is `11d3cbb43782ab5425ad8a21c2f01bc7674eb940c7428c8ec05894b9c2becce3`. The CLI and tray report 1.0.126. Lay daemon, IBus, IME, L1.1 and the L3 learner all restarted. The selected source and engine are lay-ime-ru. The previous binaries are kept in the deployment receipt directory. Source includes the prior correction-only 263b325e changes. This is a local user-test build. Physical user typing acceptance is pending.

The public release is prepared on `187434f576e5a120a52d5e77083371220da507f4`. Upstream research files and documentation are preserved. The five correction sources and corpus inputs are copied byte-for-byte from the previous source and installation proofs. The version bump updates package metadata and the tray's version and date literals. The existing trial remains installed on primary GNOME for manual acceptance.
