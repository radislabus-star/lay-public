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

#[cfg(test)]
mod tests {
    use super::*;

    fn attribute_geometry(value: &Value<'_>) -> (u32, u32, u32, u32) {
        let Value::Value(attribute) = value else {
            panic!("IBus attribute must be a variant: {value:?}");
        };
        let Value::Structure(attribute) = attribute.as_ref() else {
            panic!("IBus attribute variant must contain a structure: {attribute:?}");
        };
        let fields = attribute.fields();
        let [Value::Str(name), _, Value::U32(kind), Value::U32(value), Value::U32(start), Value::U32(end)] =
            fields
        else {
            panic!("IBus attribute has unexpected fields: {fields:?}");
        };
        assert_eq!(name.as_str(), "IBusAttribute");
        (*kind, *value, *start, *end)
    }

    #[test]
    fn retained_preedit_payload_has_exact_cursor_and_visual_attributes() {
        let Value::Structure(text) = make_preedit_ibus_text("ерка".to_string()) else {
            panic!("IBus text must be a structure");
        };
        let fields = text.fields();
        assert!(matches!(fields.get(2), Some(Value::Str(value)) if value.as_str() == "ерка"));
        let Some(Value::Value(attributes)) = fields.get(3) else {
            panic!("IBus text must contain an attribute-list variant");
        };
        let Value::Structure(attributes) = attributes.as_ref() else {
            panic!("IBus attribute list must be a structure");
        };
        let Some(Value::Array(attributes)) = attributes.fields().get(2) else {
            panic!("IBus attribute list must contain an array");
        };
        assert_eq!(
            attributes
                .inner()
                .iter()
                .map(attribute_geometry)
                .collect::<Vec<_>>(),
            [(2, 0x888888, 0, 4), (1, 1, 0, 4)]
        );
    }
}
