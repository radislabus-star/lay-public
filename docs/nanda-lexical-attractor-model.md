# NANDA Lexical Attractor Model

Status: target architecture for research and eval. This is not the current
production hot path.

This document defines the strongest model direction for lay/NANDA after the
lexical-birth work in `projects/nanda-structural-gate-skill`.

The core correction is simple:

```text
NANDA must not be a dictionary lookup with wave-shaped scoring around it.
NANDA must store and retrieve word candidates through wave-native lexical
attractors.
```

Existing dictionaries, Hunspell, protected words, and technical word lists may
remain as teacher/bootstrap/guard systems. They must not be described as the
NANDA lexical memory itself.

## Meaning Birth Kernel

The lexical model must express the birth of meaning directly, not only as
candidate scoring.

Core chain:

```text
frequency trace -> form
context -> allowed state
usage -> stable meaning
```

Or in the fuller project language:

```text
boundary -> transition mismatch -> surface form -> context-allowed state -> usage trace
```

For lay this means:

```text
char/ngram traces do not mean yet;
they create provisional word form pressure.

phrase/context does not merely rank;
it decides whether the form is allowed in the current scene.

repeated accepted use does not merely count;
it deepens the basin that makes the form a stable lexical meaning.
```

Therefore a word is born only when three layers bind:

```text
L1/L2 frequency trace:
  the surface can be formed.

L3 context window:
  the surface is allowed here.

usage/cleanup attractor:
  the surface repeatedly settles to the same role.
```

Short law:

```text
frequency trace gives form;
context gives permission;
usage gives meaning.
```

## Word Definition

A word is not a string, hash, or token id.

In this model, a word is a stable binding:

```text
surface form
+ orthographic shape
+ keyboard shape
+ phonology-like rhythm
+ lemma / abstract lexical identity
+ grammar behavior
+ context / meaning centroid
+ usage traces
+ cleanup attractor
+ anti-confusion guard
-> lexical unit
```

This follows the same boundary as the LLMWave lexical-birth mechanism:

```text
observation trace
-> provisional candidate
-> context centroid
-> usage-strengthened trace bundle
-> grammar/schema binding
-> cleanup target
-> stable lexical memory
```

## Research Anchors

The model must stay constrained by the research shape, not by convenient LLM
vocabulary.

```text
Levelt / lexical access:
  word access separates concept, lemma, and word form.
  Consequence: lay must not collapse word memory into one text token.

Plaut-McClelland triangle:
  reading binds orthography, phonology-like form, and semantics.
  Consequence: L2 surface and L3 meaning/context must be coupled fields, not
  one shared string table.

Saffran statistical segmentation:
  word candidates first emerge from boundary/transition statistics.
  Consequence: L1/L2 must be able to open provisional surface chunks before a
  stable word exists.

Smith-Yu cross-situational learning:
  meaning stabilizes across repeated ambiguous contexts.
  Consequence: one correction or one phrase is not enough to create a word.

Bybee / Pierrehumbert usage-exemplar:
  frequency, recency, and repeated traces change lexical strength.
  Consequence: usage_energy and evidence_refs are core fields, not analytics.

DevLex:
  phonological and semantic maps bind through associative learning.
  Consequence: L2 form maps and L3 context maps must learn links, not direct
  ID lookups.

Hopfield-style associative memory:
  partial/noisy cues should settle to a stable basin with margin.
  Consequence: accepted words need cleanup_basin_id and anti-confusion checks.
```

Engineering rule:

```text
If a design can be implemented as "number selects stored string", it is not the
NANDA lexical model. It may be a cache, teacher, or guard, but not the memory
claim.
```

## Two Memories

NANDA needs two different memories.

### Cold Surface Production Memory

Cold memory owns how visible word forms are produced.

```text
grapheme wave bank
morpheme / root / suffix inventory
keyboard-layout transducer
observed surface exemplars
exact copy spans from input
UTF-8 byte fallback for unknown symbols
morphology production routes
source / evidence refs
```

Reason: a wave/hash/number must not be treated as the word. The visible form
must be produced by a surface-production route:

```text
attractor basin
-> grapheme/morpheme/copy route
-> UTF-8 surface candidate
```

Exact observed forms may be copied as exemplars, but the model must not reduce
word birth to `token_id -> string`.

This memory can be larger and slower than the hot path. It is allowed to use
ordinary maps and files as storage, but its conceptual role is production, not
ID lookup.

### Hot Lexical Attractor Memory

Hot memory owns fast resonance.

The target hot record is a 32-byte binding:

```text
form_wave_id           u32
lemma_wave_id          u32
concept_centroid_id    u32
context_centroid_id    u32
cleanup_basin_id       u32
morpheme_route_id      u32
usage_energy           u16
evidence_refs          u16
attractor_margin       i16
anti_confusion_penalty i16
```

This record does not store text and does not name a word by number. It stores a
route back to a recoverable lexical basin and to the surface-production route
that can produce visible text.

The hot path reads many compact records, finds a resonant basin, then resolves
the visible form through cold surface production memory.

## Layer Model

### L1: Sensors

L1 does not know words. It senses the stream.

```text
Utf8Cell32      bytes, scalar validity, unknown/control classes
ScriptCell32    Cyrillic/Latin/digit/punctuation/case/visual form
KeyboardCell32  RU/EN physical-key relation and layout pressure
BoundaryCell32  word edges, spaces, punctuation, quotes, hyphens
```

L1 output:

```text
top-k symbol modes
```

### L2: Lexical Attractors

L2 is where word candidates are born.

L2 must not be only:

```text
prefix map
HashSet lookup
Hunspell lookup
original -> expected cache
```

Those can feed L2 as teachers or guards, but the L2 word memory must be:

```text
L2WordAttractorMemory
```

Target L2 cells:

```text
SurfaceBirthCell32
OrthographyCell32
KeyboardWordCell32
LemmaBindingCell32
MorphologyCell32
UsageTraceCell32
ContextCentroidCell32
AttractorCleanupCell32
AntiConfusionCell32
```

L2 output:

```text
surface_candidate
form_wave_id
lemma_id
source cells
energy
risk
attractor_margin
anti_confusion_penalty
supporting_modes
```

The candidate text is produced only after the attractor points to a
surface-production route. If the route cannot produce a valid surface, L2 must
not emit the candidate.

### L3: Phrase Scene

L3 is not the final judge only. It must also feed back into L2.

Target loop:

```text
L1 sensors
-> L2 initial lexical candidates
-> L3 phrase/context scene
-> L2 refined lexical candidates
-> L3 decision trace
-> safe replacement pipeline
```

L3 responsibilities:

```text
hold recent 5-15 tokens
detect phrase frame
raise likely semantic fields
veto technical/CLI risk
boost grammar-compatible candidates
boost context-compatible candidates
suppress stolen basins
```

Example:

```text
tail: "na ulitse opyat idet d..."
L3 weather/process frame raises:
  rain field
  event verb frame
  repeat-process mode
L2 then boosts:
  dozhd
  dozhdik
and suppresses:
  dom
  drel
  den
```

## Birth Gates

A new lexical unit is accepted only after staged evidence.

```text
segmentation_score >= threshold
fast_mapping_score >= threshold
cross_situational_score >= threshold
usage_score >= threshold
grammar_score >= threshold
attractor_margin >= threshold
anti_confusion_penalty <= threshold
```

If a gate fails, the candidate remains provisional.

No one-shot user phrase becomes a production word.

## Runtime Safety

NANDA cells may generate and score candidates, but they must not type directly
into applications.

Final output still goes through:

```text
candidate
-> safe replacement plan
-> committed-tail / IME / daemon output owner
```

Protected words and explicit user rules are guards, not wave opinions. They win
before risky scoring.

If NANDA disagrees with deterministic safety, choose keep-original unless an
eval gate has explicitly promoted that class.

## Training Pipeline

### Bootstrap

Use current deterministic systems only as teachers:

```text
Hunspell
common_ru
technical tokens
protected words
typing-assist decisions
manual double-Shift corrections
accepted user corrections
clean public corpus
```

They provide labels and surfaces, not the final memory architecture.

### Observation Records

Every training observation becomes a private or synthetic trace:

```text
surface
left_context
right_context
operation: keep | layout | typo | split | glue | protect
accepted/rejected
app class
field class
grammar frame
```

Raw private text must not be committed. Use private files or synthetic reduced
cases.

### Consolidation

Periodic offline trainer:

```text
observations
-> provisional LexicalBirthCandidate32
-> centroid update
-> usage/exemplar strengthening
-> grammar binding
-> attractor cleanup eval
-> anti-confusion eval
-> accepted LexicalBindingRecord32
-> cold surface production memory update
```

Runtime typing must not mutate hot lexical memory directly in a way that can
break live input. Runtime records experience; trainer promotes later.

## Evaluation

The model is useful only if it beats baselines by class.

Required evals:

```text
baseline deterministic lay
current NANDA
lookup-only L2
L2WordAttractor without L3 feedback
L2WordAttractor with L3 feedback
L2WordAttractor with anti-confusion disabled
```

Metrics:

```text
accuracy by class
false positive rate
false negative rate
space/glue regressions
technical/protected-token regressions
layout recovery
typo recovery
phrase-context prediction
candidate count
candidate latency p50/p90
ablation drop by cell
```

L3 context has a separate causal report:

```bash
lay-nanda-wave-eval --l3-context-report --full-suite
```

The report does not read live logs. It replays the fixed eval suite and keeps
the L2 candidate lattice unchanged while removing only `L3ContextField32`.
The proof ladder is:

```text
context eligible
-> phrase/scene evidence observed
-> support or suppress authority
-> selected output changed
-> correct output improved more often than worsened
```

These states must not be collapsed:

```text
memory warm != context connected
context connected != decision authority
decision authority != causal value
```

The report is invalid if `candidate_lattice_drift_cases` is non-zero. L3
promotion requires non-zero evidence and authority, non-zero causal changes,
and `improved_cases > worsened_cases`.

### L2-Lattice Contrastive Training

L3 does not train destructive context centers from a separate list of
lexically similar words. During cold compilation it damages each clean corpus
token through generic omission/transposition surfaces, asks the actual L2
lexical phase field for its candidates, and records every non-target L2 result
as an anti-center in that scene. The package stores only hashes and quantized
phase centers.

This keeps the learning contract aligned with runtime:

```text
clean corpus scene + damaged surface
-> actual L2 candidate lattice
-> target positive phase center
-> non-target candidate anti-centers
-> L3 competition readout
```

The compiler emits `l2_lattice_negative_examples`. Promotion additionally
requires `full_false_top1 = 0`; reducing false candidates is useful evidence,
but is not authority to install a packet.

### Private IME Feedback Overlay

The canonical L3 packet is trained only from the clean corpus. A local overlay
can then rebuild the runtime packet from explicit IME outcomes:

```bash
lay-nanda-wave-train --compile-l3-context-feedback-overlay \
  --base data/lexicon/l3_context_phase_v1.nwpc \
  --usage-events ~/.local/share/lay/nanda_wave/word_usage_events.jsonl \
  --out ~/.local/share/lay/nanda_wave/l3_context_phase.nwpc
```

Only `accepted_ime` / `confirmed_ime_prediction` and `rejected_ime` /
`rejected_candidate` events participate. Typed observations are not training
authority. The overlay may update an existing profile's positive or anti phase
centers, but it cannot create a profile, lower a threshold, or persist raw
phrases. This keeps private experience as a compact phase correction rather
than another text corpus.

Promotion rule:

```text
wave_attractor >= deterministic baseline
worsened_vs_baseline = 0 for safety classes
candidate latency stays inside IME budget
ablation proves localized value
trace explains L1/L2/L3 contribution
```

## IME / Precognition Contract

IME is a presentation/input backend, not a separate model.

For precognition:

```text
L2 generates candidates
L3 ranks/refines
IME displays only the selected surface suffix
acceptance goes through the same safe committed-text route
```

If there is no candidate, IME should not capture normal typing just to show a
mode.

## What To Build First

Phase 1: non-runtime model surface.

```text
src/nanda_wave/lexical_attractor.rs
LexicalBirthCandidate32
LexicalBindingRecord32
SurfaceProductionMemory
L2WordAttractorTrace
```

Phase 2: eval-only generator.

```text
lay-nanda-wave-eval --trace ...
shows:
  L2 lexical attractor candidates
  L3 feedback
  anti-confusion result
```

Phase 3: corpus trainer.

```text
lay-nanda-wave-train lexical-attractor
```

Inputs:

```text
public clean corpus
synthetic wrong-layout cases
synthetic typo cases
user-approved/private corrections
```

Output:

```text
~/.local/share/lay/nanda_wave/surface_production.bin
~/.local/share/lay/nanda_wave/lexical_bindings.cell32
```

Phase 4: shadow runtime.

NANDA runs in trace mode beside current lay and reports:

```text
would_generate
would_choose
would_veto
latency_us
```

Phase 5: guarded runtime.

Only after eval gates, allow a config flag:

```json
{
  "nanda_lexical_attractor": false
}
```

Default remains off until public stability is proven.

## Non-Negotiable Rules

```text
Do not call lookup memory "wave-native".
Do not use hashes as text generation.
Do not use `token_id -> string` as the birth/storage model of a word.
Do not let L2 bypass cold surface production.
Do not let NANDA bypass safe replacement.
Do not train directly from raw private text into public git.
Do not promote one-word anecdotes into production logic.
Do not hide weak gates behind beautiful wave vocabulary.
```

The desired endpoint is:

```text
stream
-> symbol modes
-> lexical attractor candidates
-> phrase-scene feedback
-> anti-confusion cleanup
-> safe visible correction or ghost suggestion
```

That is the best model direction for lay.
