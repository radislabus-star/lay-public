use super::*;

#[test]
fn keeps_only_last_jsonl_lines() {
    let compacted = keep_last_jsonl_lines("a\nb\nc\nd\n", 2);
    assert_eq!(compacted, "c\nd\n");
}

#[test]
fn keeps_jsonl_tail_inside_byte_limit() {
    let compacted = keep_jsonl_tail_bytes("one\ntwo\nthree\nfour\n", 11);
    assert_eq!(compacted, "three\nfour\n");

    let oversized_line = "ж".repeat(100);
    let compacted = keep_jsonl_tail_bytes(&oversized_line, 11);
    assert!(compacted.len() <= 11);
    assert!(compacted.is_char_boundary(compacted.len()));
}
