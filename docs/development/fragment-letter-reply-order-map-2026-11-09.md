# Letter reply order for managed KDE native fields

Baseline: currentW HEAD4fae57d5, source1336 SHA580ea4e28534ead2f2ce29d8f546bb40863ca7307a949f02e36bf005e60f854f, frozen remote letter-fragment-final-candidate-v50-20261109. Product124unpublished; installed/restored123bothhosts. Fullcanonical gatev45PASS incl467Python+12SKIP,3644Rust+11perf,10installables,28ownedXvfb. This does not override current native Chrome3FAIL.

Actual native Chrome120:105PASS/3FAIL/12structuralBLOCKED. Two automatic failures: normaliframe-rich RU→EN secondword and fullscreenrich RU→EN secondword; both emit native_observation_pending twice. Normaltext EN→RU secondword manualreturn did not happen; its first visible RPC in the row trace belongs to prior preparation/engineUS3, not currentengineUS7. Do not misattribute that RPC to failed gesture. Address only first proved automatic ordering mechanism, then diagnose remaining manual separately.

```text
letters → src/bin/lay_ibus_engine/ibus_interface.rs::ProcessKeyEvent
       → WindowInteraction::process_legacy_key (UNCHANGED)
       → fragment_layout.rs::request_fragment_after_letter_release
         → consider_fragment_layout_letter
         → native_document.rs existing fresh read
         → existing word flip / DecisionCore / verifier / output
       → bool key reply currently sent only after all of the above

proposed narrow late-read case:
       → same legacy processing
       → immutable letter-request stamp + existing library reply-dispatch notification
       → SAME bool wire reply
       → SAME existing engine via ObjectServer interface get_mut
       → reject changed owner/focus/epoch/mode/text/admission
       → existing fresh native read and SAME existing Fragment word flip
```

Causal native evidence: remote oracle-and-chrome-v51-20261109/chrome120/case row40. Lastphysicalletter5release2448968.520, snapshot atDOMtime118844.8still `test рудд`(9chars); nativekeyup appears118866.0, inputevent118866.6addsщ. Both reads occurred before the release reply let the client update. Raw `events-normal.json`, row40native trace and wholecase results remain unchanged. No blanket claim about all three failures.

Primary code inspected: pinned Cargo.lock zbus5.16.0; `/home/ubu/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/zbus-5.16.0/src/object_server/dispatch_notifier.rs`. Existing ResponseDispatchNotifier preserves response signature/serialization and signals dispatch. Its explicit caveat: dispatch is not peer reception; fresh actual field proof remains mandatory. Existing precognition_worker.rs already demonstrates reacquiring the same engine through ObjectServer.interface/get_mut. Reuse this ownership pattern, not its display worker queue or model scheduling.

Options: caller fulfillment after existing reply notification9/10(selected); more waiting in still-unreturned key callback3/10(cannot release dependency); extend manual pending state7/10(wider shared-state migration); cached/projectedfield aseditproof1/10(rejected); deliveryrewrite1/10(rejected).

Exact production scope: ibus_interface.rs, fragment_layout.rs, new feature-only fragment_layout/reply.rs; newtests underreply/tests.rs and five existing interface direct-call testfiles may unwrap the bool from its typed reply, preserving all behavioral assertions. Existing39authorizedpaths remain recorded for the carried feature; no other production change is implied. Common reply wrapper has Plain(bool) with exactly old serialization/no extra task/allocation for other paths; deferred variant uses library notifier. Only a positive current letter proposal in managed KDE native-document pending state may defer. No change to Space or other mechanisms, no GNOME deferral.

Pending request stamp is not edit permission: exact engine owner/path, focus serial, local tail epoch+text, composition state/mode generation; current admission must still be live. A next input/Space/focus/ABA change cancels fulfillment. No second detector, new mutation queue, repeating task, timer or retry. Rebuild and verify the current authoritative frame through existing code before the one write. Library notifier is not a client ACK. Keep mutation ordering and indeterminate refusal unchanged.

Verification pending: meaningful controlled reply/field-order regression; exact bool body/signature; stale current word/focus/owner/epoch/mode/Space refuses; OFF/nonletter/GNOME no task; one current actor and existing writer; then same short affected native cases, fullcurrent matrices and release gates. Do not publish on incomplete mandatory verification.

Implementation now prepared, not built/accepted: optional reply continuation is confined to `fragment_layout/reply.rs`; request and observation predicates are reused from the existing fragment caller. The callback keeps the same serialized bool. Twenty input/authority invalidation controls, stale-document refusal, positive-only deferral and private-bus notification/body controls are in `reply/tests.rs`. The five direct-call fixtures extract the bool explicitly without changing their assertions. The existing native-document fixture is reused through a test-only export in its two already-authorized source files; no production native-document operation changes.

Additional graphical verification v52 ran all479Python tests in an owned Xvfb/private bus/private HOME:470PASS/7FAIL/2SKIP. All7 failures are resource-guard fixtures querying the actual user cgroup manager from the deliberately private test bus (`descendant_cleanup`), not GUI or input results. Preserve that failed receipt; rerun only the10 previously skipped GUI tests under the graphical environment and retain the already-passing original-session guard suite. The two real-cgroup/L1.1 opt-in controls remain separate. No installed bytes/config or services changed in this graphical run.

Source verdict: v53c24/24focusedPASS, v53d1179/1179IMEPASS(159.84stestexecution,176.08stotal). v54 built all10fresh compiler-artifact binaries, diagnostic-only. Normaliframe-rich RU→EN second-word selected native scenario in v54b PASS44.08ms, bothmanualreturns andphysicaldecoder checked. RawactorFAIL is exactly incomplete manifest120vsselected1; parentrestorationPASS andoriginal123bytes/config/map restored. Preserve rawfailure; no wholematrix or other field claim. Use unchanged full120actor/controller next rather than modifying its coverage assertion. Final graph regeneration embeds a different proof receipt in runtimebytes; freeze actual post-gate10artifacts before final native acceptance.

Final gatev57 all3649Rust functional/package cases executed without failures and source closure matched before/after, then canonical final ledger validation correctly rejected the stale3700→3705inventory binding. No performance/build/native acceptance follows from this failed whole gate. Fix onlymanifest_sha256 of the unchanged empty ledger, retain original accepted observation hash and bothstrict drift controls. Explicitly documented under the owning ADR; zero semantic waivers. Repeat actual canonical fullgatev57b, retainv57FAIL. Prefix strictlints/479Python evidence is unchanged and separately retained; final graph and actual compiler-artifact binaries must be frozen after metadata completion.

Nativev59Chrome complete107P/1F/12structuralB,original123restored. Residualrow78privateiframe-rich EN-RUsecond:postreplyfreshnative actualcursor=11/localtailchars=11,exact_managed/known/frame/currentbinding/currentdocument true,then native_observation_pending. Therefore whole-prefix readiness veto remains despite completecurrentword; investigate/reuse existing native_managed_readout_is_current onlyforKnownWord readiness, neveraswritepermission. Exactprefixcodepointsnotrecorded,doNOTclaimNBSPasprovenliteral. Existing word/prefix normalization tests and contracts beforecode; no additionalwait/newowner/queue. Fullsourcev57bPASS;B58GNOME35nativePASS. Firefoxv59stillrunning.

Next bounded fix map, before code: letter→existing proposal→feature read-readiness (whole tail exact OR KDE KnownStart+existing native_managed_readout_is_current)→same complete InputFrame→same action/source/current-authority checks→same word flip/DecisionCore/verifier/writer. The existing readout method itself proves only fresh exact-word observation; KnownStart and the full edit pipeline are independent mandatory checks. Initial readiness and bounded fresh-read loop must use one identical predicate. Production change onlyfragment_layout.rs; fixture onlyreply/tests.rs, existing46protectedpaths retained; no shared observation/authority/adapter or GNOME edits. First TDD plainseparator control+olderNBSP/ASCII normalization with exact currentword must alreadyhave currentNativeBinding/completeFrame/executableNativeRange and fresh receipt; oldpendingguardmust fail. Metadata newtest inventoryandemptyledgerbinding must be updated together before final gates.

REDv61 initialfixturelackedactualcapabilities,notcausalproof;reuseadjacentset_client_capabilities(41). REDv61b thenexactlyreproducesoldpendingguard forNBSP→Space afterallcurrentword/frame/range preconditionspass;ASCIIfirstcontrolpasses. GREENv62initialnewnegativeincorrectlytreatedlocalcachedownerNoneascanonicalownerrevocationwhiletokenstillvalid;changedtesttoexistingrevoke_context_word afterotherclonecontrols, preservingtheirindependence. GREENv62b25/25PASSincludingactualrevocation. Productionhelperonlychangesread-readiness and bothconsistentcalleruses, notcommonwriteauthority. Shortdiagnosticactorselectionfooterbugfixedseparatelyonlyexpected_rows precedence fordocumented --only;ASTallinput/setup/observer/target/helperfunctionsbyte-structurallyidentical,r49full120unchanged. Explicitdiagnostic1marks119unselected,supportedfullmatrixnotclaimed.

## Final source and native result — 2026-11-09 21:42 UTC

The historical failures and incomplete probes above are retained as historical evidence. The final complete source gate64 is PASS, including all1180 IME tests, all ten installable release artifacts and actual Rust1.88 all-target checks. Current inventory3706 has an empty failure ledger; no old failure was waived. Exact candidate65 IME/daemon/input bytes equal63.

The unchanged full120 native actor completed on KDE/Chrome and KDE/Firefox: each108PASS,0FAIL,12structuralBLOCKED,0NOT_TESTED. Each selected letter case checks first/second word, both directions, two manual returns and the next physical glyph, across ten fields and three modes. The12 blocks per browser are exactly URL/email second-word ASCII-space sanitization before input. Parent receipt: /home/e/projects/lay-development-runner/letter-fragment-final-native-v65-20261110/RESULT.json. Private mutable learning and source-model hashes were verified; installed123 runtime/config/layout were restored. No permanent installation or publication is established by this result.

Primary GNOME acceptance on the same current runtime bytes separately verifies32/32 physical eight-model transitions plus3/3 Tab/held-Shift controls; independent negative controls20+6 reject malformed evidence. The raw Tab actor legacy producer assertion remains FAIL and is not silently rewritten as PASS. Native standard220-per-browser and ordinary terminal coverage are separate gates; their outcome is still pending.
