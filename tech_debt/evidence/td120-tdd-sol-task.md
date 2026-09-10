# Sol/High task: TD-120 failing lifecycle tests only

User authorized continuing diagnosis → TDD → scoped implementation. This first
Sol task is tests only, no production behavior change. Worktree is
`/home/ubu/projects/lay-tech-debt-20260831`, base cc1e2207. Read AGENTS.md and
applicable Graphify instructions. Do not use nanda-structural-gate, local Cargo,
live services/config/binaries, user input devices, or /root files. Do not stage,
commit, push, install, launch agents, or run broad commands.

Read tech_debt/120-scope-autocorrect-suppression-to-word-lifetime.md,
tech_debt/evidence/td120-121-installed-baseline-phase2.md and existing nearby
suppression/composition tests. The ordinary-word design is being narrowed by
Astra in td120-suppression-admission-analysis.md; do not implement a draft.

Prepare a bounded failing-test module named `td120_word_lifecycle_tests` beside
existing IME tests, using existing constructor and output recording facilities.
Only add test source and cfg(test) registration if required. Preserve the dirty
worktree; do not move existing code or fix runtime behavior. Use apply_patch.
No literal word conditions in runtime; test cases use fixtures/data where
repository policy requires it. No new production dependencies or framework.

Required first denominator (give concrete exact test names):

1. Successful ordinary manual edit → fully erase token → new word → next
   suppression check is false. Use actual producer path/typed replacement and
   actual backspace/lifecycle methods, not just copying desired data fields.
2. Same with a left sentence context (token deletion, not entire buffer).
3. Full erase then retype identical surface is a fresh word.
4. Partial erase / continuing the still-open word preserves the guard once;
   second consume false. Include ASCII layout punctuation that is a RU letter.
5. Accepted replacement/undo with trailing boundary does not protect the next
   word. Rejected/duplicate output does not newly arm suppression.
6. Characterize exact replay suppression through temporary empty tail: source
   path/epoch/expiry/revoke contract preserved. This is a control, not V1 proof.

Do not add ignored tests to hide failures. These are intentionally RED until
the next runtime patch, and main will run the exact filter remotely, not commit
a failing implementation. If a scenario cannot be expressed using existing
test facilities without production changes, record it as not yet constructed
and return the feasible subset; never fake the producer/output effect.

Return exact changed files, tests and intended RED/negative controls. Do not
claim compilation or PASS. Parent coordinates source transfer and guarded
remote Cargo on the 20-CPU machine. Limit this task to one test-only patch.
