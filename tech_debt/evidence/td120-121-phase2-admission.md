# TD-120/121: продолжение после допуска пользователя

Дата: 2026-09-05. Пользователь: «дальше без остановок пожалуйста» после карты
разработки. Допущены продолжение доказательств, TDD, минимальные исправления,
независимые code reviews и отдельные commit/push после завершения задач.
Это не отменяет открытых design gates, безопасных ресурсных лимитов и запрета
на скрытую широкую миграцию. Release/install остаются отдельным этапом.

Baseline: `cc1e2207519801ca0f9b7c6963897b55953a7751`, worktree
`/home/ubu/projects/lay-tech-debt-20260831`. Phase-1 documents/review/graph
из предыдущего этапа сохраняются; старые untracked release receipts не трогать.
Production-код на момент этого допуска не менялся.

## Параллельная работа с разными владельцами

- Astra/XHigh: canonical context/handoff TD-121, отдельный analysis artifact.
- Astra/XHigh: V1/atomic/lifecycle TD-120, отдельный analysis artifact.
- Main: безопасное исправление bootstrap private diagnostic и remote build
  preparation. Sol/High получает production-код только после записанного выбора.
- Свежий reviewer не получает авторский контекст; 1–2 repair passes максимум.

## Новый диагностический bootstrap — последствия до изменения harness

Старая проба immutable: `layout-phase1-private-WTVpDE`, 0/6. Новый writable
каталог: `/home/ubu/.cache/lay/layout-phase2-private-31hE0X` (mktemp, mode0700).
Те же установленный бинарник/SHA, private PID/net/IPC/mount namespaces,
read-only host filesystem, private `/tmp`/`/run`/`/dev`/`/proc`, без desktop
environment, uinput, сети, live bus, learner/model и config writes.

Изменение bootstrap: явный config **только нового private dbus-daemon** с
`apparmor mode="disabled"`. Он не читает недоступную policy из read-only
sandbox и не изменяет kernel AppArmor, пользовательский/session/system bus
или политику хоста. Private bus обслуживает лишь собственные driver и engine
в изолированном PID/network окружении. Граница host isolation остаётся bwrap;
на private bus не подключаются рабочие приложения. Глобальный AppArmor не
отключать. XDG_RUNTIME_DIR задаётся в private mount до запуска private bus.

Цена: один private config, без новых production путей, caches, workers или
прав на Apply. O(1) bootstrap; CPUQuota50%, Memory768M, swap0, Tasks32,
Runtime45s. Сбой не разрешает live fallback, retry mutation, перенос догаданных
букв или weakening verifier. Собственные процессы завершаются/reap; чужие PID
сверяются до/после. Историческая `l` остаётся UNKNOWN даже при воспроизведении
класса handoff. Проверяется транспорт/учёт хвоста, не качество Wave.

План: одна новая попытка bootstrap, затем максимум один узкий harness repair
при конкретной ошибке. При повторном инфраструктурном отказе перейти к
source-bound тестам на remote, а не расширять права sandbox.

## Remote compilation boundary

Read-only preflight: `e@192.168.3.94`, hostname `e-MEGA-MINI-M1-13th`, 20 CPU,
доступно RAM около29GiB, диск свободен371GiB. Это снимок, не reservation.
Старые dirty remote checkouts не переписывать. Новый source checkout, явная
source parity, Cargo через resource/cargo guards и dedicated-20cpu profile.
Локально Cargo запрещён; число одновременных heavy scopes ограничивает guard.

## Бюджет изменения

По nanda-wave-spectral-budget: +5 safety, +5 route correctness, +3 testability,
−2 private diagnostic inputs = +11, без runtime/API/schema изменения.
Это оценка диагностического шага, не разрешение пока недоказанного runtime fix.
Запрещённый nanda-structural-gate skill не используется.
