# Архитектурный чек-лист lay

Дата проверки: 2026-05-19.

Цель: держать ядро быстрым, системным и без словесного хардкода в runtime.
Примеры пользователя могут жить в тестах и fixtures, но не в production-ветках
типа `if word == ...`.

## Что проверять в коде

- Нет production-хардкода конкретных пользовательских слов.
- Runtime-правила выражены через общий признак: раскладка, словарь, n-gram,
  LEM, edit distance, тип токена, пунктуация, пробельная структура.
- Каждое правило имеет отдельный положительный тест и минимум один тест на
  ложное срабатывание.
- Правила не перекрывают друг друга без явного приоритета.
- Если одно правило нашло более сильный кандидат, соседнее правило не должно
  перетирать его более слабой догадкой.
- Автозамена после пробела всегда сохраняет пользовательский пробел как
  границу ввода.
- Double Shift остается ручной командой пользователя и не блокируется
  “умностью”, если пользователь явно хочет перевернуть текст.
- Повторный double Shift после smart-вставки или auto-undo не должен оставлять
  пустой буфер.
- Пробел не участвует в определении активной раскладки.
- `+`, `-`, CLI-флаги и технические токены не должны попадать в WordBuffer как
  обычные слова.
- Задержки не используются как архитектурное решение. Допустимы только
  низкоуровневые uinput pacing-константы в `text_output.rs`.
- GNOME/KDE/Niri/X11 различия изолированы в backend/runtime слоях, а не протекают в
  scoring/typing ядро.

## Чек-лист рефакторинга

- `lay_daemon.rs` остается оркестратором, без правил коррекции.
- `daemon_runtime.rs` отвечает только за evdev event loop и делегирует решения.
- `keyboard_io.rs` отвечает только за `/dev/input` discovery, фильтр
  виртуальных клавиатур и низкоуровневые helpers вокруг evdev/uinput mutex.
- `trigger_dispatch.rs` отвечает только за выбор trigger-клавиши и запуск
  manual correction по configured/scope-trigger, без чтения evdev loop.
- `boundary_runtime.rs` отвечает только за Space/Enter/hard-boundary runtime:
  запуск after-space assist, сохранение пробельной границы, Enter-autocorrect
  и сброс буфера на границах.
- `typing_key_runtime.rs` отвечает только за обычный typing-key press:
  обновить layout cache, принять/отфильтровать autorepeat и записать KeyEvent
  в WordBuffer.
- `correction_runtime.rs` отвечает только за execution manual correction:
  delete, insert, replay, undo, layout.
- `typing_assist_runtime.rs` отвечает только за execution typing-assist после
  Space/Enter, но не за словарную логику.
- `layout_controller.rs` отвечает за общий layout facade, GNOME/DBus/IME
  bridge и общие shell helpers.
- `layout_kde.rs` отвечает только за KDE/qdbus layout backend.
- `layout_niri.rs` отвечает только за Niri/niri-ipc layout backend.
- `layout_x11.rs` отвечает только за X11 XKB/shell-tool layout backend.
- `word_buffer.rs` отвечает за память хвоста, replay toggle, pending learning и
  pending undo, но не решает, что является хорошим словом.
- `word_reader.rs` отвечает за примитивное чтение токенов и сегментов.
- `phrase_reader.rs` отвечает за чтение фразы и склейки/разрезы слов.
- `ru_typo.rs` отвечает только за word-level русские опечатки.
- `russian_typo_candidates.rs` отвечает только за генерацию кандидатов
  русских опечаток.
- `russian_typo_scoring.rs` отвечает только за словарный/n-gram выбор
  кандидата русской опечатки.
- `russian_lexicon.rs` отвечает только за словари и распознавание форм.
- `token_language.rs` отвечает за RU/EN известность токенов.
- `mixed_script_repair.rs` отвечает за детерминированный ремонт смешанного
  Cyrillic/ASCII внутри токена.
- `layout_autoswitch.rs` отвечает только за auto-switch раскладки после пробела.
- `typing_pipeline.rs` отвечает за порядок правил и сбор кандидатов.
- `typing_rule_graph.rs` отвечает за реестр rule id, family и apply-функций.
- `typing_candidate.rs` отвечает за scoring и выбор кандидата.
- `decoder.rs` отвечает за перевод решения в действие.
- `text_edit.rs` отвечает за минимальные edit plans и пробельные границы.
- `scoped_tail.rs` отвечает за manual scoped-tail smart решение.
- Дублирующиеся списки лексических признаков должны быть вынесены в один
  shared data/module.

## Чек-лист лучших практик

- Делать изменение через failing regression test.
- Фиксировать класс ошибки, а не отдельное слово.
- Для спорной автозамены предпочитать `None`, а не агрессивную догадку.
- Для ручного double Shift предпочитать действие пользователя, а не защитное
  “не трогать”.
- Для 2/3/N слов использовать candidate ranking, а не каскад локальных if.
- Scoring должен быть сравнимым: dictionary confidence, LEM, n-gram margin,
  edit penalty, intervention penalty.
- Все пороги держать рядом с соответствующим scorer, а не раскидывать по daemon.
- Тестовые fixtures можно расширять реальными ошибками, но runtime не должен
  знать, откуда пришел пример.
- Большие тестовые матрицы нужны как safety net, но каждая новая ошибка должна
  иметь маленький уникальный regression test.
- Runtime smoke в окне обязателен для багов вывода, пробела, backspace,
  double Shift и auto-undo.

## Текущее дерево ответственности

- `src/bin/lay_daemon.rs` - CLI/daemon bootstrap, 418 строк.
- `src/bin/lay_daemon/daemon_runtime.rs` - главный evdev loop, 730 строк.
- `src/bin/lay_daemon/trigger_dispatch.rs` - trigger config и ручной
  correction dispatch, 79 строк.
- `src/bin/lay_daemon/boundary_runtime.rs` - Space/Enter/hard-boundary
  runtime, 206 строк.
- `src/bin/lay_daemon/typing_key_runtime.rs` - обычные typing-key события,
  67 строк.
- `src/bin/lay_daemon/keyboard_io.rs` - `/dev/input` discovery и evdev/uinput
  helpers, 64 строки.
- `src/bin/lay_daemon/correction_runtime.rs` - manual correction execution,
  682 строки.
- `src/bin/lay_daemon/typing_assist_runtime.rs` - typing assist execution,
  685 строк.
- `src/bin/lay_daemon/layout_controller.rs` - общий layout facade,
  GNOME/DBus/IME bridge и helpers, 596 строк.
- `src/bin/lay_daemon/layout_kde.rs` - KDE/qdbus layout backend, 132 строки.
- `src/bin/lay_daemon/layout_x11.rs` - X11 layout backend, 50 строк.
- `src/bin/lay_daemon/text_output.rs` - uinput/DBus text output, 331 строка.
- `src/llm.rs` - optional candidate arbiter facade, 341 строк.
- `src/mixed_script_repair.rs` - mixed Cyrillic/ASCII token repair,
  189 строк.
- `src/token_language.rs` - token-level RU/EN recognition, 146 строк.
- `src/phrase_reader.rs` - phrase/glued/split reader, 481 строк.
- `src/ru_typo.rs` - Russian word typo rules, 524 строки.
- `src/russian_typo_candidates.rs` - Russian typo candidate generation,
  179 строк.
- `src/russian_typo_scoring.rs` - Russian typo candidate scoring, 86 строк.
- `src/russian_lexicon.rs` - Russian dictionary/form recognition, 439 строк.
- `src/layout_autoswitch.rs` - layout auto-switch after Space, 414 строк.
- `src/word_buffer.rs` - tail memory, replay toggle, learning/undo state,
  606 строк.
- `src/typing_pipeline.rs` - typing-assist rule order and explain, 163 строки.
- `src/typing_rule_graph.rs` - typing-assist rule graph registry, 254 строки.
- `src/typing_candidate.rs` - candidate scoring, 195 строк.
- `src/decoder.rs` - action decision facade, 416 строк.
- `src/text_edit.rs` - minimal replacement plans, 256 строк.
- `src/word_reader.rs` - token/segment primitives, 160 строк.

## Что уже исправлено этой ревизией

- Space-release теперь реально запускает typing-assist после пробела, поэтому
  live case `ашду ` доходит до `file `.
- Missing-letter и extra-letter используют согласованную проверку кандидатов:
  extra-letter не перетирает более сильный missing-letter.
- Missing-letter больше не дописывает произвольную букву в конец гласного
  окончания, например `перекупа` не превращается в другую форму.
- Производные приставки в словаре распознаются консервативнее: больше нет
  правила “приставка + любая сгенерированная форма”.
- Auto-undo больше не обнуляет буфер: после отката следующий double Shift снова
  проходит smart decision layer.
- `к-лист` и похожие естественные кириллические дефисные токены не улетают в
  ASCII-layout мусор, если внутри есть известный кириллический фрагмент.
- `lay_daemon.rs` разрезан до bootstrap/оркестратора; runtime вынесен в
  `src/bin/lay_daemon/*`.
- Trigger config и общий запуск manual correction вынесены в
  `src/bin/lay_daemon/trigger_dispatch.rs`.
- Space/Enter/hard-boundary runtime вынесен в
  `src/bin/lay_daemon/boundary_runtime.rs`.
- Обычная запись typing-key events вынесена в
  `src/bin/lay_daemon/typing_key_runtime.rs`.
- KDE/qdbus backend вынесен в `src/bin/lay_daemon/layout_kde.rs`.
- X11 backend вынесен в `src/bin/lay_daemon/layout_x11.rs`.
- `typing_assist.rs` стал совместимым facade; rule order живет в
  `typing_pipeline.rs`, smart-tail в `scoped_tail.rs`.
- Генерация и scoring русских typo-кандидатов вынесены из `ru_typo.rs`.
- Словарная RU/EN оценка токенов вынесена из `llm.rs` в `token_language.rs`.
- Ремонт mixed-script токенов вынесен из `llm.rs` в
  `mixed_script_repair.rs`.
- Введён `typing_rule_graph.rs`: `typing_pipeline.rs` больше не содержит
  большой `match` по всем правилам.
- Добавлен explain-контур автозамены: CLI `lay --explain-correct 'текст '`
  показывает все правила, кандидатов, отказы, победителя, второго кандидата,
  margin уверенности и итоговый output.
- `typing_candidate.rs` теперь отдаёт единый `TypingCandidateDecision`:
  лучший кандидат, второй кандидат, margin и confidence. UI/CLI/runtime должны
  читать уверенность из этого контракта, а не пересчитывать её локально.
- `decoder.rs` теперь проверяет edit-plan как инвариант:
  `original + plan == replacement`, а committed-коррекции обязаны сохранять
  пробельный разделитель после Space/Enter.

## Что осталось резать дальше

- `phrase_reader.rs` разделить на:
  `phrase_split.rs`, `glued_phrase.rs`, `moved_prefix.rs`.
- `ru_typo.rs` при дальнейшем росте разделить на:
  `missing_letter.rs`, `extra_letter.rs`, `transpose.rs`, `substitution.rs`.
- `daemon_runtime.rs` дальше разделить по обработчикам:
  force-layout hotkeys, single trigger, caps trigger, double/multi-tap trigger.
- Следующий runtime-разрез: вынести modifier/force-layout hotkey handling, не
  меняя порядок evdev loop.
- `layout_controller.rs` дальше отделить GNOME/DBus/IME детали от общего
  layout facade.
- `lay_test_input.rs` разрезать на scenario registry и низкоуровневый uinput
  driver.
- Расширить RuleGraph до общего `CorrectionGraph` для manual scoped-tail,
  когда очередь дойдёт до унификации double Shift и after-space.
- Добавить архитектурный guard против `if word ==` в production-модулях.
- Добавить отдельный smoke-набор для KDE/X11, когда есть стабильная сессия.

## Проверочный контур

- `scripts/check-lay-full.sh`
- `scripts/check-lay-audit-50.sh` — 50-pass архитектурный audit без полного
  release build.
- `LAY_AUDIT_50=1 scripts/check-lay-full.sh` — строгий полный gate: 50-pass
  audit + обычный full check.
- `scripts/run_runtime_smoke.py --no-build` для окна ввода.
- `cargo test --all-targets`
- `cargo clippy --all-targets -- -D warnings`
- `cargo build --release --bins`
- `cargo run --quiet --bin lay-ngram-corpus -- check-cache`
- `cargo run --quiet --bin lay-lem-research`

`scripts/check-lay-full.sh` проверяет существующий пользовательский n-gram
cache, а если cache ещё не создан на чистой машине, собирает временный cache в
`target/` и проверяет его. CI делает такой же n-gram cache probe явно.
