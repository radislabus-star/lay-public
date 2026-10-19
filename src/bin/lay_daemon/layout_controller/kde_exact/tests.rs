use super::*;
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Clone, Copy)]
enum Change {
    Focus,
    Owner,
    Mapping,
    OtherGroup,
    OtherEngine,
}

#[derive(Default)]
struct Faults {
    false_group: bool,
    unchanged_group: bool,
    engine_error: bool,
    unchanged_mode: bool,
    snapshot_error: bool,
    after_group: Option<Change>,
}

struct World {
    snapshot: Snapshot,
    engine: String,
    mode: bool,
    window: String,
    mutations: Vec<String>,
    observations: Vec<&'static str>,
    faults: Faults,
}

type Fake = Rc<RefCell<World>>;

fn backend(is_ru: bool) -> Fake {
    Rc::new(RefCell::new(World {
        snapshot: Snapshot {
            owner: ":1.7".into(),
            layouts: vec![
                ("us".into(), "".into(), "English".into()),
                ("ru".into(), "".into(), "Russian".into()),
                ("de".into(), "".into(), "German".into()),
            ],
            index: u32::from(is_ru),
        },
        engine: ime_target_engine(is_ru).into(),
        mode: is_ru,
        window: "kde-window:bus:owner:lifetime:uuid".into(),
        mutations: Vec::new(),
        observations: Vec::new(),
        faults: Faults::default(),
    }))
}

fn change(world: &mut World, event: Change) {
    match event {
        Change::Focus => world.window = "another-exact-window".into(),
        Change::Owner => world.snapshot.owner = ":1.9".into(),
        Change::Mapping => world.snapshot.layouts.swap(0, 1),
        Change::OtherGroup => world.snapshot.index = 2,
        Change::OtherEngine => world.engine = "xkb:de::ger".into(),
    }
}

impl Io for Fake {
    fn window(&self, expected: &str) -> Result<(), String> {
        let mut w = self.borrow_mut();
        w.observations.push("window");
        if w.window == expected {
            Ok(())
        } else {
            Err("window changed".into())
        }
    }
    fn snapshot(&self) -> Result<Snapshot, String> {
        let mut w = self.borrow_mut();
        w.observations.push("snapshot");
        if w.faults.snapshot_error {
            Err("bus unavailable".into())
        } else {
            Ok(w.snapshot.clone())
        }
    }
    fn engine(&self) -> Result<String, String> {
        let mut w = self.borrow_mut();
        w.observations.push("engine");
        Ok(w.engine.clone())
    }
    fn mode(&self) -> Result<bool, String> {
        let mut w = self.borrow_mut();
        w.observations.push("mode");
        Ok(w.mode)
    }
    fn set_layout(&self, owner: &str, index: u32) -> Result<bool, String> {
        let mut w = self.borrow_mut();
        assert_eq!(owner, w.snapshot.owner);
        assert_eq!(w.observations.last(), Some(&"window"));
        w.mutations.push(format!("group:{index}"));
        if w.faults.false_group {
            return Ok(false);
        }
        if !w.faults.unchanged_group {
            w.snapshot.index = index;
        }
        if let Some(event) = w.faults.after_group.take() {
            change(&mut w, event);
        }
        Ok(true)
    }
    fn set_engine(&self, engine: &str) -> Result<(), String> {
        let mut w = self.borrow_mut();
        assert_eq!(w.observations.last(), Some(&"window"));
        w.mutations.push(format!("engine:{engine}"));
        if w.faults.engine_error {
            return Err("engine request failed".into());
        }
        w.engine = engine.into();
        if !w.faults.unchanged_mode {
            w.mode = engine == "lay-ime-ru";
        }
        Ok(())
    }
}

fn begin(io: &Fake, is_ru: bool) -> Result<Handoff<Fake>, String> {
    let window = io.borrow().window.clone();
    activate_on(io.clone(), is_ru, !is_ru, &window)
}

#[test]
fn both_directions_verify_actual_mode_and_restore_once_through_original_backend() {
    for initial in [false, true] {
        let io = backend(initial);
        let handoff = begin(&io, initial).unwrap();
        assert_eq!(io.borrow().snapshot.index, u32::from(!initial));
        assert_eq!(io.borrow().mode, !initial);
        assert_eq!(
            io.borrow().mutations,
            [
                format!("group:{}", u32::from(!initial)),
                format!("engine:{}", ime_target_engine(!initial))
            ]
        );
        handoff.restore_once().unwrap();
        assert_eq!(io.borrow().snapshot.index, u32::from(initial));
        assert_eq!(io.borrow().engine, ime_target_engine(initial));
        assert_eq!(io.borrow().mode, initial);
        let mutations = io.borrow().mutations.clone();
        assert_eq!(mutations.len(), 4);
        assert!(handoff.restore_once().is_err());
        assert_eq!(io.borrow().mutations, mutations);
    }
}

#[test]
fn mismatched_initial_group_engine_decoder_or_service_refuses_before_mutation() {
    for variant in 0..5 {
        let io = backend(true);
        {
            let mut w = io.borrow_mut();
            match variant {
                0 => w.snapshot.index = 0,
                1 => w.engine = "lay-ime-us".into(),
                2 => w.mode = false,
                3 => w.faults.snapshot_error = true,
                _ => w
                    .snapshot
                    .layouts
                    .push(("us".into(), "other".into(), "variant".into())),
            }
        }
        assert!(begin(&io, true).is_err());
        assert!(io.borrow().mutations.is_empty());
    }
    let io = backend(true);
    io.borrow_mut().snapshot.layouts.remove(0);
    assert!(begin(&io, true).is_err());
    assert!(io.borrow().mutations.is_empty());
}

#[test]
fn false_or_wrong_group_readback_never_advances_to_engine_mutation() {
    for false_reply in [false, true] {
        let io = backend(true);
        io.borrow_mut().faults.false_group = false_reply;
        io.borrow_mut().faults.unchanged_group = !false_reply;
        assert!(begin(&io, true).is_err());
        assert_eq!(io.borrow().mutations, ["group:0"]);
        assert_eq!(io.borrow().snapshot.index, 1);
        assert!(io.borrow().mode);
    }
}

#[test]
fn failed_engine_request_restores_group_without_retrying_the_request() {
    let io = backend(true);
    io.borrow_mut().faults.engine_error = true;
    assert!(begin(&io, true).is_err());
    assert_eq!(
        io.borrow().mutations,
        ["group:0", "engine:lay-ime-us", "group:1"]
    );
    assert_eq!(io.borrow().snapshot.index, 1);
    assert!(io.borrow().mode);
}

#[test]
fn engine_name_success_does_not_hide_failed_decoder_readback() {
    let io = backend(true);
    io.borrow_mut().faults.unchanged_mode = true;
    let result = begin(&io, true);
    assert!(
        result.is_err(),
        "engine name must not certify the actual decoder"
    );
    let error = result.err().unwrap();
    assert!(error.contains("actual Lay input mode"));
    assert_eq!(
        io.borrow().mutations,
        [
            "group:0",
            "engine:lay-ime-us",
            "group:1",
            "engine:lay-ime-ru"
        ]
    );
    assert_eq!(io.borrow().snapshot.index, 1);
    assert!(io.borrow().mode);
}

#[test]
fn changed_focus_owner_mapping_or_unrelated_state_blocks_forward_and_restore() {
    for event in [
        Change::Focus,
        Change::Owner,
        Change::Mapping,
        Change::OtherGroup,
        Change::OtherEngine,
    ] {
        let io = backend(true);
        io.borrow_mut().faults.after_group = Some(event);
        let error = begin(&io, true).err().unwrap();
        assert!(error.contains("unresolved"), "{error}");
        assert_eq!(io.borrow().mutations, ["group:0"]);
    }
}

#[test]
fn restoration_does_not_redirect_to_changed_focus_owner_mapping_engine_or_group() {
    for event in [
        Change::Focus,
        Change::Owner,
        Change::Mapping,
        Change::OtherGroup,
        Change::OtherEngine,
    ] {
        let io = backend(true);
        let handoff = begin(&io, true).unwrap();
        let mutations = io.borrow().mutations.clone();
        change(&mut io.borrow_mut(), event);
        assert!(handoff.restore_once().is_err());
        assert_eq!(io.borrow().mutations, mutations);
    }
}

#[test]
fn restoration_checks_false_reply_and_reports_unresolved_without_retries() {
    let io = backend(true);
    let handoff = begin(&io, true).unwrap();
    io.borrow_mut().faults.false_group = true;
    assert!(handoff
        .restore_once()
        .unwrap_err()
        .contains("returned false"));
    assert_eq!(
        io.borrow().mutations,
        ["group:0", "engine:lay-ime-us", "group:1"]
    );
    assert!(handoff.restore_once().is_err());
}
