use crate::data_lines::data_lines;

const RU_KEYBOARD_ROWS_DATA: &str = include_str!("../../data/lexicon/ru_keyboard_rows.txt");

pub fn are_ru_keyboard_neighbors(a: char, b: char) -> bool {
    let Some((row_a, col_a)) = ru_keyboard_position(a) else {
        return false;
    };
    let Some((row_b, col_b)) = ru_keyboard_position(b) else {
        return false;
    };

    row_a == row_b && col_a.abs_diff(col_b) <= 1
}

fn ru_keyboard_position(ch: char) -> Option<(usize, usize)> {
    data_lines(RU_KEYBOARD_ROWS_DATA)
        .enumerate()
        .find_map(|(row, keys)| keys.chars().position(|key| key == ch).map(|col| (row, col)))
}
