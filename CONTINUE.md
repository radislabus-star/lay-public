# Lay 1.0.71 installed, physical check separate

Release `1.0.71` is installed locally with status
`INSTALLED_VERIFIED_PHYSICAL_PENDING`. The loaded runtime matches candidate
`2ee1479cfc84c40ce3e8673e573d2dbfc42226f0e4dbdeb7cc310f6f1a63fcc5`:
IME PID `4152536`, daemon PID `4152503`, L3 PID `4152477`, and L1.1 PID
`4152326`. The global `ibus-daemon` PID `4715` was preserved; CLI and loaded
GNOME extension both report `1.0.71`; config, input sources, and the eight immutable model/data dependency payloads were
preserved; the L1.1 executable dependency changed from the accepted 1.0.70
service hash to `db825d2f244282392fe507ebeee335ef3e66ee33fc35f6e701a688abc7845034`.

Release 1.0.71 includes the accepted repairs for terminal word-boundary
replacement, clean repeated-word handling, candidate comparison, and
within-word Backspace/pending-learning feedback. It is installed runtime
authority for the verified local processes; physical keyboard acceptance is
still `PENDING`.

Evidence: fresh full identity `7585f7df394fa6d06c15181923d68b553f07f0bf788f3184a4b2baf5da7c36e7`,
acceptance summary `b08915f4bdf76e1951d25a98feb0abf4118fe308b189c3ddf41580ddf196a68d`,
installation receipt `8e6124c06c0f41543be7c077d84964cfcb3f919cd73d84ddd4a2cc19a665b9be`,
and post-install runtime receipt `fa9a979c2c1eefd367cbe7c7f24ebe15d655d0e85db0558fb32938449122d3a8`.
Changed/full gates passed `2756/2756` each; client checks passed
`17/17`; fixed89 stayed exact-output identical across three profiles
(`267/267`, including the known wrong-output counts `1/4/5`); diagnostic18 is
`14/18` with `7/11` dirty restored and `7/7` clean preserved.

The install cut over input journals at `2026-09-11T09:54:01.913546Z`: exactly
10 old journal files were deleted without archive, and L3 observed the new
empty usage journal alone (`source_offset=0`, `source_tail_hashes=[]`) before
daemon/IME started. The second install attempt observed L1.1
`socket_refused -> warming -> ready` and then ran one strict guard; there were
no command failures. The first failed attempt and recovery receipts remain a
truthful incident record, not the final state.

Exact graph completion is established only by
`/home/ubu/.cache/lay/development/release-1.0.71-20260911-hiu22jcy/final-document-graph/fetch-receipt.json`
with `PASS`. Exact publication refs are established only by
`/home/ubu/.cache/lay/development/release-1.0.71-20260911-hiu22jcy/publication.json`.

Current owning release evidence:
[release 1.0.71](docs/release-1.0.71-preflight-2026-09-11.md).

# Historical: poor-input mechanics installed before 1.0.71 release

2026-09-11T01:31:13Z local install completed with status
`INSTALLED_VERIFIED_PHYSICAL_PENDING`. Candidate IME
`aec4f55310e7aded386d037176070e709523e3d4a66a231c7fdd951a24920c99`
is loaded as PID `1652608` under preserved global IBus PID `4715`.
Changed verified consumers: `lay-daemon` PID `1652602`
(`be231d78fd01`), `lay-nanda-wave-train`/L3 online PID `1652603`
(`d2df74855c83`), and `lay-ibus-engine`. Preserved L1.1 PID is
`2729596`; source, config, extension, immutable dependencies, input sources,
and all release artifact identities were verified.

The accepted repair has three mechanisms: L2 absolute phase competition keeps
actual margins, generic L4 phase-bank availability is advisory until exact
transition evidence exists, and within-word Backspace keeps raw retention plus
bounded pending-learning feedback aligned with the Lay target. Private acceptance:
composed source proof `2052/2052`, changed/full both `2756/2756` with zero
failures, fixed89 `267/267` exact-output no-regression across three profiles,
diagnostic18 `10/18 -> 14/18`, and native client matrix `17/17`.

Installation receipt:
`~/.cache/lay/development/poor-input-20260911-xoi17mif/final-mechanics-v1/installation-poor-input.json`
sha256 `706022c30e9f9bd3423db36340be6dcfe17ffdb41f1a4f87241ae88ae6af4551`.
Runtime receipt:
`~/.cache/lay/development/poor-input-20260911-xoi17mif/post-install-runtime.json`
sha256 `858931b908302163a85a92372ad3fb70b96df821870941383e2d412b87be55bb`.
Backup: `/home/ubu/.local/state/lay/release-backups/poor-input-20260911-bgoa6qf_`.
Installation review: `9/10`, H0/M0/L0. Physical keyboard acceptance remains
`NOT_TESTED`; no general quality, RSS, latency, heldout, or physical PASS is
claimed. Final document-graph verification is recorded at
`~/.cache/lay/development/poor-input-20260911-xoi17mif/final-mechanics-v1/final-graph/fetch-receipt.json`;
only an existing `PASS` receipt there establishes graph completion. Full facts and limits are in
[poor-input authority](docs/poor-input-authority-2026-09-11.md).

# Historical: terminal IME commit repair installed, physical check was pending

2026-09-10T13:17:32.727160+00:00 локально установлен IME `4bfe47fa3db1`
(PID 2182844); общий IBus4715 сохранён. Для legacy terminal обычные
буквы идут через native input, замена — одним commit. Пробел и курсор
сохранены в трёх классах длины; видимая подсказка проверена отдельно.
Changed/full: по 2730/2730; actual-client: 17 отдельных случаев;
финальное независимое ревью 9/10, H0/M0/L0. Прежний cold-hint отказ
воспроизведён на обоих бинарниках и отделён от принятого post-ready сценария.

Один owning document: [terminal commit order](docs/ime-terminal-commit-order-2026-09-10.md).
Установка и откат:
`~/.cache/lay/development/ime-replacement-cursor-20260910-7xc45bzi/installation-terminal-ime.json`.
Это локальный патч с меткой 1.0.70; новый тег и публикация не выполнялись.
Не повторять установку и пройденные проверки. Следующий шаг — пользовательский
ввод в настоящем окне; GNOME/Kitty physical acceptance пока PENDING.

# Historical: Lay 1.0.70 GitHub fixes installed runtime

Исправлен #42: обычные начальные `-`, `_`, `=` и `+` сохраняются в daemon
буфере вместе с буквами и участвуют в точном replay. Для #43/#44 установщик
собирает десять устанавливаемых целей с видимым прогрессом, сохраняет журнал
и код ошибки/отмены; добавлена пропущенная зависимость `jq`.
Changed/full: по 2726/2726; installer: 8/8; final static review: 9/10 H0/M0/L0.
Проверенный комплект установлен 2026-09-10 01:07 UTC; изменился только daemon
`133a1f79e57b`, девять остальных бинарников идентичны первой 1.0.70.
Четыре загруженных процесса проверены, общий IBus4715 сохранён.
Установка: `~/.cache/lay/development/run-j4e7w_0n/installation-1.0.70.json`.

Публичная поставка: `public/main`, тег `v1.0.70`; private источник:
`origin/codex/cleanup-20260908`. Точные commit/ref и фактическое завершение
публикации проверять по `~/.cache/lay/development/github-issues-70-3ps08xs0/publication.json`.
Полная область, ARM compatibility и ограничения:
[owning public-issues document](docs/public-issues-42-44-release-1.0.70.md).
В исходных Ubuntu логах причина остановки не установлена; повторная установка
на устройствах авторов #43/#44 UNOBSERVED. Физическая клавиатура PENDING;
разрешение публикации не считается её подтверждением. Общий TD-123 и «зуын»
→ «push» OPEN. Не повторять принятую сборку/установку и не включать счётчики.

# Предыдущая установка 1.0.70: физическая проверка автопереворота ожидалась

Исправлена потеря подтверждённого начала слова после удаления наблюдённого
пробела и повторного набора. Причинный native контроль: 1.0.69 3/4 → 1.0.70 4/4;
неизвестный текст остаётся без полномочия автозамены.
Полные тесты 2723/2723 PASS; три fixed89 профиля без регрессий; native 13,
first-word GUI 7/7 и quiet legacy 5/5 PASS. Финальный review 9/10, H0/M0/L0.
Все десять бинарников, четыре загруженных процесса, CLI и расширение сверены.
Общий IBus PID 4715 сохранён. Установка: `~/.cache/lay/development/run-p4d_z81t/installation-1.0.70.json`.
Физическая проверка запрошена 2026-09-09 21:42 UTC и PENDING; commit/push
1.0.70 ещё не выполнены. Не повторять установку или неизменённые проверки.
«згыр» → «push» — чистая смена раскладки; «зуын» даёт «pesy» и требует
дополнительного восстановления двух букв. Этот отказ и общий TD-123 OPEN.
Счётчики не возобновлять. Все классы, ресурсы, ограничения и receipts:
[owning IME architecture](docs/ime-daemon-route-map-2026-06-20.md#retained-boundary-final-validation-and-installation-2026-09-10).

# Lay 1.0.69: подсказки приняты, публикация разрешена

Пользователь сообщил «Push отлично», затем открыл следующую проблему:
автопереворот не срабатывает, пример «зуын» → «push». Публикуется уже
установленное исправление подсказки первого слова и явного дополнения Tab.
Это пользовательская приёмка данного результата; приложение последней
проверки не уточнено. Новый отказ автопереворота и общий TD-123 остаются OPEN.

Полные проверки: 2717/2717; final-byte native первого слова: 7/7;
три профиля по 89 фиксированным сценариям без регрессий; ещё 13/13 native
контролей. Source review 9/10 H0/M0; installer/provenance review 10/10 H0/M0/L0.
Все десять бинарников установлены; CLI, расширение и четыре загруженных
процесса проверены как 1.0.69. Общий IBus PID 4715 сохранён.
Установка: `~/.cache/lay/development/run-btt4ilfz/installation-1.0.69.json`.
Авторизация: `~/.cache/lay/development/run-btt4ilfz/physical-acceptance-1.0.69.json`.
Точный commit и remote ref проверяются в
`~/.cache/lay/development/run-btt4ilfz/publication-1.0.69/`.
Не повторять установку и неизменённые проверки.

Прежний intermittent-hint report и старая библиотека Kitty сохранены как
ограничение доказательств в owning IME architecture. Нельзя приписать все
старые жалобы одному приложению по focused-window readback. Старые вкладки
и общий IBus не перезапускались; оба отменённых счётчика остаются выключены.
Новая диагностика автопереворота:
`~/.cache/lay/development/auto-layout-push-k9vjrshc/`.

# WeChat: пользователь подтвердил работу и разрешил публикацию

2026-09-09 пользователь сообщил «Теперь в окнах Wechat не работает ничего!».
WeChat PID 3734978, общий IBus PID 4715 и Lay 1.0.68 продолжают работать;
IME в `/proc` имеет прежний SHA `2a2df132`. Активны источники `lay-ime-us/ru`.
Новый private native контроль воспроизвёл общий отказ GUI-маршрута: `про`
в пустом поле не показывает подсказку, ` про` после наблюдённого пробела
показывает, ввод в середине существующего слова остаётся без подсказки.
Три сценария завершены: один ожидаемый отказ, два контроля PASS; все private
процессы убраны. Это не физическая приёмка WeChat и не его полный диагноз.
Причинная граница — `UnknownStart` до расчёта кандидатов; снимок surrounding
text не подтверждает начало слова в текущем reducer. Проектирование открыто:
IBus передаёт относительный фрагмент, а не отдельное доказательство начала
документа, поэтому одного `cursor=0` для общей выдачи полномочий недостаточно.
Код и установленный runtime не менялись. Затем пользователь ответил
«да заработало пушь»: работа WeChat подтверждена пользователем, публикация
разрешена. Это не отменяет отдельный воспроизведённый отказ первого слова
и не является измерением всех функций. Счётчики не возобновлять.
Доказательства: `~/.cache/lay/development/wechat-ime-no-functions-8isocthh/`;
подробности и варианты — в
[owning IME architecture](docs/ime-daemon-route-map-2026-06-20.md#wechat-report-and-first-word-admission-2026-09-09).

# Kitty: подсказки при наборе приняты пользователем

2026-09-09 после публикации 1.0.68 пользователь уточнил: на «прове» нет
видимой подсказки, а Up/Down её показывает. Проверен конечный клиент Kitty
0.48.2: live worker вернул 10 кандидатов за 2.6 ms и отправил «рка» до стрелок.
Причинный контроль исходного обработчика Kitty воспроизвёл потерю preedit
после commit: 4 RED из 13, остальные 9 контролей PASS. Минимальный патч
порядка обработки и сброса существующего cache даёт 13/13 PASS, ASan/UBSan
чистые; независимый review 9/10, H0/M0/L0. Прямого захвата исходного пакета
нет. Модуль собран на закреплённых в Kitty
зависимостях Wayland 1.24.0 / protocols 1.45; ABI и загрузка проверены для
установленного оригинала, rebuilt baseline и candidate. Открыто отдельное
окно «Lay: проверка подсказок», PID 1891726, загружен candidate `9cbbe9f7`.
На вопрос о наборе «про» / «прове» без стрелок пользователь ответил
«да пушь»: физическая проверка отдельного окна принята, публикация разрешена.
Receipt: `~/.cache/lay/development/ime-prove-no-hint-qggkm5wl/physical-acceptance.json`.
После повторного сообщения «Tab дополняет, но подсказки нет» live focus
указал прежний Kitty PID 272166 со старым модулем. 2026-09-09 12:20 UTC
принятые байты `9cbbe9f7` атомарно установлены в основной Kitty; обычный
launcher запустил PID 3048061 с теми же SHA и новым inode. Окно называется
«Kitty: исправление установлено». Прежний PID сохраняет старый mapped inode
до перезапуска; он и глобальный IBus не перезапускались.
Receipt: `~/.cache/lay/development/ime-prove-no-hint-qggkm5wl/normal-installation.json`.
Пользователь подтвердил «в новом окне работает все отлично!»; физическая
приёмка основной установки записана в `normal-physical-acceptance.json`
в той же папке. Текущий сценарий Kitty закрыт; старые окна требуют перезапуска.
Backup: `~/.local/state/lay/compat-backups/kitty-0.48.2-preedit-2lmgtmmm/`.
Патч и доказательства уже опубликованы commit `57e91b65`; версия Lay 1.0.68
и его runtime не менялись. Owning record:
[IME architecture](docs/ime-daemon-route-map-2026-06-20.md#missing-typed-suggestions-restored-by-arrows-2026-09-09).
Счётчики не возобновлять. Приёмка ограничена этим сценарием; общий TD-123
остаётся OPEN. Ветка публикации `origin/codex/cleanup-20260908`; точный commit
и remote-ref receipt: `~/.cache/lay/development/ime-prove-no-hint-qggkm5wl/publication/`.

# Lay 1.0.68 принят пользователем

Текущий этап 2026-09-09 07:19 UTC: `DELIVERY_ACCEPTED`; релизный milestone DONE.
Исправлена воспроизведённая зависимость IME-подсказки от необязательного
уведомления курсора при старом протоколе фокуса. Изменена одна runtime-функция;
KnownStart, идентичность слова/поля, сроки публикации и verifier сохранены.

Установлены десять проверенных бинарников 1.0.68. IME `2a2df132`, PID 536035
запущен сохранённым IBus PID 4715; L1.1 `570f35b9` PID 535803, daemon
`b17a4fc7` PID 536029, L3 watcher `964f2265` PID 536030. SHA файлов и /proc,
CLI и загруженное расширение 1.0.68 проверены; модели, config и источники ввода
сохранены. Receipt:
`~/.cache/lay/development/run-hvrxwcxk/installation-1.0.68.json`.
Backup: `~/.local/state/lay/release-backups/1.0.68-td123-4h4357ne/`.

2711 correctness/package и обязательные release gates завершены; отдельный
exit75 по лимиту Cargo сохранён, продолжение использовало те же проверенные
исходники. Fixed89 во всех трёх профилях без регрессий к 1.0.67. Native13 PASS;
legacy-focus hints 8/8, обе опечатки исправлены по 8/8, clean24/24 при 80 ms.
Narrow terminal hints PASS, а Space correction в этом private harness остаётся
3/5 как у baseline: физического daemon в нём нет. Это не приёмка всего desktop.
Review runtime 9/10 H0/M0/L0; три замечания второго review к установщику
исправлены, семь проверок его failure paths прошли. Полные факты и ограничения:
[owning IME architecture](docs/ime-daemon-route-map-2026-06-20.md#rare-ime-suggestions-after-the-spontaneous-input-report-2026-09-09).

После проверки обычным набором пользователь подтвердил «работает !» и разрешил
push. Это приёмка текущего IME-сценария; общий TD-123 quality остаётся OPEN.
Не повторять установку или неизменённые проверки. Публикационная ветка:
`origin/codex/cleanup-20260908`; исходный main сохранён. Точный результат push
проверяется удалённой ссылкой и receipt в
`~/.cache/lay/development/run-hvrxwcxk/publication-1.0.68/`.
В этой же папке `candidate-counts.json`: готовые списки на 31 расчёте содержат
0–12 вариантов; лимиты — 12 словесных + до 6 фразовых в Experimental,
на экране один выбранный. Полное внутреннее поле по буквам не измерено.
Все build/test/graph update только remote.

Пользователь сообщил, что точки прекратились, и отменил счётчик; оба наблюдателя
остановлены, MainPID=0. Не запускать их снова. Источник точек не объявлен
исправленным. Текущий маршрут — подсказки и автоисправление последней версии.

## История до исправления подсказок 1.0.68

Актуальное поручение 2026-09-09: исправлять последнюю 1.0.67. Пользователь
сообщил старый сбой IME в Kitty, отдельных окнах терминала, Tor Browser и
WeChat; точки появляются по одной без нажатий, после удаления возвращаются
с паузой. Это не установленная регрессия последнего релиза. Ошибочный откат
на 1.0.66 исправлен: возвращены те же проверенные байты 1.0.67 без сборки.
Receipt повторной установки 1.0.67:
`~/.cache/lay/development/ime-cross-app-failure-_ul2eo18/reinstallation-1.0.67.json`.
IME PID 3787313, L1.1 PID 3787090, daemon PID 3787309, L3 watcher PID 3787310
на момент повторной установки; глобальный IBus PID 4715 сохранён.

После сообщения пользователя «точки есть» захвачен эпизод 01:45:50–01:45:53 UTC:
96 нажатий точки пришли от IBus-соединения WeChat `:1.1961`, PID 3734978,
`/opt/wechat/wechat`; на трёх наблюдавшихся устройствах таких нажатий нет.
Принадлежность соединения проверена независимыми Peer.Ping/FD probes и парой
Unix-сокетов. Медианный интервал 30.2 ms совпадает с системным повтором 30 ms.
Источник запросов в IBus установлен, первопричина выше по цепочке ещё UNKNOWN.
В первой записи пропущен четвёртый клавиатурный вход: `2.4G Mouse`, event5,
USB 1ea7:0066. Поэтому физическое происхождение пока не исключено. Исправленный
наблюдатель v5 выбирает все устройства, объявляющие клавиши 52/53, из текущего
инвентаря, включая ресивер. Дальше различить устройство → X11 → WeChat → IBus.
Исправление не заявлено. Следующая пассивная запись охватывает XInput и IBus
с идентичностью поля/приложения при начале повтора; пользователь уже получил
просьбу оставить фокус в текстовом поле WeChat. Повторять откат, установку
или неизменённые полные проверки не надо.
[Доказательства и границы](docs/ime-daemon-route-map-2026-06-20.md#cross-app-spontaneous-input-2026-09-09).
Физическая приёмка и DONE/commit/push остаются открыты.

## Завершённая проверка установки до сообщения о старом сбое

Текущий этап на 2026-09-09: `INSTALLED_VERIFIED_PHYSICAL_PENDING`.
Установлены те же 10 проверенных бинарников 1.0.67; IME `13db8623`,
L1.1 service `db825d2f`. SHA файлов и /proc четырёх процессов совпали;
CLI и загруженное расширение показывают 1.0.67. Глобальный IBus PID 4715,
восемь модельных зависимостей, config и список источников ввода сохранены.
Backup: /home/ubu/.local/state/lay/release-backups/1.0.67-td123-1p1e47xu/.

Все 2711 correctness/package, lint и обязательные build gates прошли.
Исходный full-вызов и продолжение после исправления только byte-location
metadata сохранены раздельно. Fresh combined review: 9/10, H0/M0/L0.
L1 Gate C PASS: 260000 damaged + 852582 clean, точное качество 13 классов.
L2: 2600 случаев, качество baseline; формальный 5 ms FAIL при 5.276 ms.
Отдельная performance lane: 10/11 PASS, cold-prefix 301.738 ms > 50 ms FAIL.
Fixed89 во всех трёх safety profiles без регрессий; Experimental dirty
19/47 → 20/47, clean 39/42 → 40/42. Native13 PASS. В quiet 80 ms серии
обе ошибки исправлены по 8/8, clean 24/24 с конечным IME и сервисом L1.1.
Это ограниченный результат, а не общий quality PASS для любых слов.

Два вопроса о физическом наборе и Double Shift были заданы до сообщения о сбое.
Не повторять установку, сборку или неизменившиеся проверки. После ответа
выполнить соответствующую приёмку; общий TD-123, DONE/commit/push пока OPEN.
Остаточные механизмы качества описаны; переобучение не выбрано.
Точные receipts: ~/.cache/lay/development/run-o0dqrl5c/ и
~/.cache/lay/development/autocorrect-live-ojoasco5/; remote run-tovK3y.
Полная запись установки и границ доказательства — в последних разделах
[owning architecture](docs/l2-l11-canonical-architecture.md).
На переходах показывать дерево с `✓` и `← СЕЙЧАС` — указание пользователя.

Ветка: codex/cleanup-20260908. Исходный snapshot-коммит:
cb40ef29f6c78c97757dd0059c0ba798cb1f0789.

Пользователь поручил очистку новой копии, ориентир до80%, и отдельно потребовал
сохранить текущие результаты автокоррекции и продолжить их после очистки.
Работать только здесь; исходные деревья сохранены. Ограничение очистки на
install/restart соблюдено; последующая проверенная установка относится к
отдельно порученному этапу TD-123.

Полная точка восстановления и продолжения:
/home/ubu/.local/state/lay/project-snapshots/20260908-before-cleanup/RESTORE_AND_CONTINUE.md

[План и результат очистки](docs/project-cleanup-2026-09-08.md) ·
[Восстановление истории](ARCHIVE.md) ·
[Автокоррекция: сохранённое состояние](tech_debt/CONTINUE.md).

Проверенный архив6173файлов и исходный бинарник/ключевые receipts сохранены
в том же каталоге. Это не disposable workspace/cache новой ветки.
Очистка:4 800 исходных файлов удалены,2 684/2 684 correctness/package PASS,
46/46 architecture-tooling PASS,3 installer suites PASS,review9/10,H0/M0/L0.

История TD-123, до текущей установки. Установленный995b на2026-09-08:
19/47 dirty correct,39/42 clean preserved.
Частная сборка37073948 уменьшила повторы L3,2209 тестов и126 сравнений readout
прошли, но80ms всё ещё опаздывает; fixed89 выявил36/42 clean preserved.
Она ОТКЛОНЕНА для установки. Следующая частная сборкаfa15601f согласовала
существующее правило коротких layout-целей:2211 tests PASS, review9/10,
fixed89 снова19/47 dirty и39/42 clean, все классы совпали с995b;13 client cases PASS.
Фиксированные8 native повторов при80ms: своевременное исправление0/8→7/8,
clean24/24 у обеих сборок; gate8/8 НЕ пройден. Один срыв:22.3ms очередь+
80.6ms расчёт. Это метрика доставки одной фразы, не точность на8 разных словах.
Продолжить устранение опоздания к Space и отказов готового расчёта.
Правка повторного decoder reconstruction прошла component parity и review;
частнаяeea32f44 прошла fixed89 без изменений и13 native controls. В новой
парной серии80ms она успела8/8 против5/8 уfa15601f, clean24/24 у обеих.
Ограниченный native readiness gate пройден; теперь отказы готового расчёта.
Установки нет; общий TD-123, модельное качество и физическая приёмка открыты.
Ready-refusal audit завершён:23/25 целей удержаны,21 без L3/exact-L4 authority;
один проверенный lexical Winner теряет разрешение на общей L4 ambiguity.
Ограниченный перенос frame capability реализован после анализа вариантов9/8/4.
RED воспроизвёл общий отказ, итоговый scoped GREEN2 221/2 221, code review9/10,
H0/M0/L0. Частная7a58535c прошла весь fixed89:19/47→20/47 dirty,
clean39/42 без изменений, новых wrong outputs нет;13 native controls PASS,
8/8 своевременных исправлений при80ms, наблюдаемый RSS до352.53MiB.
Общий quality/release/physical PASS ещё не получен. Остальные21 missing-authority
случай открыты. Последствия и L1.1 наблюдения43 токенов — в owning architecture.
Полный результат и доказательства — в tech_debt/CONTINUE.md. Физический обычный
набор PENDING. Очистка файлов не является приростом качества модели.

Следующий общий дефект — raw-capacity L2 — исправлен в частной1bceb869.
Все оставшиеся final L2 frames теперь полные; ограничения поиска и74/4 сохранены.
Fixed89: dirty20/47 без изменения, clean40/42, новых ошибок нет. Scoped2227 PASS,
review9/10, native13 PASS; новая полная серия80ms8/8 у обеих версий. Первую
неполную серию из-за ошибки имени варианта в private harness сохранили отдельно.
Дальше: неподдерживаемая геометрия и независимый контекст для complete ties.
Точные границы, все проценты и receipts — конец owning architecture.

Все builds/tests/architecture refresh удалённо, dedicated-20cpu/Cargo guard.
Сохраняются требования AGENTS.md о последствиях, независимом review,
SafetyGate/verifier, единственном владельце Double Shift и fixed proof.
