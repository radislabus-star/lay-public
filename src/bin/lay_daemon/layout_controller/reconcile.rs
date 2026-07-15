use std::sync::{Arc, Condvar, Mutex, OnceLock};
use std::time::Duration;

#[derive(Debug)]
struct Request {
    target_is_ru: bool,
    ibus_engine: String,
}

struct Reconciler {
    pending: Arc<(Mutex<Option<Request>>, Condvar)>,
}

impl Reconciler {
    fn spawn() -> Self {
        let pending = Arc::new((Mutex::new(None), Condvar::new()));
        let worker_pending = Arc::clone(&pending);
        std::thread::Builder::new()
            .name("lay-layout-reconcile".to_string())
            .spawn(move || loop {
                let (lock, wake) = &*worker_pending;
                let mut slot = lock.lock().expect("layout reconcile state poisoned");
                while slot.is_none() {
                    slot = wake.wait(slot).expect("layout reconcile state poisoned");
                }
                drop(slot);
                std::thread::sleep(Duration::from_millis(90));
                let request = lock.lock().expect("layout reconcile state poisoned").take();
                if let Some(request) = request {
                    reconcile_postcondition(request);
                }
            })
            .expect("spawn layout reconciler");
        Self { pending }
    }

    fn submit(&self, request: Request) {
        let (lock, wake) = &*self.pending;
        *lock.lock().expect("layout reconcile state poisoned") = Some(request);
        wake.notify_one();
    }
}

pub(super) fn submit(target_is_ru: bool, ibus_engine: &str) {
    static RECONCILER: OnceLock<Reconciler> = OnceLock::new();
    RECONCILER.get_or_init(Reconciler::spawn).submit(Request {
        target_is_ru,
        ibus_engine: ibus_engine.to_string(),
    });
}

fn reconcile_postcondition(request: Request) {
    if !super::verify::verify_gnome_shell_layout(request.target_is_ru) {
        super::log("⚠ GNOME layout postcondition was not observed after ActivateLayout");
        return;
    }
    if let Err(error) =
        super::ibus_bridge::ensure_engine(&request.ibus_engine, request.target_is_ru)
    {
        super::log(&format!(
            "⚠ delayed IME engine reconcile failed for {}: {error}",
            request.ibus_engine
        ));
        return;
    }
    if !super::verify::verify_gnome_layout_stack(request.target_is_ru) {
        super::log("⚠ GNOME/IBus layout postcondition remained inconsistent after reconcile");
    }
}
