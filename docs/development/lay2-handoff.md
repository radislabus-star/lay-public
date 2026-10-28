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

## Сейчас:107, НЕ опубликована; C05 read-resample repair

Механическая изоляция: Chrome process policy из kde/display/chrome.rs в
kde/chrome.rs; тела/100ms budget/guards прежние,1test namespace migration.
107 source ff9ca136... прошла3548 source tests,18gates,10artifacts.
Full source local run-lag441hb / remote run-1dSrwv. GatesV2 PASS с явным
ADR doc-overlay (manifest/known_failures должны быть в protected_paths).

Установлена107 V1: engine1871589, daemon1871710, tray1871720. Полный
неизменённый Chrome220:178PASS1FAIL41BLOCKED0NOT_TESTED. Провал normal/search
space-en-ru, row17: ghbdtn остался послеSpace. Релиз не пушить/публиковать
до исправления. Nativephase остановилась до omnibox/Firefox; они не проверены.
Raw CACHE/release107-chrome-actor-v1.json, release107-chrome-row17-v1.jsonl.

Причина: решение eligible/apply готово645us, epoch272, generation5. Native read
видит5из6 букв; owned_append_resample НЕ вызван. Final identity/edit guard
правильно отказывает,0replacement. Read eligibility использует строгий старый
native binding token; ordinary callback settlement уже продвинул word token.
Для чтения нужно сохранить same field/focus/lease/layout transport и проверить
текущий token; edit receipt остаётся строго привязан к старому actual observation.
Не менять committed_tail/managed/preedit, C03/C07, timers, executor илиGNOME.
Не обобщать pair-only post-Space action: найден более узкий KDE read-only fix.

Preflight/decision:
- docs/development/release-1.0.107-chrome-client-isolation.md
- docs/architecture/decisions/2026-10-28-kde-native-key-scope-read-resample.json
Дополнительные runtime/test paths только kde/native_document/preparation.rs
и tests.rs, плюс manifest/knownfail для2новых semantic tests.

TDD V1 RED83/85: одна zero-character fixture была неприменима до целевого
действия, это не полный causal proof. Исправлено до настоящего observed-owned
prefix длиной1или5 + собственная следующая буква; missing witness — negative.
TDD V2 causal RED83/85: все83прежниеPASS, оба новыхFAIL; reader1 вместо2 и
false resample permission после production WordScope settlement.
Raw isolated-release107-key-scope-unit-v2-red/result.json, source1c3d0907...

GREEN V3 завершён:85/85PASS; raw isolated-release107-key-scope-unit-v3-green/result.json.
 изменён AST-ом ровно один KDE read predicate.
Он использует binding.observation_owner_is_current_for_token(self,currentToken)
с прежними coordinate/privacy/selection/exact-refresh/handoff и managed-witness
проверками. Старый native_word_preparation_identity/edit predicate НЕ менялся.
Source-only resample по-прежнему максимум2, тот же Instant/80ms, без sleeps.
Проверяет85 точных identities: oldNative + pair + C03fence +2новых.
Не печатать весь result.changed: там большие исходники; импорт SHA/mode
только разрешённых файлов.

Manifest обновлён:3514correctness+36package=3550,1113IME,11performance,41ignored; knownfail0. Graph AST обновлён; canon preflightPASS доfullsource.
Сейчас official full3550, execsession97281; local run-0dgci0vd.
Source708926cec7f4600a831580464ef75679242eb359b9d72c20a2480b058e3f2e7c.
ПослеPASS новые gates/installer/controller/full-native V3 ужезагружены.
GatesV3 ожидают3550/1113 и текущий frozenrequest. Не запускать V1/V2драйверы.
Использовать новые versioned пути V3, старыеFF9failedproof не перетирать.
Новыйinstall должен ожидать previous isolated-release107-deployment-v1 и
новыйSHA/PID; версияостаётся107, tagещёсвободен.

Затем неизменённый Chrome220 и существующий реальный omnibox3. У обычного
Chrome profile SingletonLock pid262925 мёртв: сначала обычный запуск подguard
через реальный desktoplauncher, не сбрасыватьпрофиль. Не запускать отдельное
окно покаактивен другойnativeactor. Далее свежий Firefox220 baseline для108.

Actor dependency-browser-native-full-r19.py SHA
360e6b9a0735fef75aba16c8d89d57d1e9447bf0cf5603c70213227843c24913.
Cadence120ms и критерии не менять. Верхнийomnibox отличаетсяотHTMLURL.
После107 acceptance commit/push/CI/stable/updater; затем108–111. Один remote
heavy/inputactor; следующиеsource checks могутперекрывать внешнююGitHubCI.

## Remote и инструменты

Host e@192.168.3.94, SSH key `/home/ubu/.ssh/mega-mini-admin`, strict known_hosts.
CROOT `/home/e/projects/lay-development-runner/kde-install-control-20261020`.
Workspace `/home/e/projects/lay-development-runner/workspace`. DISPLAY=:71/KDE,
RFB localhost5905; GUI env брать из текущего daemon `/proc/<pid>/environ`.
Manager env GNOME :10 не использовать и не импортировать глобально.
Защищённый IBus parent3168326 / lay-release97-restored-system-ibus.service,
activation binary `/home/e/projects/lay-td119-gate-v1/target/release/lay-ibus-engine`.
Сейчас установлен107 V1: engine1871589, daemon1871710, tray1871720; SHA engine
`d472b8e35a3fe7dba8ea931f6f447ed218e6f5aba51897bd77d690ce50aa685b`. Следующая установка после новых gates заменит эти bytes.

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
