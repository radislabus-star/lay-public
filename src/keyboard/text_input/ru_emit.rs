use crate::keyboard::KeyEvent;
use evdev::KeyCode;

pub(super) fn char_to_ru_key_event(ch: char) -> Option<KeyEvent> {
    use KeyCode as K;
    let mut chars = ch.to_lowercase();
    let lower = chars.next()?;
    if chars.next().is_some() {
        return None;
    }
    let shift = ch.is_uppercase();
    let (key, force_shift) = match lower {
        'й' => (K::KEY_Q, false),
        'ц' => (K::KEY_W, false),
        'у' => (K::KEY_E, false),
        'к' => (K::KEY_R, false),
        'е' => (K::KEY_T, false),
        'н' => (K::KEY_Y, false),
        'г' => (K::KEY_U, false),
        'ш' => (K::KEY_I, false),
        'щ' => (K::KEY_O, false),
        'з' => (K::KEY_P, false),
        'х' => (K::KEY_LEFTBRACE, false),
        'ъ' => (K::KEY_RIGHTBRACE, false),
        'ф' => (K::KEY_A, false),
        'ы' => (K::KEY_S, false),
        'в' => (K::KEY_D, false),
        'а' => (K::KEY_F, false),
        'п' => (K::KEY_G, false),
        'р' => (K::KEY_H, false),
        'о' => (K::KEY_J, false),
        'л' => (K::KEY_K, false),
        'д' => (K::KEY_L, false),
        'ж' => (K::KEY_SEMICOLON, false),
        'э' => (K::KEY_APOSTROPHE, false),
        'я' => (K::KEY_Z, false),
        'ч' => (K::KEY_X, false),
        'с' => (K::KEY_C, false),
        'м' => (K::KEY_V, false),
        'и' => (K::KEY_B, false),
        'т' => (K::KEY_N, false),
        'ь' => (K::KEY_M, false),
        'б' => (K::KEY_COMMA, false),
        'ю' => (K::KEY_DOT, false),
        'ё' => (K::KEY_GRAVE, false),
        '1' => (K::KEY_1, false),
        '2' => (K::KEY_2, false),
        '3' => (K::KEY_3, false),
        '4' => (K::KEY_4, false),
        '5' => (K::KEY_5, false),
        '6' => (K::KEY_6, false),
        '7' => (K::KEY_7, false),
        '8' => (K::KEY_8, false),
        '9' => (K::KEY_9, false),
        '0' => (K::KEY_0, false),
        '!' => (K::KEY_1, true),
        '"' => (K::KEY_2, true),
        '№' => (K::KEY_3, true),
        ';' => (K::KEY_4, true),
        '%' => (K::KEY_5, true),
        ':' => (K::KEY_6, true),
        '?' => (K::KEY_7, true),
        '*' => (K::KEY_8, true),
        '(' => (K::KEY_9, true),
        ')' => (K::KEY_0, true),
        '-' => (K::KEY_MINUS, false),
        '_' => (K::KEY_MINUS, true),
        '=' => (K::KEY_EQUAL, false),
        '+' => (K::KEY_EQUAL, true),
        '.' => (K::KEY_SLASH, false),
        ',' => (K::KEY_SLASH, true),
        ' ' => (K::KEY_SPACE, false),
        _ => return None,
    };

    Some(KeyEvent {
        keycode: key.code(),
        shift: shift || force_shift,
        layout_is_ru: true,
    })
}
