//! Fresh KWin evidence for the existing focus owner, never text-edit authority.
//! The embedded script is code only; no observation is read from a file.
use std::future::{poll_fn, Future};
use std::io::Write;
use std::os::unix::fs::MetadataExt;
use std::path::PathBuf;
use std::pin::pin;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Condvar, Mutex, OnceLock, TryLockError};
use std::task::Poll;
use std::time::{Duration, Instant};

const PATH: &str = "/io/github/radislabus_star/LayDaemon/KWinFocus";
const INTERFACE: &str = "io.github.radislabus_star.LayDaemon.KWinFocus";
const READ_BUDGET: Duration = Duration::from_millis(250);
const CLEANUP_BUDGET: Duration = Duration::from_millis(25);
const MAX_METADATA_BYTES: usize = 16 * 1024;
static SERIAL: AtomicU64 = AtomicU64::new(1);
static CLIENT: OnceLock<Mutex<Client>> = OnceLock::new();

pub(super) struct Observation {
    pub(super) identity: String,
    pub(super) info: String,
}

#[derive(Debug)]
pub(super) enum ReadError {
    Busy,
    Unavailable(String),
}

impl std::fmt::Display for ReadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Busy => f.write_str("KWin foreground provider busy"),
            Self::Unavailable(error) => f.write_str(error),
        }
    }
}

#[derive(Clone, Debug)]
struct RawWindow {
    uuid: String,
    app: String,
    class: String,
    title: String,
}

impl RawWindow {
    fn validate(mut self) -> Result<Self, String> {
        let uuid = if self.uuid.len() == 38 {
            self.uuid
                .strip_prefix('{')
                .and_then(|s| s.strip_suffix('}'))
                .ok_or("malformed KWin active window UUID")?
        } else {
            self.uuid.as_str()
        };
        if uuid.len() != 36
            || !uuid.bytes().enumerate().all(|(i, b)| {
                if [8, 13, 18, 23].contains(&i) {
                    b == b'-'
                } else {
                    b.is_ascii_hexdigit()
                }
            })
            || uuid.bytes().all(|b| b == b'0' || b == b'-')
        {
            return Err("KWin has no valid active window UUID".into());
        }
        if self.app.len() + self.class.len() + self.title.len() > MAX_METADATA_BYTES {
            return Err("KWin foreground metadata exceeds bound".into());
        }
        self.uuid = uuid.to_ascii_lowercase();
        Ok(self)
    }
}

struct Pending {
    owner: String,
    request: String,
    deadline: Instant,
    response: Option<Result<RawWindow, String>>,
}

#[derive(Clone, Default)]
struct Mailbox(Arc<(Mutex<Option<Pending>>, Condvar)>);

impl Mailbox {
    fn arm(&self, owner: &str, request: &str, deadline: Instant) -> Result<(), String> {
        let mut slot = self.0 .0.lock().map_err(|e| e.to_string())?;
        if slot.is_some() {
            return Err("KWin foreground request already armed".into());
        }
        *slot = Some(Pending {
            owner: owner.into(),
            request: request.into(),
            deadline,
            response: None,
        });
        Ok(())
    }

    fn deliver(&self, sender: &str, request: &str, raw: RawWindow, now: Instant) -> bool {
        let Ok(mut slot) = self.0 .0.lock() else {
            return false;
        };
        let Some(pending) = slot.as_mut() else {
            return false;
        };
        if sender != pending.owner
            || request != pending.request
            || now >= pending.deadline
            || pending.response.is_some()
        {
            return false;
        }
        let response = raw.validate();
        let accepted = response.is_ok();
        pending.response = Some(response);
        self.0 .1.notify_one();
        accepted
    }

    fn take(&self, deadline: Instant) -> Result<RawWindow, String> {
        let mut slot = self.0 .0.lock().map_err(|e| e.to_string())?;
        loop {
            let remaining = deadline
                .checked_duration_since(Instant::now())
                .filter(|d| !d.is_zero())
                .ok_or("KWin foreground read deadline exceeded")?;
            let pending = slot.as_mut().ok_or("KWin foreground request invalidated")?;
            if let Some(response) = pending.response.take() {
                return response;
            }
            slot = self
                .0
                 .1
                .wait_timeout(slot, remaining)
                .map_err(|e| e.to_string())?
                .0;
        }
    }

    fn clear(&self) {
        if let Ok(mut slot) = self.0 .0.lock() {
            *slot = None;
        }
    }
}

struct Ingress(Mailbox);

#[zbus::interface(name = "io.github.radislabus_star.LayDaemon.KWinFocus")]
impl Ingress {
    fn observe(
        &self,
        request: String,
        uuid: String,
        app: String,
        class: String,
        title: String,
        #[zbus(header)] header: zbus::message::Header<'_>,
    ) -> bool {
        let Some(sender) = header.sender() else {
            return false;
        };
        self.0.deliver(
            sender.as_str(),
            &request,
            RawWindow {
                uuid,
                app,
                class,
                title,
            },
            Instant::now(),
        )
    }
}

struct Cleanup {
    bus: String,
    owner: String,
    plugin: String,
}

#[derive(Default)]
struct Client {
    connection: Option<zbus::Connection>,
    lifetime: u64,
    mailbox: Mailbox,
    cleanup: Option<Cleanup>,
}

pub(super) fn read() -> Result<Observation, ReadError> {
    let mut client = match CLIENT
        .get_or_init(|| Mutex::new(Client::default()))
        .try_lock()
    {
        Ok(client) => client,
        Err(TryLockError::WouldBlock) => return Err(ReadError::Busy),
        Err(TryLockError::Poisoned(error)) => {
            return Err(ReadError::Unavailable(error.to_string()));
        }
    };
    client.read().map_err(ReadError::Unavailable)
}

#[cfg(test)]
pub(super) fn with_occupied_provider_for_test(f: impl FnOnce()) {
    let _client = CLIENT
        .get_or_init(|| Mutex::new(Client::default()))
        .lock()
        .expect("test provider lock");
    f();
}

impl Client {
    fn read(&mut self) -> Result<Observation, String> {
        let deadline = Instant::now() + READ_BUDGET;
        let result = self.read_until(deadline);
        self.mailbox.clear();
        if result.is_err() {
            // Do not load another script until the one cleanup obligation is discharged.
            if let Some(conn) = self.connection.clone() {
                let _ = self.cleanup_on(&conn, Instant::now() + CLEANUP_BUDGET);
            }
            // A fresh connection/lifetime is required after every failed observation.
            self.connection = None;
        }
        result
    }

    fn connect(&mut self, deadline: Instant) -> Result<zbus::Connection, String> {
        if let Some(conn) = &self.connection {
            return Ok(conn.clone());
        }
        let builder = zbus::connection::Builder::session()
            .map_err(|e| e.to_string())?
            .serve_at(PATH, Ingress(self.mailbox.clone()))
            .map_err(|e| e.to_string())?;
        let conn = bounded(deadline, builder.build())?.map_err(|e| e.to_string())?;
        self.lifetime = serial()?;
        self.connection = Some(conn.clone());
        Ok(conn)
    }

    fn read_until(&mut self, deadline: Instant) -> Result<Observation, String> {
        let runtime = PathBuf::from(
            std::env::var_os("XDG_RUNTIME_DIR").ok_or("XDG_RUNTIME_DIR unavailable")?,
        );
        self.read_in(deadline, &runtime)
    }

    fn read_in(
        &mut self,
        deadline: Instant,
        runtime: &std::path::Path,
    ) -> Result<Observation, String> {
        let conn = self.connect(deadline)?;
        self.cleanup_on(&conn, deadline)?;
        let owner = kwin_owner(&conn, deadline)?;
        let request = serial()?.to_string();
        let destination = conn
            .unique_name()
            .ok_or("session bus has no unique name")?
            .as_str();
        let plugin = format!("lay-exact-focus-{}", destination.replace([':', '.'], "_"));
        let script =
            RuntimeScript::create_in(runtime, &script_source(destination, &request), &request)?;
        self.mailbox.arm(&owner, &request, deadline)?;
        self.cleanup = Some(Cleanup {
            bus: conn.server_guid().to_string(),
            owner: owner.clone(),
            plugin: plugin.clone(),
        });
        let loaded = rpc(
            &conn,
            &owner,
            "/Scripting",
            "org.kde.kwin.Scripting",
            "loadScript",
            &(
                script.0.to_str().ok_or("non-UTF8 runtime script path")?,
                plugin.as_str(),
            ),
            deadline,
        )?;
        let id: i32 = loaded.body().deserialize().map_err(|e| e.to_string())?;
        if id < 0 {
            return Err("KWin refused foreground script load".into());
        }
        rpc(
            &conn,
            &owner,
            &format!("/{id}"),
            "org.kde.kwin.Script",
            "run",
            &(),
            deadline,
        )?;
        let window = self.mailbox.take(deadline)?;
        self.cleanup_on(&conn, deadline)?;
        if kwin_owner(&conn, deadline)? != owner {
            return Err("KWin owner changed during foreground read".into());
        }
        if Instant::now() >= deadline {
            return Err("KWin foreground read deadline exceeded".into());
        }
        Ok(observation(
            conn.server_guid(),
            &owner,
            self.lifetime,
            window,
        ))
    }

    fn cleanup_on(&mut self, conn: &zbus::Connection, deadline: Instant) -> Result<(), String> {
        let Some(cleanup) = &self.cleanup else {
            return Ok(());
        };
        if cleanup.bus == conn.server_guid().as_str() {
            let reply = rpc(
                conn,
                "org.freedesktop.DBus",
                "/org/freedesktop/DBus",
                "org.freedesktop.DBus",
                "NameHasOwner",
                &(cleanup.owner.as_str(),),
                deadline,
            )?;
            let alive: bool = reply.body().deserialize().map_err(|e| e.to_string())?;
            if alive {
                let reply = rpc(
                    conn,
                    &cleanup.owner,
                    "/Scripting",
                    "org.kde.kwin.Scripting",
                    "unloadScript",
                    &(cleanup.plugin.as_str(),),
                    deadline,
                )?;
                // false means already absent; either value discharges the obligation.
                let _: bool = reply.body().deserialize().map_err(|e| e.to_string())?;
            }
        }
        self.cleanup = None;
        Ok(())
    }
}

fn kwin_owner(conn: &zbus::Connection, deadline: Instant) -> Result<String, String> {
    let reply = rpc(
        conn,
        "org.freedesktop.DBus",
        "/org/freedesktop/DBus",
        "org.freedesktop.DBus",
        "GetNameOwner",
        &("org.kde.KWin",),
        deadline,
    )?;
    let owner: String = reply.body().deserialize().map_err(|e| e.to_string())?;
    zbus::names::UniqueName::try_from(owner.as_str()).map_err(|e| e.to_string())?;
    Ok(owner)
}

fn rpc<B: serde::ser::Serialize + zbus::zvariant::DynamicType>(
    conn: &zbus::Connection,
    destination: &str,
    path: &str,
    interface: &str,
    method: &str,
    body: &B,
    deadline: Instant,
) -> Result<zbus::Message, String> {
    bounded(
        deadline,
        conn.call_method(Some(destination), path, Some(interface), method, body),
    )?
    .map_err(|e| e.to_string())
}

fn bounded<F: Future>(deadline: Instant, future: F) -> Result<F::Output, String> {
    let mut future = pin!(future);
    let mut timer = pin!(async_io::Timer::at(deadline));
    async_io::block_on(poll_fn(|cx| {
        if Instant::now() >= deadline || timer.as_mut().poll(cx).is_ready() {
            return Poll::Ready(Err("KWin foreground read deadline exceeded".into()));
        }
        future.as_mut().poll(cx).map(Ok)
    }))
}

fn serial() -> Result<u64, String> {
    SERIAL
        .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |n| n.checked_add(1))
        .map_err(|_| "KWin foreground request serial exhausted".into())
}

fn observation(bus: &str, owner: &str, lifetime: u64, window: RawWindow) -> Observation {
    Observation {
        identity: format!("kde-window:{bus}:{owner}:{lifetime}:{}", window.uuid),
        info: serde_json::json!({ "appId": window.app, "wmClass": window.class, "title": window.title }).to_string(),
    }
}

fn script_source(destination: &str, request: &str) -> String {
    format!(
        "var layQuery = {};\n{}",
        serde_json::json!({
            "destination": destination, "request": request, "path": PATH, "interface": INTERFACE,
        }),
        include_str!("kde_focus/foreground.js")
    )
}

struct RuntimeScript(PathBuf);

impl RuntimeScript {
    fn create_in(runtime: &std::path::Path, source: &str, request: &str) -> Result<Self, String> {
        let metadata = runtime.metadata().map_err(|e| e.to_string())?;
        if !runtime.is_absolute()
            || !metadata.is_dir()
            || metadata.uid() != unsafe { libc::getuid() }
            || metadata.mode() & 0o077 != 0
        {
            return Err("KWin script requires private owned XDG_RUNTIME_DIR".into());
        }
        let path = runtime.join(format!(
            "lay-kwin-focus-{}-{request}.js",
            std::process::id()
        ));
        let mut file =
            lay::private_file::create_private_file_new(&path).map_err(|e| e.to_string())?;
        let script = Self(path);
        file.write_all(source.as_bytes())
            .map_err(|e| e.to_string())?;
        Ok(script)
    }
}

impl Drop for RuntimeScript {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.0);
    }
}

#[cfg(test)]
#[path = "kde_focus/tests.rs"]
mod tests;
