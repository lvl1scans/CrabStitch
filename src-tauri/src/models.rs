use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum WidthMode {
    NoEnforcement = 0,
    AutoUniform = 1,
    MatchMin = 2,
    Custom = 3,
    MatchMax = 4,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum DetectorType {
    Smart = 0,
    DirectSplit = 1,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum FillColor {
    Black = 0,
    White = 1,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct StitchSettings {
    pub input_path: String,
    pub output_path: String,
    pub output_type: String,
    pub split_height: u32,
    pub width_enforce_type: u8,
    pub custom_width: u32,
    pub sensitivity: u8,
    pub scan_step: u32,
    pub ignorable_margin: u32,
    pub batch_mode: bool,
    pub detector_type: u8,
    pub fill_color: u8,
    pub enable_post_process: bool,
    pub post_process_path: String,
    pub post_process_args: String,
}

impl Default for StitchSettings {
    fn default() -> Self {
        Self {
            input_path: "".to_string(),
            output_path: "".to_string(),
            output_type: ".png".to_string(),
            split_height: 5000,
            width_enforce_type: 1, // Auto Uniform
            custom_width: 720,
            sensitivity: 90,
            scan_step: 5,
            ignorable_margin: 5,
            batch_mode: false,
            detector_type: 0, // Smart
            fill_color: 0, // Black
            enable_post_process: false,
            post_process_path: "".to_string(),
            post_process_args: "".to_string(),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct AppConfig {
    pub current_profile: String,
    pub profiles: std::collections::HashMap<String, StitchSettings>,
}