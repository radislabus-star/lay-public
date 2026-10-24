use std::collections::HashMap;

use zbus::zvariant::{Structure, Value};

#[derive(Clone, Copy)]
pub(crate) struct HintStyle {
    pub(crate) foreground: u32,
    pub(crate) background: Option<u32>,
    pub(crate) auxiliary_underline: Option<u32>,
    pub(crate) preedit_underline: u32,
}

pub(super) fn empty() -> Vec<Value<'static>> {
    Vec::new()
}

pub(super) fn hint(chars: u32, style: HintStyle) -> Vec<Value<'static>> {
    if chars == 0 {
        return Vec::new();
    }
    let mut attrs = Vec::with_capacity(3);
    append_hint_attributes(&mut attrs, style, 0, chars, style.auxiliary_underline);
    attrs
}

pub(super) fn preedit(chars: u32, suggestion_start: u32, style: HintStyle) -> Vec<Value<'static>> {
    if chars == 0 {
        return Vec::new();
    }
    let suggestion_start = suggestion_start.min(chars);
    let mut attrs = Vec::with_capacity(4);
    if suggestion_start > 0 {
        // The owned prefix is real input. Request no underline for this span;
        // some clients override IBus attributes when rendering composition.
        attrs.push(ibus_attribute(1, 0, 0, suggestion_start));
    }
    if suggestion_start < chars {
        append_hint_attributes(
            &mut attrs,
            style,
            suggestion_start,
            chars,
            Some(style.preedit_underline),
        );
    }
    attrs
}

fn append_hint_attributes(
    attrs: &mut Vec<Value<'static>>,
    style: HintStyle,
    start: u32,
    end: u32,
    underline: Option<u32>,
) {
    attrs.push(ibus_attribute(2, style.foreground, start, end));
    if let Some(background) = style.background {
        attrs.push(ibus_attribute(3, background, start, end));
    }
    if let Some(underline) = underline {
        attrs.push(ibus_attribute(1, underline, start, end));
    }
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
