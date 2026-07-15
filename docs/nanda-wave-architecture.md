# NANDA Wave Architecture

> Execution authority: `docs/phase-word-recovery-canonical-cutover.md`.
> This early experimental note is historical context and cannot authorize a
> production route that conflicts with the canonical cutover.

NANDA Wave is an experimental trace/eval architecture for lay. It is not the
current production hot path and must not type into the system directly.

The goal is to test whether small cache-sized wave cells can produce a useful
layered signal:

```text
key/text stream
-> L1 symbol cells
-> L2 word cells
-> L3 phrase/context cells
-> decision trace
```

Runtime integration is allowed only after the eval gate proves that the wave
path is not worse than the current deterministic/NANDA path.

The target lexical model is defined in
`docs/nanda-lexical-attractor-model.md`. In short: L2 must not become a plain
dictionary/prefix lookup with wave-shaped scoring around it. Dictionaries,
Hunspell, protected words, and exact replacements may bootstrap or guard NANDA,
but the NANDA lexical memory itself must be a wave-native lexical-attractor
layer with cold surface-production memory. A number/hash may guide an
attractor, but visible text must be produced through grapheme, morpheme, copy,
layout, or byte-fallback routes, not through a `token_id -> string` shortcut.

## Cell Unit

Canonical cell:

```text
SymbolCell32v0 / NandaCell32v0
size: 32 768 bytes
target: one L1d cache on i7-8650U-class cores
```

Memory layout:

```text
Header               256 B
Projection bank     4096 B
Mode bank          16384 B
Transition bank     4096 B
Interference state  4096 B
Calibration/stats   2048 B
Scratch             1792 B
Total              32768 B
```

The cell does not store every Unicode symbol. It stores a universal projection
from any Unicode scalar / UTF-8 byte pattern into a sparse wave space.

## Mode8

One mode is the smallest memory unit of a cell:

```text
frequency_id  u16
sin_weight    i8
cos_weight    i8
amplitude     i8
phase         i8
damping       u8
role          u8
```

Size:

```text
8 B
```

Capacity:

```text
16 384 B mode bank / 8 B = 2048 modes
```

A mode is not a letter and not a word. A mode is a small frequency-like memory
trace. A symbol excites several modes. Modes interfere. The strongest modes are
emitted upward.

## Top-k Contract

Each cell has many internal modes, but emits only the strongest few:

```text
top-k modes
```

Default:

```text
k = 8 per cell
sparse probes = 64 per cell tick
```

The output is compact:

```text
cell_id
mode_id
role
energy
phase
coherence
```

This keeps the next layer from receiving all 2048 internal modes.

The implementation must not scan all 2048 modes on every symbol in the hot path.
It should use SFFT-style sparse readout: project the stimulus to a small set of
candidate frequencies, score that subset, then emit top-k.

## L1: Symbol Sensors

L1 cells are pretrained and stable. They should rarely learn from user data.
Their role is sensing, not decision-making.

On a 4-core CPU with 32 KB L1d per core, up to four 32 KB L1 cells can be active
in parallel:

```text
Utf8Cell32
ScriptCell32
KeyboardCell32
BoundaryCell32
```

Responsibilities:

```text
Utf8Cell32:
  Unicode scalar, UTF-8 validity, byte length, control/unknown classes

ScriptCell32:
  Cyrillic, Latin, digit, punctuation, emoji/other, case, visual form

KeyboardCell32:
  RU/EN keyboard relation, physical-key direction, Shift/Caps pressure

BoundaryCell32:
  word boundary, whitespace, hyphen, quote, punctuation, token edge
```

L1 input:

```text
current char
previous char
UTF-8 bytes
optional physical key/modifier facts
position in token
```

L1 output:

```text
top-k symbol modes per cell
```

## L2: Word Cells

L2 receives the L1 mode stream and builds local token hypotheses.

Target cells:

```text
WordShapeCell32
LayoutWordCell32
TypoWordCell32
TechTokenCell32
ShortWordCell32
SpaceGlueCell32
CaseWordCell32
UserWordMemoryCell32
```

Future L2 work should converge on the lexical-attractor model:

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

Ordinary word lists and prefix maps are allowed as teachers, bootstrap data, or
safety guards. They are not the NANDA word memory.

Responsibilities:

```text
collect symbol modes into word modes
generate word candidates
estimate layout pressure
estimate typo pressure
detect technical-token risk
detect space/glue risk
```

L2 output:

```text
word candidate
source role
energy
risk
supporting modes
```

L2 may generate candidates, but still must not apply them.

## L3: Phrase / Context / Mesh

L3 receives word candidates and context signals. It decides whether a candidate
is coherent enough to be considered by the safe replacement pipeline.

Target cells:

```text
PhraseCell32
SentenceCell32
MixedLanguageCell32
TechnicalContextCell32
AppContextCell32
UserStyleCell32
UndoRiskCell32
SpaceBoundaryCell32
MeshConsensusCell32
```

Responsibilities:

```text
preserve good neighboring words
protect CLI/technical tokens
detect phrase-level mixed RU/EN context
detect word-boundary risk
detect undo risk
combine cells through mesh consensus
```

L3 output:

```text
Apply(candidate)
KeepOriginal
Veto(reason)
```

## Evaluation Gate

Wave mode starts as trace/eval only:

```bash
lay-nanda-wave-eval --trace "html djn "
lay-nanda-wave-eval --real-suite
lay-nanda-wave-eval --real-suite --ablation
lay-nanda-wave-eval --real-suite --ensemble-sweep
lay-nanda-wave-eval --trace "html djn " --disable-cell MeshConsensusCell32
```

Promotion to runtime is forbidden until:

```text
real-suite wave >= current NANDA
worsened = 0
space/boundary regressions = 0
trace explains L1/L2/L3 contributions
hot path remains off by default
```

## Ablation Contract

A wave cell or layer is considered useful only if disabling it changes a
specific class of decisions in a predictable way.

Current ablation targets:

```text
Utf8Cell32
ScriptCell32
KeyboardCell32
BoundaryCell32
LayoutWordCell32
TechTokenCell32
TechnicalContextCell32
PhraseCell32
MeshConsensusCell32
```

Expected effects:

```text
without KeyboardCell32:
  layout candidates should weaken

without BoundaryCell32:
  word/space boundary failures should increase

without TechTokenCell32 / TechnicalContextCell32:
  technical false positives should increase

without MeshConsensusCell32:
  no candidate should be allowed to apply
```

Until ablation shows useful localized behavior and the real-suite gate is green,
NANDA Wave remains a research path only.

The eval output must make this explicit:

```text
promotion_status: trace_only_do_not_promote
mode_status: ensemble_mode_not_found ...
```

Only a future green gate may print:

```text
promotion_status: gate_green_but_manual_review_required
mode_status: ensemble_mode_candidate
```

## Non-goals

NANDA Wave v0 must not:

```text
replace current lay-daemon hot path
write text into applications
store raw private logs
commit user chat phrases into production logic
use Python for runtime correction
depend on Ollama/LLM
```
