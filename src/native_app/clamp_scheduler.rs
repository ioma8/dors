use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};

#[derive(Clone, Debug, Default)]
pub struct ClampScheduler {
    in_flight: Arc<AtomicBool>,
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
}
