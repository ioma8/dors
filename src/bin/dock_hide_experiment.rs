#[cfg(target_os = "macos")]
fn main() {
    use dors::private_dock_experiment::{hide_dock_visuals, query_dock_visual_targets};
    use std::sync::mpsc;

    match query_dock_visual_targets() {
        Ok(targets) => {
            eprintln!("[dock-hide-experiment] targets={}", targets.len());
            for target in &targets {
                eprintln!(
                    "[dock-hide-experiment] target id={} owner={} layer={} frame=({}, {}, {}, {}) alpha={}",
                    target.window_id,
                    target.owner_name,
                    target.layer,
                    target.x,
                    target.y,
                    target.width,
                    target.height,
                    target.alpha
                );
            }
        }
        Err(error) => {
            eprintln!("[dock-hide-experiment] failed to query targets: {error}");
            std::process::exit(1);
        }
    }

    let _guard = match hide_dock_visuals() {
        Ok(guard) => guard,
        Err(error) => {
            eprintln!("[dock-hide-experiment] failed to hide Dock visuals: {error}");
            std::process::exit(1);
        }
    };

    eprintln!("[dock-hide-experiment] Dock visuals hidden. Press Ctrl-C to restore.");

    let (sender, receiver) = mpsc::channel::<()>();
    if let Err(error) = ctrlc::set_handler(move || {
        let _ = sender.send(());
    }) {
        eprintln!("[dock-hide-experiment] failed to install signal handler: {error}");
        std::process::exit(1);
    }

    let _ = receiver.recv();
}

#[cfg(not(target_os = "macos"))]
fn main() {
    eprintln!("dock_hide_experiment is only supported on macOS");
    std::process::exit(1);
}
