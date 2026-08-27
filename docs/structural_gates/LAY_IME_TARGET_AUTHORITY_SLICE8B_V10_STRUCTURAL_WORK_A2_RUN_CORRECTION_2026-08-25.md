# Slice 8B V10 Structural Work A2 Run Correction

Date: 2026-08-25

## Decision

A2 build passed and produced the exact sealed ELF, but generic tree sealing
changed its mode from `0555` to `0444`. Direct `execve` would fail. The run
marker remains available and the structural test has not executed.

A read-only admission probe invoked the ELF through the system loader with the
Rust harness argument `--list`. This loaded the harness only. It had no A2
asset environment, created no subject result, did not select the ignored exact
test and consumed no run marker. The invocation is recorded rather than hidden.

## Corrected Run Route

The exact sealed ELF bytes are executed through:

```text
/usr/lib/x86_64-linux-gnu/ld-linux-x86-64.so.2
SHA-256 8d06f393f4a93bcf9b81145a259524d66a95522a646bf8d7e05b6ffdf2e63dcc
```

followed by the unchanged ELF path and exact test arguments. No ELF copy,
`chmod`, rebuild, loader substitution or fallback is allowed. The controller
must verify loader and ELF identities before consuming `run.available`.

Because the sealed build contains the pre-correction controller, one corrected
controller copy may be transferred to a temporary path and used only as the
run owner. Its SHA-256 is recorded in run provenance. It may change only:

```text
direct ELF command -> pinned loader + same ELF command
sealed controller invocation -> temporary corrected controller invocation
A2 run-correction contract/preflight pins
```

All structural counters, subject source, executable bytes, schedule, assets,
parity checks and claim boundaries remain unchanged. No build right is added.

Successful terminal verdict remains
`STRUCTURAL_WORK_OBSERVED_NO_PROMOTION`.
