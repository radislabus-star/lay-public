# L2 Russian Morphology Phase

Status: `PASS_shadow` on the expanded 462k-form proof. Runtime authority
remains disabled.

## Canonical Package Identity

```text
package_id          LAY-RU-NOUN-MORPH-462K-SHADOW-v1
layer               L2 morphology phase
scope               Russian noun forms and form-to-lemma morphology bindings
corpus artifact     data/morphology/lay_ru_noun_morph_462k_shadow_v1.tsv
manifest            data/morphology/lay_ru_noun_morph_462k_shadow_v1.manifest.json
proof receipt       docs/structural_gates/receipts/L2_RUSSIAN_MORPHOLOGY_EXPANDED_462K_2026-07-24.json
runtime authority   disabled
```

This package is the canonical 462,314-form morphology teacher and proof
artifact. It is not yet the crystallized L1.1 restoration memory. Promotion
requires compiling the same surfaces into real L1.1 `WordCenter64` records and
binding the morphology records to those stable center IDs.

## Shadow Runtime Reuse Inside `L2FieldShadow`

As of 2026-07-26, this package is no longer only an offline teacher. A narrow
runtime bridge now exists at:

- `/home/ubu/projects/lay/src/nanda_wave/morphology_phase/runtime.rs`
- `/home/ubu/projects/lay/src/nanda_wave/l2_field/bridge.rs`

Scope of this runtime reuse:

- shadow-only;
- only on already-born `L2FieldShadow` surface candidates;
- only on Cyrillic local competition;
- only when exactly one same-lemma cohort exists among those candidates;
- outputs only `Winner / Tied / Abstain` over that same-lemma cohort;
- on `Winner`, the promoted shadow candidate is retagged as
  `L2FieldShadowMorphology`.

What was tested for this runtime-reuse step:

- `scripts/cargo-guard.sh check --lib`: passed;
- `scripts/cargo-guard.sh test --lib same_lemma_`: passed;
- `scripts/cargo-guard.sh test --lib l2_field_shadow_route_`: passed;
- `target/debug/lay-nanda-wave-eval --l2-route-compare-report --limit 200 --examples 0`
  on `/home/ubu/.local/share/lay/corrections.jsonl`:
  `records_used = 134`,
  `surface_diverged = 0 / 134`,
  `gate_diverged = 0 / 134`,
  `provenance_diverged = 26 / 134`,
  `compact_apply = 36 / 134`,
  `shadow_apply = 36 / 134`.

What was not tested in this step:

- a fixed heldout proof for same-lemma local competition inside `L2FieldShadow`;
- daemon latency and RSS under live IME load;
- live authority promotion.

Verdict scope:

- the 462k morphology package now donates one real shadow runtime decision
  inside `L2FieldShadow`;
- runtime authority remains disabled;
- parity with `CompactL2` held on the measured real-log replay gate.

Receipt:

- `/home/ubu/projects/lay/docs/structural_gates/receipts/L2FIELD_SHADOW_ROUTE_COMPARE_CORRECTIONS_200_SAME_LEMMA_MORPHOLOGY_2026-07-26.json`

## Ownership

```text
L1 FormCenter
  exact visible surface, for example: дом / дома / домом
        |
        v
MorphBinding16
  FormCenterId + LemmaCenterId + POS + case + number
        |
        v
L2 LemmaCenter
  one lexical identity shared by all learned forms
        |
        v
positive / anti MorphSlot field
  scene -> case-and-number slot
        |
        v
ranked forms -> Winner / Tied / Abstain
```

L1 owns visible word forms. L2 owns the relation between forms of one lemma and
the context-dependent morphology slot. The decoder does not invent endings at
runtime: it materializes an existing learned `FormCenter`.

`pymorphy3` is a cold teacher used only by
`scripts/build-russian-morphology-corpus.py`. Neither Python nor `pymorphy3` is
part of the Lay runtime.

## Compact Records

`MorphBinding16` is exactly 16 bytes:

```text
form_center_id   u32
lemma_center_id  u32
features         u32
support          u16
phase             i8
flags              u8
```

`MorphPhaseCenter64` is exactly 64 bytes:

```text
cells[60]  i8
support    u16
mass       u16
```

The learned slot key is the complete feature mask:

```text
noun x {nom, gen, dat, acc, ins, prep} x {singular, plural}
optional singular extensions: {partitive, second locative, vocative}
```

One globally unique visible surface has one L1 `FormCenter`. Syncretism and
homonymy are represented by multiple 16-byte bindings, not duplicate strings.
One morphology slot can point to several valid surfaces, for example
`лист -> {листы, листья}`. Such surfaces remain tied until a later context layer
can distinguish their lexical sense.

## Learning

```text
Hunspell Russian dictionary
        |
        v
pymorphy3 cold teacher
        |
        v
complete six-case paradigms for every available grammatical number
plus optional partitive / second-locative / vocative evidence
        |
        +--> F: lemma / visible form / case / number
        +--> T: train context evidence
        `--> H: heldout context evidence
                   |
                   v
canonical 60-cell scene wave
                   |
        +----------+----------+
        |                     |
        v                     v
positive subcenters      anti subcenters
        |                     |
        +----------+----------+
                   v
evidence-calibrated minimum positive and margin
```

There are no morphology suffix rules in runtime. Positive and anti banks have
at most four subcenters per slot. The authority threshold is calibrated from
train evidence so that a wrong winner cannot receive authority on the training
split.

Corpus parsing and support updates use indexed
`(form, slot) -> lemma` and `(lemma, form, slot) -> binding` lookups. The first
scale implementation accidentally performed two full scans:

```text
examples x all bindings
```

After replacing both scans with indexes, the identical 300k proof fell from
`12:41.62` to `3.32 s`, about `229x`, with unchanged decisions.

## Expanded 462k Proof

What was tested:

```text
cold teacher                    pymorphy3 over the complete Hunspell input
core acceptance                 all six cases for each available number
one-to-many slots               several valid surfaces per lemma/case/number
number-defective paradigms      plural-only and singular-only nouns
special cases                   partitive, second locative, vocative
heldout                         unseen lemmas
authority gate                  zero false authority
candidate-order control         stratified permutation across all slots
```

Measured corpus:

```text
unique L1 FormCenters             462,314
L2 LemmaCenters                    47,766
MorphBinding16 records            633,016
binding payload bytes          10,128,256
TSV bytes                      78,566,535
dual-number lemmas                 44,717
plural-only lemmas                    259
singular-only lemmas                2,790
plural FormCenters                234,214
singular FormCenters              260,102
slots with multiple surfaces       72,884
maximum surfaces in one slot            5
train lemmas                          128
heldout lemmas                     47,638
train examples                      1,548
heldout examples                  554,148
```

`Top-1` below means that the selected surface belongs to the correct complete
`lemma + case + number` slot. `Exact teacher surface` is reported separately
because one slot can have several valid surfaces.

```text
correct slot top-1            554,148 / 554,148 = 100.000000%
exact teacher surface         549,215 / 554,148 =  99.109805%
authorized correct slot       472,528 / 554,148 =  85.271083%
false authority                     0 / 554,148 =   0.000000%
tied                            81,620 / 554,148 =  14.728917%
abstain                              0 / 554,148 =   0.000000%
candidate permutation parity    3,138 / 3,138   = PASS
verdict                                           PASS_shadow
```

The lower authority percentage is intentional. Multiple surfaces in the same
slot have equal morphology evidence; L2 returns `Tied` instead of selecting one
by hash or candidate order.

Per morphology class:

| Slot | Heldout | Slot top-1 | Authority | False authority |
|---|---:|---:|---:|---:|
| nominative singular | 47,379 | 100% | 83.775512% | 0 |
| genitive singular | 47,379 | 100% | 83.824057% | 0 |
| dative singular | 47,379 | 100% | 83.610882% | 0 |
| accusative singular | 47,379 | 100% | 82.458473% | 0 |
| instrumental singular | 47,379 | 100% | 59.712109% | 0 |
| prepositional singular | 47,379 | 100% | 87.234851% | 0 |
| nominative plural | 44,848 | 100% | 89.571441% | 0 |
| genitive plural | 44,848 | 100% | 97.429094% | 0 |
| dative plural | 44,848 | 100% | 90.195772% | 0 |
| accusative plural | 44,848 | 100% | 86.556814% | 0 |
| instrumental plural | 44,848 | 100% | 90.186853% | 0 |
| prepositional plural | 44,848 | 100% | 90.189083% | 0 |
| partitive singular | 501 | 100% | 100% | 0 |
| second locative singular | 283 | 100% | 100% | 0 |
| vocative singular | 2 | 100% | 100% | 0 |

Performance:

```text
corpus generation elapsed       17.10 s
corpus generation peak RSS     429,252 KiB
kernel compile phase             2.003 s
heldout proof phase              1.905 s
end-to-end proof elapsed         4.55 s
end-to-end proof peak RSS       393,004 KiB
```

Receipt:

```text
docs/structural_gates/receipts/L2_RUSSIAN_MORPHOLOGY_EXPANDED_462K_2026-07-24.json
```

Runtime authority changed: `false`.

## Baseline 300k Proof

Corpus:

```text
unique L1 FormCenters       300,004
L2 LemmaCenters              33,451
MorphBinding16 records      401,412
binding payload bytes     6,422,592
TSV bytes                52,980,013
train lemmas                    128
heldout lemmas               33,323
train examples                1,536
heldout examples            399,876
```

Overall heldout:

```text
top-1 target              398,857 / 399,876 = 99.745171%
authorized target         398,049 / 399,876 = 99.543108%
false authority                 0 / 399,876 =  0.000000%
tied                         1,827 / 399,876 =  0.456889%
abstain                          0 / 399,876 =  0.000000%
candidate permutation parity  4,096 / 4,096 = PASS
verdict                                      PASS_shadow
```

Per morphology class:

| Slot | Cases | Top-1 | Authority | False authority |
|---|---:|---:|---:|---:|
| nominative singular | 33,323 | 99.444828% | 98.922666% | 0 |
| genitive singular | 33,323 | 99.474837% | 98.976683% | 0 |
| dative singular | 33,323 | 99.423821% | 98.901660% | 0 |
| accusative singular | 33,323 | 99.450830% | 98.892657% | 0 |
| instrumental singular | 33,323 | 99.918975% | 99.906971% | 0 |
| prepositional singular | 33,323 | 99.996999% | 99.975993% | 0 |
| nominative plural | 33,323 | 99.915974% | 99.906971% | 0 |
| genitive plural | 33,323 | 99.600876% | 99.453831% | 0 |
| dative plural | 33,323 | 99.996999% | 99.981994% | 0 |
| accusative plural | 33,323 | 99.738919% | 99.648891% | 0 |
| instrumental plural | 33,323 | 99.996999% | 99.981994% | 0 |
| prepositional plural | 33,323 | 99.981994% | 99.966990% | 0 |

Release proof performance:

```text
corpus generation elapsed     10.81 s
corpus generation peak RSS   183,708 KiB
kernel compile phase           1.340 s
heldout proof phase            1.214 s
end-to-end proof elapsed       3.32 s
end-to-end proof peak RSS     263,004 KiB
```

## Honest Boundary

The heldout split contains unseen lemmas but uses the same bounded context-mode
family as training. This proves cross-lemma morphology transfer for the known
case/number modes. It does not yet prove unrestricted Russian sentence
understanding.

The expanded no-anti ablation has the same slot top-1 result, `100%`. The generated
corpus therefore does not prove that anti-memory is necessary. Anti centers are
implemented, but promotion requires a separate conflict corpus where removing
anti evidence causes a measurable regression.

Not tested:

```text
free Russian context outside the bounded teacher modes
semantic selection between same-slot variants such as листы / листья
human validation of every pymorphy3 variant
adjective, pronoun and verb paradigms
runtime latency after integration
IME, auto-apply and AuthorizedEdit integration
```

The vocative heldout denominator is only `2`; its `100%` is measured but is not
a broad vocative-language claim.

The package is shadow-only. It is not connected to IME, auto-apply,
`AuthorizedEdit`, or daemon installation.

## Reproduction

```bash
uv run --with pymorphy3 --with pymorphy3-dicts-ru \
  python scripts/build-russian-morphology-corpus.py \
  --target-forms 300000 \
  --train-lemmas 128 \
  --output data/morphology/lay_ru_noun_morph_462k_shadow_v1.tsv

scripts/cargo-guard.sh build --release \
  --bin lay-morphology-proof --locked

target/release/lay-morphology-proof \
  --corpus data/morphology/lay_ru_noun_morph_462k_shadow_v1.tsv
```
