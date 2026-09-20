# TD-126 final source acceptance

Date: 2026-09-13. Verdict: **DONE for source-only scope**. Runtime authority and
the installed runtime are unchanged. Installation, activation, real-client
behavior, physical keyboard behavior, CPU/RSS/latency and answer quality are not
accepted by this result.

The implementation review had exactly two passes: **7/10 `REQUEST_CHANGES`**,
then **8/10 `ACCEPT`**, High 0 / Medium 0. One grouped repair preserved the
actual local refusal, pending/cancelled state and partial-effect progress through
the bridge instead of inferring delegation from pre-execution authority. The
causal RED ran 488 cases with the intended one failure; the repaired focused
proof was **506/506 PASS**. There was no third review. The earlier plan review is
separate: 6/10 `REQUEST_CHANGES`, then 8/10 `ACCEPT` after one grouped repair.

The final guarded run used one `dedicated-20cpu` lease on the mini-PC, Cargo
jobs 20, test threads 1 and the guarded 12 GiB target. Its affected denominator
was **2,781/2,781 PASS**: 2,745 correctness and 36 package cases, with zero known
semantic or infrastructure failures. Both lint scopes passed without baseline
expansion; the architecture gate passed all 11 checks and the compiled receipt
matched the accepted source. The authoritative machine-readable record is the
[final source receipt](../../docs/structural_gates/receipts/LAY_TD126_COMMON_WINDOW_2026-09-12/final.json).

## Accepted identity

- Git root at freeze: `83ba0f42db5c87cdc4f17abf42197396566cb05b`.
- Immutable local root:
  `/home/ubu/.cache/lay/development/td126-final4-20260912-LHBMzvAX/`.
- Immutable remote root:
  `/home/e/projects/lay-development-runner/td126-final4-LHBMzvAX/`.
- Source archive SHA-256:
  `d6ab293cae9cab6542448933f8bb86542ba4d3d323a8e1aa724e4c89c6d2c6ba`.
- Accepted 928-path source/test manifest SHA-256:
  `8a9328b744e2845474db010a10138a249cfc1e2902035ee41a79a616360758d5`.
- Source fingerprint:
  `79797de877724b0dfa0df311d06b4ad50d19d064e920d860267f7220c5180546`.
- Graph fingerprint:
  `bcf88bc2c9217b6a5acc392a777f3c3bd6ed28867e214ac777fc32ee853eb5df`;
  binding fingerprint:
  `c72782de41b4941ffa663b94d38530e10713db8898167518aa31e33871e855e2`.
- Final run `RESULT.json` SHA-256:
  `d3e44763d2b2075f7b08ea97e56a4a16806e219bc8c3b7be0edd48cd0f386855`;
  final source receipt SHA-256:
  `af592f56b07fefd77d112d36881c0a9eccad21806698083f1d7eb36b67c3b57f`.

The future Git checkpoint is intentionally broader than a synthetic TD-126-only
patch. The accepted source composition includes the earlier reviewed but
uncommitted IME/context/native prerequisite base and the TD-126 extraction. Of
the 35 tracked code paths changed from the frozen Git root, seven retain the
pre-TD-126 accepted bytes, nineteen contain both prerequisite and TD-126 work,
and nine are TD-126-only. Final4 accepted this composition as one source
identity. The checkpoint does not claim a new quality result or an installation.

## Publication boundary

Source acceptance is complete. Git publication status, exact commit, verified
remote ref and clean post-push worktree belong to the external receipt
`/home/ubu/.cache/lay/development/td126-publication-20260913/publication.json`.
On continuation, inspect that receipt first. If it does not prove the checkpoint,
the authorized publication must target only `origin/codex/cleanup-20260908` and
then update the receipt. No self-referential commit SHA is stored in committed
documentation.

Before that checkpoint, update the canonical graph once for these closure docs,
run the explicit compiled-receipt projection and verify that the intended diff
still matches the accepted source composition. Do not repeat the 2,781 cases or
lint suites unless source identity or a relevant gate input changes. Temporary
candidate activation requires a ready owned test field/capture and
activation preflight. Permanent installation and physical promotion require the
corresponding client and real-behavior checks to pass.
