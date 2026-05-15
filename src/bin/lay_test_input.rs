//! lay-test-input — test harness для автоматической проверки lay-daemon.
//!
//! Создаёт виртуальную клавиатуру через uinput, печатает её путь в stdout
//! (для запуска `lay-daemon --device <path>`), затем по сигналу или таймеру
//! эмулирует тестовые сценарии.
//!
//! Использование:
//!   lay-test-input scenario1   — печатает "ghbvth" + двойной Shift
//!   lay-test-input ghbdtn_shift — печатает "ghbdtn" + двойной Shift
//!   lay-test-input ghbdtn_enter — печатает "ghbdtn" + двойной Shift + Enter
//!   lay-test-input ghbdtn_fast_lshift_enter — "ghbdtn" + очень быстрый двойной LShift + Enter
//!   lay-test-input ghbdtn_extra_lshift_enter — "ghbdtn" + лишние быстрые LShift-тапы + Enter
//!   lay-test-input ru_p_enter — печатает "п" в RU + Enter
//!   lay-test-input g_to_ru_enter — печатает "g" + двойной Shift + Enter
//!   lay-test-input ru_p_to_g_enter — печатает "п" + двойной Shift + Enter
//!   lay-test-input ru_p_toggle2_enter — печатает "п" + двойной Shift × 2 + Enter
//!   lay-test-input slovo_ru_to_us_fast_lshift_enter — печатает "слово" + быстрый двойной Shift + Enter
//!   lay-test-input slovo_ru_to_us_extra_lshift_enter — печатает "слово" + лишние быстрые Shift-тапы + Enter
//!   lay-test-input ctrl_plus_ghbdtn_enter — жмёт Ctrl+Shift+=, затем "ghbdtn" + двойной Shift + Enter
//!   lay-test-input dhtvz_toggle_enter — печатает "dhtvz" + двойной Shift × 2 + Enter
//!   lay-test-input dhtvz_toggle3_enter — печатает "dhtvz" + двойной Shift × 3 + Enter
//!   lay-test-input n_teper_mixed_enter — печатает "Nеперь" + двойной Shift + Enter
//!   lay-test-input scenario2   — печатает "привет" в RU + двойной Shift
//!   lay-test-input mixed_word — печатает "ghи" + двойной Shift + Enter
//!   lay-test-input three_words — печатает "ghbdtn rfr ltkf" + двойной Shift + Enter
//!   lay-test-input good_ntrcn_enter — печатает "good ntrcn" + двойной Shift + Enter
//!   lay-test-input proverka_ntrcn_enter — печатает "проверка ntrcn" + двойной Shift + Enter
//!   lay-test-input good_vshgidu_enter — печатает "good Вщгиду" + двойной Shift + Enter
//!   lay-test-input good_text_enter — печатает "пщщв ntrcn" + двойной Shift + Enter
//!   lay-test-input wifi_ye_enter — печатает "wi-fi ye" + двойной Shift + Enter
//!   lay-test-input auto_switch_words_enter — печатает "njkmrj yt hf,jnftn" через пробелы + Enter
//!   lay-test-input preparatov_typo_enter — печатает "перпаратов" + Space + Enter
//!   lay-test-input no_ne_ty_enter — печатает "но не ты" с паузами после пробелов + Enter
//!   lay-test-input vyvodim_dva_enter — печатает "dsdjlbv ldf" + двойной Shift + Enter
//!   lay-test-input mixed_coke_enter — печатает "слово кjrf-rjke" + двойной Shift + Enter
//!   lay-test-input mixed_coke_toggle3_enter — печатает "слово кjrf-rjke" + двойной Shift × 3 + Enter
//!   lay-test-input параллелепипед_long — длинное нижнерегистровое слово + Shift + Enter
//!   lay-test-input x11-diagnostics — печатает X11/backend diagnostics report
//!   lay-test-input x11-report — печатает GitHub-ready X11 validation report
//!   lay-test-input list        — только создаёт kbd и держит, печатает путь

use evdev::{uinput::VirtualDevice, AttributeSet, EventType, InputEvent, KeyCode};
use lay::config::LayConfig;
use lay::desktop::{parse_setxkbmap_layout, resolve_layout_backend};
use std::env;
use std::io::Write;
use std::process::{Command, Stdio};
use std::thread::sleep;
use std::time::Duration;

fn main() -> std::io::Result<()> {
    let scenario = env::args().nth(1).unwrap_or_else(|| "list".to_string());
    if matches!(scenario.as_str(), "x11-diagnostics" | "diagnose-x11") {
        print_x11_diagnostics();
        return Ok(());
    }
    if matches!(scenario.as_str(), "x11-report" | "report-x11") {
        print_x11_report();
        return Ok(());
    }

    let mut keys = AttributeSet::new();
    let all = [
        KeyCode::KEY_A,
        KeyCode::KEY_B,
        KeyCode::KEY_C,
        KeyCode::KEY_D,
        KeyCode::KEY_E,
        KeyCode::KEY_F,
        KeyCode::KEY_G,
        KeyCode::KEY_H,
        KeyCode::KEY_I,
        KeyCode::KEY_J,
        KeyCode::KEY_K,
        KeyCode::KEY_L,
        KeyCode::KEY_M,
        KeyCode::KEY_N,
        KeyCode::KEY_O,
        KeyCode::KEY_P,
        KeyCode::KEY_Q,
        KeyCode::KEY_R,
        KeyCode::KEY_S,
        KeyCode::KEY_T,
        KeyCode::KEY_U,
        KeyCode::KEY_V,
        KeyCode::KEY_W,
        KeyCode::KEY_X,
        KeyCode::KEY_Y,
        KeyCode::KEY_Z,
        KeyCode::KEY_SPACE,
        KeyCode::KEY_LEFTSHIFT,
        KeyCode::KEY_LEFTCTRL,
        KeyCode::KEY_BACKSPACE,
        KeyCode::KEY_ENTER,
        KeyCode::KEY_MINUS,
        KeyCode::KEY_EQUAL,
        KeyCode::KEY_COMMA,
    ];
    for k in all {
        keys.insert(k);
    }

    let mut dev = VirtualDevice::builder()?
        .name("lay-test-virtual-keyboard")
        .with_keys(&keys)?
        .build()?;

    if let Some(path) = dev.enumerate_dev_nodes_blocking()?.next().transpose()? {
        println!("{}", path.display());
        std::io::stdout().flush()?;
    }

    eprintln!("[test] virtual keyboard создана");
    let start_delay_ms = env::var("LAY_TEST_START_DELAY_MS")
        .ok()
        .and_then(|value| value.parse::<u64>().ok())
        .unwrap_or(3000);
    sleep(Duration::from_millis(start_delay_ms)); // дать daemon открыть устройство

    match scenario.as_str() {
        "ghbvth_shift" => {
            // печатает "ghbvth" потом двойной левый Shift
            for k in [
                KeyCode::KEY_G,
                KeyCode::KEY_H,
                KeyCode::KEY_B,
                KeyCode::KEY_V,
                KeyCode::KEY_T,
                KeyCode::KEY_H,
            ] {
                tap(&mut dev, k.code())?;
                sleep(Duration::from_millis(50));
            }
            sleep(Duration::from_millis(200));
            // двойной shift
            tap(&mut dev, KeyCode::KEY_LEFTSHIFT.code())?;
            sleep(Duration::from_millis(80));
            tap(&mut dev, KeyCode::KEY_LEFTSHIFT.code())?;
            sleep(Duration::from_millis(500));
            eprintln!("[test] сценарий ghbvth_shift отправлен");
        }
        "ghbdtn_shift"
        | "ghbdtn_enter"
        | "ghbdtn_fast_lshift_enter"
        | "ghbdtn_extra_lshift_enter" => {
            // печатает "ghbdtn" потом двойной левый Shift
            for k in [
                KeyCode::KEY_G,
                KeyCode::KEY_H,
                KeyCode::KEY_B,
                KeyCode::KEY_D,
                KeyCode::KEY_T,
                KeyCode::KEY_N,
            ] {
                tap(&mut dev, k.code())?;
                sleep(Duration::from_millis(50));
            }
            sleep(Duration::from_millis(200));
            if scenario == "ghbdtn_fast_lshift_enter" {
                double_shift_fast(&mut dev, 800)?;
            } else if scenario == "ghbdtn_extra_lshift_enter" {
                extra_fast_lshift_taps(&mut dev, 800)?;
            } else {
                tap(&mut dev, KeyCode::KEY_LEFTSHIFT.code())?;
                sleep(Duration::from_millis(80));
                tap(&mut dev, KeyCode::KEY_LEFTSHIFT.code())?;
                sleep(Duration::from_millis(800));
            }
            if matches!(
                scenario.as_str(),
                "ghbdtn_enter" | "ghbdtn_fast_lshift_enter" | "ghbdtn_extra_lshift_enter"
            ) {
                tap(&mut dev, KeyCode::KEY_ENTER.code())?;
            }
            eprintln!("[test] сценарий {scenario} отправлен");
        }
        "ru_p_enter" => {
            activate_layout("ru");
            sleep(Duration::from_millis(250));
            tap(&mut dev, KeyCode::KEY_G.code())?;
            sleep(Duration::from_millis(250));
            tap(&mut dev, KeyCode::KEY_ENTER.code())?;
            eprintln!("[test] сценарий ru_p_enter отправлен");
        }
        "g_to_ru_enter" => {
            activate_layout("us");
            sleep(Duration::from_millis(250));
            tap(&mut dev, KeyCode::KEY_G.code())?;
            double_shift_enter(&mut dev, 900)?;
            eprintln!("[test] сценарий g_to_ru_enter отправлен");
        }
        "ru_p_to_g_enter" | "ru_p_toggle2_enter" => {
            activate_layout("ru");
            sleep(Duration::from_millis(250));
            tap(&mut dev, KeyCode::KEY_G.code())?;
            if scenario == "ru_p_toggle2_enter" {
                double_shift(&mut dev, 900)?;
                double_shift_enter(&mut dev, 900)?;
            } else {
                double_shift_enter(&mut dev, 900)?;
            }
            eprintln!("[test] сценарий {scenario} отправлен");
        }
        "slovo_ru_to_us_fast_lshift_enter" | "slovo_ru_to_us_extra_lshift_enter" => {
            activate_layout("ru");
            sleep(Duration::from_millis(250));
            tap_keys(
                &mut dev,
                &[
                    KeyCode::KEY_C,
                    KeyCode::KEY_K,
                    KeyCode::KEY_J,
                    KeyCode::KEY_D,
                    KeyCode::KEY_J,
                ],
                35,
            )?;
            if scenario == "slovo_ru_to_us_extra_lshift_enter" {
                extra_fast_lshift_taps(&mut dev, 900)?;
            } else {
                double_shift_fast(&mut dev, 900)?;
            }
            tap(&mut dev, KeyCode::KEY_ENTER.code())?;
            eprintln!("[test] сценарий {scenario} отправлен");
        }
        "ctrl_plus_ghbdtn_enter" => {
            activate_layout("us");
            sleep(Duration::from_millis(250));
            hold_two_tap(
                &mut dev,
                KeyCode::KEY_LEFTCTRL.code(),
                KeyCode::KEY_LEFTSHIFT.code(),
                KeyCode::KEY_EQUAL.code(),
            )?;
            sleep(Duration::from_millis(180));
            tap_keys(
                &mut dev,
                &[
                    KeyCode::KEY_G,
                    KeyCode::KEY_H,
                    KeyCode::KEY_B,
                    KeyCode::KEY_D,
                    KeyCode::KEY_T,
                    KeyCode::KEY_N,
                ],
                50,
            )?;
            double_shift_enter(&mut dev, 900)?;
            eprintln!("[test] сценарий ctrl_plus_ghbdtn_enter отправлен");
        }
        "dhtvz_toggle_enter" | "dhtvz_toggle3_enter" => {
            // печатает "dhtvz" и переключает его туда-сюда несколько раз.
            for k in [
                KeyCode::KEY_D,
                KeyCode::KEY_H,
                KeyCode::KEY_T,
                KeyCode::KEY_V,
                KeyCode::KEY_Z,
            ] {
                tap(&mut dev, k.code())?;
                sleep(Duration::from_millis(50));
            }
            let toggles = if scenario == "dhtvz_toggle3_enter" {
                3
            } else {
                2
            };
            for _ in 0..toggles {
                sleep(Duration::from_millis(220));
                tap(&mut dev, KeyCode::KEY_LEFTSHIFT.code())?;
                sleep(Duration::from_millis(80));
                tap(&mut dev, KeyCode::KEY_LEFTSHIFT.code())?;
                sleep(Duration::from_millis(900));
            }
            tap(&mut dev, KeyCode::KEY_ENTER.code())?;
            eprintln!("[test] сценарий {scenario} отправлен");
        }
        "n_teper_mixed_enter" => {
            activate_layout("us");
            sleep(Duration::from_millis(250));
            hold_tap(
                &mut dev,
                KeyCode::KEY_LEFTSHIFT.code(),
                KeyCode::KEY_N.code(),
            )?;
            sleep(Duration::from_millis(120));
            activate_layout("ru");
            sleep(Duration::from_millis(250));
            for k in [
                KeyCode::KEY_T,
                KeyCode::KEY_G,
                KeyCode::KEY_T,
                KeyCode::KEY_H,
                KeyCode::KEY_M,
            ] {
                tap(&mut dev, k.code())?;
                sleep(Duration::from_millis(50));
            }
            sleep(Duration::from_millis(220));
            tap(&mut dev, KeyCode::KEY_LEFTSHIFT.code())?;
            sleep(Duration::from_millis(80));
            tap(&mut dev, KeyCode::KEY_LEFTSHIFT.code())?;
            sleep(Duration::from_millis(600));
            tap(&mut dev, KeyCode::KEY_ENTER.code())?;
            eprintln!("[test] сценарий n_teper_mixed_enter отправлен");
        }
        "привет_shift" => {
            // эти keycodes в RU дают "привет" если ibus в RU
            // KEY_G=п, KEY_H=р, KEY_B=и, KEY_D=в, KEY_T=е, KEY_M=ь(или другое)
            // используем те же физические клавиши, ВАЖНО: ibus должен быть RU
            for k in [
                KeyCode::KEY_G,
                KeyCode::KEY_H,
                KeyCode::KEY_B,
                KeyCode::KEY_D,
                KeyCode::KEY_T,
            ] {
                tap(&mut dev, k.code())?;
                sleep(Duration::from_millis(50));
            }
            sleep(Duration::from_millis(200));
            tap(&mut dev, KeyCode::KEY_LEFTSHIFT.code())?;
            sleep(Duration::from_millis(80));
            tap(&mut dev, KeyCode::KEY_LEFTSHIFT.code())?;
            sleep(Duration::from_millis(500));
            eprintln!("[test] сценарий привет_shift отправлен");
        }
        "параллелепипед_long" | "parallelepiped_long" => {
            // "паралелпипед" в RU соответствует физическим keycodes "gfhfktkgbgtl".
            // Повторяем без пробелов: это стресс-тест длинного нижнего регистра.
            let word = [
                KeyCode::KEY_G,
                KeyCode::KEY_F,
                KeyCode::KEY_H,
                KeyCode::KEY_F,
                KeyCode::KEY_K,
                KeyCode::KEY_T,
                KeyCode::KEY_K,
                KeyCode::KEY_G,
                KeyCode::KEY_B,
                KeyCode::KEY_G,
                KeyCode::KEY_T,
                KeyCode::KEY_L,
            ];
            for _ in 0..20 {
                for k in word {
                    tap(&mut dev, k.code())?;
                    sleep(Duration::from_millis(12));
                }
            }
            sleep(Duration::from_millis(160));
            tap(&mut dev, KeyCode::KEY_LEFTSHIFT.code())?;
            sleep(Duration::from_millis(80));
            tap(&mut dev, KeyCode::KEY_LEFTSHIFT.code())?;
            sleep(Duration::from_millis(5000));
            tap(&mut dev, KeyCode::KEY_ENTER.code())?;
            eprintln!("[test] сценарий параллелепипед_long отправлен");
        }
        "three_words" => {
            for k in [
                KeyCode::KEY_G,
                KeyCode::KEY_H,
                KeyCode::KEY_B,
                KeyCode::KEY_D,
                KeyCode::KEY_T,
                KeyCode::KEY_N,
                KeyCode::KEY_SPACE,
                KeyCode::KEY_R,
                KeyCode::KEY_F,
                KeyCode::KEY_R,
                KeyCode::KEY_SPACE,
                KeyCode::KEY_L,
                KeyCode::KEY_T,
                KeyCode::KEY_K,
                KeyCode::KEY_F,
            ] {
                tap(&mut dev, k.code())?;
                sleep(Duration::from_millis(35));
            }
            sleep(Duration::from_millis(180));
            tap(&mut dev, KeyCode::KEY_LEFTSHIFT.code())?;
            sleep(Duration::from_millis(80));
            tap(&mut dev, KeyCode::KEY_LEFTSHIFT.code())?;
            sleep(Duration::from_millis(1200));
            tap(&mut dev, KeyCode::KEY_ENTER.code())?;
            eprintln!("[test] сценарий three_words отправлен");
        }
        "good_ntrcn_enter" => {
            activate_layout("us");
            sleep(Duration::from_millis(250));
            tap_keys(
                &mut dev,
                &[
                    KeyCode::KEY_G,
                    KeyCode::KEY_O,
                    KeyCode::KEY_O,
                    KeyCode::KEY_D,
                    KeyCode::KEY_SPACE,
                    KeyCode::KEY_N,
                    KeyCode::KEY_T,
                    KeyCode::KEY_R,
                    KeyCode::KEY_C,
                    KeyCode::KEY_N,
                ],
                35,
            )?;
            double_shift_enter(&mut dev, 900)?;
            eprintln!("[test] сценарий good_ntrcn_enter отправлен");
        }
        "proverka_ntrcn_enter" => {
            activate_layout("ru");
            sleep(Duration::from_millis(250));
            tap_keys(
                &mut dev,
                &[
                    KeyCode::KEY_G,
                    KeyCode::KEY_H,
                    KeyCode::KEY_J,
                    KeyCode::KEY_D,
                    KeyCode::KEY_T,
                    KeyCode::KEY_H,
                    KeyCode::KEY_R,
                    KeyCode::KEY_F,
                    KeyCode::KEY_SPACE,
                ],
                35,
            )?;
            activate_layout("us");
            sleep(Duration::from_millis(250));
            tap_keys(
                &mut dev,
                &[
                    KeyCode::KEY_N,
                    KeyCode::KEY_T,
                    KeyCode::KEY_R,
                    KeyCode::KEY_C,
                    KeyCode::KEY_N,
                ],
                35,
            )?;
            double_shift_enter(&mut dev, 900)?;
            eprintln!("[test] сценарий proverka_ntrcn_enter отправлен");
        }
        "good_vshgidu_enter" => {
            activate_layout("us");
            sleep(Duration::from_millis(250));
            tap_keys(
                &mut dev,
                &[
                    KeyCode::KEY_G,
                    KeyCode::KEY_O,
                    KeyCode::KEY_O,
                    KeyCode::KEY_D,
                    KeyCode::KEY_SPACE,
                ],
                35,
            )?;
            activate_layout("ru");
            sleep(Duration::from_millis(250));
            hold_tap(
                &mut dev,
                KeyCode::KEY_LEFTSHIFT.code(),
                KeyCode::KEY_D.code(),
            )?;
            tap_keys(
                &mut dev,
                &[
                    KeyCode::KEY_O,
                    KeyCode::KEY_U,
                    KeyCode::KEY_B,
                    KeyCode::KEY_L,
                    KeyCode::KEY_E,
                ],
                35,
            )?;
            double_shift_enter(&mut dev, 900)?;
            eprintln!("[test] сценарий good_vshgidu_enter отправлен");
        }
        "good_text_enter" => {
            activate_layout("ru");
            sleep(Duration::from_millis(250));
            tap_keys(
                &mut dev,
                &[
                    KeyCode::KEY_G,
                    KeyCode::KEY_O,
                    KeyCode::KEY_O,
                    KeyCode::KEY_D,
                    KeyCode::KEY_SPACE,
                ],
                35,
            )?;
            activate_layout("us");
            sleep(Duration::from_millis(250));
            tap_keys(
                &mut dev,
                &[
                    KeyCode::KEY_N,
                    KeyCode::KEY_T,
                    KeyCode::KEY_R,
                    KeyCode::KEY_C,
                    KeyCode::KEY_N,
                ],
                35,
            )?;
            double_shift_enter(&mut dev, 900)?;
            eprintln!("[test] сценарий good_text_enter отправлен");
        }
        "wifi_ye_enter" => {
            activate_layout("us");
            sleep(Duration::from_millis(250));
            tap_keys(
                &mut dev,
                &[
                    KeyCode::KEY_W,
                    KeyCode::KEY_I,
                    KeyCode::KEY_MINUS,
                    KeyCode::KEY_F,
                    KeyCode::KEY_I,
                    KeyCode::KEY_SPACE,
                    KeyCode::KEY_Y,
                    KeyCode::KEY_E,
                ],
                35,
            )?;
            double_shift_enter(&mut dev, 900)?;
            eprintln!("[test] сценарий wifi_ye_enter отправлен");
        }
        "auto_switch_words_enter" => {
            activate_layout("us");
            sleep(Duration::from_millis(250));
            tap_keys(
                &mut dev,
                &[
                    KeyCode::KEY_N,
                    KeyCode::KEY_J,
                    KeyCode::KEY_K,
                    KeyCode::KEY_M,
                    KeyCode::KEY_R,
                    KeyCode::KEY_J,
                    KeyCode::KEY_SPACE,
                ],
                35,
            )?;
            sleep(Duration::from_millis(450));
            tap_keys(
                &mut dev,
                &[KeyCode::KEY_Y, KeyCode::KEY_T, KeyCode::KEY_SPACE],
                35,
            )?;
            sleep(Duration::from_millis(450));
            tap_keys(
                &mut dev,
                &[
                    KeyCode::KEY_H,
                    KeyCode::KEY_F,
                    KeyCode::KEY_COMMA,
                    KeyCode::KEY_J,
                    KeyCode::KEY_N,
                    KeyCode::KEY_F,
                    KeyCode::KEY_T,
                    KeyCode::KEY_N,
                    KeyCode::KEY_SPACE,
                ],
                35,
            )?;
            sleep(Duration::from_millis(650));
            tap(&mut dev, KeyCode::KEY_ENTER.code())?;
            eprintln!("[test] сценарий auto_switch_words_enter отправлен");
        }
        "preparatov_typo_enter" => {
            activate_layout("ru");
            sleep(Duration::from_millis(250));
            tap_keys(
                &mut dev,
                &[
                    KeyCode::KEY_G,
                    KeyCode::KEY_T,
                    KeyCode::KEY_H,
                    KeyCode::KEY_G,
                    KeyCode::KEY_F,
                    KeyCode::KEY_H,
                    KeyCode::KEY_F,
                    KeyCode::KEY_N,
                    KeyCode::KEY_J,
                    KeyCode::KEY_D,
                    KeyCode::KEY_SPACE,
                ],
                35,
            )?;
            sleep(Duration::from_millis(650));
            tap(&mut dev, KeyCode::KEY_ENTER.code())?;
            eprintln!("[test] сценарий preparatov_typo_enter отправлен");
        }
        "no_ne_ty_enter" => {
            activate_layout("ru");
            sleep(Duration::from_millis(250));
            tap_keys(
                &mut dev,
                &[KeyCode::KEY_Y, KeyCode::KEY_J, KeyCode::KEY_SPACE],
                35,
            )?;
            sleep(Duration::from_millis(260));
            tap_keys(
                &mut dev,
                &[KeyCode::KEY_Y, KeyCode::KEY_T, KeyCode::KEY_SPACE],
                35,
            )?;
            sleep(Duration::from_millis(260));
            tap_keys(
                &mut dev,
                &[KeyCode::KEY_N, KeyCode::KEY_S, KeyCode::KEY_SPACE],
                35,
            )?;
            sleep(Duration::from_millis(650));
            tap(&mut dev, KeyCode::KEY_ENTER.code())?;
            eprintln!("[test] сценарий no_ne_ty_enter отправлен");
        }
        "vyvodim_dva_enter" => {
            activate_layout("us");
            sleep(Duration::from_millis(250));
            tap_keys(
                &mut dev,
                &[
                    KeyCode::KEY_D,
                    KeyCode::KEY_S,
                    KeyCode::KEY_D,
                    KeyCode::KEY_J,
                    KeyCode::KEY_L,
                    KeyCode::KEY_B,
                    KeyCode::KEY_V,
                    KeyCode::KEY_SPACE,
                    KeyCode::KEY_L,
                    KeyCode::KEY_D,
                    KeyCode::KEY_F,
                ],
                35,
            )?;
            double_shift_enter(&mut dev, 900)?;
            eprintln!("[test] сценарий vyvodim_dva_enter отправлен");
        }
        "mixed_coke_enter" => {
            type_mixed_coke_tail(&mut dev)?;
            double_shift_enter(&mut dev, 900)?;
            eprintln!("[test] сценарий mixed_coke_enter отправлен");
        }
        "mixed_coke_toggle3_enter" => {
            type_mixed_coke_tail(&mut dev)?;
            double_shift(&mut dev, 900)?;
            double_shift(&mut dev, 900)?;
            double_shift_enter(&mut dev, 900)?;
            eprintln!("[test] сценарий mixed_coke_toggle3_enter отправлен");
        }
        "mixed_word" => {
            tap(&mut dev, KeyCode::KEY_G.code())?;
            sleep(Duration::from_millis(50));
            tap(&mut dev, KeyCode::KEY_H.code())?;
            sleep(Duration::from_millis(120));
            activate_layout("ru");
            sleep(Duration::from_millis(250));
            tap(&mut dev, KeyCode::KEY_B.code())?;
            sleep(Duration::from_millis(180));
            tap(&mut dev, KeyCode::KEY_LEFTSHIFT.code())?;
            sleep(Duration::from_millis(80));
            tap(&mut dev, KeyCode::KEY_LEFTSHIFT.code())?;
            sleep(Duration::from_millis(1200));
            tap(&mut dev, KeyCode::KEY_ENTER.code())?;
            eprintln!("[test] сценарий mixed_word отправлен");
        }
        "list" => {
            eprintln!("[test] держу клавиатуру открытой 60 сек, потом выхожу");
            sleep(Duration::from_secs(60));
        }
        other => {
            eprintln!("неизвестный сценарий: {other}");
            std::process::exit(1);
        }
    }

    Ok(())
}

fn type_mixed_coke_tail(dev: &mut VirtualDevice) -> std::io::Result<()> {
    activate_layout("ru");
    sleep(Duration::from_millis(250));
    tap_keys(
        dev,
        &[
            KeyCode::KEY_C,
            KeyCode::KEY_K,
            KeyCode::KEY_J,
            KeyCode::KEY_D,
            KeyCode::KEY_J,
            KeyCode::KEY_SPACE,
            KeyCode::KEY_R,
        ],
        35,
    )?;
    activate_layout("us");
    sleep(Duration::from_millis(250));
    tap_keys(
        dev,
        &[
            KeyCode::KEY_J,
            KeyCode::KEY_R,
            KeyCode::KEY_F,
            KeyCode::KEY_MINUS,
            KeyCode::KEY_R,
            KeyCode::KEY_J,
            KeyCode::KEY_K,
            KeyCode::KEY_E,
        ],
        35,
    )
}

fn activate_layout(id: &str) {
    if activate_layout_kde(id) {
        return;
    }

    let _ = Command::new("gdbus")
        .args([
            "call",
            "--session",
            "--dest",
            "org.gnome.Shell",
            "--object-path",
            "/io/github/radislabus_star/LayDaemon",
            "--method",
            "io.github.radislabus_star.LayDaemon.ActivateLayout",
            &format!("\"{id}\""),
        ])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();
    let engine = if id == "ru" {
        "xkb:ru::rus"
    } else {
        "xkb:us::eng"
    };
    let _ = Command::new("ibus")
        .args(["engine", engine])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();
}

fn print_x11_diagnostics() {
    let cfg = LayConfig::load();
    let desktop = env::var("XDG_CURRENT_DESKTOP").ok();
    let session = env::var("DESKTOP_SESSION").ok();
    let session_type = env::var("XDG_SESSION_TYPE").ok();
    let display = env::var("DISPLAY").ok();
    let auto_backend = resolve_layout_backend(
        "auto",
        desktop.as_deref(),
        session.as_deref(),
        session_type.as_deref(),
    );

    println!("lay X11 diagnostics");
    println!("version: {}", env!("CARGO_PKG_VERSION"));
    println!("configured layout_backend: {}", cfg.layout_backend);
    println!(
        "configured active backend: {}",
        cfg.active_layout_backend().label()
    );
    println!("auto-detected backend: {}", auto_backend.label());
    println!(
        "env XDG_SESSION_TYPE: {}",
        session_type.as_deref().unwrap_or("<unset>")
    );
    println!(
        "env XDG_CURRENT_DESKTOP: {}",
        desktop.as_deref().unwrap_or("<unset>")
    );
    println!(
        "env DESKTOP_SESSION: {}",
        session.as_deref().unwrap_or("<unset>")
    );
    println!("env DISPLAY: {}", display.as_deref().unwrap_or("<unset>"));
    if session_type.as_deref() != Some("x11") {
        println!("note: current session is not X11; native XKB may talk to XWayland only");
    }
    println!();

    match lay::x11_layout::ping() {
        Ok(reply) => println!("native x11rb XKB: OK ({reply})"),
        Err(err) => println!("native x11rb XKB: FAIL ({err})"),
    }
    print_command_probe("xkb-switch", &[]);
    print_command_probe("xkblayout-state", &["print", "%s"]);
    print_setxkbmap_probe();
    println!();
    println!("manual X11 smoke test:");
    println!("  1. set ~/.config/lay/config.json: layout_backend=\"x11\"");
    println!("  2. run: systemctl --user restart lay-daemon");
    println!("  3. type in any text field: ghbdtn");
    println!("  4. press double Shift; expected: привет");
}

fn print_x11_report() {
    let cfg = LayConfig::load();
    let desktop = env::var("XDG_CURRENT_DESKTOP").unwrap_or_else(|_| "<unset>".into());
    let session = env::var("DESKTOP_SESSION").unwrap_or_else(|_| "<unset>".into());
    let session_type = env::var("XDG_SESSION_TYPE").unwrap_or_else(|_| "<unset>".into());
    let display = env::var("DISPLAY").unwrap_or_else(|_| "<unset>".into());
    let auto_backend = resolve_layout_backend(
        "auto",
        Some(desktop.as_str()),
        Some(session.as_str()),
        Some(session_type.as_str()),
    );

    println!("```text");
    println!("Distro: {}", os_pretty_name());
    println!("DE/WM: {desktop}");
    println!("Session type: {session_type}");
    println!("Desktop session: {session}");
    println!("DISPLAY: {display}");
    println!("lay version: {}", env!("CARGO_PKG_VERSION"));
    println!("configured layout_backend: {}", cfg.layout_backend);
    println!(
        "configured active backend: {}",
        cfg.active_layout_backend().label()
    );
    println!("auto-detected backend: {}", auto_backend.label());
    println!("native x11rb XKB: {}", native_xkb_summary());
    println!("xkb-switch: {}", command_summary("xkb-switch", &[]));
    println!(
        "xkblayout-state: {}",
        command_summary("xkblayout-state", &["print", "%s"])
    );
    println!("setxkbmap -query: {}", setxkbmap_summary());
    println!("Input layouts: <fill from your desktop settings if different>");
    println!("What was typed: ghbdtn");
    println!("What happened: <fill after manual smoke-test>");
    println!("Expected result: привет");
    println!("```");
}

fn print_command_probe(command: &str, args: &[&str]) {
    if !command_exists(command) {
        println!("{command}: not found");
        return;
    }
    match run_command_capture(command, args) {
        Ok(output) => println!("{command}: OK ({})", one_line(&output)),
        Err(err) => println!("{command}: FAIL ({err})"),
    }
}

fn native_xkb_summary() -> String {
    match lay::x11_layout::ping() {
        Ok(reply) => format!("OK ({reply})"),
        Err(err) => format!("FAIL ({err})"),
    }
}

fn command_summary(command: &str, args: &[&str]) -> String {
    if !command_exists(command) {
        return "not found".to_string();
    }
    match run_command_capture(command, args) {
        Ok(output) => format!("OK ({})", one_line(&output)),
        Err(err) => format!("FAIL ({err})"),
    }
}

fn print_setxkbmap_probe() {
    if !command_exists("setxkbmap") {
        println!("setxkbmap -query: not found");
        return;
    }
    match run_command_capture("setxkbmap", &["-query"]) {
        Ok(output) => {
            let layout = parse_setxkbmap_layout(&output).unwrap_or_else(|| "<unparsed>".into());
            println!("setxkbmap -query: OK (layout={layout})");
        }
        Err(err) => println!("setxkbmap -query: FAIL ({err})"),
    }
}

fn setxkbmap_summary() -> String {
    if !command_exists("setxkbmap") {
        return "not found".to_string();
    }
    match run_command_capture("setxkbmap", &["-query"]) {
        Ok(output) => {
            let layout = parse_setxkbmap_layout(&output).unwrap_or_else(|| "<unparsed>".into());
            format!("OK (layout={layout})")
        }
        Err(err) => format!("FAIL ({err})"),
    }
}

fn os_pretty_name() -> String {
    let Ok(text) = std::fs::read_to_string("/etc/os-release") else {
        return "<unknown>".to_string();
    };
    text.lines()
        .find_map(|line| line.strip_prefix("PRETTY_NAME="))
        .map(|value| value.trim_matches('"').to_string())
        .unwrap_or_else(|| "<unknown>".to_string())
}

fn run_command_capture(command: &str, args: &[&str]) -> Result<String, String> {
    let output = Command::new(command)
        .args(args)
        .output()
        .map_err(|e| format!("spawn {command}: {e}"))?;
    if !output.status.success() {
        return Err(format!("exit status {}", output.status));
    }
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

fn one_line(value: &str) -> String {
    let compact = value.split_whitespace().collect::<Vec<_>>().join(" ");
    let mut chars = compact.chars();
    let head: String = chars.by_ref().take(120).collect();
    if chars.next().is_some() {
        format!("{head}...")
    } else {
        compact
    }
}

fn activate_layout_kde(id: &str) -> bool {
    let Some(qdbus) = find_qdbus_command() else {
        return false;
    };
    let Some(index) = kde_layout_index(qdbus, id) else {
        return false;
    };
    Command::new(qdbus)
        .args([
            "org.kde.keyboard",
            "/Layouts",
            "setLayout",
            &index.to_string(),
        ])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .is_ok_and(|status| status.success())
}

fn find_qdbus_command() -> Option<&'static str> {
    ["qdbus6", "qdbus-qt6", "qdbus"]
        .into_iter()
        .find(|cmd| command_exists(cmd))
}

fn command_exists(command: &str) -> bool {
    let Some(paths) = env::var_os("PATH") else {
        return false;
    };
    env::split_paths(&paths).any(|dir| dir.join(command).is_file())
}

fn kde_layout_index(qdbus: &str, id: &str) -> Option<usize> {
    let out = Command::new(qdbus)
        .args([
            "--literal",
            "org.kde.keyboard",
            "/Layouts",
            "getLayoutsList",
        ])
        .output()
        .ok()?;
    if !out.status.success() {
        return None;
    }
    let text = String::from_utf8_lossy(&out.stdout);
    parse_kde_layouts_list(&text)
        .into_iter()
        .position(|layout| layout == id)
}

fn parse_kde_layouts_list(output: &str) -> Vec<String> {
    output
        .split("[Argument: (sss)")
        .skip(1)
        .filter_map(|chunk| {
            let first = chunk.find('"')?;
            let rest = &chunk[first + 1..];
            let second = rest.find('"')?;
            Some(rest[..second].to_string())
        })
        .collect()
}

fn tap(dev: &mut VirtualDevice, code: u16) -> std::io::Result<()> {
    dev.emit(&[InputEvent::new(EventType::KEY.0, code, 1)])?;
    sleep(Duration::from_millis(10));
    dev.emit(&[InputEvent::new(EventType::KEY.0, code, 0)])?;
    Ok(())
}

fn tap_keys(dev: &mut VirtualDevice, keys: &[KeyCode], pause_ms: u64) -> std::io::Result<()> {
    for key in keys {
        tap(dev, key.code())?;
        sleep(Duration::from_millis(pause_ms));
    }
    Ok(())
}

fn double_shift_enter(dev: &mut VirtualDevice, settle_ms: u64) -> std::io::Result<()> {
    double_shift(dev, settle_ms)?;
    tap(dev, KeyCode::KEY_ENTER.code())?;
    Ok(())
}

fn double_shift(dev: &mut VirtualDevice, settle_ms: u64) -> std::io::Result<()> {
    sleep(Duration::from_millis(220));
    tap(dev, KeyCode::KEY_LEFTSHIFT.code())?;
    sleep(Duration::from_millis(80));
    tap(dev, KeyCode::KEY_LEFTSHIFT.code())?;
    sleep(Duration::from_millis(settle_ms));
    Ok(())
}

fn double_shift_fast(dev: &mut VirtualDevice, settle_ms: u64) -> std::io::Result<()> {
    tap_with_hold(dev, KeyCode::KEY_LEFTSHIFT.code(), 2)?;
    sleep(Duration::from_millis(8));
    tap_with_hold(dev, KeyCode::KEY_LEFTSHIFT.code(), 2)?;
    sleep(Duration::from_millis(settle_ms));
    Ok(())
}

fn extra_fast_lshift_taps(dev: &mut VirtualDevice, settle_ms: u64) -> std::io::Result<()> {
    for _ in 0..4 {
        tap_with_hold(dev, KeyCode::KEY_LEFTSHIFT.code(), 2)?;
        sleep(Duration::from_millis(8));
    }
    sleep(Duration::from_millis(settle_ms));
    Ok(())
}

fn tap_with_hold(dev: &mut VirtualDevice, code: u16, hold_ms: u64) -> std::io::Result<()> {
    dev.emit(&[InputEvent::new(EventType::KEY.0, code, 1)])?;
    sleep(Duration::from_millis(hold_ms));
    dev.emit(&[InputEvent::new(EventType::KEY.0, code, 0)])?;
    Ok(())
}

fn hold_tap(dev: &mut VirtualDevice, hold_code: u16, tap_code: u16) -> std::io::Result<()> {
    dev.emit(&[InputEvent::new(EventType::KEY.0, hold_code, 1)])?;
    sleep(Duration::from_millis(20));
    tap(dev, tap_code)?;
    sleep(Duration::from_millis(20));
    dev.emit(&[InputEvent::new(EventType::KEY.0, hold_code, 0)])?;
    Ok(())
}

fn hold_two_tap(
    dev: &mut VirtualDevice,
    first_hold_code: u16,
    second_hold_code: u16,
    tap_code: u16,
) -> std::io::Result<()> {
    dev.emit(&[InputEvent::new(EventType::KEY.0, first_hold_code, 1)])?;
    sleep(Duration::from_millis(10));
    dev.emit(&[InputEvent::new(EventType::KEY.0, second_hold_code, 1)])?;
    sleep(Duration::from_millis(10));
    tap(dev, tap_code)?;
    sleep(Duration::from_millis(10));
    dev.emit(&[InputEvent::new(EventType::KEY.0, second_hold_code, 0)])?;
    sleep(Duration::from_millis(10));
    dev.emit(&[InputEvent::new(EventType::KEY.0, first_hold_code, 0)])?;
    Ok(())
}
