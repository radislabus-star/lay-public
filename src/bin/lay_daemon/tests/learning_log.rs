use super::*;

#[test]
fn keeps_only_last_jsonl_lines() {
    let compacted = keep_last_jsonl_lines("a\nb\nc\nd\n", 2);
    assert_eq!(compacted, "c\nd\n");
}
