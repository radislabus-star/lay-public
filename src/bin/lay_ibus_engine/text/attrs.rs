use std::collections::HashMap;

use zbus::zvariant::{Structure, Value};

pub(super) fn empty() -> Vec<Value<'static>> {
    Vec::new()
}

pub(super) fn preedit(chars: u32) -> Vec<Value<'static>> {
    if chars == 0 {
        return Vec::new();
    }
    vec![
        ibus_attribute(2, 0x888888, 0, chars),
        ibus_attribute(1, 1, 0, chars),
    ]
}

fn ibus_attribute(kind: u32, value: u32, start: u32, end: u32) -> Value<'static> {
    Value::new(Structure::from((
        "IBusAttribute",
        HashMap::<String, Value<'static>>::new(),
        kind,
        value,
        start,
        end,
    )))
}
