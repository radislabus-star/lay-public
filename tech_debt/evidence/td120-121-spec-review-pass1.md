# TD-120 / TD-121: независимое ревью спецификаций, pass 1

Дата: 2026-09-05. Baseline: `cc1e2207519801ca0f9b7c6963897b55953a7751`.
Фаза 1, один независимый проход. Production-код не изменён; Cargo, новые
пробы, live bus/service/config/binary операции, commit/push не выполнялись.
Единственная запись reviewer — этот файл. `runtime_authority_changed=false`.

## Findings

### R1 — Medium: самостоятельность TD-120 противоречит выбранному handoff scope

**Место:** `120-scope-autocorrect-suppression-to-word-lifetime.md:91`, `:98`,
`:176`; `README.md:57`.

README разрешает независимый TD-120 с сохранением текущего handoff contract,
но спецификация требует подтверждённый same-context перенос и запрещает TTL
как основание. В baseline `engine.rs:98–119` разрешает перенос по
`preserve_active_path_until`; canonical context не проверяется. При принятом
переносе `engine.rs:131–141` меняет runtime owner и копирует receipt. Поэтому
S06/S07 в общей формулировке требуют ещё не выбранного механизма TD-121 либо
молча меняют существующую совместимость. Это препятствие определённости scope,
не требование немедленно реализовать canonical identity.

**Repair:** явно разделить самостоятельный core TD-120 — word lifecycle при
доказанном текущем владельце и regression parity существующих admitted replay
routes — и gated cross-engine/context требования. Указать, какие S06/S07 cases
проверяют только baseline parity, а какие ждут TD-121 и не засчитываются как
пройденные. Альтернатива — честно сделать весь выбранный дизайн зависимым от
TD-121. Одного заявления «TD-121 не включать» недостаточно.

### R2 — Medium: нерешённый V1 должен блокировать всю unified migration

**Место:** `120-scope-autocorrect-suppression-to-word-lifetime.md:79`, `:125–130`,
`:182`.

Предупреждение о V1 верно, но «остановить эту часть» конфликтует с обязательным
удалением независимых boolean owners. `correction_runtime/output.rs:89` вызывает
V1 до ветвления text replacement/replay; `output/replay.rs:59` вызывает V1 после
успешного replay, а failure на `:55–57` возвращается без второго вызова.
`bridge_actions.rs:120–135` принимает RPC без token/request/phase аргументов и
безусловно ставит тот же общий запрет. Нельзя предполагать, что два вызова всегда
образуют пару, или молча присвоить им один из двух новых scopes.

**Repair:** сделать доказанную V1 совместимость отдельным обязательным gate
перед **всей** миграцией общих producers/consumer в unified record. Если gate
не закрыт, единая миграция остаётся analysis-only; отдельный узкий lifecycle
вариант допустим лишь как явно выбранный и проверенный дизайн. Раскрыть S12:
before-only после abort/failure, duplicate calls, delayed after с изменившимся
владельцем/словом, replay с trailing boundary и без неё, text-replace ветка.
Нельзя различать phases счётчиком вызовов или догадкой по текущему тексту.

### R3 — Medium: S16 не проверяет overwrite нового live guard старым atomic commit

**Место:** `120-scope-autocorrect-suppression-to-word-lifetime.md:79–82`, `:157`,
`:186`.

Baseline действительно клонирует SharedState отдельно (`atomic.rs:119–123`),
но commit целиком заменяет live SharedState speculative-копией (`:196–205`).
Проверка settlement (`:139–142`) связывает daemon focus epoch/transaction receipt,
но не сравнивает поколение текущего shared suppression/owner. Само отсутствие
записей из preview в live state не защищает от более позднего overwrite. Пока
spec требует сохранить существующий clone/commit, нужно проверить interleaving,
а не считать его автоматически совместимым с единственным shared guard.

**Repair:** расширить S16 последовательностью prepare → live guard arm/consume/
revoke или owner transfer → old commit; проверить отсутствие восстановления
израсходованного guard и потери нового. Принять существующую сериализацию только
при доказательстве недостижимости этого interleaving для всех producers.
Иначе до migration требуется решение о generation validation/settlement scope;
не включать широкую atomic ownership миграцию в TD-120 молча.

## Проверенные основания и пределы

- Хеши двух frozen logs, private execution-report и receipt совпадают с аудитом.
  Узкий разбор подтвердил 54 Backspace / 45 при tail=0, затем новое слово и
  `manual_toggle_suppressed` на строке 1766. `state.rs:585–588`,
  `composition_edit.rs:44–51`, `committed_tail.rs:550–562` и `managed.rs:65–107`
  подтверждают unscoped suppression до обычной коррекции. Победа `почитай`
  после исправления не проверена.
- Actions 85–88 содержат два `candidate_before_apply` на каждый эпизод; они не
  доказывают две реальные мутации. Планы — `ctujlyz → сегодня ` / 7 Backspace и
  `jv → ом ` / 2 Backspace. Debug timing подтверждает applied route. Доставка
  исторической первой `l` и происхождение US switch остаются UNKNOWN.
- `FocusId=false` подтверждён `ibus_interface.rs:302–305`; git blame указывает
  `427b3d7ed`, 2026-06-19. `state.rs:262–279,332–340` и `engine.rs:90–156`
  подтверждают разные условия сохранения старым engine и приёма новым.
- Upstream tag `1.5.34-rc2`, `bus/engineproxy.c` и `bus/ibusimpl.c` независимо
  просмотрены: cache по engine name, native FocusInId и read-only property
  CurrentInputContext существуют. Работа runtime property, её пять timings,
  PID/cgroup состояние здесь повторно не измерялись; это provenance основного
  аудита, а не новые замеры reviewer. Эквивалентность Debian patches не доказана.
- Private receipt имеет `probe_status=FAILED`, пустой `cases`, reaped engine
  с returncode=-15. Описательная классификация `HARNESS_BLOCKED_BEFORE_CASES`
  соответствует report: 0/6, никакого semantic PASS/FAIL. Scope корректно
  отмечает observer effect VisibleTailV3 и запрещает замену private probe live
  smoke. Полный пользовательский текст в review не переносился.
- Graphify использован для навигации, код — для проверки поведения. Граф не
  обновлялся. Запрещённый nanda-structural-gate не использовался.

## Оценка и readiness

**Общая оценка документации: 8/10.** Root-cause evidence, варианты, consequence
analysis, undo/exact replay/Double Shift ограничения и разделение denominators
сильные. Три Medium относятся к определённости выбранной миграции TD-120.
High: 0. Medium: 3. Оценка документа не является implementation admission.

| Задача | Оценка документа | Verdict |
|---|---:|---|
| TD-120 | 7/10 | SPEC_REPAIR_REQUIRED. Доказательств достаточно для проектирования failing lifecycle tests после phase-2 admission; выбранная unified migration пока не готова |
| TD-121 | 8/10 | ANALYSIS_REQUIRED корректен. Документ пригоден для следующего доказательного анализа; production implementation не готова |

Открытые TD-121 gates — двухобъектное воспроизведение, выбранный безопасный
canonical receipt на работающем IBus, callback/request ordering и доставка
клавиш во время смены — допустимо оставить открытыми. Они не являются дефектом
честно ограниченной phase-1 спецификации и не закрываются этим review. Матрицы
18/16 строк не являются 18/16 выполненными тестами; требуются раскрытые cases.

## Условия одного повторного document review

Проверить только repair R1–R3 и согласованность task/audit/README статусов:
scope TD-120 не зависит скрыто от TD-121; V1 gate блокирует unified migration;
atomic interleaving имеет явные outcomes и admission boundary. Сохранить TD-121
ANALYSIS_REQUIRED, private 0/6 и отдельный phase-2 admission. Запуск тестов,
новый probe или повышение production readiness для этого document repair
не требуются. Production readiness обоих исправлений этим проходом не выдана.
