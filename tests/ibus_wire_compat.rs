//! Stock IBus is a bus daemon: its lifecycle signals use its reserved service
//! name as Sender, while engine method callbacks use org.freedesktop.DBus.
use zbus::{names::UniqueName, zvariant::serialized::Data, Message};

#[test]
fn stock_ibus_lifecycle_wire_header_keeps_sender_and_payload() {
    for profile in ["lay-ime-us", "lay-ime-ru", "xkb:us::eng"] {
        let message = Message::signal(
            "/org/freedesktop/IBus",
            "org.freedesktop.IBus",
            "GlobalEngineChanged",
        )
        .unwrap()
        .sender("org.freedesktop.DBus")
        .unwrap()
        .build(&profile)
        .unwrap();
        let mut wire = message.data().to_vec();
        // The two reserved sender names have the same wire length. Keep the
        // frame, offsets and original sender exactly as captured on IBus 1.5.26.
        let needle = b"org.freedesktop.DBus";
        let offsets: Vec<_> = wire
            .windows(needle.len())
            .enumerate()
            .filter_map(|(i, bytes)| (bytes == needle).then_some(i))
            .collect();
        assert_eq!(offsets.len(), 1);
        wire[offsets[0]..offsets[0] + needle.len()].copy_from_slice(b"org.freedesktop.IBus");
        let bytes = Data::new(wire, message.data().context());
        // SAFETY: a library-built valid frame; only the equal-length sender
        // was replaced with the reserved sender emitted by stock IBus.
        let received = unsafe { Message::from_bytes(bytes) }.unwrap();
        let header = received.header();
        assert_eq!(header.sender().unwrap().as_str(), "org.freedesktop.IBus");
        assert_eq!(header.path().unwrap().as_str(), "/org/freedesktop/IBus");
        assert_eq!(header.interface().unwrap().as_str(), "org.freedesktop.IBus");
        assert_eq!(header.member().unwrap().as_str(), "GlobalEngineChanged");
        assert_eq!(received.body().deserialize::<String>().unwrap(), profile);
    }
}

#[test]
fn compatibility_preserves_both_reserved_bus_identities_and_unique_clients() {
    for name in ["org.freedesktop.IBus", "org.freedesktop.DBus", ":1.23"] {
        assert_eq!(UniqueName::try_from(name).unwrap().as_str(), name);
    }
}

#[test]
fn compatibility_does_not_admit_arbitrary_services_or_similar_names() {
    for name in [
        "org.example.Client",
        "org.freedesktop.IBus.Fake",
        "org.freedesktop.IBusx",
        "org.freedesktop.ibus",
        "org.freedesktop.DBus.Fake",
        ":invalid",
        "",
    ] {
        assert!(UniqueName::try_from(name).is_err(), "accepted {name}");
    }
}
