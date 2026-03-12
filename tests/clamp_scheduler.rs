use std::sync::mpsc;
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
