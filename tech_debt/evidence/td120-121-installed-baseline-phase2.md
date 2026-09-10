# TD-120/121: выполненная private installed-byte baseline

Дата: 2026-09-05. Допуск и последствия записаны до запуска в
[phase-2 admission](td120-121-phase2-admission.md). Установленный engine1.0.65
SHA `391b3b44025461a71d6867dc92584dfa1906c1f8ae72ea610a13d2265aaacfab`.
Source baseline `cc1e2207519801ca0f9b7c6963897b55953a7751`.

## Запуск и результат

Команда: `./launch.sh` из `/home/ubu/.cache/lay/layout-phase2-private-31hE0X`.
Unit `lay-phase2-private-31hE0X.service`, invocation
`950c60476e3847c1a16ee5692f2b066c`. Один запуск, repair не потребовался.
`probe_status=COMPLETED`, выполнено **6/6** сценариев; это denominator
выполнения диагностики, **не шесть исправленных сценариев**.

| Сценарий | До handoff | После handoff | Время | Вывод |
|---|---|---|---:|---|
| new_path_us_to_ru | `l` | пусто | 3463 us | Подтверждена потеря уже наблюдённого начала |
| new_path_us_to_us | `l` | пусто | 3148 us | Дефект не зависит от нового decoder RU |
| same_path_quick_control | `l` | `l` | 3406 us | Контроль того же объекта сохраняет начало |
| same_focus_id_new_path | `l`, contextA | пусто, contextA | 3240 us | Одного одинакового FocusInId в baseline недостаточно |
| different_focus_id_new_path | `l`, contextA | пусто, contextB | 3263 us | Negative control: чужое поле не получает прежнее начало |
| manual_toggle_delete_new_token | `abc` → ManualToggleV3 `[1,true]` → Backspace×3 → `gjxbnfq` | Space оставляет `gjxbnfq ` | отдельно | Положительно записан `manual_toggle_suppressed` для нового токена |

Во всех handoff cases `VisibleTailV3` подтверждает `l` **до** перехода.
Время заметно меньше700ms: результат нельзя объяснять только истечением TTL.
Same-path сохраняет `ljv` после продолжения; new-path US сохраняет лишь `jv`.
US→RU stream равен `lом`, но здесь `ом` рождается при декодировании `j/v`,
не Space-коррекцией: Nanda/auto-replace отключены. Не смешивать это с
исторической коррекцией `jv→ом`. Общий установленный механизм — утрата prefix
между объектами; историческая доставка первой `l` остаётся UNKNOWN.

Пробы вызывают настоящие engine/bridge методы. В них нет IBus/GNOME desktop
переключения, физической доставки клавиш или доказательства downstream quality.
`VisibleTailV3` может refresh shared state; этот observer effect включён в scope.
Один процесс обслуживает шесть последовательных cases; это не шесть свежих
процессов. Создаются отдельные engine paths, exact before-state записан для
каждого. Старые focus receipts в plain-FocusIn cases — наблюдение baseline,
не новое доказательство canonical identity.

## Ресурсы и cleanup

Systemd: result success, exit0, service runtime1.635s, CPU583ms,
memory peak125.6M, swap0B. Лимиты50% CPU /768M /swap0 /Tasks32 /runtime45s.
Driver: elapsed1.463795129s; собственный private PID5 завершён и reaped,
returncode−15. После окончания MainPID0, ControlGroup пуст.
Live PID IBus4715/daemon3453123/IME3453154 сохранены.
`runtime_authority_changed=false`; production/config/binary/learning writes0.

## Артефакты и SHA-256

Private root `/home/ubu/.cache/lay/layout-phase2-private-31hE0X`:

| Файл | SHA-256 |
|---|---|
| `receipt.json` | `d75822dd10425f422b5f7a2d5b942eeb79e8f742c913446936bd740483acd82c` |
| `driver.py` | `7fea674691c6dfc2e5fa015c8d1fdf726191d91d472207e4ec78432bc2465721` |
| `launch.sh` | `fbd2fccfb6c2992fd0edbe6f6f51664bae4bab1c3faa150686e2f26bbba870a6` |
| `dbus.conf` | `e277abb3ef8fb2036c4983fab9a13bf27fee9920cd2b7dbbf3393ae75375ad57` |

Config/driver/shim повторяют phase1; изменён только новый private bus bootstrap
и runtime-dir placement. Исходная phase1 проба0/6 immutable и не переименована.
Доказан baseline класса TD-121 и lifecycle defect TD-120. Не доказаны исправление,
canonical hot-upgrade, race ordering, полный correction frame и client E2E.
