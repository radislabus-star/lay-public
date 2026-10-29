//! XI2 acquisition and XTest frames for keyboard-less X11 remote sessions.
//! The shared daemon loop still owns every gesture and text-mutation decision.
use evdev::{AttributeSet, EventType, InputEvent, KeyCode};
use std::{
    collections::VecDeque,
    io,
    os::fd::{AsRawFd, RawFd},
    sync::{
        atomic::{AtomicBool, AtomicU64, Ordering},
        Arc, Mutex,
    },
};
use x11rb::{
    connection::{Connection, RequestConnection},
    protocol::{
        xinput::{self, ConnectionExt as _, DeviceType, EventMask, XIEventMask},
        xkb,
        xproto::{self, ConnectionExt as _},
        xtest, Event,
    },
    rust_connection::RustConnection,
};

#[path = "../kde/native_input_focus.rs"]
mod kde_client_focus;
#[path = "keymap.rs"]
mod keymap;
#[path = "mapping.rs"]
mod mapping;
#[path = "replay_master.rs"]
mod replay_master;
const BATCH: usize = 512;

fn is_x11_session(display: Option<&str>, session: Option<&str>, wayland: Option<&str>) -> bool {
    display.is_some_and(|value| !value.is_empty())
        && wayland.is_none_or(str::is_empty)
        && session.is_none_or(|value| value.is_empty() || value.eq_ignore_ascii_case("x11"))
}

pub(crate) fn session_supported() -> bool {
    is_x11_session(
        std::env::var("DISPLAY").ok().as_deref(),
        std::env::var("XDG_SESSION_TYPE").ok().as_deref(),
        std::env::var("WAYLAND_DISPLAY").ok().as_deref(),
    )
}

fn err(error: impl std::fmt::Display) -> io::Error {
    io::Error::other(error.to_string())
}

struct Capability {
    conn: RustConnection,
    root: u32,
    input: u16,
    core: u16,
    core_pointer: u16,
    extension_event: u8,
    keys: [KeyCode; 256],
}

fn capability() -> io::Result<Capability> {
    if !session_supported() {
        return Err(err("native remote input requires an X11 session"));
    }
    let (conn, screen) = x11rb::connect(None).map_err(err)?;
    let root = conn.setup().roots[screen].root;
    let info = conn
        .extension_information("XInputExtension")
        .map_err(err)?
        .ok_or_else(|| err("XInput unavailable"))?;
    xinput::xi_query_version(&conn, 2, 2)
        .map_err(err)?
        .reply()
        .map_err(err)?;
    xtest::get_version(&conn, 2, 2)
        .map_err(err)?
        .reply()
        .map_err(err)?;
    let node = conn
        .intern_atom(true, b"Device Node")
        .map_err(err)?
        .reply()
        .map_err(err)?
        .atom;
    let devices = conn
        .xinput_xi_query_device(0u16)
        .map_err(err)?
        .reply()
        .map_err(err)?
        .infos;
    let core = devices
        .iter()
        .find(|d| {
            d.enabled
                && d.type_ == DeviceType::MASTER_KEYBOARD
                && !replay_master::is_owned_name(&d.name)
        })
        .ok_or_else(|| err("no master keyboard"))?
        .deviceid;
    if devices
        .iter()
        .filter(|d| {
            d.enabled
                && d.type_ == DeviceType::MASTER_KEYBOARD
                && !replay_master::is_owned_name(&d.name)
        })
        .count()
        != 1
    {
        return Err(err("multiple master keyboards require evdev"));
    }
    let keyboards: Vec<_> = devices
        .iter()
        .filter(|d| d.enabled && d.type_ == DeviceType::SLAVE_KEYBOARD && d.attachment == core)
        .collect();
    let mut virtuals = Vec::new();
    for d in &keyboards {
        let name = String::from_utf8_lossy(&d.name);
        if name.contains("XTEST")
            || super::super::keyboard_io::should_ignore_keyboard_device_name(&name)
        {
            continue;
        }
        let props = conn
            .xinput_xi_list_properties(d.deviceid)
            .map_err(err)?
            .reply()
            .map_err(err)?;
        if node == 0 || !props.properties.contains(&node) {
            virtuals.push(d.deviceid);
        }
    }
    // One remote source and one master make the synchronous freeze atomic.
    if virtuals.len() != 1 || keyboards.len() != 2 {
        return Err(err(
            "X11 requires one remote keyboard and one virtual server replay keyboard",
        ));
    }
    let output = u8::try_from(virtuals[0]).map_err(err)?;
    if output >= 128 {
        return Err(err("XTest device ID is out of range"));
    }
    let input = keyboards
        .iter()
        .find(|d| d.deviceid != u16::from(output))
        .unwrap()
        .deviceid;
    if !String::from_utf8_lossy(&keyboards.iter().find(|d| d.deviceid == input).unwrap().name)
        .contains("XTEST")
    {
        return Err(err("remote source must be the server XTEST keyboard"));
    }
    if !xkb::use_extension(&conn, 1, 0)
        .map_err(err)?
        .reply()
        .map_err(err)?
        .supported
    {
        return Err(err("XKB unavailable"));
    }
    let names = xkb::get_names(&conn, core, xkb::NameDetail::KEY_NAMES)
        .map_err(err)?
        .reply()
        .map_err(err)?;
    let mut keys = [KeyCode::KEY_UNKNOWN; 256];
    for (offset, name) in names
        .value_list
        .key_names
        .unwrap_or_default()
        .into_iter()
        .enumerate()
    {
        let index = usize::from(names.first_key) + offset;
        if index < keys.len() {
            keys[index] = keymap::physical_key(name.name);
        }
    }
    for key in super::super::text_output::VIRTUAL_KEYBOARD_KEYS {
        if !keys.contains(key) {
            return Err(err(format!("XKB physical key missing: {key:?}")));
        }
    }
    Ok(Capability {
        conn,
        root,
        input,
        core,
        core_pointer: devices
            .iter()
            .find(|d| d.deviceid == core)
            .unwrap()
            .attachment,
        extension_event: info.first_event + 1,
        keys,
    })
}

pub(crate) fn status() -> serde_json::Value {
    match capability() {
        Ok(c) => {
            serde_json::json!({"ready":true,"transport":"x11-xinput2","input_device":c.input,"master_keyboard":c.core})
        }
        Err(e) => {
            serde_json::json!({"ready":false,"transport":"x11-xinput2","reason":e.to_string()})
        }
    }
}

pub(crate) struct Input {
    conn: RustConnection,
    source: u16,
    keys: [KeyCode; 256],
    grabbed: bool,
    batch_pending: bool,
    pending: VecDeque<InputEvent>,
    epoch: Arc<AtomicU64>,
    valid: Arc<AtomicBool>,
    replay: Arc<Mutex<replay_master::ReplayMaster>>,
    maps: Arc<mapping::Maps>,
    core: u16,
}

pub(crate) struct Output {
    replay: Arc<Mutex<replay_master::ReplayMaster>>,
    keys: [KeyCode; 256],
    valid: Arc<AtomicBool>,
}

fn select_invalidation(conn: &RustConnection, root: u32, core: u16) -> io::Result<()> {
    conn.xinput_xi_select_events(
        root,
        &[
            EventMask {
                deviceid: 1,
                mask: vec![
                    XIEventMask::RAW_KEY_PRESS
                        | XIEventMask::RAW_KEY_RELEASE
                        | XIEventMask::RAW_BUTTON_PRESS,
                ],
            },
            EventMask {
                deviceid: 0,
                mask: vec![XIEventMask::HIERARCHY | XIEventMask::DEVICE_CHANGED],
            },
        ],
    )
    .map_err(err)?
    .check()
    .map_err(err)?;
    xkb::select_events(
        conn,
        core,
        xkb::EventType::default(),
        xkb::EventType::NEW_KEYBOARD_NOTIFY | xkb::EventType::MAP_NOTIFY,
        xkb::MapPart::default(),
        xkb::MapPart::default(),
        &xkb::SelectEventsAux::default(),
    )
    .map_err(err)?
    .check()
    .map_err(err)?;
    Ok(())
}

fn select_key_names(conn: &RustConnection, device: u16) -> io::Result<()> {
    xkb::select_events(
        conn,
        device,
        xkb::EventType::default(),
        xkb::EventType::default(),
        xkb::MapPart::default(),
        xkb::MapPart::default(),
        &xkb::SelectEventsAux::default().names_notify(xkb::SelectEventsAuxNamesNotify {
            affect_names: xkb::NameDetail::KEY_NAMES,
            names_details: xkb::NameDetail::KEY_NAMES,
        }),
    )
    .map_err(err)?
    .check()
    .map_err(err)
}

fn revalidate_capability(admitted: &Capability) -> io::Result<()> {
    let current = capability()?;
    if current.root != admitted.root
        || current.core != admitted.core
        || current.core_pointer != admitted.core_pointer
        || current.input != admitted.input
        || current.keys != admitted.keys
    {
        return Err(err(
            "native X11 topology or physical key names changed during startup",
        ));
    }
    Ok(())
}

pub(crate) fn open(epoch: Arc<AtomicU64>) -> io::Result<(Input, Output)> {
    let c = capability()?;
    let replay = replay_master::ReplayMaster::create(c.core, c.core_pointer, c.extension_event)?;
    xkb::use_extension(&replay.conn, 1, 0)
        .map_err(err)?
        .reply()
        .map_err(err)?;
    let valid = Arc::new(AtomicBool::new(true));
    select_invalidation(&c.conn, c.root, c.core)?;
    select_invalidation(&replay.conn, c.root, c.core)?;
    for device in [c.core, c.input] {
        select_key_names(&c.conn, device)?;
        select_key_names(&replay.conn, device)?;
    }
    let maps = Arc::new(mapping::Maps::new(&c.conn, [c.core, c.input])?);
    revalidate_capability(&c)?;
    let replay = Arc::new(Mutex::new(replay));
    let output = Output {
        replay: Arc::clone(&replay),
        keys: c.keys,
        valid: Arc::clone(&valid),
    };
    let input = Input {
        conn: c.conn,
        source: c.input,
        keys: c.keys,
        grabbed: false,
        batch_pending: false,
        pending: VecDeque::new(),
        epoch,
        valid,
        replay,
        maps,
        core: c.core,
    };
    Ok((input, output))
}

impl Input {
    fn revoke(&self) {
        self.valid.store(false, Ordering::Release);
        self.epoch.fetch_add(1, Ordering::AcqRel);
    }

    fn translate(&self, event: Event) -> io::Result<Option<InputEvent>> {
        if let Err(error) = self.maps.validate_event(&self.conn, &event) {
            self.revoke();
            return Err(error);
        }
        let (source, detail, press, repeat) = match event {
            Event::XinputRawKeyPress(e) => (
                e.sourceid,
                e.detail,
                true,
                e.flags.contains(xinput::KeyEventFlags::KEY_REPEAT),
            ),
            Event::XinputRawKeyRelease(e) => (e.sourceid, e.detail, false, false),
            Event::XinputRawButtonPress(_) => {
                self.epoch.fetch_add(1, Ordering::AcqRel);
                return Ok(None);
            }
            _ => return Ok(None),
        };
        if source != self.source {
            return Ok(None);
        }
        let key = self
            .keys
            .get(detail as usize)
            .copied()
            .unwrap_or(KeyCode::KEY_UNKNOWN);
        Ok(Some(InputEvent::new(
            EventType::KEY.0,
            key.code(),
            if press {
                if repeat {
                    2
                } else {
                    1
                }
            } else {
                0
            },
        )))
    }

    pub(crate) fn set_batch_pending(&mut self, pending: bool) {
        self.batch_pending = pending;
    }

    pub(crate) fn fetch_events(&mut self) -> io::Result<Vec<InputEvent>> {
        let mut events: Vec<_> = self.pending.drain(..).collect();
        let mut exhausted = false;
        for _ in 0..BATCH {
            let Some(event) = self.conn.poll_for_event().map_err(err)? else {
                exhausted = true;
                break;
            };
            let epoch = self.epoch.load(Ordering::Acquire);
            let translated = self.translate(event);
            if self.epoch.load(Ordering::Acquire) != epoch {
                // A pointer fence separates fields even within one X window.
                // Retire observations only; original keys belong to the server.
                events.clear();
            }
            match translated {
                Ok(Some(event)) => events.push(event),
                Ok(None) => {}
                Err(e) if events.is_empty() => return Err(e),
                Err(_) => break, // Return the valid prefix before reporting revocation.
            }
        }
        if !events.is_empty() {
            return Ok(events);
        }
        if !self.valid.load(Ordering::Acquire) {
            return Err(err("X11 input capability revoked"));
        }
        if !exhausted {
            return Ok(Vec::new());
        } // Buffered protocol data needs no new fd edge.
        Err(io::ErrorKind::WouldBlock.into())
    }

    pub(crate) fn key_state(&self) -> io::Result<AttributeSet<KeyCode>> {
        let state = self
            .conn
            .query_keymap()
            .map_err(err)?
            .reply()
            .map_err(err)?;
        let mut keys = AttributeSet::new();
        for (code, key) in self.keys.iter().enumerate() {
            if *key != KeyCode::KEY_UNKNOWN && state.keys[code / 8] & (1 << (code % 8)) != 0 {
                keys.insert(*key);
            }
        }
        Ok(keys)
    }

    pub(crate) fn grab(&mut self) -> io::Result<()> {
        self.grab_with_client_focus(kde_client_focus::required(crate::active_layout_backend()))
    }

    fn grab_with_client_focus(&mut self, retain_client_focus: bool) -> io::Result<()> {
        let acquisition_epoch = self.epoch.load(Ordering::Acquire);
        if self.grabbed || self.batch_pending || !self.pending.is_empty() {
            return Err(io::ErrorKind::WouldBlock.into());
        }
        if !self.valid.load(Ordering::Acquire) {
            return Err(err("X11 input revoked"));
        }
        self.replay.lock().map_err(err)?.validate_owner()?;
        // Every held key, including unmapped keys, prevents a freeze midway through a chord.
        if self
            .conn
            .query_keymap()
            .map_err(err)?
            .reply()
            .map_err(err)?
            .keys
            .iter()
            .any(|b| *b != 0)
        {
            return Err(io::ErrorKind::WouldBlock.into());
        }
        let focus = self
            .conn
            .get_input_focus()
            .map_err(err)?
            .reply()
            .map_err(err)?
            .focus;
        if focus <= 1 {
            return Err(err("X11 input has no concrete focused window"));
        }
        let mut prepared_maps = if retain_client_focus {
            let prepared = (|| {
                let maps = self.prepare_replay()?;
                let mut replay = self.replay.lock().map_err(err)?;
                kde_client_focus::retain(&mut replay, focus)?;
                Ok(maps)
            })();
            match prepared {
                Ok(maps) => Some(maps),
                Err(error) => {
                    self.ungrab()?;
                    return Err(error);
                }
            }
        } else {
            None
        };
        let acquired = (|| {
            let reply = self
                .conn
                .xinput_xi_grab_device(
                    focus,
                    x11rb::CURRENT_TIME,
                    0u32,
                    self.core,
                    xproto::GrabMode::SYNC,
                    xproto::GrabMode::ASYNC,
                    xinput::GrabOwner::NO_OWNER,
                    &[u32::from(XIEventMask::KEY_PRESS | XIEventMask::KEY_RELEASE)],
                )
                .map_err(err)?
                .reply()
                .map_err(err)?;
            if reply.status != xproto::GrabStatus::SUCCESS {
                return Err(err(format!("X11 master grab rejected: {:?}", reply.status)));
            }
            Ok(())
        })();
        if let Err(error) = acquired {
            if retain_client_focus {
                self.ungrab()?;
            }
            return Err(error);
        }
        self.grabbed = true;
        // All raw input preceding the reply was already delivered. The master now
        // freezes new input at the server; no captured key is consumed by this client.
        let boundary = (|| {
            if self
                .conn
                .get_input_focus()
                .map_err(err)?
                .reply()
                .map_err(err)?
                .focus
                != focus
            {
                return Err(err("X11 input focus changed during acquisition"));
            }
            let mut empty = false;
            for _ in 0..4096 {
                let Some(event) = self.conn.poll_for_event().map_err(err)? else {
                    empty = true;
                    break;
                };
                let epoch = self.epoch.load(Ordering::Acquire);
                let translated = self.translate(event)?;
                if self.epoch.load(Ordering::Acquire) != epoch {
                    self.pending.clear();
                }
                if let Some(event) = translated {
                    self.pending.push_back(event);
                }
            }
            if !empty
                || !self.pending.is_empty()
                || self.epoch.load(Ordering::Acquire) != acquisition_epoch
            {
                return Err(io::ErrorKind::WouldBlock.into());
            }
            if let Some(maps) = prepared_maps.take() {
                self.replay.lock().map_err(err)?.maps = Some(maps);
            } else {
                self.activate_replay()?;
            }
            if self.epoch.load(Ordering::Acquire) != acquisition_epoch || !self.pending.is_empty() {
                return Err(io::ErrorKind::WouldBlock.into());
            }
            Ok(())
        })();
        if let Err(error) = boundary {
            self.epoch.fetch_add(1, Ordering::AcqRel);
            self.ungrab()?;
            return Err(error);
        }
        Ok(())
    }

    fn settle_lifecycle(&mut self, ids: &[u16]) -> io::Result<()> {
        // A round trip bounds the events caused by our resource operation. Only
        // the four validated owned IDs may change. Never mask a foreign change.
        self.conn
            .get_input_focus()
            .map_err(err)?
            .reply()
            .map_err(err)?;
        for _ in 0..4096 {
            let Some(event) = self.conn.poll_for_event().map_err(err)? else {
                return Ok(());
            };
            if own_hierarchy(&event, ids) {
                continue;
            }
            let epoch = self.epoch.load(Ordering::Acquire);
            let translated = self.translate(event)?;
            if self.epoch.load(Ordering::Acquire) != epoch {
                self.pending.clear();
            }
            if let Some(event) = translated {
                self.pending.push_back(event);
            }
        }
        Err(err("X11 resource event budget exceeded"))
    }

    fn activate_replay(&mut self) -> io::Result<()> {
        let maps = self.prepare_replay()?;
        self.replay.lock().map_err(err)?.maps = Some(maps);
        Ok(())
    }

    fn prepare_replay(&mut self) -> io::Result<mapping::Maps> {
        let (ids, maps) = {
            let mut replay = self.replay.lock().map_err(err)?;
            replay.activate()?;
            let ids = replay.device_ids()?;
            let conn = &replay.conn;
            let maps = replay.with_owned_devices(|| {
                for device in [replay.keyboard, u16::from(replay.source)] {
                    select_key_names(conn, device)?;
                    xkb::select_events(
                        conn,
                        device,
                        xkb::EventType::default(),
                        xkb::EventType::NEW_KEYBOARD_NOTIFY | xkb::EventType::MAP_NOTIFY,
                        xkb::MapPart::default(),
                        xkb::MapPart::default(),
                        &xkb::SelectEventsAux::default(),
                    )
                    .map_err(err)?
                    .check()
                    .map_err(err)?;
                    mapping::copy_to_owned(conn, self.core, device)?;
                }
                mapping::Maps::new(
                    conn,
                    [
                        self.core,
                        self.source,
                        replay.keyboard,
                        u16::from(replay.source),
                    ],
                )
            })?;
            replay
                .conn
                .get_input_focus()
                .map_err(err)?
                .reply()
                .map_err(err)?;
            let mut empty = false;
            for _ in 0..4096 {
                let Some(event) = replay.conn.poll_for_event().map_err(err)? else {
                    empty = true;
                    break;
                };
                if own_hierarchy(&event, &ids) {
                    continue;
                }
                maps.validate_event(&replay.conn, &event)?;
            }
            if !empty {
                return Err(err("X11 replay resource event budget exceeded"));
            }
            replay.validate_owner()?;
            (ids, maps)
        };
        self.settle_lifecycle(&ids)?;
        Ok(maps)
    }

    pub(crate) fn ungrab(&mut self) -> io::Result<()> {
        if !self.grabbed && self.replay.lock().map_err(err)?.keyboard == 0 {
            return Ok(());
        }
        let cleanup = (|| {
            let ids = {
                let mut replay = self.replay.lock().map_err(err)?;
                let ids = replay.device_ids()?;
                if replay.keyboard != 0 && replay.held.is_empty() {
                    // Xorg emits core Nonlinear FocusOut/In even when the
                    // original grab and focus windows are identical. Retain
                    // the already-directed output master through thaw so its
                    // same-client focus suppresses that false deactivation.
                    // Never set the original master's focus or activate a window.
                    let focus = self
                        .conn
                        .get_input_focus()
                        .map_err(err)?
                        .reply()
                        .map_err(err)?
                        .focus;
                    if focus > 1 {
                        replay.with_owned_devices(|| {
                            replay
                                .conn
                                .xinput_xi_set_focus(focus, x11rb::CURRENT_TIME, replay.keyboard)
                                .map_err(err)?
                                .check()
                                .map_err(err)
                        })?;
                    }
                    self.conn
                        .xinput_xi_ungrab_device(x11rb::CURRENT_TIME, self.core)
                        .map_err(err)?
                        .check()
                        .map_err(err)?;
                }
                // Failed frames with held output keys retire those resources
                // before the unconditional original-input thaw below.
                replay.deactivate()?;
                // Retire notifications from this generation before reusing IDs.
                replay
                    .conn
                    .get_input_focus()
                    .map_err(err)?
                    .reply()
                    .map_err(err)?;
                for _ in 0..4096 {
                    let Some(event) = replay.conn.poll_for_event().map_err(err)? else {
                        break;
                    };
                    if !own_hierarchy(&event, &ids) {
                        self.maps.validate_event(&replay.conn, &event)?;
                    }
                }
                ids
            };
            self.settle_lifecycle(&ids)
        })();
        // A failed cleanup must never leave the user's original keys frozen.
        let thaw = self
            .conn
            .xinput_xi_ungrab_device(x11rb::CURRENT_TIME, self.core)
            .map_err(err)?
            .check()
            .map_err(err);
        if thaw.is_ok() {
            self.grabbed = false;
        }
        cleanup.and(thaw)
    }
}
impl AsRawFd for Input {
    fn as_raw_fd(&self) -> RawFd {
        self.conn.stream().as_raw_fd()
    }
}
impl Drop for Input {
    fn drop(&mut self) {
        let _ = self.ungrab();
    }
}

fn own_hierarchy(event: &Event, ids: &[u16]) -> bool {
    let Event::XinputHierarchy(event) = event else {
        return false;
    };
    // XIHierarchyEvent also reports unchanged devices. Only rows carrying
    // change flags belong to the operation; any foreign change still revokes.
    let mut changed = event
        .infos
        .iter()
        .filter(|d| u32::from(d.flags) != 0)
        .peekable();
    changed.peek().is_some() && changed.all(|d| ids.contains(&d.deviceid))
}

impl Output {
    fn prepare(&self, replay: &replay_master::ReplayMaster) -> io::Result<()> {
        if !self.valid.load(Ordering::Acquire) || replay.keyboard == 0 {
            return Err(err("X11 replay requires an active isolated edit"));
        }
        replay.validate_owner()?;
        let maps = replay
            .maps
            .as_ref()
            .ok_or_else(|| err("X11 replay maps not admitted"))?;
        let mut drained = false;
        for _ in 0..4096 {
            let Some(event) = replay.conn.poll_for_event().map_err(err)? else {
                drained = true;
                break;
            };
            if let Err(error) = maps.validate_event(&replay.conn, &event) {
                self.valid.store(false, Ordering::Release);
                return Err(error);
            }
        }
        if !drained {
            self.valid.store(false, Ordering::Release);
            return Err(err("X11 output event budget exceeded"));
        }
        replay.with_owned_devices(|| {
            if let Err(error) = maps.prepare_output(&replay.conn, !replay.held.is_empty()) {
                self.valid.store(false, Ordering::Release);
                return Err(error);
            }
            let focus = replay
                .conn
                .get_input_focus()
                .map_err(err)?
                .reply()
                .map_err(err)?
                .focus;
            if focus <= 1 {
                return Err(err("X11 replay has no concrete focused window"));
            }
            replay
                .conn
                .xinput_xi_set_focus(focus, x11rb::CURRENT_TIME, replay.keyboard)
                .map_err(err)?
                .check()
                .map_err(err)?;
            let state = xkb::get_state(&replay.conn, replay.core)
                .map_err(err)?
                .reply()
                .map_err(err)?;
            for device in [replay.keyboard, u16::from(replay.source)] {
                xkb::latch_lock_state(
                    &replay.conn,
                    device,
                    xproto::ModMask::from(0xffu16),
                    xproto::ModMask::default(),
                    true,
                    state.group,
                    xproto::ModMask::from(0xffu16),
                    true,
                    0,
                )
                .map_err(err)?
                .check()
                .map_err(err)?;
                let actual = xkb::get_state(&replay.conn, device)
                    .map_err(err)?
                    .reply()
                    .map_err(err)?;
                if u16::from(actual.locked_mods) != 0
                    || u16::from(actual.latched_mods) != 0
                    || actual.latched_group != 0
                    || actual.group != state.group
                {
                    return Err(err(
                        "owned replay lock/latch state disagrees with output plan",
                    ));
                }
            }
            Ok(())
        })
    }

    pub(crate) fn emit(&mut self, events: &[InputEvent]) -> io::Result<()> {
        let mut replay = self.replay.lock().map_err(err)?;
        self.prepare(&replay)?;
        for event in events {
            if event.event_type() != EventType::KEY {
                continue;
            }
            let key = KeyCode::new(event.code());
            let code = self
                .keys
                .iter()
                .position(|k| *k == key)
                .ok_or_else(|| err("unmapped X11 replay key"))? as u8;
            if event.value() == 0 && !replay.held.contains(&code) {
                continue;
            }
            replay.with_owned_devices(|| {
                xtest::fake_input(
                    &replay.conn,
                    replay.event_type + u8::from(event.value() == 0),
                    code,
                    x11rb::CURRENT_TIME,
                    replay.root,
                    0,
                    0,
                    replay.source,
                )
                .map_err(err)?
                .check()
                .map_err(err)
            })?;
            if event.value() == 0 {
                replay.held.remove(&code);
            } else {
                replay.held.insert(code);
            }
        }
        replay.conn.flush().map_err(err)
    }
}

impl Drop for Output {
    fn drop(&mut self) {
        if let Ok(mut replay) = self.replay.lock() {
            let _ = replay.deactivate();
        }
    }
}

#[cfg(test)]
#[path = "x11_tests.rs"]
mod tests;
