use std::collections::HashMap;

use zbus::zvariant::{Structure, Value};

#[path = "text/attrs.rs"]
mod attrs;

pub(crate) fn make_ibus_text(text: String) -> Value<'static> {
    ibus_text(text, attrs::empty())
}

pub(crate) fn make_preedit_ibus_text(text: String) -> Value<'static> {
    let chars = text.chars().count() as u32;
    ibus_text(text, attrs::preedit(chars))
}

fn ibus_text(text: String, attrs: Vec<Value<'static>>) -> Value<'static> {
    let attrs = Structure::from((
        "IBusAttrList",
        HashMap::<String, Value<'static>>::new(),
        attrs,
    ));
    Value::new(Structure::from((
        "IBusText",
        HashMap::<String, Value<'static>>::new(),
        text,
        Value::new(attrs),
    )))
}
