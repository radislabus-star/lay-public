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
//!   lay-test-input ghbdtn_enter_autocorrect — печатает "ghbdtn" + Enter
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
//!   lay-test-input eng_ru_to_us_enter — печатает "утп" в RU + двойной Shift + Enter
//!   lay-test-input plain_layout_ashdu_space_enter — печатает "ашду" + Space + Enter
//!   lay-test-input plain_layout_cargo_space_enter — печатает "сфкпщ" + Space + Enter
//!   lay-test-input plain_layout_abkt_space_enter — печатает "abkt" + Space + Enter
//!   lay-test-input good_toggle4_enter — печатает "good" + двойной Shift × 4 + Enter
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
//!   lay-test-input worked_nj_space_enter — печатает "worked 'nj" через пробелы + Enter
//!   lay-test-input html_djn_spacing_enter — проверяет, что Space после html-автозамены не съедается
//!   lay-test-input preparatov_typo_enter — печатает "перпаратов" + Space + Enter
//!   lay-test-input no_ne_ty_enter — печатает "но не ты" с паузами после пробелов + Enter
//!   lay-test-input glued_tozhesamoe_next_enter — печатает "тожесамое склено" + Enter
//!   lay-test-input glued_tozhesamoe_pause_next_enter — печатает "тожесамое", ждёт автозамену, затем "склено" + Enter
//!   lay-test-input glued_toesamoe_next_enter — печатает "тоесамое склено" + Enter
//!   lay-test-input glued_yanebudu_next_enter — печатает "янебуду склено" + Enter
//!   lay-test-input glued_context_yanebudu_next_enter — печатает "тоже самое янебуду склено" + Enter
//!   lay-test-input glued_long_phrase_next_enter — печатает "янебудузавастожесамое склено" + Enter
//!   lay-test-input vyvodim_dva_enter — печатает "dsdjlbv ldf" + двойной Shift + Enter
//!   lay-test-input mixed_coke_enter — печатает "слово кjrf-rjke" + двойной Shift + Enter
//!   lay-test-input mixed_coke_toggle3_enter — печатает "слово кjrf-rjke" + двойной Shift × 3 + Enter
//!   lay-test-input параллелепипед_long — длинное нижнерегистровое слово + Shift + Enter
//!   lay-test-input x11-diagnostics — печатает X11/backend diagnostics report
//!   lay-test-input x11-report — печатает GitHub-ready X11 validation report
//!   lay-test-input list        — только создаёт kbd и держит, печатает путь

#[path = "lay_test_input/desktop_probe.rs"]
mod desktop_probe;
#[path = "lay_test_input/input_device.rs"]
mod input_device;
#[path = "lay_test_input/scenarios.rs"]
mod scenarios;

use desktop_probe::{print_x11_diagnostics, print_x11_report};
use input_device::build_virtual_keyboard;
use scenarios::run_scenario;
use std::env;
use std::io::Write;
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

    let mut dev = build_virtual_keyboard()?;

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

    run_scenario(&mut dev, &scenario)?;

    Ok(())
}
