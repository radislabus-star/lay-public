use std::collections::HashMap;

use zbus::zvariant::{Structure, Value};

pub(super) fn empty() -> Vec<Value<'static>> {
    Vec::new()
}

pub(super) fn preedit(chars: u32, suggestion_start: u32) -> Vec<Value<'static>> {
    if chars == 0 {
        return Vec::new();
    }
    let suggestion_start = suggestion_start.min(chars);
    let mut attrs = Vec::with_capacity(3);
    if suggestion_start > 0 {
        // The owned prefix is real input. Explicit NONE also prevents clients
        // from adding their default composition underline to this span.
        attrs.push(ibus_attribute(1, 0, 0, suggestion_start));
    }
    if suggestion_start < chars {
        attrs.push(ibus_attribute(2, 0x888888, suggestion_start, chars));
        attrs.push(ibus_attribute(1, 1, suggestion_start, chars));
    }
    attrs
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
