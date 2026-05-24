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

use script::run_script;
use typing::{double_shift_manual, double_shift_manual_after, type_mixed_coke_tail, type_physical};

pub(crate) fn run_scenario(dev: &mut VirtualDevice, scenario: &str) -> std::io::Result<()> {
    if let Some(path) = scenario.strip_prefix("script:") {
        run_script(dev, Path::new(path))?;
        eprintln!("[test] script-сценарий {path} отправлен");
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
        "good_ntrcn_enter" => {
            activate_layout("us");
            sleep(Duration::from_millis(250));
            type_physical(dev, "good ntrcn", 35)?;
            double_shift_enter(dev, 900)?;
            eprintln!("[test] сценарий good_ntrcn_enter отправлен");
        }
        "proverka_ntrcn_enter" => {
            activate_layout("ru");
            sleep(Duration::from_millis(250));
            type_physical(dev, "ghjdthrf ", 35)?;
            activate_layout("us");
            sleep(Duration::from_millis(250));
            type_physical(dev, "ntrcn", 35)?;
            double_shift_enter(dev, 900)?;
            eprintln!("[test] сценарий proverka_ntrcn_enter отправлен");
        }
        "good_vshgidu_enter" => {
            activate_layout("us");
            sleep(Duration::from_millis(250));
            type_physical(dev, "good ", 35)?;
            activate_layout("ru");
            sleep(Duration::from_millis(250));
            type_physical(dev, "Double", 35)?;
            double_shift_enter(dev, 900)?;
            eprintln!("[test] сценарий good_vshgidu_enter отправлен");
        }
        "good_text_enter" => {
            activate_layout("ru");
            sleep(Duration::from_millis(250));
            type_physical(dev, "good ", 35)?;
            activate_layout("us");
            sleep(Duration::from_millis(250));
            type_physical(dev, "ntrcn", 35)?;
            double_shift_enter(dev, 900)?;
            eprintln!("[test] сценарий good_text_enter отправлен");
        }
        "wifi_ye_enter" => {
            activate_layout("us");
            sleep(Duration::from_millis(250));
            type_physical(dev, "wi-fi ye", 35)?;
            double_shift_enter(dev, 900)?;
            eprintln!("[test] сценарий wifi_ye_enter отправлен");
        }
        "auto_switch_words_enter" => {
            activate_layout("us");
            sleep(Duration::from_millis(250));
            type_physical(dev, "njkmrj ", 35)?;
            sleep(Duration::from_millis(450));
            type_physical(dev, "yt ", 35)?;
            sleep(Duration::from_millis(450));
            type_physical(dev, "hf,jnftn ", 35)?;
            sleep(Duration::from_millis(650));
            tap(dev, KeyCode::KEY_ENTER.code())?;
            eprintln!("[test] сценарий auto_switch_words_enter отправлен");
        }
        "worked_nj_space_enter" => {
            activate_layout("us");
            sleep(Duration::from_millis(250));
            type_physical(dev, "worked 'nj ", 35)?;
            sleep(Duration::from_millis(650));
            tap(dev, KeyCode::KEY_ENTER.code())?;
            eprintln!("[test] сценарий worked_nj_space_enter отправлен");
        }
        "html_djn_spacing_enter" => {
            activate_layout("ru");
            sleep(Duration::from_millis(250));
            type_physical(dev, "html ", 35)?;
            sleep(Duration::from_millis(650));
            type_physical(dev, "djn ", 35)?;
            sleep(Duration::from_millis(650));
            tap(dev, KeyCode::KEY_ENTER.code())?;
            eprintln!("[test] сценарий html_djn_spacing_enter отправлен");
        }
        "preparatov_typo_enter" => {
            activate_layout("ru");
            sleep(Duration::from_millis(250));
            type_physical(dev, "gthgfhfnjd ", 35)?;
            sleep(Duration::from_millis(650));
            tap(dev, KeyCode::KEY_ENTER.code())?;
            eprintln!("[test] сценарий preparatov_typo_enter отправлен");
        }
        "no_ne_ty_enter" => {
            activate_layout("ru");
            sleep(Duration::from_millis(250));
            type_physical(dev, "yj ", 35)?;
            sleep(Duration::from_millis(260));
            type_physical(dev, "yt ", 35)?;
            sleep(Duration::from_millis(260));
            type_physical(dev, "ns ", 35)?;
            sleep(Duration::from_millis(650));
            tap(dev, KeyCode::KEY_ENTER.code())?;
            eprintln!("[test] сценарий no_ne_ty_enter отправлен");
        }
        "glued_tozhesamoe_next_enter" => {
            activate_layout("ru");
            sleep(Duration::from_millis(250));
            type_physical(dev, "nj;tcfvjt crktyj", 18)?;
            sleep(Duration::from_millis(650));
            tap(dev, KeyCode::KEY_ENTER.code())?;
            eprintln!("[test] сценарий glued_tozhesamoe_next_enter отправлен");
        }
        "glued_tozhesamoe_pause_next_enter" => {
            activate_layout("ru");
            sleep(Duration::from_millis(250));
            type_physical(dev, "nj;tcfvjt ", 18)?;
            sleep(Duration::from_millis(650));
            type_physical(dev, "crktyj", 18)?;
            sleep(Duration::from_millis(650));
            tap(dev, KeyCode::KEY_ENTER.code())?;
            eprintln!("[test] сценарий glued_tozhesamoe_pause_next_enter отправлен");
        }
        "glued_toesamoe_next_enter" => {
            activate_layout("ru");
            sleep(Duration::from_millis(250));
            type_physical(dev, "njtcfvjt crktyj", 18)?;
            sleep(Duration::from_millis(650));
            tap(dev, KeyCode::KEY_ENTER.code())?;
            eprintln!("[test] сценарий glued_toesamoe_next_enter отправлен");
        }
        "glued_yanebudu_next_enter" => {
            activate_layout("ru");
            sleep(Duration::from_millis(250));
            type_physical(dev, "zyt,ele crktyj", 18)?;
            sleep(Duration::from_millis(650));
            tap(dev, KeyCode::KEY_ENTER.code())?;
            eprintln!("[test] сценарий glued_yanebudu_next_enter отправлен");
        }
        "glued_context_yanebudu_next_enter" => {
            activate_layout("ru");
            sleep(Duration::from_millis(250));
            type_physical(dev, "nj;t cfvjt zyt,ele crktyj", 18)?;
            sleep(Duration::from_millis(650));
            tap(dev, KeyCode::KEY_ENTER.code())?;
            eprintln!("[test] сценарий glued_context_yanebudu_next_enter отправлен");
        }
        "glued_long_phrase_next_enter" => {
            activate_layout("ru");
            sleep(Duration::from_millis(250));
            type_physical(dev, "zyt,elegfdfcnj;tcfvjt crktyj", 18)?;
            sleep(Duration::from_millis(650));
            tap(dev, KeyCode::KEY_ENTER.code())?;
            eprintln!("[test] сценарий glued_long_phrase_next_enter отправлен");
        }
        "vyvodim_dva_enter" => {
            activate_layout("us");
            sleep(Duration::from_millis(250));
            type_physical(dev, "dsdjlbv ldf", 35)?;
            double_shift_enter(dev, 900)?;
            eprintln!("[test] сценарий vyvodim_dva_enter отправлен");
        }
        "mixed_coke_enter" => {
            type_mixed_coke_tail(dev)?;
            double_shift_enter(dev, 900)?;
            eprintln!("[test] сценарий mixed_coke_enter отправлен");
        }
        "mixed_coke_toggle3_enter" => {
            type_mixed_coke_tail(dev)?;
            double_shift(dev, 900)?;
            double_shift(dev, 900)?;
            double_shift_enter(dev, 900)?;
            eprintln!("[test] сценарий mixed_coke_toggle3_enter отправлен");
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
