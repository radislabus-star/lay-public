//! Existing GNOME layout transport; the common worker retains authority.
#[cfg(test)]
use super::LAYOUT_STACK_PROBE;
#[cfg(not(test))]
use std::process::Command;

pub(super) fn activate_gnome_layout_for_ime(target_is_ru: bool) -> Result<(), String> {
    #[cfg(test)]
    {
        LAYOUT_STACK_PROBE.with(|probe| {
            if let Some(probe) = probe.borrow_mut().as_mut() {
                probe.activations.push(target_is_ru);
            }
        });
        Ok(())
    }

    #[cfg(not(test))]
    {
        let layout_id = if target_is_ru { "ru" } else { "us" };
        let out = Command::new("timeout")
            .args([
                "0.18s",
                "gdbus",
                "call",
                "--session",
                "--dest",
                "org.gnome.Shell",
                "--object-path",
                "/io/github/radislabus_star/LayDaemon",
                "--method",
                "io.github.radislabus_star.LayDaemon.ActivateLayout",
                layout_id,
            ])
            .output()
            .map_err(|error| error.to_string())?;
        if out.status.success() && String::from_utf8_lossy(&out.stdout).contains("true") {
            Ok(())
        } else {
            let stderr = String::from_utf8_lossy(&out.stderr).trim().to_string();
            Err(if stderr.is_empty() {
                format!("ActivateLayout {layout_id} exited with {}", out.status)
            } else {
                stderr
            })
        }
    }
}
