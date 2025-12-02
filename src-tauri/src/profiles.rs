use crate::models::{AppConfig, StitchSettings};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;
use tauri::{AppHandle, Manager, State};

// Global state container
pub struct ProfileState(pub Mutex<AppConfig>);

fn get_config_path(app: &AppHandle) -> PathBuf {
    // Saves to %APPDATA%/com.smartstitch.app/config.json
    app.path().app_config_dir().unwrap().join("config.json")
}

pub fn init_config(app: &AppHandle) -> AppConfig {
    let path = get_config_path(app);
    
    if path.exists() {
        let content = fs::read_to_string(&path).unwrap_or_default();
        if let Ok(config) = serde_json::from_str::<AppConfig>(&content) {
            return config;
        }
    }

    // Default init
    let mut profiles = HashMap::new();
    profiles.insert("Default".to_string(), StitchSettings::default());
    
    let config = AppConfig {
        current_profile: "Default".to_string(),
        profiles,
    };
    
    // Create dir and save
    let _ = fs::create_dir_all(path.parent().unwrap());
    let _ = fs::write(path, serde_json::to_string_pretty(&config).unwrap());
    
    config
}

fn save_to_disk(app: &AppHandle, config: &AppConfig) {
    let path = get_config_path(app);
    let _ = fs::write(path, serde_json::to_string_pretty(config).unwrap());
}

// --- COMMANDS ---

#[tauri::command]
pub fn get_all_profiles(state: State<'_, ProfileState>) -> AppConfig {
    state.0.lock().unwrap().clone()
}

#[tauri::command]
pub fn save_profile(app: AppHandle, state: State<'_, ProfileState>, name: String, settings: StitchSettings) -> Result<(), String> {
    let mut config = state.0.lock().unwrap();
    config.profiles.insert(name.clone(), settings);
    config.current_profile = name;
    save_to_disk(&app, &config);
    Ok(())
}

#[tauri::command]
pub fn delete_profile(app: AppHandle, state: State<'_, ProfileState>, name: String) -> Result<AppConfig, String> {
    let mut config = state.0.lock().unwrap();
    
    if name == "Default" {
        return Err("Cannot delete Default profile".to_string());
    }

    config.profiles.remove(&name);
    
    // Fallback if current was deleted
    if config.current_profile == name {
        config.current_profile = "Default".to_string();
    }
    
    save_to_disk(&app, &config);
    Ok(config.clone())
}

#[tauri::command]
pub fn set_current_profile(app: AppHandle, state: State<'_, ProfileState>, name: String) -> Result<(), String> {
    let mut config = state.0.lock().unwrap();
    if config.profiles.contains_key(&name) {
        config.current_profile = name;
        save_to_disk(&app, &config);
        Ok(())
    } else {
        Err("Profile not found".to_string())
    }
}