# Phase Word Recovery Baseline 0.2.241

Captured: 2026-07-15.

This is the pre-cutover baseline for
`docs/phase-word-recovery-canonical-cutover.md`.

## Provenance

```text
git_head: 2bf3f76522da1ee6851b8e80dd3a24cb51503ddf
git_describe: v0.2.0-261-g2bf3f76-dirty
package_version: 0.2.241

lay-daemon:
  executable: /home/ubu/projects/lay/target/release/lay-daemon
  sha256: 9987c0b9be516b139eb980f1295e79f48c99010a49c4630893f9c7bd55594844

lay-ibus-engine:
  executable: /home/ubu/projects/lay/target/release/lay-ibus-engine
  sha256: b26f18eefc14e59bf54c722d77dfa33c41406ce76820934f9547457382d15108
```

The running process paths and hashes matched the release binaries.

## Runtime Memory

```text
lay-daemon:
  RSS:           113908 kB
  PSS:           111180 kB
  PrivateDirty:  105036 kB

lay-ibus-engine:
  RSS:           106032 kB
  PSS:           103450 kB
  PrivateDirty:   98812 kB
```

## Real Suite

Command:

```text
target/release/lay-nanda-wave-eval --real-suite
```

Result:

```text
cases:                 1181
deterministic:          869/1181
wave:                   879/1181
wave_delta:             +10
wave_worsened:          89
promotion_status:       trace_only_do_not_promote
```

Important per-class deltas:

```text
layout:                 +6
layout_context:         +5
training_group:        +34
ru_typo:               -17
split_glued_phrase:    -14
missing_letter:         -3
repeated_letter:        -5
```

## L2 Candidate Flow

Command:

```text
target/release/lay-nanda-wave-eval --l2-candidate-flow-report --full-suite
```

Result:

```text
dirty_cases:                    645
cases_with_l2_candidates:       592
no_l2_candidates:                53
expected_candidate_present:     493
expected_candidate_missing:      99
expected_candidate_applied:      372
expected_present_not_applied:    121
```

## Surface L2 Causal Baseline

Command:

```text
target/release/lay-nanda-wave-eval --surface-l2-ablation --full-suite
```

Result:

```text
full:                  18/19
without_surface_typo: 18/19
without_completion:   18/19
without_all_surface:  18/19
surface_only:         11/19
```

This baseline does not prove causal lexical recovery by surface L2. Removing
all surface-L2 cells did not reduce the result on this suite.

## Live Candidate Latency

From the current IME debug trace at audit time:

```text
candidate total:
  p50:  470 us
  p90: 2204 us
  p99: 2944 us
  max: 3849 us

semantic branch:
  p99: 27 us
  max: 30 us
```

End-to-end action latency remained a separate backend/orchestration cost:

```text
decision p50/p90/p99: 42 / 152 / 620 ms
output   p50/p90/p99: 87 / 210 / 336 ms
total    p50/p90/p99: 150 / 300 / 1058 ms
```

## Known Architectural Gaps

1. `L2CenterMemory` contains `source_words: Vec<String>` and returns cloned
   source strings.
2. Phase admission runs after candidate generation and cannot recover a missing
   candidate.
3. The current surface decoder is a `prev2 + prev1 + position` character model,
   not a center-conditioned phase decoder.
4. `verified:true/false` is present in phase input atoms and creates proof
   leakage risk.
5. Runtime candidate sources include proof/synthetic expected data.
6. Hot memory accounting omits full strings and allocation overhead.
7. Live packet L1 and 4-gram center L1 are separate implementations.
8. The causal receipt hash for `src/typing_transition/decision.rs` is stale.

All canonical cutover comparisons must use this baseline or explicitly explain
why a denominator changed.
