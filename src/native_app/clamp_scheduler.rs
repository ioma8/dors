use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};

#[derive(Clone, Debug, Default)]
pub struct ClampScheduler {
    in_flight: Arc<AtomicBool>,
    rerun_requested: Arc<AtomicBool>,
}

impl ClampScheduler {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn try_schedule<F>(&self, task: F) -> bool
    where
        F: FnOnce() -> Result<(), String> + Send + 'static,
    {
        if self.in_flight.swap(true, Ordering::AcqRel) {
            return false;
        }

        let in_flight = Arc::clone(&self.in_flight);
        std::thread::spawn(move || {
            let _ = task();
            in_flight.store(false, Ordering::Release);
        });
        true
    }

    pub fn schedule_coalesced<F>(&self, task: F)
    where
        F: Fn() -> Result<(), String> + Send + Sync + 'static,
    {
        let task = Arc::new(task);
        if self.in_flight.swap(true, Ordering::AcqRel) {
            self.rerun_requested.store(true, Ordering::Release);
            return;
        }

        let in_flight = Arc::clone(&self.in_flight);
        let rerun_requested = Arc::clone(&self.rerun_requested);
        std::thread::spawn(move || {
            run_coalesced_loop(task, in_flight, rerun_requested);
        });
    }
}

fn run_coalesced_loop<F>(
    task: Arc<F>,
    in_flight: Arc<AtomicBool>,
    rerun_requested: Arc<AtomicBool>,
) where
    F: Fn() -> Result<(), String> + Send + Sync + 'static,
{
    loop {
        let _ = task();
        if !rerun_requested.swap(false, Ordering::AcqRel) {
            in_flight.store(false, Ordering::Release);
            if rerun_requested.swap(false, Ordering::AcqRel)
                && !in_flight.swap(true, Ordering::AcqRel)
            {
                continue;
            }
            break;
        }
    }
}
