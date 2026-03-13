use std::sync::mpsc;
use std::sync::{
    Arc, Mutex,
    atomic::{AtomicUsize, Ordering},
};
use std::time::Duration;

use dors::native_app::clamp_scheduler::ClampScheduler;

#[test]
fn clamp_scheduler_deduplicates_in_flight_work() {
    let scheduler = ClampScheduler::new();
    let (started_tx, started_rx) = mpsc::channel();
    let (release_tx, release_rx) = mpsc::channel();

    assert!(scheduler.try_schedule(move || {
        started_tx.send(()).expect("send start");
        release_rx.recv().expect("wait release");
        Ok::<_, String>(())
    }));
    started_rx
        .recv_timeout(Duration::from_secs(1))
        .expect("first job started");

    assert!(!scheduler.try_schedule(|| Ok::<_, String>(())));

    release_tx.send(()).expect("release first job");
    std::thread::sleep(Duration::from_millis(50));

    assert!(scheduler.try_schedule(|| Ok::<_, String>(())));
}

#[test]
fn clamp_scheduler_coalesces_one_follow_up_run_while_busy() {
    let scheduler = ClampScheduler::new();
    let runs = Arc::new(AtomicUsize::new(0));
    let (started_tx, started_rx) = mpsc::channel();
    let (release_tx, release_rx) = mpsc::channel();
    let release_rx = Arc::new(Mutex::new(release_rx));

    let task_runs = Arc::clone(&runs);
    let task_release_rx = Arc::clone(&release_rx);
    let task = move || {
        let current = task_runs.fetch_add(1, Ordering::SeqCst);
        started_tx.send(current).expect("send start");
        if current == 0 {
            task_release_rx
                .lock()
                .expect("lock release rx")
                .recv()
                .expect("wait release");
        }
        Ok::<_, String>(())
    };
    scheduler.schedule_coalesced(task.clone());

    assert_eq!(
        started_rx.recv_timeout(Duration::from_secs(1)).expect("first run"),
        0
    );

    scheduler.schedule_coalesced(task.clone());
    scheduler.schedule_coalesced(task);

    release_tx.send(()).expect("release first job");

    assert_eq!(
        started_rx.recv_timeout(Duration::from_secs(1)).expect("second run"),
        1
    );
    assert_eq!(runs.load(Ordering::SeqCst), 2);
}
