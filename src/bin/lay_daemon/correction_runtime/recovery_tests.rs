use super::recovered_initial_manual_toggle_target;

#[test]
fn daemon_manual_toggle_uses_recovered_initial_target_without_expanding_delete() {
    assert_eq!(
        recovered_initial_manual_toggle_target("ltkfq", "делай", 1, 5),
        "сделай"
    );
}

#[test]
fn daemon_manual_toggle_recovery_does_not_cross_multiword_tail() {
    assert_eq!(
        recovered_initial_manual_toggle_target("push ltkfq", "push делай", 2, 11),
        "push делай"
    );
}
