# KDE Firefox first-word Tab report — 2026-10-29

User confirms second-word Tab in field8, reports no Tab in field4(tel) and first
word field8(plain). Preserve installed daemon95760/4403 and IME4178957/ebd7;
no speculative production changes. Full220126P55F39B, Chrome earlier179P0F41B.

Map: native Tab -> ibus_interface.rs / managed.rs -> composition_commit.rs
accept_completion -> current visible candidate + context ownership proof ->
existing authorized edit -> output -> actual client field.

Diagnostic v1 derives cache actor6857, retains all existing function ASTs,
adds only Tab-only first/second-word probe and dispatch.12 native RFB cases:
text/tel/plain x first p/pri/prov,second pri. Requires fresh visible Lay hint,
then one native Tab, exact published suffix + one Space, caret/focus stability.
Result12PASS/0FAIL/0BLOCKED in19.801s; runtime/config restored, no heldkeys.
Receipt /home/ubu/.cache/lay/development/browser-matrix-repair-20261027/
kde-firefox-first-word-tab-native-v1-verification.json. Separate owned profile;
DOES NOT disprove user's existing-window failure or clear full matrix FAIL.

Read-only original user Firefox988548 confirms KDE/X11:71, GTK/QT ibus,
GNOME_ACCESSIBILITY=1, same compat library loaded, no IBUS_ENABLE_SYNC_MODE.
No obvious launch-environment difference from owned diagnostic. Never modify
user profile/text merely to make the diagnostic equivalent.

Next v2 repeats exact12probe assertions with RU setup once, no layout/engine reset
between cases or field transitions. This diagnoses state retention; not a change
to canonical full actor/cadence/assertions. Pacing work remains mapped, uncoded.

Diagnostic v2 raw11PASS/1FAIL/0BLOCKED. The sole failure is OBSERVER ERROR,
NOT a reproduced Tab refusal: first text input contains correct "привет ";
observer combined completed typed "при" with an in-flight preceding "пр" hint
"оверка" and expected impossible "приоверка ". Revision newer than baseline
alone does not bind a hint to the latest typed prefix. Preserve raw failed receipt;
never transfer this into browser totals or claim product root cause. Tel/plain
all8 cases passed. V3 waits for fixture's known current-prefix suffix (same
existing canonical protocol) before pressing Tab, setup once, all other native
assertions/cadence unchanged. No product code changed.

V3 stateful12/12PASS,19.049s; exact current-prefix hint binding, no per-case
mode/engine reset. All current receipts retain installedbytes/config/keys. User's
reported original-window refusal remains NOT_REPRODUCED, not declared fixed.
No Tab production changes made. Return to measured rapid failures; retain tests
as focused acceptance controls after the daemon-only pacing candidate.

V4 negative source-control diagnostic on restored daemon419352/4403 intentionally
keeps IBusUS/KDERU without any per-row reset. All3 fields text/tel/plain receive
Latin "ghb" instead of intended "при" and publish no visible completion. Raw0P3F
is a precondition diagnostic, not Tab acceptance proof (Tab not pressed). Actual
mode/config/runtime restored. Cannot claim this reproduces user's Russian-text
Tab refusal or changes the full matrix. It confirms tests need coherent modes;
original user Firefox had same launch environment as owned successful controls.

Passive original-window screen confirmed title-case "Приве" in field4 and Latin
"Gh" in field8; no original text changed. Title-case/all-caps first-word native
probe prepared (При,Приве,П,ПРИ in text/tel/plain,12 cases). Same current-prefix
hint binding, one native Tab, exact expected field/Space/caret/focus, no per-case
mode reset. All existing actor functions unchanged except added prefix fixtures.
Will run on exact next daemon candidate after its ordinary22 passes. No assertion
that lowercase proof certifies these observed title-case inputs.
