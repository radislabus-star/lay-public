/* Test observer only: no layout conversion, key injection or IME API calls. */
(function (root) {
    "use strict";
    const FIELD_IDS = ["text", "search", "url", "tel", "email", "textarea", "rich", "plain", "iframe-input", "iframe-rich"];
    const TERMINAL = new Set(["PASS_DOM", "FAIL_REPORTED", "FAIL_VISIBLE", "BLOCKED"]);
    const caseLibrary = typeof module !== "undefined" && module.exports ? require("./browser_ime_cases.js") : root.LayBrowserCases;

    function timing(pair, target) {
        const known = value => Number.isFinite(value);
        const downDelta = known(target.at) && known(pair.secondDownAt) ? target.at - pair.secondDownAt : null;
        return { ...pair, target: target.text, firstTargetAt: target.at ?? null, source: "DOM performance.now",
            keydownToTargetMs: downDelta !== null && downDelta >= 0 ? downDelta : null,
            releaseToTargetMs: known(target.at) && known(pair.secondReleaseAt) ? target.at - pair.secondReleaseAt : null };
    }
    function summarizeTimings(records) {
        const values = records.flatMap(record => (record.timings || []).map(x => x.keydownToTargetMs)).filter(x => Number.isFinite(x) && x >= 0).sort((a, b) => a - b);
        const n = values.length, middle = Math.floor(n / 2);
        return { source: "DOM second Shift keydown to first exact target; native injection timing requires an external receipt",
            samples: n, missing: records.reduce((sum, r) => sum + (r.expectedToggles ?? (r.sequence === "burst5" ? 5 : 2)), 0) - n,
            medianMs: n ? (n % 2 ? values[middle] : (values[middle - 1] + values[middle]) / 2) : null,
            p95Ms: n ? values[Math.ceil(n * .95) - 1] : null, maxMs: n ? values[n - 1] : null };
    }
    function summarizeOperations(records) {
        const operations = new Set(records.flatMap(r => r.expectedOperations || []));
        return Object.fromEntries([...operations].map(operation => {
            const values = records.flatMap(r => (r.operationTimings || []).filter(x => x.operation === operation).map(x => x.keydownToTargetMs))
                .filter(x => Number.isFinite(x) && x >= 0).sort((a, b) => a - b);
            const n = values.length, middle = Math.floor(n / 2);
            return [operation, { source: "DOM required keydown to accepted exact target", samples: n,
                missing: records.reduce((sum, r) => sum + (r.expectedOperations || []).filter(x => x === operation).length, 0) - n,
                medianMs: n ? (n % 2 ? values[middle] : (values[middle - 1] + values[middle]) / 2) : null,
                p95Ms: n ? values[Math.ceil(n * .95) - 1] : null, maxMs: n ? values[n - 1] : null }];
        }));
    }
    function createRun(fieldId, scenario = "active", sequence = "roundtrip", caseId = "greeting") {
        if (!FIELD_IDS.includes(fieldId) || !["active", "space"].includes(scenario) || !["roundtrip", "burst5"].includes(sequence)) throw new Error("Invalid case");
        const plan = caseLibrary.buildCase(caseId, scenario, sequence), stages = plan.stages;
        sequence = plan.sequence;
        const observationLimit = sequence === "burst5" ? 512 : Math.max(128, stages[0].text.length * 12);
        const state = { ...plan, fieldId, status: "RUNNING", step: 0, stageCount: stages.length, milestones: [], observations: [], timings: [], operationTimings: [], burstTargets: [], focusVisitedField: null, reason: null };
        let pressed = new Set(), taps = 0, shiftSide = null, decoderKey = false, backspaceKey = false;
        let pairStart = null, secondDownAt = null, firstTargetAt = null, pairs = [];
        let requiredKeys = 0, triggerAt = null;
        const phase = () => stages[state.step] || null;
        function stop(reason, status = "BLOCKED") { if (!TERMINAL.has(state.status)) { state.status = status; state.reason = reason; } }
        const preconditionPending = () => Boolean(phase()?.preconditionReason && requiredKeys === 0);
        function reportFailure() {
            if (TERMINAL.has(state.status)) return;
            if (preconditionPending()) stop(phase().preconditionReason);
            else stop("USER_REPORTED_FAILURE", "FAIL_REPORTED");
        }
        function visitField(snapshot) {
            if (state.status !== "RUNNING" || phase().id !== "return_focus" || !snapshot.trusted || snapshot.event !== "focusin" || !snapshot.focused) return;
            if (!FIELD_IDS.includes(snapshot.fieldId) || snapshot.fieldId === fieldId) { stop("FOCUS_DESTINATION_UNAVAILABLE"); return; }
            if (state.observations.length >= observationLimit) { stop("OBSERVER_LIMIT"); return; }
            if (typeof snapshot.text !== "string" || snapshot.text.length > 256) { stop("UNSUPPORTED_OBSERVATION"); return; }
            state.focusVisitedField = snapshot.fieldId;
            state.observations.push({ ...snapshot, step: "visit_other_field" });
        }
        function observe(snapshot, advanceMilestone = true) {
            if (TERMINAL.has(state.status) || snapshot.fieldId !== fieldId || !snapshot.trusted) return;
            if (state.observations.length >= observationLimit) { stop("OBSERVER_LIMIT"); return; }
            state.observations.push({ ...snapshot, step: phase().id });
            const stage = phase();
            if (!snapshot.focused && stage.focused !== false) {
                if (stage.id === "return_focus") {
                    if (snapshot.text !== stage.text) stop("TEXT_CHANGED_WHILE_AWAY", "FAIL_VISIBLE");
                    return;
                }
                stop("FOCUS_CHANGED"); return;
            }
            if (typeof snapshot.text !== "string" || snapshot.text.length > 256) { stop("UNSUPPORTED_OBSERVATION"); return; }
            if (/Paste|Drop/.test(snapshot.inputType || "")) { stop("PASTE_OR_DROP"); return; }
            if (!advanceMilestone) return;
            if (state.step > 0 && state.stablePrefix && !snapshot.text.startsWith(state.stablePrefix)) {
                stop("PREVIOUS_TEXT_CHANGED", "FAIL_VISIBLE"); return;
            }
            if (stage.id.startsWith("decoder_") && decoderKey && snapshot.text === stage.wrongText) {
                stop("NEXT_LETTER_WRONG_LAYOUT", "FAIL_VISIBLE"); return;
            }
            if (stage.focusEvent) {
                if (snapshot.text !== stage.text) { stop("TEXT_CHANGED_DURING_FOCUS_TRANSITION", "FAIL_VISIBLE"); return; }
                if (snapshot.event !== stage.focusEvent || snapshot.focused !== stage.focused) return;
                if (stage.id === "return_focus" && !state.focusVisitedField) { stop("OTHER_FIELD_FOCUS_UNOBSERVED"); return; }
            }
            if (stage.requiredKey && requiredKeys < (stage.requiredKey.count || 1)) {
                if (stage.operation && snapshot.text === stage.text && ["input", "compositionend"].includes(snapshot.event)) stop("REQUIRED_TRIGGER_UNOBSERVED");
                return;
            }
            if (stage.caret !== undefined || stage.selection) {
                if (!Number.isInteger(snapshot.caret) || !Number.isInteger(snapshot.anchor)) { stop("CARET_OR_SELECTION_UNAVAILABLE"); return; }
                if (stage.selection) {
                    if (Math.min(snapshot.caret, snapshot.anchor) !== stage.selection[0] || Math.max(snapshot.caret, snapshot.anchor) !== stage.selection[1]) return;
                } else if (snapshot.caret !== stage.caret || snapshot.anchor !== stage.caret) return;
            }
            if (stage.composing !== undefined && snapshot.composing !== stage.composing) return;
            if (stage.id === "toggle_burst5") {
                const expected = state.burstTargets.length % 2 === 0 ? stage.text : stage.alternateText;
                if (state.burstTargets.length < 5 && snapshot.text === expected) {
                    const index = state.burstTargets.length;
                    const pair = pairs[index] || (pairs.length === index && pairStart !== null ? { firstDownAt: pairStart } : null);
                    if (!pair || (Number.isFinite(snapshot.at) && Number.isFinite(pair.firstDownAt) && snapshot.at < pair.firstDownAt)) {
                        stop("TARGET_WITHOUT_CORRESPONDING_PAIR", "FAIL_VISIBLE"); return;
                    }
                    if (firstTargetAt === null && Number.isFinite(snapshot.at)) firstTargetAt = snapshot.at;
                    if (pairs.length > state.burstTargets.length) {
                        const target = { text: expected, at: firstTargetAt };
                        state.burstTargets.push(target); state.timings.push(timing(pair, target)); firstTargetAt = null;
                    }
                } else firstTargetAt = null;
                if (state.burstTargets.length !== 5) return;
                if (state.timings.some((x, i) => i < 4 && Number.isFinite(x.firstTargetAt) && Number.isFinite(pairs[i + 1]?.firstDownAt) && x.firstTargetAt >= pairs[i + 1].firstDownAt)) {
                    stop("TARGET_NOT_READY_BEFORE_NEXT_PAIR", "FAIL_VISIBLE"); return;
                }
            } else if (stage.id.startsWith("toggle_") && snapshot.text === stage.text && firstTargetAt === null && Number.isFinite(snapshot.at)) firstTargetAt = snapshot.at;
            if (snapshot.text !== stage.text) return;
            if (stage.id.startsWith("toggle_") && (taps !== (sequence === "burst5" ? 10 : 2) || pressed.size !== 0)) return;
            if (stage.id.startsWith("decoder_") && !decoderKey) return;
            if (stage.id.startsWith("erase_") && !backspaceKey && snapshot.inputType !== "deleteContentBackward") return;
            if (stage.id.startsWith("toggle_") && sequence !== "burst5") state.timings.push(timing(pairs[0] || {}, { text: stage.text, at: firstTargetAt }));
            if (stage.operation) {
                const delta = Number.isFinite(snapshot.at) && Number.isFinite(triggerAt) ? snapshot.at - triggerAt : null;
                state.operationTimings.push({ operation: stage.operation, triggerAt, targetAt: snapshot.at ?? null,
                    keydownToTargetMs: delta !== null && delta >= 0 ? delta : null });
            }
            state.milestones.push({ stage: stage.id, ...snapshot, shiftTaps: stage.id.startsWith("toggle_") ? taps : null });
            state.step += 1;
            pressed = new Set(); taps = 0; shiftSide = null; decoderKey = false; backspaceKey = false;
            pairStart = null; secondDownAt = null; firstTargetAt = null; pairs = [];
            requiredKeys = 0; triggerAt = null;
            if (state.step === stages.length) state.status = "PASS_DOM";
        }
        function key(event, snapshot) {
            if (TERMINAL.has(state.status) || event.fieldId !== fieldId || snapshot.fieldId !== fieldId || !event.trusted || !snapshot.trusted) return;
            const observed = { ...snapshot, code: event.code, repeat: Boolean(event.repeat), ctrlKey: Boolean(event.ctrlKey), altKey: Boolean(event.altKey), metaKey: Boolean(event.metaKey) };
            if (event.type === "keydown" && event.code === "Escape" && !event.repeat && !event.ctrlKey && !event.altKey && !event.metaKey) {
                observe(observed, false);
                if (state.status === "RUNNING") reportFailure();
                return;
            }
            // A pre-key snapshot may settle the prior stage before this key's effect.
            if (event.type === "keydown" && !phase().focusEvent) observe(observed);
            if (TERMINAL.has(state.status)) return;
            const stage = phase();
            if (stage.requiredKey && event.type === "keydown" && !/^(Shift|Control|Alt|Meta)(Left|Right)$/.test(event.code)) {
                if (event.repeat || event.code !== stage.requiredKey.code ||
                    Boolean(event.ctrlKey) !== Boolean(stage.requiredKey.ctrlKey) || event.altKey || event.metaKey ||
                    requiredKeys >= (stage.requiredKey.count || 1)) {
                    stop("UNEXPECTED_KEY_DURING_ACTION"); return;
                }
                requiredKeys += 1;
                if (triggerAt === null && Number.isFinite(snapshot.at)) triggerAt = snapshot.at;
            }
            if (stage.id.startsWith("toggle_") && event.type === "keydown" && (event.ctrlKey || event.altKey || event.metaKey || !/^Shift(Left|Right)$/.test(event.code))) {
                stop(sequence === "burst5" ? "EXTRANEOUS_KEY_DURING_BURST" : "EXTRANEOUS_KEY_DURING_TOGGLE"); return;
            }
            if (stage.id.startsWith("toggle_") && /^Shift(Left|Right)$/.test(event.code) && !event.repeat && !event.ctrlKey && !event.altKey && !event.metaKey) {
                if (event.type === "keydown") {
                    if (shiftSide && shiftSide !== event.code) { stop("MIXED_SHIFT_SIDES"); return; }
                    if (!pressed.size) { if (taps % 2 === 0) pairStart = snapshot.at; else secondDownAt = snapshot.at; }
                    shiftSide = event.code; pressed.add(event.code);
                } else if (event.type === "keyup" && pressed.delete(event.code)) {
                    taps += 1;
                    if (taps % 2 === 0) {
                        pairs.push({ firstDownAt: pairStart ?? null, secondDownAt: secondDownAt ?? null, secondReleaseAt: snapshot.at ?? null });
                        pairStart = null; secondDownAt = null;
                    }
                }
            }
            if (event.type === "keydown" && !event.ctrlKey && !event.altKey && !event.metaKey) {
                if (event.code === "KeyZ") decoderKey = true;
                if (event.code === "Backspace") backspaceKey = true;
            }
            observe(observed);
        }
        return { state, observe, key, stop, phase, visitField, reportFailure, preconditionPending };
    }

    const api = { FIELD_IDS, CASES: caseLibrary.CASES, createRun, summarizeTimings, summarizeOperations };
    if (typeof module !== "undefined" && module.exports) module.exports = api;
    root.LayFirefoxProbe = api;
    if (typeof document === "undefined") return;
    const declaredBrowser = new URLSearchParams(location.search).get("browser") === "chrome" ? "chrome" : "firefox";
    const browserLabel = declaredBrowser === "chrome" ? "Chrome" : "Firefox";
    document.title = "Lay — Double Shift в 10 полях " + browserLabel;
    document.querySelector("h1").textContent = "Lay: проверки ввода в 10 полях " + browserLabel;
    const fields = new Map(), records = [], cards = [...document.querySelectorAll(".card")];
    const byId = id => document.getElementById(id);
    let active = null, activeIndex = -1, lockedScenario = null, lockedSequence = null, lockedCase = null, batch = 0, windowMode = null;
    for (const spec of caseLibrary.CASES) {
        const option = document.createElement("option"); option.value = spec.id; option.textContent = spec.label;
        byId("test-case").append(option);
    }
    byId("test-case").addEventListener("change", () => {
        const fixed = caseLibrary.CASES.find(x => x.id === byId("test-case").value).family !== "manual";
        byId("sequence").disabled = fixed; byId("scenario").disabled = fixed;
    });
    const textOf = el => "value" in el ? el.value : el.textContent;

    function focused(el) {
        const doc = el.ownerDocument;
        return doc.hasFocus() && doc.activeElement === el && (!doc.defaultView.frameElement || document.activeElement === doc.defaultView.frameElement);
    }
    function snapshot(id, el, event) {
        let caret = null, anchor = null;
        if ("selectionStart" in el) { caret = el.selectionEnd; anchor = el.selectionStart; }
        else {
            const sel = el.ownerDocument.getSelection();
            if (sel && sel.rangeCount && el.contains(sel.focusNode) && el.contains(sel.anchorNode)) {
                const offset = (node, end) => { const range = el.ownerDocument.createRange(); range.selectNodeContents(el); range.setEnd(node, end); return range.toString().length; };
                caret = offset(sel.focusNode, sel.focusOffset); anchor = offset(sel.anchorNode, sel.anchorOffset);
            }
        }
        const rawText = textOf(el);
        return { fieldId: id, text: rawText.replace(/\u00a0/g, " "), rawText, caret, anchor, focused: focused(el), trusted: event.isTrusted,
            event: event.type, inputType: event.inputType || null, composing: fields.get(id).composing,
            at: performance.now(), caretObserved: caret !== null };
    }
    function render() {
        for (const card of cards) {
            const latest = [...records].reverse().find(r => r.fieldId === card.dataset.field && r.batch === batch);
            const running = active && active.state.fieldId === card.dataset.field;
            card.classList.toggle("active", Boolean(running && active.state.status === "RUNNING"));
            card.classList.toggle("pass", Boolean(latest && latest.status === "PASS_DOM"));
            card.classList.toggle("fail", Boolean(latest && latest.status !== "PASS_DOM" && latest.status !== "RUNNING"));
            card.querySelector("output").textContent = !latest ? "Не проверено" : ({ RUNNING: "Шаг " + (latest.step + 1) + "/" + latest.stageCount, PASS_DOM: "Все шаги ✓", FAIL_REPORTED: "Сбой", FAIL_VISIBLE: "Сбой", BLOCKED: "Прервано" })[latest.status];
        }
        const current = FIELD_IDS.map(id => [...records].reverse().find(r => r.fieldId === id && r.batch === batch));
        const count = status => current.filter(r => r && r.status === status).length;
        byId("summary").textContent = `Пройдено: ${count("PASS_DOM")} · Сбой: ${count("FAIL_REPORTED") + count("FAIL_VISIBLE")} · Прервано: ${count("BLOCKED")} · Не проверено: ${current.filter(r => !r || r.status === "RUNNING").length}`;
        const speed = summarizeTimings(current.filter(Boolean));
        const ms = value => value === null ? "—" : Math.round(value) + " мс";
        byId("speed").textContent = `Скорость по событиям браузера: медиана ${ms(speed.medianMs)} · p95 ${ms(speed.p95Ms)} · максимум ${ms(speed.maxMs)} · замеров ${speed.samples} · без замера ${speed.missing}`;
        for (const [operation, result] of Object.entries(summarizeOperations(current.filter(Boolean)))) {
            byId("speed").textContent += ` · ${operation}: медиана ${ms(result.medianMs)}, p95 ${ms(result.p95Ms)}, max ${ms(result.maxMs)}, ${result.samples} замеров/${result.missing} без замера`;
        }
        byId("failed").disabled = !active || active.state.status !== "RUNNING";
        byId("failed").textContent = active?.preconditionPending() ? "Нет подсказки (Esc)" : "Не сработало (Esc)";
        byId("next").disabled = !active || active.state.status === "RUNNING" || activeIndex === FIELD_IDS.length - 1;
        if (!active) return;
        byId("instruction").textContent = active.state.status === "RUNNING" ? `${active.state.label}. ${activeIndex + 1}/10. ${active.phase().hint}` : active.state.status === "PASS_DOM" ? "Все ожидаемые шаги совпали. Нажми «Следующее поле»." : `Проверка остановлена: ${active.state.reason}. Результат сохранён; можно перейти дальше.`;
    }
    function attach(id, el) {
        if (fields.has(id)) return;
        fields.set(id, { el, composing: false });
        for (const type of ["input", "compositionstart", "compositionupdate", "compositionend", "keydown", "keyup", "focusin", "focusout"]) {
            el.addEventListener(type, event => {
                if (type === "compositionstart") fields.get(id).composing = true;
                if (type === "compositionend") fields.get(id).composing = false;
                if (!active) return;
                if (active.state.fieldId !== id) {
                    if (active.state.status === "RUNNING" && active.phase().id === "return_focus" && event.isTrusted) {
                        if (type === "focusin") active.visitField(snapshot(id, el, event));
                        else if (["keydown", "input", "compositionstart", "compositionupdate", "compositionend"].includes(type)) active.stop("INPUT_IN_OTHER_FIELD_DURING_FOCUS_CASE");
                        render();
                    }
                    return;
                }
                const observed = snapshot(id, el, event);
                if (type === "keydown" || type === "keyup") active.key({ fieldId: id, trusted: event.isTrusted, type, code: event.code,
                    repeat: event.repeat, ctrlKey: event.ctrlKey, altKey: event.altKey, metaKey: event.metaKey }, observed);
                else active.observe(observed);
                if (event.isTrusted && type === "keydown" && event.code === "Escape" && TERMINAL.has(active.state.status)) event.preventDefault();
                render();
            });
        }
        for (const type of ["paste", "drop"]) el.addEventListener(type, () => { if (active && active.state.fieldId === id) { active.stop("PASTE_OR_DROP"); render(); } });
    }
    for (const id of FIELD_IDS) {
        const host = byId(id);
        if (host.tagName !== "IFRAME") attach(id, host);
        else {
            const ready = () => { try { const el = host.contentDocument && host.contentDocument.getElementById("field"); if (el) attach(id, el); } catch (_) { /* Missing observer stays BLOCKED. */ } };
            host.addEventListener("load", ready); ready();
        }
    }
    window.addEventListener("blur", () => {
        if (active && !document.hasFocus()) { active.stop("WINDOW_FOCUS_LOST"); render(); }
    });
    document.addEventListener("visibilitychange", () => {
        if (active && document.hidden) { active.stop("PAGE_HIDDEN"); render(); }
    });
    function startField(index) {
        if (active && active.state.status === "RUNNING") active.stop("NEW_RUN_REQUESTED");
        activeIndex = index;
        active = createRun(FIELD_IDS[index], lockedScenario, lockedSequence, lockedCase);
        const state = active.state;
        state.startedUtc = new Date().toISOString(); state.batch = batch; state.declaredWindowMode = windowMode; records.push(state);
        const entry = fields.get(state.fieldId);
        if (!entry || (!('value' in entry.el) && !entry.el.isContentEditable)) active.stop("FIELD_OR_OBSERVER_UNAVAILABLE");
        else {
            if ('value' in entry.el) entry.el.value = ''; else entry.el.replaceChildren();
            entry.composing = false; entry.el.focus();
            if (!focused(entry.el)) active.stop("FOCUS_UNAVAILABLE");
        }
        render();
    }
    byId("start").addEventListener("click", () => {
        batch += 1; windowMode = byId("window-mode").value;
        lockedScenario = byId("scenario").value;
        lockedSequence = byId("sequence").value;
        lockedCase = byId("test-case").value;
        startField(0);
    });
    byId("next").addEventListener("click", () => startField(activeIndex + 1));
    byId("failed").addEventListener("pointerdown", event => { if (event.isTrusted && active) { active.reportFailure(); render(); } });
    byId("export").addEventListener("click", () => {
        if (active && active.state.status === "RUNNING") active.stop("EXPORT_DURING_RUN");
        render();
        const report = { schema: "lay.browser-input-dom.v3", exportedUtc: new Date().toISOString(), userAgent: navigator.userAgent, declaredBrowser,
            declaredWindowMode: byId("window-mode").value, observedBrowserFullscreen: typeof window.fullScreen === "boolean" ? window.fullScreen : null,
            fieldCount: FIELD_IDS.length, caseCatalog: caseLibrary.CASES, records, speed: summarizeTimings(records), operationSpeed: summarizeOperations(records),
            scope: "Owned DOM input cases, observed Shift taps/trigger keys and next-letter decoding. Shift and functional-operation timing are separate DOM estimates, not physical injection. Required caret/selection or trigger observations cannot be invented. Candidate visibility, runtime bytes, daemon RPC, GNOME icon, browser backend and ordinary user-site fields are not observed." };
        const url = URL.createObjectURL(new Blob([JSON.stringify(report, null, 2) + "\n"], { type: "application/json" }));
        const a = document.createElement("a"); a.href = url; a.download = "lay-" + declaredBrowser + "-double-shift-" + Date.now() + ".json"; a.click();
        setTimeout(() => URL.revokeObjectURL(url), 1000);
    });
})(typeof globalThis !== "undefined" ? globalThis : this);
