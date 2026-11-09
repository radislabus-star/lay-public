use lay::config::LayConfig;
use lay::desktop::{parse_kde_layouts_list, parse_setxkbmap_layout, resolve_layout_backend};
use std::env;
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};

static INITIAL_LAYOUT_ALREADY_SET: AtomicBool = AtomicBool::new(false);

fn expected_test_client(
    pid: Option<&str>,
    title: Option<&str>,
) -> Result<Option<(u32, String)>, String> {
    match (pid, title) {
        (None, None) => Ok(None),
        (Some(pid), Some(title)) if !title.is_empty() => {
            let pid = pid.parse::<u32>().map_err(|_| "invalid test client PID")?;
            if pid == 0 {
                return Err("test client PID must be nonzero".into());
            }
            Ok(Some((pid, title.into())))
        }
        _ => Err("test focus requires both a PID and a nonempty title".into()),
    }
}

fn test_client_matches(raw: &str, expected: &(u32, String)) -> bool {
    let Ok(value) = serde_json::from_str::<serde_json::Value>(raw) else {
        return false;
    };
    let pid = value["pid"]
        .as_u64()
        .or_else(|| value["pid"].as_str()?.parse::<u64>().ok());
    pid == Some(u64::from(expected.0)) && value["title"].as_str() == Some(&expected.1)
}

/// Optional admission for the test injector, never a production focus owner.
pub(crate) fn require_test_client_focus() -> std::io::Result<()> {
    let pid = env::var("LAY_TEST_EXPECT_FOCUS_PID").ok();
    let title = env::var("LAY_TEST_EXPECT_FOCUS_TITLE").ok();
    let expected =
        expected_test_client(pid.as_deref(), title.as_deref()).map_err(std::io::Error::other)?;
    let Some(expected) = expected else {
        return Ok(());
    };
    let observed: Result<String, String> = zbus::block_on(futures_lite::future::race(
        async {
            let connection = zbus::Connection::session()
                .await
                .map_err(|e| e.to_string())?;
            let proxy = zbus::Proxy::new(
                &connection,
                "org.gnome.Shell",
                "/io/github/radislabus_star/LayDaemon",
                "io.github.radislabus_star.LayDaemon",
            )
            .await
            .map_err(|e| e.to_string())?;
            proxy
                .call("FocusedWindowInfo", &())
                .await
                .map_err(|e| e.to_string())
        },
        async {
            async_io::Timer::after(Duration::from_millis(150)).await;
            Err("test client focus observation timed out".into())
        },
    ));
    let raw = observed.map_err(std::io::Error::other)?;
    if !test_client_matches(&raw, &expected) {
        return Err(std::io::Error::other(
            "focused window is not the exact test client",
        ));
    }
    Ok(())
}

pub(crate) fn activate_layout(id: &str) {
    if env::var("LAY_TEST_IME_ENGINE")
        .ok()
        .is_some_and(|value| matches!(value.as_str(), "1" | "true" | "yes" | "on"))
    {
        activate_ibus_engine(if id == "ru" {
            "lay-ime-ru"
        } else {
            "lay-ime-us"
        });
        return;
    }

    if env::var("LAY_TEST_INITIAL_LAYOUT").ok().as_deref() == Some(id)
        && !INITIAL_LAYOUT_ALREADY_SET.swap(true, Ordering::Relaxed)
    {
        return;
    }

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
    activate_ibus_engine(engine);
}

fn activate_ibus_engine(engine: &str) {
    let _ = Command::new("ibus")
        .args(["engine", engine])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();
    let deadline = Instant::now() + Duration::from_millis(800);
    while Instant::now() < deadline {
        if active_ibus_engine().as_deref() == Some(engine) {
            return;
        }
        std::thread::sleep(Duration::from_millis(20));
    }
}

fn active_ibus_engine() -> Option<String> {
    let output = Command::new("ibus").arg("engine").output().ok()?;
    if !output.status.success() {
        return None;
    }
    Some(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

pub(crate) fn print_x11_diagnostics() {
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

pub(crate) fn print_x11_report() {
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

#[cfg(test)]
mod test_client_focus_tests {
    use super::{expected_test_client, test_client_matches};

    #[test]
    fn injector_focus_is_optional_but_partial_or_invalid_admission_is_rejected() {
        assert_eq!(expected_test_client(None, None).unwrap(), None);
        for (pid, title) in [
            (Some("12"), None),
            (None, Some("test")),
            (Some("0"), Some("test")),
            (Some("-1"), Some("test")),
            (Some("4294967296"), Some("test")),
            (Some("12"), Some("")),
        ] {
            assert!(expected_test_client(pid, title).is_err());
        }
    }

    #[test]
    fn injector_focus_requires_both_native_pid_and_exact_title() {
        let expected = expected_test_client(Some("12"), Some("owned test"))
            .unwrap()
            .unwrap();
        for raw in [
            r#"{"pid":"12","title":"owned test"}"#,
            r#"{"pid":12,"title":"owned test"}"#,
        ] {
            assert!(test_client_matches(raw, &expected));
        }
        for raw in [
            r#"{"pid":13,"title":"owned test"}"#,
            r#"{"pid":12,"title":"another window"}"#,
            r#"{"pid":12}"#,
            r#"{"title":"owned test"}"#,
            r#"{"pid":12.5,"title":"owned test"}"#,
            "null",
            "not json",
        ] {
            assert!(!test_client_matches(raw, &expected));
        }
    }
}
