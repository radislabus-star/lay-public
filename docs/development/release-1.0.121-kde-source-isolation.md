# 1.0.121 — KDE source isolation without behavior changes

Preflight recorded 2026-10-31T08:58:56.356798+00:00, before source edits. Baseline a9b9bb9fab96f6b6d4a8fa11d5dceb6ccead065b
(published120); remote KDE120, local GNOME unchanged. Explicitly authorized
field integration source organization, not a correction or native bug fix.

## Dependency and consequence map

```text
lay_daemon.rs ─┬─ kde_focus (same alias) → kde/focus_observer.rs
              │                           └─ focus_observer/foreground.js, tests.rs
              ├─ layout_kde (same alias) → kde/layout.rs
              └─ layout_controller.rs
                   └─ kde_exact (same alias) → kde/layout_handoff.rs
                                              └─ layout_handoff/tests.rs
lay_ibus_engine.rs ─ kde_focus (same alias) → same kde/focus_observer.rs
check-architecture.sh ─ same 180-line ceiling → kde/layout.rs
```

First organizational loss: desktop-specific bodies are outside the KDE directory.
There is no proven behavior change to repair in this release. The owning ADR
contains exact old/new hashes, literal-only seams and all other integration
source hashes. Shared session dbus_connection keeps its existing implementation
and callsites, despite its historical gnome_dbus filename. This move does not
claim complete desktop/client isolation. Correction, ownership, observation
freshness, verifier, executor, timing and output remain unchanged. No GNOME
source or launch/configuration change.

## Proof plan and pitfalls

Verify exact file/body parity and absence of all six old paths locally and
remotely. Reuse canonical dev-check snapshot/owned rsync --checksum --delete;
private overlay transport alone cannot represent deletes. Preserve all Rust
module names/test identities. Small canonical selections must include both
bins' kde_focus tests and daemon kde_exact tests; the existing kde-focus recipe
alone omits them. Then run mandatory full release gates, build all ten release
executables without warnings, install only remotely and verify loaded bytes,
ordinary Shift and browser controls. Source, loaded-runtime and physical-client
evidence remain separate. Historical native matrix is not rescored by controls.

Status: PREFLIGHT ONLY; no source, install or native acceptance yet.

Focused remote proof: 51/51 PASS, same canonical identities and all six old paths absent. Receipt CROOT/release121-isolation-focused-v1/result.json (45.916s); canonical self-test transport PASS14.2s. These are not full/native acceptance.

## First installed control and discriminating baseline plan

Full17 gates PASS/3617Rust/ten binaries0warnings; initial native control
CROOT/release121-isolation-browser-controls-v1/result.json:ChromePASS,FirefoxFAIL
(first after-Space flip remained unchanged). Wire proves one ManualToggleV3
reply0/false. Per-row engine trace0bytes; explicit lay-diagnose report says
UNKNOWN_REQUIRES_CORRELATED_EVENTS. No native-success/publication claim.

Before any code change or identical retry:compare previously accepted exact120
release binaries against exact121, each using unchanged two-browser r19 control
and original cadence/deadlines/config/layout/compat shim. Temporarily install120
from retained verified full-v4 artifacts, then restore121 retained full-v2 bytes.
No source,model,learning cleanup,globalIBus,GNOME or actor change. This tests
whether failure follows binary revision; it cannot prove a timing/state root
from an unpaired pass. Retain the initial FAIL regardless of subsequent result.

Baseline comparison setup-v1 refused before native input: daemon loaded120 but
IBus factory still loaded121 from its existing target/release/lay-ibus-engine
path. Raw failed receipts retained. Setup-v2 synchronizes that exact binary
path atomically with retained120 artifact after verifying current121 preimage;
XML registration,launch path,services and output routes remain unchanged. The
restoration must also restore exact121 factory bytes,then reverify loaded hashes.

## Verified outcome 2026-10-31T09:36:46.512828+00:00

51 focused/3617Rust/34targets/494isolates,430PythonPASS10SKIPof440,17gates,
all10compiler-executable release artifacts0warnings. Source1328SHA556d8b7ccb081cb27bd9591d8b7f518cb5762c3b8e1d0a732db3bd3eedf4cfbb;1332inventory. Exact unchanged946other integration files plus moved-body/seam hashes checked. Source return30files imported with preimage checks.

Initial121native ChromePASS/FirefoxFAIL retained asTD168. Published120 exact
baseline2PASS,restored121exact2PASS (4flips),same frozenr19/shim/config/cadence.
No root-cause or complete native acceptance claim. Loaded hashes/config/mode
and existing factory path verified after native controls;globalIBus preserved.
No localGNOME install/restart. Latencies are unpaired n2/browser;no speedup claim.
Publication pending; see versioned final-verification JSON for every outcome.
