//! Owns only X server transport resources; never selects an edit or a gesture.
use super::{err, xinput, xproto, Connection, RustConnection};
use std::{collections::BTreeSet, io};
use x11rb::{
    protocol::xinput::{ConnectionExt as _, DeviceType},
    protocol::xproto::ConnectionExt as _,
};

const PREFIX: &str = "Lay Remote Replay";
const SELECTION: &[u8] = b"_LAY_REMOTE_INPUT_OWNER_V1";

pub(super) fn is_owned_name(name: &[u8]) -> bool {
    owned_base(name).is_some()
}

fn owned_base(name: &[u8]) -> Option<&str> {
    let base = std::str::from_utf8(name).ok()?.strip_suffix(" keyboard")?;
    if base == PREFIX {
        return Some(base); // Recognize the earlier adapter's orphan only.
    }
    let generation = base.strip_prefix(PREFIX)?.strip_prefix(' ')?;
    (generation.len() == 8 && generation.bytes().all(|b| b.is_ascii_hexdigit())).then_some(base)
}

pub(super) struct ReplayMaster {
    pub conn: RustConnection,
    pub keyboard: u16,
    pub source: u8,
    pub core: u16,
    pub root: u32,
    window: u32,
    selection: u32,
    pointer: u16,
    core_pointer: u16,
    name: String,
    pub maps: Option<super::mapping::Maps>,
    pub held: BTreeSet<u8>,
    pub event_type: u8,
}

fn remove(conn: &RustConnection, keyboard: u16, pointer: u16, core: u16) -> io::Result<()> {
    conn.xinput_xi_change_hierarchy(&[xinput::HierarchyChange {
        len: 3,
        data: xinput::HierarchyChangeData::RemoveMaster(xinput::HierarchyChangeDataRemoveMaster {
            deviceid: keyboard,
            return_mode: xinput::ChangeMode::ATTACH,
            return_pointer: pointer,
            return_keyboard: core,
        }),
    }])
    .map_err(err)?
    .check()
    .map_err(err)
}

impl ReplayMaster {
    #[cfg(test)]
    pub fn selection_for_test(&self) -> u32 {
        self.selection
    }

    pub fn create(core: u16, core_pointer: u16, event_type: u8) -> io::Result<Self> {
        let (conn, screen) = x11rb::connect(None).map_err(err)?;
        let root = conn.setup().roots[screen].root;
        let selection = conn
            .intern_atom(false, SELECTION)
            .map_err(err)?
            .reply()
            .map_err(err)?
            .atom;
        // Startup resource lock only. Never held while editing, waiting for IBus,
        // changing focus, or treating XTest as isolated input.
        conn.grab_server().map_err(err)?.check().map_err(err)?;
        let claimed = (|| {
            if conn
                .get_selection_owner(selection)
                .map_err(err)?
                .reply()
                .map_err(err)?
                .owner
                != 0
            {
                return Err(err("native X11 input already has a live owner"));
            }
            let devices = conn
                .xinput_xi_query_device(0u16)
                .map_err(err)?
                .reply()
                .map_err(err)?
                .infos;
            for master in devices
                .iter()
                .filter(|d| d.type_ == DeviceType::MASTER_KEYBOARD && is_owned_name(&d.name))
            {
                let pair = master.attachment;
                let base = owned_base(&master.name).unwrap();
                if devices.iter().any(|d| {
                    (d.type_ == DeviceType::SLAVE_KEYBOARD || d.type_ == DeviceType::SLAVE_POINTER)
                        && (d.attachment == master.deviceid || d.attachment == pair)
                        && d.name != format!("{base} XTEST keyboard").as_bytes()
                        && d.name != format!("{base} XTEST pointer").as_bytes()
                }) {
                    return Err(err("orphan replay master has foreign attached devices"));
                }
                remove(&conn, master.deviceid, core_pointer, core)?;
            }
            let window = conn.generate_id().map_err(err)?;
            conn.create_window(
                0,
                window,
                root,
                0,
                0,
                1,
                1,
                0,
                xproto::WindowClass::INPUT_ONLY,
                0,
                &xproto::CreateWindowAux::new(),
            )
            .map_err(err)?
            .check()
            .map_err(err)?;
            conn.set_selection_owner(window, selection, x11rb::CURRENT_TIME)
                .map_err(err)?
                .check()
                .map_err(err)?;
            Ok(window)
        })();
        let unlocked = conn.ungrab_server().map_err(err)?.check().map_err(err);
        let window = claimed?;
        unlocked?;
        // X resource IDs cannot be reused while their connection stays alive.
        // XI IDs can; bind each pair to this generation for stale Drop safety.
        let name = format!("{PREFIX} {window:08x}");
        Ok(Self {
            conn,
            core,
            core_pointer,
            root,
            window,
            selection,
            name,
            keyboard: 0,
            source: 0,
            pointer: 0,
            maps: None,
            held: BTreeSet::new(),
            event_type,
        })
    }

    pub fn activate(&mut self) -> io::Result<()> {
        self.validate_owner()?;
        if self.keyboard != 0 {
            return Err(err("replay master already active"));
        }
        let conn = &self.conn;
        let name = &self.name;
        let wire_name = name.as_bytes().to_vec();
        let length = (8 + name.len()).div_ceil(4) as u16;
        self.add_preserving_layout_metadata(&[xinput::HierarchyChange {
            len: length,
            data: xinput::HierarchyChangeData::AddMaster(xinput::HierarchyChangeDataAddMaster {
                send_core: true,
                enable: true,
                name: wire_name,
            }),
        }])?;
        let devices = conn
            .xinput_xi_query_device(0u16)
            .map_err(err)?
            .reply()
            .map_err(err)?
            .infos;
        let keyboard = devices
            .iter()
            .find(|d| {
                d.type_ == DeviceType::MASTER_KEYBOARD
                    && d.name == format!("{name} keyboard").as_bytes()
            })
            .ok_or_else(|| err("replay master was not created"))?;
        let source = devices
            .iter()
            .find(|d| {
                d.attachment == keyboard.deviceid
                    && d.name == format!("{name} XTEST keyboard").as_bytes()
            })
            .ok_or_else(|| err("replay XTEST source was not created"))?;
        self.keyboard = keyboard.deviceid;
        self.pointer = keyboard.attachment;
        self.source = u8::try_from(source.deviceid).map_err(err)?;
        if self.source >= 128 {
            return Err(err("replay source exceeds XTest device range"));
        }
        self.with_owned_devices(|| self.hide_pointer())
    }

    fn add_preserving_layout_metadata(
        &self,
        changes: &[xinput::HierarchyChange],
    ) -> io::Result<()> {
        let conn = &self.conn;
        let atom = conn
            .intern_atom(false, b"_XKB_RULES_NAMES")
            .map_err(err)?
            .reply()
            .map_err(err)?
            .atom;
        // AddMaster installs the server's default RMLVO in the ROOT property,
        // even though it does not replace the original keyboard's map. KDE
        // reads that property as the session layout list. Preserve it atomically
        // with creation so no other client's layout change can be overwritten.
        // Never hold this server lock across map compilation, input or IBus.
        conn.grab_server().map_err(err)?.check().map_err(err)?;
        let result = (|| {
            let before = conn
                .get_property(false, self.root, atom, xproto::AtomEnum::ANY, 0, 16_384)
                .map_err(err)?
                .reply()
                .map_err(err)?;
            if before.bytes_after != 0 {
                return Err(err("session XKB metadata exceeds the bounded snapshot"));
            }
            let added = conn
                .xinput_xi_change_hierarchy(changes)
                .map_err(err)?
                .check()
                .map_err(err);
            // Restore even when the creation request reports an error. All
            // mutations since the snapshot belong to this short server lock.
            let restored = if before.type_ == 0 {
                conn.delete_property(self.root, atom)
                    .map_err(err)?
                    .check()
                    .map_err(err)
            } else {
                conn.change_property(
                    xproto::PropMode::REPLACE,
                    self.root,
                    atom,
                    before.type_,
                    before.format,
                    before.value_len,
                    &before.value,
                )
                .map_err(err)?
                .check()
                .map_err(err)
            };
            added.and(restored)
        })();
        let unlocked = conn.ungrab_server().map_err(err)?.check().map_err(err);
        result.and(unlocked)
    }

    fn hide_pointer(&self) -> io::Result<()> {
        let pix = self.conn.generate_id().map_err(err)?;
        let gc = self.conn.generate_id().map_err(err)?;
        let cursor = self.conn.generate_id().map_err(err)?;
        self.conn
            .create_pixmap(1, pix, self.root, 1, 1)
            .map_err(err)?
            .check()
            .map_err(err)?;
        self.conn
            .create_gc(gc, pix, &xproto::CreateGCAux::new().foreground(0))
            .map_err(err)?
            .check()
            .map_err(err)?;
        self.conn
            .poly_fill_rectangle(
                pix,
                gc,
                &[xproto::Rectangle {
                    x: 0,
                    y: 0,
                    width: 1,
                    height: 1,
                }],
            )
            .map_err(err)?
            .check()
            .map_err(err)?;
        self.conn
            .create_cursor(cursor, pix, pix, 0, 0, 0, 0, 0, 0, 0, 0)
            .map_err(err)?
            .check()
            .map_err(err)?;
        let result = self
            .conn
            .xinput_xi_change_cursor(self.root, cursor, self.pointer)
            .map_err(err)?
            .check()
            .map_err(err);
        let _ = self.conn.free_cursor(cursor);
        let _ = self.conn.free_gc(gc);
        let _ = self.conn.free_pixmap(pix);
        result
    }

    pub fn validate_owner(&self) -> io::Result<()> {
        if self
            .conn
            .get_selection_owner(self.selection)
            .map_err(err)?
            .reply()
            .map_err(err)?
            .owner
            != self.window
        {
            return Err(err("native replay owner was revoked"));
        }
        if self.keyboard == 0 {
            Ok(())
        } else {
            self.validate_devices()
        }
    }

    pub fn device_ids(&self) -> io::Result<Vec<u16>> {
        self.validate_owner()?;
        if self.keyboard == 0 {
            return Ok(Vec::new());
        }
        Ok(self
            .conn
            .xinput_xi_query_device(0u16)
            .map_err(err)?
            .reply()
            .map_err(err)?
            .infos
            .into_iter()
            .filter(|d| {
                d.deviceid == self.keyboard
                    || d.deviceid == self.pointer
                    || d.attachment == self.keyboard
                    || d.attachment == self.pointer
            })
            .map(|d| d.deviceid)
            .collect())
    }

    pub fn deactivate(&mut self) -> io::Result<()> {
        if self.keyboard == 0 {
            return Ok(());
        }
        self.with_owned_devices(|| {
            for code in &self.held {
                x11rb::protocol::xtest::fake_input(
                    &self.conn,
                    self.event_type + 1,
                    *code,
                    x11rb::CURRENT_TIME,
                    self.root,
                    0,
                    0,
                    self.source,
                )
                .map_err(err)?
                .check()
                .map_err(err)?;
            }
            remove(&self.conn, self.keyboard, self.core_pointer, self.core)
        })?;
        self.held.clear();
        self.maps = None;
        self.keyboard = 0;
        self.source = 0;
        self.pointer = 0;
        Ok(())
    }

    // Numeric XI IDs may be reused after device removal. Never emit releases or
    // remove a hierarchy pair merely because the old IDs still exist.
    pub fn validate_devices(&self) -> io::Result<()> {
        let devices = self
            .conn
            .xinput_xi_query_device(0u16)
            .map_err(err)?
            .reply()
            .map_err(err)?
            .infos;
        let master = devices.iter().any(|d| {
            d.deviceid == self.keyboard
                && d.type_ == DeviceType::MASTER_KEYBOARD
                && d.attachment == self.pointer
                && d.name == format!("{} keyboard", self.name).as_bytes()
        });
        let source = devices.iter().any(|d| {
            d.deviceid == u16::from(self.source)
                && d.type_ == DeviceType::SLAVE_KEYBOARD
                && d.attachment == self.keyboard
                && d.name == format!("{} XTEST keyboard", self.name).as_bytes()
        });
        let foreign = devices.iter().any(|d| {
            (d.type_ == DeviceType::SLAVE_KEYBOARD || d.type_ == DeviceType::SLAVE_POINTER)
                && (d.attachment == self.keyboard || d.attachment == self.pointer)
                && d.name != format!("{} XTEST keyboard", self.name).as_bytes()
                && d.name != format!("{} XTEST pointer", self.name).as_bytes()
        });
        if !master || !source || foreign {
            return Err(err("native replay device identity changed"));
        }
        Ok(())
    }

    pub fn with_owned_devices<T>(
        &self,
        operation: impl FnOnce() -> io::Result<T>,
    ) -> io::Result<T> {
        // Validate and address numeric XI IDs on this same connection while
        // no other client can remove/reuse them. Only finite wire operations
        // belong here: never an external process, event drain or IBus wait.
        self.conn.grab_server().map_err(err)?.check().map_err(err)?;
        let result = self.validate_owner().and_then(|()| operation());
        let unlocked = self.conn.ungrab_server().map_err(err)?.check().map_err(err);
        match (result, unlocked) {
            (Ok(value), Ok(())) => Ok(value),
            (Err(error), _) | (_, Err(error)) => Err(error),
        }
    }
}

impl Drop for ReplayMaster {
    fn drop(&mut self) {
        let _ = self.deactivate();
        let _ = self.conn.destroy_window(self.window);
        let _ = self.conn.flush();
    }
}
