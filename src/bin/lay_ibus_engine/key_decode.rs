use lay::keyboard::{keycode_to_ru_char, keycode_to_us_char};

use super::engine::LayIbusEngine;

impl LayIbusEngine {
    pub(super) fn physical_char(&self, keyval: u32, keycode: u32) -> Option<char> {
        let mapped = keycode.try_into().ok().and_then(|keycode| {
            if self.layout_is_ru {
                keycode_to_ru_char(keycode, self.shift_active)
            } else {
                keycode_to_us_char(keycode, self.shift_active)
            }
        });
        mapped
            .filter(|ch| is_committable_char(*ch))
            .or_else(|| x11_keysym_char(keyval))
    }

    pub(super) fn double_shift_replacement(&self, text: &str) -> String {
        lay::mixed_script_repair::repair_mixed_script(text)
            .unwrap_or_else(|| lay::dict::convert(text, lay::dict::detect_direction(text)))
    }
}

fn is_committable_char(ch: char) -> bool {
    !ch.is_control() && ch != '\u{7f}'
}

fn x11_keysym_char(keyval: u32) -> Option<char> {
    if (0x20..=0x7e).contains(&keyval) {
        return char::from_u32(keyval).filter(|ch| is_committable_char(*ch));
    }
    if (0x0100_0000..=0x0110_ffff).contains(&keyval) {
        return char::from_u32(keyval & 0x00ff_ffff).filter(|ch| is_committable_char(*ch));
    }
    if (0x0400..=0x04ff).contains(&keyval) {
        return char::from_u32(keyval).filter(|ch| is_committable_char(*ch));
    }
    x11_cyrillic_keysym_char(keyval)
}

fn x11_cyrillic_keysym_char(keyval: u32) -> Option<char> {
    let ch = match keyval {
        0x06a3 => 'ё',
        0x06b3 => 'Ё',
        0x06c0 => 'ю',
        0x06c1 => 'а',
        0x06c2 => 'б',
        0x06c3 => 'ц',
        0x06c4 => 'д',
        0x06c5 => 'е',
        0x06c6 => 'ф',
        0x06c7 => 'г',
        0x06c8 => 'х',
        0x06c9 => 'и',
        0x06ca => 'й',
        0x06cb => 'к',
        0x06cc => 'л',
        0x06cd => 'м',
        0x06ce => 'н',
        0x06cf => 'о',
        0x06d0 => 'п',
        0x06d1 => 'я',
        0x06d2 => 'р',
        0x06d3 => 'с',
        0x06d4 => 'т',
        0x06d5 => 'у',
        0x06d6 => 'ж',
        0x06d7 => 'в',
        0x06d8 => 'ь',
        0x06d9 => 'ы',
        0x06da => 'з',
        0x06db => 'ш',
        0x06dc => 'э',
        0x06dd => 'щ',
        0x06de => 'ч',
        0x06df => 'ъ',
        0x06e0 => 'Ю',
        0x06e1 => 'А',
        0x06e2 => 'Б',
        0x06e3 => 'Ц',
        0x06e4 => 'Д',
        0x06e5 => 'Е',
        0x06e6 => 'Ф',
        0x06e7 => 'Г',
        0x06e8 => 'Х',
        0x06e9 => 'И',
        0x06ea => 'Й',
        0x06eb => 'К',
        0x06ec => 'Л',
        0x06ed => 'М',
        0x06ee => 'Н',
        0x06ef => 'О',
        0x06f0 => 'П',
        0x06f1 => 'Я',
        0x06f2 => 'Р',
        0x06f3 => 'С',
        0x06f4 => 'Т',
        0x06f5 => 'У',
        0x06f6 => 'Ж',
        0x06f7 => 'В',
        0x06f8 => 'Ь',
        0x06f9 => 'Ы',
        0x06fa => 'З',
        0x06fb => 'Ш',
        0x06fc => 'Э',
        0x06fd => 'Щ',
        0x06fe => 'Ч',
        0x06ff => 'Ъ',
        _ => return None,
    };
    Some(ch)
}

#[cfg(test)]
mod tests {
    use super::LayIbusEngine;
    use lay::config::LayConfig;
    use std::sync::{Arc, Mutex};

    fn engine() -> LayIbusEngine {
        LayIbusEngine::new(
            "/test".to_string(),
            Arc::new(Mutex::new(Default::default())),
            false,
            true,
            LayConfig::default(),
        )
    }

    #[test]
    fn double_shift_replacement_repairs_mixed_script_token() {
        let engine = engine();
        assert_eq!(engine.double_shift_replacement("ghjdtрrb"), "проверки");
        assert_eq!(engine.double_shift_replacement("ghjdthrf"), "проверка");
    }

    #[test]
    fn physical_char_uses_selected_ime_engine_layout_before_client_keyval() {
        let mut engine = engine();
        assert_eq!(engine.physical_char(0x06d7, 32), Some('d'));
        engine.layout_is_ru = true;
        assert_eq!(engine.physical_char('d' as u32, 32), Some('в'));
        engine.shift_active = true;
        assert_eq!(engine.physical_char(0x06f7, 32), Some('В'));
        engine.shift_active = false;
        assert_eq!(engine.physical_char(0x06a3, 0), Some('ё'));
        assert_eq!(engine.physical_char(0x0100_0432, 32), Some('в'));
    }
}
