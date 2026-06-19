use super::cell32::{NandaCell32, SymbolStimulus, DEFAULT_TOP_K};
use super::mode::ModeRole;
use super::options::WaveOptions;
use super::signal::WavePacket;

const UTF8: NandaCell32 = NandaCell32::new("Utf8Cell32", ModeRole::Utf8, 0x01);
const SCRIPT: NandaCell32 = NandaCell32::new("ScriptCell32", ModeRole::Script, 0x02);
const KEYBOARD: NandaCell32 = NandaCell32::new("KeyboardCell32", ModeRole::Keyboard, 0x03);
const BOUNDARY: NandaCell32 = NandaCell32::new("BoundaryCell32", ModeRole::Boundary, 0x04);

pub fn run_l1(text: &str) -> Vec<WavePacket> {
    run_l1_with_options(text, &WaveOptions::default())
}

pub fn run_l1_with_options(text: &str, options: &WaveOptions) -> Vec<WavePacket> {
    let chars = text.chars().collect::<Vec<_>>();
    let mut packets = Vec::new();
    for (idx, ch) in chars.iter().copied().enumerate() {
        let stimulus = SymbolStimulus {
            ch,
            prev: idx.checked_sub(1).and_then(|prev| chars.get(prev)).copied(),
            index_in_token: index_in_token(&chars, idx),
        };
        if options.is_enabled(UTF8.id) {
            packets.push(UTF8.observe_symbol(stimulus, DEFAULT_TOP_K));
        }
        if options.is_enabled(SCRIPT.id) {
            packets.push(SCRIPT.observe_symbol(stimulus, DEFAULT_TOP_K));
        }
        if options.is_enabled(KEYBOARD.id) {
            packets.push(KEYBOARD.observe_symbol(stimulus, DEFAULT_TOP_K));
        }
        if options.is_enabled(BOUNDARY.id) {
            packets.push(BOUNDARY.observe_symbol(stimulus, DEFAULT_TOP_K));
        }
    }
    packets
}

fn index_in_token(chars: &[char], idx: usize) -> usize {
    let mut count = 0usize;
    for ch in chars[..idx].iter().rev() {
        if ch.is_whitespace() {
            break;
        }
        count += 1;
    }
    count
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn four_cells_emit_per_symbol() {
        let packets = run_l1("ab");
        assert_eq!(packets.len(), 8);
        assert!(packets
            .iter()
            .all(|packet| packet.modes.len() == DEFAULT_TOP_K));
    }
}
