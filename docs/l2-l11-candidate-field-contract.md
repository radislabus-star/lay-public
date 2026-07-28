# L2 Candidate Field Contract Above L1.1

Status: architecture contract for the intended `L1.1 -> L2 -> L3 -> verifier`
route. This document records the role split and the required qualities of the
new `L2` candidate field. It does not grant runtime authority by itself.

## 1. Role Split

```text
L1.1
  broken surface
  -> lexical center winner | tied | abstain

L2
  L1.1 candidate lattice
  + local phrase context
  + morphology slot evidence
  + neighbor couplings
  -> ordered local candidate field
  -> winner | tied | abstain

L3
  broader phrase / semantic context
  -> boosts, suppresses, or keeps L2 ties unresolved

verifier
  checks whether the chosen transition is structurally safe to apply
```

`L1.1` restores the damaged signal. `L2` decides which restored form belongs to
the local phrase scene. `L3` must not replace `L2`; it only works on top of the
real `L2` field.

## 2. What L1.1 Does Not Own

`L1.1` is not the owner of:

- phrase-local ending choice;
- lemma-internal form choice such as `посмотреть / посмотри / посмотрим`;
- neighbor competition such as `посмотри / просмотри / подсмотри`;
- morphology-slot selection from local context;
- multiword boundary decisions;
- destructive edit authority.

Those responsibilities belong to `L2` and the downstream verifier.

## 3. L2 Inputs

The new `L2` candidate field must read:

1. the current token and its visible boundaries;
2. the bounded `L1.1` lattice, not only the single top-1 winner;
3. left and right local context windows;
4. punctuation, separators, and boundary markers;
5. local continuation signals from the live input path;
6. form-to-lemma bindings and morphology-slot memory;
7. deterministic layout/boundary support when those routes remain enabled.

## 4. Required L2 Qualities

### 4.1 Lattice Preservation

`L2` must consume the full bounded `L1.1` candidate lattice:

- keep `winner / tied / abstain` semantics from `L1.1`;
- preserve candidate identity and source attribution;
- never collapse the field into one candidate before local competition runs.

### 4.2 Center-Based Reasoning

`L2` must reason over learned identities, not raw strings:

- `FormCenter` for visible surface forms;
- `LemmaCenter` for lexical identity families;
- bindings between forms, lemmas, and local slots.

### 4.3 Local Context Awareness

`L2` must see phrase-local context strongly enough to choose endings and near
neighbors:

- service words, particles, and prepositions;
- one or two lexical neighbors on each side;
- word order and adjacency patterns;
- punctuation and phrase boundaries.

### 4.4 Morphology Slot Inference

`L2` must infer the local slot that the scene demands. For Russian this
includes at least:

- case;
- number;
- gender;
- person;
- tense;
- mood;
- aspect;
- infinitive vs finite vs imperative form.

`L2` must then compare candidates against that slot rather than only against
surface similarity.

### 4.5 Intra-Lemma Competition

`L2` must hold competing forms of the same lexical family in one field:

```text
посмотреть
посмотри
посмотрим
посмотрел
```

This competition is core `L2` work and must not be delegated to `L3`.

### 4.6 Cross-Lemma Neighbor Competition

`L2` must also handle neighboring lexical centers that are geometrically close:

```text
посмотри
просмотри
подсмотри
досмотри
```

The field must rank or tie them using local phrase evidence, not only edit
distance.

### 4.7 Pairwise Competition

`L2` must support candidate-vs-candidate relations, not only one global score.

Examples:

- candidate A beats B because the local mood is imperative;
- candidate B beats C because the governing neighbor expects another slot;
- candidate C remains tied with D because the local scene is insufficient.

### 4.8 Neighbor Coupling

`L2` must learn and use short-range couplings:

- preposition -> required case;
- auxiliary / particle -> expected verb form;
- adjective / noun agreement cues;
- stable two-word and three-word local motifs;
- bounded local order patterns.

### 4.9 Live Continuation Sensitivity

`L2` must react when the user continues typing after a visible proposal. If the
next typed letters move the field away from the current winner, `L2` must
re-rank or drop that winner instead of insisting on an outdated completion.

### 4.10 Honest Tied And Abstain

`L2` must be able to return:

- `Winner` when the local field is decisive;
- `Tied` when several candidates remain valid in the same local slot;
- `Abstain` when the scene does not justify a safe local choice.

It must never invent false certainty to avoid returning `tied` or `abstain`.

### 4.11 Deterministic Readout

The same local scene and the same `L1.1` lattice must produce the same bounded
candidate order. IME and autocorrect must consume that same order.

### 4.12 Attribution And Evidence

For every emitted candidate, `L2` must preserve:

- candidate source;
- lexical center identity;
- local slot evidence;
- pairwise suppressions and supports;
- tie or abstain reason when no unique winner exists.

### 4.13 Bounded Runtime

The field must stay bounded:

- bounded number of candidates;
- bounded pairwise competition edges;
- bounded slot/profile lookups;
- bounded hot-path latency.

### 4.14 Learnability

`L2` must be trainable from real local scenes:

- which local slots actually select which forms;
- which neighbors support or repel a candidate;
- which same-family forms remain tied in practice;
- which false local winners must be destroyed by anti-relations.

## 5. Minimal L2 Memory Objects

The new `L2` should converge to a memory shaped roughly like this:

```text
L2 Candidate Field Memory
|
+-- FormCenter
|   visible learned surface
|
+-- LemmaCenter
|   lexical identity shared by forms
|
+-- MorphBinding
|   FormCenter <-> LemmaCenter <-> slot
|
+-- LocalContextMode
|   bounded phrase-local scene key
|
+-- SlotPhaseCenter
|   local slot evidence field
|
+-- CandidateCompetitionEdge
|   candidate A supports / repels candidate B
|
+-- Tie / Abstain Calibration
    honest local readout thresholds
```

This is the intended evolution of `L2` above canonical `L1.1`.

Internal package and runtime architecture:
`/home/ubu/projects/lay/docs/l2-l11-canonical-architecture.md`.

## 6. L2 Outputs

The field must emit:

1. an ordered bounded candidate lattice;
2. one of `Winner / Tied / Abstain`;
3. source and evidence attribution for each surviving candidate;
4. no direct destructive edit authority.

The public mutation route remains:

```text
L1.1 restore
-> L2 candidate field
-> L3 / L4 / Bayes support or suppression
-> verifier
-> AuthorizedEdit or no-op
```

## 7. Forbidden L2 Behaviors

The new `L2` must not:

- replace `L1.1` as the raw damaged-surface restorer;
- skip the candidate lattice and force a single top-1 too early;
- depend on broad sentence semantics as its primary signal;
- directly apply text edits;
- hide uncertainty behind an arbitrary local winner;
- drift into a second IME-specific brain.

## 8. Short Contract

In one sentence:

> `L1.1` restores damaged word signals into lexical candidates, and `L2`
> converts those candidates into a bounded local competition field that chooses
> the right form, ending, and near neighbor for the phrase scene, or honestly
> returns `tied / abstain`.
