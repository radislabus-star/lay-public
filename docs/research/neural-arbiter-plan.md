# Lay Neural Arbiter Research Plan

Status: experimental research track, not runtime behavior.

## Goal

Build and evaluate a tiny neural scorer for `lay`. The scorer must not generate
text. It ranks already-built candidates and answers whether a candidate is safer
and more natural than keeping the original text.

The desired end state is one of:

- ship a tiny scorer behind an explicit experimental flag;
- extract useful deterministic rules from the trained model and keep runtime
  neural-free;
- reject the neural layer if it does not beat the current rule stack safely.

## Contract

Input row:

- `group_id`: candidate group for the same correction situation;
- `context`: optional left context;
- `original`: what the user typed or what the current rule saw;
- `candidate`: possible replacement;
- `operation`: `keep`, `layout`, `split`, `glue`, `typo`, `mixed`;
- `label`: `1` if this candidate is the intended result, otherwise `0`;
- `source`: fixture or generated source;
- `reason`: why this row exists.

The model returns only a score. It must not create new text.

Runtime safety gates that a neural scorer may not override:

- `protected_words.txt`;
- URLs, paths, emails, CLI flags;
- explicit technical tokens;
- manual double Shift;
- committed Space boundary contract;
- output/backend preflight safety.

## Dataset Sources

The first dataset is built from existing project fixtures:

- typing-assist rule cases;
- split/glue phrase-reader fixtures;
- mixed RU/EN typing-assist fixtures;
- short alternating RU/EN cases;
- live spacing regressions;
- current layout conversion pairs.

Synthetic negatives are generated only from deterministic transforms:

- keep the original when the fixture says correction is needed;
- use the expected output as a wrong candidate for keep fixtures;
- add layout-flipped variants;
- add glued variants for multi-word expected phrases;
- add split variants for single-word candidates.

## Models

Initial models are deliberately small and dependency-light:

- 128/256-dimensional hashed char-field ranker as the first robust baseline;
- trainable char embeddings as the next model only if the hashed scorer proves
  the dataset shape is useful;
- group-softmax scorer over candidates for the same correction situation.

This is a probe, not the final architecture. If it fails, the result is still
useful because it defines dataset and baseline gaps.

## Metrics

The report must include:

- candidate-level accuracy;
- group-level accuracy;
- false positive rate where `keep` was the correct answer;
- false negative rate where a correction was the correct answer;
- timing per candidate and per group;
- worst mistakes.

## Nanda/Grokking Audit

After training, inspect whether the model learned structure rather than only
memorizing fixtures:

- compare 128d vs 256d;
- measure layout-pair embedding similarity versus random pairs;
- measure RU/EN centroid separation;
- inspect worst mistakes;
- keep the model out of runtime unless the audit supports generalization.

## Completion Criteria

This research track is complete only after `docs/research/neural-arbiter-results.md`
states one of the three decisions: ship behind flag, extract rules, or reject.
