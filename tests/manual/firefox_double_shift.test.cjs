"use strict";
const test = require("node:test");
const assert = require("node:assert/strict");
const { createRun, FIELD_IDS } = require("./firefox_double_shift.js");
const shot = (text, extra = {}) => ({ fieldId: "text", text, trusted: true, focused: true, composing: false, caret: text.length, ...extra });
function key(run, code, type, text, extra = {}) {
    run.key({ fieldId: "text", code, type, trusted: true, repeat: false, ...extra }, shot(text));
}
function pair(run, text, code = "ShiftLeft") {
    for (let i = 0; i < 2; i++) { key(run, code, "keydown", text); key(run, code, "keyup", text); }
}
function roundTrip(scenario, extra = {}) {
    const run = createRun("text", scenario), suffix = scenario === "space" ? " " : "", ru = "привет" + suffix, en = "ghbdtn" + suffix;
    run.observe(shot(ru, extra)); pair(run, en);
    key(run, "KeyZ", "keydown", en); run.observe(shot(en + "z", extra));
    key(run, "Backspace", "keydown", en + "z"); run.observe(shot(en, extra));
    pair(run, ru, "ShiftRight");
    key(run, "KeyZ", "keydown", ru); run.observe(shot(ru + "я", extra));
    key(run, "Backspace", "keydown", ru + "я"); run.observe(shot(ru, extra));
    return run;
}
test("active and closed word each require seven visible milestones", () => {
    for (const scenario of ["active", "space"]) {
        const run = roundTrip(scenario);
        assert.equal(run.state.status, "PASS_DOM");
        assert.deepEqual(run.state.milestones.map(x => x.stage), ["type_ru", "toggle_en", "decoder_en", "erase_en", "toggle_ru", "decoder_ru", "erase_ru"]);
        assert.equal(run.state.milestones[1].shiftTaps, 2); assert.equal(run.state.milestones[4].shiftTaps, 2);
    }
});
test("unchanged final word cannot hide missing intermediate conversion", () => {
    const run = createRun("text"); run.observe(shot("привет")); pair(run, "привет");
    assert.equal(run.state.step, 1); assert.equal(run.state.status, "RUNNING");
});
test("untrusted input and untrusted Shift cannot establish a milestone", () => {
    const run = createRun("text"); run.observe(shot("привет", { trusted: false })); assert.equal(run.state.step, 0);
    run.observe(shot("привет"));
    for (let i = 0; i < 2; i++) { key(run, "ShiftLeft", "keydown", "ghbdtn", { trusted: false }); key(run, "ShiftLeft", "keyup", "ghbdtn", { trusted: false }); }
    run.observe(shot("ghbdtn")); assert.equal(run.state.step, 1);
});
test("foreign-field callbacks cannot alter or finish the current field", () => {
    const run = createRun("text"); run.observe(shot("привет", { fieldId: "search" })); assert.equal(run.state.step, 0);
    run.observe(shot("привет"));
    run.observe(shot("ghbdtn", { fieldId: "search", focused: false })); assert.equal(run.state.status, "RUNNING");
});
test("focus loss blocks even a matching surface", () => {
    const run = createRun("text"); run.observe(shot("привет", { focused: false }));
    assert.equal(run.state.status, "BLOCKED"); assert.equal(run.state.reason, "FOCUS_CHANGED");
});
test("trusted Escape reports failure in every focused field, including iframes", () => {
    for (const fieldId of FIELD_IDS) {
        const run = createRun(fieldId);
        const event = { fieldId, code: "Escape", type: "keydown", trusted: true, repeat: false };
        const snapshot = shot("", { fieldId });
        run.key({ ...event, trusted: false }, snapshot);
        assert.equal(run.state.status, "RUNNING");
        run.key(event, snapshot);
        assert.equal(run.state.status, "FAIL_REPORTED");
        assert.equal(run.state.reason, "USER_REPORTED_FAILURE");
        assert.equal(run.state.observations.at(-1).code, "Escape");
    }
    const unfocused = createRun("text");
    unfocused.key({ fieldId: "text", code: "Escape", type: "keydown", trusted: true }, shot("", { focused: false }));
    assert.equal(unfocused.state.status, "BLOCKED");
    assert.equal(unfocused.state.reason, "FOCUS_CHANGED");
    const unfinished = createRun("text");
    unfinished.observe(shot("привет")); pair(unfinished, "ghbdtn");
    key(unfinished, "KeyZ", "keydown", "ghbdtn"); unfinished.observe(shot("ghbdtnz"));
    key(unfinished, "Backspace", "keydown", "ghbdtnz"); unfinished.observe(shot("ghbdtn"));
    pair(unfinished, "привет"); key(unfinished, "KeyZ", "keydown", "привет"); unfinished.observe(shot("приветя"));
    key(unfinished, "Backspace", "keydown", "приветя");
    assert.equal(unfinished.state.step, 6);
    key(unfinished, "Escape", "keydown", "привет");
    assert.equal(unfinished.state.status, "FAIL_REPORTED");
    assert.equal(unfinished.state.step, 6);
    assert.equal(unfinished.state.milestones.length, 6);
    assert.equal(unfinished.state.observations.at(-1).text, "привет");
});
test("held Shift autorepeat is one tap", () => {
    const run = createRun("text"); run.observe(shot("привет")); key(run, "ShiftLeft", "keydown", "ghbdtn");
    for (let i = 0; i < 5; i++) key(run, "ShiftLeft", "keydown", "ghbdtn", { repeat: true });
    key(run, "ShiftLeft", "keyup", "ghbdtn"); assert.equal(run.state.step, 1);
});
test("second Shift release must arrive before accepting the conversion", () => {
    const run = createRun("text"); run.observe(shot("привет"));
    key(run, "ShiftLeft", "keydown", "ghbdtn"); key(run, "ShiftLeft", "keyup", "ghbdtn"); key(run, "ShiftLeft", "keydown", "ghbdtn");
    run.observe(shot("ghbdtn")); assert.equal(run.state.step, 1);
    key(run, "ShiftLeft", "keyup", "ghbdtn"); assert.equal(run.state.step, 2);
});
test("mixed Shift sides are recorded as unsupported stimulus", () => {
    const run = createRun("text"); run.observe(shot("привет")); key(run, "ShiftLeft", "keydown", "ghbdtn"); key(run, "ShiftLeft", "keyup", "ghbdtn"); key(run, "ShiftRight", "keydown", "ghbdtn");
    assert.equal(run.state.status, "BLOCKED"); assert.equal(run.state.reason, "MIXED_SHIFT_SIDES");
});
test("wrong next-letter decoder is a visible failure", () => {
    const run = createRun("text"); run.observe(shot("привет")); pair(run, "ghbdtn");
    key(run, "KeyZ", "keydown", "ghbdtn"); run.observe(shot("ghbdtnя"));
    assert.equal(run.state.status, "FAIL_VISIBLE"); assert.equal(run.state.reason, "NEXT_LETTER_WRONG_LAYOUT");
});
test("an expected decoder surface without the physical KeyZ observation is insufficient", () => {
    const run = createRun("text"); run.observe(shot("привет")); pair(run, "ghbdtn"); run.observe(shot("ghbdtnz"));
    assert.equal(run.state.step, 2);
});
test("returning to the prior text without observed deletion does not complete erase", () => {
    const run = createRun("text"); run.observe(shot("привет")); pair(run, "ghbdtn"); key(run, "KeyZ", "keydown", "ghbdtn"); run.observe(shot("ghbdtnz")); run.observe(shot("ghbdtn"));
    assert.equal(run.state.step, 3); run.observe(shot("ghbdtn", { inputType: "deleteContentBackward" })); assert.equal(run.state.step, 4);
});
test("paste is blocked and missing observations never become PASS", () => {
    const run = createRun("text"); run.observe(shot("привет", { inputType: "insertFromPaste" }));
    assert.equal(run.state.status, "BLOCKED"); assert.equal(run.state.reason, "PASTE_OR_DROP");
    const other = createRun("text"); for (let i = 0; i < 129; i++) other.observe(shot("п"));
    assert.equal(other.state.status, "BLOCKED"); assert.equal(other.state.reason, "OBSERVER_LIMIT");
});
test("unknown caret and active composition remain explicit separate observations", () => {
    const run = roundTrip("active", { caret: null, caretObserved: false, composing: true });
    assert.equal(run.state.status, "PASS_DOM");
    assert.equal(run.state.milestones[0].caret, null); assert.equal(run.state.milestones[0].composing, true);
});
test("fixture declares ten distinct fields and refuses unknown cases", () => {
    assert.equal(FIELD_IDS.length, 10); assert.equal(new Set(FIELD_IDS).size, 10);
    assert.throws(() => createRun("password")); assert.throws(() => createRun("text", "other"));
});
