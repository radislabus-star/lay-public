//! lay — библиотечная часть. Используется и из `bin/lay` (CLI),
//! и из `bin/lay-daemon` (фоновый daemon на двойной Shift).

pub mod config;
pub mod core;
pub mod correction;
pub mod decoder;
pub mod desktop;
pub mod dict;
pub mod keyboard;
pub mod lem;
pub mod llm;
pub mod ngram;
pub mod quality;
pub mod stats;
pub mod text_backend;
pub mod text_edit;
pub mod typing_assist;
pub mod word_buffer;
pub mod x11_layout;
