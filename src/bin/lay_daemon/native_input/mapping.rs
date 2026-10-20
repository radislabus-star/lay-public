//! Validate actual maps, not XKB's master/slave copy notifications.
use super::{err, xkb, xproto, Event, RustConnection};
use std::{io, sync::Mutex};
use x11rb::connection::RequestConnection;
use x11rb::x11_utils::Serialize;

pub(super) struct Maps {
    expected: Mutex<Vec<Vec<u8>>>,
    names: Vec<Vec<u8>>,
    devices: Vec<u16>,
}

fn get_map(conn: &RustConnection, device: u16) -> io::Result<xkb::GetMapReply> {
    xkb::get_map(
        conn,
        device,
        xkb::MapPart::from(0xffu16),
        xkb::MapPart::default(),
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        xkb::VMod::default(),
        0,
        0,
        0,
        0,
        0,
        0,
    )
    .map_err(err)?
    .reply()
    .map_err(err)
}

// The caller fences generation validation and this finite wire copy on this
// same connection. An external compiler on another connection cannot share
// that fence: a removed XI ID could otherwise address a foreign successor.
pub(super) fn copy_to_owned(conn: &RustConnection, core: u16, target: u16) -> io::Result<()> {
    if core == target || target < 2 {
        return Err(err("XKB copy requires a distinct owned replay device"));
    }
    let map = get_map(conn, core)?;
    let aux = xkb::SetMapAux {
        types: map.map.types_rtrn.as_ref().map(|types| {
            types
                .iter()
                .map(|t| xkb::SetKeyType {
                    mask: t.mods_mask,
                    real_mods: t.mods_mods,
                    virtual_mods: t.mods_vmods,
                    num_levels: t.num_levels,
                    preserve: t.has_preserve,
                    entries: t
                        .map
                        .iter()
                        .map(|m| xkb::KTSetMapEntry {
                            level: m.level,
                            real_mods: m.mods_mods,
                            virtual_mods: m.mods_vmods,
                        })
                        .collect(),
                    // The generated protocol uses KTSetMapEntry for this four-byte
                    // Mods record; its first byte carries the mask, not a level.
                    preserve_entries: t
                        .preserve
                        .iter()
                        .map(|m| xkb::KTSetMapEntry {
                            level: u16::from(m.mask) as u8,
                            real_mods: m.real_mods,
                            virtual_mods: m.vmods,
                        })
                        .collect(),
                })
                .collect()
        }),
        syms: map.map.syms_rtrn.clone(),
        key_actions: map
            .map
            .key_actions
            .as_ref()
            .map(|a| xkb::SetMapAuxKeyActions {
                actions_count: a.acts_rtrn_count.clone(),
                actions: a.acts_rtrn_acts.clone(),
            }),
        behaviors: map.map.behaviors_rtrn.clone(),
        vmods: map
            .map
            .vmods_rtrn
            .as_ref()
            .map(|mods| mods.iter().map(|m| u16::from(*m) as u8).collect()),
        explicit: map.map.explicit_rtrn.clone(),
        modmap: map.map.modmap_rtrn.clone(),
        vmodmap: map.map.vmodmap_rtrn.clone(),
    };
    // Each variable-sized map component must end on a four-byte boundary.
    // The generated combined encoder misses padding after explicit/modifier
    // records. Reuse its single-component encodings (including outer padding)
    // and join them into one SetMap request. A desktop must not rebuild all its
    // shortcuts once for every intermediate component of the same owned map.
    let request = |values| xkb::SetMapRequest {
        device_spec: target,
        flags: xkb::SetMapFlags::RESIZE_TYPES,
        min_key_code: map.min_key_code,
        max_key_code: map.max_key_code,
        first_type: map.first_type,
        n_types: map.n_types,
        first_key_sym: map.first_key_sym,
        n_key_syms: map.n_key_syms,
        total_syms: map.total_syms,
        first_key_action: map.first_key_action,
        n_key_actions: map.n_key_actions,
        total_actions: map.total_actions,
        first_key_behavior: map.first_key_behavior,
        n_key_behaviors: map.n_key_behaviors,
        total_key_behaviors: map.total_key_behaviors,
        first_key_explicit: map.first_key_explicit,
        n_key_explicit: map.n_key_explicit,
        total_key_explicit: map.total_key_explicit,
        first_mod_map_key: map.first_mod_map_key,
        n_mod_map_keys: map.n_mod_map_keys,
        total_mod_map_keys: map.total_mod_map_keys,
        first_v_mod_map_key: map.first_v_mod_map_key,
        n_v_mod_map_keys: map.n_v_mod_map_keys,
        total_v_mod_map_keys: map.total_v_mod_map_keys,
        virtual_mods: map.virtual_mods,
        values: std::borrow::Cow::Owned(values),
    };
    let opcode = conn
        .extension_information(xkb::X11_EXTENSION_NAME)
        .map_err(err)?
        .ok_or_else(|| err("XKB extension disappeared"))?
        .major_opcode;
    let (full, _) = request(aux.clone()).serialize(opcode);
    let mut header = full[0].to_vec();
    let mut body = Vec::new();
    for part in [
        xkb::SetMapAux {
            types: aux.types,
            ..Default::default()
        },
        xkb::SetMapAux {
            syms: aux.syms,
            ..Default::default()
        },
        xkb::SetMapAux {
            key_actions: aux.key_actions,
            ..Default::default()
        },
        xkb::SetMapAux {
            behaviors: aux.behaviors,
            ..Default::default()
        },
        xkb::SetMapAux {
            vmods: aux.vmods,
            ..Default::default()
        },
        xkb::SetMapAux {
            explicit: aux.explicit,
            ..Default::default()
        },
        xkb::SetMapAux {
            modmap: aux.modmap,
            ..Default::default()
        },
        xkb::SetMapAux {
            vmodmap: aux.vmodmap,
            ..Default::default()
        },
    ] {
        let (encoded, _) = request(part).serialize(opcode);
        body.extend_from_slice(&encoded[1]);
        body.extend_from_slice(&encoded[2]);
    }
    let length = u16::try_from((header.len() + body.len()) / 4)
        .map_err(|_| err("owned XKB map exceeds bounded request size"))?;
    header[2..4].copy_from_slice(&length.to_ne_bytes());
    conn.send_request_without_reply(
        &[std::io::IoSlice::new(&header), std::io::IoSlice::new(&body)],
        vec![],
    )
    .map_err(err)?
    .check()
    .map_err(err)?;
    let compat = xkb::get_compat_map(conn, core, xkb::SetOfGroup::from(0x0fu8), true, 0, 0)
        .map_err(err)?
        .reply()
        .map_err(err)?;
    xkb::set_compat_map(
        conn,
        target,
        false,
        true,
        compat.groups_rtrn,
        compat.first_si_rtrn,
        &compat.si_rtrn,
        &compat.group_rtrn,
    )
    .map_err(err)?
    .check()
    .map_err(err)?;
    let c = xkb::get_controls(conn, core)
        .map_err(err)?
        .reply()
        .map_err(err)?;
    xkb::set_controls(
        conn,
        target,
        xproto::ModMask::from(0xffu16),
        c.internal_mods_real_mods,
        xproto::ModMask::from(0xffu16),
        c.ignore_lock_mods_real_mods,
        xkb::VMod::from(0xffffu16),
        c.internal_mods_vmods,
        xkb::VMod::from(0xffffu16),
        c.ignore_lock_mods_vmods,
        c.mouse_keys_dflt_btn,
        c.groups_wrap,
        c.access_x_option,
        xkb::BoolCtrl::from(0x1fffu32),
        c.enabled_controls,
        xkb::Control::from(0xf800_1fffu32),
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
    .map_err(err)?
    .check()
    .map_err(err)?;
    let names = xkb::get_names(conn, core, xkb::NameDetail::KEY_NAMES)
        .map_err(err)?
        .reply()
        .map_err(err)?;
    xkb::set_names(
        conn,
        target,
        xkb::VMod::default(),
        0,
        0,
        0,
        0,
        0,
        xkb::SetOfGroup::default(),
        0,
        names.first_key,
        names.n_keys,
        0,
        0,
        &xkb::SetNamesAux {
            key_names: names.value_list.key_names,
            ..Default::default()
        },
    )
    .map_err(err)?
    .check()
    .map_err(err)?;
    Ok(())
}

fn snapshot(conn: &RustConnection, device: u16) -> io::Result<Vec<u8>> {
    let mut map = get_map(conn, device)?;
    map.sequence = 0;
    map.device_id = 0;
    let mut bytes = map.serialize();
    bytes.extend(key_names(conn, device)?);
    bytes.extend(compatibility(conn, device)?);
    Ok(bytes)
}

fn compatibility(conn: &RustConnection, device: u16) -> io::Result<Vec<u8>> {
    let mut compat = xkb::get_compat_map(conn, device, xkb::SetOfGroup::from(0x0fu8), true, 0, 0)
        .map_err(err)?
        .reply()
        .map_err(err)?;
    compat.sequence = 0;
    compat.device_id = 0;
    let mut bytes = compat.serialize();
    let mut controls = xkb::get_controls(conn, device)
        .map_err(err)?
        .reply()
        .map_err(err)?;
    // Enabled overlays, group interpretation and accessibility controls can
    // alter a bounded key plan. Exclude only transport metadata and repeat
    // timing, which legitimately differs between independent devices.
    controls.sequence = 0;
    controls.device_id = 0;
    controls.length = 0;
    controls.repeat_delay = 0;
    controls.repeat_interval = 0;
    bytes.extend(controls.serialize());
    Ok(bytes)
}

fn key_names(conn: &RustConnection, device: u16) -> io::Result<Vec<u8>> {
    let mut names = xkb::get_names(conn, device, xkb::NameDetail::KEY_NAMES)
        .map_err(err)?
        .reply()
        .map_err(err)?;
    names.sequence = 0;
    names.device_id = 0;
    Ok(names.serialize())
}

// Compilers may compact unused type IDs. Compare each used type's semantics,
// not its numeric index, when admitting an independently compiled copy.
fn semantics(conn: &RustConnection, device: u16) -> io::Result<Vec<u8>> {
    let mut map = get_map(conn, device)?;
    let mut bytes = vec![map.min_key_code, map.max_key_code];
    let types = map
        .map
        .types_rtrn
        .take()
        .ok_or_else(|| err("missing XKB key types"))?;
    for mut key in map
        .map
        .syms_rtrn
        .take()
        .ok_or_else(|| err("missing XKB symbols"))?
    {
        let indices = key.kt_index;
        key.kt_index = [0; 4];
        bytes.extend(key.serialize());
        for index in indices.iter().take(usize::from(key.group_info & 0x0f)) {
            bytes.extend(
                types
                    .get(usize::from(*index))
                    .ok_or_else(|| err("invalid XKB type index"))?
                    .serialize(),
            );
        }
    }
    // Keep all remaining behavior-bearing map fields: actions, modifier maps,
    // virtual bindings and key behaviors. Only compiler-assigned type indexes
    // and the symbol/type data compared above are normalized away.
    map.sequence = 0;
    map.device_id = 0;
    map.length = 0;
    map.first_type = 0;
    map.n_types = 0;
    map.total_types = 0;
    map.first_key_sym = 0;
    map.n_key_syms = 0;
    map.total_syms = 0;
    // Explicit-component flags describe how a later compiler may replace a
    // field. xkbcomp makes inferred fields explicit; they do not change the
    // current key/action meaning. The full snapshot still watches these flags.
    map.map.explicit_rtrn = None;
    map.first_key_explicit = 0;
    map.n_key_explicit = 0;
    map.total_key_explicit = 0;
    bytes.extend(map.serialize());
    bytes.extend(compatibility(conn, device)?);
    let names = xkb::get_names(conn, device, xkb::NameDetail::KEY_NAMES)
        .map_err(err)?
        .reply()
        .map_err(err)?;
    bytes.push(names.first_key);
    for name in names.value_list.key_names.unwrap_or_default() {
        bytes.extend(name.name);
    }
    Ok(bytes)
}

impl Maps {
    pub fn new<const N: usize>(conn: &RustConnection, devices: [u16; N]) -> io::Result<Self> {
        let source = semantics(conn, devices[0])?;
        let mut expected = Vec::new();
        let mut names = Vec::new();
        for device in devices {
            let actual = semantics(conn, device)?;
            if actual != source {
                return Err(err(format!(
                    "X11 source/replay key semantics differ on device {device}"
                )));
            }
            expected.push(snapshot(conn, device)?);
            names.push(key_names(conn, device)?);
        }
        let maps = Self {
            expected: Mutex::new(expected),
            names,
            devices: devices.to_vec(),
        };
        maps.validate_names(conn)?;
        maps.prepare_output(conn, false)?;
        Ok(maps)
    }

    fn validate_names(&self, conn: &RustConnection) -> io::Result<()> {
        for (device, expected) in self.devices.iter().zip(&self.names) {
            if key_names(conn, *device)? != *expected {
                return Err(err(format!(
                    "X11 physical key names changed on device {device}"
                )));
            }
        }
        Ok(())
    }

    pub fn prepare_output(&self, conn: &RustConnection, held: bool) -> io::Result<()> {
        self.validate_names(conn)?;
        let current: Vec<_> = self
            .devices
            .iter()
            .map(|d| snapshot(conn, *d))
            .collect::<io::Result<_>>()?;
        let mut expected = self.expected.lock().map_err(err)?;
        if current == *expected {
            return Ok(());
        }
        if held {
            return Err(err("X11 keyboard map changed while an output key is held"));
        }
        // KDE/IBus routinely replace symbol maps when selecting a layout.
        // Physical key names are unchanged, so input remains meaningful. Align
        // only our output devices; never write the user's source/master map.
        let source = semantics(conn, self.devices[0])?;
        if semantics(conn, self.devices[1])? != source {
            return Err(err("X11 source/master maps disagree before output"));
        }
        for device in &self.devices[2..] {
            if semantics(conn, *device)? != source {
                copy_to_owned(conn, self.devices[0], *device)?;
            }
        }
        self.validate_names(conn)?;
        let accepted: Vec<_> = self
            .devices
            .iter()
            .map(|d| snapshot(conn, *d))
            .collect::<io::Result<_>>()?;
        // Compilation can yield to another layout change. Do not publish an
        // incoherent map or retry a text frame against it.
        if accepted[..2] != current[..2] {
            return Err(err("X11 keyboard map changed while preparing output"));
        }
        for device in &self.devices {
            if semantics(conn, *device)? != source {
                return Err(err("X11 source/replay maps disagree after preparation"));
            }
        }
        *expected = accepted;
        Ok(())
    }

    pub fn validate_event(&self, conn: &RustConnection, event: &Event) -> io::Result<()> {
        let relevant = match event {
            Event::XinputHierarchy(_) | Event::Error(_) => {
                return Err(err(
                    "X11 input hierarchy/connection changed; reopen required",
                ))
            }
            Event::MappingNotify(e) => {
                e.request == xproto::Mapping::KEYBOARD || e.request == xproto::Mapping::MODIFIER
            }
            Event::XkbMapNotify(e) => self.devices.contains(&u16::from(e.device_id)),
            Event::XkbNewKeyboardNotify(e) => {
                self.devices.contains(&u16::from(e.device_id))
                    || self.devices.contains(&u16::from(e.old_device_id))
            }
            Event::XkbNamesNotify(e) => self.devices.contains(&u16::from(e.device_id)),
            _ => false,
        };
        if relevant {
            self.validate_names(conn)?;
        }
        Ok(())
    }
}
