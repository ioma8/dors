use std::sync::{
    Arc, Condvar, Mutex,
    atomic::{AtomicBool, Ordering},
};

type CoalescedTask = Arc<dyn Fn() -> Result<(), String> + Send + Sync + 'static>;

#[derive(Default)]
struct CoalescedState {
    pending: bool,
    worker_started: bool,
    task: Option<CoalescedTask>,
}

#[derive(Default)]
struct CoalescedLoop {
    state: Mutex<CoalescedState>,
    condvar: Condvar,
}

#[derive(Clone, Default)]
pub struct ClampScheduler {
    in_flight: Arc<AtomicBool>,
    rerun_requested: Arc<AtomicBool>,
    coalesced_loop: Arc<CoalescedLoop>,
}

impl std::fmt::Debug for ClampScheduler {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("ClampScheduler")
            .field("in_flight", &self.in_flight.load(Ordering::Acquire))
            .field(
                "rerun_requested",
                &self.rerun_requested.load(Ordering::Acquire),
            )
            .finish_non_exhaustive()
    }
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
        let task: CoalescedTask = Arc::new(task);
        self.ensure_coalesced_worker_started();

        let mut state = self
            .coalesced_loop
            .state
            .lock()
            .expect("lock coalesced scheduler state");
        state.task = Some(task);
        state.pending = true;
        drop(state);
        self.coalesced_loop.condvar.notify_one();
    }

    fn ensure_coalesced_worker_started(&self) {
        let mut state = self
            .coalesced_loop
            .state
            .lock()
            .expect("lock coalesced scheduler state");
        if state.worker_started {
            return;
        }
        state.worker_started = true;
        drop(state);

        let in_flight = Arc::clone(&self.in_flight);
        let rerun_requested = Arc::clone(&self.rerun_requested);
        let coalesced_loop = Arc::clone(&self.coalesced_loop);
        std::thread::spawn(move || {
            run_persistent_coalesced_loop(coalesced_loop, in_flight, rerun_requested);
        });
    }
}

fn run_persistent_coalesced_loop(
    coalesced_loop: Arc<CoalescedLoop>,
    in_flight: Arc<AtomicBool>,
    rerun_requested: Arc<AtomicBool>,
) {
    loop {
        let task = {
            let mut state = coalesced_loop
                .state
                .lock()
                .expect("lock coalesced scheduler state");
            while !state.pending {
                state = coalesced_loop
                    .condvar
                    .wait(state)
                    .expect("wait on coalesced scheduler");
            }
            state.pending = false;
            state.task.clone().expect("coalesced task should be set")
        };

        in_flight.store(true, Ordering::Release);
        rerun_requested.store(false, Ordering::Release);
        let _ = task();
        in_flight.store(false, Ordering::Release);

        let state = coalesced_loop
            .state
            .lock()
            .expect("lock coalesced scheduler state");
        if state.pending {
            rerun_requested.store(true, Ordering::Release);
        }
        drop(state);
    }
}
