use super::*;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

struct Document {
    text: String,
    caret: i32,
    selected: i32,
    field: OwnedObjectPath,
    private: bool,
    hold_geometry: bool,
    hold_text: bool,
    geometry_calls: u32,
    private_text_reads: u32,
}

struct Text(Arc<Mutex<Document>>);

#[zbus::interface(name = "org.a11y.atspi.Text")]
impl Text {
    #[zbus(property)]
    fn character_count(&self) -> i32 {
        self.0.lock().unwrap().text.chars().count() as i32
    }

    #[zbus(property)]
    fn caret_offset(&self) -> i32 {
        self.0.lock().unwrap().caret
    }

    async fn get_text(&self, start: i32, end: i32) -> String {
        while self.0.lock().unwrap().hold_text {
            async_io::Timer::after(Duration::from_millis(1)).await;
        }
        let mut state = self.0.lock().unwrap();
        if state.private {
            state.private_text_reads += 1;
        }
        state
            .text
            .chars()
            .skip(start as usize)
            .take((end - start) as usize)
            .collect()
    }

    fn get_n_selections(&self) -> i32 {
        self.0.lock().unwrap().selected
    }

    async fn get_character_extents(&self, _offset: i32, _coord: u32) -> (i32, i32, i32, i32) {
        self.0.lock().unwrap().geometry_calls += 1;
        while self.0.lock().unwrap().hold_geometry {
            async_io::Timer::after(Duration::from_millis(1)).await;
        }
        (67, 285, 11, 21)
    }
}

struct Accessible(Arc<Mutex<Document>>);

#[zbus::interface(name = "org.a11y.atspi.Accessible")]
impl Accessible {
    fn get_state(&self) -> Vec<u32> {
        vec![(1 << 7) | (1 << 12), 0]
    }

    fn get_role(&self) -> u32 {
        if self.0.lock().unwrap().private {
            40
        } else {
            61
        }
    }
}

struct Collection(Arc<Mutex<Document>>);

type MatchRule = (
    Vec<i32>,
    i32,
    std::collections::HashMap<String, String>,
    i32,
    Vec<i32>,
    i32,
    Vec<String>,
    i32,
    bool,
);

#[zbus::interface(name = "org.a11y.atspi.Collection")]
impl Collection {
    fn get_matches(
        &self,
        _rule: MatchRule,
        _order: u32,
        _count: i32,
        _traverse: bool,
    ) -> Vec<AccessibleRef> {
        vec![(":1.1".into(), self.0.lock().unwrap().field.clone())]
    }
}

fn peer() -> (
    Connection,
    Connection,
    NativeFieldIdentity,
    Arc<Mutex<Document>>,
) {
    let document = Arc::new(Mutex::new(Document {
        text: "пров".into(),
        caret: 4,
        selected: 0,
        field: "/field".try_into().unwrap(),
        private: false,
        hold_geometry: true,
        hold_text: false,
        geometry_calls: 0,
        private_text_reads: 0,
    }));
    let shared = document.clone();
    let (server_socket, client_socket) = std::os::unix::net::UnixStream::pair().unwrap();
    let guid = zbus::Guid::generate();
    let server = std::thread::spawn(move || {
        zbus::block_on(
            zbus::connection::Builder::unix_stream(server_socket)
                .server(guid)
                .unwrap()
                .p2p()
                .serve_at("/field", Text(shared.clone()))
                .unwrap()
                .serve_at("/field", Accessible(shared.clone()))
                .unwrap()
                .serve_at("/application", Collection(shared))
                .unwrap()
                .build(),
        )
        .unwrap()
    });
    let client = zbus::block_on(
        zbus::connection::Builder::unix_stream(client_socket)
            .p2p()
            .build(),
    )
    .unwrap();
    let server = server.join().unwrap();
    let identity = NativeFieldIdentity {
        desktop: FocusedWindow {
            identity: "geometry-test".into(),
            pid: 1,
        },
        bus: client.server_guid().to_string(),
        owner: ":1.1".into(),
        application: "/application".try_into().unwrap(),
        field: "/field".try_into().unwrap(),
    };
    (server, client, identity, document)
}

#[test]
fn native_document_event_burst_keeps_read_receipts_live_while_engine_is_busy() {
    use std::sync::atomic::{AtomicBool, Ordering};

    let (server, client, identity, document) = peer();
    let rule = zbus::MatchRule::builder()
        .msg_type(zbus::message::Type::Signal)
        .path("/field")
        .unwrap()
        .interface("org.a11y.atspi.Event.Object")
        .unwrap()
        .member("TextChanged")
        .unwrap()
        .build();
    let events =
        zbus::block_on(zbus::MessageStream::for_match_rule(rule, &client, Some(8))).unwrap();
    let busy = Arc::new(AtomicBool::new(false));
    let entered = busy.clone();
    let observer = client.executor().spawn(
        super::super::observer::watch_changes(events, move || {
            entered.store(true, Ordering::Release);
            std::future::pending::<bool>()
        }),
        "native-event-burst-regression",
    );
    let signal = || {
        server.emit_signal(
            None::<&str>,
            "/field",
            "org.a11y.atspi.Event.Object",
            "TextChanged",
            &(),
        )
    };
    let result = zbus::block_on(async {
        signal().await.unwrap();
        super::super::bounded(Instant::now() + Duration::from_millis(250), async {
            while !busy.load(Ordering::Acquire) {
                async_io::Timer::after(Duration::from_millis(1)).await;
            }
            Ok(())
        })
        .await
        .unwrap();
        // More than the wire stream capacity, while the engine-side refresh
        // remains blocked. Notifications are wakeups, never document proof.
        for _ in 0..64 {
            signal().await.unwrap();
        }
        {
            let mut actual = document.lock().unwrap();
            actual.text = "свежий".into();
            actual.caret = 6;
        }
        super::super::bounded(
            Instant::now() + Duration::from_millis(250),
            snapshot(&client, &identity, false),
        )
        .await
    });
    drop(observer);
    assert_eq!(result.unwrap().snapshot.text, "свежий");
}

#[test]
fn native_document_bus_reuses_transport_but_reads_fresh_document_data() {
    use std::sync::atomic::{AtomicUsize, Ordering};
    let (_server, client, identity, document) = peer();
    let cache = NativeBusCache::default();
    let opens = AtomicUsize::new(0);
    zbus::block_on(async {
        for _ in 0..128 {
            let retained = cache
                .connect("bus-a", || async {
                    opens.fetch_add(1, Ordering::SeqCst);
                    Ok(client.clone())
                })
                .await
                .unwrap();
            assert!(std::ptr::eq(retained.executor(), client.executor()));
        }
        assert_eq!(opens.load(Ordering::SeqCst), 1);
        let first = text_snapshot(&client, &identity).await.unwrap();
        assert_eq!(first.text, "пров");
        document.lock().unwrap().text = "друг".into();
        let retained = cache
            .connect("bus-a", || async { Err("unexpected_open") })
            .await
            .unwrap();
        assert_eq!(
            text_snapshot(&retained, &identity).await.unwrap().text,
            "друг"
        );
    });
}

#[test]
fn native_document_bus_concurrent_openers_publish_only_one_transport() {
    use std::sync::atomic::{AtomicUsize, Ordering};
    let (_server_a, client_a, _, _) = peer();
    let (_server_b, client_b, _, _) = peer();
    let cache = NativeBusCache::default();
    let opens = AtomicUsize::new(0);
    let (first, second) = zbus::block_on(futures_lite::future::zip(
        cache.connect("same-address", || async {
            opens.fetch_add(1, Ordering::SeqCst);
            futures_lite::future::yield_now().await;
            Ok(client_a.clone())
        }),
        cache.connect("same-address", || async {
            opens.fetch_add(1, Ordering::SeqCst);
            futures_lite::future::yield_now().await;
            Ok(client_b.clone())
        }),
    ));
    assert_eq!(opens.load(Ordering::SeqCst), 2);
    let first = first.unwrap();
    let second = second.unwrap();
    assert!(std::ptr::eq(first.executor(), second.executor()));
    let retained =
        zbus::block_on(cache.connect("same-address", || async { Err("unexpected_open") })).unwrap();
    assert!(std::ptr::eq(first.executor(), retained.executor()));
}

#[test]
fn native_document_bus_cancel_failure_and_old_disconnect_cannot_poison_new_transport() {
    let (_server_a, client_a, _, _) = peer();
    let (_server_b, client_b, _, _) = peer();
    let cache = NativeBusCache::default();
    assert!(
        zbus::block_on(cache.connect("bus-a", || async { Err("native_bus_connect") })).is_err()
    );
    let mut cancelled = Box::pin(cache.connect("bus-a", || async {
        let prepared = client_a.clone();
        std::future::pending::<()>().await;
        Ok(prepared)
    }));
    assert!(zbus::block_on(futures_lite::future::poll_once(cancelled.as_mut())).is_none());
    drop(cancelled);
    let first = zbus::block_on(cache.connect("bus-a", || async { Ok(client_a.clone()) })).unwrap();
    assert!(std::ptr::eq(first.executor(), client_a.executor()));
    assert!(
        zbus::block_on(cache.connect("bus-b", || async { Err("native_bus_connect") })).is_err()
    );
    let still_a =
        zbus::block_on(cache.connect("bus-a", || async { Err("unexpected_open") })).unwrap();
    assert!(std::ptr::eq(first.executor(), still_a.executor()));
    let second = zbus::block_on(cache.connect("bus-b", || async { Ok(client_b.clone()) })).unwrap();
    assert!(!std::ptr::eq(first.executor(), second.executor()));
    cache.forget(&first);
    let still_b =
        zbus::block_on(cache.connect("bus-b", || async { Err("unexpected_open") })).unwrap();
    assert!(std::ptr::eq(second.executor(), still_b.executor()));
    cache.forget(&second);
    assert!(
        zbus::block_on(cache.connect("bus-b", || async { Err("fresh_open_requested") })).is_err()
    );
}

#[test]
fn native_document_edit_snapshot_never_requests_presentation_geometry() {
    let (_server, client, identity, document) = peer();
    let actual = zbus::block_on(super::super::bounded(
        Instant::now() + Duration::from_millis(250),
        snapshot(&client, &identity, false),
    ))
    .unwrap();
    assert_eq!(
        actual.snapshot,
        SurroundingTextSnapshot::new("пров".into(), 4, 4)
    );
    assert_eq!(actual.hint_rect, None);
    assert_eq!(document.lock().unwrap().geometry_calls, 0);

    // A required text operation still has the original bounded failure,
    // without trying geometry or retrying the read.
    document.lock().unwrap().hold_text = true;
    let actual = zbus::block_on(super::super::bounded(
        Instant::now() + Duration::from_millis(20),
        snapshot(&client, &identity, false),
    ));
    document.lock().unwrap().hold_text = false;
    assert_eq!(actual.err(), Some("native_deadline"));
    assert_eq!(document.lock().unwrap().geometry_calls, 0);
}

#[test]
fn native_document_geometry_pending_does_not_extend_the_observation_deadline() {
    let (_server, client, identity, document) = peer();
    let actual = zbus::block_on(super::super::bounded(
        Instant::now() + Duration::from_millis(20),
        snapshot(&client, &identity, true),
    ));
    document.lock().unwrap().hold_geometry = false;
    assert_eq!(actual.err(), Some("native_deadline"));
    assert_eq!(document.lock().unwrap().geometry_calls, 1);
}

#[test]
fn native_document_geometry_cannot_outlive_field_privacy_or_document_checks() {
    for change in ["none", "field", "privacy", "text", "caret", "selection"] {
        let (_server, client, identity, document) = peer();
        let actual = zbus::block_on(async {
            let mutate = async {
                let deadline = Instant::now() + Duration::from_millis(200);
                while document.lock().unwrap().geometry_calls == 0 && Instant::now() < deadline {
                    async_io::Timer::after(Duration::from_millis(1)).await;
                }
                let mut state = document.lock().unwrap();
                assert_eq!(
                    state.geometry_calls, 1,
                    "held extents request must actually start"
                );
                match change {
                    "field" => state.field = "/other".try_into().unwrap(),
                    "privacy" => state.private = true,
                    "text" => state.text = "прав".into(),
                    "caret" => state.caret = 3,
                    "selection" => state.selected = 1,
                    "none" => (),
                    _ => unreachable!(),
                }
                state.hold_geometry = false;
            };
            let (actual, ()) = futures_lite::future::zip(
                super::super::bounded(
                    Instant::now() + Duration::from_millis(250),
                    snapshot(&client, &identity, true),
                ),
                mutate,
            )
            .await;
            actual
        });
        match change {
            "none" => assert_eq!(actual.unwrap().hint_rect, Some([78, 285, 0, 21])),
            "field" => assert_eq!(actual.err(), Some("native_field_changed")),
            "privacy" => assert_eq!(actual.err(), Some("native_private_or_terminal")),
            "selection" => assert_eq!(actual.err(), Some("native_selection_present")),
            _ => assert_eq!(actual.err(), Some("native_document_changed"), "{change}"),
        }
        assert_eq!(document.lock().unwrap().private_text_reads, 0, "{change}");
    }
}
