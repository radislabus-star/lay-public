use super::*;

#[test]
fn maps_default_ru_en_groups() {
    assert_eq!(layout_for_group(0), Some("us"));
    assert_eq!(layout_for_group(1), Some("ru"));
    assert_eq!(layout_for_group(2), None);
    assert_eq!(group_for_layout("us"), Some(0));
    assert_eq!(group_for_layout("xkb:us::eng"), Some(0));
    assert_eq!(group_for_layout("ru"), Some(1));
    assert_eq!(group_for_layout("xkb:ru::rus"), Some(1));
    assert_eq!(group_for_layout("de"), None);
}
