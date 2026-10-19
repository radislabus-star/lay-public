use std::cell::{Cell, RefCell};
use std::collections::VecDeque;

use super::*;

struct Mock {
    snapshots: RefCell<VecDeque<Snapshot>>,
    events: RefCell<Vec<String>>,
    accepted: bool,
}

impl Mock {
    fn new(snapshots: impl IntoIterator<Item = Snapshot>) -> Self {
        Self {
            snapshots: RefCell::new(snapshots.into_iter().collect()),
            events: RefCell::new(vec![]),
            accepted: true,
        }
    }

    fn finish(&self) -> Result<(), String> {
        self.events.borrow_mut().push("engine".to_string());
        Ok(())
    }

    fn mutations(&self) -> Vec<String> {
        self.events
            .borrow()
            .iter()
            .filter(|s| s.as_str() != "snapshot")
            .cloned()
            .collect()
    }
}

impl Io for Mock {
    async fn snapshot(&self) -> Result<Snapshot, String> {
        self.events.borrow_mut().push("snapshot".to_string());
        self.snapshots
            .borrow_mut()
            .pop_front()
            .ok_or("unexpected snapshot".to_string())
    }
    async fn set_layout(&self, owner: &str, index: u32) -> Result<bool, String> {
        self.events
            .borrow_mut()
            .push(format!("set:{owner}:{index}"));
        Ok(self.accepted)
    }
}

fn state(index: u32) -> Snapshot {
    Snapshot {
        owner: ":1.42".to_string(),
        layouts: vec![
            ("ru".into(), "ru".into(), "Russian".into()),
            ("us".into(), "en".into(), "English".into()),
        ],
        index,
    }
}

#[test]
fn both_directions_use_fresh_indices_and_verify_before_and_after_ibus() {
    for (ru, before, after) in [(true, 1, 0), (false, 0, 1)] {
        let io = Mock::new([state(before), state(before), state(after), state(after)]);
        assert!(async_io::block_on(complete(&io, ru, || true, || io.finish())).is_ok());
        assert_eq!(
            *io.events.borrow(),
            [
                "snapshot",
                "snapshot",
                &format!("set::1.42:{after}"),
                "snapshot",
                "engine",
                "snapshot"
            ]
        );
        assert!(io.snapshots.borrow().is_empty());
    }
}

#[test]
fn already_target_never_sets_group_but_completes_engine() {
    let io = Mock::new([state(0), state(0), state(0), state(0)]);
    assert!(async_io::block_on(complete(&io, true, || true, || io.finish())).is_ok());
    assert_eq!(io.mutations(), ["engine"]);
}

#[test]
fn missing_ambiguous_or_unbounded_mapping_never_mutates() {
    let mut missing = state(1);
    missing.layouts[0].0 = "de".into();
    let mut duplicate = state(1);
    duplicate.layouts.push(duplicate.layouts[0].clone());
    let mut large = state(1);
    large.layouts[0].2 = "x".repeat(8193);
    let mut count = state(1);
    count.layouts.resize(33, count.layouts[1].clone());
    let mut bad_index = state(1);
    bad_index.index = 2;
    let mut empty = state(0);
    empty.layouts.clear();
    for initial in [missing, duplicate, large, count, bad_index, empty] {
        let io = Mock::new([initial]);
        assert!(async_io::block_on(complete(&io, true, || true, || io.finish())).is_err());
        assert!(io.mutations().is_empty());
    }
}

#[test]
fn service_remap_or_independent_group_change_before_mutation_refuses() {
    let mut owner = state(1);
    owner.owner = ":1.43".into();
    let mut mapping = state(1);
    mapping.layouts.reverse();
    for changed in [owner, mapping, state(0)] {
        let io = Mock::new([state(1), changed]);
        assert!(async_io::block_on(complete(&io, true, || true, || io.finish())).is_err());
        assert!(io.mutations().is_empty());
    }
}

#[test]
fn false_set_or_wrong_readback_never_completes_engine() {
    let mut rejected = Mock::new([state(1), state(1)]);
    rejected.accepted = false;
    assert!(async_io::block_on(complete(&rejected, true, || true, || rejected.finish())).is_err());
    assert_eq!(rejected.mutations(), ["set::1.42:0"]);
    let mut owner = state(0);
    owner.owner = ":1.43".into();
    let mut mapping = state(0);
    mapping.layouts.reverse();
    for readback in [state(1), owner, mapping] {
        let io = Mock::new([state(1), state(1), readback]);
        assert!(async_io::block_on(complete(&io, true, || true, || io.finish())).is_err());
        assert_eq!(io.mutations(), ["set::1.42:0"]);
    }
}

#[test]
fn stale_context_or_generation_at_each_boundary_never_retries_or_restores() {
    for fail_at in 0..5 {
        let io = Mock::new([state(1), state(1), state(0), state(0)]);
        let checks = Cell::new(0);
        let result = async_io::block_on(complete(
            &io,
            true,
            || {
                let n = checks.get();
                checks.set(n + 1);
                n != fail_at
            },
            || io.finish(),
        ));
        assert!(result.is_err(), "boundary={fail_at}");
        let expected = match fail_at {
            0 | 1 => vec![],
            2 => vec!["set::1.42:0"],
            _ => vec!["set::1.42:0", "engine"],
        };
        assert_eq!(io.mutations(), expected, "boundary={fail_at}");
    }
}

#[test]
fn engine_failure_and_final_group_mismatch_are_real_failures() {
    let io = Mock::new([state(1), state(1), state(0)]);
    assert!(
        async_io::block_on(complete(&io, true, || true, || Err("IBus refused".into()))).is_err()
    );
    assert_eq!(io.mutations(), ["set::1.42:0"]);
    let io = Mock::new([state(1), state(1), state(0), state(1)]);
    assert!(async_io::block_on(complete(&io, true, || true, || io.finish())).is_err());
    assert_eq!(io.mutations(), ["set::1.42:0", "engine"]);
}

#[test]
fn expired_deadline_does_not_poll_or_promote_work() {
    let polled = Cell::new(false);
    let result = bounded(Instant::now(), async {
        polled.set(true);
        Ok::<_, String>(())
    });
    assert!(result.is_err());
    assert!(!polled.get());
}

#[test]
fn one_deadline_bounds_a_pending_backend_future() {
    let result = bounded(
        Instant::now() + Duration::from_millis(1),
        std::future::pending::<()>(),
    );
    assert!(result.is_err());
}
