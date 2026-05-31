use super::*;

#[test]
fn parses_gdbus_string_tuple() {
    assert_eq!(parse_gdbus_string("('us',)"), Some("us".to_string()));
}

#[test]
fn parses_current_layout_from_list_layouts_reply() {
    assert_eq!(
        parse_current_layout_from_list("('0:xkb:us,1:xkb:ru*',)"),
        Some("ru".to_string())
    );
}

#[test]
fn parses_gdbus_bool_tuple() {
    assert_eq!(parse_gdbus_bool("(true,)"), Some(true));
    assert_eq!(parse_gdbus_bool("(false,)"), Some(false));
    assert_eq!(parse_gdbus_bool("true"), None);
}
