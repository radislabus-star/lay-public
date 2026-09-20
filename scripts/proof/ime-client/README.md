# Repository-owned private IME client proof

This is the versioned V2 portable TD-121 actual-client harness. It keeps the five
real `IBus.InputContext` scenario bodies, callback assertions, waits, hard
deadline, post-verdict trace drain and cleanup from V1. V2 corrects the client
contract: it advertises and publishes `SurroundingText`, and applies a fixture
printable press once when Lay returns native-unhandled. This is a successor
contract, not baseline parity with V1.

V1 remains immutable in Git at commit
`708245298a3f553ac3c52243728c02ba6344a140`, path
`scripts/proof/ime-client/driver.py`, blob
`04f7dbac56c92dbe0238c0754bcb6db4e242256c`, actual SHA-256
`9ece223f6689323e5cae3fc5f27cf990d37b86dff5e9f6e0ff3212d4dd488750`,
and historical normalized SHA-256
`669e3ef88cc2639794fde79dcb9132b99ebcaf142cafe39065376f42c4a056dc`.
The maintained V2 driver is pinned directly at SHA-256
`d80447f21db4d689ea49d39742feb36e2202b23979d820380c1fcef916b88c12`;
the canonical runner is pinned at SHA-256
`3d40b60f54b6224edb760203e3b9ea740a60641976edb2665a4ca946cfa33441`;
there is no copied legacy driver or source-rewriting loader.
Publication uses the public `InputContext.needs_surrounding_text()` accessor;
`RequireSurroundingText` is consumed internally by libibus, not exposed as a
public InputContext GObject signal.

Default startup schedule is `immediate` (no lexical wait). The remote worker
also accepts `--startup-schedule post-exact-ready`: once at first context
setup, before any text key, observe this candidate's exact warmup completion
using a private file notification and a2500ms startup deadline. It makes no
warmup call and adds no key delays, retries or Space budget. The five scenario
bodies remain byte-identical; their startup schedule is explicitly different.
Metadata and receipt distinguish the modes. A post-ready PASS cannot replace
or relabel an immediate cold failure. Trace batching adds observation delay,
which is not production readiness latency or evidence of physical input.

The worker also accepts `--scenario-set lifecycle` (default: `restoration`).
This separate three-case lane exercises same-engine client refocus through
IBus's intermediate dummy context, a real change to TERMINAL content type, and
real Reset during refocus. It asserts discarded internal word evidence but
unchanged client-visible text at each transition, a single real boundary,
subsequent admitted printable
input, and a working bridge. There are no new key delays or retries. The five
original restoration scenario bodies and their denominator remain unchanged.
A lifecycle PASS is not restoration quality, physical keyboard acceptance, or
proof of the desktop's cached plain-FocusIn route. Controlled real-zbus tests
separately cover exact-same-context preservation without Disable and held
compatibility Get/reply/marker interleavings. The dummy-context smoke must not
be described as causal RED if the previous candidate passes it too.

`--scenario-set manual-toggle` adds a separate three-case lane: eight terminal
manual conversions per boundary/no-boundary case and a visible preedit after a
same-context handoff. Terminal CommitText bytes go through the existing real
GNU Readline consumer. A private `org.gnome.Shell` fixture forwards one
ActivateLayout request to real IBus; GNOME and the daemon's physical detector
are not exercised. Precognition is enabled in this lane's private config.
The generated metadata binds that config and the copied consumer by SHA-256.

`--scenario-set terminal-delivery` checks shortening, growth and equal-length
correction with both correction and hints enabled before typing. Each word
is delivered through real unhandled IBus callbacks. The client uses
`IBus.unicode_to_keyval` for valid keysyms and supplies the resulting native
glyph to its existing input sink. It observes the existing prefetch publication
for the exact engine path and tail epoch before Space, without injecting a
lease or retrying input. The single replacement goes through GNU Readline;
exact prefix, separator, final text, caret marker and subsequent native input
must match. A fourth case requires a visible native-input hint. This four-case
denominator does not establish immediate-Space latency, the GNOME keyboard
encoder, Kitty or physical input. Trace observation includes flush delay.

Preferred entrypoint from the editing workstation (remote execution and the
existing heavy lease are handled automatically):

```sh
python3 scripts/dev-check.py client --client-config /absolute/remote/config.json
```

The worker API below is valid only inside the existing `lay-resource-guard.sh`
dedicated-20cpu scope. Missing active/lease/profile/jobs/thread markers are
refused before output creation or process launch; an execution-lease label
alone does not authorize a run:

```sh
python3 scripts/proof/ime-client/run.py --remote-worker \
  --config /absolute/path/config.json \
  --output /absolute/path/to/a-new-run-directory
```

Copy `config.example.json` and replace its origin-machine placeholder with the
literal output of `sha256sum /etc/machine-id` from the originating workstation.
The worker must have a different machine-id hash. Both `--remote-worker` and
that inequality are required, so the harness refuses host-local execution.
The output path must be absent and its parent must already exist; an existing
file, directory or symlink is never overwritten.

The public configuration shape remains `lay.ime-client-harness.v1`. Generated
run metadata is `lay.ime-client-harness.run-metadata.v2` and identifies the V2
proof contract, exact active-driver hash and immutable V1 Git provenance.

Before creating the output or launching anything, the runner validates the
candidate hash, the exact dependency-manifest hash, every file size and hash
listed by that manifest, all nine expected dependency roles, the deployed
private IBus loader/daemon and required host tools. Manifest paths are rebased
from its recorded `remote_root` onto the configured dependency root. The
configured embedded receipt mount must equal the manifest's
`receipt_topology.runtime_model_dir`; receipt bytes are mounted read-only and
are never rewritten.

The ninth role is `l2_lexical_phase` at
`l2/l2_lexical_phase_v2.bin`. The runner sets
`LAY_L2_LEXICAL_PHASE_MEMORY` to its read-only private dependency path. This is
a TD-124 harness follow-up for a discovered undeclared dependency, not a third
TD-121 runtime repair.

The configured candidate is mounted at `/tmp/candidate/lay-ibus-engine`, which
is injected identically into the generated component XML and V2 driver.
The run remains inside private bwrap namespaces and a private D-Bus/IBus pair,
with CPU 200%, memory 1536M, swap 0, Tasks 128 and RuntimeMaxSec 90s. It does
not install anything, restart production IBus/daemon processes or change live
input sources. A harness execution verdict remains distinct from product PASS.

Focused static/unit contract command (remote execution owner decides when to
run it):

```sh
python3 -m unittest -v tests.test_ime_client_harness
```
# Test-only terminal frame consumer

`readline_consumer.py` is called by the TD-125 IME binary tests. It takes one
JSON object on stdin (`initial`, `payload`, `marker`), passes the strings as
unchanged bytes to a private Bash/Readline PTY, and returns the actual final
line plus executable/locale/inputrc identities as JSON. Input is never shell
code. DEL interpretation belongs to GNU Readline, not a simulated Python editor.

The binary tests exercise captured production executor output. The opt-in
manual-toggle actual-client lane additionally exercises legacy IBus delivery.
Neither proves Kitty or cold Space correction. The original five-case
`SurroundingText` lane remains separate. All execution stays remote.


`--scenario-set first-word` uses the same terminal consumer and manual RPC
choreography for two separate cases: initial US and initial RU, neither with a
leading or trailing Space, each with eight exact manual conversions. It adds
no startup wait, key delay, retry, deadline or physical detector. The generic
VisibleTail snapshot must stay UnknownStart while the client-visible manual
output succeeds. The existing three manual-toggle cases and five restoration
cases retain their own denominators; this lane does not exercise preedit or
claim a cold first-prediction PASS.

`--scenario-set first-word-us` and `--scenario-set first-word-ru` select one
initial-layout case each, preserving all eight round-trip assertions. Run each
in its own fresh private process to prove two independent cold starts from the
foreign seed. Each receipt has a one-case denominator. Their combined results
do not establish the separate cross-field/profile transition in `first-word`;
retain that matrix's own result, including any setup failure before text input.

The bounded TD-121 startup packet uses `--startup-proof-profile on`, `off`, or
`absent`; the default `legacy` profile preserves every earlier scenario and
command contract. Non-legacy profiles require the unchanged `immediate`
schedule. Profile `on` admits the existing combined `first-word` lane plus the
new `fresh-preedit` and `startup-only` cases. Profile `off` admits only
`fresh-preedit` and `startup-only`: private config keeps precognition enabled
while setting `nanda_autocorrect`, `auto_replace`, `auto_switch_layout`, and
`typing_assist` to false. The earlier grouped run disabled only Nanda while
leaving exact replacement and layout switching enabled; its observed
`DeleteSurroundingText(-3,3)` plus `CommitText("дом ")` therefore proves that
the old fixture did not represent literal-off. It does not prove a runtime
defect. Samples from the corrected off profile form a new measurement group
and must not be aggregated with the three earlier off samples. Profile `absent` admits only
`packages-absent-literal` and `startup-only`; host dependency bytes are still
validated for request provenance, but no dependency directory, model receipt,
package path, or package environment variable enters the sandbox. Empty
L1.1/L2 model roots are explicit and verified by the driver.

The new cases reuse the existing SetGlobalEngine, bridge-owner and
`setup_ready` choreography. They add no wait, retry, readiness poll, RPC, or
trace observation before first input. `fresh-preedit` requires a visible L2
preedit for native ` пров`; `packages-absent-literal` requires exact literal
` ljv ` from a leading boundary with no deletion. The `off` fresh-preedit case
also settles exact literal ` ljv ` after proving its first visible preedit and
records its Space result and zero-delete output. `startup-only` performs no text input. Monotonic
timestamps bracket the existing first Lay SetGlobalEngine through
`setup_ready`; after the behavior verdict and existing trace drain, the receipt
also binds exact/warmup duration and warmup-before-factory event order. A
multi-process packet reports every sample and descriptive min/median/max by
profile. It does not report or imply a production p95.

The non-legacy startup measurement keeps procedure completion separate from
capability. It requires `l2_complete=true`; on/off profiles additionally
require `l2_available=true` and `l2_candidate_ready=true`, while the absent
profile requires both values to be false. The absent literal case uses the
same per-key `deliver_exact_literal` FIFO oracle as the other literal controls.
Here `l2_available` and `l2_candidate_ready` refer only to the lexical
candidate memory used by the IME; they do not certify that every canonical or
productive L2 package is installed, and an empty absent-material readout is not
a quality result.

The post-verdict candidate identity row reads that candidate PID's `/proc/PID/status`
once before cleanup and records `VmRSS` and kernel `VmHWM`. These are descriptive
per-process samples, not a production RSS distribution.
