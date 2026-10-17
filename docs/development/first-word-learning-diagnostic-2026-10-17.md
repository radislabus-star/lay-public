# `ltkfq` first-word refusal: diagnostic, 2026-10-17

Status: cause reproduced; no production repair or learning cleanup performed.
GitHub maintenance and the earlier compound-typo proposal remain deferred at
the user's direction. This document records diagnosis, not a new runtime plan.

## Finding

The frozen current user feedback contains a state-specific negative transition
for the complete source `ltkfq`. It contains both the target `делай` and
wildcard-target authority-barrier aliases, with stored weight8. There are six
lookup aliases, not six independent user refusals. Weight8 does not establish
eight interactions: `RejectedCandidate` projection assigns weight8 per event.
The precise historical interaction that created this feedback is not recovered
from the retained bounded event journal; do not attribute eight manual undos to
the user.

This is an actual learned refusal for this source state. The decision layer
generates the exact correct candidate and initially admits its layout proof.
L4 then sees a state-specific negative witness and marks the state `Rejected`.
Decision admission rejects that candidate with `reason=rejected`, eventually
producing the `NoApply(Rank)` consumed at committed_tail.rs:140.

The learned identity includes the entire source and replacement text. Thus
`ltkfq` and `проверка ltkfq` have different state identities. The same stored
negative state does not match the prefixed case. This explains the measured
first-versus-second behavior without an ordinal-word-number exclusion.

## Exact baseline and scope

- Authoritative source checkout: `/home/ubu/projects/lay-space-boundary-shift-20261007`,
  branch `codex/space-boundary-shift-20261007`, HEAD
  `031351b10aafd5fe4178d7a14349e73228a1208d`.
- Remote executable diagnosis used e@192.168.3.94 and the existing shared
  dedicated-20cpu/Cargo guards. All717 Rust/Cargo input hashes matched the
  authoritative source. No production Rust source was edited. A temporary
  remote-only example called existing public correction/IME APIs and was
  removed in `finally`; the compiled diagnostic executable is not installed.
- Probe SHA256: `d76ae5c894437f1c09358f8d65b4dbdb9f77b682a81a1f5ed0ea2811210b3cb4`.
- Actual configuration SHA256:
  `20fe9fa76066463698ef4edee29733d9ce61bea7f97c901cf4fce22b03856bf5`.
  Auto replacement, typing assistance and layout switching are enabled.
- Installed/loaded daemon and IME were rechecked during diagnosis and remain
  SHA256 `9b9f7dcab2772f6daf4aa62682df02a967f9c17366c50819fee925d4e646c4f2`
  and `b69b4ba0a0f6977b1608a5cb168534fc029a8ebf5d3e64bc9cd43047ac627e28`.
- Existing remote model artifacts match the local L1.1/L2/phase identities.
  The decisive comparison froze six current learning files into a private
  process HOME, with explicit usage paths and read-only model references.
  Unlike XDG_DATA_HOME alone, this also isolates the HOME-based usage reader.
- The private L1.1 service reported READY before queries and was reaped in
  `finally`. No physical keys, UI focus changes, installs or live restarts.

## Discriminating results

Each row is a fresh diagnostic process using the same source, config, models
and bound lexical frame. These are source/API verdicts, not native client PASS.

| Frozen data / counterfactual | `ltkfq ` | `проверка ltkfq ` |
| --- | --- | --- |
| Current local learning | `NoApply(Rank)`, target reason `rejected` | Selects `проверка делай `; IME plans `делай ` |
| All feedback counts cleared privately | Selects `делай ` | Selects expected layout target |
| Only generic `rejected_words[делай]` removed privately | Still `NoApply(Rank)`, reason `rejected` | Selects expected layout target |
| Only six aliases for the exact negative source state removed privately | Selects `делай ` | Selects expected layout target |

The final counterfactual retains the generic rejected-word weight8 and every
other feedback entry. It isolates the actual blocking source-state witness,
rather than treating a general word-prior penalty as the root cause.
With unchanged current learning, all51 candidates remain in the lattice;
`делай` is `ProductiveL2V90Layout`, `WrongLayout`, initial gate `Eligible`.
The later hidden-state rejection yields0 apply /51 suggest. The prefixed case
has1 apply /50 suggest. Removing only the generic word rejection raises the
target posterior0.133 ->0.272 but does not remove the hidden-state veto.

## Owning source chain

1. `src/transition_relation.rs:40`: signed source/target IDs use every token,
   retaining case and punctuation; the source hash is calculated at line48.
2. `src/nanda_wave/usage_prior/hot.rs:837`: lookup resolves exact state/target
   evidence, then the source-state wildcard-target authority barrier at line842.
3. `src/nanda_wave/l4_hidden_state.rs:239`: a state-specific witness with more
   negative than positive weight classifies its semantic state as rejected.
4. `src/typing_transition/decision/hard_structural_veto.rs:10` and
   `src/typing_transition/decision/admission.rs:185`: enforce this disposition.
5. `src/ime_correction.rs:426` identifies the resulting refusal as Rank;
   `src/bin/lay_ibus_engine/committed_tail.rs:140` consumes it without editing.

Current writer semantics are also relevant: a `RejectedCandidate` event with a
bound source state adds a source-state authority barrier in
`src/nanda_wave/usage_prior/hot.rs:377` and `src/nanda_wave/usage_prior.rs:1423`;
its weight is defined at `src/nanda_wave/usage_prior/projection.rs:215`.
This does not prove which historical event wrote the user's retained entry.

## Evidence and limits

Local private evidence directory:
`/home/ubu/.cache/lay/development/first-word-ltkfq-20261017/`.

- `CAPTURE.json`, `FINDINGS.json`, saved IME/action rings: original passive
  `KnownStart`, current worker/tail identity,51 suggestions and `NoApply(Rank)`;
  bounded ring completeness UNKNOWN. Before-apply logs are not delivery proof.
- `SOURCE-PINS.json`, `RESULT-API.json`: original six API comparisons. The old
  remote learning copy permits both positions, motivating the data comparison.
- `LEARNING-SNAPSHOT.json`, `current-learning-snapshot.tar`: exact frozen user
  data identities; private diagnostic input, never add raw learning to Git.
- `RESULT-FEEDBACK.json`: four current-versus-empty-feedback cases.
- `RESULT-FEEDBACK-REASONS.json`: four traced cases; generic word counter
  removal is a rejected causal explanation, not an implementation rollback.
- `RESULT-STATE-BARRIER.json`: final two narrowly changed data cases.

Remote raw outputs are retained respectively in
`/home/e/projects/lay-development-runner/first-word-ltkfq-20261017-probe2/`,
`first-word-ltkfq-feedback-20261017/`,
`first-word-ltkfq-feedback-reasons-20261017/` and
`first-word-ltkfq-state-barrier-20261017/` under that same runner root.
The initial diagnostic example failed to compile due to two private API
accesses; its first BUILD receipt is preserved in
`first-word-ltkfq-20261017/`. Only the diagnostic example was adjusted.

No corpus-quality, UI latency, universal first-word behavior or client repair
is claimed. No existing admission/learning guard was disabled in production.
Browser acceptance remains the previously complete default432:
Firefox170P/27F/19B; Chrome179P/23F/14B; total349P/50F/33B, NOT_TESTED0.
