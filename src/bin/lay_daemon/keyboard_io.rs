use evdev::{uinput::VirtualDevice, Device, EventType, InputEvent, KeyCode};
use lay::keyboard::is_typing_key;
use std::sync::{Arc, Mutex, MutexGuard};

use super::log;

pub(super) fn lock_virtual_keyboard(
    virtual_kbd: &Arc<Mutex<Option<VirtualDevice>>>,
) -> MutexGuard<'_, Option<VirtualDevice>> {
    match virtual_kbd.lock() {
        Ok(guard) => guard,
        Err(poisoned) => {
            log("⚠ virtual keyboard mutex poisoned; recovering shared uinput device");
            poisoned.into_inner()
        }
    }
}

pub(super) fn has_later_typing_press(events: &[InputEvent], current_index: usize) -> bool {
    events.iter().skip(current_index + 1).any(|event| {
        event.event_type() == EventType::KEY
            && event.value() == 1
            && is_typing_key(KeyCode::new(event.code()))
    })
}

pub(super) fn find_all_keyboards() -> std::io::Result<Vec<std::path::PathBuf>> {
    let mut found = Vec::new();
    for entry in std::fs::read_dir("/dev/input")? {
        let entry = entry?;
        let path = entry.path();
        if !path
            .file_name()
            .and_then(|n| n.to_str())
            .is_some_and(|s| s.starts_with("event"))
        {
            continue;
        }
        if let Ok(dev) = Device::open(&path) {
            if let Some(keys) = dev.supported_keys() {
                if keys.contains(KeyCode::KEY_LEFTSHIFT) && keys.contains(KeyCode::KEY_A) {
                    // НЕ слушаем наши/служебные uinput-устройства: это не железная
                    // клавиатура, а источник фантомных повторов в VM/desktop-тестах.
                    let name = dev.name().unwrap_or("").to_string();
                    if should_ignore_keyboard_device_name(&name) {
                        continue;
                    }
                    found.push(path);
                }
            }
        }
    }
    if found.is_empty() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "клавиатура не найдена. Возможно нет группы input — проверь `id`",
        ));
    }
    Ok(found)
}

pub(super) fn should_ignore_keyboard_device_name(name: &str) -> bool {
    matches!(name, "lay-virtual-keyboard" | "ydotoold virtual device")
}
