# Rust Architecture Rules

Эти правила нужны, чтобы `lay` не возвращался к россыпи прямых вызовов между
daemon, IME, CLI, NANDA и выводом текста.

## 1. Один вход в решение

Решение "заменять текст или оставить как есть" должно идти через
`correction_core`.

Runtime-код не должен напрямую выбирать между deterministic pipeline и NANDA.
Он передаёт `CorrectionRequest` и получает `CorrectionDecision`.

## 2. Вывод не смешивать с решением

`correction_core` не удаляет текст, не печатает символы, не двигает курсор и не
переключает раскладку.

Выводом владеют отдельные backend-слои:

- daemon: `KeyEvent` / `DecoderEditPlan` / uinput или text backend;
- IME: `CommitText`, `DeleteSurroundingText`, preedit;
- CLI: stdout / clipboard.

## 3. Состояние не переносить между backend

`WordBuffer` daemon и `tail_buffer` IME похожи по смыслу, но это разные
состояния с разными гарантиями. Общим может быть только чистое решение, не
runtime-буфер.

## 4. Preedit только предлагает

Preedit/precognition может показывать кандидата, но не должен применять замену.
Применение проходит через commit/tail path и общий decision facade.

## 5. NANDA не вызывать россыпью

Новые runtime-вызовы `run_wave_trace()` добавлять только внутри
`correction_core` или специализированного eval/debug-инструмента.

Исключения допустимы для:

- offline eval;
- trace/status;
- research tools.

## 6. Новый runtime-путь требует smoke case

Если меняется IME/daemon ввод, добавлять минимальный live-smoke сценарий в
`data/test_input/` и регистрировать его в `scripts/runtime_smoke/cases.py`.

## 7. Рефакторинг идёт слоями

Сначала выделить facade/тип контракта, потом перевести один caller, затем
проверить тесты. Не переносить одновременно decision, output, buffer sync и UI.
