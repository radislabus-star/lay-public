/* Expected test plans only. Never convert, insert or inject input into a field. */
(function (root) {
    "use strict";
    const manual = [
        { id: "greeting", label: "Базовый Double Shift", ru: "привет", en: "ghbdtn" },
        { id: "short", label: "Одна буква", ru: "а", en: "f" },
        { id: "long", label: "Длинное слово", ru: "параллелепипед", en: "gfhfkktktgbgtl" },
        { id: "mixed-case", label: "Смешанный регистр", ru: "ПрИвЕт", en: "GhBdTn" },
        { id: "previous-word", label: "Предыдущее слово и запятая", ru: "тест, привет", en: "тест, ghbdtn", stablePrefix: "тест, " },
        { id: "english-first", label: "Начало с английской раскладки", ru: "проверка", en: "ghjdthrf", mode: "en" },
    ].map(x => ({ ...x, family: "manual" }));
    const functional = [
        { id: "space-en-ru", label: "Space: EN → RU", mode: "en" },
        { id: "space-ru-en", label: "Space: RU → EN" },
        { id: "clean-en", label: "Чистое EN сохраняется", mode: "en" },
        { id: "typo-undo", label: "Опечатка → Space → отмена" },
        { id: "moved-boundary", label: "Перенос буквы между словами" },
        { id: "completion-tab", label: "Tab → Double Shift → возврат" },
        { id: "caret-edit", label: "Курсор → Backspace → ввод" },
        { id: "selection-replace", label: "Выделение → замена" },
        { id: "focus-return", label: "Уход и возврат фокуса" },
    ].map(x => ({ ...x, family: "functional" }));
    const CASES = Object.freeze([...manual, ...functional].map(Object.freeze));
    const type = (text, mode = "ru", hint) => ({ id: "type_" + mode, text,
        hint: hint || `Выбери ${mode.toUpperCase()}. Набери «${text}» физическими клавишами.` });
    const action = (id, text, hint, code, extra = {}) => ({ id, text, hint, requiredKey: { code }, ...extra });
    const decoder = (text, mode) => ({ id: "decoder_" + mode, text: text + (mode === "ru" ? "я" : "z"),
        wrongText: text + (mode === "ru" ? "z" : "я"), hint: `Нажми Я/Z: должна добавиться ${mode === "ru" ? "русская я" : "латинская z"}.` });
    const erase = (text, mode) => ({ id: "erase_" + mode, text, hint: "Backspace один раз: удали только контрольную букву." });
    const toggle = (text, mode, id = "toggle_" + mode) => ({ id, text, toggleCount: 1,
        hint: `Дважды нажми и отпусти один Shift: должно стать «${text}».` });
    const roundtrip = (ru, en, mode = "ru") => {
        const first = mode === "ru" ? [en, "en"] : [ru, "ru"], second = mode === "ru" ? [ru, "ru"] : [en, "en"];
        return [toggle(...first), decoder(...first), erase(...first), toggle(...second), decoder(...second), erase(...second)];
    };
    const space = (text) => action("apply_space", text, `Нажми Space один раз: должно стать «${text}».`, "Space", { operation: "space", composing: false });
    function buildCase(caseId = "greeting", scenario = "active", sequence = "roundtrip") {
        const spec = CASES.find(x => x.id === caseId);
        if (!spec || !["active", "space"].includes(scenario) || !["roundtrip", "burst5"].includes(sequence)) throw new Error("Invalid case");
        const mode = spec.mode || "ru";
        let stages;
        if (spec.family === "manual") {
            const suffix = scenario === "space" ? " " : "", ru = spec.ru + suffix, en = spec.en + suffix;
            const from = mode === "ru" ? ru : en, to = mode === "ru" ? en : ru, targetMode = mode === "ru" ? "en" : "ru";
            stages = [type(from, mode), ...(sequence === "burst5" ? [
                { id: "toggle_burst5", text: to, alternateText: from, toggleCount: 5,
                    hint: "Пять Double Shift подряд: десять нажатий и отпусканий одного Shift, без других клавиш и смены фокуса." },
                { ...decoder(to, targetMode), id: "decoder_burst5" }, { ...erase(to, targetMode), id: "erase_burst5" },
            ] : roundtrip(ru, en, mode))];
        } else {
            switch (caseId) {
            case "space-en-ru": stages = [type("ghbdtn", "en"), space("привет "), decoder("привет ", "ru"), erase("привет ", "ru")]; break;
            case "space-ru-en": stages = [type("дфн"), space("lay "), decoder("lay ", "en"), erase("lay ", "en")]; break;
            case "clean-en": stages = [type("max", "en"), space("max "), decoder("max ", "en"), erase("max ", "en")]; break;
            case "moved-boundary": stages = [type("должн ыбыть"), space("должны быть "), decoder("должны быть ", "ru"), erase("должны быть ", "ru")]; break;
            case "typo-undo": stages = [type("превет"), space("привет "), toggle("превет ", "ru", "toggle_undo"), decoder("превет ", "ru"), erase("превет ", "ru")]; break;
            case "completion-tab": stages = [type("пров", "ru", "Выбери RU. Набери «пров». Перед Tab должна быть видна подсказка «проверка»; если её нет — Esc."),
                action("accept_tab", "проверка ", "Подсказка должна быть «проверка». Нет — Esc. Есть — Tab: «проверка » с одним пробелом, фокус в поле.", "Tab", { operation: "tab", composing: false, preconditionReason: "EXPECTED_COMPLETION_UNAVAILABLE" }),
                ...roundtrip("проверка ", "ghjdthrf ")]; break;
            case "caret-edit": stages = [type("привет"),
                action("caret_left", "привет", "Нажми ArrowLeft три раза: курсор после «при».", "ArrowLeft", { requiredKey: { code: "ArrowLeft", count: 3 }, caret: 3 }),
                action("erase_middle", "првет", "Backspace: удали только «и» в середине слова.", "Backspace", { caret: 2 }),
                action("insert_middle", "привет", "Нажми И/B: восстанови «привет» в середине слова.", "KeyB", { caret: 3 }),
                action("caret_end", "привет", "Нажми End: курсор в конце слова.", "End", { caret: 6 }), ...roundtrip("привет", "ghbdtn")]; break;
            case "selection-replace": stages = [type("тест привет"),
                action("select_all", "тест привет", "Ctrl+A внутри поля: выдели весь его текст.", "KeyA", { requiredKey: { code: "KeyA", ctrlKey: true }, selection: [0, 11] }),
                action("replace_selection", "а", "Нажми А/F: выделение заменяется ровно одной «а».", "KeyF", { caret: 1, operation: "selected-replacement" }),
                ...roundtrip("а", "f")]; break;
            case "focus-return": stages = [type("привет"),
                { id: "leave_focus", text: "привет", focusEvent: "focusout", focused: false, hint: "Щёлкни другое тестовое поле. Ничего в нём не вводи." },
                { id: "return_focus", text: "привет", focusEvent: "focusin", focused: true, hint: "Верни фокус в исходное поле, поставив курсор в конец слова." },
                action("after_focus_letter", "приветы", "Нажми Ы/S: должна добавиться «ы» без потери старого текста.", "KeyS"),
                action("after_focus_erase", "привет", "Backspace один раз: верни «привет».", "Backspace"), ...roundtrip("привет", "ghbdtn")]; break;
            }
            sequence = "functional"; scenario = "fixed";
        }
        return { caseId, label: spec.label, family: spec.family, initialMode: mode, stablePrefix: spec.stablePrefix || null, scenario, sequence, stages,
            expectedToggles: stages.reduce((n, x) => n + (x.toggleCount || 0), 0),
            expectedOperations: stages.filter(x => x.operation).map(x => x.operation) };
    }
    const api = { CASES, buildCase };
    if (typeof module !== "undefined" && module.exports) module.exports = api;
    root.LayBrowserCases = api;
})(typeof globalThis !== "undefined" ? globalThis : this);
