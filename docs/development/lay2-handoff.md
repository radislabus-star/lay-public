# Текущий handoff Lay: семь изолированных релизов 105–111

Поручение пользователя: самостоятельно сделать семь содержательных релизов,
проверить и опубликовать их; разделить интеграции для отдельных правок и тестов.
Не заканчивать работу после промежуточного отчёта. Никаких вопросов, Harness²
или субагентов. В конце всей серии вызвать установленный звуковой hook.

## Источник и ограничения

Checkout `/home/ubu/projects/lay-dependency-updates-20261027`, ветка
`codex/browser-matrix-repair-20261027`. `origin` — PRIVATE lay.git; его main
с чужой историей 113 не менять, не ребейзить и не пушить. `public` — lay-public.git,
ветка main. Пушить свою ветку в origin и принятые коммиты fast-forward в public/main.
Теги только публичных принятых байтов, без force. Текущий HEAD смотреть git.

Канон C01–C10, AGENTS.md, ARCHITECTURE.md и полевая изоляция обязательны.
Решение коррекции, admission/ownership, verifier и один исполнитель общие.
Адаптеры среды/клиента не создают копии доставки, владельца, таймера или fallback.
Общие изменения честно обозначать общими. До правки — карта и точный preflight.
Сохранять FAIL/BLOCKED/NOT_TESTED и первый нарушенный переход.

Все сборки/тесты/graph update только на remote под существующими guard.
Локальный GNOME не устанавливать, не перезапускать, не вводить в него клавиши.
Проверка updater локально только `~/.local/bin/lay-update --check`.
Публикация принятого source не требует повторных функциональных проверок
при изменении только metadata/docs; доказательство сравнивает точные байты.

## Опубликовано

- 1.0.105 / `4ebd03f3566a981a9be384a67e1a7b1013b84ff4`: CI 4/4 SUCCESS,
  tag/release и оба updater проверены. Source `85cf6f9f...`. 3548 source tests,
  18 gates, 10 release-бинарников, 0 compiler warnings. Нативные 7/7 и полный
  Chrome220: 180 PASS / 0 FAIL / 40 BLOCKED. Firefox для этих байтов не проверен.
  Исправлены KDE native delayed-Space continuation, точный successor token и
  один resample incoherent read в прежнем бюджете 80 мс. Не возвращать старые
  отказы: см. release105 verification и owning document.
- 1.0.106 / `f88c4ba932206dd96bc04158778109f63606d250`: stable опубликована,
  CI 4/4 SUCCESS, updater local104→106 / remote106 current. Source `df48fc2b...`,
  3548 PASS, 18 gates, 10 artifacts, 0 warnings, 7/7 native controls. Полная
  Chrome220 для106 не повторялась. Три KDE preparation entrypoint перенесены
  из общего observation в kde/native_document/preparation.rs. Общий witness
  остался единственным; две функции AST-эквивалентны, separator projection
  сохраняет identity/revision/expiry и прежний порядок tail ack → witness.

Полные доказательства: `tech_debt/evidence/release-1.0.105-verification.json`
и `release-1.0.106-verification.json`, соответствующие updater receipts.
Git blobs/executable bits совпадают с tested request; writable group bit
Git нормализует. Не называть полные filesystem modes коммитными режимами Git.

## Сейчас:107, НЕ опубликована

Разрешённый preflight: `docs/development/release-1.0.107-chrome-client-isolation.md`.
Код проверки процесса Chrome перенесён из kde/display/chrome.rs в kde/chrome.rs;
два namespace вызова обновлены, тела и условия/бюджеты прежние. GNOME не тронут.
Один test namespace перенесён без изменения тела/категории; остальные identities
сохранены. Не ослаблять manifest ради переименования.

Source `ff9ca136f7ea8668cbc062a4acbf1a1600943c76c8c96a3b502b45610ae3257d`.
Полный source 3548/3548 PASS: local `run-lag441hb`, remote `run-1dSrwv`.
Manifest: 3512 correctness, 36 package, 11 performance, 41 ignored. IME1111.
Сейчас guarded gates: `isolated-release107-gates-v2`, exec session18575.
V1 отказал из-за отсутствия manifest/known_failures в protected_paths решения.
Preflight уже разрешал ровно одну namespace migration; в ADR это дополнено.
В V2 ровно один doc overlay, замороженный full request и source неизменны.
V1/prepare V1 отказы сохранены. Не повторять3548 только ради doc исправления.

После gates: импортировать только разрешённые metadata с проверкой SHA/mode,
запустить `isolated-release107-native-phase-v1.py` под remote guard. Он делает:
1. установку10 retained artifacts с PID/hash проверкой;
2. неизменённый полный Chrome220, требуя0 FAIL и0 NOT_TESTED;
3. существующий chrome-omnibox-native-capabilities-20261024.py: три реальных
   проверки верхней адресной строки (hint1, hint2, Tab2), без Enter;
4. полный Firefox220 как свежую baseline для108, с явным сохранением FAIL.

Полный actor заморожен: dependency-browser-native-full-r19.py, SHA
`360e6b9a0735fef75aba16c8d89d57d1e9447bf0cf5603c70213227843c24913`.
Не менять утверждения, 120 мс cadence, порядок жестов и deadline ради PASS.
Верхняя адресная строка отличается от HTML URL-поля. Firefox baseline не
считается успешной приёмкой Firefox; использовать её для первого общего механизма.
После приёмки107 — commit/push/CI/stable/updater, затем108–111 по roadmap.
Подготовка следующей версии может перекрывать внешнюю GitHub CI предыдущей;
remote heavy/input actor один. Принятые commit/source/artifacts каждого шага
неизменны; stable-публикации по порядку и только после собственного CI.

## Remote и инструменты

Host e@192.168.3.94, SSH key `/home/ubu/.ssh/mega-mini-admin`, strict known_hosts.
CROOT `/home/e/projects/lay-development-runner/kde-install-control-20261020`.
Workspace `/home/e/projects/lay-development-runner/workspace`. DISPLAY=:71/KDE,
RFB localhost5905; GUI env брать из текущего daemon `/proc/<pid>/environ`.
Manager env GNOME :10 не использовать и не импортировать глобально.
Защищённый IBus parent3168326 / lay-release97-restored-system-ibus.service,
activation binary `/home/e/projects/lay-td119-gate-v1/target/release/lay-ibus-engine`.
Сейчас установлен106: engine1691255, daemon1691376, tray1691382; SHA engine
`3c8d65db3667570246a4fe4736a2a0f2bae62f87e82f811bfec777f9b1dd5c21`.

Guard: source dedicated-resource.env, memory high24G/max28G,20CPU,swap1G,
one exclusive heavy lease; CARGO_TARGET_DIR=candidate-target-cache-20261023.
Cargo только scripts/cargo-guard.sh, бюджет12GiB. Нет нужды чистить диск.

CACHE `/home/ubu/.cache/lay/development/browser-matrix-repair-20261027`.
Все текущие release106/107 scripts, raw receipts, hashes и планы там.
Official entrypoint scripts/dev-check.py check --config
`/home/ubu/.cache/lay/development/dependency-updates-20261027/development.json` --compact.
Не использовать legacy bump-lay-version.sh: он устанавливает/перезапускает GNOME.
Версии синхронизировать source-only в Cargo, extension labels и VERSIONING.

CACHE/verify-committed-source.py проверяет неизменяемый commit против реального
tested request без сборки. CACHE/publish-verified-release.py публикует только
после4 GitHub SUCCESS и проверяет stable API/tag + оба updater --check.
Не запускать publisher до CI SUCCESS; exit75 означает ещё неготовую CI.

Исторический handoff доступен в Git: f88c4ba9:docs/development/lay2-handoff.md.
Он историческая информация, не второй активный план. Актуальная очередь —
`docs/development/isolated-release-series-2026-10-28.md`.
