use super::*;
use std::os::unix::fs::PermissionsExt;

const WINDOW_A: &str = "12345678-1234-4234-8234-123456789abc";
const WINDOW_B: &str = "12345678-1234-4234-8234-123456789abd";

fn reply(uuid: &str, title: &str) -> RawWindow {
    RawWindow {
        uuid: uuid.into(),
        pid: 12345,
        app: "org.kde.konsole".into(),
        class: "konsole".into(),
        title: title.into(),
    }
}

#[test]
fn process_id_is_canonical_metadata_not_a_window_identity_or_authority() {
    let a = reply(WINDOW_A, "same");
    let mut b = a.clone();
    b.pid += 1;
    let first = observation("bus1", ":1.5", 1, a);
    let second = observation("bus1", ":1.5", 1, b);
    assert_eq!(first.identity, second.identity);
    let first_info: serde_json::Value = serde_json::from_str(&first.info).unwrap();
    let second_info: serde_json::Value = serde_json::from_str(&second.info).unwrap();
    assert_eq!(first_info["pid"], 12345);
    assert_eq!(second_info["pid"], 12346);
}

#[test]
fn occupied_provider_refuses_instead_of_waiting_or_starting_ipc() {
    with_occupied_provider_for_test(|| {
        assert!(matches!(read(), Err(ReadError::Busy)));
    });
}

#[test]
fn only_current_authenticated_request_can_supply_foreground() {
    let mailbox = Mailbox::default();
    let deadline = Instant::now() + Duration::from_secs(1);
    assert!(!mailbox.deliver(":1.5", "1", reply(WINDOW_A, "same"), Instant::now()));
    mailbox.arm(":1.5", "2", deadline).unwrap();
    assert!(!mailbox.deliver(":1.6", "2", reply(WINDOW_A, "same"), Instant::now()));
    assert!(!mailbox.deliver(":1.5", "1", reply(WINDOW_A, "same"), Instant::now()));
    assert!(mailbox.deliver(":1.5", "2", reply(WINDOW_B, "same"), Instant::now()));
    assert!(!mailbox.deliver(":1.5", "2", reply(WINDOW_A, "same"), Instant::now()));
    assert_eq!(mailbox.take(deadline).unwrap().uuid, WINDOW_B);
    mailbox.clear();
    assert!(!mailbox.deliver(":1.5", "2", reply(WINDOW_A, "same"), Instant::now()));
}

#[test]
fn expired_and_previous_lifetime_replies_never_recover_authority() {
    let mailbox = Mailbox::default();
    let deadline = Instant::now() + Duration::from_secs(1);
    mailbox.arm(":1.5", "old", deadline).unwrap();
    assert!(!mailbox.deliver(":1.5", "old", reply(WINDOW_A, "old"), deadline));
    mailbox.clear();
    mailbox.arm(":1.7", "new", deadline).unwrap();
    assert!(!mailbox.deliver(":1.5", "new", reply(WINDOW_A, "old"), Instant::now()));
    assert!(!mailbox.deliver(":1.7", "old", reply(WINDOW_A, "old"), Instant::now()));
    assert!(mailbox.deliver(":1.7", "new", reply(WINDOW_B, "new"), Instant::now()));
    assert_eq!(mailbox.take(deadline).unwrap().uuid, WINDOW_B);
}

#[test]
fn no_active_window_and_metadata_cannot_replace_stable_identity() {
    for uuid in [
        "",
        "window-title",
        "1234",
        "00000000-0000-0000-0000-000000000000",
    ] {
        let mailbox = Mailbox::default();
        let deadline = Instant::now() + Duration::from_secs(1);
        mailbox.arm(":1.5", "1", deadline).unwrap();
        assert!(!mailbox.deliver(":1.5", "1", reply(uuid, WINDOW_A), Instant::now()));
        assert!(mailbox.take(deadline).is_err(), "uuid={uuid:?}");
    }
}

#[test]
fn stable_uuid_is_canonical_and_oversize_metadata_is_refused() {
    let raw = reply("{12345678-1234-4234-8234-123456789ABC}", "title");
    assert_eq!(raw.validate().unwrap().uuid, WINDOW_A);
    let raw = reply(WINDOW_A, &"x".repeat(MAX_METADATA_BYTES + 1));
    assert!(raw.validate().is_err());
}

#[test]
fn windows_and_compositor_bus_connection_lifetimes_stay_distinct() {
    let a = reply(WINDOW_A, "same").validate().unwrap();
    let b = reply(WINDOW_B, "same").validate().unwrap();
    let original = observation("bus1", ":1.5", 1, a.clone());
    assert_ne!(
        original.identity,
        observation("bus1", ":1.5", 1, b).identity
    );
    assert_ne!(
        original.identity,
        observation("bus1", ":1.6", 1, a.clone()).identity
    );
    assert_ne!(
        original.identity,
        observation("bus2", ":1.5", 1, a.clone()).identity
    );
    assert_ne!(
        original.identity,
        observation("bus1", ":1.5", 2, a).identity
    );
    assert_eq!(
        original.identity,
        observation(
            "bus1",
            ":1.5",
            1,
            reply(WINDOW_A, "renamed").validate().unwrap()
        )
        .identity
    );
}

#[test]
fn one_deadline_bounds_the_whole_read_and_does_not_accept_late_success() {
    let past = Instant::now() - Duration::from_millis(1);
    assert!(bounded(past, async { 42 }).is_err());
    let deadline = Instant::now() + Duration::from_millis(5);
    let result = bounded(deadline, async {
        async_io::Timer::after(Duration::from_millis(50)).await;
        42
    });
    assert!(result.is_err());
    assert_eq!(
        bounded(Instant::now() + Duration::from_secs(1), async { 42 }).unwrap(),
        42
    );
}

#[test]
fn script_is_request_bound_and_contains_no_text_or_layout_mutation() {
    let source = script_source(":1.8", "request-12");
    assert!(source.contains("workspace.activeClient"));
    assert!(source.contains("internalId"));
    let first = source.lines().next().unwrap();
    let value: serde_json::Value = serde_json::from_str(
        first
            .strip_prefix("var layQuery = ")
            .unwrap()
            .strip_suffix(';')
            .unwrap(),
    )
    .unwrap();
    assert_eq!(value["destination"], ":1.8");
    assert_eq!(value["request"], "request-12");
    assert!(!source.contains("QTimer"));
    assert!(!source.contains("setLayout"));
    assert!(!source.contains("activateClient"));
}

#[derive(Default)]
struct PeerState {
    config: serde_json::Value,
    calls: Vec<String>,
    bad_request: bool,
    unload_error: bool,
    wrong_sender: Option<zbus::Connection>,
    pid: Option<i32>,
}

struct PeerScripting(Arc<Mutex<PeerState>>);
#[zbus::interface(name = "org.kde.kwin.Scripting")]
impl PeerScripting {
    #[zbus(name = "loadScript")]
    fn load_script(&self, path: &str, _plugin: &str) -> i32 {
        let source = std::fs::read_to_string(path).unwrap();
        let first = source.lines().next().unwrap();
        let config = serde_json::from_str(
            first
                .strip_prefix("var layQuery = ")
                .unwrap()
                .strip_suffix(';')
                .unwrap(),
        )
        .unwrap();
        let mut state = self.0.lock().unwrap();
        state.calls.push("load".into());
        state.config = config;
        0
    }
    #[zbus(name = "unloadScript")]
    fn unload_script(&self, _plugin: &str) -> zbus::fdo::Result<bool> {
        let mut state = self.0.lock().unwrap();
        state.calls.push("unload".into());
        if state.unload_error {
            Err(zbus::fdo::Error::Failed("controlled unload failure".into()))
        } else {
            Ok(true)
        }
    }
}

struct PeerScript(Arc<Mutex<PeerState>>);
#[zbus::interface(name = "org.kde.kwin.Script")]
impl PeerScript {
    #[zbus(name = "run")]
    async fn run(&self, #[zbus(connection)] conn: &zbus::Connection) -> zbus::fdo::Result<()> {
        let (config, request, wrong_sender, pid) = {
            let mut state = self.0.lock().unwrap();
            state.calls.push("run".into());
            let request = if state.bad_request {
                "old".into()
            } else {
                state.config["request"].as_str().unwrap().to_string()
            };
            (
                state.config.clone(),
                request,
                state.wrong_sender.clone(),
                state.pid.unwrap_or(12345),
            )
        };
        wrong_sender
            .as_ref()
            .unwrap_or(conn)
            .call_method(
                Some(config["destination"].as_str().unwrap()),
                PATH,
                Some(INTERFACE),
                "Observe",
                &(
                    request,
                    WINDOW_A,
                    pid,
                    "org.kde.konsole",
                    "konsole",
                    "native peer window",
                ),
            )
            .await
            .map_err(|e| zbus::fdo::Error::Failed(e.to_string()))?;
        Ok(())
    }
}

struct PrivateBus {
    process: std::process::Child,
    address: String,
    runtime: PrivateRuntime,
}

impl PrivateBus {
    fn start() -> Self {
        let runtime = PrivateRuntime::new();
        let address = format!("unix:path={}", runtime.0.join("bus").display());
        let output = std::fs::File::create(runtime.0.join("bus.log")).unwrap();
        let process = std::process::Command::new("dbus-daemon")
            .args([
                "--session",
                "--nofork",
                "--print-address=1",
                "--address",
                &address,
            ])
            .stdout(output)
            .stderr(std::process::Stdio::null())
            .spawn()
            .unwrap();
        let mut bus = Self {
            process,
            address,
            runtime,
        };
        let deadline = Instant::now() + Duration::from_secs(3);
        loop {
            assert!(
                bus.process.try_wait().unwrap().is_none(),
                "private bus exited"
            );
            if !std::fs::read_to_string(bus.runtime.0.join("bus.log"))
                .unwrap()
                .trim()
                .is_empty()
            {
                return bus;
            }
            assert!(Instant::now() < deadline, "private bus startup deadline");
            std::thread::sleep(Duration::from_millis(2));
        }
    }
}

impl Drop for PrivateBus {
    fn drop(&mut self) {
        let _ = self.process.kill();
        self.process.wait().unwrap();
        for name in ["bus", "bus.log"] {
            let _ = std::fs::remove_file(self.runtime.0.join(name));
        }
    }
}

struct PeerService {
    connection: zbus::Connection,
    bus: PrivateBus,
}

fn service(address: &str, state: Arc<Mutex<PeerState>>) -> zbus::Connection {
    bounded(
        Instant::now() + Duration::from_secs(3),
        zbus::connection::Builder::address(address)
            .unwrap()
            .serve_at("/Scripting", PeerScripting(state.clone()))
            .unwrap()
            .serve_at("/0", PeerScript(state))
            .unwrap()
            .name("org.kde.KWin")
            .unwrap()
            .build(),
    )
    .unwrap()
    .unwrap()
}

fn peer() -> (Client, PeerService, Arc<Mutex<PeerState>>) {
    let bus = PrivateBus::start();
    let state = Arc::new(Mutex::new(PeerState::default()));
    let server = service(&bus.address, state.clone());
    let mailbox = Mailbox::default();
    let conn = bounded(
        Instant::now() + Duration::from_secs(3),
        zbus::connection::Builder::address(bus.address.as_str())
            .unwrap()
            .serve_at(PATH, Ingress(mailbox.clone()))
            .unwrap()
            .build(),
    )
    .unwrap()
    .unwrap();
    let client = Client {
        connection: Some(conn),
        lifetime: serial().unwrap(),
        mailbox,
        cleanup: None,
    };
    (
        client,
        PeerService {
            connection: server,
            bus,
        },
        state,
    )
}

struct PrivateRuntime(PathBuf);
impl PrivateRuntime {
    fn new() -> Self {
        let dir = std::env::temp_dir().join(format!(
            "lay-kde-focus-test-{}-{}",
            std::process::id(),
            serial().unwrap()
        ));
        std::fs::create_dir(&dir).unwrap();
        std::fs::set_permissions(&dir, std::fs::Permissions::from_mode(0o700)).unwrap();
        Self(dir)
    }
}
impl Drop for PrivateRuntime {
    fn drop(&mut self) {
        std::fs::remove_dir(&self.0).unwrap();
    }
}

#[test]
fn actual_ipc_query_loads_calls_authenticated_ingress_and_unloads() {
    let (mut client, _server, state) = peer();
    let runtime = PrivateRuntime::new();
    let result = client
        .read_in(Instant::now() + READ_BUDGET, &runtime.0)
        .unwrap();
    assert!(result.identity.ends_with(WINDOW_A));
    assert_eq!(
        serde_json::from_str::<serde_json::Value>(&result.info).unwrap()["wmClass"],
        "konsole"
    );
    assert_eq!(
        serde_json::from_str::<serde_json::Value>(&result.info).unwrap()["pid"],
        12345
    );
    assert_eq!(state.lock().unwrap().calls, ["load", "run", "unload"]);
    assert!(client.cleanup.is_none());
    client.mailbox.clear();
    let again = client
        .read_in(Instant::now() + READ_BUDGET, &runtime.0)
        .unwrap();
    assert_eq!(again.identity, result.identity);
    assert_eq!(state.lock().unwrap().calls.len(), 6);
    assert_eq!(std::fs::read_dir(&runtime.0).unwrap().count(), 0);
}

#[test]
fn actual_ipc_signed_process_identity_rejects_negative_before_recovery() {
    let (mut client, _server, state) = peer();
    let runtime = PrivateRuntime::new();
    state.lock().unwrap().pid = Some(-1);
    assert!(client
        .read_in(Instant::now() + Duration::from_millis(30), &runtime.0)
        .is_err());
    client.mailbox.clear();
    state.lock().unwrap().pid = Some(i32::MAX);
    let recovered = client
        .read_in(Instant::now() + READ_BUDGET, &runtime.0)
        .unwrap();
    assert_eq!(
        serde_json::from_str::<serde_json::Value>(&recovered.info).unwrap()["pid"],
        i32::MAX
    );
    assert!(client.cleanup.is_none());
}

#[test]
fn actual_ipc_wrong_request_is_refused_and_cleanup_precedes_recovery() {
    let (mut client, _server, state) = peer();
    let runtime = PrivateRuntime::new();
    state.lock().unwrap().bad_request = true;
    assert!(client
        .read_in(Instant::now() + Duration::from_millis(30), &runtime.0)
        .is_err());
    assert!(client.cleanup.is_some());
    client.mailbox.clear();
    state.lock().unwrap().bad_request = false;
    let recovered = client
        .read_in(Instant::now() + READ_BUDGET, &runtime.0)
        .unwrap();
    assert!(recovered.identity.ends_with(WINDOW_A));
    assert_eq!(
        state.lock().unwrap().calls,
        ["load", "run", "unload", "load", "run", "unload"]
    );
}

#[test]
fn actual_ipc_failed_cleanup_blocks_new_load_and_wrong_sender_refuses() {
    let (mut client, server, state) = peer();
    let runtime = PrivateRuntime::new();
    state.lock().unwrap().unload_error = true;
    assert!(client
        .read_in(Instant::now() + READ_BUDGET, &runtime.0)
        .is_err());
    client.mailbox.clear();
    assert!(client
        .read_in(Instant::now() + READ_BUDGET, &runtime.0)
        .is_err());
    assert_eq!(
        state.lock().unwrap().calls,
        ["load", "run", "unload", "unload"]
    );
    client.mailbox.clear();
    state.lock().unwrap().unload_error = false;
    let intruder = bounded(
        Instant::now() + READ_BUDGET,
        zbus::connection::Builder::address(server.bus.address.as_str())
            .unwrap()
            .build(),
    )
    .unwrap()
    .unwrap();
    state.lock().unwrap().wrong_sender = Some(intruder);
    assert!(client
        .read_in(Instant::now() + Duration::from_millis(30), &runtime.0)
        .is_err());
    // The real bus assigns a different sender: the callback cannot grant authority.
}

#[test]
fn actual_ipc_replacing_service_owner_changes_window_identity() {
    let (mut client, mut server, state) = peer();
    let runtime = PrivateRuntime::new();
    let first = client
        .read_in(Instant::now() + READ_BUDGET, &runtime.0)
        .unwrap();
    client.mailbox.clear();
    bounded(
        Instant::now() + READ_BUDGET,
        server.connection.release_name("org.kde.KWin"),
    )
    .unwrap()
    .unwrap();
    server.connection = service(&server.bus.address, state);
    let replacement = client
        .read_in(Instant::now() + READ_BUDGET, &runtime.0)
        .unwrap();
    assert_ne!(first.identity, replacement.identity);
}

#[test]
fn runtime_script_rejects_shared_directory_and_never_overwrites_existing_file() {
    let runtime = PrivateRuntime::new();
    let script = RuntimeScript::create_in(&runtime.0, "source", "one").unwrap();
    assert_eq!(
        std::fs::metadata(&script.0).unwrap().permissions().mode() & 0o777,
        0o600
    );
    assert!(RuntimeScript::create_in(&runtime.0, "replacement", "one").is_err());
    assert_eq!(std::fs::read_to_string(&script.0).unwrap(), "source");
    drop(script);
    std::fs::set_permissions(&runtime.0, std::fs::Permissions::from_mode(0o755)).unwrap();
    assert!(RuntimeScript::create_in(&runtime.0, "source", "two").is_err());
}
