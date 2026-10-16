//! Helpers for files that may contain local user text or usage metadata.

use std::io::Write;
#[cfg(unix)]
use std::os::unix::fs::OpenOptionsExt;
use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};

static ATOMIC_WRITE_SEQUENCE: AtomicU64 = AtomicU64::new(0);

/// Hold a separate lock file for a complete read/modify/rename transaction.
pub(crate) fn lock_private_file(path: &Path) -> std::io::Result<std::fs::File> {
    ensure_parent(path)?;
    let mut options = std::fs::OpenOptions::new();
    options.read(true).write(true).create(true).truncate(false);
    #[cfg(unix)]
    options
        .mode(0o600)
        .custom_flags(libc::O_CLOEXEC | libc::O_NOFOLLOW);
    let file = options.open(path)?;
    #[cfg(unix)]
    {
        use std::os::fd::AsRawFd;
        loop {
            // The owned descriptor stays open throughout the transaction.
            if unsafe { libc::flock(file.as_raw_fd(), libc::LOCK_EX) } == 0 {
                break;
            }
            let error = std::io::Error::last_os_error();
            if error.kind() != std::io::ErrorKind::Interrupted {
                return Err(error);
            }
        }
    }
    #[cfg(not(unix))]
    return Err(std::io::Error::new(
        std::io::ErrorKind::Unsupported,
        "private transaction locks require Unix",
    ));
    Ok(file)
}

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

pub fn write_private_bytes_atomic(path: &Path, bytes: &[u8]) -> std::io::Result<()> {
    ensure_parent(path)?;
    let sequence = ATOMIC_WRITE_SEQUENCE.fetch_add(1, Ordering::Relaxed);
    let name = path
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or("private");
    let temporary = path.with_file_name(format!(".{name}.{}.{}.tmp", std::process::id(), sequence));
    let mut options = std::fs::OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    options
        .mode(0o600)
        .custom_flags(libc::O_CLOEXEC | libc::O_NOFOLLOW);
    let mut file = options.open(&temporary)?;
    restrict_file_permissions(&temporary);
    if let Err(error) = file.write_all(bytes).and_then(|()| file.sync_all()) {
        let _ = std::fs::remove_file(&temporary);
        return Err(error);
    }
    drop(file);
    if let Err(error) = std::fs::rename(&temporary, path) {
        let _ = std::fs::remove_file(&temporary);
        return Err(error);
    }
    restrict_file_permissions(path);
    Ok(())
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

    #[cfg(unix)]
    #[test]
    fn transaction_lock_excludes_other_descriptors_until_drop() {
        use std::os::fd::AsRawFd;
        let tmp = std::env::temp_dir().join(format!(
            "lay-private-lock-test-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let path = tmp.join("state.lock");
        let held = lock_private_file(&path).unwrap();
        let other = std::fs::OpenOptions::new()
            .read(true)
            .write(true)
            .open(&path)
            .unwrap();
        assert_eq!(
            unsafe { libc::flock(other.as_raw_fd(), libc::LOCK_EX | libc::LOCK_NB) },
            -1
        );
        assert_eq!(
            std::io::Error::last_os_error().kind(),
            std::io::ErrorKind::WouldBlock
        );
        drop(held);
        assert_eq!(
            unsafe { libc::flock(other.as_raw_fd(), libc::LOCK_EX | libc::LOCK_NB) },
            0
        );
        drop(other);
        std::fs::remove_dir_all(tmp).unwrap();
    }

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

    #[test]
    fn atomic_write_replaces_complete_private_file() {
        let tmp = std::env::temp_dir().join(format!(
            "lay-private-atomic-test-{}",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos()
        ));
        let path = tmp.join("runtime.json");

        write_private_bytes_atomic(&path, br#"{"generation":1}"#).unwrap();
        write_private_bytes_atomic(&path, br#"{"generation":2}"#).unwrap();

        assert_eq!(std::fs::read(&path).unwrap(), br#"{"generation":2}"#);
        assert_eq!(std::fs::read_dir(&tmp).unwrap().count(), 1);
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
