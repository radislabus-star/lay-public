# Текущий этап 1.0.70: GitHub fixes установлены

Для #42 исправлена общая потеря ведущих символов daemon буфером; для #43/#44
исправлены состав/диагностика сборки и отсутствующая зависимость `jq`.
Проверки, установка и публичная поставка:
[root continuation](../CONTINUE.md),
[owning issue document](../docs/public-issues-42-44-release-1.0.70.md).
Текущая установка: `run-j4e7w_0n/installation-1.0.70.json` в development cache.
Последующая работа — физическая проверка, повторные Ubuntu результаты и
общий TD-123; прежняя сборка/установка не требует повторения. Счётчики выключены.

# Предыдущая установка 1.0.70: сохранённое IME доказательство

Потеря границы слова после Backspace исправлена: native 3/4 → 4/4.
Полные тесты 2723/2723, три fixed89 профиля без регрессий, final review 9/10
H0/M0/L0; десять бинарников и четыре процесса проверены после установки.
Receipt: `~/.cache/lay/development/run-p4d_z81t/installation-1.0.70.json`. Физическая проверка запрошена;
commit/push ожидают её результата. Общий TD-123 и составная ошибка «зуын»
→ «push» остаются OPEN. Счётчики не возобновлять; проверки не повторять.
Точная область и измерения — в [owning IME architecture](../docs/ime-daemon-route-map-2026-06-20.md#retained-boundary-final-validation-and-installation-2026-09-10).

# Lay — точка продолжения, 2026-09-09

Предыдущий milestone: 1.0.69 `DELIVERY_ACCEPTED` в пределах подсказки/Tab.
Подсказка первого слова и явное дополнение Tab отделены от полномочия
замены целого слова. Проверены 2717/2717 tests, три fixed89 профиля без
регрессий, native13 и отдельный first-word GUI7/7. Все десять бинарников,
четыре процесса и версия расширения проверены; IBus4715 сохранён.
Receipt: `~/.cache/lay/development/run-btt4ilfz/installation-1.0.69.json`.
После сохранённого intermittent-hint report пользователь сообщил «Push отлично»
и разрешил публикацию. Приложение последней проверки не уточнено. Новый отказ
автопереворота «зуын» → «push» исследуется отдельно; не менять runtime без
причинного доказательства. Commit/remote-ref receipt: `run-btt4ilfz/publication-1.0.69/`
в локальном development cache; rebuild/reinstall не нужен.
Общая цель TD-123 остаётся OPEN, счётчики не возобновлять. Полные результаты
и ограничения — в конце owning IME architecture и в root continuation.

## Предыдущая принятая поставка 1.0.68

Предыдущий milestone: `DELIVERY_ACCEPTED`, версия 1.0.68; поставка DONE.
Исправление редких подсказок прошло causal RED/GREEN, runtime review9/10,
2711 tests и release gates; fixed89 три профиля без регрессий; native13,
legacy hints8/8 и correction40/40. Установленные и загруженные байты проверены,
IBus4715, модели, config и источники ввода сохранены. Receipt:
`~/.cache/lay/development/run-hvrxwcxk/installation-1.0.68.json`.

2026-09-09 07:19 UTC пользователь подтвердил «работает !» и разрешил push.
Счётчик точек остановлен по просьбе пользователя; не возобновлять. Не повторять
установку и неизменённые проверки. Narrow terminal private correction3/5
остаётся прежним ограничением без физического daemon; общий desktop PASS
не заявлен. Полные факты, классы и ограничения:
[root continuation](../CONTINUE.md),
[owning IME architecture](../docs/ime-daemon-route-map-2026-06-20.md#rare-ime-suggestions-after-the-spontaneous-input-report-2026-09-09).

Общая цель [TD-123](123-improve-wave-restoration-quality-for-1.0.67.md)
«Wave стала умнее вообще» остаётся OPEN. Полный L1/L2 proof и два прежних
performance FAIL относятся к 1.0.67; код моделей и пакеты здесь не менялись.
Принятый результат публикуется в `origin/codex/cleanup-20260908`; точный remote
ref и локальный receipt отделены от общей цели качества. Продолжать в новой
копии cleanup; исходная сохранена. Не уходить в общий исторический аудит.

## История до установки 1.0.67

Предыдущий bounded milestone: private1dc3a283, текущий тогда код2233/2233 PASS,
fresh review9/10,H0/M0/L0. Ограниченная L3 recurrence для двух правок прошла
весь fixed89 без изменений к1b (20/47 dirty,40/42 clean), отдельную семью
16 случаев (0/8→6/8 dirty,clean8/8), native13 и новую quiet80ms серию:
missing8/8,composite8/8,clean24/24. Две оставшиеся двойные вставки имеют
prepared candidates0/field producers0 до L3 и DecisionCore. Далее — общие
оставшиеся механизмы качества. Все проценты/ресурсы/ограничения и receipts —
последний раздел owning architecture. Установки нет; общий quality/release/
physical verdict OPEN. На переходах показывать актуальную карту деревом.

- Установлен проверенный IME995b6093: PID3983217/start49187106,
  unitlay-ime-release-995b6093.service, timestamp2026-09-08T12:28:55.649410+00:00.
  File и /proc SHA совпадают. IBus4715,daemon3757261,L1.1service271400,
  два input sources и выбранный Lay RU сохранены. Backup6dc95148:
  /home/ubu/.local/state/lay/release-backups/ime-autocorrect-20260908-71oibez0/lay-ibus-engine.
- Две исправленные причины: первый cold V90 load больше не отбрасывает
  собственный подготовленный результат; L4 word/context-only prior больше
  не выдаётся за отрицательный transition. Полный прежний owner-precedence
  predicate, включая exact-positive exemption, сохранён отдельно.
- Runtime review9/10,H0/M0/L0; focused2206 PASS. Changed/full2684/2684 PASS
  каждый,414.006/651.982s. Exact private client13 cases PASS (US1,RU1,manual3,
  lifecycle3,restoration5);4 physical-owner code tests PASS. Rust697 identity
  неизменна. Target9,397,526,528<12GiB. No new daemon binary/model permissions.
- Build checks PASS; postbuild recorder FAILED на неверном требовании
  embedded receipt и сохранён как failure. Per-file candidate identity и
  отдельные source architecture/release gates подтверждены; без rebuild.
- Actual phrase dirty0/2 ->1/2,clean3/3 unchanged; original matrix2/6 dirty,
  1/1 clean unchanged. Fixed89 existing fixtures:dirty19/47 correct,25 abstain,
  3 wrong unchanged;clean38/42 ->39/42. No observed regression только в
  experimental ordered post-ready scope. General/model/heldout/all-profile/
  immediate-Space quality не доказана. TD-123 НЕ закрыт.
- Физическая проверка обычного темпа запрошена после установки и PENDING.
  Исходный combined two-field/profile failure остаётся отдельным OPEN.
- Residual composite: target добавлен L2,common L3 видит его, но exact learned
  profile отсутствует; existing sentence recurrence certifies distance1 only.
  L1 current geometry calibration0 — отдельный модельный вопрос, не разрешение
  поднять threshold без fixed proof. Модель после этой установки не менялась.
- Текущие receipts: /home/ubu/.cache/lay/development/run-c1abdlhw/{installation.json,gates.json,candidate-identity.json},
  ~/.cache/lay/development/autocorrect-live-ojoasco5/fixed-fixtures-v3-comparison.json.
  Все per-class результаты/ошибки reader и recorder сохранены в owning docs.
- Продолжение после очистки: диагностическая9f8e99e0 измерила очередь; частная
  37073948 переиспользует неизменяемую сигнатуру кандидата внутри одного вызова
  L3.126/126 readout совпадают,2209/2209 focused PASS, свежий review9/10.
  При80ms очередь46.8→20.7ms, всего120.4→92.0ms: срок всё ещё не выполнен.
- 37073948 ОТКЛОНЕНА для установки: fixed89 dirty19/47 без изменения,
  clean39/42→36/42. Для двух новых wrong-layout baseline терял L1.1 запрос;
  полные новые поля открыли ordinary ProductiveLayout Eligible при canonical
  Failed/Abstain. Третья строка — каскад сохранённой US-раскладки. Это не
  доказательство изменения L3-ранга. Установлен995b, новые бинарники частные.
- Общий обход коротких layout-целей исправлен в частнойfa15601f: один прежний
  predicate для deterministic и ProductiveL2; grounded/matching Winners,
  lattice/evidence и context promotion сохранены. RED:1760 PASS, только2 новых
  FAIL; GREEN:2211/2211; свежий review9/10,H0/M0/L0. Все89 actual outputs
  снова совпали с995b:19/47 dirty,39/42 clean;267 статусов до/после перепроверены.
  Exact native13/13 PASS. Сборка и architecture PASS; установок не было.
- Фиксированные8 пар при80ms завершены: доставка missing-letter0/8→7/8,
  clean24/24 у обеих; composite ready-NoApply0/8 отдельно. Gate8/8 FAIL.
  Единственный новый срыв:22,262us очередь+80,589us вычисления,103ms всего;
  L1.1 внутри него2,533us. Это повтор одной фразы, не8 независимых ошибок.
  Exact private receipt: cadence-stability-summary-v1.json, все16 raw runs
  сохранены. Модель/пакеты/runtime installation не менялись.
- Текущий этап: отказы готового расчёта, затем полная проверка и поставка.
  Ограниченный срок к Space пройден описанным ниже переиспользованием работы.
  Повторный decoder reconstruction теперь переиспользуется до усечения по
  limit. RED подтверждён;224 полных readout совпали,21 обход вместо147,
  шесть финальных reader/resource tests и IME PASS, code review9/10,H0/M0/L0.
  Privateeea32f44 собрана,697Rust SHA совпадают; fixed89 полностью совпал
  с995b/fa,13native controls PASS. Новая парная серия80ms: parent5/8,
  cache8/8 своевременных исправлений, clean24/24 у обеих;52.7–64.5ms наcache.
  Bounded readiness gate PASS. Composite0/8 ready-NoApply остаётся открытым.
  Ready-path audit:23/25 целей удержаны;21 без L3/exact-L4 authority, две
  отклонены L4 ambiguity. Один полный lexical Winner теряет frame authority
  из-за общей неопределённости. После вариантов9/8/4 и анализа последствий
  реализован ограниченный перенос capability. RED воспроизвёл этот отказ;
  финальный scoped GREEN2 221/2 221, design/code review9/10,H0/M0/L0.
  Частная7a58535c/run-1E8BoD прошла full fixed89:19/47→20/47 dirty,
  clean39/42 без изменений и без новых wrong outputs.13 native controls PASS;
  quiet80ms8/8 у неё иparent, clean24/24, composite0/8 у обеих. Наблюдаемый
  VmHWM352.53MiB; no swap/OOM. Медиана общей подготовки выросла53.030→66.788ms,
  хотя медиана DecisionCore9.098→8.866ms; speedup/cost-neutrality не заявляются.
  Свежий remaining-path audit:24 refusals,22 retained targets —21 SuggestOnly,
  одна Eligible ambiguity; два boundary targets отсутствуют. Продолжить поиск
  первого общего недостатка authority; release/install/physical пока открыты.
  Отдельный L1.1 service pilot43 однословных повреждений:23 цели в lattice,
  0 target Winner. Предыдущие98.94% — objective-unique rank, а не Apply-rate.
  Лексическая/контекстная недостаточность остаётся отдельной открытой работой.
  Никакой установки/model promotion ещё нет. Подробности и точные receipts —
  последний раздел owning architecture; l11-and-native-first-loss-v1.json
  и fixed89-ready-path-audit-v1.json в private autocorrect-live-ojoasco5.
  Полные актуальные факты: конец docs/l2-l11-canonical-architecture.md;
  private autocorrect-live-ojoasco5/fixed-fixtures-v4-pair-memo-comparison-v2.json
  и fixed-fixtures-v5-short-layout-comparison.json. Build/component/native
  receipts: /home/ubu/.cache/lay/development/run-xqb_96ka/. TD-123 остаётся ACTIVE.
- Текущее продолжение после этого milestone: scoped capacity repair в частной
  1bceb869/run-X9Migg, без установки. Final L2 relation partitions больше не
  теряют полноту из-за вместимости raw set. Full fixed89, native13/resources и
  тихая серия80ms завершены; подробный итог и first-loss classification —
  последний раздел docs/l2-l11-canonical-architecture.md. Текущие receipts:
  ~/.cache/lay/development/run-vx8clfk5/ и autocorrect-live-ojoasco5/*exact-capacity*.
  Следующий read-only этап: недостающая поддержка составных повреждений и
  независимые L3 основания для20 complete ties; общий TD-123/release/physical открыт.
