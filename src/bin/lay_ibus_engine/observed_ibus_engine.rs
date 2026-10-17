//! Fresh observational read on the bridge-owned IBus connection; no input state.
use std::sync::atomic::{AtomicBool, Ordering};

use zbus::zvariant::{OwnedValue, Value};
use zbus::{fdo, Connection};

// A timed-out session-bus caller does not cancel the detached server method.
// Allow at most one backend request; others retain the daemon's original CLI.
static IN_FLIGHT: AtomicBool = AtomicBool::new(false);
struct InFlight;
impl Drop for InFlight {
    fn drop(&mut self) {
        IN_FLIGHT.store(false, Ordering::Release);
    }
}

pub(crate) async fn read(connection: &Connection) -> fdo::Result<String> {
    if IN_FLIGHT
        .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
        .is_err()
    {
        return Err(fdo::Error::Failed(
            "IBus engine observation already pending".into(),
        ));
    }
    let _in_flight = InFlight;
    // Never cancel call_method while it may be sending a partial frame on the
    // shared input connection. The existing daemon RPC bounds its reply wait.
    let reply = connection
        .call_method(
            Some("org.freedesktop.IBus"),
            "/org/freedesktop/IBus",
            Some("org.freedesktop.IBus"),
            "GetGlobalEngine",
            &(),
        )
        .await
        .map_err(|error| fdo::Error::Failed(error.to_string()))?;
    let value = reply
        .body()
        .deserialize::<OwnedValue>()
        .map_err(|error| fdo::Error::Failed(error.to_string()))?;
    decode(&value)
}

#[cfg(test)]
pub(crate) fn test_read_in_flight() -> bool {
    IN_FLIGHT.load(Ordering::Acquire)
}

fn decode(value: &Value<'_>) -> fdo::Result<String> {
    let Value::Structure(desc) = value else {
        return Err(fdo::Error::Failed(
            "IBus engine descriptor is not a structure".into(),
        ));
    };
    let [Value::Str(tag), Value::Dict(attachments), Value::Str(name), ..] = desc.fields() else {
        return Err(fdo::Error::Failed(
            "IBus engine descriptor prefix is malformed".into(),
        ));
    };
    if tag.as_str() != "IBusEngineDesc"
        || attachments.signature().to_string() != "a{sv}"
        || name.as_str().is_empty()
    {
        return Err(fdo::Error::Failed(
            "IBus engine descriptor tag/name is invalid".into(),
        ));
    }
    Ok(name.as_str().to_owned())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use zbus::zvariant::Structure;

    fn descriptor(tag: &str, name: &str) -> Value<'static> {
        Value::from(Structure::from((
            tag.to_string(),
            HashMap::<String, OwnedValue>::new(),
            name.to_string(),
        )))
    }

    #[test]
    fn descriptor_accepts_actual_name_and_trailing_extension_fields() {
        assert_eq!(
            decode(&descriptor("IBusEngineDesc", "lay-ime-ru")).unwrap(),
            "lay-ime-ru"
        );
        let extended = Value::from(Structure::from((
            "IBusEngineDesc",
            HashMap::<String, OwnedValue>::new(),
            "xkb:us::eng",
            "extra",
        )));
        assert_eq!(decode(&extended).unwrap(), "xkb:us::eng");
    }

    #[test]
    fn descriptor_rejects_wrong_type_tag_prefix_and_empty_name() {
        let wrong_dict = Value::from(Structure::from((
            "IBusEngineDesc",
            HashMap::<u32, u32>::new(),
            "lay-ime-ru",
        )));
        for value in [
            wrong_dict,
            Value::from("lay-ime-ru"),
            descriptor("IBusText", "lay-ime-ru"),
            descriptor("IBusEngineDesc", ""),
            Value::from(Structure::from(("IBusEngineDesc", 0u32, "lay-ime-ru"))),
        ] {
            assert!(decode(&value).is_err());
        }
    }
}
