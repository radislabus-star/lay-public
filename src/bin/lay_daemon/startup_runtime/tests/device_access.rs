use super::*;
use std::fs::{File, OpenOptions, Permissions};
use std::os::unix::fs::PermissionsExt;
use std::sync::mpsc::{self, Sender};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

struct PrivateFile(PathBuf);

impl PrivateFile {
    fn new() -> Self {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path =
            std::env::temp_dir().join(format!("lay-device-access-{}-{stamp}", std::process::id()));
        File::create(&path).unwrap();
        Self(path)
    }

    fn permissions(&self, mode: u32) {
        std::fs::set_permissions(&self.0, Permissions::from_mode(mode)).unwrap();
    }

    fn open(&self) -> io::Result<File> {
        OpenOptions::new().read(true).write(true).open(&self.0)
    }
}

impl Drop for PrivateFile {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.0);
    }
}

fn blocked_reader(releases: &mut Vec<Sender<()>>) -> JoinHandle<()> {
    let (tx, rx) = mpsc::channel();
    releases.push(tx);
    std::thread::spawn(move || {
        let _ = rx.recv();
    })
}

fn finish(listeners: Listeners, releases: Vec<Sender<()>>) {
    drop(releases);
    for (_, handle) in listeners {
        handle.join().unwrap();
    }
}

#[test]
fn late_permissions_recover_output_once_without_replacing_ready_handle() {
    let file = PrivateFile::new();
    file.permissions(0o000);
    let shared = Mutex::new(None);
    let stop = AtomicBool::new(false);
    assert_eq!(
        recover_output(&shared, false, &stop, || file.open())
            .unwrap_err()
            .kind(),
        io::ErrorKind::PermissionDenied
    );
    assert!(shared.lock().unwrap().is_none());
    file.permissions(0o600);
    assert_eq!(
        recover_output(&shared, false, &stop, || file.open()).unwrap(),
        OutputState::Recovered
    );
    assert_eq!(
        recover_output(&shared, false, &stop, || -> io::Result<File> {
            panic!("ready handle must not reopen")
        })
        .unwrap(),
        OutputState::Ready
    );
    assert!(shared.lock().unwrap().is_some());
}

#[test]
fn late_permissions_detect_only_shutdown_and_busy_skip_device_creation() {
    let stop = AtomicBool::new(false);
    let shared = Mutex::new(None::<u8>);
    assert_eq!(
        recover_output(&shared, true, &stop, || panic!("detect-only output")).unwrap(),
        OutputState::Skipped
    );
    let guard = shared.lock().unwrap();
    assert_eq!(
        recover_output(&shared, false, &stop, || panic!("busy output")).unwrap(),
        OutputState::Skipped
    );
    drop(guard);
    stop.store(true, Ordering::Release);
    assert_eq!(
        recover_output(&shared, false, &stop, || panic!("shutdown output")).unwrap(),
        OutputState::Skipped
    );
    assert!(shared.lock().unwrap().is_none());
}

#[test]
fn late_permissions_failure_and_shutdown_during_creation_do_not_publish_output() {
    let stop = AtomicBool::new(false);
    let shared = Mutex::new(None::<u8>);
    assert!(
        recover_output(&shared, false, &stop, || Err(io::Error::other(
            "release sanitizer failed"
        )))
        .is_err()
    );
    assert!(shared.lock().unwrap().is_none());
    assert_eq!(
        recover_output(&shared, false, &stop, || {
            stop.store(true, Ordering::Release);
            Ok(7)
        })
        .unwrap(),
        OutputState::Skipped
    );
    assert!(shared.lock().unwrap().is_none());
}

#[test]
fn late_permissions_poison_recovery_preserves_existing_device() {
    let shared = Mutex::new(Some(7));
    let _ = std::panic::catch_unwind(|| {
        let _guard = shared.lock().unwrap();
        panic!("injected poison");
    });
    assert_eq!(
        recover_output(&shared, false, &AtomicBool::new(false), || panic!(
            "existing poisoned device"
        ))
        .unwrap(),
        OutputState::Ready
    );
    assert_eq!(*shared.lock().unwrap_err().into_inner(), Some(7));
}

#[test]
fn late_permissions_discovery_recovers_keyboard_and_keeps_pointers_independent() {
    let path = PathBuf::from("/test/keyboard");
    let pointer = PathBuf::from("/test/pointer");
    let (keyboard, pointers) = discover_paths(
        None,
        || Err(io::ErrorKind::PermissionDenied.into()),
        || Ok(vec![pointer.clone()]),
    );
    assert_eq!(
        keyboard.unwrap_err().kind(),
        io::ErrorKind::PermissionDenied
    );
    assert_eq!(pointers.unwrap(), vec![pointer]);
    let (keyboard, _) = discover_paths(None, || Ok(vec![path.clone()]), || Ok(vec![]));
    assert_eq!(keyboard.unwrap(), vec![path.clone()]);
    let (keyboard, _) = discover_paths(
        Some(&path),
        || panic!("explicit path must bypass automatic discovery"),
        || Ok(vec![]),
    );
    assert_eq!(keyboard.unwrap(), vec![path]);
}

#[test]
fn late_permissions_listener_recovers_after_real_access_denial() {
    let file = PrivateFile::new();
    file.permissions(0o000);
    let key = (DeviceKind::Keyboard, file.0.clone());
    let desired = vec![key.clone()];
    let stop = AtomicBool::new(false);
    let mut listeners = Listeners::new();
    let errors = refresh_listeners(&mut listeners, &desired, &stop, |_| {
        let _fd = file.open()?;
        panic!("denied file must not start a reader")
    });
    assert_eq!(
        errors[0].1.as_ref().unwrap_err().kind(),
        io::ErrorKind::PermissionDenied
    );
    assert!(listeners.is_empty());
    file.permissions(0o600);
    let mut releases = Vec::new();
    let started = refresh_listeners(&mut listeners, &desired, &stop, |_| {
        let fd = file.open()?;
        let (tx, rx) = mpsc::channel();
        releases.push(tx);
        Ok(std::thread::spawn(move || {
            let _fd = fd;
            let _ = rx.recv();
        }))
    });
    assert!(started[0].1.is_ok());
    assert_eq!(listeners.len(), 1);
    assert!(
        refresh_listeners(&mut listeners, &desired, &stop, |_| panic!(
            "duplicate reader"
        ))
        .is_empty()
    );
    // A temporary scan miss cannot discard a still-running reader.
    assert!(
        refresh_listeners(&mut listeners, &[], &stop, |_| panic!("unexpected reader")).is_empty()
    );
    assert_eq!(listeners.len(), 1);
    finish(listeners, releases);
}

#[test]
fn late_permissions_registry_reaps_finished_listener_and_distinguishes_device_kinds() {
    let stop = AtomicBool::new(false);
    let key = (DeviceKind::Keyboard, PathBuf::from("/test/input"));
    let mut listeners = Listeners::new();
    listeners.insert(key.clone(), std::thread::spawn(|| {}));
    let deadline = Instant::now() + Duration::from_secs(2);
    while !listeners[&key].is_finished() {
        assert!(Instant::now() < deadline, "fixture listener did not finish");
        std::thread::yield_now();
    }
    let desired = vec![
        key.clone(),
        key.clone(),
        (DeviceKind::Pointer, key.1.clone()),
    ];
    let mut releases = Vec::new();
    let attempts = refresh_listeners(&mut listeners, &desired, &stop, |_| {
        Ok(blocked_reader(&mut releases))
    });
    assert_eq!(attempts.len(), 2);
    assert_eq!(listeners.len(), 2);
    assert!(
        refresh_listeners(&mut listeners, &desired, &stop, |_| panic!(
            "unchanged live topology"
        ))
        .is_empty()
    );
    finish(listeners, releases);
}

#[test]
fn late_permissions_shutdown_during_discovery_prevents_listener_spawn() {
    let stop = AtomicBool::new(false);
    let (keys, pointers) = discover_paths(
        None,
        || Ok(vec![PathBuf::from("/test/key")]),
        || {
            stop.store(true, Ordering::Release);
            Ok(vec![PathBuf::from("/test/pointer")])
        },
    );
    let desired: Vec<_> = keys
        .unwrap()
        .into_iter()
        .map(|p| (DeviceKind::Keyboard, p))
        .chain(
            pointers
                .unwrap()
                .into_iter()
                .map(|p| (DeviceKind::Pointer, p)),
        )
        .collect();
    let mut listeners = Listeners::new();
    assert!(
        refresh_listeners(&mut listeners, &desired, &stop, |_| panic!(
            "spawn after shutdown"
        ))
        .is_empty()
    );
    assert!(listeners.is_empty());
}
