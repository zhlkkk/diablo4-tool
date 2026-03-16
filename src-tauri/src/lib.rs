mod types;
mod safety;
mod game_capture;
mod web_parser;
mod auto_applier;

pub use types::*;

use std::sync::Mutex;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let app_state = AppState::new();

    tauri::Builder::default()
        .manage(Mutex::new(app_state))
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
