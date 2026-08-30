# TD-104 Integration-Test Boundary Correction V4

The V3 implementation correctly removed the L3 feedback-overlay compiler from
the default library graph. A clean default `cargo check --all-targets` then
showed one remaining ownership mismatch:

```text
test:l3_context_feedback_overlay_contract
error[E0425]: compile_l3_context_feedback_overlay_memory is unavailable
```

Integration tests consume the library as a normal dependency, so the library
does not receive `cfg(test)` for that target. The contract itself exclusively
exercises the cold feedback-overlay compiler and therefore belongs to the same
explicit research route as its producer.

Effective correction:

```text
tests/l3_context_feedback_overlay_contract.rs
    #![cfg(feature = "research-tools")]
```

With `research-tools`, the test remains present under the same target and test
identity. Without the feature, the empty integration-test crate compiles and no
cold compiler API is pulled into the default library. A `--keep-going` default
all-target scan found no other integration-test ownership mismatch.

The failed V3 candidate logs remain on the mini-PC at
`/home/e/td104-candidate-v3/all-targets-1.{stdout,stderr,time}`. They are not a
runtime or semantic failure and were not included in the accepted measurement
series. The accepted V4 series was restarted from fresh target directories
after this source correction.
