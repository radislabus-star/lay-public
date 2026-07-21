# Lay Self Teacher L3 Plan

Status: offline teacher implemented, smoke shadow PASS after support schedule fix

Baseline from 2026-07-21:

```text
L3 evidence hit: 9/24 = 37.5%
L3 authority:    4/24 = 16.7%
output_changed:  0
improved/worsened: 0/0
verdict: L3_CONTEXT_OBSERVED_NOT_DECISIVE
```

Goal:

```text
lay-self-teacher turns L3 from observed context evidence into decisive context
authority.
```

## Current Checkpoint

Command:

```bash
lay-nanda-wave-eval --lay-self-teacher-l3 --max-phrases 80 --max-pairs 600 --out-dir /tmp/lay-self-teacher-l3-smoke
```

Useful options:

```text
--clean-corpus PATH      add an external clean phrase source
--usage-events PATH      add accepted local feedback events
--no-live-feedback       ignore default local usage feedback
```

Default local feedback source, when present:

```text
~/.local/share/lay/nanda_wave/word_usage_events.jsonl
```

Only accepted/confirmed outcomes are allowed into the clean teacher corpus.
Rejected events remain telemetry unless they carry an observed final target.

Previous smoke:

```bash
lay-nanda-wave-eval --lay-self-teacher-l3 --max-phrases 160 --max-pairs 1200 --out-dir /tmp/lay-self-teacher-l3-smoke
```

Result:

```text
clean_phrases:       160
dirty_pairs:         1200
surface_rows:        839
surface_admitted:    424
surface_modes:       27
semantic_states:     369
candidate_profiles:  109
pair_profiles:       291
artifact_bytes:      161,932

shadow cases:        491
evidence_hit:        183 / 491 = 37.27%
authority:           168 / 491 = 34.22%
output_changed:      168 / 491 = 34.22%
false_authority:     0 / 491 = 0.00%
target_top1:         204 / 491 = 41.55%
false_top1:          287 / 491 = 58.45%
verdict:             WATCH_shadow
runtime_authority:   false
runtime_installed:   false
```

Read this correctly:

```text
false_authority = 0 is good: L3 is not authorizing wrong candidates.
target_top1 is weak: raw energy still often ranks the wrong candidate first.
```

The next L3 work is not to add correction rules. It is to improve target
energy/top-1 inside the learned context field while preserving false_authority
at zero.

Latest smoke after fixing corpus support schedule:

```bash
lay-nanda-wave-eval --lay-self-teacher-l3 --max-phrases 160 --max-pairs 1200 --out-dir /tmp/lay-self-teacher-l3-smoke-2
```

Result:

```text
clean_phrases:              160
dirty_pairs:                1200
corpus_support_repeats:     3
corpus_fragments:           480
surface_rows:               839
semantic_states:            369
candidate_profiles:         319
artifact_bytes:             331,504
elapsed_millis:             10,520

shadow cases:               491
evidence_hit:               491 / 491 = 100.00%
authority:                  480 / 491 = 97.76%
output_changed:             480 / 491 = 97.76%
target_top1:                491 / 491 = 100.00%
support_target_top1:        491 / 491 = 100.00%
false_top1:                 0 / 491 = 0.00%
support_false_top1:         0 / 491 = 0.00%
false_authority:            0 / 491 = 0.00%
candidate_order_changed:    0
verdict:                    PASS_shadow
runtime_authority:          false
runtime_installed:          false
```

Why it changed:

```text
Context phase profiles are admitted only after repeated support.
With min_profile_support=2, a profile is born on the second observation, so a
two-line corpus repeat gives only one positive example after birth. The clean
teacher corpus must therefore repeat each clean phrase 2 * min_support - 1
times. Default min_support=2 means 3 repeats.
```

This is a training support fix, not a word-specific correction rule.

## Route

```text
clean corpus
-> synthetic dirty variants
-> same L1/L2/L3/L4 pipeline
-> expected clean target
-> positive and negative feedback
-> local phase / anti-phase / usage / L4 memory
-> shadow proof
-> runtime package only after PASS
```

This is local Lay training. It is not OpenAI fine-tuning and it must not send
typed text to external LLM providers.

## Work Items

1. Create `lay-self-teacher` as an offline trainer.
   It must feed dirty/clean pairs through the same correction pipeline used by
   Space autocorrect and IME candidate generation.

2. Build dirty generators from error classes, not word-specific rules:
   adjacent transposition, missing letter, extra letter, letter substitution,
   layout projection, premature space, glued words, split words, punctuation
   suffixes and noisy mixed RU/EN tails.

3. Build clean phrase sources:
   local clean corpora, accepted live phrases, curated short Russian phrases and
   technical mixed phrases. Dirty live text is allowed only as feedback/eval, not
   as a clean teacher source.

4. Store training as compact evidence:
   phase centers, anti-centers, usage priors and transition signatures. Raw
   corpus text is not a hot runtime authority.

5. Train L3 context on whole phrase windows:
   the target is not only the corrected word, but the scene in which the word is
   allowed. L2 births candidates; L3 must choose between plausible candidates.

6. Compile candidate-specific L3 anti-centers:
   wrong candidates from the real L2 lattice become destructive interference for
   the same scene.

7. Add shadow proof:
   compare without-L3 vs with-L3 over the same L2 lattice. A valid improvement
   requires changed decisions caused by L3, not changed candidate generation.

8. Promotion gate:

```text
evidence_hit_percent: higher than baseline
authority_percent: higher than baseline
output_changed: > 0
improved_cases: > worsened_cases
false_top1: 0 or strictly lower with no new unsafe edits
left_context_changed: 0
multiword_touch_without_boundary_proof: 0
runtime p99: acceptable
```

9. Live smoke only after offline PASS:
   test IME visibility, Space autocorrect and Tab accept in a controlled window.

## Reminder

Next architecture front after the current L2/L4 work:

```text
Build lay-self-teacher and use it to make L3 context decisive.
```
