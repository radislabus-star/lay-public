# TD-120 / TD-121: граница изолированной диагностики

Дата: 2026-09-05. Фаза: только диагностика и проектирование, до production-кода.
Baseline checkout: `cc1e2207519801ca0f9b7c6963897b55953a7751`.
Установленный `lay-ibus-engine`: 1.0.65, SHA-256
`391b3b44025461a71d6867dc92584dfa1906c1f8ae72ea610a13d2265aaacfab`.

## Что проверяется

1. Получает ли следующий экземпляр IME уже наблюдавшуюся первую букву после
   `FocusOut / Disable / CreateEngine / FocusIn` без смены реального окна.
2. Отличаются ли обычный `FocusIn`, одинаковый `FocusInId` и новый клиент.
3. Сохраняется ли обычный запрет автозамены после удаления всего защищённого
   слова и набора нового слова; перед Space читается точный `VisibleTailV3`.

Пробы используют настоящий установленный бинарник и его публичные D-Bus
методы. Они **не** являются GTK/Kitty/Wayland E2E, доказательством физической
доставки клавиш, качества корректора или точной причины исторического `lом`.
Полный расчёт Nanda и автоматическое переключение desktop отключены только
в отдельном конфигурационном файле пробы.

## Последствия и изоляция до запуска

- Production-код, installed binaries, пользовательский config, git index и
  рабочие сервисы не изменяются. `runtime_authority_changed=false`.
- `bwrap`: отдельные PID/network/IPC namespaces, `/` read-only, отдельные
  `/tmp`, `/run`, `/proc`, `/dev`; writable только конкретный каталог пробы.
- Private `dbus-run-session`; оба engine/session соединения направляются на
  него. PATH shim допускает только `ibus address` с private address; другие
  команды IBus запрещены. `HOME` не переопределяется.
- Нет `DISPLAY` / `WAYLAND_DISPLAY` из пользовательской сессии. Нет uinput,
  evdev, обычного runtime smoke, system service reload, сетевой модели.
- `nanda_autocorrect=false` не допускает startup-запуск/перезагрузку L1.1;
  private config/trace/L1.1 socket/usage paths и XDG каталоги не меняют
  production learning. Read-only mounts блокируют HOME fallback writes.
- Ограниченный diagnostic cgroup: CPUQuota=50%, MemoryMax=768M,
  MemorySwapMax=0, TasksMax=32, RuntimeMaxSec=45s, Nice=19. Это не build gate.
- Driver завершается по таймауту, останавливает только собственный subprocess;
  PID namespace убирает собственных потомков. При ошибке не переключается на
  live bus и не ослабляет изоляцию. Никакой фоновой подписки после пробы.
- `VisibleTailV3` сам вызывает refresh пустого хвоста из shared state; это
  observer effect необходимо учитывать при трактовке результата. RPC ответы
  записываются отдельно от lossy debug-log. Не делать вывод о недоставке клавиши
  только из отсутствия строки key trace.
- Первая буква должна быть подтверждена до handoff; ввод после него и новый
  хвост измеряются отдельно. Никакого увеличения backspace count или угадывания
  символа по координатам курсора. Ranking / verifier / SafetyGate не меняются.

## Артефакты и границы утверждений

Временный writable root:
`/home/ubu/.cache/lay/layout-phase1-private-WTVpDE` (создан mode 0700).
Driver, config, shim, stdout/RPC receipt и trace остаются в этом каталоге;
проверенные результаты и reproducible inputs переносятся в `tech_debt/evidence/`.
Команда, хеши, число кейсов, наблюдаемые исходы, неиспытанные ветки и статус
cleanup фиксируются после запуска, не до него.

До запуска статус результата: `NOT_RUN`. Этот документ разрешает только
описанную изолированную диагностику, не production-реализацию или релиз.

## Итог выполнения — добавлен после обеих попыток

Результат: `HARNESS_BLOCKED_BEFORE_CASES`, выполнено **0 из 6** плановых
кейсов. Это не semantic FAIL и не reproduction PASS. Предстартовая граница
выше сохранена; состояние `NOT_RUN` относится только к моменту до запуска.

| Попытка | Команда и unit | Наблюдаемый результат |
|---|---|---|
| 1, 13:12:26 EEST | `./launch.sh`, `lay-phase1-private-WTVpDE.service` | `bwrap`: `Can't mkdir /proof: Read-only file system`; до запуска engine |
| 2, 13:14:14 EEST | `./launch.sh`, `lay-phase1-private-WTVpDE-repair1.service` | Единственный repair перенёс mount в `/tmp/proof`; private bus authentication: `Failed to query AppArmor policy: Read-only file system` |

Команды выполнялись из указанного private root. После второй попытки inputs
больше не правились и запуск не повторялся. Успешных Factory/Focus/Key/Tail RPC
нет; private receipt содержит только startup/failure и cleanup. Собственный
engine завершён (`returncode=-15`) и reaped. Оба diagnostic cgroups пусты.
Измеренные duration/peak memory: попытка 1 — 17 ms / 2,035,712 B;
попытка 2 — 349 ms / 25,796,608 B. Лимиты не превышены.

Независимая от probe read-only проверка после него: рабочие PID global
IBus `4715`, daemon `3453123`, IME `3453154`, L3 learner `3453057`,
L1.1 `3452522` сохранены. Production diff отсутствует.

Артефакты сохранены в private root, полный пользовательский журнал в Git
не переносится. Основной агент прочитал отчёт и проверил SHA-256:

| Файл | SHA-256 |
|---|---|
| `execution-report.md` | `4b5e3d0152a5d46f9fd6592dfd666cb6639be04bec03f827aa484535fb093c4e` |
| `receipt.json` | `26ab8c768a9b66e74864bf797c94669d4489eb30f9c59f0feafea4ec10f1ca08` |

Точная историческая причина потери `l`: `UNKNOWN`. Проверка качества
коррекции: `NOT_TESTED`. `runtime_authority_changed=false`.
Следующий probe требует сначала отдельного безопасного решения bootstrap
private bus; не допускается подмена его live smoke или ослабление изоляции.
