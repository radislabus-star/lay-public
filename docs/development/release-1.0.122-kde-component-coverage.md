# 1.0.122 — complete the small KDE focus test closure

Preflight 2026-10-31T10:09:03.355188+00:00,baseline 9f0a158bb4c53bef2c61560075a9b62ce0fabdac / published121. Tooling-only;runtime
source,model/config,GNOME/KDE launch and all field/delivery routes are frozen.

## First loss and dependency/file map

```text
scripts/check-lay-component.sh → scripts/check-lay-component.py::RECIPES[kde-focus]
  → select_rows → current canonical manifest (3617identities)
       8guard/backend cases selected
      15daemon kde_focus tests skipped → kde/focus_observer.rs + tests.rs
      15IME    kde_focus tests skipped → same source compiled into engine
      10daemon kde_exact tests skipped → kde/layout_handoff.rs + tests.rs
  → existing cargo_discovery/canonical_drift → existing run_target/isolation
tests/test_component_checks.py → previously checked nonempty selection only
```

Change only the recipe and its regression tests. Add the three implementation
families to existing two controls:48total. Keep every other recipe,execution
contract,deadline,sandbox,manifest identities and runtime body unchanged. The
new ADR freezes all other source files; generated graph receipt and version
metadata remain their existing explicit metadata exceptions. No replacement
runner,client workaround,new owner or broad route change.

## Proof plan

Before recipe edit,add tests requiring actual canonical implementation coverage
and refusal when each implementation family is absent (other8still exist).
Record RED with original recipe;then edit three tuples only and require GREEN
all component tests plus actual canonical48-case remote execution across both
bins. Full mandatory release gates/builds follow; physical behavior evidence
remains separate from source tooling. Initial121native refusal TD168 remains
OPEN and is not fixed by this change. No PASS/FAIL matrix rescore.

RED confirmed:CROOT/release122-kde-component-red-v1/result.json. Original recipe8selected;two regression methods producedfour expected assertion failures(one missing coverage,three absent-family subcases),zero errors. All1323frozen sourcefiles intact;runtime untouched. Recipe changed only after RED.

GREEN:CROOT/release122-kde-component-green-v1/result.json;all5Pythoncomponent contracts PASS and actual48/48canonical cases across daemon/IME PASS,totalworker2.796s. Same1323frozen files intact;only3recipe tuples added. Version metadata now122;full gates next.

## Verified outcome 2026-10-31T10:42:57.679691+00:00

All17release gates PASS,3617Rust/34targets/494isolates (canonicalexit0),
432PythonPASS10SKIPof442,all10release artifacts0warnings. Source1328SHA
742e0997215caef5a73a3113e9e2aa160e970ad529a4fdc5d5e514ef400875a3.
Strict returned-source import36files;newer local handoff preserved only because
remote was unchanged from pin.1323other sourcefiles remain byte-exact.

Remote122installed,all10installed hashes andactualloadeddaemon/IME/factorypath
verified after2/2frozenr19native controls(4flips). Config/layout/globalIBus
preserved,localGNOME untouched. No complete-matrix/speedup claim;TD168OPEN.
Publication pending. Full structured evidence: release-1.0.122-final-verification.json.

Published 2026-10-31T11:11:30Z at https://github.com/radislabus-star/lay-public/releases/tag/v1.0.122. Commit5c6eee98;all4CI/SecurityPASS,identicaltagmirrors/peeledcommit,bothactualinstalled menuupdaters/sources verified. Local119sees122available,KDE122current;no local install/restart.
