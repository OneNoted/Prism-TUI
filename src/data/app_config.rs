use crate::app::SortMode;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    #[serde(default = "default_sort")]
    pub default_sort: String,
    #[serde(default = "default_true")]
    pub sort_ascending: bool,
}

fn default_true() -> bool {
    true
}

fn default_sort() -> String {
    "Last Played".to_string()
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            default_sort: default_sort(),
            sort_ascending: true,
        }
    }
}

impl AppConfig {
    pub fn config_path() -> PathBuf {
        dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("prism-tui")
            .join("config.toml")
    }

    pub fn load() -> Self {
        let path = Self::config_path();
        if path.exists() {
            match fs::read_to_string(&path) {
                Ok(content) => match toml::from_str(&content) {
                    Ok(config) => return config,
                    Err(e) => eprintln!("Warning: Failed to parse config: {}", e),
                },
                Err(e) => eprintln!("Warning: Failed to read config: {}", e),
            }
        }
        Self::default()
    }

    pub fn save(&self) {
        let path = Self::config_path();
        if let Some(parent) = path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        match toml::to_string_pretty(self) {
            Ok(content) => {
                if let Err(e) = fs::write(&path, content) {
                    eprintln!("Warning: Failed to write config: {}", e);
                }
            }
            Err(e) => eprintln!("Warning: Failed to serialize config: {}", e),
        }
    }

    pub fn default_sort_mode(&self) -> SortMode {
        match self.default_sort.as_str() {
            "Name" => SortMode::Name,
            "Playtime" => SortMode::Playtime,
            "Version" => SortMode::Version,
            "Mod Loader" => SortMode::ModLoader,
            _ => SortMode::LastPlayed,
        }
    }
}
