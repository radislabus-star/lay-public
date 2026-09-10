# TD-120 O6 full-frame proof plan

Status: `REGISTERED`, test-only source written, not compiled or run.
Runtime authority: unchanged by this proof patch.

## Exact test manifest

- `td120_o6_a_former_word_guard_is_absent_before_new_frame_all_safety_profiles`
- `td120_o6_b_exact_space_frame_matrix_all_safety_profiles`

The first test reconstructs the former-word episode independently for
`strict`, `normal`, and `experimental`: successful committed-tail manual edit,
admitted RU-to-US engine handoff, native committed-tail Backspaces through the
complete edited token, then managed printable key events for `gjxbnfq`. It
consumes the semantic suppression decision only in this dedicated fixture and
requires it to be false.

The second test reconstructs a fresh episode and does not inspect or consume
the suppression decision before Space. For each safety profile it proves three
separate cases:

1. Former-word case: the same successful manual edit and full token deletion,
   followed by a complete current `gjxbnfq` frame and Space.
2. Clean control: the same complete frame and Space with no prior manual edit.
3. Protected control: an actual `почитай -> gjxbnfq` manual edit whose
   ordinary current-word guard remains live when Space is pressed. Because a
   path handoff does not transfer `last_input_at`, this control appends and
   natively erases one character through real key routes before frame capture;
   the surface returns to `gjxbnfq` while the same word and guard stay open.

The former-word and clean cases require exactly one current-token
DeleteSurroundingText of 7 scalars, exactly one CommitText of `почитай `, and
the final mirrored surface `left почитай `. The protected control requires no
delete, one plain-space CommitText, and `left gjxbnfq `. These assertions keep
the left neighbor unchanged and make output count/surface part of the proof.

## Route and prerequisites

This is an **exact-layout Space route** proof, not a full Nanda-worker winner
proof. Each case captures the production `InputFrameIdentity` after the last
printable event and requires its observed token, context prefix, committed
tail, active US layout, exact-authority snapshot, lexical coordinates, config
identity, and live engine identity to match. It then installs the production
test helper's deterministic exact lease for that exact frame and invokes the
normal `process_pressed_key(KEY_SPACE)` path. Space still owns suppression
consumption, lease lookup, decision authorization/verifier result, physical
edit construction, and final tail mutation; no fabricated apply decision or
certificate is introduced in this module.

Required metadata/runtime setup:

- built-in exact-layout authority warm-up must succeed;
- the factory engine profiles must resolve as RU for the source and US-QWERTY
  for the target;
- the captured token must be a live printable frame (`active_composition=true`),
  which closed exact-layout certification requires;
- the admitted existing path handoff must complete within its bounded lease;
- managed IME CommitText and SurroundingText delete capability are enabled;
- `auto_replace=true`, `auto_switch_layout=true`, and the named
  `correction_safety` profile are bound into the captured frame;
- candidate material generation must remain unchanged between exact-lease
  installation and Space.

No environment variable, live IBus process, installed binary, network service,
or user input device is required. Parent added cfg(test) registration in
`lay_ibus_engine.rs` after the open-token/UI-effect fixture correction.

## Deliberate non-claims

- No compile, execution, RED, PASS, changed-gate, release, or live-runtime
  result is claimed here.
- This does not prove the asynchronous full Nanda lease wins, its rank quality,
  per-error-class heldout percentages, latency/RSS/package budgets, or
  correction quality outside the closed exact-layout `gjxbnfq -> почитай`
  contour.
- It does not replace O1-O5, V1, E1, A1-A3, C1, TD-121 context identity, or
  TD-122 legacy transport proofs.
- The guard-absence assertion alone is not counted as conversion proof; the
  independent Space output/surface assertions are the conversion denominator.
