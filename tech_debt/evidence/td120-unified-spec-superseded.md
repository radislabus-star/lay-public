# TD-120: Ограничить запрет автозамены временем жизни конкретного слова

Status: `ANALYSIS_REQUIRED`
Priority: `P1`
Stage: 1 — минимальное функциональное исправление
Size: S/M
Base commit: `cc1e2207519801ca0f9b7c6963897b55953a7751`
Admission: 2026-09-05, пользователь разрешил только фазу 1 — диагностику и
проектирование. Production-реализация, установка и push этой задачей пока
не выполнены и не разрешаются статусом спецификации.
Continuation: последующее «дальше без остановок пожалуйста» допускает TDD и
минимальную реализацию после записанного design admission; review документа
по-прежнему не закрывает G120-1/2/3 автоматически.
Implementation model: `GPT-5.6 Sol / high`, после отдельного перехода к фазе 2.

## Результат для пользователя

Ручной выбор сохраняется для текущего слова. Если пользователь удалил это
слово, закончил его или перешёл в другой контекст ввода, защита не должна
подавлять исправление следующего слова. При этом нельзя повторно исправлять
синтетический ввод, которым демон выполняет разрешённый ручной переворот.

Задача не обещает исправить потерю первой буквы при смене экземпляра IME:
это отдельная [TD-121](121-preserve-word-across-ime-layout-handoff.md).

Причина подтверждена, но выбранная unified migration ещё **не допущена** к
production-коду. Независимо можно подготовить failing lifecycle tests после
разрешения фазы 2. Нельзя считать всю реализацию независимой от TD-121.

## Подтверждённая причина

Сохранённый production-журнал 1.0.65:

1. `ibus_manual_toggle_plan`, строка 1534: переключено предыдущее `вот → djn`.
2. Строки 1552–1704: 54 Backspace, из них 45 уже с `tail_chars=0`.
3. Строки 1706–1764: новое слово `gjxbnfq`, длина хвоста 7.
4. Строка 1766: `manual_toggle_suppressed`; Space оставляет новое слово.

Причина в исходниках, а не в настройке «Осторожности»:

- `state.rs::replace_committed_tail` ставит обычный local/shared boolean,
  оставляя `exact_manual_toggle_suppression=None`.
- `composition_edit.rs::backspace_committed_tail_only` удаляет символ и
  сбрасывает режим слова при пустом токене, но не отменяет этот запрет.
- `committed_tail.rs::take_manual_toggle_autocorrect_suppression` принимает
  `None` как отсутствие ограничений identity/expiry и объединяет local/shared
  флаги через OR.
- `managed.rs` расходует запрет до обычного пути Space-коррекции.

Первый установленный разрыв authority: готовность исправления не проверяется,
потому что до обращения к подготовленному решению срабатывает чужой запрет.
Не утверждать, что после устранения запрета именно `почитай` обязательно
выиграет весь correction pipeline: это отдельная проверка с настоящим frame.
`candidates:0` в correction timing сейчас literal telemetry и не является
числом кандидатов (`trace.rs::record_correction_projection_timing`).

Снимки, SHA-256 и ограничения вывода:
[общий аудит](AUDIT_2026-09-05_LAYOUT.md).

Phase2: [installed-byte baseline](evidence/td120-121-installed-baseline-phase2.md)
также воспроизвела ManualToggleV3 → полное удаление → новый токен →
`manual_toggle_suppressed`. Качество последующей коррекции не проверялось.

## Варианты и выбор

Оценки — инженерная применимость, не измерение качества модели или кода.

| Вариант | Оценка | Решение |
|---|---:|---|
| Сбрасывать оба boolean на каждом Backspace | 3/10 | Нарушает защиту продолжаемого слова и delete/replay; отклонён |
| Сбрасывать обычный запрет на завершении/полном удалении слова, сохранив текущие local/shared флаги | 7/10 | Жизнеспособная узкая правка, но остаются два расходуемых состояния и риск старого engine callback |
| Один scoped suppression record у существующего владельца; различать защиту слова и защиту replay | **9/10 при закрытии admission gates** | Рекомендуемый дизайн: точный жизненный цикл и один расходуемый запрет; V1, atomic и cross-engine совместимость пока не доказаны |
| Переписать весь ввод на новый общий event bus / глобальную FSM | 4/10 | Для этой причины не требуется; отдельный Stage 2 |

## Контракт выбранной реализации

### 0. Допуск и зависимость — до замены общего owner

| Gate | Что требуется | Текущее состояние |
|---|---|---|
| G120-1 | Полная совместимость V1 всех существующих producers, включая abort и late after; нельзя распознавать фазу по чётности вызовов либо тексту | OPEN: V1 не передаёт request/token/phase |
| G120-2 | Безопасный settlement single shared guard при atomic interleaving либо доказанная сериализация всех producers | OPEN: текущий commit переписывает SharedState целиком |
| G120-3 | Принятый и проверенный canonical same-context contract TD-121 для обычного cross-engine переноса CurrentWord | OPEN: baseline handoff принимает TTL, а не canonical identity |

Все три gate блокируют **всю выбранную миграцию общего suppression owner**.
Нельзя удалить booleans у части producers, оставив второй расходуемый owner,
или получить READY, объявив cross-engine/V1/atomic проверки необязательными.
Историческую доставку `l` не требуется заново доказывать в TD-120, но принятый
контракт идентичности TD-121 является явной зависимостью выбранного дизайна.

Самостоятельный core для TDD — слово при доказанном текущем владельце,
его полное удаление, границы и regression parity уже разрешённых replay routes.
Это разрешает подготовку тестов, **не отдельную production-миграцию**. Если
gates требуют несоразмерного расширения, вернуться к варианту 7/10 и письменно
допустить отдельный узкий lifecycle design с собственными последствиями и
доказательствами. Такой дизайн сейчас не выбран и не реализован автоматически.

Не вводить новый процесс, worker, очередь, дисковое состояние или таймер
для обычного слова. Использовать существующий `SharedState` и существующую
систему runtime identities. Имена типов ниже описательные, не требование
создать дополнительные публичные API.

### 1. Два разных назначения, одно место расходования

- `CurrentWord`: защита **успешно** выполненного IME manual edit / принятого
  кандидата / undo. Привязана к lineage текущего слова и владельцу контекста.
- `ExactReplay`: текущая ограниченная transport-защита daemon delete/replay;
  сохранить проверку source/target path, epoch, expiry, exact cancellation.
  Пустой промежуточный хвост здесь не равен новому пользовательскому слову.

Не оставлять одновременно старые booleans и новую структуру как независимые
источники решения. Speculative-копия не пишет в live SharedState до commit;
этого недостаточно для безопасного settlement. `atomic.rs:196–205` заменяет
live SharedState целиком, без сравнения поколения suppression/owner. Сначала
закрыть G120-2: доказать, что interleaving невозможен для всех producers, либо
явно выбрать ограниченную generation validation / settlement правку. Широкая
перестройка atomic owner не входит в эту задачу без нового решения.

### 2. Что считать тем же словом

- Один lineage продолжается при вводе/удалении части ещё непустого токена.
- Полное пользовательское удаление текущего токена завершает lineage; повтор
  той же строки после пустого состояния — **новый** токен.
- Пробел и существующие реальные token boundaries закрывают lineage.
- Enter, смена реального input context, sensitive content и потеря text lease
  закрывают lineage. Soft Reset и подтверждённый same-context handoff сами по
  себе слово не завершают.
- Использовать существующие правила ASCII layout punctuation: `;`, `[`, `]`,
  апостроф и другие клавиши русских букв нельзя безусловно считать границей.
- Нельзя использовать текущий `tail_epoch` как постоянный word id: он
  меняется на каждом publish. Если нужен word-generation, обновлять только на
  перечисленных переходах и переносить вместе с соответствующим хвостом.
- Совпадения строки, TTL, app name или координат недостаточно для переноса
  запрета в другой контекст. Старый engine не расходует и не очищает guard
  нового владельца.

### 3. Когда устанавливать и снимать защиту

- Устанавливать `CurrentWord` после подтверждённого успешного edit и обновления
  хвоста. Reject, failed output и duplicate/no-op не создают новую защиту.
- Если принятая/заменённая строка уже содержит завершающую границу, её защита
  не должна попадать на следующее слово. Учесть auto-undo с сохранённым пробелом.
- Space проверяет и расходует один соответствующий guard ровно один раз.
  Второй Space и другая lineage не подавляются старым событием.
- Сначала зафиксировать feedback соответствующего текущего слова, затем
  завершать lifecycle. Не превращать suppress/censored в отрицательную оценку
  чужой подсказки или в принятие correction.

### 4. Все существующие производители обязательны для аудита

| Производитель | Что сохранить / проверить |
|---|---|
| `shift.rs::manual_toggle_active_text_target` | ActiveComposition и terminal committed-tail различаются, один физический detector |
| `state.rs::ime_manual_toggle / ime_auto_undo / ime_candidate_accept` | Успех, ошибка, trailing boundary, no-op; не подавлять следующее слово |
| `bridge_actions.rs::suppress_next_autocorrect_v2_inner` | Exact replay переживает промежуточное удаление, сохраняет expiry/revoke |
| `bridge_actions.rs::suppress_next_autocorrect_inner` | Legacy V1 вызывается и до, и после uinput replay; нельзя автоматически трактовать оба вызова как готовое новое слово |
| `bridge_actions.rs::ReplaceTail*`, `bridge_policy.rs` | Сохранить назначения существующих typed intents, не принимать строковый kind за доказательство нового права |
| `shift.rs::defer_committed_tail_manual_toggle_to_daemon` | Делегирование ещё не доказывает успешное изменение текста |

V1 transport нельзя молча перевести в CurrentWord: `correction_runtime/output.rs`
вызывает его до ветвления text-replace/replay, а `output/replay.rs` ещё раз
после успешного replay. При ошибке второго вызова может не быть. No-arg RPC
не доказывает фазу, пару вызовов, слово или request. Раскрыть S12 до patch.
Если безопасную совместимость нельзя сохранить узко, **вся unified migration
остаётся analysis-only**; предложить отдельную явную миграцию API либо заново
допустить узкий lifecycle design. Не продолжать с «пока старым V1», новым
параллельным boolean, угадыванием фазы или ослаблением exact V2.

## Файлы — предполагаемая ограниченная область

- `src/bin/lay_ibus_engine/protocol/state.rs`: общий suppression scope.
- `engine/state_groups.rs`, `engine.rs`: local token lifecycle / handoff.
- `tail_memory.rs`, `composition_edit.rs`, `preedit.rs`: закрытие и перенос.
- `atomic.rs`: сначала доказательство settlement совместимости; production
  изменение только после G120-2, не новая общая транзакционная подсистема.
- `committed_tail.rs`, `state.rs`, `shift.rs`, `bridge_actions.rs`: producers
  и одно расходование guard; существующий публичный протокол по возможности
  неизменен.
- Тесты рядом с владельцами и fixture-файл с поверхностями; не выделять
  самостоятельный framework или god-manager ради этой задачи.

Не менять в TD-120 candidate generation, ranking, L1.1/L2/L3/L4, словари,
SafetyGate, edit-plan validation, дедлайн Space, GNOME switching или learner.

## Consequence analysis до production-кода

| Измерение | Ожидание, риск и проверка |
|---|---|
| Lattice retention / ranking | Ничего не добавляется/не удаляется; только чужой pre-decision запрет перестаёт действовать. Не гарантировать результат отдельного слова без full-frame проверки |
| False authority / safety | Снятие чужого guard возвращает обычную проверку DecisionCore/verifier; не выдаёт Apply напрямую. Чужой scope не должен расходовать guard текущего поля |
| Latency / tail | Одна bounded проверка identity; без сна, RPC, word lookup, ожидания модели и изменения 3.5ms Space wait |
| CPU / RSS / allocations | O(1) scope вместо двух boolean owners; ограниченные identities, без per-key клонирования sentence и новых threads. Сравнить allocations/latency с baseline |
| Cache / invalidation | Word id отличается от tail_epoch. Не возвращать устаревшие prefetch/candidate certificates после handoff/undo |
| Package / delta reload | Guard не хранит lexicon/package pointer и не зависит от обновления словаря. Старые correction leases по-прежнему инвалидируются generation |
| Learning / feedback | Guard относится к тому же слову; censored не превращается в wrong-target rejection. Не трогать L3 admission или persisted schema |
| Concurrency | Lock защищает одну операцию, но не интервал prepare→commit. Late callback/old commit не снимает новый guard и не воскрешает consumed/revoked guard; G120-2 требует отдельного доказательства |
| Failure / rollback | Failed output не устанавливает CurrentWord. Exact replay failure и cancel не изменяют другой request. Откат — обычный отдельный commit, без reset пользовательских словарей |
| Consumers | ManagedCommit, active composition, TerminalPassthrough, daemon V1/V2, manual/Tab/undo проверяются раздельно |
| Maintenance | Удалить заменённые обычные boolean ветки, не поддерживать две параллельные системы. Не извлекать модуль только ради лимита строк |

## TDD: фиксированная матрица перед первым исправлением

Сначала воспроизвести старое неправильное поведение настоящими методами
engine, затем поменять ожидаемый исход. Не заменять semantic assertion поиском
подстроки в Rust исходниках. Новый suite фиксируется до patch, случаи не
выкидываются после неудачи.

| ID | Сценарий | Обязательный исход после исправления |
|---|---|---|
| S01 | Manual edit → Space → Space | То же слово защищено один раз; второй Space без старого guard |
| S02 | Manual edit → стереть токен полностью → новое слово → Space | Старый guard отсутствует; обычный correction route вызывается |
| S03 | То же с сохранённым левым контекстом | Проверять пустоту токена, не всего sentence buffer; левый текст неизменен |
| S04 | Удалить часть / допечатать ещё непустое слово | Сохранить согласованную защиту того же lineage |
| S05 | Полностью удалить и снова набрать идентичную строку | Новый lineage, старый guard не воскресает |
| S06 | Новый engine / same-context | S06a: characterization baseline admitted exact replay, не доказательство canonical обычного переноса. S06b: обычный CurrentWord переносится один раз только после G120-3; до этого NOT_RUN/GATED |
| S07 | Новое поле / same app / другой context | S07a: существующие invalidation при явной потере текущего lease. S07b: cross-engine canonical negative cases — gated TD-121; отсутствие переноса чужих text/guard/suggestions обязательно, не waivable |
| S08 | Enter / hard boundary / чувствительное поле / курсор | Закрытие lifecycle; ASCII layout punctuation проверяется отдельно |
| S09 | Soft Reset после Backspace | Совпадает с контрактом продолжаемого либо завершённого токена |
| S10 | Exact V2 replay: source → empty → replacement | Защита transport сохраняется; один replay, без внутренней автозамены |
| S11 | V2 expired / wrong path / revoked / failed output | Старый guard не достигает будущего Space, cancel не затрагивает новый request |
| S12 | Legacy V1: before-only abort/failure; before/after success; duplicate; delayed after уже другого word/owner; text-replace; replay с boundary и без | Нет двойной автозамены, утечки на следующее слово и принятия чужого late after. Фазы не угадываются по счётчику или тексту; весь набор закрывает G120-1 |
| S13 | Tab / accepted candidate с пробелом и без | Защита только ещё открытого принятого слова |
| S14 | Auto-undo с trailing Space | Следующее слово не наследует запрет |
| S15 | Duplicate/rejected manual output | Нет нового guard и дополнительного feedback |
| S16 | Atomic preview / abort / commit / повторный commit; prepare→live arm/consume/revoke/owner transfer→old commit | До commit live guard не меняется; old commit не теряет новый guard/owner и не воскрешает consumed/revoked. Уже submitted output не replay/retry; нет двойного feedback. Settlement либо доказанная недостижимость закрывает G120-2 |
| S17 | Пакет обновился между подготовкой и Space | Нет stale candidate authority; guard не привязан к старому пакету |
| S18 | US/RU, mixed script, clean English, protected tokens | Без literal exceptions; настройки всех трёх safety profiles соблюдаются |

Считать отдельно: число строк матрицы, раскрытых fixture cases, применений,
правильных отказов, ложных применений, изменённых соседних слов, feedback events.
S06b/S07b остаются в обязательном denominator как GATED, а не удаляются и не
считаются PASS по S06a/S07a. G120-1/2/3 нельзя закрыть оценкой документа.
Отдельно требуются frame-bound `gjxbnfq → почитай`, clean controls и существующая
IME regression class; отсутствие suppression само по себе не равно conversion PASS.

## Проверки и передача

- Локально Cargo не запускать. Remote: `e@192.168.3.94`; из отдельного checkout
  с проверенной source parity, через `scripts/cargo-guard.sh` и resource profile
  выделенного хоста. Не использовать unscoped `cargo test`.
- Focused `--bin lay-ibus-engine` по новым тестовым группам, существующие
  `manual_toggle_suppression`, `exact_manual_toggle_suppression`,
  `physical_double_shift_owner_`, `composition_edit`, atomic proof cases.
- `scripts/check-lay-changed.sh`; новый manifest объясняет каждый добавленный
  case, старые обязательные проверки не пропускаются.
- `scripts/update-architecture-graph.sh`; не использовать запрещённый skill
  `nanda-structural-gate` и не заменять им обычные проверки проекта.
- Fresh-context Astra/XHigh review: findings first, score ≥8/10, High/Medium=0,
  максимум 1–2 repair passes. Непроходящий gate остаётся открытым.
- Live proof и release/install только отдельным согласованным этапом;
  сохранение global IBus PID и real-keyboard Double Shift confirmation обязательны.
- DONE только после доказательств, review, отдельного commit и push; сейчас
  задача **не выполнена**.

## Ревью спецификации

[Pass 1](evidence/td120-121-spec-review-pass1.md): overall 8/10, TD-120 7/10,
High=0, Medium=3. Единственный document repair явно закрыл неоднозначности
описания R1–R3: dependency, общий V1 gate и atomic interleaving. Это не
закрытие самих production gates.

[Pass 2](evidence/td120-121-spec-review-pass2.md): TD-120 **8/10**, overall
**8/10**, оставшиеся High=0/Medium=0 в проверенном scope. Описание принято
как результат фазы 1. Статус `ANALYSIS_REQUIRED`, G120-1/2/3 OPEN; следующий
шаг — доказательства допуска, не запуск общей миграции. Всего два review
прохода и один repair описаний; production-код не менялся.
