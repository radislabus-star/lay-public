# L4 Multilingual Scene Field V2

Status: Lay 1.0.30 V2 organic shadow deployment PASS; live authority unchanged
Date: 2026-08-15
Runtime authority changed by this document: no

## 1. Conclusion

The current L4 cross-scene memory is too weak for multilingual correction.
This is an identity and evidence problem, not a coefficient problem.

V1 collapses language, layout, script, and keyboard geometry into a RU/EN
direction plus coarse script agreement. It therefore cannot distinguish
English, German, French, and Spanish Latin text, cannot represent a sentence
language independently from the current token, and cannot learn a generic
layout route without treating the layout name as a language name.

The target route is:

```text
L1 per-language candidate fields
-> L2 morphology and local competition
-> L3 sentence-language compatibility
-> L4 typed cross-language causal scene
-> one TransitionDecisionCore
-> verifier
-> AuthorizedEdit or no edit
```

L4 is an evidence producer in this route. It does not birth words, own a
second candidate ranking, mutate text, or bypass the verifier.

## 2. Measured V1 Baseline

Measured locally on 2026-08-15 from the installed package and journals:

```text
package path       /home/ubu/.local/share/lay/nanda_wave/l4_cross_scene_v1.bin
package bytes      13,228
profiles           16
pair profiles      58
source rows        186
joined rows        186
positive           10
reverted           176
negative           0
ambiguity          0
runtime authority  shadow_suggest_only
automatic apply    false

usage journal      1,605 rows, 511,767 bytes
correction journal 2,958 rows, 392,026 bytes
V1 package mtime   2026-08-10 07:51:28 +0300
latest usage mtime 2026-08-15 18:27:54 +0300
```

Measured code facts:

- `KeyEvent` carries `layout_is_ru: bool`.
- `LayoutProjectionDirection` has only RU/EN and mixed RU/EN variants.
- V1 scene encoding classifies only Cyrillic, ASCII Latin, mixed, or neither.
- Context compatibility is script agreement over at most four recent tokens.
- A V1 profile key contains only operator, RU/EN direction, and edit scope.
- V1 `automatic_apply()` always returns `false`.
- Runtime invokes `shadow_readout`; V1 is diagnostic, not ranking authority.
- The standard rebuild is coupled to version publication, so the installed
  package is older than its bounded source journals.
- The usage journal is not append-only: at 500 KiB it rewrites itself to a
  recent complete-line tail. During preflight its size changed from 511,990 to
  511,930 bytes. A byte cursor over this file is therefore invalid.

## 3. Terms That Must Not Be Collapsed

```text
LanguageId
  linguistic identity of a lexical or sentence field, for example ru or de

LayoutId
  keyboard map identity, for example xkb:us or xkb:ru

ScriptFamily
  writing-system family, for example Latin or Cyrillic

KeyboardGeometryId
  physical geometry shared by one or more layouts, for example pc105
```

Wrong-layout text demonstrates why these are distinct. `ghbdtn` is Latin
script produced through a US layout, but it is not reliable evidence that the
surface language is English. Its Russian candidate can carry target language
`ru`, target layout `xkb:ru`, target script Cyrillic, and shared physical
geometry `pc105` without inventing an English-language claim.

## 4. Generic Identity Contract

V2 introduces these compact types:

```text
LanguageId(u64)
LayoutId(u64)
KeyboardGeometryId(u64)
ScriptFamily(u8)
```

The three `u64` identities are deterministic hashes of canonical package
labels in separate domains. Zero means unknown. The package stores a bounded
symbol registry containing `(kind, id, canonical_label)` and rejects duplicate
labels, duplicate identities with different labels, invalid labels, and
references to absent symbols. This makes IDs compact on the hot path while
keeping packages inspectable and collision-checked.

Canonical labels are data, not Rust variants. Adding a language, layout, or
geometry therefore requires a package entry and proof corpus, not a new runtime
branch. V2 reserves no runtime condition for a literal word, suffix, phrase,
or language-specific example.

`ScriptFamily` is a small universal enum because scripts are structural
properties. Initial values are `Unknown`, `Latin`, `Cyrillic`, `Greek`,
`Armenian`, `Georgian`, `Hebrew`, `Arabic`, `Han`, `Kana`, `Hangul`, `Mixed`,
and `OtherAlphabetic`. Script is never promoted to language identity.

## 5. Typed Scene Route

Every candidate-relative L4 input carries:

```text
source_language
target_language
source_layout
target_layout
source_script
target_script
keyboard_geometry
identity_evidence
sentence_language
sentence_language_support
sentence_language_alternative_support
sentence_language_observed_tokens
operator
layout scope
L1/L2 relation identity
L3 relation class
causal outcome, for training only
```

`identity_evidence` records whether the route came from explicit package
metadata, a legacy RU/EN adapter, script-only observation, or is unknown. An
unknown field remains unknown. The encoder must not silently turn Latin into
English or Cyrillic into Russian.

Sentence-language evidence belongs to L3. L4 consumes the bounded typed
readout; it does not rescan a dictionary or run another sentence model.

## 6. Journal V3

New typing rows gain schema V3 fields:

```text
source_language
target_language
source_layout
target_layout
source_script
target_script
keyboard_geometry
identity_evidence
sentence_language
sentence_language_support_milli
sentence_language_alternative_milli
sentence_language_observed_tokens
```

Journal rows store canonical labels as durable identities. Numeric package IDs
are compiler products and are not persisted as the only identity.

Compatibility rules:

1. V1 rows remain readable as untyped legacy observations.
2. Valid V2 rows remain readable and are adapted from
   `LayoutProjectionDirection` without asserting unsupported language facts.
3. V3 rows require label/code parity for every present typed field.
4. Missing fields become `Unknown`; they never inherit the previous row.
5. Malformed V3 identity is rejected, not downgraded to V2.
6. Censored events remain non-negative and do not train L4 authority.
7. Only a complete causal episode may update positive, negative, reverted, or
   ambiguity banks.

The existing usage journal remains a bounded compatibility and hot-prior
source. Schema migration appends V3 rows and does not rewrite old rows merely
to upgrade their schema, but normal size compaction may remove an old prefix.
That compaction must become atomic and preserve whole causal episodes, not
only whole JSON lines.

L4 incremental learning therefore does not use a byte cursor over the bounded
usage journal. It receives complete episode envelopes through a separate
transactional inbox. An envelope contains every row in one `episode_id`, its
row count, and a content checksum. Partial envelopes are censored.

## 7. Package V2

V2 keeps V1 readable and writes a new file, initially
`l4_cross_scene_v2.bin`. It does not overwrite the installed V1 package during
development.

The package contains:

```text
header
symbol registry
scene-route profiles
candidate-pair profiles
quantized phase centers
applied episode-segment checkpoint
checksum
```

The scene-route profile key is:

```text
operator
scope
source LanguageId
target LanguageId
source LayoutId
target LayoutId
source ScriptFamily
target ScriptFamily
KeyboardGeometryId
identity evidence class
sentence LanguageId
sentence-evidence bucket
```

Budgets:

```text
package bytes              <= 16 MiB
symbols                    <= 512
profiles                   <= 512
pair profiles              <= 4,096
centers per ordinary bank  <= 4
centers per hard bank      <= 2
hot readout                no heap allocation after package load
incremental update input   complete new episode envelopes only
inbox segment              <= 256 KiB, immutable after seal
unacknowledged inbox       <= 16 MiB, then learning pauses visibly
hot episode seal queue    capacity 64, worst-case payload <= 16 MiB
package publisher         one nonblocking owner; all pending segments batched
```

V2 package publication is atomic: write and fsync a private temporary file,
validate it through the runtime reader, rename it, then fsync the directory.
A failure leaves the previous valid package and all journal rows intact.

## 8. Bounded Incremental Updater

The updater is independent from version publication and runs only when the
transactional inbox contains new complete causal episodes.

```text
read highest applied immutable segment from the package
-> read later sealed segments only
-> validate complete V3 episode envelopes and segment checksums
-> merge bounded sufficient statistics
-> compile candidate V2 package
-> run format, parity, and shadow proof
-> atomic publish or keep previous package
-> package and applied-segment checkpoint become visible in one atomic rename
-> delete only segments acknowledged by the published package
```

It must not poll and rebuild the entire journal every five seconds. A service
timer or path trigger may coalesce bursts, but only one updater instance may
own publication. The hot input path only hands a complete envelope to the
bounded spool writer; it never compiles a package synchronously. If the inbox
reaches its disk budget, learning pauses with an explicit status instead of
silently dropping an episode or deleting unacknowledged evidence.

## 9. Authority Contract

V2 begins as shadow-only. Promotion is not a boolean enabled by package
presence.

```text
L4 supported
  -> calibrated positive evidence passed to the one DecisionCore

L4 repelled
  -> calibrated contradictory evidence passed to the one DecisionCore

L4 tied or unknown
  -> no authority change

DecisionCore selected candidate
  -> verifier proves edit plan and layout route
  -> AuthorizedEdit

verifier rejects
  -> no edit
```

A grounded L1.1 winner can be downgraded only by independently measured
contradictory evidence. L4 uncertainty, a missing profile, or an unrelated
language basin is not contradictory evidence.

## 10. Fixed Proof

The fixed proof must report separate denominators:

```text
candidate retained by L1/L2
sentence language known by L3
L4 profile present
L4 supported correct route
L4 repelled false route
L4 tied or abstained
DecisionCore winner
verifier accepted
automatic edit applied
false authority
false language switch
```

Required promotion gates:

```text
V1 read parity                         100%
V2 deterministic byte roundtrip       100%
V2 compiler/runtime encoder parity     100%
complete-episode replay parity         100%
old journal prefix preserved           100%
false automatic language switch        0
false authority                        0
per-language clean preservation        >= 99.9%
per-route lattice target retention     >= 99.0%
hot p99 incremental L4 cost            <= 0.5 ms
whole Space contour p99                 <= 5.0 ms
package                                <= 16 MiB
```

Quality remains `UNKNOWN` for a language until that language has all four:

```text
layout mapping
lexical/morphology package
L3 sentence corpus
fixed per-language and cross-language heldout proof
```

A third language cannot be claimed from generic V2 types alone.

## 11. Implementation Order

1. DONE: add generic identity types and a legacy RU/EN compatibility adapter.
2. DONE: add V3 event emission and V1/V2/V3 reader parity tests.
3. DONE: add the V2 symbol registry, profile key, encoder, reader, and writer
   while preserving the V1 reader.
4. DONE: add candidate-relative L3 sentence-language evidence input.
5. DONE for focused tests: add sealed episode segments, durable monotonic
   sequence identity, package checkpointing, rollback, and atomic publication.
6. DONE: run the RU/EN fixed shadow proof on the reference host, including
   deterministic package, reader/runtime parity, anti ablation, and hot readout.
7. DONE: install the organic V2 package beside V1 and load it in the
   `lay-daemon` shadow route without restarting global or managed IBus.
8. Select and package one real additional language.
9. Consider live evidence promotion only after every gate above passes.

## 12. Explicit Non-Goals

- no rewrite of every `layout_is_ru` callsite in this change;
- no literal word or language-example runtime branches;
- no second candidate chooser;
- no weakening of SafetyGate, edit-plan validation, or verifier;
- no synchronous package rebuild on a key or Space event;
- no global IBus or managed-engine restart during architecture work;
- no claim that script agreement proves sentence language;
- no automatic apply from sparse V1 or unproved V2 memory.

## 13. Current Verdict

```text
V1 package integrity                 measured PASS
V1 multilingual language identity   FAIL by architecture
V1 organic promotion evidence       WATCH insufficient evidence
V2 typed format and V1 compatibility focused PASS
V2 segment/checkpoint updater        focused PASS
V2 CLI/status control surface        smoke PASS
V2 RU/EN typed layout shadow proof   PASS_SHADOW, 10/10 classes
V2 additional-language quality       UNKNOWN, no third-language corpus
V2 hot readout p99                    4.061 us, PASS against 500 us gate
V2 organic package                    21,445 bytes, reader PASS, shadow only
V2 organic promotion evidence         WATCH, zero complete negative observations
installed Lay                         1.0.30, daemon and GNOME runtime active
managed IBus engine                   preserved old inode, restart pending
runtime authority changed           false
```

## 14. Implementation Evidence, 2026-08-15

Tested:

```text
focused no-run compile               PASS, 33.54 s
focused L4 suite                     27 passed, 0 failed
test execution                       0.02 s after compilation
lay-nanda-wave-train build           PASS
empty V2 package CLI smoke           PASS, 84 bytes
V2 encoder identity                  2 / 0x4c344d554c544932
empty inbox status                   sequence_floor 0, next_segment 1
empty incremental update             NOOP, checkpoint 0 -> 0
automatic apply in update proof      0
runtime authority changed            false
```

The focused suite covers strict V3 adaptation, V1 byte compatibility, V2
registry and checksum validation, immutable segment checksums, monotonic
segment identity after deletion, old-plus-new weighted center retention,
checkpoint advancement exactly once, failed-segment rollback, runtime readout
parity, and the invariant that L4 cannot grant automatic apply authority.

Not tested in this checkpoint:

- fixed heldout quality for any language beyond RU/EN;
- whole live L1 -> L2 -> L3 -> L4 phrase-route quality;
- clean preservation and lattice retention denominators;
- whole Space contour p99;
- process-kill fault injection at every fsync/rename boundary;
- shadow installation, daemon behavior, or physical IME input.

Verdict scope: focused format, ingest, merge, checkpoint, rollback, CLI, and
authority-boundary correctness only. It is not a multilingual quality proof or
a promotion receipt. The installed V1 package remains 13,228 bytes with
SHA-256 `5a32cf50b94105679ec40bec7bd5c46c2937075ede864bd7961203427a6cf1b5`.

Remote RU/EN proof iterations on `e@192.168.3.94`:

```text
1. V1-shaped negative matrix
   WATCH: 8/8 classes at 100%, but anti ablation was not exercised
   reason: typed V2 keys correctly separated the reverse language routes

2. Three-position same-route punctuation control
   REJECTED: anti prevented 204 false supports but also suppressed all
   204 whole-token positive cases

3. Eight-position same-route negative control with hot readout measurement
   PASS_SHADOW: 640 cases, 10/10 classes at 100%
   false supports 0, automatic apply 0
   without anti false supports 204, with anti false supports 0
   positive supports retained 218/218
   hot readout p50/p99/max 3.023/4.061/7.572 us over 20,480 samples
```

The proof package is `6,469` bytes with `8` profiles, `16` pair profiles,
`5` typed symbols, encoder V2, deterministic bytes, exact reader roundtrip,
candidate-order parity, and runtime evaluator parity. The final proof took
`0.31 s`, used `22,440 KiB` peak RSS, and changed no runtime authority. This is
strictly a RU/EN typed layout-route proof; it does not establish German,
French, Spanish, morphology, semantic truth, or live automatic correction.

## 15. Organic V2 Compile Evidence, 2026-08-15

Tested on `e@192.168.3.94` from private snapshots copied before the proof:

```text
usage input rows                    1,608
correction input rows               2,958
live source observations            1,608
joined observations                   208
ignored observations                1,580
positive observations                  28
reverted observations                 180
complete negative observations           0
conflict scenes                         8
consolidated scenes                     68
profiles                               26
pair profiles                          67
symbols                                 5
raw text stored                     false
package bytes                       21,445
compile elapsed                       0.01 s
compile peak RSS                     5,948 KiB
reader/status                         PASS
runtime authority changed            false
```

The organic package is safely below the `16 MiB` budget and passes the V2
reader, encoder identity, checksum, and bounded-count checks. It is not a
quality or promotion proof. The current organic journal contains no complete
negative observations, so it cannot establish false-route repulsion or grant
live authority. It may be staged only as an inactive or shadow-only V2 package.

Remote artifacts:

```text
package  /home/e/build/lay-l4-v2-20260815/artifacts/l4-v2-20260815/l4-cross-scene-organic-v2.bin
report   /home/e/build/lay-l4-v2-20260815/artifacts/l4-v2-20260815/l4-cross-scene-organic-v2.json
status   /home/e/build/lay-l4-v2-20260815/artifacts/l4-v2-20260815/l4-cross-scene-organic-v2-status.json
sha256   1a1e926c4b4c972add54ce3235b1f1527276365abe74952abb538a3132c97e3e
```

## 16. Lay 1.0.30 Shadow Deployment, 2026-08-15

The release was built once on `e@192.168.3.94` after the version bump. The
Cargo build itself passed; an initial post-build wrapper incorrectly required
`--version` from utilities that do not implement that flag. The binary bytes
were retained and the manifest gate was corrected without rebuilding.

```text
release build                         PASS, exit 0
release build elapsed                 3:19.21
release build peak RSS                2,499,724 KiB
release binaries                      10
release binary bytes                  45,963,568
installed binary hash parity          10/10
focused L4 tests                      27/27
authority contract tests              20/20
mutation monopoly tests               16/16
cargo check --lib --bins              PASS
organic V2 package                    21,445 bytes
organic V2 reader                     PASS
automatic apply                       false
daemon version                        1.0.30
daemon PID                            150902 -> 428759
daemon restart count                  0
daemon package environment            l4_cross_scene_v2.bin
GNOME DBus version                    1.0.30
global IBus PID                       3702 -> 3702
managed engine PID                    3719399 -> 3719399
runtime authority changed             false
```

The changed-route gate also exercised the adjacent context owners. The two L3
ranking failures are not a V2 regression: the exact pre-change commit and the
current tree fail on the same two contracts with the same denominator.

```text
baseline commit                       b8b85281ddb45557a7ecce7c81eb7c0ca5aa5a6c
baseline L3 ranking                    54/56, 2 FAIL
current L3 ranking                     54/56, 2 FAIL
current context phase                  88/88
current L3 phrase gate                 8/8
current L4 hidden state                4/4
changed-route regression               none observed, PASS_NO_REGRESSION
inherited L3 ranking contract          WATCH, 2 baseline failures
```

The inherited failures are
`repeated_letter_collapse_beats_suffix_expansion_competitors` and
`typed_damage_operator_coherence_beats_morphological_drift`. They were rerun in
separate processes with all usage-memory paths bound to nonexistent files, then
reproduced on the clean baseline worktree. This rules out persisted user state,
test-order contamination, and the L4 V2 changes as their cause. It does not make
the old L3 behavior correct; resolving it remains separate work because changing
live L3 ranking here would invalidate the already built release and require a
new whole-route quality proof.

Differential evidence is retained under:

`/home/e/build/lay-l4-v2-20260815/artifacts/l4-v2-20260815/context-differential/`

The baseline L3 log SHA-256 is
`c684492ba291f831449266eb29963159690a3bc6587c531ce232d3f9f4f9b39e`; the
current L3 log SHA-256 is
`b3e869e5f49233271aa6d8748fcd6ce1914f808a44392b85a8e305b6401898de`.
Runtime authority did not change during this gate.

The managed IBus engine was deliberately not restarted. Its active executable
is the preserved pre-deployment inode while the installed on-disk binary is
`1.0.30`. This avoids risking keyboard loss, but physical IME behavior remains
`WATCH` until a user-visible check or a separately controlled engine restart.
The active `lay-daemon` is `1.0.30`, its `/proc` executable hash matches the
installed release, and its environment selects the organic V2 package.

Rollback:

```text
/home/ubu/.local/lib/lay/rollback/1.0.29-pre-1.0.30-l4v2-shadow-20260815-210832
files       32
bytes       99,087,215
manifest    f70dc96d8fe1f9090c4e931e8e9b6a029f0bc41f9f347531cab7d96e5db8f0e7
```

Exact receipt:

`/home/ubu/projects/lay-l1-exact-peak-search/docs/structural_gates/receipts/L4_MULTILINGUAL_SCENE_FIELD_V2_INCREMENTAL_2026-08-15.json`

Deployment receipt:

`/home/ubu/projects/lay-l1-exact-peak-search/docs/structural_gates/receipts/L4_MULTILINGUAL_SCENE_FIELD_V2_DEPLOYMENT_2026-08-15.json`
