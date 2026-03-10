#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    if let Err(error) = dors_tauri_lib::run() {
        eprintln!("error while running tauri application: {error}");
        std::process::exit(1);
    }
}
