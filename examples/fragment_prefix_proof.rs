//! Offline streaming proof of the real letter detector; no client input or writes.
//! Input/output TSV retains the historical open-prefix proof's word identities.
use lay::dict::{convert, Direction};
use lay::layout_fragments::on_letter;
use std::io::{self, BufRead};
use std::time::Instant;

fn main() {
    let cold = Instant::now();
    let _ = on_letter("ghbdtn", 'n', false);
    eprintln!("cold_ns={}", cold.elapsed().as_nanos());
    let mut times = Vec::new();
    for row in io::stdin().lock().lines() {
        let row = row.expect("proof input");
        let cells: Vec<_> = row.split('\t').collect();
        assert_eq!(cells.len(), 3);
        let (language, condition, expected) = (cells[0], cells[1], cells[2]);
        assert!(matches!(language, "ru" | "en"));
        assert!(matches!(condition, "clean" | "wrong"));
        let expected_ru = language == "ru";
        let mut current_ru = if condition == "clean" {
            expected_ru
        } else {
            !expected_ru
        };
        let keys = if expected_ru {
            convert(expected, Direction::Ru2Us)
        } else {
            expected.to_owned()
        };
        let mut text = String::new();
        let mut flips = 0;
        let mut first = 0;
        for (index, key) in keys.chars().enumerate() {
            let decoded = if current_ru {
                convert(&key.to_string(), Direction::Us2Ru)
            } else {
                key.to_string()
            };
            assert_eq!(decoded.chars().count(), 1);
            let letter = decoded.chars().next().unwrap();
            text.push(letter);
            if !letter.is_alphabetic() {
                continue;
            }
            let started = Instant::now();
            let request = on_letter(&text, letter, current_ru);
            times.push(started.elapsed().as_nanos() as u64);
            if let Some(request) = request {
                text = request.action().to_text().to_owned();
                current_ru = request.target_layout_is_ru();
                flips += 1;
                if first == 0 {
                    first = index + 1;
                }
            }
        }
        println!("{language}\t{condition}\t{expected}\t{flips}\t{first}\t{text}\t{current_ru}");
    }
    assert!(
        !times.is_empty(),
        "proof must execute the production detector"
    );
    times.sort_unstable();
    let count = times.len();
    eprintln!(
        "warm_calls={count} median_ns={} p95_ns={} max_ns={}",
        times[count / 2],
        times[((count * 95).div_ceil(100) - 1).min(count - 1)],
        times[count - 1]
    );
}
