#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    if let Err(error) = dors_tauri_lib::native_app::app::run() {
        eprintln!("error while running native application: {error}");
        std::process::exit(1);
    }
}
