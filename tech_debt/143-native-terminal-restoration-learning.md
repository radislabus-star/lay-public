# TD-143 — Ручной возврат слова в нативном терминале

Статус: DONE_SOURCE868/868; INSTALLED_1.0.86; physical Kitty NOT TESTED.
База: installed1.0.85, source cfcbca14; daemonSHA0d90f01dcb76…,
IME SHA7b688f9a0d18…. TD-142 source2966/2966 не доказывал нативный Kitty.

## Сценарий и первый разрыв

Пользователь: второе слово «криптой» после ручного Backspace/повторного набора
снова превращается в «криптон» на Space. Приложение — Kitty. Первое слово,
по его наблюдению, ведёт себя иначе; точная причина этого различия ещё не доказана.
Слово/предложение — только fixture, не runtime условие.

Пассивный журнал: caps9, SurroundingText=false, purpose10, hints0,
text_assistance=true; текущие клавиши идут как terminal_passthrough.
Квитанция: /home/ubu/.cache/lay/development/td143-native-restoration/native-capability-observation.json.

TD-142 вооружает pending edit-learning только из record_observed_system_outcome,
который вызывается после совпавшего SurroundingText. В нативном терминале такое
подтверждение отсутствует. Поэтому после отправки исправления траектория ручного
возврата не появляется. Локальное зеркало нельзя объявлять видимым результатом;
не ослаблять существующие SSI/selection/preedit отрицательные проверки.

## Варианты и решение до кода

- 9/10: в существующей pending траектории различать наблюдаемый результат поля
  и подтверждённое намерение пользователя из нативных клавиш. Успешная отправка
  разрешённой коррекции создаёт только временный возможный оригинал/кандидат,
  без обучения. Только последующие реальные native Backspace + новый native
  буквенный ввод + точный исходник на границе, тот же owner/public/age, дают
  существующий UserCorrection feedback. Ни отправка, ни один mirror, ни
  generic undo/import голосов не дают. Использовать прежнее CurrentWordSuppression
  и существующую usage память; новый исполнитель/доставка не нужны.
- 5/10: подавлять только ближайшую коррекцию без постоянного обучения. Не закрывает
  требование трёх возвратов в терминале.
- 2/10: перестроить доставку терминального ввода ради нового подтверждения.
  Не соответствует пользовательскому запрету и несоразмерно этому дефекту.

Выбран первый вариант. Не менять backend, физические клавиши, CommitText,
GNOME/IBus, сроки, SafetyGate, verifier, кандидаты/модели или raw shadow ownership.
Нативный ввод доказывает пользовательское намерение; он не выдаётся за client
render receipt или положительное подтверждение автоматического output.

## Минимальный план и подводные камни

1. Causal RED через успешный replace_committed_tail_with_effect_progress
   (LocalTerminalErase/CommitText) и native Backspace/letter/Space entrypoints
   на1.0.85, первое/второе слово. Прямой mock mirror не должен дать ложный PASS.
2. В существующей pending записи сохранить происхождение/прогресс native edit.
   Вооружение только после успешной разрешённой correction emission, не layout,
   zero/partial/failure; до ручной правки это не feedback и не голос.
3. Использовать существующий observer нативного буквенного ввода. Требовать
   существующие known public content, owner lease,30s и точный сохранённый хвост.
   Прогресс только из допущенных native key transitions с проверкой ожидаемого
   хвоста; нельзя удалять левее исправленного слова. Обычный повторный ввод,
   наблюдённая navigation/focus/private/unknown/command/paste, смена metadata
   или маршрута, нет новой буквы, локальная подмена mirror и generic revert
   отменяют кандидата. Начальный объём: объявленный terminal purpose10, no-SSI.
   Без SSI скрытая клиентская правка, middle-click paste или выделение могут
   остаться ненаблюдаемыми: это предел доказательства намерения, не client receipt.
4. Один подтверждённый native intent даёт одну UserCorrection causal episode;
   duplicate/reload и порог3 используют существующую проекцию. SSI-путь остаётся
   прежним. Atomic speculation не может выдать нативное доказательство.
5. Проверить source RED/GREEN и отрицательные границы, затем независимое ревью
   нового native diff максимум2прохода. Это новый найденный пользователем native
   gap после установки, а не третья переоценка прежнего TD-142 (его6/10 и7/10
   сохраняются). Все замечания/пределы сохранять явно.
6. Проверить физически на удалённом Kitty через существующий изолированный
   harness, сохранив точный текст/границы и скорость. Без выполнения не заявлять
   native PASS. При необходимости установить исправленные core bytes отдельным
   новым номером, не подменять принятую установленную85 незаметно.

Матрица прежних полных browser runs: Firefox170/27/19,Chrome179/24/13,
всего349/51/32 из432. Новый native scenario и новый input latency NOT TESTED.

## Независимое ревью плана — 7/10

Свежий read-only reviewer: `/root/td143_native_restoration_plan_review`.
Учтены все пять замечаний до runtime-изменения: настоящий terminal executor
вместо unreachable surrounding helper; ожидаемый хвост и непрерывность native
редактирования; UserCorrection вместо client-observed revert; окончательная
отмена при наблюдённых разрывах; явный предел ненаблюдаемых client edits.
Native scope исключает Atomic active/speculation; прежний deferred SSI settlement
не меняется. Оценки TD-142 не пересматриваются.

Первый helper-only RED:810/811,run-zjzcq902/RESULT.json. Он выявляет отсутствие
pending, но не воспроизводит terminal emission; не считать actual producer proof.

Actual executor RED:810PASS/1FAIL,811selected,118.5s; `/home/ubu/.cache/lay/development/run-ritauysi/RESULT.json`. Successful terminal DEL+CommitText assertions passed; pending origin was absent. Installed85 unchanged.

Source attempt813/814 reached the new native assertions, but the fixture used default uinput and ASCIIq. Corrected fixture now selects ime, supplies actual Russian keysym0x06ca and requires the shared restore helper to reach the original tail before every negative variation. This test-fixture correction is not a rollback of runtime logic.

## Code review pass1 — 7/10, real-admission RED and chosen correction

Fresh reviewer `/root/td143_native_restoration_code_review` found a real first-word
start loss which the bare engine fixture missed. Full810baseline+4new cases
were814/814PASS (`run-1rosoxv2/RESULT.json`), but this was not complete owner proof.
The new real-zbus admitted-callback/release source subset is4PASS/1FAIL of5,
`/home/ubu/.cache/lay/development/td143-source-subset-gy6il9rj/RESULT.json`: first
word yields no preference. This reproduces the review concern; not native GUI.

Keep the reducer and KnownStart mutation gates unchanged. The original
correction still must arm at a known authorized origin. Subsequent same-owner
contiguous native edits retain only preference intent while the reducer is
UnknownStart after deleting the first separator. At Space, the existing
CurrentWordSuppression preserves native input without granting edit authority.
Learning still requires KnownStart: pending intent survives that literal Space
and confirms only after normal callback settlement closes the boundary and
refreshes the real admission token. Generic completion and SSI feedback retain
their original gates; Atomic stays excluded. No synthetic KnownStart upgrade or
client-specific output path is added.

Final runtime source (815/815PASS, `/home/ubu/.cache/lay/development/run-fertw79p/RESULT.json`) preserves both admitted first/second callbacks and strict reducer states. Expanded persistence proof initially lacked an on-disk test configuration: later configuration loads reset the published test-only flags to defaults, so the spy fired but no usage events were written. The revised test writes a complete private config and binds all usage/L4 paths to its own fresh temporary directory, without runtime changes. New run pending; do not call the prior4/5 persistence failure product acceptance.

## Final native code review and persistence proof

Pass2 —8/10, no remaining material runtime defect, no third review/regrade.
The nonblocking ordinary-letter revalidation cost was removed by checking for
an existing native trace before admission revalidation; source proof includes
that ordering. Three real admitted callback origins (first and second words),
presses/releases and exact UserCorrection events reached three distinct persisted
episode IDs:5/5PASS,12.16s,
`/home/ubu/.cache/lay/development/td143-source-subset-ybvxt7ie/RESULT.json`,
archive97e1d71d43e53ff57f30308220873ce7d3e37dee82eec61b6bc7a6715a7394e2.
The test writes a complete private JSON config and binds every usage/L4 path to
its fresh directory; no live cleanup or production file access is used.

Final IME+eight affected contracts run initially stopped before executing tests
at the strict12GiB Cargo cache limit. Only disposable remote target/debug was
removed (5218MiB, total12350→7133MiB); source, test-lanes and installed artifacts
retained. The unchanged source gate was restarted; no budget/rule was weakened.
Source and exact installed release86 status will be linked to final receipts.

Final combined gate868 ran815IME+53contracts:867PASS/1contractFAIL. The monopoly guard correctly rejected direct effect-progress executor calls from test fixture files. Fixtures now call the existing replace_committed_tail wrapper, which invokes the same authorized executor and preserves its result/error conversion. Guard/allowlist and runtime execution are unchanged; no source test was weakened. New combined868 gate follows this fixture-only API correction.

Final combined source gate868/868PASS: `/home/ubu/.cache/lay/development/run-0tmq3hj2/RESULT.json`, archive7476ebb16e66367b76a6ea18360cfc390746b7ef59ae8a715326db9edde5e4d4. All740Rust hashes are frozen for the86 builder; no model/delivery/word-authority change.

## Installed86 — final scope

Установлены и загружены проверенные CLI/daemon/IME86, версия панели86.
[Переносимая сводка](evidence/2026-10-15-td143-source-installed86.json) содержит
архивы source/build, SHA,815IME+53contracts,3разных сохранённых эпизода и8/10 review.
11architecture invariants и15canon tests PASS. Все740Rust соответствуют source
proof;731не менялись, изменения в9privateIME файлах (включая fixtures).
Library,daemon,WordScope reducer,protocol и исполнители доставки byte-identical85.
Сохранены24защищённых файла,IBus/L1.1/L3,config,sources,mode; введено0тестовых
клавиш при установке. Резервные85bytes: каталог backup рядом с INSTALL.json.

Source/install scope закрыт. Физическая приёмка нового поведения в Kitty,
нативная latency и полная новая browser matrix остаются NOT TESTED / NOT MEASURED.
Исторические349/51/32/432 не повышаются по source tests. Новые модели не обучались:
изменён только перенос явного предпочтения в существующую usage память.
