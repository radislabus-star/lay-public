#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TextReplacement {
    pub move_left: u32,
    pub backspaces: u32,
    pub insert: String,
    pub move_right: u32,
}
