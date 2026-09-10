# First-word observed-suffix source review, 1.0.69

Independent fresh-context review, round 1: **9/10, H0/M0** after the two medium findings were repaired. This accepts the source checkpoint only; release, installation and physical input acceptance remain separate gates.

The reviewed mechanism permits current observed-suffix display and verified explicit append without granting KnownStart, whole-word replacement, Space autocorrection or whole-word learning. Candidate generation, ranking and the edit verifier are unchanged.

Repaired findings: Reset now clears legacy client preedit before local reset, preserving atomic ownership and reset even on signal error; repeated identical surrounding snapshots retain the visible suggestion and Tab availability.

Remaining low finding: unit fixtures do not alone establish native callback scheduling. The frozen seven-case native driver subsequently passes 7/7, including Reset. The duplicate-snapshot wire probe did not prove delivery of a duplicate callback to the engine and is inconclusive; the actual SetSurroundingText/Tab unit test proves that branch. Delayed-worker rejection remains identity/deadline unit evidence. Equal-token relocation can permit a new matching append-only frame; no absolute document-start or all-relocations rejection claim is made.

Focused correctness: 455/455, with three performance tests excluded. No general restoration-quality or performance-bound claim follows.

Reviewed source SHA-256:

| Path | SHA-256 |
| --- | --- |
| `src/bin/lay_ibus_engine/committed_tail.rs` | `8ceae2ed2c8fb1ff3292d9eae3ca33e14244ba66e94c8bacaa2cad8ec8c4d2b3` |
| `src/bin/lay_ibus_engine/composition_commit.rs` | `2dceda04efa40c9b95f3d4ee00a9dea28c5692e7c87901890e4e84a3e1b4dc3b` |
| `src/bin/lay_ibus_engine/context_admission/adapter/tests/residuals.rs` | `5d9497552cf3b2002f65b13faebce95a7c66db058e2c13a256da5dbe69a721f2` |
| `src/bin/lay_ibus_engine/context_runtime.rs` | `e2ed49e909aa17dd0b2daaa2b59d6f181f44cbeda5abcdf53224b018c2aa2c20` |
| `src/bin/lay_ibus_engine/context_runtime/tests.rs` | `8ebaf1602dc7079cb4215479651838406a888a4c3cf304e324474f15035fc876` |
| `src/bin/lay_ibus_engine/engine/types.rs` | `6522a7fef4a4ed336aa2e04f730679c43cbfb20352e3f0e27c83ebe423cc9c3f` |
| `src/bin/lay_ibus_engine/ibus_interface.rs` | `3ac43e41e4f31548ce13529efe7e33da2a72c96a5913416601c33a7f5e85d02d` |
| `src/bin/lay_ibus_engine/preedit.rs` | `abf7294199e8d57def4663a21a7433db8185ae61609da91945fea9188465e59b` |

Private evidence: `/home/ubu/.cache/lay/development/wechat-ime-no-functions-8isocthh/first-word-fix/independent-review-round1.json`, `candidate-build-v3.json`, `focused-tests-v3-SUMMARY.json`; native receipt `/home/e/projects/lay-development-runner/autocorrect-ojoasco5/phrase-first-word-candidate-v2/receipt.json`.

Runtime authority changed by this source-review checkpoint: **false**.
