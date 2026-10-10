"use strict";
const test = require("node:test");
const assert = require("node:assert/strict");
const { createRun, FIELD_IDS, summarizeTimings } = require("./firefox_double_shift.js");
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
    assert.throws(() => createRun("text", "active", "other"));
});

function timedKey(run, code, type, text, at) {
    run.key({ fieldId: "text", code, type, trusted: true, repeat: false }, shot(text, { at }));
}
function timedPair(run, text, at) {
    timedKey(run, "ShiftLeft", "keydown", text, at);
    timedKey(run, "ShiftLeft", "keyup", text, at + 10);
    timedKey(run, "ShiftLeft", "keydown", text, at + 20);
    timedKey(run, "ShiftLeft", "keyup", text, at + 30);
}
function burst(scenario = "active", deliver = true) {
    const run = createRun("text", scenario, "burst5"), suffix = scenario === "space" ? " " : "";
    const ru = "привет" + suffix, en = "ghbdtn" + suffix;
    run.observe(shot(ru, { at: 0 })); let text = ru;
    for (let i = 0; i < 5; i++) {
        timedPair(run, text, 10 + i * 80);
        if (deliver) { text = i % 2 === 0 ? en : ru; run.observe(shot(text, { at: 45 + i * 80 })); }
    }
    return { run, ru, en };
}
test("five rapid gestures retain all targets and require final decoder and deletion", () => {
    for (const scenario of ["active", "space"]) {
        const { run, ru, en } = burst(scenario);
        assert.equal(run.state.step, 2); assert.equal(run.state.status, "RUNNING");
        assert.deepEqual(run.state.burstTargets.map(x => x.text), [en, ru, en, ru, en]);
        assert.equal(run.state.milestones[1].shiftTaps, 10);
        assert.deepEqual(run.state.timings.map(x => x.keydownToTargetMs), [15, 15, 15, 15, 15]);
        assert.deepEqual(run.state.timings.map(x => x.releaseToTargetMs), [5, 5, 5, 5, 5]);
        timedKey(run, "KeyZ", "keydown", en, 420); run.observe(shot(en + "z", { at: 421 }));
        timedKey(run, "Backspace", "keydown", en + "z", 430); run.observe(shot(en, { at: 431 }));
        assert.equal(run.state.status, "PASS_DOM"); assert.equal(run.state.milestones.length, 4);
    }
});
test("an unchanged end or a lost burst target cannot hide missing conversions", () => {
    const { run, en } = burst("active", false);
    run.observe(shot(en, { at: 370 }));
    assert.equal(run.state.status, "RUNNING"); assert.equal(run.state.step, 1);
    assert.equal(run.state.burstTargets.length, 1);
});
test("five target surfaces without ten observed releases cannot pass", () => {
    const run = createRun("text", "active", "burst5"); run.observe(shot("привет", { at: 0 }));
    for (const text of ["ghbdtn", "привет", "ghbdtn", "привет", "ghbdtn"]) run.observe(shot(text, { at: 10 }));
    assert.equal(run.state.burstTargets.length, 0); assert.equal(run.state.step, 1);
    assert.equal(run.state.status, "FAIL_VISIBLE"); assert.equal(run.state.reason, "TARGET_WITHOUT_CORRESPONDING_PAIR");
});
test("all targets arriving after the last pair are a timing failure, not a burst pass", () => {
    const { run, ru, en } = burst("active", false);
    [en, ru, en, ru, en].forEach((text, i) => run.observe(shot(text, { at: 400 + i * 5 })));
    assert.equal(run.state.status, "FAIL_VISIBLE"); assert.equal(run.state.reason, "TARGET_NOT_READY_BEFORE_NEXT_PAIR");
    assert.equal(run.state.burstTargets.length, 5); assert.equal(run.state.timings.length, 5);
    assert.equal(summarizeTimings([run.state]).samples, 5);
});
test("the decoder after five gestures must type Latin z", () => {
    const { run, en } = burst(); timedKey(run, "KeyZ", "keydown", en, 420);
    run.observe(shot(en + "я", { at: 421 }));
    assert.equal(run.state.status, "FAIL_VISIBLE"); assert.equal(run.state.reason, "NEXT_LETTER_WRONG_LAYOUT");
});
test("speed summary retains slow and failed samples and reports missing timing", () => {
    const a = { sequence: "burst5", status: "FAIL_VISIBLE", timings: [1, 2, 3, 4, 100].map(keydownToTargetMs => ({ keydownToTargetMs })) };
    const b = { sequence: "roundtrip", status: "BLOCKED", timings: [{ keydownToTargetMs: null }] };
    const stats = summarizeTimings([a, b]);
    assert.equal(stats.samples, 5); assert.equal(stats.missing, 2);
    assert.equal(stats.medianMs, 3); assert.equal(stats.p95Ms, 100); assert.equal(stats.maxMs, 100);
});
test("absent or reordered clocks stay unknown rather than becoming zero latency", () => {
    const run = roundTrip("active");
    assert.equal(run.state.status, "PASS_DOM"); assert.equal(summarizeTimings([run.state]).samples, 0);
    assert.equal(summarizeTimings([run.state]).missing, 2);
    const { run: other } = burst(); other.state.timings[0].keydownToTargetMs = -1;
    assert.equal(summarizeTimings([other.state]).samples, 4);
});
test("a burst forbids intervening printable or modifier keys and keeps partial timings", () => {
    for (const code of ["KeyZ", "Backspace", "Space", "Tab", "ControlLeft"]) {
        const run = createRun("text", "active", "burst5"); run.observe(shot("привет", { at: 0 }));
        timedPair(run, "привет", 10); run.observe(shot("ghbdtn", { at: 45 }));
        timedKey(run, code, "keydown", "ghbdtn", 50);
        assert.equal(run.state.status, "BLOCKED"); assert.equal(run.state.reason, "EXTRANEOUS_KEY_DURING_BURST");
        assert.equal(summarizeTimings([run.state]).samples, 1); assert.equal(summarizeTimings([run.state]).missing, 4);
    }
});
test("an early reversal cannot supply the next pair's missing target", () => {
    const run = createRun("text", "active", "burst5"); run.observe(shot("привет", { at: 0 }));
    timedPair(run, "привет", 10); run.observe(shot("ghbdtn", { at: 45 }));
    run.observe(shot("привет", { at: 50 }));
    timedPair(run, "привет", 90);
    assert.equal(run.state.status, "FAIL_VISIBLE"); assert.equal(run.state.reason, "TARGET_WITHOUT_CORRESPONDING_PAIR");
    assert.equal(run.state.burstTargets.length, 1); assert.equal(summarizeTimings([run.state]).samples, 1);
});
test("a pre-release target that disappears cannot donate its old timestamp to a later pair", () => {
    const run = createRun("text", "active", "burst5"); run.observe(shot("привет", { at: 0 }));
    timedKey(run, "ShiftLeft", "keydown", "привет", 10); timedKey(run, "ShiftLeft", "keyup", "привет", 20);
    timedKey(run, "ShiftLeft", "keydown", "привет", 30);
    run.observe(shot("ghbdtn", { at: 35 })); run.observe(shot("привет", { at: 36 }));
    timedKey(run, "ShiftLeft", "keyup", "привет", 40);
    timedKey(run, "ShiftLeft", "keydown", "привет", 90); timedKey(run, "ShiftLeft", "keyup", "привет", 100);
    timedKey(run, "ShiftLeft", "keydown", "привет", 110);
    run.observe(shot("ghbdtn", { at: 115 })); run.observe(shot("привет", { at: 116 }));
    timedKey(run, "ShiftLeft", "keyup", "привет", 120);
    for (let i = 2; i < 5; i++) {
        timedPair(run, i % 2 ? "ghbdtn" : "привет", 10 + i * 80);
        run.observe(shot(i % 2 ? "привет" : "ghbdtn", { at: 45 + i * 80 }));
    }
    assert.equal(run.state.status, "FAIL_VISIBLE"); assert.equal(run.state.reason, "TARGET_NOT_READY_BEFORE_NEXT_PAIR");
    assert.equal(run.state.burstTargets[0].at, 115); assert.equal(run.state.timings.length, 5);
});
