use std::fs;
use std::path::PathBuf;
use serde::{Deserialize, Serialize};
use crate::tabs::{SidebarMode, SidebarPosition};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    pub sidebar_mode: SidebarMode,
    pub sidebar_position: SidebarPosition,
    pub sidebar_width: f64,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            sidebar_mode: SidebarMode::Docked,
            sidebar_position: SidebarPosition::Left,
            sidebar_width: 240.0,
        }
    }
}

pub fn get_settings_path(config_dir: Option<PathBuf>) -> PathBuf {
    let path = config_dir.unwrap_or_else(|| {
        std::env::var("LOCALAPPDATA")
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from("."))
            .join("CircleBrowser")
    });
    let _ = fs::create_dir_all(&path);
    path.join("settings.json")
}

pub fn load_settings(path: &PathBuf) -> AppSettings {
    if let Ok(content) = fs::read_to_string(path) {
        if let Ok(settings) = serde_json::from_str::<AppSettings>(&content) {
            return settings;
        }
    }
    AppSettings::default()
}

pub fn save_settings(path: &PathBuf, settings: &AppSettings) {
    if let Ok(content) = serde_json::to_string_pretty(settings) {
        let _ = fs::write(path, content);
    }
}

