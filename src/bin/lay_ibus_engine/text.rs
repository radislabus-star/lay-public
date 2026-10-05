use std::collections::HashMap;

use zbus::zvariant::{Structure, Value};

#[path = "text/attrs.rs"]
mod attrs;

pub(crate) fn make_ibus_text(text: String) -> Value<'static> {
    ibus_text(text, attrs::empty())
}

pub(crate) fn make_preedit_ibus_text(text: String, suggestion_start: u32) -> Value<'static> {
    let chars = text.chars().count() as u32;
    ibus_text(text, attrs::preedit(chars, suggestion_start))
}

#[cfg(test)]
pub(crate) fn preedit_attribute_geometry(text: &Value<'_>) -> Vec<(u32, u32, u32, u32)> {
    let Value::Structure(text) = text else {
        panic!("IBus text must be a structure");
    };
    let Some(Value::Value(attributes)) = text.fields().get(3) else {
        panic!("IBus text must contain an attribute-list variant");
    };
    let Value::Structure(attributes) = attributes.as_ref() else {
        panic!("IBus attribute list must be a structure");
    };
    let Some(Value::Array(attributes)) = attributes.fields().get(2) else {
        panic!("IBus attribute list must contain an array");
    };
    attributes.inner().iter().map(|attribute| {
        let Value::Value(attribute) = attribute else {
            panic!("IBus attribute must be a variant");
        };
        let Value::Structure(attribute) = attribute.as_ref() else {
            panic!("IBus attribute variant must contain a structure");
        };
        let [Value::Str(name), _, Value::U32(kind), Value::U32(value), Value::U32(start), Value::U32(end)] = attribute.fields() else {
            panic!("IBus attribute has unexpected fields");
        };
        assert_eq!(name.as_str(), "IBusAttribute");
        (*kind, *value, *start, *end)
    }).collect()
}

pub(crate) fn make_ibus_input_mode_property(is_ru: bool) -> Value<'static> {
    let mode = if is_ru { "RU" } else { "EN" };
    let empty_props = Structure::from((
        "IBusPropList",
        HashMap::<String, Value<'static>>::new(),
        Vec::<Value<'static>>::new(),
    ));
    Value::new(Structure::from((
        "IBusProperty",
        HashMap::<String, Value<'static>>::new(),
        "InputMode",
        0u32, // IBus.PropType.NORMAL
        make_ibus_text(mode.to_string()),
        "",
        make_ibus_text("Lay input mode".to_string()),
        true,
        false, // The mode label is informative; GNOME must not expose a no-op action.
        0u32,  // IBus.PropState.UNCHECKED
        Value::new(empty_props),
        make_ibus_text(mode.to_string()),
    )))
}

pub(crate) fn make_ibus_input_mode_properties(is_ru: bool) -> Value<'static> {
    Value::new(Structure::from((
        "IBusPropList",
        HashMap::<String, Value<'static>>::new(),
        vec![make_ibus_input_mode_property(is_ru)],
    )))
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

    #[test]
    fn input_mode_property_has_ibus_wire_shape_and_two_letter_symbol() {
        for (is_ru, expected) in [(true, "RU"), (false, "EN")] {
            let Value::Structure(list) = make_ibus_input_mode_properties(is_ru) else {
                panic!("IBus property list must be a structure");
            };
            let [Value::Str(name), _, Value::Array(properties)] = list.fields() else {
                panic!("IBus property list has unexpected fields");
            };
            assert_eq!(name.as_str(), "IBusPropList");
            assert_eq!(properties.inner().len(), 1);
            let Value::Value(property) = &properties.inner()[0] else {
                panic!("property array must contain variants");
            };
            let Value::Structure(property) = property.as_ref() else {
                panic!("property variant must contain a structure");
            };
            let fields = property.fields();
            assert_eq!(fields.len(), 12);
            assert!(matches!(&fields[2], Value::Str(key) if key.as_str() == "InputMode"));
            assert!(matches!(&fields[3], Value::U32(0)));
            assert!(matches!(&fields[7], Value::Bool(true)));
            assert!(matches!(&fields[8], Value::Bool(false)));
            let Value::Value(symbol) = &fields[11] else {
                panic!("InputMode symbol must be a variant");
            };
            let Value::Structure(symbol) = symbol.as_ref() else {
                panic!("InputMode symbol variant must contain IBusText");
            };
            assert!(matches!(&symbol.fields()[2], Value::Str(text) if text.as_str() == expected));
        }
    }

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
        let Value::Structure(text) = make_preedit_ibus_text("ерка".to_string(), 0) else {
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

    #[test]
    fn preedit_visual_ranges_exclude_typed_prefix_in_unicode_scalar_units() {
        for (text, start, expected) in [
            (
                "почему",
                2,
                vec![(1, 0, 0, 2), (2, 0x888888, 2, 6), (1, 1, 2, 6)],
            ),
            (
                "a𐐀бв",
                2,
                vec![(1, 0, 0, 2), (2, 0x888888, 2, 4), (1, 1, 2, 4)],
            ),
            ("проверка", 8, vec![(1, 0, 0, 8)]),
            ("проверка", u32::MAX, vec![(1, 0, 0, 8)]),
            ("", 0, vec![]),
        ] {
            let value = make_preedit_ibus_text(text.to_string(), start);
            assert_eq!(preedit_attribute_geometry(&value), expected, "{text:?}");
        }
    }
}
