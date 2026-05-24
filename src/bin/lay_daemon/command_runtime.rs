use std::process::Command;

pub(super) fn run_command_capture(command: &str, args: &[&str]) -> Result<String, String> {
    let out = Command::new(command)
        .args(args)
        .output()
        .map_err(|e| format!("{command}: {e}"))?;
    if !out.status.success() {
        return Err(format!(
            "{command}: {}",
            String::from_utf8_lossy(&out.stderr).trim()
        ));
    }
    Ok(String::from_utf8_lossy(&out.stdout).trim().to_string())
}

pub(super) fn command_exists(command: &str) -> bool {
    let Some(paths) = std::env::var_os("PATH") else {
        return false;
    };
    std::env::split_paths(&paths).any(|dir| dir.join(command).is_file())
}
