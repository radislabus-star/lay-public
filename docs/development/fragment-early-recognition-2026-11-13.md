# Earlier fragment decisions, 2026-11-13

## Final decision: rejected, accepted detector preserved

The user selected "Сохранить качество: этот вариант не принимать" after being
shown candidate B's measured regression. Both candidates are rejected. The two
experimental runtime/test hunks have been reversed exactly to baseline; no
production source change remains. There is no installation, service restart,
new release, model replacement or native acceptance claim from this task.

Keep the production-entry streaming proof and its per-identity comparator as
offline diagnostics. The experimental support filter and prefix-report modes
are removed from the maintained example. Rejected sources, the patch, frozen
policy and evaluation receipts are retained privately at
`/home/ubu/.cache/lay/development/fragment-early-recognition-20261113/`.
The `rejected-sources/SHA256.json` manifest identifies the archived code.
Do not repeat A or tune B on the inspected examples. A future attempt requires
new evidence that addresses clean-prefix ambiguity and a separate proposal.

## Preflight

Class: correction-only, explicitly requested after diagnosis of `djk -> вол`.
Baseline: `ffe94e1a8828d743b520ad7c852e65c366356cd3`, stable126. The existing
uncommitted handoff additions from November12/13 are preserved. Primary GNOME
Kitty's loaded126 IME matches the installed SHA
`5cf26855fa8deead77c6dee59b013182f78791fe6081380a3ad1b885db77dc9d`; fragment and
ordinary autoflip are enabled. Previous native126 evidence is historical.

First failed transition: the embedded model scores вол +25 and djk -25, but
`TransitionDecisionCore::decide_fragment_layout` rejects every unmarked target.
The existing minimum3 letters, model, candidate/verifier contracts, user
protection, Space pipeline, and all input owners/transports remain unchanged.

## Dependency map and scope

`layout_fragments/letter.rs::on_letter` (unchanged letter/length guard)
 -> `layout_fragments.rs::analyze` / `model.rs` (unchanged embedded scorer)
 -> `typing_transition/decision.rs::decide_fragment_layout` (experimental edit,
    now restored exactly to baseline)
 -> existing typed request, verifier, IME caller and writer (all unchanged).

Allowed runtime file: `src/typing_transition/decision.rs`, and only its fragment
decision entry. Tests: `src/layout_fragments/letter_tests.rs`. Offline proof:
`examples/fragment_prefix_proof.rs`, `scripts/proof/fragment-prefix-quality.py`.
Canonical test discovery manifests may record added tests; no waiver. Owning
documents and generated graph/receipt must follow existing gates. No model,
dictionary, layout adapter, installed config, service or delivery change.

## Hypothesis and alternatives

The older40,000-case actual compiled streaming proof already measured this
tradeoff. A permissive margin-only policy caused283 clean-word flips/20000;
the current Known/current-support guard reduced them to12 (RU1, EN11), restoring
5235/10000RU and7575/10000EN wrong-layout words before Space. Do not repeat the
rejected removal of all support guards. The old source decision SHA matches
current126; other dependencies and timings still need current verification.

Candidate A retains the current Known path and all current-language/protection
guards. It additionally admits an unmarked target only with at least4 matched
fragments on EACH interpretation, mean target weight>=4, and mean current
weight<=-4. Embedded training weights are bounded at5 in magnitude. This is a
predeclared strong, consistent signed-evidence hypothesis, not a calibrated
probability or a claim of independent overlapping features. It must pass the
fixed corpus before promotion. No word/suffix literal enters runtime code.

The separate short-English-target guard remains unchanged in this first scoped
repair. Relaxing it, admitting2-letter inputs, punctuation-containing prefixes,
or retraining a larger model are separate experiments, not hidden extra edits.

## Verification protocol and stop conditions

1. Reuse the previous streaming actor's physical-key decoding and actual public
   `on_letter` calls in a tracked offline example. Select the exact original
   SHA-partitioned10,000 heldout words per language, clean and wrong initial
   layouts. Freeze input/model hashes. Use isolated HOME/XDG and remote guard.
2. Obtain current126 baseline and candidate results: every intermediate clean
   flip (even if later reversed), wrong-layout final restoration, multiple flips,
   never-fired words, first-trigger letter distribution and detector timings.
3. Acceptance: no increase in clean false flips in either language; no loss of
   baseline-restored targets; no new wrong-after-flip or repeated reversals.
   Preserve corpus denominator and exclusions. Report failures without tuning
   a literal/example, weakening guards, or sweeping thresholds on heldout.
4. Actual production regressions establish third-letter requests, conflicting/
   weak/current Known evidence, protected tokens, Space/nonletter inactivity,
   and unchanged manual/field contracts. Prove new regression RED on baseline.
5. If A fails fixed quality, reject it and inspect the shared evidence failure
   before a distinct next hypothesis. Do not install a weaker-quality candidate.
6. Independent cold code review, full affected source/build gates and scoped
   actual Kitty first/second-word tests before release. Native input and source
   proof remain separate. Existing GNOME/KDE field paths are byte-unchanged.

## Execution evidence

- The first development launch was BLOCKED before execution: swap free1310MiB
  versus the configured2048MiB floor. Added an owned temporary2GiB swap file on
  the dedicated worker; retained every resource limit. Remove only that file
  after the work. No desktop/runtime process was stopped.
- The next launch stopped at formatting, not semantics. Applied rustfmt on the
  worker and retrieved the two formatted test/example files.
- Baseline RED:1873 library tests executed,1871passed, exactly the two new
  regressions failed on `djk`, scores+25/-25, both Normal. Receipt:
  `/home/ubu/.cache/lay/development/run-9rtc6m5p/RESULT.json`.
  Production decision code in this snapshot was unchanged.
- Offline runner initially used Python3.11-only `hashlib.file_digest` on the
  Python3.10 worker. Replaced it with streaming SHA-256; no corpus or runtime
  change. The incomplete first directory is retained, not a baseline receipt.
- Candidate A was implemented locally in the declared decision entry. Fresh
  baseline streaming proof completed on the unchanged remote production code:
  RU clean1/10000 false flips, EN clean11/10000; wrong-layout restoration
  RU5235/10000 and EN7575/10000, no repeated flips. Remote receipt:
  `/home/e/projects/lay-development-runner/fragment-early-baseline-20261113-v2/RESULT.json`.
  No live binary, installed configuration, service or learning was changed.
- The first candidate proof is INVALID_BUILD, despite its raw comparator's
  PASS: the restored decision source had an older mtime than the baseline
  artifact. Cargo reported fresh and reused identical executable SHA50fb13c1.
  Source hashes differed but executable bytes did not; exclude this run from
  every quality claim. The runner now cleans only the Lay release package
  (keeps dependency cache) and requires a non-fresh compiled library artifact.
  The original baseline did compile Lay and remains valid. Candidate A was
  rerun with the strengthened build check; no decision threshold changed.

## Candidate A verdict and bounded next experiment

Actual rebuilt A is FAIL_QUALITY: clean false flips RU22/10000, EN35/10000
(baseline1and11); restoration7793/10000RU and8838/10000EN, no lost baseline
targets, no repeated flips. New restorations3821, earlier triggers1279. These
gains do not override45additional clean errors. Remote receipt:
`/home/e/projects/lay-development-runner/fragment-early-candidate-a-20261113-v2/RESULT.json`.
The source patch is retained privately; A is not approved for installation.

Independent cold review scored A4/10 and rejected promotion. It also found an
offline comparator gap: aggregate wrong/repeated counts could hide a new bad
identity. The comparator now rejects per-identity new wrong flips and additional
reversals, including0->2; four regressions cover that guard. Review ran no tests.

Read-only training evidence identifies missing anchored-prefix information:
some high-score false targets have no actual target-prefix support, while
others collide with a real current prefix. djk/вол has training prefix counts
1/280, ghb/при1/1647. Overlapping clipped scores discard those distinctions.
These inspected cases must NOT select another numeric threshold.

One bounded B feasibility experiment is declared before selection. It changes
only the offline example/tooling; installed and packaged models remain intact.
Take the NEXT10000words per language from the hash-ordered training-side pool
after the fixed120000fit words as calibration. Reserve the NEXT10000words per
language after the original10000proof words as a separate untouched evaluation.
The three partitions must be disjoint and input hashes recorded. Enumerate ALL
unmarked A-admitted clean prefixes using compiled production on_letter, without
applying any flip during calibration (a rejected early flip must not hide a
later candidate). Use only exact anchored-prefix records selected by the
existing trainer. Preserve Known decisions; extra admission requires target
prefix count>=20 (existing support floor) and odds(p+4)/(n+4), with the existing
trainer's smoothing4, STRICTLY ABOVE the maximum conflicting calibration odds
in that direction. No grid or manual adjustment. Freeze the policy before
evaluating djk, original proof or untouched proof. A simulated PASS is only
feasibility, not compiled runtime/native acceptance. If djk still abstains or
either evaluation regresses, reject B without tuning the threshold.

Cold plan review8/10 clarifications, frozen before execution: counts come ONLY
from the original fit120000/language; calibration/evaluation cannot add records
or counts. Exact retained anchor means the complete prefix, so additional
admission is limited to3..5letters by existing trainer format. Enumerate both
initial layouts independently. Calibration maximum considers only otherwise
eligible retained anchors with p>=20. A direction with no such calibration
conflicts disables extra admission (null threshold). Use exact rational strict
cross multiplication and filtered streaming decoder updates during evaluation.
Original inspected proof is a regression set; new proof is confirmatory.

## Candidate B verdict and user decision

Frozen odds thresholds were39/1 for a Russian target and89/11 for an English
target, with fit/calibration/evaluation identities and disjointness recorded in
`candidate-b-frozen.json`. No tuning followed either evaluation. The policy SHA
is `d1f391398f9ebb8a13216192634df0669a3188941638778507cdd7d966556db5`.

| Evaluation / measure | Baseline | B | Verdict |
| --- | ---: | ---: | --- |
| Original RU clean false flips /10000 | 1 | 1 | unchanged |
| Original EN clean false flips /10000 | 11 | 12 | FAIL |
| Original RU wrong-layout restored /10000 | 5235 | 5467 | +232 |
| Original EN wrong-layout restored /10000 | 7575 | 7575 | unchanged |
| Fresh RU clean false flips /10000 | 0 | 0 | unchanged |
| Fresh EN clean false flips /10000 | 11 | 12 | FAIL |
| Fresh RU wrong-layout restored /10000 | 5215 | 5428 | +213 |
| Fresh EN wrong-layout restored /10000 | 7673 | 7673 | unchanged |

B made895 existing Russian restorations earlier on the original proof and854
on the fresh proof. There were no new repeated flips or losses of previously
restored wrong-layout words. The user's волшебник example converted on letter3.
However, B newly corrupted one previously correct English identity in EACH
20000-word clean evaluation. Original clean preservation fell from99.94% to
99.935%; fresh clean preservation fell from99.945% to99.94%. These observations
are sample-specific, not a bound on real-world risk. The quality gate is FAIL.

B is only an offline feasibility simulation filtering real compiled A requests;
it is not a compiled runtime/model implementation or a physical-client result.
Final receipt:
`/home/e/projects/lay-development-runner/fragment-early-calibration-b-20261113/RESULT.json`;
local copy `candidate-b-feasibility.json` in the private evidence directory above.
The frozen record's RUNNING status records the pre-evaluation freeze; the final
RESULT is REJECTED_QUALITY. The user explicitly chose rejection, so no further
threshold experiments or release work are active for this task.

## Closure verification

Production decision and letter tests are byte-identical to baselineffe94e1a.
Existing handoff additions are preserved. Only offline diagnostics, task scope,
owning evidence and generated graph metadata may remain in the worktree.
Remote closure checks passed: four per-identity comparator regressions,
repository rustfmt, and a locked release-profile compile check of the simplified
offline example with research-tools. These are tooling checks, not runtime or
native acceptance. Receipt:
`/home/e/projects/lay-development-runner/fragment-early-closure-20261113/RESULT.json`.
Local source and the worker source mirror match after restoration. The initial
closure launcher passed its script through stdin, which the guarded scope did
not forward: it executed no checks and created no receipt. The successful
launcher uses a named script and verifies an explicit RESULT instead.

The mandatory graph refresh follows these final owning-document updates. Its
log, final source/runtime identity check and owned temporary swap removal are
recorded in the private evidence directory's `closure-final.json` and
`closure-checks/`. Existing swap files are outside this cleanup. No broad release
or native-client test is being claimed; the rejected algorithms are not promoted.
