# Fragment word-mode delegation integration — 2026-11-09

Current failure and rejected alternative are in the owning letter-only document. This is a narrow mode integration, not a new text delivery route. User broad authorization covers the exact source list in the ADR and active-field scope.

```
letter-only detector / fragment_layout.rs
  -> existing DecisionCore -> verifier -> existing writer (unchanged)
  -> existing complete_fragment_word_mode (stable engine)

manual return of that GNOME word (open or already followed by exact separators)
  window_interaction/authority.rs: existing admitted manual request / exact handoff
  -> manual_toggle.rs: typed DelegateExactImeTailWordMode (new origin status5)
  -> manual_trigger_runtime/ime.rs: same ReplayExactImeTail, origin metadata only
  -> layout_controller/ime_bridge.rs: same captured exact lease and plan
  -> exact_ime_tail_replay.rs: same input isolation, authorization and writer
       layout-only marked step:
       layout_controller/fragment_word_mode.rs
       -> bridge.rs: CompleteFragmentWordModeV1
       -> window_interaction/fragment_word_mode.rs
       -> fragment_layout.rs: validate old handoff and reuse mode operation
  -> existing validate_after_controlled_layout_handoff
  -> existing suppression, delete, insert and actual result observation
```

No source switch occurs during the automatic open-word conversion. Genuine owned preedit retains its existing separate operation. A mode origin has no text authority: every old lease, focused-window, isolated-input and output check still runs. The mode descriptor holds only the existing bounded handoff identity for a single replay, never a mirrored word owner or a new queue. Rollback is before destructive output only and refuses a changed context. Normal Space, ordinary manual toggles, KDE layout paths and all model/ranking code remain on their existing branches. New protocol metadata is only emitted after the existing manual request is admitted and delegates the exact tail.

Closed-word control v38: `ghbdtn` converted on letters to `привет`, ordinary Space produced `привет `, but manual return failed before output because the selected Lay source still differs from the decoder. The manual-origin packet must preserve the exact final token plus its existing closing separators. The lexical boundary comes from existing `last_tail_token_text`; a literal suffix comparison rejects missing, extra or changed separators. This extends no edit authority and adds no Space callback. Evidence: `/home/ubu/.cache/lay/development/letter-fragment-request-20261102/gnome-closed-one-v38-20261109/RECEIPT.json` and restoration receipt. Two focused tests are added before the fix; RED/GREEN and native closed-word proof pending.
