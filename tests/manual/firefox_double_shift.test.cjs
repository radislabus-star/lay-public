"use strict";
const test = require("node:test");
const assert = require("node:assert/strict");
const { createRun, FIELD_IDS, CASES, summarizeTimings, summarizeOperations } = require("./firefox_double_shift.js");
const shot = (text, extra = {}) => ({ fieldId: "text", text, trusted: true, focused: true, composing: false, caret: text.length, anchor: text.length, ...extra });
test("an English-first case reaches the Russian target rather than the greeting setup", () => {
    const run = createRun("text", "active", "roundtrip", "english-first");
    run.observe(shot("ghjdthrf"));
    assert.equal(run.phase().id, "toggle_ru");
    pair(run, "проверка");
    assert.equal(run.phase().id, "decoder_ru");
});
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


const corpus = [
    ["greeting", "привет", "ghbdtn", "ru"], ["short", "а", "f", "ru"],
    ["long", "параллелепипед", "gfhfkktktgbgtl", "ru"], ["mixed-case", "ПрИвЕт", "GhBdTn", "ru"],
    ["previous-word", "тест, привет", "тест, ghbdtn", "ru"], ["english-first", "проверка", "ghjdthrf", "en"],
];
function modelDriver(run) {
    let current = "", clock = 0, geometry = {};
    const snapshot = (text, extra = {}) => shot(text, { fieldId: run.state.fieldId, at: clock, ...extra });
    const input = (text, extra = {}) => { clock += 7; current = text; geometry = extra; run.observe(snapshot(text, extra)); };
    const action = (code, text, event = {}, extra = {}) => {
        clock += 20;
        run.key({ fieldId: run.state.fieldId, code, type: "keydown", trusted: true, repeat: false, ...event }, snapshot(current, geometry));
        input(text, extra);
        clock += 3;
        run.key({ fieldId: run.state.fieldId, code, type: "keyup", trusted: true, repeat: false, ...event }, snapshot(current, geometry));
    };
    const toggle = (text) => {
        for (let i = 0; i < 2; i++) {
            clock += 20; run.key({ fieldId: run.state.fieldId, code: "ShiftLeft", type: "keydown", trusted: true, repeat: false }, snapshot(current, geometry));
            clock += 10; run.key({ fieldId: run.state.fieldId, code: "ShiftLeft", type: "keyup", trusted: true, repeat: false }, snapshot(current, geometry));
        }
        input(text);
    };
    const visit = () => run.visitField({ fieldId: FIELD_IDS[(FIELD_IDS.indexOf(run.state.fieldId) + 1) % FIELD_IDS.length],
        text: "", caret: 0, anchor: 0, focused: true, trusted: true, event: "focusin", at: ++clock });
    return { input, action, toggle, visit };
}
test("curated corpus matches physical key projections and has distinct functional cases", () => {
    const lowerRu = "йцукенгшщзхъфывапролджэячсмитьбю", lowerEn = "qwertyuiop[]asdfghjkl;'zxcvbnm,.",
        upperRu = lowerRu.toUpperCase(), upperEn = 'QWERTYUIOP{}ASDFGHJKL:"ZXCVBNM<>';
    for (const [id, ru, en] of corpus) {
        const prefix = id === "previous-word" ? "тест, " : "";
        const projected = [...ru.slice(prefix.length)].map(char => lowerRu.includes(char) ? lowerEn[lowerRu.indexOf(char)] : upperEn[upperRu.indexOf(char)]).join("");
        assert.equal(prefix + projected, en);
    }
    assert.equal(CASES.length, 15); assert.equal(new Set(CASES.map(x => x.id)).size, 15);
    assert.equal(CASES.filter(x => x.family === "manual").length, 6);
    assert.throws(() => createRun("text", "active", "roundtrip", "unknown-profile"));
});
test("six corpus plans exercise roundtrips and five targets in all ten field identities", () => {
    let executions = 0;
    for (const fieldId of FIELD_IDS) for (const [id, rawRu, rawEn, mode] of corpus) for (const scenario of ["active", "space"]) for (const sequence of ["roundtrip", "burst5"]) {
        const suffix = scenario === "space" ? " " : "", ru = rawRu + suffix, en = rawEn + suffix;
        const from = mode === "ru" ? ru : en, to = mode === "ru" ? en : ru;
        const run = createRun(fieldId, scenario, sequence, id), driver = modelDriver(run);
        driver.input(from);
        if (sequence === "burst5") {
            for (let i = 0; i < 5; i++) driver.toggle(i % 2 ? from : to);
            driver.action("KeyZ", to + (mode === "ru" ? "z" : "я")); driver.action("Backspace", to);
        } else {
            driver.toggle(to); driver.action("KeyZ", to + (mode === "ru" ? "z" : "я")); driver.action("Backspace", to);
            driver.toggle(from); driver.action("KeyZ", from + (mode === "ru" ? "я" : "z")); driver.action("Backspace", from);
        }
        assert.equal(run.state.status, "PASS_DOM", id + ":" + fieldId + ":" + run.state.reason);
        assert.equal(run.state.timings.length, sequence === "burst5" ? 5 : 2);
        assert.equal(summarizeTimings([run.state]).missing, 0);
        executions++;
    }
    assert.equal(executions, 240);
});
const functionalScripts = {
    "space-en-ru": [["type", "ghbdtn"], ["Space", "привет "], ["KeyZ", "привет я"], ["Backspace", "привет "]],
    "space-ru-en": [["type", "дфн"], ["Space", "lay "], ["KeyZ", "lay z"], ["Backspace", "lay "]],
    "clean-en": [["type", "max"], ["Space", "max "], ["KeyZ", "max z"], ["Backspace", "max "]],
    "moved-boundary": [["type", "должн ыбыть"], ["Space", "должны быть "], ["KeyZ", "должны быть я"], ["Backspace", "должны быть "]],
    "typo-undo": [["type", "превет"], ["Space", "привет "], ["pair", "превет "], ["KeyZ", "превет я"], ["Backspace", "превет "]],
    "completion-tab": [["type", "пров"], ["Tab", "проверка "], ["pair", "ghjdthrf "], ["KeyZ", "ghjdthrf z"], ["Backspace", "ghjdthrf "],
        ["pair", "проверка "], ["KeyZ", "проверка я"], ["Backspace", "проверка "]],
    "caret-edit": [["type", "привет"], ["ArrowLeft", "привет", {}, { caret: 5, anchor: 5 }], ["ArrowLeft", "привет", {}, { caret: 4, anchor: 4 }],
        ["ArrowLeft", "привет", {}, { caret: 3, anchor: 3 }], ["Backspace", "првет", {}, { caret: 2, anchor: 2 }],
        ["KeyB", "привет", {}, { caret: 3, anchor: 3 }], ["End", "привет", {}, { caret: 6, anchor: 6 }],
        ["pair", "ghbdtn"], ["KeyZ", "ghbdtnz"], ["Backspace", "ghbdtn"], ["pair", "привет"], ["KeyZ", "приветя"], ["Backspace", "привет"]],
    "selection-replace": [["type", "тест привет"], ["KeyA", "тест привет", { ctrlKey: true }, { caret: 11, anchor: 0 }], ["KeyF", "а"],
        ["pair", "f"], ["KeyZ", "fz"], ["Backspace", "f"], ["pair", "а"], ["KeyZ", "ая"], ["Backspace", "а"]],
    "focus-return": [["type", "привет"], ["type", "привет", {}, { focused: false, event: "focusout" }], ["visit"], ["type", "привет", {}, { focused: true, event: "focusin" }],
        ["KeyS", "приветы"], ["Backspace", "привет"], ["pair", "ghbdtn"], ["KeyZ", "ghbdtnz"], ["Backspace", "ghbdtn"],
        ["pair", "привет"], ["KeyZ", "приветя"], ["Backspace", "привет"]],
};
for (const [id, script] of Object.entries(functionalScripts)) test("functional plan " + id + " requires all effects in ten scoped field identities", () => {
    for (const fieldId of FIELD_IDS) {
        const run = createRun(fieldId, "active", "roundtrip", id), driver = modelDriver(run);
        for (const [code, text, event, extra] of script) {
            if (code === "type") driver.input(text, extra);
            else if (code === "pair") driver.toggle(text);
            else if (code === "visit") driver.visit();
            else driver.action(code, text, event, extra);
        }
        assert.equal(run.state.status, "PASS_DOM", fieldId + ":" + run.state.reason + ":" + run.phase()?.id);
        assert.equal(run.state.sequence, "functional");
        assert.equal(run.state.timings.length, run.state.expectedToggles);
        for (const summary of Object.values(summarizeOperations([run.state]))) assert.equal(summary.missing, 0);
    }
});
test("Space and Tab effects without observed trigger keys remain blocked and missing", () => {
    for (const [id, initial, expected] of [["space-en-ru", "ghbdtn", "привет "], ["completion-tab", "пров", "проверка "]]) {
        const run = createRun("text", "active", "roundtrip", id);
        run.observe(shot(initial)); run.observe(shot(expected, { event: "input" }));
        assert.equal(run.state.status, "BLOCKED"); assert.equal(run.state.reason, "REQUIRED_TRIGGER_UNOBSERVED");
        assert.equal(Object.values(summarizeOperations([run.state]))[0].missing, 1);
    }
});
test("undo must restore the recorded typo rather than project the corrected word", () => {
    const run = createRun("text", "active", "roundtrip", "typo-undo"), d = modelDriver(run);
    d.input("превет"); d.action("Space", "привет "); d.toggle("ghbdtn ");
    assert.equal(run.state.status, "RUNNING"); assert.equal(run.phase().id, "toggle_undo");
    assert.equal(run.state.timings.length, 0);
});
test("previous text damage cannot be hidden by a later restored prefix", () => {
    const run = createRun("text", "active", "roundtrip", "previous-word"), d = modelDriver(run);
    d.input("тест, привет"); d.toggle("ghbdtn"); d.input("тест, ghbdtn");
    assert.equal(run.state.status, "FAIL_VISIBLE"); assert.equal(run.state.reason, "PREVIOUS_TEXT_CHANGED");
});
test("cursor and selection cases block unknown geometry and refuse collapsed selection", () => {
    for (const id of ["caret-edit", "selection-replace"]) {
        const run = createRun("text", "active", "roundtrip", id), initial = id === "caret-edit" ? "привет" : "тест привет";
        run.observe(shot(initial));
        run.key({ fieldId: "text", code: id === "caret-edit" ? "ArrowLeft" : "KeyA", type: "keydown", trusted: true, ctrlKey: id === "selection-replace" }, shot(initial, { caret: null, anchor: null }));
        run.observe(shot(initial, { caret: null, anchor: null }));
        if (id === "caret-edit") for (let i = 0; i < 2; i++) run.key({ fieldId: "text", code: "ArrowLeft", type: "keydown", trusted: true }, shot(initial, { caret: null, anchor: null }));
        assert.equal(run.state.status, "BLOCKED"); assert.equal(run.state.reason, "CARET_OR_SELECTION_UNAVAILABLE");
    }
    const selected = createRun("text", "active", "roundtrip", "selection-replace"), d = modelDriver(selected);
    d.input("тест привет"); d.action("KeyA", "тест привет", { ctrlKey: true });
    assert.equal(selected.phase().id, "select_all");
});
test("functional trigger requires trust correct field and correct modifier state", () => {
    for (const change of [{ trusted: false }, { fieldId: "url" }, { ctrlKey: true }]) {
        const run = createRun("text", "active", "roundtrip", "space-en-ru"); run.observe(shot("ghbdtn"));
        run.key({ fieldId: "text", code: "Space", type: "keydown", trusted: true, ...change }, shot("ghbdtn"));
        run.observe(shot("привет "));
        assert.equal(run.phase().id, "apply_space"); assert.notEqual(run.state.status, "PASS_DOM");
    }
});
test("focus-return requires an actual leave return and subsequent fresh gesture", () => {
    const run = createRun("text", "active", "roundtrip", "focus-return"); run.observe(shot("привет"));
    run.observe(shot("привет", { focused: false, event: "focusout" }));
    run.observe(shot("привет", { event: "input" })); assert.equal(run.phase().id, "return_focus");
    run.visitField(shot("", { fieldId: "search", event: "focusin" }));
    run.observe(shot("привет", { event: "focusin" }));
    assert.equal(run.state.status, "RUNNING"); assert.equal(run.phase().id, "after_focus_letter");
    const stale = createRun("text", "active", "roundtrip", "focus-return"); stale.observe(shot("привет"));
    stale.observe(shot("привет", { focused: false, event: "focusout" })); stale.observe(shot("приветпривет", { event: "focusin" }));
    assert.equal(stale.state.status, "FAIL_VISIBLE");
});
test("functional timing is separate from Shift and retains failed missing clocks", () => {
    const failed = createRun("text", "active", "roundtrip", "space-en-ru"); failed.observe(shot("ghbdtn")); failed.stop("NO_TRIGGER");
    const completed = createRun("text", "active", "roundtrip", "space-en-ru"), d = modelDriver(completed);
    for (const [code, text] of functionalScripts["space-en-ru"]) code === "type" ? d.input(text) : d.action(code, text);
    const s = summarizeOperations([failed.state, completed.state]).space;
    assert.equal(s.samples, 1); assert.equal(s.missing, 1); assert.equal(s.medianMs, 7);
    assert.equal(summarizeTimings([failed.state, completed.state]).missing, 0);
    const absentClock = createRun("text", "active", "roundtrip", "clean-en"); absentClock.observe(shot("max"));
    absentClock.key({ fieldId: "text", code: "Space", type: "keydown", trusted: true }, shot("max")); absentClock.observe(shot("max "));
    assert.equal(summarizeOperations([absentClock.state]).space.samples, 0);
    assert.equal(summarizeOperations([absentClock.state]).space.missing, 1);
    assert.equal(summarizeOperations([absentClock.state]).space.medianMs, null);
});
test("wrong decoder after Space is a visible failure rather than a corrected-word pass", () => {
    const run = createRun("text", "active", "roundtrip", "space-en-ru"), d = modelDriver(run);
    d.input("ghbdtn"); d.action("Space", "привет "); d.action("KeyZ", "привет z");
    assert.equal(run.state.status, "FAIL_VISIBLE"); assert.equal(run.state.reason, "NEXT_LETTER_WRONG_LAYOUT");
});
test("Tab needs one space ended composition and retained focus", () => {
    const run = createRun("text", "active", "roundtrip", "completion-tab"), d = modelDriver(run);
    d.input("пров"); d.action("Tab", "проверка", {}, { composing: false }); assert.equal(run.phase().id, "accept_tab");
    d.input("проверка ", { composing: true }); assert.equal(run.phase().id, "accept_tab");
    d.input("проверка ", { composing: false }); assert.equal(run.phase().id, "toggle_en");
    const lost = createRun("text", "active", "roundtrip", "completion-tab"); lost.observe(shot("пров")); lost.observe(shot("проверка ", { focused: false }));
    assert.equal(lost.state.status, "BLOCKED");
});

test("Space and Tab cannot be repaired manually or triggered twice to obtain PASS", () => {
    for (const [id, initial, trigger, wrong, target] of [["space-en-ru", "ghbdtn", "Space", "ghbdtn ", "привет "],
                                                    ["completion-tab", "пров", "Tab", "пров ", "проверка "]]) {
        for (const repair of [{ code: "KeyA", ctrlKey: true }, { code: trigger }]) {
            const run = createRun("text", "active", "roundtrip", id), d = modelDriver(run);
            d.input(initial); d.action(trigger, wrong);
            run.key({ fieldId: "text", type: "keydown", trusted: true, ...repair }, shot(wrong));
            d.input(target); d.action("KeyZ", target + "я"); d.action("Backspace", target);
            assert.equal(run.state.status, "BLOCKED"); assert.equal(run.state.reason, "UNEXPECTED_KEY_DURING_ACTION");
            assert.equal(run.state.operationTimings.length, 0);
            assert.equal(Object.values(summarizeOperations([run.state]))[0].missing, 1);
        }
    }
});
test("focus-return requires a trusted visit to another registered field", () => {
    for (const destination of [null, { fieldId: "toolbar" }, { fieldId: "text" }, { fieldId: "search", trusted: false }]) {
        const run = createRun("text", "active", "roundtrip", "focus-return");
        run.observe(shot("привет")); run.observe(shot("привет", { focused: false, event: "focusout" }));
        if (destination) run.visitField(shot("", { event: "focusin", ...destination }));
        run.observe(shot("привет", { event: "focusin" }));
        assert.equal(run.state.status, "BLOCKED"); assert.equal(run.state.focusVisitedField, null);
        assert.ok(["FOCUS_DESTINATION_UNAVAILABLE", "OTHER_FIELD_FOCUS_UNOBSERVED"].includes(run.state.reason));
    }
    const bounded = createRun("text", "active", "roundtrip", "focus-return");
    bounded.observe(shot("привет")); bounded.observe(shot("привет", { focused: false, event: "focusout" }));
    for (let i = 0; i < 130; i++) bounded.visitField(shot("", { fieldId: "search", event: "focusin" }));
    assert.equal(bounded.state.status, "BLOCKED"); assert.equal(bounded.state.reason, "OBSERVER_LIMIT");
    assert.equal(bounded.state.observations.length, 128);
});
test("missing completion is BLOCKED while an observed Tab action failure remains FAIL", () => {
    for (const method of ["escape", "button"]) {
        const missing = createRun("text", "active", "roundtrip", "completion-tab"); missing.observe(shot("пров"));
        assert.equal(missing.preconditionPending(), true);
        if (method === "escape") missing.key({ fieldId: "text", type: "keydown", code: "Escape", trusted: true }, shot("пров"));
        else missing.reportFailure();
        assert.equal(missing.state.status, "BLOCKED"); assert.equal(missing.state.reason, "EXPECTED_COMPLETION_UNAVAILABLE");
        assert.equal(summarizeOperations([missing.state]).tab.missing, 1);
    }
    const failed = createRun("text", "active", "roundtrip", "completion-tab"), d = modelDriver(failed);
    d.input("пров"); d.action("Tab", "пров"); assert.equal(failed.preconditionPending(), false);
    failed.key({ fieldId: "text", type: "keydown", code: "Escape", trusted: true }, shot("пров"));
    assert.equal(failed.state.status, "FAIL_REPORTED"); assert.equal(failed.state.reason, "USER_REPORTED_FAILURE");
});
