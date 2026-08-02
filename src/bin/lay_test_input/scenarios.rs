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

const BUILTIN_SCRIPTS: &str = include_str!("../../../data/test_input/builtin_scripts.tsv");
include!(concat!(
    env!("OUT_DIR"),
    "/lay_test_input_builtin_scripts.rs"
));

pub(crate) fn run_scenario(dev: &mut VirtualDevice, scenario: &str) -> std::io::Result<()> {
    if let Some(path) = scenario.strip_prefix("script:") {
        run_script(dev, Path::new(path))?;
        eprintln!("[test] script-сценарий {path} отправлен");
        return Ok(());
    }

    if let Some(script) = builtin_script(scenario)? {
        run_script_text(dev, &script, scenario)?;
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

fn builtin_script(scenario: &str) -> std::io::Result<Option<String>> {
    let indexed = BUILTIN_SCRIPTS.lines().any(|line| {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            return false;
        }
        line.split_once('\t').is_some_and(|(id, _)| id == scenario)
    });
    if !indexed {
        return Ok(None);
    }
    builtin_script_text(scenario)
        .map(str::to_owned)
        .map(Some)
        .ok_or_else(|| {
            std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("builtin script {scenario:?} was indexed but not embedded"),
            )
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_indexed_builtin_script_is_embedded() {
        for line in BUILTIN_SCRIPTS.lines().map(str::trim) {
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            let (id, _) = line.split_once('\t').expect("builtin script index row");
            assert!(
                builtin_script_text(id).is_some(),
                "missing embedded script {id}"
            );
        }
    }
}
