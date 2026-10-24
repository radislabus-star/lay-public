use super::{
    checked_snapshot, FocusedWindow, NativeFieldIdentity, NativeResult, NativeSnapshot,
    SurroundingTextSnapshot, MAX_APPLICATIONS, MAX_DOCUMENT_CHARS,
};
use std::sync::{Mutex, OnceLock};
use zbus::zvariant::{OwnedObjectPath, OwnedValue};
use zbus::{Connection, Message};

#[cfg(test)]
#[path = "bus/tests.rs"]
mod tests;

const ACCESSIBLE: &str = "org.a11y.atspi.Accessible";
const TEXT: &str = "org.a11y.atspi.Text";
const REGISTRY: &str = "org.a11y.atspi.Registry";
type AccessibleRef = (String, OwnedObjectPath);

#[derive(Default)]
struct NativeBusCache {
    // Transport only. Each read still discovers the current application/field
    // and each binding retains its own admission, focus, lease and epoch.
    connection: Mutex<Option<(String, Connection)>>,
}

impl NativeBusCache {
    async fn connect<F, Fut>(&self, address: &str, open: F) -> NativeResult<Connection>
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = NativeResult<Connection>>,
    {
        {
            let cache = self.connection.lock().map_err(|_| "native_bus_cache")?;
            if let Some((current, connection)) = cache.as_ref() {
                if current == address {
                    return Ok(connection.clone());
                }
            }
        }
        // Never hold a blocking mutex across I/O. Concurrent misses may build
        // a temporary connection, but publish and return only one winner.
        let connection = open().await?;
        let mut cache = self.connection.lock().map_err(|_| "native_bus_cache")?;
        if let Some((current, retained)) = cache.as_ref() {
            if current == address {
                return Ok(retained.clone());
            }
        }
        *cache = Some((address.to_owned(), connection.clone()));
        Ok(connection)
    }

    fn forget(&self, failed: &Connection) {
        if let Ok(mut cache) = self.connection.lock() {
            if cache
                .as_ref()
                .is_some_and(|(_, current)| std::ptr::eq(current.executor(), failed.executor()))
            {
                *cache = None;
            }
        }
    }
}

fn native_bus_cache() -> &'static NativeBusCache {
    static CACHE: OnceLock<NativeBusCache> = OnceLock::new();
    CACHE.get_or_init(NativeBusCache::default)
}

async fn call<B: serde::Serialize + zbus::zvariant::DynamicType + Sync>(
    connection: &Connection,
    owner: &str,
    path: &str,
    interface: &str,
    method: &str,
    body: &B,
) -> NativeResult<Message> {
    connection
        .call_method(Some(owner), path, Some(interface), method, body)
        .await
        .map_err(|error| {
            if matches!(&error, zbus::Error::InputOutput(io) if matches!(io.kind(),
                std::io::ErrorKind::BrokenPipe | std::io::ErrorKind::ConnectionReset
                | std::io::ErrorKind::ConnectionAborted | std::io::ErrorKind::NotConnected
                | std::io::ErrorKind::UnexpectedEof))
            {
                // Evict only the failed transport. No retry of this operation
                // or field/app identity caching follows a disconnected socket.
                native_bus_cache().forget(connection);
            }
            "native_rpc"
        })
}

async fn process_id(connection: &Connection, owner: &str) -> NativeResult<u32> {
    call(
        connection,
        "org.freedesktop.DBus",
        "/org/freedesktop/DBus",
        "org.freedesktop.DBus",
        "GetConnectionUnixProcessID",
        &(owner,),
    )
    .await?
    .body()
    .deserialize()
    .map_err(|_| "native_peer_pid")
}

async fn property(
    connection: &Connection,
    owner: &str,
    path: &str,
    name: &str,
) -> NativeResult<i32> {
    let value: OwnedValue = call(
        connection,
        owner,
        path,
        "org.freedesktop.DBus.Properties",
        "Get",
        &(TEXT, name),
    )
    .await?
    .body()
    .deserialize()
    .map_err(|_| "native_property")?;
    i32::try_from(value).map_err(|_| "native_property_type")
}

async fn selections(connection: &Connection, identity: &NativeFieldIdentity) -> NativeResult<i32> {
    call(
        connection,
        &identity.owner,
        identity.field.as_str(),
        TEXT,
        "GetNSelections",
        &(),
    )
    .await?
    .body()
    .deserialize()
    .map_err(|_| "native_selection_count")
}

async fn state(connection: &Connection, owner: &str, path: &str) -> NativeResult<()> {
    // These metadata queries are independent. Keep the privacy/focus checks
    // before reading any text, without spending two serial round trips.
    let (bits, role) = futures_lite::future::zip(
        call(connection, owner, path, ACCESSIBLE, "GetState", &()),
        call(connection, owner, path, ACCESSIBLE, "GetRole", &()),
    )
    .await;
    let bits: Vec<u32> = bits?.body().deserialize().map_err(|_| "native_states")?;
    let has = |n: usize| bits.get(n / 32).is_some_and(|v| v & (1 << (n % 32)) != 0);
    // AT-SPI EDITABLE=7, FOCUSED=12, DEFUNCT=6, STALE=27, READ_ONLY=43.
    if !has(7) || !has(12) || has(6) || has(27) || has(43) {
        return Err("native_field_state");
    }
    let role: u32 = role?.body().deserialize().map_err(|_| "native_role")?;
    if matches!(role, 40 | 60) {
        return Err("native_private_or_terminal");
    }
    Ok(())
}

async fn text_snapshot(
    connection: &Connection,
    identity: &NativeFieldIdentity,
) -> NativeResult<SurroundingTextSnapshot> {
    let owner = &identity.owner;
    let path = identity.field.as_str();
    // Both are metadata; privacy must pass before either text read. This also
    // applies when revalidating after a held character-geometry response.
    let (state, count) = futures_lite::future::zip(
        state(connection, owner, path),
        property(connection, owner, path, "CharacterCount"),
    )
    .await;
    state?;
    let count = count?;
    if !(0..=MAX_DOCUMENT_CHARS).contains(&count) {
        return Err("native_document_size");
    }
    let text: String = call(connection, owner, path, TEXT, "GetText", &(0i32, count))
        .await?
        .body()
        .deserialize()
        .map_err(|_| "native_text")?;
    let (caret, (selected, final_count)) = futures_lite::future::zip(
        property(connection, owner, path, "CaretOffset"),
        futures_lite::future::zip(
            selections(connection, identity),
            property(connection, owner, path, "CharacterCount"),
        ),
    )
    .await;
    if final_count? != count {
        return Err("native_document_changed");
    }
    let selected = selected?;
    if selected != 0 {
        return Err("native_selection_present");
    }
    checked_snapshot(text, count, caret?, selected)
}

async fn snapshot(
    connection: &Connection,
    identity: &NativeFieldIdentity,
    with_hint_geometry: bool,
) -> NativeResult<NativeSnapshot> {
    let owner = &identity.owner;
    let path = identity.field.as_str();
    let snapshot = text_snapshot(connection, identity).await?;
    let hint_rect = if with_hint_geometry {
        hint_rect(connection, identity, &snapshot).await
    } else {
        None
    };
    // Geometry may await while text, selection or caret changes. Revalidate
    // its actual document before the final field/privacy checks below.
    if hint_rect.is_some() && text_snapshot(connection, identity).await? != snapshot {
        return Err("native_document_changed");
    }
    // Focus may move between fields in the same Chrome window while the
    // awaited document reads run. The top-level KWin check cannot catch it.
    let (field, state) = futures_lite::future::zip(
        focused_field(connection, owner, &identity.application),
        state(connection, owner, path),
    )
    .await;
    if field? != identity.field {
        return Err("native_field_changed");
    }
    state?;
    Ok(NativeSnapshot {
        snapshot,
        hint_rect,
    })
}

// Querying real character bounds enables Chrome's lazy inline text boxes.
// An initial zero-sized result is unavailable, never a guessed caret. Later
// ordinary text notifications read the same focused field again; no timer or
// synthetic preedit lifecycle is needed to move an already committed prefix.
async fn hint_rect(
    connection: &Connection,
    identity: &NativeFieldIdentity,
    snapshot: &SurroundingTextSnapshot,
) -> Option<[i32; 4]> {
    let offset = i32::try_from(snapshot.cursor_pos.checked_sub(1)?).ok()?;
    let result = call(
        connection,
        &identity.owner,
        identity.field.as_str(),
        TEXT,
        "GetCharacterExtents",
        &(offset, 0u32), // AT-SPI SCREEN coordinates.
    )
    .await
    .ok()?;
    let (x, y, width, height): (i32, i32, i32, i32) = result.body().deserialize().ok()?;
    character_end_rect([x, y, width, height])
}

pub(super) fn character_end_rect([x, y, width, height]: [i32; 4]) -> Option<[i32; 4]> {
    if !(1..=4096).contains(&width)
        || !(1..=512).contains(&height)
        || x.abs_diff(0) > 100_000
        || y.abs_diff(0) > 100_000
    {
        return None;
    }
    let end = x.checked_add(width)?;
    (end.abs_diff(0) <= 100_000).then_some([end, y, 0, height])
}

async fn application_for_pid(connection: &Connection, pid: u32) -> NativeResult<AccessibleRef> {
    let applications: Vec<AccessibleRef> = call(
        connection,
        REGISTRY,
        "/org/a11y/atspi/accessible/root",
        ACCESSIBLE,
        "GetChildren",
        &(),
    )
    .await?
    .body()
    .deserialize()
    .map_err(|_| "native_applications")?;
    if applications.len() > MAX_APPLICATIONS {
        return Err("native_application_limit");
    }
    let mut matched = None;
    for (owner, path) in applications {
        // Only the bus-authenticated unique peer can identify an application.
        if zbus::names::UniqueName::try_from(owner.as_str()).is_err() {
            continue;
        }
        if process_id(connection, &owner).await? == pid {
            if matched.is_some() {
                return Err("native_ambiguous_application");
            }
            matched = Some((owner, path));
        }
    }
    matched.ok_or("native_application_missing")
}

pub(super) async fn read_focused(
    desktop: FocusedWindow,
    with_hint_geometry: bool,
) -> NativeResult<(Connection, NativeFieldIdentity, NativeSnapshot)> {
    let session = Connection::session().await.map_err(|error| {
        crate::trace::record(
            serde_json::json!({
                "kind": "ibus_native_document",
                "stage": "session_connect_error",
                "error": error.to_string(),
            })
            .to_string(),
        );
        "native_session_bus"
    })?;
    let address: String = call(
        &session,
        "org.a11y.Bus",
        "/org/a11y/bus",
        "org.a11y.Bus",
        "GetAddress",
        &(),
    )
    .await?
    .body()
    .deserialize()
    .map_err(|_| "native_bus_address")?;
    let connection = native_bus_cache()
        .connect(&address, || async {
            zbus::connection::Builder::address(address.as_str())
                .map_err(|_| "native_bus_address")?
                .build()
                .await
                .map_err(|_| "native_bus_connect")
        })
        .await?;
    let (owner, application) = application_for_pid(&connection, desktop.pid).await?;
    // A real application request enables lazy native accessibility bridges.
    let _ = call(
        &connection,
        &owner,
        application.as_str(),
        ACCESSIBLE,
        "GetAttributes",
        &(),
    )
    .await?;
    let field = focused_field(&connection, &owner, &application).await?;
    if process_id(&connection, &owner).await? != desktop.pid {
        return Err("native_field_peer");
    }
    let identity = NativeFieldIdentity {
        desktop,
        bus: connection.server_guid().to_string(),
        owner,
        application,
        field,
    };
    let actual = snapshot(&connection, &identity, with_hint_geometry).await?;
    Ok((connection, identity, actual))
}

async fn focused_field(
    connection: &Connection,
    owner: &str,
    application: &OwnedObjectPath,
) -> NativeResult<OwnedObjectPath> {
    let rule = (
        vec![((1u32 << 7) | (1u32 << 12)) as i32],
        1i32,
        std::collections::HashMap::<String, String>::new(),
        1i32,
        Vec::<i32>::new(),
        1i32,
        Vec::<String>::new(),
        1i32,
        false,
    );
    let matches: Vec<AccessibleRef> = call(
        connection,
        owner,
        application.as_str(),
        "org.a11y.atspi.Collection",
        "GetMatches",
        &(rule, 1u32, 2i32, true),
    )
    .await?
    .body()
    .deserialize()
    .map_err(|_| "native_focused_fields")?;
    let [(field_owner, field)] = matches.as_slice() else {
        crate::trace::record(
            serde_json::json!({
                "kind": "ibus_native_document", "stage": "focused_field_lookup",
                "candidates": matches.len().min(2),
            })
            .to_string(),
        );
        return Err("native_ambiguous_field");
    };
    if field_owner != owner || field.as_str() == "/org/a11y/atspi/null" {
        return Err("native_field_peer");
    }
    Ok(field.clone())
}

pub(super) async fn read_bound(
    connection: &Connection,
    identity: &NativeFieldIdentity,
    with_hint_geometry: bool,
) -> NativeResult<NativeSnapshot> {
    if connection.server_guid().as_str() != identity.bus {
        return Err("native_field_peer");
    }
    let (application, field) = futures_lite::future::zip(
        application_for_pid(connection, identity.desktop.pid),
        focused_field(connection, &identity.owner, &identity.application),
    )
    .await;
    if application? != (identity.owner.clone(), identity.application.clone())
        || field? != identity.field
    {
        return Err("native_field_peer");
    }
    snapshot(connection, identity, with_hint_geometry).await
}

// The two-argument ABI works with AT-SPI 2.44 and newer registries. The
// optional application argument was added later; older registries can return
// an empty reply without registering it. The exact peer/path match above
// continues to constrain which notifications may wake this binding.
pub(super) fn text_event_registration(event: &'static str) -> (&'static str, Vec<String>) {
    (event, Vec::new())
}

pub(super) async fn observe_text_changes(
    connection: &Connection,
    identity: &NativeFieldIdentity,
) -> NativeResult<zbus::MessageStream> {
    let rule = zbus::MatchRule::builder()
        .msg_type(zbus::message::Type::Signal)
        .sender(identity.owner.as_str())
        .map_err(|_| "native_signal_sender")?
        .path(identity.field.as_str())
        .map_err(|_| "native_signal_path")?
        .interface("org.a11y.atspi.Event.Object")
        .map_err(|_| "native_signal_interface")?
        .member("TextChanged")
        .map_err(|_| "native_signal_member")?
        .build();
    let events = zbus::MessageStream::for_match_rule(rule, connection, Some(8))
        .await
        .map_err(|_| "native_signal_subscribe")?;
    // Register on the adapter's own retained connection, as select_exact does.
    // No external observer or test flag is needed to activate the bridge.
    call(
        connection,
        REGISTRY,
        "/org/a11y/atspi/registry",
        REGISTRY,
        "RegisterEvent",
        &text_event_registration("object:text-changed"),
    )
    .await?;
    Ok(events)
}

async fn selected_range(
    connection: &Connection,
    identity: &NativeFieldIdentity,
) -> NativeResult<Option<(i32, i32)>> {
    if selections(connection, identity).await? != 1 {
        return Ok(None);
    }
    let range = call(
        connection,
        &identity.owner,
        identity.field.as_str(),
        TEXT,
        "GetSelection",
        &(0i32,),
    )
    .await?
    .body()
    .deserialize()
    .map_err(|_| "native_selected_range")?;
    Ok(Some(range))
}

pub(super) async fn select_exact(
    connection: &Connection,
    identity: &NativeFieldIdentity,
    expected: &SurroundingTextSnapshot,
    range: (i32, i32),
) -> NativeResult<()> {
    if &snapshot(connection, identity, false).await?.snapshot != expected {
        return Err("native_source_changed");
    }
    let rule = zbus::MatchRule::builder()
        .msg_type(zbus::message::Type::Signal)
        .sender(identity.owner.as_str())
        .map_err(|_| "native_signal_sender")?
        .path(identity.field.as_str())
        .map_err(|_| "native_signal_path")?
        .interface("org.a11y.atspi.Event.Object")
        .map_err(|_| "native_signal_interface")?
        .member("TextSelectionChanged")
        .map_err(|_| "native_signal_member")?
        .build();
    let mut events = zbus::MessageStream::for_match_rule(rule, connection, Some(1))
        .await
        .map_err(|_| "native_signal_subscribe")?;
    // This connection registers its own event interest; an external test
    // listener must never be a production precondition. Disconnect removes it.
    let _ = call(
        connection,
        REGISTRY,
        "/org/a11y/atspi/registry",
        REGISTRY,
        "RegisterEvent",
        &text_event_registration("object:text-selection-changed"),
    )
    .await?;
    let accepted: bool = call(
        connection,
        &identity.owner,
        identity.field.as_str(),
        TEXT,
        "AddSelection",
        &range,
    )
    .await?
    .body()
    .deserialize()
    .map_err(|_| "native_selection_reply")?;
    if !accepted {
        return Err("native_selection_rejected");
    }
    if selected_range(connection, identity).await? != Some(range) {
        let event = crate::context_admission::next_ordered_message(std::pin::Pin::new(&mut events))
            .await
            .ok_or("native_selection_stream_closed")?
            .map_err(|_| "native_selection_stream_error")?;
        drop(event);
        if selected_range(connection, identity).await? != Some(range) {
            return Err("native_selection_mismatch");
        }
    }
    state(connection, &identity.owner, identity.field.as_str()).await?;
    let actual: String = call(
        connection,
        &identity.owner,
        identity.field.as_str(),
        TEXT,
        "GetText",
        &(0i32, MAX_DOCUMENT_CHARS),
    )
    .await?
    .body()
    .deserialize()
    .map_err(|_| "native_selected_text")?;
    if actual != expected.text {
        return Err("native_selected_source_changed");
    }
    Ok(())
}
