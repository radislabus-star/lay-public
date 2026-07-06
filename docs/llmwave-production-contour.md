# LLMWave Production Contour

This document is the working route for moving LLMWave from corpus experiments to
safe live use.

## TREE

```text
books/corpus
|
+-- compact wave memory
    |
    +-- dirty eval
        |
        +-- shadow gate
            |
            +-- promotion gate
                |
                +-- live authority
```

Current invariant:

```text
memory present != live authority
```

LLMWave can influence live output only after an explicit promotion gate. A loaded
memory packet is evidence, not permission.

## SCOREBOARD

Use:

```bash
lay-nanda-wave-eval --llmwave-promotion-gate \
  --train-corpus corpus/project_gutenberg_ru.txt \
  --include-dirty-train
```

The promotion-gate command uses a small dirty-log cap by default so it can run
inside the normal development loop. Use `--max-lines N` explicitly for broader
corpus proof runs.

The gate requires:

```text
prediction_points >= 100
records >= 100
vocabulary >= 50
ready >= 90%
top1 >= 50%
top3 >= 85%
```

Verdicts:

```text
PASS-shadow
  Corpus and dirty-log replay are good enough for shadow admission.

WATCH
  The model remains evidence-only. Do not promote to live authority.
```

`PASS-shadow` is deliberately not a live-apply verdict. Live authority still
requires a separate runtime proof because typing output is a destructive action.

## DEBT QUEUE

P0: Runtime proof for live authority

```text
shadow prediction
-> edit plan
-> safety gate
-> output trace
-> no multiword unsafe edit
```

P1: Dirty corpus stability

```text
recent_actions + corrections + phrase_experience
-> filtered train corpus
-> replay report
-> top candidates by source
```

P2: L3/L4 admission

```text
L3 phrase memory = context scorer
L4 goal-state layer = route/context packer
live apply = blocked until runtime proof
```

P3: Tray truth

The tray must show state that maps to real config/runtime fields. A switch that
does not affect a runtime path is debt, not UI.

Current audit:

```text
Input mode
  -> text_backend
  -> nanda_precognition when backend != uinput
  -> applyInputChannel(...)

Typing assist
  -> typing_assist
  -> daemon restart

Auto replace
  -> auto_replace
  -> daemon restart

Auto layout after Space
  -> auto_switch_layout
  -> daemon / IME / output runtime paths

Lay debug log
  -> debug_action_log + nanda_trace + nanda_trace_text
```

`llmwave_shadow` and `llmwave_apply` intentionally stay out of the tray quick
switches for now. They are config/runtime fields, but live authority must pass
the promotion/runtime-proof route first.

P4: Clean candidate arbitration

The final target is one candidate decision center:

```text
L1 form signal
-> L2 word candidates
-> L3 context gate
-> Bayes/usage prior
-> edit-plan safety
-> output backend
```
