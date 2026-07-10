use lay::architecture_contract::{
    all_contract_lines_pass, architecture_lines, architecture_tree, debt_queue,
    observed_contract_status,
};

fn main() {
    println!("TREE");
    for line in architecture_tree() {
        println!("{line}");
    }

    println!();
    println!("SCOREBOARD");
    for line in architecture_lines() {
        println!(
            "- {} [{}] owner={} proof={}",
            line.id,
            observed_contract_status(line.id).as_str(),
            line.owner,
            line.proof
        );
    }

    println!();
    println!("DEBT QUEUE");
    for line in debt_queue() {
        println!("- {line}");
    }

    println!();
    println!(
        "verdict={}",
        if all_contract_lines_pass() {
            "PASS"
        } else {
            "WATCH"
        }
    );
}
