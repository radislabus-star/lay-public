/* Test observer only: no layout conversion, key injection or IME API calls. */
(function (root) {
    "use strict";
    const FIELD_IDS = ["text", "search", "url", "tel", "email", "textarea", "rich", "plain", "iframe-input", "iframe-rich"];
    const TERMINAL = new Set(["PASS_DOM", "FAIL_REPORTED", "FAIL_VISIBLE", "BLOCKED"]);

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
            samples: n, missing: records.reduce((sum, r) => sum + (r.sequence === "burst5" ? 5 : 2), 0) - n,
            medianMs: n ? (n % 2 ? values[middle] : (values[middle - 1] + values[middle]) / 2) : null,
            p95Ms: n ? values[Math.ceil(n * .95) - 1] : null, maxMs: n ? values[n - 1] : null };
    }
    function createRun(fieldId, scenario = "active", sequence = "roundtrip") {
        if (!FIELD_IDS.includes(fieldId) || !["active", "space"].includes(scenario) || !["roundtrip", "burst5"].includes(sequence)) throw new Error("Invalid case");
        const suffix = scenario === "space" ? " " : "";
        const ru = "привет" + suffix, en = "ghbdtn" + suffix;
        const stages = sequence === "burst5" ? [
            { id: "type_ru", text: ru, hint: "Выбери RU. Набери «привет»" + (suffix ? " и один пробел." : " без пробела.") },
            { id: "toggle_burst5", text: en, hint: "Пять Double Shift подряд: десять нажатий и отпусканий одного Shift, без других клавиш и смены фокуса." },
            { id: "decoder_burst5", text: en + "z", hint: "После пятого переключения нажми Я/Z: должна добавиться латинская z." },
            { id: "erase_burst5", text: en, hint: "Нажми Backspace один раз: удали только z." },
        ] : [
            { id: "type_ru", text: ru, hint: "Выбери RU. Набери «привет»" + (suffix ? " и один пробел." : " без пробела.") },
            { id: "toggle_en", text: en, hint: "Дважды нажми и отпусти один Shift: должно стать «" + en + "»." },
            { id: "decoder_en", text: en + "z", hint: "Нажми клавишу Я/Z: должна добавиться латинская z." },
            { id: "erase_en", text: en, hint: "Нажми Backspace один раз: удали только z." },
            { id: "toggle_ru", text: ru, hint: "Снова Double Shift: должно вернуться «" + ru + "»." },
            { id: "decoder_ru", text: ru + "я", hint: "Нажми клавишу Я/Z: должна добавиться русская я." },
            { id: "erase_ru", text: ru, hint: "Нажми Backspace один раз: удали только я." },
        ];
        const state = { fieldId, scenario, sequence, status: "RUNNING", step: 0, stageCount: stages.length, milestones: [], observations: [], timings: [], burstTargets: [], reason: null };
        let pressed = new Set(), taps = 0, shiftSide = null, decoderKey = false, backspaceKey = false;
        let pairStart = null, secondDownAt = null, firstTargetAt = null, pairs = [];
        const phase = () => stages[state.step] || null;
        function stop(reason, status = "BLOCKED") { if (!TERMINAL.has(state.status)) { state.status = status; state.reason = reason; } }
        function observe(snapshot, advanceMilestone = true) {
            if (TERMINAL.has(state.status) || snapshot.fieldId !== fieldId || !snapshot.trusted) return;
            if (state.observations.length >= (sequence === "burst5" ? 512 : 128)) { stop("OBSERVER_LIMIT"); return; }
            state.observations.push({ ...snapshot, step: phase().id });
            if (!snapshot.focused) { stop("FOCUS_CHANGED"); return; }
            if (typeof snapshot.text !== "string" || snapshot.text.length > 256) { stop("UNSUPPORTED_OBSERVATION"); return; }
            if (/Paste|Drop/.test(snapshot.inputType || "")) { stop("PASTE_OR_DROP"); return; }
            if (!advanceMilestone) return;
            const stage = phase();
            if (stage.id.startsWith("decoder_") && decoderKey && snapshot.text === (stage.id === "decoder_ru" ? ru + "z" : en + "я")) {
                stop("NEXT_LETTER_WRONG_LAYOUT", "FAIL_VISIBLE"); return;
            }
            if (stage.id === "toggle_burst5") {
                const expected = state.burstTargets.length % 2 === 0 ? en : ru;
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
            state.milestones.push({ stage: stage.id, ...snapshot, shiftTaps: stage.id.startsWith("toggle_") ? taps : null });
            state.step += 1;
            pressed = new Set(); taps = 0; shiftSide = null; decoderKey = false; backspaceKey = false;
            pairStart = null; secondDownAt = null; firstTargetAt = null; pairs = [];
            if (state.step === stages.length) state.status = "PASS_DOM";
        }
        function key(event, snapshot) {
            if (TERMINAL.has(state.status) || event.fieldId !== fieldId || snapshot.fieldId !== fieldId || !event.trusted || !snapshot.trusted) return;
            const observed = { ...snapshot, code: event.code, repeat: Boolean(event.repeat), ctrlKey: Boolean(event.ctrlKey), altKey: Boolean(event.altKey), metaKey: Boolean(event.metaKey) };
            if (event.type === "keydown" && event.code === "Escape" && !event.repeat && !event.ctrlKey && !event.altKey && !event.metaKey) {
                observe(observed, false);
                if (state.status === "RUNNING") stop("USER_REPORTED_FAILURE", "FAIL_REPORTED");
                return;
            }
            const stage = phase();
            if (stage.id === "toggle_burst5" && event.type === "keydown" && (event.ctrlKey || event.altKey || event.metaKey || !/^Shift(Left|Right)$/.test(event.code))) {
                stop("EXTRANEOUS_KEY_DURING_BURST"); return;
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
        return { state, observe, key, stop, phase };
    }

    const api = { FIELD_IDS, createRun, summarizeTimings };
    if (typeof module !== "undefined" && module.exports) module.exports = api;
    root.LayFirefoxProbe = api;
    if (typeof document === "undefined") return;
    const declaredBrowser = new URLSearchParams(location.search).get("browser") === "chrome" ? "chrome" : "firefox";
    const browserLabel = declaredBrowser === "chrome" ? "Chrome" : "Firefox";
    document.title = "Lay — Double Shift в 10 полях " + browserLabel;
    document.querySelector("h1").textContent = "Double Shift: 10 типов полей " + browserLabel;
    const fields = new Map(), records = [], cards = [...document.querySelectorAll(".card")];
    const byId = id => document.getElementById(id);
    let active = null, activeIndex = -1, lockedScenario = null, lockedSequence = null, batch = 0, windowMode = null;
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
            card.querySelector("output").textContent = !latest ? "Не проверено" : ({ RUNNING: "Шаг " + (latest.step + 1) + "/" + latest.stageCount, PASS_DOM: latest.sequence === "burst5" ? "Пять переключений ✓" : "Два переключения ✓", FAIL_REPORTED: "Сбой", FAIL_VISIBLE: "Сбой", BLOCKED: "Прервано" })[latest.status];
        }
        const current = FIELD_IDS.map(id => [...records].reverse().find(r => r.fieldId === id && r.batch === batch));
        const count = status => current.filter(r => r && r.status === status).length;
        byId("summary").textContent = `Пройдено: ${count("PASS_DOM")} · Сбой: ${count("FAIL_REPORTED") + count("FAIL_VISIBLE")} · Прервано: ${count("BLOCKED")} · Не проверено: ${current.filter(r => !r || r.status === "RUNNING").length}`;
        const speed = summarizeTimings(current.filter(Boolean));
        const ms = value => value === null ? "—" : Math.round(value) + " мс";
        byId("speed").textContent = `Скорость по событиям браузера: медиана ${ms(speed.medianMs)} · p95 ${ms(speed.p95Ms)} · максимум ${ms(speed.maxMs)} · замеров ${speed.samples} · без замера ${speed.missing}`;
        byId("failed").disabled = !active || active.state.status !== "RUNNING";
        byId("next").disabled = !active || active.state.status === "RUNNING" || activeIndex === FIELD_IDS.length - 1;
        if (!active) return;
        byId("instruction").textContent = active.state.status === "RUNNING" ? `${activeIndex + 1}/10. ${active.phase().hint}` : active.state.status === "PASS_DOM" ? "Переключения и следующая буква совпали. Нажми «Следующее поле»." : `Проверка остановлена: ${active.state.reason}. Результат сохранён; можно перейти дальше.`;
    }
    function attach(id, el) {
        if (fields.has(id)) return;
        fields.set(id, { el, composing: false });
        for (const type of ["input", "compositionstart", "compositionupdate", "compositionend", "keydown", "keyup"]) {
            el.addEventListener(type, event => {
                if (type === "compositionstart") fields.get(id).composing = true;
                if (type === "compositionend") fields.get(id).composing = false;
                if (!active || active.state.fieldId !== id) return;
                const observed = snapshot(id, el, event);
                if (type === "keydown" || type === "keyup") active.key({ fieldId: id, trusted: event.isTrusted, type, code: event.code,
                    repeat: event.repeat, ctrlKey: event.ctrlKey, altKey: event.altKey, metaKey: event.metaKey }, observed);
                else active.observe(observed);
                if (event.isTrusted && type === "keydown" && event.code === "Escape" && active.state.reason === "USER_REPORTED_FAILURE") event.preventDefault();
                render();
            });
        }
        for (const type of ["paste", "drop"]) el.addEventListener(type, () => { if (active && active.state.fieldId === id) { active.stop("PASTE_OR_DROP"); render(); } });
        el.addEventListener("focusout", () => {
            if (active && active.state.fieldId === id && active.state.status === "RUNNING") { active.stop("FOCUS_CHANGED"); render(); }
        });
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
        active = createRun(FIELD_IDS[index], lockedScenario, lockedSequence);
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
        startField(0);
    });
    byId("next").addEventListener("click", () => startField(activeIndex + 1));
    byId("failed").addEventListener("pointerdown", event => { if (event.isTrusted && active) { active.stop("USER_REPORTED_FAILURE", "FAIL_REPORTED"); render(); } });
    byId("export").addEventListener("click", () => {
        if (active && active.state.status === "RUNNING") active.stop("EXPORT_DURING_RUN");
        render();
        const report = { schema: "lay.browser-double-shift-dom.v2", exportedUtc: new Date().toISOString(), userAgent: navigator.userAgent, declaredBrowser,
            declaredWindowMode: byId("window-mode").value, observedBrowserFullscreen: typeof window.fullScreen === "boolean" ? window.fullScreen : null,
            fieldCount: FIELD_IDS.length, records, speed: summarizeTimings(records), scope: "Owned DOM text transitions, observed Shift taps and next-letter decoding. Timing uses DOM second Shift keydown to exact target, not physical injection. Composition/caret observations are separate; missing caret/timing is UNKNOWN. Runtime bytes, daemon RPC, GNOME icon, browser backend and ordinary user-site fields are not observed." };
        const url = URL.createObjectURL(new Blob([JSON.stringify(report, null, 2) + "\n"], { type: "application/json" }));
        const a = document.createElement("a"); a.href = url; a.download = "lay-" + declaredBrowser + "-double-shift-" + Date.now() + ".json"; a.click();
        setTimeout(() => URL.revokeObjectURL(url), 1000);
    });
})(typeof globalThis !== "undefined" ? globalThis : this);
