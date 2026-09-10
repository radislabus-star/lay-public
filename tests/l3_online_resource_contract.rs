const SERVICE: &str = include_str!("../systemd/lay-l3-online.service");

fn setting_values(name: &str) -> Vec<&str> {
    let prefix = format!("{name}=");
    SERVICE
        .lines()
        .filter_map(|line| line.strip_prefix(&prefix))
        .collect()
}

#[test]
fn l3_online_service_has_a_persistent_proof_cpu_budget() {
    assert_eq!(
        setting_values("Environment"),
        ["MALLOC_ARENA_MAX=2", "LAY_L3_PROOF_WORKERS=2"]
    );
    assert_eq!(setting_values("CPUQuota"), ["150%"]);
}

#[test]
fn l3_online_service_preserves_the_complete_proof_route_and_priorities() {
    assert_eq!(
        setting_values("ExecStart"),
        ["%h/.local/bin/lay-nanda-wave-train --watch-l3-context-online --poll-ms 5000"]
    );
    assert_eq!(setting_values("Restart"), ["on-failure"]);
    assert_eq!(setting_values("RestartSec"), ["5"]);
    assert_eq!(setting_values("Nice"), ["10"]);
    assert_eq!(setting_values("CPUWeight"), ["20"]);
    assert_eq!(setting_values("IOWeight"), ["20"]);
}
