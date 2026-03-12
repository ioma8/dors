#[cfg(target_os = "macos")]
use dors::native_app::system_dock::{
    dock_window_candidates_for_main_screen, measured_dock_window_height_for_main_screen,
    measured_bottom_reserved_height_for_main_screen, measured_reserved_height_for_main_screen,
    parse_tilesize_output, read_preferences_snapshot, restart_dock, set_autohide, set_tilesize,
};

#[cfg(target_os = "macos")]
use std::thread;
#[cfg(target_os = "macos")]
use std::time::Duration;

#[cfg(target_os = "macos")]
fn main() {
    if let Err(error) = run() {
        eprintln!("dock_probe error: {error}");
        std::process::exit(1);
    }
}

#[cfg(target_os = "macos")]
fn run() -> Result<(), String> {
    let snapshot = read_preferences_snapshot().map_err(|error| error.to_string())?;
    let args: Vec<String> = std::env::args().skip(1).collect();
    let sizes = if args.is_empty() {
        (24..=72).step_by(4).collect::<Vec<_>>()
    } else {
        args.iter()
            .map(|arg| arg.parse::<i32>().map_err(|error| error.to_string()))
            .collect::<Result<Vec<_>, _>>()?
    };

    set_autohide(false).map_err(|error| error.to_string())?;
    for tilesize in sizes {
        set_tilesize(tilesize).map_err(|error| error.to_string())?;
        restart_dock().map_err(|error| error.to_string())?;
        thread::sleep(Duration::from_millis(5000));
        let defaults_output = std::process::Command::new("defaults")
            .args(["read", "com.apple.dock", "tilesize"])
            .output()
            .map_err(|error| error.to_string())?;
        let defaults_tilesize = parse_tilesize_output(&String::from_utf8_lossy(&defaults_output.stdout))
            .ok_or_else(|| "failed to parse defaults tilesize readback".to_string())?;
        let reserved_height =
            measured_reserved_height_for_main_screen().map_err(|error| error.to_string())?;
        let bottom_reserved_height =
            measured_bottom_reserved_height_for_main_screen().map_err(|error| error.to_string())?;
        let top_reserved_height = reserved_height - bottom_reserved_height;
        let dock_height =
            measured_dock_window_height_for_main_screen().map_err(|error| error.to_string())?;
        let candidates =
            dock_window_candidates_for_main_screen().map_err(|error| error.to_string())?;
        println!(
            "tilesize={tilesize} defaults_tilesize={defaults_tilesize} reserved_height={reserved_height} bottom_reserved_height={bottom_reserved_height} top_reserved_height={top_reserved_height} dock_height={dock_height}"
        );
        for candidate in candidates {
            println!(
                "  dock_window owner={} layer={} width={} y={} height={}",
                candidate.owner_name, candidate.layer, candidate.width, candidate.y, candidate.height
            );
        }
    }

    if snapshot.autohide_before {
        set_autohide(true).map_err(|error| error.to_string())?;
    }
    match snapshot.tilesize_before {
        Some(value) => set_tilesize(value).map_err(|error| error.to_string())?,
        None => {}
    }
    restart_dock().map_err(|error| error.to_string())?;

    Ok(())
}

#[cfg(not(target_os = "macos"))]
fn main() {
    eprintln!("dock_probe is only available on macOS");
    std::process::exit(1);
}
