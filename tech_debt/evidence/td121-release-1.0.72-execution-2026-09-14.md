# TD-121 / 1.0.72 final execution receipt

Status: **INSTALLED_AND_ACTIVATED_PUBLICATION_READY**.
This is the current execution record. Earlier "Current" or "Latest" sections
in the [owning architecture evidence](td121-r5-final-native-analysis-2026-09-14.md)
and TD-121 history are preserved checkpoints of their named candidates.

The accepted R12 repair retains observed exact-replay history through delayed
and interleaved Firefox resets, retires completed and expired fence scopes,
preserves literal trailing Space, and uses the verified synchronous Firefox
launcher with the reset-notification compatibility library. Source consequence
analysis and candidate failures remain in the owning evidence.

Transaction directory: `/home/ubu/.cache/lay/development/release-1.0.72-td121-r12b-20260914`.
Final release result: `raw/RESULT.json`, SHA-256 `1e505e20ff9379cc59b48d7d8c3d5c64fdfd51e71134dad400021d58c5c402f2`.
Source archive SHA-256: `1d47883e939b2a90af7c1a751c8602b029a154395c6af7f89ce42833b9c1346b`.
Source request SHA-256: `db092475a9803d236766467af60959ec37df04b6929656f4eec21dcaaf77d9ce`.

## Measured acceptance

- Both canonical `check-lay-changed.sh` and `check-lay-full.sh` passed;
  each ran all 2862 correctness/package tests with zero semantic or
  infrastructure failures: 2826 correctness and 36 package tests.
- The discovered manifest contains 2888 tests. Its 15 ignored tests and
  11 optional performance tests are outside the 2862 executed denominator.
  No heldout quality, general Wave-quality, CPU/RSS or performance gain is claimed.
- All four mandatory isolated client profiles passed and cleaned up their
  private processes. Full source/dependency and final artifact audits passed.
- All four original Firefox native scenarios passed on the final release bytes.
  Each captured the first visible target and exact returned source after exactly
  two complete DoubleShift gestures. Trailing spaces are part of the surface.
- Final focused development evidence remains 563/563 IME tests with two
  independent source-review passes. This is a separate denominator.

| Native case | First target | Exact returned source | Toggles |
|---|---|---|---:|
| `ghbdtn_extra_lshift_enter` | `"привет"` | `"ghbdtn"` | 2 |
| `ime_file_ghjdthrf_shift_twice_enter` | `"file проверка"` | `"file ghjdthrf"` | 2 |
| `td121_completion_two_toggles` | `"ghjdthrf "` | `"проверка "` | 2 |
| `td121_space_two_toggles` | `"текст "` | `"ntrcn "` | 2 |

Native proof: `final-native/two-toggle-visible-proof.json`, SHA-256
`9cee85efda148e75cdc30cbd043eaa5aabbb39d44df4c4e5f69445a9aab8ba42`. Its bound raw result is
`final-native/native-control.json`, SHA-256
`9cc30c81e765c16a5f62772f826e7b48989ee07d4dca88e3e8e539e8d5ec9ad3`.
The fixed native harness restored desktop state before installation.

## Installed and active

All ten installed release artifacts match the final research-tools build.
Installation reused these accepted bytes without rebuilding. Installation receipt:
`installation-1.0.72.json`, SHA-256 `f10c87b9c025fcd69eaff78660851f0901faa6bcee6cbf2e957a1c1e020fb02b`.
Installation reports active authority changed because the repaired binaries are
now loaded; the proof harness itself had no lasting runtime authority.

| Loaded release binary | SHA-256 |
|---|---|
| lay-daemon | `de592803db1f1ba4f13fa3de9c404c1e0bfba4a8eaa67643609a95aadef9eeac` |
| lay-ibus-engine | `01738a551745fe125b6668e0cb7650cdf928e3609047f43cc25e81fd5bf210bc` |

The global IBus process, selected input source, config, immutable model payloads
and input journals were preserved. Only owned Lay producers were restarted.
Rollback artifacts are at `/home/ubu/.local/state/lay/release-backups/1.0.72-v5625qik`.

Firefox was closed through its normal Quit command and restarted through the installed
launcher. The existing profile restored 12 of
12 tabs. The new Firefox process has
`GTK_IM_MODULE=ibus`, `IBUS_ENABLE_SYNC_MODE=1`, and the exact accepted compatibility
library mapped by path, device and inode. The Russian UI and all desktop locale
settings were verified after relaunching the exact existing profile. The one
temporary blank activation window was removed. Existing user.js network/HTTPS
preferences and the profile-selector/quit-confirmation choices were preserved.
No session content or credentials were
extracted. Activation receipt: `firefox-activation.json`, SHA-256
`fdaf34573f3214391bf20b346ff330de0d9ec2e046d58d107b7f25e72df14dfc`.

## Publication and proof boundary

The user explicitly authorized autonomous checking, installation and push while
away. A human hardware-keyboard observation remains **NOT_TESTED**.
The original `/home/ubu/projects/lay` checkout was outside the edit scope.
Publication targets `origin/codex/cleanup-20260908`; exact commit and verified
remote ref are recorded separately in `/home/ubu/.cache/lay/development/release-1.0.72-td121-r12b-20260914/publication.json`.

This execution ledger is excluded from the graph's compiled source binding.
All implementation and design inputs were frozen before the final gates.
After writing this ledger, the required guarded remote architecture refresh must
prove the embedded receipt bytes unchanged before publication; its result is
`post-install-graph2/RESULT.json`. Publication records the ledger's explicit
post-build hash delta, rather than claiming that execution facts were compiled.

## Preserved failed final attempt and metadata correction

The first R12 final attempt at
`/home/ubu/.cache/lay/development/release-1.0.72-td121-r12-20260914/raw/RESULT.json`
remains **FAILED**: all 2862 correctness/package tests passed in both routes,
then the full lint contract rejected moved positions of existing warnings.
The guarded canonical baseline writer regenerated the two inventories; exact
logical comparison with only byte offsets removed proved all 534 default and
358 research warning entries unchanged. No new warning exceptions were admitted
and no runtime source changed for the retry. Correction receipt:
`/home/ubu/.cache/lay/development/td121-r12-lint-baseline-20260914/offset-only-proof.json`.
The R12b attempt reran the actual canonical gates and supplies final acceptance.


The first post-install refresh is preserved as **FAIL** at
`post-install-graph/RESULT.json`; no output from it was imported. Its only
manifest delta was the generated `src/generated/architecture_graph_receipt.json`
itself (mtime and AST hash), which changed the binding hash through the existing
self-reference. The successful-release generator had indexed its input receipt
before writing the accepted output receipt.

The second refresh reproduces that exact generator input from the immutable
final `source.tar`: only the generated receipt is seeded with its original
bytes/mode/mtime in remote scratch space. All other accepted source inputs,
including the current execution-only ledger, remain fixed. The canonical update
and checks still run. Acceptance requires the regenerated receipt to equal the
compiled release bytes and verifies every other source/output hash and mode.
No runtime source, test assertion, graph checker or installed binary is changed.
The seed hash and original archive/request provenance are recorded in
`post-install-graph2/request.json` and its result.
