# 1.0.118 — retain safe exact ordinary candidates after fragment abstention

Preflight 2026-10-30T23:24:11.887735+00:00. Correction-only,shared semantics for opted-in users on both desktops. Baseline e65b63b0/117;installed KDE117 bytes and config remain protected. Source experiment with two new providers is REJECTED and archived;no experiment installation occurred.

First proven failed transition: typing_rule_graph/rules/fragment.rs::apply_with_mode replaces six existing callbacks. When the optional model returns no replacement,the real ordinary callback is never consulted. CLI117 and actual core RED reproduce four ordinary RU→EN candidates lost with fragmentON. The model/scorer is not the fix owner.

Options after evidence: minimal safe exact fallback 9/10 for this release;independent registry migration 5/10 immediate (A/B/C preservation regressions,requires separate typed identity/context proof,TD166);new kernel/delivery 1/10,rejected. Preserve accepted existing model behaviour/context/required safety and old five tests. Do not claim full provider independence or restore all unknown words.

Map: existing correction_core/candidate_sources.rs -> typing_context policy -> typing_pipeline/candidates.rs -> existing apply_automatic_layout_rule/apply_with_mode -> unchanged layout_fragments analyze -> ordinary callback only if optional proposal is absent -> existing common candidate admission/ranking/verifier/executor. All files in this map except fragment.rs stay byte-identical to117.

Planned production change: after optional NoOpinion only,obtain the real legacy candidate. Require unchanged prefix and a single trailing token;that token must equal the full physical-key projection in the declared direction,so spelling repair/deletion/insertion cannot borrow layout authority. Require the existing is_plain_layout_autocorrect_risky check to permit the exact source/target TOKEN pair;never check an entire phrase and accidentally bypass PlainWord safety. Master/protected-context/short-English guards remain where they are. No source IDs,registry,priority,model,context,keyboard,input or delivery changes.

RED/GREEN: one process-isolated real CorrectionRequest test with original four OFF/ON outcomes and unsafe exact English-source/nonexact repair controls;old five provider tests unchanged. Then SAME fixed20k proof versus BOTH accepted OFF and ON baselines:RUclean>=10000 restored>=693 wrong=0;ENclean>=9987 restored>=2964 wrong=0;no lost correct ordinary targets and exact OFF parity. Reject any regression. Known pre-existing independent coverage/punctuation issues remain TD166,not falsely closed here.

After quality:full canonical source,all10release binaries,CI/Security,exact installed KDE bytes and first/second-word Space controls;publish only after gates and verify actual installed updater entrypoints on both hosts. No local GNOME installation/restart. Full native matrix remains historical until a full matrix is run.

## Source quality accepted — 2026-10-30T23:45:18.704331+00:00

Actual RED safe-red-v1:new core101/old5PASS;GREEN safe-green-v1:samecorePASS8.991s,old5PASS0.065s. Strict source-return imported48verifiedfiles;restored other4runtimefiles exact117. Production change is onefile,existingfiveguards untouched.

Fixed20000 safe-quality-v1 PASS256.628s,40000trainerparity/model687981bytes. OFFexact:RU10000clean/390restored/97wrong;EN9979clean/2964restored/105wrong. ON:RU10000clean/693restored/0wrong/9307abstain;EN9987clean/2965restored/0wrong/7035abstain. AcceptedONwasRU693/0andEN2961/0;four additional ENrestorations,no lostcorrectordinarytargets,no class regression. Existing13cleanENerrors remain,not fixed here.

Receipt /home/e/projects/lay-development-runner/kde-install-control-20261020/release118-fragment-safe-quality-v1/result.json;project summary evidence/release-1.0.118-fragment-quality.json. Source1327 SHA21c4d6d7ce35c0fb20bcd3cc4a95bc5057296323b6dd872dd913fe3e79dccab5. Full source/all10release/CI/native pending;no118install yet. Raw model median3.468us/p958.369us,cold378us;this is model timing,not keyboard-to-field latency. Cargo117→118 metadata next;helper/model code stays exactly the quality-tested version.

## Full gate infrastructure stop and cache cleanup

full-v1 stopped at test-manifest discovery BEFORE functional tests:Cargo target13GiB exceeded canonical12GiB budget (discovery clean environment uses the default,despite outer16GiB). Keep this infrastructure FAIL receipt;do not classify as source PASS. No limit/allowlist/test was weakened. Guarded cleanup removed4050679296bytes of obsolete debug workspace test executables,keeping two newest per target and all active files. Active IME is target/release/lay-ibus-engine,PID2298008/SHA5f728b...,explicitly preserved. release,rlib/rmeta,models,retained evidence untouched. Cleanup receipt /home/e/projects/lay-development-runner/kde-install-control-20261020/release118-obsolete-test-cache-cleanup-v1/result.json. Retry same source in full-v2;no functional tests fromv1 need repeating.

## Full source and installed bounded proof — 2026-10-31T00:45:34.400532+00:00

Full-v2:all3616actual tests/34targets/492process-isolated PASS;canonical wrapper initiallyexit1 ONLY due old manifest binding,zeroledger rebound after matching every actual PASS. CI must validate the canonical wrapper before publication. Lints/Python439with10skips/public/installer/Node/syntax/audit50/MSRV1.88bothfeatures pass. All10real compiler-artifact release binaries retained,version118,zero warnings independently inspected;1331-file exact source+release-metadata inventory saved. Source1327 SHA58c50f93e3a5daffcac535d23779f1a7e206021b7e865c29fe9b7bee4421ea5e.

KDE118 installed and loaded through existing coordinator;daemon3119526/3e6cda78...,IME3119609/7d0d33e1...,tray3119657;globalIBus3168326 and exactconfig5d02... preserved. No localGNOME install/restart.

Initial six-case native proof5PASS/1FAIL:Chrome roundtrip60.619/65.235ms,second-word Space102.119ms;Chrome first-word retained source (FAIL). Firefox roundtrip160.003/164.166ms,first/second Space18.675/18.275ms. The failed first case had an empty trace suffix despite actor complete=true;no causal claim is justified. Subsequent diagnostic OFF/ON with temporary debug each2PASS;rawconfig restored. One default-config confirmation after restoration2PASS:Chrome roundtrip50.665/37.949ms,first Space49.817ms. This does not erase initial failure,prove cold reliability or establish a new6/6suite. It is recorded as OPEN TD167;no route changes were made to force a pass.

Project consolidated proof evidence/release-1.0.118-verification.json. All timings are a few native observations,not a p95 benchmark or speedup proof. Full440-case native table stays historical344/15/81. Release scope is only safe exact ordinary-candidate retention;not full provider independence or universal first-word reliability. Public CI/tag/release/updater checks pending.
