# 120 release gate — isolate the prepared-lease modifier fixture

Preflight 2026-10-31T06:49:24.795590+00:00. No production or test-body edit.
Baseline119/b06a294b; candidate120 runtime leaf remains its diagnostic flag fix.

Full-v1 executed34targets/3617identities with1FAIL: the original modifier wire
order test committed ghbdtn rather than привет. Remaining3616 reported PASS.
The whole IME test source is frozen against119 (SHA 5a3c34cbc727a10cc260a281600604e0dfa6e65c32e1e96a797d61aeb13cec1c). Source/code hash, actual flags, fixture assertions and expected text must not be changed to obtain PASS.

Control-v1 with default Cargo features passed but is NON_EQUIVALENT; preserve
it as such. Corrected control-v2 used canonical research-tools/all-targets
build and unchanged original test in a fresh canonical hermetic process:
PASS0.445s, compiler reused existing artifacts0.502s. Same failed source closure,
exact live119 processes/config unchanged. Receipts CROOT/release120-refusal-full-v1
and CROOT/release120-legacy-lease-isolation-control-v2/result.json.

The test installs a prepared correction lease, preserves it across modifier
and key releases, and asserts later Space consumes it. The actual fixture
helper uses the process-global worker registry/latest-request generation and
current model-material generation; other bulk cases can leave background
work or mutate this material. Shared-process dependence is demonstrated by
the differing bulk/fresh observations, but the exact interfering writer or
scheduler event is NOT independently identified. Do not claim its runtime root
cause or that isolation fixes any user's keyboard behavior.

Chosen9/10: register this prepared-lease fixture with the existing canonical
process-isolation mechanism, as already done for global runtime-flag fixtures.
This supplies a private process/namespace/config and preserves every original
assertion, event order, fixture preparation, expected text and deadline.
4/10: add reset/cancellation around the test; risks hiding the property and
introduces another lifecycle owner.1/10: increase timeout, bypass lease checks,
change expected text or retry until green; rejected.

This is test execution metadata only (`scripts/test_lanes/discovery.py`).
No src file edit, no new runner, retry, model input or field route. Keep309
frozen source hashes including the entire failing test file. No package,
learning, ranking, authority, concurrency or runtime latency effect; extra
private-process initialization cost belongs to test execution. Rollback is one
isolation registry entry. Re-run complete gates, retain full-v1 failure and
require canonical final0 after the new test manifest/observations are bound.
