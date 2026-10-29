//! Execute only in the owned Xvfb test server, never a user's desktop.
use super::*;
use x11rb::protocol::xproto::{CreateWindowAux, EventMask as CoreMask, WindowClass};

#[test]
fn native_x11_lifecycle_accepts_only_owned_changed_hierarchy_rows() {
    let stable = xinput::HierarchyInfo {
        deviceid: 3,
        ..Default::default()
    };
    let owned = xinput::HierarchyInfo {
        deviceid: 9,
        flags: xinput::HierarchyMask::MASTER_ADDED,
        ..Default::default()
    };
    let event = |infos| {
        Event::XinputHierarchy(xinput::HierarchyEvent {
            infos,
            ..Default::default()
        })
    };
    assert!(own_hierarchy(&event(vec![stable, owned]), &[9, 10, 11, 12]));
    assert!(!own_hierarchy(&event(vec![stable]), &[9, 10, 11, 12]));
    assert!(!own_hierarchy(&event(vec![]), &[9, 10, 11, 12]));
    let foreign = xinput::HierarchyInfo {
        deviceid: 4,
        flags: xinput::HierarchyMask::SLAVE_ATTACHED,
        ..Default::default()
    };
    assert!(!own_hierarchy(
        &event(vec![stable, owned, foreign]),
        &[9, 10, 11, 12]
    ));
}

#[test]
#[ignore = "requires dedicated owned Xvfb"]
fn native_x11_idle_does_not_displace_clients_single_master_keyboard() {
    let f = Fixture::new();
    let masters: Vec<_> = f
        .app
        .xinput_xi_query_device(0u16)
        .unwrap()
        .reply()
        .unwrap()
        .infos
        .into_iter()
        .filter(|d| d.type_ == DeviceType::MASTER_KEYBOARD)
        .collect();
    assert_eq!(
        masters.len(),
        1,
        "an idle observer must not replace a client's selected master"
    );
    f.pair(38);
    assert_eq!(
        f.app_keys().len(),
        2,
        "ordinary input must remain untouched"
    );
}

#[test]
#[ignore = "requires dedicated owned Xvfb"]
fn native_x11_output_event_flood_rejects_and_thaws_original_input() {
    let mut f = Fixture::new();
    f.input.grab().unwrap();
    {
        let replay = f.input.replay.lock().unwrap();
        let window = replay.conn.generate_id().unwrap();
        replay
            .conn
            .create_window(
                0,
                window,
                replay.root,
                0,
                0,
                1,
                1,
                0,
                WindowClass::INPUT_ONLY,
                0,
                &CreateWindowAux::new(),
            )
            .unwrap()
            .check()
            .unwrap();
        for _ in 0..4100 {
            let event = xproto::ClientMessageEvent::new(32, window, 0u32, [0u32; 5]);
            f.app
                .send_event(false, window, CoreMask::NO_EVENT, event)
                .unwrap()
                .check()
                .unwrap();
        }
        replay.conn.get_input_focus().unwrap().reply().unwrap();
    }
    let error = f
        .output
        .emit(&[InputEvent::new(EventType::KEY.0, KeyCode::KEY_A.code(), 1)])
        .unwrap_err();
    assert!(
        error.to_string().contains("event budget exceeded"),
        "{error}"
    );
    assert!(f.app_keys().is_empty(), "a rejected frame emitted a press");
    f.input.ungrab().unwrap();
    f.pair(38);
    assert_eq!(
        f.app_keys().len(),
        2,
        "rejected output left original input frozen"
    );
}

#[test]
#[ignore = "requires dedicated owned Xvfb"]
fn native_x11_replay_master_preserves_layout_metadata() {
    let mut f = Fixture::new();
    let root = f.app.setup().roots[0].root;
    let atom = f
        .app
        .intern_atom(false, b"_XKB_RULES_NAMES")
        .unwrap()
        .reply()
        .unwrap()
        .atom;
    let read = || {
        let property = f
            .app
            .get_property(false, root, atom, xproto::AtomEnum::ANY, 0, 16_384)
            .unwrap()
            .reply()
            .unwrap();
        (property.type_, property.format, property.value)
    };
    let original = read();
    assert!(original.2.windows(5).any(|w| w == b"us,ru"));
    // A desktop reads this root property, independently of the actual core map.
    // An owned output device must not advertise its default map as session state.
    for absent in [false, true] {
        if absent {
            f.app.delete_property(root, atom).unwrap().check().unwrap();
        }
        let before = read();
        f.input.grab().unwrap();
        let during = read();
        f.input.ungrab().unwrap();
        let after = read();
        assert_eq!(
            during, before,
            "owned activation changed global layout metadata"
        );
        assert_eq!(
            after, before,
            "owned retirement changed global layout metadata"
        );
    }
    f.pair(38);
    assert_eq!(
        f.app_keys().len(),
        2,
        "original input still works after retirement"
    );
}

fn click(f: &Fixture) {
    for kind in [4, 5] {
        xtest::fake_input(&f.source, kind, 1, x11rb::CURRENT_TIME, 0, 0, 0, 0)
            .unwrap()
            .check()
            .unwrap();
    }
}

#[test]
#[ignore = "requires dedicated owned Xvfb"]
fn native_x11_pointer_fence_retires_only_pre_click_observations() {
    let mut f = Fixture::new();
    f.observed();
    for _ in 0..2 {
        f.pair(50);
    }
    click(&f);
    f.pair(38);
    let observed = f.observed();
    assert_eq!(observed.len(), 2);
    assert!(observed.iter().all(|e| e.code() == KeyCode::KEY_A.code()));
    let delivered = f.app_keys();
    assert_eq!(
        delivered.len(),
        6,
        "the fence must not discard server delivery"
    );
    assert_eq!(delivered.iter().filter(|e| e.0 == 50).count(), 4);
}

#[test]
#[ignore = "requires dedicated owned Xvfb"]
fn native_x11_pointer_fence_retires_acquisition_pending_prefix() {
    let mut f = Fixture::new();
    f.pair(50);
    let pending = f.observed();
    assert_eq!(pending.len(), 2);
    f.input.pending.extend(pending);
    click(&f);
    f.pair(38);
    let observed = f.observed();
    assert_eq!(observed.len(), 2);
    assert!(observed.iter().all(|e| e.code() == KeyCode::KEY_A.code()));
    assert_eq!(f.app_keys().len(), 4);
}

#[test]
#[ignore = "requires dedicated owned Xvfb"]
fn native_x11_pointer_during_acquisition_rejects_empty_key_batch() {
    let mut f = Fixture::new();
    f.observed();
    click(&f);
    assert!(
        f.input.grab().is_err(),
        "a same-window pointer fence must revoke acquisition"
    );
    assert!(!f.input.grabbed);
    assert!(f.input.pending.is_empty());
    f.pair(38);
    assert_eq!(
        f.app_keys().len(),
        2,
        "a rejected acquisition must release the original master"
    );
}

fn enable_overlay(f: &Fixture, device: u16) {
    let c = xkb::get_controls(&f.app, device).unwrap().reply().unwrap();
    xkb::set_controls(
        &f.app,
        device,
        xproto::ModMask::default(),
        xproto::ModMask::default(),
        xproto::ModMask::default(),
        xproto::ModMask::default(),
        xkb::VMod::default(),
        xkb::VMod::default(),
        xkb::VMod::default(),
        xkb::VMod::default(),
        c.mouse_keys_dflt_btn,
        c.groups_wrap,
        c.access_x_option,
        xkb::BoolCtrl::OVERLAY1_MASK,
        xkb::BoolCtrl::OVERLAY1_MASK,
        xkb::Control::CONTROLS_ENABLED,
        c.repeat_delay,
        c.repeat_interval,
        c.slow_keys_delay,
        c.debounce_delay,
        c.mouse_keys_delay,
        c.mouse_keys_interval,
        c.mouse_keys_time_to_max,
        c.mouse_keys_max_speed,
        c.mouse_keys_curve,
        c.access_x_timeout,
        c.access_x_timeout_mask,
        c.access_x_timeout_values,
        c.access_x_timeout_options_mask,
        c.access_x_timeout_options_values,
        &c.per_key_repeat,
    )
    .unwrap()
    .check()
    .unwrap();
    let current = xkb::get_controls(&f.app, device).unwrap().reply().unwrap();
    assert!(current
        .enabled_controls
        .contains(xkb::BoolCtrl::OVERLAY1_MASK));
}

#[test]
#[ignore = "requires dedicated owned Xvfb"]
fn native_x11_enabled_interpreter_control_mismatch_rejects_admission() {
    let mut f = Fixture::new();
    f.input.grab().unwrap();
    let device = u16::from(f.input_replay_source());
    enable_overlay(&f, device);
    assert!(mapping::Maps::new(
        &f.app,
        [
            f.input.core,
            capability().unwrap().input,
            f.input_replay_keyboard(),
            device
        ]
    )
    .is_err());
    assert!(f.app_keys().is_empty());
}

#[test]
fn native_x11_session_admission_is_independent_of_layout_controller() {
    for desktop in ["KDE", "GNOME", "XFCE"] {
        let backend =
            lay::desktop::resolve_layout_backend("auto", Some(desktop), None, Some("x11"));
        assert!(
            is_x11_session(Some(":71"), Some("x11"), None),
            "{backend:?}"
        );
    }
    assert!(is_x11_session(Some(":71"), None, None));
    assert!(!is_x11_session(
        Some(":71"),
        Some("wayland"),
        Some("wayland-0")
    ));
    assert!(!is_x11_session(None, Some("x11"), None));
    assert!(!is_x11_session(Some(":71"), Some("wayland"), None));
}

struct Fixture {
    app: RustConnection,
    source: RustConnection,
    window: u32,
    input: Input,
    output: Output,
}

#[test]
#[ignore = "requires dedicated owned Xvfb"]
fn native_x11_shared_manual_dispatch_isolates_replay_and_thaws_original_input() {
    use crate::physical_input_grab::PhysicalInputGrab;
    use crate::{
        ManualCorrectionDispatchPlan, ManualCorrectionInputIsolation, ManualCorrectionOutputRoute,
    };

    let Fixture {
        app,
        source,
        input,
        mut output,
        ..
    } = Fixture::new();
    let mut device = crate::native_input::Device::X11(Box::new(input));
    let plan = ManualCorrectionDispatchPlan {
        output_route: ManualCorrectionOutputRoute::ConfiguredBackend,
        input_isolation: ManualCorrectionInputIsolation::Shared,
    };
    let required = plan.requires_input_isolation(&device);
    let grab = PhysicalInputGrab::new(required.then_some(&mut device));
    assert!(
        grab.is_active(),
        "remote manual dispatch must isolate native replay"
    );
    output
        .emit(&[
            InputEvent::new(EventType::KEY.0, KeyCode::KEY_C.code(), 1),
            InputEvent::new(EventType::KEY.0, KeyCode::KEY_C.code(), 0),
        ])
        .unwrap();
    assert_eq!(
        Fixture::app_keys_on(&app)
            .iter()
            .map(|event| (event.0, event.1))
            .collect::<Vec<_>>(),
        [(54, true), (54, false)]
    );
    drop(grab);
    for event_type in [2, 3] {
        xtest::fake_input(&source, event_type, 38, x11rb::CURRENT_TIME, 0, 0, 0, 0)
            .unwrap()
            .check()
            .unwrap();
    }
    assert_eq!(
        Fixture::app_keys_on(&app)
            .iter()
            .map(|event| (event.0, event.1))
            .collect::<Vec<_>>(),
        [(38, true), (38, false)],
        "original keyboard must resume after the edit"
    );
}

impl Fixture {
    fn input_replay(&self) -> std::sync::MutexGuard<'_, replay_master::ReplayMaster> {
        self.input.replay.lock().unwrap()
    }
    fn input_replay_keyboard(&self) -> u16 {
        self.input_replay().keyboard
    }
    fn input_replay_source(&self) -> u8 {
        self.input_replay().source
    }
    fn new() -> Self {
        assert_eq!(
            std::env::var("LAY_OWNED_X11_TEST").as_deref(),
            Ok("1"),
            "requires an owned Xvfb"
        );
        // Keep a connection alive while installing the map: Xvfb resets its
        // default map when its last client disconnects.
        let (app, screen) = x11rb::connect(None).unwrap();
        // Each test establishes its own map; the revocation test intentionally
        // changes the same disposable server and must not leak setup to others.
        assert!(std::process::Command::new("setxkbmap")
            .args(["-layout", "us,ru"])
            .status()
            .unwrap()
            .success());
        xkb::use_extension(&app, 1, 0).unwrap().reply().unwrap();
        let root = app.setup().roots[screen].root;
        let window = app.generate_id().unwrap();
        app.create_window(
            x11rb::COPY_DEPTH_FROM_PARENT,
            window,
            root,
            0,
            0,
            200,
            100,
            0,
            WindowClass::INPUT_OUTPUT,
            0,
            &CreateWindowAux::new().event_mask(CoreMask::KEY_PRESS | CoreMask::KEY_RELEASE),
        )
        .unwrap()
        .check()
        .unwrap();
        app.map_window(window).unwrap().check().unwrap();
        app.set_input_focus(xproto::InputFocus::PARENT, window, x11rb::CURRENT_TIME)
            .unwrap()
            .check()
            .unwrap();
        let (source, _) = x11rb::connect(None).unwrap();
        xkb::use_extension(&source, 1, 0).unwrap().reply().unwrap();
        let (input, output) = open(Arc::new(AtomicU64::new(1))).unwrap();
        Self {
            app,
            source,
            window,
            input,
            output,
        }
    }
    fn send(&self, key: u8, down: bool) {
        xtest::fake_input(
            &self.source,
            if down { 2 } else { 3 },
            key,
            x11rb::CURRENT_TIME,
            0,
            0,
            0,
            0,
        )
        .unwrap()
        .check()
        .unwrap();
    }
    fn pair(&self, key: u8) {
        self.send(key, true);
        self.send(key, false);
    }
    fn app_keys(&self) -> Vec<(u8, bool, u16)> {
        Self::app_keys_on(&self.app)
    }
    fn app_keys_on(app: &RustConnection) -> Vec<(u8, bool, u16)> {
        app.get_input_focus().unwrap().reply().unwrap();
        let mut keys = Vec::new();
        while let Some(e) = app.poll_for_event().unwrap() {
            match e {
                Event::KeyPress(e) => keys.push((e.detail, true, e.state.into())),
                Event::KeyRelease(e) => keys.push((e.detail, false, e.state.into())),
                _ => {}
            }
        }
        keys
    }
    fn observed(&mut self) -> Vec<InputEvent> {
        self.input.conn.get_input_focus().unwrap().reply().unwrap();
        let mut events = Vec::new();
        for _ in 0..32 {
            match self.input.fetch_events() {
                Ok(batch) => events.extend(batch),
                Err(e) if e.kind() == io::ErrorKind::WouldBlock => return events,
                Err(e) => panic!("input failed: {e}"),
            }
        }
        panic!("input did not drain");
    }
    fn emit(&mut self, key: KeyCode) {
        self.output
            .emit(&[
                InputEvent::new(EventType::KEY.0, key.code(), 1),
                InputEvent::new(EventType::KEY.0, key.code(), 0),
            ])
            .unwrap();
    }
}

#[test]
#[ignore = "requires dedicated owned Xvfb"]
fn native_x11_master_freeze_preserves_input_on_success_and_rejection() {
    let mut f = Fixture::new();
    for success in [true, false] {
        f.input.grab().unwrap();
        f.pair(38);
        f.pair(56);
        assert!(f.app_keys().is_empty());
        assert!(f.observed().is_empty());
        if success {
            f.emit(KeyCode::KEY_C);
            assert_eq!(
                f.app_keys().iter().map(|x| (x.0, x.1)).collect::<Vec<_>>(),
                [(54, true), (54, false)]
            );
        }
        f.input.ungrab().unwrap();
        assert_eq!(
            f.app_keys().iter().map(|x| (x.0, x.1)).collect::<Vec<_>>(),
            [(38, true), (38, false), (56, true), (56, false)]
        );
        let observed = f.observed();
        assert_eq!(observed.len(), 4);
        assert_eq!(observed[0].code(), KeyCode::KEY_A.code());
    }
}

#[test]
#[ignore = "requires dedicated owned Xvfb"]
fn native_x11_prepared_focus_cannot_emit_before_input_isolation() {
    let mut f = Fixture::new();
    let _maps = f.input.prepare_replay().unwrap();
    {
        let mut replay = f.input.replay.lock().unwrap();
        kde_client_focus::retain(&mut replay, f.window).unwrap();
        assert!(replay.maps.is_none());
    }
    assert!(!f.input.grabbed);
    assert!(f
        .output
        .emit(&[
            InputEvent::new(EventType::KEY.0, KeyCode::KEY_C.code(), 1),
            InputEvent::new(EventType::KEY.0, KeyCode::KEY_C.code(), 0),
        ])
        .is_err());
    assert!(f.app_keys().is_empty());
    f.input.ungrab().unwrap();
    assert_eq!(f.input.replay.lock().unwrap().keyboard, 0);
    f.pair(38);
    assert_eq!(f.app_keys().len(), 2);
}

#[test]
#[ignore = "requires dedicated owned Xvfb"]
fn native_x11_kde_failed_grab_retires_prepared_focus_without_thawing_foreign_grab() {
    let mut f = Fixture::new();
    f.source
        .xinput_xi_query_version(2, 2)
        .unwrap()
        .reply()
        .unwrap();
    let reply = f
        .source
        .xinput_xi_grab_device(
            f.window,
            x11rb::CURRENT_TIME,
            0u32,
            f.input.core,
            xproto::GrabMode::SYNC,
            xproto::GrabMode::ASYNC,
            xinput::GrabOwner::NO_OWNER,
            &[u32::from(XIEventMask::KEY_PRESS | XIEventMask::KEY_RELEASE)],
        )
        .unwrap()
        .reply()
        .unwrap();
    assert_eq!(reply.status, xproto::GrabStatus::SUCCESS);
    assert!(f.input.grab_with_client_focus(true).is_err());
    assert!(!f.input.grabbed);
    assert_eq!(f.input.replay.lock().unwrap().keyboard, 0);
    let reply = f
        .input
        .conn
        .xinput_xi_grab_device(
            f.window,
            x11rb::CURRENT_TIME,
            0u32,
            f.input.core,
            xproto::GrabMode::SYNC,
            xproto::GrabMode::ASYNC,
            xinput::GrabOwner::NO_OWNER,
            &[u32::from(XIEventMask::KEY_PRESS | XIEventMask::KEY_RELEASE)],
        )
        .unwrap()
        .reply()
        .unwrap();
    assert_eq!(reply.status, xproto::GrabStatus::ALREADY_GRABBED);
    f.source
        .xinput_xi_ungrab_device(x11rb::CURRENT_TIME, f.input.core)
        .unwrap()
        .check()
        .unwrap();
    f.pair(38);
    assert_eq!(f.app_keys().len(), 2);
}

#[test]
#[ignore = "requires dedicated owned Xvfb"]
fn native_x11_output_is_excluded_and_old_raw_prefix_is_not_replayed() {
    let mut f = Fixture::new();
    assert!(f
        .output
        .emit(&[InputEvent::new(EventType::KEY.0, KeyCode::KEY_C.code(), 1)])
        .is_err());
    f.input.grab().unwrap();
    f.emit(KeyCode::KEY_C);
    assert_eq!(f.app_keys().len(), 2);
    assert!(f.observed().is_empty());
    f.input.ungrab().unwrap();
    f.pair(38);
    assert!(f.input.grab().is_err());
    assert_eq!(f.app_keys().len(), 2);
    assert_eq!(f.observed().len(), 2);
    assert!(f.app_keys().is_empty());
    f.input.set_batch_pending(true);
    assert!(f.input.grab().is_err());
    f.input.set_batch_pending(false);
    f.input.grab().unwrap();
    f.input.ungrab().unwrap();
}

#[test]
#[ignore = "requires dedicated owned Xvfb"]
fn native_x11_held_key_and_competing_owner_do_not_freeze_input() {
    let mut f = Fixture::new();
    f.send(38, true);
    assert!(f.input.grab().is_err());
    f.send(38, false);
    assert_eq!(f.app_keys().len(), 2);
    assert_eq!(f.observed().len(), 2);
    let status = f
        .source
        .xinput_xi_grab_device(
            f.window,
            x11rb::CURRENT_TIME,
            0u32,
            f.input.core,
            xproto::GrabMode::ASYNC,
            xproto::GrabMode::ASYNC,
            xinput::GrabOwner::NO_OWNER,
            &[],
        )
        .unwrap()
        .reply()
        .unwrap()
        .status;
    assert_eq!(status, xproto::GrabStatus::SUCCESS);
    assert!(f.input.grab().is_err());
    f.source
        .xinput_xi_ungrab_device(x11rb::CURRENT_TIME, f.input.core)
        .unwrap()
        .check()
        .unwrap();
    f.pair(56);
    assert_eq!(f.app_keys().len(), 2);
    assert_eq!(f.observed().len(), 2);
}

#[test]
#[ignore = "requires dedicated owned Xvfb"]
fn native_x11_long_backlog_and_unknown_keys_survive_aborted_acquisition() {
    let mut f = Fixture::new();
    for _ in 0..300 {
        f.pair(38);
    }
    f.pair(67); // F1 is not a lexical/replay key; server still owns it exactly.
    assert!(f.input.grab().is_err());
    assert_eq!(f.app_keys().len(), 602);
    assert_eq!(f.observed().len(), 602);
    f.input.grab().unwrap();
    f.pair(67);
    f.input.ungrab().unwrap();
    assert_eq!(f.app_keys().len(), 2);
    assert_eq!(f.observed().len(), 2);
}

#[test]
#[ignore = "requires dedicated owned Xvfb"]
fn native_x11_drop_thaws_input_and_removes_transport_resources() {
    let mut f = Fixture::new();
    f.input.grab().unwrap();
    let keyboard = f.input_replay_keyboard();
    f.pair(38);
    assert!(f.app_keys().is_empty());
    drop(f.input);
    assert_eq!(Fixture::app_keys_on(&f.app).len(), 2);
    drop(f.output);
    assert!(!f
        .app
        .xinput_xi_query_device(0u16)
        .unwrap()
        .reply()
        .unwrap()
        .infos
        .iter()
        .any(|d| d.deviceid == keyboard));
}

#[test]
#[ignore = "requires dedicated owned Xvfb"]
fn native_x11_owner_loss_revokes_output_and_orphan_is_reaped() {
    let mut f = Fixture::new();
    f.input.grab().unwrap();
    let old = f.input_replay_keyboard();
    // Connection death destroys the selection window, but XI hierarchy is server-owned.
    assert_eq!(
        unsafe {
            libc::shutdown(
                f.input.replay.lock().unwrap().conn.stream().as_raw_fd(),
                libc::SHUT_RDWR,
            )
        },
        0
    );
    for _ in 0..100 {
        if f.app
            .get_selection_owner(f.input.replay.lock().unwrap().selection_for_test())
            .unwrap()
            .reply()
            .unwrap()
            .owner
            == 0
        {
            break;
        }
        std::thread::yield_now();
    }
    assert!(f
        .output
        .emit(&[InputEvent::new(EventType::KEY.0, KeyCode::KEY_A.code(), 1)])
        .is_err());
    drop(f.input);
    drop(f.output);
    let (mut input, output) = open(Arc::new(AtomicU64::new(1))).unwrap();
    input.grab().unwrap();
    let devices = f
        .app
        .xinput_xi_query_device(0u16)
        .unwrap()
        .reply()
        .unwrap()
        .infos;
    assert_eq!(
        devices
            .iter()
            .filter(
                |d| d.type_ == DeviceType::MASTER_KEYBOARD && replay_master::is_owned_name(&d.name)
            )
            .count(),
        1
    );
    // IDs may be reused; the new live selection, not an ID difference, proves ownership.
    let _ = old;
    input.replay.lock().unwrap().validate_owner().unwrap();
    drop(input);
    drop(output);
}

#[test]
#[ignore = "requires dedicated owned Xvfb"]
fn native_x11_live_predecessor_drop_cannot_release_or_remove_successor() {
    let mut f = Fixture::new();
    let down = InputEvent::new(EventType::KEY.0, KeyCode::KEY_A.code(), 1);
    let up = InputEvent::new(EventType::KEY.0, KeyCode::KEY_A.code(), 0);
    f.input.grab().unwrap();
    f.output.emit(&[down]).unwrap();
    f.app_keys();
    let old_source = f.input_replay_source();
    f.app
        .set_selection_owner(
            0u32,
            f.input.replay.lock().unwrap().selection_for_test(),
            x11rb::CURRENT_TIME,
        )
        .unwrap()
        .check()
        .unwrap();
    assert!(f.input.ungrab().is_err()); // Controlled loss of the old resource claim.
    let (mut input, mut output) = open(Arc::new(AtomicU64::new(1))).unwrap();
    input.grab().unwrap();
    assert_eq!(
        input.replay.lock().unwrap().source,
        old_source,
        "exercise reused numeric IDs"
    );
    f.app_keys(); // Hierarchy removal may release the predecessor's held A.
    output.emit(&[down]).unwrap();
    assert_eq!(f.app_keys().len(), 1);
    drop(f.input);
    drop(f.output);
    input.replay.lock().unwrap().validate_owner().unwrap();
    assert!(
        Fixture::app_keys_on(&f.app).is_empty(),
        "stale Drop released successor key"
    );
    output.emit(&[up]).unwrap();
    assert_eq!(Fixture::app_keys_on(&f.app).len(), 1);
    drop(input);
    drop(output);
}

#[test]
#[ignore = "requires dedicated owned Xvfb"]
fn native_x11_active_writes_reject_reused_foreign_device_ids() {
    let mut f = Fixture::new();
    f.input.grab().unwrap();
    let replay = f.input.replay.lock().unwrap();
    replay.validate_owner().unwrap(); // Deliberately stale preflight.
    let (keyboard, source) = (replay.keyboard, replay.source);
    let core_pointer = capability().unwrap().core_pointer;
    f.app
        .xinput_xi_change_hierarchy(&[xinput::HierarchyChange {
            len: 3,
            data: xinput::HierarchyChangeData::RemoveMaster(
                xinput::HierarchyChangeDataRemoveMaster {
                    deviceid: keyboard,
                    return_mode: xinput::ChangeMode::ATTACH,
                    return_pointer: core_pointer,
                    return_keyboard: replay.core,
                },
            ),
        }])
        .unwrap()
        .check()
        .unwrap();
    let name = b"Unrelated generation";
    f.app
        .xinput_xi_change_hierarchy(&[xinput::HierarchyChange {
            len: (8 + name.len()).div_ceil(4) as u16,
            data: xinput::HierarchyChangeData::AddMaster(xinput::HierarchyChangeDataAddMaster {
                send_core: true,
                enable: true,
                name: name.to_vec(),
            }),
        }])
        .unwrap()
        .check()
        .unwrap();
    let devices = f
        .app
        .xinput_xi_query_device(0u16)
        .unwrap()
        .reply()
        .unwrap()
        .infos;
    assert!(devices
        .iter()
        .any(|d| d.deviceid == keyboard && d.name == b"Unrelated generation keyboard"));
    assert!(devices.iter().any(
        |d| d.deviceid == u16::from(source) && d.name == b"Unrelated generation XTEST keyboard"
    ));
    let before = xkb::get_state(&f.app, keyboard).unwrap().reply().unwrap();
    let mut entered = false;
    let result = replay.with_owned_devices(|| {
        entered = true;
        mapping::copy_to_owned(&replay.conn, replay.core, keyboard)?;
        replay
            .conn
            .xinput_xi_set_focus(f.window, x11rb::CURRENT_TIME, keyboard)
            .map_err(err)?
            .check()
            .map_err(err)?;
        xtest::fake_input(
            &replay.conn,
            replay.event_type,
            38,
            x11rb::CURRENT_TIME,
            replay.root,
            0,
            0,
            source,
        )
        .map_err(err)?
        .check()
        .map_err(err)
    });
    assert!(result.is_err());
    assert!(
        !entered,
        "a stale preflight must never authorize active writes"
    );
    let after = xkb::get_state(&f.app, keyboard).unwrap().reply().unwrap();
    assert_eq!((before.group, before.mods), (after.group, after.mods));
    assert!(f.app_keys().is_empty());
    drop(replay);
    assert!(f.input.ungrab().is_err());
    assert!(
        !f.input.grabbed,
        "even identity refusal must thaw original input"
    );
    f.pair(38);
    assert_eq!(f.app_keys().len(), 2);
}

fn rename_a(conn: &RustConnection, device: u16) {
    xkb::set_names(
        conn,
        device,
        xkb::VMod::default(),
        0,
        0,
        0,
        0,
        0,
        xkb::SetOfGroup::default(),
        0,
        38,
        1,
        0,
        0,
        &xkb::SetNamesAux::default().key_names(vec![xkb::KeyName { name: *b"ZZZZ" }]),
    )
    .unwrap()
    .check()
    .unwrap();
}

#[test]
#[ignore = "requires dedicated owned Xvfb"]
fn native_x11_names_only_change_revokes_input_output_and_startup_table() {
    let mut f = Fixture::new();
    let admitted = capability().unwrap();
    rename_a(&f.app, admitted.core);
    assert!(revalidate_capability(&admitted).is_err());
    f.input.conn.get_input_focus().unwrap().reply().unwrap();
    assert!(f.input.fetch_events().is_err());
    assert!(f
        .output
        .emit(&[InputEvent::new(EventType::KEY.0, KeyCode::KEY_A.code(), 1)])
        .is_err());
    assert!(f.app_keys().is_empty());
}

#[test]
#[ignore = "requires dedicated owned Xvfb"]
fn native_x11_pause_reaches_configured_trigger_as_a_physical_key() {
    let mut f = Fixture::new();
    let code = f
        .input
        .keys
        .iter()
        .position(|key| *key == KeyCode::KEY_PAUSE)
        .unwrap() as u8;
    f.pair(code);
    let events = f.observed();
    assert_eq!(events.len(), 2);
    let trigger = crate::trigger_dispatch::trigger_key_from_config("single-pause");
    assert_eq!(
        events
            .iter()
            .map(|event| (event.code(), event.value()))
            .collect::<Vec<_>>(),
        [(trigger.code(), 1), (trigger.code(), 0)]
    );
}

#[test]
#[ignore = "requires dedicated owned Xvfb"]
fn native_x11_ru_us_output_tracks_the_verified_core_group() {
    let mut f = Fixture::new();
    f.input.grab().unwrap();
    for group in [0u8, 1, 0] {
        xkb::latch_lock_state(
            &f.source,
            f.input.core,
            xproto::ModMask::default(),
            xproto::ModMask::default(),
            true,
            group.into(),
            xproto::ModMask::default(),
            false,
            0,
        )
        .unwrap()
        .check()
        .unwrap();
        assert_eq!(
            u8::from(
                xkb::get_state(&f.source, f.input.core)
                    .unwrap()
                    .reply()
                    .unwrap()
                    .group
            ),
            group
        );
        f.emit(KeyCode::KEY_A);
        let events = f.app_keys();
        for device in [
            f.input_replay_keyboard(),
            u16::from(f.input_replay_source()),
        ] {
            assert_eq!(
                u8::from(
                    xkb::get_state(&f.source, device)
                        .unwrap()
                        .reply()
                        .unwrap()
                        .group
                ),
                group
            );
        }
        assert_eq!(events.len(), 2);
        assert_eq!((events[0].2 >> 13) & 3, u16::from(group));
        assert!(f.observed().is_empty());
    }
}

#[test]
#[ignore = "requires dedicated owned Xvfb"]
fn native_x11_initial_map_admission_rejects_different_modifier_actions() {
    use std::{
        io::Write,
        process::{Command, Stdio},
    };
    let mut f = Fixture::new();
    f.input.grab().unwrap();
    let display = std::env::var("DISPLAY").unwrap();
    let target = u16::from(f.input_replay_source());
    let original = Command::new("xkbcomp")
        .args(["-w0", "-xkb", "-i", &target.to_string(), &display, "-"])
        .output()
        .unwrap();
    assert!(original.status.success());
    let text = String::from_utf8(original.stdout).unwrap();
    let changed = text.replace("modifier_map Shift", "modifier_map Control");
    assert_ne!(text, changed);
    let mut compiler = Command::new("xkbcomp")
        .args(["-w0", "-i", &target.to_string(), "-", &display])
        .stdin(Stdio::piped())
        .spawn()
        .unwrap();
    compiler
        .stdin
        .take()
        .unwrap()
        .write_all(changed.as_bytes())
        .unwrap();
    assert!(compiler.wait().unwrap().success());
    // All symbols and physical names stay the same. The changed modifier/action
    // meaning must nevertheless prevent the very first output admission.
    assert!(mapping::Maps::new(
        &f.app,
        [
            f.input.core,
            capability().unwrap().input,
            f.input_replay_keyboard(),
            target
        ]
    )
    .is_err());
    assert!(f.app_keys().is_empty());
}

#[test]
#[ignore = "requires dedicated owned Xvfb"]
fn native_x11_master_thaw_preserves_client_focus() {
    let mut f = Fixture::new();
    f.app
        .change_window_attributes(
            f.window,
            &xproto::ChangeWindowAttributesAux::new().event_mask(CoreMask::FOCUS_CHANGE),
        )
        .unwrap()
        .check()
        .unwrap();
    let children = f
        .app
        .query_tree(f.window)
        .unwrap()
        .reply()
        .unwrap()
        .children;
    f.app_keys();
    f.input.grab().unwrap();
    f.input.ungrab().unwrap();
    assert_eq!(
        f.app.get_input_focus().unwrap().reply().unwrap().focus,
        f.window
    );
    while let Some(event) = f.app.poll_for_event().unwrap() {
        if let Event::FocusOut(event) = event {
            assert!(
                event.mode != xproto::NotifyMode::UNGRAB
                    || !matches!(
                        event.detail,
                        xproto::NotifyDetail::NONLINEAR | xproto::NotifyDetail::NONLINEAR_VIRTUAL
                    ),
                "thaw must not deactivate the client in KWin: {event:?}"
            );
        }
    }
    assert_eq!(
        f.app
            .query_tree(f.window)
            .unwrap()
            .reply()
            .unwrap()
            .children,
        children,
        "the transport must retire its owned replay master"
    );
}

#[test]
#[ignore = "requires dedicated owned Xvfb"]
fn native_x11_master_freeze_preserves_existing_client_focus() {
    let mut f = Fixture::new();
    f.app
        .change_window_attributes(
            f.window,
            &xproto::ChangeWindowAttributesAux::new()
                .event_mask(CoreMask::KEY_PRESS | CoreMask::KEY_RELEASE | CoreMask::FOCUS_CHANGE),
        )
        .unwrap()
        .check()
        .unwrap();
    f.app
        .xinput_xi_query_version(2, 2)
        .unwrap()
        .reply()
        .unwrap();
    f.app
        .xinput_xi_select_events(
            f.window,
            &[EventMask {
                deviceid: f.input.core,
                mask: vec![XIEventMask::FOCUS_IN | XIEventMask::FOCUS_OUT],
            }],
        )
        .unwrap()
        .check()
        .unwrap();
    f.app_keys();
    // Negative control: a root grab does move the XI2 logical focus away.
    let reply = f
        .input
        .conn
        .xinput_xi_grab_device(
            f.app.setup().roots[0].root,
            x11rb::CURRENT_TIME,
            0u32,
            f.input.core,
            xproto::GrabMode::SYNC,
            xproto::GrabMode::ASYNC,
            xinput::GrabOwner::NO_OWNER,
            &[u32::from(XIEventMask::KEY_PRESS | XIEventMask::KEY_RELEASE)],
        )
        .unwrap()
        .reply()
        .unwrap();
    assert_eq!(reply.status, xproto::GrabStatus::SUCCESS);
    f.app.get_input_focus().unwrap().reply().unwrap();
    let mut root_moved_focus = false;
    while let Some(event) = f.app.poll_for_event().unwrap() {
        root_moved_focus |= matches!(event, Event::XinputFocusOut(_));
    }
    f.input
        .conn
        .xinput_xi_ungrab_device(x11rb::CURRENT_TIME, f.input.core)
        .unwrap()
        .check()
        .unwrap();
    assert!(
        root_moved_focus,
        "root-grab negative control did not observe focus loss"
    );
    f.app_keys();
    f.input.grab_with_client_focus(true).unwrap();
    assert_eq!(
        f.app.get_input_focus().unwrap().reply().unwrap().focus,
        f.window
    );
    while let Some(event) = f.app.poll_for_event().unwrap() {
        assert!(
            !matches!(event, Event::XinputFocusOut(_)),
            "transport acquisition changed the client's logical focus: {event:?}"
        );
        assert!(
            !matches!(event, Event::FocusOut(_)),
            "transport acquisition emitted core FocusOut: {event:?}"
        );
    }
    f.pair(38);
    assert!(f.app_keys().is_empty());
    f.emit(KeyCode::KEY_C);
    assert_eq!(f.app_keys().len(), 2);
    f.input.ungrab().unwrap();
    assert_eq!(f.app_keys().len(), 2);
}

#[test]
#[ignore = "requires dedicated owned Xvfb"]
fn native_x11_owned_locks_and_latches_are_cleared_without_losing_planned_shift() {
    let mut f = Fixture::new();
    f.input.grab().unwrap();
    for device in [
        f.input_replay_keyboard(),
        u16::from(f.input_replay_source()),
    ] {
        xkb::latch_lock_state(
            &f.app,
            device,
            xproto::ModMask::LOCK,
            xproto::ModMask::LOCK,
            false,
            0u8.into(),
            xproto::ModMask::default(),
            true,
            1,
        )
        .unwrap()
        .check()
        .unwrap();
        let state = xkb::get_state(&f.app, device).unwrap().reply().unwrap();
        assert_ne!(u16::from(state.locked_mods), 0);
        assert_ne!(state.latched_group, 0);
    }
    f.emit(KeyCode::KEY_A);
    let events = f.app_keys();
    assert_eq!(events.len(), 2);
    assert_eq!(events[0].2 & 0xff, 0, "lowercase plan acquired a modifier");
    assert_eq!((events[0].2 >> 13) & 3, 0);
    for device in [
        f.input_replay_keyboard(),
        u16::from(f.input_replay_source()),
    ] {
        let state = xkb::get_state(&f.app, device).unwrap().reply().unwrap();
        assert_eq!(u16::from(state.locked_mods), 0);
        assert_eq!(u16::from(state.latched_mods), 0);
        assert_eq!(state.latched_group, 0);
    }
    f.output
        .emit(&[InputEvent::new(
            EventType::KEY.0,
            KeyCode::KEY_LEFTSHIFT.code(),
            1,
        )])
        .unwrap();
    f.app_keys();
    f.emit(KeyCode::KEY_A);
    let events = f.app_keys();
    assert_eq!(events.len(), 2);
    assert_ne!(events[0].2 & u16::from(xproto::ModMask::SHIFT), 0);
    f.output
        .emit(&[InputEvent::new(
            EventType::KEY.0,
            KeyCode::KEY_LEFTSHIFT.code(),
            0,
        )])
        .unwrap();
    assert_eq!(f.app_keys().len(), 1);
}

#[test]
#[ignore = "requires dedicated owned Xvfb"]
fn native_x11_idle_layout_switch_preserves_input_and_refreshes_owned_output() {
    let mut f = Fixture::new();
    // IBus switches a single-group map; KDE restores its multi-group map.
    // Both keep the physical positions on which the daemon FSM depends.
    for layout in ["ru", "us,ru"] {
        assert!(std::process::Command::new("setxkbmap")
            .args(["-layout", layout])
            .status()
            .unwrap()
            .success());
        f.pair(38);
        let observed = f.observed();
        assert_eq!(observed.len(), 2);
        assert_eq!(observed[0].code(), KeyCode::KEY_A.code());
        f.app_keys();
        f.input.grab().unwrap();
        f.output
            .emit(&[
                InputEvent::new(EventType::KEY.0, KeyCode::KEY_A.code(), 1),
                InputEvent::new(EventType::KEY.0, KeyCode::KEY_A.code(), 0),
            ])
            .unwrap();
        f.input.ungrab().unwrap();
        assert_eq!(f.app_keys().len(), 2);
        assert!(f.input.valid.load(Ordering::Acquire));
    }
}

#[test]
#[ignore = "requires dedicated owned Xvfb"]
fn native_x11_map_change_while_output_held_revokes_without_another_press() {
    let mut f = Fixture::new();
    f.input.grab().unwrap();
    f.output
        .emit(&[InputEvent::new(EventType::KEY.0, KeyCode::KEY_A.code(), 1)])
        .unwrap();
    assert_eq!(f.app_keys().len(), 1);
    // This changes only the disposable test server, never the user's map.
    assert!(std::process::Command::new("setxkbmap")
        .args(["-layout", "de"])
        .status()
        .unwrap()
        .success());
    assert!(f
        .output
        .emit(&[InputEvent::new(EventType::KEY.0, KeyCode::KEY_A.code(), 1)])
        .is_err());
    assert!(f.app_keys().is_empty());
    assert!(f.input.grab().is_err());
}

#[test]
#[ignore = "requires dedicated owned Xvfb"]
fn native_x11_output_uses_current_focus_and_rapid_shifts_do_not_feed_back() {
    let mut f = Fixture::new();
    f.input.grab().unwrap();
    let second = f.app.generate_id().unwrap();
    f.app
        .create_window(
            x11rb::COPY_DEPTH_FROM_PARENT,
            second,
            f.app.setup().roots[0].root,
            0,
            0,
            200,
            100,
            0,
            WindowClass::INPUT_OUTPUT,
            0,
            &CreateWindowAux::new().event_mask(CoreMask::KEY_PRESS | CoreMask::KEY_RELEASE),
        )
        .unwrap()
        .check()
        .unwrap();
    f.app.map_window(second).unwrap().check().unwrap();
    f.app
        .set_input_focus(xproto::InputFocus::PARENT, second, x11rb::CURRENT_TIME)
        .unwrap()
        .check()
        .unwrap();
    for _ in 0..10 {
        f.emit(KeyCode::KEY_LEFTSHIFT);
    }
    let events = f.app_keys();
    assert_eq!(events.len(), 20);
    assert!(events.iter().all(|e| e.0 == 50));
    assert!(f.observed().is_empty());
    let focus = f
        .app
        .xinput_xi_get_focus(f.input_replay_keyboard())
        .unwrap()
        .reply()
        .unwrap()
        .focus;
    assert_eq!(focus, second);
}
