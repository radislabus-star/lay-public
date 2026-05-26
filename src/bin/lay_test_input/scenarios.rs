use super::desktop_probe::activate_layout;
use super::input_device::{
    double_shift, double_shift_enter, double_shift_fast, extra_fast_lshift_taps, hold_tap,
    hold_two_tap, tap,
};
use evdev::{uinput::VirtualDevice, KeyCode};
use std::path::Path;
use std::thread::sleep;
use std::time::Duration;

#[path = "scenarios/script.rs"]
mod script;
#[path = "scenarios/typing.rs"]
mod typing;

use script::{run_script, run_script_text};
use typing::{double_shift_manual, double_shift_manual_after, type_physical};

pub(crate) fn run_scenario(dev: &mut VirtualDevice, scenario: &str) -> std::io::Result<()> {
    if let Some(path) = scenario.strip_prefix("script:") {
        run_script(dev, Path::new(path))?;
        eprintln!("[test] script-сценарий {path} отправлен");
        return Ok(());
    }

    if let Some(script) = builtin_script(scenario) {
        run_script_text(dev, script, scenario)?;
        eprintln!("[test] сценарий {scenario} отправлен");
        return Ok(());
    }

    match scenario {
        "ghbvth_shift" => {
            type_physical(dev, "ghbvth", 50)?;
            double_shift_manual(dev, 500)?;
            eprintln!("[test] сценарий ghbvth_shift отправлен");
        }
        "ghbdtn_enter_autocorrect" => {
            activate_layout("us");
            sleep(Duration::from_millis(250));
            type_physical(dev, "ghbdtn", 50)?;
            sleep(Duration::from_millis(200));
            tap(dev, KeyCode::KEY_ENTER.code())?;
            eprintln!("[test] сценарий ghbdtn_enter_autocorrect отправлен");
        }
        "ghbdtn_shift"
        | "ghbdtn_enter"
        | "ghbdtn_fast_lshift_enter"
        | "ghbdtn_extra_lshift_enter" => {
            type_physical(dev, "ghbdtn", 50)?;
            sleep(Duration::from_millis(200));
            if scenario == "ghbdtn_fast_lshift_enter" {
                double_shift_fast(dev, 800)?;
            } else if scenario == "ghbdtn_extra_lshift_enter" {
                extra_fast_lshift_taps(dev, 800)?;
            } else {
                double_shift_manual(dev, 800)?;
            }
            if matches!(
                scenario,
                "ghbdtn_enter" | "ghbdtn_fast_lshift_enter" | "ghbdtn_extra_lshift_enter"
            ) {
                tap(dev, KeyCode::KEY_ENTER.code())?;
            }
            eprintln!("[test] сценарий {scenario} отправлен");
        }
        "ru_p_enter" => {
            activate_layout("ru");
            sleep(Duration::from_millis(250));
            tap(dev, KeyCode::KEY_G.code())?;
            sleep(Duration::from_millis(250));
            tap(dev, KeyCode::KEY_ENTER.code())?;
            eprintln!("[test] сценарий ru_p_enter отправлен");
        }
        "g_to_ru_enter" => {
            activate_layout("us");
            sleep(Duration::from_millis(250));
            tap(dev, KeyCode::KEY_G.code())?;
            double_shift_enter(dev, 900)?;
            eprintln!("[test] сценарий g_to_ru_enter отправлен");
        }
        "ru_p_to_g_enter" | "ru_p_toggle2_enter" => {
            activate_layout("ru");
            sleep(Duration::from_millis(250));
            tap(dev, KeyCode::KEY_G.code())?;
            if scenario == "ru_p_toggle2_enter" {
                double_shift(dev, 900)?;
                double_shift_enter(dev, 900)?;
            } else {
                double_shift_enter(dev, 900)?;
            }
            eprintln!("[test] сценарий {scenario} отправлен");
        }
        "slovo_ru_to_us_fast_lshift_enter" | "slovo_ru_to_us_extra_lshift_enter" => {
            activate_layout("ru");
            sleep(Duration::from_millis(250));
            type_physical(dev, "ckjdj", 35)?;
            if scenario == "slovo_ru_to_us_extra_lshift_enter" {
                extra_fast_lshift_taps(dev, 900)?;
            } else {
                double_shift_fast(dev, 900)?;
            }
            tap(dev, KeyCode::KEY_ENTER.code())?;
            eprintln!("[test] сценарий {scenario} отправлен");
        }
        "ctrl_plus_ghbdtn_enter" => {
            activate_layout("us");
            sleep(Duration::from_millis(250));
            hold_two_tap(
                dev,
                KeyCode::KEY_LEFTCTRL.code(),
                KeyCode::KEY_LEFTSHIFT.code(),
                KeyCode::KEY_EQUAL.code(),
            )?;
            sleep(Duration::from_millis(180));
            type_physical(dev, "ghbdtn", 50)?;
            double_shift_enter(dev, 900)?;
            eprintln!("[test] сценарий ctrl_plus_ghbdtn_enter отправлен");
        }
        "dhtvz_toggle_enter" | "dhtvz_toggle3_enter" => {
            type_physical(dev, "dhtvz", 50)?;
            let toggles = if scenario == "dhtvz_toggle3_enter" {
                3
            } else {
                2
            };
            for _ in 0..toggles {
                double_shift(dev, 900)?;
            }
            tap(dev, KeyCode::KEY_ENTER.code())?;
            eprintln!("[test] сценарий {scenario} отправлен");
        }
        "good_toggle4_enter" => {
            activate_layout("us");
            sleep(Duration::from_millis(250));
            type_physical(dev, "good", 50)?;
            for _ in 0..4 {
                double_shift(dev, 900)?;
            }
            tap(dev, KeyCode::KEY_ENTER.code())?;
            eprintln!("[test] сценарий good_toggle4_enter отправлен");
        }
        "eng_ru_to_us_enter" => {
            activate_layout("ru");
            sleep(Duration::from_millis(250));
            type_physical(dev, "eng", 35)?;
            double_shift_enter(dev, 900)?;
            eprintln!("[test] сценарий eng_ru_to_us_enter отправлен");
        }
        "plain_layout_ashdu_space_enter" => {
            activate_layout("ru");
            sleep(Duration::from_millis(250));
            type_physical(dev, "file ", 35)?;
            sleep(Duration::from_millis(650));
            tap(dev, KeyCode::KEY_ENTER.code())?;
            eprintln!("[test] сценарий plain_layout_ashdu_space_enter отправлен");
        }
        "plain_layout_cargo_space_enter" => {
            activate_layout("ru");
            sleep(Duration::from_millis(250));
            type_physical(dev, "cargo ", 35)?;
            sleep(Duration::from_millis(650));
            tap(dev, KeyCode::KEY_ENTER.code())?;
            eprintln!("[test] сценарий plain_layout_cargo_space_enter отправлен");
        }
        "plain_layout_abkt_space_enter" => {
            activate_layout("us");
            sleep(Duration::from_millis(250));
            type_physical(dev, "abkt ", 35)?;
            sleep(Duration::from_millis(650));
            tap(dev, KeyCode::KEY_ENTER.code())?;
            eprintln!("[test] сценарий plain_layout_abkt_space_enter отправлен");
        }
        "n_teper_mixed_enter" => {
            activate_layout("us");
            sleep(Duration::from_millis(250));
            hold_tap(dev, KeyCode::KEY_LEFTSHIFT.code(), KeyCode::KEY_N.code())?;
            sleep(Duration::from_millis(120));
            activate_layout("ru");
            sleep(Duration::from_millis(250));
            type_physical(dev, "tgthm", 50)?;
            double_shift_manual(dev, 600)?;
            tap(dev, KeyCode::KEY_ENTER.code())?;
            eprintln!("[test] сценарий n_teper_mixed_enter отправлен");
        }
        "привет_shift" => {
            type_physical(dev, "ghbdt", 50)?;
            double_shift_manual(dev, 500)?;
            eprintln!("[test] сценарий привет_shift отправлен");
        }
        "параллелепипед_long" | "parallelepiped_long" => {
            for _ in 0..20 {
                type_physical(dev, "gfhfktkgbgtl", 12)?;
            }
            double_shift_manual_after(dev, 160, 5000)?;
            tap(dev, KeyCode::KEY_ENTER.code())?;
            eprintln!("[test] сценарий параллелепипед_long отправлен");
        }
        "three_words" => {
            type_physical(dev, "ghbdtn rfr ltkf", 35)?;
            double_shift_manual_after(dev, 180, 1200)?;
            tap(dev, KeyCode::KEY_ENTER.code())?;
            eprintln!("[test] сценарий three_words отправлен");
        }
        "mixed_word" => {
            type_physical(dev, "gh", 50)?;
            sleep(Duration::from_millis(120));
            activate_layout("ru");
            sleep(Duration::from_millis(250));
            tap(dev, KeyCode::KEY_B.code())?;
            sleep(Duration::from_millis(180));
            tap(dev, KeyCode::KEY_LEFTSHIFT.code())?;
            sleep(Duration::from_millis(80));
            tap(dev, KeyCode::KEY_LEFTSHIFT.code())?;
            sleep(Duration::from_millis(1200));
            tap(dev, KeyCode::KEY_ENTER.code())?;
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

fn builtin_script(scenario: &str) -> Option<&'static str> {
    match scenario {
        "good_ntrcn_enter" => Some(include_str!(
            "../../../data/test_input/good_ntrcn_enter.tsv"
        )),
        "proverka_ntrcn_enter" => Some(include_str!(
            "../../../data/test_input/proverka_ntrcn_enter.tsv"
        )),
        "good_vshgidu_enter" => Some(include_str!(
            "../../../data/test_input/good_vshgidu_enter.tsv"
        )),
        "good_text_enter" => Some(include_str!("../../../data/test_input/good_text_enter.tsv")),
        "wifi_ye_enter" => Some(include_str!("../../../data/test_input/wifi_ye_enter.tsv")),
        "auto_switch_words_enter" => Some(include_str!(
            "../../../data/test_input/auto_switch_words_enter.tsv"
        )),
        "worked_nj_space_enter" => Some(include_str!(
            "../../../data/test_input/worked_nj_space_enter.tsv"
        )),
        "html_djn_spacing_enter" => Some(include_str!(
            "../../../data/test_input/html_djn_spacing_enter.tsv"
        )),
        "preparatov_typo_enter" => Some(include_str!(
            "../../../data/test_input/preparatov_typo_enter.tsv"
        )),
        "no_ne_ty_enter" => Some(include_str!("../../../data/test_input/no_ne_ty_enter.tsv")),
        "glued_tozhesamoe_next_enter" => Some(include_str!(
            "../../../data/test_input/glued_tozhesamoe_next_enter.tsv"
        )),
        "glued_tozhesamoe_pause_next_enter" => Some(include_str!(
            "../../../data/test_input/glued_tozhesamoe_pause_next_enter.tsv"
        )),
        "glued_toesamoe_next_enter" => Some(include_str!(
            "../../../data/test_input/glued_toesamoe_next_enter.tsv"
        )),
        "glued_yanebudu_next_enter" => Some(include_str!(
            "../../../data/test_input/glued_yanebudu_next_enter.tsv"
        )),
        "glued_context_yanebudu_next_enter" => Some(include_str!(
            "../../../data/test_input/glued_context_yanebudu_next_enter.tsv"
        )),
        "glued_long_phrase_next_enter" => Some(include_str!(
            "../../../data/test_input/glued_long_phrase_next_enter.tsv"
        )),
        "vyvodim_dva_enter" => Some(include_str!(
            "../../../data/test_input/vyvodim_dva_enter.tsv"
        )),
        "mixed_coke_enter" => Some(include_str!(
            "../../../data/test_input/mixed_coke_enter.tsv"
        )),
        "mixed_coke_toggle3_enter" => Some(include_str!(
            "../../../data/test_input/mixed_coke_toggle3_enter.tsv"
        )),
        _ => None,
    }
}
