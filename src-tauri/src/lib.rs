mod models;
mod profiles;
mod stitcher;

use std::sync::Mutex;
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            // Load profiles on startup
            let config = profiles::init_config(app.handle());
            app.manage(profiles::ProfileState(Mutex::new(config)));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            stitcher::run_smart_stitch,
            profiles::get_all_profiles,
            profiles::save_profile,
            profiles::delete_profile,
            profiles::set_current_profile
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}