//! Helpers for files that may contain local user text or usage metadata.

use std::io::Write;
#[cfg(unix)]
use std::os::unix::fs::OpenOptionsExt;
use std::path::Path;

pub fn append_private_text(path: &Path, text: &str) -> std::io::Result<()> {
    ensure_parent(path)?;
    let mut options = std::fs::OpenOptions::new();
    options.append(true).create(true);
    #[cfg(unix)]
    options.mode(0o600);
    let mut file = options.open(path)?;
    restrict_file_permissions(path);
    file.write_all(text.as_bytes())
}

pub fn write_private_text(path: &Path, text: &str) -> std::io::Result<()> {
    write_private_bytes(path, text.as_bytes())
}

pub fn write_private_bytes(path: &Path, bytes: &[u8]) -> std::io::Result<()> {
    ensure_parent(path)?;
    let mut options = std::fs::OpenOptions::new();
    options.write(true).create(true).truncate(true);
    #[cfg(unix)]
    options.mode(0o600);
    let mut file = options.open(path)?;
    restrict_file_permissions(path);
    file.write_all(bytes)
}

fn ensure_parent(path: &Path) -> std::io::Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    Ok(())
}

#[cfg(unix)]
fn restrict_file_permissions(path: &Path) {
    use std::os::unix::fs::PermissionsExt;
    let _ = std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o600));
}

#[cfg(not(unix))]
fn restrict_file_permissions(_path: &Path) {}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn writes_and_appends_private_text() {
        let tmp = std::env::temp_dir().join(format!(
            "lay-private-file-test-{}",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos()
        ));
        let path = tmp.join("nested").join("private.txt");

        write_private_text(&path, "one\n").unwrap();
        append_private_text(&path, "two\n").unwrap();

        assert_eq!(std::fs::read_to_string(&path).unwrap(), "one\ntwo\n");

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            assert_eq!(
                std::fs::metadata(&path).unwrap().permissions().mode() & 0o777,
                0o600
            );
        }

        let _ = std::fs::remove_dir_all(tmp);
    }
}
